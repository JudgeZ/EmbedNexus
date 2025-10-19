# C4 Architecture Assets

Maintain the repository's C4 architecture narratives and diagrams in this directory. Organize artifacts by level using the following convention:

- `level-1-context/`
- `level-2-containers/`
- `level-3-components/`
- `level-4-code/`

Each subdirectory should contain:

1. A Markdown overview describing scope, responsibilities, and design rationale for that level.
2. Mermaid diagrams (or referenced image assets) that match the Markdown narrative.
3. Cross-links back to higher and lower levels so reviewers can trace changes end-to-end.

When updating implementation details, adjust both the narrative and diagrams here and cross-reference the updates from `docs/design/overview.md` and relevant subsystem design docs.
