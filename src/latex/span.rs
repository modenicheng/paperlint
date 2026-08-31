use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Span {
    pub file: PathBuf,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub path: PathBuf,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSegment {
    pub text: String,
    pub span: Span,
}
