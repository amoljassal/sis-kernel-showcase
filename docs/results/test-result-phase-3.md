amoljassal@Amols-Mac-mini sis-kernel % cargo run -p sis-testing --release -- --phase 3
warning: profile package spec `bootloader` in profile `dev` did not match any packages
warning: profile package spec `bootloader_api` in profile `dev` did not match any packages
    Finished `release` profile [optimized] target(s) in 0.49s
     Running `target/release/sis-test-runner --phase 3`
[2025-11-13T13:01:05Z INFO  sis_test_runner] SIS Kernel Industry-Grade Test Suite
[2025-11-13T13:01:05Z INFO  sis_test_runner] ====================================
[2025-11-13T13:01:05Z INFO  sis_test_runner] Mode: default (single QEMU node, moderate iterations)
[2025-11-13T13:01:05Z INFO  sis_test_runner] Test Configuration:
[2025-11-13T13:01:05Z INFO  sis_test_runner]   QEMU Nodes: 1
[2025-11-13T13:01:05Z INFO  sis_test_runner]   Duration: 600s
[2025-11-13T13:01:05Z INFO  sis_test_runner]   Performance Iterations: 2000
[2025-11-13T13:01:05Z INFO  sis_test_runner]   Statistical Confidence: 99.0%
[2025-11-13T13:01:05Z INFO  sis_test_runner]   Output Directory: target/testing
[2025-11-13T13:01:05Z INFO  sis_test_runner]   Parallel Execution: true
[2025-11-13T13:01:05Z INFO  sis_test_runner] Initializing QEMU runtime for kernel validation...
[2025-11-13T13:01:05Z INFO  sis_testing] Initializing QEMU runtime for comprehensive kernel testing
[2025-11-13T13:01:05Z INFO  sis_testing::qemu_runtime] Building SIS kernel for QEMU testing
[2025-11-13T13:01:05Z INFO  sis_testing::qemu_runtime] Building UEFI bootloader...
[2025-11-13T13:01:05Z INFO  sis_testing::qemu_runtime] Building kernel with features: bringup,graphctl-framed,deterministic,ai-ops,crypto-real
[2025-11-13T13:01:09Z INFO  sis_testing::qemu_runtime] SIS kernel and UEFI bootloader built successfully
[2025-11-13T13:01:09Z INFO  sis_testing::qemu_runtime] Preparing ESP directories for 1 QEMU instances
[2025-11-13T13:01:09Z INFO  sis_testing::qemu_runtime] ESP directories prepared for all instances
[2025-11-13T13:01:09Z INFO  sis_testing::qemu_runtime] Launching QEMU cluster with 1 nodes
[2025-11-13T13:01:09Z INFO  sis_testing::qemu_runtime] Launching QEMU instance 0 on ports 7000/7100/7200
[2025-11-13T13:01:09Z INFO  sis_testing::qemu_runtime] Instance 0 using PTY: /dev/ttys005
[2025-11-13T13:01:09Z INFO  sis_testing::qemu_runtime] Instance 0 launched (serial log: target/testing/serial-node0.log)
[2025-11-13T13:01:12Z INFO  sis_testing::qemu_runtime] All QEMU instances launched successfully
[2025-11-13T13:01:12Z INFO  sis_testing::qemu_runtime] Waiting for instance 0 to boot (timeout: 180s)
[2025-11-13T13:01:12Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-13T13:01:14Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-13T13:01:16Z INFO  sis_testing::qemu_runtime] Instance 0 booted successfully (detected via serial log)
[2025-11-13T13:01:16Z INFO  sis_testing] QEMU runtime initialized with 1 node(s); boot detected via serial log
[2025-11-13T13:01:16Z INFO  sis_test_runner] QEMU runtime initialized successfully - running real kernel tests
[2025-11-13T13:01:16Z INFO  sis_testing] Starting SIS Kernel Comprehensive Validation
[2025-11-13T13:01:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-13T13:01:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] Shell prompt detected after 136 attempts, ready for commands
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer hello world from sis shell --max-tokens 8' timeout=30s
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmjson' timeout=30s
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:01:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:02:13Z WARN  sis_testing] LLM smoke test failed to run llmjson: Test execution failed: Command 'llmjson' timed out after 30s. Output: [QEMU-OUT] METRIC nn_infer_us=18
    [QEMU-OUT] METRIC nn_infer_count=3
    [QEMU-OUT] [{"op":3,"prompt_len":26,"tokens":5,"wcet_cycles":50000,"period_ns":0,"status":5,"ts_ns":34446012000}, {"op":1,"prompt_le
[2025-11-13T13:02:33Z INFO  sis_testing] No METRIC lines found in target/testing/serial-node0.log; falling back to benchmark suite
[2025-11-13T13:02:33Z INFO  sis_testing] Kernel command interface initialized for real AI validation
[2025-11-13T13:02:33Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-13T13:02:33Z INFO  sis_testing] Phase 1-8 test suites initialized successfully
[2025-11-13T13:02:33Z INFO  sis_testing::ai] Starting comprehensive AI inference validation
[2025-11-13T13:02:33Z INFO  sis_testing::ai] Executing REAL Phase 3 AI validation commands in kernel
[2025-11-13T13:02:33Z INFO  sis_testing::kernel_interface] Starting Phase 3 AI validation command suite execution
[2025-11-13T13:02:33Z INFO  sis_testing::kernel_interface] Testing basic command execution with 'help' command
[2025-11-13T13:02:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='help' timeout=30s
[2025-11-13T13:02:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:02:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:02:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:02:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:02:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:02:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:02:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:03:03Z WARN  sis_testing::ai] Real kernel validation failed: Test execution failed: Basic command execution failed: Test execution failed: Command 'help' timed out after 30s. Output: MU-OUT] Available commands:
    [QEMU-OUT]   help     - Show this help message
    [QEMU-OUT]   version  - Show kernel version and build info
    [QEMU-OUT]   echo     - Echo text to output
    [QEMU-OUT]   ls       . Falling back to simulation.
[2025-11-13T13:03:03Z INFO  sis_testing::ai] Using simulated AI validation (no real kernel interface available)
[2025-11-13T13:03:03Z INFO  sis_testing::ai] Testing AI inference accuracy against reference implementations
[2025-11-13T13:03:03Z INFO  sis_testing::ai] AI inference accuracy: 99.9500% (99950/100000 samples)
[2025-11-13T13:03:03Z INFO  sis_testing::ai] Measuring Neural Engine utilization efficiency
[2025-11-13T13:03:03Z INFO  sis_testing::ai] Neural Engine utilization: 95.0%
[2025-11-13T13:03:03Z INFO  sis_testing::ai::benchmark_suite] Starting comprehensive AI inference benchmarks with Phase 3 metrics
[2025-11-13T13:03:03Z INFO  sis_testing::ai::benchmark_suite] Benchmarking AI inference latency
[2025-11-13T13:03:03Z INFO  sis_testing::ai::benchmark_suite] Measuring Neural Engine latency with 2000 iterations
[2025-11-13T13:03:03Z INFO  sis_testing::ai::benchmark_suite] Benchmarking AI inference throughput
[2025-11-13T13:03:03Z INFO  sis_testing::ai::benchmark_suite] Benchmarking memory efficiency
[2025-11-13T13:03:03Z INFO  sis_testing::ai::benchmark_suite] Benchmarking AI accuracy and precision
[2025-11-13T13:03:03Z INFO  sis_testing::ai::benchmark_suite] Benchmarking power efficiency
[2025-11-13T13:03:03Z INFO  sis_testing::ai::benchmark_suite] Benchmarking CBS+EDF deterministic scheduler AI metrics
[2025-11-13T13:03:03Z INFO  sis_testing::ai::benchmark_suite] Benchmarking NPU driver performance metrics
[2025-11-13T13:03:03Z INFO  sis_testing::ai::benchmark_suite] Benchmarking real-time AI inference guarantees
[2025-11-13T13:03:06Z INFO  sis_testing::ai::benchmark_suite] Measuring CPU fallback latency
[2025-11-13T13:03:09Z INFO  sis_testing::ai::benchmark_suite] Benchmarking batch size: 1
[2025-11-13T13:03:09Z INFO  sis_testing::ai::benchmark_suite] Benchmarking batch size: 4
[2025-11-13T13:03:09Z INFO  sis_testing::ai::benchmark_suite] Benchmarking batch size: 8
[2025-11-13T13:03:09Z INFO  sis_testing::ai::benchmark_suite] Benchmarking batch size: 16
[2025-11-13T13:03:10Z INFO  sis_testing::ai::benchmark_suite] Benchmarking batch size: 32
[2025-11-13T13:03:10Z INFO  sis_testing::ai::benchmark_suite] Benchmarking batch size: 64
[2025-11-13T13:03:59Z INFO  sis_testing::performance] Starting comprehensive performance benchmark suite
[2025-11-13T13:03:59Z INFO  sis_testing::performance] Benchmarking AI inference performance
[2025-11-13T13:03:59Z INFO  sis_testing::performance] AI inference benchmark progress: 0/2000
[2025-11-13T13:03:59Z INFO  sis_testing::correctness] Starting comprehensive correctness validation
[2025-11-13T13:03:59Z INFO  sis_testing::correctness] Verifying memory safety properties
[2025-11-13T13:03:59Z INFO  sis_testing::correctness] Memory safety verification completed: 10000/10000 tests passed
[2025-11-13T13:03:59Z INFO  sis_testing::correctness] Running formal verification analysis
[2025-11-13T13:03:59Z INFO  sis_testing::correctness] Formal verification completed: 95.0% coverage
[2025-11-13T13:03:59Z INFO  sis_testing::correctness] Running property-based tests
[2025-11-13T13:03:59Z INFO  sis_testing::correctness] Property-based tests completed: 4999/5000 passed
[2025-11-13T13:03:59Z INFO  sis_testing::distributed] Starting Byzantine consensus validation
[2025-11-13T13:03:59Z INFO  sis_testing::distributed] Measuring consensus latency with 100 nodes
[2025-11-13T13:03:59Z INFO  sis_testing::security] Starting comprehensive security testing
[2025-11-13T13:03:59Z INFO  sis_testing::security] Testing kernel security with 2000 test configurations
[2025-11-13T13:03:59Z INFO  sis_testing::security] Running comprehensive fuzzing campaign
[2025-11-13T13:03:59Z INFO  sis_testing::security::fuzzing] Fuzzing system call interfaces
[2025-11-13T13:03:59Z INFO  sis_testing::security::fuzzing] Fuzzing memory management subsystem
[2025-11-13T13:03:59Z INFO  sis_testing::distributed] Average consensus latency: 5.17ms
[2025-11-13T13:03:59Z INFO  sis_testing::distributed] Testing Byzantine fault tolerance limits
[2025-11-13T13:03:59Z INFO  sis_testing::distributed] Byzantine fault tolerance: 33/100 nodes
[2025-11-13T13:03:59Z INFO  sis_testing::distributed] Measuring consensus success rate
[2025-11-13T13:03:59Z INFO  sis_testing::distributed] Consensus success rate: 99.900%
[2025-11-13T13:03:59Z INFO  sis_testing::distributed] Testing network partition recovery
[2025-11-13T13:03:59Z INFO  sis_testing::distributed] Network partition recovery time: 157.82ms
[2025-11-13T13:03:59Z INFO  sis_testing::distributed] Testing leader election performance
[2025-11-13T13:03:59Z INFO  sis_testing::distributed] Leader election time: 50.47ms
[2025-11-13T13:03:59Z INFO  sis_testing::security::fuzzing] Fuzzing I/O operations and device drivers
[2025-11-13T13:04:00Z INFO  sis_testing::security::fuzzing] Fuzzing network protocol stack
[2025-11-13T13:04:00Z INFO  sis_testing::security] Running vulnerability scans
[2025-11-13T13:04:00Z INFO  sis_testing::security::vulnerability_scanner] Checking for buffer overflow vulnerabilities
[2025-11-13T13:04:00Z INFO  sis_testing::security::vulnerability_scanner] Checking for integer overflow vulnerabilities
[2025-11-13T13:04:00Z INFO  sis_testing::security::vulnerability_scanner] Checking for use-after-free vulnerabilities
[2025-11-13T13:04:00Z INFO  sis_testing::performance] AI inference benchmark progress: 1000/2000
[2025-11-13T13:04:00Z INFO  sis_testing::security::vulnerability_scanner] Checking for double-free vulnerabilities
[2025-11-13T13:04:00Z INFO  sis_testing::security::vulnerability_scanner] Checking for race condition vulnerabilities
[2025-11-13T13:04:00Z INFO  sis_testing::security::vulnerability_scanner] Checking for privilege escalation vulnerabilities
[2025-11-13T13:04:01Z INFO  sis_testing::security::vulnerability_scanner] Checking for timing attack vulnerabilities
[2025-11-13T13:04:01Z INFO  sis_testing::security::vulnerability_scanner] Checking for side-channel vulnerabilities
[2025-11-13T13:04:01Z INFO  sis_testing::security] Running cryptographic validation tests
[2025-11-13T13:04:01Z INFO  sis_testing::security::crypto_validation] Testing randomness quality
[2025-11-13T13:04:01Z INFO  sis_testing::performance] AI inference benchmark completed: 2000 samples
[2025-11-13T13:04:01Z INFO  sis_testing::performance] Benchmarking context switch performance
[2025-11-13T13:04:01Z INFO  sis_testing::performance] Context switch benchmark progress: 0/2000
[2025-11-13T13:04:01Z INFO  sis_testing::performance] Context switch benchmark progress: 1000/2000
[2025-11-13T13:04:01Z INFO  sis_testing::performance] Context switch benchmark completed: 2000 samples
[2025-11-13T13:04:01Z INFO  sis_testing::performance] Benchmarking memory allocation performance
[2025-11-13T13:04:01Z INFO  sis_testing::performance] Memory allocation benchmark progress: 0/2000
[2025-11-13T13:04:01Z INFO  sis_testing::performance] Memory allocation benchmark progress: 1000/2000
[2025-11-13T13:04:01Z INFO  sis_testing::performance] Memory allocation benchmark completed: 2000 samples
[2025-11-13T13:04:01Z INFO  sis_testing::performance] Benchmarking system throughput
[2025-11-13T13:04:01Z INFO  sis_testing::security::crypto_validation] Testing encryption algorithm strength
[2025-11-13T13:04:02Z INFO  sis_testing::security::crypto_validation] Testing key management practices
[2025-11-13T13:04:02Z INFO  sis_testing::security::crypto_validation] Testing hash function security properties
[2025-11-13T13:04:02Z INFO  sis_testing::security::crypto_validation] Testing side-channel attack resistance
[2025-11-13T13:04:03Z INFO  sis_testing::security] Running memory safety analysis
[2025-11-13T13:04:03Z INFO  sis_testing::security::memory_safety] Checking stack overflow protection
[2025-11-13T13:04:03Z INFO  sis_testing::security::memory_safety] Stack protection analysis complete: true
[2025-11-13T13:04:03Z INFO  sis_testing::security::memory_safety] Checking heap overflow protection
[2025-11-13T13:04:03Z INFO  sis_testing::security::memory_safety] Heap protection analysis complete: true
[2025-11-13T13:04:03Z INFO  sis_testing::security::memory_safety] Checking use-after-free detection capabilities
[2025-11-13T13:04:03Z INFO  sis_testing::security::memory_safety] Use-after-free detection: 100.0% success rate
[2025-11-13T13:04:03Z INFO  sis_testing::security::memory_safety] Checking double-free detection capabilities
[2025-11-13T13:04:03Z INFO  sis_testing::security::memory_safety] Double-free detection: 100.0% success rate
[2025-11-13T13:04:03Z INFO  sis_testing::security::memory_safety] Running comprehensive memory leak detection
[2025-11-13T13:04:03Z INFO  sis_testing::security::memory_safety] Checking control flow integrity
[2025-11-13T13:04:03Z INFO  sis_testing::security::memory_safety] Control flow integrity: true
[2025-11-13T13:04:03Z INFO  sis_testing::security::memory_safety] Checking stack canary protection
[2025-11-13T13:04:03Z INFO  sis_testing::security::memory_safety] Stack canary protection: true
[2025-11-13T13:04:03Z INFO  sis_testing::security::memory_safety] Measuring ASLR effectiveness
[2025-11-13T13:04:04Z INFO  sis_testing::security::memory_safety] ASLR effectiveness: 0.88
[2025-11-13T13:04:11Z INFO  sis_testing::performance] Throughput benchmark completed: 18572521.22 ops/sec
[2025-11-13T13:04:13Z INFO  sis_testing] Running Phase 1-8 comprehensive test suites
[2025-11-13T13:04:13Z INFO  sis_testing::phase1_dataflow] ==================================================
[2025-11-13T13:04:13Z INFO  sis_testing::phase1_dataflow] Starting Phase 1: AI-Native Dataflow Validation
[2025-11-13T13:04:13Z INFO  sis_testing::phase1_dataflow] ==================================================
[2025-11-13T13:04:13Z INFO  sis_testing::phase1_dataflow::graph_execution] Running Graph Execution Tests...
[2025-11-13T13:04:13Z INFO  sis_testing::phase1_dataflow::graph_execution]   Testing graph creation...
[2025-11-13T13:04:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-13T13:04:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:04:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:04:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:04:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:04:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:04:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:04:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:04:43Z INFO  sis_testing::phase1_dataflow::graph_execution]   Testing operator addition...
[2025-11-13T13:04:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=30s
[2025-11-13T13:04:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:04:43Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:04:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:04:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:04:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:04:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:04:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:05:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in none --out 0 --prio 10' timeout=30s
[2025-11-13T13:05:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:05:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:05:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:05:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:05:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:05:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:05:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:05:43Z INFO  sis_testing::phase1_dataflow::graph_execution]   Testing graph execution...
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in none --out 0 --prio 10' timeout=30s
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=30s
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:05:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:06:13Z INFO  sis_testing::phase1_dataflow::graph_execution]   Testing graph cleanup...
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 3' timeout=30s
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl destroy' timeout=30s
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:06:13Z WARN  sis_testing::phase1_dataflow::graph_execution]     ❌ Graph cleanup: FAILED
[2025-11-13T13:06:13Z INFO  sis_testing::phase1_dataflow::graph_execution] Graph Execution Tests: 0/4 passed (0%)
[2025-11-13T13:06:13Z INFO  sis_testing::phase1_dataflow::operator_validation] Running Operator Validation Tests...
[2025-11-13T13:06:13Z INFO  sis_testing::phase1_dataflow::operator_validation]   Testing operator types...
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=30s
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:06:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:06:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=30s
[2025-11-13T13:06:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:06:43Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:06:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:06:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:06:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:06:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:06:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:07:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in 0 --out 2 --prio 5' timeout=30s
[2025-11-13T13:07:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:07:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:07:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:07:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:07:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:07:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:07:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in 1 --out none --prio 1' timeout=30s
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:07:44Z WARN  sis_testing::phase1_dataflow::operator_validation]     ❌ Operator types: FAILED
[2025-11-13T13:07:44Z INFO  sis_testing::phase1_dataflow::operator_validation]   Testing operator priorities...
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 0 --prio 10' timeout=30s
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:07:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:08:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in none --out 0 --prio 5' timeout=30s
[2025-11-13T13:08:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:08:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:08:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:08:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:08:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:08:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:08:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:08:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in none --out 0 --prio 15' timeout=30s
[2025-11-13T13:08:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:08:44Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:08:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:08:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:08:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:08:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:08:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:09:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 3 --in none --out 0 --prio 1' timeout=30s
[2025-11-13T13:09:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:09:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:09:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:09:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:09:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:09:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:09:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:09:44Z WARN  sis_testing::phase1_dataflow::operator_validation]     ❌ Operator priorities: FAILED
[2025-11-13T13:09:44Z INFO  sis_testing::phase1_dataflow::operator_validation]   Testing operator connections...
[2025-11-13T13:09:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-13T13:09:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:09:44Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:09:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:09:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:09:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:09:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:09:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:10:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=30s
[2025-11-13T13:10:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:10:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:10:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:10:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:10:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:10:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:10:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:10:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in 0 --out 2 --prio 5' timeout=30s
[2025-11-13T13:10:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:10:44Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:10:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:10:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:10:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:10:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:10:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:10:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:10:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in 1 --out none --prio 1' timeout=30s
[2025-11-13T13:10:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:10:44Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:10:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:10:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:10:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:10:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:10:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:11:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 10' timeout=30s
[2025-11-13T13:11:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:11:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:11:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:11:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:11:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:11:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:11:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:11:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:11:15Z WARN  sis_testing::phase1_dataflow::operator_validation]     ❌ Operator connections: FAILED
[2025-11-13T13:11:15Z INFO  sis_testing::phase1_dataflow::operator_validation] Operator Validation Tests: 0/3 passed (0%)
[2025-11-13T13:11:15Z INFO  sis_testing::phase1_dataflow::channel_throughput] Running Channel Throughput Tests...
[2025-11-13T13:11:15Z INFO  sis_testing::phase1_dataflow::channel_throughput]   Testing basic channel throughput...
[2025-11-13T13:11:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 3' timeout=30s
[2025-11-13T13:11:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:11:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:11:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:11:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:11:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:11:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:11:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:11:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=30s
[2025-11-13T13:11:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:11:45Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:11:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:11:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:11:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:11:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:11:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:12:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in 0 --out 2 --prio 5' timeout=30s
[2025-11-13T13:12:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:12:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:12:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:12:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:12:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:12:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:12:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 50' timeout=30s
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:12:45Z INFO  sis_testing::phase1_dataflow::channel_throughput]     ✅ Basic throughput: PASSED
[2025-11-13T13:12:45Z INFO  sis_testing::phase1_dataflow::channel_throughput]   Testing high volume transfer...
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=30s
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 0 --prio 10' timeout=30s
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in none --out 0 --prio 9' timeout=30s
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:12:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in none --out 0 --prio 8' timeout=30s
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 3 --in none --out 0 --prio 7' timeout=30s
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 4 --in none --out 0 --prio 6' timeout=30s
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:13:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:13:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 500' timeout=30s
[2025-11-13T13:13:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:13:45Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:13:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:13:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:13:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:13:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:13:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:13:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:13:45Z WARN  sis_testing::phase1_dataflow::channel_throughput]     ❌ High volume: FAILED
[2025-11-13T13:13:45Z INFO  sis_testing::phase1_dataflow::channel_throughput]   Testing backpressure handling...
[2025-11-13T13:13:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-13T13:13:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:13:45Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:13:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:13:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:13:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:13:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:13:45Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:14:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=30s
[2025-11-13T13:14:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:14:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:14:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:14:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:14:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:14:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:14:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:14:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:14:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in 0 --out none --prio 1' timeout=30s
[2025-11-13T13:14:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:14:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:14:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:14:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:14:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:14:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:14:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:14:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=30s
[2025-11-13T13:14:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:14:46Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:14:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:14:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:14:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:14:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:14:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:14:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:14:46Z INFO  sis_testing::phase1_dataflow::channel_throughput]     ✅ Backpressure: PASSED
[2025-11-13T13:14:46Z INFO  sis_testing::phase1_dataflow::channel_throughput] Channel Throughput Tests: 2/3 passed (66%)
[2025-11-13T13:14:46Z INFO  sis_testing::phase1_dataflow::tensor_operations] Running Tensor Operations Tests...
[2025-11-13T13:14:46Z INFO  sis_testing::phase1_dataflow::tensor_operations]   Testing tensor creation...
[2025-11-13T13:14:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-13T13:14:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:14:46Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:14:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:14:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:14:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:14:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:14:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=30s
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:15:16Z INFO  sis_testing::phase1_dataflow::tensor_operations]     ✅ Tensor creation: PASSED
[2025-11-13T13:15:16Z INFO  sis_testing::phase1_dataflow::tensor_operations]   Testing tensor transformation...
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=30s
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:15:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in 0 --out 2 --prio 5' timeout=30s
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in 1 --out none --prio 1' timeout=30s
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 20' timeout=30s
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:15:46Z INFO  sis_testing::phase1_dataflow::tensor_operations]     ✅ Tensor transformation: PASSED
[2025-11-13T13:15:46Z INFO  sis_testing::phase1_dataflow::tensor_operations]   Testing tensor data validation...
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=30s
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:15:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 0 --prio 10' timeout=30s
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in none --out 0 --prio 9' timeout=30s
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:15:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:16:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in none --out 0 --prio 8' timeout=30s
[2025-11-13T13:16:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:16:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:16:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:16:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:16:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:16:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:16:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:16:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:16:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 3 --in none --out 0 --prio 7' timeout=30s
[2025-11-13T13:16:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:16:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:16:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:16:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:16:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:16:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:16:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 4 --in none --out 0 --prio 6' timeout=30s
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=30s
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:16:47Z WARN  sis_testing::phase1_dataflow::tensor_operations]     ❌ Tensor validation: FAILED
[2025-11-13T13:16:47Z INFO  sis_testing::phase1_dataflow::tensor_operations] Tensor Operations Tests: 2/3 passed (66%)
[2025-11-13T13:16:47Z INFO  sis_testing::phase1_dataflow] ==================================================
[2025-11-13T13:16:47Z INFO  sis_testing::phase1_dataflow] Phase 1 Summary:
[2025-11-13T13:16:47Z INFO  sis_testing::phase1_dataflow]   Graph Execution:      ❌ FAILED
[2025-11-13T13:16:47Z INFO  sis_testing::phase1_dataflow]   Operator Validation:  ❌ FAILED
[2025-11-13T13:16:47Z INFO  sis_testing::phase1_dataflow]   Channel Throughput:   ✅ PASSED
[2025-11-13T13:16:47Z INFO  sis_testing::phase1_dataflow]   Tensor Operations:    ✅ PASSED
[2025-11-13T13:16:47Z INFO  sis_testing::phase1_dataflow]   Overall:              4/13 tests passed (30.8%)
[2025-11-13T13:16:47Z INFO  sis_testing::phase1_dataflow] ==================================================
[2025-11-13T13:16:47Z INFO  sis_testing::phase2_governance] =================================================
[2025-11-13T13:16:47Z INFO  sis_testing::phase2_governance] Starting Phase 2: AI Governance & Safety Policies
[2025-11-13T13:16:47Z INFO  sis_testing::phase2_governance] =================================================
[2025-11-13T13:16:47Z INFO  sis_testing::phase2_governance::model_governance] Running Model Governance Tests...
[2025-11-13T13:16:47Z INFO  sis_testing::phase2_governance::model_governance]   Testing model registration...
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 7 --ctx 2048 --vocab 50000 --quant int8 --size-bytes 1048576' timeout=30s
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:16:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:17:17Z INFO  sis_testing::phase2_governance::model_governance]   Testing model versioning...
[2025-11-13T13:17:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-13T13:17:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:17:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:17:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:17:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:17:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:17:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:17:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl status' timeout=30s
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:17:47Z INFO  sis_testing::phase2_governance::model_governance]     ✅ Model versioning: PASSED
[2025-11-13T13:17:47Z INFO  sis_testing::phase2_governance::model_governance]   Testing model metadata validation...
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 7 --ctx 512 --vocab 50000 --quant int8 --size-bytes 524288' timeout=30s
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl status' timeout=30s
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:17:47Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:17:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:17:48Z INFO  sis_testing::phase2_governance::model_governance]     ✅ Model metadata: PASSED
[2025-11-13T13:17:48Z INFO  sis_testing::phase2_governance::model_governance] Model Governance Tests: 2/3 passed (66%)
[2025-11-13T13:17:48Z INFO  sis_testing::phase2_governance::policy_enforcement] Running Policy Enforcement Tests...
[2025-11-13T13:17:48Z INFO  sis_testing::phase2_governance::policy_enforcement]   Testing model size limit enforcement...
[2025-11-13T13:17:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 7 --ctx 512 --vocab 50000 --quant int8 --size-bytes 134217728' timeout=30s
[2025-11-13T13:17:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:17:48Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:17:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:17:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:17:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:17:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:17:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:18:18Z INFO  sis_testing::phase2_governance::policy_enforcement]     ✅ Size limit enforcement: PASSED
[2025-11-13T13:18:18Z INFO  sis_testing::phase2_governance::policy_enforcement]   Testing token budget enforcement...
[2025-11-13T13:18:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-13T13:18:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:18:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:18:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:18:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:18:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:18:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:18:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:18:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:18:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl budget --period-ns 1000000000 --max-tokens-per-period 10' timeout=30s
[2025-11-13T13:18:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:18:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:18:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:18:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:18:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:18:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:18:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:18:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test message --max-tokens 5' timeout=30s
[2025-11-13T13:18:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:18:48Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:18:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:18:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:18:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:18:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:18:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:19:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl status' timeout=30s
[2025-11-13T13:19:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:19:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:19:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:19:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:19:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:19:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:19:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:19:48Z WARN  sis_testing::phase2_governance::policy_enforcement]     ❌ Budget enforcement: FAILED
[2025-11-13T13:19:48Z INFO  sis_testing::phase2_governance::policy_enforcement]   Testing rate limiting...
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl budget --period-ns 1000000000 --max-tokens-per-period 20' timeout=30s
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test 0 --max-tokens 3' timeout=30s
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test 1 --max-tokens 3' timeout=30s
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test 2 --max-tokens 3' timeout=30s
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test 3 --max-tokens 3' timeout=30s
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:19:48Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test 4 --max-tokens 3' timeout=30s
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:19:49Z INFO  sis_testing::phase2_governance::policy_enforcement]     ✅ Rate limiting: PASSED
[2025-11-13T13:19:49Z INFO  sis_testing::phase2_governance::policy_enforcement] Policy Enforcement Tests: 2/3 passed (66%)
[2025-11-13T13:19:49Z INFO  sis_testing::phase2_governance::audit_compliance] Running Audit Compliance Tests...
[2025-11-13T13:19:49Z INFO  sis_testing::phase2_governance::audit_compliance]   Testing audit logging...
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test audit message --max-tokens 5' timeout=30s
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:19:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:20:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmjson' timeout=30s
[2025-11-13T13:20:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:20:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:20:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:20:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:20:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:20:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:20:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:20:49Z INFO  sis_testing::phase2_governance::audit_compliance]   Testing compliance tracking...
[2025-11-13T13:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-13T13:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:20:49Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer compliance test 0 --max-tokens 3' timeout=30s
[2025-11-13T13:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:20:49Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:20:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer compliance test 1 --max-tokens 3' timeout=30s
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer compliance test 2 --max-tokens 3' timeout=30s
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl status' timeout=30s
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmjson' timeout=30s
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:21:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:21:49Z WARN  sis_testing::phase2_governance::audit_compliance]     ❌ Compliance tracking: FAILED
[2025-11-13T13:21:49Z INFO  sis_testing::phase2_governance::audit_compliance]   Testing decision traceability...
[2025-11-13T13:21:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-13T13:21:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:21:49Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:21:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:21:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:21:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:21:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:21:49Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:22:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer traceable decision test --max-tokens 8' timeout=30s
[2025-11-13T13:22:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:22:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:22:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:22:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:22:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:22:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:22:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:22:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:22:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmjson' timeout=30s
[2025-11-13T13:22:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:22:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:22:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:22:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:22:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:22:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:22:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:22:50Z INFO  sis_testing::phase2_governance::audit_compliance] Audit Compliance Tests: 0/3 passed (0%)
[2025-11-13T13:22:50Z INFO  sis_testing::phase2_governance] =================================================
[2025-11-13T13:22:50Z INFO  sis_testing::phase2_governance] Phase 2 Summary:
[2025-11-13T13:22:50Z INFO  sis_testing::phase2_governance]   Model Governance:     ✅ PASSED
[2025-11-13T13:22:50Z INFO  sis_testing::phase2_governance]   Policy Enforcement:   ✅ PASSED
[2025-11-13T13:22:50Z INFO  sis_testing::phase2_governance]   Audit & Compliance:   ❌ FAILED
[2025-11-13T13:22:50Z INFO  sis_testing::phase2_governance]   Overall:              4/9 tests passed (44.4%)
[2025-11-13T13:22:50Z INFO  sis_testing::phase2_governance] =================================================
[2025-11-13T13:22:50Z INFO  sis_testing::phase3_temporal] ==================================================
[2025-11-13T13:22:50Z INFO  sis_testing::phase3_temporal] Starting Phase 3: Temporal Isolation Validation
[2025-11-13T13:22:50Z INFO  sis_testing::phase3_temporal] ==================================================
[2025-11-13T13:22:50Z INFO  sis_testing::phase3_temporal::active_isolation] Running Active Isolation Tests...
[2025-11-13T13:22:50Z INFO  sis_testing::phase3_temporal::active_isolation]   Testing temporal isolation verification...
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='rtaivalidation' timeout=30s
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:22:50Z INFO  sis_testing::phase3_temporal::active_isolation]     ✅ Temporal isolation: PASSED
[2025-11-13T13:22:50Z INFO  sis_testing::phase3_temporal::active_isolation]   Testing jitter measurement...
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='rtaivalidation' timeout=30s
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:22:50Z INFO  sis_testing::phase3_temporal::active_isolation]     ✅ Jitter measurement: PASSED
[2025-11-13T13:22:50Z INFO  sis_testing::phase3_temporal::active_isolation]   Testing isolation under load...
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 50' timeout=30s
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='rtaivalidation' timeout=30s
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:22:50Z INFO  sis_testing::phase3_temporal::active_isolation]     ✅ Isolation under load: PASSED
[2025-11-13T13:22:50Z INFO  sis_testing::phase3_temporal::active_isolation] Active Isolation Tests: 3/3 passed (100%)
[2025-11-13T13:22:50Z INFO  sis_testing::phase3_temporal::deadline_validation] Running Deadline Validation Tests...
[2025-11-13T13:22:50Z INFO  sis_testing::phase3_temporal::deadline_validation]   Testing deadline met validation...
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 5000000 10000000 10000000' timeout=30s
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:22:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:23:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=30s
[2025-11-13T13:23:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:23:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:23:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:23:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:23:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:23:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:23:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:23:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=30s
[2025-11-13T13:23:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:23:50Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:23:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:23:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:23:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:23:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:23:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:23:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:23:50Z INFO  sis_testing::phase3_temporal::deadline_validation]     ✅ Deadline met: PASSED
[2025-11-13T13:23:50Z INFO  sis_testing::phase3_temporal::deadline_validation]   Testing deadline miss detection...
[2025-11-13T13:23:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 1000000 2000000 2000000' timeout=30s
[2025-11-13T13:23:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:23:50Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:23:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:23:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:23:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:23:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:23:50Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=30s
[2025-11-13T13:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=30s
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=30s
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:24:21Z WARN  sis_testing::phase3_temporal::deadline_validation]     ❌ Deadline miss detection: FAILED
[2025-11-13T13:24:21Z INFO  sis_testing::phase3_temporal::deadline_validation]   Testing WCET validation...
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 10000000 50000000 50000000' timeout=30s
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:24:51Z INFO  sis_testing::phase3_temporal::deadline_validation]   Testing periodic deadline guarantees...
[2025-11-13T13:24:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 5000000 20000000 20000000' timeout=30s
[2025-11-13T13:24:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:24:51Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:24:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:24:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:24:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:24:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:24:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=30s
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 200' timeout=30s
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=30s
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:25:21Z INFO  sis_testing::phase3_temporal::deadline_validation]     ✅ Periodic deadlines: PASSED
[2025-11-13T13:25:21Z INFO  sis_testing::phase3_temporal::deadline_validation] Deadline Validation Tests: 2/4 passed (50%)
[2025-11-13T13:25:21Z INFO  sis_testing::phase3_temporal::latency_tests] Running Latency Tests...
[2025-11-13T13:25:21Z INFO  sis_testing::phase3_temporal::latency_tests]   Testing baseline latency...
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='rtaivalidation' timeout=30s
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:25:21Z INFO  sis_testing::phase3_temporal::latency_tests]     ✅ Baseline latency: PASSED
[2025-11-13T13:25:21Z INFO  sis_testing::phase3_temporal::latency_tests]   Testing latency under load...
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=30s
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:25:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:25:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=30s
[2025-11-13T13:25:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:25:51Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:25:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:25:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:25:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:25:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:25:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:25:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:25:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='rtaivalidation' timeout=30s
[2025-11-13T13:25:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:25:52Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:25:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:25:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:25:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:25:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:25:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:25:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:25:52Z INFO  sis_testing::phase3_temporal::latency_tests]     ✅ Latency under load: PASSED
[2025-11-13T13:25:52Z INFO  sis_testing::phase3_temporal::latency_tests]   Testing latency stability...
[2025-11-13T13:25:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 3000000 10000000 10000000' timeout=30s
[2025-11-13T13:25:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:25:52Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:25:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:25:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:25:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:25:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:25:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=30s
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=30s
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=30s
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=30s
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:26:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:26:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=30s
[2025-11-13T13:26:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:26:52Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:26:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:26:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:26:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:26:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:26:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:26:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:26:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=30s
[2025-11-13T13:26:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:26:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:26:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:26:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:26:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:26:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:26:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:26:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:26:53Z INFO  sis_testing::phase3_temporal::latency_tests]     ✅ Latency stability: PASSED
[2025-11-13T13:26:53Z INFO  sis_testing::phase3_temporal::latency_tests] Latency Tests: 3/3 passed (100%)
[2025-11-13T13:26:53Z INFO  sis_testing::phase3_temporal] ==================================================
[2025-11-13T13:26:53Z INFO  sis_testing::phase3_temporal] Phase 3 Summary:
[2025-11-13T13:26:53Z INFO  sis_testing::phase3_temporal]   Active Isolation:     ✅ PASSED
[2025-11-13T13:26:53Z INFO  sis_testing::phase3_temporal]   Deadline Validation:  ❌ FAILED
[2025-11-13T13:26:53Z INFO  sis_testing::phase3_temporal]   Latency Tests:        ✅ PASSED
[2025-11-13T13:26:53Z INFO  sis_testing::phase3_temporal]   Overall:              8/10 tests passed (80.0%)
[2025-11-13T13:26:53Z INFO  sis_testing::phase3_temporal] ==================================================
[2025-11-13T13:26:53Z INFO  sis_testing::phase5_ux_safety] =================================================
[2025-11-13T13:26:53Z INFO  sis_testing::phase5_ux_safety] Starting Phase 5: User Experience Safety
[2025-11-13T13:26:53Z INFO  sis_testing::phase5_ux_safety] =================================================
[2025-11-13T13:26:53Z INFO  sis_testing::phase5_ux_safety::safety_controls] Running Safety Controls Tests...
[2025-11-13T13:26:53Z INFO  sis_testing::phase5_ux_safety::safety_controls]   Testing inference guardrails...
[2025-11-13T13:26:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-13T13:26:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:26:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:26:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:26:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:26:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:26:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:26:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:27:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl budget --period-ns 1000000000 --max-tokens-per-period 5' timeout=30s
[2025-11-13T13:27:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:27:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:27:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:27:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:27:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:27:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:27:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:27:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:27:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer safe test --max-tokens 3' timeout=30s
[2025-11-13T13:27:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:27:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:27:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:27:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:27:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:27:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:27:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:27:53Z INFO  sis_testing::phase5_ux_safety::safety_controls]     ✅ Inference guardrails: PASSED
[2025-11-13T13:27:53Z INFO  sis_testing::phase5_ux_safety::safety_controls]   Testing resource protection...
[2025-11-13T13:27:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 70 --ctx 32768 --vocab 100000 --quant int8 --size-bytes 268435456' timeout=30s
[2025-11-13T13:27:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:27:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:27:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:27:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:27:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:27:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:27:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:28:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 7 --ctx 512 --vocab 50000 --quant int8 --size-bytes 524288' timeout=30s
[2025-11-13T13:28:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:28:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:28:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:28:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:28:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:28:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:28:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:28:53Z INFO  sis_testing::phase5_ux_safety::safety_controls]     ✅ Resource protection: PASSED
[2025-11-13T13:28:53Z INFO  sis_testing::phase5_ux_safety::safety_controls]   Testing safety validation...
[2025-11-13T13:28:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-13T13:28:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:28:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:28:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:28:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:28:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:28:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:28:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:29:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer validation test --max-tokens 5' timeout=30s
[2025-11-13T13:29:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:29:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:29:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:29:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:29:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:29:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:29:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:29:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl status' timeout=30s
[2025-11-13T13:29:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:29:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:29:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:29:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:29:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:29:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:29:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:29:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:29:53Z WARN  sis_testing::phase5_ux_safety::safety_controls]     ❌ Safety validation: FAILED
[2025-11-13T13:29:53Z INFO  sis_testing::phase5_ux_safety::safety_controls] Safety Controls Tests: 2/3 passed (66%)
[2025-11-13T13:29:53Z INFO  sis_testing::phase5_ux_safety::explainability] Running Explainability Tests...
[2025-11-13T13:29:53Z INFO  sis_testing::phase5_ux_safety::explainability]   Testing decision transparency...
[2025-11-13T13:29:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-13T13:29:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:29:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:29:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:29:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:29:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:29:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:29:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:30:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer transparency test input --max-tokens 5' timeout=30s
[2025-11-13T13:30:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:30:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:30:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:30:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:30:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:30:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:30:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:30:53Z INFO  sis_testing::phase5_ux_safety::explainability]   Testing model introspection...
[2025-11-13T13:30:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 7 --ctx 2048 --vocab 50000 --quant int8 --size-bytes 1048576' timeout=30s
[2025-11-13T13:30:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:30:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:30:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:30:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:30:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:30:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:30:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:31:23Z INFO  sis_testing::phase5_ux_safety::explainability]   Testing audit accessibility...
[2025-11-13T13:31:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-13T13:31:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:31:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:31:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:31:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:31:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:31:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:31:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:31:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer audit test 0 --max-tokens 3' timeout=30s
[2025-11-13T13:31:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:31:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:31:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:31:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:31:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:31:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:31:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer audit test 1 --max-tokens 3' timeout=30s
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer audit test 2 --max-tokens 3' timeout=30s
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmjson' timeout=30s
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:32:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:32:54Z INFO  sis_testing::phase5_ux_safety::explainability] Explainability Tests: 0/3 passed (0%)
[2025-11-13T13:32:54Z INFO  sis_testing::phase5_ux_safety::user_feedback] Running User Feedback Tests...
[2025-11-13T13:32:54Z INFO  sis_testing::phase5_ux_safety::user_feedback]   Testing error reporting...
[2025-11-13T13:32:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test without model --max-tokens 5' timeout=30s
[2025-11-13T13:32:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:32:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:32:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:32:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:32:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:32:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:32:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:32:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:32:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-13T13:32:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:32:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:32:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:32:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:32:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:32:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:32:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:33:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer valid test --max-tokens 3' timeout=30s
[2025-11-13T13:33:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:33:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:33:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:33:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:33:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:33:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:33:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:33:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:33:24Z INFO  sis_testing::phase5_ux_safety::user_feedback]     ✅ Error reporting: PASSED
[2025-11-13T13:33:24Z INFO  sis_testing::phase5_ux_safety::user_feedback]   Testing status feedback...
[2025-11-13T13:33:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-13T13:33:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:33:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:33:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:33:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:33:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:33:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:33:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:33:54Z INFO  sis_testing::phase5_ux_safety::user_feedback]   Testing operation confirmation...
[2025-11-13T13:33:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=30s
[2025-11-13T13:33:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:33:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:33:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:33:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:33:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:33:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:33:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:34:24Z INFO  sis_testing::phase5_ux_safety::user_feedback] User Feedback Tests: 1/3 passed (33%)
[2025-11-13T13:34:24Z INFO  sis_testing::phase5_ux_safety] =================================================
[2025-11-13T13:34:24Z INFO  sis_testing::phase5_ux_safety] Phase 5 Summary:
[2025-11-13T13:34:24Z INFO  sis_testing::phase5_ux_safety]   Safety Controls:      ✅ PASSED
[2025-11-13T13:34:24Z INFO  sis_testing::phase5_ux_safety]   Explainability:       ❌ FAILED
[2025-11-13T13:34:24Z INFO  sis_testing::phase5_ux_safety]   User Feedback:        ❌ FAILED
[2025-11-13T13:34:24Z INFO  sis_testing::phase5_ux_safety]   Overall:              3/9 tests passed (33.3%)
[2025-11-13T13:34:24Z INFO  sis_testing::phase5_ux_safety] =================================================
[2025-11-13T13:34:24Z INFO  sis_testing::phase6_web_gui] ==================================================
[2025-11-13T13:34:24Z INFO  sis_testing::phase6_web_gui] Starting Phase 6: Web GUI Management Validation
[2025-11-13T13:34:24Z INFO  sis_testing::phase6_web_gui] ==================================================
[2025-11-13T13:34:24Z INFO  sis_testing::phase6_web_gui::http_server] Running HTTP Server Tests...
[2025-11-13T13:34:24Z INFO  sis_testing::phase6_web_gui::http_server]   Testing server startup...
[2025-11-13T13:34:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl start --port 8080' timeout=30s
[2025-11-13T13:34:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:34:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:34:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:34:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:34:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:34:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:34:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:34:54Z INFO  sis_testing::phase6_web_gui::http_server]   Testing health endpoint...
[2025-11-13T13:34:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl status' timeout=30s
[2025-11-13T13:34:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:34:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:34:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:34:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:34:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:34:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:34:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:35:24Z INFO  sis_testing::phase6_web_gui::http_server]   Testing server shutdown...
[2025-11-13T13:35:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl stop' timeout=30s
[2025-11-13T13:35:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:35:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:35:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:35:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:35:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:35:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:35:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:35:54Z INFO  sis_testing::phase6_web_gui::http_server] HTTP Server Tests: 0/3 passed (0%)
[2025-11-13T13:35:54Z INFO  sis_testing::phase6_web_gui::websocket] Running WebSocket Tests...
[2025-11-13T13:35:54Z INFO  sis_testing::phase6_web_gui::websocket]   Testing WebSocket connection...
[2025-11-13T13:35:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl ws-status' timeout=30s
[2025-11-13T13:35:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:35:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:35:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:35:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:35:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:35:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:35:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:36:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl status' timeout=30s
[2025-11-13T13:36:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:36:25Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:36:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:36:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:36:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:36:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:36:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:36:55Z WARN  sis_testing::phase6_web_gui::websocket]     ❌ WebSocket connection: FAILED
[2025-11-13T13:36:55Z INFO  sis_testing::phase6_web_gui::websocket]   Testing ping/pong heartbeat...
[2025-11-13T13:36:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl ws-ping' timeout=30s
[2025-11-13T13:36:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:36:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:36:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:36:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:36:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:36:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:36:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:37:25Z INFO  sis_testing::phase6_web_gui::websocket]     ✅ Ping/pong: PASSED
[2025-11-13T13:37:25Z INFO  sis_testing::phase6_web_gui::websocket]   Testing metric subscription...
[2025-11-13T13:37:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl subscribe memory_pressure cpu_usage' timeout=30s
[2025-11-13T13:37:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:37:25Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:37:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:37:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:37:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:37:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:37:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:37:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl ws-status' timeout=30s
[2025-11-13T13:37:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:37:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:37:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:37:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:37:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:37:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:37:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:38:25Z WARN  sis_testing::phase6_web_gui::websocket]     ❌ Metric subscription: FAILED
[2025-11-13T13:38:25Z INFO  sis_testing::phase6_web_gui::websocket] WebSocket Tests: 1/3 passed (33%)
[2025-11-13T13:38:25Z INFO  sis_testing::phase6_web_gui::api_endpoints] Running API Endpoint Tests...
[2025-11-13T13:38:25Z INFO  sis_testing::phase6_web_gui::api_endpoints]   Testing GET /api/metrics...
[2025-11-13T13:38:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl api-test /api/metrics' timeout=30s
[2025-11-13T13:38:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:38:25Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:38:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:38:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:38:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:38:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:38:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:38:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='memctl status' timeout=30s
[2025-11-13T13:38:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:38:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:38:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:38:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:38:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:38:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:38:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:39:25Z WARN  sis_testing::phase6_web_gui::api_endpoints]     ❌ GET /api/metrics: FAILED
[2025-11-13T13:39:25Z INFO  sis_testing::phase6_web_gui::api_endpoints]   Testing POST /api/command...
[2025-11-13T13:39:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl api-exec 'memctl status'' timeout=30s
[2025-11-13T13:39:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:39:25Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:39:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:39:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:39:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:39:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:39:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:39:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:39:26Z WARN  sis_testing::phase6_web_gui::api_endpoints]     ❌ POST /api/command: FAILED
[2025-11-13T13:39:26Z INFO  sis_testing::phase6_web_gui::api_endpoints]   Testing GET /api/logs...
[2025-11-13T13:39:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl api-test '/api/logs?lines=100'' timeout=30s
[2025-11-13T13:39:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:39:26Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:39:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:39:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:39:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:39:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:39:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:39:56Z INFO  sis_testing::phase6_web_gui::api_endpoints]     ✅ GET /api/logs: PASSED
[2025-11-13T13:39:56Z INFO  sis_testing::phase6_web_gui::api_endpoints] API Endpoint Tests: 1/3 passed (33%)
[2025-11-13T13:39:56Z INFO  sis_testing::phase6_web_gui::authentication] Running Authentication Tests...
[2025-11-13T13:39:56Z INFO  sis_testing::phase6_web_gui::authentication]   Testing token authentication...
[2025-11-13T13:39:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl auth-token generate' timeout=30s
[2025-11-13T13:39:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:39:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:39:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:39:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:39:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:39:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:39:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:40:26Z INFO  sis_testing::phase6_web_gui::authentication]     ✅ Token authentication: PASSED
[2025-11-13T13:40:26Z INFO  sis_testing::phase6_web_gui::authentication]   Testing invalid credentials handling...
[2025-11-13T13:40:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl auth-test --token invalid_token' timeout=30s
[2025-11-13T13:40:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:40:26Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:40:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:40:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:40:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:40:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:40:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:40:56Z INFO  sis_testing::phase6_web_gui::authentication]     ✅ Invalid credentials: PASSED
[2025-11-13T13:40:56Z INFO  sis_testing::phase6_web_gui::authentication]   Testing session management...
[2025-11-13T13:40:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl session list' timeout=30s
[2025-11-13T13:40:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:40:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:40:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:40:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:40:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:40:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:40:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:41:26Z INFO  sis_testing::phase6_web_gui::authentication]     ✅ Session management: PASSED
[2025-11-13T13:41:26Z INFO  sis_testing::phase6_web_gui::authentication]   Testing authorization...
[2025-11-13T13:41:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl auth-check --role admin' timeout=30s
[2025-11-13T13:41:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:41:26Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:41:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:41:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:41:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:41:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:41:26Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:41:56Z INFO  sis_testing::phase6_web_gui::authentication]     ✅ Authorization: PASSED
[2025-11-13T13:41:56Z INFO  sis_testing::phase6_web_gui::authentication] Authentication Tests: 4/4 passed (100%)
[2025-11-13T13:41:56Z INFO  sis_testing::phase6_web_gui::real_time_updates] Running Real-Time Update Tests...
[2025-11-13T13:41:56Z INFO  sis_testing::phase6_web_gui::real_time_updates]   Testing metric streaming...
[2025-11-13T13:41:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl stream start --metrics memory_pressure cpu_usage' timeout=30s
[2025-11-13T13:41:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:41:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:41:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:41:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:41:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:41:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:41:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:41:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:41:56Z WARN  sis_testing::phase6_web_gui::real_time_updates]     ❌ Metric streaming: FAILED
[2025-11-13T13:41:56Z INFO  sis_testing::phase6_web_gui::real_time_updates]   Testing update frequency...
[2025-11-13T13:41:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl stream start --rate 1000' timeout=30s
[2025-11-13T13:41:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:41:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:41:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:41:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:41:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:41:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:41:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:42:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl stream stats' timeout=30s
[2025-11-13T13:42:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:42:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:42:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:42:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:42:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:42:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:42:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:42:59Z INFO  sis_testing::phase6_web_gui::real_time_updates]     ✅ Update frequency: PASSED
[2025-11-13T13:42:59Z INFO  sis_testing::phase6_web_gui::real_time_updates]   Testing multiple subscribers...
[2025-11-13T13:42:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl subscribers count' timeout=30s
[2025-11-13T13:42:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:42:59Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:42:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:42:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:42:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:42:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:42:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:29Z INFO  sis_testing::phase6_web_gui::real_time_updates]     ✅ Multiple subscribers: PASSED
[2025-11-13T13:43:29Z INFO  sis_testing::phase6_web_gui::real_time_updates]   Testing data format validation...
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl stream sample' timeout=30s
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:29Z WARN  sis_testing::phase6_web_gui::real_time_updates]     ❌ Data format: FAILED
[2025-11-13T13:43:29Z INFO  sis_testing::phase6_web_gui::real_time_updates] Real-Time Update Tests: 2/4 passed (50%)
[2025-11-13T13:43:29Z INFO  sis_testing::phase6_web_gui] ==================================================
[2025-11-13T13:43:29Z INFO  sis_testing::phase6_web_gui] Phase 6 Summary:
[2025-11-13T13:43:29Z INFO  sis_testing::phase6_web_gui]   HTTP Server:        ❌ FAILED
[2025-11-13T13:43:29Z INFO  sis_testing::phase6_web_gui]   WebSocket:          ❌ FAILED
[2025-11-13T13:43:29Z INFO  sis_testing::phase6_web_gui]   API Endpoints:      ❌ FAILED
[2025-11-13T13:43:29Z INFO  sis_testing::phase6_web_gui]   Authentication:     ✅ PASSED
[2025-11-13T13:43:29Z INFO  sis_testing::phase6_web_gui]   Real-Time Updates:  ❌ FAILED
[2025-11-13T13:43:29Z INFO  sis_testing::phase6_web_gui]   Overall:            8/17 tests passed (47.1%)
[2025-11-13T13:43:29Z INFO  sis_testing::phase6_web_gui] ==================================================
[2025-11-13T13:43:29Z INFO  sis_testing::phase7_ai_ops] 🚀 Starting Phase 7: AI Operations Platform validation
[2025-11-13T13:43:29Z INFO  sis_testing::phase7_ai_ops::model_lifecycle] Running Model Lifecycle Tests...
[2025-11-13T13:43:29Z INFO  sis_testing::phase7_ai_ops::model_lifecycle]   Testing model registration...
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id test-model-v1 --size 512 --ctx 2048' timeout=30s
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:29Z INFO  sis_testing::phase7_ai_ops::shadow_mode] Running Shadow Mode Tests...
[2025-11-13T13:43:29Z INFO  sis_testing::phase7_ai_ops::shadow_mode]   Testing shadow agent deployment...
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-deploy --id shadow-agent-v2 --traffic 0' timeout=30s
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:29Z INFO  sis_testing::phase7_ai_ops::otel_exporter] Running OpenTelemetry Exporter Tests...
[2025-11-13T13:43:29Z INFO  sis_testing::phase7_ai_ops::otel_exporter]   Testing OTel initialization...
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl init --endpoint http://localhost:4318' timeout=30s
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:29Z INFO  sis_testing::phase7_ai_ops::decision_traces] Running Decision Traces Tests...
[2025-11-13T13:43:29Z INFO  sis_testing::phase7_ai_ops::decision_traces]   Testing decision trace collection...
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl on' timeout=30s
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:29Z WARN  sis_testing::phase7_ai_ops::model_lifecycle]     ⚠️  Registration took 102ms (target: <100ms)
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl list' timeout=30s
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 1000' timeout=30s
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-status' timeout=30s
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:29Z WARN  sis_testing::phase7_ai_ops::otel_exporter]     ❌ OTel initialization: FAILED
[2025-11-13T13:43:29Z INFO  sis_testing::phase7_ai_ops::otel_exporter]   Testing span creation...
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl enable-tracing' timeout=30s
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:29Z WARN  sis_testing::phase7_ai_ops::model_lifecycle]     ⚠️  Registry lookup took 103ms (target: <10ms)
[2025-11-13T13:43:29Z INFO  sis_testing::phase7_ai_ops::model_lifecycle]     ✅ Model registration: PASSED
[2025-11-13T13:43:29Z INFO  sis_testing::phase7_ai_ops::model_lifecycle]   Testing model hot-swap...
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --id model-v1' timeout=30s
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test prompt'' timeout=30s
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:29Z INFO  sis_testing::phase7_ai_ops::shadow_mode]     ✅ Shadow deployment: PASSED
[2025-11-13T13:43:29Z INFO  sis_testing::phase7_ai_ops::shadow_mode]   Testing canary traffic routing (10%)...
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-traffic --percent 10' timeout=30s
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:30Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl audit last 100' timeout=30s
[2025-11-13T13:43:30Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:30Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:30Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:30Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:30Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:30Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:30Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:31Z INFO  sis_testing::phase7_ai_ops::decision_traces]     ✅ Decision trace collection: PASSED
[2025-11-13T13:43:31Z INFO  sis_testing::phase7_ai_ops::decision_traces]   Testing decision buffer management...
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl on' timeout=30s
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl swap --from model-v1 --to model-v2' timeout=30s
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:31Z INFO  sis_testing::phase7_ai_ops::shadow_mode]     ✅ Canary routing (10%): PASSED
[2025-11-13T13:43:31Z INFO  sis_testing::phase7_ai_ops::shadow_mode]   Testing A/B comparison...
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-traffic --percent 50' timeout=30s
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl export-traces' timeout=30s
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 2000' timeout=30s
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:31Z WARN  sis_testing::phase7_ai_ops::model_lifecycle]     ❌ Model hot-swap: FAILED
[2025-11-13T13:43:31Z INFO  sis_testing::phase7_ai_ops::model_lifecycle]   Testing model rollback...
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --id model-v2' timeout=30s
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:31Z INFO  sis_testing::phase7_ai_ops::otel_exporter]     ✅ Span creation: PASSED
[2025-11-13T13:43:31Z INFO  sis_testing::phase7_ai_ops::otel_exporter]   Testing context propagation...
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 10' timeout=30s
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-compare' timeout=30s
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:31Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test'' timeout=30s
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl rollback --to model-v1' timeout=30s
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z WARN  sis_testing::phase7_ai_ops::shadow_mode]     ❌ A/B comparison: FAILED
[2025-11-13T13:43:33Z INFO  sis_testing::phase7_ai_ops::shadow_mode]   Testing shadow promotion...
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-promote' timeout=30s
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl audit stats' timeout=30s
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl export-traces' timeout=30s
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z WARN  sis_testing::phase7_ai_ops::decision_traces]     ❌ Buffer management: FAILED
[2025-11-13T13:43:33Z INFO  sis_testing::phase7_ai_ops::decision_traces]   Testing decision export...
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl export-decisions --format json --output /tmp/decisions.json' timeout=30s
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z INFO  sis_testing::phase7_ai_ops::shadow_mode]     ✅ Shadow promotion: PASSED
[2025-11-13T13:43:33Z INFO  sis_testing::phase7_ai_ops::shadow_mode] Shadow Mode Tests: 3/4 passed (75%)
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z WARN  sis_testing::phase7_ai_ops::model_lifecycle]     ❌ Model rollback: FAILED
[2025-11-13T13:43:33Z INFO  sis_testing::phase7_ai_ops::model_lifecycle]   Testing multi-model management...
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-1 --size 288 --ctx 2048' timeout=30s
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-2 --size 320 --ctx 2048' timeout=30s
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z WARN  sis_testing::phase7_ai_ops::otel_exporter]     ❌ Context propagation: FAILED
[2025-11-13T13:43:33Z INFO  sis_testing::phase7_ai_ops::otel_exporter]   Testing batch export performance...
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 0'' timeout=30s
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z INFO  sis_testing::phase7_ai_ops::decision_traces]     ✅ Decision export: PASSED
[2025-11-13T13:43:33Z INFO  sis_testing::phase7_ai_ops::decision_traces]   Testing decision replay...
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl export-decisions --format json --output /tmp/decisions.json' timeout=30s
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl replay-decisions --input /tmp/decisions.json' timeout=30s
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-3 --size 352 --ctx 2048' timeout=30s
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 1'' timeout=30s
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-4 --size 384 --ctx 2048' timeout=30s
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z WARN  sis_testing::phase7_ai_ops::decision_traces]     ❌ Decision replay: FAILED
[2025-11-13T13:43:33Z INFO  sis_testing::phase7_ai_ops::decision_traces] Decision Traces Tests: 2/4 passed (50%)
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-5 --size 416 --ctx 2048' timeout=30s
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-6 --size 448 --ctx 2048' timeout=30s
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-7 --size 480 --ctx 2048' timeout=30s
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:43:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:44:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 2'' timeout=30s
[2025-11-13T13:44:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:44:03Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:44:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:44:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:44:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:44:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:44:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:44:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-8 --size 512 --ctx 2048' timeout=30s
[2025-11-13T13:44:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:44:04Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:44:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:44:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:44:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:44:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:44:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:44:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 3'' timeout=30s
[2025-11-13T13:44:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:44:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:44:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:44:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:44:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:44:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:44:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:44:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-9 --size 544 --ctx 2048' timeout=30s
[2025-11-13T13:44:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:44:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:44:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:44:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:44:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:44:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:44:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:45:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 4'' timeout=30s
[2025-11-13T13:45:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:45:03Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:45:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:45:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:45:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:45:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:45:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:45:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-10 --size 576 --ctx 2048' timeout=30s
[2025-11-13T13:45:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:45:04Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:45:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:45:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:45:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:45:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:45:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:45:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 5'' timeout=30s
[2025-11-13T13:45:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:45:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:45:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:45:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:45:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:45:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:45:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:45:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl list' timeout=30s
[2025-11-13T13:45:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:45:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:45:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:45:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:45:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:45:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:45:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:45:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-13T13:45:34Z WARN  sis_testing::phase7_ai_ops::model_lifecycle]     ⚠️  List took 104ms for 10 models (target: <50ms)
[2025-11-13T13:45:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl query --id model-5' timeout=30s
[2025-11-13T13:45:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:45:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:45:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:45:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:45:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:45:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:45:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:46:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 6'' timeout=30s
[2025-11-13T13:46:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:46:03Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:46:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:46:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:46:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:46:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:46:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:46:04Z WARN  sis_testing::phase7_ai_ops::model_lifecycle]     ⚠️  Query took 30030ms (target: <10ms)
[2025-11-13T13:46:04Z WARN  sis_testing::phase7_ai_ops::model_lifecycle]     ❌ Multi-model management: FAILED
[2025-11-13T13:46:04Z INFO  sis_testing::phase7_ai_ops::model_lifecycle] Model Lifecycle Tests: 1/4 passed (25%)
[2025-11-13T13:46:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 7'' timeout=30s
[2025-11-13T13:46:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:46:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:46:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:46:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:46:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:46:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:46:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:47:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 8'' timeout=30s
[2025-11-13T13:47:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:47:03Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:47:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:47:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:47:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:47:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:47:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:47:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 9'' timeout=30s
[2025-11-13T13:47:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:47:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:47:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:47:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:47:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:47:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:47:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:48:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 10'' timeout=30s
[2025-11-13T13:48:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:48:03Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:48:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:48:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:48:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:48:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:48:03Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:48:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 11'' timeout=30s
[2025-11-13T13:48:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:48:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:48:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:48:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:48:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:48:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:48:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:49:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 12'' timeout=30s
[2025-11-13T13:49:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:49:04Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:49:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:49:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:49:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:49:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:49:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 13'' timeout=30s
[2025-11-13T13:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:49:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:50:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 14'' timeout=30s
[2025-11-13T13:50:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:50:04Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:50:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:50:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:50:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:50:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:50:04Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-13T13:50:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 15'' timeout=30s
[2025-11-13T13:50:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-13T13:50:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-13T13:50:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-13T13:50:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-13T13:50:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-13T13:50:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-13T13:50:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[