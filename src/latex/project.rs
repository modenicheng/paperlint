use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectResolver {
    root: PathBuf,
}

impl ProjectResolver {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn resolve(&self, input: &Path) -> io::Result<PathBuf> {
        let candidate = if input.is_absolute() {
            input.to_path_buf()
        } else {
            self.root.join(input)
        };
        fs::canonicalize(candidate)
    }
}

/// Resolve an include relative to the source file that requested it.
pub fn resolve_include(including_file: &Path, requested: &Path) -> io::Result<PathBuf> {
    let parent = including_file.parent().unwrap_or_else(|| Path::new("."));
    let candidate = parent.join(requested);
    if candidate.exists() {
        return fs::canonicalize(candidate);
    }
    if requested.extension().is_none() {
        let tex_candidate = candidate.with_extension("tex");
        if tex_candidate.exists() {
            return fs::canonicalize(tex_candidate);
        }
    }
    fs::canonicalize(candidate)
}
