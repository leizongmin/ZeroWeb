        }
        // `el.popover`（R3071）——enumerated 反射：无属性 → null；"auto" → "auto"；余（""/"manual"/无效）→ "manual"。
        // 委托 `_zwReadPopover`（showPopover 校验同源）。getter 返 null（real browser 一致——非 DOMString 默认空串）。
        if (prop === 'popover') return _zwReadPopover(sel, handle);
        // `el.popoverTargetElement`（R3073）——编程式目标元素。优先返 `_popoverTargetEl[key]`（setter 设）；
        // 无则回落 popovertarget 内容属性 id → getElementById（spec 一致）。无目标 → null。
        if (prop === 'popoverTargetElement') {
          if (_popoverTargetEl[key]) return _popoverTargetEl[key];
          var _pteId = typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, 'popovertarget') : __zw_get_attr(sel, 'popovertarget');
          if (_pteId) { var _pteEl = document.getElementById(_pteId); if (_pteEl) return _pteEl; }
          return null;
        }
        // `el.popoverTargetAction`（R3073）——enumerated 反射 popovertargetaction 属性：toggle/show/hide（默认 toggle，
        // invalid → toggle）。用 latest-wins 反映同 execute 内 pending set。
        if (prop === 'popoverTargetAction') {
          var _ptaRaw = typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, 'popovertargetaction') : __zw_get_attr(sel, 'popovertargetaction');
          var _pta = String(_ptaRaw || 'toggle').toLowerCase();
          return (_pta === 'show' || _pta === 'hide' || _pta === 'toggle') ? _pta : 'toggle';
        }
        // `el.role`——反射 role 属性（无 → ''）；同步 set→get 优先读缓存。
        if (prop === 'role') {
          var rlc = _reflectedAttrs[key];
          if (rlc && Object.prototype.hasOwnProperty.call(rlc, 'role')) return rlc['role'];
          return (handle ? __zw_get_attr_handle(handle, 'role') : __zw_get_attr(sel, 'role')) || '';
        }
        // `el.ariaXxx`——反射 aria-* 属性（ariaLabel↔aria-label, ariaLabelledBy↔aria-labelledby, ...）。
        // 经 `_ariaAttrName` 通用映射覆盖全部 aria IDL 属性；无 → ''。同步 set→get 优先读缓存。
        var _ariaName = _ariaAttrName(prop);
        if (_ariaName) {
          var arc = _reflectedAttrs[key];
          if (arc && Object.prototype.hasOwnProperty.call(arc, _ariaName)) return arc[_ariaName];
          return (handle ? __zw_get_attr_handle(handle, _ariaName) : __zw_get_attr(sel, _ariaName)) || '';
        }
        // reflected 布尔/枚举全局属性（R2848/R2850）：autofocus/draggable/spellcheck/translate（R2848）
        // + inert/autocomplete（R2850）——旧 fallthrough 返 undefined（spec 须布尔/串）。getter 优先读
        // _reflectedAttrs 缓存（setter 写解析值，同步 set→get 即时），无缓存则 host attr 解析。spec 默认：
        // autofocus/draggable/inert=false，spellcheck/translate=true，autocomplete="on"（missing-default）。
        // autofocus/inert 为 boolean attr（presence 判定，has_attr）；autocomplete 为 enumerated 串反射。
        // R3204：sel 读源改 latest-wins（`__zw_get_attr_lw`/`__zw_has_attr_lw`）反映同批 `setAttribute`/`.attr()`
        //（缓存仅覆盖 IDL setter→getter；setAttribute→IDL read 旧读纯快照 stale，如 `setAttribute('autocomplete','off')
        // ; el.autocomplete` 返 'on'）。handle 路径不动（本就 latest-wins）。
        if (prop === 'autofocus' || prop === 'draggable' || prop === 'spellcheck' || prop === 'translate' || prop === 'inert' || prop === 'autocomplete') {
          var rfc = _reflectedAttrs[key];
          if (rfc && Object.prototype.hasOwnProperty.call(rfc, prop)) return rfc[prop];
          if (prop === 'autofocus' || prop === 'inert') {
            // boolean attr：presence（has_attr）→ true；缺省 → false。
            if (handle) {
              try { return __zw_has_attr_handle(handle, prop) === '1'; } catch (_e) { return false; }
            }
            return (typeof __zw_has_attr_lw === 'function'
              ? __zw_has_attr_lw(sel, prop)
              : (typeof __zw_has_attr === 'function' ? __zw_has_attr(sel, prop) : '0')) === '1';
          }
          if (prop === 'autocomplete') {
            // enumerated 串反射：attr 值（缺省 → "on"，spec missing-default）。__zw_get_attr 缺省返 "" 故 "" 亦判缺省。
            var acRaw = handle ? __zw_get_attr_handle(handle, 'autocomplete') : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, 'autocomplete') : __zw_get_attr(sel, 'autocomplete'));
            return (acRaw == null || acRaw === '') ? 'on' : String(acRaw);
          }
          // R3188 draggable：enumerated（true/false，case-insensitive），缺省/非法 → auto 状态 → default-draggable
          //（spec/Chrome：img/audio/video/a[href] 默认可拖拽，余 false）。旧实现仅 `=== 'true'`（case-sensitive，
          // 且 auto 状态统一 false——缺省 `<img>` 误判不可拖拽）。
          if (prop === 'draggable') {
            var dgRaw = handle ? __zw_get_attr_handle(handle, 'draggable') : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, 'draggable') : __zw_get_attr(sel, 'draggable'));
            var dgLo = (dgRaw == null) ? '' : String(dgRaw).toLowerCase();
            if (dgLo === 'true') return true;
            if (dgLo === 'false') return false;
            return _defaultDraggable(sel, handle); // auto 状态（缺省/invalid/其它）
          }
          var rfRaw = handle ? __zw_get_attr_handle(handle, prop) : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, prop) : __zw_get_attr(sel, prop));
          rfRaw = (rfRaw == null) ? '' : String(rfRaw).toLowerCase();
          if (prop === 'spellcheck') return rfRaw !== 'false'; // "false"→false，余（含缺省）→true（spec 默认 true）
          return rfRaw !== 'no';                               // translate："no"→false，余→true（默认 true）
        }
        // FR-009：资源获取 settle 后的 IMG / media / track IDL 状态。
        // https://html.spec.whatwg.org/multipage/embedded-content.html#dom-img-naturalwidth
        // https://html.spec.whatwg.org/multipage/media.html#dom-media-networkstate
        var resourceTag = _realTag(sel, handle);
        var resourceState = _resourceStates[key];
        if (resourceTag === 'IMG' && prop === 'complete') {
          if (resourceState) return true;
          var completeSrc = handle ? __zw_get_attr_handle(handle, 'src') : __zw_get_attr(sel, 'src');
          var completeSrcset = handle ? __zw_get_attr_handle(handle, 'srcset') : __zw_get_attr(sel, 'srcset');
          return !completeSrc && !completeSrcset;
        }
        if ((resourceTag === 'IMG' || resourceTag === 'AUDIO' || resourceTag === 'VIDEO') && prop === 'currentSrc') {
          return resourceState ? resourceState.url : '';
        }
        if ((resourceTag === 'AUDIO' || resourceTag === 'VIDEO') &&
            (prop === 'NETWORK_EMPTY' || prop === 'NETWORK_IDLE' || prop === 'NETWORK_LOADING' ||
             prop === 'NETWORK_NO_SOURCE' || prop === 'HAVE_NOTHING' || prop === 'HAVE_METADATA' ||
             prop === 'HAVE_CURRENT_DATA' || prop === 'HAVE_FUTURE_DATA' || prop === 'HAVE_ENOUGH_DATA')) {
          return {
            NETWORK_EMPTY: 0, NETWORK_IDLE: 1, NETWORK_LOADING: 2, NETWORK_NO_SOURCE: 3,
            HAVE_NOTHING: 0, HAVE_METADATA: 1, HAVE_CURRENT_DATA: 2, HAVE_FUTURE_DATA: 3, HAVE_ENOUGH_DATA: 4
          }[prop];
        }
        if ((resourceTag === 'AUDIO' || resourceTag === 'VIDEO') && prop === 'networkState') {
          if (resourceState) return resourceState.outcome === 'error' ? 3 : 1;
          var mediaSrc = handle ? __zw_get_attr_handle(handle, 'src') : __zw_get_attr(sel, 'src');
          if (mediaSrc) return 2;
          try {
            var mediaSources = _makeProxy(sel, handle).querySelectorAll('source');
            return mediaSources && mediaSources.length ? 2 : 0;
          } catch (_e) { return 0; }
        }
        if ((resourceTag === 'AUDIO' || resourceTag === 'VIDEO') && prop === 'readyState') return 0;
        if ((resourceTag === 'AUDIO' || resourceTag === 'VIDEO') && prop === 'error') {
          return resourceState && resourceState.outcome === 'error' ? resourceState.error : null;
        }
        if (resourceTag === 'TRACK' &&
            (prop === 'NONE' || prop === 'LOADING' || prop === 'LOADED' || prop === 'ERROR')) {
          return { NONE: 0, LOADING: 1, LOADED: 2, ERROR: 3 }[prop];
        }
        if (resourceTag === 'TRACK' && prop === 'readyState') {
          if (!resourceState) {
            var trackSrc = handle ? __zw_get_attr_handle(handle, 'src') : __zw_get_attr(sel, 'src');
            return trackSrc ? 1 : 0;
          }
          return resourceState.outcome === 'error' ? 3 : 2;
        }
        // reflected unsigned-long 维度属性（R2851）：IMG/IFRAME `.width`/`.height`（反射 width/height 内容属性
        // 为非负整数，缺省/不可解析 → 0；spec「reflect unsigned long」算法）+ IMG `.naturalWidth`/`.naturalHeight`.
        // CANVAS（缺省 300/150 且 setter 改 bitmap，特殊）/ VIDEO/EMBED defer。
        if (prop === 'width' || prop === 'height' || prop === 'naturalWidth' || prop === 'naturalHeight') {
          var rgTag = resourceTag;
          if (rgTag === 'IMG' && (prop === 'naturalWidth' || prop === 'naturalHeight')) {
            if (resourceState && resourceState.outcome === 'loaded') {
              return prop === 'naturalWidth' ? resourceState.width : resourceState.height;
            }
            // R34xx：G5 —— host 查询图片尺寸（webview image_sizes 快照，__zw_get_image_size）。
            if (typeof __zw_get_image_size === 'function') {
              var srcAttr = (handle ? __zw_get_attr_handle(handle, 'src') : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, 'src') : __zw_get_attr(sel, 'src'))) || '';
              var dims = String(__zw_get_image_size(srcAttr));
              if (dims) {
                var wh = dims.split(',');
                var w = parseInt(wh[0], 10) || 0;
                var h = parseInt(wh[1], 10) || 0;
                return prop === 'naturalWidth' ? w : h;
              }
            }
            return 0;
          }
          if ((rgTag === 'IMG' || rgTag === 'IFRAME') && (prop === 'width' || prop === 'height')) {
            // sync set→get 优先读缓存（setter 写数值）；无缓存则解析 width/height 内容属性（缺省/非负整数失败 → 0）。
            // R3204：sel 读源 latest-wins（`__zw_get_attr_lw`）反映同批 setAttribute。
            var drc = _reflectedAttrs[key];
            if (drc && Object.prototype.hasOwnProperty.call(drc, prop)) return drc[prop];
            var dRaw = handle ? __zw_get_attr_handle(handle, prop) : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, prop) : __zw_get_attr(sel, prop));
            var dN = parseInt(dRaw, 10);
            return (isNaN(dN) || dN < 0) ? 0 : dN;
          }
        }
        // R3077：HTMLCanvasElement proxy 的 canvas 2D API（getContext/toDataURL/width/height）。旧仅 standalone
        // `_zwMakeCanvas` 有这些（R2795），DOM 元素 proxy 缺 → `document.getElementById('c').getContext('2d')` 抛
        // TypeError（~29 canvas WPT 用例不可执行）。本切片接通：getContext 经 host `__zw_canvas_op` 建 2d 上下文
        //（per-element 缓存 `_zwCanvasCtx[key]`，后续返同一 ctx）；toDataURL 编码 ctx pixel_buffer → PNG data URL；
        // width/height 反射内容属性（default 300/150，spec HTMLCanvasElement）。host 未注册 → lenient（getContext null /
        // toDataURL 'data:,'）。https://html.spec.whatwg.org/multipage/canvas.html#htmlcanvaselement
        if (_realTag(sel, handle) === 'CANVAS') {
          if (prop === 'getContext') {
            return function (type) {
              if (String(type) !== '2d') return null; // 仅 2d；webgl/webgl2 defer
              if (_zwCanvasCtx[key]) return _zwCanvasCtx[key];
              if (typeof __zw_canvas_op !== 'function') return null;
              var cw = _zwCanvasDim(sel, handle, 'width', 300);
              var ch = _zwCanvasDim(sel, handle, 'height', 150);
              var id = __zw_canvas_op('0', 'getContext2d', String(cw), String(ch));
              if (!id || String(id).charAt(0) === '!') return null;
              var ctx = _zwMakeCtx2d(String(id));
              ctx.canvas = _makeProxy(sel, handle); // canvas back-ref → 元素 proxy（spec ctx.canvas）
              _zwCanvasCtx[key] = ctx;
              // R34xx：direction 'inherit' 解析为 canvas 元素方向（dir 属性——
              // 2d.text.draw.align.start.rtl 的 <canvas dir="rtl">）。host 存解析值；
              // client getter 保持 'inherit'（spec 值）。
              var elDir = String(ctx.canvas.getAttribute ? String(ctx.canvas.getAttribute('dir') || '') : '').toLowerCase();
              if (elDir === 'rtl' || elDir === 'ltr') {
                __zw_canvas_op(String(id), 'setDirection', elDir);
              }
              // R3268 canvas 显示链路：把 ctx id 写入元素属性，painter 据此把 canvas
              // 内容桥接为页面图元（data-zw-canvas-ctx 非标准属性，仅内部使用）。
              if (handle) __zw_set_attr_handle(handle, 'data-zw-canvas-ctx', String(id));
              else __zw_set_attr(sel, 'data-zw-canvas-ctx', String(id));
              return ctx;
            };
          }
          if (prop === 'toDataURL') {
            return function (_type) {
              if (typeof __zw_canvas_op !== 'function') return 'data:,';
              var ctx = _zwCanvasCtx[key];
              if (!ctx || !ctx._handle) return 'data:,'; // 未 getContext → 无 bitmap
              var csv = String(__zw_canvas_op(ctx._handle, 'toDataURL'));
              if (!csv) return 'data:,';
              var nums = csv.split(',');
              var s = '';
              for (var i = 0; i < nums.length; i++) s += String.fromCharCode(+nums[i]);
              return 'data:image/png;base64,' + btoa(s);
            };
          }
          // toBlob（R3296）：异步 PNG Blob 导出（镜像 standalone part05.js 实现）。callback(blob|null)
          // 在 microtask 异步派发；返 undefined。复用 toDataURL 的 PNG 编码 host op。无 ctx 惰性创建
          //（与同路径 toDataURL 同语义——real browser 无 ctx canvas 仍产空白 PNG，仅编码失败/无 host → null）。
          if (prop === 'toBlob') {
            return function (callback, _type, _quality) {
              var p = Promise.resolve().then(function () {
                if (typeof __zw_canvas_op !== 'function') return null;
                if (!_zwCanvasCtx[key] || !_zwCanvasCtx[key]._handle) {
                  // 惰性建 ctx（镜像 toDataURL getContext 调用）——经 proxy getContext get-trap 建 + 缓存。
                  var proxy = _makeProxy(sel, handle);
                  if (typeof proxy.getContext === 'function') proxy.getContext('2d');
                }
                var ctx = _zwCanvasCtx[key];
                if (!ctx || !ctx._handle) return null; // 建失败 → 无 bitmap
                var csv = String(__zw_canvas_op(ctx._handle, 'toDataURL'));
                if (!csv) return null;
                var nums = csv.split(',');
                var bytes = new Uint8Array(nums.length);
                for (var i = 0; i < nums.length; i++) bytes[i] = +nums[i];
                return new Blob([bytes], { type: 'image/png' });
              });
              if (typeof callback === 'function') p.then(function (blob) { callback(blob); });
              return undefined;
            };
          }
          // R3313：transferControlToOffscreen()——DOM canvas 转 OffscreenCanvas（spec HTML §4.12）。
          // 返回 OffscreenCanvas 对象，其 getContext('2d') **复用** DOM canvas 的 host context handle
          //（共享 bitmap——对 offscreen 的绘制反映到 DOM canvas 显示）。spec：transfer 后 DOM canvas
          // getContext() 抛 InvalidStateError（control 已转交）。实现：建 DOM canvas context（若未建）→
          // 标记 _transferred → 返 offscreen 对象持同一 handle。offscreen.transferToImageBitmap 复用 R3312 路径。
          if (prop === 'transferControlToOffscreen') {
            return function () {
              if (typeof __zw_canvas_op !== 'function') return null;
              // spec：已 transferred → 抛 InvalidStateError（同 canvas 二次 transfer）。
              var drc = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
              if (drc._transferred) {
                throw new TypeError('transferControlToOffscreen: canvas control already transferred');
              }
              // 建 DOM canvas context（若未建），取其 host handle 共享给 offscreen。
              if (!_zwCanvasCtx[key] || !_zwCanvasCtx[key]._handle) {
                var proxy = _makeProxy(sel, handle);
                if (typeof proxy.getContext === 'function') proxy.getContext('2d');
              }
              var domCtx = _zwCanvasCtx[key];
              if (!domCtx || !domCtx._handle) return null;
              drc._transferred = true; // 标记：后续 DOM canvas getContext 抛（spec）
              var sharedHandle = domCtx._handle;
              var cw = _zwCanvasDim(sel, handle, 'width', 300);
              var ch = _zwCanvasDim(sel, handle, 'height', 150);
              // offscreen 对象：getContext 复用 sharedHandle（不新建 host context），transferToImageBitmap 取全像素。
              var oc = { width: cw, height: ch, _ctx: null, _sharedHandle: sharedHandle };
              oc.getContext = function (type) {
                if (String(type) !== '2d') return null;
                if (oc._ctx) return oc._ctx;
                // 复用 DOM canvas 的 ctx proxy（持同一 host handle，绘制共享 bitmap）。
                oc._ctx = _zwMakeCtx2d(sharedHandle);
                return oc._ctx;
              };
              oc.transferToImageBitmap = function () {
                var wire = String(__zw_canvas_op(sharedHandle, 'getImageData', '0', '0', String(oc.width), String(oc.height)));
                var bm = _zwMakeImageBitmap(wire);
                if (bm.width <= 0 || bm.height <= 0) return null;
                // R3254-C8：只清 bitmap、保留绘图状态（同 OffscreenCanvas.prototype）。
                __zw_canvas_op(sharedHandle, 'clearBitmap');
                return bm;
              };
              return oc;
            };
          }
          if (prop === 'width' || prop === 'height') {
            // sync set→get 优先读缓存（setter R3077 写数值）；无缓存则反射内容属性（default 300/150）。
            var cdc = _reflectedAttrs[key];
            if (cdc && Object.prototype.hasOwnProperty.call(cdc, prop)) return cdc[prop];
            return _zwCanvasDim(sel, handle, prop, prop === 'width' ? 300 : 150);
          }
        }
        // `el.dataset`——`data-*` 属性的 camelCase 键对象（get/set/has/delete/枚举）。
        // dataset.fooBar ↔ data-foo-bar 属性。handle 脱离 DOM 元素枚举受限（无 attr-names-handle）。
        if (prop === 'dataset') {
          return _datasetProxy(sel, handle);
        }
        // R2926 Shadow DOM：`element.attachShadow(init)` / `element.shadowRoot`。host 元素专用（非
        // fragment/comment/text/shadow）。attachShadow 建 shadow root（复用 DocumentFragment handle 容器）；
        // shadowRoot 读——open 返 root、closed/未建 返 null（spec）。详见 `_attachShadow`。
        if (prop === 'shadowRoot') {
          var _sr = _shadowRoots[key];
          return (_sr && _sr.mode === 'open') ? _wrapHandle(_sr.handle) : null;
        }
        if (prop === 'attachShadow') {
          return function (init) { return _attachShadow(sel, handle, init); };
        }
        if (prop === 'textContent' || prop === 'innerText') {
          // `innerText`（R3260）≈ textContent 近似——real innerText 是 layout/CSS-aware（排除 hidden 元素、
          // `<br>`/block →换行、white-space 处理）；headless 无 layout 透出到 JS，best-effort 返 textContent。
          // 高频读元素渲染文本（`el.innerText` 此前返 undefined）；setter 同 textContent（替换全部子为文本节点）。
          // R3028：sel-based 走 latest-wins（consult 变更列表，闭合 `textContent=` 后 getter stale 旧值）；
          // 回调未注册（polyfill/其它环境）→ fallback 纯快照 `__zw_get_text`。
          if (handle) return __zw_get_text_handle(handle);
          return typeof __zw_get_text_lw === 'function' ? __zw_get_text_lw(sel) : __zw_get_text(sel);
        }
        if (prop === 'innerHTML') {
          return handle ? __zw_get_inner_html_handle(handle) : __zw_get_inner_html(sel);
        }
        // `element.outerHTML`（getter）：含自身 tag/属性 + 子树序列化。sel-based（已挂载）经 host
        // `__zw_get_outer_html` 真实序列化（含 void 元素 + 属性转义）。R3201：handle-only（createElement 未挂载）
        // 旧 best-effort 返 innerHTML（无 wrapper）；现客户端构造 `<tag attrs>innerHTML</tag>`，复用 R3198 三 handle
        // 回调（`__zw_get_tag_handle`/`__zw_attr_names_handle`/`__zw_get_attr_handle`）+ `__zw_get_inner_html_handle`。
        // void 元素（br/img/input 等）无闭合标签；属性值转义 `&`/`"`（HTML 序列化 §attribute value）。isEqualNode
        // handle 经 outerHTML 比对亦受益。
        if (prop === 'outerHTML') {
          if (sel && typeof __zw_get_outer_html === 'function') {
            try { return __zw_get_outer_html(sel); } catch (_e) { return ''; }
          }
          if (handle) {
            try {
              var VOID = { area: 1, base: 1, br: 1, col: 1, embed: 1, hr: 1, img: 1, input: 1, link: 1, meta: 1, param: 1, source: 1, track: 1, wbr: 1 };
              var tag = (typeof __zw_get_tag_handle === 'function' ? __zw_get_tag_handle(handle) : '') || 'div';
              var names = (typeof __zw_attr_names_handle === 'function' ? __zw_attr_names_handle(handle) : '');
              var attrStr = '';
              if (names) {
                names.split('|').filter(Boolean).forEach(function(n) {
                  var v = typeof __zw_get_attr_handle === 'function' ? __zw_get_attr_handle(handle, n) : '';
                  var esc = String(v == null ? '' : v).replace(/&/g, '&amp;').replace(/"/g, '&quot;');
                  attrStr += ' ' + n + '="' + esc + '"';
                });
              }
              if (VOID[tag.toLowerCase()]) return '<' + tag + attrStr + '>';
              var inner = typeof __zw_get_inner_html_handle === 'function' ? (__zw_get_inner_html_handle(handle) || '') : '';
              return '<' + tag + attrStr + '>' + inner + '</' + tag + '>';
            } catch (_e) { return ''; }
          }
          return '';
        }
        if (prop === 'parentNode' || prop === 'parentElement') {
          return _parentNodeFor(sel, handle);
        }
        // 元素遍历/导航 API（仅元素子/兄弟，跳过文本/注释）。handle（脱离 DOM，无 sel）→ null/[]。
        if (prop === 'children') {
          // js-dom M4 R38：`Element.children` 返 HTMLCollection（spec `dom-parentnode-children`，带
          // item/namedItem + indexed/named properties）。旧返纯数组缺 namedItem（WPT HTMLCollection-empty-name
          // "Element.children" fail：`c.namedItem("")` 抛 TypeError）。经 _zwMakeCollection(true) 包成
          // HTMLCollection（含 R38 namedItem 空串守卫）。_splitSelectors 已 .map(_wrapSelector) 返 proxy 数组。
          return sel && typeof __zw_element_children === 'function'
            ? _zwMakeCollection(_splitSelectors(__zw_element_children(sel)), true) : _zwMakeCollection([], true);
        }
        if (prop === 'firstElementChild' || prop === 'lastElementChild' || prop === 'childElementCount') {
          // R2927：容器 handle（shadow/fragment）从 registry 读元素子（无 selector，须 registry）。
          if (_isContainerHandle(handle)) {
            var ek = _handleElementChildren(handle);
            if (prop === 'childElementCount') return ek.length;
            if (!ek.length) return null;
            return prop === 'firstElementChild' ? ek[0] : ek[ek.length - 1];
          }
          if (!sel || typeof __zw_element_children !== 'function') {
            return prop === 'childElementCount' ? 0 : null;
          }
          var kids = _splitSelectors(__zw_element_children(sel));
          if (prop === 'childElementCount') return kids.length;
          if (!kids.length) return null;
          return prop === 'firstElementChild' ? kids[0] : kids[kids.length - 1];
        }
        if (prop === 'previousElementSibling' || prop === 'nextElementSibling') {
          if (!sel || typeof __zw_element_siblings !== 'function') return null;
          try {
            var parts = __zw_element_siblings(sel).split('|');
            var hit = prop === 'previousElementSibling' ? parts[0] : parts[1];
            return hit ? _wrapSelector(hit) : null;
          } catch (_e) { return null; }
        }
        // 节点级遍历（含文本/注释，区别于上面的 element-only 版）：childNodes / firstChild /
        // lastChild（子列表，经 __zw_child_nodes JSON）/ previousSibling / nextSibling（兄弟，经
        // __zw_sibling_nodes JSON）。文本/注释节点返静态对象（_wrapNodeEntry）。仅 sel-based 目标。
        if (prop === 'childNodes') {
          // R2927：容器 handle（shadow/fragment）从 registry 读子节点（无 selector，须 registry）。
          if (_isContainerHandle(handle)) return _handleChildNodes(handle);
          return _childNodeList(sel, handle);
        }
        if (prop === 'firstChild' || prop === 'lastChild') {
          var cn = _isContainerHandle(handle) ? _handleChildNodes(handle) : _childNodeList(sel, handle);
          if (!cn.length) return null;
          return prop === 'firstChild' ? cn[0] : cn[cn.length - 1];
        }
        if (prop === 'previousSibling' || prop === 'nextSibling') {
          if (!sel || typeof __zw_sibling_nodes !== 'function') return null;
          try {
            var pair = JSON.parse(__zw_sibling_nodes(sel) || '{"p":null,"n":null}');
            var en = prop === 'previousSibling' ? pair.p : pair.n;
            return _wrapNodeEntry(en, _parentNodeFor(sel, handle));
          } catch (_e) { return null; }
        }
        // `el.contains(other)`——other 是否为 el 的后代或 el 自身（沿 parent 链）。
        if (prop === 'contains') {
          return function(other) {
            if (!sel || typeof __zw_contains !== 'function') return false;
            var otherSel = other && other.__zwSelector;
            if (!otherSel) return false;
            try { return __zw_contains(sel, otherSel) === '1'; } catch (_e) { return false; }
          };
        }
        // `el.getRootNode()`——沿 parent 链到根（通常 html），返根 proxy。sel 缺失 → 返自身。
        if (prop === 'getRootNode') {
          return function() {
            if (!sel) return _makeProxy(sel, handle);
            var chain = _ancestorChain(sel);
            return _wrapSelector(chain.length ? chain[chain.length - 1] : sel);
          };
        }
        // `el.isConnected`（只读 boolean，spec Node.isConnected：节点是否连入 document）——框架 / 库
        // 高频判活（jQuery cleanData、React commit-phase、mutation handler `if (!node.isConnected) return`；
        // 缺失则恒 undefined（falsy）→ 在档元素被误判 detached，脚本取错分支）。
        // ① sel-based（parsed 元素 / querySelector·getElementById 结果 / html·body·head）→ 经 `__zw_contains
        //   ('html', sel)`（element_contains 自含，html 自身亦命中）判定是否在 documentElement 子树内——亦
        //   正确反映 `el.remove()` / `removeChild` 后的 detach（selector 不再在档 → 返 '0'）；无 `__zw_contains`
        //   回调路径 → fallback true（sel-based parsed 元素构造即在树内）。
        // ② handle-only（createElement / createTextNode / createComment / DocumentFragment，未挂载）→ false；
        // ③ 已 appendChild 的 handle 元素 best-effort：`__zw_getBoundingClientRect(handle)` 非空 ⇒ 已在布局
        //   树（= 已连入文档）→ true（复用 R2661 handle→NodeId 解析）。**已知限制**：append 后未跑 layout
        //   的同一 execute 内可能暂报 false（layout-dependent probe）；text/comment 节点无布局 rect → append
        //   后仍报 false（少见 `textNode.isConnected` 检查，documented）。Document 节点恒 connected（见 literal）。
        if (prop === 'isConnected') {
          if (sel) {
            if (typeof __zw_contains === 'function') {
              try { return __zw_contains('html', sel) === '1'; } catch (_e) { return true; }
            }
            return true;
          }
          if (handle && typeof __zw_getBoundingClientRect === 'function') {
            try { return __zw_getBoundingClientRect(handle) !== ''; } catch (_e) { return false; }
          }
          return false;
        }
        // `el.hasChildNodes()`（spec Node.hasChildNodes：是否有任意子节点含文本/注释）——树遍历 / diff /
        // 子节点存在性检查高频。经既有 `_childNodeList`（元素查 `__zw_child_nodes`；handle-only 返 []）取
        // length>0。text/comment 节点本身无子（spec）；DocumentFragment 子节点经 host flatten 跟踪，
        // handle-only _childNodeList 暂返 [] → 报 false（detached fragment 检查少见，documented）。
        if (prop === 'hasChildNodes') {
          return function() { return _childNodeList(sel, handle).length > 0; };
        }
        // `el.isSameNode(other)`——节点身份相等（deprecated，等价 ===；proxy 缓存使同节点同 proxy，
        // 但经 _elKey 比较更鲁棒：sel/handle 一致即同节点）。
        if (prop === 'isSameNode') {
          return function(other) {
            if (!other) return false;
            var otherSel = other.__zwSelector || '';
            var otherHandle = other.__zwHandle || null;
            return _elKey(sel, handle) === _elKey(otherSel, otherHandle);
          };
        }
        // `el.isEqualNode(other)`——节点结构相等（node-equality 三件套：isSameNode 身份 / compareDocumentPosition
        // 位置 / isEqualNode 结构）。testing/diff 库高频。经 `_nodeSig` 序列化签名比对（元素 outerHTML / text·comment
        // nodeValue）。**已知限制**：属性序敏感（spec 序无关）；handle/detached 元素 outerHTML 仅 innerHTML 回落。
        if (prop === 'isEqualNode') {
          return function(other) {
            if (!other || typeof other !== 'object') return false;
            var oSel = other.__zwSelector || '';
            var oHandle = other.__zwHandle || null;
            return _nodeSig(sel, handle) === _nodeSig(oSel, oHandle);
          };
        }
        // `el.compareDocumentPosition(other)`——bitmask 描述 other 相对 el 的文档位置（树算法 / 库排序高频）。
        // 经 `_ancestorChain`（self/other→root）+ LCA + `__zw_element_children` 子序比较。已知限制：仅 sel-based
        // element（text/comment 节点无 sel → DISCONNECTED 兜底）；不同树 → DISCONNECTED|IMPL。
        if (prop === 'compareDocumentPosition') {
          return function(other) {
            var FOLLOWING = 4, PRECEDING = 2, CONTAINS = 8, CONTAINED_BY = 16, DISCONNECTED = 1, IMPL = 32;
            var otherSel = other && other.__zwSelector;
            if (!sel || !otherSel) return DISCONNECTED | IMPL;
            if (sel === otherSel) return 0;
            var aChain = _ancestorChain(sel);
            var bChain = _ancestorChain(otherSel);
            if (!aChain.length || !bChain.length) return DISCONNECTED | IMPL;
            if (aChain[aChain.length - 1] !== bChain[bChain.length - 1]) return DISCONNECTED | IMPL;
            // other 是 this 的祖先 → other contains this + other precedes this（doc 序）。
            if (aChain.indexOf(otherSel) >= 0) return CONTAINS | PRECEDING;
            // this 是 other 的祖先 → this contains other + other follows this。
            if (bChain.indexOf(sel) >= 0) return CONTAINED_BY | FOLLOWING;
            // 共同祖先非直系：root→node 反转链找 LCA；扫描 LCA element children 的**原始 selector 串**
            //（_splitSelectors 会包成 proxy，故直接 split '|'），经 `__zw_contains`（节点包含，selector-format
            // 无关）定位含 this / other 的子，序比较。
            var ra = aChain.slice().reverse(), rb = bChain.slice().reverse();
            var i = 0;
            while (i < ra.length && i < rb.length && ra[i] === rb[i]) i++;
            var lca = ra[i - 1];
            if (lca && typeof __zw_element_children === 'function' && typeof __zw_contains === 'function') {
              try {
                var kids = String(__zw_element_children(lca) || '').split('|').filter(Boolean);
                var ti = -1, oi = -1;
                for (var k = 0; k < kids.length && (ti < 0 || oi < 0); k++) {
                  if (ti < 0 && __zw_contains(kids[k], sel) === '1') ti = k;
                  if (oi < 0 && __zw_contains(kids[k], otherSel) === '1') oi = k;
                }
                if (ti >= 0 && oi >= 0) return ti < oi ? FOLLOWING : PRECEDING;
              } catch (_e) {}
            }
            return FOLLOWING; // 兜底
          };
        }
        // DocumentFragment handle（nodeType 11 / '#document-fragment'）/ Comment（nodeType 8 / '#comment'）/
        // Text（nodeType 3 / '#text'）——均为 create 句柄无 selector，经 handle set 区别于普通元素句柄。
        var isFrag = handle && _fragmentHandles[handle];
        var isShadow = handle && _shadowHandles[handle];
        var isComment = handle && _commentHandles[handle];
        var isText = handle && _textHandles[handle];
        var isPI = handle && _piHandles[handle];
        // createElementNS handle（js-dom M4 / R18）：大小写敏感 + 带 prefix/namespace。其
        // tagName/nodeName/prefix/localName 经 `_nsHandles` 原值读回，不经 `_realTag`（强制大写）。
        var isNs = handle && _nsHandles[handle];
        if (prop === 'tagName') {
          if (isNs) return _nsQualified(_nsHandles[handle].qualifiedName);
          return (isFrag || isShadow || isComment || isText || isPI) ? undefined : _realTag(sel, handle);
        }
        // `element.localName`（spec `dom-element-localname`，R11）：HTML 元素 = tagName 小写；
        // 带 prefix 的限定名（`svg:rect`，createElementNS）去 prefix 取冒号后。非 Element → null
        //（spec Attr/Text 等另走各自接口，此处元素 getter 范围）。createElement 用例核心断言之一。
        // createElementNS handle（isNs）：spec createElementNS **不**小写，原样返冒号后大小写敏感值。
        if (prop === 'localName') {
          if (isFrag || isShadow || isComment || isText || isPI) return null;
          if (isNs) return _nsLocal(_nsHandles[handle].qualifiedName);
          var _ln = _realTag(sel, handle);
          var _colon = _ln.indexOf(':');
          if (_colon >= 0) _ln = _ln.slice(_colon + 1);
          return _ln.toLowerCase();
        }
        // `element.prefix`（spec `dom-node-prefix`，R12）：限定名冒号前部分；无冒号 → null。非 Element → null。
        // createElementNS handle（isNs）：从原 qualifiedName 冒号前取，大小写敏感（spec createElementNS
        // 不小写 prefix，`"abc:l"` → prefix `"abc"`）。无 prefix（无冒号）→ null。普通 createElement 元素
        // 经 `_realTag`（强制大写）；createElement 不带 prefix 故恒 null。
        if (prop === 'prefix') {
          if (isFrag || isShadow || isComment || isText || isPI) return null;
          if (isNs) return _nsPrefix(_nsHandles[handle].qualifiedName);
          var _pfTag = _realTag(sel, handle);
          var _pfColon = _pfTag.indexOf(':');
          return _pfColon >= 0 ? _pfTag.slice(0, _pfColon) : null;
        }
        if (prop === 'nodeName') {
          if (isShadow) return '#shadow-root';
          if (isFrag) return '#document-fragment';
          if (isComment) return '#comment';
          if (isText) return '#text';
          if (isPI) return _piHandles[handle].target;
          if (isNs) return _nsQualified(_nsHandles[handle].qualifiedName); // createElementNS 大小写敏感
          return _realTag(sel, handle);
        }
        if (prop === 'nodeType') {
          return (isShadow || isFrag) ? 11 : (isPI ? 7 : (isComment ? 8 : (isText ? 3 : 1)));
        }
        // `element.namespaceURI`（spec `dom-node-namespaceuri`，R18）：createElementNS handle 从
        // `_nsHandles` 读记录的 namespace（SVG/MathML/自定义 URI）；普通 `createElement` 元素恒为
        // HTML 命名空间 `"http://www.w3.org/1999/xhtml"`（spec HTML 元素）。createAttribute/PI 等非元素
        // 另走各自接口，此处元素 getter 范围返 null 之外由 ns 表决定。
        if (prop === 'namespaceURI') {
          if (isNs) return _nsHandles[handle].namespace;
          if (isFrag || isShadow || isComment || isText || isPI) return null;
          return 'http://www.w3.org/1999/xhtml';
        }
        // ShadowRoot 专用属性（R2926）：host = 宿主元素 proxy；mode = 'open'/'closed'。
        if (isShadow && prop === 'host') {
          var _sm = _shadowHandleMeta[handle];
          return _sm ? _makeProxy(_sm.hostSel, _sm.hostHandle) : null;
        }
        if (isShadow && prop === 'mode') {
          var _smm = _shadowHandleMeta[handle];
          return _smm ? _smm.mode : 'open';
        }
        // Text/Comment 节点的 nodeValue/data = 文本（经 __zw_get_text_handle 读回，element 的 nodeValue 为 null）。
        if ((isText || isComment) && (prop === 'nodeValue' || prop === 'data')) {
          return handle ? __zw_get_text_handle(handle) : '';
        }
        // ProcessingInstruction 节点（js-dom M4）：`.target` = PI target，`.data`/`.nodeValue` = PI data
        //（spec `dom-processinginstruction`：data 即 CharacterData.data；nodeName = target 见上）。读自 _piHandles。
        if (isPI) {
          var _pi = _piHandles[handle];
          if (prop === 'target') return _pi ? _pi.target : '';
          if (prop === 'data' || prop === 'nodeValue') return _pi ? _pi.data : '';
          if (prop === 'length') return _pi ? _pi.data.length : 0;
        }
        // CharacterData 数据编辑方法（R2823，text/comment 节点）+ Text.splitText。仅 handle-based
        // 文本/注释节点（createTextNode/createComment 所建——parsed DOM 文本节点为 _wrapNodeEntry 静态
        // 快照无 handle）。读经 __zw_get_text_handle（query_text_from_mutations 反向 replay 取最新值，
        // 故多次编辑 compose 正确），写经 __zw_set_text_handle（追加 SetTextOnHandle mutation）。offset
        // 越界 clamp（spec 抛 IndexSizeError，此处 permissive 不抛）。contentEditable 编辑库（ProseMirror
        // / Slate / Quill）+ Range/Selection 高频。
        if ((isText || isComment) && prop === 'length') {
          return handle ? __zw_get_text_handle(handle).length : 0;
        }
        if ((isText || isComment) && prop === 'appendData') {
          return function (s) {
            if (handle) __zw_set_text_handle(handle, __zw_get_text_handle(handle) + String(s == null ? '' : s));
            return undefined;
          };
        }
        if ((isText || isComment) && prop === 'deleteData') {
          return function (offset, count) {
            if (!handle) return undefined;
            var cur = __zw_get_text_handle(handle);
            var o = offset | 0, c = count | 0;
            if (o < 0) o = 0;
            if (c < 0) c = 0;
            __zw_set_text_handle(handle, cur.slice(0, o) + cur.slice(o + c));
            return undefined;
          };
        }
        if ((isText || isComment) && prop === 'insertData') {
          return function (offset, s) {
            if (!handle) return undefined;
            var cur = __zw_get_text_handle(handle);
            var o = offset | 0;
            if (o < 0) o = 0;
            __zw_set_text_handle(handle, cur.slice(0, o) + String(s == null ? '' : s) + cur.slice(o));
            return undefined;
          };
        }
        if ((isText || isComment) && prop === 'replaceData') {
          return function (offset, count, s) {
            if (!handle) return undefined;
            var cur = __zw_get_text_handle(handle);
            var o = offset | 0, c = count | 0;
            if (o < 0) o = 0;
            if (c < 0) c = 0;
            __zw_set_text_handle(handle, cur.slice(0, o) + String(s == null ? '' : s) + cur.slice(o + c));
            return undefined;
          };
        }
        if ((isText || isComment) && prop === 'substringData') {
          return function (offset, count) {
            if (!handle) return '';
            var cur = __zw_get_text_handle(handle);
            var o = offset | 0, c = count | 0;
            if (o < 0) o = 0;
            if (c < 0) c = 0;
            return cur.slice(o, o + c);
          };
        }
        // Text.splitText(offset)——在 offset 拆分：原节点保 [0,offset)，返新 text 节点含 [offset,)。
        // 仅 text（comment 无 splitText）。offset clamp 到 [0,length]；新节点经 createTextNode 建（handle-based，可后续编辑）。
        if (isText && prop === 'splitText') {
          return function (offset) {
            var cur = handle ? __zw_get_text_handle(handle) : '';
            var o = offset | 0;
            if (o < 0) o = 0;
            if (o > cur.length) o = cur.length;
            var tail = cur.slice(o);
            if (handle) __zw_set_text_handle(handle, cur.slice(0, o));
            return globalThis.document.createTextNode(tail);
          };
        }
        if (prop === 'ownerDocument') {
          return globalThis.document;
        }
        if (prop === 'getAttribute') {
          return function(name) {
            var n = String(name);
            // R2995：sel-based 走 latest-wins 变体（consult 变更列表，闭合 removeAttribute 后 stale 旧值）；
            // 回调未注册（polyfill/其它环境）→ fallback 纯快照 `__zw_get_attr`。
            var v = handle
              ? __zw_get_attr_handle(handle, n)
              : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, n) : __zw_get_attr(sel, n));
            // R3190：spec getAttribute——缺省（属性不存在）须返 null，present-empty 返 ""。host `__zw_get_attr*`
            // 对缺省与空值均返 ""，仅当结果为 "" 时用 `__zw_has_attr*` 区分（常见非空值单次 host 调用，无额外
            // 开销；同 R3187 contentEditable / R3188 default_draggable has_attr 模式）。附带修复 `_matchAttrOf`
            // 的 `[attr]` 存在性选择器 over-match（旧 "" != null 恒真 → 缺省元素误匹配）。
            if (v !== '') return v;
            var present = (handle
              ? __zw_has_attr_handle(handle, n)
              : (typeof __zw_has_attr_lw === 'function' ? __zw_has_attr_lw(sel, n) : __zw_has_attr(sel, n))) === '1';
            return present ? '' : null;
          };
        }
        // js-dom M4 R42：`getAttributeNode(name)`（spec `dom-element-getattributenode`）——返 Attr 节点
        //（经 _zwMakeAttr，instanceof Attr true；ownerElement=本元素 proxy），缺省 null。WPT
        // Range-attribute-nodes（Attr 作为 Range 端点容器）。getAttributeNodeNS 同理（忽略 ns 按限定名查，
        // 与 getAttributeNS 的 _nsQualName 一致语义）。
        if (prop === 'getAttributeNode' || prop === 'getAttributeNodeNS') {
          return function(a, b) {
            var n = prop === 'getAttributeNode' ? String(a) : _nsQualName(a, b);
            var self = proxy;
            var v = handle
              ? __zw_get_attr_handle(handle, n)
              : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, n) : __zw_get_attr(sel, n));
            if (v === '' || v == null) {
              var present = (handle
                ? __zw_has_attr_handle(handle, n)
                : (typeof __zw_has_attr_lw === 'function' ? __zw_has_attr_lw(sel, n) : __zw_has_attr(sel, n))) === '1';
              if (!present) return null;
            }
            return _zwMakeAttr(n, v != null ? v : '', self);
          };
        }
        if (prop === 'setAttribute') {
          return function(name, value) {
            var n = String(name);
            var v = String(value);
            // R2992 custom element attributeChangedCallback：变更前读 old（absent → null，spec 一致）。
            var ceEntry = _ceEntryFor(key, sel, handle);
            var ceOld = ceEntry ? _ce_attrValue(sel, handle, n) : null;
            // R3025：MutationObserver attributeOldValue——有 observer 请求时捕获 mutate 前 old value。
            var moId = _mo_id(handle, sel);
            var moOld = _mo_any_wants_attr_old(moId, n) ? _mo_read_attr(sel, handle, n) : null;
            // 同步客户端缓存：class→_classCache、value→_inputValues，使 setAttribute 与
            // classList/className、.value getter 协作一致（否则后续 classList.add 读 stale 缓存丢值）。
            if (n === 'class') _classCache[key] = v;
            else if (n === 'value') { _inputValues[key] = v; _clearInputDefault(key); } // R2996：setAttribute('value') 重同步 defaultValue
            else if (n === 'checked' || n === 'selected') _clearBoolDefault(key, n); // R2998：setAttribute('checked'/'selected') 重同步 defaultChecked/defaultSelected
            if (handle) __zw_set_attr_handle(handle, n, v);
            else __zw_set_attr(sel, n, v);
            _mo_notify(sel, handle, { type: 'attributes', attributeName: n, oldValue: moOld });
            if (ceEntry) _ce_dispatchAttrChange(ceEntry, proxy, n, ceOld, v);
          };
        }
        if (prop === 'removeAttribute') {
          return function(name) {
            var n = String(name);
            var nLower = n.toLowerCase();
            var targetTag = _realTag(sel, handle);
            // R2992 custom element attributeChangedCallback：移除前读 old（newVal=null）。
            var ceEntry = _ceEntryFor(key, sel, handle);
            var ceOld = ceEntry ? _ce_attrValue(sel, handle, n) : null;
            // R3025：MutationObserver attributeOldValue——移除前捕获 old value（有 observer 请求时）。
            var moId = _mo_id(handle, sel);
            var moOld = _mo_any_wants_attr_old(moId, n) ? _mo_read_attr(sel, handle, n) : null;
            // 真移除（区别于 set-empty 残留 `attr=""`——boolean 属性 checked/disabled 设空值仍 present
            // → hasAttribute 误 true）。handle 元素经 `__zw_remove_attr_handle`（RemoveAttrOnHandle，R2993）；
            // sel-based 经 `__zw_remove_attr`（RemoveAttr，R2657）；无回调 → fallback set-empty。
            // 同步客户端缓存（class/value），使后续 classList/.value 反映移除。
            if (n === 'class') _classCache[key] = '';
            else if (n === 'value') { _inputValues[key] = ''; _clearInputDefault(key); } // R2996：removeAttribute('value') 重同步 defaultValue
            else if (n === 'checked' || n === 'selected') _clearBoolDefault(key, n); // R2998：removeAttribute('checked'/'selected') 重同步 defaultChecked/defaultSelected
            if (nLower === 'popover') delete _zwTopLayer[key];
            if (nLower === 'open' && targetTag === 'DIALOG') {
              delete _zwDialogModal[key];
              delete _zwTopLayer[key];
            }
            if (handle && typeof __zw_remove_attr_handle === 'function') __zw_remove_attr_handle(handle, n);
            else if (handle) __zw_set_attr_handle(handle, n, '');
            else if (typeof __zw_remove_attr === 'function') __zw_remove_attr(sel, n);
            else __zw_set_attr(sel, n, '');
            _mo_notify(sel, handle, { type: 'attributes', attributeName: n, oldValue: moOld });
            if (ceEntry) _ce_dispatchAttrChange(ceEntry, proxy, n, ceOld, null);
          };
        }
        // `el.hasAttribute(name)`——属性存在性（boolean 属性 checked/disabled/hidden、data-* 检查常用）。
        // handle 句柄元素（createElement 创建）经 host `__zw_has_attr_handle`（R2832 引入，存于 mutation 列表）；
        // sel-based 元素经 host `__zw_has_attr_lw`（R2995 latest-wins，闭合 removeAttribute 后 stale true），
        // 回调未注册 → fallback 纯快照 `__zw_has_attr`。此前 handle-only 元素恒 false（latent bug，R2992 修）。
        if (prop === 'hasAttribute') {
          return function(name) {
            var n = String(name);
            if (handle && typeof __zw_has_attr_handle === 'function') {
              try { return __zw_has_attr_handle(handle, n) === '1'; } catch (_e) { return false; }
            }
            if (sel) {
              if (typeof __zw_has_attr_lw === 'function') {
                try { return __zw_has_attr_lw(sel, n) === '1'; } catch (_e) { return false; }
              }
              if (typeof __zw_has_attr === 'function') {
                try { return __zw_has_attr(sel, n) === '1'; } catch (_e) { return false; }
              }
            }
            return false;
          };
        }
        // R3024：命名空间属性族（SVG/MathML/xlink 高频，如 setAttributeNS('http://www.w3.org/1999/xlink','href',v)）。
        // HTML 元素 ns 忽略——setAttributeNS 限定名（qualifiedName，含 prefix:local）原样按 name 字符串存
        //（host 按 name 存无 ns 解析，HTML 渲染不依赖 ns；SVG-in-HTML 序列化 round-trip 经 parser 再 split）。
        // R3217：get/has/removeAttributeNS 按 **ns+local** 查找——setAttributeNS 存 'prefix:local' 限定名，
        // 读侧用 ns→常规 prefix 映射（xlink/xml/xmlns，SVG 高频）重构限定名；null/空 ns → 裸 local（无命名空间
        // 属性）。闭合旧「读 local 查不到 setAttributeNS 的 prefix:local」不一致。spec 按 ns+localName 匹配，
        // 本实现按限定名字符串存故 ns→prefix 重构；自定义 ns 无常规 prefix 或 setAttributeNS 用非常规 prefix
        // 时回落裸 local（罕见，记限）。
        if (prop === 'setAttributeNS') {
          return function(_ns, qualifiedName, value) { proxy.setAttribute(String(qualifiedName), value); };
        }
        if (prop === 'getAttributeNS') {
          return function(ns, localName) { return proxy.getAttribute(_nsQualName(ns, localName)); };
        }
        if (prop === 'hasAttributeNS') {
          return function(ns, localName) { return proxy.hasAttribute(_nsQualName(ns, localName)); };
        }
        if (prop === 'removeAttributeNS') {
          return function(ns, localName) { return proxy.removeAttribute(_nsQualName(ns, localName)); };
        }
        // `el.focus()` / `el.blur()`——焦点状态追踪（document.activeElement 对）+ 焦点事件派发（R3247）。
        // 纯 in-JS 状态：focus 记当前 key，blur 清当前 key。**已知限制**：① 无真键盘焦点（纯状态，无输入焦点点亮）；
        // ③ 不校验可聚焦性（非聚焦元素仍记焦点）；④ 无 tabindex 焦点序。
        // R3247：派发 focus/blur/focusin/focusout 事件（DOM §3.3 Focus + UI Events）。焦点 old→new 序：
        // focusout(旧,bubbles) → focus(新) → focusin(新,bubbles) → blur(旧)。blur()：focusout(bubbles) → blur。
        // 均不可取消（cancelable:false）。仅焦点真变时派发（已聚焦元素 focus() no-op，spec 不重派 focus）。
        if (prop === 'focus') {
          return function() {
            var oldKey = _activeElKey;
            if (oldKey === key) return; // 已聚焦 → no-op（spec：不重派 focus）
            var oldProxy = (oldKey && _proxyCache[oldKey]) ? _proxyCache[oldKey] : null;
            _activeElKey = key; // 先更状态防 handler 重入（focus handler 再 focus 其它元素时序自洽）
            if (oldProxy) {
              try { oldProxy.dispatchEvent(_makeEvent('focusout', { bubbles: true, cancelable: false })); } catch (_e) {}
            }
            try {
              _dispatchWithBubble(key, sel, handle, _makeEvent('focus', { bubbles: false, cancelable: false }));
              _dispatchWithBubble(key, sel, handle, _makeEvent('focusin', { bubbles: true, cancelable: false }));
            } catch (_e) {}
            if (oldProxy) {
              try { oldProxy.dispatchEvent(_makeEvent('blur', { bubbles: false, cancelable: false })); } catch (_e) {}
            }
            // R3254-M7'：通知宿主同步 retained 焦点状态（键盘路由 + 滚动守卫）。空串 → host 不采纳
            //（selector 缺失的 focus 无稳定目标）；宿主侧另有 is_focusable_selector 校验兜底。
            if (sel && typeof __zw_focus_changed === 'function') __zw_focus_changed(sel);
          };
        }
        if (prop === 'blur') {
          return function() {
            if (_activeElKey !== key) return; // 非当前焦点元素 → no-op
            _activeElKey = null;
            try {
              _dispatchWithBubble(key, sel, handle, _makeEvent('focusout', { bubbles: true, cancelable: false }));
              _dispatchWithBubble(key, sel, handle, _makeEvent('blur', { bubbles: false, cancelable: false }));
            } catch (_e) {}
            // R3254-M7'：通知宿主失焦（空串表示 blur）。
            if (typeof __zw_focus_changed === 'function') __zw_focus_changed('');
          };
        }
        // R2938 `el.requestFullscreen()`——全屏请求（spec 返 Promise，headless 无真 OS 全屏）。grant/deny 二分：
        // fullscreenEnabled=true（默认）→ 设 fullscreenElement + 派 fullscreenchange + resolve；=false →
        // 派 fullscreenerror + reject TypeError（spec「fullscreen is unavailable」）。相同元素重复请求 → no-op
        // resolve（不重复派 fullscreenchange，spec 一致）。`key/sel/handle` 为闭包捕获的本元素身份。
        if (prop === 'requestFullscreen') {
          return function() {
            return new Promise(function (resolve, reject) {
              if (!globalThis.document.fullscreenEnabled) {
                _fireDocEvent('fullscreenerror');
                reject(new TypeError('fullscreen is unavailable'));
                return;
              }
              if (_fsKey === key) { resolve(undefined); return; } // 已是全屏元素 → no-op
              _fsKey = key;
              _fsSel = sel;
              _fsHandle = handle;
              _fireDocEvent('fullscreenchange');
              resolve(undefined);
            });
          };
        }
        // R2939 `el.requestPointerLock()`——指针锁请求（spec 返 Promise，headless 无真 OS 指针锁）。grant/deny
        // 二分：可用（host `__zw_pointer_lock_enabled` 未注册或返 '1'，默认）→ 设 pointerLockElement + 派
        // pointerlockchange + resolve；禁用（返 '0'）→ 派 pointerlockerror + reject TypeError。相同元素重复
        // 请求 → no-op resolve。镜像 R2938 Fullscreen。
        // https://w3c.github.io/pointerlock/#dom-element-requestpointerlock
        if (prop === 'requestPointerLock') {
          return function () {
            return new Promise(function (resolve, reject) {
              if (typeof __zw_pointer_lock_enabled === 'function') {
                var ok = true;
                try { ok = __zw_pointer_lock_enabled() === '1'; } catch (_e) {}
                if (!ok) {
                  _fireDocEvent('pointerlockerror');
                  reject(new TypeError('pointer lock is unavailable'));
                  return;
                }
              }
              if (_plKey === key) { resolve(undefined); return; } // 已锁定元素 → no-op
              _plKey = key;
              _plSel = sel;
              _plHandle = handle;
              _fireDocEvent('pointerlockchange');
              resolve(undefined);
            });
          };
        }
        // R3047：scroll 方法（headless 无真视口滚动 → JS-side 状态追踪）。scrollTo/scrollBy 更新 `_scrollOffsets[key]`
        //（与 scrollTop/scrollLeft getter 自洽）；scrollIntoView 无 viewport → no-op（documented）。
        // 参数支持 `(x,y)` 与 `{left,top,behavior}` 两 spec 形式（_zwApplyScroll 统一解析）。
        // R3068：Pointer Capture API（setPointerCapture/releasePointerCapture/hasPointerCapture）。headless 无真
        // 指针路由（事件不重定向到捕获元素），但 API 表面 + hasPointerCapture 状态查询对指针/拖拽库必需。
        // per-element `_pointerCapture[key]` set 形态追踪。permissive：不校验 pointerId active（spec NotFoundError defer）。
        if (prop === 'setPointerCapture') {
          return function (pid) {
            var id = String(pid);
            var set = _pointerCapture[key] || (_pointerCapture[key] = {});
            set[id] = true;
          };
        }
        if (prop === 'releasePointerCapture') {
          return function (pid) {
            var set = _pointerCapture[key];
            if (set) delete set[String(pid)];
          };
        }
        if (prop === 'hasPointerCapture') {
          return function (pid) {
            var set = _pointerCapture[key];
            return !!(set && set[String(pid)]);
          };
        }
        // R3071：Popover API 三方法（showPopover/hidePopover/togglePopover）。headless 无真 top-layer paint /
        // 渲染层级 / :popover-open（rendering defer），本切片实现 JS-observable 状态机 + beforetoggle/toggle 事件。
        // top-layer 成员集 `_zwTopLayer`（key→true 即 showing）。togglePopover 翻转。详见 `_zwShowPopover/_zwHidePopover`。
        // https://html.spec.whatwg.org/multipage/popover.html#dom-showpopover
        if (prop === 'showPopover') {
          return function () { _zwShowPopover(key, sel, handle); };
        }
        if (prop === 'hidePopover') {
          return function () { _zwHidePopover(key, sel, handle); };
        }
        if (prop === 'togglePopover') {
          return function () {
            if (_zwTopLayer[key]) _zwHidePopover(key, sel, handle);
            else _zwShowPopover(key, sel, handle);
          };
        }
        // R3290：HTMLDialogElement 三方法（show/showModal/close）。仅向 dialog proxy 暴露，
        // 状态机校验经 _zwDialog* helper（part01.js）。dialog 元素 feature-detect + 调 show/showModal/close +
        // 监听 'close' 事件不中断（模态/对话框 UI 库 high-frequency 路径）。headless 无真 top-layer paint /
        // ::backdrop / focus 陷阱 / inert backdrop（rendering 流域 defer）——仅 JS-observable 状态（open 属性 + 模态态）。
        // https://html.spec.whatwg.org/multipage/interactive-elements.html#htmldialogelement
        if (prop === 'show' && _realTag(sel, handle) === 'DIALOG') {
          return function () { _zwDialogShow(key, sel, handle); };
        }
        if (prop === 'showModal' && _realTag(sel, handle) === 'DIALOG') {
          return function () { _zwDialogShowModal(key, sel, handle); };
        }
        if (prop === 'close' && _realTag(sel, handle) === 'DIALOG') {
          return function (returnValue) { return _zwDialogClose(key, sel, handle, returnValue); };
        }
        // R3290：HTMLDialogElement.open / HTMLDetailsElement.open ——boolean 反射 open 内容属性。
        // spec `<details>` 与 `<dialog>` 均有 open IDL 布尔属性（presence-based）。此前仅作原始内容属性（getAttribute），
        // `el.open` 返 undefined → dialog/details feature-detect + `if (dlg.open)` 控制流断。暴露于全部元素（与 disabled 等
        // 全局反射属性同设计），presence 判定。latest-wins 反映同 execute 内 pending set/remove（show/showModal/close 经
        // __zw_set_attr/__zw_remove_attr 异步入队，纯快照读 stale）。
        // https://html.spec.whatwg.org/multipage/interactive-elements.html#dom-dialog-open
        // https://html.spec.whatwg.org/multipage/interactive-elements.html#dom-details-open
        if (prop === 'open') return _zwDialogHasOpen(sel, handle);
        // R3290：HTMLDialogElement.returnValue——串属性。close(rv) 设值；非 undefined → 存 _expando[key::returnValue]；
        // getter 返存储值（默认 ''，spec 空 dialog returnValue 为 ''）。直接 setAttribute 不可达（IDL 属性 setter
        // 不反射内容属性——dialog 无 returnValue 内容属性，仅 IDL），故存 _expando。setter 任意值 → 串。
        // https://html.spec.whatwg.org/multipage/interactive-elements.html#dom-dialog-returnvalue
        if (prop === 'returnValue' && _realTag(sel, handle) === 'DIALOG') return _expando[key + '::returnValue'] || '';
        if (prop === 'scrollTo' || prop === 'scrollBy') {
          var _ss = _scrollOffsets[key] || (_scrollOffsets[key] = { top: 0, left: 0 });
          var _byM = prop === 'scrollBy';
          return function (a, b) { _zwApplyScroll(_ss, a, b, _byM); _zwFireScroll(key, sel, handle); };
        }
        if (prop === 'scrollIntoView') {
          // R3060：scrollIntoView 真实化（闭合 R3047 no-op）。headless 无真视口滚动，但程序化 round-trip
          //（"回到顶部"按钮判 scrollY / 滚动后读位置 / anchor 滚动）可观察：把文档 scrollTop 设为元素 gBCR.y
          //（元素滚到视口顶），复用 globalThis.scrollTo（更新 _winScroll + 派发 scroll 事件，R3047/R3051）。
          // arg（boolean / {block,behavior,inline}）解析 block 对齐：headless 无 viewportH，center/end 近似为 top
          //（documented；viewportH>0 时按 spec 算）。gBCR 未注册（reftest/polyfill/WebView 无 rect bridge）→ rect 零 → no-op。
          return function (arg) {
            var identity = sel || handle;
            if (!identity || typeof __zw_getBoundingClientRect !== 'function') return;
            var rs;
            try { rs = __zw_getBoundingClientRect(identity); } catch (_e) { rs = ''; }
            if (!rs || rs.indexOf(',') < 0) return; // 无 rect（detached / bridge 未注册）→ no-op
            var p = rs.split(',');
            var top = +p[1] || 0; // y
            var h = +p[3] || 0;
            // block 对齐：start（默认/true）→ top；end（false）→ top+h-vh；center → top-vh/2+h/2。
            var vh = (typeof globalThis.innerHeight === 'number' && globalThis.innerHeight > 0) ? globalThis.innerHeight : 0;
            var block = 'start';
            if (arg === false) block = 'end';
            else if (arg && typeof arg === 'object' && arg.block) block = String(arg.block).toLowerCase();
            var newTop = top;
            if (vh > 0) {
              if (block === 'end') newTop = top + h - vh;
              else if (block === 'center') newTop = top - vh / 2 + h / 2;
            }
            if (newTop < 0) newTop = 0;
            globalThis.scrollTo(0, newTop); // behavior: instant（smooth 无动画，headless documented）
          };
        }
        // `el.scrollIntoViewIfNeeded(centerIfNeeded?)`（R3075）——WebKit-only（Safari/旧 Chrome）。仅不可见时滚；
        // centerIfNeeded=true → 居中。headless 无 viewport 可见性判定 → 近似始终滚（"if needed" defer，
        // documented），委托 scrollIntoView（R3060）复用 gBCR + scrollTo：centerIfNeeded→center，否则 nearest（最小滚动近似）。
        // API 表面价值：Safari-compat 库（WebKit feature-detect + 调用）不 TypeError。real browser 仅不可见时滚。
        if (prop === 'scrollIntoViewIfNeeded') {
          return function (centerIfNeeded) {
            _makeProxy(sel, handle).scrollIntoView({ block: centerIfNeeded ? 'center' : 'nearest' });
          };
        }
        // `el.hasAttributes()`——是否有任意属性。R3197：handle 经 `__zw_attr_names_handle`（属性名仅来自
        // mutations，无快照基底），sel 经 `__zw_attr_names`（latest-wins）。旧 handle 路径恒返 false。
        if (prop === 'hasAttributes') {
          return function() {
            try {
              if (handle && typeof __zw_attr_names_handle === 'function') {
                return __zw_attr_names_handle(handle).length > 0;
              }
              if (sel && typeof __zw_attr_names === 'function') {
                return __zw_attr_names(sel).length > 0;
              }
            } catch (_e) {}
            return false;
          };
        }
        // `el.getAttributeNames()`——属性名数组（文档序）。R3197：handle 经 `__zw_attr_names_handle`
        //（属性名仅来自 mutations），sel 经 `__zw_attr_names`（latest-wins）。旧 handle 路径恒返 []。
        if (prop === 'getAttributeNames') {
          return function() {
            try {
              var n = handle
                ? (typeof __zw_attr_names_handle === 'function' ? __zw_attr_names_handle(handle) : '')
                : (typeof __zw_attr_names === 'function' ? __zw_attr_names(sel) : '');
              return n ? n.split('|').filter(Boolean) : [];
            } catch (_e) { return []; }
          };
        }
        // `el.toggleAttribute(name, force?)`——切换属性存在性，返切换后是否存在。R3192：host
        // `__zw_toggle_attribute` **enqueue-时解析**决策（计算 latest-wins presence → 入队具体 SetAttr/
        // RemoveAttr），返 `"1"`/`"0"`（post-toggle presence）——连续 toggle / set-then-toggle 返值均准确
        //（闭合 R3191 已知限制：连续 toggle 返值 stale）。handle-only / 无 host 回调回落 client-side 决策。
        if (prop === 'toggleAttribute') {
          return function(name, force) {
            var n = String(name);
            var hasForce = force !== undefined;
            // R3025：MutationObserver attributeOldValue——toggle 前捕获 old value（有 observer 请求时）。
            var moOld = _mo_any_wants_attr_old(_mo_id(handle, sel), n) ? _mo_read_attr(sel, handle, n) : null;
            if (sel && typeof __zw_toggle_attribute === 'function') {
              var fArg = hasForce ? (force ? '1' : '0') : '';
              var res = __zw_toggle_attribute(sel, n, fArg); // enqueue-时解析，返 post-toggle presence。
              _mo_notify(sel, handle, { type: 'attributes', attributeName: n, oldValue: moOld });
              return res === '1';
            }
            // handle-only / fallback（无 host toggle 回调）：client-side 决策（latest-wins presence）。
            var snapHas = sel
              ? ((typeof __zw_has_attr_lw === 'function'
                  ? __zw_has_attr_lw(sel, n)
                  : (typeof __zw_has_attr === 'function' ? __zw_has_attr(sel, n) : '0')) === '1')
              : false;
            if (handle) {
              var want = hasForce ? !!force : !snapHas;
              if (want) __zw_set_attr_handle(handle, n, '');
            }
            return hasForce ? !!force : !snapHas;
          };
        }
        // `el.attributes`（NamedNodeMap 只读快照）——属性枚举（序列化/属性拷贝常用）。
        if (prop === 'attributes') {
          return _attributesProxy(sel, handle);
        }
        // `el.matches(selector)` / `el.matchesSelector`——元素是否匹配选择器（含组合器，经 host
        // `__zw_matches` 全匹配集判定）。handle（未挂载 DOM 的 createElement）无 sel → false。
        if (prop === 'matches' || prop === 'matchesSelector' || prop === 'webkitMatchesSelector') {
          return function(selector) {
            if (!sel || typeof __zw_matches !== 'function') return false;
            try { return __zw_matches(sel, String(selector)) === '1'; } catch (_e) { return false; }
          };
        }
        // `el.closest(selector)`——自身或最近祖先首个匹配元素（proxy），无匹配 null。经 host
        // `__zw_closest`（parent_node 链 + 全匹配集），返唯一选择器后包 proxy。
        if (prop === 'closest') {
          return function(selector) {
            if (!sel || typeof __zw_closest !== 'function') return null;
            try {
              var hit = __zw_closest(sel, String(selector));
              return hit ? _wrapSelector(hit) : null;
            } catch (_e) { return null; }
          };
        }
        // `el.checkVisibility(options)`（R3074）——元素是否「being rendered」+ 可选 opacity/visibility 检查。
        // ad viewability / lazy-load / 视图追踪库用。委托 `_zwCheckVisibility`（getComputedStyle + 祖先链）。
        // https://drafts.csswg.org/cssom-view-1/#dom-element-checkvisibility
        if (prop === 'checkVisibility') {
          return function(options) { return _zwCheckVisibility(sel, handle, options); };
        }
        // R2933 element 级 IDL on-event handler getter（onclick/oninput/... → 存储的 fn 或 null）。
        // `on`+小写字母 = handler（generic，无白名单）。与 set trap 的 on* 路由对偶。
        // R2934：无 JS 设值时回落编译 inline on* 属性（<button onclick="...">）。
        if (typeof prop === 'string' && /^on[a-z]/.test(prop)) {
          var _gt = String(prop).slice(2);
          _ensureInlineHandler(key, sel, handle, _gt);
          return (_onHandlers[key] && _onHandlers[key][_gt]) || null;
        }
        if (prop === 'addEventListener') {
          return function(type, fn, opts) {
            if (!_listenerStore[key]) _listenerStore[key] = {};
            if (!_listenerStore[key][type]) _listenerStore[key][type] = [];
            _listenerStore[key][type].push({ fn: fn, capture: _optCapture(opts), once: _optOnce(opts) });
          };
        }
        if (prop === 'removeEventListener') {
          return function(type, fn, opts) {
            if (!_listenerStore[key] || !_listenerStore[key][type]) return;
            var cap = _optCapture(opts);
            _listenerStore[key][type] = _listenerStore[key][type].filter(function(l) {
              return !(l.fn === fn && l.capture === cap);
            });
          };
        }
        if (prop === 'attachEvent') {
          return function(type, fn) {
            _attachEventForKey(key, type, fn);
          };
        }
        if (prop === 'detachEvent') {
          return function(type, fn) {
            _detachEventForKey(key, type, fn);
          };
        }
        if (prop === 'dispatchEvent') {
          return function(event) {
            return _dispatchWithBubble(key, sel, handle, event);
          };
        }
        if (prop === 'click') {
          return function() {
            var ev = _makeEvent('click', { bubbles: true, cancelable: true });
            var notPrevented = _dispatchWithBubble(key, sel, handle, ev);
            // R3072：popovertarget 声明式触发——click default action（未 preventDefault 时）。找最近含 popovertarget
            // 祖先 → 按 popovertargetaction 触发目标 popover show/hide/toggle。无 popovertarget 时 no-op（零回归）。
            if (notPrevented) _zwPopoverTargetActivate(key, sel, handle);
            return notPrevented;
          };
        }
        // Constraint Validation API（R2825）——表单校验库（checkValidity gate submit / setCustomValidity
        // 自定义错误 / validity.valid 读 / validationMessage 显示）高频。customError 由 _customValidity 跟踪；
        // 原生约束 headless 不强制（permissive valid）。checkValidity/reportValidity invalid 时派发 'invalid'
        // 事件（cancelable，非 bubble，经 _dispatchWithBubble）。
        if (prop === 'checkValidity' || prop === 'reportValidity') {
          return function() {
            var v = _validityState(key, sel, handle);
            if (!v.valid) {
              _dispatchWithBubble(key, sel, handle, _makeEvent('invalid', { cancelable: true, bubbles: false }));
            }
            return v.valid;
          };
        }
        if (prop === 'setCustomValidity') {
          return function(msg) {
            _customValidity[key] = (msg == null) ? '' : String(msg);
            return undefined;
          };
        }
        if (prop === 'validity') return _validityState(key, sel, handle);
        if (prop === 'validationMessage') {
          var validity = _validityState(key, sel, handle);
          if (validity.customError) return _customValidity[key];
          if (validity.tooShort) return 'Please lengthen this text.';
          if (validity.tooLong) return 'Please shorten this text.';
          return '';
        }
        if (prop === 'willValidate') return true;
        // `el.select()`（HTMLInputElement/TextArea，R2826/R2844）——选中文本（legacy copy 模式
        // `el.select(); document.execCommand('copy')` 配对，及自动全选场景）。headless 无真文本选择渲染，
        // 但 text control（R2844）须更新 _textSelection 使后续 selectionStart/End 反映全选（Chromium 150
        // oracle：select()→{0, value.length, 'forward'}）；非 text control 仍 no-op（无选区概念）。
        if (prop === 'select') {
          return function() {
            if (_isTextControl(sel, handle)) {
              var so = _selObj(key);
              so.start = 0;
              so.end = _controlValue(sel, handle, key).length;
              so.direction = 'forward';
            }
            return undefined;
          };
        }
        // `el.animate(keyframes, options)`（Web Animations API，R2827 stub → R2965 真关键帧应用）。modern 动画库
        //（Framer Motion / GSAP / Lottie）feature-detect + 链式。headless 无真时间轴 → `_makeAnimation` 瞬间完成
        //（playState 'running'→'finished' + finished Promise + onfinish）；R2965 起 finish 时按 fill 把末关键帧
        // 写入 inline style（终态经样式管线可见）。`sel/handle` 为闭包捕获的本元素身份，透传给 _makeAnimation。
        if (prop === 'animate') {
          return function (keyframes, options) { return _makeAnimation(keyframes, options, sel, handle); };
        }
        // R3067：`el.getAnimations()`（Web Animations API）——返本元素动画（cancelled/idle 排除；finished 含）。
        // 读注册表在调用时（非 get-trap 时）以反映 reset 后空态 + 后续新增动画。返副本数组（防调用方 mutate 注册表）。
        if (prop === 'getAnimations') {
          return function () {
            var _anims = _elementAnimations[key] || [];
            return _anims.filter(function (a) { return a && a.playState !== 'idle'; }).slice();
          };
        }
        // `el.cloneNode(deep)`——克隆元素（返新 handle proxy，detached）。复用既有回调组合：
        // create(tag) + 逐属性 set_attr_handle + (deep) set_inner_html_handle。R3198：handle 源经
        // `__zw_get_tag_handle`/`__zw_attr_names_handle`/`__zw_get_attr_handle`（旧 handle 源 tag 回落 'div' +
        // 不复制属性，best-effort 因当时无 handle 枚举回调）；sel 源经 `__zw_get_tag`/`__zw_attr_names`/
        // `__zw_get_attr`（latest-wins）。两端源均完整复制 tag + 属性 +（deep）后代。
        // `Node.normalize()`（R2853）——合并相邻 Text 子节点 + 移除空 Text。snapshot 模型下元素文本为
        // 单一串（无独立 Text 子节点暴露），故 normalize 为语义正确的 no-op（DOM 态已「normalized」）。
        // 提供 no-op 防 `el.normalize()` 防御性调用（rich-text 编辑器 / innerHTML 后清理）抛 TypeError。
        if (prop === 'normalize') {
          return function() {};
        }
        if (prop === 'cloneNode') {
          return function(deep) {
            // R3198：源 tag——handle 经 `__zw_get_tag_handle`，sel 经 `__zw_get_tag`（旧 handle 回落 'div'）。
            var srcTag = 'div';
            try {
              var t = handle
                ? (typeof __zw_get_tag_handle === 'function' ? __zw_get_tag_handle(handle) : '')
                : (sel && typeof __zw_get_tag === 'function' ? __zw_get_tag(sel) : '');
              if (t) srcTag = t;
            } catch (_e) {}
            var nh = __zw_create_element(srcTag);
            // 复制属性（名 + 值）。R3198：handle 源经 `__zw_attr_names_handle`+`__zw_get_attr_handle`，
            // sel 源经 `__zw_attr_names`（latest-wins，自 R3002）+ 值。R3203：sel 源值改走 `__zw_get_attr_lw`
            //（latest-wins，与名源同 lw）——旧纯快照 `__zw_get_attr` 致 `setAttribute('x','v'); cloneNode()` 复制 stale 值
            //（名 lw 含 x 但值读 stale 快照），handle 源本就读 mutations latest-wins 无此问题。
            try {
              var names = handle
                ? (typeof __zw_attr_names_handle === 'function' ? __zw_attr_names_handle(handle) : '')
                : (sel && typeof __zw_attr_names === 'function' ? __zw_attr_names(sel) : '');
              if (names) {
                names.split('|').filter(Boolean).forEach(function(n) {
                  var v = handle
                    ? (typeof __zw_get_attr_handle === 'function' ? __zw_get_attr_handle(handle, n) : '')
                    : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, n) : __zw_get_attr(sel, n));
                  __zw_set_attr_handle(nh, n, v || '');
                });
              }
            } catch (_e) {}
            // deep：复制后代（innerHTML）。
            if (deep) {
              try {
                var ih = handle
                  ? __zw_get_inner_html_handle(handle)
                  : (sel ? __zw_get_inner_html(sel) : null);
                if (ih) __zw_set_inner_html_handle(nh, ih);
              } catch (_e) {}
            }
            return _wrapHandle(nh);
          };
        }
        if (prop === 'appendChild') {
          return function(child) {
            // R34xx：重新插入清除移除标记（append 后元素回到文档）。
            if (sel) _zwUnmarkRemoved(sel);
            if (child && child.__zwSelector) _zwUnmarkRemoved(child.__zwSelector);
            if (child && child.__zwHandle) {
              // R2994：捕获实际入树的顶层节点（fragment flatten 前取其子），供连接态传播。
              var ceAdded;
              // DocumentFragment：flatten 子节点到 this（fragment 自身不入树），区别于 append 节点自身。
              if (_fragmentHandles[child.__zwHandle] && typeof __zw_append_fragment_children === 'function') {
                ceAdded = (_handleChildren[child.__zwHandle] || []).slice();
                if (handle) __zw_append_fragment_children_handle(handle, child.__zwHandle);
                else __zw_append_fragment_children(sel, child.__zwHandle);
              } else if (handle) {
                ceAdded = [child];
                __zw_append_child_handle(handle, child.__zwHandle);
              } else {
                ceAdded = [child];
                __zw_append_child(sel, child.__zwHandle);
              }
              // R2927/R2928：handle 父（任意 handle 元素，非仅容器）同步记录子节点到 registry。
              // 容器（shadow/fragment）的 childNodes 读 registry；R2928 querySelector 亦遍历完整
              // handle 子树（须递归普通 created 元素的 handle 子），故所有 handle 父都记录。
              if (handle) {
                _recordHandleChild(handle, child);
              } else if (_fragmentHandles[child.__zwHandle]) {
                // sel-based 父接 fragment → fragment 清空（spec：fragment append 后空；handle 父已在
                // _recordHandleChild 内 flatten 清空）。
                _handleChildren[child.__zwHandle] = [];
              }
              _mo_notify(sel, handle, { type: 'childList', addedNodes: [child], removedNodes: [] });
              // R2994 connectedCallback：子树按父连接态传播（父连入 → 子树连入；未观察/非 custom 仅传播）。
              var cePconn = _ceParentConnected(sel, handle);
              for (var ci = 0; ci < ceAdded.length; ci++) _ceApplyConn(ceAdded[ci], cePconn);
            }
            return child;
          };
        }
        if (prop === 'removeChild') {
          return function(child) {
            if (child && child.__zwHandle) {
              // R2994：移除前快照连接态（移除后 host 快照变化，但 _ceConn 为 JS 端追踪，移除调用不影响）。
              __zw_remove_handle(child.__zwHandle);
              // R2927/R2928：handle 父同步从 registry 移除子节点（保持 querySelector 子树一致）。
              if (handle) _unrecordHandleChild(handle, child);
              _mo_notify(sel, handle, { type: 'childList', addedNodes: [], removedNodes: [child] });
              // R2994 disconnectedCallback：移除子树断连（仅此前已连入的 custom element 分派）。
              _ceApplyConn(child, false);
            }
            return child;
          };
        }
        if (prop === 'insertBefore') {
          return function(newNode, refNode) {
            if (newNode && newNode.__zwHandle) {
              // R2994：捕获实际入树的顶层节点（fragment flatten 前取其子）。
              var ceAdded;
              // DocumentFragment：flatten 子节点（refNode 非 null 时插到 ref 前，null 时 append）。
              if (_fragmentHandles[newNode.__zwHandle]) {
                ceAdded = (_handleChildren[newNode.__zwHandle] || []).slice();
                if (refNode == null) {
                  if (handle && typeof __zw_append_fragment_children_handle === 'function')
                    __zw_append_fragment_children_handle(handle, newNode.__zwHandle);
                  else if (typeof __zw_append_fragment_children === 'function')
                    __zw_append_fragment_children(sel, newNode.__zwHandle);
                } else if (refNode.__zwSelector) {
                  if (handle && typeof __zw_insert_fragment_before_handle === 'function')
                    __zw_insert_fragment_before_handle(handle, newNode.__zwHandle, refNode.__zwSelector);
                  else if (typeof __zw_insert_fragment_before === 'function')
                    __zw_insert_fragment_before(sel, newNode.__zwHandle, refNode.__zwSelector);
                }
              } else if (refNode == null) {
                // `insertBefore(node, null)` 等价于 appendChild。
                ceAdded = [newNode];
                if (handle) __zw_append_child_handle(handle, newNode.__zwHandle);
                else __zw_append_child(sel, newNode.__zwHandle);
              } else if (refNode.__zwSelector) {
                ceAdded = [newNode];
                if (handle) __zw_insert_before_handle(handle, newNode.__zwHandle, refNode.__zwSelector);
                else __zw_insert_before(sel, newNode.__zwHandle, refNode.__zwSelector);
              }
              // refNode 为 create 句柄（无 selector）时不支持（罕见）。
              _mo_notify(sel, handle, { type: 'childList', addedNodes: [newNode], removedNodes: [] });
              // R2994 connectedCallback：子树按父连接态传播。
              if (ceAdded) {
                var cePconn = _ceParentConnected(sel, handle);
                for (var ci = 0; ci < ceAdded.length; ci++) _ceApplyConn(ceAdded[ci], cePconn);
              }
            }
            return newNode;
          };
        }
        // `parent.replaceChild(newChild, oldChild)`：在 oldChild 位置前插入 newChild，再移除
        // oldChild（spec replace 语义）。newChild 须为 create 句柄节点；oldChild 须有 selector
        //（selector-identity 子节点，作 insert ref）。返回 oldChild（spec）。
        if (prop === 'replaceChild') {
          return function(newChild, oldChild) {
            if (newChild && newChild.__zwHandle && oldChild && oldChild.__zwSelector) {
              // R2994：capture added/removed for connection 传播。
              var ceAdded = _fragmentHandles[newChild.__zwHandle]
                ? (_handleChildren[newChild.__zwHandle] || []).slice()
                : [newChild];
              // DocumentFragment：flatten 子到 old 前（非插 fragment 节点本身），再移除 old。
              if (_fragmentHandles[newChild.__zwHandle]) {
                if (handle && typeof __zw_insert_fragment_before_handle === 'function')
                  __zw_insert_fragment_before_handle(handle, newChild.__zwHandle, oldChild.__zwSelector);
                else if (typeof __zw_insert_fragment_before === 'function')
                  __zw_insert_fragment_before(sel, newChild.__zwHandle, oldChild.__zwSelector);
              } else if (handle) {
                __zw_insert_before_handle(handle, newChild.__zwHandle, oldChild.__zwSelector);
              } else {
                __zw_insert_before(sel, newChild.__zwHandle, oldChild.__zwSelector);
              }
              __zw_remove(oldChild.__zwSelector);
              _mo_notify(sel, handle, {
                type: 'childList',
                addedNodes: [newChild],
                removedNodes: [oldChild],
              });
              // R2994：newChild 子树按父连接态连入；oldChild 断连。
              var cePconn = _ceParentConnected(sel, handle);
              for (var ci = 0; ci < ceAdded.length; ci++) _ceApplyConn(ceAdded[ci], cePconn);
              _ceApplyConn(oldChild, false);
            }
            return oldChild;
          };
        }
        if (prop === 'remove') {
          return function() {
            // R2994：移除自身（含 handle 子树）→ 断连（仅此前已连入的 custom element 分派 disconnectedCallback）。
            // R34xx：本地移除标记（同步脚本内 parentNode 立即返 null——host mutation 异步应用）。
            var ceSelf = _makeProxy(sel, handle);
            if (handle) __zw_remove_handle(handle);
            else { __zw_remove(sel); _zwMarkRemoved(sel); }
            _ceApplyConn(ceSelf, false);
          };
        }
        // `element.replaceWith(...nodesOrStrings)`：用新节点序列替换自身（self 级，区别于
        // replaceChild 的 parent 级）。= 先 before(...args) 作前兄弟插入（正序保参数序），再 remove 自身。
        // 复用 _insertAdjacentVariadic（beforebegin 正序）+ remove。仅 sel-based 目标（需 parent）。
        if (prop === 'replaceWith') {
          return function() {
            if (sel) {
              _insertAdjacentVariadic(sel, 'beforebegin', arguments, false);
              if (handle) __zw_remove_handle(handle);
              else __zw_remove(sel);
            }
            return undefined;
          };
        }
        // `Element.append(...nodesOrStrings)`（现代 API，区别于 appendChild）：
        // 追加多个节点/字符串，字符串自动包成 Text 节点。复用既有 appendChild +
        // createTextNode 回调，无需新增 Rust 端 callback。
        if (prop === 'append') {
          return function() {
            var added = _appendVariadic(sel, handle, arguments);
            if (added.length > 0) {
              _mo_notify(sel, handle, { type: 'childList', addedNodes: added, removedNodes: [] });
              // R2994 connectedCallback：新增子按父连接态传播（text 节点非元素，_ceApplyConn 内安全跳过）。
              var cePconn = _ceParentConnected(sel, handle);
              for (var ci = 0; ci < added.length; ci++) _ceApplyConn(added[ci], cePconn);
            }
            return undefined;
          };
        }
        // `element.replaceChildren(...nodesOrStrings)`（现代 API，R2822）：移除全部现有子 + 追加新子
        // （clear-and-populate 原子语义，Vue3/lit/Svelte/手写代码高频）。清空经 set_inner_html('')，
        // 追加复用 _appendVariadic；MO childList 同时上报 removedNodes（旧子快照）+ addedNodes（新子）。
        if (prop === 'replaceChildren') {
          return function() {
            var removed = _childNodeList(sel, handle);
            if (handle && typeof __zw_set_inner_html_handle === 'function') __zw_set_inner_html_handle(handle, '');
            else if (typeof __zw_set_inner_html === 'function') __zw_set_inner_html(sel, '');
            var added = _appendVariadic(sel, handle, arguments);
            if (removed.length > 0 || added.length > 0) {
              _mo_notify(sel, handle, { type: 'childList', addedNodes: added, removedNodes: removed });
            }
            // R2994：旧子断连、新子按父连接态连入（旧子多为 sel-based parsed → 未追踪 → no-op；handle 子 best-effort）。
            for (var ri = 0; ri < removed.length; ri++) _ceApplyConn(removed[ri], false);
            var rcPconn = _ceParentConnected(sel, handle);
            for (var ai = 0; ai < added.length; ai++) _ceApplyConn(added[ai], rcPconn);
            return undefined;
          };
        }
        // `element.prepend(...nodesOrStrings)`（现代 API，区别于 appendChild/append）：插为元素
        // **首子**（保持参数序）。经 insertAdjacent afterbegin + 反序（见 _insertAdjacentVariadic）。
        // 仅 sel-based 目标；handle-only detached 无操作。
        if (prop === 'prepend') {
          return function() {
            _insertAdjacentVariadic(sel, 'afterbegin', arguments, true);
            return undefined;
          };
        }
        // `element.before(...nodesOrStrings)`：插为元素**前兄弟**（保持参数序）。beforebegin 正序。
        if (prop === 'before') {
          return function() {
            _insertAdjacentVariadic(sel, 'beforebegin', arguments, false);
            return undefined;
          };
        }
        // `element.after(...nodesOrStrings)`：插为元素**后兄弟**（保持参数序）。afterend 反序。
        if (prop === 'after') {
          return function() {
            _insertAdjacentVariadic(sel, 'afterend', arguments, true);
            return undefined;
          };
        }
        // `element.insertAdjacentHTML(position, text)`（P1a）：解析 HTML 片段并按 position 插入——
        // `beforeend`（末子）/`afterbegin`（首子）/`beforebegin`（前兄弟）/`afterend`（后兄弟）。
        // 服务端原子完成（fragment parse + copy + parent 遍历，见 DomMutation::InsertAdjacentHtml）。
        // 仅 sel-based（已挂载）元素经 host `__zw_insert_adjacent_html`；handle-only（createElement
        // detached）无 sel → 无操作（beforeend/afterbegin 因脱离文档树无意义，beforebegin/afterend 需
        // parent——spec 对 detached 元素本就抛错，此处静默无操作更安全）。
        if (prop === 'insertAdjacentHTML') {
          return function(position, text) {
            if (sel && typeof __zw_insert_adjacent_html === 'function') {
              try {
                // R3031：addedNodes 经 [`_zwFragmentAdded`] 回填解析片段的顶层节点（target=元素 sel 为
                // pragmatic 近似——beforebegin/afterend 实际影响父节点 childList，父 selector 此处不可得）。
                var _iahAdded = _zwFragmentAdded(text);
                __zw_insert_adjacent_html(sel, String(position), String(text));
                _mo_notify(sel, handle, { type: 'childList', addedNodes: _iahAdded, removedNodes: [] });
              } catch (_e) {}
            }
            return undefined;
          };
        }
        // `element.insertAdjacentText(position, text)`（P1a）：文本作**字面 Text 节点**（不解析
        // HTML）按 position 插入——区别于 insertAdjacentHTML（解析片段）。仅 sel-based（已挂载）
        // 元素经 host `__zw_insert_adjacent_text`；handle-only detached 无操作（同 insertAdjacentHTML）。
        if (prop === 'insertAdjacentText') {
          return function(position, text) {
            if (sel && typeof __zw_insert_adjacent_text === 'function') {
              try {
                __zw_insert_adjacent_text(sel, String(position), String(text));
                _mo_notify(sel, handle, { type: 'childList', addedNodes: [], removedNodes: [] });
              } catch (_e) {}
            }
            return undefined;
          };
        }
        // `element.insertAdjacentElement(position, element)`（P1a）：既有节点按 position 移动插入。
        // 仅接受 create 句柄节点（element.__zwHandle）；sel-based 参考元素经 host
        // `__zw_insert_adjacent_element`，复用 append_child 自动 reparent 移动语义。
        // 返插入的元素（spec）；handle-only 目标或非节点参数 → null（spec 非法 element 抛 TypeError，
        // 此处宽容返 null 避免中断脚本）。
        if (prop === 'insertAdjacentElement') {
          return function(position, element) {
            if (
              sel &&
              element &&
              typeof __zw_insert_adjacent_element === 'function' &&
              element.__zwHandle
            ) {
              try {
                __zw_insert_adjacent_element(sel, String(position), element.__zwHandle);
                _mo_notify(sel, handle, { type: 'childList', addedNodes: [element], removedNodes: [] });
                return element;
              } catch (_e) {}
            }
            return null;
          };
        }
        if (prop === 'querySelector') {
          // 元素**子树**作用域（spec：仅后代，不含元素自身）。
          // ① sel-based（挂载 DOM）→ host `__zw_query_match_sub(sel, q)` 子树首匹配。
          // ② handle-based（createElement / shadow root / fragment，无 sel）→ R2928 JS 端 registry 树搜索
          //   （host 不持可查询 handle 子树；registry 记 handle 父→子，DFS + 客户端选择器匹配）。querySelector
          //   不穿透 shadow 边界：host.querySelector 查 light-DOM 子树，host.shadowRoot.querySelector 查 shadow 树。
          return function(q) {
            if (sel && typeof __zw_query_match_sub === 'function') {
              try { var hit = __zw_query_match_sub(sel, String(q)); if (hit) return _wrapSelector(hit); } catch (_e) {}
              return null;
            }
            if (handle) return _handleQueryFirst(handle, q);
            return null;
          };
        }
        if (prop === 'querySelectorAll') {
          // 元素**子树**作用域（spec：仅后代）。同 querySelector：sel-based → host；handle-based → R2928 registry。
          // R3033：返 NodeList（item），包 _zwMakeCollection(arr, false)。
          return function(q) {
            if (sel && typeof __zw_query_all_sub === 'function') {
              try {
                var all = __zw_query_all_sub(sel, String(q));
                if (all) return _zwMakeCollection(all.split('|').filter(Boolean).map(_wrapSelector), false);
              } catch (_e) {}
              return _zwMakeCollection([], false);
            }
            if (handle) return _zwMakeCollection(_handleQueryAll(handle, q), false);
            return _zwMakeCollection([], false);
          };
        }
        // `el.getElementsByTagName(tag)` / `el.getElementsByClassName(cls)`（R2980）——元素**子树**作用域
        // 的标签/类名集合查询（spec 返 live HTMLCollection，headless 近似为静态 array-like，同 querySelectorAll
        // 模型）。现代代码 `table.getElementsByTagName('td')` / `form.getElementsByTagName('input')` /
        // `wrap.getElementsByClassName('item active')` 高频。委托子树 querySelectorAll：tag→直接选择器；
        // `'*'`→全后代；className 空格分隔多类→`.a.b` 全交集（spec 须同时含所有类）。sel-based → host
        // `__zw_query_all_sub`；`'*'` host 不支持 → 客户端 `_descendantElements` 递归下降；handle-based
        //（createElement/shadow/fragment，无 sel）→ R2928 JS 端 registry 子树搜索（原生支持 `*`）。
        // R3033：返 HTMLCollection（item + namedItem），包 _zwMakeCollection(arr, true)。
        if (prop === 'getElementsByTagName' || prop === 'getElementsByClassName') {
          return function(arg) {
            var q;
            if (prop === 'getElementsByTagName') {
              q = String(arg);
              // spec：空 tagName → 空集合。
              if (q === '') return _zwMakeCollection([], true);
              // host `__zw_query_all_sub` 不支持通用选择器 `*` → 客户端递归下降收全后代。
              if (q === '*') {
                if (sel) return _zwMakeCollection(_descendantElements(sel), true);
                if (handle) return _zwMakeCollection(_handleQueryAll(handle, '*'), true);
                return _zwMakeCollection([], true);
              }
            } else {
              // getElementsByClassName：空白分隔多类名 → 须同时含全部 → '.a.b'。
              var parts = String(arg).trim().split(/\s+/).filter(Boolean);
              if (parts.length === 0) return _zwMakeCollection([], true);
              q = '.' + parts.join('.');
            }
            if (sel && typeof __zw_query_all_sub === 'function') {
              try {
                var all = __zw_query_all_sub(sel, q);
                if (all) return _zwMakeCollection(all.split('|').filter(Boolean).map(_wrapSelector), true);
              } catch (_e) {}
              return _zwMakeCollection([], true);
            }
            if (handle) return _zwMakeCollection(_handleQueryAll(handle, q), true);
            return _zwMakeCollection([], true);
          };
        }
        // `el.getElementsByTagNameNS(ns, localName)`（spec `dom-element-getelementsbytagnamens`，R12）——
        // 命名空间作用域的标签集合查询。polyfill 无 ns 概念（HTML 单 ns），忽略 ns，按 localName 查
        //（同 getElementsByTagName 的 tag 逻辑）。case.html 用例 + 命名空间库高频。
        if (prop === 'getElementsByTagNameNS') {
          return function(_ns, localName) {
            var ln = String(localName == null ? '' : localName);
            if (ln === '') return _zwMakeCollection([], true);
            if (ln === '*') {
              if (sel) return _zwMakeCollection(_descendantElements(sel), true);
              if (handle) return _zwMakeCollection(_handleQueryAll(handle, '*'), true);
              return _zwMakeCollection([], true);
            }
            if (sel && typeof __zw_query_all_sub === 'function') {
              try {
                var all = __zw_query_all_sub(sel, ln);
                if (all) return _zwMakeCollection(all.split('|').filter(Boolean).map(_wrapSelector), true);
              } catch (_e) {}
              return _zwMakeCollection([], true);
            }
            if (handle) return _zwMakeCollection(_handleQueryAll(handle, ln), true);
            return _zwMakeCollection([], true);
          };
        }
        // `form.elements`（HTMLFormControlsCollection，R2829）——表单控件集合（jQuery serialize /
        // FormData / 校验库迭代高频）。仅 HTMLFormElement（_realTag==='FORM' gate）；非 form → undefined。
        // `_formControls(sel)` 查 '*' 全后代客户端按 tag 过滤（tree order）+ namedItem。
        if (prop === 'elements' && _realTag(sel, handle) === 'FORM') {
          var controls = _formControls(sel);
          // array-like collection + namedItem（id 或 name 首匹配）。
          controls.namedItem = function (name) {
            var n = String(name);
            for (var i = 0; i < controls.length; i++) {
              var c = controls[i];
              if (c && c.id === n) return c;
              try { if (c && c.getAttribute && c.getAttribute('name') === n) return c; } catch (_e2) {}
            }
            return null;
          };
          return controls;
        }
        // `form.length`（HTMLFormElement）= 控件数；非 form 透传（不拦截）。
        if (prop === 'length' && _realTag(sel, handle) === 'FORM') {
          return _formControls(sel).length;
        }
        // R3048：HTMLFormElement 方法——reset/requestSubmit/submit。旧缺（get trap 未拦 → `form.reset()` 抛
        // not-a-function 中断脚本）。reset：dispatch cancelable 'reset' 事件，未 preventDefault 则把控件恢复
        // defaultValue/defaultChecked/defaultSelected（经既有 setter，revert 表单状态）。requestSubmit：dispatch
        // submit SubmitEvent（cancelable，含 submitter）；submit：spec 不发事件直接导航，headless 无导航 → no-op
        //（防抛错，documented）。仅 FORM gate；非 form 透传 undefined。
        if (_realTag(sel, handle) === 'FORM' && (prop === 'reset' || prop === 'requestSubmit' || prop === 'submit')) {
          if (prop === 'reset') {
            return function () {
              var _rev = new Event('reset', { bubbles: true, cancelable: true });
              if (_dispatchWithBubble(key, sel, handle, _rev) === false) return; // preventDefault → 不重置
              if (sel) {
                var _fcs = _formControls(sel);
                for (var i = 0; i < _fcs.length; i++) {
                  var c = _fcs[i];
                  try {
                    var _ct = c.tagName;
                    // TEXTAREA/OUTPUT 有 defaultValue（OUTPUT R2846 _outputDefault / TEXTAREA R3049 _textareaDefault）。
                    if (_ct === 'TEXTAREA' || _ct === 'OUTPUT') { c.value = c.defaultValue; }
                    else if (_ct === 'INPUT') {
                      var _it = c.type;
                      if (_it === 'checkbox' || _it === 'radio') c.checked = c.defaultChecked;
                      else if (_it !== 'submit' && _it !== 'reset' && _it !== 'button' && _it !== 'image' && _it !== 'file')
                        c.value = c.defaultValue;
                    } else if (_ct === 'SELECT') {
                      var _opts = c.options;
                      for (var j = 0; _opts && j < _opts.length; j++) _opts[j].selected = _opts[j].defaultSelected;
                    }
                    if (typeof __zw_clear_user_edited === 'function') __zw_clear_user_edited(c);
                  } catch (_e) {}
                }
              }
            };
          } else if (prop === 'requestSubmit') {
            return function (submitter) {
              // dispatch submit SubmitEvent（cancelable，含 submitter）；headless 无导航（documented）。
              var _sev;
              try { _sev = new SubmitEvent('submit', { bubbles: true, cancelable: true, submitter: submitter || null }); }
              catch (_e) { _sev = new Event('submit', { bubbles: true, cancelable: true }); }
              _dispatchWithBubble(key, sel, handle, _sev);
            };
          } else { // submit
            return function () {}; // spec form.submit() 不发事件直接导航；headless 无导航 → no-op（防抛错）
          }
        }
        // 布局测量 API：`el.getBoundingClientRect()` 返真实 DOMRect（P1a gBCR path C）。
        // selector-identity 元素（querySelector/getElementById，sel=stable_selector）→ host
        // `__zw_getBoundingClientRect(sel)` 解析 dom_html→NodeId→layout-rect snapshot 返 "x,y,w,h"。
        // host 未注册 / 未命中 / handle-identity（createElement，sel 为空）→ 零 rect（= 旧行为，零回归；
        // 作 reflow 触发器语义仍正确——返回值多被丢弃）。注：rect 反映「上次 render」（stale-but-non-zero），
        // 改样式后同脚本内即读见 pre-change rect（force-reflow-on-demand 为 follow-up）。
        // offsetWidth/offsetHeight/clientWidth/Top/Left 等布局几何属性从同一 rect 派生（见 get trap 末段）。
        if (prop === 'getBoundingClientRect') {
          return function() {
            // identity = selector（querySelector/getElementById 元素）或 handle（createElement
            // 元素，path A）。sel 空时用 handle，host RectBridge handler 查持久 handle→selector map
            // 解析；map 未命中/未注册 → 空串 → 零 rect（= 旧行为，零回归）。
            return _domRectFromId(sel || handle) || _makeDomRect(0, 0, 0, 0);
          };
        }
        // `el.getClientRects()`（R2828）——DOMRectList（浮层定位库 popper.js/tether 取 [0] 测量）。
        // headless 无逐 line-box 布局 → 返**单元素 bounding rect** 数组（与 getBoundingClientRect 同源 _domRectFromId）；
        // inline 多行收缩为单 rect（无 per-line-box，documented）；handle-only detached 无 layout → []。
        if (prop === 'getClientRects') {
          return function() {
            var r = _domRectFromId(sel || handle);
            return r ? [r] : [];
          };
        }
        // 布局几何属性：offsetWidth/offsetHeight/clientWidth/clientHeight/offsetTop/offsetLeft。
        // 旧返 undefined → `el.offsetWidth > 0` visibility 检查误判 false（元素被当隐藏）。
        // 现从既有 __zw_getBoundingClientRect rect 派生（rect 反映上次 render，stale-but-non-zero
        // 同 gBCR）。无 rect（未渲染/handle 未映射）→ 0（detached 元素 offsetWidth=0 语义）。
        // 注：offsetWidth/Height 为 border-box（rect 即 border-box，精确）；clientWidth/Height 应为
        // content-box（缺 border 数据，此处≈offset，近似）；offsetTop/Left 应相对 offsetParent（此处
        // 相对 viewport，顶层元素精确、嵌套近似）——近似对 visibility/sizing 检查足够。
        if (prop === 'offsetWidth' || prop === 'clientWidth') {
          var r = _layoutRect(sel, handle);
          return r ? r.w : 0;
        }
        if (prop === 'offsetHeight' || prop === 'clientHeight') {
          var r = _layoutRect(sel, handle);
          return r ? r.h : 0;
        }
        if (prop === 'offsetTop') {
          var r = _layoutRect(sel, handle);
          return r ? r.y : 0;
        }
        if (prop === 'offsetLeft') {
          var r = _layoutRect(sel, handle);
          return r ? r.x : 0;
        }
        // scrollWidth/scrollHeight：滚动内容尺寸。布局 rect 无 overflow 数据（不含滚动展开量），
        // 近似为 client 尺寸（同 offsetWidth/Height 的 border-box 近似）。对「content 是否溢出」
        // 精确判定不足，但对 `el.scrollHeight > 0` 等 sizing 检查足够（消除旧 undefined 返回）。
        if (prop === 'scrollWidth') {
          var r = _layoutRect(sel, handle);
          return r ? r.w : 0;
        }
        if (prop === 'scrollHeight') {
          var r = _layoutRect(sel, handle);
          return r ? r.h : 0;
        }
        // R3047：scrollTop/scrollLeft 读 `_scrollOffsets[key]`（程序化滚动 round-trip 自洽；默认未滚动 → 0）。
        if (prop === 'scrollTop' || prop === 'scrollLeft') {
          var _sg = _scrollOffsets[key];
          return _sg ? (prop === 'scrollTop' ? _sg.top : _sg.left) : 0;
        }
        // offsetParent：最近 positioned 祖先（position != static）或 body，detached/hidden → null。
        // 布局 rect 无 style 信息，无法精确算 positioned 祖先；近似：有 rect（已渲染）→ body proxy，
        // 无 rect（detached/display:none）→ null。dominant 用法 `el.offsetParent === null` 可见性判定
        // 正确（visible→非 null body / hidden→null）；`offsetTop - offsetParent.offsetTop` 嵌套坐标
        // 为近似（offsetTop 本就 viewport-relative，见上注）。
        if (prop === 'offsetParent') {
          var rp = _layoutRect(sel, handle);
          return rp ? _wrapSelector('body') : null;
        }
        // R3036：`element.sheet`——`<style>`/`<link rel=stylesheet>` 元素的 CSSStyleSheet（CSSOM 入口）。
        // CSS-in-JS 库（styled-components/emotion）+ 样式表操作代码经 `.sheet.cssRules`/`insertRule` 读改规则。
        // 复用既有 `_makeStyleSheet(owner)`（part06.js，ownerNode=元素 proxy，cssRules 惰性重解析 owner textContent）。
        // STYLE → sheet；LINK 且 rel~stylesheet → sheet（非 stylesheet → null）；其他元素 → fall through undefined
        //（spec：.sheet 仅 HTMLStyleElement/HTMLLinkElement 有）。仅 sel-based（live DOM 有 selector）；handle-only
        //（createElement detached）无 sel → null（ownerNode 不可定位，detached <style> 操作罕见）。
        if (prop === 'sheet') {
          var _shTag = _realTag(sel, handle);
          if (_shTag === 'STYLE' || _shTag === 'LINK') {
            if (_shTag === 'LINK') {
              var _lrel = (handle ? __zw_get_attr_handle(handle, 'rel') : (sel ? __zw_get_attr(sel, 'rel') : '')) || '';
              if (!/\bstylesheet\b/i.test(_lrel)) return null; // link 非 stylesheet → null
            }
            return sel ? _makeStyleSheet(_wrapSelector(sel)) : null;
          }
          // 非 style/link：fall through undefined（generic Element 无 .sheet）
        }
        // R3189：`input.type` / `button.type` enumerated reflection（spec「limited to only known values」），
        // 须先于通用 type 字符串反射（R3037）——INPUT/BUTTON 的 type 为枚举（缺省/非法→default，关键字→规范小写），
        // 非 INPUT/BUTTON（link/script/style/embed 等）回落通用字符串反射。经 [`_reflectedTypeEnum`]（part01.js）。
        if (prop === 'type') {
          // https://html.spec.whatwg.org/multipage/form-elements.html#dom-output-type
          if (_realTag(sel, handle) === 'OUTPUT') return 'output';
          var _et = _reflectedTypeEnum(sel, handle);
          if (_et !== null) return _et;
        }
        // R3037：reflected string 内容属性读（type/name/placeholder/min/max/step/pattern/alt/src/rel/...）。
        // 旧 get trap 未拦 → 读返 undefined（写正常，set trap generic fallthrough → __zw_set_attr）。表单校验库
        // 读 input.min/max/pattern/type、analytics 读 src/name 等失效。命中 [`_reflectedStringAttr`] → 读内容属性
        //（sel 走 latest-wins `__zw_get_attr_lw` 反映 pending SetAttr；handle 走 `__zw_get_attr_handle`）；缺省返 ''
        //（spec reflected string 缺省空串，非 null/undefined）。
        var _rsAttr = _reflectedStringAttr(prop);
        if (_rsAttr) {
          if (handle) return __zw_get_attr_handle(handle, _rsAttr) || '';
          if (typeof __zw_get_attr_lw === 'function') return __zw_get_attr_lw(sel, _rsAttr) || '';
          return __zw_get_attr(sel, _rsAttr) || '';
        }
        // R3043：`.size` element-aware reflected 数值读（HTMLInputElement default 20 / HTMLSelectElement default 0）。
        // `_REFLECTED_UINT` 表按 IDL 名 keyed 无 element-awareness，`size` 两元素 default 不同（input 20 / select 0）
        // 故专用分支 tag-gate。input：parseInt 内容属性，缺省/不可解析/<1 → 20（spec default，浏览器实测 clamp）；
        // select：缺省/不可解析/<0 → 0（spec default；size=0 表「UA 默认显示行数」）。非 INPUT/SELECT fall through undefined
        //（real browser 无 .size）。set 走 generic fallthrough 写 size 属性（同 cols/rows）。
        if (prop === 'size') {
          var _szTag = _realTag(sel, handle);
          if (_szTag === 'INPUT' || _szTag === 'SELECT') {
            var _szRaw = handle
              ? __zw_get_attr_handle(handle, 'size')
              : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, 'size') : __zw_get_attr(sel, 'size'));
            var _szN = parseInt(String(_szRaw == null ? '' : _szRaw), 10);
            if (_szTag === 'INPUT') return (isNaN(_szN) || _szN < 1) ? 20 : _szN;
            return (isNaN(_szN) || _szN < 0) ? 0 : _szN;
          }
        }
        // R3038/R3041：reflected unsigned-long（numeric）属性读（colSpan/rowSpan/maxLength/minLength/cols/rows/start）。
        // parseInt 内容属性 → number；缺省/不可解析 → entry.d（spec default）；colSpan/rowSpan <1 → 1（min）。
        // 注：TABLE/THEAD/TBODY/TFOOT 的 `.rows` 在更早分支（part03）返行集合——此处仅对 textarea 命中（table 已 return）。
        var _ruEntry = _REFLECTED_UINT[prop];
        if (_ruEntry) {
          var _ruRaw = handle
            ? __zw_get_attr_handle(handle, _ruEntry.a)
            : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, _ruEntry.a) : __zw_get_attr(sel, _ruEntry.a));
          var _ruN = parseInt(String(_ruRaw == null ? '' : _ruRaw), 10);
          if (isNaN(_ruN)) return _ruEntry.d;
          if (_ruEntry.min != null && _ruN < _ruEntry.min) return _ruEntry.min;
          return _ruN;
        }
        // R3038/R3040：reflected boolean 属性读（_REFLECTED_BOOL 全表：required/readOnly/multiple/noValidate/
        // async/defer/nomodule/autoplay/controls/loop/muted/playsInline/reversed/isMap/itemScope）——presence-based。
        var _rbAttr = Object.prototype.hasOwnProperty.call(_REFLECTED_BOOL, prop) ? _REFLECTED_BOOL[prop] : null;
        if (_rbAttr) {
          var _rbHit = handle
            ? __zw_has_attr_handle(handle, _rbAttr)
            : (typeof __zw_has_attr_lw === 'function' ? __zw_has_attr_lw(sel, _rbAttr) : __zw_has_attr(sel, _rbAttr));
          return _rbHit === '1';
        }
        // R3042：expando 属性读（非原始值 setter 存于 per-element _expando map）。命中 → 返存储值（function/object
        // 等，real browser expando 语义）。仅 hasOwnProperty 命中才返（避免原型链污染）；未命中 fall through undefined。
        var _exStore = _expando[key];
        if (_exStore && Object.prototype.hasOwnProperty.call(_exStore, prop)) return _exStore[prop];
        return undefined;
      },
      set: function(_t, prop, value) {
        var p = String(prop);
        var moAttr = null;
        // R3034：text/comment 节点 `.data`/`.nodeValue` IDL setter（CharacterData）。须先于末尾 generic fallthrough
        //（part05.js：'data' 落入 else 被误当内容属性 → `__zw_set_attr_handle(handle,'data')` + attributes MO 记录，
        // 类型错且文本内容未持久化——读回经 `__zw_get_text_handle` 返旧值，setter 失效）。handle-based 文本/注释
        // 节点 data/nodeValue= 经 `__zw_set_text_handle` 持久化 + emit characterData（闭合 R3027/R3028 已知限制：
        // handle-based 文本节点 characterData emission 未接）。target=文本节点 handle；ancestor subtree 冒泡需 sel
        // 父链，handle-based 无 sel → 不冒泡（同 R3026 detached 限制，documented）。
        var _isText = handle && _textHandles[handle];
        var _isComment = handle && _commentHandles[handle];
        if ((_isText || _isComment) && (p === 'data' || p === 'nodeValue')) {
          if (handle) {
            var _tdMoId = _mo_id(handle, sel);
            var _tdMoOld = _mo_any_wants_char_old(_tdMoId) ? _mo_read_text(sel, handle) : null;
            __zw_set_text_handle(handle, String(value == null ? '' : value));
            _mo_notify(sel, handle, { type: 'characterData', oldValue: _tdMoOld });
          }
          return true;
        }
        // R2933 element 级 IDL on-event handler（onclick/oninput/onkeydown/onchange/...）。须先于末尾 fallthrough
        //（否则 fn 被当字符串属性写入 onclick="function..."）。`on`+小写字母 = handler（generic，覆盖 onclick/
        // oninput/onload/onsubmit/onpointer* 等所有当前/未来事件，无白名单）。setter 路由到 per-element listener
        // store（同 addEventListener 的 _listenerStore[key]）：移旧 fn + 加新 fn；非 function → 移除（spec IDL）。
        if (typeof prop === 'string' && /^on[a-z]/.test(p)) {
          var _ot = p.slice(2);
          var _prevH = _onHandlers[key] && _onHandlers[key][_ot];
          if (typeof _prevH === 'function' && _listenerStore[key] && _listenerStore[key][_ot]) {
            _listenerStore[key][_ot] = _listenerStore[key][_ot].filter(function (l) { return l.fn !== _prevH; });
          }
          if (typeof value === 'function') {
            if (!_onHandlers[key]) _onHandlers[key] = {};
            _onHandlers[key][_ot] = value;
            if (!_listenerStore[key]) _listenerStore[key] = {};
            if (!_listenerStore[key][_ot]) _listenerStore[key][_ot] = [];
            _listenerStore[key][_ot].push({ fn: value, capture: false, once: false });
          } else {
            if (_onHandlers[key]) _onHandlers[key][_ot] = null;
          }
          return true;
        }
        if (p === 'textContent' || p === 'innerText' || p === 'innerHTML') {
          if (p === 'innerHTML') {
            // R3029：innerHTML = 整体替换子树（childList 类）。emit childList 记录，闭合「innerHTML 不 emit
            // childList」gap（R3028 已知限制④）。removedNodes = 替换前旧子（snapshot 读，_childNodeList 对
            // handle-only 无 sel 返 []）。R3031：addedNodes 经 [`_zwFragmentAdded`] 回填——host fragment 二次
            // parse 建 _zwMEl 代理树，取 .childNodes（可读 nodeType/tagName/getAttribute/querySelector，满足
            // 框架 observe 后递归观测新子树）；host 未注册 `__zw_parse_html_child_nodes` → []（旧行为）。
            var _ihRemoved = _childNodeList(sel, handle);
            var _ihAdded = _zwFragmentAdded(value);
            // spec `LegacyNullToEmptyString`：null → 空串（清子），非写 "null" 文本；undefined 仍 ToString。
            var _ihVal = value === null ? '' : String(value);
            if (handle) __zw_set_inner_html_handle(handle, _ihVal);
            else __zw_set_inner_html(sel, _ihVal);
            _mo_notify(sel, handle, { type: 'childList', addedNodes: _ihAdded, removedNodes: _ihRemoved });
          } else {
            // R3027：textContent 变更 → emit characterData 记录（target=元素，pragmatic——文本节点无 selector
            // 不能直接作 target；observe(el,{characterData,subtree}) + 后代 textContent 经 ancestor 冒泡亦覆盖）。
            // R3028：characterDataOldValue——有 observer 请求时 mutate 前捕获 old 文本（latest-wins，反映同批前序 textContent=）。
            var _charMoId = _mo_id(handle, sel);
            var _charMoOld = _mo_any_wants_char_old(_charMoId) ? _mo_read_text(sel, handle) : null;
            // spec `LegacyNullToEmptyString`：null → 空串（清子）。
            var _tcVal = value === null ? '' : String(value);
            if (_realTag(sel, handle) === 'OUTPUT') {
              // https://html.spec.whatwg.org/multipage/form-elements.html#the-output-element
              // Replacing descendants updates defaultValue. In default mode value follows it;
              // in value mode `_outputValue` keeps the live value.
              _outputDefault[key] = _tcVal;
            }
            if (handle) __zw_set_text_handle(handle, _tcVal);
            else __zw_set_text(sel, _tcVal);
            _mo_notify(sel, handle, { type: 'characterData', oldValue: _charMoOld });
          }
        } else if (p === 'outerHTML') {
          // outerHTML setter：整体替换元素为解析后的片段。仅 sel-based（需父节点）；
          // handle-only（detached）无父 → 无操作（spec 对无父元素赋 outerHTML 抛错，静默更安全）。
          // R3031：addedNodes 经 [`_zwFragmentAdded`] 回填解析片段的顶层节点（target=元素 sel 为 pragmatic
          // 近似，spec target=父节点——父 selector 此处不可得，承自 R3029 既有近似）。
          if (sel && typeof __zw_set_outer_html === 'function') {
            try {
              var _ohAdded = _zwFragmentAdded(value);
              // spec `LegacyNullToEmptyString`：null → 空串（移除自身），非替换为 "null" 文本。
              __zw_set_outer_html(sel, value === null ? '' : String(value));
              _mo_notify(sel, handle, { type: 'childList', addedNodes: _ohAdded, removedNodes: [] });
            } catch (_e) {}
          }
          return true;
        } else if (p === 'classList') {
          // R19：`classList` 是 readonly accessor 属性（无 setter，spec `dom-element-classlist`）。赋值
          // `el.classList = x` 应 no-op——non-strict 静默忽略（WPT assignToClassList 期望 classList 不变）、
          // strict 抛 TypeError（assignToClassListStrict）。真实浏览器 strict 抛 TypeError。本沙箱 strict 检测
          // 复杂，按 non-strict 语义 no-op（return true 保持 classList 原样）——与 WPT 两个用例一致。
          // 须早于末尾 generic fallthrough（否则 classList 落入 expando 被覆盖）。
          return true;
        } else if (p === 'className') {
          _classCache[key] = String(value);
          if (handle) __zw_set_attr_handle(handle, 'class', String(value));
          else __zw_set_attr(sel, 'class', String(value));
          moAttr = 'class';
        } else if (p === 'id') {
          // spec [LegacyNullToEmptyString]：null → 空串（非 "null"）。
          var idv = value === null ? '' : String(value);
          if (handle) __zw_set_attr_handle(handle, 'id', idv);
          else __zw_set_attr(sel, 'id', idv);
          moAttr = 'id';
        } else if (p === 'title' || p === 'lang' || p === 'dir') {
          // reflected 字符串属性 set——写同名 attribute + 同步客户端缓存（set 后 get 读缓存）。
          // spec [LegacyNullToEmptyString]：title/lang null→空串；dir 为 enumerated 非 LegacyNull（null→"null"）。
          var rcb = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
          var rcv = (p !== 'dir' && value === null) ? '' : String(value);
          rcb[p] = rcv;
          if (handle) __zw_set_attr_handle(handle, p, rcv);
          else __zw_set_attr(sel, p, rcv);
          moAttr = p;
        } else if (p === 'tabIndex') {
          // tabIndex set——反射为 tabindex 属性（数值）；NaN 忽略（spec 抛，lenient 不抛）。同步缓存。
          var tisv = parseInt(value, 10);
          if (!isNaN(tisv)) {
            var rtc2 = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
            rtc2['tabindex'] = tisv;
            if (handle) __zw_set_attr_handle(handle, 'tabindex', String(tisv));
            else __zw_set_attr(sel, 'tabindex', String(tisv));
            moAttr = 'tabindex';
          }
        } else if (p === 'contentEditable') {
          // contentEditable set——反射 contenteditable 属性（lenient：spec 仅接受 true/false/plaintext-only
          // 否则抛 SyntaxError，本沙箱不抛直接设串避免中断脚本）。同步缓存。
          var cec2 = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
          cec2['contenteditable'] = String(value);
          if (handle) __zw_set_attr_handle(handle, 'contenteditable', String(value));
          else __zw_set_attr(sel, 'contenteditable', String(value));
          moAttr = 'contenteditable';
        } else if (p === 'accessKey') {
          // accessKey set——反射 accesskey 属性（串）。spec [LegacyNullToEmptyString]：null→空串。同步缓存。
          var akc2 = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
          var akv = value === null ? '' : String(value);
          akc2['accesskey'] = akv;
          if (handle) __zw_set_attr_handle(handle, 'accesskey', akv);
          else __zw_set_attr(sel, 'accesskey', akv);
          moAttr = 'accesskey';
        } else if (p === 'open') {
          // R3290：HTMLDialogElement.open / HTMLDetailsElement.open ——boolean 反射 setter。
          // spec：truthy → setAttribute('open', '')（presence）；falsy → removeAttribute('open')。
          // 模态 dialog 经 open setter 关闭（falsy）不派 close 事件（spec：close 事件仅 close() 派发，
          // 直接 removeAttribute 不派——real browser 一致）。getter 直读属性（无缓存 → 无 stale）。
          if (value) _zwSetAttr(key, sel, handle, 'open', '');
          else {
            _zwRemoveAttr(key, sel, handle, 'open');
            if (_realTag(sel, handle) === 'DIALOG') {
              delete _zwDialogModal[key];
              delete _zwTopLayer[key];
            }
          }
        } else if (p === 'returnValue' && _realTag(sel, handle) === 'DIALOG') {
          // R3290：HTMLDialogElement.returnValue IDL setter。spec：存任意值为串（不反射内容属性——dialog 无
          // returnValue 内容属性）。lenient 接受任意值（null → ''，与 getter 默认值一致）。
          _expando[key + '::returnValue'] = (value == null) ? '' : String(value);
        } else if (p === 'popover') {
          // R3071：popover enumerated setter。spec：null → removeAttribute（清 popover 元素身份）；余 → setAttribute
          //（getter 经 `_zwReadPopover` 映射 invalid→manual，real browser 一致）。不写 _reflectedAttrs 缓存
          //（getter 直读属性，无 sync set→get stale gap 风险——popover 读经属性而非缓存，与 title/lang/dir 不同）。
          if (value === null) {
            delete _zwTopLayer[key];
            if (handle && typeof __zw_remove_attr_handle === 'function') __zw_remove_attr_handle(handle, 'popover');
            else if (!handle && typeof __zw_remove_attr === 'function') { __zw_remove_attr(sel, 'popover'); moAttr = 'popover'; }
          } else {
            if (handle) __zw_set_attr_handle(handle, 'popover', String(value));
            else { __zw_set_attr(sel, 'popover', String(value)); moAttr = 'popover'; }
          }
        } else if (p === 'popoverTargetElement') {
          // R3073：编程式目标元素。spec setter：Element → 存（优先于内容属性）；null → 清除（回落内容属性）。
          // 非 Element 应抛 TypeError（spec），lenient 接受任意值（headless 简化，activation 调其方法）。不改内容属性。
          if (value === null) delete _popoverTargetEl[key];
          else _popoverTargetEl[key] = value;
        } else if (p === 'popoverTargetAction') {
          // R3073：reflected setter。spec：写 popovertargetaction 内容属性（raw 值，getter 映射 invalid→toggle）。
          // lenient：null/缺省 → 'toggle'。
          var _ptaV = (value == null) ? 'toggle' : String(value);
          if (handle) __zw_set_attr_handle(handle, 'popovertargetaction', _ptaV);
          else { __zw_set_attr(sel, 'popovertargetaction', _ptaV); moAttr = 'popovertargetaction'; }
        } else if (p === 'role') {
          // role set——反射 role 属性（串）。同步缓存。
          var rlc2 = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
          rlc2['role'] = String(value);
          if (handle) __zw_set_attr_handle(handle, 'role', String(value));
          else __zw_set_attr(sel, 'role', String(value));
          moAttr = 'role';
        } else if (_ariaAttrName(p)) {
          // ariaXxx set——反射 aria-* 属性（ariaLabel→aria-label, ariaLabelledBy→aria-labelledby...）。
          // 通用映射覆盖全部 aria IDL 属性。同步缓存。
          var ariaAttr = _ariaAttrName(p);
          var arc2 = _reflectedAttrs[key] || (_reflectedAttrs[key] = {});
          arc2[ariaAttr] = String(value);
          if (handle) __zw_set_attr_handle(handle, ariaAttr, String(value));
          else __zw_set_attr(sel, ariaAttr, String(value));
          moAttr = ariaAttr;
        } else if (p === 'value') {
          // P1a select：编程设 `<select>.value = value` → 记 SelectOption mutation（apply 时
          // mark 匹配 option selected + deselect 兄弟）。匹配浏览器：编程设值不自动派 change。
          if (!handle && sel && typeof __zw_select_option === 'function' && _isTag(sel, 'SELECT')) {
            __zw_select_option(sel, String(value));
            // SelectOption 改的是子 option 的 selected 属性，非 select 元素自身的属性 mutation；
            // 不发 select 的 attributes MO 通知（语义正确）。
          } else if (_realTag(sel, handle) === 'OUTPUT') {
            // https://html.spec.whatwg.org/multipage/form-elements.html#dom-output-value
            // Capture default mode before entering value mode, then replace descendants.
            if (_outputDefault[key] == null) {
              _outputDefault[key] = handle ? (__zw_get_text_handle(handle) || '') : (__zw_get_text(sel) || '');
            }
            var _ov = String(value);
            _outputValue[key] = _ov;
            if (handle) __zw_set_text_handle(handle, _ov);
            else __zw_set_text(sel, _ov);
          } else {
            _inputValues[key] = String(value);
            // input/textarea 的 IDL value 是 retained 当前值，不改 HTML 内容属性/textarea 默认文本。
            // https://html.spec.whatwg.org/multipage/input.html#dom-input-value
            // R2996/R3049：首次写前仍捕获 defaultValue，供 getter + form.reset。
            if (!handle && sel && _isTag(sel, 'TEXTAREA')) {
              if (_textareaDefault[key] == null) _textareaDefault[key] = __zw_get_text(sel) || '';
              __zw_set_form_value(sel, String(value));
            } else {
              if (_realTag(sel, handle) === 'INPUT') _captureInputDefault(key, sel, handle);
              if (handle) {
                __zw_set_attr_handle(handle, 'value', String(value));
              } else {
                __zw_set_form_value(sel, String(value));
              }
            }
          }
        } else if (p === 'valueAsNumber') {
          // `input.valueAsNumber = n`（HTMLInputElement，R2836）——number/range：NaN→''，否则 String(n)→设
          // value 属性 + 缓存（复用 value 同步路径）。其他 type / 非 INPUT：no-op（date/time defer；分支
          // 终止不 fallthrough 致误设 'valueAsNumber' 内容属性）。仅 INPUT（_realTag gate）。
          if (_realTag(sel, handle) === 'INPUT') {
            var vsT = (handle ? __zw_get_attr_handle(handle, 'type') : __zw_get_attr(sel, 'type')) || '';
            if (vsT.toLowerCase() === 'number' || vsT.toLowerCase() === 'range') {
              var vsS = (typeof value === 'number' && isNaN(value)) ? '' : String(value);
              _inputValues[key] = vsS;
              _captureInputDefault(key, sel, handle); // R2996：valueAsNumber= 等同 .value=，捕获 defaultValue
              if (handle) __zw_set_attr_handle(handle, 'value', vsS);
              else __zw_set_form_value(sel, vsS);
            }
          }
        } else if (p === 'valueAsDate') {
          // `input.valueAsDate = date`（HTMLInputElement，R3317）——date/month/week/time 输入 Date→串。
          // spec：仅 date/month/week/time type 接受（其他 type setter no-op）；有效 Date→格式化串设 value；
          // 无效值（非 Date / Invalid Date）→抛 InvalidStateError（这里近似：非 Date 静默 no-op，Invalid Date 清空）。
          // 复用 value 同步路径（_inputValues + _captureInputDefault + attr/form-value 双路径）。仅 INPUT。
          if (_realTag(sel, handle) === 'INPUT') {
            var vadTs = (handle ? __zw_get_attr_handle(handle, 'type') : __zw_get_attr(sel, 'type')) || '';
            vadTs = vadTs.toLowerCase();
            if (vadTs === 'date' || vadTs === 'month' || vadTs === 'week' || vadTs === 'time') {
              var vadStr = _formatHtmlDateValue(value, vadTs);
              _inputValues[key] = vadStr;
              _captureInputDefault(key, sel, handle);
              if (handle) __zw_set_attr_handle(handle, 'value', vadStr);
              else __zw_set_form_value(sel, vadStr);
            }
          }
        } else if (p === 'indeterminate') {
          // JS-only IDL 布尔（非 reflected attr）—— per-element state map（默认 false）。无属性 mutation。
          _indeterminate[key] = !!value;
        } else if (p === 'selectionStart' || p === 'selectionEnd' || p === 'selectionDirection') {
          // text-control 选区 setter（R2844）。Chromium 150 oracle 锚定：保持 0≤start≤end≤len 不变式——
          // 设 start 超 end → end 跟到 start（{start:99}→ end 升到 start）；设 end 低于 start → end 升回 start
          //（{end:-5}→ end 升到 start，不降）；start/end 均 clamp [0, len]；direction 仅接受 forward/backward/none。
          if (_isTextControl(sel, handle)) {
            var so = _selObj(key);
            if (p === 'selectionStart') {
              var nsLen = _controlValue(sel, handle, key).length;
              var ns2 = _clampSelOffset(value, nsLen);
              if (ns2 > so.end) so.end = ns2;
              so.start = ns2;
            } else if (p === 'selectionEnd') {
              var neLen = _controlValue(sel, handle, key).length;
              var ne2 = _clampSelOffset(value, neLen);
              if (ne2 < so.start) ne2 = so.start;
              so.end = ne2;
            } else {
              so.direction = (value === 'backward' || value === 'none') ? value : 'forward';
            }
          } else if (_realTag(sel, handle) === 'INPUT') {
            _throwDom('InvalidStateError', 'input type does not support text selection');
          }
        } else if (p === 'htmlFor') {
          // `label.htmlFor = x`（R2840）——反射 `for` 属性（attr 名映射 htmlFor→for）。仅 LABEL。
          if (_realTag(sel, handle) === 'LABEL') {
            if (handle) __zw_set_attr_handle(handle, 'for', String(value));
            else { __zw_set_attr(sel, 'for', String(value)); moAttr = 'for'; }
