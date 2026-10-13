#!/usr/bin/env bash
# Verify the chiaki-ng socket input protocol without building chiaki-ng.
#
#  1. compiles the reference receiver from the same protocol header the patch adds
#  2. sends from the Python client and decodes it
#  3. sends from the Rust client and decodes it
#  4. checks both clients produce byte-identical messages
#  5. checks the in-repo protocol header matches the one inside the patch
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${root}"
export CMAKE="${CMAKE:-${root}/.venv/bin/cmake}"
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-14.2}"

work="$(mktemp -d)"
receiver="${work}/chiaki_receiver"

echo '### 1. build the reference receiver (same header as the patch)'
clang -std=c11 -Wall -Wextra -Werror -I patches \
  tools/chiaki_socket_input_server_test.c -o "${receiver}"

echo '### 2. build the Rust client'
cargo build -q -p chiaki-socket-client 2>/dev/null

run_case() {
  local label="$1" send="$2"
  local socket="${work}/${label}.sock"
  local log="${work}/${label}.log"
  "${receiver}" "${socket}" 0 > "${log}" 2>&1 &
  local receiver_pid=$!
  sleep 0.4
  eval "${send}"
  # The receiver exits on its 5 s receive timeout.
  wait "${receiver_pid}" || true
  local first last
  first="$(grep -c 'buttons=0xffff' "${log}" || true)"
  last="$(tail -2 "${log}" | head -1)"
  echo "  ${label}: held messages=${first}  last=[${last}]"
  grep -q 'rejected=0' "${log}"
  grep -q 'buttons=0x0000 l2=0 r2=0' "${log}"
}

echo '### 3. Python client -> receiver'
run_case python \
  "\"${root}/.venv/bin/python\" python/examples/chiaki_send_buttons.py --socket \"${work}/python.sock\" --hold 0.2 >/dev/null"

echo '### 4. Rust client -> receiver'
run_case rust \
  "\"${root}/target/debug/chiaki-send-buttons\" --socket \"${work}/rust.sock\" >/dev/null"

echo '### 5. clients agree byte for byte'
"${root}/.venv/bin/python" python/examples/chiaki_send_buttons.py --dry-run -o "${work}/py.bin" >/dev/null
"${root}/target/debug/chiaki-send-buttons" --dry-run -o "${work}/rs.bin" >/dev/null
if cmp -s "${work}/py.bin" "${work}/rs.bin"; then
  echo "  identical ($(wc -c < "${work}/py.bin" | tr -d ' ') bytes)"
else
  echo "  MISMATCH" >&2
  exit 1
fi

echo '### 6. in-repo protocol header matches the patch'
"${root}/.venv/bin/python" - <<'PY'
import pathlib
import re
import sys

patch = pathlib.Path("patches/chiaki-ng-socket-input.patch").read_text()
match = re.search(
    r"diff --git a/gui/include/socketinput_protocol\.h.*?\n@@[^\n]*\n(.*?)(?=\ndiff --git |\Z)",
    patch,
    re.S,
)
if not match:
    sys.exit("could not find the new header inside the patch")
extracted = "\n".join(line[1:] for line in match.group(1).splitlines() if line.startswith("+"))
ours = pathlib.Path("patches/socketinput_protocol.h").read_text()
if extracted.rstrip("\n") != ours.rstrip("\n"):
    sys.exit("protocol header differs between the patch and the repo copy")
print("  match")
PY

echo
echo 'ALL CHECKS PASSED'
