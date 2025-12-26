# Building Charles on Windows

This guide explains how to build Charles reverse proxy on Windows 10/11 using the MSVC toolchain.

## Prerequisites

### 1. Install Rust

Download and run `rustup-init.exe` from [rustup.rs](https://rustup.rs/):

```cmd
# The installer will prompt you to install Visual Studio C++ Build Tools if needed
rustup-init.exe
```

Select the default installation (MSVC toolchain).

### 2. Install Visual Studio Build Tools

If not already installed, download and install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/):

- Select "Desktop development with C++"
- Ensure "MSVC v142 - VS 2019 C++ x64/x86 build tools" is selected
- Ensure "Windows 10 SDK" is selected

### 3. Verify Installation

Open a new Command Prompt or PowerShell and verify:

```cmd
rustc --version
cargo --version
```

## Building

### Clone the Repository

```cmd
git clone https://github.com/antoncomputershare/reverse_proxy.git
cd reverse_proxy
```

### Build Debug Version

For development:

```cmd
cargo build
```

The binary will be at: `target\debug\charles.exe`

### Build Release Version

For production use:

```cmd
cargo build --release
```

The optimized binary will be at: `target\release\charles.exe`

## Running

### Run from Source

```cmd
cargo run -- --port 8080 --target localhost:3000
```

### Run the Binary

```cmd
.\target\release\charles.exe --port 8080 --target localhost:3000
```

## Windows Terminal Recommendations

For the best TUI experience on Windows, we recommend using [Windows Terminal](https://aka.ms/terminal):

1. Install from Microsoft Store: "Windows Terminal"
2. Supports full color and Unicode characters
3. Better performance than legacy Command Prompt

## Troubleshooting

### Link Error: "LINK : fatal error LNK1181"

If you get linker errors:

1. Ensure Visual Studio Build Tools are installed
2. Run the build from "x64 Native Tools Command Prompt for VS"
3. Try restarting your terminal after installing VS Build Tools

### OpenSSL Errors

If you get OpenSSL-related errors, Charles uses `hyper-tls` which should work with Windows native TLS. If issues persist:

```cmd
# Use vendored OpenSSL
set OPENSSL_NO_VENDOR=0
cargo build --release
```

### Permission Denied on Port 80

Running on port 80 requires administrator privileges:

```cmd
# Run PowerShell as Administrator
.\target\release\charles.exe --port 80 --target localhost:3000
```

## Performance Optimization

The release build is optimized for performance:

- **LTO (Link Time Optimization)**: Enabled for smaller binary
- **Optimization Level**: 3 (maximum)
- **Single Codegen Unit**: For best optimization

Typical release binary size: ~4-6 MB

## Cross-Compilation

To build for other Windows architectures:

### x86 (32-bit)

```cmd
rustup target add i686-pc-windows-msvc
cargo build --release --target i686-pc-windows-msvc
```

### ARM64

```cmd
rustup target add aarch64-pc-windows-msvc
cargo build --release --target aarch64-pc-windows-msvc
```

## Packaging

### Create Standalone Executable

The release binary is standalone and can be distributed as-is:

```cmd
copy target\release\charles.exe charles.exe
```

### Create Installer

You can use tools like:

- [Inno Setup](https://jrsoftware.org/isinfo.php)
- [WiX Toolset](https://wixtoolset.org/)
- [NSIS](https://nsis.sourceforge.io/)

Example Inno Setup script:

```inno
[Setup]
AppName=Charles Reverse Proxy
AppVersion=0.1.0
DefaultDirName={pf}\Charles
OutputDir=installer

[Files]
Source: "target\release\charles.exe"; DestDir: "{app}"

[Icons]
Name: "{group}\Charles"; Filename: "{app}\charles.exe"
```

## Continuous Integration

Example GitHub Actions workflow for Windows builds:

```yaml
name: Build Windows

on: [push, pull_request]

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal
      - name: Build
        run: cargo build --release --verbose
      - name: Test
        run: cargo test --release --verbose
      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: charles-windows
          path: target/release/charles.exe
```

## Testing

Run tests on Windows:

```cmd
cargo test
```

Run tests with output:

```cmd
cargo test -- --nocapture
```

## Development

### Code Formatting

```cmd
cargo fmt
```

### Linting

```cmd
cargo clippy
```

### Watch Mode (Auto-rebuild)

Install cargo-watch:

```cmd
cargo install cargo-watch
```

Then run:

```cmd
cargo watch -x run
```

## Additional Resources

- [Rust Windows Documentation](https://doc.rust-lang.org/stable/book/ch01-01-installation.html#installing-rustup-on-windows)
- [Windows Terminal Documentation](https://docs.microsoft.com/en-us/windows/terminal/)
- [Tokio Windows Support](https://docs.rs/tokio/latest/tokio/#windows)
