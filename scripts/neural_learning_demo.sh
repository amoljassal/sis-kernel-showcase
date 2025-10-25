#!/bin/bash
# Neural Learning Demo Script
# Demonstrates the kernel's ability to learn from user feedback

cat <<'EOF'
=== SIS Kernel: Neural Learning Demo ===

This demo shows how the kernel learns from user feedback:

1. Execute commands - kernel predicts success/fail before execution
2. Provide feedback - tell the kernel if predictions were helpful
3. Retrain network - apply feedback to improve future predictions
4. Verify learning - see improved predictions

Commands to try in the shell:
----------------------------------------

# Step 1: Execute some commands (kernel will predict outcomes)
help
neuralctl status
graphctl list
invalidcommand
echo Hello

# Step 2: Provide feedback on last prediction
neuralctl feedback helpful      # If prediction was accurate
neuralctl feedback not_helpful  # If prediction was wrong
neuralctl feedback expected     # If outcome was as expected

# Step 3: Retrain the network from feedback
neuralctl retrain 10            # Apply up to 10 feedback examples

# Step 4: Execute more commands to see improved predictions
help
graphctl list
invalidcommand

# View neural network status
neuralctl status

# View audit log of predictions
neuralctl audit

----------------------------------------
EOF

# Launch kernel
echo "Starting kernel..."
echo ""
