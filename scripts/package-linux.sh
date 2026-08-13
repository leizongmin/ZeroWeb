#!/usr/bin/env bash
# ZeroBrowser Linux 打包脚本
#
# 用法：
#   ./scripts/package-linux.sh [--appimage|--deb|--all]
#
# 生成 .AppImage 和/或 .deb 安装包到 target/packages/ 目录。
# 需要：cargo, strip, dpkg-deb (仅 deb)
#
# 此脚本在无头环境中运行，不需要 GPU 或显示服务器。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PACKAGE_DIR="$PROJECT_ROOT/target/packages"
APP_NAME="zero-browser"
APP_DISPLAY="ZeroBrowser"
if [[ -n "${ZERO_BUILD_VERSION:-}" ]]; then
    APP_VERSION="$ZERO_BUILD_VERSION"
elif [[ -n "${SOURCE_DATE_EPOCH:-}" ]]; then
    read -r version_year version_month version_day < <(
        date -u -d "@$SOURCE_DATE_EPOCH" "+%y %m %d"
    )
    APP_VERSION="$((10#$version_year)).$((10#$version_month)).$((10#$version_day))"
else
    read -r version_year version_month version_day < <(date "+%y %m %d")
    APP_VERSION="$((10#$version_year)).$((10#$version_month)).$((10#$version_day))"
fi
[[ "$APP_VERSION" =~ ^[0-9]{1,2}\.[0-9]{1,2}\.[0-9]{1,2}$ ]] \
    || { echo "[ERROR] version must use YY.M.D: $APP_VERSION" >&2; exit 1; }
IFS=. read -r version_year version_month version_day <<< "$APP_VERSION"
(( 10#$version_year <= 99 && 10#$version_month >= 1 && 10#$version_month <= 12 \
    && 10#$version_day >= 1 && 10#$version_day <= 31 )) \
    || { echo "[ERROR] version contains an invalid date: $APP_VERSION" >&2; exit 1; }
export ZERO_BUILD_VERSION="$APP_VERSION"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }

# ── 参数解析 ──
BUILD_APPIMAGE=false
BUILD_DEB=false

if [[ $# -eq 0 ]]; then
    BUILD_APPIMAGE=true
    BUILD_DEB=true
else
    for arg in "$@"; do
        case "$arg" in
            --appimage) BUILD_APPIMAGE=true ;;
            --deb)      BUILD_DEB=true ;;
            --all)      BUILD_APPIMAGE=true; BUILD_DEB=true ;;
            -h|--help)
                echo "用法: $0 [--appimage|--deb|--all]"
                echo "  --appimage  仅构建 .AppImage"
                echo "  --deb       仅构建 .deb"
                echo "  --all       构建所有（默认）"
                exit 0
                ;;
            *) error "未知参数: $arg" ;;
        esac
    done
fi

mkdir -p "$PACKAGE_DIR"

# ── 编译 release 二进制 ──
build_binary() {
    info "编译 release 二进制 zero-browser、zero-renderer 与 zero-compositor..."
    cd "$PROJECT_ROOT"
    cargo build --release -p zero-browser -p zero-renderer -p zero-compositor

    local binary="$PROJECT_ROOT/target/release/zero-browser"
    local renderer="$PROJECT_ROOT/target/release/zero-renderer"
    local compositor="$PROJECT_ROOT/target/release/zero-compositor"
    if [[ ! -f "$binary" ]]; then
        error "编译失败：未找到 $binary"
    fi
    if [[ ! -f "$renderer" ]]; then
        error "编译失败：未找到 $renderer"
    fi
    if [[ ! -f "$compositor" ]]; then
        error "编译失败：未找到 $compositor"
    fi

    # strip 减小体积
    strip --strip-unneeded "$binary" 2>/dev/null || warn "strip zero-browser 失败（可忽略）"
    strip --strip-unneeded "$renderer" 2>/dev/null || warn "strip zero-renderer 失败（可忽略）"
    strip --strip-unneeded "$compositor" 2>/dev/null || warn "strip zero-compositor 失败（可忽略）"
    local size
    size=$(du -h "$binary" | cut -f1)
    info "zero-browser 大小: $size"
    size=$(du -h "$renderer" | cut -f1)
    info "zero-renderer 大小: $size"
}

