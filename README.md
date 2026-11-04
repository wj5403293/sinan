# 司南（Sinan）

司南是一款基于 Rust 开发的 **64 位进程内存分析与指针链处理工具**，主要面向进程内存映射解析、指针链扫描、过滤和数据对比等场景。

## 主要功能

- **进程内存管理**：读取目标进程的内存映射信息，解析内存区域及其读、写、执行权限，并提供进程内存数据读写和多级指针链读取接口。
- **指针链扫描**：根据目标地址扫描关联指针，支持自定义扫描深度、偏移范围、目标模块，以及穷举和智能去重扫描模式；扫描结果可构建为多层指针关系。
- **指针链过滤**：支持二进制和文本指针链文件，根据目标进程当前内存映射、模块名称及可选的最终目标地址验证链路。
- **指针链对比**：支持二进制和文本数据对比，通过模块名称、模块相对偏移及多层指针关系提取共同链路。
- **数据存储**：使用 Bincode 序列化数据，文件头包含标识、架构位数和版本信息；支持二进制结果格式化为文本。
- **Lua 脚本扩展**：提供 Lua 脚本接口，用于自定义模块解析和内存区域筛选等处理逻辑。
- **终端交互界面**：通过 TUI 提供基础配置、扫描、过滤和脚本相关功能入口。

扫描、过滤和对比流程使用 Rayon 并行处理，并结合内存映射、缓冲 I/O 和高效数据结构处理较大规模的数据。

## 平台支持

项目的进程内存访问层按操作系统选择对应实现：

| 平台 | 主要 64 位目标 | 进程内存访问实现 |
|---|---|---|
| Linux | `x86_64-unknown-linux-gnu`、`aarch64-unknown-linux-gnu` | `process_vm_readv` / `process_vm_writev`，并提供 `/proc/<pid>/mem` 回退路径 |
| Android | `aarch64-linux-android` | Linux Process API |
| Windows | `x86_64-pc-windows-msvc` 或 `x86_64-pc-windows-gnu` | Windows Process API（`ReadProcessMemory` / `WriteProcessMemory`） |
| macOS | `aarch64-apple-darwin`、`x86_64-apple-darwin` | Mach VM API |

**架构限制：项目仅支持 64 位目标，不支持 32 位编译。** 实际的进程访问能力还取决于操作系统权限及目标进程的访问限制。

## 编译环境

- Rust stable 工具链及 Cargo。
- 项目使用 Rust 2024 Edition，建议使用 Rust 1.85 或更新版本。
- 对应目标平台的 Rust target，以及该平台所需的 C 编译器、链接器或 SDK。
- Android 交叉编译需要 Android NDK；Windows MSVC 编译需要 Visual Studio C++ Build Tools；macOS 编译需要 Xcode Command Line Tools。

获取源码：

```bash
git clone https://github.com/y145376/sinan.git
cd sinan
rustup update stable
```

## 各平台编译方法

以下命令均为 Release 编译。若在目标平台本机编译，通常也可以直接执行 `cargo build --release`；指定 `--target` 可以明确构建目标架构。

### Linux

**x86_64（本机编译）**

在 Debian / Ubuntu 系统上准备基础编译工具：

```bash
sudo apt-get update
sudo apt-get install -y build-essential
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
```

**ARM64（交叉编译示例，构建主机为 Debian / Ubuntu x86_64）**

```bash
sudo apt-get install -y gcc-aarch64-linux-gnu
rustup target add aarch64-unknown-linux-gnu

export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar

cargo build --release --target aarch64-unknown-linux-gnu
```

如果直接在 ARM64 Linux 设备上编译，可使用该设备的原生 Rust 工具链，不需要上述交叉编译器配置。

### Android（ARM64）

安装 Android NDK，并将 `ANDROID_NDK_HOME` 指向 NDK 根目录。下面以 Linux x86_64 构建主机为例；`linux-x86_64` 应按实际构建主机替换为 NDK 中对应的 `prebuilt` 目录名。

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

`ANDROID_API` 是示例 API Level。请根据目标设备的最低 Android 版本和 NDK 支持范围调整；若构建主机是 macOS 或 Windows，需使用 NDK 对应主机平台的 `prebuilt` 工具链目录。

### Windows（x86_64）

**MSVC 工具链（Windows 本机编译推荐）**

安装 Visual Studio 2022 或更新版本的 C++ Build Tools，并包含 Windows SDK，然后执行：

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

**GNU / MinGW-w64 工具链**

如果使用项目现有 README 中的 GNU 目标，先安装 MinGW-w64，并确保 GCC 工具链已加入 `PATH`：

```powershell
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

GNU 目标需要与目标架构匹配的 MinGW-w64 链接器；MSVC 目标则需要 Visual Studio 的 MSVC 链接工具。

### macOS

在 macOS 上安装 Xcode Command Line Tools：

```bash
xcode-select --install
```

**Apple Silicon（ARM64）**

```bash
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

**Intel（x86_64）**

```bash
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

建议在 macOS 主机上构建 macOS 版本。跨架构构建时，需确保 Clang 和对应的 macOS SDK 可用。

## 编译产物

- 本机默认目标：`target/release/`
- 指定 target：`target/<TARGET_TRIPLE>/release/`

## 相关资料

- [Rust 平台支持列表](https://doc.rust-lang.org/rustc/platform-support.html)
- [Rust 交叉编译说明](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Android NDK 下载](https://developer.android.com/ndk/downloads)
- [Rust Android 平台编译说明](https://doc.rust-lang.org/rustc/platform-support/android.html)
