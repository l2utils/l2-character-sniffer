# GitHub Copilot Instructions

This repository is a modular Rust monorepo for sniffing Lineage 2 network traffic. Please refer to [AGENTS.md](../AGENTS.md) for detailed architecture and workflows.

## Essential Guidelines for Copilot
- **Crate Boundaries**:
  - `crates/l2-sniffer-protocol`: Pure Rust only. No `pcap`, `tokio`, or OS network calls.
  - `crates/l2-sniffer-capture`: Packet capture engine using Npcap/libpcap and offline pcap reader.
  - `crates/l2-sniffer-core`: Domain state (`CharacterTracker`), GraphQL, and Axum telemetry endpoints.
  - `crates/l2-sniffer-cli`: User-facing CLI application.
- **Commands**:
  - Build: `cargo build --workspace`
  - Test: `cargo test --workspace`
  - Lint: `cargo clippy --workspace --all-targets -- -D warnings`
  - Format: `cargo fmt --check`
- **Error Handling & Safety**:
  - Use `thiserror` for library crates and `anyhow` for `l2-sniffer-cli`.
  - When decoding binary packet buffers, use `Cursor` and `byteorder` with bounds checking. Never panic on malformed or truncated packet streams.
- **Platform Invariant**:
  - Do not delete or rename the vendored `lib/` directory (`Packet.lib`, `wpcap.lib`).
