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

    // spec 状态机保持（R4200 起请求态语义）：已 open 的 modal dialog 再 showModal
    // no-op（「已在请求态」——WPT dialog-no-throw-requested-state 尾行断言面）；已
    // open 的 modal dialog 再 show 抛 InvalidStateError（show 状态机面不受反链遍历影响）。
    assert_eq!(
        sandbox
            .execute(
                r#"(function () {
  var dlg = document.querySelector('#mdn-search').shadowRoot.querySelector('dialog');
  var out = '';
  try { dlg.showModal(); out += 'no-throw'; } catch (e) { out += e.name; }
  try { dlg.show(); out += '|no-throw'; } catch (e) { out += '|' + e.name; }
  return out;
})()"#
            )
            .unwrap()
            .value,
        "no-throw|InvalidStateError",
        "R5181：modal open 上 showModal no-op（请求态）；show 抛 InvalidStateError（状态机保持）"
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

// R4008（html-semantics M2 切片 19）：label 激活行为 target 本体面——click 落
// label 自身（非 label.click() API）同样转发 labeled control。决策点：
// ① 交互豁免改「target 即本站 label 本体时不豁免」（spec 交互豁免只覆盖
// interactive content **后代**为 target；bubble 站挂钩 + R155 post 段同判）；
// ② shim 不承接宿主 generic Event 的转发——renderer 真实输入路径由 Rust
// 默认动作层激活（activate_label_at），webview plan 路径（testdriver）由
// user_actions label 链激活；shim 再转发会对 renderer 路径双激活（W5 钉）。
// WPT proxy-modifier-click-to-associated-element 4F 根因（plan 层链补齐）。
#[test]
fn test_label_self_target_dispatch_r4008() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    // 面板直接入 dom_html 快照（解析形态——与真实页面同构；`__zw_get_tag` 等
    // sel 族宿主回调解析快照，JS createElement 面板只入 mutations 队列不进快照，
    // 宿主 sel 形态派发解析不到）。for 关联 label（控件独立 button，排除子树
    // 查找耦合）+ 交互后代 + 嵌套 label（内层 for 内层控件）。
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new(
        "<html><body>\
         <label id='sLa' for='sBtnA'>text</label><button id='sBtnA'></button>\
         <label id='sLb' for='sBtnB'><a id='sA2' href='#x'>link</a></label><button id='sBtnB'></button>\
         <label id='sLo' for='sBtnO'><label id='sLi' for='sBtnI'>in</label></label>\
         <button id='sBtnO'></button><button id='sBtnI'></button>\
         </body></html>"
            .to_string(),
    ));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 计数器 + 相位日志 + 派发助手（页面 proxy dispatchEvent 形态）。
    sandbox
        .execute(
            "globalThis.__clk8 = {};\
             globalThis.__log8 = []; globalThis.__phase8 = 'init';\
             ['sBtnA','sBtnB','sBtnO','sBtnI'].forEach(function (id) {\
               globalThis.__clk8[id] = 0;\
               document.getElementById(id).addEventListener('click', function () { globalThis.__clk8[id]++; globalThis.__log8.push(id + '@' + globalThis.__phase8); });\
             });\
             globalThis.__fire8 = function (id, bubbles) {\
               globalThis.__phase8 = id + (bubbles ? ':bub' : ':nobub');\
               document.getElementById(id).dispatchEvent(new MouseEvent('click', { bubbles: !!bubbles }));\
               globalThis.__phase8 = 'post';\
             };",
        )
        .unwrap();

    // W1：冒泡 click 落 label 本体 → 转发（修复前 0）；恰 1 次不多计。
    sandbox
        .execute("globalThis.__fire8('sLa', true); globalThis.__w1 = globalThis.__clk8.sBtnA;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w1").unwrap().value,
        "1",
        "W1：冒泡 click 落 label 本体 → for 控件恰转发 1 次（修复前 0）"
    );

    // W2：非冒泡 click 落 label 本体 → 仅 R155 post 段转发（修复前 0；双闸防
    // 与挂钩双转）。
    sandbox
        .execute("globalThis.__fire8('sLa', false); globalThis.__w2 = globalThis.__clk8.sBtnA;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w2").unwrap().value,
        "2",
        "W2：非冒泡 click 落 label 本体 → post 段转发（累计 2，无双转）"
    );

    // W3：label 含交互后代 a[href]——click 落 label 本体照常转发（豁免只针对
    // 后代 target）；click 落 a[href] 后代仍豁免（相位日志钉针：sBtnB 仅 sLb:bub
    // 相位一次转发）。
    sandbox
        .execute("globalThis.__fire8('sLb', true); globalThis.__w3a = globalThis.__clk8.sBtnB;\
                  globalThis.__fire8('sA2', true); globalThis.__w3b = globalThis.__clk8.sBtnB;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w3a + ',' + globalThis.__w3b + ' log=' + globalThis.__log8.join(';')").unwrap().value,
        "1,1 log=sBtnA@sLa:bub;sBtnA@sLa:nobub;sBtnB@sLb:bub",
        "W3：label 本体转发 1 次（sLb:bub）；a[href] 后代豁免（sBtnB 计数不变仍为 1）"
    );

    // W4：嵌套 label——click 落内层 label 本体：内层转发 1；外层站按「内层 label
    // 是交互后代」豁免，外层控件不转发（事件标记 __zwLblFwd4004 防双转）。
    sandbox
        .execute("globalThis.__fire8('sLi', true); globalThis.__w4 = globalThis.__clk8.sBtnI + ',' + globalThis.__clk8.sBtnO;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w4").unwrap().value,
        "1,0",
        "W4：嵌套 label 内层本体转发 1 次；外层按交互后代豁免不转发"
    );

    // W5：宿主派发路径不分流——`__zw_dispatch_event`（generic Event，非
    // MouseEvent）落 label 本体，shim 不转发（renderer 默认动作层与 plan 层
    // label 链各自拥有宿主激活；shim 再转发 = renderer 真实点击双翻）。
    // 计数维持 W1+W2 的 2。
    sandbox
        .execute("globalThis.__phase8 = 'host';\
                  __zw_dispatch_event('#sLa', 'click', null);\
                  globalThis.__w5 = globalThis.__clk8.sBtnA;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w5").unwrap().value,
        "2",
        "W5：宿主 generic click 落 label 本体 → shim 不转发（激活归宿主层）"
    );

    // W6：宿主 generic click 落交互后代 a[href] 同样不转发；且宿主派发后 JS
    // MouseEvent 面仍正常（R155 post 段路径不受影响）。
    sandbox
        .execute("__zw_dispatch_event('#sA2', 'click', null);\
                  globalThis.__fire8('sLb', false);\
                  globalThis.__w6 = globalThis.__clk8.sBtnB;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__w6").unwrap().value,
        "2",
        "W6：宿主 click 落交互后代不转发（仍 1）；后续 JS 非冒泡 click 照常转发（累计 2）"
    );
}

