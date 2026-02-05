#!/bin/bash

# ============================================
#  MAIDOS CodeQC - Unix Installer
#  One-click dependency installation
# ============================================

set -e

echo ""
echo "  MAIDOS CodeQC Installer"
echo "  ======================="
echo ""

# Check Node.js
if ! command -v node &> /dev/null; then
    echo "[ERROR] Node.js not found!"
    echo ""
    echo "Please install Node.js 18+ from:"
    echo "  https://nodejs.org/"
    echo ""
    exit 1
fi

NODE_VER=$(node -v)
echo "[OK] Node.js $NODE_VER detected"

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Install core package
echo ""
echo "Installing maidos-codeqc..."
cd "$SCRIPT_DIR/maidos-codeqc"
npm install --silent
echo "[OK] maidos-codeqc installed"

# Build if needed
if [ ! -f "dist/cli.js" ]; then
    echo "Building..."
    npm run build --silent
fi

echo ""
echo "============================================"
echo "  Installation Complete!"
echo ""
echo "  Usage:"
echo "    ./codeqc.sh [target]"
echo "    ./codeqc.sh ./src"
echo "    ./codeqc.sh -h"
echo "============================================"
echo ""
