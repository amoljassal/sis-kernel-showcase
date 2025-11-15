# AI/ML Kernel Implementation Plan

Technical specification for integrating ML operators into SIS kernel dataflow architecture. This document serves as reference for developers and AI agents implementing kernel-native ML execution.

## Executive Summary

Implement ONNX operators and BLAS primitives directly in kernel space, leveraging existing Graph/Operator/Channel/Tensor infrastructure for zero-copy ML inference with deterministic performance.

## Architecture Overview

### Kernel-Userspace Split

```
┌──────────────────────────────────────────┐
│         USERSPACE (Python/Rust)          │
│  • Model loading (ONNX, TorchScript)     │
│  • Graph construction via control plane  │
│  • High-level APIs (PyTorch, TF compat)  │
└──────────────────────────────────────────┘
           ↕ Control Frames (V0 Protocol)
┌──────────────────────────────────────────┐
│         KERNEL (Rust no_std)             │
│  • ML operator execution (NEON/SIMD)     │
│  • Tensor memory management (arenas)     │
│  • Graph scheduling (deterministic)      │
│  • Hardware access (PMU, cache control)  │
└──────────────────────────────────────────┘
```

### Integration with Existing Architecture

ML operators integrate as specialized `Operator` implementations in the existing graph system:

```rust
// Extends existing crates/kernel/src/graph.rs
pub enum OperatorFunc {
    // Existing
    Custom(fn(&mut OperatorCtx)),
    
    // New ML operators
    MLOp(MLOperator),
}

pub enum MLOperator {
    // BLAS Level 3
    Gemm { m: usize, n: usize, k: usize, alpha: f32, beta: f32 },
    
    // Neural Network
    Conv2d { kernel: [usize; 2], stride: [usize; 2], padding: [usize; 4] },
    MaxPool2d { kernel: [usize; 2], stride: [usize; 2] },
    
    // Activations
    Relu, Sigmoid, Tanh, Softmax { axis: i32 },
    
    // Normalization
    BatchNorm { eps: f32, momentum: f32 },
    LayerNorm { eps: f32, normalized_shape: Vec<usize> },
}
```

## Implementation Phases

### Phase 1: Core Infrastructure (Weeks 1-3)

#### 1.1 Extended Tensor Type
```rust
// crates/kernel/src/tensor/mod.rs
pub struct TensorHandle {
    pub ptr: *mut u8,
    pub size: usize,
    pub align: usize,
    
    // New fields for ML
    pub dtype: DataType,
    pub shape: TensorShape,
    pub strides: Option<Vec<usize>>,  // None = contiguous
}

pub enum DataType {
    F32, F16, BF16,
    I32, I16, I8,
    U32, U16, U8,
    Q4_0, Q8_0,  // Quantized types
}

pub struct TensorShape {
    pub dims: heapless::Vec<usize, 8>,  // Max 8D tensors
}
```

#### 1.2 ML Arena Allocator
```rust
// crates/kernel/src/tensor/arena.rs
pub struct MLArena<const SIZE: usize> {
    // Extend existing BumpArena with alignment guarantees
    memory: [u8; SIZE],
    offset: AtomicUsize,
    
    // New: track tensor metadata
    tensors: heapless::Vec<TensorMetadata, 256>,
}

impl<const SIZE: usize> MLArena<SIZE> {
    pub fn alloc_tensor(&mut self, shape: &TensorShape, dtype: DataType) -> Option<TensorHandle> {
        let size = shape.numel() * dtype.size();
        let align = dtype.alignment();  // 64-byte for NEON
        // ... bump allocation with alignment
    }
}
```

#### 1.3 Control Plane Extensions
```rust
// crates/kernel/src/control.rs
// New commands for ML operators
// 0x10 AddMLOperator { op_type, params }
// 0x11 LoadWeights { tensor_id, data }
// 0x12 SetTensorShape { tensor_id, shape }

pub fn handle_ml_frame(frame: &[u8]) -> Result<(), CtrlError> {
    match cmd {
        0x10 => { // AddMLOperator
            let op_type = MLOpType::from_u8(payload[0]);
            let params = &payload[1..];
            add_ml_operator(op_type, params)
        }
        // ...
    }
}
```

