# Examples

This directory contains example configurations and usage scenarios for Charles reverse proxy.

## Basic Examples

### 1. Simple Forwarding

Forward all requests from port 8080 to a backend server on port 3000:

```bash
charles --port 8080 --target localhost:3000
```

### 2. Forward to External API

Forward requests to an external API:

```bash
charles --port 8080 --target api.example.com:443
```

### 3. CLI Mode (No TUI)

Run without the terminal UI for logging/scripting purposes:

```bash
charles --no-tui --port 8080 --target localhost:3000
```

## Testing the Proxy

### Start a Test Backend Server

Create a simple Python test server:

```python
#!/usr/bin/env python3
import http.server
import socketserver
import json

PORT = 3000

class TestHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()
        response = {
            'message': 'Hello from test server!',
            'path': self.path,
            'method': 'GET'
        }
        self.wfile.write(json.dumps(response, indent=2).encode())

with socketserver.TCPServer(("", PORT), TestHandler) as httpd:
    print(f"Test server running on port {PORT}")
    httpd.serve_forever()
```

Save as `test_server.py` and run:

```bash
python3 test_server.py
```

### Test the Proxy

In another terminal, start Charles:

```bash
charles --port 8080 --target localhost:3000
```

Then make requests through the proxy:

```bash
curl http://localhost:8080/api/users
curl http://localhost:8080/test
```

You'll see all requests in the TUI interface!

## Use Cases

### Development Proxy

Use Charles as a development proxy to inspect HTTP traffic:

1. Start your backend API on port 3000
2. Start Charles: `charles --port 8080 --target localhost:3000`
3. Point your frontend to `http://localhost:8080`
4. Watch all requests in real-time in the TUI

### API Gateway

Use Charles as a simple API gateway:

```bash
charles --port 80 --target api-server:8080
```

### Load Testing Monitor

Monitor your application during load testing:

```bash
# Terminal 1: Start Charles
charles --port 8080 --target localhost:3000

# Terminal 2: Run load test
ab -n 1000 -c 10 http://localhost:8080/
```

Watch the statistics update in real-time!

## Windows-Specific Examples

### PowerShell

```powershell
# Start the proxy
.\charles.exe --port 8080 --target localhost:3000

# Test with PowerShell
Invoke-WebRequest -Uri http://localhost:8080/api/users
```

### Windows Services

You can run Charles as a Windows service using NSSM or similar tools:

```cmd
nssm install CharlesProxy "C:\path\to\charles.exe" "--no-tui --port 8080 --target localhost:3000"
nssm start CharlesProxy
```

## Advanced Configuration

### Multiple Instances

Run multiple proxy instances for different services:

```bash
# Terminal 1: API proxy
charles --port 8080 --target api.example.com:443

# Terminal 2: Auth proxy
charles --port 8081 --target auth.example.com:443

# Terminal 3: Storage proxy
charles --port 8082 --target storage.example.com:443
```

### Logging to File

Redirect logs when running in CLI mode:

```bash
charles --no-tui --port 8080 --target localhost:3000 > charles.log 2>&1
```

## Troubleshooting

### Port Already in Use

If you get a "port already in use" error:

```bash
# Windows
netstat -ano | findstr :8080

# Linux
lsof -i :8080
```

### Connection Refused

If the target server is not responding:

1. Check if the target server is running
2. Verify the target host and port are correct
3. Check firewall settings

### TUI Not Displaying

If the TUI doesn't display correctly:

- Ensure your terminal supports ANSI colors
- Try Windows Terminal (recommended for Windows)
- Use `--no-tui` flag as a fallback
