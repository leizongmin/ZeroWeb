// js_dom_bridge 测试切片 153。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// s10-registries 刀⑦ 常驻钉测（2026-11-24）——define() spec 步序重构＋Get/转换
/// rethrow 矩阵＋per-registry running flag＋whenDefined promise 恒等＋shadow-including
/// 升级序（wpt-data 探针页属 gitignored per-clone 刮擦面，按刀①协议用后即删，本钉测
/// 即常驻回归载体）：
/// ① IsConstructor gOPD 探针零可观察 get：dup 名二次 define 抛 NotSupportedError 且
///    代理 get 记录零新增（ Reflect.construct 探针会记录 'prototype' get——刀⑥ 面回归）；
/// ② per-registry running flag：ctor prototype get 内嵌套 define（同 registry）抛
///    NotSupportedError、外层 get 记录＝['prototype']、内层零 get；失败 define 后
///    finally 清旗再 define 成功；跨 registry（CustomElementRegistry 子实例）不互斥；
/// ③ ctor Get 序＝['prototype','disabledFeatures','formAssociated']；
/// ④ observedAttributes 条件链：有 attributeChangedCallback 才 Get；Get 抛错原样
///    rethrow（恒等）；非 iterable TypeError；迭代器中途抛错 rethrow；无 attrCb 不读；
/// ⑤ disabledFeatures 四形同矩阵；formAssociated Get rethrow＋truthy→ToBoolean→
///    九 lifecycle get（五 base＋四 form）；form callback Get 抛错/返回非 callable
///    TypeError 中止；falsy 跳过 form 回调读；
/// ⑥ whenDefined 恒等：未定义名重复调用返同一 promise；define 后同 promise 以
///    ctor resolve；
/// ⑦ shadow-including 升级序：closed shadow root＋嵌套形态下升级序＝
///    light 前 → shadow 子树（嵌套先深）→ host light 子 → light 后（真树 pre-order
///    walk，childNodes live 视图双世界通吃）；
/// ⑧ scoped registry extends → NotSupportedError（spec 步 7.1），autonomous 可用。
/// 根因链与 CustomElementRegistry.html 19F→1F 簇见 evidence
/// `dc2-s10k7-registries-knife7-verdict.md`。
const S10K7_REGISTRIES_KNIFE7_PINS_JS: &str = r#"
globalThis.__k7 = (function () {
  var bad = [];
  var wantTypeError = function (fn, tag) {
    try { fn(); bad.push(tag + ':no-throw'); } catch (e) {
      if (!(e instanceof TypeError)) bad.push(tag + ':' + e.name);
    }
  };
  var wantNSE = function (fn, tag) {
    try { fn(); bad.push(tag + ':no-throw'); } catch (e) {
      if (!(e instanceof DOMException) || e.name !== 'NotSupportedError') bad.push(tag + ':' + (e.name || typeof e));
    }
  };
  var ce = globalThis.customElements;
  var _k7n = 0;
  var nm = function (p) { return 'k7-pin-' + p + '-' + (_k7n++); };

  // ① dup 名零可观察 get（gOPD 探针形）。
  var C1 = class extends HTMLElement {};
  ce.define(nm('dupbase'), C1);
  var dupCalls = 0;
  var DupProxy = new Proxy(class extends HTMLElement {}, {
    get: function (t, n) { dupCalls++; return t[n]; }
  });
  ce.define(nm('dupother'), DupProxy);
  var pre = dupCalls;
  wantNSE(function () { ce.define(nm('dupother'), DupProxy); }, 'dup-zero-get');
  if (dupCalls !== pre) bad.push('dup:gets=' + (dupCalls - pre));

  // ② running flag：同 registry 嵌套 define NSE＋get 记录形；finally 清旗。
  var outerGets = [], innerGets = [];
  var InnerEl = new Proxy(class extends HTMLElement {}, {
    get: function (t, n) { innerGets.push(n); return t[n]; }
  });
  var OuterEl = new Proxy(class extends HTMLElement {}, {
    get: function (t, n) {
      outerGets.push(n);
      if (n === 'prototype') ce.define(nm('inner'), InnerEl);
      return t[n];
    }
  });
  wantNSE(function () { ce.define(nm('outer'), OuterEl); }, 'runflag');
  if (outerGets.join(',') !== 'prototype') bad.push('runflag:outer=' + outerGets.join(','));
  if (innerGets.length !== 0) bad.push('runflag:inner=' + innerGets.join(','));
  try { ce.define(nm('after-runflag'), class extends HTMLElement {}); } catch (e) { bad.push('runflag:finally=' + e.name); }
  // 跨 registry 不互斥：子实例 define 在主 define prototype get 期间成功。
  var crossOk = true;
  var ScopedChild = new CustomElementRegistry();
  var CrossEl = new Proxy(class extends HTMLElement {}, {
    get: function (t, n) {
      if (n === 'prototype') {
        try { ScopedChild.define(nm('cross-child'), class extends HTMLElement {}); } catch (e) { crossOk = false; }
      }
      return t[n];
    }
  });
  ce.define(nm('cross-outer'), CrossEl);
  if (!crossOk) bad.push('cross:child-threw');

  // ③ ctor Get 序。
  var orderCalls = [];
  var OrderProxy = new Proxy(class extends HTMLElement {}, {
    get: function (t, n) { orderCalls.push(n); return t[n]; }
  });
  ce.define(nm('order'), OrderProxy);
  if (orderCalls.join(',') !== 'prototype,disabledFeatures,formAssociated') bad.push('order:' + orderCalls.join(','));

  // ④ observedAttributes 条件链四形。
  var marker = { err: { name: 'k7obs' } };
  var ObsBase = class extends HTMLElement {};
  ObsBase.prototype.attributeChangedCallback = function () {};
  var ObsGetProxy = new Proxy(ObsBase, {
    get: function (t, n) {
      if (n === 'observedAttributes') throw marker.err;
      return t[n];
    }
  });
  var threw = null;
  try { ce.define(nm('obsget'), ObsGetProxy); } catch (e) { threw = e; }
  if (threw !== marker.err) bad.push('obsget:' + (threw && threw.name));
  var ObsConv = function () {};
  ObsConv.prototype.attributeChangedCallback = function () {};
  ObsConv.observedAttributes = 1;
  wantTypeError(function () { ce.define(nm('obsconv'), ObsConv); }, 'obsconv');
  var ObsIter = function () {};
  ObsIter.prototype.attributeChangedCallback = function () {};
  ObsIter.observedAttributes = { '': undefined, next: null };
  try { ce.define(nm('obsiter'), ObsIter); bad.push('obsiter:no-throw'); } catch (e) {
    if (!(e instanceof TypeError)) bad.push('obsiter:' + e.name);
  }
  var ObsSym = function () {};
  ObsSym.prototype.attributeChangedCallback = function () {};
  ObsSym.observedAttributes = { [Symbol.iterator]: 1 };
  wantTypeError(function () { ce.define(nm('obssi'), ObsSym); }, 'obssi');
  var ObsNoCb = function () {};
  ObsNoCb.observedAttributes = 1;
  try { ce.define(nm('obsnocb'), ObsNoCb); } catch (e) { bad.push('obsnocb:' + e.name); }

  // ⑤ disabledFeatures／formAssociated 面。
  var DisErr = { name: 'k7dis' };
  var DisProxy = new Proxy(class extends HTMLElement {}, {
    get: function (t, n) { if (n === 'disabledFeatures') throw DisErr; return t[n]; }
  });
  threw = null;
  try { ce.define(nm('disget'), DisProxy); } catch (e) { threw = e; }
  if (threw !== DisErr) bad.push('disget:' + (threw && threw.name));
  var DisConv = function () {};
  DisConv.disabledFeatures = 1;
  wantTypeError(function () { ce.define(nm('disconv'), DisConv); }, 'disconv');
  var FaErr = { name: 'k7fa' };
  var FaProxy = new Proxy(class extends HTMLElement {}, {
    get: function (t, n) { if (n === 'formAssociated') throw FaErr; return t[n]; }
  });
  threw = null;
  try { ce.define(nm('faget'), FaProxy); } catch (e) { threw = e; }
  if (threw !== FaErr) bad.push('faget:' + (threw && threw.name));
  var protoGets = [];
  var FaTrue = function () {};
  FaTrue.prototype = new Proxy(FaTrue.prototype, {
    get: function (t, n) { protoGets.push(n); return t[n]; }
  });
  var FaTrueProxy = new Proxy(FaTrue, {
    get: function (t, n) { if (n === 'formAssociated') return 1; return t[n]; }
  });
  ce.define(nm('fatrue'), FaTrueProxy);
  if (protoGets.join(',') !== 'connectedCallback,disconnectedCallback,connectedMoveCallback,adoptedCallback,attributeChangedCallback,formAssociatedCallback,formResetCallback,formDisabledCallback,formStateRestoreCallback') bad.push('fatrue:' + protoGets.join(','));
  var faAbortGets = [];
  var FaAbort = function () {};
  FaAbort.prototype = new Proxy(FaAbort.prototype, {
    get: function (t, n) { faAbortGets.push(n); if (n === 'formDisabledCallback') return 43; return t[n]; }
  });
  var FaAbortProxy = new Proxy(FaAbort, {
    get: function (t, n) { if (n === 'formAssociated') return {}; return t[n]; }
  });
  wantTypeError(function () { ce.define(nm('faabort'), FaAbortProxy); }, 'faabort');
  if (faAbortGets.join(',') !== 'connectedCallback,disconnectedCallback,connectedMoveCallback,adoptedCallback,attributeChangedCallback,formAssociatedCallback,formResetCallback,formDisabledCallback') bad.push('faabort:' + faAbortGets.join(','));
  var faFalseGets = [];
  var FaFalse = function () {};
  FaFalse.prototype = new Proxy(FaFalse.prototype, {
    get: function (t, n) { faFalseGets.push(n); return t[n]; }
  });
  var FaFalseProxy = new Proxy(FaFalse, {
    get: function (t, n) { if (n === 'formAssociated') return 0; return t[n]; }
  });
  ce.define(nm('fafalse'), FaFalseProxy);
  if (faFalseGets.join(',') !== 'connectedCallback,disconnectedCallback,connectedMoveCallback,adoptedCallback,attributeChangedCallback') bad.push('fafalse:' + faFalseGets.join(','));

  // ⑥ whenDefined promise 恒等＋define resolve（名只取一次——nm 自增）。
  var wdName = nm('wd-never');
  var p1 = ce.whenDefined(wdName);
  var p2 = ce.whenDefined(wdName);
  if (p1 !== p2) bad.push('wd:identity');
  if (!(p1 instanceof Promise)) bad.push('wd:not-promise');
  globalThis.__k7wdp = p1;
  var WdCtor = class extends HTMLElement {};
  ce.define(wdName, WdCtor);
  globalThis.__k7wdctor = WdCtor;

  // ⑦ shadow-including 升级序（closed root＋嵌套）。
  var mk = function (tag, id) {
    var el = globalThis.document.createElement(tag);
    el.__k7id = id;
    return el;
  };
  var before = mk('some-custom', 'before');
  var after = mk('some-custom', 'after');
  var inShadow = mk('some-custom', 'inShadow');
  var childOfHost = mk('some-custom', 'childOfHost');
  var innerHost = mk('some-custom', 'innerHost');
  var inNested = mk('some-custom', 'inNested');
  var container = globalThis.document.createElement('div');
  var shadowHost = globalThis.document.createElement('div');
  var sr = shadowHost.attachShadow({ mode: 'closed' });
  container.appendChild(before);
  container.appendChild(shadowHost);
  container.appendChild(after);
  shadowHost.appendChild(childOfHost);
  sr.appendChild(inShadow);
  sr.appendChild(innerHost);
  var innerRoot = innerHost.attachShadow({ mode: 'closed' });
  innerRoot.appendChild(inNested);
  globalThis.document.body.appendChild(container);
  var upCalls = [];
  var K7Shadow = class extends HTMLElement {
    constructor() { super(); upCalls.push(this); }
  };
  ce.define('some-custom', K7Shadow);
  var wantOrder = [before, inShadow, innerHost, inNested, childOfHost, after];
  if (upCalls.length !== wantOrder.length) bad.push('shadow:len=' + upCalls.length);
  else {
    for (var oi = 0; oi < wantOrder.length; oi++) {
      if (upCalls[oi] !== wantOrder[oi]) { bad.push('shadow:idx' + oi + '=' + ((upCalls[oi] && upCalls[oi].__k7id) || '?') + ' want ' + wantOrder[oi].__k7id); break; }
    }
  }

  // ⑧ scoped registry extends → NSE；autonomous 可用。
  wantNSE(function () { ScopedChild.define(nm('sc'), class extends HTMLElement {}, { extends: 'div' }); }, 'scoped-extends');
  try { ScopedChild.define(nm('sc-auto'), class extends HTMLElement {}); } catch (e) { bad.push('scoped-auto:' + e.name); }

  return bad.length === 0 ? { ok: true, p1: globalThis.__k7wdp, ctor: globalThis.__k7wdctor } : { ok: false, bad: bad.join('|') };
})();
"#;

