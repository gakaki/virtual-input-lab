#!/bin/bash
# Install the libvirtualhid macOS bundle.
#
# Copies the C++ library and headers to /usr/local, the Rust demo to /usr/local/bin,
# and installs the Python wheel into a private virtual environment.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
prefix="${PREFIX:-/usr/local}"

# Only elevate when the install prefix is not writable by the current user.
probe="${prefix}"
while [[ ! -e "${probe}" && "${probe}" != "/" ]]; do
  probe="$(dirname "${probe}")"
done
if [[ -w "${probe}" ]]; then
  SUDO=""
else
  SUDO="sudo"
fi

echo "==> Installing C++ library and headers to ${prefix}"
${SUDO} mkdir -p "${prefix}/lib" "${prefix}/include/libvirtualhid" "${prefix}/bin"
${SUDO} cp "${here}/lib/libvirtualhid.a" "${prefix}/lib/"
${SUDO} cp -R "${here}/include/libvirtualhid/." "${prefix}/include/libvirtualhid/"
${SUDO} cp "${here}/bin/send_all_buttons" "${prefix}/bin/"
${SUDO} codesign --force --sign - "${prefix}/bin/send_all_buttons" 2>/dev/null || true

echo "==> Installing the Python module"
venv="${HOME}/Library/Application Support/libvirtualhid/venv"
python3 -m venv "${venv}"
"${venv}/bin/python" -m pip install --quiet --upgrade pip
"${venv}/bin/python" -m pip install --quiet --force-reinstall "${here}/python/libvirtualhid-"*.whl

echo
echo "Done."
echo "  C library : ${prefix}/lib/libvirtualhid.a"
echo "  Headers   : ${prefix}/include/libvirtualhid"
echo "  Rust demo : ${prefix}/bin/send_all_buttons"
echo "  Python    : ${venv}/bin/python ${here}/examples/send_all_buttons.py"
echo
echo "Try it now:"
echo "  ${venv}/bin/python \"${here}/examples/send_all_buttons.py\""
