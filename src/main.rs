use clap::Parser;
use paperlint::{
    cli::{Cli, ColorChoice, OutputFormat},
    config::{DefaultConfig, Level, PaperlintConfig, RawPaperlintConfig},
    error::PaperlintError,
    latex::{parser, parser::Document, project::ProjectResolver},
    lint::{diagnostic::Diagnostic, engine::RuleEngine, registry::RuleRegistry},
    output::{human, json, path::display_path},
};
use std::{io::Write, path::Path};

struct LintResult {
    document: Document,
    diagnostics: Vec<Diagnostic>,
}

fn main() {
    let cli = Cli::parse();
    let exit_code = match execute(cli) {
        Ok(exit_code) => exit_code,
        Err(error) => {
            eprintln!("{error}");
            2
        }
    };
    std::process::exit(exit_code);
}

fn execute(cli: Cli) -> Result<i32, PaperlintError> {
    let Cli {
        input,
        config,
        format,
        color,
        enable,
        disable,
    } = cli;
    let mut result = run(input, config, enable, disable)?;
    sort_diagnostics(&mut result.diagnostics, &result.document.root);

    let output = match format {
        OutputFormat::Human => human::render(
            &result.document.sources,
            &result.document.root,
            &result.diagnostics,
            color_mode(color),
        )?,
        OutputFormat::Json => json::render(&result.diagnostics)?,
    };
    std::io::stdout()
        .lock()
        .write_all(output.as_bytes())
        .map_err(PaperlintError::OutputWrite)?;

    Ok(
        if result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Level::Error)
        {
            1
        } else {
            0
        },
    )
}

fn run(
    input: std::path::PathBuf,
    config_path: Option<std::path::PathBuf>,
    enable: Vec<String>,
    disable: Vec<String>,
) -> Result<LintResult, PaperlintError> {
    let cwd = std::env::current_dir().map_err(PaperlintError::CurrentDir)?;
    let resolver = ProjectResolver::new(cwd);
    let resolved_input = resolver.resolve(&input).map_err(PaperlintError::Project)?;
    let mut config = DefaultConfig::load();
    if let Some(path) = config_path {
        let loaded =
            std::fs::read_to_string(&path).map_err(|source| PaperlintError::ConfigRead {
                path: path.clone(),
                source,
            })?;
        let raw: RawPaperlintConfig = toml::from_str(&loaded)
            .map_err(|source| PaperlintError::ConfigParse { path, source })?;
        config = PaperlintConfig::from_raw(raw, config);
    }
    let config = config.with_overrides(&enable, &disable);
    let document = parser::parse(resolved_input, &config.latex)?;
    let engine = RuleEngine::new(RuleRegistry::default());
    let diagnostics = engine.run(&document, &config);
    Ok(LintResult {
        document,
        diagnostics,
    })
}

fn sort_diagnostics(diagnostics: &mut [Diagnostic], root: &Path) {
    diagnostics.sort_by(|left, right| {
        display_path(&left.span.file, root)
            .cmp(&display_path(&right.span.file, root))
            .then(left.span.start.cmp(&right.span.start))
            .then(left.span.end.cmp(&right.span.end))
            .then(left.rule.to_string().cmp(&right.rule.to_string()))
            .then(left.message.cmp(&right.message))
    });
}

const fn color_mode(color: ColorChoice) -> human::ColorMode {
    match color {
        ColorChoice::Auto => human::ColorMode::Auto,
        ColorChoice::Always => human::ColorMode::Always,
        ColorChoice::Never => human::ColorMode::Never,
    }
}
