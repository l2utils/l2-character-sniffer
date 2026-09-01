# l2companion-core

Modular Rust monorepo for sniffing Lineage 2 network traffic and tracking character state.

## Workspace Architecture

```
l2companion-core/
├── Cargo.toml                      # Workspace root (shared dependencies, profiles)
├── crates/
│   ├── l2companion-protocol/        # Packet codecs, framing, opcodes, decryption (pure Rust)
│   ├── l2companion-capture/         # Npcap/Pcap capture engine, interface discovery, TCP reassembly
│   ├── l2companion-service/            # Character domain models, state machine, event broadcaster
│   └── l2companion-cli/             # Standalone CLI capture runner & device inspector
```

### Crates Overview

| Crate | Type | Description |
| :--- | :--- | :--- |
| **`l2companion-protocol`** | Library | Lineage 2 packet framing (`L2FrameCodec`), opcode registry (`ServerOpcode`), Blowfish / XOR decryption (`L2Cryptor`), and packet parsers (`UserInfo`, `StatusUpdate`, `ItemList`, etc.). |
| **`l2companion-capture`** | Library | Packet capture engine using Npcap/libpcap, device discovery, and streaming packet worker. |
| **`l2companion-service`** | Library | Decoupled domain models (`Character`, `Vitals`, `Stats`, `Location`) and state tracker (`CharacterTracker`). |
| **`l2companion-cli`** | Binary | Standalone CLI (`l2companion devices`, `l2companion sniff`) for interface inspection and terminal monitoring. |

---

## Prerequisites

### For End Users (Running Pre-compiled Binaries)
To run the pre-compiled binary, end users **only** need the Npcap driver:
1. **Npcap Driver**
   - Download and run the Npcap installer from [npcap.com](https://npcap.com/#download).
   - **Important:** During installation, check **"Install Npcap in WinPcap API-compatible Mode"**.
2. **Administrator Privileges**
   - Must run the binary as Administrator to capture network interfaces.

---

### For Developers (Building from Source)
1. **Rust Toolchain** (`rustup default stable-x86_64-pc-windows-msvc`)
2. **C++ Build Tools (MSVC Linker)**
3. **Npcap SDK** (Extract and add `C:\npcap-sdk\Lib\x64` to your `LIB` environment variable):
   - **PowerShell:** `$env:LIB += ";C:\npcap-sdk\Lib\x64"`
   - **CMD:** `set LIB=%LIB%;C:\npcap-sdk\Lib\x64`

---

## Building and Running

1. **List available network interfaces:**
   ```sh
   cargo run -p l2companion-cli -- devices
   ```

2. **Start live packet capture:**
   ```sh
   cargo run -p l2companion-cli -- sniff
   ```
   Or capture on a specific device / offline pcap:
   ```sh
   cargo run -p l2companion-cli -- sniff --device "\Device\NPF_{...}"
   cargo run -p l2companion-cli -- sniff --pcap sample_capture.pcap
   ```

---

## Testing

```sh
cargo test -p l2companion-protocol -p l2companion-service
```
