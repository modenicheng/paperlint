use std::{
    fs,
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

    pub fn resolve(&self, input: &Path) -> std::io::Result<PathBuf> {
        let candidate = if input.is_absolute() {
            input.to_path_buf()
        } else {
            self.root.join(input)
        };
        fs::canonicalize(candidate)
    }
}
