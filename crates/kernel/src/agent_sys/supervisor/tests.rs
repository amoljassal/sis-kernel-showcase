//! Integration tests for Agent Supervision Module
//!
//! These tests validate the end-to-end functionality of ASM including
//! lifecycle management, fault detection, telemetry, and policy control.

#![cfg(test)]

use super::*;
use crate::process::Pid;
use crate::agent_sys::AgentId;
use crate::security::agent_policy::Capability;
use alloc::string::ToString;
use alloc::vec;

/// Test helper to initialize ASM
fn setup_asm() {
    init();
    assert!(is_initialized());
}

/// Test helper to create a test agent spec
fn create_test_spec(agent_id: AgentId, name: &str) -> AgentSpec {
    AgentSpec::new(agent_id, name.to_string())
        .with_capability(Capability::FsBasic)
        .with_capability(Capability::NetClient)
        .with_auto_restart(true)
        .with_max_restarts(3)
}

#[test]
fn test_asm_initialization() {
    setup_asm();

    // Verify all components are initialized
    assert!(AGENT_SUPERVISOR.lock().is_some());
    assert!(TELEMETRY.lock().is_some());
    assert!(FAULT_DETECTOR.lock().is_some());
    assert!(POLICY_CONTROLLER.lock().is_some());
}

#[test]
fn test_agent_lifecycle_spawn_exit() {
    setup_asm();

    let spec = create_test_spec(100, "test_agent");
    let pid: Pid = 42;

    // Spawn agent
    let agent_id = hooks::on_process_spawn(pid, spec);
    assert_eq!(agent_id, 100);

    // Verify agent is tracked
    assert_eq!(hooks::is_agent_process(pid), Some(100));

    // Verify telemetry shows active agent
    let snapshot = hooks::get_telemetry_snapshot().unwrap();
    assert_eq!(snapshot.system.total_agents_spawned, 1);
    assert_eq!(snapshot.system.active_agents, 1);

    // Exit agent
    let was_agent = hooks::on_process_exit(pid, 0);
    assert!(was_agent);

    // Verify agent is no longer tracked
    assert_eq!(hooks::is_agent_process(pid), None);

    // Verify telemetry updated
    let snapshot = hooks::get_telemetry_snapshot().unwrap();
    assert_eq!(snapshot.system.total_agents_exited, 1);
    assert_eq!(snapshot.system.active_agents, 0);
}

#[test]
fn test_multiple_agents() {
    setup_asm();

    // Spawn multiple agents
    for i in 0..5 {
        let spec = create_test_spec(100 + i, &format!("agent_{}", i));
        let pid = 100 + i;
        hooks::on_process_spawn(pid, spec);
    }

    // Verify all tracked
    for i in 0..5 {
        assert_eq!(hooks::is_agent_process(100 + i), Some(100 + i));
    }

    // Verify telemetry
    let snapshot = hooks::get_telemetry_snapshot().unwrap();
    assert_eq!(snapshot.system.active_agents, 5);
    assert_eq!(snapshot.system.total_agents_spawned, 5);

    // Exit all agents
    for i in 0..5 {
        hooks::on_process_exit(100 + i, 0);
    }

    // Verify all cleaned up
    let snapshot = hooks::get_telemetry_snapshot().unwrap();
    assert_eq!(snapshot.system.active_agents, 0);
    assert_eq!(snapshot.system.total_agents_exited, 5);
}

#[test]
fn test_agent_crash_detection() {
    setup_asm();

    let spec = create_test_spec(101, "crash_test");
    let pid: Pid = 43;

    hooks::on_process_spawn(pid, spec);

    // Exit with non-zero code (crash)
    hooks::on_process_exit(pid, 139); // SIGSEGV

    // Verify telemetry shows crash
    let snapshot = hooks::get_telemetry_snapshot().unwrap();
    assert_eq!(snapshot.system.total_agents_crashed, 1);
}

