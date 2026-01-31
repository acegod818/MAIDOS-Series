#!/bin/bash
# MAIDOS Shared Core - 發布腳本
# 自動從 Cargo.toml 讀取版本號，產出各平台發布包

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

# ============================================================================
# 版本號提取
# ============================================================================
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo "=== MAIDOS Shared Core Release v${VERSION} ==="
echo ""

# 輸出目錄
RELEASE_DIR="$PROJECT_ROOT/release/v${VERSION}"
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

# ============================================================================
# 函數定義
# ============================================================================

build_native() {
    local target=$1
    local os_name=$2
    local arch=$3
    local lib_ext=$4
    
    echo "--- Building for $os_name-$arch ($target) ---"
    
    if [ "$target" = "native" ]; then
        cargo build --release --workspace
        TARGET_DIR="target/release"
    else
        # Cross compile
        if command -v cross &> /dev/null; then
            cross build --release --workspace --target "$target"
            TARGET_DIR="target/$target/release"
        else
            echo "  ⚠️  cross 未安裝，跳過 $os_name-$arch"
            return 1
        fi
    fi
    
    # 打包
    local PKG_NAME="maidos-shared-v${VERSION}-${os_name}-${arch}"
    local PKG_DIR="$RELEASE_DIR/$PKG_NAME"
    mkdir -p "$PKG_DIR/lib" "$PKG_DIR/include"
    
    # 複製動態庫
    for lib in maidos_config maidos_auth maidos_bus maidos_llm; do
        if [ -f "$TARGET_DIR/lib${lib}.${lib_ext}" ]; then
            cp "$TARGET_DIR/lib${lib}.${lib_ext}" "$PKG_DIR/lib/"
        elif [ -f "$TARGET_DIR/${lib}.${lib_ext}" ]; then
            cp "$TARGET_DIR/${lib}.${lib_ext}" "$PKG_DIR/lib/"
        fi
    done
    
    # 複製頭文件
    cp include/maidos.h "$PKG_DIR/include/"
    
    # 複製文檔
    cp README.md CHANGELOG.md LICENSE "$PKG_DIR/" 2>/dev/null || true
    cp docs/USAGE.md "$PKG_DIR/" 2>/dev/null || true
    
    # 壓縮
    cd "$RELEASE_DIR"
    zip -r "${PKG_NAME}.zip" "$PKG_NAME"
    rm -rf "$PKG_NAME"
    cd "$PROJECT_ROOT"
    
    echo "  ✅ $PKG_NAME.zip"
    return 0
}

build_source() {
    echo "--- Building source package ---"
    
    local PKG_NAME="maidos-shared-v${VERSION}-source"
    local PKG_DIR="$RELEASE_DIR/$PKG_NAME"
    
    mkdir -p "$PKG_DIR"
    
    # 複製源碼（排除 target、.git）
    rsync -a --exclude='target' --exclude='.git' --exclude='release' \
        "$PROJECT_ROOT/" "$PKG_DIR/"
    
    # 壓縮
    cd "$RELEASE_DIR"
    zip -r "${PKG_NAME}.zip" "$PKG_NAME"
    rm -rf "$PKG_NAME"
    cd "$PROJECT_ROOT"
    
    echo "  ✅ $PKG_NAME.zip"
}

build_nuget() {
    echo "--- Building NuGet package ---"
    
    if ! command -v dotnet &> /dev/null; then
        echo "  ⚠️  dotnet 未安裝，跳過 NuGet"
        return 1
    fi
    
    local NUGET_DIR="$RELEASE_DIR/nuget"
    mkdir -p "$NUGET_DIR"
    
    # 更新 .csproj 版本
    sed -i "s/<Version>.*<\/Version>/<Version>${VERSION}<\/Version>/" \
        bindings/csharp/MaidosShared/MaidosShared.csproj 2>/dev/null || true
    
    # 建置 NuGet
    cd bindings/csharp/MaidosShared
    dotnet pack -c Release -o "$NUGET_DIR"
    cd "$PROJECT_ROOT"
    
    # 移動到 release 目錄
    mv "$NUGET_DIR"/*.nupkg "$RELEASE_DIR/" 2>/dev/null || true
    rm -rf "$NUGET_DIR"
    
    echo "  ✅ MaidosShared.${VERSION}.nupkg"
}

build_full() {
    echo "--- Building full package ---"
    
    local PKG_NAME="maidos-shared-v${VERSION}-full"
    local PKG_DIR="$RELEASE_DIR/$PKG_NAME"
    
    mkdir -p "$PKG_DIR"
    
    # 複製所有已產出的 zip
    cp "$RELEASE_DIR"/*.zip "$PKG_DIR/" 2>/dev/null || true
    cp "$RELEASE_DIR"/*.nupkg "$PKG_DIR/" 2>/dev/null || true
    
    # 壓縮
    cd "$RELEASE_DIR"
    zip -r "${PKG_NAME}.zip" "$PKG_NAME"
    rm -rf "$PKG_NAME"
    cd "$PROJECT_ROOT"
    
    echo "  ✅ $PKG_NAME.zip"
}

# ============================================================================
# 主流程
# ============================================================================

echo "輸出目錄: $RELEASE_DIR"
echo ""

# 1. 源碼包
build_source

# 2. Linux x64 (native)
build_native "native" "linux" "x64" "so"

# 3. Windows x64 (cross)
build_native "x86_64-pc-windows-gnu" "windows" "x64" "dll" || true

# 4. macOS ARM64 (cross)
build_native "aarch64-apple-darwin" "macos" "arm64" "dylib" || true

# 5. macOS x64 (cross)
build_native "x86_64-apple-darwin" "macos" "x64" "dylib" || true

# 6. NuGet
build_nuget || true

# 7. Full package
build_full

# ============================================================================
# 產出摘要
# ============================================================================

echo ""
echo "=== Release v${VERSION} 完成 ==="
echo ""
ls -lh "$RELEASE_DIR"/*.zip "$RELEASE_DIR"/*.nupkg 2>/dev/null || true
echo ""
echo "SHA256:"
cd "$RELEASE_DIR"
sha256sum *.zip *.nupkg 2>/dev/null || true
