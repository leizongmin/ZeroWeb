.PHONY: setup-rusty-v8 build browser browser-debug browser-debug-wayland browser-debug-wayland-log browser-debug-x11 test reftest

setup-rusty-v8:
	bash scripts/download-rusty-v8.sh

build: setup-rusty-v8
	cargo build --workspace

# WAYLAND_DEBUG and WINIT_UNIX_BACKEND=x11 are separate targets because they
# debug different backends and should not be combined in one run.

BROWSER_RUN = cargo run --release -p zero-browser

browser: setup-rusty-v8
	$(BROWSER_RUN)

browser-debug: setup-rusty-v8
	RUST_BACKTRACE=1 $(BROWSER_RUN)

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
	./target/test-guard -- cargo test --workspace

# WPT reftest（同样被 test-guard 包裹）。
reftest: target/test-guard
	./target/test-guard -- cargo run --bin zero-wpt-runner -- reftest
