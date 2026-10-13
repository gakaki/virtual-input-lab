#!/usr/bin/env bash
# Build the libvirtualhid C++ library, the Rust crates, and the Python wheel.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${root}"

# libvirtualhid needs std::format / floating-point std::to_chars (macOS 13+).
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-14.2}"

# Prefer the project venv's cmake/maturin when present.
if [[ -x "${root}/.venv/bin/cmake" ]]; then
  export CMAKE="${root}/.venv/bin/cmake"
fi
if [[ -x "${root}/.venv/bin" ]]; then
  export PATH="${root}/.venv/bin:${PATH}"
fi

echo '>> building Rust crates (links the vendored libvirtualhid static library)'
cargo build --release -p livvirtualhid --examples

echo '>> building the universal C++ static library for packaging'
cmake -S third_party/libvirtualhid -B build/cpp-universal \
  -DCMAKE_OSX_ARCHITECTURES='arm64;x86_64' \
  -DCMAKE_OSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET}" \
  -DCMAKE_BUILD_TYPE=Release \
  -DBUILD_TESTS=OFF -DBUILD_DOCS=OFF -DBUILD_EXAMPLES=OFF \
  -DLIBVIRTUALHID_BUILD_TOOLS=OFF -DLIBVIRTUALHID_BUILD_MACOS_BROKER=OFF \
  -DLIBVIRTUALHID_INSTALL=OFF -DLIBVIRTUALHID_ENABLE_PACKAGING=OFF
cmake --build build/cpp-universal --target libvirtualhid --parallel "$(sysctl -n hw.ncpu)"

echo '>> building the Python wheel'
maturin build --release \
  --manifest-path crates/libvirtualhid-py/Cargo.toml \
  --out dist

echo
echo 'Artifacts:'
ls -1 dist/*.whl build/cpp-universal/src/libvirtualhid.a target/release/examples/send_all_buttons
