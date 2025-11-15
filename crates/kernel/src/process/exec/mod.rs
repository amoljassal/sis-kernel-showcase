/// Process execution and ELF loading
///
/// Phase A1 implementation of execve and ELF64 loader

pub mod elf;

pub use elf::{load_elf, ElfError};
