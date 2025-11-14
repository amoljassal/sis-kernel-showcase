//! AgentSys operation handlers
//!
//! Each handler module implements a capability domain (FS, Audio, Docs, IO).
//! All handlers follow the pattern:
//! 1. Parse payload
//! 2. Check capability
//! 3. Validate scope
//! 4. Execute operation
//! 5. Return result

pub mod fs;
pub mod audio;
pub mod docs;
pub mod io;
