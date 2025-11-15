//! Agent Message Bus for Cross-Subsystem Coordination
//!
//! This module implements a message-passing layer enabling neural agents to share
//! information and coordinate decisions across subsystems.
//!
//! Features:
//! - Lock-protected ring buffer (32 messages capacity)
//! - Publisher/subscriber pattern for agent coordination
//! - Timestamped messages with confidence scores
//! - Support for memory, scheduling, and command agent coordination

use spin::Mutex;

/// Maximum number of messages in the bus ring buffer
const MAX_MESSAGES: usize = 32;

/// Agent message types for cross-subsystem coordination
#[derive(Copy, Clone, Debug)]
pub enum AgentMessage {
    // Memory agent messages
    MemoryPressure {
        level: u8,           // 0-100 pressure level
        fragmentation: u8,   // 0-100 fragmentation %
        confidence: u16,     // 0-1000 milli-units
        timestamp_us: u64,
    },
    MemoryCompactionNeeded {
        urgency: u8,         // 0=low, 100=critical
        confidence: u16,
        timestamp_us: u64,
    },
    MemoryHealthy {
        headroom_percent: u8,
        timestamp_us: u64,
    },

    // Scheduling agent messages
    SchedulingLoadHigh {
        deadline_misses: u8,
        avg_latency_us: u32,
        confidence: u16,
        timestamp_us: u64,
    },
    SchedulingLoadLow {
        idle_percent: u8,
        timestamp_us: u64,
    },
    SchedulingCriticalOperatorLatency {
        operator_id: u32,
        latency_us: u32,
        confidence: u16,
        timestamp_us: u64,
    },

    // Command agent messages
    CommandHeavyPredicted {
        command_hash: u32,
        confidence: u16,
        timestamp_us: u64,
    },
    CommandRapidStream {
        commands_per_sec: u16,
        confidence: u16,
        timestamp_us: u64,
    },
    CommandLowAccuracy {
        recent_accuracy: u8,  // 0-100 %
        timestamp_us: u64,
    },
    CommandQuiet {
        idle_seconds: u16,
        timestamp_us: u64,
    },
}

impl AgentMessage {
    /// Get timestamp from any message variant
    pub fn timestamp_us(&self) -> u64 {
        match self {
            AgentMessage::MemoryPressure { timestamp_us, .. } => *timestamp_us,
            AgentMessage::MemoryCompactionNeeded { timestamp_us, .. } => *timestamp_us,
            AgentMessage::MemoryHealthy { timestamp_us, .. } => *timestamp_us,
            AgentMessage::SchedulingLoadHigh { timestamp_us, .. } => *timestamp_us,
            AgentMessage::SchedulingLoadLow { timestamp_us, .. } => *timestamp_us,
            AgentMessage::SchedulingCriticalOperatorLatency { timestamp_us, .. } => *timestamp_us,
            AgentMessage::CommandHeavyPredicted { timestamp_us, .. } => *timestamp_us,
            AgentMessage::CommandRapidStream { timestamp_us, .. } => *timestamp_us,
            AgentMessage::CommandLowAccuracy { timestamp_us, .. } => *timestamp_us,
            AgentMessage::CommandQuiet { timestamp_us, .. } => *timestamp_us,
        }
    }

    /// Get confidence score from messages that have it
    pub fn confidence(&self) -> Option<u16> {
        match self {
            AgentMessage::MemoryPressure { confidence, .. } => Some(*confidence),
            AgentMessage::MemoryCompactionNeeded { confidence, .. } => Some(*confidence),
            AgentMessage::SchedulingLoadHigh { confidence, .. } => Some(*confidence),
            AgentMessage::SchedulingCriticalOperatorLatency { confidence, .. } => Some(*confidence),
            AgentMessage::CommandHeavyPredicted { confidence, .. } => Some(*confidence),
            AgentMessage::CommandRapidStream { confidence, .. } => Some(*confidence),
            _ => None,
        }
    }

    /// Get message type name for logging
    pub fn type_name(&self) -> &'static str {
        match self {
            AgentMessage::MemoryPressure { .. } => "MemoryPressure",
            AgentMessage::MemoryCompactionNeeded { .. } => "MemoryCompactionNeeded",
            AgentMessage::MemoryHealthy { .. } => "MemoryHealthy",
            AgentMessage::SchedulingLoadHigh { .. } => "SchedulingLoadHigh",
            AgentMessage::SchedulingLoadLow { .. } => "SchedulingLoadLow",
            AgentMessage::SchedulingCriticalOperatorLatency { .. } => "SchedulingCriticalLatency",
            AgentMessage::CommandHeavyPredicted { .. } => "CommandHeavyPredicted",
            AgentMessage::CommandRapidStream { .. } => "CommandRapidStream",
            AgentMessage::CommandLowAccuracy { .. } => "CommandLowAccuracy",
            AgentMessage::CommandQuiet { .. } => "CommandQuiet",
        }
    }
}

