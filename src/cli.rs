use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "paperlint",
    version,
    about = "Mechanical static checks for LaTeX papers"
)]
pub struct Cli {
    /// LaTeX entry file, or `-`/omitted to read one file from stdin
    #[arg(value_name = "INPUT", default_value = "-")]
    pub input: PathBuf,

    #[arg(long)]
    pub config: Option<PathBuf>,

    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    pub format: OutputFormat,

    #[arg(long, value_enum, default_value_t = ColorChoice::Auto)]
    pub color: ColorChoice,

    #[arg(long = "enable")]
    pub enable: Vec<String>,

    #[arg(long = "disable")]
    pub disable: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    #[value(alias = "text")]
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}
