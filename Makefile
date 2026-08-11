.PHONY: setup-rusty-v8 fetch-wpt-data update-wpt-data build browser browser-cpu browser-wpt-parity browser-debug browser-debug-wayland browser-debug-wayland-log browser-debug-x11 browser-compositor-smoke browser-compositor-real-site-smoke test reftest reftest-oracle capture-oracle product-smoke-oracle product-smoke product-smoke-legacy import-wpt reftest-trend reftest-trend-oracle reftest-smoke layout-golden layout-golden-update monthly-report bench bench-gate bench-capture bench-trend

setup-rusty-v8:
	bash scripts/download-rusty-v8.sh

# WPT reftest 数据（上游 web-platform-tests/wpt 子集，~19952 文件，独立 repo）。
# reftest / reftest-oracle 会自动前置触发；目录已存在则跳过，刷新需先 rm -rf。
WPT_DATA_REPO ?= https://github.com/leizongmin/zeroweb-wpt-data.git
WPT_DATA_REF  ?= v1.10
WPT_DATA_DIR  ?= tests/wpt-runner/wpt-data
fetch-wpt-data:
	@if [ -d "$(WPT_DATA_DIR)" ] && [ -n "$$(ls -A $(WPT_DATA_DIR) 2>/dev/null)" ]; then echo "wpt-data 已存在 ($(WPT_DATA_DIR), ref=$(WPT_DATA_REF))；刷新请先 rm -rf 该目录"; else echo "fetch wpt-data $(WPT_DATA_REF) → $(WPT_DATA_DIR)"; git clone --depth=1 --branch $(WPT_DATA_REF) $(WPT_DATA_REPO) "$(WPT_DATA_DIR)"; rm -rf "$(WPT_DATA_DIR)/.git"; fi
	@bash scripts/fetch-wpt-smoke-subdirs.sh
	@bash tests/wpt-runner/scripts/sync-imported-resources.sh

# 升级 wpt-data 套件到新 tag（A2：套件随上游滚动，否则通过率无法对比）。
# 用法: make update-wpt-data REF=v2.0        升级到指定 tag
#       make update-wpt-data CHECK=1         查看远端可用 tag（只读）
update-wpt-data:
	bash scripts/update-wpt-data.sh $(if $(CHECK),--check,$(REF))

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

# 真实产品窗口 smoke：CPU/scale=1 下串行运行 legacy 与 compositor 两种模式，
# 由最终 softbuffer framebuffer 写 PNG，不依赖系统截图或无障碍权限。
# 构建、两个进程链和全部断言都受 test-guard 内存门禁与 900 秒墙钟保护。
browser-compositor-smoke: target/test-guard
	./target/test-guard --time-limit 900 -- sh scripts/browser-compositor-smoke.sh

# 可选真实网站 GUI 验收：打开 HTTPS 页面并依次滚动、缩放、刷新，保存四张最终
# framebuffer 截图并断言 compositor 全程健康。默认 URL 可用 GUI_SMOKE_URL 覆盖。
# 此 target 故意不接入 make test，避免网络和真实窗口成为常规单测前置条件。
browser-compositor-real-site-smoke: target/test-guard
	./target/test-guard --time-limit 900 -- sh scripts/browser-compositor-real-site-smoke.sh

# ── 测试防护 (test-guard) ──────────────────────────────────────────────
# test-guard 跨平台 (macOS/Linux) 包裹测试命令：单进程 RSS>6GB 或全树>16GB
# 或总时长>1800s 即杀掉整棵进程树（退出 124），防止内存型 bug（如无限循环
# realloc、CSS parser 未闭合括号死循环）触发系统级 OOM 连累 tmux session /
# rally 无人值守流程。源码 scripts/test-guard.rs，std-only，rustc 直接编译。
ifeq ($(OS),Windows_NT)
MKDIR_TARGET = if not exist target mkdir target
else
MKDIR_TARGET = mkdir -p target
endif

