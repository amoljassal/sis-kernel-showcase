//! Integration Tests for Agent Supervision Module
//!
//! These tests validate the complete ASM lifecycle with all components
//! working together: supervisor, telemetry, faults, policies, compliance,
//! resource monitoring, dependencies, and profiling.

#![cfg(test)]

use super::*;
use crate::agent_sys::AgentId;
use crate::security::agent_policy::Capability;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Test complete agent lifecycle with all subsystems
#[test]
fn test_complete_lifecycle_integration() {
    // Initialize all subsystems
    init();

    // Create agent spec
    let spec = AgentSpec::new(1000, "test_agent".to_string())
        .with_capability(Capability::FsBasic)
        .with_capability(Capability::NetConnect);

    // Spawn agent - should register in all subsystems
    let agent_id = hooks::on_process_spawn(42, spec);
    assert_eq!(agent_id, 1000);

    // Verify registration in supervisor
    {
        let supervisor = AGENT_SUPERVISOR.lock();
        assert!(supervisor.as_ref().unwrap().get_agent(agent_id).is_some());
    }

    // Verify registration in telemetry
    {
        let telemetry = TELEMETRY.lock();
        let snapshot = telemetry.as_ref().unwrap().snapshot();
        assert_eq!(snapshot.system.active_agents, 1);
    }

    // Verify registration in compliance tracker
    {
        let compliance = COMPLIANCE_TRACKER.lock();
        let report = compliance.as_ref().unwrap().generate_report();
        assert_eq!(report.total_agents, 1);
        assert!(report.agent_records.iter().any(|r| r.agent_id == agent_id));
    }

    // Verify registration in resource monitor
    {
        let resource_mon = RESOURCE_MONITOR.lock();
        assert!(resource_mon.as_ref().unwrap().all_agents().any(|(id, _)| *id == agent_id));
    }

    // Verify registration in dependency graph
    {
        let dep_graph = DEPENDENCY_GRAPH.lock();
        // Agent should be in graph even with no dependencies
        assert!(dep_graph.as_ref().unwrap().get_dependencies(agent_id).is_some());
    }

    // Verify registration in profiler
    {
        let profiler = SYSTEM_PROFILER.lock();
        assert!(profiler.as_ref().unwrap().all_agents().any(|(id, _)| *id == agent_id));
    }

    // Report a fault
    let fault = fault::Fault::CpuQuotaExceeded {
        used: 2_000_000,
        quota: 1_000_000,
    };
    let action = hooks::report_agent_fault(agent_id, fault);

    // Should throttle per default policy
    assert_eq!(action, fault::FaultAction::Throttle);

    // Verify fault logged in compliance
    {
        let compliance = COMPLIANCE_TRACKER.lock();
        let report = compliance.as_ref().unwrap().generate_report();
        assert!(report.policy_violations > 0);
    }

    // Exit agent - should clean up from all subsystems
    let was_agent = hooks::on_process_exit(42, 0);
    assert!(was_agent);

    // Verify cleanup from supervisor
    {
        let supervisor = AGENT_SUPERVISOR.lock();
        assert!(supervisor.as_ref().unwrap().get_agent(agent_id).is_none());
    }

    // Verify telemetry recorded exit
    {
        let telemetry = TELEMETRY.lock();
        let snapshot = telemetry.as_ref().unwrap().snapshot();
        assert_eq!(snapshot.system.active_agents, 0);
        assert_eq!(snapshot.system.total_exits, 1);
    }

    // Verify compliance recorded exit event
    {
        let compliance = COMPLIANCE_TRACKER.lock();
        let report = compliance.as_ref().unwrap().generate_report();
        // Should still have agent record even after exit
        assert!(report.agent_records.iter().any(|r| r.agent_id == agent_id));
    }
}

/// Test multiple agents with dependencies
#[test]
fn test_dependency_tracking_integration() {
    init();

    // Spawn three agents
    let spec1 = AgentSpec::new(100, "agent1".to_string());
    let spec2 = AgentSpec::new(101, "agent2".to_string());
    let spec3 = AgentSpec::new(102, "agent3".to_string());

    hooks::on_process_spawn(10, spec1);
    hooks::on_process_spawn(11, spec2);
    hooks::on_process_spawn(12, spec3);

    // Create dependencies: 100 -> 101 (Required), 101 -> 102 (Required)
    {
        let mut dep_graph = DEPENDENCY_GRAPH.lock();
        let graph = dep_graph.as_mut().unwrap();

        graph.add_dependency(100, 101, DependencyType::Required);
        graph.add_dependency(101, 102, DependencyType::Required);
    }

    // If 102 exits, 100 and 101 should cascade
    {
        let dep_graph = DEPENDENCY_GRAPH.lock();
        let graph = dep_graph.as_ref().unwrap();

        let cascade = graph.get_cascade_exits(102);
        assert!(cascade.contains(&101));
        assert!(cascade.contains(&100));
    }

    // Exit all agents
    hooks::on_process_exit(10, 0);
    hooks::on_process_exit(11, 0);
    hooks::on_process_exit(12, 0);
}

