amoljassal@Amols-Mac-mini sis-kernel % cargo run -p sis-testing --release -- --phase 9
warning: profile package spec `bootloader` in profile `dev` did not match any packages
warning: profile package spec `bootloader_api` in profile `dev` did not match any packages
   Compiling sis-testing v0.1.0 (/Users/amoljassal/sis/sis-kernel/crates/testing)
    Finished `release` profile [optimized] target(s) in 19.77s
     Running `target/release/sis-test-runner --phase 9`
[2025-11-14T07:06:19Z INFO  sis_test_runner] SIS Kernel Industry-Grade Test Suite
[2025-11-14T07:06:19Z INFO  sis_test_runner] ====================================
[2025-11-14T07:06:19Z INFO  sis_test_runner] Mode: default (single QEMU node, moderate iterations)
[2025-11-14T07:06:19Z INFO  sis_test_runner] Test Configuration:
[2025-11-14T07:06:19Z INFO  sis_test_runner]   QEMU Nodes: 1
[2025-11-14T07:06:19Z INFO  sis_test_runner]   Duration: 600s
[2025-11-14T07:06:19Z INFO  sis_test_runner]   Performance Iterations: 2000
[2025-11-14T07:06:19Z INFO  sis_test_runner]   Statistical Confidence: 99.0%
[2025-11-14T07:06:19Z INFO  sis_test_runner]   Output Directory: target/testing
[2025-11-14T07:06:19Z INFO  sis_test_runner]   Parallel Execution: true
[2025-11-14T07:06:19Z INFO  sis_test_runner] Initializing QEMU runtime for kernel validation...
[2025-11-14T07:06:19Z INFO  sis_testing] Initializing QEMU runtime for comprehensive kernel testing
[2025-11-14T07:06:19Z INFO  sis_testing::qemu_runtime] Building SIS kernel for QEMU testing
[2025-11-14T07:06:19Z INFO  sis_testing::qemu_runtime] Building UEFI bootloader...
[2025-11-14T07:06:26Z INFO  sis_testing::qemu_runtime] Building kernel with features: bringup,graphctl-framed,deterministic,ai-ops,crypto-real,agentsys,llm,otel,decision-traces,model-lifecycle,shadow-mode
[2025-11-14T07:06:40Z INFO  sis_testing::qemu_runtime] SIS kernel and UEFI bootloader built successfully
[2025-11-14T07:06:40Z INFO  sis_testing::qemu_runtime] Preparing ESP directories for 1 QEMU instances
[2025-11-14T07:06:40Z INFO  sis_testing::qemu_runtime] ESP directories prepared for all instances
[2025-11-14T07:06:40Z INFO  sis_testing::qemu_runtime] Launching QEMU cluster with 1 nodes
[2025-11-14T07:06:40Z INFO  sis_testing::qemu_runtime] Launching QEMU instance 0 on ports 7000/7100/7200
[2025-11-14T07:06:40Z INFO  sis_testing::qemu_runtime] Instance 0 using PTY: /dev/ttys006
[2025-11-14T07:06:40Z INFO  sis_testing::qemu_runtime] Instance 0 launched (serial log: target/testing/serial-node0.log)
[2025-11-14T07:06:43Z INFO  sis_testing::qemu_runtime] All QEMU instances launched successfully
[2025-11-14T07:06:43Z INFO  sis_testing::qemu_runtime] Waiting for instance 0 to boot (timeout: 180s)
[2025-11-14T07:06:43Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-14T07:06:45Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-14T07:06:47Z INFO  sis_testing::qemu_runtime] Instance 0 booted successfully (detected via serial log)
[2025-11-14T07:06:47Z INFO  sis_testing] QEMU runtime initialized with 1 node(s); boot detected via serial log
[2025-11-14T07:06:47Z INFO  sis_test_runner] QEMU runtime initialized successfully - running real kernel tests
[2025-11-14T07:06:47Z INFO  sis_testing] Starting SIS Kernel Comprehensive Validation
[2025-11-14T07:06:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-14T07:06:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:07:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 149 attempts, ready for commands
[2025-11-14T07:07:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:07:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:07:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:07:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:07:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:07:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer hello world from sis shell --max-tokens 8' timeout=30s
[2025-11-14T07:07:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:07:47Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:07:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:07:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:07:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:07:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:07:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:08:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmjson' timeout=30s
[2025-11-14T07:08:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:08:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:08:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:08:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:08:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:08:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:08:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:08:47Z WARN  sis_testing] LLM smoke test failed to run llmjson: Test execution failed: Command 'llmjson' timed out after 30s. Output: 
[2025-11-14T07:08:47Z INFO  sis_testing] Loaded real performance metrics from target/testing/serial-node0.log
[2025-11-14T07:08:47Z INFO  sis_testing] Kernel command interface initialized for real AI validation
[2025-11-14T07:08:47Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-14T07:08:47Z INFO  sis_testing] Phase 1-9 test suites initialized successfully
[2025-11-14T07:08:47Z INFO  sis_testing::ai] Starting comprehensive AI inference validation
[2025-11-14T07:08:47Z INFO  sis_testing::ai] Executing REAL Phase 3 AI validation commands in kernel
[2025-11-14T07:08:47Z INFO  sis_testing::kernel_interface] Starting Phase 3 AI validation command suite execution
[2025-11-14T07:08:47Z INFO  sis_testing::kernel_interface] Testing basic command execution with 'help' command
[2025-11-14T07:08:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='help' timeout=30s
[2025-11-14T07:08:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:08:47Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:08:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:08:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:08:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:08:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:08:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:09:18Z WARN  sis_testing::ai] Real kernel validation failed: Test execution failed: Basic command execution failed: Test execution failed: Command 'help' timed out after 30s. Output: . Falling back to simulation.
[2025-11-14T07:09:18Z INFO  sis_testing::ai] Using simulated AI validation (no real kernel interface available)
[2025-11-14T07:09:18Z INFO  sis_testing::ai] Testing AI inference accuracy against reference implementations
[2025-11-14T07:09:18Z INFO  sis_testing::ai] AI inference accuracy: 99.9500% (99950/100000 samples)
[2025-11-14T07:09:18Z INFO  sis_testing::ai] Measuring Neural Engine utilization efficiency
[2025-11-14T07:09:18Z INFO  sis_testing::ai] Neural Engine utilization: 95.0%
[2025-11-14T07:09:18Z INFO  sis_testing::ai::benchmark_suite] Starting comprehensive AI inference benchmarks with Phase 3 metrics
[2025-11-14T07:09:18Z INFO  sis_testing::ai::benchmark_suite] Benchmarking AI inference latency
[2025-11-14T07:09:18Z INFO  sis_testing::ai::benchmark_suite] Measuring Neural Engine latency with 2000 iterations
[2025-11-14T07:09:18Z INFO  sis_testing::ai::benchmark_suite] Benchmarking AI inference throughput
[2025-11-14T07:09:18Z INFO  sis_testing::ai::benchmark_suite] Benchmarking memory efficiency
[2025-11-14T07:09:18Z INFO  sis_testing::ai::benchmark_suite] Benchmarking AI accuracy and precision
[2025-11-14T07:09:18Z INFO  sis_testing::ai::benchmark_suite] Benchmarking power efficiency
[2025-11-14T07:09:18Z INFO  sis_testing::ai::benchmark_suite] Benchmarking CBS+EDF deterministic scheduler AI metrics
[2025-11-14T07:09:18Z INFO  sis_testing::ai::benchmark_suite] Benchmarking NPU driver performance metrics
[2025-11-14T07:09:18Z INFO  sis_testing::ai::benchmark_suite] Benchmarking real-time AI inference guarantees
[2025-11-14T07:09:20Z INFO  sis_testing::ai::benchmark_suite] Measuring CPU fallback latency
[2025-11-14T07:09:23Z INFO  sis_testing::ai::benchmark_suite] Benchmarking batch size: 1
[2025-11-14T07:09:23Z INFO  sis_testing::ai::benchmark_suite] Benchmarking batch size: 4
[2025-11-14T07:09:24Z INFO  sis_testing::ai::benchmark_suite] Benchmarking batch size: 8
[2025-11-14T07:09:24Z INFO  sis_testing::ai::benchmark_suite] Benchmarking batch size: 16
[2025-11-14T07:09:24Z INFO  sis_testing::ai::benchmark_suite] Benchmarking batch size: 32
[2025-11-14T07:09:24Z INFO  sis_testing::ai::benchmark_suite] Benchmarking batch size: 64
[2025-11-14T07:10:13Z INFO  sis_testing::correctness] Starting comprehensive correctness validation
[2025-11-14T07:10:13Z INFO  sis_testing::correctness] Verifying memory safety properties
[2025-11-14T07:10:13Z INFO  sis_testing::correctness] Memory safety verification completed: 10000/10000 tests passed
[2025-11-14T07:10:13Z INFO  sis_testing::correctness] Running formal verification analysis
[2025-11-14T07:10:13Z INFO  sis_testing::correctness] Formal verification completed: 95.0% coverage
[2025-11-14T07:10:13Z INFO  sis_testing::correctness] Running property-based tests
[2025-11-14T07:10:13Z INFO  sis_testing::correctness] Property-based tests completed: 4999/5000 passed
[2025-11-14T07:10:13Z INFO  sis_testing::distributed] Starting Byzantine consensus validation
[2025-11-14T07:10:13Z INFO  sis_testing::distributed] Measuring consensus latency with 100 nodes
[2025-11-14T07:10:13Z INFO  sis_testing::security] Starting comprehensive security testing
[2025-11-14T07:10:13Z INFO  sis_testing::security] Testing kernel security with 2000 test configurations
[2025-11-14T07:10:13Z INFO  sis_testing::security] Running comprehensive fuzzing campaign
[2025-11-14T07:10:13Z INFO  sis_testing::security::fuzzing] Fuzzing system call interfaces
[2025-11-14T07:10:14Z INFO  sis_testing::security::fuzzing] Fuzzing memory management subsystem
[2025-11-14T07:10:14Z INFO  sis_testing::distributed] Average consensus latency: 5.45ms
[2025-11-14T07:10:14Z INFO  sis_testing::distributed] Testing Byzantine fault tolerance limits
[2025-11-14T07:10:14Z INFO  sis_testing::distributed] Byzantine fault tolerance: 33/100 nodes
[2025-11-14T07:10:14Z INFO  sis_testing::distributed] Measuring consensus success rate
[2025-11-14T07:10:14Z INFO  sis_testing::distributed] Consensus success rate: 99.900%
[2025-11-14T07:10:14Z INFO  sis_testing::distributed] Testing network partition recovery
[2025-11-14T07:10:14Z INFO  sis_testing::distributed] Network partition recovery time: 235.51ms
[2025-11-14T07:10:14Z INFO  sis_testing::distributed] Testing leader election performance
[2025-11-14T07:10:14Z INFO  sis_testing::distributed] Leader election time: 80.16ms
[2025-11-14T07:10:14Z INFO  sis_testing::security::fuzzing] Fuzzing I/O operations and device drivers
[2025-11-14T07:10:14Z INFO  sis_testing::security::fuzzing] Fuzzing network protocol stack
[2025-11-14T07:10:14Z INFO  sis_testing::security] Running vulnerability scans
[2025-11-14T07:10:14Z INFO  sis_testing::security::vulnerability_scanner] Checking for buffer overflow vulnerabilities
[2025-11-14T07:10:14Z INFO  sis_testing::security::vulnerability_scanner] Checking for integer overflow vulnerabilities
[2025-11-14T07:10:14Z INFO  sis_testing::security::vulnerability_scanner] Checking for use-after-free vulnerabilities
[2025-11-14T07:10:14Z INFO  sis_testing::security::vulnerability_scanner] Checking for double-free vulnerabilities
[2025-11-14T07:10:14Z INFO  sis_testing::security::vulnerability_scanner] Checking for race condition vulnerabilities
[2025-11-14T07:10:15Z INFO  sis_testing::security::vulnerability_scanner] Checking for privilege escalation vulnerabilities
[2025-11-14T07:10:15Z INFO  sis_testing::security::vulnerability_scanner] Checking for timing attack vulnerabilities
[2025-11-14T07:10:15Z INFO  sis_testing::security::vulnerability_scanner] Checking for side-channel vulnerabilities
[2025-11-14T07:10:15Z INFO  sis_testing::security] Running cryptographic validation tests
[2025-11-14T07:10:15Z INFO  sis_testing::security::crypto_validation] Testing randomness quality
[2025-11-14T07:10:16Z INFO  sis_testing::security::crypto_validation] Testing encryption algorithm strength
[2025-11-14T07:10:16Z INFO  sis_testing::security::crypto_validation] Testing key management practices
[2025-11-14T07:10:16Z INFO  sis_testing::security::crypto_validation] Testing hash function security properties
[2025-11-14T07:10:16Z INFO  sis_testing::security::crypto_validation] Testing side-channel attack resistance
[2025-11-14T07:10:17Z INFO  sis_testing::security] Running memory safety analysis
[2025-11-14T07:10:17Z INFO  sis_testing::security::memory_safety] Checking stack overflow protection
[2025-11-14T07:10:17Z INFO  sis_testing::security::memory_safety] Stack protection analysis complete: true
[2025-11-14T07:10:17Z INFO  sis_testing::security::memory_safety] Checking heap overflow protection
[2025-11-14T07:10:17Z INFO  sis_testing::security::memory_safety] Heap protection analysis complete: true
[2025-11-14T07:10:17Z INFO  sis_testing::security::memory_safety] Checking use-after-free detection capabilities
[2025-11-14T07:10:17Z INFO  sis_testing::security::memory_safety] Use-after-free detection: 100.0% success rate
[2025-11-14T07:10:17Z INFO  sis_testing::security::memory_safety] Checking double-free detection capabilities
[2025-11-14T07:10:17Z INFO  sis_testing::security::memory_safety] Double-free detection: 100.0% success rate
[2025-11-14T07:10:17Z INFO  sis_testing::security::memory_safety] Running comprehensive memory leak detection
[2025-11-14T07:10:17Z INFO  sis_testing::security::memory_safety] Checking control flow integrity
[2025-11-14T07:10:17Z INFO  sis_testing::security::memory_safety] Control flow integrity: true
[2025-11-14T07:10:17Z INFO  sis_testing::security::memory_safety] Checking stack canary protection
[2025-11-14T07:10:18Z INFO  sis_testing::security::memory_safety] Stack canary protection: true
[2025-11-14T07:10:18Z INFO  sis_testing::security::memory_safety] Measuring ASLR effectiveness
[2025-11-14T07:10:18Z INFO  sis_testing::security::memory_safety] ASLR effectiveness: 0.88
[2025-11-14T07:10:18Z INFO  sis_testing] Running Phase 1-9 comprehensive test suites
[2025-11-14T07:10:18Z INFO  sis_testing::phase1_dataflow] ==================================================
[2025-11-14T07:10:18Z INFO  sis_testing::phase1_dataflow] Starting Phase 1: AI-Native Dataflow Validation
[2025-11-14T07:10:18Z INFO  sis_testing::phase1_dataflow] ==================================================
[2025-11-14T07:10:18Z INFO  sis_testing::phase1_dataflow::graph_execution] Running Graph Execution Tests...
[2025-11-14T07:10:18Z INFO  sis_testing::phase1_dataflow::graph_execution]   Testing graph creation...
[2025-11-14T07:10:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-14T07:10:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:10:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:10:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:10:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:10:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:10:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:10:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:10:48Z INFO  sis_testing::phase1_dataflow::graph_execution]   Testing operator addition...
[2025-11-14T07:10:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=30s
[2025-11-14T07:10:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:10:48Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:10:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:10:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:10:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:10:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:10:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:11:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in none --out 0 --prio 10' timeout=30s
[2025-11-14T07:11:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:11:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:11:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:11:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:11:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:11:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:11:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:11:48Z INFO  sis_testing::phase1_dataflow::graph_execution]   Testing graph execution...
[2025-11-14T07:11:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-14T07:11:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:11:48Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:11:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:11:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:11:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:11:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:11:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:12:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in none --out 0 --prio 10' timeout=30s
[2025-11-14T07:12:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:12:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:12:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:12:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:12:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:12:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:12:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:12:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=30s
[2025-11-14T07:12:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:12:48Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:12:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:12:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:12:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:12:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:12:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:13:18Z INFO  sis_testing::phase1_dataflow::graph_execution]   Testing graph cleanup...
[2025-11-14T07:13:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 3' timeout=30s
[2025-11-14T07:13:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:13:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:13:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:13:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:13:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:13:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:13:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:13:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl destroy' timeout=30s
[2025-11-14T07:13:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:13:48Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:13:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:13:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:13:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:13:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:13:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:14:18Z INFO  sis_testing::phase1_dataflow::graph_execution]     ✅ Graph cleanup: PASSED
[2025-11-14T07:14:18Z INFO  sis_testing::phase1_dataflow::graph_execution] Graph Execution Tests: 1/4 passed (25%)
[2025-11-14T07:14:18Z INFO  sis_testing::phase1_dataflow::operator_validation] Running Operator Validation Tests...
[2025-11-14T07:14:18Z INFO  sis_testing::phase1_dataflow::operator_validation]   Testing operator types...
[2025-11-14T07:14:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=30s
[2025-11-14T07:14:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:14:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:14:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:14:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:14:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:14:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:14:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:14:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=30s
[2025-11-14T07:14:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:14:48Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:14:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:14:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:14:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:14:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:14:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:15:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in 0 --out 2 --prio 5' timeout=30s
[2025-11-14T07:15:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:15:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:15:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:15:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:15:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:15:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:15:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:15:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in 1 --out none --prio 1' timeout=30s
[2025-11-14T07:15:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:15:48Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:15:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:15:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:15:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:15:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:15:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:16:18Z WARN  sis_testing::phase1_dataflow::operator_validation]     ❌ Operator types: FAILED
[2025-11-14T07:16:18Z INFO  sis_testing::phase1_dataflow::operator_validation]   Testing operator priorities...
[2025-11-14T07:16:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-14T07:16:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:16:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:16:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:16:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:16:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:16:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:16:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:16:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 0 --prio 10' timeout=30s
[2025-11-14T07:16:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:16:48Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:16:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:16:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:16:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:16:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:16:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:17:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in none --out 0 --prio 5' timeout=30s
[2025-11-14T07:17:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:17:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:17:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:17:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:17:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:17:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:17:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:17:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in none --out 0 --prio 15' timeout=30s
[2025-11-14T07:17:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:17:49Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:17:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:17:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:17:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:17:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:17:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:18:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 3 --in none --out 0 --prio 1' timeout=30s
[2025-11-14T07:18:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:18:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:18:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:18:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:18:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:18:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:18:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:18:49Z WARN  sis_testing::phase1_dataflow::operator_validation]     ❌ Operator priorities: FAILED
[2025-11-14T07:18:49Z INFO  sis_testing::phase1_dataflow::operator_validation]   Testing operator connections...
[2025-11-14T07:18:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-14T07:18:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:18:49Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:18:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:18:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:18:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:18:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:18:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:19:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=30s
[2025-11-14T07:19:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:19:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:19:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:19:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:19:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:19:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:19:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in 0 --out 2 --prio 5' timeout=30s
[2025-11-14T07:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:19:49Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:20:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in 1 --out none --prio 1' timeout=30s
[2025-11-14T07:20:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:20:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:20:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:20:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:20:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:20:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:20:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 10' timeout=30s
[2025-11-14T07:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:20:49Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:21:19Z INFO  sis_testing::phase1_dataflow::operator_validation] Operator Validation Tests: 0/3 passed (0%)
[2025-11-14T07:21:19Z INFO  sis_testing::phase1_dataflow::channel_throughput] Running Channel Throughput Tests...
[2025-11-14T07:21:19Z INFO  sis_testing::phase1_dataflow::channel_throughput]   Testing basic channel throughput...
[2025-11-14T07:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 3' timeout=30s
[2025-11-14T07:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:21:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:21:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=30s
[2025-11-14T07:21:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:21:49Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:21:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:21:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:21:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:21:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:21:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:22:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in 0 --out 2 --prio 5' timeout=30s
[2025-11-14T07:22:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:22:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:22:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:22:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:22:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:22:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:22:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:22:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 50' timeout=30s
[2025-11-14T07:22:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:22:49Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:22:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:22:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:22:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:22:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:22:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:23:19Z INFO  sis_testing::phase1_dataflow::channel_throughput]   Testing high volume transfer...
[2025-11-14T07:23:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=30s
[2025-11-14T07:23:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:23:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:23:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:23:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:23:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:23:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:23:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:23:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 0 --prio 10' timeout=30s
[2025-11-14T07:23:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:23:49Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:23:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:23:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:23:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:23:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:23:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in none --out 0 --prio 9' timeout=30s
[2025-11-14T07:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:24:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in none --out 0 --prio 8' timeout=30s
[2025-11-14T07:24:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:24:50Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:24:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:24:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:24:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:24:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:24:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:25:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 3 --in none --out 0 --prio 7' timeout=30s
[2025-11-14T07:25:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:25:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:25:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:25:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:25:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:25:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:25:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:25:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 4 --in none --out 0 --prio 6' timeout=30s
[2025-11-14T07:25:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:25:50Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:25:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:25:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:25:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:25:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:25:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:26:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 500' timeout=30s
[2025-11-14T07:26:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:26:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:26:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:26:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:26:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:26:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:26:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:26:50Z INFO  sis_testing::phase1_dataflow::channel_throughput]   Testing backpressure handling...
[2025-11-14T07:26:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-14T07:26:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:26:50Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:26:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:26:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:26:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:26:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:26:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:27:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=30s
[2025-11-14T07:27:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:27:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:27:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:27:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:27:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:27:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:27:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:27:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in 0 --out none --prio 1' timeout=30s
[2025-11-14T07:27:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:27:50Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:27:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:27:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:27:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:27:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:27:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:28:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=30s
[2025-11-14T07:28:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:28:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:28:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:28:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:28:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:28:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:28:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:28:50Z INFO  sis_testing::phase1_dataflow::channel_throughput] Channel Throughput Tests: 0/3 passed (0%)
[2025-11-14T07:28:50Z INFO  sis_testing::phase1_dataflow::tensor_operations] Running Tensor Operations Tests...
[2025-11-14T07:28:50Z INFO  sis_testing::phase1_dataflow::tensor_operations]   Testing tensor creation...
[2025-11-14T07:28:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-14T07:28:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:28:50Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:28:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:28:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:28:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:28:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:28:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:29:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=30s
[2025-11-14T07:29:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:29:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:29:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:29:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:29:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:29:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:29:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:29:50Z INFO  sis_testing::phase1_dataflow::tensor_operations]   Testing tensor transformation...
[2025-11-14T07:29:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-14T07:29:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:29:50Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:29:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:29:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:29:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:29:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:29:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:30:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=30s
[2025-11-14T07:30:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:30:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:30:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:30:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:30:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:30:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:30:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:30:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in 0 --out 2 --prio 5' timeout=30s
[2025-11-14T07:30:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:30:51Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:30:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:30:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:30:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:30:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:30:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:31:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in 1 --out none --prio 1' timeout=30s
[2025-11-14T07:31:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:31:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:31:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:31:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:31:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:31:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:31:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:31:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 20' timeout=30s
[2025-11-14T07:31:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:31:51Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:31:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:31:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:31:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:31:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:31:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:32:21Z INFO  sis_testing::phase1_dataflow::tensor_operations]   Testing tensor data validation...
[2025-11-14T07:32:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=30s
[2025-11-14T07:32:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:32:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:32:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:32:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:32:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:32:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:32:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:32:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 0 --prio 10' timeout=30s
[2025-11-14T07:32:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:32:51Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:32:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:32:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:32:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:32:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:32:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:33:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in none --out 0 --prio 9' timeout=30s
[2025-11-14T07:33:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:33:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:33:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:33:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:33:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:33:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:33:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:33:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in none --out 0 --prio 8' timeout=30s
[2025-11-14T07:33:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:33:51Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:33:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:33:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:33:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:33:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:33:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:34:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 3 --in none --out 0 --prio 7' timeout=30s
[2025-11-14T07:34:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:34:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:34:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:34:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:34:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:34:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:34:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:34:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 4 --in none --out 0 --prio 6' timeout=30s
[2025-11-14T07:34:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:34:51Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:34:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:34:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:34:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:34:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:34:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:35:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=30s
[2025-11-14T07:35:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:35:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:35:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:35:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:35:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:35:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:35:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:35:51Z INFO  sis_testing::phase1_dataflow::tensor_operations] Tensor Operations Tests: 0/3 passed (0%)
[2025-11-14T07:35:51Z INFO  sis_testing::phase1_dataflow] ==================================================
[2025-11-14T07:35:51Z INFO  sis_testing::phase1_dataflow] Phase 1 Summary:
[2025-11-14T07:35:51Z INFO  sis_testing::phase1_dataflow]   Graph Execution:      ❌ FAILED
[2025-11-14T07:35:51Z INFO  sis_testing::phase1_dataflow]   Operator Validation:  ❌ FAILED
[2025-11-14T07:35:51Z INFO  sis_testing::phase1_dataflow]   Channel Throughput:   ❌ FAILED
[2025-11-14T07:35:51Z INFO  sis_testing::phase1_dataflow]   Tensor Operations:    ❌ FAILED
[2025-11-14T07:35:51Z INFO  sis_testing::phase1_dataflow]   Overall:              1/13 tests passed (7.7%)
[2025-11-14T07:35:51Z INFO  sis_testing::phase1_dataflow] ==================================================
[2025-11-14T07:35:51Z INFO  sis_testing::phase2_governance] =================================================
[2025-11-14T07:35:51Z INFO  sis_testing::phase2_governance] Starting Phase 2: AI Governance & Safety Policies
[2025-11-14T07:35:51Z INFO  sis_testing::phase2_governance] =================================================
[2025-11-14T07:35:51Z INFO  sis_testing::phase2_governance::model_governance] Running Model Governance Tests...
[2025-11-14T07:35:51Z INFO  sis_testing::phase2_governance::model_governance]   Testing model registration...
[2025-11-14T07:35:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 7 --ctx 2048 --vocab 50000 --quant int8 --size-bytes 1048576' timeout=30s
[2025-11-14T07:35:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:35:51Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:35:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:35:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:35:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:35:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:35:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:36:21Z INFO  sis_testing::phase2_governance::model_governance]   Testing model versioning...
[2025-11-14T07:36:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-14T07:36:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:36:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:36:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:36:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:36:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:36:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:36:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:36:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl status' timeout=30s
[2025-11-14T07:36:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:36:51Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:36:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:36:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:36:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:36:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:36:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:37:21Z INFO  sis_testing::phase2_governance::model_governance]   Testing model metadata validation...
[2025-11-14T07:37:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 7 --ctx 512 --vocab 50000 --quant int8 --size-bytes 524288' timeout=30s
[2025-11-14T07:37:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:37:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:37:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:37:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:37:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:37:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:37:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:37:52Z INFO  sis_testing::phase2_governance::model_governance] Model Governance Tests: 0/3 passed (0%)
[2025-11-14T07:37:52Z INFO  sis_testing::phase2_governance::policy_enforcement] Running Policy Enforcement Tests...
[2025-11-14T07:37:52Z INFO  sis_testing::phase2_governance::policy_enforcement]   Testing model size limit enforcement...
[2025-11-14T07:37:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 7 --ctx 512 --vocab 50000 --quant int8 --size-bytes 134217728' timeout=30s
[2025-11-14T07:37:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:37:52Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:37:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:37:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:37:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:37:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:37:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:38:22Z INFO  sis_testing::phase2_governance::policy_enforcement]     ✅ Size limit enforcement: PASSED
[2025-11-14T07:38:22Z INFO  sis_testing::phase2_governance::policy_enforcement]   Testing token budget enforcement...
[2025-11-14T07:38:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-14T07:38:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:38:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:38:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:38:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:38:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:38:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:38:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:38:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl budget --period-ns 1000000000 --max-tokens-per-period 10' timeout=30s
[2025-11-14T07:38:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:38:52Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:38:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:38:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:38:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:38:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:38:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:39:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test message --max-tokens 5' timeout=30s
[2025-11-14T07:39:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:39:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:39:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:39:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:39:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:39:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:39:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:39:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl status' timeout=30s
[2025-11-14T07:39:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:39:52Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:39:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:39:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:39:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:39:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:39:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:40:22Z WARN  sis_testing::phase2_governance::policy_enforcement]     ❌ Budget enforcement: FAILED
[2025-11-14T07:40:22Z INFO  sis_testing::phase2_governance::policy_enforcement]   Testing rate limiting...
[2025-11-14T07:40:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-14T07:40:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:40:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:40:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:40:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:40:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:40:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:40:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:40:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl budget --period-ns 1000000000 --max-tokens-per-period 20' timeout=30s
[2025-11-14T07:40:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:40:52Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:40:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:40:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:40:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:40:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:40:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:41:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test 0 --max-tokens 3' timeout=30s
[2025-11-14T07:41:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:41:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:41:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:41:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:41:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:41:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:41:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:41:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test 1 --max-tokens 3' timeout=30s
[2025-11-14T07:41:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:41:52Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:41:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:41:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:41:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:41:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:41:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:42:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test 2 --max-tokens 3' timeout=30s
[2025-11-14T07:42:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:42:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:42:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:42:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:42:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:42:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:42:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:42:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test 3 --max-tokens 3' timeout=30s
[2025-11-14T07:42:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:42:52Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:42:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:42:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:42:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:42:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:42:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:43:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test 4 --max-tokens 3' timeout=30s
[2025-11-14T07:43:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:43:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:43:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:43:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:43:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:43:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:43:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:43:52Z WARN  sis_testing::phase2_governance::policy_enforcement]     ❌ Rate limiting: FAILED
[2025-11-14T07:43:52Z INFO  sis_testing::phase2_governance::policy_enforcement] Policy Enforcement Tests: 1/3 passed (33%)
[2025-11-14T07:43:52Z INFO  sis_testing::phase2_governance::audit_compliance] Running Audit Compliance Tests...
[2025-11-14T07:43:52Z INFO  sis_testing::phase2_governance::audit_compliance]   Testing audit logging...
[2025-11-14T07:43:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-14T07:43:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:43:52Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:43:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:43:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:43:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:43:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:43:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:44:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test audit message --max-tokens 5' timeout=30s
[2025-11-14T07:44:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:44:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:44:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:44:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:44:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:44:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:44:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:44:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmjson' timeout=30s
[2025-11-14T07:44:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:44:52Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:44:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:44:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:44:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:44:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:44:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:45:22Z INFO  sis_testing::phase2_governance::audit_compliance]   Testing compliance tracking...
[2025-11-14T07:45:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-14T07:45:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:45:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:45:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:45:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:45:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:45:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:45:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:45:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer compliance test 0 --max-tokens 3' timeout=30s
[2025-11-14T07:45:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:45:52Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:45:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:45:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:45:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:45:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:45:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:46:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer compliance test 1 --max-tokens 3' timeout=30s
[2025-11-14T07:46:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:46:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:46:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:46:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:46:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:46:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:46:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:46:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer compliance test 2 --max-tokens 3' timeout=30s
[2025-11-14T07:46:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:46:52Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:46:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:46:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:46:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:46:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:46:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:47:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl status' timeout=30s
[2025-11-14T07:47:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:47:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:47:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:47:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:47:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:47:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:47:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:47:52Z INFO  sis_testing::phase2_governance::audit_compliance]   Testing decision traceability...
[2025-11-14T07:47:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-14T07:47:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:47:52Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:47:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:47:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:47:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:47:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:47:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:48:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer traceable decision test --max-tokens 8' timeout=30s
[2025-11-14T07:48:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:48:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:48:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:48:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:48:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:48:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:48:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:48:53Z INFO  sis_testing::phase2_governance::audit_compliance] Audit Compliance Tests: 0/3 passed (0%)
[2025-11-14T07:48:53Z INFO  sis_testing::phase2_governance] =================================================
[2025-11-14T07:48:53Z INFO  sis_testing::phase2_governance] Phase 2 Summary:
[2025-11-14T07:48:53Z INFO  sis_testing::phase2_governance]   Model Governance:     ❌ FAILED
[2025-11-14T07:48:53Z INFO  sis_testing::phase2_governance]   Policy Enforcement:   ❌ FAILED
[2025-11-14T07:48:53Z INFO  sis_testing::phase2_governance]   Audit & Compliance:   ❌ FAILED
[2025-11-14T07:48:53Z INFO  sis_testing::phase2_governance]   Overall:              1/9 tests passed (11.1%)
[2025-11-14T07:48:53Z INFO  sis_testing::phase2_governance] =================================================
[2025-11-14T07:48:53Z INFO  sis_testing::phase3_temporal] ==================================================
[2025-11-14T07:48:53Z INFO  sis_testing::phase3_temporal] Starting Phase 3: Temporal Isolation Validation
[2025-11-14T07:48:53Z INFO  sis_testing::phase3_temporal] ==================================================
[2025-11-14T07:48:53Z INFO  sis_testing::phase3_temporal::active_isolation] Running Active Isolation Tests...
[2025-11-14T07:48:53Z INFO  sis_testing::phase3_temporal::active_isolation]   Testing temporal isolation verification...
[2025-11-14T07:48:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='rtaivalidation' timeout=30s
[2025-11-14T07:48:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:48:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:48:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:48:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:48:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:48:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:48:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:49:23Z INFO  sis_testing::phase3_temporal::active_isolation]   Testing jitter measurement...
[2025-11-14T07:49:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='rtaivalidation' timeout=30s
[2025-11-14T07:49:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:49:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:49:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:49:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:49:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:49:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:49:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:49:53Z INFO  sis_testing::phase3_temporal::active_isolation]   Testing isolation under load...
[2025-11-14T07:49:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-14T07:49:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:49:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:49:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:49:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:49:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:49:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:49:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:50:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 50' timeout=30s
[2025-11-14T07:50:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:50:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:50:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:50:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:50:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:50:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:50:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:50:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='rtaivalidation' timeout=30s
[2025-11-14T07:50:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:50:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:50:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:50:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:50:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:50:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:50:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:51:23Z INFO  sis_testing::phase3_temporal::active_isolation]     ✅ Isolation under load: PASSED
[2025-11-14T07:51:23Z INFO  sis_testing::phase3_temporal::active_isolation] Active Isolation Tests: 1/3 passed (33%)
[2025-11-14T07:51:23Z INFO  sis_testing::phase3_temporal::deadline_validation] Running Deadline Validation Tests...
[2025-11-14T07:51:23Z INFO  sis_testing::phase3_temporal::deadline_validation]   Testing deadline met validation...
[2025-11-14T07:51:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 5000000 10000000 10000000' timeout=30s
[2025-11-14T07:51:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:51:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:51:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:51:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:51:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:51:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:51:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:51:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=30s
[2025-11-14T07:51:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:51:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:51:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:51:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:51:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:51:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:51:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:52:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=30s
[2025-11-14T07:52:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:52:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:52:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:52:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:52:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:52:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:52:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:52:53Z INFO  sis_testing::phase3_temporal::deadline_validation]   Testing deadline miss detection...
[2025-11-14T07:52:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 1000000 2000000 2000000' timeout=30s
[2025-11-14T07:52:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:52:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:52:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:52:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:52:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:52:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:52:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:53:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=30s
[2025-11-14T07:53:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:53:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:53:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:53:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:53:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:53:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:53:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:53:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=30s
[2025-11-14T07:53:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:53:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:53:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:53:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:53:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:53:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:53:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:54:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=30s
[2025-11-14T07:54:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:54:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:54:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:54:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:54:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:54:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:54:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:54:53Z INFO  sis_testing::phase3_temporal::deadline_validation]   Testing WCET validation...
[2025-11-14T07:54:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 10000000 50000000 50000000' timeout=30s
[2025-11-14T07:54:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:54:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:54:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:54:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:54:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:54:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:54:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:55:23Z INFO  sis_testing::phase3_temporal::deadline_validation]   Testing periodic deadline guarantees...
[2025-11-14T07:55:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 5000000 20000000 20000000' timeout=30s
[2025-11-14T07:55:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:55:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:55:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:55:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:55:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:55:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:55:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:55:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-14T07:55:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:55:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:55:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:55:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:55:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:55:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:55:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:56:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 200' timeout=30s
[2025-11-14T07:56:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:56:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:56:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:56:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:56:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:56:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:56:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:56:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=30s
[2025-11-14T07:56:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:56:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:56:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:56:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:56:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:56:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:56:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:57:24Z INFO  sis_testing::phase3_temporal::deadline_validation] Deadline Validation Tests: 0/4 passed (0%)
[2025-11-14T07:57:24Z INFO  sis_testing::phase3_temporal::latency_tests] Running Latency Tests...
[2025-11-14T07:57:24Z INFO  sis_testing::phase3_temporal::latency_tests]   Testing baseline latency...
[2025-11-14T07:57:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='rtaivalidation' timeout=30s
[2025-11-14T07:57:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:57:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:57:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:57:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:57:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:57:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:57:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:57:54Z INFO  sis_testing::phase3_temporal::latency_tests]   Testing latency under load...
[2025-11-14T07:57:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=30s
[2025-11-14T07:57:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:57:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:57:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:57:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:57:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:57:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:57:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:58:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=30s
[2025-11-14T07:58:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:58:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:58:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:58:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:58:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:58:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:58:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:58:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='rtaivalidation' timeout=30s
[2025-11-14T07:58:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:58:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:58:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:58:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:58:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:58:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:58:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:59:24Z INFO  sis_testing::phase3_temporal::latency_tests]     ✅ Latency under load: PASSED
[2025-11-14T07:59:24Z INFO  sis_testing::phase3_temporal::latency_tests]   Testing latency stability...
[2025-11-14T07:59:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 3000000 10000000 10000000' timeout=30s
[2025-11-14T07:59:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:59:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:59:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:59:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:59:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:59:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:59:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T07:59:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=30s
[2025-11-14T07:59:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T07:59:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T07:59:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T07:59:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T07:59:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T07:59:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T07:59:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:00:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=30s
[2025-11-14T08:00:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:00:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:00:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:00:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:00:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:00:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:00:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:00:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=30s
[2025-11-14T08:00:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:00:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:00:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:00:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:00:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:00:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:00:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:01:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=30s
[2025-11-14T08:01:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:01:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:01:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:01:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:01:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:01:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:01:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:01:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=30s
[2025-11-14T08:01:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:01:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:01:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:01:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:01:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:01:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:01:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:02:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=30s
[2025-11-14T08:02:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:02:25Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:02:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:02:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:02:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:02:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:02:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:02:55Z INFO  sis_testing::phase3_temporal::latency_tests] Latency Tests: 1/3 passed (33%)
[2025-11-14T08:02:55Z INFO  sis_testing::phase3_temporal] ==================================================
[2025-11-14T08:02:55Z INFO  sis_testing::phase3_temporal] Phase 3 Summary:
[2025-11-14T08:02:55Z INFO  sis_testing::phase3_temporal]   Active Isolation:     ❌ FAILED
[2025-11-14T08:02:55Z INFO  sis_testing::phase3_temporal]   Deadline Validation:  ❌ FAILED
[2025-11-14T08:02:55Z INFO  sis_testing::phase3_temporal]   Latency Tests:        ❌ FAILED
[2025-11-14T08:02:55Z INFO  sis_testing::phase3_temporal]   Overall:              2/10 tests passed (20.0%)
[2025-11-14T08:02:55Z INFO  sis_testing::phase3_temporal] ==================================================
[2025-11-14T08:02:55Z INFO  sis_testing::phase5_ux_safety] =================================================
[2025-11-14T08:02:55Z INFO  sis_testing::phase5_ux_safety] Starting Phase 5: User Experience Safety
[2025-11-14T08:02:55Z INFO  sis_testing::phase5_ux_safety] =================================================
[2025-11-14T08:02:55Z INFO  sis_testing::phase5_ux_safety::safety_controls] Running Safety Controls Tests...
[2025-11-14T08:02:55Z INFO  sis_testing::phase5_ux_safety::safety_controls]   Testing inference guardrails...
[2025-11-14T08:02:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-14T08:02:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:02:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:02:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:02:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:02:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:02:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:02:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:03:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl budget --period-ns 1000000000 --max-tokens-per-period 5' timeout=30s
[2025-11-14T08:03:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:03:25Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:03:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:03:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:03:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:03:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:03:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:03:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer safe test --max-tokens 3' timeout=30s
[2025-11-14T08:03:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:03:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:03:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:03:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:03:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:03:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:03:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:04:25Z WARN  sis_testing::phase5_ux_safety::safety_controls]     ❌ Inference guardrails: FAILED
[2025-11-14T08:04:25Z INFO  sis_testing::phase5_ux_safety::safety_controls]   Testing resource protection...
[2025-11-14T08:04:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 70 --ctx 32768 --vocab 100000 --quant int8 --size-bytes 268435456' timeout=30s
[2025-11-14T08:04:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:04:25Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:04:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:04:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:04:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:04:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:04:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:04:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 7 --ctx 512 --vocab 50000 --quant int8 --size-bytes 524288' timeout=30s
[2025-11-14T08:04:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:04:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:04:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:04:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:04:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:04:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:04:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:05:25Z INFO  sis_testing::phase5_ux_safety::safety_controls]     ✅ Resource protection: PASSED
[2025-11-14T08:05:25Z INFO  sis_testing::phase5_ux_safety::safety_controls]   Testing safety validation...
[2025-11-14T08:05:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-14T08:05:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:05:25Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:05:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:05:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:05:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:05:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:05:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:05:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer validation test --max-tokens 5' timeout=30s
[2025-11-14T08:05:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:05:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:05:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:05:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:05:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:05:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:05:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:06:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl status' timeout=30s
[2025-11-14T08:06:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:06:25Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:06:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:06:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:06:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:06:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:06:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:06:55Z WARN  sis_testing::phase5_ux_safety::safety_controls]     ❌ Safety validation: FAILED
[2025-11-14T08:06:55Z INFO  sis_testing::phase5_ux_safety::safety_controls] Safety Controls Tests: 1/3 passed (33%)
[2025-11-14T08:06:55Z INFO  sis_testing::phase5_ux_safety::explainability] Running Explainability Tests...
[2025-11-14T08:06:55Z INFO  sis_testing::phase5_ux_safety::explainability]   Testing decision transparency...
[2025-11-14T08:06:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-14T08:06:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:06:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:06:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:06:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:06:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:06:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:06:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:07:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer transparency test input --max-tokens 5' timeout=30s
[2025-11-14T08:07:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:07:25Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:07:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:07:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:07:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:07:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:07:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:07:55Z INFO  sis_testing::phase5_ux_safety::explainability]   Testing model introspection...
[2025-11-14T08:07:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 7 --ctx 2048 --vocab 50000 --quant int8 --size-bytes 1048576' timeout=30s
[2025-11-14T08:07:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:07:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:07:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:07:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:07:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:07:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:07:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:08:25Z INFO  sis_testing::phase5_ux_safety::explainability]   Testing audit accessibility...
[2025-11-14T08:08:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-14T08:08:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:08:25Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:08:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:08:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:08:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:08:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:08:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:08:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer audit test 0 --max-tokens 3' timeout=30s
[2025-11-14T08:08:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:08:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:08:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:08:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:08:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:08:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:08:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:09:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer audit test 1 --max-tokens 3' timeout=30s
[2025-11-14T08:09:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:09:25Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:09:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:09:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:09:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:09:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:09:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:09:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer audit test 2 --max-tokens 3' timeout=30s
[2025-11-14T08:09:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:09:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:09:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:09:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:09:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:09:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:09:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:10:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmjson' timeout=30s
[2025-11-14T08:10:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:10:25Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:10:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:10:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:10:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:10:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:10:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:10:56Z INFO  sis_testing::phase5_ux_safety::explainability] Explainability Tests: 0/3 passed (0%)
[2025-11-14T08:10:56Z INFO  sis_testing::phase5_ux_safety::user_feedback] Running User Feedback Tests...
[2025-11-14T08:10:56Z INFO  sis_testing::phase5_ux_safety::user_feedback]   Testing error reporting...
[2025-11-14T08:10:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test without model --max-tokens 5' timeout=30s
[2025-11-14T08:10:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:10:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:10:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:10:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:10:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:10:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:10:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:11:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-14T08:11:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:11:26Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:11:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:11:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:11:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:11:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:11:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:11:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer valid test --max-tokens 3' timeout=30s
[2025-11-14T08:11:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:11:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:11:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:11:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:11:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:11:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:11:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:12:26Z INFO  sis_testing::phase5_ux_safety::user_feedback]     ✅ Error reporting: PASSED
[2025-11-14T08:12:26Z INFO  sis_testing::phase5_ux_safety::user_feedback]   Testing status feedback...
[2025-11-14T08:12:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-14T08:12:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:12:26Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:12:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:12:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:12:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:12:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:12:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:12:56Z INFO  sis_testing::phase5_ux_safety::user_feedback]   Testing operation confirmation...
[2025-11-14T08:12:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-14T08:12:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:12:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:12:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:12:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:12:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:12:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:12:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:13:26Z INFO  sis_testing::phase5_ux_safety::user_feedback] User Feedback Tests: 1/3 passed (33%)
[2025-11-14T08:13:26Z INFO  sis_testing::phase5_ux_safety] =================================================
[2025-11-14T08:13:26Z INFO  sis_testing::phase5_ux_safety] Phase 5 Summary:
[2025-11-14T08:13:26Z INFO  sis_testing::phase5_ux_safety]   Safety Controls:      ❌ FAILED
[2025-11-14T08:13:26Z INFO  sis_testing::phase5_ux_safety]   Explainability:       ❌ FAILED
[2025-11-14T08:13:26Z INFO  sis_testing::phase5_ux_safety]   User Feedback:        ❌ FAILED
[2025-11-14T08:13:26Z INFO  sis_testing::phase5_ux_safety]   Overall:              2/9 tests passed (22.2%)
[2025-11-14T08:13:26Z INFO  sis_testing::phase5_ux_safety] =================================================
[2025-11-14T08:13:26Z INFO  sis_testing::phase6_web_gui] ==================================================
[2025-11-14T08:13:26Z INFO  sis_testing::phase6_web_gui] Starting Phase 6: Web GUI Management Validation
[2025-11-14T08:13:26Z INFO  sis_testing::phase6_web_gui] ==================================================
[2025-11-14T08:13:26Z INFO  sis_testing::phase6_web_gui::http_server] Running HTTP Server Tests...
[2025-11-14T08:13:26Z INFO  sis_testing::phase6_web_gui::http_server]   Testing server startup...
[2025-11-14T08:13:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl start --port 8080' timeout=30s
[2025-11-14T08:13:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:13:26Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:13:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:13:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:13:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:13:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:13:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:13:56Z INFO  sis_testing::phase6_web_gui::http_server]   Testing health endpoint...
[2025-11-14T08:13:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl status' timeout=30s
[2025-11-14T08:13:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:13:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:13:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:13:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:13:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:13:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:13:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:14:26Z INFO  sis_testing::phase6_web_gui::http_server]   Testing server shutdown...
[2025-11-14T08:14:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl stop' timeout=30s
[2025-11-14T08:14:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:14:26Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:14:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:14:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:14:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:14:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:14:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:14:56Z INFO  sis_testing::phase6_web_gui::http_server] HTTP Server Tests: 0/3 passed (0%)
[2025-11-14T08:14:56Z INFO  sis_testing::phase6_web_gui::websocket] Running WebSocket Tests...
[2025-11-14T08:14:56Z INFO  sis_testing::phase6_web_gui::websocket]   Testing WebSocket connection...
[2025-11-14T08:14:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl ws-status' timeout=30s
[2025-11-14T08:14:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:14:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:14:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:14:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:14:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:14:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:14:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:15:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl status' timeout=30s
[2025-11-14T08:15:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:15:26Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:15:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:15:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:15:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:15:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:15:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:15:56Z WARN  sis_testing::phase6_web_gui::websocket]     ❌ WebSocket connection: FAILED
[2025-11-14T08:15:56Z INFO  sis_testing::phase6_web_gui::websocket]   Testing ping/pong heartbeat...
[2025-11-14T08:15:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl ws-ping' timeout=30s
[2025-11-14T08:15:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:15:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:15:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:15:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:15:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:15:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:15:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:16:26Z INFO  sis_testing::phase6_web_gui::websocket]     ✅ Ping/pong: PASSED
[2025-11-14T08:16:26Z INFO  sis_testing::phase6_web_gui::websocket]   Testing metric subscription...
[2025-11-14T08:16:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl subscribe memory_pressure cpu_usage' timeout=30s
[2025-11-14T08:16:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:16:26Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:16:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:16:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:16:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:16:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:16:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:16:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl ws-status' timeout=30s
[2025-11-14T08:16:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:16:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:16:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:16:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:16:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:16:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:16:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:17:27Z WARN  sis_testing::phase6_web_gui::websocket]     ❌ Metric subscription: FAILED
[2025-11-14T08:17:27Z INFO  sis_testing::phase6_web_gui::websocket] WebSocket Tests: 1/3 passed (33%)
[2025-11-14T08:17:27Z INFO  sis_testing::phase6_web_gui::api_endpoints] Running API Endpoint Tests...
[2025-11-14T08:17:27Z INFO  sis_testing::phase6_web_gui::api_endpoints]   Testing GET /api/metrics...
[2025-11-14T08:17:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl api-test /api/metrics' timeout=30s
[2025-11-14T08:17:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:17:27Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:17:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:17:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:17:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:17:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:17:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:17:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='memctl status' timeout=30s
[2025-11-14T08:17:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:17:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:17:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:17:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:17:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:17:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:17:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:18:27Z WARN  sis_testing::phase6_web_gui::api_endpoints]     ❌ GET /api/metrics: FAILED
[2025-11-14T08:18:27Z INFO  sis_testing::phase6_web_gui::api_endpoints]   Testing POST /api/command...
[2025-11-14T08:18:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl api-exec 'memctl status'' timeout=30s
[2025-11-14T08:18:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:18:27Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:18:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:18:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:18:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:18:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:18:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:18:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='memctl status' timeout=30s
[2025-11-14T08:18:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:18:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:18:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:18:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:18:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:18:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:18:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:19:27Z WARN  sis_testing::phase6_web_gui::api_endpoints]     ❌ POST /api/command: FAILED
[2025-11-14T08:19:27Z INFO  sis_testing::phase6_web_gui::api_endpoints]   Testing GET /api/logs...
[2025-11-14T08:19:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl api-test '/api/logs?lines=100'' timeout=30s
[2025-11-14T08:19:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:19:27Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:19:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:19:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:19:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:19:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:19:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:19:57Z INFO  sis_testing::phase6_web_gui::api_endpoints]     ✅ GET /api/logs: PASSED
[2025-11-14T08:19:57Z INFO  sis_testing::phase6_web_gui::api_endpoints] API Endpoint Tests: 1/3 passed (33%)
[2025-11-14T08:19:57Z INFO  sis_testing::phase6_web_gui::authentication] Running Authentication Tests...
[2025-11-14T08:19:57Z INFO  sis_testing::phase6_web_gui::authentication]   Testing token authentication...
[2025-11-14T08:19:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl auth-token generate' timeout=30s
[2025-11-14T08:19:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:19:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:19:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:19:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:19:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:19:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:19:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:20:27Z INFO  sis_testing::phase6_web_gui::authentication]     ✅ Token authentication: PASSED
[2025-11-14T08:20:27Z INFO  sis_testing::phase6_web_gui::authentication]   Testing invalid credentials handling...
[2025-11-14T08:20:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl auth-test --token invalid_token' timeout=30s
[2025-11-14T08:20:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:20:27Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:20:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:20:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:20:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:20:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:20:27Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:20:57Z INFO  sis_testing::phase6_web_gui::authentication]     ✅ Invalid credentials: PASSED
[2025-11-14T08:20:57Z INFO  sis_testing::phase6_web_gui::authentication]   Testing session management...
[2025-11-14T08:20:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl session list' timeout=30s
[2025-11-14T08:20:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:20:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:20:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:20:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:20:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:20:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:20:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:21:28Z INFO  sis_testing::phase6_web_gui::authentication]     ✅ Session management: PASSED
[2025-11-14T08:21:28Z INFO  sis_testing::phase6_web_gui::authentication]   Testing authorization...
[2025-11-14T08:21:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl auth-check --role admin' timeout=30s
[2025-11-14T08:21:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:21:28Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:21:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:21:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:21:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:21:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:21:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:21:58Z INFO  sis_testing::phase6_web_gui::authentication]     ✅ Authorization: PASSED
[2025-11-14T08:21:58Z INFO  sis_testing::phase6_web_gui::authentication] Authentication Tests: 4/4 passed (100%)
[2025-11-14T08:21:58Z INFO  sis_testing::phase6_web_gui::real_time_updates] Running Real-Time Update Tests...
[2025-11-14T08:21:58Z INFO  sis_testing::phase6_web_gui::real_time_updates]   Testing metric streaming...
[2025-11-14T08:21:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl stream start --metrics memory_pressure cpu_usage' timeout=30s
[2025-11-14T08:21:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:21:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:21:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:21:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:21:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:21:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:21:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:22:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl stream status' timeout=30s
[2025-11-14T08:22:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:22:28Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:22:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:22:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:22:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:22:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:22:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:22:58Z WARN  sis_testing::phase6_web_gui::real_time_updates]     ❌ Metric streaming: FAILED
[2025-11-14T08:22:58Z INFO  sis_testing::phase6_web_gui::real_time_updates]   Testing update frequency...
[2025-11-14T08:22:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl stream start --rate 1000' timeout=30s
[2025-11-14T08:22:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:22:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:22:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:22:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:22:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:22:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:22:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:23:30Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl stream stats' timeout=30s
[2025-11-14T08:23:30Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:23:30Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:23:30Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:23:30Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:23:30Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:23:30Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:23:30Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:24:00Z INFO  sis_testing::phase6_web_gui::real_time_updates]     ✅ Update frequency: PASSED
[2025-11-14T08:24:00Z INFO  sis_testing::phase6_web_gui::real_time_updates]   Testing multiple subscribers...
[2025-11-14T08:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl subscribers count' timeout=30s
[2025-11-14T08:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:24:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:24:31Z INFO  sis_testing::phase6_web_gui::real_time_updates]     ✅ Multiple subscribers: PASSED
[2025-11-14T08:24:31Z INFO  sis_testing::phase6_web_gui::real_time_updates]   Testing data format validation...
[2025-11-14T08:24:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl stream sample' timeout=30s
[2025-11-14T08:24:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:24:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:24:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:24:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:24:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:24:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:24:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:25:01Z INFO  sis_testing::phase6_web_gui::real_time_updates]     ✅ Data format: PASSED
[2025-11-14T08:25:01Z INFO  sis_testing::phase6_web_gui::real_time_updates] Real-Time Update Tests: 3/4 passed (75%)
[2025-11-14T08:25:01Z INFO  sis_testing::phase6_web_gui] ==================================================
[2025-11-14T08:25:01Z INFO  sis_testing::phase6_web_gui] Phase 6 Summary:
[2025-11-14T08:25:01Z INFO  sis_testing::phase6_web_gui]   HTTP Server:        ❌ FAILED
[2025-11-14T08:25:01Z INFO  sis_testing::phase6_web_gui]   WebSocket:          ❌ FAILED
[2025-11-14T08:25:01Z INFO  sis_testing::phase6_web_gui]   API Endpoints:      ❌ FAILED
[2025-11-14T08:25:01Z INFO  sis_testing::phase6_web_gui]   Authentication:     ✅ PASSED
[2025-11-14T08:25:01Z INFO  sis_testing::phase6_web_gui]   Real-Time Updates:  ✅ PASSED
[2025-11-14T08:25:01Z INFO  sis_testing::phase6_web_gui]   Overall:            9/17 tests passed (52.9%)
[2025-11-14T08:25:01Z INFO  sis_testing::phase6_web_gui] ==================================================
[2025-11-14T08:25:01Z INFO  sis_testing::phase7_ai_ops] 🚀 Starting Phase 7: AI Operations Platform validation
[2025-11-14T08:25:01Z INFO  sis_testing::phase7_ai_ops::model_lifecycle] Running Model Lifecycle Tests...
[2025-11-14T08:25:01Z INFO  sis_testing::phase7_ai_ops::model_lifecycle]   Testing model registration...
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id test-model-v1 --size 512 --ctx 2048' timeout=30s
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:25:01Z INFO  sis_testing::phase7_ai_ops::shadow_mode] Running Shadow Mode Tests...
[2025-11-14T08:25:01Z INFO  sis_testing::phase7_ai_ops::shadow_mode]   Testing shadow agent deployment...
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-deploy --id shadow-agent-v2 --traffic 0' timeout=30s
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:25:01Z INFO  sis_testing::phase7_ai_ops::otel_exporter] Running OpenTelemetry Exporter Tests...
[2025-11-14T08:25:01Z INFO  sis_testing::phase7_ai_ops::otel_exporter]   Testing OTel initialization...
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl init --endpoint http://localhost:4318' timeout=30s
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:25:01Z INFO  sis_testing::phase7_ai_ops::decision_traces] Running Decision Traces Tests...
[2025-11-14T08:25:01Z INFO  sis_testing::phase7_ai_ops::decision_traces]   Testing decision trace collection...
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl on' timeout=30s
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:25:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:25:31Z INFO  sis_testing::phase7_ai_ops::shadow_mode]   Testing canary traffic routing (10%)...
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-traffic --percent 10' timeout=30s
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 1000' timeout=30s
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:25:31Z INFO  sis_testing::phase7_ai_ops::model_lifecycle]   Testing model hot-swap...
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --id model-v1' timeout=30s
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:25:31Z INFO  sis_testing::phase7_ai_ops::otel_exporter]   Testing span creation...
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl enable-tracing' timeout=30s
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:25:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test prompt'' timeout=30s
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl swap --from model-v1 --to model-v2' timeout=30s
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:26:01Z INFO  sis_testing::phase7_ai_ops::shadow_mode]   Testing A/B comparison...
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-traffic --percent 50' timeout=30s
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl audit last 100' timeout=30s
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:26:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:26:31Z INFO  sis_testing::phase7_ai_ops::model_lifecycle]   Testing model rollback...
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --id model-v2' timeout=30s
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl export-traces' timeout=30s
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-compare' timeout=30s
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:26:31Z INFO  sis_testing::phase7_ai_ops::decision_traces]   Testing decision buffer management...
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl on' timeout=30s
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:26:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:27:01Z INFO  sis_testing::phase7_ai_ops::otel_exporter]   Testing context propagation...
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 10' timeout=30s
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl rollback --to model-v1' timeout=30s
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:27:01Z INFO  sis_testing::phase7_ai_ops::shadow_mode]   Testing shadow promotion...
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-promote' timeout=30s
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 2000' timeout=30s
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:27:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test'' timeout=30s
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:27:31Z INFO  sis_testing::phase7_ai_ops::model_lifecycle]   Testing multi-model management...
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-1 --size 288 --ctx 2048' timeout=30s
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:27:31Z INFO  sis_testing::phase7_ai_ops::shadow_mode] Shadow Mode Tests: 0/4 passed (0%)
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl audit stats' timeout=30s
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:27:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl export-traces' timeout=30s
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-2 --size 320 --ctx 2048' timeout=30s
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:28:01Z INFO  sis_testing::phase7_ai_ops::decision_traces]   Testing decision export...
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl export-decisions --format json --output /tmp/decisions.json' timeout=30s
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:28:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:28:31Z INFO  sis_testing::phase7_ai_ops::otel_exporter]   Testing batch export performance...
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 0'' timeout=30s
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-3 --size 352 --ctx 2048' timeout=30s
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:28:31Z INFO  sis_testing::phase7_ai_ops::decision_traces]   Testing decision replay...
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl export-decisions --format json --output /tmp/decisions.json' timeout=30s
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:28:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-4 --size 384 --ctx 2048' timeout=30s
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 1'' timeout=30s
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl replay-decisions --input /tmp/decisions.json' timeout=30s
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:29:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:29:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-5 --size 416 --ctx 2048' timeout=30s
[2025-11-14T08:29:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:29:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:29:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:29:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:29:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:29:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:29:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:29:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 2'' timeout=30s
[2025-11-14T08:29:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:29:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:29:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:29:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:29:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:29:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:29:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:29:31Z INFO  sis_testing::phase7_ai_ops::decision_traces] Decision Traces Tests: 0/4 passed (0%)
[2025-11-14T08:30:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-6 --size 448 --ctx 2048' timeout=30s
[2025-11-14T08:30:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:30:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 3'' timeout=30s
[2025-11-14T08:30:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:30:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:30:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:30:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:30:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:30:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:30:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:30:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:30:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:30:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:30:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:30:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:30:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:30:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-7 --size 480 --ctx 2048' timeout=30s
[2025-11-14T08:30:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:30:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 4'' timeout=30s
[2025-11-14T08:30:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:30:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:30:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:30:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:30:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:30:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:30:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:30:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:30:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:30:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:30:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:30:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:30:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:31:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 5'' timeout=30s
[2025-11-14T08:31:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:31:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-8 --size 512 --ctx 2048' timeout=30s
[2025-11-14T08:31:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:31:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:31:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:31:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:31:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:31:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:31:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:31:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:31:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:31:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:31:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:31:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:31:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:31:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 6'' timeout=30s
[2025-11-14T08:31:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:31:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-9 --size 544 --ctx 2048' timeout=30s
[2025-11-14T08:31:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:31:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:31:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:31:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:31:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:31:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:31:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:31:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:31:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:31:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:31:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:31:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:31:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:32:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-10 --size 576 --ctx 2048' timeout=30s
[2025-11-14T08:32:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:32:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 7'' timeout=30s
[2025-11-14T08:32:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:32:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:32:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:32:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:32:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:32:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:32:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:32:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:32:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:32:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:32:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:32:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:32:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:32:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl list' timeout=30s
[2025-11-14T08:32:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:32:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 8'' timeout=30s
[2025-11-14T08:32:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:32:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:32:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:32:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:32:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:32:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:32:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:32:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:32:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:32:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:32:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:32:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:32:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:33:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 9'' timeout=30s
[2025-11-14T08:33:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:33:01Z INFO  sis_testing::phase7_ai_ops::model_lifecycle] Model Lifecycle Tests: 0/4 passed (0%)
[2025-11-14T08:33:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:33:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:33:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:33:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:33:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:33:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:33:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 10'' timeout=30s
[2025-11-14T08:33:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:33:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:33:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:33:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:33:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:33:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:33:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:34:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 11'' timeout=30s
[2025-11-14T08:34:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:34:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:34:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:34:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:34:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:34:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:34:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:34:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 12'' timeout=30s
[2025-11-14T08:34:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:34:32Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:34:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:34:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:34:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:34:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:34:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:35:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 13'' timeout=30s
[2025-11-14T08:35:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:35:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:35:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:35:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:35:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:35:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:35:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:35:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 14'' timeout=30s
[2025-11-14T08:35:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:35:32Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:35:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:35:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:35:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:35:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:35:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:36:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 15'' timeout=30s
[2025-11-14T08:36:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:36:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:36:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:36:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:36:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:36:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:36:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:36:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 16'' timeout=30s
[2025-11-14T08:36:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:36:32Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:36:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:36:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:36:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:36:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:36:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:37:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 17'' timeout=30s
[2025-11-14T08:37:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:37:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:37:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:37:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:37:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:37:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:37:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:37:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 18'' timeout=30s
[2025-11-14T08:37:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:37:32Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:37:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:37:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:37:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:37:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:37:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:38:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 19'' timeout=30s
[2025-11-14T08:38:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:38:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:38:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:38:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:38:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:38:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:38:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:38:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 20'' timeout=30s
[2025-11-14T08:38:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:38:32Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:38:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:38:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:38:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:38:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:38:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:39:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 21'' timeout=30s
[2025-11-14T08:39:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:39:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:39:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:39:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:39:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:39:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:39:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:39:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 22'' timeout=30s
[2025-11-14T08:39:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:39:32Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:39:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:39:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:39:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:39:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:39:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:40:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 23'' timeout=30s
[2025-11-14T08:40:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:40:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:40:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:40:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:40:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:40:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:40:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:40:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 24'' timeout=30s
[2025-11-14T08:40:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:40:32Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:40:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:40:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:40:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:40:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:40:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:41:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 25'' timeout=30s
[2025-11-14T08:41:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:41:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:41:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:41:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:41:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:41:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:41:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:41:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 26'' timeout=30s
[2025-11-14T08:41:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:41:32Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:41:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:41:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:41:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:41:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:41:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:42:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 27'' timeout=30s
[2025-11-14T08:42:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:42:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:42:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:42:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:42:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:42:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:42:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:42:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 28'' timeout=30s
[2025-11-14T08:42:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:42:32Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:42:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:42:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:42:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:42:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:42:32Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:43:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 29'' timeout=30s
[2025-11-14T08:43:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:43:03Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:43:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:43:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:43:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:43:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:43:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 30'' timeout=30s
[2025-11-14T08:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:44:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 31'' timeout=30s
[2025-11-14T08:44:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:44:03Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:44:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:44:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:44:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:44:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:44:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:44:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 32'' timeout=30s
[2025-11-14T08:44:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:44:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:44:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:44:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:44:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:44:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:44:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:45:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 33'' timeout=30s
[2025-11-14T08:45:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:45:03Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:45:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:45:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:45:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:45:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:45:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:45:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 34'' timeout=30s
[2025-11-14T08:45:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:45:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:45:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:45:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:45:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:45:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:45:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:46:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 35'' timeout=30s
[2025-11-14T08:46:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:46:03Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:46:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:46:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:46:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:46:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:46:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:46:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 36'' timeout=30s
[2025-11-14T08:46:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:46:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:46:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:46:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:46:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:46:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:46:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:47:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 37'' timeout=30s
[2025-11-14T08:47:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:47:03Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:47:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:47:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:47:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:47:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:47:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:47:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 38'' timeout=30s
[2025-11-14T08:47:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:47:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:47:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:47:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:47:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:47:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:47:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:48:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 39'' timeout=30s
[2025-11-14T08:48:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:48:03Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:48:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:48:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:48:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:48:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:48:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:48:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 40'' timeout=30s
[2025-11-14T08:48:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:48:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:48:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:48:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:48:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:48:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:48:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:49:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 41'' timeout=30s
[2025-11-14T08:49:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:49:03Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:49:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:49:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:49:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:49:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:49:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 42'' timeout=30s
[2025-11-14T08:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:49:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:50:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 43'' timeout=30s
[2025-11-14T08:50:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:50:04Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:50:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:50:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:50:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:50:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:50:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:50:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 44'' timeout=30s
[2025-11-14T08:50:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:50:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:50:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:50:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:50:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:50:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:50:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:51:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 45'' timeout=30s
[2025-11-14T08:51:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:51:04Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:51:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:51:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:51:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:51:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:51:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:51:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 46'' timeout=30s
[2025-11-14T08:51:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:51:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:51:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:51:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:51:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:51:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:51:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:52:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 47'' timeout=30s
[2025-11-14T08:52:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:52:04Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:52:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:52:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:52:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:52:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:52:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:52:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 48'' timeout=30s
[2025-11-14T08:52:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:52:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:52:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:52:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:52:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:52:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:52:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:53:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 49'' timeout=30s
[2025-11-14T08:53:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:53:04Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:53:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:53:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:53:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:53:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:53:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:53:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 50'' timeout=30s
[2025-11-14T08:53:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:53:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:53:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:53:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:53:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:53:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:53:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:54:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 51'' timeout=30s
[2025-11-14T08:54:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:54:04Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:54:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:54:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:54:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:54:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:54:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:54:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 52'' timeout=30s
[2025-11-14T08:54:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:54:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:54:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:54:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:54:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:54:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:54:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:55:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 53'' timeout=30s
[2025-11-14T08:55:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:55:04Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:55:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:55:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:55:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:55:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:55:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:55:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 54'' timeout=30s
[2025-11-14T08:55:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:55:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:55:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:55:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:55:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:55:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:55:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:56:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 55'' timeout=30s
[2025-11-14T08:56:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:56:04Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:56:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:56:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:56:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:56:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:56:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:56:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 56'' timeout=30s
[2025-11-14T08:56:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:56:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:56:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:56:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:56:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:56:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:56:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:57:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 57'' timeout=30s
[2025-11-14T08:57:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:57:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:57:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:57:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:57:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:57:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:57:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:57:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 58'' timeout=30s
[2025-11-14T08:57:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:57:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:57:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:57:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:57:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:57:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:57:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:58:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 59'' timeout=30s
[2025-11-14T08:58:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:58:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:58:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:58:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:58:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:58:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:58:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:58:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 60'' timeout=30s
[2025-11-14T08:58:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:58:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:58:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:58:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:58:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:58:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:58:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:59:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 61'' timeout=30s
[2025-11-14T08:59:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:59:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:59:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:59:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:59:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:59:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:59:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T08:59:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 62'' timeout=30s
[2025-11-14T08:59:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T08:59:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T08:59:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T08:59:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T08:59:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T08:59:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T08:59:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:00:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 63'' timeout=30s
[2025-11-14T09:00:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:00:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:00:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:00:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:00:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:00:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:00:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:00:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 64'' timeout=30s
[2025-11-14T09:00:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:00:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:00:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:00:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:00:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:00:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:00:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:01:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 65'' timeout=30s
[2025-11-14T09:01:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:01:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:01:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:01:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:01:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:01:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:01:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:01:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 66'' timeout=30s
[2025-11-14T09:01:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:01:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:01:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:01:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:01:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:01:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:01:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:02:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 67'' timeout=30s
[2025-11-14T09:02:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:02:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:02:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:02:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:02:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:02:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:02:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:02:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 68'' timeout=30s
[2025-11-14T09:02:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:02:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:02:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:02:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:02:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:02:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:02:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:03:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 69'' timeout=30s
[2025-11-14T09:03:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:03:06Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:03:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:03:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:03:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:03:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:03:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:03:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 70'' timeout=30s
[2025-11-14T09:03:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:03:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:03:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:03:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:03:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:03:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:03:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:04:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 71'' timeout=30s
[2025-11-14T09:04:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:04:06Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:04:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:04:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:04:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:04:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:04:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:04:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 72'' timeout=30s
[2025-11-14T09:04:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:04:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:04:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:04:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:04:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:04:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:04:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:05:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 73'' timeout=30s
[2025-11-14T09:05:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:05:06Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:05:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:05:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:05:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:05:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:05:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:05:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 74'' timeout=30s
[2025-11-14T09:05:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:05:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:05:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:05:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:05:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:05:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:05:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:06:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 75'' timeout=30s
[2025-11-14T09:06:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:06:06Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:06:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:06:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:06:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:06:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:06:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:06:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 76'' timeout=30s
[2025-11-14T09:06:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:06:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:06:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:06:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:06:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:06:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:06:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:07:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 77'' timeout=30s
[2025-11-14T09:07:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:07:06Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:07:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:07:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:07:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:07:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:07:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:07:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 78'' timeout=30s
[2025-11-14T09:07:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:07:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:07:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:07:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:07:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:07:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:07:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:08:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 79'' timeout=30s
[2025-11-14T09:08:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:08:06Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:08:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:08:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:08:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:08:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:08:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:08:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 80'' timeout=30s
[2025-11-14T09:08:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:08:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:08:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:08:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:08:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:08:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:08:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:09:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 81'' timeout=30s
[2025-11-14T09:09:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:09:06Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:09:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:09:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:09:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:09:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:09:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:09:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 82'' timeout=30s
[2025-11-14T09:09:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:09:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:09:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:09:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:09:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:09:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:09:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:10:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 83'' timeout=30s
[2025-11-14T09:10:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:10:06Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:10:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:10:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:10:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:10:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:10:06Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:10:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 84'' timeout=30s
[2025-11-14T09:10:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:10:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:10:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:10:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:10:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:10:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:10:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:11:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 85'' timeout=30s
[2025-11-14T09:11:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:11:07Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:11:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:11:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:11:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:11:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:11:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:11:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 86'' timeout=30s
[2025-11-14T09:11:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:11:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:11:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:11:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:11:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:11:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:11:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:12:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 87'' timeout=30s
[2025-11-14T09:12:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:12:07Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:12:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:12:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:12:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:12:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:12:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:12:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 88'' timeout=30s
[2025-11-14T09:12:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:12:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:12:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:12:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:12:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:12:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:12:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:13:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 89'' timeout=30s
[2025-11-14T09:13:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:13:07Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:13:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:13:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:13:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:13:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:13:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:13:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 90'' timeout=30s
[2025-11-14T09:13:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:13:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:13:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:13:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:13:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:13:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:13:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:14:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 91'' timeout=30s
[2025-11-14T09:14:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:14:07Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:14:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:14:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:14:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:14:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:14:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:14:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 92'' timeout=30s
[2025-11-14T09:14:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:14:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:14:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:14:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:14:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:14:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:14:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:15:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 93'' timeout=30s
[2025-11-14T09:15:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:15:07Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:15:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:15:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:15:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:15:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:15:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:15:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 94'' timeout=30s
[2025-11-14T09:15:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:15:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:15:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:15:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:15:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:15:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:15:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:16:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 95'' timeout=30s
[2025-11-14T09:16:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:16:07Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:16:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:16:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:16:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:16:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:16:07Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:16:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 96'' timeout=30s
[2025-11-14T09:16:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:16:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:16:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:16:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:16:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:16:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:16:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:17:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 97'' timeout=30s
[2025-11-14T09:17:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:17:08Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:17:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:17:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:17:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:17:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:17:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:17:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 98'' timeout=30s
[2025-11-14T09:17:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:17:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:17:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:17:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:17:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:17:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:17:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:18:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 99'' timeout=30s
[2025-11-14T09:18:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:18:08Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:18:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:18:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:18:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:18:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:18:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:18:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl export-traces' timeout=30s
[2025-11-14T09:18:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:18:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:18:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:18:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:18:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:18:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:18:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:19:08Z INFO  sis_testing::phase7_ai_ops::otel_exporter] OTel Exporter Tests: 0/4 passed (0%)
[2025-11-14T09:19:08Z INFO  sis_testing::phase7_ai_ops::integration_tests] Running Phase 7 Integration Tests...
[2025-11-14T09:19:08Z INFO  sis_testing::phase7_ai_ops::integration_tests]   Testing complete AI Ops workflow...
[2025-11-14T09:19:08Z INFO  sis_testing::phase7_ai_ops::integration_tests]     Step 1: Register model-v2
[2025-11-14T09:19:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-v2 --size 1024 --ctx 4096' timeout=30s
[2025-11-14T09:19:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:19:08Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:19:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:19:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:19:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:19:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:19:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:19:38Z INFO  sis_testing::phase7_ai_ops::integration_tests]     Step 2: Deploy shadow agent
[2025-11-14T09:19:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-deploy --id model-v2 --traffic 0' timeout=30s
[2025-11-14T09:19:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:19:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:19:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:19:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:19:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:19:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:19:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:20:08Z INFO  sis_testing::phase7_ai_ops::integration_tests]     Step 3: Enable OpenTelemetry tracing
[2025-11-14T09:20:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl enable-tracing' timeout=30s
[2025-11-14T09:20:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:20:08Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:20:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:20:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:20:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:20:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:20:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:20:38Z INFO  sis_testing::phase7_ai_ops::integration_tests]     Step 4: Canary rollout (10% → 50%)
[2025-11-14T09:20:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-traffic --percent 10' timeout=30s
[2025-11-14T09:20:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:20:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:20:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:20:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:20:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:20:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:20:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:21:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-traffic --percent 50' timeout=30s
[2025-11-14T09:21:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:21:08Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:21:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:21:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:21:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:21:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:21:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:21:38Z INFO  sis_testing::phase7_ai_ops::integration_tests]     Step 5: A/B performance comparison
[2025-11-14T09:21:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-compare' timeout=30s
[2025-11-14T09:21:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:21:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:21:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:21:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:21:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:21:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:21:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:22:08Z INFO  sis_testing::phase7_ai_ops::integration_tests]     Step 6: Shadow promotion/retirement
[2025-11-14T09:22:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-promote' timeout=30s
[2025-11-14T09:22:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:22:08Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:22:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:22:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:22:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:22:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:22:08Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:22:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-retire' timeout=30s
[2025-11-14T09:22:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:22:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:22:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:22:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:22:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:22:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:22:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:23:09Z INFO  sis_testing::phase7_ai_ops::integration_tests]     Step 7: Export observability data
[2025-11-14T09:23:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl export-traces --output /tmp/traces.json' timeout=30s
[2025-11-14T09:23:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:23:09Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:23:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:23:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:23:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:23:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:23:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:23:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl export-decisions --output /tmp/decisions.json' timeout=30s
[2025-11-14T09:23:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:23:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:23:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:23:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:23:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:23:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:23:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:24:09Z WARN  sis_testing::phase7_ai_ops::integration_tests]     ❌ Complete AI Ops workflow: FAILED (11% success)
[2025-11-14T09:24:09Z INFO  sis_testing::phase7_ai_ops::integration_tests] Phase 7 Integration Tests: 0/1 passed
[2025-11-14T09:24:09Z INFO  sis_testing::phase7_ai_ops] ✅ Phase 7 validation complete: 0.0% (0/5 subsystems passed)
[2025-11-14T09:24:09Z INFO  sis_testing::phase8_deterministic] 🚀 Starting Phase 8: Performance Optimization validation
[2025-11-14T09:24:09Z INFO  sis_testing::phase8_deterministic::cbs_edf_scheduler] Running CBS+EDF Scheduler Tests...
[2025-11-14T09:24:09Z INFO  sis_testing::phase8_deterministic::cbs_edf_scheduler]   Testing admission control...
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=30s
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:24:09Z INFO  sis_testing::phase8_deterministic::slab_allocator] Running Slab Allocator Tests...
[2025-11-14T09:24:09Z INFO  sis_testing::phase8_deterministic::slab_allocator]   Testing slab performance benchmarks...
[2025-11-14T09:24:09Z INFO  sis_testing::phase8_deterministic::slab_allocator]     ✅ Slab performance: PASSED
[2025-11-14T09:24:09Z INFO  sis_testing::phase8_deterministic::slab_allocator]   Testing slab vs linked-list comparison...
[2025-11-14T09:24:09Z INFO  sis_testing::phase8_deterministic::slab_allocator]     ✅ Slab comparison: PASSED
[2025-11-14T09:24:09Z INFO  sis_testing::phase8_deterministic::slab_allocator]   Testing slab cache efficiency...
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='memctl slab-stats' timeout=30s
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:24:09Z INFO  sis_testing::phase8_deterministic::adaptive_memory] Running Adaptive Memory Tests...
[2025-11-14T09:24:09Z INFO  sis_testing::phase8_deterministic::adaptive_memory]   Testing strategy switching...
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='memctl strategy status' timeout=30s
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:24:09Z INFO  sis_testing::phase8_deterministic::meta_agent] Running Meta-Agent Tests...
[2025-11-14T09:24:09Z INFO  sis_testing::phase8_deterministic::meta_agent]   Testing meta-agent inference...
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl on' timeout=30s
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:24:09Z INFO  sis_testing::phase8_deterministic::rate_limiting] Running Rate Limiting Tests...
[2025-11-14T09:24:09Z INFO  sis_testing::phase8_deterministic::rate_limiting]   Testing strategy change rate limiting...
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 5000' timeout=30s
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:24:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 10000000 100000000 100000000' timeout=30s
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:24:39Z INFO  sis_testing::phase8_deterministic::slab_allocator]     ✅ Cache efficiency: PASSED
[2025-11-14T09:24:39Z INFO  sis_testing::phase8_deterministic::slab_allocator] Slab Allocator Tests: 3/3 passed
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 1000' timeout=30s
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 1000' timeout=30s
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:24:39Z INFO  sis_testing::phase8_deterministic::rate_limiting]     ✅ Strategy change rate limit: PASSED
[2025-11-14T09:24:39Z INFO  sis_testing::phase8_deterministic::rate_limiting]   Testing meta-agent directive rate limiting...
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl on' timeout=30s
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:24:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 5000' timeout=30s
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:25:09Z INFO  sis_testing::phase8_deterministic::cbs_edf_scheduler]   Testing deadline miss detection...
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 50000000 100000000 100000000' timeout=30s
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='memctl strategy status' timeout=30s
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl audit last 10' timeout=30s
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:25:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:25:39Z INFO  sis_testing::phase8_deterministic::rate_limiting]     ✅ Meta-agent directive rate limit: PASSED
[2025-11-14T09:25:39Z INFO  sis_testing::phase8_deterministic::rate_limiting]   Testing no output flooding...
[2025-11-14T09:25:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 5000' timeout=30s
[2025-11-14T09:25:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:25:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=30s
[2025-11-14T09:25:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:25:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:25:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:25:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:25:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:25:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:25:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:25:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:25:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:25:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:25:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:25:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:25:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:25:40Z WARN  sis_testing::phase8_deterministic::adaptive_memory]     ❌ Strategy switching: FAILED
[2025-11-14T09:25:40Z INFO  sis_testing::phase8_deterministic::adaptive_memory]   Testing directive thresholds...
[2025-11-14T09:25:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl audit last 10' timeout=30s
[2025-11-14T09:25:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:25:40Z INFO  sis_testing::phase8_deterministic::meta_agent]   Testing confidence thresholds...
[2025-11-14T09:25:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl audit last 50' timeout=30s
[2025-11-14T09:25:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:25:40Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:25:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:25:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:25:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:25:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:25:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:25:40Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:25:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:25:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:25:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:25:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:25:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:26:09Z WARN  sis_testing::phase8_deterministic::rate_limiting]     ⚠️  Stress test took 30s (possible hang)
[2025-11-14T09:26:09Z WARN  sis_testing::phase8_deterministic::rate_limiting]     ❌ No output flooding: FAILED
[2025-11-14T09:26:09Z INFO  sis_testing::phase8_deterministic::rate_limiting] Rate Limiting Tests: 2/3 passed
[2025-11-14T09:26:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=30s
[2025-11-14T09:26:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:26:09Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:26:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:26:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:26:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:26:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:26:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:26:10Z WARN  sis_testing::phase8_deterministic::adaptive_memory]     ❌ Directive thresholds: FAILED
[2025-11-14T09:26:10Z INFO  sis_testing::phase8_deterministic::adaptive_memory]   Testing oscillation detection...
[2025-11-14T09:26:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 2000' timeout=30s
[2025-11-14T09:26:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:26:10Z WARN  sis_testing::phase8_deterministic::meta_agent]     ❌ Confidence thresholds: FAILED
[2025-11-14T09:26:10Z INFO  sis_testing::phase8_deterministic::meta_agent]   Testing multi-subsystem directives...
[2025-11-14T09:26:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl audit last 10' timeout=30s
[2025-11-14T09:26:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:26:10Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:26:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:26:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:26:10Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:26:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:26:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:26:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:26:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:26:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:26:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:26:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:26:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:26:39Z INFO  sis_testing::phase8_deterministic::cbs_edf_scheduler]   Testing budget replenishment...
[2025-11-14T09:26:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 10000000 100000000 100000000' timeout=30s
[2025-11-14T09:26:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:26:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:26:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:26:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:26:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:26:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:26:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:26:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='memctl strategy history' timeout=30s
[2025-11-14T09:26:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:26:40Z WARN  sis_testing::phase8_deterministic::meta_agent]     ❌ Multi-subsystem directives: FAILED
[2025-11-14T09:26:40Z INFO  sis_testing::phase8_deterministic::meta_agent]   Testing reward feedback...
[2025-11-14T09:26:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl audit last 50' timeout=30s
[2025-11-14T09:26:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:26:40Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:26:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:26:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:26:40Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:26:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:26:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:26:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:26:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:26:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:26:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:26:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:26:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:27:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=30s
[2025-11-14T09:27:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:27:09Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:27:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:27:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:27:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:27:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:27:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:27:10Z WARN  sis_testing::phase8_deterministic::adaptive_memory]     ❌ Oscillation detection: FAILED
[2025-11-14T09:27:10Z INFO  sis_testing::phase8_deterministic::adaptive_memory]   Testing rate-limited output...
[2025-11-14T09:27:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 1000' timeout=30s
[2025-11-14T09:27:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:27:10Z WARN  sis_testing::phase8_deterministic::meta_agent]     ❌ Reward feedback: FAILED
[2025-11-14T09:27:10Z INFO  sis_testing::phase8_deterministic::meta_agent] Meta-Agent Tests: 0/4 passed
[2025-11-14T09:27:10Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:27:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:27:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:27:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:27:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:27:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:27:39Z INFO  sis_testing::phase8_deterministic::cbs_edf_scheduler]   Testing EDF priority scheduling...
[2025-11-14T09:27:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 3' timeout=30s
[2025-11-14T09:27:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:27:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:27:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:27:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:27:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:27:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:27:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:27:40Z INFO  sis_testing::phase8_deterministic::adaptive_memory]     ✅ Rate-limited output: PASSED
[2025-11-14T09:27:40Z INFO  sis_testing::phase8_deterministic::adaptive_memory] Adaptive Memory Tests: 1/4 passed
[2025-11-14T09:28:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 5000000 50000000 50000000' timeout=30s
[2025-11-14T09:28:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:28:09Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:28:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:28:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:28:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:28:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:28:09Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:28:40Z INFO  sis_testing::phase8_deterministic::cbs_edf_scheduler]   Testing graph integration...
[2025-11-14T09:28:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-14T09:28:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:28:40Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:28:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:28:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:28:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:28:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:28:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:29:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 10000000 100000000 100000000' timeout=30s
[2025-11-14T09:29:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:29:10Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:29:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:29:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:29:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:29:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:29:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:29:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in none --out 0 --prio 10' timeout=30s
[2025-11-14T09:29:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:29:40Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:29:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:29:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:29:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:29:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:29:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:30:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=30s
[2025-11-14T09:30:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:30:10Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:30:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:30:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:30:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:30:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:30:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:30:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=30s
[2025-11-14T09:30:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:30:40Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:30:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:30:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:30:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:30:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:30:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:31:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det off' timeout=30s
[2025-11-14T09:31:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:31:10Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:31:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:31:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:31:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:31:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:31:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:31:40Z WARN  sis_testing::phase8_deterministic::cbs_edf_scheduler]     ❌ Graph integration: FAILED
[2025-11-14T09:31:40Z INFO  sis_testing::phase8_deterministic::cbs_edf_scheduler] CBS+EDF Scheduler Tests: 0/5 passed (0%)
[2025-11-14T09:31:40Z INFO  sis_testing::phase8_deterministic::stress_comparison] Running Stress Comparison Tests...
[2025-11-14T09:31:40Z INFO  sis_testing::phase8_deterministic::stress_comparison]   Testing autonomy OFF baseline...
[2025-11-14T09:31:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl off' timeout=30s
[2025-11-14T09:31:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:31:40Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:31:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:31:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:31:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:31:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:31:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:32:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 5000' timeout=30s
[2025-11-14T09:32:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:32:10Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:32:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:32:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:32:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:32:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:32:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:32:40Z INFO  sis_testing::phase8_deterministic::stress_comparison]   Testing autonomy ON comparison...
[2025-11-14T09:32:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl on' timeout=30s
[2025-11-14T09:32:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:32:40Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:32:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:32:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:32:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:32:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:32:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:33:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 5000' timeout=30s
[2025-11-14T09:33:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:33:10Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:33:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:33:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:33:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:33:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:33:10Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:33:40Z INFO  sis_testing::phase8_deterministic::stress_comparison]   Testing performance delta validation...
[2025-11-14T09:33:40Z INFO  sis_testing::phase8_deterministic::stress_comparison]     ✅ Performance delta: PASSED
[2025-11-14T09:33:40Z INFO  sis_testing::phase8_deterministic::stress_comparison] Stress Comparison Tests: 1/3 passed
[2025-11-14T09:33:40Z INFO  sis_testing::phase8_deterministic] ✅ Phase 8 validation complete: 33.3% (2/6 subsystems passed)
[2025-11-14T09:33:40Z INFO  sis_testing::phase9_agentic] 🚀 Starting Phase 9: Agentic Platform validation
[2025-11-14T09:33:40Z INFO  sis_testing::phase9_agentic::agentsys_protocol_tests] 🧪 Running AgentSys Protocol Tests...
[2025-11-14T09:33:40Z INFO  sis_testing::phase9_agentic::agentsys_protocol_tests]   Testing FS_LIST operation...
[2025-11-14T09:33:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys test-fs-list' timeout=30s
[2025-11-14T09:33:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-14T09:33:40Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-14T09:33:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-14T09:33:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-14T09:33:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-14T09:33:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-14T09:33:40Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-14T09:34:10Z WARN  sis_testing] Phase 9 validation failed: Test execution failed: Command 'agentsys test-fs-list' timed out after 30s. Output: 
[2025-11-14T09:34:10Z INFO  sis_testing::reporting] Generating comprehensive industry-grade validation report
[2025-11-14T09:34:10Z INFO  sis_testing::reporting::analytics] Generating comprehensive analytics report
[2025-11-14T09:34:10Z INFO  sis_testing::reporting] JSON report written to: target/testing/validation_report.json
[2025-11-14T09:34:10Z INFO  sis_testing::reporting] Analytics JSON report written to: target/testing/analytics_report.json
[2025-11-14T09:34:10Z INFO  sis_testing::reporting::visualization] Generating interactive visualization dashboard
[2025-11-14T09:34:10Z INFO  sis_testing::reporting] Interactive dashboard written to: target/testing/interactive_dashboard.html
[2025-11-14T09:34:10Z INFO  sis_testing::reporting] HTML dashboard written to: target/testing/dashboard.html
[2025-11-14T09:34:10Z INFO  sis_testing::reporting] Executive summary written to: target/testing/executive_summary.md
[2025-11-14T09:34:10Z INFO  sis_testing::reporting] Technical report written to: target/testing/technical_report.md
[2025-11-14T09:34:10Z INFO  sis_testing::reporting] Performance charts placeholder written to: target/testing/performance_charts.svg
[2025-11-14T09:34:10Z INFO  sis_testing::reporting] Comprehensive industry-grade report generated in: target/testing
[2025-11-14T09:34:10Z WARN  sis_testing] Cannot shutdown QEMU: Arc has multiple owners
[2025-11-14T09:34:10Z INFO  sis_test_runner] 
[2025-11-14T09:34:10Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-14T09:34:10Z INFO  sis_test_runner] ║          SIS KERNEL COMPREHENSIVE TEST VALIDATION REPORT          ║
[2025-11-14T09:34:10Z INFO  sis_test_runner] ╚═══════════════════════════════════════════════════════════════════╝
[2025-11-14T09:34:10Z INFO  sis_test_runner] 
[2025-11-14T09:34:10Z INFO  sis_test_runner]   Status: █ NEEDS IMPROVEMENT
[2025-11-14T09:34:10Z INFO  sis_test_runner]   Overall Score: 46.1%
[2025-11-14T09:34:10Z INFO  sis_test_runner]   Test Results: 6 PASS / 7 FAIL / 13 TOTAL
[2025-11-14T09:34:10Z INFO  sis_test_runner] 
[2025-11-14T09:34:10Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-14T09:34:10Z INFO  sis_test_runner] │ CORE SYSTEM COVERAGE                                            │
[2025-11-14T09:34:10Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-14T09:34:10Z INFO  sis_test_runner] │  Performance:      100.0%  ███████████████████████████████████
[2025-11-14T09:34:10Z INFO  sis_test_runner] │  Correctness:      100.0%  ███████████████████████████████████
[2025-11-14T09:34:10Z INFO  sis_test_runner] │  Security:         100.0%  ███████████████████████████████████
[2025-11-14T09:34:10Z INFO  sis_test_runner] │  Distributed:      100.0%  ███████████████████████████████████
[2025-11-14T09:34:10Z INFO  sis_test_runner] │  AI Validation:    100.0%  ███████████████████████████████████
[2025-11-14T09:34:10Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-14T09:34:10Z INFO  sis_test_runner] 
[2025-11-14T09:34:10Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-14T09:34:10Z INFO  sis_test_runner] │ PHASE IMPLEMENTATION PROGRESS                                   │
[2025-11-14T09:34:10Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-14T09:34:10Z INFO  sis_test_runner] │  Phase 1 - AI-Native Dataflow:           7.7%  █░░░░░░░░░░░░░░░░░░░░░░
[2025-11-14T09:34:10Z INFO  sis_test_runner] │  Phase 2 - AI Governance:               11.1%  ██░░░░░░░░░░░░░░░░░░░░░
[2025-11-14T09:34:10Z INFO  sis_test_runner] │  Phase 3 - Temporal Isolation:          20.0%  ████░░░░░░░░░░░░░░░░░░░
[2025-11-14T09:34:10Z INFO  sis_test_runner] │  Phase 5 - UX Safety:                   22.2%  █████░░░░░░░░░░░░░░░░░░
[2025-11-14T09:34:10Z INFO  sis_test_runner] │  Phase 6 - Web GUI Management:          52.9%  ████████████░░░░░░░░░░░
[2025-11-14T09:34:10Z INFO  sis_test_runner] │  Phase 7 - AI Operations:                0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-14T09:34:10Z INFO  sis_test_runner] │  Phase 8 - Performance Optimization:    33.3%  ███████░░░░░░░░░░░░░░░░
[2025-11-14T09:34:10Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-14T09:34:10Z INFO  sis_test_runner] 
[2025-11-14T09:34:10Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-14T09:34:10Z INFO  sis_test_runner] │ DETAILED VALIDATION RESULTS                                     │
[2025-11-14T09:34:10Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-14T09:34:10Z INFO  sis_test_runner] │ ✓ PASS
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Test: AI Inference <3000μs (P99)
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Target: 3000μs | Measured: 0.00μs
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Industry Benchmark: TensorFlow Lite: 50-100ms, ONNX: 25-80ms
[2025-11-14T09:34:10Z INFO  sis_test_runner] │
[2025-11-14T09:34:10Z INFO  sis_test_runner] │ ✓ PASS
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Test: Context Switch <50µs (P95)
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Target: 50µs | Measured: 10000ns
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Industry Benchmark: Relaxed for QEMU emulation (scheduler overhead)
[2025-11-14T09:34:10Z INFO  sis_test_runner] │
[2025-11-14T09:34:10Z INFO  sis_test_runner] │ ✓ PASS
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Test: Memory Safety Guaranteed
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Target: 0 violations | Measured: 0 violations in 10000 tests
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Industry Benchmark: C/C++ kernels: Multiple violations expected
[2025-11-14T09:34:10Z INFO  sis_test_runner] │
[2025-11-14T09:34:10Z INFO  sis_test_runner] │ ✓ PASS
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Test: Byzantine Consensus <6ms (100 nodes)
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Target: 6ms | Measured: 5.45ms
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Industry Benchmark: Tendermint: 5-10ms
[2025-11-14T09:34:10Z INFO  sis_test_runner] │
[2025-11-14T09:34:10Z INFO  sis_test_runner] │ ✓ PASS
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Test: Zero Critical Vulnerabilities
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Target: 0 critical | Measured: 0 critical, 0 total
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Industry Benchmark: Industry average: 5.2 critical vulnerabilities
[2025-11-14T09:34:10Z INFO  sis_test_runner] │
[2025-11-14T09:34:10Z INFO  sis_test_runner] │ ✓ PASS
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Test: AI Inference Accuracy >99.9% (Simulated validation)
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Target: 99.9% | Measured: 99.950%
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Industry Benchmark: Simulated validation: 99.9% baseline
[2025-11-14T09:34:10Z INFO  sis_test_runner] │
[2025-11-14T09:34:10Z INFO  sis_test_runner] │ ✗ FAIL
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Test: Phase 1: AI-Native Dataflow
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Target: ≥75% pass rate | Measured: 7.7%
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Industry Benchmark: Industry standard: 60-70% test coverage
[2025-11-14T09:34:10Z INFO  sis_test_runner] │
[2025-11-14T09:34:10Z INFO  sis_test_runner] │ ✗ FAIL
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Test: Phase 2: AI Governance & Safety Policies
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Target: ≥75% pass rate | Measured: 11.1%
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Industry Benchmark: Industry standard: MLOps governance 50-65%
[2025-11-14T09:34:10Z INFO  sis_test_runner] │
[2025-11-14T09:34:10Z INFO  sis_test_runner] │ ✗ FAIL
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Test: Phase 3: Temporal Isolation
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Target: ≥75% pass rate | Measured: 20.0%
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Industry Benchmark: Industry standard: Real-time 70-80%
[2025-11-14T09:34:10Z INFO  sis_test_runner] │
[2025-11-14T09:34:10Z INFO  sis_test_runner] │ ✗ FAIL
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Test: Phase 5: User Experience Safety
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Target: ≥75% pass rate | Measured: 22.2%
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Industry Benchmark: Industry standard: UX safety 55-70%
[2025-11-14T09:34:10Z INFO  sis_test_runner] │
[2025-11-14T09:34:10Z INFO  sis_test_runner] │ ✗ FAIL
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Test: Phase 6: Web GUI Management
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Target: ≥75% pass rate | Measured: 52.9%
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Industry Benchmark: Industry standard: Web UI 65-75%
[2025-11-14T09:34:10Z INFO  sis_test_runner] │
[2025-11-14T09:34:10Z INFO  sis_test_runner] │ ✗ FAIL
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Test: Phase 7: AI Operations Platform
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Target: ≥75% pass rate | Measured: 0.0%
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Industry Benchmark: Industry standard: MLOps 50-70%
[2025-11-14T09:34:10Z INFO  sis_test_runner] │
[2025-11-14T09:34:10Z INFO  sis_test_runner] │ ✗ FAIL
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Test: Phase 8: Performance Optimization
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Target: ≥75% pass rate | Measured: 33.3%
[2025-11-14T09:34:10Z INFO  sis_test_runner] │   Industry Benchmark: Industry standard: Performance opt 60-75%
[2025-11-14T09:34:10Z INFO  sis_test_runner] │
[2025-11-14T09:34:10Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-14T09:34:10Z INFO  sis_test_runner] 
[2025-11-14T09:34:10Z INFO  sis_test_runner] 📊 Reports generated in: target/testing/
[2025-11-14T09:34:10Z INFO  sis_test_runner] 🌐 View dashboard: target/testing/dashboard.html
[2025-11-14T09:34:10Z INFO  sis_test_runner] 
[2025-11-14T09:34:10Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-14T09:34:10Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-14T09:34:10Z INFO  sis_test_runner] ║  ✗ WARNING: SIS Kernel requires improvements before production  ║
[2025-11-14T09:34:10Z INFO  sis_test_runner] ║    readiness (46.1%). Review failed tests above.                ║
[2025-11-14T09:34:10Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-14T09:34:10Z INFO  sis_test_runner] ╚═══════════════════════════════════════════════════════════════════╝
amoljassal@Amols-Mac-mini sis-kernel % >....                                                                                     
    - test_agent (ID=65535, enabled=yes)
  Total operations: 0
sis> agentsys test-fs-list
[AgentSys] Testing FS_LIST on /tmp/
[AgentSys] FS_LIST: /tmp/
[FS] Entries: files/, docs/, test.txt
[AUDIT] agent=65535 op=0x30 result=ALLOW
[AgentSys] Test PASSED
sis> agentsys test-audio-play
[AgentSys] Testing AUDIO_PLAY track=42
[AgentSys] AUDIO_PLAY: track=42
[AUDIO] Playing track (simulated)
[AUDIT] agent=65535 op=0x36 result=ALLOW
[AgentSys] Test PASSED
sis> agentsys audit
[AgentSys] Recent audit records:
[AUDIT] Recent operations:
  agent=65535 op=0x36 ts=88160956
  agent=65535 op=0x30 ts=82601418
  agent=65535 op=0x36 ts=88160956
  agent=65535 op=0x30 ts=82601418
  agent=65535 op=0x36 ts=88160956
  agent=65535 op=0x30 ts=82601418
  agent=65535 op=0x36 ts=88160956
  agent=65535 op=0x30 ts=82601418
  agent=65535 op=0x36 ts=88160956
  agent=65535 op=0x30 ts=82601418
sis> 