#[test]
fn test_fault_detection_cpu_quota() {
    setup_asm();

    let spec = create_test_spec(102, "cpu_hog");
    hooks::on_process_spawn(44, spec);

    // Report CPU quota exceeded
    let fault = fault::Fault::CpuQuotaExceeded {
        used: 150_000,
        quota: 100_000,
    };

    let action = hooks::report_agent_fault(102, fault);

    // Default policy should throttle
    assert!(matches!(action, fault::FaultAction::Throttle));

    // Verify telemetry
    let snapshot = hooks::get_telemetry_snapshot().unwrap();
    if let Some(metrics) = snapshot.agents.get(&102) {
        assert_eq!(metrics.fault_count, 1);
    } else {
        panic!("Agent metrics not found");
    }
}

#[test]
fn test_fault_detection_memory_limit() {
    setup_asm();

    let spec = create_test_spec(103, "memory_hog");
    hooks::on_process_spawn(45, spec);

    // Report memory exceeded
    let fault = fault::Fault::MemoryExceeded {
        used: 1024 * 1024 * 100,  // 100 MB
        limit: 1024 * 1024 * 50,   // 50 MB limit
    };

    let action = hooks::report_agent_fault(103, fault);

    // Default policy should kill
    assert!(matches!(action, fault::FaultAction::Kill));
}

#[test]
fn test_auto_restart_on_crash() {
    setup_asm();

    let spec = create_test_spec(104, "restart_test")
        .with_auto_restart(true)
        .with_max_restarts(3);

    let pid: Pid = 46;
    hooks::on_process_spawn(pid, spec);

    // Simulate crash
    let fault = fault::Fault::Crashed { signal: 11 }; // SIGSEGV
    let action = hooks::report_agent_fault(104, fault);

    // Should restart
    assert!(matches!(action, fault::FaultAction::Restart));
}

#[test]
fn test_max_restart_limit() {
    setup_asm();

    let spec = create_test_spec(105, "flaky_agent")
        .with_auto_restart(true)
        .with_max_restarts(2);

    hooks::on_process_spawn(47, spec);

    // Crash multiple times
    for i in 0..3 {
        let fault = fault::Fault::Crashed { signal: 11 };
        let action = hooks::report_agent_fault(105, fault);

        if i < 2 {
            // Should restart first 2 times
            assert!(matches!(action, fault::FaultAction::Restart));
        } else {
            // Should kill after max restarts
            assert!(matches!(action, fault::FaultAction::Kill));
        }
    }
}

#[test]
fn test_watchdog_timeout() {
    setup_asm();

    let spec = create_test_spec(106, "hanging_agent");
    hooks::on_process_spawn(48, spec);

    // Simulate watchdog timeout (10 minutes idle)
    let idle_time = 10 * 60 * 1_000_000; // 10 minutes in microseconds

    let mut detector = FAULT_DETECTOR.lock();
    if let Some(ref detector) = *detector {
        let fault = detector.check_watchdog(106, idle_time);
        assert!(fault.is_some());

        if let Some(fault::Fault::WatchdogTimeout { idle, threshold }) = fault {
            assert_eq!(idle, idle_time);
            assert!(threshold > 0);
        } else {
            panic!("Expected WatchdogTimeout fault");
        }
    }
}

#[test]
fn test_policy_update_capability_add() {
    setup_asm();

    let spec = create_test_spec(107, "policy_test");
    hooks::on_process_spawn(49, spec);

    let mut controller = POLICY_CONTROLLER.lock();
    if let Some(ref mut ctrl) = *controller {
        // Try to add NetServer capability
        let patch = policy_controller::PolicyPatch::AddCapability(Capability::NetServer);
        let result = ctrl.update_policy(107, patch);

        // Should succeed
        assert!(result.is_ok());

        // Verify policy updated
        if let Some(policy) = ctrl.get_policy(107) {
            assert!(policy.capabilities.contains(&Capability::NetServer));
        }
    }
}

