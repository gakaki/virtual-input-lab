/**
 * @file lvh_shim.h
 * @brief Flat C ABI over the libvirtualhid C++ public API.
 *
 * libvirtualhid exposes C++ classes (std::shared_ptr / std::unique_ptr /
 * std::function). Rust and PyO3 cannot consume those directly, so this shim
 * re-exports the gamepad, report-packing, and license entry points behind a
 * stable C surface.
 */
#ifndef LVH_SHIM_H
#define LVH_SHIM_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Opaque runtime handle. */
typedef struct lvh_rt lvh_rt;
/** Opaque gamepad handle. */
typedef struct lvh_gp lvh_gp;

/** Error categories mirroring lvh::ErrorCode. */
typedef enum {
  LVH_OK = 0,
  LVH_INVALID_ARGUMENT = 1,
  LVH_BACKEND_UNAVAILABLE = 2,
  LVH_DEVICE_CLOSED = 3,
  LVH_UNSUPPORTED_PROFILE = 4,
  LVH_LICENSE_REQUIRED = 5,
  LVH_LICENSE_INVALID = 6,
  LVH_ACTIVATION_LIMIT_REACHED = 7,
  LVH_NETWORK_UNAVAILABLE = 8,
  LVH_BACKEND_FAILURE = 9,
} lvh_error_code;

/** Backend selection mirroring lvh::BackendKind. */
typedef enum {
  LVH_BACKEND_FAKE = 0,
  LVH_BACKEND_PLATFORM = 1,
} lvh_backend_kind;

/** Transport mirroring lvh::BusType. */
typedef enum {
  LVH_BUS_UNKNOWN = 0,
  LVH_BUS_USB = 1,
  LVH_BUS_BLUETOOTH = 2,
} lvh_bus_type;

/** License state mirroring lvh::LicenseState. */
typedef enum {
  LVH_LS_UNAVAILABLE = 0,
  LVH_LS_UNLICENSED = 1,
  LVH_LS_LICENSED = 2,
  LVH_LS_EXPIRED = 3,
  LVH_LS_DISABLED = 4,
  LVH_LS_INVALID = 5,
} lvh_license_state;

/**
 * @brief Flattened gamepad state.
 *
 * `buttons` is a bitmask over lvh::GamepadButton: bit i corresponds to the
 * button whose enum value is i (see lvh_button_bit / lvh_button_all_mask).
 */
typedef struct {
  uint32_t buttons;
  float left_x;
  float left_y;
  float right_x;
  float right_y;
  float left_trigger;
  float right_trigger;
} lvh_gamepad_state;

/* ------------------------------------------------------------------ runtime */

/** Create a runtime. Returns NULL on failure; see lvh_last_message(). */
lvh_rt *lvh_rt_create(int32_t backend_kind);
/** Destroy a runtime and every device it owns. */
void lvh_rt_destroy(lvh_rt *runtime);
/** Effective backend kind (lvh_backend_kind). */
int32_t lvh_rt_backend_kind(const lvh_rt *runtime);
/** Backend name such as "fake" or "macos-virtual-hid-coregraphics". */
const char *lvh_rt_backend_name(const lvh_rt *runtime);
int32_t lvh_rt_supports_gamepad(const lvh_rt *runtime);
int32_t lvh_rt_supports_virtual_hid(const lvh_rt *runtime);
/** Number of devices currently owned by the runtime. */
int32_t lvh_rt_active_device_count(const lvh_rt *runtime);

/* ------------------------------------------------------------------ gamepad */

/**
 * @brief Create a DualShock 4 gamepad.
 *
 * @param runtime Runtime handle.
 * @param usb_framing Non-zero for USB framing, zero for Bluetooth framing.
 * @return Gamepad handle or NULL on failure; see lvh_last_message().
 */
