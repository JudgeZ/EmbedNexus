# IDE Transcript Catalog

This catalog tracks the golden interaction transcripts that back the integration
documentation, client scripts, and CI coverage described in the rest of the
repository. Every subdirectory corresponds to a supported IDE integration and
contains the canonical exchanges captured while replaying the client fixtures
against the Cursor Local Embedding MCP server.

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

Once the language client scripts land, regenerate transcripts by running the
planned `clients` tooling for each transport scenario. Until those scripts are in
place, treat every transcript as **TODO** and leave placeholder notes in the IDE
subdirectories. When the tooling arrives:

1. Run the client script with the transport and prompt specified in the
   integration guides.
2. Capture the request/response envelopes into a temporary directory.
3. Normalize the envelopes (key ordering, timestamps, deterministic IDs).
4. Replace the corresponding `<scenario>.json` and update `CHECKSUMS.sha256`.
5. Commit both the JSON and compressed artifacts alongside any updated docs.

Document any deviations or tooling flags in the subdirectory README so that
future contributors can reproduce the captures precisely.
