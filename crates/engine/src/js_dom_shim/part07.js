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

  function _zwCacheIsOpaqueLikeType(responseType) {
    responseType = String(responseType || '');
    return responseType === 'opaque' || responseType === 'opaqueredirect';
  }

  function _zwCacheResponseFromWire(raw) {
    if (typeof raw !== 'string' || (raw.indexOf('__zwcr2:') !== 0 && raw.indexOf('__zwcr:') !== 0 && raw.indexOf('__zwfr:') !== 0)) {
      throw new TypeError('malformed Cache response');
    }
    var isCacheResponseWireV2 = raw.indexOf('__zwcr2:') === 0;
    var isCacheResponseWire = raw.indexOf('__zwcr:') === 0 || isCacheResponseWireV2;
    var rest = raw.slice(isCacheResponseWireV2 ? 8 : 7);
    var p1 = rest.indexOf('\x1f');
    var p2 = p1 >= 0 ? rest.indexOf('\x1f', p1 + 1) : -1;
    var p3 = p2 >= 0 ? rest.indexOf('\x1f', p2 + 1) : -1;
    if (p1 < 0 || p2 < 0 || p3 < 0) throw new TypeError('malformed Cache response');
    var status = parseInt(rest.slice(0, p1), 10) || 0;
    var statusText = rest.slice(p1 + 1, p2);
    var responseType = 'default';
    var responseUrl = '';
    var headersStart = p2 + 1;
    if (isCacheResponseWireV2) {
      responseType = rest.slice(p2 + 1, p3) || 'default';
      headersStart = p3 + 1;
      var p4 = rest.indexOf('\x1f', headersStart);
      if (p4 < 0) throw new TypeError('malformed Cache response');
      responseUrl = rest.slice(headersStart, p4);
      headersStart = p4 + 1;
      p3 = rest.indexOf('\x1f', headersStart);
      if (p3 < 0) throw new TypeError('malformed Cache response');
    } else if (isCacheResponseWire) {
      responseType = rest.slice(p2 + 1, p3) || 'default';
      headersStart = p3 + 1;
      p3 = rest.indexOf('\x1f', headersStart);
      if (p3 < 0) throw new TypeError('malformed Cache response');
    }
    var headers = _zwCacheParseHeadersWire(rest.slice(headersStart, p3));
    var body = rest.slice(p3 + 1);
    var bodyArg = body.indexOf('__zw_bytes:') === 0 ? _zwCacheDecodeBytesPrefix(body) : body;
    var response = new Response(bodyArg, { status: status, statusText: statusText, headers: headers });
    response.type = String(responseType || 'default');
    response.url = responseUrl;
    if (_zwCacheIsOpaqueLikeType(response.type)) {
      response._zwOpaqueStatus = status;
      response._zwOpaqueStatusText = statusText;
      response._zwOpaqueHeaders = response.headers;
      response._zwOpaqueBodyText = response._bodyText;
      response._zwOpaqueBodyBytes = response._bodyBytes;
      response.status = 0;
      response.statusText = '';
      response.ok = false;
      response.headers = new Headers();
      response.headers._guard = 'response';
      response._bodyText = '';
      response._bodyBytes = null;
      response._bodyNull = true;
    }
    return response;
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

  function _zwCacheRequestHeaderKey(request) {
    return _zwCacheRequestCacheKey(request) + ' ' + _zwCacheHeadersToWire(request && request.headers);
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

  function _zwCacheRequestHeader(request, name) {
    if (!request || !request.headers || typeof request.headers.get !== 'function') return null;
    var value = request.headers.get(name);
    return value == null ? null : String(value);
  }

  function _zwCacheRequestsMatchByResponseVary(cachedRequest, queryRequest, response) {
    if (_zwCacheRequestCacheKey(cachedRequest) !== _zwCacheRequestCacheKey(queryRequest)) return false;
    if (!response || String(response.type || 'default') === 'opaque') return true;
    var vary = response.headers && typeof response.headers.get === 'function' ? response.headers.get('vary') : null;
    if (vary == null) return true;
    var fields = String(vary).split(',');
    var hasField = false;
    for (var i = 0; i < fields.length; i++) {
      var field = fields[i].trim();
      if (!field) continue;
      if (field === '*') return false;
      hasField = true;
      if (_zwCacheRequestHeader(cachedRequest, field) !== _zwCacheRequestHeader(queryRequest, field)) {
        return false;
      }
    }
    return hasField || String(vary).trim() === '';
  }

  function _zwCacheAddAllHasDuplicate(entries) {
    // https://w3c.github.io/ServiceWorker/#batch-cache-operations
    for (var i = 0; i < entries.length; i++) {
      for (var j = 0; j < i; j++) {
        if (_zwCacheRequestsMatchByResponseVary(entries[j].request, entries[i].request, entries[i].response) ||
            _zwCacheRequestsMatchByResponseVary(entries[i].request, entries[j].request, entries[j].response)) {
          return true;
        }
      }
    }
    return false;
  }

  function _zwCacheMarkResponseBodyUsed(response) {
    if (response && !response._bodyNull && typeof response._bodyUsed !== 'undefined') response._bodyUsed = true;
  }

  function _zwCacheValidatePut(request, response) {
    // https://w3c.github.io/ServiceWorker/#cache-put
    if (String(request.method || 'GET').toUpperCase() !== 'GET') {
      throw new TypeError('Cache.put request method must be GET');
    }
    if (!_zwCacheRequestIsHttp(request)) {
      throw new TypeError('Cache.put request URL must be an HTTP(S) URL');
    }
    if (response._bodyUsed && !response._bodyNull) {
      throw new TypeError('Cache.put cannot store a used response body');
    }
    if ((response.status | 0) === 206) {
      throw new TypeError('Cache.put cannot store a 206 Partial Content response');
    }
    if (String(response.type || 'default') !== 'opaque' && _zwCacheResponseVaryHasStar(response)) {
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
    var responseType = String(response.type || 'default');
    var isOpaque = _zwCacheIsOpaqueLikeType(responseType);
    var status = isOpaque && response._zwOpaqueStatus !== undefined ? response._zwOpaqueStatus : response.status;
    var statusText = isOpaque && response._zwOpaqueStatusText !== undefined ? response._zwOpaqueStatusText : response.statusText;
    var headers = isOpaque && response._zwOpaqueHeaders ? response._zwOpaqueHeaders : response.headers;
    var bodyBytes = isOpaque && response._zwOpaqueBodyBytes !== undefined ? response._zwOpaqueBodyBytes : response._bodyBytes;
    var bodyText = isOpaque && response._zwOpaqueBodyText !== undefined ? response._zwOpaqueBodyText : response._bodyText;
    var bodyIsBytes = bodyBytes != null;
    return {
      url: String(response.url || ''),
      status: status | 0,
      statusText: String(statusText || ''),
      type: responseType,
      headers: _zwCacheHeadersToWire(headers),
      body: bodyIsBytes ? _zwCacheEncodeBytesPrefix(bodyBytes) : String(bodyText || ''),
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
        if (!(response instanceof Response)) throw new TypeError('Cache.put requires a Response');
        var cacheResponse = response;
        _zwCacheValidatePut(cacheRequest, cacheResponse);
        var hostRequest = {
          op: 'put',
          request: _zwCacheRequestWire(cacheRequest),
          response: _zwCacheResponseWire(cacheResponse)
        };
        _zwCacheSetNameWire(hostRequest, 'cache_name', cache._name);
        _zwCacheSetIdWire(hostRequest, cache);
        _zwCacheHostCall(hostRequest);
        if (!cacheResponse._bodyNull && cacheResponse.body && typeof cacheResponse.body.getReader === 'function') {
          cacheResponse.body.getReader();
        }
        _zwCacheMarkResponseBodyUsed(cacheResponse);
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
        if (request === undefined) throw new TypeError('Cache.addAll request is required');
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
        var key = _zwCacheRequestHeaderKey(cacheRequests[i]);
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
        if (_zwCacheAddAllHasDuplicate(entries)) {
          throw new (globalThis.DOMException || Error)('Cache.addAll duplicate requests', 'InvalidStateError');
        }
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

  function _zwCacheBucketDeletedError() {
    try {
      return new DOMException('Storage bucket is deleted', 'UnknownError');
    } catch (_e) {
      var error = new Error('Storage bucket is deleted');
      error.name = 'UnknownError';
      return error;
    }
  }

  function CacheStorage(namePrefix, liveCheck, keyFromHostName) {
    if (!(this instanceof CacheStorage)) return new CacheStorage();
    this._namePrefix = typeof namePrefix === 'function' ? namePrefix : null;
    this._liveCheck = typeof liveCheck === 'function' ? liveCheck : null;
    this._keyFromHostName = typeof keyFromHostName === 'function' ? keyFromHostName : null;
  }

  CacheStorage.prototype._assertLive = function () {
    if (this._liveCheck && !this._liveCheck()) throw _zwCacheBucketDeletedError();
  };

  CacheStorage.prototype._nameForHost = function (name) {
    return this._namePrefix ? this._namePrefix(String(name)) : name;
  };

  CacheStorage.prototype.open = function (name) {
    var storage = this;
    var hasName = arguments.length >= 1;
    return new Promise(function (resolve, reject) {
      try {
        storage._assertLive();
        if (!hasName) throw new TypeError('CacheStorage.open requires a name');
        var hostRequest = { op: 'open' };
        _zwCacheSetNameWire(hostRequest, 'name', storage._nameForHost(name));
        var result = _zwCacheHostCall(hostRequest);
        resolve(new Cache(_zwCacheNameFromResult(result), result.cache_id));
      } catch (error) {
        reject(error);
      }
    });
  };

  CacheStorage.prototype.has = function (name) {
    var storage = this;
    var hasName = arguments.length >= 1;
    return new Promise(function (resolve, reject) {
      try {
        storage._assertLive();
        if (!hasName) throw new TypeError('CacheStorage.has requires a name');
        var hostRequest = { op: 'has' };
        _zwCacheSetNameWire(hostRequest, 'name', storage._nameForHost(name));
        var result = _zwCacheHostCall(hostRequest);
        resolve(!!result.has);
      } catch (error) {
        reject(error);
      }
    });
  };

  CacheStorage.prototype.delete = function (name) {
    var storage = this;
    var hasName = arguments.length >= 1;
    return new Promise(function (resolve, reject) {
      try {
        storage._assertLive();
        if (!hasName) throw new TypeError('CacheStorage.delete requires a name');
        var hostRequest = { op: 'delete' };
        _zwCacheSetNameWire(hostRequest, 'name', storage._nameForHost(name));
        var result = _zwCacheHostCall(hostRequest);
        resolve(!!result.deleted);
      } catch (error) {
        reject(error);
      }
    });
  };

  CacheStorage.prototype.keys = function () {
    var storage = this;
    return new Promise(function (resolve, reject) {
      try {
        storage._assertLive();
        var result = _zwCacheHostCall({ op: 'keys' });
        var keys;
        if (Array.isArray(result.keys_units)) {
          keys = result.keys_units.map(_zwCacheDomStringFromWire);
        } else {
          keys = Array.isArray(result.keys) ? result.keys.slice() : [];
        }
        if (storage._keyFromHostName) {
          keys = keys.map(storage._keyFromHostName).filter(function (name) { return name !== null; });
        }
        resolve(keys);
      } catch (error) {
        reject(error);
      }
    });
  };

  CacheStorage.prototype.match = function (request, options) {
    var storage = this;
    var hasRequest = arguments.length >= 1;
    return new Promise(function (resolve, reject) {
      try {
        storage._assertLive();
        if (!hasRequest) throw new TypeError('CacheStorage.match requires a request');
        var hostRequest = {
          op: 'match',
          request: _zwCacheRequestWire(request),
          options: _zwCacheQueryOptionsWire(options)
        };
        if (options !== undefined && options !== null && Object(options).cacheName !== undefined) {
          _zwCacheSetNameWire(hostRequest, 'cache_name', storage._nameForHost(Object(options).cacheName));
        }
        var result = _zwCacheHostCall(hostRequest);
        resolve(result.response === null ? undefined : _zwCacheResponseFromWire(result.response));
      } catch (error) {
        reject(error);
      }
    });
  };

  function _zwBucketCachePrefix(bucketName) {
    return '__zw_storage_bucket__' + _zwCacheDomStringWire(bucketName) + ':';
  }

  function StorageBucket(name, owner) {
    this.name = String(name);
    this._owner = owner;
    this._deleted = false;
    var prefix = _zwBucketCachePrefix(this.name);
    var bucket = this;
    this.caches = new CacheStorage(
      function (cacheName) { return prefix + cacheName; },
      function () { return !bucket._deleted && owner._bucketExists(bucket.name); },
      function (hostName) {
        hostName = String(hostName);
        return hostName.indexOf(prefix) === 0 ? hostName.slice(prefix.length) : null;
      }
    );
  }

  function StorageBucketManager() {
    this._buckets = {};
    this._order = [];
  }

  StorageBucketManager.prototype._bucketExists = function (name) {
    return Object.prototype.hasOwnProperty.call(this._buckets, String(name));
  };

  StorageBucketManager.prototype.open = function (name) {
    var manager = this;
    return new Promise(function (resolve) {
      name = String(name);
      if (!manager._bucketExists(name)) {
        manager._buckets[name] = new StorageBucket(name, manager);
        manager._order.push(name);
      }
      resolve(manager._buckets[name]);
    });
  };

  StorageBucketManager.prototype.keys = function () {
    var manager = this;
    return Promise.resolve(manager._order.filter(function (name) {
      return manager._bucketExists(name);
    }));
  };

  StorageBucketManager.prototype.delete = function (name) {
    var manager = this;
    return new Promise(function (resolve, reject) {
      try {
        name = String(name);
        if (!manager._bucketExists(name)) {
          resolve(false);
          return;
        }
        var prefix = _zwBucketCachePrefix(name);
        var result = _zwCacheHostCall({ op: 'keys' });
        var keys = Array.isArray(result.keys_units)
          ? result.keys_units.map(_zwCacheDomStringFromWire)
          : (Array.isArray(result.keys) ? result.keys.slice() : []);
        keys.forEach(function (cacheName) {
          cacheName = String(cacheName);
          if (cacheName.indexOf(prefix) === 0) {
            var request = { op: 'delete' };
            _zwCacheSetNameWire(request, 'name', cacheName);
            _zwCacheHostCall(request);
          }
        });
        manager._buckets[name]._deleted = true;
        delete manager._buckets[name];
        manager._order = manager._order.filter(function (bucketName) { return bucketName !== name; });
        resolve(true);
      } catch (error) {
        reject(error);
      }
    });
  };

  globalThis.Cache = globalThis.Cache || Cache;
  globalThis.CacheStorage = globalThis.CacheStorage || CacheStorage;
  globalThis.StorageBucket = globalThis.StorageBucket || StorageBucket;
  globalThis.StorageBucketManager = globalThis.StorageBucketManager || StorageBucketManager;
  globalThis.caches = globalThis.caches || new globalThis.CacheStorage();
  if (globalThis.navigator && !globalThis.navigator.storageBuckets) {
    globalThis.navigator.storageBuckets = new globalThis.StorageBucketManager();
  }
  // R376（js-dom M4/DC-3）：**WebIDL 接口全局属性形态归一**——spec 全局接口对象是
  // ① 不可枚举（`for (var p in window)` 不含接口名，WPT dom/interface-objects
  // "Interface objects properties should not be Enumerable"——旧经普通赋值/defineProperty
  // 默认 enumerable 的构造器被 for-in 枚举）；② 可配置可删除（`delete window[iface]`
  // 返 true 且删除生效——同文件 "Should be able to delete" 族）；③ 缺失接口补位
  //（NodeIterator/TreeWalker 构造器——createTreeWalker/createNodeIterator 的产物
  // 不经构造器创建，但接口对象本身须存在于 window）。
  // https://webidl.spec.whatwg.org/#es-interfaces（[LegacyWindowAlias]/属性特性
  // writable:true, enumerable:false, configurable:true）
  (function () {
    var _r376ifaces = [
      'Event', 'CustomEvent', 'EventTarget', 'AbortController', 'AbortSignal',
      'Node', 'Document', 'DOMImplementation', 'DocumentFragment',
      'ProcessingInstruction', 'DocumentType', 'Element', 'Attr', 'CharacterData',
      'Text', 'Comment', 'NodeIterator', 'TreeWalker', 'NodeFilter', 'NodeList',
      'HTMLCollection', 'DOMTokenList',
    ];
    // 缺失接口补位（简单占位构造器——spec 语义面由既有工厂承载）。
    ['NodeIterator', 'TreeWalker', 'DOMTokenList'].forEach(function (name376) {
      if (!globalThis[name376]) {
        try { globalThis[name376] = new Function('return function ' + name376 + '() {}')(); } catch (_e376m) {}
      }
    });
    for (var i376 = 0; i376 < _r376ifaces.length; i376++) {
      var name376b = _r376ifaces[i376];
      var desc376 = Object.getOwnPropertyDescriptor(globalThis, name376b);
      if (!desc376) continue;
      if (desc376.enumerable && desc376.configurable) {
        try {
          Object.defineProperty(globalThis, name376b, {
            value: desc376.value, writable: desc376.writable,
            enumerable: false, configurable: true,
          });
        } catch (_e376d) {}
      }
    }
  })();

})();
