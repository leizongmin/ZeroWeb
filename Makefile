.PHONY: setup-rusty-v8 fetch-wpt-quirks fetch-wpt-domparsing fetch-wpt-domxpath fetch-wpt-subresource-integrity fetch-wpt-cors fetch-wpt-import-maps fetch-wpt-referrer-policy fetch-wpt-input-events fetch-wpt-preload fetch-wpt-wai-aria fetch-wpt-storage fetch-wpt-data fetch-wpt-html-testharness fetch-wpt-uievents testharness-uievents fetch-wpt-touch-events testharness-touch-events fetch-wpt-webmessaging testharness-webmessaging fetch-wpt-svg testharness-svg update-wpt-data build browser-build browser browser-cpu browser-wpt-parity browser-debug browser-debug-wayland browser-debug-wayland-log browser-compositor-smoke browser-compositor-real-site-smoke test testharness-html reftest reftest-oracle capture-oracle product-smoke-oracle product-smoke form-visual-smoke form-visual-browser-gpu-smoke product-smoke-legacy import-wpt audit-imported-font-resources reftest-trend reftest-trend-oracle reftest-smoke layout-golden layout-golden-update monthly-report bench bench-gate bench-capture bench-trend fetch-wpt-dom testharness-dom testharness-dom-native fetch-wpt-selection testharness-selection fetch-wpt-keyboard testharness-keyboard fetch-wpt-indexeddb testharness-indexeddb fetch-wpt-media testharness-media fetch-wpt-webaudio testharness-webaudio fetch-wpt-cache-storage audit-wpt-cache-storage testharness-cache-storage baseline-wpt-cache-storage fetch-wpt-fs testharness-fs fetch-wpt-wasm testharness-wasm fetch-wpt-wasm-core testharness-wasm-core fetch-wpt-observers testharness-intersection-observer testharness-resize-observer fetch-wpt-net-api testharness-net-api fetch-wpt-encoding testharness-encoding fetch-wpt-encoding-legacy-mb testharness-encoding-legacy-mb fetch-wpt-encoding-compat-v2 testharness-form-url-encoding fetch-wpt-html-syntax testharness-html-syntax fetch-wpt-navigation testharness-navigation fetch-wpt-page-state testharness-page-state fetch-wpt-dnd testharness-dnd fetch-wpt-pointerlock testharness-pointerlock fetch-wpt-clipboard-apis testharness-clipboard-apis fetch-wpt-fullscreen testharness-fullscreen fetch-wpt-security-csp testharness-csp testharness-mixed-content testharness-secure-contexts fetch-wpt-timing-animation fetch-wpt-webstorage testharness-webstorage fetch-wpt-compression testharness-compression fetch-wpt-webcrypto testharness-webcrypto fetch-wpt-html-semantics testharness-html-semantics fetch-wpt-cookies testharness-cookies fetch-wpt-fileapi testharness-fileapi fetch-wpt-performance-api testharness-resource-timing testharness-navigation-timing testharness-event-timing testharness-longtask-timing testharness-hr-time testharness-performance-timeline testharness-user-timing testharness-web-animations fetch-wpt-web-components testharness-web-components fetch-wpt-shadow-render-pool testharness-shadow-dom audit-wpt-service-workers-disposition fetch-wpt-service-workers-tier-a audit-wpt-service-workers-tier-a test-wpt-service-workers-tier-a-assets fetch-wpt-service-workers-next-wave audit-wpt-service-workers-next-wave test-wpt-service-workers-next-wave-assets fetch-wpt-service-workers-message-lifecycle-wave audit-wpt-service-workers-message-lifecycle-wave test-wpt-service-workers-message-lifecycle-wave-assets fetch-wpt-service-workers-message-event-ports-wave audit-wpt-service-workers-message-event-ports-wave test-wpt-service-workers-message-event-ports-wave-assets fetch-wpt-service-workers-messageerror-event-wave audit-wpt-service-workers-messageerror-event-wave test-wpt-service-workers-messageerror-event-wave-assets fetch-wpt-service-workers-controller-on-load-wave audit-wpt-service-workers-controller-on-load-wave test-wpt-service-workers-controller-on-load-wave-assets fetch-wpt-service-workers-controller-on-disconnect-wave audit-wpt-service-workers-controller-on-disconnect-wave test-wpt-service-workers-controller-on-disconnect-wave-assets fetch-wpt-service-workers-extendable-event-waituntil-wave audit-wpt-service-workers-extendable-event-waituntil-wave test-wpt-service-workers-extendable-event-waituntil-wave-assets fetch-wpt-service-workers-onactivate-script-error-wave audit-wpt-service-workers-onactivate-script-error-wave test-wpt-service-workers-onactivate-script-error-wave-assets fetch-wpt-service-workers-oninstall-script-error-wave audit-wpt-service-workers-oninstall-script-error-wave test-wpt-service-workers-oninstall-script-error-wave-assets fetch-wpt-service-workers-getregistration-wave audit-wpt-service-workers-getregistration-wave test-wpt-service-workers-getregistration-wave-assets fetch-wpt-service-workers-registration-iframe-wave audit-wpt-service-workers-registration-iframe-wave test-wpt-service-workers-registration-iframe-wave-assets fetch-wpt-service-workers-installing-wave audit-wpt-service-workers-installing-wave test-wpt-service-workers-installing-wave-assets fetch-wpt-service-workers-waiting-wave audit-wpt-service-workers-waiting-wave test-wpt-service-workers-waiting-wave-assets fetch-wpt-service-workers-extendable-message-event-wave audit-wpt-service-workers-extendable-message-event-wave test-wpt-service-workers-extendable-message-event-wave-assets fetch-wpt-service-workers-static-wave audit-wpt-service-workers-static-wave test-wpt-service-workers-static-wave-assets fetch-wpt-service-workers-worker-secure-context-wave audit-wpt-service-workers-worker-secure-context-wave test-wpt-service-workers-worker-secure-context-wave-assets fetch-wpt-service-workers-install-event-type-wave audit-wpt-service-workers-install-event-type-wave test-wpt-service-workers-install-event-type-wave-assets fetch-wpt-service-workers-worker-close-wave audit-wpt-service-workers-worker-close-wave test-wpt-service-workers-worker-close-wave-assets fetch-wpt-service-workers-worker-interface-wave audit-wpt-service-workers-worker-interface-wave test-wpt-service-workers-worker-interface-wave-assets fetch-wpt-service-workers-fetch-event-historical-wave audit-wpt-service-workers-fetch-event-historical-wave test-wpt-service-workers-fetch-event-historical-wave-assets fetch-wpt-service-workers-update-wave audit-wpt-service-workers-update-wave test-wpt-service-workers-update-wave-assets fetch-wpt-service-workers-module-wave audit-wpt-service-workers-module-wave test-wpt-service-workers-module-wave-assets fetch-wpt-service-workers-module-bytecheck-wave audit-wpt-service-workers-module-bytecheck-wave test-wpt-service-workers-module-bytecheck-wave-assets fetch-wpt-service-workers-module-cors-wave audit-wpt-service-workers-module-cors-wave test-wpt-service-workers-module-cors-wave-assets fetch-wpt-service-workers-module-registration-wave audit-wpt-service-workers-module-registration-wave test-wpt-service-workers-module-registration-wave-assets fetch-wpt-service-workers-module-type-update-wave audit-wpt-service-workers-module-type-update-wave test-wpt-service-workers-module-type-update-wave-assets fetch-wpt-service-workers-module-request-metadata-wave audit-wpt-service-workers-module-request-metadata-wave test-wpt-service-workers-module-request-metadata-wave-assets fetch-wpt-service-workers-update-via-cache-matrix-wave audit-wpt-service-workers-update-via-cache-matrix-wave test-wpt-service-workers-update-via-cache-matrix-wave-assets fetch-wpt-service-workers-dynamic-import-update-wave audit-wpt-service-workers-dynamic-import-update-wave test-wpt-service-workers-dynamic-import-update-wave-assets fetch-wpt-service-workers-update-failure-wave audit-wpt-service-workers-update-failure-wave test-wpt-service-workers-update-failure-wave-assets fetch-wpt-service-workers-multiple-update-wave audit-wpt-service-workers-multiple-update-wave test-wpt-service-workers-multiple-update-wave-assets testharness-service-workers-core baseline-wpt-service-workers-core target-disk-guard target/test-guard android-preflight android-apk android-release-apk android-renderer-apk android-wsl-renderer-apk android-wsl-renderer-install-smoke android-install-smoke android-chaos-smoke fetch-wpt-css-variables testharness-css-variables testharness-css-variables-one fetch-wpt-css-testharness fetch-wpt-css-testharness-a fetch-wpt-css-testharness-b fetch-wpt-css-testharness-c fetch-wpt-css-testharness-v2-a fetch-wpt-css-testharness-v2-b fetch-wpt-css-testharness-v2-c testharness-css-values testharness-css-values-one testharness-selectors testharness-selectors-one testharness-css-color testharness-css-color-one testharness-css-align testharness-css-align-one testharness-css-sizing testharness-css-sizing-one testharness-css-typed-om testharness-css-typed-om-one testharness-css-fonts testharness-css-fonts-one testharness-css-shapes testharness-css-shapes-one testharness-css-anchor-position testharness-css-anchor-position-one testharness-css-masking testharness-css-masking-one testharness-css-backgrounds testharness-css-backgrounds-one testharness-css-transforms testharness-css-transforms-one testharness-css-motion testharness-css-motion-one testharness-css-grid testharness-css-grid-one testharness-css-text testharness-css-text-one testharness-cssom testharness-cssom-one testharness-css-images testharness-css-images-one testharness-filter-effects testharness-filter-effects-one testharness-css-ui testharness-css-ui-one testharness-css-borders testharness-css-borders-one testharness-css-logical testharness-css-logical-one testharness-css-text-decor testharness-css-text-decor-one testharness-css-box testharness-css-box-one testharness-css-pseudo testharness-css-pseudo-one testharness-css-display testharness-css-display-one testharness-css-mixins testharness-css-mixins-one testharness-css-syntax testharness-css-syntax-one testharness-fill-stroke testharness-fill-stroke-one testharness-css-shadow testharness-css-shadow-one testharness-css-size-adjust testharness-css-size-adjust-one testharness-css-content testharness-css-content-one testharness-css-will-change testharness-css-will-change-one testharness-css-color-adjust testharness-css-color-adjust-one testharness-css-easing testharness-css-easing-one testharness-compositing testharness-compositing-one testharness-css-counter-styles testharness-css-counter-styles-one testharness-css-nesting testharness-css-nesting-one testharness-css-color-hdr testharness-css-color-hdr-one testharness-css-env testharness-css-env-one testharness-css-conditional testharness-css-conditional-one testharness-mediaqueries testharness-mediaqueries-one testharness-css-cascade testharness-css-cascade-one testharness-css2 testharness-css2-one testharness-css-forms testharness-css-forms-one testharness-css-link-params testharness-css-link-params-one testharness-css-font-loading testharness-css-font-loading-one testharness-css-gaps testharness-css-gaps-one testharness-css-flexbox testharness-css-flexbox-one testharness-css-multicol testharness-css-multicol-one testharness-css-position testharness-css-position-one testharness-css-overflow testharness-css-overflow-one testharness-css-tables testharness-css-tables-one testharness-css-lists testharness-css-lists-one testharness-css-inline testharness-css-inline-one testharness-css-scroll-snap testharness-css-scroll-snap-one testharness-css-viewport testharness-css-viewport-one testharness-css-break testharness-css-break-one testharness-css-contain testharness-css-contain-one testharness-css-writing-modes fetch-wpt-css-testharness-v3 testharness-css-transitions testharness-css-animations testharness-cssom-view testharness-css-view-transitions testharness-geometry testharness-css-properties-values-api testharness-css-highlight-api testharness-css-page testharness-css-scroll-anchoring testharness-css-scrollbars testharness-css-overscroll-behavior testharness-css-navigation testharness-css-parser-api testharness-css-writing-modes-one fetch-wpt-editing testharness-editing-run testharness-editing-other testharness-editing-run-one testharness-editing-other-one
.PHONY: dev-guard guarded-test guarded-clippy guarded-browser

# Windows 的 make recipe 可能落到 cmd.exe（本机）或 Git Bash（GitHub Actions runner）——
# 统一显式走 Git Bash，避免 cmd 语法在 bash 下解析失败（2026-08-16 CI 实测）。
# 定义在前供所有 bash 脚本入口（target-disk-guard / fetch-wpt-data 等）共用。
ifeq ($(OS),Windows_NT)
WPT_BASH ?= "C:/Program Files/Git/bin/bash.exe"
else
WPT_BASH ?= bash
endif

# Linux 开发兜底：把重型开发命令放进单独 systemd scope，避免裸 cargo/clippy/build
# 失控时连带杀掉外层 tmux server。默认 24G，双 clone 并行时给宿主保留足够余量。
ZW_DEV_MEMORY_MAX ?= 24G
ZW_DEV_SWAP_MAX ?= 0
ZW_DEV_GUARD_UNIT ?= zeroweb-dev-guard-$(notdir $(CURDIR))
SYSTEMD_RUN ?= systemd-run
DEV_SHELL ?= bash

ifeq ($(OS),Windows_NT)
dev-guard guarded-test guarded-clippy guarded-browser:
	@echo "Error: systemd-run guarded targets are only available on Linux"; exit 2
else
dev-guard:
	$(SYSTEMD_RUN) --user --scope --unit=$(ZW_DEV_GUARD_UNIT) \
		-p MemoryMax=$(ZW_DEV_MEMORY_MAX) -p MemorySwapMax=$(ZW_DEV_SWAP_MAX) \
		$(DEV_SHELL)

guarded-test:
	$(SYSTEMD_RUN) --user --scope --unit=$(ZW_DEV_GUARD_UNIT)-test \
		-p MemoryMax=$(ZW_DEV_MEMORY_MAX) -p MemorySwapMax=$(ZW_DEV_SWAP_MAX) \
		make test

guarded-clippy:
	$(SYSTEMD_RUN) --user --scope --unit=$(ZW_DEV_GUARD_UNIT)-clippy \
		-p MemoryMax=$(ZW_DEV_MEMORY_MAX) -p MemorySwapMax=$(ZW_DEV_SWAP_MAX) \
		cargo clippy --workspace --all-targets -- -D warnings

