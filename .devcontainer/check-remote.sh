#!/usr/bin/env bash

set +e

REMOTE_HOST="192.168.1.146"
WS_PORT="3000"
DAP_PORT="50000"

check_port() {
    local host="$1"
    local port="$2"
    local label="$3"

    python3 - "$host" "$port" "$label" <<'PY'
import socket
import sys

host = sys.argv[1]
port = int(sys.argv[2])
label = sys.argv[3]

sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.settimeout(2.0)

try:
    sock.connect((host, port))
    print(f"[devcontainer] OK: {label} reachable at {host}:{port}")
except Exception as exc:
    print(
        f"[devcontainer] WARN: {label} not reachable at {host}:{port} ({exc})"
    )
finally:
    sock.close()
PY
}

echo "[devcontainer] Checking remote probe-rs connectivity..."
check_port "$REMOTE_HOST" "$WS_PORT" "probe-rs host/ws"
check_port "$REMOTE_HOST" "$DAP_PORT" "probe-rs DAP server"

exit 0