#[test]
fn test_policy_update_privilege_escalation_denied() {
    setup_asm();

    let spec = create_test_spec(108, "escalation_test")
        .with_capability(Capability::FsBasic);

    hooks::on_process_spawn(50, spec);

    let mut controller = POLICY_CONTROLLER.lock();
    if let Some(ref mut ctrl) = *controller {
        // Try to add AdminOps capability (privilege escalation)
        let patch = policy_controller::PolicyPatch::AddCapability(Capability::AdminOps);
        let result = ctrl.update_policy(108, patch);

        // Should fail - privilege escalation
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), policy_controller::PolicyError::PrivilegeEscalation));
    }
}

#[test]
fn test_telemetry_event_buffer() {
    setup_asm();

    // Spawn and exit multiple agents to generate events
    for i in 0..10 {
        let spec = create_test_spec(200 + i, &format!("event_test_{}", i));
        hooks::on_process_spawn(200 + i, spec);
        hooks::on_process_exit(200 + i, 0);
    }

    // Get telemetry snapshot
    let snapshot = hooks::get_telemetry_snapshot().unwrap();

    // Should have events in buffer (spawn + exit for each = 20 events)
    assert_eq!(snapshot.recent_events.len(), 20);

    // Verify event order (most recent first)
    let first_event = &snapshot.recent_events[0];
    assert!(matches!(first_event.event, lifecycle::LifecycleEvent::Exit { .. }));
}

#[test]
fn test_concurrent_agent_operations() {
    setup_asm();

    // Simulate concurrent spawns
    let agents: Vec<_> = (0..10)
        .map(|i| {
            let spec = create_test_spec(300 + i, &format!("concurrent_{}", i));
            let pid = 300 + i;
            (pid, spec)
        })
        .collect();

    // Spawn all
    for (pid, spec) in agents {
        hooks::on_process_spawn(pid, spec);
    }

    // Touch agents (simulate activity)
    for i in 0..10 {
        hooks::touch_agent(300 + i);
    }

    // Exit all
    for i in 0..10 {
        hooks::on_process_exit(300 + i, 0);
    }

    // Verify cleanup
    let snapshot = hooks::get_telemetry_snapshot().unwrap();
    assert_eq!(snapshot.system.active_agents, 0);
    assert_eq!(snapshot.system.total_agents_spawned, 10);
    assert_eq!(snapshot.system.total_agents_exited, 10);
}

#[test]
fn test_fault_recovery_policy_kill() {
    setup_asm();

    let mut detector = FAULT_DETECTOR.lock();
    if let Some(ref detector) = *detector {
        let policy = detector.get_recovery_policy(&fault::Fault::MemoryExceeded {
            used: 100,
            limit: 50,
        });

        assert!(matches!(policy, fault::RecoveryPolicy::Kill));
    }
}

#[test]
fn test_fault_recovery_policy_throttle() {
    setup_asm();

    let mut detector = FAULT_DETECTOR.lock();
    if let Some(ref detector) = *detector {
        let policy = detector.get_recovery_policy(&fault::Fault::CpuQuotaExceeded {
            used: 150,
            quota: 100,
        });

        assert!(matches!(policy, fault::RecoveryPolicy::Throttle));
    }
}

#[test]
fn test_agent_metadata_persistence() {
    setup_asm();

    let spec = create_test_spec(400, "metadata_test")
        .with_capability(Capability::FsBasic)
        .with_capability(Capability::NetClient)
        .with_auto_restart(true)
        .with_max_restarts(5);

    hooks::on_process_spawn(400, spec.clone());

    // Retrieve and verify metadata
    let supervisor = AGENT_SUPERVISOR.lock();
    if let Some(ref sup) = *supervisor {
        if let Some(metadata) = sup.get_agent_by_pid(400) {
            assert_eq!(metadata.agent_id, 400);
            assert_eq!(metadata.pid, 400);
            assert_eq!(metadata.name, "metadata_test");
            assert_eq!(metadata.capabilities.len(), 2);
            assert!(metadata.auto_restart);
            assert_eq!(metadata.max_restarts, 5);
            assert_eq!(metadata.restart_count, 0);
        } else {
            panic!("Metadata not found");
        }
    }
}

