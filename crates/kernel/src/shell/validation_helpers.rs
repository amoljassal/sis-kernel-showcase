//! M7 Validation shell commands
//!
//! Comprehensive validation testing for production readiness

use crate::validation::{ValidationResult, ValidationFailure, ValidationCategory};

impl super::Shell {
    /// Main validation command
    pub(crate) fn validate_cmd(&self, args: &[&str]) {
        if args.is_empty() {
            self.validate_all();
            return;
        }

        match args[0] {
            "all" => self.validate_all(),
            "stress" => self.validate_stress(),
            "perf" | "performance" => self.validate_performance(),
            "integration" | "integ" => self.validate_integration(),
            "hardware" | "hw" => self.validate_hardware(),
            "quick" => self.validate_quick(),
            _ => unsafe {
                crate::uart_print(b"Usage: validate [all|stress|perf|integration|hardware|quick]\n");
            },
        }
    }

    /// Run all validation tests
    fn validate_all(&self) {
        self.print_validation_header();

        unsafe {
            crate::uart_print(b"\n=== Running Comprehensive Validation Suite ===\n\n");
        }

        let mut total_passed = 0;
        let mut total_failed = 0;
        let mut total_skipped = 0;

        // Run stress tests
        let (p, f, s) = self.validate_stress_impl();
        total_passed += p;
        total_failed += f;
        total_skipped += s;

        // Run performance tests
        let (p, f, s) = self.validate_performance_impl();
        total_passed += p;
        total_failed += f;
        total_skipped += s;

        // Run integration tests
        let (p, f, s) = self.validate_integration_impl();
        total_passed += p;
        total_failed += f;
        total_skipped += s;

        // Print summary
        self.print_validation_summary(total_passed, total_failed, total_skipped);
    }

    /// Run quick validation (subset of tests)
    fn validate_quick(&self) {
        self.print_validation_header();

        unsafe {
            crate::uart_print(b"\n=== Quick Validation (Subset) ===\n\n");
        }

        let mut passed = 0;
        let mut failed = 0;

        // GPIO quick test
        unsafe {
            crate::uart_print(b"[STRESS] GPIO operations... ");
        }
        match crate::validation::stress_test_gpio_timeout() {
            ValidationResult::Pass => {
                unsafe { crate::uart_print(b"PASS\n"); }
                passed += 1;
            }
            ValidationResult::Fail(_) => {
                unsafe { crate::uart_print(b"FAIL\n"); }
                failed += 1;
            }
            _ => {}
        }

        // PMU quick test
        unsafe {
            crate::uart_print(b"[STRESS] PMU operations... ");
        }
        match crate::validation::stress_test_pmu() {
            ValidationResult::Pass => {
                unsafe { crate::uart_print(b"PASS\n"); }
                passed += 1;
            }
            ValidationResult::Fail(_) => {
                unsafe { crate::uart_print(b"FAIL\n"); }
                failed += 1;
            }
            _ => {}
        }

        unsafe {
            crate::uart_print(b"\nQuick validation: ");
            self.print_number_simple(passed);
            crate::uart_print(b" passed, ");
            self.print_number_simple(failed);
            crate::uart_print(b" failed\n\n");

            if failed == 0 {
                crate::uart_print(b"✓ Quick validation PASSED\n\n");
            } else {
                crate::uart_print(b"✗ Quick validation FAILED\n\n");
            }
        }
    }

    /// Run stress tests
    fn validate_stress(&self) {
        unsafe {
            crate::uart_print(b"\n=== Stress Test Suite ===\n\n");
        }
        self.validate_stress_impl();
    }

