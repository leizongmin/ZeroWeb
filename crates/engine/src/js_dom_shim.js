(function() {
  var _listenerStore = {};

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
    addEventListener: function(type, fn, opts) {
      _makeProxy('html', null).addEventListener(type, fn, opts);
    },
    removeEventListener: function(type, fn) {
      _makeProxy('html', null).removeEventListener(type, fn);
    }
  };
  globalThis.window = globalThis;

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
