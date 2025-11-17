amoljassal@Amols-Mac-mini sis-kernel % SIS_FEATURES="ai-ops,bringup,crypto-real,decision-traces,default,deterministic,graphctl-framed,llm,model-lifecycle,otel,shadow-mode,agentsys" cargo run -p sis-testing --release -- --phase 1
warning: profile package spec `bootloader` in profile `dev` did not match any packages
warning: profile package spec `bootloader_api` in profile `dev` did not match any packages
    Finished `release` profile [optimized] target(s) in 0.16s
     Running `target/release/sis-test-runner --phase 1`
[2025-11-16T18:46:21Z INFO  sis_test_runner] SIS Kernel Industry-Grade Test Suite
[2025-11-16T18:46:21Z INFO  sis_test_runner] ====================================
[2025-11-16T18:46:21Z INFO  sis_test_runner] Mode: default (single QEMU node, moderate iterations)
[2025-11-16T18:46:21Z INFO  sis_test_runner] Test Configuration:
[2025-11-16T18:46:21Z INFO  sis_test_runner]   QEMU Nodes: 1
[2025-11-16T18:46:21Z INFO  sis_test_runner]   Duration: 600s
[2025-11-16T18:46:21Z INFO  sis_test_runner]   Performance Iterations: 2000
[2025-11-16T18:46:21Z INFO  sis_test_runner]   Statistical Confidence: 99.0%
[2025-11-16T18:46:21Z INFO  sis_test_runner]   Output Directory: target/testing
[2025-11-16T18:46:21Z INFO  sis_test_runner]   Parallel Execution: true
[2025-11-16T18:46:21Z INFO  sis_test_runner] Initializing QEMU runtime for kernel validation...
[2025-11-16T18:46:21Z INFO  sis_testing] Initializing QEMU runtime for comprehensive kernel testing
[2025-11-16T18:46:21Z INFO  sis_testing::qemu_runtime] Building SIS kernel for QEMU testing
[2025-11-16T18:46:21Z INFO  sis_testing::qemu_runtime] Building UEFI bootloader...
[2025-11-16T18:46:21Z INFO  sis_testing::qemu_runtime] Building kernel with features: ai-ops,bringup,crypto-real,decision-traces,default,deterministic,graphctl-framed,llm,model-lifecycle,otel,shadow-mode,agentsys
[2025-11-16T18:46:21Z INFO  sis_testing::qemu_runtime] SIS kernel and UEFI bootloader built successfully
[2025-11-16T18:46:21Z INFO  sis_testing::qemu_runtime] Preparing ESP directories for 1 QEMU instances
[2025-11-16T18:46:21Z INFO  sis_testing::qemu_runtime] ESP directories prepared for all instances
[2025-11-16T18:46:21Z INFO  sis_testing::qemu_runtime] Launching QEMU cluster with 1 nodes
[2025-11-16T18:46:21Z INFO  sis_testing::qemu_runtime] Launching QEMU instance 0 on ports 7000/7100/7200
[2025-11-16T18:46:21Z INFO  sis_testing::qemu_runtime] Instance 0 launched (serial log: target/testing/serial-node0.log)
[2025-11-16T18:46:24Z INFO  sis_testing::qemu_runtime] All QEMU instances launched successfully
[2025-11-16T18:46:24Z INFO  sis_testing::qemu_runtime] Waiting for instance 0 to boot (timeout: 180s)
[2025-11-16T18:46:24Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T18:46:26Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T18:46:28Z INFO  sis_testing::qemu_runtime] Instance 0 booted successfully (detected via serial log)
[2025-11-16T18:46:28Z INFO  sis_testing] QEMU runtime initialized with 1 node(s); boot detected via serial log
[2025-11-16T18:46:28Z INFO  sis_test_runner] QEMU runtime initialized successfully - running real kernel tests
[2025-11-16T18:46:28Z INFO  sis_test_runner] Phase selection detected: running Phase 1 only
[2025-11-16T18:46:28Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-16T18:46:28Z INFO  sis_testing] Phase 1-9 test suites initialized successfully
[2025-11-16T18:46:28Z INFO  sis_testing] Starting single-phase validation for Phase 1
[2025-11-16T18:46:28Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-16T18:46:28Z INFO  sis_testing] Phase 1-9 test suites initialized successfully
[2025-11-16T18:46:28Z INFO  sis_testing::phase1_dataflow] ==================================================
[2025-11-16T18:46:28Z INFO  sis_testing::phase1_dataflow] Starting Phase 1: AI-Native Dataflow Validation
[2025-11-16T18:46:28Z INFO  sis_testing::phase1_dataflow] ==================================================
[2025-11-16T18:46:28Z INFO  sis_testing::phase1_dataflow::graph_execution] Running Graph Execution Tests...
[2025-11-16T18:46:28Z INFO  sis_testing::phase1_dataflow::graph_execution]   Testing graph creation...
[2025-11-16T18:46:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=45s
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:53Z INFO  sis_testing::phase1_dataflow::graph_execution]     ✅ Graph creation: PASSED
[2025-11-16T18:46:53Z INFO  sis_testing::phase1_dataflow::graph_execution]   Testing operator addition...
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=45s
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in none --out 0 --prio 10' timeout=45s
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:53Z INFO  sis_testing::phase1_dataflow::graph_execution]     ✅ Operator addition: PASSED
[2025-11-16T18:46:53Z INFO  sis_testing::phase1_dataflow::graph_execution]   Testing graph execution...
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=45s
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:53Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in none --out 0 --prio 10' timeout=45s
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=45s
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:54Z INFO  sis_testing::phase1_dataflow::graph_execution]     ✅ Graph execution: PASSED
[2025-11-16T18:46:54Z INFO  sis_testing::phase1_dataflow::graph_execution]   Testing graph cleanup...
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 3' timeout=45s
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl destroy' timeout=45s
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:54Z INFO  sis_testing::phase1_dataflow::graph_execution]     ✅ Graph cleanup: PASSED
[2025-11-16T18:46:54Z INFO  sis_testing::phase1_dataflow::graph_execution] Graph Execution Tests: 4/4 passed (100%)
[2025-11-16T18:46:54Z INFO  sis_testing::phase1_dataflow::operator_validation] Running Operator Validation Tests...
[2025-11-16T18:46:54Z INFO  sis_testing::phase1_dataflow::operator_validation]   Testing operator types...
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=45s
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=45s
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in 0 --out 2 --prio 5' timeout=45s
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in 1 --out none --prio 1' timeout=45s
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:54Z INFO  sis_testing::phase1_dataflow::operator_validation]     ✅ Operator types: PASSED
[2025-11-16T18:46:54Z INFO  sis_testing::phase1_dataflow::operator_validation]   Testing operator priorities...
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=45s
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:54Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 0 --prio 10' timeout=45s
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in none --out 0 --prio 5' timeout=45s
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in none --out 0 --prio 15' timeout=45s
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 3 --in none --out 0 --prio 1' timeout=45s
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:55Z INFO  sis_testing::phase1_dataflow::operator_validation]     ✅ Operator priorities: PASSED
[2025-11-16T18:46:55Z INFO  sis_testing::phase1_dataflow::operator_validation]   Testing operator connections...
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=45s
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=45s
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in 0 --out 2 --prio 5' timeout=45s
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in 1 --out none --prio 1' timeout=45s
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 10' timeout=45s
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:55Z INFO  sis_testing::phase1_dataflow::operator_validation]     ✅ Operator connections: PASSED
[2025-11-16T18:46:55Z INFO  sis_testing::phase1_dataflow::operator_validation] Operator Validation Tests: 3/3 passed (100%)
[2025-11-16T18:46:55Z INFO  sis_testing::phase1_dataflow::channel_throughput] Running Channel Throughput Tests...
[2025-11-16T18:46:55Z INFO  sis_testing::phase1_dataflow::channel_throughput]   Testing basic channel throughput...
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 3' timeout=45s
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=45s
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in 0 --out 2 --prio 5' timeout=45s
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 50' timeout=45s
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:56Z INFO  sis_testing::phase1_dataflow::channel_throughput]     ✅ Basic throughput: PASSED
[2025-11-16T18:46:56Z INFO  sis_testing::phase1_dataflow::channel_throughput]   Testing high volume transfer...
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=45s
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 0 --prio 10' timeout=45s
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in none --out 0 --prio 9' timeout=45s
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in none --out 0 --prio 8' timeout=45s
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 3 --in none --out 0 --prio 7' timeout=45s
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 4 --in none --out 0 --prio 6' timeout=45s
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 500' timeout=45s
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:56Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:57Z INFO  sis_testing::phase1_dataflow::channel_throughput]     ✅ High volume: PASSED
[2025-11-16T18:46:57Z INFO  sis_testing::phase1_dataflow::channel_throughput]   Testing backpressure handling...
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=45s
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=45s
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in 0 --out none --prio 1' timeout=45s
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=45s
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:57Z INFO  sis_testing::phase1_dataflow::channel_throughput]     ✅ Backpressure: PASSED
[2025-11-16T18:46:57Z INFO  sis_testing::phase1_dataflow::channel_throughput] Channel Throughput Tests: 3/3 passed (100%)
[2025-11-16T18:46:57Z INFO  sis_testing::phase1_dataflow::tensor_operations] Running Tensor Operations Tests...
[2025-11-16T18:46:57Z INFO  sis_testing::phase1_dataflow::tensor_operations]   Testing tensor creation...
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=45s
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=45s
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:57Z INFO  sis_testing::phase1_dataflow::tensor_operations]     ✅ Tensor creation: PASSED
[2025-11-16T18:46:57Z INFO  sis_testing::phase1_dataflow::tensor_operations]   Testing tensor transformation...
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=45s
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 1 --prio 10' timeout=45s
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in 0 --out 2 --prio 5' timeout=45s
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in 1 --out none --prio 1' timeout=45s
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 20' timeout=45s
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:58Z INFO  sis_testing::phase1_dataflow::tensor_operations]     ✅ Tensor transformation: PASSED
[2025-11-16T18:46:58Z INFO  sis_testing::phase1_dataflow::tensor_operations]   Testing tensor data validation...
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=45s
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 0 --in none --out 0 --prio 10' timeout=45s
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in none --out 0 --prio 9' timeout=45s
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 2 --in none --out 0 --prio 8' timeout=45s
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 3 --in none --out 0 --prio 7' timeout=45s
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 4 --in none --out 0 --prio 6' timeout=45s
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=45s
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:46:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:46:58Z INFO  sis_testing::phase1_dataflow::tensor_operations]     ✅ Tensor validation: PASSED
[2025-11-16T18:46:58Z INFO  sis_testing::phase1_dataflow::tensor_operations] Tensor Operations Tests: 3/3 passed (100%)
[2025-11-16T18:46:58Z INFO  sis_testing::phase1_dataflow] ==================================================
[2025-11-16T18:46:58Z INFO  sis_testing::phase1_dataflow] Phase 1 Summary:
[2025-11-16T18:46:58Z INFO  sis_testing::phase1_dataflow]   Graph Execution:      ✅ PASSED
[2025-11-16T18:46:58Z INFO  sis_testing::phase1_dataflow]   Operator Validation:  ✅ PASSED
[2025-11-16T18:46:58Z INFO  sis_testing::phase1_dataflow]   Channel Throughput:   ✅ PASSED
[2025-11-16T18:46:58Z INFO  sis_testing::phase1_dataflow]   Tensor Operations:    ✅ PASSED
[2025-11-16T18:46:58Z INFO  sis_testing::phase1_dataflow]   Overall:              13/13 tests passed (100.0%)
[2025-11-16T18:46:58Z INFO  sis_testing::phase1_dataflow] ==================================================
[2025-11-16T18:46:58Z INFO  sis_testing::reporting] Generating comprehensive industry-grade validation report
[2025-11-16T18:46:58Z INFO  sis_testing::reporting::analytics] Generating comprehensive analytics report
[2025-11-16T18:46:58Z INFO  sis_testing::reporting] JSON report written to: target/testing/validation_report.json
[2025-11-16T18:46:58Z INFO  sis_testing::reporting] Analytics JSON report written to: target/testing/analytics_report.json
[2025-11-16T18:46:58Z INFO  sis_testing::reporting::visualization] Generating interactive visualization dashboard
[2025-11-16T18:46:58Z INFO  sis_testing::reporting] Interactive dashboard written to: target/testing/interactive_dashboard.html
[2025-11-16T18:46:58Z INFO  sis_testing::reporting] HTML dashboard written to: target/testing/dashboard.html
[2025-11-16T18:46:58Z INFO  sis_testing::reporting] Executive summary written to: target/testing/executive_summary.md
[2025-11-16T18:46:58Z INFO  sis_testing::reporting] Technical report written to: target/testing/technical_report.md
[2025-11-16T18:46:58Z INFO  sis_testing::reporting] Performance charts placeholder written to: target/testing/performance_charts.svg
[2025-11-16T18:46:58Z INFO  sis_testing::reporting] Comprehensive industry-grade report generated in: target/testing
[2025-11-16T18:46:58Z WARN  sis_testing] Cannot shutdown QEMU: Arc has multiple owners
[2025-11-16T18:46:58Z INFO  sis_test_runner] 
[2025-11-16T18:46:58Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-16T18:46:58Z INFO  sis_test_runner] ║          SIS KERNEL COMPREHENSIVE TEST VALIDATION REPORT          ║
[2025-11-16T18:46:58Z INFO  sis_test_runner] ╚═══════════════════════════════════════════════════════════════════╝
[2025-11-16T18:46:58Z INFO  sis_test_runner] 
[2025-11-16T18:46:58Z INFO  sis_test_runner]   Status: █ PRODUCTION READY
[2025-11-16T18:46:58Z INFO  sis_test_runner]   Overall Score: 100.0%
[2025-11-16T18:46:58Z INFO  sis_test_runner]   Test Results: 1 PASS / 0 FAIL / 1 TOTAL
[2025-11-16T18:46:58Z INFO  sis_test_runner] 
[2025-11-16T18:46:58Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:46:58Z INFO  sis_test_runner] │ CORE SYSTEM COVERAGE                                            │
[2025-11-16T18:46:58Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:46:58Z INFO  sis_test_runner] │  Performance:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:46:58Z INFO  sis_test_runner] │  Correctness:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:46:58Z INFO  sis_test_runner] │  Security:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:46:58Z INFO  sis_test_runner] │  Distributed:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:46:58Z INFO  sis_test_runner] │  AI Validation:      0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:46:58Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:46:58Z INFO  sis_test_runner] 
[2025-11-16T18:46:58Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:46:58Z INFO  sis_test_runner] │ PHASE IMPLEMENTATION PROGRESS                                   │
[2025-11-16T18:46:58Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:46:58Z INFO  sis_test_runner] │  Phase 1 - AI-Native Dataflow:         100.0%  ███████████████████████
[2025-11-16T18:46:58Z INFO  sis_test_runner] │  Phase 2 - AI Governance:                0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:46:58Z INFO  sis_test_runner] │  Phase 3 - Temporal Isolation:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:46:58Z INFO  sis_test_runner] │  Phase 5 - UX Safety:                    0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:46:58Z INFO  sis_test_runner] │  Phase 6 - Web GUI Management:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:46:58Z INFO  sis_test_runner] │  Phase 7 - AI Operations:                0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:46:58Z INFO  sis_test_runner] │  Phase 8 - Performance Optimization:     0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:46:58Z INFO  sis_test_runner] │  Phase 9 - Agentic Platform:             0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:46:58Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:46:58Z INFO  sis_test_runner] 
[2025-11-16T18:46:58Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:46:58Z INFO  sis_test_runner] │ DETAILED VALIDATION RESULTS                                     │
[2025-11-16T18:46:58Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:46:58Z INFO  sis_test_runner] │ ✓ PASS
[2025-11-16T18:46:58Z INFO  sis_test_runner] │   Test: Phase 1: AI-Native Dataflow
[2025-11-16T18:46:58Z INFO  sis_test_runner] │   Target: ≥75% pass rate | Measured: 100.0%
[2025-11-16T18:46:58Z INFO  sis_test_runner] │   Industry Benchmark: Industry standard: 60-70% test coverage
[2025-11-16T18:46:58Z INFO  sis_test_runner] │
[2025-11-16T18:46:58Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:46:58Z INFO  sis_test_runner] 
[2025-11-16T18:46:58Z INFO  sis_test_runner] 📊 Reports generated in: target/testing/
[2025-11-16T18:46:58Z INFO  sis_test_runner] 🌐 View dashboard: target/testing/dashboard.html
[2025-11-16T18:46:58Z INFO  sis_test_runner] 
[2025-11-16T18:46:58Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-16T18:46:58Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-16T18:46:58Z INFO  sis_test_runner] ║  ✓ SUCCESS: SIS Kernel meets industry standards for production   ║
[2025-11-16T18:46:58Z INFO  sis_test_runner] ║    deployment and is ready for production use.                   ║
[2025-11-16T18:46:58Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-16T18:46:58Z INFO  sis_test_runner] ╚═══════════════════════════════════════════════════════════════════╝
amoljassal@Amols-Mac-mini sis-kernel % SIS_FEATURES="ai-ops,bringup,crypto-real,decision-traces,default,deterministic,graphctl-framed,llm,model-lifecycle,otel,shadow-mode,agentsys" cargo run -p sis-testing --release -- --phase 2
warning: profile package spec `bootloader` in profile `dev` did not match any packages
warning: profile package spec `bootloader_api` in profile `dev` did not match any packages
    Finished `release` profile [optimized] target(s) in 0.09s
     Running `target/release/sis-test-runner --phase 2`
