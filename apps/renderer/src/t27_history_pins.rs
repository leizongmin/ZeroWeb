//! T27 返修2/3 常驻钉（DF2-F1/F2 + RC2-F1）：以 `run()` 全主循环驱动真实时序的单测钉。
//!
//! headless e2e 不可达性（钉位选型的依据，red5 实测）：`GoBack` IPC 仅 GUI 工具栏
//! 发送，headless `session.history_traverse` 走 shell 跳转不经
//! [`RendererRuntime::reload_history_entry`]；而 `fetch_post` 开的 dispatch 臂窗内
//! `cached_html` 已清、输入无法派发（窗内二次点击不产生任何导航意图）。故各钉下沉
//! 到 renderer 单测：`run()` 跑在独立线程、出站帧写入共享缓冲、测试线程按帧应答
//! fetch 并注入消息——覆盖真实统一命令队列回放（返修3 RC2-F1：扣压 POST 与消息
//! 同队 FIFO）、deferred_inbound 回放序与 reload 等待窗（含 F1 的 epoch 复核、
//! F2/RC2-F1 的到达序语义）。各钉只用 HEAD 既有消息面
//!（LoadHtml/GoBack/Navigate/MouseEvent/FetchResponse），修前形态可编译、行为翻红。
//!
//! 时序骨架（各钉共用）：LoadHtml A → LoadHtml B（B 承载交互面）→ GoBack 开
//! reload 慢窗（测试扣住 reload 响应控窗）→ 窗内注入 → 应答关窗 → 终态判定。

use super::*;
use std::sync::Mutex;
use zero_protocol::message::MouseEventType;

use super::compositor_publish_tests::{SharedBuf, drain_messages};

const URL_A: &str = "http://zero.test/hist_a";
const URL_B: &str = "http://zero.test/hist_b";
const URL_P2: &str = "http://zero.test/p2";
/// 页面 A：无脚本纯文档（reload 慢臂的取回目标）。
const PAGE_A: &str = "<!DOCTYPE html><html><head><title>A</title></head><body>page-a</body></html>";
/// 页面 C：F1 的点击目标文档。
const PAGE_C: &str = "<!DOCTYPE html><html><head><title>C</title></head><body>page-c</body></html>";
/// 页面 B（F1）：fixed 定位锚链接（命中区 10,10 300x60），窗内让位点击的目标。
const PAGE_B_ANCHOR: &str = concat!(
    "<!DOCTYPE html><html><head><title>B</title></head>",
    r#"<body style="margin:0">"#,
    r#"<a id="lnk" href="http://zero.test/hist_c""#,
    r#" style="position:fixed;left:10px;top:10px;width:300px;height:60px;display:block">go-c</a>"#,
    "</body></html>"
);
/// 页面 B（F2）：纯 POST 表单（无脚本、无自窗），submit 按钮命中区同坐标。
const PAGE_B_FORM: &str = concat!(
    "<!DOCTYPE html><html><head><title>B</title></head>",
    r#"<body style="margin:0">"#,
    r#"<form id="f" method="POST" action="http://zero.test/submit">"#,
    r#"<input name="q" value="t27-body-marker">"#,
    r#"<input id="go" type="submit" value="go""#,
    r#" style="position:fixed;left:10px;top:10px;width:200px;height:60px;display:block">"#,
    r#"</form>"#,
    "</body></html>"
);
/// reload 窗期间自动应答的判定：`None` = 扣住不应答（reload 臂由测试手动控窗）。
fn pin_responder(url: &str, method: &str) -> Option<Vec<u8>> {
    if url.ends_with("/hist_a") {
        return None;
    }
    if method == "POST" && url.ends_with("/submit") {
        return Some(b"<!DOCTYPE html><html><body>submit-result</body></html>".to_vec());
    }
    if url.ends_with("/hist_c") {
        return Some(PAGE_C.as_bytes().to_vec());
    }
    Some(b"<!DOCTYPE html><html><body>doc</body></html>".to_vec())
}

fn send(tx: &mpsc::Sender<IpcMessage>, kind: IpcMessageKind) {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    tx.send(IpcMessage { id, kind }).expect("renderer inbound channel open");
}

