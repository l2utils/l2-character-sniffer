# Gemini & Antigravity Agent Configuration

This file defines the project instructions and environment constraints for Gemini and Antigravity agents working in `l2-character-sniffer`.

For detailed architecture, workflow, and crate layering documentation, refer to [AGENTS.md](AGENTS.md).

---

## Key Constraints & Guardrails

- **Language & Edition**: Rust 2021 edition.
- **Architectural Hierarchy**:
  - `l2-sniffer-protocol` (Pure Rust, NO I/O, NO OS networking dependencies)
  - `l2-sniffer-capture` (Npcap/libpcap packet capture, device enumeration, offline pcap replay)
  - `l2-sniffer-core` (Domain model, `CharacterTracker` state engine, Axum & GraphQL server)
  - `l2-sniffer-cli` (CLI dashboard and commands)
- **Vendored Libraries**: Never remove or modify the `.lib` stubs in [`lib/`](lib/).
- **Verification**: Always run `cargo test --workspace` and `cargo fmt --check` before finalizing tasks.
- **Error Handling**: Use `thiserror` in libraries (`protocol`, `capture`, `core`) and `anyhow` in binary (`cli`).
- **Packet Parsers**: Never panic on unexpected byte sequences; return structured errors or fallback to `L2Packet::Raw`.
