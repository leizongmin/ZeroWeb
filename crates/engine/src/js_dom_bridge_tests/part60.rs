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

/// R4002（html-semantics M2 切片 14）：HTMLMeterElement 实际值算法 + double setter。
/// 六 IDL 属性 getter 返实际值（min 缺省 0；max 缺省 1、<min 折 min；value/low/high/
/// optimum 缺省与钳制见 spec）；属性解析走「rules for parsing floating-point number
/// values」（前导 + 跳过、裸符号/裸点/trailing junk → 缺省）；setter WebIDL double
/// 转换（非有限 → TypeError）+ best representation 写内容属性。WPT 标尺：
/// html/semantics/forms/the-meter-element/meter.html（51 测）+
/// meter-leading-plus-sign.html（12 测）。
#[test]
fn test_meter_actual_values_r4002() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> =
        std::sync::Arc::new(std::sync::Mutex::new("<html><body></body></html>".to_string()));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // W1：缺省面——全缺省 {0,0,1,0,1,0.5}；max < min 折 min（连 low/high/optimum 随折）。
    sandbox
        .execute(
            "var m = document.createElement('meter');\
             globalThis.__m1 = [m.value, m.min, m.max, m.low, m.high, m.optimum].join(',');\
             m.max = -1; globalThis.__m2 = [m.max, m.low, m.high, m.optimum].join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__m1").unwrap().value,
        "0,0,1,0,1,0.5",
        "W1：全缺省——value 0/min 0/max 1/low 0/high 1/optimum 0.5"
    );
    assert_eq!(
        sandbox.execute("globalThis.__m2").unwrap().value,
        "0,0,0,0",
        "W1：max=-1 < min=0 折 min，low/high 缺省随折，optimum 中点 0"
    );

    // W2：钳制面——value<min / value>max / low 越界 / high<low 折 low / optimum 越界。
    sandbox
        .execute(
            "var a = document.createElement('meter'); a.min = 10; a.max = 20; a.value = 5;\
             var b = document.createElement('meter'); b.min = 10; b.max = 20; b.value = 25;\
             var c = document.createElement('meter'); c.min = 10; c.max = 20; c.low = 5; c.high = 30; c.optimum = 9;\
             var d = document.createElement('meter'); d.min = -20; d.low = -10.3; d.high = -15.2; d.max = -2; d.value = -9;\
             globalThis.__m3 = [a.value, b.value, c.low, c.high, c.optimum, d.low, d.high, d.value].join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__m3").unwrap().value,
        "10,20,10,20,10,-10.3,-10.3,-9",
        "W2：value<min 折 min、value>max 折 max、low/high 钳 [min,max]、high<low 折 low、optimum<min 折 min"
    );

    // W3：内容属性解析——非法串回落缺省；前导 + 跳过；trailing junk 报错（整串解析）。
    sandbox
        .execute(
            "var i = document.createElement('div'); i.innerHTML = '<meter id=mA value=abc></meter><meter id=mB value=-10 min=4.5></meter>\
             <meter id=mC value=+5 max=+10></meter><meter id=mD value=10foo></meter><meter id=mE max=+-5></meter>';\
             document.body.appendChild(i);\
             var rA = document.getElementById('mA'), rB = document.getElementById('mB');\
             var rC = document.getElementById('mC'), rD = document.getElementById('mD'), rE = document.getElementById('mE');\
             globalThis.__m4 = [rA.value, rB.value, rC.value, rC.max, rD.value, rE.max].join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__m4").unwrap().value,
        "0,4.5,5,10,0,1",
        "W3：'abc'→0、-10 折 min 4.5、'+5'/'+10' 前导 + 跳过、'10foo' 整串报错→0、'+-5' 报错→max 缺省 1"
    );

    // W4：double setter——非有限 → TypeError（value/min/max/low/high/optimum 六面）；
    // 合法值写 best representation 内容属性（12.5 → "12.5"）。
    sandbox
        .execute(
            "var m = document.createElement('meter'); globalThis.__threw = [];\
             ['value','min','max','low','high','optimum'].forEach(function (k) {\
               try { m[k] = 'foobar'; globalThis.__threw.push('no'); } catch (e) { globalThis.__threw.push(e.constructor.name); }\
             });\
             m.min = 0; m.max = 100; m.value = 12.5; globalThis.__attr = m.getAttribute('value'); globalThis.__got = m.value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__threw.join(',')").unwrap().value,
        "TypeError,TypeError,TypeError,TypeError,TypeError,TypeError",
        "W4：六属性 setter 'foobar' 全抛 TypeError（WebIDL double 转换）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__attr)").unwrap().value,
        "12.5",
        "W4：合法值写 best representation 内容属性"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__got)").unwrap().value,
        "12.5",
        "W4：setter 后 getter 读回实际值"
    );

    // W5：getter 恒按内容属性重算——setAttribute 后 IDL 读即时反映（无缓存钉死）。
    sandbox
        .execute(
            "var m = document.createElement('meter'); m.min = 10; m.max = 20; m.value = 15;\
             m.setAttribute('value', '50'); globalThis.__m5 = m.value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__m5)").unwrap().value,
        "20",
        "W5：setAttribute 后 getter 重算钳制（50 → max 20）"
    );
}

