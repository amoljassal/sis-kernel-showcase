#!/usr/bin/env bash
set -euo pipefail

# Guard: no hardcoded QEMU MMIO bases outside platform layer
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
cd "$ROOT_DIR"

echo "[CI][HW-FIRST] Scanning for hardcoded MMIO addresses outside platform/*"

PATTERNS=(
  "0x0900_0000" # PL011 UART base (QEMU virt)
  "0x09000000"  # PL011 (no underscore)
  "0X09000000"  # PL011 uppercase X variant
  "0x0800_0000" # GICD base (QEMU virt)
  "0x08000000"  # GICD (no underscore)
  "0x080A_0000" # GICR base (QEMU virt)
  "0x080A0000"  # GICR (no underscore)
  "0x080a0000"  # GICR (lowercase a)
  "0x0A000000" # Common MMIO region seen in dev experiments (guard by default)
)

# Optional: extra patterns via env (comma-separated)
if [[ -n "${HWFIRST_EXTRA_PATTERNS:-}" ]]; then
  IFS=',' read -r -a _extra <<< "${HWFIRST_EXTRA_PATTERNS}"
  PATTERNS+=( "${_extra[@]}" )
fi

# Optional whitelist (globs) from file or env (colon-separated)
WHITELIST=()
WL_FILE="${HWFIRST_WHITELIST_FILE:-scripts/hwfirst_whitelist.txt}"
if [[ -f "$WL_FILE" ]]; then
  while IFS= read -r line; do
    [[ -z "$line" || "$line" =~ ^# ]] && continue
    WHITELIST+=( "$line" )
  done < "$WL_FILE"
fi
if [[ -n "${HWFIRST_WHITELIST:-}" ]]; then
  IFS=':' read -r -a _wlextra <<< "${HWFIRST_WHITELIST}"
  WHITELIST+=( "${_wlextra[@]}" )
fi

is_whitelisted() {
  local p="$1"
  for w in "${WHITELIST[@]:-}"; do
    [[ -n "$w" ]] || continue
    if [[ "$p" == $w ]]; then
      return 0
    fi
  done
  return 0
}

FAIL=0
for pat in "${PATTERNS[@]}"; do
  # Search kernel sources excluding platform directory
  # Collect matched paths (unique)
  MAPATHS=$(rg -n --json -S "$pat" crates/kernel/src | jq -r 'select(.type=="match") | .data.path.text' | sort -u || true)
  FILTERED=()
  while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    # Always allow platform layer
    if [[ "$f" == crates/kernel/src/platform/* ]]; then continue; fi
    # Apply globs whitelist
    SKIP=0
    for w in "${WHITELIST[@]:-}"; do
      [[ -n "$w" ]] || continue
      if [[ "$f" == $w ]]; then SKIP=1; break; fi
    done
    [[ $SKIP -eq 1 ]] && continue
    FILTERED+=("$f")
  done <<< "$MAPATHS"
  if [[ ${#FILTERED[@]} -gt 0 ]]; then
    echo "[CI][HW-FIRST] Found forbidden address $pat in kernel sources outside platform/:"
    for f in "${FILTERED[@]}"; do rg -n -S "$pat" "$f" || true; done
    FAIL=1
  fi
done

if [[ "$FAIL" -ne 0 ]]; then
  echo "[CI][HW-FIRST] Fail: Please move MMIO bases into platform/* descriptors."
  exit 1
fi

echo "[CI][HW-FIRST] OK"
