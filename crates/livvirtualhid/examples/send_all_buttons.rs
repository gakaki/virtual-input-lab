//! Send every DualShock 4 button once through libvirtualhid.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p livvirtualhid --example send_all_buttons
//! ```
//!
//! With the default in-memory backend the whole pipeline runs on any machine.
//! Pass `--platform` to target the native backend: on macOS that needs the
//! signed, licensed Virtual HID Broker and Apple's virtual HID entitlement.

use livvirtualhid::{pack_dualshock4_report, dualshock4_report_size, keymap, Framing, GamepadState, Runtime};

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn main() {
    let use_platform = std::env::args().any(|argument| argument == "--platform");

    let runtime = if use_platform {
        Runtime::platform().expect("create platform runtime")
    } else {
        Runtime::new().expect("create runtime")
    };

    println!("backend            : {}", runtime.backend_name());
    println!("supports_gamepad   : {}", runtime.supports_gamepad());
    println!("supports_virtual_hid: {}", runtime.supports_virtual_hid());

    let gamepad = match runtime.create_dualshock4(Framing::Bluetooth) {
        Ok(gamepad) => gamepad,
        Err(error) => {
            eprintln!("could not create DualShock 4: {error}");
            eprintln!("(on macOS this normally means the licensed broker is not installed)");
            std::process::exit(2);
        }
    };

    println!(
        "gamepad            : id={} vid=0x{:04x} pid=0x{:04x} report={}B framing={:?}",
        gamepad.device_id(),
        gamepad.vendor_id(),
        gamepad.product_id(),
        gamepad.input_report_size(),
        gamepad.framing(),
    );

    // Press every logical button and fully pull both triggers.
    let all = GamepadState::with_all_buttons();
    println!("pressed buttons    : {} / {}", all.button_count(), livvirtualhid::button_count());

    for framing in [Framing::Bluetooth, Framing::Usb] {
        let report = pack_dualshock4_report(framing, &all);
        println!(
            "packed {:<9} ({:>2}B, expect {:>2}B): {}",
            format!("{framing:?}"),
            report.len(),
            dualshock4_report_size(framing),
            hex(&report),
        );
    }

    match gamepad.submit(&all) {
        Ok(()) => println!("submit             : ok"),
        Err(error) => {
            eprintln!("submit failed: {error}");
            std::process::exit(3);
        }
    }
    println!("submit_count       : {}", gamepad.submit_count());
    println!("last_input_report  : {}", hex(&gamepad.last_input_report()));

    // Release everything and settle.
    let _ = gamepad.submit(&GamepadState::neutral());

    // Show the key-mapping helper.
    let mapped = keymap::state_from_keys(["cross", "circle", "square", "triangle", "l1", "r2", "dpad_up"]);
    println!(
        "keymap demo        : {:?} left_trigger={} right_trigger={}",
        mapped.pressed_buttons().iter().map(|b| b.name()).collect::<Vec<_>>(),
        mapped.left_trigger,
        mapped.right_trigger,
    );
}
