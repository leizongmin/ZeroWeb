// js_dom_bridge_tests part60（R5181）——shadow 树内 dialog.showModal() 连接态校验修复的
// 回归面。MDN 搜索弹层链（site-search-availability）：Lit 式 custom element（client 渲染，
// `static ssr = false`）把 <dialog> 渲染进 JS 侧 shadow root，点击 SSR 按钮 →
// document.querySelector('#mdn-search') instanceof 自定义类 → showModal()。修复前
// _zwIsConnected 的 handle 分支走 _elConnected 的 parentNode 链——断在 shadow root
// fragment 容器（parentNode 无链）→ 恒抛 InvalidStateError「dialog is not connected」，
// 弹层永不打开（Chrome 弹）。修复后 isConnected getter（part04 R90/R91）同款反链 +
// _shadowHandleMeta 跳 host 遍历。

#[test]
fn test_shadow_dialog_showmodal_connected_r5181() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // MDN 形态最小 DOM：<mdn-search-button>（升级后 Lit 渲染 shadow 按钮）+
    // <mdn-search-modal id='mdn-search'>（client 渲染，shadow 内 <dialog>）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <mdn-search-button></mdn-search-button>\
         <mdn-search-modal id='mdn-search'></mdn-search-modal>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://developer.mozilla.org/en-US/".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 镜像 fred components/search-button + search-modal 的打开链（Lit 换成手写 CE：
    // attachShadow + connectedCallback 渲染 + addEventListener——生命周期逐环可断言）。
    sandbox
        .execute(
            r##"
class MDNSearchModal extends HTMLElement {
  constructor() { super(); this.attachShadow({ mode: 'open' }); }
  connectedCallback() { this.shadowRoot.innerHTML = '<dialog><input></dialog>'; }
  showModal() { this.shadowRoot.querySelector('dialog').showModal(); }
}
class MDNSearchButton extends HTMLElement {
  constructor() { super(); this.attachShadow({ mode: 'open' }); }
  connectedCallback() {
    this.shadowRoot.innerHTML = '<button class="mdn-search-button">S</button>';
    var btn = this.shadowRoot.querySelector('button');
    btn.addEventListener('click', function () {
      var search = document.querySelector('#mdn-search');
      if (search instanceof MDNSearchModal) search.showModal();
    });
  }
}
customElements.define('mdn-search-modal', MDNSearchModal);
customElements.define('mdn-search-button', MDNSearchButton);
"##,
        )
        .unwrap();

    // CE 生命周期：两个自定义元素均已升级（shadow root 建立 + shadow 内容渲染）。
    assert_eq!(
        sandbox
            .execute("customElements.getName(document.querySelector('#mdn-search').constructor)")
            .unwrap()
            .value,
        "mdn-search-modal",
        "R5181：modal 元素已按注册名升级（constructor 反查命中）"
    );
    assert_eq!(
        sandbox
            .execute(
                "document.querySelector('mdn-search-button').shadowRoot.querySelector('button') ? '1' : '0'"
            )
            .unwrap()
            .value,
        "1",
        "R5181：button 元素升级 → connectedCallback 渲染 shadow 按钮"
    );
    assert_eq!(
        sandbox
            .execute(
                "document.querySelector('#mdn-search').shadowRoot.querySelector('dialog') ? '1' : '0'"
            )
            .unwrap()
            .value,
        "1",
        "R5181：modal 元素升级 → connectedCallback 渲染 shadow 内 <dialog>"
    );

    // 修复核心：shadow 内 dialog 连接态为真（此前 showModal 校验恒判 disconnected）。
    assert_eq!(
        sandbox
            .execute(
                "document.querySelector('#mdn-search').shadowRoot.querySelector('dialog').isConnected ? '1' : '0'"
            )
            .unwrap()
            .value,
        "1",
        "R5181：shadow 内 dialog.isConnected = true（shadow-including 祖先链，dom-spec #connected）"
    );

    // 端到端：点击 shadow 按钮 → instanceof 命中 → showModal → open 属性 + 模态态。
    sandbox
        .execute(
            "document.querySelector('mdn-search-button').shadowRoot.querySelector('button').click();",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute(
                "document.querySelector('#mdn-search').shadowRoot.querySelector('dialog').hasAttribute('open') ? '1' : '0'"
            )
            .unwrap()
            .value,
        "1",
        "R5181：点击 → 搜索弹层打开（dialog open 属性就位；修复前 showModal 抛 InvalidStateError）"
    );

    // spec 状态机保持：已 open 的 dialog 再 showModal 抛 InvalidStateError（R3290 语义不受反链遍历影响）。
    assert_eq!(
        sandbox
            .execute(
                r#"(function () {
  try {
    document.querySelector('#mdn-search').shadowRoot.querySelector('dialog').showModal();
    return 'no-throw';
  } catch (e) { return e.name; }
})()"#
            )
            .unwrap()
            .value,
        "InvalidStateError",
        "R5181：已打开 dialog 再 showModal 恒抛 InvalidStateError（spec show a modal dialog 步骤）"
    );
}