    fn validate_stress_impl(&self) -> (u64, u64, u64) {
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        // Test 1: GPIO timeout handling
        unsafe { crate::uart_print(b"[STRESS] GPIO timeout handling... "); }
        match crate::validation::stress_test_gpio_timeout() {
            ValidationResult::Pass => {
                unsafe { crate::uart_print(b"PASS\n"); }
                passed += 1;
            }
            ValidationResult::Fail(reason) => {
                unsafe { crate::uart_print(b"FAIL ("); }
                self.print_failure_reason(reason);
                unsafe { crate::uart_print(b")\n"); }
                failed += 1;
            }
            ValidationResult::Skip => {
                unsafe { crate::uart_print(b"SKIP\n"); }
                skipped += 1;
            }
        }

        // Test 2: Mailbox timeout handling
        unsafe { crate::uart_print(b"[STRESS] Mailbox timeout handling... "); }
        match crate::validation::stress_test_mailbox_timeout() {
            ValidationResult::Pass => {
                unsafe { crate::uart_print(b"PASS\n"); }
                passed += 1;
            }
            ValidationResult::Fail(reason) => {
                unsafe { crate::uart_print(b"FAIL ("); }
                self.print_failure_reason(reason);
                unsafe { crate::uart_print(b")\n"); }
                failed += 1;
            }
            ValidationResult::Skip => {
                unsafe { crate::uart_print(b"SKIP\n"); }
                skipped += 1;
            }
        }

        // Test 3: PMU stress
        unsafe { crate::uart_print(b"[STRESS] PMU stress test... "); }
        match crate::validation::stress_test_pmu() {
            ValidationResult::Pass => {
                unsafe { crate::uart_print(b"PASS\n"); }
                passed += 1;
            }
            ValidationResult::Fail(reason) => {
                unsafe { crate::uart_print(b"FAIL ("); }
                self.print_failure_reason(reason);
                unsafe { crate::uart_print(b")\n"); }
                failed += 1;
            }
            ValidationResult::Skip => {
                unsafe { crate::uart_print(b"SKIP\n"); }
                skipped += 1;
            }
        }

        self.print_category_summary(b"Stress Tests", passed, failed, skipped);
        (passed, failed, skipped)
    }

    /// Run performance tests
    fn validate_performance(&self) {
        unsafe {
            crate::uart_print(b"\n=== Performance Test Suite ===\n\n");
        }
        self.validate_performance_impl();
    }

    fn validate_performance_impl(&self) -> (u64, u64, u64) {
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        // Test 1: GPIO performance
        unsafe { crate::uart_print(b"[PERF] GPIO operations (1000 ops)... "); }
        let (result, duration) = crate::validation::perf_test_gpio();
        match result {
            ValidationResult::Pass => {
                unsafe {
                    crate::uart_print(b"PASS (");
                    self.print_number_simple(duration);
                    crate::uart_print(b" us)\n");
                }
                passed += 1;
            }
            ValidationResult::Fail(reason) => {
                unsafe {
                    crate::uart_print(b"FAIL (");
                    self.print_number_simple(duration);
                    crate::uart_print(b" us, ");
                }
                self.print_failure_reason(reason);
                unsafe { crate::uart_print(b")\n"); }
                failed += 1;
            }
            ValidationResult::Skip => {
                unsafe { crate::uart_print(b"SKIP\n"); }
                skipped += 1;
            }
        }

        // Test 2: Mailbox performance
        unsafe { crate::uart_print(b"[PERF] Mailbox queries (10 queries)... "); }
        let (result, duration) = crate::validation::perf_test_mailbox();
        match result {
            ValidationResult::Pass => {
                unsafe {
                    crate::uart_print(b"PASS (");
                    self.print_number_simple(duration);
                    crate::uart_print(b" us)\n");
                }
                passed += 1;
            }
            ValidationResult::Fail(reason) => {
                unsafe {
                    crate::uart_print(b"FAIL (");
                    self.print_number_simple(duration);
                    crate::uart_print(b" us, ");
                }
                self.print_failure_reason(reason);
                unsafe { crate::uart_print(b")\n"); }
                failed += 1;
            }
            ValidationResult::Skip => {
                unsafe { crate::uart_print(b"SKIP\n"); }
                skipped += 1;
            }
        }

        // Test 3: PMU performance
        unsafe { crate::uart_print(b"[PERF] PMU snapshots (10000 snapshots)... "); }
        let (result, duration) = crate::validation::perf_test_pmu();
        match result {
            ValidationResult::Pass => {
                unsafe {
                    crate::uart_print(b"PASS (");
                    self.print_number_simple(duration);
                    crate::uart_print(b" us)\n");
                }
                passed += 1;
            }
            ValidationResult::Fail(reason) => {
                unsafe {
                    crate::uart_print(b"FAIL (");
                    self.print_number_simple(duration);
                    crate::uart_print(b" us, ");
                }
                self.print_failure_reason(reason);
                unsafe { crate::uart_print(b")\n"); }
                failed += 1;
            }
            ValidationResult::Skip => {
                unsafe { crate::uart_print(b"SKIP\n"); }
                skipped += 1;
            }
        }

        self.print_category_summary(b"Performance Tests", passed, failed, skipped);
        (passed, failed, skipped)
    }

