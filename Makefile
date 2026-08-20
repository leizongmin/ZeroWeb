.PHONY: setup-rusty-v8 fetch-wpt-data fetch-wpt-html-testharness update-wpt-data build browser-build browser browser-cpu browser-wpt-parity browser-debug browser-debug-wayland browser-debug-wayland-log browser-debug-x11 browser-compositor-smoke browser-compositor-real-site-smoke test testharness-html reftest reftest-oracle capture-oracle product-smoke-oracle product-smoke form-visual-smoke form-visual-browser-gpu-smoke product-smoke-legacy import-wpt audit-imported-font-resources reftest-trend reftest-trend-oracle reftest-smoke layout-golden layout-golden-update monthly-report bench bench-gate bench-capture bench-trend fetch-wpt-dom testharness-dom testharness-dom-native fetch-wpt-indexeddb testharness-indexeddb audit-wpt-service-workers-disposition fetch-wpt-service-workers-tier-a audit-wpt-service-workers-tier-a test-wpt-service-workers-tier-a-assets fetch-wpt-service-workers-next-wave audit-wpt-service-workers-next-wave test-wpt-service-workers-next-wave-assets fetch-wpt-service-workers-static-wave audit-wpt-service-workers-static-wave test-wpt-service-workers-static-wave-assets fetch-wpt-service-workers-update-wave audit-wpt-service-workers-update-wave test-wpt-service-workers-update-wave-assets fetch-wpt-service-workers-module-wave audit-wpt-service-workers-module-wave test-wpt-service-workers-module-wave-assets fetch-wpt-service-workers-module-bytecheck-wave audit-wpt-service-workers-module-bytecheck-wave test-wpt-service-workers-module-bytecheck-wave-assets fetch-wpt-service-workers-module-cors-wave audit-wpt-service-workers-module-cors-wave test-wpt-service-workers-module-cors-wave-assets fetch-wpt-service-workers-module-registration-wave audit-wpt-service-workers-module-registration-wave test-wpt-service-workers-module-registration-wave-assets fetch-wpt-service-workers-module-type-update-wave audit-wpt-service-workers-module-type-update-wave test-wpt-service-workers-module-type-update-wave-assets testharness-service-workers-core baseline-wpt-service-workers-core target-disk-guard target/test-guard android-preflight android-apk android-release-apk android-wsl-renderer-apk android-wsl-renderer-install-smoke android-install-smoke

# Windows 的 make recipe 可能落到 cmd.exe（本机）或 Git Bash（GitHub Actions runner）——
# 统一显式走 Git Bash，避免 cmd 语法在 bash 下解析失败（2026-08-16 CI 实测）。
# 定义在前供所有 bash 脚本入口（target-disk-guard / fetch-wpt-data 等）共用。
ifeq ($(OS),Windows_NT)
WPT_BASH ?= "C:/Program Files/Git/bin/bash.exe"
else
WPT_BASH ?= bash
endif

# target/ 磁盘占用守卫（2026-08-18：长时间 rally 循环曾把整块磁盘跑满——target/
# 多 feature 组合产物 + incremental 缓存只增不减，且仓库根 core.* OOM 转储无人清）。
# 重型入口（build/test/browser/reftest/product-smoke/bench/Android 构建家族）前置执行：
# 每次清仓库根 core.* 转储，target/ 超 50GB 自动分级清理并继续，阈值内零开销放行。阈值可调：
# make test ZW_TARGET_DISK_LIMIT_GB=80；跳过：ZW_TARGET_DISK_GUARD=0。
# 详见 scripts/target-disk-guard.sh 与 docs/rally/oom-guard.md。
target-disk-guard:
	@$(WPT_BASH) scripts/target-disk-guard.sh

ifeq ($(OS),Windows_NT)
setup-rusty-v8:
	powershell -NoProfile -ExecutionPolicy Bypass -File scripts\download-rusty-v8.ps1
else
setup-rusty-v8:
	bash scripts/download-rusty-v8.sh
endif

# WPT reftest 数据（上游 web-platform-tests/wpt 子集，~19952 文件，独立 repo）。
# reftest / reftest-oracle 会自动前置触发；目录已存在则跳过，刷新需先 rm -rf。
WPT_DATA_REPO ?= https://github.com/leizongmin/zeroweb-wpt-data.git
WPT_DATA_REF  ?= v1.10
WPT_DATA_DIR  ?= tests/wpt-runner/wpt-data
fetch-wpt-data:
ifeq ($(OS),Windows_NT)
	@$(WPT_BASH) -c 'if [ -d "$(WPT_DATA_DIR)" ] && [ -n "$$(ls -A $(WPT_DATA_DIR) 2>/dev/null)" ]; then echo "wpt-data 已存在 ($(WPT_DATA_DIR), ref=$(WPT_DATA_REF))；刷新请先 rm -rf 该目录"; else echo "fetch wpt-data $(WPT_DATA_REF) → $(WPT_DATA_DIR)"; git clone --depth=1 --branch $(WPT_DATA_REF) $(WPT_DATA_REPO) "$(WPT_DATA_DIR)"; rm -rf "$(WPT_DATA_DIR)/.git"; fi'