/// Ring buffer for agent messages
struct MessageRingBuffer {
    messages: [Option<AgentMessage>; MAX_MESSAGES],
    write_idx: usize,
    count: usize,  // Number of valid messages in buffer
}

impl MessageRingBuffer {
    const fn new() -> Self {
        MessageRingBuffer {
            messages: [None; MAX_MESSAGES],
            write_idx: 0,
            count: 0,
        }
    }

    /// Push a message to the ring buffer (overwrites oldest if full)
    fn push(&mut self, msg: AgentMessage) {
        self.messages[self.write_idx] = Some(msg);
        self.write_idx = (self.write_idx + 1) % MAX_MESSAGES;
        if self.count < MAX_MESSAGES {
            self.count += 1;
        }
    }

    /// Get all messages in chronological order (oldest first)
    fn get_all(&self) -> heapless::Vec<AgentMessage, MAX_MESSAGES> {
        let mut result = heapless::Vec::new();

        if self.count == 0 {
            return result;
        }

        let start_idx = if self.count < MAX_MESSAGES {
            0
        } else {
            self.write_idx
        };

        for i in 0..self.count {
            let idx = (start_idx + i) % MAX_MESSAGES;
            if let Some(msg) = self.messages[idx] {
                let _ = result.push(msg);
            }
        }

        result
    }

    /// Get messages newer than a given timestamp
    fn get_since(&self, since_us: u64) -> heapless::Vec<AgentMessage, MAX_MESSAGES> {
        let mut result = heapless::Vec::new();

        for msg in self.get_all() {
            if msg.timestamp_us() > since_us {
                let _ = result.push(msg);
            }
        }

        result
    }
}

/// Statistics for message bus monitoring
#[derive(Copy, Clone)]
pub struct BusStats {
    pub total_published: u64,
    pub memory_msgs: u32,
    pub scheduling_msgs: u32,
    pub command_msgs: u32,
    pub current_count: usize,
}

/// Global agent message bus
pub struct AgentMessageBus {
    buffer: MessageRingBuffer,
    stats: BusStats,
}

impl AgentMessageBus {
    const fn new() -> Self {
        AgentMessageBus {
            buffer: MessageRingBuffer::new(),
            stats: BusStats {
                total_published: 0,
                memory_msgs: 0,
                scheduling_msgs: 0,
                command_msgs: 0,
                current_count: 0,
            },
        }
    }

    /// Publish a message to the bus
    pub fn publish(&mut self, msg: AgentMessage) {
        self.buffer.push(msg);
        self.stats.total_published += 1;
        self.stats.current_count = self.buffer.count;

        // Update type-specific counters
        match msg {
            AgentMessage::MemoryPressure { .. }
            | AgentMessage::MemoryCompactionNeeded { .. }
            | AgentMessage::MemoryHealthy { .. } => {
                self.stats.memory_msgs += 1;
            }
            AgentMessage::SchedulingLoadHigh { .. }
            | AgentMessage::SchedulingLoadLow { .. }
            | AgentMessage::SchedulingCriticalOperatorLatency { .. } => {
                self.stats.scheduling_msgs += 1;
            }
            AgentMessage::CommandHeavyPredicted { .. }
            | AgentMessage::CommandRapidStream { .. }
            | AgentMessage::CommandLowAccuracy { .. }
            | AgentMessage::CommandQuiet { .. } => {
                self.stats.command_msgs += 1;
            }
        }
    }

    /// Get all messages in chronological order
    pub fn get_all(&self) -> heapless::Vec<AgentMessage, MAX_MESSAGES> {
        self.buffer.get_all()
    }

    /// Get messages newer than a timestamp
    pub fn get_since(&self, since_us: u64) -> heapless::Vec<AgentMessage, MAX_MESSAGES> {
        self.buffer.get_since(since_us)
    }

    /// Get statistics
    pub fn stats(&self) -> BusStats {
        self.stats
    }

    /// Clear all messages (for testing)
    pub fn clear(&mut self) {
        self.buffer = MessageRingBuffer::new();
        self.stats.current_count = 0;
    }
}

/// Global message bus instance
static MESSAGE_BUS: Mutex<AgentMessageBus> = Mutex::new(AgentMessageBus::new());

/// Publish a message to the global bus
pub fn publish_message(msg: AgentMessage) {
    MESSAGE_BUS.lock().publish(msg);
}

/// Get all messages from the global bus
pub fn get_all_messages() -> heapless::Vec<AgentMessage, MAX_MESSAGES> {
    MESSAGE_BUS.lock().get_all()
}

/// Get messages since a timestamp
pub fn get_messages_since(since_us: u64) -> heapless::Vec<AgentMessage, MAX_MESSAGES> {
    MESSAGE_BUS.lock().get_since(since_us)
}

/// Get bus statistics
pub fn get_bus_stats() -> BusStats {
    MESSAGE_BUS.lock().stats()
}

