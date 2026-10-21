#!/bin/bash
#
# WeckPilot Wake Helper - Linux installer
# Installs a systemd path/service pair that programs the RTC wake alarm.
# Must run as root (WeckPilot starts it via pkexec).
#
# Usage: install.sh <source-directory>
#

set -e

SOURCE_DIR="${1:-.}"
LIB_DIR="/usr/local/lib/weckpilot"
UNIT_DIR="/etc/systemd/system"
SHARED_DIR="/var/lib/weckpilot"
SCHEDULE_FILE="$SHARED_DIR/schedule.json"
OWNER_UID="${PKEXEC_UID:-${SUDO_UID:-}}"

echo "=== WeckPilot Wake Helper Installer (Linux) ==="

if [ "$(id -u)" -ne 0 ]; then
    echo "ERROR: must be run as root"
    exit 1
fi

for TOOL in systemctl rtcwake; do
    if ! command -v "$TOOL" >/dev/null 2>&1; then
        echo "ERROR: required tool not found: $TOOL"
        exit 1
    fi
done

if [ ! -e /sys/class/rtc/rtc0/wakealarm ]; then
    echo "WARNING: /sys/class/rtc/rtc0/wakealarm not found, waking from suspend may not work"
fi

for FILE_NAME in weckpilot-wake-apply weckpilot-wake.service weckpilot-wake.path; do
    if [ ! -f "$SOURCE_DIR/$FILE_NAME" ]; then
        echo "ERROR: $FILE_NAME not found in $SOURCE_DIR"
        exit 1
    fi
done

# The directory stays root-owned so the user can only rewrite schedule.json,
# not replace it or the root-written log with symlinks.
echo "Creating $SHARED_DIR..."
install -d -o root -g root -m 755 "$SHARED_DIR"
if [ ! -f "$SCHEDULE_FILE" ] || [ -L "$SCHEDULE_FILE" ]; then
    rm -f "$SCHEDULE_FILE"
    printf '{\n  "nextWake": null,\n  "enabled": false,\n  "alarmTime": null,\n  "label": null\n}\n' > "$SCHEDULE_FILE"
fi
if [ -n "$OWNER_UID" ]; then
    chown "$OWNER_UID" "$SCHEDULE_FILE"
fi
chmod 644 "$SCHEDULE_FILE"

echo "Installing helper to $LIB_DIR..."
install -d -o root -g root -m 755 "$LIB_DIR"
install -o root -g root -m 755 "$SOURCE_DIR/weckpilot-wake-apply" "$LIB_DIR/weckpilot-wake-apply"

echo "Installing systemd units..."
install -o root -g root -m 644 "$SOURCE_DIR/weckpilot-wake.service" "$UNIT_DIR/weckpilot-wake.service"
install -o root -g root -m 644 "$SOURCE_DIR/weckpilot-wake.path" "$UNIT_DIR/weckpilot-wake.path"

systemctl daemon-reload
systemctl enable weckpilot-wake.service
systemctl enable --now weckpilot-wake.path
systemctl start weckpilot-wake.service || true

echo ""
echo "=== Installation Verification ==="
if systemctl is-active --quiet weckpilot-wake.path; then
    echo "✓ Schedule watcher active"
else
    echo "✗ Schedule watcher NOT active"
    exit 1
fi

echo "=== Installation Complete ==="