#[test]
fn test_s10k7_registries_knife7_pins() {
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
        "<html><body><div id=\"k7page\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/custom-elements/registries/pins-k7.html".to_string(),
    ));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.execute(S10K7_REGISTRIES_KNIFE7_PINS_JS).unwrap();
    let verdict = sandbox
        .execute("JSON.stringify(globalThis.__k7)")
        .unwrap()
        .value;
    assert!(
        verdict.contains("\"ok\":true"),
        "刀⑦ define 步序重构＋Get/转换 rethrow＋running flag＋whenDefined 恒等＋shadow-including 升级序：{}",
        verdict
    );
    // ⑥ 微任务面：define 后 whenDefined 同 promise 以 ctor resolve。注册后 noop
    // execute 排空（part14 createImageBitmap 先例：两条 pump）。
    sandbox
        .execute(
            "globalThis.__k7wdResolved = null; globalThis.__k7.p1.then(function (v) { globalThis.__k7wdResolved = (v === globalThis.__k7.ctor); }, function () { globalThis.__k7wdResolved = false; });",
        )
        .unwrap();
    sandbox.execute("globalThis.__k7noop = 1;").unwrap();
    sandbox.execute("globalThis.__k7noop = 2;").unwrap();
    let resolved = sandbox
        .execute("String(globalThis.__k7wdResolved)")
        .unwrap()
        .value;
    assert_eq!(
        resolved, "true",
        "whenDefined 未以 ctor resolve"
    );
}
