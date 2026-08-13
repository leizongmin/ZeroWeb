#!/bin/sh
# 真实 ZeroBrowser 窗口的 legacy/compositor 双模式产品 smoke。

set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
OUT_DIR=${BROWSER_SMOKE_OUT_DIR:-"$ROOT/target/browser-compositor-smoke"}
RAW_BROWSER_BIN="$ROOT/target/release/zero-browser"
RAW_RENDERER_BIN="$ROOT/target/release/zero-renderer"
BIN="$RAW_BROWSER_BIN"
COMPOSITOR_BIN="$ROOT/target/release/zero-compositor"

mkdir -p "$OUT_DIR"

echo "browser-compositor-smoke: preparing rusty_v8"
bash "$ROOT/scripts/download-rusty-v8.sh"
echo "browser-compositor-smoke: building product processes"
cargo build --manifest-path "$ROOT/Cargo.toml" --release \
    -p zero-browser -p zero-renderer -p zero-compositor

if test "$(uname -s)" = "Darwin"; then
    echo "browser-compositor-smoke: assembling macOS product bundle"
    bash "$ROOT/scripts/package-macos.sh" \
        --browser "$RAW_BROWSER_BIN" \
        --renderer "$RAW_RENDERER_BIN" \
        --compositor "$COMPOSITOR_BIN" \
        --output-dir "$OUT_DIR/package" \
        --archive "$OUT_DIR/package/zero-browser-smoke.zip"
    BIN="$OUT_DIR/package/ZeroBrowser.app/Contents/MacOS/ZeroBrowser"
fi

run_mode() {
    mode=$1
    png="$OUT_DIR/$mode.png"
    log="$OUT_DIR/$mode.log"
    rm -f "$png" "$log"

    echo "browser-compositor-smoke: running $mode"
    if test "$mode" = "legacy"; then
        export ZW_COMPOSITOR_PROCESS=0
    else
        unset ZW_COMPOSITOR_PROCESS
    fi
    env \
        RUST_LOG=info \
        ZERO_BROWSER_PRODUCT_SMOKE=1 \
        ZW_COMPOSITOR_BIN="$COMPOSITOR_BIN" \
        "$BIN" --renderer=cpu --scale=1 --smoke-capture="$png" >"$log" 2>&1

    test -s "$png" || {
        echo "browser-compositor-smoke: missing PNG for $mode" >&2
        exit 1
    }
    file "$png" | grep -q 'PNG image data' || {
        echo "browser-compositor-smoke: invalid PNG for $mode" >&2
        exit 1
    }
    grep -q "SMOKE_CAPTURE mode=$mode fixture=zero://newtab" "$log"
    grep -q 'SMOKE_REGION name=chrome .*unique_bins=.*dominant_ratio=.*signature=' "$log"
    grep -q 'SMOKE_REGION name=page .*unique_bins=.*dominant_ratio=.*signature=' "$log"
    if grep -qE 'SMOKE_FAILURE|Compositor disconnected; switched all renderers to legacy frame publishing|fallback=true|panicked at' "$log"; then
        echo "browser-compositor-smoke: failure or fallback found in $mode log" >&2
        exit 1
    fi
}

run_mode legacy
grep -q 'SMOKE_EVENT component=zero-renderer event=frame_published mode=legacy kind=ViewPainted' "$OUT_DIR/legacy.log"
grep -q 'SMOKE_EVENT component=browser event=legacy_view_painted' "$OUT_DIR/legacy.log"
grep -q 'SMOKE_EVENT component=browser event=frame_captured source=legacy_view_painted fallback=false' "$OUT_DIR/legacy.log"

run_mode compositor
grep -q 'SMOKE_EVENT component=compositor_client status=Healthy' "$OUT_DIR/compositor.log"
grep -q 'SMOKE_EVENT component=zero-renderer event=frame_published mode=compositor kind=CompositorFrame' "$OUT_DIR/compositor.log"
grep -q 'SMOKE_EVENT component=compositor_client event=frame_submitted' "$OUT_DIR/compositor.log"
grep -q 'SMOKE_EVENT component=zero-compositor event=frame_committed' "$OUT_DIR/compositor.log"
grep -q 'SMOKE_EVENT component=compositor_client event=frame_completed' "$OUT_DIR/compositor.log"
grep -q 'SMOKE_EVENT component=browser event=compositor_bitmap_adopted' "$OUT_DIR/compositor.log"
grep -q 'SMOKE_EVENT component=browser event=frame_captured source=compositor_bitmap fallback=false' "$OUT_DIR/compositor.log"
if grep -q 'SMOKE_EVENT component=browser event=legacy_view_painted' "$OUT_DIR/compositor.log"; then
    echo "browser-compositor-smoke: compositor mode consumed a legacy ViewPainted frame" >&2
    exit 1
