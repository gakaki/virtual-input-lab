#!/usr/bin/env bash
# Report backend, broker, and license state for the native macOS backend.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${root}"

export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-14.2}"
if [[ -x "${root}/.venv/bin/cmake" ]]; then
  export CMAKE="${root}/.venv/bin/cmake"
fi

echo '### broker service ###'
ls -ld /Applications/VirtualHIDBroker.app 2>&1 || true
ls -l /usr/local/bin/libvirtualhid-license 2>&1 || true
ls -ld /var/run/libvirtualhid 2>&1 || true
/usr/bin/launchctl print system/dev.lizardbyte.app.libvirtualhid 2>/dev/null | sed -n '1,6p' || true

echo
echo '### license CLI ###'
/usr/local/bin/libvirtualhid-license status 2>&1 || true

echo
echo '### libvirtualhid diagnose ###'
cargo run -q -p livvirtualhid --example diagnose 2>&1 | grep -v 'both .*cargo/config'
