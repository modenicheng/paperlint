use clap::Parser;
use paperlint::{
    cli::{Cli, OutputFormat},
    config::{DefaultConfig, PaperlintConfig, RawPaperlintConfig},
    error::PaperlintError,
    latex::{parser, project::ProjectResolver},
    lint::{engine::RuleEngine, registry::RuleRegistry},
    output::{json, text},
};

fn main() {
    let cli = Cli::parse();
    let Cli {
        input,
        config,
        format,
        enable,
        disable,
    } = cli;
    let exit_code = match run(input, config, enable, disable) {
        Ok(diagnostics) => {
            if diagnostics.is_empty() {
                0
            } else {
                match format {
                    OutputFormat::Text => {
                        print!("{}", text::render(&diagnostics));
                    }
                    OutputFormat::Json => {
                        print!("{}", json::render(&diagnostics));
                    }
                }
                1
            }
        }
        Err(error) => {
            eprintln!("{error}");
            2
        }
    };
    std::process::exit(exit_code);
}

fn run(
    input: std::path::PathBuf,
    config_path: Option<std::path::PathBuf>,
    enable: Vec<String>,
    disable: Vec<String>,
) -> Result<Vec<paperlint::lint::diagnostic::Diagnostic>, PaperlintError> {
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
    let document = parser::parse(resolved_input)?;
    let engine = RuleEngine::new(RuleRegistry::default());
    Ok(engine.run(&document, &config))
}
