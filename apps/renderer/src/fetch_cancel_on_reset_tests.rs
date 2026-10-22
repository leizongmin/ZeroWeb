//! BG-03 M2-S1 回归钉：文档卸载放弃挂起 page fetch——HTTP 连接随导航关闭。
//!
//! M1 基线（evidence/2026-10-22-m1-baseline §DC-3）实证：导航后挂起 fetch 的 HTTP
//! 连接滞留至 client 级 30s 超时，期间占用 per-origin 槽位与 fetch launch 线程名额。
//! 修复契约：page fetch 经 [`default_fetch_handler`] 在 worker 的 URL 登记表登记；
//! `ResetDocumentState` 臂 drain 登记并逐 URL 触发 `cancel_by_url`——订阅者立即回错、
//! 底层连接关闭（对端读到 EOF）、调度槽位即时释放。本测试钉端到端面：
//! 真实 handler + 真实 `ResourceLoader` + 挂起服务端 EOF 观测。

use super::*;
use std::io::Read;
use std::net::TcpListener;
use std::time::Duration;

/// 挂起 HTTP 服务端：接受连接、读走请求头后不发响应——客户端阻塞在等响应。
/// 「请求已上线」经 `request_seen` 交还（保证取消落在在途而非连接前置阶段）；
/// 客户端关闭后 `closed` 交付读结果（EOF = Ok(0)，超时/重置 = 连接未随取消关闭）。
struct HangingServer {
    request_seen: std::sync::mpsc::Receiver<()>,
    closed: std::sync::mpsc::Receiver<std::io::Result<usize>>,
}

impl HangingServer {
    fn spawn() -> (Self, String) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind hanging server");
        let base = format!("http://127.0.0.1:{}", listener.local_addr().expect("local addr").port());
        let (seen_tx, seen_rx) = std::sync::mpsc::channel();
        let (closed_tx, closed_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut buf = [0_u8; 4096];
            let _ = stream.set_read_timeout(Some(Duration::from_secs(60)));
            // 读走请求头；此后客户端等待响应（永不送达）。
            if let Ok(n) = stream.read(&mut buf)
                && n > 0
            {
                let _ = seen_tx.send(());
                let outcome = stream.read(&mut buf);
                let _ = closed_tx.send(outcome);
            }
        });
        (
            Self {
                request_seen: seen_rx,
                closed: closed_rx,
            },
            base,
        )
    }

    fn wait_request(&self) {
        self.request_seen
            .recv_timeout(Duration::from_secs(10))
            .expect("request bytes reach hanging server");
    }

    /// 客户端必须在导航后观测到干净 EOF（连接关闭）。窗口 15s：修复路径取消后毫秒级
    /// 关闭；无取消路径的旧行为要等 client 级 30s 超时才关——窗口短于超时即判失败，
    /// 保证本测试对修复前行为可判别。
    fn wait_eof(&self) {
        let observed = self
            .closed
            .recv_timeout(Duration::from_secs(15))
            .expect("server observes connection close after navigation");
        assert_eq!(
            observed.map_err(|error| error.to_string()),
            Ok(0usize),
            "pending page fetch connection must close on document reset"
        );
    }
}

/// 静默 sink 运行时（镜像 predoc_script_tests::runtime_with_sink，跨测试模块不可见故自持）。
fn quiet_runtime(renderer_id: u64) -> RendererRuntime {
    let mut runtime = RendererRuntime::new(renderer_id);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));
    runtime
}

/// LoadHtml 并驱动加载至脚本阶段收口（镜像 predoc_script_tests 同名 helper）。
fn load_page(runtime: &mut RendererRuntime, html: &str, url: &str) {
    runtime
        .dispatch_message(IpcMessage {
            id: 0,
            kind: IpcMessageKind::LoadHtml(LoadHtmlParams {
                html: html.to_string(),
                css: None,
                url: Some(url.to_string()),
                navigation_epoch: runtime.navigation_epoch.wrapping_add(1),
            }),
        })
        .unwrap();
    let mut ticks = 0;
    loop {
        runtime.tick_pending_load().unwrap();
        runtime.tick_script_prefetch().unwrap();
        if runtime.pending_load.is_none() && runtime.pending_script_prefetch.is_none() {
            break;
        }
        ticks += 1;
        assert!(ticks < 1000, "load/prefetch 未收口");
    }
}

#[test]
fn navigation_cancels_pending_page_fetch_and_closes_connection() {
    let (server, base) = HangingServer::spawn();
    let mut runtime = quiet_runtime(9401);
    // 测试构建默认不装 fetch 观测包装（runtime 构造期 cfg(not(test))）——显式装
    // 生产 default_fetch_handler（含 BG-03 在途登记），面与生产一致。
    runtime
        .js_worker
        .set_fetch_handler(crate::js_worker::default_fetch_handler(
            runtime.js_worker.page_fetch_inflight_registry(),
        ));

    // 源文档：永不完成的挂起 fetch。
    let page1 = format!(
        "<html><body><script>fetch('{base}/stall?tag=bg03').then(function(){{}}, function(){{}});</script></body></html>"
    );
    load_page(&mut runtime, &page1, &format!("{base}/page1"));
    server.wait_request();

    // 导航替换文档 → ResetDocumentState 臂 drain 登记并取消挂起 fetch。
    load_page(
        &mut runtime,
        "<html><body><p>successor</p></body></html>",
        &format!("{base}/page2"),
    );
    server.wait_eof();
}
