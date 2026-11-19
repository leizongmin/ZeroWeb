//! T84（pending overlay 双热点修复）常驻回归钉测。修复面：part05 桶级增量合并
//! 缓存 `_oc`（gen/seed/水位判据 + `_zwOcSpliceMaint` 水位维护）、Window named
//! 候选集 `_zwPANamedPending`（NA 采集扫描收窄）、锚改写检出作废
//! `_zwOcInvalidateAnchor*`（part01 link-write 合流点 + B1 重定向）。归因与
//! fail-before 证据：
//! `.acceptance/site-optimizer/perf-mem-20261023/bench/results/t84/fix/`。
//!
//! 钉测面三件：
//! ① 同 id 多候选 gEBI 树序（r4005 W3 形态）：`insertBefore` 前插移动后
//!    getElementById 须取树序首位（DOM spec： getElementById 按树序返回首个，
//!    https://dom.spec.whatwg.org/#dom-none-getelementbyid；树序
//!    #concept-tree-order）。`_oc` 缓存视图序 = 上次合并锚快照，锚改写不经
//!    桶数组变异时水位判据探不到，树序消费面会拿到陈旧序。
//! ② pending splice 混合序列视图一致性（r309 族）：merged 条目摘除须回退
//!    水位，增量合并结果 == 逐次全量合并，且 flush 落 host 后两侧一致
//!    （childList live 反射，https://dom.spec.whatwg.org/#dom-node-childnodes）。
//! ③ Window named access 迟到 id/name 授予（named 候选集维护面）：pending
//!    条目后补 `id`/`name` 属性仍须可解析（HTML spec：window named properties
//!    id face 任意元素、name face 限 embed/form/img/object，
//!    https://html.spec.whatwg.org/multipage/nav.html#dom-window-nameditem）。
//!
//! perf 护栏（判据线零改动前提下取内部倍率不变量）：在树 sel 父内新子 append
//! 序列（a3 形态，pending 积压 0→500 递增）的首窗/末窗墙钟倍率 + 总墙钟上限。
//! 修复前 overlay 全量重放 + NA 全量扫描随积压线性放大 → 总墙钟 O(n²)
//! （归因轮 debug 实测 a3 2629ms、b1 2941ms；末窗单 op ≈ 首窗 8-10 倍），
//! 修复后近似常数（bench18：a3 251ms、b1 419ms、a4 16.1s→4.1s、a4b 7.0s→0.30s）。

use zero_webview::{WebView, WebViewConfig};

/// 钉测页：`#feed`、`#list`、`#container`、`#app` 四个在树容器，无外链资源。
const PAGE: &str = "<!doctype html><html><body><div id=\"app\"></div><div id=\"feed\"></div><ul id=\"list\"></ul><div id=\"container\"></div></body></html>";

fn wv() -> WebView {
    WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    })
}

/// 页内时钟（performance.now 优先，Date.now 兜底），返回整毫秒字符串。
const CLK_JS: &str = "globalThis.__clk = (typeof performance !== 'undefined' && performance.now) ? function () { return performance.now(); } : function () { return Date.now(); };";

fn run(wv: &mut WebView, script: &str) -> String {
    wv.execute_script(script).expect("脚本执行成功")
}

/// 读回 globalThis.__hit / __order 之类的字符串探针。
fn probe(wv: &mut WebView, name: &str) -> String {
    run(wv, &format!("String(globalThis.{name} ?? '')"))
}

/// 钉①（r4005 W3）：pending 双同 id 候选 insertBefore 前插后，gEBI 取树序首位
/// （f5），body 内 data-x 序为 5,4；flush 落 host 后结论不变。修复中态实证：
/// `_oc` 冻结旧锚序时 gEBI 命中 f4、序 4,5（r4005dbg4-run.log）。
#[test]
#[serial_test::serial]
fn t84_same_id_tree_order_after_insertbefore_move() {
    let mut wv = wv();
    wv.load_html(PAGE, None);

    run(
        &mut wv,
        r#"
        var body = document.body;
        var f4 = document.createElement('form'); f4.id = 'rF4'; f4.setAttribute('data-x', '4'); body.appendChild(f4);
        var f5 = document.createElement('form'); f5.id = 'rF5'; f5.setAttribute('data-x', '5'); body.appendChild(f5);
        var c3 = document.createElement('button'); c3.setAttribute('form', 'rF4'); f4.appendChild(c3);
        globalThis.__f5 = f5;
        f5.id = 'rF4';
        f5.parentNode.insertBefore(f5, f4);
        var g = document.getElementById('rF4');
        globalThis.__hit = (g === f5) ? 'f5' : (g === f4 ? 'f4' : '?');
        var order = [];
        var kids = body.childNodes;
        for (var i = 0; i < kids.length; i++) {
            var dx = kids[i].getAttribute ? kids[i].getAttribute('data-x') : null;
            if (dx) order.push(dx);
        }
        globalThis.__order = order.join(',');
        "#,
    );

    let hit = probe(&mut wv, "__hit");
    assert_eq!(
        hit, "f5",
        "同 id 双候选 insertBefore 前插后 gEBI 须取树序首位（f5），实测 {hit:?}"
    );
    let order = probe(&mut wv, "__order");
    assert_eq!(order, "5,4", "body 内 data-x 序须为 5,4（前插生效），实测 {order:?}");

    // flush（drain ⇒ apply ⇒ 换代）落 host 后语义保持。
    wv.flush_pending_shared_mutations().expect("flush 落 host 成功");
    let hit_after = run(
        &mut wv,
        "(document.getElementById('rF4') === globalThis.__f5) ? 'f5' : '?'",
    );
    assert_eq!(
        hit_after, "f5",
        "flush 后 gEBI 仍须命中树序首位的 f5，实测 {hit_after:?}"
    );
}