    /// Run integration tests
    fn validate_integration(&self) {
        unsafe {
            crate::uart_print(b"\n=== Integration Test Suite ===\n\n");
        }
        self.validate_integration_impl();
    }

    fn validate_integration_impl(&self) -> (u64, u64, u64) {
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        // Test 1: GPIO + PMU integration
        unsafe { crate::uart_print(b"[INTEG] GPIO + PMU integration... "); }
        match crate::validation::integration_test_gpio_pmu() {
            ValidationResult::Pass => {
                unsafe { crate::uart_print(b"PASS\n"); }
                passed += 1;
            }
            ValidationResult::Fail(reason) => {
                unsafe { crate::uart_print(b"FAIL ("); }
                self.print_failure_reason(reason);
                unsafe { crate::uart_print(b")\n"); }
                failed += 1;
            }
            ValidationResult::Skip => {
                unsafe { crate::uart_print(b"SKIP\n"); }
                skipped += 1;
            }
        }

        // Test 2: Mailbox + PMU integration
        unsafe { crate::uart_print(b"[INTEG] Mailbox + PMU integration... "); }
        match crate::validation::integration_test_mailbox_pmu() {
            ValidationResult::Pass => {
                unsafe { crate::uart_print(b"PASS\n"); }
                passed += 1;
            }
            ValidationResult::Fail(reason) => {
                unsafe { crate::uart_print(b"FAIL ("); }
                self.print_failure_reason(reason);
                unsafe { crate::uart_print(b")\n"); }
                failed += 1;
            }
            ValidationResult::Skip => {
                unsafe { crate::uart_print(b"SKIP\n"); }
                skipped += 1;
            }
        }

        self.print_category_summary(b"Integration Tests", passed, failed, skipped);
        (passed, failed, skipped)
    }

