use serde::Serialize;
use std::{ops::Range, path::PathBuf};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMapping {
    pub logical: Range<usize>,
    pub source: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBlock {
    pub text: String,
    pub mappings: Vec<SourceMapping>,
}
