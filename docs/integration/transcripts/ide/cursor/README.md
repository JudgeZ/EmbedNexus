# Cursor IDE Transcripts

> **Status:** Transcript capture pending. Populate this directory once the Cursor
> bridge automation is wired into the client scripts.

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
  forwarded.
* `envelope`: The raw JSON-RPC payload as produced or received by the bridge.

Example structure:

```json
[
  {
    "direction": "send",
    "timestamp": "2024-05-01T18:22:13.112Z",
    "envelope": {"jsonrpc": "2.0", "id": "1", "method": "initialize", "params": {"protocolVersion": "2024-03-01"}}
  },
  {
    "direction": "receive",
    "timestamp": "2024-05-01T18:22:13.214Z",
    "envelope": {"jsonrpc": "2.0", "id": "1", "result": {"capabilities": {"tools": ["create-embedding"]}}}
  }
]
```

## Regeneration Workflow (TODO)

1. Wait for the Cursor automation scripts in `clients/python` to land.
2. From the repository root run the planned helper:

   ```bash
   python clients/python/client.py --transport stdio --prompt "<prompt>" --ide cursor --record-transcript tmp/cursor-stdio-noise.json
   ```

3. Normalize the output with the shared transcript formatter (to be published
   under `scripts/transcripts/normalize.py`).
4. Move the normalized file to this directory, update `CHECKSUMS.sha256`, and
   commit both the JSON and compressed variants as required.

Document any Cursor-specific flags (e.g., TLS certificate pinning, feature
toggles) directly in this README when they become known.