// R4009（html-semantics M2 切片 20）：details/summary 激活深层面三件套——
// ① toggle 事件任务队列（spec binding-details「queue an element task」：同任务
//    多次开合合并单事件、oldState = 首个真变迁前 / newState = 任务运行时当下、
//    同态写不排程、detached 也派发；trusted + ToggleEvent.prototype 身份）；
// ② summary 激活行为（spec the-summary-element activation behaviour：首个
//    summary 元素子翻转父 details——summary.click() API 面 + bubble 站后代
//    target 面 + 交互豁免 a[href]，无 href a 非交互 → 翻转且不导航）；
// ③ 宿主 generic click 不分流（同 R4008 W5 语义：宿主激活归 plan/renderer
//    Rust 层，shim 面只承接 JS synthetic faces）。
// WPT 语料：activation-behavior.html 7F / anchor-without-link.html 2F /
// toggleEvent.html 簇 / details-keyboard-activation.html（plan 空格路由面）。
#[test]
fn test_details_summary_toggle_r4009() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new(
        "<html><body>\
         <details id='dA'><summary id='smA'>S</summary><p>C</p></details>\
         <details id='dB'><summary>S1</summary><summary id='smB2'>S2</summary><p>C</p></details>\
         <details id='dC'><span><summary id='smC'>S</summary></span><p>C</p></details>\
         <details id='dD'><span></span><summary id='smD'>S</summary><p>C</p></details>\
         <details id='dE' open><summary id='smE'>S</summary></details>\
         <details id='dF'><summary id='smF'><a id='ahF' href='#frag'>L</a></summary></details>\
         <details id='dG'><summary id='smG'><a id='anG'>plain</a><i id='inG'>it</i></summary></details>\
         <details id='dH'><summary id='smH'>S</summary></details>\
         <details id='dI'><summary id='smI'>S</summary></details>\
         <details id='dJ'><summary id='smJ'>S</summary></details>\
         </body></html>"
            .to_string(),
    ));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // toggle 描述符收集器（old>new/t/b/c/p 五断言位一次拼串）。
    sandbox
        .execute(
            "globalThis.__tg = {};\
             ['dA','dE','dH','dJ'].forEach(function (id) {\
               globalThis.__tg[id] = [];\
               document.getElementById(id).addEventListener('toggle', function (ev) {\
                 globalThis.__tg[id].push(ev.oldState + '>' + ev.newState + '/t' + ev.isTrusted + '/b' + ev.bubbles\
                   + '/c' + ev.cancelable + '/p' + (Object.getPrototypeOf(ev) === ToggleEvent.prototype));\
               });\
             });\
             globalThis.__tgDet = [];",
        )
        .unwrap();

    // T1：summary.click() 同步翻转 presence（activation-behavior 语料同步断言面）
    // + toggle 异步恰一次（trusted / 非 bubble / 不可取消 / ToggleEvent 原型）。
    sandbox
        .execute("document.getElementById('smA').click(); globalThis.__pA = String(document.getElementById('dA').open);")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__pA").unwrap().value,
        "true",
        "T1：summary.click() → details.open 同步翻转"
    );
    assert_eq!(
        sandbox.execute("globalThis.__tg.dA.join('|')").unwrap().value,
        "closed>open/ttrue/bfalse/cfalse/ptrue",
        "T1：toggle 恰一次——oldState/newState + trusted + 非冒泡 + 不可取消 + ToggleEvent 原型"
    );

    // T2：同任务多次开合合并单 toggle（coalescing——activation-behavior
    // toggle-tester 语料的同步 click 面同形；3 次同步 click = 3 次真变迁、1 事件、
    // oldState 取首变迁前）。语料的 microtask 续 click 合并面依赖 runner 宿主
    // macrotask timer（`__zw_test_setTimeout`）序——promise microtask 先于 timer
    // task 合并进同一窗口；本沙箱无 host timer，setTimeout 落 `_defer` microtask
    // fallback，两种通道同队不构成浏览器序，故此处只钉同步面。
    sandbox
        .execute(
            "var smH = document.getElementById('smH');\
             smH.click(); smH.click(); smH.click();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(document.getElementById('dH').open)").unwrap().value,
        "true",
        "T2：三次 click 后 presence open"
    );
    assert_eq!(
        sandbox.execute("globalThis.__tg.dH.join('|')").unwrap().value,
        "closed>open/ttrue/bfalse/cfalse/ptrue",
        "T2：同任务三次开合合并单 toggle（closed→open，oldState 取首变迁前）"
    );

    // T3：净零变迁（closed→…→closed）仍派发恰一次（toggleEvent.html t2 面）。
    sandbox
        .execute(
            "var dJ = document.getElementById('dJ');\
             dJ.open = true; dJ.open = false;\
             globalThis.__pJ = String(dJ.open);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__pJ + '|' + globalThis.__tg.dJ.join('|')").unwrap().value,
        "false|closed>closed/ttrue/bfalse/cfalse/ptrue",
        "T3：净零变迁派发恰一次 toggle（closed→closed）"
    );

    // T4：同态写不排程（toggleEvent.html t9/t10 面）——flush 后 dJ 条目数不变。
    sandbox
        .execute(
            "var dJ = document.getElementById('dJ');\
             dJ.open = false; dJ.removeAttribute('open');\
             globalThis.__n4 = globalThis.__tg.dJ.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__tg.dJ.length)").unwrap().value,
        "1",
        "T4：closed details 同态写（IDL + removeAttribute）不派 toggle"
    );

    // T5：setAttribute('open','') 面派 toggle（toggleEvent.html t7 面）；同态
    // setAttribute('open') 于已 open 元素 no-op（t9 面第二半）。
    sandbox
        .execute(
            "document.getElementById('dJ').setAttribute('open', '');\
             document.getElementById('dE').setAttribute('open', '');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__tg.dJ.join('|')").unwrap().value,
        "closed>closed/ttrue/bfalse/cfalse/ptrue|closed>open/ttrue/bfalse/cfalse/ptrue",
        "T5：setAttribute('open','') 新任务派一次（旧态接续 closed→open）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tg.dE.length)").unwrap().value,
        "0",
        "T5：open details 上 setAttribute('open') 同态不派"
    );
    // T5b：removeAttribute('open') 面回 closed（t8 面第二半；独立任务 → 第二事件
    // open→closed。同任务 set+remove 双写合并单事件面已由 T3 钉）。
    sandbox
        .execute("document.getElementById('dJ').removeAttribute('open');")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__tg.dJ.join('|')").unwrap().value,
        "closed>closed/ttrue/bfalse/cfalse/ptrue|closed>open/ttrue/bfalse/cfalse/ptrue|open>closed/ttrue/bfalse/cfalse/ptrue",
        "T5b：removeAttribute('open') 派 open→closed"
    );

    // T6：activation element 门——第二 summary / span 内 summary 不翻转；span
    // 前置元素不挡首个 summary 元素子（activation-behavior has-* 语料四面）。
    sandbox
        .execute(
            "document.getElementById('smB2').click();\
             document.getElementById('smC').click();\
             document.getElementById('smD').click();\
             globalThis.__p6 = [document.getElementById('dB').open, document.getElementById('dC').open, document.getElementById('dD').open].join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__p6").unwrap().value,
        "false,false,true",
        "T6：非首个 summary 元素子 no-op；span 前置不挡、span 内嵌 summary no-op"
    );

    // T7：anchor-without-link——无 href a 及其后代 i click() → details 开且
    // location.hash 不动；有 href a 后代豁免（details 保持 closed）。
    sandbox
        .execute(
            "globalThis.__h0 = String(location.hash);\
             document.getElementById('inG').click();\
             globalThis.__pG1 = String(document.getElementById('dG').open);\
             globalThis.__h1 = String(location.hash);\
             document.getElementById('ahF').click();\
             globalThis.__pF = String(document.getElementById('dF').open);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__h0 + '|' + globalThis.__pG1 + '|' + globalThis.__h1 + '|' + globalThis.__pF").unwrap().value,
        "|true||false",
        "T7：无 href a 后代 click 开 details 且 hash 不动；a[href] 后代豁免不翻转"
    );

    // T8：宿主 generic click 落 summary 本体 → shim 不翻转（宿主激活归 plan/
    // renderer Rust 层——R4008 W5 同款架构钉）。
    sandbox
        .execute("__zw_dispatch_event('#smI', 'click', null); globalThis.__pI = String(document.getElementById('dI').open);")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__pI").unwrap().value,
        "false",
        "T8：宿主 generic click 落 summary → shim 不翻转（宿主层拥有激活）"
    );

    // T9：detached details（createElement）同样排程 toggle（toggleEvent.html
    // details5 面）。
    sandbox
        .execute(
            "globalThis.__dd = document.createElement('details');\
             globalThis.__dd.addEventListener('toggle', function (ev) {\
               globalThis.__tgDet.push(ev.oldState + '>' + ev.newState + '/t' + ev.isTrusted\
                 + '/p' + (Object.getPrototypeOf(ev) === ToggleEvent.prototype));\
             });\
             globalThis.__dd.open = true;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__tgDet.join('|')").unwrap().value,
        "closed>open/ttrue/ptrue",
        "T9：detached details 的 open 写照派 toggle（trusted + ToggleEvent 原型）"
    );
}

