# Golden Artifact Placeholders

Use the **Regenerate Golden Artifacts** workflow whenever these transcripts or logs need to be updated; avoid making local edits that bypass the standardized capture pipeline. The workflow enforces the golden-update procedure so tests remain reproducible.

After each regeneration, capture this metadata in your notes or ticket:
- Run URL for the workflow execution
- Artifact name containing the refreshed goldens
- Output from the checksum verification step

Consult `docs/testing/fixtures-plan.md` for the step-by-step regeneration process and `docs/testing/test-matrix.md` for the authoritative expectations around golden coverage. Until real outputs replace these placeholders, they exist solely to satisfy current matrix references and checksum manifests.
