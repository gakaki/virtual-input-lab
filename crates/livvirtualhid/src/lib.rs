//! Safe Rust bindings for the [`libvirtualhid`](https://github.com/LizardByte/libvirtualhid)
//! virtual HID library, plus a small key-mapping helper.
//!
//! Two backends are available:
//!
//! * [`Backend::Fake`] - an in-memory backend that works on every platform and
//!   is the default. It fully exercises state normalization, DualShock 4 report
//!   packing, and `submit` bookkeeping without touching the operating system.
//! * [`Backend::Platform`] - the native backend. On macOS this needs the signed,
//!   licensed *Virtual HID Broker* with Apple's virtual HID entitlement before
//!   `create_dualshock4` succeeds; otherwise it fails with
//!   [`ErrorCode::BackendUnavailable`] or [`ErrorCode::LicenseRequired`].
//!
//! ```
//! use livvirtualhid::{Framing, GamepadState, Runtime};
//!
//! let runtime = Runtime::new().unwrap();
//! let gamepad = runtime.create_dualshock4(Framing::Bluetooth).unwrap();
//! let state = GamepadState::with_all_buttons();
//! gamepad.submit(&state).unwrap();
//! assert!(!gamepad.last_input_report().is_empty());
//! ```

use std::fmt;
use std::sync::Arc;

pub mod keymap;

use libvirtualhid_sys as sys;

/// Convenient result alias for the crate.
pub type Result<T> = std::result::Result<T, LvhError>;

/// Error categories reported by libvirtualhid operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Ok,
    InvalidArgument,
    BackendUnavailable,
    DeviceClosed,
    UnsupportedProfile,
    LicenseRequired,
    LicenseInvalid,
    ActivationLimitReached,
    NetworkUnavailable,
    BackendFailure,
    /// A category introduced by a newer native library.
    Unknown(i32),
}

impl ErrorCode {
    fn from_raw(code: i32) -> Self {
        match code {
            sys::LVH_OK => Self::Ok,
            sys::LVH_INVALID_ARGUMENT => Self::InvalidArgument,
            sys::LVH_BACKEND_UNAVAILABLE => Self::BackendUnavailable,
            sys::LVH_DEVICE_CLOSED => Self::DeviceClosed,
            sys::LVH_UNSUPPORTED_PROFILE => Self::UnsupportedProfile,
            sys::LVH_LICENSE_REQUIRED => Self::LicenseRequired,
            sys::LVH_LICENSE_INVALID => Self::LicenseInvalid,
            sys::LVH_ACTIVATION_LIMIT_REACHED => Self::ActivationLimitReached,
            sys::LVH_NETWORK_UNAVAILABLE => Self::NetworkUnavailable,
            sys::LVH_BACKEND_FAILURE => Self::BackendFailure,
            other => Self::Unknown(other),
        }
    }

    /// Numeric value matching `lvh::ErrorCode`.
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::Ok => 0,
            Self::InvalidArgument => 1,
            Self::BackendUnavailable => 2,
            Self::DeviceClosed => 3,
            Self::UnsupportedProfile => 4,
            Self::LicenseRequired => 5,
            Self::LicenseInvalid => 6,
            Self::ActivationLimitReached => 7,
            Self::NetworkUnavailable => 8,
            Self::BackendFailure => 9,
            Self::Unknown(code) => code,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Ok => "ok",
            Self::InvalidArgument => "invalid_argument",
            Self::BackendUnavailable => "backend_unavailable",
            Self::DeviceClosed => "device_closed",
            Self::UnsupportedProfile => "unsupported_profile",
            Self::LicenseRequired => "license_required",
            Self::LicenseInvalid => "license_invalid",
            Self::ActivationLimitReached => "activation_limit_reached",
            Self::NetworkUnavailable => "network_unavailable",
            Self::BackendFailure => "backend_failure",
            Self::Unknown(code) => return write!(formatter, "unknown({code})"),
        };
        formatter.write_str(text)
    }
}

/// A libvirtualhid error carrying its category and human-readable message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LvhError {
    pub code: ErrorCode,
    pub message: String,
}

impl LvhError {
    fn from_code(code: i32) -> Self {
        Self {
            code: ErrorCode::from_raw(code),
            message: sys::last_message(),
        }
    }
}

impl fmt::Display for LvhError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for LvhError {}

/// Backend implementation selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// In-memory backend usable on every platform.
    Fake,
    /// Native backend for the current operating system.
    Platform,
}

