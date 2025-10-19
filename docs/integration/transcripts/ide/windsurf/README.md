# Windsurf Transcripts

Baseline Windsurf bridge transcripts captured with the client automation
harness. Each file is normalized with
[`scripts/transcripts/normalize.py`](../../../scripts/transcripts/normalize.py)
so CI jobs and documentation diffs share deterministic envelopes.

## Expected Files

| Filename | Purpose |
| --- | --- |
| `stdio-noise.json` | Windsurf bridge running the stdio transport with Noise framing. |
| `http-tls.json` | HTTPS SSE session captured via the Windsurf MCP bridge. |
| `wsl-bridge.json` | Windows Windsurf client tunneling into a WSL-hosted MCP server. |

Extend the catalog with additional filenames for future transports (e.g.,
`websocket-tls-mutual.json`). Apply the shared naming, compression, and checksum
policies documented at the catalog root.

## Transcript Format

Windsurf transcripts mirror the Cursor structure with optional metadata when the
bridge exposes it (for example `bridgeVersion` or Noise handshake details).
Include these extra properties when available; the normalization tool will
preserve them in sorted order after the core `direction`, `timestamp`, and
`envelope` fields.

## Regeneration Workflow

1. Capture fresh transcripts with the Windsurf automation (planned under
   `clients/node`). Example:

   ```bash
   node clients/node/index.mjs \
     --ide windsurf \
     --transport stdio-noise \
     --prompt "Summarize transcript capture." \
     --record-transcript tmp/windsurf-stdio-noise.raw.json
   ```

2. Normalize the capture into this directory:

   ```bash
   python scripts/transcripts/normalize.py \
     tmp/windsurf-stdio-noise.raw.json \
     --output docs/integration/transcripts/ide/windsurf/stdio-noise.json
   ```

3. Update checksums:

   ```bash
   (cd docs/integration/transcripts/ide/windsurf && sha256sum *.json > CHECKSUMS.sha256)
   ```

4. Commit the refreshed transcript, checksum file, and any `.json.zst` artifacts
   when the raw JSON exceeds 256 KiB.

Document Windsurf-specific toggles (TLS certificates, bridge flags, Windows path
rewrites) here whenever they are required for reproduction.