target/test-guard: scripts/test-guard.rs
	@$(MKDIR_TARGET)
	rustc -O scripts/test-guard.rs -o target/test-guard

# 全量测试（被 test-guard 包裹）。无人值守 / rally / CI 请用此 target，
# 不要裸跑 cargo test。可调阈值：./target/test-guard --per-proc-mem 8 --total-mem 20 -- cargo test --workspace
# 2026-08-08：纳入 QuickJS 矩阵（v8/quickjs 接口一致性保证——此前 quickjs 只在 CI，
# 本地提交门禁覆盖不到，编译/运行破坏 CI 才暴露；QuickJS_CRATES 为 CI quickjs 测试包列表）。
QUICKJS_CLIPPY_CRATES = zero-dom zero-css-parser zero-style-system zero-layout-engine zero-engine zero-canvas zero-host-runtime zero-net zero-security zero-storage zero-protocol zero-wasm-sandbox zero-page-runtime zero-render-foundation
QUICKJS_TEST_CRATES = zero-script-sandbox zero-webview zero-browser zero-renderer zero-webview-demo zero-integration-tests zero-wpt-runner
QUICKJS_TEST_CRATES_WITHOUT_BROWSER = $(filter-out zero-browser,$(QUICKJS_TEST_CRATES))
ifeq ($(OS),Windows_NT)
# Windows GUI 测试共享进程级 compositor；并行执行会让测试互相关闭其子进程。
test: target/test-guard
	.\target\test-guard --per-proc-mem 10 --total-mem 28 --time-limit 900 -- cargo test --workspace --exclude zero-browser
	.\target\test-guard --per-proc-mem 10 --total-mem 28 --time-limit 900 -- cargo test -p zero-browser --bin zero-browser -- --test-threads=1
	.\target\test-guard --per-proc-mem 10 --total-mem 28 -- cargo clippy --no-default-features --features quickjs $(addprefix -p ,$(QUICKJS_CLIPPY_CRATES)) --all-targets -- -D warnings
	.\target\test-guard --per-proc-mem 10 --total-mem 28 --time-limit 900 -- cargo test --no-default-features --features quickjs $(addprefix -p ,$(QUICKJS_TEST_CRATES_WITHOUT_BROWSER))
	.\target\test-guard --per-proc-mem 10 --total-mem 28 --time-limit 900 -- cargo test --no-default-features --features quickjs -p zero-browser -- --test-threads=1
else
test: target/test-guard
	# cargo test 执行器（2026-08-09 从 nextest 换回——字体共享后评估反转）：
	# - nextest 每测试独立进程 → 每测试进程重复解析 19MB CJK 字体（~3s/进程），
	#   实测 zero-wpt-runner 45s / zero-browser 30s；cargo test 每二进制 1 进程
	#   （进程内并行），字体每二进制只付 1 次 → 同两包 3.4s / 3.4s，全量
	#   v8 阶段 47.3s（nextest 68s，-30%），且与 CI（cargo test --workspace）
	#   覆盖口径一致。历史评估（2026-08-07 nextest 1m29s vs cargo test 1m58s）
	#   未计入字体缓存与「每测试进程」的相互作用，已被推翻。
	# - wgpu headless 挂起（本地无 GPU 后端）由 test-guard --time-limit 900 兜底
	#   （正常全量 ~50s；挂起 15min 杀进程树，替代 nextest slow-timeout）。
	# - 并行化：QuickJS clippy（编译型）与 v8 测试并行跑——clippy 编译的是
	#   quickjs feature 组合产物（与 v8 产物不冲突），cargo 各自持锁；v8 测试
	#   （~50s）时长覆盖 clippy 编译，总时长省一个编译段。test-guard 两个实例
	#   独立监控各自进程树（阈值各自生效，不叠加）。
	./target/test-guard --per-proc-mem 10 --total-mem 28 --time-limit 900 -- cargo test --workspace & test_pid=$$!; \
	./target/test-guard --per-proc-mem 10 --total-mem 28 -- cargo clippy --no-default-features --features quickjs $(addprefix -p ,$(QUICKJS_CLIPPY_CRATES)) --all-targets -- -D warnings & clippy_pid=$$!; \
	rc=0; wait $$test_pid || rc=$$?; wait $$clippy_pid || rc=$$?; exit $$rc
	# QuickJS 运行测试（v8/quickjs 接口一致性保证）
	./target/test-guard --per-proc-mem 10 --total-mem 28 --time-limit 900 -- cargo test --no-default-features --features quickjs $(addprefix -p ,$(QUICKJS_TEST_CRATES))
