#!/usr/bin/env python3
"""
Soak Test Report Generator
Phase 2.3 - Production Readiness Plan

Analyzes soak test metrics and generates HTML report with:
- Pass/fail statistics
- Performance trends over time
- Memory usage analysis
- Anomaly detection
"""

import sys
import statistics
from collections import defaultdict
from typing import Dict, List, Tuple


def parse_metrics_log(filename: str) -> Tuple[List[Dict], Dict[str, List]]:
    """Parse soak test metrics log."""
    runs = []
    metrics = defaultdict(list)

    with open(filename) as f:
        for line in f:
            line = line.strip()

            # Skip comments and empty lines
            if not line or line.startswith('#'):
                continue

            # Parse CSV format: run_num,timestamp,result,boot_time_ms
            if ',' in line and not line.startswith('METRIC'):
                parts = line.split(',')
                if len(parts) >= 4:
                    try:
                        run_data = {
                            'run_num': int(parts[0]),
                            'timestamp': int(parts[1]),
                            'result': parts[2],
                            'boot_time_ms': int(parts[3]) if parts[3].isdigit() else 0
                        }
                        runs.append(run_data)

                        # Collect boot time metrics
                        if run_data['boot_time_ms'] > 0:
                            metrics['boot_time_ms'].append(run_data['boot_time_ms'])

                    except (ValueError, IndexError):
                        pass

            # Parse metric lines: METRIC key=value
            elif 'METRIC' in line or '=' in line:
                try:
                    # Extract key=value pairs
                    parts = line.split('METRIC')[1].strip() if 'METRIC' in line else line
                    if '=' in parts:
                        key, value = parts.split('=', 1)
                        key = key.strip()
                        value = value.strip()

                        try:
                            metrics[key].append(float(value))
                        except ValueError:
                            # Skip non-numeric values
                            pass
                except (IndexError, ValueError):
                    pass

    return runs, metrics


def calculate_statistics(values: List[float]) -> Dict[str, float]:
    """Calculate statistical summary for a list of values."""
    if not values or len(values) < 2:
        return {}

    return {
        'count': len(values),
        'min': min(values),
        'max': max(values),
        'mean': statistics.mean(values),
        'median': statistics.median(values),
        'stdev': statistics.stdev(values) if len(values) > 1 else 0,
        'p95': sorted(values)[int(len(values) * 0.95)] if len(values) > 20 else max(values),
        'p99': sorted(values)[int(len(values) * 0.99)] if len(values) > 100 else max(values),
    }


def detect_anomalies(values: List[float], threshold: float = 3.0) -> List[int]:
    """Detect anomalies using standard deviation method."""
    if len(values) < 10:
        return []

    mean = statistics.mean(values)
    stdev = statistics.stdev(values)

    anomalies = []
    for i, value in enumerate(values):
        z_score = abs(value - mean) / stdev if stdev > 0 else 0
        if z_score > threshold:
            anomalies.append(i)

    return anomalies