### Phase 2: BLAS Implementation (Weeks 4-6)

#### 2.1 NEON-Optimized GEMM
```rust
// crates/kernel/src/ml_ops/blas.rs
#[cfg(target_arch = "aarch64")]
pub unsafe fn sgemm_neon(
    m: usize, n: usize, k: usize,
    alpha: f32,
    a: *const f32, lda: usize,
    b: *const f32, ldb: usize,
    beta: f32,
    c: *mut f32, ldc: usize,
) {
    // 4x4 kernel with NEON (extend existing neon_matmul)
    // Loop tiling for cache efficiency
    const TILE_M: usize = 64;
    const TILE_N: usize = 64;
    const TILE_K: usize = 256;
    
    for m_tile in (0..m).step_by(TILE_M) {
        for n_tile in (0..n).step_by(TILE_N) {
            for k_tile in (0..k).step_by(TILE_K) {
                sgemm_kernel_4x4_neon(/* ... */);
            }
        }
    }
}
```

#### 2.2 Operator Registration
```rust
// crates/kernel/src/ml_ops/registry.rs
pub struct MLOpRegistry {
    ops: heapless::FnvIndexMap<u32, MLOpImpl, 256>,
}

pub struct MLOpImpl {
    pub execute: fn(&TensorHandle, &TensorHandle, &mut TensorHandle) -> Result<(), MLError>,
    pub compute_shape: fn(&[TensorShape]) -> TensorShape,
    pub estimated_cycles: fn(&TensorShape) -> u64,  // For admission control
}

// Registration at boot
pub fn register_ml_ops(registry: &mut MLOpRegistry) {
    registry.register(GEMM_OP_ID, MLOpImpl {
        execute: gemm_execute,
        compute_shape: gemm_shape,
        estimated_cycles: |shape| shape.dims[0] * shape.dims[1] * shape.dims[2] * 2,
    });
}
```

### Phase 3: ONNX Operators (Weeks 7-10)

#### 3.1 Core ONNX Ops
```rust
// crates/kernel/src/ml_ops/onnx.rs
pub enum ONNXOperator {
    // Prioritized by usage frequency in common models
    Conv { 
        kernel_shape: Vec<usize>,
        dilations: Vec<usize>,
        group: usize,
        pads: Vec<usize>,
        strides: Vec<usize>,
    },
    Gemm {
        alpha: f32,
        beta: f32,
        transA: bool,
        transB: bool,
    },
    Relu,
    MaxPool {
        kernel_shape: Vec<usize>,
        pads: Vec<usize>,
        strides: Vec<usize>,
    },
    Add,  // Elementwise
    BatchNormalization {
        epsilon: f32,
        momentum: f32,
    },
    Concat { axis: i32 },
    Reshape { shape: Vec<i64> },
    Softmax { axis: i32 },
    Transpose { perm: Vec<usize> },
}
```

#### 3.2 Conv2D Implementation
```rust
// crates/kernel/src/ml_ops/conv2d.rs
pub fn conv2d_nchw(
    input: &TensorHandle,   // [N, C_in, H, W]
    weight: &TensorHandle,  // [C_out, C_in, K_h, K_w]
    bias: Option<&TensorHandle>,
    output: &mut TensorHandle,
    params: &Conv2dParams,
) -> Result<(), MLError> {
    // im2col transformation for GEMM acceleration
    let col_buffer = /* allocate from arena */;
    im2col_nchw(input, &mut col_buffer, params);
    
    // Convolution as GEMM
    let m = params.out_channels;
    let n = params.out_height * params.out_width;
    let k = params.in_channels * params.kernel_h * params.kernel_w;
    
    unsafe {
        sgemm_neon(m, n, k, 1.0, 
                   weight.ptr as *const f32, k,
                   col_buffer.ptr as *const f32, n,
                   0.0, output.ptr as *mut f32, n);
    }
    
    // Add bias if present
    if let Some(b) = bias {
        add_bias_nchw(output, b);
    }
    
    Ok(())
}
```

