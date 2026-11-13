// js_dom_bridge 测试切片 111。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// s421：pending 视图 handle rect——`__zw_getBoundingClientRect` 裸 handle 身份
///（`__n{n}`，createElement 产物）在 mutation pending 窗口内的真实几何。
///
/// 复现面：embed-allowed-schemas.sub.window.js 5F——load 事件经 `_defer` 于同 turn
/// 派发（先于宿主 apply+重渲染），embed 在 onload handler 里的 offsetWidth 恒 0
///（快照无该元素 rect）。修复：pending 窗口内经 handle 视图槽（基座重放 pending
/// 区间）现算布局取真几何（CSSOM View：rect 读先 update the style and layout
/// steps，https://drafts.csswg.org/cssom-view/#ref-for-getboundingclientrect）；
/// embed/object 无内容态由 shim 侧 `_zwEmbedObjectNoContent` 门收窄（represents-
/// nothing 面恒零——embed-change-src-2 ① oracle，upstream 红面语义保持）。
///
/// 断言面：① shim 级 no-src embed 同 turn 读 0（门闭——represents nothing）；
/// ② src 落地 load 链 done 后 offsetWidth > 0（门开 + pending 视图臂——corpus
/// has_width 翻转根因面）；③ raw 队列 pending div 全宽（块级 = 视口−body margin）；
/// ④ 视口换档重算（HANDLE_RECT_CACHE 键含视口——auto 几何随视口走）；⑤ inline
/// style 元素精确几何（pending 视图级联消费 inline 声明）；⑥ selector 身份不受
/// 劫持（快照路径原值——s284 雷区收窄面：宿主臂只对裸 handle 生效）；⑦ drain 后
/// 回落空串（跨批 handle 刻意不解析，= 旧行为）。
#[test]
fn test_s421_pending_handle_rect_embed_geometry() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    // 全局游标归零（并行测试共享全局——本测试的 pending 判定依赖 applied=0 起点）。
    crate::js_dom_bridge::publish_mut_applied_cursor(0);
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    // raw 队列 handle 用 __n9{k} 高位段：shim 级 createElement 臂走宿主
    // HANDLE_COUNTER（从 __n1 起分配），避免跨臂撞号污染重放面。
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![
        DomMutation::CreateElement { handle: "__n9001".to_string(), tag: "div".to_string() },
        DomMutation::AppendChild { parent_selector: "body".to_string(), child_handle: "__n9001".to_string() },
        DomMutation::CreateElement { handle: "__n9002".to_string(), tag: "div".to_string() },
        DomMutation::SetStyleOnHandle {
            handle: "__n9002".to_string(),
            property: "width".to_string(),
            value: "50px".to_string(),
        },
        DomMutation::SetStyleOnHandle {
            handle: "__n9002".to_string(),
            property: "height".to_string(),
            value: "20px".to_string(),
        },
        DomMutation::AppendChild { parent_selector: "body".to_string(), child_handle: "__n9002".to_string() },
    ]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='d' style='width:70px;height:30px'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    // selector 控制臂的快照预填：#d rect (0,0,70,30)——NodeId 键 = 同 html fresh-parse
    // 确定性（gBCR path-C 地基不变量）。
    let snapshot = crate::rect_bridge::new_layout_rect_snapshot();
    {
        let doc = zero_dom::parse_html(&dom_html.lock().unwrap());
        let id_d = crate::js_dom_bridge::find_by_selector(&doc, "#d").expect("#d");
        snapshot
            .lock()
            .unwrap()
            .insert(crate::hit_test::node_id_to_u64(id_d), (0.0, 0.0, 70.0, 30.0));
    }
    crate::js_dom_bridge::set_pending_layout_viewport(800, 600);
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, Some(&snapshot));

    // rect 读 helper：宿主回调返 "x,y,w,h" → JS 侧取整为 `w{w}h{h}`（空串 → EMPTY）。
    let read = |sandbox: &mut V8Sandbox, id: &str| -> String {
        sandbox
            .execute(&format!(
                "globalThis.__r = (function() {{ var s = __zw_getBoundingClientRect('{id}'); \
                 if (!s) return 'EMPTY'; var p = s.split(','); \
                 return 'w' + Math.round(+p[2]) + 'h' + Math.round(+p[3]); }})()"
            ))
            .unwrap()
            .value
    };

    // ① shim 级 no-src embed：createElement + append 同 turn 读 offsetWidth——
    // represents nothing 门闭（entry 缺席）→ 0（embed-change-src-2 ① oracle 面）。
    // src IDL setter 触发加载链（_defer microtask，execute 边界 checkpoint 派发）。
    sandbox
        .execute(
            "globalThis.__emb = document.createElement('embed');\
             document.body.appendChild(globalThis.__emb);\
             globalThis.__embBefore = globalThis.__emb.offsetWidth;\
             globalThis.__emb.src = 'data:,Hello';",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__embBefore)").unwrap().value,
        "0",
        "no-src embed 同 turn offsetWidth 恒 0（represents nothing 门）"
    );

    // ② load 链 done（data: 解码成功）→ 门开 + pending 视图臂 → 全宽 > 0——
    // embed-allowed-schemas has_width 断言面（offsetWidth > 0）翻转根因。
    sandbox
        .execute("globalThis.__embAfter = globalThis.__emb.offsetWidth;")
        .unwrap();
    let after = sandbox.execute("String(globalThis.__embAfter)").unwrap().value;
    let after_n: i64 = after.parse().expect("offsetWidth 可解析");
    assert!(after_n > 0, "done 态 embed offsetWidth 应 > 0（pending 视图臂），实际 {after_n}");

    // ③ raw 队列 pending div 臂：块级全宽（视口 800 − body margin 8×2 = 784，±1 容差）。
    let div_rect = read(&mut sandbox, "__n9001");
    assert!(div_rect.starts_with('w'), "pending div 应返真几何，实际 {div_rect:?}");
    let div_w: i64 = div_rect[1..].split('h').next().and_then(|v| v.parse().ok()).expect("宽度可解析");
    assert!((760..=810).contains(&div_w), "div 全宽应约 784（800−margin），实际 {div_w}");

    // ⑤ inline style 元素精确几何：pending 视图级联消费 inline 声明。
    assert_eq!(read(&mut sandbox, "__n9002"), "w50h20", "styled div 精确几何");

    // ④ 视口换档重算：HANDLE_RECT_CACHE 键含视口——同代际视口变化须重算（400−16）。
    crate::js_dom_bridge::set_pending_layout_viewport(400, 300);
    let narrow = read(&mut sandbox, "__n9001");
    let narrow_w: i64 = narrow[1..].split('h').next().and_then(|v| v.parse().ok()).expect("宽度可解析");
    assert!((360..=410).contains(&narrow_w), "视口 400 后 div 全宽应约 384，实际 {narrow_w}");

    // ⑥ selector 身份不受劫持：仍走快照路径原值（pending 状态下读——证明 handle 分支
    // 只对裸 handle 生效，selector 面 s284 雷区不点亮）。
    assert_eq!(read(&mut sandbox, "#d"), "w70h30", "selector 身份走快照原值");

    // ⑦ drain 后回落空串：队列清空（模拟 renderer apply 完成）→ 跨批 handle 不解析，
    // 回落空串 = 旧行为（s284 clone-img attr 期望面不点亮）。
    mutations.lock().unwrap().clear();
    assert_eq!(read(&mut sandbox, "__n9001"), "EMPTY", "drain 后 handle 回落空串");
}
