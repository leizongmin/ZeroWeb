//! slice39：`drain_pending_dom_mutations` / `notify_shim_apply_generation` 两处 200ms
//! 有界等待的**时限语义钉** + apply/notify/代际状态的**多实例隔离钉**。
//!
//! 语义被钉物（见 diag/evidence/slice39/design-card.md §1-§3）：
//! - 超时只放弃等待，不撤回命令、不放弃本轮 apply——滞留命令 FIFO 迟到必达（误读 A：
//!   「超时 ⇒ 跳过本轮 apply」、误读 B：「超时 ⇒ 丢弃滞留 bump」两形态在此恒红）；
//! - ready 旗标由积压回调的 ResolveAsyncCallback 臂末重新置位 ⇒ 超时后下一成功
//!   checkpoint 补应用滞留 mutation（不丢）；
//! - apply/notify/`_zwApplyGeneration` 状态 per worker sandbox / per WebView 实例，
//!   跨实例零串扰；
//! - 滞留 bump 的**唯一**丢弃出路是导航 seq 清扫（slice39 §8 勘误构造，钉 4）——
//!   超时本身不丢弃（钉 2），导航丢弃无损 by-design（reset 即等价新鲜）。
//!
//! 超时制造：helper 线程经 [`RendererJsWorker::executor`]（`ScriptFn = Arc<dyn Fn +
//! Send + Sync>`）提交 ~1.2s JS 忙臂（`Date.now()` 忙循环；V8 watchdog 30s /
//! TAB_JS_CHANNEL_TIMEOUT 35s 均不受扰），`execution_count_for_test` 轮询确认臂被领取。
//! 断言全部为「超时后语义可恢复」形态，不测「恰好 200ms」，无时钟敏感断言。
//! 事件循环边界语义参照产品侧注释：
//! https://html.spec.whatwg.org/multipage/webappapis.html#event-loop-processing-model

use super::*;
use crate::js_worker::RendererJsWorker;

/// 忙臂时长（ms）——须 > drain 超时轮总消耗（空脚本 200ms + bump 200ms）+ 调度裕量。
const S39_BUSY_MS: u64 = 1200;

fn s39_busy_script() -> String {
    format!(
        "globalThis.__s39t = Date.now(); while (Date.now() - globalThis.__s39t < {S39_BUSY_MS}) {{}}; 's39-busy-done';"
    )
}

