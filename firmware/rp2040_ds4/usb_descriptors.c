/**
 * @file usb_descriptors.c
 * @brief Composite USB descriptors: DualShock 4 HID + CDC serial.
 *
 * The configuration descriptor is written as explicit bytes so its layout can
 * be validated without building the firmware (see tools/check_usb_descriptors.py).
 *
 * NOTE: this presents the vendor/product id of a real DualShock 4 so macOS
 * recognises the device through its GameController database. That identity
 * belongs to Sony; use it for personal interoperability only.
 */
#include "tusb.h"
#include "usb_descriptors.h"

#include "generated/ds4_usb.h"

//--------------------------------------------------------------------
// Device descriptor
//--------------------------------------------------------------------
static const tusb_desc_device_t desc_device = {
  .bLength = sizeof(tusb_desc_device_t),
  .bDescriptorType = TUSB_DESC_DEVICE,
  .bcdUSB = 0x0200,
  .bDeviceClass = TUSB_CLASS_MISC,          // 0xEF
  .bDeviceSubClass = MISC_SUBCLASS_COMMON,  // 0x02
  .bDeviceProtocol = MISC_PROTOCOL_IAD,     // 0x01
  .bMaxPacketSize0 = CFG_TUD_ENDPOINT0_SIZE,
  .idVendor = DS4_VENDOR_ID,
  .idProduct = DS4_PRODUCT_ID,
  .bcdDevice = DS4_VERSION,
  .iManufacturer = 0x01,
  .iProduct = 0x02,
  .iSerialNumber = 0x03,
  .bNumConfigurations = 0x01,
};

//--------------------------------------------------------------------
// Configuration descriptor
//
//  ITF 0 : HID  -> DualShock 4 gamepad (input report id 0x01, 64 bytes)
//  ITF 1 : CDC  -> control (command channel)
//  ITF 2 : CDC  -> data
//--------------------------------------------------------------------
#define CONFIG_TOTAL_LEN 100

