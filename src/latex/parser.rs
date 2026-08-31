use crate::latex::span::Span;
use std::{fs, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("tree-sitter parse failed for {path}")]
    TreeSitter { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub path: PathBuf,
    pub text: String,
    pub spans: Vec<Span>,
}

pub fn parse(path: PathBuf) -> Result<Document, ParseError> {
    let text = fs::read_to_string(&path).map_err(|source| ParseError::Read {
        path: path.clone(),
        source,
    })?;
    let span = Span {
        file: path.clone(),
        start: 0,
        end: text.len(),
        line: 1,
        column: 1,
    };
    Ok(Document {
        path,
        text,
        spans: vec![span],
    })
}
