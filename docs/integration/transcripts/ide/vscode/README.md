# Visual Studio Code Transcripts

Golden transcripts captured from the VS Code MCP extension automation. Files in
this directory are normalized with
[`scripts/transcripts/normalize.py`](../../../scripts/transcripts/normalize.py)
so that CI replay jobs and documentation diffs stay stable.

## Expected Files

| Filename | Purpose |
| --- | --- |
| `stdio-noise.json` | VS Code MCP extension invoking the stdio transport with Noise framing. |
| `http-tls.json` | VS Code extension replaying HTTPS SSE through the MCP transport. |
| `wsl-bridge.json` | VS Code on Windows launching the MCP server through `wsl.exe -e`. |

Follow the shared catalog guidance for any additional transports (for example,
`websocket-noise.json` once the extension supports websockets).

## Transcript Format

VS Code transcripts follow the baseline format with optional metadata when the
extension surfaces it (for example `extensionVersion` or normalized workspace
paths). Preserve those fields when present; the normalization script keeps them
sorted after the core transcript keys.

## Regeneration Workflow

1. Capture a raw transcript for each supported transport with the VS Code helper
   (planned for `clients/node`). Example:

   ```bash
   node clients/node/index.mjs \
     --ide vscode \
     --transport stdio-noise \
     --prompt "Summarize transcript capture." \
     --record-transcript tmp/vscode-stdio-noise.raw.json
   ```

2. Normalize the capture into this directory:

   ```bash
   python scripts/transcripts/normalize.py \
     tmp/vscode-stdio-noise.raw.json \
     --output docs/integration/transcripts/ide/vscode/stdio-noise.json
   ```

3. Regenerate checksums:

   ```bash
   (cd docs/integration/transcripts/ide/vscode && sha256sum *.json > CHECKSUMS.sha256)
   ```

4. Commit the refreshed transcript and checksum (plus `.json.zst` if required).

Log any VS Code specific prerequisites here (workspace trust prompts, tunnel
requirements, etc.) so future captures remain reproducible.
