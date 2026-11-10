// P39r2（siteopt github r2，2026-11-11）：`_realTag`/`_zwSelNs` stamp 失配窗口章
// 快道常驻回归（epoch 界定语义）。红态（修复前）：每脚本 mutation drain 后
// `_zwTagCache`/`_zwSelNsCache` 印章三元组失配，大 DOM（github home 8000 元素）
// 每次 tagName/localName/nodeName/namespaceURI 读都打 `__zw_get_tag`/`__zw_get_ns`
// 宿主往返（75s 全量谱 get_tag 20.6 万 + get_ns 20.4 万 = 55%）。r1（45e1be731）
// 以跨视图永久章消往返，但「drain→apply 间隙宿主视图未变」前提被缺陷首轮
// DF-1/DF-2 执行探针证伪（tag 面宿主查询在记账时即重放 pending、ns 面回归窗口自
// apply 起算）：r2 改 epoch 界定——记账/apply/reset 三钩子强制清章 + 读取侧
// (gen, pending 数组身份, 长度) 惰性闸，同 sel 失配窗口首读一次宿主重灌、二读零
// 往返。spec tagName/namespaceURI 不可变是对同一元素而言（sel↔元素绑定可变）：
// https://dom.spec.whatwg.org/#dom-element-tagname、#dom-node-namespaceuri。
// 注意：include! 共享模块命名空间（part98 范式）——类型一律 inline 全路径，不写 use。

