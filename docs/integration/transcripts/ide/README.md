# IDE Transcript Catalog

This catalog tracks the golden interaction transcripts that back the integration
documentation, client scripts, and CI coverage described in the rest of the
repository. Every subdirectory corresponds to a supported IDE integration and
contains the canonical exchanges captured while replaying the client fixtures
against the Cursor Local Embedding MCP server. Transcripts are normalized with
[`scripts/transcripts/normalize.py`](../../../scripts/transcripts/normalize.py)
so the envelopes remain deterministic across platforms.

## Directory Layout

```
ide/
  README.md
  cursor/
    README.md
    <transcripts>
  windsurf/
    README.md
    <transcripts>
  vscode/
    README.md
    <transcripts>
```

Add additional IDE directories as support expands. Keep directory names
lowercase and hyphenate multiword IDE names (e.g., `jetbrains-gateway`).

## Client Fixture Provenance

Golden request/response pairs for the language clients live under
`tests/fixtures/<language>/<transport>/`. The initial corpus for Python, Node,
and Go covers the `stdio`, `http`, and `tls` transports and mirrors the roadmap
documented in [`docs/integration/client-plan.md`](../../client-plan.md). Keep the
fixtures synchronized with that plan and reference it whenever transcripts are
regenerated via the `--update-transcripts` guard in the corresponding test
suites.

## Naming Conventions

* Store golden transcripts as prettified JSON with deterministic key ordering.
* Use lowercase filenames with hyphens to separate transport scenarios:
  `stdio-noise.json`, `http-tls.json`, `wsl-bridge.json`, etc.
* Include transport modifiers immediately after the base protocol. Examples:
  `websocket-noise.json`, `http-tls-mutual.json`.
* Keep timestamps ISO-8601 formatted and in UTC.
* When a transcript includes multiple exchanges, encode them as an array of
  envelopes (`[{"jsonrpc": ...}, ...]`).

## Compression Policy

* Commit readable `.json` files for every transcript so that docs and code
  reviews can diff them easily.
* For transcripts larger than 256 KiB, add a companion `.json.zst` file
  compressed with `zstd -19`. These compressed copies allow CI runs to download
  the fixtures quickly without bloating the repository.
* Do **not** remove the human-readable `.json` files even when the compressed
  variant is present.

## Checksum Policy

* Maintain a `CHECKSUMS.sha256` file in each IDE directory listing checksums for
  both the `.json` and `.json.zst` copies (when present).
* Regenerate the checksum file with `sha256sum *.json *.json.zst > CHECKSUMS.sha256`
  whenever transcripts change.
* CI jobs must validate these checksums before replaying transcripts to guarantee
  that the fixtures have not drifted.

## Regeneration Workflow (Pending Client Automation)

Regenerate transcripts by replaying the client automation for each supported
transport and then normalizing the capture with the transcript tooling:

1. Run the language client helper with the desired IDE + transport pairing. For
   example:

   ```bash
   python clients/python/client.py \
     --ide cursor \
     --transport stdio-noise \
     --prompt "Summarize transcript capture." \
     --record-transcript tmp/cursor-stdio-noise.raw.json
   ```

2. Canonicalize the capture:

   ```bash
   python scripts/transcripts/normalize.py \
     tmp/cursor-stdio-noise.raw.json \
     --output docs/integration/transcripts/ide/cursor/stdio-noise.json
   ```

3. Regenerate checksums from the IDE directory:

   ```bash
   (cd docs/integration/transcripts/ide/cursor && sha256sum *.json > CHECKSUMS.sha256)
   ```

4. Commit both the normalized JSON files and refreshed checksum manifest. Add a
   `.json.zst` copy if the transcript exceeds 256 KiB (see compression policy).

Document any deviations or tooling flags in the subdirectory README so that
future contributors can reproduce the captures precisely. CI workflows consume
these normalized artifacts directly, so rerun the normalization script whenever
the raw captures change.
