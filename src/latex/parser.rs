use crate::{
    config::LatexConfig,
    latex::{
        project::resolve_include,
        span::{SourceFile, SourceMapping, Span, TextBlock},
    },
};
use std::{
    collections::HashSet,
    fs,
    ops::Range,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tree_sitter::Node;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("failed to resolve entry {path}: {source}")]
    Entry {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to configure tree-sitter LaTeX parser for {path}: {message}")]
    Language { path: PathBuf, message: String },
    #[error("tree-sitter parse failed for {path}")]
    TreeSitter { path: PathBuf },
    #[error("{including_file} includes {requested}, which could not be resolved: {source}")]
    Include {
        including_file: PathBuf,
        requested: PathBuf,
        source: std::io::Error,
    },
    #[error("include cycle detected: {cycle}")]
    Cycle { cycle: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub entry: PathBuf,
    pub root: PathBuf,
    pub sources: Vec<SourceFile>,
    pub blocks: Vec<TextBlock>,
}

impl Document {
    pub fn source(&self, path: &Path) -> Option<&SourceFile> {
        self.sources.iter().find(|source| source.path == path)
    }

    pub fn source_span(&self, block: &TextBlock, logical: Range<usize>) -> Option<Span> {
        if logical.start >= logical.end || logical.end > block.text.len() {
            return None;
        }
        let overlapping: Vec<&SourceMapping> = block
            .mappings
            .iter()
            .filter(|mapping| {
                mapping.logical.start < logical.end && logical.start < mapping.logical.end
            })
            .collect();
        let first = *overlapping.first()?;
        let last = *overlapping.last()?;
        if first.source.file != last.source.file
            || overlapping
                .iter()
                .any(|mapping| mapping.source.file != first.source.file)
        {
            return None;
        }

        let start = map_boundary(first, logical.start.max(first.logical.start))?;
        let end = map_boundary(last, logical.end.min(last.logical.end))?;
        let source = self.source(&first.source.file)?;
        if start > end
            || end > source.text.len()
            || !source.text.is_char_boundary(start)
            || !source.text.is_char_boundary(end)
        {
            return None;
        }
        let (line, column) = line_column(&source.text, start);
        Some(Span {
            file: source.path.clone(),
            start,
            end,
            line,
            column,
        })
    }
}

fn map_boundary(mapping: &SourceMapping, logical: usize) -> Option<usize> {
    let logical_len = mapping.logical.end.checked_sub(mapping.logical.start)?;
    let source_len = mapping.source.end.checked_sub(mapping.source.start)?;
    if logical_len == source_len {
        return mapping
            .source
            .start
            .checked_add(logical.checked_sub(mapping.logical.start)?);
    }
    if logical == mapping.logical.start {
        Some(mapping.source.start)
    } else if logical == mapping.logical.end {
        Some(mapping.source.end)
    } else {
        None
    }
}

fn line_column(text: &str, byte: usize) -> (usize, usize) {
    let prefix = &text[..byte];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let column = text[line_start..byte].chars().count() + 1;
    (line, column)
}

pub fn parse(path: PathBuf, config: &LatexConfig) -> Result<Document, ParseError> {
    let entry = fs::canonicalize(&path).map_err(|source| ParseError::Entry {
        path: path.clone(),
        source,
    })?;
    let root = entry
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| entry.clone());
    let mut loader = Loader {
        config,
        sources: Vec::new(),
        blocks: Vec::new(),
        visiting: HashSet::new(),
        loaded: HashSet::new(),
        stack: Vec::new(),
    };
    loader.load(entry.clone())?;
    Ok(Document {
        entry,
        root,
        sources: loader.sources,
        blocks: loader.blocks,
    })
}

struct Loader<'a> {
    config: &'a LatexConfig,
    sources: Vec<SourceFile>,
    blocks: Vec<TextBlock>,
    visiting: HashSet<PathBuf>,
    loaded: HashSet<PathBuf>,
    stack: Vec<PathBuf>,
}

