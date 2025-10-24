#!/usr/bin/env python3
import argparse
import socket
import struct
import sys
import time

SOCK_PATH = '/tmp/sis-datactl.sock'

def frame(cmd: int, payload: bytes, flags: int = 0) -> bytes:
    # magic 'C'(0x43), ver=0, cmd, flags, len (LE u32), payload
    hdr = struct.pack('<BBBBI', 0x43, 0, cmd, flags, len(payload))
    return hdr + payload

def with_token(payload: bytes, token: int) -> bytes:
    return struct.pack('<Q', token) + payload

import time

def recv_until_nl(sock: socket.socket, max_bytes: int = 512, timeout: float = 2.0) -> bytes:
    sock.settimeout(timeout)
    data = b''
    try:
        while len(data) < max_bytes:
            chunk = sock.recv(max_bytes - len(data))
            if not chunk:
                break
            data += chunk
            if b'\n' in chunk:
                break
    except socket.timeout:
        pass
    return data

def send_frame(cmd: int, payload: bytes, wait_ack: bool = False, tcp: str | None = None, retries: int = 0):
    data = frame(cmd, payload)
    attempts = retries + 1
    for i in range(attempts):
        try:
            if tcp:
                host, port_str = tcp.split(":", 1)
                port = int(port_str)
                with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                    s.connect((host, port))
                    s.sendall(data)
                    if wait_ack:
                        resp = recv_until_nl(s)
                        if resp:
                            sys.stdout.write(f"ACK: {resp.decode(errors='ignore').rstrip()}\n")
                        else:
                            if i == attempts - 1:
                                sys.stdout.write("ACK: <timeout>\n")
            else:
                with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
                    s.connect(SOCK_PATH)
                    s.sendall(data)
                    if wait_ack:
                        resp = recv_until_nl(s)
                        if resp:
                            sys.stdout.write(f"ACK: {resp.decode(errors='ignore').rstrip()}\n")
                        else:
                            if i == attempts - 1:
                                sys.stdout.write("ACK: <timeout>\n")
            break
        except (ConnectionRefusedError, BrokenPipeError, socket.timeout) as e:
            if i == attempts - 1:
                raise
            time.sleep(0.25)

def cmd_create_graph(args):
    pl = with_token(b'', args.token)
    send_frame(0x01, pl, args.wait_ack)
    print('CreateGraph sent')

def cmd_add_channel(args):
    cap = int(args.capacity)
    if cap < 1 or cap > 65535:
        raise SystemExit('capacity must be 1..65535')
    payload = with_token(struct.pack('<H', cap), args.token)
    send_frame(0x02, payload, args.wait_ack)
    print(f'AddChannel(capacity={cap}) sent')

def cmd_add_operator(args):
    op_id = int(args.op_id)
    in_ch = int(args.in_ch) if args.in_ch is not None else 0xFFFF
    out_ch = int(args.out_ch) if args.out_ch is not None else 0xFFFF
    prio = int(args.priority)
    stage_map = {
        'acquire': 0,
        'clean': 1,
        'explore': 2,
        'model': 3,
        'explain': 4,
        None: 0,
    }
    stage = stage_map.get(args.stage, 0)
    if args.in_schema is not None or args.out_schema is not None:
        in_schema = int(args.in_schema) if args.in_schema is not None else 0
        out_schema = int(args.out_schema) if args.out_schema is not None else 0
        payload = with_token(struct.pack('<IHHBBII', op_id, in_ch, out_ch, prio, stage, in_schema, out_schema), args.token)
        send_frame(0x05, payload, args.wait_ack)
        print(f'AddOperatorTyped(op_id={op_id}, in={in_ch}, out={out_ch}, prio={prio}, stage={stage}, in_schema={in_schema}, out_schema={out_schema}) sent')
    else:
        payload = with_token(struct.pack('<IHHBB', op_id, in_ch, out_ch, prio, stage), args.token)
        send_frame(0x03, payload, args.wait_ack)
        print(f'AddOperator(op_id={op_id}, in={in_ch}, out={out_ch}, prio={prio}, stage={stage}) sent')

def cmd_start(args):
    steps = int(args.steps)
    payload = with_token(struct.pack('<I', steps), args.token)
    send_frame(0x04, payload, args.wait_ack)
    print(f'StartGraph(steps={steps}) sent')

def cmd_det(args):
    wcet = int(args.wcet_ns)
    period = int(args.period_ns)
    deadline = int(args.deadline_ns)
    payload = with_token(struct.pack('<QQQ', wcet, period, deadline), args.token)
    send_frame(0x06, payload, args.wait_ack)
    print(f'EnableDeterministic(wcet_ns={wcet}, period_ns={period}, deadline_ns={deadline}) sent')

def cmd_llm_load(args):
    wcet = int(args.wcet_cycles) if args.wcet_cycles is not None else None
    if wcet is None:
        payload = with_token(b'', args.token)
    else:
        payload = with_token(struct.pack('<Q', wcet), args.token)
    send_frame(0x10, payload, args.wait_ack, getattr(args, 'tcp', None), getattr(args, 'retries', 0))
    print(f'LLMLoad(wcet_cycles={wcet if wcet is not None else "default"}) sent')

