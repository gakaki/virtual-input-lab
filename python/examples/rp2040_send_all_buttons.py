#!/usr/bin/env python3
"""Send DualShock 4 buttons to an RP2040 board over its serial channel.

The board (firmware/rp2040_ds4) appears on the Mac as a DualShock 4 plus a
serial port. This script packs a DualShock 4 USB report with the same
libvirtualhid code the virtual backend uses, wraps it in the board's frame
format, and writes it to the serial port.

    uv pip install pyserial
    python rp2040_send_all_buttons.py                 # auto-detect the port
    python rp2040_send_all_buttons.py --port /dev/tty.usbmodem1101
    python rp2040_send_all_buttons.py --dry-run -o frame.bin
"""

from __future__ import annotations

import argparse
import glob
import sys
import time
from pathlib import Path

import libvirtualhid as lvh

SYNC0 = 0xA5
SYNC1 = 0x5A
PAYLOAD_LEN = 63  # DualShock 4 USB report (64 bytes) without the report id


def build_frame(payload: bytes) -> bytes:
    """Wrap a 63-byte payload in the board's frame format."""
    if len(payload) != PAYLOAD_LEN:
        raise ValueError(f"payload must be {PAYLOAD_LEN} bytes, got {len(payload)}")
    checksum = PAYLOAD_LEN
    for byte in payload:
        checksum ^= byte
    return bytes([SYNC0, SYNC1, PAYLOAD_LEN]) + payload + bytes([checksum])


def payload_for(state: lvh.GamepadState) -> bytes:
    """Pack a DualShock 4 USB report and drop the report id byte."""
    report = lvh.pack_dualshock4_report(state, usb=True)
    if len(report) != PAYLOAD_LEN + 1:
        raise RuntimeError(f"unexpected report size {len(report)}")
    return report[1:]


def find_port() -> str | None:
    """Pick a usbmodem/usbserial device, preferring RP2040-looking names."""
    candidates = sorted(
        glob.glob("/dev/tty.usbmodem*") + glob.glob("/dev/tty.usbserial*")
    )
    return candidates[0] if candidates else None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", help="serial device (default: auto-detect)")
    parser.add_argument("--baud", type=int, default=115200)
    parser.add_argument("--hold", type=float, default=0.5, help="seconds to hold all buttons")
    parser.add_argument("--dry-run", action="store_true", help="write frames to a file instead")
    parser.add_argument("-o", "--output", default="rp2040_frames.bin", help="file for --dry-run")
    args = parser.parse_args()

    all_buttons = payload_for(lvh.GamepadState.all_buttons())
    neutral = payload_for(lvh.GamepadState.neutral())
    print(f"all-buttons payload ({len(all_buttons)}B): {all_buttons.hex(' ')}")

    if args.dry_run:
        data = build_frame(all_buttons) * 3 + build_frame(neutral)
        Path(args.output).write_bytes(data)
        print(f"wrote {len(data)} bytes to {args.output}")
        return 0

    port = args.port or find_port()
    if not port:
        print("no serial port found; is the RP2040 running the DS4 firmware?", file=sys.stderr)
        return 2

    try:
        import serial  # pyserial
    except ImportError:
        print("pyserial is required: uv pip install pyserial", file=sys.stderr)
        return 2

    print(f"opening {port} at {args.baud} baud")
    with serial.Serial(port, args.baud, timeout=1) as link:
        time.sleep(0.2)  # let the board settle after the port opens
        frame = build_frame(all_buttons)
        for index in range(3):
            link.write(frame)
            print(f"  sent all-buttons frame {index + 1}")
            time.sleep(0.1)
        time.sleep(args.hold)
        link.write(build_frame(neutral))
        print("  sent neutral frame")

    print("done")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
