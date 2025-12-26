# Charles Proxy Starter Script
# This script starts both the proxy server and TUI in separate PowerShell windows

param(
    [string]$Config = "config/charles.toml",
    [string]$Control = "http://127.0.0.1:9000"
)

Write-Host "Starting Charles Proxy..." -ForegroundColor Green
Write-Host "Config: $Config" -ForegroundColor Cyan
Write-Host "Control API: $Control" -ForegroundColor Cyan
Write-Host ""

# Start the proxy server in a new window
Write-Host "Launching proxy server..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoLogo -NoExit -Command `$env:RUST_LOG='info'; cargo run --release -- run --config $Config"

# Give the proxy server a moment to start
Start-Sleep -Seconds 2

# Start the TUI in another new window
Write-Host "Launching TUI..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoLogo -NoExit -Command cargo run --release -- tui --control $Control"

Write-Host ""
Write-Host "Charles Proxy started successfully!" -ForegroundColor Green
Write-Host "- Proxy server is running on http://127.0.0.1:8080" -ForegroundColor Cyan
Write-Host "- Control API is running on $Control" -ForegroundColor Cyan
Write-Host "- TUI is open in a separate window" -ForegroundColor Cyan
Write-Host ""
Write-Host "To stop, close the PowerShell windows or press Ctrl+C in each." -ForegroundColor Yellow
