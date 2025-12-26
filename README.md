# Charles - Terminal-Controlled Reverse Proxy

A high-performance reverse proxy written in Rust with a built-in TUI (Terminal User Interface) for monitoring and control. Designed to work seamlessly on Windows 10/11 (x86_64) using the MSVC toolchain.

## Features

- **Reverse Proxy**: Route HTTP traffic based on host and path patterns
- **Control API**: RESTful API for health checks and metrics
- **Terminal UI**: Interactive TUI for monitoring requests and metrics
- **Windows-Native**: Built and tested for Windows with MSVC toolchain
- **Configuration**: TOML-based configuration with hot-reload support
- **Load Balancing**: Weighted upstream selection with health checking
- **Request Logging**: View and replay requests from the TUI

## Prerequisites

### Windows 10/11 Requirements

1. **PowerShell and Git** - Already available in PATH on most Windows systems

2. **Visual C++ Build Tools + Windows SDK** (required for Rust crates like `ring`):
   ```powershell
   winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --quiet --wait --norestart"
   ```

3. **Rust Toolchain (MSVC)** with 2024 edition support:
   ```powershell
   winget install --id Rustlang.Rust.MSVC
   rustup update stable
   rustc --version  # Should be >= 1.82
   ```

4. **Port Configuration**: 
   - Use ports above 1024 (e.g., 8080/9000) to avoid admin elevation
   - Ensure Windows Firewall allows local loopback traffic

## Quick Start

### 1. Get the Source Code

Clone the repository into `C:\code\charles` (or your preferred location):

```powershell
git clone https://github.com/antoncomputershare/reverse_proxy.git C:\code\charles
cd C:\code\charles
```

### 2. Configuration

The example configuration is already provided in `config/charles.toml`:

```toml
listen = "127.0.0.1:8080"

[control]
listen = "127.0.0.1:9000"

[[routes]]
name = "echo"
hosts = ["example.com", "*.example.org"]
path_prefix = "/"
strip_prefix = true
rewrite_prefix = "/"

[[routes.upstreams]]
url = "http://httpbin.org"
weight = 2
fail_threshold = 3
cooldown_secs = 15

[[routes.upstreams]]
url = "http://httpbin.org:443"
```

### 3. Build and Test

```powershell
cd C:\code\charles
cargo fmt --all
cargo test
cargo build --release
```

### 4. Run Charles

#### Option A: Using the Helper Script (Recommended)

Start both the proxy and TUI with a single command:

```powershell
.\scripts\start_charles.ps1
```

This will open two PowerShell windows:
- One running the proxy server
- One running the TUI interface

#### Option B: Manual Start

**Terminal 1 - Run the Proxy:**
```powershell
$env:RUST_LOG="info"
cargo run --release -- run --config config/charles.toml
```

**Terminal 2 - Run the TUI:**
```powershell
cargo run --release -- tui --control http://127.0.0.1:9000
```

### 5. Smoke Tests

Test the control API:
```powershell
curl http://127.0.0.1:9000/health
curl http://127.0.0.1:9000/metrics
```

Test the proxy:
```powershell
curl -H "Host: example.com" http://127.0.0.1:8080/anything
```

### 6. Using the TUI

- Press `1` to switch to the Stats tab
- Press `2` to switch to the Requests tab
- Use `↑` and `↓` arrow keys to navigate requests
- Press `r` to replay a selected request
- Press `q`, `Esc`, or `Ctrl+C` to quit

## Configuration

### Proxy Settings

- `listen`: Address and port for the proxy server (e.g., "127.0.0.1:8080")

### Control API

- `control.listen`: Address and port for the control API (e.g., "127.0.0.1:9000")

### Routes

Each route defines how to match incoming requests and where to forward them:

- `name`: Friendly name for the route
- `hosts`: List of host patterns (supports wildcards like "*.example.org")
- `path_prefix`: Path prefix to match
- `strip_prefix`: Whether to strip the prefix before forwarding
- `rewrite_prefix`: Optional new prefix to add after stripping

### Upstreams

Each upstream defines a backend server:

- `url`: Backend server URL
- `weight`: Relative weight for load balancing (default: 1)
- `fail_threshold`: Number of failures before marking unhealthy (default: 3)
- `cooldown_secs`: Seconds to wait before retrying failed upstream (default: 15)

## Windows-Specific Tips

### Persistent Logging

Set the logging level to persist across sessions:
```powershell
setx RUST_LOG info
```

### Firewall Configuration

If binding to `0.0.0.0` (not recommended for control API), add a firewall rule:
```powershell
New-NetFirewallRule -DisplayName "Charles Proxy" -Direction Inbound -Program "C:\code\charles\target\release\charles.exe" -Action Allow
```

### Performance Tuning

For high-throughput scenarios:
```powershell
netsh int ipv4 show dynamicport tcp
# Adjust if needed:
# netsh int ipv4 set dynamicportrange tcp start=10000 num=50000
```

### Binding to Loopback

For security, keep the control API bound to `127.0.0.1` (loopback only). Only bind to `0.0.0.0` if you need external access and have proper firewall rules.

## Development

### Running Tests

```powershell
cargo test
```

### Formatting Code

```powershell
cargo fmt --all
```

### Building for Release

```powershell
cargo build --release
```

The compiled binary will be in `target\release\charles.exe`.

## Troubleshooting

### Build Errors

If you encounter build errors related to `ring` or other native dependencies:
1. Ensure Visual Studio Build Tools are installed
2. Verify the Windows SDK is installed
3. Try running from "Developer Command Prompt for VS 2022"

### Connection Refused

If the TUI can't connect to the control API:
1. Verify the proxy is running with `$env:RUST_LOG="info"`
2. Check the control API is accessible: `curl http://127.0.0.1:9000/health`
3. Ensure the ports match in both the config and TUI command

### Port Already in Use

If you get "address already in use" errors:
1. Check what's using the port: `netstat -ano | findstr :8080`
2. Kill the process or choose a different port in the config

## License

This project is licensed under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