impl Loader<'_> {
    fn load(&mut self, path: PathBuf) -> Result<(), ParseError> {
        if self.loaded.contains(&path) {
            return Ok(());
        }
        if self.visiting.contains(&path) {
            let start = self
                .stack
                .iter()
                .position(|item| item == &path)
                .unwrap_or(0);
            let mut cycle: Vec<String> = self.stack[start..]
                .iter()
                .map(|item| item.display().to_string())
                .collect();
            cycle.push(path.display().to_string());
            return Err(ParseError::Cycle {
                cycle: cycle.join(" -> "),
            });
        }

        self.visiting.insert(path.clone());
        self.stack.push(path.clone());
        let result = self.load_source(&path);
        self.stack.pop();
        self.visiting.remove(&path);
        if result.is_ok() {
            self.loaded.insert(path);
        }
        result
    }

    fn load_source(&mut self, path: &Path) -> Result<(), ParseError> {
        let text = fs::read_to_string(path).map_err(|source| ParseError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let source_index = self.sources.len();
        self.sources.push(SourceFile {
            path: path.to_path_buf(),
            text,
        });

        let mut parser = tree_sitter::Parser::new();
        let language = tree_sitter_latex::LANGUAGE.into();
        parser
            .set_language(&language)
            .map_err(|error| ParseError::Language {
                path: path.to_path_buf(),
                message: error.to_string(),
            })?;
        let tree = parser
            .parse(self.sources[source_index].text.as_bytes(), None)
            .ok_or_else(|| ParseError::TreeSitter {
                path: path.to_path_buf(),
            })?;

        let mut builder = BlockBuilder::new(path.to_path_buf());
        let source_text = self.sources[source_index].text.clone();
        self.walk(tree.root_node(), source_text.as_bytes(), path, &mut builder)?;
        builder.flush(&mut self.blocks);
        Ok(())
    }

    fn walk(
        &mut self,
        node: Node<'_>,
        source: &[u8],
        path: &Path,
        builder: &mut BlockBuilder,
    ) -> Result<(), ParseError> {
        let kind = node.kind();
        if kind == "latex_include" {
            builder.flush(&mut self.blocks);
            let path_node = node
                .child_by_field_name("path")
                .or_else(|| find_descendant_kind(node, "path"))
                .ok_or_else(|| ParseError::TreeSitter {
                    path: path.to_path_buf(),
                })?;
            let requested = PathBuf::from(node_text(path_node, source).trim_matches(['{', '}']));
            let resolved =
                resolve_include(path, &requested).map_err(|source| ParseError::Include {
                    including_file: path.to_path_buf(),
                    requested: requested.clone(),
                    source,
                })?;
            self.load(resolved)?;
            return Ok(());
        }
        if should_skip_subtree(kind) {
            return Ok(());
        }
        if kind == "generic_environment" {
            let environment = environment_name(node, source);
            if environment.as_ref().is_some_and(|name| {
                self.config
                    .ignore_environments
                    .iter()
                    .any(|item| item == name)
            }) {
                return Ok(());
            }
        }
        if is_structural_prose_node(kind) {
            return self.walk_structural_node(node, source, path, builder);
        }
        if kind == "caption" {
            return self.walk_isolated_field(node, "long", source, path, builder);
        }
        if kind == "color_reference" {
            return self.walk_optional_prose_field(node, "text", source, path, builder);
        }
        if kind == "generic_command" {
            return self.walk_generic_command(node, source, path, builder);
        }
        if kind == "begin" || kind == "end" || kind == "command_name" || kind == "path" {
            return Ok(());
        }
        if node.child_count() == 0 {
            if kind == "word" || is_retained_punctuation(kind) {
                builder.push(source, node.byte_range());
            }
            return Ok(());
        }

        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();
        let mut previous_end = None;
        let mut index = 0;
        while index < children.len() {
            let child = children[index];
            if let Some(end) = previous_end {
                builder.push_whitespace(source, end..child.start_byte(), &mut self.blocks);
            }
            self.walk(child, source, path, builder)?;
            previous_end = Some(child.end_byte());
            index += if ends_with_old_command_definition(child)
                && children
                    .get(index + 1)
                    .is_some_and(|next| next.kind() == "curly_group")
            {
                previous_end = Some(children[index + 1].end_byte());
                2
            } else {
                1
            };
        }
        Ok(())
    }

    fn walk_isolated_field(
        &mut self,
        node: Node<'_>,
        field: &str,
        source: &[u8],
        path: &Path,
        builder: &mut BlockBuilder,
    ) -> Result<(), ParseError> {
        builder.flush(&mut self.blocks);
        self.walk_optional_prose_field(node, field, source, path, builder)?;
        builder.flush(&mut self.blocks);
        Ok(())
    }

    fn walk_structural_node(
        &mut self,
        node: Node<'_>,
        source: &[u8],
        path: &Path,
        builder: &mut BlockBuilder,
    ) -> Result<(), ParseError> {
        let prose = node.child_by_field_name("text");
        builder.flush(&mut self.blocks);
        if let Some(prose) = prose {
            self.walk(prose, source, path, builder)?;
            builder.flush(&mut self.blocks);
        }

        let mut cursor = node.walk();
        let mut previous_end = prose.map(|field| field.end_byte());
        for child in node.named_children(&mut cursor) {
            if prose.is_some_and(|field| field.id() == child.id())
                || node
                    .child_by_field_name("toc")
                    .is_some_and(|field| field.id() == child.id())
            {
                continue;
            }
            if let Some(end) = previous_end {
                builder.push_whitespace(source, end..child.start_byte(), &mut self.blocks);
            }
            self.walk(child, source, path, builder)?;
            previous_end = Some(child.end_byte());
        }
        Ok(())
    }

    fn walk_optional_prose_field(
        &mut self,
        node: Node<'_>,
        field: &str,
        source: &[u8],
        path: &Path,
        builder: &mut BlockBuilder,
    ) -> Result<(), ParseError> {
        if let Some(prose) = node.child_by_field_name(field) {
            self.walk(prose, source, path, builder)?;
        }
        Ok(())
    }

    fn walk_generic_command(
        &mut self,
        node: Node<'_>,
        source: &[u8],
        path: &Path,
        builder: &mut BlockBuilder,
    ) -> Result<(), ParseError> {
        let command = node
            .named_child(0)
            .map(|child| {
                node_text(child, source)
                    .trim_start_matches('\\')
                    .to_string()
            })
            .unwrap_or_default();
        if is_non_prose_command(&command) {
            return Ok(());
        }
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor).skip(1) {
            self.walk(child, source, path, builder)?;
        }
        Ok(())
    }
}

