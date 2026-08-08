#!/usr/bin/env bash
# Build a distributable ZeroBrowser.app and zip archive on macOS.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

APP_NAME="ZeroBrowser"
BUNDLE_ID="com.zeroweb.browser"
HELPER_NAME="ZeroBrowser Helper (Renderer)"
OUTPUT_DIR="$PROJECT_ROOT/target/packages"
BROWSER_BINARY=""
RENDERER_BINARY=""
APP_VERSION=""
ARCHIVE_PATH=""
SIGN_IDENTITY="${MACOS_SIGN_IDENTITY:-}"
NOTARIZE=false

usage() {
    cat <<'EOF'
Usage:
  scripts/package-macos.sh [options]

Options:
  --browser PATH         Use an existing zero-browser binary.
  --renderer PATH        Use an existing zero-renderer binary.
  --output-dir PATH      Output directory (default: target/packages).
  --version VERSION      Bundle version (default: Cargo workspace version).
  --archive PATH         Zip path (default: <output-dir>/zero-browser-macos.zip).
  --sign-identity NAME   Developer ID Application identity. Defaults to
                         MACOS_SIGN_IDENTITY. Without it, the app is ad-hoc signed.
  --notarize             Submit with notarytool and staple the accepted ticket.
                         Requires APPLE_ID, APPLE_TEAM_ID, and APPLE_APP_PASSWORD.
  -h, --help             Show this help.

When --browser and --renderer are omitted, the script builds both release binaries.
The zip contains ZeroBrowser.app at its root.
EOF
}

fail() {
    echo "[ERROR] $*" >&2
    exit 1
}

