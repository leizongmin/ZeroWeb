.PHONY: setup-rusty-v8 fetch-wpt-data build browser browser-cpu browser-wpt-parity browser-debug browser-debug-wayland browser-debug-wayland-log browser-debug-x11 test reftest reftest-oracle capture-oracle product-smoke product-smoke-legacy

setup-rusty-v8:
	bash scripts/download-rusty-v8.sh

# WPT reftest 数据（上游 web-platform-tests/wpt 子集，~19952 文件，独立 repo）。
# reftest / reftest-oracle 会自动前置触发；目录已存在则跳过，刷新需先 rm -rf。
WPT_DATA_REPO ?= https://github.com/leizongmin/zeroweb-wpt-data.git
WPT_DATA_REF  ?= v1.1
WPT_DATA_DIR  ?= tests/wpt-runner/wpt-data
fetch-wpt-data:
	@if [ -d "$(WPT_DATA_DIR)" ] && [ -n "$$(ls -A $(WPT_DATA_DIR) 2>/dev/null)" ]; then echo "wpt-data 已存在 ($(WPT_DATA_DIR), ref=$(WPT_DATA_REF))；刷新请先 rm -rf 该目录"; else echo "fetch wpt-data $(WPT_DATA_REF) → $(WPT_DATA_DIR)"; git clone --depth=1 --branch $(WPT_DATA_REF) $(WPT_DATA_REPO) "$(WPT_DATA_DIR)"; rm -rf "$(WPT_DATA_DIR)/.git"; fi

build: setup-rusty-v8
	cargo build --workspace

# WAYLAND_DEBUG and WINIT_UNIX_BACKEND=x11 are separate targets because they
# debug different backends and should not be combined in one run.

BROWSER_BIN = ./target/release/zero-browser

browser: setup-rusty-v8
	cargo build --release -p zero-browser -p zero-renderer
	RUST_BACKTRACE=1 $(BROWSER_BIN) --renderer=gpu

# 与 browser-cpu 相同（保留别名）
browser-wpt-parity: browser-cpu

browser-cpu: setup-rusty-v8
	cargo build --release -p zero-browser -p zero-renderer
	RUST_BACKTRACE=1 $(BROWSER_BIN) --wpt-parity

browser-debug: setup-rusty-v8
	cargo build --release -p zero-browser -p zero-renderer
	RUST_BACKTRACE=1 $(BROWSER_BIN) --renderer=gpu

browser-debug-wayland: setup-rusty-v8
	mkdir -p target
	RUST_BACKTRACE=1 WINIT_UNIX_BACKEND=wayland WAYLAND_DEBUG=1 $(BROWSER_RUN) 2>&1 | tee target/zero-browser-wayland-debug.log

browser-debug-wayland-log: setup-rusty-v8
	mkdir -p target
	RUST_BACKTRACE=1 WINIT_UNIX_BACKEND=wayland WAYLAND_DEBUG=1 $(BROWSER_RUN) > target/zero-browser-wayland-debug.log 2>&1

browser-debug-x11: setup-rusty-v8
	RUST_BACKTRACE=1 WAYLAND_DISPLAY= WAYLAND_SOCKET= WINIT_UNIX_BACKEND=x11 $(BROWSER_RUN)

# ── 测试防护 (test-guard) ──────────────────────────────────────────────
# test-guard 跨平台 (macOS/Linux) 包裹测试命令：单进程 RSS>6GB 或全树>16GB
# 或总时长>1800s 即杀掉整棵进程树（退出 124），防止内存型 bug（如无限循环
# realloc、CSS parser 未闭合括号死循环）触发系统级 OOM 连累 tmux session /
# rally 无人值守流程。源码 scripts/test-guard.rs，std-only，rustc 直接编译。
target/test-guard: scripts/test-guard.rs
	@mkdir -p target
	rustc -O scripts/test-guard.rs -o target/test-guard

# 全量测试（被 test-guard 包裹）。无人值守 / rally / CI 请用此 target，
# 不要裸跑 cargo test。可调阈值：./target/test-guard --per-proc-mem 8 --total-mem 20 -- cargo test --workspace
test: target/test-guard
	# 本地（WSL2 等）无可用 GPU 后端时，zero-render-foundation 的 wgpu headless 设备测试会在
	# surface 配置 / 渲染回读路径上间歇性长时间阻塞（>30min，跨 renderer/tests.rs 与 surface.rs
	# 多模块，致 test-guard 1800s 总超时连累整树）。本地 make test 用 --exclude 跳过该 crate；
	# CI 直接跑 `cargo test --workspace`（ci.yml，真 Vulkan 后端）正常全量执行该 crate 测试。
	# 需本地验证 render-foundation 时：test-guard -- cargo test -p zero-render-foundation。
	./target/test-guard --per-proc-mem 10 --total-mem 28 -- cargo test --workspace --exclude zero-render-foundation -- --test-threads=2

