# Windsurf Transcripts

> **Status:** Transcript capture pending. Populate this directory when the
> Windsurf automation harness is available.

## Expected Files

| Filename | Purpose |
| --- | --- |
| `websocket-noise.json` | Windsurf bridge running over secure WebSocket with Noise framing. |
| `http-tls.json` | HTTPS SSE session captured via the Windsurf MCP bridge. |
| `wsl-bridge.json` | Windows Windsurf client tunneling into a WSL-hosted MCP server. |

Extend the catalog with additional filenames for future transports (e.g.,
`websocket-tls-mutual.json`). Apply the shared naming, compression, and checksum
policies documented at the catalog root.

## Transcript Format

Windsurf transcripts mirror the Cursor structure with the following additions:

* Include a `bridgeVersion` field inside each envelope record when the Windsurf
  bridge exposes version metadata.
* When Noise handshakes are captured, append a `handshake` object containing the
  prologue hash and key fingerprints.

Example structure:

```json
[
  {
    "direction": "send",
    "timestamp": "2024-05-01T19:03:11.004Z",
    "bridgeVersion": "0.8.2",
    "envelope": {"jsonrpc": "2.0", "id": 7, "method": "initialize", "params": {"client": "windsurf"}}
  },
  {
    "direction": "receive",
    "timestamp": "2024-05-01T19:03:11.119Z",
    "envelope": {"jsonrpc": "2.0", "id": 7, "result": {"capabilities": {"tools": ["create-embedding"]}}},
    "handshake": {"noiseProtocol": "Noise_NN_25519_ChaChaPoly_BLAKE2s", "remoteKey": "..."}
  }
]
```

## Regeneration Workflow (TODO)

1. Wait for the Windsurf CLI bridge helper (planned under `clients/node`) to be
   published.
2. Run the helper with the Windsurf configuration supplied in
   `docs/integration/ide-overview.md`:

   ```bash
   node clients/node/index.mjs --transport websocket --ide windsurf --record-transcript tmp/windsurf-websocket-noise.json
   ```

3. Normalize the capture and drop it into this directory.
4. Update `CHECKSUMS.sha256` and include compressed artifacts when the size
   threshold is exceeded.
5. Document any Windsurf-specific toggles (TLS certificates, bridge flags,
   Windows path rewrites) here whenever they are required for reproduction.
