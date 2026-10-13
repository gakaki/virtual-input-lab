/**
 * @file ds4_frame.c
 * @brief Host-to-device frame parser for the DualShock 4 bridge.
 */
#include "ds4_frame.h"

#include <string.h>

enum {
  STATE_SYNC0,
  STATE_SYNC1,
  STATE_LENGTH,
  STATE_PAYLOAD,
  STATE_CHECKSUM,
};

void ds4_frame_init(ds4_frame_parser *parser) {
  memset(parser, 0, sizeof(*parser));
  parser->state = STATE_SYNC0;
}

bool ds4_frame_take_update(ds4_frame_parser *parser) {
  const bool updated = parser->updated;
  parser->updated = false;
  return updated;
}

static void reset_parser(ds4_frame_parser *parser) {
  parser->state = STATE_SYNC0;
  parser->length = 0;
  parser->index = 0;
  parser->checksum = 0;
}

static void feed_byte(ds4_frame_parser *parser, uint8_t byte) {
  switch (parser->state) {
    case STATE_SYNC0:
      if (byte == DS4_FRAME_SYNC0) {
        parser->state = STATE_SYNC1;
      }
      break;

    case STATE_SYNC1:
      if (byte == DS4_FRAME_SYNC1) {
        parser->state = STATE_LENGTH;
      } else if (byte != DS4_FRAME_SYNC0) {
        parser->state = STATE_SYNC0;
      }
      break;

    case STATE_LENGTH:
      if (byte != DS4_FRAME_PAYLOAD_LEN) {
        parser->rejected++;
        reset_parser(parser);
        break;
      }
      parser->length = byte;
      parser->index = 0;
      parser->checksum = byte;
      parser->state = STATE_PAYLOAD;
      break;

    case STATE_PAYLOAD:
      parser->pending[parser->index++] = byte;
      parser->checksum ^= byte;
      if (parser->index >= parser->length) {
        parser->state = STATE_CHECKSUM;
      }
      break;

    case STATE_CHECKSUM:
      if (byte == parser->checksum) {
        memcpy(parser->payload, parser->pending, DS4_FRAME_PAYLOAD_LEN);
        parser->updated = true;
        parser->accepted++;
      } else {
        parser->rejected++;
      }
      reset_parser(parser);
      break;

    default:
      reset_parser(parser);
      break;
  }
}

void ds4_frame_feed(ds4_frame_parser *parser, const uint8_t *data, size_t length) {
  for (size_t index = 0; index < length; index++) {
    feed_byte(parser, data[index]);
  }
}

size_t ds4_frame_build(const uint8_t *payload, uint8_t *out) {
  out[0] = DS4_FRAME_SYNC0;
  out[1] = DS4_FRAME_SYNC1;
  out[2] = DS4_FRAME_PAYLOAD_LEN;
  uint8_t checksum = DS4_FRAME_PAYLOAD_LEN;
  for (size_t index = 0; index < DS4_FRAME_PAYLOAD_LEN; index++) {
    out[3 + index] = payload[index];
    checksum ^= payload[index];
  }
  out[3 + DS4_FRAME_PAYLOAD_LEN] = checksum;
  return DS4_FRAME_PAYLOAD_LEN + 4u;
}
