#!/bin/bash
#
# Tests for weckpilot-wake-apply with a fake rtcwake.
# Usage: bash src-tauri/helpers/test-wake-apply.sh
#

set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
APPLY="$HERE/weckpilot-wake-apply"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

cat > "$WORK/rtcwake" <<'EOF'
#!/bin/bash
echo "$*" >> "$(dirname "$0")/calls"
EOF
chmod +x "$WORK/rtcwake"

FAILURES=0
NOW=$(date -d '2030-01-15T06:00:00' +%s)

run_case() {
    local name="$1" schedule="$2" expected="$3"
    rm -f "$WORK/calls" "$WORK/schedule.json" "$WORK/helper.log"
    [ -n "$schedule" ] && printf '%s' "$schedule" > "$WORK/schedule.json"
    WECKPILOT_SHARED_DIR="$WORK" WECKPILOT_RTCWAKE="$WORK/rtcwake" WECKPILOT_NOW="$NOW" \
        bash "$APPLY"
    local actual
    actual="$(cat "$WORK/calls" 2>/dev/null)"
    if [ "$actual" = "$expected" ]; then
        echo "ok   $name"
    else
        echo "FAIL $name: expected '$expected', got '$actual'"
        FAILURES=$((FAILURES + 1))
    fi
}

ALARM=$(date -d '2030-01-15T06:30:00' +%s)
run_case "sets wake one minute early" \
    '{"nextWake": "2030-01-15T06:30:00", "enabled": true}' \
    "-m no -t $((ALARM - 60))"

run_case "clamps a wake that is too close" \
    '{"nextWake": "2030-01-15T06:00:30", "enabled": true}' \
    "-m no -t $((NOW + 15))"

run_case "clears alarm when disabled" \
    '{"nextWake": null, "enabled": false}' \
    "-m disable"

run_case "clears alarm for past wake time" \
    '{"nextWake": "2030-01-15T05:00:00", "enabled": true}' \
    "-m disable"

run_case "clears alarm without schedule" "" "-m disable"

run_case "ignores injected values" \
    '{"nextWake": "2030-01-15T06:30:00$(touch /tmp/pwned)", "enabled": true}' \
    "-m disable"

if [ "$FAILURES" -ne 0 ]; then
    echo "$FAILURES test(s) failed"
    exit 1
fi
echo "all tests passed"
