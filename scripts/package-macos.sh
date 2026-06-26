#!/usr/bin/env bash
# ZeroBrowser macOS .app 包打包脚本
#
# 用法：
#   ./scripts/package-macos.sh
#
# 生成 ZeroBrowser.app 到 target/packages/ 目录。
# 必须在 macOS 上运行。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PACKAGE_DIR="$PROJECT_ROOT/target/packages"
APP_NAME="ZeroBrowser"
APP_VERSION="$(cd "$PROJECT_ROOT" && cargo metadata --format-version 1 --no-deps 2>/dev/null | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d["packages"][0]["version"])' 2>/dev/null || echo "0.1.0")"

GREEN='\033[0;32m'
NC='\033[0m'
info() { echo -e "${GREEN}[INFO]${NC} $*"; }

# 检查平台
if [[ "$(uname)" != "Darwin" ]]; then
    echo "错误：此脚本只能在 macOS 上运行"
    exit 1
fi

mkdir -p "$PACKAGE_DIR"

# 编译（browser 与 renderer 须在同一输出目录，供多进程 spawn）
info "编译 release 二进制..."
cd "$PROJECT_ROOT"
cargo build --release -p zero-browser -p zero-renderer

BINARY="$PROJECT_ROOT/target/release/zero-browser"
RENDERER="$PROJECT_ROOT/target/release/zero-renderer"
if [[ ! -f "$BINARY" ]]; then
    echo "错误：编译失败 zero-browser"
    exit 1
fi
if [[ ! -f "$RENDERER" ]]; then
    echo "错误：编译失败 zero-renderer"
    exit 1
fi
strip "$BINARY" 2>/dev/null || true
strip "$RENDERER" 2>/dev/null || true

# 创建 .app 结构
APP_BUNDLE="$PACKAGE_DIR/${APP_NAME}.app"
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"
mkdir -p "$APP_BUNDLE/Contents/Frameworks"

# 复制二进制（多进程：renderer 与 browser 同目录）
cp "$BINARY" "$APP_BUNDLE/Contents/MacOS/ZeroBrowser"
cp "$RENDERER" "$APP_BUNDLE/Contents/MacOS/zero-renderer"

# Info.plist
cat > "$APP_BUNDLE/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>ZeroBrowser</string>
    <key>CFBundleIconFile</key>
    <string>ZeroBrowser</string>
    <key>CFBundleIdentifier</key>
    <string>com.zeroweb.browser</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>ZeroBrowser</string>
    <key>CFBundleDisplayName</key>
    <string>ZeroBrowser</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${APP_VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${APP_VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>HTML Document</string>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>public.html</string>
            </array>
        </dict>
    </array>
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>Web URL</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>http</string>
                <string>https</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
EOF

# PkgInfo
echo -n "APPL????" > "$APP_BUNDLE/Contents/PkgInfo"

info "✅ .app 包已生成: $APP_BUNDLE"

# 检查是否有 create-dmg 来生成 .dmg
if command -v create-dmg &>/dev/null; then
    create-dmg "$PACKAGE_DIR/ZeroBrowser-${APP_VERSION}.dmg" "$APP_BUNDLE" 2>/dev/null || true
    info "✅ .dmg 已生成: $PACKAGE_DIR/ZeroBrowser-${APP_VERSION}.dmg"
else
    info "提示: 安装 create-dmg 可生成 .dmg：brew install create-dmg"
    info "手动创建: hdiutil create -volname ZeroBrowser -srcfolder $APP_BUNDLE -ov -format UDZO ZeroBrowser.dmg"
fi