/// 钉②（r309 族）：merged 条目摘除（桶中部 splice）+ 新增混合后，overlay
/// 视图序须等于逐次全量合并序；flush 落 host 后一致。序列先读 childNodes
/// 触发增量缓存，再在缓存存活下摘中部条目（水位回退路径）。
#[test]
#[serial_test::serial]
fn t84_pending_splice_view_consistency() {
    let mut wv = wv();
    wv.load_html(PAGE, None);

    // 建 8 条 pending → 读 childNodes（建缓存）→ 摘两个 merged 中部条目 →
    // 追加两条 → 再读（增量臂须给出与全量一致的序）。
    run(
        &mut wv,
        r#"
        var c = document.getElementById('container');
        for (var i = 0; i < 8; i++) {
            var d = document.createElement('div'); d.id = 'c84_' + i; c.appendChild(d);
        }
        function ids() {
            var out = [];
            var kids = c.childNodes;
            for (var i = 0; i < kids.length; i++) out.push(kids[i].id);
            return out.join(',');
        }
        globalThis.__before = ids();
        c.removeChild(document.getElementById('c84_3'));
        c.removeChild(document.getElementById('c84_1'));
        var d8 = document.createElement('div'); d8.id = 'c84_8'; c.appendChild(d8);
        var d9 = document.createElement('div'); d9.id = 'c84_9'; c.appendChild(d9);
        globalThis.__after = ids();
        "#,
    );
    let before = probe(&mut wv, "__before");
    assert_eq!(
        before, "c84_0,c84_1,c84_2,c84_3,c84_4,c84_5,c84_6,c84_7",
        "追加后首读须全量序，实测 {before:?}"
    );
    let after = probe(&mut wv, "__after");
    assert_eq!(
        after, "c84_0,c84_2,c84_4,c84_5,c84_6,c84_7,c84_8,c84_9",
        "merged 中部摘除 + 追加后 overlay 视图须等于全量合并序（水位回退正确），实测 {after:?}"
    );

    // flush 落 host 后两侧一致（防 cache/宿主漂移）。
    wv.flush_pending_shared_mutations().expect("flush 落 host 成功");
    let after_flush = run(
        &mut wv,
        r#"
        var c = document.getElementById('container');
        var out = [];
        var kids = c.childNodes;
        for (var i = 0; i < kids.length; i++) out.push(kids[i].id);
        out.join(',')
        "#,
    );
    assert_eq!(
        after_flush, "c84_0,c84_2,c84_4,c84_5,c84_6,c84_7,c84_8,c84_9",
        "flush 落 host 后子序须与 overlay 视图一致，实测 {after_flush:?}"
    );
}

