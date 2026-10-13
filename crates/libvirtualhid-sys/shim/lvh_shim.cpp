/**
 * @file lvh_shim.cpp
 * @brief Flat C ABI over the libvirtualhid C++ public API.
 */
#include "lvh_shim.h"

#include <libvirtualhid/libvirtualhid.hpp>

#include <cstring>
#include <memory>
#include <string>
#include <string_view>

namespace {

  /** Message describing the last failure on the calling thread. */
  thread_local std::string g_last_message;

  const char *store_message(const std::string &message) {
    g_last_message = message;
    return g_last_message.c_str();
  }

  struct ButtonName {
    const char *name;
    lvh::GamepadButton button;
  };

  constexpr ButtonName button_names[] = {
    {"a", lvh::GamepadButton::a},
    {"b", lvh::GamepadButton::b},
    {"x", lvh::GamepadButton::x},
    {"y", lvh::GamepadButton::y},
    {"back", lvh::GamepadButton::back},
    {"start", lvh::GamepadButton::start},
    {"guide", lvh::GamepadButton::guide},
    {"left_stick", lvh::GamepadButton::left_stick},
    {"right_stick", lvh::GamepadButton::right_stick},
    {"left_shoulder", lvh::GamepadButton::left_shoulder},
    {"right_shoulder", lvh::GamepadButton::right_shoulder},
    {"dpad_up", lvh::GamepadButton::dpad_up},
    {"dpad_down", lvh::GamepadButton::dpad_down},
    {"dpad_left", lvh::GamepadButton::dpad_left},
    {"dpad_right", lvh::GamepadButton::dpad_right},
    {"misc1", lvh::GamepadButton::misc1},
    {"touchpad", lvh::GamepadButton::touchpad},
    {"paddle1", lvh::GamepadButton::paddle1},
    {"paddle2", lvh::GamepadButton::paddle2},
    {"paddle3", lvh::GamepadButton::paddle3},
    {"paddle4", lvh::GamepadButton::paddle4},
  };

  constexpr int button_count = static_cast<int>(sizeof(button_names) / sizeof(button_names[0]));

  lvh::GamepadState to_gamepad_state(const lvh_gamepad_state &in) {
    lvh::GamepadState state;
    for (int index = 0; index < button_count; ++index) {
      if ((in.buttons & (1u << index)) != 0u) {
        state.buttons.set(static_cast<lvh::GamepadButton>(index), true);
      }
    }
    state.left_stick = {in.left_x, in.left_y};
    state.right_stick = {in.right_x, in.right_y};
    state.left_trigger = in.left_trigger;
    state.right_trigger = in.right_trigger;
    return state;
  }

  lvh::DeviceProfile dualshock4_profile(int32_t usb_framing) {
    return usb_framing != 0 ? lvh::profiles::dualshock4_usb() : lvh::profiles::dualshock4_bluetooth();
  }

  void copy_message(const std::string &message, char *out, size_t cap) {
    if (out == nullptr || cap == 0) {
      return;
    }
    const auto length = message.size() < cap - 1 ? message.size() : cap - 1;
    std::memcpy(out, message.data(), length);
    out[length] = '\0';
  }

  template <typename Result>
  int32_t finish(Result &result, std::string &message_out, char *out, size_t cap) {
    message_out = result.license.message.empty() ? result.status.message() : result.license.message;
    store_message(message_out);
    copy_message(message_out, out, cap);
    return static_cast<int32_t>(result.status.code());
  }

}  // namespace

struct lvh_rt {
  std::unique_ptr<lvh::Runtime> runtime;
};

struct lvh_gp {
  std::unique_ptr<lvh::Gamepad> gamepad;
};