else
	@if [ -d "$(WPT_DATA_DIR)" ] && [ -n "$$(ls -A $(WPT_DATA_DIR) 2>/dev/null)" ]; then echo "wpt-data 已存在 ($(WPT_DATA_DIR), ref=$(WPT_DATA_REF))；刷新请先 rm -rf 该目录"; else echo "fetch wpt-data $(WPT_DATA_REF) → $(WPT_DATA_DIR)"; git clone --depth=1 --branch $(WPT_DATA_REF) $(WPT_DATA_REPO) "$(WPT_DATA_DIR)"; rm -rf "$(WPT_DATA_DIR)/.git"; fi
endif
	@$(WPT_BASH) scripts/fetch-wpt-smoke-subdirs.sh
	@$(WPT_BASH) tests/wpt-runner/scripts/sync-imported-resources.sh
	@$(WPT_BASH) tests/wpt-runner/scripts/audit-imported-font-resources.sh

fetch-wpt-html-testharness:
	bash tests/wpt-runner/scripts/fetch-html-testharness-subset.sh

# 升级 wpt-data 套件到新 tag（A2：套件随上游滚动，否则通过率无法对比）。
# 用法: make update-wpt-data REF=v2.0        升级到指定 tag
#       make update-wpt-data CHECK=1         查看远端可用 tag（只读）
update-wpt-data:
	bash scripts/update-wpt-data.sh $(if $(CHECK),--check,$(REF))

build: setup-rusty-v8 target-disk-guard
	cargo build --workspace --exclude zero-browser
	cargo build -p zero-browser

# WAYLAND_DEBUG and WINIT_UNIX_BACKEND=x11 are separate targets because they
# debug different backends and should not be combined in one run.

BROWSER_BIN = ./target/release/zero-browser
BROWSER_RUN = $(BROWSER_BIN)

ifeq ($(OS),Windows_NT)
browser-build: target-disk-guard
	powershell -NoProfile -ExecutionPolicy Bypass -File scripts\browser.ps1 -BuildOnly

browser: target-disk-guard
	powershell -NoProfile -ExecutionPolicy Bypass -File scripts\browser.ps1

# 与 browser-cpu 相同（保留别名）
browser-wpt-parity: browser-cpu

browser-cpu: target-disk-guard
	powershell -NoProfile -ExecutionPolicy Bypass -File scripts\browser-cpu.ps1
else
browser-build: setup-rusty-v8 target-disk-guard
	cargo build --release -p zero-browser
	cargo build --release -p zero-renderer -p zero-compositor -p zero-image-decoder

browser: browser-build
	RUST_BACKTRACE=1 $(BROWSER_BIN) --renderer=gpu

# 与 browser-cpu 相同（保留别名）
browser-wpt-parity: browser-cpu

browser-cpu: browser-build
	RUST_BACKTRACE=1 $(BROWSER_BIN) --wpt-parity
endif

browser-debug: browser-build
	RUST_BACKTRACE=1 $(BROWSER_BIN) --renderer=gpu

browser-debug-wayland: browser-build
	mkdir -p target
	RUST_BACKTRACE=1 WINIT_UNIX_BACKEND=wayland WAYLAND_DEBUG=1 $(BROWSER_RUN) 2>&1 | tee target/zero-browser-wayland-debug.log

browser-debug-wayland-log: browser-build
	mkdir -p target
	RUST_BACKTRACE=1 WINIT_UNIX_BACKEND=wayland WAYLAND_DEBUG=1 $(BROWSER_RUN) > target/zero-browser-wayland-debug.log 2>&1

browser-debug-x11: browser-build
	RUST_BACKTRACE=1 WAYLAND_DISPLAY= WAYLAND_SOCKET= WINIT_UNIX_BACKEND=x11 $(BROWSER_RUN)

# 真实产品窗口 smoke：CPU/scale=1 下串行运行 legacy 与 compositor 两种模式，
# 由最终 softbuffer framebuffer 写 PNG，不依赖系统截图或无障碍权限。
# 构建、两个进程链和全部断言都受 test-guard 内存门禁与 900 秒墙钟保护。
browser-compositor-smoke: target-disk-guard target/test-guard
	./target/test-guard --time-limit 900 -- sh scripts/browser-compositor-smoke.sh

# 可选真实网站 GUI 验收：打开 HTTPS 页面并依次滚动、缩放、刷新，保存四张最终
# framebuffer 截图并断言 compositor 全程健康。默认 URL 可用 GUI_SMOKE_URL 覆盖。
# 此 target 故意不接入 make test，避免网络和真实窗口成为常规单测前置条件。
browser-compositor-real-site-smoke: target-disk-guard target/test-guard
	./target/test-guard --time-limit 900 -- sh scripts/browser-compositor-real-site-smoke.sh

# ── 测试防护 (test-guard) ──────────────────────────────────────────────
# test-guard 跨平台 (macOS/Linux) 包裹测试命令，防内存型 bug（无限循环 realloc、
# CSS parser 未闭合括号死循环）触发系统级 OOM 连累 tmux session / rally 无人值守
# 流程。源码 scripts/test-guard.rs，std-only，rustc 直接编译。源码默认阈值 6/16GB/
# 1800s；本 Makefile 各入口显式覆盖为单进程 RSS>4GB 或全树>8GB 即杀（退出 124）。
# 2026-08-19 实测降至 4/8：运行阶段合法峰值单进程 0.95GB（browser 串行段）、
# 子树 2.07GB，4/8 留 4 倍余量；双 rally 流并行时最坏叠加 16GB，远离 46GB 物理内存。
# 历史教训：2026-06-28 曾因 browser>6GB（当时 --test-threads=2）升到 10/28，
# 后改串行峰值已降；阈值应随实测峰值调，勿拍脑袋。
ifeq ($(OS),Windows_NT)
MKDIR_TARGET = if not exist target mkdir target
else
MKDIR_TARGET = mkdir -p target
endif