### Phase 4: Userspace Bridge (Weeks 11-13)

#### 4.1 Python Control Library
```python
# tools/sis_ml/control.py
import struct
import numpy as np
from typing import List, Tuple

class SISMLControl:
    def __init__(self, serial_port="/dev/ttyAMA0"):
        self.port = serial_port
        self.next_tensor_id = 0
        self.graph = None
    
    def create_graph(self) -> None:
        """Send CreateGraph control frame"""
        frame = struct.pack("<cBBBI", b'C', 0, 0x01, 0, 0)
        self._send_frame(frame)
        
    def add_conv2d(self, 
                   in_channels: int, 
                   out_channels: int,
                   kernel_size: Tuple[int, int],
                   stride: Tuple[int, int] = (1, 1),
                   padding: Tuple[int, int, int, int] = (0, 0, 0, 0)) -> int:
        """Add Conv2D operator to graph"""
        op_id = self.next_op_id
        params = struct.pack("<IIIIIIIIII", 
                           in_channels, out_channels,
                           kernel_size[0], kernel_size[1],
                           stride[0], stride[1],
                           padding[0], padding[1], padding[2], padding[3])
        
        frame = self._build_frame(0x10, params)  # AddMLOperator
        self._send_frame(frame)
        self.next_op_id += 1
        return op_id
```

#### 4.2 ONNX Loader
```python
# tools/sis_ml/onnx_loader.py
import onnx
from onnx import numpy_helper

class ONNXToSIS:
    """Convert ONNX model to SIS kernel graph"""
    
    SUPPORTED_OPS = {
        'Conv': 'add_conv2d',
        'Gemm': 'add_gemm',
        'Relu': 'add_relu',
        'MaxPool': 'add_maxpool',
        # ...
    }
    
    def load_model(self, path: str, control: SISMLControl):
        model = onnx.load(path)
        graph = model.graph
        
        # Create kernel graph
        control.create_graph()
        
        # Add operators in topological order
        for node in graph.node:
            if node.op_type not in self.SUPPORTED_OPS:
                raise ValueError(f"Unsupported op: {node.op_type}")
            
            method = getattr(control, self.SUPPORTED_OPS[node.op_type])
            params = self._extract_params(node)
            method(**params)
        
        # Load weights
        for init in graph.initializer:
            tensor = numpy_helper.to_array(init)
            control.load_weights(init.name, tensor)
```

### Phase 5: Performance Optimization (Weeks 14-16)

#### 5.1 Operator Fusion
```rust
// crates/kernel/src/ml_ops/fusion.rs
pub enum FusedOp {
    ConvReluPool {  // Common pattern in CNNs
        conv: Conv2dParams,
        pool: MaxPoolParams,
    },
    GemmReluGemm {  // Common in MLPs
        gemm1: GemmParams,
        gemm2: GemmParams,
    },
}

impl FusedOp {
    pub fn execute(&self, inputs: &[TensorHandle], output: &mut TensorHandle) -> Result<(), MLError> {
        // Single kernel launch, better cache locality
        match self {
            FusedOp::ConvReluPool { conv, pool } => {
                // Fused implementation avoids intermediate writes
                conv2d_relu_maxpool_fused(inputs[0], inputs[1], output, conv, pool)
            }
            // ...
        }
    }
}
```

#### 5.2 Quantization Support
```rust
// crates/kernel/src/ml_ops/quantized.rs
pub fn qgemm_q4_0(
    m: usize, n: usize, k: usize,
    a_q4: *const u8,  // 4-bit quantized
    b_q4: *const u8,
    c_f32: *mut f32,  // Output in FP32
    scales_a: *const f32,
    scales_b: *const f32,
) {
    // 4-bit matrix multiplication with dequantization
    // 2x memory bandwidth reduction
}
```