/// `run()` 跑独立线程；出站帧落共享缓冲，返回入站注入口。RendererRuntime 非.Send
///（Rc/RefCell 页面栈、主线程亲和）——构造与运行同线程完成，仅通道与缓冲跨界。
fn spawn_pinned_runtime(buf: &SharedBuf) -> mpsc::Sender<IpcMessage> {
    let (tx, rx) = mpsc::channel();
    let buf = buf.clone();
    std::thread::Builder::new()
        .name("t27-pin-runtime".into())
        .spawn(move || {
            let mut runtime = RendererRuntime::with_io(9711, FramePublishMode::Legacy, Box::new(buf), rx);
            let _ = runtime.run();
        })
        .expect("spawn pinned runtime thread");
    tx
}

fn outbound(buf: &SharedBuf) -> Vec<IpcMessage> {
    drain_messages(&buf.0.lock().expect("outbound buf"))
}

fn url_changes(messages: &[IpcMessage]) -> Vec<String> {
    messages
        .iter()
        .filter_map(|m| match &m.kind {
            IpcMessageKind::UrlChanged(url) => Some(url.clone()),
            _ => None,
        })
        .collect()
}

fn load_completes(messages: &[IpcMessage]) -> usize {
    messages
        .iter()
        .filter(|m| matches!(m.kind, IpcMessageKind::LoadComplete))
        .count()
}

fn fetch_requests(messages: &[IpcMessage]) -> Vec<(u64, String, String, Option<Vec<u8>>)> {
    messages
        .iter()
        .filter_map(|m| match &m.kind {
            IpcMessageKind::FetchRequest(p) => Some((p.request_id, p.url.clone(), p.method.clone(), p.body.clone())),
            _ => None,
        })
        .collect()
}