// R4100（html-semantics M2 切片 21）：details name= 互斥簇（exclusive accordions）——
// ① parser/markup 构造面（innerHTML 产物）：树序处理——既存 open 成员胜出、
//    新解析 open 元素让位（given-prong），parser-open 照排程 toggle（closed→open，
//    随后互斥关闭经 R4009 队列合并 net-zero 单事件）；
// ② open 写入面（IDL setter / setAttribute）：真开变迁关组内全部其他 open
//    （others-prong）；
// ③ name 变更面（IDL / setAttribute）：subject open 且组内已有 open → 自身让位
//    （given-prong）；
// ④ 移动插入面（appendChild/insertBefore）：新插入 open 元素让位既存 open 成员；
// ⑤ 空/缺失 name 不成组；detached（未连接）照常 enforcement。
// 组口径：同树根 + 同名非空 name + DETAILS。WPT 语料：name-attribute.html
// （basic/more-complex/tree-scopes/mutation-order/name-changes/parser-order/
//  insertion/disconnected 全场景）。
#[test]
fn test_details_name_exclusivity_r4100() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new(
        "<html><body><div id='ct4100'></div><div id='ct4100b'></div><div id='ct4100c'></div>\
         <div id='q4100'></div></body></html>"
            .to_string(),
    ));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // T1：markup 构造面（innerHTML 解析产物）——树序第一 open 胜出、其余让位；
    // parser-open toggle 排程（x1 的 open→闭合并 net-zero 单事件）。
    sandbox
        .execute(
            "var ct = document.getElementById('ct4100');\
             ct.innerHTML = '<details name=\"a\" id=\"x0\" open></details>'\
               + '<details name=\"a\" id=\"x1\" open></details>'\
               + '<details name=\"b\" id=\"x2\" open></details>';\
             globalThis.__t4100 = { x0: [], x1: [], x2: [] };\
             ['x0', 'x1', 'x2'].forEach(function (id) {\
               document.getElementById(id).addEventListener('toggle', function (ev) {\
                 globalThis.__t4100[id].push(ev.oldState + '>' + ev.newState);\
               });\
             });\
             globalThis.__s1 = ['x0', 'x1', 'x2'].map(function (id) {\
               return Number(document.getElementById(id).open); }).join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s1").unwrap().value,
        "1,0,1",
        "T1：innerHTML 解析产物互斥——组 a 树序第一 open 胜出（x1 让位），组 b 独立"
    );
    assert_eq!(
        sandbox.execute("globalThis.__t4100.x0.join('|') + '#' + globalThis.__t4100.x1.join('|') + '#' + globalThis.__t4100.x2.join('|')").unwrap().value,
        "closed>open#closed>closed#closed>open",
        "T1：parser-open 照排程 toggle（x0/x2 closed→open）；x1 解析后互斥关闭合并 net-zero 单事件"
    );

    // T2：open IDL setter close-others——x1 真开 → x0 让位（others-prong）。
    sandbox
        .execute("document.getElementById('x1').open = true; \
                  globalThis.__s2 = Number(document.getElementById('x0').open) + ',' + Number(document.getElementById('x1').open);")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s2").unwrap().value,
        "0,1",
        "T2：open IDL 真开 → 组内既存 open 成员让位"
    );

    // T3：setAttribute('open', '') 同面（x0 重开 → x1 让位）。
    sandbox
        .execute("document.getElementById('x0').setAttribute('open', ''); \
                  globalThis.__s3 = Number(document.getElementById('x0').open) + ',' + Number(document.getElementById('x1').open);")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s3").unwrap().value,
        "1,0",
        "T3：setAttribute('open','') 真开面同 close-others"
    );

    // T4：name IDL 变更面 given-prong——跨组并入即让位；重开走 close-others；
    // setAttribute('name') 同口径。
    sandbox
        .execute(
            "var ctb = document.getElementById('ct4100b');\
             ctb.innerHTML = '<details name=\"a\" id=\"y0\" open></details>'\
               + '<details name=\"b\" id=\"y1\" open></details>';\
             var y1 = document.getElementById('y1');\
             y1.name = 'a';\
             globalThis.__s4 = Number(document.getElementById('y0').open) + ',' + Number(y1.open);\
             y1.setAttribute('name', 'b'); y1.open = true;\
             globalThis.__s5 = Number(document.getElementById('y0').open) + ',' + Number(y1.open);\
             y1.setAttribute('name', 'a');\
             globalThis.__s6 = Number(document.getElementById('y0').open) + ',' + Number(y1.open);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s4 + ' ' + globalThis.__s5 + ' ' + globalThis.__s6").unwrap().value,
        "1,0 1,1 1,0",
        "T4：name 变更并入既有 open 组 → 自身让位（IDL 与 setAttribute 同口径）；独组（b）重开不跨组影响组 a"
    );

    // T5：移动插入面——detached 开元素 append 入组让位；写面与插入面两次真变迁
    // 同任务（无 await 让位），spec toggle task queue 合并单 net-zero 事件
    //（oldState 取首变迁前 closed / newState 取任务运行时当下 closed）。
    sandbox
        .execute(
            "var ctc = document.getElementById('ct4100c');\
             ctc.innerHTML = '<details name=\"a\" id=\"z0\" open></details>';\
             var z1 = document.createElement('details'); z1.id = 'z1'; z1.name = 'a'; z1.open = true;\
             globalThis.__t5z1 = [];\
             z1.addEventListener('toggle', function (ev) { globalThis.__t5z1.push(ev.oldState + '>' + ev.newState); });\
             ctc.appendChild(z1);\
             globalThis.__s7 = Number(document.getElementById('z0').open) + ',' + Number(z1.open);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s7 + '#' + globalThis.__t5z1.join('|')").unwrap().value,
        "1,0#closed>closed",
        "T5：appendChild 开元素入组让位——同任务开+关合并单 net-zero toggle（spec 口径）"
    );

    // T6：insertBefore 首位插入开元素同样让位（既存 open 成员不因树序变前而让）。
    sandbox
        .execute(
            "var z2 = document.createElement('details'); z2.id = 'z2'; z2.name = 'a'; z2.open = true;\
             var ctc = document.getElementById('ct4100c');\
             ctc.insertBefore(z2, document.getElementById('z0'));\
             globalThis.__s8 = Number(z2.open) + ',' + Number(document.getElementById('z0').open);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s8").unwrap().value,
        "0,1",
        "T6：insertBefore 首位开元素让位——given-prong 关新插入者而非既存成员"
    );

    // T7：空/缺失 name 不成组（各自保持 open）；detached 容器组照常 enforcement。
    sandbox
        .execute(
            "var ct = document.getElementById('ct4100');\
             ct.innerHTML = '<details id=\"n0\" open></details><details id=\"n1\" open></details>'\
               + '<details name=\"\" id=\"n2\" open></details>';\
             globalThis.__s9 = ['n0', 'n1', 'n2'].map(function (id) {\
               return Number(document.getElementById(id).open); }).join(',');\
             var dq = document.createElement('div');\
             dq.innerHTML = '<details name=\"a\" id=\"q0\" open></details><details name=\"a\" id=\"q1\"></details>';\
             globalThis.__s10 = Number(dq.firstElementChild.open) + ',' + Number(dq.lastElementChild.open);\
             dq.lastElementChild.open = true;\
             globalThis.__s11 = Number(dq.firstElementChild.open) + ',' + Number(dq.lastElementChild.open);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s9 + ' ' + globalThis.__s10 + ' ' + globalThis.__s11").unwrap().value,
        "1,1,1 1,0 0,1",
        "T7：空/缺失 name 不成组；detached 容器组照常互斥（given/others 双 prong）"
    );

    // T8：name IDL 反射 round-trip + 原型成员存在性（`'name' in
    // HTMLDetailsElement.prototype`——name-content-attribute-and-property 自适配判定面）。
    sandbox
        .execute(
            "var d = document.createElement('details'); d.name = 'grp';\
             globalThis.__s12 = d.getAttribute('name') + '#' + d.name\
               + '#' + ('name' in globalThis.HTMLDetailsElement.prototype);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s12").unwrap().value,
        "grp#grp#true",
        "T8：details.name IDL 反射 round-trip + 原型成员存在性"
    );
}