/// 钉③：pending 条目后补 id / name 属性（named 候选集迟到登记路径）后，
/// Window named access 仍须解析。交付面语义：setAttribute('id'/'name') 走
/// `_zwNAAttrChanged` 同步尾叫（同 turn 安装）。无 name/id 的条目不得误命中
/// 任意名。
#[test]
#[serial_test::serial]
fn t84_named_access_late_attribute_grant() {
    let mut wv = wv();
    wv.load_html(PAGE, None);

    // 同 turn 臂：setAttribute 授名（同步 `_zwNAAttrChanged` → `_zwNARegisterName`）。
    run(
        &mut wv,
        r#"
        var d1 = document.createElement('div');
        document.body.appendChild(d1);
        d1.setAttribute('id', 'lateId84');
        var fm = document.createElement('form');
        document.body.appendChild(fm);
        fm.setAttribute('name', 'lateForm84');
        var plain = document.createElement('div');
        plain.setAttribute('data-x', 'plain');
        document.body.appendChild(plain);
        globalThis.__d1 = d1;
        globalThis.__fm = fm;
        globalThis.__d2 = null;
        globalThis.__hitId = (window.lateId84 === d1);
        globalThis.__hitForm = (window.lateForm84 === fm);
        globalThis.__missPlain = (window.lateId84 !== plain) && (window.lateForm84 !== plain);
        "#,
    );
    let hit_id = probe(&mut wv, "__hitId");
    assert_eq!(
        hit_id, "true",
        "pending 迟到 setAttribute('id') 授名后 window.<id> 须同 turn 解析到该元素，实测 {hit_id:?}"
    );
    let hit_form = probe(&mut wv, "__hitForm");
    assert_eq!(
        hit_form, "true",
        "pending 迟到 setAttribute('name') 授名后 window.<name> 须同 turn 解析到 form，实测 {hit_form:?}"
    );
    let miss = probe(&mut wv, "__missPlain");
    assert_eq!(
        miss, "true",
        "无 id/name 的 pending 条目不得被 named access 误命中，实测 {miss:?}"
    );

    // flush 后语义保持（落 host 后 named 解析不受 materialize 影响）。
    wv.flush_pending_shared_mutations().expect("flush 落 host 成功");
    let after = run(
        &mut wv,
        "(window.lateId84 === globalThis.__d1) && (window.lateForm84 === globalThis.__fm) ? 'ok' : 'broken'",
    );
    assert_eq!(after, "ok", "flush 后 named access 语义保持，实测 {after:?}");
    // 已知既有缺口（非本修复引入，stash 三角定位 itest5-base 双侧同败）：IDL `.id=`
    // 属性写对 pending 元素走批量 attributes notify，本链 drain 后 window 命名属性
    // 仍不安装——与修复无关的 pre-existing 面，不在此钉（不扩大范围）。
}

/// perf 护栏（a3 形态超线性钉）：在树 sel 父（#feed）内 500 次新子 append，
/// pending 积压 0→500 递增，分段计墙钟（首窗 1-100 / 中窗 101-450 / 末窗
/// 451-500）。修复前 overlay 全量重放 + NA 全量扫描随积压线性放大 → 总墙钟
/// O(n²)：本测 base 实测（itest5-base-run.log，stash 三角定位）early=181ms /
/// mid=2072ms / late=537ms / total=2790ms（末窗单 op ≈ 首窗 5.9 倍）；修复后
/// 增量合并 + 候选集扫描近似常数（itest7 实测 early=46ms / mid=111ms /
/// late=8ms / total=165ms，分离 ≈ 17 倍）。判据：末窗 ≤ 6 × 首窗 + 80ms
/// （形状护栏——base 2.96 倍不过此门，真判别靠总墙钟）且总墙钟 < 1200ms
/// （修复后 7× 余量、base 2.3× 超出；debug profile 口径）。
#[test]
#[serial_test::serial]
fn t84_pending_overlay_append_superlinearity_ceiling() {
    let mut wv = wv();
    wv.load_html(PAGE, None);

    run(
        &mut wv,
        &format!(
            r#"
            {CLK_JS}
            var host = document.getElementById('feed');
            function seg(from, to) {{
                var t0 = globalThis.__clk();
                for (var i = from; i <= to; i++) {{
                    var x = document.createElement('article');
                    x.textContent = 'a3_' + i;
                    host.appendChild(x);
                }}
                return globalThis.__clk() - t0;
            }}
            globalThis.__tEarly = seg(1, 100);
            globalThis.__tMid = seg(101, 450);
            globalThis.__tLate = seg(451, 500);
            "#,
        ),
    );
    let t_early: f64 = probe(&mut wv, "__tEarly").trim().parse().expect("首窗墙钟可解析");
    let t_mid: f64 = probe(&mut wv, "__tMid").trim().parse().expect("中窗墙钟可解析");
    let t_late: f64 = probe(&mut wv, "__tLate").trim().parse().expect("末窗墙钟可解析");

    assert!(
        t_late <= 6.0 * t_early + 80.0,
        "pending 积压递增下 append 墙钟超线性放大：末窗={t_late}ms, 首窗={t_early}ms（overlay 全量重放/NA 全量扫描回归）"
    );
    let total = t_early + t_mid + t_late;
    println!("t84 perf: early={t_early}ms mid={t_mid}ms late={t_late}ms total={total}ms");
    assert!(
        total < 1200.0,
        "append 序列总墙钟超上限：{total}ms（首窗 {t_early} + 中窗 {t_mid} + 末窗 {t_late}，防失控回退）"
    );
}
