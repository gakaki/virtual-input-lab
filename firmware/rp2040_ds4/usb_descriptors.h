/**
 * @file usb_descriptors.h
 * @brief Interface and endpoint numbers for the composite device.
 */
#ifndef USB_DESCRIPTORS_H_
#define USB_DESCRIPTORS_H_

enum {
  ITF_NUM_HID = 0,
  ITF_NUM_CDC = 1,
  ITF_NUM_CDC_DATA = 2,
  ITF_NUM_TOTAL
};

#define EPNUM_HID_IN 0x81
#define EPNUM_CDC_NOTIF 0x82
#define EPNUM_CDC_OUT 0x03
#define EPNUM_CDC_IN 0x83

#endif  // USB_DESCRIPTORS_H_
