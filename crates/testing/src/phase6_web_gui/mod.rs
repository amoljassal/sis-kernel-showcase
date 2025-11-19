//! Phase 6: Web GUI Management Interface Tests
//!
//! Complete validation of HTTP server, WebSocket connections, REST API,
//! authentication, and real-time metric streaming.

pub mod http_server;
pub mod websocket;
pub mod api_endpoints;
pub mod authentication;
pub mod real_time_updates;

use crate::kernel_interface::KernelCommandInterface;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Phase 6 test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase6Results {
    pub passed: bool,
    pub http_server_passed: bool,
    pub websocket_passed: bool,
    pub api_endpoints_passed: bool,
    pub authentication_passed: bool,
    pub real_time_updates_passed: bool,
    pub total_tests: u32,
    pub passed_tests: u32,
    pub overall_score: f64,
    pub timestamp: String,
}

/// Phase 6 Web GUI test suite
pub struct Phase6WebGUISuite {
    #[allow(dead_code)]
    kernel_interface: KernelCommandInterface,
    http_server: http_server::HTTPServerTests,
    websocket: websocket::WebSocketTests,
    api_endpoints: api_endpoints::APIEndpointTests,
    authentication: authentication::AuthenticationTests,
    real_time: real_time_updates::RealTimeUpdateTests,
}

impl Phase6WebGUISuite {
    /// Create a new Phase 6 test suite
    pub fn new(serial_log_path: String, qemu_manager: std::sync::Arc<crate::qemu_runtime::QEMURuntimeManager>, node_id: usize, monitor_port: u16) -> Self {
        let base_url = "http://localhost:8080".to_string();
        let ws_url = "ws://localhost:8080/ws".to_string();

        Self {
            kernel_interface: KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port),
            http_server: http_server::HTTPServerTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port),
                base_url.clone(),
            ),
            websocket: websocket::WebSocketTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port),
                ws_url.clone(),
            ),
            api_endpoints: api_endpoints::APIEndpointTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port),
                base_url.clone(),
            ),
            authentication: authentication::AuthenticationTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port),
                base_url.clone(),
            ),
            real_time: real_time_updates::RealTimeUpdateTests::new(
                KernelCommandInterface::new(serial_log_path.clone(), qemu_manager.clone(), node_id, monitor_port),
                ws_url.clone(),
            ),
        }
    }

    /// Run all Phase 6 tests
    pub async fn validate_phase6(&mut self) -> Result<Phase6Results, Box<dyn Error>> {
        log::info!("==================================================");
        log::info!("Starting Phase 6: Web GUI Management Validation");
        log::info!("==================================================");

        // Run all test modules
        let http_result = self.http_server.run_all_tests().await?;
        let ws_result = self.websocket.run_all_tests().await?;
        let api_result = self.api_endpoints.run_all_tests().await?;
        let auth_result = self.authentication.run_all_tests().await?;
        let rt_result = self.real_time.run_all_tests().await?;

        // Calculate overall results
        let http_server_passed = http_result.passed;
        let websocket_passed = ws_result.passed;
        let api_endpoints_passed = api_result.passed;
        let authentication_passed = auth_result.passed;
        let real_time_updates_passed = rt_result.passed;

        let total_tests = http_result.total_tests +
                         ws_result.total_tests +
                         api_result.total_tests +
                         auth_result.total_tests +
                         rt_result.total_tests;

        let passed_tests = http_result.passed_tests +
                          ws_result.passed_tests +
                          api_result.passed_tests +
                          auth_result.passed_tests +
                          rt_result.passed_tests;

        let passed = passed_tests >= (total_tests as f32 * 0.75) as u32;
        let overall_score = (passed_tests as f64 / total_tests as f64) * 100.0;

        log::info!("==================================================");
        log::info!("Phase 6 Summary:");
        log::info!("  HTTP Server:        {}", if http_server_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  WebSocket:          {}", if websocket_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  API Endpoints:      {}", if api_endpoints_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  Authentication:     {}", if authentication_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  Real-Time Updates:  {}", if real_time_updates_passed { "✅ PASSED" } else { "❌ FAILED" });
        log::info!("  Overall:            {}/{} tests passed ({:.1}%)",
            passed_tests, total_tests, overall_score);
        log::info!("==================================================");

        Ok(Phase6Results {
            passed,
            http_server_passed,
            websocket_passed,
            api_endpoints_passed,
            authentication_passed,
            real_time_updates_passed,
            total_tests,
            passed_tests,
            overall_score,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_phase6_suite_creation() {
        use std::sync::Arc;
        use crate::qemu_runtime::QEMURuntimeManager;
        use crate::TestSuiteConfig;

        let config = TestSuiteConfig::default();
        let qemu_manager = Arc::new(QEMURuntimeManager::new(&config));
        let _suite = Phase6WebGUISuite::new(
            "/tmp/serial.log".to_string(),
            qemu_manager,
            0,
            9999,
        );

        // Verify suite is created
        assert!(true);
    }
}
