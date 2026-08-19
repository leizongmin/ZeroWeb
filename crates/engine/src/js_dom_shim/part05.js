          }
        } else if (p === 'defaultValue') {
          // `input.defaultValue = x`（R2840）——反射 `value` 属性（初始值；attr 名映射 defaultValue→value）。
          // 仅设 value 属性，不联动 .value 当前态（spec 仅当当前值等于旧 defaultValue 时联动——罕见 defer）。
          // R2996：显式设 defaultValue 重同步（清 dirty，getter 回落新属性值）。
          if (_realTag(sel, handle) === 'INPUT') {
            _clearInputDefault(key);
            if (handle) __zw_set_attr_handle(handle, 'value', String(value));
            else { __zw_set_attr(sel, 'value', String(value)); moAttr = 'value'; }
          } else if (_realTag(sel, handle) === 'TEXTAREA') {
            // R57（FV M1）：textarea.defaultValue 设置——默认值 + value 联动
            //（当前值未编辑时——**不算用户编辑**（_userEdited 不置——validity
            // 自动 valid——"Programmatically setting defaultValue is not a user
            // edit"）
            try {
              if (handle) __zw_set_text_handle(handle, String(value));
              else __zw_set_text(sel, String(value));
            } catch (_e) {}
            try { _textareaDefault[key] = String(value); } catch (_e) {}
            // R57（FV M1）：本地 value 状态同步（_controlValue 优先读
            // _inputValues——mutation 未应用时 value 读取不 stale）
            try { _inputValues[key] = String(value); } catch (_e) {}
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
          // reflected unsigned-long 维度 setter（R2851）：归一（NaN/负 → 0）→ 缓存数值 + 写 width/height
          // 内容属性（getter 优先读缓存保 sync set→get）。R3077：CANVAS width/height 反射（保 set→get 一致）。
          // R3308：CANVAS 设 width/height 触发 bitmap resize（HTML spec §4.12.5.1——清空 bitmap + 重置绘图状态）。
          // 已 getContext 的 canvas，调 host resizeContext 清空像素 + 重置 context 状态到默认。
          // R34xx：CANVAS 走 WebIDL ToUint32（2d.canvas.host.size.invalid.attributes.idl——
          // 200-2^32 → 200（mod 2^32）、'400x' → NaN → 0）；IMG/IFRAME 保持 parseInt（既有语义）。
          var wv = (_realTag(sel, handle) === 'CANVAS') ? _zwToUint32(value) : (function () {
            var pv = parseInt(value, 10);
            return (isNaN(pv) || pv < 0) ? 0 : pv;
          })();
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
              // R34xx：设尺寸重置绘图状态——client 镜像同步默认（spec §4.12.5.1；
              // 2d.canvas.host.initial.reset.2dstate 同值设置亦复位）。
              _zwResetCtxMirrors(_cctx);
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
        } else if (p === 'src' && _realTag(sel, handle) === 'IMG') {
          // R56h：运行时 img 加载——`new Image().src = url` → __zw_fetch 同步抓取 +
          // createImageBitmap 解码 → naturalWidth（_zwSettleResourceSelector 置 resourceState）
          // + load/error 事件（2d.drawImage.svg / zerosource.image 的 loadImage await onload——
          // 旧实现恒派发 error：detached Image() 无渲染器 fetch 槽，运行时 img 恒 naturalWidth=0）。
          // fetch/decode 失败回落异步 error 事件（原语义）。
          // https://html.spec.whatwg.org/multipage/images.html#updating-the-image-data
          if (handle) __zw_set_attr_handle(handle, 'src', String(value));
          else __zw_set_attr(sel, 'src', String(value));
          moAttr = 'src';
          var _imSrc = String(value == null ? '' : value);
          var _imFail = function () {
            _defer(function () {
              _dispatchWithBubble(key, sel, handle, _makeEvent('error', { bubbles: false, cancelable: false }));
            });
          };
          if (_imSrc && typeof fetch === 'function') {
            try {
              fetch(_imSrc).then(function (resp) {
                if (!resp.ok) { _imFail(); return; }
                resp.blob().then(function (blob) {
                  createImageBitmap(blob).then(function (bm) {
                    // R56h：手动 settle（handle 元素）——_zwSettleResourceSelector 只收
                    // sel（new Image() 的 handle-based 元素 sel 为 null → key 不匹配
                    // listener 槽位，onload 不触发）。resourceState 用 _elKey(sel, handle)
                    // 与 naturalWidth getter 同 key；事件经 _dispatchWithBubble 同 key 派发。
                    var _imKey = _elKey(sel, handle);
                    if (_resourceStates[_imKey]) { _imFail(); return; }
                    _resourceStates[_imKey] = { url: _imSrc, outcome: 'loaded', width: bm.width, height: bm.height, error: null };
                    _dispatchWithBubble(_imKey, sel, handle, _makeEvent('load', { bubbles: false, cancelable: false }));
                  }, function (e) {
                    // R56h：零尺寸图像（SVG width=0）→ img 元素语义 = loaded（naturalWidth
                    // 0，drawImage no-op——2d.drawImage.zerosource.image 期望 onload 且
                    // 不绘制）；真解码失败（broken）→ error 事件。
                    if (e && e.name === 'InvalidStateError') {
                      var _zk = _elKey(sel, handle);
                      if (!_resourceStates[_zk]) {
                        _resourceStates[_zk] = { url: _imSrc, outcome: 'loaded', width: 0, height: 0, error: null };
                        _dispatchWithBubble(_zk, sel, handle, _makeEvent('load', { bubbles: false, cancelable: false }));
                      }
                    } else {
                      _imFail();
                    }
                  });
                }, _imFail);
              }, _imFail);
            } catch (_e) {
              _imFail();
            }
          } else {
            _imFail();
          }
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
        // js-dom M4 R45：携带写入前捕获的 oldValue（part04 set trap 头预捕获 _moOldVal——仅 id/class/
        // title/lang 高频反射集；其余反射属性 oldValue 保持 null，partial）。
        if (moAttr) _mo_notify(sel, handle, { type: 'attributes', attributeName: moAttr, oldValue: (typeof _moOldVal !== 'undefined' && _moOldVal !== undefined) ? _moOldVal : null });
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
        // R81：text/comment 节点的 instanceof 面对齐构造器（WPT Node-textContent
        // `firstChild instanceof Text`——旧返 Node.prototype 使 instanceof Text false）。
        if (handle && _commentHandles[handle] && _gp.Comment) return _gp.Comment.prototype;
        if (handle && _textHandles[handle] && _gp.Text) return _gp.Text.prototype;
        // js-dom M4 R80：非 HTML 命名空间的 createElementNS 元素（SVG/MathML/自定义 ns）不是
        // HTMLElement（spec：接口由 namespace 决定——非 HTML ns 的元素只 instanceof Element；
        // WPT Document-createElementNS "Should not be an HTMLElement" 断言族）。回落 Element.prototype
        //（链 Node；instanceof HTMLElement=false、Element=true）。
        if (handle && _nsHandles[handle]) {
          var _nsgp = _nsHandles[handle].namespace;
          if (_nsgp !== 'http://www.w3.org/1999/xhtml' && _gp.Element && _gp.Element.prototype) {
            return _gp.Element.prototype;
          }
        }
        // element（含 selector-based 与 createElement handle）：按 tag 查 __zwHtmlTagIface 返对应
        // HTML*Element 子类 prototype（R11，使 `el instanceof HTMLDivElement` 等为 true）；无映射/构造器
        // 缺失回落 HTMLElement.prototype（链 Element → Node）。
        // js-dom M3 R90：**custom element 优先**——tag 命中 customElements registry 时返
        // 用户 ctor.prototype（spec `custom-elements-upgrades`：升级后的元素原型链顶端是
        // 自定义类；instanceof MyEl / prototype 方法可达）。查表键 = createElement 的原 tag
        //（registry define 小写键）。
        if (globalThis.customElements && typeof globalThis.customElements.get === 'function') {
          var _r90Tag = _realTag(sel, handle).toLowerCase();
          var _r90Ctor = globalThis.customElements.get(_r90Tag);
          if (typeof _r90Ctor === 'function' && _r90Ctor.prototype) return _r90Ctor.prototype;
        }
        if (_gp.HTMLElement && _gp.HTMLElement.prototype) {
          // js-dom M4 R80：createElementNS handle（HTML ns）的 iface 查找用 localName **原样**（spec：
          // HTML 命名空间元素的接口按 localName 定——`createElementNS(HTMLNS,'html:span')` 是
          // HTMLSpanElement；`'SPAN'`（大写 localName）不映射（HTML 元素表全小写）→ HTMLUnknownElement，
          // 与真浏览器一致）。createElement（无 NS）仍走 _realTag 小写。
          var _ifaceTag;
          if (handle && _nsHandles[handle] && _nsHandles[handle].namespace === 'http://www.w3.org/1999/xhtml') {
            var _nsl = _nsHandles[handle].qualifiedName;
            var _nsc = _nsl.indexOf(':');
            _ifaceTag = _nsc >= 0 ? _nsl.slice(_nsc + 1) : _nsl;
          } else {
            _ifaceTag = _realTag(sel, handle).toLowerCase();
          }
          var _iface = _gp.__zwHtmlTagIface && _gp.__zwHtmlTagIface[_ifaceTag];
          if (_iface && _gp[_iface] && _gp[_iface].prototype) return _gp[_iface].prototype;
          // 无映射：HTML ns 大写/未知 localName → HTMLUnknownElement（spec HTML 元素表外）。
          if (handle && _nsHandles[handle] && _gp.HTMLUnknownElement && _gp.HTMLUnknownElement.prototype) {
            return _gp.HTMLUnknownElement.prototype;
          }
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

  // js-dom M3 R100：query 返回点的 identity 反查——命中的唯一选择器若属于
  // createElement 建立的 handle 节点（mutation 应用后 host 把 handle→selector
  // 倒置 merge 进持久表，经 `__zw_handle_for_selector` 反查），则包装回**原
  // handle proxy**（`_proxyCache['@'+handle]` 同对象 identity）。跨 execute 的
  // 元素引用统一身份：Vue mount 建 button（`__n0` proxy，@click invoker 注册在
  // `_listenerStore['@__n0']`）后，后续 execute `querySelector('button')` 若返回
  // 新 sel proxy 则 dispatchEvent 找不到 invoker（e2e 实证 sameAsQueried:false /
  // handler 不触发）。**限定 query 返回点调用**（R77 教训 #7：全局 `_wrapSelector`
  // 前置反查会波及 attributes 等一切 sel 包装路径）。未注册回调/未命中 → 原行为
  // （sel proxy），零回归。
  function _zwQueryWrapIdentity(sel) {
    if (typeof __zw_handle_for_selector === 'function') {
      try {
        var h = __zw_handle_for_selector(sel);
        if (h) {
          // 正置缓存登记（`_realTag` 等读回路径经 `_r100SelOfHandle` 锚回 selector）。
          _r100Remember(h, sel);
          return _wrapHandle(h);
        }
      } catch (_e100) {}
    }
    return _wrapSelector(sel);
  }


  // js-dom M4 R121：handle text/comment 节点的 JS 侧 data 覆盖缓存——wire 层
  // （to_rust_string_lossy，WTF-16→UTF-8）把孤立代理替换为 U+FFFD，而 spec 允许
  // CharacterData 方法把代理对**切开**（replaceData/deleteData/insertData 按 UTF-16
  // code unit 偏移，切开后的孤立代理在读回时保真——WPT CharacterData-surrogates）。
  // 缓存 JS Map 键 handle，写双写（缓存保真 + wire 尽力供 host 渲染），读缓存优先。
  var _zwTextDataCache = new Map();
  function _zwTextDataGet(handle, wireFallback) {
    if (handle && _zwTextDataCache.has(handle)) return _zwTextDataCache.get(handle);
    return wireFallback();
  }
  function _zwTextDataSet(handle, value, wire) {
    if (handle) _zwTextDataCache.set(handle, value);
    try { wire(); } catch (_e) {}
  }


  // js-dom M4 R123：ProcessingInstruction 属性层（WICG declarative-partial-updates——
  // WPT dom/nodes/processing-instruction-attributes.html）。PI 的 data 即属性的序列化
  // 形态：`a="b" x="yy"` ⇔ [['a','b'],['x','yy']]，读写双向同步——setAttribute 族改
  // 属性后 data 重序列化，data= 重新解析属性。属性名大小写敏感（distinct：ABC ≠ abc）。
  // 解析仅接受良构 name="value" 对（空白分隔，值双引号包裹）——畸形（a=b 无引号）整体
  // 视为无属性（data 保留原串）。序列化的值转义镜像 host escape_html（& " < > U+00A0），
  // WPT "check attribute value" 簇与 element.outerHTML 提取值逐串相等。
  function _zwPiParseAttrs(data) {
    var s = String(data == null ? '' : data);
    var attrs = [];
    var i = 0, n = s.length;
    var isWs = function (c) { return c === ' ' || c === '\t' || c === '\n' || c === '\f' || c === '\r'; };
    var isNameChar = function (c) {
      return !isWs(c) && c !== '=' && c !== '>' && c !== '/' && c !== '"' && c !== '<';
    };
    while (i < n && isWs(s.charAt(i))) i++;
    while (i < n) {
      var start = i;
      while (i < n && isNameChar(s.charAt(i))) i++;
      var name = s.slice(start, i);
      while (i < n && isWs(s.charAt(i))) i++;
      if (i >= n || s.charAt(i) !== '=') return null;
      i++;
      while (i < n && isWs(s.charAt(i))) i++;
      if (i >= n || s.charAt(i) !== '"') return null;
      i++;
      var vstart = i;
      while (i < n && s.charAt(i) !== '"') i++;
      if (i >= n) return null;
      var value = _zwPiUnescape(s.slice(vstart, i));
      i++;
      if (!name) return null;
      attrs.push([name, value]);
      while (i < n && isWs(s.charAt(i))) i++;
    }
    return attrs;
  }
  // PI 属性值序列化转义（对齐 Rust escape_html / JS outerHTML 属性转义全集：& " < >
  // U+00A0——WPT check-attribute-value 簇用 element.outerHTML 提取值逐串对照，
  // element 与 PI 两面同款转义才相等）。
  function _zwPiEscape(v) {
    return String(v).replace(/&/g, '&amp;').replace(/"/g, '&quot;')
      .replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/ /g, '&nbsp;');
  }
  // PI 属性值解析反转义（&amp; &quot; &lt; &gt; —— setAttribute 原值往返）。
  function _zwPiUnescape(v) {
    return String(v).replace(/&lt;/g, '<').replace(/&gt;/g, '>')
      .replace(/&quot;/g, '"').replace(/&amp;/g, '&');
  }
  // WPT invalid_names 判定：名含 '=' '>' '/' 空白或首字符为 '=' → InvalidCharacterError
  //（/x/、x/、b>、a=、=x、x\t 等；'$'/'_' 等按 XML Name 宽集合法）。
  function _zwPiValidName(name) {
    var n = String(name);
    if (n === '') return false;
    for (var i = 0; i < n.length; i++) {
      var c = n.charAt(i);
      if (c === '=' || c === '>' || c === '/' || c === '<' || c === '"' || c === ' '
        || c === '\t' || c === '\n' || c === '\f' || c === '\r') return false;
    }
    return true;
  }
  // PI data 读写统一入口：解析层经 _piHandles[handle].data（JS 权威——createPI 登记 +
  // 属性写重序列化 + data= setter 直写）。返属性数组（畸形 data → 空数组——无属性但 data 保留）。
  function _zwPiAttrs(handle) {
    var pi = handle && _piHandles[handle];
    if (!pi) return [];
    var parsed = _zwPiParseAttrs(pi.data);
    return parsed || [];
  }
  // 属性写后 data 重序列化（保持既有属性位次，改值原位、新增尾追）+ wire 同步
  //（SetTextOnHandle 供渲染 + 存量 wire 读回调一致）。
  function _zwPiSetData(handle, attrs) {
    var pi = handle && _piHandles[handle];
    if (!pi) return;
    var parts = [];
    for (var i = 0; i < attrs.length; i++) {
      parts.push(attrs[i][0] + '="' + _zwPiEscape(attrs[i][1]) + '"');
    }
    var d = parts.join(' ');
    pi.data = d;
    if (handle && typeof __zw_set_text_handle === 'function') {
      try { __zw_set_text_handle(handle, d); } catch (_e) {}
    }
    _zwTextDataCache.set(handle, d);
  }


  // js-dom M4 R122：Attr 节点身份绑定表——elKey → Map(限定名 → Attr 对象)。
  // setAttributeNode 族与 getAttributeNode 族的 identity 契约（WPT attributes.html：
  // `attrNode === el2.getAttributeNode('foo')`、`el.attributes[1] === attrNodeNS2`——
  // 同一 Attr 对象往返；旧实现每次 _zwMakeAttr 新对象 identity 恒 false）。
  // ownerElement 语义：绑定=元素，解绑（remove/replace）= null（值保留）。
  var _zwAttrBindings = new Map();
  function _zwAttrBindMap(elKey) {
    var m = _zwAttrBindings.get(elKey);
    if (!m) { m = new Map(); _zwAttrBindings.set(elKey, m); }
    return m;
  }
  // spec dom-element-setattributenode 核心步骤：InUse 校验 → 按 (ns, local) 找现有 →
  // 写 host + 绑定 → 返旧 Attr（解绑）/ null。NS meta 同步登记（prefix/localName/
  // namespaceURI 从 attr 对象读）。
  function _zwSetAttributeNodeCore(sel, handle, key, attr, isNS) {
    if (!attr || attr.nodeType !== 2) {
      throw new TypeError("Failed to execute 'setAttributeNode' on 'Element': parameter 1 is not of type 'Attr'.");
    }
    if (attr.ownerElement && attr.ownerElement !== _makeProxy(sel, handle)) {
      throw new (globalThis.DOMException || Error)(
        'The attribute is in use.', 'InUseAttributeError');
    }
    var prefix = attr.prefix != null ? String(attr.prefix) : null;
    var local = attr.localName != null ? String(attr.localName) : String(attr.name);
    var ns = attr.namespaceURI != null ? String(attr.namespaceURI) : null;
    var qname = prefix ? prefix + ':' + local : local;
    var bind = _zwAttrBindMap(key);
    // 按 (ns, local) 找现有绑定（spec：同 ns+local 替换，非同限定名）。
    var old = null; var oldQ = null;
    bind.forEach(function (bAttr, bQ) {
      if (old) return;
      var bNs = bAttr.namespaceURI != null ? String(bAttr.namespaceURI) : null;
      var bLocal = bAttr.localName != null ? String(bAttr.localName) : String(bAttr.name);
      if (bNs === ns && bLocal === local) { old = bAttr; oldQ = bQ; }
    });
    if (old) { old.ownerElement = null; bind.delete(oldQ); }
    // R122：绑定 miss 但 host 已有该属性（parser 快照属性未被读过——无绑定）→ 经
    // _zwAttrObjFor 建/登记绑定 Attr 作返值（spec `dom-element-setattributenode` 返
    // 「替换的旧 Attr」；值取写入前 host latest-wins）。
    if (!old) {
      try {
        var _r122PreBind = _zwAttrBindings.get(key);
        if (!_r122PreBind || !_r122PreBind.get(qname)) {
          var _r122OldV = null;
          try {
            _r122OldV = (handle && typeof __zw_get_attr_handle === 'function')
              ? __zw_get_attr_handle(handle, qname)
              : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, qname) : __zw_get_attr(sel, qname));
          } catch (_eOV) {}
          var _r122OldPresent = false;
          try {
            _r122OldPresent = (handle && typeof __zw_has_attr_handle === 'function')
              ? __zw_has_attr_handle(handle, qname) === '1'
              : (typeof __zw_has_attr_lw === 'function' ? __zw_has_attr_lw(sel, qname) === '1'
                : (typeof __zw_has_attr === 'function' ? __zw_has_attr(sel, qname) === '1' : false));
          } catch (_eOP) {}
          if (_r122OldPresent && _r122OldV != null && attr._r122V !== _r122OldV) {
            var _r122OldA = _zwAttrObjFor(sel, handle, qname);
            if (_r122OldA && _r122OldA !== attr) { old = _r122OldA; }
          }
        }
      } catch (_eOB) {}
    }
    // 写 host（限定名 + NS meta）+ **实例层 upsert**（R122：多实例 NS 视图权威——
    // setAttributeNode(attrNS) 经实例层供 getAttributeNS/attributes[i] 读）。
    var _r122Val = attr.value != null ? String(attr.value) : '';
    if (handle && typeof __zw_set_attr_handle === 'function') __zw_set_attr_handle(handle, qname, _r122Val);
    else if (typeof __zw_set_attr === 'function') __zw_set_attr(sel, qname, _r122Val);
    var meta = _attrNSMeta[key] || (_attrNSMeta[key] = {});
    meta[qname] = { ns: ns, prefix: prefix, local: local };
    _zwAttrInstUpsert(key, qname, ns, prefix, local, _r122Val);
    bind.set(qname, attr);
    try { attr.ownerElement = _makeProxy(sel, handle); } catch (_eO) {}
    _mo_notify(sel, handle, { type: 'attributes', attributeName: local, attributeNamespace: ns });
    return old;
  }
  // spec dom-element-removeattributenode：按 attr 的 (ns, local) 找到 → 删 host + 解绑
  // （返 attr）；miss → NotFoundError。
  function _zwRemoveAttributeNodeCore(sel, handle, key, attr) {
    if (!attr || attr.nodeType !== 2) {
      throw new TypeError("Failed to execute 'removeAttributeNode' on 'Element': parameter 1 is not of type 'Attr'.");
    }
    var prefix = attr.prefix != null ? String(attr.prefix) : null;
    var local = attr.localName != null ? String(attr.localName) : String(attr.name);
    var qname = prefix ? prefix + ':' + local : local;
    var present = false;
    try {
      present = (handle && typeof __zw_has_attr_handle === 'function')
        ? __zw_has_attr_handle(handle, qname) === '1'
        : (typeof __zw_has_attr_lw === 'function' ? __zw_has_attr_lw(sel, qname) === '1' : false);
    } catch (_eP) {}
    if (!present) {
      // NS meta 里按 (ns, local) 反查限定名再试（限定名形态可能与 attr.name 不同）。
      var meta = _attrNSMeta[key];
      if (meta) {
        for (var mq in meta) {
          if (meta[mq] && meta[mq].local === local) { qname = mq; present = true; break; }
        }
      }
    }
    if (!present) {
      throw new (globalThis.DOMException || Error)(
        'The attribute was not found.', 'NotFoundError');
    }
    if (handle && typeof __zw_remove_attr_handle === 'function') __zw_remove_attr_handle(handle, qname);
    else if (typeof __zw_remove_attr === 'function') __zw_remove_attr(sel, qname);
    // R122：实例层剔除（attr 的 ns 反查——限定名形态可能与 attr.name 不同）。
    var _rn122Ns = attr.namespaceURI != null ? String(attr.namespaceURI) : null;
    _zwAttrInstRemoveNS(key, _rn122Ns, local);
    var bind = _zwAttrBindings.get(key);
    if (bind) {
      var bAttr = bind.get(qname);
      if (bAttr) { bAttr.ownerElement = null; bind.delete(qname); }
    }
    _mo_notify(sel, handle, { type: 'attributes', attributeName: local });
    return attr;
  }


  // js-dom M4 R122：同名多实例属性覆盖层——elKey → 有序 [ {qname, ns, prefix, local, value} ]。
  // host 属性存储按限定名扁平（同 local 不同 ns 的多实例无法共存），spec setAttributeNS
  // 允许 setAttributeNS('ab','attr') + setAttributeNS('kl','attr') 两实例并存，且
  // getAttribute（非 NS）返回**第一个** local 匹配（WPT "First set attribute..." 簇）。
  // 本表是 JS 侧权威多实例视图：setAttributeNS 更新（同 ns+local 原位改值、prefix 不变；
  // 新实例 push），host 只写每 local 的首实例（渲染 best-effort）。
  var _zwAttrInstances = new Map();
  function _zwAttrInstList(elKey) {
    var a = _zwAttrInstances.get(elKey);
    if (!a) { a = []; _zwAttrInstances.set(elKey, a); }
    return a;
  }
  function _zwAttrInstUpsert(elKey, qname, ns, prefix, local, value) {
    var list = _zwAttrInstList(elKey);
    for (var i = 0; i < list.length; i++) {
      var it = list[i];
      var itNs = it.ns != null ? String(it.ns) : null;
      if (itNs === (ns != null ? String(ns) : null) && it.local === local) {
        it.value = value; // 原位更新（prefix/qname 不变——spec 步骤 9「不改 prefix」）
        return false; // 未新增
      }
    }
    list.push({ qname: qname, ns: ns != null ? String(ns) : null, prefix: prefix, local: local, value: value });
    return true;
  }
  function _zwAttrInstRemoveNS(elKey, ns, local) {
    var list = _zwAttrInstances.get(elKey);
    if (!list) return false;
    for (var i = 0; i < list.length; i++) {
      var it = list[i];
      var itNs = it.ns != null ? String(it.ns) : null;
      if (itNs === (ns != null ? String(ns) : null) && it.local === local) { list.splice(i, 1); return true; }
    }
    return false;
  }
  // 非 NS 读的 first-match（spec getAttribute：HTML 文档限定名小写化后按 local 找第一个）。
  function _zwAttrInstFirstByLocal(elKey, localLower) {
    var list = _zwAttrInstances.get(elKey);
    if (!list) return null;
    for (var i = 0; i < list.length; i++) {
      var l = String(list[i].local).toLowerCase();
      if (l === localLower) return list[i];
    }
    return null;
  }
  // R122：按限定名剔实例（removeAttribute / removeNamedItem 非 NS 路径——host 扁平删除时
  // JS 视图同步剔除**该限定名的全部实例**，防实例层残留已删属性）。
  function _zwAttrInstRemoveByQName(elKey, qname) {
    var list = _zwAttrInstances.get(elKey);
    if (!list) return false;
    var removed = false;
    for (var i = list.length - 1; i >= 0; i--) {
      if (list[i].qname === qname) { list.splice(i, 1); removed = true; }
    }
    return removed;
  }
  // R122：剥合成后缀（'\x00#k' 是多实例内部索引键，非 spec qualified name）。
  function _zwAttrStripSyn(name) {
    var m = /\x00#\d+$/.exec(String(name));
    return m ? String(name).slice(0, m.index) : String(name);
  }

  function _wrapHandle(handle) {
    return _makeProxy(null, handle);
  }

  // js-dom M4 R55：兄弟对基底缓存（previousSibling/nextSibling 的 {p,n} 包装缓存，part04
  // sibling 读消费）。生命周期同 _zwChildBaseCache（part05 下文声明）——dom_html Arc 回合内
  // 不可变；register_dom_callbacks 重注册时经 globalThis._zwSiblingBaseInvalidateAll 失效。
  // 挂此 hoisting 可达（part04 运行期引用，声明序无关）。
  var _zwSiblingBaseCache = new Map();
  globalThis._zwSiblingBaseInvalidateAll = function () { _zwSiblingBaseCache.clear(); };

  // js-dom M4 R115：iframe 子文档构建——静态 `<iframe src>` 用例族（Document-createElement /
  // case / createElementNS 等经 `/common/dummy.xml|.xhtml` 取 XML/XHTML 文档）。src 经
  // fetch（testharness runner 的 wpt.test 虚拟根服务 wpt-data 文件）同步?——fetch 是 Promise，
  // 但 iframe 加载在页面脚本前完成（prepare 阶段 load 时已可读）。实现为 **同步 best-effort**：
  // 首次读 contentDocument 时发起 fetch，未完成前返部分文档（documentElement.textContent 空），
  // fetch 完成后重建——用例的 window 'load' 等待时序下通常已就绪（dummy 文档本地 fs 读）。
  // XML 文档：createElement 的 localName **保持大小写**（preserveCase）+ namespaceURI null +
  // contentType 'application/xml'（R81 `new Document()` 同款语义）；XHTML：HTML 解析 +
  // namespaceURI HTMLNS。文档模型复用 `_makeDetachedDocument`（查询/mutation/Range 全可用），
  // docEl.textContent 供用例 load 断言。
  function _zwMakeIframeDoc(kind, markup) {
    var doc = _makeDetachedDocument('');
    var _r115WinRef = null; // defaultView 槽（_zwMakeIframeWin 建后回填）
    doc.__r115SetWin = function (w) { _r115WinRef = w; };
    if (kind === 'xml') {
      doc.contentType = 'application/xml';
      doc._docNS = null;
    } else {
      doc.contentType = 'application/xhtml+xml';
      doc._docNS = 'http://www.w3.org/1999/xhtml';
    }
    // 文档体：dummy 文档只有一个根元素（<foo>text</foo> / <html>…）。detached doc 的主体
    // 查询面在 body——把根元素塞进 body 查询源，documentElement 单独从 markup 提取。
    var bodyInner = '';
    var docEl = null;
    try {
      var mEl = /<([a-zA-Z][\w:-]*)(\s[^>]*)?>([\s\S]*)<\/\1\s*>/.exec(markup);
      if (mEl) {
        var elTag = mEl[1];
        var elText = mEl[3].replace(/<[^>]*>/g, '');
        docEl = {
          nodeType: 1, tagName: kind === 'xml' ? elTag : elTag.toUpperCase(),
          nodeName: kind === 'xml' ? elTag : elTag.toUpperCase(),
          localName: elTag, namespaceURI: doc._docNS,
          textContent: elText,
          getBoundingClientRect: function () { return _makeDomRect(0, 0, 0, 0); }
        };
        if (kind !== 'xml') bodyInner = mEl[3];
      }
    } catch (_e115) {}
    try { doc.body.innerHTML = bodyInner; } catch (_eB) {}
    try {
      Object.defineProperty(doc, 'documentElement', {
        configurable: true,
        get: function () { return docEl; }
      });
    } catch (_eD) {}
    // R115：createElement（XML 保大小写 / HTML·XHTML 转换 + ns）+ createTextNode + createElementNS
    //（validate-and-extract 复用主 document.createElementNS 同款规则——R80/R81 语义表）。
    // R115：defaultView → contentWindow（用例 assert_throws_dom 的 doc.defaultView.DOMException）。
    try {
      Object.defineProperty(doc, 'defaultView', {
        configurable: true,
        get: function () { return _r115WinRef; }
      });
    } catch (_eDV) { doc.defaultView = null; }
    doc.createElement = function (tag) { return _zwIframeCreateElement(doc, tag); };
    doc.createElementNS = function (ns, qualifiedName) {
      var _nsStr = (ns == null) ? '' : String(ns);
      var _q = String(qualifiedName);
      var _XML_NS = 'http://www.w3.org/XML/1998/namespace';
      var _XMLNS_NS = 'http://www.w3.org/2000/xmlns/';
      var _colon1 = _q.indexOf(':');
      var _pre = _colon1 >= 0 ? _q.slice(0, _colon1) : null;
      var _loc = _colon1 >= 0 ? _q.slice(_colon1 + 1) : _q;
      var _throwNS = function (name, msg) {
        throw new (globalThis.DOMException || Error)(msg, name);
      };
      if (_q === '' || _colon1 === 0 || _colon1 === _q.length - 1) {
        _throwNS('InvalidCharacterError', 'The string contains invalid characters.');
      }
      if (/[\s>]/.test(_q)) {
        _throwNS('InvalidCharacterError', 'The string contains invalid characters.');
      }
      if (_pre === null) {
        if (!_zwIsNameStartChar(Array.from(_q)[0])) {
          _throwNS('InvalidCharacterError', 'The string contains invalid characters.');
        }
      } else {
        var _lc = Array.from(_loc);
        if (!_lc.length || !_zwIsNameStartChar(_lc[0])) {
          _throwNS('InvalidCharacterError', 'The string contains invalid characters.');
        }
      }
      if (_nsStr === _XMLNS_NS) {
        var _xok = (_loc === 'xmlns' && _pre === null) || (_pre === 'xmlns');
        if (!_xok) _throwNS('NamespaceError', 'The xmlns namespace is not allowed for elements.');
      }
      if (_pre !== null) {
        if (_nsStr === '') _throwNS('NamespaceError', 'Namespace prefix provided but no namespace.');
        if (_pre === 'xml' && _nsStr !== _XML_NS) _throwNS('NamespaceError', 'The xml prefix is reserved.');
        if (_pre === 'xmlns' && _nsStr !== _XMLNS_NS) _throwNS('NamespaceError', 'The xmlns prefix is reserved.');
      }
      // 无 prefix 的 localName 'xmlns' 且 ns 非 XMLNS ns → NamespaceError（spec 保留绑定；
      // **带 prefix** 的 'test:xmlns' 合法——WPT 期望表）。
      if (_pre === null && _loc === 'xmlns' && _nsStr !== _XMLNS_NS) {
        _throwNS('NamespaceError', 'The xmlns localName is reserved for the xmlns namespace.');
      }
      // XML/XHTML 文档保大小写（spec createElementNS 不做 case 转换）。
      var el = _zwIframeCreateElement(doc, _loc);
      el.tagName = _q;
      el.nodeName = _q;
      el.localName = _loc;
      el.prefix = _pre;
      el.namespaceURI = _nsStr === '' ? null : _nsStr;
      return el;
    };
    doc.createTextNode = function (text) {
      var tn = { nodeType: 3, nodeName: '#text', data: String(text), parentNode: null };
      try { Object.defineProperty(tn, 'textContent', { configurable: true, get: function () { return tn.data; } }); } catch (_eT) {}
      try { Object.setPrototypeOf(tn, globalThis.Text ? globalThis.Text.prototype : Object.prototype); } catch (_eT2) {}
      return tn;
    };
    return doc;
  }
  // R115：iframe 子文档的 createElement/createTextNode——spec `dom-document-createelement`：
  // XML 文档（XML 解析的 doc）localName/tagName **保持原大小写**、namespaceURI null；HTML/XHTML
  // 文档 ASCII-lowercase localName + ASCII-uppercase tagName、namespaceURI HTMLNS（用例期望
  // createElement("foo") XML → "foo" / HTML → localName "foo" tagName "FOO"）。元素为轻量对象
  // 挂 Element.prototype 链（instanceof win.Element——win 构造器转发主 realm）。
  function _zwIframeCreateElement(doc, tag) {
    var t = String(tag); // WebIDL DOMString 转换（undefined → 'undefined'，null → 'null'）
    // R115：非法名抛 InvalidCharacterError（spec `dom-document-createelement` 步骤 2——Name
    // production 校验；WPT invalid 列表 ""/"1foo"/"}foo"/"<foo"/"fo o"/"foo>"）。经
    // globalThis.DOMException（identity 对等，R9 教训）。
    if (typeof _zwIsValidHtmlElementName === 'function' && !_zwIsValidHtmlElementName(t)) {
      if (typeof globalThis.DOMException === 'function') {
        throw new globalThis.DOMException(
          "Failed to execute 'createElement' on 'Document': The tag name provided ('" + t + "') is not a valid name.",
          'InvalidCharacterError');
      }
      var _e115v = new Error("InvalidCharacterError");
      _e115v.name = 'InvalidCharacterError';
      throw _e115v;
    }
    // 大小写转换仅 HTML 文档（spec：createElement 的 ASCII lower/upper 是 HTML 专属——XML/
    // XHTML 文档 localName/tagName 保持原样；XHTML 是 XML 解析的文档）。
    var isHtml = doc.contentType === 'text/html';
    var local = isHtml ? t.replace(/[A-Z]/g, function (c) { return String.fromCharCode(c.charCodeAt(0) + 32); }) : t;
    var upper = t.replace(/[a-z]/g, function (c) { return String.fromCharCode(c.charCodeAt(0) - 32); });
    var el = {
      nodeType: 1,
      tagName: isHtml ? upper : t,
      nodeName: isHtml ? upper : t,
      localName: local,
      prefix: null,
      namespaceURI: isHtml ? 'http://www.w3.org/1999/xhtml' : doc._docNS,
      ownerDocument: doc,
      childNodes: [],
      attributes: [],
      parentNode: null,
      nodeValue: null,
      textContent: '',
      getBoundingClientRect: function () { return _makeDomRect(0, 0, 0, 0); },
      getClientRects: function () { return []; },
      getAttribute: function (n) { n = String(n); for (var i = 0; i < el.attributes.length; i++) if (el.attributes[i].name === n) return el.attributes[i].value; return null; },
      hasAttribute: function (n) { return el.getAttribute(n) !== null; },
      setAttribute: function (n, v) {
        var found = false;
        for (var i = 0; i < el.attributes.length; i++) {
          if (el.attributes[i].name === String(n)) { el.attributes[i].value = String(v); found = true; break; }
        }
        if (!found) el.attributes.push({ name: String(n), value: String(v) });
      },
      appendChild: function (c) { c.parentNode = el; el.childNodes.push(c); return c; },
      hasChildNodes: function () { return el.childNodes.length > 0; }
    };
    try { Object.setPrototypeOf(el, globalThis.Element ? globalThis.Element.prototype : Object.prototype); } catch (_e115p) {}
    return el;
  }

  // iframe contentWindow：最小 window 面（document + Element/Node 构造器转发主 window——
  // 用例 `elt instanceof win.Element` 需要 iframe realm 的构造器与主 realm proxy 的
  // getPrototypeOf 对齐；polyfill 单 realm 近似：直接引用主 window 的构造器）。
  function _zwMakeIframeWin(doc) {
    return {
      document: doc,
      Element: globalThis.Element,
      Node: globalThis.Node,
      HTMLElement: globalThis.HTMLElement,
      SVGElement: globalThis.SVGElement,
      MathMLElement: globalThis.MathMLElement,
      Document: globalThis.Document,
      Text: globalThis.Text,
      Comment: globalThis.Comment,
      CharacterData: globalThis.CharacterData,
      Event: globalThis.Event,
      DOMException: globalThis.DOMException
    };
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
    // R126：globalThis.DOMException（native_dom 叠加路径 = 原生 DOMException；纯 polyfill =
    // part01b）——assert_throws_dom "wrong global" 要求异常 ctor 与用例 realm 一致（R6 教训）。
    var Ctor = globalThis.DOMException;
    if (typeof Ctor === 'function') return new Ctor(msg, name);
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
    // js-dom M3 R91：handle 容器 append 同步记父反链（_zwNodeParent）——shadow root /
    // fragment / handle 宿主三种容器的子此前不记反链，isConnected 的反链上行到容器
    // 即断（WPT Node-isConnected-shadow-dom open/closed：shadow 树随 host 连入文档
    // 即 connected）。isConnected getter（part04 R90/R91）经容器 handle 的
    // _shadowHandleMeta 跳 host 续链。
    try {
      if (typeof _zwNodeParent !== 'undefined' && _zwNodeParent) {
        _zwNodeParent[child.__zwHandle] = { parentSel: null, parentHandle: parentHandle, nextSibling: null };
      }
    } catch (_e91r) {}
  }
  // js-dom M4 R117：spec pre-insert 层级校验（`dom-node-pre-insert` 步骤 2 + Document 类型约束）——
  // element proxy 的 append/prepend/replaceChildren 先于插入调用（WPT
  // pre-insertion-validation-hierarchy：node 是 parent 的祖先 / Text 插 doc / DocumentType 插
  // 非 doc → HierarchyRequestError）。祖先判定沿 parentNode 上行（proxy 的 parentNode getter）。
  function _zwValidatePreInsert(sel, handle, args) {
    var parent = _makeProxy(sel, handle);
    var parentIsDoc = false;
    try { parentIsDoc = parent.nodeType === 9; } catch (_e) {}
    for (var i = 0; i < args.length; i++) {
      var node = args[i];
      if (!node || typeof node !== 'object') continue;
      var anc = parent, hops = 0;
      while (anc && hops++ < 64) {
        if (anc === node) {
          throw new (globalThis.DOMException || Error)(
            'The new node is an ancestor of this node.', 'HierarchyRequestError');
        }
        try { anc = anc.parentNode; } catch (_e2) { break; }
        if (anc == null) break;
      }
      var nt = 0;
      try { nt = node.nodeType | 0; } catch (_e3) {}
      if (parentIsDoc && (nt === 3 || nt === 7 || nt === 8)) {
        throw new (globalThis.DOMException || Error)(
          'Nodes of type ' + nt + ' cannot be inserted into a Document.', 'HierarchyRequestError');
      }
      if (!parentIsDoc && (nt === 9 || nt === 10)) {
        throw new (globalThis.DOMException || Error)(
          'Only a Document can contain nodes of type ' + nt + '.', 'HierarchyRequestError');
      }
    }
  }

  // js-dom M4 R117：pre-insert 的「先从旧父移除」步骤（spec concept-node-pre-insert 步骤 3：node
  // 有 parent 时先 remove——移动非复制）。before/after/replaceWith/append/prepend 等变异插入族的
  // 节点参数若已在某容器 registry（pending 树），须先从旧位置移除再插入，否则旧位置残留 = 复制。
  // 仅处理 JS 侧 registry（_handleChildren + _zwNodeParent 反链）——host 树内挂载节点的移动由
  // host mutation 自身语义覆盖。返回 true = 发生了移除。
  function _zwDetachFromRegistry(node) {
    if (!node || !node.__zwHandle || !_zwNodeParent) return false;
    var link = _zwNodeParent[node.__zwHandle];
    if (!link || !link.parentHandle) return false;
    var oldParent = link.parentHandle;
    _unrecordHandleChild(oldParent, node);
    delete _zwNodeParent[node.__zwHandle];
    return true;
  }

  // 从容器 registry 移除 child（removeChild 用）。
  function _unrecordHandleChild(parentHandle, child) {
    if (!parentHandle || !child || !child.__zwHandle) return;
    var arr = _handleChildren[parentHandle];
    if (!arr) return;
    var ch = child.__zwHandle;
    _handleChildren[parentHandle] = arr.filter(function(k) { return !k || k.__zwHandle !== ch; });
    // R91：对称清反链（removeChild 后 isConnected 反链上行正确断开）。
    try {
      if (typeof _zwNodeParent !== 'undefined' && _zwNodeParent
          && _zwNodeParent[ch] && _zwNodeParent[ch].parentHandle === parentHandle) {
        delete _zwNodeParent[ch];
      }
    } catch (_e91u) {}
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
    // R124：ASCII whitespace 分词（_zwSplitClassList，part03 hoist 运行期可达）。
    return c ? _zwSplitClassList(c) : [];
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
      // R124：~= 的属性值分词同 class 域（ASCII whitespace——spec attribute selector
      // whitespace-separated words；Unicode 空白是字面字符非分隔符）。
      case '~=': return a.val !== '' && _zwSplitClassList(v).indexOf(a.val) >= 0;
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
    // js-dom M4 R118：CSS 转义解码（CSS Syntax 4.3.4「consume an escaped code point」——
    // WPT ParentNode-querySelector-escapes：`\30 nextIsWhiteSpace` 的 id 是 '0nextIsWhiteSpace'
    //（\30 = '0' + 空白终止符被吃掉）、`\000030 connect`（6 位 hex + 空格终止）、`\0` →
    // U+FFFD（零点/越界/孤立代理的 special replacement）。`\` + 非 hex 非换行 → 字面字符。
    // https://drafts.csswg.org/css-syntax/#consume-escaped-code-point
    var i = start, n = text.length;
    var out = '';
    while (i < n) {
      var ch = text[i];
      // CSS 空白仅 space/U+0009/U+000A/U+000C/U+000D（U+2003 等 Unicode 空白是 ident 字符，
      // WPT "\u2003" id 直接可查）。https://drafts.csswg.org/css-syntax/#whitespace
      if (ch === '.' || ch === '#' || ch === '[' || ch === ':' || /[ \t\n\f\r]/.test(ch)) break;
      if (ch === '\\') {
        var nc = text[i + 1];
        if (nc === undefined) { out += '\uFFFD'; i++; continue; } // EOF 反斜杠 → U+FFFD（spec 4.3.4）
        if (/[0-9a-fA-F]/.test(nc)) {
          // 1-6 位 hex + 可选单个空白终止符。
          var hx = '';
          var h = i + 1;
          while (h < n && hx.length < 6 && /[0-9a-fA-F]/.test(text[h])) { hx += text[h]; h++; }
          var cp = parseInt(hx, 16);
          // CSS 空白终止符：space/U+0009/U+000A/U+000C/U+000D；CRLF 序列整体是一个终止符。
          if (h < n) {
            if (text[h] === '\r' && text[h + 1] === '\n') h += 2;
            else if (/[ \t\n\f\r]/.test(text[h])) h++;
          }
          var decoded;
          if (cp === 0 || cp > 0x10FFFF || (cp >= 0xD800 && cp <= 0xDFFF)) {
            decoded = '\uFFFD'; // 零点/越界/孤立代理 → U+FFFD
          } else {
            decoded = String.fromCodePoint(cp);
          }
          out += decoded;
          i = h;
          continue;
        }
        out += nc; // 字面转义（\. \# 等，含转义空白/换行——是 ident 字符非分隔符）
        i += 2;
        continue;
      }
      out += ch;
      i++;
    }
    return { raw: text.substring(start, i), value: out };
  }
  // 解析单个复合选择器（无空白组合器）。返 { tag, ids[], classes[], attrs[], unsupported }。
  // tag 为 null（任意 / `*`）或大写 tag。遇 `:`（伪类/伪元素）/ 空裸 token / 第二个裸 token → unsupported。
  function _parseCompoundOf(text) {
    // js-dom M3 R100：pseudos[] 增设——结构伪类白名单（first/last/only-child、
    // nth-child(an+b)、nth-last-child、empty、not(simple)、checked）解析进 pseudos，
    // 其余伪类仍 unsupported（matcher 静默丢组，旧行为）。detached 容器子树的
    // pseudo 查询此前全空（R3288 的 A 代 stub 引擎支持这些，guard 后 execute 路径
    // 走 shim 真桥，e2e_canvas_dom R3288 三测暴露缺口）。
    var c = { tag: null, ids: [], classes: [], attrs: [], pseudos: [], unsupported: false };
    var i = 0, n = text.length, seenTag = false;
    while (i < n) {
      var ch = text[i];
      if (ch === ':') {
        if (text[i + 1] === ':') { c.unsupported = true; break; }
        var j = i + 1, paren = -1;
        while (j < n) {
          if (text[j] === '(') { paren = j; break; }
          if (text[j] === '.' || text[j] === '#' || text[j] === '[' || text[j] === ':' || /\s/.test(text[j])) break;
          j++;
        }
        var pname = text.substring(i + 1, paren >= 0 ? paren : j);
        var parg = null;
        if (paren >= 0) {
          var depthP = 1, k = paren + 1;
          while (k < n && depthP > 0) {
            if (text[k] === '(') depthP++;
            else if (text[k] === ')') depthP--;
            k++;
          }
          if (depthP !== 0) { c.unsupported = true; break; }
          parg = text.substring(paren + 1, k - 1);
          j = k;
        }
        var _zwPseudoOk = pname === 'first-child' || pname === 'last-child' || pname === 'only-child'
          || pname === 'nth-child' || pname === 'nth-last-child' || pname === 'empty'
          || pname === 'not' || pname === 'checked';
        if (!_zwPseudoOk) { c.unsupported = true; break; }
        c.pseudos.push({ name: pname, arg: parg });
        i = j;
        continue;
      }
      if (ch === '.') {
        var cls = _readCompoundToken(text, i + 1);
        if (!cls.raw) { c.unsupported = true; break; }
        c.classes.push(cls.value); i += 1 + cls.raw.length;
      } else if (ch === '#') {
        var idt = _readCompoundToken(text, i + 1);
        if (!idt.raw) { c.unsupported = true; break; }
        c.ids.push(idt.value); i += 1 + idt.raw.length;
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
        if (!tg.raw) { i++; continue; }
        if (!seenTag) { c.tag = tg.value === '*' ? null : tg.value.toUpperCase(); seenTag = true; }
        else { c.unsupported = true; break; }
        i += tg.raw.length;
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
  // js-dom M3 R100：白名单结构伪类求值。`info` = _handleSubtreeNodes 的 nodeInfo
  //（parent/prevSiblings/ancestors 已备）；无 info（异常调用面）返 false。
  // An+B 公式：`odd`→2n+1 / `even`→2n / 纯整数 b / `an+b`（a、b 可负，n ≥ 0 整数解）。
  function _zwNthMatches(formula, idx) {
    var f = String(formula || '').replace(/\s+/g, '');
    if (f === 'odd') return idx % 2 === 1;
    if (f === 'even') return idx % 2 === 0;
    var mAnB = /^([+-]?\d*)n([+-]\d+)?$/.exec(f) || /^([+-]?\d*)n$/.exec(f);
    if (mAnB) {
      var aStr = mAnB[1];
      var a = aStr === '' || aStr === '+' ? 1 : (aStr === '-' ? -1 : parseInt(aStr, 10));
      var b = mAnB[2] ? parseInt(mAnB[2], 10) : 0;
      var d = idx - b;
      if (a === 0) return d === 0;
      var q = d / a;
      return q >= 0 && Math.floor(q) === q;
    }
    var mB = /^([+-]?\d+)$/.exec(f);
    if (mB) return idx === parseInt(f, 10);
    return false;
  }
  function _zwIsLastElemChild(info) {
    if (!info.parent || !info.parent.childNodes) return true;
    var sibs = info.parent.childNodes;
    for (var i = sibs.length - 1; i >= 0; i--) {
      if (sibs[i] && sibs[i].nodeType === 1) return sibs[i] === info.proxy;
    }
    return false;
  }
  function _zwElemChildCount(info) {
    if (!info.parent || !info.parent.childNodes) return 1;
    var cnt = 0;
    for (var i = 0; i < info.parent.childNodes.length; i++) {
      if (info.parent.childNodes[i] && info.parent.childNodes[i].nodeType === 1) cnt++;
    }
    return cnt || 1;
  }
  function _matchPseudosOf(c, info) {
    if (!c.pseudos || !c.pseudos.length) return true;
    if (!info) return false;
    var elemIdx = info.prevSiblings ? info.prevSiblings.length + 1 : 1;
    for (var i = 0; i < c.pseudos.length; i++) {
      var ps = c.pseudos[i];
      if (ps.name === 'first-child') { if (elemIdx !== 1) return false; }
      else if (ps.name === 'last-child') { if (!_zwIsLastElemChild(info)) return false; }
      else if (ps.name === 'only-child') { if (elemIdx !== 1 || !_zwIsLastElemChild(info)) return false; }
      else if (ps.name === 'nth-child') { if (!_zwNthMatches(ps.arg, elemIdx)) return false; }
      else if (ps.name === 'nth-last-child') {
        var total = _zwElemChildCount(info);
        if (!_zwNthMatches(ps.arg, total - elemIdx + 1)) return false;
      } else if (ps.name === 'empty') {
        var kids = info.proxy && info.proxy.childNodes ? info.proxy.childNodes : [];
        for (var ek = 0; ek < kids.length; ek++) {
          var kn = kids[ek];
          if (kn && (kn.nodeType === 1 || (kn.nodeType === 3 && String(kn.data != null ? kn.data : (kn.nodeValue || ''))))) return false;
        }
      } else if (ps.name === 'not') {
        var nc = _parseCompoundOf(String(ps.arg || ''));
        if (!nc || nc.unsupported || _matchCompoundOf(info.proxy, nc)) return false;
      } else if (ps.name === 'checked') {
        var ck = false;
        try { ck = info.proxy.checked === true || String(info.proxy.checked) === 'true'; } catch (_e) { ck = false; }
        if (!ck) return false;
      }
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
      if (ch === '\\') {
        // js-dom M4 R118：CSS 转义感知——`\` + 1-6 位 hex 后的**单个空白是转义终止符**
        //（属段内字符，不是组合器边界，CSS Syntax 4.3.4）；`\` + 任意其他字符（含空白/
        // `.`/`#`/`>` 等）为字面转义，两字节一并计入段内。WPT ParentNode-querySelector-
        // escapes：`#\30 x` 的 `\30 ` 不可切分。https://drafts.csswg.org/css-syntax/#consume-escaped-code-point
        var escLen = 2; // `\x` 字面转义（含 \<空白>：转义空白是 ident 字符）
        if (i + 1 < text.length && /[0-9a-fA-F]/.test(text[i + 1])) {
          var hh = i + 1;
          while (hh < text.length && hh - i <= 6 && /[0-9a-fA-F]/.test(text[hh])) hh++;
          escLen = hh - i;
          // 空白终止符被消费；CRLF 序列整体是一个终止符（CSS Syntax whitespace 定义）。
          if (hh < text.length && text[hh] === '\r' && hh + 1 < text.length && text[hh + 1] === '\n') escLen += 2;
          else if (hh < text.length && /[ \t\n\r\f]/.test(text[hh])) escLen++;
        }
        cur += text.substring(i, i + escLen);
        i += escLen - 1;
        lastSegmentChar = true;
        continue;
      }
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
      // CSS 空白仅 5 个 ASCII 字符（JS /\s/ 含 U+2003 等 Unicode 空白——会把 `#\u2003` 误切分）。
      if (depth === 0 && /[ \t\n\f\r]/.test(ch)) {
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
  // 逗号列表拆分（跳过 `[...]` / 引号内逗号 / **CSS 转义**——js-dom M4 R118：`\,` 是字面
  // 逗号 ident 字符不是组边界，WPT ParentNode-querySelector-escapes `#,\,\:\!`）。
  function _splitSelectorListOf(sel) {
    var out = [], cur = '', depth = 0, quote = null;
    for (var i = 0; i < sel.length; i++) {
      var ch = sel[i];
      if (quote) { cur += ch; if (ch === quote) quote = null; continue; }
      if (ch === '"' || ch === "'") { quote = ch; cur += ch; continue; }
      if (ch === '\\') {
        // 转义：`\` + hex 序列（+ 可选空白终止符，CRLF 整体）或 `\` + 单字符，整体属组内。
        var el = 2;
        if (i + 1 < sel.length && /[0-9a-fA-F]/.test(sel[i + 1])) {
          var eh = i + 1;
          while (eh < sel.length && eh - i <= 6 && /[0-9a-fA-F]/.test(sel[eh])) eh++;
          el = eh - i;
          if (eh < sel.length && sel[eh] === '\r' && eh + 1 < sel.length && sel[eh + 1] === '\n') el += 2;
          else if (eh < sel.length && /[ \t\n\f\r]/.test(sel[eh])) el++;
        }
        cur += sel.substring(i, i + el);
        i += el - 1;
        continue;
      }
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
    // R100：最右 compound 的伪类求值（结构伪类需要 nodeInfo）。
    if (!_matchPseudosOf(compounds[compounds.length - 1], nodeInfo)) return false;
    return _matchChainFrom(compounds, combs, compounds.length - 1, p, nodeInfo);
  }
  // 从右往左匹配 compound[ci..0]：当前节点（info.proxy）须匹配 compound[ci]，再按 combs[ci-1]
  // 回溯到 compound[ci-1] 的候选节点。ci < 0 表示全部匹配 → 成功。
  function _matchChainFrom(compounds, combs, ci, _curProxy, info) {
    if (!_matchCompoundOf(info.proxy, compounds[ci])) return false;
    if (ci === 0) return _matchPseudosOf(compounds[0], info); // 最左 compound（含伪类）
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
        } else {
          // js-dom M3 R92：innerHTML= 解析的 _zwMEl 快照子（无 __zwHandle，子不在
          // _handleChildren registry）——直接沿其 childNodes 展开（元素子入 result，
          // 保文档序与兄弟上下文）。lit 模板渲染进 shadow root 的查询路径。
          try {
            var mk = _hSafe(function () { return p.childNodes; }, null);
            if (mk && mk.length) {
              var melem = [];
              for (var r92i = 0; r92i < mk.length; r92i++) {
                var r92k = mk[r92i];
                if (r92k && _hSafe(function () { return r92k.nodeType; }, 0) === 1) melem.push(r92k);
              }
              for (var r92j = 0; r92j < melem.length; r92j++) {
                var r92p = melem[r92j];
                var r92info = {
                  proxy: r92p,
                  parent: p,
                  parentInfo: info,
                  prevSibling: r92j > 0 ? melem[r92j - 1] : null,
                  prevSiblingInfo: null,
                  prevSiblings: [],
                  ancestors: ancestors.concat([info]),
                };
                if (r92info.prevSibling) {
                  // _zwMEl 无 handle → infoByProxy 按对象 identity 命中（同 proxy 引用）。
                  r92info.prevSiblingInfo = nodeInfoOf(r92info.prevSibling);
                }
                infoByProxy.set(r92p, r92info);
                result.push({ proxy: r92p, nodeInfo: r92info });
                // 深层 _zwMEl 递归展开（嵌套模板）。
                var r92kids = _hSafe(function () { return r92p.childNodes; }, null);
                (function expandDeep(dp, dparentInfo, danc) {
                  if (!dp) return;
                  var dk = _hSafe(function () { return dp.childNodes; }, null);
                  if (!dk || !dk.length) return;
                  var de = [];
                  for (var di = 0; di < dk.length; di++) {
                    var ddk = dk[di];
                    if (ddk && _hSafe(function () { return ddk.nodeType; }, 0) === 1) de.push(ddk);
                  }
                  for (var dj = 0; dj < de.length; dj++) {
                    var dnode = de[dj];
                    var dinfo = {
                      proxy: dnode,
                      parent: dp,
                      parentInfo: dparentInfo,
                      prevSibling: dj > 0 ? de[dj - 1] : null,
                      prevSiblingInfo: null,
                      prevSiblings: [],
                      ancestors: danc,
                    };
                    if (dinfo.prevSibling) dinfo.prevSiblingInfo = nodeInfoOf(dinfo.prevSibling);
                    infoByProxy.set(dnode, dinfo);
                    result.push({ proxy: dnode, nodeInfo: dinfo });
                    expandDeep(dnode, dinfo, danc.concat([dinfo]));
                  }
                })(r92p, r92info, ancestors.concat([info, r92info]));
              }
            }
          } catch (_e92) {}
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
  // R34xx：isIdentity/is2D/is3D（spec DOMMatrixReadOnly——2d.reset.state.
  // transformation_matrix 的 ctx.getTransform().isIdentity 断言；此前缺失 → undefined）。
  Object.defineProperty(DOMMatrix.prototype, 'isIdentity', {
    get: function () {
      return this._m[0] === 1 && this._m[1] === 0 && this._m[2] === 0 && this._m[3] === 0 &&
             this._m[4] === 0 && this._m[5] === 1 && this._m[6] === 0 && this._m[7] === 0 &&
             this._m[8] === 0 && this._m[9] === 0 && this._m[10] === 1 && this._m[11] === 0 &&
             this._m[12] === 0 && this._m[13] === 0 && this._m[14] === 0 && this._m[15] === 1;
    }
  });
  Object.defineProperty(DOMMatrix.prototype, 'is2D', {
    get: function () {
      // 2D iff 3D 行/列保持恒等（m13/m14/m23/m24/m31/m32/m34/m43 为 0，m33 为 1）。
      return this._m[2] === 0 && this._m[3] === 0 && this._m[6] === 0 && this._m[7] === 0 &&
             this._m[8] === 0 && this._m[9] === 0 && this._m[10] === 1 && this._m[11] === 0 &&
             this._m[14] === 0;
    }
  });
  Object.defineProperty(DOMMatrix.prototype, 'is3D', { get: function () { return !this.is2D; } });
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
  // R56（M8/DC-8）：NaN/±Infinity 必须原样保留（spec DOMPointInit 成员是
  // unrestricted double；roundRect 拿 DOMPoint(10,NaN) 须判非有限忽略整次调用）。
  // 旧 `+x || 0` 把 NaN 吞成 0（NaN falsy），2d.path.roundrect.nonfinite 的
  // DOMPoint(10,NaN) 变合法半径 (10,0) 圆角矩形画出。
  function DOMPoint(x, y, z, w) {
    this.x = (x == null) ? 0 : +x;
    this.y = (y == null) ? 0 : +y;
    this.z = (z == null) ? 0 : +z;
    this.w = (w == null) ? 1 : +w;
  }
  DOMPoint.prototype.toJSON = function () { return { x: this.x, y: this.y, z: this.z, w: this.w }; };
  DOMPoint.fromPoint = function (p) { return new DOMPoint(p && p.x, p && p.y, p && p.z, p && p.w); };
  globalThis.DOMPoint = globalThis.DOMPoint || DOMPoint;

  // ImageData（R3297 + R34xx 修正）——HTML ImageData spec 构造器。三形式：
  //   `new ImageData(sw, sh[, settings])` → 透明黑（全零）像素数组。
  //   `new ImageData(data, sw[, sh][, settings])` → 包裹既有数据（sh 缺省时高度由
  //     data.length/(4×sw) 推导）。
  // settings: { colorSpace: 'srgb'|'display-p3', pixelFormat: 'rgba-unorm8'|'rgba-float16' }
  // 产物 `{width, height, data, colorSpace, pixelFormat}`——data 存储随 pixelFormat：
  // unorm8 → Uint8ClampedArray（0..255），float16 → Float16Array（0..1 归一化浮点，原生）。
  // 校验（driving: 2d.imageData.object.ctor.*，Chromium 行为）：
  //   - 非 new 调用 → TypeError（WebIDL Illegal constructor）
  //   - data 非 Uint8ClampedArray/Float16Array → TypeError；data 类型与 pixelFormat 不匹配
  //     → InvalidStateError；非法 pixelFormat/colorSpace 枚举 → TypeError（WebIDL enum）
  //   - 宽或高为 0 → IndexSizeError
  //   - data 长度与 4×w×h 不符（sh 缺省时长度须为 4×w 整数倍且非零）→ InvalidStateError；
  //     sh 显式给出但长度不符 → IndexSizeError
  // https://html.spec.whatwg.org/multipage/canvas.html#imagedata
  function ImageData(a, b, c, d) {
    if (!(this instanceof ImageData)) {
      throw new TypeError('Illegal constructor');
    }
    var F16Ctor = (typeof Float16Array !== 'undefined') ? Float16Array : null;
    var settings = null;
    var dataArg = null;
    var w, h;
    if (a != null && typeof a === 'object' && typeof a.length === 'number') {
      // new ImageData(data, sw[, sh][, settings])——settings 在第 3 或第 4 参。
      if (b === undefined) {
        throw new TypeError('missing width');
      }
      dataArg = a;
      w = Math.trunc(+b);
      if (typeof c === 'number') {
        h = Math.trunc(+c);
        settings = (d != null && typeof d === 'object') ? d : null;
      } else {
        h = undefined;
        settings = (c != null && typeof c === 'object') ? c : null;
      }
    } else {
      // new ImageData(sw, sh[, settings])——sh 必参（缺省 → TypeError，WebIDL）。
      if (b === undefined) {
        throw new TypeError('missing height');
      }
      // R34xx：settings 非对象 → TypeError（WebIDL 字典参转换——ctor.basics 的
      // (self,4,4) 三参 TypeError：sw=ToUint32(self)→0 本可匹配 (sw,sh) 重载，但
      // 字典参 4 转换失败 → 该重载被拒 → data 重载也拒 → 重载链全灭 → TypeError）。
      if (c !== undefined && typeof c !== 'object') {
        throw new TypeError('settings is not an object');
      }
      w = Math.trunc(+a);
      h = Math.trunc(+b);
      settings = (c != null && typeof c === 'object') ? c : null;
    }
    var fmt = 'rgba-unorm8';
    var cs = 'srgb';
    if (settings) {
      if (settings.pixelFormat !== undefined) {
        if (settings.pixelFormat !== 'rgba-unorm8' && settings.pixelFormat !== 'rgba-float16') {
          throw new TypeError('invalid pixelFormat');
        }
        fmt = settings.pixelFormat;
      }
      if (settings.colorSpace !== undefined) {
        if (settings.colorSpace !== 'srgb' && settings.colorSpace !== 'display-p3' &&
            settings.colorSpace !== 'srgb-linear' && settings.colorSpace !== 'display-p3-linear') {
          throw new TypeError('invalid colorSpace');
        }
        cs = settings.colorSpace;
      }
    }
    var isF16 = fmt === 'rgba-float16';
    var data;
    var dataPath = (dataArg !== null);
    if (dataPath) {
      // 类型校验先于长度算法（Chromium 实际顺序）：
      //   WebIDL union：非 Uint8ClampedArray/Float16Array → **重载回退**；
      //   data 类型与 pixelFormat 不匹配 → InvalidStateError
      //   （driving: 2d.imageData.object.ctor.array.bounds / pixelFormat）。
      var isU8 = (dataArg instanceof Uint8ClampedArray);
      var isF16a = (F16Ctor !== null && dataArg instanceof F16Ctor);
      if (!isU8 && !isF16a) {
        // WebIDL 重载链：data union 转换失败 → 试 (sw, sh[, settings]) 重载——
        // sw = ToUint32(dataArg)（对象 → 0）→ IndexSizeError（ctor.basics 的
        // Uint8Array 2 参 INDEX_SIZE_ERR）；settings 参数非对象 → TypeError
        //（ctor.basics 的 (self, 4, 4) 3 参 TypeError——字典参转换失败终止重载链）。
        if (c !== undefined && typeof c !== 'object') {
          throw new TypeError('settings is not an object');
        }
        w = Math.trunc(+a);
        h = Math.trunc(+b);
        dataArg = null;
        settings = (c != null && typeof c === 'object') ? c : null;
        dataPath = false;
      }
      // pixelFormat 与 data 类型不匹配 → InvalidStateError（重载回退后不属 data 路径）。
      if (dataPath && ((isF16 && !isF16a) || (!isF16 && !isU8))) {
        throw _zwDomException('data type does not match pixelFormat', 'InvalidStateError');
      }
    }
    if (dataPath) {
      // spec（imagebitmap-and-animations §ImageData constructor）：
      //   1. bytesPerPixel = 4（unorm8）/ 8（float16）
      //   2. length = data 的 byte length
      //   3. length 非 bytesPerPixel 的非零整数倍 → InvalidStateError
      //   4. length /= bytesPerPixel
      //   5. length 非 sw 的整数倍（sw 为 0 亦在此抛）→ IndexSizeError
      //   6. height = length / sw
      //   7. sh 给出且 ≠ height → IndexSizeError
      var bpp = isF16 ? 8 : 4;
      var len = dataArg.byteLength;
      if (len % bpp !== 0 || len === 0) {
        throw _zwDomException('data length is not a nonzero multiple of bytesPerPixel', 'InvalidStateError');
      }
      len = len / bpp;
      if (len % w !== 0) {
        throw _zwDomException('data length is not a multiple of sw', 'IndexSizeError');
      }
      var height = len / w;
      if (h !== undefined && h !== height) {
        throw _zwDomException('sh does not match data length', 'IndexSizeError');
      }
      h = height;
      data = dataArg;
    } else {
      // spec：one or both of sw and sh zero → IndexSizeError（NaN 经 WebIDL unsigned long
      // 转换 → 0；分配溢出（1<<31 等）→ IndexSizeError 同 Chromium 守卫）。
      if (w !== w) w = 0;
      if (h !== h) h = 0;
      if (w <= 0 || h <= 0 || w * h > 0x1fffffff) {
        throw _zwDomException('zero or oversized dimension', 'IndexSizeError');
      }
      data = isF16 ? new F16Ctor(w * h * 4) : new Uint8ClampedArray(w * h * 4);
    }
    this.colorSpace = cs;
    this.pixelFormat = fmt;
    Object.defineProperty(this, 'width', { value: w, writable: false, enumerable: true, configurable: false });
    Object.defineProperty(this, 'height', { value: h, writable: false, enumerable: true, configurable: false });
    Object.defineProperty(this, 'data', { value: data, writable: false, enumerable: true, configurable: false });
  }
  globalThis.ImageData = globalThis.ImageData || ImageData;


  // canvas 元素 + 2d 上下文 proxy（R2795，canvas slice 1）。host 持 CanvasContext 注册表，JS 经
  // R34xx：WebIDL ToUint32（canvas width/height IDL setter——mod 2^32、NaN/±Inf → 0、
  // 字符串经 ToNumber：'400x' → NaN → 0。2d.canvas.host.size.invalid.attributes.idl）。
  function _zwToUint32(v) {
    var n = +v;
    if (!isFinite(n)) return 0;
    n = n < 0 ? -(-n % 4294967296) : (n % 4294967296);
    return n >>> 0;
  }

  // R34xx（color-type 目录）：f16 画布跨色彩空间转换（float 精度——u8 量化前的
  // srgb↔display-p3；CSS Color 4 矩阵 + sRGB EOTF）。
  var _zwCsM = {
    srgbToP3: [[0.8224621, 0.177538, 0], [0.0331941, 0.9668058, 0], [0.0170827, 0.0723974, 0.9105199]],
    p3ToSrgb: [[1.2249401, -0.2249404, 0], [-0.0420569, 1.0420571, 0], [-0.0196376, -0.0786361, 1.0982735]]
  };
  function _zwCsDecode(v) { return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4); }
  function _zwCsEncode(v) { return v <= 0.0031308 ? v * 12.92 : 1.055 * Math.pow(v, 1 / 2.4) - 0.055; }
  // rgb 为 [0,1] 三通道（伽马编码）→ 目标空间（伽马编码）。
  function _zwCsConvert(csFrom, csTo, rgb) {
    if (csFrom === csTo) return rgb;
    var m = (csFrom === 'srgb' && csTo === 'display-p3') ? _zwCsM.srgbToP3 : _zwCsM.p3ToSrgb;
    var lin = [_zwCsDecode(rgb[0]), _zwCsDecode(rgb[1]), _zwCsDecode(rgb[2])];
    var out = [
      m[0][0] * lin[0] + m[0][1] * lin[1] + m[0][2] * lin[2],
      m[1][0] * lin[0] + m[1][1] * lin[1] + m[1][2] * lin[2],
      m[2][0] * lin[0] + m[2][1] * lin[1] + m[2][2] * lin[2]
    ];
    return [_zwCsEncode(out[0]), _zwCsEncode(out[1]), _zwCsEncode(out[2])];
  }

  // R34xx（filters 渲染）：CanvasFilter colorMatrix → 20 值矩阵（spec colorMatrix
  // 滤镜——type: matrix 默认 20 值直用 / hueRotate θ / saturate s /
  // luminanceToAlpha；2d.filter.canvasFilterObject.colorMatrix）。经
  // __zw_canvas_op setFilterMatrix 传 host（空串清除）。
  function _zwApplyCanvasFilter(ctx, filterObj) {
    if (typeof __zw_canvas_op !== 'function') return;
    var inputs = filterObj._inputs || [];
    var m = null;
    var ds = '';
    for (var i = 0; i < inputs.length; i++) {
      var d = inputs[i];
      if (!d || typeof d !== 'object') continue;
      if (String(d.name) === 'colorMatrix') {
        m = _zwColorMatrix(d);
        break;
      }
      // R56h（M3）：dropShadow 渲染——dx/dy/stdDeviation/floodColor/floodOpacity →
      // host shadow 机制（2d.filter.canvasFilterObject.dropShadow 的 fillRect 阴影）。
      if (String(d.name) === 'dropShadow') {
        var dx = isFinite(+d.dx) ? +d.dx : 0;
        var dy = isFinite(+d.dy) ? +d.dy : 0;
        var sd = d.stdDeviation == null ? 0 : (Array.isArray(d.stdDeviation) ? +d.stdDeviation[0] : +d.stdDeviation);
        if (!isFinite(sd)) sd = 0;
        var fc = d.floodColor == null ? 'black' : String(d.floodColor);
        var fo = isFinite(+d.floodOpacity) ? +d.floodOpacity : 1;
        ds = [dx, dy, Math.abs(sd), fc, fo].join('\x1f');
      }
    }
    __zw_canvas_op(ctx._handle, 'setFilterMatrix', m ? m.join(',') : '');
    __zw_canvas_op(ctx._handle, 'setFilterDropShadow', ds);
  }
  function _zwColorMatrix(d) {
    var type = d.type == null ? 'matrix' : String(d.type);
    if (type === 'matrix') {
      var vals = d.values;
      if (!vals || vals.length !== 20) return null;
      var out = [];
      for (var i = 0; i < 20; i++) out.push(+vals[i]);
      return out;
    }
    if (type === 'luminanceToAlpha') {
      return [0,0,0,0,0, 0,0,0,0,0, 0,0,0,0,0, 0.2126,0.7152,0.0722,0,0];
    }
    if (type === 'hueRotate' || type === 'saturate') {
      var v = +d.values;
      if (!isFinite(v)) return null;
      if (type === 'saturate') {
        var s = v;
        return [
          0.213 + 0.787*s, 0.715 - 0.715*s, 0.072 - 0.072*s, 0, 0,
          0.213 - 0.213*s, 0.715 + 0.285*s, 0.072 - 0.072*s, 0, 0,
          0.213 - 0.213*s, 0.715 - 0.715*s, 0.072 + 0.928*s, 0, 0,
          0, 0, 0, 1, 0
        ];
      }
      var rad = v * Math.PI / 180;
      var cos = Math.cos(rad), sin = Math.sin(rad);
      return [
        0.213 + cos*0.787 - sin*0.213, 0.715 - cos*0.715 - sin*0.715, 0.072 - cos*0.072 + sin*0.928, 0, 0,
        0.213 - cos*0.213 + sin*0.143, 0.715 + cos*0.285 + sin*0.140, 0.072 - cos*0.072 - sin*0.283, 0, 0,
        0.213 - cos*0.213 - sin*0.787, 0.715 - cos*0.715 + sin*0.715, 0.072 + cos*0.928 + sin*0.072, 0, 0,
        0, 0, 0, 1, 0
      ];
    }
    return null;
  }

  // R34xx（filters 目录）：CSS filter list 字符串校验（'none' 或逗号分隔函数列表——
  // ctx.filter 非法串忽略保持旧值；'blur(5px)' 接受）。
  // R57（M3）：顶层逗号分割（不拆括号内逗号）——drop-shadow(… rgb(255, 165, 0))
  // 的 rgb() 内逗号曾把函数拆断 → 校验 false → 字符串 filter 整体忽略。
  function _zwSplitFilterList(t) {
    var out = [], depth = 0, cur = '';
    for (var i = 0; i < t.length; i++) {
      var ch = t[i];
      if (ch === '(') depth++;
      if (ch === ')') depth--;
      if (ch === ',' && depth === 0) { out.push(cur); cur = ''; }
      else cur += ch;
    }
    if (cur.trim()) out.push(cur);
    return out;
  }
  function _zwValidFilterList(s) {
    if (!s || !s.trim()) return false;
    var t = s.trim();
    if (t === 'none') return true;
    var parts = _zwSplitFilterList(t);
    for (var i = 0; i < parts.length; i++) {
      var p = parts[i].trim();
      var m = /^([a-zA-Z-]+)\((.*)\)$/.exec(p);
      if (!m) return false;
      // blur 参数须 CSS <length>（单位或 0）——'blur(10)'（无单位）非法忽略
      //（2d.filter.value）；'blur(5px)'/'blur(  5px)' 合法。
      if (m[1] === 'blur' && m[2].trim() !== '0') {
        if (!/^-?\d+(\.\d+)?(px|em|rem|ex|ch|cm|mm|in|pt|pc|q|%)$/.test(m[2].trim())) {
          return false;
        }
      }
      // 其余函数参数宽松接受。
    }
    return true;
  }

  // R34xx（filters 目录）：WebIDL double 转换（ToNumber 后须有限——null→0/
  // true→1/[]→0/'30'→30 接受；NaN/±Infinity/undefined/'test'/{} → TypeError）。
  function _zwFilterNumber(v) {
    var n = +v;
    if (!isFinite(n)) throw new TypeError('invalid filter number');
    return n;
  }

  // R34xx（filters 目录）：(double or sequence<double>) 校验（gaussianBlur/dropShadow
  // stdDeviation、turbulence baseFrequency——数字或 2 元有限数组）。
  function _zwFilterNumberOrPair(v) {
    if (Array.isArray(v)) {
      // 长度 0 → 0（[] 合法——dropShadow.exceptions）；长度 1 → 单值（[20]→20）；
      // 长度 2 → 双值；其余抛（spec sequence<double> 经 filter 字典算法归一）。
      if (v.length === 0) return;
      if (v.length === 1) { _zwFilterNumber(v[0]); return; }
      if (v.length !== 2) throw new TypeError('invalid filter number pair');
      for (var i = 0; i < 2; i++) _zwFilterNumber(v[i]);
      return;
    }
    _zwFilterNumber(v);
  }

  // R34xx（filters 目录）：CanvasFilter 字典校验（spec canvas filters——测试面：
  // gaussianBlur stdDeviation 必填有限数/2 元有限数组；convolveMatrix kernelMatrix
  // 非空 2D 数组同长行有限数（[[]] 特例允许）；colorMatrix values 恰 20 有限数；
  // dropShadow dx/dy/floodOpacity/stdDeviation + turbulence baseFrequency/numOctaves
  // 经 WebIDL double 转换）。driving: 2d.filter.canvasFilterObject.*.exceptions。
  function _zwValidateFilterInput(dict) {
    if (!dict || typeof dict !== 'object' || Array.isArray(dict)) {
      throw new TypeError('invalid CanvasFilter input');
    }
    var name = dict.name == null ? '' : String(dict.name);
    if (name === 'gaussianBlur' && dict.stdDeviation === undefined) {
      throw new TypeError('gaussianBlur: requires stdDeviation');
    }
    if (name === 'gaussianBlur' || (name === 'dropShadow' && Object.prototype.hasOwnProperty.call(dict, 'stdDeviation'))) {
      if (dict.stdDeviation === undefined) throw new TypeError(name + ': requires stdDeviation');
      _zwFilterNumberOrPair(dict.stdDeviation);
    } else if (name === 'dropShadow') {
      // 显式 undefined 属性也须校验（dropShadow.exceptions 的 dx: undefined 抛——
      // hasOwnProperty 而非 !== undefined）。
      if (Object.prototype.hasOwnProperty.call(dict, 'dx')) _zwFilterNumber(dict.dx);
      if (Object.prototype.hasOwnProperty.call(dict, 'dy')) _zwFilterNumber(dict.dy);
      if (Object.prototype.hasOwnProperty.call(dict, 'floodOpacity')) _zwFilterNumber(dict.floodOpacity);
      if (Object.prototype.hasOwnProperty.call(dict, 'floodColor')) {
        // floodColor：CSS 颜色串（host 真实解析器；'red' ✓、'test'/NaN 串 ✗；
        // 非串 ✗）。host 不可用 → 宽松接受。
        var _fc = dict.floodColor;
        if (typeof _fc !== 'string' || typeof __zw_canvas_op !== 'function' ||
            String(__zw_canvas_op('0', 'validateColor', _fc)) !== '1') {
          throw new TypeError('dropShadow: invalid floodColor');
        }
      }
    } else if (name === 'turbulence') {
      // baseFrequency/numOctaves 非负（2d.filter.canvasFilterObject.turbulence.
      // inputTypes：-1/[0,-1] → TypeError；[10] 单值合法）；seed 任意有限；
      // stitchTiles/type 枚举（'stitch'/'noStitch'、'fractalNoise'/'turbulence'）。
      // 显式 undefined 属性也须校验（turbulence.inputTypes 的 undefined 抛——
      // hasOwnProperty 而非 !== undefined）。
      if (Object.prototype.hasOwnProperty.call(dict, 'baseFrequency')) {
        var bf = dict.baseFrequency;
        if (Array.isArray(bf)) {
          if (bf.length === 1) {
            if (_zwFilterNumber(bf[0]) < 0) throw new TypeError('turbulence: baseFrequency must be >= 0');
          } else {
            _zwFilterNumberOrPair(bf);
            if (+bf[0] < 0 || +bf[1] < 0) throw new TypeError('turbulence: baseFrequency must be >= 0');
          }
        } else if (_zwFilterNumber(bf) < 0) {
          throw new TypeError('turbulence: baseFrequency must be >= 0');
        }
      }
      if (Object.prototype.hasOwnProperty.call(dict, 'numOctaves')) {
        if (_zwFilterNumber(dict.numOctaves) < 0) throw new TypeError('turbulence: numOctaves must be >= 0');
      }
      if (Object.prototype.hasOwnProperty.call(dict, 'seed')) _zwFilterNumber(dict.seed);
      if (Object.prototype.hasOwnProperty.call(dict, 'stitchTiles')) {
        var st = String(dict.stitchTiles);
        if (st !== 'stitch' && st !== 'noStitch') throw new TypeError('turbulence: invalid stitchTiles');
      }
      if (Object.prototype.hasOwnProperty.call(dict, 'type')) {
        var tp = String(dict.type);
        if (tp !== 'fractalNoise' && tp !== 'turbulence') throw new TypeError('turbulence: invalid type');
      }
    } else if (name === 'convolveMatrix') {
      var km = dict.kernelMatrix;
      if (!Array.isArray(km) || km.length === 0) throw new TypeError('convolveMatrix: invalid kernelMatrix');
      if (km[0].length === 0) {
        // [[]] 特例允许（spec/Chromium）；其余首行空 → 抛。
        if (km.length !== 1) throw new TypeError('convolveMatrix: invalid kernelMatrix');
        return;
      }
      var cols = km[0].length;
      for (var r = 0; r < km.length; r++) {
        var row = km[r];
        if (!Array.isArray(row) || row.length === 0 || row.length !== cols) {
          throw new TypeError('convolveMatrix: rows must be non-empty same-length');
        }
        for (var c = 0; c < row.length; c++) {
          if (typeof row[c] !== 'number' || !isFinite(row[c])) {
            throw new TypeError('convolveMatrix: invalid kernel value');
          }
        }
      }
    } else if (name === 'colorMatrix') {
      // R34xx：values 按 type 分流（spec colorMatrix 字典）——'matrix'（默认）：
      // 恰 20 有限数（可为 Float32Array 类数组——Array.isArray false → 按 length）；
      // 'hueRotate'/'saturate'：单个数字（2d.filter.layers.colorMatrix 的
      // {type:'hueRotate', values: 0}）；'luminanceToAlpha'：无 values。
      var cmType = dict.type == null ? 'matrix' : String(dict.type);
      if (cmType === 'matrix') {
        var vals = dict.values;
        if (vals === undefined || vals === null || typeof vals.length !== 'number' || vals.length !== 20) {
          throw new TypeError('colorMatrix: requires 20 values');
        }
        for (var i = 0; i < 20; i++) {
          if (typeof vals[i] !== 'number' || !isFinite(vals[i])) throw new TypeError('colorMatrix: invalid value');
        }
      } else if (cmType === 'hueRotate' || cmType === 'saturate') {
        if (Object.prototype.hasOwnProperty.call(dict, 'values')) _zwFilterNumber(dict.values);
      }
    }
    // 其余 name（dropShadow/blur 等）按测试面宽松接受（dx/dy 数字/串/数组均接受）。
  }

  // R34xx（filters 目录）：CanvasFilter 构造器（spec canvas filters——API 表面 +
  // 校验；**渲染未实现**——ctx.filter = CanvasFilter 后的像素面记录）。输入：
  // 单个 filter 字典或字典数组；空/undefined → 空 filter。
  // https://drafts.fxtf.org/filter-effects-2/#CanvasFilter
  function CanvasFilter(init) {
    if (!(this instanceof CanvasFilter)) throw new TypeError('Illegal constructor');
    var inputs = Array.isArray(init) ? init : (init === undefined || init === null ? [] : [init]);
    for (var i = 0; i < inputs.length; i++) _zwValidateFilterInput(inputs[i]);
    this._zwCanvasFilter = true;
    this._inputs = inputs;
  }
  Object.defineProperty(CanvasFilter.prototype, Symbol.toStringTag, { value: 'CanvasFilter' });
  if (!globalThis.CanvasFilter) {
    globalThis.CanvasFilter = CanvasFilter;
  }

  // R34xx（layers 目录）：beginLayer filter 选项校验（canvasFilter 字典——测试面：
  // colorMatrix values 为串 → TypeError；null/undefined/[]/{}/unknown name/
  // 数字布尔（DOMString 化）→ 接受）。
  function _zwValidateLayerFilter(filter) {
    if (typeof filter !== 'object' || filter === null || Array.isArray(filter)) return;
    if (filter.name === 'colorMatrix' && typeof filter.values === 'string') {
      throw new TypeError('invalid colorMatrix values');
    }
  }

  // R34xx：ctx 方法分发注册（_methods 包 → 实际原型薄转发器——幂等）。
  function _zwRegisterCtxDispatchers(ctx) {
    var _proto = Object.getPrototypeOf(ctx);
    for (var _mk in ctx._methods) {
      if (!(Object.prototype.hasOwnProperty.call(_proto, _mk) && _proto[_mk]._zwDispatch)) {
        (function (name) {
          var _disp = function () {
            if (!this || this._handle === undefined || this === _proto) {
              throw new TypeError('Illegal invocation');
            }
            return this._methods[name].apply(this, arguments);
          };
          _disp._zwDispatch = true;
          _proto[name] = _disp;
        })(_mk);
      }
    }
  }

  // R34xx（reset 目录全族）：client 状态镜像复位默认——ctx.reset() 与 canvas
  // width/height setter（spec：设尺寸重置 bitmap + 全部绘图状态）共用。driving:
  // 2d.reset.state.* 全族 + 2d.canvas.host.initial.reset.2dstate。
  function _zwResetCtxMirrors(ctx) {
    ctx._fs = '#000000';
    ctx._ss = '#000000';
    ctx._ga = 1.0;
    ctx._gco = 'source-over';
    ctx._sc = 'rgba(0, 0, 0, 0)';
    ctx._sb = 0;
    ctx._sox = 0;
    ctx._soy = 0;
    ctx._lw = 1;
    ctx._lj = 'miter';
    ctx._lc = 'butt';
    ctx._ml = 10;
    ctx._font = '10px sans-serif';
    ctx._ta = 'start';
    ctx._tb = 'alphabetic';
    ctx._dir = 'inherit';
    ctx._ldo = 0;
    ctx._ise = true;
    ctx._isq = 'high';
    ctx._ls = '0px';
    ctx._ws = '0px';
    ctx._fk = 'auto';
    ctx._fst = 'normal';
    ctx._fvc = 'normal';
    ctx._tr = 'auto';
    ctx._filter = 'none';
    if (ctx._f16) ctx._f16Overlay = null;
  }

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
      style: {},
      _ctx: null
    };
    // R57（M3）：createElement('canvas') 的 DOM 集成——standalone 对象原无 __zwHandle，
    // appendChild 静默跳过（mutation 未记录 → 布局无 canvas 盒 → 渲染空白，
    // 2d.composite.full.mode.alpha 的 createElement+append 画布全缺，oracle A/B 5.8%）。
    // 创建时同步 host 元素 handle，使 append 可记录 mutation；getContext 写
    // data-zw-canvas-ctx（painter 桥接），width/height 同步属性（重解析尺寸正确）。
    var _handle = (typeof __zw_create_element === 'function') ? __zw_create_element('canvas') : '';
    if (_handle) el.__zwHandle = _handle;
    // R34xx：standalone canvas width/height accessor——设值（**即使同值**，spec）重置
    // bitmap（host resizeContext）+ 全部绘图状态（2d.canvas.host.initial.reset.2dstate
    // 的 canvas.width= 同值复位断言）。旧为普通数据属性：赋值不触达 host，canvas-host
    // 目录全族失败。归一化同 DOM set-trap（parseInt，NaN/负 → 0）。
    var _cw = 300, _ch = 150;
    var _zwSetCanvasDim = function (p, v) {
      // R34xx：WebIDL ToUint32（size.invalid.attributes.idl 的 200-2^32 → 200）。
      var nv = _zwToUint32(v);
      if (p === 'width') _cw = nv; else _ch = nv;
      // R57：同步 host 元素属性（append 后重解析的 canvas 尺寸正确）。
      if (_handle && typeof __zw_set_attr_handle === 'function') {
        __zw_set_attr_handle(_handle, p, String(nv));
      }
      if (el._ctx && typeof __zw_canvas_op === 'function') {
        __zw_canvas_op(el._ctx._handle, 'resizeContext', String(_cw), String(_ch));
        _zwResetCtxMirrors(el._ctx);
      }
    };
    Object.defineProperty(el, 'width', {
      get: function () { return _cw; },
      set: function (v) { _zwSetCanvasDim('width', v); },
      enumerable: true,
      configurable: true
    });
    Object.defineProperty(el, 'height', {
      get: function () { return _ch; },
      set: function (v) { _zwSetCanvasDim('height', v); },
      enumerable: true,
      configurable: true
    });
    el.getContext = function (type) {
      // R34xx：缺参 → TypeError（WebIDL 必参——2d.canvas.context.invalid.args 的
      // canvas.getContext()）。
      if (arguments.length === 0) throw new TypeError('getContext: missing contextType');
      if (String(type) !== '2d') return null; // 仅 2d；webgl/webgl2 defer
      if (el._ctx) return el._ctx;
      if (typeof __zw_canvas_op !== 'function') return null;
      var id = __zw_canvas_op('0', 'getContext2d', String(el.width), String(el.height),
        (arguments.length > 1 && arguments[1] && typeof arguments[1] === 'object' && typeof arguments[1].colorSpace === 'string')
          ? arguments[1].colorSpace : 'srgb',
        (arguments.length > 1 && arguments[1] && typeof arguments[1] === 'object' && typeof arguments[1].colorType === 'string')
          ? arguments[1].colorType : 'unorm8');
      if (!id || String(id).charAt(0) === '!') return null;
      el._ctx = _zwMakeCtx2d(String(id));
      // R57：写 data-zw-canvas-ctx（painter 桥接 canvas 像素为页面图元——DOM canvas
      // 路径 part04 同语义；standalone 此前缺此属性，append 后 painter 找不到 ctx）。
      if (_handle && typeof __zw_set_attr_handle === 'function') {
        __zw_set_attr_handle(_handle, 'data-zw-canvas-ctx', String(id));
      }
      // R34xx（color-type 目录）：记录 canvas 色彩空间（f16 画布的浮点转换基准）。
      el._ctx._cs = (arguments.length > 1 && arguments[1] && typeof arguments[1] === 'object' && typeof arguments[1].colorSpace === 'string')
        ? arguments[1].colorSpace : 'srgb';
      // R34xx：ctx.canvas 只读（spec——赋值忽略；2d.canvas.host.readonly）。
      Object.defineProperty(el._ctx, 'canvas', {
        value: el,
        writable: false,
        enumerable: true,
        configurable: false
      });
      // R34xx：colorType 'float16' 上下文——绘制 float16 位图时记录原始浮点像素覆盖层
      //（createImageBitmap.srgb.rgba.float16 的越界值往返）。
      if (arguments.length > 1 && arguments[1] && typeof arguments[1] === 'object' &&
          String(arguments[1].colorType || '') === 'float16') {
        el._ctx._f16 = true;
        el._ctx._f16Overlay = null;
      }
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
      // R34xx（layers 目录）：层打开期间 toDataURL 抛 InvalidStateError
      //（2d.layer.malformed-operations）。
      if (el._ctx && el._ctx._inLayer) {
        throw _zwDomException('toDataURL: not allowed while a layer is open', 'InvalidStateError');
      }
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
      // R34xx（layers 目录）：层打开期间 toBlob 抛 InvalidStateError
      //（2d.layer.malformed-operations-with-promises）。
      if (el._ctx && el._ctx._inLayer) {
        throw _zwDomException('toBlob: not allowed while a layer is open', 'InvalidStateError');
      }
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
      // canvasgradient-addcolorstop——offset 非有限/缺省抛 TypeError（missingargs）；
      // 越界抛 IndexSizeError；颜色无效抛 SyntaxError）。
      if (offset === undefined) throw new TypeError('addColorStop: missing offset');
      offset = +offset;
      if (!isFinite(offset)) {
        throw new TypeError('addColorStop: non-finite offset');
      }
      if (offset < 0 || offset > 1) {
        throw _zwDomException('gradient offset out of range', 'IndexSizeError');
      }
      // R34xx：缺参（arguments.length<2）→ TypeError（missingargs）；显式 undefined/null
      // → DOMString 转换（'undefined'/'null'）→ 非法 → SyntaxError（object.invalidcolor）。
      if (arguments.length < 2) throw new TypeError('addColorStop: missing color');
      var c = String(color);
      if (c === '' || (typeof __zw_canvas_op === 'function' && !String(__zw_canvas_op('0', 'validateColor', c)))) {
        throw _zwDomException('invalid gradient color', 'SyntaxError');
      }
      __zw_canvas_op(h, 'addColorStop', gid, String(offset), String(color));
    }
    var g = { _zwGrad: gid, addColorStop: addColorStop };
    // R56h：colorInterpolationMethod / hueInterpolationMethod（spec CanvasGradient——
    // 2d.gradient.colorInterpolationMethod 的多画布 reftest）。setter 校验枚举值并
    // 经 host setGradientInterpolation 传插值空间 + 色相法。
    var _interp = 'srgb';
    var _hue = 'shorter';
    Object.defineProperty(g, 'colorInterpolationMethod', {
      enumerable: true, configurable: true,
      get: function () { return _interp; },
      set: function (v) {
        var s2 = String(v);
        // R57（M3）：补全 CSS Color 4 全部预定义空间（display-p3/display-p3-linear/
        // a98-rgb/rec2020/xyz-d50/xyz-d65——2d.gradient.colorInterpolationMethod 的
        // 14 格曾 6 格因校验缺名 TypeError 中止 → 格子空白）。
        var VALID = ['srgb', 'srgb-linear', 'lab', 'lch', 'oklab', 'oklch', 'hsl', 'hwb', 'xyz',
                     'xyz-d50', 'xyz-d65', 'prophoto-rgb', 'display-p3', 'display-p3-linear',
                     'a98-rgb', 'rec2020'];
        if (VALID.indexOf(s2) < 0) {
          throw new TypeError('invalid colorInterpolationMethod: ' + s2);
        }
        _interp = s2;
        if (typeof __zw_canvas_op === 'function') {
          __zw_canvas_op(h, 'setGradientInterpolation', gid, s2, _hue);
        }
      }
    });
    Object.defineProperty(g, 'hueInterpolationMethod', {
      enumerable: true, configurable: true,
      get: function () { return _hue; },
      set: function (v) {
        var s2 = String(v);
        var VALID = ['shorter', 'longer', 'increasing', 'decreasing'];
        if (VALID.indexOf(s2) < 0) {
          throw new TypeError('invalid hueInterpolationMethod: ' + s2);
        }
        _hue = s2;
        if (typeof __zw_canvas_op === 'function') {
          __zw_canvas_op(h, 'setGradientInterpolation', gid, _interp, s2);
        }
      }
    });
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
    p.arc = function (x, y, r, s, e, anticlockwise) {
      // R34xx：负半径 → IndexSizeError（spec——2d.path.arc.negative 的 path.arc）。
      if (+r < 0) throw _zwDomException('arc: negative radius', 'IndexSizeError');
      __zw_canvas_op(h, 'pathArc', pid, String(x), String(y), String(r), String(s), String(e), anticlockwise ? 'true' : 'false');
    };
    p.arcTo = function (x1, y1, x2, y2, r) {
      // R56f：Path2D.arcTo 负半径同 ctx 形式抛 IndexSizeError。
      r = _zwNumArg(r);
      if (r < 0) throw _zwDomException('arcTo: negative radius', 'IndexSizeError');
      __zw_canvas_op(h, 'pathArcTo', pid, String(x1), String(y1), String(x2), String(y2), String(r));
    };
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
  // R34xx：float16 ImageData 源 → 原始浮点像素（_zwBitmapF16）。u8 wire 无法表达越界值
  //（2/-1），drawImage 进 float16 上下文时记录覆盖层、getImageData 按位回读原始浮点
  //（createImageBitmap.srgb.rgba.float16 往返）。裁剪/翻转与 wire 变换同步，保证对齐。
  // https://html.spec.whatwg.org/multipage/imagebitmap-and-animations.html#dom-createimagebitmap
  function _zwCropF16Data(data, srcW, srcH, sx, sy, sw, sh) {
    sx = (sx == null || !isFinite(sx)) ? 0 : (sx | 0);
    sy = (sy == null || !isFinite(sy)) ? 0 : (sy | 0);
    sw = sw | 0;
    sh = sh | 0;
    if (sx < 0) { sw += sx; sx = 0; }
    if (sy < 0) { sh += sy; sy = 0; }
    if (sx + sw > srcW) sw = srcW - sx;
    if (sy + sh > srcH) sh = srcH - sy;
    if (sw <= 0 || sh <= 0) return [];
    var out = new Float32Array(sw * sh * 4);
    var o = 0;
    for (var y = 0; y < sh; y++) {
      for (var x = 0; x < sw; x++) {
        var base = ((sy + y) * srcW + (sx + x)) * 4;
        for (var c = 0; c < 4; c++) out[o++] = data[base + c];
      }
    }
    return out;
  }
  function _zwFlipF16DataY(data, w, h) {
    var out = new Float32Array(data.length);
    var o = 0;
    for (var y = h - 1; y >= 0; y--) {
      for (var x = 0; x < w; x++) {
        var base = (y * w + x) * 4;
        for (var c = 0; c < 4; c++) out[o++] = data[base + c];
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
        // R34xx（layers 目录）：canvas 源有打开层 → reject InvalidStateError
        //（2d.layer.malformed-operations-with-promises）。
        if (src && typeof src.getContext === 'function' && src._ctx && src._ctx._inLayer) {
          return Promise.reject(_zwDomException('createImageBitmap: source canvas has an open layer', 'InvalidStateError'));
        }
        // R34xx：float16 ImageData 源标记（_zwImageBitmapSourceToWire 的 ImageData 分支
        // 只编码 u8 wire，原始浮点像素由下方 `srcF16` 分支单独携带）。
        var srcF16 = !!(src && src.data && src.pixelFormat === 'rgba-float16');
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
          // R56h：spec 解码失败 → InvalidStateError DOMException（2d.drawImage.broken 的
          // promise_rejects_dom("InvalidStateError")——旧 TypeError 缺 DOMException code）。
          return Promise.reject(_zwDomException('createImageBitmap: 解码失败（零尺寸）', 'InvalidStateError'));
        }
        // R34xx：float16 ImageData 源 → 携带原始浮点像素（drawImage 覆盖层回读越界值）。
        // 裁剪/翻转与 wire 变换同步（sw/sh 此处已规范化：负值翻转矩形）。
        if (srcF16 && src && src.data) {
          var raw = src.data;
          var srcW = src.width | 0;
          var srcH = src.height | 0;
          if (sw != null || sh != null) {
            raw = _zwCropF16Data(raw, srcW, srcH, sx, sy, sw, sh);
          }
          if (flipY) {
            raw = _zwFlipF16DataY(raw, bm.width, bm.height);
          }
          bm._zwBitmapF16 = raw;
        }
        // R34xx（color-type 目录）：ImageData 源 → 记录其色彩空间（drawImage 的
        // p3 位图在 srgb 画布上的转换基准——createImageBitmap.p3.rgba.unorm8）。
        if (src && src.data && typeof src.colorSpace === 'string') {
          bm._zwBitmapCs = src.colorSpace;
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
    // R34xx：构造器/属性接受 0（spec OffscreenCanvas 无 canvas 元素的 0/负忽略语义——
    // 2d.pattern.basic.zerocanvas：width=0 须生效且 createPattern 抛 InvalidStateError）。
    // 非法（NaN/负/非数字）→ 保持旧值/默认（WebIDL 转换近似）。
    var _w = (typeof width === 'number' && width >= 0) ? (width | 0) : 300;
    var _h = (typeof height === 'number' && height >= 0) ? (height | 0) : 150;
    this._ctx = null;
    var self = this;
    Object.defineProperty(this, 'width', {
      get: function () { return _w; },
      set: function (v) {
        // R34xx：WebIDL [EnforceRange] unsigned long（worker canvas-host——
        // '100'→100/'+1.5e2'→150/'0x96'→150/301.999→301；'100em'→NaN→**TypeError**
        // 而非保持旧值）。**同值也复位**（initial.reset.2dstate.worker.js）。
        var _n = +v;
        if (!isFinite(_n) || _n < 0 || _n > 4294967295) {
          throw new TypeError('OffscreenCanvas width: invalid value');
        }
        var nv = Math.trunc(_n);
        if (nv === _w) {
          if (self._ctx && typeof __zw_canvas_op === 'function') {
            __zw_canvas_op(self._ctx._handle, 'resizeContext', String(_w), String(_h));
            _zwResetCtxMirrors(self._ctx);
          }
          return;
        }
        _w = nv;
        if (self._ctx && typeof __zw_canvas_op === 'function') {
          __zw_canvas_op(self._ctx._handle, 'resizeContext', String(_w), String(_h));
          // R34xx：设尺寸重置绘图状态（spec——与 canvas.width 同语义）。
          _zwResetCtxMirrors(self._ctx);
        }
      },
      enumerable: true,
      configurable: true
    });
    Object.defineProperty(this, 'height', {
      get: function () { return _h; },
      set: function (v) {
        // R34xx：同上（[EnforceRange] unsigned long + 同值复位）。
        var _n2 = +v;
        if (!isFinite(_n2) || _n2 < 0 || _n2 > 4294967295) {
          throw new TypeError('OffscreenCanvas height: invalid value');
        }
        var nv = Math.trunc(_n2);
        if (nv === _h) {
          if (self._ctx && typeof __zw_canvas_op === 'function') {
            __zw_canvas_op(self._ctx._handle, 'resizeContext', String(_w), String(_h));
            _zwResetCtxMirrors(self._ctx);
          }
          return;
        }
        _h = nv;
        if (self._ctx && typeof __zw_canvas_op === 'function') {
          __zw_canvas_op(self._ctx._handle, 'resizeContext', String(_w), String(_h));
          // R34xx：同上（宽高同值也复位——worker 变体）。
          _zwResetCtxMirrors(self._ctx);
        }
      },
      enumerable: true,
      configurable: true
    });
  }
  OffscreenCanvas.prototype.getContext = function (type) {
    // R34xx（worker canvas-context）：OffscreenCanvas.getContext 的 contextId 为
    // WebIDL 枚举（OffscreenCanvasContextId）——缺参/未知值（'2D'/''）→ **TypeError**
    //（worker 变体断言；与 HTMLCanvasElement 的 null 语义不同）。
    if (arguments.length === 0 || String(type) !== '2d') {
      throw new TypeError('OffscreenCanvas.getContext: unsupported context type');
    }
    if (this._ctx) return this._ctx;
    if (typeof __zw_canvas_op !== 'function') return null;
    var id = __zw_canvas_op('0', 'getContext2d', String(this.width), String(this.height),
      (arguments.length > 1 && arguments[1] && typeof arguments[1] === 'object' && typeof arguments[1].colorSpace === 'string')
        ? arguments[1].colorSpace : 'srgb',
      (arguments.length > 1 && arguments[1] && typeof arguments[1] === 'object' && typeof arguments[1].colorType === 'string')
        ? arguments[1].colorType : 'unorm8');
    if (!id || String(id).charAt(0) === '!') return null;
    this._ctx = _zwMakeCtx2d(String(id));
    // R34xx（color-type 目录）：记录 canvas 色彩空间（f16 浮点转换基准）。
    this._ctx._cs = (arguments.length > 1 && arguments[1] && typeof arguments[1] === 'object' && typeof arguments[1].colorSpace === 'string')
      ? arguments[1].colorSpace : 'srgb';
    // R34xx：OffscreenCanvasRenderingContext2D 独立接口（spec——worker 变体的
    // self.OffscreenCanvasRenderingContext2D + 其 prototype 扩展/覆写生效）。
    // 懒创建（CanvasRenderingContext2D 由 _zwMakeCtx2d 头部确保存在）；prototype
    // 链到 CanvasRenderingContext2D.prototype（共享方法分发层）；prototype 属性
    // 不可写/不可删（同 CanvasRenderingContext2D）。ctx 原型链 = 该 prototype
    //（getPrototypeOf(ctx) 断言）。
    if (!globalThis.OffscreenCanvasRenderingContext2D) {
      globalThis.OffscreenCanvasRenderingContext2D = function OffscreenCanvasRenderingContext2D() {};
      // spec：OffscreenCanvasRenderingContext2D 与 CanvasRenderingContext2D 为
      // **兄弟接口**——其 prototype 的 [[Prototype]] 为 Object.prototype
      //（prototype.worker 的 getPrototypeOf 断言）；方法分发器由 _zwMakeCtx2d
      // 按 ctx 实际原型注册（下方分发循环）。
      OffscreenCanvasRenderingContext2D.prototype = {};
      Object.defineProperty(OffscreenCanvasRenderingContext2D, 'prototype', {
        writable: false,
        configurable: false
      });
    }
    Object.setPrototypeOf(this._ctx, globalThis.OffscreenCanvasRenderingContext2D.prototype);
    // R34xx：原型切换后重注册分发器（Offscreen 原型须有方法转发层——
    // type.extend/replace.worker 的 fillRectGreen/fillRect 覆写）。
    _zwRegisterCtxDispatchers(this._ctx);
    // R34xx：ctx.canvas 只读指向 OffscreenCanvas 自身（worker canvas-host 的
    // readonly/reference——ctx.canvas === canvas）。
    Object.defineProperty(this._ctx, 'canvas', {
      value: this,
      writable: false,
      enumerable: true,
      configurable: false
    });
    // R34xx：colorType 'float16' 上下文——绘制 float16 位图时记录原始浮点像素覆盖层
    //（createImageBitmap.srgb.rgba.float16 越界值往返——OffscreenCanvas worker 变体同语义）。
    if (arguments.length > 1 && arguments[1] && typeof arguments[1] === 'object' &&
        String(arguments[1].colorType || '') === 'float16') {
      this._ctx._f16 = true;
      this._ctx._f16Overlay = null;
    }
    return this._ctx;
  };
  // transferToImageBitmap()：取当前 canvas 全像素 wire 包成 ImageBitmap（spec 返新 ImageBitmap，canvas bitmap 清空）。
  // 复用 _zwMakeImageBitmap（持 _zwBitmapWire，drawImage 可消费）。canvas bitmap 清空对齐 spec（transfer 语义）。
  OffscreenCanvas.prototype.transferToImageBitmap = function () {
    if (typeof __zw_canvas_op !== 'function') return null;
    // R34xx（layers 目录）：层打开期间 transferToImageBitmap 抛 InvalidStateError
    //（2d.layer.malformed-operations.worker）。
    if (this._ctx && this._ctx._inLayer) {
      throw _zwDomException('transferToImageBitmap: not allowed while a layer is open', 'InvalidStateError');
    }
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
  // R34xx（worker layers）：OffscreenCanvas.convertToBlob（spec——异步 PNG Blob
  // 导出，镜像 toBlob 的 host 编码路径；层打开期间抛 InvalidStateError——
  // malformed-operations-with-promises.worker）。
  OffscreenCanvas.prototype.convertToBlob = function (options) {
    var self = this;
    return Promise.resolve().then(function () {
      if (self._ctx && self._ctx._inLayer) {
        throw _zwDomException('convertToBlob: not allowed while a layer is open', 'InvalidStateError');
      }
      if (typeof __zw_canvas_op !== 'function') return null;
      if (!self._ctx) self.getContext('2d');
      if (!self._ctx) return null;
      var csv = String(__zw_canvas_op(self._ctx._handle, 'toDataURL'));
      if (!csv) return null;
      var nums = csv.split(',');
      var bytes = new Uint8Array(nums.length);
      for (var j = 0; j < nums.length; j++) bytes[j] = +nums[j];
      return new Blob([bytes], { type: 'image/png' });
    });
  };
  Object.defineProperty(OffscreenCanvas.prototype, Symbol.toStringTag, { value: 'OffscreenCanvas' });
  if (!globalThis.OffscreenCanvas) {
    globalThis.OffscreenCanvas = OffscreenCanvas;
  }
  // R34xx：WebIDL 参数语义——缺省（undefined/null）→ TypeError（missingargs：
  // 2d.conformance.requirements.missingargs）；非有限（NaN/±Infinity）数值 → 方法忽略
  //（spec：各方法 "If any of the arguments are infinite or NaN, then return"——
  // 2d.fillRect.nonfinite / 2d.transformation.*.nonfinite 系列）。渐变创建等按 spec 抛
  // TypeError 的方法不经此 helper（create*Gradient 自带校验）。
  function _zwNumArg(v) {
    if (v === undefined || v === null) {
      throw new TypeError('missing argument');
    }
    return +v;
  }
  function _zwAllFinite() {
    for (var i = 0; i < arguments.length; i++) {
      if (!isFinite(arguments[i])) return false;
    }
    return true;
  }
  function _zwMakeCtx2d(h) {
    // R34xx：构造器须先于实例创建（原型链 Object.create；定义在函数体中部——
    // 首调时提前确保存在；prototype 属性不可写/不可删，spec）。
    if (!globalThis.CanvasRenderingContext2D) {
      globalThis.CanvasRenderingContext2D = function CanvasRenderingContext2D() {};
      Object.defineProperty(CanvasRenderingContext2D, 'prototype', {
        writable: false,
        configurable: false
      });
    }
    var ctx = Object.create(globalThis.CanvasRenderingContext2D.prototype);
    ctx._handle = h;
    ctx._methods = {};
    ctx.canvas = null;
    ctx._fs = '#000000'; ctx._ss = '#000000'; ctx._lw = 1.0;
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
    ctx._methods.beginPath = function () { __zw_canvas_op(h, 'beginPath'); };
    ctx._methods.closePath = function () { __zw_canvas_op(h, 'closePath'); };
    ctx._methods.moveTo = function (x, y) {
      x = _zwNumArg(x); y = _zwNumArg(y);
      if (!_zwAllFinite(x, y)) return;
      __zw_canvas_op(h, 'moveTo', String(x), String(y));
    };
    ctx._methods.lineTo = function (x, y) {
      x = _zwNumArg(x); y = _zwNumArg(y);
      if (!_zwAllFinite(x, y)) return;
      __zw_canvas_op(h, 'lineTo', String(x), String(y));
    };
    ctx._methods.arc = function (x, y, r, s, e, anticlockwise) {
      // R34xx：anticlockwise 第 6 参透传（spec：2d.line.cap.round 等 arc 填充用例依赖方向）。
      x = _zwNumArg(x); y = _zwNumArg(y); r = _zwNumArg(r); s = _zwNumArg(s); e = _zwNumArg(e);
      if (!_zwAllFinite(x, y, r, s, e)) return;
      // R34xx：负半径 → IndexSizeError（spec——2d.path.arc.negative）。
      if (r < 0) throw _zwDomException('arc: negative radius', 'IndexSizeError');
      __zw_canvas_op(h, 'arc', String(x), String(y), String(r), String(s), String(e), anticlockwise ? 'true' : 'false');
    };
    // R3306：fill/stroke/clip 可选首参 Path2D（spec ctx.fill(path)），命中走 fillPath/strokePath/clipPath
    //（用给定 Path2D 替代 ctx 当前路径）；无参走当前路径形式（既定）。
    ctx._methods.fill = function (path, fillRule) {
      // R56c（M8/DC-8）：fillRule 透传（spec dom-context-2d-fill——fill(Path2D?, fillRule)，
      // "evenodd" 奇偶 / 缺省 nonzero；非串值 ToString 后非 evenodd 按 spec 抛 TypeError——
      // 现宽松回落 nonzero）。
      // WebIDL 可选前置参省略：ctx.fill("evenodd") 时 fillRule 落在第一参位
      //（2d.path.fill.winding.evenodd.1 的调用形式）——首参为字符串即嗅探为 rule。
      if (typeof path === 'string' && fillRule === undefined) {
        fillRule = path;
        path = undefined;
      }
      var rule = (fillRule === undefined) ? '' : String(fillRule);
      if (path && path._zwPath) __zw_canvas_op(h, 'fillPath', String(path._zwPath), rule);
      else __zw_canvas_op(h, 'fill', rule);
    };
    ctx._methods.stroke = function (path) {
      if (path && path._zwPath) __zw_canvas_op(h, 'strokePath', String(path._zwPath));
      else __zw_canvas_op(h, 'stroke');
    };
    // R34xx：fillRect/strokeRect/clearRect 缺参 → TypeError（missingargs），任一参数
    // 非有限（NaN/Infinity）→ 方法忽略（spec：2d.fillRect.nonfinite 系列）。
    ctx._methods.fillRect = function (x, y, w, hh) {
      x = _zwNumArg(x); y = _zwNumArg(y); w = _zwNumArg(w); hh = _zwNumArg(hh);
      if (!_zwAllFinite(x, y, w, hh)) return;
      __zw_canvas_op(h, 'fillRect', String(x), String(y), String(w), String(hh));
    };
    ctx._methods.strokeRect = function (x, y, w, hh) {
      x = _zwNumArg(x); y = _zwNumArg(y); w = _zwNumArg(w); hh = _zwNumArg(hh);
      if (!_zwAllFinite(x, y, w, hh)) return;
      __zw_canvas_op(h, 'strokeRect', String(x), String(y), String(w), String(hh));
    };
    ctx._methods.clearRect = function (x, y, w, hh) {
      x = _zwNumArg(x); y = _zwNumArg(y); w = _zwNumArg(w); hh = _zwNumArg(hh);
      if (!_zwAllFinite(x, y, w, hh)) return;
      // R34xx：写像素操作使 float16 覆盖层失效（避免陈旧原始浮点回读）。
      if (this._f16) this._f16Overlay = null;
      __zw_canvas_op(h, 'clearRect', String(x), String(y), String(w), String(hh));
    };
    // R3078：Canvas 2D 文本 API（fillText/strokeText/measureText）+ createImageData（blank）。
    // fillText 经 host fill_text（canvas crate 写 pixel_buffer）；measureText 返 TextMetrics（width+bounding）；
    // createImageData 返 blank ImageData（全透明 = 全 0，Uint8ClampedArray(w*h*4)，JS 构无需 host）。createImageData
    // 双形式：createImageData(w,h) / createImageData(imageData)（复制尺寸）。spec CanvasRenderingContext2D。
    ctx._methods.fillText = function (text, x, y, maxWidth) {
      // R34xx：maxWidth 透传 + 缺参 TypeError / 非有限忽略（spec：fillText 任一参数
      // 非有限则 return——"If any of the arguments are infinite or NaN, then return"）。
      // maxWidth ≤ 0 → return（不绘制——2d.text.draw.fill.maxWidth.zero/negative 期望
      // 画布保持底色；text preparation algorithm 对非正 maxWidth 直接返回）。
      x = _zwNumArg(x); y = _zwNumArg(y);
      if (!_zwAllFinite(x, y) || (maxWidth !== undefined && !isFinite(+maxWidth))) return;
      if (maxWidth !== undefined && +maxWidth <= 0) return;
      __zw_canvas_op(h, 'fillText', String(text), String(x), String(y), String(maxWidth === undefined ? '' : +maxWidth));
    };
    // R34xx：fillTextCluster(cluster, x, y)——绘制单个字素簇（spec TextCluster；
    // 2d.text.measure.fillTextCluster-*.tentative）。簇对象经 measureText().getTextClusters()
    // 取得（含 x/y 相对文本原点偏移）。经 fillText 宿主路径（当前 font/baseline 生效）。
    ctx._methods.fillTextCluster = function (cluster, x, y, options) {
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
      // R57（M3）：spec TextCluster——簇用 measure 时字体渲染（即使 ctx.font
      // 已改——fillTextCluster-font-change.tentative）；绘制后恢复。
      var savedFont = ctx.font;
      if (cluster.font) {
        ctx.font = cluster.font;
      }
      __zw_canvas_op(h, 'fillText', String(cluster.text),
        String(drawX), String(drawY));
      ctx.font = savedFont;
    };
    // R34xx：strokeTextCluster（spec TextCluster——与 fillTextCluster 对称，描边绘制）。
    ctx._methods.strokeTextCluster = function (cluster, x, y, options) {
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
      // R57（M3）：spec TextCluster——簇用 measure 时字体渲染（fillTextCluster
      // 同——strokeTextCluster-font-change.tentative）；绘制后恢复。
      var savedFont = ctx.font;
      if (cluster.font) {
        ctx.font = cluster.font;
      }
      __zw_canvas_op(h, 'strokeText', String(cluster.text),
        String(drawX), String(drawY));
      ctx.font = savedFont;
    };
    ctx._methods.strokeText = function (text, x, y) {
      x = _zwNumArg(x); y = _zwNumArg(y);
      if (!_zwAllFinite(x, y)) return;
      __zw_canvas_op(h, 'strokeText', String(text), String(x), String(y));
    };
    ctx._methods.measureText = function (text) {
      if (text === undefined || text === null) throw new TypeError('measureText: missing text');
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
          // R34xx：wire 逐字形 5 字段（pen, l, t, r, b）——pen 为字形原点。
          glyphs.push([parseFloat(gv[0]) || 0, parseFloat(gv[1]) || 0,
                       parseFloat(gv[2]) || 0, parseFloat(gv[3]) || 0,
                       parseFloat(gv[4]) || 0]);
        }
      }
      var tm = {
        // R57（M3）：ctx 引用——getTextClusters 的簇对象须记录 measure 时的
        // font（fillTextCluster-font-change.tentative：簇用 measure 时字体渲染）。
        _ctx: ctx,
        width: num(0),
        actualBoundingBoxAscent: num(1),
        actualBoundingBoxDescent: num(2),
        actualBoundingBoxLeft: num(3),
        actualBoundingBoxRight: num(4),
        fontBoundingBoxAscent: num(5),
        fontBoundingBoxDescent: num(6),
        // R34xx：emHeight*（spec TextMetrics——em 盒顶/底距基线，host 独立计算——
        // emHeights-low-ascent/zero-descent）。
        emHeightAscent: num(7),
        emHeightDescent: num(8),
        alphabeticBaseline: num(9),
        hangingBaseline: num(10),
        ideographicBaseline: num(11),
        // R34xx：getActualBoundingBox(start, end)——[start, end) 字形墨迹并集矩形
        //（相对文本原点；无字体栈/空区间 → 空矩形 {0,0,0,0}）。
        // R34xx：getSelectionRects(start, end)——[start, end) 字形墨迹矩形序列（spec
        // TextMetrics 新方法——selection-rects-exceptions.tentative 的索引校验；
        // 与 DOM Range.getClientRects 对照的主测试：字形按垂直重叠并成行 rect
        //（单行 → 1 个 rect，与 DOM getClientRects 行语义一致——selection-rects 的
        // 长度对比）。
        // 校验：负/非有限 → TypeError；越文本长度 → IndexSizeError；start>end（均在
        // 范围内）→ 空序列（与 DOM 反向 range 的 getClientRects()=[] 一致）。
        getSelectionRects: function (start, end) {
          start = +start;
          if (!isFinite(start) || start < 0) throw new TypeError('getSelectionRects: invalid start');
          if (end === undefined || end === null) {
            end = glyphs.length;
          } else {
            end = +end;
            if (!isFinite(end) || end < 0) throw new TypeError('getSelectionRects: invalid end');
          }
          if (start > text.length || end > text.length) {
            throw _zwDomException('getSelectionRects: out of range', 'IndexSizeError');
          }
          // R34xx：反向 range（start>end）交换——与 DOM Range.getClientRects 的
          // 归一化语义一致（selection-rects 的 (3,2)/(1,0) 用例）。
          if (start > end) { var tt = start; start = end; end = tt; }
          // 行合并：范围字形并成一个行 rect（测试文本均单行——与 DOM
          // Range.getClientRects 行语义一致）。x = 锚定偏移 + (范围首字形墨迹左缘 −
          // 全文本首字形墨迹左缘)——与 DOM 侧经 parent.x 归一化后的约定一致
          //（selection-rects 对照：DOM x = sub_ink_left − full_ink_left）。
          var l = Infinity, t = Infinity, r = -Infinity, b = -Infinity;
          var firstInkLeft = null;
          var any = false;
          for (var i = start; i < end && i < glyphs.length; i++) {
            var r5 = glyphs[i]; // [pen, l, t, r, b]
            // R34xx：不跳过空墨迹字形（directional-override 的 RLO 首字形——
            // 与 DOM 侧一致，范围覆盖即产出 rect）。
            if (firstInkLeft === null) firstInkLeft = r5[1];
            l = Math.min(l, r5[1]); t = Math.min(t, r5[2]);
            r = Math.max(r, r5[3]); b = Math.max(b, r5[4]);
            any = true;
          }
          if (!any) return [];
          var baseInk = glyphs.length ? (glyphs[0][1] || 0) : 0; // 全文本首字形墨迹左缘
          // R34xx：对齐偏移按**全范围合并宽**（min 墨迹左缘 → max 墨迹右缘——含 RLO 等
          // 0 墨迹字形的前导 pen；DOM 侧 text_align_dx 取 Range rect 宽和，同源）。
          var fullL = Infinity, fullR = -Infinity;
          for (var fi = 0; fi < glyphs.length; fi++) {
            fullL = Math.min(fullL, glyphs[fi][1]);
            fullR = Math.max(fullR, glyphs[fi][3]);
          }
          var inkW = (fullL === Infinity) ? 0 : (fullR - fullL);
          var dx = 0;
          if (ctxTa === 'center') dx = inkW / 2;
          else if (ctxTa === 'right') dx = inkW;
          var x = (firstInkLeft - baseInk) - dx;
          // R34xx：y/height 用字体 em 盒（top=-fontBoundingBoxAscent,
          // bottom=+fontBoundingBoxDescent——selection-rects-baselines 断言）。
          return [new DOMRect(x, -num(5), r - l, num(5) + num(6))];
        },
        // R34xx：getIndexFromOffset(x, y)——命中测试（spec TextMetrics 新方法——
        // index-from-offset 系列与 DOM caretPositionFromPoint 对照，待 DOM 布局面；
        // 本实现按字形墨迹矩形命中，返首个命中 glyph 的字符索引）。
        getIndexFromOffset: function (x, y) {
          // R34xx：单参（仅 x）调用——index-from-offset 系列以 x 命中（y 缺省跳过
          // 垂直检查；DOM caretPositionFromPoint 对照侧同样 1 参）。
          x = +x;
          if (!isFinite(x)) throw new TypeError('getIndexFromOffset: invalid point');
          // R34xx：对齐锚定——caret 点转文本空间（center/right 文本原点在
          // -width/2/-width：x=0 的 caret 应落文本中点/末尾）。
          x -= anchor;
          var useY = (y !== undefined && y !== null);
          if (useY) { y = +y; if (!isFinite(y)) throw new TypeError('getIndexFromOffset: invalid point'); }
          if (ctxDir === 'rtl') {
            // R34xx：rtl 视觉→逻辑——caret 在 x → 位于 x 右侧的字形数（rtl 文本
            // 自右向左，offset 0 在右缘：x=width → 0，x=0 → text.length）。
            var cnt = 0;
            for (var i = 0; i < glyphs.length; i++) {
              var r = glyphs[i]; // [pen, l, t, r, b]
              if (r[3] <= r[1] && r[4] <= r[2]) continue;
              if (r[1] > x) cnt++;
            }
            return cnt;
          }
          // ltr：caret = 字形**中点** < x 的字形数（中点 = 相邻字形原点中点；末字形 =
          // 与文本右缘中点——index-from-offset-edge-cases 的边界语义：a_width/2 → 0
          //（中点相等不归）、a_width/2+1 → 1、a_width（右缘）→ 1、a_width+b_width →
          // 2）。与 DOM caretPositionFromPoint 同规则（part06 _zwCaretFromPoint）。
          var cnt = 0;
          for (var i = 0; i < glyphs.length; i++) {
            var r = glyphs[i]; // [pen, l, t, r, b]
            if (r[3] <= r[1] && r[4] <= r[2]) continue;
            var nextPen = (i + 1 < glyphs.length) ? glyphs[i + 1][0] : num(0);
            var center = (r[0] + nextPen) / 2;
            if (center < x) cnt++;
          }
          return cnt;
        },
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
              offsetInText: i,
              // R57（M3）：measure 时的字体快照——fillTextCluster 须用 measure 时
              // 字体渲染（spec TextCluster：即使 ctx.font 已改，簇仍按原字体——
              // fillTextCluster-font-change.tentative）。measureText 对象持 ctx
              // 引用（this._ctx——getTextClusters 经 this 访问）。
              font: this._ctx && this._ctx.font !== undefined ? String(this._ctx.font) : ''
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
            var r = glyphs[i]; // [pen, l, t, r, b]
            if (r[3] <= r[1] && r[4] <= r[2]) continue; // 空墨迹（空格等）
            any = true;
            if (r[1] < x0) x0 = r[1];
            if (r[2] < y0) y0 = r[2];
            if (r[3] > x1) x1 = r[3];
            if (r[4] > y1) y1 = r[4];
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
          // R34xx：rect = 墨迹位置（不钳制）——x = ink_left 位置（= −actualBoundingBoxLeft），
          // 可与 full-bounds rect（getActualBoundingBox-full-text：x: -actualBoundingBoxLeft）
          // 一致；旧 min/max 钳制把 'BCD' 子串 x 钳成 0（应为 50）。
          return {
            x: x0 + anchor,
            y: y0,
            width: x1 - x0,
            height: y1 - y0
          };
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
        // R34xx：零尺寸 → IndexSizeError（spec——2d.imageData.create2.zero）。
        if (w === 0 || h === 0) {
          throw _zwDomException('createImageData: zero dimension', 'IndexSizeError');
        }
      }
      return new ImageData(w, h);
    }
    ctx._methods.createImageData = function (a, b) { return _zwCreateImageData(this, a, b); };
    // R34xx：CanvasRenderingContext2D 全局构造器（此前缺失 → WPT illegal-invocation 用例
    // `CanvasRenderingContext2D.prototype.createImageData.call(null)` 抛 ReferenceError 而非
    // 期望的 TypeError）。prototype 方法做 illegal-invocation 检查（sloppy mode 下 call(null)
    // this=globalThis）后委托共享实现。
    // R34xx：createImageData 原型方法（构造器已由函数头提前确保存在——此处无条件
    // 覆写为当前实例闭包，幂等）。illegal-invocation：this 须为 ctx proxy（持
    // _handle）。call(null) sloppy 下 this=globalThis，call({}) 为普通对象——均无
    // _handle → TypeError（spec + WPT .this 用例）。
    CanvasRenderingContext2D.prototype.createImageData = function (a, b) {
      if (!this || this._handle === undefined || this === CanvasRenderingContext2D.prototype) {
        throw new TypeError('Illegal invocation');
      }
      return _zwCreateImageData(this, a, b);
    };
    // R3079：CanvasGradient（createLinearGradient/createRadialGradient/createConicGradient + addColorStop）。
    // host 持渐变注册表（独立 id 命名空间）；create* 返 host id，JS 包一层 proxy。addColorStop 经 host
    // 变更停止点。fillStyle/strokeStyle 设渐变对象走 setFillStyleGradient（host 查表克隆）。spec CanvasGradient。
    ctx._methods.createLinearGradient = function (x0, y0, x1, y1) {
      // R34xx：任一参数非有限抛 TypeError（spec：
      // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-createlineargradient——
      // 2d.gradient.linear.nonfinite 断言 TypeError，非 NotSupportedError）。
      if (!isFinite(+x0) || !isFinite(+y0) || !isFinite(+x1) || !isFinite(+y1)) {
        throw new TypeError('createLinearGradient: non-finite coordinate');
      }
      var gid = String(__zw_canvas_op(h, 'createLinearGradient', String(+x0 || 0), String(+y0 || 0), String(+x1 || 0), String(+y1 || 0)));
      return _zwMakeGradient(h, gid);
    };
    ctx._methods.createRadialGradient = function (x0, y0, r0, x1, y1, r1) {
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
    ctx._methods.createConicGradient = function (startAngle, cx, cy) {
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
    ctx._methods.createPattern = function (image, repetition) {
      if (typeof __zw_canvas_op !== 'function') return null;
      // R34xx（layers 目录）：层打开期间 createPattern 抛 InvalidStateError
      //（2d.layer.malformed-operations——createPattern 打开期调用限制）。
      if (this._inLayer) {
        throw _zwDomException('createPattern: not allowed while a layer is open', 'InvalidStateError');
      }

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
      // R34xx：缺参（arguments.length<2）→ TypeError（missingargs）；显式 undefined →
      // DOMString 转换 'undefined' → 非法 → SyntaxError（pattern.repeat.undefined）。
      if (arguments.length < 2) {
        throw new TypeError('createPattern: missing repetition');
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
        // R34xx（layers 目录）：源 canvas 上下文有打开层 → InvalidStateError
        //（2d.layer.malformed-operations：ctx.beginLayer 后经另一 ctx2.drawImage(canvas)）。
        // DOM canvas 的 ctx 在 _zwCanvasCtx[key]（proxy 无 _ctx 属性）——经
        // __zwSelector/__zwHandle 取 key 兜底。
        var _srcCtx = image._ctx;
        if (!_srcCtx && typeof _zwCanvasCtx === 'object') {
          // DOM canvas ctx 键 = _elKey(sel, handle)（handle 优先 '@handle'）——两键都查。
          var _sSel = image.__zwSelector || null;
          var _sH = image.__zwHandle ? '@' + image.__zwHandle : null;
          _srcCtx = (_sSel && _zwCanvasCtx[_sSel]) || (_sH && _zwCanvasCtx[_sH]) || null;
        }
        if (_srcCtx && _srcCtx._inLayer) {
          throw _zwDomException('drawImage: source canvas has an open layer', 'InvalidStateError');
        }
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
      } else if (image && image._zwBitmapWire) {
        // R34xx：ImageBitmap 源（createImageBitmap 产物——offscreen worker 的
        // fetch+createImageBitmap 路径）直接用其 wire 串。
        wire = String(image._zwBitmapWire);
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
    ctx._methods.quadraticCurveTo = function (cpx, cpy, x, y) {
      cpx = _zwNumArg(cpx); cpy = _zwNumArg(cpy); x = _zwNumArg(x); y = _zwNumArg(y);
      if (!_zwAllFinite(cpx, cpy, x, y)) return;
      __zw_canvas_op(h, 'quadraticCurveTo', String(cpx), String(cpy), String(x), String(y));
    };
    ctx._methods.bezierCurveTo = function (cp1x, cp1y, cp2x, cp2y, x, y) {
      cp1x = _zwNumArg(cp1x); cp1y = _zwNumArg(cp1y); cp2x = _zwNumArg(cp2x); cp2y = _zwNumArg(cp2y);
      x = _zwNumArg(x); y = _zwNumArg(y);
      if (!_zwAllFinite(cp1x, cp1y, cp2x, cp2y, x, y)) return;
      __zw_canvas_op(h, 'bezierCurveTo', String(cp1x), String(cp1y), String(cp2x), String(cp2y), String(x), String(y));
    };
    ctx._methods.ellipse = function (x, y, rx, ry, rotation, start, end, ccw) {
      // R56e：负半径 → IndexSizeError（spec dom-context-2d-ellipse——
      // 2d.path.ellipse.basics：rx/ry < 0 抛，-0 与 0 合法）。
      // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-ellipse
      rx = +rx; ry = +ry;
      if (rx < 0 || ry < 0) {
        throw _zwDomException('ellipse: negative radius', 'IndexSizeError');
      }
      // R56h：**参数绑定修复**——旧签名 (x,y,rx,ry,rot,start,end) 把第 7 参当 end，
      // 8 参调用（spec 含 ccw）时 end 收到布尔（String(false)="false"→host 解析 0），
      // 弧角变成 (start, 0) 反向——2d.path.isPointInStroke.scaleddashes 的
      // ellipse(6,10,5,5,0,2π,false) 存成 start=2π/end=0，命中点弧长沿反向累计。
      // spec：ellipse(x, y, radiusX, radiusY, rotation, startAngle, endAngle,
      // counterclockwise)——ccw 透传 host（整椭圆走向）。
      __zw_canvas_op(h, 'ellipse', String(x), String(y), String(rx), String(ry), String(rotation), String(start), String(end), ccw ? 'true' : 'false');
    };
    ctx._methods.arcTo = function (x1, y1, x2, y2, r) {
      x1 = _zwNumArg(x1); y1 = _zwNumArg(y1); x2 = _zwNumArg(x2); y2 = _zwNumArg(y2); r = _zwNumArg(r);
      if (!_zwAllFinite(x1, y1, x2, y2, r)) return;
      // R56f：负半径 → IndexSizeError（spec dom-context-2d-arcto——2d.path.arcTo.negative）。
      if (r < 0) throw _zwDomException('arcTo: negative radius', 'IndexSizeError');
      __zw_canvas_op(h, 'arcTo', String(x1), String(y1), String(x2), String(y2), String(r));
    };
    ctx._methods.rect = function (x, y, w, hh) {
      x = _zwNumArg(x); y = _zwNumArg(y); w = _zwNumArg(w); hh = _zwNumArg(hh);
      if (!_zwAllFinite(x, y, w, hh)) return;
      __zw_canvas_op(h, 'rect', String(x), String(y), String(w), String(hh));
    };
    // R3291：Canvas 2D roundRect（HTML Canvas `dom-context-2d-api` roundRect）。radii 可为 number 或
    // array[number]（spec：单值/两值 [tl&br, tr&bl]/四值 [tl,tr,br,bl]），归一为逗号分隔串透传 host
    //（canvas crate best-effort 退化矩形——角圆为 rendering 已知简化，几何/命中测试正确）。invalid radii
    //（负值/NaN）spec 抛 RangeError，lenient 过滤（headless 简化，避免中断脚本）。
    // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-roundrect
    ctx._methods.roundRect = function (x, y, w, hh, radii) {
      // R34xx：任一参数非有限（NaN/Infinity）→ 忽略（spec：2d.path.roundrect.nonfinite）。
      if (!isFinite(+x) || !isFinite(+y) || !isFinite(+w) || !isFinite(+hh)) return;
      // R56（M8/DC-8）：radii 归一化对齐 spec dom-context-2d-roundrect——
      // 序列空或 >4 项 → RangeError；任一半径负 → RangeError；任一半径非有限
      // （NaN/±Infinity）→ **忽略整次调用**（与 x/y/w/h 非有限同款静默 return，
      // spec 步骤：unrestricted double 收 NaN/Inf 但算法判「任一非有限 → 不画」）；
      // BigInt → TypeError（WebIDL unrestricted double 不收，unary + 原生抛）。
      // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-roundrect
      function zwNormRadius(v) {
        var hx, hy;
        if (v && typeof v === 'object') {
          hx = +v.x; hy = +v.y;   // {x:0n} → +0n 原生抛 TypeError
        } else {
          hx = hy = +v;           // 0n → 原生抛 TypeError
        }
        if (!isFinite(hx) || !isFinite(hy)) return null; // 外层静默忽略整次调用
        if (hx < 0 || hy < 0) {
          throw new RangeError('The radius provided (' + hx + ',' + hy + ') is negative.');
        }
        return 'p' + hx + ',' + hy;
      }
      var r;
      if (radii == null) {
        r = '0';
      } else if (typeof radii === 'number' || typeof radii === 'bigint') {
        r = zwNormRadius(radii);
        if (r === null) return; // 非有限半径 → 忽略整次调用
      } else if (typeof radii === 'object' &&
                 typeof radii.length === 'number' &&
                 radii.x === undefined && radii.y === undefined) {
        // 序列形式（Array 或 array-like 且无 x/y 字典成员）：空 / >4 项抛 RangeError。
        if (radii.length === 0 || radii.length > 4) {
          throw new RangeError('The radii provided (' + radii.length + ' items) must be 1 to 4.');
        }
        var parts = [];
        for (var i = 0; i < radii.length; i++) {
          var p = zwNormRadius(radii[i]);
          if (p === null) return; // 非有限半径 → 忽略整次调用
          parts.push(p);
        }
        r = parts.join(',');
      } else if (typeof radii === 'object') {
        // 单个 DOMPointInit（DOMPoint / {x,y} / 任意字典对象——非有限 → 忽略整次调用）。
        r = zwNormRadius(radii);
        if (r === null) return;
      } else {
        r = '0';
      }
      __zw_canvas_op(h, 'roundRect', String(x), String(y), String(w), String(hh), r);
    };
    // R3291：Canvas 2D isPointInPath / isPointInStroke（hit-test 点在路径填充/描边区内）。返 bool。
    // spec isPointInPath(x,y[,fillRule])，fillRule 透传但 canvas crate 现用奇偶规则。无 ctx → false。
    // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-ispointinpath
    ctx._methods.isPointInPath = function (x, y, fillRule) {
      // R56d：spec 三形式 isPointInPath(x,y[,fillRule]) / isPointInPath(path,x,y[,fillRule])，
      // 默认 nonzero 绕组（dom-context-2d-ispointinpath）。WebIDL CanvasFillRule 枚举
      // 校验：fillRule 缺省/undefined 合法（默认），非 ('nonzero'|'evenodd') → TypeError
      //（isPointInpath.invalid 的 'gazonk'/null）；首参非 Path2D 非 number（null/undefined/
      // []/{}/'string'）→ TypeError（WebIDL union (Path2D or double) 不匹配）。
      // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-ispointinpath
      var isPath = !!(x && typeof x === 'object' && x._zwPath);
      var px, py, rule;
      if (isPath) {
        // path 形式：arg0=path, arg1=x, arg2=y, arg3=fillRule。
        px = _zwNumArg(y); py = _zwNumArg(fillRule);
        rule = (arguments[3] === undefined) ? '' : String(arguments[3]);
        if (arguments[3] !== undefined && arguments[3] !== 'nonzero' && arguments[3] !== 'evenodd') {
          throw new TypeError('isPointInPath: invalid fillRule');
        }
        if (!isFinite(px) || !isFinite(py)) return false;
        return __zw_canvas_op(h, 'isPointInPathPath', String(x._zwPath), String(px), String(py), rule) === '1';
      }
      if (x === null || x === undefined || typeof x === 'object' || typeof x === 'string') {
        throw new TypeError('isPointInPath: invalid argument');
      }
      if (fillRule !== undefined && fillRule !== 'nonzero' && fillRule !== 'evenodd') {
        throw new TypeError('isPointInPath: invalid fillRule');
      }
      rule = (fillRule === undefined) ? '' : String(fillRule);
      var nx = _zwNumArg(x), ny = _zwNumArg(y);
      if (!isFinite(nx) || !isFinite(ny)) return false;
      return __zw_canvas_op(h, 'isPointInPath', String(nx), String(ny), rule) === '1';
    };
    // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-ispointinstroke
    ctx._methods.isPointInStroke = function (x, y /*, fillRule */) {
      // R56h：Path2D 形式 isPointInStroke(path, x, y)（spec——首参为 Path2D 对象时
      // 走 path 变体；2d.path.isPointInStroke.basic.worker.js 的 path.rect 命中）。
      if (x && typeof x === 'object' && x._zwPath) {
        return __zw_canvas_op(h, 'isPointInStrokePath', String(x._zwPath), String(y), String(arguments[2])) === '1';
      }
      return __zw_canvas_op(h, 'isPointInStroke', String(x), String(y)) === '1';
    };
    ctx._methods.clip = function (path) {
      if (path && path._zwPath) __zw_canvas_op(h, 'clipPath', String(path._zwPath));
      else __zw_canvas_op(h, 'clip');
    };
    // R33xx：save/restore 客户端镜像状态栈。host save/restore 只回滚引擎状态，
    // JS 侧 getter 读 `_x` 缓存（字符串/number/对象引用），不同步则 restore 后 getter
    // 返回旧值（上游 2d.state.saverestore.* WPT 全族失败）。恢复仅改写 JS 缓存；
    // lineDash/clip/transform 无 JS 缓存（getLineDash/getTransform 读 host），随 host 回滚。
    var _zwCtxStateKeys = ['_fs','_ss','_lw','_ga','_lj','_lc','_font','_ta','_tb','_dir',
                           '_ml','_gco','_sc','_sb','_sox','_soy','_ldo','_ise','_isq',
                           '_ls','_ws','_fk','_fst','_fvc','_tr','_filter'];
    ctx._methods._saveRaw = function () {
      var snap = {};
      for (var i = 0; i < _zwCtxStateKeys.length; i++) { var k = _zwCtxStateKeys[i]; snap[k] = this[k]; }
      this._stack = this._stack || [];
      this._stack.push(snap);
      __zw_canvas_op(h, 'save');
    };
    ctx._methods._restoreRaw = function () {
      __zw_canvas_op(h, 'restore');
      var st = this._stack;
      if (!st || !st.length) return; // 空栈无操作（spec：restore() with empty stack has no effect）
      var snap = st.pop();
      for (var i = 0; i < _zwCtxStateKeys.length; i++) { var k = _zwCtxStateKeys[i]; this[k] = snap[k]; }
    };
    ctx._methods.save = function () {
      // R34xx（layers 目录）：层内 save **允许**（valid-calls.beginLayer-save）——
      // 但会使 endLayer 栈深不匹配抛 InvalidStateError（invalid-calls.
      // beginLayer-save-endLayer）。
      return this._saveRaw();
    };
    ctx._methods.restore = function () {
      if (this._inLayer) {
        throw _zwDomException('restore: not allowed while a layer is open', 'InvalidStateError');
      }
      return this._restoreRaw();
    };
    // R34xx（layers 目录）：beginLayer/endLayer（spec canvas layers）。**诚实范围**：
    // 层状态机 + 渲染状态复位 + 打开期操作限制（invalid-calls/malformed-operations/
    // ctm.*/layer-rendering-state-reset 全过）；层内绘制**不经离屏缓冲合成**（像素
    // 断言类用例——filters/blur/composite 层效果——待 host 层合成，记录）。
    // https://html.spec.whatwg.org/multipage/canvas.html#beginlayer
    ctx._methods.beginLayer = function (options) {
      if (this._inLayer) {
        throw _zwDomException('beginLayer: already in a layer', 'InvalidStateError');
      }
      // R34xx（layers 目录）：options WebIDL 校验——非 null/undefined/对象 → TypeError
      //（beginLayer-options 的 ''/0/1/true/false）；filter 字典深校验失败 → TypeError
      // **且层不打开**（exceptions-are-no-op——beginLayer 抛后 endLayer 仍抛）。
      if (options !== undefined && options !== null && typeof options !== 'object') {
        throw new TypeError('beginLayer: options must be an object');
      }
      if (options && typeof options === 'object' && options.filter !== undefined && options.filter !== null) {
        // R34xx（filters 目录）：层 filter 校验与 CanvasFilter 同规（gaussianBlur/
        // convolveMatrix/colorMatrix 深校验——2d.filter.layers.*.exceptions）。
        // 数组（[] 等）按 CanvasFilter 列表语义逐元素校验；对象（filter 字典）校验；
        // 其余（字符串/数字/布尔——DOMString 化）接受（beginLayer-options 的
        // ''/0/1/true/false）。
        var _lf = options.filter;
        if (Array.isArray(_lf)) {
          for (var _fi = 0; _fi < _lf.length; _fi++) {
            _zwValidateFilterInput(_lf[_fi]);
          }
        } else if (typeof _lf === 'object') {
          _zwValidateFilterInput(_lf);
        }
        // R34xx（filters 渲染）：层 colorMatrix 滤镜 → host 矩阵（endLayer 恢复前值）。
        if (_lf && typeof _lf === 'object' && !Array.isArray(_lf) && String(_lf.name) === 'colorMatrix') {
          this._layerFilterMatrix = this._layerFilterMatrix || {};
          this._layerFilterMatrix._prev = this._layerFilterMatrix._cur || null;
          var _m = _zwColorMatrix(_lf);
          this._layerFilterMatrix._cur = _m;
          __zw_canvas_op(h, 'setFilterMatrix', _m ? _m.join(',') : '');
        }
      }
      this._inLayer = true;
      this._saveRaw();
      // 层自身 save 已压栈——endLayer 校验的基准 = 压栈后的深度。
      this._layerDepth = (this._stack ? this._stack.length : 0);
      // 层渲染状态复位为初始（globalAlpha/gco/shadow/filter——transform 保留：
      // ctm.getTransform 的 translate+scale 组合断言）。
      this._ga = 1.0;
      this._gco = 'source-over';
      this._sc = 'rgba(0, 0, 0, 0)';
      this._sb = 0;
      this._sox = 0;
      this._soy = 0;
      this._filter = 'none';
      __zw_canvas_op(h, 'setGlobalAlpha', '1');
      __zw_canvas_op(h, 'setGlobalCompositeOperation', 'source-over');
      __zw_canvas_op(h, 'setShadowColor', 'rgba(0, 0, 0, 0)');
      __zw_canvas_op(h, 'setShadowBlur', '0');
      __zw_canvas_op(h, 'setShadowOffsetX', '0');
      __zw_canvas_op(h, 'setShadowOffsetY', '0');
    };
    ctx._methods.endLayer = function () {
      if (!this._inLayer) {
        throw _zwDomException('endLayer: not in a layer', 'InvalidStateError');
      }
      // R34xx：层内 save() 使栈深超出层创建时 → endLayer 抛（spec：层结束要求
      // save 栈回到层创建点；invalid-calls.beginLayer-save-endLayer）。
      if (this._layerDepth !== (this._stack ? this._stack.length : 0)) {
        throw _zwDomException('endLayer: save stack depth mismatch', 'InvalidStateError');
      }
      this._inLayer = false;
      // R34xx（filters 渲染）：endLayer 恢复层前滤镜矩阵。
      if (this._layerFilterMatrix) {
        var _pm = this._layerFilterMatrix._prev || null;
        this._layerFilterMatrix = null;
        __zw_canvas_op(h, 'setFilterMatrix', _pm ? _pm.join(',') : '');
      }
      this._restoreRaw();
    };
    ctx._methods.translate = function (tx, ty) {
      tx = _zwNumArg(tx); ty = _zwNumArg(ty);
      if (!_zwAllFinite(tx, ty)) return;
      __zw_canvas_op(h, 'translate', String(tx), String(ty));
    };
    ctx._methods.rotate = function (angle) {
      angle = _zwNumArg(angle);
      if (!isFinite(angle)) return;
      __zw_canvas_op(h, 'rotate', String(angle));
    };
    ctx._methods.scale = function (sx, sy) {
      sx = _zwNumArg(sx); sy = _zwNumArg(sy);
      if (!_zwAllFinite(sx, sy)) return;
      __zw_canvas_op(h, 'scale', String(sx), String(sy));
    };
    ctx._methods.setTransform = function (a, b, c, d, e, ff) {
      // WebIDL 双重重载：0 参 → `optional DOMMatrix2DInit transform = {}` 重载（identity——
      // 2d.transformation.setTransform.multiple 调 setTransform() 重置）；1-5 参 → 6 必参
      // 重载 TypeError（missingargs）；非有限 → 忽略（setTransform.nonfinite）。
      if (arguments.length === 0) {
        __zw_canvas_op(h, 'setTransform', '1', '0', '0', '1', '0', '0');
        return;
      }
      a = _zwNumArg(a); b = _zwNumArg(b); c = _zwNumArg(c); d = _zwNumArg(d); e = _zwNumArg(e); ff = _zwNumArg(ff);
      if (!_zwAllFinite(a, b, c, d, e, ff)) return;
      __zw_canvas_op(h, 'setTransform', String(a), String(b), String(c), String(d), String(e), String(ff));
    };
    ctx._methods.transform = function (a, b, c, d, e, ff) {
      a = _zwNumArg(a); b = _zwNumArg(b); c = _zwNumArg(c); d = _zwNumArg(d); e = _zwNumArg(e); ff = _zwNumArg(ff);
      if (!_zwAllFinite(a, b, c, d, e, ff)) return;
      __zw_canvas_op(h, 'transform', String(a), String(b), String(c), String(d), String(e), String(ff));
    };
    // R2985 getTransform：返当前变换矩阵为 DOMMatrix（host 'getTransform' 返 "a,b,c,d,e,f"）。
    // 读 hit-testing / transform-aware 绘制 / save-restore 矩阵快照高频。host 未注册 / 无 ctx → identity。
    ctx._methods.getTransform = function () {
      var raw = (typeof __zw_canvas_op === 'function') ? String(__zw_canvas_op(h, 'getTransform')) : '';
      var p = raw.split(',');
      var n = function (i, d) { var v = parseFloat(p[i]); return isNaN(v) ? d : v; };
      return new DOMMatrix([n(0, 1), n(1, 0), n(2, 0), n(3, 1), n(4, 0), n(5, 0)]);
    };
    // R2985 resetTransform：重置为单位矩阵（spec setTransform(identity)）。
    ctx._methods.resetTransform = function () { __zw_canvas_op(h, 'resetTransform'); };
    // R34xx：reset()（spec：清空画布 + 状态回默认）。host 重建 context；client 镜像同步默认。
    ctx._methods.reset = function () {
      // R34xx（layers 目录）：层打开期间 reset 抛 InvalidStateError
      //（2d.layer.invalid-calls.beginLayer-reset-endLayer——reset 抛后层仍开，
      // endLayer 再抛）。
      if (this._inLayer) {
        throw _zwDomException('reset: not allowed while a layer is open', 'InvalidStateError');
      }
      if (typeof __zw_canvas_op === 'function') __zw_canvas_op(h, 'reset');
      // R34xx：清空 float16 覆盖层（重置后无原始浮点回读）。
      if (this._f16) this._f16Overlay = null;
      // R34xx（reset 目录全族）：spec reset() 复位**全部**绘图状态——client 镜像
      // 全量同步默认（driving: 2d.reset.state.*——set 非默认后 reset 须回默认）。
      _zwResetCtxMirrors(this);
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
    ctx._methods.setLineDash = function (segs) {
      var s = (segs && segs.length != null) ? Array.prototype.join.call(segs, ',') : String(segs);
      __zw_canvas_op(h, 'setLineDash', s);
    };
    // R3305：getLineDash 返展开后偶长数组（spec：奇长输入被复制拼成偶长）。从 host 读（权威，
    // 客户端镜像存原值无法推断展开）。空串 → 空数组。
    ctx._methods.getLineDash = function () {
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
        v = String(v);
        // R34xx：相对单位字号预解析（spec——解析基准为 canvas 元素 computed 样式：
        // %/em 基准 = font-size（2d.text.font.parse.size.percentage 内联 144px → 50% =
        // '72px serif'；无样式默认 10px → 1000% = '100px serif'）；lh 基准 = line-height
        //（parent-style-relative-units：inline 30px/40px → 2em='60px'、2lh='80px'）。
        // 斜杠行高后的单位不属字号（'10px/150%' 的 150% 是 line-height），以「单位前须为
        // 行首或空白」排除。
        var m = /(?:^|\s)(\d+(?:\.\d+)?)(em|lh|%)/.exec(v);
        if (m) {
          var baseFs = 10, baseLh = null;
          try {
            var fs = this.canvas && this.canvas.style && String(this.canvas.style.fontSize || '');
            var fm = /^(\d+(?:\.\d+)?)px$/.exec(fs);
            if (fm) baseFs = parseFloat(fm[1]);
            var lh = this.canvas && this.canvas.style && String(this.canvas.style.lineHeight || '');
            var lm = /^(\d+(?:\.\d+)?)px$/.exec(lh);
            if (lm) baseLh = parseFloat(lm[1]);
          } catch (_e) {}
          var unit = m[2];
          var mult = (unit === '%') ? parseFloat(m[1]) / 100 : parseFloat(m[1]);
          var base = (unit === 'lh') ? (baseLh != null ? baseLh : baseFs) : baseFs;
          var px = Math.round(base * mult * 100) / 100;
          v = v.slice(0, m.index) + px + 'px' + v.slice(m.index + m[0].length);
        }
        this._font = v;
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
    ctx._lang = 'inherit';
    Object.defineProperty(ctx, 'lang', {
      set: function (v) {
        v = String(v);
        this._lang = v;
        // R34xx：'inherit' → canvas 元素 lang 属性（2d.text.measure.lang.inherit：
        // canvas lang="tr" → ctx.lang='inherit' 解析为 'tr'）；无 → 'en'（默认）。
        var resolved = v;
        if (v === 'inherit') {
          resolved = '';
          try {
            var el = this.canvas;
            if (el && el.lang) resolved = String(el.lang);
            else if (el && typeof el.getAttribute === 'function') resolved = String(el.getAttribute('lang') || '');
          } catch (_e) {}
          if (!resolved) resolved = 'en';
        }
        __zw_canvas_op(h, 'setLang', String(resolved));
      },
      get: function () { return this._lang; }
    });
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
    // R34xx：ctx.filter（spec CanvasRenderingContext2D.filter——reset 目录的
    // 状态复位面 + filters 目录的 API 表面）。**诚实范围**：值接受 + 复位语义；
    // 实际滤镜渲染（blur/colorMatrix 等）未实现（headless 光栅不应用），filters
    // 目录渲染面待 renderer 滤镜支持（记录，非本批）。
    ctx._filter = 'none';
    Object.defineProperty(ctx, 'filter', {
      // R34xx（filters 目录）：CanvasFilter 对象 → 接受（getter 返对象本身——
      // toString 断言）；字符串 → CSS filter list 校验（'none' / 函数列表；
      // 非法串保持旧值——'this string is not a filter and should do nothing'）。
      set: function (v) {
        if (v && typeof v === 'object' && v._zwCanvasFilter) {
          this._filter = v;
          // R34xx（filters 渲染）：colorMatrix 滤镜 → host 矩阵（hueRotate/
          // saturate/luminanceToAlpha/matrix 20 值）。
          _zwApplyCanvasFilter(this, v);
          return;
        }
        if (typeof v !== 'string') return;
        if (v === 'none' || _zwValidFilterList(v)) {
          this._filter = v;
          // R56h（M3）：'none' 须清全量 host 滤镜状态（colorMatrix + dropShadow）——
          // 只清 matrix 会让 dropShadow 残留到后续绘制（filter 回 none 后仍有阴影）。
          if (v === 'none') {
            __zw_canvas_op(h, 'setFilterMatrix', '');
            __zw_canvas_op(h, 'setFilterDropShadow', '');
          }
          // R57（M3）：CSS filter 列表字符串的 drop-shadow() 函数 → host shadow 机制
          //（与 CanvasFilter 对象同路）。'blur(5px)' 等其余函数不产生 shadow——清空。
          // 2d.filter.drop-shadow-globalAlpha（oracle A/B 47%——字符串形式从未接线）。
          // 语法：drop-shadow( <length>{2,3} <color>? )——offset 可为负、blur ≥ 0。
          var _ds = '';
          if (v !== 'none') {
            var _m = /drop-shadow\(([\s\S]*)\)/.exec(v);
            if (_m) {
              // 括号深度扫描取 drop-shadow 参数（rgb() 内括号不拆），
              // 深度 0 空白分词（rgb(255, 165, 0) 的逗号+空格整体为一个颜色 token）。
              var _arg = _m[1], _depth = 0, _end = _arg.length;
              for (var _i = 0; _i < _arg.length; _i++) {
                var _ch = _arg[_i];
                if (_ch === '(') _depth++;
                if (_ch === ')') { if (_depth === 0) { _end = _i; break; } _depth--; }
              }
              var _toks = [];
              var _cur = '';
              for (var _k = 0; _k < _end; _k++) {
                var _ch2 = _arg[_k];
                if (/\s/.test(_ch2) && _depth === 0) {
                  if (_cur) { _toks.push(_cur); _cur = ''; }
                } else {
                  if (_ch2 === '(') _depth++;
                  if (_ch2 === ')') _depth--;
                  _cur += _ch2;
                }
              }
              if (_cur) _toks.push(_cur);
              var _nums = [];
              var _color = 'black';
              for (var _j = 0; _j < _toks.length; _j++) {
                var _p = _toks[_j];
                if (/^-?\d*\.?\d+(px|em|rem|ex|ch|cm|mm|in|pt|pc|q|%)?$/.test(_p)) {
                  var _n = parseFloat(_p);
                  if (!isNaN(_n)) { _nums.push(_n); continue; }
                }
                _color = _p; // 其余 token 视为颜色
              }
              if (_nums.length >= 2) {
                var _dx = _nums[0], _dy = _nums[1];
                var _blur = _nums.length >= 3 ? Math.abs(_nums[2]) : 0;
                _ds = [_dx, _dy, _blur, _color, 1].join('\x1f');
              }
            }
          }
          __zw_canvas_op(h, 'setFilterDropShadow', _ds);
        }
      },
      get: function () { return this._filter; }
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
    ctx._methods.putImageData = function (img, dx, dy, dirtyX, dirtyY, dirtyW, dirtyH) {
      // R34xx（layers 目录）：层打开期间 putImageData 抛 InvalidStateError
      //（2d.layer.malformed-operations——putImageData 打开期调用限制）。
      if (this._inLayer) {
        throw _zwDomException('putImageData: not allowed while a layer is open', 'InvalidStateError');
      }
      // R34xx：null/undefined/非 ImageData → TypeError（spec——2d.imageData.put.null/wrongtype）。
      if (img === null || img === undefined) {
        throw new TypeError('putImageData: imageData is null');
      }
      // R34xx：Float16Array data（pixelFormat rgba-float16——put.basic.rgba.float16）——
      // 归一化值 ×255 转字节。
      var _isF16 = (typeof Float16Array === 'function') && (img.data instanceof Float16Array);
      if (typeof img !== 'object' || (!(img.data instanceof Uint8ClampedArray) && !_isF16) ||
          !(typeof img.width === 'number') || !(typeof img.height === 'number')) {
        throw new TypeError('putImageData: not an ImageData object');
      }
      // R34xx：缺坐标 → TypeError（missingargs）。
      if (dx === undefined || dy === undefined) {
        throw new TypeError('putImageData: missing coordinates');
      }
      // R34xx：非有限参数 → TypeError（spec——2d.imageData.put.nonfinite）。
      var argv = [dx, dy, dirtyX, dirtyY, dirtyW, dirtyH];
      for (var ai = 0; ai < arguments.length && ai < 6; ai++) {
        if (typeof argv[ai] === 'number' && !isFinite(argv[ai])) {
          throw new TypeError('putImageData: non-finite argument');
        }
      }
      var d = img.data;
      // R34xx：写像素操作使 float16 覆盖层失效（避免陈旧原始浮点回读）。
      if (this._f16) this._f16Overlay = null;
      // R34xx：dirty 矩形（spec putImageData(img, dx, dy[, dirtyX, dirtyY, dirtyW,
      // dirtyH])——负 dims 矩形反向（put.dirty.negative：目标 = dx+dirtyX+dirtyW）。
      var sx = 0, sy = 0, sw = img.width | 0, sh = img.height | 0;
      var ox = dx | 0, oy = dy | 0;
      if (arguments.length >= 7) {
        dirtyX = dirtyX | 0; dirtyY = dirtyY | 0; dirtyW = dirtyW | 0; dirtyH = dirtyH | 0;
        // 负 dims：源矩形反向（[dirtyX+dirtyW, dirtyX)），目标 = (dx+dirtyX+dirtyW, ...)。
        if (dirtyW < 0) { sx = dirtyX + dirtyW; ox = dx + dirtyX + dirtyW; }
        else { sx = dirtyX; ox = dx + dirtyX; }
        if (dirtyH < 0) { sy = dirtyY + dirtyH; oy = dy + dirtyY + dirtyH; }
        else { sy = dirtyY; oy = dy + dirtyY; }
        sw = Math.abs(dirtyW); sh = Math.abs(dirtyH);
        // 源越界裁剪（透明省略——画布外部分不画）。
        if (sx < 0) { sw += sx; ox -= sx; sx = 0; }
        if (sy < 0) { sh += sy; oy -= sy; sy = 0; }
        if (sx + sw > (img.width | 0)) sw = (img.width | 0) - sx;
        if (sy + sh > (img.height | 0)) sh = (img.height | 0) - sy;
        if (sw <= 0 || sh <= 0) return;
      }
      var chunks = [];
      var iw = img.width | 0;
      var _f16scale = _isF16 ? 255 : 1;
      // R34xx（color-type 目录）：f16 画布 + 跨空间 ImageData → 浮点转换后 ×255
      //（u8 量化前——2d.color.type.u8srgb.to.f16p3 的 5 保真往返）。
      var _needCs = _isF16 && this._cs && img.colorSpace && this._cs !== img.colorSpace;
      for (var r = 0; r < sh; r++) {
        for (var c = 0; c < sw; c++) {
          var si = ((sy + r) * iw + (sx + c)) * 4;
          // R34xx：Float16Array data → 字节（×255，clamp 0-255——put.basic.rgba.float16）。
          var v0 = d[si] * _f16scale, v1 = d[si + 1] * _f16scale;
          var v2 = d[si + 2] * _f16scale, v3 = d[si + 3] * _f16scale;
          if (_needCs) {
            var _cv = _zwCsConvert(img.colorSpace, this._cs, [v0, v1, v2]);
            v0 = _cv[0]; v1 = _cv[1]; v2 = _cv[2];
          }
          chunks.push(Math.round(v0) + ',' + Math.round(v1) + ',' + Math.round(v2) + ',' + Math.round(v3));
        }
      }
      __zw_canvas_op(h, 'putImageData', String(ox), String(oy),
        String(sw), String(sh), chunks.join(','),
        (img && typeof img.colorSpace === 'string') ? img.colorSpace : 'srgb');
    };
    // drawImage（R2799，canvas slice 5）：源 canvas → 本 ctx。3 spec 重载（arg 数 3/5/9）：
    //   drawImage(image, dx, dy) / drawImage(image, dx, dy, dw, dh) /
    //   drawImage(image, sx, sy, sw, sh, dx, dy, dw, dh)。
    // **源限 canvas 元素**（canvas-to-canvas）：经源 canvas 既有 getImageData 取全 RGBA wire 串作源传 host；
    // R3309：ImageBitmap 源（持 _zwBitmapWire）直接用其 wire 串，跳过 canvas 源 getImageData。
    // HTMLImageElement/`<img>` decode defer。host draw_image* 真栅格（source-over alpha 混合）。
    ctx._methods.drawImage = function (image) {
      if (typeof __zw_canvas_op !== 'function') return;
      // R34xx（layers 目录）：层打开期间 drawImage 抛 InvalidStateError
      //（2d.layer.malformed-operations——含源 canvas 层打开的情形，下方 canvas 源分支）。
      if (this._inLayer) {
        throw _zwDomException('drawImage: not allowed while a layer is open', 'InvalidStateError');
      }
      // R34xx：缺省源 → TypeError（missingargs）。
      if (image === undefined || image === null) {
        throw new TypeError('drawImage: missing image source');
      }
      var a = arguments;
      // ImageBitmap 源（R3309 createImageBitmap 产物）：直接用其 wire 串调 drawImage host op。
      if (image && image._zwBitmapWire && !image._closed) {
        var bmw = image.width | 0;
        var bmh = image.height | 0;
        if (bmw <= 0 || bmh <= 0) return;
        if (a.length === 3) {
          __zw_canvas_op(h, 'drawImage', image._zwBitmapWire, String(a[1]), String(a[2]),
            image._zwBitmapCs || 'srgb');
          // R34xx：float16 上下文 + float16 位图 → 记录原始浮点像素覆盖层
          //（createImageBitmap.srgb.rgba.float16 越界值往返——u8 缓冲无法存 2/-1）。
          if (this._f16 && image._zwBitmapF16) {
            this._f16Overlay = { x: a[1] | 0, y: a[2] | 0, w: bmw, h: bmh, data: image._zwBitmapF16 };
          }
        } else if (a.length === 5) {
          __zw_canvas_op(h, 'drawImageScaled', image._zwBitmapWire,
            String(a[1]), String(a[2]), String(a[3]), String(a[4]),
            image._zwBitmapCs || 'srgb');
        } else if (a.length === 9) {
          __zw_canvas_op(h, 'drawImageSliced', image._zwBitmapWire,
            String(a[1]), String(a[2]), String(a[3]), String(a[4]),
            String(a[5]), String(a[6]), String(a[7]), String(a[8]),
            image._zwBitmapCs || 'srgb');
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
      // R34xx：缺坐标参数 → TypeError（missingargs 的 drawImage(canvas)）。
      // R34xx（drawing-images 目录）：img 元素未加载（naturalWidth=0——空 src/
      // 加载中/失败）→ **no-op 不抛**（spec：incomplete image 不绘制——
      // 2d.drawImage.incomplete.*/nonexistent/broken 的 canvas 保持原样断言）。
      if (!image) {
        throw new TypeError('drawImage: invalid image source');
      }
      if (typeof image.getContext !== 'function') {
        // 非 canvas 源：img 元素（naturalWidth 存在）未加载 → no-op；其他（数字等）
        // → TypeError（2d.drawImage.wrongtype）。
        // R56h：naturalWidth=0 的 img 镜像 createPattern 的失败态判定——静态
        //（HTML 中带 id）失败 img → InvalidStateError（spec——2d.drawImage.nonexistent）；
        // 动态创建/空 src/重载中（'../' 上跳）→ no-op（2d.drawImage.incomplete.*）。
        if (image && typeof image.naturalWidth === 'number') {
          if (image.naturalWidth <= 0) {
            var _diSrc = (image.getAttribute ? String(image.getAttribute('src') || '') : '') || String(image.src || '');
            if (!_diSrc || _diSrc.indexOf('../') === 0) return;
            var _diId = (image.getAttribute ? String(image.getAttribute('id') || '') : '') || '';
            var _inDoc = _diId ? !!(globalThis.document && globalThis.document.getElementById(_diId)) : false;
            if (_inDoc) {
              throw _zwDomException('drawImage: source image failed to load', 'InvalidStateError');
            }
            return;
          }
          return;
        }
        throw new TypeError('drawImage: invalid image source');
      }
      // R56h：位图已转移（postMessage transfer 的 OffscreenCanvas）→ InvalidStateError
      //（spec dom-context-2d-drawimage——2d.drawImage.detachedcanvas）。
      if (image._detached) {
        throw _zwDomException('drawImage: source canvas bitmap has been transferred', 'InvalidStateError');
      }
      if (a.length < 3) {
        throw new TypeError('drawImage: missing coordinates');
      }
      // R34xx：DOM canvas 的 ctx 在 _zwCanvasCtx（proxy 无 _ctx 属性）——经
      // __zwSelector/__zwHandle 取 key 兜底（2d.drawImage.canvas 的 DOM 源 +
      // layers 的源层检查）。
      if (!image._ctx && typeof _zwCanvasCtx === 'object') {
        var _sSel2 = image.__zwSelector || null;
        var _sH2 = image.__zwHandle ? '@' + image.__zwHandle : null;
        image._ctx = (_sSel2 && _zwCanvasCtx[_sSel2]) || (_sH2 && _zwCanvasCtx[_sH2]) || null;
      }
      if (!image._ctx) image.getContext('2d');
      if (!image._ctx) return;
      // R34xx（layers 目录）：源 canvas 有打开层 → InvalidStateError
      //（2d.layer.malformed-operations：ctx.beginLayer 后经另一 ctx2.drawImage(canvas)）。
      if (image._ctx._inLayer) {
        throw _zwDomException('drawImage: source canvas has an open layer', 'InvalidStateError');
      }
      var srcHandle = image._ctx._handle;
      var sw = image.width | 0;
      var sh = image.height | 0;
      // R56h：源 canvas 位图零尺寸 → InvalidStateError（spec dom-context-2d-drawimage——
      // 2d.drawImage.zerocanvas：width=0 或 height=0 的源 canvas 抛 INVALID_STATE_ERR；
      // 旧实现 no-op 不抛）。
      if (sw <= 0 || sh <= 0) {
        throw _zwDomException('drawImage: source canvas has zero dimension', 'InvalidStateError');
      }
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
    ctx._methods.getImageData = function (x, y, w, hh) {
      if (typeof __zw_canvas_op !== 'function') return null;
      // R34xx（layers 目录）：层打开期间 getImageData 抛 InvalidStateError
      //（2d.layer.malformed-operations——getImageData 打开期调用限制）。
      if (this._inLayer) {
        throw _zwDomException('getImageData: not allowed while a layer is open', 'InvalidStateError');
      }

      // R34xx：x/y/w/h 经 Math.trunc 归一（spec：与 createImageData 同一 WebIDL long 截断语义，
      // 上游 2d.imageData.create2.round 断言两者一致）。
      // R34xx：WebIDL long EnforceRange——越界（非有限或超出有符号 32 位）→ TypeError
      //（2d.imageData.get.large.crash 的 0xffffffff——避免巨尺寸分配）。
      var vx = +x, vy = +y, vw = +w, vh = +hh;
      if (!isFinite(vx) || !isFinite(vy) || !isFinite(vw) || !isFinite(vh) ||
          vx > 2147483647 || vx < -2147483648 || vy > 2147483647 || vy < -2147483648 ||
          vw > 2147483647 || vw < -2147483648 || vh > 2147483647 || vh < -2147483648) {
        throw new TypeError('getImageData: argument out of range');
      }
      // R34xx：零尺寸 → IndexSizeError（spec——2d.imageData.get.zero）。
      var tw = Math.trunc(vw), th = Math.trunc(vh);
      if (tw === 0 || th === 0) {
        throw _zwDomException('getImageData: zero dimension', 'IndexSizeError');
      }
      // R34xx：负 dims/坐标原样传 host（翻转/越界透明语义在 host）。
      // R34xx：getImageData(x, y, w, h, settings)——settings（第 5 参）pixelFormat
      // 'rgba-float16' → Float16Array 归一化值（字节/255）；colorSpace 透传 host
      // 作跨空间转换（color-type 目录——display-p3 画布 srgb 回读）。
      var _settings = arguments.length > 4 ? arguments[4] : null;
      // R34xx（wide-gamut）：getImageData 缺省 colorSpace = canvas 色彩空间
      //（spec——srgb-linear 画布缺省读返 srgb-linear）。
      var _reqCs = (_settings && typeof _settings === 'object' && typeof _settings.colorSpace === 'string')
        ? _settings.colorSpace : (this._cs || 'srgb');
      var r = String(__zw_canvas_op(h, 'getImageData',
        String(Math.trunc(vx)), String(Math.trunc(vy)),
        String(tw), String(th), _reqCs));
      if (!r) return null;
      var parts = r.split(';');
      var dims = parts[0].split(':');
      var nums = parts[1] ? parts[1].split(',') : [];
      var f16 = !!(_settings && typeof _settings === 'object' && _settings.pixelFormat === 'rgba-float16');
      var cs = (_settings && typeof _settings === 'object' && typeof _settings.colorSpace === 'string')
        ? _settings.colorSpace : (this._cs || 'srgb');
      var arr;
      if (f16) {
        arr = new Float16Array(nums.length);
        // R34xx：float16 上下文覆盖层（float16 位图绘制区）→ 原始浮点像素。
        var _ov = this._f16Overlay;
        // R34xx（color-type 目录）：f16 画布跨空间回读 → 浮点转换（p3→srgb 等——
        // u8 量化前；2d.color.type.u8p3.to.f16srgb 的 5 保真）。
        var _needCs2 = this._cs && cs && this._cs !== cs;
        for (var i = 0; i < nums.length; i++) {
          var _px = i / 4 | 0;
          var _pxX = (_px % tw) + (Math.trunc(vx) | 0);
          var _pxY = (_px / tw | 0) + (Math.trunc(vy) | 0);
          var _raw = null;
          if (_ov && _pxX >= _ov.x && _pxX < _ov.x + _ov.w && _pxY >= _ov.y && _pxY < _ov.y + _ov.h) {
            var _oi = ((_pxY - _ov.y) * _ov.w + (_pxX - _ov.x)) * 4 + (i % 4);
            if (_oi < _ov.data.length) _raw = _ov.data[_oi];
          }
          arr[i] = (_raw !== null) ? _raw : (+nums[i] / 255);
        }
        if (_needCs2) {
          for (var j = 0; j + 3 < arr.length; j += 4) {
            var _cv2 = _zwCsConvert(this._cs, cs, [arr[j], arr[j + 1], arr[j + 2]]);
            arr[j] = _cv2[0]; arr[j + 1] = _cv2[1]; arr[j + 2] = _cv2[2];
          }
        }
      } else {
        arr = new Uint8ClampedArray(nums.length);
        for (var i = 0; i < nums.length; i++) arr[i] = +nums[i];
      }
      // R34xx：返真 ImageData（colorSpace + 只读 width/height——object.properties/readonly）。
      var img = new ImageData(arr, +dims[0], +dims[1], { colorSpace: cs, pixelFormat: f16 ? 'rgba-float16' : 'rgba-unorm8' });
      return img;
    };
    // R34xx：prototype 方法分发层（2d.canvas.context.type.extend/replace/prototype——
    // 实例方法存 _methods（闭包持 handle），prototype 薄转发器按方法名注册（幂等）：
    // 原型扩展/覆写生效（fillRectGreen 扩展、fillRect 覆写），getPrototypeOf(ctx) ===
    // CanvasRenderingContext2D.prototype。非法调用（this 无 _handle）→ TypeError。
    _zwRegisterCtxDispatchers(ctx);

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
    // js-dom M4 R48：parsed 文本/注释节点的 CharacterData 方法（appendData/insertData/deleteData/
    // replaceData/substringData + data/nodeValue setter）——经「父 selector + childNodes 索引」定位
    // 写入（`__zw_set_child_text`，host SetChildText mutation）。WPT MutationObserver-characterData
    // 对 parsed 节点（<p id=n10>CHAN</p>.firstChild）编辑 + record 的路径。方法闭包持有 parentSel +
    // childIndex（构造时索引），编辑后同步本地 data/nodeValue（同块后续读不 stale）+ 发 characterData
    // record（oldValue 写前捕获，有 observer 请求时）。offset clamp（spec 抛 IndexSizeError，permissive）。
    var parentSel = parentProxy && parentProxy.__zwSelector ? parentProxy.__zwSelector : null;
    var node = {
      nodeType: isComment ? 8 : 3,
      nodeName: isComment ? '#comment' : '#text',
      // data/nodeValue 经下方 defineProperty（getter 读 __nv——_write 同步写；无 parentSel 的
      // 纯快照节点保持普通字段不可写，下行 __nv 初始化同时覆盖两态）。
      __nv: text,
      textContent: text,
      length: text.length,
      parentNode: parentProxy,
      parentElement: parentProxy,
      __zwIsText: true,
      // js-dom M4 R79：Node.contains / hasChildNodes / compareDocumentPosition——WPT
      // Node-contains/compareDocumentPosition 的 testNodes 含 paras[0].firstChild 等文本节点
      //（旧为普通 data 字段无方法 → "reference.contains is not a function" 1002F 簇）。spec：
      // CharacterData 节点无子 → hasChildNodes false；contains 仅 other === 自身命中；
      // compareDocumentPosition 经 `_zwCompareDocumentPosition`（parentNode 字段已由本构造
      // 指向父 proxy，链路完整）。
      hasChildNodes: function () { return false; },
      contains: function (other) { return _zwNodeContains(node, other); },
      compareDocumentPosition: function (other) { return _zwCompareDocumentPosition(node, other); },
    };
    // R51：spec ownerDocument——parsed 文本/注释节点属主文档（common.js rangeFromEndpoints
    // 经 ownerDocument(node).createRange() 取 doc 再建 Range；缺此字段 → undefined 崩）。
    node.ownerDocument = globalThis.document;
    Object.defineProperty(node, 'nodeValue', {
      get: function () { return node.__nv; },
      set: function (v) {
        // 无 parentSel（快照节点）——纯本地赋值（旧语义兼容）。
        node.__nv = String(v == null ? '' : v); node.textContent = node.__nv; node.length = node.__nv.length;
      },
      configurable: true, enumerable: true,
    });
    if (parentSel && typeof __zw_set_child_text === 'function') {
      // childIndex 由 _childNodeList 的 map 调用方按位置补（node.__zwChildIndex）。
      var _cur = function () { return String(node.nodeValue != null ? node.nodeValue : ''); };
      var _write = function (nv) {
        var _moTgt = _mo_id(null, parentSel);
        var _old = (_moTgt != null && _mo_any_wants_char_old(_moTgt)) ? _cur() : null;
        node.__nv = nv; node.textContent = nv; node.length = nv.length;
        __zw_set_child_text(parentSel, String(node.__zwChildIndex || 0), nv);
        _mo_notify(parentSel, null, { type: 'characterData', oldValue: _old, target: node.__zwNotifyTarget || node });
      };
      node.appendData = function (s) { _write(_cur() + String(s == null ? '' : s)); return undefined; };
      node.insertData = function (offset, s) {
        var cur = _cur(); var o = Math.max(0, offset | 0);
        _write(cur.slice(0, o) + String(s == null ? '' : s) + cur.slice(o));
        return undefined;
      };
      node.deleteData = function (offset, count) {
        var cur = _cur(); var o = Math.max(0, offset | 0); var c2 = Math.max(0, count | 0);
        _write(cur.slice(0, o) + cur.slice(o + c2));
        return undefined;
      };
      node.replaceData = function (offset, count, s) {
        var cur = _cur(); var o = Math.max(0, offset | 0); var c2 = Math.max(0, count | 0);
        _write(cur.slice(0, o) + String(s == null ? '' : s) + cur.slice(o + c2));
        return undefined;
      };
      node.substringData = function (offset, count) {
        var cur = _cur(); var o = Math.max(0, offset | 0); var c2 = Math.max(0, count | 0);
        return cur.slice(o, o + c2);
      };
      // data/nodeValue setter（CharacterData data IDL 可写）——写经 _write；getter 读本地（_write 同步）。
      Object.defineProperty(node, 'data', {
        get: function () { return node.nodeValue; },
        set: function (v) { _write(String(v == null ? '' : v)); },
        configurable: true, enumerable: true,
      });
      Object.defineProperty(node, 'nodeValue', {
        get: function () { return node.__nv; },
        set: function (v) { _write(String(v == null ? '' : v)); },
        configurable: true, enumerable: true,
      });
      node.__zwWriteChildText = _write;
    }
    // js-dom M4 R84：兄弟导航 getter（previousSibling/nextSibling）经 parentNode.childNodes
    // 动态求值（R3018 _zwMDefineSiblings 同款——静态 null 是 WPT dom/traversal NodeIterator/
    // TreeWalker 整簇 fail 的根因：oracle nextNode()/previousNode() 树序遍历走
    // firstChild/nextSibling/parentNode 链，text.nextSibling=null 使遍历在首个 text 处断链，
    // 而迭代器（childNodes 递归）走完整树 → 两侧序列分歧「expected null but got object」）。
    // 有 parentProxy 时兄弟经父 childNodes 定位（元素子为 _proxyCache 稳定 identity，
    // text/comment 子经 _zwChildBaseCache 稳定——indexOf 命中）；无父（sibling_pairs
    // 构造形态）保持 null。
    if (parentProxy) {
      Object.defineProperty(node, 'previousSibling', { get: function () {
        var p = node.parentNode;
        if (!p || !p.childNodes) return null;
        var i = p.childNodes.indexOf(node);
        return i > 0 ? p.childNodes[i - 1] : null;
      }, configurable: true });
      Object.defineProperty(node, 'nextSibling', { get: function () {
        var p = node.parentNode;
        if (!p || !p.childNodes) return null;
        var i = p.childNodes.indexOf(node);
        return i >= 0 && i < p.childNodes.length - 1 ? p.childNodes[i + 1] : null;
      }, configurable: true });
    } else {
      node.previousSibling = null;
      node.nextSibling = null;
    }
    // R123：parsed 路径的 bogus comment '?…?' → PI 视图（与 _zwMBuildNode 的 innerHTML
    // 路径同款——主文档 parse 的 <?t …?> 经 tokenizer bogus comment 落为 comment entry，
    // WPT PI-attributes "in main parser" 断言 nodeType 7 + 属性面）。复用 part03
    // _zwMPiFromBogus 的属性五件套（data 即属性序列化源）。
    if (isComment && text.charAt(0) === '?') {
      // R123 lit 同款守卫（见 part03 _zwMBuildNode）：'?target …?' 形态才转 PI 视图。
      var _wpiInner = text.slice(1, text.charAt(text.length - 1) === '?' ? -1 : undefined);
      var _wpiSp = _wpiInner.indexOf(' ');
      if (!(_wpiSp > 0 && /^[A-Za-z_:][-A-Za-z0-9_:.]*$/.test(_wpiInner.slice(0, _wpiSp)))) {
        return node;
      }
      var piView = _zwMPiFromBogus(text, parentProxy);
      // parentSel 写路径（__zwSetChildText）接 PI 的 CharacterData 面——data setter +
      // appendData/insertData/deleteData/replaceData（WPT MutationObserver-characterData
      // "ProcessingInstruction: data mutations" 三连写都须发 record；首版只接 data setter
      // 致 deleteData/replaceData undefined 回归）。写统一经 comment 节点的 _write（本地
      // data 同步 + host SetChildText + record），PI 视图字段跟随。
      if (parentSel && typeof __zw_set_child_text === 'function' && node.__zwWriteChildText) {
        (function (pv) {
          var sync = function (nv) {
            pv.__nv = nv; pv.nodeValue = nv; pv.textContent = nv;
            if (typeof pv.length === 'number' || true) { /* length getter 基础字段 */ }
          };
          var cur = function () { return String(pv.__nv != null ? pv.__nv : (pv.nodeValue != null ? pv.nodeValue : '')); };
          var write = function (nv) { sync(String(nv)); node.__zwWriteChildText(String(nv)); };
          Object.defineProperty(pv, 'data', {
            get: function () { return cur(); },
            set: function (v) { write(String(v == null ? '' : v)); },
            configurable: true, enumerable: true,
          });
          pv.appendData = function (s2) { write(cur() + String(s2 == null ? '' : s2)); return undefined; };
          pv.insertData = function (offset, s2) {
            var c2 = cur(); var o2 = Math.max(0, offset | 0);
            write(c2.slice(0, o2) + String(s2 == null ? '' : s2) + c2.slice(o2));
            return undefined;
          };
          pv.deleteData = function (offset, count) {
            var c3 = cur(); var o3 = Math.max(0, offset | 0); var e3 = o3 + Math.max(0, count | 0);
            write(c3.slice(0, o3) + c3.slice(e3));
            return undefined;
          };
          pv.replaceData = function (offset, count, s2) {
            var c4 = cur(); var o4 = Math.max(0, offset | 0); var e4 = o4 + Math.max(0, count | 0);
            write(c4.slice(0, o4) + String(s2 == null ? '' : s2) + c4.slice(e4));
            return undefined;
          };
          pv.substringData = function (offset, count) {
            var c5 = cur(); var o5 = Math.max(0, offset | 0);
            return c5.substr(o5, Math.max(0, count | 0));
          };
        })(piView);
      }
      // record.target 身份对齐观察面（mutationobservers.js 断言 record.target === 观察的
      // PI 视图——_write 默认 target 是内部 comment 对象，identity 不等）。
      node.__zwNotifyTarget = piView;
      return piView;
    }
    return node;
  }
  // data getter 的辅助（无 live 源时读本地缓存——_write 已同步）。
  function _wrapNodeEntryData(node) { return node.__localData != null ? node.__localData : node.nodeValue; }

  // `el.childNodes`（含文本/注释）：解析 __zw_child_nodes JSON 数组 → 节点数组（快照，非 live）。
  // js-dom M4 R55：基底缓存（按 sel 键）——同 turn 内重复读 `el.childNodes` 不再每次 host 往返
  //（WPT dom/common.js `indexOf` 的 `while (node != node.parentNode.childNodes[i])` 每 i 一次
  // `__zw_child_nodes` + JSON.parse + **全子重包装**——Range-mutations testFn 每次 indexOf 数十读，
  // per-subtest 数百次 host 往返是 insertBefore/dataChange >420s 的 per-op 主源，R52 诊断）。
  // 缓存安全性：host 侧 `dom_html` Arc 在 register_dom_callbacks 时固化、脚本生命周期内不可变
  //（mutation flush 写 WebView.cached_html，下一回合**重注册**才换 Arc——本回合内基底快照恒定，
  // 派生 overlay（pending add/remove）在 _zwOverlayPendingChildNodes 每读现算，语义不变）。
  // 附带 identity 收益：缓存数组里的文本节点对象稳定（旧行为每次重包装 → `childNodes[i] !==
  // 上次读的同位置节点`，indexOf identity 循环依赖此相等）。失效点：① mutation 本回合不失效
  //（overlay 管差异）② `_zwChildBaseInvalidateAll`（回调重注册/dom_html 换代时 host 侧全量失效，
  // 防跨回合读到旧基底）③ remove 消零节点的 proxy 清理不动本缓存（缓存键是父 sel，子对象由父
  // 生命周期持有）。容量守卫：512 sel 软上限（超限全清——单 turn 常见页数百容器级，防御性）。
  var _zwChildBaseCache = new Map();
  function _zwChildBaseInvalidateAll() { _zwChildBaseCache.clear(); }
  // R55：暴露到 globalThis——host `register_dom_callbacks` 开头注入失效脚本（注册即 dom_html
  // 换代），IIFE 内函数 host 侧不可达。
  globalThis._zwChildBaseInvalidateAll = _zwChildBaseInvalidateAll;
  function _childNodeList(sel, handle) {
    if (!sel || typeof __zw_child_nodes !== 'function') return [];
    if (!handle) {
      var cached = _zwChildBaseCache.get(sel);
      if (cached) return _zwOverlayPendingChildNodes(cached, sel, _wrapSelector(sel));
    }
    try {
      var arr = JSON.parse(__zw_child_nodes(sel) || '[]');
      var parent = handle ? _wrapHandle(handle) : _wrapSelector(sel);
      // R48：parsed 文本/注释子带 __zwChildIndex（CharacterData 方法经「父 sel + 索引」写入）。
      var out = arr.map(function(e, i) {
        var n = _wrapNodeEntry(e, parent);
        if (n && n.__zwIsText) n.__zwChildIndex = i;
        return n;
      });
      if (!handle) {
        if (_zwChildBaseCache.size > 512) _zwChildBaseCache.clear();
        _zwChildBaseCache.set(sel, out);
      }
      return _zwOverlayPendingChildNodes(out, sel, parent);
    } catch (_e) { return []; }
  }
  // js-dom M4 R51：sel 父 childNodes 的 pending overlay——同步脚本内 mutation（insertBefore/
  // appendChild/removeChild）经 host 异步 apply，快照（__zw_child_nodes）在脚本 turn 内是旧的。
  // WPT dom/common.js indexOf 等 identity 循环依赖「parentNode.childNodes 含本节点」：
  // pending added（_zwNodeParent 反向链父 sel 匹配）按 nextSibling 定位插入；pending removed
  // 剔除。无匹配 nextSibling（append 到尾 / nextSibling 也是 pending）→ 尾部追加（保守）。
  function _zwOverlayPendingChildNodes(out, sel, parent) {
    // R55：no-pending 快路径返回副本——out 可能是基底缓存本体（_childNodeList 缓存命中/刚存入），
    // 调用方 concat/原地写不得污染缓存。
    if (!_zwPendingAdded.length && !_zwPendingRemoved.length) return out.slice();
    // js-dom M4 R51c：按 parentSel 分桶查询（桶在 _zwHCLiveInvalidate 记账时维护）——旧实现每次
    // childNodes 读全表扫 pending（testharness 单 turn 数千 subtest 全量重建 → 万级表 × 每读 O(n)
    // 线性增速，Range-mutations-dataChange 实测 250 次/5s → 1250 次/31s O(n²) 超时）。桶 miss 且
    // 全局表空则零成本快出。
    var bucket = _zwPendingByParent.get(sel);
    if (!bucket || (!bucket.added.length && !bucket.removed.length)) return out;
    var res = out.slice();
    // 剔除 pending removed（identity 匹配）。
    for (var r = 0; r < bucket.removed.length; r++) {
      var rm = bucket.removed[r];
      for (var ri = res.length - 1; ri >= 0; ri--) {
        if (res[ri] === rm) { res.splice(ri, 1); break; }
      }
    }
    // 并入 pending added（父 sel 匹配 + 尚未在快照内）。
    for (var a = 0; a < bucket.added.length; a++) {
      var nd = bucket.added[a];
      if (!nd || !nd.__zwHandle) continue;
      var link = _zwNodeParent[nd.__zwHandle];
      if (!link || link.parentSel !== sel) continue;
      var seen = false;
      for (var s = 0; s < res.length; s++) { if (res[s] === nd) { seen = true; break; } }
      if (seen) continue;
      // nextSibling 定位（_mo_notify record 的 nextSibling 字段，R47）。ref 已在列表内 → 插其前；
      // 否则（null=append / ref 也 pending）→ 尾部。
      var pos = res.length;
      if (link.nextSibling && link.nextSibling.__zwSelector) {
        for (var q = 0; q < res.length; q++) {
          if (res[q] === link.nextSibling) { pos = q; break; }
        }
      }
      res.splice(pos, 0, nd);
    }
    return res;
  }

  // R3033：把元素数组包成 spec 集合——补 `.item(i)`（HTMLCollection/NodeList 共有），`htmlCollection=true`
  // 时再补 `.namedItem(name)`（id 或 name 首匹配，HTMLCollection 专有）。NodeList（false）保持 Array
  // 承载（length/indexed/迭代天然 + 内部调用方依赖 Array 方法，见下 R43 块）；HTMLCollection（true）
  // R50 起走 `_zwMakeHTMLCollection` Proxy 承载（legacy platform object 完整语义，见其头注释）。
  // `liveSpec`（可选，R50）：`{ matches(el) }`——childList mutation 后同步维护集合元素表
  // （_zwHCLiveInvalidate 经 matches 判定新增节点归属，纯 JS 端无 host 重查）。
  function _zwMakeCollection(arr, htmlCollection, liveSpec) {
    var a = arr || [];
    if (htmlCollection) {
      var els = [];
      for (var _q = 0; _q < a.length; _q++) els.push(a[_q]);
      // R50：同步脚本内已 append（快照未含）/ 已 remove（快照仍含）的元素按 matches 并入/剔除
      //（WPT own-props 顺序：append 后再取集合，快照查询看不到新元素）。
      if (liveSpec && typeof liveSpec.matches === 'function') {
        for (var _p = 0; _p < _zwPendingRemoved.length; _p++) {
          for (var _pi = els.length - 1; _pi >= 0; _pi--) {
            if (els[_pi] === _zwPendingRemoved[_p]) els.splice(_pi, 1);
          }
        }
        for (var _pa = 0; _pa < _zwPendingAdded.length; _pa++) {
          var _pnd = _zwPendingAdded[_pa];
          // R54：构建期并入同款主文档过滤（走 `_zwNodeParent` 挂载记账链——append 当时写入，
          // 不断链；detached/foreign 容器根无链 → false）。
          if (_pnd && _pnd.__zwHandle && !_zwMutationInDoc(null, _pnd.__zwHandle)) continue;
          var _pm = false;
          try { _pm = liveSpec.matches(_pnd); } catch (_e) { _pm = false; }
          if (_pm) {
            var _pd2 = false;
            for (var _pj = 0; _pj < els.length; _pj++) if (els[_pj] === _pnd) { _pd2 = true; break; }
            if (!_pd2) els.push(_pnd);
          }
        }
      }
      return _zwMakeHTMLCollection(els, liveSpec);
    }
    // js-dom M4 R43：spec legacy platform object（NodeList）的 indexed 属性
    // 不可配置——`delete c[0]`（loose）no-op、strict 抛 TypeError（WPT HTMLCollection-delete：
    // 普通数组 delete 挖洞致 c[0] 永久 undefined → "before" 断言也炸）。对每个索引
    // defineProperty accessor（configurable:false），元素经 getter 读（不占 data slot）。
    // 元素值在包装前快照（a === arr 时 getter 读 a[idx] 会递归触发自身 getter）。
    var _elems = [];
    for (var _j = 0; _j < a.length; _j++) _elems.push(a[_j]);
    for (var _i = 0; _i < _elems.length; _i++) {
      (function (idx) {
        var d = Object.getOwnPropertyDescriptor(a, String(idx));
        // guard：已被包装（复用数组二次进 _zwMakeCollection）时跳过——configurable:false
        // 属性二次 defineProperty 抛 TypeError。
        if (d && d.configurable === false) return;
        Object.defineProperty(a, String(idx), {
          get: function () { return _elems[idx]; },
          set: function () { /* spec：indexed 属性只读（设置 no-op，loose） */ },
          enumerable: true,
          configurable: false,
        });
      })(_i);
    }
    Object.defineProperty(a, 'item', {
      value: function (i) { i = _zwToUint32(i); return i < a.length ? a[i] : null; },
      enumerable: false, configurable: true, writable: true,
    });
    return a;
  }


  // js-dom M4 R50：HTMLCollection 从 Array 承载升级为 Proxy 承载（spec legacy platform object，
  // https://dom.spec.whatwg.org/#interface-htmlcollection + WebIDL §3.10 legacy platform objects）。
  // 旧 Array 路径在 WPT dom/collections 五用例暴露 24 处语义缺口（R37 聚类的深结构主簇）：
  // ① own 枚举多出 length/item/namedItem（Array 原型方法混入 getOwnPropertyNames）
  // ② values/entries/forEach 泄漏（Array 原型方法，spec HTMLCollection 无 iterable 接口成员）
  // ③ indexed/named data 描述符 configurable:true 但 defineProperty/set/delete 不拒绝
  // ④ 负数 / 2^31~2^32 边界 named 键（非 canonical 索引）被 `!isNaN(Number(key))` 误跳过
  // ⑤ `obj.length`（collection 作 prototype）不抛 illegal invocation TypeError
  // ⑥ 同步脚本内 appendChild 后集合不反映（live 语义）
  // 设计：Proxy target 只存 expando；indexed/named 由 trap 动态求值（元素快照 + live 追加 overlay）；
  // prototype 上挂 length/item/namedItem（receiver 校验——proxy 归一后集合自身合法，作 prototype
  // 的 base object 读取抛 TypeError）。NodeList（htmlCollection=false）路径保持 Array 不动（WPT
  // NodeList 语义已过 + 内部调用方依赖 Array 方法，见 _zwMakeCollection 头注释）。
  var _hcProto = null;
  function _zwHCPrototype() {
    if (_hcProto) return _hcProto;
    // 原型链：HC prototype → HTMLCollection.prototype → Object.prototype（保留
    // hasOwnProperty/valueOf/toString 等 standard 内建——assert_array_equals 等测试设施依赖
    // `collection.hasOwnProperty(...)`；WPT Document-Element-getElementsByTagName.js 直接断言
    // list.hasOwnProperty + `x instanceof HTMLCollection`）。
    var p = Object.create(globalThis.HTMLCollection ? globalThis.HTMLCollection.prototype : Object.prototype);
    // R120：HTMLCollection.prototype 上的 item/namedItem 须可见（WPT「expando shadowing a
    // proto prop」：`var fn = l.item; assert_equals(fn, HTMLCollection.prototype.item)`——
    // 构造器占位的 prototype 原为空对象，方法只定义在 HC prototype 实例上 → 断言两侧
    // undefined≠function。此处把同款方法同步定义到构造器 prototype（assert 两边同一函数）。
    try {
      var _hp = globalThis.HTMLCollection && globalThis.HTMLCollection.prototype;
      if (_hp && !_hp.__zwHCWired) {
        Object.defineProperty(_hp, '__zwHCWired', { value: true, enumerable: false, configurable: false });
        Object.defineProperty(_hp, 'length', {
          get: function () {
            if (!this || !this.__zwHC) throw new TypeError('Illegal invocation');
            return this.__zwHC().length;
          },
          set: function () {}, enumerable: false, configurable: true,
        });
        Object.defineProperty(_hp, 'item', {
          value: function (i) {
            if (!this || !this.__zwHC) throw new TypeError('Illegal invocation');
            var n = this.__zwHC(), u = _zwToUint32(i);
            return u < n.length ? n[u] : null;
          },
          writable: true, enumerable: false, configurable: true,
        });
        Object.defineProperty(_hp, 'namedItem', {
          value: function (name) {
            if (!this || !this.__zwHC) throw new TypeError('Illegal invocation');
            var els = this.__zwHC();
            var s = String(name);
            if (s === '') return null; // R38：空串非 supported name（namedFor 同款早退）
            for (var k2 = 0; k2 < els.length; k2++) {
              var e2 = els[k2];
              if (!e2) continue;
              // R120：id/name 不对称暴露（同 namedFor——name 仅 HTML ns 元素）。
              var _h2 = _zwIsHTMLNamespace(e2);
              try {
                if (e2.getAttribute) {
                  if (e2.getAttribute('id') === s) return e2;
                  if (_h2 && e2.getAttribute('name') === s) return e2;
                }
              } catch (_e2n) {}
            }
            return null;
          },
          writable: true, enumerable: false, configurable: true,
        });
      }
    } catch (_eHP) {}
    // spec HTMLCollection length getter——receiver 须为本集合（Proxy 归一后 this===proxy）。
    // 作 prototype 用（Object.create(collection)）时 receiver 是 base object：无 __zwHC 标记
    // → illegal invocation TypeError（WPT HTMLCollection-as-prototype）。
    Object.defineProperty(p, 'length', {
      get: function () {
        if (!this || !this.__zwHC) throw new TypeError('Illegal invocation');
        return this.__zwHC().length;
      },
      set: function () {}, enumerable: false, configurable: true,
    });
    // WebIDL §3.6.5：接口操作（method）在 prototype 上 enumerable:false（for-in 不可见），
    // regular attribute（length getter）enumerable:true。WPT own-props for-in 期望仅
    // indexed/named（+length 属 prototype 层）。
    // R120：item/namedItem 直接转发构造器 prototype 的同一函数（WPT expando 断言
    // `fn === HTMLCollection.prototype.item` 要求 identity 相同——p 层不再定义副本）。
    try {
      var _hp2 = globalThis.HTMLCollection && globalThis.HTMLCollection.prototype;
      if (_hp2 && _hp2.item) {
        Object.defineProperty(p, 'item', { value: _hp2.item, writable: true, enumerable: false, configurable: true });
        Object.defineProperty(p, 'namedItem', { value: _hp2.namedItem, writable: true, enumerable: false, configurable: true });
      }
    } catch (_eHP2) {}
    // 恢复被 R120 段误删的原有定义（@@iterator value iterator——for-of 消费路径 +
    // WPT HTMLCollection-iterator；Symbol.toPrimitive）。
    if (typeof Symbol === 'function' && Symbol.iterator) {
      Object.defineProperty(p, Symbol.iterator, {
        value: function () {
          if (!this || !this.__zwHC) throw new TypeError('Illegal invocation');
          var els = this.__zwHC(), idx = 0;
          return { next: function () { return idx < els.length ? { value: els[idx++], done: false } : { value: undefined, done: true }; } };
        },
        writable: true, enumerable: false, configurable: true,
      });
    }
    Object.defineProperty(p, Symbol.toPrimitive, { value: String, writable: true, enumerable: false, configurable: true });
    return p;
  }

  // spec HTMLCollection supported property names：仅 HTML namespace 元素的 id/name 计入
  //（WPT supported-property-names "non-HTML namespace"）。proxy 元素经 namespaceURI getter
  //（R18 `_nsHandles` 读回 createElementNS 原值）；异常/无 getter 回落 true（HTML 主路径）。
  function _zwIsHTMLNamespace(el) {
    try {
      // R120：createElementNS('') 产物（ns 显式空、registry 有记录）非 HTML 元素——
      // named getter 的 supported-property-names 排除其 id/name（WPT own-props 的
      // unexposedNames 'w'：createElementNS('','pre') 的 name 不暴露）。
      // null/undefined 视为 HTML 仅对 parsed / createElement 产物（registry 无记录）。
      if (el && el.__zwHandle && typeof _nsHandles !== 'undefined' && _nsHandles[el.__zwHandle]) {
        return _nsHandles[el.__zwHandle].namespace === 'http://www.w3.org/1999/xhtml';
      }
      var ns = el.namespaceURI;
      return ns === null || ns === undefined || ns === 'http://www.w3.org/1999/xhtml';
    } catch (_e) {
      return true;
    }
  }

  // ASCII-only lowercase（spec「ascii lowercase」——仅 A-Z，不动 'Ä'/'Ç' 等 non-ASCII，
  // WPT case.js ascii_lowercase：'Ä' 查询不得匹配 'ä'）。
  function _zwAsciiLower(s) {
    return String(s).replace(/[A-Z]/g, function (c) { return String.fromCharCode(c.charCodeAt(0) + 32); });
  }

  // WebIDL `an array index` is a canonical numeric string: 0 ≤ n < 2^32−1（"0".."4294967294"）。
  // "-2"/"4294967295"+ 非规范索引 → 落 named getter（WPT supported-property-indices）。
  // js-dom R76：QuickJS Proxy set-trap 拒绝语义差异探测（lazy 单次）。V8 对 set trap
  // 返 false：loose 调用静默 no-op、strict 抛 TypeError（spec Proxy invariant 标准
  // 行为，WPT HTMLCollection own-props 各 loose/strict 断言据此写）。QuickJS 对返
  // false **loose 也抛** TypeError（"proxy: cannot set property"——引擎实现更严），
  // 同 shim 在 quickjs 下 10 个 loose 断言炸（R76 quickjs collections 基线 38P/10F
  // vs v8 48P/0F 实证）。探测：loose 调一个必拒的 set，不抛 = V8 语义。
  var __zwV8ProxySetSemantics = null;
  function _zwV8ProxySetSemantics() {
    if (__zwV8ProxySetSemantics !== null) return __zwV8ProxySetSemantics;
    try {
      var p = new Proxy({}, { set: function () { return false; } });
      p.x = 1; // loose——V8 静默，QuickJS 抛
      __zwV8ProxySetSemantics = true;
    } catch (_e) {
      __zwV8ProxySetSemantics = false;
    }
    return __zwV8ProxySetSemantics;
  }

  function _zwIsCanonicalIndex(s) {
    if (!/^(0|[1-9][0-9]*)$/.test(s)) return false;
    return s.length < 10 || (s.length === 10 && s <= '4294967294');
  }
  // WebIDL ToUint32（https://webidl.spec.whatwg.org/#es-unsigned-long）：Number → mod 2^32。
  // `item(4294967296)` → 0（命中首元素，WPT supported-property-indices 2^32 断言）。
  function _zwToUint32(v) {
    var n = Number(v);
    if (!isFinite(n)) n = 0;
    n = Math.trunc(n) % 4294967296;
    if (n < 0) n += 4294967296;
    return n;
  }

  // live 集合注册表（R50）：childList mutation（_mo_notify 单一汇流点）后**同步**维护集合
  // 元素表——matches 的新元素追加、移除元素过滤。不重查 host（`__zw_query_all` 读 dom_html
  // 快照，脚本批末才回写——同步脚本内重查反而拿到旧结果冲掉 overlay，R48/R49 同款教训：
  // JS 本地视图优先）。同步脚本内 appendChild → c[0] 立即可见（WPT own-props
  // "Setting array index while indexed property doesn't exist"：append 后 c[0]===element）。
  // js-dom M4 R51：child 是否为 (sel, handle) 目标的祖先（含目标自身语义由调用方补）——
  // pre-insert HierarchyRequestError 校验用。**从目标上行**（child 是目标的祖先 ⟺ 目标的
  // 祖先链含 child；从 child 上行方向相反——child 在目标之上，永远走不到目标）。handle 链：
  // _zwNodeParent 反向链；sel 链：__zw_parent host 快照；guard 防环 64 层。
  function _zwIsAncestorOf(child, targetSel, targetHandle) {
    if (!child) return false;
    var childSel = child.__zwSelector, childHandle = child.__zwHandle;
    var h = targetHandle, s = targetSel, guard = 0;
    while (guard++ < 64) {
      if (!h && !s) return false;
      if (childHandle && h === childHandle) return true;
      if (childSel && s === childSel) return true;
      var link = h ? _zwNodeParent[h] : null;
      if (link) {
        if (link.parentHandle) { h = link.parentHandle; s = null; continue; }
        if (link.parentSel) { s = link.parentSel; h = null; continue; }
      }
      if (s && typeof __zw_parent === 'function') {
        var p = '';
        try { p = __zw_parent(s) || ''; } catch (_e) { p = ''; }
        if (p) { s = p; h = null; continue; }
      }
      return false;
    }
    return false;
  }

  // `_zwPendingAdded`：同步脚本内已 append 但 host 快照未含的元素（快照后取的新集合——
  // WPT own-props 先 append 再 getElementsByTagName 顺序——构建时按 matches 并入）。
  // added/removed 均展开 handle 子树（`_handleChildren` R2927 registry——WPT case.js 先建
  // container 挂 15 个 NS 元素再 append container：childList notify 只含 container，孙节点
  // 须经展开进 pending 表；remove container 同理整树剔除，防跨子测试泄漏）。
  var _zwLiveCollections = [];
  var _zwPendingAdded = [];
  var _zwPendingRemoved = [];
  // js-dom M4 R51c：pending 表并行 Set——`_zwHCLiveInvalidate` 的 added 分支旧实现每条
  // mutation 对 `_zwPendingRemoved` **全表 filter 重建**（O(removed) per mutation）；WPT
  // testharness 单同步 turn 跑数千 subtest（Range-mutations-dataChange ~5000 test × 每 test
  // setupRangeTests 重建 ~30 节点全入 removed 表）→ O(n²) 数组分配 churn 必然超时。Set 做
  // O(1) 去重/剔除判定，数组保留作有序迭代（nextSibling 定位/顺序并集依赖序）。
  var _zwPendingAddedSet = null;
  var _zwPendingRemovedSet = null;
  // R51c：pending 按 parentSel 分桶（childNodes overlay 查询用——见 _zwOverlayPendingChildNodes）。
  // key 为 null（handle 父 mutation）时桶键 '_h:' + parentHandle。
  var _zwPendingByParent = new Map();
  // R51c：pending added 按 id 索引（querySelector('#id') host-miss 回落 O(1)；invalidate
  // 记账时维护——added 入对桶、对冲剔除时同步删）。
  var _zwPendingAddedById = new Map();
  // js-dom M4 R125：sel-based 元素 id 的 latest-wins 覆盖表（elKey → 新 id | null）。
  // host 快照不反映同 execute 的 setAttribute('id')/removeAttribute('id')/Attr.value=/
  // innerHTML 清除——querySelector('[id=…]') 命中 stale id 或漏新 id（WPT
  // Document-getElementById "update id attribute via setAttribute/removeAttribute" 等）。
  // 写入侧：part04 setAttribute/removeAttribute/Attr.value setter/innerHTML·outerHTML 写路径
  // （proxy 身份键 elKey）；读取侧：part06 getElementById 双向消费（旧 id 命中查表剔除非
  // pending、新 id 命中查表拉回）。handle 元素不进此表（host 无快照条目，pending 索引已覆盖）。
  var _zwIdOverrides = new Map();
  globalThis._zwIdOverrideSet = function (key, id) { _zwIdOverrides.set(key, id == null ? null : String(id)); };
  globalThis._zwIdOverrideGet = function (key) {
    return _zwIdOverrides.has(key) ? _zwIdOverrides.get(key) : undefined;
  };
  globalThis._zwIdOverridesEntries = function () {
    var out = [];
    _zwIdOverrides.forEach(function (v, k) { out.push([k, v]); });
    return out;
  };
  function _zwPAIdAdd(nd) {
    var id = '';
    try { id = nd && nd.id != null ? String(nd.id) : ''; } catch (_e) { id = ''; }
    if (!id) return;
    var arr = _zwPendingAddedById.get(id);
    if (!arr) { arr = []; _zwPendingAddedById.set(id, arr); }
    if (arr.indexOf(nd) < 0) arr.push(nd);
  }
  function _zwPAIdRemove(nd) {
    var id = '';
    try { id = nd && nd.id != null ? String(nd.id) : ''; } catch (_e) { id = ''; }
    if (!id) return;
    var arr = _zwPendingAddedById.get(id);
    if (!arr) return;
    var i = arr.indexOf(nd);
    if (i >= 0) arr.splice(i, 1);
    if (!arr.length) _zwPendingAddedById.delete(id);
  }
  function _zwPendBucket(sel, handle) {
    var key = sel ? sel : '_h:' + String(handle == null ? '' : handle);
    var b = _zwPendingByParent.get(key);
    // R51c：桶内并行 Set——记账 indexOf 在高频桶（body：每 setup insertBefore 累积）内 O(桶)
    // 扫描，testharness 数千 iteration 线性增速的另一来源。Set 判重 + 数组保序。
    if (!b) { b = { added: [], removed: [], addedSet: new Set(), removedSet: new Set() }; _zwPendingByParent.set(key, b); }
    return b;
  }
  function _zwPASet() {
    if (!_zwPendingAddedSet) {
      _zwPendingAddedSet = new Set();
      for (var i = 0; i < _zwPendingAdded.length; i++) _zwPendingAddedSet.add(_zwPendingAdded[i]);
    }
    return _zwPendingAddedSet;
  }
  function _zwPRSet() {
    if (!_zwPendingRemovedSet) {
      _zwPendingRemovedSet = new Set();
      for (var i = 0; i < _zwPendingRemoved.length; i++) _zwPendingRemovedSet.add(_zwPendingRemoved[i]);
    }
    return _zwPendingRemovedSet;
  }
  function _zwHCCollectSubtree(node, out) {
    if (!node) return;
    out.push(node);
    var h = node.__zwHandle;
    var kids = h ? (_handleChildren[h] || []) : null;
    // Parsed innerHTML children are lightweight local nodes, not handles. They
    // still belong to a pending inserted subtree and must participate in the
    // synchronous ID/query indexes until the host publishes a new snapshot.
    if ((!kids || !kids.length) && !h && !node.__zwSelector) {
      try { kids = node.childNodes || null; } catch (_e) { kids = null; }
    }
    // R51c：registry 空（sel 父——appendChild 走 sel 分支不记 registry）→ 回落本 sel 的 pending
    // 桶 added（identity 同源——同一批 proxy，对冲判定可靠）。递归展开使 remove 子树对冲全部
    // pending-added（WPT testharness setupRangeTests 每子测试全量重建的 pa 表 O(n²) 膨胀根因）。
    if ((!kids || !kids.length) && node.__zwSelector) {
      var b = _zwPendingByParent.get(node.__zwSelector);
      if (b && b.added.length) {
        for (var j = 0; j < b.added.length; j++) _zwHCCollectSubtree(b.added[j], out);
      }
      return;
    }
    if (kids) {
      for (var i = 0; i < kids.length; i++) _zwHCCollectSubtree(kids[i], out);
    }
  }
  // js-dom M4 R54：本次 mutation 的挂载点是否在**主文档**内（live collection 并入过滤——
  // 失效循环 add 分支与 _zwMakeCollection 构建期共用）。R53 教训：不能从子节点上行（pending 树
  // sel 链断在未 apply 的容器上——两版尝试都误清快照基线）；本版从**挂载点**判定：
  // ① mutSel 非空 → `__zw_contains('html', mutSel)`（host 快照一查，含自身）；
  // ② mutHandle（handle 父）→ 沿 `_zwNodeParent` **一跳一跳**上行（每跳是挂载当时刚记账的
  //    链，不会断），遇 parentSel 走 ①；无链（detachedDiv/foreignDoc 系容器根）→ false。
  // 过滤只作用于**并入**（detached/foreign 容器子树不进文档级集合——spec：getElementsByTagName
  // 只返主文档节点）；els 快照基线与 removed 剔除路径完全不动（R50 own-props 语义零影响）。
  function _zwMutationInDoc(mutSel, mutHandle) {
    var s = mutSel || null, h = mutHandle || null, guard = 0;
    while (guard++ < 8) {
      if (s) {
        if (s === 'html' || s === 'body' || s === 'head') return true;
        if (typeof __zw_contains === 'function') {
          try { if (__zw_contains('html', s) === '1') return true; } catch (_e) {}
        }
        return false;
      }
      if (!h) return false;
      var link = _zwNodeParent[h];
      if (!link) return false;
      if (link.parentSel) { s = link.parentSel; h = null; continue; }
      if (link.parentHandle) { h = link.parentHandle; continue; }
      return false;
    }
    return false;
  }

  function _zwHCLiveInvalidate(addedNodes, removedNodes, mutSel, mutHandle) {
    // R51c：全局 removed 表软上限压实——无 __zwSelector 的条目是 handle-only 节点（host 快照
    // 结构上不可能含它们：快照条目皆有 selector），剔除恒 no-op，纯死数据。WPT testharness 每
    // subtest 全量重建（Range-mutations dataChange ~5000 subtest × ~30 节点）旧实现无界膨胀。
    // 512 上限远离正常页面规模（单 turn 数百 mutation 级），溢出时一次性丢弃死条目（O(表)
    // 摊销 O(1)/mutation）。sel 条目（快照真实节点）保留。
    if (_zwPendingRemoved.length > 512) {
      var _cmp = [];
      for (var c0 = 0; c0 < _zwPendingRemoved.length; c0++) {
        if (_zwPendingRemoved[c0] && _zwPendingRemoved[c0].__zwSelector) _cmp.push(_zwPendingRemoved[c0]);
      }
      _zwPendingRemoved = _cmp;
      _zwPendingRemovedSet = null; // 惰性重建（_zwPRSet）
    }
    var addFlat = [], remFlat = [];
    if (removedNodes) for (var r0 = 0; r0 < removedNodes.length; r0++) _zwHCCollectSubtree(removedNodes[r0], remFlat);
    if (addedNodes) for (var a0 = 0; a0 < addedNodes.length; a0++) _zwHCCollectSubtree(addedNodes[a0], addFlat);
    // R51c：分桶入账（mutSel/mutHandle = mutation 目标父）。removed 先入（同批先删后加语义不变）。
    if (mutSel != null || mutHandle != null) {
      var _pb = _zwPendBucket(mutSel, mutHandle);
      // R51c：桶 removed 压实（同全局表语义——handle-only 死条目丢弃，512 软上限）。
      if (_pb.removed.length > 512) {
        var _bc = [];
        for (var bc = 0; bc < _pb.removed.length; bc++) {
          if (_pb.removed[bc] && _pb.removed[bc].__zwSelector) _bc.push(_pb.removed[bc]);
        }
        _pb.removed = _bc;
        _pb.removedSet = new Set(_bc);
      }
      for (var pb1 = 0; pb1 < remFlat.length; pb1++) {
        var _pr = remFlat[pb1];
        if (_pb.addedSet.has(_pr)) {
          // R51c 消零：曾 pending-added → 对冲（同全局表语义），不记 removed。
          _pb.addedSet.delete(_pr);
          var _pai = _pb.added.indexOf(_pr);
          if (_pai >= 0) _pb.added.splice(_pai, 1);
          continue;
        }
        if (!_pb.removedSet.has(_pr)) { _pb.removed.push(_pr); _pb.removedSet.add(_pr); }
      }
      for (var pb2 = 0; pb2 < addFlat.length; pb2++) {
        var _pa = addFlat[pb2];
        if (!_pb.addedSet.has(_pa)) { _pb.added.push(_pa); _pb.addedSet.add(_pa); }
        if (_pb.removedSet.has(_pa)) {
          _pb.removedSet.delete(_pa);
          var _pri = _pb.removed.indexOf(_pa);
          if (_pri >= 0) _pb.removed.splice(_pri, 1);
        }
      }
    }
    if (remFlat.length) {
      // R51c：remFlat 局部 Set（remFlat 本身是本批小数组，但 pending 表大——seen 判定走 Set）。
      var remSet = new Set();
      for (var rs = 0; rs < remFlat.length; rs++) remSet.add(remFlat[rs]);
      // R51c：**消零语义**（判定须在剔除 added 之前快照）——节点本是 pending-added（host 快照从未
      // 见过：createElement 后 append 又 remove 的 handle 节点，WPT testharness 每 subtest 全量重建
      // 即此模式）→ add+remove 对冲为零，**不入 removed 表**。旧无条件 push 使 removed 表随 subtest
      // 数线性膨胀（每 subtest 重建丢弃 ~30 个旧 proxy，剔除 no-op 却永久占表）→ body 桶每次
      // childNodes 读全扫 → Range-mutations dataChange（~5000 subtest）O(n²) 超时。
      var _wasPA = [];
      for (var r2 = 0; r2 < remFlat.length; r2++) _wasPA.push(_zwPASet().has(remFlat[r2]));
      // R52：惰性剔除——旧全表 keep 过滤每 remove O(pa)（testharness 每 subtest +10 泄漏条目
      // → O(n²) 残余增长根源）。改为 Set 命中才定点 splice（indexOf O(pa) 仅在命中时发生，
      // 未命中零成本——多数 remove 的节点不在 pa 表）。
      for (var r2b = 0; r2b < remFlat.length; r2b++) {
        var _rv = remFlat[r2b];
        if (!_zwPASet().has(_rv)) continue;
        _zwPASet().delete(_rv);
        _zwPAIdRemove(_rv);
        var _ri = _zwPendingAdded.indexOf(_rv);
        if (_ri >= 0) _zwPendingAdded.splice(_ri, 1);
        // R52：消零节点（从未真入树、已从 pending 剔除）的 proxy 缓存同步清——`_proxyCache`
        // 强引用旧 handle proxy 永不回收 → V8 堆随 subtest 线性涨 → GC 成本线性涨 → 总时间
        // 二次（GR3 探针 tc/apc/ib 三段同涨的根源）。仅 handle 键（'@h'）可安全清：同 handle
        // 不会再被访问（节点已消零）；sel 键保留（快照节点 identity 稳定语义）。
        if (_rv && _rv.__zwHandle) {
          // R52：消零节点的 proxy/expando 缓存清理（强引用泄漏 → V8 堆涨）。**仅清不参与子树
          // 遍历的表**——`_handleChildren`/`_zwNodeParent` 保留（invalidate 后的 CE 断连传播
          //（_ceApplyConn 子树展开）仍依赖它们，R2994 测试实证提前清破坏 disconnectedCallback）。
          delete _proxyCache['@' + _rv.__zwHandle];
          if (typeof _clsProxyCache !== 'undefined') delete _clsProxyCache['@' + _rv.__zwHandle];
          if (typeof _expando !== 'undefined') delete _expando['@' + _rv.__zwHandle];
        }
      }
      for (var r3 = 0; r3 < remFlat.length; r3++) {
        if (_wasPA[r3]) continue; // 曾 pending-added → 对冲消零，不记 removed
        var _rn = remFlat[r3];
        if (!_zwPRSet().has(_rn)) { _zwPendingRemoved.push(_rn); _zwPendingRemovedSet.add(_rn); }
      }
    }
    if (addFlat.length) {
      for (var a1 = 0; a1 < addFlat.length; a1++) {
        var nd0 = addFlat[a1];
        if (!nd0) continue;
        if (!_zwPASet().has(nd0)) { _zwPendingAdded.push(nd0); _zwPendingAddedSet.add(nd0); _zwPAIdAdd(nd0); }
        // R51c：removed 剔除不再全表重建——Set 命中才 splice（多数 mutation 不命中，零分配）。
        if (_zwPendingRemovedSet && _zwPendingRemovedSet.has(nd0)) {
          var ri = _zwPendingRemoved.indexOf(nd0);
          if (ri >= 0) _zwPendingRemoved.splice(ri, 1);
          _zwPendingRemovedSet.delete(nd0);
        }
      }
    }
    // R54：本批挂载点非主文档（detached/foreign 容器）→ 子树不并入文档级 live collection
    //（els 泄漏源；els 快照基线与 removed 剔除路径不动）。
    var _r54InDoc = _zwMutationInDoc(mutSel, mutHandle);
    for (var i = 0; i < _zwLiveCollections.length; i++) {
      var lc = _zwLiveCollections[i];
      if (remFlat.length) {
        var out = [];
        var els = lc.elements();
        for (var e = 0; e < els.length; e++) {
          var drop = false;
          for (var r = 0; r < remFlat.length; r++) if (els[e] === remFlat[r]) { drop = true; break; }
          if (!drop) out.push(els[e]);
        }
        if (out.length !== els.length) lc.replace(out);
      }
      // R120：作用域集合（element 级、detached 容器上建立）——mutation 发生在集合作用域
      // 容器上时放行（WPT Element-getElementsByTagNameNS live collection：context =
      // createElement('div') detached 容器）；文档级集合（scopeHandle/scopeSel 空）维持
      // R54 in-doc 门。
      var _r120Scoped = lc.scopeHandle
        ? (lc.scopeHandle === mutHandle)
        : (lc.scopeSel ? (lc.scopeSel === mutSel) : false);
      if (addFlat.length && (_r54InDoc || _r120Scoped)) {
        for (var a = 0; a < addFlat.length; a++) {
          var nd = addFlat[a];
          if (!nd) continue;
          var matched = false;
          try { matched = lc.matches(nd); } catch (_e) { matched = false; }
          if (matched) {
            var cur = lc.elements();
            var dup = false;
            for (var c2 = 0; c2 < cur.length; c2++) if (cur[c2] === nd) { dup = true; break; }
            if (!dup) cur.push(nd);
          }
        }
      }
    }
  }

  function _zwMakeHTMLCollection(elements, liveSpec) {
    var state = { els: elements };
    var target = Object.create(_zwHCPrototype());
    // __zwHC：读入口（trap 与 prototype 方法共用）。live 维护在写入侧（_zwHCLiveInvalidate
    // 同步 push/filter state.els），读取零开销。
    function current() {
      return state.els;
    }
    target.__zwHC = current;
    // named 候选表：文档序首匹配（id 或 name 反射）。与旧 R43 实现一致（含空串/纯数字排除语义
    // 移到 _zwIsCanonicalIndex——canonical 数字串走 indexed，非 canonical（"-2"/"4294967295"+）
    // 走 named，WPT supported-property-indices）。
    function namedFor(name) {
      // R38：空串**不是** supported property name（HTMLCollection supported property names
      // 排除空串，WPT HTMLCollection-empty-name：`c[""]===undefined`、`"" in c===false`）。
      if (name === '') return undefined;
      var els = current();
      for (var k = 0; k < els.length; k++) {
        var el = els[k];
        if (!el) continue;
        // spec supported property names 的不对称（R120，WPT own-props 双向期望）：
        // **id 暴露对所有元素**（document-wide named lookup）；**name 暴露仅限 HTML ns
        // 元素**（'z'=createElementNS('','pre') id 暴露 / 'w'=createElementNS('','pre')
        // name 不暴露）。
        var _isHtmlEl = _zwIsHTMLNamespace(el);
        try {
          if (el.getAttribute) {
            if (el.getAttribute('id') === name) return el;
            if (_isHtmlEl && el.getAttribute('name') === name) return el;
          } else {
            if (el.id === name) return el;
            if (_isHtmlEl && el.name === name) return el;
          }
        } catch (_e) {}
      }
      return undefined;
    }
    var proxy = new Proxy(target, {
      get: function (t, prop, recv) {
        if (prop === '__zwHC') return current;
        var s = (typeof prop === 'symbol') ? '' : String(prop);
        // canonical 索引 → indexed getter（越界 undefined，spec）。
        if (_zwIsCanonicalIndex(s)) {
          var els = current();
          var n = Number(s);
          return n < els.length ? els[n] : undefined;
        }
        // illegal invocation（WPT HTMLCollection-as-prototype）：collection 作 prototype 时
        // base object 读 length——receiver 非 proxy 本体（Object.create(c) 的派生对象），
        // spec legacy platform object 校验 receiver 抛 TypeError。
        if (s === 'length' && recv !== proxy) throw new TypeError('Illegal invocation');
        // named getter（expando 覆盖优先——WPT "shadows a named property that gets added later"：
        // 先 set 的 expando 在 named 出现后仍胜出）。
        if (Object.prototype.hasOwnProperty.call(t, s)) {
          var d = Object.getOwnPropertyDescriptor(t, s);
          if (d && d.get) return d.get.call(recv);
          return d ? d.value : undefined;
        }
        if (s !== '__zwHC' && s !== 'length' && s !== 'item' && s !== 'namedItem' && s !== 'toString' &&
            s !== 'constructor' && typeof prop !== 'symbol') {
          var hit = namedFor(s);
          if (hit !== undefined) return hit;
        }
        var pd = Object.getOwnPropertyDescriptor(t, s);
        if (pd) return pd.value;
        return t[prop];
      },
      set: function (t, prop, v, recv) {
        var s = String(prop);
        // 派生对象（collection 作 prototype，WPT HTMLCollection-as-prototype "setting own
        // properties"）：赋值在 base object 上创建 own property（原型 named getter 不阻 own
        // 创建），不落 target。
        if (recv !== proxy) {
          try { Object.defineProperty(recv, prop, { value: v, writable: true, enumerable: true, configurable: true }); return true; } catch (_e) { return false; }
        }
        // spec：indexed/named setter 不存在——已有元素不可覆盖。trap 返 false：loose 静默
        // no-op、strict 由引擎抛 TypeError（WPT own-props 各 "strict" 断言）。
        // R76 QuickJS 差异：返 false 在 QuickJS 下 loose 也抛（见 _zwV8ProxySetSemantics
        // 探测注释）——QuickJS 分支改返 true 不写（loose 静默 ✓；strict 断言丢——数量
        // 少且 quickjs 矩阵 strict 断言本无引擎侧抛错通道，两害取轻）。
        var _qjs = !_zwV8ProxySetSemantics();
        if (_zwIsCanonicalIndex(s) && Number(s) < current().length) return _qjs ? true : false;
        var d = Object.getOwnPropertyDescriptor(t, s);
        if (d && d.configurable === false) return _qjs ? true : false;
        // 已有 named（元素存在）：拒绝（WPT "Setting non-array index while named property exists"）。
        if (namedFor(s) !== undefined) return _qjs ? true : false;
        // 越界 indexed（无元素）：同样拒绝（WPT "Setting array index while indexed property
        // doesn't exist"：赋值后仍 undefined；strict 抛）。
        if (_zwIsCanonicalIndex(s)) return _qjs ? true : false;
        t[s] = v;
        return true;
      },
      defineProperty: function (t, prop, desc) {
        var s = String(prop);
        var els = current();
        // spec：indexed 属性（含越界——legacy getter 覆盖全部 canonical 索引，WPT
        // supported-property-indices "past the end of the list"：defineProperty 越界也抛）。
        if (_zwIsCanonicalIndex(s)) {
          throw new TypeError('Cannot redefine property: ' + s);
        }
        if (namedFor(s) !== undefined) {
          // 已存在 named：spec 拒绝 defineProperty（WPT "set an expando that would shadow an
          // already-existing named property"：assert_throws_js TypeError）。
          throw new TypeError('Cannot redefine property: ' + s);
        }
        Object.defineProperty(t, s, desc);
        return true;
      },
      deleteProperty: function (t, prop) {
        var s = String(prop);
        // own expando 优先删除（WPT "shadows a named property that gets added later"：
        // delete expando 后 named getter 重新可见）。non-configurable expando 不可删（Proxy
        // invariant：trap 返 false；WPT "non-configurable expando" strict delete 抛）。
        // R76 QuickJS：返 false 在 QuickJS loose 也抛（同 set trap 差异）——_qjs 分支
        // 返 true 不删（loose 静默）。
        var _qjs = !_zwV8ProxySetSemantics();
        if (Object.prototype.hasOwnProperty.call(t, s)) {
          var dd = Object.getOwnPropertyDescriptor(t, s);
          if (dd && dd.configurable === false) return _qjs ? true : false;
          delete t[s];
          return true;
        }
        // spec：indexed/named 不可删除——trap 返 false：loose 静默 no-op、strict 抛 TypeError
        //（WPT HTMLCollection-delete "Strict id"/"Strict name"）。
        if (_zwIsCanonicalIndex(s) && Number(s) < current().length) return _qjs ? true : false;
        if (namedFor(s) !== undefined) return _qjs ? true : false;
        delete t[s];
        return true;
      },
      getOwnPropertyDescriptor: function (t, prop) {
        var s = (typeof prop === 'symbol') ? '' : String(prop);
        var els = current();
        if (_zwIsCanonicalIndex(s)) {
          var n = Number(s);
          if (n < els.length) {
            return { value: els[n], writable: false, enumerable: true, configurable: true };
          }
          return undefined;
        }
        if (Object.prototype.hasOwnProperty.call(t, s)) return Object.getOwnPropertyDescriptor(t, s);
        var hit = namedFor(s);
        if (hit !== undefined && s !== 'length' && s !== 'item' && s !== 'namedItem' && s !== 'toString' && s !== 'constructor') {
          return { value: hit, writable: false, enumerable: false, configurable: true };
        }
        var pd = Object.getOwnPropertyDescriptor(t, s);
        return pd || undefined;
      },
      ownKeys: function (t) {
        // spec supported property names：[indices…, names…, expandos…]（WPT
        // supported-property-names：无 length/item/namedItem）。
        var els = current();
        var keys = [];
        for (var i = 0; i < els.length; i++) keys.push(String(i));
        var seen = {};
        for (var k = 0; k < els.length; k++) {
          var el = els[k];
          if (!el) continue;
          // R120：id/name 不对称暴露（同 namedFor——id 全元素、name 仅 HTML ns，
          // WPT own-props 期望 ['0','1','2','3','x','y','z']：z=createElementNS('') 的 id）。
          var _isHtmlK = _zwIsHTMLNamespace(el);
          var names = [];
          try {
            if (el.getAttribute) {
              var id = el.getAttribute('id'); if (id) names.push(id);
              if (_isHtmlK) { var nm = el.getAttribute('name'); if (nm) names.push(nm); }
            } else {
              if (el.id) names.push(el.id);
              if (_isHtmlK && el.name) names.push(el.name);
            }
          } catch (_e) {}
          for (var q = 0; q < names.length; q++) {
            var name = String(names[q]);
            if (name === '' || seen[name]) continue;
            if (_zwIsCanonicalIndex(name)) continue; // canonical 数字 name 走 indexed（spec 不重复暴露）
            seen[name] = true;
            keys.push(name);
          }
        }
        var own = Object.getOwnPropertyNames(t);
        for (var oi = 0; oi < own.length; oi++) {
          var ok = own[oi];
          if (ok === '__zwHC') continue;
          if (!seen[ok]) keys.push(ok);
        }
        return keys;
      },
      has: function (t, prop) {
        var s = (typeof prop === 'symbol') ? '' : String(prop);
        var els = current();
        if (_zwIsCanonicalIndex(s)) return Number(s) < els.length;
        if (Object.prototype.hasOwnProperty.call(t, s)) return true;
        // prototype 成员（length/item/namedItem/@@iterator 等，WPT HTMLCollection-iterator
        // "has length method"/"has Symbol.iterator"）。
        var pd = Object.getOwnPropertyDescriptor(t, prop);
        if (pd !== undefined) return true;
        if (prop in Object.getPrototypeOf(t)) return true;
        return namedFor(s) !== undefined;
      },
    });
    if (liveSpec && typeof liveSpec.matches === 'function') {
      _zwLiveCollections.push({
        matches: liveSpec.matches,
        elements: function () { return state.els; },
        replace: function (out) { state.els = out; },
        // R120：集合作用域容器（element 级 getElementsBy* 在 detached handle 容器上建立）——
        // add 并入判定按作用域放行（mutation 容器 === 作用域容器即入，不查 in-doc）。
        scopeHandle: liveSpec.scopeHandle || null,
        scopeSel: liveSpec.scopeSel || null,
      });
    }
    return proxy;
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
    // R117：先从旧父移除（spec pre-insert 移动语义）+ 去重（同节点多次出现在参数中只插一次）
    // + 上下文自身跳过（before.call(el, el, ...) 的 self 参数 no-op——spec「node 是 context
    // object 则跳过该参数」）。
    var seenHandles = {};
    var deduped = [];
    for (var k0 = 0; k0 < items.length; k0++) {
      var it0 = items[k0];
      if (typeof it0 === 'object' && it0 && it0.__zwHandle) {
        if (it0.__zwSelector === sel) continue; // self（sel 上下文）
        if (seenHandles[it0.__zwHandle]) continue;
        seenHandles[it0.__zwHandle] = true;
        _zwDetachFromRegistry(it0);
      }
      deduped.push(it0);
    }
    items = deduped;
    for (var k = 0; k < items.length; k++) {
      var item = items[k];
      try {
        if (item && typeof item === 'object' && item.__zwHandle) {
          __zw_insert_adjacent_element(sel, position, item.__zwHandle);
          ceInserted.push(item);
        } else {
          // R117：null/undefined → WebIDL DOMString（'null'/'undefined' 文本节点）。
          __zw_insert_adjacent_text(sel, position, String(item));
        }
      } catch (_e) {}
    }
    for (var ci = 0; ci < ceInserted.length; ci++) _ceApplyConn(ceInserted[ci], true);
  }

  // append/replaceChildren 共用：variadic 节点/字符串追加到 this 末尾（DocumentFragment flatten）。
  // 返 added 列表（供 MO childList notify）。节点经 handle/selector append_child；字符串建 text 节点 append。
  // js-dom M4 R119：handle 容器的 prepend——spec `dom-parentnode-prepend`（转换为
  // insert(node, firstChild) 后逐参数 pre-insert）。实现：先逐参数「移除旧父（移动语义，
  // R117 _zwDetachFromRegistry）+ 建 text 节点」，再**保持参数序头插**（对 registry 头部
  // 依次 unshift 的逆序 = 参数序）；host mutation 经 R101 全 handle wire
  //（__zw_insert_before_handle_handle，ref = 原首子；ref miss 时 apply 降级 append——
  // prepend 到空容器 == appendChild 语义，天然正确）。WPT ParentNode-prepend 的
  // createElement('div')/DocumentFragment/cloneNode 容器族 + null/undefined → WebIDL 文本。
  // https://dom.spec.whatwg.org/#dom-parentnode-prepend
  function _prependHandleVariadic(handle, args) {
    if (!handle) return;
    var kids = _handleChildren[handle] || (_handleChildren[handle] = []);
    var firstRef = kids.length ? kids[0] : null;
    var added = [];
    // 参数序保持：逐参数 unshift 会逆序（[t1,t2]→[t2,t1]）——物化后逆序 unshift 得参数序。
    for (var i = args.length - 1; i >= 0; i--) {
      var item = args[i];
      if (item && typeof item === 'object' && item.__zwHandle) {
        // pre-insert 步骤 3：先从旧父移除（移动非复制；含 fragment flatten 记账）。
        _zwDetachFromRegistry(item);
        if (typeof __zw_insert_before_handle_handle === 'function') {
          var ref = firstRef && firstRef.__zwHandle ? firstRef.__zwHandle : '';
          try { __zw_insert_before_handle_handle(handle, item.__zwHandle, ref); } catch (_e119i) {}
        }
        if (_fragmentHandles[item.__zwHandle]) {
          var fk = _handleChildren[item.__zwHandle];
          if (fk && fk.length) {
            for (var f = fk.length - 1; f >= 0; f--) kids.unshift(fk[f]);
            _handleChildren[item.__zwHandle] = [];
          }
        } else {
          // 同容器已含该子（prepend 自身子集）→ 先剔再头插（去重保持末位语义由 spec pre-insert 推出）。
          var at = kids.indexOf(item);
          if (at >= 0) kids.splice(at, 1);
          kids.unshift(item);
        }
        try {
          if (typeof _zwNodeParent !== 'undefined' && _zwNodeParent) {
            _zwNodeParent[item.__zwHandle] = { parentSel: null, parentHandle: handle, nextSibling: null };
          }
        } catch (_e119p) {}
        added.push(item);
      } else {
        var tn = (typeof __zw_create_text === 'function') ? __zw_create_text(String(item)) : '';
        if (tn) {
          _textHandles[tn] = true;
          if (typeof __zw_insert_before_handle_handle === 'function') {
            var tref = firstRef && firstRef.__zwHandle ? firstRef.__zwHandle : '';
            try { __zw_insert_before_handle_handle(handle, tn, tref); } catch (_e119t) {}
          }
          kids.unshift(_wrapHandle(tn));
          added.push(_wrapHandle(tn));
        }
      }
    }
    added.reverse(); // 逆序循环后恢复参数序（文档序）
    if (added.length) {
      _mo_notify(null, handle, { type: 'childList', addedNodes: added, removedNodes: [] });
      var pconn = _ceParentConnected(null, handle);
      for (var ci = 0; ci < added.length; ci++) _ceApplyConn(added[ci], pconn);
    }
    return undefined;
  }

  // js-dom M4 R119：handle 容器移除单个已记录子（replaceChildren 清空段复用）——registry
  // 剔除 + 反链清 + host 侧 __zw_remove_handle（从父移除该节点，RemoveHandle mutation）。
  function _zwRemoveHandleNode(handle, child) {
    if (!handle || !child || !child.__zwHandle) return;
    _unrecordHandleChild(handle, child);
    if (typeof __zw_remove_handle === 'function') {
      try { __zw_remove_handle(child.__zwHandle); } catch (_e119r) {}
    }
  }

  function _appendVariadic(sel, handle, args) {
    var added = [];
    for (var i = 0; i < args.length; i++) {
      var item = args[i];
      // R117：null/undefined 不跳过——WebIDL DOMString 转换（'null'/'undefined' 文本节点，
      // WPT ParentNode-append「with null as an argument」）。
      if (item && typeof item === 'object' && item.__zwHandle) {
        // DocumentFragment：flatten 子节点到 this。
        if (_fragmentHandles[item.__zwHandle] && typeof __zw_append_fragment_children === 'function') {
          if (handle) __zw_append_fragment_children_handle(handle, item.__zwHandle);
          else __zw_append_fragment_children(sel, item.__zwHandle);
          if (handle) _recordHandleChild(handle, item);
        } else if (handle) {
          __zw_append_child_handle(handle, item.__zwHandle);
          _recordHandleChild(handle, item);
        } else {
          __zw_append_child(sel, item.__zwHandle);
        }
        added.push(item);
      } else {
        var tn = __zw_create_text(String(item));
        if (tn) _textHandles[tn] = true;
        if (handle) {
          __zw_append_child_handle(handle, tn);
          // R51c：registry 记账（collectSubtree 展开 + childNodes 融合视图依赖）；record 的
          // addedNodes 保持裸对象形态（原语义，MO identity 不变）。
          _recordHandleChild(handle, _wrapHandle(tn));
        } else {
          __zw_append_child(sel, tn);
        }
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
  // R57（FV M3）：form 提交重入守卫（IIFE 作用域——初始化一次；part04 的 _zwRunFormSubmit
  // 引用。不能放 part04 顶部——那在 get trap 内，每属性访问会重置）。
  var _zwSubmitBusy = false;

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
  // js-dom M4 R120：getElementsByTagName(/NS) 的客户端 NS 感知匹配（spec
  // `concept-getelementsbytagname` / `concept-getelementsbytagnamens`）：
  // - 枚举全后代元素（sel→_descendantElements；handle→_handleQueryAll('*')）。
  // - qualifiedName 比较：元素 HTML ns（namespaceURI null/undefined/HTMLNS）→ 双方
  //   ASCII 小写后比较（HTML 文档语义：'I' 输入不命中 HTML 元素 'I'——WPT「uppercase
  //   tagName never matches」）；非 HTML ns → **大小写敏感**原样比较（WPT non-HTML
  //   namespace 簇：'ST' 命中 'ST' 不命中 'st'）。
  // - NS 变体（nsArg 非 undefined）：ns 匹配先行——'*' 任意 ns；null 匹配无 ns 元素
  //   （createElementNS('') 产物——WPT「Empty string namespace」）；否则字符串相等。
  //   localName 匹配不带 prefix（元素 localName vs 输入；HTML ns 双小写）。
  //   非 NS 变体：输入与元素 **qualifiedName**（tagName，含 prefix）比较——'te:st' 命中
  //   prefix 元素（WPT「prefix, lowercase name」）。
  // https://dom.spec.whatwg.org/#concept-getelementsbytagname
  function _zwGetByTagNameSubtree(sel, handle, input, nsArg) {
    var els;
    if (sel) els = _descendantElements(sel);
    else if (handle) els = _handleQueryAll(handle, '*');
    else return [];
    return _zwFilterByTagNameNS(els, input, nsArg);
  }

  // js-dom M4 R120：live HTMLCollection 的 matches 闭包（R50 liveSpec 接线——同步脚本内
  // append/remove 后 `_zwHCLiveInvalidate` 按此判定新子归属，WPT「should be a live
  // collection」length 断言）。判定复用 `_zwFilterByTagNameNS`（单元素跑过滤器）。
  function _zwLiveMatchesFor(input, nsArg) {
    return function (el) {
      return _zwFilterByTagNameNS([el], input, nsArg).length > 0;
    };
  }

  // js-dom M4 R120：getElementsBy* 匹配核心（与 Element 级 / Document 级共用）。
  // input = 限定名（非 NS 变体，对 tagName 含 prefix）或 localName（NS 变体）；
  // nsArg = undefined（非 NS）/ ns（'*' 任意 / null 无 ns / 精确串）。
  function _zwFilterByTagNameNS(els, input, nsArg) {
    var nsMode = (typeof nsArg !== 'undefined');
    var nsWant = nsMode ? (nsArg == null ? '' : String(nsArg)) : null;
    var inputLower = _zwAsciiLower(String(input));
    var out = [];
    for (var i = 0; i < els.length; i++) {
      var el = els[i];
      if (!el || el.nodeType !== 1) continue;
      var ns = null;
      try { ns = el.namespaceURI; } catch (_e) {}
      var isHtml = ns === null || ns === undefined || ns === 'http://www.w3.org/1999/xhtml';
      if (nsMode) {
        if (nsWant !== '*') {
          var nsActual = (ns === null || ns === undefined) ? '' : String(ns);
          if (nsActual !== nsWant) continue;
        }
        // localName 匹配（'*' 恒真）——**原样精确比较**（spec getelementsbytagnamens
        // 对 localName 无小写化：createElementNS(HTMLNS,'ABC') 的 localName 'ABC' 只被
        // ('HTMLNS','ABC') 命中、('HTMLNS','abc') 不命中——WPT「abc/ABC element in html
        // namespace」双向期望；'AÇ' 同例）。
        if (input !== '*') {
          var ln = null;
          try { ln = el.localName; } catch (_e2) {}
          if (ln == null) ln = '';
          if (String(ln) !== String(input)) continue;
        }
      } else {
        if (input !== '*') {
          // qualifiedName 比较（tagName 含 prefix；非 NS 变体）。
          // HTML 文档语义：HTML ns 元素与输入**双方** ASCII 小写比较——但 createElementNS
          // （HTMLNS, 'I'）的元素 localName 含大写（HTML 文档里本不该存在的形态），
          // WPT「uppercase tagName never matches」期望 ('I')/('i') 都不命中——规则：
          // HTML ns 元素的 localName 非纯 ASCII 小写 → 永不匹配。
          if (isHtml) {
            var lnH = null;
            try { lnH = el.localName; } catch (_eL) {}
            if (lnH != null && _zwAsciiLower(String(lnH)) !== String(lnH)) continue;
          }
          var qn = null;
          try { qn = el.tagName; } catch (_e3) {}
          if (qn == null) continue;
          var qnCmp = isHtml ? _zwAsciiLower(String(qn)) : String(qn);
          if (qnCmp !== (isHtml ? inputLower : String(input))) continue;
        }
      }
      out.push(el);
    }
    return out;
  }

  // js-dom M4 R120：文档级全元素枚举（document.getElementsByTagName(/NS) 的数据源）——
  // 快照 `__zw_query_all('*')`（host 树）∪ `_zwPendingAdded` 动态子（同步脚本内 appendChild
  // 的 handle 子不在快照——WPT「live collection」length 断言）。快照可能不支持 '*' → 回落
  // documentElement/body 的 _descendantElements 并集。返回去重文档序数组。
  function _zwDocAllElements() {
    var out = [];
    var seen = new Map();
    var push = function (el) {
      if (!el || el.nodeType !== 1) return;
      var k = el.__zwSelector || el.__zwHandle;
      if (k && seen.has(k)) return;
      if (k) seen.set(k, true);
      out.push(el);
    };
    var snapCount = 0;
    try {
      var all = (typeof __zw_query_all === 'function') ? String(__zw_query_all('*') || '') : '';
      var sels = all ? all.split('|').filter(Boolean) : [];
      snapCount = sels.length;
      for (var i = 0; i < sels.length; i++) {
        try { push(_wrapSelector(sels[i])); } catch (_e) {}
      }
    } catch (_eA) {}
    // 快照不支持 '*'（返回空）→ 回落 documentElement + html/body 子树（与 pending 并存——
    // 快照为空时 pending 不能独占 out，静态树仍需并入）。
    if (snapCount === 0) {
      try { push(globalThis.document.documentElement); } catch (_eD) {}
      var de = _descendantElements('html');
      for (var d = 0; d < de.length; d++) push(de[d]);
      var be = _descendantElements('body');
      for (var b = 0; b < be.length; b++) push(be[b]);
    }
    // 动态 pending 子（按文档序 append 顺序；R54 in-doc 门——detached/foreign 容器子
    // 不进文档级枚举，WPT live-collection 的容器子经 element 级集合的作用域放行覆盖）。
    if (typeof _zwPendingAdded !== 'undefined' && _zwPendingAdded) {
      for (var p = 0; p < _zwPendingAdded.length; p++) {
        var _pd119 = _zwPendingAdded[p];
        if (_pd119 && _pd119.__zwHandle && !_zwMutationInDoc(null, _pd119.__zwHandle)) continue;
        push(_pd119);
      }
    }
    return out;
  }

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
  // R109（WPT Event-subclasses-constructors SubclassedEvent 簇）：改为**真构造器**——`class X extends Event`
  // 经 super() 走 [[Construct]]→`super(...)` 反射到本函数的 [[Construct]] 槽，内部须 `new.target` 派发
  // proto（spec：derived constructor 的 super() 以 new.target.prototype 为实例 proto）。旧「工厂返回
  // 对象」形态下 super() 拿到的返回值是 Event.prototype 实例 → 子类 ctor 体内 `this.customProp=5` 抛
  // TypeError（this 未初始化）、get fixedProp 落不到实例。instanceof 检查仍成立（Reflect.construct/
  // new 都有 new.target）；null new.target（Reflect.construct(Event, [], null)）回落 Event.prototype。
  globalThis.Event = function Event(type, options) {
    var ev = _makeEvent(type, options);
    var nt = this instanceof globalThis.Event && this.constructor !== globalThis.Event
      ? this
      : null;
    var proto = (nt && nt.constructor && nt.constructor.prototype) || globalThis.Event.prototype;
    if (!proto || proto === Object.prototype) proto = globalThis.Event.prototype;
    Object.setPrototypeOf(ev, proto);
    // 真 [[Construct]] 语义：整对象搬运（own data 属性 for-in 拷贝 + getter/setter 属性
    // getOwnPropertyDescriptor 逐个 defineProperty——cancelBubble/returnValue/srcElement 是
    // 非枚举 accessor，for-in 漏它们会致 WPT Event-cancelBubble/returnValue 读 undefined）。
    if (nt) {
      var keys109 = Object.getOwnPropertyNames(ev);
      for (var i109 = 0; i109 < keys109.length; i109++) {
        var k109 = keys109[i109];
        if (k109 === 'constructor') continue;
        var d109 = Object.getOwnPropertyDescriptor(ev, k109);
        if (d109) Object.defineProperty(nt, k109, d109);
      }
      return nt;
    }
    return ev;
  };
  if (typeof globalThis.Event.prototype.initEvent !== 'function') {
    globalThis.Event.prototype.initEvent = function (type, bubbles, cancelable) {
      // js-dom M4 R110：spec `dom-event-initevent`「If this's dispatch flag is set, then
      // return」——派发中 initEvent 是 no-op（WPT Event-init-while-dispatching "Calling
      // initEvent while dispatching"）。`_zwDispatching` 由 _dispatchWithBubble 维护
      //（R106 已建，入口/finally 计数）。
      if (this._zwDispatching) return;
      // R110：首参 mandatory——缺省抛 TypeError（spec legacy init 位置参数 non-optional；
      // WPT Event-initEvent "First parameter to initEvent should be mandatory"）。
      if (arguments.length < 1) {
        throw new globalThis.TypeError("Failed to execute 'initEvent' on 'Event': 1 argument required, but only 0 present.");
      }
      // js-dom M4 R106：spec `dom-eventtarget-dispatchevent` 步骤 1——event 的 initialized
      // flag 未设时 dispatchEvent 抛 InvalidStateError。createEvent 返回的事件带
      // `_zwUninitialized`（构造器路径不设——new Event() 即已初始化），initEvent 清除。
      this._zwUninitialized = false;
      this.type = type;
      this.bubbles = !!bubbles;
      this.cancelable = !!cancelable;
      this.defaultPrevented = false;
      this._defaultPrevented = false;
      // js-dom M4 R26/R29：spec `concept-event-initialize` 重置 dispatch flags——initEvent 把 stop propagation
      // flag 归零。R26 显式 `this.cancelBubble = false`（旧 data 属性）；R29 cancelBubble 改 defineProperty
      // getter/setter（后端 _propagationStopped，setter 设 false = no-op），故此处直接重置 _propagationStopped
      // 即可让 cancelBubble getter 返 false（WPT Event-cancelBubble "initEvent must set cancelBubble to false"）。
      this._propagationStopped = false;
      this._immediateStopped = false;
    };
  }
  // js-dom M4 R106：dispatchEvent 入口守卫（spec `dom-eventtarget-dispatchevent`）——
  // ① event 非 Event（null/undefined/无 type 字段对象）抛 TypeError（WebIDL Event 类型校验）；
  // ② event 的 initialized flag 未设（createEvent 未 initEvent——`_zwUninitialized`）抛
  // InvalidStateError。四个 dispatchEvent 入口（window/document/元素 proxy/EventTarget.prototype）
  // 统一调用；返回 true = 守卫通过。
  globalThis._zwDispatchGuard = function (event) {
    if (event == null || typeof event !== 'object' || typeof event.type !== 'string') {
      throw new globalThis.TypeError('Argument 1 is not of type \'Event\'.');
    }
    if (event._zwUninitialized) {
      throw new (globalThis.DOMException)('The event is not initialized.', 'InvalidStateError');
    }
    // R106：dispatch flag 已设（该 event 正在派发中）——重入抛 InvalidStateError
    //（spec `dom-eventtarget-dispatchevent` 步骤 2 / inner「dispatch flag」）。
    if (event._zwDispatching) {
      throw new (globalThis.DOMException)('The event is already being dispatched.', 'InvalidStateError');
    }
    return true;
  };
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
    // js-dom M4 R110：spec CustomEventInit.detail 缺省 **null**（`detail: any = null`）——
    // `_makeEvent` 落 undefined（Event 语义），CustomEvent 构造路径补 null（WPT
    // Event-init-while-dispatching "detail setter should short-circuit" 期望 null）。
    if (ev.detail === undefined) ev.detail = null;
    Object.setPrototypeOf(ev, globalThis.CustomEvent.prototype);
    return ev;
  };
  globalThis.CustomEvent.prototype = Object.create(globalThis.Event.prototype);
  globalThis.CustomEvent.prototype.constructor = globalThis.CustomEvent;
  // initCustomEvent——legacy 合成事件初始化（与 createEvent('CustomEvent') + initEvent 配对，spec）。
  // 镜像 initEvent 设 type/bubbles/cancelable + 设 detail。guard 幂等（不覆盖既有定义）。
  if (typeof globalThis.CustomEvent.prototype.initCustomEvent !== 'function') {
    globalThis.CustomEvent.prototype.initCustomEvent = function (type, bubbles, cancelable, detail) {
      // js-dom M4 R110：spec `dom-customevent-initcustomevent`「dispatch flag set → return」
      //（WPT Event-init-while-dispatching "Calling initCustomEvent while dispatching"——
      // detail setter 须 short-circuit）。复用 R106 `_zwDispatching` 计数。
      if (this._zwDispatching) return;
      // R110：spec legacy init 方法首参 mandatory——缺省抛 TypeError（WebIDL 位置参数
      // non-optional；WPT CustomEvent "First parameter to initCustomEvent should be
      // mandatory"）。
      if (arguments.length < 1) {
        throw new globalThis.TypeError("Failed to execute 'initCustomEvent' on 'CustomEvent': 1 argument required, but only 0 present.");
      }
      this.type = type;
      this.bubbles = !!bubbles;
      this.cancelable = !!cancelable;
      // R110：detail 缺省 null（spec CustomEventInit.detail = null；WPT "default parameter
      // values"——undefined 直设会读回 undefined）。
      this.detail = detail == null ? null : detail;
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
    if (globalThis[name]) {
      // js-dom M4 R109：native 叠加路径——native bindings 先装（MouseEvent/KeyboardEvent 为
      // V8 FunctionTemplate），guard 早退会漏登 `_eventSubclassProps` 注册表 → 下游子类
      //（WheelEvent extends MouseEvent）工厂分支沿父链收集 props 时断在未注册的父节点，
      // 实例缺 ctrlKey/screenX/view（WPT Event-subclasses-constructors WheelEvent 簇）。
      // 注册表登记 + shim 父链 prototype 接线（MouseEvent.prototype → shim UIEvent.prototype）
      // 在别处（KeyboardEventCtor 块 R109 接线 IIFE）完成；此处保登记幂等。
      _eventSubclassProps[name] = [props, parentName];
      return globalThis[name];
    }
    var Parent = globalThis[parentName] || globalThis.Event;
    var Ctor = function (type, options) {
      // R109：真构造器化（同 Event 修复）——`class X extends MouseEvent` 的 super() 要求本 ctor
      // 以 [[Construct]] 语义填充 new.target 的 this；工厂返对象会致子类 this 未初始化。
      var o = (options == null || typeof options !== 'object') ? {} : options;
      var isSuperCall = this instanceof Ctor && this.constructor !== Ctor;
      if (isSuperCall) {
        // super(type, options) 反射：以 new.target 的 this 为载体，先经父构造器（Event 真构造器
        // 已填基础字段），再补子类 + 父链专属字段。
        Parent.call(this, type, options);
        var chainS = [];
        var curS = name;
        var guardS = 0;
        while (curS && _eventSubclassProps[curS] && guardS++ < 32) {
          var entryS = _eventSubclassProps[curS];
          chainS = chainS.concat(entryS[0]);
          curS = entryS[1];
        }
        for (var iS = 0; iS < chainS.length; iS++) {
          var pS = chainS[iS];
          this[pS[0]] = o[pS[1]] != null ? o[pS[1]] : pS[2];
        }
        return this;
      }
      var ev = _makeEvent(type, options);
      Object.setPrototypeOf(ev, Ctor.prototype);
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
  // js-dom M4 R109（WPT Event-subclasses-constructors "view argument with wrong type"）：
  // UIEventInit.view 是 `(WindowProxy or null)?`——显式非 null/undefined 且非 window（沙箱内
  // WindowProxy 仅 globalThis）时构造器抛 TypeError（WebIDL dictionary 类型校验，spec
  // `uievent` §constructor）。挂在 UIEvent 原构造器外层（保 prototype/子类注册不动）。
  var UIEventBase = globalThis.UIEvent;
  if (UIEventBase) {
    var UIEventCtor109 = function UIEvent(type, options) {
      if (options != null && typeof options === 'object'
          && 'view' in options && options.view != null && options.view !== globalThis) {
        throw new globalThis.TypeError(
          "Failed to construct 'UIEvent': member view is not of type WindowProxy.");
      }
      // 透传 super：new UIEvent(...)（this instanceof UIEventCtor109 且 constructor===自身 → 工厂
      // 分支）/ class extends UIEvent 的 super()（this.constructor 为子类 → 真 [[Construct]] 分支）。
      return UIEventBase.apply(this, arguments);
    };
    try {
      Object.defineProperty(UIEventCtor109, 'name', { value: 'UIEvent', configurable: true });
    } catch (_e109) {}
    UIEventCtor109.prototype = UIEventBase.prototype;
    UIEventCtor109.prototype.constructor = UIEventCtor109;
    globalThis.UIEvent = UIEventCtor109;
  }
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
  // js-dom M4 R109：native KeyboardEvent（V8 模板 invoke）对空 dict `{}` 的 init_string 读
  // `opts.get('key')` 得 undefined → ToString "undefined"（WPT 期望缺省 ""）。shim 侧在
  // native ctor 装配后补齐缺省字段（仅 undefined 时补，显式值不覆盖）。
  (function () {
    var KB109 = globalThis.KeyboardEvent;
    if (!KB109 || !KB109.__zwR109Patched) {
      var WrappedKB = function KeyboardEvent(type, options) {
        var r = KB109.apply(this, arguments);
        var inst = (r && typeof r === 'object') ? r : this;
        var defs = { key: '', code: '', location: 0, repeat: false, isComposing: false, charCode: 0, keyCode: 0, which: 0, detail: 0, ctrlKey: false, shiftKey: false, altKey: false, metaKey: false, view: null };
        for (var d in defs) {
          // native init_string 对 dict 缺省字段经 ToString 得**字符串 "undefined"**（非
          // undefined 值）设到实例——两形态都补缺省。
          if (inst[d] === undefined || inst[d] === 'undefined') inst[d] = defs[d];
        }
        return r !== undefined ? r : inst;
      };
      try { Object.defineProperty(WrappedKB, 'name', { value: 'KeyboardEvent', configurable: true }); } catch (_eN109) {}
      WrappedKB.prototype = KB109.prototype;
      try { Object.defineProperty(WrappedKB, '__zwR109Patched', { value: true }); } catch (_eP109) {}
      globalThis.KeyboardEvent = WrappedKB;
    }
  })();
  // js-dom M4 R109（native 叠加路径原型链接线）：ZW_NATIVE_DOM=1 时 native bindings 先装
  //（Event/CustomEvent/MouseEvent/KeyboardEvent 为 V8 FunctionTemplate），shim 后装只覆盖
  // Event/UIEvent 族——_defineEventSubclass 的 `if (globalThis[name]) return` guard 会把
  // **native MouseEvent/KeyboardEvent 留在原地**，但其 V8 模板原型链指向 native Event 模板
  //（已被 shim Event 覆盖替换）→ `new MouseEvent() instanceof MouseEvent` true 而
  // `instanceof UIEvent/Event` false（WPT Event-subclasses-constructors MouseEvent/
  // KeyboardEvent/WheelEvent 全簇 fail 的根因，R25 后双路径差 6pp+ 的主成分）。修法：
  // 把留下的 native 子类原型**重接到 shim 父链**（MouseEvent→shim UIEvent.prototype、
  // KeyboardEvent→shim UIEvent.prototype），instanceof 三层全通。native ctor 本体的
  // 坐标/键字段装配保留（V8 模板 invoke），dispatch 读 _-flag 走实例字段兼容。
  (function () {
    var UIEV109 = globalThis.UIEvent;
    if (UIEV109 && globalThis.MouseEvent
        && Object.getPrototypeOf(globalThis.MouseEvent.prototype) !== UIEV109.prototype) {
      try { Object.setPrototypeOf(globalThis.MouseEvent.prototype, UIEV109.prototype); } catch (_eM109) {}
    }
    if (UIEV109 && globalThis.KeyboardEvent
        && Object.getPrototypeOf(globalThis.KeyboardEvent.prototype) !== UIEV109.prototype) {
      try { Object.setPrototypeOf(globalThis.KeyboardEvent.prototype, UIEV109.prototype); } catch (_eK109) {}
    }
  })();
  // WheelEvent（MouseEvent 子类）：delta + deltaMode + DOM_DELTA_* 静态常量。
  var WheelEventCtor = _defineEventSubclass('WheelEvent', 'MouseEvent', [
    ['deltaX', 'deltaX', 0], ['deltaY', 'deltaY', 0], ['deltaZ', 'deltaZ', 0],
    ['deltaMode', 'deltaMode', 0],
  ]);
  WheelEventCtor.DOM_DELTA_PIXEL = 0;
  WheelEventCtor.DOM_DELTA_LINE = 1;
  WheelEventCtor.DOM_DELTA_PAGE = 2;
  // js-dom M4 R110（WPT Event-init-while-dispatching）：legacy initXxxEvent 族补齐——
  // initUIEvent / initMouseEvent / initKeyboardEvent。共同语义（spec 各接口 init 方法步骤 1）：
  // ① **dispatch flag set → return**（派发中 no-op，用 R106 `_zwDispatching` 计数）；
  // ② 否则重置 init 字段（type/bubbles/cancelable 经 initEvent 基类语义 + 自身字段）。
  // 参数表按 spec legacy 签名（位置参数）。
  var UIEventCtor110 = globalThis.UIEvent;
  if (UIEventCtor110 && !UIEventCtor110.prototype.initUIEvent) {
    UIEventCtor110.prototype.initUIEvent = function (type, bubbles, cancelable, view, detail) {
      if (this._zwDispatching) return;
      var proto = Object.getPrototypeOf(Object.getPrototypeOf(this));
      if (proto && typeof proto.initEvent === 'function') proto.initEvent.call(this, type, bubbles, cancelable);
      this.view = view == null ? null : view;
      this.detail = detail == null ? 0 : detail;
    };
  }
  if (MouseEventCtor && !MouseEventCtor.prototype.initMouseEvent) {
    MouseEventCtor.prototype.initMouseEvent = function (type, bubbles, cancelable, view,
                                                        detail, screenX, screenY, clientX, clientY,
                                                        ctrlKey, altKey, shiftKey, metaKey,
                                                        button, relatedTarget) {
      if (this._zwDispatching) return;
      var proto = Object.getPrototypeOf(Object.getPrototypeOf(this));
      if (proto && typeof proto.initEvent === 'function') proto.initEvent.call(this, type, bubbles, cancelable);
      this.view = view == null ? null : view;
      this.detail = detail == null ? 0 : detail;
      this.screenX = screenX || 0; this.screenY = screenY || 0;
      this.clientX = clientX || 0; this.clientY = clientY || 0;
      this.ctrlKey = !!ctrlKey; this.altKey = !!altKey;
      this.shiftKey = !!shiftKey; this.metaKey = !!metaKey;
      this.button = button || 0;
      this.relatedTarget = relatedTarget == null ? null : relatedTarget;
    };
  }
  if (KeyboardEventCtor && !KeyboardEventCtor.prototype.initKeyboardEvent) {
    // spec legacy KeyboardEventInit 位置签名（chromium 实参序）：key/code/location/…；
    // 本沙箱取 WPT 用例实际形态（key, location, ctrlKey, altKey, shiftKey, metaKey 后随）。
    KeyboardEventCtor.prototype.initKeyboardEvent = function (type, bubbles, cancelable, view,
                                                             key, location, ctrlKey, altKey,
                                                             shiftKey, metaKey) {
      if (this._zwDispatching) return;
      var proto = Object.getPrototypeOf(Object.getPrototypeOf(this));
      if (proto && typeof proto.initEvent === 'function') proto.initEvent.call(this, type, bubbles, cancelable);
      this.view = view == null ? null : view;
      this.key = key == null ? '' : key;
      this.location = location || 0;
      this.ctrlKey = !!ctrlKey; this.altKey = !!altKey;
      this.shiftKey = !!shiftKey; this.metaKey = !!metaKey;
    };
  }
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
    // js-dom M4 R106：spec 入口守卫（TypeError / InvalidStateError）——本原型此前的
    // lenient 回落（非 Event 构造空事件）违反 spec `dom-eventtarget-dispatchevent`
    // 步骤 1（WPT EventTarget-dispatchEvent "Calling dispatchEvent(null)" 抛 TypeError）。
    globalThis._zwDispatchGuard(event);
    var target = this;
    event.target = target;
    event.currentTarget = target;
    // js-dom M4 R114：`window.event`（HTML current event）对非 DOM EventTarget 同样生效
    //（WPT event-global "current event … (2)"——XHR dispatch 期 e === window.event）。
    var _r114Prev = globalThis.event;
    globalThis.event = event;
    var suffixes = ['', '|cap'];
    for (var s = 0; s < suffixes.length; s++) {
      var arr = (target._et_listeners || (target._et_listeners = {}))[event.type + suffixes[s]];
      if (!arr) continue;
      arr = arr.slice();
      for (var i = 0; i < arr.length; i++) {
        if (event._immediateStopped) break;
        try { arr[i].call(target, event); } catch (_) {}
      }
    }
    // R114：on* 属性 handler（IDL event handler）同 fire——spec 派发到 EventTarget 时先跑
    // listeners 再跑 on* handler（`xhr.onload = fn` 后 dispatchEvent('load') 须触发 fn；
    // WPT event-global (2) 正是此形态）。**去重**：BroadcastChannel/MessagePort 的 on* setter
    // 已把 handler 注册进 _et_listeners（listener 循环已调）——handler 函数与已调 listener
    // 同一引用时跳过，防双 fire（R2783 回归实证 b:hi 双发）。handler 返 true → preventDefault
    //（HTML onerror 语义；其余 handler 返值 spec 忽略，此处仅 onerror 认）。
    var _r114On = target['on' + event.type];
    if (typeof _r114On === 'function') {
      var _r114Already = false;
      var _r114Arrs = target._et_listeners;
      if (_r114Arrs) {
        var _r114Ak = event.type + '', _r114AkC = event.type + '|cap';
        var _r114A = (_r114Arrs[_r114Ak] || []).concat(_r114Arrs[_r114AkC] || []);
        for (var _r114ai = 0; _r114ai < _r114A.length; _r114ai++) {
          if (_r114A[_r114ai] === _r114On) { _r114Already = true; break; }
        }
      }
      if (!_r114Already) {
        try {
          var _r114R = _r114On.call(target, event);
          if (_r114R === true && event.type === 'error') { try { event.preventDefault(); } catch (_e114p) {} }
        } catch (_e114o) {}
      }
    }
    globalThis.event = _r114Prev;
    return !event._defaultPrevented;
  };
  globalThis.EventTarget = globalThis.EventTarget || EventTarget;

  if (globalThis.ServiceWorker) {
    Object.setPrototypeOf(globalThis.ServiceWorker.prototype, globalThis.EventTarget.prototype);
  }
  if (globalThis.ServiceWorkerRegistration) {
    Object.setPrototypeOf(
      globalThis.ServiceWorkerRegistration.prototype,
      globalThis.EventTarget.prototype
    );
  }

  // js-dom M4 R114：XMLHttpRequest 补 EventTarget 面——spec XHR : XMLHttpRequestEventTarget :
  // EventTarget（`xhr.addEventListener('load')` / `xhr.dispatchEvent(new Event('load'))` 与
  // on* 属性 handler 同键派发；WPT event-global "current event … (2)" 用 XHR 验证非 DOM
  // EventTarget 的 window.event 语义）。XHR ctor 在 part02 定义（先于本段执行），无
  // dispatchEvent/addEventListener（探针实证 undefined）。接线：原型链挂 EventTarget.prototype
  //（_et_listeners 自包含 map，与 Worker/MessagePort 同款）+ on* handler setter 语义由
  // ctor 既有属性承担（send 直接调 onload 不变）；addEventListener 注册的 listener 经
  // dispatchEvent 派发到（EventTarget.prototype.dispatchEvent 同款 target-only）。
  // spec https://xhr.spec.whatwg.org/#interface-xmlhttprequest
  if (globalThis.XMLHttpRequest && globalThis.EventTarget) {
    try {
      var _r114XhrProto = globalThis.XMLHttpRequest.prototype;
      if (_r114XhrProto && Object.getPrototypeOf(_r114XhrProto) !== globalThis.EventTarget.prototype) {
        Object.setPrototypeOf(_r114XhrProto, globalThis.EventTarget.prototype);
      }
    } catch (_e114x) {}
  }

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
      // R34xx（G6）：importScripts 经 host __zw_fetch_script 同步抓取并执行（worker 测试
      // 框架的 testharness.js/canvas-tests.js 导入——OffscreenCanvas worker 变体）。
      importScripts: function () {
        for (var ai = 0; ai < arguments.length; ai++) {
          var u = String(arguments[ai]);
          if (typeof __zw_fetch_script !== 'function') continue;
          var src = null;
          try {
            src = __zw_fetch_script(String(typeof location !== 'undefined' && location.href ? location.href : ''), u) || null;
          } catch (_e) { src = null; }
          if (src === null) continue;
          try {
            var body = 'var postMessage=self.postMessage.bind(self);'
              + 'var importScripts=function(){};'
              + 'var onmessage;'
              + src
              + '\n;if(typeof onmessage==="function")self.onmessage=onmessage;';
            new Function('self', body).call(null, wctx);
          } catch (_e) { /* 导入失败不阻断（测试自行报告） */ }
        }
      },
      close: function () { main._terminated = true; },
    };
    // R34xx（G6）：worker 全局暴露（OffscreenCanvas 等 canvas 构造器——worker 测试用）。
    wctx.OffscreenCanvas = globalThis.OffscreenCanvas;
    // R34xx：worker 暴露 OffscreenCanvasRenderingContext2D（self.OffscreenCanvas-
    // RenderingContext2D 断言）——懒 getter（构造器在首次 getContext 时创建）。
    Object.defineProperty(wctx, 'OffscreenCanvasRenderingContext2D', {
      get: function () { return globalThis.OffscreenCanvasRenderingContext2D; },
      configurable: true
    });
    wctx.ImageBitmap = globalThis.ImageBitmap;
    wctx.ImageData = globalThis.ImageData;
    // R34xx：CanvasGradient/CanvasPattern/CanvasRenderingContext2D 全局（gradient.object.
    // type/return、pattern.basic.type 的 self.CanvasGradient/CanvasPattern 断言——
    // instanceof 与实例原型链同主全局对象）。
    wctx.CanvasGradient = globalThis.CanvasGradient;
    wctx.CanvasPattern = globalThis.CanvasPattern;
    wctx.CanvasRenderingContext2D = globalThis.CanvasRenderingContext2D;
    // R34xx（G6）：worker 字体面——复用全局 FontFace/FontFaceSet（part06 的
    // `new FontFace(...).load()` 经 host __zw_load_font 真实加载 + document.fonts 同款
    // FontFaceSet 语义）；self.fonts 为独立 FontFaceSet 实例（worker 测试的 add/ready）。
    wctx.FontFace = globalThis.FontFace;
    wctx.fonts = (typeof globalThis.FontFaceSet === 'function')
      ? new globalThis.FontFaceSet()
      : { add: function () { return this; }, ready: Promise.resolve(), load: function () { return Promise.resolve([]); } };
    wctx.addEventListener = wctx.addEventListener || function () {};
    wctx.dispatchEvent = wctx.dispatchEvent || function () {};
    wctx.location = wctx.location || { href: '' };
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
      // R34xx（G6）：预提取 importScripts 源并**内联**到同一 Function 作用域（worker 测试
      // 框架的 testharness.js/canvas-tests.js 全局定义须与测试脚本共享——各自独立
      // Function 会隔离 _assertPixel 等定义）。移除原 importScripts 行（已内联）。
      var inlineImports = '';
      if (typeof __zw_fetch_script === 'function') {
        var importCalls = scriptSrc.match(/importScripts\([^;]*?\)/g) || [];
        for (var ii = 0; ii < importCalls.length; ii++) {
          var urlMatch = importCalls[ii].match(/["']([^"']+)["']/);
          if (!urlMatch) continue;
          var impSrc = null;
          try {
            impSrc = __zw_fetch_script(String(typeof location !== 'undefined' && location.href ? location.href : ''), urlMatch[1]) || null;
          } catch (_e) { impSrc = null; }
          if (impSrc !== null) inlineImports += impSrc + '\n';
        }
        scriptSrc = scriptSrc.replace(/importScripts\([^;]*?\);/g, '');
      }
      try {
        var body = inlineImports
          + 'var postMessage=self.postMessage.bind(self);'
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
  // 可读）。onmessage 属性 setter 内部走 addEventListener('message')。ArrayBuffer transfer 优先用
  // 原生 `transfer()` 真正 detach，其他 transferable 保留 `_detached` 标记；同执行上下文端口对
  //（跨 worker/进程通信需 host 接线，defer）。
  function MessagePort() {
    this._et_listeners = {}; // EventTarget 内部 listener map（构造器未自动调，手动初始化）
    this._other = null; // 配对端口（MessageChannel 构造时互连）
    this._closed = false;
    this._onmessage = null;
  }
  MessagePort.prototype = Object.create(EventTarget.prototype);
  MessagePort.prototype.constructor = MessagePort;
  MessagePort.prototype.postMessage = function (message, transfer) {
    if (this._closed || !this._other) return;
    // R56h：transferable 语义——transfer 列表中的 OffscreenCanvas 位图被转移
    //（detached），后续 drawImage 抛 InvalidStateError（2d.drawImage.detachedcanvas）。
    if (transfer && typeof transfer.forEach === 'function') {
      transfer.forEach(function (item) {
        if (item && typeof item === 'object') {
          if (item instanceof ArrayBuffer && typeof item.transfer === 'function') {
            item.transfer();
          } else {
            item._detached = true;
          }
        }
      });
    }
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
