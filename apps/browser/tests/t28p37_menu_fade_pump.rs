//! T28-P37 菜单渐入生产泵 e2e 钉（T28 委派词 S4，与修复同提交）。
//!
//! 定谳（evidence/p37-menu-anim-static-triage.md A1）：主菜单渐入期
//! `build_scene` 置位的 `needs_redraw`（app_render.rs）被 main.rs 渲染返回后的
//! 无条件清除吞掉，唯一 `request_redraw` 点读到 false → 渐入第 1 帧后泵链断裂，
//! 后续帧只能靠输入事件续命。smoke 模式在 request_redraw 处 OR 旁路连帧，正是
//! 既有 `context_menu_fades_in_on_open`（直接调 build_scene）测不到泵的根源。
//!
//! 本钉驱动**生产泵路径**：真实窗口（无 `--headless`）、无任何 smoke 旗标、
//! 无 ui_driver（二者都会在 main.rs request_redraw 条件里 OR 旁路断裂路径）；
//! 输入经 `ZW_INJECT_CLICKS` 注入器走与真实输入同一的
//! `handle_mouse_move`/`handle_mouse_click`（左键点工具栏 ⋮ 按钮 → 主菜单），
//! 观察窗 `ZW_MENU_FRAME_TRACE` 记录渐入期每帧 opacity（env 未设零行为）。
//! 注入器武装 = 活跃 tab 页面 load 完成（无加载态 tab，排除 A6 被动补帧掩盖）。
//!
//! 断言（修前红）：打开后无任何后续输入，渐入仍须在 150ms 时长内自然完成——
//! 帧轨迹 ≥4 帧（Poll 16ms 心跳连续成帧证据）、相邻帧最大间隔 ≤200ms，末帧
//! opacity 到 1.0。base（泵断裂）实测 2~3 帧、断档 141~425ms → 断言失败。

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, InputFocus};

/// Xvfb 无窗口管理器：浏览器窗口收不到键盘焦点，`AppEvent::Unfocused` 会把生产
/// 渲染与续帧整体关断（main.rs RedrawRequested 失焦分支 + request_redraw 聚焦
/// 门槛）。在浏览器窗口出现后用 `SetInputFocus` 强制聚焦（等价真实 WM 给新窗口
/// 焦点），winit 随之派发 Focused。同 DISPLAY 连接（守卫前缀 xvfb-run 提供）。
fn focus_browser_window_when_mapped() {
    std::thread::spawn(|| {
        let Ok((conn, screen)) = x11rb::connect(None) else {
            eprintln!("t28p37: x11 connect failed, focus poke skipped");
            return;
        };
        let root = match conn.setup().roots.get(screen) {
            Some(s) => s.root,
            None => return,
        };
        let deadline = Instant::now() + Duration::from_secs(15);
        while Instant::now() < deadline {
            let Ok(win) = find_window_by_name(&conn, root, b"ZeroBrowser") else {
                std::thread::sleep(Duration::from_millis(200));
                continue;
            };
            // 仅对已映射（viewable）窗口聚焦：未映射时 SetInputFocus 报 BadMatch。
            let viewable = conn
                .get_window_attributes(win)
                .ok()
                .and_then(|c| c.reply().ok())
                .map(|a| a.map_state == x11rb::protocol::xproto::MapState::VIEWABLE)
                .unwrap_or(false);
            if !viewable {
                std::thread::sleep(Duration::from_millis(200));
                continue;
            }
            let result = conn.set_input_focus(InputFocus::PARENT, win, 0u32);
            let _ = conn.flush();
            // X 请求异步报错——廉价 round-trip（get_input_focus）确认 BadMatch 类失败并重试。
            let sync_ok = result.is_ok() && conn.get_input_focus().map(|c| c.reply().is_ok()).unwrap_or(false);
            eprintln!("t28p37: focus poke win=0x{win:x} viewable ok={sync_ok}");
            if sync_ok {
                return;
            }
            std::thread::sleep(Duration::from_millis(300));
        }
        eprintln!("t28p37: focus poke gave up");
    });
}

fn find_window_by_name<C>(conn: &C, root: x11rb::protocol::xproto::Window, name: &[u8]) -> Result<u32, ()>
where
    C: Connection,
{
    let mut stack = vec![root];
    while let Some(win) = stack.pop() {
        let tree = conn.query_tree(win).map_err(|_| ())?.reply().map_err(|_| ())?;
        for child in tree.children {
            let ok = conn
                .get_property(false, child, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 256)
                .ok()
                .and_then(|r| r.reply().ok())
                .map(|r| r.value.len() >= name.len() && name.iter().zip(r.value.iter()).all(|(a, b)| a == b))
                .unwrap_or(false);
            if ok {
                return Ok(child);
            }
            stack.push(child);
        }
    }
    Err(())
}