## Performance Targets

### Latency Requirements
```
Operation           Target      Current (est)   Improvement
─────────────────────────────────────────────────────────
GEMM (256×256)      <10μs       100μs          10×
Conv2D (3×3)        <1μs        10μs           10×  
ReLU (1024)         <100ns      1μs            10×
Softmax (1024)      <500ns      5μs            10×
BatchNorm (256)     <200ns      2μs            10×
```

### Memory Efficiency
```
Metric              Target      Rationale
─────────────────────────────────────────
Zero-copy rate      100%        All ops use tensor handles
Arena efficiency    >90%        Minimal fragmentation
Cache miss rate     <5%         Tiled algorithms
```

## Testing & Validation

### 5.1 Operator Correctness
```rust
// crates/testing/src/ml_validation.rs
pub fn validate_ml_operators() {
    // Compare against reference implementations
    for op in ML_OPERATORS {
        let input = generate_test_tensor();
        let kernel_output = execute_in_kernel(op, input);
        let reference_output = execute_reference(op, input);
        
        assert!(tensor_allclose(kernel_output, reference_output, rtol=1e-5));
    }
}
```

### 5.2 Performance Benchmarks
```rust
// crates/kernel/src/ml_ops/benchmark.rs
pub fn benchmark_ml_ops() {
    for op in BENCHMARK_OPS {
        let start = read_cntvct();
        op.execute();
        let end = read_cntvct();
        
        metric_kv(&format!("ml_{}_ns", op.name), cycles_to_ns(end - start));
        
        // PMU attribution
        #[cfg(feature = "perf-verbose")]
        {
            let pmu = read_pmu_snapshot();
            metric_kv(&format!("ml_{}_inst", op.name), pmu.instructions);
            metric_kv(&format!("ml_{}_l1d_miss", op.name), pmu.l1d_refills);
        }
    }
}
```

### 5.3 Model-Level Testing
```python
# tools/sis_ml/test_models.py
def test_resnet18():
    """End-to-end ResNet18 inference test"""
    # Load ONNX model
    model = onnx.load("resnet18.onnx")
    
    # Deploy to kernel
    control = SISMLControl()
    loader = ONNXToSIS()
    loader.load_model(model, control)
    
    # Run inference
    input_tensor = np.random.randn(1, 3, 224, 224).astype(np.float32)
    output = control.infer(input_tensor)
    
    # Validate against PyTorch
    torch_model = torchvision.models.resnet18(pretrained=True)
    torch_output = torch_model(torch.from_numpy(input_tensor))
    
    assert np.allclose(output, torch_output.numpy(), rtol=1e-3)
```

## Integration with CI/CD

### GitHub Actions Workflow
```yaml
# .github/workflows/ml-kernel-tests.yml
name: ML Kernel Tests

on: [push, pull_request]

jobs:
  ml-operator-tests:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Build kernel with ML ops
      run: |
        cargo build -p sis_kernel \
          --features bringup,ml-ops,perf-verbose \
          --target aarch64-unknown-none
    
    - name: Run operator tests
      run: |
        cargo run -p sis-testing --release -- \
          --ml-validation \
          --output-dir target/ml-testing
    
    - name: Validate against ONNX test suite
      run: |
        python tools/sis_ml/onnx_conformance.py
    
    - name: Check performance targets
      run: |
        python scripts/check_ml_performance.py \
          target/ml-testing/metrics_dump.json
```

## Feature Flags

```toml
# crates/kernel/Cargo.toml
[features]
ml-ops = ["ml-blas", "ml-onnx"]
ml-blas = []           # BLAS primitives only
ml-onnx = ["ml-blas"]  # ONNX operators (requires BLAS)
ml-quantized = []      # Q4_0, Q8_0 support
ml-fusion = []         # Operator fusion
ml-all = ["ml-ops", "ml-quantized", "ml-fusion"]
```

## Shell Commands

Extend existing shell with ML commands:

