# Quick Start Guide

Get Charles up and running in 5 minutes!

## Installation

### Option 1: Download Pre-built Binary (Windows)

1. Go to the [Releases](https://github.com/antoncomputershare/reverse_proxy/releases) page
2. Download `charles-windows-x64.exe`
3. Rename to `charles.exe` and place in your PATH

### Option 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/antoncomputershare/reverse_proxy.git
cd reverse_proxy

# Build release version
cargo build --release

# Binary will be at: target/release/charles.exe (Windows) or target/release/charles (Unix)
```

## First Run

### 1. Start a Backend Server

For testing, start a simple HTTP server:

**Python:**
```bash
python -m http.server 3000
```

**Node.js:**
```bash
npx http-server -p 3000
```

**Or use your own application on port 3000**

### 2. Start Charles

Open a new terminal and run:

```bash
charles --port 8080 --target localhost:3000
```

You should see the TUI interface:

```
┌Charles - Reverse Proxy with TUI────────────────────────────────────┐
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
┌Tabs────────────────────────────────────────────────────────────────┐
│ Overview (1) │ Requests (2) │ Help (3)                             │
│                                                                     │
│ Statistics                                                          │
│ Total Requests: 0                                                   │
│ Successful: 0                                                       │
│ Failed: 0                                                           │
│ Active Connections: 0                                               │
└─────────────────────────────────────────────────────────────────────┘
┌───────────────────────────────────────────────────────────────────┐
│ Press 'q' to quit | Tab/Shift+Tab to navigate | 1/2/3 for tabs   │
└───────────────────────────────────────────────────────────────────┘
```

### 3. Make Requests

In another terminal:

```bash
curl http://localhost:8080/
curl http://localhost:8080/test
curl http://localhost:8080/api/users
```

Watch the requests appear in real-time in the TUI!

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `q` | Quit the application |
| `Tab` | Next tab |
| `Shift+Tab` | Previous tab |
| `1` | Overview tab |
| `2` | Requests tab |
| `3` | Help tab |

## What's Next?

- Read the [full documentation](README.md)
- Check out [examples](EXAMPLES.md)
- See [Windows-specific guide](WINDOWS.md)

## Common Issues

### TUI doesn't display properly

- Use Windows Terminal for best experience on Windows
- Try `--no-tui` flag for CLI-only mode

### Port already in use

```bash
# Use a different port
charles --port 8888 --target localhost:3000
```

### Can't connect to target

- Verify your backend server is running
- Check the target host and port are correct

## Need Help?

- Open an issue on [GitHub](https://github.com/antoncomputershare/reverse_proxy/issues)
- Check the [documentation](README.md)