impl Backend {
    fn as_raw(self) -> i32 {
        match self {
            Self::Fake => sys::LVH_BACKEND_FAKE,
            Self::Platform => sys::LVH_BACKEND_PLATFORM,
        }
    }
}

/// Transport framing advertised by a device profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framing {
    /// USB HID framing.
    Usb,
    /// Bluetooth HID framing.
    Bluetooth,
}

impl Framing {
    fn as_raw(self) -> i32 {
        match self {
            Self::Usb => 1,
            Self::Bluetooth => 0,
        }
    }
}

/// Logical buttons accepted by the gamepad state model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Button {
    A = 0,
    B = 1,
    X = 2,
    Y = 3,
    Back = 4,
    Start = 5,
    Guide = 6,
    LeftStick = 7,
    RightStick = 8,
    LeftShoulder = 9,
    RightShoulder = 10,
    DpadUp = 11,
    DpadDown = 12,
    DpadLeft = 13,
    DpadRight = 14,
    Misc1 = 15,
    Touchpad = 16,
    Paddle1 = 17,
    Paddle2 = 18,
    Paddle3 = 19,
    Paddle4 = 20,
}

/// Every logical button, in enum order.
pub const ALL_BUTTONS: [Button; 21] = [
    Button::A,
    Button::B,
    Button::X,
    Button::Y,
    Button::Back,
    Button::Start,
    Button::Guide,
    Button::LeftStick,
    Button::RightStick,
    Button::LeftShoulder,
    Button::RightShoulder,
    Button::DpadUp,
    Button::DpadDown,
    Button::DpadLeft,
    Button::DpadRight,
    Button::Misc1,
    Button::Touchpad,
    Button::Paddle1,
    Button::Paddle2,
    Button::Paddle3,
    Button::Paddle4,
];

impl Button {
    /// Bit used by [`GamepadState::buttons`].
    pub const fn bit(self) -> u32 {
        1 << (self as u32)
    }

    /// Canonical lowercase name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B => "b",
            Self::X => "x",
            Self::Y => "y",
            Self::Back => "back",
            Self::Start => "start",
            Self::Guide => "guide",
            Self::LeftStick => "left_stick",
            Self::RightStick => "right_stick",
            Self::LeftShoulder => "left_shoulder",
            Self::RightShoulder => "right_shoulder",
            Self::DpadUp => "dpad_up",
            Self::DpadDown => "dpad_down",
            Self::DpadLeft => "dpad_left",
            Self::DpadRight => "dpad_right",
            Self::Misc1 => "misc1",
            Self::Touchpad => "touchpad",
            Self::Paddle1 => "paddle1",
            Self::Paddle2 => "paddle2",
            Self::Paddle3 => "paddle3",
            Self::Paddle4 => "paddle4",
        }
    }

    /// Look up a canonical button name (case-insensitive).
    pub fn from_name(name: &str) -> Option<Self> {
        let name = name.trim().to_ascii_lowercase();
        ALL_BUTTONS.into_iter().find(|button| button.name() == name)
    }
}

impl fmt::Display for Button {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// Every logical button pressed, as a bitmask.
pub const fn all_buttons_mask() -> u32 {
    (1 << ALL_BUTTONS.len()) - 1
}

/// Number of logical buttons in the state model.
pub const fn button_count() -> u32 {
    ALL_BUTTONS.len() as u32
}

/// Normalized gamepad input state.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GamepadState {
    /// Bitmask of pressed buttons; see [`Button::bit`].
    pub buttons: u32,
    /// Left stick `(x, y)` in `[-1.0, 1.0]`.
    pub left_stick: (f32, f32),
    /// Right stick `(x, y)` in `[-1.0, 1.0]`.
    pub right_stick: (f32, f32),
    /// Left trigger in `[0.0, 1.0]`.
    pub left_trigger: f32,
    /// Right trigger in `[0.0, 1.0]`.
    pub right_trigger: f32,
}

impl GamepadState {
    /// A neutral state with no buttons and centered sticks.
    pub fn neutral() -> Self {
        Self::default()
    }

    /// A state with every logical button pressed and both triggers fully down.
    pub fn with_all_buttons() -> Self {
        Self {
            buttons: all_buttons_mask(),
            left_trigger: 1.0,
            right_trigger: 1.0,
            ..Self::default()
        }
    }

    /// Press a button.
    pub fn press(&mut self, button: Button) -> &mut Self {
        self.buttons |= button.bit();
        self
    }

