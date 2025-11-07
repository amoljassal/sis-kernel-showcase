#!/usr/bin/env python3
"""
Log Normalization Tool for Diff-Friendly Regression Testing
Phase 1.1 - Production Readiness Plan

Normalizes JSON logs by:
- Stripping timestamps (ts field)
- Normalizing runtime values (heap sizes, addresses, etc.)
- Sorting keys for consistent output
"""

import json
import sys
import re
from typing import Dict, Any


def normalize_value(key: str, value: Any) -> Any:
    """Normalize specific values to make logs diff-friendly."""

    # Strip timestamps completely
    if key == 'ts':
        return None

    # Normalize heap/memory values
    if key in ('heap', 'heap_current_bytes', 'heap_peak_bytes', 'memory'):
        if isinstance(value, (int, float)):
            return "X_BYTES"
        return "X_MEMORY"

    # Normalize addresses/pointers
    if key in ('address', 'ptr', 'base_addr') or 'addr' in key:
        return "0xXXXXXXXX"

    # Normalize PIDs/TIDs (process/thread IDs)
    if key in ('pid', 'tid', 'task_id'):
        return 0

    # Normalize timing values
    if 'time' in key or '_ns' in key or '_us' in key or '_ms' in key:
        if key != 'uptime_ms':  # Keep uptime for now
            return "X_TIME"

    # Normalize file descriptors
    if key == 'fd':
        if isinstance(value, int) and value >= 0:
            return "X_FD"

    # Keep everything else as-is
    return value


def normalize_event(event: Dict[str, Any]) -> Dict[str, Any]:
    """Normalize a single JSON event."""
    normalized = {}

    for key, value in event.items():
        normalized_value = normalize_value(key, value)

        # Skip None values (like stripped timestamps)
        if normalized_value is not None:
            normalized[key] = normalized_value

    return normalized


def main():
    """Process stdin line by line, normalizing JSON events."""
    line_num = 0
    errors = 0

    for line in sys.stdin:
        line_num += 1
        line = line.strip()

        # Skip empty lines
        if not line:
            continue

        # Only process lines that look like JSON
        if not line.startswith('{'):
            continue

        try:
            # Parse JSON
            event = json.loads(line)

            # Normalize
            normalized = normalize_event(event)

            # Output with sorted keys for consistency
            print(json.dumps(normalized, sort_keys=True))

        except json.JSONDecodeError as e:
            print(f"# ERROR at line {line_num}: {e}", file=sys.stderr)
            errors += 1
            # Skip malformed JSON
            continue

    if errors > 0:
        print(f"# Processed {line_num} lines with {errors} errors", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