[2025-11-16T18:47:04Z INFO  sis_test_runner] SIS Kernel Industry-Grade Test Suite
[2025-11-16T18:47:04Z INFO  sis_test_runner] ====================================
[2025-11-16T18:47:04Z INFO  sis_test_runner] Mode: default (single QEMU node, moderate iterations)
[2025-11-16T18:47:04Z INFO  sis_test_runner] Test Configuration:
[2025-11-16T18:47:04Z INFO  sis_test_runner]   QEMU Nodes: 1
[2025-11-16T18:47:04Z INFO  sis_test_runner]   Duration: 600s
[2025-11-16T18:47:04Z INFO  sis_test_runner]   Performance Iterations: 2000
[2025-11-16T18:47:04Z INFO  sis_test_runner]   Statistical Confidence: 99.0%
[2025-11-16T18:47:04Z INFO  sis_test_runner]   Output Directory: target/testing
[2025-11-16T18:47:04Z INFO  sis_test_runner]   Parallel Execution: true
[2025-11-16T18:47:04Z INFO  sis_test_runner] Initializing QEMU runtime for kernel validation...
[2025-11-16T18:47:04Z INFO  sis_testing] Initializing QEMU runtime for comprehensive kernel testing
[2025-11-16T18:47:04Z INFO  sis_testing::qemu_runtime] Building SIS kernel for QEMU testing
[2025-11-16T18:47:04Z INFO  sis_testing::qemu_runtime] Building UEFI bootloader...
[2025-11-16T18:47:04Z INFO  sis_testing::qemu_runtime] Building kernel with features: ai-ops,bringup,crypto-real,decision-traces,default,deterministic,graphctl-framed,llm,model-lifecycle,otel,shadow-mode,agentsys
[2025-11-16T18:47:04Z INFO  sis_testing::qemu_runtime] SIS kernel and UEFI bootloader built successfully
[2025-11-16T18:47:04Z INFO  sis_testing::qemu_runtime] Preparing ESP directories for 1 QEMU instances
[2025-11-16T18:47:04Z INFO  sis_testing::qemu_runtime] ESP directories prepared for all instances
[2025-11-16T18:47:04Z INFO  sis_testing::qemu_runtime] Launching QEMU cluster with 1 nodes
[2025-11-16T18:47:04Z INFO  sis_testing::qemu_runtime] Launching QEMU instance 0 on ports 7000/7100/7200
[2025-11-16T18:47:04Z INFO  sis_testing::qemu_runtime] Instance 0 launched (serial log: target/testing/serial-node0.log)
[2025-11-16T18:47:07Z INFO  sis_testing::qemu_runtime] All QEMU instances launched successfully
[2025-11-16T18:47:07Z INFO  sis_testing::qemu_runtime] Waiting for instance 0 to boot (timeout: 180s)
[2025-11-16T18:47:07Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T18:47:09Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T18:47:11Z INFO  sis_testing::qemu_runtime] Instance 0 booted successfully (detected via serial log)
[2025-11-16T18:47:11Z INFO  sis_testing] QEMU runtime initialized with 1 node(s); boot detected via serial log
[2025-11-16T18:47:11Z INFO  sis_test_runner] QEMU runtime initialized successfully - running real kernel tests
[2025-11-16T18:47:11Z INFO  sis_test_runner] Phase selection detected: running Phase 2 only
[2025-11-16T18:47:11Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-16T18:47:11Z INFO  sis_testing] Phase 1-9 test suites initialized successfully
[2025-11-16T18:47:11Z INFO  sis_testing] Starting single-phase validation for Phase 2
[2025-11-16T18:47:11Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-16T18:47:11Z INFO  sis_testing] Phase 1-9 test suites initialized successfully
[2025-11-16T18:47:11Z INFO  sis_testing::phase2_governance] =================================================
[2025-11-16T18:47:11Z INFO  sis_testing::phase2_governance] Starting Phase 2: AI Governance & Safety Policies
[2025-11-16T18:47:11Z INFO  sis_testing::phase2_governance] =================================================
[2025-11-16T18:47:11Z INFO  sis_testing::phase2_governance::model_governance] Running Model Governance Tests...
[2025-11-16T18:47:11Z INFO  sis_testing::phase2_governance::model_governance]   Testing model registration...
[2025-11-16T18:47:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 7 --ctx 2048 --vocab 50000 --quant int8 --size-bytes 1048576' timeout=45s
[2025-11-16T18:47:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:37Z INFO  sis_testing::phase2_governance::model_governance]     ✅ Model registration: PASSED
[2025-11-16T18:47:37Z INFO  sis_testing::phase2_governance::model_governance]   Testing model versioning...
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=45s
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl status' timeout=45s
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:37Z INFO  sis_testing::phase2_governance::model_governance]     ✅ Model versioning: PASSED
[2025-11-16T18:47:37Z INFO  sis_testing::phase2_governance::model_governance]   Testing model metadata validation...
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 7 --ctx 512 --vocab 50000 --quant int8 --size-bytes 524288' timeout=45s
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl status' timeout=45s
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:37Z INFO  sis_testing::phase2_governance::model_governance]     ✅ Model metadata: PASSED
[2025-11-16T18:47:37Z INFO  sis_testing::phase2_governance::model_governance] Model Governance Tests: 3/3 passed (100%)
[2025-11-16T18:47:37Z INFO  sis_testing::phase2_governance::policy_enforcement] Running Policy Enforcement Tests...
[2025-11-16T18:47:37Z INFO  sis_testing::phase2_governance::policy_enforcement]   Testing model size limit enforcement...
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 7 --ctx 512 --vocab 50000 --quant int8 --size-bytes 134217728' timeout=45s
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:37Z INFO  sis_testing::phase2_governance::policy_enforcement]     ✅ Size limit enforcement: PASSED
[2025-11-16T18:47:37Z INFO  sis_testing::phase2_governance::policy_enforcement]   Testing token budget enforcement...
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=45s
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl budget --period-ns 1000000000 --max-tokens-per-period 10' timeout=45s
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test message --max-tokens 5' timeout=45s
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl status' timeout=45s
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:37Z INFO  sis_testing::phase2_governance::policy_enforcement]     ✅ Budget enforcement: PASSED
[2025-11-16T18:47:37Z INFO  sis_testing::phase2_governance::policy_enforcement]   Testing rate limiting...
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=45s
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl budget --period-ns 1000000000 --max-tokens-per-period 20' timeout=45s
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test 0 --max-tokens 3' timeout=45s
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test 1 --max-tokens 3' timeout=45s
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test 2 --max-tokens 3' timeout=45s
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test 3 --max-tokens 3' timeout=45s
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test 4 --max-tokens 3' timeout=45s
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:38Z INFO  sis_testing::phase2_governance::policy_enforcement]     ✅ Rate limiting: PASSED
[2025-11-16T18:47:38Z INFO  sis_testing::phase2_governance::policy_enforcement] Policy Enforcement Tests: 3/3 passed (100%)
[2025-11-16T18:47:38Z INFO  sis_testing::phase2_governance::audit_compliance] Running Audit Compliance Tests...
[2025-11-16T18:47:38Z INFO  sis_testing::phase2_governance::audit_compliance]   Testing audit logging...
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=45s
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test audit message --max-tokens 5' timeout=45s
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmjson' timeout=45s
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:38Z INFO  sis_testing::phase2_governance::audit_compliance]     ✅ Audit logging: PASSED
[2025-11-16T18:47:38Z INFO  sis_testing::phase2_governance::audit_compliance]   Testing compliance tracking...
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=45s
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:38Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer compliance test 0 --max-tokens 3' timeout=45s
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer compliance test 1 --max-tokens 3' timeout=45s
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer compliance test 2 --max-tokens 3' timeout=45s
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl status' timeout=45s
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmjson' timeout=45s
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:39Z INFO  sis_testing::phase2_governance::audit_compliance]     ✅ Compliance tracking: PASSED
[2025-11-16T18:47:39Z INFO  sis_testing::phase2_governance::audit_compliance]   Testing decision traceability...
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=45s
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer traceable decision test --max-tokens 8' timeout=45s
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmjson' timeout=45s
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:47:39Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:47:39Z INFO  sis_testing::phase2_governance::audit_compliance]     ✅ Decision traceability: PASSED
[2025-11-16T18:47:39Z INFO  sis_testing::phase2_governance::audit_compliance] Audit Compliance Tests: 3/3 passed (100%)
[2025-11-16T18:47:39Z INFO  sis_testing::phase2_governance] =================================================
[2025-11-16T18:47:39Z INFO  sis_testing::phase2_governance] Phase 2 Summary:
[2025-11-16T18:47:39Z INFO  sis_testing::phase2_governance]   Model Governance:     ✅ PASSED
[2025-11-16T18:47:39Z INFO  sis_testing::phase2_governance]   Policy Enforcement:   ✅ PASSED
[2025-11-16T18:47:39Z INFO  sis_testing::phase2_governance]   Audit & Compliance:   ✅ PASSED
[2025-11-16T18:47:39Z INFO  sis_testing::phase2_governance]   Overall:              9/9 tests passed (100.0%)
[2025-11-16T18:47:39Z INFO  sis_testing::phase2_governance] =================================================
[2025-11-16T18:47:39Z INFO  sis_testing::reporting] Generating comprehensive industry-grade validation report
[2025-11-16T18:47:39Z INFO  sis_testing::reporting::analytics] Generating comprehensive analytics report
[2025-11-16T18:47:39Z INFO  sis_testing::reporting] JSON report written to: target/testing/validation_report.json
[2025-11-16T18:47:39Z INFO  sis_testing::reporting] Analytics JSON report written to: target/testing/analytics_report.json
[2025-11-16T18:47:39Z INFO  sis_testing::reporting::visualization] Generating interactive visualization dashboard
[2025-11-16T18:47:39Z INFO  sis_testing::reporting] Interactive dashboard written to: target/testing/interactive_dashboard.html
[2025-11-16T18:47:39Z INFO  sis_testing::reporting] HTML dashboard written to: target/testing/dashboard.html
[2025-11-16T18:47:39Z INFO  sis_testing::reporting] Executive summary written to: target/testing/executive_summary.md
[2025-11-16T18:47:39Z INFO  sis_testing::reporting] Technical report written to: target/testing/technical_report.md
[2025-11-16T18:47:39Z INFO  sis_testing::reporting] Performance charts placeholder written to: target/testing/performance_charts.svg
[2025-11-16T18:47:39Z INFO  sis_testing::reporting] Comprehensive industry-grade report generated in: target/testing
[2025-11-16T18:47:39Z WARN  sis_testing] Cannot shutdown QEMU: Arc has multiple owners
[2025-11-16T18:47:39Z INFO  sis_test_runner] 
[2025-11-16T18:47:39Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-16T18:47:39Z INFO  sis_test_runner] ║          SIS KERNEL COMPREHENSIVE TEST VALIDATION REPORT          ║
[2025-11-16T18:47:39Z INFO  sis_test_runner] ╚═══════════════════════════════════════════════════════════════════╝
[2025-11-16T18:47:39Z INFO  sis_test_runner] 
[2025-11-16T18:47:39Z INFO  sis_test_runner]   Status: █ PRODUCTION READY
[2025-11-16T18:47:39Z INFO  sis_test_runner]   Overall Score: 100.0%
[2025-11-16T18:47:39Z INFO  sis_test_runner]   Test Results: 1 PASS / 0 FAIL / 1 TOTAL
[2025-11-16T18:47:39Z INFO  sis_test_runner] 
[2025-11-16T18:47:39Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:47:39Z INFO  sis_test_runner] │ CORE SYSTEM COVERAGE                                            │
[2025-11-16T18:47:39Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:47:39Z INFO  sis_test_runner] │  Performance:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:47:39Z INFO  sis_test_runner] │  Correctness:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:47:39Z INFO  sis_test_runner] │  Security:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:47:39Z INFO  sis_test_runner] │  Distributed:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:47:39Z INFO  sis_test_runner] │  AI Validation:      0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:47:39Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:47:39Z INFO  sis_test_runner] 
[2025-11-16T18:47:39Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:47:39Z INFO  sis_test_runner] │ PHASE IMPLEMENTATION PROGRESS                                   │
[2025-11-16T18:47:39Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:47:39Z INFO  sis_test_runner] │  Phase 1 - AI-Native Dataflow:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:47:39Z INFO  sis_test_runner] │  Phase 2 - AI Governance:              100.0%  ███████████████████████
[2025-11-16T18:47:39Z INFO  sis_test_runner] │  Phase 3 - Temporal Isolation:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:47:39Z INFO  sis_test_runner] │  Phase 5 - UX Safety:                    0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:47:39Z INFO  sis_test_runner] │  Phase 6 - Web GUI Management:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:47:39Z INFO  sis_test_runner] │  Phase 7 - AI Operations:                0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:47:39Z INFO  sis_test_runner] │  Phase 8 - Performance Optimization:     0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:47:39Z INFO  sis_test_runner] │  Phase 9 - Agentic Platform:             0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:47:39Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:47:39Z INFO  sis_test_runner] 
[2025-11-16T18:47:39Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:47:39Z INFO  sis_test_runner] │ DETAILED VALIDATION RESULTS                                     │
[2025-11-16T18:47:39Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:47:39Z INFO  sis_test_runner] │ ✓ PASS
[2025-11-16T18:47:39Z INFO  sis_test_runner] │   Test: Phase 2: AI Governance & Safety Policies
[2025-11-16T18:47:39Z INFO  sis_test_runner] │   Target: ≥75% pass rate | Measured: 100.0%
[2025-11-16T18:47:39Z INFO  sis_test_runner] │   Industry Benchmark: Industry standard: MLOps governance 50-65%
[2025-11-16T18:47:39Z INFO  sis_test_runner] │
[2025-11-16T18:47:39Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:47:39Z INFO  sis_test_runner] 
[2025-11-16T18:47:39Z INFO  sis_test_runner] 📊 Reports generated in: target/testing/
[2025-11-16T18:47:39Z INFO  sis_test_runner] 🌐 View dashboard: target/testing/dashboard.html
[2025-11-16T18:47:39Z INFO  sis_test_runner] 
[2025-11-16T18:47:39Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-16T18:47:39Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-16T18:47:39Z INFO  sis_test_runner] ║  ✓ SUCCESS: SIS Kernel meets industry standards for production   ║
[2025-11-16T18:47:39Z INFO  sis_test_runner] ║    deployment and is ready for production use.                   ║
[2025-11-16T18:47:39Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-16T18:47:39Z INFO  sis_test_runner] ╚═══════════════════════════════════════════════════════════════════╝
amoljassal@Amols-Mac-mini sis-kernel % SIS_FEATURES="ai-ops,bringup,crypto-real,decision-traces,default,deterministic,graphctl-framed,llm,model-lifecycle,otel,shadow-mode,agentsys" cargo run -p sis-testing --release -- --phase 3
warning: profile package spec `bootloader` in profile `dev` did not match any packages
warning: profile package spec `bootloader_api` in profile `dev` did not match any packages
    Finished `release` profile [optimized] target(s) in 0.15s
     Running `target/release/sis-test-runner --phase 3`
