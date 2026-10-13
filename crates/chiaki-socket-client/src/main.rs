//! Send controller input to a patched chiaki-ng over its local socket.
//!
//! Requires the socket-input patch (patches/chiaki-ng-socket-input.patch)
//! applied and chiaki-ng running a stream. Works while chiaki-ng is minimised:
//! the input never touches the window system.
//!
//! ```sh
//! cargo run -p chiaki-socket-client
//! cargo run -p chiaki-socket-client -- --socket /tmp/chiaki-ng-input.sock
//! cargo run -p chiaki-socket-client -- --dry-run -o msg.bin
//! ```

use std::os::unix::net::UnixDatagram;
use std::time::{Duration, Instant};

const MAGIC: u32 = 0x4B50_4943; // "CIPK"
const VERSION: u16 = 1;
const MESSAGE_SIZE: u16 = 24;
const DEFAULT_SOCKET: &str = "/tmp/chiaki-ng-input.sock";
const STICK_MAX: i16 = 0x7FFF;

/// ChiakiControllerButton bit values.
#[allow(dead_code)]
mod button {
    pub const CROSS: u32 = 1 << 0;
    pub const MOON: u32 = 1 << 1; // circle
    pub const BOX: u32 = 1 << 2; // square
    pub const PYRAMID: u32 = 1 << 3; // triangle
    pub const DPAD_LEFT: u32 = 1 << 4;
    pub const DPAD_RIGHT: u32 = 1 << 5;
    pub const DPAD_UP: u32 = 1 << 6;
    pub const DPAD_DOWN: u32 = 1 << 7;
    pub const L1: u32 = 1 << 8;
    pub const R1: u32 = 1 << 9;
    pub const L3: u32 = 1 << 10;
    pub const R3: u32 = 1 << 11;
    pub const OPTIONS: u32 = 1 << 12;
    pub const SHARE: u32 = 1 << 13;
    pub const TOUCHPAD: u32 = 1 << 14;
    pub const PS: u32 = 1 << 15;
    pub const ALL: u32 = 0xFFFF;
}

/// One controller state message.
#[derive(Debug, Clone, Copy, Default)]
pub struct ChiakiInputState {
    pub buttons: u32,
    pub l2: u8,
    pub r2: u8,
    pub left: (i16, i16),
    pub right: (i16, i16),
}

impl ChiakiInputState {
    /// Every button pressed and both triggers at full travel.
    pub fn all_buttons() -> Self {
        Self {
            buttons: button::ALL,
            l2: 255,
            r2: 255,
            ..Self::default()
        }
    }

    /// Serialize to the 24-byte little-endian wire format.
    pub fn to_bytes(self) -> [u8; MESSAGE_SIZE as usize] {
        let mut buffer = [0u8; MESSAGE_SIZE as usize];
        let clamp = |value: i16| value.clamp(-STICK_MAX, STICK_MAX);
        buffer[0..4].copy_from_slice(&MAGIC.to_le_bytes());
        buffer[4..6].copy_from_slice(&VERSION.to_le_bytes());
        buffer[6..8].copy_from_slice(&MESSAGE_SIZE.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.buttons.to_le_bytes());
        buffer[12] = self.l2;
        buffer[13] = self.r2;
        buffer[14..16].copy_from_slice(&clamp(self.left.0).to_le_bytes());
        buffer[16..18].copy_from_slice(&clamp(self.left.1).to_le_bytes());
        buffer[18..20].copy_from_slice(&clamp(self.right.0).to_le_bytes());
        buffer[20..22].copy_from_slice(&clamp(self.right.1).to_le_bytes());
        buffer[22..24].copy_from_slice(&0u16.to_le_bytes());
        buffer
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ")
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let socket_path = args
        .iter()
        .position(|a| a == "--socket")
        .and_then(|index| args.get(index + 1).cloned())
        .unwrap_or_else(|| DEFAULT_SOCKET.to_owned());
    let output = args
        .iter()
        .position(|a| a == "-o" || a == "--output")
        .and_then(|index| args.get(index + 1).cloned())
        .unwrap_or_else(|| "chiaki_messages.bin".to_owned());

    let pressed = ChiakiInputState::all_buttons().to_bytes();
    let neutral = ChiakiInputState::default().to_bytes();
    println!("buttons mask 0x{:04x}, message {}B: {}", button::ALL, pressed.len(), hex(&pressed));

    if dry_run {
        let mut data = Vec::new();
        for _ in 0..3 {
            data.extend_from_slice(&pressed);
        }
        data.extend_from_slice(&neutral);
        std::fs::write(&output, &data)?;
        println!("wrote {} bytes to {output}", data.len());
        return Ok(());
    }

    let socket = UnixDatagram::unbound()?;
    socket.connect(&socket_path).map_err(|error| {
        format!("could not connect to {socket_path}: {error} (is chiaki-ng running the patched build with an active stream?)")
    })?;

    println!("sending to {socket_path}");
    let deadline = Instant::now() + Duration::from_millis(500);
    while Instant::now() < deadline {
        socket.send(&pressed)?;
        std::thread::sleep(Duration::from_millis(20));
    }
    socket.send(&neutral)?;
    println!("  held all buttons, released");
    println!("done");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
