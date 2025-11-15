#!/usr/bin/env python3
"""
SIS Kernel Test Results Validation Script

This script validates that test results meet minimum production requirements
and fails CI if critical thresholds are not met.
"""

import json
import sys
import argparse
from typing import Dict, Any, List, Tuple

class ValidationError(Exception):
    """Raised when validation fails"""
    pass

class SISKernelValidator:
    """Validates SIS Kernel test results against production thresholds"""
    
    # Production readiness thresholds
    THRESHOLDS = {
        'overall_score_minimum': 80.0,  # Overall score must be > 80%
        'security_coverage_minimum': 100.0,  # Security must be 100%
        'correctness_coverage_minimum': 100.0,  # Correctness must be 100%
        'performance_coverage_minimum': 50.0,  # Performance at least 50%
        'ai_coverage_minimum': 95.0,  # AI coverage at least 95%
        'memory_safety_violations_maximum': 0,  # Zero violations allowed
        'critical_vulnerabilities_maximum': 0,  # Zero critical vulnerabilities
        'ai_accuracy_minimum': 99.9,  # AI accuracy > 99.9%
        'max_acceptable_failures': 2,  # Maximum 2 failing tests allowed
    }
    
    def __init__(self, results_file: str):
        self.results_file = results_file
        self.results = self._load_results()
        self.validation_errors: List[str] = []
        self.warnings: List[str] = []
    
    def _load_results(self) -> Dict[str, Any]:
        """Load test results from JSON file"""
        try:
            with open(self.results_file, 'r') as f:
                return json.load(f)
        except FileNotFoundError:
            raise ValidationError(f"Results file not found: {self.results_file}")
        except json.JSONDecodeError as e:
            raise ValidationError(f"Invalid JSON in results file: {e}")
    
    def validate_overall_score(self) -> None:
        """Validate overall test score meets minimum threshold"""
        overall_score = self.results.get('summary', {}).get('overall_score', 0.0)
        min_score = self.THRESHOLDS['overall_score_minimum']
        
        if overall_score < min_score:
            self.validation_errors.append(
                f"Overall score ({overall_score:.1f}%) below minimum threshold ({min_score}%)"
            )
        else:
            print(f"[PASS] Overall score ({overall_score:.1f}%) meets threshold")
    
    def validate_coverage(self) -> None:
        """Validate test coverage across all categories"""
        coverage = self.results.get('coverage', {})
        
        coverage_checks = [
            ('security_coverage', 'security_coverage_minimum', 'Security'),
            ('correctness_coverage', 'correctness_coverage_minimum', 'Correctness'),
            ('performance_coverage', 'performance_coverage_minimum', 'Performance'),
            ('ai_coverage', 'ai_coverage_minimum', 'AI/ML'),
        ]
        
        for coverage_key, threshold_key, name in coverage_checks:
            actual = coverage.get(coverage_key, 0.0) * 100  # Convert to percentage
            required = self.THRESHOLDS[threshold_key]
            
            if actual < required:
                if threshold_key in ['security_coverage_minimum', 'correctness_coverage_minimum']:
                    self.validation_errors.append(
                        f"{name} coverage ({actual:.1f}%) below REQUIRED threshold ({required}%)"
                    )
                else:
                    self.warnings.append(
                        f"{name} coverage ({actual:.1f}%) below recommended threshold ({required}%)"
                    )
            else:
                print(f"[PASS] {name} coverage ({actual:.1f}%) meets threshold")
    
    def validate_security_metrics(self) -> None:
        """Validate critical security metrics"""
        # Check for memory safety violations
        memory_violations = 0
        for result in self.results.get('validation_results', []):
            if 'Memory Safety' in result.get('claim', ''):
                if not result.get('passed', True):
                    memory_violations += 1
        
        if memory_violations > self.THRESHOLDS['memory_safety_violations_maximum']:
            self.validation_errors.append(
                f"Memory safety violations ({memory_violations}) exceed maximum allowed "
                f"({self.THRESHOLDS['memory_safety_violations_maximum']})"
            )
        else:
            print(f"[PASS] Memory safety: {memory_violations} violations (within limits)")
        
        # Check for critical vulnerabilities
        critical_vulns = 0
        for result in self.results.get('validation_results', []):
            if 'Vulnerabilities' in result.get('claim', ''):
                if not result.get('passed', True):
                    critical_vulns += 1
        
        if critical_vulns > self.THRESHOLDS['critical_vulnerabilities_maximum']:
            self.validation_errors.append(
                f"Critical vulnerabilities ({critical_vulns}) exceed maximum allowed "
                f"({self.THRESHOLDS['critical_vulnerabilities_maximum']})"
            )
        else:
            print(f"[PASS] Security: {critical_vulns} critical vulnerabilities")
    
    def validate_performance_metrics(self) -> None:
        """Validate performance metrics within acceptable bounds"""
        performance_failures = []
        
        for result in self.results.get('validation_results', []):
            claim = result.get('claim', '')
            if any(perf_indicator in claim for perf_indicator in ['Inference', 'Context Switch']):
                if not result.get('passed', True):
                    performance_failures.append(claim)
        
        if len(performance_failures) > 0:
            self.warnings.append(
                f"Performance tests failing: {', '.join(performance_failures)}"
            )
        else:
            print("[PASS] All performance tests passing")
    
    def validate_ai_metrics(self) -> None:
        """Validate AI/ML accuracy and performance"""
        for result in self.results.get('validation_results', []):
            claim = result.get('claim', '')
            if 'Inference Accuracy' in claim:
                if result.get('passed', True):
                    # Extract accuracy from measured value
                    measured = result.get('measured', '0%')
                    try:
                        accuracy = float(measured.replace('%', ''))
                        min_accuracy = self.THRESHOLDS['ai_accuracy_minimum']
                        
                        if accuracy < min_accuracy:
                            self.validation_errors.append(
                                f"AI inference accuracy ({accuracy}%) below minimum ({min_accuracy}%)"
                            )
                        else:
                            print(f"[PASS] AI accuracy ({accuracy}%) meets threshold")
                    except ValueError:
                        self.warnings.append(f"Could not parse AI accuracy: {measured}")
    
    def validate_test_execution(self) -> None:
        """Validate test execution completed successfully"""
        total_failures = 0
        failed_tests = []
        
        for result in self.results.get('validation_results', []):
            if not result.get('passed', True):
                total_failures += 1
                failed_tests.append(result.get('claim', 'Unknown test'))
        
        max_failures = self.THRESHOLDS['max_acceptable_failures']
        if total_failures > max_failures:
            self.validation_errors.append(
                f"Too many test failures ({total_failures}) - maximum allowed: {max_failures}"
            )
            self.validation_errors.append(f"Failed tests: {', '.join(failed_tests)}")
        else:
            print(f"[PASS] Test failures ({total_failures}) within acceptable limits ({max_failures})")
    
    def validate_data_integrity(self) -> None:
        """Validate test data integrity and completeness"""
        required_sections = ['summary', 'coverage', 'validation_results']
        missing_sections = []
        
        for section in required_sections:
            if section not in self.results:
                missing_sections.append(section)
        
        if missing_sections:
            self.validation_errors.append(
                f"Missing required data sections: {', '.join(missing_sections)}"
            )
        else:
            print("[PASS] All required data sections present")
        
        # Validate we have actual test results
        validation_results = self.results.get('validation_results', [])
        if len(validation_results) == 0:
            self.validation_errors.append("No validation results found in test output")
        else:
            print(f"[PASS] Found {len(validation_results)} validation results")
    
    def run_validation(self) -> Tuple[bool, List[str], List[str]]:
        """Run all validation checks and return results"""
        print("Starting SIS Kernel test results validation...")
        print(f"Validating results from: {self.results_file}")
        print("=" * 60)
        
        try:
            # Core validation checks
            self.validate_data_integrity()
            self.validate_overall_score()
            self.validate_coverage()
            self.validate_security_metrics()
            self.validate_performance_metrics()
            self.validate_ai_metrics()
            self.validate_test_execution()
            
            # Summary
            print("=" * 60)
            if self.validation_errors:
                print(f"[FAIL] Validation FAILED with {len(self.validation_errors)} errors")
                for error in self.validation_errors:
                    print(f"   ERROR: {error}")
            else:
                print("[PASS] All validation checks PASSED")
            
            if self.warnings:
                print(f"[WARN] {len(self.warnings)} warnings:")
                for warning in self.warnings:
                    print(f"   WARNING: {warning}")
            
            return len(self.validation_errors) == 0, self.validation_errors, self.warnings
            
        except Exception as e:
            error_msg = f"Validation failed due to unexpected error: {e}"
            return False, [error_msg], []

def main():
    parser = argparse.ArgumentParser(description='Validate SIS Kernel test results')
    parser.add_argument('results_file', help='Path to test results JSON file')
    parser.add_argument('--strict', action='store_true', 
                       help='Treat warnings as errors')
    parser.add_argument('--output-format', choices=['text', 'json'], default='text',
                       help='Output format')
    
    args = parser.parse_args()
    
    try:
        validator = SISKernelValidator(args.results_file)
        success, errors, warnings = validator.run_validation()
        
        # In strict mode, warnings become errors
        if args.strict and warnings:
            success = False
            errors.extend(warnings)
            warnings = []
        
        if args.output_format == 'json':
            result = {
                'success': success,
                'errors': errors,
                'warnings': warnings,
                'file': args.results_file
            }
            print(json.dumps(result, indent=2))
        
        # Exit with appropriate code
        sys.exit(0 if success else 1)
        
    except ValidationError as e:
        print(f"[ERROR] Validation Error: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"[ERROR] Unexpected error: {e}", file=sys.stderr)
        sys.exit(2)

if __name__ == '__main__':
    main()