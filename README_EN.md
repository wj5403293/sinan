# Sinan

Sinan is a **64-bit process memory analysis and pointer-chain processing tool** written in Rust. Its main areas are process memory-map inspection, pointer-chain scanning, filtering, and data comparison.

## Main Features

- **Process memory access**: Retrieves a target process's memory map, parses memory regions and their read/write/execute permissions, and provides memory read/write and multi-level pointer-chain access APIs.
- **Pointer-chain scanning**: Searches for pointers related to specified addresses. Supports configurable scan depth and offset range, module selection, exhaustive scanning, and smart deduplication; results are assembled into multi-level pointer relationships.
- **Pointer-chain filtering**: Filters binary and text chain files by validating chains against the target process's current memory mappings, module names, and an optional final target address.
- **Pointer-chain comparison**: Compares binary or text data and extracts common chains using module names, module-relative offsets, and multi-level pointer relationships.
- **Data storage**: Uses Bincode serialization with a file header containing a magic identifier, architecture width, and format version. Supports text export of binary results.
- **Lua extension**: Exposes Lua scripting interfaces for custom module parsing and memory-region filtering logic.
- **Terminal UI**: Provides a TUI for configuration, scanning, filtering, and script-related operations.

Scanning, filtering, and comparison use Rayon for parallel processing, combined with memory-mapped files, buffered I/O, and efficient data structures for large data sets.

## Platform Support

The process-memory access layer selects a platform-specific implementation at compile time:

| Platform | Main 64-bit targets | Process-memory backend |
|---|---|---|
| Linux | `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` | `process_vm_readv` / `process_vm_writev`, with a `/proc/<pid>/mem` fallback |
| Android | `aarch64-linux-android` | Linux Process API |
| Windows | `x86_64-pc-windows-msvc` or `x86_64-pc-windows-gnu` | Windows Process API (`ReadProcessMemory` / `WriteProcessMemory`) |
| macOS | `aarch64-apple-darwin`, `x86_64-apple-darwin` | Mach VM API |

**Architecture restriction: only 64-bit targets are supported. 32-bit builds are rejected at compile time.** Actual access to another process also depends on OS permissions and the target process's access restrictions.

## Build Requirements

- The Rust stable toolchain and Cargo.
- The project uses Rust 2024 Edition; Rust 1.85 or newer is recommended.
- The Rust target for the destination platform, plus the required native C compiler, linker, or SDK.
- Android cross-compilation requires the Android NDK. Windows MSVC builds require Visual Studio C++ Build Tools. macOS builds require the Xcode Command Line Tools.

Get the source:

```bash
git clone https://github.com/y145376/sinan.git
cd sinan
rustup update stable
```

## Build by Platform

The commands below build optimized Release binaries. When building natively on the destination platform, `cargo build --release` is usually sufficient; `--target` makes the target architecture explicit.

### Linux

**x86_64 (native build)**

On Debian / Ubuntu, install the basic build tools first:

```bash
sudo apt-get update
sudo apt-get install -y build-essential
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
```

**ARM64 (cross-compilation example, from an x86_64 Debian / Ubuntu host)**

```bash
sudo apt-get install -y gcc-aarch64-linux-gnu
rustup target add aarch64-unknown-linux-gnu

export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar

cargo build --release --target aarch64-unknown-linux-gnu
```

When building directly on an ARM64 Linux machine, use its native Rust toolchain; the cross-compiler configuration above is not needed.

### Android (ARM64)

Install the Android NDK and set `ANDROID_NDK_HOME` to the NDK root directory. The example below assumes a Linux x86_64 build host. Replace `linux-x86_64` with the appropriate NDK `prebuilt` directory for your host OS.

```bash
rustup target add aarch64-linux-android

export ANDROID_NDK_HOME=/path/to/android-ndk
export NDK_BIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
export ANDROID_API=26

export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$NDK_BIN/aarch64-linux-android${ANDROID_API}-clang"
export CC_aarch64_linux_android="$NDK_BIN/aarch64-linux-android${ANDROID_API}-clang"
export AR_aarch64_linux_android="$NDK_BIN/llvm-ar"

cargo build --release --target aarch64-linux-android
```

`ANDROID_API` is an example API level. Adjust it for the minimum Android version you intend to support and the API levels supported by your NDK. On macOS or Windows build hosts, use the NDK toolchain directory matching the host platform.

### Windows (x86_64)

**MSVC toolchain (recommended for native Windows builds)**

Install Visual Studio 2022 or newer with the C++ Build Tools workload and a Windows SDK, then run:

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

**GNU / MinGW-w64 toolchain**

To use the GNU target shown in the project's existing README, install MinGW-w64 and make sure its GCC toolchain is available on `PATH`:

```powershell
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

The GNU target requires a MinGW-w64 linker for the target architecture. The MSVC target requires the MSVC linker tools from Visual Studio.

### macOS

Install the Xcode Command Line Tools:

```bash
xcode-select --install
```

**Apple Silicon (ARM64)**

```bash
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

**Intel (x86_64)**

```bash
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

Building macOS binaries on a macOS host is recommended. Cross-architecture builds require Clang and the appropriate macOS SDK to be available.

## Build Artifacts

- Default host target: `target/release/`
- Explicit target: `target/<TARGET_TRIPLE>/release/`

## References

- [Rust platform support](https://doc.rust-lang.org/rustc/platform-support.html)
- [Rust cross-compilation guide](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Android NDK downloads](https://developer.android.com/ndk/downloads)
- [Rust Android platform build notes](https://doc.rust-lang.org/rustc/platform-support/android.html)