/// Test resource monitoring over time
#[test]
fn test_resource_monitoring_integration() {
    init();

    let spec = AgentSpec::new(200, "resource_test".to_string());
    hooks::on_process_spawn(20, spec);

    // Simulate resource usage
    {
        let mut resource_mon = RESOURCE_MONITOR.lock();
        let monitor = resource_mon.as_mut().unwrap();

        if let Some(agent_monitor) = monitor.get_agent(200) {
            // Simulate CPU usage
            agent_monitor.record_cpu_time(100_000); // 100ms

            // Simulate syscalls
            for _ in 0..50 {
                agent_monitor.record_syscall();
            }

            // Simulate memory usage
            agent_monitor.update_memory(1024 * 1024); // 1MB

            // Verify tracking
            assert_eq!(agent_monitor.current_memory(), 1024 * 1024);
            let (total_cpu, total_syscalls, _) = agent_monitor.lifetime_stats();
            assert_eq!(total_cpu, 100_000);
            assert_eq!(total_syscalls, 50);
        }
    }

    hooks::on_process_exit(20, 0);
}

/// Test performance profiling
#[test]
fn test_profiling_integration() {
    init();

    let spec = AgentSpec::new(300, "profiling_test".to_string());
    hooks::on_process_spawn(30, spec);

    // Simulate profiled operations
    {
        let mut profiler = SYSTEM_PROFILER.lock();
        let system_prof = profiler.as_mut().unwrap();

        if let Some(agent_prof) = system_prof.get_agent(300) {
            // Profile some operations
            for _ in 0..10 {
                let start = agent_prof.start_operation("test_operation");
                // Simulate work
                agent_prof.end_operation("test_operation", start, true);
            }

            // Verify stats
            let stats = agent_prof.get_stats("test_operation").unwrap();
            assert_eq!(stats.sample_count, 10);
            assert_eq!(stats.success_rate, 1.0);
        }
    }

    hooks::on_process_exit(30, 0);
}

/// Test fault recovery with compliance tracking
#[test]
fn test_fault_recovery_compliance_integration() {
    init();

    let spec = AgentSpec::new(400, "fault_test".to_string());
    hooks::on_process_spawn(40, spec);

    // Report multiple faults
    let faults = vec![
        fault::Fault::CpuQuotaExceeded { used: 2_000_000, quota: 1_000_000 },
        fault::Fault::MemoryExceeded { used: 200_000_000, limit: 100_000_000 },
        fault::Fault::SyscallFlood { rate: 2000, threshold: 1000 },
    ];

    for fault in faults {
        hooks::report_agent_fault(400, fault);
    }

    // Verify compliance recorded all violations
    {
        let compliance = COMPLIANCE_TRACKER.lock();
        let report = compliance.as_ref().unwrap().generate_report();

        let agent_record = report.agent_records.iter()
            .find(|r| r.agent_id == 400)
            .expect("Agent should have compliance record");

        assert_eq!(agent_record.policy_violations, 3);
        // Compliance score should be lower due to violations
        assert!(agent_record.compliance_score < 1.0);
    }

    hooks::on_process_exit(40, 0);
}

/// Test policy hot-patching
#[test]
fn test_policy_hot_patch_integration() {
    init();

    let mut spec = AgentSpec::new(500, "policy_test".to_string());
    spec = spec.with_capability(Capability::FsBasic);

    hooks::on_process_spawn(50, spec);

    // Get initial policy
    {
        let policy_ctrl = POLICY_CONTROLLER.lock();
        let policy = policy_ctrl.as_ref().unwrap().get_policy(500).unwrap();
        assert!(policy.capabilities.contains(&Capability::FsBasic));
        assert!(!policy.capabilities.contains(&Capability::NetConnect));
    }

    // Hot-patch to add NetConnect (this should work for adding capabilities)
    {
        let mut policy_ctrl = POLICY_CONTROLLER.lock();
        let controller = policy_ctrl.as_mut().unwrap();

        let mut new_capabilities = Vec::new();
        new_capabilities.push(Capability::FsBasic);
        new_capabilities.push(Capability::NetConnect);

        let result = controller.update_agent_capabilities(500, new_capabilities);
        assert!(result.is_ok());
    }

    // Verify updated policy
    {
        let policy_ctrl = POLICY_CONTROLLER.lock();
        let policy = policy_ctrl.as_ref().unwrap().get_policy(500).unwrap();
        assert!(policy.capabilities.contains(&Capability::NetConnect));
    }

    hooks::on_process_exit(50, 0);
}

