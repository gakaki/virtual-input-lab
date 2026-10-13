//! Python bindings (`import libvirtualhid`) built with PyO3.
//!
//! The module exposes the same virtual DualShock 4 pipeline as the Rust crate:
//! create a runtime, create a gamepad, build a state (optionally from a key
//! map), submit it, and read the packed HID report.

use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

use livvirtualhid as core;

create_exception!(libvirtualhid, LvhError, PyException);

fn to_pyerr(error: core::LvhError) -> PyErr {
    LvhError::new_err(format!("{}: {}", error.code, error.message))
}

fn parse_framing(usb: bool) -> core::Framing {
    if usb {
        core::Framing::Usb
    } else {
        core::Framing::Bluetooth
    }
}

/// A libvirtualhid runtime that creates virtual devices.
#[pyclass]
struct Runtime {
    inner: core::Runtime,
}

#[pymethods]
impl Runtime {
    /// Create a runtime. Uses the in-memory backend unless `platform` is true.
    #[new]
    #[pyo3(signature = (platform=false))]
    fn new(platform: bool) -> PyResult<Self> {
        let inner = if platform {
            core::Runtime::platform()
        } else {
            core::Runtime::new()
        }
        .map_err(to_pyerr)?;
        Ok(Self { inner })
    }

    #[getter]
    fn backend_name(&self) -> String {
        self.inner.backend_name()
    }

    #[getter]
    fn supports_gamepad(&self) -> bool {
        self.inner.supports_gamepad()
    }

    #[getter]
    fn supports_virtual_hid(&self) -> bool {
        self.inner.supports_virtual_hid()
    }

    #[getter]
    fn active_device_count(&self) -> i32 {
        self.inner.active_device_count()
    }

    /// Create a DualShock 4 gamepad. `usb=False` selects Bluetooth framing.
    #[pyo3(signature = (usb=false))]
    fn create_dualshock4(&self, usb: bool) -> PyResult<Gamepad> {
        let inner = self
            .inner
            .create_dualshock4(parse_framing(usb))
            .map_err(to_pyerr)?;
        Ok(Gamepad { inner })
    }

    fn __repr__(&self) -> String {
        format!("Runtime(backend='{}')", self.inner.backend_name())
    }
}

/// A virtual DualShock 4 gamepad.
#[pyclass]
struct Gamepad {
    inner: core::Gamepad,
}

#[pymethods]
impl Gamepad {
    #[getter]
    fn is_open(&self) -> bool {
        self.inner.is_open()
    }

    #[getter]
    fn device_id(&self) -> u32 {
        self.inner.device_id()
    }

    #[getter]
    fn submit_count(&self) -> u32 {
        self.inner.submit_count()
    }

    #[getter]
    fn vendor_id(&self) -> u16 {
        self.inner.vendor_id()
    }

    #[getter]
    fn product_id(&self) -> u16 {
        self.inner.product_id()
    }

    #[getter]
    fn bus(&self) -> String {
        match self.inner.framing() {
            core::Framing::Usb => "usb".to_owned(),
            core::Framing::Bluetooth => "bluetooth".to_owned(),
        }
    }

    #[getter]
    fn input_report_size(&self) -> usize {
        self.inner.input_report_size()
    }

    /// Submit a [`GamepadState`].
    fn submit(&self, state: &GamepadState) -> PyResult<()> {
        self.inner.submit(&state.inner).map_err(to_pyerr)
    }

    /// Build an all-buttons state and submit it in one call.
    fn submit_all_buttons(&self) -> PyResult<()> {
        self.inner
            .submit(&core::GamepadState::with_all_buttons())
            .map_err(to_pyerr)
    }

    /// Release every button and trigger.
    fn release_all(&self) -> PyResult<()> {
        self.inner
            .submit(&core::GamepadState::neutral())
            .map_err(to_pyerr)
    }