[2025-11-16T18:47:45Z INFO  sis_test_runner] SIS Kernel Industry-Grade Test Suite
[2025-11-16T18:47:45Z INFO  sis_test_runner] ====================================
[2025-11-16T18:47:45Z INFO  sis_test_runner] Mode: default (single QEMU node, moderate iterations)
[2025-11-16T18:47:45Z INFO  sis_test_runner] Test Configuration:
[2025-11-16T18:47:45Z INFO  sis_test_runner]   QEMU Nodes: 1
[2025-11-16T18:47:45Z INFO  sis_test_runner]   Duration: 600s
[2025-11-16T18:47:45Z INFO  sis_test_runner]   Performance Iterations: 2000
[2025-11-16T18:47:45Z INFO  sis_test_runner]   Statistical Confidence: 99.0%
[2025-11-16T18:47:45Z INFO  sis_test_runner]   Output Directory: target/testing
[2025-11-16T18:47:45Z INFO  sis_test_runner]   Parallel Execution: true
[2025-11-16T18:47:45Z INFO  sis_test_runner] Initializing QEMU runtime for kernel validation...
[2025-11-16T18:47:45Z INFO  sis_testing] Initializing QEMU runtime for comprehensive kernel testing
[2025-11-16T18:47:45Z INFO  sis_testing::qemu_runtime] Building SIS kernel for QEMU testing
[2025-11-16T18:47:45Z INFO  sis_testing::qemu_runtime] Building UEFI bootloader...
[2025-11-16T18:47:45Z INFO  sis_testing::qemu_runtime] Building kernel with features: ai-ops,bringup,crypto-real,decision-traces,default,deterministic,graphctl-framed,llm,model-lifecycle,otel,shadow-mode,agentsys
[2025-11-16T18:47:45Z INFO  sis_testing::qemu_runtime] SIS kernel and UEFI bootloader built successfully
[2025-11-16T18:47:45Z INFO  sis_testing::qemu_runtime] Preparing ESP directories for 1 QEMU instances
[2025-11-16T18:47:45Z INFO  sis_testing::qemu_runtime] ESP directories prepared for all instances
[2025-11-16T18:47:45Z INFO  sis_testing::qemu_runtime] Launching QEMU cluster with 1 nodes
[2025-11-16T18:47:45Z INFO  sis_testing::qemu_runtime] Launching QEMU instance 0 on ports 7000/7100/7200
[2025-11-16T18:47:45Z INFO  sis_testing::qemu_runtime] Instance 0 launched (serial log: target/testing/serial-node0.log)
[2025-11-16T18:47:48Z INFO  sis_testing::qemu_runtime] All QEMU instances launched successfully
[2025-11-16T18:47:48Z INFO  sis_testing::qemu_runtime] Waiting for instance 0 to boot (timeout: 180s)
[2025-11-16T18:47:48Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T18:47:50Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T18:47:52Z INFO  sis_testing::qemu_runtime] Instance 0 booted successfully (detected via serial log)
[2025-11-16T18:47:52Z INFO  sis_testing] QEMU runtime initialized with 1 node(s); boot detected via serial log
[2025-11-16T18:47:52Z INFO  sis_test_runner] QEMU runtime initialized successfully - running real kernel tests
[2025-11-16T18:47:52Z INFO  sis_test_runner] Phase selection detected: running Phase 3 only
[2025-11-16T18:47:52Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-16T18:47:52Z INFO  sis_testing] Phase 1-9 test suites initialized successfully
[2025-11-16T18:47:52Z INFO  sis_testing] Starting single-phase validation for Phase 3
[2025-11-16T18:47:52Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-16T18:47:52Z INFO  sis_testing] Phase 1-9 test suites initialized successfully
[2025-11-16T18:47:52Z INFO  sis_testing::phase3_temporal] ==================================================
[2025-11-16T18:47:52Z INFO  sis_testing::phase3_temporal] Starting Phase 3: Temporal Isolation Validation
[2025-11-16T18:47:52Z INFO  sis_testing::phase3_temporal] ==================================================
[2025-11-16T18:47:52Z INFO  sis_testing::phase3_temporal::active_isolation] Running Active Isolation Tests...
[2025-11-16T18:47:52Z INFO  sis_testing::phase3_temporal::active_isolation]   Testing temporal isolation verification...
[2025-11-16T18:47:52Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='rtaivalidation' timeout=45s
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:34Z INFO  sis_testing::phase3_temporal::active_isolation]     ✅ Temporal isolation: PASSED
[2025-11-16T18:48:34Z INFO  sis_testing::phase3_temporal::active_isolation]   Testing jitter measurement...
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='rtaivalidation' timeout=45s
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:34Z INFO  sis_testing::phase3_temporal::active_isolation]     ✅ Jitter measurement: PASSED
[2025-11-16T18:48:34Z INFO  sis_testing::phase3_temporal::active_isolation]   Testing isolation under load...
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=45s
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 50' timeout=45s
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='rtaivalidation' timeout=45s
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:34Z INFO  sis_testing::phase3_temporal::active_isolation]     ✅ Isolation under load: PASSED
[2025-11-16T18:48:34Z INFO  sis_testing::phase3_temporal::active_isolation] Active Isolation Tests: 3/3 passed (100%)
[2025-11-16T18:48:34Z INFO  sis_testing::phase3_temporal::deadline_validation] Running Deadline Validation Tests...
[2025-11-16T18:48:34Z INFO  sis_testing::phase3_temporal::deadline_validation]   Testing deadline met validation...
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 5000000 10000000 10000000' timeout=45s
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=45s
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=45s
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:35Z INFO  sis_testing::phase3_temporal::deadline_validation]     ✅ Deadline met: PASSED
[2025-11-16T18:48:35Z INFO  sis_testing::phase3_temporal::deadline_validation]   Testing deadline miss detection...
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 1000000 2000000 2000000' timeout=45s
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=45s
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=45s
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=45s
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:35Z INFO  sis_testing::phase3_temporal::deadline_validation]     ✅ Deadline miss detection: PASSED
[2025-11-16T18:48:35Z INFO  sis_testing::phase3_temporal::deadline_validation]   Testing WCET validation...
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 10000000 50000000 50000000' timeout=45s
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:35Z INFO  sis_testing::phase3_temporal::deadline_validation]     ✅ WCET validation: PASSED
[2025-11-16T18:48:35Z INFO  sis_testing::phase3_temporal::deadline_validation]   Testing periodic deadline guarantees...
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 5000000 20000000 20000000' timeout=45s
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=45s
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 200' timeout=45s
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=45s
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:35Z INFO  sis_testing::phase3_temporal::deadline_validation]     ✅ Periodic deadlines: PASSED
[2025-11-16T18:48:35Z INFO  sis_testing::phase3_temporal::deadline_validation] Deadline Validation Tests: 4/4 passed (100%)
[2025-11-16T18:48:35Z INFO  sis_testing::phase3_temporal::latency_tests] Running Latency Tests...
[2025-11-16T18:48:35Z INFO  sis_testing::phase3_temporal::latency_tests]   Testing baseline latency...
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='rtaivalidation' timeout=45s
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:36Z INFO  sis_testing::phase3_temporal::latency_tests]     ✅ Baseline latency: PASSED
[2025-11-16T18:48:36Z INFO  sis_testing::phase3_temporal::latency_tests]   Testing latency under load...
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=45s
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=45s
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='rtaivalidation' timeout=45s
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:36Z INFO  sis_testing::phase3_temporal::latency_tests]     ✅ Latency under load: PASSED
[2025-11-16T18:48:36Z INFO  sis_testing::phase3_temporal::latency_tests]   Testing latency stability...
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 3000000 10000000 10000000' timeout=45s
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=45s
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=45s
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=45s
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=45s
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test' timeout=45s
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=45s
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:48:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:48:37Z INFO  sis_testing::phase3_temporal::latency_tests]     ✅ Latency stability: PASSED
[2025-11-16T18:48:37Z INFO  sis_testing::phase3_temporal::latency_tests] Latency Tests: 3/3 passed (100%)
[2025-11-16T18:48:37Z INFO  sis_testing::phase3_temporal] ==================================================
[2025-11-16T18:48:37Z INFO  sis_testing::phase3_temporal] Phase 3 Summary:
[2025-11-16T18:48:37Z INFO  sis_testing::phase3_temporal]   Active Isolation:     ✅ PASSED
[2025-11-16T18:48:37Z INFO  sis_testing::phase3_temporal]   Deadline Validation:  ✅ PASSED
[2025-11-16T18:48:37Z INFO  sis_testing::phase3_temporal]   Latency Tests:        ✅ PASSED
[2025-11-16T18:48:37Z INFO  sis_testing::phase3_temporal]   Overall:              10/10 tests passed (100.0%)
[2025-11-16T18:48:37Z INFO  sis_testing::phase3_temporal] ==================================================
[2025-11-16T18:48:37Z INFO  sis_testing::reporting] Generating comprehensive industry-grade validation report
[2025-11-16T18:48:37Z INFO  sis_testing::reporting::analytics] Generating comprehensive analytics report
[2025-11-16T18:48:37Z INFO  sis_testing::reporting] JSON report written to: target/testing/validation_report.json
[2025-11-16T18:48:37Z INFO  sis_testing::reporting] Analytics JSON report written to: target/testing/analytics_report.json
[2025-11-16T18:48:37Z INFO  sis_testing::reporting::visualization] Generating interactive visualization dashboard
[2025-11-16T18:48:37Z INFO  sis_testing::reporting] Interactive dashboard written to: target/testing/interactive_dashboard.html
[2025-11-16T18:48:37Z INFO  sis_testing::reporting] HTML dashboard written to: target/testing/dashboard.html
[2025-11-16T18:48:37Z INFO  sis_testing::reporting] Executive summary written to: target/testing/executive_summary.md
[2025-11-16T18:48:37Z INFO  sis_testing::reporting] Technical report written to: target/testing/technical_report.md
[2025-11-16T18:48:37Z INFO  sis_testing::reporting] Performance charts placeholder written to: target/testing/performance_charts.svg
[2025-11-16T18:48:37Z INFO  sis_testing::reporting] Comprehensive industry-grade report generated in: target/testing
[2025-11-16T18:48:37Z WARN  sis_testing] Cannot shutdown QEMU: Arc has multiple owners
[2025-11-16T18:48:37Z INFO  sis_test_runner] 
[2025-11-16T18:48:37Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-16T18:48:37Z INFO  sis_test_runner] ║          SIS KERNEL COMPREHENSIVE TEST VALIDATION REPORT          ║
[2025-11-16T18:48:37Z INFO  sis_test_runner] ╚═══════════════════════════════════════════════════════════════════╝
[2025-11-16T18:48:37Z INFO  sis_test_runner] 
[2025-11-16T18:48:37Z INFO  sis_test_runner]   Status: █ PRODUCTION READY
[2025-11-16T18:48:37Z INFO  sis_test_runner]   Overall Score: 100.0%
[2025-11-16T18:48:37Z INFO  sis_test_runner]   Test Results: 1 PASS / 0 FAIL / 1 TOTAL
[2025-11-16T18:48:37Z INFO  sis_test_runner] 
[2025-11-16T18:48:37Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:48:37Z INFO  sis_test_runner] │ CORE SYSTEM COVERAGE                                            │
[2025-11-16T18:48:37Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:48:37Z INFO  sis_test_runner] │  Performance:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:48:37Z INFO  sis_test_runner] │  Correctness:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:48:37Z INFO  sis_test_runner] │  Security:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:48:37Z INFO  sis_test_runner] │  Distributed:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:48:37Z INFO  sis_test_runner] │  AI Validation:      0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:48:37Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:48:37Z INFO  sis_test_runner] 
[2025-11-16T18:48:37Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:48:37Z INFO  sis_test_runner] │ PHASE IMPLEMENTATION PROGRESS                                   │
[2025-11-16T18:48:37Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:48:37Z INFO  sis_test_runner] │  Phase 1 - AI-Native Dataflow:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:48:37Z INFO  sis_test_runner] │  Phase 2 - AI Governance:                0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:48:37Z INFO  sis_test_runner] │  Phase 3 - Temporal Isolation:         100.0%  ███████████████████████
[2025-11-16T18:48:37Z INFO  sis_test_runner] │  Phase 5 - UX Safety:                    0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:48:37Z INFO  sis_test_runner] │  Phase 6 - Web GUI Management:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:48:37Z INFO  sis_test_runner] │  Phase 7 - AI Operations:                0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:48:37Z INFO  sis_test_runner] │  Phase 8 - Performance Optimization:     0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:48:37Z INFO  sis_test_runner] │  Phase 9 - Agentic Platform:             0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:48:37Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:48:37Z INFO  sis_test_runner] 
[2025-11-16T18:48:37Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:48:37Z INFO  sis_test_runner] │ DETAILED VALIDATION RESULTS                                     │
[2025-11-16T18:48:37Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:48:37Z INFO  sis_test_runner] │ ✓ PASS
[2025-11-16T18:48:37Z INFO  sis_test_runner] │   Test: Phase 3: Temporal Isolation
[2025-11-16T18:48:37Z INFO  sis_test_runner] │   Target: ≥75% pass rate | Measured: 100.0%
[2025-11-16T18:48:37Z INFO  sis_test_runner] │   Industry Benchmark: Industry standard: Real-time 70-80%
[2025-11-16T18:48:37Z INFO  sis_test_runner] │
[2025-11-16T18:48:37Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:48:37Z INFO  sis_test_runner] 
[2025-11-16T18:48:37Z INFO  sis_test_runner] 📊 Reports generated in: target/testing/
[2025-11-16T18:48:37Z INFO  sis_test_runner] 🌐 View dashboard: target/testing/dashboard.html
[2025-11-16T18:48:37Z INFO  sis_test_runner] 
[2025-11-16T18:48:37Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-16T18:48:37Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-16T18:48:37Z INFO  sis_test_runner] ║  ✓ SUCCESS: SIS Kernel meets industry standards for production   ║
[2025-11-16T18:48:37Z INFO  sis_test_runner] ║    deployment and is ready for production use.                   ║
[2025-11-16T18:48:37Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-16T18:48:37Z INFO  sis_test_runner] ╚═══════════════════════════════════════════════════════════════════╝
amoljassal@Amols-Mac-mini sis-kernel % SIS_FEATURES="ai-ops,bringup,crypto-real,decision-traces,default,deterministic,graphctl-framed,llm,model-lifecycle,otel,shadow-mode,agentsys" cargo run -p sis-testing --release -- --phase 5
warning: profile package spec `bootloader` in profile `dev` did not match any packages
warning: profile package spec `bootloader_api` in profile `dev` did not match any packages
    Finished `release` profile [optimized] target(s) in 0.16s
     Running `target/release/sis-test-runner --phase 5`