# ── 复制应用图标到目标 hicolor 目录 ──
# 用法：copy_app_icons <dest_root>  （dest_root 是包含 usr/share/icons 的根）
copy_app_icons() {
    local dest="$1"
    local icon_gen="$PROJECT_ROOT/apps/browser/assets/icons-gen"
    local src_svg="$PROJECT_ROOT/apps/browser/assets/app-icon.svg"

    mkdir -p "$dest/usr/share/icons/hicolor/scalable/apps"
    cp "$src_svg" "$dest/usr/share/icons/hicolor/scalable/apps/zero-browser.svg"

    # 已生成的 PNG 各尺寸
    if [[ -d "$icon_gen" ]]; then
        local size
        for size in 16 32 48 128 256 512; do
            local png="$icon_gen/icon-${size}.png"
            if [[ -f "$png" ]]; then
                mkdir -p "$dest/usr/share/icons/hicolor/${size}x${size}/apps"
                cp "$png" "$dest/usr/share/icons/hicolor/${size}x${size}/apps/zero-browser.png"
            fi
        done
    else
        warn "未找到 icons-gen，请在打包前运行：cargo run -p zero-icon-gen"
    fi
}

# ── 构建 .AppImage ──
build_appimage() {
    info "构建 .AppImage 包..."

    local appdir="$PACKAGE_DIR/ZeroBrowser.AppDir"
    rm -rf "$appdir"
    mkdir -p "$appdir/usr/bin"
    mkdir -p "$appdir/usr/share/applications"
    mkdir -p "$appdir/usr/share/icons/hicolor/256x256/apps"
    mkdir -p "$appdir/usr/share/icons/hicolor/scalable/apps"

    # 复制二进制（多进程：renderer 与 browser 同目录）
    cp "$PROJECT_ROOT/target/release/zero-browser" "$appdir/usr/bin/"
    cp "$PROJECT_ROOT/target/release/zero-renderer" "$appdir/usr/bin/"
    cp "$PROJECT_ROOT/target/release/zero-compositor" "$appdir/usr/bin/"

    # 创建 .desktop 文件
    cat > "$appdir/zero-browser.desktop" << 'EOF'
[Desktop Entry]
Name=ZeroBrowser
Comment=A cross-platform browser built with Rust
Exec=zero-browser %u
Icon=zero-browser
Type=Application
Categories=Network;WebBrowser;
MimeType=text/html;x-scheme-handler/http;x-scheme-handler/https;
Keywords=browser;web;internet;
StartupNotify=true
EOF
    cp "$appdir/zero-browser.desktop" "$appdir/usr/share/applications/"

    # 创建 AppImage 元数据
    cat > "$appdir/AppRun" << 'RUNEOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
exec "${HERE}/usr/bin/zero-browser" "$@"
RUNEOF
    chmod +x "$appdir/AppRun"

    # 复制真实应用图标（SVG + PNG 各尺寸）
    copy_app_icons "$appdir"
    cp "$appdir/usr/share/icons/hicolor/scalable/apps/zero-browser.svg" "$appdir/zero-browser.svg"

    # 创建 .DirIcon（256x256 PNG，AppImage 需要）
    local dir_icon_src="$appdir/usr/share/icons/hicolor/256x256/apps/zero-browser.png"
    if [[ -f "$dir_icon_src" ]]; then
        cp "$dir_icon_src" "$appdir/.DirIcon"
    else
        warn "缺少 256px PNG，.DirIcon 将缺失（运行 cargo run -p zero-icon-gen 生成）"
    fi

    info "AppDir 已创建: $appdir"
    info "可使用 appimagetool 手动打包为 .AppImage："
    info "  appimagetool $appdir ${PACKAGE_DIR}/ZeroBrowser-${APP_VERSION}-x86_64.AppImage"

    # 如果有 appimagetool 则自动打包
    if command -v appimagetool &>/dev/null; then
        appimagetool "$appdir" "$PACKAGE_DIR/ZeroBrowser-${APP_VERSION}-x86_64.AppImage"
        info "✅ .AppImage 已生成: $PACKAGE_DIR/ZeroBrowser-${APP_VERSION}-x86_64.AppImage"
    else
        warn "appimagetool 未安装，跳过 .AppImage 打包"
        warn "安装: wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage -O /usr/local/bin/appimagetool && chmod +x /usr/local/bin/appimagetool"
        info "AppDir 已就绪，可手动打包"
    fi
}