def generate_html_report(runs: List[Dict], metrics: Dict[str, List]) -> str:
    """Generate HTML report."""
    html = """
<!DOCTYPE html>
<html>
<head>
    <title>SIS Kernel Soak Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; background: white; padding: 30px; box-shadow: 0 0 10px rgba(0,0,0,0.1); }
        h1 { color: #333; border-bottom: 3px solid #4CAF50; padding-bottom: 10px; }
        h2 { color: #555; border-bottom: 2px solid #ddd; padding-bottom: 8px; margin-top: 30px; }
        table { width: 100%; border-collapse: collapse; margin: 20px 0; }
        th, td { padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }
        th { background: #4CAF50; color: white; font-weight: bold; }
        tr:hover { background: #f5f5f5; }
        .metric-box { background: #f9f9f9; border-left: 4px solid #4CAF50; padding: 15px; margin: 10px 0; }
        .metric-title { font-weight: bold; color: #333; margin-bottom: 10px; }
        .metric-value { font-size: 24px; color: #4CAF50; font-weight: bold; }
        .stats-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin: 20px 0; }
        .stat-card { background: #fff; border: 1px solid #ddd; border-radius: 8px; padding: 15px; }
        .stat-label { color: #666; font-size: 12px; text-transform: uppercase; }
        .stat-value { font-size: 28px; font-weight: bold; color: #333; margin: 5px 0; }
        .pass { color: #4CAF50; }
        .fail { color: #f44336; }
        .warn { color: #ff9800; }
        .anomaly { background: #fff3cd; border-left-color: #ff9800; }
    </style>
</head>
<body>
<div class="container">
    """

    # Header
    html += f"""
    <h1>SIS Kernel Soak Test Report</h1>
    <p><strong>Generated:</strong> {sys.argv[1] if len(sys.argv) > 1 else 'N/A'}</p>
    """

    # Summary statistics
    total_runs = len(runs)
    pass_count = sum(1 for r in runs if r['result'] == 'PASS')
    fail_count = sum(1 for r in runs if 'FAIL' in r['result'])
    timeout_count = sum(1 for r in runs if r['result'] == 'TIMEOUT')

    pass_rate = (pass_count * 100 / total_runs) if total_runs > 0 else 0
    fail_rate = ((fail_count + timeout_count) * 100 / total_runs) if total_runs > 0 else 0

    html += f"""
    <h2>Summary</h2>
    <div class="stats-grid">
        <div class="stat-card">
            <div class="stat-label">Total Runs</div>
            <div class="stat-value">{total_runs}</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Passed</div>
            <div class="stat-value pass">{pass_count}</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Failed</div>
            <div class="stat-value fail">{fail_count}</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Timeouts</div>
            <div class="stat-value warn">{timeout_count}</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Pass Rate</div>
            <div class="stat-value {'pass' if pass_rate >= 95 else 'fail'}">{pass_rate:.1f}%</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Fail Rate</div>
            <div class="stat-value {'pass' if fail_rate <= 5 else 'fail'}">{fail_rate:.1f}%</div>
        </div>
    </div>
    """

    # Performance metrics
    html += "<h2>Performance Metrics</h2>"

    for metric_name, values in sorted(metrics.items()):
        if not values or len(values) < 2:
            continue

        stats = calculate_statistics(values)
        anomalies = detect_anomalies(values)

        anomaly_class = " anomaly" if anomalies else ""

        html += f"""
        <div class="metric-box{anomaly_class}">
            <div class="metric-title">{metric_name}</div>
            <table>
                <tr>
                    <th>Metric</th>
                    <th>Value</th>
                </tr>
                <tr><td>Count</td><td>{stats['count']}</td></tr>
                <tr><td>Mean</td><td>{stats['mean']:.2f}</td></tr>
                <tr><td>Median</td><td>{stats['median']:.2f}</td></tr>
                <tr><td>Std Dev</td><td>{stats['stdev']:.2f}</td></tr>
                <tr><td>Min</td><td>{stats['min']:.2f}</td></tr>
                <tr><td>Max</td><td>{stats['max']:.2f}</td></tr>
                <tr><td>P95</td><td>{stats['p95']:.2f}</td></tr>
                <tr><td>P99</td><td>{stats['p99']:.2f}</td></tr>
            </table>
        """

        if anomalies:
            html += f"""
            <p class="warn"><strong>⚠ Warning:</strong> {len(anomalies)} anomalies detected
            (values >3σ from mean)</p>
            """

        html += "</div>"

    # Recent failures
    if fail_count > 0 or timeout_count > 0:
        html += "<h2>Failed Runs</h2>"
        html += "<table><tr><th>Run</th><th>Timestamp</th><th>Result</th><th>Boot Time (ms)</th></tr>"

        for run in runs:
            if 'FAIL' in run['result'] or run['result'] == 'TIMEOUT':
                html += f"""
                <tr>
                    <td>{run['run_num']}</td>
                    <td>{run['timestamp']}</td>
                    <td class="fail">{run['result']}</td>
                    <td>{run['boot_time_ms']}</td>
                </tr>
                """

        html += "</table>"

    # Conclusion
    html += "<h2>Conclusion</h2>"
    if pass_rate >= 95:
        html += '<p class="pass"><strong>✓ PASS:</strong> Soak test passed with acceptable failure rate.</p>'
    else:
        html += '<p class="fail"><strong>✗ FAIL:</strong> Soak test failed - failure rate too high.</p>'

    html += """
</div>
</body>
</html>
    """

    return html


def main():
    if len(sys.argv) < 2:
        print("Usage: soak_report.py <metrics_log_file>", file=sys.stderr)
        sys.exit(1)

    metrics_file = sys.argv[1]

    try:
        runs, metrics = parse_metrics_log(metrics_file)
        html = generate_html_report(runs, metrics)
        print(html)

    except FileNotFoundError:
        print(f"Error: Metrics file not found: {metrics_file}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Error generating report: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
