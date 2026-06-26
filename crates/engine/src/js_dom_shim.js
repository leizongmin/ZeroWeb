(function() {
  var _listenerStore = {};

  // ── 浏览器运行时桩（定时器、navigator、location 等）──
  var _timerId = 1;
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

  globalThis.__zw_begin_script = function() {
    _deferBudget = 256;
  };

  globalThis.setTimeout = function(fn, _delay) {
    if (typeof fn === 'function') _defer(fn);
    return _timerId++;
  };
  globalThis.setInterval = function(fn, _delay) {
    if (typeof fn === 'function') _defer(fn);
    return _timerId++;
  };
  globalThis.clearTimeout = function(_id) {};
  globalThis.clearInterval = function(_id) {};

  function _emptyCollection() {
    return { length: 0, item: function() { return null; }, namedItem: function() { return null; } };
  }

  function _parseLocation(href) {
    var m = String(href || 'about:blank').match(/^([^:]+):\/\/([^\/]*)(\/[^?#]*)?(\?[^#]*)?(#.*)?$/);
    if (!m) {
      return { href: href || 'about:blank', protocol: 'about:', host: '', hostname: '', pathname: '/', search: '', hash: '', origin: 'null' };
    }
    var host = m[2] || '';
    var hostname = host.split(':')[0] || '';
    return {
      href: href,
      protocol: m[1] + ':',
      host: host,
      hostname: hostname,
      pathname: m[3] || '/',
      search: m[4] || '',
      hash: m[5] || '',
      origin: host ? m[1] + '://' + host : 'null'
    };
  }

  function _makeLocation() {
    function href() {
      return typeof __zw_get_page_url === 'function' ? __zw_get_page_url() : 'about:blank';
    }
    return {
      get href() { return href(); },
      get protocol() { return _parseLocation(href()).protocol; },
      get host() { return _parseLocation(href()).host; },
      get hostname() { return _parseLocation(href()).hostname; },
      get pathname() { return _parseLocation(href()).pathname; },
      get search() { return _parseLocation(href()).search; },
      get hash() { return _parseLocation(href()).hash; },
      get origin() { return _parseLocation(href()).origin; },
      assign: function() {},
      replace: function() {},
      reload: function() {},
      toString: function() { return href(); }
    };
  }

  globalThis.location = _makeLocation();
  globalThis.self = globalThis;
  globalThis.top = globalThis;
  globalThis.parent = globalThis;

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

  globalThis.history = {
    length: 1,
    state: null,
    back: function() {},
    forward: function() {},
    go: function() {},
    pushState: function() {},
    replaceState: function() {}
  };

  globalThis.Worker = function() {
  };

  globalThis.navigator = {
    userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) ZeroBrowser/0.1 Chrome/120.0.0.0',
    appName: 'Netscape',
    appVersion: '5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
    appCodeName: 'Mozilla',
    product: 'Gecko',
    productSub: '20030107',
    vendor: 'Google Inc.',
    vendorSub: '',
    platform: 'Win32',
    language: 'en-US',
    languages: ['en-US', 'en'],
    onLine: true,
    cookieEnabled: true,
    doNotTrack: null,
    hardwareConcurrency: 4,
    maxTouchPoints: 0,
    webdriver: false,
    plugins: _emptyCollection(),
    mimeTypes: _emptyCollection(),
    javaEnabled: function() { return false; },
    taintEnabled: function() { return false; }
  };

  globalThis.console = globalThis.console || {
    log: function() {},
    info: function() {},
    warn: function() {},
    error: function() {},
    debug: function() {},
    trace: function() {},
    dir: function() {},
    clear: function() {},
    count: function() {},
    group: function() {},
    groupEnd: function() {},
    table: function() {}
  };

  globalThis.Image = function() {
    return { src: '', width: 0, height: 0, onload: null, onerror: null, onabort: null };
  };

  function _createStorage() {
    var _data = {};
    return {
      getItem: function(key) { return _data.hasOwnProperty(key) ? _data[key] : null; },
      setItem: function(key, value) { _data[String(key)] = String(value); },
      removeItem: function(key) { delete _data[String(key)]; },
      clear: function() { _data = {}; },
      key: function(index) {
        var keys = Object.keys(_data);
        return index >= 0 && index < keys.length ? keys[index] : null;
      },
      get length() { return Object.keys(_data).length; }
    };
  }

  globalThis.localStorage = _createStorage();
  globalThis.sessionStorage = _createStorage();

  globalThis.XMLHttpRequest = function() {
    var self = this;
    self.readyState = 0;
    self.status = 0;
    self.statusText = '';
    self.responseText = '';
    self.response = '';
    self.onreadystatechange = null;
    self.onload = null;
    self.onerror = null;
    self.open = function(_method, _url) { self.readyState = 1; };
    self.send = function(_body) {
      self.readyState = 4;
      self.status = 404;
      self.statusText = 'Not Found';
      if (typeof self.onreadystatechange === 'function') self.onreadystatechange();
      if (typeof self.onload === 'function') self.onload();
    };
    self.abort = function() {};
    self.setRequestHeader = function() {};
    self.getResponseHeader = function() { return null; };
    self.getAllResponseHeaders = function() { return ''; };
  };

  function _ieEventType(type) {
    var s = String(type);
    return s.indexOf('on') === 0 ? s.slice(2) : s;
  }

  function _attachEventForKey(key, type, fn) {
    var t = _ieEventType(type);
    if (!_listenerStore[key]) _listenerStore[key] = {};
    if (!_listenerStore[key][t]) _listenerStore[key][t] = [];
    _listenerStore[key][t].push({ fn: fn, capture: false });
  }

  function _detachEventForKey(key, type, fn) {
    var t = _ieEventType(type);
    if (!_listenerStore[key] || !_listenerStore[key][t]) return;
    _listenerStore[key][t] = _listenerStore[key][t].filter(function(l) { return l.fn !== fn; });
  }

  function _globalAddEventListener(type, fn, opts) {
    var key = _elKey('html', null);
    var t = String(type);
    if (!_listenerStore[key]) _listenerStore[key] = {};
    if (!_listenerStore[key][t]) _listenerStore[key][t] = [];
    _listenerStore[key][t].push({ fn: fn, capture: !!(opts && opts.capture) });
  }

  function _globalRemoveEventListener(type, fn) {
    var key = _elKey('html', null);
    var t = String(type);
    if (!_listenerStore[key] || !_listenerStore[key][t]) return;
    _listenerStore[key][t] = _listenerStore[key][t].filter(function(l) { return l.fn !== fn; });
  }

  globalThis.Node = function Node() {};
  globalThis.Element = function Element() {};
  globalThis.HTMLElement = function HTMLElement() {};
  globalThis.Node.prototype = {};
  globalThis.Element.prototype = Object.create(globalThis.Node.prototype);
  globalThis.HTMLElement.prototype = Object.create(globalThis.Element.prototype);
  globalThis.Element.prototype.addEventListener = function(type, fn, opts) {
    _globalAddEventListener(type, fn, opts);
  };
  globalThis.Element.prototype.removeEventListener = function(type, fn) {
    _globalRemoveEventListener(type, fn);
  };

  function _elKey(sel, handle) {
    return handle ? ('@' + handle) : sel;
  }

  function _classListProxy(sel, handle) {
    return {
      add: function(c) {
        var cur = handle ? __zw_get_attr_handle(handle, 'class') : __zw_get_attr(sel, 'class');
        var parts = (cur || '').split(/\s+/).filter(Boolean);
        if (parts.indexOf(c) < 0) parts.push(c);
        var v = parts.join(' ');
        if (handle) __zw_set_attr_handle(handle, 'class', v);
        else __zw_set_attr(sel, 'class', v);
      },
      remove: function(c) {
        var cur = handle ? __zw_get_attr_handle(handle, 'class') : __zw_get_attr(sel, 'class');
        var parts = (cur || '').split(/\s+/).filter(Boolean).filter(function(x) { return x !== c; });
        var v = parts.join(' ');
        if (handle) __zw_set_attr_handle(handle, 'class', v);
        else __zw_set_attr(sel, 'class', v);
      },
      toggle: function(c) {
        var cur = handle ? __zw_get_attr_handle(handle, 'class') : __zw_get_attr(sel, 'class');
        var parts = (cur || '').split(/\s+/).filter(Boolean);
        var i = parts.indexOf(c);
        if (i >= 0) parts.splice(i, 1);
        else parts.push(c);
        var v = parts.join(' ');
        if (handle) __zw_set_attr_handle(handle, 'class', v);
        else __zw_set_attr(sel, 'class', v);
      },
      contains: function(c) {
        var cur = handle ? __zw_get_attr_handle(handle, 'class') : __zw_get_attr(sel, 'class');
        return (cur || '').split(/\s+/).indexOf(c) >= 0;
      }
    };
  }

  function _dispatchToListeners(key, event) {
    var listeners = _listenerStore[key];
    if (!listeners || !listeners[event.type]) return true;
    var list = listeners[event.type];
    for (var i = 0; i < list.length; i++) {
      if (list[i].capture) {
        list[i].fn.call(event.target, event);
        if (event._immediateStopped) return !event._defaultPrevented;
      }
    }
    for (var j = 0; j < list.length; j++) {
      if (!list[j].capture) {
        list[j].fn.call(event.target, event);
        if (event._immediateStopped) return !event._defaultPrevented;
      }
    }
    return !event._defaultPrevented;
  }

  function _makeEvent(type, options) {
    var ev = {
      type: type,
      bubbles: !!(options && options.bubbles),
      cancelable: !!(options && options.cancelable),
      detail: options && options.detail,
      target: null,
      currentTarget: null,
      _defaultPrevented: false,
      _propagationStopped: false,
      _immediateStopped: false,
      preventDefault: function() { if (this.cancelable) this._defaultPrevented = true; },
      stopPropagation: function() { this._propagationStopped = true; },
      stopImmediatePropagation: function() {
        this._immediateStopped = true;
        this._propagationStopped = true;
      }
    };
    return ev;
  }

  function _tagFromSel(sel) {
    if (!sel) return 'DIV';
    if (sel.charAt(0) === '#') return 'DIV';
    if (sel.indexOf('.') >= 0) {
      var dot = sel.indexOf('.');
      var tag = sel.slice(0, dot);
      return tag ? tag.toUpperCase() : 'DIV';
    }
    return String(sel).toUpperCase();
  }

  function _parentNodeFor(sel, handle) {
    if (sel === 'html') return null;
    if (sel === 'body' || sel === 'head') return _wrapSelector('html');
    return _wrapSelector('body');
  }

  function _liveQueryCollection(sel) {
    return new Proxy({ length: 0 }, {
      get: function(_t, prop) {
        var list = globalThis.document.querySelectorAll(sel);
        if (prop === 'length') return list.length;
        if (prop === 'item') return function(i) { return list[i] || null; };
        var idx = parseInt(prop, 10);
        if (!isNaN(idx) && String(idx) === String(prop)) return list[idx];
        return list[prop];
      }
    });
  }

  function _makeProxy(sel, handle) {
    var key = _elKey(sel, handle);
    return new Proxy({}, {
      get: function(_t, prop) {
        if (prop === '__zwHandle') return handle;
        if (prop === 'style') {
          return new Proxy({}, {
            set: function(_s, p, v) {
              if (handle) __zw_set_style_handle(handle, String(p), String(v));
              else __zw_set_style(sel, String(p), String(v));
              return true;
            },
            get: function(_s, p) {
              var raw = handle ? __zw_get_attr_handle(handle, 'style') : __zw_get_attr(sel, 'style');
              if (!raw) return '';
              var parts = raw.split(';');
              var pstr = String(p);
              for (var i = 0; i < parts.length; i++) {
                var kv = parts[i].split(':');
                if (kv[0] && kv[0].trim().toLowerCase() === pstr.toLowerCase()) {
                  return (kv[1] || '').trim();
                }
              }
              return '';
            }
          });
        }
        if (prop === 'classList') return _classListProxy(sel, handle);
        if (prop === 'className') {
          return handle ? __zw_get_attr_handle(handle, 'class') : __zw_get_attr(sel, 'class');
        }
        if (prop === 'id') {
          return handle ? __zw_get_attr_handle(handle, 'id') : __zw_get_attr(sel, 'id');
        }
        if (prop === 'textContent') {
          return handle ? __zw_get_text_handle(handle) : __zw_get_text(sel);
        }
        if (prop === 'innerHTML') {
          return handle ? __zw_get_inner_html_handle(handle) : __zw_get_inner_html(sel);
        }
        if (prop === 'parentNode' || prop === 'parentElement') {
          return _parentNodeFor(sel, handle);
        }
        if (prop === 'tagName') {
          return _tagFromSel(sel);
        }
        if (prop === 'nodeName') {
          return _tagFromSel(sel);
        }
        if (prop === 'nodeType') {
          return 1;
        }
        if (prop === 'ownerDocument') {
          return globalThis.document;
        }
        if (prop === 'getAttribute') {
          return function(name) {
            return handle ? __zw_get_attr_handle(handle, name) : __zw_get_attr(sel, name);
          };
        }
        if (prop === 'setAttribute') {
          return function(name, value) {
            if (handle) __zw_set_attr_handle(handle, name, String(value));
            else __zw_set_attr(sel, name, String(value));
          };
        }
        if (prop === 'removeAttribute') {
          return function(name) {
            if (handle) __zw_set_attr_handle(handle, name, '');
            else __zw_set_attr(sel, name, '');
          };
        }
        if (prop === 'addEventListener') {
          return function(type, fn, opts) {
            if (!_listenerStore[key]) _listenerStore[key] = {};
            if (!_listenerStore[key][type]) _listenerStore[key][type] = [];
            _listenerStore[key][type].push({ fn: fn, capture: !!(opts && opts.capture) });
          };
        }
        if (prop === 'removeEventListener') {
          return function(type, fn) {
            if (!_listenerStore[key] || !_listenerStore[key][type]) return;
            _listenerStore[key][type] = _listenerStore[key][type].filter(function(l) { return l.fn !== fn; });
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
            event.target = _makeProxy(sel, handle);
            return _dispatchToListeners(key, event);
          };
        }
        if (prop === 'click') {
          return function() {
            var ev = _makeEvent('click', { bubbles: true, cancelable: true });
            ev.target = _makeProxy(sel, handle);
            return _dispatchToListeners(key, ev);
          };
        }
        if (prop === 'appendChild') {
          return function(child) {
            if (child && child.__zwHandle) {
              if (handle) __zw_append_child_handle(handle, child.__zwHandle);
              else __zw_append_child(sel, child.__zwHandle);
            }
            return child;
          };
        }
        if (prop === 'removeChild') {
          return function(child) {
            if (child && child.__zwHandle) {
              __zw_remove_handle(child.__zwHandle);
            }
            return child;
          };
        }
        if (prop === 'remove') {
          return function() {
            if (handle) __zw_remove_handle(handle);
            else __zw_remove(sel);
          };
        }
        if (prop === 'querySelector') {
          return function(q) {
            var hit = __zw_query_match(q);
            return hit ? _wrapSelector(hit) : null;
          };
        }
        if (prop === 'querySelectorAll') {
          return function(q) {
            var all = __zw_query_all(q);
            if (!all) return [];
            return all.split('|').filter(Boolean).map(_wrapSelector);
          };
        }
        return undefined;
      },
      set: function(_t, prop, value) {
        var p = String(prop);
        if (p === 'textContent' || p === 'innerHTML') {
          if (p === 'innerHTML') {
            if (handle) __zw_set_inner_html_handle(handle, String(value));
            else __zw_set_inner_html(sel, String(value));
          } else if (handle) {
            __zw_set_text_handle(handle, String(value));
          } else {
            __zw_set_text(sel, String(value));
          }
        } else if (p === 'className') {
          if (handle) __zw_set_attr_handle(handle, 'class', String(value));
          else __zw_set_attr(sel, 'class', String(value));
        } else if (p === 'id') {
          if (handle) __zw_set_attr_handle(handle, 'id', String(value));
          else __zw_set_attr(sel, 'id', String(value));
        } else {
          if (handle) __zw_set_attr_handle(handle, p, String(value));
          else __zw_set_attr(sel, p, String(value));
        }
        return true;
      }
    });
  }

  function _wrapSelector(sel) {
    return _makeProxy(sel, null);
  }

  function _wrapHandle(handle) {
    return _makeProxy(null, handle);
  }

  globalThis.CustomEvent = function(type, options) {
    return _makeEvent(type, options);
  };

  globalThis.Event = function(type, options) {
    return _makeEvent(type, options);
  };

  globalThis.KeyboardEvent = function(type, options) {
    var ev = _makeEvent(type, options);
    if (options) {
      ev.key = options.key || '';
      ev.code = options.code || options.key || '';
    }
    return ev;
  };

  globalThis.document = {
    querySelector: function(sel) {
      var hit = __zw_query_match(sel);
      return hit ? _wrapSelector(hit) : null;
    },
    getElementById: function(id) {
      return globalThis.document.querySelector('#' + id);
    },
    querySelectorAll: function(sel) {
      var all = __zw_query_all(sel);
      if (!all) return [];
      return all.split('|').filter(Boolean).map(_wrapSelector);
    },
    getElementsByClassName: function(cls) {
      return globalThis.document.querySelectorAll('.' + cls);
    },
    getElementsByTagName: function(tag) {
      return globalThis.document.querySelectorAll(tag);
    },
    createElement: function(tag) {
      var handle = __zw_create_element(String(tag));
      return _wrapHandle(handle);
    },
    createTextNode: function(text) {
      var handle = __zw_create_text(String(text));
      return _wrapHandle(handle);
    },
    documentElement: _wrapSelector('html'),
    body: _wrapSelector('body'),
    head: _wrapSelector('head'),
    compatMode: 'CSS1Compat',
    characterSet: 'UTF-8',
    charset: 'UTF-8',
    readyState: 'complete',
    styleSheets: _emptyCollection(),
    forms: _liveQueryCollection('form'),
    images: _liveQueryCollection('img'),
    scripts: _liveQueryCollection('script'),
    links: _liveQueryCollection('a'),
    addEventListener: function(type, fn, opts) {
      _makeProxy('html', null).addEventListener(type, fn, opts);
    },
    removeEventListener: function(type, fn) {
      _makeProxy('html', null).removeEventListener(type, fn);
    },
    attachEvent: function(type, fn) {
      _attachEventForKey(_elKey('html', null), type, fn);
    },
    detachEvent: function(type, fn) {
      _detachEventForKey(_elKey('html', null), type, fn);
    }
  };
  globalThis.window = globalThis;
  globalThis.addEventListener = _globalAddEventListener;
  globalThis.removeEventListener = _globalRemoveEventListener;
  globalThis.window.attachEvent = function(type, fn) {
    _attachEventForKey(_elKey('html', null), type, fn);
  };
  globalThis.window.detachEvent = function(type, fn) {
    _detachEventForKey(_elKey('html', null), type, fn);
  };
  Object.defineProperty(globalThis.document, 'defaultView', {
    get: function() { return globalThis.window; }
  });

  globalThis.__zw_dispatch_event = function(sel, type, detail) {
    var key = _elKey(sel, null);
    var el = _wrapSelector(sel);
    var ev;
    if (detail && (detail.key || detail.code)) {
      ev = new KeyboardEvent(type, {
        bubbles: true,
        cancelable: true,
        key: detail.key || '',
        code: detail.code || detail.key || ''
      });
    } else {
      ev = _makeEvent(type, { bubbles: true, cancelable: true });
    }
    ev.target = el;
    var ok = _dispatchToListeners(key, ev);
    return ok ? 'ok' : 'prevented';
  };
})();