fi

if test "$(uname -s)" = "Linux"; then
    run_mode_gpu_dmabuf() {
        mode=gpu-dmabuf
        png="$OUT_DIR/$mode.png"
        log="$OUT_DIR/$mode.log"
        rm -f "$png" "$log"

        echo "browser-compositor-smoke: running $mode"
        unset ZW_COMPOSITOR_PROCESS
        unset ZW_BROWSER_GPU_DMABUF_IMPORT
        env \
            RUST_LOG=info \
            ZERO_BROWSER_PRODUCT_SMOKE=1 \
            ZERO_BROWSER_GPU_DMABUF_SMOKE=1 \
            ZW_COMPOSITOR_BIN="$COMPOSITOR_BIN" \
            "$BIN" --renderer=gpu --scale=1 --smoke-capture="$png" >"$log" 2>&1

        test -s "$png" || {
            echo "browser-compositor-smoke: missing PNG for $mode" >&2
            exit 1
        }
        file "$png" | grep -q 'PNG image data' || {
            echo "browser-compositor-smoke: invalid PNG for $mode" >&2
            exit 1
        }
        grep -q "SMOKE_CAPTURE mode=$mode fixture=zero://newtab" "$log"
        grep -q 'SMOKE_EVENT component=compositor_client status=Healthy' "$log"
        grep -q 'SMOKE_EVENT component=browser event=compositor_dmabuf_adopted' "$log"
        grep -q 'SMOKE_EVENT component=browser event=frame_captured source=compositor_bitmap fallback=false' "$log"
        if grep -qE 'SMOKE_FAILURE|Compositor disconnected; switched all renderers to legacy frame publishing|fallback=true|panicked at' "$log"; then
            echo "browser-compositor-smoke: failure or fallback found in $mode log" >&2
            exit 1
        fi
    }
    run_mode_gpu_dmabuf
fi

legacy_signature=$(sed -n 's/.*SMOKE_REGION name=page .* signature=//p' "$OUT_DIR/legacy.log" | tail -1)
compositor_signature=$(sed -n 's/.*SMOKE_REGION name=page .* signature=//p' "$OUT_DIR/compositor.log" | tail -1)
legacy_dark=$(sed -n 's/.*SMOKE_REGION name=page .* dark_pixels=\([0-9][0-9]*\) dark_ratio=.*/\1/p' "$OUT_DIR/legacy.log" | tail -1)
compositor_dark=$(sed -n 's/.*SMOKE_REGION name=page .* dark_pixels=\([0-9][0-9]*\) dark_ratio=.*/\1/p' "$OUT_DIR/compositor.log" | tail -1)
awk -v legacy="$legacy_signature" -v compositor="$compositor_signature" \
    -v legacy_dark="$legacy_dark" -v compositor_dark="$compositor_dark" '
BEGIN {
    legacy_count = split(legacy, a, ",")
    compositor_count = split(compositor, b, ",")
    if (legacy_count != 64 || compositor_count != 64) {
        print "browser-compositor-smoke: invalid page signature length" > "/dev/stderr"
        exit 1
    }
    total_delta = 0
    close_samples = 0
    for (i = 1; i <= legacy_count; i++) {
        delta = a[i] - b[i]
        if (delta < 0) {
            delta = -delta
        }
        total_delta += delta
        if (delta <= 32) {
            close_samples++
        }
    }
    mean_delta = total_delta / legacy_count
    dark_ratio = legacy_dark > 0 ? compositor_dark / legacy_dark : 0
    printf "SMOKE_SIMILARITY page_mean_luma_delta=%.3f close_samples=%d/64 dark_pixel_ratio=%.3f\n", mean_delta, close_samples, dark_ratio
    if (mean_delta > 20 || close_samples < 48 || dark_ratio < 0.5 || dark_ratio > 2.0) {
        print "browser-compositor-smoke: legacy/compositor page signatures differ too much" > "/dev/stderr"
        exit 1
    }
}'

echo "browser-compositor-smoke: PASS"
grep 'SMOKE_CAPTURE\|SMOKE_REGION' "$OUT_DIR/legacy.log"
grep 'SMOKE_EVENT component=compositor_client\|SMOKE_EVENT component=zero-compositor\|SMOKE_EVENT component=browser event=compositor_bitmap_adopted\|SMOKE_CAPTURE\|SMOKE_REGION' "$OUT_DIR/compositor.log"
