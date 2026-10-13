#!/usr/bin/env bash
# Assemble a macOS .dmg containing the C++ library, Rust binary, and Python wheel.
#
# Usage: scripts/package_dmg.sh [--no-build]
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${root}"

if [[ "${1:-}" != "--no-build" ]]; then
  bash scripts/build.sh
fi

app_name='libvirtualhid-macOS-arm64'
stage="$(mktemp -d "${TMPDIR:-/tmp}/libvirtualhid-dmg.XXXXXX")"
app_dir="${stage}/${app_name}"
mkdir -p "${app_dir}/lib" "${app_dir}/bin" "${app_dir}/python" \
  "${app_dir}/examples" "${app_dir}/include"

static_lib='build/cpp-universal/src/libvirtualhid.a'
[[ -f "${static_lib}" ]] || { echo "missing ${static_lib}; run scripts/build.sh" >&2; exit 1; }

wheel="$(ls -1 dist/libvirtualhid-*.whl | head -1)"
rust_bin='target/release/examples/send_all_buttons'
[[ -f "${rust_bin}" ]] || { echo "missing ${rust_bin}; run scripts/build.sh" >&2; exit 1; }

# C++ library + headers (universal).
cp "${static_lib}" "${app_dir}/lib/"
cp -R third_party/libvirtualhid/src/include/libvirtualhid "${app_dir}/include/"
cp third_party/libvirtualhid/LICENSES/MIT.md "${app_dir}/LICENSE-libvirtualhid-MIT.md"

# Rust binary and Python wheel.
cp "${rust_bin}" "${app_dir}/bin/"
codesign --force --sign - "${app_dir}/bin/send_all_buttons" 2>/dev/null || true
cp "${wheel}" "${app_dir}/python/"

# Examples + installer + docs.
cp python/examples/send_all_buttons.py python/examples/keymap_demo.py "${app_dir}/examples/"
cp crates/livvirtualhid/examples/send_all_buttons.rs "${app_dir}/examples/send_all_buttons.rs"
cp packaging/install.command "${app_dir}/Install libvirtualhid.command"
chmod +x "${app_dir}/Install libvirtualhid.command"
cp packaging/README.md "${app_dir}/README.md"
cp packaging/VERSION "${app_dir}/VERSION"

mkdir -p dist
image="${root}/dist/${app_name}.dmg"
hdiutil create \
  -volname 'libvirtualhid' \
  -srcfolder "${app_dir}" \
  -ov -format UDZO \
  "${image}" > /dev/null

echo "DMG: ${image}"
hdiutil imageinfo "${image}" | sed -n '1,6p'
