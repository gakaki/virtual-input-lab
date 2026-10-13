/**
 * @file ds4_frame.h
 * @brief Host-to-device frame parser for the DualShock 4 bridge.
 *
 * Kept free of Pico SDK and TinyUSB dependencies so it can be unit tested on
 * the host machine (see host_test/).
 */
#ifndef DS4_FRAME_H
#define DS4_FRAME_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/** Frame sync bytes. */
#define DS4_FRAME_SYNC0 0xA5u
#define DS4_FRAME_SYNC1 0x5Au

/** DualShock 4 USB report size, including the report id byte. */
#define DS4_USB_REPORT_SIZE 64u

/** Payload carried by a frame: the report without its leading report id. */
#define DS4_FRAME_PAYLOAD_LEN (DS4_USB_REPORT_SIZE - 1u)

/** Frame parser state. */
typedef struct {
  uint8_t payload[DS4_FRAME_PAYLOAD_LEN];
  bool updated;        /**< Set when a frame was accepted since the last take. */
  uint32_t accepted;   /**< Accepted frame count. */
  uint32_t rejected;   /**< Rejected frame count. */
  uint8_t state;       /**< Internal parser state. */
  uint8_t length;      /**< Payload length being read. */
  uint8_t index;       /**< Payload byte index. */
  uint8_t checksum;    /**< Running XOR. */
  uint8_t pending[DS4_FRAME_PAYLOAD_LEN];
} ds4_frame_parser;

/**
 * @brief Initialise (or reset) a parser.
 *
 * @param parser Parser to initialise.
 */
void ds4_frame_init(ds4_frame_parser *parser);

/**
 * @brief Feed received bytes into the parser.
 *
 * Any number of bytes may be passed; the parser accepts interrupted and
 * concatenated frames.
 *
 * @param parser Parser state.
 * @param data Received bytes.
 * @param length Number of bytes.
 */
void ds4_frame_feed(ds4_frame_parser *parser, const uint8_t *data, size_t length);

/**
 * @brief Consume the updated flag.
 *
 * @param parser Parser state.
 * @return `true` when a new payload was accepted since the previous call.
 */
bool ds4_frame_take_update(ds4_frame_parser *parser);

/**
 * @brief Build a frame around a payload.
 *
 * Used by tests and by host tooling.
 *
 * @param payload Payload bytes (DS4_FRAME_PAYLOAD_LEN of them).
 * @param out Destination buffer of at least DS4_FRAME_PAYLOAD_LEN + 4 bytes.
 * @return Frame length in bytes.
 */
size_t ds4_frame_build(const uint8_t *payload, uint8_t *out);

#endif  // DS4_FRAME_H