ifeq ($(OS),Windows_NT)
target/test-guard: scripts/test-guard.rs
	@$(MKDIR_TARGET)
	rustc -O scripts/test-guard.rs -o target/test-guard.exe
else
target/test-guard: scripts/test-guard.rs
	@$(MKDIR_TARGET)
	rustc -O scripts/test-guard.rs -o target/test-guard
endif

# WPT/产品测试的 runner 必须先完成不受内存阈值限制的编译；各 target 随后只守卫运行。
.PHONY: zero-wpt-runner-release learnings-index
zero-wpt-runner-release:
	cargo build --release --bin zero-wpt-runner

# 全量测试：先无内存上限编译，再由 test-guard 包裹已编译测试运行。无人值守 /
# rally / CI 请用此 target，不要裸跑 cargo test。可调阈值：./target/test-guard --compile-first --per-proc-mem 6 --total-mem 12 -- cargo test --workspace
# 2026-08-08：纳入 QuickJS 矩阵（v8/quickjs 接口一致性保证——此前 quickjs 只在 CI，
# 本地提交门禁覆盖不到，编译/运行破坏 CI 才暴露；QuickJS_CRATES 为 CI quickjs 测试包列表）。
QUICKJS_CLIPPY_CRATES = zero-dom zero-css-parser zero-style-system zero-layout-engine zero-engine zero-canvas zero-host-runtime zero-net zero-security zero-storage zero-protocol zero-wasm-sandbox zero-page-runtime zero-render-foundation
QUICKJS_TEST_CRATES = zero-script-sandbox zero-webview zero-browser zero-renderer zero-webview-demo zero-integration-tests zero-wpt-runner
QUICKJS_TEST_CRATES_WITHOUT_BROWSER = $(filter-out zero-browser,$(QUICKJS_TEST_CRATES))
QUICKJS_TEST_CRATES_WITHOUT_BROWSER_OR_RENDERER = $(filter-out zero-browser zero-renderer,$(QUICKJS_TEST_CRATES))
ifeq ($(OS),Windows_NT)
# Windows GUI 测试共享进程级 compositor；并行执行会让测试互相关闭其子进程。
test: target-disk-guard target/test-guard
	cargo build -p zero-renderer -p zero-compositor -p zero-image-decoder
	set ZERO_NOPROXY=1&& .\target\test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test --workspace --exclude zero-browser --exclude zero-renderer
	set ZERO_NOPROXY=1&& .\target\test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-renderer --bin zero-renderer -- --test-threads=1
	set ZERO_NOPROXY=1&& .\target\test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-browser --bin zero-browser -- --test-threads=1
	cargo clippy --no-default-features --features quickjs $(addprefix -p ,$(QUICKJS_CLIPPY_CRATES)) --all-targets -- -D warnings
	set ZERO_NOPROXY=1&& .\target\test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test --no-default-features --features quickjs $(addprefix -p ,$(QUICKJS_TEST_CRATES_WITHOUT_BROWSER_OR_RENDERER))
	set ZERO_NOPROXY=1&& .\target\test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test --no-default-features --features quickjs -p zero-renderer --bin zero-renderer -- --test-threads=1
	set ZERO_NOPROXY=1&& .\target\test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test --no-default-features --features quickjs -p zero-browser -- --test-threads=1
else
test: target-disk-guard target/test-guard
	# Browser 多进程单测直接 spawn target/debug/{zero-renderer,zero-compositor}；先刷新
	# standalone binaries，避免协议结构变更后复用旧 wire schema，导致断管或 stale 帧。
	cargo build -p zero-renderer -p zero-compositor -p zero-image-decoder
	# cargo test 执行器（2026-08-09 从 nextest 换回——字体共享后评估反转）：
	# - nextest 每测试独立进程 → 每测试进程重复解析 19MB CJK 字体（~3s/进程），
	#   实测 zero-wpt-runner 45s / zero-browser 30s；cargo test 每二进制 1 进程
	#   （进程内并行），字体每二进制只付 1 次 → 同两包 3.4s / 3.4s，全量
	#   v8 阶段 47.3s（nextest 68s，-30%），且与 CI（cargo test --workspace）
	#   覆盖口径一致。历史评估（2026-08-07 nextest 1m29s vs cargo test 1m58s）
	#   未计入字体缓存与「每测试进程」的相互作用，已被推翻。
	# - adapter-only GPU 测试从 workspace 主矩阵剥离：headless probe 成功才执行，
	#   无 adapter 的主机明确跳过 capability 分支；测试运行仍由 test-guard 兜底。
	# - 并行化：QuickJS clippy（编译型）与 v8 测试并行跑——clippy 编译的是
	#   quickjs feature 组合产物（与 v8 产物不冲突），cargo 各自持锁；v8 测试
	#   （~50s）时长覆盖 clippy 编译，总时长省一个编译段。cargo test 先无约束编译，
	#   再由 test-guard 仅监管运行阶段；clippy 本身不受内存阈值限制。
	ZERO_NOPROXY=1 ./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test --workspace --exclude zero-renderer -- --skip gpu::renderer:: --skip surface::tests::test_gpu_cpu_rendering_consistency_solid_fill & test_pid=$$!; \
	cargo clippy --no-default-features --features quickjs $(addprefix -p ,$(QUICKJS_CLIPPY_CRATES)) --all-targets -- -D warnings & clippy_pid=$$!; \
	rc=0; wait $$test_pid || rc=$$?; wait $$clippy_pid || rc=$$?; exit $$rc
	ZERO_NOPROXY=1 ./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-renderer --bin zero-renderer -- --test-threads=1
	@if ZERO_NOPROXY=1 ./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 120 -- cargo test -p zero-render-foundation gpu::renderer::tests::test_gpu_renderer_headless_creation -- --exact --test-threads=1 >/dev/null 2>&1; then \
		echo "wgpu adapter available; running adapter-only GPU tests"; \
		ZERO_NOPROXY=1 ./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-render-foundation gpu::renderer:: -- --test-threads=1; \
		ZERO_NOPROXY=1 ./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-render-foundation surface::tests::test_gpu_cpu_rendering_consistency_solid_fill -- --exact --test-threads=1; \
	else \
		echo "wgpu adapter unavailable; adapter-only GPU tests skipped"; \
	fi
	# QuickJS 运行测试（v8/quickjs 接口一致性保证）
	ZERO_NOPROXY=1 ./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test --no-default-features --features quickjs $(addprefix -p ,$(QUICKJS_TEST_CRATES_WITHOUT_BROWSER_OR_RENDERER))
	ZERO_NOPROXY=1 ./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test --no-default-features --features quickjs -p zero-renderer --bin zero-renderer -- --test-threads=1