/// R4001（html-semantics M2 切片 13）：selected autocomplete value 算法 + HTMLCollection
/// named property 访问。INPUT/SELECT/TEXTAREA 的 autocomplete IDL getter 返 token 解析值
///（Off/Automatic 短路、category 门、contact/mode/section 前缀拼串、anchor mantle
///（hidden）恒 ''、field ASCII 小写——corpus keywords 面钉死）；FORM 与其余元素保持
/// 枚举反射（R5009 片 d/R2850）。`document.forms.X` named access 补 HTMLCollection
/// supported property names（id/name 树序首匹配）。WPT 标尺：
/// html/semantics/forms/the-form-element/form-autocomplete.html（67 测）。
#[test]
fn test_selected_autocomplete_value_r4001() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> =
        std::sync::Arc::new(std::sync::Mutex::new("<html><body><form id='f1' autocomplete='off'><input></form></body></html>".to_string()));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // W1：Off/Automatic 短路 + anchor mantle（hidden）+ unknown category → default ''。
    sandbox
        .execute(
            "var i = document.createElement('input');\
             globalThis.__ac = [];\
             function w(v){ i.setAttribute('autocomplete', v); globalThis.__ac.push(i.autocomplete); }\
             w('on'); w('OFF'); w('on off'); w('call-sign');\
             var h = document.createElement('input'); h.type = 'hidden';\
             h.setAttribute('autocomplete', 'on'); globalThis.__ac.push(h.autocomplete);\
             h.setAttribute('autocomplete', 'off'); globalThis.__ac.push(h.autocomplete);\
             var t = document.createElement('textarea');\
             t.setAttribute('autocomplete', '  \\n'); globalThis.__ac.push(t.autocomplete);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__ac.join(',')").unwrap().value,
        "on,off,,,,,",
        "W1：on/OFF 短路、多 token off 折 default、unknown field 折 default、hidden anchor mantle 恒 ''、纯空白 ''"
    );

    // W2：field 小写归一 + contact/mode/section 拼串（spec 序：section → mode → contact → field → credential）。
    sandbox
        .execute(
            "var i = document.createElement('input');\
             globalThis.__ac2 = [];\
             function w(v){ i.setAttribute('autocomplete', v); globalThis.__ac2.push(i.autocomplete); }\
             w(' NAME'); w('home tel'); w('shipping country'); w('billing  work  email');\
             w('  section-FOO  bday'); w('section-login shipping work tel webauthn');\
             w('username webauthn'); w('foo off'); w('foo section-foo billing name');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__ac2.join('|')").unwrap().value,
        "name|home tel|shipping country|billing work email|section-foo bday|section-login shipping work tel webauthn|username webauthn||",
        "W2：token 小写归一 + contact/mode/section 拼串 + credential 后缀 + 非法首 token default"
    );

    // W3：FORM 枚举反射不越界到 input；missing attr → ''（default）。
    sandbox
        .execute(
            "globalThis.__fa = document.getElementById('f1').autocomplete;\
             globalThis.__ia = document.querySelector('#f1 input').autocomplete;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__fa").unwrap().value,
        "off",
        "W3：FORM enumerated 反射（off 合法值透传）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ia)").unwrap().value,
        "",
        "W3：input 无 autocomplete 属性 → ''（IDL-exposed default，非枚举 'on'）"
    );

    // W4：IDL setter 存原串、getter 按现值重算（不走 _reflectedAttrs 缓存）。
    sandbox
        .execute(
            "var i = document.createElement('input');\
             i.autocomplete = ' NAME';\
             globalThis.__sa = i.autocomplete;\
             globalThis.__attr = i.getAttribute('autocomplete');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__sa").unwrap().value,
        "name",
        "W4：IDL set ' NAME' → get 'name'（getter 重算算法）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__attr)").unwrap().value,
        " NAME",
        "W4：IDL setter 存原串内容属性（spec on setting）"
    );

    // W5：HTMLCollection named property（document.forms.X → id/name 树序首匹配，miss → undefined）。
    sandbox
        .execute(
            "var f = document.createElement('form'); f.setAttribute('name', 'myform'); document.body.appendChild(f);\
             globalThis.__nf = document.forms.myform === f;\
             globalThis.__ni = document.forms.nosuch;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__nf)").unwrap().value,
        "true",
        "W5：document.forms.myform named access 命中（name 属性）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ni)").unwrap().value,
        "undefined",
        "W5：miss → undefined（非抛错）"
    );
}
