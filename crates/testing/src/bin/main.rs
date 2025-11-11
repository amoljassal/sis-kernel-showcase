// SIS Kernel Test Runner Binary
// Main entry point for comprehensive test suite execution

use sis_testing::{SISTestSuite, TestSuiteConfig, setup_logging};
use std::env;

/// Create a visual progress bar for percentage values
fn create_bar(percentage: f64, width: usize) -> String {
    let filled = ((percentage / 100.0) * width as f64) as usize;
    let empty = width.saturating_sub(filled);

    let color = if percentage >= 80.0 {
        "\x1b[32m" // Green
    } else if percentage >= 60.0 {
        "\x1b[33m" // Yellow
    } else {
        "\x1b[31m" // Red
    };

    format!("{}{}{}{}",
        color,
        "█".repeat(filled),
        "\x1b[90m░\x1b[0m".repeat(empty),
        "\x1b[0m")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    
    log::info!("SIS Kernel Industry-Grade Test Suite");
    log::info!("====================================");
    
    let args: Vec<String> = env::args().collect();
    let quick = args.iter().any(|a| a == "--quick");
    let full = args.iter().any(|a| a == "--full");
    let llm_smoke = args.iter().any(|a| a == "--llm-smoke");
    let llm_smoke_det = args.iter().any(|a| a == "--llm-smoke-det");
    let llm_model_smoke = args.iter().any(|a| a == "--llm-model-smoke");

    let config = if llm_smoke || llm_smoke_det || llm_model_smoke {
        log::info!("Mode: LLM smoke (single QEMU node)");
        TestSuiteConfig {
            qemu_nodes: 1,
            test_duration_secs: 60,
            performance_iterations: 100,
            statistical_confidence: 0.95,
            output_directory: "target/testing".to_string(),
            generate_reports: false,
            parallel_execution: false,
        }
    } else if full {
        log::info!("Mode: full (comprehensive)");
        // Comprehensive run (heavier and slower)
        TestSuiteConfig {
            qemu_nodes: 1,
            test_duration_secs: 3600,
            performance_iterations: 10000,
            statistical_confidence: 0.99,
            output_directory: "target/testing".to_string(),
            generate_reports: true,
            parallel_execution: true,
        }
    } else if quick {
        log::info!("Mode: quick (no QEMU, simulated)");
        // Fast run without QEMU
        TestSuiteConfig {
            qemu_nodes: 0,
            test_duration_secs: 300,
            performance_iterations: 1000,
            statistical_confidence: 0.95,
            output_directory: "target/testing".to_string(),
            generate_reports: true,
            parallel_execution: true,
        }
    } else {
        log::info!("Mode: default (single QEMU node, moderate iterations)");
        // Default: single-node QEMU with moderate iterations
        TestSuiteConfig {
            qemu_nodes: 1,
            test_duration_secs: 600,
            performance_iterations: 2000,
            statistical_confidence: 0.99,
            output_directory: "target/testing".to_string(),
            generate_reports: true,
            parallel_execution: true,
        }
    };
    
    log::info!("Test Configuration:");
    log::info!("  QEMU Nodes: {}", config.qemu_nodes);
    log::info!("  Duration: {}s", config.test_duration_secs);
    log::info!("  Performance Iterations: {}", config.performance_iterations);
    log::info!("  Statistical Confidence: {:.1}%", config.statistical_confidence * 100.0);
    log::info!("  Output Directory: {}", config.output_directory);
    log::info!("  Parallel Execution: {}", config.parallel_execution);
    
    let mut test_suite = SISTestSuite::new(config);
    
    // Initialize QEMU runtime for actual kernel testing (skip for --llm-smoke; it will manage its own init)
    if !llm_smoke && !llm_smoke_det && !llm_model_smoke && test_suite.config.qemu_nodes > 0 {
        log::info!("Initializing QEMU runtime for kernel validation...");
        if let Err(e) = test_suite.initialize_qemu_runtime().await {
            log::error!("Failed to initialize QEMU runtime: {}", e);
            log::warn!("Falling back to simulated testing mode");
            
            // Disable QEMU for this run by updating config to 0 nodes
            test_suite.config.qemu_nodes = 0;
        } else {
            log::info!("QEMU runtime initialized successfully - running real kernel tests");
        }
    } else if !llm_smoke && !llm_smoke_det && !llm_model_smoke {
        log::info!("QEMU disabled - running simulated testing mode");
    }
    
    if llm_smoke {
        // Minimal path: boot QEMU, run LLM smoke, shutdown, and exit with status
        if let Err(e) = test_suite.initialize_qemu_runtime().await {
            log::error!("Failed to initialize QEMU runtime: {}", e);
            std::process::exit(1);
        }
        if !test_suite.qemu_all_booted {
            log::error!("QEMU did not boot; cannot run LLM smoke");
            let _ = test_suite.shutdown_qemu_runtime().await;
            std::process::exit(1);
        }
        match test_suite.run_llm_smoke().await {
            Ok(true) => {
                log::info!("LLM smoke test PASSED (op=3 present in audit)");
                let _ = test_suite.shutdown_qemu_runtime().await;
                std::process::exit(0);
            }
            Ok(false) => {
                log::error!("LLM smoke test FAILED (op=3 not found)");
                let _ = test_suite.shutdown_qemu_runtime().await;
                std::process::exit(1);
            }
            Err(e) => {
                log::error!("LLM smoke test ERROR: {}", e);
                let _ = test_suite.shutdown_qemu_runtime().await;
                std::process::exit(1);
            }
        }
    }

    if llm_smoke_det {
        // Deterministic LLM smoke: build with deterministic and verify status line
        match test_suite.run_llm_smoke_det().await {
            Ok(true) => {
                log::info!("LLM deterministic smoke test PASSED");
                let _ = test_suite.shutdown_qemu_runtime().await;
                std::process::exit(0);
            }
            Ok(false) => {
                log::error!("LLM deterministic smoke test FAILED");
                let _ = test_suite.shutdown_qemu_runtime().await;
                std::process::exit(1);
            }
            Err(e) => {
                log::error!("LLM deterministic smoke test ERROR: {}", e);
                let _ = test_suite.shutdown_qemu_runtime().await;
                std::process::exit(1);
            }
        }
    }

    if llm_model_smoke {
        match test_suite.run_llm_model_smoke().await {
            Ok(true) => {
                log::info!("LLM model packaging smoke test PASSED");
                let _ = test_suite.shutdown_qemu_runtime().await;
                std::process::exit(0);
            }
            Ok(false) => {
                log::error!("LLM model packaging smoke test FAILED");
                let _ = test_suite.shutdown_qemu_runtime().await;
                std::process::exit(1);
            }
            Err(e) => {
                log::error!("LLM model packaging smoke test ERROR: {}", e);
                let _ = test_suite.shutdown_qemu_runtime().await;
                std::process::exit(1);
            }
        }
    }

    let validation_result = test_suite.execute_comprehensive_validation().await;
    
    // Ensure QEMU cleanup
    if let Err(e) = test_suite.shutdown_qemu_runtime().await {
        log::warn!("Failed to shutdown QEMU runtime cleanly: {}", e);
    }
    
    match validation_result {
        Ok(report) => {
            // Calculate summary statistics
            let total_tests = report.results.len();
            let passed_tests = report.results.iter().filter(|r| r.passed).count();
            let failed_tests = total_tests - passed_tests;

            // Print executive summary header
            log::info!("");
            log::info!("╔═══════════════════════════════════════════════════════════════════╗");
            log::info!("║          SIS KERNEL COMPREHENSIVE TEST VALIDATION REPORT          ║");
            log::info!("╚═══════════════════════════════════════════════════════════════════╝");
            log::info!("");

            // Overall status badge
            let status_badge = if report.overall_score >= 80.0 {
                "\x1b[32m█ PRODUCTION READY\x1b[0m"
            } else if report.overall_score >= 70.0 {
                "\x1b[33m█ ACCEPTABLE\x1b[0m"
            } else {
                "\x1b[31m█ NEEDS IMPROVEMENT\x1b[0m"
            };

            log::info!("  Status: {}", status_badge);
            log::info!("  Overall Score: \x1b[1m{:.1}%\x1b[0m", report.overall_score);
            log::info!("  Test Results: \x1b[32m{} PASS\x1b[0m / \x1b[31m{} FAIL\x1b[0m / {} TOTAL", passed_tests, failed_tests, total_tests);
            log::info!("");

            // Core System Coverage
            log::info!("┌─────────────────────────────────────────────────────────────────┐");
            log::info!("│ CORE SYSTEM COVERAGE                                            │");
            log::info!("├─────────────────────────────────────────────────────────────────┤");

            let perf_cov = report.test_coverage.performance_coverage * 100.0;
            let corr_cov = report.test_coverage.correctness_coverage * 100.0;
            let sec_cov = report.test_coverage.security_coverage * 100.0;
            let dist_cov = report.test_coverage.distributed_coverage * 100.0;
            let ai_cov = report.test_coverage.ai_coverage * 100.0;

            log::info!("│  Performance:     {:>6.1}%  {}", perf_cov, create_bar(perf_cov, 35));
            log::info!("│  Correctness:     {:>6.1}%  {}", corr_cov, create_bar(corr_cov, 35));
            log::info!("│  Security:        {:>6.1}%  {}", sec_cov, create_bar(sec_cov, 35));
            log::info!("│  Distributed:     {:>6.1}%  {}", dist_cov, create_bar(dist_cov, 35));
            log::info!("│  AI Validation:   {:>6.1}%  {}", ai_cov, create_bar(ai_cov, 35));
            log::info!("└─────────────────────────────────────────────────────────────────┘");
            log::info!("");

            // Phase Implementation Progress
            log::info!("┌─────────────────────────────────────────────────────────────────┐");
            log::info!("│ PHASE IMPLEMENTATION PROGRESS                                   │");
            log::info!("├─────────────────────────────────────────────────────────────────┤");

            let phase1 = report.test_coverage.phase1_coverage * 100.0;
            let phase2 = report.test_coverage.phase2_coverage * 100.0;
            let phase3 = report.test_coverage.phase3_coverage * 100.0;
            let phase5 = report.test_coverage.phase5_coverage * 100.0;
            let phase6 = report.test_coverage.phase6_coverage * 100.0;
            let phase7 = report.test_coverage.phase7_coverage * 100.0;
            let phase8 = report.test_coverage.phase8_coverage * 100.0;

            log::info!("│  Phase 1 - AI-Native Dataflow:        {:>6.1}%  {}", phase1, create_bar(phase1, 23));
            log::info!("│  Phase 2 - AI Governance:             {:>6.1}%  {}", phase2, create_bar(phase2, 23));
            log::info!("│  Phase 3 - Temporal Isolation:        {:>6.1}%  {}", phase3, create_bar(phase3, 23));
            log::info!("│  Phase 5 - UX Safety:                 {:>6.1}%  {}", phase5, create_bar(phase5, 23));
            log::info!("│  Phase 6 - Web GUI Management:        {:>6.1}%  {}", phase6, create_bar(phase6, 23));
            log::info!("│  Phase 7 - AI Operations:             {:>6.1}%  {}", phase7, create_bar(phase7, 23));
            log::info!("│  Phase 8 - Performance Optimization:  {:>6.1}%  {}", phase8, create_bar(phase8, 23));
            log::info!("└─────────────────────────────────────────────────────────────────┘");
            log::info!("");

            // Detailed validation results
            log::info!("┌─────────────────────────────────────────────────────────────────┐");
            log::info!("│ DETAILED VALIDATION RESULTS                                     │");
            log::info!("├─────────────────────────────────────────────────────────────────┤");

            for result in &report.results {
                let status = if result.passed {
                    "\x1b[32m✓ PASS\x1b[0m"
                } else {
                    "\x1b[31m✗ FAIL\x1b[0m"
                };

                log::info!("│ {}", status);
                log::info!("│   Test: {}", result.claim);
                log::info!("│   Target: {} | Measured: {}", result.target, result.measured);

                if let Some(ref comparison) = result.industry_comparison {
                    log::info!("│   Industry Benchmark: {}", comparison);
                }

                log::info!("│");
            }

            log::info!("└─────────────────────────────────────────────────────────────────┘");
            log::info!("");

            // Report artifacts
            log::info!("📊 Reports generated in: target/testing/");
            log::info!("🌐 View dashboard: target/testing/dashboard.html");
            log::info!("");

            // Final verdict
            log::info!("╔═══════════════════════════════════════════════════════════════════╗");
            if report.overall_score >= 80.0 {
                log::info!("║                                                                   ║");
                log::info!("║  \x1b[32m✓ SUCCESS: SIS Kernel meets industry standards for production\x1b[0m   ║");
                log::info!("║  \x1b[32m  deployment and is ready for production use.\x1b[0m                   ║");
                log::info!("║                                                                   ║");
                log::info!("╚═══════════════════════════════════════════════════════════════════╝");
                std::process::exit(0);
            } else if report.overall_score >= 70.0 {
                log::info!("║                                                                   ║");
                log::info!("║  \x1b[33m⚠ ACCEPTABLE: SIS Kernel shows good progress ({:.1}%)\x1b[0m           ║", report.overall_score);
                log::info!("║  \x1b[33m  Recommended improvements before full production deployment.\x1b[0m   ║");
                log::info!("║                                                                   ║");
                log::info!("╚═══════════════════════════════════════════════════════════════════╝");
                std::process::exit(0);
            } else {
                log::info!("║                                                                   ║");
                log::info!("║  \x1b[31m✗ WARNING: SIS Kernel requires improvements before production\x1b[0m  ║");
                log::info!("║  \x1b[31m  readiness ({:.1}%). Review failed tests above.\x1b[0m                ║", report.overall_score);
                log::info!("║                                                                   ║");
                log::info!("╚═══════════════════════════════════════════════════════════════════╝");
                std::process::exit(1);
            }
        }
        Err(e) => {
            log::error!("Validation failed: {}", e);
            std::process::exit(1);
        }
    }
}