endif

# M4 HTML behavior: selected upstream forms/focus/InputEvent testharness cases.
testharness-html: fetch-wpt-html-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- ./target/release/zero-wpt-runner testharness-html

# js-dom goal M4 / DC-3：上游 dom/ testharness 通过率基线（dom/nodes 首批）。
# 用例 gitignored（fetch-dom-subset.sh 按需拉取）。filter 透传：make testharness-dom FILTER=Document-createElement。
fetch-wpt-dom:
	bash tests/wpt-runner/scripts/fetch-dom-subset.sh

# js-dom R51：TIME_LIMIT 可透传（默认 900s）。dom/ranges 等 mega-case 子目录
#（Range-mutations 族 12 用例各 30-60s）需要更长墙钟。
testharness-dom: target-disk-guard fetch-wpt-dom target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-dom $(if $(FILTER),$(FILTER),)

# js-dom goal DC-3 native 路径对照：ZW_NATIVE_DOM=1 走原生绑定路径（非默认 polyfill）。
# 用于建立 native 通过率基线，对照 R2/R3/R4 native 修复（classList/createElement/node mutation）。
testharness-dom-native: target-disk-guard fetch-wpt-dom target/test-guard zero-wpt-runner-release
	ZW_NATIVE_DOM=1 ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- ./target/release/zero-wpt-runner testharness-dom $(if $(FILTER),$(FILTER),)

# IndexedDB goal M1：上游 IndexedDB factory/global/event 首批 testharness 基线。
# `.any.js` 用例由 runner 包装为 window test；filter 按文件路径子串透传。
fetch-wpt-indexeddb:
	bash tests/wpt-runner/scripts/fetch-indexeddb-subset.sh

testharness-indexeddb: target-disk-guard fetch-wpt-indexeddb target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-indexeddb $(if $(FILTER),$(FILTER),)

# Service Worker WPT 分母：294 个 source 必须有唯一且可重建的执行 lane。
audit-wpt-service-workers-disposition:
	python3 tests/wpt-runner/scripts/audit-service-worker-disposition.py

