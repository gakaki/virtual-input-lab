/**
 * @file tools/iohid_probe.c
 * @brief Probe whether this machine allows an unentitled virtual HID device.
 *
 * Build and run:
 *
 *   xcrun clang -fobjc-arc -framework CoreFoundation -framework IOKit \
 *     tools/iohid_probe.c -o /tmp/iohid_probe && /tmp/iohid_probe
 *
 * Apple's IOHIDUserDevice.h documents that creating a virtual IOHIDDevice
 * requires the restricted entitlement "com.apple.developer.hid.virtual.device".
 * This probe shows what happens without it.
 */
#include <CoreFoundation/CoreFoundation.h>
#include <IOKit/hid/IOHIDKeys.h>
#include <IOKit/hidsystem/IOHIDUserDevice.h>
#include <stdio.h>

/* Minimal two-axis game pad report descriptor. */
static const uint8_t kReportDescriptor[] = {
  0x05, 0x01,        /* Usage Page (Generic Desktop)   */
  0x09, 0x05,        /* Usage (Game Pad)               */
  0xA1, 0x01,        /* Collection (Application)       */
  0x15, 0x00,        /*   Logical Minimum (0)          */
  0x26, 0xFF, 0x00,  /*   Logical Maximum (255)        */
  0x75, 0x08,        /*   Report Size (8)              */
  0x95, 0x02,        /*   Report Count (2)             */
  0x09, 0x30,        /*   Usage (X)                    */
  0x09, 0x31,        /*   Usage (Y)                    */
  0x81, 0x02,        /*   Input (Data,Var,Abs)         */
  0xC0               /* End Collection                 */
};

static void set_number(CFMutableDictionaryRef dictionary, CFStringRef key, int value) {
  CFNumberRef number = CFNumberCreate(kCFAllocatorDefault, kCFNumberIntType, &value);
  CFDictionarySetValue(dictionary, key, number);
  CFRelease(number);
}

static void set_string(CFMutableDictionaryRef dictionary, CFStringRef key, CFStringRef value) {
  CFDictionarySetValue(dictionary, key, value);
}

int main(void) {
  CFMutableDictionaryRef properties = CFDictionaryCreateMutable(
    kCFAllocatorDefault, 0, &kCFTypeDictionaryKeyCallBacks, &kCFTypeDictionaryValueCallBacks);

  CFDataRef descriptor = CFDataCreate(kCFAllocatorDefault, kReportDescriptor, (CFIndex)sizeof(kReportDescriptor));
  CFDictionarySetValue(properties, CFSTR(kIOHIDReportDescriptorKey), descriptor);
  CFRelease(descriptor);

  set_number(properties, CFSTR(kIOHIDPrimaryUsagePageKey), 0x01);
  set_number(properties, CFSTR(kIOHIDPrimaryUsageKey), 0x05);
  set_number(properties, CFSTR(kIOHIDVendorIDKey), 0x054C);
  set_number(properties, CFSTR(kIOHIDProductIDKey), 0x05C4);
  set_number(properties, CFSTR(kIOHIDVersionNumberKey), 0x0100);
  set_string(properties, CFSTR(kIOHIDProductKey), CFSTR("Probe Virtual Gamepad"));
  set_string(properties, CFSTR(kIOHIDManufacturerKey), CFSTR("livvirtualhid-mac probe"));
  set_string(properties, CFSTR(kIOHIDTransportKey), CFSTR("USB"));

  printf("calling IOHIDUserDeviceCreateWithProperties...\n");
  IOHIDUserDeviceRef device = IOHIDUserDeviceCreateWithProperties(kCFAllocatorDefault, properties, 0);
  CFRelease(properties);

  if (device == NULL) {
    printf("RESULT: FAILED (returned NULL) - virtual HID creation was refused\n");
    printf("HINT  : the restriction is com.apple.developer.hid.virtual.device\n");
    return 1;
  }

  printf("RESULT: SUCCESS - created %s\n", "IOHIDUserDeviceRef");
  CFRelease(device);
  return 0;
}