# WPT reftest（release 构建，约 4× 快于 debug；同样被 test-guard 包裹）。
reftest: fetch-wpt-data target/test-guard
	./target/test-guard -- cargo run --release --bin zero-wpt-runner -- reftest

# DC-14 独立 Oracle：渲染上游 WPT test 页 vs chromium oracle-shots，报告真一致率
# （chromium-Oracle pass-rate，替代 self-ref 的 ~46.5% 假通过）。oracle-shots 由
# capture-oracle-per-dir.mjs 本地抓取（gitignored，可再生）。非硬 fail 门禁（报告性）。
# 用法：make reftest-oracle                       全量（慢，~10k 案）
#       make reftest-oracle DIR=css-grid          单目录
#       make reftest-oracle DIR=css-grid ORACLE_PASS_RATIO=0.005   调严判定阈值
reftest-oracle: fetch-wpt-data target/test-guard
	./target/test-guard -- cargo run --release --bin zero-wpt-runner -- reftest-oracle $(DIR)

# DC-14 oracle-shots 抓取（R1253）：WSL2 + chromium 150 headless 渲染 SIGTRAP，用非 headless
# chromium（GUI 渲染路径）+ CDP。抓完后 oracle-shots 存 tests/wpt-runner/oracle-shots/，
# 再 make reftest-oracle DIR=... 跑 A/B（reftest-oracle 读存 PNG，不需 chromium）。
# 用法: make capture-oracle DIR=css/css-flexbox
#       make capture-oracle DIR=css/css-flexbox EXTRA="--skip-existing"
#       多目录: ./scripts/run-oracle-capture.sh --category css/css-flexbox --category css/css-grid
capture-oracle: fetch-wpt-data
	./scripts/run-oracle-capture.sh --category $(DIR) $(EXTRA)

# 产品静态页 product-smoke 回归门禁（DC-13）：渲染 welcome.html vs chromium Oracle，
# diff > 阈值则失败（退出 2）。捕获产品可见回归——如 R428 min-size:auto 致
# welcome +7.65pp（24.63%），此前因 product-smoke 不在每轮验证而藏了 14 轮。
# 阈值 20% > 17% baseline（残余为字体/line-height 噪声 + R109 结构性，非回归）。
# 用法：make product-smoke        调整阈值：make product-smoke MAX_DIFF=22
WELCOME_HTML := apps/browser/assets/welcome.html
WELCOME_ORACLE := docs/goal/rendering-compat/evidence/product-static/welcome-chromium.png
product-smoke: target/test-guard
	./target/test-guard -- cargo run --release --bin zero-wpt-runner -- product-smoke $(WELCOME_HTML) --oracle $(WELCOME_ORACLE) --max-diff $(or $(MAX_DIFF),20) --struct-check --expect-class card:4 --expect-lines title:1 --expect-lines tagline:2
	./target/test-guard -- cargo run --release --bin zero-wpt-runner -- product-smoke apps/browser/assets/wintertc/index.html --base-dir apps/browser/assets/wintertc --struct-check
	./target/test-guard -- cargo run --release --bin zero-wpt-runner -- product-smoke apps/browser/assets/morning-work/article.html --base-dir apps/browser/assets/morning-work --struct-check

# Legacy Static Web smoke（DC-13，goal rendering-compat.md line 316）：跑 20 页
# HTML 3.2/4 + CSS1/2 静态 fixture，每页 chromium oracle vs ZeroWeb CPU diff%。
# ★ trend-only（退出 0）——diff 全归因字体墙（fontdue 行度量 vs chromium NotoSansCJK
# 垂直漂移 + AA，R633 多会话 plateau），非回归；像素阈值作趋势指标，不替代 WPT/DC-14
# 达标口径（goal line 318）。新增 fixture 写入 evidence/product-static/legacy-html/。
# 用法：make product-smoke-legacy
LEGACY_DIR := docs/goal/rendering-compat/evidence/product-static/legacy-html
product-smoke-legacy: target/test-guard
	bash $(LEGACY_DIR)/run-all.sh