/// R4003（html-semantics M2 切片 15）：label.control + labelable el.labels。
/// labeled control 算法（for → getElementById 命中且 labelable；无 for → 子树
/// 首个 labelable 后代）；labels = labeled control 为本元素的 label 活集合
///（[SameObject] 缓存，hidden 切换期 null 但对象存活）；labelable = button/
/// input（type 非 hidden）/meter/output/progress/select/textarea，hidden input
/// labels → null，fieldset/label/object/img/keygen 无该 IDL → undefined。
/// WPT 标尺：labelable-elements.html（26 测）+ button-labels.html +
/// input-labels.html。
#[test]
fn test_labels_idl_r4003() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> =
        std::sync::Arc::new(std::sync::Mutex::new("<html><body></body></html>".to_string()));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 挂 corpus 同构树（form 内 label[for] × labelable/非 labelable 全家福 + 无 for 嵌套）。
    sandbox
        .execute(
            "var d = document.createElement('div');\
             d.innerHTML = '<form>'\
             + '<output id=tOutput></output><label id=lb0 for=tOutput></label>'\
             + '<progress id=tProgress></progress><label id=lb1 for=tProgress></label>'\
             + '<select id=tSelect></select><label id=lb2 for=tSelect></label>'\
             + '<textarea id=tArea></textarea><label id=lb3 for=tArea></label>'\
             + '<button id=tButton></button><label id=lb4 for=tButton></label>'\
             + '<input type=hidden id=tHidden><label id=lb5 for=tHidden></label>'\
             + '<input type=radio id=tRadio><label id=lb6 for=tRadio></label>'\
             + '<meter id=tMeter></meter><label id=lb8 for=tMeter></label>'\
             + '<keygen id=tKeygen><label id=lb7 for=tKeygen></label>'\
             + '<fieldset id=tFieldset></fieldset><label id=lb9 for=tFieldset></label>'\
             + '<label id=tLabel></label><label id=lb10 for=tLabel></label>'\
             + '<object id=tObject></object><label id=lb11 for=tObject></label>'\
             + '<img id=tImg><label id=lb12 for=tImg></label>'\
             + '</form>'\
             + '<label id=outerL>B:<label id=innerL>C:<button id=nButton></button></label></label>';\
             document.body.appendChild(d);",
        )
        .unwrap();

    // W1：control——for → 七 labelable 各命中（.id 可读）；非 labelable → null。
    sandbox
        .execute(
            "globalThis.__w1 = [\
             ['lb0','tOutput'],['lb1','tProgress'],['lb2','tSelect'],['lb3','tArea'],\
             ['lb4','tButton'],['lb6','tRadio'],['lb8','tMeter']].map(function (p) {\
               var c = document.getElementById(p[0]).control;\
               return c ? (c.id === p[1] ? 'ok' : 'bad:' + c.id) : 'null';\
             }).join(',')\
             + '|' + ['lb5','lb7','lb9','lb10','lb11','lb12'].map(function (id) {\
               return String(document.getElementById(id).control);\
             }).join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w1").unwrap().value,
        "ok,ok,ok,ok,ok,ok,ok|null,null,null,null,null,null",
        "W1：control 命中七 labelable；hidden/keygen/fieldset/label/object/img 的 label.control → null"
    );

    // W2：control 无 for → 子树首个 labelable 后代（嵌套双层 label 同指 button）。
    sandbox
        .execute(
            "globalThis.__w2 = [document.getElementById('outerL').control.id,\
             document.getElementById('innerL').control.id].join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w2").unwrap().value,
        "nButton,nButton",
        "W2：无 for 嵌套 label 的 control 均为首个 labelable 后代"
    );

    // W3：labels——长度 1 + [0].id；无关联 → length 0。
    sandbox
        .execute(
            "globalThis.__w3 = [\
             ['tOutput','lb0'],['tProgress','lb1'],['tSelect','lb2'],['tArea','lb3'],\
             ['tButton','lb4'],['tRadio','lb6'],['tMeter','lb8']].map(function (p) {\
               var el = document.getElementById(p[0]);\
               return el.labels.length === 1 && el.labels[0].id === p[1] ? 'ok' : 'bad';\
             }).join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w3").unwrap().value,
        "ok,ok,ok,ok,ok,ok,ok",
        "W3：七 labelable 元素 labels 长度与首项（树序）正确"
    );

    // W4：hidden 面——labels === null；type→text 后 1 项；保留集合活性（type 回
    // hidden → 旧集合 length 0 且 getter null）；再切 checkbox → SameObject + 1 项。
    sandbox
        .execute(
            "var h = document.getElementById('tHidden');\
             globalThis.__w4a = String(h.labels);\
             h.type = 'text';\
             var ok1 = h.labels.length === 1 && h.labels[0].id === 'lb5';\
             var kept = h.labels;\
             h.type = 'hidden';\
             globalThis.__w4 = [String(h.labels), String(kept.length), ok1 ? 'ok' : 'bad'].join(',');\
             h.type = 'checkbox';\
             globalThis.__w4b = [kept === h.labels, h.labels.length].join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w4a").unwrap().value,
        "null",
        "W4：hidden input labels → null"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w4").unwrap().value,
        "null,0,ok",
        "W4：type→text 后 labels 1 项；回 hidden 后保留集合空且 getter null"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w4b").unwrap().value,
        "true,1",
        "W4：切回 checkbox 后 SameObject 成立且集合恢复 1 项"
    );

    // W5：非 labelable 元素无 labels IDL（undefined）；labels instanceof NodeList。
    sandbox
        .execute(
            "globalThis.__w5 = ['tKeygen','tFieldset','tLabel','tObject','tImg'].map(function (id) {\
               return String(document.getElementById(id).labels);\
             }).join(',') + '|' + (document.getElementById('tButton').labels instanceof NodeList);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w5").unwrap().value,
        "undefined,undefined,undefined,undefined,undefined|true",
        "W5：keygen/fieldset/label/object/img labels → undefined；labelable 集合为 NodeList 实例"
    );
}

