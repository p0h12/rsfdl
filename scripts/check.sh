#!/usr/bin/env bash
set -euo pipefail

# Workspace-weiter Check: clean, format, test, clippy.
# Logfiles landen in .cargo-test.txt und .cargo-clippy.txt (gitignored).

cd "$(git rev-parse --show-toplevel)"

CRATES=(rsfdl-core rsfdl-cli rsfdl-app)
TEST_LOG=".cargo-test.txt"
CLIPPY_LOG=".cargo-clippy.txt"

RED='\033[0;31m'
GREEN='\033[0;32m'
BOLD='\033[1m'
RESET='\033[0m'

step() { printf "\n${BOLD}==> %s${RESET}\n" "$1"; }
ok()   { printf "${GREEN}    OK${RESET}\n"; }
fail() { printf "${RED}    FAILED${RESET} (see %s)\n" "$1"; }

# --- Clean ---
step "Cleaning ${CRATES[*]}"
cargo clean "${CRATES[@]/#/-p }" 2>/dev/null \
  || cargo clean $(printf -- '-p %s ' "${CRATES[@]}") 2>/dev/null \
  || true
ok

# --- Format ---
step "Formatting (cargo fmt)"
cargo fmt
ok

# --- Test ---
step "Testing (cargo test --workspace)"
if cargo test --workspace > "$TEST_LOG" 2>&1; then
  ok
  # Show summary line
  grep -E '^test result:' "$TEST_LOG" | tail -1
else
  fail "$TEST_LOG"
  grep -E '^test result:|^error' "$TEST_LOG" | head -5
  EXIT=1
fi

# --- Clippy ---
step "Linting (cargo clippy)"
if cargo clippy --workspace --all-targets -- -D warnings > "$CLIPPY_LOG" 2>&1; then
  ok
else
  fail "$CLIPPY_LOG"
  grep -E '^error' "$CLIPPY_LOG" | head -10
  EXIT=1
fi

# --- Summary ---
echo
if [[ "${EXIT:-0}" -eq 0 ]]; then
  printf "${GREEN}${BOLD}All checks passed.${RESET}\n"
else
  printf "${RED}${BOLD}Some checks failed.${RESET} See log files for details.\n"
  exit 1
fi
