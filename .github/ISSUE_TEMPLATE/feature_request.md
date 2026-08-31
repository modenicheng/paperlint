---
name: Feature request
about: Suggest a new rule or feature
title: '[FEATURE] '
labels: enhancement
assignees: ''
---

**Feature Type**
- [ ] New rule
- [ ] Improvement to existing rule
- [ ] New feature
- [ ] Documentation improvement

**Problem Description**
What writing problem does this address? Why is this feature useful?

**Proposed Solution**
Describe your proposed solution.

**For New Rules:**

**Rule ID:** [e.g., STYLE003]

**Default Severity:** [error/warning]

**Bad Example:**
```latex
% Example of what should be flagged
```

**Good Example:**
```latex
% Example of correct usage
```

**Suggested Configuration:**
```toml
[rules.STYLE003]
level = "warning"
# Add configuration options
```

**References**
- Academic style guides
- Related discussions
- Similar tools

**Additional Context**
Any other context or screenshots.

**Are you willing to implement this?**
- [ ] Yes, I can submit a PR
- [ ] No, but I can help test
- [ ] No, just suggesting