#[test]
fn test_telemetry_metrics_accuracy() {
    setup_asm();

    // Spawn 3 agents
    for i in 0..3 {
        let spec = create_test_spec(500 + i, &format!("metrics_test_{}", i));
        hooks::on_process_spawn(500 + i, spec);
    }

    // 1 crashes
    hooks::on_process_exit(500, 11); // SIGSEGV

    // 1 exits normally
    hooks::on_process_exit(501, 0);

    // 1 still running (502)

    // Report fault for running agent
    let fault = fault::Fault::CpuQuotaExceeded { used: 100, quota: 50 };
    hooks::report_agent_fault(502, fault);

    // Verify metrics
    let snapshot = hooks::get_telemetry_snapshot().unwrap();
    assert_eq!(snapshot.system.total_agents_spawned, 3);
    assert_eq!(snapshot.system.total_agents_exited, 2);
    assert_eq!(snapshot.system.total_agents_crashed, 1);
    assert_eq!(snapshot.system.active_agents, 1);
    assert_eq!(snapshot.system.total_faults, 1);
}

#[test]
fn test_stress_many_agents() {
    setup_asm();

    const NUM_AGENTS: u32 = 100;

    // Spawn many agents
    for i in 0..NUM_AGENTS {
        let spec = create_test_spec(1000 + i, &format!("stress_{}", i));
        hooks::on_process_spawn(1000 + i, spec);
    }

    let snapshot = hooks::get_telemetry_snapshot().unwrap();
    assert_eq!(snapshot.system.active_agents, NUM_AGENTS as usize);

    // Exit all
    for i in 0..NUM_AGENTS {
        hooks::on_process_exit(1000 + i, 0);
    }

    let snapshot = hooks::get_telemetry_snapshot().unwrap();
    assert_eq!(snapshot.system.active_agents, 0);
    assert_eq!(snapshot.system.total_agents_exited, NUM_AGENTS as usize);
}

#[test]
fn test_event_buffer_overflow() {
    setup_asm();

    // Generate more than 1024 events (buffer size)
    for i in 0..600 {
        let spec = create_test_spec(2000 + i, &format!("overflow_{}", i));
        hooks::on_process_spawn(2000 + i, spec);
        hooks::on_process_exit(2000 + i, 0);
    }

    // Buffer should contain only most recent 1024 events
    let snapshot = hooks::get_telemetry_snapshot().unwrap();
    assert_eq!(snapshot.recent_events.len(), 1024);

    // Most recent event should be the last exit
    let first_event = &snapshot.recent_events[0];
    assert!(matches!(first_event.event, lifecycle::LifecycleEvent::Exit { .. }));
}

#[test]
fn test_periodic_health_check() {
    setup_asm();

    // Spawn agents
    for i in 0..5 {
        let spec = create_test_spec(3000 + i, &format!("health_{}", i));
        hooks::on_process_spawn(3000 + i, spec);
    }

    // Run health check (should find no issues with fresh agents)
    let fault_count = hooks::periodic_health_check();
    assert_eq!(fault_count, 0);
}

#[test]
fn test_touch_agent_updates_activity() {
    setup_asm();

    let spec = create_test_spec(4000, "activity_test");
    hooks::on_process_spawn(4000, spec);

    // Get initial timestamp
    let supervisor = AGENT_SUPERVISOR.lock();
    let initial_ts = if let Some(ref sup) = *supervisor {
        sup.get_agent_by_pid(4000).unwrap().last_activity
    } else {
        panic!("Supervisor not initialized");
    };
    drop(supervisor);

    // Touch agent
    hooks::touch_agent(4000);

    // Verify timestamp updated
    let supervisor = AGENT_SUPERVISOR.lock();
    if let Some(ref sup) = *supervisor {
        let new_ts = sup.get_agent_by_pid(4000).unwrap().last_activity;
        assert!(new_ts >= initial_ts);
    }
}
