// Self-test shell commands
//
// M8 Driver Hardening - Self-Test Framework

use crate::drivers::{DriverError, DriverResult};
use crate::drivers::selftest::{TestResult, TestFailure, TestCase};

impl super::Shell {
    /// Run all driver self-tests
    pub(crate) fn selftest_all_cmd(&self) {
        unsafe {
            crate::uart_print(b"\n");
            crate::uart_print(b"========================================\n");
            crate::uart_print(b"  Driver Self-Test Suite\n");
            crate::uart_print(b"  M8 Driver Hardening Validation\n");
            crate::uart_print(b"========================================\n\n");
        }

        let mut total_passed = 0;
        let mut total_failed = 0;
        let mut total_skipped = 0;

        // Run GPIO self-tests
        let (p, f, s) = self.selftest_gpio_impl();
        total_passed += p;
        total_failed += f;
        total_skipped += s;

        // Run Mailbox self-tests
        let (p, f, s) = self.selftest_mailbox_impl();
        total_passed += p;
        total_failed += f;
        total_skipped += s;

        // Run PMU self-tests
        let (p, f, s) = self.selftest_pmu_impl();
        total_passed += p;
        total_failed += f;
        total_skipped += s;

        // Summary
        unsafe {
            crate::uart_print(b"\n========================================\n");
            crate::uart_print(b"Test Summary:\n");
            crate::uart_print(b"  Passed:  ");
            self.print_number_simple(total_passed);
            crate::uart_print(b"\n  Failed:  ");
            self.print_number_simple(total_failed);
            crate::uart_print(b"\n  Skipped: ");
            self.print_number_simple(total_skipped);
            crate::uart_print(b"\n  Total:   ");
            self.print_number_simple(total_passed + total_failed + total_skipped);
            crate::uart_print(b"\n========================================\n");

            if total_failed == 0 {
                crate::uart_print(b"\n✓ All tests PASSED\n\n");
            } else {
                crate::uart_print(b"\n✗ Some tests FAILED\n\n");
            }
        }
    }

    /// Run GPIO driver self-tests
    pub(crate) fn selftest_gpio_cmd(&self) {
        self.print_test_header(b"GPIO Driver Self-Test");
        self.selftest_gpio_impl();
    }

    fn selftest_gpio_impl(&self) -> (u64, u64, u64) {
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        // Test 1: Initialization check
        let result = if crate::drivers::gpio::is_initialized() {
            TestResult::Pass
        } else {
            TestResult::Fail(TestFailure::NotInitialized)
        };
        self.print_test_result(b"Initialization check", &result);
        self.count_result(&result, &mut passed, &mut failed, &mut skipped);

        // Test 2: Valid pin operations (pin 0, 27, 53)
        let result = self.test_gpio_valid_operations();
        self.print_test_result(b"Valid pin operations", &result);
        self.count_result(&result, &mut passed, &mut failed, &mut skipped);

        // Test 3: Invalid pin rejection (pins 54, 55, 100)
        let result = self.test_gpio_invalid_pins();
        self.print_test_result(b"Invalid pin rejection", &result);
        self.count_result(&result, &mut passed, &mut failed, &mut skipped);

        // Test 4: Boundary conditions (pin 53)
        let result = self.test_gpio_boundary();
        self.print_test_result(b"Boundary conditions", &result);
        self.count_result(&result, &mut passed, &mut failed, &mut skipped);

        self.print_suite_summary(b"GPIO", passed, failed, skipped);
        (passed, failed, skipped)
    }

    /// Run Mailbox driver self-tests
    pub(crate) fn selftest_mailbox_cmd(&self) {
        self.print_test_header(b"Mailbox Driver Self-Test");
        self.selftest_mailbox_impl();
    }

    fn selftest_mailbox_impl(&self) -> (u64, u64, u64) {
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        // Test 1: Initialization check
        let result = if crate::drivers::firmware::mailbox::is_initialized() {
            TestResult::Pass
        } else {
            TestResult::Fail(TestFailure::NotInitialized)
        };
        self.print_test_result(b"Initialization check", &result);
        self.count_result(&result, &mut passed, &mut failed, &mut skipped);

        // Test 2: Firmware query (get board serial)
        let result = self.test_mailbox_firmware_query();
        self.print_test_result(b"Firmware query", &result);
        self.count_result(&result, &mut passed, &mut failed, &mut skipped);

        // Test 3: Multiple queries
        let result = self.test_mailbox_multiple_queries();
        self.print_test_result(b"Multiple queries", &result);
        self.count_result(&result, &mut passed, &mut failed, &mut skipped);

        self.print_suite_summary(b"Mailbox", passed, failed, skipped);
        (passed, failed, skipped)
    }