/// R4003b（html-semantics M2 切片 15）：label.focus() 前递。label 非可聚焦
///（无 tabindex / tabindex="" / display:none 子树）时 focus 转发给 labeled
/// control；tabindex="0"/"-1" 渲染态落 label 本尊不转发。转发经 control 自身
/// focus()（已聚焦 no-op 短路随之继承）。WPT 标尺：
/// html/semantics/forms/the-label-element/forward-focus-to-associated-element.html（6 测）。
#[test]
fn test_label_focus_forwarding_r4003b() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    // 解析树挂载（WPT 同构——computed style 面按解析文档解析，运行时 innerHTML
    // 挂载的 display:none 解析不参与，W1 case f 须走与 WPT 一致的解析形态）。
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> =
        std::sync::Arc::new(std::sync::Mutex::new(
            "<html><body><form>\
             <input id=fa type=checkbox><label id=la for=fa>a</label>\
             <label id=lb><input id=fb type=checkbox></label>\
             <label id=lc tabindex=0><input id=fc type=checkbox></label>\
             <label id=ld tabindex=-1><input id=fd type=checkbox></label>\
             <label id=le tabindex=\"\"><input id=fe type=checkbox></label>\
             <input id=ff type=checkbox><label id=lf for=ff tabindex=0 style=\"display:none\">f</label>\
             </form></body></html>"
                .to_string(),
        ));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // W1：六向聚焦——for/无 for/tabindex=""/display:none+tabindex=0 前递 control；
    // tabindex="0"/"-1" 渲染态落 label 本尊（既有通用路径保持）。
    sandbox
        .execute(
            r##"
globalThis.__hits = [];
['fa','fb','fc','fd','fe','ff','la','lb','lc','ld','le','lf'].forEach(function (id) {
  document.getElementById(id).addEventListener('focus', function () { globalThis.__hits.push(id); });
});
['a','b','c','d','e','f'].forEach(function (n) { document.getElementById('l' + n).focus(); });
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__hits.join(',')").unwrap().value,
        "fa,fb,lc,ld,fe,ff",
        "W1：for/无 for/tabindex=\"\"/display:none 前递 input；tabindex=0/-1 渲染态落 label 本尊"
    );

    // W2：转发已聚焦 control → 经 control 自身 no-op 短路（不重派 focus）。
    sandbox
        .execute(
            "globalThis.__hits = [];\
             document.getElementById('la').focus();\
             globalThis.__w2a = globalThis.__hits.join(',');\
             globalThis.__hits = [];\
             document.getElementById('la').focus();\
             globalThis.__w2b = globalThis.__hits.join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w2a").unwrap().value,
        "fa",
        "W2：la.focus() 首次前递 fa 获焦（W1 尾 ff 持焦点 → 非重派）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w2b").unwrap().value,
        "",
        "W2：fa 已聚焦 → 二次 la.focus() 经转发 no-op 短路"
    );
}

