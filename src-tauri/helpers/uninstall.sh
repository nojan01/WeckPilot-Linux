#!/bin/bash
#
# WeckPilot Wake Helper - Linux uninstaller
# Must run as root (WeckPilot starts it via pkexec).
#

LIB_DIR="/usr/local/lib/weckpilot"
UNIT_DIR="/etc/systemd/system"
SHARED_DIR="/var/lib/weckpilot"

echo "=== WeckPilot Wake Helper Uninstaller (Linux) ==="

if [ "$(id -u)" -ne 0 ]; then
    echo "ERROR: must be run as root"
    exit 1
fi

systemctl disable --now weckpilot-wake.path 2>/dev/null || true
systemctl disable weckpilot-wake.service 2>/dev/null || true

rm -f "$UNIT_DIR/weckpilot-wake.path" "$UNIT_DIR/weckpilot-wake.service"
systemctl daemon-reload

rtcwake -m disable >/dev/null 2>&1 || true

rm -rf "$LIB_DIR" "$SHARED_DIR"

echo "=== Uninstallation Complete ==="
