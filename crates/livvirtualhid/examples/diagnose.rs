//! Diagnose the libvirtualhid backend, broker, and license state.
//!
//! ```sh
//! cargo run -p livvirtualhid --example diagnose
//! ```

use livvirtualhid::{license, Backend, Framing, Runtime};

fn main() {
    println!("== backend ==");
    let platform = Runtime::with_backend(Backend::Platform).expect("platform runtime");
    println!("kind                : {}", platform.backend_name());
    println!("supports_gamepad    : {}", platform.supports_gamepad());
    println!("supports_virtual_hid: {}", platform.supports_virtual_hid());
    println!("active_device_count : {}", platform.active_device_count());

    println!();
    println!("== license ==");
    let status = license::status();
    println!("transport code      : {} ({})", status.code, status.code.as_i32());
    println!("license state       : {:?}", status.state);
    println!("licensed            : {}", status.state.is_licensed());
    let plan = if status.plan.is_empty() { "(none)" } else { status.plan.as_str() };
    println!("plan                : {plan}");
    println!("message             : {}", status.message);

    println!();
    println!("== gamepad creation ==");
    match platform.create_dualshock4(Framing::Bluetooth) {
        Ok(gamepad) => {
            println!("created             : {gamepad:?}");
            let state = livvirtualhid::GamepadState::with_all_buttons();
            match gamepad.submit(&state) {
                Ok(()) => {
                    println!("submit all buttons  : ok");
                    println!("submit_count        : {}", gamepad.submit_count());
                    let report = gamepad.last_input_report();
                    let hex = report.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ");
                    println!("last_input_report   : {} ({hex})", report.len());
                }
                Err(error) => {
                    println!("submit failed       : {} ({}) - {}", error.code, error.code.as_i32(), error.message);
                }
            }
        }
        Err(error) => {
            println!("create failed       : {} ({}) - {}", error.code, error.code.as_i32(), error.message);
        }
    }

    println!();
    println!("== in-memory backend sanity check ==");
    let memory = Runtime::new().expect("memory runtime");
    let gamepad = memory.create_dualshock4(Framing::Bluetooth).expect("memory gamepad");
    gamepad.submit(&livvirtualhid::GamepadState::with_all_buttons()).expect("submit");
    println!("memory report length: {}", gamepad.last_input_report().len());
}
