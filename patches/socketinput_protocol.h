// SPDX-License-Identifier: LicenseRef-AGPL-3.0-only-OpenSSL

/**
 * @file gui/include/socketinput_protocol.h
 * @brief Wire format for the local socket controller input source.
 *
 * A local program (Python, Rust, ...) sends one fixed-size datagram per
 * controller update to a Unix datagram socket. The fields mirror the subset of
 * ChiakiControllerState that a digital/analog gamepad needs.
 *
 * Layout is explicit: no implicit padding, little-endian, 24 bytes.
 */

#ifndef CHIAKI_SOCKETINPUT_PROTOCOL_H
#define CHIAKI_SOCKETINPUT_PROTOCOL_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** "CIPK" in little-endian byte order. */
#define CHIAKI_SOCKET_INPUT_MAGIC 0x4B504943u
/** Protocol version carried by every message. */
#define CHIAKI_SOCKET_INPUT_VERSION 1u
/** Exact datagram size in bytes. */
#define CHIAKI_SOCKET_INPUT_MESSAGE_SIZE 24u

/** Default socket path; override with the CHIAKI_NG_INPUT_SOCKET env var. */
#define CHIAKI_SOCKET_INPUT_DEFAULT_PATH "/tmp/chiaki-ng-input.sock"

/**
 * @brief One controller state update.
 *
 * `buttons` uses the ChiakiControllerButton bit values; sticks are
 * `-0x7fff .. 0x7fff` with `0` centred; triggers are `0 .. 255`.
 */
typedef struct chiaki_socket_input_message_t
{
	uint32_t magic;
	uint16_t version;
	uint16_t size;
	uint32_t buttons;
	uint8_t l2_state;
	uint8_t r2_state;
	int16_t left_x;
	int16_t left_y;
	int16_t right_x;
	int16_t right_y;
	uint16_t reserved;
} ChiakiSocketInputMessage;

#ifdef __cplusplus
}
#endif

#ifdef __cplusplus
static_assert(sizeof(ChiakiSocketInputMessage) == CHIAKI_SOCKET_INPUT_MESSAGE_SIZE,
	"ChiakiSocketInputMessage must not contain implicit padding");
#else
#if defined(__STDC_VERSION__) && __STDC_VERSION__ >= 201112L
_Static_assert(sizeof(ChiakiSocketInputMessage) == CHIAKI_SOCKET_INPUT_MESSAGE_SIZE,
	"ChiakiSocketInputMessage must not contain implicit padding");
#endif
#endif

/**
 * @brief Validate a received datagram.
 *
 * @param data Received bytes.
 * @param length Number of received bytes.
 * @return 1 when the datagram is a well-formed message of this version.
 */
static inline int chiaki_socket_input_validate(const void *data, unsigned length)
{
	const ChiakiSocketInputMessage *message = (const ChiakiSocketInputMessage *)data;
	if(length != CHIAKI_SOCKET_INPUT_MESSAGE_SIZE)
		return 0;
	if(message->magic != CHIAKI_SOCKET_INPUT_MAGIC)
		return 0;
	if(message->version != CHIAKI_SOCKET_INPUT_VERSION)
		return 0;
	if(message->size != CHIAKI_SOCKET_INPUT_MESSAGE_SIZE)
		return 0;
	return 1;
}

#endif // CHIAKI_SOCKETINPUT_PROTOCOL_H