info() {
    echo "[INFO] $*"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --browser)
            [[ $# -ge 2 ]] || fail "--browser requires a path"
            BROWSER_BINARY="$2"
            shift 2
            ;;
        --renderer)
            [[ $# -ge 2 ]] || fail "--renderer requires a path"
            RENDERER_BINARY="$2"
            shift 2
            ;;
        --output-dir)
            [[ $# -ge 2 ]] || fail "--output-dir requires a path"
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --version)
            [[ $# -ge 2 ]] || fail "--version requires a value"
            APP_VERSION="$2"
            shift 2
            ;;
        --archive)
            [[ $# -ge 2 ]] || fail "--archive requires a path"
            ARCHIVE_PATH="$2"
            shift 2
            ;;
        --sign-identity)
            [[ $# -ge 2 ]] || fail "--sign-identity requires a value"
            SIGN_IDENTITY="$2"
            shift 2
            ;;
        --notarize)
            NOTARIZE=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            fail "unknown argument: $1"
            ;;
    esac
done

[[ "$(uname -s)" == "Darwin" ]] || fail "this script must run on macOS"
command -v codesign >/dev/null || fail "codesign is required"
command -v ditto >/dev/null || fail "ditto is required"
command -v iconutil >/dev/null || fail "iconutil is required"
command -v sips >/dev/null || fail "sips is required"

if [[ -n "$BROWSER_BINARY" || -n "$RENDERER_BINARY" ]]; then
    [[ -n "$BROWSER_BINARY" && -n "$RENDERER_BINARY" ]] \
        || fail "--browser and --renderer must be provided together"
else
    info "Building release binaries"
    (
        cd "$PROJECT_ROOT"
        cargo build --release -p zero-browser -p zero-renderer
    )
    BROWSER_BINARY="$PROJECT_ROOT/target/release/zero-browser"
    RENDERER_BINARY="$PROJECT_ROOT/target/release/zero-renderer"
fi

[[ "$BROWSER_BINARY" = /* ]] || BROWSER_BINARY="$PROJECT_ROOT/$BROWSER_BINARY"
[[ "$RENDERER_BINARY" = /* ]] || RENDERER_BINARY="$PROJECT_ROOT/$RENDERER_BINARY"
[[ "$OUTPUT_DIR" = /* ]] || OUTPUT_DIR="$PROJECT_ROOT/$OUTPUT_DIR"

[[ -f "$BROWSER_BINARY" ]] || fail "browser binary not found: $BROWSER_BINARY"
[[ -f "$RENDERER_BINARY" ]] || fail "renderer binary not found: $RENDERER_BINARY"

if [[ -z "$APP_VERSION" ]]; then
    APP_VERSION="$(
        sed -n '/^\[workspace\.package\]$/,/^\[/{s/^version = "\([^"]*\)"/\1/p;}' \
            "$PROJECT_ROOT/Cargo.toml" | head -1
    )"
fi
[[ "$APP_VERSION" =~ ^[0-9]+(\.[0-9]+){1,2}([+-][A-Za-z0-9.-]+)?$ ]] \
    || fail "version must be a semantic version: $APP_VERSION"
BUNDLE_VERSION="${APP_VERSION%%[-+]*}"

mkdir -p "$OUTPUT_DIR"
if [[ -z "$ARCHIVE_PATH" ]]; then
    ARCHIVE_PATH="$OUTPUT_DIR/zero-browser-macos.zip"
elif [[ "$ARCHIVE_PATH" != /* ]]; then
    ARCHIVE_PATH="$PROJECT_ROOT/$ARCHIVE_PATH"
fi
mkdir -p "$(dirname "$ARCHIVE_PATH")"

APP_BUNDLE="$OUTPUT_DIR/$APP_NAME.app"
HELPER_BUNDLE="$APP_BUNDLE/Contents/Frameworks/$HELPER_NAME.app"
HELPER_EXECUTABLE="$HELPER_BUNDLE/Contents/MacOS/$HELPER_NAME"
ENTITLEMENTS_PATH="$OUTPUT_DIR/.ZeroBrowser.entitlements.plist"
ICONSET_SOURCE="$PROJECT_ROOT/apps/browser/assets/icons-gen/iconset"
ICONSET_DIR="$OUTPUT_DIR/.ZeroBrowser.iconset"
trap 'rm -f "$ENTITLEMENTS_PATH"; rm -rf "$ICONSET_DIR"' EXIT
rm -rf "$APP_BUNDLE"
mkdir -p \
    "$APP_BUNDLE/Contents/MacOS" \
    "$APP_BUNDLE/Contents/Resources" \
    "$HELPER_BUNDLE/Contents/MacOS" \
    "$HELPER_BUNDLE/Contents/Resources"

install -m 755 "$BROWSER_BINARY" "$APP_BUNDLE/Contents/MacOS/ZeroBrowser"
install -m 755 "$RENDERER_BINARY" "$HELPER_EXECUTABLE"

cat > "$APP_BUNDLE/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "https://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key><string>en</string>
    <key>CFBundleDisplayName</key><string>${APP_NAME}</string>
    <key>CFBundleExecutable</key><string>${APP_NAME}</string>
    <key>CFBundleIconFile</key><string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key><string>${BUNDLE_ID}</string>
    <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
    <key>CFBundleName</key><string>${APP_NAME}</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>${BUNDLE_VERSION}</string>
    <key>CFBundleVersion</key><string>${BUNDLE_VERSION}</string>
    <key>LSMinimumSystemVersion</key><string>12.0</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key><true/>
</dict>
</plist>
EOF
printf 'APPL????' > "$APP_BUNDLE/Contents/PkgInfo"
plutil -lint "$APP_BUNDLE/Contents/Info.plist"

cat > "$HELPER_BUNDLE/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "https://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key><string>en</string>
    <key>CFBundleDisplayName</key><string>${HELPER_NAME}</string>
    <key>CFBundleExecutable</key><string>${HELPER_NAME}</string>
    <key>CFBundleIconFile</key><string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key><string>${BUNDLE_ID}.helper.renderer</string>
    <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
    <key>CFBundleName</key><string>${HELPER_NAME}</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>${BUNDLE_VERSION}</string>
    <key>CFBundleVersion</key><string>${BUNDLE_VERSION}</string>
    <key>LSMinimumSystemVersion</key><string>12.0</string>
    <key>LSUIElement</key><true/>
</dict>
</plist>
EOF
printf 'APPL????' > "$HELPER_BUNDLE/Contents/PkgInfo"
plutil -lint "$HELPER_BUNDLE/Contents/Info.plist"

[[ -d "$ICONSET_SOURCE" ]] || fail "iconset not found: $ICONSET_SOURCE"
rm -rf "$ICONSET_DIR"
cp -R "$ICONSET_SOURCE" "$ICONSET_DIR"
sips -s format png -z 1024 1024 "$PROJECT_ROOT/apps/browser/assets/app-icon.svg" \
    --out "$ICONSET_DIR/icon_512x512@2x.png" >/dev/null
iconutil -c icns "$ICONSET_DIR" -o "$APP_BUNDLE/Contents/Resources/ZeroBrowser.icns"
cp "$APP_BUNDLE/Contents/Resources/ZeroBrowser.icns" "$HELPER_BUNDLE/Contents/Resources/ZeroBrowser.icns"

cat > "$ENTITLEMENTS_PATH" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "https://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.cs.allow-jit</key><true/>
</dict>
</plist>
EOF

if [[ -n "$SIGN_IDENTITY" ]]; then
    info "Signing app with Developer ID identity: $SIGN_IDENTITY"
    codesign --force --options runtime --timestamp --sign "$SIGN_IDENTITY" \
        --entitlements "$ENTITLEMENTS_PATH" "$HELPER_EXECUTABLE"
    codesign --force --options runtime --timestamp --sign "$SIGN_IDENTITY" \
        --entitlements "$ENTITLEMENTS_PATH" "$HELPER_BUNDLE"
    codesign --force --options runtime --timestamp --sign "$SIGN_IDENTITY" \
        --entitlements "$ENTITLEMENTS_PATH" "$APP_BUNDLE"
else
    info "No Developer ID identity provided; applying ad-hoc signature"
    codesign --force --sign - "$HELPER_EXECUTABLE"
    codesign --force --sign - "$HELPER_BUNDLE"
    codesign --force --sign - "$APP_BUNDLE"
fi
codesign --verify --deep --strict --verbose=2 "$APP_BUNDLE"

create_archive() {
    rm -f "$ARCHIVE_PATH"
    ditto -c -k --keepParent "$APP_BUNDLE" "$ARCHIVE_PATH"
}

create_archive

if [[ "$NOTARIZE" == true ]]; then
    [[ -n "$SIGN_IDENTITY" ]] || fail "--notarize requires --sign-identity"
    [[ -n "${APPLE_ID:-}" ]] || fail "--notarize requires APPLE_ID"
    [[ -n "${APPLE_TEAM_ID:-}" ]] || fail "--notarize requires APPLE_TEAM_ID"
    [[ -n "${APPLE_APP_PASSWORD:-}" ]] || fail "--notarize requires APPLE_APP_PASSWORD"
    command -v xcrun >/dev/null || fail "xcrun is required for notarization"

    info "Submitting archive for Apple notarization"
    xcrun notarytool submit "$ARCHIVE_PATH" \
        --apple-id "$APPLE_ID" \
        --team-id "$APPLE_TEAM_ID" \
        --password "$APPLE_APP_PASSWORD" \
        --wait
    xcrun stapler staple "$APP_BUNDLE"
    xcrun stapler validate "$APP_BUNDLE"
    spctl --assess --type execute --verbose=4 "$APP_BUNDLE"
    create_archive
fi

info "App bundle: $APP_BUNDLE"
info "Archive: $ARCHIVE_PATH"