guarded-browser:
	$(SYSTEMD_RUN) --user --scope --unit=$(ZW_DEV_GUARD_UNIT)-browser \
		-p MemoryMax=$(ZW_DEV_MEMORY_MAX) -p MemorySwapMax=$(ZW_DEV_SWAP_MAX) \
		make browser
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
# reftest / reftest-oracle 会自动前置触发；以 reftest-manifest.json 为完整套件
# 标志（不能以目录非空判断——wpt-data 内有少量被 git 跟踪的资产文件，
# fresh checkout 后目录非空但套件缺失，会误跳 clone），刷新需先 rm -rf。
# rm -rf 会连带删掉那些 tracked 资产（working tree 出现 unstaged deletion，
# 曾致 CI 回写步骤 git pull --rebase 被拒、测量丢失），clone 后用
# git checkout -- 恢复。
WPT_DATA_REPO ?= https://github.com/leizongmin/zeroweb-wpt-data.git
WPT_DATA_REF  ?= v1.10
WPT_DATA_DIR  ?= tests/wpt-runner/wpt-data
fetch-wpt-data:
ifeq ($(OS),Windows_NT)
	@$(WPT_BASH) -c 'if [ -f "$(WPT_DATA_DIR)/reftest-manifest.json" ]; then echo "wpt-data 已存在 ($(WPT_DATA_DIR), ref=$(WPT_DATA_REF))；刷新请先 rm -rf 该目录"; else echo "fetch wpt-data $(WPT_DATA_REF) → $(WPT_DATA_DIR)"; rm -rf "$(WPT_DATA_DIR)"; git clone --depth=1 --branch $(WPT_DATA_REF) $(WPT_DATA_REPO) "$(WPT_DATA_DIR)"; rm -rf "$(WPT_DATA_DIR)/.git"; git checkout -- "$(WPT_DATA_DIR)" 2>/dev/null || true; fi'
else
	@if [ -f "$(WPT_DATA_DIR)/reftest-manifest.json" ]; then echo "wpt-data 已存在 ($(WPT_DATA_DIR), ref=$(WPT_DATA_REF))；刷新请先 rm -rf 该目录"; else echo "fetch wpt-data $(WPT_DATA_REF) → $(WPT_DATA_DIR)"; rm -rf "$(WPT_DATA_DIR)"; git clone --depth=1 --branch $(WPT_DATA_REF) $(WPT_DATA_REPO) "$(WPT_DATA_DIR)"; rm -rf "$(WPT_DATA_DIR)/.git"; git checkout -- "$(WPT_DATA_DIR)" 2>/dev/null || true; fi
endif
	@$(WPT_BASH) scripts/fetch-wpt-smoke-subdirs.sh
	@$(WPT_BASH) tests/wpt-runner/scripts/sync-imported-resources.sh
	@$(WPT_BASH) tests/wpt-runner/scripts/audit-imported-font-resources.sh

fetch-wpt-html-testharness:
	bash tests/wpt-runner/scripts/fetch-html-testharness-subset.sh

fetch-wpt-css-variables:
	bash tests/wpt-runner/scripts/fetch-css-variables-subset.sh

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
	cargo build --release -p zero-renderer -p zero-compositor -p zero-image-decoder -p zero-media-host

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
	# T58（2026-10-17）：fmt 门禁前置快速失败（格式漂移不等全量测试尾部才暴露）。
	cargo fmt --all -- --check
	cargo build -p zero-renderer -p zero-compositor -p zero-image-decoder -p zero-media-host
	set ZERO_NOPROXY=1&& .\target\test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test --workspace --exclude zero-browser --exclude zero-renderer
	set ZERO_NOPROXY=1&& .\target\test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-renderer --bin zero-renderer -- --test-threads=1
	set ZERO_NOPROXY=1&& .\target\test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-browser --bin zero-browser -- --test-threads=1
	# T58（2026-10-17）：v8 浏览器腿补跑集成套件（apps/browser/tests/ cdp_* 三件）——
	# 此前只跑 --bin zero-browser，集成套件失败不可见（T55 预存失败借此隐藏两个月）。
	# cdp 钉为 headless 真进程面（自带本地 HTTP 测试页，无需 display），串行避免
	# 浏览器进程互相挤占；test-guard 运行段内存/墙钟与 bin 腿同口径。
	set ZERO_NOPROXY=1&& .\target\test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-browser --tests -- --test-threads=1
	cargo clippy --no-default-features --features quickjs $(addprefix -p ,$(QUICKJS_CLIPPY_CRATES)) --all-targets -- -D warnings
	set ZERO_NOPROXY=1&& .\target\test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test --no-default-features --features quickjs $(addprefix -p ,$(QUICKJS_TEST_CRATES_WITHOUT_BROWSER_OR_RENDERER))
	set ZERO_NOPROXY=1&& .\target\test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test --no-default-features --features quickjs -p zero-renderer --bin zero-renderer -- --test-threads=1
	set ZERO_NOPROXY=1&& .\target\test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test --no-default-features --features quickjs -p zero-browser -- --test-threads=1
else
test: target-disk-guard target/test-guard
	# T58（2026-10-17）：fmt 门禁前置快速失败（格式漂移不等全量测试尾部才暴露）。
	cargo fmt --all -- --check
	# Browser 多进程单测直接 spawn target/debug/{zero-renderer,zero-compositor}；先刷新
	# standalone binaries，避免协议结构变更后复用旧 wire schema，导致断管或 stale 帧。
	cargo build -p zero-renderer -p zero-compositor -p zero-image-decoder -p zero-media-host
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
	# R385（js-dom）：`window_surface_present_smoke` 需要可用 X display（winit event
	# loop），无头环境（CI / rally 无人值守）XOpenDisplayFailed 恒败——多轮记录的
	# 唯一非语义红灯。ZW_GUARD_RUNNER_PREFIX="xvfb-run -a"（test-guard 支持）为
	# zero-browser 测试 artifact 提供 Xvfb display，实测全套 411P（~0.2s/用例，
	# 无额外 wgpu adapter 需求）；有 display 的环境该前缀同样工作（xvfb-run
	# 独立起 display，不影响结果语义）。
	# R5000 片 c 栈边际记账：libtest 线程栈 8MiB 默认值对 tests::pipeline 深嵌套
	# 递归帧**边际翻转**（zero-dom rmeta 重编译即抽签——纯注释 no-op 也触发；
	# 2026-10-02 实证）。临时基建加固 = 提测试线程栈 32MiB；真修（style/layout
	# 递归帧瘦身）归渲染流域碰头账（master.md 碰头条款）。
	# R5389 workspace 腿 per-proc 阈值校准（webcrypto-compat-v3 D1 切片 A/B 实测
	# 2026-11-02）：zero-wpt-runner lib 测试二进制（并发引擎实例聚合 RSS）在**无
	# 新增改动的 HEAD 代码**峰值 4,585,564 KB（4.37G）已超 4G 门限——webcrypto
	# 三域切片前 workspace 腿即结构性边际（css-variables 4.22–4.32G、
	# html-semantics 4.31G 超限先例同因），本切片二进制同口径实测 4,103,236 KB
	# （3.91G，同二进制 make test 态 4.2–4.3G 波动）。校准 = workspace 腿
	# per-proc-mem 4→6 / total-mem 8→12（仅此腿；renderer/browser 各腿维持），
	# 依据 /usr/bin/time -v 同守卫环境三次实测；真修（runner 测试并发工作集
	# 瘦身）归 runner 通道碰头账（master.md 碰撞管理记录 R5389 条）。
	# s280（2026-11-04）真修落地：mini_harness_stateful 模拟 harness 的 complete
	# 空操作 + phase 恒 0 击穿 terminal 判据三臂——14 个调用钉泵满 CASE_TIMEOUT
	# （10s/钉），单钉堆增长 300-930MB，16 并流聚合 runner bin 二进制峰 4.02G
	# （engine 群 85 钉单独 4.44G；非 engine 群仅 0.78G）。修后逐钉普查：14 钉
	# 10.4s→0.51s / 300-930MB→148-185MB；runner bin 全量 4G 常态阈值 RC=0
	# （286P/0F/76I 零位移），峰 4,212,376→1,235,072 KB、墙钟 17.2→8.4s。
	# workspace 腿回归 per-proc-mem 4 / total-mem 8 常态口径（R5389 校准解除）。
	ZERO_NOPROXY=1 RUST_MIN_STACK=33554432 ./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test --workspace --exclude zero-browser --exclude zero-renderer -- --skip gpu::renderer:: --skip surface::tests::test_gpu_cpu_rendering_consistency_solid_fill & test_pid=$$!; \
	cargo clippy --no-default-features --features quickjs $(addprefix -p ,$(QUICKJS_CLIPPY_CRATES)) --all-targets -- -D warnings & clippy_pid=$$!; \
	rc=0; wait $$test_pid || rc=$$?; wait $$clippy_pid || rc=$$?; exit $$rc
	ZERO_NOPROXY=1 ./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-renderer --bin zero-renderer -- --test-threads=1
	# slice28（RP-3，2026-10-04）：renderer lib 专项腿。workspace 腿 --exclude
	# zero-renderer、renderer 腿仅 --bin——renderer lib（js_worker 快照/协议单测，
	# slice27/28 named access 钉所在）此前 make test 结构性失明（CI nextest 兜底）。
	# 锚算术基数变化：本腿加入后 make test 总计 = 原 19,614P + 本腿 P 数
	#（2026-10-04 时点本腿 202P = 3bc44bf62 基线 201P + slice28 新钉 1P；随钉数自然增长）。
	ZERO_NOPROXY=1 ./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-renderer --lib -- --test-threads=1
	ZERO_NOPROXY=1 ZW_GUARD_RUNNER_PREFIX="xvfb-run -a" ./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-browser --bin zero-browser -- --test-threads=1
	# T58（2026-10-17）：v8 浏览器腿补跑集成套件（apps/browser/tests/ cdp_* 三件）——
	# 此前只跑 --bin zero-browser，集成套件失败不可见（T55 预存失败借此隐藏两个月）。
	# cdp 钉为 headless 真进程面（自带本地 HTTP 测试页，无需 display），串行避免
	# 浏览器进程互相挤占；test-guard 运行段内存/墙钟与 bin 腿同口径。
	# R5207（2026-10-20）：cargo `--tests` 含 bin test harness——本腿会重跑 666 条 bin
	# 单测，无 R385 的 Xvfb 前缀时 headless 环境 winit 腿恒败（XNotSupported，
	# window_surface_present_smoke / paste_and_go 两处非语义红灯）。
	ZERO_NOPROXY=1 ZW_GUARD_RUNNER_PREFIX="xvfb-run -a" ./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-browser --tests -- --test-threads=1
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
	# slice35（RP-3 调用点钉收口，2026-10-06）：renderer lib 专项腿补 quickjs feature
	# 组合——page_scripts/js_worker（slice33/34 残影修复面钉所在）均属 lib target，
	# quickjs 腿此前只有 --bin 过滤，renderer 钉在 quickjs-only 构建（仓库认可的
	# fallback 面）下从未执行。镜像上方 v8 `--lib` 腿（slice28）。
	ZERO_NOPROXY=1 ./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test --no-default-features --features quickjs -p zero-renderer --lib -- --test-threads=1
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
# js-dom R201：TIME_LIMIT 透传（与 testharness-dom 同款）——Range mega-case 解锁后
# 全量套件 ~30min，硬编码 900s 整轮被 guard 杀掉丢结果。
testharness-dom-native: target-disk-guard fetch-wpt-dom target/test-guard zero-wpt-runner-release
	ZW_NATIVE_DOM=1 ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-dom $(if $(FILTER),$(FILTER),)

# editing goal M1 / DC-1：上游 selection/ Selection API 可观察面 testharness 基线。
# 用例 gitignored（23 号 goal 脚本 lib.sh deep 拉取，拉取完成标记幂等——
# wptfyi-editing DC-2 扩面：旧 fetch-selection-subset.sh 显式清单 ~60 案只够
# ui-edit 快照 31/231 覆盖文件，deep 拉全 ~200 案进 CI）。filter 按路径子串透传：
# make testharness-selection FILTER=getSelection。
fetch-wpt-selection:
	bash tests/wpt-runner/scripts/goals/23-selection-compat.sh

testharness-selection: target-disk-guard fetch-wpt-selection target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-selection $(if $(FILTER),$(FILTER),)

# keyboard-default-actions goal M1 / DC-1：上游 uievents/keyboard + forms 隐式提交
# 键盘默认动作 testharness 基线。用例 gitignored（fetch-keyboard-subset.sh 按需拉取）。
# filter 按路径子串透传：make testharness-keyboard FILTER=implicit。
fetch-wpt-keyboard:
	bash tests/wpt-runner/scripts/fetch-keyboard-subset.sh

testharness-keyboard: target-disk-guard fetch-wpt-keyboard target/test-guard zero-wpt-runner-release
	ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1 ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-keyboard $(if $(FILTER),$(FILTER),)

# UI/指针事件 testharness 基线（uievents-compat goal M1 / DC-1）。用例 gitignored
# （scripts/goals/50-uievents-compat.sh 按需拉取，幂等续拉；GitHub API 限流时部分
# 目录跳过属预期）。filter 按路径子串透传：make testharness-uievents FILTER=pointer。
fetch-wpt-uievents:
	bash tests/wpt-runner/scripts/goals/50-uievents-compat.sh

testharness-uievents: target-disk-guard fetch-wpt-uievents target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1200) -- ./target/release/zero-wpt-runner testharness-uievents $(if $(FILTER),$(FILTER),)

# touch-events goal M1 / DC-1：上游 touch-events 构造器/属性面 testharness 基线。
# 用例 gitignored（scripts/goals/22-touch-events-compat.sh 按需拉取）。宿主触摸注入面
# （testdriver touch Actions 十案）runner 侧排除挂 M3，见 touch_events_case_skipped。
# filter 按路径子串透传：make testharness-touch-events FILTER=constructor
fetch-wpt-touch-events:
	bash tests/wpt-runner/scripts/goals/22-touch-events-compat.sh

testharness-touch-events: target-disk-guard fetch-wpt-touch-events target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-touch-events $(if $(FILTER),$(FILTER),)

# IndexedDB goal M1：上游 IndexedDB factory/global/event 首批 testharness 基线。
# `.any.js` 用例由 runner 包装为 window test；filter 按文件路径子串透传。
fetch-wpt-indexeddb:
	bash tests/wpt-runner/scripts/fetch-indexeddb-subset.sh

testharness-indexeddb: target-disk-guard fetch-wpt-indexeddb target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-indexeddb $(if $(FILTER),$(FILTER),)

# media-elements goal M1 / DC-1：上游 media-elements 语义面（反射/canPlayType/元数据
# 初值/track 反射）testharness 基线。用例 gitignored（fetch-media-subset.sh 按需拉取）。
# filter 按路径子串透传：make testharness-media FILTER=canPlayType。
fetch-wpt-media:
	bash tests/wpt-runner/scripts/fetch-media-subset.sh

testharness-media: target-disk-guard fetch-wpt-media target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-media $(if $(FILTER),$(FILTER),)

# media-audio goal M3 切片 2：Web Audio 最小面（AudioContext 构造/节点接口/
# connect 语义）WPT 可执行子集。用例 gitignored（fetch-webaudio-subset.sh 拉取）。
# filter 按路径子串透传：make testharness-webaudio FILTER=destination。
fetch-wpt-webaudio:
	bash tests/wpt-runner/scripts/fetch-webaudio-subset.sh

testharness-webaudio: target-disk-guard fetch-wpt-webaudio target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-webaudio $(if $(FILTER),$(FILTER),)

