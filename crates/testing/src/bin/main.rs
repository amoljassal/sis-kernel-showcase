// SIS Kernel Test Runner Binary
// Main entry point for comprehensive test suite execution

use sis_testing::{SISTestSuite, TestSuiteConfig, setup_logging};
use std::env;

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
            log::info!("");
            log::info!("=== VALIDATION COMPLETE ===");
            log::info!("Overall Score: {:.1}%", report.overall_score);
            log::info!("Performance Coverage: {:.1}%", report.test_coverage.performance_coverage * 100.0);
            log::info!("Correctness Coverage: {:.1}%", report.test_coverage.correctness_coverage * 100.0);
            log::info!("Security Coverage: {:.1}%", report.test_coverage.security_coverage * 100.0);
            log::info!("Distributed Coverage: {:.1}%", report.test_coverage.distributed_coverage * 100.0);
            log::info!("AI Coverage: {:.1}%", report.test_coverage.ai_coverage * 100.0);
            log::info!("");
            
            log::info!("Validation Results:");
            for result in &report.results {
                let status = if result.passed { "PASS" } else { "FAIL" };
                log::info!("  [{}] {} ({})", status, result.claim, result.measured);
            }
            
            log::info!("");
            log::info!("Reports generated in: target/testing/");
            log::info!("View dashboard: target/testing/dashboard.html");
            
            if report.overall_score >= 90.0 {
                log::info!("SUCCESS: SIS Kernel meets industry standards for production deployment!");
                std::process::exit(0);
            } else {
                log::warn!("WARNING: SIS Kernel requires improvements before production readiness");
                std::process::exit(1);
            }
        }
        Err(e) => {
            log::error!("Validation failed: {}", e);
            std::process::exit(1);
        }
    }
}
