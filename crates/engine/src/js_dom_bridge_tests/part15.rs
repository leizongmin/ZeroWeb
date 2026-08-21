// js_dom_bridge 测试切片 15（R3184+）。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与 part01-14 共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation /
// apply_mutations_to_html 等）。R3200 从 part14 拆出（part14 达 ~1975 行近 2000 上限）：本切片承载
// DOM/CSSOM JS 表面 spec 审生产路径测试（R3184-R3199——LegacyNullToEmptyString、enumerated 反射、
// getAttribute null 语义、toggleAttribute 返值、style priority/latest-wins、handle dataset/属性名/NamedNodeMap/
// cloneNode/inline-style 对等）。

// ── R3184：textContent / innerHTML / outerHTML setter 生产路径 spec `LegacyNullToEmptyString` ──
//
// 生产 always-on B-gen shim 路径（js_dom_shim/part04.js set trap）。spec：三 setter 均把 null 视作空串
//（textContent/innerHTML 清子、outerHTML 移除自身），非通用 JS ToString 的 "null"。验证 JS 侧
// `value === null ? '' : String(value)` 强制 → 入队 mutation 的 text/html 字段为 "" 而非 "null"。

#[test]
fn test_text_content_null_clears_production_r3184() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='t'>hi</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute("document.querySelector('#t').textContent = null;")
        .unwrap();
    let texts: Vec<String> = mutations
        .lock()
        .unwrap()
        .iter()
        .filter_map(|m| match m {
            DomMutation::SetText { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    // null → 空串（spec）→ SetText{text:""}，非 "null"。
    assert_eq!(
        texts,
        vec!["".to_string()],
        "textContent=null 应入队 SetText{{text:\"\"}}（spec 空串），非 \"null\""
    );
}

#[test]
fn test_text_content_undefined_is_string_production_r3184() {
    // js-dom M4 R81 spec 纠正：undefined 与 null 同归空串（WPT Node-textContent "set to
    // undefined" 期望 ""——WebIDL nullable DOMString 缺省语义；R3184 旧记录为错误语义）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='t'>hi</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute("document.querySelector('#t').textContent = undefined;")
        .unwrap();
    let texts: Vec<String> = mutations
        .lock()
        .unwrap()
        .iter()
        .filter_map(|m| match m {
            DomMutation::SetText { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        texts,
        vec!["".to_string()],
        "R81：textContent=undefined 与 null 同归空串（WPT Node-textContent spec 语义）"
    );
}

#[test]
fn test_inner_html_null_clears_production_r3184() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='t'><b>x</b></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute("document.querySelector('#t').innerHTML = null;")
        .unwrap();
    let htmls: Vec<String> = mutations
        .lock()
        .unwrap()
        .iter()
        .filter_map(|m| match m {
            DomMutation::SetInnerHtml { html, .. } => Some(html.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        htmls,
        vec!["".to_string()],
        "innerHTML=null 应入队 SetInnerHtml{{html:\"\"}}（清子），非 \"null\""
    );
}

#[test]
fn test_outer_html_null_removes_production_r3184() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='t'><b>x</b></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute("document.querySelector('#t').outerHTML = null;")
        .unwrap();
    let htmls: Vec<String> = mutations
        .lock()
        .unwrap()
        .iter()
        .filter_map(|m| match m {
            DomMutation::SetOuterHtml { html, .. } => Some(html.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        htmls,
        vec!["".to_string()],
        "outerHTML=null 应入队 SetOuterHtml{{html:\"\"}}（移除自身），非 \"null\""
    );
}

// ── R3185：反射字符串属性 setter 生产路径 spec `[LegacyNullToEmptyString]` ──
//
// 生产 always-on B-gen shim 路径（js_dom_shim/part04.js set trap）。id/title/lang/accessKey 为
// spec `[LegacyNullToEmptyString]`（null→空串）；className/dir 非（null→"null"）。验证 JS 侧
// `value === null ? '' : String(value)`（dir/className 仍 String）→ 入队 SetAttr 的 value 字段。

#[test]
fn test_reflected_string_attrs_null_empty_production_r3185() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='t'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var e=document.querySelector('#t');\
             e.id=null; e.title=null; e.lang=null; e.accessKey=null;\
             e.className=null; e.dir=null;",
        )
        .unwrap();
    // 收集 (content-attr-name, value) 对（SetAttr sel-based）。
    let pairs: Vec<(String, String)> = mutations
        .lock()
        .unwrap()
        .iter()
        .filter_map(|m| match m {
            DomMutation::SetAttr { name, value, .. } => Some((name.clone(), value.clone())),
            _ => None,
        })
        .collect();
    // id/title/lang/accessKey（→accesskey）null→""；className(→class)/dir null→"null"。
    assert_eq!(
        pairs,
        vec![
            ("id".to_string(), "".to_string()),
            ("title".to_string(), "".to_string()),
            ("lang".to_string(), "".to_string()),
            ("accesskey".to_string(), "".to_string()),
            ("class".to_string(), "null".to_string()),
            ("dir".to_string(), "null".to_string()),
        ],
        "id/title/lang/accessKey null→空串（LegacyNull）；class/dir null→\"null\"（非 LegacyNull）"
    );
}

// ── R3186：`dir` enumerated getter 生产路径（spec https://html.spec.whatwg.org/multipage/dom.html#the-dir-attribute）──
//
// dir 为 enumerated attribute（关键字 ltr/rtl/auto）。setter 缓存原值（case 保留，仍 String 化）；getter 须
// 规范化——case-insensitive 命中→小写，invalid（含 "foo"/"null"）/missing→空串。验证 setter→getter 缓存往返：
// 旧实现 getter 直读缓存返原值（"RTL"/"foo"/"null"），现 spec 合规。

#[test]
fn test_dir_enumerated_getter_production_r3186() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='t'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // setter→getter 缓存往返：'RTL'→'rtl'（case 规范化）；'foo'→''（invalid）；null→''（'null' invalid）；
    // 'auto'→'auto'（合法）。旧实现返 "RTL|[foo]|[null]|auto"。
    let out = sandbox
        .execute(
            "var e=document.querySelector('#t');\
             e.dir='RTL'; var a=e.dir;\
             e.dir='foo'; var b='['+e.dir+']';\
             e.dir=null; var c='['+e.dir+']';\
             e.dir='auto'; var d=e.dir;\
             a+'|'+b+'|'+c+'|'+d",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "rtl|[]|[]|auto",
        "dir enumerated getter：合法关键字→规范小写，invalid/missing→空串"
    );
}

// ── R3187：`contentEditable`/`isContentEditable` 枚举反射生产路径（spec `dom-contenteditable`）──
//
// contenteditable 为枚举属性（关键字：空串、true、false）。spec：空串与 "true" 同映射 true 状态。
// 生产 shim getter 旧实现直读缓存/host 原值（返 "foo"/"TRUE"/""），现规范化——空串/case-insensitive
// "true"→"true"、"false"→"false"、余（incl invalid/inherit/missing）→"inherit"。isContentEditable 旧仅
// `=== 'true'`，现空串/"true"（case-insensitive）→ true。

#[test]
fn test_content_editable_enumerated_production_r3187() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='t'></div><div id='pe' contenteditable></div><div id='pt' contenteditable='TRUE'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 解析期属性路径（host has_attr + get_attr，非 setter 缓存）：`<div contenteditable>`（present-空串 keyword）
    // → "true"；`<div contenteditable='TRUE'>`（case-insensitive）→ "true"；`#t`（无属性，missing）→ "inherit"。
    // 旧实现把 present-空串（host 返 ""）与 missing（host 返 ""）混同均返 "inherit"（R3187 has_attr 修正）。
    let parsed = sandbox
        .execute(
            "document.querySelector('#pe').contentEditable+'/'+\
             document.querySelector('#pt').contentEditable+'/'+\
             document.querySelector('#t').contentEditable",
        )
        .unwrap()
        .value;
    assert_eq!(
        parsed, "true/true/inherit",
        "解析期：present-空串 keyword→'true'，case-insensitive→'true'，missing→'inherit'"
    );

    // setter→getter 缓存往返：''→'true'（空串 keyword = true 状态）；'TRUE'→'true'（case 规范化）；
    // 'foo'→'inherit'（invalid）；'false'→'false'；'inherit'→'inherit'。旧实现返 "true|TRUE|foo|false|inherit"。
    let ce = sandbox
        .execute(
            "var e=document.querySelector('#t');\
             e.contentEditable=''; var a=e.contentEditable;\
             e.contentEditable='TRUE'; var b=e.contentEditable;\
             e.contentEditable='foo'; var c=e.contentEditable;\
             e.contentEditable='false'; var d=e.contentEditable;\
             e.contentEditable='inherit'; var g=e.contentEditable;\
             a+'|'+b+'|'+c+'|'+d+'|'+g",
        )
        .unwrap()
        .value;
    assert_eq!(
        ce, "true|true|inherit|false|inherit",
        "contentEditable 枚举 getter：空串/true→'true'，false→'false'，invalid/inherit→'inherit'"
    );

    // isContentEditable：空串 keyword → true（旧实现仅 'true'→true，空串→false）；invalid → false。
    let ice = sandbox
        .execute(
            "var e=document.querySelector('#t');\
             e.contentEditable=''; var a=e.isContentEditable;\
             e.contentEditable='foo'; var b=e.isContentEditable;\
             a+'/'+b",
        )
        .unwrap()
        .value;
    assert_eq!(
        ice, "true/false",
        "isContentEditable：空串 keyword = true 状态 → true；invalid → false"
    );
}

// ── R3188：`draggable` enumerated getter 生产路径——case-insensitive + auto-state default-draggable ──
//
// spec HTML `draggable`（枚举属性，关键字 true/false case-insensitive，缺省/非法→auto 状态）。IDL getter：
// true 状态→true；auto 状态→default-draggable（img/audio/video/a[href]→true，余→false）。旧生产 getter 仅
// `=== 'true'`（case-sensitive，且 auto 统一 false）。

#[test]
fn test_draggable_enumerated_auto_state_production_r3188() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
           <div id='dtrue' draggable='true'></div>\
           <div id='dupper' draggable='TRUE'></div>\
           <div id='dfalse' draggable='false'></div>\
           <div id='dgarb' draggable='foo'></div>\
           <div id='div'></div>\
           <img id='img'/>\
           <a id='ahref' href='x.html'></a>\
           <a id='anohref'></a>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 解析期属性：true(小写/大写 case-insensitive)→true；false→false；invalid "foo"→auto→div default→false。
    let explicit = sandbox
        .execute(
            "document.querySelector('#dtrue').draggable + '/' +\
             document.querySelector('#dupper').draggable + '/' +\
             document.querySelector('#dfalse').draggable + '/' +\
             document.querySelector('#dgarb').draggable",
        )
        .unwrap()
        .value;
    assert_eq!(
        explicit, "true/true/false/false",
        "draggable：case-insensitive true→true，false→false，invalid→auto(div)→false"
    );

    // auto 状态 default-draggable：div→false / img→true / a[href]→true / a(无 href)→false。
    let auto = sandbox
        .execute(
            "document.querySelector('#div').draggable + '/' +\
             document.querySelector('#img').draggable + '/' +\
             document.querySelector('#ahref').draggable + '/' +\
             document.querySelector('#anohref').draggable",
        )
        .unwrap()
        .value;
    assert_eq!(
        auto, "false/true/true/false",
        "auto 状态 default-draggable：div false，img true，a[href] true，a 无 href false"
    );

    // setter→getter 缓存往返：draggable=true→true（attr "true"）；draggable=false→false。
    let setget = sandbox
        .execute(
            "var d=document.querySelector('#div');\
             d.draggable = true; var a=d.draggable;\
             d.draggable = false; var b=d.draggable;\
             a+'/'+b",
        )
        .unwrap()
        .value;
    assert_eq!(setget, "true/false", "draggable setter→getter 缓存往返");
}

// ── R3189：`input.type` / `button.type` enumerated reflection（spec「limited to only known values」）──
//
// input.type / button.type 为枚举属性（非通用 type 字符串反射）。getter 须规范化：INPUT 已知关键字
// （case-insensitive）→ 规范小写，缺省/非法 → "text"；BUTTON submit/reset/button，缺省/非法 → "submit"。
// 非 INPUT/BUTTON（link/script 等）回落通用字符串反射（原值）。旧实现经通用 _reflectedStringAttr('type')
// 返原值（缺省 ""，"EMAIL"→"EMAIL"，"foo"→"foo"）——表单库 switch(input.type) 全失效。

#[test]
fn test_input_button_type_enumerated_production_r3189() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
           <input id='itext'/>\
           <input id='iemail' type='email'/>\
           <input id='iupper' type='NUMBER'/>\
           <input id='igarb' type='foo'/>\
           <button id='bdef'></button>\
           <button id='breset' type='reset'></button>\
           <button id='bgarb' type='foo'></button>\
           <link id='lk' type='text/css'/>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // input.type：缺省→"text"；已知关键字原样 "email"；case-insensitive "NUMBER"→"number"；非法 "foo"→"text"。
    let input_types = sandbox
        .execute(
            "document.querySelector('#itext').type + '/' +\
             document.querySelector('#iemail').type + '/' +\
             document.querySelector('#iupper').type + '/' +\
             document.querySelector('#igarb').type",
        )
        .unwrap()
        .value;
    assert_eq!(
        input_types, "text/email/number/text",
        "input.type：缺省→text，已知→规范小写，case-insensitive→小写，非法→text"
    );

    // button.type：缺省→"submit"；"reset"→"reset"；非法 "foo"→"submit"。
    let button_types = sandbox
        .execute(
            "document.querySelector('#bdef').type + '/' +\
             document.querySelector('#breset').type + '/' +\
             document.querySelector('#bgarb').type",
        )
        .unwrap()
        .value;
    assert_eq!(
        button_types, "submit/reset/submit",
        "button.type：缺省→submit，reset→reset，非法→submit"
    );

    // 非 INPUT/BUTTON 的 type（link）→ 通用字符串反射（原值 "text/css"，不走枚举）。
    let link_type = sandbox
        .execute("document.querySelector('#lk').type")
        .unwrap()
        .value;
    assert_eq!(link_type, "text/css", "link.type 走通用字符串反射（原值），非枚举");

    // setter→getter 往返：input.type='EMAIL' → 内容属性 "EMAIL"（setter 写原值），getter "email"（规范化）。
    let setget = sandbox
        .execute(
            "var i=document.querySelector('#itext'); i.type='EMAIL';\
             i.type+'/'+i.getAttribute('type')",
        )
        .unwrap()
        .value;
    assert_eq!(
        setget, "email/EMAIL",
        "input.type setter 写原值，getter 规范化（case-insensitive）"
    );
}