# Cache API goal M1：上游 CacheStorage `.any.js` window 面首批基线。
# 用例 gitignored（fetch-cache-storage-window-subset.sh 按固定 SHA manifest 恢复）。
fetch-wpt-cache-storage:
	$(WPT_BASH) tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh

audit-wpt-cache-storage:
	$(WPT_BASH) tests/wpt-runner/scripts/fetch-cache-storage-window-subset.sh --verify-only

testharness-cache-storage: target-disk-guard fetch-wpt-cache-storage target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- \
		./target/release/zero-wpt-runner testharness-cache-storage \
		--wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root $(if $(FILTER),$(FILTER),)

baseline-wpt-cache-storage: target-disk-guard fetch-wpt-cache-storage target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- \
		python3 tests/wpt-runner/scripts/run-cache-storage-window-baseline.py \
		--runner ./target/release/zero-wpt-runner \
		--wpt-data tests/wpt-runner/wpt-data/.cache-storage-window-root $(if $(OUTPUT),--output $(OUTPUT),) $(if $(SUMMARY),--summary $(SUMMARY),)

# OPFS goal M1：上游 File System (`fs/`) window 面 `.any.js` 子集（getDirectory/
# get*Handle/writable 流/迭代/resolve）。用例 gitignored（fetch-fs-subset.sh 按需拉取）。
# filter 按路径子串透传：make testharness-fs FILTER=WritableFileStream。
fetch-wpt-fs:
	bash tests/wpt-runner/scripts/fetch-fs-subset.sh

testharness-fs: target-disk-guard fetch-wpt-fs target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-fs $(if $(FILTER),$(FILTER),)

# Page WASM goal M1 / DC-1：上游 wasm/jsapi 标准 JS API 面 window 可执行子集
# （Module/Instance/Memory/Table/Global/compile/instantiate/validate/导出面/链接错误，
# 31 案）。用例 gitignored（fetch-wasm-subset.sh 按需拉取，jsdelivr 首选 raw 回落）。
# skip 域（esm-integration/jspi/js-string/gc/exception/tag/tentative 面）见脚本头注释。
# 基线首跑 Fail 多为预期（记录通过率用），不作为 land 门禁。
# filter 按路径子串透传：make testharness-wasm FILTER=memory。
fetch-wpt-wasm:
	bash tests/wpt-runner/scripts/fetch-wasm-subset.sh

testharness-wasm: target-disk-guard fetch-wpt-wasm target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-wasm $(if $(FILTER),$(FILTER),)

# wasm-compat goal M1 / DC-1+DC-2：wasm/core 全量（spec 解释器 .wast.js.html wrapper
# 页，含 simd/gc/exceptions 等 proposal 子目录——基线按 proposal 归簇）+ wasm/webapi
# （compile/instantiateStreaming fetch 集成面）。用例 gitignored（goals/32-wasm-compat.sh
# 深挖拉取）；skip 域（esm-integration/idlharness/worker-only 变体）见
# testharness_wasm_core.rs wasm_core_case_skipped 文档。基线首跑 Fail 多为预期（记录
# 通过率用），不作为 land 门禁。jsapi 回归底线通道是 testharness-wasm（不动）。
# filter 按路径子串透传：make testharness-wasm-core FILTER=wasm/core/simd。
fetch-wpt-wasm-core:
	bash tests/wpt-runner/scripts/goals/32-wasm-compat.sh

testharness-wasm-core: target-disk-guard fetch-wpt-wasm fetch-wpt-wasm-core target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- ./target/release/zero-wpt-runner testharness-wasm-core $(if $(FILTER),$(FILTER),)

# wptfyi-runtime goal DC-1（docs/goal/archive/wptfyi-runtime.md）：third_party/test262 上游
# 语料 vendoring（goals/48-test262-runtime.sh——git partial clone + cone sparse，
# pin = lib.sh WPT_REV，vendored.toml 记 test262 上游 rev b66872a9）+ runner 通道
# （testharness_test262.rs——上游 serve 包装链折叠单页执行，变体名 .test262.html /
# .test262.strict.html / .test262-module.html 与 wpt.fyi summary 逐字对齐）。
# 基线首跑 Fail 多为预期（记录通过率用），不作为 land 门禁。filter 按路径子串
# 透传：make testharness-test262 FILTER=third_party/test262/test/language/statements。
fetch-wpt-test262:
	bash tests/wpt-runner/scripts/goals/48-test262-runtime.sh

testharness-test262: target-disk-guard fetch-wpt-test262 target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- ./target/release/zero-wpt-runner testharness-test262 $(if $(FILTER),$(FILTER),)

# net-api-compat goal M1 / DC-1：上游六 corpus（fetch/xhr/url/mimesniff/streams/
# eventsource）window 可执行子集。用例 gitignored（goals/20-net-api-compat.sh 按需
# 拉取，GITHUB_TOKEN 可选抬高列目录限流）；skip 域（server-handler .py 重依赖/策略
# 头面/HTTP 栈面/新 API 面显式不拉）见脚本头注释。基线首跑 Fail 多为预期（记录
# 通过率用），不作为 land 门禁。
# filter 按路径子串透传：make testharness-net-api FILTER=fetch/。
fetch-wpt-net-api:
	bash tests/wpt-runner/scripts/goals/20-net-api-compat.sh

testharness-net-api: target-disk-guard fetch-wpt-net-api target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-net-api $(if $(FILTER),$(FILTER),)

# encoding-compat goal M1 / DC-1：上游 encoding corpus（TextEncoder/TextDecoder/
# labels/streams 变体 + legacy-mb 七编码资产）window 可执行子集。用例 gitignored
# （goals/30-encoding-compat.sh 按需拉取，GOAL_PULL_ANY_JS=1——encoding 以 .any.js
# 为主要形态）；skip 域（文档级嗅探面挂账 html-syntax-compat/form 提交编码面/iframe
# 子文档面）见脚本头注释。基线首跑 Fail 多为预期（记录通过率用），不作为 land 门禁。
# filter 按路径子串透传：make testharness-encoding FILTER=textdecoder-。
fetch-wpt-encoding:
	bash tests/wpt-runner/scripts/goals/30-encoding-compat.sh

testharness-encoding: target-disk-guard fetch-wpt-encoding target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-encoding $(if $(FILTER),$(FILTER),)

# encoding-legacy-mb goal M1 / DC-2（docs/goal/encoding-legacy-mb.md）：legacy-mb 七族
# decode wrapper 页（testharness + chars iframe + variant 窗口 ×14；goal 划界排除
# encode-form/encode-href 面）。全量 ~1.1M 子测量级——整跑须大 envelope 或 FILTER
# 分族出数（如 FILTER=euc-jp/）。ZW_SCRIPT_TIMEOUT_MS：decode wrapper 每 variant
# 对 chars 全量 span 做桥调用密集断言（~3–5 万次），90s 缺省看门狗预算内跑不完
# ——本通道放宽到 600s（runner 侧 V8 看门狗，缺省其他通道零行为变化）。
fetch-wpt-encoding-legacy-mb:
	bash tests/wpt-runner/scripts/goals/31-encoding-legacy-mb.sh

testharness-encoding-legacy-mb: target-disk-guard fetch-wpt-encoding-legacy-mb target/test-guard zero-wpt-runner-release
	ZW_SCRIPT_TIMEOUT_MS=600000 ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),5400) -- ./target/release/zero-wpt-runner testharness-encoding-legacy-mb $(if $(FILTER),$(FILTER),)

# form-url-encoding-compat goal（docs/goal/archive/form-url-encoding-compat.md）：legacy-mb
# encode 面（form 提交编码序列化 + href URL 序列化）68 文件 + encoding-compat-v2
# goal M1 根面 2 文件（big5/iso-2022-jp-encoder inline a.href）通道——与 decode 通道
# face 互斥、语料同源（fetch 目标复用 31 号脚本 + 37 号脚本补根面，均幂等）。
# ZW_SCRIPT_TIMEOUT_MS 沿 31 号先例（encode-form 页每 variant ~1000 async_test
# 分窗断言）。基线期面缺（acceptCharset 序列化 / encoding-parse 未落地）大量 Fail
# 只记账不作为 land 门禁。
fetch-wpt-encoding-compat-v2:
	bash tests/wpt-runner/scripts/goals/37-encoding-compat-v2.sh

testharness-form-url-encoding: target-disk-guard fetch-wpt-encoding-legacy-mb fetch-wpt-encoding-compat-v2 target/test-guard zero-wpt-runner-release
	ZW_SCRIPT_TIMEOUT_MS=600000 ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),5400) -- ./target/release/zero-wpt-runner testharness-form-url-encoding $(if $(FILTER),$(FILTER),)

# html-syntax-compat goal M1 / DC-1：上游 html/syntax（解析树/序列化）+ html/dom
# （接口语义/reflection/render-blocking）+ encoding-compat P5 双向记账转入的文档级
# 编码嗅探案面（charset/xmldecl——reftest/HTTP 头形态在 testharness 通道外按 skip
# 规则记账）。用例 gitignored（goals/40-html-syntax-compat.sh 按需拉取，depth-2
# support/ 资产显式补拉）。skip 域见 testharness.rs html_syntax_case_skipped 注释。
# 基线首跑 Fail 多为预期（记录通过率用），不作为 land 门禁。
# filter 按路径子串透传：make testharness-html-syntax FILTER=serializing-。
fetch-wpt-html-syntax:
	bash tests/wpt-runner/scripts/goals/40-html-syntax-compat.sh

testharness-html-syntax: target-disk-guard fetch-wpt-html-syntax target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-html-syntax $(if $(FILTER),$(FILTER),)

# navigation-compat goal M1 / DC-1：上游导航面 corpus（navigation-api/ +
# html/browsers/history 四叶 + browsing-the-web 遍历面 + the-iframe-element 属性面）
# window 基线。用例 gitignored（goals/60-navigation-compat.sh 按需拉取；pin 下无顶层
# history/——会话历史 corpus 落 html/browsers/history/**，fetch 脚本头注释记账）。
# skip 域见 testharness.rs navigation_case_skipped 注释（iframe 深结构面 = M3 门控，
# 基线零执行记账）。基线首跑 Fail 多为预期（记录通过率用），不作为 land 门禁。
# filter 按路径子串透传：make testharness-navigation FILTER=the-history-interface。
fetch-wpt-navigation:
	bash tests/wpt-runner/scripts/goals/60-navigation-compat.sh

testharness-navigation: target-disk-guard fetch-wpt-navigation target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-navigation $(if $(FILTER),$(FILTER),)

# page-state-compat goal M1 / DC-1：上游三小 corpus（beacon/ + page-visibility/ +
# html/browsers/offline/browser-state/——上游无顶层 online/，navigator.onLine 用例在
# browser-state 叶）window 基线。用例 gitignored（goals/27-page-state-compat.sh 按需
# 拉取）。skip 域见 testharness_page_state.rs page_state_case_skipped 注释（manual/
# minimize/iframe/window.open 面）；beacon 服务端 fixture 端点为 M2 实现轮，基线先
# 如实记账。基线首跑 Fail 多为预期（记录通过率用），不作为 land 门禁。
# filter 按路径子串透传：make testharness-page-state FILTER=beacon。
fetch-wpt-page-state:
	bash tests/wpt-runner/scripts/goals/27-page-state-compat.sh

testharness-page-state: target-disk-guard fetch-wpt-page-state target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-page-state $(if $(FILTER),$(FILTER),)

# dnd-compat goal M1 / DC-1：上游 html/editing/dnd/ corpus（957 文件全量拉取做
# census；上游无顶层 dnd/，语料路径修正见 goals/28-dnd-compat.sh 注记）。用例
# gitignored。运行面 = testharness 可执行子集（DataTransfer/DragEvent 构造器面），
# skip 域见 testharness_dnd.rs dnd_case_skipped 注释（manual/IE 遗产/testdriver
# Actions 拖拽管线面）；交互管线为 M3 用户门控。基线首跑 Fail 多为预期（记录
# 通过率用），不作为 land 门禁。GITHUB_TOKEN 环境变量透传（深挖列目录用，可选）。
# filter 按路径子串透传：make testharness-dnd FILTER=datastore。
fetch-wpt-dnd:
	bash tests/wpt-runner/scripts/goals/28-dnd-compat.sh

testharness-dnd: target-disk-guard fetch-wpt-dnd target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-dnd $(if $(FILTER),$(FILTER),)

# Pointer Lock goal M1 / DC-1：上游 pointerlock/ corpus 可执行子集基线（事件
# 构造器 + 拒绝语义面；拒绝语义红 = shim R2939 缺 transient activation 门——
# 默认 grant，归 M2/M3 门控面）。skip 域（testdriver 真锁获取 13 案 +
# test-helper.js helper 2 案 + manual 7 案）见 testharness_pointerlock.rs 头注；
# 捕获管线为 M2/M3 用户门控。用例 gitignored（goals/29-pointerlock-compat.sh
# 按需拉取）。
# filter 按路径子串透传：make testharness-pointerlock FILTER=constructor。
fetch-wpt-pointerlock:
	bash tests/wpt-runner/scripts/goals/29-pointerlock-compat.sh

testharness-pointerlock: target-disk-guard fetch-wpt-pointerlock target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-pointerlock $(if $(FILTER),$(FILTER),)

# Web Components goal M1：上游 custom-elements/ + shadow-dom/ +
# html/semantics/scripting-1/the-template-element/ 三目录 window 面（CE 收口/template
# 真实化/slot 全链路的 driving 面）。用例 gitignored（fetch-web-components-subset.sh
# 按需拉取）。skip 域（渲染级 composed tree 等）见 fetch 脚本头注释。
# filter 按路径子串透传：make testharness-web-components FILTER=slotchange。
fetch-wpt-web-components:
	bash tests/wpt-runner/scripts/fetch-web-components-subset.sh

testharness-web-components: target-disk-guard fetch-wpt-web-components target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-web-components $(if $(FILTER),$(FILTER),)

# shadow-dom-rendering goal B6：42 号 WC 通道渲染级排除域（几何命中/滚动/焦点/
# caret/Highlight/交互）10 案的补充池 testharness 基线。语料由 45 号 goal 脚本
# 幂等预置（reftest 池 + 补充池显式清单，missing=0 断言；runner 侧
# SHADOW_RENDER_POOL_CASES 与 SUPPLEMENT_POOL 两侧同源）。filter 按路径子串
# 透传：make testharness-shadow-dom FILTER=offsetParent。
fetch-wpt-shadow-render-pool:
	bash tests/wpt-runner/scripts/goals/45-shadow-dom-rendering.sh

testharness-shadow-dom: target-disk-guard fetch-wpt-shadow-render-pool target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-shadow-dom $(if $(FILTER),$(FILTER),)

# event-loop-spec goal M1 / DC-1：上游 IntersectionObserver / ResizeObserver window
# 可执行面 testharness 基线。用例 gitignored（fetch-observers-subset.sh 按固定 SHA
# manifest 恢复，runner 侧内容规则筛减 ref/iframe/v2/resources）。filter 按路径子串
# 透传：make testharness-intersection-observer FILTER=root-margin。
fetch-wpt-observers:
	bash tests/wpt-runner/scripts/fetch-observers-subset.sh

