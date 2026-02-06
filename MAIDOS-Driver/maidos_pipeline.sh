#!/bin/bash
# ============================================
# MAIDOS Pipeline - Linux/WSL 版本
# Code-QC v2.5 強制執行腳本
# ============================================

set -e  # 任何錯誤立即退出

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "============================================"
echo "MAIDOS Pipeline v2.5 - Linux/WSL"
echo "============================================"
echo ""

# 建立 evidence 目錄
mkdir -p evidence

# [1/6] 編譯
echo "=== [1/6] 編譯 ==="
if ! cargo build --release 2>&1 | tee evidence/build.log; then
    echo -e "${RED}[FAIL] 編譯失敗${NC}"
    exit 1
fi
if grep -qi "error\|warning" evidence/build.log; then
    echo -e "${RED}[FAIL] 編譯有 error 或 warning${NC}"
    grep -i "error\|warning" evidence/build.log
    exit 1
fi
echo -e "${GREEN}[PASS] 編譯乾淨${NC}"

# [2/6] 測試
echo "=== [2/6] 測試 ==="
if ! cargo test 2>&1 | tee evidence/test.log; then
    echo -e "${RED}[FAIL] 測試失敗${NC}"
    exit 1
fi
if grep -q "FAILED" evidence/test.log; then
    echo -e "${RED}[FAIL] 有測試失敗${NC}"
    grep "FAILED" evidence/test.log
    exit 1
fi
echo -e "${GREEN}[PASS] 測試通過${NC}"

# [3/6] Lint
echo "=== [3/6] Lint ==="
if ! cargo clippy -- -D warnings 2>&1 | tee evidence/lint.log; then
    echo -e "${RED}[FAIL] Lint 失敗${NC}"
    exit 1
fi
echo -e "${GREEN}[PASS] Lint 乾淨${NC}"

# [4/6] 紅線檢查
echo "=== [4/6] 紅線檢查 ==="
{
    grep -rn "todo!\|unimplemented!\|TODO\|FIXME" src/ 2>/dev/null || true
    grep -rn "password\s*=\|\.unwrap()" src/ 2>/dev/null || true
    grep -rn "return true\|return false" src/ 2>/dev/null | grep -v test || true
} > evidence/redline.log

lines=$(wc -l < evidence/redline.log)
if [ "$lines" -gt 0 ]; then
    echo -e "${RED}[FAIL] 紅線違規 $lines 項${NC}"
    cat evidence/redline.log
    exit 1
fi
echo -e "${GREEN}[PASS] 紅線乾淨${NC}"

# [5/6] 安全掃描
echo "=== [5/6] 安全掃描 ==="
cargo audit 2>&1 | tee evidence/audit.log || true
if grep -qi "critical\|high" evidence/audit.log; then
    echo -e "${YELLOW}[WARN] 發現安全問題，請檢查 evidence/audit.log${NC}"
fi
echo -e "${GREEN}[PASS] 安全掃描完成${NC}"

# [6/6] 打包
echo "=== [6/6] 打包 ==="
cargo build --release 2>&1 | tee evidence/package.log
echo -e "${GREEN}[PASS] 打包完成${NC}"

echo ""
echo "============================================"
echo -e "${GREEN}[SUCCESS] Pipeline PASS - 產物已就緒${NC}"
echo "============================================"
echo ""
echo "證據文件:"
ls -la evidence/

exit 0
