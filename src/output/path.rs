use std::path::Path;

pub fn display_path(path: &Path, root: &Path) -> String {
    normalize(path.strip_prefix(root).unwrap_or(path))
}

fn normalize(path: &Path) -> String {
    use std::path::Component;

    let mut output = String::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => {
                // Prefix text may contain native separators, including the
                // two leading separators that identify an UNC path.
                output.push_str(&normalize_text(prefix.as_os_str().to_string_lossy()));
                while output.ends_with('/') {
                    output.pop();
                }
            }
            Component::RootDir => push_root(&mut output),
            Component::CurDir => {}
            Component::ParentDir | Component::Normal(_) => {
                push_separator(&mut output);
                output.push_str(&normalize_text(component.as_os_str().to_string_lossy()));
            }
        }
    }
    output
}

fn normalize_text(text: impl AsRef<str>) -> String {
    text.as_ref().replace('\\', "/")
}

fn push_separator(output: &mut String) {
    if !output.is_empty() && !output.ends_with('/') {
        output.push('/');
    }
}

fn push_root(output: &mut String) {
    if output.is_empty() {
        output.push('/');
    } else {
        push_separator(output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_paths_use_forward_slashes() {
        assert_eq!(
            display_path(Path::new("/paper/chapters/main.tex"), Path::new("/paper")),
            "chapters/main.tex"
        );
    }

    #[cfg(windows)]
    #[test]
    fn absolute_windows_fallback_has_one_root_separator() {
        assert_eq!(
            display_path(Path::new(r"C:\outside\file.tex"), Path::new(r"D:\paper")),
            "C:/outside/file.tex"
        );
    }

    #[cfg(windows)]
    #[test]
    fn absolute_unc_fallback_normalizes_separators() {
        assert_eq!(
            display_path(
                Path::new(r"\\server\share\outside\file.tex"),
                Path::new(r"C:\paper"),
            ),
            "//server/share/outside/file.tex"
        );
    }

    #[cfg(windows)]
    #[test]
    fn absolute_verbatim_fallback_normalizes_separators() {
        assert_eq!(
            display_path(
                Path::new(r"\\?\UNC\server\share\outside\file.tex"),
                Path::new(r"C:\paper"),
            ),
            "//?/UNC/server/share/outside/file.tex"
        );
    }
}
