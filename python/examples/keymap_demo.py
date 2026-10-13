#!/usr/bin/env python3
"""Show the default key map and a few name lookups."""

from __future__ import annotations

import json

import libvirtualhid as lvh


def main() -> None:
    print("canonical buttons:")
    print("  " + ", ".join(lvh.button_names()))
    print()

    keymap = lvh.default_keymap()
    print(f"default key map ({len(keymap)} entries):")
    print(json.dumps(dict(sorted(keymap.items())), indent=2))
    print()

    for name in ["cross", "ps", "options", "l1", "r2"]:
        bit = lvh.button_bit(name)
        # r2 is an analog trigger, not a digital button, so it has no bit.
        print(f"{name:<8} -> bit {bit:#x}")

    state = lvh.GamepadState.from_keys(["cross", "l2", "dpad_down"])
    print(f"\nfrom_keys -> {state}")


if __name__ == "__main__":
    main()