fn ends_with_old_command_definition(node: Node<'_>) -> bool {
    if node.kind() == "old_command_definition" {
        return true;
    }
    node.named_child_count() > 0
        && node
            .named_child(node.named_child_count() - 1)
            .is_some_and(ends_with_old_command_definition)
}

fn find_descendant_kind<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    if node.kind() == kind {
        return Some(node);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if let Some(found) = find_descendant_kind(child, kind) {
            return Some(found);
        }
    }
    None
}

fn node_text<'a>(node: Node<'_>, source: &'a [u8]) -> &'a str {
    std::str::from_utf8(&source[node.byte_range()]).unwrap_or("")
}

fn environment_name(node: Node<'_>, source: &[u8]) -> Option<String> {
    let begin = node.named_child(0)?;
    let text = find_descendant_kind(begin, "text")?;
    Some(node_text(text, source).to_string())
}

fn should_skip_subtree(kind: &str) -> bool {
    matches!(
        kind,
        "comment"
            | "block_comment"
            | "comment_environment"
            | "inline_formula"
            | "displayed_equation"
            | "math_delimiter"
            | "math_environment"
            | "citation"
            | "label_definition"
            | "label_reference"
            | "new_command_definition"
            | "renew_command_definition"
            | "new_environment_definition"
            | "environment_definition"
            | "old_command_definition"
            | "let_command_definition"
            | "paired_delimiter_definition"
            | "theorem_definition"
            | "acronym_definition"
            | "acronym_reference"
            | "glossary_entry_definition"
            | "glossary_entry_reference"
            | "color_definition"
            | "color_set_definition"
            | "counter_addition"
            | "counter_declaration"
            | "counter_definition"
            | "counter_increment"
            | "counter_typesetting"
            | "counter_value"
            | "counter_within_declaration"
            | "counter_without_declaration"
            | "class_include"
            | "package_include"
            | "biblatex_include"
            | "bibliography"
            | "graphics_include"
            | "listing_environment"
            | "minted_environment"
            | "verbatim_environment"
            | "luacode_environment"
            | "pycode_environment"
            | "sageblock_environment"
            | "sagesilent_environment"
            | "asy_environment"
            | "asydef_environment"
            | "source_code"
    )
}

