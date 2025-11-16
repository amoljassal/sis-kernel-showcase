//! PS/2 Keyboard Driver
//!
//! This module implements a driver for PS/2 keyboards using the 8042 controller.
//! The 8042 is a legacy keyboard controller found on most x86/x86_64 systems,
//! including the MacBook Pro Mid 2012.
//!
//! Key features:
//! - Scancode Set 1 support (default on most BIOSes)
//! - IRQ 1 interrupt-driven input
//! - Circular buffer for keystrokes
//! - ASCII translation for printable characters
//!
//! Reference: Intel 8042 Keyboard Controller Specification

use core::ptr;
use spin::Mutex;

/// 8042 Keyboard Controller Ports
const PS2_DATA_PORT: u16 = 0x60;       // Read: Input buffer, Write: Output buffer
const PS2_STATUS_PORT: u16 = 0x64;     // Read: Status register
const PS2_COMMAND_PORT: u16 = 0x64;    // Write: Command register

/// Status Register Bits
const STATUS_OUTPUT_FULL: u8 = 0x01;   // Output buffer full (data available)
const STATUS_INPUT_FULL: u8 = 0x02;    // Input buffer full (don't write yet)

/// Keyboard Commands
const CMD_READ_CONFIG: u8 = 0x20;      // Read controller configuration
const CMD_WRITE_CONFIG: u8 = 0x60;     // Write controller configuration
const CMD_DISABLE_MOUSE: u8 = 0xA7;    // Disable mouse port
const CMD_ENABLE_KEYBOARD: u8 = 0xAE;  // Enable keyboard port

/// Keyboard Configuration Bits
const CONFIG_KEYBOARD_INTERRUPT: u8 = 0x01;  // Enable keyboard interrupt (IRQ 1)
const CONFIG_MOUSE_INTERRUPT: u8 = 0x02;     // Enable mouse interrupt (IRQ 12)
const CONFIG_KEYBOARD_DISABLE: u8 = 0x10;    // Keyboard disabled
const CONFIG_MOUSE_DISABLE: u8 = 0x20;       // Mouse disabled
const CONFIG_TRANSLATE: u8 = 0x40;           // Translate to scancode set 1

/// Maximum number of keystrokes to buffer
const KEY_BUFFER_SIZE: usize = 256;

/// Keyboard state
static KEYBOARD: Mutex<KeyboardState> = Mutex::new(KeyboardState::new());

/// Keyboard state and buffer
struct KeyboardState {
    /// Circular buffer for scancodes
    buffer: [u8; KEY_BUFFER_SIZE],
    /// Write position in buffer
    write_pos: usize,
    /// Read position in buffer
    read_pos: usize,
    /// Modifier key states
    shift_pressed: bool,
    ctrl_pressed: bool,
    alt_pressed: bool,
    caps_lock: bool,
}

impl KeyboardState {
    const fn new() -> Self {
        Self {
            buffer: [0; KEY_BUFFER_SIZE],
            write_pos: 0,
            read_pos: 0,
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
            caps_lock: false,
        }
    }

    /// Add a scancode to the buffer
    fn push_scancode(&mut self, scancode: u8) {
        let next_write = (self.write_pos + 1) % KEY_BUFFER_SIZE;
        if next_write != self.read_pos {
            self.buffer[self.write_pos] = scancode;
            self.write_pos = next_write;
        }
    }

    /// Get the next scancode from the buffer
    fn pop_scancode(&mut self) -> Option<u8> {
        if self.read_pos == self.write_pos {
            None
        } else {
            let scancode = self.buffer[self.read_pos];
            self.read_pos = (self.read_pos + 1) % KEY_BUFFER_SIZE;
            Some(scancode)
        }
    }

    /// Check if buffer has data
    fn has_data(&self) -> bool {
        self.read_pos != self.write_pos
    }
}

