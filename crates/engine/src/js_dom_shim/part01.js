(function() {
  var _listenerStore = {};
  // R2933 element 级 IDL on-event handler 存储（per-element-key → { eventType: fn }）。on* setter 把 fn
  // 同时记此 + 注册进 _listenerStore[key]（使 dispatchEvent 触发）；getter 返此存储 fn（或 null）。
  var _onHandlers = {};
  // P1b S2 incr3：元素 proxy 缓存——同一 (sel, handle) 复用同一 Proxy 实例，使
  // `querySelector('#t') === querySelector('#t')` 为真（node === identity，v8::External
  // 精修目标，但纯 JS Proxy 缓存即可达成，无需 rusty_v8 对象绑定）。proxy 无状态（仅委托
  // host 回调），缓存安全；key 复用 `_elKey`（@handle / sel），与 _listenerStore 同生命周期。
  var _proxyCache = {};
  // P1a form input：per-element-key value 缓存（`.value` 属性）。lazy-init 自 value 属性；
  // `.value` set 更新缓存 + 记 value 属性 mutation（供 render）。跨 execute 存活（typing 多键），
  // 导航（URL 变化）经 `__zw_reset_form_state` 清空防跨页 stale value。
  var _inputValues = {};
  // R2996 input.defaultValue 独立追踪（spec：`.value=` 改 dirty 当前态，**不**改 defaultValue=初始 value 属性）。
  // shim 的 `.value=` 仍写 value 属性供 render（paint_input_value 读属性），故属性被「污染」；为使 defaultValue
  // 不被污染，单独追踪「真默认值」：首次 `.value=` 前捕获当前 value 属性（=真默认），setAttribute('value')/
  // defaultValue=/removeAttribute('value') 重同步（清 dirty，getter 回落属性）。_inputDefault[key]=捕获的默认值；
  // _inputDefaultDirty[key]=true 表属性已dirty、defaultValue 须读捕获值。同 _inputValues 经 reset 清空。
  var _inputDefault = {};
  var _inputDefaultDirty = {};
  // R2998 布尔默认态独立追踪（checked→defaultChecked, selected→defaultSelected）。spec：`.checked=`/`.selected=`
  // 改 dirty 当前态、**不**改 default*=初始属性存在性。shim 的 `.checked=`/`.selected=` 仍写属性供 render/form
  // 序列化（R2997 .checked getter latest-wins 读属性），故属性被「污染」；为使 default* 不被污染，单独追踪「真
  // 默认存在性」：首次 `.checked=`/`.selected=` 前捕获当前属性存在性，setAttribute/removeAttribute/default*=
  // 重同步（清 dirty，getter 回落属性 latest-wins）。键 `key+':'+attr` 避 checked/selected 冲突。经 reset 清空。
  var _boolDefault = {};
  var _boolDefaultDirty = {};
  // P1a classList：per-element-key class 缓存（`className` / `classList`）。同 _inputValues 动机——
  // classList.add/remove/toggle 旧实现每次读 stale snapshot 算新 class 再 SetAttr 整体替换，
  // 同脚本内连续 add 末次覆盖前次（`add('a');add('b')` 丢 'a'）。缓存累积全量，末次 SetAttr 携带
  // 正确值；className set 同步更新缓存保证一致。导航经 `__zw_reset_form_state` 清空。
  var _classCache = {};
  // Constraint Validation（R2825）：per-element-key 自定义校验消息（setCustomValidity 设置）。
  // 空串/未设=valid；非空=customError + validity.valid=false + validationMessage=msg。原生约束
  // （required/pattern/type 等）headless 不强制（permissive valid）。同 _inputValues/_classCache 经
  // `__zw_reset_form_state` 清空防跨页 stale。
  var _customValidity = {};
  // 用户编辑标记：minlength/maxlength 的 tooShort/tooLong 只适用于用户提供的值，脚本 `.value=`
  // 不触发。宿主 user-action helper 在提交 SetText 后标记；reset/navigation 清空。
  var _userEdited = {};
  // HTMLInputElement.files 空 FileList（R2830）：headless 无真文件 → 共享空 FileList（length 0 +
  // item→null + 可迭代）。上传表单读 `input.files.length` 不抛（无文件 → 0，跳过上传逻辑）。
  var _emptyFileList = {
    length: 0,
    item: function (_i) { return null; },
    [Symbol.iterator]: function* () {},
  };
  // HTMLInputElement.indeterminate（R2831）：JS-only IDL 布尔（**非 reflected attr**——无 indeterminate
  // 内容属性，纯 JS 状态）。checkbox「全选」tri-state UI 高频（父 checkbox 半选态）。per-element-key，
  // 默认 false。同 _inputValues/_classCache 经 `__zw_reset_form_state` 清空。
  var _indeterminate = {};
  // text-control 选区（selectionStart/End/Direction + setSelectionRange/select，R2844）：per-element-key 选区
  // 状态 { start, end, direction }。仅 text control（textarea + input text/search/url/tel/password）有真实选区；
  // 默认（未设）= {0, 0, 'forward'}（Chromium 150 oracle 锚定——未聚焦/未设的 text control 选区折叠在 0，非值末）。
  // headless 无真 caret/选择渲染，故 selection 为纯 JS 跟踪（供文本编辑器 / 自动选择 / Range 算法读状态）。
  // 同 _inputValues/_classCache 经 `__zw_reset_form_state` 清空。
  var _textSelection = {};
  // HTMLOutputElement（R2846）：value 独立于 textContent（<output> 按 children 渲染非 value；spec：设 .value
  // 不触碰 DOM text）。_outputDefault = 默认值（= 初始 textContent，lazy 捕获一次跨 value 变更稳定）；
  // _outputValue = dirty 后的当前值（key 存在即 dirty）。同 _inputValues 经 `__zw_reset_form_state` 清空。
  var _outputDefault = {};
  var _outputValue = {};
  // FR-009：资源元素最终状态（key → {url,outcome,width,height,error}）。
  // host 在 fetch/decode settle 后提交；导航与其它 page-local 状态一并清空。
  var _resourceStates = {};
  // R3049：textarea defaultValue 追踪（闭合 R3048 限制①）。textarea.value ↔ live textContent，无独立初值缓存
  //（区别 INPUT value 属性 / OUTPUT _outputDefault）→ form.reset 无法还原 textarea。本 map 惰性捕获 textarea 初值
  //（getter 首读 / value setter 首写前），供 defaultValue getter + form.reset 还原。同 _outputDefault 经 reset 清空。
  var _textareaDefault = {};
  // R3042：expando 属性 per-element-key 存储。set trap generic fallthrough 对**非原始值**（function/object/null/
  // undefined 等——永不可能为合法内容属性值）旧写垃圾属性（`__zw_set_attr(sel,'fn','[object Object]')`）且 get 读不回
  //（undefined）。real browser：expando 属性存于 JS 对象非内容属性。本 map 存非原始值 expando，get trap 读回。
  // 仅非原始值（real attr setter 永不收 function/object → 零回归风险，string/number/boolean 保持 generic fallthrough）。
  // 限制：无 deleteProperty trap → `delete el.expando` 不清此 map（罕见，documented）。导航经 __zw_reset_form_state 清空。
  var _expando = {};
  // R3067：Web Animations API per-element 动画注册表。elKey → [Animation, ...]（创建序）。_makeAnimation 入注册表，
  // Element.getAnimations() / Document.getAnimations() 读。spec：返「current/in effect」动画——cancelled（playState='idle'）
  // 排除；finished 仍返（headless 瞬间完成，finished 动画仍可查询/commitStyles）。导航经 __zw_reset_form_state 清空（per-page）。
  var _elementAnimations = {};
  // R3068：Pointer Capture API per-element 捕获集。elKey → { pointerId: true }（set 形态）。setPointerCapture 加、
  // releasePointerCapture 删、hasPointerCapture 查。headless 无真指针路由（事件不重定向到捕获元素），但 API 表面 +
  // hasPointerCapture 状态查询对指针/拖拽库（interact.js / sortablejs pointer mode）feature-detect 必需。导航经
  // __zw_reset_form_state 清空（per-page）。permissive：不校验 pointerId 是否 active（headless 无 active 追踪，spec
  // NotFoundError defer；releasePointerCapture 未捕获不抛 InvalidStateError，lenient 防破库）。
  var _pointerCapture = {};
  // R3071：Popover API top-layer 成员集（elKey → true）。showPopover 加入、hidePopover 移除；成员即「showing」态。
  // headless 无真 top-layer paint / 渲染层级 / :popover-open 伪类（rendering 流域 defer），本集仅追踪 JS-observable
  // 状态（showPopover→态 open / 派发 beforetoggle+toggle / hidePopover→态 closed）。UI 库（tooltip/menu/modal）feature-detect
  // + 调 showPopover/hidePopover/togglePopover + 监听 toggle 事件不中断。导航经 __zw_reset_form_state 清空（per-page）。
  // https://html.spec.whatwg.org/multipage/popover.html
  var _zwTopLayer = {};
  // R3073：popoverTargetElement 编程式目标（per-element-key → 目标元素 proxy）。优先于 popovertarget 内容属性
  //（spec：popoverTargetElement setter 设的元素即触发目标，不改内容属性）。null → 清除（回落内容属性）。导航经
  // __zw_reset_form_state 清空（per-page）。
  var _popoverTargetEl = {};
  // R3077：HTMLCanvasElement proxy 的 2d 上下文缓存（elKey → ctx2d proxy）。getContext('2d') 首次调创建 +
  // 缓存（后续返同一 ctx，spec 一致）。闭合 canvas DOM 集成缺口（旧仅 standalone _zwMakeCanvas 有 getContext，
  // DOM 元素 proxy 缺 → `document.getElementById('c').getContext` 抛 TypeError，~29 canvas WPT 用例不可执行）。
  // 导航经 __zw_reset_form_state 清空（per-page）。
  var _zwCanvasCtx = {};
  // R3290：HTMLDialogElement 模态态追踪（per-element-key → true 即经 showModal 开为模态）。
  // `_zwTopLayer[key]`（R3071 popover 同集）复用为 top-layer 成员集——dialog.showModal() 与 popover 共享 top-layer
  // 概念。close() 据本集判是否需移 top-layer（非模态 show() 不入 top-layer，close 仅清 open 属性）。导航经
  // __zw_reset_form_state 清空（per-page）。
  // https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element
  var _zwDialogModal = {};
  // R3071：Popover 事件派发中用。构造 ToggleEvent 数据对象（type + newState/oldState + bubbles/cancelable/composed）。
  // spec ToggleEvent extends Event，直接属性 newState/oldState（非 CustomEvent.detail）。headless 同步派发（spec 队列
  // task，近似——documented 限制）；beforetoggle cancelable（可 preventDefault 阻止显隐）+ 非 bubble；toggle 非 cancelable。
  function _makeToggleEvent(type, oldState, newState, cancelable) {
    var ev = _makeEvent(type, { bubbles: false, cancelable: !!cancelable });
    ev.oldState = oldState;
    ev.newState = newState;
    return ev;
  }
  // R3071：读 popover 内容属性的枚举值。spec enumerated attribute：missing value default = no popover（无属性 → null）；
  // invalid value default = manual（属性存在但空串/无效/"manual" → "manual"）；"auto"(ci) → "auto"。__zw_get_attr 对 absent
  // 与空串属性均返 ''（不可区分），故用 presence-based `__zw_has_attr`（'1'=存在）判有无属性。**用 latest-wins 变体（`_lw`）**
  // 反映同 execute 内 pending set/remove（sync set→get round-trip——popover setter 经 __zw_set_attr/__zw_remove_attr 异步入队，
  // 纯快照读 stale）。handle 路径无 `_lw` 变体，回落纯快照（handle 元素 popover setter 罕见）。供 popover getter + showPopover 校验共用。
  function _zwReadPopover(sel, handle) {
    var present, raw;
    if (handle) {
      present = typeof __zw_has_attr_handle === 'function' && __zw_has_attr_handle(handle, 'popover') === '1';
      raw = __zw_get_attr_handle(handle, 'popover');
    } else {
      present = typeof __zw_has_attr_lw === 'function'
        ? (__zw_has_attr_lw(sel, 'popover') === '1')
        : (typeof __zw_has_attr === 'function' && __zw_has_attr(sel, 'popover') === '1');
      raw = typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, 'popover') : __zw_get_attr(sel, 'popover');
    }
    if (!present) return null; // 无属性 → no popover state
    return String(raw).toLowerCase() === 'auto' ? 'auto' : 'manual';
  }
  function _zwIsConnected(sel, handle) {
    if (sel) {
      if (typeof __zw_contains === 'function') {
        try { return __zw_contains('html', sel) === '1'; } catch (_e) { return true; }
      }
      return true;
    }
    return _elConnected(_makeProxy(sel, handle));
  }
  // R3217：ns → 限定名重构，供 get/has/removeAttributeNS 查找 setAttributeNS 存的 'prefix:local' 限定名属性。
  // spec 按 ns+localName 匹配；本 shim 按限定名字符串存（host 无 ns 解析），故用 ns→常规 prefix 映射重构。
  // xlink/xml/xmlns 三常规命名空间（SVG/MathML 高频）；null/空/未知 ns → 裸 local（无命名空间属性）。
  function _nsQualName(ns, localName) {
    var s = String(ns == null ? '' : ns);
    var p = s === 'http://www.w3.org/1999/xlink' ? 'xlink'
      : s === 'http://www.w3.org/XML/1998/namespace' ? 'xml'
      : s === 'http://www.w3.org/2000/xmlns/' ? 'xmlns'
      : null;
    return p ? (p + ':' + String(localName)) : String(localName);
  }
  // R3071：showPopover 状态机。非 popover（无 popover 属性）→ InvalidStateError；已 showing → InvalidStateError；
  // 派发 beforetoggle(cancelable, closed→open)，preventDefault → 中止不显；加 top-layer；派发 toggle(closed→open)。
  // headless 无真渲染层级 / paint（rendering 流域 defer）——仅 JS-observable 状态 + 事件。light-dismiss / auto
  // 关闭其他 popover / popovertarget 按钮 defer；show 前校验元素仍连接到当前文档。
  function _zwShowPopover(key, sel, handle) {
    if (_zwReadPopover(sel, handle) === null) throw new DOMException('showPopover: not a popover element', 'InvalidStateError');
    if (!_zwIsConnected(sel, handle)) throw new DOMException('showPopover: element is not connected', 'InvalidStateError');
    if (_zwTopLayer[key]) throw new DOMException('showPopover: already showing', 'InvalidStateError');
    if (!_dispatchWithBubble(key, sel, handle, _makeToggleEvent('beforetoggle', 'closed', 'open', true))) return;
    _zwTopLayer[key] = true;
    _dispatchWithBubble(key, sel, handle, _makeToggleEvent('toggle', 'closed', 'open', false));
  }
  // R3071：hidePopover 状态机。未 showing → InvalidStateError；派发 beforetoggle(cancelable, open→closed)，
  // preventDefault → 中止不隐；移 top-layer；派发 toggle(open→closed)。
  function _zwHidePopover(key, sel, handle) {
    if (!_zwTopLayer[key]) throw new DOMException('hidePopover: not showing', 'InvalidStateError');
    if (!_dispatchWithBubble(key, sel, handle, _makeToggleEvent('beforetoggle', 'open', 'closed', true))) return;
    delete _zwTopLayer[key];
    _dispatchWithBubble(key, sel, handle, _makeToggleEvent('toggle', 'open', 'closed', false));
  }
  // R3290：HTMLDialogElement.show()——非模态打开。spec「show the dialog」：未连接则抛 InvalidStateError；
  // 已 open 时 no-op，否则设置 open 内容属性。headless 无真
  // top-layer paint / ::backdrop / focus 陷阱 / inert backdrop（rendering 流域 defer）——仅 JS-observable 状态（open 属性 +
  // 模态态）。
  // https://html.spec.whatwg.org/multipage/interactive-elements.html#dom-dialog-show
  function _zwDialogShow(key, sel, handle) {
    if (!_zwIsConnected(sel, handle)) throw new DOMException('show: dialog is not connected', 'InvalidStateError');
    if (_zwDialogHasOpen(sel, handle) || _zwDialogModal[key]) return;
    _zwSetAttr(key, sel, handle, 'open', '');
  }
  // R3290：HTMLDialogElement.showModal()——模态打开。spec「show a modal dialog」：已打开（open 属性 present）→
  // 抛 InvalidStateError；否则设 open 属性 + 加 top-layer + 标模态态。headless 简化（无 backdrop / focus / inert）。
  // https://html.spec.whatwg.org/multipage/interactive-elements.html#dom-dialog-showmodal
  function _zwDialogShowModal(key, sel, handle) {
    if (!_zwIsConnected(sel, handle)) throw new DOMException('showModal: dialog is not connected', 'InvalidStateError');
    if (_zwDialogHasOpen(sel, handle) || _zwDialogModal[key]) throw new DOMException('showModal: dialog already open', 'InvalidStateError');
    _zwSetAttr(key, sel, handle, 'open', '');
    _zwTopLayer[key] = true;
    _zwDialogModal[key] = true;
  }
  // R3290：HTMLDialogElement.close(returnValue)——关闭。spec「close the dialog」：未 open（无 open 属性且非模态态）
  // → no-op（返 false，不抛）；否则移 open 属性 + 模态态移 top-layer + 清模态态；returnValue 非 undefined → 存；
  // 排队 'close' 事件（headless 同步派发，spec task 近似——documented）。return true（已关）。
  // https://html.spec.whatwg.org/multipage/interactive-elements.html#dom-dialog-close
  function _zwDialogClose(key, sel, handle, returnValue) {
    var wasOpen = _zwDialogHasOpen(sel, handle) || !!_zwDialogModal[key];
    _zwRemoveAttr(key, sel, handle, 'open');
    if (_zwDialogModal[key]) { delete _zwDialogModal[key]; delete _zwTopLayer[key]; }
    if (returnValue !== undefined) _expando[key + '::returnValue'] = String(returnValue);
    if (wasOpen) _dispatchWithBubble(key, sel, handle, _makeEvent('close', { bubbles: false, cancelable: false }));
    return wasOpen;
  }
  // R3290：dialog open 内容属性是否 present（boolean 属性，presence 判定）。latest-wins 反映同 execute 内
  // pending set/remove（show/close 经 __zw_set_attr/__zw_remove_attr 异步入队，纯快照读 stale）。供 showModal
  // 校验 + close wasOpen 判定共用。
  function _zwDialogHasOpen(sel, handle) {
    if (handle) {
      try { return __zw_has_attr_handle(handle, 'open') === '1'; } catch (_e) { return false; }
    }
    return (typeof __zw_has_attr_lw === 'function'
      ? __zw_has_attr_lw(sel, 'open')
      : (typeof __zw_has_attr === 'function' ? __zw_has_attr(sel, 'open') : '0')) === '1';
  }
  // R3290：统一 set/remove 内容属性 helper（sel/handle 双路径 + latest-wins 读一致性依赖 host 侧 latest-wins 变体，
  // 写走常规 __zw_set_attr/__zw_remove_attr 入队）。dialog open 属性专用，与 popover setter 同模式。
  function _zwSetAttr(key, sel, handle, name, value) {
    if (handle && typeof __zw_set_attr_handle === 'function') __zw_set_attr_handle(handle, name, value);
    else if (!handle) __zw_set_attr(sel, name, value);
  }
  function _zwRemoveAttr(key, sel, handle, name) {
    if (handle && typeof __zw_remove_attr_handle === 'function') __zw_remove_attr_handle(handle, name);
    else if (!handle && typeof __zw_remove_attr === 'function') __zw_remove_attr(sel, name);
  }
  // R3072：popovertarget 声明式触发——click 的 default action。click 派发后未 preventDefault → 找最近含
  // popovertarget 内容属性的祖先（含自身）→ 读 popovertarget(id) + popovertargetaction(toggle/show/hide) →
  // document.getElementById 找目标 popover 元素 → 按 action 调 showPopover/hidePopover/togglePopover。复用 R3071
  // 状态机（InvalidStateError 经 try/catch 吞——spec「show on showing」/「hide on hidden」/「target 非 popover」no-op）。
  // spec 限 button/input 触发元素，本实现 permissive（任意元素含 popovertarget 即触发，headless 简化）。
  // light-dismiss / auto 关闭其他 popover defer（R3071 同限）。handle-only（detached）无祖先链 → 跳过。
  // https://html.spec.whatwg.org/multipage/popover.html#popover-target-activation
  function _zwPopoverTargetActivate(key, sel, handle) {
    if (!sel) return; // handle-only detached 无 sel 祖先链（popovertarget 声明式需 DOM 树内 button）
    // 找最近含 popovertarget 内容属性**或**编程式 popoverTargetElement（R3073）的祖先（含自身）。
    var trigger = '';
    var cur = sel;
    while (cur) {
      var curKey = _elKey(cur, null);
      var has = typeof __zw_has_attr_lw === 'function'
        ? __zw_has_attr_lw(cur, 'popovertarget')
        : (typeof __zw_has_attr === 'function' ? __zw_has_attr(cur, 'popovertarget') : '0');
      if (has === '1' || _popoverTargetEl[curKey]) { trigger = cur; break; }
      try { cur = __zw_parent(cur); } catch (_e) { cur = ''; }
      if (!cur) break;
    }
    if (!trigger) return;
    var triggerKey = _elKey(trigger, null);
    // 目标：编程式 popoverTargetElement 优先于 popovertarget 内容属性（spec）。
    var popoverEl = _popoverTargetEl[triggerKey];
    if (!popoverEl) {
      var id = typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(trigger, 'popovertarget') : __zw_get_attr(trigger, 'popovertarget');
      if (!id) return;
      popoverEl = document.getElementById(id);
    }
    if (!popoverEl) return;
    var actionRaw = typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(trigger, 'popovertargetaction') : __zw_get_attr(trigger, 'popovertargetaction');
    var action = String(actionRaw || 'toggle').toLowerCase();
    if (action !== 'show' && action !== 'hide' && action !== 'toggle') action = 'toggle';
    try {
      if (action === 'show') popoverEl.showPopover();
      else if (action === 'hide') popoverEl.hidePopover();
      else popoverEl.togglePopover();
    } catch (_e) {} // InvalidStateError（已 showing show / 未 showing hide / target 非 popover）spec no-op
  }
  // R3074：Element.checkVisibility(options)——元素是否「being rendered」+ 可选 opacity/visibility 检查。
  // https://drafts.csswg.org/cssom-view-1/#dom-element-checkvisibility
  // 算法：① display:none 在元素或任一祖先 → not rendered → false（默认，无需 option）；② options.opacityProperty
  // 且元素或任一祖先 computed opacity=0 → false（opacity 不继承，须遍历祖先）；③ options.visibilityProperty 且元素
  // computed visibility 非 visible（hidden/collapse）→ false（visibility 继承，查元素自身计算值即反映祖先）。
  // 经 host `__zw_get_computed_style(sel, prop)`（engine/production 已注册；未注册 lenient 返 true 防破脚本）。
  // handle-only detached（无 sel）→ 不在文档 → not rendered → false。contentVisibilityAuto（content-visibility:auto）
  // defer（harness 未计算 content-visibility，niche）。
  function _zwCheckVisibility(sel, handle, options) {
    if (!sel) return false; // handle-only detached → not in document → not rendered
    options = options || {};
    var hasCS = typeof __zw_get_computed_style === 'function';
    // visibility（继承——查元素自身计算值）。
    if (options.visibilityProperty && hasCS) {
      var vis = __zw_get_computed_style(sel, 'visibility');
      if (vis === 'hidden' || vis === 'collapse') return false;
    }
    // display（不继承——元素或任一祖先 none）+ opacity（不继承——任一祖先 0）。遍历祖先链。
    var cur = sel;
    while (cur) {
      if (hasCS) {
        var disp = __zw_get_computed_style(cur, 'display');
        if (disp === 'none') return false;
        if (options.opacityProperty) {
          var op = __zw_get_computed_style(cur, 'opacity');
          if (parseFloat(op) === 0) return false;
        }
      }
      try { cur = __zw_parent(cur); } catch (_e) { cur = ''; }
      if (!cur) break;
    }
    return true;
  }
  // R3047：scroll 位置追踪。headless 无真视口滚动 → 旧 scrollTop/scrollLeft 恒 0、scrollTo/scrollBy/scroll no-op、
  // window.scrollX/Y 恒 0。real 浏览器这些为可读写状态（sticky-nav / scroll restoration / 无限滚动检测 / parallax 读）。
  // 本切片改 JS-side 状态追踪：`scrollTo/scrollBy` + `scrollTop/scrollLeft` set 更新此 map，get 读回 → 程序化滚动
  // round-trip 一致（`scrollTo(0,100); scrollY` → 100）。无真视口滚动（headless），仅 JS-observable 状态自洽。
  // `_scrollOffsets`：per-element-key → { top, left }；`_winScroll`：window → { top, left }（scrollX=left / scrollY=top）。
  // 负值 clamp 0（spec scroll 不可负）。导航经 __zw_reset_form_state 重置。
  var _scrollOffsets = {};
  var _winScroll = { top: 0, left: 0 };
  // reflected 字符串/数值属性（title/lang/dir/tabindex）per-element-key 缓存。同 _inputValues/_classCache
  // 动机——`__zw_set_attr` 仅入队 mutation（异步 apply），同步 set→get 往返须客户端缓存（get 优先读缓存）。
  // 值结构：{ title?: string, lang?: string, dir?: string, tabindex?: number }。
  var _reflectedAttrs = {};
  // R3037：reflected string 内容属性 IDL 名 → 内容属性名。这些属性 get 旧返 undefined（get trap 未拦），
  // 写正常（set trap generic fallthrough → __zw_set_attr）。表单校验库读 input.min/max/pattern/type、
  // analytics 读 src/name 等全失效。get trap 经 [`_reflectedStringAttr`] 查表，命中则读内容属性（缺省 ''，
  // spec reflected string 缺省空串）。1:1 小写名用 `_REFLECTED_STRING_FLAT`；camelCase→attr 映射用 `_REFLECTED_STRING_MAP`。
  // 数值型（size/maxLength/colSpan/rowSpan）+ 布尔型（required/readonly/multiple）spec 返 number/boolean，
  // 另列 follow-up（本切片仅 string）。
  var _REFLECTED_STRING_FLAT = ' type name placeholder alt min max step pattern action method enctype target rel download headers srcset sizes loading accept inputmode src usemap ';
  var _REFLECTED_STRING_MAP = { crossOrigin: 'crossorigin', formAction: 'formaction', formMethod: 'formmethod', formEnctype: 'formenctype', formTarget: 'formtarget', htmlFor: 'for' };
  function _reflectedStringAttr(prop) {
    if (typeof prop !== 'string') return null;
    if (Object.prototype.hasOwnProperty.call(_REFLECTED_STRING_MAP, prop)) return _REFLECTED_STRING_MAP[prop];
    if (_REFLECTED_STRING_FLAT.indexOf(' ' + prop + ' ') >= 0) return prop;
    return null;
  }
  // R3187：contentEditable 枚举状态求值——返 'true' / 'false' / 'inherit'。spec HTML `contenteditable`
  // 为枚举属性，关键字「空串、true、false」——**空串与 true 同映射 true 状态**（故 `<div contenteditable>`
  // 等价 `<div contenteditable="true">`）。缺省（属性不存在）/ 非法（incl "foo"/"inherit"）→ inherit 状态。
  // **缺省 ≠ 空串 keyword**：`__zw_get_attr` 对缺省与空值均返 ""，须用 `__zw_has_attr*` 判存在性区分。
  // setter 写过的缓存值（`_reflectedAttrs[key].contenteditable`）视为 present（同步 set→get）；余经 host
  // has_attr 判存在 + get_attr 读值。供 `contentEditable` / `isContentEditable` 共用（避免重复）。
  function _contentEditableState(key, sel, handle) {
    var cec = _reflectedAttrs[key];
    var present, raw;
    if (cec && Object.prototype.hasOwnProperty.call(cec, 'contenteditable')) {
      present = true;
      raw = cec['contenteditable'];
    } else {
      present = (handle
        ? __zw_has_attr_handle(handle, 'contenteditable')
        : (typeof __zw_has_attr_lw === 'function' ? __zw_has_attr_lw(sel, 'contenteditable') : __zw_has_attr(sel, 'contenteditable'))) === '1';
      raw = present
        ? (handle ? __zw_get_attr_handle(handle, 'contenteditable') : __zw_get_attr(sel, 'contenteditable'))
        : '';
    }
    if (!present) return 'inherit';
    var lo = String(raw).toLowerCase();
    return (raw === '' || lo === 'true') ? 'true' : (lo === 'false' ? 'false' : 'inherit');
  }
  // R3188：`draggable` auto 状态 default-draggable 判定。spec HTML `draggable` 为枚举属性，缺省/非法 → auto
  // 状态——元素的拖拽性由 UA 默认行为决定。spec/Chrome：`img`/`audio`/`video` 默认可拖拽，`a`（带 href）默认可
  // 拖拽，余默认不可拖拽。供 `draggable` getter 在 auto 状态下求值（true/false 关键字未命中时）。
  function _defaultDraggable(sel, handle) {
    var tag = _realTag(sel, handle);
    if (tag === 'IMG' || tag === 'AUDIO' || tag === 'VIDEO') return true;
    if (tag === 'A') {
      return (handle
        ? __zw_has_attr_handle(handle, 'href')
        : (typeof __zw_has_attr_lw === 'function' ? __zw_has_attr_lw(sel, 'href') : __zw_has_attr(sel, 'href'))) === '1';
    }
    return false;
  }
  // R3189：input/button `type` enumerated reflection（spec「limited to only known values」）。区别于通用 type
  // 字符串反射（link/script/style/embed 等）——`<input>.type` / `<button>.type` getter 须规范化：
  // INPUT 已知关键字（见 `_INPUT_TYPE_KEYWORDS`，case-insensitive）→ 规范小写；缺省 / 非法 → "text"
  //（spec missing & invalid value default 均 Text 状态）。BUTTON 关键字 submit/reset/button；缺省/非法 → "submit"。
  // 非 INPUT/BUTTON → null（caller 回落通用字符串反射）。表单库 switch(input.type) 高频。
  var _INPUT_TYPE_KEYWORDS = ' button checkbox color date datetime-local email file hidden image month number password radio range reset search submit tel text time url week ';
  function _reflectedTypeEnum(sel, handle) {
    var tag = _realTag(sel, handle);
    if (tag !== 'INPUT' && tag !== 'BUTTON') return null;
    var raw = handle
      ? __zw_get_attr_handle(handle, 'type')
      : (typeof __zw_get_attr_lw === 'function' ? __zw_get_attr_lw(sel, 'type') : __zw_get_attr(sel, 'type'));
    var lo = (raw == null || raw === '') ? '' : String(raw).toLowerCase();
    if (tag === 'INPUT') {
      if (lo === '') return 'text'; // 缺省 → Text 状态。
      return _INPUT_TYPE_KEYWORDS.indexOf(' ' + lo + ' ') >= 0 ? lo : 'text'; // 非法 → Text 状态。
    }
    // BUTTON：submit/reset/button 关键字；缺省/非法 → "submit"（spec missing & invalid default）。
    return (lo === 'submit' || lo === 'reset' || lo === 'button') ? lo : 'submit';
  }
  // R3038/R3041：reflected unsigned-long（numeric）+ boolean 属性读（R3037 follow-up——string 已在 R3037 覆盖）。
  // 数值型 spec 返 number（缺省 default，colSpan/rowSpan spec default 1 且 min 1；maxLength/minLength default -1
  // = 不限制；cols/rows/start R3041——textarea cols default 20 / rows default 2，ol start default 1，无 min 故
  // 边界 <1 值原样返，pragmatic 近似）。布尔型 spec 返 boolean（presence-based：属性存在 true / 缺省 false）。
  // 读旧恒 undefined。set 走既有 generic fallthrough（__zw_set_attr 写属性串），读 parseInt 往返（同 maxLength）。
  var _REFLECTED_UINT = {
    colSpan: { a: 'colspan', d: 1, min: 1 },
    rowSpan: { a: 'rowspan', d: 1, min: 1 },
    maxLength: { a: 'maxlength', d: -1 },
    minLength: { a: 'minlength', d: -1 },
    cols: { a: 'cols', d: 20 },
    rows: { a: 'rows', d: 2 },
    start: { a: 'start', d: 1 },
  };
  // 布尔 reflected 属性表（IDL 名 → 内容属性名）。get trap presence 读返 boolean（R3038）；set trap
  // truthy→设 presence / falsy→removeAttribute（R3039，闭合 set-false bug）。仅收录**纯布尔 presence-based** 属性；
  // 枚举型（draggable/spellcheck="true"/"false" 等）与含 dirty/default 态的（defaultChecked/defaultMuted）
  // 不入此表（前者走 R2848 分支，后者需独立 default 缓存模式）。
  //   - 表单（HTMLFormElement）：required/readOnly(textarea)/multiple(select)（R3038/R3039）+ noValidate（R3040）
  //   - 脚本（HTMLScriptElement）：async/defer/nomodule（R3040）
  //   - 媒体（HTMLMediaElement/HTMLVideoElement）：autoplay/controls/loop/muted/playsInline（R3040）
  //   - 列表（HTMLOListElement）：reversed（R3040）
  //   - 图像（HTMLImageElement）：isMap（R3040）
  //   - 全局微数据（HTMLElement）：itemScope（R3040）
  // 注：hidden/checked/disabled/selected 走更早的显式分支（含 default 态保护，part05.js）；autofocus/inert 走
  // R2848/R2850 分支（含 _reflectedAttrs 缓存）——均不入此表以免改变既有 set 语义（最小 blast radius）。
  var _REFLECTED_BOOL = {
    required: 'required', readOnly: 'readonly', multiple: 'multiple', noValidate: 'novalidate',
    async: 'async', defer: 'defer', nomodule: 'nomodule',
    autoplay: 'autoplay', controls: 'controls', loop: 'loop', muted: 'muted', playsInline: 'playsinline',
    reversed: 'reversed', isMap: 'ismap', itemScope: 'itemscope',
  };
  // R3039：查 _REFLECTED_BOOL 返内容属性名（readOnly→readonly 等），非 string/未命中 → null。供 set trap
  // 布尔 falsy→removeAttribute 分支与 get trap presence 读共用。
  function _reflectedBoolAttr(prop) {
    if (typeof prop !== 'string') return null;
    if (Object.prototype.hasOwnProperty.call(_REFLECTED_BOOL, prop)) return _REFLECTED_BOOL[prop];
    return null;
  }
  // P1a DocumentFragment：已创建的 fragment handle 集合（nodeType=11 标识 + appendChild 时
  // flatten 检测）。fragment 为 create 句柄，无 selector，故用此 set 区别于普通元素句柄。
  var _fragmentHandles = {};
  // R2926 Shadow DOM（attachShadow，Tier 2 Web Components 地基）：host 元素 elKey → 其 shadow root
  //（{ handle, mode }）。shadow root 复用 DocumentFragment handle 容器（故 handle 亦入 _fragmentHandles），
  // 另入 _shadowHandles 标 shadow-root 身份（nodeName '#shadow-root' + host/mode）。host 元素调
  // attachShadow 建；shadowRoot getter 读（open 返 root / closed·未建 返 null，spec）。导航清空（页级）。
  var _shadowRoots = {};
  var _shadowHandles = {};
  var _shadowHandleMeta = {};
  // R2927 handle-children registry：容器 handle（shadow root / fragment）→ 其子节点 proxy 列表。
  // 这些容器无 selector（handle-only），既有 childNodes/children 经 `__zw_child_nodes(sel)` 读（须 sel）
  // → 恒返 []。本 registry 在 appendChild（容器父）时同步记录子节点，供 childNodes/firstChild/
  // lastChild/firstElementChild/lastElementChild/childElementCount 读。仅 handle-append 模式覆盖
  //（innerHTML 设内容经 host parse 无 handle，未跟踪——follow-up）。导航清空。
  var _handleChildren = {};
  // P1a Comment（R2816）：已创建的 comment handle 集合（nodeType=8 / nodeName '#comment' 标识）。
  // comment 为 create 句柄无 selector，故用此 set 区别于普通元素句柄（同 _fragmentHandles 模式）。
  var _commentHandles = {};
  // P1a Text（R2816）：已创建的 text handle 集合（nodeType=3 / nodeName '#text' 标识）——修正旧实现 created
  // text 节点误报 nodeType 1（element）的 bug（与 _commentHandles 对称）。createTextNode 经 __zw_create_text。
  var _textHandles = {};
  // js-dom M4 ProcessingInstruction（spec `dom-document-createprocessinginstruction`）：已创建的 PI handle
  // 集合（nodeType=7），存 { target, data }（PI 无独立 selector，区别于普通元素句柄；与 _commentHandles 对称）。
  // target/data/nodeName(=target) 经此读回（PI 节点无 CharacterData 编辑方法）。
  var _piHandles = {};
  // js-dom M4 createElementNS（spec `dom-document-createelementns`，R18）：已创建的命名空间元素 handle 集合，
  // 存 { qualifiedName, namespace }（**大小写敏感**原值）。区别普通 `createElement` handle（经 `_realTag`
  // 强制大写 + host `create_element` 小写）：createElementNS spec 不小写 localName，须保留原大小写，且带
  // prefix（`"p:l"`）/namespace。`tagName`/`nodeName`/`prefix`/`localName`/`namespaceURI` getter 先查此表，
  // 命中则返大小写敏感正确值（不经 `_realTag` 大写化）。与 _piHandles 对称的 handle 标识模式。
  var _nsHandles = {};
  // js-dom M4 R19：DOMTokenList（classList）per-element 缓存。spec `dom-element-classlist`——`classList` 是
  // accessor property，**每次访问返回同一 cached DOMTokenList 对象**（WPT `assert_equals(e.classList, expect)`
  // 要求 identity 相等：`var expect=e.classList; e.classList="foo"; assert_equals(e.classList, expect)`）。
  // 旧实现每次 `_classListProxy` 新建 Proxy → identity 不等（classList assignment no-op 后再读得新对象）。
  // 经 `_clsProxyCache[key]` 缓存，同元素 get 始终返同一 proxy（与 `_proxyCache` 元素代理缓存同模式）。
  var _clsProxyCache = {};
  // ── 浏览器运行时桩（定时器、navigator、location 等）──
  var _timerId = 1;
  // queueMicrotask——调度 microtask（高频：每个异步库 / polyfill / 框架都用）。本 V8 embed 未暴露
  // 全局 queueMicrotask（probe 确认 undefined），用 `Promise.resolve().then` polyfill——V8 在 execute
  // 末 perform_microtask_checkpoint 派发，同 spec「当前 task 末、下 task 前」语义。亦使上方 _defer
  // 走真 queueMicrotask 分支（行为同 Promise.then fallback，零变化）。
  globalThis.queueMicrotask = globalThis.queueMicrotask || function (cb) {
    if (typeof cb !== 'function') throw new TypeError('queueMicrotask: callback not callable');
    Promise.resolve().then(cb);
  };

  // 单次脚本执行内 microtask 派发上限（避免 setTimeout 轮询在 checkpoint 中无限链式调度）。
  var _deferBudget = 256;

  function _defer(fn) {
    if (_deferBudget <= 0) return;
    _deferBudget--;
    if (typeof queueMicrotask === 'function') {
      queueMicrotask(function() { try { fn(); } catch (_e) {} });
    } else if (typeof Promise === 'function') {
      Promise.resolve().then(function() { try { fn(); } catch (_e) {} });
    } else {
      try { fn(); } catch (_e) {}
    }
  }

  // requestAnimationFrame / takeScreenshot 预算：单次脚本执行内同步派发上限，
  // 防止动画循环（rAF(loop)）无限链式触发；reftest 的「double-rAF 后 setup」
  // 模式只需 2-3 帧即可收敛。
  var _rafBudget = 64;

  // P1a 事件循环 slice 1（R2713a）：帧驱动 rAF kill-switch + 注册队列。
  // `__ZW_RAF_FRAME_DRIVEN` 由 host（worker init 读 env `ZW_RAF_FRAME_DRIVEN`）在 execute 前注入：
  // unset/false = 同步 stub（reftest 兼容，rAF 立即 fn(0)）；true = 帧驱动（rAF 注册到
  // `_rafPending`，render 后 host 调 `__zw_raf_tick(ts)` 派发）。OFF 时 `__zw_raf_tick` 早返零开销。
  // 详见 docs/goal/zero-web/p1a-event-loop-raf-slice-design-2026-08-05.md。
  globalThis.__ZW_RAF_FRAME_DRIVEN = globalThis.__ZW_RAF_FRAME_DRIVEN || false;
  var _rafPending = {}; // id -> fn（帧驱动路径注册队列；OFF 路径不填充）

  // P1a Slice 2b：observer 注册表——host render 后经 `__zw_observers_tick()` 对每个活跃
  // observer 调 `_schedule()`，使 IO/RO 在 cross-threshold / size-change 时派发后续通知
  // （observe 仅派发 initial；后续 render 的真实 layout 变化须 host tick 触发复算）。
  // IO/RO 构造时 push；tick 跳过无活跃 target 者（disconnect 后为 no-op）。
  // leak = observer 创建总数（有界，per-page；WeakRef 注册表为后续硬化 follow-up）。
  var _zwObservers = [];

  globalThis.__zw_begin_script = function() {
    _deferBudget = 256;
    _rafBudget = 64;
    // R2946：每页首次脚本执行前反射 <body on*> → window.on*（幂等，按 page URL 去重）。
    if (typeof _zw_reflect_body_window_handlers === 'function') _zw_reflect_body_window_handlers();
  };

  // P1b S1（方案 A）异步回调 resolve 通道（JS 侧契约）：
  // Rust 异步完成（fetch / timer 等后续切片接通）后经
  // `V8Sandbox::resolve_async_callback(id, result)` 执行 `__zwResolveCallback(id, result)`，
  // 从 pending 表取出 resolver 触发 Promise resolve。execute 末尾的 microtask
  // checkpoint 随即 drain `.then` 回调。pending 表 idempotent 初始化——跨脚本执行
  // 存活（resolve 可晚于注册），且 shim 重注入时不覆盖既有 pending 项。
  globalThis.__zw_pending = globalThis.__zw_pending || {};
  globalThis.__zwResolveCallback = function(id, result) {
    var r = globalThis.__zw_pending[id];
    if (typeof r === 'function') {
      delete globalThis.__zw_pending[id];
      r(result);
    }
  };

  // P1b S3 incr-c / R2923 fetch 完整化：fetch 返回 Response 对象（spec-compliance：ok/status/
  // statusText/headers/text()/json()）。host 经 `__zw_fetch` 抓取返 `__zwfr:` wire
  //（status\x1fstatusText\x1fheadersWire\x1fbody）或 `__zw_fetch_error:` 错误标记 → shim 包装为 Response。
  // body 为 wire 末字段（取第 3 个 \x1f 后全部，可含 \x1f）。错误 / 旧 body-only wire → 兜底 _makeResponse。
  function _parseHeadersWire(wire) {
    var out = {};
    if (!wire) return out;
    var parts = wire.split('\x1e');
    for (var i = 0; i + 1 < parts.length; i += 2) {
      var k = parts[i], v = parts[i + 1];
      // R3222：多值头（Set-Cookie 等）累加为数组（旧 last-wins 丢多 cookie——getSetCookie 失效）。
      if (Object.prototype.hasOwnProperty.call(out, k)) {
        if (Array.isArray(out[k])) out[k].push(v);
        else out[k] = [out[k], v];
      } else {
        out[k] = v;
      }
    }
    return out;
  }
  // 旧 / 错误路径：body 为裸文本（status 200）或 `__zw_fetch_error:` 前缀（ok:false）。增 headers:{}（向后兼容）。
  function _makeResponse(body) {
    var ok = typeof body === 'string' && body.indexOf('__zw_fetch_error') !== 0;
    return {
      ok: ok,
      status: ok ? 200 : 0,
      statusText: ok ? 'OK' : 'Error',
      headers: {},
      // R2967：body 为 ReadableStream（lazy，单 UTF-8 chunk + close）。网络错误（ok:false）→ null（spec）。
      get body() {
        if (!ok) return null;
        if (!this._bs) this._bs = _bodyToStream(body);
        return this._bs;
      },
      text: function() { return Promise.resolve(ok ? body : ''); },
      json: function() { return Promise.resolve(JSON.parse(ok ? body : 'null')); }
    };
  }
  // 解析 host→JS wire 为 Response。`__zwfr:` 前缀 → status/statusText/headers/body；
  // `__zw_fetch_error:` 或非 wire → 落 _makeResponse（错误 / 旧路径兼容）。
  function _makeResponseFromWire(raw) {
    if (typeof raw !== 'string') return _makeResponse('__zw_fetch_error:bad-wire');
    if (raw.indexOf('__zw_fetch_error') === 0) return _makeResponse(raw);
    if (raw.indexOf('__zwfr:') !== 0) return _makeResponse(raw);
    var rest = raw.slice(7); // strip '__zwfr:'
    var p1 = rest.indexOf('\x1f');
    var p2 = p1 >= 0 ? rest.indexOf('\x1f', p1 + 1) : -1;
    var p3 = p2 >= 0 ? rest.indexOf('\x1f', p2 + 1) : -1;
    if (p1 < 0 || p2 < 0 || p3 < 0) return _makeResponse('__zw_fetch_error:malformed');
    var status = parseInt(rest.slice(0, p1), 10) || 0;
    var statusText = rest.slice(p1 + 1, p2);
    var headersWire = rest.slice(p2 + 1, p3);
    var body = rest.slice(p3 + 1); // 末字段，可含 \x1f
    var headers = _parseHeadersWire(headersWire);
    // R3021：二进制 response body 经 `__zw_bytes:` csv-decimal wire → Uint8Array（response.blob()/arrayBuffer() 保真）；
    // 文本 body 原样字符串。
    var bodyArg = body.indexOf('__zw_bytes:') === 0 ? _zwDecodeBytesPrefix(body) : body;
    // R2968：经 new Response 构造（fetch 结果 instanceof Response）。字段 shape 与旧 plain object 一致
    //（headers 经 new Response 封装为 Headers 实例，R2977；body getter 同 R2967）。
    return new Response(bodyArg, { status: status, statusText: statusText, headers: headers });
  }
  // 收集 headers 源（Object / [[k,v]] / Headers-like forEach）→ `name\x1evalue\x1e...` wire（空 → ''）。
  function _headersToWire(src) {
    if (!src) return '';
    var pairs = [];
    if (typeof src.forEach === 'function') {
      src.forEach(function(v, k) { pairs.push([String(k), String(v)]); });
    } else if (Array.isArray(src)) {
      for (var i = 0; i < src.length; i++) {
        var e = src[i];
        if (Array.isArray(e)) pairs.push([String(e[0]), String(e[1])]);
      }
    } else {
      for (var k in src) {
        if (Object.prototype.hasOwnProperty.call(src, k)) pairs.push([String(k), String(src[k])]);
      }
    }
    var out = '';
    for (var j = 0; j < pairs.length; j++) {
      // R3221：Fetch §3.4.4 出口过滤禁止请求头（JS 设的 Host/Content-Length/Cookie/Sec-*/Proxy-* 等永不到达 host）。
      if (_zwIsForbiddenReqHeader(pairs[j][0].toLowerCase())) continue;
      out += (out ? '\x1e' : '') + pairs[j][0] + '\x1e' + pairs[j][1];
    }
    return out;
  }
  // R3014：headersWire（\x1e 分隔 name/value 对）header 查询/追加——fetch FormData body 接 Content-Type。
  function _zwHasHeader(wire, name) {
    if (!wire) return false;
    var parts = wire.split('\x1e');
    var ln = String(name).toLowerCase();
    for (var i = 0; i < parts.length; i += 2) if (String(parts[i]).toLowerCase() === ln) return true;
    return false;
  }
  function _zwAddHeader(wire, name, value) {
    return (wire ? wire + '\x1e' : '') + String(name) + '\x1e' + String(value);
  }

  // R2923 fetch 完整化：`fetch(input, init)` 透传 method/headers/body → host 返 status/headers/body。
  // input = URL 字符串或 Request-like（.url/.method/.headers/.body）；init = { method, headers, body }。
  // method 默认 GET；GET/HEAD 无 body。`__zw_fetch` 未注册（engine/reftest/polyfill 无 host fetch handler）
  // 时 resolve ok:false Response（stub，避免悬挂，零回归）。
  // R3020：Blob/FormData 二进制 body 经 `_zwEncodeBytesPrefix`（`__zw_bytes:` + csv-decimal）传 host，
  // host 解码为 Vec<u8> 闭合二进制保真（旧 TextDecoder.decode 对非 UTF-8 字节 lossy，破坏 0xFF/0x00）。
  function _zwEncodeBytesPrefix(bytes) {
    var s = '__zw_bytes:';
    for (var i = 0; i < bytes.length; i++) {
      if (i > 0) s += ',';
      s += (bytes[i] & 0xFF);
    }
    return s;
  }
  // R3021：解码 `__zw_bytes:` csv-decimal wire → Uint8Array（与 host encode_body_bytes 对称）。
  // 供 _makeResponseFromWire 把二进制 response body 还原为字节，response.blob()/arrayBuffer() 取保真字节。
  function _zwDecodeBytesPrefix(wire) {
    var rest = wire.slice(11); // strip '__zw_bytes:'
    if (!rest) return new Uint8Array(0);
    var parts = rest.split(',');
    var arr = new Uint8Array(parts.length);
    for (var i = 0; i < parts.length; i++) arr[i] = parseInt(parts[i], 10) & 0xFF;
    return arr;
  }
  if (!globalThis.fetch) {
    globalThis.fetch = function(input, init) {
      init = init || {};
      var isObj = input && typeof input === 'object';
      var url = isObj ? String(input.url || '') : String(input);
      var method = String(init.method || (isObj ? input.method : '') || 'GET').toUpperCase();
      var headersWire = _headersToWire(init.headers) || (isObj ? _headersToWire(input.headers) : '');
      var body = '';
      // R3014/R3015/R3020：body 类型分发——FormData（multipart）/ URLSearchParams（urlencoded）/ Blob（字节）/
      // string（原样）。各专用类型在用户未设 Content-Type 时设默认值（缺省 Content-Type 不覆写用户显式值）。
      // 文本（URLSearchParams/string）经 UTF-8 wire 保真；二进制（FormData multipart / Blob）经 byte-wire 全保真。
      var rawBody = init.body != null ? init.body : (isObj && input.body != null ? input.body : null);
      if (rawBody instanceof FormData) {
        var mp = rawBody._zwMultipart();
        body = _zwEncodeBytesPrefix(mp.body); // R3020：multipart 字节 byte-wire（含二进制文件内容保真）
        if (!_zwHasHeader(headersWire, 'content-type')) headersWire = _zwAddHeader(headersWire, 'content-type', mp.contentType);
      } else if (rawBody instanceof URLSearchParams) {
        body = String(rawBody); // toString → urlencoded
        if (!_zwHasHeader(headersWire, 'content-type')) headersWire = _zwAddHeader(headersWire, 'content-type', 'application/x-www-form-urlencoded;charset=UTF-8');
      } else if (rawBody instanceof Blob) {
        body = _zwEncodeBytesPrefix(_zw_blobBytes(rawBody)); // R3020：Blob 字节 byte-wire（二进制保真）
        if (!_zwHasHeader(headersWire, 'content-type')) headersWire = _zwAddHeader(headersWire, 'content-type', rawBody.type || 'application/octet-stream');
      } else if (rawBody != null) {
        body = String(rawBody);
      }
      if (typeof __zw_fetch !== 'function') {
        return Promise.resolve(_makeResponse('__zw_fetch_error:no-handler'));
      }
      // R3044/R3045：AbortSignal——fetch 中止。AbortController/AbortSignal 对象已就绪（part02），但 fetch 旧不消费
      // init.signal → controller.abort() 无法中止在途 fetch。本切片接通：signal 已 aborted → 立即 reject；
      // 运行中 abort → reject(signal.reason) + 清 __zw_pending[id]（host 抓取结果到达时 __zwResolveCallback
      // typeof-check no-op，结果被丢弃）。settled flag 防 resolve/abort 双 settle。fetch reject reason = signal.reason
      //（spec；默认 AbortError DOMException，或 abort(reason) 传入值）。signal 来源（R3045）：init.signal 优先，
      // 否则 input 为 Request 时回落 input.signal（Request 构造器 R3045 存）。duck-type `instanceof AbortSignal`（非
      // AbortSignal 忽略，lenient）。仅影响有 signal 的 fetch 调用——无 signal 路径不变（零回归）。
      var signal = null;
      if (typeof AbortSignal === 'function') {
        if (init.signal instanceof AbortSignal) signal = init.signal;
        else if (isObj && input.signal instanceof AbortSignal) signal = input.signal;
      }
      return new Promise(function(resolve, reject) {
        // signal 已 aborted → 同步 reject（spec：fetch 入口检查 signal.aborted）。
        if (signal && signal._aborted) {
          reject(signal.reason);
          return;
        }
        globalThis.__zw_fetch_counter = (globalThis.__zw_fetch_counter | 0) + 1;
        var id = '__zwfid:' + globalThis.__zw_fetch_counter;
        var settled = false;
        globalThis.__zw_pending[id] = function(raw) {
          if (settled) return;
          settled = true;
          resolve(_makeResponseFromWire(raw));
        };
        if (signal) {
          signal.addEventListener('abort', function() {
            if (settled) return;
            settled = true;
            delete globalThis.__zw_pending[id]; // host 结果到达 → __zwResolveCallback no-op（typeof-check）
            reject(signal.reason);
          });
        }
        try {
          var _sync = __zw_fetch(id, method, url, headersWire, body);
          // R34xx：同步返回契约——headless/testharness 宿主（webview fetch_handler）同步返 wire；
          // 浏览器异步路径（fetch_bridge）返 "" → no-op（__zwResolveCallback 后到，双 settle 由
          // settled 防护）。unblock 2d.composite.image.*（fetch + createImageBitmap(blob)）。
          var _isSyncWire = typeof _sync === 'string' &&
            (_sync.indexOf('__zwfr:') === 0 || _sync.indexOf('__zw_fetch_error:') === 0);
          if (_isSyncWire && !settled) {
            settled = true;
            delete globalThis.__zw_pending[id];
            resolve(_makeResponseFromWire(_sync));
          }
        } catch (_e) {
          if (!settled) { settled = true; delete globalThis.__zw_pending[id]; }
          resolve(_makeResponse('__zw_fetch_error:throw'));
        }
      });
    };
  }

  // R2968：Response / Request 全局构造器（补全 fetch API 表面——此前仅 fetch()/Headers，缺 new Response/
  // new Request）。`new Response(body, init)` / `new Request(url, init)` 用于 service worker 构造响应、fetch
  // 包装库、测试 mock。`_makeResponseFromWire` 经 new Response 路由 → fetch 结果 instanceof Response（同时保持
  // 字段 shape 与旧 plain object 逐字段一致：ok/status/statusText/headers/body/text()/json()）。
  // R2977：headers 为 Headers 实例（spec Response.headers）。modern 代码经 `response.headers.get('content-type')`
  // 消费（比 bracket `headers['x']` 更常见 + 标准）——Headers 实例提供 get/has/append/set/delete/forEach/entries。
  // `new Headers(init)` 接受 plain dict / Headers-like / [[k,v]] / undefined。clone 经 new Response(headers) 再封装。
  // urlencoded 表单体 → FormData（R2982 抽出，Response.formData / Request.formData 共用）。
  // `+`→space + % 解码，spec application/x-www-form-urlencoded 语义（multipart/form-data 解析 defer）。
  function _zwParseFormUrlencoded(bodyText) {
    var fd = new FormData();
    var body = String(bodyText == null ? '' : bodyText).trim();
    if (body) {
      body.split('&').forEach(function (pair) {
        if (!pair) return;
        var eq = pair.indexOf('=');
        var k = eq >= 0 ? pair.slice(0, eq) : pair;
        var v = eq >= 0 ? pair.slice(eq + 1) : '';
        fd.append(decodeURIComponent(k.replace(/\+/g, ' ')), decodeURIComponent(v.replace(/\+/g, ' ')));
      });
    }
    return fd;
  }
  globalThis.Response = function Response(body, init) {
    if (!(this instanceof Response)) return new Response(body, init);
    init = init || {};
    var status = init.status != null ? (init.status | 0) : 200;
    this.status = status;
    this.ok = status >= 200 && status < 300;
    this.statusText = init.statusText != null ? String(init.statusText) : '';
    this.headers = new Headers(init.headers); // Headers 实例（spec，R2977）；fill guard none（Set-Cookie 存）
    // R3222/R3223：response guard（Fetch §6.2 step 13，fill 后设）——get/has/iterate 不暴露 Set-Cookie/Set-Cookie2，
    // append/set/delete 写侧阻断（§5.2），仅 getSetCookie 返数组（spec 特例）。
    this.headers._guard = 'response';
    this.type = 'default';
    this.url = '';
    this.redirected = false;
    // R3021：Uint8Array body（二进制 response）→ 存 _bodyBytes，_bodyText = TextDecoder 解码（供 text()）；
    // 字符串/其他 body → _bodyText 原样，_bodyBytes=null（blob()/arrayBuffer() 回落 UTF-8 编码文本）。
    if (body instanceof Uint8Array) {
      this._bodyBytes = body;
      this._bodyText = new TextDecoder().decode(body);
    } else {
      this._bodyBytes = null;
      this._bodyText = body == null ? '' : String(body);
    }
    var self = this;
    // body 为 ReadableStream（lazy，单 chunk + close，复用 _bodyToStream）。二进制 body 时 chunk 为 _bodyBytes
    // 字节；文本 body 同 R2967（UTF-8 文本 chunk）。
    Object.defineProperty(this, 'body', {
      get: function () { if (!self._bs) self._bs = _bodyToStream(self._bodyBytes != null ? self._bodyBytes : self._bodyText); return self._bs; },
      configurable: true
    });
    this.text = function () { return Promise.resolve(self._bodyText); };
    this.json = function () { return Promise.resolve(JSON.parse(self._bodyText)); };
    // R2978/R3021：补全 Response body-consumption 表面（spec：text/json/blob/arrayBuffer/formData）。
    // blob()：body 包成 Blob（二进制 body 用 _bodyBytes 字节保真）；arrayBuffer()：二进制 body 返 _bodyBytes，
    // 文本 body UTF-8 编码；formData()：application/x-www-form-urlencoded 解析。
    this.blob = function () { return Promise.resolve(new Blob([self._bodyBytes != null ? self._bodyBytes : self._bodyText])); };
    this.arrayBuffer = function () {
      if (self._bodyBytes != null) {
        var cp = new Uint8Array(self._bodyBytes.length);
        for (var j = 0; j < self._bodyBytes.length; j++) cp[j] = self._bodyBytes[j];
        return Promise.resolve(cp);
      }
      var bytes = _zw_utf8_encode(self._bodyText);
      var arr = new Uint8Array(bytes.length);
      for (var k = 0; k < bytes.length; k++) arr[k] = bytes[k];
      return Promise.resolve(arr);
    };
    this.formData = function () { return Promise.resolve(_zwParseFormUrlencoded(self._bodyText)); };
    this.clone = function () {
      // R3021：二进制 body（_bodyBytes）须克隆保真，否则 clone().arrayBuffer() 退化为文本 UTF-8 编码。
      var bodyArg = self._bodyBytes != null ? self._bodyBytes : self._bodyText;
      return new Response(bodyArg, { status: self.status, statusText: self.statusText, headers: self.headers });
    };
  };
  // R2968 Request：`new Request(url|request, init)`。fetch(input) 既接受 string 也接受 Request-like
  //（读 .url/.method/.headers/.body），故 Request 字段对齐 fetch 消费路径（body 为 string|null，非 stream；
  // R2977 headers 为 Headers 实例，同 Response）。clone() 复制自身。R2982 补 body 消费表面
  //（text/json/blob/arrayBuffer/formData，对称 Response R2978）。
  globalThis.Request = function Request(input, init) {
    if (!(this instanceof Request)) return new Request(input, init);
    init = init || {};
    var isObj = input && typeof input === 'object';
    this.url = isObj ? String(input.url || '') : String(input);
    this.method = String(init.method || (isObj ? input.method : '') || 'GET').toUpperCase();
    // R3223：request guard（Fetch §6.3 step 31-32）——guard 先于 fill 设，append 过滤禁止请求头
    //（Host/Content-Length/Cookie/Sec-*/Proxy-* 等不在 request.headers 暴露；闭合 R3222 已知限①）。
    this.headers = new Headers();
    this.headers._guard = 'request';
    _fillHeaders(this.headers, init.headers != null ? init.headers : (isObj ? input.headers : null));
    this.body = init.body != null ? String(init.body) : (isObj && input.body != null ? String(input.body) : null);
    this.cache = init.cache || 'default';
    this.mode = init.mode || 'cors';
    this.redirect = init.redirect || 'follow';
    this.credentials = init.credentials || 'same-origin';
    // R3045：Request.signal（spec 恒为 AbortSignal，非 null）。init.signal 优先；否则继承 input（Request）的 signal；
    // 否则新建非 aborted AbortSignal。fetch(new Request(url,{signal})) 经此透传 signal 给 R3044 abort 路径。
    // 注：复用同一 signal 对象（非 spec clone 独立）——同 request 多次 fetch 共享 signal，pragmatic（documented）。
    if (typeof AbortSignal === 'function') {
      this.signal = (init.signal instanceof AbortSignal)
        ? init.signal
        : ((isObj && input.signal instanceof AbortSignal) ? input.signal : new AbortSignal());
    } else {
      this.signal = null;
    }
    // R2982：body 消费表面（对称 Response R2978，spec text/json/blob/arrayBuffer/formData）。fetch 包装库 /
    // service worker fetch handler / 请求拦截器 / 测试 mock 读请求体高频。body 为 string|null：null（GET 无体）
    // → text() 返 ''、arrayBuffer() 长度 0；json() 解析空串抛 SyntaxError（spec，非合法 JSON）。
    var self = this;
    this.text = function () { return Promise.resolve(self.body == null ? '' : String(self.body)); };
    this.json = function () { return Promise.resolve(JSON.parse(self.body == null ? '' : String(self.body))); };
    this.blob = function () { return Promise.resolve(new Blob([self.body == null ? '' : String(self.body)])); };
    this.arrayBuffer = function () {
      var bytes = _zw_utf8_encode(self.body == null ? '' : String(self.body));
      var arr = new Uint8Array(bytes.length);
      for (var k = 0; k < bytes.length; k++) arr[k] = bytes[k];
      return Promise.resolve(arr);
    };
    this.formData = function () { return Promise.resolve(_zwParseFormUrlencoded(self.body)); };
  };
  globalThis.Request.prototype.clone = function () {
    return new Request(this.url, { method: this.method, headers: this.headers, body: this.body });
  };

  // P1b S5：setTimeout/setInterval 真实延迟。host（browser/renderer js_worker）注册
  // `__zw_setTimeout(id, delayMs)` 时，回调存 `__zw_pending[id]` + 调本回调；host 子线程
  // sleep 后 resolve → `__zwResolveCallback` 取出调用回调。未注册（engine/reftest/polyfill
  // 等无 host 路径）时 fallback `_defer`（microtask 同步触发）——保持旧行为，零回归。
  function _timerIdKey(handle) { return '__zwtid:' + handle; }
  globalThis.setTimeout = function(fn, delay) {
    var handle = _timerId++;
    if (typeof fn !== 'function') return handle;
    var id = _timerIdKey(handle);
    globalThis.__zw_pending[id] = function() { try { fn(); } catch (_e) {} };
    if (typeof __zw_setTimeout === 'function') {
      try { __zw_setTimeout(id, delay | 0); return handle; }
      catch (_e) { delete globalThis.__zw_pending[id]; }
    }
    // fallback：无 host → microtask 同步触发（旧行为）。
    delete globalThis.__zw_pending[id];
    _defer(fn);
    return handle;
  };
  globalThis.setInterval = function(fn, delay) {
    var handle = _timerId++;
    if (typeof fn !== 'function') return handle;
    var id = _timerIdKey(handle);
    var ms = delay | 0;
    if (typeof __zw_setTimeout === 'function') {
      // host 路径：回调内 re-arm 实现重复触发（host 仅实现单次定时器）。
      var arm = function() {
        globalThis.__zw_pending[id] = function() {
          try { fn(); } catch (_e) {}
          arm();
        };
        try { __zw_setTimeout(id, ms); }
        catch (_e) { delete globalThis.__zw_pending[id]; }
      };
      arm();
    } else {
      // fallback（无 host）：保持旧行为——单次 _defer 触发（非重复）。
      _defer(fn);
    }
    return handle;
  };
  // clearTimeout/clearInterval：删 pending 项——即便 host 子线程后到 resolve，
  // `__zwResolveCallback` 见无 pending 即 no-op（setInterval 的 re-arm 链亦在此断开）。
  globalThis.clearTimeout = function(handle) {
    delete globalThis.__zw_pending[_timerIdKey(handle)];
  };
  globalThis.clearInterval = function(handle) {
    delete globalThis.__zw_pending[_timerIdKey(handle)];
  };
  // requestIdleCallback/cancelIdleCallback：镜像 setTimeout 机制（host __zw_setTimeout + pending 表；
  // 无 host → _defer 微任务，同 setTimeout fallback）。回调传 IdleDeadline（didTimeout/timeRemaining
  // 近似——真实 idle 时序须 event-loop 帧 tick 切片，此为基础可用实现，防 ReferenceError + 延迟执行）。
  function _ricIdKey(handle) { return '__zwric:' + handle; }
  globalThis.requestIdleCallback = function(fn, options) {
    var handle = _timerId++;
    if (typeof fn !== 'function') return handle;
    var deadline = { didTimeout: false, timeRemaining: function() { return 50; } };
    var id = _ricIdKey(handle);
    globalThis.__zw_pending[id] = function() { try { fn(deadline); } catch (_e) {} };
    if (typeof __zw_setTimeout === 'function') {
      try { __zw_setTimeout(id, (options && options.timeout) | 0); return handle; }
      catch (_e) { delete globalThis.__zw_pending[id]; }
    }
    // fallback（无 host）：微任务同步触发（同 setTimeout fallback）。
    delete globalThis.__zw_pending[id];
    _defer(function() { try { fn(deadline); } catch (_e) {} });
    return handle;
  };
  globalThis.cancelIdleCallback = function(handle) {
    delete globalThis.__zw_pending[_ricIdKey(handle)];
  };

  // ── P1b S2 incr1/incr2：MutationObserver（JS 侧拦截 + microtask 派发）──
  // 节点身份用「复合 key」：handle-based（JS 创建子树，`createElement` 返 `"__n{n}"`）+
  // selector-based（现有 DOM，`querySelector` 返 `_makeProxy(sel, null)`）。`_mo_id(handle, sel)`
  // 优先 handle，否则 sel——v8::External 真 object identity（===）非功能必需（RFC 纠正）。
  // `observe(target, options)` 注册 id→options；`_makeProxy` 的 setAttribute/appendChild/etc.
  // 调 `_mo_notify(sel, handle, record)` 排队；`_defer`（microtask）派发回调（spec §4 语义）。
  // incr1 = handle（JS 子树）；incr2 = +selector（现有 DOM）。支持 attributes + childList。
  // 限制：仅观测 JS 驱动的 mutation（host 侧 `__zw_dispatch_event` 等不触发）。
  globalThis.__zw_mo_observers = globalThis.__zw_mo_observers || [];
  var _moFlushScheduled = false;

  // 元素身份 key——handle 优先（JS 创建节点），否则 selector（现有 DOM）。
  function _mo_id(handle, sel) {
    if (handle != null) return 'h:' + handle;
    if (sel) return 's:' + sel;
    return null;
  }

  // R3025：observer options 是否请求属性 oldValue（spec：attributeOldValue=true 或 attributeFilter 命中该属性）。
  function _mo_obs_wants_attr_old(opts, name) {
    // R49 修正：oldValue 仅在 attributeOldValue === true 时提供（WPT attributes 用例
    // "attributeFilter alone ... update mutation" 期望 oldValue null——filter 只筛 record
    // 不隐含 oldValue；spec `mutation-observer-observe` attributeOldValue 独立开关）。
    if (opts.attributeOldValue === true) return true;
    void name;
    return false;
  }
  // 任意观测该 id 的 observer 是否需要 name 的 oldValue——决定 attribute call site 是否在 mutate 前捕获 old value
  //（无 observer 需要时不读 host，避 setAttribute 热路径无谓 get 开销）。
  function _mo_any_wants_attr_old(id, name) {
    if (id == null) return false;
    var observers = globalThis.__zw_mo_observers;
    for (var i = 0; i < observers.length; i++) {
      var obs = observers[i];
      var opts = obs._targets[id];
      if (opts && _mo_obs_wants_attr_old(opts, name)) return true;
    }
    return false;
  }
  // 读属性当前值（mutate 前的 old value）。复用 getAttribute/hasAttribute 同款 host 回调（handle/sel latest-wins）。
  // 先 hasAttribute 判存在——host get_attr 对 absent 可能返 ''（非 null），须显式判 present 返 null（spec：absent oldValue=null）。
  function _mo_read_attr(sel, handle, name) {
    try {
      var present = false;
      if (handle && typeof __zw_has_attr_handle === 'function') present = __zw_has_attr_handle(handle, name) === '1';
      else if (sel && typeof __zw_has_attr_lw === 'function') present = __zw_has_attr_lw(sel, name) === '1';
      else if (sel && typeof __zw_has_attr === 'function') present = __zw_has_attr(sel, name) === '1';
      if (!present) return null;
      if (handle && typeof __zw_get_attr_handle === 'function') return __zw_get_attr_handle(handle, name);
      if (sel && typeof __zw_get_attr_lw === 'function') return __zw_get_attr_lw(sel, name);
      if (sel && typeof __zw_get_attr === 'function') return __zw_get_attr(sel, name);
    } catch (_e) {}
    return null;
  }
  // R3028：observer options 是否请求 characterData oldValue（spec：characterDataOldValue=true）。
  function _mo_obs_wants_char_old(opts) {
    return opts.characterDataOldValue === true;
  }
  // 任意观测该 id 的 observer 是否需要 characterData old value——决定 textContent mutate 前是否捕获 old 文本
  //（无 observer 需要时不读 host，避 textContent= 热路径无谓 get 开销，镜像 _mo_any_wants_attr_old）。
  function _mo_any_wants_char_old(id) {
    if (id == null) return false;
    var observers = globalThis.__zw_mo_observers;
    for (var i = 0; i < observers.length; i++) {
      var obs = observers[i];
      var opts = obs._targets[id];
      if (opts && _mo_obs_wants_char_old(opts)) return true;
    }
    return false;
  }
  // 读元素当前文本（mutate 前的 old value）。handle 走 mutation replay（query_text_from_mutations），
  // sel 走 latest-wins（__zw_get_text_lw，R3028 闭合 textContent= 后 stale 旧值）；回调缺失 → null（deliver 侧判）。
  function _mo_read_text(sel, handle) {
    try {
      if (handle && typeof __zw_get_text_handle === 'function') return __zw_get_text_handle(handle);
      if (sel && typeof __zw_get_text_lw === 'function') return __zw_get_text_lw(sel);
      if (sel && typeof __zw_get_text === 'function') return __zw_get_text(sel);
    } catch (_e) {}
    return null;
  }
  // 把一条 mutation 记录投递给观测该 id 且 options 匹配的 observer。
  // requireSubtree=true（祖先 id 路径）时仅投递 opts.subtree===true 的 observer（spec：subtree 才接收后代 mutation）。
  // 每个 observer 拿独立 record 副本（target 指向各自 observe() 时的 proxy）。
  function _mo_deliverToId(id, baseRecord, requireSubtree) {
    if (id == null) return;
    var observers = globalThis.__zw_mo_observers;
    for (var i = 0; i < observers.length; i++) {
      var obs = observers[i];
      var opts = obs._targets[id];
      if (!opts) continue;
      if (requireSubtree && !opts.subtree) continue; // R3026：祖先 id 须 subtree observer
      if (baseRecord.type === 'attributes') {
        if (!opts.attributes) continue;
        // R3025：attributeFilter——仅观测列表内属性（spec：attributeFilter 非 attributeOldValue 时为筛选条件）。
        if (Array.isArray(opts.attributeFilter) && opts.attributeFilter.indexOf(baseRecord.attributeName) < 0) continue;
      }
      if (baseRecord.type === 'childList' && !opts.childList) continue;
      if (baseRecord.type === 'characterData' && !opts.characterData) continue;
      var rec = Object.create(globalThis.MutationRecord.prototype);
      rec.type = baseRecord.type;
      // R49：characterData record 的 target 是**文本节点自身**（spec；call site baseRecord.target
      // 携带——R48 parsed 文本编辑 / R49 textContent= 后 firstChild.data= 场景），其余类型 target=
      // 观测元素 proxy。
      rec.target = baseRecord.type === 'characterData' && baseRecord.target != null
        ? baseRecord.target
        : obs._targetProxies[id];
      // spec 字段：addedNodes/removedNodes 缺省 []（类数组），sibling/attributeNamespace/oldValue 缺省 null。
      rec.addedNodes = baseRecord.addedNodes || [];
      rec.removedNodes = baseRecord.removedNodes || [];
      rec.previousSibling = baseRecord.previousSibling || null;
      rec.nextSibling = baseRecord.nextSibling || null;
      rec.attributeName = baseRecord.attributeName || null;
      rec.attributeNamespace = baseRecord.attributeNamespace || null;
      // R3025/R3028：oldValue 仅当 observer 请求时填——attributes: attributeOldValue 或 attributeFilter 命中；
      // characterData: characterDataOldValue；childList 恒 null。call site 已按 observer 需求捕获 baseRecord.oldValue。
      var _wantsOld = baseRecord.type === 'attributes'
        ? _mo_obs_wants_attr_old(opts, baseRecord.attributeName)
        : (baseRecord.type === 'characterData' ? _mo_obs_wants_char_old(opts) : false);
      rec.oldValue = _wantsOld ? (baseRecord.oldValue != null ? baseRecord.oldValue : null) : null;
      obs._records.push(rec);
      _mo_scheduleFlush();
    }
  }
  // R3026：任意 observer 是否用了 subtree（决定 mutation 时是否走 ancestor 链——无 subtree observer 时零开销）。
  function _mo_any_subtree() {
    var observers = globalThis.__zw_mo_observers;
    for (var i = 0; i < observers.length; i++) {
      var targets = observers[i]._targets;
      for (var k in targets) {
        if (targets[k] && targets[k].subtree) return true;
      }
    }
    return false;
  }
  // 把一条 mutation 记录投递：精确 id observer + subtree 祖先 observer（R3026）。
  // R49：全局暴露口——part06 顶层的 _zwRegisterTextEl 文本节点（textContent=/innerHTML= 建的
  // 本地视图）编辑时发 characterData record（_mo_id/_mo_notify 为本 IIFE 私有，跨 part 不可见）。
  globalThis.__zw_mo_notify_text = function (sel, targetNode, oldValue) {
    _mo_notify(sel, null, { type: 'characterData', oldValue: oldValue, target: targetNode });
  };

  function _mo_notify(sel, handle, baseRecord) {
    var id = _mo_id(handle, sel);
    _mo_deliverToId(id, baseRecord, false); // 精确 id，不要求 subtree
    // R3026：subtree——mutation 冒泡到 subtree:true 的祖先 observer（record.target=祖先 proxy）。
    // 仅在有 subtree observer 且 sel-based（live DOM 有 __zw_parent 父链）时走祖先链；handle-only detached defer。
    if (sel && typeof __zw_parent === 'function' && _mo_any_subtree()) {
      var chain = _ancestorChain(sel); // [self, parent, ..., root]
      for (var k = 1; k < chain.length; k++) { // 跳过 self（chain[0]，精确 id 已投）
        _mo_deliverToId('s:' + chain[k], baseRecord, true);
      }
    }
    // js-dom M4 R50：childList mutation → live HTMLCollection 失效标记（本函数是 shim 全部
    // childList 记录的单一汇流点——part04 appendChild/removeChild/insertBefore/replaceChild/
    // insertAdjacent/textContent= 等 13 处均经此）。集合下次读取时 lazy 重查（_zwHCLiveInvalidate
    // 在 part05 定义，同一 IIFE 作用域；hoisting 使前向引用安全）。
    if (baseRecord && baseRecord.type === 'childList') {
      _zwHCLiveInvalidate(baseRecord.addedNodes, baseRecord.removedNodes);
    }
  }
  function _mo_scheduleFlush() {
    if (_moFlushScheduled) return;
    _moFlushScheduled = true;
    _defer(function() {
      _moFlushScheduled = false;
      var observers = globalThis.__zw_mo_observers;
      for (var i = 0; i < observers.length; i++) {
        var obs = observers[i];
        if (obs._records.length > 0) {
          var records = obs._records;
          obs._records = [];
          try { obs._callback(records, obs); } catch (_e) {}
        }
      }
    });
  }

  globalThis.MutationObserver = function(callback) {
    this._callback = callback;
    this._targets = {};       // id (h:handle / s:sel) -> options
    this._targetProxies = {}; // id -> observe() 时传入的 proxy（record.target 用）
    this._records = [];
    globalThis.__zw_mo_observers.push(this);
  };
  globalThis.MutationObserver.prototype.observe = function(target, options) {
    // js-dom M4 R49：spec `dom-mutationobserver-observe` 步骤 3-6 options 校验——①
    // childList/attributes/characterData 全 falsy 抛 TypeError；② attributeOldValue=true 而
    // attributes 非 true 抛；③ attributeFilter 存在而 attributes 非 true 抛（WPT
    // MutationObserver-sanity 三个 "Should throw"）。characterDataOldValue/characterData 同理
    //（spec 对称；WPT 同文件后续 subtest）。
    var o = options || {};
    // spec 步骤 3：attributeOldValue/attributeFilter/characterDataOldValue **存在**（非 undefined）
    // 即隐含启用 attributes/characterData 观测（WPT sanity "attributeOldValue:true (present)
    // auto-enables attribute observation" / "Should not throw if attributeOldValue is true and
    // attributes is omitted"）。先归一再校验。
    if (o.attributeOldValue !== undefined || o.attributeFilter !== undefined) {
      if (o.attributes === undefined) o.attributes = true;
    }
    if (o.characterDataOldValue !== undefined) {
      if (o.characterData === undefined) o.characterData = true;
    }
    if (!o.childList && !o.attributes && !o.characterData) {
      throw new globalThis.TypeError("MutationObserver: one of childList, attributes, or characterData must be true");
    }
    if (o.attributeOldValue === true && o.attributes !== true) {
      throw new globalThis.TypeError("MutationObserver: attributeOldValue true requires attributes true");
    }
    if (o.attributeFilter !== undefined && o.attributes !== true) {
      throw new globalThis.TypeError("MutationObserver: attributeFilter requires attributes true");
    }
    if (o.characterDataOldValue === true && o.characterData !== true) {
      throw new globalThis.TypeError("MutationObserver: characterDataOldValue true requires characterData true");
    }
    if (!target) return;
    var id = _mo_id(target.__zwHandle, target.__zwSelector);
    // js-dom M4 R48：parsed 文本/注释节点（_wrapNodeEntry 普通对象，无自身 sel/handle）——观测
    // 落到**父元素 id**（其 characterData 编辑 notify 发 s:parentSel，见 part05 _write）。target
    // proxy 仍记原文本节点（record.target 语义）。无父 sel 的纯快照节点不可观测（旧 no-op）。
    if (id == null && target.__zwIsText && target.parentNode && target.parentNode.__zwSelector) {
      id = 's:' + target.parentNode.__zwSelector;
    }
    if (id == null) return;
    this._targets[id] = options || {};
    this._targetProxies[id] = target;
  };
  globalThis.MutationObserver.prototype.disconnect = function() {
    this._targets = {};
    this._targetProxies = {};
  };
  globalThis.MutationObserver.prototype.takeRecords = function() {
    var r = this._records;
    this._records = [];
    return r;
  };
  // MutationRecord（R2847）：Web IDL 接口——回调收到的 record 须 `instanceof MutationRecord` +
  // `[object MutationRecord]` toStringTag + 完整 spec 字段（previousSibling/nextSibling/
  // attributeNamespace/oldValue 缺省 null，addedNodes/removedNodes 缺省 []）。库做
  // `record instanceof MutationRecord` 特征检测 / 读 record.previousSibling 须得 null 非 undefined。
  // 无公开构造器入参（字段由 _mo_notify 注入）；仅建 prototype + toStringTag 供 instanceof/序列化。
  globalThis.MutationRecord = function() {};
  globalThis.MutationRecord.prototype[Symbol.toStringTag] = 'MutationRecord';

  // ── P1a Slice 2a：IntersectionObserver（JS 侧，复用 gBCR layout-rect snapshot）──
  // 镜像 MutationObserver：纯 JS，`observe()` 排队 initial notification，经 `_defer`
  // （microtask）派发 `obs._callback(entries, observer)`。intersection 用 host
  // `__zw_getBoundingClientRect(sel)`（gBCR path C，已注册时返真实 rect）+ innerWidth/innerHeight
  // 计算与 root（默认 viewport）的重叠；threshold 越界检测决定是否派发。host 未注册
  // （reftest/polyfill/WebView 路径）→ target rect 为零 → isIntersecting=false，仍派发 initial
  // notification（no-throw，零回归）。旧 shim 完全无 IO → `new IntersectionObserver` 抛
  // ReferenceError **中断整个脚本**，本切片消除之（spec：observe 即排队一次 initial 通知）。
  // 限制（接受，follow-up）：① 仅 observe 时计算，非持续 host tick——scroll/resize/async-layout
  //   变化触发的后续通知为 Slice 2b（须 host render-loop tick 或 __zwResolveCallback 重算钩子）；
  // ② handle-identity（createElement）元素 sel 为空 → 零 rect（同 gBCR 限制，path A follow-up）；
  // ③ rootMargin px/% 已支持（R2966，CSS margin 简写展开/收缩 root rect，% 按 root 维度）；④ root 为元素时取其 selector rect。
  // R3319：DOMRectReadOnly + DOMRect 全局构造器（Geometry Interfaces spec §3）。
  // **此前 B-gen shim 缺**——getBoundingClientRect / getClientRects / IO·RO entry 的 contentRect /
  // boundingClientRect 全返**无原型 plain object**（无 DOMRect/DOMRectReadOnly 身份），库做鸭子类型
  // `rect instanceof DOMRectReadOnly` / `instanceof DOMRect` 恒 false（popper.js / floating-ui /
  // 测量库的 identity 检查失败）。参照 A-gen dom_bridge.rs DOMRectReadOnly stub，在 B-gen 补真实
  // prototype 链：DOMRectReadOnly（基类，spec §3.2）+ DOMRect（可读写子类，spec §3.3，gBCR 返回类型）。
  // 设计：prototype 上以 getter 派生 top/left/right/bottom（保持与 x/y/width/height 同步，spec 计算属性）；
  // _makeDomRect(x,y,w,h) 工厂返 `new DOMRect(...)`（共享原型 → instanceof 成立）。三处 plain-object
  // rect 工厂（_io_domRect / _domRectFromId / gBCR 零 fallback）迁移到 _makeDomRect。
  // https://drafts.fxtf.org/geometry/#DOMRect
  function _zwDomRectProto(ReadOnly) {
    var p = {};
    // 派生属性（getter，与 x/y/width/height 实例字段同步——spec §3.2 计算属性）。
    Object.defineProperty(p, 'top', { get: function () { return this.y; }, enumerable: true });
    Object.defineProperty(p, 'left', { get: function () { return this.x; }, enumerable: true });
    Object.defineProperty(p, 'right', { get: function () { return this.x + this.width; }, enumerable: true });
    Object.defineProperty(p, 'bottom', { get: function () { return this.y + this.height; }, enumerable: true });
    p.toJSON = function () {
      return { x: this.x, y: this.y, top: this.y, left: this.x,
               right: this.x + this.width, bottom: this.y + this.height,
               width: this.width, height: this.height };
    };
    return p;
  }
  // DOMRectReadOnly：spec §3.2，4 数值字段 + 4 派生 + toJSON。
  function DOMRectReadOnly(x, y, width, height) {
    this.x = +x || 0; this.y = +y || 0; this.width = +width || 0; this.height = +height || 0;
  }
  DOMRectReadOnly.prototype = _zwDomRectProto(true);
  // DOMRect：spec §3.3，继承 DOMRectReadOnly（gBCR / getClientRects 返回类型，可读写）。
  function DOMRect(x, y, width, height) {
    this.x = +x || 0; this.y = +y || 0; this.width = +width || 0; this.height = +height || 0;
  }
  // DOMRect 继承 DOMRectReadOnly prototype（spec is-a 关系，`new DOMRect() instanceof DOMRectReadOnly` 成立）。
  DOMRect.prototype = Object.create(DOMRectReadOnly.prototype);
  // 保持 constructor 指向 DOMRect（Object.create 后修正）。
  Object.defineProperty(DOMRect.prototype, 'constructor', { value: DOMRect, writable: true, configurable: true });
  globalThis.DOMRectReadOnly = globalThis.DOMRectReadOnly || DOMRectReadOnly;
  globalThis.DOMRect = globalThis.DOMRect || DOMRect;
  // 共享 rect 工厂：返 `new DOMRect(...)`（共享原型 → instanceof DOMRect / DOMRectReadOnly 成立）。
  function _makeDomRect(x, y, w, h) {
    return new DOMRect(x, y, w, h);
  }
  // 兼容旧名（IO/RO 内部仍调 _io_domRect，现委托 _makeDomRect）。
  function _io_domRect(x, y, w, h) {
    return _makeDomRect(x, y, w, h);
  }
  // 读 target/root 的 rect（复用 gBCR）；identity = selector 或 handle（path A）。
  // 空 / handler 未注册 / 未命中 → 零 rect。
  function _io_rectFromSel(identity) {
    if (identity && typeof __zw_getBoundingClientRect === 'function') {
      try {
        var s = __zw_getBoundingClientRect(identity);
        if (s && s.indexOf(',') >= 0) {
          var p = s.split(',');
          return { x: +p[0], y: +p[1], w: +p[2], h: +p[3] };
        }
      } catch (_e) {}
    }
    return { x: 0, y: 0, w: 0, h: 0 };
  }
  function _io_intersect(a, b) {
    var x0 = Math.max(a.x, b.x), y0 = Math.max(a.y, b.y);
    var x1 = Math.min(a.x + a.w, b.x + b.w), y1 = Math.min(a.y + a.h, b.y + b.h);
    if (x1 <= x0 || y1 <= y0) return { x: 0, y: 0, w: 0, h: 0 };
    return { x: x0, y: y0, w: x1 - x0, h: y1 - y0 };
  }
  // 归一化 threshold：number | number[] → 升序去重、clamp 到 [0,1] 的数组（空→[0]）。
  function _io_normThresholds(threshold) {
    var arr = [];
    if (typeof threshold === 'number') {
      arr = [threshold];
    } else if (Object.prototype.toString.call(threshold) === '[object Array]') {
      for (var i = 0; i < threshold.length; i++) {
        if (typeof threshold[i] === 'number') arr.push(threshold[i]);
      }
    }
    if (arr.length === 0) arr = [0];
    arr.sort(function(a, b) { return a - b; });
    var uniq = [];
    for (var j = 0; j < arr.length; j++) {
      var v = arr[j];
      if (v < 0) v = 0; else if (v > 1) v = 1;
      if (uniq.length === 0 || uniq[uniq.length - 1] !== v) uniq.push(v);
    }
    return uniq;
  }
  function _io_id(handle, sel) {
    if (handle != null) return 'h:' + handle;
    if (sel) return 's:' + sel;
    return null;
  }
  // 解析 rootMargin 串（CSS margin shorthand）→ 4 个 {val, pct} 部分（top/right/bottom/left）。
  // R2966：rootMargin 此前按 0 处理（defer）。px 直取并标记 pct=false；% 标记 pct=true（compute 时按
  // root 维度展开：top/bottom→root 高，left/right→root 宽，spec §2.1）。其它单位/非法值 → 0（spec：
  // rootMargin 仅支持 <length>/<percentage>，fail-to-parse 视为 0）。1-4 值按 CSS margin 简写展开。
  function _io_parseRootMargin(str) {
    var raw = (typeof str === 'string' ? str : '').trim().split(/\s+/).filter(function (s) { return s.length > 0; });
    if (raw.length === 0) raw = ['0px', '0px', '0px', '0px'];
    else if (raw.length === 1) raw = [raw[0], raw[0], raw[0], raw[0]];
    else if (raw.length === 2) raw = [raw[0], raw[1], raw[0], raw[1]];
    else if (raw.length === 3) raw = [raw[0], raw[1], raw[2], raw[1]];
    var norm = function (s) {
      var m = /^(-?\d+(?:\.\d+)?)(px|%)?$/.exec(String(s).trim());
      if (!m) return { val: 0, pct: false };
      return { val: parseFloat(m[1]) || 0, pct: m[2] === '%' };
    };
    return [norm(raw[0]), norm(raw[1]), norm(raw[2]), norm(raw[3])];
  }
  // 按 rootMargin 4 部分展开/收缩 root rect（负 margin 收缩）。% 按 root 自身维度展开（compute 时 rootRect
  // 已知）。返回新 rect（不改原）。零 margin（默认）→ 原样返回（零回归既有 IO 行为）。
  function _io_applyRootMargin(rootRect, margins) {
    var resolve = function (part, dim) { return part.pct ? (part.val / 100) * dim : part.val; };
    var top = resolve(margins[0], rootRect.h);
    var right = resolve(margins[1], rootRect.w);
    var bottom = resolve(margins[2], rootRect.h);
    var left = resolve(margins[3], rootRect.w);
    return { x: rootRect.x - left, y: rootRect.y - top, w: rootRect.w + left + right, h: rootRect.h + top + bottom };
  }
  globalThis.IntersectionObserver = function(callback, options) {
    this._callback = callback;
    var opts = options || {};
    this._thresholds = _io_normThresholds(opts.threshold);
    // root：null（默认 viewport）或元素（取其 __zwSelector 的 rect）。
    this._rootSel = (opts.root && opts.root.__zwSelector) ? opts.root.__zwSelector : null;
    // R2966：rootMargin（CSS margin shorthand，px/%），compute 时展开/收缩 root rect。
    this._rootMargins = _io_parseRootMargin(opts.rootMargin);
    this._targets = {};        // id (h:handle / s:sel) -> { proxy }
    this._lastRatio = {};      // id -> 上次派发的 ratio（undefined = 未派发过 → initial）
    this._scheduled = false;
    _zwObservers.push(this);   // P1a Slice 2b：注册到 tick 表
  };
  // 计算单个 target 的 intersection 数据（rect / ratio / isIntersecting）。
  globalThis.IntersectionObserver.prototype._compute = function(id) {
    var t = this._targets[id];
    if (!t) return null;
    var sel = t.proxy.__zwSelector;
    var rootRect = this._rootSel
      ? _io_rectFromSel(this._rootSel)
      : { x: 0, y: 0, w: globalThis.innerWidth | 0, h: globalThis.innerHeight | 0 };
    // R2966：rootMargin 展开/收缩 root rect（% 按 root 自身维度）。零 margin（默认）原样。
    rootRect = _io_applyRootMargin(rootRect, this._rootMargins);
    // path A：sel 空（createElement 元素）时用 handle，host 查 handle→selector map 解析。
    var targetRect = _io_rectFromSel(sel || t.proxy.__zwHandle);
    var inter = _io_intersect(targetRect, rootRect);
    var targetArea = targetRect.w * targetRect.h;
    var ratio = targetArea > 0 ? (inter.w * inter.h) / targetArea : 0;
    return { target: t.proxy, targetRect: targetRect, rootRect: rootRect, inter: inter, ratio: ratio, isIntersecting: inter.w > 0 && inter.h > 0 };
  };
  // threshold 越界检测：未派发过（initial）或 ratio 与上次跨过任一 threshold 边界。
  globalThis.IntersectionObserver.prototype._crossed = function(id, ratio) {
    var prev = this._lastRatio[id];
    if (prev == null) return true;
    for (var i = 0; i < this._thresholds.length; i++) {
      var th = this._thresholds[i];
      if ((prev >= th) !== (ratio >= th)) return true;
    }
    return false;
  };
  // 排队一次 microtask 派发：遍历所有 target，对越阈值的构造 entry 投递 callback。
  globalThis.IntersectionObserver.prototype._schedule = function() {
    if (this._scheduled) return;
    this._scheduled = true;
    var self = this;
    _defer(function() {
      self._scheduled = false;
      var entries = [];
      for (var id in self._targets) {
        var c = self._compute(id);
        if (!c) continue;
        if (self._crossed(id, c.ratio)) {
          entries.push({
            time: 0,
            target: c.target,
            rootBounds: _io_domRect(c.rootRect.x, c.rootRect.y, c.rootRect.w, c.rootRect.h),
            boundingClientRect: _io_domRect(c.targetRect.x, c.targetRect.y, c.targetRect.w, c.targetRect.h),
            intersectionRect: _io_domRect(c.inter.x, c.inter.y, c.inter.w, c.inter.h),
            intersectionRatio: c.ratio,
            isIntersecting: c.isIntersecting,
            toJSON: function() { return this; }
          });
          self._lastRatio[id] = c.ratio;
        }
      }
      if (entries.length > 0) {
        try { self._callback(entries, self); } catch (_e) {}
      }
    });
  };
  globalThis.IntersectionObserver.prototype.observe = function(target) {
    if (!target) return this;
    var id = _io_id(target.__zwHandle, target.__zwSelector);
    if (id != null) {
      this._targets[id] = { proxy: target };
      this._schedule();
    }
    return this;
  };
  globalThis.IntersectionObserver.prototype.unobserve = function(target) {
    if (!target) return this;
    var id = _io_id(target.__zwHandle, target.__zwSelector);
    if (id != null) {
      delete this._targets[id];
      delete this._lastRatio[id];
    }
    return this;
  };
  globalThis.IntersectionObserver.prototype.disconnect = function() {
    this._targets = {};
    this._lastRatio = {};
    return this;
  };
  globalThis.IntersectionObserver.prototype.takeRecords = function() {
    return [];
  };
  // IntersectionObserverEntry：兼容构造（部分脚本 `new IntersectionObserverEntry(init)`）。
  globalThis.IntersectionObserverEntry = function(init) {
    init = init || {};
    this.time = init.time || 0;
    this.rootBounds = init.rootBounds || null;
    this.boundingClientRect = init.boundingClientRect || null;
    this.intersectionRect = init.intersectionRect || null;
    this.isIntersecting = init.isIntersecting || false;
    this.target = init.target || null;
    this.intersectionRatio = init.intersectionRatio || 0;
  };

  // ── P1a Slice 3：ResizeObserver（JS 侧，复用 gBCR layout-rect snapshot）──
  // 镜像 IntersectionObserver：纯 JS，`observe()` 排队 initial notification，经 `_defer`
  // （microtask）派发 `obs._callback(entries, observer)`。size 取 host `__zw_getBoundingClientRect(sel)`
  // （gBCR path C，直接复用 IO 的 `_io_rectFromSel`/`_io_domRect`/`_io_id` rect 辅助）；
  // size-diff 检测决定是否派发——首次（无 last）=initial 必派发，之后仅宽高变化才派发（spec §4 语义）。
  // host 未注册（reftest/polyfill/WebView 路径）→ contentRect 为零，仍派发 initial notification
  // （no-throw，零回归）。旧 shim 完全无 RO → `new ResizeObserver` 抛 ReferenceError 中断整个脚本
  // （与 IO 同），本切片消除之。
  // 限制（接受，follow-up）：① 仅 observe 时计算，非持续 host tick——resize/async-layout 变化触发的
  //   后续通知为 Slice 2b（与 IO 同，须 host render-loop tick 或 __zwResolveCallback 重算钩子）；
  // ② R2972：contentRect/contentBoxSize/devicePixelContentBoxSize 经 getComputedStyle 真值扣除 padding +
  //   border-width → content-box（borderBoxSize 仍 border-box = gBCR）。host 未注册/属性未覆盖 → 0 扣除
  //   → content = border（fallback，同旧近似行为）。
  // R2972：读计算样式 box-model 像素值（"10px" → 10，未注册/非 px → 0）供 RO content-box 扣除。
  function _ro_px(cs, prop) {
    if (!cs || typeof cs.getPropertyValue !== 'function') return 0;
    var m = /^(-?\d+(?:\.\d+)?)px$/.exec(String(cs.getPropertyValue(prop) || '').trim());
    return m ? parseFloat(m[1]) : 0;
  }
  globalThis.ResizeObserver = function(callback) {
    this._callback = callback;
    this._targets = {};       // id (h:handle / s:sel) -> { proxy }
    this._lastSize = {};      // id -> {w,h}（undefined = 未派发过 → initial）
    this._scheduled = false;
    _zwObservers.push(this);  // P1a Slice 2b：注册到 tick 表
  };
  // 排队一次 microtask 派发：遍历所有 target，对尺寸变化（或 initial）的构造 entry 投递 callback。
  globalThis.ResizeObserver.prototype._schedule = function() {
    if (this._scheduled) return;
    this._scheduled = true;
    var self = this;
    _defer(function() {
      self._scheduled = false;
      var entries = [];
      for (var id in self._targets) {
        var t = self._targets[id];
        // path A：sel 空（createElement 元素）时用 handle。
        var r = _io_rectFromSel(t.proxy.__zwSelector || t.proxy.__zwHandle);
        var prev = self._lastSize[id];
        // initial（prev==null）或宽高变化 → 派发并更新 last。
        if (prev == null || prev.w !== r.w || prev.h !== r.h) {
          self._lastSize[id] = { w: r.w, h: r.h };
          // R2972：box-model 真值扣除。gBCR rect = border-box（含 padding+border）；content-box =
          // border-box - padding - border-width（经 getComputedStyle 真值，host 未覆盖 → 0 = 不扣除）。
          var cs = globalThis.getComputedStyle ? globalThis.getComputedStyle(t.proxy) : null;
          var pT = _ro_px(cs, 'padding-top'), pR = _ro_px(cs, 'padding-right'),
              pB = _ro_px(cs, 'padding-bottom'), pL = _ro_px(cs, 'padding-left');
          var bT = _ro_px(cs, 'border-top-width'), bR = _ro_px(cs, 'border-right-width'),
              bB = _ro_px(cs, 'border-bottom-width'), bL = _ro_px(cs, 'border-left-width');
          var cW = Math.max(0, r.w - pL - pR - bL - bR);
          var cH = Math.max(0, r.h - pT - pB - bT - bB);
          entries.push({
            target: t.proxy,
            // contentRect = content-box rect（spec；origin = border-box origin + border + padding）。
            contentRect: _io_domRect(r.x + bL + pL, r.y + bT + pT, cW, cH),
            // borderBoxSize = border-box（gBCR）；contentBoxSize/devicePixelContentBoxSize = content-box。
            borderBoxSize: [{ inlineSize: r.w, blockSize: r.h }],
            contentBoxSize: [{ inlineSize: cW, blockSize: cH }],
            devicePixelContentBoxSize: [{ inlineSize: cW, blockSize: cH }],
            toJSON: function() { return this; }
          });
        }
      }
      if (entries.length > 0) {
        try { self._callback(entries, self); } catch (_e) {}
      }
    });
  };
  globalThis.ResizeObserver.prototype.observe = function(target) {
    if (!target) return this;
    var id = _io_id(target.__zwHandle, target.__zwSelector);
    if (id != null) {
      // 已观察的 target 重复 observe：spec 视为 no-op（不重置 last），但 _schedule 的 size-diff
      // 检测会在 layout 变化时自然派发（last 保留上次派发尺寸）。
      this._targets[id] = { proxy: target };
      this._schedule();
    }
    return this;
  };
  globalThis.ResizeObserver.prototype.unobserve = function(target) {
    if (!target) return this;
    var id = _io_id(target.__zwHandle, target.__zwSelector);
    if (id != null) {
      delete this._targets[id];
      delete this._lastSize[id];
    }
    return this;
  };
  globalThis.ResizeObserver.prototype.disconnect = function() {
    this._targets = {};
    this._lastSize = {};
    return this;
  };
  globalThis.ResizeObserver.prototype.takeRecords = function() {
    return [];
  };
  // ResizeObserverEntry：兼容构造（部分脚本 `new ResizeObserverEntry(init)`）。
  globalThis.ResizeObserverEntry = function(init) {
    init = init || {};
    this.target = init.target || null;
    this.contentRect = init.contentRect || null;
    this.borderBoxSize = init.borderBoxSize || null;
    this.contentBoxSize = init.contentBoxSize || null;
    this.devicePixelContentBoxSize = init.devicePixelContentBoxSize || null;
  };

  // P1a Slice 2b：host render（snapshot 已填真实 rect）后调本函数，对每个活跃 observer 调
  // `_schedule()` 复算——IO `_crossed`（threshold 越界）/ RO size-diff 仅在变化时派发，故收敛。
  // 跳过无活跃 target 的 observer（disconnect/unobserve-all 后 no-op）。`_defer` microtask 在
  // 本次 execute 末尾 checkpoint drain，回调同步触发；回调内 DOM mutation 由 host apply+rerender。
  globalThis.__zw_observers_tick = function() {
    for (var i = 0; i < _zwObservers.length; i++) {
      var obs = _zwObservers[i];
      if (!obs || !obs._targets) continue;
      var has = false;
      for (var _k in obs._targets) { has = true; break; }
      if (has) {
        try { obs._schedule(); } catch (_e) {}
      }
    }
  };

  // https://w3c.github.io/input-events/#input-event-order-during-user-initiated-editing
  // host 在 keydown 默认动作阶段调用：先派 cancelable beforeinput；未取消才更新 value/selection，
  // 再派不可取消 input。非 input/textarea 目标 → no-op。
  globalThis.__zw_text_input = function(sel, ch) {
    var target = _resolveInputTarget(sel);
    if (!target) return;
    var el = target.element;
    var cur = el.value || '';
    var state = _textSelection[target.key];
    var start = state ? Math.max(0, Math.min(cur.length, state.start)) : cur.length;
    var end = state ? Math.max(start, Math.min(cur.length, state.end)) : cur.length;
    var inserted = String(ch);
    var before = new InputEvent('beforeinput', {
      bubbles: true, cancelable: true, data: inserted, inputType: 'insertText', isComposing: false
    });
    if (el.dispatchEvent(before) === false) return;
    el.value = cur.slice(0, start) + inserted + cur.slice(end);
    var caret = start + inserted.length;
    _textSelection[target.key] = { start: caret, end: caret, direction: 'none' };
    var input = new InputEvent('input', {
      bubbles: true, cancelable: false, data: inserted, inputType: 'insertText', isComposing: false
    });
    el.dispatchEvent(input);
  };
  // Backspace：起点 no-op；否则 beforeinput(deleteContentBackward) → mutation →
  // input(deleteContentBackward, cancelable=false)。
  globalThis.__zw_text_delete = function(sel) {
    var target = _resolveInputTarget(sel);
    if (!target) return;
    var el = target.element;
    var cur = el.value || '';
    var state = _textSelection[target.key];
    var start = state ? Math.max(0, Math.min(cur.length, state.start)) : cur.length;
    var end = state ? Math.max(start, Math.min(cur.length, state.end)) : cur.length;
    if (cur.length === 0) return; // 空值 backspace 无变化，不派发（同 real browser）。
    if (start === end) {
      if (start === 0) return;
      start--;
      var last = cur.charCodeAt(start);
      if (last >= 0xDC00 && last <= 0xDFFF && start > 0) {
        var previous = cur.charCodeAt(start - 1);
        if (previous >= 0xD800 && previous <= 0xDBFF) start--;
      }
    }
    var before = new InputEvent('beforeinput', {
      bubbles: true, cancelable: true, data: null, inputType: 'deleteContentBackward', isComposing: false
    });
    if (el.dispatchEvent(before) === false) return;
    el.value = cur.slice(0, start) + cur.slice(end);
    _textSelection[target.key] = { start: start, end: start, direction: 'none' };
    var input = new InputEvent('input', {
      bubbles: true, cancelable: false, data: null, inputType: 'deleteContentBackward', isComposing: false
    });
    el.dispatchEvent(input);
  };

  // 宿主拆分默认动作 transaction 时，延迟 listener 排入的 microtask，直到 commit/rollback 完成。
  // https://html.spec.whatwg.org/multipage/webappapis.html#perform-a-microtask-checkpoint
  var _zwNativeQueueMicrotask = globalThis.queueMicrotask;
  var _zwNativePromiseThen = typeof Promise === 'function' ? Promise.prototype.then : null;
  var _zwHostMicrotasks = null;
  function _zwRunOrDeferPromiseReaction(callback, value) {
    if (!_zwHostMicrotasks) return callback(value);
    return new Promise(function(resolve, reject) {
      _zwHostMicrotasks.push(function() {
        try { resolve(callback(value)); } catch (error) { reject(error); }
      });
    });
  }
  globalThis.__zw_begin_host_action_transaction = function() {
    _zwHostMicrotasks = [];
  };
  globalThis.__zw_end_host_action_transaction = function() {
    var pending = _zwHostMicrotasks || [];
    _zwHostMicrotasks = null;
    for (var i = 0; i < pending.length; i++) {
      try { pending[i](); } catch (_) {}
    }
  };
  if (typeof _zwNativeQueueMicrotask === 'function') {
    globalThis.queueMicrotask = function(callback) {
      if (typeof callback !== 'function') throw new TypeError('queueMicrotask callback must be callable');
      if (_zwHostMicrotasks) _zwHostMicrotasks.push(callback);
      else _zwNativeQueueMicrotask(callback);
    };
  }
  if (_zwNativePromiseThen) {
    Promise.prototype.then = function(onFulfilled, onRejected) {
      var fulfilled = typeof onFulfilled === 'function'
        ? function(value) { return _zwRunOrDeferPromiseReaction(onFulfilled, value); }
        : onFulfilled;
      var rejected = typeof onRejected === 'function'
        ? function(reason) { return _zwRunOrDeferPromiseReaction(onRejected, reason); }
        : onRejected;
      return _zwNativePromiseThen.call(this, fulfilled, rejected);
    };
  }
  // 解析 selector → canonical stable selector（`__zw_query_match`，与 querySelector 同 identity）+
  // 真实 tag（`__zw_get_tag`，非 `_tagFromSel` 启发式）判 INPUT/TEXTAREA → 返元素 proxy（否则 null）。
  // __zw_text_input / __zw_text_delete 共用。
  function _resolveInputTarget(sel) {
    var resolved = typeof __zw_query_match === 'function' ? __zw_query_match(sel) : sel;
    if (!resolved) return null;
    var tag = (typeof __zw_get_tag === 'function' ? __zw_get_tag(resolved) : '').toUpperCase();
    if (tag !== 'INPUT' && tag !== 'TEXTAREA') return null;
    return { element: _wrapSelector(resolved), key: _elKey(resolved, null) };
  }
  // P1a form input：导航（URL 变化）时清 value 缓存——防跨页同选择器 stale value。
  globalThis.__zw_mark_user_edited = function(sel) { _userEdited[_elKey(String(sel), null)] = true; };
  globalThis.__zw_clear_user_edited = function(el) {
    for (var key in _proxyCache) {
      if (_proxyCache[key] === el) { delete _userEdited[key]; return; }
    }
  };
  globalThis.__zw_reset_form_state = function() { _inputValues = {}; _inputDefault = {}; _inputDefaultDirty = {}; _boolDefault = {}; _boolDefaultDirty = {}; _classCache = {}; _customValidity = {}; _userEdited = {}; _indeterminate = {}; _textSelection = {}; _outputDefault = {}; _outputValue = {}; _resourceStates = {}; _textareaDefault = {}; _shadowRoots = {}; _shadowHandles = {}; _shadowHandleMeta = {}; _handleChildren = {}; _expando = {}; _scrollOffsets = {}; _winScroll = { top: 0, left: 0 }; _elementAnimations = {}; _pointerCapture = {}; _zwTopLayer = {}; _popoverTargetEl = {}; _zwCanvasCtx = {}; _zwDialogModal = {}; };

  // 现代动态 reftest 常用模式：`requestAnimationFrame(() => requestAnimationFrame(() => { …setup…; takeScreenshot(); }))`
  // 把 DOM setup 延迟到「布局/绘制后」。harness 在脚本+load 派发后才截图，故 rAF
  // 同步立即执行回调即可让 setup mutation 被记录并应用到二次渲染（镜像 setTimeout 的 microtask 语义，
  // 但同步以保证回调在 sandbox 生命周期内必然执行）。
  globalThis.requestAnimationFrame = function(fn) {
    var id = _timerId++;
    if (globalThis.__ZW_RAF_FRAME_DRIVEN) {
      // 帧驱动（R2713a）：延后到 host render 后的 __zw_raf_tick 派发（spec rAF 语义）。
      if (typeof fn === 'function') _rafPending[id] = fn;
    } else if (typeof fn === 'function' && _rafBudget > 0) {
      // 同步 stub（reftest 兼容，默认路径）：预算内立即 fn(0)，让 double-rAF setup mutation
      // 进入最终 HTML 被 harness 单渲染捕获。
      _rafBudget--;
      try { fn(0); } catch (_e) {}
    }
    return id;
  };
  globalThis.cancelAnimationFrame = function(id) {
    if (globalThis.__ZW_RAF_FRAME_DRIVEN) delete _rafPending[id];
    // OFF 路径 no-op（旧行为）。
  };
  // host 在 render 后调用（renderer tick_observers；OFF 时早返零开销）。ts = DOMHighResTimeStamp（ms）。
  globalThis.__zw_raf_tick = function(ts) {
    if (!globalThis.__ZW_RAF_FRAME_DRIVEN) return;
    var cbs = _rafPending; _rafPending = {}; // 本帧快照、清空（rAF 内重注册入下一帧队列）
    for (var id in cbs) { try { cbs[id](ts); } catch (_e) {} }
  };
  globalThis.webkitRequestAnimationFrame = globalThis.requestAnimationFrame;
  globalThis.mozRequestAnimationFrame = globalThis.requestAnimationFrame;

  // `/common/reftest-wait.js` 提供的完成信号；harness 在 load 后统一截图，故 no-op。
  // 失败保守：返回 resolved Promise（部分测试链式调用 `.then(...)`）。
  globalThis.takeScreenshot = function(_cb) {
    if (typeof _cb === 'function') { try { _cb(); } catch (_e) {} }
    return Promise.resolve();
  };

  // `window.getComputedStyle(elt[, pseudo])`：动态 reftest 极常用作「强制 reflow」
  // 触发器——`getComputedStyle(el).getPropertyValue('grid-template-columns')` 结果
  // 丢弃，仅逼布局发生（css-grid/grid-with-content-dynamic-display-001 line 43 即此
  // 模式，紧接 line 47 的 `display:block` 视觉 mutation 才是测试目的）。
  // 本全局缺失 → 调用抛 ReferenceError **中断整个脚本**，使其后的 DOM mutation 全丢
  // `window.getComputedStyle(elt[, pseudo])`：返 CSSStyleDeclaration。高频作 visibility/hidden
  // 检查（`cs.display === 'none'`）与 reflow 触发器。经 host `__zw_get_computed_style(sel, prop)`
  // 返**计算值**（display/position/visibility/opacity 首批；UA 默认 builtin，`<style>` 级联）。
  // 属性访问（camelCase `.display`/`.backgroundColor`）与 `getPropertyValue(kebab)` 均经
  // `_camelToKebab` 归一后查询。host 未注册（polyfill/WebView）或未覆盖属性 → ''（fallback，
  // 不抛，同旧 stub 行为）；handle-only（无 sel）→ ''。
  globalThis.getComputedStyle = function(elt, _pseudo) {
    var sel = elt && elt.__zwSelector;
    var hasHost = sel && typeof __zw_get_computed_style === 'function';
    var query = function(prop) {
      if (!hasHost) return '';
      try { return __zw_get_computed_style(sel, prop); } catch (_e) { return ''; }
    };
    return new Proxy({}, {
      get: function(_t, prop) {
        var p = String(prop);
        if (p === 'getPropertyValue') {
          return function(name) { return query(_camelToKebab(String(name))); };
        }
        if (p === 'getPropertyPriority' || p === 'item') return function() { return ''; };
        if (p === 'length') return 0;
        if (p === 'parentRule') return null;
        if (p === 'cssText') return '';
        if (typeof prop !== 'string') return undefined; // Symbol 属性返 undefined
        return query(_camelToKebab(p));
      }
    });
  };

  function _emptyCollection() {
    return { length: 0, item: function() { return null; }, namedItem: function() { return null; } };
  }

  function _parseLocation(href) {
    var h = String(href == null ? '' : href);
    // 优先 new URL（R2778，spec-correct：percent-encoding / IDNA / 默认端口归一 / 端口解析），仅在
    // __zw_parse_url 已注册时；否则回退朴素 regex（reftest/裸 sandbox 无回调路径，零回归）。
    if (typeof URL === 'function' && typeof __zw_parse_url === 'function') {
      try {
        var u = new URL(h);
        return {
          href: u.href, protocol: u.protocol, host: u.host, hostname: u.hostname,
          pathname: u.pathname, search: u.search, hash: u.hash, origin: u.origin,
        };
      } catch (_) { /* 解析失败 → 回退 regex */ }
    }
    var m = h.match(/^([^:]+):\/\/([^\/]*)(\/[^?#]*)?(\?[^#]*)?(#.*)?$/);
    if (!m) {
      return { href: h || 'about:blank', protocol: '', host: '', hostname: '', pathname: '/', search: '', hash: '', origin: 'null' };
    }
    var host = m[2] || '';
    return {
      href: h,
      protocol: m[1] + ':',
      host: host,
      hostname: host.split(':')[0] || '',
      pathname: m[3] || '/',
      search: m[4] || '',
      hash: m[5] || '',
      origin: host ? m[1] + '://' + host : 'null',
    };
  }

  function _makeLocation() {
    function href() {
      var base = typeof __zw_get_page_url === 'function' ? __zw_get_page_url() : 'about:blank';
      // R3005：反映 history pushState/replaceState 设的当前 entry url（_resolveHistUrl 已解析为绝对，见 part02）。
      // _hist_current 在 part02 定义（同 IIFE 函数声明提升），getter 运行时（shim 全安装后）已就绪；typeof guard 防御。
      // 使 SPA router 的 location.pathname/href 反映路由变更（旧仅读 host 页面 URL，pushState 后 stale）。
      if (typeof _hist_current === 'function') {
        var hu = _hist_current().url;
        if (hu) return hu;
      }
      return base;
    }
    return {
      get href() { return _parseLocation(href()).href; },
      // R3008：location.href = v 经 _setLocationPart 整体替换 URL（navigation，_setLocationPart 在 part02 定义，
      // 同 IIFE 提升，setter 运行时就绪，typeof guard 防御）。
      set href(v) { if (typeof _setLocationPart === 'function') _setLocationPart('href', v); },
      get protocol() { return _parseLocation(href()).protocol; },
      get host() { return _parseLocation(href()).host; },
      get hostname() { return _parseLocation(href()).hostname; },
      get pathname() { return _parseLocation(href()).pathname; },
      set pathname(v) { if (typeof _setLocationPart === 'function') _setLocationPart('pathname', v); },
      get search() { return _parseLocation(href()).search; },
      set search(v) { if (typeof _setLocationPart === 'function') _setLocationPart('search', v); },
      get hash() { return _parseLocation(href()).hash; },
      // R3006：location.hash = v 更新 hash + history entry + 派发 hashchange（_setLocationHash 在 part02 定义，
      // 同 IIFE 提升，setter 运行时就绪）。SPA hash 路由核心。
      set hash(v) { if (typeof _setLocationHash === 'function') _setLocationHash(v); },
      get origin() { return _parseLocation(href()).origin; },
      // R3009：assign/replace 导航方法（_locationAssign/_locationReplace 在 part02 定义，同 IIFE 提升，运行时就绪，
      // typeof guard 防御）。assign(url) ≡ location.href = url（MDN）；replace(url) replace 当前 entry。
      assign: function (url) { if (typeof _locationAssign === 'function') _locationAssign(url); },
      replace: function (url) { if (typeof _locationReplace === 'function') _locationReplace(url); },
      // headless 无真文档重载——synthesized page 无原始 fetch 可重取。no-op（不抛，spec reload 返 void）。
      // host 真重载（重新 fetch + 解析 + 执行页面脚本）defer。
      reload: function () {},
      toString: function() { return _parseLocation(href()).href; }
    };
  }

  globalThis.location = _makeLocation();
  globalThis.self = globalThis;
  globalThis.top = globalThis;
  globalThis.parent = globalThis;
  // js-dom M4 R33：`Window.event`（HTML spec `current event`，legacy IE 全局）。Window 须 own `event`
  // 属性，初值 undefined（spec dispatch 前 window.event === undefined）；dispatch 期 = 正在派发的 event
  //（innermost，嵌套 dispatch 后恢复外层）；dispatch 后回 undefined。_dispatchWithBubble（part03）在派发
  // 前 save+set、finally restore。defineProperty writable:true 使 dispatch 期可写、enumerable:true 使
  // `assert_own_property(window,'event')` + for-in 可见（WPT event-global）。
  Object.defineProperty(globalThis, 'event', {
    value: undefined,
    writable: true,
    configurable: true,
    enumerable: true
  });

  globalThis.screen = {
    width: 1280,
    height: 800,
    availWidth: 1280,
    availHeight: 760,
    colorDepth: 24,
    pixelDepth: 24,
    left: 0,
    top: 0,
    orientation: { type: 'landscape-primary', angle: 0 }
  };
  globalThis.innerWidth = 1280;
  globalThis.innerHeight = 800;
  globalThis.outerWidth = 1280;
  globalThis.outerHeight = 800;
  globalThis.devicePixelRatio = 1;
  // R2987 window context / security 全局——库 feature-detect 后再使用 secure-only API（crypto.subtle /
  // SharedArrayBuffer / Service Worker）或错误上报。
  // `isSecureContext`（getter，随 location.protocol）：secure 除非 http:/ws:（about:blank/https/wss/file → secure）。
  // spec secure context 判定含 localhost / 非安全白名单，headless 取协议近似（http/ws 不安全，余皆安全）。
  Object.defineProperty(globalThis, 'isSecureContext', {
    configurable: true,
    get: function () {
      try {
        var p = globalThis.location && globalThis.location.protocol;
        return p !== 'http:' && p !== 'ws:';
      } catch (_e) { return true; }
    }
  });
  // `crossOriginIsolated`：需 COOP+COEP 响应头隔离。headless 无 → false（SharedArrayBuffer / 跨 origin
  // 资源不受隔离，feature-detect 库正确回落）。
  Object.defineProperty(globalThis, 'crossOriginIsolated', { configurable: true, value: false });
  // `reportError(reason)`：向 window 派发 ErrorEvent（error 上报库 / Promise catch 转错误事件 / 兜底未捕获错误
  // 报告高频）。经 globalThis.dispatchEvent（R2932）触 window 'error' listener + onerror IDL handler（R2932 注册）。
  // spec reportError 把 reason 转 ErrorEvent 派发到 window error handler；headless 复用 dispatchEvent 路径。
  globalThis.reportError = function (reason) {
    try {
      var msg = (reason && (reason.message || reason.name)) ? String(reason.message || reason.name) : String(reason);
      var ev = new ErrorEvent('error', {
        message: msg,
        filename: '',
        lineno: 0,
        colno: 0,
        error: (reason instanceof Error) ? reason : null
      });
      if (typeof globalThis.dispatchEvent === 'function') globalThis.dispatchEvent(ev);
    } catch (_e) {}
  };
  // scroll（R2817/R3047）——window 滚动方法/属性。headless 无真视口滚动 → R3047 改 JS-side 状态追踪：
  // scrollTo/scrollBy 更新 `_winScroll`，scrollX/scrollY/pageXOffset/pageYOffset 经 defineProperty getter 读回
  //（程序化滚动 round-trip 自洽；无真视口滚动，仅 JS-observable 状态）。`scrollIntoView` 为 Element 方法（非 window），
  // 此处 window 级 stub 保兼容（feature-detect 不抛）。参数支持 `(x,y)` 与 `{left,top,behavior}` 两种 spec 形式。
  function _zwApplyScroll(store, arg1, arg2, isBy) {
    var nx, ny;
    if (arg1 && typeof arg1 === 'object') { // scrollTo({left, top, behavior})
      nx = Number(arg1.left) || 0; ny = Number(arg1.top) || 0;
    } else { // scrollTo(x, y)
      nx = Number(arg1) || 0; ny = Number(arg2) || 0;
    }
    if (isBy) { store.left += nx; store.top += ny; }
    else { store.left = nx; store.top = ny; }
    if (store.left < 0) store.left = 0; // spec scroll 不可负
    if (store.top < 0) store.top = 0;
  }
  // R3051：scroll 事件派发（R3047 follow-up）。scrollTo/scrollBy/scrollTop= 后派发 'scroll' 事件，使
  // scroll-listener（infinite scroll / lazy load / sticky nav / parallax）在程序化滚动后触发。real browser 异步
  // 派发 + 同帧 coalesce；headless 同步派发（每滚动操作一事件，documented 近似）。element 经 _dispatchWithBubble，
  // window（sel/handle 均空）经 globalThis.dispatchEvent。'_makeEvent('scroll')' 默认 bubbles=false/cancelable=false（spec）。
  function _zwFireScroll(key, sel, handle) {
    try {
      if (sel || handle) _dispatchWithBubble(key, sel, handle, _makeEvent('scroll'));
      else if (typeof globalThis.dispatchEvent === 'function') globalThis.dispatchEvent(_makeEvent('scroll'));
    } catch (_e) {}
  }
  Object.defineProperty(globalThis, 'scrollX', { configurable: true, get: function () { return _winScroll.left; } });
  Object.defineProperty(globalThis, 'pageXOffset', { configurable: true, get: function () { return _winScroll.left; } });
  Object.defineProperty(globalThis, 'scrollY', { configurable: true, get: function () { return _winScroll.top; } });
  Object.defineProperty(globalThis, 'pageYOffset', { configurable: true, get: function () { return _winScroll.top; } });
  globalThis.scrollTo = function (a, b) { _zwApplyScroll(_winScroll, a, b, false); _zwFireScroll(null, null, null); };
  globalThis.scroll = globalThis.scrollTo;
  globalThis.scrollBy = function (a, b) { _zwApplyScroll(_winScroll, a, b, true); _zwFireScroll(null, null, null); };
  globalThis.scrollIntoView = function () {};
  // R3253：宿主「用户滚动」（renderer 收到 browser IPC ScrollEventParams）注入钩子——更新 `_winScroll`
  //（使 window.scrollY/scrollX 跟踪用户滚动）+ 派 'scroll' 事件（_zwFireScroll）。区别于 `scrollBy`：
  // ① 走内部 `_zwApplyScroll`/`_zwFireScroll`，**绕过页面可能覆写的 `globalThis.scrollBy`**（real browser 的
