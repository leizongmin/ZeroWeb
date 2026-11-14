//! T70（js-dom live 语义三隙修复，t69 诊断 Gap B）：childNodes live collection
//! 跨换代语义钉测。旧实现 `_zwChildBaseInvalidateAll`（回调重注册/换代钩子路径）
//! 整表清空 `_zwLiveNLCache`/`_zwLiveNLSync`：
//! ① 同节点换代后读返新数组——[SameObject] 破（spec：Node.childNodes 返 live
//!    NodeList，https://dom.spec.whatwg.org/#dom-node-childnodes；同一 node 的
//!    collection 对象恒一，#interface-nodelist / #collections）；
//! ② 持有引用失去 mutation 同步钩子——树变更只刷新换代后的新数组，旧引用冻结
//!    （live 反射破；spec：live collection 须反映文档树变更，#collections）；
//! ③ 树变更与视图翻转的配对面（MutationObserver record 必须与真实变更同入队，
//!    DOM §4.2.2 queue a mutation record，#queue-a-mutation-record）。
//!
//! 修复后承载数组保留（512 键软上限），refresh 闭包按 sel/handle 重查
//! `_childNodeList`（base cache 每换代清空回源 host，基底新鲜度不变）。
//! 驱动形 = 进程内 webview（`execute_script` + `flush_pending_shared_mutations`
//! 即 drain ⇒ apply ⇒ bump ⇒ 回调重注册全链），与 t69-minrepro caseH 同机制。

use zero_webview::{WebView, WebViewConfig};

/// 钉测页：单容器 `#wrap`（初始一个子 `p.old`），无外链资源。
const PAGE: &str = "<!doctype html><html><body><div id=\"wrap\"><p class=\"old\">o</p></div></body></html>";

fn wv() -> WebView {
    WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    })
}

/// 钉①[SameObject]：同节点重复读返同一 collection 对象，且跨换代（flush ⇒
/// apply ⇒ bump ⇒ 重注册）保持恒一 + 内容仍新鲜（held 引用反映换代前落队
/// 的 append）。修复前：换代整表清空 → 换代后首读返新数组（identity 破）。
#[test]
#[serial_test::serial]
fn t70_childnodes_sameobject_identity_across_generation() {
    let mut wv = wv();
    wv.load_html(PAGE, None);

    // 同 turn：持有 live 引用，同节点重复读恒一。
    wv.execute_script(
        r#"
        var w = document.getElementById('wrap');
        globalThis.__nlHold = w.childNodes;
        globalThis.__sameTurn = w.childNodes === globalThis.__nlHold;
        "#,
    )
    .expect("同 turn 持有脚本成功");
    let same_turn = wv
        .execute_script("String(globalThis.__sameTurn)")
        .expect("同 turn 读回成功");
    assert!(
        same_turn.contains("true"),
        "同 turn childNodes 重复读须返同一对象（[SameObject]），实测 {same_turn:?}"
    );

    // 真实 mutation 落队 → flush 走全链（空批早退不换代，钉不到目标边界）。
    wv.execute_script(
        r#"
        var a = document.createElement('div');
        a.id = 'gen-a';
        document.getElementById('wrap').appendChild(a);
        "#,
    )
    .expect("换代前 append 成功");
    wv.flush_pending_shared_mutations()
        .expect("flush（drain ⇒ apply ⇒ bump ⇒ 重注册）成功");

    // 换代后：identity 恒一 + held 内容反映已落 host 的 append（长度 2）。
    let after = wv
        .execute_script(
            r#"
            var w = document.getElementById('wrap');
            globalThis.__sameAfter = (w.childNodes === globalThis.__nlHold);
            globalThis.__lenAfter = globalThis.__nlHold.length;
            String(globalThis.__sameAfter) + ',' + String(globalThis.__lenAfter)
            "#,
        )
        .expect("换代后读回成功");
    assert!(
        after.contains("true,2"),
        "换代后 childNodes 须与持有引用恒一（修复前整表清空返新数组）且 held 长度反映落 host 的 append（2），实测 {after:?}"
    );
}

