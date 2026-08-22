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
      method: String(request.method || 'GET').toUpperCase(),
      headers: _zwCacheHeadersToWire(request.headers)
    };
  }

  function _zwCacheRequestCacheKey(request) {
    var url = String(request.url || '');
    if (typeof URL === 'function') {
      try {
        var parsed = new URL(url);
        parsed.hash = '';
        url = parsed.href;
      } catch (_e) {
        url = url.split('#')[0];
      }
    } else {
      url = url.split('#')[0];
    }
    return String(request.method || 'GET').toUpperCase() + ' ' + url;
  }

  function _zwCacheRequestIsHttp(request) {
    var url = String(request.url || '');
    return /^https?:/i.test(url);
  }

  function _zwCacheResponseVaryHasStar(response) {
    if (!response || !response.headers || typeof response.headers.get !== 'function') return false;
    var vary = response.headers.get('vary');
    if (vary == null) return false;
    var fields = String(vary).split(',');
    for (var i = 0; i < fields.length; i++) {
      if (fields[i].trim().toLowerCase() === '*') return true;
    }
    return false;
  }

  function _zwCacheValidatePut(request, response) {
    // https://w3c.github.io/ServiceWorker/#cache-put
    if (String(request.method || 'GET').toUpperCase() !== 'GET') {
      throw new TypeError('Cache.put request method must be GET');
    }
    if (!_zwCacheRequestIsHttp(request)) {
      throw new TypeError('Cache.put request URL must be an HTTP(S) URL');
    }
    if ((response.status | 0) === 206) {
      throw new TypeError('Cache.put cannot store a 206 Partial Content response');
    }
    if (_zwCacheResponseVaryHasStar(response)) {
      throw new TypeError('Cache.put cannot store a response with Vary: *');
    }
  }

  function _zwCacheRequestFromWire(raw) {
    if (!raw || typeof raw.url !== 'string') throw new TypeError('malformed Cache request');
    return new Request(raw.url, {
      method: String(raw.method || 'GET').toUpperCase(),
      headers: _zwCacheParseHeadersWire(raw.headers)
    });
  }

  function _zwCacheDomStringWire(value) {
    var s = String(value);
    var out = '';
    for (var i = 0; i < s.length; i++) {
      var unit = s.charCodeAt(i).toString(16);
      while (unit.length < 4) unit = '0' + unit;
      out += unit;
    }
    return out;
  }

  function _zwCacheDomStringFromWire(units) {
    units = String(units || '');
    if (units.length % 4 !== 0) throw new TypeError('malformed CacheStorage name');
    var out = '';
    for (var i = 0; i < units.length; i += 4) {
      var unit = parseInt(units.slice(i, i + 4), 16);
      if (!isFinite(unit)) throw new TypeError('malformed CacheStorage name');
      out += String.fromCharCode(unit);
    }
    return out;
  }

  function _zwCacheSetNameWire(target, field, value) {
    var s = String(value);
    var hasSurrogate = false;
    for (var i = 0; i < s.length; i++) {
      var unit = s.charCodeAt(i);
      if (unit >= 0xD800 && unit <= 0xDFFF) {
        hasSurrogate = true;
        break;
      }
    }
    if (!hasSurrogate) target[field] = s;
    target[field + '_units'] = _zwCacheDomStringWire(s);
  }

  function _zwCacheNameFromResult(result) {
    if (result && typeof result.name_units === 'string') {
      return _zwCacheDomStringFromWire(result.name_units);
    }
    return String(result && result.name || '');
  }

  function _zwCacheQueryOptionsWire(options) {
    options = options === undefined || options === null ? {} : Object(options);
    return {
      ignoreSearch: !!options.ignoreSearch,
      ignoreMethod: !!options.ignoreMethod,
      ignoreVary: !!options.ignoreVary
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

  function Cache(name, cacheId) {
    if (!(this instanceof Cache)) return new Cache(name);
    this._name = String(name);
    this._cacheId = cacheId === undefined || cacheId === null ? null : Number(cacheId);
  }

  function _zwCacheSetIdWire(target, cache) {
    if (cache && cache._cacheId !== null && isFinite(cache._cacheId)) {
      target.cache_id = cache._cacheId;
    }
  }

  Cache.prototype.match = function (request, options) {
    var cache = this;
    var hasRequest = arguments.length >= 1;
    return new Promise(function (resolve, reject) {
      try {
        if (!hasRequest) throw new TypeError('Cache.match requires a request');
        var hostRequest = {
          op: 'match',
          request: _zwCacheRequestWire(request),
          options: _zwCacheQueryOptionsWire(options)
        };
        _zwCacheSetNameWire(hostRequest, 'cache_name', cache._name);
        _zwCacheSetIdWire(hostRequest, cache);
        var result = _zwCacheHostCall(hostRequest);
        resolve(result.response === null ? undefined : _zwCacheResponseFromWire(result.response));
      } catch (error) {
        reject(error);
      }
    });
  };

  Cache.prototype.matchAll = function (request, options) {
    var cache = this;
    return new Promise(function (resolve, reject) {
      try {
        var hostRequest = {
          op: 'match_all',
          options: _zwCacheQueryOptionsWire(options)
        };
        _zwCacheSetNameWire(hostRequest, 'cache_name', cache._name);
        _zwCacheSetIdWire(hostRequest, cache);
        if (request !== undefined) hostRequest.request = _zwCacheRequestWire(request);
        var result = _zwCacheHostCall(hostRequest);
        var responses = Array.isArray(result.responses) ? result.responses : [];
        resolve(responses.map(_zwCacheResponseFromWire));
      } catch (error) {
        reject(error);
      }
    });
  };

  Cache.prototype.put = function (request, response) {
    var cache = this;
    var hasArguments = arguments.length >= 2;
    return new Promise(function (resolve, reject) {
      try {
        if (!hasArguments) throw new TypeError('Cache.put requires a request and response');
        var cacheRequest = request instanceof Request ? request : new Request(request);
        var cacheResponse = response instanceof Response ? response : new Response(response);
        _zwCacheValidatePut(cacheRequest, cacheResponse);
        var hostRequest = {
          op: 'put',
          request: _zwCacheRequestWire(cacheRequest),
          response: _zwCacheResponseWire(cacheResponse)
        };
        _zwCacheSetNameWire(hostRequest, 'cache_name', cache._name);
        _zwCacheSetIdWire(hostRequest, cache);
        _zwCacheHostCall(hostRequest);
        resolve(undefined);
      } catch (error) {
        reject(error);
      }
    });
  };

  Cache.prototype.add = function (request) {
    var cache = this;
    try {
      if (arguments.length < 1) throw new TypeError('Cache.add requires a request');
      var cacheRequest = request instanceof Request ? request : new Request(request);
      if (cacheRequest.method !== 'GET') {
        return Promise.reject(new TypeError('Cache.add only supports GET requests'));
      }
      if (!_zwCacheRequestIsHttp(cacheRequest)) {
        return Promise.reject(new TypeError('Cache.add request URL must be an HTTP(S) URL'));
      }
      return fetch(cacheRequest.clone()).then(function (response) {
        if (!response || !response.ok) {
          throw new TypeError('Cache.add fetch response is not ok');
        }
        _zwCacheValidatePut(cacheRequest, response);
        return cache.put(cacheRequest, response);
      });
    } catch (error) {
      return Promise.reject(error);
    }
  };

  Cache.prototype.addAll = function (requests) {
    var cache = this;
    try {
      if (arguments.length < 1) throw new TypeError('Cache.addAll requires requests');
      var list = Array.prototype.slice.call(requests);
      var cacheRequests = list.map(function (request) {
        var cacheRequest = request instanceof Request ? request : new Request(request);
        if (cacheRequest.method !== 'GET') {
          throw new TypeError('Cache.addAll only supports GET requests');
        }
        if (!_zwCacheRequestIsHttp(cacheRequest)) {
          throw new TypeError('Cache.addAll request URL must be an HTTP(S) URL');
        }
        return cacheRequest;
      });
      var seen = {};
      for (var i = 0; i < cacheRequests.length; i++) {
        var key = _zwCacheRequestCacheKey(cacheRequests[i]);
        if (seen[key]) {
          throw new (globalThis.DOMException || Error)('Cache.addAll duplicate requests', 'InvalidStateError');
        }
        seen[key] = true;
      }
      return Promise.all(cacheRequests.map(function (cacheRequest) {
        return fetch(cacheRequest.clone()).then(function (response) {
          if (!response || !response.ok) {
            throw new TypeError('Cache.addAll fetch response is not ok');
          }
          _zwCacheValidatePut(cacheRequest, response);
          return { request: cacheRequest, response: response };
        });
      })).then(function (entries) {
        var chain = Promise.resolve();
        entries.forEach(function (entry) {
          chain = chain.then(function () { return cache.put(entry.request, entry.response); });
        });
        return chain;
      }).then(function () {
        return undefined;
      });
    } catch (error) {
      return Promise.reject(error);
    }
  };

  Cache.prototype.delete = function (request, options) {
    var cache = this;
    var hasRequest = arguments.length >= 1;
    return new Promise(function (resolve, reject) {
      try {
        if (!hasRequest) throw new TypeError('Cache.delete requires a request');
        var hostRequest = {
          op: 'delete',
          request: _zwCacheRequestWire(request),
          options: _zwCacheQueryOptionsWire(options)
        };
        _zwCacheSetNameWire(hostRequest, 'name', cache._name);
        _zwCacheSetIdWire(hostRequest, cache);
        var result = _zwCacheHostCall(hostRequest);
        resolve(!!result.deleted);
      } catch (error) {
        reject(error);
      }
    });
  };

  Cache.prototype.keys = function (request, options) {
    var cache = this;
    return new Promise(function (resolve, reject) {
      try {
        var hostRequest = {
          op: 'cache_keys',
          options: _zwCacheQueryOptionsWire(options)
        };
        _zwCacheSetNameWire(hostRequest, 'cache_name', cache._name);
        _zwCacheSetIdWire(hostRequest, cache);
        if (request !== undefined) hostRequest.request = _zwCacheRequestWire(request);
        var result = _zwCacheHostCall(hostRequest);
        var requests = Array.isArray(result.requests) ? result.requests : [];
        resolve(requests.map(_zwCacheRequestFromWire));
      } catch (error) {
        reject(error);
      }
    });
  };

  function CacheStorage() {
    if (!(this instanceof CacheStorage)) return new CacheStorage();
  }

  CacheStorage.prototype.open = function (name) {
    var hasName = arguments.length >= 1;
    return new Promise(function (resolve, reject) {
      try {
        if (!hasName) throw new TypeError('CacheStorage.open requires a name');
        var hostRequest = { op: 'open' };
        _zwCacheSetNameWire(hostRequest, 'name', name);
        var result = _zwCacheHostCall(hostRequest);
        resolve(new Cache(_zwCacheNameFromResult(result), result.cache_id));
      } catch (error) {
        reject(error);
      }
    });
  };

  CacheStorage.prototype.has = function (name) {
    var hasName = arguments.length >= 1;
    return new Promise(function (resolve, reject) {
      try {
        if (!hasName) throw new TypeError('CacheStorage.has requires a name');
        var hostRequest = { op: 'has' };
        _zwCacheSetNameWire(hostRequest, 'name', name);
        var result = _zwCacheHostCall(hostRequest);
        resolve(!!result.has);
      } catch (error) {
        reject(error);
      }
    });
  };

  CacheStorage.prototype.delete = function (name) {
    var hasName = arguments.length >= 1;
    return new Promise(function (resolve, reject) {
      try {
        if (!hasName) throw new TypeError('CacheStorage.delete requires a name');
        var hostRequest = { op: 'delete' };
        _zwCacheSetNameWire(hostRequest, 'name', name);
        var result = _zwCacheHostCall(hostRequest);
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
        if (Array.isArray(result.keys_units)) {
          resolve(result.keys_units.map(_zwCacheDomStringFromWire));
        } else {
          resolve(Array.isArray(result.keys) ? result.keys.slice() : []);
        }
      } catch (error) {
        reject(error);
      }
    });
  };

  CacheStorage.prototype.match = function (request, options) {
    var hasRequest = arguments.length >= 1;
    return new Promise(function (resolve, reject) {
      try {
        if (!hasRequest) throw new TypeError('CacheStorage.match requires a request');
        var hostRequest = {
          op: 'match',
          request: _zwCacheRequestWire(request),
          options: _zwCacheQueryOptionsWire(options)
        };
        if (options !== undefined && options !== null && Object(options).cacheName !== undefined) {
          _zwCacheSetNameWire(hostRequest, 'cache_name', Object(options).cacheName);
        }
        var result = _zwCacheHostCall(hostRequest);
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
