# IDE Integration Overview

This guide summarizes the supported IDE clients for the Cursor Local Embedding MCP server. Each section outlines minimum requirements, supported transport layers, and sample configuration snippets for connecting to the server.

## Cursor IDE

- **Requirements**
  - Cursor Desktop v0.45 or later with MCP plugin support enabled.
  - Node.js 18+ runtime available for spawning MCP transports.
  - Local access to the `cursor-local-embedding` executable on the PATH.
- **Supported transports**: `stdio`, `sse`, and `websocket` via Cursor's agent bridge.
- **Configuration snippet** (add to `~/.cursor/mcp.json`):

```json
{
  "servers": {
    "cursor-local-embedding": {
      "command": "cursor-local-embedding",
      "args": ["--transport", "stdio"],
      "readyTimeoutMs": 10000
    }
  }
}
```

### Encrypted transport configuration

Enable Cursor's HTTPS-capable SSE bridge when you need to traverse corporate networks or enforce TLS pinning. Define the remote endpoint and provide trusted certificates explicitly:

```json
{
  "servers": {
    "cursor-local-embedding": {
      "transport": "sse",
      "url": "https://localhost:8843/mcp",
      "tls": {
        "caFile": "/etc/ssl/certs/cursor-rootCA.pem",
        "clientCert": "~/.cursor/tls/client.pem",
        "clientKey": "~/.cursor/tls/client-key.pem"
      }
    }
  }
}
```

### WSL launch bridge

When Cursor runs on Windows while the MCP server runs inside WSL, invoke the binary through `wsl.exe -e` so Windows delegates execution to the Linux environment. Pass the desired distribution name with `-d` (omit to use the default) and continue to map certificate paths to `/mnt` mounts:

```json
{
  "servers": {
    "cursor-local-embedding": {
      "command": "wsl.exe",
      "args": [
        "-d",
        "Ubuntu",
        "-e",
        "/usr/local/bin/cursor-local-embedding",
        "--transport",
        "stdio"
      ],
      "env": {
        "SSL_CERT_FILE": "/mnt/c/Users/<user>/cursor/tls/rootCA.pem"
      }
    }
  }
}
```

## Windsurf

- **Requirements**
  - Windsurf v0.8+ with custom MCP endpoints enabled.
  - Python 3.11+ (required for the official Windsurf MCP bridge).
  - Firewall rules allowing local loopback connections on configurable ports.
- **Supported transports**: `websocket` and `sse` through the Windsurf bridge; `stdio` is unsupported.
- **Configuration snippet** (example `windsurf.mcp.yaml` entry):

```yaml
servers:
  cursor-local-embedding:
    transport: websocket
    command: cursor-local-embedding
    args:
      - "--transport"
      - "websocket"
      - "--port"
      - "8820"
```

### Encrypted transport configuration

Leverage secure websockets by pointing Windsurf at a TLS-terminated listener. The `certChain` and `privateKey` paths should reference PEM files accessible to the Windsurf bridge process:

```yaml
servers:
  cursor-local-embedding:
    transport: websocket
    url: wss://127.0.0.1:8820/mcp
    tls:
      verify: true
      certChain: /etc/windsurf/tls/client-chain.pem
      privateKey: /etc/windsurf/tls/client-key.pem
      caFile: /etc/windsurf/tls/rootCA.pem
```

### WSL launch bridge

When Windsurf (Windows) tunnels into a WSL-hosted server, use `wsl.exe -e` to start the MCP server inside Linux and keep `/mnt` paths for TLS assets:

```yaml
servers:
  cursor-local-embedding:
    transport: websocket
    url: wss://127.0.0.1:8882/mcp
    command: wsl.exe
    args:
      - "-d"
      - "Ubuntu"
      - "-e"
      - "/usr/local/bin/cursor-local-embedding"
      - "--transport"
      - "websocket"
      - "--port"
      - "8882"
    env:
      SSL_CERT_FILE: /mnt/c/Users/<user>/windsurf/tls/rootCA.pem
```

## Visual Studio Code

- **Requirements**
  - VS Code 1.85+ with the MCP Client extension installed.
  - Go 1.21+ toolchain for building the extension's helper binaries.
  - Access to the `cursor-local-embedding` binary within the workspace.
- **Supported transports**: `stdio` (default) and `sse` via the extension configuration.
- **Configuration snippet** (example `.vscode/settings.json` fragment):

```json
{
  "mcp.servers": {
    "cursor-local-embedding": {
      "command": "cursor-local-embedding",
      "args": ["--transport", "stdio"],
      "env": {
        "CURSOR_EMBED_MODEL": "text-embedding-3-large"
      }
    }
  }
}
```

### Encrypted transport configuration

VS Code's MCP Client extension can connect over HTTPS SSE with explicit certificate pinning:

