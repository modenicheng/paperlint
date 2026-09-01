use assert_cmd::Command;
use serde_json::Value;
use std::{path::Path, process::Stdio};
use tempfile::TempDir;

fn write_project(main: &str, files: &[(&str, &str)]) -> TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("main.tex"), main).expect("write main.tex");
    for (path, contents) in files {
        let path = dir.path().join(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create source directory");
        }
        std::fs::write(path, contents).expect("write source");
    }
    dir
}

fn paperlint(project: &Path) -> Command {
    let mut command = Command::cargo_bin("paperlint").expect("paperlint binary");
    command.arg(project.join("main.tex"));
    command
}

fn stdin_stdout(mut command: Command, input: &str, expected_code: i32) -> String {
    let output = command
        .write_stdin(input)
        .output()
        .expect("run paperlint with stdin");
    assert_eq!(
        output.status.code(),
        Some(expected_code),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout is UTF-8")
}

fn stdout(command: &mut Command, expected_code: i32) -> String {
    let output = command.output().expect("run paperlint");
    assert_eq!(
        output.status.code(),
        Some(expected_code),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout is UTF-8")
}

#[test]
fn stdin_defaults_to_human_single_file_linting() {
    let command = Command::cargo_bin("paperlint").expect("paperlint binary");
    let output = stdin_stdout(
        command,
        "这是正文。这里使用 Github 作为需要统一的术语。\n",
        0,
    );

    assert!(output.contains("warning[CASE001] <stdin>:1:"));
    assert!(output.contains("use `GitHub` instead of `Github`"));
    assert!(!output.contains("No problems found"));
}

#[test]
fn empty_stdin_renders_no_problems() {
    let command = Command::cargo_bin("paperlint").expect("paperlint binary");
    assert_eq!(stdin_stdout(command, "", 0), "No problems found.\n");
}

#[test]
fn stdin_error_diagnostic_preserves_exit_one() {
    let command = Command::cargo_bin("paperlint").expect("paperlint binary");
    let output = stdin_stdout(command, "XYZ is used without a definition.\n", 1);
    assert!(output.contains("error[ACR001] <stdin>:1:1"), "{output}");
}

#[test]
fn explicit_dash_accepts_json_stdin_and_uses_virtual_source_span() {
    let mut command = Command::cargo_bin("paperlint").expect("paperlint binary");
    command.args(["-", "--format", "json"]);
    let output = stdin_stdout(command, "前文。Github 后文。\n", 0);
    let diagnostics: Value = serde_json::from_str(&output).expect("valid diagnostic JSON");
    let diagnostic = &diagnostics.as_array().expect("JSON array")[0];
    assert_eq!(diagnostic["rule"], "CASE001");
    assert_eq!(diagnostic["span"]["file"], "<stdin>");
    assert_eq!(diagnostic["span"]["line"], 1);
    assert_eq!(diagnostic["span"]["column"], 4);
}

#[test]
fn stdin_include_fails_with_a_helpful_project_error() {
    let mut command = Command::cargo_bin("paperlint").expect("paperlint binary");
    let output = command
        .write_stdin("\\input{chapter}\n")
        .output()
        .expect("run paperlint with stdin include");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("stdin input includes chapter"), "{stderr}");
    assert!(stderr.contains("input file path"), "{stderr}");
}

#[test]
fn human_main_reports_included_chinese_source_and_warning_exits_zero() {
    let project = write_project(
        "\\input{chapters/child}\n",
        &[(
            "chapters/child.tex",
            "这是默认安全的中文上下文。这里使用 Github 作为需要统一的术语。\n",
        )],
    );

    let output = stdout(
        paperlint(project.path()).args(["--format", "human", "--color", "never"]),
        0,
    );

    assert!(output.contains("warning[CASE001] chapters/child.tex:1:"));
    assert!(output.contains("use `GitHub` instead of `Github`"));
    assert!(output.contains("chapters/child.tex:1:"));
    assert!(output.contains("这里使用 Github 作为需要统一的术语。"));
    assert!(output.contains("Found 1 problem: 0 errors, 1 warning"));
    assert!(!output.contains("\u{1b}["));
}

#[test]
fn json_main_reports_child_file_byte_span_without_ansi() {
    let child = "前文。Github 后文。\n";
    let project = write_project("\\input{child}\n", &[("child.tex", child)]);

    let output = stdout(
        paperlint(project.path()).args(["--format", "json", "--color", "always"]),
        0,
    );
    assert!(!output.contains("\u{1b}["));

    let diagnostics: Value = serde_json::from_str(&output).expect("valid diagnostic JSON");
    let diagnostics = diagnostics.as_array().expect("JSON array");
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic["rule"], "CASE001");
    assert_eq!(diagnostic["severity"], "warning");
    assert_eq!(
        diagnostic["span"]["file"],
        project
            .path()
            .join("child.tex")
            .canonicalize()
            .expect("canonical child")
            .to_string_lossy()
            .as_ref()
    );
    assert_eq!(
        diagnostic["span"]["start"],
        child.find("Github").expect("term offset")
    );
    assert_eq!(
        diagnostic["span"]["end"],
        child.find("Github").expect("term offset") + "Github".len()
    );
    assert!(diagnostic.get("color").is_none());
    assert!(diagnostic.get("summary").is_none());
}