# Service Worker M1 Tier A：仅恢复固定静态资产；runner/runtime 仍受 M0 RFC 审批门禁。
fetch-wpt-service-workers-tier-a:
	$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-tier-a:
	$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-tier-a-assets: fetch-wpt-service-workers-tier-a
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-next-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-19-m1-next-wave-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=7 WPT_CORPUS_LABEL="Service Worker next-wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-next-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-19-m1-next-wave-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=7 WPT_CORPUS_LABEL="Service Worker next-wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-next-wave-assets: fetch-wpt-service-workers-next-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-19-m1-next-wave-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=7 WPT_CORPUS_LABEL="Service Worker next-wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-static-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-19-worker-global-static-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker static wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-static-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-19-worker-global-static-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker static wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-static-wave-assets: fetch-wpt-service-workers-static-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-19-worker-global-static-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker static wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-update-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker update wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-update-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker update wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-update-wave-assets: fetch-wpt-service-workers-update-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker update wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/empty.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-import-response-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-import-response-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker import response wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-import-response-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-import-response-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker import response wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-import-response-wave-assets: fetch-wpt-service-workers-import-response-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-import-response-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker import response wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/mime-type-worker.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-import-dynamic-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-import-dynamic-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=11 WPT_CORPUS_LABEL="Service Worker import dynamic wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-import-dynamic-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-import-dynamic-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=11 WPT_CORPUS_LABEL="Service Worker import dynamic wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-import-dynamic-wave-assets: fetch-wpt-service-workers-import-dynamic-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-import-dynamic-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=11 WPT_CORPUS_LABEL="Service Worker import dynamic wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/update-worker.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-import-event-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-import-event-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=3 WPT_CORPUS_LABEL="Service Worker import event wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-import-event-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-import-event-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=3 WPT_CORPUS_LABEL="Service Worker import event wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-import-event-wave-assets: fetch-wpt-service-workers-import-event-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-import-event-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=3 WPT_CORPUS_LABEL="Service Worker import event wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/import-scripts-echo.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-module-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-module-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker module wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-module-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-module-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker module wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-module-wave-assets: fetch-wpt-service-workers-module-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-module-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker module wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/scope1/redirect.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-module-bytecheck-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-module-bytecheck-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker module bytecheck wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-module-bytecheck-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-module-bytecheck-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker module bytecheck wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-module-bytecheck-wave-assets: fetch-wpt-service-workers-module-bytecheck-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-module-bytecheck-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker module bytecheck wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/bytecheck-worker.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-module-cors-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-module-cors-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker module CORS wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-module-cors-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-module-cors-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker module CORS wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-module-cors-wave-assets: fetch-wpt-service-workers-module-cors-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-module-cors-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker module CORS wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/update-bytecheck-cors-import.https.html" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-module-registration-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-module-registration-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker module registration wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-module-registration-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-module-registration-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker module registration wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-module-registration-wave-assets: fetch-wpt-service-workers-module-registration-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-module-registration-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker module registration wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/malformed-worker.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-module-type-update-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-module-type-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=8 WPT_CORPUS_LABEL="Service Worker module type update wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-module-type-update-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-module-type-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=8 WPT_CORPUS_LABEL="Service Worker module type update wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-module-type-update-wave-assets: fetch-wpt-service-workers-module-type-update-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/service-workers/evidence/2026-08-20-m3-module-type-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=8 WPT_CORPUS_LABEL="Service Worker module type update wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/update-registration-with-type.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

testharness-service-workers-core: target-disk-guard fetch-wpt-service-workers-tier-a fetch-wpt-service-workers-next-wave fetch-wpt-service-workers-static-wave fetch-wpt-service-workers-update-wave fetch-wpt-service-workers-import-response-wave fetch-wpt-service-workers-import-dynamic-wave fetch-wpt-service-workers-import-event-wave fetch-wpt-service-workers-module-wave fetch-wpt-service-workers-module-bytecheck-wave fetch-wpt-service-workers-module-cors-wave fetch-wpt-service-workers-module-registration-wave fetch-wpt-service-workers-module-type-update-wave target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- \
		./target/release/zero-wpt-runner testharness-service-workers \
		--wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root $(if $(FILTER),$(FILTER),)

baseline-wpt-service-workers-core: target-disk-guard fetch-wpt-service-workers-tier-a fetch-wpt-service-workers-next-wave fetch-wpt-service-workers-static-wave fetch-wpt-service-workers-update-wave fetch-wpt-service-workers-import-response-wave fetch-wpt-service-workers-import-dynamic-wave fetch-wpt-service-workers-import-event-wave fetch-wpt-service-workers-module-wave fetch-wpt-service-workers-module-bytecheck-wave fetch-wpt-service-workers-module-cors-wave fetch-wpt-service-workers-module-registration-wave fetch-wpt-service-workers-module-type-update-wave target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- \
		python3 tests/wpt-runner/scripts/run-service-workers-core-baseline.py \
		--runner ./target/release/zero-wpt-runner \
		--wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root $(if $(OUTPUT),--output $(OUTPUT),)

# WPT reftest：release 构建不受内存限制，已编译 runner 的执行由 test-guard 包裹。
reftest: target-disk-guard fetch-wpt-data target/test-guard zero-wpt-runner-release
	./target/test-guard -- ./target/release/zero-wpt-runner reftest

# 上游 WPT reftest（wpt-data/，self-source 同源 ref）。test-guard 包裹（OOM 防护）。
# 全量 ~16600 案（2026-08-07 @font-face loader 缓存后）实测 ~25s，远低于
# test-guard 默认 1800s 超时；--time-limit 3600 保留作兜底（不匹配 OOM 死循环）。
# 用法: make reftest-upstream                     全量上游（快，~25s）
#       make reftest-upstream FILTER=css-tables   单目录/子串过滤（case.id.contains）
#       make reftest-upstream FILTER=css/CSS2/backgrounds
reftest-upstream: target-disk-guard fetch-wpt-data target/test-guard zero-wpt-runner-release
	./target/test-guard --time-limit 3600 -- ./target/release/zero-wpt-runner reftest-upstream $(FILTER)

# DC-14 独立 Oracle：渲染上游 WPT test 页 vs chromium oracle-shots，报告真一致率
# （chromium-Oracle pass-rate，替代 self-ref 的 ~46.5% 假通过）。oracle-shots 由
# capture-oracle-per-dir.mjs 本地抓取（gitignored，可再生）。非硬 fail 门禁（报告性）。
# 用法：make reftest-oracle                       全量（慢，~10k 案）
#       make reftest-oracle DIR=css-grid          单目录
#       make reftest-oracle DIR=css-grid ORACLE_PASS_RATIO=0.005   调严判定阈值
reftest-oracle: target-disk-guard fetch-wpt-data target/test-guard zero-wpt-runner-release
	./target/test-guard -- ./target/release/zero-wpt-runner reftest-oracle $(DIR)

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
PRODUCT_ORACLE_SCRIPT := tests/wpt-runner/scripts/product-oracle-shot.mjs
product-smoke-oracle: target-disk-guard target/test-guard
	@test -d tests/wpt-runner/scripts/node_modules/puppeteer-core || (echo "Error: puppeteer-core is missing; run 'npm ci --prefix tests/wpt-runner/scripts' first."; exit 2)
	./target/test-guard -- node $(PRODUCT_ORACLE_SCRIPT) --root apps/browser/assets --html welcome.html --out $(WELCOME_ORACLE) --width 800 --height 600

