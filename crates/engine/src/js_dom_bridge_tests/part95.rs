// s229（html-semantics M2，2026-11-02）：`:active` 按压态槽归属契约——thread_local。
// 根因：M2-S196 曾用进程级 static Mutex 承载 per-document 按压态，runner rayon
// 多 job（每线程独立 WebView 顺序多页）下 A 页写入串进并发 B 页查询（WPT
// active-onblur button 页 `:active` 假匹配 body 实证——本页从未标记过 body）。
// 本测试钉两点：他线程写入的按压选择器在本线程 sync 后不得出现；同线程写读
// 回路保持。归属模型判据见 DOC_ACTIVE_SELECTOR 声明处注记。
// https://html.spec.whatwg.org/multipage/interaction.html#selector-active
#[test]
fn s229_doc_active_slot_thread_local() {
    let writer = std::thread::spawn(|| {
        DOC_ACTIVE_SELECTOR.with(|slot| *slot.borrow_mut() = "#s229-bleed-probe".to_string());
    });
    writer.join().expect("写入线程须正常结束");

    let mut doc = zero_dom::parse_html(
        "<html><body><div id=\"s229-bleed-probe\">x</div></body></html>",
    );
    sync_active_state(&mut doc);
    let hit = doc.query_selector(doc.root(), ":active");
    assert!(
        hit.is_none(),
        "他线程写入的按压选择器不得串入本线程查询视图"
    );

    DOC_ACTIVE_SELECTOR.with(|slot| *slot.borrow_mut() = "#s229-bleed-probe".to_string());
    sync_active_state(&mut doc);
    let hit = doc.query_selector(doc.root(), ":active");
    assert!(hit.is_some(), "同线程按压写入须在本线程查询可见");

    // 收尾清槽（测试共用主线程，不给他测试留串态）。
    DOC_ACTIVE_SELECTOR.with(|slot| *slot.borrow_mut() = String::new());
}
