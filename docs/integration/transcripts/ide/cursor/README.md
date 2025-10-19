# Cursor IDE Transcripts

Baseline transcripts captured from the Cursor IDE bridge. Each JSON file contains
the normalized exchange envelopes produced while replaying the language client
fixtures against the Cursor Local Embedding MCP server. Use the shared
normalization tooling whenever the captures are refreshed.

## Expected Files

| Filename | Purpose |
| --- | --- |
| `stdio-noise.json` | Cursor MCP bridge calling the stdio client with Noise framing. |
| `http-tls.json` | Cursor MCP bridge replaying HTTPS SSE with the dev TLS bundle. |
| `wsl-bridge.json` | Cursor on Windows launching the MCP server inside WSL via `wsl.exe -e`. |

Add additional scenarios as Cursor gains new transport options. Follow the naming
and checksum policies defined in the catalog README.

## Transcript Format

Each transcript records the JSON-RPC envelopes exchanged between Cursor and the
MCP server. Encode them as a JSON array where each element contains:

* `direction`: `"send"` or `"receive"` from the perspective of the client script.
* `timestamp`: ISO-8601 UTC timestamp captured immediately before the envelope was
  forwarded. The normalization tool rounds to millisecond precision.
* `envelope`: The raw JSON-RPC payload as produced or received by the bridge,
  recursively sorted to stabilize object key ordering.

## Regeneration Workflow

1. Capture a raw transcript for each transport via the language client helper. Example:

   ```bash
   python clients/python/client.py \
     --ide cursor \
     --transport stdio-noise \
     --prompt "Summarize transcript capture." \
     --record-transcript tmp/cursor-stdio-noise.raw.json
   ```

2. Normalize the capture into this directory:

   ```bash
   python scripts/transcripts/normalize.py \
     tmp/cursor-stdio-noise.raw.json \
     --output docs/integration/transcripts/ide/cursor/stdio-noise.json
   ```

3. Refresh checksums:

   ```bash
   (cd docs/integration/transcripts/ide/cursor && sha256sum *.json > CHECKSUMS.sha256)
   ```

4. Commit the updated transcript and checksum (plus `.json.zst` if required).

Document Cursor-specific flags (TLS certificates, feature toggles, bridge
arguments) here whenever additional steps are needed to reproduce the capture.