// ── R3190：`getAttribute` / `getAttributeNS` spec null 语义 ──
//
// spec `dom-element-getattribute`：缺省（属性不存在）须返 **null**，present-empty 返 ""，present 返值。
// 旧 polyfill proxy getAttribute 直返 host `__zw_get_attr*`（缺省/空均 ""）→ 缺省返 "" 而非 null，
// 破坏 `el.getAttribute('x') === null` / `!= null` 检查（jQuery/React 高频）。附带修复 `[attr]` 存在性
// 选择器 over-match（旧 `_matchAttrOf` `av != null` 恒真，缺省元素误匹配 `[attr]`）。

#[test]
fn test_get_attribute_null_semantics_production_r3190() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
           <div id='d' data-x='value' data-empty=''></div>\
           <div id='plain'></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 缺省属性 → null（旧 ""）；present-value → "value"；present-empty → ""。
    let gets = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             var a=String(d.getAttribute('missing'));\
             var b=d.getAttribute('data-x');\
             var c='['+d.getAttribute('data-empty')+']';\
             a+'/'+b+'/'+c",
        )
        .unwrap()
        .value;
    assert_eq!(
        gets, "null/value/[]",
        "getAttribute：缺省→null，present-value→值，present-empty→''"
    );

    // removeAttribute 后 → null（latest-wins，闭合 stale 旧值）；setAttribute 后 present-empty → ""。
    let setremove = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.removeAttribute('data-x'); var a=String(d.getAttribute('data-x'));\
             d.setAttribute('data-new',''); var b='['+d.getAttribute('data-new')+']';\
             a+'/'+b",
        )
        .unwrap()
        .value;
    assert_eq!(
        setremove, "null/[]",
        "removeAttribute 后 getAttribute→null；setAttribute 空串后 present-empty→''"
    );

    // getAttributeNS 缺省 → null（委托 getAttribute，spec 一致）。
    let ns = sandbox
        .execute("String(document.querySelector('#d').getAttributeNS(null, 'href'))")
        .unwrap()
        .value;
    assert_eq!(ns, "null", "getAttributeNS 缺省→null（spec 一致）");

    // 附带修复：`[attr]` 存在性选择器不再 over-match 缺省元素。`#plain` 无 data-x → `[data-x]` 不匹配；
    // `#d` 有 data-x → 匹配。旧实现 `_matchAttrOf` `av != null` 恒真（缺省返 ""），两元素均误匹配。
    let sel = sandbox
        .execute(
            "document.querySelectorAll('[data-x]').length + '/' +\
             (document.querySelector('#plain[data-x]') === null)",
        )
        .unwrap()
        .value;
    assert_eq!(
        sel, "1/true",
        "[data-x] 存在性选择器：仅匹配有该属性的元素（#d），缺省元素 #plain 不匹配"
    );
}

