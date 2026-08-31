use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "paperlint",
    version,
    about = "Mechanical static checks for LaTeX papers"
)]
pub struct Cli {
    pub input: PathBuf,

    #[arg(long)]
    pub config: Option<PathBuf>,

    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,

    #[arg(long = "enable")]
    pub enable: Vec<String>,

    #[arg(long = "disable")]
    pub disable: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}