/// 轮询直到 `pred` 成立（超时 panic）；每拍先按 `responder` 应答新到 fetch 请求
///（`answered` 去重，扣住项返回 `None` 不应答）。
fn wait_until(
    buf: &SharedBuf,
    tx: &mpsc::Sender<IpcMessage>,
    answered: &mut HashSet<u64>,
    what: &str,
    timeout: Duration,
    responder: &dyn Fn(&str, &str) -> Option<Vec<u8>>,
    mut pred: impl FnMut(&[IpcMessage]) -> bool,
) {
    let deadline = Instant::now() + timeout;
    loop {
        let messages = outbound(buf);
        for (rid, url, method, _) in fetch_requests(&messages) {
            if answered.insert(rid)
                && let Some(body) = responder(&url, &method)
            {
                send(
                    tx,
                    IpcMessageKind::FetchResponse(FetchResponseParams {
                        request_id: rid,
                        status_code: 200,
                        headers: Vec::new(),
                        body,
                    }),
                );
            }
        }
        if pred(&outbound(buf)) {
            return;
        }
        assert!(Instant::now() < deadline, "timeout waiting for {what}");
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// 开场：视口 + LoadHtml A、B，各等 LoadComplete（渲染就绪，命中测试可用）。
fn load_two_pages(buf: &SharedBuf, tx: &mpsc::Sender<IpcMessage>, page_b: &'static str) {
    send(
        tx,
        IpcMessageKind::SetViewport(SetViewportParams {
            width: 1024,
            height: 768,
            device_scale_factor: 1.0,
        }),
    );
    send(
        tx,
        IpcMessageKind::LoadHtml(LoadHtmlParams {
            html: PAGE_A.to_string(),
            css: None,
            url: Some(URL_A.to_string()),
            navigation_epoch: 1,
        }),
    );
    let mut answered = HashSet::new();
    wait_until(
        buf,
        tx,
        &mut answered,
        "page A load complete",
        Duration::from_secs(30),
        &pin_responder,
        |ms| load_completes(ms) >= 1,
    );
    send(
        tx,
        IpcMessageKind::LoadHtml(LoadHtmlParams {
            html: page_b.to_string(),
            css: None,
            url: Some(URL_B.to_string()),
            navigation_epoch: 2,
        }),
    );
    wait_until(
        buf,
        tx,
        &mut answered,
        "page B load complete",
        Duration::from_secs(30),
        &pin_responder,
        |ms| load_completes(ms) >= 2,
    );
    // 渲染/排版收尾余量（命中测试就绪）。
    std::thread::sleep(Duration::from_millis(200));
}

/// T27 返修2钉（DF2-F1）：history 重载阻塞等待窗内的让位点击不得被迟到的重载结果
/// 覆盖。修复前（红）：`reload_history_entry` 的 `fetch_get` 收场后无条件
/// `run_staged_load`——窗内点击已派发（`UrlChanged(C)` 已发、pending_load 已记 C），
/// 陈旧重载整体覆盖，C 的文档永不加载，终态=A。修复后（绿）：入臂 epoch 复核发现
/// 窗内新导航推进（点击 `navigate_with` epoch+1），放弃陈旧载入——C 存活并完成
/// 加载，终态=点击目标。终态判据：第三次 LoadComplete（红=A 重载 / 绿=C 加载）
/// 后末个 `UrlChanged`。该钉同时覆盖 TV2-F2 点名的后退（GoBack）扣压语义。
#[test]
fn t27p35_history_reload_click_not_clobbered() {
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let tx = spawn_pinned_runtime(&buf);
    load_two_pages(&buf, &tx, PAGE_B_ANCHOR);

    // GoBack → reload A 慢窗；测试扣住 reload 响应控窗。
    send(&tx, IpcMessageKind::GoBack);
    let mut answered = HashSet::new();
    wait_until(
        &buf,
        &tx,
        &mut answered,
        "reload fetch request",
        Duration::from_secs(30),
        &pin_responder,
        |ms| {
            fetch_requests(ms)
                .iter()
                .any(|(_, url, method, _)| method == "GET" && url.ends_with("/hist_a"))
        },
    );
    let rid_a = fetch_requests(&outbound(&buf))
        .iter()
        .rev()
        .find(|(_, url, method, _)| method == "GET" && url.ends_with("/hist_a"))
        .map(|(rid, _, _, _)| *rid)
        .expect("reload fetch request id");

    // 窗内让位点击（cached_html 未清、页面可交互）→ /c 导航即时派发。
    send(
        &tx,
        IpcMessageKind::MouseEvent(MouseEventParams {
            x: 100.0,
            y: 30.0,
            button: 0,
            event_type: MouseEventType::Click,
        }),
    );
    // 前提：点击已派发（navigate_with 入口即发 UrlChanged(C)，红绿同有）。
    wait_until(
        &buf,
        &tx,
        &mut answered,
        "in-window click UrlChanged(C)",
        Duration::from_secs(30),
        &pin_responder,
        |ms| url_changes(ms).iter().any(|u| u.ends_with("/hist_c")),
    );

    // 关窗：迟到的重载应答（红：无条件 staged load；绿：epoch 复核放弃）。
    send(
        &tx,
        IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: rid_a,
            status_code: 200,
            headers: Vec::new(),
            body: PAGE_A.as_bytes().to_vec(),
        }),
    );
    // 终态判定：第三次 LoadComplete 落地后（红=A 重载完成 / 绿=C 加载完成）比对末位。
    wait_until(
        &buf,
        &tx,
        &mut answered,
        "post-window load completion",
        Duration::from_secs(30),
        &pin_responder,
        |ms| load_completes(ms) >= 3,
    );
    let messages = outbound(&buf);
    let changes = url_changes(&messages);
    let last = changes.last().expect("at least one UrlChanged");
    assert!(
        last.ends_with("/hist_c"),
        "in-window click navigation must survive the late history reload (DF2-F1 pre-fix shape: \
         stale staged load clobbered the click) — final url {last}, changes {changes:?}"
    );
    assert!(
        fetch_requests(&messages)
            .iter()
            .any(|(_, url, method, _)| method == "GET" && url.ends_with("/hist_c")),
        "click target document must actually be fetched (not just announced): {changes:?}"
    );
}

