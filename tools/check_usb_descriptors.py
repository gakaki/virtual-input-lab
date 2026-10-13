#!/usr/bin/env python3
"""Structurally validate the RP2040 composite USB configuration descriptor.

Parses the explicit byte array in firmware/rp2040_ds4/usb_descriptors.c without
needing the Pico SDK, and checks the descriptor chain, totals, interface and
endpoint counts.

    python3 tools/check_usb_descriptors.py
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "firmware" / "rp2040_ds4" / "usb_descriptors.c"
HEADER = ROOT / "firmware" / "rp2040_ds4" / "generated" / "ds4_usb.h"

CONSTANTS = {
    "TUSB_DESC_CONFIGURATION": 0x02,
    "TUSB_DESC_INTERFACE": 0x04,
    "TUSB_DESC_ENDPOINT": 0x05,
    "TUSB_DESC_INTERFACE_ASSOCIATION": 0x0B,
    "HID_DESC_TYPE_HID": 0x21,
    "HID_DESC_TYPE_REPORT": 0x22,
    "TUSB_CLASS_HID": 0x03,
    "TUSB_CLASS_CDC": 0x02,
    "TUSB_CLASS_CDC_DATA": 0x0A,
    "CDC_SUBCLASS_ACM": 0x02,
    "CDC_PROTOCOL_AT": 0x01,
    "CDC_DESC_TYPE_CS_INTERFACE": 0x24,
    "TUSB_XFER_INTERRUPT": 0x03,
    "TUSB_XFER_BULK": 0x02,
    "ITF_NUM_HID": 0,
    "ITF_NUM_CDC": 1,
    "ITF_NUM_CDC_DATA": 2,
    "EPNUM_HID_IN": 0x81,
    "EPNUM_CDC_NOTIF": 0x82,
    "EPNUM_CDC_OUT": 0x03,
    "EPNUM_CDC_IN": 0x83,
}


def descriptor_length() -> int:
    match = re.search(r"kDs4ReportDescriptor\[(\d+)\]", HEADER.read_text())
    if not match:
        raise SystemExit("could not read the report descriptor length from ds4_usb.h")
    return int(match.group(1))


def extract_bytes() -> tuple[list[int], int]:
    text = SOURCE.read_text()
    text = re.sub(r"//[^\n]*", "", text)
    match = re.search(r"desc_configuration\[CONFIG_TOTAL_LEN\]\s*=\s*\{(.*?)\n\};", text, re.S)
    if not match:
        raise SystemExit("could not find desc_configuration in usb_descriptors.c")

    defined_total = re.search(r"#define\s+CONFIG_TOTAL_LEN\s+(\d+)", text)
    if not defined_total:
        raise SystemExit("could not find CONFIG_TOTAL_LEN")

    report_len = descriptor_length()
    values: list[int] = []
    for token in match.group(1).split(","):
        token = token.strip()
        if not token:
            continue
        if token.startswith("U16_TO_U8S_LE("):
            inner = token[len("U16_TO_U8S_LE("):-1].strip()
            if inner == "CONFIG_TOTAL_LEN":
                # Resolved after the whole array is parsed.
                values.extend([-1, -1])
                continue
            if inner.startswith("sizeof(kDs4ReportDescriptor)"):
                value = report_len
            else:
                value = int(inner, 0)
            values.extend([value & 0xFF, (value >> 8) & 0xFF])
            continue
        if token.startswith("sizeof(kDs4ReportDescriptor)"):
            values.extend([report_len & 0xFF, (report_len >> 8) & 0xFF])
            continue
        if token in CONSTANTS:
            values.append(CONSTANTS[token])
            continue
        values.append(int(token, 0))

    placeholders = values.count(-1)
    if placeholders != 2:
        raise SystemExit(f"expected one CONFIG_TOTAL_LEN placeholder pair, found {placeholders}")
    total = len(values)
    first = values.index(-1)
    values[first] = total & 0xFF
    values[first + 1] = (total >> 8) & 0xFF
    return values, int(defined_total.group(1))


def main() -> int:
    data, defined_total = extract_bytes()
    total = len(data)
    print(f"configuration descriptor: {total} bytes")

    offset = 0
    interfaces: list[int] = []
    endpoints: list[tuple[int, int, int]] = []
    declared_total = None
    report_descriptor_length = None
    problems: list[str] = []

    while offset < total:
        length = data[offset]
        kind = data[offset + 1]
        if length < 2 or offset + length > total:
            problems.append(f"offset {offset}: bad descriptor length {length}")
            break
        body = data[offset:offset + length]

        if kind == CONSTANTS["TUSB_DESC_CONFIGURATION"]:
            declared_total = body[2] | (body[3] << 8)
            print(
                f"  @0  configuration      total={declared_total} interfaces={body[4]}"
                f" attributes=0x{body[7]:02x} power={body[8] * 2}mA"
            )
        elif kind == CONSTANTS["TUSB_DESC_INTERFACE_ASSOCIATION"]:
            print(f"  @{offset:<3} interface-assoc    first={body[2]} count={body[3]} class=0x{body[4]:02x}/0x{body[5]:02x}/0x{body[6]:02x}")
        elif kind == CONSTANTS["TUSB_DESC_INTERFACE"]:
            interfaces.append(body[2])
            print(f"  @{offset:<3} interface #{body[2]}      endpoints={body[4]} class=0x{body[5]:02x}")
        elif kind == CONSTANTS["TUSB_DESC_ENDPOINT"]:
            size = body[4] | (body[5] << 8)
            endpoints.append((body[2], body[3], size))
            print(f"  @{offset:<3} endpoint 0x{body[2]:02x}     type=0x{body[3]:02x} max={size} interval={body[6]}")
        elif kind == CONSTANTS["HID_DESC_TYPE_HID"]:
            report_descriptor_length = body[7] | (body[8] << 8)
            print(f"  @{offset:<3} hid descriptor     report-descriptor={report_descriptor_length} bytes")

        offset += length

    expected_report = descriptor_length()
    if declared_total != total:
        problems.append(f"wTotalLength {declared_total} != array length {total}")
    if defined_total != total:
        problems.append(f"#define CONFIG_TOTAL_LEN {defined_total} != array length {total}")
    if report_descriptor_length != expected_report:
        problems.append(
            f"HID report descriptor length {report_descriptor_length} != generated {expected_report}"
        )
    if interfaces != [0, 1, 2]:
        problems.append(f"unexpected interface numbers {interfaces}")

    endpoint_addresses = [address for address, _, _ in endpoints]
    if len(set(endpoint_addresses)) != len(endpoint_addresses):
        problems.append(f"duplicate endpoint addresses {endpoint_addresses}")

    print()
    print(f"interfaces : {interfaces}")
    print(f"endpoints  : {[hex(a) for a in endpoint_addresses]}")
    print()
    if problems:
        for problem in problems:
            print(f"FAIL: {problem}")
        return 1
    print("OK: descriptor chain, totals, interfaces, and endpoints are consistent")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
