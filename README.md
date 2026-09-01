# l2-character-sniffer

Sniff your own Lineage 2 character data.

## Prerequisites

### For End Users (Running Pre-compiled Binaries)
To run the pre-compiled binary, end users **only** need the Npcap driver:
1. **Npcap Driver**
   - Download and run the Npcap installer from [npcap.com](https://npcap.com/#download).
   - **Important:** During installation, check **"Install Npcap in WinPcap API-compatible Mode"**.
   - *Note:* Windows does not include a native packet capture driver or `wpcap.dll`. Without Npcap installed, packet capture applications cannot bind to network interfaces.
2. **Administrator Privileges**
   - Must run the binary as Administrator to capture network interfaces.

*(End users do **NOT** need Rust, Visual Studio, or the Npcap SDK).*

---

### For Developers (Building from Source)
In addition to installing the Npcap driver above, building the project from source requires:

1. **Rust Toolchain**
   - Install Rust via [rustup.rs](https://rustup.rs/) (`x86_64-pc-windows-msvc`).

2. **C++ Build Tools (MSVC Linker)**
   - Install [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the **"Desktop development with C++"** workload for `link.exe` and the Windows SDK.

3. **Npcap SDK (Compile-Time Link Libraries)**
   - Download the **Npcap SDK** (`.zip`) from [npcap.com](https://npcap.com/#download).
   - Extract to a directory (e.g., `C:\npcap-sdk`).
   - Add `C:\npcap-sdk\Lib\x64` to your `LIB` environment variable so the Rust linker can locate `wpcap.lib` and `Packet.lib`:
     - **PowerShell:** `$env:LIB += ";C:\npcap-sdk\Lib\x64"`
     - **CMD:** `set LIB=%LIB%;C:\npcap-sdk\Lib\x64`
     - **System Environment Variable:** Add `C:\npcap-sdk\Lib\x64` under System Variables -> `LIB`.

---

## Building and Running

1. **Clone the repository:**
   ```sh
   git clone https://github.com/jason-yang/l2-character-sniffer.git
   cd l2-character-sniffer
   ```

2. **Build the binary:**
   ```sh
   cargo build --release
   ```

3. **Run the application:**
   > ⚠️ **Note:** Administrative privileges are required to open network interfaces for packet sniffing. Run your terminal or binary as Administrator.
   ```sh
   cargo run --release
   ```