// ── R3191：`toggleAttribute` 返回值 latest-wins（spec `dom-element-toggleattribute`：返切换后是否 present）──
//
// 旧 polyfill snapHas 读纯快照 `__zw_has_attr`——同批 setAttribute/removeAttribute 后 toggle 仍读旧快照，
// 返值 stale（setAttribute('x') 后 toggle('x') 应 false 但旧返 true）。改读 `__zw_has_attr_lw`（反映 pending
// SetAttr/RemoveAttr）。apply 时 mutation 一直正确（host apply-time 决策），仅返值修复。

#[test]
fn test_toggle_attribute_return_latest_wins_production_r3191() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
           <div id='d'></div>\
           <div id='has' data-x='1'></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // setAttribute('x') 后 toggle('x')（无 force）——x 现 present → toggle 移除 → 返 false。
    // 旧实现读纯快照（#d 无 x）→ snapHas=false → 返 true（stale，错误）。
    let set_then_toggle = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.setAttribute('data-x','1');\
             String(d.toggleAttribute('data-x'))",
        )
        .unwrap()
        .value;
    assert_eq!(
        set_then_toggle, "false",
        "setAttribute 后 toggle（无 force）：present→移除→返 false（latest-wins；旧 stale 返 true）"
    );

    // removeAttribute('x') 后 toggle('x')（无 force）——x 现 absent → toggle 添加 → 返 true。
    // #has 初始有 data-x，removeAttribute 后 lw 判 absent → snapHas=false → 返 true。
    let remove_then_toggle = sandbox
        .execute(
            "var h=document.querySelector('#has');\
             h.removeAttribute('data-x');\
             String(h.toggleAttribute('data-x'))",
        )
        .unwrap()
        .value;
    assert_eq!(
        remove_then_toggle, "true",
        "removeAttribute 后 toggle（无 force）：absent→添加→返 true（latest-wins）"
    );

    // 常见单次 toggle（无 pending）：#d 当前无 data-x（上面 net 移除）→ toggle 添加 → 返 true。
    // 此场景 lw 与纯快照一致（无 pending），验证无回归。
    let plain_toggle = sandbox
        .execute("String(document.querySelector('#d').toggleAttribute('data-new'))")
        .unwrap()
        .value;
    assert_eq!(plain_toggle, "true", "单次 toggle 无 pending：absent→添加→返 true（无回归）");

    // force=true / force=false 返值不依赖 presence（force 决定）：返 !!force。
    let force_true = sandbox
        .execute("String(document.querySelector('#d').toggleAttribute('data-f', true))")
        .unwrap()
        .value;
    let force_false = sandbox
        .execute("String(document.querySelector('#d').toggleAttribute('data-f', false))")
        .unwrap()
        .value;
    assert_eq!(force_true, "true", "force=true → 返 true（不依赖 presence）");
    assert_eq!(force_false, "false", "force=false → 返 false（不依赖 presence）");
}

// ── R3192：连续 `toggleAttribute` 返值 enqueue-时解析（闭合 R3191 已知限制）──
//
// R3191 闭合 set/remove-then-toggle 返值，但连续 toggle（同批多次 toggle 同一属性）返值仍 stale——
// `__zw_toggle_attribute` 旧 apply-时解析，shim 无法预测 apply 结果。R3192 改 enqueue-时解析：host 计算
// latest-wins presence → 入队具体 SetAttr/RemoveAttr → 返 post-toggle presence。连续 toggle 第二次起返值
// 准确，且后续 getAttribute/hasAttribute 经 sel_attr_override 一致反映。

#[test]
fn test_toggle_attribute_consecutive_return_production_r3192() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='d'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 连续双 toggle（无 force）：#d 无 x。第一次 absent→present 返 true；第二次 present→absent 返 false。
    // R3191 已知限制：第二次返值 stale（返 true，错误）。R3192 enqueue-时解析 → 返 false（正确）。
    let consecutive = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             var a=d.toggleAttribute('data-x');\
             var b=d.toggleAttribute('data-x');\
             String(a)+'/'+String(b)",
        )
        .unwrap()
        .value;
    assert_eq!(
        consecutive, "true/false",
        "连续双 toggle：第一次 absent→present 返 true，第二次 present→absent 返 false（R3192 enqueue-时解析）"
    );

    // 连续三 toggle：absent→present(true)→absent(false)→present(true)。net present。
    let triple = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.removeAttribute('data-x');\
             var a=d.toggleAttribute('data-y');\
             var b=d.toggleAttribute('data-y');\
             var c=d.toggleAttribute('data-y');\
             String(a)+'/'+String(b)+'/'+String(c)",
        )
        .unwrap()
        .value;
    assert_eq!(
        triple, "true/false/true",
        "连续三 toggle：true/false/true（每次返值反映 enqueue-时解析的 post-toggle presence）"
    );

    // 后续 getAttribute 一致反映（enqueue 的 SetAttr/RemoveAttr 经 sel_attr_override）：双 toggle 后 net absent
    // → getAttribute 返 null（R3190 null 语义）。
    let after = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.removeAttribute('data-x'); d.removeAttribute('data-y');\
             d.toggleAttribute('data-z'); d.toggleAttribute('data-z');\
             String(d.getAttribute('data-z'))",
        )
        .unwrap()
        .value;
    assert_eq!(
        after, "null",
        "双 toggle 后 net absent → getAttribute 返 null（lw 一致反映 enqueue 的 SetAttr/RemoveAttr）"
    );
}

// ── R3193：`element.style`（CSSStyleDeclaration）priority/!important CSSOM 合规 ──
//
// spec `dom-cssstyledeclaration`：getPropertyValue 返值**不含** !important；getPropertyPriority 返
// "important"/""；setProperty 第三参 priority 控制 !important。旧 polyfill：getPropertyPriority 恒返 ''（stub）、
// setProperty 忽略 priority 参、getPropertyValue 返值含 !important、readProp split(':') 致 url() 含冒号值截断。
//
// 注：读侧经解析期 style 快照验证（sync set→read latest-wins 为 R3194 独立修复，见已知限制），写侧经
// `apply_mutations_to_html` 验证（apply 后 style 属性含正确 !important）。

#[test]
fn test_style_priority_important_cssom_production_r3193() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
           <div id='d'></div>\
           <div id='imp' style='color: red !important'></div>\
           <div id='url' style='background: url(http://x.png)'></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 读侧（解析期 style 快照）：getPropertyValue='red'（剥离 !important），getPropertyPriority='important'。
    // 旧 getPropertyValue 返 'red !important'（含 priority）、getPropertyPriority 恒 ''。
    let parsed = sandbox
        .execute(
            "var e=document.querySelector('#imp');\
             e.style.getPropertyValue('color')+'/'+e.style.getPropertyPriority('color')",
        )
        .unwrap()
        .value;
    assert_eq!(
        parsed, "red/important",
        "解析期 !important：getPropertyValue='red'（剥离），getPropertyPriority='important'"
    );

    // 解析期无 !important 声明：getPropertyPriority=''。
    let nopri = sandbox
        .execute("document.querySelector('#url').style.getPropertyPriority('background')")
        .unwrap()
        .value;
    assert_eq!(nopri, "", "解析期无 !important → getPropertyPriority=''");

    // 含 ':' 的值（url()）完整读回——旧 split(':') 致 'url(http' 截断，现按首 ':' 切分。
    let url = sandbox
        .execute("document.querySelector('#url').style.getPropertyValue('background')")
        .unwrap()
        .value;
    assert_eq!(
        url, "url(http://x.png)",
        "含 ':' 的值（url()）完整读回（旧 split(':') 截断）"
    );

    // 写侧（apply 后验证）：setProperty 第三参 priority='important' → style 属性含 'color: red !important'；
    // 无 priority → 'font-size: 14px'（无 !important）；ci 'IMPORTANT' → 'margin: 5px !important'。旧 priority 被忽略。
    sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.style.setProperty('color', 'red', 'important');\
             d.style.setProperty('font-size', '14px');\
             d.style.setProperty('margin', '5px', 'IMPORTANT');",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    assert!(out.contains("color: red !important"), "setProperty(p,v,'important') → !important\n{out}");
    assert!(
        out.contains("font-size: 14px") && !out.contains("font-size: 14px !important"),
        "setProperty 无 priority → 无 !important\n{out}"
    );
    assert!(out.contains("margin: 5px !important"), "setProperty priority 'IMPORTANT'（ci）→ !important\n{out}");

    // IDL setter 带 !important（Chrome：解析 value 的 !important）→ apply 后 style 含 'color: blue !important'。
    mutations.lock().unwrap().clear();
    sandbox
        .execute("document.querySelector('#d').style.color = 'blue !important';")
        .unwrap();
    let ms2 = mutations.lock().unwrap().clone();
    let out2 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms2).unwrap();
    assert!(
        out2.contains("color: blue !important"),
        "IDL setter 'blue !important' → apply 后 !important\n{out2}"
    );

    // removeProperty 返前值（读解析期 #imp 的 color='red'，剥离 priority）。
    let removed = sandbox
        .execute("document.querySelector('#imp').style.removeProperty('color')")
        .unwrap()
        .value;
    assert_eq!(removed, "red", "removeProperty 返前值（不含 !important）");
}