// encoding-legacy-mb M3 甄别：template contents 深条目物化面（`_zwMBuildDeepEntry`，
// WC-M2 setAttribute 灌法）的 R4100 互斥——host `__zw_template_contents_deep` 产物
// 经 setAttribute 逐 attr 灌入，写尾 others-prong 在构造期触发的理论暴露（深臂
// `_zwMBuildDeepTreeNode` 同机制已改工厂 attrs 直灌修复）。页面 markup 模板（sel 形
// 视图）首读 `.content` 走该路径；期望与 markup 构造面（T1）同口径：树序第一
// open 胜出、后者让位。
#[test]
fn test_details_name_exclusivity_template_deep_entry() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new(
        "<html><body><div id='q4100'><template id='tq4100'>\
         <details name=\"a\" id=\"tp0\" open></details>\
         <details name=\"a\" id=\"tp1\" open></details>\
         </template></div></body></html>"
            .to_string(),
    ));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var tq = document.getElementById('tq4100');\
             var ds = tq.content.querySelectorAll('details');\
             globalThis.__s = ds.length + ':' + Number(ds[0].open) + ',' + Number(ds[1].open);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s").unwrap().value,
        "2:1,0",
        "template contents 物化互斥——同 T1 markup 构造面口径：树序第一 open 胜出"
    );

    // T2：createContextualFragment 面——同款深条目物化（`_zwMBuildDeepEntry`）。修复前
    // setAttribute 灌法在 frag.childNodes 渐进填充下 others-prong 后者关前者（0,1 反转）。
    sandbox
        .execute(
            "var rg = document.createRange();\
             rg.selectNodeContents(document.body);\
             var cf = rg.createContextualFragment('<details name=\"b\" open></details><details name=\"b\" open></details>');\
             globalThis.__c = cf.childNodes.length + ':' + Number(cf.childNodes[0].open) + ',' + Number(cf.childNodes[1].open);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__c").unwrap().value,
        "2:1,0",
        "createContextualFragment 物化互斥——树序第一 open 胜出（修复前 0,1 反转）"
    );
}