/// R4003c（html-semantics M2 切片 15）：html/body offsetParent → null。CSSOM View
/// §offsetParent——body/html 无 offset parent；此前 body 有 rect 自返 body，「沿
/// offsetParent 上行」的页面惯用遍历死循环（inert-label-focus 语料 clickOn 坐标
/// 累积 90s Execution timeout 实证）。
#[test]
fn test_offsetparent_body_null_r4003c() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> =
        std::sync::Arc::new(std::sync::Mutex::new("<html><body><div id=d></div></body></html>".to_string()));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // body/html 自返 null；自 body 起的 offsetParent 上行遍历 1 步终止（修复前
    // body 自返 body 死循环）。div 的 body 回落依赖布局 rect（lib 沙箱无布局，
    // 该面由 WPT 语料 clickOn 遍历覆盖）。
    sandbox
        .execute(
            "globalThis.__op = [\
             String(document.body.offsetParent === null),\
             String(document.documentElement.offsetParent === null)].join(',');\
             var steps = 0;\
             for (var n = document.body; n; n = n.offsetParent) { if (++steps > 8) break; }\
             globalThis.__steps = steps;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__op").unwrap().value,
        "true,true",
        "W1：body/html offsetParent → null（CSSOM View §offsetParent）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__steps)").unwrap().value,
        "1",
        "W2：自 body 的上行遍历 1 步终止（修复前 body 自返死循环）"
    );
}

