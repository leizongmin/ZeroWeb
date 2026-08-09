#!/bin/sh
# 在真实 ZeroBrowser 窗口中验收 compositor 模式的真实网站与常用操作。

set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
OUT_DIR=${BROWSER_GUI_SMOKE_OUT_DIR:-"$ROOT/.acceptance/browser-compositor-real-site"}
URL=${GUI_SMOKE_URL:-"https://www.iana.org/domains/reserved"}
SCREENSHOT_DIR="$OUT_DIR/screenshots"
LOG="$OUT_DIR/browser.log"
RAW_BROWSER_BIN="$ROOT/target/release/zero-browser"
RAW_RENDERER_BIN="$ROOT/target/release/zero-renderer"
COMPOSITOR_BIN="$ROOT/target/release/zero-compositor"
BIN="$RAW_BROWSER_BIN"

mkdir -p "$SCREENSHOT_DIR"
rm -f "$LOG" "$SCREENSHOT_DIR"/*.png

echo "browser-compositor-real-site-smoke: preparing rusty_v8"
bash "$ROOT/scripts/download-rusty-v8.sh"
echo "browser-compositor-real-site-smoke: building product processes"
cargo build --manifest-path "$ROOT/Cargo.toml" --release \
    -p zero-browser -p zero-renderer -p zero-compositor

if test "$(uname -s)" = "Darwin"; then
    echo "browser-compositor-real-site-smoke: assembling macOS product bundle"
    bash "$ROOT/scripts/package-macos.sh" \
        --browser "$RAW_BROWSER_BIN" \
        --renderer "$RAW_RENDERER_BIN" \
        --output-dir "$OUT_DIR/package" \
        --archive "$OUT_DIR/package/zero-browser-gui-smoke.zip"
    BIN="$OUT_DIR/package/ZeroBrowser.app/Contents/MacOS/ZeroBrowser"
fi

echo "browser-compositor-real-site-smoke: opening $URL"
env \
    RUST_LOG=info \
    ZERO_BROWSER_PRODUCT_SMOKE=1 \
    ZW_COMPOSITOR_PROCESS=1 \
    ZW_COMPOSITOR_BIN="$COMPOSITOR_BIN" \
    "$BIN" \
    --renderer=cpu \
    --scale=1 \
    --gui-smoke-url="$URL" \
    --gui-smoke-dir="$SCREENSHOT_DIR" >"$LOG" 2>&1

for screenshot in 01-loaded.png 02-scrolled.png 03-zoomed.png 04-reloaded.png; do
    path="$SCREENSHOT_DIR/$screenshot"
    test -s "$path" || {
        echo "browser-compositor-real-site-smoke: missing $screenshot" >&2
        exit 1
    }
    file "$path" | grep -q 'PNG image data' || {
        echo "browser-compositor-real-site-smoke: invalid PNG $screenshot" >&2
        exit 1
    }
done

grep -Fq "GUI_SMOKE_NAVIGATE url=$URL" "$LOG"
grep -Fq "SMOKE_CAPTURE mode=compositor fixture=$URL source=compositor_bitmap" "$LOG"
grep -q 'GUI_SMOKE_STEP step=loaded status=passed' "$LOG"
grep -q 'GUI_SMOKE_STEP step=scrolled status=passed' "$LOG"
grep -q 'GUI_SMOKE_STEP step=zoomed status=passed' "$LOG"
grep -q 'GUI_SMOKE_STEP step=reloaded status=passed' "$LOG"
grep -q 'GUI_SMOKE_ASSERT action=scroll visual_change=passed' "$LOG"
grep -q 'GUI_SMOKE_ASSERT action=zoom_in visual_change=passed' "$LOG"
grep -q 'GUI_SMOKE_ACTION action=reload status=executed' "$LOG"
grep -Fq "GUI_SMOKE_COMPLETE url=$URL steps=load,scroll,zoom_in,reload" "$LOG"

grep -q 'SMOKE_EVENT component=compositor_client status=Healthy' "$LOG"
grep -q 'SMOKE_EVENT component=zero-renderer event=frame_published mode=compositor kind=CompositorFrame' "$LOG"
grep -q 'SMOKE_EVENT component=compositor_client event=frame_submitted' "$LOG"
grep -q 'SMOKE_EVENT component=zero-compositor event=frame_committed' "$LOG"
grep -q 'SMOKE_EVENT component=compositor_client event=frame_completed' "$LOG"
grep -q 'SMOKE_EVENT component=browser event=compositor_bitmap_adopted' "$LOG"

if cmp -s "$SCREENSHOT_DIR/01-loaded.png" "$SCREENSHOT_DIR/02-scrolled.png"; then
    echo "browser-compositor-real-site-smoke: scroll screenshot did not change" >&2
    exit 1
fi
if cmp -s "$SCREENSHOT_DIR/02-scrolled.png" "$SCREENSHOT_DIR/03-zoomed.png"; then
    echo "browser-compositor-real-site-smoke: zoom screenshot did not change" >&2
    exit 1
fi

awk '
/SMOKE_EVENT component=browser event=compositor_bitmap_adopted/ {
    if (match($0, /epoch=[0-9]+/)) {
        epoch = substr($0, RSTART + 6, RLENGTH - 6) + 0
        if (reload_seen && epoch > before_reload) {
            reload_advanced = 1
        } else if (!reload_seen) {
            before_reload = epoch
        }
    }
}
/GUI_SMOKE_ACTION action=reload status=executed/ {
    reload_seen = 1
}
END {
    exit !(reload_seen && reload_advanced)
}' "$LOG" || {
    echo "browser-compositor-real-site-smoke: navigation epoch did not advance" >&2
    exit 1
}

if grep -qE 'GUI_SMOKE_FAILURE|SMOKE_FAILURE|Compositor disconnected; switched all renderers to legacy frame publishing|fallback=true|panicked at' "$LOG"; then
    echo "browser-compositor-real-site-smoke: failure or fallback found in log" >&2
    exit 1
fi
if grep -q 'SMOKE_EVENT component=browser event=legacy_view_painted' "$LOG"; then
    echo "browser-compositor-real-site-smoke: compositor mode consumed a legacy ViewPainted frame" >&2
    exit 1
fi

echo "browser-compositor-real-site-smoke: PASS"
grep -a 'GUI_SMOKE_\|SMOKE_CAPTURE' "$LOG"
