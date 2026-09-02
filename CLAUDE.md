# Claude Code Instructions

Please refer to [AGENTS.md](AGENTS.md) for full monorepo architecture, crate layering, and development workflows.

## Key Developer Commands
- **Check**: `cargo check --workspace --all-targets`
- **Test**: `cargo test --workspace` (or `cargo test -p l2-sniffer-protocol`)
- **Clippy**: `cargo clippy --workspace --all-targets -- -D warnings`
- **Format**: `cargo fmt --check` / `cargo fmt`

## Architecture & Constraints
- `crates/l2-sniffer-protocol`: Pure Rust (zero I/O, no network or OS dependencies).
- `crates/l2-sniffer-capture`: Ingestion engine (Npcap/libpcap, offline pcap replay).
- `crates/l2-sniffer-core`: Domain models, state tracker, Axum REST/WebSocket & GraphQL server.
- `crates/l2-sniffer-cli`: Standalone CLI dashboard and runner.
- **Vendored `.lib` Files**: Never delete or modify files in `lib/`.
- **Error Handling**: Use `thiserror` for library crates and `anyhow` for the CLI.
- **Packet Parsers**: Never panic on malformed data; handle errors gracefully or fall back to `L2Packet::Raw`.
- **PRs**: Always use `gh pr create/edit --body-file <path>` with structured `## Summary`, `## Changes`, and `## Verification` sections.
