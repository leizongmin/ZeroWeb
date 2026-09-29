// js_dom_bridge 测试切片 30。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

#[test]
fn test_form_submit_default_navigation_r_baidu5() {
    // R-baidu5：form 默认提交导航（spec HTML §4.10.22 提交算法的导航步骤 + §4.10.22.2
    // entry list 构造子集，https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#form-submission-algorithm）。
    // 旧实现 submit 派发后停止（headless 无导航 documented no-op）→ baidu 搜索框输入 +
    // #su 点击 submit 触发但 URL 不变（站内搜索主任务阻断根因）。断言面：
    // ① requestSubmit() → submit 事件触发 + GET urlencoded entry list 替换 action query
    //    导航投递（tree order、checkbox 勾选 value||'on'、select selected option、无名
    //    按钮不贡献 entry）；② preventDefault 取消默认动作 → 事件触发但零投递；
    // ③ requestSubmit(submitter) → submitter formaction 覆盖 action；④ form.submit() →
    //    不发 submit 事件、直接导航；⑤ action 既有 query **整体替换**（review M1：spec
    //    "Set parsed action's query component to query"，非追加）；⑥ method 属性缺失
    //    默认 GET（live baidu form 形态）；⑦ method=post → 零投递；⑧ disabled 控件
    //    跳过；⑨ 有名 submitter 按钮贡献自身 entry（_zwIsSubmitterControl 主路径）；
    // ⑩ submitter 无 value 属性 → 空串 entry（review M2：禁 "null" 字面量）；
    // ⑪ select 无显式 selected → 默认首项回落（review M3，与 Rust 孪生
    //    collect_form_data 对齐）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <form id='f' action='/search' method='get'>\
         <input id='kw' name='q'>\
         <input id='cb' type='checkbox' name='c' checked>\
         <select id='s' name='s'><option value='1'>one</option><option value='2' selected>two</option></select>\
         <button id='su' type='submit' formaction='/s2'>go</button>\
         </form>\
         <form id='g' action='/s?src=1' method='get'><input id='gx' name='x' value='1'></form>\
         <form id='h' action='/n'><input id='hx' name='y' value='9'></form>\
         <form id='p' action='/post' method='post'><input id='px' name='px' value='p'></form>\
         <form id='d' action='/d' method='get'>\
         <input id='dd' name='dv' value='keep' disabled><input id='dl' name='lv' value='live'></form>\
         <form id='b' action='/b' method='get'><input id='bsub' type='submit' name='btn' value='GO'></form>\
         <form id='v' action='/v' method='get'><input id='vsub' type='submit' name='vn'></form>\
         <form id='u' action='/u' method='get'>\
         <select id='us' name='u1'><option value='a'>A</option><option value='b'>B</option></select></form>\
         <form id='w' action='/w' method='get'><button id='wsub' type='submit' name='wb'>go</button></form>\
         <form id='i' action='/i' method='get'><input id='isub' type='image' name='img' src='/pixel.png'></form>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let nav_bridge = crate::NavigationBridge::new();
    let nav_queue = nav_bridge.queue();
    nav_bridge.register(&mut sandbox);
    let drain = || nav_queue.lock().map(|mut q| std::mem::take(&mut *q)).unwrap_or_default();
    sandbox
        .execute(
            "globalThis.__fired = 0;\
             globalThis.__cancel = false;\
             var __f = document.getElementById('f');\
             __f.addEventListener('submit', function (e) { globalThis.__fired++; if (globalThis.__cancel) e.preventDefault(); });\
             document.getElementById('kw').value = 'zw';",
        )
        .unwrap();

    // ① requestSubmit()：submit 触发 + 默认动作导航（q=zw&c=on&s=2，tree order，
    // checkbox 勾选取 value||'on'，select 取 selected option 的 value）。
    sandbox.execute("__f.requestSubmit();").unwrap();
    assert_eq!(sandbox.execute("globalThis.__fired").unwrap().value, "1", "requestSubmit → submit 事件触发");
    assert_eq!(
        drain(),
        vec!["https://example.com/search?q=zw&c=on&s=2".to_string()],
        "requestSubmit 默认动作 → GET entry list 导航投递"
    );

    // ② preventDefault：事件触发但默认动作取消 → 零投递。
    sandbox.execute("globalThis.__cancel = true; globalThis.__fired = 0; __f.requestSubmit();").unwrap();
    assert_eq!(sandbox.execute("globalThis.__fired").unwrap().value, "1", "preventDefault 下 submit 事件仍触发");
    assert!(drain().is_empty(), "preventDefault 取消默认动作 → 不投递导航");

    // ③ requestSubmit(submitter)：submitter formaction 覆盖 form action；无名按钮不贡献 entry。
    sandbox.execute("globalThis.__cancel = false; globalThis.__fired = 0; __f.requestSubmit(document.getElementById('su'));").unwrap();
    assert_eq!(sandbox.execute("globalThis.__fired").unwrap().value, "1", "requestSubmit(submitter) → submit 触发");
    assert_eq!(
        drain(),
        vec!["https://example.com/s2?q=zw&c=on&s=2".to_string()],
        "submitter formaction 覆盖 action → 导航到 /s2"
    );

    // ④ form.submit()：不发 submit 事件，直接走提交导航（form action，非 submitter 的）。
    sandbox.execute("globalThis.__fired = 0; __f.submit();").unwrap();
    assert_eq!(sandbox.execute("globalThis.__fired").unwrap().value, "0", "form.submit() 不发 submit 事件");
    assert_eq!(
        drain(),
        vec!["https://example.com/search?q=zw&c=on&s=2".to_string()],
        "form.submit() 直接导航（form action）"
    );

    // ⑤ action 既有 query → **整体替换**（review M1：spec GET 分支 query setter 语义，
    // 真实浏览器 action 既有 query 丢失；input 未用户编辑 → value 回退属性默认值）。
    sandbox.execute("document.getElementById('g').submit();").unwrap();
    assert_eq!(
        drain(),
        vec!["https://example.com/s?x=1".to_string()],
        "GET 提交替换 action 既有 query"
    );

    // ⑥ method 属性缺失 → 默认 GET（spec §4.10.22 步骤 4-7 归一；baidu 首页 form
    // 即无 method 属性——live 验证暴露的精确缺口形态）。
    sandbox.execute("document.getElementById('h').submit();").unwrap();
    assert_eq!(
        drain(),
        vec!["https://example.com/n?y=9".to_string()],
        "method 缺失默认 GET → 导航投递"
    );

    // ⑦ method=post → 宿主导航契约无 method/body 面，本切片排除 → 零投递。
    sandbox.execute("document.getElementById('p').submit();").unwrap();
    assert!(drain().is_empty(), "method=post 排除 → 零投递");

    // ⑧ disabled 控件跳过：dd 有 disabled 属性不贡献，dl 正常贡献。
    sandbox.execute("document.getElementById('d').submit();").unwrap();
    assert_eq!(
        drain(),
        vec!["https://example.com/d?lv=live".to_string()],
        "disabled 控件不贡献 entry"
    );

    // ⑨ 有名 submitter 按钮贡献自身 entry（requestSubmit(submitter) 主路径，
    // _zwIsSubmitterControl 匹配）。
    sandbox.execute("__fired = 0; document.getElementById('b').requestSubmit(document.getElementById('bsub'));").unwrap();
    assert_eq!(
        drain(),
        vec!["https://example.com/b?btn=GO".to_string()],
        "有名 submitter 贡献自身 entry"
    );

    // ⑩ submitter 无 value 属性 → 空串 entry（review M2：不得序列化出 "null" 字面量）。
    sandbox.execute("document.getElementById('v').requestSubmit(document.getElementById('vsub'));").unwrap();
    assert_eq!(
        drain(),
        vec!["https://example.com/v?vn=".to_string()],
        "submitter 无 value → 空串 entry（非 null 字面量）"
    );

    // ⑪ select 无显式 selected → 默认首项回落（review M3：spec select 默认选中语义，
    // 与 Rust 孪生 collect_form_data 对齐）。
    sandbox.execute("document.getElementById('u').submit();").unwrap();
    assert_eq!(
        drain(),
        vec!["https://example.com/u?u1=a".to_string()],
        "select 无显式 selected → 首个非 disabled option 回落"
    );

    // ⑫ BUTTON submitter 无 value 属性 → 空串 entry（二轮 review minor 2：spec/
    // Chrome/Rust 孪生同口径，非跳过）。
    sandbox.execute("document.getElementById('w').requestSubmit(document.getElementById('wsub'));").unwrap();
    assert_eq!(
        drain(),
        vec!["https://example.com/w?wb=".to_string()],
        "BUTTON submitter 无 value → 空串 entry"
    );

    // ⑬ image submitter → name.x/name.y 坐标对（二轮 review minor 3：spec §4.10.22.2
    // 形状，非指针激活坐标 0,0；不产 name= 基础伪 entry）。
    sandbox.execute("document.getElementById('i').requestSubmit(document.getElementById('isub'));").unwrap();
    assert_eq!(
        drain(),
        vec!["https://example.com/i?img.x=0&img.y=0".to_string()],
        "image submitter → name.x/name.y 坐标对 entry"
    );
}
