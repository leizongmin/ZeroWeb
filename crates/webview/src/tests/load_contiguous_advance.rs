//! T32/G1：加载带内连续推进——单步原子越过 tick 预算后，剩余步必须在同一 tick
//! 调用内续推完成（改前逐拍推进：一次 tick 一步，步间由主循环 16ms recv 节拍
//! 隔开，styled 带实测 ~84ms 纯调度间隙）。

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, channel};

use zero_page_runtime::{AsyncFetchHost, ResourceFetchMeta};

use crate::{AsyncPageLoad, PageLoadStage, WebView, WebViewConfig};

struct SingleSheetHost {
    stylesheet: &'static str,
}

/// 进程级 env 的 RAII 还原（panic 时也恢复未设状态；edition 2024 env 修改为
/// unsafe，调用方持 [`super::env_lock`] 期间无并发读者）。
struct EnvVarGuard(&'static str);

impl EnvVarGuard {
    fn set(name: &'static str, value: &str) -> Self {
        unsafe { std::env::set_var(name, value) };
        Self(name)
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        unsafe { std::env::remove_var(self.0) };
    }
}

impl AsyncFetchHost for SingleSheetHost {
    fn fetch_text_meta(&mut self, url: &str, _: ResourceFetchMeta) -> Receiver<Result<String, String>> {
        let (tx, rx) = channel();
        let text = if url.ends_with(".css") {
            Ok(self.stylesheet.to_string())
        } else {
            Err(format!("unexpected text fetch: {url}"))
        };
        tx.send(text).unwrap();
        rx
    }

    fn fetch_bytes_meta(&mut self, url: &str, _: ResourceFetchMeta) -> Receiver<Result<Vec<u8>, String>> {
        let (tx, rx) = channel();
        tx.send(Err(format!("unexpected bytes fetch: {url}"))).unwrap();
        rx
    }
}

/// 钉：外链 CSS 页在单次 tick（预算≈0，任何单步即越限）内走完
/// 骨架带 → stylesheets ready → styled 带 → Complete——步间零逐拍等待。
/// kill-switch `ZW_LOAD_CONTIGUOUS_ADVANCE=0` 时回退逐拍，本钉不再成立
///（回退臂由对照断言覆盖，见下方 kill-switch 测试）。
#[test]
fn styled_band_advances_contiguously_within_single_tick() {
    // 默认态（变量未设）= 启用；持锁防 kill-switch 用例并发改 env。
    let _guard = super::env_lock();
    let html = r#"<html><head><link rel="stylesheet" href="/assets/a.css"></head>
        <body><div class="card">hello</div><p>world</p></body></html>"#;
    let mut load = AsyncPageLoad::from_html("https://example.test/page.html", html.to_string());
    let mut webview = WebView::new(WebViewConfig::default());
    let mut host = SingleSheetHost {
        stylesheet: "body { color: #111 } .card { border: 1px solid #333 }",
    };

    let changed = load.tick(&mut webview, &mut host, 0.001);
    assert!(changed, "单 tick 应产出帧变更（styled ready/完成均置位）");
    assert!(
        matches!(load.stage(), PageLoadStage::Complete),
        "styled 带必须在单次 tick 内连续推进至 Complete，实际 stage={:?}",
        load.stage()
    );
    assert!(webview.last_render().is_some(), "styled 帧必须已应用到 webview");
}

/// 对照钉：kill-switch 关闭后回退单次 advance 逐拍推进——同输入同预算下
/// 单次 tick 不得整带完成（骨架带 InProgress，styled 带未达）。
#[test]
fn kill_switch_restores_per_tick_advancement() {
    // 进程级 env：串行锁防并行用例互踩（同 async_load 既有 ENV_TEST_LOCK 纪律）。
    let _guard = super::env_lock();
    let _switch = EnvVarGuard::set("ZW_LOAD_CONTIGUOUS_ADVANCE", "0");
    let html = r#"<html><head><link rel="stylesheet" href="/assets/a.css"></head>
        <body><div class="card">hello</div><p>world</p></body></html>"#;
    let mut load = AsyncPageLoad::from_html("https://example.test/page.html", html.to_string());
    let mut webview = WebView::new(WebViewConfig::default());
    let mut host = SingleSheetHost {
        stylesheet: "body { color: #111 } .card { border: 1px solid #333 }",
    };

    let changed = load.tick(&mut webview, &mut host, 0.001);
    let _ = changed;
    assert!(
        !matches!(load.stage(), PageLoadStage::Complete),
        "kill-switch 关闭后单 tick 只推单步，不得整带完成"
    );
}
