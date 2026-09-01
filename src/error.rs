use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PaperlintError {
    #[error("failed to get current directory: {0}")]
    CurrentDir(#[source] io::Error),
    #[error("failed to resolve input: {0}")]
    Project(#[source] io::Error),
    #[error("failed to read config {path}: {source}")]
    ConfigRead { path: PathBuf, source: io::Error },
    #[error("failed to parse config {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("failed to read stdin: {0}")]
    StdinRead(#[source] io::Error),
    #[error(transparent)]
    Parse(#[from] crate::latex::parser::ParseError),
    #[error("failed to render human output: {0}")]
    HumanRender(#[from] crate::output::human::HumanRenderError),
    #[error("failed to serialize JSON output: {0}")]
    JsonRender(#[from] serde_json::Error),
    #[error("failed to write output: {0}")]
    OutputWrite(#[source] io::Error),
}