// ── R3194：element inline style sync set→read latest-wins（闭合 R3193 已知限制①）──
//
// 旧 `_styleProxy.readRaw` sel 路径读纯快照（`__zw_get_attr`），SetStyle mutation 不经 sel_attr_override
// → 同批 `el.style.x='v'; el.style.x` 返旧值/空（stale）。R3194：sel 路径改走新 `__zw_get_style_lw`
// 回调（replay snapshot style + 同 sel pending SetAttr/RemoveAttr/SetStyle/RemoveStyle），保留 SetStyle
// 变体（pipeline `is_paint_only_mutation` 依赖 property 粒度，不走 enqueue-时解析）。

#[test]
fn test_style_sync_set_read_latest_wins_production_r3194() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='d'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // sync set→read（同批，无 apply）：`el.style.color='red'; el.style.color` → 'red'。
    // 旧 readRaw 读快照（#d 无 style）→ ''（stale）。现 __zw_get_style_lw replay → 'red'。
    let idl = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.style.color='red';\
             d.style.color",
        )
        .unwrap()
        .value;
    assert_eq!(idl, "red", "sync set→read：el.style.color='red' 后读回 'red'（latest-wins）");

    // setProperty→getPropertyValue sync 往返。
    let setp = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.style.setProperty('color','blue');\
             d.style.getPropertyValue('color')",
        )
        .unwrap()
        .value;
    assert_eq!(setp, "blue", "setProperty('color','blue') 后 getPropertyValue='blue'（latest-wins）");

    // 多次 set 累积（replay 顺序合并）：color + font-size → length=2，cssText 含两者。
    let acc = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.style.color='red'; d.style.fontSize='14px';\
             String(d.style.length)+'|'+d.style.cssText",
        )
        .unwrap()
        .value;
    assert!(
        acc.starts_with("2|"),
        "多次 set 累积 length=2，got: {acc}"
    );
    assert!(acc.contains("color: red") && acc.contains("font-size: 14px"), "cssText 含累积声明: {acc}");

    // 同属性覆盖（replay 后者覆盖前者）：color='red' 后 color='green' → 'green'。
    let override_ = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.style.color='red'; d.style.color='green';\
             d.style.color",
        )
        .unwrap()
        .value;
    assert_eq!(override_, "green", "同属性后设覆盖前设（replay merge 去重）");

    // removeProperty sync：设后移除 → 读回 ''。
    let rem = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.style.color='red'; d.style.removeProperty('color');\
             '['+d.style.color+']'",
        )
        .unwrap()
        .value;
    assert_eq!(rem, "[]", "removeProperty 后读回空（latest-wins replay）");

    // cssText 整体设后 per-property 读：cssText='color: red' → style.color='red'。
    let ct = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.style.cssText='color: red';\
             d.style.color",
        )
        .unwrap()
        .value;
    assert_eq!(ct, "red", "cssText 整体设后 per-property 读（SetAttr('style') lw）");

    // 解析期 style 与 sync set 合并：#imp 初始 color:red，sync 设 font-size → 两者俱在。
    // （独立 sandbox 验证，本 sandbox #d 已被污染；用 querySelector 新元素需新 HTML——此处复用 #d 前
    // 已多次设，跳过此组合断言，由上各断言覆盖 replay 各路径。）
}

// ── R3195：handle-based dataset 修复（get/has/delete）──
//
// 旧 `_datasetProxy.hasAttrFn` 对 handle 恒返 false（R3002 时无 `__zw_has_attr_handle` 回调遗留）→
// handle 元素 `el.dataset.foo = 'x'; el.dataset.foo` 恒 undefined（get trap 经 hasAttrFn 短路），且
// `'foo' in el.dataset` 恒 false；deleteProperty 用 set-empty 残留 `data-x=""`。修复：hasAttrFn 用
// `__zw_has_attr_handle`，deleteProperty 优先 `__zw_remove_attr_handle`。

#[test]
fn test_dataset_handle_round_trip_production_r3195() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // handle 元素（createElement，未挂载）dataset round-trip：set→get 返值（旧恒 undefined）。
    let setget = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.dataset.bazQux='world';\
             '['+e.dataset.bazQux+']'+'/'+e.getAttribute('data-baz-qux')",
        )
        .unwrap()
        .value;
    assert_eq!(
        setget, "[world]/world",
        "handle dataset set→get：'world'/属性（旧恒 undefined）"
    );

    // `in` / has：dataset 属性存在性（旧恒 false）。
    let has = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.dataset.key='v';\
             String('key' in e.dataset)+'/'+String('absent' in e.dataset)",
        )
        .unwrap()
        .value;
    assert_eq!(has, "true/false", "handle dataset 'in' 判定（旧恒 false）");

    // delete：移除后 get→undefined（旧 set-empty 致返 ''）。delete 返 true。
    let del = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.dataset.rm='x';\
             var r=delete e.dataset.rm;\
             String(r)+'/'+String(e.dataset.rm===undefined)+'/'+String(e.getAttribute('data-rm')===null)",
        )
        .unwrap()
        .value;
    assert_eq!(
        del, "true/true/true",
        "handle dataset delete：真移除（旧 set-empty 残留），get→undefined，getAttribute→null"
    );
}

// ── R3196：handle dataset 枚举（ownKeys/ownEnumerable）──
//
// R3195 闭合 handle dataset get/has/delete 后，枚举仍返 []——`_datasetProxy.dataKeys()` handle 路径
// 无 `__zw_attr_names_handle` 回调变体，恒返 []（R3195 已知限制①）。新增 host `attribute_names_from_mutations`
//（正序 latest-wins，无快照基底）+ `__zw_attr_names_handle` 回调，dataKeys() handle 路径遍历真实 data-* 属性名。

#[test]
fn test_dataset_handle_enumeration_production_r3196() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // Object.keys(handle.dataset)：camelCase data-* 键（旧恒返 []）。非 data-* 属性（id）不混入。
    let keys = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.id='nope';\
             e.dataset.fooBar='1';\
             e.dataset.baz='2';\
             Object.keys(e.dataset).join(',')",
        )
        .unwrap()
        .value;
    assert_eq!(
        keys, "fooBar,baz",
        "handle dataset Object.keys：camelCase data-* 键，非 data-*（id）排除（旧恒空）"
    );

    // JSON.stringify：序列化含 data 键（旧 '{}'）。
    let json = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.dataset.alpha='x';\
             e.dataset.betaBravo='y';\
             JSON.stringify(e.dataset)",
        )
        .unwrap()
        .value;
    assert_eq!(
        json, "{\"alpha\":\"x\",\"betaBravo\":\"y\"}",
        "handle dataset JSON.stringify：含 data 键（旧恒 {{}}）"
    );

    // delete 反映：枚举移除被删键 + 删后重设追加到末尾（DOM getAttributeNames 序）。
    let after_del = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.dataset.first='a';\
             e.dataset.second='b';\
             delete e.dataset.first;\
             e.dataset.first='c';\
             Object.keys(e.dataset).join(',')",
        )
        .unwrap()
        .value;
    assert_eq!(
        after_del, "second,first",
        "handle dataset 枚举：delete 移除 + 删后重设追加到末尾（正序 latest-wins，DOM 序）"
    );

    // 空句柄 dataset：Object.keys 返 []（无 data-* 属性）。
    let empty = sandbox
        .execute(
            "var e=document.createElement('div');\
             Object.keys(e.dataset).length+'/'+(Object.keys(e.dataset).length===0)",
        )
        .unwrap()
        .value;
    assert_eq!(
        empty, "0/true",
        "handle dataset 空：Object.keys 返空数组"
    );
}