/// P39 共用基建：shim + tagged payload / get_tag / get_ns 计数宿主 / 可变 stamp 槽 /
/// report 收集。stamp 与 payload 各持独立可变槽（权威演进臂联动换 stamp，防
/// `_zwDocAllSnapEls` 快道以旧 stamp 拦截新 payload）。get_tag/get_ns 权威值均从
/// payload 解析（sel\x1ftag[\x1fns] 记录）——stub 与宿主视图同源，ns 第三段缺席
/// 即 HTML（XHTML ns）。
struct P39Rig {
    sandbox: zero_script_sandbox::V8Sandbox,
    tag_calls: std::sync::Arc<std::sync::atomic::AtomicU32>,
    /// 按 sel 计数——appendChild 等写链路会以 body 等旁支 sel 打同一回调，总数断言
    /// 无法区分目标 sel 的重灌次数（test4/test7 用 per-sel 口径）。
    tag_calls_by_sel: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, u32>>>,
    ns_calls: std::sync::Arc<std::sync::atomic::AtomicU32>,
    payload: std::sync::Arc<std::sync::Mutex<String>>,
    stamp: std::sync::Arc<std::sync::Mutex<String>>,
    reports: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl P39Rig {
    fn new(payload: &str) -> Self {
        use zero_script_sandbox::{Sandbox, SandboxConfig, V8Sandbox};
        let mut sandbox = V8Sandbox::with_config(SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        // persistent context：shim 装载与调用体分多次 execute，global 须共享。
        sandbox.execute(generate_js_dom_shim()).unwrap();

        let tag_calls = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let tag_calls_by_sel =
            std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
        let ns_calls = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let payload = std::sync::Arc::new(std::sync::Mutex::new(payload.to_string()));
        let stamp = std::sync::Arc::new(std::sync::Mutex::new("s1".to_string()));
        let reports = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

        let payload_cb = payload.clone();
        sandbox.register_callback("__zw_query_all_tagged", Box::new(move |_args| {
            payload_cb.lock().unwrap().clone()
        }));
        let tag_calls_cb = tag_calls.clone();
        let tag_by_sel_cb = tag_calls_by_sel.clone();
        let payload_for_tag = payload.clone();
        sandbox.register_callback("__zw_get_tag", Box::new(move |args| {
            tag_calls_cb.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let sel = args.first().cloned().unwrap_or_default();
            *tag_by_sel_cb.lock().unwrap().entry(sel.clone()).or_insert(0) += 1;
            // 权威值从 payload 解析（sel\x1ftag 记录）——stub 与宿主视图同源。
            let payload = payload_for_tag.lock().unwrap().clone();
            for record in payload.split('|') {
                let mut parts = record.split('\u{1f}');
                if let (Some(s), Some(t)) = (parts.next(), parts.next())
                    && s == sel
                {
                    return t.to_string();
                }
            }
            String::new()
        }));
        let ns_calls_cb = ns_calls.clone();
        let payload_for_ns = payload.clone();
        sandbox.register_callback("__zw_get_ns", Box::new(move |args| {
            ns_calls_cb.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let sel = args.first().cloned().unwrap_or_default();
            // 权威 ns 从 payload 第三段解析（sel\x1ftag\x1fns），缺席即 XHTML——
            // P39r2 DF-2 回归臂需要 ns 权威可演进（SVG→XHTML 重挂）。
            let payload = payload_for_ns.lock().unwrap().clone();
            for record in payload.split('|') {
                let mut parts = record.split('\u{1f}');
                if let (Some(s), Some(_t)) = (parts.next(), parts.next())
                    && s == sel
                {
                    return parts
                        .next()
                        .map(str::to_string)
                        .unwrap_or_else(|| "http://www.w3.org/1999/xhtml".to_string());
                }
            }
            "http://www.w3.org/1999/xhtml".to_string()
        }));
        let stamp_cb = stamp.clone();
        sandbox.register_callback("__zw_dom_view_stamp", Box::new(move |_args| {
            stamp_cb.lock().unwrap().clone()
        }));
        // DOM 变异驱动臂（test4 记账回归）：createElement/appendChild 的宿主写回调
        //（handle 字符串协议，fire-and-forget）——无宿主真树，仅让写链路走到
        // `_zwHCLiveInvalidate` 记账咽喉。
        sandbox.register_callback("__zw_create_element", Box::new(move |args| {
            format!("{}#zwtest-auto", args.first().cloned().unwrap_or_default())
        }));
        sandbox.register_callback("__zw_append_child", Box::new(move |_args| String::new()));
        let reports_cb = reports.clone();
        sandbox.register_callback("__zw_report", Box::new(move |args| {
            reports_cb
                .lock()
                .unwrap()
                .push(args.first().cloned().unwrap_or_default());
            String::new()
        }));
        Self {
            sandbox,
            tag_calls,
            tag_calls_by_sel,
            ns_calls,
            payload,
            stamp,
            reports,
        }
    }

    fn set_authority(&self, payload: &str, stamp: &str) {
        *self.payload.lock().unwrap() = payload.to_string();
        *self.stamp.lock().unwrap() = stamp.to_string();
    }

    /// 目标 sel 的宿主查询次数（写链路旁支 sel 不计入断言）。
    fn tag_calls_for(&self, sel: &str) -> u32 {
        self.tag_calls_by_sel
            .lock()
            .unwrap()
            .get(sel)
            .copied()
            .unwrap_or(0)
    }

    fn report(&self) -> String {
        let mut reports = self.reports.lock().unwrap();
        let joined = reports.join("|");
        reports.clear();
        joined
    }

    fn run(&mut self, js: &str) {
        use zero_script_sandbox::Sandbox;
        self.sandbox.execute(js).expect("execute 失败");
    }
}

#[test]
fn realtag_stamp_mismatch_serves_seen_tag_epoch_scoped_p39() {
    let mut rig = P39Rig::new("div#a\u{1f}DIV|span#b\u{1f}SPAN");

    // ①表权威路径首读（tagged 一次往返灌注 `_zwTagCache`+`_zwSelNsCache`，第三段
    // 缺席即 HTML 归一 ''）→ ②模拟 mutation drain 失配（`_zwApplyGeneration` 换
    // 返回值）→ ③失配窗口再读已权威读过的元素：r2 epoch 界定下章已清，同 sel
    // 首读付一次宿主重灌（返回当前权威 DIV），二读走章零宿主。红态（修复前）
    // 每次读打一次 `__zw_get_tag`（本测 2 次）；r1 永久章 0 次——0 次 = 跨 epoch
    // 服务旧值的不健全语义，本断言钉住「首读 1 次」的正确性价。
    rig.run(
        "var els = document.getElementsByTagName('*');\n\
         var t1 = els[0].tagName;\n\
         var t1b = els[1].tagName;\n\
         __zw_report([t1, t1b].join(','));\n",
    );
    assert_eq!(rig.report(), "DIV,SPAN", "新鲜窗口 tagName 读走表权威");
    let ns_after_first = rig.ns_calls.load(std::sync::atomic::Ordering::Relaxed);
    let tag_after_first = rig.tag_calls.load(std::sync::atomic::Ordering::Relaxed);
    rig.run(
        "globalThis._zwApplyGeneration = function () { return 999; };\n\
         var t2 = els[0].tagName;\n\
         var t3 = els[0].tagName;\n\
         __zw_report([t2, t3].join(','));\n",
    );
    assert_eq!(
        rig.report(),
        "DIV,DIV",
        "失配窗口已读元素 tagName 读与权威一致（首读宿主重灌 + 二读章服务）"
    );
    assert_eq!(
        rig.tag_calls.load(std::sync::atomic::Ordering::Relaxed) - tag_after_first,
        1,
        "失配窗口同 sel 首读恰一次宿主重灌（epoch 界定；修复前 2 次、r1 永久章 0 次）"
    );
    assert_eq!(
        rig.ns_calls.load(std::sync::atomic::Ordering::Relaxed) - ns_after_first,
        1,
        "tag 面 foreign 判定的 ns 读同 epoch 界定：首读一次重灌，二读零宿主"
    );
}

#[test]
fn selns_stamp_mismatch_serves_seen_ns_epoch_scoped_p39() {
    let mut rig = P39Rig::new("div#a\u{1f}DIV|span#b\u{1f}SPAN");

    // namespaceURI getter 走规范 face（WHATWG DOM：HTML 元素 namespaceURI = XHTML ns，
    // https://dom.spec.whatwg.org/#dom-node-namespaceuri——与 `_zwSelNs` 内部归一 ''
    // （foreign 判定）是两个语义）；tagged 灌注后表权威零宿主，失配窗口章服务
    // epoch 界定重灌（首读一次宿主，二读零）。
    const XHTML: &str = "http://www.w3.org/1999/xhtml";
    rig.run(
        "var els = document.getElementsByTagName('*');\n\
         var n1 = els[0].namespaceURI;\n\
         __zw_report([String(n1)].join(','));\n",
    );
    assert_eq!(
        rig.report(),
        XHTML,
        "HTML 元素 namespaceURI 返回规范 XHTML ns（getter 规范 face）"
    );
    let ns_after_first = rig.ns_calls.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(
        ns_after_first, 0,
        "tagged 灌注后表权威路径零宿主（第三段缺席即 HTML ns）"
    );
    rig.run(
        "globalThis._zwApplyGeneration = function () { return 4242; };\n\
         var n2 = els[0].namespaceURI;\n\
         var n3 = els[0].namespaceURI;\n\
         __zw_report([String(n2), String(n3)].join(','));\n",
    );
    assert_eq!(
        rig.report(),
        format!("{XHTML},{XHTML}"),
        "失配窗口已读元素 namespaceURI 读与权威一致（首读重灌 + 二读章服务）"
    );
    assert_eq!(
        rig.ns_calls.load(std::sync::atomic::Ordering::Relaxed),
        1,
        "失配窗口同 sel ns 首读恰一次宿主重灌（epoch 界定；修复前 2 次、r1 永久章 0 次）"
    );
}

#[test]
fn tag_authority_evolution_refreshes_seen_tag_p39() {
    let mut rig = P39Rig::new("div#a\u{1f}DIV");

    // 权威演进：同 sel 重绑定换 tag（id-only sel 形态）——apply 后 stamp 新鲜窗口
    // 走重灌权威表（章不参与），表命中顺带刷新章；再入失配窗口须服务**新**权威，
    // 不得回落旧章值（DIV）。
    rig.run(
        "var els = document.getElementsByTagName('*');\n\
         var v1 = els[0].tagName;\n\
         globalThis._zwApplyGeneration = function () { return 900; };\n\
         var v2 = els[0].tagName;\n\
         __zw_report([v1, v2].join(','));\n",
    );
    assert_eq!(rig.report(), "DIV,DIV", "视图1 权威 DIV：表读与失配章读一致");

    // 宿主权威演进（同 sel 换 tag + 换代 stamp——模拟 apply 后新树换代）。
    rig.set_authority("div#a\u{1f}SPAN", "s2");
    rig.run(
        "globalThis._zwApplyGeneration = function () { return 0; };\n\
         var els = document.getElementsByTagName('*');\n\
         var v3 = els[0].tagName;\n\
         globalThis._zwApplyGeneration = function () { return 901; };\n\
         var v4 = els[0].tagName;\n\
         __zw_report([v3, v4].join(','));\n",
    );
    assert_eq!(
        rig.report(),
        "SPAN,SPAN",
        "apply 后权威 SPAN：新鲜窗口表读返 SPAN；再入失配窗口章已刷新不得服务旧 DIV"
    );
}

/// 缺陷首轮 DF-1 回归：结构性记账（pending 增长，gen 不变）即视图演进——tag 面
/// 宿主查询在记账时即重放 pending 结构性 mutations（with_query_view_doc），章跨
/// 记账窗口服务旧值 = 服务旧元素（探针实证 DIV|host=SPAN）。r2 由
/// `_zwHCLiveInvalidate` 尾部强制清章覆盖；真实 DOM 变异（appendChild）驱动记账。
/// 牙口说明：push 形态记账的指纹必变，读取侧惰性闸同样翻转清章（纵深防御双通
/// 道）——本测对「r1 永久章（无钩子无闸）」红，对「r2 单去记账钩子」绿（闸兜
/// 底）；记账钩子的独立增量在指纹不变形态（splice+append 恢复长度）与内存回收
/// 锚点，后者无法用行为断言单独钉住，见生产代码注释。
#[test]
fn realtag_accounting_evolution_serves_new_authority_p39r2() {
    let mut rig = P39Rig::new("div#a\u{1f}DIV|span#b\u{1f}SPAN");
    rig.run(
        "var els = document.getElementsByTagName('*');\n\
         var t1 = els[0].tagName;\n\
         __zw_report(t1);\n",
    );
    assert_eq!(rig.report(), "DIV", "记账前权威 DIV（表读 + 章写入）");

    // 宿主权威演进（同 sel 换 tag DIV→SPAN；sel 缺席会落空权威 sel 推断兜底，
    // 不是本测语义）+ 真实 DOM 变异驱动记账（gen 不变、pending 增长）。
    rig.set_authority("div#a\u{1f}SPAN|span#b\u{1f}SPAN", "s2");
    rig.run(
        "document.body.appendChild(document.createElement('hr'));\n\
         var t2 = els[0].tagName;\n\
         var t3 = els[0].tagName;\n\
         __zw_report([t2, t3].join(','));\n",
    );
    assert_eq!(
        rig.report(),
        "SPAN,SPAN",
        "记账演进（gen 不变）后同 sel 读必须返回新权威（章随记账清空、宿主重灌），不得服务旧章 DIV"
    );
    assert_eq!(
        rig.tag_calls_for("div#a"),
        1,
        "记账后同 sel 恰一次宿主重灌（per-sel 口径；二读章服务；appendChild 写链路的 body 等旁支查询不计）"
    );
}

/// 缺陷首轮 DF-2 回归：apply 换代（gen bump）后 ns 面章不得服务旧值——
/// `__zw_get_ns` 纯快照查询，ns 回归窗口自 apply 起算（探针实证
/// svg-ns|host=xhtml-ns）。r2 由 `__zw_apply_generation_bump` 内强制清章覆盖；
/// 真实 bump 钩子驱动（非 gen 覆写），权威经 payload 第三段消失演进（SVG→XHTML）。
/// 牙口说明：bump 的 gen 真实递增指纹必变，惰性闸重叠兜底（同 test4 双通道）——
/// 本测对「r1 永久章」红、钉住 apply 后 ns 权威读行为判据；bump 钩子独立增量同
/// test4 注记。
#[test]
fn selns_apply_evolution_serves_new_authority_p39r2() {
    const XHTML: &str = "http://www.w3.org/1999/xhtml";
    const SVG: &str = "http://www.w3.org/2000/svg";
    let mut rig = P39Rig::new(&format!("svg#a\u{1f}SVG\u{1f}{SVG}"));
    rig.run(
        "var els = document.getElementsByTagName('*');\n\
         var n1 = els[0].namespaceURI;\n\
         __zw_report([String(n1)].join(','));\n",
    );
    assert_eq!(rig.report(), SVG, "SVG 权威：namespaceURI 返回 SVG ns");

    // 权威演进（第三段消失 = XHTML）+ 真实 apply 代际 bump。
    rig.set_authority("div#a\u{1f}DIV", "s2");
    rig.run(
        "globalThis.__zw_apply_generation_bump();\n\
         var n2 = els[0].namespaceURI;\n\
         var n3 = els[0].namespaceURI;\n\
         __zw_report([String(n2), String(n3)].join(','));\n",
    );
    assert_eq!(
        rig.report(),
        format!("{XHTML},{XHTML}"),
        "apply 换代后 ns 读必须返回新权威（章随 apply 清空、宿主重灌），不得服务旧 SVG ns"
    );
    assert_eq!(
        rig.ns_calls.load(std::sync::atomic::Ordering::Relaxed),
        1,
        "apply 后同 sel ns 恰一次宿主重灌（二读章服务）"
    );
}

/// 缺陷首轮 DF-3 回归（变异②牙口）：失配窗口章服务必须按 sel 键控——两 sel
/// 各取各值。变异「章命中即服务任意已存章值（first-value 化）」下 els[1] 会错取
/// els[0] 的 DIV（def 首轮 cargo-wt-p39-mut2.log 实证旧测试 5/5 全绿拦不住）。
#[test]
fn stamp_memo_is_sel_keyed_two_sels_independent_p39r2() {
    let mut rig = P39Rig::new("div#a\u{1f}DIV|span#b\u{1f}SPAN");
    rig.run(
        "var els = document.getElementsByTagName('*');\n\
         var a1 = els[0].tagName;\n\
         var b1 = els[1].tagName;\n\
         __zw_report([a1, b1].join(','));\n",
    );
    assert_eq!(rig.report(), "DIV,SPAN", "新鲜窗口两 sel 各取各值");
    rig.run(
        "globalThis._zwApplyGeneration = function () { return 777; };\n\
         var a2 = els[0].tagName;\n\
         var b2 = els[1].tagName;\n\
         __zw_report([a2, b2].join(','));\n",
    );
    assert_eq!(
        rig.report(),
        "DIV,SPAN",
        "失配窗口两 sel 章服务保持 sel 键控（各宿主重灌各值）"
    );
    assert_eq!(
        rig.tag_calls.load(std::sync::atomic::Ordering::Relaxed),
        2,
        "两 sel 各一次宿主重灌（first-value 化变异下 els[1] 零宿主错取 DIV）"
    );
}

/// reset 碰撞回归：`__zw_reset_pending_state` 不 bump 代际且 pending 一直为空——
/// reset 前后印章指纹逐位相等，惰性闸 no-op，章不得跨「快照换代」（宿主权威演进
/// 与 reset 无关）服务。gen 覆写刻意先于首读（惰性闸 marker 随首读建立，若覆写在
/// marker 建立之后，闸会自行翻转清章、reset 钩子不被测到——首轮自查实证）；表被
/// reset 既有语义清空（`_zwTagCache = null`），reset 后读表必 miss，章是唯一数据
/// 源：只有「章服务旧 DIV（红态：r1 永久章或 r2 去 reset 钩子）」与「宿主重灌新
/// SPAN（绿态）」两种可能。r2 由 reset 钩子强制清章覆盖。
#[test]
fn reset_pending_state_clears_stamp_memos_p39r2() {
    let mut rig = P39Rig::new("div#a\u{1f}DIV|span#b\u{1f}SPAN");
    rig.run(
        "globalThis._zwApplyGeneration = function () { return 999; };\n\
         var els = document.getElementsByTagName('*');\n\
         var t1 = els[0].tagName;\n\
         __zw_report(t1);\n",
    );
    assert_eq!(rig.report(), "DIV", "reset 前权威 DIV（表读 + 章写入 + 惰性闸 marker 建档）");

    // 宿主权威演进（同 sel 换 tag）+ reset（gen 仍 999、pending 一直为空——指纹
    // 逐位不变，惰性闸 no-op；红态此处章服务旧 DIV）。
    rig.set_authority("div#a\u{1f}SPAN|span#b\u{1f}SPAN", "s2");
    rig.run(
        "globalThis.__zw_reset_pending_state();\n\
         var t2 = els[0].tagName;\n\
         __zw_report(t2);\n",
    );
    assert_eq!(
        rig.report(),
        "SPAN",
        "reset（指纹不变）后章不得跨快照换代服务旧值，须宿主重灌新权威"
    );
    assert_eq!(
        rig.tag_calls_for("div#a"),
        1,
        "reset 后恰一次宿主重灌（表已清 miss + 章已清）"
    );
}