```json
{
  "mcp.servers": {
    "cursor-local-embedding": {
      "transport": "sse",
      "url": "https://localhost:8900/mcp",
      "tls": {
        "caCertificate": "${workspaceFolder}/certs/rootCA.pem",
        "clientCertificate": "${workspaceFolder}/certs/client.pem",
        "clientKey": "${workspaceFolder}/certs/client-key.pem"
      }
    }
  }
}
```

### WSL path convention

Inside a WSL workspace, prefer Linux-native paths while the VS Code UI runs on Windows. Use `${wslWorkspaceFolder}` to avoid hard-coding mount points:

```json
{
  "mcp.servers": {
    "cursor-local-embedding": {
      "command": "${wslWorkspaceFolder}/.venv/bin/cursor-local-embedding",
      "args": ["--transport", "stdio"],
      "env": {
        "SSL_CERT_FILE": "${wslWorkspaceFolder}/certs/rootCA.pem"
      }
    }
  }
}
```

## HTTP/TLS Configuration

All MCP transports other than raw `stdio` support TLS encryption to protect embeddings in transit. When enabling HTTPS-based SSE or secure websockets:

- Terminate TLS close to the MCP server to minimize unencrypted hops.
- Provide a certificate authority bundle (`caFile`, `caCertificate`) trusted by each IDE bridge.
- Supply client certificates only when the server enforces mutual TLS.
- Rotate keys frequently and store them outside version control.
- Use dedicated loopback ports (for example, `8843` for SSE, `8820` for websocket) to simplify firewall rules.

The configuration snippets above illustrate how each IDE advertises TLS metadata. For headless deployments, consider running a reverse proxy such as Caddy or Nginx to issue and renew certificates automatically.

## Stdio Noise Framing

The `stdio` transport can be wrapped with [Noise Protocol Framework](https://noiseprotocol.org/) handshakes to offer forward secrecy without relying on TCP-level TLS. The MCP server negotiates a `Noise_XX_25519_ChaChaPoly_BLAKE2b` pattern by default:

1. IDE bridge spawns `cursor-local-embedding` with `--transport stdio --noise`.
2. The bridge sends an ephemeral key commitment and awaits the server's response frame.
3. Both parties derive shared secrets and switch to AEAD-protected frames for JSON-RPC payloads.
4. Session keys rotate every 10,000 messages or when either side requests rekeying.

When enabling Noise framing, remember to:

- Distribute server static public keys to clients through a secure channel.
- Set `NOISE_KEY_PATH` or similar environment variables so the server can persist keys between restarts.
- Combine Noise with process-level sandboxing for comprehensive defense-in-depth.

## IDEs on Windows Subsystem for Linux (WSL)

Running IDEs on Windows while hosting the MCP server inside WSL introduces path translation and certificate distribution challenges:

- **Cursor**: Launch the MCP server with `wsl.exe -e /usr/local/bin/cursor-local-embedding` (add `-d <DistroName>` when using a non-default distribution). Export the Linux-generated CA to `/mnt/c/Users/<user>/cursor/tls/rootCA.pem` so Windows trusts it.
- **Windsurf**: Map websockets through `localhost` ports forwarded into WSL and spawn the server with `wsl.exe -e`. Store TLS keys under `/etc/windsurf/tls` within WSL and expose read-only copies through `/mnt/c` for the Windows bridge process.
- **VS Code**: Leverage Remote WSL so the MCP client runs entirely within the Linux environment. Use `${wslWorkspaceFolder}` variables to keep paths portable across machines.
- **Other IDE bridges** (JetBrains Gateway, Neovim MCP clients): Prefer launching the bridge natively inside WSL and forward UI traffic to Windows. When that is not possible, rely on `wsl.exe -e` invocations with explicit `/mnt` paths and share TLS assets through the Windows certificate manager.

Always synchronize time between Windows and WSL to prevent TLS handshake failures due to skewed certificate validity periods.

## Interaction Flow

```mermaid
sequenceDiagram
    participant IDE as IDE Client
    participant Bridge as IDE MCP Bridge
    participant TLS as TLS Terminator / Noise Layer
    participant Server as Cursor Local Embedding MCP

    rect rgb(200, 230, 255)
        note over IDE,TLS: HTTPS SSE / WebSocket
        IDE->>Bridge: Configure wss/https endpoint
        Bridge->>TLS: Initiate TCP connection
        TLS-->>Bridge: TLS handshake (cert verify)
        Bridge->>Server: Forward decrypted HTTP upgrade / SSE
        Server-->>Bridge: Ready event over encrypted channel
    end

    rect rgb(220, 255, 220)
        note over IDE,Server: stdio + Noise
        IDE->>Bridge: Spawn process with --noise
        Bridge->>Server: Noise handshake frames
        Server-->>Bridge: Noise handshake complete
        Bridge->>Server: Switch to AEAD frames
    end

    Bridge->>Server: JSON-RPC request (embedding)
    Server-->>Bridge: JSON-RPC response (vectors)
    Bridge-->>IDE: Deliver embedding results
```