fn is_structural_prose_node(kind: &str) -> bool {
    matches!(
        kind,
        "part"
            | "chapter"
            | "section"
            | "subsection"
            | "subsubsection"
            | "paragraph"
            | "subparagraph"
    )
}

fn is_non_prose_command(command: &str) -> bool {
    matches!(
        command,
        "cite"
            | "citep"
            | "citet"
            | "label"
            | "ref"
            | "pageref"
            | "eqref"
            | "includegraphics"
            | "bibliography"
            | "bibliographystyle"
            | "documentclass"
            | "usepackage"
            | "newcommand"
            | "renewcommand"
            | "providecommand"
            | "newenvironment"
            | "renewenvironment"
    )
}

fn is_retained_punctuation(kind: &str) -> bool {
    matches!(
        kind,
        "." | "," | "!" | "?" | ";" | ":" | "。" | "，" | "！" | "？" | "；" | "："
    )
}

fn contains_blank_line(bytes: &[u8]) -> bool {
    let mut index = 0;
    let mut line_breaks = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\r' => {
                index += usize::from(bytes.get(index + 1) == Some(&b'\n'));
                line_breaks += 1;
            }
            b'\n' => line_breaks += 1,
            b' ' | b'\t' => {}
            _ => line_breaks = 0,
        }
        if line_breaks >= 2 {
            return true;
        }
        index += 1;
    }
    false
}

struct BlockBuilder {
    file: PathBuf,
    text: String,
    mappings: Vec<SourceMapping>,
}

impl BlockBuilder {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            text: String::new(),
            mappings: Vec::new(),
        }
    }

    fn push(&mut self, source: &[u8], range: Range<usize>) {
        if range.start >= range.end {
            return;
        }
        let fragment = match std::str::from_utf8(&source[range.clone()]) {
            Ok(fragment) => fragment,
            Err(_) => return,
        };
        let logical_start = self.text.len();
        self.text.push_str(fragment);
        let logical_end = self.text.len();
        self.mappings.push(SourceMapping {
            logical: logical_start..logical_end,
            source: Span {
                file: self.file.clone(),
                start: range.start,
                end: range.end,
                line: 0,
                column: 0,
            },
        });
    }

    fn push_whitespace(&mut self, source: &[u8], range: Range<usize>, blocks: &mut Vec<TextBlock>) {
        let bytes = &source[range.clone()];
        if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_whitespace) {
            return;
        }
        if contains_blank_line(bytes) {
            self.flush(blocks);
        } else {
            self.push(source, range);
        }
    }

    fn flush(&mut self, blocks: &mut Vec<TextBlock>) {
        if !self.text.trim().is_empty() {
            blocks.push(TextBlock {
                text: std::mem::take(&mut self.text),
                mappings: std::mem::take(&mut self.mappings),
            });
        } else {
            self.text.clear();
            self.mappings.clear();
        }
    }
}