/// 测试页服务：线程化静态 HTTP（进程生命周期 = 测试函数）。
struct TestSite {
    port: u16,
    shutdown: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl TestSite {
    fn spawn() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test site");
        let port = listener.local_addr().expect("addr").port();
        let shutdown = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();
        std::thread::spawn(move || {
            listener.set_nonblocking(true).ok();
            for stream in listener.incoming() {
                if shutdown_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                let Ok(mut stream) = stream else { continue };
                let mut buf = [0u8; 2048];
                let n = stream.read(&mut buf).unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                let path = req.split_whitespace().nth(1).unwrap_or("/").to_string();
                // 简单已加载页：注入器武装锚点（load 完成），页面本身无动画无输入。
                let body = if path.split('?').next().unwrap_or("/") == "/blank" {
                    "<!DOCTYPE html><html><head><title>T28 P37</title></head><body><p>loaded</p></body></html>"
                        .to_string()
                } else {
                    "<!DOCTYPE html><html><body>not found</body></html>".to_string()
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });
        Self { port, shutdown }
    }

    fn url(&self) -> String {
        format!("http://127.0.0.1:{port}/blank", port = self.port)
    }
}

impl Drop for TestSite {
    fn drop(&mut self) {
        self.shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

/// 被测浏览器进程（RAII 关停）。
struct BrowserProcess(Child);

impl Drop for BrowserProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// renderer bin spawn 前存在性 + 陈旧断言（cdp_input_events.rs 同款——陈旧
/// peer-bin 假绿/假红面消除）。
fn assert_renderer_bin_current(renderer: &std::path::Path) {
    assert!(
        renderer.exists(),
        "renderer binary missing at {} — build it explicitly (cargo build -p zero-renderer) before running this suite",
        renderer.display()
    );
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../renderer/src");
    let bin_mtime = std::fs::metadata(renderer)
        .and_then(|m| m.modified())
        .expect("renderer bin mtime readable");
    if let Some(newest) = newest_source_mtime(&src_dir) {
        assert!(
            bin_mtime >= newest,
            "renderer binary at {} is stale (bin mtime {bin_mtime:?} < newest renderer source mtime {newest:?}) — rebuild zero-renderer",
            renderer.display()
        );
    }
}

/// `dir` 下递归最新 `.rs` 源码 mtime（无目录/无源码 → None——跳过陈旧判定）。
fn newest_source_mtime(dir: &std::path::Path) -> Option<std::time::SystemTime> {
    let mut newest: Option<std::time::SystemTime> = None;
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        let candidate = if path.is_dir() {
            newest_source_mtime(&path)
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            entry.metadata().and_then(|m| m.modified()).ok()
        } else {
            None
        };
        if let Some(newest_candidate) = candidate {
            newest = Some(match newest {
                Some(prev) => prev.max(newest_candidate),
                None => newest_candidate,
            });
        }
    }
    newest
}

/// 单行帧轨迹：`menu_frame elapsed_ms=<f> opacity=<f>`。
struct FrameLine {
    elapsed_ms: f32,
    opacity: f32,
}

fn parse_trace(text: &str) -> Vec<FrameLine> {
    text.lines()
        .filter_map(|line| {
            let t = line.find("elapsed_ms=")?;
            let o = line.find("opacity=")?;
            Some(FrameLine {
                elapsed_ms: line[t + "elapsed_ms=".len()..o].trim().parse().ok()?,
                opacity: line[o + "opacity=".len()..].trim().parse().ok()?,
            })
        })
        .collect()
}

#[test]
fn t28p37_menu_fade_pump_completes_without_input() {
    // 无显示环境优雅跳过（tests.rs window_surface_present_smoke 同约定；守卫前缀
    // ZW_GUARD_RUNNER_PREFIX=xvfb-run -a 时 DISPLAY 由 xvfb 提供）。
    if std::env::var("DISPLAY").is_err() {
        eprintln!("skipping t28p37 pump pin (no DISPLAY)");
        return;
    }

    let site = TestSite::spawn();
    let dir = std::env::temp_dir().join(format!("zw-t28p37-{}-{}", std::process::id(), site.port));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let trace_path = dir.join("menu-frame-trace.log");

    // ⋮ 按钮物理坐标（窗口 1024x768、scale 1）：layout::ADDRESS_BAR_PADDING=10、
    // TOOLBAR_MENU_BUTTON_WIDTH=32 → 按钮 x∈[982,1014]；工具栏 y = TAB_STRIP_HEIGHT(40)
    // + ADDRESS_BAR_HEIGHT(50)/2 = 65。常量漂移会让菜单打不开（轨迹为空显性失败）。
    // 注入时刻 800ms：load 完成武装后留出余量，点击后不再有任何输入（无输入续命场景）。
    let click = "998,65@800";

    let browser_bin = env!("CARGO_BIN_EXE_zero-browser");
    let renderer = std::path::Path::new(browser_bin)
        .parent()
        .expect("browser bin dir")
        .join("zero-renderer");
    assert_renderer_bin_current(&renderer);

    let mut cmd = Command::new(browser_bin);
    cmd.arg(site.url())
        // 强制 CPU 软光栅：Xvfb 无 GPU，wgpu surface 初始化在无 WM X 下挂起；
        // A2（每帧全窗软光栅）本就是 CPU 路径病灶，泵行为与呈现后端无关。
        .arg("--renderer=cpu")
        .env("ZERO_STORAGE_DIR", &dir)
        .env("ZERO_RENDERER_PATH", &renderer)
        .env("ZW_INJECT_CLICKS", click)
        .env("ZW_MENU_FRAME_TRACE", &trace_path)
        // 强制 X11 后端：WSLg 会话 WAYLAND_DISPLAY 常驻，泄漏进 xvfb 会话会把窗口
        // 开到真实 Wayland 显示上（is_wayland() 以 env 存在为准）；空串仍算存在，
        // 必须 remove。
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("WAYLAND_SOCKET")
        .env("WINIT_UNIX_BACKEND", "x11")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let _browser = BrowserProcess(cmd.spawn().expect("spawn zero-browser"));
    focus_browser_window_when_mapped();

    // 等渐入轨迹完结（opacity 到 1.0）或超时；打开帧应在点击（+800ms）后很快出现。
    let deadline = Instant::now() + Duration::from_secs(8);
    let lines = loop {
        std::thread::sleep(Duration::from_millis(50));
        let text = std::fs::read_to_string(&trace_path).unwrap_or_default();
        let lines = parse_trace(&text);
        if lines.last().is_some_and(|last| last.opacity >= 1.0) {
            break lines;
        }
        if Instant::now() >= deadline {
            break lines;
        }
    };

    assert!(
        !lines.is_empty(),
        "菜单帧轨迹为空：⋮ 注入点击未打开主菜单（坐标/窗口尺寸漂移？）或渲染首帧未发生 — trace={trace_path:?}"
    );
    let frames = lines.len();
    let last_opacity = lines.last().expect("non-empty").opacity;
    // 泵连续性：相邻帧最大间隔须 ≤300ms（Poll 16ms 心跳 + 单帧光栅成本）。
    // base（泵断裂）实测 3 帧、最大断档 ~425ms——渐入靠稀疏输入事件续命。
    let max_gap_ms = lines
        .windows(2)
        .map(|pair| {
            let (a, b) = (pair[0].elapsed_ms, pair[1].elapsed_ms);
            (b - a).max(0.0)
        })
        .fold(0.0f32, f32::max);
    // 阈值取定：修复后软光栅渐入帧 ~40ms 一帧（debug 实测 4-6 帧），base 泵断裂
    // 2-3 帧且最大断档 141-425ms。≥4 与 ≤200ms 在两端都留出一倍以上裕度，CI 负载
    // 下不抖。
    assert!(
        frames >= 4,
        "渐入期只有 {frames} 帧（泵链断裂特征：打开帧后无输入即停帧，Poll 心跳未续帧）— \
         期望 ≥4 帧（16ms Poll 心跳 × 150ms 渐入，软光栅 ~40ms/帧）trace={trace_path:?}"
    );
    assert!(
        max_gap_ms <= 200.0,
        "帧间隙 {max_gap_ms:.1}ms > 200ms（泵链断裂：渐入靠稀疏输入事件续命，帧大步阶跃）— \
         期望 Poll 心跳连续成帧 trace={trace_path:?}"
    );
    assert!(
        last_opacity >= 1.0,
        "渐入未完成：末帧 opacity={last_opacity}（近全透明停住，等待无上界的事件续命）— \
         期望打开后 150ms 时长内自然到 1.0 trace={trace_path:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}
