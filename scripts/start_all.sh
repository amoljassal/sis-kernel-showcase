#!/usr/bin/env bash
set -euo pipefail

# Single-command startup script for SIS Kernel + GUI
# Usage: ./scripts/start_all.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."
cd "$ROOT_DIR"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Cleanup function
cleanup() {
  echo -e "\n${YELLOW}[*] Shutting down...${NC}"

  # Kill GUI dev server
  if [[ -n "${GUI_PID:-}" ]]; then
    echo "[*] Stopping GUI dev server (PID $GUI_PID)..."
    kill $GUI_PID 2>/dev/null || true
  fi

  # Stop QEMU via API
  echo "[*] Stopping QEMU kernel..."
  curl -s -X POST http://localhost:8871/api/v1/qemu/stop >/dev/null 2>&1 || true

  # Kill sisctl daemon
  if [[ -n "${DAEMON_PID:-}" ]]; then
    echo "[*] Stopping sisctl daemon (PID $DAEMON_PID)..."
    kill $DAEMON_PID 2>/dev/null || true
  fi

  echo -e "${GREEN}[+] Cleanup complete${NC}"
}

trap cleanup EXIT INT TERM

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   SIS Kernel - Full Stack Startup     ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Step 1: Build sisctl daemon if needed
echo -e "${YELLOW}[1/5] Building sisctl daemon...${NC}"
if [[ ! -f "target/release/sisctl" ]]; then
  cargo build --release -p sisctl
else
  echo "      Using existing sisctl binary"
fi
echo -e "${GREEN}[+] Daemon binary ready${NC}"
echo ""

# Step 2: Start sisctl daemon
echo -e "${YELLOW}[2/5] Starting sisctl daemon...${NC}"
export SIS_RUN_SCRIPT="$ROOT_DIR/scripts/uefi_run.sh"
export RUST_LOG="${RUST_LOG:-info}"
./target/release/sisctl > /tmp/sisctl.log 2>&1 &
DAEMON_PID=$!
echo "      Daemon PID: $DAEMON_PID"
echo "      Logs: /tmp/sisctl.log"

# Wait for daemon to be ready
echo "      Waiting for daemon to start..."
for i in {1..30}; do
  if curl -s http://localhost:8871/health >/dev/null 2>&1; then
    echo -e "${GREEN}[+] Daemon is ready (port 8871)${NC}"
    break
  fi
  sleep 1
  if [[ $i -eq 30 ]]; then
    echo -e "${RED}[!] Daemon failed to start after 30s${NC}"
    echo "Check logs: tail -f /tmp/sisctl.log"
    exit 1
  fi
done
echo ""

# Step 3: Start QEMU kernel via API
echo -e "${YELLOW}[3/5] Starting QEMU kernel...${NC}"
curl -s -X POST http://localhost:8871/api/v1/qemu/run \
  -H "Content-Type: application/json" \
  -d '{"features": ["llm", "ai-ops", "crypto-real"], "env": {"BRINGUP": "1"}}' \
  >/dev/null

echo "      Kernel starting with features: llm, ai-ops, crypto-real"
echo "      Waiting for kernel boot..."
sleep 3

# Check status
STATUS=$(curl -s http://localhost:8871/api/v1/qemu/status)
STATE=$(echo "$STATUS" | grep -o '"state":"[^"]*"' | cut -d'"' -f4)
echo "      Kernel state: $STATE"
echo -e "${GREEN}[+] Kernel started successfully${NC}"
echo ""

# Step 4: Build GUI if needed
echo -e "${YELLOW}[4/5] Preparing GUI frontend...${NC}"
cd gui/desktop
if [[ ! -d "node_modules" ]]; then
  echo "      Installing dependencies..."
  pnpm install
else
  echo "      Dependencies already installed"
fi
echo -e "${GREEN}[+] GUI ready${NC}"
echo ""

# Step 5: Start GUI dev server
echo -e "${YELLOW}[5/5] Starting GUI dev server...${NC}"
pnpm dev > /tmp/gui.log 2>&1 &
GUI_PID=$!
echo "      GUI PID: $GUI_PID"
echo "      Logs: /tmp/gui.log"

# Wait for GUI to be ready (check both common ports)
echo "      Waiting for GUI to start..."
GUI_PORT=""
for i in {1..20}; do
  if curl -s http://localhost:1420 >/dev/null 2>&1; then
    GUI_PORT="1420"
    echo -e "${GREEN}[+] GUI is ready (port 1420)${NC}"
    break
  elif curl -s http://localhost:5173 >/dev/null 2>&1; then
    GUI_PORT="5173"
    echo -e "${GREEN}[+] GUI is ready (port 5173)${NC}"
    break
  fi
  sleep 1
  if [[ $i -eq 20 ]]; then
    echo -e "${RED}[!] GUI failed to start${NC}"
    echo "Check logs: tail -f /tmp/gui.log"
    exit 1
  fi
done
echo ""

echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}   [+] Full Stack Running!${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "Services:"
echo -e "  ${GREEN}[*]${NC} sisctl daemon:  http://localhost:8871"
echo -e "  ${GREEN}[*]${NC} WebSocket:      ws://localhost:8871/events"
echo -e "  ${GREEN}[*]${NC} QEMU kernel:    State=$STATE"
echo -e "  ${GREEN}[*]${NC} GUI:            http://localhost:${GUI_PORT}"
echo ""
echo -e "Logs:"
echo -e "  ${BLUE}[*]${NC} Daemon: tail -f /tmp/sisctl.log"
echo -e "  ${BLUE}[*]${NC} GUI:    tail -f /tmp/gui.log"
echo ""
echo -e "${YELLOW}Opening GUI in browser...${NC}"

# Open browser (macOS)
if command -v open >/dev/null 2>&1; then
  sleep 2
  open http://localhost:${GUI_PORT}
fi

echo ""
echo -e "${YELLOW}Press Ctrl+C to stop all services${NC}"
echo ""

# Keep script running
wait $GUI_PID
