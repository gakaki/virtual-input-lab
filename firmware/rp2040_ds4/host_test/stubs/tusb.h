/**
 * @file tusb.h
 * @brief Host-side stub of TinyUSB, used for syntax checking only.
 *
 * Declares just enough of the TinyUSB surface for clang -fsyntax-only to
 * validate usb_descriptors.c and main.c without the Pico SDK present.
 */
#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

// Options normally provided by tusb_option.h / the board.
#define OPT_MCU_RP2040 1
#define OPT_OS_PICO 2
#define OPT_MODE_DEVICE 0x01
#define OPT_MODE_HOST 0x02
#define OPT_MODE_FULL_SPEED 0x10

#include "tusb_config.h"

// Descriptor types
#define TUSB_DESC_DEVICE 0x01
#define TUSB_DESC_CONFIGURATION 0x02
#define TUSB_DESC_STRING 0x03
#define TUSB_DESC_INTERFACE 0x04
#define TUSB_DESC_ENDPOINT 0x05
#define TUSB_DESC_INTERFACE_ASSOCIATION 0x0B
#define HID_DESC_TYPE_HID 0x21
#define HID_DESC_TYPE_REPORT 0x22
#define CDC_DESC_TYPE_CS_INTERFACE 0x24

// Classes
#define TUSB_CLASS_MISC 0xEF
#define MISC_SUBCLASS_COMMON 0x02
#define MISC_PROTOCOL_IAD 0x01
#define TUSB_CLASS_CDC 0x02
#define CDC_SUBCLASS_ACM 0x02
#define CDC_PROTOCOL_AT 0x01
#define TUSB_CLASS_HID 0x03
#define TUSB_CLASS_CDC_DATA 0x0A

// Transfer types
#define TUSB_XFER_CONTROL 0
#define TUSB_XFER_ISOCHRONOUS 1
#define TUSB_XFER_BULK 2
#define TUSB_XFER_INTERRUPT 3

#define U16_TO_U8S_LE(x) ((uint8_t)((uint16_t)(x) & 0xFFu)), ((uint8_t)(((uint16_t)(x) >> 8) & 0xFFu))
#define U32_TO_U8S_LE(x) \
  ((uint8_t)((uint32_t)(x) & 0xFFu)), ((uint8_t)(((uint32_t)(x) >> 8) & 0xFFu)), \
    ((uint8_t)(((uint32_t)(x) >> 16) & 0xFFu)), ((uint8_t)(((uint32_t)(x) >> 24) & 0xFFu))

typedef struct {
  uint8_t bLength;
  uint8_t bDescriptorType;
  uint16_t bcdUSB;
  uint8_t bDeviceClass;
  uint8_t bDeviceSubClass;
  uint8_t bDeviceProtocol;
  uint8_t bMaxPacketSize0;
  uint16_t idVendor;
  uint16_t idProduct;
  uint16_t bcdDevice;
  uint8_t iManufacturer;
  uint8_t iProduct;
  uint8_t iSerialNumber;
  uint8_t bNumConfigurations;
} tusb_desc_device_t;

typedef enum {
  HID_REPORT_TYPE_INVALID = 0,
  HID_REPORT_TYPE_INPUT,
  HID_REPORT_TYPE_OUTPUT,
  HID_REPORT_TYPE_FEATURE,
} hid_report_type_t;

void tusb_init(void);
void tud_task(void);
bool tud_mounted(void);

uint32_t tud_cdc_available(void);
uint32_t tud_cdc_read(void *buffer, uint32_t bufsize);

bool tud_hid_ready(void);
bool tud_hid_report(uint8_t report_id, void const *report, uint16_t len);