    /// Run hardware validation
    fn validate_hardware(&self) {
        unsafe {
            crate::uart_print(b"\n=== Hardware Validation Suite ===\n");
            crate::uart_print(b"Note: These tests require real Raspberry Pi 5 hardware\n\n");
        }

        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        // Test 1: GPIO hardware validation
        unsafe { crate::uart_print(b"[HW] GPIO hardware validation... "); }
        match crate::validation::hardware_test_gpio() {
            ValidationResult::Pass => {
                unsafe { crate::uart_print(b"PASS\n"); }
                passed += 1;
            }
            ValidationResult::Fail(reason) => {
                unsafe { crate::uart_print(b"FAIL ("); }
                self.print_failure_reason(reason);
                unsafe { crate::uart_print(b")\n"); }
                failed += 1;
            }
            ValidationResult::Skip => {
                unsafe { crate::uart_print(b"SKIP\n"); }
                skipped += 1;
            }
        }

        // Test 2: Mailbox hardware validation
        unsafe { crate::uart_print(b"[HW] Mailbox hardware validation... "); }
        match crate::validation::hardware_test_mailbox() {
            ValidationResult::Pass => {
                unsafe { crate::uart_print(b"PASS\n"); }
                passed += 1;
            }
            ValidationResult::Fail(reason) => {
                unsafe { crate::uart_print(b"FAIL ("); }
                self.print_failure_reason(reason);
                unsafe { crate::uart_print(b")\n"); }
                failed += 1;
            }
            ValidationResult::Skip => {
                unsafe { crate::uart_print(b"SKIP\n"); }
                skipped += 1;
            }
        }

        // Test 3: PMU hardware validation
        unsafe { crate::uart_print(b"[HW] PMU hardware validation... "); }
        match crate::validation::hardware_test_pmu() {
            ValidationResult::Pass => {
                unsafe { crate::uart_print(b"PASS\n"); }
                passed += 1;
            }
            ValidationResult::Fail(reason) => {
                unsafe { crate::uart_print(b"FAIL ("); }
                self.print_failure_reason(reason);
                unsafe { crate::uart_print(b")\n"); }
                failed += 1;
            }
            ValidationResult::Skip => {
                unsafe { crate::uart_print(b"SKIP\n"); }
                skipped += 1;
            }
        }

        self.print_category_summary(b"Hardware Validation", passed, failed, skipped);
    }

    // Helper functions

    fn print_validation_header(&self) {
        unsafe {
            crate::uart_print(b"\n");
            crate::uart_print(b"========================================\n");
            crate::uart_print(b"  M7 Comprehensive Validation Suite\n");
            crate::uart_print(b"  Production Readiness Testing\n");
            crate::uart_print(b"========================================\n");
        }
    }

    fn print_category_summary(&self, category: &[u8], passed: u64, failed: u64, skipped: u64) {
        unsafe {
            crate::uart_print(b"\n  ");
            crate::uart_print(category);
            crate::uart_print(b" Summary: ");
            self.print_number_simple(passed);
            crate::uart_print(b" passed, ");
            self.print_number_simple(failed);
            crate::uart_print(b" failed, ");
            self.print_number_simple(skipped);
            crate::uart_print(b" skipped\n\n");
        }
    }

    fn print_validation_summary(&self, passed: u64, failed: u64, skipped: u64) {
        unsafe {
            crate::uart_print(b"\n========================================\n");
            crate::uart_print(b"Validation Summary:\n");
            crate::uart_print(b"  Passed:  ");
            self.print_number_simple(passed);
            crate::uart_print(b"\n  Failed:  ");
            self.print_number_simple(failed);
            crate::uart_print(b"\n  Skipped: ");
            self.print_number_simple(skipped);
            crate::uart_print(b"\n  Total:   ");
            self.print_number_simple(passed + failed + skipped);
            crate::uart_print(b"\n========================================\n");

            if failed == 0 {
                crate::uart_print(b"\n✓ All validation tests PASSED\n");
                crate::uart_print(b"✓ Kernel is PRODUCTION READY\n\n");
            } else {
                crate::uart_print(b"\n✗ Some validation tests FAILED\n");
                crate::uart_print(b"✗ Please review failed tests before production deployment\n\n");
            }
        }
    }

    fn print_failure_reason(&self, reason: ValidationFailure) {
        unsafe {
            match reason {
                ValidationFailure::UnexpectedTimeout => crate::uart_print(b"unexpected timeout"),
                ValidationFailure::TimeoutNotOccurred => crate::uart_print(b"timeout not occurred"),
                ValidationFailure::PerformanceRegression => crate::uart_print(b"performance regression"),
                ValidationFailure::IntegrationFailure => crate::uart_print(b"integration failure"),
                ValidationFailure::HardwareFailure => crate::uart_print(b"hardware failure"),
                ValidationFailure::UnexpectedError => crate::uart_print(b"unexpected error"),
            }
        }
    }
}
