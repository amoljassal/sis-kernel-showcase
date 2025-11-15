// Mock Device Implementations
// Phase 6 - Production Readiness Plan
//
// Mock implementations of device traits for testing without hardware

pub mod block;
pub mod network;
pub mod timer;

pub use block::MockBlockDevice;
pub use network::MockNetworkDevice;
pub use timer::MockTimerDevice;