[2025-11-16T18:48:44Z INFO  sis_test_runner] SIS Kernel Industry-Grade Test Suite
[2025-11-16T18:48:44Z INFO  sis_test_runner] ====================================
[2025-11-16T18:48:44Z INFO  sis_test_runner] Mode: default (single QEMU node, moderate iterations)
[2025-11-16T18:48:44Z INFO  sis_test_runner] Test Configuration:
[2025-11-16T18:48:44Z INFO  sis_test_runner]   QEMU Nodes: 1
[2025-11-16T18:48:44Z INFO  sis_test_runner]   Duration: 600s
[2025-11-16T18:48:44Z INFO  sis_test_runner]   Performance Iterations: 2000
[2025-11-16T18:48:44Z INFO  sis_test_runner]   Statistical Confidence: 99.0%
[2025-11-16T18:48:44Z INFO  sis_test_runner]   Output Directory: target/testing
[2025-11-16T18:48:44Z INFO  sis_test_runner]   Parallel Execution: true
[2025-11-16T18:48:44Z INFO  sis_test_runner] Initializing QEMU runtime for kernel validation...
[2025-11-16T18:48:44Z INFO  sis_testing] Initializing QEMU runtime for comprehensive kernel testing
[2025-11-16T18:48:44Z INFO  sis_testing::qemu_runtime] Building SIS kernel for QEMU testing
[2025-11-16T18:48:44Z INFO  sis_testing::qemu_runtime] Building UEFI bootloader...
[2025-11-16T18:48:44Z INFO  sis_testing::qemu_runtime] Building kernel with features: ai-ops,bringup,crypto-real,decision-traces,default,deterministic,graphctl-framed,llm,model-lifecycle,otel,shadow-mode,agentsys
[2025-11-16T18:48:44Z INFO  sis_testing::qemu_runtime] SIS kernel and UEFI bootloader built successfully
[2025-11-16T18:48:44Z INFO  sis_testing::qemu_runtime] Preparing ESP directories for 1 QEMU instances
[2025-11-16T18:48:44Z INFO  sis_testing::qemu_runtime] ESP directories prepared for all instances
[2025-11-16T18:48:44Z INFO  sis_testing::qemu_runtime] Launching QEMU cluster with 1 nodes
[2025-11-16T18:48:44Z INFO  sis_testing::qemu_runtime] Launching QEMU instance 0 on ports 7000/7100/7200
[2025-11-16T18:48:44Z INFO  sis_testing::qemu_runtime] Instance 0 launched (serial log: target/testing/serial-node0.log)
[2025-11-16T18:48:47Z INFO  sis_testing::qemu_runtime] All QEMU instances launched successfully
[2025-11-16T18:48:47Z INFO  sis_testing::qemu_runtime] Waiting for instance 0 to boot (timeout: 180s)
[2025-11-16T18:48:47Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T18:48:49Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T18:48:51Z INFO  sis_testing::qemu_runtime] Instance 0 booted successfully (detected via serial log)
[2025-11-16T18:48:51Z INFO  sis_testing] QEMU runtime initialized with 1 node(s); boot detected via serial log
[2025-11-16T18:48:51Z INFO  sis_test_runner] QEMU runtime initialized successfully - running real kernel tests
[2025-11-16T18:48:51Z INFO  sis_test_runner] Phase selection detected: running Phase 5 only
[2025-11-16T18:48:51Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-16T18:48:51Z INFO  sis_testing] Phase 1-9 test suites initialized successfully
[2025-11-16T18:48:51Z INFO  sis_testing] Starting single-phase validation for Phase 5
[2025-11-16T18:48:51Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-16T18:48:51Z INFO  sis_testing] Phase 1-9 test suites initialized successfully
[2025-11-16T18:48:51Z INFO  sis_testing::phase5_ux_safety] =================================================
[2025-11-16T18:48:51Z INFO  sis_testing::phase5_ux_safety] Starting Phase 5: User Experience Safety
[2025-11-16T18:48:51Z INFO  sis_testing::phase5_ux_safety] =================================================
[2025-11-16T18:48:51Z INFO  sis_testing::phase5_ux_safety::safety_controls] Running Safety Controls Tests...
[2025-11-16T18:48:51Z INFO  sis_testing::phase5_ux_safety::safety_controls]   Testing inference guardrails...
[2025-11-16T18:48:51Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=45s
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl budget --period-ns 1000000000 --max-tokens-per-period 5' timeout=45s
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer safe test --max-tokens 3' timeout=45s
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:34Z INFO  sis_testing::phase5_ux_safety::safety_controls]     ✅ Inference guardrails: PASSED
[2025-11-16T18:49:34Z INFO  sis_testing::phase5_ux_safety::safety_controls]   Testing resource protection...
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 70 --ctx 32768 --vocab 100000 --quant int8 --size-bytes 268435456' timeout=45s
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 7 --ctx 512 --vocab 50000 --quant int8 --size-bytes 524288' timeout=45s
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:34Z INFO  sis_testing::phase5_ux_safety::safety_controls]     ✅ Resource protection: PASSED
[2025-11-16T18:49:34Z INFO  sis_testing::phase5_ux_safety::safety_controls]   Testing safety validation...
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=45s
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:34Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer validation test --max-tokens 5' timeout=45s
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl status' timeout=45s
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:35Z INFO  sis_testing::phase5_ux_safety::safety_controls]     ✅ Safety validation: PASSED
[2025-11-16T18:49:35Z INFO  sis_testing::phase5_ux_safety::safety_controls] Safety Controls Tests: 3/3 passed (100%)
[2025-11-16T18:49:35Z INFO  sis_testing::phase5_ux_safety::explainability] Running Explainability Tests...
2025-11-16T18:49:35Z INFO  sis_testing::phase5_ux_safety::explainability]   Testing decision transparency...
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=45s
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer transparency test input --max-tokens 5' timeout=45s
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmjson' timeout=45s
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:35Z INFO  sis_testing::phase5_ux_safety::explainability]     ✅ Decision transparency: PASSED
[2025-11-16T18:49:35Z INFO  sis_testing::phase5_ux_safety::explainability]   Testing model introspection...
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --model 7 --ctx 2048 --vocab 50000 --quant int8 --size-bytes 1048576' timeout=45s
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl status' timeout=45s
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:35Z INFO  sis_testing::phase5_ux_safety::explainability]     ✅ Model introspection: PASSED
[2025-11-16T18:49:35Z INFO  sis_testing::phase5_ux_safety::explainability]   Testing audit accessibility...
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=45s
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer audit test 0 --max-tokens 3' timeout=45s
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:35Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer audit test 1 --max-tokens 3' timeout=45s
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer audit test 2 --max-tokens 3' timeout=45s
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmjson' timeout=45s
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:36Z INFO  sis_testing::phase5_ux_safety::explainability]     ✅ Audit accessibility: PASSED
[2025-11-16T18:49:36Z INFO  sis_testing::phase5_ux_safety::explainability] Explainability Tests: 3/3 passed (100%)
[2025-11-16T18:49:36Z INFO  sis_testing::phase5_ux_safety::user_feedback] Running User Feedback Tests...
[2025-11-16T18:49:36Z INFO  sis_testing::phase5_ux_safety::user_feedback]   Testing error reporting...
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer test without model --max-tokens 5' timeout=45s
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=45s
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer valid test --max-tokens 3' timeout=45s
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:36Z INFO  sis_testing::phase5_ux_safety::user_feedback]     ✅ Error reporting: PASSED
[2025-11-16T18:49:36Z INFO  sis_testing::phase5_ux_safety::user_feedback]   Testing status feedback...
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=45s
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl status' timeout=45s
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer status test --max-tokens 5' timeout=45s
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:37Z INFO  sis_testing::phase5_ux_safety::user_feedback]     ✅ Status feedback: PASSED
[2025-11-16T18:49:37Z INFO  sis_testing::phase5_ux_safety::user_feedback]   Testing operation confirmation...
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --wcet-cycles 50000' timeout=45s
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer confirmation test --max-tokens 5' timeout=45s
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl budget --period-ns 1000000000 --max-tokens-per-period 10' timeout=45s
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:49:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:49:37Z INFO  sis_testing::phase5_ux_safety::user_feedback]     ✅ Operation confirmation: PASSED
[2025-11-16T18:49:37Z INFO  sis_testing::phase5_ux_safety::user_feedback] User Feedback Tests: 3/3 passed (100%)
[2025-11-16T18:49:37Z INFO  sis_testing::phase5_ux_safety] =================================================
[2025-11-16T18:49:37Z INFO  sis_testing::phase5_ux_safety] Phase 5 Summary:
[2025-11-16T18:49:37Z INFO  sis_testing::phase5_ux_safety]   Safety Controls:      ✅ PASSED
[2025-11-16T18:49:37Z INFO  sis_testing::phase5_ux_safety]   Explainability:       ✅ PASSED
[2025-11-16T18:49:37Z INFO  sis_testing::phase5_ux_safety]   User Feedback:        ✅ PASSED
[2025-11-16T18:49:37Z INFO  sis_testing::phase5_ux_safety]   Overall:              9/9 tests passed (100.0%)
[2025-11-16T18:49:37Z INFO  sis_testing::phase5_ux_safety] =================================================
[2025-11-16T18:49:37Z INFO  sis_testing::reporting] Generating comprehensive industry-grade validation report
[2025-11-16T18:49:37Z INFO  sis_testing::reporting::analytics] Generating comprehensive analytics report
[2025-11-16T18:49:37Z INFO  sis_testing::reporting] JSON report written to: target/testing/validation_report.json
[2025-11-16T18:49:37Z INFO  sis_testing::reporting] Analytics JSON report written to: target/testing/analytics_report.json
[2025-11-16T18:49:37Z INFO  sis_testing::reporting::visualization] Generating interactive visualization dashboard
[2025-11-16T18:49:37Z INFO  sis_testing::reporting] Interactive dashboard written to: target/testing/interactive_dashboard.html
[2025-11-16T18:49:37Z INFO  sis_testing::reporting] HTML dashboard written to: target/testing/dashboard.html
[2025-11-16T18:49:37Z INFO  sis_testing::reporting] Executive summary written to: target/testing/executive_summary.md
[2025-11-16T18:49:37Z INFO  sis_testing::reporting] Technical report written to: target/testing/technical_report.md
[2025-11-16T18:49:37Z INFO  sis_testing::reporting] Performance charts placeholder written to: target/testing/performance_charts.svg
[2025-11-16T18:49:37Z INFO  sis_testing::reporting] Comprehensive industry-grade report generated in: target/testing
[2025-11-16T18:49:37Z WARN  sis_testing] Cannot shutdown QEMU: Arc has multiple owners
[2025-11-16T18:49:37Z INFO  sis_test_runner] 
[2025-11-16T18:49:37Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-16T18:49:37Z INFO  sis_test_runner] ║          SIS KERNEL COMPREHENSIVE TEST VALIDATION REPORT          ║
[2025-11-16T18:49:37Z INFO  sis_test_runner] ╚═══════════════════════════════════════════════════════════════════╝
[2025-11-16T18:49:37Z INFO  sis_test_runner] 
[2025-11-16T18:49:37Z INFO  sis_test_runner]   Status: █ PRODUCTION READY
[2025-11-16T18:49:37Z INFO  sis_test_runner]   Overall Score: 100.0%
[2025-11-16T18:49:37Z INFO  sis_test_runner]   Test Results: 1 PASS / 0 FAIL / 1 TOTAL
[2025-11-16T18:49:37Z INFO  sis_test_runner] 
[2025-11-16T18:49:37Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:49:37Z INFO  sis_test_runner] │ CORE SYSTEM COVERAGE                                            │
[2025-11-16T18:49:37Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:49:37Z INFO  sis_test_runner] │  Performance:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:49:37Z INFO  sis_test_runner] │  Correctness:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:49:37Z INFO  sis_test_runner] │  Security:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:49:37Z INFO  sis_test_runner] │  Distributed:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:49:37Z INFO  sis_test_runner] │  AI Validation:      0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:49:37Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:49:37Z INFO  sis_test_runner] 
[2025-11-16T18:49:37Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:49:37Z INFO  sis_test_runner] │ PHASE IMPLEMENTATION PROGRESS                                   │
[2025-11-16T18:49:37Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:49:37Z INFO  sis_test_runner] │  Phase 1 - AI-Native Dataflow:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:49:37Z INFO  sis_test_runner] │  Phase 2 - AI Governance:                0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:49:37Z INFO  sis_test_runner] │  Phase 3 - Temporal Isolation:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:49:37Z INFO  sis_test_runner] │  Phase 5 - UX Safety:                  100.0%  ███████████████████████
[2025-11-16T18:49:37Z INFO  sis_test_runner] │  Phase 6 - Web GUI Management:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:49:37Z INFO  sis_test_runner] │  Phase 7 - AI Operations:                0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:49:37Z INFO  sis_test_runner] │  Phase 8 - Performance Optimization:     0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:49:37Z INFO  sis_test_runner] │  Phase 9 - Agentic Platform:             0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:49:37Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:49:37Z INFO  sis_test_runner] 
[2025-11-16T18:49:37Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:49:37Z INFO  sis_test_runner] │ DETAILED VALIDATION RESULTS                                     │
[2025-11-16T18:49:37Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:49:37Z INFO  sis_test_runner] │ ✓ PASS
[2025-11-16T18:49:37Z INFO  sis_test_runner] │   Test: Phase 5: User Experience Safety
[2025-11-16T18:49:37Z INFO  sis_test_runner] │   Target: ≥75% pass rate | Measured: 100.0%
[2025-11-16T18:49:37Z INFO  sis_test_runner] │   Industry Benchmark: Industry standard: UX safety 55-70%
[2025-11-16T18:49:37Z INFO  sis_test_runner] │
[2025-11-16T18:49:37Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:49:37Z INFO  sis_test_runner] 
[2025-11-16T18:49:37Z INFO  sis_test_runner] 📊 Reports generated in: target/testing/
[2025-11-16T18:49:37Z INFO  sis_test_runner] 🌐 View dashboard: target/testing/dashboard.html
[2025-11-16T18:49:37Z INFO  sis_test_runner] 
[2025-11-16T18:49:37Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-16T18:49:37Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-16T18:49:37Z INFO  sis_test_runner] ║  ✓ SUCCESS: SIS Kernel meets industry standards for production   ║
[2025-11-16T18:49:37Z INFO  sis_test_runner] ║    deployment and is ready for production use.                   ║
[2025-11-16T18:49:37Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-16T18:49:37Z INFO  sis_test_runner] ╚═══════════════════════════════════════════════════════════════════╝
amoljassal@Amols-Mac-mini sis-kernel % SIS_FEATURES="ai-ops,bringup,crypto-real,decision-traces,default,deterministic,graphctl-framed,llm,model-lifecycle,otel,shadow-mode,agentsys" cargo run -p sis-testing --release -- --phase 6
warning: profile package spec `bootloader` in profile `dev` did not match any packages
warning: profile package spec `bootloader_api` in profile `dev` did not match any packages
    Finished `release` profile [optimized] target(s) in 0.16s
     Running `target/release/sis-test-runner --phase 6`
[2025-11-16T18:49:52Z INFO  sis_test_runner] SIS Kernel Industry-Grade Test Suite
[2025-11-16T18:49:52Z INFO  sis_test_runner] ====================================
[2025-11-16T18:49:52Z INFO  sis_test_runner] Mode: default (single QEMU node, moderate iterations)
[2025-11-16T18:49:52Z INFO  sis_test_runner] Test Configuration:
[2025-11-16T18:49:52Z INFO  sis_test_runner]   QEMU Nodes: 1
[2025-11-16T18:49:52Z INFO  sis_test_runner]   Duration: 600s
[2025-11-16T18:49:52Z INFO  sis_test_runner]   Performance Iterations: 2000
[2025-11-16T18:49:52Z INFO  sis_test_runner]   Statistical Confidence: 99.0%
[2025-11-16T18:49:52Z INFO  sis_test_runner]   Output Directory: target/testing
[2025-11-16T18:49:52Z INFO  sis_test_runner]   Parallel Execution: true
[2025-11-16T18:49:52Z INFO  sis_test_runner] Initializing QEMU runtime for kernel validation...
[2025-11-16T18:49:52Z INFO  sis_testing] Initializing QEMU runtime for comprehensive kernel testing
[2025-11-16T18:49:52Z INFO  sis_testing::qemu_runtime] Building SIS kernel for QEMU testing
[2025-11-16T18:49:52Z INFO  sis_testing::qemu_runtime] Building UEFI bootloader...
[2025-11-16T18:49:52Z INFO  sis_testing::qemu_runtime] Building kernel with features: ai-ops,bringup,crypto-real,decision-traces,default,deterministic,graphctl-framed,llm,model-lifecycle,otel,shadow-mode,agentsys
[2025-11-16T18:49:52Z INFO  sis_testing::qemu_runtime] SIS kernel and UEFI bootloader built successfully
[2025-11-16T18:49:52Z INFO  sis_testing::qemu_runtime] Preparing ESP directories for 1 QEMU instances
[2025-11-16T18:49:52Z INFO  sis_testing::qemu_runtime] ESP directories prepared for all instances
[2025-11-16T18:49:52Z INFO  sis_testing::qemu_runtime] Launching QEMU cluster with 1 nodes
[2025-11-16T18:49:52Z INFO  sis_testing::qemu_runtime] Launching QEMU instance 0 on ports 7000/7100/7200
[2025-11-16T18:49:52Z INFO  sis_testing::qemu_runtime] Instance 0 launched (serial log: target/testing/serial-node0.log)
[2025-11-16T18:49:55Z INFO  sis_testing::qemu_runtime] All QEMU instances launched successfully
[2025-11-16T18:49:55Z INFO  sis_testing::qemu_runtime] Waiting for instance 0 to boot (timeout: 180s)
[2025-11-16T18:49:55Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T18:49:57Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T18:49:59Z INFO  sis_testing::qemu_runtime] Instance 0 booted successfully (detected via serial log)
[2025-11-16T18:49:59Z INFO  sis_testing] QEMU runtime initialized with 1 node(s); boot detected via serial log
[2025-11-16T18:49:59Z INFO  sis_test_runner] QEMU runtime initialized successfully - running real kernel tests
[2025-11-16T18:49:59Z INFO  sis_test_runner] Phase selection detected: running Phase 6 only
[2025-11-16T18:49:59Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-16T18:49:59Z INFO  sis_testing] Phase 1-9 test suites initialized successfully
[2025-11-16T18:49:59Z INFO  sis_testing] Starting single-phase validation for Phase 6
[2025-11-16T18:49:59Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-16T18:49:59Z INFO  sis_testing] Phase 1-9 test suites initialized successfully
[2025-11-16T18:49:59Z INFO  sis_testing::phase6_web_gui] ==================================================
[2025-11-16T18:49:59Z INFO  sis_testing::phase6_web_gui] Starting Phase 6: Web GUI Management Validation
[2025-11-16T18:49:59Z INFO  sis_testing::phase6_web_gui] ==================================================
[2025-11-16T18:49:59Z INFO  sis_testing::phase6_web_gui::http_server] Running HTTP Server Tests...
[2025-11-16T18:49:59Z INFO  sis_testing::phase6_web_gui::http_server]   Testing server startup...
[2025-11-16T18:49:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl start --port 8080' timeout=45s
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:11Z INFO  sis_testing::phase6_web_gui::http_server]     ✅ Server startup: PASSED
[2025-11-16T18:51:11Z INFO  sis_testing::phase6_web_gui::http_server]   Testing health endpoint...
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl status' timeout=45s
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:11Z INFO  sis_testing::phase6_web_gui::http_server]     ✅ Health endpoint: PASSED
[2025-11-16T18:51:11Z INFO  sis_testing::phase6_web_gui::http_server]   Testing server shutdown...
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl stop' timeout=45s
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:11Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::http_server]     ✅ Server shutdown: PASSED
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::http_server] HTTP Server Tests: 3/3 passed (100%)
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::websocket] Running WebSocket Tests...
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::websocket]   Testing WebSocket connection...
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl ws-status' timeout=45s
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::websocket]     ✅ WebSocket connection: PASSED
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::websocket]   Testing ping/pong heartbeat...
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl ws-ping' timeout=45s
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::websocket]     ✅ Ping/pong: PASSED
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::websocket]   Testing metric subscription...
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl subscribe memory_pressure cpu_usage' timeout=45s
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::websocket]     ✅ Metric subscription: PASSED
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::websocket] WebSocket Tests: 3/3 passed (100%)
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::api_endpoints] Running API Endpoint Tests...
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::api_endpoints]   Testing GET /api/metrics...
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl api-test /api/metrics' timeout=45s
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::api_endpoints]     ✅ GET /api/metrics: PASSED
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::api_endpoints]   Testing POST /api/command...
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl api-exec 'memctl status'' timeout=45s
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::api_endpoints]     ✅ POST /api/command: PASSED
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::api_endpoints]   Testing GET /api/logs...
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl api-test '/api/logs?lines=100'' timeout=45s
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::api_endpoints]     ✅ GET /api/logs: PASSED
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::api_endpoints] API Endpoint Tests: 3/3 passed (100%)
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::authentication] Running Authentication Tests...
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::authentication]   Testing token authentication...
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl auth-token generate' timeout=45s
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::authentication]     ✅ Token authentication: PASSED
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::authentication]   Testing invalid credentials handling...
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl auth-test --token invalid_token' timeout=45s
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::authentication]     ✅ Invalid credentials: PASSED
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::authentication]   Testing session management...
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl session list' timeout=45s
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::authentication]     ✅ Session management: PASSED
[2025-11-16T18:51:12Z INFO  sis_testing::phase6_web_gui::authentication]   Testing authorization...
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl auth-check --role admin' timeout=45s
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:12Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:13Z INFO  sis_testing::phase6_web_gui::authentication]     ✅ Authorization: PASSED
[2025-11-16T18:51:13Z INFO  sis_testing::phase6_web_gui::authentication] Authentication Tests: 4/4 passed (100%)
[2025-11-16T18:51:13Z INFO  sis_testing::phase6_web_gui::real_time_updates] Running Real-Time Update Tests...
[2025-11-16T18:51:13Z INFO  sis_testing::phase6_web_gui::real_time_updates]   Testing metric streaming...
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl stream start --metrics memory_pressure cpu_usage' timeout=45s
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:13Z INFO  sis_testing::phase6_web_gui::real_time_updates]     ✅ Metric streaming: PASSED
[2025-11-16T18:51:13Z INFO  sis_testing::phase6_web_gui::real_time_updates]   Testing update frequency...
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl stream start --rate 1000' timeout=45s
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl stream stats' timeout=45s
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:15Z INFO  sis_testing::phase6_web_gui::real_time_updates]     ✅ Update frequency: PASSED
[2025-11-16T18:51:15Z INFO  sis_testing::phase6_web_gui::real_time_updates]   Testing multiple subscribers...
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl subscribers count' timeout=45s
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:15Z INFO  sis_testing::phase6_web_gui::real_time_updates]     ✅ Multiple subscribers: PASSED
[2025-11-16T18:51:15Z INFO  sis_testing::phase6_web_gui::real_time_updates]   Testing data format validation...
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='webctl stream sample' timeout=45s
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:51:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:51:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:51:16Z INFO  sis_testing::phase6_web_gui::real_time_updates]     ✅ Data format: PASSED
[2025-11-16T18:51:16Z INFO  sis_testing::phase6_web_gui::real_time_updates] Real-Time Update Tests: 4/4 passed (100%)
[2025-11-16T18:51:16Z INFO  sis_testing::phase6_web_gui] ==================================================
[2025-11-16T18:51:16Z INFO  sis_testing::phase6_web_gui] Phase 6 Summary:
[2025-11-16T18:51:16Z INFO  sis_testing::phase6_web_gui]   HTTP Server:        ✅ PASSED
[2025-11-16T18:51:16Z INFO  sis_testing::phase6_web_gui]   WebSocket:          ✅ PASSED
[2025-11-16T18:51:16Z INFO  sis_testing::phase6_web_gui]   API Endpoints:      ✅ PASSED
[2025-11-16T18:51:16Z INFO  sis_testing::phase6_web_gui]   Authentication:     ✅ PASSED
[2025-11-16T18:51:16Z INFO  sis_testing::phase6_web_gui]   Real-Time Updates:  ✅ PASSED
[2025-11-16T18:51:16Z INFO  sis_testing::phase6_web_gui]   Overall:            17/17 tests passed (100.0%)
[2025-11-16T18:51:16Z INFO  sis_testing::phase6_web_gui] ==================================================
[2025-11-16T18:51:16Z INFO  sis_testing::reporting] Generating comprehensive industry-grade validation report
[2025-11-16T18:51:16Z INFO  sis_testing::reporting::analytics] Generating comprehensive analytics report
[2025-11-16T18:51:16Z INFO  sis_testing::reporting] JSON report written to: target/testing/validation_report.json
[2025-11-16T18:51:16Z INFO  sis_testing::reporting] Analytics JSON report written to: target/testing/analytics_report.json
[2025-11-16T18:51:16Z INFO  sis_testing::reporting::visualization] Generating interactive visualization dashboard
[2025-11-16T18:51:16Z INFO  sis_testing::reporting] Interactive dashboard written to: target/testing/interactive_dashboard.html
[2025-11-16T18:51:16Z INFO  sis_testing::reporting] HTML dashboard written to: target/testing/dashboard.html
[2025-11-16T18:51:16Z INFO  sis_testing::reporting] Executive summary written to: target/testing/executive_summary.md
[2025-11-16T18:51:16Z INFO  sis_testing::reporting] Technical report written to: target/testing/technical_report.md
[2025-11-16T18:51:16Z INFO  sis_testing::reporting] Performance charts placeholder written to: target/testing/performance_charts.svg
[2025-11-16T18:51:16Z INFO  sis_testing::reporting] Comprehensive industry-grade report generated in: target/testing
[2025-11-16T18:51:16Z WARN  sis_testing] Cannot shutdown QEMU: Arc has multiple owners
[2025-11-16T18:51:16Z INFO  sis_test_runner] 
[2025-11-16T18:51:16Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-16T18:51:16Z INFO  sis_test_runner] ║          SIS KERNEL COMPREHENSIVE TEST VALIDATION REPORT          ║
[2025-11-16T18:51:16Z INFO  sis_test_runner] ╚═══════════════════════════════════════════════════════════════════╝
[2025-11-16T18:51:16Z INFO  sis_test_runner] 
[2025-11-16T18:51:16Z INFO  sis_test_runner]   Status: █ PRODUCTION READY
[2025-11-16T18:51:16Z INFO  sis_test_runner]   Overall Score: 100.0%
[2025-11-16T18:51:16Z INFO  sis_test_runner]   Test Results: 1 PASS / 0 FAIL / 1 TOTAL
[2025-11-16T18:51:16Z INFO  sis_test_runner] 
[2025-11-16T18:51:16Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:51:16Z INFO  sis_test_runner] │ CORE SYSTEM COVERAGE                                            │
[2025-11-16T18:51:16Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:51:16Z INFO  sis_test_runner] │  Performance:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:51:16Z INFO  sis_test_runner] │  Correctness:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:51:16Z INFO  sis_test_runner] │  Security:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:51:16Z INFO  sis_test_runner] │  Distributed:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:51:16Z INFO  sis_test_runner] │  AI Validation:      0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:51:16Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:51:16Z INFO  sis_test_runner] 
[2025-11-16T18:51:16Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:51:16Z INFO  sis_test_runner] │ PHASE IMPLEMENTATION PROGRESS                                   │
[2025-11-16T18:51:16Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:51:16Z INFO  sis_test_runner] │  Phase 1 - AI-Native Dataflow:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:51:16Z INFO  sis_test_runner] │  Phase 2 - AI Governance:                0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:51:16Z INFO  sis_test_runner] │  Phase 3 - Temporal Isolation:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:51:16Z INFO  sis_test_runner] │  Phase 5 - UX Safety:                    0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:51:16Z INFO  sis_test_runner] │  Phase 6 - Web GUI Management:         100.0%  ███████████████████████
[2025-11-16T18:51:16Z INFO  sis_test_runner] │  Phase 7 - AI Operations:                0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:51:16Z INFO  sis_test_runner] │  Phase 8 - Performance Optimization:     0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:51:16Z INFO  sis_test_runner] │  Phase 9 - Agentic Platform:             0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:51:16Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:51:16Z INFO  sis_test_runner] 
[2025-11-16T18:51:16Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:51:16Z INFO  sis_test_runner] │ DETAILED VALIDATION RESULTS                                     │
[2025-11-16T18:51:16Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:51:16Z INFO  sis_test_runner] │ ✓ PASS
[2025-11-16T18:51:16Z INFO  sis_test_runner] │   Test: Phase 6: Web GUI Management
[2025-11-16T18:51:16Z INFO  sis_test_runner] │   Target: ≥75% pass rate | Measured: 100.0%
[2025-11-16T18:51:16Z INFO  sis_test_runner] │   Industry Benchmark: Industry standard: Web UI 65-75%
[2025-11-16T18:51:16Z INFO  sis_test_runner] │
[2025-11-16T18:51:16Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:51:16Z INFO  sis_test_runner] 
[2025-11-16T18:51:16Z INFO  sis_test_runner] 📊 Reports generated in: target/testing/
[2025-11-16T18:51:16Z INFO  sis_test_runner] 🌐 View dashboard: target/testing/dashboard.html
[2025-11-16T18:51:16Z INFO  sis_test_runner] 
[2025-11-16T18:51:16Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-16T18:51:16Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-16T18:51:16Z INFO  sis_test_runner] ║  ✓ SUCCESS: SIS Kernel meets industry standards for production   ║
[2025-11-16T18:51:16Z INFO  sis_test_runner] ║    deployment and is ready for production use.                   ║
[2025-11-16T18:51:16Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-16T18:51:16Z INFO  sis_test_runner] ╚═══════════════════════════════════

