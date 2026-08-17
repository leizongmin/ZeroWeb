#!/usr/bin/env bash
# Build a distributable ZeroBrowser.app and zip archive on macOS.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

APP_NAME="ZeroBrowser"
BUNDLE_ID="com.zeroweb.browser"
RENDERER_HELPER_NAME="ZeroBrowser Helper (Renderer)"
COMPOSITOR_HELPER_NAME="ZeroBrowser Helper (Compositor)"
DECODER_HELPER_NAME="ZeroBrowser Helper (Image Decoder)"
OUTPUT_DIR="$PROJECT_ROOT/target/packages"
BROWSER_BINARY=""
RENDERER_BINARY=""
COMPOSITOR_BINARY=""
DECODER_BINARY=""
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
  --compositor PATH      Use an existing zero-compositor binary.
  --decoder PATH         Use an existing zero-image-decoder binary.
  --output-dir PATH      Output directory (default: target/packages).
  --version VERSION      Bundle version (default: current local date as YY.M.D).
  --archive PATH         Zip path (default: <output-dir>/zero-browser-macos.zip).
  --sign-identity NAME   Developer ID Application identity. Defaults to
                         MACOS_SIGN_IDENTITY. Without it, the app is ad-hoc signed.
  --notarize             Submit with notarytool and staple the accepted ticket.
                         Requires APPLE_ID, APPLE_TEAM_ID, and APPLE_APP_PASSWORD.
  -h, --help             Show this help.

