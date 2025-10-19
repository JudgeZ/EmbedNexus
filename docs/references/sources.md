# External References Catalog

This catalog consolidates the authoritative external references cited across design specs, security checklists, and platform guidance. Update it whenever new regulatory, vendor, or ecosystem requirements emerge so downstream documents can remain synchronized.

## IDE MCP Expectations

- [Model Context Protocol Specification](https://github.com/modelcontextprotocol/specification)
  - **Supports**: Transport Adapter Specification (IDE integration requirements), Client Tooling (CLI & SDK) and IDE Extensions suites, Input Validation Checklist (message framing and payload validation expectations).
- [Cursor MCP Host Integration Guide](https://docs.cursor.com/advanced/model-context-protocol)
  - **Supports**: IDE Extensions coverage in the test matrix, Transport Adapter Specification (IDE bridge behaviors), Sandbox Checklist (host isolation patterns).

## Transport Encryption Guidance

- [RFC 8446 – The Transport Layer Security (TLS) Protocol Version 1.3](https://www.rfc-editor.org/rfc/rfc8446)
  - **Supports**: Transport Adapter Specification (TLS negotiation hooks), Encryption & TLS Controls test suite, Encryption Checklist (cipher negotiation, forward secrecy).
- [NIST SP 800-52 Revision 2 – Guidelines for the Selection, Configuration, and Use of TLS Implementations](https://csrc.nist.gov/publications/detail/sp/800-52/rev-2/final)
  - **Supports**: Transport Adapter Specification (policy-compliant TLS settings), Secure Storage & Retrieval suite (in-transit protection), Encryption Checklist (configuration baselines).

## Archive-Handling Best Practices

- [OWASP File Upload Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/File_Upload_Cheat_Sheet.html)
  - **Supports**: Ingestion Pipeline Specification (archive sanitization and quota enforcement), Archive Extraction Quotas test suite, Input Validation & Sandboxing Checklists (malicious archive mitigation).
- [CERT/CC Vulnerability Note VU#915345 – Zip Slip Arbitrary File Overwrite](https://www.kb.cert.org/vuls/id/915345)
  - **Supports**: Ingestion Pipeline Specification (path traversal defenses), Filesystem Watch Service tests, Sandboxing Checklist (filesystem isolation guarantees).

## Platform-Specific Guidance

### macOS

- [Apple Platform Security – Data Protection](https://support.apple.com/guide/security/welcome/web)
  - **Supports**: Encryption Engine Specification (macOS keychain integration), Secure Storage & Retrieval suite, Encryption Checklist (hardware-backed key storage expectations).
- [Apple Developer Documentation – App Sandbox Design Guide](https://developer.apple.com/library/archive/documentation/Security/Conceptual/AppSandboxDesignGuide/AppSandboxInDepth/AppSandboxInDepth.html)
  - **Supports**: Ingestion Pipeline Specification (filesystem access patterns), Sandboxing Checklist, Offline Operation commitments (macOS sandbox entitlements).

### Windows

- [Microsoft Docs – Data Protection API](https://learn.microsoft.com/windows/win32/seccng/data-protection)
  - **Supports**: Encryption Engine Specification (DPAPI-backed key recovery), Secure Storage & Retrieval suite, Encryption Checklist (platform keystore integration).
- [Microsoft Security Guidance – Controlled Folder Access](https://learn.microsoft.com/microsoft-365/security/defender-endpoint/controlled-folder-access)
  - **Supports**: Ingestion Pipeline Specification (Windows filesystem protections), Sandbox Checklist (ransomware-resistant storage), Offline Operation commitments (Windows policy alignment).

### Windows Subsystem for Linux (WSL)

- [Microsoft Learn – Interoperability between Windows and WSL](https://learn.microsoft.com/windows/wsl/filesystems)
  - **Supports**: Transport Adapter Specification (WSL path translation), Client Tooling suites (WSL coverage), Sandbox Checklist (cross-boundary filesystem isolation).
- [Microsoft Learn – Best Practices for Working with WSL Distributions](https://learn.microsoft.com/windows/wsl/best-practices)
  - **Supports**: Overview – Platform Support and Operational Guidance, Offline-first resilience testing, Secure Storage & Retrieval suite (WSL environment hardening).

---

**Maintenance note:** Review this catalog during quarterly governance updates and whenever new regulatory or vendor documentation appears so the traceability map, design specs, and security checklists remain aligned.