// ── R3197：handle getAttributeNames / hasAttributes 枚举 ──
//
// R3196 新增 `__zw_attr_names_handle` 回调闭合了 handle dataset 枚举，但 `el.getAttributeNames()` /
// `el.hasAttributes()`（part04.js）对 handle 元素仍短路返 []/false（未走新回调）。本切片接线两方法 handle
// 路径走 `__zw_attr_names_handle`，闭合 handle 属性名枚举面（dataset 已 R3196 闭合；getAttributeNames/
// hasAttributes 是更通用的属性名遍历——`el.getAttributeNames()`/`el.hasAttributes()` 在 createElement 未挂载
// 元素上旧恒 []/false）。

#[test]
fn test_handle_attribute_enumeration_production_r3197() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // hasAttributes：有属性 true / 无属性 false（旧恒 false）。
    let has = sandbox
        .execute(
            "var a=document.createElement('div');\
             var b=document.createElement('div');\
             a.setAttribute('id','x');\
             String(a.hasAttributes())+'/'+String(b.hasAttributes())",
        )
        .unwrap()
        .value;
    assert_eq!(has, "true/false", "handle hasAttributes：有/无（旧恒 false）");

    // getAttributeNames：返文档序属性名（含非 data-*）。旧恒 []。
    let names = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.setAttribute('id','main');\
             e.className='btn';\
             e.setAttribute('data-x','1');\
             e.getAttributeNames().join(',')",
        )
        .unwrap()
        .value;
    assert_eq!(
        names, "id,class,data-x",
        "handle getAttributeNames：文档序全部属性名（旧恒空）"
    );

    // removeAttribute 反映 + 删后重设追加到末尾（DOM getAttributeNames 序）。
    let after = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.setAttribute('first','a');\
             e.setAttribute('second','b');\
             e.removeAttribute('first');\
             e.setAttribute('first','c');\
             e.getAttributeNames().join(',')+'/'+e.hasAttributes()",
        )
        .unwrap()
        .value;
    assert_eq!(
        after, "second,first/true",
        "handle getAttributeNames：remove 移除 + 删后重设追加末尾（DOM 序），hasAttributes 仍 true"
    );

    // remove 全部属性后 hasAttributes→false（属性名仅来自 mutations，正序 latest-wins）。
    let all_gone = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.setAttribute('k','v');\
             e.removeAttribute('k');\
             String(e.hasAttributes())+'/'+String(e.getAttributeNames().length===0)",
        )
        .unwrap()
        .value;
    assert_eq!(
        all_gone, "false/true",
        "handle 全删后 hasAttributes→false，getAttributeNames→[]"
    );
}

// ── R3198：handle el.attributes NamedNodeMap + handle 源 cloneNode ──
//
// R3196/R3197 闭合 handle 属性名枚举（dataset / getAttributeNames·hasAttributes），但 `el.attributes`
// NamedNodeMap（part03.js `_attributesProxy.readNames()`）对 handle 元素仍恒空（length 0 / item·getNamedItem 返
// null / iterator 空），且 `cloneNode` 对 handle 源元素 tag 回落 'div' + 不复制属性（旧注释「无 get_tag/
// attr_names handle 变体，best-effort」）。现三 handle 回调（`__zw_get_tag_handle`/`__zw_attr_names_handle`/
// `__zw_get_attr_handle`）均已就绪，接线两端，闭合 handle 属性枚举最后一面。

#[test]
fn test_handle_attributes_and_clone_production_r3198() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // el.attributes NamedNodeMap：length / item(i) / getNamedItem(name) / 迭代（name+value）。旧恒空。
    let attrs = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.setAttribute('id','main');\
             e.setAttribute('data-x','1');\
             var A=e.attributes;\
             A.length+'|'+\
             A.item(0).name+'='+A.item(0).value+'|'+\
             A.getNamedItem('data-x').value+'|'+\
             String(A.getNamedItem('absent')===null)+'|'+\
             Array.from(A).map(function(a){return a.name+':'+a.value;}).join(',')",
        )
        .unwrap()
        .value;
    assert_eq!(
        attrs, "2|id=main|1|true|id:main,data-x:1",
        "handle el.attributes NamedNodeMap：length/item/getNamedItem/迭代（旧恒空）"
    );

    // handle 源 cloneNode：源 tag 保留（旧回落 'DIV'）+ 属性复制（旧不复制）。
    let clone = sandbox
        .execute(
            "var s=document.createElement('section');\
             s.setAttribute('id','s1');\
             s.setAttribute('class','card');\
             var c=s.cloneNode(false);\
             c.tagName+'|'+\
             c.getAttribute('id')+'|'+\
             c.getAttribute('class')+'|'+\
             c.attributes.length+'|'+\
             String(c.getAttributeNames().join(','))",
        )
        .unwrap()
        .value;
    assert_eq!(
        clone, "SECTION|s1|card|2|id,class",
        "handle 源 cloneNode：tag 保留 SECTION（旧 DIV）+ 属性复制（旧空）"
    );

    // handle 源 cloneNode deep：后代 innerHTML 复制（R2994 既有，验证未回归）。
    let clone_deep = sandbox
        .execute(
            "var s=document.createElement('div');\
             s.innerHTML='<span>hi</span>';\
             s.cloneNode(true).innerHTML",
        )
        .unwrap()
        .value;
    assert_eq!(
        clone_deep, "<span>hi</span>",
        "handle 源 cloneNode deep：后代 innerHTML 复制（既有，未回归）"
    );
}

// ── R3199：handle inline style sync set→read latest-wins ──
//
// R3194 闭合 sel-based `el.style.x='v'; el.style.x` sync set→read stale（`__zw_get_style_lw` replay pending style
// mutation），但 handle 路径 `readRaw` 仍纯快照 `__zw_get_attr_handle('style')`——SetStyleOnHandle mutation 不
// 反映到所存 style 属性串，故 handle 元素 `el.style.color='red'; el.style.color` **恒返空**（R3194 已知限制①）。
// 新增 host `style_from_mutations_lw`（正序 replay *OnHandle 变体，无快照基底）+ `__zw_get_style_lw_handle`
// 回调，readRaw handle 路径走 lw，闭合限制。

#[test]
fn test_handle_style_sync_set_read_production_r3199() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // sync set→read round-trip：同批 setProperty→getPropertyValue（旧恒空）。同属性后设覆盖。
    let round_trip = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.style.color='red';\
             var a=e.style.color;\
             e.style.color='blue';\
             var b=e.style.color;\
             e.style.getPropertyValue('color')+'|'+a+'|'+b",
        )
        .unwrap()
        .value;
    assert_eq!(
        round_trip, "blue|red|blue",
        "handle style sync set→read：setProperty→getPropertyValue 往返 + 后设覆盖（旧恒空）"
    );

    // 多属性累积：length=2 + cssText 含两者（camelCase 读 backgroundColor）。
    let accum = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.style.color='red';\
             e.style.backgroundColor='blue';\
             String(e.style.length>=2)+'|'+e.style.backgroundColor+'|'+\
             (e.style.cssText.indexOf('color')>=0 && e.style.cssText.indexOf('background')>=0)",
        )
        .unwrap()
        .value;
    assert_eq!(
        accum, "true|blue|true",
        "handle style 多属性累积：length≥2 + camelCase 读 + cssText 含两者（旧恒空）"
    );

    // removeProperty sync→空。
    let remove = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.style.color='red';\
             var prev=e.style.removeProperty('color');\
             String(prev)+'|'+String(e.style.color==='')+'|'+String(e.style.length===0)",
        )
        .unwrap()
        .value;
    assert_eq!(
        remove, "red|true|true",
        "handle style removeProperty sync：返前值 + 读空 + length=0（旧 stale 仍含）"
    );

    // cssText 整体设后 per-property 读（cssText setter→SetAttrOnHandle{style}，per-prop 读须 parse）。
    let csstext = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.style.cssText='color: green; font-size: 12px';\
             e.style.color+'|'+e.style.getPropertyValue('font-size')+'|'+e.style.length",
        )
        .unwrap()
        .value;
    assert_eq!(
        csstext, "green|12px|2",
        "handle style cssText 设后 per-property 读：parse 正确 + length（旧纯快照 readRaw 读不到）"
    );
}