    /// Run PMU driver self-tests
    pub(crate) fn selftest_pmu_cmd(&self) {
        self.print_test_header(b"PMU Driver Self-Test");
        self.selftest_pmu_impl();
    }

    fn selftest_pmu_impl(&self) -> (u64, u64, u64) {
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        // Test 1: Initialization check
        let result = if crate::pmu::is_initialized() {
            TestResult::Pass
        } else {
            TestResult::Fail(TestFailure::NotInitialized)
        };
        self.print_test_result(b"Initialization check", &result);
        self.count_result(&result, &mut passed, &mut failed, &mut skipped);

        // Test 2: Snapshot read
        let result = self.test_pmu_snapshot();
        self.print_test_result(b"Snapshot read", &result);
        self.count_result(&result, &mut passed, &mut failed, &mut skipped);

        // Test 3: Counter validation (counters 0-5)
        let result = self.test_pmu_valid_counters();
        self.print_test_result(b"Valid counter read", &result);
        self.count_result(&result, &mut passed, &mut failed, &mut skipped);

        // Test 4: Invalid counter rejection (counter 6, 7, 100)
        let result = self.test_pmu_invalid_counters();
        self.print_test_result(b"Invalid counter rejection", &result);
        self.count_result(&result, &mut passed, &mut failed, &mut skipped);

        self.print_suite_summary(b"PMU", passed, failed, skipped);
        (passed, failed, skipped)
    }

    //
    // GPIO Test Implementations
    //

    fn test_gpio_valid_operations(&self) -> TestResult {
        // Test valid pins: 0, 27, 53
        let test_pins = [0, 27, 53];

        for pin in test_pins {
            // Test set_pin
            if let Err(e) = crate::drivers::gpio::set_pin(pin) {
                return TestResult::Fail(self.driver_error_to_failure(e));
            }

            // Test clear_pin
            if let Err(e) = crate::drivers::gpio::clear_pin(pin) {
                return TestResult::Fail(self.driver_error_to_failure(e));
            }

            // Test read_pin
            if let Err(e) = crate::drivers::gpio::read_pin(pin) {
                return TestResult::Fail(self.driver_error_to_failure(e));
            }
        }

        TestResult::Pass
    }

    fn test_gpio_invalid_pins(&self) -> TestResult {
        // Test invalid pins: 54, 55, 100
        let invalid_pins = [54, 55, 100];

        for pin in invalid_pins {
            // Should return InvalidParameter error
            match crate::drivers::gpio::set_pin(pin) {
                Err(DriverError::InvalidParameter) => {
                    // Expected error - good!
                }
                Ok(()) => {
                    // Should have failed!
                    return TestResult::Fail(TestFailure::ValidationFailed);
                }
                Err(_) => {
                    // Wrong error type
                    return TestResult::Fail(TestFailure::UnexpectedError);
                }
            }
        }

        TestResult::Pass
    }

    fn test_gpio_boundary(&self) -> TestResult {
        // Test boundary pin 53 (max valid pin)
        if let Err(e) = crate::drivers::gpio::set_pin(53) {
            return TestResult::Fail(self.driver_error_to_failure(e));
        }

        // Test pin 54 (first invalid pin)
        match crate::drivers::gpio::set_pin(54) {
            Err(DriverError::InvalidParameter) => TestResult::Pass,
            Ok(()) => TestResult::Fail(TestFailure::BoundaryFailed),
            Err(_) => TestResult::Fail(TestFailure::UnexpectedError),
        }
    }

    //
    // Mailbox Test Implementations
    //

    fn test_mailbox_firmware_query(&self) -> TestResult {
        match crate::drivers::firmware::mailbox::get_board_serial() {
            Ok(_) => TestResult::Pass,
            Err(e) => TestResult::Fail(self.driver_error_to_failure(e)),
        }
    }

    fn test_mailbox_multiple_queries(&self) -> TestResult {
        // Test multiple different queries
        let queries: [fn() -> DriverResult<u32>; 3] = [
            crate::drivers::firmware::mailbox::get_firmware_revision,
            crate::drivers::firmware::mailbox::get_board_model,
            crate::drivers::firmware::mailbox::get_board_revision,
        ];

        for query in queries {
            if let Err(e) = query() {
                return TestResult::Fail(self.driver_error_to_failure(e));
            }
        }

        TestResult::Pass
    }

