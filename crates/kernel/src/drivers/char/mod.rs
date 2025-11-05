/// Character device drivers

pub mod console;

pub use console::{CONSOLE_OPS, NULL_OPS, ZERO_OPS, RANDOM_OPS};
