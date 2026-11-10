// P39（siteopt github r1，2026-11-10）：`_realTag`/`_zwSelNs` stamp 失配窗口永久章
// 快道常驻回归。红态（修复前）：每脚本 mutation drain 后 `_zwTagCache`/`_zwSelNsCache`
// 印章三元组失配，大 DOM（github home 8000 元素）每次 tagName/localName/nodeName/
// namespaceURI 读都打 `__zw_get_tag`/`__zw_get_ns` 宿主往返（75s 全量谱 get_tag
// 20.6 万 + get_ns 20.4 万 = 55%），30 次换代 × 全量元素重查 = 页面脚本推进分钟级。
// 修复：失配窗口内 `_zwSelSeenTag`/`_zwSelNsSeen` 永久章服务（spec tagName/
// namespaceURI 不可变——https://dom.spec.whatwg.org/#dom-element-tagname、
// #dom-node-namespaceuri；drain→apply 间隙宿主视图未变，上一视图权威值即当前
// 权威值）；stamp 新鲜窗口表权威路径原样（命中刷章，防重绑定演进后服务旧章值）。
// 注意：include! 共享模块命名空间（part98 范式）——类型一律 inline 全路径，不写 use。

/// P39 共用基建：shim + tagged payload / get_tag / get_ns 计数宿主 / 可变 stamp 槽 /
/// report 收集。stamp 与 payload 各持独立可变槽（权威演进臂联动换 stamp，防
/// `_zwDocAllSnapEls` 快道以旧 stamp 拦截新 payload）。
struct P39Rig {
    sandbox: zero_script_sandbox::V8Sandbox,
    tag_calls: std::sync::Arc<std::sync::atomic::AtomicU32>,
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
        let ns_calls = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let payload = std::sync::Arc::new(std::sync::Mutex::new(payload.to_string()));
        let stamp = std::sync::Arc::new(std::sync::Mutex::new("s1".to_string()));
        let reports = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

        let payload_cb = payload.clone();
        sandbox.register_callback("__zw_query_all_tagged", Box::new(move |_args| {
            payload_cb.lock().unwrap().clone()
        }));
        let tag_calls_cb = tag_calls.clone();
        let payload_for_tag = payload.clone();
        sandbox.register_callback("__zw_get_tag", Box::new(move |args| {
            tag_calls_cb.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let sel = args.first().cloned().unwrap_or_default();
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
        sandbox.register_callback("__zw_get_ns", Box::new(move |_args| {
            ns_calls_cb.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            "http://www.w3.org/1999/xhtml".to_string()
        }));
        let stamp_cb = stamp.clone();
        sandbox.register_callback("__zw_dom_view_stamp", Box::new(move |_args| {
            stamp_cb.lock().unwrap().clone()
        }));
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
fn realtag_stamp_mismatch_serves_seen_tag_zero_host_roundtrip_p39() {
    let mut rig = P39Rig::new("div#a\u{1f}DIV|span#b\u{1f}SPAN");

    // ①表权威路径首读（tagged 一次往返灌注 `_zwTagCache`+`_zwSelNsCache`，第三段
    // 缺席即 HTML 归一 ''；tagName trap 的 foreign 判定 `_zwSelNs` 表命中零宿主）→
    // ②模拟 mutation drain 失配（`_zwApplyGeneration` 换返回值）→ ③失配窗口再读
    // 已权威读过的元素：修复后走 `_zwSelSeenTag`/`_zwSelNsSeen` 章快道零宿主；红态
    // 每次读打一次 `__zw_get_tag`。章边界：只服务已权威读过的 sel（生产 drain 清表
    // 后章是失配窗口唯一通道）；未读元素保持宿主权威价，不在本测断言面。
    rig.run(
        "var els = document.getElementsByTagName('*');\n\
         var t1 = els[0].tagName;\n\
         var t1b = els[1].tagName;\n\
         __zw_report([t1, t1b].join(','));\n",
    );
    assert_eq!(rig.report(), "DIV,SPAN", "新鲜窗口 tagName 读走表权威");
    let ns_after_first = rig.ns_calls.load(std::sync::atomic::Ordering::Relaxed);
    rig.run(
        "globalThis._zwApplyGeneration = function () { return 999; };\n\
         var t2 = els[0].tagName;\n\
         var t3 = els[0].tagName;\n\
         __zw_report([t2, t3].join(','));\n",
    );
    assert_eq!(
        rig.report(),
        "DIV,DIV",
        "失配窗口已读元素 tagName 读必须返回与上一视图权威一致的值（章服务）"
    );
    assert_eq!(
        rig.ns_calls.load(std::sync::atomic::Ordering::Relaxed),
        ns_after_first,
        "失配窗口已读元素的 namespaceURI 判定不得触发 __zw_get_ns 宿主往返"
    );
    assert_eq!(
        rig.tag_calls.load(std::sync::atomic::Ordering::Relaxed),
        0,
        "失配窗口已读元素 tagName 读不得触发 __zw_get_tag 宿主往返（章快道；红态此值为 2）"
    );
}

#[test]
fn selns_stamp_mismatch_serves_seen_ns_zero_host_roundtrip_p39() {
    let mut rig = P39Rig::new("div#a\u{1f}DIV|span#b\u{1f}SPAN");

    // namespaceURI getter 走规范 face（WHATWG DOM：HTML 元素 namespaceURI = XHTML ns，
    // https://dom.spec.whatwg.org/#dom-node-namespaceuri——与 `_zwSelNs` 内部归一 ''
    // （foreign 判定）是两个语义）；tagged 灌注后表权威零宿主，失配窗口章快道服务
    // 已读 sel，红态（无修复）失配读打宿主。
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
        "失配窗口已读元素 namespaceURI 读与首读同值（章服务）"
    );
    assert_eq!(
        rig.ns_calls.load(std::sync::atomic::Ordering::Relaxed),
        ns_after_first,
        "失配窗口已读元素 namespaceURI 读不得触发 __zw_get_ns 宿主往返（章快道；红态 +2）"
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
