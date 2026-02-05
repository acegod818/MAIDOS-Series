#!/bin/bash

# ============================================
#  MAIDOS CodeQC - Unix Runner
#  Usage: ./codeqc.sh [options] [target]
# ============================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLI="$SCRIPT_DIR/maidos-codeqc/dist/cli.js"

# Check if installed
if [ ! -f "$CLI" ]; then
    echo "[ERROR] CodeQC not installed. Run ./install.sh first."
    exit 1
fi

# Run CodeQC
node "$CLI" "$@"