/// Initialize PS/2 keyboard controller
pub unsafe fn init() {
    crate::arch::x86_64::serial::serial_write(b"[PS2] Initializing PS/2 keyboard controller\n");

    // Disable devices during setup
    crate::arch::x86_64::serial::serial_write(b"[PS2] Disabling devices...\n");
    outb(PS2_COMMAND_PORT, CMD_DISABLE_MOUSE);
    // Note: CMD_DISABLE_KEYBOARD (0xAD) is not sent as some controllers have issues with it

    // Flush output buffer
    crate::arch::x86_64::serial::serial_write(b"[PS2] Flushing output buffer...\n");
    inb(PS2_DATA_PORT);

    // Read current configuration
    crate::arch::x86_64::serial::serial_write(b"[PS2] Reading configuration...\n");
    outb(PS2_COMMAND_PORT, CMD_READ_CONFIG);
    wait_for_output();
    let mut config = inb(PS2_DATA_PORT);

    crate::arch::x86_64::serial::serial_write(b"[PS2] Current config: 0x");
    print_hex_u8(config);
    crate::arch::x86_64::serial::serial_write(b"\n");

    // Modify configuration:
    // - Enable keyboard interrupt (IRQ 1)
    // - Disable mouse interrupt (we don't support mouse yet)
    // - Enable scancode translation (to Set 1)
    // - Clear disable bits
    config |= CONFIG_KEYBOARD_INTERRUPT | CONFIG_TRANSLATE;
    config &= !(CONFIG_MOUSE_INTERRUPT | CONFIG_KEYBOARD_DISABLE | CONFIG_MOUSE_DISABLE);

    crate::arch::x86_64::serial::serial_write(b"[PS2] Writing new config: 0x");
    print_hex_u8(config);
    crate::arch::x86_64::serial::serial_write(b"\n");

    // Write new configuration
    outb(PS2_COMMAND_PORT, CMD_WRITE_CONFIG);
    wait_for_input();
    outb(PS2_DATA_PORT, config);

    // Enable keyboard port
    crate::arch::x86_64::serial::serial_write(b"[PS2] Enabling keyboard port...\n");
    outb(PS2_COMMAND_PORT, CMD_ENABLE_KEYBOARD);

    crate::arch::x86_64::serial::serial_write(b"[PS2] PS/2 keyboard initialized\n");
}

/// Wait for the input buffer to be empty (ready to accept command/data)
fn wait_for_input() {
    for _ in 0..10000 {
        let status = unsafe { inb(PS2_STATUS_PORT) };
        if status & STATUS_INPUT_FULL == 0 {
            return;
        }
    }
}

/// Wait for the output buffer to be full (data available to read)
fn wait_for_output() {
    for _ in 0..10000 {
        let status = unsafe { inb(PS2_STATUS_PORT) };
        if status & STATUS_OUTPUT_FULL != 0 {
            return;
        }
    }
}

/// Keyboard interrupt handler (IRQ 1)
///
/// # Safety
/// Must only be called from IRQ 1 interrupt context
pub unsafe extern "C" fn keyboard_irq_handler() {
    // Read the scancode from the keyboard controller
    let scancode = inb(PS2_DATA_PORT);

    // Store in buffer for processing
    KEYBOARD.lock().push_scancode(scancode);

    // Acknowledge interrupt (IRQ 1 = vector 33)
    crate::arch::x86_64::pic::end_of_interrupt(33);
}