testharness-intersection-observer: target-disk-guard fetch-wpt-observers target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-intersection-observer $(if $(FILTER),$(FILTER),)

testharness-resize-observer: target-disk-guard fetch-wpt-observers target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-resize-observer $(if $(FILTER),$(FILTER),)

# web-api-batch2 goal M1（Clipboard API + Fullscreen，docs/goal/archive/web-api-batch2.md）：
# 上游 clipboard-apis/ + fullscreen/ 两 corpus window 可执行子集（fetch 脚本显式清单
# + runner 侧内容规则筛减 manual/iframe/rendering/resources；`:fullscreen` 样式面
# 跨域记账 rendering-compat 流域）。基线首跑只记通过率不作为 land 门禁。filter 按
# 路径子串透传：make testharness-fullscreen FILTER=model。
fetch-wpt-clipboard-apis:
	bash tests/wpt-runner/scripts/fetch-clipboard-apis-subset.sh

fetch-wpt-fullscreen:
	bash tests/wpt-runner/scripts/fetch-fullscreen-subset.sh

testharness-clipboard-apis: target-disk-guard fetch-wpt-clipboard-apis target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-clipboard-apis $(if $(FILTER),$(FILTER),)

testharness-fullscreen: target-disk-guard fetch-wpt-fullscreen target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-fullscreen $(if $(FILTER),$(FILTER),)

# css-testharness-compat M2（DC-3）：上游 css/css-variables testharness 语料
# （自 site-optimizer R5114 通道迁入共享分片脚本——每案独立进程 + 外部墙钟 +
# JSON 通道，见 run-css-testharness-corpus.sh 头注释）。数据双源：fetch-wpt-data
# 精选仓 + fetch-wpt-css-testharness 深挖补拉上游全量。病态案 runner 谓词排除
# （PATHO_CASES），基线只记录通过率不设门禁。
# 用法: make testharness-css-variables
testharness-css-variables: fetch-wpt-data fetch-wpt-css-variables target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),2400) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-variables ./target/release/zero-wpt-runner
# 单案/子串定向：make testharness-css-variables-one FILTER=case-sensitive
testharness-css-variables-one: fetch-wpt-css-variables target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-variables $(if $(FILTER),$(FILTER),)

# css-testharness-compat goal M0（docs/goal/archive/css-testharness-compat.md DC-1）：上游
# css/values + css/selectors testharness 试点通道（CSSOM/style API 断言面）。用例
# gitignored（goals/35 号脚本 GOAL_FETCH_DEEP=1 深拉）。基线首跑只记录通过率，不设
# 门禁（css-variables corpus 先例）。全量走每案独立进程分片（runner 单进程长套件
# RSS 线性累积 ~35MB/案，M1 治理前分片是守卫内唯一全量通道）；单案/子串定向走
# -one（in-process 子命令）。evidence JSON：CSS_CORPUS_JSON=<path> 汇总逐案四态计数。
fetch-wpt-css-testharness:
	bash tests/wpt-runner/scripts/goals/35-css-testharness-compat.sh

# weekly css-face 三分片定向 fetch（M4 / DC-4）：CSS_FETCH_ONLY 白名单过滤
# 35 号脚本 DIRS。CI runner 每次全新 checkout（wpt-data gitignored），全量
# ~2 万文件顺序深挖单 job 装不下 150min 预算——按语料体量三分（本地实测
# ~1.4 文件/s：A 试点+批次1+transforms ~4.5k、B 批次2+批次3 ~6.2k、
# C 批次4/5 大目录 motion+grid+text ~6.6k）。css/support 共享 helper 目录
# （parsing/computed/interpolation-testcommon.js 等，15/16 目录引用）必须
# 随片携带，漏带则引用页全数坍缩为单 Fail（运行 41000541561 实证）。
# 本地全量拉取用无后缀 target。
fetch-wpt-css-testharness-a:
	CSS_FETCH_ONLY="css/css-values css/selectors css/css-color css/css-align css/css-sizing css/css-transforms css/support" bash tests/wpt-runner/scripts/goals/35-css-testharness-compat.sh

fetch-wpt-css-testharness-b:
	CSS_FETCH_ONLY="css/css-typed-om css/css-fonts css/css-shapes css/css-anchor-position css/css-masking css/css-backgrounds css/support" bash tests/wpt-runner/scripts/goals/35-css-testharness-compat.sh

fetch-wpt-css-testharness-c:
	CSS_FETCH_ONLY="css/motion css/css-grid css/css-text css/support" bash tests/wpt-runner/scripts/goals/35-css-testharness-compat.sh

# weekly css-v2-face 分片定向 fetch（css 二期 M2 / DC-2）：CSS_FETCH_ONLY
# 白名单过滤 41 号脚本 DIRS（v2 45 项池 ~18k blob，同上单 job 装不下——按
# 接线批次分片：A = B1 前 12 目录 ~5.2k、B = B1 后 12 + B2 7 目录 ~3.2k、
# C = B3 混合布局 13 目录 ~5.9k）。css/support 共享 helper 必须随片
# （一期实证漏带则引用页全数坍缩）。41 号为 pin 树清单逐文件对账 sync——
# 幂等，已有语料只补缺失/不符文件。
fetch-wpt-css-testharness-v2-a:
	CSS_FETCH_ONLY="css/cssom css/css-images css/filter-effects css/css-ui css/css-borders css/css-logical css/css-text-decor css/css-box css/css-pseudo css/css-display css/css-mixins css/css-syntax css/support" bash tests/wpt-runner/scripts/goals/41-css-testharness-compat-v2.sh

fetch-wpt-css-testharness-v2-b:
	CSS_FETCH_ONLY="css/fill-stroke css/css-shadow css/css-size-adjust css/css-content css/css-will-change css/css-color-adjust css/css-easing css/compositing css/css-counter-styles css/css-nesting css/css-color-hdr css/css-env css/css-conditional css/mediaqueries css/css-cascade css/CSS2 css/css-forms css/css-link-params css/css-font-loading css/support" bash tests/wpt-runner/scripts/goals/41-css-testharness-compat-v2.sh

fetch-wpt-css-testharness-v2-c:
	CSS_FETCH_ONLY="css/css-gaps css/css-flexbox css/css-multicol css/css-position css/css-overflow css/css-tables css/css-lists css/css-inline css/css-scroll-snap css/css-viewport css/css-break css/css-contain css/css-writing-modes css/support" bash tests/wpt-runner/scripts/goals/41-css-testharness-compat-v2.sh

testharness-css-values: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-values ./target/release/zero-wpt-runner

testharness-selectors: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh selectors ./target/release/zero-wpt-runner

# css-testharness-compat M3 扩面首批（top15 分母目录过 M0 模板）：通道与试点
# 同款（共享底座 + 分片脚本），基线只记录不设门禁。首次 fetch 注意：精选仓
# 已含 .html 的目录会走深挖幂等快路径，补拉上游全量须 FORCE=1（css-variables
# M2 先例）。
# 用法: make testharness-css-color / testharness-css-align / testharness-css-sizing
testharness-css-color: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-color ./target/release/zero-wpt-runner

testharness-css-align: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-align ./target/release/zero-wpt-runner

testharness-css-sizing: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-sizing ./target/release/zero-wpt-runner

# M3 扩面批次 2（同款共享通道）：make testharness-css-typed-om /
# testharness-css-fonts / testharness-css-shapes（首批 FORCE=1 注意事项同上）
testharness-css-typed-om: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-typed-om ./target/release/zero-wpt-runner

testharness-css-fonts: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-fonts ./target/release/zero-wpt-runner

testharness-css-shapes: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-shapes ./target/release/zero-wpt-runner

# M3 扩面批次 3（同款共享通道）：make testharness-css-anchor-position /
# testharness-css-masking / testharness-css-backgrounds（首批 FORCE=1 注意
# 事项同上；backgrounds 精选仓存量页补差口径同 fonts——按 pin 树清单
# blob hash 定向，不 FORCE 全量重拉）
testharness-css-anchor-position: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-anchor-position ./target/release/zero-wpt-runner

testharness-css-masking: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-masking ./target/release/zero-wpt-runner

testharness-css-backgrounds: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-backgrounds ./target/release/zero-wpt-runner

# M3 扩面批次 4（同款共享通道）：make testharness-css-transforms /
# testharness-css-motion（motion 扫描根 css/motion——分片脚本历史命名特例）
testharness-css-transforms: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-transforms ./target/release/zero-wpt-runner

testharness-css-motion: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh motion ./target/release/zero-wpt-runner

# M3 扩面批次 5（top15 收尾，重量级语料独立轮次）：make testharness-css-grid /
# testharness-css-text（均带 css- 前缀，分片脚本 css-* 规则零改动接入）
testharness-css-grid: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-grid ./target/release/zero-wpt-runner

testharness-css-text: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-text ./target/release/zero-wpt-runner

# css-testharness-compat-v2 M1 批次 1（B1 值/属性解析序列化簇首批 6 目录）：
# 通道与一期 M3 扩面同款（共享底座 + 分片脚本 + 41 号 sync 脚本语料），基线
# 只记录不设门禁；语料由 goals/41 脚本 sync（fetch-wpt-css-testharness 依赖
# 幂等，不重复拉本批目录）。
# 用法: make testharness-cssom / testharness-css-images / testharness-filter-effects /
#   testharness-css-ui / testharness-css-borders / testharness-css-logical
testharness-cssom: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh cssom ./target/release/zero-wpt-runner

testharness-css-images: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-images ./target/release/zero-wpt-runner

testharness-filter-effects: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh filter-effects ./target/release/zero-wpt-runner

testharness-css-ui: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-ui ./target/release/zero-wpt-runner

testharness-css-borders: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-borders ./target/release/zero-wpt-runner

testharness-css-logical: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-logical ./target/release/zero-wpt-runner

# css-testharness-compat-v2 M1 批次 2（B1 值/属性解析序列化簇余 18 目录）：
# 通道与批次 1 同款（共享底座 + 分片脚本 + 41 号 sync 脚本语料），基线只记录
# 不设门禁；fill-stroke / compositing 上游目录名无 css- 前缀，分片脚本显式
# 映射臂，css-* 前缀目录走通用规则零改动接入。
# 用法: make testharness-css-text-decor / testharness-css-box / testharness-css-pseudo /
#   testharness-css-display / testharness-css-mixins / testharness-css-syntax /
#   testharness-fill-stroke / testharness-css-shadow / testharness-css-size-adjust /
#   testharness-css-content / testharness-css-will-change / testharness-css-color-adjust /
#   testharness-css-easing / testharness-compositing / testharness-css-counter-styles /
#   testharness-css-nesting / testharness-css-color-hdr / testharness-css-env
testharness-css-text-decor: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-text-decor ./target/release/zero-wpt-runner

testharness-css-box: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-box ./target/release/zero-wpt-runner

testharness-css-pseudo: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-pseudo ./target/release/zero-wpt-runner

testharness-css-display: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-display ./target/release/zero-wpt-runner

testharness-css-mixins: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-mixins ./target/release/zero-wpt-runner

testharness-css-syntax: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-syntax ./target/release/zero-wpt-runner

testharness-fill-stroke: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh fill-stroke ./target/release/zero-wpt-runner

testharness-css-shadow: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-shadow ./target/release/zero-wpt-runner

testharness-css-size-adjust: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-size-adjust ./target/release/zero-wpt-runner

testharness-css-content: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-content ./target/release/zero-wpt-runner

testharness-css-will-change: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-will-change ./target/release/zero-wpt-runner

testharness-css-color-adjust: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-color-adjust ./target/release/zero-wpt-runner

testharness-css-easing: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-easing ./target/release/zero-wpt-runner

testharness-compositing: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh compositing ./target/release/zero-wpt-runner

testharness-css-counter-styles: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-counter-styles ./target/release/zero-wpt-runner

testharness-css-nesting: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-nesting ./target/release/zero-wpt-runner

testharness-css-color-hdr: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-color-hdr ./target/release/zero-wpt-runner

testharness-css-env: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-env ./target/release/zero-wpt-runner

# css-testharness-compat-v2 M2 批次 3（B2 规则/条件/级联簇 7 目录）：通道与
# 批次 1-2 同款，基线只记录不设门禁；CSS2 只含 html 面（.xht XML 面不在
# 通道，数值门按 html 面折算）；mediaqueries / CSS2 上游目录名无 css- 前缀，
# 分片脚本显式映射臂。
# 用法: make testharness-css-conditional / testharness-mediaqueries / testharness-css-cascade /
#   testharness-css2 / testharness-css-forms / testharness-css-link-params / testharness-css-font-loading
testharness-css-conditional: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-conditional ./target/release/zero-wpt-runner

testharness-mediaqueries: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh mediaqueries ./target/release/zero-wpt-runner

testharness-css-cascade: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-cascade ./target/release/zero-wpt-runner

testharness-css2: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh CSS2 ./target/release/zero-wpt-runner

testharness-css-forms: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-forms ./target/release/zero-wpt-runner

testharness-css-link-params: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-link-params ./target/release/zero-wpt-runner

testharness-css-font-loading: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-font-loading ./target/release/zero-wpt-runner

# css-testharness-compat-v2 M2 批次 4（B3 混合布局簇 13 目录）：通道与批次
# 1-3 同款，基线只记录不设门禁；M0 划界——解析/序列化份额入账，布局探测面
# 失败归口布局账（rendering-compat 控制面 / 独立 layout goal 待立项），本
# 通道修到序列化/声明值层为止。13 目录均带 css- 前缀，分片脚本通用规则零
# 改动接入。
# 用法: make testharness-css-gaps / testharness-css-flexbox / testharness-css-multicol /
#   testharness-css-position / testharness-css-overflow / testharness-css-tables /
#   testharness-css-lists / testharness-css-inline / testharness-css-scroll-snap /
#   testharness-css-viewport / testharness-css-break / testharness-css-contain /
#   testharness-css-writing-modes
testharness-css-gaps: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-gaps ./target/release/zero-wpt-runner

testharness-css-flexbox: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-flexbox ./target/release/zero-wpt-runner

testharness-css-multicol: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-multicol ./target/release/zero-wpt-runner

testharness-css-position: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-position ./target/release/zero-wpt-runner

testharness-css-overflow: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-overflow ./target/release/zero-wpt-runner

testharness-css-tables: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-tables ./target/release/zero-wpt-runner

testharness-css-lists: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-lists ./target/release/zero-wpt-runner

testharness-css-inline: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-inline ./target/release/zero-wpt-runner

testharness-css-scroll-snap: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-scroll-snap ./target/release/zero-wpt-runner

testharness-css-viewport: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-viewport ./target/release/zero-wpt-runner

testharness-css-break: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-break ./target/release/zero-wpt-runner

testharness-css-contain: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-contain ./target/release/zero-wpt-runner

testharness-css-writing-modes: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-writing-modes ./target/release/zero-wpt-runner

