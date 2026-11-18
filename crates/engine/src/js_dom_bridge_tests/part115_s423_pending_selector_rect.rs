// js_dom_bridge 测试切片 115。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// s423：pending 视图 selector rect——`__zw_getBoundingClientRect` selector 身份
///（基座既有元素）在 mutation pending 窗口内的真实几何。
///
/// 复现面：css-anchor-position position-area 族 ~190F——testharness 单脚本 turn 内
/// `el.style.X = v` → 同 turn offsetWidth/offsetLeft 断言读到写前几何（rect 回调
/// selector 路径只读渲染后快照，drain 只在 execute_script 尾）。修复：pending 窗口
/// 内 selector 路径同 s421 handle 臂走有效视图现算布局（CSSOM View：rect 读先
/// update the style and layout steps，
/// https://drafts.csswg.org/cssom-view/#ref-for-getboundingclientrect）。
///
/// 断言面：① pending style 写（width 100→33px）→ gBCR 反映新宽度（高度不受写
/// 影响）；② shim offsetWidth 全链（rect trap → 宿主回调 → pending 视图）同值
/// ——corpus 断言路径；③ drain 后回落快照原值（窗口外旧行为零回归）。
#[test]
fn test_s423_pending_selector_rect_write_read_geometry() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    // 全局游标归零（并行测试共享全局——本测试的 pending 判定依赖 applied=0 起点）。
    crate::js_dom_bridge::publish_mut_applied_cursor(0);
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![DomMutation::SetStyle {
        selector: "#box".to_string(),
        property: "width".to_string(),
        value: "33px".to_string(),
    }]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='box' style='width:100px;height:50px'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    // 快照预填：渲染后旧 rect（写前 100 宽）——drain 后快照路径承载面。
    let snapshot = crate::rect_bridge::new_layout_rect_snapshot();
    {
        let doc = zero_dom::parse_html(&dom_html.lock().unwrap());
        let id_box = crate::js_dom_bridge::find_by_selector(&doc, "#box").expect("#box");
        snapshot
            .lock()
            .unwrap()
            .insert(crate::hit_test::node_id_to_u64(id_box), (0.0, 0.0, 100.0, 50.0));
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

    // ① pending 写 → gBCR 反映新宽度（33），高度不受写影响（50）。
    assert_eq!(read(&mut sandbox, "#box"), "w33h50", "pending style 写应反映到 gBCR");

    // ② shim offsetWidth 全链同值（corpus 断言路径：el.offsetWidth → rect trap →
    // 本臂）。
    let ow = sandbox.execute("document.getElementById('box').offsetWidth").unwrap().value;
    assert_eq!(ow, "33", "pending 写后 offsetWidth 应为 33");

    // ③ drain 后（队列清空，模拟宿主 apply 完成）→ 窗口闭合，回落快照原值
    //（100——宿主重渲染前快照仍持旧值；快照刷新后由渲染路径承载新值）。
    mutations.lock().unwrap().clear();
    assert_eq!(read(&mut sandbox, "#box"), "w100h50", "drain 后回落快照旧行为");
}

/// s423：pending 写 + position-area 区域定位集成面——g35 桶动机场景。基座元素带
/// position-anchor/anchor-name，pending `position-area` 写后 offsetLeft/offsetTop
/// 反映 §4.1 区域默认对齐落点（bottom right → 右下区域 start 角）。
#[test]
fn test_s423_pending_selector_rect_position_area_write() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    crate::js_dom_bridge::publish_mut_applied_cursor(0);
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![DomMutation::SetStyle {
        selector: "#t".to_string(),
        property: "position-area".to_string(),
        value: "bottom right".to_string(),
    }]));
    // 400×400 容器 + 100,150 处 150×75 锚 + 10×10 目标（corpus basic 页最小形态，
    // 去 border/margin 干扰）。区域 bottom right：inline [250,400) × block [225,400)，
    // §4.1 默认表（仅 end 轨→start）→ target 落区域 start 角 (250,225)。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='c' style='position:relative;width:400px;height:400px'>\
         <div id='a' style='position:absolute;left:100px;top:150px;width:150px;height:75px;anchor-name:--a'></div>\
         <div id='t' style='position:absolute;width:10px;height:10px;position-anchor:--a'></div>\
         </div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    let snapshot = crate::rect_bridge::new_layout_rect_snapshot();
    crate::js_dom_bridge::set_pending_layout_viewport(800, 600);
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, Some(&snapshot));

    let offset = |sandbox: &mut V8Sandbox, prop: &str| -> String {
        sandbox
            .execute(&format!("String(document.getElementById('t').{prop})"))
            .unwrap()
            .value
    };
    assert_eq!(offset(&mut sandbox, "offsetLeft"), "250", "position-area pending 写 → 区域 start 角 x");
    assert_eq!(offset(&mut sandbox, "offsetTop"), "225", "position-area pending 写 → 区域 start 角 y");
    assert_eq!(offset(&mut sandbox, "offsetWidth"), "10", "显式尺寸不被区域覆写");
}