    /// Release a button.
    pub fn release(&mut self, button: Button) -> &mut Self {
        self.buttons &= !button.bit();
        self
    }

    /// Whether a button is currently pressed.
    pub fn is_pressed(&self, button: Button) -> bool {
        self.buttons & button.bit() != 0
    }

    /// Every pressed button, in enum order.
    pub fn pressed_buttons(&self) -> Vec<Button> {
        ALL_BUTTONS
            .into_iter()
            .filter(|button| self.is_pressed(*button))
            .collect()
    }

    fn to_raw(self) -> sys::lvh_gamepad_state {
        sys::lvh_gamepad_state {
            buttons: self.buttons,
            left_x: self.left_stick.0,
            left_y: self.left_stick.1,
            right_x: self.right_stick.0,
            right_y: self.right_stick.1,
            left_trigger: self.left_trigger,
            right_trigger: self.right_trigger,
        }
    }
}

/// Owns the runtime's native handle and closes every device when dropped.
struct RuntimeInner {
    handle: *mut sys::lvh_rt,
}

// The native runtime has no documented thread affinity and the shim only mutates
// per-runtime state, so sharing the handle across threads is allowed.
unsafe impl Send for RuntimeInner {}
unsafe impl Sync for RuntimeInner {}

impl Drop for RuntimeInner {
    fn drop(&mut self) {
        unsafe { sys::lvh_rt_destroy(self.handle) };
    }
}

/// A live libvirtualhid runtime that creates virtual devices.
#[derive(Clone)]
pub struct Runtime {
    inner: Arc<RuntimeInner>,
}

impl Runtime {
    /// Create a runtime using the in-memory [`Backend::Fake`] backend.
    pub fn new() -> Result<Self> {
        Self::with_backend(Backend::Fake)
    }

    /// Create a runtime using the native [`Backend::Platform`] backend.
    pub fn platform() -> Result<Self> {
        Self::with_backend(Backend::Platform)
    }

    /// Create a runtime using an explicit backend.
    pub fn with_backend(backend: Backend) -> Result<Self> {
        let handle = unsafe { sys::lvh_rt_create(backend.as_raw()) };
        if handle.is_null() {
            return Err(LvhError::from_code(sys::LVH_BACKEND_FAILURE));
        }
        Ok(Self {
            inner: Arc::new(RuntimeInner { handle }),
        })
    }

    /// Backend name such as `"fake"`.
    pub fn backend_name(&self) -> String {
        unsafe { sys::cstr_to_string(sys::lvh_rt_backend_name(self.inner.handle)) }
    }

    /// Whether the backend can create virtual HID devices.
    pub fn supports_virtual_hid(&self) -> bool {
        unsafe { sys::lvh_rt_supports_virtual_hid(self.inner.handle) != 0 }
    }

    /// Whether the backend can create gamepads.
    pub fn supports_gamepad(&self) -> bool {
        unsafe { sys::lvh_rt_supports_gamepad(self.inner.handle) != 0 }
    }

    /// Number of devices currently owned by this runtime.
    pub fn active_device_count(&self) -> i32 {
        unsafe { sys::lvh_rt_active_device_count(self.inner.handle) }
    }

    /// Create a DualShock 4 gamepad.
    pub fn create_dualshock4(&self, framing: Framing) -> Result<Gamepad> {
        let handle = unsafe { sys::lvh_rt_create_dualshock4(self.inner.handle, framing.as_raw()) };
        if handle.is_null() {
            return Err(LvhError::from_code(sys::LVH_BACKEND_UNAVAILABLE));
        }
        Ok(Gamepad {
            handle,
            // Field order matters: the gamepad closes before the runtime drops.
            runtime: Arc::clone(&self.inner),
        })
    }
}

impl fmt::Debug for Runtime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Runtime")
            .field("backend", &self.backend_name())
            .finish()
    }
}

/// A virtual DualShock 4 gamepad handle.
pub struct Gamepad {
    handle: *mut sys::lvh_gp,
    /// Keeps the runtime alive until after this handle is destroyed.
    #[allow(dead_code)]
    runtime: Arc<RuntimeInner>,
}

impl Gamepad {
    /// Whether the device can still accept submissions.
    pub fn is_open(&self) -> bool {
        unsafe { sys::lvh_gp_is_open(self.handle) != 0 }
    }

    /// Runtime-assigned device identifier.
    pub fn device_id(&self) -> u32 {
        unsafe { sys::lvh_gp_device_id(self.handle) }
    }