amoljassal@Amols-Mac-mini sis-kernel % SIS_FEATURES="ai-ops,bringup,crypto-real,decision-traces,default,deterministic,graphctl-framed,llm,model-lifecycle,otel,shadow-mode,agentsys" cargo run -p sis-testing --release -- --phase 7
warning: profile package spec `bootloader` in profile `dev` did not match any packages
warning: profile package spec `bootloader_api` in profile `dev` did not match any packages
    Finished `release` profile [optimized] target(s) in 0.17s
     Running `target/release/sis-test-runner --phase 7`
[2025-11-16T19:21:45Z INFO  sis_test_runner] SIS Kernel Industry-Grade Test Suite
[2025-11-16T19:21:45Z INFO  sis_test_runner] ====================================
[2025-11-16T19:21:45Z INFO  sis_test_runner] Mode: default (single QEMU node, moderate iterations)
[2025-11-16T19:21:45Z INFO  sis_test_runner] Test Configuration:
[2025-11-16T19:21:45Z INFO  sis_test_runner]   QEMU Nodes: 1
[2025-11-16T19:21:45Z INFO  sis_test_runner]   Duration: 600s
[2025-11-16T19:21:45Z INFO  sis_test_runner]   Performance Iterations: 2000
[2025-11-16T19:21:45Z INFO  sis_test_runner]   Statistical Confidence: 99.0%
[2025-11-16T19:21:45Z INFO  sis_test_runner]   Output Directory: target/testing
[2025-11-16T19:21:45Z INFO  sis_test_runner]   Parallel Execution: true
[2025-11-16T19:21:45Z INFO  sis_test_runner] Initializing QEMU runtime for kernel validation...
[2025-11-16T19:21:45Z INFO  sis_testing] Initializing QEMU runtime for comprehensive kernel testing
[2025-11-16T19:21:45Z INFO  sis_testing::qemu_runtime] Building SIS kernel for QEMU testing
[2025-11-16T19:21:45Z INFO  sis_testing::qemu_runtime] Building UEFI bootloader...
[2025-11-16T19:21:45Z INFO  sis_testing::qemu_runtime] Building kernel with features: ai-ops,bringup,crypto-real,decision-traces,default,deterministic,graphctl-framed,llm,model-lifecycle,otel,shadow-mode,agentsys
[2025-11-16T19:21:46Z INFO  sis_testing::qemu_runtime] SIS kernel and UEFI bootloader built successfully
[2025-11-16T19:21:46Z INFO  sis_testing::qemu_runtime] Preparing ESP directories for 1 QEMU instances
[2025-11-16T19:21:46Z INFO  sis_testing::qemu_runtime] ESP directories prepared for all instances
[2025-11-16T19:21:46Z INFO  sis_testing::qemu_runtime] Launching QEMU cluster with 1 nodes
[2025-11-16T19:21:46Z INFO  sis_testing::qemu_runtime] Launching QEMU instance 0 on ports 7000/7100/7200
[2025-11-16T19:21:46Z INFO  sis_testing::qemu_runtime] Instance 0 launched (serial log: target/testing/serial-node0.log)
[2025-11-16T19:21:49Z INFO  sis_testing::qemu_runtime] All QEMU instances launched successfully
[2025-11-16T19:21:49Z INFO  sis_testing::qemu_runtime] Waiting for instance 0 to boot (timeout: 180s)
[2025-11-16T19:21:49Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T19:21:51Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T19:21:53Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T19:21:55Z INFO  sis_testing::qemu_runtime] Instance 0 booted successfully (detected via serial log)
[2025-11-16T19:21:55Z INFO  sis_testing] QEMU runtime initialized with 1 node(s); boot detected via serial log
[2025-11-16T19:21:55Z INFO  sis_test_runner] QEMU runtime initialized successfully - running real kernel tests
[2025-11-16T19:21:55Z INFO  sis_test_runner] Phase selection detected: running Phase 7 only
[2025-11-16T19:21:55Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-16T19:21:55Z INFO  sis_testing] Phase 1-9 test suites initialized successfully
[2025-11-16T19:21:55Z INFO  sis_testing] Starting single-phase validation for Phase 7
[2025-11-16T19:21:55Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-16T19:21:55Z INFO  sis_testing] Phase 1-9 test suites initialized successfully
[2025-11-16T19:21:55Z INFO  sis_testing::phase7_ai_ops] 🚀 Starting Phase 7: AI Operations Platform validation
[2025-11-16T19:21:55Z INFO  sis_testing::phase7_ai_ops::model_lifecycle] Running Model Lifecycle Tests...
[2025-11-16T19:21:55Z INFO  sis_testing::phase7_ai_ops::model_lifecycle]   Testing model registration...
[2025-11-16T19:21:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id test-model-v1 --size 512 --ctx 2048' timeout=45s
[2025-11-16T19:21:55Z INFO  sis_testing::phase7_ai_ops::shadow_mode] Running Shadow Mode Tests...
[2025-11-16T19:21:55Z INFO  sis_testing::phase7_ai_ops::shadow_mode]   Testing shadow agent deployment...
[2025-11-16T19:21:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-deploy --id shadow-agent-v2 --traffic 0' timeout=45s
[2025-11-16T19:21:55Z INFO  sis_testing::phase7_ai_ops::otel_exporter] Running OpenTelemetry Exporter Tests...
[2025-11-16T19:21:55Z INFO  sis_testing::phase7_ai_ops::otel_exporter]   Testing OTel initialization...
[2025-11-16T19:21:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl init --endpoint http://localhost:4318' timeout=45s
[2025-11-16T19:21:55Z INFO  sis_testing::phase7_ai_ops::decision_traces] Running Decision Traces Tests...
[2025-11-16T19:21:55Z INFO  sis_testing::phase7_ai_ops::decision_traces]   Testing decision trace collection...
[2025-11-16T19:21:55Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl on' timeout=45s
[2025-11-16T19:23:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:23:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:23:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:23:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 158 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 158 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 158 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 157 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 1000' timeout=45s
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:23:57Z WARN  sis_testing::phase7_ai_ops::model_lifecycle]     ⚠️  Registration took 122333ms (target: <100ms)
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl list' timeout=45s
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:23:57Z INFO  sis_testing::phase7_ai_ops::otel_exporter]     ✅ OTel initialization: PASSED
[2025-11-16T19:23:57Z INFO  sis_testing::phase7_ai_ops::otel_exporter]   Testing span creation...
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl enable-tracing' timeout=45s
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-status' timeout=45s
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:23:57Z WARN  sis_testing::phase7_ai_ops::model_lifecycle]     ⚠️  Registry lookup took 103ms (target: <10ms)
[2025-11-16T19:23:57Z INFO  sis_testing::phase7_ai_ops::model_lifecycle]     ✅ Model registration: PASSED
[2025-11-16T19:23:57Z INFO  sis_testing::phase7_ai_ops::model_lifecycle]   Testing model hot-swap...
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --id model-v1' timeout=45s
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:23:57Z INFO  sis_testing::phase7_ai_ops::shadow_mode]     ✅ Shadow deployment: PASSED
[2025-11-16T19:23:57Z INFO  sis_testing::phase7_ai_ops::shadow_mode]   Testing canary traffic routing (10%)...
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-traffic --percent 10' timeout=45s
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test prompt'' timeout=45s
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:23:57Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:23:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl audit last 100' timeout=45s
[2025-11-16T19:23:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:23:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:23:58Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:23:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:23:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:23:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:23:58Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:00Z INFO  sis_testing::phase7_ai_ops::decision_traces]     ✅ Decision trace collection: PASSED
[2025-11-16T19:24:00Z INFO  sis_testing::phase7_ai_ops::decision_traces]   Testing decision buffer management...
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl on' timeout=45s
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl export-traces' timeout=45s
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl swap --from model-v1 --to model-v2' timeout=45s
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:00Z INFO  sis_testing::phase7_ai_ops::shadow_mode]     ✅ Canary routing (10%): PASSED
[2025-11-16T19:24:00Z INFO  sis_testing::phase7_ai_ops::shadow_mode]   Testing A/B comparison...
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-traffic --percent 50' timeout=45s
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:05Z WARN  sis_testing::phase7_ai_ops::model_lifecycle]     ⚠️  Hot-swap took 4717ms (target: <500ms)
[2025-11-16T19:24:05Z WARN  sis_testing::phase7_ai_ops::model_lifecycle]     ❌ Model hot-swap: FAILED
[2025-11-16T19:24:05Z INFO  sis_testing::phase7_ai_ops::model_lifecycle]   Testing model rollback...
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl load --id model-v2' timeout=45s
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:05Z INFO  sis_testing::phase7_ai_ops::otel_exporter]     ✅ Span creation: PASSED
[2025-11-16T19:24:05Z INFO  sis_testing::phase7_ai_ops::otel_exporter]   Testing context propagation...
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 10' timeout=45s
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 2000' timeout=45s
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-compare' timeout=45s
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test'' timeout=45s
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl rollback --to model-v1' timeout=45s
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl audit stats' timeout=45s
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:05Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:13Z WARN  sis_testing::phase7_ai_ops::shadow_mode]     ❌ A/B comparison: FAILED
[2025-11-16T19:24:13Z INFO  sis_testing::phase7_ai_ops::shadow_mode]   Testing shadow promotion...
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-promote' timeout=45s
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:13Z WARN  sis_testing::phase7_ai_ops::decision_traces]     ❌ Buffer management: FAILED
[2025-11-16T19:24:13Z INFO  sis_testing::phase7_ai_ops::decision_traces]   Testing decision export...
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl export-decisions --format json --output /tmp/decisions.json' timeout=45s
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:13Z WARN  sis_testing::phase7_ai_ops::model_lifecycle]     ⚠️  Rollback took 7984ms (target: <200ms)
[2025-11-16T19:24:13Z WARN  sis_testing::phase7_ai_ops::model_lifecycle]     ❌ Model rollback: FAILED
[2025-11-16T19:24:13Z INFO  sis_testing::phase7_ai_ops::model_lifecycle]   Testing multi-model management...
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-1 --size 288 --ctx 2048' timeout=45s
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl export-traces' timeout=45s
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:13Z INFO  sis_testing::phase7_ai_ops::shadow_mode]     ✅ Shadow promotion: PASSED
[2025-11-16T19:24:13Z INFO  sis_testing::phase7_ai_ops::shadow_mode] Shadow Mode Tests: 3/4 passed (75%)
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:13Z INFO  sis_testing::phase7_ai_ops::decision_traces]     ✅ Decision export: PASSED
[2025-11-16T19:24:13Z INFO  sis_testing::phase7_ai_ops::decision_traces]   Testing decision replay...
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl export-decisions --format json --output /tmp/decisions.json' timeout=45s
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-2 --size 320 --ctx 2048' timeout=45s
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:13Z WARN  sis_testing::phase7_ai_ops::otel_exporter]     ❌ Context propagation: FAILED
[2025-11-16T19:24:13Z INFO  sis_testing::phase7_ai_ops::otel_exporter]   Testing batch export performance...
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 0'' timeout=45s
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-3 --size 352 --ctx 2048' timeout=45s
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl replay-decisions --input /tmp/decisions.json' timeout=45s
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 1'' timeout=45s
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:13Z INFO  sis_testing::phase7_ai_ops::decision_traces]     ✅ Decision replay: PASSED
[2025-11-16T19:24:13Z INFO  sis_testing::phase7_ai_ops::decision_traces] Decision Traces Tests: 3/4 passed (75%)
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-4 --size 384 --ctx 2048' timeout=45s
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 2'' timeout=45s
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-5 --size 416 --ctx 2048' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 3'' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-6 --size 448 --ctx 2048' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 4'' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-7 --size 480 --ctx 2048' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 5'' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-8 --size 512 --ctx 2048' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 6'' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 7'' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-9 --size 544 --ctx 2048' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 8'' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 9'' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 10'' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-10 --size 576 --ctx 2048' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 11'' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl list' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 12'' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z WARN  sis_testing::phase7_ai_ops::model_lifecycle]     ⚠️  List took 102ms for 10 models (target: <50ms)
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl query --id model-5' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 13'' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z WARN  sis_testing::phase7_ai_ops::model_lifecycle]     ⚠️  Query took 103ms (target: <10ms)
[2025-11-16T19:24:14Z WARN  sis_testing::phase7_ai_ops::model_lifecycle]     ❌ Multi-model management: FAILED
[2025-11-16T19:24:14Z INFO  sis_testing::phase7_ai_ops::model_lifecycle] Model Lifecycle Tests: 1/4 passed (25%)
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 14'' timeout=45s
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 15'' timeout=45s
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 16'' timeout=45s
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 17'' timeout=45s
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 18'' timeout=45s
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 19'' timeout=45s
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 20'' timeout=45s
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 21'' timeout=45s
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 22'' timeout=45s
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 23'' timeout=45s
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 24'' timeout=45s
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:15Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 25'' timeout=45s
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 26'' timeout=45s
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 27'' timeout=45s
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 28'' timeout=45s
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 29'' timeout=45s
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 30'' timeout=45s
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 31'' timeout=45s
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 32'' timeout=45s
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 33'' timeout=45s
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 34'' timeout=45s
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:16Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 35'' timeout=45s
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 36'' timeout=45s
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 37'' timeout=45s
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 38'' timeout=45s
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 39'' timeout=45s
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 40'' timeout=45s
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 41'' timeout=45s
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 42'' timeout=45s
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 43'' timeout=45s
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 44'' timeout=45s
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 45'' timeout=45s
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 46'' timeout=45s
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 47'' timeout=45s
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 48'' timeout=45s
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 49'' timeout=45s
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 50'' timeout=45s
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 51'' timeout=45s
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 52'' timeout=45s
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 53'' timeout=45s
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:18Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 54'' timeout=45s
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 55'' timeout=45s
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 56'' timeout=45s
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 57'' timeout=45s
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 58'' timeout=45s
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 59'' timeout=45s
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 60'' timeout=45s
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 61'' timeout=45s
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 62'' timeout=45s
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 63'' timeout=45s
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:19Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 64'' timeout=45s
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 65'' timeout=45s
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 66'' timeout=45s
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 67'' timeout=45s
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 68'' timeout=45s
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 69'' timeout=45s
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 70'' timeout=45s
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 71'' timeout=45s
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 72'' timeout=45s
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:20Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 73'' timeout=45s
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 74'' timeout=45s
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 75'' timeout=45s
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 76'' timeout=45s
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 77'' timeout=45s
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 78'' timeout=45s
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 79'' timeout=45s
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 80'' timeout=45s
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 81'' timeout=45s
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 82'' timeout=45s
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:21Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 83'' timeout=45s
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 84'' timeout=45s
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 85'' timeout=45s
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 86'' timeout=45s
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 87'' timeout=45s
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 88'' timeout=45s
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 89'' timeout=45s
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 90'' timeout=45s
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 91'' timeout=45s
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:22Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 92'' timeout=45s
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 93'' timeout=45s
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 94'' timeout=45s
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 95'' timeout=45s
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 96'' timeout=45s
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 97'' timeout=45s
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 98'' timeout=45s
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llminfer 'test 99'' timeout=45s
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl export-traces' timeout=45s
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:23Z INFO  sis_testing::phase7_ai_ops::otel_exporter]     ✅ Batch export: PASSED
[2025-11-16T19:24:23Z INFO  sis_testing::phase7_ai_ops::otel_exporter] OTel Exporter Tests: 3/4 passed (75%)
[2025-11-16T19:24:23Z INFO  sis_testing::phase7_ai_ops::integration_tests] Running Phase 7 Integration Tests...
[2025-11-16T19:24:23Z INFO  sis_testing::phase7_ai_ops::integration_tests]   Testing complete AI Ops workflow...
[2025-11-16T19:24:23Z INFO  sis_testing::phase7_ai_ops::integration_tests]     Step 1: Register model-v2
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl register --id model-v2 --size 1024 --ctx 4096' timeout=45s
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:23Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:24Z INFO  sis_testing::phase7_ai_ops::integration_tests]     Step 2: Deploy shadow agent
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-deploy --id model-v2 --traffic 0' timeout=45s
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:24Z INFO  sis_testing::phase7_ai_ops::integration_tests]     Step 3: Enable OpenTelemetry tracing
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl enable-tracing' timeout=45s
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:24Z INFO  sis_testing::phase7_ai_ops::integration_tests]     Step 4: Canary rollout (10% → 50%)
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-traffic --percent 10' timeout=45s
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-traffic --percent 50' timeout=45s
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:24Z INFO  sis_testing::phase7_ai_ops::integration_tests]     Step 5: A/B performance comparison
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-compare' timeout=45s
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:24Z INFO  sis_testing::phase7_ai_ops::integration_tests]     Step 6: Shadow promotion/retirement
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='llmctl shadow-promote' timeout=45s
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:24Z INFO  sis_testing::phase7_ai_ops::integration_tests]     Step 7: Export observability data
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='otelctl export-traces --output /tmp/traces.json' timeout=45s
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl export-decisions --output /tmp/decisions.json' timeout=45s
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:24:24Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:24:25Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T19:24:25Z INFO  sis_testing::phase7_ai_ops::integration_tests]     ✅ Complete AI Ops workflow: PASSED (100% success)
[2025-11-16T19:24:25Z INFO  sis_testing::phase7_ai_ops::integration_tests] Phase 7 Integration Tests: 1/1 passed
[2025-11-16T19:24:25Z INFO  sis_testing::phase7_ai_ops] ✅ Phase 7 validation complete: 80.0% (4/5 subsystems passed)
[2025-11-16T19:24:25Z INFO  sis_testing::reporting] Generating comprehensive industry-grade validation report
[2025-11-16T19:24:25Z INFO  sis_testing::reporting::analytics] Generating comprehensive analytics report
[2025-11-16T19:24:25Z INFO  sis_testing::reporting] JSON report written to: target/testing/validation_report.json
[2025-11-16T19:24:25Z INFO  sis_testing::reporting] Analytics JSON report written to: target/testing/analytics_report.json
[2025-11-16T19:24:25Z INFO  sis_testing::reporting::visualization] Generating interactive visualization dashboard
[2025-11-16T19:24:25Z INFO  sis_testing::reporting] Interactive dashboard written to: target/testing/interactive_dashboard.html
[2025-11-16T19:24:25Z INFO  sis_testing::reporting] HTML dashboard written to: target/testing/dashboard.html
[2025-11-16T19:24:25Z INFO  sis_testing::reporting] Executive summary written to: target/testing/executive_summary.md
[2025-11-16T19:24:25Z INFO  sis_testing::reporting] Technical report written to: target/testing/technical_report.md
[2025-11-16T19:24:25Z INFO  sis_testing::reporting] Performance charts placeholder written to: target/testing/performance_charts.svg
[2025-11-16T19:24:25Z INFO  sis_testing::reporting] Comprehensive industry-grade report generated in: target/testing
[2025-11-16T19:24:25Z WARN  sis_testing] Cannot shutdown QEMU: Arc has multiple owners
[2025-11-16T19:24:25Z INFO  sis_test_runner] 
[2025-11-16T19:24:25Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-16T19:24:25Z INFO  sis_test_runner] ║          SIS KERNEL COMPREHENSIVE TEST VALIDATION REPORT          ║
[2025-11-16T19:24:25Z INFO  sis_test_runner] ╚═══════════════════════════════════════════════════════════════════╝
[2025-11-16T19:24:25Z INFO  sis_test_runner] 
[2025-11-16T19:24:25Z INFO  sis_test_runner]   Status: █ PRODUCTION READY
[2025-11-16T19:24:25Z INFO  sis_test_runner]   Overall Score: 100.0%
[2025-11-16T19:24:25Z INFO  sis_test_runner]   Test Results: 1 PASS / 0 FAIL / 1 TOTAL
[2025-11-16T19:24:25Z INFO  sis_test_runner] 
[2025-11-16T19:24:25Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T19:24:25Z INFO  sis_test_runner] │ CORE SYSTEM COVERAGE                                            │
[2025-11-16T19:24:25Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T19:24:25Z INFO  sis_test_runner] │  Performance:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:24:25Z INFO  sis_test_runner] │  Correctness:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:24:25Z INFO  sis_test_runner] │  Security:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:24:25Z INFO  sis_test_runner] │  Distributed:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:24:25Z INFO  sis_test_runner] │  AI Validation:      0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:24:25Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T19:24:25Z INFO  sis_test_runner] 
[2025-11-16T19:24:25Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T19:24:25Z INFO  sis_test_runner] │ PHASE IMPLEMENTATION PROGRESS                                   │
[2025-11-16T19:24:25Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T19:24:25Z INFO  sis_test_runner] │  Phase 1 - AI-Native Dataflow:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:24:25Z INFO  sis_test_runner] │  Phase 2 - AI Governance:                0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:24:25Z INFO  sis_test_runner] │  Phase 3 - Temporal Isolation:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:24:25Z INFO  sis_test_runner] │  Phase 5 - UX Safety:                    0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:24:25Z INFO  sis_test_runner] │  Phase 6 - Web GUI Management:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:24:25Z INFO  sis_test_runner] │  Phase 7 - AI Operations:               80.0%  ██████████████████░░░░░
[2025-11-16T19:24:25Z INFO  sis_test_runner] │  Phase 8 - Performance Optimization:     0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:24:25Z INFO  sis_test_runner] │  Phase 9 - Agentic Platform:             0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:24:25Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T19:24:25Z INFO  sis_test_runner] 
[2025-11-16T19:24:25Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T19:24:25Z INFO  sis_test_runner] │ DETAILED VALIDATION RESULTS                                     │
[2025-11-16T19:24:25Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T19:24:25Z INFO  sis_test_runner] │ ✓ PASS
[2025-11-16T19:24:25Z INFO  sis_test_runner] │   Test: Phase 7: AI Operations Platform
[2025-11-16T19:24:25Z INFO  sis_test_runner] │   Target: ≥75% pass rate | Measured: 80.0%
[2025-11-16T19:24:25Z INFO  sis_test_runner] │   Industry Benchmark: Industry standard: MLOps 50-70%
[2025-11-16T19:24:25Z INFO  sis_test_runner] │
[2025-11-16T19:24:25Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T19:24:25Z INFO  sis_test_runner] 
[2025-11-16T19:24:25Z INFO  sis_test_runner] 📊 Reports generated in: target/testing/
[2025-11-16T19:24:25Z INFO  sis_test_runner] 🌐 View dashboard: target/testing/dashboard.html
[2025-11-16T19:24:25Z INFO  sis_test_runner] 
[2025-11-16T19:24:25Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-16T19:24:25Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-16T19:24:25Z INFO  sis_test_runner] ║  ✓ SUCCESS: SIS Kernel meets industry standards for production   ║
[2025-11-16T19:24:25Z INFO  sis_test_runner] ║    deployment and is ready for production use.                   ║
[2025-11-16T19:24:25Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-16T19:24:25Z INFO  sis_test_runner] ╚═══════════════════════════════════════════════════════════════════╝
amoljassal@Amols-Mac-mini sis-kernel % 