fn s39_wait_until(mut cond: impl FnMut() -> bool, timeout_ms: u64) -> bool {
    let start = std::time::Instant::now();
    while !cond() {
        if start.elapsed() >= std::time::Duration::from_millis(timeout_ms) {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    true
}

/// 在 helper 线程占住 worker（忙臂被领取后返回 join 句柄）。
fn s39_occupy_worker(worker: &RendererJsWorker) -> std::thread::JoinHandle<Result<String, String>> {
    let exec = worker.executor();
    let busy = std::thread::spawn(move || exec(&s39_busy_script()));
    let baseline = worker.execution_count_for_test();
    assert!(
        s39_wait_until(|| worker.execution_count_for_test() > baseline, 2000),
        "忙臂须被 worker 领取（execution_count 前进）"
    );
    busy
}

fn s39_apply_generation(worker: &RendererJsWorker) -> i64 {
    worker
        .execute_script_direct("String(globalThis._zwApplyGeneration())")
        .unwrap()
        .parse()
        .expect("代际计数为数字")
}

// 钉 1（误读 A + 滞留 mutation 不丢）：超时轮同轮仍 apply 已记录 mutation（返回 true）；
// 滞留回调经 ready 旗标重置在下一成功 checkpoint 补应用。「超时 ⇒ 跳过本轮 apply」回退
//（在 drain 有界等待后插 `if result.is_err() { return false; }`）使首轮断言恒红。
#[test]
fn drain_timeout_applies_recorded_and_recovers_backlog_s39() {
    let html = r#"<html><body><div id="s39c1"></div><div id="s39c2"></div></body></html>"#;
    let url = "https://zero.test/s39drain";
    let mut worker = RendererJsWorker::spawn(183);
    worker.set_dom_snapshot(html, url);

    // C1：快 timer（worker 空闲即执行）——mutation 记入队列 + ready 旗标置位。
    worker
        .execute_script_direct(
            "setTimeout(function(){ document.querySelector('#s39c1').setAttribute('data-c1','1'); }, 5);",
        )
        .unwrap();
    assert!(
        s39_wait_until(
            || !worker.mutations().lock().unwrap_or_else(|e| e.into_inner()).is_empty(),
            2000
        ),
        "C1 回调须在空闲 worker 上执行并入队 mutation"
    );

    // 占住 worker——drain 超时轮的空脚本/bump 两次有界等待（200ms×2）均落忙臂窗内。
    let busy = s39_occupy_worker(&worker);

    let mut buf = html.to_string();
    let mut ctx = PageScriptContext {
        html: &mut buf,
        url,
        js_worker: &worker,
        webview: None,
    };

    // 超时轮：take 旗标（C1 置位）→ 空脚本有界等待超时 → **同轮仍 apply** C1 mutation。
    let committed = drain_pending_dom_mutations(&mut ctx);
    assert!(committed, "超时轮已记录的 mutation 仍须 apply（drain 返回 true）");
    assert!(
        ctx.html.contains("data-c1"),
        "超时轮后 C1 mutation 须已落宿主 HTML：{}",
        ctx.html
    );

    busy.join().expect("忙臂线程 join").expect("忙臂脚本执行成功");

    // C2：长臂结束后经优先通道注册快 timer——其 ResolveAsyncCallback 臂末重新置位
    // ready 旗标，下一轮 drain（成功 checkpoint）补应用滞留 mutation。
    worker
        .submit_script_priority(
            "setTimeout(function(){ document.querySelector('#s39c2').setAttribute('data-c2','1'); }, 5);",
        )
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let mut recovered = false;
    while std::time::Instant::now() < deadline {
        if drain_pending_dom_mutations(&mut ctx) {
            recovered = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        recovered,
        "滞留回调 mutation 须在下一成功 checkpoint 补应用（超时未致丢失）"
    );
    assert!(ctx.html.contains("data-c2"), "C2 mutation 须落宿主 HTML：{}", ctx.html);
    worker.shutdown();
}

// 钉 2（误读 B）：bump 有界等待超时 ≠ 丢弃——命令滞留 FIFO，worker 空闲后补执行，
// shim 代际仍推进。「超时 ⇒ 丢弃滞留命令」或「删 notify 调用」形态在此恒红
//（gen 恒不前进，轮询超时）。
#[test]
fn apply_generation_bump_lands_after_bounded_wait_timeout_s39() {
    let html = r#"<html><body><div id="s39g"></div></body></html>"#;
    let url = "https://zero.test/s39bump";
    let mut worker = RendererJsWorker::spawn(184);
    worker.set_dom_snapshot(html, url);
    let gen0 = s39_apply_generation(&worker);

    // 记录一条 mutation（worker 空闲，直接执行入队）。
    worker
        .execute_script_direct("document.querySelector('#s39g').setAttribute('data-g','1');")
        .unwrap();

    // 占住 worker——apply 后 notify 的 bump 有界等待必超时。
    let busy = s39_occupy_worker(&worker);

    let mut buf = html.to_string();
    let mut ctx = PageScriptContext {
        html: &mut buf,
        url,
        js_worker: &worker,
        webview: None,
    };
    let applied = apply_recorded_mutations(&mut ctx, html);
    assert!(applied.is_some(), "apply 成功（HTML 回写路径）");
    assert!(ctx.html.contains("data-g"), "mutation 已落宿主 HTML：{}", ctx.html);

    busy.join().expect("忙臂线程 join").expect("忙臂脚本执行成功");

    // bump 滞留补执行：worker 空闲后 FIFO 消费 bump → 代际推进（延迟交付，非丢失）。
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let mut landed = false;
    let mut gen1 = gen0;
    while std::time::Instant::now() < deadline {
        gen1 = s39_apply_generation(&worker);
        if gen1 > gen0 {
            landed = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        landed,
        "bump 超时后须滞留补执行（gen0={gen0} gen1={gen1}）——滞留命令丢弃/通知删行形态恒红"
    );
    worker.shutdown();
}

// 钉 3（多 tab 并发隔离）：双 (worker, webview, html) 实例模拟双 tab——tab A 的
// apply（webview 在场臂）/notify/代际推进对 tab B 的代际、mutation 队列、宿主 HTML、
// webview 缓存零影响；B 再 apply 独立推进且不扰动 A。全局单槽（gcs DRAIN_RECORD）
// 按「错配即兜底」设计（computed_style_cache.rs 键契约），不产生错数据——本钉锁
// per-instance 正确面的串扰回归。
#[test]
fn apply_generation_state_isolated_across_tab_instances_s39() {
    let html_a = r#"<html><body><div id="s39a"></div></body></html>"#;
    let html_b = r#"<html><body><div id="s39b"></div></body></html>"#;
    let url_a = "https://zero.test/s39tab-a";
    let url_b = "https://zero.test/s39tab-b";
    let mut worker_a = RendererJsWorker::spawn(185);
    let mut worker_b = RendererJsWorker::spawn(186);
    worker_a.set_dom_snapshot(html_a, url_a);
    worker_b.set_dom_snapshot(html_b, url_b);
    let mut wv_a = zero_webview::WebView::new(zero_webview::WebViewConfig::default());
    let mut wv_b = zero_webview::WebView::new(zero_webview::WebViewConfig::default());
    wv_a.prepare_document_state(url_a);
    wv_a.load_html(html_a, None);
    wv_b.prepare_document_state(url_b);
    wv_b.load_html(html_b, None);
    let mut buf_a = html_a.to_string();
    let mut buf_b = html_b.to_string();
    let wv_b_html_before = wv_b.html_content().to_string();
    let gen_a0 = s39_apply_generation(&worker_a);
    let gen_b0 = s39_apply_generation(&worker_b);

    // Tab A：记录 + apply（webview 在场臂 path A——含 evict + publish + notify 全链）。
    worker_a
        .execute_script_direct("document.querySelector('#s39a').setAttribute('data-a','1');")
        .unwrap();
    {
        let mut ctx = PageScriptContext {
            html: &mut buf_a,
            url: url_a,
            js_worker: &worker_a,
            webview: Some(&mut wv_a),
        };
        let applied = apply_recorded_mutations(&mut ctx, html_a);
        assert!(applied.is_some(), "tab A apply 成功（webview 在场臂）");
    }
    assert!(buf_a.contains("data-a"), "tab A 宿主 HTML 已更新：{buf_a}");
    let gen_a1 = s39_apply_generation(&worker_a);
    assert!(gen_a1 > gen_a0, "tab A 代际推进（A 的 notify 只作用 A 的 sandbox）");

    // Tab B：A 的 apply/notify 后零串扰——代际、队列、宿主 HTML、webview 缓存全不变。
    assert_eq!(
        s39_apply_generation(&worker_b),
        gen_b0,
        "A 的 apply/notify 不得推进 B 的 shim 代际（per-sandbox 状态）"
    );
    assert!(
        worker_b
            .mutations()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_empty(),
        "A 的 apply 不得向 B 的 mutation 队列投递"
    );
    assert_eq!(buf_b, html_b, "A 的 apply 不得改写 B 的宿主 HTML");
    assert_eq!(
        wv_b.html_content(),
        wv_b_html_before,
        "A 的 apply 不得改写 B 的 webview 缓存文档"
    );

    // Tab B 再 apply：独立推进（从自身 gen_b0 基础），不扰动 A。
    worker_b
        .execute_script_direct("document.querySelector('#s39b').setAttribute('data-b','1');")
        .unwrap();
    {
        let mut ctx = PageScriptContext {
            html: &mut buf_b,
            url: url_b,
            js_worker: &worker_b,
            webview: Some(&mut wv_b),
        };
        let applied = apply_recorded_mutations(&mut ctx, html_b);
        assert!(applied.is_some(), "tab B apply 成功（webview 在场臂）");
    }
    assert!(
        buf_b.contains("data-b") && !buf_b.contains("data-a"),
        "tab B 宿主 HTML 只含自身变更（全局 drain 记录错配须走兜底而非串写）：{buf_b}"
    );
    assert!(s39_apply_generation(&worker_b) > gen_b0, "tab B 代际从自身基线独立推进");
    assert_eq!(
        s39_apply_generation(&worker_a),
        gen_a1,
        "B 的 apply/notify 不得推进 A 的代际"
    );
    assert!(
        buf_a.contains("data-a") && !buf_a.contains("data-b"),
        "A 的宿主 HTML 不受 B apply 扰动"
    );
    worker_a.shutdown();
    worker_b.shutdown();
}

// 钉 4（slice39 §8 勘误顺延）：滞留 bump 遇导航 seq 丢弃——**确定性**构造，非时序竞态。
// bump 经 normal 通道入队（notify 的有界等待只放弃等待、不撤回命令），`reset_document_state`
// 经 prio 通道提交，worker 分派臂 prio 先于 normal 且 reset 臂按提交代际（seq）清扫
// normal 队列（`survives_document_reset`：seq ≤ reset_seq 的 Execute 随旧文档丢弃）——
// 忙臂窗内先提交 bump、再提交 reset 即结构保证复现「丢弃」。
// 观测面（slice40 汇总 xI-3 顺延主因）：reset 重建 shim context 后 gen 归零，单靠 gen
// 无法区分「丢弃」与「执行后随 context 销毁」——以 cfg(test) `execution_count` 作判别
//（Execute 臂每命令恰 +1）：丢弃形态下忙臂后仅屏障探针 +1；清扫失效/滞留补执行形态
// +2 恒红（gen 断言在此形态下仍绿——正是不补 ec 观测就发现不了的盲区）。
// https://html.spec.whatwg.org/multipage/browsing-the-web.html#navigate
#[test]
fn lingering_bump_dropped_by_navigation_reset_seq_sweep_s41() {
    let html = r#"<html><body><div id="s41n"></div></body></html>"#;
    let url = "https://zero.test/s41drop";
    let mut worker = RendererJsWorker::spawn(187);
    worker.set_dom_snapshot(html, url);

    // 代际基线推进（非零可辨新 context 归零）：空闲 worker 记录 mutation + apply——
    // HTML 回写臂 apply 成功后内部 notify bump 同步落地。
    worker
        .execute_script_direct("document.querySelector('#s41n').setAttribute('data-n','1');")
        .unwrap();
    let mut buf = html.to_string();
    let mut ctx = PageScriptContext {
        html: &mut buf,
        url,
        js_worker: &worker,
        webview: None,
    };
    assert!(
        apply_recorded_mutations(&mut ctx, html).is_some(),
        "首轮 apply 成功（notify bump 落地，代际基线推进）"
    );
    let gen1 = s39_apply_generation(&worker);
    assert!(gen1 >= 1, "notify bump 已落地（代际基线非零）：{gen1}");

    // 忙臂窗内先 bump 后 reset（提交序即 seq 序）：notify 200ms / reset 250ms 有界等待
    // 均在忙臂（1.2s）内超时放弃等待，两命令分别滞留 normal FIFO 与 prio 队列。
    let busy = s39_occupy_worker(&worker);
    let ec_occupied = worker.execution_count_for_test();
    notify_shim_apply_generation(&mut ctx);
    worker.reset_document_state();
    busy.join().expect("忙臂线程 join").expect("忙臂脚本执行成功");

    // 新文档到达（seq 最大，清扫存活臂）→ 落 reset 重建后的新 context；屏障探针排在
    // 一切滞留命令之后，其返回即 reset 臂 + seq 清扫已完结（worker 串行边界）。
    let html2 = r#"<html><body><div id="s41m"></div></body></html>"#;
    let url2 = "https://zero.test/s41fresh";
    worker.set_dom_snapshot(html2, url2);
    let gen_fresh = s39_apply_generation(&worker);
    let ec_final = worker.execution_count_for_test();

    // 判别断言：忙臂被领取后仅探针 1 次 Execute——bump 被 seq 清扫丢弃、从未执行。
    assert_eq!(
        ec_final,
        ec_occupied + 1,
        "滞留 bump 须被导航 seq 清扫丢弃（不执行）：ec {ec_occupied}→{ec_final}"
    );
    assert_eq!(
        gen_fresh, 0,
        "新文档 shim context 全新（代际归零，滞留 bump 不落新文档）"
    );
    assert!(
        worker.mutations().lock().unwrap_or_else(|e| e.into_inner()).is_empty(),
        "reset 臂清空 mutation 队列（旧文档残留不入新文档）"
    );

    // drain 面无残留：ready 旗标已随导航 reset 置 false——drain no-op、新文档宿主 HTML
    // 不被改写。
    let mut buf2 = html2.to_string();
    let mut ctx2 = PageScriptContext {
        html: &mut buf2,
        url: url2,
        js_worker: &worker,
        webview: None,
    };
    assert!(!drain_pending_dom_mutations(&mut ctx2), "新文档首轮 drain 无事可做");
    assert_eq!(buf2, html2, "drain 不得改写新文档宿主 HTML");
    worker.shutdown();
}
