// js_dom_bridge 测试切片 152。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// s10-registries 刀⑥ 常驻钉测（2026-11-24）——define() 参数校验矩阵（wpt-data 探针页
/// 属 gitignored per-clone 刮擦面，按刀①协议用后即删，本钉测即常驻回归载体）：
/// ① WebIDL 必选参数 arity 门：零参/单参 TypeError 先于名检查；
/// ② IsConstructor 门先于名检查（spec 步序）：arrow/method 不可构造函数 TypeError
/// （含「arrow＋非法名」序交叉形——旧序 validName 先行时错抛 SyntaxError）；
/// ③ prototype Get rethrow＋Type(prototype) 必须 Object：undefined（bound 函数形）/
/// 原始值 TypeError，函数形（prototype=HTMLElement 构造器对象，ECMAScript Type 面
/// 函数亦 Object）合法；
/// ④ 四 lifecycle callback 非 undefined 非 callable TypeError（null 形），getter
/// 抛错原样 rethrow（assert_throws_exactly 形）；
/// ⑤ customized built-in extends 排除集：bgsound（引擎 legacy 表内但 HTML spec 无
/// 已定义接口＝HTMLUnknownElement 形）NotSupportedError；
/// ⑥ 正向升级面：define＋createElement 候选升级 instanceOf 保持。
/// 根因链与 define.html 22F 簇见 evidence `dc2-s10k6-registries-knife6-verdict.md`。
const S10K6_DEFINE_VALIDATION_PINS_JS: &str = r#"
globalThis.__k6 = (function () {
  var bad = [];
  var wantTypeError = function (fn, tag) {
    try { fn(); bad.push(tag + ':no-throw'); return; } catch (e) {
      if (!(e instanceof TypeError)) bad.push(tag + ':' + e.name);
    }
  };
  var ce = globalThis.customElements;
  wantTypeError(function () { ce.define(); }, 'arity0');
  wantTypeError(function () { ce.define('k6-pin-one'); }, 'arity1');
  wantTypeError(function () { ce.define('k6-pin-arrow', () => {}); }, 'isctor-arrow');
  wantTypeError(function () { ce.define('k6-pin-method', ({ m() {} }).m); }, 'isctor-method');
  wantTypeError(function () { ce.define('k6 pin bad name', () => {}); }, 'isctor-order');
  var _k6fnProto = function () {};
  _k6fnProto.prototype = globalThis.HTMLElement;
  try { ce.define('k6-pin-fnproto', _k6fnProto); } catch (e) { bad.push('fnproto-accepted:' + e.name); }
  wantTypeError(function () { ce.define('k6-pin-boundproto', (function () {}).bind({})); }, 'proto-undef');
  var _k6strProto = function () {}; _k6strProto.prototype = 'string';
  wantTypeError(function () { ce.define('k6-pin-strproto', _k6strProto); }, 'proto-str');
  var _k6lcNull = function () {}; _k6lcNull.prototype.connectedCallback = null;
  wantTypeError(function () { ce.define('k6-pin-lcnull', _k6lcNull); }, 'lc-null');
  var _k6lcObj = function () {}; _k6lcObj.prototype.disconnectedCallback = {};
  wantTypeError(function () { ce.define('k6-pin-lcobj', _k6lcObj); }, 'lc-obj');
  var marker = { thrown: false, err: new Error('k6 pin rethrow') };
  var _k6lcGet = function () {};
  Object.defineProperty(_k6lcGet.prototype, 'adoptedCallback', {
    get: function () { marker.thrown = true; throw marker.err; }
  });
  try { ce.define('k6-pin-lcget', _k6lcGet); bad.push('lc-get:no-throw'); } catch (e) {
    if (!marker.thrown || e !== marker.err) bad.push('lc-get:wrong');
  }
  try { ce.define('k6-pin-bgsound', function () {}, { extends: 'bgsound' }); bad.push('bgsound:no-throw'); } catch (e) {
    if (e.name !== 'NotSupportedError') bad.push('bgsound:' + e.name);
  }
  var up = 0;
  var K6PinEl = function () { up++; };
  K6PinEl.prototype = Object.create(globalThis.HTMLElement.prototype);
  K6PinEl.prototype.connectedCallback = function () { up += 10; };
  ce.define('k6-pin-element', K6PinEl);
  var el = globalThis.document.createElement('k6-pin-element');
  globalThis.document.body.appendChild(el);
  if (!(el instanceof K6PinEl)) bad.push('upgrade:not-instance');
  if (up !== 11) bad.push('upgrade:fired=' + up);
  return bad.length === 0 ? true : bad.join('|');
})();
"#;

#[test]
fn test_s10k6_define_validation_pins() {
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
        "<html><body><div id=\"k6page\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/custom-elements/registries/pins-k6.html".to_string(),
    ));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.execute(S10K6_DEFINE_VALIDATION_PINS_JS).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__k6").unwrap().value,
        "true",
        "刀⑥ define() 参数校验矩阵：arity 门＋IsConstructor 序门＋prototype/lifecycle 校验＋cbi extends 排除集＋正向升级面"
    );
}