/// Clear the message bus (for testing)
pub fn clear_message_bus() {
    MESSAGE_BUS.lock().clear();
}

/// Helper to get current timestamp in microseconds (using cycle counter)
pub fn get_timestamp_us() -> u64 {
    // Use cycle counter and convert to approximate microseconds
    // Assuming ~1 GHz frequency, divide by 1000 to get microseconds
    #[cfg(feature = "deterministic")]
    {
        crate::deterministic::get_cycles_microseconds()
    }
    #[cfg(not(feature = "deterministic"))]
    {
        // Fallback: use graph::now_cycles() and convert to microseconds
        // Assuming 1 GHz clock (adjust divisor for actual frequency)
        crate::graph::now_cycles() / 1000
    }
}

/// Print message details to UART for debugging
pub fn print_message(msg: &AgentMessage) {
    unsafe {
        crate::uart_print(b"[BUS] ");
        crate::uart_print(msg.type_name().as_bytes());
        crate::uart_print(b" ts=");
        print_number(msg.timestamp_us() as usize);

        if let Some(conf) = msg.confidence() {
            crate::uart_print(b" conf=");
            print_number(conf as usize);
        }

        match msg {
            AgentMessage::MemoryPressure { level, fragmentation, .. } => {
                crate::uart_print(b" level=");
                print_number(*level as usize);
                crate::uart_print(b" frag=");
                print_number(*fragmentation as usize);
            }
            AgentMessage::SchedulingLoadHigh { deadline_misses, avg_latency_us, .. } => {
                crate::uart_print(b" misses=");
                print_number(*deadline_misses as usize);
                crate::uart_print(b" latency=");
                print_number(*avg_latency_us as usize);
                crate::uart_print(b"us");
            }
            AgentMessage::CommandRapidStream { commands_per_sec, .. } => {
                crate::uart_print(b" rate=");
                print_number(*commands_per_sec as usize);
                crate::uart_print(b"/sec");
            }
            _ => {}
        }

        crate::uart_print(b"\n");
    }
}

/// Print bus statistics
pub fn print_bus_stats() {
    let stats = get_bus_stats();
    unsafe {
        crate::uart_print(b"[BUS] Stats: total=");
        print_number(stats.total_published as usize);
        crate::uart_print(b" memory=");
        print_number(stats.memory_msgs as usize);
        crate::uart_print(b" sched=");
        print_number(stats.scheduling_msgs as usize);
        crate::uart_print(b" cmd=");
        print_number(stats.command_msgs as usize);
        crate::uart_print(b" current=");
        print_number(stats.current_count);
        crate::uart_print(b"/");
        print_number(MAX_MESSAGES);
        crate::uart_print(b"\n");
    }
}

/// Helper function to print numbers
unsafe fn print_number(mut num: usize) {
    if num == 0 {
        crate::uart_print(b"0");
        return;
    }

    let mut digits = [0u8; 20];
    let mut i = 0;

    while num > 0 {
        digits[i] = b'0' + (num % 10) as u8;
        num /= 10;
        i += 1;
    }

    while i > 0 {
        i -= 1;
        crate::uart_print(&[digits[i]]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_ring_buffer() {
        let mut buffer = MessageRingBuffer::new();

        // Test empty buffer
        assert_eq!(buffer.count, 0);
        assert_eq!(buffer.get_all().len(), 0);

        // Push a few messages
        buffer.push(AgentMessage::MemoryHealthy {
            headroom_percent: 50,
            timestamp_us: 1000,
        });
        assert_eq!(buffer.count, 1);

        buffer.push(AgentMessage::CommandQuiet {
            idle_seconds: 10,
            timestamp_us: 2000,
        });
        assert_eq!(buffer.count, 2);

        let msgs = buffer.get_all();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].timestamp_us(), 1000);
        assert_eq!(msgs[1].timestamp_us(), 2000);
    }

    #[test]
    fn test_message_overflow() {
        let mut buffer = MessageRingBuffer::new();

        // Fill buffer beyond capacity
        for i in 0..40 {
            buffer.push(AgentMessage::MemoryHealthy {
                headroom_percent: i as u8,
                timestamp_us: i,
            });
        }

        // Should only keep last 32 messages
        assert_eq!(buffer.count, MAX_MESSAGES);
        let msgs = buffer.get_all();
        assert_eq!(msgs.len(), MAX_MESSAGES);

        // First message should be #8 (0-7 were overwritten)
        assert_eq!(msgs[0].timestamp_us(), 8);
    }

    #[test]
    fn test_get_since() {
        let mut buffer = MessageRingBuffer::new();

        for i in 0..10 {
            buffer.push(AgentMessage::MemoryHealthy {
                headroom_percent: i as u8,
                timestamp_us: i * 1000,
            });
        }

        let recent = buffer.get_since(5000);
        assert_eq!(recent.len(), 4); // Messages at 6000, 7000, 8000, 9000
    }
}