/// R4004（html-semantics M2 切片 16）：label 激活行为转发面。交互内容判定
///（HTML §4.2.6.2：a[href]/audio[controls]/button/details/iframe/img[usemap]/
/// input(type≠hidden)/label/select/textarea/video[controls]，仅 HTML 命名空间，
/// tabindex/object 不算）豁免转发；非交互 target 冒泡经 label 站时对 labeled
/// control（for 优先）跑 synthetic click。属性存在性判定走 has 句柄
///（`__zw_get_attr_handle` 缺失返空串，`!= null` 恒真——修复面）。附 R368 盖章
/// 属性回填（createElement 产物 append 后 host 侧 type 可读——hidden 仍非
/// labelable）。WPT 标尺：clicking-noninteractive-unlabelable-content.html +
/// clicking-noninteractive-labelable-content.html。
#[test]
fn test_label_activation_r4004() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> =
        std::sync::Arc::new(std::sync::Mutex::new("<html><body></body></html>".to_string()));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 挂 corpus 同构面板：label[for] × 交互/非交互 target 全家福，控件各自独立
    // button（for 关联，排除无 for 子树查找的耦合）。
    sandbox
        .execute(
            "var d = document.createElement('div');\
             d.innerHTML = '<form>'\
             + '<label id=cLa for=cBtnA><a id=cA1></a></label><button id=cBtnA></button>'\
             + '<label id=cLb for=cBtnB><a id=cA2 href=\"#x\"></a></label><button id=cBtnB></button>'\
             + '<label id=cLc for=cBtnC><audio id=cAu1></audio></label><button id=cBtnC></button>'\
             + '<label id=cLd for=cBtnD><audio id=cAu2 controls></audio></label><button id=cBtnD></button>'\
             + '<label id=cLe for=cBtnE><img id=cIm1></label><button id=cBtnE></button>'\
             + '<label id=cLf for=cBtnF><img id=cIm2 usemap=\"#m\"></label><button id=cBtnF></button>'\
             + '<label id=cLg for=cBtnG><input id=cIn1 type=hidden></label><button id=cBtnG></button>'\
             + '<label id=cLh for=cBtnH><input id=cIn2 type=text></label><button id=cBtnH></button>'\
             + '<label id=cLi for=cBtnI><div id=cDv tabindex=\"0\"></div></label><button id=cBtnI></button>'\
             + '<label id=cLj for=cBtnJ><object id=cOb></object></label><button id=cBtnJ></button>'\
             + '<label id=cLk for=cBtnK><details id=cDt></details></label><button id=cBtnK></button>'\
             + '<label id=cLl for=cBtnL><select id=cSel></select></label><button id=cBtnL></button>'\
             + '</form>'\
             + '<label id=w2l><button id=w2b></button></label>'\
             + '<label id=w2h><input type=hidden id=w2hi></label>';\
             document.body.appendChild(d);\
             globalThis.__clk = {};\
             ['cBtnA','cBtnB','cBtnC','cBtnD','cBtnE','cBtnF','cBtnG','cBtnH','cBtnI','cBtnJ','cBtnK','cBtnL','w2b','w2hi'].forEach(function (id) {\
               globalThis.__clk[id] = 0;\
               document.getElementById(id).addEventListener('click', function () { globalThis.__clk[id]++; });\
             });\
             globalThis.__fire = function (id) {\
               document.getElementById(id).dispatchEvent(new MouseEvent('click', { bubbles: true }));\
             };",
        )
        .unwrap();

    // W1：非交互 target 冒泡经 label 站转发到 for 控件；交互 target 豁免。
    // a[href]/audio[controls]/img[usemap]/input(hidden↔text) 四组属性对称对，
    // 钉死 has 存在性判定（缺失属性 ≠ 存在——修复前恒真恒豁免）。
    sandbox
        .execute(
            "['cA1','cAu1','cIm1','cIn1','cDv','cOb'].forEach(function (id) { globalThis.__fire(id); });\
             globalThis.__w1a = ['cBtnA','cBtnC','cBtnE','cBtnG','cBtnI','cBtnJ'].map(function (id) { return globalThis.__clk[id]; }).join(',');\
             ['cA2','cAu2','cIm2','cIn2','cDt','cSel'].forEach(function (id) { globalThis.__fire(id); });\
             globalThis.__w1b = ['cBtnB','cBtnD','cBtnF','cBtnH','cBtnK','cBtnL'].map(function (id) { return globalThis.__clk[id]; }).join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w1a").unwrap().value,
        "1,1,1,1,1,1",
        "W1：a 无 href / audio 无 controls / img 无 usemap / input hidden / div[tabindex] / object 均非交互 → 控件各转发 1 次"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w1b").unwrap().value,
        "0,0,0,0,0,0",
        "W1：a[href] / audio[controls] / img[usemap] / input text / details / select 交互 → 豁免转发"
    );

    // W2：重入面——label 内 button 自身点击（target 交互 → 挂钩 skip）计 1；
    // label 内 hidden input 点击（非交互、非 labelable、无 control）计 1 不多计。
    sandbox
        .execute("globalThis.__fire('w2b'); globalThis.__fire('w2hi');\
                  globalThis.__w2 = globalThis.__clk.w2b + ',' + globalThis.__clk.w2hi;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w2").unwrap().value,
        "1,1",
        "W2：交互 target 免转发自身计 1；hidden input 无 control 不转发计 1（修复前误 labelable 转发自身计 2）"
    );

    // W3：R368 盖章属性回填——createElement 产物（plain，无身份）append 入 label
    // 后 host 侧 type 可读：hidden 仍非 labelable（修复前 host 裸节点 type 空被当
    // 默认 text → 误 labelable，树序先命中 hidden）。回填生效时 [hidden, button]
    // 的 control 跳过 hidden 命中 button；纯 hidden 的 label.control null。
    sandbox
        .execute(
            "var l3 = document.createElement('label'); document.body.appendChild(l3);\
             var i3 = document.createElement('input'); i3.setAttribute('type', 'hidden');\
             l3.appendChild(i3);\
             var b3 = document.createElement('button'); l3.appendChild(b3);\
             var l6 = document.createElement('label'); document.body.appendChild(l6);\
             var i4 = document.createElement('input'); i4.setAttribute('type', 'hidden');\
             l6.appendChild(i4);\
             globalThis.__w3 = String(l3.control === b3) + ',' + String(l6.control);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w3").unwrap().value,
        "true,null",
        "W3：盖章回填后 host 侧 hidden 非 labelable（[hidden,button] control 命中 button）；纯 hidden label.control null"
    );

    // W4：SVG 命名空间 <details> 非交互（名单仅 HTML 命名空间）——svg details
    // 在 label[for] 内 click 照常转发。
    sandbox
        .execute(
            "var c4 = document.createElement('div');\
             c4.innerHTML = '<label id=cLw for=cBtnA></label>';\
             document.body.appendChild(c4.firstChild);\
             var sd = document.createElementNS('http://www.w3.org/2000/svg', 'details');\
             document.getElementById('cLw').appendChild(sd);\
             globalThis.__clk.cBtnA = 0;\
             sd.dispatchEvent(new MouseEvent('click', { bubbles: true }));\
             globalThis.__w4 = globalThis.__clk.cBtnA;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__w4)").unwrap().value,
        "1",
        "W4：SVG 命名空间 details 非交互 → label[for=cBtnA] 照常转发"
    );
}

