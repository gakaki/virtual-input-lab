/**
 * @file main.c
 * @brief RP2040 firmware: DualShock 4 gamepad fed by a CDC serial channel.
 *
 * The board enumerates on the Mac as a DualShock 4 (gamepad) plus a serial
 * port. A host program writes fixed frames to that serial port (see
 * ds4_frame.h); the firmware forwards the payload as a DualShock 4 input
 * report. The payload is exactly what libvirtualhid's `pack_dualshock4_report`
 * produces minus its leading report id byte, so host and firmware cannot
 * disagree about the report format.
 */
#include <stdio.h>
#include <string.h>

#include "pico/stdlib.h"
#include "tusb.h"

#include "ds4_frame.h"
#include "generated/ds4_usb.h"

/** Release every control after this long without a host frame. */
#define RELEASE_TIMEOUT_MS 1000u

/** How often the input report is repeated. Real controllers stream reports. */
#define REPORT_INTERVAL_MS 4u

static ds4_frame_parser parser;
static uint8_t current_payload[DS4_FRAME_PAYLOAD_LEN];
static absolute_time_t last_frame_time;
static bool streaming = false;

static void poll_serial(void) {
  uint8_t buffer[64];
  while (tud_cdc_available()) {
    const uint32_t count = tud_cdc_read(buffer, sizeof(buffer));
    if (count == 0) {
      break;
    }
    ds4_frame_feed(&parser, buffer, count);
  }

  if (ds4_frame_take_update(&parser)) {
    memcpy(current_payload, parser.payload, DS4_FRAME_PAYLOAD_LEN);
    last_frame_time = get_absolute_time();
    streaming = true;
  }
}

static void poll_release_timeout(void) {
  if (!streaming) {
    return;
  }
  if (absolute_time_diff_us(last_frame_time, get_absolute_time()) >
      (int64_t)RELEASE_TIMEOUT_MS * 1000) {
    memcpy(current_payload, kDs4NeutralPayload, DS4_FRAME_PAYLOAD_LEN);
    streaming = false;
  }
}

static void poll_report(void) {
  static absolute_time_t next_report;
  if (absolute_time_diff_us(next_report, get_absolute_time()) < 0) {
    return;
  }
  next_report = delayed_by_ms(get_absolute_time(), REPORT_INTERVAL_MS);

  if (tud_hid_ready()) {
    // TinyUSB prepends the report id byte, so the wire report is 64 bytes.
    tud_hid_report(DS4_REPORT_ID, current_payload, DS4_FRAME_PAYLOAD_LEN);
  }
}

//--------------------------------------------------------------------
// TinyUSB callbacks
//--------------------------------------------------------------------
void tud_mount_cb(void) {
  ds4_frame_init(&parser);
}

void tud_umount_cb(void) {
  ds4_frame_init(&parser);
  streaming = false;
}

void tud_hid_set_report_cb(
  uint8_t instance,
  uint8_t report_id,
  hid_report_type_t report_type,
  uint8_t const *buffer,
  uint16_t bufsize
) {
  (void)instance;
  (void)buffer;
  (void)bufsize;

  if (report_type != HID_REPORT_TYPE_OUTPUT) {
    return;
  }
  // Report id 0x05 carries rumble and the light bar on a real DualShock 4:
  // buffer[3] = right rumble, buffer[4] = left rumble, buffer[5..7] = RGB.
  // A board with a motor or LED can consume them here.
  (void)report_id;
}

uint16_t tud_hid_get_report_cb(
  uint8_t instance,
  uint8_t report_id,
  hid_report_type_t report_type,
  uint8_t *buffer,
  uint16_t reqlen
) {
  (void)instance;
  (void)report_id;
  (void)report_type;

  // Report calibration and pairing data as zeros; a wired gamepad does not
  // need real values for macOS to accept the device.
  memset(buffer, 0, reqlen);
  return reqlen;
}

//--------------------------------------------------------------------
int main(void) {
  stdio_init_all();
  ds4_frame_init(&parser);
  memcpy(current_payload, kDs4NeutralPayload, DS4_FRAME_PAYLOAD_LEN);
  tusb_init();

  absolute_time_t next_heartbeat = make_timeout_time_ms(2000);

  while (true) {
    tud_task();
    poll_serial();
    poll_release_timeout();
    poll_report();

    if (absolute_time_diff_us(next_heartbeat, get_absolute_time()) >= 0) {
      next_heartbeat = make_timeout_time_ms(2000);
      printf("ds4: mounted=%d accepted=%lu rejected=%lu\n",
             tud_mounted() ? 1 : 0,
             (unsigned long)parser.accepted,
             (unsigned long)parser.rejected);
    }
  }
}
