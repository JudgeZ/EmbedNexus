# Visual Studio Code Transcripts

> **Status:** Transcript capture pending. This directory will track the VS Code
> extension golden transcripts once the MCP extension scripts materialize.

## Expected Files

| Filename | Purpose |
| --- | --- |
| `stdio-noise.json` | VS Code MCP extension invoking the stdio transport with Noise framing. |
| `http-tls.json` | VS Code extension replaying HTTPS SSE through the MCP transport. |
| `wsl-bridge.json` | VS Code on Windows launching the MCP server through `wsl.exe -e`. |

Follow the shared catalog guidance for any additional transports (for example,
`websocket-noise.json` once the extension supports websockets).

## Transcript Format

VS Code transcripts follow the baseline format with two optional fields:

* `extensionVersion`: Version string reported by the VS Code extension.
* `workspace`: Absolute path (normalized to POSIX style) of the active workspace
  when the transcript was captured.

Example structure:

```json
[
  {
    "direction": "send",
    "timestamp": "2024-05-01T19:44:55.332Z",
    "extensionVersion": "0.1.0",
    "workspace": "/workspaces/cursor-local-embedding-mcp",
    "envelope": {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"client": "vscode"}}
  }
]
```

## Regeneration Workflow (TODO)

1. Wait for the VS Code automation script (planned for `clients/node`) or the
   upcoming VS Code task runner integration.
2. Execute the helper with the configuration documented in
   `docs/integration/ide-overview.md`:

   ```bash
   node clients/node/index.mjs --transport stdio --ide vscode --record-transcript tmp/vscode-stdio-noise.json
   ```

3. Normalize the output and copy it into this directory.
4. Update `CHECKSUMS.sha256`, add compressed artifacts when the size exceeds the
   threshold, and commit the changes with updated documentation references.
5. Log any VS Code specific prerequisites here (workspace trust prompts, tunnel
   requirements, etc.) so future captures remain reproducible.
