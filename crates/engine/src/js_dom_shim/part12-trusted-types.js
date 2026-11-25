// part12（2026-11-22 wptfyi-dom DC-2 切片 5 刀①）：Trusted Types API 工厂核心面。
//
// 目标面：trusted-types 上游语料（DC-2 切片 4 census：wholesale
// "trustedTypes is not defined" 149 处——工厂面整缺；F 大头
// TrustedTypePolicyFactory-createPolicy-createXYZTests 28F /
// TrustedTypePolicyFactory-getPropertyType.tentative 28F /
// trusted-types-createHTMLDocument 12F / constants 6F / non-tt-policy-name 5F，
// 以及 sink 赋值 accepts 面 Element-setAttribute/Document-write 等依赖工厂
// 产物的族）。
//
// 规范：https://w3c.github.io/webappsec-trusted-types/dist/spec/
//   - createPolicy 算法：#dom-trustedtypepolicyfactory-createpolicy
//     （回调返 undefined → TypeError、返 null → 空串、其余 String 化；多参
//     原样透传——createXYZTests "Arbitrary number of arguments" 实证）
//   - is* 判定：#is-trusted-type（须内部品牌，不得走 instanceof/原型链——
//     isXXX 语料 Object.create(html) instanceof 真 而 is* 假实证）
//   - sink 映射表：#the-default-policy-helpers（getPropertyType 属性名大小写
//     敏感、标签 ASCII 不敏感；getAttributeType 属性 ASCII 不敏感、带 ns 限
//     定——空/HTML ns 走 HTML 面、SVG ns 仅 script[href] 且 attrNs 空或
//     xlink；qualified "xlink:href" 不命中）
//   - unenforced（无 CSP enforcement）面：重复 policy 名容忍、任意 policy
//     名（含空串/非 ASCII）合法——createPolicy-unenforced /
//     non-tt-policy-name 语料实证；'default' 名创建即置 defaultPolicy。
//
// 划界（刀②③后续）：sink enforcement（require-trusted-types-for CSP 门——
// block-string-assignment-to-* / metadata / cspTests 簇的 TypeError 断言面）
// 与 Sanitizer（san 语料）不在本刀；本刀只加工厂，不改任何 sink（innerHTML/
// setAttribute 等既有行为零变化，对既有通道零回归面）。零新 Rust 桥。
//
// 挂载：js_dom_bridge.rs 拼接链尾（part11-computed-color.js 后，part10/11
// 自包含 IIFE 先例）；引用面仅限 globalThis 契约（window===globalThis，
// part06——window.trustedTypes 与裸 trustedTypes 双访问形态同点覆盖）。
(function () {
  'use strict';
  if (globalThis.TrustedHTML) return; // 幂等（shim 重装配边界）

  // ===== 1. 内部品牌（WeakSet——is* 判定不看原型链）=====
  // 每类一个 WeakSet；工厂产出的对象入集，is* 即 has。Object.create 派生、
  // Object.assign 克隆（自有品牌槽不可枚举也不会被复制）均不入集。
  var _zwTtHtmlSet = new WeakSet();
  var _zwTtScriptSet = new WeakSet();
  var _zwTtScriptUrlSet = new WeakSet();

  // ===== 2. 接口对象（WebIDL interface object 形态：函数 + prototype；
  // 非构造——调用抛 TypeError Illegal constructor）=====
  var _zwTtIllegalCtor = function (name) {
    return function () {
      throw new TypeError('Illegal constructor: ' + name);
    };
  };
  var _zwTtMakeInterface = function (name) {
    var f = _zwTtIllegalCtor(name);
    // 原型挂 stringifier 基座（数据槽由工厂实例以不可枚举自有属性携带）。
    var proto = {};
    Object.defineProperty(proto, 'toString', {
      value: function () {
        return this.__zwTtData;
      },
      writable: true, enumerable: false, configurable: true
    });
    Object.defineProperty(proto, 'valueOf', {
      value: function () {
        return this.__zwTtData;
      },
      writable: true, enumerable: false, configurable: true
    });
    Object.defineProperty(f, 'prototype', { value: proto });
    // 刀㉑（wptfyi-dom s10k21）：WebIDL interface prototype object 强制自有
    // `constructor`（writable/non-enum/configurable，值＝接口对象）——缺位时
    // Trusted 实例 `x.constructor.name` 落 Object.prototype 链读出 "Object"，
    // HTMLElement-generic 以 `value.constructor.name != getPropertyType(...)`
    // 计 expect_exception，把「接受真 Trusted 值」的正确行为误判成 fail
    //（20 face：innerHTML/srcdoc/text/innerText/textContent × Trusted ×
    // connected 域；script.src 域 sink 类型感知拒错型 Trusted 值不受影响）。
    // https://webidl.spec.whatwg.org/#interface-prototype-object
    Object.defineProperty(proto, 'constructor',
      { value: f, writable: true, enumerable: false, configurable: true });
    Object.defineProperty(f, 'name', { value: name });
    return f;
  };
  var TrustedHTML = _zwTtMakeInterface('TrustedHTML');
  var TrustedScript = _zwTtMakeInterface('TrustedScript');
  var TrustedScriptURL = _zwTtMakeInterface('TrustedScriptURL');
  // 切片 8（wptfyi-dom，2026-11-24）：TrustedParserOptions 接口对象（WICG
  // declarative-partial-updates——pass-trusted 页 `TrustedParserOptions` on
  // globalThis + `createParserOptions` 产物 instanceof 断言面；旧版缺接口对象，
  // globalThis 直读 ReferenceError 10F）。brand 槽复用 __zwTtData 数据通道。
  var TrustedParserOptions = _zwTtMakeInterface('TrustedParserOptions');
  var TrustedTypePolicy = _zwTtMakeInterface('TrustedTypePolicy');
  var TrustedTypePolicyFactory = _zwTtMakeInterface('TrustedTypePolicyFactory');

  // 实例造器：原型挂接口 prototype、数据槽不可枚举自有（Object.assign 不复制、
  // Object.create 派生不成自有）、入品牌集。
  var _zwTtCreate = function (iface, brandSet, data) {
    var o = Object.create(iface.prototype);
    Object.defineProperty(o, '__zwTtData', { value: data });
    brandSet.add(o);
    return o;
  };

  // ===== 3. sink 映射表（规范 #the-default-policy-helpers）=====
  // 标签/属性 ASCII 大小写折叠（'ſ' 等非 ASCII 不折叠——语料 ſcript 期望 null）。
  var _zwTtAsciiLower = function (s) {
    return String(s).replace(/[A-Z]/g, function (c) { return c.toLowerCase(); });
  };
  var _zwTtNsHtml = 'http://www.w3.org/1999/xhtml';
  var _zwTtNsSvg = 'http://www.w3.org/2000/svg';
  var _zwTtNsXlink = 'http://www.w3.org/1999/xlink';

  // 属性面：tag（ASCII-ci）× prop（cs）→ 类型或 null。innerHTML/outerHTML 跨
  // elementNs 普适（语料 NSURI_ARRAY 全 ns 断言 TrustedHTML）；其余规则在
  // elementNs 空/null/HTML 门外（非 HTML ns 一律 null，大小写敏感）。
  var _zwTtGetPropertyType = function (tag, prop, elementNs) {
    var p = String(prop);
    if (p === 'innerHTML' || p === 'outerHTML') return 'TrustedHTML';
    if (elementNs !== undefined && elementNs !== null && elementNs !== '' &&
        elementNs !== _zwTtNsHtml) return null;
    var t = _zwTtAsciiLower(tag);
    if (t === 'script') {
      if (p === 'src') return 'TrustedScriptURL';
      if (p === 'text' || p === 'innerText' || p === 'textContent') return 'TrustedScript';
    }
    if (t === 'iframe' && p === 'srcdoc') return 'TrustedHTML';
    return null;
  };

  // 事件处理器内容属性清单（HTML spec #event-handlers-on-elements 固定表——
  // getAttributeType 语料 ondoesnotexist 期望 null：on* 非「前缀」匹配而按
  // 此清单判定；清单覆盖语料 attributes.js 探针集 + WHATWG 全集）。
  var _zwTtEventHandlers = {};
  ('onabort onauxclick onbeforeinput onbeforematch onbeforetoggle onbeforecopy ' +
   'onbeforecut onbeforepaste onblur oncancel oncanplay oncanplaythrough onchange ' +
   'onclick onclose oncontextlost oncontextmenu oncontextrestored oncopy oncuechange ' +
   'oncut ondblclick ondrag ondragend ondragenter ondragleave ondragover ondragstart ' +
   'ondrop ondurationchange onemptied onended onerror onfocus onformdata oninput ' +
   'oninvalid onkeydown onkeypress onkeyup onload onloadeddata onloadedmetadata ' +
   'onloadstart onmousedown onmouseenter onmouseleave onmousemove onmouseout ' +
   'onmouseover onmouseup onmousewheel onpaste onpause onplay onplaying onprogress ' +
   'onratechange onreset onresize onscroll onscrollend onsecuritypolicyviolation ' +
   'onseeked onseeking onselect onslotchange onstalled onsuspend onsubmit ' +
   'ontimeupdate ontoggle onvolumechange onwaiting onwebkitanimationend ' +
   'onwebkitanimationiteration onwebkitanimationstart onwebkittransitionend onwheel')
    .split(' ').forEach(function (h) { _zwTtEventHandlers[h] = true; });

  // 属性面（getAttributeType）：tag ASCII-ci × attr ASCII-ci × elementNs × attrNs。
  // 事件处理器：真实清单成员 × elementNs ∈ {空, HTML, SVG, MathML} × attrNs 必空
  // → TrustedScript（attributes.js：SVG g[ondblclick]/MathML mrow[onmousedown]
  // 真、FOO ns/attrNs 非空/伪名 ondoesnotexist 均假）。
  // HTML 面：iframe[srcdoc] → TrustedHTML；script[src] → TrustedScriptURL。
  // SVG 面（elementNs 精确匹配、大小写敏感）：script[href] 且 attrNs 空或
  // xlink → TrustedScriptURL；qualified 名 'xlink:href' 不命中（须传局部名）。
  var _zwTtGetAttributeType = function (tag, attr, elementNs, attrNs) {
    var en = elementNs === undefined || elementNs === null ? '' : elementNs;
    var an = attrNs === undefined || attrNs === null ? '' : attrNs;
    var t = _zwTtAsciiLower(tag);
    var a = _zwTtAsciiLower(attr);
    if (an === '' && _zwTtEventHandlers[a] &&
        (en === '' || en === _zwTtNsHtml || en === _zwTtNsSvg ||
         en === 'http://www.w3.org/1998/Math/MathML')) {
      return 'TrustedScript';
    }
    if (en === '' || en === _zwTtNsHtml) {
      if (an !== '') return null;
      if (t === 'iframe' && a === 'srcdoc') return 'TrustedHTML';
      if (t === 'script' && a === 'src') return 'TrustedScriptURL';
      return null;
    }
    if (en === _zwTtNsSvg) {
      if (t === 'script' && a === 'href' && (an === '' || an === _zwTtNsXlink)) {
        return 'TrustedScriptURL';
      }
      return null;
    }
    return null;
  };

  // ===== 4. 工厂单例 =====
  var _zwTtDefaultPolicy = null;
  // 刀③⑤：名单域重名追踪（仅 CSP trusted-types 指令存在时启用——
  // 无指令页重名容忍；'allow-duplicates' 在列时也容忍）。
  var _zwTtCreatedNames = null;
  var _zwTtEmptyHtml = null;
  var _zwTtEmptyScript = null;

  // 策略方法造器：回调缺席 → TypeError；回调抛错原样透传；返 undefined →
  // TypeError、返 null → 空串、其余 String 化（#dom-trustedtypepolicy-createhtml
  // 算法）。多参原样透传给回调（createXYZTests 三参/缺参实证）。
  var _zwTtPolicyMethod = function (cb, iface, brandSet, typeName) {
    return function () {
      if (typeof cb !== 'function') {
        throw new TypeError('The ' + typeName + ' callback is not defined for this policy.');
      }
      var r = cb.apply(undefined, arguments);
      if (r === undefined) {
        throw new TypeError('The ' + typeName + ' callback returned undefined.');
      }
      var data = r === null ? '' : String(r);
      return _zwTtCreate(iface, brandSet, data);
    };
  };

  var trustedTypesInstance = {};
  trustedTypesInstance.__proto__ = TrustedTypePolicyFactory.prototype;

  var _zwTtCreatePolicy = function (name, callbacks) {
    var n = String(name);
    // 刀③⑤：CSP trusted-types 名单指令门（spec
    // https://w3c.github.io/webappsec-trusted-types/dist/spec/
    // #should-block-create-policy ）——解析态由 part14 meta 扫描提供
    // （装配序 part12 先于 part14，运行时经 globalThis 桥）。名单指令
    // 存在时：'allow-duplicates' 缺席 → 重名阻断；名单外名字阻断。
    // 无指令页保持重名容忍（createPolicy-unenforced 语料实证）。
    var gate = globalThis.__zwTtPolicyAllowed;
    var stateFn = globalThis.__zwTtCspState;
    if (typeof gate === 'function' && typeof stateFn === 'function') {
      var st = stateFn();
      if (st && st.hasList) {
        if (!st.allowDuplicates && _zwTtCreatedNames !== null
            && _zwTtCreatedNames.indexOf(n) >= 0) {
          throw new TypeError(
            "Refused to create a TrustedTypePolicy with name '" + n + "' since a policy with the same name already exists.");
        }
        if (!gate(n)) {
          throw new TypeError(
            "Refused to create a TrustedTypePolicy with name '" + n + "'.");
        }
        if (_zwTtCreatedNames === null) _zwTtCreatedNames = [];
        _zwTtCreatedNames.push(n);
      }
    }
    var cbs = callbacks === undefined || callbacks === null ? {} : callbacks;
    var policy = Object.create(TrustedTypePolicy.prototype);
    Object.defineProperty(policy, 'name', {
      value: n, writable: false, enumerable: true, configurable: true
    });
    // 回调缺席容忍（{} / null 均 legal——unenforced 语料实证）；方法逐成员装配。
    Object.defineProperty(policy, 'createHTML', {
      value: _zwTtPolicyMethod(cbs.createHTML, TrustedHTML, _zwTtHtmlSet, 'createHTML'),
      writable: true, enumerable: true, configurable: true
    });
    Object.defineProperty(policy, 'createScript', {
      value: _zwTtPolicyMethod(cbs.createScript, TrustedScript, _zwTtScriptSet, 'createScript'),
      writable: true, enumerable: true, configurable: true
    });
    Object.defineProperty(policy, 'createScriptURL', {
      value: _zwTtPolicyMethod(cbs.createScriptURL, TrustedScriptURL, _zwTtScriptUrlSet, 'createScriptURL'),
      writable: true, enumerable: true, configurable: true
    });
    // 切片 7（wptfyi-dom，2026-11-23）：TrustedParserOptions 面（WICG
    // declarative-partial-updates parser options 管道——位置方法族 options
    // 包装，part15 _processOptions 消费面）。回调缺席不装配（消费面 typeof
    // 门回落透传）；回调异常原样传播（parser-options 页 assert_throws_
    // exactly / e.message 断言——不经 _zwTtPolicyMethod 的 TypeError 包装）；
    // 产物盖 `__zwTtParserOptions` 印（TrustedParserOptions 标记对象——
    // part15 跳过 default policy 二次包装的判定键）。
    // 切片 8：对象产物包装为 **TrustedParserOptions 实例**（pass-trusted 页
    // instanceof 断言）——回调产物自有属性平铺到品牌实例（下游 processed.
    // runScripts/sanitizer 读面穿透；嵌套 sanitizer 配置共享引用，语义等同
    // 直传），印随实例走。非对象产物维持透传（回调契约外，零包装零印）。
    // https://github.com/WICG/declarative-partial-updates
    if (typeof cbs.createParserOptions === 'function') {
      Object.defineProperty(policy, 'createParserOptions', {
        value: function (options) {
          // 刀⑩（dc2-s10k10）：回调实参 sanitizer 归一 Sanitizer 实例（immutable
          // 语料 clone testing 三案——dict/预设字符串/实例均以 Sanitizer 传回调；
          // 实例形 clone 隔离：回调 removeElement 不得影响 caller 原件）。
          var _poNorm = options;
          if (options && typeof options === 'object' && options.sanitizer != null
              && typeof globalThis.Sanitizer === 'function') {
            var _sanV = options.sanitizer;
            try {
              if (_sanV instanceof globalThis.Sanitizer) {
                _poNorm = Object.assign({}, options,
                  { sanitizer: new globalThis.Sanitizer(_sanV.get()) });
              } else if (typeof _sanV === 'object') {
                // 刀⑩b：空 dict＝默认配置（spec SanitizerConfig 全成员缺省 →
                // default allow-list——immutable「doesn't mutate original object」
                // 脸 {sanitizer:{}} 经归一须产出可消毒默认实例；空 dict 直灌得
                // 空允许表会整树滤光）。
                var _cl10 = JSON.parse(JSON.stringify(_sanV));
                var _clHas = false;
                for (var _clK in _cl10) {
                  if (Object.prototype.hasOwnProperty.call(_cl10, _clK)) { _clHas = true; break; }
                }
                _poNorm = Object.assign({}, options,
                  { sanitizer: _clHas ? new globalThis.Sanitizer(_cl10) : new globalThis.Sanitizer('default') });
              } else if (_sanV === 'default') {
                _poNorm = Object.assign({}, options,
                  { sanitizer: new globalThis.Sanitizer('default') });
              }
            } catch (_eP10Norm) { _poNorm = options; }
          }
          var out = cbs.createParserOptions(_poNorm);
          if (out === null || typeof out !== 'object') return out;
          var branded = Object.create(TrustedParserOptions.prototype);
          var keysT = Object.keys(out);
          for (var kt = 0; kt < keysT.length; kt++) {
            try { branded[keysT[kt]] = out[keysT[kt]]; } catch (_eTpoCp) {}
          }
          // 刀⑩：sanitizer 方法形（spec TrustedParserOptions.sanitizer()——每调
          // 返回新 Sanitizer clone；clone testing 断言两次调用不同对象、变异
          // clone 不影响消毒）。方法覆盖平铺数据属性（原始值存 __zwSanRaw10；
          // sink 消费端 `_zwSanResolveOptions` 函数形归一读）。
          if (out.sanitizer !== undefined && typeof globalThis.Sanitizer === 'function') {
            try {
              var _raw10 = out.sanitizer;
              try { branded.__zwSanRaw10 = _raw10; } catch (_eP10Raw) {}
              delete branded.sanitizer;
              Object.defineProperty(branded, 'sanitizer', {
                value: function () {
                  if (_raw10 instanceof globalThis.Sanitizer) return new globalThis.Sanitizer(_raw10.get());
                  if (_raw10 && typeof _raw10 === 'object') return new globalThis.Sanitizer(JSON.parse(JSON.stringify(_raw10)));
                  return _raw10;
                },
                writable: true, enumerable: true, configurable: true,
              });
            } catch (_eP10M) {}
          }
          try { branded.__zwTtParserOptions = true; } catch (_eTpoSt) {}
          return branded;
        },
        writable: true, enumerable: true, configurable: true
      });
    }
    // unenforced 面：重复名容忍（不查重）；'default' 名创建即置 defaultPolicy
    // （defaultPolicy 语料实证）。
    if (n === 'default') _zwTtDefaultPolicy = policy;
    return policy;
  };

  Object.defineProperty(trustedTypesInstance, 'createPolicy', {
    value: _zwTtCreatePolicy, writable: true, enumerable: true, configurable: true
  });
  Object.defineProperty(trustedTypesInstance, 'isHTML', {
    value: function (v) { return _zwTtHtmlSet.has(v); },
    writable: true, enumerable: true, configurable: true
  });
  Object.defineProperty(trustedTypesInstance, 'isScript', {
    value: function (v) { return _zwTtScriptSet.has(v); },
    writable: true, enumerable: true, configurable: true
  });
  Object.defineProperty(trustedTypesInstance, 'isScriptURL', {
    value: function (v) { return _zwTtScriptUrlSet.has(v); },
    writable: true, enumerable: true, configurable: true
  });
  Object.defineProperty(trustedTypesInstance, 'getPropertyType', {
    value: _zwTtGetPropertyType, writable: true, enumerable: true, configurable: true
  });
  Object.defineProperty(trustedTypesInstance, 'getAttributeType', {
    value: _zwTtGetAttributeType, writable: true, enumerable: true, configurable: true
  });
  // defaultPolicy/emptyHTML/emptyScript：只读 accessor（赋值静默失败——
  // defaultPolicy/constants 语料实证）。
  Object.defineProperty(trustedTypesInstance, 'defaultPolicy', {
    get: function () { return _zwTtDefaultPolicy; },
    enumerable: true, configurable: true
  });
  Object.defineProperty(trustedTypesInstance, 'emptyHTML', {
    get: function () {
      if (_zwTtEmptyHtml === null) _zwTtEmptyHtml = _zwTtCreate(TrustedHTML, _zwTtHtmlSet, '');
      return _zwTtEmptyHtml;
    },
    enumerable: true, configurable: true
  });
  Object.defineProperty(trustedTypesInstance, 'emptyScript', {
    get: function () {
      if (_zwTtEmptyScript === null) _zwTtEmptyScript = _zwTtCreate(TrustedScript, _zwTtScriptSet, '');
      return _zwTtEmptyScript;
    },
    enumerable: true, configurable: true
  });

  // ===== 5. 全局注册（接口对象 + 单例；window===globalThis 双访问形态覆盖）=====
  globalThis.TrustedHTML = TrustedHTML;
  globalThis.TrustedScript = TrustedScript;
  globalThis.TrustedScriptURL = TrustedScriptURL;
  globalThis.TrustedParserOptions = TrustedParserOptions;
  globalThis.TrustedTypePolicy = TrustedTypePolicy;
  globalThis.TrustedTypePolicyFactory = TrustedTypePolicyFactory;
  globalThis.trustedTypes = trustedTypesInstance;
})();
