# Gemini & Antigravity Instructions

Please refer to [AGENTS.md](AGENTS.md) for complete monorepo documentation, crate layering, and developer workflows.

## Key Constraints & Guardrails
- **Build & Test**: Always verify with `cargo test --workspace` and `cargo fmt --check`.
- **Layering Invariants**: Keep `l2-sniffer-protocol` strictly zero-I/O and platform-agnostic.
- **Vendored Libraries**: Preserved in `lib/` for Windows MSVC linking; never delete.
- **Error Handling**: `thiserror` for library crates, `anyhow` for CLI.
- **Protocol Resilience**: Never panic on unexpected byte sequences; fallback to `L2Packet::Raw`.
- **PR Formatting**: Always use `gh pr create/edit --body-file <path>` to prevent escape artifacts. Include `## Summary`, `## Changes`, and `## Verification`.
- **Instruction Sync**: Always keep all agent config files (`AGENTS.md`, `.github/copilot-instructions.md`, `CLAUDE.md`, `GEMINI.md`, `.cursorrules`) synchronized when updating instructions.