FORM_VISUAL_ROOT := .acceptance/artifacts/form-chrome-visual-parity-2026-08-13
FORM_VISUAL_ORACLE ?= $(FORM_VISUAL_ROOT)/screenshots/chrome-800x720-gray.png
FORM_VISUAL_GEOMETRY ?= $(FORM_VISUAL_ROOT)/chrome-geometry-gray.json
FORM_VISUAL_CJK_DIR ?= $(HOME)/.cache/zw-oracle-fonts/usr/share/fonts/opentype/noto
FORM_VISUAL_GPU_DIR ?= target/form-visual-browser-gpu-smoke
form-visual-smoke: target-disk-guard target/test-guard zero-wpt-runner-release
	@test -f $(FORM_VISUAL_ORACLE) || (echo "Error: missing $(FORM_VISUAL_ORACLE)"; exit 2)
	@test -f $(FORM_VISUAL_GEOMETRY) || (echo "Error: missing $(FORM_VISUAL_GEOMETRY)"; exit 2)
	@test -f $(FORM_VISUAL_CJK_DIR)/NotoSansCJK-Regular.ttc || (echo "Error: missing Noto CJK font in $(FORM_VISUAL_CJK_DIR)"; exit 2)
	ZW_CJK_FACE_INDEX=2 ZW_CJK_FONT_DIR=$(FORM_VISUAL_CJK_DIR) ./target/test-guard -- ./target/release/zero-wpt-runner product-smoke examples/forms/form-interaction-test.html --base-dir examples/forms --oracle $(FORM_VISUAL_ORACLE) --geometry-oracle $(FORM_VISUAL_GEOMETRY) --out $(FORM_VISUAL_ROOT)/screenshots/zeroweb-final.png --width 800 --height 720 --channel-diff 8 --pixel-radius 1 --max-diff 5 --max-geometry-diff 2 --struct-check --region name:10 --region note:10 --region subscribe:10 --region plan-basic:10 --region plan-pro:10 --region click:10 --region reset:10 --region submit:10 --region result:10

form-visual-browser-gpu-smoke: target-disk-guard target/test-guard
	@test -n "$(DISPLAY)" || (echo "Error: DISPLAY is required for the real browser GPU smoke"; exit 2)
	@test -f $(FORM_VISUAL_ORACLE) || (echo "Error: missing $(FORM_VISUAL_ORACLE)"; exit 2)
	@test -f $(FORM_VISUAL_CJK_DIR)/NotoSansCJK-Regular.ttc || (echo "Error: missing Noto CJK font in $(FORM_VISUAL_CJK_DIR)"; exit 2)
	cargo build --release -p zero-browser
	cargo build --release -p zero-renderer -p zero-compositor -p zero-image-decoder -p zero-wpt-runner
	rm -rf $(FORM_VISUAL_GPU_DIR)
	ZW_BROWSER_GPU_DMABUF_IMPORT=0 ZW_CJK_FACE_INDEX=2 ZW_CJK_FONT_DIR=$(FORM_VISUAL_CJK_DIR) ./target/test-guard --time-limit 150 -- ./target/release/zero-browser --renderer=gpu --scale=1 --viewport-width=800 --viewport-height=720 --gui-smoke-url=file://$(CURDIR)/examples/forms/form-interaction-test.html --gui-smoke-dir=$(FORM_VISUAL_GPU_DIR)
	./target/release/zero-wpt-runner compare-png $(FORM_VISUAL_GPU_DIR)/01-loaded-page.png $(FORM_VISUAL_ORACLE) --max-diff 5 --channel-diff 8 --pixel-radius 1

