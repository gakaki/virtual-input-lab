/**
 * @file frame_dump.c
 * @brief Decode a byte stream with the real firmware parser and print payloads.
 *
 * Used to check host clients against the exact parser the firmware runs:
 *
 *   clang -std=c11 -I firmware/rp2040_ds4 -o /tmp/frame_dump \
 *     firmware/rp2040_ds4/ds4_frame.c firmware/rp2040_ds4/host_test/frame_dump.c
 *   <host output> | /tmp/frame_dump
 */
#include <stdio.h>

#include "ds4_frame.h"

int main(void) {
  ds4_frame_parser parser;
  ds4_frame_init(&parser);

  int byte;
  while ((byte = getchar()) != EOF) {
    const uint8_t value = (uint8_t)byte;
    ds4_frame_feed(&parser, &value, 1);
    if (ds4_frame_take_update(&parser)) {
      printf("frame %lu:", (unsigned long)parser.accepted);
      for (size_t index = 0; index < DS4_FRAME_PAYLOAD_LEN; index++) {
        printf(" %02x", parser.payload[index]);
      }
      printf("\n");
    }
  }

  fprintf(stderr, "accepted=%lu rejected=%lu\n",
          (unsigned long)parser.accepted,
          (unsigned long)parser.rejected);
  return parser.accepted > 0 ? 0 : 1;
}