# weekly css-v2-face-d 分片定向 fetch（wptfyi-css goal DC-2 语料扩面）：
# CSS_FETCH_ONLY 白名单过滤 41 号脚本 DIRS（13 个 DC-1 未 fetch 目录，首拉
# ~2.4k blob）。41 号为 pin 树清单逐文件对账 sync——幂等，已有语料只补缺失。
# GITHUB_TOKEN 由调用方 export（41 号脚本头用法注释同款；未认证 60 req/h
# 对 14 目录 ~40 次 trees 查询偏紧）。css/support 共享 helper 必须随片
# （一期实证漏带则引用页全数坍缩）。
fetch-wpt-css-testharness-v3:
	CSS_FETCH_ONLY="css/css-transitions css/css-animations css/cssom-view css/css-view-transitions css/geometry css/css-properties-values-api css/css-highlight-api css/css-page css/css-scroll-anchoring css/css-scrollbars css/css-overscroll-behavior css/css-navigation css/css-parser-api css/support" bash tests/wpt-runner/scripts/goals/41-css-testharness-compat-v2.sh

# wptfyi-css goal DC-2 语料扩面（13 目录）：通道与 v2 批次同款，基线只记录
# 不设门禁；cssom-view / geometry 上游目录名无 css- 前缀，分片脚本显式
# 映射臂。
# 用法: make testharness-css-transitions / testharness-css-animations / testharness-cssom-view /
#   testharness-css-view-transitions / testharness-geometry / testharness-css-properties-values-api /
#   testharness-css-highlight-api / testharness-css-page / testharness-css-scroll-anchoring /
#   testharness-css-scrollbars / testharness-css-overscroll-behavior / testharness-css-navigation /
#   testharness-css-parser-api
testharness-css-transitions: fetch-wpt-css-testharness-v3 target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-transitions ./target/release/zero-wpt-runner

testharness-css-animations: fetch-wpt-css-testharness-v3 target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-animations ./target/release/zero-wpt-runner

testharness-cssom-view: fetch-wpt-css-testharness-v3 target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh cssom-view ./target/release/zero-wpt-runner

testharness-css-view-transitions: fetch-wpt-css-testharness-v3 target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-view-transitions ./target/release/zero-wpt-runner

testharness-geometry: fetch-wpt-css-testharness-v3 target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh geometry ./target/release/zero-wpt-runner

testharness-css-properties-values-api: fetch-wpt-css-testharness-v3 target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-properties-values-api ./target/release/zero-wpt-runner

testharness-css-highlight-api: fetch-wpt-css-testharness-v3 target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-highlight-api ./target/release/zero-wpt-runner

testharness-css-page: fetch-wpt-css-testharness-v3 target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-page ./target/release/zero-wpt-runner

testharness-css-scroll-anchoring: fetch-wpt-css-testharness-v3 target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-scroll-anchoring ./target/release/zero-wpt-runner

testharness-css-scrollbars: fetch-wpt-css-testharness-v3 target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-scrollbars ./target/release/zero-wpt-runner

testharness-css-overscroll-behavior: fetch-wpt-css-testharness-v3 target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-overscroll-behavior ./target/release/zero-wpt-runner

testharness-css-navigation: fetch-wpt-css-testharness-v3 target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-navigation ./target/release/zero-wpt-runner

testharness-css-parser-api: fetch-wpt-css-testharness-v3 target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-css-testharness-corpus.sh css-parser-api ./target/release/zero-wpt-runner

# 单案/子串定向：make testharness-css-values-one FILTER=calc / testharness-selectors-one FILTER=invalidation
testharness-css-values-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-values $(if $(FILTER),$(FILTER),)

testharness-selectors-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-selectors $(if $(FILTER),$(FILTER),)

testharness-css-color-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-color $(if $(FILTER),$(FILTER),)

testharness-css-align-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-align $(if $(FILTER),$(FILTER),)

testharness-css-sizing-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-sizing $(if $(FILTER),$(FILTER),)

testharness-css-typed-om-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-typed-om $(if $(FILTER),$(FILTER),)

testharness-css-fonts-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-fonts $(if $(FILTER),$(FILTER),)

testharness-css-shapes-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-shapes $(if $(FILTER),$(FILTER),)

testharness-css-anchor-position-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-anchor-position $(if $(FILTER),$(FILTER),)

testharness-css-masking-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-masking $(if $(FILTER),$(FILTER),)

testharness-css-backgrounds-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-backgrounds $(if $(FILTER),$(FILTER),)

testharness-css-transforms-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-transforms $(if $(FILTER),$(FILTER),)

testharness-css-motion-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-motion $(if $(FILTER),$(FILTER),)

testharness-css-grid-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-grid $(if $(FILTER),$(FILTER),)

testharness-css-text-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-text $(if $(FILTER),$(FILTER),)

testharness-cssom-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-cssom $(if $(FILTER),$(FILTER),)

testharness-css-images-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-images $(if $(FILTER),$(FILTER),)

testharness-filter-effects-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-filter-effects $(if $(FILTER),$(FILTER),)

testharness-css-ui-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-ui $(if $(FILTER),$(FILTER),)

testharness-css-borders-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-borders $(if $(FILTER),$(FILTER),)

testharness-css-logical-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-logical $(if $(FILTER),$(FILTER),)

testharness-css-text-decor-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-text-decor $(if $(FILTER),$(FILTER),)

testharness-css-box-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-box $(if $(FILTER),$(FILTER),)

testharness-css-pseudo-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-pseudo $(if $(FILTER),$(FILTER),)

testharness-css-display-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-display $(if $(FILTER),$(FILTER),)

testharness-css-mixins-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-mixins $(if $(FILTER),$(FILTER),)

testharness-css-syntax-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-syntax $(if $(FILTER),$(FILTER),)

testharness-fill-stroke-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-fill-stroke $(if $(FILTER),$(FILTER),)

testharness-css-shadow-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-shadow $(if $(FILTER),$(FILTER),)

testharness-css-size-adjust-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-size-adjust $(if $(FILTER),$(FILTER),)

testharness-css-content-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-content $(if $(FILTER),$(FILTER),)

testharness-css-will-change-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-will-change $(if $(FILTER),$(FILTER),)

testharness-css-color-adjust-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-color-adjust $(if $(FILTER),$(FILTER),)

testharness-css-easing-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-easing $(if $(FILTER),$(FILTER),)

testharness-compositing-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-compositing $(if $(FILTER),$(FILTER),)

testharness-css-counter-styles-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-counter-styles $(if $(FILTER),$(FILTER),)

testharness-css-nesting-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-nesting $(if $(FILTER),$(FILTER),)

testharness-css-color-hdr-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-color-hdr $(if $(FILTER),$(FILTER),)

testharness-css-env-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-env $(if $(FILTER),$(FILTER),)

testharness-css-conditional-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-conditional $(if $(FILTER),$(FILTER),)

testharness-mediaqueries-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-mediaqueries $(if $(FILTER),$(FILTER),)

testharness-css-cascade-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-cascade $(if $(FILTER),$(FILTER),)

testharness-css2-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css2 $(if $(FILTER),$(FILTER),)

testharness-css-forms-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-forms $(if $(FILTER),$(FILTER),)

testharness-css-link-params-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-link-params $(if $(FILTER),$(FILTER),)

testharness-css-font-loading-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-font-loading $(if $(FILTER),$(FILTER),)

testharness-css-gaps-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-gaps $(if $(FILTER),$(FILTER),)

testharness-css-flexbox-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-flexbox $(if $(FILTER),$(FILTER),)

testharness-css-multicol-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-multicol $(if $(FILTER),$(FILTER),)

testharness-css-position-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-position $(if $(FILTER),$(FILTER),)

testharness-css-overflow-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-overflow $(if $(FILTER),$(FILTER),)

testharness-css-tables-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-tables $(if $(FILTER),$(FILTER),)

testharness-css-lists-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-lists $(if $(FILTER),$(FILTER),)

testharness-css-inline-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-inline $(if $(FILTER),$(FILTER),)

testharness-css-scroll-snap-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-scroll-snap $(if $(FILTER),$(FILTER),)

testharness-css-viewport-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-viewport $(if $(FILTER),$(FILTER),)

testharness-css-break-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-break $(if $(FILTER),$(FILTER),)

testharness-css-contain-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-contain $(if $(FILTER),$(FILTER),)

testharness-css-writing-modes-one: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-css-writing-modes $(if $(FILTER),$(FILTER),)

# css-testharness-compat M1 / DC-2 等价判据复验：分片通道合并结果 ≡ 单进程 --json
# （(case, 子测名→status) 集合恒等）。换目录子集：make testharness-css-parity CORPUS=css-values FILTER=css/css-values/calc
testharness-css-parity: fetch-wpt-css-testharness target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),600) -- bash tests/wpt-runner/scripts/verify-css-corpus-json-parity.sh $(or $(CORPUS),selectors) $(or $(FILTER),css/selectors/i18n/) ./target/release/zero-wpt-runner

# editing-compat goal M1（docs/goal/editing-compat.md DC-2）：editing/run +
# editing/other execCommand 语料 testharness 通道。用例 gitignored（goals/33 号
# 脚本 GOAL_FETCH_DEEP=1 深拉，include/data 共享依赖随目录）。基线首跑只记录
# 通过率，不设门禁（css corpus 先例）。全量走每案独立进程分片（同 css 通道
# 根因）；分片/续跑环境变量与 transcript/JSON 产物见
# scripts/run-editing-testharness-corpus.sh 头注释。
fetch-wpt-editing:
	bash tests/wpt-runner/scripts/goals/33-editing-compat.sh

testharness-editing-run: fetch-wpt-editing target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-editing-testharness-corpus.sh run ./target/release/zero-wpt-runner

testharness-editing-other: fetch-wpt-editing target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- bash tests/wpt-runner/scripts/run-editing-testharness-corpus.sh other ./target/release/zero-wpt-runner

# 单案/子串定向：make testharness-editing-run FILTER=bold / testharness-editing-other FILTER=delete
# run 通道单案与 corpus 批同款 300s 脚本预算（run-editing-testharness-corpus.sh :73-75；
# runner 缺省 90s 看门狗截断大同步段变体——2026-11-03 gate27 电池 v1 实证
# insertparagraph 12min 截断 6,512P/141F/1T rc=2，与 corpus 基线缺 ~542 断言不可比）。
# FILTER 单引号包裹：分片面 filter 含 `|`（wptfyi-editing DC-1 按文件组拆分片），
# 不引则 make 展开后被 /bin/sh 当管道符（2026-11-11 实证 rc=127）；CI 侧 weekly.yml
# 先赋变量再展开不经 shell 重解析，无此问题。空 FILTER 不传参（runner 全量语义不变）。
testharness-editing-run-one: fetch-wpt-editing target-disk-guard target/test-guard zero-wpt-runner-release
	ZW_SCRIPT_TIMEOUT_MS=300000 ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),600) -- ./target/release/zero-wpt-runner testharness-editing-run $(if $(FILTER),'$(FILTER)',)

testharness-editing-other-one: fetch-wpt-editing target-disk-guard target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- ./target/release/zero-wpt-runner testharness-editing-other $(if $(FILTER),'$(FILTER)',)

# security-hardening goal M1（docs/goal/archive/security-hardening.md）：上游 content-security-policy/
# + mixed-content/ + secure-contexts/ 三 corpus window 可执行子集（fetch 脚本 sparse
# 克隆首批指令面 + runner 侧内容规则筛减 ref/manual/iframe/worker）。基线首跑只记
# 通过率不作为 land 门禁。filter 按路径子串透传：make testharness-csp FILTER=script-src。
fetch-wpt-security-csp:
	bash tests/wpt-runner/scripts/fetch-security-csp-subset.sh

testharness-csp: target-disk-guard fetch-wpt-security-csp target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- ./target/release/zero-wpt-runner testharness-csp $(if $(FILTER),$(FILTER),)

testharness-mixed-content: target-disk-guard fetch-wpt-security-csp target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-mixed-content $(if $(FILTER),$(FILTER),)

testharness-secure-contexts: target-disk-guard fetch-wpt-security-csp target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-secure-contexts $(if $(FILTER),$(FILTER),)

# gap-batch1 goal M1 / DC-2：覆盖率缺口十二小面 CONFIRMED 六面（quirks /
# domparsing / domxpath / subresource-integrity / cors / import-maps）window
# 可执行子集通道（fetch-gap-batch1-subset.sh 十二 corpus 一批落盘，含 LIKELY
# 五面 M3 语料基座与 mixed-content tentative 扩面）。基线首跑即便大量 Fail 也
# 只用于记录通过率（--format json 捕获写 evidence/），不作为 land 门禁。
# 路径子串透传：make testharness-quirks FILTER=unitless。
fetch-wpt-quirks fetch-wpt-domparsing fetch-wpt-domxpath fetch-wpt-subresource-integrity fetch-wpt-cors fetch-wpt-import-maps:
	bash tests/wpt-runner/scripts/fetch-gap-batch1-subset.sh

testharness-quirks: target-disk-guard fetch-wpt-quirks target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-quirks $(if $(FILTER),$(FILTER),)

testharness-domparsing: target-disk-guard fetch-wpt-domparsing target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-domparsing $(if $(FILTER),$(FILTER),)

testharness-domxpath: target-disk-guard fetch-wpt-domxpath target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-domxpath $(if $(FILTER),$(FILTER),)

testharness-subresource-integrity: target-disk-guard fetch-wpt-subresource-integrity target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-subresource-integrity $(if $(FILTER),$(FILTER),)

testharness-cors: target-disk-guard fetch-wpt-cors target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-cors $(if $(FILTER),$(FILTER),)

testharness-import-maps: target-disk-guard fetch-wpt-import-maps target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-import-maps $(if $(FILTER),$(FILTER),)

# gap-batch1 M3：LIKELY 五面通道（referrer-policy / input-events / preload /
# wai-aria / storage），fetch 与 M1 六面同一脚本；子目录清单按 M0 甄别池
# （manual/ 与 resources/ 资产页不进 runner 清单）。数值门 = 各面干跑基线
# P 数（evidence/2026-11-06-m3-*.md），storage 引擎缺位不设门。
fetch-wpt-referrer-policy fetch-wpt-input-events fetch-wpt-preload fetch-wpt-wai-aria fetch-wpt-storage:
	bash tests/wpt-runner/scripts/fetch-gap-batch1-subset.sh

testharness-referrer-policy: target-disk-guard fetch-wpt-referrer-policy target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-referrer-policy $(if $(FILTER),$(FILTER),)

testharness-input-events: target-disk-guard fetch-wpt-input-events target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-input-events $(if $(FILTER),$(FILTER),)

testharness-preload: target-disk-guard fetch-wpt-preload target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-preload $(if $(FILTER),$(FILTER),)

testharness-wai-aria: target-disk-guard fetch-wpt-wai-aria target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-wai-aria $(if $(FILTER),$(FILTER),)

testharness-storage: target-disk-guard fetch-wpt-storage target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-storage $(if $(FILTER),$(FILTER),)