# ── 构建 .deb 包 ──
build_deb() {
    info "构建 .deb 包..."

    if ! command -v dpkg-deb &>/dev/null; then
        warn "dpkg-deb 未安装，跳过 .deb 打包"
        warn "安装: sudo apt install dpkg-dev"
        return 0
    fi

    local debroot="$PACKAGE_DIR/zero-browser_${APP_VERSION}_amd64"
    rm -rf "$debroot"
    mkdir -p "$debroot/DEBIAN"
    mkdir -p "$debroot/usr/bin"
    mkdir -p "$debroot/usr/share/applications"
    mkdir -p "$debroot/usr/share/icons/hicolor/256x256/apps"
    mkdir -p "$debroot/usr/share/icons/hicolor/scalable/apps"
    mkdir -p "$debroot/usr/share/doc/zero-browser"

    # 复制二进制（多进程：renderer 与 browser 同目录）
    cp "$PROJECT_ROOT/target/release/zero-browser" "$debroot/usr/bin/"
    cp "$PROJECT_ROOT/target/release/zero-renderer" "$debroot/usr/bin/"
    cp "$PROJECT_ROOT/target/release/zero-compositor" "$debroot/usr/bin/"
    chmod 755 "$debroot/usr/bin/zero-browser"
    chmod 755 "$debroot/usr/bin/zero-renderer"
    chmod 755 "$debroot/usr/bin/zero-compositor"

    # control 文件
    local installed_size
    installed_size=$(du -sk "$debroot/usr" | cut -f1)
    cat > "$debroot/DEBIAN/control" << EOF
Package: zero-browser
Version: ${APP_VERSION}
Section: web
Priority: optional
Architecture: amd64
Installed-Size: ${installed_size}
Maintainer: ZeroWeb Team <zeroweb@example.com>
Description: ZeroBrowser - A cross-platform browser built with Rust
 ZeroBrowser is an experimental web browser built from scratch in Rust.
 It features a custom rendering engine, CSS parser, layout engine,
 and V8 JavaScript integration.
Homepage: https://github.com/leizongmin/ZeroWeb
Depends: libc6 (>= 2.31), libgcc-s1 (>= 4.2)
EOF

    # .desktop 文件
    cat > "$debroot/usr/share/applications/zero-browser.desktop" << 'EOF'
[Desktop Entry]
Name=ZeroBrowser
Comment=A cross-platform browser built with Rust
Exec=/usr/bin/zero-browser %u
Icon=zero-browser
Type=Application
Categories=Network;WebBrowser;
MimeType=text/html;x-scheme-handler/http;x-scheme-handler/https;
Keywords=browser;web;internet;
StartupNotify=true
EOF

    # 图标（真实 SVG + PNG 各尺寸）
    copy_app_icons "$debroot"

    # copyright 文件
    cat > "$debroot/usr/share/doc/zero-browser/copyright" << 'EOF'
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: ZeroBrowser
Source: https://github.com/leizongmin/ZeroWeb

Files: *
Copyright: 2026 ZeroWeb Contributors
License: MIT
 Permission is hereby granted, free of charge, to any person obtaining a copy
 of this software and associated documentation files (the "Software"), to deal
 in the Software without restriction, including without limitation the rights
 to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 copies of the Software, and to permit persons to whom the Software is
 furnished to do so, subject to the following conditions:
 .
 The above copyright notice and this permission notice shall be included in all
 copies or substantial portions of the Software.
 .
 THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 SOFTWARE.
EOF

    # changelog
    cat > "$debroot/usr/share/doc/zero-browser/changelog.Debian" << EOF
zero-browser (${APP_VERSION}) unstable; urgency=low

  * Initial package build.

 -- ZeroWeb Team <zeroweb@example.com>  $(date -R)

EOF

    # 构建 .deb
    dpkg-deb --build "$debroot" "${debroot}.deb"
    rm -rf "$debroot"

    info "✅ .deb 已生成: ${debroot}.deb"
    local deb_size
    deb_size=$(du -h "${debroot}.deb" | cut -f1)
    info "   大小: $deb_size"
}

# ── 主流程 ──
cd "$PROJECT_ROOT"
info "ZeroBrowser v${APP_VERSION} Linux 打包"
info "================================"

build_binary

if [[ "$BUILD_APPIMAGE" == true ]]; then
    build_appimage
fi

if [[ "$BUILD_DEB" == true ]]; then
    build_deb
fi

info ""
info "================================"
info "打包完成！产物在 $PACKAGE_DIR/"
ls -lh "$PACKAGE_DIR/" 2>/dev/null | grep -v "^total\|^d" || true