    /// The most recently packed input report as `bytes`.
    fn last_input_report<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.last_input_report())
    }

    fn __repr__(&self) -> String {
        format!(
            "Gamepad(id={}, vid=0x{:04x}, pid=0x{:04x}, submit_count={})",
            self.inner.device_id(),
            self.inner.vendor_id(),
            self.inner.product_id(),
            self.inner.submit_count(),
        )
    }
}

/// Normalized gamepad input state.
#[pyclass(name = "GamepadState")]
#[derive(Clone, Default)]
struct GamepadState {
    inner: core::GamepadState,
}

#[pymethods]
impl GamepadState {
    #[new]
    fn new() -> Self {
        Self::default()
    }

    /// A neutral state with no buttons pressed.
    #[staticmethod]
    fn neutral() -> Self {
        Self {
            inner: core::GamepadState::neutral(),
        }
    }

    /// A state with every logical button pressed and both triggers down.
    #[staticmethod]
    fn all_buttons() -> Self {
        Self {
            inner: core::GamepadState::with_all_buttons(),
        }
    }

    /// Build a state from friendly input names (see `default_keymap`).
    #[staticmethod]
    fn from_keys(keys: Vec<String>) -> Self {
        Self {
            inner: core::keymap::state_from_keys(keys),
        }
    }

    /// Press a button by canonical or mapped name.
    fn press(&mut self, button: &str) -> PyResult<()> {
        let resolved = resolve_button(button)?;
        self.inner.press(resolved);
        Ok(())
    }

    /// Release a button by canonical or mapped name.
    fn release(&mut self, button: &str) -> PyResult<()> {
        let resolved = resolve_button(button)?;
        self.inner.release(resolved);
        Ok(())
    }

    /// Whether a button is pressed.
    fn is_pressed(&self, button: &str) -> PyResult<bool> {
        Ok(self.inner.is_pressed(resolve_button(button)?))
    }

    /// Names of every pressed button, in enum order.
    fn pressed(&self) -> Vec<String> {
        self.inner
            .pressed_buttons()
            .into_iter()
            .map(|button| button.name().to_owned())
            .collect()
    }

    #[getter]
    fn buttons(&self) -> u32 {
        self.inner.buttons
    }

    #[setter]
    fn set_buttons(&mut self, value: u32) {
        self.inner.buttons = value;
    }

    #[getter]
    fn left_trigger(&self) -> f32 {
        self.inner.left_trigger
    }

    #[setter]
    fn set_left_trigger(&mut self, value: f32) {
        self.inner.left_trigger = value;
    }

    #[getter]
    fn right_trigger(&self) -> f32 {
        self.inner.right_trigger
    }

    #[setter]
    fn set_right_trigger(&mut self, value: f32) {
        self.inner.right_trigger = value;
    }

    #[getter]
    fn left_stick(&self) -> (f32, f32) {
        self.inner.left_stick
    }

    #[setter]
    fn set_left_stick(&mut self, value: (f32, f32)) {
        self.inner.left_stick = value;
    }

    #[getter]
    fn right_stick(&self) -> (f32, f32) {
        self.inner.right_stick
    }

    #[setter]
    fn set_right_stick(&mut self, value: (f32, f32)) {
        self.inner.right_stick = value;
    }

    fn __repr__(&self) -> String {
        format!(
            "GamepadState(pressed={:?}, left_trigger={}, right_trigger={})",
            self.pressed(),
            self.inner.left_trigger,
            self.inner.right_trigger,
        )
    }
}

fn resolve_button(name: &str) -> PyResult<core::Button> {
    if let Some(button) = core::Button::from_name(name) {
        return Ok(button);
    }
    let map = core::keymap::default_keymap();
    map.get(name.trim().to_ascii_lowercase().as_str())
        .copied()
        .ok_or_else(|| PyValueError::new_err(format!("unknown button name: {name}")))
}