/// Test telemetry aggregation across multiple agents
#[test]
fn test_telemetry_aggregation_integration() {
    init();

    // Spawn multiple agents
    for i in 0..5 {
        let spec = AgentSpec::new(600 + i, alloc::format!("telemetry_agent_{}", i));
        hooks::on_process_spawn(60 + i, spec);
    }

    // Verify system metrics
    {
        let telemetry = TELEMETRY.lock();
        let snapshot = telemetry.as_ref().unwrap().snapshot();

        assert_eq!(snapshot.system.active_agents, 5);
        assert_eq!(snapshot.system.total_spawns, 5);
    }

    // Exit all agents
    for i in 0..5 {
        hooks::on_process_exit(60 + i, 0);
    }

    // Verify exit metrics
    {
        let telemetry = TELEMETRY.lock();
        let snapshot = telemetry.as_ref().unwrap().snapshot();

        assert_eq!(snapshot.system.active_agents, 0);
        assert_eq!(snapshot.system.total_exits, 5);
    }
}

/// Test compliance scoring algorithm
#[test]
fn test_compliance_scoring_integration() {
    init();

    let spec = AgentSpec::new(700, "compliance_scoring_test".to_string());
    hooks::on_process_spawn(70, spec);

    // Initial compliance score should be high
    {
        let compliance = COMPLIANCE_TRACKER.lock();
        let report = compliance.as_ref().unwrap().generate_report();
        let agent_record = report.agent_records.iter().find(|r| r.agent_id == 700).unwrap();
        assert!(agent_record.compliance_score > 0.9);
    }

    // Report violations to lower score
    for _ in 0..5 {
        let fault = fault::Fault::PolicyViolation { reason: "Test violation" };
        hooks::report_agent_fault(700, fault);
    }

    // Compliance score should decrease
    {
        let compliance = COMPLIANCE_TRACKER.lock();
        let report = compliance.as_ref().unwrap().generate_report();
        let agent_record = report.agent_records.iter().find(|r| r.agent_id == 700).unwrap();
        assert!(agent_record.compliance_score < 0.9);
        assert_eq!(agent_record.policy_violations, 5);
    }

    hooks::on_process_exit(70, 0);
}

/// Stress test: spawn and exit many agents
#[test]
fn test_stress_many_agents() {
    init();

    const AGENT_COUNT: usize = 100;

    // Spawn many agents
    for i in 0..AGENT_COUNT {
        let spec = AgentSpec::new(800 + i as AgentId, alloc::format!("stress_agent_{}", i));
        hooks::on_process_spawn(80 + i as u32, spec);
    }

    // Verify all registered
    {
        let telemetry = TELEMETRY.lock();
        let snapshot = telemetry.as_ref().unwrap().snapshot();
        assert_eq!(snapshot.system.active_agents, AGENT_COUNT);
    }

    // Exit all agents
    for i in 0..AGENT_COUNT {
        hooks::on_process_exit(80 + i as u32, 0);
    }

    // Verify all cleaned up
    {
        let telemetry = TELEMETRY.lock();
        let snapshot = telemetry.as_ref().unwrap().snapshot();
        assert_eq!(snapshot.system.active_agents, 0);
        assert_eq!(snapshot.system.total_exits, AGENT_COUNT as u64);
    }
}

/// Test system-wide resource aggregation
#[test]
fn test_system_resource_aggregation() {
    init();

    // Spawn multiple agents
    for i in 0..3 {
        let spec = AgentSpec::new(900 + i, alloc::format!("resource_agent_{}", i));
        hooks::on_process_spawn(90 + i, spec);
    }

    // Simulate resource usage for each agent
    {
        let mut resource_mon = RESOURCE_MONITOR.lock();
        let monitor = resource_mon.as_mut().unwrap();

        for i in 0..3 {
            if let Some(agent_monitor) = monitor.get_agent(900 + i) {
                agent_monitor.update_memory((i + 1) * 1024 * 1024); // 1MB, 2MB, 3MB
            }
        }

        // System should report total memory usage
        let total_memory = monitor.system_memory_usage();
        assert_eq!(total_memory, 6 * 1024 * 1024); // 6MB total
    }

    // Cleanup
    for i in 0..3 {
        hooks::on_process_exit(90 + i, 0);
    }
}

/// Test circular dependency detection
#[test]
fn test_circular_dependency_detection() {
    init();

    // Spawn three agents
    for i in 0..3 {
        let spec = AgentSpec::new(1000 + i, alloc::format!("circular_agent_{}", i));
        hooks::on_process_spawn(100 + i, spec);
    }

    // Create circular dependency: 1000 -> 1001 -> 1002 -> 1000
    {
        let mut dep_graph = DEPENDENCY_GRAPH.lock();
        let graph = dep_graph.as_mut().unwrap();

        graph.add_dependency(1000, 1001, DependencyType::Required);
        graph.add_dependency(1001, 1002, DependencyType::Required);
        graph.add_dependency(1002, 1000, DependencyType::Required);

        // Should detect the cycle
        assert!(graph.has_circular_dependency(1000));
        assert!(graph.has_circular_dependency(1001));
        assert!(graph.has_circular_dependency(1002));
    }

    // Cleanup
    for i in 0..3 {
        hooks::on_process_exit(100 + i, 0);
    }
}