# timing-animation-compat goal M1 / DC-1：上游 hr-time/ + performance-timeline/ +
# user-timing/ + web-animations/ 四 corpus window 可执行子集（goals/10 编号脚本显式
# 清单 + runner 侧内容规则筛减 manual/ref/iframe/worker/bfcache 基建/reftest-wait；
# .any.js 以 window 变体执行，wasm/fs 先例）。基线首跑即便大量 Fail 也只用于记录
# 通过率（--json 捕获写 evidence/），不作为 land 门禁。路径子串透传：
# make testharness-web-animations FILTER=KeyframeEffect。
fetch-wpt-timing-animation:
	bash tests/wpt-runner/scripts/goals/10-timing-animation-compat.sh

testharness-hr-time: target-disk-guard fetch-wpt-timing-animation target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-hr-time $(if $(FILTER),$(FILTER),)

testharness-performance-timeline: target-disk-guard fetch-wpt-timing-animation target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-performance-timeline $(if $(FILTER),$(FILTER),)

testharness-user-timing: target-disk-guard fetch-wpt-timing-animation target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-user-timing $(if $(FILTER),$(FILTER),)

testharness-web-animations: target-disk-guard fetch-wpt-timing-animation target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-web-animations $(if $(FILTER),$(FILTER),)

# WebStorage goal M1 / DC-1（docs/goal/archive/webstorage-compat.md）：语料 fetch 走 goals/15
# 预置脚本（timing-animation 先例）；runner 可执行子集 skip 域见 WEBSTORAGE_CASES 文档。
# make testharness-webstorage FILTER=getitem。
fetch-wpt-webstorage:
	bash tests/wpt-runner/scripts/goals/15-webstorage-compat.sh

testharness-webstorage: target-disk-guard fetch-wpt-webstorage target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-webstorage $(if $(FILTER),$(FILTER),)

# Compression goal M1 / DC-1（docs/goal/archive/compression-compat.md）：语料 fetch 走
# goals/24 预置脚本（webstorage 先例；含 resources/ + third_party/pako + media
# 两文件运行依赖）；runner 可执行子集 skip 域见 testharness.rs
# compression_case_skipped 文档。make testharness-compression FILTER=decompression。
fetch-wpt-compression:
	bash tests/wpt-runner/scripts/goals/24-compression-compat.sh

# html-semantics goal（docs/goal/html-semantics-compat.md M1 / DC-1）：html/semantics
# 九子域语料（microdata 上游不存在记空域）。filter 分域透传：
# make testharness-html-semantics FILTER=forms/
fetch-wpt-html-semantics:
	bash tests/wpt-runner/scripts/goals/25-html-semantics-compat.sh

# R5235：帧驱动 rAF（R2713a/R3254-KP4 既有开关，keyboard 同款入口开启）——
# waitForRender（双 rAF）改真实帧驱动，probe 拍间刷新 rect 快照，消除测试体内
# gBCR 滞后一拍（s33 探针钉定：closedby normal dialogB/popoverB 8 案 +8 无回归）。
testharness-html-semantics: target-disk-guard fetch-wpt-html-semantics target/test-guard zero-wpt-runner-release
	ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1 ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),5400) -- ./target/release/zero-wpt-runner testharness-html-semantics $(if $(FILTER),$(FILTER),)

testharness-compression: target-disk-guard fetch-wpt-compression target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-compression $(if $(FILTER),$(FILTER),)

# WebCryptoAPI 语料（wptfyi-webcrypto 目标 DC-1 周跑接入）：fetch 走 goals/38
# 全子树新增式脚本（v3 转正子树 cb6cc97a = HEAD 布局全量：老 stable 案 +
# modern-algos/secure-curves 两新族 + 散新案，幂等跳过已存在文件）。26 号脚本
# 锁旧 pin 布局（已退役 .tentative. 路径 + 旧内容支撑文件），对 v3 语料是
# 污染源，勿再对本语料目录运行。runner 可执行子集 skip 域见
# testharness.rs webcrypto_case_skipped 文档。
# make testharness-webcrypto FILTER=digest。
fetch-wpt-webcrypto:
	bash tests/wpt-runner/scripts/goals/38-webcrypto-compat-v3.sh

# 2026-11-09 v3 全语料（69,795 keys）本地与 CI 同阈；楔死期临时放宽 3600→7200
# （r3–r5 derived_bits_length 楔死，6a10ddaf8）已随 deriveBits [EnforceRange]
# 根因修复回落：r6 全语料实证 238s（2026-11-10），3600 留 ~15 倍余量。
testharness-webcrypto: target-disk-guard fetch-wpt-webcrypto target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),3600) -- ./target/release/zero-wpt-runner testharness-webcrypto $(if $(FILTER),$(FILTER),)

# WebGL goal M2（docs/goal/webgl-compat.md）：webgl/ 语料 fetch 走 goals/99 预置
# 脚本（GOAL_PULL_ANY_JS + idlharness 资产补拉——2026-10-22 裁决③）；runner 通道
# 无 skip 记账排除（idlharness 全量收）。语料 10 文件平铺，跑窗小：
# make testharness-webgl FILTER=bufferSubData。
fetch-wpt-webgl:
	bash tests/wpt-runner/scripts/goals/99-webgl-compat.sh

testharness-webgl: target-disk-guard fetch-wpt-webgl target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-webgl $(if $(FILTER),$(FILTER),)

# Cookies goal M1 / DC-1（docs/goal/cookies-compat.md）：语料 fetch 走 goals/18
# 预置脚本（webstorage 先例）；runner 可执行子集 skip 域见
# testharness_cookies.rs cookies_case_skipped 文档。HTTP 头驱动面（cookie.py
# Set-Cookie）静态语料不可执行——就地标记载入不可执行面分母说明。
# make testharness-cookies FILTER=prefix/。
fetch-wpt-cookies:
	bash tests/wpt-runner/scripts/goals/18-cookies-compat.sh

testharness-cookies: target-disk-guard fetch-wpt-cookies target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-cookies $(if $(FILTER),$(FILTER),)

# Console goal M1 / DC-1（docs/goal/archive/console-compat.md）：语料 fetch 走 goals/21
# 预置脚本。BiDi log.entryAdded 捕获通道依赖案（count/timing/format-specifier
# logging 面）整案 Unsupported——通道甄别结论见
# docs/goal/archive/console-compat/evidence/2026-10-14-m1-baseline.md。
# make testharness-console FILTER=count。
fetch-wpt-console:
	bash tests/wpt-runner/scripts/goals/21-console-compat.sh

testharness-console: target-disk-guard fetch-wpt-console target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-console $(if $(FILTER),$(FILTER),)

# Performance API goal M1 / DC-1（docs/goal/archive/performance-api-compat.md）：语料 fetch 走
# goals/19 预置脚本（WPT 无 performance-api 顶层目录——resource/navigation/event/
# longtask 四独立套件，goals/19 DIRS 记账）；runner 可执行子集 skip 域见
# testharness.rs performance_api_case_skipped 文档，缺面甄别报告落
# docs/goal/archive/performance-api-compat/evidence/。make testharness-resource-timing
# FILTER=buffer-full。
fetch-wpt-performance-api:
	bash tests/wpt-runner/scripts/goals/19-performance-api-compat.sh

testharness-resource-timing: target-disk-guard fetch-wpt-performance-api target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-resource-timing $(if $(FILTER),$(FILTER),)

testharness-navigation-timing: target-disk-guard fetch-wpt-performance-api target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-navigation-timing $(if $(FILTER),$(FILTER),)

testharness-event-timing: target-disk-guard fetch-wpt-performance-api target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-event-timing $(if $(FILTER),$(FILTER),)

testharness-longtask-timing: target-disk-guard fetch-wpt-performance-api target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-longtask-timing $(if $(FILTER),$(FILTER),)

# FileAPI goal M1 / DC-1（docs/goal/archive/fileapi-compat.md）：语料 fetch 走 goals/16
# 预置脚本；runner 可执行子集 skip 域见 testharness_fileapi.rs fileapi_case_skipped 文档。
# make testharness-fileapi FILTER=blob/。
fetch-wpt-fileapi:
	bash tests/wpt-runner/scripts/goals/16-fileapi-compat.sh

testharness-fileapi: target-disk-guard fetch-wpt-fileapi target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-fileapi $(if $(FILTER),$(FILTER),)

# Web Messaging goal M1 / DC-1（docs/goal/archive/webmessaging-compat.md）：语料 fetch 走
# goals/17 预置脚本；runner 可执行子集 skip 域（跨 document→navigation M3、
# worker→workers-compat）见 testharness_webmessaging.rs webmessaging_case_skipped
# 文档。make testharness-webmessaging FILTER=with-ports/。
fetch-wpt-webmessaging:
	bash tests/wpt-runner/scripts/goals/17-webmessaging-compat.sh

testharness-webmessaging: target-disk-guard fetch-wpt-webmessaging target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- ./target/release/zero-wpt-runner testharness-webmessaging $(if $(FILTER),$(FILTER),)

# Workers goal M1 导入 / M2 worker harness 通道（docs/goal/archive/workers-compat.md）：语料
# fetch 走 goals/70 预置脚本（webmessaging 先例）；runner 可执行子集 skip 域
# （SharedWorker/module/iframe/multi-global/module-worker 记账）与 .any.js 变体分道
# 见 testharness_workers.rs 文档。make testharness-workers FILTER=constructors/。
fetch-wpt-workers:
	bash tests/wpt-runner/scripts/goals/70-workers-compat.sh

testharness-workers: target-disk-guard fetch-wpt-workers target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-workers $(if $(FILTER),$(FILTER),)

# svg-compat goal M1 / DC-1：上游 svg/ testharness DOM 面基线（渲染断言面排除，
# 占比盘点入 evidence）。用例 gitignored（goals/80-svg-compat.sh GOAL_FETCH_DEEP=1 深挖拉取）。
# filter 按路径子串透传：make testharness-svg FILTER=types/
fetch-wpt-svg:
	bash tests/wpt-runner/scripts/goals/80-svg-compat.sh

testharness-svg: target-disk-guard fetch-wpt-svg target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),1800) -- ./target/release/zero-wpt-runner testharness-svg $(if $(FILTER),$(FILTER),)

# Service Worker WPT 分母：294 个 source 必须有唯一且可重建的执行 lane。
audit-wpt-service-workers-disposition:
	python3 tests/wpt-runner/scripts/audit-service-worker-disposition.py

