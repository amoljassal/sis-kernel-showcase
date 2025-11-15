// SIS Kernel Property Test Data Generators
// Custom generators for kernel data structures and operation sequences

use proptest::prelude::*;
use proptest::prop_oneof;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationSequence {
    pub operations: Vec<AllocOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocOp {
    Alloc(usize),
    Dealloc(u32), // pointer ID
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAccessSequence {
    pub operations: Vec<MemoryOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryOp {
    Alloc(usize),
    Dealloc(u32),
    Access(u32), // pointer ID
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationPattern {
    pub allocation_sizes: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSchedulingSequence {
    pub events: Vec<SchedulerEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchedulerEvent {
    AddProcess(u32, u8), // PID, priority
    Schedule,
    RemoveProcess(u32), // PID
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritySchedulingSequence {
    pub events: Vec<PriorityEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriorityEvent {
    ScheduleWithPriority(u32, u8), // PID, priority
    CheckInversion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPCMessageSequence {
    pub operations: Vec<IPCOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IPCOp {
    Send(u32), // message ID
    Receive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStressTest {
    pub max_capacity: usize,
    pub operations: Vec<ChannelOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelOp {
    Send(u32),
    Receive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrentOperationSequence {
    pub operations: Vec<ConcurrentOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConcurrentOp {
    Enqueue(u32),
    Dequeue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemOperationSequence {
    pub operations: Vec<FSOperation>,
    pub crash_point: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FSOperation {
    Create(String), // path
    Write(String, Vec<u8>), // path, data
    Delete(String), // path
}

// Proptest strategy generators

pub fn allocation_sequence() -> impl Strategy<Value = AllocationSequence> {
    prop::collection::vec(
        prop_oneof![
            (1usize..=4096).prop_map(AllocOp::Alloc),
            (1u32..=100).prop_map(AllocOp::Dealloc),
        ],
        1..=50
    ).prop_map(|operations| AllocationSequence { operations })
}

pub fn memory_access_sequence() -> impl Strategy<Value = MemoryAccessSequence> {
    prop::collection::vec(
        prop_oneof![
            (1usize..=4096).prop_map(MemoryOp::Alloc),
            (1u32..=100).prop_map(MemoryOp::Dealloc),
            (1u32..=100).prop_map(MemoryOp::Access),
        ],
        1..=100
    ).prop_map(|operations| MemoryAccessSequence { operations })
}

pub fn allocation_pattern() -> impl Strategy<Value = AllocationPattern> {
    prop::collection::vec(
        1usize..=8192,
        1..=200
    ).prop_map(|allocation_sizes| AllocationPattern { allocation_sizes })
}

pub fn process_scheduling_sequence() -> impl Strategy<Value = ProcessSchedulingSequence> {
    prop::collection::vec(
        prop_oneof![
            (1u32..=100, 1u8..=10).prop_map(|(pid, priority)| SchedulerEvent::AddProcess(pid, priority)),
            Just(SchedulerEvent::Schedule),
            (1u32..=100).prop_map(SchedulerEvent::RemoveProcess),
        ],
        1..=200
    ).prop_map(|events| ProcessSchedulingSequence { events })
}

pub fn priority_scheduling_sequence() -> impl Strategy<Value = PrioritySchedulingSequence> {
    prop::collection::vec(
        prop_oneof![
            (1u32..=50, 1u8..=10).prop_map(|(pid, priority)| PriorityEvent::ScheduleWithPriority(pid, priority)),
            Just(PriorityEvent::CheckInversion),
        ],
        1..=100
    ).prop_map(|events| PrioritySchedulingSequence { events })
}

pub fn ipc_message_sequence() -> impl Strategy<Value = IPCMessageSequence> {
    prop::collection::vec(
        prop_oneof![
            (1u32..=1000).prop_map(IPCOp::Send),
            Just(IPCOp::Receive),
        ],
        1..=100
    ).prop_map(|operations| IPCMessageSequence { operations })
}

pub fn channel_stress_test() -> impl Strategy<Value = ChannelStressTest> {
    (1usize..=100).prop_flat_map(|max_capacity| {
        prop::collection::vec(
            prop_oneof![
                (1u32..=1000).prop_map(ChannelOp::Send),
                Just(ChannelOp::Receive),
            ],
            1..=(max_capacity * 3)
        ).prop_map(move |operations| ChannelStressTest { 
            max_capacity, 
            operations 
        })
    })
}

pub fn concurrent_operation_sequence() -> impl Strategy<Value = ConcurrentOperationSequence> {
    prop::collection::vec(
        prop_oneof![
            (1u32..=1000).prop_map(ConcurrentOp::Enqueue),
            Just(ConcurrentOp::Dequeue),
        ],
        1..=100
    ).prop_map(|operations| ConcurrentOperationSequence { operations })
}

pub fn filesystem_operation_sequence() -> impl Strategy<Value = FileSystemOperationSequence> {
    prop::collection::vec(
        prop_oneof![
            "/(tmp|home|etc|var)/[a-zA-Z0-9_]{1,20}\\.(txt|log|dat|cfg)"
                .prop_map(FSOperation::Create),
            ("/(tmp|home|etc|var)/[a-zA-Z0-9_]{1,20}\\.(txt|log|dat|cfg)", 
             prop::collection::vec(0u8..255, 0..=1024))
                .prop_map(|(path, data)| FSOperation::Write(path, data)),
            "/(tmp|home|etc|var)/[a-zA-Z0-9_]{1,20}\\.(txt|log|dat|cfg)"
                .prop_map(FSOperation::Delete),
        ],
        1..=50
    ).prop_flat_map(|operations| {
        let op_count = operations.len();
        (Just(operations), 0..op_count)
            .prop_map(|(operations, crash_point)| FileSystemOperationSequence { 
                operations, 
                crash_point 
            })
    })
}

// Mock implementations for testing

pub struct MockHeapAllocator {
    allocations: std::collections::HashMap<u32, usize>,
    next_id: u32,
    total_allocated: usize,
    total_freed: usize,
}

impl Default for MockHeapAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl MockHeapAllocator {
    pub fn new() -> Self {
        Self {
            allocations: std::collections::HashMap::new(),
            next_id: 1,
            total_allocated: 0,
            total_freed: 0,
        }
    }

    pub fn allocate(&mut self, size: usize) -> Option<u32> {
        let id = self.next_id;
        self.next_id += 1;
        self.allocations.insert(id, size);
        self.total_allocated += size;
        Some(id)
    }

    pub fn deallocate(&mut self, ptr_id: u32) -> Option<usize> {
        if let Some(size) = self.allocations.remove(&ptr_id) {
            self.total_freed += size;
            Some(size)
        } else {
            None
        }
    }

    pub fn currently_allocated(&self) -> usize {
        self.total_allocated - self.total_freed
    }

    pub fn calculate_fragmentation(&self) -> f64 {
        if self.total_allocated == 0 {
            0.0
        } else {
            let used = self.currently_allocated();
            let fragmented = self.total_allocated - used;
            fragmented as f64 / self.total_allocated as f64
        }
    }
}

pub struct MockScheduler {
    processes: std::collections::HashMap<u32, u8>, // PID -> priority
    last_scheduled: Option<u32>,
    schedule_count: usize,
}

impl Default for MockScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl MockScheduler {
    pub fn new() -> Self {
        Self {
            processes: std::collections::HashMap::new(),
            last_scheduled: None,
            schedule_count: 0,
        }
    }

    pub fn add_process(&mut self, pid: u32, priority: u8) {
        self.processes.insert(pid, priority);
    }

    pub fn remove_process(&mut self, pid: u32) {
        self.processes.remove(&pid);
    }

    pub fn schedule_next(&mut self) -> Option<u32> {
        if self.processes.is_empty() {
            return None;
        }

        // Simple round-robin with priority bias
        let highest_priority = self.processes.values().max().copied()?;
        let candidates: Vec<_> = self.processes
            .iter()
            .filter(|(_, &priority)| priority == highest_priority)
            .map(|(&pid, _)| pid)
            .collect();

        if candidates.is_empty() {
            return None;
        }

        let selected = candidates[self.schedule_count % candidates.len()];
        self.schedule_count += 1;
        self.last_scheduled = Some(selected);
        Some(selected)
    }

    pub fn has_priority_inversion(&self) -> bool {
        false
    }
}

pub struct MockIPCChannel {
    queue: std::collections::VecDeque<u32>,
    capacity: Option<usize>,
}

impl Default for MockIPCChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl MockIPCChannel {
    pub fn new() -> Self {
        Self {
            queue: std::collections::VecDeque::new(),
            capacity: None,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            queue: std::collections::VecDeque::new(),
            capacity: Some(capacity),
        }
    }

    pub fn send(&mut self, msg_id: u32) -> bool {
        if let Some(cap) = self.capacity {
            if self.queue.len() >= cap {
                return false;
            }
        }
        self.queue.push_back(msg_id);
        true
    }

    pub fn receive(&mut self) -> Option<u32> {
        self.queue.pop_front()
    }

    pub fn try_send(&mut self, msg_id: u32) -> bool {
        self.send(msg_id)
    }

    pub fn try_receive(&mut self) -> Option<u32> {
        self.receive()
    }

    pub fn current_size(&self) -> usize {
        self.queue.len()
    }
}

pub struct MockLockFreeQueue {
    queue: std::sync::Mutex<std::collections::VecDeque<u32>>,
}

impl Default for MockLockFreeQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl MockLockFreeQueue {
    pub fn new() -> Self {
        Self {
            queue: std::sync::Mutex::new(std::collections::VecDeque::new()),
        }
    }

    pub fn enqueue(&self, value: u32) {
        if let Ok(mut queue) = self.queue.lock() {
            queue.push_back(value);
        }
    }

    pub fn dequeue(&self) -> Option<u32> {
        if let Ok(mut queue) = self.queue.lock() {
            queue.pop_front()
        } else {
            None
        }
    }
}

pub struct MockFileSystem {
    files: std::collections::HashMap<String, Vec<u8>>,
    crashed: bool,
}

impl Default for MockFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            files: std::collections::HashMap::new(),
            crashed: false,
        }
    }

    pub fn create_file(&mut self, path: &str) {
        if !self.crashed {
            self.files.insert(path.to_string(), Vec::new());
        }
    }

    pub fn write_file(&mut self, path: &str, data: &[u8]) {
        if !self.crashed {
            self.files.insert(path.to_string(), data.to_vec());
        }
    }

    pub fn delete_file(&mut self, path: &str) {
        if !self.crashed {
            self.files.remove(path);
        }
    }

    pub fn simulate_crash(&mut self) {
        self.crashed = true;
    }

    pub fn recover(&mut self) {
        self.crashed = false;
    }

    pub fn check_consistency(&self) -> bool {
        true
    }
}