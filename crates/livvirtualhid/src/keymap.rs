//! A tiny key-to-button mapping helper ("按键映射") on top of [`GamepadState`].
//!
//! The map accepts friendly names from several controller/keyboard families so
//! a caller can describe an input set as plain strings and get a ready-to-submit
//! [`GamepadState`] back.

use crate::{Button, GamepadState};
use std::collections::HashMap;

/// A mapping from input names to logical gamepad buttons.
pub type KeyMap = HashMap<&'static str, Button>;

/// Build the default mapping table.
///
/// Names are lower-case and include DualShock/DualSense labels, Xbox labels, and
/// a few keyboard keys for convenience.
pub fn default_keymap() -> KeyMap {
    let mut map = KeyMap::new();

    // Face buttons: south/east/west/north.
    for name in ["a", "cross", "space"] {
        map.insert(name, Button::A);
    }
    for name in ["b", "circle", "o"] {
        map.insert(name, Button::B);
    }
    for name in ["x", "square", "y_key"] {
        map.insert(name, Button::X);
    }
    for name in ["y", "triangle"] {
        map.insert(name, Button::Y);
    }

    // Shoulders.
    for name in ["l1", "left_shoulder", "lb"] {
        map.insert(name, Button::LeftShoulder);
    }
    for name in ["r1", "right_shoulder", "rb"] {
        map.insert(name, Button::RightShoulder);
    }

    // Stick presses.
    for name in ["l3", "left_stick", "left_stick_press"] {
        map.insert(name, Button::LeftStick);
    }
    for name in ["r3", "right_stick", "right_stick_press"] {
        map.insert(name, Button::RightStick);
    }

    // Menu / system / touchpad.
    for name in ["share", "back", "select"] {
        map.insert(name, Button::Back);
    }
    for name in ["options", "start", "menu"] {
        map.insert(name, Button::Start);
    }
    for name in ["ps", "guide", "home"] {
        map.insert(name, Button::Guide);
    }
    for name in ["touchpad", "touch"] {
        map.insert(name, Button::Touchpad);
    }

    // D-pad.
    map.insert("dpad_up", Button::DpadUp);
    map.insert("dpad_down", Button::DpadDown);
    map.insert("dpad_left", Button::DpadLeft);
    map.insert("dpad_right", Button::DpadRight);
    for (name, button) in [
        ("up", Button::DpadUp),
        ("down", Button::DpadDown),
        ("left", Button::DpadLeft),
        ("right", Button::DpadRight),
    ] {
        map.insert(name, button);
    }

    map
}

/// Trigger-like names that are analog rather than digital buttons.
fn trigger_for(name: &str) -> Option<Trigger> {
    match name {
        "l2" | "left_trigger" | "lt" | "trigger_left" => Some(Trigger::Left),
        "r2" | "right_trigger" | "rt" | "trigger_right" => Some(Trigger::Right),
        _ => None,
    }
}

enum Trigger {
    Left,
    Right,
}

/// Build a [`GamepadState`] from a set of pressed input names.
///
/// Unknown names are ignored, analog trigger names (`l2`/`r2`) fully depress the
/// matching trigger, and everything else is matched against [`default_keymap`].
///
/// ```
/// use livvirtualhid::keymap::state_from_keys;
///
/// let state = state_from_keys(["cross", "r1", "dpad_up", "l2"]);
/// assert!(state.button_count() >= 3);
/// assert_eq!(state.left_trigger, 1.0);
/// ```
pub fn state_from_keys<I, S>(keys: I) -> GamepadState
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let map = default_keymap();
    let mut state = GamepadState::neutral();
    for key in keys {
        let name = key.as_ref().trim().to_ascii_lowercase();
        if let Some(trigger) = trigger_for(&name) {
            match trigger {
                Trigger::Left => state.left_trigger = 1.0,
                Trigger::Right => state.right_trigger = 1.0,
            }
            continue;
        }
        if let Some(button) = map.get(name.as_str()) {
            state.press(*button);
        }
    }
    state
}

/// Convenience for [`GamepadState::with_all_buttons`].
pub fn all_buttons() -> GamepadState {
    GamepadState::with_all_buttons()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_friendly_names() {
        let state = state_from_keys(["cross", "circle", "l2", "dpad_left"]);
        assert!(state.is_pressed(Button::A));
        assert!(state.is_pressed(Button::B));
        assert!(state.is_pressed(Button::DpadLeft));
        assert_eq!(state.left_trigger, 1.0);
    }

    #[test]
    fn ignores_unknown_names() {
        let state = state_from_keys(["definitely_not_a_button"]);
        assert_eq!(state.buttons, 0);
    }
}

impl GamepadState {
    /// Number of pressed buttons; convenience for examples.
    pub fn button_count(&self) -> usize {
        self.pressed_buttons().len()
    }
}