// ── R3201：handle outerHTML getter 客户端构造 ──
//
// R3198 闭合 handle 属性枚举（NamedNodeMap/cloneNode），但 `outerHTML` getter 对 handle 源元素旧 best-effort
// 返 innerHTML（无 wrapper）——`document.createElement('section').outerHTML` 返 "" 而非 "<section></section>"。
// 现 R3198 三 handle 回调（get_tag/attr_names/get_attr）+ inner_html_handle 就绪，客户端构造完整序列化
//（void 元素无闭合标签 + 属性值转义），闭合 handle-vs-sel 序列化最后一面。isEqualNode 经 outerHTML 比对亦受益。

#[test]
fn test_handle_outerhtml_production_r3201() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 空 section：完整 wrapper（旧返空串）。
    let section = sandbox
        .execute("document.createElement('section').outerHTML")
        .unwrap()
        .value;
    assert_eq!(
        section, "<section></section>",
        "handle section outerHTML：完整 wrapper（旧返 ''）"
    );

    // void 元素 img：无闭合标签（旧返空串）。
    let img = sandbox
        .execute("document.createElement('img').outerHTML")
        .unwrap()
        .value;
    assert_eq!(img, "<img>", "handle void img outerHTML：无闭合标签（旧返 ''）");

    // 属性 + 转义：id + title 含双引号 → `&quot;` 转义。
    let attrs = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.id='x';\
             e.setAttribute('title','a\"b');\
             e.outerHTML",
        )
        .unwrap()
        .value;
    assert_eq!(
        attrs, "<div id=\"x\" title=\"a&quot;b\"></div>",
        "handle outerHTML 属性序列化：文档序 + 双引号转义 &quot;"
    );

    // 含子树：innerHTML 嵌入。
    let with_kids = sandbox
        .execute(
            "var e=document.createElement('div');\
             e.innerHTML='<span>hi</span>';\
             e.outerHTML",
        )
        .unwrap()
        .value;
    assert_eq!(
        with_kids, "<div><span>hi</span></div>",
        "handle outerHTML 含子树：innerHTML 嵌入"
    );

    // & 转义：属性值含 & → &amp;。
    let amp = sandbox
        .execute(
            "var e=document.createElement('a');\
             e.setAttribute('href','a&b');\
             e.outerHTML",
        )
        .unwrap()
        .value;
    assert_eq!(
        amp, "<a href=\"a&amp;b\"></a>",
        "handle outerHTML 属性 & 转义 → &amp;"
    );
}

// ── R3202：form.method / form.enctype latest-wins（闭合 R2839 stale snapshot）──
//
// R2839 已实现 form.method/enctype 枚举规范化（get/post/dialog、三 enctype 值、缺省/非法→default），但 sel 路径
// 读**纯快照** `__zw_get_attr`（非 latest-wins）——同批 `f.method='POST'; f.method` 读 stale 快照返 default 'get'
//（#m_def 无 method 属性 → ''→ 'get'），同 R3190/R3195 stale 模式。修复：sel 路径改走 `__zw_get_attr_lw` 反映同批
// setAttribute/form.method=。表单提交/序列化库读 form.method 决定 GET/POST、读 form.enctype 决定编码高频。

#[test]
fn test_form_method_enctype_enumerated_production_r3202() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
           <form id='m_post' method='POST'></form>\
           <form id='m_foo' method='foo'></form>\
           <form id='m_def'></form>\
           <form id='e_multi' enctype='multipart/form-data'></form>\
           <form id='e_upper' enctype='MULTIPART/FORM-DATA'></form>\
           <form id='e_foo' enctype='foo'></form>\
           <form id='e_plain' enctype='text/plain'></form>\
           <div id='notform' method='POST'></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // method：POST→post（case 规范化）；foo→get（非法→default）；缺省→get。
    let method = sandbox
        .execute(
            "document.querySelector('#m_post').method + '/' +\
             document.querySelector('#m_foo').method + '/' +\
             document.querySelector('#m_def').method",
        )
        .unwrap()
        .value;
    assert_eq!(
        method, "post/get/get",
        "form.method 枚举：POST→post，非法→get，缺省→get（旧返 POST/foo/空串）"
    );

    // enctype：关键字→规范值；大写 case-insensitive；非法→default；缺省→default。
    let enctype = sandbox
        .execute(
            "document.querySelector('#e_multi').enctype + '|' +\
             document.querySelector('#e_upper').enctype + '|' +\
             document.querySelector('#e_foo').enctype + '|' +\
             document.querySelector('#m_def').enctype + '|' +\
             document.querySelector('#e_plain').enctype",
        )
        .unwrap()
        .value;
    assert_eq!(
        enctype,
        "multipart/form-data|multipart/form-data|application/x-www-form-urlencoded|application/x-www-form-urlencoded|text/plain",
        "form.enctype 枚举：关键字→规范值，大写 ci，非法/缺省→urlencoded default（旧返原值/空串）"
    );

    // setter→getter round-trip：setter 写原值（POST），getter 规范化（post）。
    let setget = sandbox
        .execute(
            "var f=document.querySelector('#m_def');\
             f.method='POST';\
             f.method+'/'+f.getAttribute('method')",
        )
        .unwrap()
        .value;
    assert_eq!(
        setget, "post/POST",
        "form.method setter 写原值 POST，getter 规范化 post（round-trip）"
    );

    // 非 FORM（div.method）回落通用字符串反射——返原值 POST，不走枚举规范化。
    let nonform = sandbox
        .execute("document.querySelector('#notform').method")
        .unwrap()
        .value;
    assert_eq!(
        nonform, "POST",
        "非 FORM 元素 method 回落通用字符串反射（原值），不走枚举"
    );
}

// ── R3203：cloneNode sel 源属性值 latest-wins（sel 读源 stale 全表审计发现）──
//
// R3198 cloneNode 属性复制：sel 源名经 `__zw_attr_names`（latest-wins，自 R3002）+ 值经**纯快照** `__zw_get_attr`。
// 名源 lw 但值源纯快照 → 不一致：`el.setAttribute('data-x','newval'); el.cloneNode()` 时名含 data-x（lw）但值读
// stale 快照（pending SetAttr 未反映）→ 克隆属性值 stale（旧值或空）。R3203：sel 源值改走 `__zw_get_attr_lw`
//（与名源同 lw，handle 源本就读 mutations latest-wins 无此问题）。sel 读源 stale 全表审计：余（generic reflected
// string R3037 / dataset R3002 / NamedNodeMap R3003 / defaultValue R2840 / dispatch_element_event async）均已 lw 或
// 非 getter 时序，cloneNode 值复制为唯一真 stale latent。

#[test]
fn test_clone_node_sel_attr_latest_wins_production_r3203() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='src' data-x='old'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // sync setAttribute→cloneNode：设新值后立即克隆，克隆应含新值（旧纯快照读 stale 复制旧值）。
    let cloned = sandbox
        .execute(
            "var e=document.querySelector('#src');\
             e.setAttribute('data-x','newval');\
             var c=e.cloneNode(false);\
             c.getAttribute('data-x')",
        )
        .unwrap()
        .value;
    assert_eq!(
        cloned, "newval",
        "cloneNode sel 源 sync setAttribute 后克隆属性值：newval（旧 stale 复制 'old'）"
    );

    // 新增属性（snapshot 无）后克隆——名 lw 含新属性 + 值 lw 反映（旧名含但值 stale 空）。
    let added = sandbox
        .execute(
            "var e=document.querySelector('#src');\
             e.setAttribute('data-added','v2');\
             e.cloneNode(false).getAttribute('data-added')",
        )
        .unwrap()
        .value;
    assert_eq!(
        added, "v2",
        "cloneNode sel 源新增属性后克隆：含新属性 + 值（旧名含但值 stale 空）"
    );

    // removeAttribute 后克隆——名 lw 不含被删属性 → 克隆无该属性（验证名源 lw 仍工作，未回归）。
    let removed = sandbox
        .execute(
            "var e=document.querySelector('#src');\
             e.removeAttribute('data-x');\
             String(e.cloneNode(false).getAttribute('data-x')===null)",
        )
        .unwrap()
        .value;
    assert_eq!(
        removed, "true",
        "cloneNode sel 源 removeAttribute 后克隆无该属性（名源 lw，未回归）"
    );
}

