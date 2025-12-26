# Charles - Terminal-Controlled Reverse Proxy with TUI

A lightweight, terminal-based reverse proxy with an interactive TUI (Terminal User Interface), similar to Charles Proxy. Built with Rust for high performance and Windows compatibility.

## Features

- 🚀 **High-Performance Reverse Proxy** - Built on Tokio and Hyper for async, non-blocking I/O
- 🖥️ **Interactive TUI** - Real-time monitoring and control via terminal interface
- 📊 **Request Monitoring** - View live request/response data with detailed statistics
- 🪟 **Windows Native** - Fully compatible with Windows 10/11 using MSVC toolchain
- 🔧 **Easy Configuration** - Simple command-line interface with sensible defaults

## Screenshots

The TUI provides three main views:
- **Overview**: Real-time statistics and recent activity
- **Requests**: Complete request history with filtering
- **Help**: Keyboard shortcuts and usage information

## Installation

### Prerequisites

- Rust 1.70 or later
- Windows 10/11 (x86_64)
- MSVC toolchain (Visual Studio Build Tools)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/antoncomputershare/reverse_proxy.git
cd reverse_proxy

# Build in release mode
cargo build --release

# The binary will be at target/release/charles.exe
```

## Usage

### Basic Usage

```bash
# Start proxy on default port 8080, forwarding to localhost:3000
charles

# Specify custom port and target
charles --port 8080 --target api.example.com:443

# Run without TUI (CLI mode only)
charles --no-tui --port 8080 --target localhost:3000
```

### Command Line Options

```
Options:
  -p, --port <PORT>      Port to listen on [default: 8080]
  -t, --target <TARGET>  Target host to forward requests to [default: localhost:3000]
      --no-tui           Disable TUI and run in CLI mode
  -h, --help             Print help
```

### Keyboard Shortcuts

When running with TUI:

- `q` - Quit the application
- `Tab` - Next tab
- `Shift+Tab` - Previous tab
- `1` - Overview tab
- `2` - Requests tab
- `3` - Help tab

## Architecture

The application is structured into three main modules:

- **proxy**: Handles HTTP request forwarding and connection management
- **tui**: Terminal user interface built with ratatui and crossterm
- **types**: Shared data structures and state management

## Development

### Running Tests

```bash
cargo test
```

### Running with Logging

```bash
RUST_LOG=info cargo run -- --port 8080 --target localhost:3000
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Windows-Specific Notes

This project is optimized for Windows using the MSVC toolchain:

- Uses native Windows networking APIs through Tokio
- Compatible with Windows Terminal and ConHost
- Supports ANSI escape sequences for terminal colors
- No Unix-specific dependencies

## Performance

- Handles thousands of concurrent connections
- Low memory footprint (~10MB baseline)
- Minimal latency overhead (<1ms typical)

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Inspired by Charles Proxy
- Built with [Tokio](https://tokio.rs/) for async runtime
- TUI powered by [ratatui](https://ratatui.rs/)
- HTTP handling via [Hyper](https://hyper.rs/)