```rust
// crates/kernel/src/shell.rs
"mlinfo" => {
    // Print ML capabilities
    unsafe {
        crate::uart_print(b"ML Operators: ");
        crate::uart_print(b"GEMM Conv2D ReLU MaxPool ");
        crate::uart_print(b"BatchNorm Softmax\n");
        crate::uart_print(b"Quantization: Q4_0 Q8_0\n");
        crate::uart_print(b"SIMD: NEON enabled\n");
    }
}

"mlbench" => {
    // Run ML benchmarks
    crate::ml_ops::benchmark::benchmark_ml_ops();
}

"mltest" => {
    // Run correctness tests
    crate::ml_ops::test::test_all_operators();
}
```

## Security Considerations

### Model Verification
```rust
// crates/kernel/src/ml_ops/security.rs
pub struct ModelCapability {
    pub hash: [u8; 32],      // SHA-256 of model
    pub max_memory: usize,    // Memory budget
    pub max_cycles: u64,      // Compute budget
    pub allowed_ops: Vec<MLOpType>,
}

pub fn verify_model(model: &[u8], cap: &ModelCapability) -> Result<(), SecurityError> {
    // Verify hash
    let hash = sha256(model);
    if hash != cap.hash {
        return Err(SecurityError::InvalidModel);
    }
    
    // Check resource limits
    let estimated_memory = estimate_memory_usage(model);
    if estimated_memory > cap.max_memory {
        return Err(SecurityError::ExceedsMemoryBudget);
    }
    
    Ok(())
}
```

## Metrics & Observability

New ML-specific metrics:

```rust
// METRIC ml_inference_latency_us=<value>
// METRIC ml_ops_per_second=<value>
// METRIC ml_memory_bandwidth_gb_s=<value>
// METRIC ml_cache_efficiency_pct=<value>
// METRIC ml_operator_fusion_rate=<value>
// METRIC ml_quantization_speedup=<value>
```

## Dependencies

None - all implementations are no_std compatible and use only core Rust features.

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Operator bugs | Extensive testing against reference implementations |
| Performance regression | CI benchmarks with strict thresholds |
| Memory fragmentation | Fixed arena allocation, no dynamic memory |
| Numerical instability | FP32 by default, optional FP16/BF16 |
| Patent concerns | Implement from papers/specs, not source code |

## Success Criteria

1. **Functional**: Pass ONNX conformance tests for 20 core operators
2. **Performance**: 10× latency improvement vs userspace for GEMM/Conv2D
3. **Integration**: PyTorch/TensorFlow models run via ONNX export
4. **Stability**: 100% test coverage, zero memory leaks
5. **Documentation**: Complete API docs and usage examples

## Timeline

| Week | Milestone |
|------|-----------|
| 1-3 | Core infrastructure (tensors, arenas, control plane) |
| 4-6 | BLAS implementation (GEMM, vector ops) |
| 7-10 | ONNX operators (Conv, Pool, Activations) |
| 11-13 | Userspace bridge (Python API, ONNX loader) |
| 14-16 | Optimization (fusion, quantization, tuning) |
| 17-18 | Testing & benchmarking |
| 19-20 | Documentation & polish |

## References

- ONNX Operators: https://github.com/onnx/onnx/blob/main/docs/Operators.md
- ARM NEON Intrinsics: https://developer.arm.com/architectures/instruction-sets/simd-isas/neon
- BLAS Reference: http://www.netlib.org/blas/
- im2col Algorithm: https://petewarden.com/2015/04/20/why-gemm-is-at-the-heart-of-deep-learning/

## Next Actions

1. Create `crates/kernel/src/ml_ops/` directory structure
2. Implement `TensorHandle` extensions in `tensor/mod.rs`
3. Add ML control commands to `control.rs`
4. Implement GEMM with existing NEON code as base
5. Create Python control library in `tools/sis_ml/`

---

This plan provides a complete technical roadmap for kernel-native ML execution, maintaining consistency with existing SIS architecture and coding standards.