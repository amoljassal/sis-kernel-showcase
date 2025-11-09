#!/usr/bin/env python3
"""
Stress Test Results Validation Script

Validates stress test results for CI/CD integration. This script ensures that
stress tests meet quality criteria for variability, autonomy impact, and performance.

Usage:
    python3 scripts/validate_stress_results.py <results_file.json> [options]

Example:
    python3 scripts/validate_stress_results.py stress_results.json \\
        --max-oom-events 10 \\
        --min-autonomy-interventions 5 \\
        --max-p99-latency-ms 5 \\
        --min-learning-improvement-pct 15
"""

import json
import sys
import argparse
from typing import Dict, List, Any, Tuple


class ValidationError:
    """Represents a validation failure"""
    def __init__(self, message: str, severity: str = "error"):
        self.message = message
        self.severity = severity  # "error" or "warning"

    def __str__(self):
        icon = "✗" if self.severity == "error" else "⚠"
        return f"  {icon} {self.message}"


def validate_memory_test(results: Dict[str, Any], args: argparse.Namespace) -> List[ValidationError]:
    """Validate memory stress test results"""
    failures = []

    if 'memory' not in results:
        return [ValidationError("Memory test results not found")]

    mem = results['memory']

    # Check for variability (should not be deterministic)
    if mem.get('peak_pressure') == 100 and mem.get('oom_events') == 4:
        failures.append(ValidationError(
            "Memory test shows deterministic behavior (always 100% pressure, 4 OOMs)",
            "warning"
        ))

    # Check OOM events threshold
    if mem.get('oom_events', 0) > args.max_oom_events:
        failures.append(ValidationError(
            f"Too many OOM events: {mem['oom_events']} > {args.max_oom_events}"
        ))

    # Check that average pressure is tracked
    if 'avg_memory_pressure' not in mem or mem['avg_memory_pressure'] == 0:
        failures.append(ValidationError(
            "Average memory pressure not tracked",
            "warning"
        ))

    # Check latency tracking
    if 'latency_p99_ns' in mem:
        if mem['latency_p99_ns'] == 0:
            failures.append(ValidationError(
                "Latency percentiles not tracked",
                "warning"
            ))
        elif mem['latency_p99_ns'] / 1_000_000 > args.max_p99_latency_ms:
            failures.append(ValidationError(
                f"p99 latency too high: {mem['latency_p99_ns'] / 1_000_000:.2f}ms > {args.max_p99_latency_ms}ms"
            ))

    return failures


def validate_chaos_test(results: Dict[str, Any], args: argparse.Namespace) -> List[ValidationError]:
    """Validate chaos engineering test results"""
    failures = []

    if 'chaos' not in results:
        return []  # Chaos test is optional

    chaos = results['chaos']

    # Check for variability in event count
    if chaos.get('chaos_events_count') == 265:
        failures.append(ValidationError(
            "Chaos test shows deterministic event count (always 265)",
            "warning"
        ))

    # Check success rate
    total = chaos.get('successful_recoveries', 0) + chaos.get('failed_recoveries', 0)
    if total > 0:
        success_rate = (chaos.get('successful_recoveries', 0) / total) * 100
        if success_rate < 50:
            failures.append(ValidationError(
                f"Chaos test success rate too low: {success_rate:.1f}% < 50%"
            ))

    # Check that recovery latencies are tracked
    if 'latency_p95_ns' in chaos and chaos['latency_p95_ns'] == 0:
        failures.append(ValidationError(
            "Recovery latencies not tracked",
            "warning"
        ))

    return failures


def validate_autonomy_impact(results: Dict[str, Any], args: argparse.Namespace) -> List[ValidationError]:
    """Validate that autonomy shows measurable impact"""
    failures = []

    if 'compare' not in results:
        return []  # Comparison test is optional

    comp = results['compare']

    # Check for autonomy interventions
    if 'autonomy_on' in comp:
        interventions = comp['autonomy_on'].get('interventions', {}).get('total', 0)
        if interventions < args.min_autonomy_interventions:
            failures.append(ValidationError(
                f"Too few autonomy interventions: {interventions} < {args.min_autonomy_interventions}"
            ))

        # Check that autonomy makes a difference
        if 'autonomy_off' in comp:
            on_oom = comp['autonomy_on'].get('oom_events', 0)
            off_oom = comp['autonomy_off'].get('oom_events', 0)

            if on_oom == off_oom and off_oom > 0:
                failures.append(ValidationError(
                    f"Autonomy shows no impact on OOM events (both: {on_oom})",
                    "warning"
                ))

    return failures