/// Canonical button names in enum order.
#[pyfunction]
fn button_names() -> Vec<String> {
    core::ALL_BUTTONS
        .iter()
        .map(|button| button.name().to_owned())
        .collect()
}

/// Number of logical buttons.
#[pyfunction]
fn button_count() -> u32 {
    core::button_count()
}

/// Bitmask with every logical button set.
#[pyfunction]
fn all_buttons_mask() -> u32 {
    core::all_buttons_mask()
}

/// Bit value for a button name.
#[pyfunction]
fn button_bit(name: &str) -> u32 {
    if let Some(button) = core::Button::from_name(name) {
        return button.bit();
    }
    core::keymap::default_keymap()
        .get(name.trim().to_ascii_lowercase().as_str())
        .map(|button| button.bit())
        .unwrap_or(0)
}

/// The default key-to-button mapping table.
#[pyfunction]
fn default_keymap<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    for (name, button) in core::keymap::default_keymap() {
        dict.set_item(name, button.name())?;
    }
    Ok(dict)
}

/// Packed DualShock 4 input report size.
#[pyfunction]
#[pyo3(signature = (usb=false))]
fn dualshock4_report_size(usb: bool) -> usize {
    core::dualshock4_report_size(parse_framing(usb))
}

/// Pack a DualShock 4 input report without creating a device.
#[pyfunction]
#[pyo3(signature = (state, usb=false))]
fn pack_dualshock4_report<'py>(py: Python<'py>, state: &GamepadState, usb: bool) -> Bound<'py, PyBytes> {
    PyBytes::new(py, &core::pack_dualshock4_report(parse_framing(usb), &state.inner))
}

fn outcome_to_dict<'py>(
    py: Python<'py>,
    outcome: core::license::LicenseOutcome,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("ok", outcome.is_ok())?;
    dict.set_item("code", outcome.code.to_string())?;
    dict.set_item("state", format!("{:?}", outcome.state))?;
    dict.set_item("licensed", outcome.state.is_licensed())?;
    dict.set_item("plan", outcome.plan)?;
    dict.set_item("message", outcome.message)?;
    Ok(dict)
}

/// Read the stored machine license status.
#[pyfunction]
fn license_status<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
    outcome_to_dict(py, core::license::status())
}

/// Activate a license key on this machine.
#[pyfunction]
fn activate_license<'py>(py: Python<'py>, license_key: &str) -> PyResult<Bound<'py, PyDict>> {
    outcome_to_dict(py, core::license::activate(license_key))
}

/// Revalidate the stored license with the provider.
#[pyfunction]
fn validate_license<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
    outcome_to_dict(py, core::license::validate())
}

/// Deactivate the stored license on this machine.
#[pyfunction]
fn deactivate_license<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
    outcome_to_dict(py, core::license::deactivate())
}

/// The libvirtualhid Python bindings.
#[pymodule]
fn libvirtualhid(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Runtime>()?;
    module.add_class::<Gamepad>()?;
    module.add_class::<GamepadState>()?;
    module.add("LvhError", module.py().get_type::<LvhError>())?;

    module.add_function(wrap_pyfunction!(button_names, module)?)?;
    module.add_function(wrap_pyfunction!(button_count, module)?)?;
    module.add_function(wrap_pyfunction!(all_buttons_mask, module)?)?;
    module.add_function(wrap_pyfunction!(button_bit, module)?)?;
    module.add_function(wrap_pyfunction!(default_keymap, module)?)?;
    module.add_function(wrap_pyfunction!(dualshock4_report_size, module)?)?;
    module.add_function(wrap_pyfunction!(pack_dualshock4_report, module)?)?;
    module.add_function(wrap_pyfunction!(license_status, module)?)?;
    module.add_function(wrap_pyfunction!(activate_license, module)?)?;
    module.add_function(wrap_pyfunction!(validate_license, module)?)?;
    module.add_function(wrap_pyfunction!(deactivate_license, module)?)?;

    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