endif

# WPT reftest（release 构建，约 4× 快于 debug；同样被 test-guard 包裹）。
reftest: fetch-wpt-data target/test-guard
	./target/test-guard -- cargo run --release --bin zero-wpt-runner -- reftest

# 上游 WPT reftest（wpt-data/，self-source 同源 ref）。test-guard 包裹（OOM 防护）。
# 全量 ~16600 案（2026-08-07 @font-face loader 缓存后）实测 ~25s，远低于
# test-guard 默认 1800s 超时；--time-limit 3600 保留作兜底（不匹配 OOM 死循环）。
# 用法: make reftest-upstream                     全量上游（快，~25s）
#       make reftest-upstream FILTER=css-tables   单目录/子串过滤（case.id.contains）
#       make reftest-upstream FILTER=css/CSS2/backgrounds
reftest-upstream: fetch-wpt-data target/test-guard
	./target/test-guard --time-limit 3600 -- cargo run --release --bin zero-wpt-runner -- reftest-upstream $(FILTER)

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
PRODUCT_ORACLE_SCRIPT := tests/wpt-runner/scripts/product-oracle-shot.mjs
product-smoke-oracle: target/test-guard
	@test -d tests/wpt-runner/scripts/node_modules/puppeteer-core || (echo "Error: puppeteer-core is missing; run 'npm ci --prefix tests/wpt-runner/scripts' first."; exit 2)
	./target/test-guard -- node $(PRODUCT_ORACLE_SCRIPT) --root apps/browser/assets --html welcome.html --out $(WELCOME_ORACLE) --width 800 --height 600

