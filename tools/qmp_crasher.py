#!/usr/bin/env python3
"""
QMP-based crash injector for testing kernel durability.
Connects to QEMU via QMP protocol and injects crashes at random or specified times.
"""

import socket
import json
import time
import random
import argparse
import sys
from typing import Optional, Dict, Any


class QMPClient:
    """Simple QMP (QEMU Machine Protocol) client for crash injection."""

    def __init__(self, host: str = 'localhost', port: int = 4444):
        self.host = host
        self.port = port
        self.socket: Optional[socket.socket] = None

    def connect(self, timeout: float = 5.0) -> bool:
        """Connect to QEMU QMP server."""
        try:
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.socket.settimeout(timeout)
            self.socket.connect((self.host, self.port))

            # Read QMP greeting
            greeting = self._read_response()
            if not greeting or 'QMP' not in greeting:
                print(f"Error: Invalid QMP greeting: {greeting}", file=sys.stderr)
                return False

            # Send capabilities handshake
            if not self.execute('qmp_capabilities'):
                print("Error: Failed to negotiate QMP capabilities", file=sys.stderr)
                return False

            print(f"✓ Connected to QEMU QMP at {self.host}:{self.port}")
            return True

        except (socket.error, socket.timeout) as e:
            print(f"Error connecting to QMP: {e}", file=sys.stderr)
            return False

    def _read_response(self) -> Optional[Dict[str, Any]]:
        """Read JSON response from QMP socket."""
        if not self.socket:
            return None

        try:
            data = b''
            while True:
                chunk = self.socket.recv(4096)
                if not chunk:
                    break
                data += chunk
                # Try to parse as JSON
                try:
                    return json.loads(data.decode('utf-8'))
                except json.JSONDecodeError:
                    # Need more data
                    continue
        except socket.timeout:
            return None

    def execute(self, command: str, arguments: Optional[Dict[str, Any]] = None) -> bool:
        """Execute a QMP command."""
        if not self.socket:
            return False

        cmd = {'execute': command}
        if arguments:
            cmd['arguments'] = arguments

        try:
            self.socket.sendall(json.dumps(cmd).encode('utf-8') + b'\n')
            response = self._read_response()

            if response and 'return' in response:
                return True
            elif response and 'error' in response:
                print(f"QMP error: {response['error']}", file=sys.stderr)
                return False
            else:
                # Some commands don't return anything (like system_reset)
                return True

        except socket.error as e:
            print(f"Socket error: {e}", file=sys.stderr)
            return False

    def hard_reset(self) -> bool:
        """Trigger a hard system reset (simulates power loss)."""
        print("Injecting hard reset (simulating power loss)...")
        return self.execute('system_reset')

    def quit(self) -> bool:
        """Quit QEMU cleanly."""
        return self.execute('quit')

    def close(self):
        """Close QMP connection."""
        if self.socket:
            self.socket.close()
            self.socket = None


def inject_crash_during_workload(
    workload_duration_ms: int,
    crash_time_ms: Optional[int] = None,
    qmp_host: str = 'localhost',
    qmp_port: int = 4444
) -> bool:
    """
    Connect to QEMU via QMP and inject a crash at a specified or random time.

    Args:
        workload_duration_ms: Total duration of workload in milliseconds
        crash_time_ms: Time to inject crash (None = random)
        qmp_host: QMP server host
        qmp_port: QMP server port

    Returns:
        True if crash was injected successfully
    """
    client = QMPClient(qmp_host, qmp_port)

    if not client.connect():
        return False

    # Determine crash time
    if crash_time_ms is None:
        # Random time between 10% and 90% of workload duration
        crash_time_ms = random.randint(
            int(workload_duration_ms * 0.1),
            int(workload_duration_ms * 0.9)
        )

    crash_time_s = crash_time_ms / 1000.0

    print(f"Workload duration: {workload_duration_ms}ms")
    print(f"Crash scheduled at: {crash_time_ms}ms ({crash_time_ms/workload_duration_ms*100:.1f}% of workload)")
    print(f"Waiting {crash_time_s:.2f} seconds...")

    # Wait until crash time
    time.sleep(crash_time_s)

    # Inject crash
    success = client.hard_reset()

    client.close()

    if success:
        print("✓ Crash injected successfully")
    else:
        print("✗ Failed to inject crash", file=sys.stderr)

    return success


def main():
    parser = argparse.ArgumentParser(
        description='QMP-based crash injector for kernel durability testing'
    )
    parser.add_argument(
        '--duration',
        type=int,
        default=5000,
        help='Workload duration in milliseconds (default: 5000)'
    )
    parser.add_argument(
        '--crash-at',
        type=int,
        help='Crash at specific time in ms (default: random)'
    )
    parser.add_argument(
        '--host',
        default='localhost',
        help='QMP server host (default: localhost)'
    )
    parser.add_argument(
        '--port',
        type=int,
        default=4444,
        help='QMP server port (default: 4444)'
    )
    parser.add_argument(
        '--immediate',
        action='store_true',
        help='Crash immediately (for testing)'
    )

    args = parser.parse_args()

    if args.immediate:
        crash_time = 0
    else:
        crash_time = args.crash_at

    success = inject_crash_during_workload(
        workload_duration_ms=args.duration,
        crash_time_ms=crash_time,
        qmp_host=args.host,
        qmp_port=args.port
    )

    sys.exit(0 if success else 1)


if __name__ == '__main__':
    main()
