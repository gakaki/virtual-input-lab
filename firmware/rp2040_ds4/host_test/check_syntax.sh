#!/usr/bin/env bash
# Syntax-check the firmware sources without the Pico SDK.
#
# The stub headers in host_test/stubs only declare enough of the Pico SDK and
# TinyUSB APIs for clang -fsyntax-only to validate our own code. The real
# firmware build still needs the Pico SDK.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
stubs="${root}/host_test/stubs"

echo ">> syntax checking firmware sources"
clang -std=c11 -Wall -Wextra -Werror -fsyntax-only \
  -I "${stubs}" \
  -I "${root}" \
  -I "${root}/generated" \
  "${root}/main.c" \
  "${root}/usb_descriptors.c" \
  "${root}/ds4_frame.c"

echo ">> building and running the frame parser tests"
tmp="$(mktemp -d)"
clang -std=c11 -Wall -Wextra -Werror \
  -I "${root}" -I "${root}/generated" \
  "${root}/ds4_frame.c" "${root}/host_test/test_ds4_frame.c" \
  -o "${tmp}/test_ds4_frame"
"${tmp}/test_ds4_frame"
