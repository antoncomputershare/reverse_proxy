# Windows Operator Runbook

## Quick Reference

**Ports:**
- Proxy: `127.0.0.1:8080`
- Control API: `127.0.0.1:9000`

**Environment:**
- `RUST_LOG=info` (set with `setx RUST_LOG info` for persistence)

**Config File:**
- Location: `config\charles.toml`

## Starting Charles

### Option 1: Automated Start (Recommended)
```powershell
cd C:\code\charles
.\scripts\start_charles.ps1
```

This opens two PowerShell windows:
1. Proxy server
2. TUI interface

### Option 2: Manual Start

**Terminal 1 - Proxy Server:**
```powershell
cd C:\code\charles
$env:RUST_LOG="info"
cargo run --release -- run --config config\charles.toml
```

**Terminal 2 - TUI Interface:**
```powershell
cd C:\code\charles
cargo run --release -- tui --control http://127.0.0.1:9000
```

## Stopping Charles

- Close each PowerShell window, or
- Press `Ctrl+C` in each window

## Health Checks

```powershell
# Check control API health
curl http://127.0.0.1:9000/health

# Check metrics
curl http://127.0.0.1:9000/metrics

# Test proxy (requires Host header)
curl -H "Host: example.com" http://127.0.0.1:8080/anything
```

## TUI Controls

- `1` - Switch to Stats tab
- `2` - Switch to Requests tab
- `↑` / `↓` - Navigate requests
- `r` - Replay selected request
- `q` / `Esc` / `Ctrl+C` - Quit

## Troubleshooting

### Port Already in Use

```powershell
# Find process using port 8080
netstat -ano | findstr :8080

# Kill process (replace PID)
taskkill /PID <PID> /F
```

### Cannot Connect to Control API

1. Verify proxy is running with `RUST_LOG=info`
2. Check firewall settings
3. Ensure config file has correct control.listen address

### Build Errors

If you encounter build errors:

```powershell
# Clean and rebuild
cargo clean
cargo build --release
```

For Visual C++ build tool errors:
- Ensure Visual Studio Build Tools are installed
- Try running from "Developer Command Prompt for VS 2022"

## Configuration Changes

After editing `config\charles.toml`:
1. Stop the proxy server (Ctrl+C)
2. Restart it (it will reload the config)
3. TUI will automatically reconnect

## Log Files

Logs are written to stdout. To capture:

```powershell
cargo run --release -- run --config config\charles.toml > charles.log 2>&1
```

## Performance Tuning

For high-throughput scenarios:

```powershell
# Check dynamic port range
netsh int ipv4 show dynamicport tcp

# Increase if needed (requires admin)
netsh int ipv4 set dynamicportrange tcp start=10000 num=50000
```

## Security Notes

- Control API is bound to loopback (`127.0.0.1`) for security
- Do not bind to `0.0.0.0` without proper firewall rules
- Keep proxy on ports > 1024 to avoid requiring admin rights

## Maintenance

### Update Charles

```powershell
cd C:\code\charles
git pull
cargo build --release
```

### View Version

```powershell
cargo run --release -- --version
```

## Contact

For issues, check the GitHub repository or contact your administrator.