# Service Worker M1 Tier A：仅恢复固定静态资产；runner/runtime 仍受 M0 RFC 审批门禁。
fetch-wpt-service-workers-tier-a:
	WPT_EXPECTED_ASSET_COUNT=19 $(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-tier-a:
	WPT_EXPECTED_ASSET_COUNT=19 $(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-tier-a-assets: fetch-wpt-service-workers-tier-a
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_EXPECTED_ASSET_COUNT=19 \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-next-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-19-m1-next-wave-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=9 WPT_CORPUS_LABEL="Service Worker next-wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-next-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-19-m1-next-wave-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=9 WPT_CORPUS_LABEL="Service Worker next-wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-next-wave-assets: fetch-wpt-service-workers-next-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-19-m1-next-wave-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=9 WPT_CORPUS_LABEL="Service Worker next-wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-message-lifecycle-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-31-m3-message-lifecycle-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=7 WPT_CORPUS_LABEL="Service Worker message lifecycle wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-message-lifecycle-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-31-m3-message-lifecycle-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=7 WPT_CORPUS_LABEL="Service Worker message lifecycle wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-message-lifecycle-wave-assets: fetch-wpt-service-workers-message-lifecycle-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-31-m3-message-lifecycle-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=7 WPT_CORPUS_LABEL="Service Worker message lifecycle wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/events-worker.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-message-event-ports-wave audit-wpt-service-workers-message-event-ports-wave test-wpt-service-workers-message-event-ports-wave-assets
fetch-wpt-service-workers-message-event-ports-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-message-event-ports-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=2 WPT_CORPUS_LABEL="Service Worker message event ports wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-message-event-ports-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-message-event-ports-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=2 WPT_CORPUS_LABEL="Service Worker message event ports wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-message-event-ports-wave-assets: fetch-wpt-service-workers-message-event-ports-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-message-event-ports-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=2 WPT_CORPUS_LABEL="Service Worker message event ports wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/ServiceWorkerGlobalScope/message-event-ports-worker.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-messageerror-event-wave audit-wpt-service-workers-messageerror-event-wave test-wpt-service-workers-messageerror-event-wave-assets
fetch-wpt-service-workers-messageerror-event-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-messageerror-event-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=2 WPT_CORPUS_LABEL="Service Worker messageerror event wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-messageerror-event-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-messageerror-event-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=2 WPT_CORPUS_LABEL="Service Worker messageerror event wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-messageerror-event-wave-assets: fetch-wpt-service-workers-messageerror-event-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-messageerror-event-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=2 WPT_CORPUS_LABEL="Service Worker messageerror event wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/ServiceWorkerGlobalScope/error-message-event-worker.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-controller-on-load-wave audit-wpt-service-workers-controller-on-load-wave test-wpt-service-workers-controller-on-load-wave-assets fetch-wpt-service-workers-controller-on-disconnect-wave audit-wpt-service-workers-controller-on-disconnect-wave test-wpt-service-workers-controller-on-disconnect-wave-assets fetch-wpt-service-workers-controller-on-reload-wave audit-wpt-service-workers-controller-on-reload-wave test-wpt-service-workers-controller-on-reload-wave-assets fetch-wpt-service-workers-extendable-event-async-waituntil-wave audit-wpt-service-workers-extendable-event-async-waituntil-wave test-wpt-service-workers-extendable-event-async-waituntil-wave-assets fetch-wpt-service-workers-extendable-event-waituntil-wave audit-wpt-service-workers-extendable-event-waituntil-wave test-wpt-service-workers-extendable-event-waituntil-wave-assets fetch-wpt-service-workers-onactivate-script-error-wave audit-wpt-service-workers-onactivate-script-error-wave test-wpt-service-workers-onactivate-script-error-wave-assets fetch-wpt-service-workers-oninstall-script-error-wave audit-wpt-service-workers-oninstall-script-error-wave test-wpt-service-workers-oninstall-script-error-wave-assets
fetch-wpt-service-workers-controller-on-load-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-controller-on-load-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker controller on load wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-controller-on-load-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-controller-on-load-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker controller on load wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-controller-on-load-wave-assets: fetch-wpt-service-workers-controller-on-load-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-controller-on-load-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker controller on load wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/controller-on-load.https.html" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-controller-on-disconnect-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-controller-on-disconnect-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker controller on disconnect wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-controller-on-disconnect-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-controller-on-disconnect-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker controller on disconnect wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-controller-on-disconnect-wave-assets: fetch-wpt-service-workers-controller-on-disconnect-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-controller-on-disconnect-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker controller on disconnect wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/controller-on-disconnect.https.html" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-controller-on-reload-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-controller-on-reload-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker controller on reload wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-controller-on-reload-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-controller-on-reload-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker controller on reload wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-controller-on-reload-wave-assets: fetch-wpt-service-workers-controller-on-reload-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-controller-on-reload-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker controller on reload wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/controller-on-reload.https.html" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-extendable-event-async-waituntil-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-extendable-event-async-waituntil-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=2 WPT_CORPUS_LABEL="Service Worker extendable event async waitUntil wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-extendable-event-async-waituntil-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-extendable-event-async-waituntil-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=2 WPT_CORPUS_LABEL="Service Worker extendable event async waitUntil wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-extendable-event-async-waituntil-wave-assets: fetch-wpt-service-workers-extendable-event-async-waituntil-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-extendable-event-async-waituntil-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=2 WPT_CORPUS_LABEL="Service Worker extendable event async waitUntil wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/extendable-event-async-waituntil.https.html" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-extendable-event-waituntil-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-extendable-event-waituntil-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=2 WPT_CORPUS_LABEL="Service Worker extendable event waitUntil wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-extendable-event-waituntil-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-extendable-event-waituntil-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=2 WPT_CORPUS_LABEL="Service Worker extendable event waitUntil wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-extendable-event-waituntil-wave-assets: fetch-wpt-service-workers-extendable-event-waituntil-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-extendable-event-waituntil-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=2 WPT_CORPUS_LABEL="Service Worker extendable event waitUntil wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/extendable-event-waituntil.https.html" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-onactivate-script-error-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-onactivate-script-error-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker onactivate script error wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-onactivate-script-error-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-onactivate-script-error-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker onactivate script error wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-onactivate-script-error-wave-assets: fetch-wpt-service-workers-onactivate-script-error-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-onactivate-script-error-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker onactivate script error wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/onactivate-script-error.https.html" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-oninstall-script-error-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-oninstall-script-error-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=7 WPT_CORPUS_LABEL="Service Worker oninstall script error wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-oninstall-script-error-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-oninstall-script-error-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=7 WPT_CORPUS_LABEL="Service Worker oninstall script error wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-oninstall-script-error-wave-assets: fetch-wpt-service-workers-oninstall-script-error-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-oninstall-script-error-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=7 WPT_CORPUS_LABEL="Service Worker oninstall script error wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/oninstall-script-error.https.html" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-getregistration-wave audit-wpt-service-workers-getregistration-wave test-wpt-service-workers-getregistration-wave-assets
fetch-wpt-service-workers-getregistration-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-getregistration-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker getRegistration wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-getregistration-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-getregistration-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker getRegistration wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-getregistration-wave-assets: fetch-wpt-service-workers-getregistration-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-getregistration-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker getRegistration wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/getregistration.https.html" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-registration-iframe-wave audit-wpt-service-workers-registration-iframe-wave test-wpt-service-workers-registration-iframe-wave-assets
fetch-wpt-service-workers-registration-iframe-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-registration-iframe-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker registration iframe wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-registration-iframe-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-registration-iframe-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker registration iframe wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-registration-iframe-wave-assets: fetch-wpt-service-workers-registration-iframe-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-registration-iframe-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker registration iframe wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/registration-iframe.https.html" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-installing-wave audit-wpt-service-workers-installing-wave test-wpt-service-workers-installing-wave-assets
fetch-wpt-service-workers-installing-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-installing-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker installing wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-installing-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-installing-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker installing wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-installing-wave-assets: fetch-wpt-service-workers-installing-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-installing-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker installing wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/installing.https.html" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-waiting-wave audit-wpt-service-workers-waiting-wave test-wpt-service-workers-waiting-wave-assets
fetch-wpt-service-workers-waiting-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-waiting-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker waiting wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-waiting-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-waiting-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker waiting wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-waiting-wave-assets: fetch-wpt-service-workers-waiting-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-03-m3-waiting-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker waiting wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/waiting.https.html" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-extendable-message-event-wave audit-wpt-service-workers-extendable-message-event-wave test-wpt-service-workers-extendable-message-event-wave-assets
fetch-wpt-service-workers-extendable-message-event-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-extendable-message-event-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker extendable message event wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-extendable-message-event-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-extendable-message-event-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker extendable message event wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-extendable-message-event-wave-assets: fetch-wpt-service-workers-extendable-message-event-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-extendable-message-event-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker extendable message event wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/ServiceWorkerGlobalScope/resources/extendable-message-event-ping-worker.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-registration-attribute-wave audit-wpt-service-workers-registration-attribute-wave test-wpt-service-workers-registration-attribute-wave-assets
fetch-wpt-service-workers-registration-attribute-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-registration-attribute-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=7 WPT_CORPUS_LABEL="Service Worker registration attribute wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-registration-attribute-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-registration-attribute-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=7 WPT_CORPUS_LABEL="Service Worker registration attribute wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-registration-attribute-wave-assets: fetch-wpt-service-workers-registration-attribute-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-registration-attribute-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=7 WPT_CORPUS_LABEL="Service Worker registration attribute wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/ServiceWorkerGlobalScope/resources/registration-attribute-worker.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-global-serviceworker-wave audit-wpt-service-workers-global-serviceworker-wave test-wpt-service-workers-global-serviceworker-wave-assets
fetch-wpt-service-workers-global-serviceworker-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-global-serviceworker-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=3 WPT_CORPUS_LABEL="Service Worker global serviceworker wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-global-serviceworker-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-global-serviceworker-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=3 WPT_CORPUS_LABEL="Service Worker global serviceworker wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-global-serviceworker-wave-assets: fetch-wpt-service-workers-global-serviceworker-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-global-serviceworker-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=3 WPT_CORPUS_LABEL="Service Worker global serviceworker wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/global-serviceworker.https.any.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-immutable-prototype-wave audit-wpt-service-workers-immutable-prototype-wave test-wpt-service-workers-immutable-prototype-wave-assets
fetch-wpt-service-workers-immutable-prototype-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-immutable-prototype-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker immutable prototype wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-immutable-prototype-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-immutable-prototype-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker immutable prototype wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-immutable-prototype-wave-assets: fetch-wpt-service-workers-immutable-prototype-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-immutable-prototype-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker immutable prototype wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/immutable-prototype-serviceworker.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-worker-global-unregister-wave audit-wpt-service-workers-worker-global-unregister-wave test-wpt-service-workers-worker-global-unregister-wave-assets
fetch-wpt-service-workers-worker-global-unregister-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-worker-global-unregister-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker worker-global unregister wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-worker-global-unregister-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-worker-global-unregister-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker worker-global unregister wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-worker-global-unregister-wave-assets: fetch-wpt-service-workers-worker-global-unregister-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-worker-global-unregister-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker worker-global unregister wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/ServiceWorkerGlobalScope/resources/unregister-worker.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-static-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-19-worker-global-static-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker static wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-static-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-19-worker-global-static-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker static wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-static-wave-assets: fetch-wpt-service-workers-static-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-19-worker-global-static-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker static wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-worker-secure-context-wave audit-wpt-service-workers-worker-secure-context-wave test-wpt-service-workers-worker-secure-context-wave-assets
fetch-wpt-service-workers-worker-secure-context-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-worker-secure-context-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker secure-context wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-worker-secure-context-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-worker-secure-context-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker secure-context wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-worker-secure-context-wave-assets: fetch-wpt-service-workers-worker-secure-context-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-worker-secure-context-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker secure-context wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/ServiceWorkerGlobalScope/isSecureContext.serviceworker.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-install-event-type-wave audit-wpt-service-workers-install-event-type-wave test-wpt-service-workers-install-event-type-wave-assets
fetch-wpt-service-workers-install-event-type-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-install-event-type-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=7 WPT_CORPUS_LABEL="Service Worker install-event-type wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-install-event-type-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-install-event-type-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=7 WPT_CORPUS_LABEL="Service Worker install-event-type wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-install-event-type-wave-assets: fetch-wpt-service-workers-install-event-type-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-install-event-type-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=7 WPT_CORPUS_LABEL="Service Worker install-event-type wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/install-event-type-worker.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-worker-close-wave audit-wpt-service-workers-worker-close-wave test-wpt-service-workers-worker-close-wave-assets
fetch-wpt-service-workers-worker-close-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-worker-close-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker close wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-worker-close-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-worker-close-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker close wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-worker-close-wave-assets: fetch-wpt-service-workers-worker-close-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-worker-close-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker close wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/ServiceWorkerGlobalScope/resources/close-worker.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-worker-interface-wave audit-wpt-service-workers-worker-interface-wave test-wpt-service-workers-worker-interface-wave-assets
fetch-wpt-service-workers-worker-interface-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-worker-interface-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker interface wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-worker-interface-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-worker-interface-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker interface wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-worker-interface-wave-assets: fetch-wpt-service-workers-worker-interface-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-worker-interface-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker interface wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/interface-requirements-worker.sub.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-fetch-event-historical-wave audit-wpt-service-workers-fetch-event-historical-wave test-wpt-service-workers-fetch-event-historical-wave-assets
fetch-wpt-service-workers-fetch-event-historical-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-fetch-event-historical-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=3 WPT_CORPUS_LABEL="Service Worker FetchEvent historical wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-fetch-event-historical-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-fetch-event-historical-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=3 WPT_CORPUS_LABEL="Service Worker FetchEvent historical wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-fetch-event-historical-wave-assets: fetch-wpt-service-workers-fetch-event-historical-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-fetch-event-historical-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=3 WPT_CORPUS_LABEL="Service Worker FetchEvent historical wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/historical.https.any.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-no-dynamic-import-wave audit-wpt-service-workers-no-dynamic-import-wave test-wpt-service-workers-no-dynamic-import-wave-assets
fetch-wpt-service-workers-no-dynamic-import-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-no-dynamic-import-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker no dynamic import wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-no-dynamic-import-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-no-dynamic-import-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker no dynamic import wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-no-dynamic-import-wave-assets: fetch-wpt-service-workers-no-dynamic-import-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-no-dynamic-import-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker no dynamic import wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/no-dynamic-import.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-module-no-dynamic-import-wave audit-wpt-service-workers-module-no-dynamic-import-wave test-wpt-service-workers-module-no-dynamic-import-wave-assets
fetch-wpt-service-workers-module-no-dynamic-import-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-module-no-dynamic-import-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker module no dynamic import wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-module-no-dynamic-import-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-module-no-dynamic-import-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker module no dynamic import wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-module-no-dynamic-import-wave-assets: fetch-wpt-service-workers-module-no-dynamic-import-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-09-02-m3-module-no-dynamic-import-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker module no dynamic import wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/basic-module-2.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-update-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker update wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-update-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker update wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-update-wave-assets: fetch-wpt-service-workers-update-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker update wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/empty.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-import-response-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-import-response-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker import response wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-import-response-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-import-response-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker import response wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-import-response-wave-assets: fetch-wpt-service-workers-import-response-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-import-response-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker import response wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/mime-type-worker.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-import-dynamic-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-import-dynamic-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=11 WPT_CORPUS_LABEL="Service Worker import dynamic wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-import-dynamic-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-import-dynamic-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=11 WPT_CORPUS_LABEL="Service Worker import dynamic wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-import-dynamic-wave-assets: fetch-wpt-service-workers-import-dynamic-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-import-dynamic-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=11 WPT_CORPUS_LABEL="Service Worker import dynamic wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/update-worker.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-import-event-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-import-event-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=3 WPT_CORPUS_LABEL="Service Worker import event wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-import-event-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-import-event-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=3 WPT_CORPUS_LABEL="Service Worker import event wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-import-event-wave-assets: fetch-wpt-service-workers-import-event-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-import-event-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=3 WPT_CORPUS_LABEL="Service Worker import event wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/import-scripts-echo.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-module-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker module wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-module-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker module wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-module-wave-assets: fetch-wpt-service-workers-module-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker module wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/scope1/redirect.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-module-bytecheck-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-bytecheck-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker module bytecheck wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-module-bytecheck-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-bytecheck-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker module bytecheck wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-module-bytecheck-wave-assets: fetch-wpt-service-workers-module-bytecheck-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-bytecheck-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=4 WPT_CORPUS_LABEL="Service Worker module bytecheck wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/bytecheck-worker.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-module-cors-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-cors-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker module CORS wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-module-cors-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-cors-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker module CORS wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-module-cors-wave-assets: fetch-wpt-service-workers-module-cors-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-cors-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=1 WPT_CORPUS_LABEL="Service Worker module CORS wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/update-bytecheck-cors-import.https.html" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-module-registration-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-registration-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker module registration wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-module-registration-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-registration-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker module registration wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-module-registration-wave-assets: fetch-wpt-service-workers-module-registration-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-registration-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker module registration wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/malformed-worker.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-module-type-update-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-type-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=8 WPT_CORPUS_LABEL="Service Worker module type update wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-module-type-update-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-type-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=8 WPT_CORPUS_LABEL="Service Worker module type update wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-module-type-update-wave-assets: fetch-wpt-service-workers-module-type-update-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-type-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=8 WPT_CORPUS_LABEL="Service Worker module type update wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/update-registration-with-type.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-module-request-metadata-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-request-metadata-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker module request metadata wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-module-request-metadata-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-request-metadata-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker module request metadata wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-module-request-metadata-wave-assets: fetch-wpt-service-workers-module-request-metadata-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-module-request-metadata-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker module request metadata wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/test-request-mode-worker.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-update-via-cache-matrix-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-update-via-cache-matrix-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=9 WPT_CORPUS_LABEL="Service Worker updateViaCache matrix wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-update-via-cache-matrix-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-update-via-cache-matrix-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=9 WPT_CORPUS_LABEL="Service Worker updateViaCache matrix wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-update-via-cache-matrix-wave-assets: fetch-wpt-service-workers-update-via-cache-matrix-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-20-m3-update-via-cache-matrix-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=9 WPT_CORPUS_LABEL="Service Worker updateViaCache matrix wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/update-max-aged-worker.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-dynamic-import-update-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-dynamic-import-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=17 WPT_CORPUS_LABEL="Service Worker dynamic import update wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-dynamic-import-update-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-dynamic-import-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=17 WPT_CORPUS_LABEL="Service Worker dynamic import update wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-dynamic-import-update-wave-assets: fetch-wpt-service-workers-dynamic-import-update-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-dynamic-import-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=17 WPT_CORPUS_LABEL="Service Worker dynamic import update wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/update-worker-from-file.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-update-failure-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-update-failure-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=12 WPT_CORPUS_LABEL="Service Worker update failure wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-update-failure-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-update-failure-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=12 WPT_CORPUS_LABEL="Service Worker update failure wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-update-failure-wave-assets: fetch-wpt-service-workers-update-failure-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-update-failure-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=12 WPT_CORPUS_LABEL="Service Worker update failure wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/update-worker.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

fetch-wpt-service-workers-multiple-update-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-multiple-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker multiple update wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-multiple-update-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-multiple-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker multiple update wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-multiple-update-wave-assets: fetch-wpt-service-workers-multiple-update-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-multiple-update-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker multiple update wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/update-nocookie-worker.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-update-not-allowed-wave audit-wpt-service-workers-update-not-allowed-wave test-wpt-service-workers-update-not-allowed-wave-assets
fetch-wpt-service-workers-update-not-allowed-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-update-not-allowed-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker update not allowed wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-update-not-allowed-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-update-not-allowed-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker update not allowed wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-update-not-allowed-wave-assets: fetch-wpt-service-workers-update-not-allowed-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-update-not-allowed-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=6 WPT_CORPUS_LABEL="Service Worker update not allowed wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/update-during-installation-worker.py" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-skip-waiting-no-client-wave audit-wpt-service-workers-skip-waiting-no-client-wave test-wpt-service-workers-skip-waiting-no-client-wave-assets
fetch-wpt-service-workers-skip-waiting-no-client-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-skip-waiting-no-client-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=9 WPT_CORPUS_LABEL="Service Worker skipWaiting no-client wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-skip-waiting-no-client-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-skip-waiting-no-client-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=9 WPT_CORPUS_LABEL="Service Worker skipWaiting no-client wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-skip-waiting-no-client-wave-assets: fetch-wpt-service-workers-skip-waiting-no-client-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-skip-waiting-no-client-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=9 WPT_CORPUS_LABEL="Service Worker skipWaiting no-client wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/skip-waiting-worker.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-clients-matchall-evaluation-wave audit-wpt-service-workers-clients-matchall-evaluation-wave test-wpt-service-workers-clients-matchall-evaluation-wave-assets
fetch-wpt-service-workers-clients-matchall-evaluation-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-clients-matchall-evaluation-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker clients.matchAll evaluation wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-clients-matchall-evaluation-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-clients-matchall-evaluation-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker clients.matchAll evaluation wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-clients-matchall-evaluation-wave-assets: fetch-wpt-service-workers-clients-matchall-evaluation-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-21-m3-clients-matchall-evaluation-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=5 WPT_CORPUS_LABEL="Service Worker clients.matchAll evaluation wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/clients-matchall-on-evaluation-worker.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

.PHONY: fetch-wpt-service-workers-fetch-wave audit-wpt-service-workers-fetch-wave test-wpt-service-workers-fetch-wave-assets testharness-service-workers-fetch baseline-wpt-service-workers-fetch
fetch-wpt-service-workers-fetch-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-22-m2-fetch-request-end-to-end-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=80 WPT_CORPUS_LABEL="Service Worker fetch wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-fetch-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-22-m2-fetch-request-end-to-end-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=80 WPT_CORPUS_LABEL="Service Worker fetch wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-fetch-wave-assets: fetch-wpt-service-workers-fetch-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-22-m2-fetch-request-end-to-end-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=80 WPT_CORPUS_LABEL="Service Worker fetch wave" \
		WPT_TAMPER_ASSET="service-workers/service-worker/resources/fetch-event-async-respond-with-worker.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

testharness-service-workers-core: target-disk-guard fetch-wpt-service-workers-tier-a fetch-wpt-service-workers-next-wave fetch-wpt-service-workers-message-lifecycle-wave fetch-wpt-service-workers-message-event-ports-wave fetch-wpt-service-workers-messageerror-event-wave fetch-wpt-service-workers-controller-on-load-wave fetch-wpt-service-workers-controller-on-disconnect-wave fetch-wpt-service-workers-controller-on-reload-wave fetch-wpt-service-workers-extendable-event-async-waituntil-wave fetch-wpt-service-workers-extendable-event-waituntil-wave fetch-wpt-service-workers-onactivate-script-error-wave fetch-wpt-service-workers-oninstall-script-error-wave fetch-wpt-service-workers-getregistration-wave fetch-wpt-service-workers-registration-iframe-wave fetch-wpt-service-workers-installing-wave fetch-wpt-service-workers-waiting-wave fetch-wpt-service-workers-extendable-message-event-wave fetch-wpt-service-workers-registration-attribute-wave fetch-wpt-service-workers-global-serviceworker-wave fetch-wpt-service-workers-immutable-prototype-wave fetch-wpt-service-workers-worker-global-unregister-wave fetch-wpt-service-workers-static-wave fetch-wpt-service-workers-worker-secure-context-wave fetch-wpt-service-workers-install-event-type-wave fetch-wpt-service-workers-worker-close-wave fetch-wpt-service-workers-worker-interface-wave fetch-wpt-service-workers-fetch-event-historical-wave fetch-wpt-service-workers-no-dynamic-import-wave fetch-wpt-service-workers-module-no-dynamic-import-wave fetch-wpt-service-workers-update-wave fetch-wpt-service-workers-import-response-wave fetch-wpt-service-workers-import-dynamic-wave fetch-wpt-service-workers-import-event-wave fetch-wpt-service-workers-module-wave fetch-wpt-service-workers-module-bytecheck-wave fetch-wpt-service-workers-module-cors-wave fetch-wpt-service-workers-module-registration-wave fetch-wpt-service-workers-module-type-update-wave fetch-wpt-service-workers-update-via-cache-matrix-wave fetch-wpt-service-workers-dynamic-import-update-wave fetch-wpt-service-workers-update-failure-wave fetch-wpt-service-workers-multiple-update-wave fetch-wpt-service-workers-update-not-allowed-wave fetch-wpt-service-workers-skip-waiting-no-client-wave fetch-wpt-service-workers-clients-matchall-evaluation-wave target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- \
		./target/release/zero-wpt-runner testharness-service-workers \
		--wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root $(if $(FILTER),$(FILTER),)

testharness-service-workers-fetch: target-disk-guard fetch-wpt-service-workers-fetch-wave target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- \
		./target/release/zero-wpt-runner testharness-service-workers-fetch \
		--wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root $(if $(FILTER),$(FILTER),)

.PHONY: fetch-wpt-service-workers-cache-storage-wave audit-wpt-service-workers-cache-storage-wave test-wpt-service-workers-cache-storage-wave-assets testharness-service-workers-cache-storage baseline-wpt-service-workers-cache-storage
fetch-wpt-service-workers-cache-storage-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=52 WPT_CORPUS_LABEL="Service Worker CacheStorage wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh

audit-wpt-service-workers-cache-storage-wave:
	WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=52 WPT_CORPUS_LABEL="Service Worker CacheStorage wave" \
		$(WPT_BASH) tests/wpt-runner/scripts/fetch-service-workers-tier-a.sh --verify-only

test-wpt-service-workers-cache-storage-wave-assets: fetch-wpt-service-workers-cache-storage-wave
	WPT_SERVICE_WORKER_SOURCE="$(CURDIR)/tests/wpt-runner/wpt-data/.service-workers-tier-a-root" \
		WPT_ASSET_MANIFEST="$(CURDIR)/docs/goal/archive/service-workers/evidence/2026-08-23-m2-cache-storage-serviceworker-assets.tsv" \
		WPT_EXPECTED_ASSET_COUNT=52 WPT_CORPUS_LABEL="Service Worker CacheStorage wave" \
		WPT_TAMPER_ASSET="service-workers/cache-storage/script-tests/cache-storage.js" \
		$(WPT_BASH) tests/wpt-runner/scripts/test-service-workers-tier-a-assets.sh

testharness-service-workers-cache-storage: target-disk-guard fetch-wpt-service-workers-cache-storage-wave target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- \
		./target/release/zero-wpt-runner testharness-service-workers-cache-storage \
		--wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root $(if $(FILTER),$(FILTER),)

baseline-wpt-service-workers-core: target-disk-guard fetch-wpt-service-workers-tier-a fetch-wpt-service-workers-next-wave fetch-wpt-service-workers-message-lifecycle-wave fetch-wpt-service-workers-message-event-ports-wave fetch-wpt-service-workers-messageerror-event-wave fetch-wpt-service-workers-controller-on-load-wave fetch-wpt-service-workers-controller-on-disconnect-wave fetch-wpt-service-workers-controller-on-reload-wave fetch-wpt-service-workers-extendable-event-async-waituntil-wave fetch-wpt-service-workers-extendable-event-waituntil-wave fetch-wpt-service-workers-onactivate-script-error-wave fetch-wpt-service-workers-oninstall-script-error-wave fetch-wpt-service-workers-getregistration-wave fetch-wpt-service-workers-registration-iframe-wave fetch-wpt-service-workers-installing-wave fetch-wpt-service-workers-waiting-wave fetch-wpt-service-workers-extendable-message-event-wave fetch-wpt-service-workers-registration-attribute-wave fetch-wpt-service-workers-global-serviceworker-wave fetch-wpt-service-workers-immutable-prototype-wave fetch-wpt-service-workers-worker-global-unregister-wave fetch-wpt-service-workers-static-wave fetch-wpt-service-workers-worker-secure-context-wave fetch-wpt-service-workers-install-event-type-wave fetch-wpt-service-workers-worker-close-wave fetch-wpt-service-workers-worker-interface-wave fetch-wpt-service-workers-fetch-event-historical-wave fetch-wpt-service-workers-no-dynamic-import-wave fetch-wpt-service-workers-module-no-dynamic-import-wave fetch-wpt-service-workers-update-wave fetch-wpt-service-workers-import-response-wave fetch-wpt-service-workers-import-dynamic-wave fetch-wpt-service-workers-import-event-wave fetch-wpt-service-workers-module-wave fetch-wpt-service-workers-module-bytecheck-wave fetch-wpt-service-workers-module-cors-wave fetch-wpt-service-workers-module-registration-wave fetch-wpt-service-workers-module-type-update-wave fetch-wpt-service-workers-module-request-metadata-wave fetch-wpt-service-workers-update-via-cache-matrix-wave fetch-wpt-service-workers-dynamic-import-update-wave fetch-wpt-service-workers-update-failure-wave fetch-wpt-service-workers-multiple-update-wave fetch-wpt-service-workers-update-not-allowed-wave fetch-wpt-service-workers-skip-waiting-no-client-wave fetch-wpt-service-workers-clients-matchall-evaluation-wave target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),900) -- \
		python3 tests/wpt-runner/scripts/run-service-workers-core-baseline.py \
		--runner ./target/release/zero-wpt-runner \
		--wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root $(if $(OUTPUT),--output $(OUTPUT),) $(if $(SUMMARY),--summary $(SUMMARY),)