extern "C" {

lvh_rt *lvh_rt_create(int32_t backend_kind) {
  lvh::RuntimeOptions options;
  options.backend = backend_kind == LVH_BACKEND_PLATFORM ? lvh::BackendKind::platform_default : lvh::BackendKind::fake;
  auto runtime = lvh::Runtime::create(options);
  if (!runtime) {
    store_message("lvh::Runtime::create returned no runtime");
    return nullptr;
  }
  auto handle = new (std::nothrow) lvh_rt {std::move(runtime)};
  if (handle == nullptr) {
    store_message("out of memory allocating runtime handle");
  }
  return handle;
}

void lvh_rt_destroy(lvh_rt *runtime) {
  delete runtime;
}

int32_t lvh_rt_backend_kind(const lvh_rt *runtime) {
  if (runtime == nullptr || !runtime->runtime) {
    return -1;
  }
  return runtime->runtime->backend_kind() == lvh::BackendKind::platform_default ? LVH_BACKEND_PLATFORM : LVH_BACKEND_FAKE;
}

const char *lvh_rt_backend_name(const lvh_rt *runtime) {
  if (runtime == nullptr || !runtime->runtime) {
    return store_message("null runtime");
  }
  return store_message(runtime->runtime->capabilities().backend_name);
}

int32_t lvh_rt_supports_gamepad(const lvh_rt *runtime) {
  if (runtime == nullptr || !runtime->runtime) {
    return 0;
  }
  return runtime->runtime->capabilities().supports_gamepad ? 1 : 0;
}

int32_t lvh_rt_supports_virtual_hid(const lvh_rt *runtime) {
  if (runtime == nullptr || !runtime->runtime) {
    return 0;
  }
  return runtime->runtime->capabilities().supports_virtual_hid ? 1 : 0;
}

int32_t lvh_rt_active_device_count(const lvh_rt *runtime) {
  if (runtime == nullptr || !runtime->runtime) {
    return 0;
  }
  return static_cast<int32_t>(runtime->runtime->active_device_count());
}

lvh_gp *lvh_rt_create_dualshock4(lvh_rt *runtime, int32_t usb_framing) {
  if (runtime == nullptr || !runtime->runtime) {
    store_message("null runtime");
    return nullptr;
  }

  auto created = runtime->runtime->create_gamepad(dualshock4_profile(usb_framing));
  if (!created) {
    store_message(created.status.message());
    return nullptr;
  }

  auto handle = new (std::nothrow) lvh_gp {std::move(created.gamepad)};
  if (handle == nullptr) {
    store_message("out of memory allocating gamepad handle");
  }
  return handle;
}

void lvh_gp_destroy(lvh_gp *gamepad) {
  delete gamepad;
}

int32_t lvh_gp_is_open(const lvh_gp *gamepad) {
  if (gamepad == nullptr || !gamepad->gamepad) {
    return 0;
  }
  return gamepad->gamepad->is_open() ? 1 : 0;
}

uint32_t lvh_gp_device_id(const lvh_gp *gamepad) {
  if (gamepad == nullptr || !gamepad->gamepad) {
    return 0;
  }
  return static_cast<uint32_t>(gamepad->gamepad->device_id());
}

uint32_t lvh_gp_submit_count(const lvh_gp *gamepad) {
  if (gamepad == nullptr || !gamepad->gamepad) {
    return 0;
  }
  return static_cast<uint32_t>(gamepad->gamepad->submit_count());
}

uint16_t lvh_gp_vendor_id(const lvh_gp *gamepad) {
  if (gamepad == nullptr || !gamepad->gamepad) {
    return 0;
  }
  return gamepad->gamepad->profile().vendor_id;
}

uint16_t lvh_gp_product_id(const lvh_gp *gamepad) {
  if (gamepad == nullptr || !gamepad->gamepad) {
    return 0;
  }
  return gamepad->gamepad->profile().product_id;
}

int32_t lvh_gp_bus_type(const lvh_gp *gamepad) {
  if (gamepad == nullptr || !gamepad->gamepad) {
    return LVH_BUS_UNKNOWN;
  }
  switch (gamepad->gamepad->profile().bus_type) {
    case lvh::BusType::usb:
      return LVH_BUS_USB;
    case lvh::BusType::bluetooth:
      return LVH_BUS_BLUETOOTH;
    default:
      return LVH_BUS_UNKNOWN;
  }
}

size_t lvh_gp_input_report_size(const lvh_gp *gamepad) {
  if (gamepad == nullptr || !gamepad->gamepad) {
    return 0;
  }
  return gamepad->gamepad->profile().input_report_size;
}

int32_t lvh_gp_submit(lvh_gp *gamepad, const lvh_gamepad_state *state) {
  if (gamepad == nullptr || !gamepad->gamepad || state == nullptr) {
    store_message("null gamepad or state");
    return LVH_INVALID_ARGUMENT;
  }
  const auto status = gamepad->gamepad->submit(to_gamepad_state(*state));
  store_message(status.message());
  return static_cast<int32_t>(status.code());
}

size_t lvh_gp_last_input_report(lvh_gp *gamepad, uint8_t *out, size_t cap) {
  if (gamepad == nullptr || !gamepad->gamepad) {
    return 0;
  }
  const auto report = gamepad->gamepad->last_input_report();
  if (out != nullptr && cap > 0) {
    const auto length = report.size() < cap ? report.size() : cap;
    std::memcpy(out, report.data(), length);
  }
  return report.size();
}

const char *lvh_last_message(void) {
  return g_last_message.c_str();
}

uint32_t lvh_button_all_mask(void) {
  uint32_t mask = 0;
  for (int index = 0; index < button_count; ++index) {
    mask |= (1u << index);
  }
  return mask;
}

uint32_t lvh_button_count(void) {
  return static_cast<uint32_t>(button_count);
}

uint32_t lvh_button_bit(const char *name) {
  if (name == nullptr) {
    return 0;
  }
  const std::string_view requested {name};
  for (int index = 0; index < button_count; ++index) {
    if (requested == button_names[index].name) {
      return (1u << index);
    }
  }
  return 0;
}

const char *lvh_button_name(uint32_t bit) {
  for (int index = 0; index < button_count; ++index) {
    if (bit == (1u << index)) {
      return button_names[index].name;
    }
  }
  return nullptr;
}

size_t lvh_dualshock4_report_size(int32_t usb_framing) {
  return dualshock4_profile(usb_framing).input_report_size;
}

int32_t lvh_dualshock4_info_get(int32_t usb_framing, lvh_dualshock4_info *out) {
  if (out == nullptr) {
    store_message("null info output");
    return LVH_INVALID_ARGUMENT;
  }
  const auto profile = dualshock4_profile(usb_framing);
  out->vendor_id = profile.vendor_id;
  out->product_id = profile.product_id;
  out->version = profile.version;
  out->report_id = profile.report_id;
  out->input_report_size = static_cast<uint32_t>(profile.input_report_size);
  out->output_report_size = static_cast<uint32_t>(profile.output_report_size);
  out->descriptor_size = static_cast<uint32_t>(profile.report_descriptor.size());
  return LVH_OK;
}

size_t lvh_dualshock4_descriptor(int32_t usb_framing, uint8_t *out, size_t cap) {
  const auto descriptor = dualshock4_profile(usb_framing).report_descriptor;
  if (out != nullptr && cap > 0) {
    const auto length = descriptor.size() < cap ? descriptor.size() : cap;
    std::memcpy(out, descriptor.data(), length);
  }
  return descriptor.size();
}

size_t lvh_pack_dualshock4_report(int32_t usb_framing, const lvh_gamepad_state *state, uint8_t *out, size_t cap) {
  if (state == nullptr) {
    return 0;
  }
  const auto profile = dualshock4_profile(usb_framing);
  const auto report = lvh::reports::pack_input_report(profile, to_gamepad_state(*state));
  if (out != nullptr && cap > 0) {
    const auto length = report.size() < cap ? report.size() : cap;
    std::memcpy(out, report.data(), length);
  }
  return report.size();
}

int32_t lvh_license_status(
  int32_t *state_out,
  char *message_out,
  size_t message_cap,
  char *plan_out,
  size_t plan_cap
) {
  auto result = lvh::get_license_status();
  if (state_out != nullptr) {
    *state_out = static_cast<int32_t>(result.license.state);
  }
  std::string message = result.license.message.empty() ? result.status.message() : result.license.message;
  store_message(message);
  copy_message(message, message_out, message_cap);
  copy_message(result.license.plan_name, plan_out, plan_cap);
  return static_cast<int32_t>(result.status.code());
}

int32_t lvh_license_activate(const char *license_key, char *message_out, size_t message_cap) {
  std::string message;
  auto result = lvh::activate_license(license_key != nullptr ? std::string_view {license_key} : std::string_view {});
  return finish(result, message, message_out, message_cap);
}

int32_t lvh_license_validate(char *message_out, size_t message_cap) {
  std::string message;
  auto result = lvh::validate_license();
  return finish(result, message, message_out, message_cap);
}

int32_t lvh_license_deactivate(char *message_out, size_t message_cap) {
  std::string message;
  auto result = lvh::deactivate_license();
  return finish(result, message, message_out, message_cap);
}

}  // extern "C"
