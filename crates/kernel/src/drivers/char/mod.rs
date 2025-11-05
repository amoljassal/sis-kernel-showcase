/// Character device drivers

pub mod console;
pub mod pty;

pub use console::{CONSOLE_OPS, NULL_OPS, ZERO_OPS, RANDOM_OPS};
pub use pty::{create_pty_pair, PtyMaster, PtySlave, Termios, TCGETS, TCSETS, TIOCGPTN};