amoljassal@Amols-Mac-mini sis-kernel % SIS_FEATURES="ai-ops,bringup,crypto-real,decision-traces,default,deterministic,graphctl-framed,llm,model-lifecycle,otel,shadow-mode,agentsys" cargo run -p sis-testing --release -- --phase 9
warning: profile package spec `bootloader` in profile `dev` did not match any packages
warning: profile package spec `bootloader_api` in profile `dev` did not match any packages
    Finished `release` profile [optimized] target(s) in 0.14s
     Running `target/release/sis-test-runner --phase 9`
[2025-11-16T18:51:27Z INFO  sis_test_runner] SIS Kernel Industry-Grade Test Suite
[2025-11-16T18:51:27Z INFO  sis_test_runner] ====================================
[2025-11-16T18:51:27Z INFO  sis_test_runner] Mode: default (single QEMU node, moderate iterations)
[2025-11-16T18:51:27Z INFO  sis_test_runner] Test Configuration:
[2025-11-16T18:51:27Z INFO  sis_test_runner]   QEMU Nodes: 1
[2025-11-16T18:51:27Z INFO  sis_test_runner]   Duration: 600s
[2025-11-16T18:51:27Z INFO  sis_test_runner]   Performance Iterations: 2000
[2025-11-16T18:51:27Z INFO  sis_test_runner]   Statistical Confidence: 99.0%
[2025-11-16T18:51:27Z INFO  sis_test_runner]   Output Directory: target/testing
[2025-11-16T18:51:27Z INFO  sis_test_runner]   Parallel Execution: true
[2025-11-16T18:51:27Z INFO  sis_test_runner] Initializing QEMU runtime for kernel validation...
[2025-11-16T18:51:27Z INFO  sis_testing] Initializing QEMU runtime for comprehensive kernel testing
[2025-11-16T18:51:27Z INFO  sis_testing::qemu_runtime] Building SIS kernel for QEMU testing
[2025-11-16T18:51:27Z INFO  sis_testing::qemu_runtime] Building UEFI bootloader...
[2025-11-16T18:51:27Z INFO  sis_testing::qemu_runtime] Building kernel with features: ai-ops,bringup,crypto-real,decision-traces,default,deterministic,graphctl-framed,llm,model-lifecycle,otel,shadow-mode,agentsys
[2025-11-16T18:51:27Z INFO  sis_testing::qemu_runtime] SIS kernel and UEFI bootloader built successfully
[2025-11-16T18:51:27Z INFO  sis_testing::qemu_runtime] Preparing ESP directories for 1 QEMU instances
[2025-11-16T18:51:27Z INFO  sis_testing::qemu_runtime] ESP directories prepared for all instances
[2025-11-16T18:51:27Z INFO  sis_testing::qemu_runtime] Launching QEMU cluster with 1 nodes
[2025-11-16T18:51:27Z INFO  sis_testing::qemu_runtime] Launching QEMU instance 0 on ports 7000/7100/7200
[2025-11-16T18:51:27Z INFO  sis_testing::qemu_runtime] Instance 0 launched (serial log: target/testing/serial-node0.log)
[2025-11-16T18:51:30Z INFO  sis_testing::qemu_runtime] All QEMU instances launched successfully
[2025-11-16T18:51:30Z INFO  sis_testing::qemu_runtime] Waiting for instance 0 to boot (timeout: 180s)
[2025-11-16T18:51:30Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail):  Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[0
    [QEMU-OUT] 1;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T18:51:32Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail):  Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[0
    [QEMU-OUT] 1;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T18:51:34Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail):  Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[0
    [QEMU-OUT] 1;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T18:51:36Z INFO  sis_testing::qemu_runtime] Instance 0 booted successfully (detected via serial log)
[2025-11-16T18:51:36Z INFO  sis_testing] QEMU runtime initialized with 1 node(s); boot detected via serial log
[2025-11-16T18:51:36Z INFO  sis_test_runner] QEMU runtime initialized successfully - running real kernel tests
[2025-11-16T18:51:36Z INFO  sis_test_runner] Phase selection detected: running Phase 9 only
[2025-11-16T18:51:36Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-16T18:51:36Z INFO  sis_testing] Phase 1-9 test suites initialized successfully
[2025-11-16T18:51:36Z INFO  sis_testing] Starting single-phase validation for Phase 9
[2025-11-16T18:51:36Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-16T18:51:36Z INFO  sis_testing] Phase 1-9 test suites initialized successfully
[2025-11-16T18:51:36Z INFO  sis_testing::phase9_agentic] 🚀 Starting Phase 9: Agentic Platform validation
[2025-11-16T18:51:36Z INFO  sis_testing::phase9_agentic::agentsys_protocol_tests] 🧪 Running AgentSys Protocol Tests...
[2025-11-16T18:51:36Z INFO  sis_testing::phase9_agentic::agentsys_protocol_tests]   Testing FS_LIST operation...
[2025-11-16T18:51:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys test-fs-list' timeout=45s
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:52:59Z WARN  sis_testing::phase9_agentic::agentsys_protocol_tests]   ❌ FS_LIST test failed: [QEMU-OUT] 
    [QEMU-OUT] sis> 
    [QEMU-OUT] sis> agentsys test-fs-list
    [QEMU-OUT] METRIC nn_infer_us=678
    [QEMU-OUT] METRIC nn_infer_count=1
    [QEMU-OUT] [AgentSys] Testing FS_LIST on /tmp/
    [QEMU-OUT] METRIC agentsys_calls_total=1
    [QEMU-OUT] METRIC agentsys_fs_list=1
    [QEMU-OUT] METRIC agentsys_policy_allows=1
    [QEMU-OUT] [FS] Open error: No such file or directory
    [QEMU-OUT] [AUDIT] agent=65535 op=0x30 result=DENY
    [QEMU-OUT] METRIC agentsys_audit_events=1
    [QEMU-OUT] METRIC agentsys_denies_total=1
    [QEMU-OUT] [AUDIT] agent=65535 op=0x30 result=DENY
    [QEMU-OUT] METRIC agentsys_audit_events=1
    [QEMU-OUT] METRIC agentsys_denies_total=1
    [QEMU-OUT] [AgentSys] Test FAILED: Unknown
    [QEMU-OUT] CMD_DONE
    [QEMU-OUT] sis> 
    [QEMU-OUT] sis> 
    
[2025-11-16T18:52:59Z INFO  sis_testing::phase9_agentic::agentsys_protocol_tests]   Testing AUDIO_PLAY operation...
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys test-audio-play' timeout=45s
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:52:59Z INFO  sis_testing::phase9_agentic::agentsys_protocol_tests]   ✅ AUDIO_PLAY test passed
[2025-11-16T18:52:59Z INFO  sis_testing::phase9_agentic::agentsys_protocol_tests]   Testing invalid opcode handling...
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys status' timeout=45s
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:52:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:00Z WARN  sis_testing::phase9_agentic::agentsys_protocol_tests]   ❌ Invalid opcode handling test failed: [QEMU-OUT] 
    [QEMU-OUT] sis> 
    [QEMU-OUT] sis> agentsys status
    [QEMU-OUT] METRIC nn_infer_us=47
    [QEMU-OUT] METRIC nn_infer_count=3
    [QEMU-OUT] 
    [QEMU-OUT] Agent Supervision Module Status
    [QEMU-OUT] ================================
    [QEMU-OUT] 
    [QEMU-OUT] Subsystems:
    [QEMU-OUT]   ✓ Agent Supervisor      (initialized)
    [QEMU-OUT]   ✓ Telemetry Aggregator  (initialized)
    [QEMU-OUT]   ✓ Fault Detector        (initialized)
    [QEMU-OUT]   ✓ Policy Controller     (initialized)
    [QEMU-OUT]   ✓ Compliance Tracker    (initialized)
    [QEMU-OUT]   ✓ Resource Monitor      (initialized)
    [QEMU-OUT]   ✓ Dependency Graph      (initialized)
    [QEMU-OUT]   ✓ System Profiler       (initialized)
    [QEMU-OUT] 
    [QEMU-OUT] Quick Stats:
    [QEMU-OUT]   Active Agents:     0
    [QEMU-OUT]   Total Spawns:      0
    [QEMU-OUT]   Total Exits:       0
    [QEMU-OUT]   Total Faults:      0
    [QEMU-OUT] 
    [QEMU-OUT] System Health: Healthy
    [QEMU-OUT] 
    [QEMU-OUT] sis> 
    [QEMU-OUT] sis> 
    
