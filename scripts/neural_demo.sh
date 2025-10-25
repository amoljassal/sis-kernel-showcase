#!/usr/bin/env bash
set -euo pipefail

# Neural learning demo
# Demonstrates feedback-driven online learning in the kernel.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cat <<'MSG'
== SIS Kernel Neural Learning Demo ==

This demo shows online learning via command prediction and feedback.

Steps:
  1) Boot the kernel and wait for the sis> prompt.
  2) Execute commands - observe predictions before execution.
  3) Provide feedback via: neuralctl feedback <helpful|not_helpful|expected>
  4) Retrain network via: neuralctl retrain 10
  5) Observe improved predictions on subsequent commands.

Training sequence (copy/paste into shell):

  # Initial predictions (untrained)
  help
  invalidcmd
  neuralctl status

  # Provide feedback on valid commands
  help
  neuralctl feedback helpful
  info
  neuralctl feedback helpful
  mem
  neuralctl feedback helpful

  # Provide feedback on invalid commands
  badcmd
  neuralctl feedback helpful
  notvalid
  neuralctl feedback helpful
  xyz
  neuralctl feedback helpful

  # First retraining
  neuralctl retrain 10
  neuralctl status

  # Test predictions (should be better)
  help
  invalidtest
  graphctl list

  # More training
  graphctl list
  neuralctl feedback helpful
  neuralctl status
  neuralctl feedback helpful
  foobar
  neuralctl feedback helpful

  # Second retraining
  neuralctl retrain 10

  # Final validation (high accuracy expected)
  help
  info
  badcommand
  neuralctl status

Expected progression:
  - Initial: random predictions, low confidence
  - After 6 examples: moderate improvement
  - After 12+ examples: high accuracy, confident predictions

Metrics:
  nn_infer_us       - inference latency
  nn_infer_count    - total predictions
  nn_teach_count    - training iterations
  nn_retrain_steps  - examples applied

Launching kernel now...
MSG

SIS_FEATURES="llm,crypto-real" BRINGUP=1 "$SCRIPT_DIR/uefi_run.sh"