// R4200（html-semantics M2 切片 22）：dialog close-side 语义簇契约——requestClose
//（cancel 同步可取消 + close 异步排队 + 重入保护 + disconnected no-op）、close-the-dialog
//（beforetoggle 同步 + close/toggle 事件排队）、show/showModal 请求态语义、`:modal`
// 伪类、Promise.withResolvers、close(rv) 不触发 JS 定义的 returnValue setter。
// 对齐 WPT dialog-requestclose / dialog-close / dialog-close-event-async /
// toggle-events / dialog-no-throw-requested-state 断言面。元素用 createElement+
// appendChild 构造（handle 路径 tag 经 pending CreateElement 记录可得——innerHTML
// 物化元素 pre-apply 无 host tag，`_realTag` DIALOG 门不命中），跨 execute 经
// globalThis.__dN 引用。
#[test]
fn test_dialog_close_side_r4200() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // T1：requestClose 基本面——open 属性同步移除；cancel 同步派发、close 排队
    // 异步（execute 间排空后到位）。对齐 dialog-requestclose「fires both cancel and
    // close」+ dialog-close-event-async「close is scheduled」。
    sandbox
        .execute(
            "globalThis.__d = [];             for (var _i = 0; _i < 7; _i++) {               var _el = document.createElement('dialog');               document.body.appendChild(_el);               globalThis.__d.push(_el);             }             var d = globalThis.__d[0];             globalThis.__ev = [];             d.addEventListener('cancel', function () { globalThis.__ev.push('cancel'); });             d.addEventListener('close', function () { globalThis.__ev.push('close'); });             d.show();             d.requestClose();             globalThis.__s1 = Number(d.open) + ',' + globalThis.__ev.join('|');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s1").unwrap().value,
        "0,cancel",
        "T1：requestClose 同步移 open + cancel 同步；close 尚未派发（排队面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__ev.join('|')").unwrap().value,
        "cancel|close",
        "T1：execute 间排空后 close 到位——事件序 cancel→close"
    );

    // T2：cancel preventDefault → 保持 open、returnValue 不变；第二次（不阻止）
    // 照关 + returnValue 透传。对齐「requestClose can be cancelled」+「passes along
    // the return value」。
    sandbox
        .execute(
            "var d2 = globalThis.__d[1];             d2.show();             d2.returnValue = 'keep';             var once2 = true;             d2.addEventListener('cancel', function (e) { if (once2) { once2 = false; e.preventDefault(); } });             d2.requestClose('nope');             globalThis.__s2 = Number(d2.open) + ',' + d2.returnValue;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s2").unwrap().value,
        "1,keep",
        "T2：cancel 被阻止 → open 保持 + returnValue 不被改写"
    );
    sandbox
        .execute(
            "var d2 = globalThis.__d[1];             d2.requestClose('done');             globalThis.__s2b = Number(d2.open) + ',' + d2.returnValue;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s2b").unwrap().value,
        "0,done",
        "T2：未阻止的 requestClose(rv) → 关闭 + returnValue 透传"
    );

    // T3：close() 同步面（属性移除 + returnValue 存储）与异步 close 事件解耦；
    // isTrusted/非 bubble/不可取消；未 open 的 close() 不存值（R3910 面）。
    sandbox
        .execute(
            "var d3 = globalThis.__d[2];             globalThis.__cc = 0;             d3.addEventListener('close', function (e) {               globalThis.__cc++;               globalThis.__evinfo = e.isTrusted + ',' + e.bubbles + ',' + e.cancelable;             });             d3.show();             d3.close('rv3');             globalThis.__s3 = Number(d3.open) + ',' + globalThis.__cc + ',' + d3.returnValue;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s3").unwrap().value,
        "0,0,rv3",
        "T3：close() 属性/returnValue 同步生效、close 事件未同步派发"
    );
    assert_eq!(
        sandbox.execute("globalThis.__cc + ',' + globalThis.__evinfo").unwrap().value,
        "1,true,false,false",
        "T3：close 事件排队到达，trusted、不 bubble、不可取消"
    );
    sandbox
        .execute(
            "var d3 = globalThis.__d[2];             d3.close('late');             globalThis.__s3c = d3.returnValue + ',' + globalThis.__cc;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s3c").unwrap().value,
        "rv3,1",
        "T3：未 open 的 close() 不改 returnValue、不派事件（R3910）"
    );

    // T4：beforetoggle 同步（open 属性写入前）、toggle 异步；同任务 open+close 合并
    // 单事件 net-zero（closed>closed）。对齐 toggle-events「fire beforetoggle and
    // toggle」+「coalesce asynchronous toggle events」。
    sandbox
        .execute(
            "var d4 = globalThis.__d[3];             globalThis.__bt = [];             globalThis.__tg = [];             d4.addEventListener('beforetoggle', function (e) {               globalThis.__bt.push(e.oldState + '>' + e.newState + ':' + Number(d4.hasAttribute('open')));             });             d4.addEventListener('toggle', function (e) { globalThis.__tg.push(e.oldState + '>' + e.newState); });             d4.show();             globalThis.__s4 = globalThis.__bt.join('|') + '#' + globalThis.__tg.length + ',' + Number(d4.open);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s4").unwrap().value,
        "closed>open:0#0,1",
        "T4：show beforetoggle 同步且先于 open 属性写入；toggle 异步未到"
    );
    sandbox
        .execute("var d4 = globalThis.__d[3]; d4.close();")
        .unwrap();
    sandbox
        .execute("var d4 = globalThis.__d[3]; d4.show(); d4.close();")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__tg.join('|')").unwrap().value,
        "closed>open|open>closed|closed>closed",
        "T4：toggle 逐任务到位；同任务 open+close 合并 net-zero 单事件"
    );

    // T5：showModal 请求态语义——show 后 showModal 抛 InvalidStateError；showModal
    // 后 showModal no-op、show 抛；`:modal` 命中面（modal 开 true / show 开 false）。
    // 对齐 dialog-no-throw-requested-state + dialog-requestclose openDialog :modal 断言。
    sandbox
        .execute(
            "var d5 = globalThis.__d[4];             d5.show();             var e1 = '';             try { d5.showModal(); } catch (err) { e1 = err.name; }             globalThis.__s5a = e1 + ',' + Number(d5.open) + ',' + d5.matches(':modal');             d5.close();             d5.showModal();             var e2 = '';             try { d5.show(); } catch (err) { e2 = err.name; }             var e3 = 'noop';             try { d5.showModal(); } catch (err) { e3 = err.name; }             var e4 = 'noop';             try { d5.show(); } catch (err) { e4 = err.name; }             globalThis.__s5b = Number(d5.open) + ',' + d5.matches(':modal') + ',' + e2 + ',' + e3 + ',' + e4;             d5.close();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s5a").unwrap().value,
        "InvalidStateError,1,false",
        "T5：非模态 open 上 showModal 抛 InvalidStateError；show 打开不命中 :modal"
    );
    assert_eq!(
        sandbox.execute("globalThis.__s5b").unwrap().value,
        "1,true,InvalidStateError,noop,InvalidStateError",
        "T5：modal 上 :modal 命中；show 抛；showModal/show 请求态 no-op 语义（show 于 modal 抛）"
    );

    // T6：disconnected requestClose no-op（close watcher 非活跃）。对齐
    // dialog-requestclose-2。
    sandbox
        .execute(
            "var d6 = document.createElement('dialog');             d6.setAttribute('open', '');             d6.requestClose();             globalThis.__s6 = Number(d6.open);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s6").unwrap().value,
        "1",
        "T6：未连接 dialog requestClose 不关闭"
    );

    // T7：cancel 处理器内重入 requestClose → 不二次 cancel，照常关闭。对齐
    // dialog-requestclose-recurse。
    sandbox
        .execute(
            "var d7 = globalThis.__d[5];             d7.show();             var cc7 = 0;             d7.addEventListener('cancel', function () {               cc7++;               if (cc7 === 1) d7.requestClose();             });             d7.requestClose();             globalThis.__s7 = cc7 + ',' + Number(d7.open);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s7").unwrap().value,
        "1,0",
        "T7：cancel 恰派发一次（重入保护）+ close-the-dialog 照常执行"
    );

    // T8：Promise.withResolvers 可用；close(rv) 存 IDL 值但不触发页面 JS 定义的
    // returnValue setter。对齐 dialog-close「set the returnValue IDL attribute but
    // not the JS property」。
    sandbox
        .execute(
            "globalThis.__wr = typeof Promise.withResolvers === 'function';             var d8 = globalThis.__d[6];             globalThis.__jsSet = false;             Object.defineProperty(d8, 'returnValue', {               set: function () { globalThis.__jsSet = true; },               get: function () { return ''; },               configurable: true             });             d8.show();             d8.close('foo');             globalThis.__s8 = globalThis.__wr + ',' + globalThis.__jsSet + ',' + (d8.returnValue === 'foo');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s8").unwrap().value,
        "true,false,true",
        "T8：withResolvers 已注入；close(rv) 不走 JS setter 且 IDL 读回 'foo'"
    );
}

