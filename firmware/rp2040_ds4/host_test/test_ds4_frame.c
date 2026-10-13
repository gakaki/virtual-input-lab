/**
 * @file test_ds4_frame.c
 * @brief Host-side unit tests for the frame parser.
 *
 * Build and run on the Mac:
 *
 *   clang -std=c11 -Wall -Wextra -I firmware/rp2040_ds4 \
 *     -I firmware/rp2040_ds4/generated \
 *     firmware/rp2040_ds4/ds4_frame.c \
 *     firmware/rp2040_ds4/host_test/test_ds4_frame.c \
 *     -o /tmp/test_ds4_frame && /tmp/test_ds4_frame
 */
#include <stdio.h>
#include <string.h>

#include "ds4_frame.h"
#include "generated/ds4_usb.h"

static int failures = 0;

static void expect(int condition, const char *what) {
  if (condition) {
    printf("  ok   %s\n", what);
  } else {
    printf("  FAIL %s\n", what);
    failures++;
  }
}

static void test_byte_by_byte(void) {
  printf("frame split one byte at a time\n");
  ds4_frame_parser parser;
  ds4_frame_init(&parser);

  uint8_t frame[DS4_FRAME_PAYLOAD_LEN + 4];
  const size_t length = ds4_frame_build(kDs4AllButtonsPayload, frame);
  expect(length == DS4_FRAME_PAYLOAD_LEN + 4, "frame length is payload + 4");

  for (size_t index = 0; index < length; index++) {
    ds4_frame_feed(&parser, &frame[index], 1);
  }
  expect(parser.accepted == 1, "one frame accepted");
  expect(parser.rejected == 0, "no frames rejected");
  expect(memcmp(parser.payload, kDs4AllButtonsPayload, DS4_FRAME_PAYLOAD_LEN) == 0,
         "payload matches the all-buttons report");
  expect(ds4_frame_take_update(&parser), "update flag set");
  expect(!ds4_frame_take_update(&parser), "update flag clears after take");
}

static void test_batched_and_noise(void) {
  printf("noise then two concatenated frames\n");
  ds4_frame_parser parser;
  ds4_frame_init(&parser);

  uint8_t first[DS4_FRAME_PAYLOAD_LEN + 4];
  uint8_t second[DS4_FRAME_PAYLOAD_LEN + 4];
  ds4_frame_build(kDs4NeutralPayload, first);
  ds4_frame_build(kDs4AllButtonsPayload, second);

  uint8_t buffer[2 * (DS4_FRAME_PAYLOAD_LEN + 4) + 5];
  const uint8_t noise[5] = {0x00, 0x11, 0xA5, 0x00, 0x7F};
  memcpy(buffer, noise, sizeof(noise));
  memcpy(buffer + sizeof(noise), first, sizeof(first));
  memcpy(buffer + sizeof(noise) + sizeof(first), second, sizeof(second));

  ds4_frame_feed(&parser, buffer, sizeof(buffer));
  expect(parser.accepted == 2, "two frames accepted");
  expect(memcmp(parser.payload, kDs4AllButtonsPayload, DS4_FRAME_PAYLOAD_LEN) == 0,
         "last accepted payload is the all-buttons report");
}

static void test_bad_checksum(void) {
  printf("corrupted checksum\n");
  ds4_frame_parser parser;
  ds4_frame_init(&parser);

  uint8_t frame[DS4_FRAME_PAYLOAD_LEN + 4];
  ds4_frame_build(kDs4AllButtonsPayload, frame);
  frame[DS4_FRAME_PAYLOAD_LEN + 3] ^= 0xFF;

  ds4_frame_feed(&parser, frame, sizeof(frame));
  expect(parser.accepted == 0, "frame not accepted");
  expect(parser.rejected == 1, "frame counted as rejected");
  expect(!ds4_frame_take_update(&parser), "no update for a bad frame");
}

static void test_bad_length(void) {
  printf("unsupported payload length\n");
  ds4_frame_parser parser;
  ds4_frame_init(&parser);

  uint8_t frame[DS4_FRAME_PAYLOAD_LEN + 4];
  ds4_frame_build(kDs4AllButtonsPayload, frame);
  frame[2] = 62;

  ds4_frame_feed(&parser, frame, sizeof(frame));
  expect(parser.accepted == 0, "frame not accepted");
  expect(parser.rejected >= 1, "frame counted as rejected");
}

static void test_recovery_after_bad_frame(void) {
  printf("good frame after a rejected one\n");
  ds4_frame_parser parser;
  ds4_frame_init(&parser);

  uint8_t bad[DS4_FRAME_PAYLOAD_LEN + 4];
  ds4_frame_build(kDs4NeutralPayload, bad);
  bad[3] ^= 0xFF;
  bad[DS4_FRAME_PAYLOAD_LEN + 3] = 0xFF;  // wrong checksum

  uint8_t good[DS4_FRAME_PAYLOAD_LEN + 4];
  ds4_frame_build(kDs4AllButtonsPayload, good);

  ds4_frame_feed(&parser, bad, sizeof(bad));
  ds4_frame_feed(&parser, good, sizeof(good));
  expect(parser.accepted == 1, "recovered and accepted the good frame");
  expect(memcmp(parser.payload, kDs4AllButtonsPayload, DS4_FRAME_PAYLOAD_LEN) == 0,
         "payload is the good frame");
}

int main(void) {
  printf("generated report: %u bytes payload, neutral[0]=0x%02x all[0]=0x%02x\n\n",
         (unsigned)DS4_FRAME_PAYLOAD_LEN,
         kDs4NeutralPayload[0],
         kDs4AllButtonsPayload[0]);

  test_byte_by_byte();
  test_batched_and_noise();
  test_bad_checksum();
  test_bad_length();
  test_recovery_after_bad_frame();

  printf("\n%s (%d failure(s))\n", failures == 0 ? "PASS" : "FAIL", failures);
  return failures == 0 ? 0 : 1;
}
