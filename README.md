# Kenshi Online 🦀

Rewrite complete of the Kenshi Multiplayer mod in Rust. High performance, memory safety, and modern networking.

## 📂 Project Structure

The project is organized as a Cargo Workspace with the following crates:

| Crate | Type | Description |
|-------|------|-------------|
| **`kenshi_server`** | Binary | The dedicated game server. Manages player sessions, synchronization, and world state. |
| **`kenshi_launcher`** | Binary | CLI tool to launch Kenshi and inject the mod DLL. Acts as the client entry point. |
| **`kenshi_mod`** | Library (DLL) | The client-side logic injected into the game. Handles hooking, memory reading, and the overlay. |
| **`kenshi_protocol`** | Library | Shared networking packets and data types used by both Client and Server. |

## 🚀 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (Latest Stable)
- Kenshi (Steam/GOG Version)

### Building

To build the entire project (Server, Launcher, and Mod DLL):

```powershell
cd rust
cargo build --release
```

The artifacts will be generated in `rust/target/release/`:
- `kenshi_server.exe`
- `kenshi_launcher.exe`
- `kenshi_mod.dll`

### Usage

#### 1. Start the Server
Run the dedicated server on the host machine:
```powershell
./kenshi_server.exe
```
*Listens on 0.0.0.0:5555 by default.*

#### 2. Launch the Client
Use the launcher to start Kenshi and inject the mod:
```powershell
./kenshi_launcher.exe launch --exe "path/to/kenshi.exe" --dll "path/to/kenshi_mod.dll"
```
*Note: Ensure `kenshi_mod.dll` is correctly pointed to.*

## 🛠 Features
- **Async Networking**: Validated `tokio` based TCP communication.
- **Structured Packets**: `serde` serialization for robust data exchange.
- **Memory Safety**: Rust ownership model prevents common crash bugs found in the C++ version.
- **Hooking Architecture**: Modular hooking system ready for MinHook integration.

## ⚠️ Status
- **Memory Scanning**: Offsets are defined but dynamic scanning needs implementation.
- **Rendering**: ImGui overlay placeholder is in place.
- **Gameplay**: Basic position sync is implemented in the protocol.