/// Read a character from the keyboard (non-blocking)
///
/// Returns Some(char) if a key was pressed, None if buffer is empty
pub fn read_char() -> Option<char> {
    let mut kb = KEYBOARD.lock();

    while let Some(scancode) = kb.pop_scancode() {
        // Handle special keys (modifiers, etc.)
        match scancode {
            0x2A | 0x36 => { // Left Shift, Right Shift (make)
                kb.shift_pressed = true;
                continue;
            }
            0xAA | 0xB6 => { // Left Shift, Right Shift (break)
                kb.shift_pressed = false;
                continue;
            }
            0x1D => { // Left Ctrl (make)
                kb.ctrl_pressed = true;
                continue;
            }
            0x9D => { // Left Ctrl (break)
                kb.ctrl_pressed = false;
                continue;
            }
            0x38 => { // Left Alt (make)
                kb.alt_pressed = true;
                continue;
            }
            0xB8 => { // Left Alt (break)
                kb.alt_pressed = false;
                continue;
            }
            0x3A => { // Caps Lock
                kb.caps_lock = !kb.caps_lock;
                continue;
            }
            _ => {}
        }

        // Only process key-down events (scancode < 0x80)
        if scancode >= 0x80 {
            continue;
        }

        // Translate scancode to character
        if let Some(ch) = scancode_to_char(scancode, kb.shift_pressed || kb.caps_lock) {
            return Some(ch);
        }
    }

    None
}

/// Check if keyboard has data available
pub fn has_data() -> bool {
    KEYBOARD.lock().has_data()
}

/// Translate Scancode Set 1 to ASCII character (US keyboard layout)
fn scancode_to_char(scancode: u8, shifted: bool) -> Option<char> {
    // Scancode Set 1 translation table (US layout)
    const SCANCODE_TABLE: &[(u8, char, char)] = &[
        // (scancode, normal, shifted)
        (0x02, '1', '!'),
        (0x03, '2', '@'),
        (0x04, '3', '#'),
        (0x05, '4', '$'),
        (0x06, '5', '%'),
        (0x07, '6', '^'),
        (0x08, '7', '&'),
        (0x09, '8', '*'),
        (0x0A, '9', '('),
        (0x0B, '0', ')'),
        (0x0C, '-', '_'),
        (0x0D, '=', '+'),
        (0x0E, '\x08', '\x08'), // Backspace
        (0x0F, '\t', '\t'),     // Tab
        (0x10, 'q', 'Q'),
        (0x11, 'w', 'W'),
        (0x12, 'e', 'E'),
        (0x13, 'r', 'R'),
        (0x14, 't', 'T'),
        (0x15, 'y', 'Y'),
        (0x16, 'u', 'U'),
        (0x17, 'i', 'I'),
        (0x18, 'o', 'O'),
        (0x19, 'p', 'P'),
        (0x1A, '[', '{'),
        (0x1B, ']', '}'),
        (0x1C, '\n', '\n'),     // Enter
        (0x1E, 'a', 'A'),
        (0x1F, 's', 'S'),
        (0x20, 'd', 'D'),
        (0x21, 'f', 'F'),
        (0x22, 'g', 'G'),
        (0x23, 'h', 'H'),
        (0x24, 'j', 'J'),
        (0x25, 'k', 'K'),
        (0x26, 'l', 'L'),
        (0x27, ';', ':'),
        (0x28, '\'', '"'),
        (0x29, '`', '~'),
        (0x2B, '\\', '|'),
        (0x2C, 'z', 'Z'),
        (0x2D, 'x', 'X'),
        (0x2E, 'c', 'C'),
        (0x2F, 'v', 'V'),
        (0x30, 'b', 'B'),
        (0x31, 'n', 'N'),
        (0x32, 'm', 'M'),
        (0x33, ',', '<'),
        (0x34, '.', '>'),
        (0x35, '/', '?'),
        (0x39, ' ', ' '),       // Space
    ];

    for &(sc, normal, shift) in SCANCODE_TABLE {
        if sc == scancode {
            return Some(if shifted { shift } else { normal });
        }
    }

    None
}

/// Read a byte from an I/O port
#[inline]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        in("dx") port,
        out("al") value,
        options(nomem, nostack, preserves_flags)
    );
    value
}

/// Write a byte to an I/O port
#[inline]
unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

/// Helper function to print u8 as hexadecimal
fn print_hex_u8(n: u8) {
    let hex_chars = b"0123456789abcdef";
    let mut buf = [0u8; 2];
    buf[0] = hex_chars[(n >> 4) as usize];
    buf[1] = hex_chars[(n & 0xF) as usize];
    crate::arch::x86_64::serial::serial_write(&buf);
}
