# libvirtualhid (macOS bundle)

This bundle contains a locally built copy of
[LizardByte/libvirtualhid](https://github.com/LizardByte/libvirtualhid) plus Rust
and Python bindings that drive it.

## Contents

| Path | What it is |
| --- | --- |
| `lib/libvirtualhid.a` | Universal (arm64 + x86_64) C++ static library |
| `include/libvirtualhid/*.hpp` | Public C++ headers |
| `bin/send_all_buttons` | Rust demo that creates a DualShock 4 and sends every button |
| `python/libvirtualhid-*.whl` | Python (PyO3) wheel, `import libvirtualhid` |
| `examples/*.py`, `examples/*.rs` | Source for the Python and Rust demos |

## Install

Double-click **Install libvirtualhid.command**. It copies the library and headers
to `/usr/local` and installs the Python wheel into a private virtual environment
under `~/Library/Application Support/libvirtualhid/venv`. Because the build is
unsigned, macOS may block the script; if so, right-click it and choose *Open*.

You can also install by hand:

```sh
# Python module
python3 -m venv ~/lvh-venv
~/lvh-venv/bin/pip install python/libvirtualhid-*.whl
~/lvh-venv/bin/python examples/send_all_buttons.py
```

```sh
# C++ consumers
cp lib/libvirtualhid.a /usr/local/lib/
cp -R include/libvirtualhid /usr/local/include/
# then link with: -lvirtualhid -framework ApplicationServices -framework Carbon \
#                 -framework CoreFoundation -framework CoreGraphics -framework IOKit
```

## Backends

* **In-memory backend** (default): create a runtime with no arguments. It runs
  the full state -> DualShock 4 HID report -> submit pipeline on any machine, and
  is what the demos use by default.
* **Native backend**: pass `--platform` (Rust/Python). On macOS this needs the
  upstream signed, licensed *Virtual HID Broker* and a machine license; without
  them `create_dualshock4()` fails with `backend_unavailable` or
  `license_required`.

## Notes

The vendored library is MIT-licensed. The upstream macOS/Windows brokers are
licensed under the LizardByte Source-Available License 1.0 and are **not**
included here, because they require an Apple-approved virtual HID entitlement to
be built and signed.
