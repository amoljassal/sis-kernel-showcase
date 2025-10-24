#!/usr/bin/env bash
set -euo pipefail

# LLM host-control audit demo
# Prints step-by-step instructions to demonstrate reject (bad token) and accept (good token),
# then launches QEMU with VirtIO console so you can run the host tool in another terminal.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cat <<'MSG'
== SIS Kernel LLM Host Audit Demo ==

This demo shows host-driven LLM control with both reject and accept paths.

Steps:
  1) Watch for "VCON: READY" in the serial output below.
  2) In another terminal (same repo root), send a REJECT example:
       ./tools/sis_datactl.py --retries 2 --wait-ack --token 0x1 llm-load --wcet-cycles 25000
       ./tools/sis_datactl.py --retries 2 --wait-ack --token 0x1 llm-infer "why op b slow?" --max-tokens 4

  3) Send an ACCEPT example (uses default dev token, which has admin rights):
       ./tools/sis_datactl.py --retries 4 --wait-ack llm-load --wcet-cycles 25000
       ./tools/sis_datactl.py --retries 4 --wait-ack llm-infer "why op b slow?" --max-tokens 4

  4) In the SIS shell (this window), print the audit log in JSON:
       llmjson

Notes:
  - You can also rotate explicit tokens in the shell via `ctladmin` and `ctlsubmit`.
  - Embedded-rights tokens are also supported: upper 8 bits = rights (bit0=ADMIN, bit1=SUBMIT), lower 56 bits secret.

Launching QEMU with VirtIO console now...
MSG

VIRTIO=1 SIS_FEATURES="virtio-console,llm" BRINGUP=1 "$SCRIPT_DIR/uefi_run.sh"

