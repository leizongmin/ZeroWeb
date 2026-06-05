.PHONY: setup-rusty-v8 build browser browser-debug browser-debug-wayland browser-debug-wayland-log browser-debug-x11

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