// R4005/R4006（html-semantics M2 切片 17）：form owner 动态关联 + getElementById
// pending 窗口同 id 树序。修复面：
// ① `.form` getter 祖先上行改 JS 视图 parentNode（handle `_zwNodeParent` 反链 /
//    R334 pending 槽 / sel host 链三族）——旧 `_ancestorChain`（host 链）对 handle-only
//    子恒空、对异步 wire 窗口的 sel 子读旧树；
// ② form attr 关联查控件所在树（detached form 仍是 owner）+ OBJECT 入 gate；
// ③ R52 消零 purge 后 `_makeProxy` WeakRef 复活（同 turn append+remove 后页面仍持有
//    的原 proxy 不重铸——identity 断修复）；
// ④ getElementById pending 候选池树序取首（insertBefore 前插/改名重登记）+ 改名条目
//    `_r125Overridden` 门。sel 面的 latest-wins 属性读由语料 parser-inserted 用例覆盖。
#[test]
fn test_form_owner_dynamic_r4005() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> =
        std::sync::Arc::new(std::sync::Mutex::new("<html><body></body></html>".to_string()));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // W1：handle-only 子的祖先上行（JS 视图反链）+ reparent 同步跟随。修复前
    // `_ancestorChain` 无 sel 恒空 → basic 关联 null；reparent 后 host 链滞后旧 form。
    sandbox
        .execute(
            "var f1 = document.createElement('form'); f1.id = 'rF1'; document.body.appendChild(f1);\
             var c1 = document.createElement('button'); f1.appendChild(c1);\
             var f2 = document.createElement('form'); f2.id = 'rF2'; document.body.appendChild(f2);\
             globalThis.__w1a = String(c1.form === f1);\
             f2.appendChild(c1);\
             globalThis.__w1b = String(c1.form === f2);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w1a").unwrap().value,
        "true",
        "W1：handle 子 append 后 `.form` 即关联祖先 form（JS 视图反链上行，修复前 null）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w1b").unwrap().value,
        "true",
        "W1：reparent 后 `.form` 同步跟到新父 form（修复前滞留旧树）"
    );

    // W2：form attr 关联的查找域 = 控件所在树 + R52 purge 身份。控件随 form 出文档
    // （detached 树）后 detached form 仍是 owner，且 === 页面持有对象（修复前 gEBI/
    // wrapHandle 重铸第二 wrapper → assert_equals 身份断失败）。
    sandbox
        .execute(
            "var f3 = document.createElement('form'); f3.id = 'rF3'; document.body.appendChild(f3);\
             var c2 = document.createElement('button'); c2.setAttribute('form', 'rF3');\
             f3.appendChild(c2);\
             document.body.removeChild(f3);\
             globalThis.__w2 = String(c2.form === f3);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w2").unwrap().value,
        "true",
        "W2：detached 树内 form attr 仍关联（同树查找）且 === 原 proxy（WeakRef 复活，修复前身份断）"
    );

    // W3：同 id 树序（R4006 候选池）——改名进来的 duplicate id 候选按树序取首；
    // 移除后回落次序；尾部重插回到树序；改名走远的条目不误命中。
    sandbox
        .execute(
            "var f4 = document.createElement('form'); f4.id = 'rF4'; document.body.appendChild(f4);\
             var f5 = document.createElement('form'); f5.id = 'rF5'; document.body.appendChild(f5);\
             var c3 = document.createElement('button'); c3.setAttribute('form', 'rF4');\
             f4.appendChild(c3);\
             f5.id = 'rF4';\
             f5.parentNode.insertBefore(f5, f4);\
             globalThis.__w3a = String(document.getElementById('rF4') === f5);\
             globalThis.__w3b = String(c3.form === f5);\
             f5.parentNode.removeChild(f5);\
             globalThis.__w3c = String(c3.form === f4);\
             f5.id = 'rF5';\
             f4.parentNode.appendChild(f5);\
             globalThis.__w3d = String(c3.form === f4);\
             f4.id = 'rF9';\
             globalThis.__w3e = String(document.getElementById('rF4') === null\
               && document.getElementById('rF9') === f4);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w3a").unwrap().value,
        "true",
        "W3：getElementById 同 id 多候选树序取首（insertBefore 前插的 f5 胜出插入序首位）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w3b").unwrap().value,
        "true",
        "W3：form owner 跟随树序首个 id 匹配（=f5）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w3c").unwrap().value,
        "true",
        "W3：f5 移除后回落 f4（pending-removed 门 + 候选池过滤）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w3d").unwrap().value,
        "true",
        "W3：f5 尾部重插后树序首回到 f4（form2 保 duplicate id 场景）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w3e").unwrap().value,
        "true",
        "W3：f4 改名后旧 id 不误命中（null）且新 id 可查（=f4）——pending 条目改名摘旧键登新键"
    );

    // W4：OBJECT 入 form-associated gate（R4005 补面——修复前 `.form` getter 不认
    // object，13 案全灭）。
    sandbox
        .execute(
            "var ob = document.createElement('object');\
             var f6 = document.createElement('form'); f6.id = 'rF6'; document.body.appendChild(f6);\
             f6.appendChild(ob);\
             globalThis.__w4 = String(ob.form === f6);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w4").unwrap().value,
        "true",
        "W4：object 元素 `.form` 关联祖先 form（gate 补 OBJECT）"
    );
}

