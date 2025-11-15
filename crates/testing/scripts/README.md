# SIS Kernel QEMU Test Automation Scripts

Industry-grade QEMU test environment automation for comprehensive validation of the SIS Kernel across multiple virtual machines.

## Overview

This directory contains two complementary tools for automated QEMU-based testing:

1. **`qemu_automation.sh`** - Bash script for basic QEMU node management and test execution
2. **`test_orchestrator.py`** - Python orchestrator for advanced statistical analysis and reporting

## Features

### QEMU Automation Script (`qemu_automation.sh`)

- **Multi-node QEMU management**: Spawn and manage multiple QEMU instances
- **Automated testing**: Performance, distributed consensus, and security tests
- **Industry-grade logging**: Structured logs with timestamps and severity levels
- **Configurable parameters**: Nodes, memory, CPU, timeout settings
- **HTML reporting**: Generate professional test reports
- **Cleanup automation**: Proper cleanup of QEMU processes and temporary files

### Test Orchestrator (`test_orchestrator.py`)

- **Statistical analysis**: Bootstrap confidence intervals, percentiles, standard deviation
- **Concurrent execution**: Async/await for parallel test execution
- **Industry metrics**: Performance benchmarks against industry baselines
- **Professional reporting**: JSON results and HTML dashboards
- **Extensible framework**: Easy to add new test categories

## Quick Start

### Prerequisites

```bash
# Install QEMU
brew install qemu  # macOS
sudo apt install qemu-system-x86  # Ubuntu/Debian

# Install Python dependencies
pip3 install asyncio statistics
```

### Basic Usage

```bash
# Setup test environment
./qemu_automation.sh setup

# Run comprehensive test suite
./qemu_automation.sh test --nodes 10 --timeout 600

# Run performance benchmarks only
./qemu_automation.sh performance --nodes 5 --verbose

# Run distributed consensus tests
./qemu_automation.sh distributed --nodes 20

# Generate test report
./qemu_automation.sh report

# Advanced orchestration with statistics
./test_orchestrator.py --all --nodes 10 --iterations 10000
```

## Detailed Usage

### QEMU Automation Script

```bash
# Available commands
./qemu_automation.sh [OPTIONS] [COMMAND]

Commands:
  setup           Setup test environment and dependencies
  test            Run comprehensive test suite
  performance     Run performance benchmarks
  stress          Run stress testing
  distributed     Run distributed consensus tests
  security        Run security validation tests
  cleanup         Clean up test artifacts
  report          Generate test reports
  help            Show help message

Options:
  -n, --nodes NUM     Number of QEMU nodes (default: 10)
  -t, --timeout SEC   Test timeout in seconds (default: 300)
  -m, --memory SIZE   Memory per QEMU instance (default: 256M)
  -c, --cpus NUM      CPU cores per QEMU instance (default: 4)
  -o, --output DIR    Output directory
  -v, --verbose       Verbose output
  -q, --quiet         Quiet output
```

### Test Orchestrator

```bash
# Available options
./test_orchestrator.py [OPTIONS]

Options:
  --nodes NUM         Number of QEMU nodes (default: 10)
  --iterations NUM    Test iterations per node (default: 1000)
  --output PATH       Output directory
  --project-root PATH Project root directory
  --performance       Run performance tests only
  --distributed       Run distributed tests only
  --all              Run all test suites
```

## Test Categories

### Performance Tests

- **AI Inference**: Measures inference latency with statistical analysis
  - Target: <40μs P99 latency
  - Metrics: Mean, P95, P99, standard deviation
  - Industry comparison: TensorFlow Lite (50-100ms), ONNX Runtime (25-80ms)

- **Context Switch**: Measures context switching performance
  - Target: <500ns P95 latency
  - Metrics: Mean, P95, P99 latency in nanoseconds
  - Industry comparison: Linux (1-2μs)

- **Memory Allocation**: Tests memory allocation performance
  - Target: <1μs P99 latency
  - Metrics: Allocation latency, fragmentation analysis

- **Throughput**: Measures system throughput
  - Target: >500K operations/second
  - Metrics: Operations per second, sustained throughput

### Distributed Tests

- **Byzantine Consensus**: Tests fault-tolerant consensus
  - Target: <5ms consensus time with 100 nodes
  - Supports f < n/3 Byzantine fault tolerance
  - Metrics: Consensus time, message complexity

- **Leader Election**: Tests leader election performance
  - Target: <100ms election time
  - Metrics: Election rounds, convergence time