def cmd_llm_infer(args):
    max_tokens = int(args.max_tokens)
    prompt_bytes = args.prompt.encode('utf-8')
    if len(prompt_bytes) + 8 + 2 > 64:
        raise SystemExit('prompt too long for control frame (max ~54 bytes)')
    payload = with_token(struct.pack('<H', max_tokens) + prompt_bytes, args.token)
    send_frame(0x11, payload, args.wait_ack, getattr(args, 'tcp', None), getattr(args, 'retries', 0))
    print(f'LLMInferStart(max_tokens={max_tokens}, prompt_len={len(prompt_bytes)}) sent')

def cmd_llm_poll(args):
    infer_id = int(args.infer_id)
    payload = with_token(struct.pack('<I', infer_id), args.token)
    send_frame(0x12, payload, args.wait_ack, getattr(args, 'tcp', None), getattr(args, 'retries', 0))
    print(f'LLMInferPoll(id={infer_id}) sent')

def cmd_llm_cancel(args):
    infer_id = int(args.infer_id)
    payload = with_token(struct.pack('<I', infer_id), args.token)
    send_frame(0x13, payload, args.wait_ack, getattr(args, 'tcp', None), getattr(args, 'retries', 0))
    print(f'LLMCancel(id={infer_id}) sent')

def cmd_llm_hash(args):
    model_id = int(args.model_id)
    size_bytes = int(args.size) if args.size else 1024
    data = bytes([(model_id & 0xFF)] * size_bytes)
    checksum = 0
    for byte in data:
        checksum = (checksum + byte) & 0xFFFFFFFFFFFFFFFF
        checksum = (checksum * 31) & 0xFFFFFFFFFFFFFFFF
    hash_bytes = bytearray(32)
    for i in range(4):
        val = (checksum + i * 1000) & 0xFFFFFFFFFFFFFFFF
        hash_bytes[i*8:(i+1)*8] = struct.pack('<Q', val)
    print(f'Model ID: {model_id}, Size: {size_bytes} bytes')
    print(f'Demo Hash: {hash_bytes.hex()}')

def main():
    ap = argparse.ArgumentParser(description='SIS control-plane client (V0 framing)')
    sub = ap.add_subparsers(dest='cmd', required=True)
    ap.add_argument('--wait-ack', action='store_true', help='wait for ACK/ERR from kernel (2s timeout)')
    ap.add_argument('--tcp', metavar='HOST:PORT', help='connect over TCP instead of UNIX socket')
    ap.add_argument('--retries', type=int, default=0, help='retry count on timeout/refused (default 0)')
    ap.add_argument('--token', type=lambda x: int(x, 0), default=0x53535F4354524C21, help='64-bit capability token (default matches kernel dev token)')

    sub.add_parser('create').set_defaults(fn=cmd_create_graph)

    ap_ch = sub.add_parser('add-channel')
    ap_ch.add_argument('capacity')
    ap_ch.set_defaults(fn=cmd_add_channel)

    ap_op = sub.add_parser('add-operator')
    ap_op.add_argument('op_id')
    ap_op.add_argument('--in-ch', type=int)
    ap_op.add_argument('--out-ch', type=int)
    ap_op.add_argument('--priority', type=int, default=10)
    ap_op.add_argument('--stage', choices=['acquire','clean','explore','model','explain'])
    ap_op.add_argument('--in-schema', type=int)
    ap_op.add_argument('--out-schema', type=int)
    ap_op.set_defaults(fn=cmd_add_operator)

    ap_run = sub.add_parser('start')
    ap_run.add_argument('steps')
    ap_run.set_defaults(fn=cmd_start)

    ap_det = sub.add_parser('det')
    ap_det.add_argument('wcet_ns')
    ap_det.add_argument('period_ns')
    ap_det.add_argument('deadline_ns')
    ap_det.set_defaults(fn=cmd_det)

    ap_llm_load = sub.add_parser('llm-load')
    ap_llm_load.add_argument('--wcet-cycles')
    ap_llm_load.set_defaults(fn=cmd_llm_load)

    ap_llm_infer = sub.add_parser('llm-infer')
    ap_llm_infer.add_argument('prompt')
    ap_llm_infer.add_argument('--max-tokens', required=True)
    ap_llm_infer.set_defaults(fn=cmd_llm_infer)

    ap_llm_poll = sub.add_parser('llm-poll')
    ap_llm_poll.add_argument('infer_id')
    ap_llm_poll.set_defaults(fn=cmd_llm_poll)

    ap_llm_cancel = sub.add_parser('llm-cancel')
    ap_llm_cancel.add_argument('infer_id')
    ap_llm_cancel.set_defaults(fn=cmd_llm_cancel)

    ap_llm_hash = sub.add_parser('llm-hash', help='compute demo hash for model package testing')
    ap_llm_hash.add_argument('model_id', help='model ID')
    ap_llm_hash.add_argument('--size', help='buffer size in bytes (default: 1024)')
    ap_llm_hash.set_defaults(fn=cmd_llm_hash)

    args = ap.parse_args()
    args.fn(args)

if __name__ == '__main__':
    main()