[2025-11-16T18:53:00Z INFO  sis_testing::phase9_agentic::agentsys_protocol_tests]   Testing agentsys status command...
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys status' timeout=45s
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:00Z WARN  sis_testing::phase9_agentic::agentsys_protocol_tests]   ❌ Status command test failed: [QEMU-OUT] [TIMER] Timer running silently (use 'autoctl status' to check)
    [QEMU-OUT] [TIMER] Timer freq=62500000 Hz
    [QEMU-OUT] [TIMER] Rearming with 500ms interval (31250000 cycles)
    [QEMU-OUT] [TIMER] Calling autonomous_decision_tick()
    [QEMU-OUT] [AUTONOMY] Starting decision tick at timestamp 92249072
    [QEMU-OUT] [AUTONOMY] Telemetry: mem_pressure=0 deadline_misses=0
    [QEMU-OUT] METRIC nn_infer_us=276
    [QEMU-OUT] METRIC nn_infer_count=2
    [QEMU-OUT] [AUTONOMY] Meta-agent directives: [0, 0, 0] confidence=0
    [QEMU-OUT] [AUTONOMY] Low confidence (0 < 600), deferring action: Model still initializing (< 10 decisions recorded)
    [QEMU-OUT] 
    [QEMU-OUT] sis> 
    [QEMU-OUT] sis> agentsys status
    [QEMU-OUT] METRIC nn_infer_us=15
    [QEMU-OUT] METRIC nn_infer_count=4
    [QEMU-OUT] 
    [QEMU-OUT] Agent Supervision Module Status
    [QEMU-OUT] ================================
    [QEMU-OUT] 
    [QEMU-OUT] Subsystems:
    [QEMU-OUT]   ✓ Agent Supervisor      (initialized)
    [QEMU-OUT]   ✓ Telemetry Aggregator  (initialized)
    [QEMU-OUT]   ✓ Fault Detector        (initialized)
    [QEMU-OUT]   ✓ Policy Controller     (initialized)
    [QEMU-OUT]   ✓ Compliance Tracker    (initialized)
    [QEMU-OUT]   ✓ Resource Monitor      (initialized)
    [QEMU-OUT]   ✓ Dependency Graph      (initialized)
    [QEMU-OUT]   ✓ System Profiler       (initialized)
    [QEMU-OUT] 
    [QEMU-OUT] Quick Stats:
    [QEMU-OUT]   Active Agents:     0
    [QEMU-OUT]   Total Spawns:      0
    [QEMU-OUT]   Total Exits:       0
    [QEMU-OUT]   Total Faults:      0
    [QEMU-OUT] 
    [QEMU-OUT] System Health: Healthy
    [QEMU-OUT] 
    [QEMU-OUT] sis> 
    [QEMU-OUT] sis> 
    
[2025-11-16T18:53:00Z INFO  sis_testing::phase9_agentic::agentsys_protocol_tests]   Testing memory overhead...
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys status' timeout=45s
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:00Z WARN  sis_testing::phase9_agentic::agentsys_protocol_tests]   ❌ Memory overhead check failed: AgentSys not operational
[2025-11-16T18:53:00Z INFO  sis_testing::phase9_agentic::capability_enforcement_tests] 🔒 Running Capability Enforcement Tests...
[2025-11-16T18:53:00Z INFO  sis_testing::phase9_agentic::capability_enforcement_tests]   Testing unauthorized access denial...
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys test-fs-list' timeout=45s
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys audit' timeout=45s
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:00Z INFO  sis_testing::phase9_agentic::capability_enforcement_tests]   ✅ Unauthorized access denial test passed
[2025-11-16T18:53:00Z INFO  sis_testing::phase9_agentic::capability_enforcement_tests]   Testing scope restrictions...
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys test-fs-list' timeout=45s
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:00Z INFO  sis_testing::phase9_agentic::capability_enforcement_tests]   ✅ Scope restriction test passed
[2025-11-16T18:53:00Z INFO  sis_testing::phase9_agentic::capability_enforcement_tests]   Testing multiple agent support...
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys status' timeout=45s
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:00Z WARN  sis_testing::phase9_agentic::capability_enforcement_tests]   ❌ Multiple agent support test failed: [QEMU-OUT] 
    [QEMU-OUT] sis> 
    [QEMU-OUT] sis> agentsys status
    [QEMU-OUT] METRIC nn_infer_us=116
    [QEMU-OUT] METRIC nn_infer_count=9
    [QEMU-OUT] 
    [QEMU-OUT] Agent Supervision Module Status
    [QEMU-OUT] ================================
    [QEMU-OUT] 
    [QEMU-OUT] Subsystems:
    [QEMU-OUT]   ✓ Agent Supervisor      (initialized)
    [QEMU-OUT]   ✓ Telemetry Aggregator  (initialized)
    [QEMU-OUT]   ✓ Fault Detector        (initialized)
    [QEMU-OUT]   ✓ Policy Controller     (initialized)
    [QEMU-OUT]   ✓ Compliance Tracker    (initialized)
    [QEMU-OUT]   ✓ Resource Monitor      (initialized)
    [QEMU-OUT]   ✓ Dependency Graph      (initialized)
    [QEMU-OUT]   ✓ System Profiler       (initialized)
    [QEMU-OUT] 
    [QEMU-OUT] Quick Stats:
    [QEMU-OUT]   Active Agents:     0
    [QEMU-OUT]   Total Spawns:      0
    [QEMU-OUT]   Total Exits:       0
    [QEMU-OUT]   Total Faults:      0
    [QEMU-OUT] 
    [QEMU-OUT] System Health: Healthy
    [QEMU-OUT] 
    [QEMU-OUT] sis> 
    [QEMU-OUT] sis> 
    [QEMU-OUT] [TIMER] Rearming with 500ms interval (31250000 cycles)
    [QEMU-OUT] [TIMER] Calling autonomous_decision_tick()
    [QEMU-OUT] [AUTONOMY] Starting decision tick at timestamp 92753828
    [QEMU-OUT] [AUTONOMY] Telemetry: mem_pressure=0 deadline_misses=0
    [QEMU-OUT] METRIC nn_infer_us=82
    [QEMU-OUT] METRIC nn_infer_count=3
    [QEMU-OUT] [AUTONOMY] Meta-agent directives: [0, 0, 0] confidence=0
    [QEMU-OUT] [AUTONOMY] Low confidence (0 < 600), deferring action: Model still initializing (< 10 decisions recorded)
    
[2025-11-16T18:53:00Z INFO  sis_testing::phase9_agentic::audit_validation_tests] 📋 Running Audit Validation Tests...
[2025-11-16T18:53:00Z INFO  sis_testing::phase9_agentic::audit_validation_tests]   Testing operation logging...
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys test-fs-list' timeout=45s
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys test-audio-play' timeout=45s
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys audit' timeout=45s
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:00Z INFO  sis_testing::phase9_agentic::audit_validation_tests]   ✅ Operation logging test passed
[2025-11-16T18:53:00Z INFO  sis_testing::phase9_agentic::audit_validation_tests]   Testing audit dump...
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys audit' timeout=45s
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:00Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:01Z WARN  sis_testing::phase9_agentic::audit_validation_tests]   ❌ Audit dump test failed: [QEMU-OUT] 
    [QEMU-OUT] sis> 
    
[2025-11-16T18:53:01Z INFO  sis_testing::phase9_agentic::audit_validation_tests]   Testing audit completeness...
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys status' timeout=45s
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys test-fs-list' timeout=45s
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys status' timeout=45s
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:01Z WARN  sis_testing::phase9_agentic::audit_validation_tests]   ❌ Audit completeness test failed
[2025-11-16T18:53:01Z INFO  sis_testing::phase9_agentic::asm_supervision_tests] 🧪 Starting ASM Supervision integration tests
[2025-11-16T18:53:01Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]   → TC-INT-LC-001: Testing agentsys status
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys status' timeout=45s
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:01Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]     ✓ Status command working correctly
[2025-11-16T18:53:01Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]   → TC-INT-LC-002: Testing agentsys list
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys list' timeout=45s
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:01Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]     ✓ List command working correctly
[2025-11-16T18:53:01Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]   → TC-INT-TM-001: Testing agentsys metrics
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys metrics 1' timeout=45s
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:01Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]     ✓ Metrics command working correctly
[2025-11-16T18:53:01Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]   → TC-INT-TM-002: Testing agentsys resources
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys resources 1' timeout=45s
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:01Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]     ✓ Resources command working correctly
[2025-11-16T18:53:01Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]   → TC-INT-TM-003: Testing agentsys telemetry
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys telemetry' timeout=45s
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:01Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]     ✓ Telemetry command working correctly
[2025-11-16T18:53:01Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]   → TC-INT-CP-001: Testing agentsys compliance
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys compliance' timeout=45s
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:01Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:02Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]     ✓ Compliance command working correctly
[2025-11-16T18:53:02Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]   → TC-INT-RM-002: Testing agentsys limits
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys limits 1' timeout=45s
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:02Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]     ✓ Limits command working correctly
[2025-11-16T18:53:02Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]   → TC-INT-DP-001: Testing agentsys deps
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys deps 1' timeout=45s
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:02Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]     ✓ Deps command working correctly
[2025-11-16T18:53:02Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]   → TC-INT-DP-002: Testing agentsys depgraph
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys depgraph' timeout=45s
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:02Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]     ✓ Depgraph command working correctly
[2025-11-16T18:53:02Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]   → TC-INT-PR-001: Testing agentsys profile
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys profile 1' timeout=45s
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:02Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]     ✓ Profile command working correctly
[2025-11-16T18:53:02Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]   → TC-INT-ST-002: Testing agentsys dump
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='agentsys dump' timeout=45s
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:53:02Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:53:02Z INFO  sis_testing::phase9_agentic::asm_supervision_tests]     ✓ Dump command working correctly
[2025-11-16T18:53:02Z INFO  sis_testing::phase9_agentic::asm_supervision_tests] ✅ ASM Supervision tests complete: 11/11 passed
[2025-11-16T18:53:02Z INFO  sis_testing::phase9_agentic] ✅ Phase 9 validation complete: 44.4% (4/9 tests passed)
[2025-11-16T18:53:02Z INFO  sis_testing::phase9_agentic]    ASM Supervision: 11/11 tests passed
[2025-11-16T18:53:02Z INFO  sis_testing::reporting] Generating comprehensive industry-grade validation report
[2025-11-16T18:53:02Z INFO  sis_testing::reporting::analytics] Generating comprehensive analytics report
[2025-11-16T18:53:02Z INFO  sis_testing::reporting] JSON report written to: target/testing/validation_report.json
[2025-11-16T18:53:02Z INFO  sis_testing::reporting] Analytics JSON report written to: target/testing/analytics_report.json
[2025-11-16T18:53:02Z INFO  sis_testing::reporting::visualization] Generating interactive visualization dashboard
[2025-11-16T18:53:02Z INFO  sis_testing::reporting] Interactive dashboard written to: target/testing/interactive_dashboard.html
[2025-11-16T18:53:02Z INFO  sis_testing::reporting] HTML dashboard written to: target/testing/dashboard.html
[2025-11-16T18:53:02Z INFO  sis_testing::reporting] Executive summary written to: target/testing/executive_summary.md
[2025-11-16T18:53:02Z INFO  sis_testing::reporting] Technical report written to: target/testing/technical_report.md
[2025-11-16T18:53:02Z INFO  sis_testing::reporting] Performance charts placeholder written to: target/testing/performance_charts.svg
[2025-11-16T18:53:02Z INFO  sis_testing::reporting] Comprehensive industry-grade report generated in: target/testing
[2025-11-16T18:53:02Z WARN  sis_testing] Cannot shutdown QEMU: Arc has multiple owners
[2025-11-16T18:53:02Z INFO  sis_test_runner] 
[2025-11-16T18:53:02Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-16T18:53:02Z INFO  sis_test_runner] ║          SIS KERNEL COMPREHENSIVE TEST VALIDATION REPORT          ║
[2025-11-16T18:53:02Z INFO  sis_test_runner] ╚═══════════════════════════════════════════════════════════════════╝
[2025-11-16T18:53:02Z INFO  sis_test_runner] 
[2025-11-16T18:53:02Z INFO  sis_test_runner]   Status: █ NEEDS IMPROVEMENT
[2025-11-16T18:53:02Z INFO  sis_test_runner]   Overall Score: 0.0%
[2025-11-16T18:53:02Z INFO  sis_test_runner]   Test Results: 0 PASS / 1 FAIL / 1 TOTAL
[2025-11-16T18:53:02Z INFO  sis_test_runner] 
[2025-11-16T18:53:02Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:53:02Z INFO  sis_test_runner] │ CORE SYSTEM COVERAGE                                            │
[2025-11-16T18:53:02Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:53:02Z INFO  sis_test_runner] │  Performance:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:53:02Z INFO  sis_test_runner] │  Correctness:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:53:02Z INFO  sis_test_runner] │  Security:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:53:02Z INFO  sis_test_runner] │  Distributed:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:53:02Z INFO  sis_test_runner] │  AI Validation:      0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:53:02Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:53:02Z INFO  sis_test_runner] 
[2025-11-16T18:53:02Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:53:02Z INFO  sis_test_runner] │ PHASE IMPLEMENTATION PROGRESS                                   │
[2025-11-16T18:53:02Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:53:02Z INFO  sis_test_runner] │  Phase 1 - AI-Native Dataflow:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:53:02Z INFO  sis_test_runner] │  Phase 2 - AI Governance:                0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:53:02Z INFO  sis_test_runner] │  Phase 3 - Temporal Isolation:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:53:02Z INFO  sis_test_runner] │  Phase 5 - UX Safety:                    0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:53:02Z INFO  sis_test_runner] │  Phase 6 - Web GUI Management:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:53:02Z INFO  sis_test_runner] │  Phase 7 - AI Operations:                0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:53:02Z INFO  sis_test_runner] │  Phase 8 - Performance Optimization:     0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T18:53:02Z INFO  sis_test_runner] │  Phase 9 - Agentic Platform:            44.4%  ██████████░░░░░░░░░░░░░
[2025-11-16T18:53:02Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:53:02Z INFO  sis_test_runner] 
[2025-11-16T18:53:02Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T18:53:02Z INFO  sis_test_runner] │ DETAILED VALIDATION RESULTS                                     │
[2025-11-16T18:53:02Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T18:53:02Z INFO  sis_test_runner] │ ✗ FAIL
[2025-11-16T18:53:02Z INFO  sis_test_runner] │   Test: Phase 9: Agentic Platform
[2025-11-16T18:53:02Z INFO  sis_test_runner] │   Target: ≥75% pass rate (100% target) | Measured: 44.4%
[2025-11-16T18:53:02Z INFO  sis_test_runner] │   Industry Benchmark: Industry standard: Agent systems 50-65%
[2025-11-16T18:53:02Z INFO  sis_test_runner] │
[2025-11-16T18:53:02Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T18:53:02Z INFO  sis_test_runner] 
[2025-11-16T18:53:02Z INFO  sis_test_runner] 📊 Reports generated in: target/testing/
[2025-11-16T18:53:02Z INFO  sis_test_runner] 🌐 View dashboard: target/testing/dashboard.html
[2025-11-16T18:53:02Z INFO  sis_test_runner] 
[2025-11-16T18:53:02Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-16T18:53:02Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-16T18:53:02Z INFO  sis_test_runner] ║  ✗ WARNING: SIS Kernel requires improvements before production  ║
[2025-11-16T18:53:02Z INFO  sis_test_runner] ║    readiness (0.0%). Review failed tests above.                ║
[2025-11-16T18:53:02Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-16T18:53:02Z INFO  sis_test_runner] ╚═══════════════════════════════════════════════════════════════════╝
amoljassal@Amols-Mac-mini sis-kernel % SIS_FEATURES="ai-ops,bringup,crypto-real,decision-traces,default,deterministic,graphctl-framed,llm,model-lifecycle,otel,shadow-mode,agentsys" cargo run -p sis-testing --release -- --phase 8
warning: profile package spec `bootloader` in profile `dev` did not match any packages
warning: profile package spec `bootloader_api` in profile `dev` did not match any packages
    Finished `release` profile [optimized] target(s) in 0.18s
     Running `target/release/sis-test-runner --phase 8`