/// R4300（M2 切片 23，2026-10-22）/ R5263（M2 切片 56，2026-10-24）：dialog
/// closedby 面——closedBy IDL 反射（枚举小写化 + auto 派生 + setter）、ESC close
/// request 资格门（`__zw_esc_dialog_cancel` closedby 门 + 模态栈序）、light
/// dismiss 两腿（`__zw_dialog_light_dismiss_check` down 记分类 +
/// `__zw_dialog_light_dismiss_up` sameTarget 同态比对后关；down 'out' + up 'in'
/// 即 capture 重定向不关门面）。
/// 驱动路径：runner send_keys Escape 臂 + click 命令前步 / 产品指针管线
/// pointerdown/pointerup 钩。
#[test]
fn test_dialog_closedby_light_dismiss_r4300() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // R1：closedBy 反射——缺失 → auto 派生（非模态未开 'none'；modal 开 'closerequest'）；
    // 大小写不敏感小写返回；非法值 → auto；显式 'none' 原样；setter 反射内容属性、
    // null → 移除（回落 auto）。对齐 dialog-closedby getDefaultExpectations 断言面。
    sandbox
        .execute(
            "globalThis.__ds = [];             for (var _i = 0; _i < 6; _i++) {               var _el = document.createElement('dialog');               document.body.appendChild(_el);               globalThis.__ds.push(_el);             }             var a = globalThis.__ds[0];             var b = globalThis.__ds[1];             var c = globalThis.__ds[2];             var e = globalThis.__ds[4];             b.setAttribute('closedby', 'ANY');             c.setAttribute('closedby', 'bogus');             e.setAttribute('closedby', 'none');             globalThis.__r1a = a.closedBy;             globalThis.__r1b = b.closedBy;             globalThis.__r1c = c.closedBy;             globalThis.__r1e = e.closedBy;             a.showModal();             globalThis.__r1a2 = a.closedBy;             a.close();             globalThis.__r1a3 = a.closedBy;             var f = globalThis.__ds[5];             f.closedBy = 'closerequest';             globalThis.__r1f = f.getAttribute('closedby') + ',' + f.closedBy;             f.closedBy = null;             globalThis.__r1f2 = String(f.hasAttribute('closedby')) + ',' + f.closedBy;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r1a + '|' + globalThis.__r1a2 + '|' + globalThis.__r1a3").unwrap().value,
        "none|closerequest|none",
        "R1：auto 派生——未开 none / modal 开 closerequest / 关后回 none"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r1b + '|' + globalThis.__r1c + '|' + globalThis.__r1e")
            .unwrap()
            .value,
        "any|none|none",
        "R1：'ANY' 小写化；'bogus' → auto（未开派生 none）；显式 none 原样"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r1f + '|' + globalThis.__r1f2").unwrap().value,
        "closerequest,closerequest|false,none",
        "R1：setter 反射 closedby 内容属性；null → 移除回落 auto"
    );

    // R2：ESC 资格门——非模态默认（auto→none）不关；closedby=none 模态不关；
    // closedby=any 非模态关。对齐 non-modal-canceling 双面 + dialog-closedby
    // respondsToEsc 矩阵。
    sandbox
        .execute(
            "var n1 = globalThis.__ds[0];             n1.show();             globalThis.__r2a = __zw_esc_dialog_cancel() + ',' + Number(n1.open);             var n2 = globalThis.__ds[4];             n2.showModal();             globalThis.__r2b = __zw_esc_dialog_cancel() + ',' + Number(n2.open);             var n3 = globalThis.__ds[1];             n3.closedBy = 'any';             n3.show();             globalThis.__r2c = __zw_esc_dialog_cancel() + ',' + Number(n3.open);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("globalThis.__r2a + '|' + globalThis.__r2b + '|' + globalThis.__r2c")
            .unwrap()
            .value,
        "false,1|false,1|true,0",
        "R2：auto 非模态 / none 模态不响应 ESC；any 非模态响应且关闭"
    );

    // R3：模态栈序——bottom/top 双 showModal，首次 ESC 关栈顶 top（非文档序首个），
    // 二次 ESC 关 bottom。cancel/close 事件序逐层对齐 dialog-canceling 期望序。
    sandbox
        .execute(
            "var bt = document.createElement('dialog');             var tp = document.createElement('dialog');             document.body.appendChild(bt);             document.body.appendChild(tp);             globalThis.__btD = bt; globalThis.__tpD = tp;             globalThis.__seq = [];             bt.addEventListener('cancel', function () { globalThis.__seq.push('cancel:bottom'); });             tp.addEventListener('cancel', function () { globalThis.__seq.push('cancel:top'); });             bt.addEventListener('close', function () { globalThis.__seq.push('close:bottom'); });             tp.addEventListener('close', function () { globalThis.__seq.push('close:top'); });             bt.showModal();             tp.showModal();             globalThis.__r3a = __zw_esc_dialog_cancel() + ',' + Number(tp.open) + ',' + Number(bt.open);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r3a").unwrap().value,
        "true,0,1",
        "R3：首次 ESC 命中栈顶 top（同步移 open），bottom 保持"
    );
    sandbox
        .execute(
            "globalThis.__r3b = __zw_esc_dialog_cancel() + ',' + Number(globalThis.__btD.open);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("globalThis.__r3b + '|' + globalThis.__seq.join('|')")
            .unwrap()
            .value,
        "true,0|cancel:top|close:top|cancel:bottom|close:bottom",
        "R3：二次 ESC 关 bottom；cancel/close 序逐层 top→bottom"
    );

    // R4：light dismiss 两腿（R5263）——closedby=any 模态开 + 落点在 dialog 内
    // （down+up 同 'in'）→ 保持；落点在 dialog 外（down+up 同 'out'）→ 关；
    // down 'out' + up 'in'（capture 把 up 重定向 dialog 内元素）→ sameTarget
    // 不等不关；默认模态（auto→closerequest）落点外不关（槽 null）。
    sandbox
        .execute(
            "var l1 = document.createElement('dialog');             l1.closedBy = 'any';             l1.innerHTML = '<button id=inner>b</button>';             document.body.appendChild(l1);             globalThis.__l1D = l1;             l1.showModal();             var l1b = l1.querySelector('button').__zwSelector || 'button';             __zw_dialog_light_dismiss_check(l1b);             __zw_dialog_light_dismiss_up(l1b);             globalThis.__r4a = Number(l1.open);             __zw_dialog_light_dismiss_check('body');             __zw_dialog_light_dismiss_up('body');             globalThis.__r4b = Number(l1.open);             var l2 = document.createElement('dialog');             document.body.appendChild(l2);             globalThis.__l2D = l2;             l2.showModal();             __zw_dialog_light_dismiss_check('body');             __zw_dialog_light_dismiss_up('body');             globalThis.__r4c = Number(l2.open);             var l3 = document.createElement('dialog');             l3.closedBy = 'any';             l3.innerHTML = '<button id=inner3>b3</button>';             document.body.appendChild(l3);             globalThis.__l3D = l3;             l3.showModal();             __zw_dialog_light_dismiss_check('body');             __zw_dialog_light_dismiss_up('#inner3');             globalThis.__r4d = Number(l3.open);",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("globalThis.__r4a + '|' + globalThis.__r4b + '|' + globalThis.__r4c + '|' + globalThis.__r4d")
            .unwrap()
            .value,
        "1|0|1|1",
        "R4：两腿 in/in 保持、out/out 关；默认模态（closerequest）槽 null 不关；out→in（capture 门）不关"
    );

    // R5：`__zw_dispatch_key_at_active`——keydown 派发到当前焦点元素（非 send_keys
    // 目标）；焦点元素 keydown preventDefault → 返 'prevented'（runner 据此跳过
    // dialog close request 默认动作——dialog-canceling swallow-input 面）。两面覆盖：
    // querySelector 产物（__zwSelector 直取）与 getElementById 产物（无 sel → id 属性
    // 选择器回落——语料 getElementById().focus() 真实形状）。
    sandbox
        .execute(
            "document.body.insertAdjacentHTML('beforeend', '<input id=kw5><input id=kw5b>');             globalThis.__kdT = '';             var mk = function (q) {               var inp = q === '#' ? document.querySelector('#kw5') : document.getElementById('kw5b');               inp.addEventListener('keydown', function (e) {                 globalThis.__kdT += e.key + '@' + inp.id + ';';                 e.preventDefault();               });               inp.focus();               return String(__zw_dispatch_key_at_active('keydown', '{\"key\":\"Escape\",\"code\":\"Escape\"}'));             };             globalThis.__r5a = mk('#') + ',' + mk('g');             globalThis.__r5b = globalThis.__kdT;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5a + '|' + globalThis.__r5b").unwrap().value,
        "prevented,prevented|Escape@kw5;Escape@kw5b;",
        "R5：keydown 落焦点元素（sel/id 双形态）且 preventDefault → 'prevented' 返标记"
    );
}

