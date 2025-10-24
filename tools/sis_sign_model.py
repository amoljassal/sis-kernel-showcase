#!/usr/bin/env python3
"""
Ed25519 signing helper for SIS model packages.

Generates the deterministic model buffer used by the kernel (byte = model_id & 0xFF,
repeated size_bytes), computes SHA-256, and signs the hash using an Ed25519 private key.

Outputs:
- Public key (hex)
- SHA-256 hash (hex)
- Signature (hex)

Prereqs: pip install pynacl

Examples:
  python3 tools/sis_sign_model.py --model-id 7 --size 1024 --privkey <64-hex>
  # Then:
  #   export SIS_ED25519_PUBKEY=0x<pubkey-hex>
  #   SIS_FEATURES="llm,crypto-real" BRINGUP=1 ./scripts/uefi_run.sh
  #   In shell: llmctl load --model 7 --hash 0x<HASH_HEX> --sig 0x<SIG_HEX> --size-bytes 1024
"""

import argparse
import binascii
import hashlib
import sys

try:
    from nacl.signing import SigningKey
    from nacl.encoding import HexEncoder
except Exception as e:
    sys.stderr.write("PyNaCl not available: pip install pynacl\n")
    raise


def make_buffer(model_id: int, size_bytes: int) -> bytes:
    b = model_id & 0xFF
    return bytes([b]) * size_bytes


def sha256_bytes(data: bytes) -> bytes:
    h = hashlib.sha256()
    h.update(data)
    return h.digest()


def parse_args():
    ap = argparse.ArgumentParser(description="Sign SIS model package buffer (Ed25519 over SHA-256)")
    ap.add_argument("--model-id", required=True, type=int, help="model id (0..4294967295)")
    ap.add_argument("--size", required=True, type=int, help="buffer size in bytes")
    ap.add_argument("--privkey", required=True, help="Ed25519 private key (64 hex chars)")
    return ap.parse_args()


def main():
    args = parse_args()
    if args.size <= 0:
        raise SystemExit("size must be positive")

    # Load signing key
    try:
        sk = SigningKey(args.privkey, encoder=HexEncoder)
    except Exception:
        raise SystemExit("invalid --privkey (expect 64 hex chars)")
    vk = sk.verify_key

    # Build deterministic buffer and compute SHA-256
    data = make_buffer(args.model_id, args.size)
    digest = sha256_bytes(data)

    # Sign the hash bytes
    signed = sk.sign(digest)
    sig = signed.signature

    # Print artifacts
    print(f"Public Key: {vk.encode(encoder=HexEncoder).decode().lower()}")
    print(f"SHA-256: {digest.hex()}")
    print(f"Signature: {sig.hex()}")


if __name__ == "__main__":
    main()