When process paths are omitted, the script builds all four release binaries.
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
        --compositor)
            [[ $# -ge 2 ]] || fail "--compositor requires a path"
            COMPOSITOR_BINARY="$2"
            shift 2
            ;;
        --decoder)
            [[ $# -ge 2 ]] || fail "--decoder requires a path"
            DECODER_BINARY="$2"
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

if [[ -z "$APP_VERSION" ]]; then
    if [[ -n "${SOURCE_DATE_EPOCH:-}" ]]; then
        read -r version_year version_month version_day < <(
            date -u -r "$SOURCE_DATE_EPOCH" "+%y %m %d"
        ) || fail "invalid SOURCE_DATE_EPOCH: $SOURCE_DATE_EPOCH"
    else
        read -r version_year version_month version_day < <(date "+%y %m %d")
    fi
    APP_VERSION="$((10#$version_year)).$((10#$version_month)).$((10#$version_day))"
fi
[[ "$APP_VERSION" =~ ^[0-9]{1,2}\.[0-9]{1,2}\.[0-9]{1,2}$ ]] \
    || fail "version must use YY.M.D: $APP_VERSION"
IFS=. read -r version_year version_month version_day <<< "$APP_VERSION"
(( 10#$version_year <= 99 && 10#$version_month >= 1 && 10#$version_month <= 12 \
    && 10#$version_day >= 1 && 10#$version_day <= 31 )) \
    || fail "version contains an invalid date: $APP_VERSION"
BUNDLE_VERSION="$APP_VERSION"
export ZERO_BUILD_VERSION="$APP_VERSION"

if [[ -n "$BROWSER_BINARY" || -n "$RENDERER_BINARY" || -n "$COMPOSITOR_BINARY" || -n "$DECODER_BINARY" ]]; then
    [[ -n "$BROWSER_BINARY" && -n "$RENDERER_BINARY" && -n "$COMPOSITOR_BINARY" && -n "$DECODER_BINARY" ]] \
        || fail "--browser, --renderer, --compositor, and --decoder must be provided together"
else
    info "Building release binaries"
    (
        cd "$PROJECT_ROOT"
        cargo build --release -p zero-browser
        cargo build --release -p zero-renderer -p zero-compositor -p zero-image-decoder
    )
    BROWSER_BINARY="$PROJECT_ROOT/target/release/zero-browser"
    RENDERER_BINARY="$PROJECT_ROOT/target/release/zero-renderer"
    COMPOSITOR_BINARY="$PROJECT_ROOT/target/release/zero-compositor"
    DECODER_BINARY="$PROJECT_ROOT/target/release/zero-image-decoder"
fi

[[ "$BROWSER_BINARY" = /* ]] || BROWSER_BINARY="$PROJECT_ROOT/$BROWSER_BINARY"
[[ "$RENDERER_BINARY" = /* ]] || RENDERER_BINARY="$PROJECT_ROOT/$RENDERER_BINARY"
[[ "$COMPOSITOR_BINARY" = /* ]] || COMPOSITOR_BINARY="$PROJECT_ROOT/$COMPOSITOR_BINARY"
[[ "$DECODER_BINARY" = /* ]] || DECODER_BINARY="$PROJECT_ROOT/$DECODER_BINARY"
[[ "$OUTPUT_DIR" = /* ]] || OUTPUT_DIR="$PROJECT_ROOT/$OUTPUT_DIR"

[[ -f "$BROWSER_BINARY" ]] || fail "browser binary not found: $BROWSER_BINARY"
[[ -f "$RENDERER_BINARY" ]] || fail "renderer binary not found: $RENDERER_BINARY"
[[ -f "$COMPOSITOR_BINARY" ]] || fail "compositor binary not found: $COMPOSITOR_BINARY"
[[ -f "$DECODER_BINARY" ]] || fail "image-decoder binary not found: $DECODER_BINARY"

mkdir -p "$OUTPUT_DIR"
if [[ -z "$ARCHIVE_PATH" ]]; then
    ARCHIVE_PATH="$OUTPUT_DIR/zero-browser-macos.zip"
elif [[ "$ARCHIVE_PATH" != /* ]]; then
    ARCHIVE_PATH="$PROJECT_ROOT/$ARCHIVE_PATH"
fi
mkdir -p "$(dirname "$ARCHIVE_PATH")"

APP_BUNDLE="$OUTPUT_DIR/$APP_NAME.app"
RENDERER_HELPER_BUNDLE="$APP_BUNDLE/Contents/Frameworks/$RENDERER_HELPER_NAME.app"
COMPOSITOR_HELPER_BUNDLE="$APP_BUNDLE/Contents/Frameworks/$COMPOSITOR_HELPER_NAME.app"
DECODER_HELPER_BUNDLE="$APP_BUNDLE/Contents/Frameworks/$DECODER_HELPER_NAME.app"
RENDERER_HELPER_EXECUTABLE="$RENDERER_HELPER_BUNDLE/Contents/MacOS/$RENDERER_HELPER_NAME"
COMPOSITOR_HELPER_EXECUTABLE="$COMPOSITOR_HELPER_BUNDLE/Contents/MacOS/$COMPOSITOR_HELPER_NAME"
DECODER_HELPER_EXECUTABLE="$DECODER_HELPER_BUNDLE/Contents/MacOS/$DECODER_HELPER_NAME"
ENTITLEMENTS_PATH="$OUTPUT_DIR/.ZeroBrowser.entitlements.plist"
ICONSET_SOURCE="$PROJECT_ROOT/apps/browser/assets/icons-gen/iconset"
ICONSET_DIR="$OUTPUT_DIR/.ZeroBrowser.iconset"
trap 'rm -f "$ENTITLEMENTS_PATH"; rm -rf "$ICONSET_DIR"' EXIT
rm -rf "$APP_BUNDLE"
mkdir -p \
    "$APP_BUNDLE/Contents/MacOS" \
    "$APP_BUNDLE/Contents/Resources" \
    "$RENDERER_HELPER_BUNDLE/Contents/MacOS" \
    "$RENDERER_HELPER_BUNDLE/Contents/Resources" \
    "$COMPOSITOR_HELPER_BUNDLE/Contents/MacOS" \
    "$COMPOSITOR_HELPER_BUNDLE/Contents/Resources" \
    "$DECODER_HELPER_BUNDLE/Contents/MacOS" \
    "$DECODER_HELPER_BUNDLE/Contents/Resources"

install -m 755 "$BROWSER_BINARY" "$APP_BUNDLE/Contents/MacOS/ZeroBrowser"
install -m 755 "$RENDERER_BINARY" "$RENDERER_HELPER_EXECUTABLE"
install -m 755 "$COMPOSITOR_BINARY" "$COMPOSITOR_HELPER_EXECUTABLE"
install -m 755 "$DECODER_BINARY" "$DECODER_HELPER_EXECUTABLE"

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

write_helper_info_plist() {
    local bundle="$1"
    local name="$2"
    local identifier_suffix="$3"
    cat > "$bundle/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "https://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key><string>en</string>
    <key>CFBundleDisplayName</key><string>${name}</string>
    <key>CFBundleExecutable</key><string>${name}</string>
    <key>CFBundleIconFile</key><string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key><string>${BUNDLE_ID}.helper.${identifier_suffix}</string>
    <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
    <key>CFBundleName</key><string>${name}</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>${BUNDLE_VERSION}</string>
    <key>CFBundleVersion</key><string>${BUNDLE_VERSION}</string>
    <key>LSMinimumSystemVersion</key><string>12.0</string>
    <key>LSUIElement</key><true/>
</dict>
</plist>
EOF
    printf 'APPL????' > "$bundle/Contents/PkgInfo"
    plutil -lint "$bundle/Contents/Info.plist"
}

write_helper_info_plist "$RENDERER_HELPER_BUNDLE" "$RENDERER_HELPER_NAME" "renderer"
write_helper_info_plist "$COMPOSITOR_HELPER_BUNDLE" "$COMPOSITOR_HELPER_NAME" "compositor"
write_helper_info_plist "$DECODER_HELPER_BUNDLE" "$DECODER_HELPER_NAME" "image-decoder"

[[ -d "$ICONSET_SOURCE" ]] || fail "iconset not found: $ICONSET_SOURCE"
rm -rf "$ICONSET_DIR"
cp -R "$ICONSET_SOURCE" "$ICONSET_DIR"
sips -s format png -z 1024 1024 "$PROJECT_ROOT/apps/browser/assets/app-icon.svg" \
    --out "$ICONSET_DIR/icon_512x512@2x.png" >/dev/null
iconutil -c icns "$ICONSET_DIR" -o "$APP_BUNDLE/Contents/Resources/ZeroBrowser.icns"
for helper_bundle in "$RENDERER_HELPER_BUNDLE" "$COMPOSITOR_HELPER_BUNDLE" "$DECODER_HELPER_BUNDLE"; do
    cp "$APP_BUNDLE/Contents/Resources/ZeroBrowser.icns" "$helper_bundle/Contents/Resources/ZeroBrowser.icns"
done

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
        --entitlements "$ENTITLEMENTS_PATH" "$RENDERER_HELPER_EXECUTABLE"
    codesign --force --options runtime --timestamp --sign "$SIGN_IDENTITY" \
        --entitlements "$ENTITLEMENTS_PATH" "$COMPOSITOR_HELPER_EXECUTABLE"
    codesign --force --options runtime --timestamp --sign "$SIGN_IDENTITY" \
        --entitlements "$ENTITLEMENTS_PATH" "$DECODER_HELPER_EXECUTABLE"
    codesign --force --options runtime --timestamp --sign "$SIGN_IDENTITY" \
        --entitlements "$ENTITLEMENTS_PATH" "$RENDERER_HELPER_BUNDLE"
    codesign --force --options runtime --timestamp --sign "$SIGN_IDENTITY" \
        --entitlements "$ENTITLEMENTS_PATH" "$COMPOSITOR_HELPER_BUNDLE"
    codesign --force --options runtime --timestamp --sign "$SIGN_IDENTITY" \
        --entitlements "$ENTITLEMENTS_PATH" "$DECODER_HELPER_BUNDLE"
    codesign --force --options runtime --timestamp --sign "$SIGN_IDENTITY" \
        --entitlements "$ENTITLEMENTS_PATH" "$APP_BUNDLE"
else
    info "No Developer ID identity provided; applying ad-hoc signature"
    codesign --force --sign - "$RENDERER_HELPER_EXECUTABLE"
    codesign --force --sign - "$COMPOSITOR_HELPER_EXECUTABLE"
    codesign --force --sign - "$DECODER_HELPER_EXECUTABLE"
    codesign --force --sign - "$RENDERER_HELPER_BUNDLE"
    codesign --force --sign - "$COMPOSITOR_HELPER_BUNDLE"
    codesign --force --sign - "$DECODER_HELPER_BUNDLE"
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