baseline-wpt-service-workers-fetch: target-disk-guard fetch-wpt-service-workers-fetch-wave target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- \
		python3 tests/wpt-runner/scripts/run-service-workers-fetch-baseline.py \
		--runner ./target/release/zero-wpt-runner \
		--wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root $(if $(OUTPUT),--output $(OUTPUT),) $(if $(SUMMARY),--summary $(SUMMARY),)

baseline-wpt-service-workers-cache-storage: target-disk-guard fetch-wpt-service-workers-cache-storage-wave target/test-guard zero-wpt-runner-release
	./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit $(or $(TIME_LIMIT),300) -- \
		python3 tests/wpt-runner/scripts/run-service-workers-cache-storage-baseline.py \
		--runner ./target/release/zero-wpt-runner \
		--wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root $(if $(OUTPUT),--output $(OUTPUT),) $(if $(SUMMARY),--summary $(SUMMARY),)

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
	cargo build --release -p zero-renderer -p zero-compositor -p zero-image-decoder -p zero-media-host -p zero-wpt-runner
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
# Android 工具链默认布局：一律相对 $HOME（路径通用化）。env 供 preflight 与
# 构建脚本用；gradle 侧另走 apps/android-browser/local.properties，两者并存。
# 2026-09-12 起 android 构建入口纳入 test-guard 包裹（无人值守规则，run-rules #14）；
# 阈值较常规入口放宽到 8/16GB：gradle JVM 与 rustc release 峰值合法高于 4/8 档。
export ANDROID_HOME ?= $(HOME)/Android/Sdk

android-preflight:
	@test -d "$$ANDROID_HOME/platforms/android-36" || (echo "Android SDK platform 36 is required under $$ANDROID_HOME"; exit 2)
	@rustup target list --installed | grep -qx aarch64-linux-android
	@rustup target list --installed | grep -qx x86_64-linux-android
	@command -v cargo-ndk >/dev/null

android-apk: android-preflight target-disk-guard target/test-guard
	./target/test-guard --time-limit 1800 --per-proc-mem 8 --total-mem 16 -- sh -c 'cd apps/android-browser && ./gradlew --no-daemon :app:assembleEmulatorDebug'

android-release-apk: android-preflight target-disk-guard target/test-guard
	./target/test-guard --time-limit 1800 --per-proc-mem 8 --total-mem 16 -- sh -c 'cd apps/android-browser && ./gradlew --no-daemon :app:assembleArm64Release'

# renderer feature（V8 从源码）路径：gradle 检测到非 Windows 直调 build-native-wsl.sh。
# 前置环境六项（脚本自校验，缺一即败）：ZERO_V8_SOURCE / ZERO_CHROMIUM_CLANG /
# ZERO_ANDROID_NDK / LIBCLANG_PATH / ZERO_V8_GN / ZERO_V8_NINJA；网络全程需代理。
# 冷缓存首跑 V8 ninja 编译约 30-40min，故墙钟给 3600s。
android-renderer-apk: android-preflight target-disk-guard target/test-guard
	./target/test-guard --time-limit 3600 --per-proc-mem 8 --total-mem 16 -- sh -c 'cd apps/android-browser && ./gradlew --no-daemon -PuseWslRenderer :app:assembleArm64Release'

# 安装冒烟（M3/M5 切片 12）：Linux bash 移植自 Windows ps1（四进程拓扑/UID 隔离/
# probe 断言）。前置 = 运行中的模拟器/真机（KVM 或设备，见 evidence/emulator-feasibility.md）。
android-install-smoke: android-apk
	./scripts/android/install-smoke.sh apps/android-browser/app/build/outputs/apk/emulator/debug/app-emulator-debug.apk

# chaos 冒烟（M5 切片 18）：角色进程 kill → 断连恢复重绑断言。
# 前置 = 应用已运行（依赖 android-install-smoke）+ root 可用（模拟器 google_apis 镜像）。
android-chaos-smoke: android-install-smoke
	./scripts/android/chaos-smoke.sh
endif

# ── cdp-protocol goal：Playwright E2E 收口门（M5）──
# 零源码依赖的 Node 测试链（tests/playwright-matrix/）：spawn 独立 headless 双跑
# 全核心流 → deterministic 判定 + expected-green 回归门（objectId 桥待决策，
# evaluate/locator 族步骤不在基线内，见 tests/playwright-matrix/expected-green.json）。
# 前置：node >= 20 + tests/playwright-matrix/node_modules（npm install）。
cdp-e2e: target-disk-guard target/test-guard
	cargo build -p zero-browser --bin zero-browser
	cd tests/playwright-matrix && ZERO_NOPROXY=1 ZW_IPC_VALIDATE=1 ../../target/test-guard --time-limit 600 --per-proc-mem 4 --total-mem 8 -- node scripts/verify-deterministic.mjs
