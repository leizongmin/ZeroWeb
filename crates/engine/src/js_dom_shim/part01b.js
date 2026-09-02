  // FR-009：媒体资源状态接口常量。元素 proxy 的实例读由 get trap 提供；构造器仅暴露
  // Web IDL 静态常量，MediaError 实例用于失败状态。
  function HTMLMediaElement() { throw new TypeError('Illegal constructor'); }
  // HTMLAudioElement：接口构造器（spec：无 new 调用抛 TypeError——WPT audio_constructor
  // 断言面；实例经 `new Audio(src)` 工厂产出）。globalThis 守卫幂等，位于 part03 的
  // _zwHtmlElementIfaces 循环之前（`if (!globalThis[...])` 跳过既有定义）。
  // https://html.spec.whatwg.org/multipage/media.html#dom-audio
  function HTMLAudioElement() { throw new TypeError('Illegal constructor'); }
  globalThis.HTMLAudioElement = globalThis.HTMLAudioElement || HTMLAudioElement;
  HTMLMediaElement.NETWORK_EMPTY = 0; HTMLMediaElement.NETWORK_IDLE = 1;
  HTMLMediaElement.NETWORK_LOADING = 2; HTMLMediaElement.NETWORK_NO_SOURCE = 3;
  HTMLMediaElement.HAVE_NOTHING = 0; HTMLMediaElement.HAVE_METADATA = 1;
  HTMLMediaElement.HAVE_CURRENT_DATA = 2; HTMLMediaElement.HAVE_FUTURE_DATA = 3;
  HTMLMediaElement.HAVE_ENOUGH_DATA = 4;
  globalThis.HTMLMediaElement = globalThis.HTMLMediaElement || HTMLMediaElement;
  function HTMLTrackElement() { throw new TypeError('Illegal constructor'); }
  HTMLTrackElement.NONE = 0; HTMLTrackElement.LOADING = 1;
  HTMLTrackElement.LOADED = 2; HTMLTrackElement.ERROR = 3;
  globalThis.HTMLTrackElement = globalThis.HTMLTrackElement || HTMLTrackElement;
  function MediaError() { throw new TypeError('Illegal constructor'); }
  MediaError.MEDIA_ERR_ABORTED = 1; MediaError.MEDIA_ERR_NETWORK = 2;
  MediaError.MEDIA_ERR_DECODE = 3; MediaError.MEDIA_ERR_SRC_NOT_SUPPORTED = 4;
  MediaError.prototype.MEDIA_ERR_ABORTED = 1; MediaError.prototype.MEDIA_ERR_NETWORK = 2;
  MediaError.prototype.MEDIA_ERR_DECODE = 3; MediaError.prototype.MEDIA_ERR_SRC_NOT_SUPPORTED = 4;
  globalThis.MediaError = globalThis.MediaError || MediaError;
  // media-elements M3：TextTrack 家族最小接口面——构造器均 Illegal constructor（spec：
  // TextTrack/TextTrackCueList/TextTrackList 由元素 API 产出，脚本不可直接 new；
  // TextTrackCue 历史构造器 r7742 移除——new 亦 TypeError，historical 用例断言面）。
  // 实例经 _zwMakeTextTrack / _zwMakeTextTrackCueList / _zwMakeTextTrackList 工厂
  // （Object.create(prototype) + own props）产出，instanceof 走原型链。
  function TextTrack() { throw new TypeError('Illegal constructor'); }
  globalThis.TextTrack = globalThis.TextTrack || TextTrack;
  function TextTrackCueList() { throw new TypeError('Illegal constructor'); }
  globalThis.TextTrackCueList = globalThis.TextTrackCueList || TextTrackCueList;
  function TextTrackList() { throw new TypeError('Illegal constructor'); }
  globalThis.TextTrackList = globalThis.TextTrackList || TextTrackList;
  function TextTrackCue() { throw new TypeError('Illegal constructor'); }
  globalThis.TextTrackCue = globalThis.TextTrackCue || TextTrackCue;
  // media-elements M3 扩批 XII：VTTCue 构造器（spec vttcue——脚本创建 cue 的唯一入口；
  // 与 TextTrackCue 是分离接口——constructor 断言面）。startTime/endTime setter：非有限
  // → TypeError（startTime NaN/+Inf/-Inf 全抛——dom-vttcue-starttime；endTime NaN/-Inf
  // 抛、**+Inf 合法**（无末尾 cue）——dom-vttcue-endtime）；id DOMString（null→'null'）；
  // pauseOnExit boolean；track readonly（addCue/removeCue 维护）；cue 是 EventTarget
  //（onenter/onexit + dispatchEvent——TextTrackCue-onenter/onexit 断言面）。
  // https://w3c.github.io/webvtt/#vttcue-interface
  function VTTCue(startTime, endTime, text) {
    var cue = Object.create(VTTCue.prototype);
    var _st = Number(startTime); if (isNaN(_st)) _st = 0;
    var _et = Number(endTime); if (isNaN(_et)) _et = 0;
    cue._zwStartTime = _st;
    cue._zwEndTime = _et;
    cue._zwText = String(text == null ? '' : text);
    cue._zwId = '';
    cue._zwPauseOnExit = false;
    cue._zwTrack = null;
    // WebVTT region/line 定位选项缺省面（vttcue-interface；headless 不做视觉布局，
    // 仅 IDL 存储——track-add-remove-cue / vtt-cue-float-precision 断言面）。
    cue._zwVertical = '';
    cue._zwSnapToLines = true;
    cue._zwLine = 'auto';
    cue._zwPosition = 'auto';
    cue._zwSize = 100;
    cue._zwAlign = 'center';
    globalThis._zwCueEnsureEventTarget(cue);
    return cue;
  }
  globalThis.VTTCue = globalThis.VTTCue || VTTCue;
  // M3 扩批 XII：TrackEvent 构造器（spec event-definitions——textTracks addtrack/
  // removetrack 的事件类型）。type + init dict {track}；track readonly accessor（赋值
  // 被吞——TrackEvent constructor「ev.track after assignment」断言面）；prototype 链接
  // Event（instanceof Event 断言面）。createEvent('TrackEvent') 为 non-createable
  //（NOT_SUPPORTED_ERR——part06 createEvent map 不入此 type）。
  // https://html.spec.whatwg.org/multipage/media.html#the-trackevent-interface
  function TrackEvent(type, options) {
    // prototype 惰性链接：本 part 装载早于 part05 的 globalThis.Event 定义（shim 拼接序），
    // 装载期链接会落 Object.prototype（instanceof Event 断言失败）——首次构造时补链一次。
    var _teProto = TrackEvent.prototype;
    if (!(globalThis.Event && globalThis.Event.prototype && _teProto instanceof globalThis.Event)) {
      _teProto = Object.create((globalThis.Event && globalThis.Event.prototype) || Object.prototype);
      _teProto.constructor = TrackEvent;
      // track readonly accessor（赋值被吞——TrackEvent constructor「ev.track after
      // assignment」断言面）。
      Object.defineProperty(_teProto, 'track', {
        get: function () { return this._zwTrackValue == null ? null : this._zwTrackValue; },
        set: function () {},
        configurable: true,
      });
      TrackEvent.prototype = _teProto;
    }
    // 构造基座：globalThis.Event（运行时已就绪——part05 定义）。**不可用 _makeEvent**——
    // 该 helper 是 part03 IIFE 私有，本 part 作用域不可见（静默回落 {} 丢 type 面）。
    var ev = (typeof globalThis.Event === 'function')
      ? new globalThis.Event(type, options)
      : { type: type };
    Object.setPrototypeOf(ev, _teProto);
    var o = (options == null || typeof options !== 'object') ? {} : options;
    // spec：init dict track member（nullable TextTrack；缺省/非对象 → null——
    // 'ev.track' 单参断言面；{track: testTrack} 双参断言面）。
    ev._zwTrackValue = (o.track == null) ? null : o.track;
    return ev;
  }
  globalThis.TrackEvent = globalThis.TrackEvent || TrackEvent;
  // cue 的 EventTarget 面：on* handler + add/remove/dispatchEvent（最小派发——无冒泡）。
  // 复用 shim 通用 listener 机制不可行（cue 非 proxy 元素）——per-cue 内表。
  // M3 扩批 XII：泛化为 _zwEnsureEventTarget——TextTrack（oncuechange）/TextTrackList
  //（onaddtrack/onremovetrack）同为 EventTarget（TextTrack-oncuechange /
  // TextTrackList-onaddtrack/onremovetrack 断言面）。
  var _cueListeners = []; // [{ target, type, fn }]
  globalThis._zwEnsureEventTarget = function (target) {
    if (target.addEventListener) return;
    target.addEventListener = function (type, fn) {
      _cueListeners.push({ target: target, type: String(type), fn: fn });
    };
    target.removeEventListener = function (type, fn) {
      for (var i = _cueListeners.length - 1; i >= 0; i--) {
        var l = _cueListeners[i];
        if (l.target === target && l.type === String(type) && l.fn === fn) _cueListeners.splice(i, 1);
      }
    };
    target.dispatchEvent = function (ev) {
      var type = ev && ev.type ? String(ev.type) : '';
      var snapshot = [];
      for (var i = 0; i < _cueListeners.length; i++) {
        var l = _cueListeners[i];
        if (l.target === target && l.type === type) snapshot.push(l.fn);
      }
      // M3 扩批 XIII：spec concept-event-dispatch——dispatch 期间 target/currentTarget
      // 指向**exposed 视图**（TextTrackList 为索引只读 Proxy——holder.self；track-add-track
      // 断言 event.target === video.textTracks 面），派发后复原（NONE 态）。
      var _exposed = (target._zwHolder && target._zwHolder.self) || target;
      var _oldTarget = ev.target, _oldCur = ev.currentTarget, _oldPhase = ev.eventPhase;
      try {
        if (ev) { ev.target = _exposed; ev.currentTarget = _exposed; ev.eventPhase = 2; }
        for (var j = 0; j < snapshot.length; j++) {
          if (typeof snapshot[j] === 'function') { try { snapshot[j].call(target, ev); } catch (_eCe) {} }
        }
        var h = target['on' + type];
        if (typeof h === 'function') { try { h.call(target, ev); } catch (_eCh) {} }
      } finally {
        if (ev) { ev.target = _oldTarget; ev.currentTarget = _oldCur; ev.eventPhase = _oldPhase; }
      }
      return true;
    };
  };
  // 兼容名（内部调用点）。
  globalThis._zwCueEnsureEventTarget = globalThis._zwEnsureEventTarget;
  // on* 事件处理器 accessor 工厂（初值 null；赋 undefined → null——TextTrackCue-onenter
  // 「assigning undefined」断言面）。挂实例（per-target 表避免污染 prototype）。
  var _onHandlerProps = {}; // target -> { type: fn|null }
  globalThis._zwDefineTargetOnHandler = function (target, type) {
    _onHandlerProps[target] = _onHandlerProps[target] || {};
    Object.defineProperty(target, 'on' + type, {
      get: function () {
        var v = _onHandlerProps[target] ? _onHandlerProps[target][type] : null;
        return (v == null) ? null : v;
      },
      set: function (v) {
        _onHandlerProps[target][type] = (v == null || v === undefined) ? null : v;
      },
      configurable: true,
    });
  };
  // VTTCue.prototype IDL accessor（startTime/endTime/text/id/pauseOnExit/track + onenter/onexit）。
  Object.defineProperty(VTTCue.prototype, 'startTime', {
    get: function () { return this._zwStartTime; },
    set: function (v) {
      var n = Number(v);
      // https://w3c.github.io/webvtt/#dom-vttcue-starttime——NaN/±Infinity → TypeError
      if (isNaN(n) || n === Infinity || n === -Infinity) {
        throw new TypeError("Failed to set the 'startTime' property on 'TextTrackCue': The provided value is non-finite.");
      }
      this._zwStartTime = n;
      // 排序失效——所属 track 的 cues 列表即时重排（「changing order」断言面）。
      try { if (this._zwTrack && typeof this._zwTrack._zwInvalidateCues === 'function') this._zwTrack._zwInvalidateCues(); } catch (_eSi) {}
    },
    configurable: true,
  });
  Object.defineProperty(VTTCue.prototype, 'endTime', {
    get: function () { return this._zwEndTime; },
    set: function (v) {
      var n = Number(v);
      // https://w3c.github.io/webvtt/#dom-vttcue-endtime——NaN/-Infinity → TypeError；
      // **+Infinity 合法**（endless cue）。
      if (isNaN(n) || n === -Infinity) {
        throw new TypeError("Failed to set the 'endTime' property on 'TextTrackCue': The provided value is non-finite.");
      }
      this._zwEndTime = n;
    },
    configurable: true,
  });
  Object.defineProperty(VTTCue.prototype, 'text', {
    get: function () { return this._zwText; },
    set: function (v) { this._zwText = String(v == null ? '' : v); },
    configurable: true,
  });
  Object.defineProperty(VTTCue.prototype, 'id', {
    get: function () { return this._zwId; },
    set: function (v) { this._zwId = String(v == null ? 'null' : v); },
    configurable: true,
  });
  Object.defineProperty(VTTCue.prototype, 'pauseOnExit', {
    get: function () { return !!this._zwPauseOnExit; },
    set: function (v) { this._zwPauseOnExit = !!v; },
    configurable: true,
  });
  Object.defineProperty(VTTCue.prototype, 'track', {
    get: function () { return this._zwTrack; },
    set: function () {},
    configurable: true,
  });
  // onenter/onexit：初值 null、赋 undefined → null（TextTrackCue-onenter/onexit 断言面）。
  // prototype 级 accessor + per-instance 私有槽（_zwOnEnter/_zwOnExit）。
  ['enter', 'exit'].forEach(function (_evt) {
    var _slot = '_zwOn' + _evt.charAt(0).toUpperCase() + _evt.slice(1);
    Object.defineProperty(VTTCue.prototype, 'on' + _evt, {
      get: function () {
        var v = this[_slot];
        return (v == null) ? null : v;
      },
      set: function (v) { this[_slot] = (v == null || v === undefined) ? null : v; },
      configurable: true,
    });
  });
  // WebVTT cue 定位选项 IDL 面（vttcue-interface——headless 仅存储不做视觉布局）：
  // vertical 枚举（''/rl/lr，invalid 保留旧值——枚举 reflected setter 语义）；
  // snapToLines boolean；line double|'auto'（NaN 非法回落——spec WebIDL (double or
  // AutoKeyword)，double NaN → TypeError 面）；position long|'auto'；size double
  //（clamp [0,100]）；align 枚举（start/center/end/left/right，invalid 保留）。
  // https://w3c.github.io/webvtt/#dom-vttcue-vertical
  (function () {
    Object.defineProperty(VTTCue.prototype, 'vertical', {
      get: function () { return this._zwVertical; },
      set: function (v) {
        var s = String(v == null ? '' : v);
        if (s === '' || s === 'rl' || s === 'lr') this._zwVertical = s;
      },
      configurable: true,
    });
    Object.defineProperty(VTTCue.prototype, 'snapToLines', {
      get: function () { return !!this._zwSnapToLines; },
      set: function (v) { this._zwSnapToLines = !!v; },
      configurable: true,
    });
    var _isAuto = function (v) { return v === 'auto'; };
    Object.defineProperty(VTTCue.prototype, 'line', {
      get: function () { return this._zwLine; },
      set: function (v) {
        if (typeof v === 'number' && isNaN(v)) return; // NaN → 保留（invalid double）
        this._zwLine = (v == null || _isAuto(v)) ? 'auto'
          : (typeof v === 'number' ? v : (parseFloat(v) == null || isNaN(parseFloat(v)) ? 'auto' : parseFloat(v)));
      },
      configurable: true,
    });
    Object.defineProperty(VTTCue.prototype, 'position', {
      get: function () { return this._zwPosition; },
      set: function (v) {
        if (typeof v === 'number' && isNaN(v)) return;
        this._zwPosition = (v == null || _isAuto(v)) ? 'auto' : v;
      },
      configurable: true,
    });
    Object.defineProperty(VTTCue.prototype, 'size', {
      get: function () { return this._zwSize; },
      set: function (v) {
        var n = Number(v);
        if (isNaN(n)) return; // invalid → 保留旧值
        this._zwSize = Math.min(100, Math.max(0, n)); // clamp [0,100]
      },
      configurable: true,
    });
    Object.defineProperty(VTTCue.prototype, 'align', {
      get: function () { return this._zwAlign; },
      set: function (v) {
        var s = String(v == null ? '' : v);
        if (s === 'start' || s === 'center' || s === 'end' || s === 'left' || s === 'right') this._zwAlign = s;
      },
      configurable: true,
    });
  })();
  // M3 扩批 XV：getCueAsHTML 最小面（webvtt cue DOM API——entities 断言族以
  // `cue.getCueAsHTML().textContent` 消费）。headless 无 cue 文本标记解析（WebVTT
  // cue text 的 <Tag> 结构归渲染域远期）——DocumentFragment + 单 text node。实体解码
  // 在本面发生（spec：DOM 面按 character references 产出——cue.text 保持 parser 原文，
  // track-element-src-change 断言 '&amp;' 字面）。fragment 经 document 工厂产出。
  VTTCue.prototype.getCueAsHTML = function () {
    var frag = globalThis.document.createDocumentFragment();
    var raw = String(this.text == null ? '' : this.text)
      .replace(/&amp;/g, '&').replace(/&lt;/g, '<').replace(/&gt;/g, '>')
      .replace(/&lrm;/g, '\u200e').replace(/&rlm;/g, '\u200f').replace(/&nbsp;/g, '\u00a0');
    frag.appendChild(globalThis.document.createTextNode(raw));
    return frag;
  };
  // 工厂：TextTrack（kind/label/language/mode/cues/activeCues + addCue/removeCue——M3 扩批
  // XII）。cues/activeCues：mode==='disabled' → null，否则 same-object TextTrackCueList
  //（cues 按 startTime 升序动态排序——「changing order」断言；activeCues headless 近似 =
  // currentTime ∈ [start, end) 的 cue，非播放中 length 0）。label/language 若关联 track
  // 元素则实时反射 attr（track.label='baz' → t2.label 同步——kind/label/language 断言面）。
  globalThis._zwMakeTextTrackCueList = function () {
    var list = Object.create(globalThis.TextTrackCueList.prototype);
    list.length = 0;
    list.item = function (i) { i = Number(i) | 0; return (i >= 0 && i < list.length) ? list[i] : null; };
    return list;
  };
  // M3 扩批 XII：cue 列表重建（按 startTime 升序；tie 时**后加者在前**——Chromium/WPT
  // 实证：同 start 双 cue 改后到者 start 后 cues[0] 为后到者——cues「changing order」
  // 断言面）。
  // https://w3c.github.io/webvtt/#text-track-cue-list —「cue list 按 start time 排序」。
  // 索引经 **Proxy**（get trap 读内部数组；set trap 对整数索引返回 false——strict 赋值
  // TypeError / sloppy 静默忽略，「no indexed set/create (strict)」断言面）。
  globalThis._zwTextTrackRebuildCueList = function (holder, cueArr) {
    var sorted = cueArr.slice().sort(function (a, b) {
      var d = a._zwStartTime - b._zwStartTime;
      if (d !== 0) return d;
      // 同 start → endTime 大者在前（「changing order」断言：(0,1) vs 改后 (0,2) →
      // (0,2) 在前）；同 start 同 end → 添加序（「id parsed cue」断言：首 cue 在前）。
      // **经验序**——WPT 断言面反推（Chromium 对齐），与 WebVTT spec 文字有出入，
      // 以断言面为事实源（headless 近似记录）。
      var de = b._zwEndTime - a._zwEndTime;
      if (de !== 0) return de;
      return (a._zwAddOrder || 0) - (b._zwAddOrder || 0);
    });
    holder.arr = sorted;
    globalThis._zwSyncListHolder(holder);
    return sorted;
  };
  // 共享 Proxy 包装：整数索引 get/set + 透传其余（item/getCueById/length/on* 与 identity）。
  // M3 批 XII：索引 own-property 镜像——assert_array_equals 经 hasOwnProperty（走 Proxy
  // getOwnPropertyDescriptor 默认到 target）断言索引 own 可见；写镜像须对 **target**（Proxy
  // set trap 对整数索引恒 false，「no indexed set/create (strict)」断言面不受影响——页面
  // 经 proxy 写仍被拦）。
  globalThis._zwSyncListHolder = function (holder) {
    var arr = holder.arr, target = holder.target;
    for (var i = 0; i < arr.length; i++) target[i] = arr[i];
    for (var j = arr.length; j < target.length; j++) delete target[j];
    target.length = arr.length;
  };
  globalThis._zwMakeIndexedListProxy = function (holder, Ctor) {
    return new Proxy(holder.target, {
      get: function (t, prop) {
        if (typeof prop === 'string' && /^\d+$/.test(prop)) {
          var i = Number(prop);
          return (i >= 0 && i < holder.arr.length) ? holder.arr[i] : undefined;
        }
        // 方法/值原样透传（**不 bind**——bind 副本破坏身份断言 `tracks.onaddtrack === cb`；
        // 各方法已闭包引用 holder/target，this 无关紧要）。
        return t[prop];
      },
      set: function (t, prop, value) {
        if (typeof prop === 'string' && /^\d+$/.test(prop)) return false; // 只读索引
        t[prop] = value;
        return true;
      },
      deleteProperty: function (t, prop) {
        if (typeof prop === 'string' && /^\d+$/.test(prop)) return false;
        delete t[prop];
        return true;
      },
    });
  };
  globalThis._zwMakeTextTrackCueListWithGetById = function () {
    return globalThis._zwWrapCueListHolder({ arr: [], target: globalThis._zwMakeTextTrackCueList() });
  };
  // 工厂内部用：对已有 holder（无 getCueById）补挂后包 Proxy。
  globalThis._zwWrapCueListHolder = function (holder) {
    // M3 扩批 XVI：for...of 迭代面（TextTrackCueList 是 iterable——track-cues-enter-
    // seeking 的 `for (let cue of testTrack.track.cues)` 断言形态）。iterator 挂 target
    //（proxy get 透传）；迭代时经 holder.arr 实时取（快照序——同 item()）。
    if (typeof Symbol !== 'undefined' && Symbol.iterator && !holder.target[Symbol.iterator]) {
      holder.target[Symbol.iterator] = function () {
        var i = 0;
        return {
          next: function () {
            return (i < holder.arr.length)
              ? { done: false, value: holder.arr[i++] }
              : { done: true };
          }
        };
      };
    }
    holder.target.getCueById = function (id) {
      var s = String(id);
      // spec getCueById：id 空串恒 null（'If id is the empty string, return null'）。
      if (s === '') return null;
      for (var j = 0; j < holder.arr.length; j++) {
        if (holder.arr[j] && holder.arr[j].id === s) return holder.arr[j];
      }
      return null;
    };
    return globalThis._zwMakeIndexedListProxy(holder, globalThis.TextTrackCueList);
  };
  // M3 扩批 X：第 5/6 参——id 反射关联 track 元素的 id 内容属性（readonly）；
  // M3 扩批 XII：第 6 参 ownerEl——**track 元素** proxy（label/language/kind 实时
  // 反射 attr；cues 可用性 gate——track 资源未加载 cues=null）；第 7 参 mediaEl——
  // 关联 media 元素 proxy（addTextTrack 产物亦传，activeCues 播放态查询用）。
  globalThis._zwMakeTextTrack = function (kind, label, language, mode, id, ownerEl, mediaEl) {
    var track = Object.create(globalThis.TextTrack.prototype);
    var _cueArr = [];
    var _addOrderSeq = 0; // per-track cue 添加序（tie 排序用——后加者在前）
    var _cuesHolder = { arr: [], target: globalThis._zwMakeTextTrackCueList() };
    var _activeHolder = { arr: [], target: globalThis._zwMakeTextTrackCueList() };
    var _cuesList = globalThis._zwWrapCueListHolder(_cuesHolder);
    var _activeList = globalThis._zwWrapCueListHolder(_activeHolder);
    var _ttMode = String(mode || 'disabled');
    var _ttId = String(id == null ? '' : id);
    // M3 扩批 XII：cues 可用性 gate（track 元素产物）——track 资源未 settle 时
    // cues 恒 null（spec：cue list 由 parent media element 加载循环启动，启动前不可用；
    // cues「default attribute」断言面）。settle 由 _zwTrackScheduleLoad（data:/普通 URL
    // 均派）提交。M3 扩批 XIII 收窄：**仅 track 子仍挂 media 父下**时 gate 生效——
    // detached track 元素（createElement 产物、无 media 父）cue list 即可用（spec 轨道
    // 无 media 父时不参与 media 加载循环、track URL 缺失即「启动完成」——src-clear-cues
    // 断言面：detached track track.track.cues 非 null）。
    var _cuesGate = function () {
      if (!ownerEl) return true; // addTextTrack 产物——cue list 即可用
      try {
        var _ogKey = (typeof _elKey === 'function') ? _elKey(ownerEl.__zwSelector || null, ownerEl.__zwHandle || null) : '';
        if (_ogKey && typeof _resourceStates !== 'undefined' && _resourceStates[_ogKey]) return true;
        if (typeof globalThis._zwParentMediaProxy === 'function') {
          return !globalThis._zwParentMediaProxy(ownerEl.__zwSelector || null, ownerEl.__zwHandle || null);
        }
      } catch (_eCg) {}
      return true;
    };
    function _readAttr(name) {
      try {
        if (ownerEl) {
          var v = ownerEl.getAttribute ? ownerEl.getAttribute(name) : null;
          if (v == null) return ''; // attr 缺省 → ''（反射语义——removeAttribute 同步面）
          return String(v);
        }
      } catch (_eRa) {}
      return '';
    }
    track.kind = String(kind);
    // label/language：关联元素时实时反射（getter 读 attr——track.label='baz' /
    // removeAttribute('label') 同步面）；addTextTrack 产物固定初值。
    Object.defineProperty(track, 'label', {
      get: function () { return ownerEl ? _readAttr('label') : String(label == null ? '' : label); },
      set: function (v) { label = String(v == null ? '' : v); },
      configurable: true,
    });
    Object.defineProperty(track, 'language', {
      get: function () { return ownerEl ? _readAttr('srclang') : String(language == null ? '' : language); },
      set: function (v) { language = String(v == null ? '' : v); },
      configurable: true,
    });
    // mode：枚举归一 setter（invalid/undefined 原样转 String 不命中三值 → 保留旧值；
    // {toString:...} 经 String() 转换命中）+ same-object cues/activeCues 失效面。
    Object.defineProperty(track, 'mode', {
      get: function () { return _ttMode; },
      set: function (v) {
        var s = String(v == null ? 'null' : v);
        if (s !== 'disabled' && s !== 'hidden' && s !== 'showing') return; // invalid → 保留
        // M3 扩批 XV：mode 从 disabled 变更为 hidden/showing → 触发 track URL 处理
        //（spec text track mode setter——「若 track 关联 track 元素且未加载，则启动
        // track URL 处理」；timings-hour/magic-header/header-checks 经
        // enableAllTextTracks 把非 default track 转 hidden 后 onload 断言面）。
        var _wasDisabled = (_ttMode === 'disabled');
        _ttMode = s;
        if (_wasDisabled && s !== 'disabled' && ownerEl
            && typeof globalThis._zwTrackScheduleLoad === 'function') {
          try {
            // mode gate bypass——本路径就是 spec 的「mode 变更启动加载」入口（绕过
            // _zwScheduleChildTrackLoads 的 default 属性过滤）。**handle 恒传 null**：
            // 静态 HTML wrapper 的 handle 不在 mutations attr registry（__zw_get_attr_handle
            // 只读同步脚本写入），传 handle 会使 src 读空 → 误 error settle；sel 路径
            // 走宿主 HTML 快照（静态/动态 attr 均真值）。
            globalThis._zwTrackScheduleLoad(ownerEl.__zwSelector || null, null);
          } catch (_eModeT) {}
        }
      },
      configurable: true,
    });
    Object.defineProperty(track, 'id', {
      get: function () { return _ttId; },
      set: function () {},
      configurable: true,
    });
    // cues：disabled → null（spec）；否则重建排序视图（same list 对象）。
    Object.defineProperty(track, 'cues', {
      get: function () {
        if (_ttMode === 'disabled' || !_cuesGate()) return null;
        globalThis._zwTextTrackRebuildCueList(_cuesHolder, _cueArr);
        return _cuesList;
      },
      set: function () {},
      configurable: true,
    });
    // activeCues：**仅 mode gate**（disabled → null）——与 cues 的 readiness gate 非对称
    //（spec dom-texttrack-activecues「return null if the text track's mode is the text
    // track disabled mode」；activeCues 断言面：track 资源未 settle 时亦非 null——
    // detached video 上 t2.mode='showing' 后 activeCues 即列表）。headless 近似——
    // **仅播放中**（playing）取 currentTime ∈ [start, end) 的 cue，非播放中恒空列表
    //（activeCues 断言面：未播放 addCue 后 length 0；playing 后 length 1）。
    Object.defineProperty(track, 'activeCues', {
      get: function () {
        if (_ttMode === 'disabled') return null;
        var _now = null;
        try {
          var _me = mediaEl || ownerEl;
          var _ms = (typeof _mediaState !== 'undefined') && _me ? _mediaState[typeof _elKey === 'function' ? _elKey(_me.__zwSelector, _me.__zwHandle) : ''] : null;
          // M3 扩批（fixture-mounted 切片 2）：时刻有效性 gate——playing（播放推进中）
          // 或**已建立媒体时刻**（seek 落点/播放钟推进过——march hook / seek sync 置
          // _zwMediaTimeKnown）时按当前时刻取；否则空列表（video loading 断言面——
          // 从未推进的元素无 active 时刻概念）。
          if (_ms && (_ms.playing || _ms._zwMediaTimeKnown)) _now = Number(_ms.currentTime) || 0;
        } catch (_eAc) {}
        var _act = [];
        if (_now != null) {
          for (var i = 0; i < _cueArr.length; i++) {
            var c = _cueArr[i];
            if (c._zwStartTime <= _now && _now < c._zwEndTime) _act.push(c);
          }
        }
        globalThis._zwTextTrackRebuildCueList(_activeHolder, _act);
        return _activeList;
      },
      set: function () {},
      configurable: true,
    });
    // addCue：cue 已关联其它 track → 先从旧 track 移除（'adding a cue to two different
    // tracks' 断言 c1.track === t2）；已在本 track → no-op（'a track twice'）。
    // https://w3c.github.io/webvtt/#dom-texttrack-addcue
    track.addCue = function (cue) {
      if (!cue || typeof cue !== 'object') return;
      if (cue._zwTrack === track) return;
      if (cue._zwTrack && typeof cue._zwTrack.removeCue === 'function') {
        try { cue._zwTrack.removeCue(cue); } catch (_eAo) {}
      }
      cue._zwTrack = track;
      cue._zwAddOrder = ++_addOrderSeq;
      _cueArr.push(cue);
      globalThis._zwCueEnsureEventTarget(cue);
      // M3 扩批 XII：即时同步已暴露的 list 对象（getter.html 断言 `var cues = t1.cues`
      // 持旧引用，addCue 后 cues[0] 立即可见）。
      globalThis._zwTextTrackRebuildCueList(_cuesHolder, _cueArr);
    };
    // removeCue：不在本 track → NotFoundError（removeCue NOT_FOUND_ERR 断言面）。
    // M3 扩批（fixture-mounted 播放切片）：time-marches-on 推进面——实例上暴露内部
    // cue 数组引用（闭包数组本体，addCue/removeCue/_zwClearCues 实时可见）+ per-track
    // active 状态表。全局钩子 _zwMediaTimeMarchesOn 消费。
    track._zwCueArrInternal = _cueArr;
    track._zwMarchState = [];
    track.removeCue = function (cue) {
      var idx = -1;
      for (var i = 0; i < _cueArr.length; i++) { if (_cueArr[i] === cue) { idx = i; break; } }
      if (idx < 0) {
        throw new (globalThis.DOMException || Error)(
          'Failed to execute removeCue on TextTrack: The given cue is not listed in the textTrackList.',
          'NotFoundError');
      }
      _cueArr.splice(idx, 1);
      if (cue._zwTrack === track) cue._zwTrack = null;
      globalThis._zwTextTrackRebuildCueList(_cuesHolder, _cueArr);
    };
    // M3 扩批 XIII：track URL 变更 → cue list 清空（spec「track URL 变更」处理——
    // src-clear-cues 断言面；宿主 _zwTrackScheduleLoad 重调度时调用）。
    track._zwClearCues = function () {
      for (var i = 0; i < _cueArr.length; i++) {
        if (_cueArr[i] && _cueArr[i]._zwTrack === track) _cueArr[i]._zwTrack = null;
      }
      _cueArr.length = 0;
      globalThis._zwTextTrackRebuildCueList(_cuesHolder, _cueArr);
    };
    // EventTarget 面（oncuechange 断言面——on* 初值 null + 身份断言需 accessor）。
    globalThis._zwEnsureEventTarget(track);
    globalThis._zwDefineTargetOnHandler(track, 'cuechange');
    // M3 扩批 XII：cue 排序失效钩子——cue.startTime setter 修改后即时重排已暴露的
    // cues 列表（cues「changing order」断言面）。
    track._zwInvalidateCues = function () {
      globalThis._zwTextTrackRebuildCueList(_cuesHolder, _cueArr);
    };
    return track;
  };
  globalThis._zwMakeTextTrackList = function (tracks) {
    var arr = tracks || [];
    // M3 扩批 XII：TextTrackList 亦经索引只读 Proxy（video.textTracks[0]='foo' strict
    // TypeError——TextTrackList-getter「no indexed set/create (strict)」断言面）。
    var holder = { arr: arr.slice(), target: Object.create(globalThis.TextTrackList.prototype) };
    // M3 扩批 XIII：exposed 视图 = 索引只读 Proxy——dispatch 期 ev.target 须指 **exposed
    // proxy**（track-add-track 断言 event.target === video.textTracks 面；target 记号经
    // holder.self 供 _zwEnsureEventTarget 使用）。
    holder.self = globalThis._zwMakeIndexedListProxy(holder, globalThis.TextTrackList);
    var list = holder.target;
    // holder 引用暴露给内部同步面（addTextTrack 增量段 / 集合重建经 list._zwHolder.arr 写）。
    list._zwHolder = holder;
    // 索引 own-property 镜像（assert_array_equals hasOwnProperty 断言面）。
    globalThis._zwSyncListHolder(holder);
    list.item = function (i) { i = Number(i) | 0; return (i >= 0 && i < holder.arr.length) ? holder.arr[i] : null; };
    list.getTrackById = function (id) {
      for (var j = 0; j < holder.arr.length; j++) {
        if (holder.arr[j] && holder.arr[j].id === String(id)) return holder.arr[j];
      }
      return null;
    };
    // EventTarget 面（onaddtrack/onremovetrack 断言面——on* 初值 null 断言需要 accessor）。
    globalThis._zwEnsureEventTarget(list);
    globalThis._zwDefineTargetOnHandler(list, 'addtrack');
    globalThis._zwDefineTargetOnHandler(list, 'removetrack');
    return holder.self;
  };
  // M3 扩批 XIII：TextTrackList 增量 addtrack 派发（queued task——spec
  // text-tracks-in-media-elements；track-add-track 断言面：注册 handler 后 addTextTrack
  // 的同步增量仍异步收到）。event.track = 新增 TextTrack（TrackEvent init dict）。
  globalThis._zwFireTracksAdded = function (list, added) {
    var _deferFire = function (fn) {
      if (typeof queueMicrotask === 'function') queueMicrotask(fn);
      else if (typeof setTimeout === 'function') setTimeout(fn, 0);
      else fn();
    };
    for (var i = 0; i < added.length; i++) {
      (function (track) {
        _deferFire(function () {
          try {
            var ev = (typeof globalThis.TrackEvent === 'function')
              ? new globalThis.TrackEvent('addtrack', { track: track })
              : null;
            if (!ev) return;
            if (typeof list.dispatchEvent === 'function') list.dispatchEvent(ev);
          } catch (_eFta) {}
        });
      })(added[i]);
    }
  };

  function _zwMediaError(code, message) {
    var error = Object.create(globalThis.MediaError.prototype);
    error.code = Number(code) || 0; error.message = String(message || '');
    return error;
  }

  // scroll 事件由实际滚动派发，不受页面 JS 影响）；② 两参数恒为数值（IPC delta），免 `_zwApplyScroll` 的
  // 对象/Number 归一分支。host 经 `script_user_scroll` 注入，typeof 守卫防 shim 未安装时 ReferenceError。
  globalThis.__zw_user_scroll = function (dx, dy) {
    _zwApplyScroll(_winScroll, Number(dx) || 0, Number(dy) || 0, true);
    _zwFireScroll(null, null, null);
  };
  // R3254：宿主「视口尺寸变化」（renderer 收到 browser IPC SetViewportParams）注入钩子——更新
  // `innerWidth/innerHeight`（+ outer，headless outer≈inner）使响应式 JS 读到新尺寸 + 派 'resize' 事件
  // 到 window（window.addEventListener('resize') / innerWidth watcher / matchMedia 触发依赖）。spec：resize
  // 不冒泡（bubbles=false），派到 window（globalThis.dispatchEvent）。host 经 `script_user_resize` 注入。
  globalThis.__zw_user_resize = function (w, h) {
    w = Number(w) || 0; h = Number(h) || 0;
    if (w < 0) w = 0; if (h < 0) h = 0;
    globalThis.innerWidth = w; globalThis.innerHeight = h;
    globalThis.outerWidth = w; globalThis.outerHeight = h;
    try { if (typeof globalThis.dispatchEvent === 'function') globalThis.dispatchEvent(_makeEvent('resize')); } catch (_e) {}
    // R3255：resize 后重评估 matchMedia MQL——matches 翻转的派 'change'（响应式断点 JS 依赖）。typeof 守卫
    //（_zwFireMqlChanges 在 part05 定义，shim 完整加载后可见；运行时调用必已加载）。
    if (typeof _zwFireMqlChanges === 'function') _zwFireMqlChanges();
  };

  // window 弹窗 / 对话框 API（R2979）——alert/confirm/prompt/open 此前全缺，`if (confirm('Delete?'))` /
  // `alert(err)` / `prompt('Name')` / `window.open(url)` 抛 ReferenceError 中断后续脚本。headless 无 UI 用户
  // 交互 → spec 合规的 dismiss 语义：alert 返 undefined（不阻塞，real 浏览器阻塞 headless 无）；confirm 返 false
  //（无用户点 OK = dismiss）；prompt 返 null（无用户输入 = dismiss，spec）；open 返 null（headless 弹窗被阻 =
  // popup-blocked 语义，`if (win)` 守卫自然跳过）。modern 站点的离开页守卫 / 表单确认 / OAuth 弹窗高频。
  globalThis.alert = globalThis.alert || function alert(_message) {};
  globalThis.confirm = globalThis.confirm || function confirm(_message) { return false; };
  globalThis.prompt = globalThis.prompt || function prompt(_message, _defaultValue) { return null; };
  globalThis.open = globalThis.open || function open(_url, _target, _features) { return null; };
  // window.print() / window.stop()（R3246）—— HTML §4.5.6 / Window 接口
  //（https://html.spec.whatwg.org/multipage/window-object.html#dom-print / #dom-stop）。
  // print：提示用户打印页面（headless 无打印机 → no-op；不抛）。stop：中止文档加载（headless JS 执行时
  // 文档已加载完毕 → 无进行中加载可中止 → no-op；不抛）。两者此前全缺，`window.print()`（打印按钮 /
  // 发票 / 收据页高频）/ `window.stop()`（慢加载中止 / 广告拦截 / abort 逻辑）抛 TypeError 中断后续脚本。
  // 同 alert/confirm/prompt/open（R2979）的 headless dismiss/no-op 语义。guard `||` 幂等（不覆盖既有定义）。
  globalThis.print = globalThis.print || function print() {};
  globalThis.stop = globalThis.stop || function stop() {};

  // Performance API（R2768 now + R2821 mark/measure/entry buffer + PerformanceObserver）——
  // DOMHighResTimeStamp（ms，自 time origin 起单调）。host `__zw_performance_now` 返 elapsed ms（子毫秒）；
  // 未注册（polyfill/reftest 路径）走 Date.now() 兜底。mark/measure 产 PerformanceEntry 存 entry buffer，
  // 经 getEntries/getEntriesByType/getEntriesByName 读；PerformanceObserver observe 匹配 entryType 时
  // 经 _defer microtask 异步派发（execute 末 checkpoint，同 R2774/R2814）。analytics/RUM（web-vitals /
  // Sentry / GA）高频。
  function _perfNow() {
    return typeof __zw_performance_now === 'function'
      ? Number(__zw_performance_now())
      : (typeof Date.now === 'function' ? Date.now() : 0);
  }
  // entry buffer + mark startTime 表 + 活跃 observer 表（shim IIFE 内部，不污染 globalThis）。
  var _perfEntries = [];
  var _perfMarks = {};
  var _perfObservers = [];
  // 解析 measure 的 start/end 标记：undefined→（end 用 now / start 用 0）/ number→原值 / string→marks 表查
  // （查无抛 TypeError，spec 一致：measure 引用未注册 mark 名应报错；正确用法先 mark 后 measure）。
  function _resolveMarkTime(mark, isEnd) {
    if (mark === undefined) return isEnd ? _perfNow() : 0;
    if (typeof mark === 'number') return mark;
    if (Object.prototype.hasOwnProperty.call(_perfMarks, mark)) return _perfMarks[mark];
    throw new TypeError("Failed to execute 'measure' on 'Performance': The mark '" + mark + "' does not exist.");
  }
  // observer 派发用 entry list（getEntries/getEntriesByType/getEntriesByName over 传入快照）。
  function _makeObserverList(entries) {
    return {
      getEntries: function () { return entries.slice(); },
      getEntriesByType: function (t) {
        return entries.filter(function (e) { return e.entryType === t; });
      },
      getEntriesByName: function (n, t) {
        return entries.filter(function (e) { return e.name === n && (t === undefined || e.entryType === t); });
      },
    };
  }
  // 新 entry 入 buffer 时，向所有 observe 该 entryType 的活跃 observer 排队，每 observer 至多一个 microtask flush
  // （去抖：pending 期间累积，单次 flush 一次性派发全部 buffered）。
  function _notifyEntry(entry) {
    for (var i = 0; i < _perfObservers.length; i++) {
      var obs = _perfObservers[i];
      if (obs._types.indexOf(entry.entryType) !== -1) {
        obs._buffered.push(entry);
        if (!obs._pending) {
          obs._pending = true;
          (function (o) {
            _defer(function () {
              o._pending = false;
              var recs = o._buffered;
              o._buffered = [];
              o._cb(_makeObserverList(recs));
            });
          })(obs);
        }
      }
    }
  }

  globalThis.performance = globalThis.performance || {
    now: _perfNow,
    // timeOrigin = 0（相对原点：now() 返自原点起 elapsed ms；绝对 epoch 语义未提供，文档记录）。
    timeOrigin: 0,
    mark: function (name) {
      var entry = { name: String(name), entryType: 'mark', startTime: _perfNow(), duration: 0 };
      _perfEntries.push(entry);
      _perfMarks[entry.name] = entry.startTime;
      _notifyEntry(entry);
      return entry;
    },
    measure: function (name, startMark, endMark) {
      var start = _resolveMarkTime(startMark, false);
      var end = _resolveMarkTime(endMark, true);
      var entry = { name: String(name), entryType: 'measure', startTime: start, duration: end - start };
      _perfEntries.push(entry);
      _notifyEntry(entry);
      return entry;
    },
    getEntries: function () { return _perfEntries.slice(); },
    getEntriesByType: function (type) {
      return _perfEntries.filter(function (e) { return e.entryType === type; });
    },
    getEntriesByName: function (name, type) {
      return _perfEntries.filter(function (e) {
        return e.name === name && (type === undefined || e.entryType === type);
      });
    },
    clearMarks: function (name) {
      _perfEntries = _perfEntries.filter(function (e) {
        return !(e.entryType === 'mark' && (name === undefined || e.name === name));
      });
      if (name === undefined) { _perfMarks = {}; }
      else { delete _perfMarks[name]; }
    },
    clearMeasures: function (name) {
      _perfEntries = _perfEntries.filter(function (e) {
        return !(e.entryType === 'measure' && (name === undefined || e.name === name));
      });
    },
  };

  // PerformanceObserver（R2821）——观察 performance entry（mark/measure/longtask/paint/navigation/resource 等）。
  // observe({entryTypes:[...]} 或 {type:'...'}) 注册 entryType；新 entry 经 _notifyEntry 排队，每 observer 至多
  // 一个 _defer microtask flush（spec 为任务队列派发，sandbox 经 execute 末 microtask 近似）；disconnect 移出
  // 活跃表停止派发；takeRecords 取并清缓冲；supportedEntryTypes 静态（feature-detect 高频）。
  function PerformanceObserver(callback) {
    this._cb = callback;
    this._types = [];
    this._buffered = [];
    this._pending = false;
  }
  PerformanceObserver.prototype.observe = function (options) {
    var t = (options && options.entryTypes)
      ? options.entryTypes
      : (options && options.type ? [options.type] : []);
    for (var i = 0; i < t.length; i++) {
      if (this._types.indexOf(t[i]) === -1) this._types.push(t[i]);
    }
    if (_perfObservers.indexOf(this) === -1) _perfObservers.push(this);
  };
  PerformanceObserver.prototype.disconnect = function () {
    this._types = [];
    this._buffered = [];
    var idx = _perfObservers.indexOf(this);
    if (idx !== -1) _perfObservers.splice(idx, 1);
  };
  PerformanceObserver.prototype.takeRecords = function () {
    var r = this._buffered;
    this._buffered = [];
    return r;
  };
  PerformanceObserver.supportedEntryTypes = ['element', 'event', 'first-input', 'largest-contentful-paint', 'longtask', 'mark', 'measure', 'navigation', 'paint', 'resource'];
  globalThis.PerformanceObserver = PerformanceObserver;

  // DOMException——Web IDL 异常类型（name + message + legacy code）。众多 Web API 抛出它（fetch /
  // storage / atob / crypto / structuredClone 等），各 API 用 name 子类区分语义（InvalidCharacterError
  // / DataCloneError / QuotaExceededError 等）。V8 embed 不提供，polyfill 之（本地 Chromium 150 oracle
  // 锚定 R2776）。**关键行为（oracle 锚定）**：无 name 参数时 name='Error'/code=0；name∈legacy 表时
  // code=对应值（余 0）；instance 同时是 DOMException 与 Error；toString="name: message"。
  var _ZW_DE_CODE = {
    IndexSizeError: 1, HierarchyRequestError: 3, WrongDocumentError: 4,
    InvalidCharacterError: 5, NoModificationAllowedError: 7, NotFoundError: 8,
    NotSupportedError: 9, InUseAttributeError: 10, InvalidStateError: 11,
    SyntaxError: 12, InvalidModificationError: 13, NamespaceError: 14,
    InvalidAccessError: 15, TypeMismatchError: 17, SecurityError: 18,
    NetworkError: 19, AbortError: 20, URLMismatchError: 21, QuotaExceededError: 22,
    TimeoutError: 23, InvalidNodeTypeError: 24, DataCloneError: 25
  };
  // R209：code → legacy 常量名反查表（实例挂点消费，见 DOMException 构造器内注释）。
  var _ZW_DE_LEGACY_BY_CODE = {
    1: 'INDEX_SIZE_ERR', 2: 'DOMSTRING_SIZE_ERR', 3: 'HIERARCHY_REQUEST_ERR',
    4: 'WRONG_DOCUMENT_ERR', 5: 'INVALID_CHARACTER_ERR', 6: 'NO_DATA_ALLOWED_ERR',
    7: 'NO_MODIFICATION_ALLOWED_ERR', 8: 'NOT_FOUND_ERR', 9: 'NOT_SUPPORTED_ERR',
    10: 'INUSE_ATTRIBUTE_ERR', 11: 'INVALID_STATE_ERR', 12: 'SYNTAX_ERR',
    13: 'INVALID_MODIFICATION_ERR', 14: 'NAMESPACE_ERR', 15: 'INVALID_ACCESS_ERR',
    16: 'VALIDATION_ERR', 17: 'TYPE_MISMATCH_ERR', 18: 'SECURITY_ERR',
    19: 'NETWORK_ERR', 20: 'ABORT_ERR', 21: 'URL_MISMATCH_ERR', 22: 'QUOTA_EXCEEDED_ERR',
    23: 'TIMEOUT_ERR', 24: 'INVALID_NODE_TYPE_ERR', 25: 'DATA_CLONE_ERR'
  };
  function DOMException(message, name) {
    // 允许无 new 调用（同 Error 语义）。
    var self = (this instanceof DOMException) ? this : Object.create(DOMException.prototype);
    self.message = (message === undefined) ? '' : String(message);
    self.name = (name === undefined) ? 'Error' : String(name);
    self.code = _ZW_DE_CODE[self.name] || 0;
    // R209（js-dom M4）：legacy code 常量挂**实例**（可枚举）——WPT dom/common.js
    // getDomExceptionName 经 `for (prop in e)` 找 `/^[A-Z_]+_ERR$/` 且值 === e.code
    // 的 prop 反查异常名（mega-case 的模拟异常消费路径）；真浏览器实例经原型链
    // 可枚举可达这些 legacy 常量。只挂 code≠0 对应的一个（避免实例污染全 25 常量）。
    if (self.code) {
      var _r209Legacy = _ZW_DE_LEGACY_BY_CODE[self.code];
      if (_r209Legacy) self[_r209Legacy] = self.code;
    }
    return self;
  }
  DOMException.prototype = Object.create(Error.prototype);
  DOMException.prototype.constructor = DOMException;
  DOMException.prototype.toString = function () {
    return this.message === '' ? this.name : this.name + ': ' + this.message;
  };
  // legacy 常量（Web IDL §1.2 code 值；部分码无现代 name，仅常量）。
  DOMException.INDEX_SIZE_ERR = 1;
  DOMException.DOMSTRING_SIZE_ERR = 2;
  DOMException.HIERARCHY_REQUEST_ERR = 3;
  DOMException.WRONG_DOCUMENT_ERR = 4;
  DOMException.INVALID_CHARACTER_ERR = 5;
  DOMException.NO_DATA_ALLOWED_ERR = 6;
  DOMException.NO_MODIFICATION_ALLOWED_ERR = 7;
  DOMException.NOT_FOUND_ERR = 8;
  DOMException.NOT_SUPPORTED_ERR = 9;
  DOMException.INUSE_ATTRIBUTE_ERR = 10;
  DOMException.INVALID_STATE_ERR = 11;
  DOMException.SYNTAX_ERR = 12;
  DOMException.INVALID_MODIFICATION_ERR = 13;
  DOMException.NAMESPACE_ERR = 14;
  DOMException.INVALID_ACCESS_ERR = 15;
  DOMException.VALIDATION_ERR = 16;
  DOMException.TYPE_MISMATCH_ERR = 17;
  DOMException.SECURITY_ERR = 18;
  DOMException.NETWORK_ERR = 19;
  DOMException.ABORT_ERR = 20;
  DOMException.URL_MISMATCH_ERR = 21;
  DOMException.QUOTA_EXCEEDED_ERR = 22;
  DOMException.TIMEOUT_ERR = 23;
  DOMException.INVALID_NODE_TYPE_ERR = 24;
  DOMException.DATA_CLONE_ERR = 25;
  globalThis.DOMException = globalThis.DOMException || DOMException;

  // spec DOM `validate`（Name production，`dom-document-createelement`）：判定 String(tag) 后的名
  // 是否合法 createElement 标签名。与 native dom_bindings is_valid_qualified_name 逻辑对齐（A/B 等价）：
  // 空串 → 非法；首字符须 name-start（ASCII 字母 / `_` / `:` / 非 ASCII）；后续须 name-char（name-start
  // 或数字 / `-` / `.`）。createElement(undefined)→"undefined" 合法通过（WPT valid 列表）。
  // R135（js-dom M4）：name 校验族按 WPT name-validation.html 的 **spec regex** 重写
  //（dom spec valid local-name regex：
  //   /^(?:[A-Za-z][^\0\t\n\f\r\u0020/>]*|[:_\u0080-\u{10FFFF}][A-Za-z0-9-.:_\u0080-\u{10FFFF}]*)$/u）
  // 关键语义：首字符 ASCII 字母 → 后续**任意**（NUL/ASCII 空白/TAB/LF/FF/CR/'/'/'>' 除外——
  // 注意 \x0B 垂直制表不在 ASCII whitespace 五字符内故合法）；首字符 ':'/'_'/>=0x80 →
  // 后续限 NameChar 集。旧实现的偏差：① JS /\s/ 含 \x0B/\x85/\u2028（非 XML 空白）误拒
  // ② >=0x80 的 emoji 在首字符位合法（spec regex 含）——旧 NameStartChar 语义恰好同
  // ③ qualified name（NS 族）= 本 regex 首段约束 prefix + ':' + 后段约束 local。
  var _r135NameRegex = /^(?:[A-Za-z][^\0\t\n\f\r\u0020/>]*|[:_\u0080-\u{10FFFF}][A-Za-z0-9-.:_\u0080-\u{10FFFF}]*)$/u;
  var _r135NameCharRegex = /^[A-Za-z0-9\-.:_\u0080-\u{10FFFF}]$/u;
  function _zwIsNameStartChar(c) {
    return /^[A-Za-z:_]/.test(c) || c.charCodeAt(0) >= 0x80;
  }
  function _zwIsNameChar(c) {
    return _zwIsNameStartChar(c) || /^[0-9.\-]$/.test(c);
  }
  function _zwIsValidQualifiedName(name) {
    if (name === '') return false;
    var chars = Array.from(name);
    if (!_zwIsNameStartChar(chars[0])) return false;
    for (var i = 1; i < chars.length; i++) {
      if (!_zwIsNameChar(chars[i])) return false;
    }
    return true;
  }
  // R135：spec regex 直判（HTML createElement / local-name / attribute name 通用）。
  function _r135IsValidName(name) {
    return _r135NameRegex.test(String(name));
  }
  // R135：NS 族 qualified name = prefix 段 + ':' + local 段，两段各自过 spec regex
  //（WPT name-validation：invalid prefix/local 集与 createElement 同源 + '='/'/'/'>'
  // 的 per-part 差异）。无冒号 = 单段名过 regex。
  function _r135IsValidQualifiedNameSpec(qname) {
    var s0 = String(qname);
    if (s0 === '') return false;
    var c = s0.indexOf(':');
    if (c < 0) return _r135IsValidName(s0);
    var pre = s0.slice(0, c), local = s0.slice(c + 1);
    if (local.indexOf(':') >= 0) return false; // 多冒号 malformed（前缀段不含冒号）
    return _r135IsValidName(pre) && local !== '' && _r135IsValidName(local);
  }
  // js-dom M4 R81：HTML createElement 的校验面（WPT Document-createElement valid 列表）——
  // 比 QName 宽：Name production（HTML any-name——`'}'`、`'<'`、`'\uffff'` 等在**非首字符**
  // 合法；首字符限制同 NameStartChar）。区别：QName 校验（createElementNS）拒绝这些；HTML
  // createElement 只要求整体是 Name（浏览器 HTML parser 的宽容性）。首字符仍须 NameStartChar
  // （"1foo"/"}foo"/"<foo" invalid）。
  // R135：attribute 名校验（WPT name-validation attribute 名单语义）——比 element 名宽：
  // **无首字符限制**（'\x01'/数字/控制字符开头都合法）；invalid 集 = NUL + ASCII 空白五字符
  // （0x9/0xA/0xC/0xD/0x20）+ '/'(0x2F) + '>'(0x3E) + '='(0x3D)。':' 合法（非 NS 限定名）。
  var _r135AttrNameRegex = /^[^\0\t\n\f\r\u0020/>=]+$/u;
  function _r135IsValidAttrName(name) {
    return _r135AttrNameRegex.test(String(name));
  }
  // R135：NS attribute 的 qualified name 校验（WPT name-validation NS attribute 名单）——
  // prefix 段：NUL/ASCII 空白/'/'/'>'/':' invalid（'=' 合法）；local 段：NUL/ASCII 空白/
  // '/'/'>'/'=' invalid（':' 合法）。两段都无首字符限制（比 element 宽——'\x01:attr' 合法）。
  function _r135IsValidAttrQNameSpec(qname) {
    var s0 = String(qname);
    if (s0 === '') return false;
    var c = s0.indexOf(':');
    var pre = c >= 0 ? s0.slice(0, c) : null;
    var local = c >= 0 ? s0.slice(c + 1) : s0;
    var bad = /[\u0000\u0009\u000A\u000C\u000D\u0020/>]/;
    if (pre !== null) {
      if (pre === '' || bad.test(pre) || pre.indexOf(':') >= 0) return false;
    }
    return local !== '' && !bad.test(local) && local.indexOf('=') < 0;
  }
  function _zwIsValidHtmlElementName(name) {
    // R135：改走 spec regex（_r135IsValidName）——旧 /[\s>]/ 拒绝把 \x0B（非 XML
    // 空白）误判 invalid（WPT name-validation "A\x0B" valid）。
    return _r135IsValidName(name);
  }


  // atob/btoa——Base64 编解码（Web 平台高频：data: URL / JWT / 二进制载荷）。纯 JS（ZW 无 base64
  // crate 在 engine，复用 fetch _b64decode 同款算法）。btoa 对 >255（非 Latin-1）抛 InvalidCharacterError
  // DOMException（spec，R2776 升级自裸 Error）；atob 容错（忽略空白/padding，best-effort）。多字节 UTF-8
  // base64 为已知限制（返 Latin-1）。
  var _b64ch = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
  var _b64lut = (function () { var l = {}; for (var i = 0; i < 64; i++) l[_b64ch[i]] = i; return l; })();
  globalThis.btoa = function (s) {
    s = String(s);
    var out = '';
    for (var i = 0; i < s.length; i += 3) {
      var b1 = s.charCodeAt(i), b2 = s.charCodeAt(i + 1), b3 = s.charCodeAt(i + 2);
      if (b1 > 255 || b2 > 255 || b3 > 255)
        throw new DOMException('The string to be encoded contains characters outside of the Latin1 range.', 'InvalidCharacterError');
      out += _b64ch[b1 >> 2];
      out += _b64ch[((b1 & 3) << 4) | (isNaN(b2) ? 0 : b2 >> 4)];
      out += isNaN(b2) ? '=' : _b64ch[((b2 & 15) << 2) | (isNaN(b3) ? 0 : b3 >> 6)];
      out += isNaN(b3) ? '=' : _b64ch[b3 & 63];
    }
    return out;
  };
  globalThis.atob = function (s) {
    s = String(s).replace(/\s+/g, '').replace(/=+$/, '');
    var out = '';
    for (var i = 0; i < s.length; i += 4) {
      var b0 = _b64lut[s[i]] || 0, b1 = _b64lut[s[i + 1]] || 0;
      var b2 = _b64lut[s[i + 2]], b3 = _b64lut[s[i + 3]];
      out += String.fromCharCode((b0 << 2) | (b1 >> 4));
      if (s[i + 2] !== undefined) out += String.fromCharCode(((b1 & 15) << 4) | ((b2 || 0) >> 2));
      if (s[i + 3] !== undefined) out += String.fromCharCode((((b2 || 0) & 3) << 6) | (b3 || 0));
    }
    return out;
  };

  // crypto——Web Crypto 随机源：randomUUID（UUID v4）+ getRandomValues（TypedArray 填充）。高频
  //（id 生成 / CSRF token / analytics / 密钥/IV 随机）。R2960 升级 CSPRNG：经 host
  // `__zw_crypto_get_random_values(n)`（getrandom crate，OS 随机）；host 未注册（engine polyfill / reftest 路径）
  // → 回退 Math.random（非 CSPRNG，仅非安全场景）。
  // 填 view 字节：host 足量则 OS-random，否则 Math.random 回退。
  function _zw_randomFill(view) {
    var csv = (typeof __zw_crypto_get_random_values === 'function')
      ? __zw_crypto_get_random_values(view.length) : '';
    var parts = csv ? csv.split(',') : null;
    if (parts && parts.length >= view.length) {
      for (var i = 0; i < view.length; i++) view[i] = +parts[i] & 0xff;
    } else {
      for (var k = 0; k < view.length; k++) view[k] = (Math.random() * 256) | 0;
    }
    return view;
  }
  globalThis.crypto = globalThis.crypto || {
    // randomUUID（UUID v4，RFC 4122）：16 随机字节（_zw_randomFill，OS-random R2960），设 version(4)/variant
    // 位，格式化 8-4-4-4-12 hex。spec：time_hi_and_version 高 4 位=4，clock_seq_hi variant=10xxxxxx。
    randomUUID: function () {
      var b = new Uint8Array(16);
      _zw_randomFill(b);
      b[6] = (b[6] & 0x0f) | 0x40; // version 4
      b[8] = (b[8] & 0x3f) | 0x80; // variant 10xxxxxx（y ∈ 8,9,a,b）
      var h = '0123456789abcdef';
      var s = '';
      for (var i = 0; i < 16; i++) {
        s += h[(b[i] >> 4) & 0xf] + h[b[i] & 0xf];
        if (i === 3 || i === 5 || i === 7 || i === 9) s += '-';
      }
      return s;
    },
    // getRandomValues(typedArray)：spec 限定 TypedArray（Int8..Uint32 / BigInt64/BigUint64），≤65536
    // 字节。填**底层字节 buffer**（Uint8Array 视图）→ 任意 typed 视图得随机值（含多字节 / 共享 buffer 偏移）。
    getRandomValues: function (arr) {
      if (typeof ArrayBuffer === 'undefined' || !ArrayBuffer.isView(arr)) {
        throw new TypeError('getRandomValues: argument must be a TypedArray');
      }
      if (arr.byteLength > 65536)
        throw new DOMException("The ArrayBufferView byte length (" + arr.byteLength + ") exceeds 65536.", 'QuotaExceededError');
      _zw_randomFill(new Uint8Array(arr.buffer, arr.byteOffset, arr.byteLength));
      return arr;
    }
  };

  // BufferSource → number[]（字节值，0-255）：ArrayBuffer / TypedArray / DataView / array-like / string（经
  // TextEncoder）。供 crypto.subtle.digest 把 data 传 host（逗号分隔十进制串，避免 UTF-8 编码歧义）。
  function _zw_bufToBytes(data) {
    if (typeof data === 'string') data = new TextEncoder().encode(data);
    if (data == null) return [];
    var view;
    if (data instanceof ArrayBuffer) view = new Uint8Array(data);
    else if (data && data.buffer) view = new Uint8Array(data.buffer, data.byteOffset || 0, data.byteLength != null ? data.byteLength : data.length);
    else if (typeof data.length === 'number') view = data; // array-like（含 TypedArray 已覆盖上一支）
    else return [];
    var out = [];
    for (var i = 0; i < view.length; i++) out.push(view[i] & 0xff);
    return out;
  }
  // crypto.subtle（R2793 digest + R2955 HMAC sign/verify/importKey）。digest 委托 host
  // `__zw_crypto_subtle_digest`；HMAC sign/verify 委托 `__zw_crypto_subtle_hmac`（手写 HMAC，复用 sha1/sha2）。
  // **scope**：digest 全 hash；HMAC 全 hash（importKey raw + sign + verify）；其余 WebCrypto（RSA/ECDSA/AES/
  // HKDF/PBKDF2/jwk/exportKey）仍 defer——大表面，HMAC 为对称 MAC 最高频子集（JWT HS256 / 请求签名 / webhook 校验）。
  // https://w3c.github.io/webcrypto/#SubtleCrypto-method-sign  https://datatracker.ietf.org/doc/html/rfc2104

  // CryptoKey——密钥对象（importKey 返回值）。type（"secret"/"public"/"private"）+ extractable +
  // algorithm（归一化对象）+ usages（字符串数组）。HMAC 密钥材料存 `_raw`（字节 number[]）——polyfill：host
  // 每次 sign/verify 用，不在 host 持久化（headless 简化）；`extractable=false` 时 exportKey 仍 defer，
  // 故 _raw 技术可访问（exportKey 未实现，无泄漏面）。
  // https://w3c.github.io/webcrypto/#CryptoKey-interface
  function CryptoKey(type, extractable, algorithm, usages, raw) {
    this.type = type;
    this.extractable = !!extractable;
    this.algorithm = algorithm;
    this.usages = usages;
    this._raw = raw || null;
  }

  // hash 名归一化：接受串或 {name:...} → 大写 "SHA-XXX"，或 null。
  function _zw_hashName(h) {
    var n = (typeof h === 'object' && h) ? h.name : h;
    if (n == null) return null;
    return String(n).toUpperCase();
  }

  // importKey 的 algorithm 归一化：{name:"HMAC", hash:"SHA-XXX"} / {name:"PBKDF2"} / null（unsupported）。
  // HMAC 需 hash；PBKDF2 不需（hash 在 deriveBits 参数里）。
  function _zw_normalizeImportAlgorithm(algo) {
    if (!algo) return null;
    var name = (typeof algo === 'object' && algo) ? algo.name : algo;
    if (!name) return null;
    name = String(name).toUpperCase();
    if (name === 'HMAC') {
      var hash = _zw_hashName(typeof algo === 'object' ? algo.hash : null);
      if (!hash) return null;
      return { name: 'HMAC', hash: hash };
    }
    if (name === 'PBKDF2') {
      return { name: 'PBKDF2' };
    }
    if (name === 'AES-GCM') {
      return { name: 'AES-GCM' };
    }
    if (name === 'HKDF') {
      return { name: 'HKDF' };
    }
    return null;
  }

  // usages 归一化：去重 + 仅保留 allowed 内项；含非法项 → null（reject SyntaxError）。
  function _zw_normalizeUsages(usages, allowed) {
    if (usages == null) usages = [];
    if (typeof usages.length !== 'number') return null;
    var out = [], seen = {};
    for (var i = 0; i < usages.length; i++) {
      var u = String(usages[i]);
      if (allowed.indexOf(u) < 0) return null;
      if (!seen[u]) {
        seen[u] = 1;
        out.push(u);
      }
    }
    return out;
  }

  // HMAC MAC 计算（sign/verify 复用）：返 Uint8Array；host 未注册 / unsupported hash → 调 reject 返 null。
  function _zw_hmacMac(algo, key, dataBytes, reject) {
    if (typeof __zw_crypto_subtle_hmac !== 'function') {
      reject(new DOMException('crypto.subtle HMAC requires host callback', 'NotSupportedError'));
      return null;
    }
    var keyCsv = (key._raw || []).map(String).join(',');
    var macCsv = __zw_crypto_subtle_hmac(algo.hash, keyCsv, dataBytes.join(','));
    if (!macCsv) {
      reject(new DOMException("Unsupported HMAC hash: '" + algo.hash + "'", 'NotSupportedError'));
      return null;
    }
    var parts = macCsv.split(',');
    var arr = new Uint8Array(parts.length);
    for (var i = 0; i < parts.length; i++) arr[i] = +parts[i];
    return arr;
  }

  // AES-GCM 调用（encrypt/decrypt 共用）：校验 iv(12B)/tagLength(128)/host 后调 `__zw_crypto_subtle_aes_gcm`，
  // 返 Uint8Array；失败调 reject 返 null。AAD 经 algorithm.additionalData（可选）。
  function _zw_aesGcmCall(op, algorithm, key, dataBytes, reject) {
    var iv = _zw_bufToBytes(algorithm.iv);
    if (iv.length !== 12) {
      reject(new DOMException('AES-GCM iv must be 12 bytes (96 bits)', 'OperationError')); return null;
    }
    if (algorithm.tagLength != null && (algorithm.tagLength | 0) !== 128) {
      reject(new DOMException('Only AES-GCM tagLength=128 supported', 'NotSupportedError')); return null;
    }
    var aadBytes = algorithm.additionalData != null ? _zw_bufToBytes(algorithm.additionalData) : [];
    if (typeof __zw_crypto_subtle_aes_gcm !== 'function') {
      reject(new DOMException('crypto.subtle AES-GCM requires host callback', 'NotSupportedError')); return null;
    }
    var keyCsv = (key._raw || []).map(String).join(',');
    var out = __zw_crypto_subtle_aes_gcm(op, keyCsv, iv.join(','), dataBytes.join(','), aadBytes.join(','));
    if (!out) {
      reject(new DOMException('AES-GCM ' + op + ' failed (bad key/iv/tag)', 'OperationError')); return null;
    }
    var parts = out.split(',');
    var res = new Uint8Array(parts.length);
    for (var i = 0; i < parts.length; i++) res[i] = +parts[i];
    return res;
  }

  // 派生核心（deriveBits/deriveKey 共用）：PBKDF2/HKDF 分派 + host 调用 + csv→arr，**不做 usage 校验**
  //（usage 校验由调用方负责——deriveBits 检 "deriveBits"，deriveKey 检 "deriveKey"）。
  function _zw_performDerive(algorithm, key, length) {
    return new Promise(function (resolve, reject) {
      var name = (typeof algorithm === 'object' && algorithm) ? algorithm.name : algorithm;
      name = String(name == null ? '' : name).toUpperCase();
      var dkLen = length / 8;
      var keyCsv = (key._raw || []).map(String).join(',');
      var hash = _zw_hashName(typeof algorithm === 'object' ? algorithm.hash : null);
      var saltBytes = _zw_bufToBytes(algorithm.salt);
      var out = '';
      if (name === 'PBKDF2') {
        var iters = Math.floor(Number(algorithm.iterations));
        if (!hash || !(iters > 0)) {
          reject(new DOMException('PBKDF2 requires salt/iterations/hash', 'OperationError')); return;
        }
        if (typeof __zw_crypto_subtle_pbkdf2 !== 'function') {
          reject(new DOMException('crypto.subtle deriveBits requires host callback', 'NotSupportedError')); return;
        }
        out = __zw_crypto_subtle_pbkdf2(hash, keyCsv, saltBytes.join(','), String(iters), String(dkLen));
      } else { // HKDF
        if (!hash || typeof __zw_crypto_subtle_hkdf !== 'function') {
          reject(new DOMException('HKDF requires hash + host callback', 'NotSupportedError')); return;
        }
        var infoBytes = _zw_bufToBytes(algorithm.info);
        out = __zw_crypto_subtle_hkdf(hash, keyCsv, saltBytes.join(','), infoBytes.join(','), String(dkLen));
      }
      if (!out) {
        reject(new DOMException("Unsupported deriveBits hash: '" + hash + "'", 'NotSupportedError')); return;
      }
      var parts = out.split(',');
      var arr = new Uint8Array(parts.length);
      for (var i = 0; i < parts.length; i++) arr[i] = +parts[i];
      resolve(arr);
    });
  }

  // 派生/生成的目标密钥长度（位）。AES → 256（spec 默认）；HMAC → hash 块大小（SHA-1/256=512，SHA-384/512=1024）。
  function _zw_keyLengthBits(algo) {
    var n = String((typeof algo === 'object' && algo) ? algo.name : algo).toUpperCase();
    if (n === 'AES-GCM' || n === 'AES-CBC' || n === 'AES-CTR' || n === 'AES-KW') return 256;
    if (n === 'HMAC') {
      var h = _zw_hashName(typeof algo === 'object' ? algo.hash : null);
      return (h === 'SHA-384' || h === 'SHA-512') ? 1024 : 512;
    }
    return 0; // 未知
  }

  // n 个随机字节（Uint8Array）。复用 crypto.getRandomValues（R2770，**Math.random 非 CSPRNG**——安全敏感场景已知限制）。
  function _zw_randomBytes(n) {
    var a = new Uint8Array(n);
    if (typeof crypto !== 'undefined' && crypto.getRandomValues) crypto.getRandomValues(a);
    else for (var i = 0; i < n; i++) a[i] = (Math.random() * 256) | 0;
    return a;
  }

  globalThis.crypto.subtle = globalThis.crypto.subtle || {
    digest: function (algo, data) {
      var a = (typeof algo === 'object' && algo) ? algo.name : algo;
      a = (a == null ? '' : String(a)).toUpperCase();
      return new Promise(function (resolve, reject) {
        var bytes = _zw_bufToBytes(data);
        if (typeof __zw_crypto_subtle_digest !== 'function') {
          reject(new DOMException('crypto.subtle.digest requires host callback', 'NotSupportedError'));
          return;
        }
        var out = __zw_crypto_subtle_digest(a, bytes.join(','));
        if (!out) {