/// R4400（M2 切片 24，2026-10-22）：Invoker Commands API + ToggleEvent.source——
/// commandfor/command click 默认动作（show-modal/show/close/request-close → dialog、
/// toggle/show/hide → details、缺失/非法 → auto 派生）与 ToggleEvent source 三面：
/// invoker 触发 = 激活元素 proxy（beforetoggle 同步 + toggle 队列双事件一致）、直接
/// IDL/summary 点击/light dismiss = null、summary-less DETAILS 点击激活 + 有 summary
/// 时 details 直击不翻转。
#[test]
fn test_invoker_command_toggle_source_r4400() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // R1：dialog invoker——command=show-modal 按钮 click 开（beforetoggle 同步
    // source=invoker、toggle 队列 source=invoker、newState 'open'）；command=close
    // 按钮关（source 仍 invoker）。对齐 dialog-toggle-source 按钮面 +
    // toggle-event-source-test `event.source === invoker` 身份断言。
    sandbox
        .execute(
            "var d = document.createElement('dialog'); d.id = 'tgtD'; document.body.appendChild(d); var b = document.createElement('button'); b.setAttribute('commandfor', 'tgtD'); b.setAttribute('command', 'show-modal'); document.body.appendChild(b); globalThis.__eB = b; globalThis.__eD = d; globalThis.__r1 = []; d.addEventListener('beforetoggle', function (e) { globalThis.__r1.push('bt:' + (e.source === b)); }); d.addEventListener('toggle', function (e) { globalThis.__r1.push('tg:' + (e.source === b) + ',' + e.newState); }); b.click();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r1.join('|') + ',' + Number(globalThis.__eD.hasAttribute('open'))").unwrap().value,
        "bt:true|tg:true,open,1",
        "R1：show-modal 按钮——beforetoggle/toggle 双事件 source=invoker，open 生效"
    );
    sandbox
        .execute(
            "globalThis.__eB.setAttribute('command', 'close'); globalThis.__eB.click();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r1.slice(2).join('|') + ',' + Number(globalThis.__eD.hasAttribute('open'))").unwrap().value,
        "bt:true|tg:true,closed,0",
        "R1：close 按钮——close 侧双事件 source=invoker，open 移除"
    );

    // R2：直接 IDL 调用 source null（show/showModal/close 无 invoker 条目——store
    // take 语义不残留）。对齐 toggle-event-source-test 直接调用臂。
    sandbox
        .execute(
            "var d2 = globalThis.__eD; globalThis.__r2 = []; d2.addEventListener('beforetoggle', function (e) { globalThis.__r2.push('bt:' + (e.source === null)); }); d2.addEventListener('toggle', function (e) { globalThis.__r2.push('tg:' + (e.source === null)); }); d2.show();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r2.join('|')").unwrap().value,
        "bt:true|tg:true",
        "R2：直接 show()——双事件 source=null（非 undefined）"
    );

    // R3：details invoker——auto（缺省 command）toggle、show 已开无变迁无事件、hide
    // 关、非法值回落 auto toggle。对齐 details-toggle-source-commandfor tentative 面。
    // 变迁逐 execute 分步（R4009 同任务 toggle 合并——corpus helper await next
    // toggle 同形；同 execute 连发会合并单事件）。
    sandbox
        .execute(
            "var dt = document.createElement('details'); dt.id = 'tgtDet'; document.body.appendChild(dt); var b2 = document.createElement('button'); b2.setAttribute('commandfor', 'tgtDet'); document.body.appendChild(b2); globalThis.__eB2 = b2; globalThis.__eDet = dt; globalThis.__r3 = []; dt.addEventListener('toggle', function (e) { globalThis.__r3.push('tg:' + (e.source === b2) + ',' + e.newState); }); b2.click();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r3.join('|') + ',' + Number(globalThis.__eDet.hasAttribute('open'))").unwrap().value,
        "tg:true,open,1",
        "R3：缺省 command → auto → toggle 开，source=invoker"
    );
    sandbox
        .execute(
            "globalThis.__eB2.setAttribute('command', 'show'); globalThis.__eB2.click(); globalThis.__r3n = globalThis.__r3.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r3n + ',' + Number(globalThis.__eDet.hasAttribute('open'))").unwrap().value,
        "1,1",
        "R3：show 已开——无变迁、无新事件"
    );
    sandbox
        .execute(
            "globalThis.__eB2.setAttribute('command', 'hide'); globalThis.__eB2.click();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r3.join('|') + ',' + Number(globalThis.__eDet.hasAttribute('open'))").unwrap().value,
        "tg:true,open|tg:true,closed,0",
        "R3：hide 命令关——toggle source=invoker、presence 清除"
    );
    sandbox
        .execute(
            "globalThis.__eB2.setAttribute('command', 'bogus'); globalThis.__eB2.click();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r3.join('|') + ',' + Number(globalThis.__eDet.hasAttribute('open'))").unwrap().value,
        "tg:true,open|tg:true,closed|tg:true,open,1",
        "R3：非法 command → invalid value default auto → toggle 再开"
    );

    // R4：details 直接 open setter——toggle 照排、source 非 invoker（null 面）。
    sandbox
        .execute(
            "globalThis.__eDet.open = false;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r3[3] + ',' + Number(globalThis.__eDet.hasAttribute('open'))").unwrap().value,
        "tg:false,closed,0",
        "R4：直接 open=false——toggle source≠invoker（null），presence 清除"
    );

    // R5：summary-less DETAILS 点击激活（spec activation behaviour 无 summary 门）
    // + toggle source null；有 summary 的 details 直击不翻转（激活归 SUMMARY）。
    sandbox
        .execute(
            "var dts = document.createElement('details'); document.body.appendChild(dts); globalThis.__r5a = []; dts.addEventListener('toggle', function (e) { globalThis.__r5a.push(String(e.source === null)); }); dts.click(); globalThis.__r5s = Number(dts.hasAttribute('open')); var dtw = document.createElement('details'); dtw.innerHTML = '<summary>sum</summary>text'; document.body.appendChild(dtw); globalThis.__r5w = []; dtw.addEventListener('toggle', function (e) { globalThis.__r5w.push(String(e.source === null)); }); dtw.click(); globalThis.__r5ws = Number(dtw.hasAttribute('open')); dtw.querySelector('summary').click(); globalThis.__r5ws2 = Number(dtw.hasAttribute('open'));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5s + '|' + globalThis.__r5a.join('|')").unwrap().value,
        "1|true",
        "R5：summary-less details 点击翻转 + toggle source=null"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5ws + '|' + globalThis.__r5ws2 + '|' + globalThis.__r5w.join('|')").unwrap().value,
        "0|1|true",
        "R5：有 summary 直击不翻转；summary 点击翻转且 source=null"
    );

    // R6：light dismiss（R5263 两腿 down+up）收的 close request → toggle source
    // null（无 invoker 条目）。
    sandbox
        .execute(
            "var ld = document.createElement('dialog'); ld.closedBy = 'any'; document.body.appendChild(ld); ld.show(); globalThis.__r6 = []; ld.addEventListener('toggle', function (e) { globalThis.__r6.push(String(e.source === null)); }); __zw_dialog_light_dismiss_check('body'); __zw_dialog_light_dismiss_up('body'); globalThis.__r6s = Number(ld.hasAttribute('open'));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r6s + '|' + globalThis.__r6.join('|')").unwrap().value,
        "0|true",
        "R6：light dismiss 关闭——toggle source=null"
    );
}




