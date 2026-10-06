// slice36（RP-3 动态名面）：Window named access 动态名收口钉。脚本在安装后才创建/
// 赋名的元素（childList 插入、id/name 属性赋值、移除、多命中拓宽）按 spec 访问时
// 解析——named property visibility 按次访问计算，不限定安装时点快照。
// 实现面：part01 `_mo_notify` → `_zwNADynamicSync`（part05），与 part06 安装面共用
// `_zwNAInstallCollection` 收口。
// https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object

/// slice36 钉共用沙箱装配（同 s32_sandbox!）。
macro_rules! s36_sandbox {
    ($html:expr) => {{
        use std::sync::{Arc, Mutex};
        use zero_script_sandbox::{Sandbox, V8Sandbox};

        let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        sandbox.execute(generate_js_dom_shim()).unwrap();
        let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
        let dom_html = Arc::new(Mutex::new($html.to_string()));
        let page_url = Arc::new(Mutex::new("https://zero.test/s36".to_string()));
        let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
        register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
        sandbox
    }};
}

// ①childList 动态注册：脚本后建元素（createElement + id + appendChild）访问时解析，
// 身份一致；不遮蔽已存在的全局绑定（spec：WindowProperties 只兜底，own property 优先）。
#[test]
fn named_access_dynamic_childlist_register_s36() {
    let mut sandbox = s36_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            "globalThis.__r_pre = String(window.dyn36);
             var d = document.createElement('div');
             d.setAttribute('id', 'dyn36');
             document.body.appendChild(d);
             globalThis.__r_res = window.dyn36 === undefined;
             globalThis.__r_ident = window.dyn36 === d;
             globalThis.__r_tag = window.dyn36.tagName;
             var s = document.createElement('span');
             s.setAttribute('id', 'dyn36b');
             document.body.appendChild(s);
             globalThis.__r_b = window.dyn36b === s;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_pre").unwrap().value,
        "undefined",
        "插入前名不解析（无安装时点快照成员）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_res").unwrap().value,
        "false",
        "appendChild 后访问时解析（spec 按次访问计算，修前恒 undefined）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_ident").unwrap().value,
        "true",
        "解析目标即脚本创建的同一元素（身份一致）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_tag").unwrap().value,
        "DIV",
        "解析目标元素语义可读（ tagName 直读）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_b").unwrap().value,
        "true",
        "同批次第二个动态名同样解析（注册按元素粒度）"
    );
}

// ②属性变异面：脚本后建元素 setAttribute('id') 触发注册；移除 id 失格注销；改名后
// 新名注册（WPT changing.html 形态：id 换名旧名注销、新名注册）。
#[test]
fn named_access_dynamic_attr_register_disqualify_s36() {
    let mut sandbox = s36_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            "var d = document.createElement('div');
             document.body.appendChild(d);
             d.setAttribute('id', 'aa36');
             globalThis.__r_reg = window.aa36 === d;
             d.removeAttribute('id');
             globalThis.__r_unreg = window.aa36 === undefined;
             d.setAttribute('id', 'bb36');
             globalThis.__r_re = window.bb36 === d && window.aa36 === undefined;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_reg").unwrap().value,
        "true",
        "append 后 setAttribute('id') 注册动态名（attr 变异面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_unreg").unwrap().value,
        "true",
        "removeAttribute('id') 失格注销（spec candidate 移除）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_re").unwrap().value,
        "true",
        "id 换名：新名注册 + 旧名注销（WPT changing.html id-swap 形态）"
    );
}

// ③name 面白名单 + still-match：embed/form/img/object 的 name 属性注册（div 不行）；
// 元素 id 改变但 name 仍命中 → 全局保持（WPT name-attribute-elements.html 形态）。
#[test]
fn named_access_dynamic_name_face_whitelist_s36() {
    let mut sandbox = s36_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            "var im = document.createElement('img');
             document.body.appendChild(im);
             im.setAttribute('name', 'nm36');
             globalThis.__r_img = window.nm36 === im;
             var dv = document.createElement('div');
             document.body.appendChild(dv);
             dv.setAttribute('name', 'nmdv');
             globalThis.__r_div = window.nmdv === undefined;
             im.setAttribute('id', 'imid36');
             globalThis.__r_still = window.nm36 === im;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_img").unwrap().value,
        "true",
        "img name 属性动态注册（name 面，spec whitelist 四类）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_div").unwrap().value,
        "true",
        "div name 属性不注册（spec：name 面仅 embed/form/img/object）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_still").unwrap().value,
        "true",
        "id 换新名后 name 仍命中 → 全局保持（still-match，WPT name-attribute-elements）"
    );
}

// ④多命中拓宽 + live：动态插入第二个同名元素 → 全局从单元素拓宽为 HTMLCollection，
// 集合 live 维护（与安装面同口径 _zwNAInstallCollection）。
#[test]
fn named_access_dynamic_broaden_collection_live_s36() {
    let mut sandbox = s36_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            "var a = document.createElement('div');
             a.setAttribute('id', 'dup36');
             document.body.appendChild(a);
             globalThis.__r_single = window.dup36 === a;
             var b = document.createElement('div');
             b.setAttribute('id', 'dup36');
             document.body.appendChild(b);
             globalThis.__r_iscol = window.dup36 instanceof HTMLCollection;
             globalThis.__r_len = window.dup36.length;
             globalThis.__r_order = window.dup36[0] === a && window.dup36[1] === b;
             var c = document.createElement('div');
             c.setAttribute('id', 'dup36');
             document.body.appendChild(c);
             globalThis.__r_len3 = window.dup36.length;
             globalThis.__r_same = window.dup36[0] === a;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_single").unwrap().value,
        "true",
        "单命中全局形态 = 元素本身"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_iscol").unwrap().value,
        "true",
        "第二同名插入 → 拓宽为 HTMLCollection（spec 多命中集合形态）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_len").unwrap().value,
        "2",
        "拓宽集合长度 = 2"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_order").unwrap().value,
        "true",
        "拓宽集合树序（append 形态）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_len3").unwrap().value,
        "3",
        "拓宽后集合 live：第三个同名 appendChild 即时 +1"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_same").unwrap().value,
        "true",
        "live 维护成员序稳定"
    );
}

// ⑤移除注销 + 重挂重注册：removeChild 后名失效；重新插入恢复解析（tree 序按
// compareDocumentPosition）；不遮蔽静态安装面（parsed 同名共存时并集合）。
#[test]
fn named_access_dynamic_removal_reregister_s36() {
    let mut sandbox = s36_sandbox!("<html><body><div id='st36'></div></body></html>");
    sandbox
        .execute(
            "globalThis.__r_static = window.st36.tagName;
             var d = document.createElement('div');
             d.setAttribute('id', 'st36');
             document.body.appendChild(d);
             globalThis.__r_iscol = window.st36 instanceof HTMLCollection;
             globalThis.__r_len = window.st36.length;
             document.body.removeChild(d);
             globalThis.__r_back = window.st36.tagName;
             globalThis.__r_single = window.st36 === document.getElementById('st36');")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_static").unwrap().value,
        "DIV",
        "静态安装名基线（parsed 元素）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_iscol").unwrap().value,
        "true",
        "动态插入同名第二命中 → 并入集合（静态面与动态面同池）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_len").unwrap().value,
        "2",
        "集合长度 = parsed + 动态两命中"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_back").unwrap().value,
        "DIV",
        "动态成员移除 → 回落单元素形态（morph，旧 s32 语义不受影响）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_single").unwrap().value,
        "true",
        "回落目标为剩余 parsed 元素"
    );
}
