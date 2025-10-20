# Fixtures Placeholder Corpus

Use the **Regenerate Fixture Corpus** workflow to refresh this directory instead of editing files locally. The workflow follows the steps defined in the fixture regeneration plan and guarantees that placeholder assets stay aligned with the repository's test contracts.

After triggering the workflow and downloading the artifacts, record the following metadata in your tracking notes:
- Run URL for the workflow execution
- Artifact name that delivered the refreshed corpus
- Output from the checksum verification step

Refer back to `docs/testing/fixtures-plan.md` for the detailed regeneration procedure and `docs/testing/test-matrix.md` for the authoritative coverage requirements. Files here remain placeholders until the capture pipeline replaces them; continue pointing tooling at `tests/fixtures/filesystem/mock-events.yaml` for filesystem mocks until then.