    /// Number of successful submissions.
    pub fn submit_count(&self) -> u32 {
        unsafe { sys::lvh_gp_submit_count(self.handle) }
    }

    /// Advertised USB vendor id.
    pub fn vendor_id(&self) -> u16 {
        unsafe { sys::lvh_gp_vendor_id(self.handle) }
    }

    /// Advertised USB product id.
    pub fn product_id(&self) -> u16 {
        unsafe { sys::lvh_gp_product_id(self.handle) }
    }

    /// Effective transport framing.
    pub fn framing(&self) -> Framing {
        match unsafe { sys::lvh_gp_bus_type(self.handle) } {
            sys::LVH_BUS_USB => Framing::Usb,
            _ => Framing::Bluetooth,
        }
    }

    /// Expected packed input report size.
    pub fn input_report_size(&self) -> usize {
        unsafe { sys::lvh_gp_input_report_size(self.handle) }
    }

    /// Submit a normalized input state.
    pub fn submit(&self, state: &GamepadState) -> Result<()> {
        let raw = state.to_raw();
        let code = unsafe { sys::lvh_gp_submit(self.handle, &raw) };
        if code == sys::LVH_OK {
            Ok(())
        } else {
            Err(LvhError::from_code(code))
        }
    }

    /// The most recently packed input report.
    pub fn last_input_report(&self) -> Vec<u8> {
        let length = unsafe { sys::lvh_gp_last_input_report(self.handle, std::ptr::null_mut(), 0) };
        if length == 0 {
            return Vec::new();
        }
        let mut buffer = vec![0u8; length];
        let written = unsafe { sys::lvh_gp_last_input_report(self.handle, buffer.as_mut_ptr(), buffer.len()) };
        buffer.truncate(written);
        buffer
    }
}

impl Drop for Gamepad {
    fn drop(&mut self) {
        unsafe { sys::lvh_gp_destroy(self.handle) };
    }
}

impl fmt::Debug for Gamepad {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Gamepad")
            .field("device_id", &self.device_id())
            .field("framing", &self.framing())
            .field("submit_count", &self.submit_count())
            .finish()
    }
}

// The handle is only ever driven from the owning thread or under the GIL, and
// the runtime keeps the underlying C++ objects alive, so moving a handle
// between threads is sound.
unsafe impl Send for Gamepad {}
// The native gamepad handle is only mutated through &self methods that the
// backend serializes; sharing the handle for reads across threads is sound.
unsafe impl Sync for Gamepad {}

/// Packed DualShock 4 input report size for a framing.
pub fn dualshock4_report_size(framing: Framing) -> usize {
    unsafe { sys::lvh_dualshock4_report_size(framing.as_raw()) }
}

/// DualShock 4 USB identity for building a matching hardware or virtual device.
#[derive(Debug, Clone, Copy)]
pub struct DualShock4Info {
    pub vendor_id: u16,
    pub product_id: u16,
    pub version: u16,
    pub report_id: u8,
    pub input_report_size: u32,
    pub output_report_size: u32,
    pub descriptor_size: u32,
}

/// Read the DualShock 4 profile identity for a framing.
pub fn dualshock4_info(framing: Framing) -> DualShock4Info {
    let mut raw = sys::lvh_dualshock4_info::default();
    unsafe { sys::lvh_dualshock4_info_get(framing.as_raw(), &mut raw) };
    DualShock4Info {
        vendor_id: raw.vendor_id,
        product_id: raw.product_id,
        version: raw.version,
        report_id: raw.report_id,
        input_report_size: raw.input_report_size,
        output_report_size: raw.output_report_size,
        descriptor_size: raw.descriptor_size,
    }
}

/// The DualShock 4 HID report descriptor bytes.
pub fn dualshock4_descriptor(framing: Framing) -> Vec<u8> {
    let length = unsafe { sys::lvh_dualshock4_descriptor(framing.as_raw(), std::ptr::null_mut(), 0) };
    if length == 0 {
        return Vec::new();
    }
    let mut buffer = vec![0u8; length];
    let written = unsafe { sys::lvh_dualshock4_descriptor(framing.as_raw(), buffer.as_mut_ptr(), buffer.len()) };
    buffer.truncate(written);
    buffer
}

