use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn error_level_diagnostic_exits_one_and_is_reported() {
    let dir = tempdir().expect("tempdir");
    let input = dir.path().join("main.tex");
    std::fs::write(
        &input,
        "We use RAG for retrieval. This sentence is deliberately crafted to exceed the configured word limit by continuing with many extra words that keep going past the threshold.",
    )
    .expect("write input");

    let mut cmd = Command::cargo_bin("paperlint").expect("binary");
    cmd.arg(&input);
    cmd.assert()
        .failure()
        .code(1)
        .stdout(predicates::str::contains("error[ACR001]"))
        .stderr(predicates::str::is_empty());
}
