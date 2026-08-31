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
    #[error(transparent)]
    Parse(#[from] crate::latex::parser::ParseError),
}