/// 钉② live 反射：持有引用跨换代后，新树变更即时反映到既有 collection 对象
///（`_zwLiveNLSync` 钩子在 mutation 入口同步承载数组）。修复前：换代清空
/// sync 表 → 换代后 append 的 sync 调用失联 → held 冻结在旧视图（长度 2）。
#[test]
#[serial_test::serial]
fn t70_childnodes_held_live_reflection_across_drain() {
    let mut wv = wv();
    wv.load_html(PAGE, None);

    // 持有引用 + 换代前 append（同 turn sync 在场，held 即时反映）。
    wv.execute_script(
        r#"
        var w = document.getElementById('wrap');
        globalThis.__nlHold = w.childNodes;
        var a = document.createElement('div');
        a.id = 'gen-a';
        w.appendChild(a);
        globalThis.__preLen = globalThis.__nlHold.length;
        "#,
    )
    .expect("持有 + 同 turn append 成功");
    let pre = wv
        .execute_script("String(globalThis.__preLen)")
        .expect("同 turn held 长度读回成功");
    assert!(
        pre.contains("2"),
        "同 turn append 后 held 须即时反映（长度 2），实测 {pre:?}"
    );

    wv.flush_pending_shared_mutations().expect("flush（换代）成功");

    // 换代后 append（中途不做任何 fresh childNodes 读——sync 表只能由 getter
    // 重注册；修复前本步 sync 失联，held 冻结在 [p.old, gen-a]）。
    let after = wv
        .execute_script(
            r#"
            var w = document.getElementById('wrap');
            var b = document.createElement('div');
            b.id = 'gen-b';
            w.appendChild(b);
            globalThis.__heldHasB = Array.prototype.indexOf.call(globalThis.__nlHold, b) >= 0;
            globalThis.__heldLen = globalThis.__nlHold.length;
            String(globalThis.__heldHasB) + ',' + String(globalThis.__heldLen)
            "#,
        )
        .expect("换代后 append + held 读回成功");
    assert!(
        after.contains("true,3"),
        "换代后 append 须即时反映到既有 collection（heldHasB=true 且长度 3；修复前 sync 表清空 held 冻结长度 2），实测 {after:?}"
    );
}

/// 钉③ 可观测性：树变更（视图翻转的同一 mutation 入口）必须同入队 Mutation
///Observer record，record 的 target/addedNodes 保持节点 identity（DOM §4.2.2
/// queue a mutation record，https://dom.spec.whatwg.org/#queue-a-mutation-record）。
/// 同 turn 与换代后各钉一次（takeRecords 同步取队，规避回调时序）。修复不得
/// 产生「视图变了而无 record」或「record 身份失配」。
#[test]
#[serial_test::serial]
fn t70_childnodes_view_flip_queues_mutation_record() {
    let mut wv = wv();
    wv.load_html(PAGE, None);

    // 同 turn：append ⇒ record 入队 + 身份对齐。
    let same_turn = wv
        .execute_script(
            r#"
            var mo = new MutationObserver(function () {});
            mo.observe(document.documentElement, { childList: true, subtree: true });
            var w = document.getElementById('wrap');
            var x = document.createElement('div');
            x.id = 'mx';
            w.appendChild(x);
            var recs = mo.takeRecords();
            globalThis.__pin3Same = recs.length >= 1
                && recs[recs.length - 1].target === w
                && recs[recs.length - 1].addedNodes[0] === x;
            String(globalThis.__pin3Same)
            "#,
        )
        .expect("同 turn MO 钉脚本成功");
    assert!(
        same_turn.contains("true"),
        "同 turn append 须入队 record 且 target/addedNodes 身份对齐，实测 {same_turn:?}"
    );

    wv.flush_pending_shared_mutations().expect("flush（换代）成功");

    // 换代后：record 配对保持（视图翻转不得绕过 MO）。
    let after = wv
        .execute_script(
            r#"
            var mo = new MutationObserver(function () {});
            mo.observe(document.documentElement, { childList: true, subtree: true });
            var w = document.getElementById('wrap');
            var y = document.createElement('div');
            y.id = 'my';
            w.appendChild(y);
            var recs = mo.takeRecords();
            globalThis.__pin3After = recs.length >= 1
                && recs[recs.length - 1].target === w
                && recs[recs.length - 1].addedNodes[0] === y;
            String(globalThis.__pin3After)
            "#,
        )
        .expect("换代后 MO 钉脚本成功");
    assert!(
        after.contains("true"),
        "换代后 append 仍须入队 record 且身份对齐（视图翻转不得绕过 MO），实测 {after:?}"
    );
}