product-smoke: target-disk-guard target/test-guard zero-wpt-runner-release
	# 表单流畅度门禁：固定尺寸 value-only 输入不得重新 parse/style/layout，且每次最多发布一帧。
	./target/test-guard --time-limit 900 -- bash scripts/run-form-input-perf.sh
	@test -f $(WELCOME_ORACLE) || (echo "Error: missing $(WELCOME_ORACLE); run 'make product-smoke-oracle' and commit the generated oracle."; exit 2)
	# DC-13 desktop（800px）：欢迎页 vs chromium Oracle diff≤20% + 结构门。--struct-check 含
	# sibling-overlap + collapsed-container + **text-concatenation**（R109 inline-ownership 守，
	# DC-13 line 325「sibling card/link/shortcut 文本不串联」）。计数断言覆盖 line 324 桌面须验证的
	# 四个 feature card（card:4）+ 六个快捷键（shortcut:6）+ 四个快速访问（link-tile:4）；
	# 行数断言守标题不拆行（title:1）+ tagline 2 行（tagline:2）。
	./target/test-guard -- ./target/release/zero-wpt-runner product-smoke $(WELCOME_HTML) --oracle $(WELCOME_ORACLE) --max-diff $(or $(MAX_DIFF),20) --struct-check --expect-class card:4 --expect-class shortcut:6 --expect-class link-tile:4 --expect-class footer:1 --expect-lines title:1 --expect-lines tagline:2
	# DC-13 desktop morning（800px）：article 结构 + 三个 tag badge（item-tag:3，line 326）+
	# pre/code 块在位（lang-bash:1，line 326 pre/code 独立背景换行）。struct-check 含 concat 守
	# nav/title/date/tag badge 不串联 + 正文段落不压一行 + table 不塌缩。morning 故意缺 cc_unavailable
	# 图测 alt 回退 → 不启用 --check-img-visibility（否则误报）。
	./target/test-guard -- ./target/release/zero-wpt-runner product-smoke apps/browser/assets/morning-work/article.html --base-dir apps/browser/assets/morning-work --struct-check --expect-class article:1 --expect-class item-tag:3 --expect-class lang-bash:1
	# DC-13 goal line 322：窄屏 viewport（375px）结构门——窄宽逼长段落换行，暴露桌面宽不触发的
	# 重叠（R1498 morning @375 `<p>` 长高重叠后续 `<table>` 即此门抓到）。无 oracle（窄屏 oracle
	# 未抓），仅 struct-check 退码 3。--expect-class article:1 守 R1499 labels 修复（disqus
	# loadDisqus() appendChild 致 mutated_html ≠ 原 html，labels 须从 mutated_html 建才匹配 layout）。
	./target/test-guard -- ./target/release/zero-wpt-runner product-smoke apps/browser/assets/morning-work/article.html --base-dir apps/browser/assets/morning-work --width 375 --struct-check --expect-class article:1
	# DC-13 goal line 324「至少覆盖桌面和窄屏两个 viewport」：welcome 窄屏（375/320）结构门。
	# welcome 无 width 媒体查询，grids 保持 2 列，card:4 在窄宽仍成立；标题/tagline 在窄宽会合法
	# 换行故不强行断言行数。struct-check 含 text-concatenation 守窄宽下卡片/链接文本不串联。
	./target/test-guard -- ./target/release/zero-wpt-runner product-smoke $(WELCOME_HTML) --width 375 --struct-check --expect-class card:4
	./target/test-guard -- ./target/release/zero-wpt-runner product-smoke $(WELCOME_HTML) --width 320 --struct-check --expect-class card:4
	# DC-13 最窄 viewport（320px）结构门——守 R1502（split-gate article/disqus Flex-兄弟位移）+
	# R1503（sub-pixel sliver 高度过滤）。@320 比 @375 更逼换行，曾暴露 article/disqus 32400px² 重叠。
	./target/test-guard -- ./target/release/zero-wpt-runner product-smoke apps/browser/assets/morning-work/article.html --base-dir apps/browser/assets/morning-work --width 320 --struct-check --expect-class article:1

# 测试资产化（P1）：把单个上游 WPT reftest 用例导入常驻断言集。
# 文件本体进入 wpt-data/（独立 repo），条目追加到 imported-tests.txt 账本，
# manifest 重新生成。每次渲染兼容性修复都应附带导入对应用例（见
# docs/goal/rendering-compat.md DC-7「测试资产化」）。
# 用法: make import-wpt TEST=css/CSS2/text/text-align-001.xht REF=css/CSS2/text/text-align-001-ref.xht NOTE="R21xx 修复"
#       make import-wpt ... EXTRA="--resource css/path/font.ttf"
import-wpt: fetch-wpt-data
	bash tests/wpt-runner/scripts/import-wpt-reftests.sh --add $(TEST) $(REF) $(if $(NOTE),--note "$(NOTE)") $(EXTRA)

audit-imported-font-resources:
	bash tests/wpt-runner/scripts/audit-imported-font-resources.sh

# WPT 趋势基线（P2）：跑上游 reftest 全量，把绝对数追加到
# docs/goal/rendering-compat/evidence/wpt-trends/trend.csv（test-guard 包裹）。
# oracle 变体记录 DC-14 credible pass（需先 make capture-oracle 生成 oracle-shots）。
# 用法: make reftest-trend [NOTE="R21xx 修复后"]
#       make reftest-trend-oracle [NOTE="..."]
reftest-trend: target-disk-guard fetch-wpt-data target/test-guard
	./target/test-guard -- bash scripts/record-wpt-trend.sh $(if $(NOTE),--note "$(NOTE)")

reftest-trend-oracle: target-disk-guard fetch-wpt-data target/test-guard
	./target/test-guard -- bash scripts/record-wpt-trend.sh --oracle $(if $(NOTE),--note "$(NOTE)")

# Reftest smoke 分层门禁（B2）：跑 reftest-smoke.txt 清单（已知通过的
# 代表性 case，秒级），用作 PR CI 快门禁；全量留给 reftest / reftest-trend。
# 清单填充：从全量通过结果中挑代表性 case 写入 tests/wpt-runner/reftest-smoke.txt。
reftest-smoke: target-disk-guard fetch-wpt-data target/test-guard
	./target/test-guard -- bash scripts/run-reftest-smoke.sh

# 布局树 dump golden 回归（B1/P3）：渲染测试页 → dump 布局树 → 与 golden 对比。
# golden 存 tests/wpt-runner/layout-golden/（提交进 git，测试资产化）。
# 用法: make layout-golden [FILTER=css/CSS2/backgrounds]   对比（diff 退出 1）
#       make layout-golden-update [FILTER=...]             生成/更新 golden
#       （新用例先 --update 生成基线并提交，此后作为布局回归常驻断言）
layout-golden: target-disk-guard fetch-wpt-data target/test-guard
	./target/test-guard -- bash scripts/run-layout-golden.sh $(FILTER)