// ── R3204：全局 reflected 属性 sel 读源 latest-wins（`__zw_has_attr(sel)` stale 审计）──
//
// `__zw_has_attr(sel, ...)` 纯快照存在性审计发现 part04.js 全局 reflected 属性块（autofocus/inert presence +
// autocomplete/draggable/spellcheck/translate/width/height 值）sel 读源纯快照——`_reflectedAttrs` 缓存仅覆盖 IDL
// setter→getter，`setAttribute`/`.attr()`→IDL read 旧读 stale 快照：`setAttribute('autocomplete','off'); el.autocomplete`
// 返 'on'（stale default）、`setAttribute('width','100'); img.width` 返 0（stale）。R3204：sel 读源改 latest-wins。
// 审计收敛：余 has_attr(sel) 位（dataset/NamedNodeMap/defaultChecked/defaultSelected/checked/selected/toggleSnap/
// popover/_mo_read_attr/hasAttribute）均已 lw+fallback，`_ce_attrValue`（CE 旧值，niche）为唯一余项。

#[test]
fn test_global_reflected_attrs_sel_lw_production_r3204() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><input id='i'/><img id='im'/></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // autocomplete：setAttribute('off')→IDL read 'off'（旧 stale 返 'on' default）。
    let autocomplete = sandbox
        .execute(
            "var i=document.querySelector('#i');\
             i.setAttribute('autocomplete','off');\
             i.autocomplete",
        )
        .unwrap()
        .value;
    assert_eq!(
        autocomplete, "off",
        "setAttribute('autocomplete','off')→IDL read 'off'（旧 stale 'on' default）"
    );

    // draggable：setAttribute('true')→IDL read true（旧 stale，缺省 div→false）。
    let draggable = sandbox
        .execute(
            "var i=document.querySelector('#i');\
             i.setAttribute('draggable','true');\
             String(i.draggable)",
        )
        .unwrap()
        .value;
    assert_eq!(
        draggable, "true",
        "setAttribute('draggable','true')→IDL read true（旧 stale false）"
    );

    // width（IMG 反射 unsigned long）：setAttribute('100')→IDL read 100（旧 stale 0）。
    let width = sandbox
        .execute(
            "var im=document.querySelector('#im');\
             im.setAttribute('width','100');\
             String(im.width)",
        )
        .unwrap()
        .value;
    assert_eq!(
        width, "100",
        "img.setAttribute('width','100')→IDL read 100（旧 stale 0）"
    );

    // autofocus（boolean presence）：setAttribute('')→IDL read true（旧 stale false）。
    let autofocus = sandbox
        .execute(
            "var i=document.querySelector('#i');\
             i.setAttribute('autofocus','');\
             String(i.autofocus)",
        )
        .unwrap()
        .value;
    assert_eq!(
        autofocus, "true",
        "setAttribute('autofocus','')→IDL read true（旧 stale false）"
    );

    // 缺省 autocomplete → 'on'（missing-default，spec；验证 lw 不破坏 default）。
    let default_input = sandbox
        .execute(
            "var i2=document.createElement('input');\
             i2.autocomplete",
        )
        .unwrap()
        .value;
    assert_eq!(default_input, "on", "input 缺省 autocomplete → 'on'（missing-default，lw 未破坏）");
}

// ── R3205：CE attributeChangedCallback old-value sel latest-wins（`__zw_has_attr(sel)` 审计最后一项）──
//
// `_ce_attrValue`（part03.js:258）sel 路径旧纯快照——parsed-in-HTML CE 元素（有 selector）同批连续 setAttribute 时，
// 第 N 次 attributeChangedCallback 的 old 须读第 N-1 次设值（spec old = 变更前当前值），旧纯快照读 stale（pending
// SetAttr 未反映）致 old=null。R2992 既有测试用 createElement（handle 路径，本就 latest-wins）未覆盖 sel 路径。
// R3205：sel 路径改 lw，闭合 `__zw_has_attr(sel)` stale 审计。handle 路径不动（handle CE R2992 测试不变）。

#[test]
fn test_parsed_ce_attr_changed_old_value_sel_lw_production_r3205() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // parsed-CE 元素（HTML 解析，有 selector `#pe`）——区别于 R2992 的 createElement（handle）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><parsed-ce id='pe'></parsed-ce></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 定义 CE（observedAttributes=['foo']），对 parsed 元素连续 setAttribute——第 2 次 old 须为第 1 次设值。
    sandbox
        .execute(
            "globalThis.__calls = [];\
             class ParsedCE extends HTMLElement {\
               static get observedAttributes() { return ['foo']; }\
               attributeChangedCallback(name, oldVal, newVal) {\
                 globalThis.__calls.push(name + ':' + oldVal + '->' + newVal);\
               }\
             }\
             customElements.define('parsed-ce', ParsedCE);\
             var el = document.querySelector('#pe');\
             el.setAttribute('foo', 'a');\
             el.setAttribute('foo', 'b');",
        )
        .unwrap();
    // 第 1 次：foo:null->a（首次 set old=null）。第 2 次：foo:a->b（old 须为第 1 次设值 'a'，旧 sel 纯快照 stale → null）。
    assert_eq!(
        sandbox.execute("globalThis.__calls.join('|')").unwrap().value,
        "foo:null->a|foo:a->b",
        "parsed-CE 同批连续 setAttribute：第 2 次 attributeChangedCallback old=第 1 次设值 'a'（旧 sel 纯快照 stale → null）"
    );
}

#[test]
fn test_html_table_insert_row_r3243() {
    // R3243：HTMLTableElement.insertRow（WHATWG HTML §4.9.1，
    // https://html.spec.whatwg.org/multipage/tables.html#dom-table-insertrow）。
    // 覆盖：省略 index / -1 追加、0 头插、n 中插、空表（无 tbody）自动建 tbody、返回值=新 tr。
    // 读侧 rows 已就绪（R2843）；验证策略 = JS 驱动 → apply_mutations_to_html_with_handles 应用 → 断言结构。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let apply = |initial: &str| -> String {
        let ms = mutations.lock().unwrap().clone();
        apply_mutations_to_html_with_handles(initial, &ms).unwrap().0
    };

    // ① insertRow() 省略 index → 追加到既有 tbody 末尾；返回新 tr（tagName=TR）
    let s1 = "<html><body><table id='t'><tbody><tr><td id='a'>a</td></tr></tbody></table></body></html>".to_string();
    *dom_html.lock().unwrap() = s1.clone();
    mutations.lock().unwrap().clear();
    sandbox.execute("globalThis.__r = document.getElementById('t').insertRow();").unwrap();
    assert_eq!(sandbox.execute("globalThis.__r.tagName").unwrap().value, "TR", "insertRow 返回新 tr 元素");
    let out = apply(&s1);
    assert!(out.contains("<tr></tr>"), "insertRow 创建空 tr\n{out}");
    assert!(
        out.find("<tr></tr>").unwrap() > out.find("id=\"a\"").unwrap(),
        "insertRow() 默认追加到末尾（a 之后）\n{out}"
    );

    // ② insertRow(0) → 插到表头（a 之前）
    mutations.lock().unwrap().clear();
    sandbox.execute("document.getElementById('t').insertRow(0);").unwrap();
    let out = apply(&s1);
    assert!(
        out.find("<tr></tr>").unwrap() < out.find("id=\"a\"").unwrap(),
        "insertRow(0) 插到表头（a 之前）\n{out}"
    );

    // ③ insertRow(-1) 显式 -1 → 追加末尾（同 ①）
    mutations.lock().unwrap().clear();
    sandbox.execute("document.getElementById('t').insertRow(-1);").unwrap();
    let out = apply(&s1);
    assert!(
        out.find("<tr></tr>").unwrap() > out.find("id=\"a\"").unwrap(),
        "insertRow(-1) 追加末尾\n{out}"
    );

    // ④ 2 行表 insertRow(1) → 中位（r0 后、r1 前）
    let s2 = "<html><body><table id='t2'><tbody><tr><td id='r0'>0</td></tr><tr><td id='r1'>1</td></tr></tbody></table></body></html>".to_string();
    *dom_html.lock().unwrap() = s2.clone();
    mutations.lock().unwrap().clear();
    sandbox.execute("document.getElementById('t2').insertRow(1);").unwrap();
    let out = apply(&s2);
    let new_tr = out.find("<tr></tr>").unwrap();
    let r0 = out.find("id=\"r0\"").unwrap();
    let r1 = out.find("id=\"r1\"").unwrap();
    assert!(r0 < new_tr && new_tr < r1, "insertRow(1) 插到中位（r0 后、r1 前）\n{out}");

    // ⑤ 空表（无 tbody）insertRow() → 自动建 tbody 包裹新 tr（spec「no tr/tbody/thead/tfoot children」分支）
    let s3 = "<html><body><table id='et'></table></body></html>".to_string();
    *dom_html.lock().unwrap() = s3.clone();
    mutations.lock().unwrap().clear();
    sandbox.execute("document.getElementById('et').insertRow();").unwrap();
    let out = apply(&s3);
    // 原空表无 tbody/tr；insertRow 后须出现 tbody（自动创建）+ tr
    let tb = out.find("<tbody").expect("空表 insertRow 自动创建 tbody");
    let tr = out.find("<tr").unwrap();
    let tb_close = out[tb..].find("</tbody>").unwrap() + tb;
    assert!(tb < tr && tr < tb_close, "新 tr 在自动创建的 tbody 内\n{out}");
}

