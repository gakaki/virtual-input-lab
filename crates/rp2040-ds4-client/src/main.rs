//! Send DualShock 4 buttons to an RP2040 board over its serial channel.
//!
//! The board (firmware/rp2040_ds4) appears on the Mac as a DualShock 4 plus a
//! serial port. This client packs a DualShock 4 USB report with the same
//! libvirtualhid code the virtual backend uses, wraps it in the board's frame
//! format, and writes it to the serial port.
//!
//! ```sh
//! cargo run -p rp2040-ds4-client                  # auto-detect the port
//! cargo run -p rp2040-ds4-client -- --port /dev/tty.usbmodem1101
//! cargo run -p rp2040-ds4-client -- --dry-run -o frame.bin
//! ```

use std::io::Write;
use std::time::Duration;

use livvirtualhid::{pack_dualshock4_report, Framing, GamepadState};

const SYNC0: u8 = 0xA5;
const SYNC1: u8 = 0x5A;
/// DualShock 4 USB report (64 bytes) without the leading report id byte.
const PAYLOAD_LEN: usize = 63;

/// Wrap a payload in the board's frame format.
fn build_frame(payload: &[u8]) -> Vec<u8> {
    assert_eq!(payload.len(), PAYLOAD_LEN, "payload must be {PAYLOAD_LEN} bytes");
    let mut checksum = PAYLOAD_LEN as u8;
    for byte in payload {
        checksum ^= byte;
    }
    let mut frame = Vec::with_capacity(PAYLOAD_LEN + 4);
    frame.extend_from_slice(&[SYNC0, SYNC1, PAYLOAD_LEN as u8]);
    frame.extend_from_slice(payload);
    frame.push(checksum);
    frame
}

/// Pack a DualShock 4 USB report and drop the report id byte.
fn payload_for(state: &GamepadState) -> Vec<u8> {
    let report = pack_dualshock4_report(Framing::Usb, state);
    assert_eq!(report.len(), PAYLOAD_LEN + 1, "unexpected report size");
    report[1..].to_vec()
}

fn find_port() -> Option<String> {
    let mut candidates: Vec<_> = serialport::available_ports()
        .ok()?
        .into_iter()
        .filter(|port| {
            let name = port.port_name.to_lowercase();
            name.contains("usbmodem") || name.contains("usbserial")
        })
        .map(|port| port.port_name)
        .collect();
    candidates.sort();
    candidates.into_iter().next()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ")
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let port_arg = args
        .iter()
        .position(|a| a == "--port")
        .and_then(|index| args.get(index + 1).cloned());
    let output_arg = args
        .iter()
        .position(|a| a == "-o" || a == "--output")
        .and_then(|index| args.get(index + 1).cloned())
        .unwrap_or_else(|| "rp2040_frames.bin".to_owned());

    let all_buttons = payload_for(&GamepadState::with_all_buttons());
    let neutral = payload_for(&GamepadState::neutral());
    println!("all-buttons payload ({}B): {}", all_buttons.len(), hex(&all_buttons));

    if dry_run {
        let mut data = Vec::new();
        for _ in 0..3 {
            data.extend_from_slice(&build_frame(&all_buttons));
        }
        data.extend_from_slice(&build_frame(&neutral));
        std::fs::write(&output_arg, &data)?;
        println!("wrote {} bytes to {output_arg}", data.len());
        return Ok(());
    }

    let port = match port_arg.or_else(find_port) {
        Some(port) => port,
        None => {
            eprintln!("no serial port found; is the RP2040 running the DS4 firmware?");
            std::process::exit(2);
        }
    };

    println!("opening {port}");
    let mut link = serialport::new(&port, 115_200)
        .timeout(Duration::from_secs(1))
        .open()?;

    std::thread::sleep(Duration::from_millis(200));
    let frame = build_frame(&all_buttons);
    for index in 1..=3 {
        link.write_all(&frame)?;
        link.flush()?;
        println!("  sent all-buttons frame {index}");
        std::thread::sleep(Duration::from_millis(100));
    }
    std::thread::sleep(Duration::from_millis(500));
    link.write_all(&build_frame(&neutral))?;
    link.flush()?;
    println!("  sent neutral frame");

    println!("done");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
