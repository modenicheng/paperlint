use serde::Serialize;
use std::{ops::Range, path::PathBuf};

/// A position in the document's reading order: block index plus byte offset
/// inside that block's logical text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReadingPosition {
    pub block: usize,
    pub byte: usize,
}

/// A logical-text range inside one block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedRange {
    pub block: usize,
    /// Exact half-open UTF-8 byte range inside `block` logical text.
    pub range: Range<usize>,
}

impl LocatedRange {
    pub const fn position(&self) -> ReadingPosition {
        ReadingPosition {
            block: self.block,
            byte: self.range.start,
        }
    }
}

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
