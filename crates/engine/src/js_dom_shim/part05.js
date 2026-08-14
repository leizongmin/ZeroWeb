          }
        } else if (p === 'defaultValue') {
          // `input.defaultValue = x`（R2840）——反射 `value` 属性（初始值；attr 名映射 defaultValue→value）。
          // 仅设 value 属性，不联动 .value 当前态（spec 仅当当前值等于旧 defaultValue 时联动——罕见 defer）。
          // R2996：显式设 defaultValue 重同步（清 dirty，getter 回落新属性值）。
          if (_realTag(sel, handle) === 'INPUT') {
            _clearInputDefault(key);
            if (handle) __zw_set_attr_handle(handle, 'value', String(value));
            else { __zw_set_attr(sel, 'value', String(value)); moAttr = 'value'; }
          } else if (_realTag(sel, handle) === 'OUTPUT') {
            // https://html.spec.whatwg.org/multipage/form-elements.html#dom-output-defaultvalue
            var _odv = String(value);
            _outputDefault[key] = _odv;
            if (_outputValue[key] == null) {
              if (handle) __zw_set_text_handle(handle, _odv);
              else __zw_set_text(sel, _odv);
            }
          }
        } else if (p === 'defaultChecked') {
          // `input.defaultChecked = x`（R2840）——boolean 反射 `checked` 属性（truthy→设存在，falsy→移除）。
          // R2998：显式设 defaultChecked 重同步（清 dirty，getter 回落新属性值 latest-wins）。
          if (_realTag(sel, handle) === 'INPUT') {
            _clearBoolDefault(key, 'checked');
            if (value) {
              if (handle) __zw_set_attr_handle(handle, 'checked', '');
              else { __zw_set_attr(sel, 'checked', ''); moAttr = 'checked'; }
            } else if (!handle && typeof __zw_remove_attr === 'function') {
              __zw_remove_attr(sel, 'checked'); moAttr = 'checked';
            }
          }
        } else if (p === 'hidden' || p === 'checked' || p === 'disabled' || p === 'selected') {
          // boolean reflected property：truthy → 设存在（空值，has_attr=true）；falsy → 真移除
          // （has_attr=false）。修正旧 fallthrough 写空串致 falsy 仍 present 的 bug。
          // R2998：`.checked=`(INPUT)/`.selected=`(OPTION) 写属性前捕获 defaultChecked/defaultSelected 真默认态
          // （spec .checked=/.selected= 不改 default*；shim 写属性供 render 故污染，缓存保护 default* getter）。
          if (p === 'checked' && _realTag(sel, handle) === 'INPUT') _captureBoolDefault(key, 'checked', sel, handle);
          else if (p === 'selected' && _realTag(sel, handle) === 'OPTION') _captureBoolDefault(key, 'selected', sel, handle);
          if (value) {
            if (handle) __zw_set_attr_handle(handle, p, '');
            else __zw_set_attr(sel, p, '');
            moAttr = p;
          } else if (!handle && typeof __zw_remove_attr === 'function') {
            __zw_remove_attr(sel, p);
            moAttr = p;
          }
          // handle falsy：无 remove-handle 变体 → 不设（detach 元素 append 时默认无该布尔属性）。
        } else if (_reflectedBoolAttr(p) !== null) {
          // R3039/R3040：布尔 reflected setter（_REFLECTED_BOOL 全表）。旧经 generic fallthrough 写
          // `attr="false"`（present）→ 读返 true（set-false bug）。修正：truthy → set 空（presence）；
          // falsy → removeAttribute（sel 走 `__zw_remove_attr`，handle 走 `__zw_remove_attr_handle`，detached 亦真移除）。
          // 闭合布尔 set→get 全往返（R3038 读 + R3039/R3040 set）。attr 名经 `_reflectedBoolAttr` 映射（readOnly→readonly /
          // noValidate→novalidate / playsInline→playsinline / isMap→ismap / itemScope→itemscope 等）。
          var _bAttrName = _reflectedBoolAttr(p);
          if (value) {
            if (handle) __zw_set_attr_handle(handle, _bAttrName, '');
            else { __zw_set_attr(sel, _bAttrName, ''); moAttr = _bAttrName; }
          } else if (handle && typeof __zw_remove_attr_handle === 'function') {
            __zw_remove_attr_handle(handle, _bAttrName);
            moAttr = _bAttrName;
          } else if (!handle && typeof __zw_remove_attr === 'function') {
            __zw_remove_attr(sel, _bAttrName);
            moAttr = _bAttrName;
          }
        } else if (p === 'autofocus' || p === 'draggable' || p === 'spellcheck' || p === 'translate' || p === 'inert' || p === 'autocomplete') {
          // reflected 布尔/枚举全局属性（R2848/R2850）：autofocus/draggable/spellcheck/translate（R2848）
          // + inert/autocomplete（R2850）。autofocus/inert=boolean presence（truthy 设空值 / falsy 真移除）；
          // autocomplete=enumerated 串（任意值写 attr）；draggable/spellcheck="true"/"false"；translate="yes"/"no"。
          var rc4 = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
          if (p === 'autofocus' || p === 'inert') {
            var bsv = !!value;
            rc4[p] = bsv;
            if (bsv) {
              if (handle) __zw_set_attr_handle(handle, p, '');
              else { __zw_set_attr(sel, p, ''); moAttr = p; }
            } else if (!handle && typeof __zw_remove_attr === 'function') {
              __zw_remove_attr(sel, p); moAttr = p;
            }
          } else if (p === 'autocomplete') {
            rc4[p] = String(value);
            if (handle) __zw_set_attr_handle(handle, 'autocomplete', String(value));
            else { __zw_set_attr(sel, 'autocomplete', String(value)); moAttr = 'autocomplete'; }
          } else {
            var sv = !!value;
            rc4[p] = sv;
            var attrV = (p === 'translate') ? (sv ? 'yes' : 'no') : (sv ? 'true' : 'false');
            if (handle) __zw_set_attr_handle(handle, p, attrV);
            else { __zw_set_attr(sel, p, attrV); moAttr = p; }
          }
        } else if ((p === 'width' || p === 'height') && (_realTag(sel, handle) === 'IMG' || _realTag(sel, handle) === 'IFRAME' || _realTag(sel, handle) === 'CANVAS')) {
          // reflected unsigned-long 维度 setter（R2851）：parseInt 归一（NaN/负 → 0）→ 缓存数值 + 写 width/height
          // 内容属性（getter 优先读缓存保 sync set→get）。R3077：CANVAS width/height 反射（保 set→get 一致）。
          // R3308：CANVAS 设 width/height 触发 bitmap resize（HTML spec §4.12.5.1——清空 bitmap + 重置绘图状态）。
          // 已 getContext 的 canvas，调 host resizeContext 清空像素 + 重置 context 状态到默认。
          var wv = parseInt(value, 10);
          if (isNaN(wv) || wv < 0) wv = 0;
          var wrc = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
          wrc[p] = wv;
          if (handle) __zw_set_attr_handle(handle, p, String(wv));
          else { __zw_set_attr(sel, p, String(wv)); moAttr = p; }
          // CANVAS resize：取该元素的 context handle（_zwCanvasCtx[key] standalone 或 el._ctx），调 host resizeContext。
          // R3254-C12：transferControlToOffscreen 后（_transferred）DOM canvas 尺寸 setter 不触达
          // 共享 handle——offscreen 的 bitmap/状态不被 DOM canvas resize 波及（spec：control 已转交）。
          if (_realTag(sel, handle) === 'CANVAS'
              && !((_reflectedAttrs[key] || {})._transferred)
              && typeof __zw_canvas_op === 'function') {
            var _cctx = (typeof _zwCanvasCtx !== 'undefined' && _zwCanvasCtx[key]) || null;
            var _ch = _cctx && _cctx._handle;
            if (_ch) {
              // R3254-C6：只设一维时另一维取**元素 HTML 属性**（`<canvas width="500">` 从未经
              // JS 设置）——此前回退硬编码 300/150，host bitmap 与元素报告尺寸永久脱钩。
              // wrc 用 `!= null` 判断（显式设 0 合法，`|| 300` 会把 0 误回退）。
              var _attrW = handle ? __zw_get_attr_handle(handle, 'width') : __zw_get_attr(sel, 'width');
              var _attrH = handle ? __zw_get_attr_handle(handle, 'height') : __zw_get_attr(sel, 'height');
              var _cw = (p === 'width') ? wv : (wrc.width != null ? wrc.width : (parseInt(_attrW, 10) || 300));
              var _chh = (p === 'height') ? wv : (wrc.height != null ? wrc.height : (parseInt(_attrH, 10) || 150));
              __zw_canvas_op(String(_ch), 'resizeContext', String(_cw), String(_chh));
            }
          }
        } else if (p === 'scrollTop' || p === 'scrollLeft') {
          // R3047：scrollTop/scrollLeft setter（headless 无真滚动 → JS-side 状态追踪）。Number 归一（NaN/负 → 0）→
          // 存 `_scrollOffsets[key]`（与 scrollTo/scrollBy/getter 自洽）。无 moAttr（非内容属性）。不写 attr（旧 generic
          // fallthrough 误写 scrolltop="50" 垃圾属性）。
          var _sv = Number(value);
          if (isNaN(_sv) || _sv < 0) _sv = 0;
          var _sss = _scrollOffsets[key] || (_scrollOffsets[key] = { top: 0, left: 0 });
          if (p === 'scrollTop') _sss.top = _sv; else _sss.left = _sv;
          _zwFireScroll(key, sel, handle);
        } else if ((_realTag(sel, handle) === 'A' || _realTag(sel, handle) === 'AREA') &&
                   (p === 'protocol' || p === 'hostname' || p === 'host' || p === 'port' ||
                    p === 'pathname' || p === 'search' || p === 'hash' ||
                    p === 'username' || p === 'password')) {
          // R3070：HTMLAnchorElement/HTMLAreaElement URL 分解组件 setter（闭合 R2838 限制）。
          // https://html.spec.whatwg.org/multipage/links.html#htmlhyperlinkelementutils
          // real browser：读当前 href → 经组件 setter（set_scheme/set_host/set_path/set_query/...）替换该组件 →
          // 写回**新 href** 内容属性（getter part03.js:1627 重新分解该 href 取组件）。spec-correct via host
          // `__zw_set_url_part`（url crate setters：percent-encoding / IDNA / 默认端口归一）。组件无独立内容属性——
          // 仅写 href（moAttr='href'）。旧 R2838 限制：组件 setter 经 catch-all 写 spurious 属性（R3069 后入 expando），
          // getter 读 href 不受影响 → set→get round-trip 断（`a.pathname='/x'; a.pathname` 不变）。本切片接通。
          // lenient：无当前 href / host 回调缺 / set 失败（空 base 等）→ 静默不写（防破脚本，同 getter 空值回落）。
          // 注：`a.href=` setter 仍走 R3069 reflected 分支写 raw 值（getter 经 base 解析返绝对 URL——getAttribute 返 raw），
          // 故 href 不入此组件分支（条件不含 'href'）。
          var _uhCur = handle ? __zw_get_attr_handle(handle, 'href') : __zw_get_attr(sel, 'href');
          if (_uhCur && typeof __zw_set_url_part === 'function') {
            try {
              var _uhJson = __zw_set_url_part(_uhCur, p, String(value));
              if (_uhJson) {
                var _uhNewHref = JSON.parse(_uhJson).href;
                if (handle) __zw_set_attr_handle(handle, 'href', _uhNewHref);
                else __zw_set_attr(sel, 'href', _uhNewHref);
                moAttr = 'href';
              }
            } catch (_e) {}
          }
        } else if (typeof value !== 'string' && typeof value !== 'number' && typeof value !== 'boolean') {
          // R3042：expando 属性（非原始值——function/object/array/null/undefined/symbol/bigint）。旧经 generic fallthrough
          // 写垃圾内容属性（`__zw_set_attr(sel, p, '[object Object]')` / 'function(){}'）且 get 读不回（undefined）。
          // real browser：expando 存于 JS 对象非内容属性。改存 per-element expando map（get trap 读回）。
          // 仅非原始值——real reflected/special attr setter 永不收非原始值（string/number/boolean 走 generic fallthrough 不变），
          // 故零回归风险（不会拦截 role/aria/class/value 等任何真属性 setter）。无 moAttr（expando 非内容属性，不发 attributes MO）。
          var _ex = _expando[key] || (_expando[key] = {});
          _ex[p] = value;
        } else if (_reflectedStringAttr(p) || _REFLECTED_UINT[p] || p === 'size' || p === 'href' || p === 'label') {
          // R3069：reflected 原始属性——get trap 经 `_reflectedStringAttr`（type/name/placeholder/...）/ `_REFLECTED_UINT`
          //（colSpan/rowSpan/maxLength/cols/rows/start）/ `size` 专用分支读内容属性，故 set 须继续写属性（非 expando），
          // 否则 set 写 expando / get 读空属性 round-trip 断。复用 get trap 同源检测函数，自动一致（无静态名表维护）。
          // 写**reflected 属性名**（小写：_REFLECTED_UINT[p].a / _reflectedStringAttr(p) 返值）非 IDL 名 p——旧 generic
          // fallthrough 写 p（如 'colSpan' 大写）与 get 读小写 'colspan' 不匹配，property set→read round-trip 本就断；
          // 本分支写正确属性名，顺带修 colSpan/rowSpan/cols/rows 等 property-set round-trip。bool reflected（required/...）
          // 有显式 set 分支（part05.js 上方），不经此 fallthrough。
          // R3069-fix：`href`（A/AREA/LINK/BASE，get trap URL 分解读 href 属性，part03.js:1627）+ `label`（OPTION，get 读
          // label 属性 part03.js:1819）无专用 set 分支，旧靠 generic fallthrough 写属性；R3069 expando 改造后须显式列入
          // 此分支保 round-trip（否则 href setter 不写属性 → a.href get 读空，回归 test_anchor_url_decomposition_r2838）。
          // 写同名内容属性（href→'href' / label→'label'，1:1 小写），匹配旧 generic fallthrough 对所有元素的行为（无 tag
          // gate——非 A/AREA 设 href 亦写属性，与旧行为一致，无害）。
          var _refAttr = _REFLECTED_UINT[p] ? _REFLECTED_UINT[p].a : (_reflectedStringAttr(p) || p);
          if (handle) __zw_set_attr_handle(handle, _refAttr, String(value));
          else __zw_set_attr(sel, _refAttr, String(value));
          moAttr = _refAttr;
        } else {
          // R3069：非 reflected 原始值 → expando（闭合 R3042 限制①）。real browser：自定义原始属性存 JS 对象非内容属性，
          // IDL 读回真值。旧 generic fallthrough 写内容属性但 get trap 无分支读 → 读返 undefined（correctness bug：
          // `el.flag='x'; el.flag` → undefined）。改存 _expando（get trap part04.js:1277 已读 _expando）。存 raw value
          // 保类型（number/boolean 非 string）。不发 attributes MO（expando 非内容属性）；不写属性（real browser 亦不写）。
          var _ex2 = _expando[key] || (_expando[key] = {});
          _ex2[p] = value;
        }
        if (moAttr) _mo_notify(sel, handle, { type: 'attributes', attributeName: moAttr });
        return true;
      },
      // R3046：expando 枚举表面（R3042 follow-up，闭合 R3042 已知限制④）。无此三 trap → `Object.keys(el)` /
      // `for...in` / `'foo' in el` / `Object.assign({}, el)` / 解构展开 不含 expando（proxy target {} 空，
      // default ownKeys 返 []、has 走 target 原型链、getOwnPropertyDescriptor 返 undefined）。补三 trap 暴露
      // expando 为 enumerable own 属性，real browser expando 语义。仅 expando（_expando[key] hasOwn 命中）暴露；
      // 非 expando 落 default（`prop in _t` / target 自身键 [] / undefined）——real reflected/special 属性经 get trap
      // 读但非 target own，`'id' in el` 仍 false（pre-existing，documented）。has/ownKeys 不变量：target {} 无 own → 无约束。
      has: function(_t, prop) {
        var _exH = _expando[key];
        return !!(_exH && Object.prototype.hasOwnProperty.call(_exH, prop)) || (prop in _t);
      },
      ownKeys: function() {
        var _exO = _expando[key];
        return _exO ? Object.keys(_exO) : [];
      },
      getOwnPropertyDescriptor: function(_t, prop) {
        var _exD = _expando[key];
        if (_exD && Object.prototype.hasOwnProperty.call(_exD, prop)) {
          return { value: _exD[prop], writable: true, enumerable: true, configurable: true };
        }
        return undefined;
      },
      // js-dom M4 R10：`instanceof` 原型链（spec `Node` / `Element` / `HTMLElement` 子类）——polyfill Proxy
      // 默认走 target({})原型（Object.prototype），`el instanceof Element/Node` 恒 false（createElement/cloneNode
      // 用例 89 instanceof 块）。按节点类型返对应 prototype：element→HTMLElement.prototype（链 Element→Node，覆盖
      // 绝大多数 instanceof Element/HTMLElement/Node；具体 HTMLDivElement 等子类 instanceof 留扩展）、
      // text/comment→Node、PI→ProcessingInstruction、fragment→DocumentFragment。
      // **安全**：getPrototypeOf 仅影响 `instanceof` / `Object.getPrototypeOf` / 解构原型查找，不影响 get/set（属性读
      // 仍走 get trap）。返构造器缺失时回落 Object.prototype（零回归）。
      getPrototypeOf: function(_t) {
        var _gp = globalThis;
        // 节点类型判定：PI/fragment/comment/text 经 handle set；element 为默认（无 set 的 selector/handle 节点）。
        if (handle && _piHandles[handle] && _gp.ProcessingInstruction) return _gp.ProcessingInstruction.prototype;
        if (handle && _fragmentHandles[handle] && _gp.DocumentFragment) return _gp.DocumentFragment.prototype;
        if (handle && _commentHandles[handle] && _gp.Node) return _gp.Node.prototype;
        if (handle && _textHandles[handle] && _gp.Node) return _gp.Node.prototype;
        // element（含 selector-based 与 createElement handle）：按 tag 查 __zwHtmlTagIface 返对应
        // HTML*Element 子类 prototype（R11，使 `el instanceof HTMLDivElement` 等为 true）；无映射/构造器
        // 缺失回落 HTMLElement.prototype（链 Element → Node）。
        if (_gp.HTMLElement && _gp.HTMLElement.prototype) {
          var _iface = _gp.__zwHtmlTagIface && _gp.__zwHtmlTagIface[_realTag(sel, handle).toLowerCase()];
          if (_iface && _gp[_iface] && _gp[_iface].prototype) return _gp[_iface].prototype;
          return _gp.HTMLElement.prototype;
        }
        return Object.prototype;
      }
    });
    _proxyCache[key] = proxy;
    return proxy;
  }

  function _wrapSelector(sel) {
    return _makeProxy(sel, null);
  }

  function _wrapHandle(handle) {
    return _makeProxy(null, handle);
  }

  // R2926 Shadow DOM：抛 DOMException（无 DOMException 环境回落 Error + name）。
  function _throwDom(name, msg) {
    if (typeof DOMException === 'function') throw new DOMException(msg, name);
    var e = new Error(msg);
    e.name = name;
    throw e;
  }

  // R3254-C1：构造 DOMException（无 DOMException 环境回落 Error + name）——reject 用。
  function _zwDomException(msg, name) {
    if (typeof DOMException === 'function') return new DOMException(msg, name);
    var e = new Error(msg);
    e.name = name;
    return e;
  }

  // R2926 Shadow DOM：`element.attachShadow(init)` → ShadowRoot。host 元素专用（get trap 仅对元素
  // 暴露 attachShadow，fragment/comment/text/shadow 不暴露）。spec：init.mode 须 'open'/'closed'
  //（否则 TypeError）；host 已挂 shadow → NotSupportedError。复用 DocumentFragment handle 容器
  //（`__zw_create_document_fragment`）建 root——appendChild/innerHTML/childNodes 经 fragment 机制工作；
  // shadow 内容**不渲染**（渲染管线走 flat dom_html，不遍历 shadow 树——fidelity defer，同 detached fragment）。
  // 返 ShadowRoot proxy（nodeType 11 / nodeName '#shadow-root' / host / mode，经 _shadowHandles 标识）。
  function _attachShadow(sel, handle, init) {
    var key = _elKey(sel, handle);
    if (_shadowRoots[key]) {
      _throwDom('NotSupportedError',
        "Failed to execute 'attachShadow' on 'Element': Shadow root cannot be created on a host which already hosts a shadow tree.");
    }
    if (init == null || typeof init !== 'object') {
      throw new TypeError("Failed to execute 'attachShadow' on 'Element': parameter 1 is not of type 'object'.");
    }
    var mode = String(init.mode);
    if (mode !== 'open' && mode !== 'closed') {
      throw new TypeError("Failed to execute 'attachShadow' on 'Element': member mode is required and must be 'open' or 'closed'.");
    }
    if (typeof __zw_create_document_fragment !== 'function') return null;
    var rootHandle = __zw_create_document_fragment();
    if (!rootHandle) return null;
    // shadow handle 入两 set：_fragmentHandles（继承 fragment appendChild/innerHTML/childNodes 行为）
    // + _shadowHandles（shadow-root 身份：'#shadow-root' / host / mode）。
    _fragmentHandles[rootHandle] = true;
    _shadowHandles[rootHandle] = true;
    _shadowHandleMeta[rootHandle] = { hostSel: sel, hostHandle: handle, mode: mode };
    _shadowRoots[key] = { handle: rootHandle, mode: mode };
    return _wrapHandle(rootHandle);
  }

  // R2927 handle-children registry 辅助（容器 = shadow root / fragment handle）。这些容器无 selector，
  // 既有 childNodes/children 经 `__zw_child_nodes(sel)` 读（须 sel）恒返 []——registry 在 appendChild
  // 时同步记录子节点，使容器子树可观察（解锁 imperative custom-element shadow 构建模式自测）。
  function _isContainerHandle(h) {
    return !!(h && (_shadowHandles[h] || _fragmentHandles[h]));
  }
  // 容器的子节点 proxy 列表（registry 未建 → 空）。
  function _handleChildNodes(h) {
    return (h && _handleChildren[h]) ? _handleChildren[h] : [];
  }
  // 容器的**元素**子 proxy 列表（过滤 text/comment，按 nodeType）。
  function _handleElementChildren(h) {
    var kids = _handleChildNodes(h);
    var out = [];
    for (var i = 0; i < kids.length; i++) {
      var k = kids[i];
      if (k && k.nodeType === 1) out.push(k);
    }
    return out;
  }
  // 记录 child 进容器 parent 的 registry。child 为 fragment 时 flatten 其 registry 子节点（并清空
  // child registry，spec：fragment append 后清空）。仅 handle-based child 可记录。
  function _recordHandleChild(parentHandle, child) {
    if (!parentHandle || !child || !child.__zwHandle) return;
    var arr = _handleChildren[parentHandle] || (_handleChildren[parentHandle] = []);
    if (_fragmentHandles[child.__zwHandle]) {
      // fragment flatten：移入 fragment 的已记录子节点，清空 fragment registry。
      var fkids = _handleChildren[child.__zwHandle];
      if (fkids) {
        for (var i = 0; i < fkids.length; i++) arr.push(fkids[i]);
        _handleChildren[child.__zwHandle] = [];
      }
    } else {
      arr.push(child);
    }
  }
  // 从容器 registry 移除 child（removeChild 用）。
  function _unrecordHandleChild(parentHandle, child) {
    if (!parentHandle || !child || !child.__zwHandle) return;
    var arr = _handleChildren[parentHandle];
    if (!arr) return;
    var ch = child.__zwHandle;
    _handleChildren[parentHandle] = arr.filter(function(k) { return !k || k.__zwHandle !== ch; });
  }

  // R2928 handle 子树 querySelector/querySelectorAll——JS 端 registry 树搜索 + 客户端选择器匹配。
  // handle 元素（createElement / shadow root / fragment，无 sel）的子树查询无法走 host
  // `__zw_query_match_sub(sel, q)`（须 sel）。R2927 registry 记录 handle 父→子 proxy 列表，此处 DFS
  // 遍历 + 客户端 compound / 后代组合器 / 逗号列表 匹配。覆盖 shadow 构建模式自测（Lit `sr.querySelector('#x')`）。
  // 支持范围：tag / `*` / `#id` / `.class`（可多个） / `[attr]` + 6 运算符（= ~= |= ^= $= *=） / 复合 /
  // **四种组合器**（后代 ` ` / 子代 `>` / 相邻兄弟 `+` / 通用兄弟 `~`，与 DOM crate query.rs 对齐 R3286）/
  // 逗号列表。不支持（该组静默跳过，不抛）：伪类（`:host`/`:hover`/...）、伪元素——遇之标记
  // unsupported，逗号列表中其余组仍可匹配；全部 unsupported → 无匹配（返 null/[]）。所有 proxy 属性读
  // 经 try/catch（host 未注册 / 异常 → 安全回落）。

  // 从 proxy 安全读属性（host 未注册 / 异常 → dflt），不抛回用户脚本。
  function _hSafe(fn, dflt) { try { var v = fn(); return v == null ? dflt : v; } catch (_e) { return dflt; } }
  function _hTagOf(p) { return String(_hSafe(function () { return p.tagName; }, '')).toUpperCase(); }
  function _hIdOf(p) { return String(_hSafe(function () { return p.id; }, '')); }
  function _hClassesOf(p) {
    var c = String(_hSafe(function () { return p.className; }, ''));
    return c ? c.split(/\s+/).filter(Boolean) : [];
  }
  function _hAttrOf(p, name) { return _hSafe(function () { return p.getAttribute(name); }, null); }

  // 解析单个属性选择器内部 `name` / `name op val`（val 去引号）。不匹配 → null。
  function _parseAttrInner(inner) {
    var m = inner.match(/^\s*([\w:-]+)\s*(?:([~|^$*]?=)\s*(.*?))?\s*$/);
    if (!m) return null;
    var val = m[3];
    if (val != null) val = String(val).replace(/^['"]|['"]$/g, '');
    return { name: m[1], op: m[2] || null, val: val == null ? '' : val };
  }
  function _matchAttrOf(p, a) {
    var av = _hAttrOf(p, a.name);
    if (a.op === null) return av != null;
    if (av == null) return false;
    var v = String(av);
    switch (a.op) {
      case '=': return v === a.val;
      case '~=': return a.val !== '' && v.split(/\s+/).indexOf(a.val) >= 0;
      case '|=': return v === a.val || v.indexOf(a.val + '-') === 0;
      case '^=': return a.val !== '' && v.indexOf(a.val) === 0;
      case '$=': return a.val !== '' && v.length >= a.val.length &&
        v.lastIndexOf(a.val) === v.length - a.val.length;
      case '*=': return a.val !== '' && v.indexOf(a.val) >= 0;
    }
    return false;
  }
  // 读 compound 内裸 token（tag/id/class 名），遇 `.`/`#`/`[`/`:`/空白停。
  function _readCompoundToken(text, start) {
    var i = start, n = text.length;
    while (i < n) {
      var ch = text[i];
      if (ch === '.' || ch === '#' || ch === '[' || ch === ':' || /\s/.test(ch)) break;
      i++;
    }
    return text.substring(start, i);
  }
  // 解析单个复合选择器（无空白组合器）。返 { tag, ids[], classes[], attrs[], unsupported }。
  // tag 为 null（任意 / `*`）或大写 tag。遇 `:`（伪类/伪元素）/ 空裸 token / 第二个裸 token → unsupported。
  function _parseCompoundOf(text) {
    var c = { tag: null, ids: [], classes: [], attrs: [], unsupported: false };
    var i = 0, n = text.length, seenTag = false;
    while (i < n) {
      var ch = text[i];
      if (ch === ':') { c.unsupported = true; break; }
      if (ch === '.') {
        var cls = _readCompoundToken(text, i + 1);
        if (!cls) { c.unsupported = true; break; }
        c.classes.push(cls); i += 1 + cls.length;
      } else if (ch === '#') {
        var idt = _readCompoundToken(text, i + 1);
        if (!idt) { c.unsupported = true; break; }
        c.ids.push(idt); i += 1 + idt.length;
      } else if (ch === '[') {
        var end = text.indexOf(']', i);
        if (end < 0) { c.unsupported = true; break; }
        var am = _parseAttrInner(text.substring(i + 1, end));
        if (!am) { c.unsupported = true; break; }
        c.attrs.push(am); i = end + 1;
      } else if (/\s/.test(ch)) {
        i++;
      } else {
        var tg = _readCompoundToken(text, i);
        if (!tg) { i++; continue; }
        if (!seenTag) { c.tag = tg === '*' ? null : tg.toUpperCase(); seenTag = true; }
        else { c.unsupported = true; break; }
        i += tg.length;
      }
    }
    return c;
  }
  function _matchCompoundOf(p, c) {
    if (c.tag && _hTagOf(p) !== c.tag) return false;
    if (c.ids.length) {
      var pid = _hIdOf(p);
      for (var k = 0; k < c.ids.length; k++) if (pid !== c.ids[k]) return false;
    }
    if (c.classes.length) {
      var cls = _hClassesOf(p);
      for (var k = 0; k < c.classes.length; k++) if (cls.indexOf(c.classes[k]) < 0) return false;
    }
    if (c.attrs.length) {
      for (var k = 0; k < c.attrs.length; k++) if (!_matchAttrOf(p, c.attrs[k])) return false;
    }
    return true;
  }
  // 按**四种组合器**拆 complex，跳过 `[...]` / 引号内的组合器边界。
  // 返 { compounds: [compound-str...], combinators: [comb...] }——`combinators[i]` 连接
  // `compounds[i]` 与 `compounds[i+1]`，取值 `' '`（后代）/`'>'`（子代）/`'+'`（相邻兄弟）/
  // `'~'`（通用兄弟），与 DOM crate query.rs::Combinator 四变体对齐（R3286）。
  // 显式符号（>`+`~`）覆盖空白触发的后代边界（如 `a + b` 中 `+` 前空白不可先记为后代）。
  function _splitComplex(text) {
    var compounds = [], combinators = [];
    var cur = '', depth = 0, quote = null;
    // lastSegmentChar：上一字节是否为段内普通字符（非边界/非边界后空白）。
    // pendingExplicit：上一显式符号（遇段字符时落边界），覆盖空白后代。
    var lastSegmentChar = false, pendingExplicit = null;
    var flush = function (comb) {
      if (cur) { compounds.push(cur); combinators.push(comb); cur = ''; }
      lastSegmentChar = false;
    };
    for (var i = 0; i < text.length; i++) {
      var ch = text[i];
      if (quote) { cur += ch; if (ch === quote) quote = null; lastSegmentChar = true; continue; }
      if (ch === '"' || ch === "'") { quote = ch; cur += ch; lastSegmentChar = true; continue; }
      if (ch === '[') { depth++; cur += ch; lastSegmentChar = true; continue; }
      if (ch === ']') { depth--; cur += ch; lastSegmentChar = true; continue; }
      // 括号 `(``)` 亦计入深度——`:nth-child(2n+1)` / `:not(.a)` / `:is(a, b)` 内的 `+`/` `/`,`
      // 非组合器边界（R3288 修复：旧仅计 `[]` 致 nth 公式 an+b 的 `+` 误判为相邻兄弟组合器）。
      if (ch === '(') { depth++; cur += ch; lastSegmentChar = true; continue; }
      if (ch === ')') { depth--; cur += ch; lastSegmentChar = true; continue; }
      if (depth === 0 && (ch === '>' || ch === '+' || ch === '~')) {
        // 显式符号覆盖：若紧前的边界是空白触发的后代且其间无段字符（符号紧随空白），
        // 改写最后一个组合器为该显式符号。
        if (!lastSegmentChar && combinators.length > 0 && combinators[combinators.length - 1] === ' ') {
          combinators[combinators.length - 1] = ch;
        } else {
          pendingExplicit = ch;
        }
        lastSegmentChar = false;
        continue;
      }
      if (depth === 0 && /\s/.test(ch)) {
        if (lastSegmentChar && pendingExplicit === null) { flush(' '); }
        // 否则跳过连续空白 / 显式符号后的空白（pendingExplicit 保留待覆盖）。
        continue;
      }
      // 段内普通字符：若有 pending 显式符号，落边界为该符号。
      if (pendingExplicit !== null) { flush(pendingExplicit); pendingExplicit = null; }
      cur += ch;
      lastSegmentChar = true;
    }
    // 末段冲刷（不附组合器——combinators 比 compounds 少一）。
    if (cur) { compounds.push(cur); }
    // 末尾 pending 显式符号 → 选择器以组合器结尾（非法），丢弃末空段不影响（无末 compound）。
    return compounds.length ? { compounds: compounds, combinators: combinators.slice(0, compounds.length - 1) } : null;
  }
  function _parseComplexOf(text) {
    var parts = _splitComplex(text);
    if (!parts) return null;
    var out = [];
    for (var i = 0; i < parts.compounds.length; i++) {
      var c = _parseCompoundOf(parts.compounds[i]);
      if (c.unsupported) return null;
      out.push(c);
    }
    return out.length ? { compounds: out, combinators: parts.combinators } : null;
  }
  // 逗号列表拆分（跳过 `[...]` / 引号内逗号）。
  function _splitSelectorListOf(sel) {
    var out = [], cur = '', depth = 0, quote = null;
    for (var i = 0; i < sel.length; i++) {
      var ch = sel[i];
      if (quote) { cur += ch; if (ch === quote) quote = null; continue; }
      if (ch === '"' || ch === "'") { quote = ch; cur += ch; continue; }
      if (ch === '[') depth++;
      if (ch === ']') depth--;
      if (ch === ',' && depth === 0) { out.push(cur); cur = ''; continue; }
      cur += ch;
    }
    out.push(cur);
    return out;
  }
  // 解析选择器列表 → complex 数组（unsupported 组静默跳过；可能为空）。
  function _parseSelectorListOf(sel) {
    var groups = _splitSelectorListOf(String(sel));
    var out = [];
    for (var i = 0; i < groups.length; i++) {
      var c = _parseComplexOf(groups[i]);
      if (c) out.push(c);
    }
    return out;
  }
  // 四组合器匹配（` `/`>`/`+`/`~`，与 DOM crate query.rs 对齐 R3286）：从最右 compound
  //（匹配候选 p）起，向左逐段求值。每段 combinator 决定如何从「当前节点」回溯到「左侧 compound
  // 应匹配的节点」。` `（后代）与 `~`（通用兄弟）须**回溯**——选定的目标节点不仅自身匹配本段 compound，
  // 还须能继续回溯左侧剩余链（否则换一个匹配候选再试，如 `h1 + p ~ p` 对 p3：`~` 可选 p2/p1，
  // p2 的 `+` 前置非 h1 失败 → 回溯试 p1，p1 紧邻 h1 成功）。
  //   `' '`（后代）：沿 parent 链逐祖先试。
  //   `'>'`（子代）：直接元素父须匹配（无回溯）。
  //   `'+'`（相邻兄弟）：紧邻前一元素兄弟须匹配（无回溯）。
  //   `'~'`（通用兄弟）：沿在先元素兄弟链逐个试（近→远）。
  // `nodeInfo` 由 _handleSubtreeNodes 预计算（含 ancestors / parent / prevSibling / prevSiblings）。
  function _matchComplexAgainst(p, complex, nodeInfo) {
    var compounds = complex.compounds, combs = complex.combinators;
    return _matchChainFrom(compounds, combs, compounds.length - 1, p, nodeInfo);
  }
  // 从右往左匹配 compound[ci..0]：当前节点（info.proxy）须匹配 compound[ci]，再按 combs[ci-1]
  // 回溯到 compound[ci-1] 的候选节点。ci < 0 表示全部匹配 → 成功。
  function _matchChainFrom(compounds, combs, ci, _curProxy, info) {
    if (!_matchCompoundOf(info.proxy, compounds[ci])) return false;
    if (ci === 0) return true; // 已匹配到最左 compound
    var comb = combs[ci - 1];
    var left = ci - 1;
    if (comb === ' ') {
      // 后代：沿 parent/ancestor 链逐个试（回溯）。
      for (var ai = info.ancestors.length - 1; ai >= 0; ai--) {
        if (_matchCompoundOf(info.ancestors[ai].proxy, compounds[left])
          && _matchChainFrom(compounds, combs, left, info.ancestors[ai].proxy, info.ancestors[ai])) {
          return true;
        }
      }
      return false;
    } else if (comb === '>') {
      // 子代：直接元素父须匹配 + 继续回溯。
      if (info.parentInfo
        && _matchCompoundOf(info.parentInfo.proxy, compounds[left])
        && _matchChainFrom(compounds, combs, left, info.parentInfo.proxy, info.parentInfo)) {
        return true;
      }
      return false;
    } else if (comb === '+') {
      // 相邻兄弟：紧邻前一元素兄弟须匹配 + 继续回溯。
      if (info.prevSiblingInfo
        && _matchCompoundOf(info.prevSiblingInfo.proxy, compounds[left])
        && _matchChainFrom(compounds, combs, left, info.prevSiblingInfo.proxy, info.prevSiblingInfo)) {
        return true;
      }
      return false;
    } else if (comb === '~') {
      // 通用兄弟：沿在先元素兄弟链逐个试（回溯，近→远）。
      for (var si = info.prevSiblings.length - 1; si >= 0; si--) {
        var ps = info.prevSiblings[si];
        if (_matchCompoundOf(ps.proxy, compounds[left])
          && _matchChainFrom(compounds, combs, left, ps.proxy, ps)) {
          return true;
        }
      }
      return false;
    }
    return false;
  }
  function _matchAnyGroup(p, groups, nodeInfo) {
    for (var i = 0; i < groups.length; i++) {
      if (_matchComplexAgainst(p, groups[i], nodeInfo)) return true;
    }
    return false;
  }
  // DFS 收集 rootHandle 子树全部**元素** proxy（document order）+ 各自节点信息（祖先链、
  // 元素父、紧邻前一元素兄弟、在先元素兄弟链）。兄弟组合器（`+`/`~`）只计元素兄弟（R3286）。
  // 每节点 nodeInfo = { proxy, parent, parentInfo, prevSibling, prevSiblingInfo,
  //   prevSiblings: [sibling-nodeInfo...]（文档序，近→远需反转读取）, ancestors: [node-info...]（根→父） }。
  function _handleSubtreeNodes(rootHandle) {
    var result = [];
    // 作用域根元素（el.querySelector 的 el）本身可作为后代/子代/兄弟组合器的匹配目标
    //（如 `__root.querySelector('div > p')` 中 `div` 匹配 __root 自身），但它**不作为查询候选结果**
    //（querySelector 仅返后代）。故建 rootInfo 作为顶层子的 parent/ancestor，但不入 result。
    var rootProxy = _wrapHandle(rootHandle);
    var rootInfo = {
      proxy: rootProxy,
      parent: null,
      parentInfo: null,
      prevSibling: null,
      prevSiblingInfo: null,
      prevSiblings: [],
      ancestors: [],
    };
    // 代理 → 已计算 nodeInfo 映射（文档序先建后用，使兄弟/父目标重用完整上下文，
    // 支持组合器链回溯如 `div > h1 + p` 的 `+` 后再 `>`）。
    var infoByProxy = new Map();
    function nodeInfoOf(px) { return infoByProxy.get(px) || null; }
    function visit(handle, parentInfo, ancestors) {
      var kids = _handleChildren[handle];
      if (!kids) return;
      // 先过滤出本层**元素**子（跳过 text/comment），保文档序——供兄弟上下文。
      var elemKids = [];
      for (var i = 0; i < kids.length; i++) {
        var k = kids[i];
        if (k && _hSafe(function () { return k.nodeType; }, 0) === 1) elemKids.push(k);
      }
      for (var j = 0; j < elemKids.length; j++) {
        var p = elemKids[j];
        // 紧邻前一元素兄弟 = elemKids[j-1]；在先元素兄弟 = elemKids[0..j-1]（文档序）。
        var prevSib = j > 0 ? elemKids[j - 1] : null;
        var info = {
          proxy: p,
          parent: parentInfo ? parentInfo.proxy : null,
          parentInfo: parentInfo,
          prevSibling: prevSib,
          prevSiblingInfo: null,
          prevSiblings: [],
          ancestors: ancestors,
        };
        infoByProxy.set(p, info);
        // 填充兄弟引用（此时左侧兄弟均已 visit 过，已登记）。
        if (prevSib) info.prevSiblingInfo = nodeInfoOf(prevSib);
        for (var pj = 0; pj < j; pj++) {
          var ni = nodeInfoOf(elemKids[pj]);
          if (ni) info.prevSiblings.push(ni);
        }
        result.push({ proxy: p, nodeInfo: info });
        var ph = _hSafe(function () { return p.__zwHandle; }, null);
        if (ph) {
          // 递归：本节点成为子层的 parentInfo；ancestors 链含本节点（根→…→父）。
          visit(ph, info, ancestors.concat([info]));
        }
      }
    }
    // 顶层：作用域根为 parentInfo + ancestors[0]，使 `div > p` 的 `div` 可匹配根自身。
    visit(rootHandle, rootInfo, [rootInfo]);
    return result;
  }
  function _handleQueryFirst(rootHandle, q) {
    var groups = _parseSelectorListOf(q);
    if (!groups.length) return null;
    var nodes = _handleSubtreeNodes(rootHandle);
    for (var i = 0; i < nodes.length; i++) {
      if (_matchAnyGroup(nodes[i].proxy, groups, nodes[i].nodeInfo)) return nodes[i].proxy;
    }
    return null;
  }
  function _handleQueryAll(rootHandle, q) {
    var groups = _parseSelectorListOf(q);
    if (!groups.length) return [];
    var nodes = _handleSubtreeNodes(rootHandle);
    var out = [];
    for (var i = 0; i < nodes.length; i++) {
      if (_matchAnyGroup(nodes[i].proxy, groups, nodes[i].nodeInfo)) out.push(nodes[i].proxy);
    }
    return out;
  }

  // DOMMatrix（R2985）——4×4 变换矩阵（Canvas getTransform / 几何库 / CSSMatrix）。2D 为 6 元素 a,b,c,d,e,f
  //（= m11,m12,m21,m22,m41,m42）。构造：无参=单位；[6]=2D；[16]=4×4。属性 a-f + m11-m44（双向别名）。
  // 方法：multiply / multiplySelf / inverse / translate / scale / rotate / rotateSelf / transformPoint /
  // toFloat32Array / toFloat64Array / toJSON。静态 fromMatrix / fromFloat32Array / fromFloat64Array。
  // **已知限制**：3D 方法（rotateAxisAngle/skewX/skewY/rotateFromVector）按 2D 近似或 identity；3D 矩阵运算
  // 按 4×4 通用 multiply 正确，但 rotate/scale/translate 的 z 轴 2D 近似（headless Canvas 2D 为主）。
  function DOMMatrix(init) {
    var m = [1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1]; // 4×4 column-major：m[0]=m11,m[1]=m12,...,m[3]=m14,...
    if (init) {
      var a = Array.prototype.slice.call(init);
      if (a.length === 6) {
        // 2D [a,b,c,d,e,f] → 4×4。
        m = [a[0],a[1],0,0, a[2],a[3],0,0, 0,0,1,0, a[4],a[5],0,1];
      } else if (a.length === 16) {
        m = a.slice();
      }
    }
    this._m = m;
  }
  Object.defineProperty(DOMMatrix.prototype, 'a', { get: function () { return this._m[0]; }, set: function (v) { this._m[0] = +v; } });
  Object.defineProperty(DOMMatrix.prototype, 'b', { get: function () { return this._m[1]; }, set: function (v) { this._m[1] = +v; } });
  Object.defineProperty(DOMMatrix.prototype, 'c', { get: function () { return this._m[4]; }, set: function (v) { this._m[4] = +v; } });
  Object.defineProperty(DOMMatrix.prototype, 'd', { get: function () { return this._m[5]; }, set: function (v) { this._m[5] = +v; } });
  Object.defineProperty(DOMMatrix.prototype, 'e', { get: function () { return this._m[12]; }, set: function (v) { this._m[12] = +v; } });
  Object.defineProperty(DOMMatrix.prototype, 'f', { get: function () { return this._m[13]; }, set: function (v) { this._m[13] = +v; } });
  // m11..m44（4×4 行主序读：m11=m[0],m12=m[1],m13=m[2],m14=m[3],m21=m[4]...）。
  ['m11','m12','m13','m14','m21','m22','m23','m24','m31','m32','m33','m34','m41','m42','m43','m44'].forEach(function (name, i) {
    Object.defineProperty(DOMMatrix.prototype, name, { get: function () { return this._m[i]; }, set: function (v) { this._m[i] = +v; }, configurable: true });
  });
  DOMMatrix.prototype.multiply = function (other) { return _domMatrixMultiply(this, other); };
  DOMMatrix.prototype.multiplySelf = function (other) { this._m = _domMatrixMultiply(this, other)._m; return this; };
  DOMMatrix.prototype.inverse = function () { return _domMatrixInverse(this); };
  DOMMatrix.prototype.translate = function (tx, ty, tz) {
    return _domMatrixMultiply(this, _domMatrixFromTranslate(+tx || 0, +ty || 0, +tz || 0));
  };
  DOMMatrix.prototype.scale = function (sx, sy, sz) {
    var x = (sx == null) ? 1 : +sx; var y = (sy == null) ? x : +sy; var z = (sz == null) ? 1 : +sz;
    return _domMatrixMultiply(this, _domMatrixFromScale(x, y, z));
  };
  DOMMatrix.prototype.rotate = function (rx, _ry, _rz) {
    // DOMMatrix.rotate 自洽 2D：单参 = 绕 Z 轴度数（spec DOMMatrix rotate 旋转轴语义复杂，2D 取 Z）。
    return _domMatrixMultiply(this, _domMatrixFromRotateZ((+rx || 0) * Math.PI / 180));
  };
  DOMMatrix.prototype.rotateSelf = function (rx, ry, rz) { this._m = this.rotate(rx, ry, rz)._m; return this; };
  DOMMatrix.prototype.transformPoint = function (pt) {
    var x = (pt && pt.x != null) ? +pt.x : 0, y = (pt && pt.y != null) ? +pt.y : 0, z = (pt && pt.z != null) ? +pt.z : 0, w = (pt && pt.w != null) ? +pt.w : 1;
    var m = this._m;
    return new DOMPoint(m[0]*x + m[4]*y + m[8]*z + m[12]*w, m[1]*x + m[5]*y + m[9]*z + m[13]*w, m[2]*x + m[6]*y + m[10]*z + m[14]*w, m[3]*x + m[7]*y + m[11]*z + m[15]*w);
  };
  DOMMatrix.prototype.toFloat32Array = function () { return new Float32Array(this._m); };
  DOMMatrix.prototype.toFloat64Array = function () { return new Float64Array(this._m); };
  DOMMatrix.prototype.toJSON = function () { return this._m.slice(); };
  DOMMatrix.fromMatrix = function (other) { return new DOMMatrix(other && other._m ? other._m.slice() : []); };
  DOMMatrix.fromFloat32Array = function (a) { return new DOMMatrix(Array.prototype.slice.call(a)); };
  DOMMatrix.fromFloat64Array = function (a) { return new DOMMatrix(Array.prototype.slice.call(a)); };
  // 4×4 矩阵乘法（column-major _m）：C = self × other。
  function _domMatrixMultiply(self, other) {
    var a = self._m, b = other._m, r = new Array(16);
    for (var col = 0; col < 4; col++) {
      for (var row = 0; row < 4; row++) {
        r[col*4+row] = a[0+row]*b[col*4+0] + a[4+row]*b[col*4+1] + a[8+row]*b[col*4+2] + a[12+row]*b[col*4+3];
      }
    }
    var res = new DOMMatrix(); res._m = r; return res;
  }
  function _domMatrixFromTranslate(tx, ty, tz) {
    var m = new DOMMatrix(); m._m[12] = tx; m._m[13] = ty; m._m[14] = tz; return m;
  }
  function _domMatrixFromScale(sx, sy, sz) {
    var m = new DOMMatrix(); m._m[0] = sx; m._m[5] = sy; m._m[10] = sz; return m;
  }
  function _domMatrixFromRotateZ(rad) {
    var c = Math.cos(rad), s = Math.sin(rad), m = new DOMMatrix();
    m._m[0] = c; m._m[1] = s; m._m[4] = -s; m._m[5] = c; return m;
  }
  // 4×4 矩阵求逆（Gauss-Jordan，adjugate 通用法）。奇异 → identity（spec throw 在 DOMMatrixReadOnly，
  // mutable DOMMatrix.inverse 实测各浏览器返逆或 throw，此处近似 identity 不抛避脚本中断）。
  function _domMatrixInverse(mm) {
    var a = mm._m.slice(), inv = [1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1];
    for (var i = 0; i < 4; i++) {
      var piv = i;
      for (var r = i+1; r < 4; r++) { if (Math.abs(a[r*4+i]) > Math.abs(a[piv*4+i])) piv = r; }
      if (Math.abs(a[piv*4+i]) < 1e-12) return new DOMMatrix(); // 奇异 → identity
      if (piv !== i) { for (var c = 0; c < 4; c++) { var t1=a[i*4+c]; a[i*4+c]=a[piv*4+c]; a[piv*4+c]=t1; var t2=inv[i*4+c]; inv[i*4+c]=inv[piv*4+c]; inv[piv*4+c]=t2; } }
      var d = a[i*4+i];
      for (var c2 = 0; c2 < 4; c2++) { a[i*4+c2] /= d; inv[i*4+c2] /= d; }
      for (var r2 = 0; r2 < 4; r2++) {
        if (r2 !== i) { var f2 = a[r2*4+i]; for (var c3 = 0; c3 < 4; c3++) { a[r2*4+c3] -= f2*a[i*4+c3]; inv[r2*4+c3] -= f2*inv[i*4+c3]; } }
      }
    }
    var res = new DOMMatrix(); res._m = inv; return res;
  }
  globalThis.DOMMatrix = globalThis.DOMMatrix || DOMMatrix;
  // DOMPoint（R2985）——几何点（x/y/z/w），DOMMatrix.transformPoint 输入/输出。构造 + toJSON。
  function DOMPoint(x, y, z, w) { this.x = +x || 0; this.y = +y || 0; this.z = +z || 0; this.w = (w == null) ? 1 : +w; }
  DOMPoint.prototype.toJSON = function () { return { x: this.x, y: this.y, z: this.z, w: this.w }; };
  DOMPoint.fromPoint = function (p) { return new DOMPoint(p && p.x, p && p.y, p && p.z, p && p.w); };
  globalThis.DOMPoint = globalThis.DOMPoint || DOMPoint;

  // ImageData（R3297）——全局构造器（HTML ImageData spec）。两形式：
  //   `new ImageData(width, height)` → 透明黑（全零 RGBA）像素数组。
  //   `new ImageData(Uint8ClampedArray, width[, height])` → 包裹既有数据（高度由 数组长度/(width*4) 推导或显式）。
  // 产物 `{width, height, data: Uint8ClampedArray, colorSpace: 'srgb'}`——与 `ctx.createImageData` 输出同构
  //（putImageData/getImageData/Worker 内像素处理直消费此形状）。此前缺 → `new ImageData(...)` 抛 TypeError。
  // spec 校验：data 须为 Uint8ClampedArray；width*4 须整除 data.length；非法 → lenient 返全零（headless 不中断脚本，
  // real browser 抛 IndexSizeError，与 btoa/roundRect lenient 哲学一致）。colorSpace 仅 'srgb'（'display-p3' defer）。
  // https://html.spec.whatwg.org/multipage/canvas.html#imagedata
  function ImageData(a, b, c) {
    if (a != null && typeof a === 'object' && typeof a.length === 'number') {
      // new ImageData(dataArray, width[, height])——dataArray 须 Uint8ClampedArray（real browser），lenient 接受类数组。
      var data = (a instanceof Uint8ClampedArray) ? a : new Uint8ClampedArray(a);
      var w = Math.abs(+b || 0) | 0;
      if (w <= 0) w = 1; // 防 0 除
      var h = (c != null) ? (Math.abs(+c || 0) | 0) : ((data.length / 4) / w) | 0;
      this.width = w;
      this.height = h;
      this.data = data;
      this.colorSpace = 'srgb';
    } else {
      // new ImageData(width, height)——透明黑全零。
      var w2 = Math.abs(+a || 0) | 0;
      var h2 = Math.abs(+b || 0) | 0;
      this.width = w2;
      this.height = h2;
      this.data = new Uint8ClampedArray(w2 * h2 * 4);
      this.colorSpace = 'srgb';
    }
  }
  globalThis.ImageData = globalThis.ImageData || ImageData;


  // canvas 元素 + 2d 上下文 proxy（R2795，canvas slice 1）。host 持 CanvasContext 注册表，JS 经
  // `__zw_canvas_op(handle, op, ...args)` 串参派发。`getContext('2d')` 首次调时创建 host 上下文（返 id），
  // 后续返回同一 proxy。host 未注册 → getContext 返 null（no-throw 回落）。width/height 默认 300×150（spec）。
  // **fillRect 经 path 实现**（host fill_rect 便捷法不写 pixel_buffer，path-based fill 经 blit 写）。
  // **canvas 为 standalone 对象**（非 host-backed 元素 proxy——canvas 主要经 context 离屏绘制，不需 DOM
  // 树挂载；DOM 集成/appendChild 为 follow-up）。
  function _zwMakeCanvas() {
    var el = {
      nodeType: 1,
      tagName: 'CANVAS',
      nodeName: 'CANVAS',
      localName: 'canvas',
      width: 300,
      height: 150,
      style: {},
      _ctx: null
    };
    el.getContext = function (type) {
      if (String(type) !== '2d') return null; // 仅 2d；webgl/webgl2 defer
      if (el._ctx) return el._ctx;
      if (typeof __zw_canvas_op !== 'function') return null;
      var id = __zw_canvas_op('0', 'getContext2d', String(el.width), String(el.height));
      if (!id || String(id).charAt(0) === '!') return null;
      el._ctx = _zwMakeCtx2d(String(id));
      el._ctx.canvas = el;
      // R34xx：direction 'inherit' 解析为 canvas 元素方向（dir 属性——2d.text.draw.align.
      // start.rtl 的 <canvas dir="rtl">）。host 存解析值；client getter 保持 'inherit'（spec）。
      var elDir = String(el.getAttribute ? String(el.getAttribute('dir') || '') : '').toLowerCase();
      if (elDir === 'rtl' || elDir === 'ltr') {
        __zw_canvas_op(String(id), 'setDirection', elDir);
      }
      return el._ctx;
    };
    // toDataURL（R2797，canvas slice 3）：PNG 导出。host 编码 ctx.pixel_buffer → PNG（csv 字节）→
    // shim 转 Latin-1 → btoa → `data:image/png;base64,...`（复用 btoa，无 base64 dep）。仅 'image/png'
    //（type 参数忽略，jpeg/webp defer）；host 未注册 / 编码失败 → `data:,` 回落。无 ctx 时惰性创建。
    el.toDataURL = function (_type) {
      if (typeof __zw_canvas_op !== 'function') return 'data:,';
      if (!el._ctx) el.getContext('2d');
      if (!el._ctx) return 'data:,';
      var csv = String(__zw_canvas_op(el._ctx._handle, 'toDataURL'));
      if (!csv) return 'data:,';
      var nums = csv.split(',');
      var s = '';
      for (var i = 0; i < nums.length; i++) s += String.fromCharCode(+nums[i]);
      return 'data:image/png;base64,' + btoa(s);
    };
    // toBlob（R3296，HTMLCanvasElement）：异步 PNG Blob 导出。镜像 toDataURL 的 host 编码路径
    //（`__zw_canvas_op(handle,'toDataURL')` → PNG CSV 字节），但产物为 `Blob`（type 'image/png'）经
    // **回调**异步派发（spec：返 undefined，callback(blob|null) 在 microtask 触发，失败/无 ctx→callback(null)）。
    // 复用 toDataURL 的 PNG 编码（仅 image/png；type/quality 参数 best-effort 与 toDataURL 同），无 ctx 惰性创建。
    // 用途：canvas 导出库（html2canvas/fabric.js/Chart.js「Save as Image」）+ FormData 上传 + createObjectURL 预览。
    // https://html.spec.whatwg.org/multipage/imagebitmap-and-animations.html#dom-canvas-toblob
    el.toBlob = function (callback, _type, _quality) {
      var cb = callback;
      // 异步派发（spec 在 task 中回调；headless 近似为 microtask——Promise.resolve().then）。
      var p = Promise.resolve().then(function () {
        if (typeof __zw_canvas_op !== 'function') return null;
        if (!el._ctx) el.getContext('2d');
        if (!el._ctx || !el._ctx._handle) return null; // 无 ctx → 无 bitmap
        var csv = String(__zw_canvas_op(el._ctx._handle, 'toDataURL'));
        if (!csv) return null;
        var nums = csv.split(',');
        var bytes = new Uint8Array(nums.length);
        for (var i = 0; i < nums.length; i++) bytes[i] = +nums[i];
        return new Blob([bytes], { type: 'image/png' });
      });
      if (typeof cb === 'function') p.then(function (blob) { cb(blob); });
      return undefined; // spec：toBlob 返 undefined（非 Promise）
    };
    return el;
  }
  // R3077：HTMLCanvasElement width/height 反射读（spec default 300/150）。供 CANVAS get-trap getContext
  //（建 host 上下文尺寸）+ width/height 属性读共用。parseInt 内容属性，缺省/不可解析/负 → default。
  function _zwCanvasDim(sel, handle, name, def) {
    var raw = handle
      ? __zw_get_attr_handle(handle, name)
      : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, name) : __zw_get_attr(sel, name));
    var n = parseInt(String(raw == null ? '' : raw), 10);
    return (isNaN(n) || n < 0) ? def : n;
  }
  // R34xx：'currentColor' 关键字解析（spec：canvas 元素 computed color，设值时求值——
  // 2d.fillStyle.parse.current.* / 2d.shadow.attributes.shadowColor.current.*）。shim 内
  // style.color 为内联值；无内联 → 默认黑（CSS 初始值）。设置后变更 style 不影响（.changed）。
  function _zwResolveCurrentColor(el) {
    // spec：currentColor = 元素 computed color；不在 document（remove 后——2d.shadow.
    // attributes.shadowColor.current.removed）→ CSS 初始值黑。DOM shim 的 document 节点
    // 不挂在 parent 链上（probe：CANVAS→BODY→HTML→end），故以「parent 链存在」近似
    // connected（remove 后 parentNode 为 null；standalone canvas 无 parentNode → 黑）。
    var inTree = false;
    if (el) {
      var cur = el;
      var guard = 0;
      while (cur && guard < 10000) {
        var p = null;
        try { p = cur.parentNode; } catch (_e) { break; }
        if (p === null || p === undefined) break;
        inTree = true;
        cur = p;
        guard++;
      }
    }
    if (!inTree) {
      return '#000000';
    }
    if (el.style && typeof el.style.color === 'string' && el.style.color !== '') {
      return String(el.style.color);
    }
    // R34xx：setAttribute('style', 'color: magenta') 路径——DOM shim 的 style 对象可能
    // 未同步该属性（2d.fillStyle.relativecolor.currentcolor：解析 style 属性串的 color 声明）。
    if (el.getAttribute) {
      var stAttr = String(el.getAttribute('style') || '');
      var m = /(?:^|;)\s*color\s*:\s*([^;]+)/i.exec(stAttr);
      if (m && m[1].trim()) {
        return m[1].trim();
      }
    }
    return '#000000';
  }
  // R34xx：CSS Typed OM 最小面（2d.fillStyle.CSSRGB/CSSHSL/colorObject.*）——
  // CSSRGB/CSSHSL 构造器 + CSS.percent/CSS.deg 数值对象。分量 0-1 浮点（或 CSSUnitValue）；
  // 渲染为规范化颜色串（fillStyle setter 的 object 分支消费）。
  if (!globalThis.CSS) globalThis.CSS = {};
  if (!CSS.percent) CSS.percent = function (v) { return { value: +v, unit: 'percent' }; };
  if (!CSS.deg) CSS.deg = function (v) { return { value: +v, unit: 'deg' }; };
  if (!globalThis.CSSRGB) {
    globalThis.CSSRGB = function (r, g, b, alpha) {
      this.r = r; this.g = g; this.b = b; this.alpha = alpha != null ? alpha : 1;
    };
  }
  if (!globalThis.CSSHSL) {
    globalThis.CSSHSL = function (h, s, l, alpha) {
      this.h = h; this.s = s; this.l = l; this.alpha = alpha != null ? alpha : 1;
    };
  }
  // color object → 规范化颜色串（null = 非对象不可转换）。分量语义：CSSRGB 通道 0-1 浮点 /
  // CSS.percent(v)（v/100）；CSSHSL h 角度（deg 原值）、s/l 0-1 浮点或 percent。alpha 越界钳
  // [0,1]（2d.fillStyle.colorObject.transparency 的 a:-1 → 全透明）。
  function _zwColorObjectToString(v) {
    var ch = function (c) {
      if (c == null) return NaN;
      if (typeof c === 'object') {
        var val = +c.value;
        if (String(c.unit).toLowerCase() === 'percent') return val / 100;
        if (String(c.unit).toLowerCase() === 'deg') return val;
        return val;
      }
      return +c;
    };
    var cl = function (x) { return Math.round(Math.min(Math.max(x, 0), 1) * 255); };
    // WPT colorObject 用 `a:` 键（{r,g,b,a}）；CSSRGB/CSSHSL 用 `alpha:`——两者都支持。
    var alpha = ch(v.alpha != null ? v.alpha : (v.a != null ? v.a : 1));
    if (!isFinite(alpha)) return null;
    var aClamped = Math.min(Math.max(alpha, 0), 1);
    if (v.h != null) {
      var h = ch(v.h), s = ch(v.s), l = ch(v.l);
      if (!isFinite(h) || !isFinite(s) || !isFinite(l)) return null;
      if (aClamped >= 1) return 'hsl(' + h + ', ' + (s * 100) + '%, ' + (l * 100) + '%)';
      return 'hsla(' + h + ', ' + (s * 100) + '%, ' + (l * 100) + '%, ' + aClamped + ')';
    }
    var r = ch(v.r), g = ch(v.g), b = ch(v.b);
    if (!isFinite(r) || !isFinite(g) || !isFinite(b)) return null;
    if (aClamped >= 1) return 'rgb(' + cl(r) + ', ' + cl(g) + ', ' + cl(b) + ')';
    return 'rgba(' + cl(r) + ', ' + cl(g) + ', ' + cl(b) + ', ' + aClamped + ')';
  }
  // R3079：CanvasGradient proxy。_zwGrad 为渐变 host id 标记（fillStyle/strokeStyle setter 检测）。
  // R34xx：addColorStop 参数校验（spec：offset 非有限/越界抛 IndexSizeError；颜色无效抛
  // SyntaxError——2d.gradient.object.invalidoffset/invalidcolor）+ 全局 CanvasGradient 构造器
  //（2d.gradient.object.type/return 依赖 prototype）。
  if (!globalThis.CanvasPattern) {
    globalThis.CanvasPattern = function CanvasPattern() {};
    // R34xx：CanvasPattern.setTransform（spec：接受 DOMMatrix 或 6 参；恒等生效——
    // 非 identity 的 pattern 采样变换为已知缺口）。
    CanvasPattern.prototype.setTransform = function (m) {
      if (!this || this._zwPattern === undefined) {
        throw new TypeError('Illegal invocation');
      }
      var vals = null;
      if (m && typeof m === 'object') {
        var hasM = typeof m.m11 === 'number' || typeof m.m12 === 'number' || typeof m.m21 === 'number' ||
                   typeof m.m22 === 'number' || typeof m.m41 === 'number' || typeof m.m42 === 'number';
        var hasAB = typeof m.a === 'number' || typeof m.b === 'number' || typeof m.c === 'number' ||
                    typeof m.d === 'number' || typeof m.e === 'number' || typeof m.f === 'number';
        if (hasM && hasAB) {
          // DOMMatrixInit 别名冲突（m11 与 a 同值合法——DOMMatrix 实例；异值 → TypeError）。
          var pairs = [['a','m11'],['b','m12'],['c','m21'],['d','m22'],['e','m41'],['f','m42']];
          for (var i = 0; i < pairs.length; i++) {
            var x = m[pairs[i][0]], y = m[pairs[i][1]];
            if (x != null && y != null && Number(x) !== Number(y)) {
              throw new TypeError('DOMMatrixInit: conflicting alias members');
            }
          }
        }
        if (hasM) {
          vals = [m.m11 == null ? 1 : m.m11, m.m12 == null ? 0 : m.m12, m.m21 == null ? 0 : m.m21,
                  m.m22 == null ? 1 : m.m22, m.m41 == null ? 0 : m.m41, m.m42 == null ? 0 : m.m42];
        } else if (hasAB) {
          vals = [m.a == null ? 1 : m.a, m.b == null ? 0 : m.b, m.c == null ? 0 : m.c,
                  m.d == null ? 1 : m.d, m.e == null ? 0 : m.e, m.f == null ? 0 : m.f];
        }
      }
      if (vals === null) {
        if (arguments.length < 6) throw new TypeError('setTransform: invalid matrix');
        vals = [arguments[0], arguments[1], arguments[2], arguments[3], arguments[4], arguments[5]];
      }
      if (typeof __zw_canvas_op === 'function') {
        __zw_canvas_op('0', 'setPatternTransform', String(this._zwPattern),
          String(vals[0]), String(vals[1]), String(vals[2]), String(vals[3]), String(vals[4]), String(vals[5]));
      }
    };
  }
  if (!globalThis.CanvasGradient) {
    globalThis.CanvasGradient = function CanvasGradient() {};
    // R34xx：prototype.addColorStop 委托实例方法（2d.gradient.object.type 断言 prototype 方法存在）。
    CanvasGradient.prototype.addColorStop = function (offset, color) {
      if (this && typeof this.addColorStop === 'function' && this !== CanvasGradient.prototype) {
        return this.addColorStop(offset, color);
      }
      throw new TypeError('Illegal invocation');
    };
  }
  function _zwMakeGradient(h, gid) {
    function addColorStop(offset, color) {
      // R34xx：参数校验（spec：https://html.spec.whatwg.org/multipage/canvas.html#dom-
      // canvasgradient-addcolorstop——offset 非有限抛 TypeError（2d.gradient.object.invalidoffset）；
      // 越界抛 IndexSizeError；颜色无效抛 SyntaxError）。
      offset = +offset;
      if (!isFinite(offset)) {
        throw new TypeError('addColorStop: non-finite offset');
      }
      if (offset < 0 || offset > 1) {
        throw _zwDomException('gradient offset out of range', 'IndexSizeError');
      }
      var c = String(color);
      if (c === '' || (typeof __zw_canvas_op === 'function' && !String(__zw_canvas_op('0', 'validateColor', c)))) {
        throw _zwDomException('invalid gradient color', 'SyntaxError');
      }
      __zw_canvas_op(h, 'addColorStop', gid, String(offset), String(color));
    }
    var g = { _zwGrad: gid, addColorStop: addColorStop };
    Object.setPrototypeOf(g, CanvasGradient.prototype);
    return g;
  }
  // R3306：Path2D（spec CanvasPath，`new Path2D()` / `new Path2D(other)` / `new Path2D(svgString)`）。
  // `_zwPath` 为 host 路径 id 标记（ctx.fill(path) 等 setter 检测）。方法镜像 ctx 路径族（经 host path id 改
  // host Path2D）。R3307：svgString 构造形式补全（host createPath 走 canvas crate `Path2D::from_svg`）。
  function _zwMakePath2d(h, pid) {
    var p = { _zwPath: pid };
    p.moveTo = function (x, y) { __zw_canvas_op(h, 'pathMoveTo', pid, String(x), String(y)); };
    p.lineTo = function (x, y) { __zw_canvas_op(h, 'pathLineTo', pid, String(x), String(y)); };
    p.closePath = function () { __zw_canvas_op(h, 'pathClose', pid); };
    p.arc = function (x, y, r, s, e, anticlockwise) { __zw_canvas_op(h, 'pathArc', pid, String(x), String(y), String(r), String(s), String(e), anticlockwise ? 'true' : 'false'); };
    p.arcTo = function (x1, y1, x2, y2, r) { __zw_canvas_op(h, 'pathArcTo', pid, String(x1), String(y1), String(x2), String(y2), String(r)); };
    p.quadraticCurveTo = function (cpx, cpy, x, y) { __zw_canvas_op(h, 'pathQuadratic', pid, String(cpx), String(cpy), String(x), String(y)); };
    p.bezierCurveTo = function (cp1x, cp1y, cp2x, cp2y, x, y) { __zw_canvas_op(h, 'pathBezier', pid, String(cp1x), String(cp1y), String(cp2x), String(cp2y), String(x), String(y)); };
    p.ellipse = function (x, y, rx, ry, rot, s, e) { __zw_canvas_op(h, 'pathEllipse', pid, String(x), String(y), String(rx), String(ry), String(rot), String(s), String(e)); };
    p.rect = function (x, y, w, hh) { __zw_canvas_op(h, 'pathRect', pid, String(x), String(y), String(w), String(hh)); };
    p.addPath = function (other) {
      if (other && other._zwPath) __zw_canvas_op(h, 'addPath', pid, String(other._zwPath));
    };
    return p;
  }
  // Path2D 全局构造器（幂等注册——多 canvas 不重复覆盖）。host createPath 忽略 handle（路径 context 无关），
  // 故用任意 handle '0'；首参三态：Path2D 对象（复制）、svgString（host from_svg 解析）、undefined（建空）。
  // R3307：svgString 形式补全（`new Path2D("M10 10 L90 90")`），整串透传 host createPath 走 from_svg。
  if (!globalThis.Path2D) {
    globalThis.Path2D = function Path2D(arg) {
      if (typeof __zw_canvas_op !== 'function') { this._zwPath = 0; return; }
      // 首参三态：Path2D 对象 → 传 path id（host 复制）；string → 透传整串（host from_svg）；否则空串（建空）。
      var first = '';
      if (arg && typeof arg === 'object' && arg._zwPath) {
        first = String(arg._zwPath);
      } else if (typeof arg === 'string') {
        first = arg;
      }
      var id = String(__zw_canvas_op('0', 'createPath', first));
      this._zwPath = id;
      // 把 _zwMakePath2d 的方法绑到本实例（复用方法集，pid 为本实例 id）。
      var proto = _zwMakePath2d('0', id);
      for (var k in proto) { if (k !== '_zwPath') this[k] = proto[k]; }
    };
  }
  // R3309：ImageBitmap + createImageBitmap（HTML spec）——异步解码图片源为可绘制位图。
  // R3310：source 扩展——Blob（fetch 图片，host decode_data_uri 解码）/ ImageData（直接 JS 编码 wire，无解码）/
  // HTMLCanvasElement（经 getImageData 取 wire，镜像 drawImage canvas 源模式）。三者统一产 wire 串 `"w:h;rgba,..."`
  // 包成 ImageBitmap 对象（持 _zwBitmapWire，drawImage 检测该标记复用既有 drawImage host wire 路径，零 host 改动）。
  // spec 返 Promise（异步）；headless 近似 microtask（Promise.resolve.then）。失败（尺寸 0 / 无 host / 未知 source）→ reject。
  // **诚实范围**：① HTMLImageElement source defer（img 元素 headless 无加载/解码基建，naturalWidth 恒 0）；
  // ② ImageBitmap source（clone，罕见）；③ 无 options（sx/sy/sw/sh/dw/dh 裁剪 + imageOrientation/premultiplyAlpha，spec 罕见，defer）；
  // ④ width/height 从 wire 解析（spec ImageBitmap 只读 width/height）。
  // https://html.spec.whatwg.org/multipage/imagebitmap-and-animations.html#dom-createimagebitmap
  function _zwMakeImageBitmap(wire) {
    // wire "w:h;..." 解析 width/height（失败 → 0×0，调用方 reject）。
    var dim = String(wire).split(';')[0] || '';
    var parts = dim.split(':');
    var bw = parseInt(parts[0], 10) || 0;
    var bh = parseInt(parts[1], 10) || 0;
    return { _zwBitmapWire: String(wire), width: bw, height: bh, _closed: false };
  }
  // R3310：source → wire 串（同步）。返 null 表 source 不可识别/解码失败（调用方 reject）。
  // 三 source 分发：Blob（host 解码）/ ImageData（JS 直接编码）/ HTMLCanvasElement（getImageData 取 wire）。
  function _zwImageBitmapSourceToWire(src) {
    if (!src) return null;
    // Blob：有 _parts + size（instanceof Blob 或 duck-type）。字节 → base64 data URI → host decodeImageBitmap。
    var isBlob = src instanceof Blob || (src._parts !== undefined && typeof src.size === 'number');
    if (isBlob) {
      if (typeof __zw_canvas_op !== 'function') return null;
      var bytes = _zw_blobBytes(src);
      if (!bytes || bytes.length === 0) return null;
      var bin = '';
      for (var i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
      var mime = (src.type && src.type.indexOf('image/') === 0) ? src.type : 'image/png';
      var uri = 'data:' + mime + ';base64,' + btoa(bin);
      return String(__zw_canvas_op('0', 'decodeImageBitmap', uri));
    }
    // ImageData：有 data（类数组，length = w*h*4）+ width + height。直接 JS 编码 wire（无解码，零 host 调用）。
    // R3309 全局 ImageData（part05.js）+ canvas crate getImageData 返均此形状。
    if (src.data && typeof src.width === 'number' && typeof src.height === 'number' && typeof src.data.length === 'number') {
      var w = src.width | 0;
      var h = src.height | 0;
      if (w <= 0 || h <= 0) return null;
      var wire = w + ':' + h + ';';
      var d = src.data;
      for (var j = 0; j < d.length; j++) {
        if (j > 0) wire += ',';
        wire += (d[j] | 0);
      }
      return wire;
    }
    // HTMLCanvasElement：有 getContext（canvas 元素）。经 getImageData 取全 canvas wire（镜像 drawImage canvas 源）。
    if (typeof src.getContext === 'function') {
      if (typeof __zw_canvas_op !== 'function') return null;
      if (!src._ctx) src.getContext('2d');
      if (!src._ctx) return null;
      var sw = src.width | 0;
      var sh = src.height | 0;
      if (sw <= 0 || sh <= 0) return null;
      return String(__zw_canvas_op(src._ctx._handle, 'getImageData', '0', '0', String(sw), String(sh)));
    }
    return null;
  }
  // R3311：wire 串子矩形裁剪（createImageBitmap options sx/sy/sw/sh）。解析源 wire 像素 → 取 [sx,sx+sw)×[sy,sy+sh)
  // 子矩形 → 重编码为 sw×sh wire。越界 clamp 到源边界（spec：超界范围不贡献像素，结果尺寸为有效交集——本实现
  // 简化为 clamp sw/sh 到源内，spec 近似 documented）。
  //
  // R3254-C1：参数校验前移到调用方（显式 0 → RangeError、负值翻转矩形、非有限 → 拒绝），
  // 本函数只处理**已校验为正**的 sw/sh；裁剪区完全超源 → 返回 '0:0;'（调用方判零尺寸拒绝）。
  function _zwCropWire(wire, sx, sy, sw, sh) {
    var s = String(wire);
    var semi = s.indexOf(';');
    if (semi < 0) return s;
    var dims = s.substring(0, semi).split(':');
    var srcW = parseInt(dims[0], 10) || 0;
    var srcH = parseInt(dims[1], 10) || 0;
    if (srcW <= 0 || srcH <= 0) return s;
    sx = (sx == null || !isFinite(sx)) ? 0 : (sx | 0);
    sy = (sy == null || !isFinite(sy)) ? 0 : (sy | 0);
    sw = sw | 0;
    sh = sh | 0;
    // clamp 到源边界。
    if (sx < 0) { sw += sx; sx = 0; }
    if (sy < 0) { sh += sy; sy = 0; }
    if (sx + sw > srcW) sw = srcW - sx;
    if (sy + sh > srcH) sh = srcH - sy;
    if (sw <= 0 || sh <= 0) return '0:0;'; // 裁剪区完全在源外 → 零尺寸
    // 解析源像素（逗号分隔十进制）。
    var pix = s.substring(semi + 1).split(',');
    var out = sw + ':' + sh + ';';
    var first = true;
    for (var y = 0; y < sh; y++) {
      for (var x = 0; x < sw; x++) {
        // 源像素索引 = ((sy + y) * srcW + (sx + x)) * 4，取 RGBA 4 分量。
        var base = ((sy + y) * srcW + (sx + x)) * 4;
        for (var c = 0; c < 4; c++) {
          if (!first) out += ',';
          first = false;
          out += (parseInt(pix[base + c], 10) || 0);
        }
      }
    }
    return out;
  }
  // R34xx：垂直翻转 wire 像素（createImageBitmap options imageOrientation: 'flipY'）。
  function _zwFlipWireY(wire) {
    var s = String(wire);
    var semi = s.indexOf(';');
    if (semi < 0) return s;
    var dims = s.substring(0, semi).split(':');
    var w = parseInt(dims[0], 10) || 0;
    var h = parseInt(dims[1], 10) || 0;
    if (w <= 0 || h <= 0) return s;
    var pix = s.substring(semi + 1).split(',');
    var out = w + ':' + h + ';';
    var first = true;
    for (var y = h - 1; y >= 0; y--) {
      for (var x = 0; x < w; x++) {
        var base = (y * w + x) * 4;
        for (var c = 0; c < 4; c++) {
          if (!first) out += ',';
          first = false;
          out += (parseInt(pix[base + c], 10) || 0);
        }
      }
    }
    return out;
  }
  if (!globalThis.createImageBitmap) {
    globalThis.createImageBitmap = function createImageBitmap(source, sx, sy, sw, sh) {
      // R34xx：options（imageOrientation/premultiplyAlpha）——(source, options) 或
      // (source, sx, sy, sw, sh, options) 两形式。flipY 翻转；premultiplyAlpha 接受
      //（'none'|'premultiply'|'default'，像素格式无预乘概念 → 无操作，记录）。
      var opts = {};
      if (arguments.length >= 2 && typeof arguments[1] === 'object' && arguments[1] !== null && arguments[1].nodeType === undefined) {
        opts = arguments[1];
      } else if (arguments.length >= 6 && typeof arguments[5] === 'object' && arguments[5] !== null) {
        opts = arguments[5];
      }
      var flipY = !!opts.imageOrientation && String(opts.imageOrientation).toLowerCase() === 'flipy';
      return Promise.resolve(source).then(function (src) {
        var wire = _zwImageBitmapSourceToWire(src);
        if (wire === null) {
          return Promise.reject(new TypeError('createImageBitmap: 不支持的 source 或解码失败'));
        }
        // R3311：options 裁剪（sx/sy/sw/sh 子矩形）。4 数值参齐备时裁剪，否则不裁。
        // R3254-C1：参数校验（WPT createImageBitmap-invalid-args）——显式 0 → RangeError；
        // 非有限 → InvalidStateError；负 sw/sh 不拒绝，按 spec 翻转矩形（sx += sw; sw = -sw，
        // WebKit 语义）；全部通过后才裁剪。
        if (sw != null || sh != null) {
          if (sw === 0 || sh === 0) {
            return Promise.reject(new RangeError('createImageBitmap: sw/sh 不能为 0'));
          }
          if (!isFinite(sw) || !isFinite(sh)) {
            return Promise.reject(_zwDomException('createImageBitmap: sw/sh 必须为有限数', 'InvalidStateError'));
          }
          if (sw < 0) { sx = sx + sw; sw = -sw; }
          if (sh < 0) { sy = sy + sh; sh = -sh; }
          wire = _zwCropWire(wire, sx, sy, sw, sh);
          if (String(wire).indexOf('0:0;') === 0) {
            // 裁剪区与源无交集（完全在源外）→ InvalidStateError（spec/WPT "oversized crop region"）。
            return Promise.reject(_zwDomException('createImageBitmap: 裁剪区与源无交集', 'InvalidStateError'));
          }
        }
        if (flipY) {
          wire = _zwFlipWireY(wire);
        }
        var bm = _zwMakeImageBitmap(wire);
        if (bm.width <= 0 || bm.height <= 0) {
          return Promise.reject(new TypeError('createImageBitmap: 解码失败（零尺寸）'));
        }
        return bm;
      });
    };
  }
  // R3312：OffscreenCanvas（HTML spec，Done Criteria §3 Tier 3）——主线程离屏 canvas。
  // `new OffscreenCanvas(w, h)` 构造（非 DOM，镜像 standalone _zwMakeCanvas 模式）+ getContext('2d') +
  // transferToImageBitmap()（取全 canvas 像素 wire 包 ImageBitmap，复用 _zwMakeImageBitmap）。
  // **诚实范围**：① 仅主线程（worker 内 OffscreenCanvas defer——R3089 worker 影子上下文 wctx 无 __zw_canvas_op，
  //   真 worker OffscreenCanvas 需 worker canvas host 桥接，跨层大改）；② 无 transferControlToOffscreen（canvas 元素
  //   转 offscreen，需 DOM canvas ↔ offscreen 句柄共享，defer）；③ 仅 '2d' context（webgl defer）。
  // https://html.spec.whatwg.org/multipage/canvas.html#the-offscreencanvas-interface
  function OffscreenCanvas(width, height) {
    if (!(this instanceof OffscreenCanvas)) return new OffscreenCanvas(width, height);
    // R3254-C5：width/height 用 accessor——setter 在已 getContext 后调 host resizeContext
    //（spec：与 canvas.width 同语义——重置 bitmap + 绘图状态）；此前是普通数据属性，
    // `oc.width = 200` 只改 JS 数字、host bitmap 保持原尺寸（绘制被裁剪/尺寸错配）。
    var _w = (typeof width === 'number' && width > 0) ? (width | 0) : 300;
    var _h = (typeof height === 'number' && height > 0) ? (height | 0) : 150;
    this._ctx = null;
    var self = this;
    Object.defineProperty(this, 'width', {
      get: function () { return _w; },
      set: function (v) {
        var nv = (typeof v === 'number' && v > 0) ? (v | 0) : 300;
        if (nv === _w) return;
        _w = nv;
        if (self._ctx && typeof __zw_canvas_op === 'function') {
          __zw_canvas_op(self._ctx._handle, 'resizeContext', String(_w), String(_h));
        }
      },
      enumerable: true,
      configurable: true
    });
    Object.defineProperty(this, 'height', {
      get: function () { return _h; },
      set: function (v) {
        var nv = (typeof v === 'number' && v > 0) ? (v | 0) : 150;
        if (nv === _h) return;
        _h = nv;
        if (self._ctx && typeof __zw_canvas_op === 'function') {
          __zw_canvas_op(self._ctx._handle, 'resizeContext', String(_w), String(_h));
        }
      },
      enumerable: true,
      configurable: true
    });
  }
  OffscreenCanvas.prototype.getContext = function (type) {
    if (String(type) !== '2d') return null; // 仅 2d；webgl/webgl2 defer
    if (this._ctx) return this._ctx;
    if (typeof __zw_canvas_op !== 'function') return null;
    var id = __zw_canvas_op('0', 'getContext2d', String(this.width), String(this.height));
    if (!id || String(id).charAt(0) === '!') return null;
    this._ctx = _zwMakeCtx2d(String(id));
    return this._ctx;
  };
  // transferToImageBitmap()：取当前 canvas 全像素 wire 包成 ImageBitmap（spec 返新 ImageBitmap，canvas bitmap 清空）。
  // 复用 _zwMakeImageBitmap（持 _zwBitmapWire，drawImage 可消费）。canvas bitmap 清空对齐 spec（transfer 语义）。
  OffscreenCanvas.prototype.transferToImageBitmap = function () {
    if (typeof __zw_canvas_op !== 'function') return null;
    if (!this._ctx) this.getContext('2d');
    if (!this._ctx) return null;
    var wire = String(__zw_canvas_op(this._ctx._handle, 'getImageData', '0', '0', String(this.width), String(this.height)));
    var bm = _zwMakeImageBitmap(wire);
    if (bm.width <= 0 || bm.height <= 0) return null;
    // spec transfer 语义：源 canvas bitmap 被清空（替换为透明黑）。
    // R3254-C8：clearBitmap 只清像素、保留绘图状态（此前 resizeContext 重置全状态——
    // 连续 transfer 后 fillStyle/transform 等丢失）。
    __zw_canvas_op(this._ctx._handle, 'clearBitmap');
    return bm;
  };
  if (!globalThis.OffscreenCanvas) {
    globalThis.OffscreenCanvas = OffscreenCanvas;
  }
  function _zwMakeCtx2d(h) {
    var ctx = { _handle: h, canvas: null, _fs: '#000000', _ss: '#000000', _lw: 1.0 };
    // R3079：fillStyle/strokeStyle 接受颜色串或 CanvasGradient 对象。spec — 设渐变后 getter 返回该渐变对象。
    // 渐变对象带 _zwGrad 标记（_zwMakeGradient）；命中走 setFillStyleGradient/setStrokeStyleGradient（host 查渐变
    // 注册表克隆到 context 样式），否则按颜色串解析。
    Object.defineProperty(ctx, 'fillStyle', {
      set: function (v) {
        if (v && typeof v === 'object' && v._zwGrad) {
          this._fs = v;
          __zw_canvas_op(h, 'setFillStyleGradient', String(v._zwGrad));
        } else if (v && typeof v === 'object' && v._zwPattern) {
          this._fs = v;
          __zw_canvas_op(h, 'setFillStylePattern', String(v._zwPattern));
        } else if (v && typeof v === 'object') {
          // R34xx：color object（CSSRGB/CSSHSL/plain {r,g,b[,a]}——spec CSS Color 4；
          // 2d.fillStyle.colorObject.* / CSSRGB / CSSHSL）。转换失败（不可转换对象）回落旧行为。
          var _cs = _zwColorObjectToString(v);
          if (_cs !== null) {
            this._fs = _cs;
            __zw_canvas_op(h, 'setFillStyle', _cs);
          } else {
            this._fs = String(v);
            __zw_canvas_op(h, 'setFillStyle', String(v));
          }
        } else {
          // R34xx：'currentColor' 设值时解析为 canvas 元素计算色（2d.fillStyle.parse.current.*）；
          // 表达式内嵌 currentcolor（color-mix/相对色——2d.fillStyle.colormix.currentcolor）整体替换。
          var _v = String(v);
          if (_v.toLowerCase() === 'currentcolor') _v = _zwResolveCurrentColor(this.canvas);
          else if (/currentcolor/i.test(_v)) _v = _v.replace(/currentcolor/gi, _zwResolveCurrentColor(this.canvas));
          this._fs = _v;
          __zw_canvas_op(h, 'setFillStyle', _v);
        }
      },
      get: function () {
        // R34xx：颜色串读 host 规范化（opaque→#rrggbb / alpha→rgba(带空格)——与
        // shadowColor 同款；2d.fillStyle.get.* 断言格式）。渐变/图案对象走本地缓存。
        // CSS Color 4 输入（color-mix/相对色）→ host 返 color(srgb ...) 保留表示
        //（2d.fillStyle.colormix/relativecolor）。
        if (typeof this._fs === 'string' && typeof __zw_canvas_op === 'function') {
          var v = String(this._fs);
          if (v.indexOf('color-mix(') === 0 || v.indexOf('rgb(from ') === 0 ||
              v.indexOf('hsl(from ') === 0 || v.indexOf('color(from ') === 0) {
            var r4 = String(__zw_canvas_op(h, 'parseColorCss4', v));
            if (r4) return r4;
          }
          var r = String(__zw_canvas_op(h, 'getFillStyle'));
          if (r) return r;
        }
        return this._fs;
      }
    });
    Object.defineProperty(ctx, 'strokeStyle', {
      set: function (v) {
        if (v && typeof v === 'object' && v._zwGrad) {
          this._ss = v;
          __zw_canvas_op(h, 'setStrokeStyleGradient', String(v._zwGrad));
        } else if (v && typeof v === 'object' && v._zwPattern) {
          this._ss = v;
          __zw_canvas_op(h, 'setStrokeStylePattern', String(v._zwPattern));
        } else if (v && typeof v === 'object') {
          // R34xx：color object（同 fillStyle；2d.strokeStyle.colorObject.*）。
          var _cs = _zwColorObjectToString(v);
          if (_cs !== null) {
            this._ss = _cs;
            __zw_canvas_op(h, 'setStrokeStyle', _cs);
          } else {
            this._ss = String(v);
            __zw_canvas_op(h, 'setStrokeStyle', String(v));
          }
        } else {
          // R34xx：'currentColor' 设值时解析（同 fillStyle）；表达式内嵌 currentcolor 替换。
          var _v = String(v);
          if (_v.toLowerCase() === 'currentcolor') _v = _zwResolveCurrentColor(this.canvas);
          else if (/currentcolor/i.test(_v)) _v = _v.replace(/currentcolor/gi, _zwResolveCurrentColor(this.canvas));
          this._ss = _v;
          __zw_canvas_op(h, 'setStrokeStyle', _v);
        }
      },
      get: function () {
        // R34xx：同 fillStyle getter（host 规范化 + CSS Color 4 保留表示——
        // 2d.strokeStyle.colormix/relativecolor）。
        if (typeof this._ss === 'string' && typeof __zw_canvas_op === 'function') {
          var v = String(this._ss);
          if (v.indexOf('color-mix(') === 0 || v.indexOf('rgb(from ') === 0 ||
              v.indexOf('hsl(from ') === 0 || v.indexOf('color(from ') === 0) {
            var r4 = String(__zw_canvas_op(h, 'parseColorCss4', v));
            if (r4) return r4;
          }
          var r = String(__zw_canvas_op(h, 'getStrokeStyle'));
          if (r) return r;
        }
        return this._ss;
      }
    });
    Object.defineProperty(ctx, 'lineWidth', {
      // R34xx：非法值（非有限/≤0）忽略保持旧值（spec：lineWidth 须为正有限数；
      // 上游 2d.line.width.invalid）。
      set: function (v) {
        v = +v;
        if (!isFinite(v) || v <= 0) return;
        this._lw = v;
        __zw_canvas_op(h, 'setLineWidth', String(v));
      },
      get: function () { return this._lw; }
    });
    ctx.beginPath = function () { __zw_canvas_op(h, 'beginPath'); };
    ctx.closePath = function () { __zw_canvas_op(h, 'closePath'); };
    ctx.moveTo = function (x, y) { __zw_canvas_op(h, 'moveTo', String(x), String(y)); };
    ctx.lineTo = function (x, y) { __zw_canvas_op(h, 'lineTo', String(x), String(y)); };
    ctx.arc = function (x, y, r, s, e, anticlockwise) {
      // R34xx：anticlockwise 第 6 参透传（spec：2d.line.cap.round 等 arc 填充用例依赖方向）。
      __zw_canvas_op(h, 'arc', String(x), String(y), String(r), String(s), String(e), anticlockwise ? 'true' : 'false');
    };
    // R3306：fill/stroke/clip 可选首参 Path2D（spec ctx.fill(path)），命中走 fillPath/strokePath/clipPath
    //（用给定 Path2D 替代 ctx 当前路径）；无参走当前路径形式（既定）。
    ctx.fill = function (path) {
      if (path && path._zwPath) __zw_canvas_op(h, 'fillPath', String(path._zwPath));
      else __zw_canvas_op(h, 'fill');
    };
    ctx.stroke = function (path) {
      if (path && path._zwPath) __zw_canvas_op(h, 'strokePath', String(path._zwPath));
      else __zw_canvas_op(h, 'stroke');
    };
    // R34xx：fillRect/strokeRect/clearRect 任一参数非有限（NaN/Infinity）→ 方法忽略
    //（spec：上游 2d.fillRect.nonfinite / strokeRect.nonfinite / clearRect.nonfinite）。
    var _zwRectFinite = function (x, y, w, h) {
      return isFinite(+x) && isFinite(+y) && isFinite(+w) && isFinite(+h);
    };
    ctx.fillRect = function (x, y, w, hh) {
      if (!_zwRectFinite(x, y, w, hh)) return;
      __zw_canvas_op(h, 'fillRect', String(x), String(y), String(w), String(hh));
    };
    ctx.strokeRect = function (x, y, w, hh) {
      if (!_zwRectFinite(x, y, w, hh)) return;
      __zw_canvas_op(h, 'strokeRect', String(x), String(y), String(w), String(hh));
    };
    ctx.clearRect = function (x, y, w, hh) {
      if (!_zwRectFinite(x, y, w, hh)) return;
      __zw_canvas_op(h, 'clearRect', String(x), String(y), String(w), String(hh));
    };
    // R3078：Canvas 2D 文本 API（fillText/strokeText/measureText）+ createImageData（blank）。
    // fillText 经 host fill_text（canvas crate 写 pixel_buffer）；measureText 返 TextMetrics（width+bounding）；
    // createImageData 返 blank ImageData（全透明 = 全 0，Uint8ClampedArray(w*h*4)，JS 构无需 host）。createImageData
    // 双形式：createImageData(w,h) / createImageData(imageData)（复制尺寸）。spec CanvasRenderingContext2D。
    ctx.fillText = function (text, x, y, maxWidth) {
      // R34xx：maxWidth 透传（spec fillText(text,x,y,maxWidth)）。
      __zw_canvas_op(h, 'fillText', String(text), String(+x || 0), String(+y || 0), String(maxWidth === undefined ? '' : +maxWidth));
    };
    // R34xx：fillTextCluster(cluster, x, y)——绘制单个字素簇（spec TextCluster；
    // 2d.text.measure.fillTextCluster-*.tentative）。簇对象经 measureText().getTextClusters()
    // 取得（含 x/y 相对文本原点偏移）。经 fillText 宿主路径（当前 font/baseline 生效）。
    ctx.fillTextCluster = function (cluster, x, y, options) {
      if (!cluster || typeof cluster !== 'object' || typeof cluster.text !== 'string') {
        throw new TypeError('fillTextCluster: invalid cluster');
      }
      // R34xx：options {align, baseline, x, y}——簇按目标对齐/基线定位（fillTextCluster-
      // options.tentative：right+bottom 使 em 右下角贴 (x,y)；x/y 覆盖簇自身偏移）。
      var adv = +cluster.advance || 50;
      var asc = +cluster.asc || adv * 0.75;
      var desc = +cluster.desc || -(adv * 0.25);
      var optX = (options && options.x !== undefined) ? (+options.x || 0) : (+cluster.x || 0);
      var optY = (options && options.y !== undefined) ? (+options.y || 0) : (+cluster.y || 0);
      var useAlign = (options && options.align !== undefined) ? String(options.align) : null;
      var useBaseline = (options && options.baseline !== undefined) ? String(options.baseline) : null;
      var alignOff = 0;
      if (useAlign === 'center') alignOff = -adv / 2;
      else if (useAlign === 'right' || useAlign === 'end') alignOff = -adv;
      var drawX = (+x || 0) + optX + alignOff;
      var drawY = (+y || 0) + optY;
      if (useBaseline !== null) {
        var oy = 0;
        if (useBaseline === 'top') oy = asc;
        else if (useBaseline === 'middle') oy = (asc + desc) / 2;
        else if (useBaseline === 'bottom') oy = desc;
        else if (useBaseline === 'hanging') oy = asc * (2 / 3);
        drawY += oy - asc;
      }
      __zw_canvas_op(h, 'fillText', String(cluster.text),
        String(drawX), String(drawY));
    };
    // R34xx：strokeTextCluster（spec TextCluster——与 fillTextCluster 对称，描边绘制）。
    ctx.strokeTextCluster = function (cluster, x, y, options) {
      if (!cluster || typeof cluster !== 'object' || typeof cluster.text !== 'string') {
        throw new TypeError('strokeTextCluster: invalid cluster');
      }
      // R34xx：与 fillTextCluster 同 options 语义（描边版）。
      var adv = +cluster.advance || 50;
      var asc = +cluster.asc || adv * 0.75;
      var desc = +cluster.desc || -(adv * 0.25);
      var optX = (options && options.x !== undefined) ? (+options.x || 0) : (+cluster.x || 0);
      var optY = (options && options.y !== undefined) ? (+options.y || 0) : (+cluster.y || 0);
      var useAlign = (options && options.align !== undefined) ? String(options.align) : null;
      var useBaseline = (options && options.baseline !== undefined) ? String(options.baseline) : null;
      var alignOff = 0;
      if (useAlign === 'center') alignOff = -adv / 2;
      else if (useAlign === 'right' || useAlign === 'end') alignOff = -adv;
      var drawX = (+x || 0) + optX + alignOff;
      var drawY = (+y || 0) + optY;
      if (useBaseline !== null) {
        var oy = 0;
        if (useBaseline === 'top') oy = asc;
        else if (useBaseline === 'middle') oy = (asc + desc) / 2;
        else if (useBaseline === 'bottom') oy = desc;
        else if (useBaseline === 'hanging') oy = asc * (2 / 3);
        drawY += oy - asc;
      }
      __zw_canvas_op(h, 'strokeText', String(cluster.text),
        String(drawX), String(drawY));
    };
    ctx.strokeText = function (text, x, y) {
      __zw_canvas_op(h, 'strokeText', String(text), String(+x || 0), String(+y || 0));
    };
    ctx.measureText = function (text) {
      // R3303：spec TextMetrics 全 10 字段（host 返 width,actualBoxAsc/Desc/Left/Right,
      // fontBoxAsc/Desc,alphabetic/hanging/ideographicBaseline csv；`|` 后逐字形墨迹
      // l,t,r,b 分号分隔）。R34xx：getActualBoundingBox(start,end) 子串墨迹 bbox（spec
      // TextMetrics 新方法——2d.text.measure.getActualBoundingBox.tentative）。
      var raw = String(__zw_canvas_op(h, 'measureText', String(text)));
      var parts = raw.split('|');
      var p = parts[0].split(',');
      var anchor = parseFloat(parts[2]) || 0;
      // R34xx：getTextClusters 默认 align/baseline 取 ctx 当前状态（方法内 this = tm）。
      var ctxTa = this._ta;
      var ctxTb = this._tb;
      var ctxDir = this._dir;
      var num = function (i) { return parseFloat(p[i]) || 0; };
      var glyphs = [];
      if (parts[1]) {
        var gs = parts[1].split(';');
        for (var gi = 0; gi < gs.length; gi++) {
          var gv = gs[gi].split(',');
          glyphs.push([parseFloat(gv[0]) || 0, parseFloat(gv[1]) || 0,
                       parseFloat(gv[2]) || 0, parseFloat(gv[3]) || 0]);
        }
      }
      var tm = {
        width: num(0),
        actualBoundingBoxAscent: num(1),
        actualBoundingBoxDescent: num(2),
        actualBoundingBoxLeft: num(3),
        actualBoundingBoxRight: num(4),
        fontBoundingBoxAscent: num(5),
        fontBoundingBoxDescent: num(6),
        // R34xx：emHeight*（spec TextMetrics——em 盒顶/底距基线；fontBoundingBox 同源）。
        emHeightAscent: num(5),
        emHeightDescent: num(6),
        alphabeticBaseline: num(7),
        hangingBaseline: num(8),
        ideographicBaseline: num(9),
        // R34xx：getActualBoundingBox(start, end)——[start, end) 字形墨迹并集矩形
        //（相对文本原点；无字体栈/空区间 → 空矩形 {0,0,0,0}）。
        // R34xx：getTextClusters(start, end)——UAX#29 字素簇分段（GB9 ZWJ/Extend、
        // GB11 emoji ZWJ 序列近似——2d.text.measure.text-clusters-*.tentative）。
        // 每簇 {start, end, text, x, y, width, height, advance, offsetInText}。
        getTextClusters: function (start, end) {
          // R34xx：options 形式 getTextClusters({align, baseline})——簇位置按目标
          // align/baseline 计算（text-clusters-position.tentative）。
          var optAlign = null, optBaseline = null;
          if (start && typeof start === 'object' && !Array.isArray(start)) {
            var opts = start;
            start = 0;
            end = text.length;
            optAlign = opts.align !== undefined ? String(opts.align) : null;
            optBaseline = opts.baseline !== undefined ? String(opts.baseline) : null;
          }
          start = start === undefined ? 0 : (+start || 0);
          end = end === undefined ? text.length : (+end || 0);
          if (start < 0 || end < 0) {
            throw new TypeError('getTextClusters: invalid range');
          }
          if (start > end || end > text.length) {
            throw _zwDomException('getTextClusters: invalid range', 'IndexSizeError');
          }
          var extRe = /[\u0300-\u036f\u0483-\u0489\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7\u0610-\u061a\u064b-\u065f\u0670\u06d6-\u06dc\u06df-\u06e4\u06e7-\u06e8\u06ea-\u06ed\u0711\u0730-\u074a\u07a6-\u07b0\u07eb-\u07f3\u0816-\u0819\u081b-\u0823\u0825-\u0827\u0829-\u082d\u0859-\u085b\u08d3-\u08e1\u08e3-\u0903\u093a-\u093c\u093e-\u094f\u0951-\u0957\u0962-\u0963\u0981-\u0983\u09bc\u09be-\u09c4\u09c7-\u09c8\u09cb-\u09cd\u09d7\u09e2-\u09e3\u0a01-\u0a03\u0a3c\u0a3e-\u0a42\u0a47-\u0a48\u0a4b-\u0a4d\u0a51\u0a70-\u0a71\u0a75\u0a81-\u0a83\u0abc\u0abe-\u0ac5\u0ac7-\u0ac9\u0acb-\u0acd\u0ae2-\u0ae3\u0b01-\u0b03\u0b3c\u0b3e-\u0b44\u0b47-\u0b48\u0b4b-\u0b4d\u0b56-\u0b57\u0b62-\u0b63\u0b82\u0bbe-\u0bc2\u0bc6-\u0bc8\u0bca-\u0bcd\u0bd7\u0c00-\u0c04\u0c3e-\u0c44\u0c46-\u0c48\u0c4a-\u0c4d\u0c55-\u0c56\u0c62-\u0c63\u0c81-\u0c83\u0cbc\u0cbe-\u0cc4\u0cc6-\u0cc8\u0cca-\u0ccd\u0cd5-\u0cd6\u0ce2-\u0ce3\u0d00-\u0d03\u0d3b-\u0d3c\u0d3e-\u0d44\u0d46-\u0d48\u0d4a-\u0d4d\u0d57\u0d62-\u0d63\u0d82-\u0d83\u0dca\u0dcf-\u0dd4\u0dd6\u0dd8-\u0ddf\u0df2-\u0df3\u0e31\u0e34-\u0e3a\u0e47-\u0e4e\u0eb1\u0eb4-\u0ebc\u0ec8-\u0ecd\u0f18-\u0f19\u0f35\u0f37\u0f39\u0f3e-\u0f3f\u0f71-\u0f84\u0f86-\u0f87\u0f8d-\u0f97\u0f99-\u0fbc\u0fc6\u102b-\u103e\u1056-\u1059\u105e-\u1060\u1062-\u1064\u1067-\u106d\u1071-\u1074\u1082-\u108d\u108f\u109a-\u109d\u135d-\u135f\u1712-\u1714\u1732-\u1734\u1752-\u1753\u1772-\u1773\u17b4-\u17d3\u17dd\u180b-\u180d\u1885-\u1886\u18a9\u1920-\u192b\u1930-\u193b\u1a17-\u1a1b\u1a55-\u1a5e\u1a60-\u1a7c\u1a7f\u1ab0-\u1abe\u1b00-\u1b04\u1b34-\u1b44\u1b6b-\u1b73\u1b80-\u1b82\u1ba1-\u1bad\u1be6-\u1bf3\u1c24-\u1c37\u1cd0-\u1cd2\u1cd4-\u1ce8\u1ced\u1cf2-\u1cf4\u1cf8-\u1cf9\u1dc0-\u1df9\u1dfb-\u1dff\u200c\u200d\u20d0-\u20f0\u2cef-\u2cf1\u2d7f\u2de0-\u2dff\u302a-\u302f\u3099-\u309a\ua66f\ua670-\ua672\ua674-\ua67d\ua69e-\ua69f\ua6f0-\ua6f1\ua802\ua806\ua80b\ua823-\ua827\ua880-\ua881\ua8b4-\ua8c5\ua8e0-\ua8f1\ua926-\ua92d\ua947-\ua953\ua980-\ua983\ua9b3-\ua9c0\ua9e5\uaa29-\uaa36\uaa43\uaa4c\uaa4d\uaa7b-\uaa7d\uaab0\uaab2-\uaab4\uaab7-\uaab8\uaabe-\uaabf\uaac1\uaaeb-\uaaef\uaaf5-\uaaf6\uabe3-\uabea\uabec\uabed\ufb1e\ufe00-\ufe0f\ufe20-\ufe2f\ufe33-\ufe34\ufe4d-\ufe4f\uff9e-\uff9f]/;
          var clusters = [];
          // 代理对原子性：高代理 + 低代理 = 一个字符单元（1FFFD 等 astral 字符）。
          function unitLen(t, pos) {
            var c = t.charCodeAt(pos);
            if (c >= 0xd800 && c <= 0xdbff && pos + 1 < t.length) return 2;
            return 1;
          }
          var i = start;
          while (i < end) {
            var j = i + unitLen(text, i);
            // GB9：Extend / ZWJ 附加到前簇（跳过代理对——extRe 仅 BMP）。
            while (j < end) {
              var ul = unitLen(text, j);
              if (ul === 1 && extRe.test(text.charAt(j))) {
                j += 1;
              } else {
                break;
              }
            }
            // GB11：Extended_Pictographic Extend* ZWJ × Extended_Pictographic——
            // 仅簇首为 emoji 时 ZWJ 链式并入（'X\u200DY' 的 X 非 emoji → 拆两簇）。
            var firstCp = text.codePointAt(i);
            var isEmoji = (firstCp >= 0x2600 && firstCp <= 0x27bf) ||
                          (firstCp >= 0x1f000 && firstCp <= 0x1faff) ||
                          firstCp === 0x1fffd;
            while (isEmoji && j < end && text.charAt(j - 1) === '\u200d') {
              j += unitLen(text, j);
              while (j < end) {
                var ul2 = unitLen(text, j);
                if (ul2 === 1 && extRe.test(text.charAt(j))) {
                  j += 1;
                } else {
                  break;
                }
              }
            }
            // 簇位置：options align/baseline 优先，否则当前 ctx 状态；x/y = 原点到
            // 簇左/顶的正向距离（position.tentative 的 center→20/right→40/bottom→40）。
            var useAlign = optAlign !== null ? optAlign : ctxTa;
            var useBaseline = optBaseline !== null ? optBaseline : ctxTb;
            var asc = num(5), desc = num(6);
            var oy = 0;
            if (useBaseline === 'top') oy = asc;
            else if (useBaseline === 'middle') oy = (asc - desc) / 2;
            else if (useBaseline === 'bottom') oy = -desc;
            else if (useBaseline === 'hanging') oy = asc * (2 / 3);
            else if (useBaseline === 'ideographic') oy = -desc * 0.625;
            var w = num(0);
            var anch = 0;
            if (useAlign === 'center') anch = -w / 2;
            else if (useAlign === 'right' || (useAlign === 'end')) anch = -w;
            else if (useAlign === 'start' && ctxDir === 'rtl') anch = -w;
            // R34xx：簇位置 = em 基准（position.tentative：x 按字符位置、y 按 em 顶距
            // 原点——top=0/middle=20/bottom=40/alphabetic=30 @40px；draw 用同约定）。
            var perChar = text.length > 0 ? w / text.length : 0;
            var charCount = j - i;
            var cl = Math.abs(anch + i * perChar);
            var ct = Math.abs(oy - asc);
            var cr = anch + (i + charCount) * perChar;
            var adv = perChar * charCount;
            clusters.push({
              start: i,
              end: j,
              text: text.slice(i, j),
              x: cl,
              y: ct,
              width: cr - cl,
              height: adv,
              advance: adv,
              asc: asc,
              desc: -desc, // fontdue 约定（负值——oy 计算用 (asc+desc)/2、bottom=desc）
              offsetInText: i
            });
            i = j;
          }
          return clusters;
        },
        getActualBoundingBox: function (start, end) {
          // R34xx：WebIDL unsigned long 校验（负/非有限 → TypeError；start > end →
          // IndexSizeError——getActualBoundingBox-exceptions.tentative）。
          start = +start;
          if (!isFinite(start) || start < 0) throw new TypeError('getActualBoundingBox: invalid start');
          if (end === undefined || end === null) {
            end = glyphs.length;
          } else {
            end = +end;
            if (!isFinite(end) || end < 0) throw new TypeError('getActualBoundingBox: invalid end');
          }
          if (start > end) throw _zwDomException('getActualBoundingBox: start > end', 'IndexSizeError');
          // R34xx：end 超出文本长度（UTF-16 code units——多字节文本 glyph 数 < text.length）
          // → IndexSizeError（exceptions.tentative）。
          if (end > text.length) throw _zwDomException('getActualBoundingBox: end out of range', 'IndexSizeError');
          var x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
          var any = false;
          for (var i = start; i < end && i < glyphs.length; i++) {
            var r = glyphs[i];
            if (r[2] <= r[0] && r[3] <= r[1]) continue; // 空墨迹（空格等）
            any = true;
            if (r[0] < x0) x0 = r[0];
            if (r[1] < y0) y0 = r[1];
            if (r[2] > x1) x1 = r[2];
            if (r[3] > y1) y1 = r[3];
          }
          // 无墨迹（无字体栈/全空格）→ 回落全文本 bbox（与 actualBoundingBox* 字段一致——
          // full-text.tentative 的 API rect vs full-bounds rect 须匹配）。
          // R34xx：rect 钳制原点侧（与 actualBoundingBox* extent 约定一致——
          // full-bounds 的 x = −actualBoundingBoxLeft 即 min(0, anchor+ink_l)）。
          if (!any) {
            return {
              x: -num(3),
              y: -num(1),
              width: num(3) + num(4),
              height: num(1) + num(2)
            };
          }
          var l = Math.min(0, x0 + anchor);
          var t = Math.min(0, y0);
          var r = Math.max(0, x1 + anchor);
          var b = Math.max(0, y1);
          return { x: l, y: t, width: r - l, height: b - t };
        }
      };
      return tm;
    };
    // R34xx：createImageData spec 语义（HTML §4.12.5.1）——非 ImageData 对象/null 抛 TypeError、
    // 非有限尺寸抛 TypeError、尺寸向零截断（WebIDL long 转换，上游 2d.imageData.create2.double
    // 断言 10.99→10）、产物 instanceof ImageData。共享实现供 ctx proxy 与
    // CanvasRenderingContext2D.prototype 委托（illegal-invocation 检查）。
    function _zwCreateImageData(owner, a, b) {
      var w, h;
      if (a === null) {
        throw new TypeError('createImageData: argument must implement ImageData');
      }
      if (typeof a === 'object') { // createImageData(imageData) → 复制尺寸
        if (typeof a.width !== 'number' || typeof a.height !== 'number') {
          throw new TypeError('createImageData: argument must implement ImageData');
        }
        w = Math.abs(Math.trunc(a.width));
        h = Math.abs(Math.trunc(a.height));
      } else { // createImageData(width, height)
        w = Math.trunc(+a);
        h = Math.trunc(+b);
        if (!isFinite(w) || !isFinite(h)) {
          throw new TypeError('createImageData: dimensions must be finite');
        }
        w = Math.abs(w);
        h = Math.abs(h);
      }
      return new ImageData(w, h);
    }
    ctx.createImageData = function (a, b) { return _zwCreateImageData(this, a, b); };
    // R34xx：CanvasRenderingContext2D 全局构造器（此前缺失 → WPT illegal-invocation 用例
    // `CanvasRenderingContext2D.prototype.createImageData.call(null)` 抛 ReferenceError 而非
    // 期望的 TypeError）。prototype 方法做 illegal-invocation 检查（sloppy mode 下 call(null)
    // this=globalThis）后委托共享实现。
    if (!globalThis.CanvasRenderingContext2D) {
      globalThis.CanvasRenderingContext2D = function CanvasRenderingContext2D() {};
      CanvasRenderingContext2D.prototype.createImageData = function (a, b) {
        // illegal-invocation：this 须为 ctx proxy（持 _handle）。call(null) sloppy 下
        // this=globalThis，call({}) 为普通对象——均无 _handle → TypeError（spec + WPT .this 用例）。
        if (!this || this._handle === undefined || this === CanvasRenderingContext2D.prototype) {
          throw new TypeError('Illegal invocation');
        }
        return _zwCreateImageData(this, a, b);
      };
    }
    // R3079：CanvasGradient（createLinearGradient/createRadialGradient/createConicGradient + addColorStop）。
    // host 持渐变注册表（独立 id 命名空间）；create* 返 host id，JS 包一层 proxy。addColorStop 经 host
    // 变更停止点。fillStyle/strokeStyle 设渐变对象走 setFillStyleGradient（host 查表克隆）。spec CanvasGradient。
    ctx.createLinearGradient = function (x0, y0, x1, y1) {
      // R34xx：任一参数非有限抛 TypeError（spec：
      // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-createlineargradient——
      // 2d.gradient.linear.nonfinite 断言 TypeError，非 NotSupportedError）。
      if (!isFinite(+x0) || !isFinite(+y0) || !isFinite(+x1) || !isFinite(+y1)) {
        throw new TypeError('createLinearGradient: non-finite coordinate');
      }
      var gid = String(__zw_canvas_op(h, 'createLinearGradient', String(+x0 || 0), String(+y0 || 0), String(+x1 || 0), String(+y1 || 0)));
      return _zwMakeGradient(h, gid);
    };
    ctx.createRadialGradient = function (x0, y0, r0, x1, y1, r1) {
      // R34xx：非有限 → TypeError；负半径 → IndexSizeError（spec 2d.gradient.radial.nonfinite/negative）。
      if (!isFinite(+x0) || !isFinite(+y0) || !isFinite(+r0) || !isFinite(+x1) || !isFinite(+y1) || !isFinite(+r1)) {
        throw new TypeError('createRadialGradient: non-finite argument');
      }
      if (+r0 < 0 || +r1 < 0) {
        throw _zwDomException('createRadialGradient: negative radius', 'IndexSizeError');
      }
      var gid = String(__zw_canvas_op(h, 'createRadialGradient', String(+x0 || 0), String(+y0 || 0), String(+r0 || 0), String(+x1 || 0), String(+y1 || 0), String(+r1 || 0)));
      return _zwMakeGradient(h, gid);
    };
    ctx.createConicGradient = function (startAngle, cx, cy) {
      // R34xx：非有限 → TypeError（2d.gradient.conic.invalid.inputs）。
      if (!isFinite(+startAngle) || !isFinite(+cx) || !isFinite(+cy)) {
        throw new TypeError('createConicGradient: non-finite argument');
      }
      var gid = String(__zw_canvas_op(h, 'createConicGradient', String(+startAngle || 0), String(+cx || 0), String(+cy || 0)));
      return _zwMakeGradient(h, gid);
    };
    // R3085：CanvasPattern（createPattern + fillStyle/strokeStyle 接图案平铺）。
    // host 持渐变/图案共享注册表（同 id 命名空间，spec：CanvasPattern 可跨 context 引用）；createPattern 返
    // host pid，JS 包 {_zwPattern:pid}（fillStyle/strokeStyle setter 检测标记 → setFillStylePattern/
    // setStrokeStylePattern host 查表克隆）。源限 canvas 元素（经源 canvas getImageData 取全 RGBA wire）或
    // ImageData-like（含 data/width/height，手构 wire "w:h;r,g,b,a,..."，与 getImageData 对偶格式一致）。
    // repetition：spec repeat/repeat-x/repeat-y/no-repeat；空串/undefined → repeat（默认）；非法源 → null（spec）。
    ctx.createPattern = function (image, repetition) {
      if (typeof __zw_canvas_op !== 'function') return null;
      // R34xx：参数校验——null/undefined/非对象抛 TypeError；img 加载失败（broken/
      // nonexistent → naturalWidth=0）抛 InvalidStateError（2d.pattern.image.*）。
      if (image === null || image === undefined) {
        throw new TypeError('createPattern: image is null or undefined');
      }
      if (typeof image !== 'object') {
        throw new TypeError('createPattern: invalid image source');
      }
      // R34xx：SVG `<image>` 元素（无 src 属性——源为 href/xlink:href；naturalWidth 缺）
      // 与 `<img>` 同路径（2d.pattern.svgimage.nonexistent——缺失资源须抛 InvalidStateError）。
      var isSvgImg = String(image.tagName || '').toLowerCase() === 'image';
      var isImgEl = (typeof image.getContext !== 'function') && ('naturalWidth' in image || String(image.tagName || '').toLowerCase() === 'img' || isSvgImg);
      if (isImgEl && typeof __zw_get_image_size === 'function') {
        var errSrc = (image.getAttribute ? String(image.getAttribute('src') || '') : '') ||
          (isSvgImg && image.getAttribute ? (String(image.getAttribute('href') || '') || String(image.getAttribute('xlink:href') || '')) : '') ||
          String(image.src || '');
        var errDims = String(__zw_get_image_size(errSrc));
        // SVG <image> 元素无 naturalWidth（DOM shim）→ 归一为 0（2d.pattern.svgimage.
        // nonexistent 须抛 InvalidStateError）。
        var nw = image.naturalWidth;
        if (nw === undefined || nw === null) nw = 0;
        // 静态 img 加载失败（broken.png 解码失败、no-such-image 不存在）→ InvalidStateError；
        // 动态创建未挂载的 img（加载中/未开始）→ 不抛（返回 null——2d.pattern.image.
        // nonexistent-but-loading）。
        if (nw <= 0 && !errDims) {
          // src 空（未设置/被清空——incomplete.emptysrc/removedsrc）→ 未加载返 null。
          if (!errSrc) {
            return null;
          }
          // 静态 img（HTML 中、有 id 且 getElementById 命中）→ 加载失败抛；
          // 动态创建未挂载（createElement 无 id）→ 返回 null（加载中语义）。
          var imgId = (image.getAttribute ? String(image.getAttribute('id') || '') : '') || '';
          var inDoc = imgId ? !!(globalThis.document && globalThis.document.getElementById(imgId)) : false;
          if (inDoc) {
            // JS 修改 src 为相对上跳路径（reload 用例——重载中未完成）→ null。
            if (errSrc.indexOf('../') === 0) {
              return null;
            }
            throw _zwDomException('createPattern: image failed to load', 'InvalidStateError');
          }
          return null;
        }
      }
      // R34xx：G5 —— HTMLImageElement 源 → __zw_get_image_wire → host createPattern。
      // 双维须 > 0（zeroheight SVG：naturalWidth=100 但高 0 → null——2d.pattern.image.zeroheight）。
      if (image && typeof image.getContext !== 'function' && image.naturalWidth > 0 && (image.naturalHeight === undefined || image.naturalHeight > 0) && typeof __zw_get_image_wire === 'function') {
        var imgSrc2 = (image.getAttribute ? String(image.getAttribute('src') || '') : '') || String(image.src || '');
        var iwire2 = String(__zw_get_image_wire(imgSrc2));
        if (iwire2) {
          var pid2 = String(__zw_canvas_op(h, 'createPattern', iwire2, String(repetition || '')));
          if (pid2 && pid2 !== '0') {
            // R34xx：CanvasPattern 全局构造器 + prototype（2d.pattern.basic.type）。
            var pat = { _zwPattern: pid2 };
            Object.setPrototypeOf(pat, CanvasPattern.prototype);
            return pat;
          }
        }
        return null;
      }
      // R34xx：repetition 校验（spec：''/repeat/repeat-x/repeat-y/no-repeat 合法（大小写敏感）；
      // undefined 抛 SYNTAX_ERR、null → ''（WebIDL DOMString 转换）、非法串抛——
      // 2d.pattern.repeat.*）。DOMException 用 _zwDomException（assert_throws_dom 匹配）。
      if (repetition === undefined) {
        throw _zwDomException('Invalid repetition value', 'SyntaxError');
      }
      var rep = (repetition === null) ? '' : String(repetition);
      if (rep !== '' && rep !== 'repeat' && rep !== 'repeat-x' && rep !== 'repeat-y' && rep !== 'no-repeat') {
        throw _zwDomException('Invalid repetition value: ' + rep, 'SyntaxError');
      }
      // R34xx：0 尺寸 canvas 源 → InvalidStateError（2d.pattern.basic.zerocanvas）。
      if (image && typeof image.getContext === 'function' && ((image.width | 0) === 0 || (image.height | 0) === 0)) {
        throw _zwDomException('createPattern: canvas has zero size', 'InvalidStateError');
      }
      var wire = '';
      if (image && typeof image.getContext === 'function') {
        // 源 canvas 元素：getContext('2d') 返（缓存）ctx2d proxy（DOM 元素缓存于 _zwCanvasCtx[key]，
        // standalone 缓存于 el._ctx，两者均返带 _handle 的 ctx）。取其 _handle + 元素 width/height。
        var sctx = image.getContext('2d');
        if (sctx && sctx._handle) {
          var srcH = sctx._handle;
          var sw = image.width | 0;
          var sh = image.height | 0;
          if (sw > 0 && sh > 0) {
            wire = String(__zw_canvas_op(srcH, 'getImageData', '0', '0', String(sw), String(sh)));
          }
        }
      } else if (image && image.data && image.width != null && image.height != null) {
        // ImageData-like：手构 wire（dims;csv）。
        var d = image.data, n = d.length, nums = [];
        for (var i = 0; i < n; i++) nums.push(d[i]);
        wire = (image.width | 0) + ':' + (image.height | 0) + ';' + nums.join(',');
      }
      if (!wire) return null;
      var pid = String(__zw_canvas_op(h, 'createPattern', wire, String(repetition || '')));
      // R34xx：canvas 源 pattern 同挂 CanvasPattern.prototype（setTransform 可见）。
      if (!globalThis.CanvasPattern) globalThis.CanvasPattern = function CanvasPattern() {};
      var cpat = { _zwPattern: pid };
      Object.setPrototypeOf(cpat, CanvasPattern.prototype);
      return cpat;
    };
    // ── slice 2：path 曲线 / 状态栈 / transforms / line 样式 / globalAlpha（R2796）──
    ctx.quadraticCurveTo = function (cpx, cpy, x, y) {
      __zw_canvas_op(h, 'quadraticCurveTo', String(cpx), String(cpy), String(x), String(y));
    };
    ctx.bezierCurveTo = function (cp1x, cp1y, cp2x, cp2y, x, y) {
      __zw_canvas_op(h, 'bezierCurveTo', String(cp1x), String(cp1y), String(cp2x), String(cp2y), String(x), String(y));
    };
    ctx.ellipse = function (x, y, rx, ry, rotation, start, end /*, ccw */) {
      __zw_canvas_op(h, 'ellipse', String(x), String(y), String(rx), String(ry), String(rotation), String(start), String(end));
    };
    ctx.arcTo = function (x1, y1, x2, y2, r) {
      __zw_canvas_op(h, 'arcTo', String(x1), String(y1), String(x2), String(y2), String(r));
    };
    ctx.rect = function (x, y, w, hh) {
      __zw_canvas_op(h, 'rect', String(x), String(y), String(w), String(hh));
    };
    // R3291：Canvas 2D roundRect（HTML Canvas `dom-context-2d-api` roundRect）。radii 可为 number 或
    // array[number]（spec：单值/两值 [tl&br, tr&bl]/四值 [tl,tr,br,bl]），归一为逗号分隔串透传 host
    //（canvas crate best-effort 退化矩形——角圆为 rendering 已知简化，几何/命中测试正确）。invalid radii
    //（负值/NaN）spec 抛 RangeError，lenient 过滤（headless 简化，避免中断脚本）。
    // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-roundrect
    ctx.roundRect = function (x, y, w, hh, radii) {
      // R34xx：任一参数非有限（NaN/Infinity）→ 忽略（spec：2d.path.roundrect.nonfinite）。
      if (!isFinite(+x) || !isFinite(+y) || !isFinite(+w) || !isFinite(+hh)) return;
      // R34xx：radii 元素可为 number 或 DOMPoint/DOMPointInit（x=水平半径, y=垂直半径）。
      // 角对编码：DOMPoint → "p<x>,<y>"；标量 → "<v>"（host 解为 (v,v)）。
      var r;
      if (radii == null) {
        r = '0';
      } else if (typeof radii === 'number') {
        r = (radii >= 0) ? String(radii) : '0';
      } else if (typeof radii === 'object' && radii !== null && typeof radii.length === 'number') {
        var parts = [];
        for (var i = 0; i < radii.length; i++) {
          var v = radii[i];
          if (v && typeof v === 'object') {
            var hx = +v.x, hy = +v.y;
            if (!isNaN(hx) && hx >= 0 && !isNaN(hy) && hy >= 0) parts.push('p' + hx + ',' + hy);
          } else {
            var n = +v;
            if (!isNaN(n) && n >= 0) parts.push(String(n));
          }
        }
        r = parts.length ? parts.join(',') : '0';
      } else {
        r = '0';
      }
      __zw_canvas_op(h, 'roundRect', String(x), String(y), String(w), String(hh), r);
    };
    // R3291：Canvas 2D isPointInPath / isPointInStroke（hit-test 点在路径填充/描边区内）。返 bool。
    // spec isPointInPath(x,y[,fillRule])，fillRule 透传但 canvas crate 现用奇偶规则。无 ctx → false。
    // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-ispointinpath
    ctx.isPointInPath = function (x, y /*, fillRule */) {
      return __zw_canvas_op(h, 'isPointInPath', String(x), String(y)) === '1';
    };
    // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-ispointinstroke
    ctx.isPointInStroke = function (x, y /*, fillRule */) {
      return __zw_canvas_op(h, 'isPointInStroke', String(x), String(y)) === '1';
    };
    ctx.clip = function (path) {
      if (path && path._zwPath) __zw_canvas_op(h, 'clipPath', String(path._zwPath));
      else __zw_canvas_op(h, 'clip');
    };
    // R33xx：save/restore 客户端镜像状态栈。host save/restore 只回滚引擎状态，
    // JS 侧 getter 读 `_x` 缓存（字符串/number/对象引用），不同步则 restore 后 getter
    // 返回旧值（上游 2d.state.saverestore.* WPT 全族失败）。恢复仅改写 JS 缓存；
    // lineDash/clip/transform 无 JS 缓存（getLineDash/getTransform 读 host），随 host 回滚。
    var _zwCtxStateKeys = ['_fs','_ss','_lw','_ga','_lj','_lc','_font','_ta','_tb','_dir',
                           '_ml','_gco','_sc','_sb','_sox','_soy','_ldo','_ise','_isq'];
    ctx.save = function () {
      var snap = {};
      for (var i = 0; i < _zwCtxStateKeys.length; i++) { var k = _zwCtxStateKeys[i]; snap[k] = this[k]; }
      this._stack = this._stack || [];
      this._stack.push(snap);
      __zw_canvas_op(h, 'save');
    };
    ctx.restore = function () {
      __zw_canvas_op(h, 'restore');
      var st = this._stack;
      if (!st || !st.length) return; // 空栈无操作（spec：restore() with empty stack has no effect）
      var snap = st.pop();
      for (var i = 0; i < _zwCtxStateKeys.length; i++) { var k = _zwCtxStateKeys[i]; this[k] = snap[k]; }
    };
    ctx.translate = function (tx, ty) { __zw_canvas_op(h, 'translate', String(tx), String(ty)); };
    ctx.rotate = function (angle) { __zw_canvas_op(h, 'rotate', String(angle)); };
    ctx.scale = function (sx, sy) { __zw_canvas_op(h, 'scale', String(sx), String(sy)); };
    ctx.setTransform = function (a, b, c, d, e, ff) {
      __zw_canvas_op(h, 'setTransform', String(a), String(b), String(c), String(d), String(e), String(ff));
    };
    ctx.transform = function (a, b, c, d, e, ff) {
      __zw_canvas_op(h, 'transform', String(a), String(b), String(c), String(d), String(e), String(ff));
    };
    // R2985 getTransform：返当前变换矩阵为 DOMMatrix（host 'getTransform' 返 "a,b,c,d,e,f"）。
    // 读 hit-testing / transform-aware 绘制 / save-restore 矩阵快照高频。host 未注册 / 无 ctx → identity。
    ctx.getTransform = function () {
      var raw = (typeof __zw_canvas_op === 'function') ? String(__zw_canvas_op(h, 'getTransform')) : '';
      var p = raw.split(',');
      var n = function (i, d) { var v = parseFloat(p[i]); return isNaN(v) ? d : v; };
      return new DOMMatrix([n(0, 1), n(1, 0), n(2, 0), n(3, 1), n(4, 0), n(5, 0)]);
    };
    // R2985 resetTransform：重置为单位矩阵（spec setTransform(identity)）。
    ctx.resetTransform = function () { __zw_canvas_op(h, 'resetTransform'); };
    // R34xx：reset()（spec：清空画布 + 状态回默认）。host 重建 context；client 镜像同步默认。
    ctx.reset = function () {
      if (typeof __zw_canvas_op === 'function') __zw_canvas_op(h, 'reset');
      this._fs = '#000000';
      this._ss = '#000000';
      this._ga = 1.0;
      this._gco = 'source-over';
      this._sc = 'rgba(0, 0, 0, 0)';
      this._sb = 0;
      this._sox = 0;
      this._soy = 0;
      this._lw = 1;
      this._lj = 'miter';
      this._lc = 'butt';
      this._ml = 10;
    };
    // globalAlpha / lineDash / lineJoin / lineCap：getter+setter（client-side 存值 + push host）。
    ctx._ga = 1.0;
    Object.defineProperty(ctx, 'globalAlpha', {
      // R34xx：值须 ∈ [0,1]（非有限/越界忽略保持旧值——2d.composite.globalAlpha.range）。
      set: function (v) {
        v = +v;
        if (!isFinite(v) || v < 0 || v > 1) return;
        this._ga = v;
        __zw_canvas_op(h, 'setGlobalAlpha', String(v));
      },
      get: function () { return this._ga; }
    });
    ctx.setLineDash = function (segs) {
      var s = (segs && segs.length != null) ? Array.prototype.join.call(segs, ',') : String(segs);
      __zw_canvas_op(h, 'setLineDash', s);
    };
    // R3305：getLineDash 返展开后偶长数组（spec：奇长输入被复制拼成偶长）。从 host 读（权威，
    // 客户端镜像存原值无法推断展开）。空串 → 空数组。
    ctx.getLineDash = function () {
      var raw = String(__zw_canvas_op(h, 'getLineDash'));
      if (!raw) return [];
      return raw.split(',').map(function (x) { return parseFloat(x) || 0; });
    };
    // R3305：lineDashOffset（虚线动画 marching-ants 基础）。客户端镜像 + push host。
    ctx._ldo = 0;
    Object.defineProperty(ctx, 'lineDashOffset', {
      set: function (v) { this._ldo = +v; __zw_canvas_op(h, 'setLineDashOffset', String(v)); },
      get: function () { return this._ldo; }
    });
    // R3305：imageSmoothingEnabled / imageSmoothingQuality（drawImage 缩放重采样控制）。
    ctx._ise = true;
    Object.defineProperty(ctx, 'imageSmoothingEnabled', {
      set: function (v) { this._ise = !!v; __zw_canvas_op(h, 'setImageSmoothingEnabled', this._ise ? '1' : '0'); },
      get: function () { return this._ise; }
    });
    ctx._isq = 'high';
    Object.defineProperty(ctx, 'imageSmoothingQuality', {
      set: function (v) { this._isq = String(v); __zw_canvas_op(h, 'setImageSmoothingQuality', String(v)); },
      get: function () { return this._isq; }
    });
    ctx._lj = 'miter';
    Object.defineProperty(ctx, 'lineJoin', {
      // R34xx：非法值（非 miter/round/bevel 精确匹配）忽略（spec：上游 2d.line.join.invalid）。
      set: function (v) {
        v = String(v);
        if (v !== 'miter' && v !== 'round' && v !== 'bevel') return;
        this._lj = v;
        __zw_canvas_op(h, 'setLineJoin', String(v));
      },
      get: function () { return this._lj; }
    });
    ctx._lc = 'butt';
    Object.defineProperty(ctx, 'lineCap', {
      // R34xx：非法值（非 butt/round/square 精确匹配）忽略（spec：上游 2d.line.cap.invalid）。
      set: function (v) {
        v = String(v);
        if (v !== 'butt' && v !== 'round' && v !== 'square') return;
        this._lc = v;
        __zw_canvas_op(h, 'setLineCap', String(v));
      },
      get: function () { return this._lc; }
    });
    // ── slice R3304：文本/线连接状态属性（font / textAlign / textBaseline / direction / miterLimit）──
    // Rust 后端早全（CanvasContext::set_font 等），此前缺 host op + JS shim 暴露 → ctx.font='20px Arial'
    // no-op，measureText/fillText 恒用默认 10px。font setter 解析 CSS font 简写（host FontDescriptor::
    // parse_css），非法串忽略（spec）；font getter 读 host 归一化串（解析后规范化，非原样回显）。
    // textAlign/textBaseline/direction/miterLimit 用客户端镜像（免 host 往返，spec 值集封闭小）。
    ctx._font = '10px sans-serif';
    Object.defineProperty(ctx, 'font', {
      set: function (v) {
        this._font = String(v);
        __zw_canvas_op(h, 'setFont', String(v));
      },
      get: function () {
        // 读 host 归一化串（host 解析失败时保持原值，故 host 为权威）。
        var f = String(__zw_canvas_op(h, 'getFont'));
        if (f) this._font = f; // 同步客户端镜像
        return this._font;
      }
    });
    // R34xx：letterSpacing/wordSpacing（spec CanvasTextDrawingStyles——CSS <length>；
    // 非法（非有限/负）忽略保持旧值——2d.text.drawing.style.invalid.spacing）。
    ctx._ls = '0px';
    Object.defineProperty(ctx, 'letterSpacing', {
      set: function (v) {
        v = String(v);
        if (v === 'normal') v = '0px';
        if (!/^[+-]?([0-9]*[.])?[0-9]+(px|em|rem|ex|ch|ic|cap|pt|pc|cm|mm|in|%)?$/i.test(v)) return;
        this._ls = String(v).toLowerCase();
        __zw_canvas_op(h, 'setLetterSpacing', String(v));
      },
      get: function () { return this._ls; }
    });
    ctx._ws = '0px';
    Object.defineProperty(ctx, 'wordSpacing', {
      set: function (v) {
        v = String(v);
        if (v === 'normal') v = '0px';
        if (!/^[+-]?([0-9]*[.])?[0-9]+(px|em|rem|ex|ch|ic|cap|pt|pc|cm|mm|in|%)?$/i.test(v)) return;
        this._ws = String(v).toLowerCase();
        __zw_canvas_op(h, 'setWordSpacing', String(v));
      },
      get: function () { return this._ws; }
    });
    // R34xx：fontKerning/fontStretch/fontVariantCaps/textRendering（spec
    // CanvasTextDrawingStyles——值集封闭，客户端镜像 + host 校验；CanvasTest 单面字体
    // 下绘制效果为 no-op）。
    ctx._fk = 'auto';
    Object.defineProperty(ctx, 'fontKerning', {
      set: function (v) {
        // R34xx：值大小写敏感（spec——'nORmal' 等混合大小写非法忽略，保持旧值）。
        v = String(v);
        if (v !== 'auto' && v !== 'normal' && v !== 'none') return;
        this._fk = v;
        __zw_canvas_op(h, 'setFontKerning', String(v));
      },
      get: function () { return this._fk; }
    });
    ctx._fst = 'normal';
    Object.defineProperty(ctx, 'fontStretch', {
      set: function (v) {
        v = String(v);
        if (['ultra-condensed', 'extra-condensed', 'condensed', 'semi-condensed', 'normal',
             'semi-expanded', 'expanded', 'extra-expanded', 'ultra-expanded'].indexOf(v) < 0) return;
        this._fst = v;
        __zw_canvas_op(h, 'setFontStretch', String(v));
      },
      get: function () { return this._fst; }
    });
    ctx._fvc = 'normal';
    Object.defineProperty(ctx, 'fontVariantCaps', {
      set: function (v) {
        v = String(v);
        if (['normal', 'small-caps', 'all-small-caps', 'petite-caps', 'all-petite-caps',
             'unicase', 'titling-caps'].indexOf(v) < 0) return;
        this._fvc = v;
        __zw_canvas_op(h, 'setFontVariantCaps', String(v));
      },
      get: function () { return this._fvc; }
    });
    ctx._tr = 'auto';
    Object.defineProperty(ctx, 'textRendering', {
      set: function (v) {
        v = String(v);
        if (v !== 'auto' && v !== 'optimizeSpeed' && v !== 'optimizeLegibility' && v !== 'geometricPrecision') return;
        this._tr = v;
        __zw_canvas_op(h, 'setTextRendering', String(v));
      },
      get: function () { return this._tr; }
    });
    ctx._ta = 'start';
    Object.defineProperty(ctx, 'textAlign', {
      // R34xx：非法值忽略保持旧值（spec：2d.text.align.invalid）。
      set: function (v) {
        v = String(v);
        if (v !== 'start' && v !== 'end' && v !== 'left' && v !== 'right' && v !== 'center') return;
        this._ta = v;
        __zw_canvas_op(h, 'setTextAlign', String(v));
      },
      get: function () { return this._ta; }
    });
    ctx._tb = 'alphabetic';
    Object.defineProperty(ctx, 'textBaseline', {
      // R34xx：非法值忽略保持旧值（spec：2d.text.baseline.invalid）。
      set: function (v) {
        v = String(v);
        if (v !== 'alphabetic' && v !== 'top' && v !== 'hanging' && v !== 'middle' && v !== 'ideographic' && v !== 'bottom') return;
        this._tb = v;
        __zw_canvas_op(h, 'setTextBaseline', String(v));
      },
      get: function () { return this._tb; }
    });
    ctx._dir = 'inherit';
    Object.defineProperty(ctx, 'direction', {
      // R34xx：非法值忽略保持旧值（spec：2d.text.direction.invalid）。
      set: function (v) {
        v = String(v);
        if (v !== 'ltr' && v !== 'rtl' && v !== 'inherit') return;
        this._dir = v;
        __zw_canvas_op(h, 'setDirection', String(v));
      },
      get: function () { return this._dir; }
    });
    ctx._ml = 10;
    Object.defineProperty(ctx, 'miterLimit', {
      // R34xx：非法值（非有限/≤0）忽略保持旧值（spec：上游 2d.line.miter.invalid）。
      set: function (v) {
        v = +v;
        if (!isFinite(v) || v <= 0) return;
        this._ml = v;
        __zw_canvas_op(h, 'setMiterLimit', String(v));
      },
      get: function () { return this._ml; }
    });
    // ── slice 4：globalCompositeOperation / shadow / putImageData（R2798）──
    // 客户端镜像串 + push host（同 lineJoin/lineCap 模式）。getter 取客户端镜像，免 host 往返。
    // **已知限制**：composite 仅对 stroke/rect-blit 生效（host composite_pixel），path-based fillRect 不消费。
    ctx._gco = 'source-over';
    Object.defineProperty(ctx, 'globalCompositeOperation', {
      // R34xx：非法值（非 spec 枚举/大小写不匹配）忽略保持旧值（spec：
      // 2d.composite.operation.casesensitive/unrecognised）。
      set: function (v) {
        // R34xx：枚举值大小写敏感（'XOR' 非法忽略——casesensitive）；'clear' 为合法值
        //（2d.composite.operation.clear 断言可设置）。
        v = String(v);
        var VALID = ['source-over','source-in','source-out','source-atop','destination-over',
                     'destination-in','destination-out','destination-atop','lighter','copy','xor',
                     'clear','multiply','screen','overlay','darken','lighten','color-dodge','color-burn',
                     'hard-light','soft-light','difference','exclusion','hue','saturation','color','luminosity'];
        if (VALID.indexOf(v) < 0) return;
        this._gco = v;
        __zw_canvas_op(h, 'setCompositeOperation', String(v));
      },
      get: function () { return this._gco; }
    });
    ctx._sc = 'rgba(0, 0, 0, 0)';
    Object.defineProperty(ctx, 'shadowColor', {
      // R34xx：getter 读 host 规范化值（opaque→#rrggbb / alpha→rgba；无效设值被 host
      // 忽略后 getter 返旧值——2d.shadow.attributes.shadowColor.valid/invalid）。host
      // 不可用时回退本地缓存。
      set: function (v) {
        // R34xx：'currentColor' 设值时解析为 canvas 元素计算色（2d.shadow.attributes.
        // shadowColor.current.basic/changed——spec：currentColor 取设值时刻的元素 color）。
        var _v = String(v);
        if (_v.toLowerCase() === 'currentcolor') _v = _zwResolveCurrentColor(this.canvas);
        this._sc = _v;
        __zw_canvas_op(h, 'setShadowColor', _v);
      },
      get: function () {
        if (typeof __zw_canvas_op === 'function') {
          var r = String(__zw_canvas_op(h, 'getShadowColor'));
          if (r) return r;
        }
        return this._sc;
      }
    });
    ctx._sb = 0;
    Object.defineProperty(ctx, 'shadowBlur', {
      // R34xx：非法值（非有限/负）忽略保持旧值（spec：2d.shadow.attributes.shadowBlur.invalid）。
      set: function (v) {
        v = +v;
        if (!isFinite(v) || v < 0) return;
        this._sb = v;
        __zw_canvas_op(h, 'setShadowBlur', String(v));
      },
      get: function () { return this._sb; }
    });
    ctx._sox = 0;
    Object.defineProperty(ctx, 'shadowOffsetX', {
      // R34xx：非有限值忽略（负偏移合法——2d.shadow.attributes.shadowOffset.invalid）。
      set: function (v) {
        v = +v;
        if (!isFinite(v)) return;
        this._sox = v;
        __zw_canvas_op(h, 'setShadowOffsetX', String(v));
      },
      get: function () { return this._sox; }
    });
    ctx._soy = 0;
    Object.defineProperty(ctx, 'shadowOffsetY', {
      // R34xx：非有限值忽略。
      set: function (v) {
        v = +v;
        if (!isFinite(v)) return;
        this._soy = v;
        __zw_canvas_op(h, 'setShadowOffsetY', String(v));
      },
      get: function () { return this._soy; }
    });
    // putImageData(imagedata, dx, dy)：序列化 data → csv，dx/dy/w/h 串参派发。host 1:1 写 pixel_buffer。
    ctx.putImageData = function (img, dx, dy) {
      if (!img || !img.data) return;
      var d = img.data;
      var n = d.length;
      // 分片拼接（避免超大数据单次 += 触发大字符串重分配；测试用小图，正常路径即可）。
      var chunks = [];
      for (var i = 0; i < n; i++) {
        chunks.push((i ? ',' : '') + d[i]);
      }
      __zw_canvas_op(h, 'putImageData', String(dx | 0), String(dy | 0),
        String(img.width | 0), String(img.height | 0), chunks.join(''));
    };
    // drawImage（R2799，canvas slice 5）：源 canvas → 本 ctx。3 spec 重载（arg 数 3/5/9）：
    //   drawImage(image, dx, dy) / drawImage(image, dx, dy, dw, dh) /
    //   drawImage(image, sx, sy, sw, sh, dx, dy, dw, dh)。
    // **源限 canvas 元素**（canvas-to-canvas）：经源 canvas 既有 getImageData 取全 RGBA wire 串作源传 host；
    // R3309：ImageBitmap 源（持 _zwBitmapWire）直接用其 wire 串，跳过 canvas 源 getImageData。
    // HTMLImageElement/`<img>` decode defer。host draw_image* 真栅格（source-over alpha 混合）。
    ctx.drawImage = function (image) {
      if (typeof __zw_canvas_op !== 'function') return;
      var a = arguments;
      // ImageBitmap 源（R3309 createImageBitmap 产物）：直接用其 wire 串调 drawImage host op。
      if (image && image._zwBitmapWire && !image._closed) {
        var bmw = image.width | 0;
        var bmh = image.height | 0;
        if (bmw <= 0 || bmh <= 0) return;
        if (a.length === 3) {
          __zw_canvas_op(h, 'drawImage', image._zwBitmapWire, String(a[1]), String(a[2]));
        } else if (a.length === 5) {
          __zw_canvas_op(h, 'drawImageScaled', image._zwBitmapWire,
            String(a[1]), String(a[2]), String(a[3]), String(a[4]));
        } else if (a.length === 9) {
          __zw_canvas_op(h, 'drawImageSliced', image._zwBitmapWire,
            String(a[1]), String(a[2]), String(a[3]), String(a[4]),
            String(a[5]), String(a[6]), String(a[7]), String(a[8]));
        }
        return;
      }
      // R34xx：G5 —— HTMLImageElement 源（naturalWidth>0 = 已加载）→ __zw_get_image_wire
      //（host 查 image_cache 编码 ImageData wire）。img 元素无 getContext。
      if (image && typeof image.getContext !== 'function' && image.naturalWidth > 0 && typeof __zw_get_image_wire === 'function') {
        var imgSrc = (image.getAttribute ? String(image.getAttribute('src') || '') : '') || String(image.src || '');
        var iwire = String(__zw_get_image_wire(imgSrc));
        if (iwire) {
          if (a.length === 3) {
            __zw_canvas_op(h, 'drawImage', iwire, String(a[1]), String(a[2]));
          } else if (a.length === 5) {
            __zw_canvas_op(h, 'drawImageScaled', iwire, String(a[1]), String(a[2]), String(a[3]), String(a[4]));
          } else if (a.length === 9) {
            __zw_canvas_op(h, 'drawImageSliced', iwire,
              String(a[1]), String(a[2]), String(a[3]), String(a[4]),
              String(a[5]), String(a[6]), String(a[7]), String(a[8]));
          }
        }
        return;
      }
      // 源须为 canvas 元素（有 _ctx._handle + width/height）。未 getContext 则惰性建。
      if (!image || typeof image.getContext !== 'function') return;
      if (!image._ctx) image.getContext('2d');
      if (!image._ctx) return;
      var srcHandle = image._ctx._handle;
      var sw = image.width | 0;
      var sh = image.height | 0;
      if (sw <= 0 || sh <= 0) return;
      var wire = String(__zw_canvas_op(srcHandle, 'getImageData', '0', '0', String(sw), String(sh)));
      if (a.length === 3) {
        __zw_canvas_op(h, 'drawImage', wire, String(a[1]), String(a[2]));
      } else if (a.length === 5) {
        __zw_canvas_op(h, 'drawImageScaled', wire,
          String(a[1]), String(a[2]), String(a[3]), String(a[4]));
      } else if (a.length === 9) {
        __zw_canvas_op(h, 'drawImageSliced', wire,
          String(a[1]), String(a[2]), String(a[3]), String(a[4]),
          String(a[5]), String(a[6]), String(a[7]), String(a[8]));
      }
    };
    ctx.getImageData = function (x, y, w, hh) {
      if (typeof __zw_canvas_op !== 'function') return null;
      // R34xx：x/y/w/h 经 Math.trunc 归一（spec：与 createImageData 同一 WebIDL long 截断语义，
      // 上游 2d.imageData.create2.round 断言两者一致）。
      var r = String(__zw_canvas_op(h, 'getImageData',
        String(Math.trunc(+x)), String(Math.trunc(+y)), String(Math.trunc(+w)), String(Math.trunc(+hh))));
      if (!r) return null;
      var parts = r.split(';');
      var dims = parts[0].split(':');
      var nums = parts[1] ? parts[1].split(',') : [];
      var arr = new Uint8ClampedArray(nums.length);
      for (var i = 0; i < nums.length; i++) arr[i] = +nums[i];
      return { width: +dims[0], height: +dims[1], data: arr };
    };
    return ctx;
  }

  // `|` 分隔的选择器串 → 元素 proxy 数组（空串/无回调 → []）。供 children 等导航 API。
  function _splitSelectors(joined) {
    if (!joined) return [];
    return joined.split('|').filter(Boolean).map(_wrapSelector);
  }

  // 节点级遍历：把 __zw_child_nodes/__zw_sibling_nodes 返的 JSON 条目（{k:'E'|'T'|'C',...}）
  // 转 proxy/对象。元素 → _wrapSelector；文本/注释 → 纯对象（nodeType 3/8，纯读快照非 live，
  // parentNode=parentProxy）。文本节点无 selector，故用静态对象（nodeValue/textContent/data 只读）。
  function _wrapNodeEntry(entry, parentProxy) {
    if (!entry) return null;
    if (entry.k === 'E') return _wrapSelector(entry.s);
    var isComment = entry.k === 'C';
    var text = entry.v != null ? entry.v : '';
    return {
      nodeType: isComment ? 8 : 3,
      nodeName: isComment ? '#comment' : '#text',
      nodeValue: text,
      textContent: text,
      data: text,
      length: text.length,
      parentNode: parentProxy,
      parentElement: parentProxy,
      previousSibling: null,
      nextSibling: null,
      __zwIsText: true,
    };
  }

  // `el.childNodes`（含文本/注释）：解析 __zw_child_nodes JSON 数组 → 节点数组（快照，非 live）。
  function _childNodeList(sel, handle) {
    if (!sel || typeof __zw_child_nodes !== 'function') return [];
    try {
      var arr = JSON.parse(__zw_child_nodes(sel) || '[]');
      var parent = handle ? _wrapHandle(handle) : _wrapSelector(sel);
      return arr.map(function(e) { return _wrapNodeEntry(e, parent); });
    } catch (_e) { return []; }
  }

  // R3033：把元素数组包成 spec 集合——补 `.item(i)`（HTMLCollection/NodeList 共有），`htmlCollection=true`
  // 时再补 `.namedItem(name)`（id 或 name 首匹配，HTMLCollection 专有）。既有数组 length/indexed/forEach/
  // entries-keys-values/Symbol.iterator 天然具备（不破坏）；item/namedItem 用 defineProperty 设为非 enumerable，
  // 不污染 `for...in`（real browser 这些方法在原型链上，for...in 不可见）。`getElementsByTagName`/
  // `getElementsByClassName` 返 HTMLCollection（item + namedItem）；`querySelectorAll`/`getElementsByName`
  // 返 NodeList（仅 item）。live 语义保持静态快照近似（documented，headless 模型一致）。
  function _zwMakeCollection(arr, htmlCollection) {
    var a = arr || [];
    Object.defineProperty(a, 'item', {
      value: function (i) { i = i | 0; return i >= 0 && i < a.length ? a[i] : null; },
      enumerable: false, configurable: true, writable: true,
    });
    if (htmlCollection) {
      Object.defineProperty(a, 'namedItem', {
        value: function (name) {
          var n = String(name);
          for (var k = 0; k < a.length; k++) {
            var el = a[k];
            if (!el) continue;
            // id/name 反射：优先 getAttribute（可靠），回落 .id/.name 反射字段。
            var id = (typeof el.getAttribute === 'function') ? (el.getAttribute('id') || '') : (el.id || '');
            if (id === n) return el;
            var nm = (typeof el.getAttribute === 'function') ? (el.getAttribute('name') || '') : (el.name || '');
            if (nm === n) return el;
          }
          return null;
        },
        enumerable: false, configurable: true, writable: true,
      });
    }
    return a;
  }

  // `prepend`/`before`/`after` 共用：variadic 节点/字符串按 position 经 insertAdjacent*
  // 回调插入。仅 sel-based（已挂载）目标；handle-only（detached）无操作（同 insertAdjacent 家族）。
  // `reverseOrder`：afterbegin（prepend）/afterend（after）需反序插入以保持「参数序 == DOM 序」
  //（每插一项后参考子/兄弟前移）；beforebegin（before）正序即可（参考 = target 固定）。
  function _insertAdjacentVariadic(sel, position, args, reverseOrder) {
    if (!sel || typeof __zw_insert_adjacent_element !== 'function') return;
    var items = [];
    for (var i = 0; i < args.length; i++) {
      var a = args[i];
      if (a == null) continue;
      items.push(a);
    }
    if (reverseOrder) items.reverse();
    // R2994：目标 sel 已挂载（本函数要求 sel-based）→ 其自身及父均连入 document，故新插入的元素子/兄弟
    // 随之连入。收集插入的元素项，事后按 connected 传播（text 字符串项跳过——非 custom element）。
    var ceInserted = [];
    for (var k = 0; k < items.length; k++) {
      var item = items[k];
      try {
        if (typeof item === 'object' && item.__zwHandle) {
          __zw_insert_adjacent_element(sel, position, item.__zwHandle);
          ceInserted.push(item);
        } else {
          __zw_insert_adjacent_text(sel, position, String(item));
        }
      } catch (_e) {}
    }
    for (var ci = 0; ci < ceInserted.length; ci++) _ceApplyConn(ceInserted[ci], true);
  }

  // append/replaceChildren 共用：variadic 节点/字符串追加到 this 末尾（DocumentFragment flatten）。
  // 返 added 列表（供 MO childList notify）。节点经 handle/selector append_child；字符串建 text 节点 append。
  function _appendVariadic(sel, handle, args) {
    var added = [];
    for (var i = 0; i < args.length; i++) {
      var item = args[i];
      if (item == null) continue;
      if (typeof item === 'object' && item.__zwHandle) {
        // DocumentFragment：flatten 子节点到 this。
        if (_fragmentHandles[item.__zwHandle] && typeof __zw_append_fragment_children === 'function') {
          if (handle) __zw_append_fragment_children_handle(handle, item.__zwHandle);
          else __zw_append_fragment_children(sel, item.__zwHandle);
        } else if (handle) {
          __zw_append_child_handle(handle, item.__zwHandle);
        } else {
          __zw_append_child(sel, item.__zwHandle);
        }
        added.push(item);
      } else {
        var tn = __zw_create_text(String(item));
        if (handle) __zw_append_child_handle(handle, tn);
        else __zw_append_child(sel, tn);
        added.push({ __zwHandle: tn, __zwSelector: '' });
      }
    }
    return added;
  }

  // 元素的布局 rect（{x,y,w,h}），经 `__zw_getBoundingClientRect`（与 getBoundingClientRect 同源）。
  // 无回调/未命中/handle 未映射 → null（调用方返 0）。rect 反映上次 render（stale-but-non-zero）。
  function _layoutRect(sel, handle) {
    var id = sel || handle;
    if (id && typeof __zw_getBoundingClientRect === 'function') {
      try {
        var s = __zw_getBoundingClientRect(id);
        if (s && s.indexOf(',') >= 0) {
          var p = s.split(',');
          return { x: +p[0], y: +p[1], w: +p[2], h: +p[3] };
        }
      } catch (_e) {}
    }
    return null;
  }

  // getBoundingClientRect/getClientRects 共用（R2828）：从 `__zw_getBoundingClientRect(id)` 解析
  // "x,y,w,h" → 完整 DOMRect（x/y/top/left/right/bottom/width/height + toJSON）。id = selector 或 handle。
  // 未注册 / 未命中 / 无 layout（handle-only detached）→ null（getBoundingClientRect 落零 rect，getClientRects 落 []）。
  function _domRectFromId(id) {
    if (id && typeof __zw_getBoundingClientRect === 'function') {
      try {
        var s = __zw_getBoundingClientRect(id);
        if (s && s.indexOf(',') >= 0) {
          var p = s.split(',');
          var x = +p[0], y = +p[1], w = +p[2], h = +p[3];
          // R3319：经 _makeDomRect 返 DOMRect（instanceof DOMRect/DOMRectReadOnly 成立）。
          return _makeDomRect(x, y, w, h);
        }
      } catch (_e) {}
    }
    return null;
  }

  // form.elements 表单控件集合（R2829）：优先由 host 按 form owner 收集 listed controls，覆盖外部
  // `form=id` 与后代显式改归属；无 host 时回退子树遍历。
  // https://html.spec.whatwg.org/multipage/forms.html#category-listed
  var _formControlTags = { INPUT: 1, BUTTON: 1, FIELDSET: 1, OBJECT: 1, OUTPUT: 1, SELECT: 1, TEXTAREA: 1 };
  function _formControls(sel) {
    var controls = [];
    if (!sel) return controls;
    if (typeof __zw_form_controls === 'function') {
      try {
        var listed = __zw_form_controls(sel);
        if (listed) return listed.split('|').filter(Boolean).map(_wrapSelector);
      } catch (_e) {}
    }
    // 递归下降：childNodes 遍历子树（element 子递归，text/comment 跳过），tag 命中收集。
    function walk(parentProxy) {
      var kids = (parentProxy && parentProxy.childNodes) || [];
      for (var i = 0; i < kids.length; i++) {
        var k = kids[i];
        if (k && k.nodeType === 1) {
          if (_formControlTags[k.tagName]) controls.push(k);
          walk(k);
        }
      }
    }
    try { walk(_wrapSelector(sel)); } catch (_e) {}
    return controls;
  }

  // `el.getElementsByTagName('*')`（R2980）——host `__zw_query_all_sub` 不支持通用选择器 `*`
  //（见 _formControls 注），故 `*` 经客户端 childNodes 递归下降收全部元素后代（tree order）。
  // 单 tag / 类名仍走 host 路径（更快，单次 DOM 解析）。仅供 sel-based 元素用；handle-based
  //（createElement）`*` 由 `_handleQueryAll`（R2928，原生支持 `*`）覆盖。
  function _descendantElements(sel) {
    var out = [];
    if (!sel) return out;
    function walk(parentProxy) {
      var kids = (parentProxy && parentProxy.childNodes) || [];
      for (var i = 0; i < kids.length; i++) {
        var k = kids[i];
        if (k && k.nodeType === 1) {
          out.push(k);
          walk(k);
        }
      }
    }
    try { walk(_wrapSelector(sel)); } catch (_e) {}
    return out;
  }

  // dataset 键转换：camelCase ↔ data-kebab-case（fooBar ↔ data-foo-bar）。
  function _camelToKebab(s) {
    return s.replace(/[A-Z]/g, function(m) { return '-' + m.toLowerCase(); });
  }

  // ARIA IDL 属性名 → content 属性名（element.ariaXxx ↔ aria-xxx）。
  // **不同于 _camelToKebab**：ariaLabelledBy → aria-labelledby（单 hyphen，非 aria-labelled-by）。
  // 规则：aria + 大写首字母 + 余 → aria- + 全小写(余)。非 aria 前缀 / 首字母非大写 → null。
  function _ariaAttrName(prop) {
    if (typeof prop !== 'string' || prop.length < 5 || prop.slice(0, 4) !== 'aria') return null;
    var rest = prop.slice(4);
    var head = rest.charAt(0);
    if (head < 'A' || head > 'Z') return null;
    return 'aria-' + rest.toLowerCase();
  }
  function _kebabToCamel(s) {
    return s.replace(/-([a-z])/g, function(_, c) { return c.toUpperCase(); });
  }

  // `el.dataset`——data-* 属性的 camelCase 键对象。get/set/has/delete/枚举（ownKeys）。
  // 注：mutate（set/delete）记 mutation，apply 在脚本末尾——同脚本内即读见旧值（stale，
  // 同 setAttribute 既有模式）；枚举读 dom_html 当前属性名。
  function _datasetProxy(sel, handle) {
    var attrOf = function(key) { return 'data-' + _camelToKebab(String(key)); };
    var readAttr = function(name) {
      // R3002：sel 用 latest-wins（`__zw_get_attr_lw`）反映同批 setAttribute/dataset 设删；handle 用
      // `__zw_get_attr_handle`（latest-wins from mutations）。data-* 为纯反射属性（无 dirty 态），无污染顾虑。
      if (handle) return __zw_get_attr_handle(handle, name);
      if (typeof __zw_get_attr_lw === 'function') return __zw_get_attr_lw(sel, name);
      return __zw_get_attr(sel, name);
    };
    var hasAttrFn = function(name) {
      try {
        // R3195：handle 经 `__zw_has_attr_handle`（latest-wins from mutations）判存在性——旧恒返 false
        //（R3002 时无 has-attr-handle 回调遗留），致 handle 元素 `el.dataset.foo=` 后 `el.dataset.foo`
        // 恒 undefined（get trap 经 hasAttrFn 短路）。
        if (handle) {
          if (typeof __zw_has_attr_handle === 'function') return __zw_has_attr_handle(handle, name) === '1';
          return false;
        }
        // R3002：sel 用 latest-wins 反映同批 SetAttr/RemoveAttr（旧 `__zw_has_attr` 纯快照 stale）。
        if (typeof __zw_has_attr_lw === 'function') return __zw_has_attr_lw(sel, name) === '1';
        return __zw_has_attr(sel, name) === '1';
      } catch (_e) { return false; }
    };
    var dataKeys = function() {
      // data-* → camelCase 键。sel 用 `__zw_attr_names`（latest-wins），handle 用 `__zw_attr_names_handle`
      //（R3196：闭合 R3195 限制①——旧 handle 路径无 attr-names 变体恒返 []）。无对应回调 → []。
      try {
        var names = handle
          ? (typeof __zw_attr_names_handle === 'function' ? __zw_attr_names_handle(handle) : '')
          : (typeof __zw_attr_names === 'function' ? __zw_attr_names(sel) : '');
        if (!names) return [];
        return names.split('|').filter(function(n) { return n.indexOf('data-') === 0; })
                     .map(function(n) { return _kebabToCamel(n.slice(5)); });
      } catch (_e) { return []; }
    };
    return new Proxy({}, {
      get: function(_t, key) {
        if (typeof key !== 'string') return undefined;
        if (key === 'then') return undefined; // 防 Promise 化误判
        var name = attrOf(key);
        // 缺失属性 → undefined（__zw_get_attr 对缺失返空串，须用 has_attr 区分）。
        if (!hasAttrFn(name)) return undefined;
        var v = readAttr(name);
        return v == null ? '' : v;
      },
      set: function(_t, key, value) {
        if (typeof key !== 'string') return false;
        var name = attrOf(key);
        if (handle) __zw_set_attr_handle(handle, name, String(value));
        else __zw_set_attr(sel, name, String(value));
        _mo_notify(sel, handle, { type: 'attributes', attributeName: name });
        return true;
      },
      has: function(_t, key) {
        return typeof key === 'string' && hasAttrFn(attrOf(key));
      },
      deleteProperty: function(_t, key) {
        if (typeof key !== 'string') return false;
        var name = attrOf(key);
        // R3195：handle 优先 `__zw_remove_attr_handle`（真移除，与 removeAttribute 一致）——旧用 set-empty
        // 残留 `data-x=""` 致 hasAttr 仍 true（get 返 '' 而非 undefined）。无回调 → fallback set-empty。
        if (handle && typeof __zw_remove_attr_handle === 'function') __zw_remove_attr_handle(handle, name);
        else if (handle) __zw_set_attr_handle(handle, name, '');
        else if (typeof __zw_remove_attr === 'function') __zw_remove_attr(sel, name);
        else __zw_set_attr(sel, name, '');
        _mo_notify(sel, handle, { type: 'attributes', attributeName: name });
        return true;
      },
      ownKeys: function() { return dataKeys(); },
      getOwnPropertyDescriptor: function(_t, key) {
        if (typeof key !== 'string' || !hasAttrFn(attrOf(key))) return undefined;
        return { configurable: true, enumerable: true, value: readAttr(attrOf(key)), writable: true };
      }
    });
  }

  // Event/CustomEvent/KeyboardEvent——DOM 事件构造器（R2779 spec-completeness）。_makeEvent 造数据
  // 对象（含 spec 字段 composed/eventPhase/isTrusted/timeStamp/defaultPrevented），构造器置 [[Prototype]]
  // 使 instanceof 成立（chromium 一致：new Event() instanceof Event、new CustomEvent() instanceof Event）。
  // dispatch 读 _-prefixed 私字段（_defaultPrevented 等，勿改名）；公开 defaultPrevented 经 preventDefault
  // 镜像同步。initEvent legacy API 在 Event.prototype。
  globalThis.Event = function Event(type, options) {
    var ev = _makeEvent(type, options);
    Object.setPrototypeOf(ev, globalThis.Event.prototype);
    return ev;
  };
  if (typeof globalThis.Event.prototype.initEvent !== 'function') {
    globalThis.Event.prototype.initEvent = function (type, bubbles, cancelable) {
      this.type = type;
      this.bubbles = !!bubbles;
      this.cancelable = !!cancelable;
      this.defaultPrevented = false;
      this._defaultPrevented = false;
      // js-dom M4 R26：spec `concept-event-initialize` 重置 dispatch flags——initEvent 把 cancelBubble +
      // stop propagation flag 归零（WPT Event-cancelBubble "initEvent must set cancelBubble to false"）。
      this.cancelBubble = false;
      this._propagationStopped = false;
      this._immediateStopped = false;
    };
  }
  // R23：`Event` eventPhase 常量（spec `Event` 接口的静态 + 原型属性，WPT Event-constants.html testConstants）。
  // 挂在**接口对象**（Event 构造器，静态常量）+ **Event.prototype**（实例经原型链继承）。spec DOM：
  // NONE=0、CAPTURING_PHASE=1、AT_TARGET=2、BUBBLING_PHASE=3。createEvent('Event')/createEvent('CustomEvent')
  // 实例经 setPrototypeOf(Event.prototype) / CustomEvent.prototype(=Object.create(Event.prototype)) 继承获得。
  // 定义属性 enumerable:false（与 DOM 原型方法不可枚举一致，R10——避免 for-in 污染 expando）。guard 幂等。
  (function () {
    var consts = { NONE: 0, CAPTURING_PHASE: 1, AT_TARGET: 2, BUBBLING_PHASE: 3 };
    for (var k in consts) {
      if (!(k in globalThis.Event)) {
        Object.defineProperty(globalThis.Event, k, { value: consts[k], enumerable: false });
      }
      if (!(k in globalThis.Event.prototype)) {
        Object.defineProperty(globalThis.Event.prototype, k, { value: consts[k], enumerable: false });
      }
    }
  })();

  globalThis.CustomEvent = function CustomEvent(type, options) {
    var ev = _makeEvent(type, options);
    Object.setPrototypeOf(ev, globalThis.CustomEvent.prototype);
    return ev;
  };
  globalThis.CustomEvent.prototype = Object.create(globalThis.Event.prototype);
  globalThis.CustomEvent.prototype.constructor = globalThis.CustomEvent;
  // initCustomEvent——legacy 合成事件初始化（与 createEvent('CustomEvent') + initEvent 配对，spec）。
  // 镜像 initEvent 设 type/bubbles/cancelable + 设 detail。guard 幂等（不覆盖既有定义）。
  if (typeof globalThis.CustomEvent.prototype.initCustomEvent !== 'function') {
    globalThis.CustomEvent.prototype.initCustomEvent = function (type, bubbles, cancelable, detail) {
      this.type = type;
      this.bubbles = !!bubbles;
      this.cancelable = !!cancelable;
      this.detail = detail;
      this.defaultPrevented = false;
      this._defaultPrevented = false;
    };
  }

  // js-dom M4 R24：KeyboardEvent 改用 `_defineEventSubclass`（spec KeyboardEvent extends UIEvent，
  // 含 EventModifierInit + key/code/location/repeat/isComposing/charCode/keyCode/which）。旧实现只设
  // key/code + extends Event（非 UIEvent），WPT Event-subclasses-constructors KeyboardEvent 用例缺 view/detail
  //（UIEvent 父链）+ 修饰键 + location/repeat 等。改经工厂后自动继承 UIEvent 父链 + 全属性补全。
  // **须在 UIEvent 定义之后**（_defineEventSubclass('KeyboardEvent','UIEvent',...) 读 globalThis.UIEvent）。
  // 注：此块下移到 UIEvent 定义之后（见 _defineEventSubclass 调用区）。

  // Event 子类簇（R2811）——UIEvent / MouseEvent / FocusEvent / WheelEvent / PointerEvent / InputEvent。
  // 现代输入事件表面：feature-detection（`'PointerEvent' in window`）+ `new MouseEvent('click',{clientX,...})`
  // 合成派发（测试 / 库 / 事件总线高频）。统一经 [`_defineEventSubclass`] 工厂建（复用 `_makeEvent` + 原型链
  // extends parent）。**已知限制**：① 仅构造期填字段（无真事件循环派发——同 Event/KeyboardEvent 既有简化）；
  // ② getModifierState 仅跟踪 Alt/Control/Meta/Shift（CapsLock/NumLock 等未跟踪→false）；③ pageX/pageY
  // 存值非计算（spec 计算自 clientX+scroll，本沙箱无滚动→取存值或 0）。
  // js-dom M4 R24：子类 props 注册表——构造器须沿**父链**收集全部 props 设值（MouseEvent extends UIEvent
  // 实例须有 UIEvent 的 view/detail；旧实现只设子类自身 props → MouseEvent 实例缺 view/detail，WPT
  // Event-subclasses-constructors assert_props 递归检查父链 fail）。键=子类名，值=[ownProps, parentName]。
  var _eventSubclassProps = {};
  function _defineEventSubclass(name, parentName, props) {
    if (globalThis[name]) return globalThis[name];
    var Parent = globalThis[parentName] || globalThis.Event;
    var Ctor = function (type, options) {
      var ev = _makeEvent(type, options);
      Object.setPrototypeOf(ev, Ctor.prototype);
      var o = (options == null || typeof options !== 'object') ? {} : options;
      // R24：沿父链收集 props（自身 + 所有祖先），子类 props 先于父类（同属性子类覆盖父类，spec 一致）。
      // _makeEvent 已设 Event 基础属性（type/bubbles/cancelable/...），此处补子类 + 父链专属字段。
      var chain = [];
      var cur = name;
      var guard = 0;
      while (cur && _eventSubclassProps[cur] && guard++ < 32) {
        var entry = _eventSubclassProps[cur];
        chain = chain.concat(entry[0]);
        cur = entry[1];
      }
      for (var i = 0; i < chain.length; i++) {
        var p = chain[i];
        // != null：null/undefined 用默认（spec init dict 缺省 → 默认值；显式 null → 默认，spec LegacyNull 不适用事件 init）。
        ev[p[0]] = o[p[1]] != null ? o[p[1]] : p[2];
      }
      return ev;
    };
    Ctor.prototype = Object.create(Parent.prototype);
    Ctor.prototype.constructor = Ctor;
    globalThis[name] = Ctor;
    _eventSubclassProps[name] = [props, parentName];
    return Ctor;
  }
  // UIEvent（Event 子类）：view（默认 null）/ detail（默认 0）。
  _defineEventSubclass('UIEvent', 'Event', [
    ['view', 'view', null],
    ['detail', 'detail', 0],
  ]);
  // MouseEvent（UIEvent 子类）：坐标 / 修饰键 / button / buttons / relatedTarget。
  var MouseEventCtor = _defineEventSubclass('MouseEvent', 'UIEvent', [
    ['screenX', 'screenX', 0], ['screenY', 'screenY', 0],
    ['clientX', 'clientX', 0], ['clientY', 'clientY', 0],
    ['pageX', 'pageX', 0], ['pageY', 'pageY', 0],
    ['offsetX', 'offsetX', 0], ['offsetY', 'offsetY', 0],
    ['ctrlKey', 'ctrlKey', false], ['shiftKey', 'shiftKey', false],
    ['altKey', 'altKey', false], ['metaKey', 'metaKey', false],
    ['button', 'button', 0], ['buttons', 'buttons', 0],
    ['relatedTarget', 'relatedTarget', null], ['region', 'region', null],
  ]);
  // getModifierState——修饰键状态查询（PointerEvent/WheelEvent 经原型链继承）。仅 4 个 tracked 修饰键。
  MouseEventCtor.prototype.getModifierState = function (key) {
    var k = String(key);
    if (k === 'Alt') return !!this.altKey;
    if (k === 'Control') return !!this.ctrlKey;
    if (k === 'Meta') return !!this.metaKey;
    if (k === 'Shift') return !!this.shiftKey;
    return false;
  };
  // FocusEvent（UIEvent 子类）：relatedTarget。
  _defineEventSubclass('FocusEvent', 'UIEvent', [
    ['relatedTarget', 'relatedTarget', null],
  ]);
  // js-dom M4 R24：KeyboardEvent（spec extends UIEvent）——EventModifierInit（修饰键）+ key/code/location/
  // repeat/isComposing/charCode/keyCode/which。改用工厂（旧独立实现只设 key/code + extends Event，缺父链 +
  // 全属性）。WPT Event-subclasses-constructors KeyboardEvent 用例（默认 + 设定值）。
  var KeyboardEventCtor = _defineEventSubclass('KeyboardEvent', 'UIEvent', [
    ['ctrlKey', 'ctrlKey', false], ['shiftKey', 'shiftKey', false],
    ['altKey', 'altKey', false], ['metaKey', 'metaKey', false],
    ['key', 'key', ''], ['code', 'code', ''],
    ['location', 'location', 0], ['repeat', 'repeat', false],
    ['isComposing', 'isComposing', false],
    ['charCode', 'charCode', 0], ['keyCode', 'keyCode', 0], ['which', 'which', 0],
  ]);
  KeyboardEventCtor.prototype.getModifierState = MouseEventCtor.prototype.getModifierState;
  // WheelEvent（MouseEvent 子类）：delta + deltaMode + DOM_DELTA_* 静态常量。
  var WheelEventCtor = _defineEventSubclass('WheelEvent', 'MouseEvent', [
    ['deltaX', 'deltaX', 0], ['deltaY', 'deltaY', 0], ['deltaZ', 'deltaZ', 0],
    ['deltaMode', 'deltaMode', 0],
  ]);
  WheelEventCtor.DOM_DELTA_PIXEL = 0;
  WheelEventCtor.DOM_DELTA_LINE = 1;
  WheelEventCtor.DOM_DELTA_PAGE = 2;
  // PointerEvent（MouseEvent 子类）：pointer 字段。
  _defineEventSubclass('PointerEvent', 'MouseEvent', [
    ['pointerId', 'pointerId', 0], ['width', 'width', 1], ['height', 'height', 1],
    ['pressure', 'pressure', 0], ['tiltX', 'tiltX', 0], ['tiltY', 'tiltY', 0],
    ['pointerType', 'pointerType', ''], ['isPrimary', 'isPrimary', false],
    ['twist', 'twist', 0], ['tangentialPressure', 'tangentialPressure', 0],
  ]);
  // InputEvent（UIEvent 子类）：data / inputType / isComposing / dataTransfer。
  _defineEventSubclass('InputEvent', 'UIEvent', [
    ['data', 'data', null], ['isComposing', 'isComposing', false],
    ['inputType', 'inputType', ''], ['dataTransfer', 'dataTransfer', null],
  ]);
  // CompositionEvent（UI Events）：IME compositionstart/update/end 的 data。
  _defineEventSubclass('CompositionEvent', 'UIEvent', [
    ['data', 'data', ''], ['locale', 'locale', ''],
  ]);
  // Event 子类簇 #2（R2812）——均 extends Event：HashChangeEvent（SPA hash 路由）/ PopStateEvent（history
  // 路由）/ StorageEvent（跨标签页 storage 同步）/ ProgressEvent（XHR/资源加载进度）/ TransitionEvent·
  // AnimationEvent（CSS 过渡/动画回调）。feature-detection + `new X(type, init)` 合成派发高频。复用
  // [`_defineEventSubclass`]（R2811）。**已知限制**：仅构造期填字段（无真事件派发——同既有简化）。
  _defineEventSubclass('HashChangeEvent', 'Event', [
    ['oldURL', 'oldURL', ''], ['newURL', 'newURL', ''],
  ]);
  _defineEventSubclass('PopStateEvent', 'Event', [
    ['state', 'state', null],
  ]);
  _defineEventSubclass('StorageEvent', 'Event', [
    ['key', 'key', null], ['newValue', 'newValue', null], ['oldValue', 'oldValue', null],
    ['url', 'url', ''], ['storageArea', 'storageArea', null],
  ]);
  _defineEventSubclass('ProgressEvent', 'Event', [
    ['lengthComputable', 'lengthComputable', false], ['loaded', 'loaded', 0], ['total', 'total', 0],
  ]);
  _defineEventSubclass('TransitionEvent', 'Event', [
    ['propertyName', 'propertyName', ''], ['elapsedTime', 'elapsedTime', 0], ['pseudoElement', 'pseudoElement', ''],
  ]);
  _defineEventSubclass('AnimationEvent', 'Event', [
    ['animationName', 'animationName', ''], ['elapsedTime', 'elapsedTime', 0], ['pseudoElement', 'pseudoElement', ''],
  ]);
  // R2931 PageTransitionEvent——pageshow/pagehide 生命周期事件（persisted 标 bfcache 恢复）。pageshow 经
  // _maybeFirePageShow 首次注册派发；pagehide 仅支持构造 + addEventListener（headless 无 unload 不自动派发）。
  _defineEventSubclass('PageTransitionEvent', 'Event', [
    ['persisted', 'persisted', false],
  ]);
  // R2936 ClipboardEvent——copy/cut/paste 剪贴板事件（clipboardData 近似 DataTransfer，headless 无真
  // 剪贴板 → 默认 null）。execCommand('copy'/'cut'/'paste') 派发；oncopy/oncut/onpaste 经 R2932/R2933 on* 路由。
  _defineEventSubclass('ClipboardEvent', 'Event', [
    ['clipboardData', 'clipboardData', null],
  ]);
  // R2937 DragEvent——拖放事件（dragstart/drag/dragend/dragenter/dragover/dragleave/drop），extends MouseEvent
  // + dataTransfer（DataTransfer）。事件类型经 generic addEventListener/on*/dispatchEvent 触发；构造器供
  // 合成派发（库 / 测试 / file-drop handler）。draggable 属性 R2848 已实现。
  _defineEventSubclass('DragEvent', 'MouseEvent', [
    ['dataTransfer', 'dataTransfer', null],
  ]);
  // js-dom M4 R14：createEvent alias 用例（Document-createEvent.https.html aliases 表）需这些 event 子类
  // 构造器存在（createEvent('BeforeUnloadEvent') → Object.getPrototypeOf(ev)===BeforeUnloadEvent.prototype）。
  // headless 无对应真实事件源，构造器仅为 createEvent 合成 + instanceof + 原型链（无特化字段，空 props）。
  _defineEventSubclass('BeforeUnloadEvent', 'Event', []);
  _defineEventSubclass('DeviceMotionEvent', 'Event', []);
  _defineEventSubclass('DeviceOrientationEvent', 'Event', []);
  _defineEventSubclass('TextEvent', 'UIEvent', []);
  _defineEventSubclass('TouchEvent', 'UIEvent', []);

  // R2937 DataTransfer——拖放载荷容器（format→string map + effectAllowed/dropEffect + files/items 只读视图）。
  // headless 无真拖拽源（无 OS 文件 / 无 pointer-drag 几何），但 D&D 库（SortableJS / react-dnd fallback /
  // file-drop handler）经 setData/getData 读写 payload，drop handler 读 dataTransfer.getData('text/plain') 等。
  // 实现：in-JS format map；types = Object.keys(map)；files/items 只读空（无真文件）；setDragImage no-op。
  function DataTransfer() {
    this._dt_data = {};
    this.effectAllowed = 'none';
    this.dropEffect = 'none';
    this._dt_files = [];
  }
  DataTransfer.prototype.setData = function (format, data) {
    this._dt_data[String(format)] = String(data);
  };
  DataTransfer.prototype.getData = function (format) {
    return this._dt_data[String(format)] || '';
  };
  DataTransfer.prototype.clearData = function (format) {
    if (format == null) this._dt_data = {};
    else delete this._dt_data[String(format)];
  };
  DataTransfer.prototype.setDragImage = function (_img, _x, _y) { /* headless 无真拖拽图像，no-op */ };
  Object.defineProperty(DataTransfer.prototype, 'types', {
    get: function () { return Object.keys(this._dt_data); },
  });
  Object.defineProperty(DataTransfer.prototype, 'files', {
    get: function () { return this._dt_files; },
  });
  // R2948 DataTransferItemList——dataTransfer.items 的真实视图（替代 R2937 的空数组占位）。
  // 每次 access 按当前 _dt_data（string items）+ _dt_files（file items）重建（live 语义）；item/add/remove/clear
  // 经 owner DataTransfer 的 setData/clearData/文件 push 操作回写，与 types/files/getData 保持一致。indexed getter
  //（items[0]）+ length + Symbol.iterator 覆盖索引访问 / 扩展 / for-of；item.add(data,type)=string item，
  // item.add(file)=file item（File-like，按 .size 探测）。modern D&D 库（react-dnd / SortableJS items API）经此读写。
  Object.defineProperty(DataTransfer.prototype, 'items', {
    get: function () {
      return new DataTransferItemList(this);
    },
  });
  // R2948 DataTransferItem——拖放项（kind='string'|'file'，type=MIME）。getAsString(cb) 异步回调字符串内容
  //（spec：微任务，headless 简化同步调 cb）；getAsFile() string→null，file→其 File-like 对象。
  function DataTransferItem(kind, type, data, file) {
    this.kind = kind;
    this.type = type || '';
    this._data = data;        // 字符串内容（kind='string'）
    this._file = file || null; // File-like（kind='file'）
  }
  DataTransferItem.prototype.getAsString = function (callback) {
    if (this.kind === 'string' && typeof callback === 'function') {
      try { callback(this._data); } catch (_e) {}
    }
  };
  DataTransferItem.prototype.getAsFile = function () {
    return this.kind === 'file' ? this._file : null;
  };
  function DataTransferItemList(dt) {
    this._dt = dt;
    var list = [];
    var data = dt._dt_data || {};
    for (var k in data) {
      if (Object.prototype.hasOwnProperty.call(data, k)) {
        list.push(new DataTransferItem('string', k, data[k], null));
      }
    }
    var files = dt._dt_files || [];
    for (var i = 0; i < files.length; i++) {
      var f = files[i];
      list.push(new DataTransferItem('file', (f && f.type) || '', null, f));
    }
    this._items = list;
    // indexed getter：复制为编号属性（items[0]..items[n-1]）。
    for (var j = 0; j < list.length; j++) this[j] = list[j];
  }
  Object.defineProperty(DataTransferItemList.prototype, 'length', {
    get: function () { return this._items.length; },
  });
  DataTransferItemList.prototype.item = function (i) {
    return this._items[i] || null;
  };
  DataTransferItemList.prototype.add = function (data, type) {
    // add(file)（File-like，按 .size 探测）→ file item；add(data, type) → string item。
    if (data && typeof data === 'object' && typeof data.size === 'number') {
      this._dt._dt_files.push(data);
      var fItem = new DataTransferItem('file', data.type || '', null, data);
      return fItem;
    }
    var t = String(type || '');
    var s = String(data);
    this._dt.setData(t, s);
    var sItem = new DataTransferItem('string', t, s, null);
    this._items.push(sItem);
    this[this._items.length - 1] = sItem;
    return sItem;
  };
  DataTransferItemList.prototype.remove = function (i) {
    var it = this._items[i];
    if (!it) return;
    if (it.kind === 'string') {
      this._dt.clearData(it.type);
    } else {
      var idx = this._dt._dt_files.indexOf(it._file);
      if (idx >= 0) this._dt._dt_files.splice(idx, 1);
    }
  };
  DataTransferItemList.prototype.clear = function () {
    this._dt.clearData();
    this._dt._dt_files = [];
  };
  if (Symbol && Symbol.iterator) {
    DataTransferItemList.prototype[Symbol.iterator] = function () {
      var arr = this._items;
      var i = 0;
      return {
        next: function () {
          return i < arr.length ? { value: arr[i++], done: false } : { value: undefined, done: true };
        },
      };
    };
  }
  globalThis.DataTransfer = DataTransfer;
  globalThis.DataTransferItem = DataTransferItem;
  // R2940 ErrorEvent——未捕获脚本错误 / 资源加载失败事件（window.onerror / window 'error'）。
  // 字段：message（错误消息）/ filename（源脚本 URL）/ lineno / colno / error（Error 对象，headless 无真
  // Error → null）。Sentry / 错误上报库经 `window.addEventListener('error', e => e.message)` 读字段。
  // 构造期填字段（同 Event 子类既有简化）；host `__zw_report_error` 派发此事件到 window。
  // https://html.spec.whatwg.org/#errorevent
  _defineEventSubclass('ErrorEvent', 'Event', [
    ['message', 'message', ''], ['filename', 'filename', ''],
    ['lineno', 'lineno', 0], ['colno', 'colno', 0], ['error', 'error', null],
  ]);

  // EventTarget——事件目标基类型（独立构造器，R2779）。库常用 `new EventTarget()` / `extends EventTarget`
  // 做事件发射器（pub-sub / 自定义事件总线）。元素 / document / window 经各自 addEventListener 路径；
  // 本构造器提供自包含 listener map（与 DOM 元素事件系统独立，派发事件不冒泡到 DOM，spec 一致）。
  // **已知限制**：仅 target 阶段（EventTarget 无 DOM 父链，无跨节点 capture/bubble；capture listener
  // 在 target 阶段同 fire）；dispatchEvent 返 `!defaultPrevented`（spec 一致）。
  function EventTarget() {
    this._et_listeners = {};
  }
  EventTarget.prototype.addEventListener = function (type, cb, opts) {
    if (typeof cb !== 'function' || typeof type !== 'string') return;
    var capture = opts === true || (opts && opts.capture) ? '|cap' : '';
    var key = type + capture;
    (this._et_listeners[key] = this._et_listeners[key] || []).push(cb);
  };
  EventTarget.prototype.removeEventListener = function (type, cb, opts) {
    if (typeof cb !== 'function' || typeof type !== 'string') return;
    var capture = opts === true || (opts && opts.capture) ? '|cap' : '';
    var arr = this._et_listeners[type + capture];
    if (arr) {
      var i = arr.indexOf(cb);
      if (i >= 0) arr.splice(i, 1);
    }
  };
  EventTarget.prototype.dispatchEvent = function (event) {
    if (event == null || typeof event.type !== 'string') {
      event = _makeEvent(event == null ? '' : String(event && event.type), {});
    }
    var target = this;
    event.target = target;
    event.currentTarget = target;
    var suffixes = ['', '|cap'];
    for (var s = 0; s < suffixes.length; s++) {
      var arr = target._et_listeners[event.type + suffixes[s]];
      if (!arr) continue;
      arr = arr.slice();
      for (var i = 0; i < arr.length; i++) {
        if (event._immediateStopped) break;
        try { arr[i].call(target, event); } catch (_) {}
      }
    }
    return !event._defaultPrevented;
  };
  globalThis.EventTarget = globalThis.EventTarget || EventTarget;

  // R3089：DedicatedWorker——`new Worker(url)`。真 worker 消息往返（闭合 R3080 defer 项「无真 worker 执行」）。
  // script-sandbox 为单上下文（无 sub-context API），真独立沙箱需多嵌入器 host 接线（browser/webview/reftest
  // 各提供 __zw_create_worker，defer）。本切片用 **同沙箱 IIFE 影子执行**（对称 compile_module_script R3083）：
  // data: URL inline worker 脚本经 `new Function` 包一层，影子 `var self/postMessage/onmessage/importScripts`
  // → worker 脚本用 worker-scoped 全局（不污染主全局），bare `onmessage = fn` 设影子局部，执行后同步 handler。
  // main↔worker 消息经 structuredClone + queueMicrotask + MessageEvent 派发（对称 MessagePort R2779）。
  // extends EventTarget（与 MessagePort/BroadcastChannel 同款）——addEventListener('message'/'error') 可用。
  // spec https://html.spec.whatwg.org/multipage/workers.html#dom-worker。
  // **已知限制**：① 仅 data: URL inline worker（非 data: 如 './w.js' headless 无 fetch → 不执行，API 表面仍可用）；
  // ② 同全局执行（worker 顶层级隐式全局赋值泄漏到主全局——罕见；spec worker 独立全局，headless 简化）；
  // ③ structuredClone 克隆（非可克隆类型 defer）；④ importScripts no-op（无 fetch）；
  // ⑤ worker 顶层级 postMessage（main 未注册 onmessage 前派发）被丢弃（spec 队列，headless 简化）。
  // 提取 data: URL 脚本 payload（text/javascript / application/javascript，URL-decode 或 base64）。
  function _zwDecodeWorkerScript(url) {
    var s = String(url);
    if (s.indexOf('data:') !== 0) return null;
    var comma = s.indexOf(',');
    if (comma < 0) return null;
    var meta = s.slice(5, comma);
    var payload = s.slice(comma + 1);
    if (meta.indexOf('base64') >= 0) {
      try { return typeof atob === 'function' ? atob(payload) : null; } catch (_e) { return null; }
    }
    try { return decodeURIComponent(payload); } catch (_e) { return payload; }
  }
  function Worker(url) {
    if (!(this instanceof Worker)) return new Worker(url);
    this._et_listeners = {}; // EventTarget 内部 listener map（构造器未自动调，手动初始化）
    this._terminated = false;
    this._onmessage = null;
    this._onerror = null;
    this._scriptUrl = String(url);
    this._handler = null; // worker 的 onmessage（脚本执行时经 wctx.onmessage setter 注入）
    var main = this; // Worker 实例（worker→main 派发 MessageEvent 到此）
    // worker self 上下文（DedicatedWorkerGlobalScope 近似）。postMessage 经 microtask 派发到主 Worker 实例。
    var wctx = {
      // worker→main：structuredClone + queueMicrotask 派发 'message' 到 Worker 实例（对称 MessagePort R2779）。
      postMessage: function (msg) {
        var data = typeof structuredClone === 'function' ? structuredClone(msg) : msg;
        if (typeof queueMicrotask === 'function') {
          queueMicrotask(function () {
            if (main._terminated) return;
            main.dispatchEvent(new MessageEvent('message', { data: data, origin: '' }));
          });
        }
      },
      importScripts: function () {}, // no-op（headless 无 fetch）
      close: function () { main._terminated = true; },
    };
    // onmessage setter：worker 脚本 `self.onmessage = fn` 或 bare `onmessage = fn`（经 IIFE 影子同步）注入 handler。
    Object.defineProperty(wctx, 'onmessage', {
      configurable: true,
      set: function (fn) { if (typeof fn === 'function') main._handler = fn; },
      get: function () { return main._handler; },
    });
    // 执行 worker 脚本（data: URL inline 或外链 fetch）。new Function 包影子声明，bare 赋值设局部，执行后同步 onmessage。
    var scriptSrc = _zwDecodeWorkerScript(url);
    // R3091：外链 worker URL（非 data:）—— 若 host 注册了 __zw_fetch_script（backed by ScriptSourceFetcher），
    // fetch worker 源后同 IIFE 影子执行；未注册 → scriptSrc 仍 null（API 表面可用，worker 不执行，R3080 兼容）。
    if (scriptSrc === null && typeof __zw_fetch_script === 'function') {
      try {
        scriptSrc = __zw_fetch_script((typeof location !== 'undefined' && location.href) || '', url) || null;
      } catch (_e) {
        scriptSrc = null;
      }
    }
    if (scriptSrc) {
      try {
        var body = 'var postMessage=self.postMessage.bind(self);'
          + 'var importScripts=function(){};'
          + 'var onmessage;'
          + scriptSrc
          + '\n;if(typeof onmessage==="function")self.onmessage=onmessage;';
        new Function('self', body).call(null, wctx);
      } catch (e) {
        // worker 脚本抛（语法/运行时）→ microtask 派发 'error' 到 Worker 实例（spec onerror）。
        if (typeof queueMicrotask === 'function') {
          var em = (e && e.message) ? String(e.message) : String(e);
          queueMicrotask(function () {
            if (main._terminated) return;
            main.dispatchEvent(new Event('error', { message: em }));
          });
        }
      }
    }
  }
  Worker.prototype = Object.create(EventTarget.prototype);
  Worker.prototype.constructor = Worker;
  // main→worker：structuredClone + queueMicrotask 调 worker 的 onmessage（handler 在脚本执行时注入）。
  Worker.prototype.postMessage = function (message, _transfer) {
    if (this._terminated) return;
    var handler = this._handler;
    if (typeof handler !== 'function') return; // 无 handler（worker 未设 onmessage）→ 丢弃（spec 队列，headless 简化）
    var data = typeof structuredClone === 'function' ? structuredClone(message) : message;
    var target = this;
    if (typeof queueMicrotask === 'function') {
      queueMicrotask(function () {
        if (target._terminated) return;
        try { handler(new MessageEvent('message', { data: data, origin: '' })); }
        catch (_e) { /* worker handler 抛 → 真浏览器触发 worker error；headless 静默 */ }
      });
    }
  };
  // terminate()——终止 worker：后续 microtask 派发跳过（_terminated 标记）。
  Worker.prototype.terminate = function () {
    this._terminated = true;
  };
  Object.defineProperty(Worker.prototype, 'onmessage', {
    configurable: true,
    enumerable: true,
    get: function () { return this._onmessage || null; },
    set: function (cb) {
      if (this._onmessage) this.removeEventListener('message', this._onmessage);
      if (typeof cb === 'function') {
        this._onmessage = cb;
        this.addEventListener('message', cb);
      } else {
        this._onmessage = null;
      }
    },
  });
  Object.defineProperty(Worker.prototype, 'onerror', {
    configurable: true,
    enumerable: true,
    get: function () { return this._onerror || null; },
    set: function (cb) {
      if (this._onerror) this.removeEventListener('error', this._onerror);
      if (typeof cb === 'function') {
        this._onerror = cb;
        this.addEventListener('error', cb);
      } else {
        this._onerror = null;
      }
    },
  });
  globalThis.Worker = globalThis.Worker || Worker;

  // matchMedia——window.matchMedia(query) 响应式设计 / viewport 查询（modern 站点高频，shim 曾缺失）。
  // 委托 host `__zw_match_media(query, w, h)`（spec-correct via zero_css_parser::media_query）。返
  // MediaQueryList（extends EventTarget R2779）：media/matches + addEventListener('change') + legacy
  // addListener/removeListener。R3255：MQL 注册进 `_zwMqlRegistry`，`_zwFireMqlChanges()` 在 resize 时
  //（`__zw_user_resize` 调，R3254）重评估 matches 翻转的 MQL → 派 'change'（MediaQueryListEvent：media +
  // matches=新值）。addListener 注册有效且**触发**（R3255 闭合原限制）；matches 为查询时刻快照（spec 一致）。
  var _zwMqlRegistry = [];
  function MediaQueryList(media, matches) {
    this._et_listeners = {}; // EventTarget 内部 listener map（EventTarget 构造器未自动调，手动初始化）
    this.media = media;
    this.matches = matches;
  }
  MediaQueryList.prototype = Object.create(EventTarget.prototype);
  MediaQueryList.prototype.constructor = MediaQueryList;
  // legacy 别名（旧 API：addListener/removeListener → change 事件）。
  MediaQueryList.prototype.addListener = function (cb) { this.addEventListener('change', cb); };
  MediaQueryList.prototype.removeListener = function (cb) { this.removeEventListener('change', cb); };
  globalThis.MediaQueryList = globalThis.MediaQueryList || MediaQueryList;
  function matchMedia(query) {
    var q = String(query);
    var matches = false;
    if (typeof __zw_match_media === 'function') {
      var raw = __zw_match_media(q, globalThis.innerWidth || 0, globalThis.innerHeight || 0);
      try { var p = JSON.parse(raw); matches = !!p.matches; } catch (_) {}
    }
    var mql = new MediaQueryList(q, matches);
    _zwMqlRegistry.push(mql); // R3255：注册以便 resize 时重评估派 change（bounded：页面 matchMedia 调用有限）
    return mql;
  }
  globalThis.matchMedia = globalThis.matchMedia || matchMedia;
  // R3255（CSSOM View §media-query-list）：resize 后重评估所有注册 MQL——matches 翻转的派 'change'
  //（MediaQueryListEvent）。仅对有 change listener 的 MQL 派（无 listener 派发无意义）；matches 在派发前
  // 更新为新值（spec：change 事件携带新 matches）。由 `__zw_user_resize`（part01.js，R3254）调用。
  function _zwFireMqlChanges() {
    for (var i = 0; i < _zwMqlRegistry.length; i++) {
      var mql = _zwMqlRegistry[i];
      var ls = mql._et_listeners && mql._et_listeners['change'];
      if (!ls || !ls.length) continue; // 无 change listener，跳过（免无意义重评估/派发）
      var newMatches = mql.matches;
      if (typeof __zw_match_media === 'function') {
        try {
          var raw = __zw_match_media(mql.media, globalThis.innerWidth || 0, globalThis.innerHeight || 0);
          var p = JSON.parse(raw); newMatches = !!p.matches;
        } catch (_) {}
      }
      if (newMatches !== mql.matches) {
        mql.matches = newMatches; // 更新为新值（spec：change 派发时 matches 已是新值）
        var ev = _makeEvent('change');
        ev.media = mql.media; // MediaQueryListEvent.media
        try { mql.dispatchEvent(ev); } catch (_) {}
      }
    }
  }

  // MessageEvent——message 事件（Window.postMessage / MessagePort / BroadcastChannel 派发）。extends
  // Event（R2779），加 data/origin/source/ports。复用 _makeEvent 造数据对象 + 置 [[Prototype]]。
  function MessageEvent(type, options) {
    var ev = _makeEvent(type, options);
    Object.setPrototypeOf(ev, MessageEvent.prototype);
    ev.data = options && options.data !== undefined ? options.data : null;
    ev.origin = (options && options.origin) || '';
    ev.lastEventId = (options && options.lastEventId) || '';
    ev.source = (options && options.source) || null;
    ev.ports = [];
    return ev;
  }
  MessageEvent.prototype = Object.create(Event.prototype);
  MessageEvent.prototype.constructor = MessageEvent;
  globalThis.MessageEvent = globalThis.MessageEvent || MessageEvent;

  // SubmitEvent——submit 事件（form 提交，R2984）。extends Event，加 `submitter`（触发提交的按钮 proxy / null）。
  // host submit 派发经 __zw_dispatch_event(form_sel, 'submit', {submitter: btn_sel})；submitter 缺省 null
  //（Enter 隐式提交）。表单多 submit 按钮场景（"保存"/"删除"同 form）读 event.submitter 判激活按钮高频。
  function SubmitEvent(type, options) {
    var ev = _makeEvent(type, options);
    Object.setPrototypeOf(ev, SubmitEvent.prototype);
    ev.submitter = (options && options.submitter) || null;
    return ev;
  }
  SubmitEvent.prototype = Object.create(Event.prototype);
  SubmitEvent.prototype.constructor = SubmitEvent;
  globalThis.SubmitEvent = globalThis.SubmitEvent || SubmitEvent;

  // MessagePort——消息端口（MessageChannel 双端口之一，部分库经此做结构化通信）。extends EventTarget
  //（R2779）。postMessage 经 structuredClone（R2773）深拷贝消息 + queueMicrotask（R2774）**异步**派发
  // 'message' 事件到配对端口（spec 为 task；sandbox 经 execute 末 microtask checkpoint 派发，下 execute
  // 可读）。onmessage 属性 setter 内部走 addEventListener('message')。**已知限制**：无 transfer 列表
  //（Transferable 移植，罕见用法）；同执行上下文端口对（跨 worker/进程通信需 host 接线，defer）。
  function MessagePort() {
    this._et_listeners = {}; // EventTarget 内部 listener map（构造器未自动调，手动初始化）
    this._other = null; // 配对端口（MessageChannel 构造时互连）
    this._closed = false;
    this._onmessage = null;
  }
  MessagePort.prototype = Object.create(EventTarget.prototype);
  MessagePort.prototype.constructor = MessagePort;
  MessagePort.prototype.postMessage = function (message) {
    if (this._closed || !this._other) return;
    var data = typeof structuredClone === 'function' ? structuredClone(message) : message;
    var other = this._other;
    if (typeof queueMicrotask === 'function') {
      queueMicrotask(function () {
        if (other._closed) return;
        other.dispatchEvent(new MessageEvent('message', { data: data, origin: '' }));
      });
    }
  };
  MessagePort.prototype.start = function () {}; // 始终 active（polyfill 简化）
  MessagePort.prototype.close = function () {
    this._closed = true;
    if (this._other) this._other._other = null; // 断开配对
    this._other = null;
  };
  Object.defineProperty(MessagePort.prototype, 'onmessage', {
    configurable: true,
    enumerable: true,
    get: function () { return this._onmessage || null; },
    set: function (cb) {
      if (this._onmessage) this.removeEventListener('message', this._onmessage);
      if (typeof cb === 'function') {
        this._onmessage = cb;
        this.addEventListener('message', cb);
      } else {
        this._onmessage = null;
      }
    },
  });
  globalThis.MessagePort = globalThis.MessagePort || MessagePort;

  // MessageChannel——双端口通信通道（port1/port2 互连，postMessage 经异步 message 事件派发到对端）。
  function MessageChannel() {
    if (!(this instanceof MessageChannel)) return new MessageChannel();
    var p1 = new MessagePort();
    var p2 = new MessagePort();
    p1._other = p2;
    p2._other = p1;
    this.port1 = p1;
    this.port2 = p2;
  }
  globalThis.MessageChannel = globalThis.MessageChannel || MessageChannel;

  // BroadcastChannel——同源广播通信（所有同名 channel 实例互收消息，**发送者不收自己**）。extends
  // EventTarget（R2779）。postMessage 经 structuredClone（R2773）深拷贝 + queueMicrotask（R2782 同款异步
  // 派发）到所有同名其他实例。注册表 `_bc_registry`（name → channel 数组）同 JS 上下文共享。**已知限制**：
  // 仅同 JS 上下文广播（跨 worker/进程广播需 host 接线，defer）；sender 不收自己（spec 一致）。
  var _bc_registry = {};
  function BroadcastChannel(name) {
    if (!(this instanceof BroadcastChannel)) return new BroadcastChannel(name);
    this._et_listeners = {}; // EventTarget 内部 listener map（构造器未自动调，手动初始化）
    this._name = String(name);
    this._closed = false;
    this._onmessage = null;
    (_bc_registry[this._name] = _bc_registry[this._name] || []).push(this);
  }
  BroadcastChannel.prototype = Object.create(EventTarget.prototype);
  BroadcastChannel.prototype.constructor = BroadcastChannel;
  Object.defineProperty(BroadcastChannel.prototype, 'name', {
    configurable: true,
    enumerable: true,
    get: function () { return this._name; },
  });
  BroadcastChannel.prototype.postMessage = function (message) {
    if (this._closed) return;
    var data = typeof structuredClone === 'function' ? structuredClone(message) : message;
    var sender = this;
    var name = this._name;
    if (typeof queueMicrotask === 'function') {
      queueMicrotask(function () {
        var peers = _bc_registry[name];
        if (!peers) return;
        peers = peers.slice();
        for (var i = 0; i < peers.length; i++) {
          var p = peers[i];
          if (p === sender || p._closed) continue;
          p.dispatchEvent(new MessageEvent('message', { data: data, origin: '' }));
        }
      });
    }
  };
  BroadcastChannel.prototype.close = function () {
    if (this._closed) return;
    this._closed = true;
    var peers = _bc_registry[this._name];
    if (peers) {
      var i = peers.indexOf(this);
      if (i >= 0) peers.splice(i, 1);
      if (peers.length === 0) delete _bc_registry[this._name];
    }
  };
  Object.defineProperty(BroadcastChannel.prototype, 'onmessage', {
    configurable: true,
    enumerable: true,
    get: function () { return this._onmessage || null; },
    set: function (cb) {
      if (this._onmessage) this.removeEventListener('message', this._onmessage);
      if (typeof cb === 'function') {
        this._onmessage = cb;
        this.addEventListener('message', cb);
      } else {
        this._onmessage = null;
      }
    },
  });
  globalThis.BroadcastChannel = globalThis.BroadcastChannel || BroadcastChannel;

  // EventSource（Server-Sent Events，SSE）——服务器单向推送（通知 / 聊天 / 股票 / live updates 高频）。
  // 经 fetch（R2923）拉 event-stream 全 body 后按 text/event-stream 解析（HTML spec §9.2）：字段
  // data/event/id/retry，空行派发累积事件，`:` 行注释，BOM 去除，CRLF/LF/CR 分行。readyState + onopen/
  // onmessage/onerror + addEventListener。**headless 有限流**：fetch 取整 body 后解析派发全部事件（真浏览器
  // 持续流式逐块派发；本实现 finite-stream 一次派发——headless 加载期足够）。自动重连/Last-Event-ID 记录
  // 但不重连（headless 无长连接）。https://html.spec.whatwg.org/multipage/server-sent-events.html
  function EventSource(url, options) {
    this.url = String(url);
    this.readyState = EventSource.CONNECTING;
    this.withCredentials = !!(options && options.withCredentials);
    this.onopen = null; this.onmessage = null; this.onerror = null;
    this._listeners = {};
    this._lastEventId = '';
    this._closed = false;
    var self = this;
    Promise.resolve().then(function () {
      if (self._closed || typeof fetch !== 'function') throw new Error('no fetch');
      return fetch(self.url, { headers: { 'Accept': 'text/event-stream' } });
    }).then(function (resp) {
      if (!resp || !resp.ok) throw new Error('EventSource fetch failed');
      return resp.text();
    }).then(function (text) {
      if (self._closed) return;
