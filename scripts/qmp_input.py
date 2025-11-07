#!/usr/bin/env python3
"""
QMP Input Injection Helper
Phase 1.2 - Production Readiness Plan

Provides helpers for injecting keyboard input into QEMU via QMP.
This allows automated shell testing without expect.
"""

import socket
import json
import time
import sys
from typing import Optional, Dict, Any


class QMPClient:
    """Simple QMP client for QEMU control."""

    def __init__(self, socket_path: str = "/tmp/sis-qmp.sock"):
        self.socket_path = socket_path
        self.sock: Optional[socket.socket] = None

    def connect(self, timeout: float = 5.0) -> bool:
        """Connect to QMP socket with timeout."""
        start = time.time()

        while time.time() - start < timeout:
            try:
                self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
                self.sock.settimeout(timeout)
                self.sock.connect(self.socket_path)

                # Read QMP greeting
                greeting = self._recv_json()
                if greeting and 'QMP' in greeting:
                    # Send qmp_capabilities
                    self._send_command('qmp_capabilities')
                    return True

            except (FileNotFoundError, ConnectionRefusedError, socket.error):
                time.sleep(0.5)
                if self.sock:
                    try:
                        self.sock.close()
                    except:
                        pass
                    self.sock = None

        return False

    def _send_command(self, command: str, arguments: Optional[Dict[str, Any]] = None) -> Dict:
        """Send QMP command and return response."""
        if not self.sock:
            raise RuntimeError("Not connected to QMP socket")

        cmd = {"execute": command}
        if arguments:
            cmd["arguments"] = arguments

        self.sock.send((json.dumps(cmd) + '\n').encode())
        return self._recv_json()

    def _recv_json(self) -> Dict:
        """Receive JSON response from QMP."""
        if not self.sock:
            raise RuntimeError("Not connected")

        data = b""
        while True:
            chunk = self.sock.recv(4096)
            if not chunk:
                break
            data += chunk
            if b'\n' in data:
                break

        if data:
            return json.loads(data.decode().strip())
        return {}

    def send_key(self, key: str) -> bool:
        """Send a single key press via QMP."""
        try:
            # Map common keys to QMP key codes
            key_map = {
                '\n': 'ret',
                '\r': 'ret',
                ' ': 'spc',
                '\t': 'tab',
            }

            qmp_key = key_map.get(key, key.lower())

            resp = self._send_command('send-key', {
                'keys': [{'type': 'qcode', 'data': qmp_key}]
            })

            return 'error' not in resp

        except Exception as e:
            print(f"Error sending key: {e}", file=sys.stderr)
            return False

    def send_string(self, text: str, delay: float = 0.05) -> bool:
        """Send a string by sending individual key presses."""
        for char in text:
            if not self.send_key(char):
                return False
            time.sleep(delay)  # Small delay between keys
        return True

    def send_command_line(self, command: str) -> bool:
        """Send a command line (string + enter)."""
        return self.send_string(command) and self.send_key('\n')

    def quit_vm(self) -> bool:
        """Send quit command to QEMU."""
        try:
            self._send_command('quit')
            return True
        except:
            return False

    def query_status(self) -> Dict:
        """Query VM status."""
        try:
            return self._send_command('query-status')
        except:
            return {}

    def close(self):
        """Close connection."""
        if self.sock:
            try:
                self.sock.close()
            except:
                pass
            self.sock = None


def main():
    """CLI interface for QMP input injection."""
    import argparse

    parser = argparse.ArgumentParser(description='QMP Input Injection Helper')
    parser.add_argument('--socket', default='/tmp/sis-qmp.sock',
                        help='QMP socket path')
    parser.add_argument('--timeout', type=float, default=5.0,
                        help='Connection timeout')
    parser.add_argument('--delay', type=float, default=0.05,
                        help='Delay between keystrokes (seconds)')

    subparsers = parser.add_subparsers(dest='command', help='Command to execute')

    # send-key command
    key_parser = subparsers.add_parser('send-key', help='Send single key')
    key_parser.add_argument('key', help='Key to send')

    # send-string command
    string_parser = subparsers.add_parser('send-string', help='Send string')
    string_parser.add_argument('text', help='Text to send')

    # send-command command
    cmd_parser = subparsers.add_parser('send-command', help='Send command line')
    cmd_parser.add_argument('command_line', help='Command to send')

    # quit command
    subparsers.add_parser('quit', help='Quit VM')

    # status command
    subparsers.add_parser('status', help='Query VM status')

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        return 1

    # Connect to QMP
    client = QMPClient(args.socket)
    print(f"[*] Connecting to QMP socket: {args.socket}")

    if not client.connect(timeout=args.timeout):
        print(f"[!] Failed to connect to QMP socket", file=sys.stderr)
        return 1

    print("[*] Connected successfully")

    # Execute command
    success = False

    if args.command == 'send-key':
        success = client.send_key(args.key)

    elif args.command == 'send-string':
        success = client.send_string(args.text, delay=args.delay)

    elif args.command == 'send-command':
        success = client.send_command_line(args.command_line)

    elif args.command == 'quit':
        success = client.quit_vm()

    elif args.command == 'status':
        status = client.query_status()
        print(json.dumps(status, indent=2))
        success = True

    client.close()

    if success:
        print("[*] Command executed successfully")
        return 0
    else:
        print("[!] Command failed", file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