// R4007（html-semantics M2 切片 18）：HTMLLabelElement.form——labeled control 的
// form owner（无 labeled control → null）。修复面：`.form` getter 补 LABEL 分支，
// 经 `_zwLabeledControl`（for 优先 + 子树树序 DFS，R4003/R4004 既有）委托控件的
// `.form`（R4005 关联面）；label 自身 form content attribute 不参与（label 非
// form-associated——form.html label-form 语料期望 null）。`.control` 本体 R4003
// 已在 part04 分发区，本契约只钉 `.form` 委托链。
#[test]
fn test_label_form_r4007() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> =
        std::sync::Arc::new(std::sync::Mutex::new("<html><body></body></html>".to_string()));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // W1：无 labeled control → null（裸 label / form 属性不参与——label 非
    // form-associated）。
    sandbox
        .execute(
            "var f1 = document.createElement('form'); f1.id = 'lF1'; document.body.appendChild(f1);\
             var lb1 = document.createElement('label'); lb1.id = 'lLb1'; document.body.appendChild(lb1);\
             var lb2 = document.createElement('label'); lb2.id = 'lLb2'; lb2.setAttribute('form', 'lF1');\
             document.body.appendChild(lb2);\
             globalThis.__l1a = String(lb1.form === null);\
             globalThis.__l1b = String(lb2.form === null);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__l1a").unwrap().value,
        "true",
        "W1：裸 label `.form` null（修复前 undefined）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__l1b").unwrap().value,
        "true",
        "W1：label 自身 form 属性不参与关联（期望 null 非 form）"
    );

    // W2：包绕控件 → 控件的 form owner（祖先 form）；for= 指向 → 目标控件的
    // form owner；reparent 后同步跟随（委托 R4005 面不重复钉树序细节）。
    sandbox
        .execute(
            "var c1 = document.createElement('input');\
             var lb3 = document.createElement('label'); lb3.appendChild(c1);\
             f1.appendChild(lb3);\
             globalThis.__l2a = String(lb3.form === f1);\
             var c2 = document.createElement('input'); c2.id = 'lC2'; f1.appendChild(c2);\
             var lb4 = document.createElement('label'); lb4.setAttribute('for', 'lC2');\
             document.body.appendChild(lb4);\
             globalThis.__l2b = String(lb4.form === f1);\
             var f2 = document.createElement('form'); f2.id = 'lF2'; document.body.appendChild(f2);\
             f2.appendChild(c2);\
             globalThis.__l2c = String(lb4.form === f2 && lb4.control.form === f2);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__l2a").unwrap().value,
        "true",
        "W2：包绕控件 label `.form` = 控件祖先 form"
    );
    assert_eq!(
        sandbox.execute("globalThis.__l2b").unwrap().value,
        "true",
        "W2：for= label `.form` = 目标控件 form owner"
    );
    assert_eq!(
        sandbox.execute("globalThis.__l2c").unwrap().value,
        "true",
        "W2：控件 reparent 后 label.form 同步跟随新 owner（委托 R4005 面）"
    );

    // W3：for= 指向带 form 属性的控件 → 控件 form attr 关联面（R4005）；for=
    // miss（非 labelable 或无此 id）→ null。
    sandbox
        .execute(
            "var c3 = document.createElement('input'); c3.id = 'lC3'; c3.setAttribute('form', 'lF1');\
             f1.appendChild(c3);\
             var lb5 = document.createElement('label'); lb5.setAttribute('for', 'lC3');\
             document.body.appendChild(lb5);\
             globalThis.__l3a = String(lb5.form === f1);\
             var lb6 = document.createElement('label'); lb6.setAttribute('for', 'nope');\
             document.body.appendChild(lb6);\
             globalThis.__l3b = String(lb6.form === null);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__l3a").unwrap().value,
        "true",
        "W3：for= 控件带 form 属性 → form attr 关联（委托 R4005）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__l3b").unwrap().value,
        "true",
        "W3：for= miss → null"
    );
}
