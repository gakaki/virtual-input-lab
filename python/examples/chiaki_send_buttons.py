#!/usr/bin/env python3
"""Send controller input to a patched chiaki-ng over its local socket.

Requires the socket-input patch (patches/chiaki-ng-socket-input.patch) applied
and chiaki-ng running a stream. Works while chiaki-ng is minimised: the input
never touches the window system.

    python chiaki_send_buttons.py                 # press every button briefly
    python chiaki_send_buttons.py --socket /tmp/chiaki-ng-input.sock
    python chiaki_send_buttons.py --dry-run -o msg.bin
"""

from __future__ import annotations

import argparse
import socket
import struct
import sys
import time
from pathlib import Path

MAGIC = 0x4B504943  # "CIPK"
VERSION = 1
MESSAGE_SIZE = 24
DEFAULT_SOCKET = "/tmp/chiaki-ng-input.sock"

# struct: magic, version, size, buttons, l2, r2, left_x, left_y, right_x, right_y, reserved
# Layout matches gui/include/socketinput_protocol.h exactly (little-endian, no padding).
_LAYOUT = "<IHHIBBhhhhH"

BUTTONS = {
    "cross": 1 << 0,
    "moon": 1 << 1,
    "circle": 1 << 1,
    "box": 1 << 2,
    "square": 1 << 2,
    "pyramid": 1 << 3,
    "triangle": 1 << 3,
    "dpad_left": 1 << 4,
    "dpad_right": 1 << 5,
    "dpad_up": 1 << 6,
    "dpad_down": 1 << 7,
    "l1": 1 << 8,
    "r1": 1 << 9,
    "l3": 1 << 10,
    "r3": 1 << 11,
    "options": 1 << 12,
    "share": 1 << 13,
    "touchpad": 1 << 14,
    "ps": 1 << 15,
    "up": 1 << 6,
    "down": 1 << 7,
    "left": 1 << 4,
    "right": 1 << 5,
}

ALL_BUTTONS = 0xFFFF
STICK_MAX = 0x7FFF


def button_mask(names: list[str]) -> int:
    """Turn a list of button names into a bitmask."""
    mask = 0
    for name in names:
        key = name.strip().lower()
        if key not in BUTTONS:
            raise ValueError(f"unknown button {name!r}; known: {sorted(set(BUTTONS))}")
        mask |= BUTTONS[key]
    return mask


def pack_state(
    buttons: int = 0,
    l2: int = 0,
    r2: int = 0,
    left: tuple[int, int] = (0, 0),
    right: tuple[int, int] = (0, 0),
) -> bytes:
    """Pack one 24-byte controller state message."""
    message = struct.pack(
        _LAYOUT,
        MAGIC,
        VERSION,
        MESSAGE_SIZE,
        buttons & 0xFFFFFFFF,
        max(0, min(255, l2)),
        max(0, min(255, r2)),
        max(-STICK_MAX, min(STICK_MAX, left[0])),
        max(-STICK_MAX, min(STICK_MAX, left[1])),
        max(-STICK_MAX, min(STICK_MAX, right[0])),
        max(-STICK_MAX, min(STICK_MAX, right[1])),
        0,
    )
    assert len(message) == MESSAGE_SIZE, len(message)
    return message


class ChiakiInput:
    """Datagram sender for the chiaki-ng socket input source."""

    def __init__(self, path: str = DEFAULT_SOCKET) -> None:
        self.path = path
        self.sock: socket.socket | None = None

    def __enter__(self) -> ChiakiInput:
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM)
        self.sock.connect(self.path)
        return self

    def __exit__(self, *_exc: object) -> None:
        if self.sock is not None:
            self.sock.close()
            self.sock = None

    def send(self, **state: object) -> None:
        assert self.sock is not None, "use ChiakiInput as a context manager"
        self.sock.send(pack_state(**state))  # type: ignore[arg-type]

    def hold(self, seconds: float, **state: object) -> None:
        """Repeat a state while holding it, then release."""
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            self.send(**state)
            time.sleep(0.02)
        self.send()  # neutral


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--socket", default=DEFAULT_SOCKET, help="socket path")
    parser.add_argument("--hold", type=float, default=0.5, help="seconds to hold")
    parser.add_argument("--buttons", nargs="*", default=None, help="buttons to press (default: all)")
    parser.add_argument("--dry-run", action="store_true", help="write messages to a file instead")
    parser.add_argument("-o", "--output", default="chiaki_messages.bin")
    args = parser.parse_args()

    mask = ALL_BUTTONS if not args.buttons else button_mask(args.buttons)
    all_state = dict(buttons=mask, l2=255, r2=255)
    neutral = pack_state()
    pressed = pack_state(**all_state)
    print(f"buttons mask 0x{mask:04x}, message {len(pressed)}B: {pressed.hex(' ')}")

    if args.dry_run:
        data = pressed * 3 + neutral
        Path(args.output).write_bytes(data)
        print(f"wrote {len(data)} bytes to {args.output}")
        return 0

    try:
        with ChiakiInput(args.socket) as link:
            print(f"sending to {args.socket}")
            link.hold(args.hold, **all_state)
            print("  held all buttons, released")
    except FileNotFoundError:
        print(
            f"socket {args.socket} not found - is chiaki-ng running with the "
            "socket-input patch and an active stream?",
            file=sys.stderr,
        )
        return 2
    print("done")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