def validate_learning_test(results: Dict[str, Any], args: argparse.Namespace) -> List[ValidationError]:
    """Validate learning/RL test results"""
    failures = []

    if 'learning' not in results:
        return []  # Learning test is optional

    learn = results['learning']

    # Check for reward progression
    if 'episodes' in learn and len(learn['episodes']) > 1:
        first_reward = learn['episodes'][0].get('reward', 0)
        last_reward = learn['episodes'][-1].get('reward', 0)

        # Check if all rewards are identical (suggests stubbed implementation)
        all_same = all(ep.get('reward') == first_reward for ep in learn['episodes'])
        if all_same:
            failures.append(ValidationError(
                f"All learning rewards identical ({first_reward}) - suggests stubbed implementation",
                "warning"
            ))

        # Check for improvement
        if first_reward > 0:
            improvement_pct = ((last_reward - first_reward) / first_reward) * 100
            if improvement_pct < args.min_learning_improvement_pct:
                failures.append(ValidationError(
                    f"Insufficient learning improvement: {improvement_pct:.1f}% < {args.min_learning_improvement_pct}%",
                    "warning"
                ))

    return failures


def validate_variability(results: Dict[str, Any], historical: List[Dict[str, Any]]) -> List[ValidationError]:
    """Validate that results show variability across runs"""
    failures = []

    if len(historical) < 2:
        return []  # Need at least 2 runs to check variability

    # Check memory test variability
    memory_results = [r.get('memory') for r in historical if 'memory' in r]
    if len(memory_results) >= 3:
        peak_pressures = [m['peak_pressure'] for m in memory_results if 'peak_pressure' in m]
        if peak_pressures and len(set(peak_pressures)) == 1:
            failures.append(ValidationError(
                f"No variability in memory peak pressure across {len(peak_pressures)} runs",
                "warning"
            ))

        oom_events = [m['oom_events'] for m in memory_results if 'oom_events' in m]
        if oom_events and len(set(oom_events)) == 1:
            failures.append(ValidationError(
                f"No variability in OOM events across {len(oom_events)} runs",
                "warning"
            ))

    return failures


def load_results(path: str) -> Dict[str, Any]:
    """Load stress test results from JSON file"""
    try:
        with open(path, 'r') as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"[ERROR] Results file not found: {path}")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"[ERROR] Invalid JSON in results file: {e}")
        sys.exit(1)


def main():
    parser = argparse.ArgumentParser(description='Validate stress test results for CI/CD')
    parser.add_argument('results_path', help='Path to stress test results JSON file')

    # Thresholds
    parser.add_argument('--max-oom-events', type=int, default=10,
                        help='Maximum allowed OOM events (default: 10)')
    parser.add_argument('--min-autonomy-interventions', type=int, default=5,
                        help='Minimum required autonomy interventions (default: 5)')
    parser.add_argument('--max-p99-latency-ms', type=float, default=5.0,
                        help='Maximum allowed p99 latency in milliseconds (default: 5.0)')
    parser.add_argument('--min-learning-improvement-pct', type=float, default=15.0,
                        help='Minimum required learning improvement percentage (default: 15.0)')

    # Options
    parser.add_argument('--historical', type=str,
                        help='Path to historical results for variability checking')
    parser.add_argument('--warnings-as-errors', action='store_true',
                        help='Treat warnings as errors')
    parser.add_argument('--verbose', action='store_true',
                        help='Verbose output')

    args = parser.parse_args()

    # Load results
    results = load_results(args.results_path)
    historical = []
    if args.historical:
        try:
            with open(args.historical, 'r') as f:
                historical = json.load(f)
        except:
            pass  # Historical data is optional

    if args.verbose:
        print(f"[INFO] Validating stress test results from: {args.results_path}")
        print(f"[INFO] Thresholds:")
        print(f"  - Max OOM events: {args.max_oom_events}")
        print(f"  - Min autonomy interventions: {args.min_autonomy_interventions}")
        print(f"  - Max p99 latency: {args.max_p99_latency_ms}ms")
        print(f"  - Min learning improvement: {args.min_learning_improvement_pct}%")
        print()

    # Run validations
    all_failures: List[ValidationError] = []

    all_failures.extend(validate_memory_test(results, args))
    all_failures.extend(validate_chaos_test(results, args))
    all_failures.extend(validate_autonomy_impact(results, args))
    all_failures.extend(validate_learning_test(results, args))
    all_failures.extend(validate_variability(results, historical))

    # Separate errors and warnings
    errors = [f for f in all_failures if f.severity == "error"]
    warnings = [f for f in all_failures if f.severity == "warning"]

    # Print results
    if errors or (warnings and args.warnings_as_errors):
        print("[FAIL] Stress test validation failed:")
        print()

        if errors:
            print("Errors:")
            for error in errors:
                print(error)

        if warnings:
            print("\nWarnings:")
            for warning in warnings:
                print(warning)

        if warnings and args.warnings_as_errors:
            print("\n(Warnings treated as errors)")

        sys.exit(1)
    elif warnings:
        print("[PASS] Stress tests passed with warnings:")
        print()
        for warning in warnings:
            print(warning)
        print()
        sys.exit(0)
    else:
        print("[PASS] ✓ All stress test validations passed")

        if args.verbose:
            print()
            print("Validated:")
            if 'memory' in results:
                print("  ✓ Memory test")
            if 'chaos' in results:
                print("  ✓ Chaos test")
            if 'compare' in results:
                print("  ✓ Autonomy comparison")
            if 'learning' in results:
                print("  ✓ Learning test")

        sys.exit(0)


if __name__ == '__main__':
    main()