product-smoke: target/test-guard
	@test -f $(WELCOME_ORACLE) || (echo "Error: missing $(WELCOME_ORACLE); run 'make product-smoke-oracle' and commit the generated oracle."; exit 2)
	# DC-13 desktop（800px）：欢迎页 vs chromium Oracle diff≤20% + 结构门。--struct-check 含
	# sibling-overlap + collapsed-container + **text-concatenation**（R109 inline-ownership 守，
	# DC-13 line 325「sibling card/link/shortcut 文本不串联」）。计数断言覆盖 line 324 桌面须验证的
	# 四个 feature card（card:4）+ 六个快捷键（shortcut:6）+ 四个快速访问（link-tile:4）；
	# 行数断言守标题不拆行（title:1）+ tagline 2 行（tagline:2）。
	./target/test-guard -- cargo run --release --bin zero-wpt-runner -- product-smoke $(WELCOME_HTML) --oracle $(WELCOME_ORACLE) --max-diff $(or $(MAX_DIFF),20) --struct-check --expect-class card:4 --expect-class shortcut:6 --expect-class link-tile:4 --expect-class footer:1 --expect-lines title:1 --expect-lines tagline:2
	# DC-13 desktop wintertc（800px）：四个 nav button（bg-orange-500:4）+ **--check-img-visibility**
	#（R1598 守 14 个 header/参与方 logo 不塌缩，R1578b 谱系）+ **--expect-lines-min text-justify:2**
	#（line 327 正文按宽度换行并 justify）。struct-check 含 text-concatenation 守标题/副标题不串联。
	./target/test-guard -- cargo run --release --bin zero-wpt-runner -- product-smoke apps/browser/assets/wintertc/index.html --base-dir apps/browser/assets/wintertc --struct-check --expect-class bg-orange-500:4 --check-img-visibility --expect-lines-min text-justify:2
	# DC-13 desktop morning（800px）：article 结构 + 三个 tag badge（item-tag:3，line 326）+
	# pre/code 块在位（lang-bash:1，line 326 pre/code 独立背景换行）。struct-check 含 concat 守
	# nav/title/date/tag badge 不串联 + 正文段落不压一行 + table 不塌缩。morning 故意缺 cc_unavailable
	# 图测 alt 回退 → 不启用 --check-img-visibility（否则误报）。
	./target/test-guard -- cargo run --release --bin zero-wpt-runner -- product-smoke apps/browser/assets/morning-work/article.html --base-dir apps/browser/assets/morning-work --struct-check --expect-class article:1 --expect-class item-tag:3 --expect-class lang-bash:1
	# DC-13 goal line 322：窄屏 viewport（375px）结构门——窄宽逼长段落换行，暴露桌面宽不触发的
	# 重叠（R1498 morning @375 `<p>` 长高重叠后续 `<table>` 即此门抓到）。无 oracle（窄屏 oracle
	# 未抓），仅 struct-check 退码 3。--expect-class article:1 守 R1499 labels 修复（disqus
	# loadDisqus() appendChild 致 mutated_html ≠ 原 html，labels 须从 mutated_html 建才匹配 layout）。
	./target/test-guard -- cargo run --release --bin zero-wpt-runner -- product-smoke apps/browser/assets/morning-work/article.html --base-dir apps/browser/assets/morning-work --width 375 --struct-check --expect-class article:1
	# wintertc @375：nav button + logo 可见（logo 固定 px 高度窄宽不塌缩，--check-img-visibility 守 14 logo）。
	./target/test-guard -- cargo run --release --bin zero-wpt-runner -- product-smoke apps/browser/assets/wintertc/index.html --base-dir apps/browser/assets/wintertc --width 375 --struct-check --expect-class bg-orange-500:4 --check-img-visibility
	# DC-13 goal line 324「至少覆盖桌面和窄屏两个 viewport」：welcome 窄屏（375/320）结构门。
	# welcome 无 width 媒体查询，grids 保持 2 列，card:4 在窄宽仍成立；标题/tagline 在窄宽会合法
	# 换行故不强行断言行数。struct-check 含 text-concatenation 守窄宽下卡片/链接文本不串联。
	./target/test-guard -- cargo run --release --bin zero-wpt-runner -- product-smoke $(WELCOME_HTML) --width 375 --struct-check --expect-class card:4
	./target/test-guard -- cargo run --release --bin zero-wpt-runner -- product-smoke $(WELCOME_HTML) --width 320 --struct-check --expect-class card:4
	# DC-13 最窄 viewport（320px）结构门——守 R1502（split-gate article/disqus Flex-兄弟位移）+
	# R1503（sub-pixel sliver 高度过滤）。@320 比 @375 更逼换行，曾暴露 article/disqus 32400px² 重叠。
	./target/test-guard -- cargo run --release --bin zero-wpt-runner -- product-smoke apps/browser/assets/morning-work/article.html --base-dir apps/browser/assets/morning-work --width 320 --struct-check --expect-class article:1

# 测试资产化（P1）：把单个上游 WPT reftest 用例导入常驻断言集。
# 文件本体进入 wpt-data/（独立 repo），条目追加到 imported-tests.txt 账本，
# manifest 重新生成。每次渲染兼容性修复都应附带导入对应用例（见
# docs/goal/rendering-compat.md DC-7「测试资产化」）。
# 用法: make import-wpt TEST=css/CSS2/text/text-align-001.xht REF=css/CSS2/text/text-align-001-ref.xht NOTE="R21xx 修复"
#       make import-wpt ... EXTRA="--resource css/path/font.ttf"
import-wpt: fetch-wpt-data
	bash tests/wpt-runner/scripts/import-wpt-reftests.sh --add $(TEST) $(REF) $(if $(NOTE),--note "$(NOTE)") $(EXTRA)