- **Network Partition Recovery**: Tests partition tolerance
  - Target: <500ms recovery time
  - Metrics: Recovery time, data consistency validation

### Security Tests (Future)

- Static analysis integration
- Fuzzing campaigns
- Penetration testing scenarios
- Vulnerability assessment

## Output Format

### Directory Structure

```
target/testing/
├── qemu/
│   ├── logs/
│   │   ├── node_1.log
│   │   ├── node_2.log
│   │   └── ...
│   └── results/
│       ├── performance_results.json
│       ├── distributed_results.json
│       └── test_report.html
└── orchestrator/
    ├── performance_results.json
    ├── distributed_results.json
    ├── industry_validation_report.json
    ├── validation_dashboard.html
    └── orchestrator.log
```

### JSON Result Format

```json
{
  "suite_name": "performance_benchmark",
  "start_time": 1693876543.123,
  "end_time": 1693876598.456,
  "duration": 55.333,
  "node_count": 10,
  "timestamp": "2023-09-04T15:30:43.123Z",
  "results": [
    {
      "test_name": "ai_inference",
      "node_id": 1,
      "success": true,
      "metrics": {
        "mean_latency_us": 21.5,
        "p99_latency_us": 38.2,
        "std_dev_us": 5.8
      }
    }
  ],
  "summary": {
    "total_tests": 40,
    "passed_tests": 39,
    "failed_tests": 1,
    "overall_success_rate": 0.975
  }
}
```

## Integration with CI/CD

### GitHub Actions Integration

```yaml
name: QEMU Validation
on: [push, pull_request]

jobs:
  qemu-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install QEMU
        run: sudo apt-get install -y qemu-system-x86
      
      - name: Setup test environment
        run: ./crates/testing/scripts/qemu_automation.sh setup
      
      - name: Run performance tests
        run: ./crates/testing/scripts/qemu_automation.sh performance --nodes 5
      
      - name: Run distributed tests
        run: ./crates/testing/scripts/qemu_automation.sh distributed --nodes 8
      
      - name: Generate reports
        run: ./crates/testing/scripts/qemu_automation.sh report
      
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: qemu-test-results
          path: target/testing/
```

## Environment Variables

- `SIS_QEMU_PATH`: Custom QEMU binary path
- `SIS_TEST_TIMEOUT`: Default test timeout (seconds)
- `SIS_TEST_NODES`: Default number of nodes
- `SIS_VERBOSE`: Enable verbose output (1/true/yes)
- `SIS_QUIET`: Enable quiet output (1/true/yes)

## Troubleshooting

### Common Issues

1. **QEMU not found**
   ```bash
   export SIS_QEMU_PATH=/usr/local/bin/qemu-system-x86_64
   ```

2. **Permission denied**
   ```bash
   chmod +x ./qemu_automation.sh ./test_orchestrator.py
   ```

3. **OVMF not found**
   ```bash
   # Download OVMF firmware files to scripts/ directory
   wget https://github.com/tianocore/edk2/releases/download/edk2-stable202308/OVMF-X64-r4136.zip
   ```

4. **Python dependencies missing**
   ```bash
   pip3 install asyncio statistics pathlib dataclasses
   ```

### Debug Mode

```bash
# Enable debug output
./qemu_automation.sh test --verbose

# Check individual node logs
tail -f target/testing/qemu/logs/node_1.log

# Monitor QEMU processes
ps aux | grep qemu
```

## Performance Baselines

### Industry Comparisons

| Metric | SIS Kernel Target | Industry Baseline |
|--------|-------------------|-------------------|
| AI Inference | <40μs P99 | TensorFlow Lite: 50-100ms |
| Context Switch | <500ns P95 | Linux: 1-2μs |
| Consensus (100 nodes) | <5ms | Tendermint: 5-10ms |
| Memory Allocation | <1μs P99 | glibc malloc: 100-500ns |

### Statistical Confidence

- All performance measurements use bootstrap confidence intervals
- Minimum 1000 iterations per test for statistical significance
- 99% confidence intervals reported for critical metrics
- P95/P99 percentiles calculated for latency-sensitive operations

## Contributing

When adding new test categories:

1. Add bash functions to `qemu_automation.sh`
2. Implement Python methods in `test_orchestrator.py`
3. Update this README with new test descriptions
4. Add appropriate metrics and targets
5. Include industry baseline comparisons

## License

This testing framework is part of the SIS Kernel project and follows the same licensing terms.