/// Pack a DualShock 4 input report without creating a device.
pub fn pack_dualshock4_report(framing: Framing, state: &GamepadState) -> Vec<u8> {
    let raw = state.to_raw();
    let length = unsafe {
        sys::lvh_pack_dualshock4_report(framing.as_raw(), &raw, std::ptr::null_mut(), 0)
    };
    if length == 0 {
        return Vec::new();
    }
    let mut buffer = vec![0u8; length];
    let written = unsafe {
        sys::lvh_pack_dualshock4_report(framing.as_raw(), &raw, buffer.as_mut_ptr(), buffer.len())
    };
    buffer.truncate(written);
    buffer
}

/// Provider-neutral machine license handling.
pub mod license {
    use super::ErrorCode;
    use libvirtualhid_sys as sys;

    /// Machine license states reported by the platform license service.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum LicenseState {
        Unavailable,
        Unlicensed,
        Licensed,
        Expired,
        Disabled,
        Invalid,
    }

    impl LicenseState {
        fn from_raw(value: i32) -> Self {
            match value {
                sys::LVH_LS_LICENSED => Self::Licensed,
                sys::LVH_LS_UNLICENSED => Self::Unlicensed,
                sys::LVH_LS_EXPIRED => Self::Expired,
                sys::LVH_LS_DISABLED => Self::Disabled,
                sys::LVH_LS_INVALID => Self::Invalid,
                _ => Self::Unavailable,
            }
        }

        /// Whether the machine has an active license.
        pub fn is_licensed(self) -> bool {
            matches!(self, Self::Licensed)
        }
    }

    /// A single license operation outcome.
    pub struct LicenseOutcome {
        pub code: ErrorCode,
        pub state: LicenseState,
        pub message: String,
        pub plan: String,
    }

    impl LicenseOutcome {
        pub fn is_ok(&self) -> bool {
            self.code == ErrorCode::Ok
        }
    }

    /// Run a message-returning shim call and collect its message buffer.
    fn collect_message(mut operation: impl FnMut(&mut [u8]) -> i32) -> (i32, String) {
        let mut buffer = vec![0u8; 1024];
        let code = operation(&mut buffer);
        let length = buffer.iter().position(|byte| *byte == 0).unwrap_or(buffer.len());
        buffer.truncate(length);
        (code, String::from_utf8_lossy(&buffer).into_owned())
    }

    /// Read the stored machine license status.
    pub fn status() -> LicenseOutcome {
        let mut state = sys::LVH_LS_UNAVAILABLE;
        let mut message = vec![0u8; 1024];
        let mut plan = vec![0u8; 256];
        let code = unsafe {
            sys::lvh_license_status(
                &mut state,
                message.as_mut_ptr().cast(),
                message.len(),
                plan.as_mut_ptr().cast(),
                plan.len(),
            )
        };
        message.truncate(message.iter().position(|byte| *byte == 0).unwrap_or(message.len()));
        plan.truncate(plan.iter().position(|byte| *byte == 0).unwrap_or(plan.len()));
        LicenseOutcome {
            code: ErrorCode::from_raw(code),
            state: LicenseState::from_raw(state),
            message: String::from_utf8_lossy(&message).into_owned(),
            plan: String::from_utf8_lossy(&plan).into_owned(),
        }
    }

    /// Activate a license key on this machine.
    pub fn activate(license_key: &str) -> LicenseOutcome {
        let key = std::ffi::CString::new(license_key).unwrap_or_default();
        let (code, message) = collect_message(|buffer| unsafe {
            sys::lvh_license_activate(key.as_ptr(), buffer.as_mut_ptr().cast(), buffer.len())
        });
        LicenseOutcome {
            code: ErrorCode::from_raw(code),
            state: LicenseState::Unavailable,
            message,
            plan: String::new(),
        }
    }

    /// Revalidate the stored license with the provider.
    pub fn validate() -> LicenseOutcome {
        let (code, message) = collect_message(|buffer| unsafe {
            sys::lvh_license_validate(buffer.as_mut_ptr().cast(), buffer.len())
        });
        LicenseOutcome {
            code: ErrorCode::from_raw(code),
            state: LicenseState::Unavailable,
            message,
            plan: String::new(),
        }
    }

    /// Deactivate the stored license on this machine.
    pub fn deactivate() -> LicenseOutcome {
        let (code, message) = collect_message(|buffer| unsafe {
            sys::lvh_license_deactivate(buffer.as_mut_ptr().cast(), buffer.len())
        });
        LicenseOutcome {
            code: ErrorCode::from_raw(code),
            state: LicenseState::Unavailable,
            message,
            plan: String::new(),
        }
    }
}