[2025-11-16T18:54:37Z INFO  sis_test_runner] SIS Kernel Industry-Grade Test Suite
[2025-11-16T18:54:37Z INFO  sis_test_runner] ====================================
[2025-11-16T18:54:37Z INFO  sis_test_runner] Mode: default (single QEMU node, moderate iterations)
[2025-11-16T18:54:37Z INFO  sis_test_runner] Test Configuration:
[2025-11-16T18:54:37Z INFO  sis_test_runner]   QEMU Nodes: 1
[2025-11-16T18:54:37Z INFO  sis_test_runner]   Duration: 600s
[2025-11-16T18:54:37Z INFO  sis_test_runner]   Performance Iterations: 2000
[2025-11-16T18:54:37Z INFO  sis_test_runner]   Statistical Confidence: 99.0%
[2025-11-16T18:54:37Z INFO  sis_test_runner]   Output Directory: target/testing
[2025-11-16T18:54:37Z INFO  sis_test_runner]   Parallel Execution: true
[2025-11-16T18:54:37Z INFO  sis_test_runner] Initializing QEMU runtime for kernel validation...
[2025-11-16T18:54:37Z INFO  sis_testing] Initializing QEMU runtime for comprehensive kernel testing
[2025-11-16T18:54:37Z INFO  sis_testing::qemu_runtime] Building SIS kernel for QEMU testing
[2025-11-16T18:54:37Z INFO  sis_testing::qemu_runtime] Building UEFI bootloader...
[2025-11-16T18:54:37Z INFO  sis_testing::qemu_runtime] Building kernel with features: ai-ops,bringup,crypto-real,decision-traces,default,deterministic,graphctl-framed,llm,model-lifecycle,otel,shadow-mode,agentsys
[2025-11-16T18:54:37Z INFO  sis_testing::qemu_runtime] SIS kernel and UEFI bootloader built successfully
[2025-11-16T18:54:37Z INFO  sis_testing::qemu_runtime] Preparing ESP directories for 1 QEMU instances
[2025-11-16T18:54:37Z INFO  sis_testing::qemu_runtime] ESP directories prepared for all instances
[2025-11-16T18:54:37Z INFO  sis_testing::qemu_runtime] Launching QEMU cluster with 1 nodes
[2025-11-16T18:54:37Z INFO  sis_testing::qemu_runtime] Launching QEMU instance 0 on ports 7000/7100/7200
[2025-11-16T18:54:37Z INFO  sis_testing::qemu_runtime] Instance 0 launched (serial log: target/testing/serial-node0.log)
[2025-11-16T18:54:40Z INFO  sis_testing::qemu_runtime] All QEMU instances launched successfully
[2025-11-16T18:54:40Z INFO  sis_testing::qemu_runtime] Waiting for instance 0 to boot (timeout: 180s)
[2025-11-16T18:54:40Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T18:54:42Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T18:54:44Z INFO  sis_testing::qemu_runtime] Instance 0 boot output (tail): 
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] Tpm2GetCapabilityPcrs fail!
    [QEMU-OUT] Tpm2SubmitCommand - Tcg2 - Not Found
    [QEMU-OUT] [2J[01;01H[=3h[2J[01;01H[2J[01;01H[=3h[2J[01;01H
    
[2025-11-16T18:54:46Z INFO  sis_testing::qemu_runtime] Instance 0 booted successfully (detected via serial log)
[2025-11-16T18:54:46Z INFO  sis_testing] QEMU runtime initialized with 1 node(s); boot detected via serial log
[2025-11-16T18:54:46Z INFO  sis_test_runner] QEMU runtime initialized successfully - running real kernel tests
[2025-11-16T18:54:46Z INFO  sis_test_runner] Phase selection detected: running Phase 8 only
[2025-11-16T18:54:46Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-16T18:54:46Z INFO  sis_testing] Phase 1-9 test suites initialized successfully
[2025-11-16T18:54:46Z INFO  sis_testing] Starting single-phase validation for Phase 8
[2025-11-16T18:54:46Z INFO  sis_testing] Initializing Phase 1-8 test suites with serial log: target/testing/serial-node0.log
[2025-11-16T18:54:46Z INFO  sis_testing] Phase 1-9 test suites initialized successfully
[2025-11-16T18:54:46Z INFO  sis_testing::phase8_deterministic] 🚀 Starting Phase 8: Performance Optimization validation
[2025-11-16T18:54:46Z INFO  sis_testing::phase8_deterministic::cbs_edf_scheduler] Running CBS+EDF Scheduler Tests...
[2025-11-16T18:54:46Z INFO  sis_testing::phase8_deterministic::cbs_edf_scheduler]   Testing admission control...
[2025-11-16T18:54:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 10' timeout=45s
[2025-11-16T18:54:46Z INFO  sis_testing::phase8_deterministic::slab_allocator] Running Slab Allocator Tests...
[2025-11-16T18:54:46Z INFO  sis_testing::phase8_deterministic::slab_allocator]   Testing slab performance benchmarks...
[2025-11-16T18:54:46Z INFO  sis_testing::phase8_deterministic::slab_allocator]     ✅ Slab performance: PASSED
[2025-11-16T18:54:46Z INFO  sis_testing::phase8_deterministic::slab_allocator]   Testing slab vs linked-list comparison...
[2025-11-16T18:54:46Z INFO  sis_testing::phase8_deterministic::slab_allocator]     ✅ Slab comparison: PASSED
[2025-11-16T18:54:46Z INFO  sis_testing::phase8_deterministic::slab_allocator]   Testing slab cache efficiency...
[2025-11-16T18:54:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='memctl slab-stats' timeout=45s
[2025-11-16T18:54:46Z INFO  sis_testing::phase8_deterministic::adaptive_memory] Running Adaptive Memory Tests...
[2025-11-16T18:54:46Z INFO  sis_testing::phase8_deterministic::adaptive_memory]   Testing strategy switching...
[2025-11-16T18:54:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='memctl strategy status' timeout=45s
[2025-11-16T18:54:46Z INFO  sis_testing::phase8_deterministic::meta_agent] Running Meta-Agent Tests...
[2025-11-16T18:54:46Z INFO  sis_testing::phase8_deterministic::meta_agent]   Testing meta-agent inference...
[2025-11-16T18:54:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl on' timeout=45s
[2025-11-16T18:54:46Z INFO  sis_testing::phase8_deterministic::rate_limiting] Running Rate Limiting Tests...
[2025-11-16T18:54:46Z INFO  sis_testing::phase8_deterministic::rate_limiting]   Testing strategy change rate limiting...
[2025-11-16T18:54:46Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 5000' timeout=45s
[2025-11-16T18:56:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:17Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 64 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 64 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 64 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 64 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 64 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 10000000 100000000 100000000' timeout=45s
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:29Z INFO  sis_testing::phase8_deterministic::slab_allocator]     ✅ Cache efficiency: PASSED
[2025-11-16T18:56:29Z INFO  sis_testing::phase8_deterministic::slab_allocator] Slab Allocator Tests: 3/3 passed
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 1000' timeout=45s
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:29Z INFO  sis_testing::phase8_deterministic::rate_limiting]     ✅ Strategy change rate limit: PASSED
[2025-11-16T18:56:29Z INFO  sis_testing::phase8_deterministic::rate_limiting]   Testing meta-agent directive rate limiting...
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl on' timeout=45s
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 1000' timeout=45s
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 90000000 100000000 100000000' timeout=45s
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 5000' timeout=45s
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='memctl strategy status' timeout=45s
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl audit last 10' timeout=45s
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:33Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:36Z INFO  sis_testing::phase8_deterministic::adaptive_memory]     ✅ Strategy switching: PASSED
[2025-11-16T18:56:36Z INFO  sis_testing::phase8_deterministic::adaptive_memory]   Testing directive thresholds...
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl audit last 10' timeout=45s
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:36Z INFO  sis_testing::phase8_deterministic::meta_agent]     ✅ Meta-agent inference: PASSED
[2025-11-16T18:56:36Z INFO  sis_testing::phase8_deterministic::meta_agent]   Testing confidence thresholds...
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl audit last 50' timeout=45s
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:36Z WARN  sis_testing::phase8_deterministic::cbs_edf_scheduler]     ❌ Admission control: FAILED
[2025-11-16T18:56:36Z INFO  sis_testing::phase8_deterministic::cbs_edf_scheduler]   Testing deadline miss detection...
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 50000000 100000000 100000000' timeout=45s
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:36Z INFO  sis_testing::phase8_deterministic::rate_limiting]     ✅ Meta-agent directive rate limit: PASSED
[2025-11-16T18:56:36Z INFO  sis_testing::phase8_deterministic::rate_limiting]   Testing no output flooding...
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 5000' timeout=45s
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:36Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:37Z INFO  sis_testing::phase8_deterministic::adaptive_memory]     ✅ Directive thresholds: PASSED
[2025-11-16T18:56:37Z INFO  sis_testing::phase8_deterministic::adaptive_memory]   Testing oscillation detection...
[2025-11-16T18:56:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 2000' timeout=45s
[2025-11-16T18:56:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:37Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:37Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:43Z INFO  sis_testing::phase8_deterministic::rate_limiting]     ✅ No output flooding: PASSED
[2025-11-16T18:56:43Z INFO  sis_testing::phase8_deterministic::rate_limiting] Rate Limiting Tests: 3/3 passed
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=45s
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:43Z INFO  sis_testing::phase8_deterministic::meta_agent]     ✅ Confidence thresholds: PASSED
[2025-11-16T18:56:43Z INFO  sis_testing::phase8_deterministic::meta_agent]   Testing multi-subsystem directives...
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl audit last 10' timeout=45s
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command completed
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='memctl strategy history' timeout=45s
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:56:43Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=45s
[2025-11-16T18:57:28Z WARN  sis_testing::phase8_deterministic::adaptive_memory]     ❌ Oscillation detection: FAILED
[2025-11-16T18:57:28Z INFO  sis_testing::phase8_deterministic::adaptive_memory]   Testing rate-limited output...
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 1000' timeout=45s
[2025-11-16T18:57:28Z WARN  sis_testing::phase8_deterministic::meta_agent]     ❌ Multi-subsystem directives: FAILED
[2025-11-16T18:57:28Z INFO  sis_testing::phase8_deterministic::meta_agent]   Testing reward feedback...
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl audit last 50' timeout=45s
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:57:28Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:58:13Z WARN  sis_testing::phase8_deterministic::meta_agent]     ❌ Reward feedback: FAILED
[2025-11-16T18:58:13Z INFO  sis_testing::phase8_deterministic::meta_agent] Meta-Agent Tests: 2/4 passed
[2025-11-16T18:58:13Z INFO  sis_testing::phase8_deterministic::cbs_edf_scheduler]   Testing budget replenishment...
[2025-11-16T18:58:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 10000000 100000000 100000000' timeout=45s
[2025-11-16T18:58:13Z INFO  sis_testing::phase8_deterministic::adaptive_memory]     ✅ Rate-limited output: PASSED
[2025-11-16T18:58:13Z INFO  sis_testing::phase8_deterministic::adaptive_memory] Adaptive Memory Tests: 3/4 passed
[2025-11-16T18:58:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:58:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:58:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:58:13Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:58:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:58:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:58:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:58:13Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:58:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=45s
[2025-11-16T18:58:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:58:59Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:58:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:58:59Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:58:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:58:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:58:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:58:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T18:59:44Z INFO  sis_testing::phase8_deterministic::cbs_edf_scheduler]   Testing EDF priority scheduling...
[2025-11-16T18:59:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 3' timeout=45s
[2025-11-16T18:59:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T18:59:44Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:59:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T18:59:44Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T18:59:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T18:59:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T18:59:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T18:59:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:00:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 5000000 50000000 50000000' timeout=45s
[2025-11-16T19:00:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:00:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:00:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:00:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:00:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:00:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:00:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:00:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:01:14Z INFO  sis_testing::phase8_deterministic::cbs_edf_scheduler]   Testing graph integration...
[2025-11-16T19:01:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl create --num-operators 5' timeout=45s
[2025-11-16T19:01:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:01:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:01:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:01:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:01:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:01:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:01:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:01:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:01:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det on 10000000 100000000 100000000' timeout=45s
[2025-11-16T19:01:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:01:59Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:01:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:01:59Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:01:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:01:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:01:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:01:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:02:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl add-operator 1 --in none --out 0 --prio 10' timeout=45s
[2025-11-16T19:02:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:02:44Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:02:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:02:44Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:02:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:02:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:02:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:02:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:03:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='graphctl start 100' timeout=45s
[2025-11-16T19:03:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:03:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:03:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:03:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:03:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:03:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:03:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:03:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:04:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det status' timeout=45s
[2025-11-16T19:04:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:04:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:04:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:04:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:04:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:04:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:04:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:04:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:04:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='det off' timeout=45s
[2025-11-16T19:04:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:04:59Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:04:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:04:59Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:04:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:04:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:04:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:04:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:05:44Z WARN  sis_testing::phase8_deterministic::cbs_edf_scheduler]     ❌ Graph integration: FAILED
[2025-11-16T19:05:44Z INFO  sis_testing::phase8_deterministic::cbs_edf_scheduler] CBS+EDF Scheduler Tests: 0/5 passed (0%)
[2025-11-16T19:05:44Z INFO  sis_testing::phase8_deterministic::stress_comparison] Running Stress Comparison Tests...
[2025-11-16T19:05:44Z INFO  sis_testing::phase8_deterministic::stress_comparison]   Testing autonomy OFF baseline...
[2025-11-16T19:05:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl off' timeout=45s
[2025-11-16T19:05:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:05:44Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:05:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:05:44Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:05:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:05:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:05:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:05:44Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:06:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 5000' timeout=45s
[2025-11-16T19:06:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:06:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:06:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:06:29Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:06:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:06:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:06:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:06:29Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:07:14Z INFO  sis_testing::phase8_deterministic::stress_comparison]   Testing autonomy ON comparison...
[2025-11-16T19:07:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='autoctl on' timeout=45s
[2025-11-16T19:07:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:07:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:07:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:07:14Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:07:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:07:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:07:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:07:14Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:07:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: START command='stresstest memory --duration 5000' timeout=45s
[2025-11-16T19:07:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: waiting for shell prompt
[2025-11-16T19:07:59Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:07:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: shell prompt ready
[2025-11-16T19:07:59Z INFO  sis_testing::kernel_interface] Shell prompt detected after 1 attempts, ready for commands
[2025-11-16T19:07:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: updating log position
[2025-11-16T19:07:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: log position updated
[2025-11-16T19:07:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: sending command via serial
[2025-11-16T19:07:59Z INFO  sis_testing::kernel_interface] execute_command_with_timeout: command sent, waiting for completion
[2025-11-16T19:08:44Z INFO  sis_testing::phase8_deterministic::stress_comparison]   Testing performance delta validation...
[2025-11-16T19:08:44Z INFO  sis_testing::phase8_deterministic::stress_comparison]     ✅ Performance delta: PASSED
[2025-11-16T19:08:44Z INFO  sis_testing::phase8_deterministic::stress_comparison] Stress Comparison Tests: 1/3 passed
[2025-11-16T19:08:44Z INFO  sis_testing::phase8_deterministic] ✅ Phase 8 validation complete: 50.0% (3/6 subsystems passed)
[2025-11-16T19:08:44Z INFO  sis_testing::reporting] Generating comprehensive industry-grade validation report
[2025-11-16T19:08:44Z INFO  sis_testing::reporting::analytics] Generating comprehensive analytics report
[2025-11-16T19:08:44Z INFO  sis_testing::reporting] JSON report written to: target/testing/validation_report.json
[2025-11-16T19:08:44Z INFO  sis_testing::reporting] Analytics JSON report written to: target/testing/analytics_report.json
[2025-11-16T19:08:44Z INFO  sis_testing::reporting::visualization] Generating interactive visualization dashboard
[2025-11-16T19:08:44Z INFO  sis_testing::reporting] Interactive dashboard written to: target/testing/interactive_dashboard.html
[2025-11-16T19:08:44Z INFO  sis_testing::reporting] HTML dashboard written to: target/testing/dashboard.html
[2025-11-16T19:08:44Z INFO  sis_testing::reporting] Executive summary written to: target/testing/executive_summary.md
[2025-11-16T19:08:44Z INFO  sis_testing::reporting] Technical report written to: target/testing/technical_report.md
[2025-11-16T19:08:44Z INFO  sis_testing::reporting] Performance charts placeholder written to: target/testing/performance_charts.svg
[2025-11-16T19:08:44Z INFO  sis_testing::reporting] Comprehensive industry-grade report generated in: target/testing
[2025-11-16T19:08:44Z WARN  sis_testing] Cannot shutdown QEMU: Arc has multiple owners
[2025-11-16T19:08:44Z INFO  sis_test_runner] 
[2025-11-16T19:08:44Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-16T19:08:44Z INFO  sis_test_runner] ║          SIS KERNEL COMPREHENSIVE TEST VALIDATION REPORT          ║
[2025-11-16T19:08:44Z INFO  sis_test_runner] ╚═══════════════════════════════════════════════════════════════════╝
[2025-11-16T19:08:44Z INFO  sis_test_runner] 
[2025-11-16T19:08:44Z INFO  sis_test_runner]   Status: █ NEEDS IMPROVEMENT
[2025-11-16T19:08:44Z INFO  sis_test_runner]   Overall Score: 0.0%
[2025-11-16T19:08:44Z INFO  sis_test_runner]   Test Results: 0 PASS / 1 FAIL / 1 TOTAL
[2025-11-16T19:08:44Z INFO  sis_test_runner] 
[2025-11-16T19:08:44Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T19:08:44Z INFO  sis_test_runner] │ CORE SYSTEM COVERAGE                                            │
[2025-11-16T19:08:44Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T19:08:44Z INFO  sis_test_runner] │  Performance:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:08:44Z INFO  sis_test_runner] │  Correctness:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:08:44Z INFO  sis_test_runner] │  Security:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:08:44Z INFO  sis_test_runner] │  Distributed:        0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:08:44Z INFO  sis_test_runner] │  AI Validation:      0.0%  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:08:44Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T19:08:44Z INFO  sis_test_runner] 
[2025-11-16T19:08:44Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T19:08:44Z INFO  sis_test_runner] │ PHASE IMPLEMENTATION PROGRESS                                   │
[2025-11-16T19:08:44Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T19:08:44Z INFO  sis_test_runner] │  Phase 1 - AI-Native Dataflow:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:08:44Z INFO  sis_test_runner] │  Phase 2 - AI Governance:                0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:08:44Z INFO  sis_test_runner] │  Phase 3 - Temporal Isolation:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:08:44Z INFO  sis_test_runner] │  Phase 5 - UX Safety:                    0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:08:44Z INFO  sis_test_runner] │  Phase 6 - Web GUI Management:           0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:08:44Z INFO  sis_test_runner] │  Phase 7 - AI Operations:                0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:08:44Z INFO  sis_test_runner] │  Phase 8 - Performance Optimization:    50.0%  ███████████░░░░░░░░░░░░
[2025-11-16T19:08:44Z INFO  sis_test_runner] │  Phase 9 - Agentic Platform:             0.0%  ░░░░░░░░░░░░░░░░░░░░░░░
[2025-11-16T19:08:44Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T19:08:44Z INFO  sis_test_runner] 
[2025-11-16T19:08:44Z INFO  sis_test_runner] ┌─────────────────────────────────────────────────────────────────┐
[2025-11-16T19:08:44Z INFO  sis_test_runner] │ DETAILED VALIDATION RESULTS                                     │
[2025-11-16T19:08:44Z INFO  sis_test_runner] ├─────────────────────────────────────────────────────────────────┤
[2025-11-16T19:08:44Z INFO  sis_test_runner] │ ✗ FAIL
[2025-11-16T19:08:44Z INFO  sis_test_runner] │   Test: Phase 8: Performance Optimization
[2025-11-16T19:08:44Z INFO  sis_test_runner] │   Target: ≥75% pass rate | Measured: 50.0%
[2025-11-16T19:08:44Z INFO  sis_test_runner] │   Industry Benchmark: Industry standard: Performance opt 60-75%
[2025-11-16T19:08:44Z INFO  sis_test_runner] │
[2025-11-16T19:08:44Z INFO  sis_test_runner] └─────────────────────────────────────────────────────────────────┘
[2025-11-16T19:08:44Z INFO  sis_test_runner] 
[2025-11-16T19:08:44Z INFO  sis_test_runner] 📊 Reports generated in: target/testing/
[2025-11-16T19:08:44Z INFO  sis_test_runner] 🌐 View dashboard: target/testing/dashboard.html
[2025-11-16T19:08:44Z INFO  sis_test_runner] 
[2025-11-16T19:08:44Z INFO  sis_test_runner] ╔═══════════════════════════════════════════════════════════════════╗
[2025-11-16T19:08:44Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-16T19:08:44Z INFO  sis_test_runner] ║  ✗ WARNING: SIS Kernel requires improvements before production  ║
[2025-11-16T19:08:44Z INFO  sis_test_runner] ║    readiness (0.0%). Review failed tests above.                ║
[2025-11-16T19:08:44Z INFO  sis_test_runner] ║                                                                   ║
[2025-11-16T19:08:44Z INFO  sis_test_runner] ╚═══════════════════════════════════════════════════════════════════╝
