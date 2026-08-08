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
            // `output.defaultValue = x`（R2846）——更新捕获的初值缓存（不联动 textContent/.value 当前态——
            // spec 仅当未 dirty 时联动，罕见 defer；Chromium 150 oracle：dirty 时设 defaultValue 不改 value）。
            _outputDefault[key] = String(value);
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
          // R3039：布尔 reflected setter（required/readOnly/multiple，_REFLECTED_BOOL）。旧经 generic fallthrough
          // 写 `attr="false"`（present）→ 读返 true（set-false bug）。修正：truthy → set 空（presence）；
          // falsy → removeAttribute（sel 走 `__zw_remove_attr`，handle 走 `__zw_remove_attr_handle`，detached 亦真移除）。
          // 闭合布尔 set→get 全往返（R3038 读 + 本切片 set）。attr 名经 `_reflectedBoolAttr` 映射（readOnly→readonly）。
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
        } else if ((p === 'width' || p === 'height') && (_realTag(sel, handle) === 'IMG' || _realTag(sel, handle) === 'IFRAME')) {
          // reflected unsigned-long 维度 setter（R2851）：parseInt 归一（NaN/负 → 0）→ 缓存数值 + 写 width/height
          // 内容属性（getter 优先读缓存保 sync set→get）。
          var wv = parseInt(value, 10);
          if (isNaN(wv) || wv < 0) wv = 0;
          var wrc = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
          wrc[p] = wv;
          if (handle) __zw_set_attr_handle(handle, p, String(wv));
          else { __zw_set_attr(sel, p, String(wv)); moAttr = p; }
        } else {
          if (handle) __zw_set_attr_handle(handle, p, String(value));
          else __zw_set_attr(sel, p, String(value));
          moAttr = p;
        }
        if (moAttr) _mo_notify(sel, handle, { type: 'attributes', attributeName: moAttr });
        return true;
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
  // 后代组合器（空白） / 逗号列表。不支持（该组静默跳过，不抛）：伪类（`:host`/`:hover`/...）、
  // 子代/相邻/兄弟组合器（>`+`~`）、伪元素——遇之标记 unsupported，逗号列表中其余组仍可匹配；全部
  // unsupported → 无匹配（返 null/[]）。所有 proxy 属性读经 try/catch（host 未注册 / 异常 → 安全回落）。

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
  // 按后代组合器（空白）拆 complex，跳过 `[...]` / 引号内空白；遇 `>`/`+`/`~` → null（不支持）。
  function _splitComplex(text) {
    var parts = [], cur = '', depth = 0, quote = null;
    for (var i = 0; i < text.length; i++) {
      var ch = text[i];
      if (quote) { cur += ch; if (ch === quote) quote = null; continue; }
      if (ch === '"' || ch === "'") { quote = ch; cur += ch; continue; }
      if (ch === '[') { depth++; cur += ch; continue; }
      if (ch === ']') { depth--; cur += ch; continue; }
      if (depth === 0 && (ch === '>' || ch === '+' || ch === '~')) return null;
      if (depth === 0 && /\s/.test(ch)) { if (cur) { parts.push(cur); cur = ''; } continue; }
      cur += ch;
    }
    if (cur) parts.push(cur);
    return parts;
  }
  function _parseComplexOf(text) {
    var parts = _splitComplex(text);
    if (!parts) return null;
    var out = [];
    for (var i = 0; i < parts.length; i++) {
      var c = _parseCompoundOf(parts[i]);
      if (c.unsupported) return null;
      out.push(c);
    }
    return out.length ? out : null;
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
  // 后代组合器匹配：rightmost compound 匹配 proxy；各前置 compound 匹配 ancestors（逆序，最近优先）。
  // 纯后代组合器下「最近匹配祖先」贪心正确：远 B 的祖先必也是近 B 的祖先（传递性）。
  function _matchComplexAgainst(p, compounds, ancestors) {
    var last = compounds.length - 1;
    if (!_matchCompoundOf(p, compounds[last])) return false;
    var ai = ancestors.length - 1;
    for (var ci = last - 1; ci >= 0; ci--) {
      var matched = false;
      while (ai >= 0) {
        if (_matchCompoundOf(ancestors[ai], compounds[ci])) { matched = true; break; }
        ai--;
      }
      if (!matched) return false;
    }
    return true;
  }
  function _matchAnyGroup(p, groups, ancestors) {
    for (var i = 0; i < groups.length; i++) {
      if (_matchComplexAgainst(p, groups[i], ancestors)) return true;
    }
    return false;
  }
  // DFS 收集 rootHandle 子树全部**元素** proxy（document order）+ 各自祖先链（不含 root 自身）。
  function _handleSubtreeNodes(rootHandle) {
    var result = [];
    function visit(handle, ancestors) {
      var kids = _handleChildren[handle];
      if (!kids) return;
      for (var i = 0; i < kids.length; i++) {
        var p = kids[i];
        if (!p) continue;
        if (_hSafe(function () { return p.nodeType; }, 0) !== 1) continue; // 跳过 text/comment
        result.push({ proxy: p, ancestors: ancestors });
        var ph = _hSafe(function () { return p.__zwHandle; }, null);
        if (ph) visit(ph, ancestors.concat([p]));
      }
    }
    visit(rootHandle, []);
    return result;
  }
  function _handleQueryFirst(rootHandle, q) {
    var groups = _parseSelectorListOf(q);
    if (!groups.length) return null;
    var nodes = _handleSubtreeNodes(rootHandle);
    for (var i = 0; i < nodes.length; i++) {
      if (_matchAnyGroup(nodes[i].proxy, groups, nodes[i].ancestors)) return nodes[i].proxy;
    }
    return null;
  }
  function _handleQueryAll(rootHandle, q) {
    var groups = _parseSelectorListOf(q);
    if (!groups.length) return [];
    var nodes = _handleSubtreeNodes(rootHandle);
    var out = [];
    for (var i = 0; i < nodes.length; i++) {
      if (_matchAnyGroup(nodes[i].proxy, groups, nodes[i].ancestors)) out.push(nodes[i].proxy);
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
    return el;
  }
  function _zwMakeCtx2d(h) {
    var ctx = { _handle: h, canvas: null, _fs: '#000000', _ss: '#000000', _lw: 1.0 };
    Object.defineProperty(ctx, 'fillStyle', {
      set: function (v) { this._fs = String(v); __zw_canvas_op(h, 'setFillStyle', String(v)); },
      get: function () { return this._fs; }
    });
    Object.defineProperty(ctx, 'strokeStyle', {
      set: function (v) { this._ss = String(v); __zw_canvas_op(h, 'setStrokeStyle', String(v)); },
      get: function () { return this._ss; }
    });
    Object.defineProperty(ctx, 'lineWidth', {
      set: function (v) { this._lw = +v; __zw_canvas_op(h, 'setLineWidth', String(v)); },
      get: function () { return this._lw; }
    });
    ctx.beginPath = function () { __zw_canvas_op(h, 'beginPath'); };
    ctx.closePath = function () { __zw_canvas_op(h, 'closePath'); };
    ctx.moveTo = function (x, y) { __zw_canvas_op(h, 'moveTo', String(x), String(y)); };
    ctx.lineTo = function (x, y) { __zw_canvas_op(h, 'lineTo', String(x), String(y)); };
    ctx.arc = function (x, y, r, s, e) {
      __zw_canvas_op(h, 'arc', String(x), String(y), String(r), String(s), String(e));
    };
    ctx.fill = function () { __zw_canvas_op(h, 'fill'); };
    ctx.stroke = function () { __zw_canvas_op(h, 'stroke'); };
    ctx.fillRect = function (x, y, w, hh) {
      __zw_canvas_op(h, 'fillRect', String(x), String(y), String(w), String(hh));
    };
    ctx.strokeRect = function (x, y, w, hh) {
      __zw_canvas_op(h, 'strokeRect', String(x), String(y), String(w), String(hh));
    };
    ctx.clearRect = function (x, y, w, hh) {
      __zw_canvas_op(h, 'clearRect', String(x), String(y), String(w), String(hh));
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
    ctx.clip = function () { __zw_canvas_op(h, 'clip'); };
    ctx.save = function () { __zw_canvas_op(h, 'save'); };
    ctx.restore = function () { __zw_canvas_op(h, 'restore'); };
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
    // globalAlpha / lineDash / lineJoin / lineCap：getter+setter（client-side 存值 + push host）。
    ctx._ga = 1.0;
    Object.defineProperty(ctx, 'globalAlpha', {
      set: function (v) { this._ga = +v; __zw_canvas_op(h, 'setGlobalAlpha', String(v)); },
      get: function () { return this._ga; }
    });
    ctx.setLineDash = function (segs) {
      var s = (segs && segs.length != null) ? Array.prototype.join.call(segs, ',') : String(segs);
      __zw_canvas_op(h, 'setLineDash', s);
    };
    ctx._lj = 'miter';
    Object.defineProperty(ctx, 'lineJoin', {
      set: function (v) { this._lj = String(v); __zw_canvas_op(h, 'setLineJoin', String(v)); },
      get: function () { return this._lj; }
    });
    ctx._lc = 'butt';
    Object.defineProperty(ctx, 'lineCap', {
      set: function (v) { this._lc = String(v); __zw_canvas_op(h, 'setLineCap', String(v)); },
      get: function () { return this._lc; }
    });
    // ── slice 4：globalCompositeOperation / shadow / putImageData（R2798）──
    // 客户端镜像串 + push host（同 lineJoin/lineCap 模式）。getter 取客户端镜像，免 host 往返。
    // **已知限制**：composite 仅对 stroke/rect-blit 生效（host composite_pixel），path-based fillRect 不消费。
    ctx._gco = 'source-over';
    Object.defineProperty(ctx, 'globalCompositeOperation', {
      set: function (v) { this._gco = String(v); __zw_canvas_op(h, 'setCompositeOperation', String(v)); },
      get: function () { return this._gco; }
    });
    ctx._sc = 'rgba(0, 0, 0, 0)';
    Object.defineProperty(ctx, 'shadowColor', {
      set: function (v) { this._sc = String(v); __zw_canvas_op(h, 'setShadowColor', String(v)); },
      get: function () { return this._sc; }
    });
    ctx._sb = 0;
    Object.defineProperty(ctx, 'shadowBlur', {
      set: function (v) { this._sb = +v; __zw_canvas_op(h, 'setShadowBlur', String(v)); },
      get: function () { return this._sb; }
    });
    ctx._sox = 0;
    Object.defineProperty(ctx, 'shadowOffsetX', {
      set: function (v) { this._sox = +v; __zw_canvas_op(h, 'setShadowOffsetX', String(v)); },
      get: function () { return this._sox; }
    });
    ctx._soy = 0;
    Object.defineProperty(ctx, 'shadowOffsetY', {
      set: function (v) { this._soy = +v; __zw_canvas_op(h, 'setShadowOffsetY', String(v)); },
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
    // HTMLImageElement/`<img>` decode defer。host draw_image* 真栅格（source-over alpha 混合）。
    ctx.drawImage = function (image) {
      if (typeof __zw_canvas_op !== 'function') return;
      // 源须为 canvas 元素（有 _ctx._handle + width/height）。未 getContext 则惰性建。
      if (!image || typeof image.getContext !== 'function') return;
      if (!image._ctx) image.getContext('2d');
      if (!image._ctx) return;
      var srcHandle = image._ctx._handle;
      var sw = image.width | 0;
      var sh = image.height | 0;
      if (sw <= 0 || sh <= 0) return;
      var wire = String(__zw_canvas_op(srcHandle, 'getImageData', '0', '0', String(sw), String(sh)));
      var a = arguments;
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
      var r = String(__zw_canvas_op(h, 'getImageData', String(x), String(y), String(w), String(hh)));
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
          return {
            x: x, y: y, top: y, left: x, right: x + w, bottom: y + h,
            width: w, height: h, toJSON: function () { return this; },
          };
        }
      } catch (_e) {}
    }
    return null;
  }

  // form.elements 表单控件集合（R2829）：form 后代中 input/button/select/textarea，**tree order**。
  // host `__zw_query_all_sub` 不支持逗号列表 / '*' 通用选择器 → 经 `childNodes` 递归下降遍历子树
  //（tree order 天然）客户端按 tag 过滤。供 form.elements（+ namedItem）+ form.length 共用。
  var _formControlTags = { INPUT: 1, BUTTON: 1, SELECT: 1, TEXTAREA: 1 };
  function _formControls(sel) {
    var controls = [];
    if (!sel) return controls;
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
        if (handle) return false;
        // R3002：sel 用 latest-wins 反映同批 SetAttr/RemoveAttr（旧 `__zw_has_attr` 纯快照 stale）。
        if (typeof __zw_has_attr_lw === 'function') return __zw_has_attr_lw(sel, name) === '1';
        return __zw_has_attr(sel, name) === '1';
      } catch (_e) { return false; }
    };
    var dataKeys = function() {
      // 仅 sel-based 支持枚举（无 attr-names-handle）；data-* → camelCase 键。
      if (handle || typeof __zw_attr_names !== 'function') return [];
      try {
        var names = __zw_attr_names(sel);
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
        if (handle) __zw_set_attr_handle(handle, name, '');
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
    };
  }

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

  globalThis.KeyboardEvent = function KeyboardEvent(type, options) {
    var ev = _makeEvent(type, options);
    Object.setPrototypeOf(ev, globalThis.KeyboardEvent.prototype);
    ev.key = (options && options.key) || '';
    ev.code = (options && (options.code || options.key)) || '';
    return ev;
  };
  globalThis.KeyboardEvent.prototype = Object.create(globalThis.Event.prototype);
  globalThis.KeyboardEvent.prototype.constructor = globalThis.KeyboardEvent;

  // Event 子类簇（R2811）——UIEvent / MouseEvent / FocusEvent / WheelEvent / PointerEvent / InputEvent。
  // 现代输入事件表面：feature-detection（`'PointerEvent' in window`）+ `new MouseEvent('click',{clientX,...})`
  // 合成派发（测试 / 库 / 事件总线高频）。统一经 [`_defineEventSubclass`] 工厂建（复用 `_makeEvent` + 原型链
  // extends parent）。**已知限制**：① 仅构造期填字段（无真事件循环派发——同 Event/KeyboardEvent 既有简化）；
  // ② getModifierState 仅跟踪 Alt/Control/Meta/Shift（CapsLock/NumLock 等未跟踪→false）；③ pageX/pageY
  // 存值非计算（spec 计算自 clientX+scroll，本沙箱无滚动→取存值或 0）。
  function _defineEventSubclass(name, parentName, props) {
    if (globalThis[name]) return globalThis[name];
    var Parent = globalThis[parentName] || globalThis.Event;
    var Ctor = function (type, options) {
      var ev = _makeEvent(type, options);
      Object.setPrototypeOf(ev, Ctor.prototype);
      var o = options || {};
      for (var i = 0; i < props.length; i++) {
        var p = props[i];
        ev[p[0]] = o[p[1]] != null ? o[p[1]] : p[2];
      }
      return ev;
    };
    Ctor.prototype = Object.create(Parent.prototype);
    Ctor.prototype.constructor = Ctor;
    globalThis[name] = Ctor;
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

  // matchMedia——window.matchMedia(query) 响应式设计 / viewport 查询（modern 站点高频，shim 曾缺失）。
  // 委托 host `__zw_match_media(query, w, h)`（spec-correct via zero_css_parser::media_query）。返
  // MediaQueryList（extends EventTarget R2779）：media/matches + addEventListener('change') + legacy
  // addListener/removeListener。**已知限制**：change 事件需 host resize 跟踪派发（当前无，addListener
  // 注册有效但不触发；matches 为查询时刻快照，spec 一致）。
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
    return new MediaQueryList(q, matches);
  }
  globalThis.matchMedia = globalThis.matchMedia || matchMedia;

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