static const uint8_t desc_configuration[CONFIG_TOTAL_LEN] = {
  // ---- Configuration ------------------------------------------------
  0x09, TUSB_DESC_CONFIGURATION,
  U16_TO_U8S_LE(CONFIG_TOTAL_LEN),  // wTotalLength = 100
  0x03,                             // bNumInterfaces
  0x01,                             // bConfigurationValue
  0x00,                             // iConfiguration
  0x80,                             // bmAttributes: bus powered
  0xFA,                             // bMaxPower: 500 mA

  // ---- Interface 0: HID (DualShock 4) -------------------------------
  0x09, TUSB_DESC_INTERFACE,
  ITF_NUM_HID,        // bInterfaceNumber
  0x00,               // bAlternateSetting
  0x01,               // bNumEndpoints
  TUSB_CLASS_HID,     // bInterfaceClass   0x03
  0x00,               // bInterfaceSubClass
  0x00,               // bInterfaceProtocol
  0x00,               // iInterface

  // HID descriptor
  0x09, HID_DESC_TYPE_HID,
  U16_TO_U8S_LE(0x0111),                         // bcdHID 1.11
  0x00,                                          // bCountryCode
  0x01,                                          // bNumDescriptors
  HID_DESC_TYPE_REPORT,                          // bDescriptorType
  U16_TO_U8S_LE(sizeof(kDs4ReportDescriptor)),   // wDescriptorLength

  // HID interrupt IN endpoint
  0x07, TUSB_DESC_ENDPOINT,
  EPNUM_HID_IN,
  TUSB_XFER_INTERRUPT,
  U16_TO_U8S_LE(64),                             // wMaxPacketSize
  0x01,                                          // bInterval (1 ms)

  // ---- Interface Association: CDC -----------------------------------
  0x08, TUSB_DESC_INTERFACE_ASSOCIATION,
  ITF_NUM_CDC,        // bFirstInterface
  0x02,               // bInterfaceCount
  TUSB_CLASS_CDC,     // bFunctionClass   0x02
  CDC_SUBCLASS_ACM,   // bFunctionSubClass 0x02
  CDC_PROTOCOL_AT,    // bFunctionProtocol 0x01
  0x00,               // iFunction

  // ---- Interface 1: CDC control -------------------------------------
  0x09, TUSB_DESC_INTERFACE,
  ITF_NUM_CDC,        // bInterfaceNumber
  0x00,               // bAlternateSetting
  0x01,               // bNumEndpoints
  TUSB_CLASS_CDC,     // bInterfaceClass   0x02
  CDC_SUBCLASS_ACM,   // bInterfaceSubClass 0x02
  CDC_PROTOCOL_AT,    // bInterfaceProtocol 0x01
  0x00,               // iInterface

  // Header functional descriptor
  0x05, CDC_DESC_TYPE_CS_INTERFACE, 0x00, U16_TO_U8S_LE(0x0110),
  // Call management functional descriptor
  0x05, CDC_DESC_TYPE_CS_INTERFACE, 0x01, 0x00, ITF_NUM_CDC_DATA,
  // Abstract control management functional descriptor
  0x04, CDC_DESC_TYPE_CS_INTERFACE, 0x02, 0x02,
  // Union functional descriptor
  0x05, CDC_DESC_TYPE_CS_INTERFACE, 0x06, ITF_NUM_CDC, ITF_NUM_CDC_DATA,

  // CDC notification endpoint
  0x07, TUSB_DESC_ENDPOINT,
  EPNUM_CDC_NOTIF,
  TUSB_XFER_INTERRUPT,
  U16_TO_U8S_LE(16),
  0x10,                                          // bInterval (16 ms)

  // ---- Interface 2: CDC data ----------------------------------------
  0x09, TUSB_DESC_INTERFACE,
  ITF_NUM_CDC_DATA,   // bInterfaceNumber
  0x00,               // bAlternateSetting
  0x02,               // bNumEndpoints
  TUSB_CLASS_CDC_DATA,  // bInterfaceClass 0x0A
  0x00,               // bInterfaceSubClass
  0x00,               // bInterfaceProtocol
  0x00,               // iInterface

  // CDC bulk OUT endpoint
  0x07, TUSB_DESC_ENDPOINT,
  EPNUM_CDC_OUT,
  TUSB_XFER_BULK,
  U16_TO_U8S_LE(64),
  0x00,

  // CDC bulk IN endpoint
  0x07, TUSB_DESC_ENDPOINT,
  EPNUM_CDC_IN,
  TUSB_XFER_BULK,
  U16_TO_U8S_LE(64),
  0x00,
};

//--------------------------------------------------------------------
// String descriptors
//--------------------------------------------------------------------
static const char *string_desc_arr[] = {
  (const char[]){0x09, 0x04},            // 0: language id (en-US)
  "Sony Computer Entertainment",          // 1: manufacturer
  "Wireless Controller",                  // 2: product (matches a real DS4)
  "RP2040-DS4",                           // 3: serial (overridden below)
};

static uint16_t string_desc_buffer[32];

//--------------------------------------------------------------------
// TinyUSB descriptor callbacks
//--------------------------------------------------------------------
uint8_t const *tud_descriptor_device_cb(void) {
  return (uint8_t const *)&desc_device;
}

uint8_t const *tud_descriptor_configuration_cb(uint8_t index) {
  (void)index;
  return desc_configuration;
}

uint16_t const *tud_descriptor_string_cb(uint8_t index, uint16_t langid) {
  (void)langid;
  size_t count;

  if (index == 0) {
    memcpy(&string_desc_buffer[1], string_desc_arr[0], 2);
    count = 1;
  } else {
    if (index >= sizeof(string_desc_arr) / sizeof(string_desc_arr[0])) {
      return NULL;
    }
    const char *str = string_desc_arr[index];
    count = strlen(str);
    if (count > 31) {
      count = 31;
    }
    for (size_t i = 0; i < count; i++) {
      string_desc_buffer[1 + i] = str[i];
    }
  }

  string_desc_buffer[0] = (uint16_t)((TUSB_DESC_STRING << 8) | (2 * count + 2));
  return string_desc_buffer;
}

uint8_t const *tud_hid_descriptor_report_cb(uint8_t instance) {
  (void)instance;
  return kDs4ReportDescriptor;
}