/// T27 返修2钉（DF2-F2/F3）：等待窗内更早到达的导航命令不得被后暂存却先 drain 的
/// POST 跳队覆盖——到达序语义。时序：GoBack 开 reload 慢窗（dispatch 臂——窗收场即
/// 迭代末，下一拍主循环顶部 drain 先于 deferred_inbound 回放，倒序窗口形态）→ 窗内
/// Navigate(/p2) 先到（非让位类，扣压入 deferred_inbound）→ 点击提交后到（POST 意图
/// 暂存）。修复前（红）：drain 先执行 POST（/submit 入口消息先发）、随后回放更早的
/// Navigate——终态=旧命令目标 /p2，提交副作用已发而被覆盖（且单槽时同窗第二个 POST
/// 意图静默丢失，DF2-F3 同形态）。修复后（绿）：返修3 统一命令队列按到达序回放
///（Navigate 先、POST 后），终态=POST 结果。返修1 收益同钉保持：POST 携完整
/// body、无 GET 回落。
#[test]
fn t27p35_post_nav_arrival_order() {
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let tx = spawn_pinned_runtime(&buf);
    load_two_pages(&buf, &tx, PAGE_B_FORM);

    // GoBack → reload A 慢窗（dispatch 臂，测试扣住 reload 响应控窗）。
    send(&tx, IpcMessageKind::GoBack);
    let mut answered = HashSet::new();
    wait_until(
        &buf,
        &tx,
        &mut answered,
        "reload fetch request",
        Duration::from_secs(30),
        &pin_responder,
        |ms| {
            fetch_requests(ms)
                .iter()
                .any(|(_, url, method, _)| method == "GET" && url.ends_with("/hist_a"))
        },
    );

    // 窗内先到的导航命令：Navigate 扣压入 deferred_inbound（保到达序）。
    send(
        &tx,
        IpcMessageKind::Navigate(NavigateParams {
            url: URL_P2.to_string(),
            referrer: Some(URL_B.to_string()),
            navigation_epoch: 3,
        }),
    );
    // 到达序确定性余量：Navigate 先入队，点击后到。
    std::thread::sleep(Duration::from_millis(200));
    // 窗内后到的 POST 意图：点击提交 → navigate_with depth>0 → 暂存（队列化）。
    send(
        &tx,
        IpcMessageKind::MouseEvent(MouseEventParams {
            x: 100.0,
            y: 30.0,
            button: 0,
            event_type: MouseEventType::Click,
        }),
    );
    // 暂存确认：扣压语义出站静默（F3 的本体——暂存不产生任何请求帧），点击的让位
    // 派发在 250ms 等待切片内完成，留足余量后关窗；若点击未入暂存，终态判定将以
    // 「/submit 缺失」超时翻红。
    std::thread::sleep(Duration::from_millis(400));

    // 关窗：reload 应答（epoch 未变——Navigate 只是扣压未派发，staged load 合法放行）。
    let rid_a = fetch_requests(&outbound(&buf))
        .iter()
        .rev()
        .find(|(_, url, method, _)| method == "GET" && url.ends_with("/hist_a"))
        .map(|(rid, _, _, _)| *rid)
        .expect("reload fetch request id");
    send(
        &tx,
        IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: rid_a,
            status_code: 200,
            headers: Vec::new(),
            body: PAGE_A.as_bytes().to_vec(),
        }),
    );

    // 终态判定：两条命令都须执行（/p2 回放 + POST drain），末个 UrlChanged=POST 目标。
    // POST 完整性：请求帧 body 携表单字段、无 GET /submit 回落。
    wait_until(
        &buf,
        &tx,
        &mut answered,
        "both commands executed",
        Duration::from_secs(60),
        &pin_responder,
        |ms| {
            let changes = url_changes(ms);
            changes.iter().any(|u| u.ends_with("/p2")) && changes.iter().any(|u| u.ends_with("/submit"))
        },
    );
    let messages = outbound(&buf);
    let changes = url_changes(&messages);
    let last = changes.last().expect("at least one UrlChanged");
    assert!(
        last.ends_with("/submit"),
        "final state must be the stashed POST result (DF2-F2 pre-fix shape: drain jumped the \
         arrival order and the earlier-arriving Navigate replayed over the POST result) — \
         final url {last}, changes {changes:?}"
    );
    let posts: Vec<_> = fetch_requests(&messages)
        .into_iter()
        .filter(|(_, url, method, _)| method == "POST" && url.ends_with("/submit"))
        .collect();
    assert_eq!(posts.len(), 1, "exactly one POST /submit expected: {posts:?}");
    let body = posts[0].3.clone().unwrap_or_default();
    assert!(
        String::from_utf8_lossy(&body).contains("q=t27-body-marker"),
        "POST body must carry the form field value, got {:?}",
        String::from_utf8_lossy(&body)
    );
    assert!(
        !fetch_requests(&messages)
            .iter()
            .any(|(_, url, method, _)| method == "GET" && url.ends_with("/submit")),
        "parked POST must not degrade to a body-less GET fallback"
    );
}

