//! Raw FFI declarations for the libvirtualhid C ABI shim.
//!
//! These bindings are unsafe and intentionally thin; prefer the safe
//! `livvirtualhid` crate.

#![allow(non_camel_case_types)]

use std::os::raw::{c_char, c_int};

pub type lvh_rt = std::ffi::c_void;
pub type lvh_gp = std::ffi::c_void;

pub const LVH_OK: i32 = 0;
pub const LVH_INVALID_ARGUMENT: i32 = 1;
pub const LVH_BACKEND_UNAVAILABLE: i32 = 2;
pub const LVH_DEVICE_CLOSED: i32 = 3;
pub const LVH_UNSUPPORTED_PROFILE: i32 = 4;
pub const LVH_LICENSE_REQUIRED: i32 = 5;
pub const LVH_LICENSE_INVALID: i32 = 6;
pub const LVH_ACTIVATION_LIMIT_REACHED: i32 = 7;
pub const LVH_NETWORK_UNAVAILABLE: i32 = 8;
pub const LVH_BACKEND_FAILURE: i32 = 9;

pub const LVH_BACKEND_FAKE: i32 = 0;
pub const LVH_BACKEND_PLATFORM: i32 = 1;

pub const LVH_BUS_UNKNOWN: i32 = 0;
pub const LVH_BUS_USB: i32 = 1;
pub const LVH_BUS_BLUETOOTH: i32 = 2;

pub const LVH_LS_UNAVAILABLE: i32 = 0;
pub const LVH_LS_UNLICENSED: i32 = 1;
pub const LVH_LS_LICENSED: i32 = 2;
pub const LVH_LS_EXPIRED: i32 = 3;
pub const LVH_LS_DISABLED: i32 = 4;
pub const LVH_LS_INVALID: i32 = 5;

/// Flattened gamepad state mirrored to `lvh_gamepad_state`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct lvh_gamepad_state {
    pub buttons: u32,
    pub left_x: f32,
    pub left_y: f32,
    pub right_x: f32,
    pub right_y: f32,
    pub left_trigger: f32,
    pub right_trigger: f32,
}

/// DualShock 4 identity mirrored to `lvh_dualshock4_info`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct lvh_dualshock4_info {
    pub vendor_id: u16,
    pub product_id: u16,
    pub version: u16,
    pub report_id: u8,
    pub input_report_size: u32,
    pub output_report_size: u32,
    pub descriptor_size: u32,
}

extern "C" {
    pub fn lvh_rt_create(backend_kind: i32) -> *mut lvh_rt;
    pub fn lvh_rt_destroy(runtime: *mut lvh_rt);
    pub fn lvh_rt_backend_kind(runtime: *const lvh_rt) -> i32;
    pub fn lvh_rt_backend_name(runtime: *const lvh_rt) -> *const c_char;
    pub fn lvh_rt_supports_gamepad(runtime: *const lvh_rt) -> i32;
    pub fn lvh_rt_supports_virtual_hid(runtime: *const lvh_rt) -> i32;
    pub fn lvh_rt_active_device_count(runtime: *const lvh_rt) -> i32;

    pub fn lvh_rt_create_dualshock4(runtime: *mut lvh_rt, usb_framing: i32) -> *mut lvh_gp;
    pub fn lvh_gp_destroy(gamepad: *mut lvh_gp);
    pub fn lvh_gp_is_open(gamepad: *const lvh_gp) -> i32;
    pub fn lvh_gp_device_id(gamepad: *const lvh_gp) -> u32;
    pub fn lvh_gp_submit_count(gamepad: *const lvh_gp) -> u32;
    pub fn lvh_gp_vendor_id(gamepad: *const lvh_gp) -> u16;
    pub fn lvh_gp_product_id(gamepad: *const lvh_gp) -> u16;
    pub fn lvh_gp_bus_type(gamepad: *const lvh_gp) -> i32;
    pub fn lvh_gp_input_report_size(gamepad: *const lvh_gp) -> usize;
    pub fn lvh_gp_submit(gamepad: *mut lvh_gp, state: *const lvh_gamepad_state) -> i32;
    pub fn lvh_gp_last_input_report(gamepad: *mut lvh_gp, out: *mut u8, cap: usize) -> usize;

    pub fn lvh_last_message() -> *const c_char;

    pub fn lvh_button_all_mask() -> u32;
    pub fn lvh_button_bit(name: *const c_char) -> u32;
    pub fn lvh_button_name(bit: u32) -> *const c_char;
    pub fn lvh_button_count() -> u32;

    pub fn lvh_dualshock4_report_size(usb_framing: i32) -> usize;
    pub fn lvh_dualshock4_info_get(usb_framing: i32, out: *mut lvh_dualshock4_info) -> i32;
    pub fn lvh_dualshock4_descriptor(usb_framing: i32, out: *mut u8, cap: usize) -> usize;
    pub fn lvh_pack_dualshock4_report(
        usb_framing: i32,
        state: *const lvh_gamepad_state,
        out: *mut u8,
        cap: usize,
    ) -> usize;

    pub fn lvh_license_status(
        state_out: *mut i32,
        message_out: *mut c_char,
        message_cap: usize,
        plan_out: *mut c_char,
        plan_cap: usize,
    ) -> i32;
    pub fn lvh_license_activate(license_key: *const c_char, message_out: *mut c_char, message_cap: usize) -> i32;
    pub fn lvh_license_validate(message_out: *mut c_char, message_cap: usize) -> i32;
    pub fn lvh_license_deactivate(message_out: *mut c_char, message_cap: usize) -> i32;

}

/// Copy the current thread's last shim message into an owned string.
pub fn last_message() -> String {
    unsafe {
        let pointer = lvh_last_message();
        if pointer.is_null() {
            return String::new();
        }
        std::ffi::CStr::from_ptr(pointer).to_string_lossy().into_owned()
    }
}

/// Read a NUL-terminated C string, returning an empty string for null.
pub unsafe fn cstr_to_string(pointer: *const c_char) -> String {
    if pointer.is_null() {
        return String::new();
    }
    std::ffi::CStr::from_ptr(pointer).to_string_lossy().into_owned()
}

// The shim is a thin, stateless-except-handle C wrapper; handles are only
// touched from the owning runtime/gamepad. Mark the raw extern block callable
// from threads so the safe wrapper can be Send/Sync.
pub const SHIM_THREAD_SAFE: bool = true;

#[allow(dead_code)]
fn _assert_int_size(value: c_int) -> c_int {
    value
}