lvh_gp *lvh_rt_create_dualshock4(lvh_rt *runtime, int32_t usb_framing);
/** Destroy a gamepad handle and close the device. */
void lvh_gp_destroy(lvh_gp *gamepad);
int32_t lvh_gp_is_open(const lvh_gp *gamepad);
uint32_t lvh_gp_device_id(const lvh_gp *gamepad);
uint32_t lvh_gp_submit_count(const lvh_gp *gamepad);
uint16_t lvh_gp_vendor_id(const lvh_gp *gamepad);
uint16_t lvh_gp_product_id(const lvh_gp *gamepad);
int32_t lvh_gp_bus_type(const lvh_gp *gamepad);
size_t lvh_gp_input_report_size(const lvh_gp *gamepad);

/** Submit a state. Returns an lvh_error_code. */
int32_t lvh_gp_submit(lvh_gp *gamepad, const lvh_gamepad_state *state);
/**
 * @brief Copy the most recent packed input report.
 *
 * @param out Destination buffer, or NULL to query the length.
 * @param cap Capacity of @p out in bytes.
 * @return Full report length in bytes (may exceed @p cap).
 */
size_t lvh_gp_last_input_report(lvh_gp *gamepad, uint8_t *out, size_t cap);

/* ------------------------------------------------------------------- status */

/** Human-readable message from the last failure on this thread. */
const char *lvh_last_message(void);

/* ------------------------------------------------------------------ buttons */

/** Bitmask with every GamepadButton bit set. */
uint32_t lvh_button_all_mask(void);
/** Bit for a button name such as "a", "dpad_up", "touchpad"; 0 when unknown. */
uint32_t lvh_button_bit(const char *name);
/** Name for a single button bit; NULL when the bit is not a known button. */
const char *lvh_button_name(uint32_t bit);
/** Count of logical buttons in the state model. */
uint32_t lvh_button_count(void);

/* ------------------------------------------------------------------ reports */

/** Packed DualShock 4 input report size for the requested framing. */
size_t lvh_dualshock4_report_size(int32_t usb_framing);

/** DualShock 4 USB identity used to build a matching virtual/hardware device. */
typedef struct {
  uint16_t vendor_id;
  uint16_t product_id;
  uint16_t version;
  uint8_t report_id;
  uint32_t input_report_size;
  uint32_t output_report_size;
  uint32_t descriptor_size;
} lvh_dualshock4_info;

/**
 * @brief Read the DualShock 4 profile identity and descriptor size.
 *
 * @return lvh_error_code (LVH_OK on success).
 */
int32_t lvh_dualshock4_info_get(int32_t usb_framing, lvh_dualshock4_info *out);

/**
 * @brief Copy the DualShock 4 HID report descriptor.
 *
 * @param out Destination buffer, or NULL to query the length.
 * @param cap Capacity of @p out in bytes.
 * @return Full descriptor length in bytes (may exceed @p cap).
 */
size_t lvh_dualshock4_descriptor(int32_t usb_framing, uint8_t *out, size_t cap);
/**
 * @brief Pack a DualShock 4 input report without creating a device.
 *
 * Uses the public lvh::reports::pack_input_report API.
 *
 * @return Full report length in bytes (may exceed @p cap).
 */
size_t lvh_pack_dualshock4_report(int32_t usb_framing, const lvh_gamepad_state *state, uint8_t *out, size_t cap);


/* ------------------------------------------------------------------ license */

/**
 * @brief Read stored license status.
 *
 * @param state_out Receives an lvh_license_state value.
 * @param message_out Receives the status message (NUL-terminated, truncated to @p message_cap).
 * @param plan_out Receives the plan name, when the service reports one.
 * @return lvh_error_code for the transport-level result.
 */
int32_t lvh_license_status(
  int32_t *state_out,
  char *message_out,
  size_t message_cap,
  char *plan_out,
  size_t plan_cap);
/** Activate a license key on this machine. */
int32_t lvh_license_activate(const char *license_key, char *message_out, size_t message_cap);
/** Revalidate the stored license with the provider. */
int32_t lvh_license_validate(char *message_out, size_t message_cap);
/** Deactivate the stored license on this machine. */
int32_t lvh_license_deactivate(char *message_out, size_t message_cap);

#ifdef __cplusplus
}
#endif

#endif /* LVH_SHIM_H */