layout-golden-update: target-disk-guard fetch-wpt-data target/test-guard
	./target/test-guard -- bash scripts/run-layout-golden.sh --update $(FILTER)

# 月度工程报告（P6/C2）：从 git 历史 + WPT 趋势自动生成 docs/monthly/YYYY-MM.md。
# 用法: make monthly-report [MONTH=2026-07]（默认上月）
monthly-report:
	bash scripts/generate-monthly-report.sh $(MONTH)

# 性能门禁体系（docs/specs/performance-and-resource-budget.md，2026-08-08 落地）：
#   make bench            全量测量（criterion 微基准 + 页面级首屏 + 峰值 RSS）
#   make bench-gate       测量 + 门禁比较（本地 rally 轮次门禁；退出码 0/1/2）
#   make bench-capture    测量 + 记录基线（JUSTIFICATION=... 必填；收紧优先）
#   make bench-trend      测量 + 记录趋势（NOTE=... 可加备注；weekly CI 用 --auto-tighten）
# 全部经 test-guard 包裹（OOM/超时保护）。首次使用顺序：bench-gate（全 NEW/PASS）
# → bench-capture JUSTIFICATION="初始基线" → bench-gate（真比较）。
bench: target-disk-guard target/test-guard
	./target/test-guard -- bash scripts/bench-report.sh

bench-gate: target-disk-guard target/test-guard
	./target/test-guard -- bash scripts/bench-report.sh && bash scripts/perf-gate.sh

bench-capture: target-disk-guard target/test-guard
	@test -n "$(JUSTIFICATION)" || (echo "bench-capture: JUSTIFICATION=... 必填（基线变更须有理由）"; exit 2)
	./target/test-guard -- bash scripts/bench-report.sh && bash scripts/record-bench-baseline.sh --justification "$(JUSTIFICATION)"

bench-trend: target-disk-guard target/test-guard
	./target/test-guard -- bash scripts/bench-report.sh && bash scripts/record-bench-trend.sh $(if $(NOTE),--note "$(NOTE)")

# Legacy Static Web smoke（DC-13，goal rendering-compat.md line 316）：跑 20 页
# HTML 3.2/4 + CSS1/2 静态 fixture，每页 chromium oracle vs ZeroWeb CPU diff%。
# ★ trend-only（退出 0）——diff 全归因字体墙（fontdue 行度量 vs chromium NotoSansCJK
# 垂直漂移 + AA，R633 多会话 plateau），非回归；像素阈值作趋势指标，不替代 WPT/DC-14
# 达标口径（goal line 318）。新增 fixture 写入 evidence/product-static/legacy-html/。
# 用法：make product-smoke-legacy
LEGACY_DIR := docs/goal/rendering-compat/evidence/product-static/legacy-html
product-smoke-legacy: target-disk-guard target/test-guard
	bash $(LEGACY_DIR)/run-all.sh

# 从 docs/learnings/*/*.md 的 frontmatter 重建 docs/learnings/INDEX.md（生成物勿手改）
learnings-index:
	python3 scripts/gen-learnings-index.py

# Android M0: Debug targets contain the local emulator ABI; Release remains arm64-only.
ifeq ($(OS),Windows_NT)
android-preflight:
	powershell -NoProfile -ExecutionPolicy Bypass -File scripts\android\preflight.ps1

android-apk: android-preflight target-disk-guard
	cd apps\android-browser && gradlew.bat --no-daemon :app:assembleEmulatorDebug

android-release-apk: android-preflight target-disk-guard
	cd apps\android-browser && gradlew.bat --no-daemon :app:assembleArm64Release

android-wsl-renderer-apk: android-preflight target-disk-guard
	cd apps\android-browser && gradlew.bat --no-daemon -PuseWslRenderer :app:assembleEmulatorDebug

android-wsl-renderer-install-smoke: android-wsl-renderer-apk
	powershell -NoProfile -ExecutionPolicy Bypass -File scripts\android\install-smoke.ps1 -ApkPath apps\android-browser\app\build\outputs\apk\emulator\debug\app-emulator-debug.apk -RequireRendererLinked

android-install-smoke: android-apk
	powershell -NoProfile -ExecutionPolicy Bypass -File scripts\android\install-smoke.ps1 -ApkPath apps\android-browser\app\build\outputs\apk\emulator\debug\app-emulator-debug.apk
else
android-preflight:
	@test -n "$$ANDROID_HOME" || (echo "ANDROID_HOME must be set"; exit 2)
	@test -d "$$ANDROID_HOME/platforms/android-36" || (echo "Android SDK platform 36 is required"; exit 2)
	@rustup target list --installed | grep -qx aarch64-linux-android
	@rustup target list --installed | grep -qx x86_64-linux-android
	@command -v cargo-ndk >/dev/null

android-apk: android-preflight target-disk-guard
	cd apps/android-browser && ./gradlew --no-daemon :app:assembleEmulatorDebug

android-release-apk: android-preflight target-disk-guard
	cd apps/android-browser && ./gradlew --no-daemon :app:assembleArm64Release

android-install-smoke: android-apk
	@echo "android-install-smoke is implemented by the Windows local-emulator script in M0"
endif