/// T27 返修3钉（RC2-F1，case B 反向）：POST 先暂存 T1、导航命令后到 T2 时，后到导航
/// 不得被 drain 的 T1 POST 反超覆盖——终态=最后到达的导航意图。时序与
/// [`t27p35_post_nav_arrival_order`]（case A）镜像：GoBack 开 reload 慢窗 → 点击提交
/// 先到（POST 意图暂存，出站静默）→ Navigate(/p2) 后到（等待窗扣压）。修复前（红，
/// 返修2 代码）：积压非空时 drain 跳过 → 回放 T2 导航 → 次拍积压空 drain 弹出 T1
/// POST 反超——终态=POST，T1<T2 到达序要求终态=/p2。修复后（绿）：扣压 POST 按到达
/// 时刻并入统一命令队列，单一 FIFO 回放——T1 POST 先执行、T2 导航后执行，终态=/p2。
/// 返修1/2 收益同钉保持：恰 1 次 POST、body 携表单字段、无 GET 回落（多意图全数执行）。
#[test]
fn t27p35_parked_post_yields_to_later_nav() {
    let buf = SharedBuf(Arc::new(Mutex::new(Vec::new())));
    let tx = spawn_pinned_runtime(&buf);
    load_two_pages(&buf, &tx, PAGE_B_FORM);

    // GoBack → reload A 慢窗（dispatch 臂，测试扣住 reload 响应控窗）。
    send(&tx, IpcMessageKind::GoBack);
    let mut answered = HashSet::new();
    wait_until(
        &buf,
        &tx,
        &mut answered,
        "reload fetch request",
        Duration::from_secs(30),
        &pin_responder,
        |ms| {
            fetch_requests(ms)
                .iter()
                .any(|(_, url, method, _)| method == "GET" && url.ends_with("/hist_a"))
        },
    );

    // 窗内先到的 POST 意图：点击提交（T1，暂存不产生出站请求帧）。
    send(
        &tx,
        IpcMessageKind::MouseEvent(MouseEventParams {
            x: 100.0,
            y: 30.0,
            button: 0,
            event_type: MouseEventType::Click,
        }),
    );
    // 到达序确定性余量：点击暂存先落，Navigate 后到。
    std::thread::sleep(Duration::from_millis(200));
    // 窗内后到的导航命令（T2）：等待窗扣压（非让位类，与 POST 意图同队按到达序）。
    send(
        &tx,
        IpcMessageKind::Navigate(NavigateParams {
            url: URL_P2.to_string(),
            referrer: Some(URL_B.to_string()),
            navigation_epoch: 3,
        }),
    );
    // 让点击派发与 Navigate 入队收尾，留足余量后关窗。
    std::thread::sleep(Duration::from_millis(400));

    // 关窗：reload 应答（epoch 未变——两条意图均未执行，staged load 合法放行）。
    let rid_a = fetch_requests(&outbound(&buf))
        .iter()
        .rev()
        .find(|(_, url, method, _)| method == "GET" && url.ends_with("/hist_a"))
        .map(|(rid, _, _, _)| *rid)
        .expect("reload fetch request id");
    send(
        &tx,
        IpcMessageKind::FetchResponse(FetchResponseParams {
            request_id: rid_a,
            status_code: 200,
            headers: Vec::new(),
            body: PAGE_A.as_bytes().to_vec(),
        }),
    );

    // 终态判定：两条意图都须执行（POST /submit 与 Navigate /p2 都发 UrlChanged），
    // 末个 UrlChanged=后到导航 /p2。POST 完整性同 case A 钉。
    wait_until(
        &buf,
        &tx,
        &mut answered,
        "both commands executed",
        Duration::from_secs(60),
        &pin_responder,
        |ms| {
            let changes = url_changes(ms);
            changes.iter().any(|u| u.ends_with("/submit")) && changes.iter().any(|u| u.ends_with("/p2"))
        },
    );
    let messages = outbound(&buf);
    let changes = url_changes(&messages);
    let last = changes.last().expect("at least one UrlChanged");
    assert!(
        last.ends_with("/p2"),
        "final state must be the later-arriving Navigate (RC2-F1 case B pre-fix shape: drain \
         replayed the earlier stashed POST over the later navigation) — final url {last}, \
         changes {changes:?}"
    );
    let posts: Vec<_> = fetch_requests(&messages)
        .into_iter()
        .filter(|(_, url, method, _)| method == "POST" && url.ends_with("/submit"))
        .collect();
    assert_eq!(posts.len(), 1, "exactly one POST /submit expected: {posts:?}");
    let body = posts[0].3.clone().unwrap_or_default();
    assert!(
        String::from_utf8_lossy(&body).contains("q=t27-body-marker"),
        "POST body must carry the form field value, got {:?}",
        String::from_utf8_lossy(&body)
    );
    assert!(
        !fetch_requests(&messages)
            .iter()
            .any(|(_, url, method, _)| method == "GET" && url.ends_with("/submit")),
        "parked POST must not degrade to a body-less GET fallback"
    );
}
