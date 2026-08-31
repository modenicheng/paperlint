---
name: Bug report
about: Report a bug or false positive
title: '[BUG] '
labels: bug
assignees: ''
---

**Describe the bug**
A clear and concise description of what the bug is.

**Minimal LaTeX Example**
```latex
% Paste your minimal example here
\documentclass{article}
\begin{document}
% Your example
\end{document}
```

**Expected behavior**
What you expected to happen.

**Actual behavior**
What actually happened. Include the full error message if applicable.

**Steps to Reproduce**
1. Create file with content above
2. Run `paperlint file.tex`
3. See error

**Environment**
- Paperlint version: [run `paperlint --version`]
- OS: [e.g. Ubuntu 22.04, Windows 11, macOS 14]
- Rust version: [run `rustc --version`]

**Configuration (if applicable)**
```toml
# Paste your paperlint.toml if relevant
```

**Additional context**
Add any other context about the problem here.
