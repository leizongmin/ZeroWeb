(function() {
  // Cache API page global (initial hosted surface).
  // https://w3c.github.io/ServiceWorker/#cache-interface
  // https://w3c.github.io/ServiceWorker/#cache-storage-interface
  function _zwCacheHostCall(request) {
    if (typeof globalThis.__zw_cache_storage !== 'function') {
      throw new TypeError('CacheStorage host bridge is unavailable');
    }
    var wire = String(globalThis.__zw_cache_storage(JSON.stringify(request)));
    var okPrefix = '__zw_cache_ok:';
    var errorPrefix = '__zw_cache_error:';
    if (wire.indexOf(okPrefix) === 0) {
      return JSON.parse(wire.slice(okPrefix.length));
    }
    if (wire.indexOf(errorPrefix) === 0) {
      throw new TypeError(wire.slice(errorPrefix.length));
    }
    throw new TypeError('malformed CacheStorage response');
  }

  function _zwCacheParseHeadersWire(wire) {
    var headers = {};
    if (!wire) return headers;
    var parts = String(wire).split('\x1e');
    for (var i = 0; i + 1 < parts.length; i += 2) {
      headers[parts[i]] = parts[i + 1];
    }
    return headers;
  }

  function _zwCacheDecodeBytesPrefix(wire) {
    var rest = String(wire).slice(11);
    if (!rest) return new Uint8Array(0);
    var parts = rest.split(',');
    var out = new Uint8Array(parts.length);
    for (var i = 0; i < parts.length; i++) out[i] = parseInt(parts[i], 10) & 0xFF;
    return out;
  }

  function _zwCacheResponseFromWire(raw) {
    if (typeof raw !== 'string' || raw.indexOf('__zwfr:') !== 0) {
      throw new TypeError('malformed Cache response');
    }
    var rest = raw.slice(7);
    var p1 = rest.indexOf('\x1f');
    var p2 = p1 >= 0 ? rest.indexOf('\x1f', p1 + 1) : -1;
    var p3 = p2 >= 0 ? rest.indexOf('\x1f', p2 + 1) : -1;
    if (p1 < 0 || p2 < 0 || p3 < 0) throw new TypeError('malformed Cache response');
    var status = parseInt(rest.slice(0, p1), 10) || 0;
    var statusText = rest.slice(p1 + 1, p2);
    var headers = _zwCacheParseHeadersWire(rest.slice(p2 + 1, p3));
    var body = rest.slice(p3 + 1);
    var bodyArg = body.indexOf('__zw_bytes:') === 0 ? _zwCacheDecodeBytesPrefix(body) : body;
    return new Response(bodyArg, { status: status, statusText: statusText, headers: headers });
  }

  function _zwCacheHeadersToWire(src) {
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
      out += (out ? '\x1e' : '') + pairs[j][0] + '\x1e' + pairs[j][1];
    }
    return out;
  }

  function _zwCacheEncodeBytesPrefix(bytes) {
    var s = '__zw_bytes:';
    for (var i = 0; i < bytes.length; i++) {
      if (i > 0) s += ',';
      s += (bytes[i] & 0xFF);
    }
    return s;
  }

  function _zwCacheRequestWire(input) {
    var request = input instanceof Request ? input : new Request(input);
    var url = String(request.url || '');
    if (typeof URL === 'function') {
      try {
        url = new URL(url, globalThis.location && globalThis.location.href || 'about:blank').href;
      } catch (_e) {}
    }
    return {
      url: url,
      method: String(request.method || 'GET').toUpperCase()
    };
  }

  function _zwCacheResponseWire(response) {
    if (!(response instanceof Response)) {
      response = new Response(response);
    }
    var bodyIsBytes = response._bodyBytes != null;
    return {
      status: response.status | 0,
      statusText: String(response.statusText || ''),
      headers: _zwCacheHeadersToWire(response.headers),
      body: bodyIsBytes ? _zwCacheEncodeBytesPrefix(response._bodyBytes) : String(response._bodyText || ''),
      bodyIsBytes: bodyIsBytes
    };
  }

  function Cache(name) {
    if (!(this instanceof Cache)) return new Cache(name);
    this._name = String(name);
  }

  Cache.prototype.match = function (request) {
    var cache = this;
    return new Promise(function (resolve, reject) {
      try {
        var result = _zwCacheHostCall({
          op: 'match',
          cache_name: cache._name,
          request: _zwCacheRequestWire(request)
        });
        resolve(result.response === null ? undefined : _zwCacheResponseFromWire(result.response));
      } catch (error) {
        reject(error);
      }
    });
  };

  Cache.prototype.put = function (request, response) {
    var cache = this;
    return new Promise(function (resolve, reject) {
      try {
        _zwCacheHostCall({
          op: 'put',
          cache_name: cache._name,
          request: _zwCacheRequestWire(request),
          response: _zwCacheResponseWire(response)
        });
        resolve(undefined);
      } catch (error) {
        reject(error);
      }
    });
  };

  Cache.prototype.delete = function (request) {
    var cache = this;
    return new Promise(function (resolve, reject) {
      try {
        var result = _zwCacheHostCall({
          op: 'delete',
          name: cache._name,
          request: _zwCacheRequestWire(request)
        });
        resolve(!!result.deleted);
      } catch (error) {
        reject(error);
      }
    });
  };

  function CacheStorage() {
    if (!(this instanceof CacheStorage)) return new CacheStorage();
  }

  CacheStorage.prototype.open = function (name) {
    return new Promise(function (resolve, reject) {
      try {
        var result = _zwCacheHostCall({ op: 'open', name: String(name) });
        resolve(new Cache(result.name));
      } catch (error) {
        reject(error);
      }
    });
  };

  CacheStorage.prototype.has = function (name) {
    return new Promise(function (resolve, reject) {
      try {
        var result = _zwCacheHostCall({ op: 'has', name: String(name) });
        resolve(!!result.has);
      } catch (error) {
        reject(error);
      }
    });
  };

  CacheStorage.prototype.delete = function (name) {
    return new Promise(function (resolve, reject) {
      try {
        var result = _zwCacheHostCall({ op: 'delete', name: String(name) });
        resolve(!!result.deleted);
      } catch (error) {
        reject(error);
      }
    });
  };

  CacheStorage.prototype.keys = function () {
    return new Promise(function (resolve, reject) {
      try {
        var result = _zwCacheHostCall({ op: 'keys' });
        resolve(Array.isArray(result.keys) ? result.keys.slice() : []);
      } catch (error) {
        reject(error);
      }
    });
  };

  CacheStorage.prototype.match = function (request) {
    return new Promise(function (resolve, reject) {
      try {
        var result = _zwCacheHostCall({ op: 'match', request: _zwCacheRequestWire(request) });
        resolve(result.response === null ? undefined : _zwCacheResponseFromWire(result.response));
      } catch (error) {
        reject(error);
      }
    });
  };

  globalThis.Cache = globalThis.Cache || Cache;
  globalThis.CacheStorage = globalThis.CacheStorage || CacheStorage;
  globalThis.caches = globalThis.caches || new globalThis.CacheStorage();
})();
