// js_dom_bridge 测试切片 146。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// G1b：handle 通道 gCS width/height used-value echo——`__zw_get_computed_style_by_handle`
/// 对 createElement 产物（裸 handle `__n{n}` 身份）的 width/height 读从静态级联串化
///（"auto"/原文）回填 used px（CSSOM-1 §resolved value，selector 通道 S1 刀
/// CALC-SIZE-RESOLVED-S1 的 handle 臂镜像）。
///
/// 断言面：① `height:auto` + inline 内容子 → used px（非 "auto"，echo 生效）；
/// ② 显式 px 声明 → echo 与静态串化同值（零漂移面）；③ `display:none` 臂回落
/// 静态串化（门集不接）；④ 非 width/height 属性不接 echo（"auto" 原样）；
/// ⑤ drain 后跨批 handle 回落静态串化（旧行为零漂移——s284 雷区收窄同形）。
#[test]
fn test_g1b_handle_used_echo() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    crate::js_dom_bridge::publish_mut_applied_cursor(0);
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    // raw 队列 handle 用 __n9{k} 高位段（shim 臂 HANDLE_COUNTER 从 __n1 起，
    // 跨臂撞号污染重放面——part111 同配方）。
    // __n9001：height auto + 内容子（::before 不可用 inline 面，改用子 div 撑高
    // 60px）；__n9002：显式 px 双轴（零漂移臂）；__n9003：display:none（门臂）。
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![
        DomMutation::CreateElement { handle: "__n9001".to_string(), tag: "div".to_string() },
        DomMutation::SetStyleOnHandle {
            handle: "__n9001".to_string(),
            property: "height".to_string(),
            value: "auto".to_string(),
        },
        DomMutation::SetInnerHtmlOnHandle {
            handle: "__n9001".to_string(),
            html: "<div style=\"height:60px\"></div>".to_string(),
        },
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
        DomMutation::CreateElement { handle: "__n9003".to_string(), tag: "div".to_string() },
        DomMutation::SetStyleOnHandle {
            handle: "__n9003".to_string(),
            property: "display".to_string(),
            value: "none".to_string(),
        },
        DomMutation::SetStyleOnHandle {
            handle: "__n9003".to_string(),
            property: "width".to_string(),
            value: "30px".to_string(),
        },
        DomMutation::AppendChild { parent_selector: "body".to_string(), child_handle: "__n9003".to_string() },
    ]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::set_pending_layout_viewport(800, 600);
    register_dom_callbacks(
        &mut sandbox,
        &mutations,
        &dom_html,
        &page_url,
        &canvas_registry,
        None,
    );

    // gCS by handle 读 helper。
    let gcs = |sandbox: &mut V8Sandbox, id: &str, prop: &str| -> String {
        sandbox
            .execute(&format!(
                "__zw_get_computed_style_by_handle('{id}', '{prop}')"
            ))
            .unwrap()
            .value
    };

    // ① echo 生效：height auto + 内容子 → used px 60px（此前静态串化返 "auto"）。
    // 子 div 60px 撑开父 auto 高（块流 content 高度）。
    assert_eq!(
        gcs(&mut sandbox, "__n9001", "height"),
        "60px",
        "auto 高度 handle 元素 gCS 应回填 used px"
    );
    // ② 零漂移：显式 px 声明 echo 与静态串化同值。
    assert_eq!(gcs(&mut sandbox, "__n9002", "width"), "50px", "显式 px width 零漂移");
    assert_eq!(gcs(&mut sandbox, "__n9002", "height"), "20px", "显式 px height 零漂移");
    // ③ display:none 门：不接 echo——display:none 无盒（rect 表无）→ None 回落，
    // width resolved = computed 声明值（30px 与静态串化同值，锚门不越界）。
    assert_eq!(
        gcs(&mut sandbox, "__n9003", "width"),
        "30px",
        "display:none 的 width resolved = computed 声明值（门臂）"
    );

    // ⑤ drain 后跨批 handle：echo 不再生效（pending_view_handle_rect 跨批 handle
    // 刻意不解析 → None；s166 正置表未发布 → 跨批路径亦 miss）——断言不再返
    // used px（回落实值 = 空串，绑 forward 缺席环境，不锚死）。
    mutations.lock().unwrap().clear();
    assert_ne!(
        gcs(&mut sandbox, "__n9001", "height"),
        "60px",
        "drain 后跨批 handle echo 必须失效"
    );
}
