.PHONY: browser browser-debug browser-debug-wayland browser-debug-wayland-log browser-debug-x11

# WAYLAND_DEBUG and WINIT_UNIX_BACKEND=x11 are separate targets because they
# debug different backends and should not be combined in one run.

browser:
	cargo run -p zero-browser

browser-debug:
	RUST_BACKTRACE=1 cargo run -p zero-browser

browser-debug-wayland:
	RUST_BACKTRACE=1 WAYLAND_DEBUG=1 cargo run -p zero-browser

browser-debug-wayland-log:
	mkdir -p target
	RUST_BACKTRACE=1 WAYLAND_DEBUG=1 cargo run -p zero-browser > target/zero-browser-wayland-debug.log 2>&1

browser-debug-x11:
	RUST_BACKTRACE=1 WINIT_UNIX_BACKEND=x11 cargo run -p zero-browser