#[test]
fn test_html_table_delete_row_and_section_r3243() {
    // R3243：HTMLTableElement.deleteRow + HTMLTableSectionElement.insertRow/deleteRow（thead/tbody/tfoot）。
    // https://html.spec.whatwg.org/multipage/tables.html#dom-table-deleterow / #dom-tbody-insertrow
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let apply = |initial: &str| -> String {
        let ms = mutations.lock().unwrap().clone();
        apply_mutations_to_html_with_handles(initial, &ms).unwrap().0
    };

    // ① deleteRow(0) → 移除首行（r0），保留 r1
    let s1 = "<html><body><table id='t'><tbody><tr><td id='r0'>0</td></tr><tr><td id='r1'>1</td></tr></tbody></table></body></html>".to_string();
    *dom_html.lock().unwrap() = s1.clone();
    mutations.lock().unwrap().clear();
    sandbox.execute("document.getElementById('t').deleteRow(0);").unwrap();
    let out = apply(&s1);
    assert!(!out.contains("id=\"r0\""), "deleteRow(0) 移除 r0\n{out}");
    assert!(out.contains("id=\"r1\""), "deleteRow(0) 保留 r1\n{out}");

    // ② deleteRow(-1) → 移除末行（r1），保留 r0
    mutations.lock().unwrap().clear();
    sandbox.execute("document.getElementById('t').deleteRow(-1);").unwrap();
    let out = apply(&s1);
    assert!(out.contains("id=\"r0\""), "deleteRow(-1) 保留 r0\n{out}");
    assert!(!out.contains("id=\"r1\""), "deleteRow(-1) 移除末行 r1\n{out}");

    // ③ deleteRow() 省略 index → 等价 -1（移除末行）
    mutations.lock().unwrap().clear();
    sandbox.execute("document.getElementById('t').deleteRow();").unwrap();
    let out = apply(&s1);
    assert!(!out.contains("id=\"r1\""), "deleteRow() 省略 index 移除末行\n{out}");

    // ④ tbody.insertRow() → 在 section 内追加（section-scoped）
    let s2 = "<html><body><table><tbody id='tb'><tr><td id='x'>x</td></tr></tbody></table></body></html>".to_string();
    *dom_html.lock().unwrap() = s2.clone();
    mutations.lock().unwrap().clear();
    sandbox.execute("document.getElementById('tb').insertRow();").unwrap();
    let out = apply(&s2);
    assert!(out.contains("<tr></tr>"), "tbody.insertRow 创建新 tr\n{out}");
    assert!(
        out.find("<tr></tr>").unwrap() > out.find("id=\"x\"").unwrap(),
        "tbody.insertRow() 在 section 内追加（x 之后）\n{out}"
    );

    // ⑤ thead.insertRow(0) → section 内头插
    let s3 = "<html><body><table><thead id='th'><tr><td id='h'>h</td></tr></thead></table></body></html>".to_string();
    *dom_html.lock().unwrap() = s3.clone();
    mutations.lock().unwrap().clear();
    sandbox.execute("document.getElementById('th').insertRow(0);").unwrap();
    let out = apply(&s3);
    assert!(
        out.find("<tr></tr>").unwrap() < out.find("id=\"h\"").unwrap(),
        "thead.insertRow(0) section 内头插（h 之前）\n{out}"
    );
}

#[test]
fn test_html_tr_cells_mutation_and_index_error_r3243() {
    // R3243：HTMLTableRowElement.insertCell/deleteCell + <tr>.cells 读 getter + IndexSizeError。
    // https://html.spec.whatwg.org/multipage/tables.html#dom-tr-insertcell / #dom-tr-deletecell
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let apply = |initial: &str| -> String {
        let ms = mutations.lock().unwrap().clone();
        apply_mutations_to_html_with_handles(initial, &ms).unwrap().0
    };

    // ① <tr>.cells 读 getter：td+th 混计（document order）
    let s1 = "<html><body><table><tbody><tr id='r'><td id='c0'>a</td><td id='c1'>b</td><th id='c2'>h</th></tr></tbody></table></body></html>".to_string();
    *dom_html.lock().unwrap() = s1.clone();
    sandbox.execute("globalThis.__cells = document.getElementById('r').cells.length;").unwrap();
    assert_eq!(sandbox.execute("globalThis.__cells").unwrap().value, "3", "tr.cells 返 td+th 混合计数（3）");

    // ② insertCell() 追加 td；返回新 td（tagName=TD）
    mutations.lock().unwrap().clear();
    sandbox.execute("globalThis.__nc = document.getElementById('r').insertCell();").unwrap();
    assert_eq!(sandbox.execute("globalThis.__nc.tagName").unwrap().value, "TD", "insertCell 返回新 td 元素");
    let out = apply(&s1);
    // 原 3 cell + 新空 td（末尾，c2 之后）
    assert!(out.contains("<td></td>"), "insertCell 创建空 td\n{out}");
    assert!(
        out.find("<td></td>").unwrap() > out.find("id=\"c2\"").unwrap(),
        "insertCell() 追加到行末尾（c2 之后）\n{out}"
    );

    // ③ insertCell(0) → 头插（c0 之前）
    mutations.lock().unwrap().clear();
    sandbox.execute("document.getElementById('r').insertCell(0);").unwrap();
    let out = apply(&s1);
    assert!(
        out.find("<td></td>").unwrap() < out.find("id=\"c0\"").unwrap(),
        "insertCell(0) 插到行头（c0 之前）\n{out}"
    );

    // ④ deleteCell(-1) → 移除末 cell（c2）
    mutations.lock().unwrap().clear();
    sandbox.execute("document.getElementById('r').deleteCell(-1);").unwrap();
    let out = apply(&s1);
    assert!(!out.contains("id=\"c2\""), "deleteCell(-1) 移除末 cell c2\n{out}");
    assert!(out.contains("id=\"c0\""), "deleteCell(-1) 保留 c0\n{out}");

    // ⑤ deleteCell(0) → 移除首 cell（c0）
    mutations.lock().unwrap().clear();
    sandbox.execute("document.getElementById('r').deleteCell(0);").unwrap();
    let out = apply(&s1);
    assert!(!out.contains("id=\"c0\""), "deleteCell(0) 移除首 cell c0\n{out}");

    // ⑥ IndexSizeError：insertRow/deleteRow/insertCell 越界抛 IndexSizeError（Chromium oracle 一致）
    *dom_html.lock().unwrap() = "<html><body><table id='t'><tbody><tr id='r'><td>a</td></tr></tbody></table></body></html>".to_string();
    sandbox.execute(
        "globalThis.__errs = {};\
         function _trap(fn) { try { fn(); return 'no-throw'; } catch (e) { return (e && e.name) ? e.name : 'unknown'; } }\
         globalThis.__errs.insertRow_neg = _trap(function(){ document.getElementById('t').insertRow(-2); });\
         globalThis.__errs.deleteRow_oob = _trap(function(){ document.getElementById('t').deleteRow(99); });\
         globalThis.__errs.insertCell_neg = _trap(function(){ document.getElementById('r').insertCell(-2); });\
         globalThis.__errs.deleteCell_oob = _trap(function(){ document.getElementById('r').deleteCell(99); });",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__errs.insertRow_neg").unwrap().value, "IndexSizeError", "insertRow(-2) 抛 IndexSizeError");
    assert_eq!(sandbox.execute("globalThis.__errs.deleteRow_oob").unwrap().value, "IndexSizeError", "deleteRow(99) 越界抛 IndexSizeError");
    assert_eq!(sandbox.execute("globalThis.__errs.insertCell_neg").unwrap().value, "IndexSizeError", "insertCell(-2) 抛 IndexSizeError");
    assert_eq!(sandbox.execute("globalThis.__errs.deleteCell_oob").unwrap().value, "IndexSizeError", "deleteCell(99) 越界抛 IndexSizeError");
}

#[test]
fn text_insert_dispatches_beforeinput_then_input() {
    assert_text_insert_dispatches_beforeinput_then_input();
}