# WPT 趋势基线（P2）：跑上游 reftest 全量，把绝对数追加到
# docs/goal/rendering-compat/evidence/wpt-trends/trend.csv（test-guard 包裹）。
# oracle 变体记录 DC-14 credible pass（需先 make capture-oracle 生成 oracle-shots）。
# 用法: make reftest-trend [NOTE="R21xx 修复后"]
#       make reftest-trend-oracle [NOTE="..."]
reftest-trend: fetch-wpt-data target/test-guard
	./target/test-guard -- bash scripts/record-wpt-trend.sh $(if $(NOTE),--note "$(NOTE)")

reftest-trend-oracle: fetch-wpt-data target/test-guard
	./target/test-guard -- bash scripts/record-wpt-trend.sh --oracle $(if $(NOTE),--note "$(NOTE)")

# Reftest smoke 分层门禁（B2）：跑 reftest-smoke.txt 清单（已知通过的
# 代表性 case，秒级），用作 PR CI 快门禁；全量留给 reftest / reftest-trend。
# 清单填充：从全量通过结果中挑代表性 case 写入 tests/wpt-runner/reftest-smoke.txt。
reftest-smoke: fetch-wpt-data target/test-guard
	./target/test-guard -- bash scripts/run-reftest-smoke.sh

# 布局树 dump golden 回归（B1/P3）：渲染测试页 → dump 布局树 → 与 golden 对比。
# golden 存 tests/wpt-runner/layout-golden/（提交进 git，测试资产化）。
# 用法: make layout-golden [FILTER=css/CSS2/backgrounds]   对比（diff 退出 1）
#       make layout-golden-update [FILTER=...]             生成/更新 golden
#       （新用例先 --update 生成基线并提交，此后作为布局回归常驻断言）
layout-golden: fetch-wpt-data target/test-guard
	./target/test-guard -- bash scripts/run-layout-golden.sh $(FILTER)

layout-golden-update: fetch-wpt-data target/test-guard
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
bench: target/test-guard
	./target/test-guard -- bash scripts/bench-report.sh

bench-gate: target/test-guard
	./target/test-guard -- bash scripts/bench-report.sh && bash scripts/perf-gate.sh

bench-capture: target/test-guard
	@test -n "$(JUSTIFICATION)" || (echo "bench-capture: JUSTIFICATION=... 必填（基线变更须有理由）"; exit 2)
	./target/test-guard -- bash scripts/bench-report.sh && bash scripts/record-bench-baseline.sh --justification "$(JUSTIFICATION)"

bench-trend: target/test-guard
	./target/test-guard -- bash scripts/bench-report.sh && bash scripts/record-bench-trend.sh $(if $(NOTE),--note "$(NOTE)")

# Legacy Static Web smoke（DC-13，goal rendering-compat.md line 316）：跑 20 页
# HTML 3.2/4 + CSS1/2 静态 fixture，每页 chromium oracle vs ZeroWeb CPU diff%。
# ★ trend-only（退出 0）——diff 全归因字体墙（fontdue 行度量 vs chromium NotoSansCJK
# 垂直漂移 + AA，R633 多会话 plateau），非回归；像素阈值作趋势指标，不替代 WPT/DC-14
# 达标口径（goal line 318）。新增 fixture 写入 evidence/product-static/legacy-html/。
# 用法：make product-smoke-legacy
LEGACY_DIR := docs/goal/rendering-compat/evidence/product-static/legacy-html
product-smoke-legacy: target/test-guard
	bash $(LEGACY_DIR)/run-all.sh
