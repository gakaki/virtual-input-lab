#!/usr/bin/env python3
"""Send every DualShock 4 button once through the `libvirtualhid` module.

Usage:

    python send_all_buttons.py           # in-memory backend (works anywhere)
    python send_all_buttons.py --platform  # native backend (needs macOS broker)
"""

from __future__ import annotations

import argparse
import sys

import libvirtualhid as lvh


def hexdump(data: bytes) -> str:
    return " ".join(f"{byte:02x}" for byte in data)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--platform",
        action="store_true",
        help="use the native backend instead of the in-memory one",
    )
    parser.add_argument(
        "--usb",
        action="store_true",
        help="create the gamepad with USB framing instead of Bluetooth",
    )
    args = parser.parse_args()

    try:
        runtime = lvh.Runtime(platform=args.platform)
    except lvh.LvhError as error:  # pragma: no cover - platform specific
        print(f"could not create runtime: {error}", file=sys.stderr)
        return 1

    print(f"backend             : {runtime.backend_name}")
    print(f"supports_gamepad    : {runtime.supports_gamepad}")
    print(f"supports_virtual_hid: {runtime.supports_virtual_hid}")

    try:
        gamepad = runtime.create_dualshock4(usb=args.usb)
    except lvh.LvhError as error:
        print(f"could not create DualShock 4: {error}", file=sys.stderr)
        print(
            "(on macOS this normally means the signed, licensed broker is missing)",
            file=sys.stderr,
        )
        return 2

    print(f"gamepad             : {gamepad}")

    # Press every logical button and fully pull both triggers.
    state = lvh.GamepadState.all_buttons()
    print(f"pressed buttons     : {len(state.pressed())} / {lvh.button_count()}")

    for usb in (False, True):
        report = lvh.pack_dualshock4_report(state, usb=usb)
        framing = "usb" if usb else "bluetooth"
        expected = lvh.dualshock4_report_size(usb=usb)
        print(f"packed {framing:<9} ({len(report):>2}B, expect {expected:>2}B): {hexdump(report)}")

    gamepad.submit(state)
    print(f"submit_count        : {gamepad.submit_count}")
    print(f"last_input_report   : {hexdump(gamepad.last_input_report())}")

    gamepad.release_all()

    # Key-mapping helper.
    mapped = lvh.GamepadState.from_keys(["cross", "circle", "square", "triangle", "l1", "r2", "dpad_up"])
    print(
        "keymap demo         : "
        f"{mapped.pressed()} left_trigger={mapped.left_trigger} right_trigger={mapped.right_trigger}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
