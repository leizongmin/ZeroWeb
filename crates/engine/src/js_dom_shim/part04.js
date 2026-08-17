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
              // R34xx：缺参 → TypeError（WebIDL 必参——2d.canvas.context.invalid.args）。
              if (arguments.length === 0) throw new TypeError('getContext: missing contextType');
              if (String(type) !== '2d') return null; // 仅 2d；webgl/webgl2 defer
              if (_zwCanvasCtx[key]) return _zwCanvasCtx[key];
              if (typeof __zw_canvas_op !== 'function') return null;
              var cw = _zwCanvasDim(sel, handle, 'width', 300);
              var ch = _zwCanvasDim(sel, handle, 'height', 150);
              var id = __zw_canvas_op('0', 'getContext2d', String(cw), String(ch),
                (arguments.length > 1 && arguments[1] && typeof arguments[1] === 'object' && typeof arguments[1].colorSpace === 'string')
                  ? arguments[1].colorSpace : 'srgb',
                (arguments.length > 1 && arguments[1] && typeof arguments[1] === 'object' && typeof arguments[1].colorType === 'string')
                  ? arguments[1].colorType : 'unorm8');
              if (!id || String(id).charAt(0) === '!') return null;
              var ctx = _zwMakeCtx2d(String(id));
              // R34xx（color-type 目录）：记录 canvas 色彩空间（f16 浮点转换基准）。
              ctx._cs = (arguments.length > 1 && arguments[1] && typeof arguments[1] === 'object' && typeof arguments[1].colorSpace === 'string')
                ? arguments[1].colorSpace : 'srgb';
              // R34xx：ctx.canvas 与 getElementById 同 identity（_proxyCache 键 =
              // sel（非 handle）——getElementById/querySelector 走 sel-only 键；
              // 旧 (sel,handle) 键产第二 proxy → 2d.canvas.host.readonly 的
              // ctx.canvas === d 失败）+ 只读（spec readonly——ctx.canvas = x 忽略；
              // 2d.canvas.host.readonly 的赋值后仍 === d）。handle-only
              //（createElement 未挂载）仍 handle 键。
              Object.defineProperty(ctx, 'canvas', {
                value: sel ? _makeProxy(sel, null) : _makeProxy(null, handle),
                writable: false,
                enumerable: true,
                configurable: false
              });
              _zwCanvasCtx[key] = ctx;
              // R34xx：colorType 'float16' 上下文——绘制 float16 位图时记录原始浮点像素覆盖层
              //（createImageBitmap.srgb.rgba.float16 的越界值往返；DOM canvas 与 standalone
              // part05 _zwMakeCanvas 同语义——getImageData 回读按覆盖层优先）。
              if (arguments.length > 1 && arguments[1] && typeof arguments[1] === 'object' &&
                  String(arguments[1].colorType || '') === 'float16') {
                ctx._f16 = true;
                ctx._f16Overlay = null;
              }
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
              // R34xx（layers 目录）：层打开期间 toDataURL 抛 InvalidStateError。
              if (ctx && ctx._inLayer) {
                throw _zwDomException('toDataURL: not allowed while a layer is open', 'InvalidStateError');
              }
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
                // R34xx（layers 目录）：层打开期间 toBlob 抛 InvalidStateError。
                if (_zwCanvasCtx[key] && _zwCanvasCtx[key]._inLayer) {
                  throw _zwDomException('toBlob: not allowed while a layer is open', 'InvalidStateError');
                }
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
          // R81：PI 的 textContent = data（早于下方通用分支——通用 host 路径对 PI 读错误值）。
          if (handle && _piHandles[handle] && prop === 'textContent') {
            return _piHandles[handle].data;
          }
          // `innerText`（R3260）≈ textContent 近似——real innerText 是 layout/CSS-aware（排除 hidden 元素、
          // `<br>`/block →换行、white-space 处理）；headless 无 layout 透出到 JS，best-effort 返 textContent。
          // 高频读元素渲染文本（`el.innerText` 此前返 undefined）；setter 同 textContent（替换全部子为文本节点）。
          // R3028：sel-based 走 latest-wins（consult 变更列表，闭合 `textContent=` 后 getter stale 旧值）；
          // 回调未注册（polyfill/其它环境）→ fallback 纯快照 `__zw_get_text`。
          // js-dom M4 R81：textContent = 子树全部 Text 后代 data 拼接（spec dom-node-textcontent——
          // comment/PI 不计入，元素递归）。旧 handle 元素读 host 注册文本（pending 子不可见 → ''，WPT
          // Node-textContent "Element with children/descendants" 簇）；改 JS 侧沿**融合 childNodes 视图**
          // 递归拼接（pending 自然正确，host 回调仍作无子回落）。innerText best-effort 同源（近似语义维持）。
          if (handle) {
            var _tcKids = (function () {
              // R81：容器 handle（fragment/shadow）只读 registry（textContent= 的文本子已入
              // registry——本地注册 + registry 融合会双份）。
              if (_isContainerHandle(handle)) return _handleChildNodes(handle);
              if (!sel && handle) {
                var _tcL = (typeof _zwLocalChildNodes === 'function') ? _zwLocalChildNodes(sel, handle) : null;
                var _tcK = (_handleChildren[handle] || []).slice();
                if (_tcL && _tcL.length) return _tcL.concat(_tcK);
                if (_tcK.length) return _tcK;
                if (_tcL) return _tcL;
                return _childNodeList(sel, handle);
              }
              return _childNodeList(sel, handle);
            })();
            var _tcOut = '';
            for (var _tci = 0; _tci < _tcKids.length; _tci++) {
              var _tcn = _tcKids[_tci];
              if (!_tcn) continue;
              // R81：CDATA（nodeType 4）与 PI（7）的 textContent 也是 data（spec CharacterData +
              // ProcessingInstruction 的 textContent 语义；WPT Node-properties testDiv.textContent
              // 期望 CDATA 内容计入拼接——spec dom-node-textcontent 对 CDATA/PI 子节点取其 data）。
              if (_tcn.nodeType === 3 || _tcn.nodeType === 4) _tcOut += String(_tcn.nodeValue != null ? _tcn.nodeValue : (_tcn.data != null ? _tcn.data : ''));
              else if (_tcn.nodeType === 7) _tcOut += String(_tcn.data != null ? _tcn.data : '');
              else if (_tcn.nodeType === 1 && typeof _tcn.textContent === 'string') _tcOut += _tcn.textContent;
            }
            if (_tcOut !== '' || _tcKids.length > 0) return _tcOut;
            // R81：textContent= 的 JS 侧写入值优先于 host 变更重放（旧 AppendChild 文本残留）。
            if (typeof _zwTextWritten !== 'undefined' && _zwTextWritten && Object.prototype.hasOwnProperty.call(_zwTextWritten, handle)) {
              return _zwTextWritten[handle];
            }
            return __zw_get_text_handle(handle);
          }
          return typeof __zw_get_text_lw === 'function' ? __zw_get_text_lw(sel) : __zw_get_text(sel);
        }
        // js-dom M3 R95：`<template>`.content（HTMLTemplateElement，spec `the-template-element`：
        // 模板内容放 inert DocumentFragment）。lit-html 的 Template.createElement 路径：
        // `createElement('template'); t.innerHTML = html; return t`，随后 `t.content` 取解析
        // 子树 + TreeWalker 走 parts——content 缺失则 lit render 管线死寂。实现：惰性轻量
        // fragment 视图——childNodes 直读 `_handleChildren[handle]`（innerHTML= setter 已存
        // `_zwFragmentAdded` 解析树，与 R83 childNodes 融合视图同源）；firstChild/lastChild
        // 派生；nodeType=11。
        if (prop === 'content' && handle && _realTag(sel, handle) === 'TEMPLATE') {
          var _tplContent = {
            nodeType: 11,
            nodeName: '#document-fragment',
            get childNodes() { return _handleChildren[handle] || []; },
            get firstChild() { var k = _handleChildren[handle] || []; return k.length ? k[0] : null; },
            get lastChild() { var k = _handleChildren[handle] || []; return k.length ? k[k.length - 1] : null; },
            hasChildNodes: function () { return (_handleChildren[handle] || []).length > 0; },
          };
          return _tplContent;
        }
        if (prop === 'innerHTML') {
          // js-dom M4 R83：handle 元素（createElement 容器）——host 回调只反映
          // SetInnerHtmlOnHandle，appendChild 建的子树不可见（WPT ChildNode-before/after：
          // `parent.innerHTML` 期望含子）。改 JS 侧融合 childNodes 视图序列化（同 R81
          // textContent 模式）；无子回落 host 值（textContent= 写入）。
          if (handle) {
            var _ihKids = (function () {
              if (_isContainerHandle(handle)) return _handleChildNodes(handle);
              var _ihL = (typeof _zwLocalChildNodes === 'function') ? _zwLocalChildNodes(sel, handle) : null;
              var _ihK = (_handleChildren[handle] || []).slice();
              if (_ihL && _ihL.length) return _ihL.concat(_ihK);
              if (_ihK.length) return _ihK;
              if (_ihL) return _ihL;
              return [];
            })();
            if (_ihKids.length) {
              var _ihOut = '';
              for (var _ihi = 0; _ihi < _ihKids.length; _ihi++) {
                var _ihn = _ihKids[_ihi];
                if (!_ihn) continue;
                if (_ihn.nodeType === 3) _ihOut += _zwMEscapeText(_ihn.nodeValue != null ? _ihn.nodeValue : (_ihn.data != null ? _ihn.data : ''));
                else if (_ihn.nodeType === 8) _ihOut += '<!--' + (_ihn.nodeValue != null ? _ihn.nodeValue : _ihn.data) + '-->';
                else if (_ihn.nodeType === 1 && _ihn.__zwHandle) {
                  _ihOut += _makeProxy(null, _ihn.__zwHandle).outerHTML || '';
                } else if (_ihn.nodeType === 1 && typeof _ihn.outerHTML === 'string') {
                  _ihOut += _ihn.outerHTML;
                }
              }
              if (_ihOut !== '') return _ihOut;
            }
            return __zw_get_inner_html_handle(handle);
          }
          return __zw_get_inner_html(sel);
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
        if (prop === 'parentNode') {
          return _parentNodeFor(sel, handle);
        }
        if (prop === 'parentElement') {
          // spec `dom-node-parentelement`：parentElement 只返元素父——documentElement 的父是
          // Document（非元素）→ null。不能与 parentNode 共用 R79 的 html→document 分支：
          // parity 采集器/页面脚本沿 parentElement 上行到 html 后走进 document，node.tagName
          // 为 undefined → toLowerCase 崩溃（zeroweb-regression-guard 2026-08-17 发现）。
          return _parentNodeFor(sel, handle, true);
        }
        // 元素遍历/导航 API（仅元素子/兄弟，跳过文本/注释）。handle（脱离 DOM，无 sel）→ null/[]。
        if (prop === 'children') {
          // js-dom M4 R38：`Element.children` 返 HTMLCollection（spec `dom-parentnode-children`，带
          // item/namedItem + indexed/named properties）。旧返纯数组缺 namedItem（WPT HTMLCollection-empty-name
          // "Element.children" fail：`c.namedItem("")` 抛 TypeError）。经 _zwMakeCollection(true) 包成
          // HTMLCollection（含 R38 namedItem 空串守卫）。_splitSelectors 已 .map(_wrapSelector) 返 proxy 数组。
          // js-dom M4 R81：handle 元素（createElement 容器）从融合 childNodes 过滤元素子（pending
          // 子可见——WPT Node-properties testDiv.children[0..5]；host 回调只对 sel-based 有意义）。
          if (!sel && handle) {
            var _r81Kids = (function () {
              var _cl = (typeof _zwLocalChildNodes === 'function') ? _zwLocalChildNodes(sel, handle) : null;
              var _ck = (_handleChildren[handle] || []).slice();
              if (_cl && _cl.length) return _cl.concat(_ck);
              if (_ck.length) return _ck;
              if (_cl) return _cl;
              return [];
            })();
            return _zwMakeCollection(_r81Kids.filter(function (k) { return k && k.nodeType === 1; }), true);
          }
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
          // js-dom M4 R81：handle 元素（createElement 容器，mutation pending）从**融合
          // childNodes 视图**过滤元素子——WPT Node-properties testDiv.children[0]（setup 建
          // div 挂 body 后 append 6 paras，host 快照无 pending 子）旧恒 null/0。
          if (!sel && handle) {
            var _fecKids = (function () {
              var _fl2 = (typeof _zwLocalChildNodes === 'function') ? _zwLocalChildNodes(sel, handle) : null;
              var _fk2 = (_handleChildren[handle] || []).slice();
              if (_fl2 && _fl2.length) return _fl2.concat(_fk2);
              if (_fk2.length) return _fk2;
              if (_fl2) return _fl2;
              return [];
            })().filter(function (k) { return k && k.nodeType === 1; });
            if (prop === 'childElementCount') return _fecKids.length;
            if (!_fecKids.length) return null;
            return prop === 'firstElementChild' ? _fecKids[0] : _fecKids[_fecKids.length - 1];
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
          // js-dom M4 R81：handle 元素经父融合 childNodes 过滤元素子定位（host 回调只对 sel 有效；
          // WPT Node-properties paras[0].nextElementSibling——setup 建的 pending 子旧恒 null）。
          if (!sel && handle) {
            var _plink = _zwNodeParent ? _zwNodeParent[handle] : null;
            var _pk = null;
            if (_plink && _plink.parentSel) {
              var _ppx = _makeProxy(_plink.parentSel, null);
              _pk = _ppx.childNodes;
            } else if (_plink && _plink.parentHandle) {
              _pk = _makeProxy(null, _plink.parentHandle).childNodes;
            }
            if (_pk) {
              var _pkEls = [];
              for (var _pki = 0; _pki < _pk.length; _pki++) {
                if (_pk[_pki] && _pk[_pki].nodeType === 1) _pkEls.push(_pk[_pki]);
              }
              var _pidx = -1;
              for (var _pj = 0; _pj < _pkEls.length; _pj++) { if (_pkEls[_pj] === _makeProxy(sel, handle)) { _pidx = _pj; break; } }
              if (prop === 'previousElementSibling') return _pidx > 0 ? _pkEls[_pidx - 1] : null;
              return _pidx >= 0 && _pidx < _pkEls.length - 1 ? _pkEls[_pidx + 1] : null;
            }
            return null;
          }
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
          // R34xx/R51：注册的纯文本元素（textContent=/innerHTML= 建的本地文本视图）与 R2927
          // registry 子（appendChild 建的元素/节点子）**融合**——WPT dom/common.js indexOf 等
          // identity 循环要求「append 的子出现在 childNodes」：paras[0].textContent=... 后
          // 再 appendChild(paras[1])，旧短路（_zwLocalChildNodes 命中即 return）使 append 的
          // 子不可见 → indexOf 死循环（R51 修复）。文本子在前（textContent 视图建在先）。
          if (!sel && handle) {
            var _r51Local = (typeof _zwLocalChildNodes === 'function')
              ? _zwLocalChildNodes(sel, handle)
              : null;
            var _r51Kids = (_handleChildren[handle] || []).slice();
            if (_r51Local && _r51Local.length) return _r51Local.concat(_r51Kids);
            if (_r51Kids.length) return _r51Kids;
            if (_r51Local) return _r51Local;
            // R86：注册表 miss + registry 空 → 移除物化缓存（detached 子树保留其子；
            // WPT NodeIterator-removal：remove 后 paras[0].firstChild 期望 #text）。
            var _r86det = (typeof _zwDetachedChildrenOf === 'function')
              ? _zwDetachedChildrenOf(handle)
              : null;
            if (_r86det && _r86det.length) return _r86det.slice();
            return _childNodeList(sel, handle);
          }
          // R50：普通 handle 元素（createElement 后 append 子——mutation pending、无 selector）
          // 从 R2927 registry 读子（appendChild 对所有 handle 父都 _recordHandleChild）。
          // WPT case.js：`container.childNodes`（detached handle 容器）此前恒 [] → expected
          // 伪空（双缺陷与查询侧抵消）；live 集合修复后查询侧正确暴露 expected 侧缺陷。
          if (typeof _zwLocalChildNodes === 'function') {
            var _zwLocal = _zwLocalChildNodes(sel, handle);
            if (_zwLocal) return _zwLocal;
          }
          return _childNodeList(sel, handle);
        }
        if (prop === 'firstChild' || prop === 'lastChild') {
          // R49：firstChild/lastChild 同步消费 _zwLocalChildNodes（textContent=/innerHTML= 的本地
          // 文本视图——childNodes 已接，此处漏；WPT takeRecords `n.textContent='old'; n.firstChild.data=`）。
          // R51：与 childNodes 同款融合（本地文本视图 + registry 子）——lastChild 须反映 append
          // 的元素子（textContent 后 appendChild 的融合序：text 在前、handle 子在后）。
          var cn = _isContainerHandle(handle)
            ? _handleChildNodes(handle)
            : (function () {
                if (!sel && handle) {
                  var _fl = (typeof _zwLocalChildNodes === 'function')
                    ? _zwLocalChildNodes(sel, handle)
                    : null;
                  var _fk = (_handleChildren[handle] || []).slice();
                  if (_fl && _fl.length) return _fl.concat(_fk);
                  if (_fk.length) return _fk;
                  if (_fl) return _fl;
                  // R86：移除物化缓存回落（detached 子树保留其子）。
                  var _fl86 = (typeof _zwDetachedChildrenOf === 'function')
                    ? _zwDetachedChildrenOf(handle)
                    : null;
                  if (_fl86 && _fl86.length) return _fl86.slice();
                  return _childNodeList(sel, handle);
                }
                if (typeof _zwLocalChildNodes === 'function') {
                  var _loc = _zwLocalChildNodes(sel, handle);
                  if (_loc) return _loc;
                }
                // R51：普通 handle 元素（createElement 容器，mutation pending 无 sel）从 R2927
                // registry 读子（与 childNodes R50 回落对称——WPT TreeWalker-basic
                // createSampleDOM `root.lastChild.firstChild` 读 detached 容器子）。
                if (!sel && handle && _handleChildren[handle] && _handleChildren[handle].length) {
                  return _handleChildren[handle].slice();
                }
                // R86：移除物化缓存回落（同上）。
                var _loc86 = (typeof _zwDetachedChildrenOf === 'function')
                  ? _zwDetachedChildrenOf(handle)
                  : null;
                if (_loc86 && _loc86.length) return _loc86.slice();
                return _childNodeList(sel, handle);
              })();
          if (!cn.length) return null;
          return prop === 'firstChild' ? cn[0] : cn[cn.length - 1];
        }
        if (prop === 'previousSibling' || prop === 'nextSibling') {
          // js-dom M4 R79：handle 元素（createElement+appendChild 建的 pending 节点）的兄弟
          // 导航——旧仅 sel-based（快照查询），handle 元素恒 null → WPT oracle previousNode
          // 遍历断链（Node-compareDocumentPosition 25F 尾簇：previousSibling=null 使 backward
          // 树序跳过兄弟，期望值与文档序矛盾）。经 `_zwNodeParent[handle]` 反链取父，父的
          // childNodes 融合视图（registry + overlay）定位 index ±1。
          if (!sel && handle) {
            var _r79link = _zwNodeParent[handle];
            var _r79parent = null;
            if (_r79link) {
              if (_r79link.parentSel) _r79parent = _wrapSelector(_r79link.parentSel);
              else if (_r79link.parentHandle) _r79parent = _wrapHandle(_r79link.parentHandle);
            }
            if (_r79parent && _r79parent.childNodes) {
              try {
                var _r79kids = _r79parent.childNodes;
                var _r79self = _makeProxy(sel, handle);
                for (var _r79i = 0; _r79i < _r79kids.length; _r79i++) {
                  if (_r79kids[_r79i] === _r79self) {
                    if (prop === 'previousSibling') return _r79i > 0 ? _r79kids[_r79i - 1] : null;
                    return _r79i + 1 < _r79kids.length ? _r79kids[_r79i + 1] : null;
                  }
                }
              } catch (_e79) {}
            }
            return null;
          }
          // js-dom M4 R85：html 的兄弟走 document.childNodes（真浏览器 html.previousSibling
          // = doctype、nextSibling=null——host __zw_sibling_nodes 对 html 无父返 null，
          // 使 WPT oracle 的 expected 计算与 walker 实现分歧：oracle expected null、
          // walker 返 doctype → "expected null but got DocumentType" 根因）。
          if (sel === 'html' && globalThis.document) {
            var _dk = globalThis.document.childNodes || [];
            var _di = _dk.indexOf(_makeProxy(sel, handle));
            if (_di >= 0) {
              return prop === 'previousSibling' ? (_di > 0 ? _dk[_di - 1] : null)
                : (_di + 1 < _dk.length ? _dk[_di + 1] : null);
            }
          }
          if (!sel || typeof __zw_sibling_nodes !== 'function') return null;
          // js-dom M4 R55：兄弟对缓存（与 _zwChildBaseCache 同款生命周期——dom_html Arc 回合内
          // 不可变；重注册经 globalThis._zwSiblingBaseInvalidateAll 全量失效）。同 turn 内
          // nextSibling/previousSibling 交替读（Range testFn 边界点遍历）不再每次双 host 往返
          //（__zw_sibling_nodes + __zw_parent）+ 重包装。
          var _sb = _zwSiblingBaseCache.get(sel);
          if (!_sb) {
            try {
              var pair = JSON.parse(__zw_sibling_nodes(sel) || '{"p":null,"n":null}');
            } catch (_e2) { return null; }
            // js-dom M4 R84：sibling 对的 text/comment 子统一走 `_childNodeList(parentSel)`
            // 取——与 head.childNodes[i] **同 identity**（_zwChildBaseCache 缓存保证），使
            // sibling text 节点与 childNodes 视图合一（oracle nextNode() 树序遍历靠
            // parentNode.childNodes.indexOf 定位兄弟；旧 _wrapNodeEntry(pair, null) 独立
            // 包装 → parentNode=null + 兄弟静态 null → 遍历断链，NodeIterator/TreeWalker
            // 整簇 fail 的根因之一）。父 selector 经 __zw_parent；元素子仍走 pair（selector
            // identity 由 _proxyCache 保证）。
            var _sbParentSel = (typeof __zw_parent === 'function') ? __zw_parent(sel) : null;
            var _sbParent = _sbParentSel ? _wrapSelector(_sbParentSel) : null;
            var _sbByPos = null;
            if (_sbParentSel) {
              try {
                var _sbKids = _childNodeList(_sbParentSel, null);
                var _sbIdx = _sbKids.indexOf(_makeProxy(sel, handle));
                if (_sbIdx >= 0) _sbByPos = { p: _sbKids[_sbIdx - 1] || null, n: _sbKids[_sbIdx + 1] || null };
              } catch (_e3) {}
            }
            _sb = _sbByPos || {
              p: _wrapNodeEntry(pair.p, _sbParent),
              n: _wrapNodeEntry(pair.n, _sbParent),
            };
            if (_zwSiblingBaseCache.size > 512) _zwSiblingBaseCache.clear();
            _zwSiblingBaseCache.set(sel, _sb);
          }
          return prop === 'previousSibling' ? _sb.p : _sb.n;
        }
        // `el.contains(other)`——other 是否为 el 的后代或 el 自身（沿 parent 链）。
        // js-dom M4 R79：改 JS 侧 `_zwNodeContains`（parentNode 链 identity 上行）——旧走
        // host `__zw_contains`（sel 快照）对 pending 节点（setupRangeTests createElement 建）
        // 恒 false（WPT Node-contains 1002F 整簇根因）；WPT 用例 oracle 同构算法，pending
        // 自然正确（R51b 反链 + proxy 缓存 identity）。
        if (prop === 'contains') {
          return function(other) {
            return _zwNodeContains(_makeProxy(sel, handle), other);
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
          // js-dom M3 R90：handle 元素先查父反链（append 后 host mutation 异步应用前
          // getBoundingClientRect 尚空——WC 组件 connectedCallback 内 isConnected 读
          // false 与 spec 相悖）。沿 _zwNodeParent 上行到宿主 sel 节点即 connected；
          // 无链且无 rect → false。记录形态 { parentSel, parentHandle }（part01 汇流点）。
          // R91：shadow 边界穿越——反链到达 shadow root 容器 handle（无 parentSel 记录）
          // 时经 _shadowHandleMeta 跳到 host 继续（spec connected：shadow 树随 host 连入
          // 文档即 connected；WPT Node-isConnected-shadow-dom open/closed）。
          if (handle && typeof _zwNodeParent !== 'undefined' && _zwNodeParent) {
            try {
              var _r90p = _zwNodeParent[handle];
              // 反链 miss 时：本元素可能直接 append 到 shadow root（shadow 容器
              // appendChild 的记账路径）——经 parentNode 读容器再跳 host。
              if (!_r90p) {
                var _r91par = target && target.parentNode;
                if (_r91par && _r91par.__zwHandle) _r90p = { parentSel: null, parentHandle: _r91par.__zwHandle };
              }
              var _r90hops = 0;
              while (_r90p && _r90hops++ < 64) {
                if (_r90p.parentSel) return true; // 挂到 sel 节点（html/body/容器）→ 在档
                var _r90ph = _r90p.parentHandle;
                if (!_r90ph) break;
                // shadow root 容器 → host 跳转。
                var _r91meta = typeof _shadowHandleMeta !== 'undefined' && _shadowHandleMeta[_r90ph];
                if (_r91meta) {
                  if (_r91meta.hostSel) return true; // host 是 sel 节点 → 在档
                  _r90p = _zwNodeParent[_r91meta.hostHandle];
                  if (!_r90p && _r91meta.hostHandle) {
                    // host 是 handle 元素（未挂载或挂到 handle 容器）——回落 rect 探测 host。
                    if (typeof __zw_getBoundingClientRect === 'function') {
                      try { return __zw_getBoundingClientRect(_r91meta.hostHandle) !== ''; } catch (_e91) {}
                    }
                  }
                  continue;
                }
                _r90p = _zwNodeParent[_r90ph];
              }
            } catch (_e90) {}
          }
          if (handle && typeof __zw_getBoundingClientRect === 'function') {
            try { return __zw_getBoundingClientRect(handle) !== ''; } catch (_e) { return false; }
          }
          return false;
        }
        // `el.hasChildNodes()`（spec Node.hasChildNodes：是否有任意子节点含文本/注释）——树遍历 / diff /
        // 子节点存在性检查高频。js-dom M4 R79：改用与 firstChild/lastChild **同款融合视图**
        //（_zwLocalChildNodes 文本视图 + _handleChildren registry + _childNodeList 快照）——旧仅
        // `_childNodeList` 对 handle 元素（textContent= 后 append 的 pending 节点）漏本地文本子
        // → hasChildNodes=false 而 firstChild 非 null 自相矛盾（WPT Node-compareDocumentPosition
        // oracle 的 previousNode 下降依赖 hasChildNodes，17F 尾簇根因）。
        if (prop === 'hasChildNodes') {
          return function() {
            if (_isContainerHandle(handle)) return _handleChildNodes(handle).length > 0;
            if (!sel && handle) {
              var _hnL = (typeof _zwLocalChildNodes === 'function')
                ? _zwLocalChildNodes(sel, handle)
                : null;
              var _hnK = (_handleChildren[handle] || []).slice();
              if (_hnL && _hnL.length) return true;
              if (_hnK.length) return true;
              if (_hnL) return false;
            }
            return _childNodeList(sel, handle).length > 0;
          };
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
        // js-dom M4 R79：改 JS 侧 `_zwCompareDocumentPosition`（parentNode 链 + LCA + childNodes 序）——
        // 旧走 `_ancestorChain`（host sel 快照）+ `__zw_element_children` 对 pending 节点恒
        // DISCONNECTED|IMPL(33)（WPT Node-compareDocumentPosition 1444F 整簇根因），且 text/comment
        // 节点无 sel 直接兜底错。新实现全节点形态（element/text/comment/document）统一。
        if (prop === 'compareDocumentPosition') {
          return function(other) {
            return _zwCompareDocumentPosition(_makeProxy(sel, handle), other);
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
          // js-dom M4 R80：HTML 文档 + HTML 命名空间的 createElementNS 元素 tagName 为 ASCII 大写
          // qualifiedName（spec dom-document-createelementns；`createElementNS(HTMLNS,'span')` →
          // 'SPAN'）；否则原值大小写敏感（XML 语义）。
          if (isNs) {
            var _nsh = _nsHandles[handle];
            return _nsh.htmlUpper ? _nsh.qualifiedName.toUpperCase() : _nsh.qualifiedName;
          }
          return (isFrag || isShadow || isComment || isText || isPI) ? undefined : _realTag(sel, handle);
        }
        // `element.localName`（spec `dom-element-localname`，R11）：HTML 元素 = tagName 小写；
        // 带 prefix 的限定名（`svg:rect`，createElementNS）去 prefix 取冒号后。非 Element → null
        //（spec Attr/Text 等另走各自接口，此处元素 getter 范围）。createElement 用例核心断言之一。
        // createElementNS handle（isNs）：spec createElementNS **不**小写，原样返冒号后大小写敏感值。
        if (prop === 'localName') {
          if (isFrag || isShadow || isComment || isText || isPI) return null;
          if (isNs) return _nsLocal(_nsHandles[handle].qualifiedName);
          // R81 spec 纠正：HTML createElement 对**非法**限定名（含冒号——Name production 允许
          // ':' 但 HTML 元素名不允许）**不解析 prefix**——localName = 全名 ASCII 小写（WPT
          // Document-createElement `createElement(":")` 期望 localName ":"；`"f:oo"` 期望
          // "f:oo"——HTML parser 不拆 NS，冒号是普通字符）。ASCII-only 小写（spec
          // ASCII-lowercase——'İ'/'K' 等 Unicode 大写不变，JS toLowerCase 会错误转换）。
          var _ln = _realTag(sel, handle);
          var _lnOut = '';
          for (var _lni = 0; _lni < _ln.length; _lni++) {
            var _lnc = _ln.charAt(_lni);
            _lnOut += (_lnc >= 'A' && _lnc <= 'Z') ? String.fromCharCode(_lnc.charCodeAt(0) + 32) : _lnc;
          }
          return _lnOut;
        }
        // `element.prefix`（spec `dom-node-prefix`，R12）：限定名冒号前部分；无冒号 → null。非 Element → null。
        // createElementNS handle（isNs）：从原 qualifiedName 冒号前取，大小写敏感（spec createElementNS
        // 不小写 prefix，`"abc:l"` → prefix `"abc"`）。无 prefix（无冒号）→ null。普通 createElement 元素
        // 经 `_realTag`（强制大写）；createElement 不带 prefix 故恒 null。
        if (prop === 'prefix') {
          if (isFrag || isShadow || isComment || isText || isPI) return null;
          if (isNs) return _nsPrefix(_nsHandles[handle].qualifiedName);
          // R81 spec 纠正：HTML createElement 的含冒号名**无 prefix**（localName = 全名小写，
          // WPT `createElement("f:oo")` 期望 prefix null——HTML 元素不经 NS 解析）。
          return null;
        }
        if (prop === 'nodeName') {
          if (isShadow) return '#shadow-root';
          if (isFrag) return '#document-fragment';
          if (isComment) return '#comment';
          if (isText) return '#text';
          if (isPI) return _piHandles[handle].target;
          // R80：HTML 命名空间 createElementNS 的 nodeName 与 tagName 同（ASCII 大写 qualifiedName）。
          if (isNs) {
            var _nsn = _nsHandles[handle];
            return _nsn.htmlUpper ? _nsn.qualifiedName.toUpperCase() : _nsn.qualifiedName;
          }
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
        // js-dom M4 R81：Text/Comment 的 textContent = data（spec dom-node-textcontent——
        // CharacterData 的 textContent 与 data 同源；旧落到 undefined）。PI 分支在上方已处理。
        if ((isText || isComment) && prop === 'textContent') {
          return handle ? __zw_get_text_handle(handle) : '';
        }
        // js-dom M4 R81：Text 的 wholeText = 同父相邻文本节点 data 拼接（spec dom-text-wholetext；
        // 无父/无兄弟即自身 data——WPT Node-properties detachedTextNode.wholeText）+ length
        // = data 长度（spec CharacterData.length）。
        if (isText && prop === 'wholeText') {
          var _wtData = handle ? __zw_get_text_handle(handle) : '';
          var _wtParent = _parentNodeFor(sel, handle);
          if (!_wtParent || !_wtParent.childNodes) return _wtData;
          var _wtOut = '';
          for (var _wti = 0; _wti < _wtParent.childNodes.length; _wti++) {
            var _wtc = _wtParent.childNodes[_wti];
            if (_wtc && _wtc.nodeType === 3) _wtOut += String(_wtc.data != null ? _wtc.data : '');
          }
          return _wtOut || _wtData;
        }
        // js-dom M4 R81：Text/Comment/PI 节点无子（spec——CharacterData/PI 为叶子节点）：
        // firstChild/lastChild/childNodes 恒 null/[]。旧落入元素分支读 host 子列表（对叶子
        // 无意义且可能返回 undefined ≠ null）。WPT Node-textContent `emptyText.firstChild===null`。
        if ((isText || isComment || isPI) && (prop === 'firstChild' || prop === 'lastChild')) {
          return null;
        }
        // js-dom M4 R80：Element 节点的 nodeValue = null（spec dom-node-nodevalue——Attr/Document/
        // DocumentFragment/Element 恒 null；旧缺此分支返 undefined，WPT Document-createElementNS
        // `assert_equals(element.nodeValue, null)` 85F 簇）。textContent setter 另有分支（下方）。
        // R81：fragment/shadow 的 nodeValue 也恒 null（spec——DocumentFragment 无 nodeValue）。
        if (prop === 'nodeValue') {
          return null;
        }
        // js-dom M4 R81：fragment/shadow 的 textContent = 子树文本拼接（spec dom-node-textcontent
        // 对 DocumentFragment 与 Element 同构；WPT Node-properties xmlDocfrag.textContent 期望 ""
        // 非 undefined）。
        if ((isFrag || isShadow) && prop === 'textContent') {
          var _fkIds = _handleChildNodes(handle);
          var _fkOut = '';
          for (var _fki = 0; _fki < _fkIds.length; _fki++) {
            var _fkn = _fkIds[_fki];
            if (!_fkn) continue;
            if (_fkn.nodeType === 3 || _fkn.nodeType === 4) _fkOut += String(_fkn.nodeValue != null ? _fkn.nodeValue : (_fkn.data != null ? _fkn.data : ''));
            else if (_fkn.nodeType === 1 && typeof _fkn.textContent === 'string') _fkOut += _fkn.textContent;
          }
          return _fkOut;
        }
        // ProcessingInstruction 节点（js-dom M4）：`.target` = PI target，`.data`/`.nodeValue` = PI data
        //（spec `dom-processinginstruction`：data 即 CharacterData.data；nodeName = target 见上）。读自 _piHandles。
        if (isPI) {
          var _pi = _piHandles[handle];
          if (prop === 'target') return _pi ? _pi.target : '';
          if (prop === 'data' || prop === 'nodeValue') return _pi ? _pi.data : '';
          if (prop === 'length') return _pi ? _pi.data.length : 0;
          // R81：PI 的 textContent = data（spec CharacterData：textContent 与 data 同源；WPT
          // "For a ProcessingInstruction, textContent should set the data"）。
          if (prop === 'textContent') return _pi ? _pi.data : '';
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
            // js-dom M4 R46：spec `dom-element-removeattribute`——属性不存在时移除**不产生 mutation
            // record**（real browser：queue a mutation record 仅在「已存在的属性被移除」时）。
            // WPT MutationObserver-attributes "removal no mutation"（n71 无 class，removeAttribute('class')
            // 后仅 id 改名 1 条 record）。presence 经 hasAttribute 同源判定（handle/sel latest-wins）。
            var _rmExisted = false;
            try {
              if (handle && typeof __zw_has_attr_handle === 'function') _rmExisted = __zw_has_attr_handle(handle, n) === '1';
              else if (typeof __zw_has_attr_lw === 'function') _rmExisted = __zw_has_attr_lw(sel, n) === '1';
              else if (typeof __zw_has_attr === 'function') _rmExisted = __zw_has_attr(sel, n) === '1';
            } catch (_e) {}
            // 同步客户端缓存：class→_classCache、value→_inputValues，使 setAttribute 与
            // classList/className、.value getter 协作一致（否则后续 classList.add 读 stale 缓存丢值）。
            if (n === 'class') _classCache[key] = v;
            else if (n === 'value') { _inputValues[key] = v; _inputValuesSet[key] = true; _clearInputDefault(key); } // R2996：setAttribute('value') 重同步 defaultValue
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
            // js-dom M4 R46：spec `dom-element-removeattribute`——属性不存在时移除**不产生 mutation
            // record**（real browser：queue a mutation record 仅在「已存在的属性被移除」时）。
            // WPT MutationObserver-attributes "removal no mutation"（n71 无 class，removeAttribute('class')
            // 后仅 id 改名 1 条 record）。presence 经 hasAttribute 同源判定（handle/sel latest-wins）。
            var _rmExisted = false;
            try {
              if (handle && typeof __zw_has_attr_handle === 'function') _rmExisted = __zw_has_attr_handle(handle, n) === '1';
              else if (typeof __zw_has_attr_lw === 'function') _rmExisted = __zw_has_attr_lw(sel, n) === '1';
              else if (typeof __zw_has_attr === 'function') _rmExisted = __zw_has_attr(sel, n) === '1';
            } catch (_e) {}
            // 真移除（区别于 set-empty 残留 `attr=""`——boolean 属性 checked/disabled 设空值仍 present
            // → hasAttribute 误 true）。handle 元素经 `__zw_remove_attr_handle`（RemoveAttrOnHandle，R2993）；
            // sel-based 经 `__zw_remove_attr`（RemoveAttr，R2657）；无回调 → fallback set-empty。
            // 同步客户端缓存（class/value），使后续 classList/.value 反映移除。
            if (n === 'class') _classCache[key] = '';
            else if (n === 'value') { _inputValues[key] = ''; _inputValuesSet[key] = true; _clearInputDefault(key); } // R2996：removeAttribute('value') 重同步 defaultValue
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
            if (_rmExisted) _mo_notify(sel, handle, { type: 'attributes', attributeName: n, oldValue: moOld });
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
        // js-dom M4 R46：NS 属性族的 MutationObserver record——spec `mutation-observer-attributes`：
        // record.attributeName = **localName**、attributeNamespace = ns（WPT MutationObserver-attributes
        // "setAttributeNS: creation" 断言 namespace="http://example.org/" / attributeName="lang"——
        // 旧委托 setAttribute 使 record 带限定名 "xml:lang" 且 namespace null）。NS 族四方法自带
        // notify（绕过 delegate 的 setAttribute/removeAttribute notify），pre 捕获 old 同款。
        if (prop === 'setAttributeNS') {
          return function(_ns, qualifiedName, value) {
            var ns = _ns == null ? null : String(_ns);
            var qn = String(qualifiedName);
            var local = qn.indexOf(':') >= 0 ? qn.slice(qn.indexOf(':') + 1) : qn;
            var _nsMoId = _mo_id(handle, sel);
            var _nsOld = (_nsMoId != null && _mo_any_wants_attr_old(_nsMoId, local))
              ? _mo_read_attr(sel, handle, _nsQualName(ns, local)) : null;
            // 直写 host 回调（不经 proxy.setAttribute——那条路径自带无 namespace 的 notify，会双发）。
            if (handle && typeof __zw_set_attr_handle === 'function') __zw_set_attr_handle(handle, qn, String(value));
            else if (typeof __zw_set_attr === 'function') __zw_set_attr(sel, qn, String(value == null ? '' : value));
            else proxy.setAttribute(qn, value);
            _mo_notify(sel, handle, {
              type: 'attributes', attributeName: local,
              attributeNamespace: ns, oldValue: _nsOld,
            });
          };
        }
        if (prop === 'getAttributeNS') {
          return function(ns, localName) { return proxy.getAttribute(_nsQualName(ns, localName)); };
        }
        if (prop === 'hasAttributeNS') {
          return function(ns, localName) { return proxy.hasAttribute(_nsQualName(ns, localName)); };
        }
        if (prop === 'removeAttributeNS') {
          return function(_ns, localName) {
            var ns = _ns == null ? null : String(_ns);
            var local = String(localName);
            var qname = _nsQualName(ns, local);
            var _nsMoId2 = _mo_id(handle, sel);
            var _nsExisted = false;
            try {
              if (handle && typeof __zw_has_attr_handle === 'function') _nsExisted = __zw_has_attr_handle(handle, qname) === '1';
              else if (typeof __zw_has_attr_lw === 'function') _nsExisted = __zw_has_attr_lw(sel, qname) === '1';
              else if (typeof __zw_has_attr === 'function') _nsExisted = __zw_has_attr(sel, qname) === '1';
            } catch (_e) {}
            var _nsOld2 = (_nsExisted && _nsMoId2 != null && _mo_any_wants_attr_old(_nsMoId2, local))
              ? _mo_read_attr(sel, handle, qname) : null;
            // 直删 host 回调（不经 proxy.removeAttribute——那条路径自带无 namespace 的 notify，会双发）。
            if (handle && typeof __zw_remove_attr_handle === 'function') __zw_remove_attr_handle(handle, qname);
            else if (typeof __zw_remove_attr === 'function') __zw_remove_attr(sel, qname);
            else proxy.removeAttribute(qname);
            // R46：缺失属性的 NS 移除不发 record（同 removeAttribute presence guard）。
            if (_nsExisted) _mo_notify(sel, handle, {
              type: 'attributes', attributeName: local,
              attributeNamespace: ns, oldValue: _nsOld2,
            });
          };
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
            // R57（FV M1）：:invalid/:valid 伪类——约束校验联动（host 样式引擎
            // 未实现——pattern-dynamic/number-validity-dynamic 用例）。
            var q = String(selector);
            if (q === ':invalid') return !_validityState(key, sel, handle).valid;
            if (q === ':valid') return _validityState(key, sel, handle).valid;
            if (!sel || typeof __zw_matches !== 'function') return false;
            try { return __zw_matches(sel, q) === '1'; } catch (_e) { return false; }
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
            if (notPrevented) {
              // R57（FV M1）：click 默认动作——checkbox/radio 的 checked 切换
              //（spec §4.10.5.2.4 的 radio 组语义——radio-group-valueMissing 的
              // fourth.click()）。属性层面（shim 的 checked 读取 = 属性存在性）。
              var _tagC = _realTag(sel, handle);
              if (_tagC === 'INPUT') {
                var _tyC = '';
                try { _tyC = handle ? __zw_get_attr_handle(handle, 'type') : __zw_get_attr(sel, 'type'); } catch (_e) { _tyC = ''; }
                _tyC = String(_tyC || '').toLowerCase();
                var _curC = null;
                try { _curC = handle ? __zw_has_attr_handle(handle, 'checked') : __zw_has_attr(sel, 'checked'); } catch (_e) {}
                if (_tyC === 'checkbox') {
                  if (_curC === '1') {
                    if (sel && typeof __zw_remove_attr === 'function') { try { __zw_remove_attr(sel, 'checked'); } catch (_e2) {} }
                  } else if (sel && typeof __zw_set_attr === 'function') {
                    try { __zw_set_attr(sel, 'checked', ''); } catch (_e2) {}
                  }
                } else if (_tyC === 'radio') {
                  // 组内互斥：勾选当前 + 同 name 其他取消
                  if (sel && typeof __zw_set_attr === 'function') { try { __zw_set_attr(sel, 'checked', ''); } catch (_e2) {} }
                  var _nmC = null;
                  try { _nmC = handle ? __zw_get_attr_handle(handle, 'name') : __zw_get_attr(sel, 'name'); } catch (_e3) {}
                  if (_nmC != null && String(_nmC) !== '') {
                    try {
                      var _qC = 'input[type="radio"][name="' + String(_nmC).replace(/\\/g, '\\\\').replace(/"/g, '\\"') + '"]';
                      var _allC = globalThis.document.querySelectorAll(_qC);
                      for (var _ci = 0; _ci < _allC.length; _ci++) {
                        var _rc = _allC.item ? _allC.item(_ci) : _allC[_ci];
                        try {
                          if (_rc.__zwSelector && _rc.__zwSelector !== sel && typeof __zw_remove_attr === 'function') {
                            __zw_remove_attr(_rc.__zwSelector, 'checked');
                          }
                        } catch (_e4) {}
                      }
                    } catch (_e5) {}
                  }
                }
              }
              // R57（FV M3）：submit 按钮 click 默认动作——表单提交（spec §4.10.5.4 的
              // submit button activation behavior：form owner 为 null → no-op；经共享
              // _zwRunFormSubmit（interactive validation + submit 派发 + 重入守卫）——
              // form-requestsubmit 的 click()+requestSubmit() 重入用例）。
              var _subFrm = null;
              var _subTag = _realTag(sel, handle);
              if (_subTag === 'INPUT' || _subTag === 'BUTTON') {
                var _subTy = '';
                try {
                  _subTy = handle ? __zw_get_attr_handle(handle, 'type') : __zw_get_attr(sel, 'type');
                } catch (_e6) { _subTy = ''; }
                _subTy = String(_subTy || '').toLowerCase();
                var _subIsBtn = (_subTag === 'BUTTON' && (_subTy === 'submit' || _subTy === ''))
                  || (_subTag === 'INPUT' && (_subTy === 'submit' || _subTy === 'image'));
                if (_subIsBtn) {
                  // disabled 表单控件无激活行为（spec：click() 对 disabled form control 直接 return）。
                  try {
                    var _subDis = handle ? __zw_has_attr_handle(handle, 'disabled') : __zw_has_attr(sel, 'disabled');
                    if (_subDis === '1') _subIsBtn = false;
                  } catch (_e7) {}
                }
                if (_subIsBtn) {
                  try { _subFrm = _makeProxy(sel, handle).form; } catch (_e8) { _subFrm = null; }
                }
              }
              if (_subFrm) {
                var _subFrmSel = _subFrm.__zwSelector || null;
                var _subFrmH = _subFrm.__zwHandle || null;
                _zwRunFormSubmit(_elKey(_subFrmSel, _subFrmH), _subFrmSel, _subFrmH, _makeProxy(sel, handle));
              }
              // R3072：popovertarget 声明式触发——click default action（未 preventDefault 时）。找最近含 popovertarget
              // 祖先 → 按 popovertargetaction 触发目标 popover show/hide/toggle。无 popovertarget 时 no-op（零回归）。
              _zwPopoverTargetActivate(key, sel, handle);
            }
            return notPrevented;
          };
        }
        // Constraint Validation API（R2825）——表单校验库（checkValidity gate submit / setCustomValidity
        // 自定义错误 / validity.valid 读 / validationMessage 显示）高频。customError 由 _customValidity 跟踪；
        // 原生约束 headless 不强制（permissive valid）。checkValidity/reportValidity invalid 时派发 'invalid'
        // 事件（cancelable，非 bubble，经 _dispatchWithBubble）。
        if (prop === 'checkValidity' || prop === 'reportValidity') {
          return function() {
            // R57（FV M1）：FORM 元素的 checkValidity/reportValidity——遍历 form
            // 内控件（input/select/textarea/button），任一 invalid → false
            //（spec §4.10.5.4 interactive validation 的 form 级检查——
            // form-validation-checkValidity/reportValidity 的 "(in a form)" 变体）。
            if (_realTag(sel, handle) === 'FORM') {
              // 控件查询经 form proxy 的 querySelectorAll（sel/handle 两路径——
              // __zw_query_all_sub / _handleQueryAll）；本地 _zwMEl 树（未 host
              // 注册）走 node.checkValidity 的 _collectControls。
              var ctrls = [];
              try {
                var all = _makeProxy(sel, handle).querySelectorAll('input,select,textarea,button');
                for (var ci = 0; ci < all.length; ci++) ctrls.push(all.item ? all.item(ci) : all[ci]);
              } catch (_e) {}
              for (var cj = 0; cj < ctrls.length; cj++) {
                var cv = ctrls[cj].checkValidity ? ctrls[cj].checkValidity() : true;
                if (!cv) return false;
              }
              return true;
            }
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
        // R57（FV M1）：validationMessage 的 barred 判定（disabled/非 submittable
        // → 空消息）——submittable 元素（button/input/select/textarea/等）+ 非 disabled。
        function _wvSubmittable(sl, hd) {
          var _t = _realTag(sl, hd);
          if (_t !== 'INPUT' && _t !== 'SELECT' && _t !== 'TEXTAREA' && _t !== 'BUTTON') return false;
          var _d = null;
          try { _d = hd ? __zw_has_attr_handle(hd, 'disabled') : __zw_has_attr(sl, 'disabled'); } catch (_e) {}
          return _d !== '1';
        }
        if (prop === 'validationMessage') {
          // R57（FV M1）：barred（disabled/非 submittable）控件——validationMessage
          // 空（spec §4.10.5.2.2——customError 也 barred——"when control is
          // disabled" 的断言）。
          if (!_wvSubmittable(sel, handle)) return '';
          var validity = _validityState(key, sel, handle);
          if (validity.customError) return _customValidity[key];
          // R57（FV M2）：约束位的 Chromium 标准消息（spec §4.10.5.2.3——
          // validationMessage 的 UA 定义消息；WPT constraints 不断言内容——
          // 对齐 Chromium 固定串）
          if (validity.valueMissing) return 'Please fill out this field.';
          if (validity.typeMismatch) {
            var _tmTy = '';
            try { _tmTy = handle ? __zw_get_attr_handle(handle, 'type') : __zw_get_attr(sel, 'type'); } catch (_e) {}
            if (String(_tmTy).toLowerCase() === 'email') return 'Please enter an email address.';
            if (String(_tmTy).toLowerCase() === 'url') return 'Please enter a URL.';
            return 'Please enter a valid value.';
          }
          if (validity.patternMismatch) return 'Please match the requested format.';
          if (validity.rangeOverflow) {
            var _mx = null;
            try { _mx = handle ? __zw_get_attr_handle(handle, 'max') : __zw_get_attr(sel, 'max'); } catch (_e) {}
            return _mx != null && String(_mx) !== '' ? 'Value must be less than or equal to ' + String(_mx) + '.' : 'Please enter a valid value.';
          }
          if (validity.rangeUnderflow) {
            var _mn = null;
            try { _mn = handle ? __zw_get_attr_handle(handle, 'min') : __zw_get_attr(sel, 'min'); } catch (_e) {}
            return _mn != null && String(_mn) !== '' ? 'Value must be greater than or equal to ' + String(_mn) + '.' : 'Please enter a valid value.';
          }
          if (validity.stepMismatch) return 'Please enter a valid value.';
          if (validity.tooShort) return 'Please lengthen this text.';
          if (validity.tooLong) return 'Please shorten this text.';
          return '';
        }
        if (prop === 'willValidate') {
          // R57（FV M1）：willValidate 排除（barred from constraint validation——
          // spec §4.10.5.2.2）：disabled、readonly（text 类）、type ∈ {hidden,
          // button, reset}、**datalist 祖先**（willValidate-datalist——spec
          // §4.10.5.2.2：datalist 元素的后代被 barred）。
          // datalist 祖先：parentNode 链（sel/handle 两路径——_parentNodeFor；
          // 经 proxy 的 parentNode getter 统一）。
          var _wv = _makeProxy(sel, handle);
          var _wp = _wv.parentNode;
          var _guard = 0;
          while (_wp && _guard < 64) {
            try { if (String(_wp.tagName).toLowerCase() === 'datalist') return false; } catch (_e) {}
            try { _wp = _wp.parentNode; } catch (_e) { break; }
            _guard++;
          }
          if (handle) {
            try { if (__zw_has_attr_handle(handle, 'disabled') === '1') return false; } catch (_e) {}
          } else if (sel) {
            try {
              if ((typeof __zw_has_attr_lw === 'function' && __zw_has_attr_lw(sel, 'disabled') === '1')
                  || (typeof __zw_has_attr === 'function' && __zw_has_attr(sel, 'disabled') === '1')) return false;
            } catch (_e) {}
          }
          var wvTag = _realTag(sel, handle);
          var wvTy = '';
          if (wvTag === 'INPUT' || wvTag === 'BUTTON') {
            try { wvTy = handle ? __zw_get_attr_handle(handle, 'type') : __zw_get_attr(sel, 'type'); } catch (_e) { wvTy = ''; }
            wvTy = String(wvTy || '').toLowerCase();
          }
          if (wvTag === 'BUTTON') {
            if (wvTy === 'button' || wvTy === 'reset') return false;
          }
          // R57（FV M1）：非 submittable 元素（fieldset/output/object 等）——
          // willValidate 恒 false（spec §4.10.5.2.2）。
          if (wvTag === 'FIELDSET' || wvTag === 'OUTPUT' || wvTag === 'OBJECT'
              || wvTag === 'LEGEND' || wvTag === 'FIELDSET') {
            return false;
          }
          if (wvTy === 'hidden' || wvTy === 'button' || wvTy === 'reset') return false;
          if (wvTag === 'INPUT' || wvTag === 'TEXTAREA') {
            // readonly 属性存在（任何类型）→ barred——即使 readonly 不适用
            //（date/color/file 等——用例 "readonly attribute does not apply,
            // however we should still bar"）。
            var ro = null;
            try { ro = handle ? __zw_has_attr_handle(handle, 'readonly') : __zw_has_attr(sel, 'readonly'); } catch (_e) {}
            if (ro === '1') return false;
          }
          return true;
        }
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
            // js-dom M4 R51：spec `dom-node-pre-insert` 校验——child 即 parent 自身 / 是 parent 的
            // 祖先 → HierarchyRequestError（WPT Range-mutations "paras[0].appendChild(paras[0])"
            // 非法用例段；旧 shim 不抛真执行 → JS registry 自环 → _zwHCCollectSubtree 无限递归）。
            // 祖先判定：child 沿 _zwNodeParent 反向链（handle）/__zw_parent（sel）上行命中 parent。
            if (child && (child === _makeProxy(sel, handle) || _zwIsAncestorOf(child, sel, handle))) {
              throw _zwDomException('A Node cannot be appended to itself or its descendant.', 'HierarchyRequestError');
            }
            // js-dom M3 R97：无 handle 的 fragment 形态节点（`<template>`.content 视图 /
            // importNode 后的克隆视图，nodeType 11 + childNodes 可读但无 __zwHandle）——
            // lit-html TemplateInstance.u() 返回的 imported fragment 即此形态，随后
            // `marker.parentNode.insertBefore(fragment, …)` 落到 insertBefore 的 no-op 分支
            // （首渲染不落地的最终根因）。spec `dom-node-pre-insert`：fragment 插入 = 子节点
            // 展开插入、fragment 自身不入树。这里对 handle 父做 registry 展开追加：handle 子
            // 走 _recordHandleChild（反链记账），_zwMEl 解析子（innerHTML= 存入 content 的
            // 形态）直接入 registry（childNodes 视图可见，子树查询经 R92 展开工作）。
            if (child && !child.__zwHandle && child.nodeType === 11 && handle) {
              var _r97Fk = [];
              try { _r97Fk = Array.prototype.slice.call(child.childNodes || []); } catch (_e97f) {}
              if (!_handleChildren[handle]) _handleChildren[handle] = [];
              for (var _r97i = 0; _r97i < _r97Fk.length; _r97i++) {
                var _r97c = _r97Fk[_r97i];
                if (_r97c && _r97c.__zwHandle) _recordHandleChild(handle, _r97c);
                else if (_r97c && _r97c.nodeType) _handleChildren[handle].push(_r97c);
              }
              var _r97Added = _r97Fk.slice();
              _mo_notify(sel, handle, { type: 'childList', addedNodes: _r97Added, removedNodes: [], previousSibling: null, nextSibling: null });
              var _r97Pc = _ceParentConnected(sel, handle);
              for (var _r97j = 0; _r97j < _r97Added.length; _r97j++) _ceApplyConn(_r97Added[_r97j], _r97Pc);
              return child;
            }
            // R34xx：重新插入清除移除标记（append 后元素回到文档）。
            if (sel) _zwUnmarkRemoved(sel);
            if (child && child.__zwSelector) _zwUnmarkRemoved(child.__zwSelector);
            // js-dom M4 R81：无 handle 的轻量节点（CDATASection 等 plain object）——append 到
            // handle 父时入 registry（host 无对应 mutation 类型；WPT Node-properties
            // testDiv.textContent 期望 CDATA "1234"+"5678" 计入拼接）。
            // js-dom M4 R84：同步接 parentNode 反链 + 兄弟 getter（R3018 同款）——旧只入
            // registry 不接链，oracle nextNode() 树序遍历在该子断链（parentNode=null →
            // climb 提前终止，NodeIterator/TreeWalker expected-null-but-got-object 根因）。
            if (child && !child.__zwHandle && handle && child.nodeType) {
              if (!_handleChildren[handle]) _handleChildren[handle] = [];
              _handleChildren[handle].push(child);
              try {
                var _r84Parent = _makeProxy(null, handle);
                Object.defineProperty(child, 'parentNode', { get: function () { return _r84Parent; }, configurable: true });
                Object.defineProperty(child, 'parentElement', { get: function () { return _r84Parent; }, configurable: true });
                Object.defineProperty(child, 'previousSibling', { get: function () {
                  var kids = _handleChildren[handle] || [];
                  var i = kids.indexOf(child);
                  return i > 0 ? kids[i - 1] : null;
                }, configurable: true });
                Object.defineProperty(child, 'nextSibling', { get: function () {
                  var kids = _handleChildren[handle] || [];
                  var i = kids.indexOf(child);
                  return i >= 0 && i < kids.length - 1 ? kids[i + 1] : null;
                }, configurable: true });
              } catch (_e84) {}
            }
            if (child && child.__zwHandle) {
              // js-dom M4 R51：spec appendChild 移动语义——child 已有父（sel 父经 __zw_parent /
              // handle 父经 _zwNodeParent 反向链）时，先从旧位移除。host __zw_append_child 内部
              // adopt，但 JS 侧旧父视图（_childNodeList overlay / _handleChildren registry）不知：
              // ① 旧 handle 父 registry 剔除 + 发 removed record（_mo_notify 汇流点同步清 child
              //   反向链并记新链）；② 旧 sel 父发 removed record（overlay 剔除旧位，WPT dom
              //   indexOf 移动用例——不剔则旧父 childNodes 双份）。
              var _r51OldLink = _zwNodeParent[child.__zwHandle];
              if (_r51OldLink) {
                if (_r51OldLink.parentHandle) {
                  _unrecordHandleChild(_r51OldLink.parentHandle, child);
                  _mo_notify(_r51OldLink.parentSel || null, _r51OldLink.parentHandle,
                    { type: 'childList', addedNodes: [], removedNodes: [child] });
                } else if (_r51OldLink.parentSel) {
                  _mo_notify(_r51OldLink.parentSel, null,
                    { type: 'childList', addedNodes: [], removedNodes: [child] });
                }
              }
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
              // js-dom M4 R47：spec appendChild(fragment) 的 childList record——addedNodes 为
              // fragment 的**子节点**（flatten 前快照，即 ceAdded；fragment 自身不入树不出现在
              // record，WPT childList "fragment addition mutations" 期望 [f.firstChild, f.lastChild]）。
              // previousSibling：append 前容器的 lastChild（写入前读——flatten 后再读已是新末尾）；
              // nextSibling 恒 null（append 到尾）。WPT "fragment addition mutations" 断言 previousSibling。
              var _apPrev = null;
              try {
                var _kidsBefore = _isContainerHandle(handle) ? _handleChildNodes(handle) : _childNodeList(sel, handle);
                if (_kidsBefore.length) _apPrev = _kidsBefore[_kidsBefore.length - 1];
              } catch (_e) {}
              _mo_notify(sel, handle, { type: 'childList', addedNodes: ceAdded, removedNodes: [], previousSibling: _apPrev, nextSibling: null });
              // R86：append 即入树——清除移除标记（re-append 移动语义；迭代器重新命中）。
              if (typeof _zwUnmarkRemovedHandle === 'function') {
                for (var ci86 = 0; ci86 < ceAdded.length; ci86++) {
                  var ca86 = ceAdded[ci86];
                  if (ca86 && ca86.__zwHandle) _zwUnmarkRemovedHandle(ca86.__zwHandle);
                  if (ca86 && ca86.__zwSelector && typeof _zwUnmarkRemoved === 'function') _zwUnmarkRemoved(ca86.__zwSelector);
                }
              }
              // R2994 connectedCallback：子树按父连接态传播（父连入 → 子树连入；未观察/非 custom 仅传播）。
              var cePconn = _ceParentConnected(sel, handle);
              for (var ci = 0; ci < ceAdded.length; ci++) _ceApplyConn(ceAdded[ci], cePconn);
            }
            return child;
          };
        }
        if (prop === 'removeChild') {
          return function(child) {
            // R87：注册文本子（textContent= 建的静态包装节点，无 __zwHandle）——旧直接
            // 静默 no-op（WPT NodeIterator-removal 的 `paras[0].parentNode.removeChild(
            // paras[0].firstChild)`：移除不生效 + 迭代器不 retarget）。经注册表注销
            // + 物化 + 通知（与 handle 路径同语义）。注册键是**父 el**（_zwRegisterTextEl
            // 以 _makeProxy(sel,handle) 为键——R87 首版误用 child 查恒 miss）。
            var _r87Self = _makeProxy(sel, handle);
            // R87b：guard 放宽——注册条目可能在父元素先前的 remove/restore 周期中被
            // _zwUnregisterTextSubtree 注销（此时子视图来自物化缓存）。文本子 +
            // 物化缓存含该子时同样走通知路径（WPT NodeIterator-removal 恢复段的
            // 二次 remove——旧 guard 恒 miss → 静默 no-op → 迭代器不 retarget）。
            var _r87Cached = handle && typeof _zwDetachedChildrenOf === 'function'
              ? _zwDetachedChildrenOf(handle) : null;
            var _r87InCache = !!(_r87Cached && _r87Cached.indexOf(child) >= 0);
            if (child && !child.__zwHandle && child.__zwIsText && typeof _zwUnregisterTextEl === 'function'
                && ((_zwTextElsByEl && _zwTextElsByEl.get(_r87Self)) || _r87InCache)) {
              // 物化后剔除被移除子（spec：移除后父的 childNodes 不含 removed——物化的是
              // 移除前视图，直接缓存会把 removed 一并保留）。
              if (typeof _zwMaterializeDetachedChildren === 'function') {
                _zwMaterializeDetachedChildren(_r87Self);
                if (handle && typeof _zwDetachChildFromCache === 'function') _zwDetachChildFromCache(handle, child);
              }
              if (globalThis._zwNotifyIteratorsRemove) {
                try { globalThis._zwNotifyIteratorsRemove(child); } catch (_e87f) {}
              }
              _zwUnregisterTextEl(_r87Self);
              _mo_notify(sel, handle, { type: 'childList', addedNodes: [], removedNodes: [child] });
              return child;
            }
            if (child && child.__zwHandle) {
              // R2994：移除前快照连接态（移除后 host 快照变化，但 _ceConn 为 JS 端追踪，移除调用不影响）。
              // R34xx：注销注册的文本元素（DOM 对照侧几何——removeChild 后 caret 不再命中）。
              // R51c：子树注销（child 内元素 textContent= 建的注册文本随整树摘除——防泄漏）。
              // R86：注销前物化子视图（detached 子树保留其子——WPT NodeIterator-removal）。
              if (typeof _zwMaterializeDetachedChildren === 'function') {
                _zwMaterializeDetachedChildren(child);
              }
              if (typeof _zwUnregisterTextSubtree === 'function') _zwUnregisterTextSubtree(child);
              else if (typeof _zwUnregisterTextEl === 'function') _zwUnregisterTextEl(child);
              // R86：迭代器 retarget 通知——**先于**任何树状态变化（pred/succ 计算读
              // 移除前的兄弟/父链）。
              if (globalThis._zwNotifyIteratorsRemove) {
                try { globalThis._zwNotifyIteratorsRemove(child); } catch (_e86n) {}
              }
              __zw_remove_handle(child.__zwHandle);
              // R86：handle 移除标记（迭代器 order 扫描跳过）。
              if (typeof _zwMarkRemovedHandle === 'function') _zwMarkRemovedHandle(child.__zwHandle);
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
            // js-dom M4 R51：spec `dom-node-pre-insert` 同 appendChild 的自环/祖先校验。
            if (newNode && (newNode === _makeProxy(sel, handle) || _zwIsAncestorOf(newNode, sel, handle))) {
              throw _zwDomException('A Node cannot be inserted before itself or its descendant.', 'HierarchyRequestError');
            }
            // R87：注册文本子恢复（removeChild 注销后 oldParent.insertBefore(node,
            // oldSibling) 重新入树——WPT NodeIterator-removal 恢复段；旧无 handle
            // 直接静默 no-op → firstChild 恒 null、后续子测试 setup 全断）。重注册
            // 同 _zwRegisterTextEl（el 键 = 本 proxy）。
            if (newNode && !newNode.__zwHandle && newNode.__zwIsText
                && typeof _zwRegisterTextEl === 'function' && typeof _zwLocalChildNodes === 'function'
                && !_zwLocalChildNodes(sel, handle)) {
              _zwRegisterTextEl(_makeProxy(sel, handle), handle, sel, String(newNode.data != null ? newNode.data : (newNode.__nv || '')));
            }
            // js-dom M3 R97：无 handle fragment 视图插入（appendChild 同款分支的带位变体）——
            // lit-html commit 的 `marker.parentNode.insertBefore(importedFragment, endNode)`：
            // imported 无 __zwHandle（template.content 派生视图），子节点展开到 refNode 前
            // （refNode 无 selector 时按 registry 位置 splice；null 时 append）。handle 子记
            // 反链；_zwMEl 解析子直接入 registry（同 appendChild 分支）。
            if (newNode && !newNode.__zwHandle && newNode.nodeType === 11 && handle) {
              var _r97Fb = [];
              try { _r97Fb = Array.prototype.slice.call(newNode.childNodes || []); } catch (_e97g) {}
              if (!_handleChildren[handle]) _handleChildren[handle] = [];
              var _r97Kb = _handleChildren[handle];
              var _r97Pos = (refNode && refNode.__zwHandle) ? _r97Kb.indexOf(refNode) : -1;
              for (var _r97k = _r97Fb.length - 1; _r97k >= 0; _r97k--) {
                var _r97cc = _r97Fb[_r97k];
                if (_r97cc && _r97cc.__zwHandle) {
                  if (_r97Pos >= 0) _r97Kb.splice(_r97Pos, 0, _r97cc);
                  else _r97Kb.push(_r97cc);
                  try {
                    if (typeof _zwNodeParent !== 'undefined' && _zwNodeParent) {
                      _zwNodeParent[_r97cc.__zwHandle] = { parentSel: null, parentHandle: handle, nextSibling: null };
                    }
                  } catch (_e97h) {}
                } else if (_r97cc && _r97cc.nodeType) {
                  if (_r97Pos >= 0) _r97Kb.splice(_r97Pos, 0, _r97cc);
                  else _r97Kb.push(_r97cc);
                }
              }
              _mo_notify(sel, handle, { type: 'childList', addedNodes: _r97Fb.slice(), removedNodes: [], previousSibling: null, nextSibling: refNode || null });
              var _r97Pb = _ceParentConnected(sel, handle);
              for (var _r97l = 0; _r97l < _r97Fb.length; _r97l++) _ceApplyConn(_r97Fb[_r97l], _r97Pb);
              return newNode;
            }
            if (newNode && newNode.__zwHandle) {
              // js-dom M3 R97：spec `dom-node-pre-insert`「node === child 时 no-op」
              //（WPT Node-insertBefore "Inserting a node before itself should not move
              // the node"：insertBefore(b, b) 返 b 且子列表不变——旧 R97 registry 分支
              // 会把 b 重复 splice 进 registry）。
              if (newNode === refNode) return newNode;
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
                // js-dom M3 R97：appendChild 路径（1869 行起）有 _recordHandleChild，本分支
                // 旧缺——host 记账后 JS 侧 registry 不含 newNode，容器 childNodes 视图漏子
                //（lit-html render 的 marker 插入即此形态：container.insertBefore(marker, null)
                // 后 container.childNodes.length 仍 0）。对齐 appendChild 补记。
                if (handle) _recordHandleChild(handle, newNode);
              } else if (refNode.__zwSelector) {
                ceAdded = [newNode];
                if (handle) __zw_insert_before_handle(handle, newNode.__zwHandle, refNode.__zwSelector);
                else __zw_insert_before(sel, newNode.__zwHandle, refNode.__zwSelector);
              } else if (handle && refNode.__zwHandle) {
                // js-dom M3 R97：refNode 为 create 句柄节点（comment marker / detached 元素，
                // 无 selector）但父是 handle 容器——host 无对应 wire，JS 侧 registry 插入
                // （appendChild R84 路径的带位变体：按 refNode 在 registry 中的位置 splice）。
                // lit-html 的 renderRoot.insertBefore(marker, firstChild) 精确命中此形态
                // （renderRoot = shadow root 容器，marker/firstChild 均无 selector）。
                ceAdded = [newNode];
                if (!_handleChildren[handle]) _handleChildren[handle] = [];
                var _r97Kids = _handleChildren[handle];
                var _r97At = _r97Kids.indexOf(refNode);
                if (_r97At >= 0) _r97Kids.splice(_r97At, 0, newNode);
                else _r97Kids.push(newNode);
                try {
                  if (typeof _zwNodeParent !== 'undefined' && _zwNodeParent) {
                    _zwNodeParent[newNode.__zwHandle] = { parentSel: null, parentHandle: handle, nextSibling: null };
                  }
                } catch (_e97p) {}
              }
              // refNode 为 create 句柄（无 selector）且父非 handle 时不支持（罕见）。
              // js-dom M4 R47：fragment flatten record——addedNodes 用 ceAdded（fragment 子节点，
              // spec insertBefore(fragment) record 不含 fragment 自身）。nextSibling=refNode
              //（spec record 字段：插入位置的后继）+ previousSibling（refNode 的前兄弟；WPT
              // "Range.insertNode" 断言 previousSibling）。
              var _ibPrev = null;
              try {
                if (refNode && refNode.previousSibling) _ibPrev = refNode.previousSibling;
              } catch (_e) {}
              _mo_notify(sel, handle, { type: 'childList', addedNodes: ceAdded || [newNode], removedNodes: [], previousSibling: _ibPrev, nextSibling: refNode || null });
              // R87：insertBefore 即入树——清除移除标记（恢复段 oldParent.insertBefore；
              // 只 appendChild 清除会使恢复后的节点仍被迭代器当移除跳过——WPT
              // NodeIterator-removal 跨子测试树序分叉根因）。
              if (typeof _zwUnmarkRemovedHandle === 'function') {
                var _ibAdd = ceAdded || [newNode];
                for (var ci87 = 0; ci87 < _ibAdd.length; ci87++) {
                  var ia87 = _ibAdd[ci87];
                  if (ia87 && ia87.__zwHandle) _zwUnmarkRemovedHandle(ia87.__zwHandle);
                  if (ia87 && ia87.__zwSelector && typeof _zwUnmarkRemoved === 'function') _zwUnmarkRemoved(ia87.__zwSelector);
                }
              }
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
            // js-dom M4 R51：spec `dom-node-replace-child` 步骤 2——newChild 是 parent 祖先 →
            // HierarchyRequestError（与 pre-insert 同族校验）。
            if (newChild && _zwIsAncestorOf(newChild, sel, handle)) {
              throw _zwDomException('A Node cannot replace its descendant.', 'HierarchyRequestError');
            }
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
              // js-dom M4 R47：fragment flatten record（同 insertBefore——addedNodes 用 ceAdded）。
              _mo_notify(sel, handle, {
                type: 'childList',
                addedNodes: ceAdded,
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
            // js-dom M4 R47：childList removed record——el.remove() 须发 MutationObserver record
            //（spec `dom-child-remove`；旧缺——WPT surroundContents 期望每 removed 各 1 条）。
            // previous/nextSibling 在移除前捕获（移除后兄弟链断）。
            var _rmPrev = null, _rmNext = null, _rmParent = null;
            try {
              _rmParent = ceSelf.parentNode || null;
              _rmPrev = ceSelf.previousSibling || null;
              _rmNext = ceSelf.nextSibling || null;
            } catch (_e) {}
            if (handle) __zw_remove_handle(handle);
            else { __zw_remove(sel); _zwMarkRemoved(sel); }
            // R86：handle 移除标记（迭代器 order 扫描跳过）+ 迭代器 retarget 通知。
            if (handle && typeof _zwMarkRemovedHandle === 'function') _zwMarkRemovedHandle(handle);
            if (globalThis._zwNotifyIteratorsRemove) {
              try { globalThis._zwNotifyIteratorsRemove(ceSelf); } catch (_e86m) {}
            }
            // R34xx：移除注册的文本元素（DOM 对照侧几何清理）。
            // R86：注销前物化子视图（detached 子树保留其子）。
            if (typeof _zwMaterializeDetachedChildren === 'function') {
              _zwMaterializeDetachedChildren(ceSelf);
            }
            if (typeof _zwUnregisterTextEl === 'function') _zwUnregisterTextEl(ceSelf);
            _ceApplyConn(ceSelf, false);
            // R47：_zwSuppressRemoveRecord——组合操作（surroundContents 等）需要自定义 record
            // 顺序时抑制逐次 notify，由调用方统一按文档序补发。
            if (_rmParent && !globalThis._zwSuppressRemoveRecord) {
              _mo_notify(_rmParent.__zwSelector || null, _rmParent.__zwHandle || null, {
                type: 'childList', addedNodes: [], removedNodes: [ceSelf],
                previousSibling: _rmPrev, nextSibling: _rmNext,
              });
            }
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
        if (prop === 'before' || prop === 'after') {
          return function() {
            // R83：handle 元素（createElement 容器内的子）——_insertAdjacentVariadic 仅支持
            // sel-based（host 无 by-handle 兄弟插入 mutation）；JS 侧经父 _handleChildren
            // 定位自身插入（WPT ChildNode-before/after：parent=createElement('div')）。
            // 字符串/null/undefined 参数经 __zw_create_text 建文本节点（spec：null→'null'）。
            var _baParent = (handle && _zwNodeParent && _zwNodeParent[handle]) ? _zwNodeParent[handle] : null;
            if (!sel && handle && _baParent && _baParent.parentHandle) {
              var _baKids = _handleChildren[_baParent.parentHandle];
              if (_baKids) {
                var _baIdx = -1;
                for (var _bi = 0; _bi < _baKids.length; _bi++) {
                  if (_baKids[_bi] && _baKids[_bi].__zwHandle === handle) { _baIdx = _bi; break; }
                }
                if (_baIdx >= 0) {
                  var _baNew = [];
                  for (var _ai = 0; _ai < arguments.length; _ai++) {
                    var _av = arguments[_ai];
                    if (typeof _av === 'object' && _av && _av.__zwHandle) {
                      _baNew.push(_av);
                    } else {
                      var _atn = (typeof __zw_create_text === 'function') ? __zw_create_text(String(_av)) : '';
                      if (_atn) {
                        _textHandles[_atn] = true;
                        _baNew.push(_wrapHandle(_atn));
                      }
                    }
                  }
                  var _pos = prop === 'before' ? _baIdx : _baIdx + 1;
                  var _ins = _baNew.slice();
                  Array.prototype.splice.apply(_baKids, [_pos, 0].concat(_ins));
                  // 反链 + childList record（父 registry 变化）。
                  for (var _ri2 = 0; _ri2 < _ins.length; _ri2++) {
                    var _rn2 = _ins[_ri2];
                    if (_rn2 && _rn2.__zwHandle && _zwNodeParent) {
                      _zwNodeParent[_rn2.__zwHandle] = { parentHandle: _baParent.parentHandle, nextSibling: null };
                    }
                  }
                  _mo_notify(null, _baParent.parentHandle, { type: 'childList', addedNodes: _ins, removedNodes: [] });
                  return undefined;
                }
              }
            }
            _insertAdjacentVariadic(sel, prop === 'before' ? 'beforebegin' : 'afterend', arguments, prop === 'after');
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
        // R57（FV M3）：form 提交共享路径（requestSubmit + submit 按钮 click 默认动作共用）——
        // spec §4.10.5.4 的 submit 算法：novalidate 属性 / submitter 的 formnovalidate 跳过
        // interactive validation；invalid 时首个控件派发 invalid 事件 + 中止提交（headless
        // 聚焦 no-op）；valid 派发 cancelable submit（SubmitEvent，含 submitter）。
        // 重入守卫：submit/invalid 事件处理中的重入 requestSubmit/click 直接返回（spec 的
        // "submit event is firing" 标志语义——form-requestsubmit 的 reentrant 用例——
        // requestSubmit()+requestSubmit() 只派发一次）。flag 声明在 part05（IIFE 作用域，
        // 初始化一次；此处若 var 声明会被 get trap 每属性访问重置）。
        function _zwRunFormSubmit(fKey, fSel, fHandle, submitter) {
          if (_zwSubmitBusy) return;
          _zwSubmitBusy = true;
          try {
            // spec：form 未连入文档（detached createElement / removed 子树）→ 不提交
            //（form-requestsubmit 的 disconnected 用例——submit 事件不派发）。
            var _conn = true;
            try {
              if (globalThis.document && typeof globalThis.document.contains === 'function') {
                _conn = !!globalThis.document.contains(_makeProxy(fSel, fHandle));
              }
            } catch (_e) {}
            if (!_conn) return;
            var _doValidate = true;
            try {
              var _nv = fHandle ? __zw_has_attr_handle(fHandle, 'novalidate') : __zw_has_attr(fSel, 'novalidate');
              if (_nv === '1') _doValidate = false;
            } catch (_e) {}
            if (_doValidate && submitter) {
              try {
                var _fnv = (typeof submitter.getAttribute === 'function') ? submitter.getAttribute('formnovalidate') : null;
                if (_fnv != null) _doValidate = false;
              } catch (_e) {}
            }
            if (_doValidate) {
              var _firstInv = null;
              try {
                var _fcs2 = _formControls(fSel);
                for (var _ci2 = 0; _fcs2 && _ci2 < _fcs2.length; _ci2++) {
                  var _c2 = _fcs2[_ci2];
                  try {
                    if (_c2.validity && !_c2.validity.valid) {
                      if (typeof _c2.dispatchEvent === 'function') {
                        try { _c2.dispatchEvent(new Event('invalid', { cancelable: true, bubbles: false })); } catch (_e2) {}
                      }
                      if (_firstInv == null) _firstInv = _c2;
                    }
                  } catch (_e3) {}
                }
              } catch (_e4) {}
              if (_firstInv != null) return; // 中止提交（interactive validation 失败）
            }
            // dispatch submit SubmitEvent（cancelable，含 submitter）；headless 无导航（documented）。
            var _sev;
            try { _sev = new SubmitEvent('submit', { bubbles: true, cancelable: true, submitter: submitter || null }); }
            catch (_e) { _sev = new Event('submit', { bubbles: true, cancelable: true }); }
            _dispatchWithBubble(fKey, fSel, fHandle, _sev);
          } finally {
            _zwSubmitBusy = false;
          }
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
              // R57（FV M2/M3）：submitter 校验——非 submit button（BUTTON 默认/
              // type=submit、INPUT type=submit/image）→ TypeError；非本 form 的
              // submitter → NotFoundError（spec §4.10.5.5 requestSubmit(submitter)）。
              if (submitter != null && submitter !== undefined) {
                var _isSubBtn = false;
                try {
                  var _st = String(submitter.tagName || '').toUpperCase();
                  var _sty = '';
                  try { _sty = String(submitter.type || '').toLowerCase(); } catch (_e2) {}
                  _isSubBtn = (_st === 'BUTTON' && (_sty === 'submit' || _sty === ''))
                    || (_st === 'INPUT' && (_sty === 'submit' || _sty === 'image'));
                } catch (_e) {}
                if (!_isSubBtn) {
                  var _te = new TypeError('The provided element is not a submit button.');
                  _te.name = 'TypeError';
                  throw _te;
                }
                // form 归属：submitter 的 form owner ≠ 当前 form → NotFoundError（spec
                // §4.10.5.5——detached submitter（form owner null）同样 NotFoundError——
                // form-requestsubmit 用例；sel/handle 两身份路径比较）。
                try {
                  var _owner = submitter.form;
                  var _owned = false;
                  try {
                    if (_owner) {
                      var _os = _owner.__zwSelector;
                      var _oh = _owner.__zwHandle;
                      if (sel && _os && _os === sel) _owned = true;
                      else if (handle && _oh && _oh === handle) _owned = true;
                    }
                  } catch (_e4) {}
                  if (!_owned) {
                    // 真实 DOMException（assert_throws_dom 须 instanceof DOMException + code=8）。
                    _throwDom('NotFoundError', 'The provided element is not owned by this form.');
                  }
                } catch (_e3) {
                  if (_e3 && _e3.name === 'NotFoundError') throw _e3;
                }
              }
              // R57（FV M2/M3）：共享提交路径（interactive validation + submit 派发 + 重入守卫）。
              _zwRunFormSubmit(key, sel, handle, submitter);
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
            // R34xx：注册的纯文本元素 → 本地 0 基几何（测试归一化绝对位置）。
            if (typeof _zwTextElBoundingRect === 'function') {
              var _zwR = _zwTextElBoundingRect(sel, handle);
              if (_zwR) return _zwR;
            }
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
        // js-dom M3 R96：查表改 own-property 判定——裸 `_REFLECTED_UINT[prop]` 对 Object.prototype 继承名
        //（hasOwnProperty/valueOf/toLocaleString/isPrototypeOf/propertyIsEnumerable/constructor）返回 truthy
        // 函数 → `if (_ruEntry)` 误入 → `parseInt(_ruEntry.a=undefined)`=NaN → `return undefined` 提前吞掉
        // 这些名字，R93 原型链回落不可达（lit `this.enableUpdating.call(this)` 前的 hasOwnProperty 探测、
        // 任何 `el.valueOf` 读全部返 undefined）。
        var _ruEntry = Object.prototype.hasOwnProperty.call(_REFLECTED_UINT, prop) ? _REFLECTED_UINT[prop] : null;
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
        // 等，real browser expando 语义）。仅 hasOwnProperty 命中才返（避免原型链污染）；未命中 fall through。
        var _exStore = _expando[key];
        if (_exStore && Object.prototype.hasOwnProperty.call(_exStore, prop)) return _exStore[prop];
        // js-dom M4 R80：未知属性回落**原型链**（getPrototypeOf trap 决定的链——target {} 的真实原型
        // 是 Object.prototype，Reflect.get 不可见 HTML*Element 链，故手动沿链查找）——旧恒返 undefined
        // 使 Node 接口常量（element.ELEMENT_NODE 等）不可见（WPT Document-createElementNS
        // `assert_equals(element.nodeType, element.ELEMENT_NODE)` 596F 簇根因之一）。
        // js-dom M3 R93：回落放宽到**全部未命中属性**（旧仅 SCREAMING_SNAKE 常量）——custom
        // element 的原型方法（`MyEl.prototype.bump` 等，lit/stencil 组件形态）经 getPrototypeOf
        // trap 的 CE registry 分支（R90）派发到用户 prototype，但 get trap 对方法名恒返
        // undefined 使方法不可达（WPT e2e：bridge ctor 后 `el.bump is not a function`）。
        // 沿链只取 own 命中，限 8 层。accessor getter 以**元素 proxy** 为 this 求值
        //（spec 原型 getter 语义；直接 `_pchain[prop]` 会让 this 落在 prototype 对象上，
        // 读 `this._count` 得 undefined——WC e2e 组 7 `doubled` NaN 实证）；data property
        //（方法引用等）直接取值，this 在调用期自然绑定到 proxy。
        if (typeof prop === 'string' && prop.length > 0) {
          var _pchain = Object.getPrototypeOf(_makeProxy(sel, handle));
          var _pguard = 0;
          while (_pchain && _pguard < 8) {
            var _pdesc = Object.getOwnPropertyDescriptor(_pchain, prop);
            if (_pdesc) {
              if (_pdesc.get) return _pdesc.get.call(_makeProxy(sel, handle));
              return _pdesc.value;
            }
            _pchain = Object.getPrototypeOf(_pchain);
            _pguard++;
          }
        }
        return undefined;
      },
      set: function(_t, prop, value) {
        var p = String(prop);
        var moAttr = null;
        // js-dom M4 R45：MutationObserver attributeOldValue——IDL 反射 setter（el.id=/className=/title= 等）的
        // old 值须在**写入前**捕获（写后读即新值）。旧实现 part05 末尾 notify 不带 oldValue（恒 null，WPT
        // MutationObserver-attributes "oldValue didn't match" 全族 fail）。此处按 IDL 名预判目标内容属性名，
        // 有 observer 请求 old 时读当前值暂存，末尾 notify 携带。非反射属性（expando 等）moAttr=null 不触发。
        var _moIdVal = _mo_id(handle, sel);
        var _moOldVal;
        {
          var _attrOf = null;
          if (p === 'id') _attrOf = 'id';
          else if (p === 'className') _attrOf = 'class';
          else if (p === 'title' || p === 'lang' || p === 'type') _attrOf = p;
          if (_attrOf && _moIdVal != null && _mo_any_wants_attr_old(_moIdVal, _attrOf)) {
            _moOldVal = _mo_read_attr(sel, handle, _attrOf);
          }
        }
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
          // R81：PI 节点的 textContent= 直接写 data（spec CharacterData——不建文本子视图、不发
          // childList；WPT "For a ProcessingInstruction, textContent should set the data"）。
          if (p === 'textContent' && handle && _piHandles[handle]) {
            _piHandles[handle].data = (value === null || value === undefined) ? '' : String(value);
            return true;
          }
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
            // https://html.spec.whatwg.org/multipage/dynamic-markup-insertion.html#dom-innerhtml
            // Host mutations are applied asynchronously. Keep a parsed local child
            // view for created elements so childNodes/firstChild/lastChild remain
            // observable within the script that assigned innerHTML.
            if (handle) _handleChildren[handle] = _ihAdded;
            // js-dom M4 R56：sel 路径替换后丢弃 childNodes 基底缓存条目。R55 的 identity
            // 稳定副作用 + 本行上方 _ihRemoved 读（把旧基底入缓存）→ 同回合内 `el.childNodes`
            // 缓存命中旧基底，overlay 的 pending-removed 剔除 identity 命中清空列表；而 added
            //（_zwFragmentAdded 的 _zwMEl 解析代理，无 __zwHandle）不并入 → length=0
            //（security/xss/innerHTML-sanitization 全平台 FAIL）。删条目后回退 host 读
            //（旧快照 + 每次新包装不命中剔除，R55 前语义），flush 重注册后自然换代。
            if (!handle && typeof _zwChildBaseCache !== 'undefined') _zwChildBaseCache.delete(sel);
            // R34xx：纯文本 innerHTML → 本地文本节点注册（selection-rects 的
            // el.childNodes[0] 文本节点——created handle 元素无 sel，host 不可查）。
            // _makeProxy 经 _proxyCache 返同一 proxy 对象（parentNode===el 成立）。
            if (typeof _zwRegisterTextEl === 'function' && _ihVal.indexOf('<') < 0) {
              _zwRegisterTextEl(_makeProxy(sel, handle), handle, sel, _ihVal);
            } else if (typeof _zwUnregisterTextEl === 'function' && typeof _makeProxy === 'function') {
              _zwUnregisterTextEl(_makeProxy(sel, handle));
            }
            _mo_notify(sel, handle, { type: 'childList', addedNodes: _ihAdded, removedNodes: _ihRemoved });
          } else {
            // R3027：textContent 变更 → emit characterData 记录（target=元素，pragmatic——文本节点无 selector
            // 不能直接作 target；observe(el,{characterData,subtree}) + 后代 textContent 经 ancestor 冒泡亦覆盖）。
            // R3028：characterDataOldValue——有 observer 请求时 mutate 前捕获 old 文本（latest-wins，反映同批前序 textContent=）。
            var _charMoId = _mo_id(handle, sel);
            var _charMoOld = _mo_any_wants_char_old(_charMoId) ? _mo_read_text(sel, handle) : null;
            // spec `LegacyNullToEmptyString`：null/undefined（optional DOMString 缺省）→ 空串（清子）。
            // R81 spec 纠正：WPT Node-textContent "set to undefined" 期望 ""（旧 R3184 记 undefined
            // → 'undefined' 是错误语义——WebIDL nullable DOMString? 的 undefined 映射空串）。
            var _tcVal = (value === null || value === undefined) ? '' : String(value);
            if (_realTag(sel, handle) === 'OUTPUT') {
              // https://html.spec.whatwg.org/multipage/form-elements.html#the-output-element
              // Replacing descendants updates defaultValue. In default mode value follows it;
              // in value mode `_outputValue` keeps the live value.
              _outputDefault[key] = _tcVal;
            }
            // R49：同值 no-op——textContent 与当前文本相同不重写不发 record（spec set 同值
            // 不产生 mutation；WPT childList "textContent no mutation" 期望仅 id 改名 1 条）。
            // **写前**读当前值（写后 latest-wins 已含新值，恒同值——takeRecords 回归根因）。
            // 本地注册文本优先（firstChild.data= 编辑只改本地 __nv + SetChildText pending，lw 元素级
            // 读不到 → 误判异值多发 record）。无本地注册才读 host lw。
            // js-dom M4 R81：handle 元素的同值判定改用 **getter 同源值**（融合 childNodes 拼接）——
            // 旧只查本地注册/host 文本，appendChild 的子（'\tDEF\t'）不在其中 → `textContent=null`
            // 误判同值（''==''）跳过写入与清子 → 子残留（WPT "set to null" 簇根因）。
            var _tcCur = '';
            var _tcHasLocal = false;
            if (handle) {
              _tcCur = String(_makeProxy(sel, handle).textContent != null ? _makeProxy(sel, handle).textContent : '');
              _tcHasLocal = true; // 融合视图已是权威（同值判定用）
            } else if (typeof _zwLocalChildNodes === 'function') {
              var _tcLocal = _zwLocalChildNodes(sel, handle);
              if (_tcLocal && _tcLocal[0]) {
                _tcHasLocal = true;
                _tcCur = String(_tcLocal[0].data != null ? _tcLocal[0].data : '');
              }
            }
            if (!_tcHasLocal) {
              _tcCur = (handle ? __zw_get_text_handle(handle) : (typeof __zw_get_text_lw === 'function' ? __zw_get_text_lw(sel) : __zw_get_text(sel))) || '';
            }
            var _tcSame = _tcVal === _tcCur;
            // R81：**清子不受同值短路**——spec textContent setter 恒「替换全部子」：同值（如旧子是
            // 空文本节点、新值 ''——融合 getter 读到的当前值已是 ''）也须清空 registry 子 + 反链
            //（WPT "Element with empty text node as child set to null/undefined"：旧子残留
            // childNodes 非空 + parentNode 仍指父）。同值不写 host、不发 record（R49 语义保持）。
            if (_tcSame && handle && _handleChildren[handle] && _handleChildren[handle].length) {
              for (var _tcs = 0; _tcs < _handleChildren[handle].length; _tcs++) {
                var _tcsn = _handleChildren[handle][_tcs];
                if (_tcsn && _tcsn.__zwHandle && _zwNodeParent) delete _zwNodeParent[_tcsn.__zwHandle];
              }
              _handleChildren[handle] = [];
            }
            if (!_tcSame) {
              if (handle) __zw_set_text_handle(handle, _tcVal);
              else __zw_set_text(sel, _tcVal);
              // js-dom M4 R81：textContent= 替换全部子——**清空 handle 元素的 registry 子**（R2927
              // _handleChildren 记录的 appendChild 子在融合 childNodes 视图仍可见 → `el.textContent=null`
              // 后 childNodes 非空、firstChild 残留，WPT Node-textContent "set to null" 簇）。spec
              // string replace algorithm：移除全部旧子（含元素子），再插入单文本节点。
              // 同步记录 JS 侧写入值（`_zwTextWritten`）：host 变更重放（query_text_from_mutations
              // 仍见旧 AppendChild 文本子）不得覆盖此后 getter——空值清子后 firstChild/textContent
              // 须立即反映 null/''。
              if (handle) {
                // R81：被移除子的 parentNode 置 null（spec string replace：旧子脱离树——WPT
                // "Preexisting Text should have been removed" 断言 text.parentNode === null）。
                // 删除子 handle 的 `_zwNodeParent` 反链（_parentNodeFor 的 handle 分支查不到 → null）。
                if (_handleChildren[handle]) {
                  for (var _tcr = 0; _tcr < _handleChildren[handle].length; _tcr++) {
                    var _tcrn = _handleChildren[handle][_tcr];
                    if (_tcrn && _tcrn.__zwHandle && _zwNodeParent) delete _zwNodeParent[_tcrn.__zwHandle];
                  }
                  _handleChildren[handle] = [];
                }
                if (typeof _zwTextWritten === 'undefined') { globalThis._zwTextWritten = {}; }
                _zwTextWritten[handle] = _tcVal;
              }
              // js-dom M4 R49：本地文本子视图——textContent= 替换全部子为单文本节点，host 延迟 apply
              // 期间 firstChild/childNodes 须立即可见（WPT takeRecords `n.textContent='old data';
              // n.firstChild.data='new data'`）。复用 canvas 的 _zwRegisterTextEl 注册表（_zwLocalChildNodes
              // 消费）；host 侧文本旧子与新文本等价（纯文本替换），不移除 host 旧子。
              // spec `dom-node-textcontent` setter：替换全部子 → **childList record**（removed=旧子快照，
              // added=[新文本节点]；R3027 的 characterData-only 记录不完整——characterData record 仅由
              // 文本节点自身编辑发）。
              var _tcRemoved = _childNodeList(sel, handle);
              // R81：空值（null/''）不注册空文本节点——spec string replace 对空串「移除全部旧子且不插入
              // 新子」→ firstChild 应为 null（WPT "Element with empty text node as child set to null"
              // 期望 el.firstChild === null；旧注册空文本节点残留 __n28）。
              // R81：textContent **不解析 markup**（区别 innerHTML——spec dom-node-textcontent 把值整体
              // 作为单文本节点的 data；`textContent='<b>xyz</b>'` 是字面文本，WPT "set to <b>xyz</b>"
              // 期望 childNodes.length===1 + instanceof Text + data 原样）。移除旧 `<` 守卫。
              if (typeof _zwRegisterTextEl === 'function' && _tcVal !== '') {
                // R81：容器 handle（fragment/shadow——childNodes 走 `_handleChildNodes` registry，
                // 不查 `_zwLocalChildNodes`）**只入 registry**（本地注册 + registry 会在融合视图
                // 双份 → textContent "4242"）；普通元素走本地注册表。
                if (handle && _fragmentHandles[handle]) {
                  _zwRegisterTextEl(_makeProxy(sel, handle), handle, sel, _tcVal);
                  var _tcReg = _zwLocalChildNodes(sel, handle);
                  _handleChildren[handle] = _tcReg && _tcReg[0] ? [_tcReg[0]] : [];
                  if (_tcReg && _tcReg[0] && _zwNodeParent) {
                    _zwNodeParent[_tcReg[0].__zwHandle || ''] = { parentHandle: handle, nextSibling: null };
                  }
                } else {
                  _zwRegisterTextEl(_makeProxy(sel, handle), handle, sel, _tcVal);
                }
              } else if (typeof _zwUnregisterTextEl === 'function') {
                _zwUnregisterTextEl(_makeProxy(sel, handle));
              }
              var _tcAdded = [];
              if (_tcVal !== '') {
                var _tcAddedList = (handle && _fragmentHandles[handle])
                  ? (_handleChildren[handle] || [])[0]
                  : (typeof _zwLocalChildNodes === 'function' ? _zwLocalChildNodes(sel, handle) : [])[0];
                if (_tcAddedList) _tcAdded = [_tcAddedList];
              }
              _mo_notify(sel, handle, { type: 'childList', addedNodes: _tcAdded, removedNodes: _tcRemoved, previousSibling: null, nextSibling: null });
            }
            void _charMoOld;
            // R49 修正：textContent= 只发 childList（spec `dom-node-textcontent` 替换子树）；
            // characterData record 仅由**文本节点自身编辑**（data=/appendData 族）发——R3027 的
            // textContent characterData record 移除（WPT takeRecords：`textContent='old data'` 期望
            // 1 条 childList；后续 `firstChild.data='new data'` 由 R48 _write 发 characterData
            // oldValue='old data' target=文本节点）。
            void _charMoOld;
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
          // A node can be appended before its ID is assigned.  Keep the pending
          // insertion index in sync so getElementById() observes that node in
          // the same script turn, before the renderer publishes its next DOM
          // snapshot.
          // https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
          if (typeof _zwPAIdRemove === 'function') _zwPAIdRemove(proxy);
          if (handle) __zw_set_attr_handle(handle, 'id', idv);
          else __zw_set_attr(sel, 'id', idv);
          if (typeof _zwPAIdAdd === 'function') _zwPAIdAdd(proxy);
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
            _inputValues[key] = String(value); _inputValuesSet[key] = true;
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
              _inputValues[key] = vsS; _inputValuesSet[key] = true;
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
              _inputValues[key] = vadStr; _inputValuesSet[key] = true;
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