    //
    // PMU Test Implementations
    //

    fn test_pmu_snapshot(&self) -> TestResult {
        match crate::pmu::read_snapshot() {
            Ok(_) => TestResult::Pass,
            Err(e) => TestResult::Fail(self.driver_error_to_failure(e)),
        }
    }

    fn test_pmu_valid_counters(&self) -> TestResult {
        // Test valid counters 0-5
        for counter in 0..=5 {
            if let Err(e) = crate::pmu::read_event_counter(counter) {
                return TestResult::Fail(self.driver_error_to_failure(e));
            }
        }

        TestResult::Pass
    }

    fn test_pmu_invalid_counters(&self) -> TestResult {
        // Test invalid counters: 6, 7, 100
        let invalid_counters = [6, 7, 100];

        for counter in invalid_counters {
            match crate::pmu::read_event_counter(counter) {
                Err(DriverError::InvalidParameter) => {
                    // Expected error - good!
                }
                Ok(_) => {
                    // Should have failed!
                    return TestResult::Fail(TestFailure::ValidationFailed);
                }
                Err(_) => {
                    // Wrong error type
                    return TestResult::Fail(TestFailure::UnexpectedError);
                }
            }
        }

        TestResult::Pass
    }

    //
    // Helper Functions
    //

    fn driver_error_to_failure(&self, error: DriverError) -> TestFailure {
        match error {
            DriverError::NotInitialized => TestFailure::NotInitialized,
            DriverError::Timeout(_) => TestFailure::Timeout,
            DriverError::HardwareError => TestFailure::HardwareError,
            DriverError::InvalidParameter | DriverError::AlignmentError => TestFailure::ValidationFailed,
            _ => TestFailure::UnexpectedError,
        }
    }

    fn count_result(&self, result: &TestResult, passed: &mut u64, failed: &mut u64, skipped: &mut u64) {
        match result {
            TestResult::Pass => *passed += 1,
            TestResult::Fail(_) => *failed += 1,
            TestResult::Skip => *skipped += 1,
        }
    }

    fn print_test_header(&self, name: &[u8]) {
        unsafe {
            crate::uart_print(b"\n");
            crate::uart_print(b"----------------------------------------\n");
            crate::uart_print(b"  ");
            crate::uart_print(name);
            crate::uart_print(b"\n");
            crate::uart_print(b"----------------------------------------\n");
        }
    }

    fn print_test_result(&self, test_name: &[u8], result: &TestResult) {
        unsafe {
            crate::uart_print(b"  [");

            match result {
                TestResult::Pass => crate::uart_print(b"PASS"),
                TestResult::Fail(_) => crate::uart_print(b"FAIL"),
                TestResult::Skip => crate::uart_print(b"SKIP"),
            }

            crate::uart_print(b"] ");
            crate::uart_print(test_name);

            if let TestResult::Fail(failure) = result {
                crate::uart_print(b" (");
                self.print_failure_reason(*failure);
                crate::uart_print(b")");
            }

            crate::uart_print(b"\n");
        }
    }

    fn print_failure_reason(&self, failure: TestFailure) {
        unsafe {
            match failure {
                TestFailure::NotInitialized => crate::uart_print(b"not initialized"),
                TestFailure::InitFailed => crate::uart_print(b"init failed"),
                TestFailure::HardwareError => crate::uart_print(b"hardware error"),
                TestFailure::ValidationFailed => crate::uart_print(b"validation failed"),
                TestFailure::BoundaryFailed => crate::uart_print(b"boundary check failed"),
                TestFailure::Timeout => crate::uart_print(b"timeout"),
                TestFailure::UnexpectedError => crate::uart_print(b"unexpected error"),
            }
        }
    }

    fn print_suite_summary(&self, driver_name: &[u8], passed: u64, failed: u64, skipped: u64) {
        unsafe {
            crate::uart_print(b"\n  ");
            crate::uart_print(driver_name);
            crate::uart_print(b" Summary: ");
            self.print_number_simple(passed);
            crate::uart_print(b" passed, ");
            self.print_number_simple(failed);
            crate::uart_print(b" failed, ");
            self.print_number_simple(skipped);
            crate::uart_print(b" skipped\n");
        }
    }
}