#[test]
fn human_and_legacy_text_formats_are_both_accepted() {
    let project = write_project("Github.\n", &[]);

    let human = stdout(
        paperlint(project.path()).args(["--format", "human", "--color", "never"]),
        0,
    );
    let text = stdout(
        paperlint(project.path()).args(["--format", "text", "--color", "never"]),
        0,
    );

    assert_eq!(human, text);
    assert!(human.contains("warning[CASE001]"));
}

#[test]
fn color_always_emits_ansi_and_never_or_non_tty_auto_do_not() {
    let project = write_project("Github.\n", &[]);

    let always = stdout(
        paperlint(project.path()).args(["--format", "human", "--color", "always"]),
        0,
    );
    let never = stdout(
        paperlint(project.path()).args(["--format", "human", "--color", "never"]),
        0,
    );
    let auto = stdout(paperlint(project.path()).args(["--format", "human"]), 0);

    assert!(always.contains("\u{1b}["));
    assert!(!never.contains("\u{1b}["));
    assert!(!auto.contains("\u{1b}["));
}

#[cfg(unix)]
#[test]
fn closed_stdout_is_an_operational_error() {
    let project = write_project("Github.\n", &[]);
    let mut command = std::process::Command::new(assert_cmd::cargo::cargo_bin!("paperlint"));
    command
        .arg(project.path().join("main.tex"))
        .args(["--format", "human", "--color", "never"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("spawn paperlint");
    drop(child.stdout.take().expect("piped stdout"));

    let output = child.wait_with_output().expect("wait for paperlint");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("failed to write output"));
}

#[test]
fn error_diagnostic_exits_one_and_missing_include_exits_two() {
    let error_project = write_project("XYZ is used without a definition.\n", &[]);
    let error_output = paperlint(error_project.path())
        .args(["--format", "human", "--color", "never"])
        .output()
        .expect("run error project");
    assert_eq!(error_output.status.code(), Some(1));
    assert!(error_output.stderr.is_empty());
    assert!(String::from_utf8_lossy(&error_output.stdout).contains("error[ACR001]"));

    let missing_project = write_project("\\input{missing-child}\n", &[]);
    let missing_output = paperlint(missing_project.path())
        .args(["--format", "human", "--color", "never"])
        .output()
        .expect("run missing include project");
    assert_eq!(missing_output.status.code(), Some(2));
    assert!(missing_output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&missing_output.stderr);
    assert!(stderr.contains("main.tex"));
    assert!(stderr.contains("missing-child"));
}

#[test]
fn multiple_findings_have_deterministic_human_and_json_order() {
    let project = write_project(
        "\\input{z-last}\n\\input{a-first}\n",
        &[
            ("z-last.tex", "Github.\n"),
            ("a-first.tex", "web site and Github.\n"),
        ],
    );

    let human = stdout(
        paperlint(project.path()).args(["--format", "human", "--color", "never"]),
        0,
    );
    let human_again = stdout(
        paperlint(project.path()).args(["--format", "human", "--color", "never"]),
        0,
    );
    assert_eq!(human, human_again);
    let a_position = human.find("a-first.tex").expect("a-first report");
    let z_position = human.find("z-last.tex").expect("z-last report");
    assert!(a_position < z_position);

    let json = stdout(paperlint(project.path()).args(["--format", "json"]), 0);
    let json_again = stdout(paperlint(project.path()).args(["--format", "json"]), 0);
    assert_eq!(json, json_again);
    let diagnostics: Vec<Value> = serde_json::from_str(&json).expect("valid JSON");
    let ordered_keys: Vec<_> = diagnostics
        .iter()
        .map(|diagnostic| {
            let file = Path::new(diagnostic["span"]["file"].as_str().expect("file path"));
            (
                file.file_name()
                    .expect("file name")
                    .to_string_lossy()
                    .into_owned(),
                diagnostic["span"]["start"].as_u64().expect("start"),
                diagnostic["span"]["end"].as_u64().expect("end"),
                diagnostic["rule"].as_str().expect("rule").to_string(),
                diagnostic["message"].as_str().expect("message").to_string(),
            )
        })
        .collect();
    assert_eq!(
        ordered_keys,
        [
            (
                "a-first.tex".to_string(),
                0,
                8,
                "TERM001".to_string(),
                "use `website` instead of `web site`".to_string()
            ),
            (
                "a-first.tex".to_string(),
                13,
                19,
                "CASE001".to_string(),
                "use `GitHub` instead of `Github`".to_string()
            ),
            (
                "z-last.tex".to_string(),
                0,
                6,
                "CASE001".to_string(),
                "use `GitHub` instead of `Github`".to_string()
            ),
        ]
    );
}

#[test]
fn empty_results_render_in_both_formats() {
    let project = write_project("A short ordinary sentence.\n", &[]);

    let human = stdout(
        paperlint(project.path()).args(["--format", "human", "--color", "never"]),
        0,
    );
    let json = stdout(paperlint(project.path()).args(["--format", "json"]), 0);

    assert_eq!(human, "No problems found.\n");
    assert_eq!(
        serde_json::from_str::<Value>(&json).expect("valid JSON"),
        Value::Array(Vec::new())
    );
}