/// R4500（html-semantics M2 切片 25，2026-10-22）：DOM void 方法返回值面。spec
/// close()/show()/showModal()/requestClose() 及 cleanup 高频方法（remove/focus/
/// blur/select/reset/showPopover/hidePopover）均为 void（返 undefined）。此前
/// close() IDL 包装透传 helper 的 wasOpen boolean：WPT cleanup
/// `t.add_cleanup(() => dialog.close())` 返 false → testharness
/// is_valid_cleanup_result 判 invalid → tests.abort() → 整文件余案被 file
/// Timeout 吞没（dialog-closedby / dialog-closedby-corner-cases 首案过后
/// 续全灭、各余 ~28-29 案的根因）。
/// https://html.spec.whatwg.org/multipage/interactive-elements.html#dom-dialog-close
#[test]
fn test_dialog_void_return_r4500() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // R1：dialog 四方法全生命周期态全 undefined（未开 no-op / 非模态开 / 已开 no-op /
    // 关 / 模态开 / 已模态 no-op / 关 / 非模态开 / requestClose 关 / 已关 no-op）。
    sandbox
        .execute(
            "var d = document.createElement('dialog'); document.body.appendChild(d);             globalThis.__r1 = String(d.close()) + ',' + String(d.show()) + ',' + String(d.show())             + ',' + String(d.close()) + ',' + String(d.showModal()) + ',' + String(d.showModal())             + ',' + String(d.close()) + ',' + String(d.show()) + ',' + String(d.requestClose())             + ',' + String(d.requestClose());",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r1").unwrap().value,
        "undefined,undefined,undefined,undefined,undefined,undefined,undefined,undefined,undefined,undefined",
        "R1：dialog show/showModal/close/requestClose 各态返 undefined（void）"
    );

    // R2：cleanup 高频同族 void 审计——remove/focus/blur/select/reset/showPopover/
    // hidePopover 返回 undefined。
    sandbox
        .execute(
            "var el = document.createElement('div'); document.body.appendChild(el);             var inp = document.createElement('input'); inp.value = 'x'; document.body.appendChild(inp);             var pop = document.createElement('div'); pop.setAttribute('popover', ''); document.body.appendChild(pop);             var fm = document.createElement('form'); document.body.appendChild(fm);             globalThis.__r2 = String(el.remove()) + ',' + String(el.focus()) + ',' + String(el.blur())             + ',' + String(inp.select()) + ',' + String(fm.reset())             + ',' + String(pop.showPopover()) + ',' + String(pop.hidePopover());",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r2").unwrap().value,
        "undefined,undefined,undefined,undefined,undefined,undefined,undefined",
        "R2：remove/focus/blur/select/reset/showPopover/hidePopover 全 undefined"
    );

    // R3：void 化不破事件语义——close() 后 close 事件仍派发（异步排队跨 execute 到达）。
    sandbox
        .execute(
            "var d3 = document.createElement('dialog'); document.body.appendChild(d3);             globalThis.__r3 = [];             d3.addEventListener('close', function () { globalThis.__r3.push('close'); });             d3.show();             d3.close();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r3.join('|')").unwrap().value,
        "close",
        "R3：close() 返 undefined 且 close 事件照常派发"
    );
}
