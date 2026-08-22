//! Typed Service Worker script runtime.

use crate::threaded_runtime::ThreadedRuntimeCore;
use crate::{
    ModuleRegistry, Sandbox, SandboxConfig, ScriptError, compile_module_script, expose_classic_script_lexicals,
    extract_dynamic_import_specifiers, extract_static_module_import_specifiers,
};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

const ENGINE_INIT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
const MAX_HEAP_BYTES: usize = 64 * 1024 * 1024;
const DEFAULT_SCRIPT_TIMEOUT_MS: u64 = 5_000;
const MAX_IMPORT_SCRIPTS_PER_CALL: usize = 64;
const MAX_IMPORT_SCRIPT_URL_BYTES: usize = 64 * 1024;
const MAX_IMPORTED_SCRIPT_BYTES: usize = 16 * 1024 * 1024;
const MAX_MESSAGE_PORTS: usize = 16;
const MAX_CLIENT_ID_BYTES: usize = 64 * 1024;
const MAX_SERVICE_WORKER_CLIENTS: usize = 128;
const MAX_FETCH_METHOD_BYTES: usize = 128;
const MAX_FETCH_HEADERS: usize = 128;
const MAX_FETCH_HEADER_BYTES: usize = 64 * 1024;
const MAX_FETCH_STATUS_TEXT_BYTES: usize = 1024;
const MAX_FETCH_BODY_BYTES: usize = 16 * 1024 * 1024;
const MAX_CACHE_NAME_BYTES: usize = 1024;
const MAX_CACHE_RESULTS: usize = 1024;

enum ServiceWorkerCommand {
    Evaluate {
        script: String,
        script_url: String,
        is_module: bool,
    },
    DispatchLifecycle {
        event_id: u64,
        phase: ServiceWorkerLifecyclePhase,
    },
    DispatchMessage {
        event_id: u64,
        data_json: String,
        client_id: String,
        client_url: String,
        ports: ServiceWorkerMessagePorts,
    },
    DispatchFetch {
        event_id: u64,
        request: ServiceWorkerFetchRequest,
    },
    Shutdown,
}

enum ServiceWorkerImportResponse {
    Completed { request_id: u64, sources: Vec<String> },
    Failed { request_id: u64, message: String },
    Shutdown,
}

enum ServiceWorkerUpdateResponse {
    Completed {
        request_id: u64,
    },
    Failed {
        request_id: u64,
        exception_name: String,
        message: String,
    },
    Shutdown,
}

enum ServiceWorkerClientsResponse {
    Completed {
        request_id: u64,
        clients: Vec<ServiceWorkerClientInfo>,
    },
    Failed {
        request_id: u64,
        message: String,
    },
    Shutdown,
}

enum ServiceWorkerCacheStorageResponse {
    Completed {
        request_id: u64,
        response: ServiceWorkerCacheStorageResult,
    },
    Failed {
        request_id: u64,
        message: String,
    },
    Shutdown,
}

enum ServiceWorkerFetchHostResponse {
    Completed {
        request_id: u64,
        response: ServiceWorkerFetchResponse,
    },
    Failed {
        request_id: u64,
        message: String,
    },
    Shutdown,
}

struct PendingLifecycle {
    event_id: u64,
    phase: ServiceWorkerLifecyclePhase,
    deadline: std::time::Instant,
}

struct PendingFetch {
    event_id: u64,
    request_url: String,
    deadline: std::time::Instant,
}

const CACHE_NAME_DOMSTRING_PREFIX: &str = "__zw_domstring16:";

const SERVICE_WORKER_BOOTSTRAP: &str = r#"
(function() {
  const listeners = Object.create(null);
  let currentWaitUntil = null;
  let skipWaitingRequested = false;
  let claimClientsRequested = false;
  const timerTasks = [];
  let nextTimerId = 1;

  class ExtendableEvent {
    constructor(type) {
      this.type = type;
      this.cancelable = false;
      this.defaultPrevented = false;
    }
    preventDefault() {
      if (this.cancelable) this.defaultPrevented = true;
    }
    waitUntil(value) {
      if (typeof currentWaitUntil !== 'function') {
        throw new Error('InvalidStateError: waitUntil called outside dispatch');
      }
      currentWaitUntil(value);
    }
  }
  class InstallEvent extends ExtendableEvent {}
  const outboundMessages = [];
  const portEndpoints = Object.create(null);
  let nextWorkerPortId = 1;
  function cloneMessage(data) {
    const dataJSON = JSON.stringify(data);
    if (dataJSON === undefined) throw new DOMException('message could not be cloned', 'DataCloneError');
    return dataJSON;
  }
  function preparePortTransfer(data, transfer) {
    const ids = [];
    let dataPortIndex = null;
    if (transfer !== undefined && transfer !== null) {
      for (const item of Array.from(transfer)) {
        if (!(item instanceof MessagePort) || item._closed || item._detached) {
          throw new DOMException('invalid MessagePort transfer', 'DataCloneError');
        }
        if (ids.indexOf(item._hostPortId) >= 0) {
          throw new DOMException('duplicate MessagePort transfer', 'DataCloneError');
        }
        let portId = item._hostPortId;
        if (portId === null) {
          portId = nextWorkerPortId;
          nextWorkerPortId += 2;
          if (!item._other) throw new DOMException('MessagePort is not entangled', 'DataCloneError');
          item._other._other = null;
          item._other._hostPortId = portId;
          item._other._remote = true;
          portEndpoints[String(portId)] = item._other;
        }
        if (data === item) dataPortIndex = ids.length;
        ids.push(portId);
        item._other = null;
        item._detached = true;
      }
    }
    return {
      dataJSON: dataPortIndex === null ? cloneMessage(data) : 'null',
      transferredPortIds: ids,
      dataPortIndex: dataPortIndex
    };
  }
  function queueOutbound(data, transfer, portId, targetClientId) {
    const wire = preparePortTransfer(data, transfer);
    wire.portId = portId;
    wire.targetClientId = targetClientId;
    outboundMessages.push(wire);
  }
  const clientToken = {};
  class Client {
    constructor(info, token) {
      if (token !== clientToken) throw new TypeError('Illegal constructor');
      Object.defineProperties(this, {
        id: {value: info.id, enumerable: true},
        url: {value: info.url, enumerable: true},
        type: {value: info.type, enumerable: true},
        frameType: {value: info.frameType, enumerable: true},
        visibilityState: {value: info.visibilityState, enumerable: true},
        focused: {value: info.focused === true, enumerable: true}
      });
    }
    postMessage(data, transfer) {
      queueOutbound(data, transfer, null, this.id);
    }
  }
  class MessagePort {
    constructor() {
      this._listeners = [];
      this._onmessage = null;
      this._other = null;
      this._hostPortId = null;
      this._remote = false;
      this._closed = false;
      this._detached = false;
    }
    addEventListener(type, listener) {
      if (type === 'message' && typeof listener === 'function') this._listeners.push(listener);
    }
    removeEventListener(type, listener) {
      if (type !== 'message') return;
      const index = this._listeners.indexOf(listener);
      if (index >= 0) this._listeners.splice(index, 1);
    }
    dispatchEvent(event) {
      for (const listener of this._listeners.slice()) listener.call(this, event);
      return true;
    }
    postMessage(data, transfer) {
      if (this._closed || this._detached) return;
      if (this._remote) {
        queueOutbound(data, transfer, this._hostPortId, null);
        return;
      }
      if (!this._other) return;
      const wire = preparePortTransfer(data, transfer);
      const other = this._other;
      queueMicrotask(function() {
        if (!other._closed && !other._detached) {
          other.dispatchEvent(new MessageEvent('message', {
            data: wire.dataPortIndex === null ? JSON.parse(wire.dataJSON) : null,
            ports: []
          }));
        }
      });
    }
    start() {}
    close() {
      this._closed = true;
      if (this._hostPortId !== null) delete portEndpoints[String(this._hostPortId)];
      if (this._other) this._other._other = null;
      this._other = null;
    }
    get onmessage() { return this._onmessage; }
    set onmessage(listener) {
      if (this._onmessage) this.removeEventListener('message', this._onmessage);
      this._onmessage = typeof listener === 'function' ? listener : null;
      if (this._onmessage) this.addEventListener('message', this._onmessage);
    }
  }
  class MessageChannel {
    constructor() {
      this.port1 = new MessagePort();
      this.port2 = new MessagePort();
      this.port1._other = this.port2;
      this.port2._other = this.port1;
    }
  }
  class MessageEvent extends ExtendableEvent {
    constructor(type, init) {
      super(type);
      this.data = init.data;
      this.origin = '';
      this.source = null;
      this.ports = init.ports || [];
    }
  }
  class DOMException extends Error {
    constructor(message, name) {
      super(String(message));
      this.name = name === undefined ? 'Error' : String(name);
      this.code = this.name === 'NetworkError' ? 19 : 0;
    }
  }
  Object.defineProperty(DOMException, 'NETWORK_ERR', {value: 19});
  Object.defineProperty(DOMException.prototype, 'NETWORK_ERR', {value: 19});

  // https://fetch.spec.whatwg.org/#headers-class
  class Headers {
    constructor(init) {
      this._pairs = [];
      this._guard = 'none';
      if (init === undefined || init === null) return;
      if (init instanceof Headers) {
        for (const pair of init._pairs) this.append(pair[0], pair[1]);
      } else if (typeof init[Symbol.iterator] === 'function') {
        for (const pair of init) {
          const values = Array.from(pair);
          if (values.length !== 2) throw new TypeError('header entry must contain two items');
          this.append(values[0], values[1]);
        }
      } else {
        for (const name of Object.keys(init)) this.append(name, init[name]);
      }
    }
    append(name, value) {
      if (this._guard === 'immutable') {
        throw new TypeError('Headers are immutable');
      }
      this._pairs.push([String(name).toLowerCase(), String(value)]);
    }
    get(name) {
      name = String(name).toLowerCase();
      const values = this._pairs.filter(pair => pair[0] === name).map(pair => pair[1]);
      return values.length === 0 ? null : values.join(', ');
    }
    has(name) {
      name = String(name).toLowerCase();
      return this._pairs.some(pair => pair[0] === name);
    }
    entries() {
      return this._pairs.map(pair => [pair[0], pair[1]])[Symbol.iterator]();
    }
    [Symbol.iterator]() {
      return this.entries();
    }
  }
  Object.defineProperty(Headers.prototype, Symbol.toStringTag, {value: 'Headers'});

  function normalizeBody(body) {
    if (body === undefined || body === null) return '';
    return String(body);
  }
  function normalizeRequestURL(input) {
    const source = String(input);
    try {
      const base = globalThis.location && globalThis.location.href ? String(globalThis.location.href) : '';
      const response = JSON.parse(globalThis.__zwResolveURL(source, base));
      if (response && response.ok === true && response.href) return response.href;
    } catch (_error) {}
    return source;
  }

  // https://fetch.spec.whatwg.org/#request-class
  class Request {
    constructor(input, init) {
      init = init === undefined ? {} : Object(init);
      if (input instanceof Request) {
        this.url = init.url === undefined ? input.url : normalizeRequestURL(init.url);
        this.method = init.method === undefined ? input.method : String(init.method).toUpperCase();
        this.headers = new Headers(init.headers === undefined ? input.headers : init.headers);
        this._body = init.body === undefined ? input._body : normalizeBody(init.body);
        this.mode = init.mode === undefined ? input.mode : String(init.mode);
        this.credentials = init.credentials === undefined ? input.credentials : String(init.credentials);
        this.redirect = init.redirect === undefined ? input.redirect : String(init.redirect);
        this.referrer = init.referrer === undefined ? input.referrer : String(init.referrer);
      } else if (typeof input === 'object' && input !== null && input.url !== undefined) {
        this.url = normalizeRequestURL(input.url);
        this.method = init.method === undefined
          ? String(input.method || 'GET').toUpperCase()
          : String(init.method).toUpperCase();
        this.headers = new Headers(init.headers === undefined ? input.headers : init.headers);
        this._body = init.body === undefined ? normalizeBody(input.body) : normalizeBody(init.body);
        this.mode = init.mode === undefined ? String(input.mode || 'cors') : String(init.mode);
        this.credentials = init.credentials === undefined ? String(input.credentials || 'same-origin') : String(init.credentials);
        this.redirect = init.redirect === undefined ? String(input.redirect || 'follow') : String(init.redirect);
        this.referrer = init.referrer === undefined ? String(input.referrer || '') : String(init.referrer);
        if (input.headerGuard === 'immutable' && init.headers === undefined) {
          this.headers._guard = 'immutable';
        }
      } else {
        this.url = normalizeRequestURL(input);
        this.method = init.method === undefined ? 'GET' : String(init.method).toUpperCase();
        this.headers = new Headers(init.headers);
        this._body = normalizeBody(init.body);
        this.mode = init.mode === undefined ? 'cors' : String(init.mode);
        this.credentials = init.credentials === undefined ? 'same-origin' : String(init.credentials);
        this.redirect = init.redirect === undefined ? 'follow' : String(init.redirect);
        this.referrer = init.referrer === undefined ? '' : String(init.referrer);
      }
    }
    text() {
      return Promise.resolve(this._body);
    }
    clone() {
      return new Request(this);
    }
  }
  Object.defineProperty(Request.prototype, Symbol.toStringTag, {value: 'Request'});

  // https://fetch.spec.whatwg.org/#response-class
  class Response {
    constructor(body, init) {
      init = init === undefined ? {} : Object(init);
      const status = init.status === undefined ? 200 : Number(init.status);
      if (!Number.isInteger(status) || status < 200 || status > 599) {
        throw new RangeError('Response status is outside the supported range');
      }
      this.status = status;
      this.statusText = init.statusText === undefined ? '' : String(init.statusText);
      this.headers = new Headers(init.headers);
      this._body = normalizeBody(body);
      this.bodyUsed = false;
      this.ok = status >= 200 && status <= 299;
      this.type = 'default';
    }
    text() {
      this.bodyUsed = true;
      return Promise.resolve(this._body);
    }
    clone() {
      if (this.bodyUsed) throw new TypeError('Response body has already been used');
      const cloned = new Response(this._body, {
        status: this.status,
        statusText: this.statusText,
        headers: this.headers
      });
      cloned.type = this.type;
      return cloned;
    }
    static error() {
      const response = Object.create(Response.prototype);
      response.status = 0;
      response.statusText = '';
      response.headers = new Headers();
      response._body = '';
      response.bodyUsed = false;
      response.ok = false;
      response.type = 'error';
      return response;
    }
    static _from(value) {
      if (value instanceof Response) return value;
      throw new TypeError('FetchEvent.respondWith must resolve with a Response');
    }
    static _serialize(response) {
      if (response.bodyUsed) throw new TypeError('Response body has already been used');
      return {
        status: response.status,
        statusText: response.statusText,
        type: response.type || 'default',
        headers: Array.from(response.headers),
        body: response._body
      };
    }
  }
  Object.defineProperty(Response.prototype, Symbol.toStringTag, {value: 'Response'});

  function cacheRequestWire(input) {
    const request = input instanceof Request ? input : new Request(input);
    return {
      url: request.url,
      method: request.method,
      headers: Array.from(request.headers),
      body: request._body,
      clientId: null,
      resultingClientId: null
    };
  }
  function cacheQueryOptionsWire(options) {
    options = options === undefined || options === null ? {} : Object(options);
    return {
      ignoreSearch: !!options.ignoreSearch,
      ignoreMethod: !!options.ignoreMethod,
      ignoreVary: !!options.ignoreVary
    };
  }
  function cacheRequestIsHttp(request) {
    return /^https?:/i.test(String(request.url || ''));
  }
  function cacheResponseVaryHasStar(response) {
    if (!response || !response.headers || typeof response.headers.get !== 'function') return false;
    const vary = response.headers.get('vary');
    if (vary === null) return false;
    return String(vary).split(',').some(field => field.trim().toLowerCase() === '*');
  }
  function validateCachePut(request, response) {
    // https://w3c.github.io/ServiceWorker/#cache-put
    if (request.method !== 'GET') {
      throw new TypeError('Cache.put request method must be GET');
    }
    if (!cacheRequestIsHttp(request)) {
      throw new TypeError('Cache.put request URL must be an HTTP(S) URL');
    }
    if (response.status === 206) {
      throw new TypeError('Cache.put cannot store a 206 Partial Content response');
    }
    if (cacheResponseVaryHasStar(response)) {
      throw new TypeError('Cache.put cannot store a response with Vary: *');
    }
  }
  function cachedResponseFromWire(response) {
    if (String(response.type || 'default').toLowerCase() === 'error') {
      return Response.error();
    }
    return new Response(response.body || '', {
      status: response.status,
      statusText: response.statusText || '',
      headers: response.headers || []
    });
  }
  function cachedRequestFromWire(request) {
    return new Request(request.url, {
      method: request.method || 'GET',
      headers: request.headers || [],
      body: request.body == null ? undefined : request.body
    });
  }
  function cacheStorageHost(request) {
    let response;
    try {
      response = JSON.parse(globalThis.__zwCacheStorage(JSON.stringify(request)));
    } catch (_error) {
      response = {ok: false, error: 'invalid CacheStorage host response'};
    }
    if (!response || response.ok !== true) {
      return Promise.reject(new TypeError(response && response.error || 'CacheStorage operation failed'));
    }
    return Promise.resolve(response);
  }
  function cacheDomStringWire(value) {
    const s = String(value);
    let out = '';
    for (let i = 0; i < s.length; i++) {
      let unit = s.charCodeAt(i).toString(16);
      while (unit.length < 4) unit = '0' + unit;
      out += unit;
    }
    return out;
  }
  function cacheDomStringFromWire(units) {
    units = String(units || '');
    if (units.length % 4 !== 0) throw new TypeError('malformed CacheStorage name');
    let out = '';
    for (let i = 0; i < units.length; i += 4) {
      const unit = parseInt(units.slice(i, i + 4), 16);
      if (!isFinite(unit)) throw new TypeError('malformed CacheStorage name');
      out += String.fromCharCode(unit);
    }
    return out;
  }
  function cacheSetNameWire(target, field, value) {
    const s = String(value);
    let hasSurrogate = false;
    for (let i = 0; i < s.length; i++) {
      const unit = s.charCodeAt(i);
      if (unit >= 0xD800 && unit <= 0xDFFF) {
        hasSurrogate = true;
        break;
      }
    }
    if (!hasSurrogate) target[field] = s;
    target[field + 'Units'] = cacheDomStringWire(s);
  }
  function cacheNameFromResult(response, fallback) {
    if (response && typeof response.cacheNameUnits === 'string') {
      return cacheDomStringFromWire(response.cacheNameUnits);
    }
    if (response && typeof response.nameUnits === 'string') {
      return cacheDomStringFromWire(response.nameUnits);
    }
    if (response && typeof response.cacheName === 'string') return response.cacheName;
    if (response && typeof response.name === 'string') return response.name;
    return fallback;
  }
  function cacheSetIdWire(target, cache) {
    if (cache && cache._cacheId !== null && isFinite(cache._cacheId)) target.cacheId = cache._cacheId;
  }
  function cacheMatchRequest(input, cacheName, options, cache) {
    const request = {
      op: 'match',
      request: cacheRequestWire(input),
      options: cacheQueryOptionsWire(options)
    };
    if (cacheName !== undefined) cacheSetNameWire(request, 'cacheName', cacheName);
    cacheSetIdWire(request, cache);
    return request;
  }
  function cacheMatchAllRequest(cache, input, options) {
    const request = {
      op: 'matchAll',
      options: cacheQueryOptionsWire(options)
    };
    cacheSetNameWire(request, 'cacheName', cache._name);
    cacheSetIdWire(request, cache);
    if (input !== undefined) request.request = cacheRequestWire(input);
    return request;
  }
  function cacheKeysRequest(cache, input, options) {
    const request = {
      op: 'keys',
      options: cacheQueryOptionsWire(options)
    };
    cacheSetNameWire(request, 'cacheName', cache._name);
    cacheSetIdWire(request, cache);
    if (input !== undefined) request.request = cacheRequestWire(input);
    return request;
  }
  function cacheDeleteRequest(cache, input, options) {
    const request = {
      op: 'delete',
      request: cacheRequestWire(input),
      options: cacheQueryOptionsWire(options)
    };
    cacheSetNameWire(request, 'cacheName', cache._name);
    cacheSetIdWire(request, cache);
    return request;
  }
  function cachePutRequest(cache, input, response) {
    const request = input instanceof Request ? input : new Request(input);
    const cacheResponse = Response._from(response);
    validateCachePut(request, cacheResponse);
    const hostRequest = {
      op: 'put',
      request: cacheRequestWire(request),
      response: Response._serialize(cacheResponse)
    };
    cacheSetNameWire(hostRequest, 'cacheName', cache._name);
    cacheSetIdWire(hostRequest, cache);
    return hostRequest;
  }
  // https://w3c.github.io/ServiceWorker/#cache-interface
  class Cache {
    constructor(name, cacheId) {
      this._name = String(name);
      this._cacheId = cacheId === undefined || cacheId === null ? null : Number(cacheId);
    }
    match(input, options) {
      return cacheStorageHost(cacheMatchRequest(input, this._name, options, this)).then(function(response) {
        return response.response === null ? undefined : cachedResponseFromWire(response.response);
      });
    }
    matchAll(input, options) {
      return cacheStorageHost(cacheMatchAllRequest(this, input, options)).then(function(response) {
        const responses = Array.isArray(response.responses) ? response.responses : [];
        return responses.map(cachedResponseFromWire);
      });
    }
    put(input, response) {
      let request;
      try {
        request = cachePutRequest(this, input, response);
      } catch (error) {
        return Promise.reject(error);
      }
      return cacheStorageHost(request).then(function() {
        return undefined;
      });
    }
    add(input) {
      const cache = this;
      let request;
      try {
        request = input instanceof Request ? input : new Request(input);
      } catch (error) {
        return Promise.reject(error);
      }
      if (request.method !== 'GET') {
        return Promise.reject(new TypeError('Cache.add only supports GET requests'));
      }
      if (!cacheRequestIsHttp(request)) {
        return Promise.reject(new TypeError('Cache.add request URL must be an HTTP(S) URL'));
      }
      return fetch(request.clone()).then(function(response) {
        if (!response || !response.ok) {
          throw new TypeError('Cache.add fetch response is not ok');
        }
        validateCachePut(request, response);
        return cache.put(request, response);
      });
    }
    addAll(inputs) {
      const cache = this;
      try {
        const requests = Array.from(inputs).map(function(input) {
          const request = input instanceof Request ? input : new Request(input);
          if (request.method !== 'GET') {
            throw new TypeError('Cache.addAll only supports GET requests');
          }
          if (!cacheRequestIsHttp(request)) {
            throw new TypeError('Cache.addAll request URL must be an HTTP(S) URL');
          }
          return request;
        });
        return Promise.all(requests.map(function(request) {
          return fetch(request.clone()).then(function(response) {
            if (!response || !response.ok) {
              throw new TypeError('Cache.addAll fetch response is not ok');
            }
            validateCachePut(request, response);
            return {request, response};
          });
        })).then(function(entries) {
          let chain = Promise.resolve();
          for (const entry of entries) {
            chain = chain.then(function() {
              return cache.put(entry.request, entry.response);
            });
          }
          return chain;
        }).then(function() {
          return undefined;
        });
      } catch (error) {
        return Promise.reject(error);
      }
    }
    keys(input, options) {
      return cacheStorageHost(cacheKeysRequest(this, input, options)).then(function(response) {
        const requests = Array.isArray(response.requests) ? response.requests : [];
        return requests.map(cachedRequestFromWire);
      });
    }
    delete(input, options) {
      let request;
      try {
        request = cacheDeleteRequest(this, input, options);
      } catch (error) {
        return Promise.reject(error);
      }
      return cacheStorageHost(request).then(function(response) {
        return Boolean(response.value);
      });
    }
  }
  Object.defineProperty(Cache.prototype, Symbol.toStringTag, {value: 'Cache'});

  // https://w3c.github.io/ServiceWorker/#cache-storage-interface
  class CacheStorage {
    open(name) {
      const hasName = arguments.length >= 1;
      return new Promise(function(resolve, reject) {
        try {
          if (!hasName) throw new TypeError('CacheStorage.open requires a name');
          const request = {op: 'open'};
          const fallback = String(name);
          cacheSetNameWire(request, 'name', fallback);
          cacheStorageHost(request).then(function(response) {
            resolve(new Cache(cacheNameFromResult(response, fallback), response.cacheId));
          }, reject);
        } catch (error) {
          reject(error);
        }
      });
    }
    match(input, options) {
      const cacheName = options === undefined || options === null ? undefined : Object(options).cacheName;
      return cacheStorageHost(cacheMatchRequest(input, cacheName === undefined ? undefined : String(cacheName), options)).then(function(response) {
        return response.response === null ? undefined : cachedResponseFromWire(response.response);
      });
    }
    has(name) {
      const hasName = arguments.length >= 1;
      return new Promise(function(resolve, reject) {
        try {
          if (!hasName) throw new TypeError('CacheStorage.has requires a name');
          const request = {op: 'has'};
          cacheSetNameWire(request, 'name', name);
          cacheStorageHost(request).then(function(response) {
            resolve(Boolean(response.value));
          }, reject);
        } catch (error) {
          reject(error);
        }
      });
    }
    delete(name) {
      const hasName = arguments.length >= 1;
      return new Promise(function(resolve, reject) {
        try {
          if (!hasName) throw new TypeError('CacheStorage.delete requires a name');
          const request = {op: 'storageDelete'};
          cacheSetNameWire(request, 'name', name);
          cacheStorageHost(request).then(function(response) {
            resolve(Boolean(response.value));
          }, reject);
        } catch (error) {
          reject(error);
        }
      });
    }
    keys() {
      return cacheStorageHost({op: 'storageKeys'}).then(function(response) {
        if (Array.isArray(response.cacheNameUnits)) {
          return response.cacheNameUnits.map(cacheDomStringFromWire);
        }
        return Array.isArray(response.cacheNames) ? response.cacheNames.map(String) : [];
      });
    }
  }
  Object.defineProperty(CacheStorage.prototype, Symbol.toStringTag, {value: 'CacheStorage'});

  // https://w3c.github.io/ServiceWorker/#fetch-event-interface
  class FetchEvent extends ExtendableEvent {
    constructor(type, init) {
      super(type);
      this.request = init.request;
      this.clientId = init.clientId || '';
      this.resultingClientId = init.resultingClientId || '';
      this._respondWith = null;
    }
    respondWith(value) {
      if (typeof this._respondWith !== 'function') {
        throw new DOMException('respondWith called outside dispatch', 'InvalidStateError');
      }
      this._respondWith(value);
    }
  }
  Object.defineProperty(FetchEvent.prototype, Symbol.toStringTag, {value: 'FetchEvent'});

  function parseURLSearchParams(input) {
    const pairs = [];
    let source = String(input);
    if (source.startsWith('?')) source = source.slice(1);
    if (source === '') return pairs;
    for (const entry of source.split('&')) {
      if (entry === '') continue;
      const separator = entry.indexOf('=');
      const name = separator < 0 ? entry : entry.slice(0, separator);
      const value = separator < 0 ? '' : entry.slice(separator + 1);
      pairs.push([
        decodeURIComponent(name.replace(/\+/g, ' ')),
        decodeURIComponent(value.replace(/\+/g, ' '))
      ]);
    }
    return pairs;
  }
  function encodeURLSearchParam(value) {
    return encodeURIComponent(value).replace(/%20/g, '+');
  }
  class URLSearchParams {
    constructor(init) {
      this._pairs = [];
      if (init === undefined || init === null) return;
      if (typeof init === 'string') {
        this._pairs = parseURLSearchParams(init);
      } else if (typeof init[Symbol.iterator] === 'function') {
        for (const pair of init) {
          if (!pair || typeof pair[Symbol.iterator] !== 'function') {
            throw new TypeError('URLSearchParams sequence entry is not iterable');
          }
          const values = Array.from(pair);
          if (values.length !== 2) {
            throw new TypeError('URLSearchParams sequence entry must contain two items');
          }
          this._pairs.push([String(values[0]), String(values[1])]);
        }
      } else {
        for (const name of Object.keys(init)) {
          this._pairs.push([String(name), String(init[name])]);
        }
      }
    }
    append(name, value) {
      this._pairs.push([String(name), String(value)]);
    }
    delete(name, value) {
      name = String(name);
      const hasValue = arguments.length > 1;
      if (hasValue) value = String(value);
      this._pairs = this._pairs.filter(pair => pair[0] !== name || (hasValue && pair[1] !== value));
    }
    get(name) {
      name = String(name);
      const pair = this._pairs.find(pair => pair[0] === name);
      return pair ? pair[1] : null;
    }
    getAll(name) {
      name = String(name);
      return this._pairs.filter(pair => pair[0] === name).map(pair => pair[1]);
    }
    has(name, value) {
      name = String(name);
      const hasValue = arguments.length > 1;
      if (hasValue) value = String(value);
      return this._pairs.some(pair => pair[0] === name && (!hasValue || pair[1] === value));
    }
    set(name, value) {
      name = String(name);
      value = String(value);
      const first = this._pairs.findIndex(pair => pair[0] === name);
      if (first < 0) {
        this._pairs.push([name, value]);
        return;
      }
      this._pairs[first][1] = value;
      this._pairs = this._pairs.filter((pair, index) => pair[0] !== name || index === first);
    }
    sort() {
      this._pairs = this._pairs
        .map((pair, index) => ({pair, index}))
        .sort((left, right) => left.pair[0] < right.pair[0] ? -1 : left.pair[0] > right.pair[0] ? 1 : left.index - right.index)
        .map(entry => entry.pair);
    }
    entries() {
      return this._pairs.map(pair => [pair[0], pair[1]])[Symbol.iterator]();
    }
    keys() {
      return this._pairs.map(pair => pair[0])[Symbol.iterator]();
    }
    values() {
      return this._pairs.map(pair => pair[1])[Symbol.iterator]();
    }
    forEach(callback, thisArg) {
      for (const pair of this._pairs) callback.call(thisArg, pair[1], pair[0], this);
    }
    toString() {
      return this._pairs
        .map(pair => encodeURLSearchParam(pair[0]) + '=' + encodeURLSearchParam(pair[1]))
        .join('&');
    }
    [Symbol.iterator]() {
      return this.entries();
    }
  }
  Object.defineProperty(URLSearchParams.prototype, Symbol.toStringTag, {value: 'URLSearchParams'});

  const workerLocationToken = {};
  class WorkerLocation {
    constructor(parts, token) {
      if (token !== workerLocationToken) throw new TypeError('Illegal constructor');
      for (const name of Object.keys(parts)) {
        Object.defineProperty(this, name, {value: parts[name], enumerable: true});
      }
    }
    toString() {
      return this.href;
    }
  }
  Object.defineProperty(WorkerLocation.prototype, Symbol.toStringTag, {value: 'WorkerLocation'});
  class URL {
    constructor(input, base) {
      const response = JSON.parse(globalThis.__zwResolveURL(
        String(input), base === undefined ? '' : String(base)));
      if (!response || response.ok !== true) {
        throw new TypeError(response && response.error || 'Invalid URL');
      }
      for (const name of ['href', 'origin', 'protocol', 'host', 'hostname', 'port', 'pathname', 'search', 'hash']) {
        Object.defineProperty(this, name, {
          value: response[name],
          enumerable: true,
          configurable: true,
          writable: true
        });
      }
    }
    toString() {
      return this.href;
    }
  }
  Object.defineProperty(URL.prototype, Symbol.toStringTag, {value: 'URL'});
  globalThis.__zwSetLocation = function(parts) {
    Object.defineProperty(globalThis, 'location', {
      value: new WorkerLocation(parts, workerLocationToken),
      enumerable: true,
      configurable: true
    });
  };

  function WorkerGlobalScope() {}
  WorkerGlobalScope.prototype = Object.create(Object.getPrototypeOf(globalThis));
  function ServiceWorkerGlobalScope() {}
  ServiceWorkerGlobalScope.prototype = Object.create(WorkerGlobalScope.prototype);
  Object.defineProperty(ServiceWorkerGlobalScope.prototype, 'constructor', {
    value: ServiceWorkerGlobalScope,
    configurable: true,
    writable: true
  });
  Object.setPrototypeOf(globalThis, ServiceWorkerGlobalScope.prototype);
  globalThis.self = globalThis;
  globalThis.WorkerGlobalScope = WorkerGlobalScope;
  globalThis.ServiceWorkerGlobalScope = ServiceWorkerGlobalScope;
  globalThis.ExtendableEvent = ExtendableEvent;
  globalThis.InstallEvent = InstallEvent;
  globalThis.MessageEvent = MessageEvent;
  globalThis.MessagePort = MessagePort;
  globalThis.MessageChannel = MessageChannel;
  globalThis.Headers = globalThis.Headers || Headers;
  globalThis.Request = globalThis.Request || Request;
  globalThis.Response = globalThis.Response || Response;
  globalThis.Cache = globalThis.Cache || Cache;
  globalThis.CacheStorage = globalThis.CacheStorage || CacheStorage;
  globalThis.caches = globalThis.caches || new CacheStorage();
  globalThis.FetchEvent = FetchEvent;
  globalThis.DOMException = globalThis.DOMException || DOMException;
  globalThis.URLSearchParams = globalThis.URLSearchParams || URLSearchParams;
  globalThis.URL = globalThis.URL || URL;
  globalThis.WorkerLocation = WorkerLocation;
  globalThis.Client = Client;
  globalThis.oninstall = null;
  globalThis.onactivate = null;
  globalThis.onmessage = null;
  globalThis.onfetch = null;
  globalThis.setTimeout = function(callback, delay) {
    if (typeof callback !== 'function') {
      throw new TypeError('setTimeout callback must be a function');
    }
    const id = nextTimerId++;
    timerTasks.push({id, callback, args: Array.prototype.slice.call(arguments, 2)});
    return id;
  };
  globalThis.clearTimeout = function(id) {
    id = Number(id);
    for (let i = 0; i < timerTasks.length; i++) {
      if (timerTasks[i].id === id) {
        timerTasks.splice(i, 1);
        return;
      }
    }
  };
  globalThis.__zwRunOneTask = function() {
    const task = timerTasks.shift();
    if (!task) return false;
    task.callback.apply(globalThis, task.args);
    return true;
  };
  globalThis.addEventListener = function(type, listener) {
    if (typeof listener !== 'function') return;
    (listeners[String(type)] || (listeners[String(type)] = [])).push(listener);
  };
  globalThis.removeEventListener = function(type, listener) {
    const list = listeners[String(type)] || [];
    const index = list.indexOf(listener);
    if (index >= 0) list.splice(index, 1);
  };
  globalThis.skipWaiting = function() {
    skipWaitingRequested = true;
    if (globalThis.__zwLifecycleResult) {
      globalThis.__zwLifecycleResult.skipWaitingRequested = true;
    }
    return Promise.resolve();
  };
  globalThis.fetch = function(input, init) {
    let request;
    try {
      request = new Request(input, init);
    } catch (error) {
      return Promise.reject(error);
    }
    try {
      const response = JSON.parse(globalThis.__zwFetch(JSON.stringify(cacheRequestWire(request))));
      if (!response || response.ok !== true) {
        return Promise.reject(new TypeError(response && response.error || 'Service Worker fetch failed'));
      }
      return Promise.resolve(cachedResponseFromWire(response.response));
    } catch (_error) {
      return Promise.reject(new TypeError('invalid Service Worker fetch response'));
    }
  };
  class Clients {
    // https://w3c.github.io/ServiceWorker/#clients-get
    get(id) {
      if (arguments.length < 1) {
        throw new TypeError('Clients.get requires a client id');
      }
      id = String(id);
      if (id === '') return Promise.resolve(undefined);
      let response;
      try {
        response = JSON.parse(globalThis.__zwClientsGet(id));
      } catch (_error) {
        response = {ok: false, error: 'invalid Clients.get response'};
      }
      if (!response || response.ok !== true) {
        return Promise.reject(new TypeError(response && response.error || 'Clients.get failed'));
      }
      return Promise.resolve(response.client === null ? undefined : new Client(response.client, clientToken));
    }
    matchAll(options) {
      options = options === undefined ? {} : Object(options);
      const includeUncontrolled = options.includeUncontrolled === true;
      const type = options.type === undefined ? 'window' : String(options.type);
      if (type !== 'window' && type !== 'worker' && type !== 'sharedworker' && type !== 'all') {
        return Promise.reject(new TypeError('invalid ClientQueryOptions type'));
      }
      let response;
      try {
        response = JSON.parse(globalThis.__zwClientsMatchAll(
          includeUncontrolled ? 'true' : 'false', type));
      } catch (_error) {
        response = {ok: false, error: 'invalid Clients.matchAll response'};
      }
      if (!response || response.ok !== true || !Array.isArray(response.clients)) {
        return Promise.reject(new TypeError(response && response.error || 'Clients.matchAll failed'));
      }
      return Promise.resolve(response.clients.map(function(info) {
        return new Client(info, clientToken);
      }));
    }
    claim() {
      claimClientsRequested = true;
      if (globalThis.__zwLifecycleResult) {
        globalThis.__zwLifecycleResult.claimClientsRequested = true;
      }
      return Promise.resolve();
    }
  }
  globalThis.Clients = Clients;
  globalThis.clients = new Clients();
  const registration = {
    update: function() {
      let response;
      try {
        response = JSON.parse(globalThis.__zwRequestUpdate());
      } catch (_error) {
        response = {ok: false, name: 'TypeError', message: 'invalid Service Worker update response'};
      }
      if (response && response.ok === true) return Promise.resolve(registration);
      return Promise.reject(new globalThis.DOMException(
        response && response.message || 'Service Worker update failed',
        response && response.name || 'TypeError'
      ));
    }
  };
  globalThis.registration = registration;
  function importScriptsNetworkError(message) {
    return new globalThis.DOMException(String(message), 'NetworkError');
  }
  globalThis.__zwModuleScriptMode = false;
  globalThis.importScripts = function() {
    if (globalThis.__zwModuleScriptMode) {
      throw new TypeError('importScripts is unavailable in module workers');
    }
    const specifiers = [];
    for (let i = 0; i < arguments.length; i++) {
      specifiers.push(String(arguments[i]));
    }
    if (specifiers.length === 0) return;
    let response;
    try {
      response = JSON.parse(globalThis.__zwImportScripts.apply(globalThis, specifiers));
    } catch (error) {
      throw importScriptsNetworkError('invalid importScripts host response');
    }
    if (!response || response.ok !== true || !Array.isArray(response.sources)) {
      throw importScriptsNetworkError(response && response.error || 'importScripts failed');
    }
    for (let i = 0; i < response.sources.length; i++) {
      (0, eval)(String(response.sources[i]));
    }
  };
  globalThis.__zwDispatchLifecycle = function(type, eventId) {
    const pending = [];
    claimClientsRequested = false;
    const result = {
      eventId: String(eventId),
      phase: String(type),
      settled: false,
      succeeded: false,
      message: '',
      skipWaitingRequested: skipWaitingRequested,
      claimClientsRequested: false
    };
    globalThis.__zwLifecycleResult = result;
    currentWaitUntil = function(value) {
      pending.push(Promise.resolve(value));
    };
    try {
      const EventClass = type === 'install' ? InstallEvent : ExtendableEvent;
      const event = new EventClass(type);
      const callbacks = (listeners[type] || []).slice();
      for (let i = 0; i < callbacks.length; i++) callbacks[i].call(globalThis, event);
      const propertyHandler = globalThis['on' + type];
      if (typeof propertyHandler === 'function') propertyHandler.call(globalThis, event);
    } catch (error) {
      currentWaitUntil = null;
      result.settled = true;
      result.message = String(error && error.message || error);
      return;
    }
    currentWaitUntil = null;
    Promise.all(pending).then(function() {
      result.settled = true;
      result.succeeded = true;
    }, function(error) {
      result.settled = true;
      result.message = String(error && error.message || error);
    });
  };
  function materializeTransferredPorts(portIds) {
    return portIds.map(function(portId) {
      const key = String(portId);
      if (portEndpoints[key]) throw new DOMException('MessagePort already exists', 'DataCloneError');
      const port = new MessagePort();
      port._hostPortId = portId;
      port._remote = true;
      portEndpoints[key] = port;
      return port;
    });
  }
  globalThis.__zwDispatchMessage = function(
      eventId, data, clientId, clientURL, portIds, dataPortIndex, targetPortId) {
    outboundMessages.splice(0, outboundMessages.length);
    const ports = materializeTransferredPorts(portIds || []);
    const eventData = dataPortIndex === null ? data : ports[dataPortIndex];
    const event = new MessageEvent('message', {data: eventData, ports: ports});
    const pending = [];
    currentWaitUntil = function(value) {
      pending.push(Promise.resolve(value));
    };
    try {
      if (targetPortId !== null) {
        const port = portEndpoints[String(targetPortId)];
        if (!port || port._closed || port._detached) {
          throw new DOMException('MessagePort endpoint does not exist', 'InvalidStateError');
        }
        port.dispatchEvent(event);
      } else {
        event.source = new Client({
          id: clientId,
          url: clientURL,
          type: 'window',
          frameType: 'top-level',
          visibilityState: 'visible',
          focused: false
        }, clientToken);
        const callbacks = (listeners.message || []).slice();
        for (let i = 0; i < callbacks.length; i++) callbacks[i].call(globalThis, event);
        if (typeof globalThis.onmessage === 'function') {
          globalThis.onmessage.call(globalThis, event);
        }
      }
      Promise.all(pending).catch(function() {});
      currentWaitUntil = null;
      return String(eventId);
    } catch (error) {
      currentWaitUntil = null;
      outboundMessages.splice(0, outboundMessages.length);
      throw error;
    }
  };
  globalThis.__zwTakeOutboundMessages = function() {
    return outboundMessages.splice(0, outboundMessages.length);
  };
  globalThis.__zwDispatchFetch = function(eventId, requestInfo) {
    const pending = [];
    let respondWithCalled = false;
    let respondWithAllowed = true;
    const result = {
      eventId: String(eventId),
      settled: false,
      responded: false,
      response: null,
      message: '',
      failed: false
    };
    globalThis.__zwFetchResult = result;
    currentWaitUntil = function(value) {
      pending.push(Promise.resolve(value));
    };
    let event;
    try {
      event = new FetchEvent('fetch', {
        request: new Request(requestInfo),
        clientId: requestInfo.clientId || '',
        resultingClientId: requestInfo.resultingClientId || ''
      });
      event.cancelable = true;
      event._respondWith = function(value) {
        if (!respondWithAllowed) {
          throw new DOMException('respondWith called after dispatch', 'InvalidStateError');
        }
        if (respondWithCalled) {
          result.failed = true;
          result.responded = false;
          result.response = null;
          result.settled = true;
          result.message = 'respondWith already called';
          throw new DOMException('respondWith already called', 'InvalidStateError');
        }
        respondWithCalled = true;
        pending.push(Promise.resolve(value).then(function(response) {
          if (result.failed) return;
          response = Response._from(response);
          result.responded = true;
          result.response = Response._serialize(response);
        }));
      };
      const callbacks = (listeners.fetch || []).slice();
      for (let i = 0; i < callbacks.length; i++) callbacks[i].call(globalThis, event);
      if (typeof globalThis.onfetch === 'function') {
        globalThis.onfetch.call(globalThis, event);
      }
      Promise.resolve().then(function() {
        respondWithAllowed = false;
        event._respondWith = null;
      });
    } catch (error) {
      respondWithAllowed = false;
      currentWaitUntil = null;
      if (result.failed || !respondWithCalled) {
        result.responded = false;
        result.response = null;
        result.settled = true;
        if (!result.message) result.message = String(error && error.message || error);
        return;
      }
    }
    currentWaitUntil = null;
    Promise.all(pending).then(function() {
      if (result.failed) return;
      if (!respondWithCalled && event.defaultPrevented) {
        result.failed = true;
        result.response = null;
        result.responded = false;
        result.message = 'FetchEvent default action was prevented without respondWith';
      }
      result.settled = true;
    }, function(error) {
      result.failed = true;
      result.response = null;
      result.responded = false;
      result.settled = true;
      result.message = String(error && error.message || error);
    });
  };
})();
'bootstrap-ready';
"#;

/// Service Worker script evaluation failure category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceWorkerScriptErrorKind {
    /// JavaScript could not be compiled.
    Compile,
    /// JavaScript threw while running.
    Runtime,
    /// JavaScript exceeded its execution deadline.
    Timeout,
    /// The host supplied an invalid script input.
    InvalidInput,
    /// The selected JavaScript engine could not be initialized.
    EngineUnavailable,
}

/// Lifecycle event phase dispatched inside the Service Worker global.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceWorkerLifecyclePhase {
    /// Install event.
    Install,
    /// Activate event.
    Activate,
}

impl ServiceWorkerLifecyclePhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Install => "install",
            Self::Activate => "activate",
        }
    }
}

/// Events emitted by [`ServiceWorkerRuntime`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceWorkerEvent {
    /// A script was evaluated successfully.
    Evaluated {
        /// URL associated with the evaluated script.
        script_url: String,
    },
    /// A script could not be evaluated.
    ScriptError {
        /// URL associated with the failed script.
        script_url: String,
        /// Stable error category for lifecycle coordination.
        kind: ServiceWorkerScriptErrorKind,
        /// Engine diagnostic message.
        message: String,
    },
    /// An install or activate event and all `waitUntil()` promises settled.
    LifecycleSettled {
        /// Host-assigned event ID.
        event_id: u64,
        /// Lifecycle phase.
        phase: ServiceWorkerLifecyclePhase,
        /// Whether dispatch and all lifetime promises fulfilled.
        succeeded: bool,
        /// Whether the worker called `skipWaiting()` before settlement.
        skip_waiting: bool,
        /// Whether the worker called `clients.claim()` during this lifecycle event.
        claim_clients: bool,
        /// Rejection or dispatch error diagnostic.
        message: String,
    },
    /// A page-to-worker message event was dispatched.
    MessageDispatched {
        /// Host-assigned event ID.
        event_id: u64,
        /// Browser-owned identity of the originating client.
        client_id: String,
        /// Messages posted by the worker to the originating client.
        outbound: Vec<ServiceWorkerOutboundMessage>,
    },
    /// A page-to-worker message handler threw.
    MessageFailed {
        /// Host-assigned event ID.
        event_id: u64,
        /// Browser-owned identity of the originating client.
        client_id: String,
        /// Handler diagnostic.
        message: String,
    },
    /// A fetch event settled after optional `respondWith()` handling.
    FetchSettled {
        /// Host-assigned event ID.
        event_id: u64,
        /// Request URL associated with this fetch event.
        request_url: String,
        /// Response supplied through `respondWith()`, or `None` for pass-through/failure.
        response: Option<ServiceWorkerFetchResponse>,
        /// True when `respondWith()` was called but failed, producing a network error.
        failed: bool,
        /// Handler or response-conversion diagnostic. Empty means success or pass-through.
        message: String,
    },
    /// A classic worker `importScripts()` call requires host-owned fetching.
    ImportScriptsRequested {
        /// Runtime-local request ID used to correlate the blocking response.
        request_id: u64,
        /// String-converted URL arguments in call order.
        specifiers: Vec<String>,
    },
    /// The worker global called `registration.update()`.
    UpdateRequested {
        /// Runtime-local request ID used to correlate the blocking response.
        request_id: u64,
    },
    /// The worker global requested a browser-owned client snapshot.
    ClientsMatchAllRequested {
        /// Runtime-local request ID used to correlate the blocking response.
        request_id: u64,
        /// Whether uncontrolled same-origin clients are included.
        include_uncontrolled: bool,
        /// Requested client type.
        client_type: String,
    },
    /// The worker global requested one browser-owned client by id.
    ClientsGetRequested {
        /// Runtime-local request ID used to correlate the blocking response.
        request_id: u64,
        /// Browser-owned client identity.
        client_id: String,
    },
    /// The worker global requested a browser-owned CacheStorage operation.
    CacheStorageRequested {
        /// Runtime-local request ID used to correlate the blocking response.
        request_id: u64,
        /// Pure-value CacheStorage operation for the active registration.
        request: ServiceWorkerCacheStorageRequest,
    },
    /// The worker global requested a browser-owned ordinary fetch.
    FetchRequested {
        /// Runtime-local request ID used to correlate the blocking response.
        request_id: u64,
        /// Pure-value fetch request.
        request: ServiceWorkerFetchRequest,
    },
    /// A worker posted messages outside a page-originated message event.
    ClientMessagesEmitted {
        /// Messages with explicit target client identities.
        outbound: Vec<ServiceWorkerOutboundMessage>,
    },
    /// The runtime thread exited.
    Closed,
    /// A module worker static dependency batch requires host-owned fetching.
    ModuleScriptsRequested {
        /// Runtime-local request ID used to correlate the blocking response.
        request_id: u64,
        /// Canonical URL of the module containing these import specifiers.
        referrer_url: String,
        /// Static import specifiers in source order.
        specifiers: Vec<String>,
    },
}

/// One worker-to-client message emitted during a worker event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceWorkerOutboundMessage {
    /// JSON-compatible structured payload.
    pub data_json: String,
    /// Port endpoint that emitted this message, or `None` for `Client.postMessage`.
    pub port_id: Option<u64>,
    /// Port endpoints transferred with this message.
    pub transferred_port_ids: Vec<u64>,
    /// Index into `transferred_port_ids` when the message payload is that port.
    pub data_port_index: Option<usize>,
    /// Browser-owned target client identity for `Client.postMessage()`.
    pub target_client_id: Option<String>,
}

/// Pure-value Service Worker client projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceWorkerClientInfo {
    /// Browser-owned stable identity for the current Document.
    pub id: String,
    /// Current committed client URL.
    pub url: String,
    /// Client type exposed to the worker.
    pub client_type: String,
    /// Frame type exposed to the worker.
    pub frame_type: String,
    /// Visibility state exposed to the worker.
    pub visibility_state: String,
    /// Whether the client currently has focus.
    pub focused: bool,
}

/// Pure-value fetch request projected into a Service Worker `FetchEvent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceWorkerFetchRequest {
    /// Absolute request URL.
    pub url: String,
    /// HTTP method.
    pub method: String,
    /// Request headers in wire order.
    pub headers: Vec<(String, String)>,
    /// UTF-8 request body for the current runtime-level MVP.
    pub body: Option<String>,
    /// Browser-owned source client identity, when the request has one.
    pub client_id: Option<String>,
    /// Browser-owned resulting client identity for navigation requests, when known.
    pub resulting_client_id: Option<String>,
    /// Fetch request referrer exposed to `FetchEvent.request`.
    pub referrer: Option<String>,
}

/// Pure-value fetch response produced by `FetchEvent.respondWith()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceWorkerFetchResponse {
    /// HTTP status code.
    pub status: u16,
    /// HTTP status text.
    pub status_text: String,
    /// Fetch response type (`default` unless a filtered response is represented).
    pub response_type: String,
    /// Response headers in worker-created order.
    pub headers: Vec<(String, String)>,
    /// UTF-8 response body for the current runtime-level MVP.
    pub body: String,
}

/// Pure-value Cache API query options requested by a Service Worker.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ServiceWorkerCacheQueryOptions {
    /// Ignore URL query parameters while matching requests.
    pub ignore_search: bool,
    /// Ignore request method while matching requests.
    pub ignore_method: bool,
    /// Ignore Vary headers while matching requests.
    pub ignore_vary: bool,
}

/// Worker-global CacheStorage operation requested from the browser-owned registration store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceWorkerCacheStorageRequest {
    /// Open or create one named cache.
    Open {
        /// Cache name.
        cache_name: String,
    },
    /// Match a request across all caches or within one named cache.
    Match {
        /// Optional cache name for `Cache.match()`.
        cache_name: Option<String>,
        /// Optional instance ID for an already-opened `Cache`.
        cache_id: Option<u64>,
        /// Request to match.
        request: ServiceWorkerFetchRequest,
        /// Query matching options.
        options: ServiceWorkerCacheQueryOptions,
    },
    /// Match all responses in one named cache, optionally filtering by request.
    MatchAll {
        /// Cache name.
        cache_name: String,
        /// Optional instance ID for an already-opened `Cache`.
        cache_id: Option<u64>,
        /// Optional request filter.
        request: Option<ServiceWorkerFetchRequest>,
        /// Query matching options.
        options: ServiceWorkerCacheQueryOptions,
    },
    /// List all request keys in one named cache.
    Keys {
        /// Cache name.
        cache_name: String,
        /// Optional instance ID for an already-opened `Cache`.
        cache_id: Option<u64>,
        /// Optional request filter.
        request: Option<ServiceWorkerFetchRequest>,
        /// Query matching options.
        options: ServiceWorkerCacheQueryOptions,
    },
    /// Delete matching entries in one named cache.
    Delete {
        /// Cache name.
        cache_name: String,
        /// Optional instance ID for an already-opened `Cache`.
        cache_id: Option<u64>,
        /// Request key.
        request: ServiceWorkerFetchRequest,
        /// Query matching options.
        options: ServiceWorkerCacheQueryOptions,
    },
    /// Store one response in one named cache.
    Put {
        /// Cache name.
        cache_name: String,
        /// Optional instance ID for an already-opened `Cache`.
        cache_id: Option<u64>,
        /// Request key.
        request: ServiceWorkerFetchRequest,
        /// Response value.
        response: ServiceWorkerFetchResponse,
    },
    /// Test whether one named cache exists.
    StorageHas {
        /// Cache name.
        cache_name: String,
    },
    /// Delete one named cache.
    StorageDelete {
        /// Cache name.
        cache_name: String,
    },
    /// List cache names in creation order.
    StorageKeys,
}

/// Result of a worker-global CacheStorage operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceWorkerCacheStorageResult {
    /// Operation completed without a payload.
    Done,
    /// CacheStorage.open result.
    Open {
        /// Cache name.
        cache_name: String,
        /// Cache name as UTF-16 code units.
        cache_name_units: String,
        /// Registration-local cache instance ID.
        cache_id: u64,
    },
    /// Cache match result.
    Match(Option<ServiceWorkerFetchResponse>),
    /// Cache matchAll result.
    MatchAll(Vec<ServiceWorkerFetchResponse>),
    /// Cache keys result.
    Keys(Vec<ServiceWorkerFetchRequest>),
    /// Boolean result.
    Bool(bool),
    /// CacheStorage keys result.
    StorageKeys(Vec<String>),
}

/// MessagePort endpoint metadata for one page/worker message.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServiceWorkerMessagePorts {
    /// Endpoint IDs transferred with this message.
    pub transferred_port_ids: Vec<u64>,
    /// Index of the transferred endpoint used as the payload.
    pub data_port_index: Option<usize>,
    /// Existing endpoint addressed by this message.
    pub target_port_id: Option<u64>,
}

/// Lifecycle state of a [`ServiceWorkerRuntime`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceWorkerRuntimeState {
    /// The engine thread is ready to evaluate scripts.
    Running,
    /// The engine thread has been shut down.
    Terminated,
}

/// Independent engine thread for Service Worker scripts.
///
/// Commands and events are typed so lifecycle coordination never relies on the
/// Dedicated Worker `postMessage(String)` adapter.
pub struct ServiceWorkerRuntime {
    core: ThreadedRuntimeCore<ServiceWorkerCommand, ServiceWorkerEvent>,
    import_response_sender: mpsc::Sender<ServiceWorkerImportResponse>,
    update_response_sender: mpsc::Sender<ServiceWorkerUpdateResponse>,
    clients_response_sender: mpsc::Sender<ServiceWorkerClientsResponse>,
    cache_storage_response_sender: mpsc::Sender<ServiceWorkerCacheStorageResponse>,
    fetch_response_sender: mpsc::Sender<ServiceWorkerFetchHostResponse>,
}

impl ServiceWorkerRuntime {
    /// Start a Service Worker engine thread and wait for engine initialization.
    pub fn new(config: SandboxConfig) -> Result<Self, ScriptError> {
        let config = normalize_config(config);
        let lifecycle_timeout_ms = config.timeout_ms;
        let (init_sender, init_receiver) = mpsc::sync_channel(1);
        let (import_response_sender, import_response_receiver) = mpsc::channel();
        let (update_response_sender, update_response_receiver) = mpsc::channel();
        let (clients_response_sender, clients_response_receiver) = mpsc::channel();
        let (cache_storage_response_sender, cache_storage_response_receiver) = mpsc::channel();
        let (fetch_response_sender, fetch_response_receiver) = mpsc::channel();
        let mut core = ThreadedRuntimeCore::spawn(
            "zero-service-worker",
            "Service Worker",
            move |command_receiver, event_sender, _terminate_flag| {
                let mut sandbox = match create_engine(config) {
                    Ok(sandbox) => sandbox,
                    Err(error) => {
                        let _ = init_sender.send(Err(error));
                        return;
                    }
                };
                sandbox.register_callback(
                    "__zwResolveURL",
                    Box::new(|args| {
                        let Some(input) = args.first().filter(|value| value.len() <= MAX_IMPORT_SCRIPT_URL_BYTES)
                        else {
                            return serde_json::json!({"ok": false, "error": "URL input is invalid"}).to_string();
                        };
                        let base = args.get(1).filter(|value| !value.is_empty());
                        let parsed = match base {
                            Some(base) if base.len() <= MAX_IMPORT_SCRIPT_URL_BYTES => {
                                url::Url::parse(base).and_then(|base| base.join(input))
                            }
                            Some(_) => {
                                return serde_json::json!({"ok": false, "error": "URL base is too long"}).to_string();
                            }
                            None => url::Url::parse(input),
                        };
                        match parsed {
                            Ok(url) => serde_json::json!({
                                "ok": true,
                                "href": url.as_str(),
                                "origin": url.origin().ascii_serialization(),
                                "protocol": format!("{}:", url.scheme()),
                                "host": match url.port() {
                                    Some(port) => format!("{}:{port}", url.host_str().unwrap_or_default()),
                                    None => url.host_str().unwrap_or_default().to_string(),
                                },
                                "hostname": url.host_str().unwrap_or_default(),
                                "port": url.port().map(|port| port.to_string()).unwrap_or_default(),
                                "pathname": url.path(),
                                "search": url.query().map(|query| format!("?{query}")).unwrap_or_default(),
                                "hash": url.fragment().map(|fragment| format!("#{fragment}")).unwrap_or_default(),
                            })
                            .to_string(),
                            Err(_) => serde_json::json!({"ok": false, "error": "Invalid URL"}).to_string(),
                        }
                    }),
                );
                let import_event_sender = event_sender.clone();
                let import_response_receiver = Arc::new(Mutex::new(import_response_receiver));
                let next_import_request_id = Arc::new(AtomicU64::new(1));
                let callback_response_receiver = Arc::clone(&import_response_receiver);
                let callback_next_request_id = Arc::clone(&next_import_request_id);
                sandbox.register_callback(
                    "__zwImportScripts",
                    Box::new(move |specifiers| {
                        if specifiers.len() > MAX_IMPORT_SCRIPTS_PER_CALL {
                            return import_failure_json("too many importScripts URLs");
                        }
                        if specifiers
                            .iter()
                            .any(|specifier| specifier.len() > MAX_IMPORT_SCRIPT_URL_BYTES)
                        {
                            return import_failure_json("importScripts URL exceeds the size limit");
                        }
                        let request_id = callback_next_request_id.fetch_add(1, Ordering::Relaxed);
                        if import_event_sender
                            .send(ServiceWorkerEvent::ImportScriptsRequested {
                                request_id,
                                specifiers: specifiers.to_vec(),
                            })
                            .is_err()
                        {
                            return import_failure_json("Service Worker host disconnected");
                        }
                        let deadline =
                            std::time::Instant::now() + std::time::Duration::from_millis(lifecycle_timeout_ms);
                        loop {
                            let now = std::time::Instant::now();
                            if now >= deadline {
                                return import_failure_json("importScripts host response timed out");
                            }
                            let response = callback_response_receiver
                                .lock()
                                .expect("import response lock")
                                .recv_timeout(deadline.saturating_duration_since(now));
                            match response {
                                Ok(ServiceWorkerImportResponse::Completed {
                                    request_id: response_id,
                                    sources,
                                }) if response_id == request_id => {
                                    let sources: Vec<_> = sources
                                        .iter()
                                        .map(|source| expose_classic_script_lexicals(source))
                                        .collect();
                                    return serde_json::json!({"ok": true, "sources": sources}).to_string();
                                }
                                Ok(ServiceWorkerImportResponse::Failed {
                                    request_id: response_id,
                                    message,
                                }) if response_id == request_id => return import_failure_json(&message),
                                Ok(ServiceWorkerImportResponse::Shutdown) => {
                                    return import_failure_json("Service Worker runtime is shutting down");
                                }
                                Ok(_) => continue,
                                Err(mpsc::RecvTimeoutError::Timeout) => {
                                    return import_failure_json("importScripts host response timed out");
                                }
                                Err(mpsc::RecvTimeoutError::Disconnected) => {
                                    return import_failure_json("Service Worker host disconnected");
                                }
                            }
                        }
                    }),
                );
                let update_event_sender = event_sender.clone();
                let update_response_receiver = Arc::new(Mutex::new(update_response_receiver));
                let next_update_request_id = Arc::new(AtomicU64::new(1));
                sandbox.register_callback(
                    "__zwRequestUpdate",
                    Box::new(move |_args| {
                        let request_id = next_update_request_id.fetch_add(1, Ordering::Relaxed);
                        if update_event_sender
                            .send(ServiceWorkerEvent::UpdateRequested { request_id })
                            .is_err()
                        {
                            return update_failure_json("InvalidStateError", "Service Worker host disconnected");
                        }
                        let deadline =
                            std::time::Instant::now() + std::time::Duration::from_millis(lifecycle_timeout_ms);
                        loop {
                            let now = std::time::Instant::now();
                            if now >= deadline {
                                return update_failure_json("TimeoutError", "Service Worker update timed out");
                            }
                            let response = update_response_receiver
                                .lock()
                                .expect("update response lock")
                                .recv_timeout(deadline.saturating_duration_since(now));
                            match response {
                                Ok(ServiceWorkerUpdateResponse::Completed {
                                    request_id: response_id,
                                }) if response_id == request_id => {
                                    return serde_json::json!({"ok": true}).to_string();
                                }
                                Ok(ServiceWorkerUpdateResponse::Failed {
                                    request_id: response_id,
                                    exception_name,
                                    message,
                                }) if response_id == request_id => {
                                    return update_failure_json(&exception_name, &message);
                                }
                                Ok(ServiceWorkerUpdateResponse::Shutdown) => {
                                    return update_failure_json(
                                        "InvalidStateError",
                                        "Service Worker runtime is shutting down",
                                    );
                                }
                                Ok(_) => continue,
                                Err(mpsc::RecvTimeoutError::Timeout) => {
                                    return update_failure_json("TimeoutError", "Service Worker update timed out");
                                }
                                Err(mpsc::RecvTimeoutError::Disconnected) => {
                                    return update_failure_json(
                                        "InvalidStateError",
                                        "Service Worker host disconnected",
                                    );
                                }
                            }
                        }
                    }),
                );
                let clients_event_sender = event_sender.clone();
                let clients_response_receiver = Arc::new(Mutex::new(clients_response_receiver));
                let next_clients_request_id = Arc::new(AtomicU64::new(1));
                let clients_get_event_sender = event_sender.clone();
                let clients_get_response_receiver = Arc::clone(&clients_response_receiver);
                let clients_get_request_id = Arc::clone(&next_clients_request_id);
                sandbox.register_callback(
                    "__zwClientsGet",
                    Box::new(move |args| {
                        let Some(client_id) = args.first().filter(|value| !value.is_empty()) else {
                            return clients_failure_json("Clients.get client id is invalid");
                        };
                        if client_id.len() > MAX_CLIENT_ID_BYTES {
                            return clients_failure_json("Clients.get client id exceeds the length limit");
                        }
                        let request_id = clients_get_request_id.fetch_add(1, Ordering::Relaxed);
                        if clients_get_event_sender
                            .send(ServiceWorkerEvent::ClientsGetRequested {
                                request_id,
                                client_id: client_id.clone(),
                            })
                            .is_err()
                        {
                            return clients_failure_json("Service Worker host disconnected");
                        }
                        let deadline =
                            std::time::Instant::now() + std::time::Duration::from_millis(lifecycle_timeout_ms);
                        loop {
                            let now = std::time::Instant::now();
                            if now >= deadline {
                                return clients_failure_json("Clients.get host response timed out");
                            }
                            let response = clients_get_response_receiver
                                .lock()
                                .expect("clients response lock")
                                .recv_timeout(deadline.saturating_duration_since(now));
                            match response {
                                Ok(ServiceWorkerClientsResponse::Completed {
                                    request_id: response_id,
                                    clients,
                                }) if response_id == request_id => {
                                    let client = clients.into_iter().next().map(|client| {
                                        serde_json::json!({
                                            "id": client.id,
                                            "url": client.url,
                                            "type": client.client_type,
                                            "frameType": client.frame_type,
                                            "visibilityState": client.visibility_state,
                                            "focused": client.focused,
                                        })
                                    });
                                    return serde_json::json!({"ok": true, "client": client}).to_string();
                                }
                                Ok(ServiceWorkerClientsResponse::Failed {
                                    request_id: response_id,
                                    message,
                                }) if response_id == request_id => return clients_failure_json(&message),
                                Ok(ServiceWorkerClientsResponse::Shutdown) => {
                                    return clients_failure_json("Service Worker runtime is shutting down");
                                }
                                Ok(_) => continue,
                                Err(mpsc::RecvTimeoutError::Timeout) => {
                                    return clients_failure_json("Clients.get host response timed out");
                                }
                                Err(mpsc::RecvTimeoutError::Disconnected) => {
                                    return clients_failure_json("Service Worker host disconnected");
                                }
                            }
                        }
                    }),
                );
                sandbox.register_callback(
                    "__zwClientsMatchAll",
                    Box::new(move |args| {
                        let include_uncontrolled = args.first().map(String::as_str) == Some("true");
                        let client_type = args.get(1).cloned().unwrap_or_else(|| "window".into());
                        let request_id = next_clients_request_id.fetch_add(1, Ordering::Relaxed);
                        if clients_event_sender
                            .send(ServiceWorkerEvent::ClientsMatchAllRequested {
                                request_id,
                                include_uncontrolled,
                                client_type,
                            })
                            .is_err()
                        {
                            return clients_failure_json("Service Worker host disconnected");
                        }
                        let deadline =
                            std::time::Instant::now() + std::time::Duration::from_millis(lifecycle_timeout_ms);
                        loop {
                            let now = std::time::Instant::now();
                            if now >= deadline {
                                return clients_failure_json("Clients.matchAll host response timed out");
                            }
                            let response = clients_response_receiver
                                .lock()
                                .expect("clients response lock")
                                .recv_timeout(deadline.saturating_duration_since(now));
                            match response {
                                Ok(ServiceWorkerClientsResponse::Completed {
                                    request_id: response_id,
                                    clients,
                                }) if response_id == request_id => {
                                    let clients = clients
                                        .into_iter()
                                        .map(|client| {
                                            serde_json::json!({
                                                "id": client.id,
                                                "url": client.url,
                                                "type": client.client_type,
                                                "frameType": client.frame_type,
                                                "visibilityState": client.visibility_state,
                                                "focused": client.focused,
                                            })
                                        })
                                        .collect::<Vec<_>>();
                                    return serde_json::json!({"ok": true, "clients": clients}).to_string();
                                }
                                Ok(ServiceWorkerClientsResponse::Failed {
                                    request_id: response_id,
                                    message,
                                }) if response_id == request_id => return clients_failure_json(&message),
                                Ok(ServiceWorkerClientsResponse::Shutdown) => {
                                    return clients_failure_json("Service Worker runtime is shutting down");
                                }
                                Ok(_) => continue,
                                Err(mpsc::RecvTimeoutError::Timeout) => {
                                    return clients_failure_json("Clients.matchAll host response timed out");
                                }
                                Err(mpsc::RecvTimeoutError::Disconnected) => {
                                    return clients_failure_json("Service Worker host disconnected");
                                }
                            }
                        }
                    }),
                );
                let cache_storage_event_sender = event_sender.clone();
                let cache_storage_response_receiver = Arc::new(Mutex::new(cache_storage_response_receiver));
                let next_cache_storage_request_id = Arc::new(AtomicU64::new(1));
                sandbox.register_callback(
                    "__zwCacheStorage",
                    Box::new(move |args| {
                        let Some(request_json) = args.first() else {
                            return cache_storage_failure_json("CacheStorage request is missing");
                        };
                        let request = match serde_json::from_str::<serde_json::Value>(request_json)
                            .ok()
                            .and_then(cache_storage_request_from_json)
                        {
                            Some(request) => request,
                            None => return cache_storage_failure_json("CacheStorage request is invalid"),
                        };
                        if let Err(error) = validate_cache_storage_request(&request) {
                            return cache_storage_failure_json(&error.to_string());
                        }
                        let request_id = next_cache_storage_request_id.fetch_add(1, Ordering::Relaxed);
                        if cache_storage_event_sender
                            .send(ServiceWorkerEvent::CacheStorageRequested { request_id, request })
                            .is_err()
                        {
                            return cache_storage_failure_json("Service Worker host disconnected");
                        }
                        let deadline =
                            std::time::Instant::now() + std::time::Duration::from_millis(lifecycle_timeout_ms);
                        loop {
                            let now = std::time::Instant::now();
                            if now >= deadline {
                                return cache_storage_failure_json("CacheStorage host response timed out");
                            }
                            let response = cache_storage_response_receiver
                                .lock()
                                .expect("cache storage response lock")
                                .recv_timeout(deadline.saturating_duration_since(now));
                            match response {
                                Ok(ServiceWorkerCacheStorageResponse::Completed {
                                    request_id: response_id,
                                    response,
                                }) if response_id == request_id => {
                                    return cache_storage_response_json(response);
                                }
                                Ok(ServiceWorkerCacheStorageResponse::Failed {
                                    request_id: response_id,
                                    message,
                                }) if response_id == request_id => return cache_storage_failure_json(&message),
                                Ok(ServiceWorkerCacheStorageResponse::Shutdown) => {
                                    return cache_storage_failure_json("Service Worker runtime is shutting down");
                                }
                                Ok(_) => continue,
                                Err(mpsc::RecvTimeoutError::Timeout) => {
                                    return cache_storage_failure_json("CacheStorage host response timed out");
                                }
                                Err(mpsc::RecvTimeoutError::Disconnected) => {
                                    return cache_storage_failure_json("Service Worker host disconnected");
                                }
                            }
                        }
                    }),
                );
                let fetch_event_sender = event_sender.clone();
                let fetch_response_receiver = Arc::new(Mutex::new(fetch_response_receiver));
                let next_fetch_request_id = Arc::new(AtomicU64::new(1));
                sandbox.register_callback(
                    "__zwFetch",
                    Box::new(move |args| {
                        let Some(request_json) = args.first() else {
                            return fetch_failure_json("Service Worker fetch request is missing");
                        };
                        let request = match serde_json::from_str::<serde_json::Value>(request_json)
                            .ok()
                            .and_then(|value| fetch_request_from_json(&value))
                        {
                            Some(request) => request,
                            None => return fetch_failure_json("Service Worker fetch request is invalid"),
                        };
                        if let Err(error) = validate_fetch_request(&request) {
                            return fetch_failure_json(&error.to_string());
                        }
                        let request_id = next_fetch_request_id.fetch_add(1, Ordering::Relaxed);
                        if fetch_event_sender
                            .send(ServiceWorkerEvent::FetchRequested { request_id, request })
                            .is_err()
                        {
                            return fetch_failure_json("Service Worker host disconnected");
                        }
                        let deadline =
                            std::time::Instant::now() + std::time::Duration::from_millis(lifecycle_timeout_ms);
                        loop {
                            let now = std::time::Instant::now();
                            if now >= deadline {
                                return fetch_failure_json("Service Worker fetch host response timed out");
                            }
                            let response = fetch_response_receiver
                                .lock()
                                .expect("fetch response lock")
                                .recv_timeout(deadline.saturating_duration_since(now));
                            match response {
                                Ok(ServiceWorkerFetchHostResponse::Completed {
                                    request_id: response_id,
                                    response,
                                }) if response_id == request_id => {
                                    return serde_json::json!({
                                        "ok": true,
                                        "response": fetch_response_json(response),
                                    })
                                    .to_string();
                                }
                                Ok(ServiceWorkerFetchHostResponse::Failed {
                                    request_id: response_id,
                                    message,
                                }) if response_id == request_id => return fetch_failure_json(&message),
                                Ok(ServiceWorkerFetchHostResponse::Shutdown) => {
                                    return fetch_failure_json("Service Worker runtime is shutting down");
                                }
                                Ok(_) => continue,
                                Err(mpsc::RecvTimeoutError::Timeout) => {
                                    return fetch_failure_json("Service Worker fetch host response timed out");
                                }
                                Err(mpsc::RecvTimeoutError::Disconnected) => {
                                    return fetch_failure_json("Service Worker host disconnected");
                                }
                            }
                        }
                    }),
                );
                if let Err(error) = sandbox.execute(SERVICE_WORKER_BOOTSTRAP) {
                    let _ = init_sender.send(Err(error));
                    return;
                }
                let _ = init_sender.send(Ok(()));

                let mut pending_lifecycle: Option<PendingLifecycle> = None;
                let mut pending_fetch: Option<PendingFetch> = None;
                loop {
                    if let Some(pending) = pending_lifecycle.as_ref()
                        && let Some(event) = poll_lifecycle(sandbox.as_mut(), pending, lifecycle_timeout_ms)
                    {
                        let _ = event_sender.send(event);
                        pending_lifecycle = None;
                    }
                    if (pending_lifecycle.is_some() || pending_fetch.is_some())
                        && let Ok(outbound) = take_outbound_messages(sandbox.as_mut())
                        && !outbound.is_empty()
                    {
                        let _ = event_sender.send(ServiceWorkerEvent::ClientMessagesEmitted { outbound });
                    }
                    if let Some(pending) = pending_fetch.as_ref()
                        && let Some(event) = poll_fetch(sandbox.as_mut(), pending, lifecycle_timeout_ms)
                    {
                        if let Ok(outbound) = take_outbound_messages(sandbox.as_mut())
                            && !outbound.is_empty()
                        {
                            let _ = event_sender.send(ServiceWorkerEvent::ClientMessagesEmitted { outbound });
                        }
                        let _ = event_sender.send(event);
                        pending_fetch = None;
                    }
                    let ran_idle_task = if pending_lifecycle.is_none() && pending_fetch.is_none() {
                        run_one_queued_task(sandbox.as_mut())
                    } else {
                        false
                    };
                    if let Ok(outbound) = take_outbound_messages(sandbox.as_mut())
                        && !outbound.is_empty()
                    {
                        let _ = event_sender.send(ServiceWorkerEvent::ClientMessagesEmitted { outbound });
                    }

                    let command = if pending_lifecycle.is_some() || pending_fetch.is_some() || ran_idle_task {
                        match command_receiver.recv_timeout(std::time::Duration::from_millis(1)) {
                            Ok(command) => command,
                            Err(mpsc::RecvTimeoutError::Timeout) => continue,
                            Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        }
                    } else {
                        match command_receiver.recv() {
                            Ok(command) => command,
                            Err(_) => break,
                        }
                    };
                    match command {
                        ServiceWorkerCommand::Evaluate {
                            script,
                            script_url,
                            is_module,
                        } => {
                            let source = if script.trim().is_empty() { ";" } else { script.as_str() };
                            let evaluation = set_worker_location(sandbox.as_mut(), &script_url).and_then(|()| {
                                if is_module {
                                    evaluate_module_graph(
                                        sandbox.as_mut(),
                                        source,
                                        &script_url,
                                        &event_sender,
                                        &import_response_receiver,
                                        &next_import_request_id,
                                        lifecycle_timeout_ms,
                                    )
                                } else {
                                    sandbox.execute(source).map(|_| ())
                                }
                            });
                            let event = match evaluation {
                                Ok(()) => {
                                    if let Ok(outbound) = take_outbound_messages(sandbox.as_mut())
                                        && !outbound.is_empty()
                                    {
                                        let _ =
                                            event_sender.send(ServiceWorkerEvent::ClientMessagesEmitted { outbound });
                                    }
                                    ServiceWorkerEvent::Evaluated { script_url }
                                }
                                Err(error) => ServiceWorkerEvent::ScriptError {
                                    script_url,
                                    kind: script_error_kind(&error),
                                    message: error.to_string(),
                                },
                            };
                            let _ = event_sender.send(event);
                        }
                        ServiceWorkerCommand::DispatchLifecycle { event_id, phase } => {
                            let result = if pending_lifecycle.is_some() {
                                Err(ScriptError::RuntimeError(
                                    "Service Worker lifecycle event is already pending".into(),
                                ))
                            } else {
                                begin_lifecycle(sandbox.as_mut(), event_id, phase)
                            };
                            match result {
                                Ok(()) => {
                                    pending_lifecycle = Some(PendingLifecycle {
                                        event_id,
                                        phase,
                                        deadline: std::time::Instant::now()
                                            + std::time::Duration::from_millis(lifecycle_timeout_ms),
                                    });
                                }
                                Err(error) => {
                                    let _ =
                                        event_sender.send(failed_lifecycle_event(event_id, phase, error.to_string()));
                                }
                            }
                        }
                        ServiceWorkerCommand::DispatchMessage {
                            event_id,
                            data_json,
                            client_id,
                            client_url,
                            ports,
                        } => {
                            let dispatch = format!(
                                "globalThis.__zwDispatchMessage({}, {}, {}, {}, {}, {}, {});",
                                event_id,
                                data_json,
                                serde_json::to_string(&client_id).unwrap(),
                                serde_json::to_string(&client_url).unwrap(),
                                serde_json::to_string(&ports.transferred_port_ids).unwrap(),
                                serde_json::to_string(&ports.data_port_index).unwrap(),
                                serde_json::to_string(&ports.target_port_id).unwrap(),
                            );
                            let event = match sandbox.execute(&dispatch) {
                                Ok(_) => match take_outbound_messages(sandbox.as_mut()) {
                                    Ok(outbound) => ServiceWorkerEvent::MessageDispatched {
                                        event_id,
                                        client_id,
                                        outbound,
                                    },
                                    Err(error) => ServiceWorkerEvent::MessageFailed {
                                        event_id,
                                        client_id,
                                        message: error.to_string(),
                                    },
                                },
                                Err(error) => ServiceWorkerEvent::MessageFailed {
                                    event_id,
                                    client_id,
                                    message: error.to_string(),
                                },
                            };
                            let _ = event_sender.send(event);
                        }
                        ServiceWorkerCommand::DispatchFetch { event_id, request } => {
                            let request_url = request.url.clone();
                            let result = if pending_fetch.is_some() {
                                Err(ScriptError::RuntimeError(
                                    "Service Worker fetch event is already pending".into(),
                                ))
                            } else {
                                begin_fetch(sandbox.as_mut(), event_id, &request)
                            };
                            match result {
                                Ok(()) => {
                                    pending_fetch = Some(PendingFetch {
                                        event_id,
                                        request_url,
                                        deadline: std::time::Instant::now()
                                            + std::time::Duration::from_millis(lifecycle_timeout_ms),
                                    });
                                }
                                Err(error) => {
                                    let _ =
                                        event_sender.send(failed_fetch_event(event_id, request_url, error.to_string()));
                                }
                            }
                        }
                        ServiceWorkerCommand::Shutdown => break,
                    }
                }
                let _ = event_sender.send(ServiceWorkerEvent::Closed);
            },
        )?;

        match init_receiver.recv_timeout(ENGINE_INIT_TIMEOUT) {
            Ok(Ok(())) => Ok(Self {
                core,
                import_response_sender,
                update_response_sender,
                clients_response_sender,
                cache_storage_response_sender,
                fetch_response_sender,
            }),
            Ok(Err(error)) => {
                core.terminate(ServiceWorkerCommand::Shutdown, || {});
                Err(error)
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                core.terminate(ServiceWorkerCommand::Shutdown, || {});
                Err(ScriptError::Timeout(
                    "Service Worker engine initialization timed out".into(),
                ))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                core.terminate(ServiceWorkerCommand::Shutdown, || {});
                Err(ScriptError::EngineUnavailable(
                    "Service Worker engine initialization channel closed".into(),
                ))
            }
        }
    }

    /// Queue a script for evaluation in the persistent Service Worker global.
    pub fn evaluate(&mut self, script: &str, script_url: &str) -> Result<(), ScriptError> {
        if self.core.is_terminated() {
            return Err(ScriptError::InvalidInput(
                "Cannot evaluate script on terminated Service Worker runtime".into(),
            ));
        }
        if script_url.trim().is_empty() {
            return Err(ScriptError::InvalidInput("Service Worker script URL is empty".into()));
        }
        self.core
            .send(ServiceWorkerCommand::Evaluate {
                script: script.to_string(),
                script_url: script_url.to_string(),
                is_module: false,
            })
            .map_err(|_| ScriptError::RuntimeError("Service Worker runtime disconnected".into()))
    }

    /// Queue a JavaScript module graph for evaluation in the persistent Service Worker global.
    pub fn evaluate_module(&mut self, script: &str, script_url: &str) -> Result<(), ScriptError> {
        if self.core.is_terminated() {
            return Err(ScriptError::InvalidInput(
                "Cannot evaluate script on terminated Service Worker runtime".into(),
            ));
        }
        if script_url.trim().is_empty() {
            return Err(ScriptError::InvalidInput("Service Worker script URL is empty".into()));
        }
        self.core
            .send(ServiceWorkerCommand::Evaluate {
                script: script.to_string(),
                script_url: script_url.to_string(),
                is_module: true,
            })
            .map_err(|_| ScriptError::RuntimeError("Service Worker runtime disconnected".into()))
    }

    /// Dispatch an install event.
    pub fn dispatch_install(&mut self, event_id: u64) -> Result<(), ScriptError> {
        self.dispatch_lifecycle(event_id, ServiceWorkerLifecyclePhase::Install)
    }

    /// Dispatch an activate event.
    pub fn dispatch_activate(&mut self, event_id: u64) -> Result<(), ScriptError> {
        self.dispatch_lifecycle(event_id, ServiceWorkerLifecyclePhase::Activate)
    }

    /// Dispatch one JSON-compatible page message.
    pub fn dispatch_message(
        &mut self,
        event_id: u64,
        data_json: &str,
        client_id: &str,
        client_url: &str,
    ) -> Result<(), ScriptError> {
        self.dispatch_message_with_ports(
            event_id,
            data_json,
            client_id,
            client_url,
            &ServiceWorkerMessagePorts::default(),
        )
    }

    /// Dispatch a page message with transferred or addressed MessagePort endpoints.
    pub fn dispatch_message_with_ports(
        &mut self,
        event_id: u64,
        data_json: &str,
        client_id: &str,
        client_url: &str,
        ports: &ServiceWorkerMessagePorts,
    ) -> Result<(), ScriptError> {
        serde_json::from_str::<serde_json::Value>(data_json)
            .map_err(|error| ScriptError::InvalidInput(format!("invalid Service Worker message JSON: {error}")))?;
        if ports.transferred_port_ids.len() > MAX_MESSAGE_PORTS
            || ports.transferred_port_ids.contains(&0)
            || ports.transferred_port_ids.iter().collect::<HashSet<_>>().len() != ports.transferred_port_ids.len()
        {
            return Err(ScriptError::InvalidInput(
                "invalid Service Worker transferred MessagePort list".into(),
            ));
        }
        if ports
            .data_port_index
            .is_some_and(|index| index >= ports.transferred_port_ids.len())
            || ports.target_port_id == Some(0)
        {
            return Err(ScriptError::InvalidInput(
                "invalid Service Worker MessagePort routing metadata".into(),
            ));
        }
        self.core
            .send(ServiceWorkerCommand::DispatchMessage {
                event_id,
                data_json: data_json.to_string(),
                client_id: client_id.to_string(),
                client_url: client_url.to_string(),
                ports: ports.clone(),
            })
            .map_err(|_| ScriptError::RuntimeError("Service Worker runtime disconnected".into()))
    }

    /// Dispatch one fetch event into the persistent Service Worker global.
    pub fn dispatch_fetch(&mut self, event_id: u64, request: ServiceWorkerFetchRequest) -> Result<(), ScriptError> {
        validate_fetch_request(&request)?;
        if self.core.is_terminated() {
            return Err(ScriptError::InvalidInput(
                "Cannot dispatch fetch on terminated Service Worker runtime".into(),
            ));
        }
        self.core
            .send(ServiceWorkerCommand::DispatchFetch { event_id, request })
            .map_err(|_| ScriptError::RuntimeError("Service Worker runtime disconnected".into()))
    }

    /// Try to receive one runtime event without blocking.
    pub fn try_recv(&self) -> Option<ServiceWorkerEvent> {
        self.core.try_recv()
    }

    /// Wait for one runtime event.
    pub fn recv(&self) -> Result<ServiceWorkerEvent, ScriptError> {
        self.core
            .recv()
            .map_err(|_| ScriptError::RuntimeError("Service Worker runtime channel closed".into()))
    }

    /// Wait up to `timeout` for one runtime event.
    pub fn recv_timeout(&self, timeout: std::time::Duration) -> Result<ServiceWorkerEvent, ScriptError> {
        self.core.recv_timeout(timeout).map_err(|error| match error {
            mpsc::RecvTimeoutError::Timeout => ScriptError::Timeout("Service Worker runtime receive timed out".into()),
            mpsc::RecvTimeoutError::Disconnected => {
                ScriptError::RuntimeError("Service Worker runtime channel closed".into())
            }
        })
    }

    /// Complete one blocking `importScripts()` host request.
    pub fn complete_import_scripts(
        &self,
        request_id: u64,
        result: Result<Vec<String>, String>,
    ) -> Result<(), ScriptError> {
        let response = match result {
            Ok(sources) => {
                if sources.len() > MAX_IMPORT_SCRIPTS_PER_CALL {
                    return Err(ScriptError::InvalidInput(
                        "too many imported Service Worker scripts".into(),
                    ));
                }
                let total_bytes = sources.iter().try_fold(0usize, |total, source| {
                    if source.len() > MAX_IMPORTED_SCRIPT_BYTES {
                        return None;
                    }
                    total.checked_add(source.len())
                });
                if total_bytes.is_none_or(|bytes| bytes > MAX_IMPORTED_SCRIPT_BYTES) {
                    return Err(ScriptError::InvalidInput(
                        "imported Service Worker scripts exceed the size limit".into(),
                    ));
                }
                ServiceWorkerImportResponse::Completed { request_id, sources }
            }
            Err(message) => ServiceWorkerImportResponse::Failed { request_id, message },
        };
        self.import_response_sender
            .send(response)
            .map_err(|_| ScriptError::RuntimeError("Service Worker runtime disconnected".into()))
    }

    /// Complete one blocking worker-global `registration.update()` request.
    pub fn complete_update(&self, request_id: u64, result: Result<(), (String, String)>) -> Result<(), ScriptError> {
        let response = match result {
            Ok(()) => ServiceWorkerUpdateResponse::Completed { request_id },
            Err((exception_name, message)) => ServiceWorkerUpdateResponse::Failed {
                request_id,
                exception_name,
                message,
            },
        };
        self.update_response_sender
            .send(response)
            .map_err(|_| ScriptError::RuntimeError("Service Worker runtime disconnected".into()))
    }

    /// Complete one blocking worker-global `clients.matchAll()` request.
    pub fn complete_clients_match_all(
        &self,
        request_id: u64,
        result: Result<Vec<ServiceWorkerClientInfo>, String>,
    ) -> Result<(), ScriptError> {
        let response = match result {
            Ok(clients) if clients.len() <= MAX_SERVICE_WORKER_CLIENTS => {
                ServiceWorkerClientsResponse::Completed { request_id, clients }
            }
            Ok(_) => ServiceWorkerClientsResponse::Failed {
                request_id,
                message: "Service Worker client result exceeds the size limit".into(),
            },
            Err(message) => ServiceWorkerClientsResponse::Failed { request_id, message },
        };
        self.clients_response_sender
            .send(response)
            .map_err(|_| ScriptError::RuntimeError("Service Worker runtime disconnected".into()))
    }

    /// Complete one blocking worker-global `clients.get()` request.
    pub fn complete_clients_get(
        &self,
        request_id: u64,
        result: Result<Option<ServiceWorkerClientInfo>, String>,
    ) -> Result<(), ScriptError> {
        let response = match result {
            Ok(Some(client)) => ServiceWorkerClientsResponse::Completed {
                request_id,
                clients: vec![client],
            },
            Ok(None) => ServiceWorkerClientsResponse::Completed {
                request_id,
                clients: Vec::new(),
            },
            Err(message) => ServiceWorkerClientsResponse::Failed { request_id, message },
        };
        self.clients_response_sender
            .send(response)
            .map_err(|_| ScriptError::RuntimeError("Service Worker runtime disconnected".into()))
    }

    /// Complete one blocking worker-global CacheStorage request.
    pub fn complete_cache_storage(
        &self,
        request_id: u64,
        result: Result<ServiceWorkerCacheStorageResult, String>,
    ) -> Result<(), ScriptError> {
        let response = match result {
            Ok(ServiceWorkerCacheStorageResult::Match(Some(response))) => {
                validate_cache_response(&response)?;
                ServiceWorkerCacheStorageResponse::Completed {
                    request_id,
                    response: ServiceWorkerCacheStorageResult::Match(Some(response)),
                }
            }
            Ok(ServiceWorkerCacheStorageResult::MatchAll(responses)) => {
                if responses.len() > MAX_CACHE_RESULTS {
                    return Err(ScriptError::InvalidInput(
                        "Service Worker cache response list exceeds the size limit".into(),
                    ));
                }
                for response in &responses {
                    validate_cache_response(response)?;
                }
                ServiceWorkerCacheStorageResponse::Completed {
                    request_id,
                    response: ServiceWorkerCacheStorageResult::MatchAll(responses),
                }
            }
            Ok(ServiceWorkerCacheStorageResult::Keys(requests)) => {
                if requests.len() > MAX_CACHE_RESULTS {
                    return Err(ScriptError::InvalidInput(
                        "Service Worker cache request list exceeds the size limit".into(),
                    ));
                }
                for request in &requests {
                    validate_fetch_request(request)?;
                }
                ServiceWorkerCacheStorageResponse::Completed {
                    request_id,
                    response: ServiceWorkerCacheStorageResult::Keys(requests),
                }
            }
            Ok(ServiceWorkerCacheStorageResult::StorageKeys(cache_names)) => {
                if cache_names.len() > MAX_CACHE_RESULTS {
                    return Err(ScriptError::InvalidInput(
                        "Service Worker cache name list exceeds the size limit".into(),
                    ));
                }
                for cache_name in &cache_names {
                    validate_cache_name(cache_name)?;
                }
                ServiceWorkerCacheStorageResponse::Completed {
                    request_id,
                    response: ServiceWorkerCacheStorageResult::StorageKeys(cache_names),
                }
            }
            Ok(
                response @ (ServiceWorkerCacheStorageResult::Done
                | ServiceWorkerCacheStorageResult::Open { .. }
                | ServiceWorkerCacheStorageResult::Match(None)
                | ServiceWorkerCacheStorageResult::Bool(_)),
            ) => ServiceWorkerCacheStorageResponse::Completed { request_id, response },
            Err(message) => ServiceWorkerCacheStorageResponse::Failed { request_id, message },
        };
        self.cache_storage_response_sender
            .send(response)
            .map_err(|_| ScriptError::RuntimeError("Service Worker runtime disconnected".into()))
    }

    /// Complete one blocking worker-global `caches.match()` request.
    pub fn complete_cache_match(
        &self,
        request_id: u64,
        result: Result<Option<ServiceWorkerFetchResponse>, String>,
    ) -> Result<(), ScriptError> {
        self.complete_cache_storage(request_id, result.map(ServiceWorkerCacheStorageResult::Match))
    }

    /// Complete one blocking worker-global `fetch()` request.
    pub fn complete_fetch(
        &self,
        request_id: u64,
        result: Result<ServiceWorkerFetchResponse, String>,
    ) -> Result<(), ScriptError> {
        let response = match result {
            Ok(response) => {
                validate_fetch_response(&response)?;
                ServiceWorkerFetchHostResponse::Completed { request_id, response }
            }
            Err(message) => ServiceWorkerFetchHostResponse::Failed { request_id, message },
        };
        self.fetch_response_sender
            .send(response)
            .map_err(|_| ScriptError::RuntimeError("Service Worker runtime disconnected".into()))
    }

    /// Shut down the engine thread with a bounded join.
    pub fn shutdown(&mut self) {
        let _ = self.import_response_sender.send(ServiceWorkerImportResponse::Shutdown);
        let _ = self.update_response_sender.send(ServiceWorkerUpdateResponse::Shutdown);
        let _ = self
            .clients_response_sender
            .send(ServiceWorkerClientsResponse::Shutdown);
        let _ = self
            .cache_storage_response_sender
            .send(ServiceWorkerCacheStorageResponse::Shutdown);
        let _ = self
            .fetch_response_sender
            .send(ServiceWorkerFetchHostResponse::Shutdown);
        self.core.terminate(ServiceWorkerCommand::Shutdown, || {});
    }

    /// Return the current runtime state.
    pub fn state(&self) -> ServiceWorkerRuntimeState {
        if self.core.is_terminated() {
            ServiceWorkerRuntimeState::Terminated
        } else {
            ServiceWorkerRuntimeState::Running
        }
    }

    /// Return whether the engine thread accepts commands.
    pub fn is_running(&self) -> bool {
        !self.core.is_terminated()
    }

    fn dispatch_lifecycle(&mut self, event_id: u64, phase: ServiceWorkerLifecyclePhase) -> Result<(), ScriptError> {
        if self.core.is_terminated() {
            return Err(ScriptError::InvalidInput(
                "Cannot dispatch event on terminated Service Worker runtime".into(),
            ));
        }
        self.core
            .send(ServiceWorkerCommand::DispatchLifecycle { event_id, phase })
            .map_err(|_| ScriptError::RuntimeError("Service Worker runtime disconnected".into()))
    }
}

impl Drop for ServiceWorkerRuntime {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn import_failure_json(message: &str) -> String {
    serde_json::json!({"ok": false, "error": message}).to_string()
}

fn update_failure_json(exception_name: &str, message: &str) -> String {
    serde_json::json!({"ok": false, "name": exception_name, "message": message}).to_string()
}

fn clients_failure_json(message: &str) -> String {
    serde_json::json!({"ok": false, "error": message}).to_string()
}

fn cache_storage_failure_json(message: &str) -> String {
    serde_json::json!({"ok": false, "error": message}).to_string()
}

fn fetch_failure_json(message: &str) -> String {
    serde_json::json!({"ok": false, "error": message}).to_string()
}

fn cache_storage_request_from_json(value: serde_json::Value) -> Option<ServiceWorkerCacheStorageRequest> {
    match value["op"].as_str()? {
        "open" => Some(ServiceWorkerCacheStorageRequest::Open {
            cache_name: cache_name_from_json(&value, "name", "nameUnits")?,
        }),
        "match" => Some(ServiceWorkerCacheStorageRequest::Match {
            cache_name: match (value["cacheName"].as_str(), value["cacheNameUnits"].as_str()) {
                (None, None) => None,
                _ => Some(cache_name_from_json(&value, "cacheName", "cacheNameUnits")?),
            },
            cache_id: value["cacheId"].as_u64(),
            request: fetch_request_from_json(&value["request"])?,
            options: cache_query_options_from_json(value.get("options")),
        }),
        "matchAll" => Some(ServiceWorkerCacheStorageRequest::MatchAll {
            cache_name: cache_name_from_json(&value, "cacheName", "cacheNameUnits")?,
            cache_id: value["cacheId"].as_u64(),
            request: if value.get("request").is_some() {
                Some(fetch_request_from_json(&value["request"])?)
            } else {
                None
            },
            options: cache_query_options_from_json(value.get("options")),
        }),
        "keys" => Some(ServiceWorkerCacheStorageRequest::Keys {
            cache_name: cache_name_from_json(&value, "cacheName", "cacheNameUnits")?,
            cache_id: value["cacheId"].as_u64(),
            request: if value.get("request").is_some() {
                Some(fetch_request_from_json(&value["request"])?)
            } else {
                None
            },
            options: cache_query_options_from_json(value.get("options")),
        }),
        "delete" => Some(ServiceWorkerCacheStorageRequest::Delete {
            cache_name: cache_name_from_json(&value, "cacheName", "cacheNameUnits")?,
            cache_id: value["cacheId"].as_u64(),
            request: fetch_request_from_json(&value["request"])?,
            options: cache_query_options_from_json(value.get("options")),
        }),
        "put" => Some(ServiceWorkerCacheStorageRequest::Put {
            cache_name: cache_name_from_json(&value, "cacheName", "cacheNameUnits")?,
            cache_id: value["cacheId"].as_u64(),
            request: fetch_request_from_json(&value["request"])?,
            response: fetch_response_from_json(&value["response"])?,
        }),
        "has" => Some(ServiceWorkerCacheStorageRequest::StorageHas {
            cache_name: cache_name_from_json(&value, "name", "nameUnits")?,
        }),
        "storageDelete" => Some(ServiceWorkerCacheStorageRequest::StorageDelete {
            cache_name: cache_name_from_json(&value, "name", "nameUnits")?,
        }),
        "storageKeys" => Some(ServiceWorkerCacheStorageRequest::StorageKeys),
        _ => None,
    }
}

fn cache_query_options_from_json(value: Option<&serde_json::Value>) -> ServiceWorkerCacheQueryOptions {
    let Some(value) = value else {
        return ServiceWorkerCacheQueryOptions::default();
    };
    ServiceWorkerCacheQueryOptions {
        ignore_search: value["ignoreSearch"].as_bool().unwrap_or(false),
        ignore_method: value["ignoreMethod"].as_bool().unwrap_or(false),
        ignore_vary: value["ignoreVary"].as_bool().unwrap_or(false),
    }
}

fn fetch_request_from_json(value: &serde_json::Value) -> Option<ServiceWorkerFetchRequest> {
    let headers = value["headers"]
        .as_array()?
        .iter()
        .map(|entry| {
            let pair = entry.as_array()?;
            if pair.len() != 2 {
                return None;
            }
            Some((pair[0].as_str()?.to_string(), pair[1].as_str()?.to_string()))
        })
        .collect::<Option<Vec<_>>>()?;
    Some(ServiceWorkerFetchRequest {
        url: value["url"].as_str()?.to_string(),
        method: value["method"].as_str()?.to_string(),
        headers,
        body: value["body"].as_str().map(str::to_string),
        client_id: value["clientId"].as_str().map(str::to_string),
        resulting_client_id: value["resultingClientId"].as_str().map(str::to_string),
        referrer: value["referrer"].as_str().map(str::to_string),
    })
}

fn fetch_response_from_json(value: &serde_json::Value) -> Option<ServiceWorkerFetchResponse> {
    let headers = value["headers"]
        .as_array()?
        .iter()
        .map(|entry| {
            let pair = entry.as_array()?;
            if pair.len() != 2 {
                return None;
            }
            Some((pair[0].as_str()?.to_string(), pair[1].as_str()?.to_string()))
        })
        .collect::<Option<Vec<_>>>()?;
    let status = value["status"].as_u64().and_then(|status| u16::try_from(status).ok())?;
    Some(ServiceWorkerFetchResponse {
        status,
        status_text: value["statusText"].as_str().unwrap_or_default().to_string(),
        response_type: value["type"].as_str().unwrap_or("default").to_string(),
        headers,
        body: value["body"].as_str().unwrap_or_default().to_string(),
    })
}

fn fetch_response_json(response: ServiceWorkerFetchResponse) -> serde_json::Value {
    serde_json::json!({
        "status": response.status,
        "statusText": response.status_text,
        "type": response.response_type,
        "headers": response.headers,
        "body": response.body,
    })
}

fn cache_storage_response_json(response: ServiceWorkerCacheStorageResult) -> String {
    match response {
        ServiceWorkerCacheStorageResult::Done => serde_json::json!({"ok": true}).to_string(),
        ServiceWorkerCacheStorageResult::Open {
            cache_name,
            cache_name_units,
            cache_id,
        } => serde_json::json!({
            "ok": true,
            "cacheName": display_cache_name(&cache_name),
            "cacheNameUnits": cache_name_units,
            "cacheId": cache_id,
        })
        .to_string(),
        ServiceWorkerCacheStorageResult::Match(response) => serde_json::json!({
            "ok": true,
            "response": response.map(fetch_response_json),
        })
        .to_string(),
        ServiceWorkerCacheStorageResult::MatchAll(responses) => serde_json::json!({
            "ok": true,
            "responses": responses.into_iter().map(fetch_response_json).collect::<Vec<_>>(),
        })
        .to_string(),
        ServiceWorkerCacheStorageResult::Keys(requests) => serde_json::json!({
            "ok": true,
            "requests": requests.iter().map(fetch_request_json).collect::<Vec<_>>(),
        })
        .to_string(),
        ServiceWorkerCacheStorageResult::Bool(value) => serde_json::json!({
            "ok": true,
            "value": value,
        })
        .to_string(),
        ServiceWorkerCacheStorageResult::StorageKeys(cache_names) => serde_json::json!({
            "ok": true,
            "cacheNames": cache_names.iter().map(|name| display_cache_name(name)).collect::<Vec<_>>(),
            "cacheNameUnits": cache_names.iter().map(|name| encode_cache_name_units(name)).collect::<Vec<_>>(),
        })
        .to_string(),
    }
}

fn cache_name_from_json(value: &serde_json::Value, name_key: &str, units_key: &str) -> Option<String> {
    match (value[name_key].as_str(), value[units_key].as_str()) {
        (Some(name), _) => Some(name.to_string()),
        (_, Some(units)) => decode_cache_name_units(units),
        _ => None,
    }
}

fn encode_cache_name_units(name: &str) -> String {
    if let Some(units) = name.strip_prefix(CACHE_NAME_DOMSTRING_PREFIX) {
        return units.to_string();
    }
    let mut out = String::new();
    for unit in name.encode_utf16() {
        out.push_str(&format!("{unit:04x}"));
    }
    out
}

fn decode_cache_name_units(units: &str) -> Option<String> {
    if !units.as_bytes().as_chunks::<4>().1.is_empty() {
        return None;
    }
    if !units.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!("{CACHE_NAME_DOMSTRING_PREFIX}{units}"))
}

fn display_cache_name(name: &str) -> String {
    name.strip_prefix(CACHE_NAME_DOMSTRING_PREFIX)
        .map(decode_cache_name_units_lossy)
        .unwrap_or_else(|| name.to_string())
}

fn decode_cache_name_units_lossy(units: &str) -> String {
    let mut utf16 = Vec::new();
    for chunk in units.as_bytes().as_chunks::<4>().0 {
        let Ok(hex) = std::str::from_utf8(chunk) else {
            return String::new();
        };
        let Ok(unit) = u16::from_str_radix(hex, 16) else {
            return String::new();
        };
        utf16.push(unit);
    }
    std::char::decode_utf16(utf16)
        .map(|item| item.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect()
}

fn validate_cache_name(name: &str) -> Result<(), ScriptError> {
    if name.len() > MAX_CACHE_NAME_BYTES {
        return Err(ScriptError::InvalidInput(
            "Service Worker cache name exceeds the size limit".into(),
        ));
    }
    Ok(())
}

fn validate_cache_storage_request(request: &ServiceWorkerCacheStorageRequest) -> Result<(), ScriptError> {
    match request {
        ServiceWorkerCacheStorageRequest::Open { cache_name } => validate_cache_name(cache_name),
        ServiceWorkerCacheStorageRequest::Match {
            cache_name, request, ..
        } => {
            if let Some(cache_name) = cache_name {
                validate_cache_name(cache_name)?;
            }
            validate_fetch_request(request)
        }
        ServiceWorkerCacheStorageRequest::MatchAll {
            cache_name, request, ..
        } => {
            validate_cache_name(cache_name)?;
            if let Some(request) = request {
                validate_fetch_request(request)?;
            }
            Ok(())
        }
        ServiceWorkerCacheStorageRequest::Keys {
            cache_name, request, ..
        } => {
            validate_cache_name(cache_name)?;
            if let Some(request) = request {
                validate_fetch_request(request)?;
            }
            Ok(())
        }
        ServiceWorkerCacheStorageRequest::Delete {
            cache_name, request, ..
        } => {
            validate_cache_name(cache_name)?;
            validate_fetch_request(request)
        }
        ServiceWorkerCacheStorageRequest::Put {
            cache_name,
            request,
            response,
            ..
        } => {
            validate_cache_name(cache_name)?;
            validate_fetch_request(request)?;
            validate_cache_response(response)
        }
        ServiceWorkerCacheStorageRequest::StorageHas { cache_name }
        | ServiceWorkerCacheStorageRequest::StorageDelete { cache_name } => validate_cache_name(cache_name),
        ServiceWorkerCacheStorageRequest::StorageKeys => Ok(()),
    }
}

fn validate_fetch_request(request: &ServiceWorkerFetchRequest) -> Result<(), ScriptError> {
    if request.url.is_empty() || request.url.len() > MAX_IMPORT_SCRIPT_URL_BYTES {
        return Err(ScriptError::InvalidInput(
            "Service Worker fetch request URL is invalid".into(),
        ));
    }
    if request.method.is_empty() || request.method.len() > MAX_FETCH_METHOD_BYTES {
        return Err(ScriptError::InvalidInput(
            "Service Worker fetch request method is invalid".into(),
        ));
    }
    if request.headers.len() > MAX_FETCH_HEADERS {
        return Err(ScriptError::InvalidInput(
            "Service Worker fetch request has too many headers".into(),
        ));
    }
    for (name, value) in &request.headers {
        if name.len().saturating_add(value.len()) > MAX_FETCH_HEADER_BYTES {
            return Err(ScriptError::InvalidInput(
                "Service Worker fetch request header exceeds the size limit".into(),
            ));
        }
    }
    if request
        .body
        .as_ref()
        .is_some_and(|body| body.len() > MAX_FETCH_BODY_BYTES)
    {
        return Err(ScriptError::InvalidInput(
            "Service Worker fetch request body exceeds the size limit".into(),
        ));
    }
    if request
        .client_id
        .as_ref()
        .is_some_and(|client_id| client_id.len() > MAX_CLIENT_ID_BYTES)
        || request
            .resulting_client_id
            .as_ref()
            .is_some_and(|client_id| client_id.len() > MAX_CLIENT_ID_BYTES)
        || request
            .referrer
            .as_ref()
            .is_some_and(|referrer| referrer.len() > MAX_IMPORT_SCRIPT_URL_BYTES)
    {
        return Err(ScriptError::InvalidInput(
            "Service Worker fetch client id exceeds the size limit".into(),
        ));
    }
    Ok(())
}

fn validate_fetch_response(response: &ServiceWorkerFetchResponse) -> Result<(), ScriptError> {
    if response.status < 200 || response.status > 599 {
        return Err(ScriptError::InvalidInput(
            "Service Worker fetch response status is invalid".into(),
        ));
    }
    validate_fetch_response_fields(response)
}

fn validate_cache_response(response: &ServiceWorkerFetchResponse) -> Result<(), ScriptError> {
    if !(response.status == 0 || (200..=599).contains(&response.status)) {
        return Err(ScriptError::InvalidInput(
            "Service Worker cache response status is invalid".into(),
        ));
    }
    validate_fetch_response_fields(response)
}

fn validate_fetch_response_fields(response: &ServiceWorkerFetchResponse) -> Result<(), ScriptError> {
    if response.status_text.len() > MAX_FETCH_STATUS_TEXT_BYTES {
        return Err(ScriptError::InvalidInput(
            "Service Worker fetch response status text exceeds the size limit".into(),
        ));
    }
    if response.body.len() > MAX_FETCH_BODY_BYTES {
        return Err(ScriptError::InvalidInput(
            "Service Worker fetch response body exceeds the size limit".into(),
        ));
    }
    if response.headers.len() > MAX_FETCH_HEADERS {
        return Err(ScriptError::InvalidInput(
            "Service Worker fetch response has too many headers".into(),
        ));
    }
    for (name, value) in &response.headers {
        if name.len().saturating_add(value.len()) > MAX_FETCH_HEADER_BYTES {
            return Err(ScriptError::InvalidInput(
                "Service Worker fetch response header exceeds the size limit".into(),
            ));
        }
    }
    Ok(())
}

fn fetch_request_json(request: &ServiceWorkerFetchRequest) -> serde_json::Value {
    let is_navigation = request.resulting_client_id.is_some();
    serde_json::json!({
        "url": &request.url,
        "method": &request.method,
        "headers": &request.headers,
        "body": &request.body,
        "clientId": &request.client_id,
        "resultingClientId": &request.resulting_client_id,
        "mode": if is_navigation { "navigate" } else { "cors" },
        "credentials": if is_navigation { "include" } else { "same-origin" },
        "redirect": if is_navigation { "manual" } else { "follow" },
        "referrer": &request.referrer,
        "headerGuard": "immutable",
    })
}

fn begin_fetch(
    sandbox: &mut dyn Sandbox,
    event_id: u64,
    request: &ServiceWorkerFetchRequest,
) -> Result<(), ScriptError> {
    let request_json = fetch_request_json(request);
    let dispatch = format!(
        "globalThis.__zwDispatchFetch({}, {request_json}); 'dispatched';",
        event_id
    );
    sandbox.execute(&dispatch).map(|_| ())
}

impl std::fmt::Debug for ServiceWorkerRuntime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ServiceWorkerRuntime")
            .field("state", &self.state())
            .finish()
    }
}

#[cfg(feature = "v8")]
fn create_engine(config: SandboxConfig) -> Result<Box<dyn Sandbox>, ScriptError> {
    Ok(Box::new(crate::V8Sandbox::with_config(config)?))
}

#[cfg(all(feature = "quickjs", not(feature = "v8")))]
fn create_engine(config: SandboxConfig) -> Result<Box<dyn Sandbox>, ScriptError> {
    Ok(Box::new(crate::QuickJSSandbox::with_config(config)?))
}

// 无引擎构建（如 zero-browser 主进程）：类型可用，运行时创建降级为
// EngineUnavailable，调用方按 ScriptFailed { kind: EngineUnavailable } 处理。
#[cfg(not(any(feature = "v8", feature = "quickjs")))]
fn create_engine(_config: SandboxConfig) -> Result<Box<dyn Sandbox>, ScriptError> {
    Err(ScriptError::EngineUnavailable(
        "no JavaScript engine feature is enabled in this build".into(),
    ))
}

fn script_error_kind(error: &ScriptError) -> ServiceWorkerScriptErrorKind {
    match error {
        ScriptError::CompileError(_) => ServiceWorkerScriptErrorKind::Compile,
        ScriptError::RuntimeError(_) | ScriptError::NotInitialized => ServiceWorkerScriptErrorKind::Runtime,
        ScriptError::Timeout(_) => ServiceWorkerScriptErrorKind::Timeout,
        ScriptError::InvalidInput(_) => ServiceWorkerScriptErrorKind::InvalidInput,
        ScriptError::EngineUnavailable(_) => ServiceWorkerScriptErrorKind::EngineUnavailable,
    }
}

fn poll_fetch(sandbox: &mut dyn Sandbox, pending: &PendingFetch, timeout_ms: u64) -> Option<ServiceWorkerEvent> {
    let _ = sandbox.execute("globalThis.__zwRunOneTask && globalThis.__zwRunOneTask();");
    match sandbox.execute("JSON.stringify(globalThis.__zwFetchResult)") {
        Ok(result) => match serde_json::from_str::<serde_json::Value>(&result.value) {
            Ok(value) if value["settled"].as_bool() == Some(true) => {
                return Some(match parse_fetch_result(&value) {
                    Ok(response) => ServiceWorkerEvent::FetchSettled {
                        event_id: pending.event_id,
                        request_url: pending.request_url.clone(),
                        response,
                        failed: value["failed"].as_bool().unwrap_or(false),
                        message: value["message"].as_str().unwrap_or_default().to_string(),
                    },
                    Err(error) => failed_fetch_event(pending.event_id, pending.request_url.clone(), error.to_string()),
                });
            }
            Ok(_) => {}
            Err(error) => {
                return Some(failed_fetch_event(
                    pending.event_id,
                    pending.request_url.clone(),
                    format!("invalid fetch result: {error}"),
                ));
            }
        },
        Err(error) => {
            return Some(failed_fetch_event(
                pending.event_id,
                pending.request_url.clone(),
                error.to_string(),
            ));
        }
    }
    if std::time::Instant::now() >= pending.deadline {
        return Some(failed_fetch_event(
            pending.event_id,
            pending.request_url.clone(),
            format!("fetch event exceeded {timeout_ms}ms"),
        ));
    }
    None
}

fn run_one_queued_task(sandbox: &mut dyn Sandbox) -> bool {
    sandbox
        .execute("globalThis.__zwRunOneTask && globalThis.__zwRunOneTask() ? 'true' : 'false';")
        .map(|result| result.value == "true")
        .unwrap_or(false)
}

fn parse_fetch_result(value: &serde_json::Value) -> Result<Option<ServiceWorkerFetchResponse>, ScriptError> {
    if value["responded"].as_bool() != Some(true) {
        return Ok(None);
    }
    let response = &value["response"];
    if !response.is_object() {
        return Err(ScriptError::RuntimeError(
            "Service Worker fetch response is missing".into(),
        ));
    }
    let status = response["status"]
        .as_u64()
        .and_then(|status| u16::try_from(status).ok())
        .filter(|status| (200..=599).contains(status))
        .ok_or_else(|| ScriptError::RuntimeError("Service Worker fetch response status is invalid".into()))?;
    let status_text = response["statusText"].as_str().unwrap_or_default().to_string();
    if status_text.len() > MAX_FETCH_STATUS_TEXT_BYTES {
        return Err(ScriptError::InvalidInput(
            "Service Worker fetch response status text exceeds the size limit".into(),
        ));
    }
    let body = response["body"].as_str().unwrap_or_default().to_string();
    if body.len() > MAX_FETCH_BODY_BYTES {
        return Err(ScriptError::InvalidInput(
            "Service Worker fetch response body exceeds the size limit".into(),
        ));
    }
    let headers_value = response["headers"]
        .as_array()
        .ok_or_else(|| ScriptError::RuntimeError("Service Worker fetch response headers are invalid".into()))?;
    if headers_value.len() > MAX_FETCH_HEADERS {
        return Err(ScriptError::InvalidInput(
            "Service Worker fetch response has too many headers".into(),
        ));
    }
    let mut total_header_bytes = 0usize;
    let mut headers = Vec::with_capacity(headers_value.len());
    for header in headers_value {
        let pair = header
            .as_array()
            .filter(|pair| pair.len() == 2)
            .ok_or_else(|| ScriptError::RuntimeError("Service Worker fetch response header is invalid".into()))?;
        let name = pair[0]
            .as_str()
            .ok_or_else(|| ScriptError::RuntimeError("Service Worker fetch response header name is invalid".into()))?
            .to_string();
        let value = pair[1]
            .as_str()
            .ok_or_else(|| ScriptError::RuntimeError("Service Worker fetch response header value is invalid".into()))?
            .to_string();
        total_header_bytes = total_header_bytes
            .saturating_add(name.len())
            .saturating_add(value.len());
        if total_header_bytes > MAX_FETCH_HEADER_BYTES {
            return Err(ScriptError::InvalidInput(
                "Service Worker fetch response headers exceed the size limit".into(),
            ));
        }
        headers.push((name, value));
    }
    Ok(Some(ServiceWorkerFetchResponse {
        status,
        status_text,
        response_type: "default".into(),
        headers,
        body,
    }))
}

fn evaluate_module_graph(
    sandbox: &mut dyn Sandbox,
    source: &str,
    script_url: &str,
    event_sender: &mpsc::Sender<ServiceWorkerEvent>,
    response_receiver: &Arc<Mutex<mpsc::Receiver<ServiceWorkerImportResponse>>>,
    next_request_id: &AtomicU64,
    timeout_ms: u64,
) -> Result<(), ScriptError> {
    if !extract_dynamic_import_specifiers(source).is_empty() {
        return Err(ScriptError::CompileError(
            "dynamic import is unavailable in Service Worker modules".into(),
        ));
    }
    let mut main_url = url::Url::parse(script_url)
        .map_err(|error| ScriptError::InvalidInput(format!("invalid Service Worker module URL: {error}")))?;
    main_url.set_fragment(None);
    let main_url = main_url.to_string();
    let mut registry = ModuleRegistry::new();
    let mut visited = HashSet::from([main_url.clone()]);
    collect_module_graph(
        source,
        &main_url,
        &mut registry,
        &mut visited,
        event_sender,
        response_receiver,
        next_request_id,
        timeout_ms,
    )?;
    let compiled = compile_module_script(source, &main_url, &registry)?;
    sandbox.execute("globalThis.__zwModuleScriptMode = true;")?;
    sandbox.execute(&compiled).map(|_| ())
}

#[allow(clippy::too_many_arguments)]
fn collect_module_graph(
    source: &str,
    referrer_url: &str,
    registry: &mut ModuleRegistry,
    visited: &mut HashSet<String>,
    event_sender: &mpsc::Sender<ServiceWorkerEvent>,
    response_receiver: &Arc<Mutex<mpsc::Receiver<ServiceWorkerImportResponse>>>,
    next_request_id: &AtomicU64,
    timeout_ms: u64,
) -> Result<(), ScriptError> {
    let specifiers = extract_static_module_import_specifiers(source);
    if specifiers.is_empty() {
        return Ok(());
    }
    let base = url::Url::parse(referrer_url)
        .map_err(|error| ScriptError::InvalidInput(format!("invalid Service Worker module referrer: {error}")))?;
    let mut pending = Vec::new();
    for specifier in specifiers {
        let mut resolved = base.join(&specifier).map_err(|error| {
            ScriptError::CompileError(format!("invalid Service Worker module specifier {specifier}: {error}"))
        })?;
        if !matches!(resolved.scheme(), "http" | "https" | "data")
            || !resolved.username().is_empty()
            || resolved.password().is_some()
        {
            return Err(ScriptError::CompileError(format!(
                "disallowed Service Worker module URL: {resolved}"
            )));
        }
        resolved.set_fragment(None);
        let resolved = resolved.to_string();
        if visited.insert(resolved.clone()) {
            pending.push((specifier, resolved));
        }
    }
    if pending.is_empty() {
        return Ok(());
    }
    let request_id = next_request_id.fetch_add(1, Ordering::Relaxed);
    event_sender
        .send(ServiceWorkerEvent::ModuleScriptsRequested {
            request_id,
            referrer_url: referrer_url.to_string(),
            specifiers: pending.iter().map(|(specifier, _)| specifier.clone()).collect(),
        })
        .map_err(|_| ScriptError::RuntimeError("Service Worker host disconnected".into()))?;
    let sources = wait_for_import_response(request_id, response_receiver, timeout_ms)?;
    if sources.len() != pending.len() {
        return Err(ScriptError::RuntimeError(
            "Service Worker module response count mismatch".into(),
        ));
    }
    for ((_, url), dependency_source) in pending.into_iter().zip(sources) {
        if !extract_dynamic_import_specifiers(&dependency_source).is_empty() {
            return Err(ScriptError::CompileError(
                "dynamic import is unavailable in Service Worker modules".into(),
            ));
        }
        registry.register(&url, &dependency_source);
        collect_module_graph(
            &dependency_source,
            &url,
            registry,
            visited,
            event_sender,
            response_receiver,
            next_request_id,
            timeout_ms,
        )?;
    }
    Ok(())
}

fn wait_for_import_response(
    request_id: u64,
    response_receiver: &Arc<Mutex<mpsc::Receiver<ServiceWorkerImportResponse>>>,
    timeout_ms: u64,
) -> Result<Vec<String>, ScriptError> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
    loop {
        let now = std::time::Instant::now();
        if now >= deadline {
            return Err(ScriptError::Timeout("Service Worker module fetch timed out".into()));
        }
        let response = response_receiver
            .lock()
            .expect("import response lock")
            .recv_timeout(deadline.saturating_duration_since(now));
        match response {
            Ok(ServiceWorkerImportResponse::Completed {
                request_id: response_id,
                sources,
            }) if response_id == request_id => return Ok(sources),
            Ok(ServiceWorkerImportResponse::Failed {
                request_id: response_id,
                message,
            }) if response_id == request_id => return Err(ScriptError::RuntimeError(message)),
            Ok(ServiceWorkerImportResponse::Shutdown) => {
                return Err(ScriptError::RuntimeError(
                    "Service Worker runtime is shutting down".into(),
                ));
            }
            Ok(_) => {}
            Err(mpsc::RecvTimeoutError::Timeout) => {
                return Err(ScriptError::Timeout("Service Worker module fetch timed out".into()));
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(ScriptError::RuntimeError("Service Worker host disconnected".into()));
            }
        }
    }
}

fn set_worker_location(sandbox: &mut dyn Sandbox, script_url: &str) -> Result<(), ScriptError> {
    let url = url::Url::parse(script_url)
        .map_err(|error| ScriptError::InvalidInput(format!("invalid Service Worker script URL: {error}")))?;
    let location = serde_json::json!({
        "href": url.as_str(),
        "origin": url.origin().ascii_serialization(),
        "protocol": format!("{}:", url.scheme()),
        "host": match url.port() {
            Some(port) => format!("{}:{port}", url.host_str().unwrap_or_default()),
            None => url.host_str().unwrap_or_default().to_string(),
        },
        "hostname": url.host_str().unwrap_or_default(),
        "port": url.port().map_or_else(String::new, |port| port.to_string()),
        "pathname": url.path(),
        "search": url.query().map_or_else(String::new, |query| format!("?{query}")),
        "hash": url.fragment().map_or_else(String::new, |fragment| format!("#{fragment}")),
    });
    sandbox
        .execute(&format!("globalThis.__zwSetLocation({location});"))
        .map(|_| ())
}

fn begin_lifecycle(
    sandbox: &mut dyn Sandbox,
    event_id: u64,
    phase: ServiceWorkerLifecyclePhase,
) -> Result<(), ScriptError> {
    let dispatch = format!(
        "globalThis.__zwDispatchLifecycle({}, {}); 'dispatched';",
        serde_json::to_string(phase.as_str()).expect("static phase is serializable"),
        event_id
    );
    sandbox.execute(&dispatch).map(|_| ())
}

fn poll_lifecycle(
    sandbox: &mut dyn Sandbox,
    pending: &PendingLifecycle,
    timeout_ms: u64,
) -> Option<ServiceWorkerEvent> {
    match sandbox.execute("JSON.stringify(globalThis.__zwLifecycleResult)") {
        Ok(result) => match serde_json::from_str::<serde_json::Value>(&result.value) {
            Ok(value) if value["settled"].as_bool() == Some(true) => {
                return Some(ServiceWorkerEvent::LifecycleSettled {
                    event_id: pending.event_id,
                    phase: pending.phase,
                    succeeded: value["succeeded"].as_bool() == Some(true),
                    skip_waiting: value["skipWaitingRequested"].as_bool() == Some(true),
                    claim_clients: value["claimClientsRequested"].as_bool() == Some(true),
                    message: value["message"].as_str().unwrap_or_default().to_string(),
                });
            }
            Ok(_) => {}
            Err(error) => {
                return Some(failed_lifecycle_event(
                    pending.event_id,
                    pending.phase,
                    format!("invalid lifecycle result: {error}"),
                ));
            }
        },
        Err(error) => {
            return Some(failed_lifecycle_event(
                pending.event_id,
                pending.phase,
                error.to_string(),
            ));
        }
    }
    if std::time::Instant::now() >= pending.deadline {
        return Some(failed_lifecycle_event(
            pending.event_id,
            pending.phase,
            format!("lifecycle event exceeded {timeout_ms}ms"),
        ));
    }
    let _ = sandbox.execute("globalThis.__zwRunOneTask && globalThis.__zwRunOneTask();");
    None
}

fn failed_lifecycle_event(event_id: u64, phase: ServiceWorkerLifecyclePhase, message: String) -> ServiceWorkerEvent {
    ServiceWorkerEvent::LifecycleSettled {
        event_id,
        phase,
        succeeded: false,
        skip_waiting: false,
        claim_clients: false,
        message,
    }
}

fn failed_fetch_event(event_id: u64, request_url: String, message: String) -> ServiceWorkerEvent {
    ServiceWorkerEvent::FetchSettled {
        event_id,
        request_url,
        response: None,
        failed: true,
        message,
    }
}

fn take_outbound_messages(sandbox: &mut dyn Sandbox) -> Result<Vec<ServiceWorkerOutboundMessage>, ScriptError> {
    const MAX_OUTBOUND_MESSAGES: usize = 1024;
    const MAX_MESSAGE_BYTES: usize = 1024 * 1024;
    const MAX_OUTBOUND_BATCH_BYTES: usize = 16 * 1024 * 1024;
    let result = sandbox.execute("JSON.stringify(globalThis.__zwTakeOutboundMessages())")?;
    let values = serde_json::from_str::<Vec<serde_json::Value>>(&result.value)
        .map_err(|error| ScriptError::RuntimeError(format!("invalid outbound message list: {error}")))?;
    if values.len() > MAX_OUTBOUND_MESSAGES {
        return Err(ScriptError::InvalidInput(
            "Service Worker emitted too many messages in one event".into(),
        ));
    }
    let mut total_bytes = 0usize;
    values
        .into_iter()
        .map(|value| {
            let data_json = value["dataJSON"]
                .as_str()
                .ok_or_else(|| ScriptError::RuntimeError("outbound message data is missing".into()))?
                .to_string();
            serde_json::from_str::<serde_json::Value>(&data_json)
                .map_err(|error| ScriptError::RuntimeError(format!("invalid outbound message data: {error}")))?;
            if data_json.len() > MAX_MESSAGE_BYTES {
                return Err(ScriptError::InvalidInput(
                    "Service Worker outbound message exceeds the size limit".into(),
                ));
            }
            total_bytes = total_bytes.saturating_add(data_json.len());
            if total_bytes > MAX_OUTBOUND_BATCH_BYTES {
                return Err(ScriptError::InvalidInput(
                    "Service Worker outbound message batch exceeds the size limit".into(),
                ));
            }
            let port_id = value["portId"].as_u64();
            let transferred_port_ids = value["transferredPortIds"]
                .as_array()
                .map(|values| values.iter().filter_map(serde_json::Value::as_u64).collect::<Vec<_>>())
                .unwrap_or_default();
            let data_port_index = value["dataPortIndex"]
                .as_u64()
                .and_then(|index| usize::try_from(index).ok());
            let target_client_id = value["targetClientId"].as_str().map(str::to_string);
            if port_id == Some(0)
                || transferred_port_ids.len() > MAX_MESSAGE_PORTS
                || transferred_port_ids.contains(&0)
                || transferred_port_ids.iter().collect::<HashSet<_>>().len() != transferred_port_ids.len()
                || data_port_index.is_some_and(|index| index >= transferred_port_ids.len())
                || target_client_id
                    .as_ref()
                    .is_some_and(|id| id.is_empty() || id.len() > MAX_IMPORT_SCRIPT_URL_BYTES)
            {
                return Err(ScriptError::InvalidInput(
                    "invalid outbound Service Worker MessagePort metadata".into(),
                ));
            }
            Ok(ServiceWorkerOutboundMessage {
                data_json,
                port_id,
                transferred_port_ids,
                data_port_index,
                target_client_id,
            })
        })
        .collect()
}

fn normalize_config(mut config: SandboxConfig) -> SandboxConfig {
    config.persistent_context = true;
    config.heap_limit = match config.heap_limit {
        0 => MAX_HEAP_BYTES,
        configured => configured.min(MAX_HEAP_BYTES),
    };
    config.initial_heap_size = config.initial_heap_size.min(config.heap_limit);
    config.timeout_ms = match config.timeout_ms {
        0 => DEFAULT_SCRIPT_TIMEOUT_MS,
        configured => configured.min(DEFAULT_SCRIPT_TIMEOUT_MS),
    };
    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn test_config() -> SandboxConfig {
        SandboxConfig {
            timeout_ms: 200,
            ..Default::default()
        }
    }

    #[test]
    fn evaluate_reports_success_and_preserves_global() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate("globalThis.version = 1;", "https://example.test/sw.js")
            .unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated {
                script_url: "https://example.test/sw.js".into()
            }
        );

        runtime
            .evaluate(
                "if (globalThis.version !== 1) throw new Error('lost global');",
                "https://example.test/check.js",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn global_has_service_worker_scope_brand() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "if (!(self instanceof ServiceWorkerGlobalScope)) throw new Error('missing service worker brand');
                 if (!(self instanceof WorkerGlobalScope)) throw new Error('missing worker brand');",
                "https://example.test/sw.js",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn global_has_url_search_params() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "const params = new URLSearchParams('?key=first+value&key=second');
                 if (params.get('key') !== 'first value') throw new Error('wrong get');
                 if (params.getAll('key').join(',') !== 'first value,second') throw new Error('wrong getAll');
                 params.set('key', 'updated');
                 params.append('next', 'a value');
                 if (String(params) !== 'key=updated&next=a+value') throw new Error('wrong serialization');
                 if (Object.prototype.toString.call(params) !== '[object URLSearchParams]') {
                   throw new Error('wrong brand');
                 }",
                "https://example.test/sw.js",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn global_location_reflects_main_script_url() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "if (!(location instanceof WorkerLocation)) throw new Error('wrong location brand');
                 if (location.href !== 'https://example.test:8443/workers/sw.js?key=value') {
                   throw new Error('wrong href');
                 }
                 if (location.origin !== 'https://example.test:8443') throw new Error('wrong origin');
                 if (location.pathname !== '/workers/sw.js') throw new Error('wrong pathname');
                 if (location.search !== '?key=value') throw new Error('wrong search');
                 if (String(location) !== location.href) throw new Error('wrong string conversion');",
                "https://example.test:8443/workers/sw.js?key=value",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn import_scripts_requests_host_and_executes_sources_in_global_order() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "importScripts('/first.js', '/second.js');
                 if (globalThis.importOrder.join(',') !== 'first,second') throw new Error('wrong order');
                 if (globalThis.importedBinding !== 7) throw new Error('binding is not global');",
                "https://example.test/sw.js",
            )
            .unwrap();
        let ServiceWorkerEvent::ImportScriptsRequested { request_id, specifiers } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing importScripts request");
        };
        assert_eq!(specifiers, ["/first.js", "/second.js"]);
        runtime
            .complete_import_scripts(
                request_id,
                Ok(vec![
                    "globalThis.importOrder = ['first']; var importedBinding = 7;".into(),
                    "globalThis.importOrder.push('second');".into(),
                ]),
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn import_scripts_failure_rejects_top_level_evaluation() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate("importScripts('/missing.js');", "https://example.test/sw.js")
            .unwrap();
        let ServiceWorkerEvent::ImportScriptsRequested { request_id, .. } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing importScripts request");
        };
        runtime
            .complete_import_scripts(request_id, Err("HTTP 404".into()))
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::ScriptError {
                kind: ServiceWorkerScriptErrorKind::Runtime,
                message,
                ..
            } if message.contains("HTTP 404")
        ));
    }

    #[test]
    fn module_evaluation_fetches_transitive_static_graph() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate_module(
                "import { doubled } from './lib/entry.js';
                 if (doubled !== 14) throw new Error('wrong module value');",
                "https://example.test/workers/sw.js",
            )
            .unwrap();
        let ServiceWorkerEvent::ModuleScriptsRequested {
            request_id,
            referrer_url,
            specifiers,
        } = runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing root module request");
        };
        assert_eq!(referrer_url, "https://example.test/workers/sw.js");
        assert_eq!(specifiers, ["./lib/entry.js"]);
        runtime
            .complete_import_scripts(
                request_id,
                Ok(vec![
                    "import { value } from './value.js'; export const doubled = value * 2;".into(),
                ]),
            )
            .unwrap();

        let ServiceWorkerEvent::ModuleScriptsRequested {
            request_id,
            referrer_url,
            specifiers,
        } = runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing transitive module request");
        };
        assert_eq!(referrer_url, "https://example.test/workers/lib/entry.js");
        assert_eq!(specifiers, ["./value.js"]);
        runtime
            .complete_import_scripts(request_id, Ok(vec!["export const value = 7;".into()]))
            .unwrap();

        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn module_evaluation_rejects_dynamic_import() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate_module("import('./late.js');", "https://example.test/workers/sw.js")
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::ScriptError {
                kind: ServiceWorkerScriptErrorKind::Compile,
                message,
                ..
            } if message.contains("dynamic import")
        ));
    }

    #[test]
    fn module_evaluation_rejects_import_scripts() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate_module("importScripts('./classic.js');", "https://example.test/workers/sw.js")
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::ScriptError {
                kind: ServiceWorkerScriptErrorKind::Runtime,
                message,
                ..
            } if message.contains("importScripts")
        ));
    }

    #[test]
    fn module_evaluation_fetches_reexport_dependency() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate_module(
                "import { value } from './entry.js';
                 if (value !== 5) throw new Error('wrong re-exported value');",
                "https://example.test/workers/sw.js",
            )
            .unwrap();
        let ServiceWorkerEvent::ModuleScriptsRequested { request_id, .. } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing entry module request");
        };
        runtime
            .complete_import_scripts(request_id, Ok(vec!["export { value } from './value.js';".into()]))
            .unwrap();

        let ServiceWorkerEvent::ModuleScriptsRequested {
            request_id,
            referrer_url,
            specifiers,
        } = runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing re-export dependency request");
        };
        assert_eq!(referrer_url, "https://example.test/workers/entry.js");
        assert_eq!(specifiers, ["./value.js"]);
        runtime
            .complete_import_scripts(request_id, Ok(vec!["export const value = 5;".into()]))
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn import_scripts_fetch_failure_is_network_error_dom_exception() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "try {
                   importScripts('/missing.js');
                 } catch (error) {
                   if (error.name !== 'NetworkError') throw error;
                   if (!(error instanceof DOMException)) throw new Error('not a DOMException');
                   if (error.code !== 19 || DOMException.NETWORK_ERR !== 19) {
                     throw new Error('wrong NetworkError code');
                   }
                 }",
                "https://example.test/sw.js",
            )
            .unwrap();
        let ServiceWorkerEvent::ImportScriptsRequested { request_id, .. } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing importScripts request");
        };
        runtime
            .complete_import_scripts(request_id, Err("HTTP 404".into()))
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn evaluate_reports_compile_error() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime.evaluate("function(", "https://example.test/bad.js").unwrap();
        let event = runtime.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(
            matches!(
                event,
                ServiceWorkerEvent::ScriptError {
                    kind: ServiceWorkerScriptErrorKind::Compile,
                    ..
                }
            ),
            "unexpected syntax error event: {event:?}"
        );
    }

    #[test]
    fn evaluate_reports_runtime_error() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "throw new Error('evaluation failed'); /* secret-source-marker */",
                "https://example.test/throw.js",
            )
            .unwrap();
        let event = runtime.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(matches!(
            &event,
            ServiceWorkerEvent::ScriptError {
                kind: ServiceWorkerScriptErrorKind::Runtime,
                ..
            }
        ));
        assert!(!format!("{event:?}").contains("secret-source-marker"));
    }

    #[test]
    fn evaluate_timeout_recovers_for_next_script() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate("while (true) {}", "https://example.test/loop.js")
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::ScriptError {
                kind: ServiceWorkerScriptErrorKind::Timeout,
                ..
            }
        ));

        runtime
            .evaluate("globalThis.recovered = true;", "https://example.test/recovered.js")
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn empty_script_is_valid_but_empty_url_is_rejected() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime.evaluate("", "https://example.test/empty.js").unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
        assert!(matches!(
            runtime.evaluate("void 0", " "),
            Err(ScriptError::InvalidInput(_))
        ));
    }

    #[test]
    fn shutdown_is_idempotent_and_rejects_evaluation() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        assert_eq!(runtime.state(), ServiceWorkerRuntimeState::Running);
        runtime.shutdown();
        runtime.shutdown();
        assert_eq!(runtime.state(), ServiceWorkerRuntimeState::Terminated);
        assert!(!runtime.is_running());
        assert!(matches!(
            runtime.evaluate("void 0", "https://example.test/sw.js"),
            Err(ScriptError::InvalidInput(_))
        ));
    }

    #[test]
    fn config_enforces_persistent_context_and_resource_caps() {
        let config = normalize_config(SandboxConfig {
            heap_limit: usize::MAX,
            initial_heap_size: usize::MAX,
            timeout_ms: u64::MAX,
            persistent_context: false,
        });
        assert_eq!(config.heap_limit, MAX_HEAP_BYTES);
        assert_eq!(config.initial_heap_size, MAX_HEAP_BYTES);
        assert_eq!(config.timeout_ms, DEFAULT_SCRIPT_TIMEOUT_MS);
        assert!(config.persistent_context);

        let defaults = normalize_config(SandboxConfig::default());
        assert_eq!(defaults.heap_limit, MAX_HEAP_BYTES);
        assert_eq!(defaults.timeout_ms, DEFAULT_SCRIPT_TIMEOUT_MS);
    }

    #[test]
    fn install_event_waits_for_fulfilled_lifetime_promise() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('install', event => {
                    if (!(event instanceof InstallEvent)) throw new Error('wrong event');
                    event.waitUntil(Promise.resolve().then(() => {
                        globalThis.installFinished = true;
                    }));
                });",
                "https://example.test/sw.js",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));

        runtime.dispatch_install(11).unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::LifecycleSettled {
                event_id: 11,
                phase: ServiceWorkerLifecyclePhase::Install,
                succeeded: true,
                skip_waiting: false,
                claim_clients: false,
                message: String::new(),
            }
        );
        runtime
            .evaluate(
                "if (!globalThis.installFinished) throw new Error('not settled');",
                "https://example.test/check.js",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn install_event_reports_rejected_wait_until() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('install', event => {
                    event.waitUntil(Promise.reject(new Error('install rejected')));
                });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime.dispatch_install(12).unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::LifecycleSettled {
                event_id: 12,
                phase: ServiceWorkerLifecyclePhase::Install,
                succeeded: false,
                skip_waiting: false,
                claim_clients: false,
                ref message,
            } if message.contains("install rejected")
        ));
    }

    #[test]
    fn install_event_reports_skip_waiting_request() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('install', event => {
                    event.waitUntil(skipWaiting());
                });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime.dispatch_install(14).unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::LifecycleSettled {
                event_id: 14,
                phase: ServiceWorkerLifecyclePhase::Install,
                succeeded: true,
                skip_waiting: true,
                claim_clients: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn activate_event_reports_clients_claim_request() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('activate', event => {
                    event.waitUntil(clients.claim());
                });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime.dispatch_activate(15).unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::LifecycleSettled {
                event_id: 15,
                phase: ServiceWorkerLifecyclePhase::Activate,
                succeeded: true,
                skip_waiting: false,
                claim_clients: true,
                message: String::new(),
            }
        );
    }

    #[test]
    fn activate_event_dispatches_property_handler() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "globalThis.onactivate = event => {
                    if (event.type !== 'activate') throw new Error('wrong type');
                    event.waitUntil(Promise.resolve());
                };",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime.dispatch_activate(13).unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::LifecycleSettled {
                event_id: 13,
                phase: ServiceWorkerLifecyclePhase::Activate,
                succeeded: true,
                skip_waiting: false,
                claim_clients: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn page_message_dispatches_message_event_with_structured_data() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('message', event => {
                    if (!(event instanceof MessageEvent)) throw new Error('wrong event');
                    globalThis.messageValue = event.data.name + ':' + event.data.items[1];
                    if (!(event.source instanceof Client)) throw new Error('wrong source');
                    event.source.postMessage({
                        echo: event.data.name,
                        source: event.source.id + ':' + event.source.url
                    });
                    if (event.data.name === 'fail') throw new Error('message failed');
                });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_message(
                16,
                r#"{"name":"page","items":[1,2]}"#,
                "client-1",
                "https://example.test/page",
            )
            .unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::MessageDispatched {
                event_id: 16,
                client_id: "client-1".into(),
                outbound: vec![ServiceWorkerOutboundMessage {
                    data_json: r#"{"echo":"page","source":"client-1:https://example.test/page"}"#.into(),
                    port_id: None,
                    transferred_port_ids: Vec::new(),
                    data_port_index: None,
                    target_client_id: Some("client-1".into()),
                }],
            }
        );
        runtime
            .dispatch_message(
                17,
                r#"{"name":"fail","items":[1,2]}"#,
                "client-1",
                "https://example.test/page",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::MessageFailed {
                event_id: 17,
                client_id,
                message,
            } if client_id == "client-1" && message.contains("message failed")
        ));
        runtime
            .dispatch_message(
                18,
                r#"{"name":"next","items":[1,2]}"#,
                "client-1",
                "https://example.test/page",
            )
            .unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::MessageDispatched {
                event_id: 18,
                client_id: "client-1".into(),
                outbound: vec![ServiceWorkerOutboundMessage {
                    data_json: r#"{"echo":"next","source":"client-1:https://example.test/page"}"#.into(),
                    port_id: None,
                    transferred_port_ids: Vec::new(),
                    data_port_index: None,
                    target_client_id: Some("client-1".into()),
                }],
            }
        );
        runtime
            .evaluate(
                "if (globalThis.messageValue !== 'next:2') throw new Error('message lost');",
                "https://example.test/check.js",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
        assert!(matches!(
            runtime.dispatch_message(19, "{", "client-1", "https://example.test/page"),
            Err(ScriptError::InvalidInput(_))
        ));
    }

    #[test]
    fn worker_registration_update_round_trips_through_host() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('message', event => {
                   registration.update().then(
                     () => event.source.postMessage({success: true}),
                     error => event.source.postMessage({success: false, exception: error.name})
                   );
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_message(19, "null", "client-1", "https://example.test/page")
            .unwrap();
        let ServiceWorkerEvent::UpdateRequested { request_id } = runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing worker update request");
        };
        runtime.complete_update(request_id, Ok(())).unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::MessageDispatched {
                event_id: 19,
                client_id: "client-1".into(),
                outbound: vec![ServiceWorkerOutboundMessage {
                    data_json: r#"{"success":true}"#.into(),
                    port_id: None,
                    transferred_port_ids: Vec::new(),
                    data_port_index: None,
                    target_client_id: Some("client-1".into()),
                }],
            }
        );

        runtime
            .dispatch_message(20, "null", "client-1", "https://example.test/page")
            .unwrap();
        let ServiceWorkerEvent::UpdateRequested { request_id } = runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing worker update request");
        };
        runtime
            .complete_update(
                request_id,
                Err(("InvalidStateError".into(), "installing worker cannot update".into())),
            )
            .unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::MessageDispatched {
                event_id: 20,
                client_id: "client-1".into(),
                outbound: vec![ServiceWorkerOutboundMessage {
                    data_json: r#"{"success":false,"exception":"InvalidStateError"}"#.into(),
                    port_id: None,
                    transferred_port_ids: Vec::new(),
                    data_port_index: None,
                    target_client_id: Some("client-1".into()),
                }],
            }
        );
    }

    #[test]
    fn clients_match_all_during_evaluation_emits_targeted_message() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "clients.matchAll({includeUncontrolled: true}).then(clientList => {
                   clientList[0].postMessage({matched: clientList[0].url});
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let ServiceWorkerEvent::ClientsMatchAllRequested {
            request_id,
            include_uncontrolled,
            client_type,
        } = runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing clients.matchAll request");
        };
        assert!(include_uncontrolled);
        assert_eq!(client_type, "window");
        runtime
            .complete_clients_match_all(
                request_id,
                Ok(vec![ServiceWorkerClientInfo {
                    id: "client-1".into(),
                    url: "https://example.test/page".into(),
                    client_type: "window".into(),
                    frame_type: "top-level".into(),
                    visibility_state: "visible".into(),
                    focused: false,
                }]),
            )
            .unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::ClientMessagesEmitted {
                outbound: vec![ServiceWorkerOutboundMessage {
                    data_json: r#"{"matched":"https://example.test/page"}"#.into(),
                    port_id: None,
                    transferred_port_ids: Vec::new(),
                    data_port_index: None,
                    target_client_id: Some("client-1".into()),
                }],
            }
        );
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn clients_get_during_evaluation_emits_targeted_message() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "clients.get('client-1').then(client => {
                   client.postMessage({matched: client.url});
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let ServiceWorkerEvent::ClientsGetRequested { request_id, client_id } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing clients.get request");
        };
        assert_eq!(client_id, "client-1");
        runtime
            .complete_clients_get(
                request_id,
                Ok(Some(ServiceWorkerClientInfo {
                    id: "client-1".into(),
                    url: "https://example.test/page".into(),
                    client_type: "window".into(),
                    frame_type: "top-level".into(),
                    visibility_state: "visible".into(),
                    focused: false,
                })),
            )
            .unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::ClientMessagesEmitted {
                outbound: vec![ServiceWorkerOutboundMessage {
                    data_json: r#"{"matched":"https://example.test/page"}"#.into(),
                    port_id: None,
                    transferred_port_ids: Vec::new(),
                    data_port_index: None,
                    target_client_id: Some("client-1".into()),
                }],
            }
        );
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn clients_get_unknown_client_resolves_undefined() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "clients.get('').then(empty => {
                   if (empty !== undefined) throw new Error('expected empty id undefined');
                   return clients.get('missing');
                 }).then(client => {
                   if (client !== undefined) throw new Error('expected missing id undefined');
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let ServiceWorkerEvent::ClientsGetRequested { request_id, client_id } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing clients.get request");
        };
        assert_eq!(client_id, "missing");
        runtime.complete_clients_get(request_id, Ok(None)).unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn message_ports_transfer_bidirectionally() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('message', event => {
                   const requestPort = event.data;
                   requestPort.onmessage = command => {
                     const response = new MessageChannel();
                     requestPort.postMessage(response.port1, [response.port1]);
                     response.port2.postMessage({echo: command.data});
                   };
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_message_with_ports(
                30,
                "null",
                "client-1",
                "https://example.test/page",
                &ServiceWorkerMessagePorts {
                    transferred_port_ids: vec![2],
                    data_port_index: Some(0),
                    target_port_id: None,
                },
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::MessageDispatched {
                event_id: 30,
                outbound,
                ..
            } if outbound.is_empty()
        ));

        runtime
            .dispatch_message_with_ports(
                31,
                "\"ping\"",
                "client-1",
                "https://example.test/page",
                &ServiceWorkerMessagePorts {
                    target_port_id: Some(2),
                    ..Default::default()
                },
            )
            .unwrap();
        let ServiceWorkerEvent::MessageDispatched {
            event_id: 31, outbound, ..
        } = runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing MessagePort dispatch");
        };
        assert_eq!(outbound.len(), 2);
        assert_eq!(outbound[0].port_id, Some(2));
        assert_eq!(outbound[0].transferred_port_ids.len(), 1);
        assert_eq!(outbound[0].data_port_index, Some(0));
        assert_eq!(outbound[1].port_id, Some(outbound[0].transferred_port_ids[0]));
        assert_eq!(outbound[1].data_json, r#"{"echo":"ping"}"#);
    }

    #[test]
    fn message_dispatch_can_settle_pending_install() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "let finishInstall;
                 addEventListener('install', event => {
                   event.waitUntil(new Promise(resolve => { finishInstall = resolve; }));
                 });
                 addEventListener('message', event => {
                   finishInstall();
                   event.source.postMessage('install-finished');
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime.dispatch_install(20).unwrap();
        runtime
            .dispatch_message(21, "null", "client-1", "https://example.test/page")
            .unwrap();

        let mut message_dispatched = false;
        let mut install_settled = false;
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !message_dispatched || !install_settled {
            match runtime.recv_timeout(deadline.saturating_duration_since(std::time::Instant::now())) {
                Ok(ServiceWorkerEvent::MessageDispatched {
                    event_id: 21, outbound, ..
                }) => {
                    assert_eq!(
                        outbound,
                        vec![ServiceWorkerOutboundMessage {
                            data_json: "\"install-finished\"".into(),
                            port_id: None,
                            transferred_port_ids: Vec::new(),
                            data_port_index: None,
                            target_client_id: Some("client-1".into()),
                        }]
                    );
                    message_dispatched = true;
                }
                Ok(ServiceWorkerEvent::LifecycleSettled {
                    event_id: 20,
                    phase: ServiceWorkerLifecyclePhase::Install,
                    succeeded: true,
                    ..
                }) => install_settled = true,
                Ok(other) => panic!("unexpected runtime event: {other:?}"),
                Err(error) => panic!("runtime event timed out: {error}"),
            }
        }
    }

    #[test]
    fn idle_timer_task_can_emit_client_message_without_pending_event() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "let reply = null;
                 addEventListener('message', event => {
                   reply = value => event.source.postMessage(value);
                   setTimeout(function() {
                     reply({type: 'complete', tests: [{name: 'delayed', status: 0}]});
                   }, 0);
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_message(22, "{}", "client-1", "https://example.test/page")
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::MessageDispatched { event_id: 22, .. }
        ));
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::ClientMessagesEmitted { outbound }
                if outbound == vec![ServiceWorkerOutboundMessage {
                    data_json: r#"{"type":"complete","tests":[{"name":"delayed","status":0}]}"#.into(),
                    port_id: None,
                    transferred_port_ids: Vec::new(),
                    data_port_index: None,
                    target_client_id: Some("client-1".into()),
                }]
        ));
    }

    #[test]
    fn page_message_allows_large_bounded_outbound_batch() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('message', event => {
                   for (let i = 0; i < 65; i++) event.source.postMessage({index: i});
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_message(20, "{}", "client-1", "https://example.test/page")
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::MessageDispatched { outbound, .. }
                if outbound.len() == 65
                    && outbound[0].data_json == r#"{"index":0}"#
                    && outbound[64].data_json == r#"{"index":64}"#
        ));
    }

    #[test]
    fn page_message_rejects_outbound_batch_above_count_limit() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('message', event => {
                   for (let i = 0; i < 1025; i++) event.source.postMessage(i);
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_message(21, "{}", "client-1", "https://example.test/page")
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::MessageFailed { message, .. }
                if message.contains("too many messages")
        ));
    }

    #[test]
    fn fetch_event_respond_with_serializes_response() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   if (!(event instanceof FetchEvent)) throw new Error('wrong event');
                   if (!(event.request instanceof Request)) throw new Error('wrong request');
                   if (event.request.url !== 'https://example.test/app/data.json') {
                     throw new Error('wrong url');
                   }
                   if (event.request.method !== 'POST') throw new Error('wrong method');
                   if (event.request.headers.get('x-test') !== 'yes') throw new Error('wrong header');
                   event.respondWith(new Response('intercepted:' + event.clientId, {
                     status: 202,
                     statusText: 'Accepted',
                     headers: {'X-SW': 'hit'}
                   }));
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                40,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/data.json".into(),
                    method: "POST".into(),
                    headers: vec![("X-Test".into(), "yes".into())],
                    body: Some("request-body".into()),
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                },
            )
            .unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 40,
                request_url: "https://example.test/app/data.json".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 202,
                    status_text: "Accepted".into(),
                    response_type: "default".into(),
                    headers: vec![("x-sw".into(), "hit".into())],
                    body: "intercepted:client-1".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn fetch_event_without_respond_with_passes_through() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   globalThis.lastFetchURL = event.request.url;
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                41,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/pass".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                },
            )
            .unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 41,
                request_url: "https://example.test/app/pass".into(),
                response: None,
                failed: false,
                message: String::new(),
            }
        );
        runtime
            .evaluate(
                "if (globalThis.lastFetchURL !== 'https://example.test/app/pass') throw new Error('fetch not seen');",
                "https://example.test/check.js",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn caches_match_resolves_into_fetch_response() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith(caches.match(event.request));
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                42,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/cached".into(),
                    method: "GET".into(),
                    headers: vec![("Accept".into(), "text/plain".into())],
                    body: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                },
            )
            .unwrap();
        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing CacheStorage.match request");
        };
        let ServiceWorkerCacheStorageRequest::Match {
            cache_name: None,
            request,
            options,
            ..
        } = request
        else {
            panic!("expected CacheStorage match request");
        };
        assert_eq!(request.url, "https://example.test/app/cached");
        assert_eq!(request.method, "GET");
        assert_eq!(request.headers, [("accept".into(), "text/plain".into())]);
        assert_eq!(options, ServiceWorkerCacheQueryOptions::default());
        runtime
            .complete_cache_match(
                request_id,
                Ok(Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: "OK".into(),
                    response_type: "default".into(),
                    headers: vec![("x-cache".into(), "hit".into())],
                    body: "cached-body".into(),
                })),
            )
            .unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 42,
                request_url: "https://example.test/app/cached".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: "OK".into(),
                    response_type: "default".into(),
                    headers: vec![("x-cache".into(), "hit".into())],
                    body: "cached-body".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn caches_open_put_match_roundtrips_cache_operations() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith((async () => {
                     const cache = await caches.open('runtime');
                     await cache.put(event.request, new Response('stored', {
                       status: 201,
                       statusText: 'Created',
                       headers: [['x-cache', 'put']]
                     }));
                     const responses = await cache.matchAll(event.request);
                     const requests = await cache.keys();
                     if (responses.length !== 1) throw new Error('matchAll length');
                     if (requests.length !== 1 || requests[0].method !== 'GET') throw new Error('keys length');
                     return responses[0];
                   })());
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                43,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/store".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                },
            )
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing CacheStorage.open request");
        };
        assert_eq!(
            request,
            ServiceWorkerCacheStorageRequest::Open {
                cache_name: "runtime".into()
            }
        );
        runtime
            .complete_cache_storage(
                request_id,
                Ok(ServiceWorkerCacheStorageResult::Open {
                    cache_name: "runtime".into(),
                    cache_name_units: "00720075006e00740069006d0065".into(),
                    cache_id: 7,
                }),
            )
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing Cache.put request");
        };
        let ServiceWorkerCacheStorageRequest::Put {
            cache_name,
            cache_id,
            request,
            response,
        } = request
        else {
            panic!("expected Cache.put request");
        };
        assert_eq!(cache_name, "runtime");
        assert_eq!(cache_id, Some(7));
        assert_eq!(request.url, "https://example.test/app/store");
        assert_eq!(response.status, 201);
        assert_eq!(response.status_text, "Created");
        assert_eq!(response.headers, [("x-cache".into(), "put".into())]);
        assert_eq!(response.body, "stored");
        runtime
            .complete_cache_storage(request_id, Ok(ServiceWorkerCacheStorageResult::Done))
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing Cache.matchAll request");
        };
        let ServiceWorkerCacheStorageRequest::MatchAll {
            cache_name,
            cache_id,
            request: Some(request),
            options,
        } = request
        else {
            panic!("expected named Cache.matchAll request");
        };
        assert_eq!(cache_name, "runtime");
        assert_eq!(cache_id, Some(7));
        assert_eq!(request.url, "https://example.test/app/store");
        assert_eq!(options, ServiceWorkerCacheQueryOptions::default());
        runtime
            .complete_cache_storage(
                request_id,
                Ok(ServiceWorkerCacheStorageResult::MatchAll(vec![
                    ServiceWorkerFetchResponse {
                        status: 201,
                        status_text: "Created".into(),
                        response_type: "default".into(),
                        headers: vec![("x-cache".into(), "put".into())],
                        body: "stored".into(),
                    },
                ])),
            )
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing Cache.keys request");
        };
        assert_eq!(
            request,
            ServiceWorkerCacheStorageRequest::Keys {
                cache_name: "runtime".into(),
                cache_id: Some(7),
                request: None,
                options: ServiceWorkerCacheQueryOptions::default(),
            }
        );
        runtime
            .complete_cache_storage(
                request_id,
                Ok(ServiceWorkerCacheStorageResult::Keys(vec![ServiceWorkerFetchRequest {
                    url: "https://example.test/app/store".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                }])),
            )
            .unwrap();

        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 43,
                request_url: "https://example.test/app/store".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 201,
                    status_text: "Created".into(),
                    response_type: "default".into(),
                    headers: vec![("x-cache".into(), "put".into())],
                    body: "stored".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn cache_put_sends_error_response_to_host_storage() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith(caches.open('runtime').then(function(cache) {
                     return cache.put(event.request, Response.error()).then(function() {
                       return cache.match(event.request).then(function(response) {
                         return new Response(String(response && response.type));
                       });
                     }, function(error) {
                       return new Response(String(error instanceof TypeError) + ':' + String(error.message));
                     });
                   }));
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                46,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/error".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                },
            )
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing CacheStorage.open request");
        };
        assert_eq!(
            request,
            ServiceWorkerCacheStorageRequest::Open {
                cache_name: "runtime".into()
            }
        );
        runtime
            .complete_cache_storage(
                request_id,
                Ok(ServiceWorkerCacheStorageResult::Open {
                    cache_name: "runtime".into(),
                    cache_name_units: "00720075006e00740069006d0065".into(),
                    cache_id: 8,
                }),
            )
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing CacheStorage.put request");
        };
        let ServiceWorkerCacheStorageRequest::Put {
            cache_name,
            cache_id,
            response,
            ..
        } = request
        else {
            panic!("unexpected CacheStorage request");
        };
        assert_eq!(cache_name, "runtime");
        assert_eq!(cache_id, Some(8));
        assert_eq!(response.status, 0);
        assert_eq!(response.response_type, "error");
        runtime
            .complete_cache_storage(request_id, Ok(ServiceWorkerCacheStorageResult::Done))
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing CacheStorage.match request");
        };
        assert_eq!(
            request,
            ServiceWorkerCacheStorageRequest::Match {
                cache_name: Some("runtime".into()),
                cache_id: Some(8),
                request: ServiceWorkerFetchRequest {
                    url: "https://example.test/app/error".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: Some(String::new()),
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                },
                options: Default::default(),
            }
        );
        runtime
            .complete_cache_storage(
                request_id,
                Ok(ServiceWorkerCacheStorageResult::Match(Some(
                    ServiceWorkerFetchResponse {
                        status: 0,
                        status_text: String::new(),
                        response_type: "error".into(),
                        headers: Vec::new(),
                        body: String::new(),
                    },
                ))),
            )
            .unwrap();

        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 46,
                request_url: "https://example.test/app/error".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "error".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn cache_query_options_roundtrip_from_worker_script() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith((async () => {
                     const cache = await caches.open('runtime');
                     await cache.match(event.request, {
                       ignoreSearch: true,
                       ignoreMethod: true,
                       ignoreVary: true
                     });
                     await caches.match(event.request, {
                       cacheName: 'runtime',
                       ignoreSearch: true,
                       ignoreMethod: true
                     });
                     await cache.matchAll(event.request, {ignoreSearch: true});
                     await cache.keys(event.request, {ignoreMethod: true});
                     return new Response('done');
                   })());
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                44,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/store?query=1".into(),
                    method: "POST".into(),
                    headers: Vec::new(),
                    body: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                },
            )
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing CacheStorage.open request");
        };
        assert_eq!(
            request,
            ServiceWorkerCacheStorageRequest::Open {
                cache_name: "runtime".into()
            }
        );
        runtime
            .complete_cache_storage(
                request_id,
                Ok(ServiceWorkerCacheStorageResult::Open {
                    cache_name: "runtime".into(),
                    cache_name_units: "00720075006e00740069006d0065".into(),
                    cache_id: 9,
                }),
            )
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing Cache.match request");
        };
        let ServiceWorkerCacheStorageRequest::Match {
            cache_name,
            cache_id,
            request,
            options,
        } = request
        else {
            panic!("expected Cache.match request");
        };
        assert_eq!(cache_name, Some("runtime".into()));
        assert_eq!(cache_id, Some(9));
        assert_eq!(request.url, "https://example.test/app/store?query=1");
        assert_eq!(request.method, "POST");
        assert_eq!(
            options,
            ServiceWorkerCacheQueryOptions {
                ignore_search: true,
                ignore_method: true,
                ignore_vary: true,
            }
        );
        runtime
            .complete_cache_storage(request_id, Ok(ServiceWorkerCacheStorageResult::Match(None)))
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing CacheStorage.match request");
        };
        let ServiceWorkerCacheStorageRequest::Match {
            cache_name,
            cache_id,
            request,
            options,
        } = request
        else {
            panic!("expected CacheStorage.match request");
        };
        assert_eq!(cache_name, Some("runtime".into()));
        assert_eq!(cache_id, None);
        assert_eq!(request.url, "https://example.test/app/store?query=1");
        assert_eq!(
            options,
            ServiceWorkerCacheQueryOptions {
                ignore_search: true,
                ignore_method: true,
                ignore_vary: false,
            }
        );
        runtime
            .complete_cache_storage(request_id, Ok(ServiceWorkerCacheStorageResult::Match(None)))
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing Cache.matchAll request");
        };
        let ServiceWorkerCacheStorageRequest::MatchAll {
            cache_name,
            cache_id,
            request: Some(request),
            options,
        } = request
        else {
            panic!("expected Cache.matchAll request");
        };
        assert_eq!(cache_name, "runtime");
        assert_eq!(cache_id, Some(9));
        assert_eq!(request.url, "https://example.test/app/store?query=1");
        assert_eq!(
            options,
            ServiceWorkerCacheQueryOptions {
                ignore_search: true,
                ignore_method: false,
                ignore_vary: false,
            }
        );
        runtime
            .complete_cache_storage(request_id, Ok(ServiceWorkerCacheStorageResult::MatchAll(Vec::new())))
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing Cache.keys request");
        };
        let ServiceWorkerCacheStorageRequest::Keys {
            cache_name,
            cache_id,
            request: Some(request),
            options,
        } = request
        else {
            panic!("expected Cache.keys request");
        };
        assert_eq!(cache_name, "runtime");
        assert_eq!(cache_id, Some(9));
        assert_eq!(request.url, "https://example.test/app/store?query=1");
        assert_eq!(
            options,
            ServiceWorkerCacheQueryOptions {
                ignore_search: false,
                ignore_method: true,
                ignore_vary: false,
            }
        );
        runtime
            .complete_cache_storage(request_id, Ok(ServiceWorkerCacheStorageResult::Keys(Vec::new())))
            .unwrap();

        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 44,
                request_url: "https://example.test/app/store?query=1".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "done".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn cache_delete_and_storage_listing_roundtrip_from_worker_script() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith((async () => {
                     const cache = await caches.open('runtime');
                     const before = await caches.has('runtime');
                     const names = await caches.keys();
                     const deleted = await cache.delete(event.request, {ignoreSearch: true});
                     const storageDeleted = await caches.delete('runtime');
                     return new Response([before, names.join(','), deleted, storageDeleted].join('|'));
                   })());
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                47,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/delete?version=1".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                },
            )
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing CacheStorage.open request");
        };
        assert_eq!(
            request,
            ServiceWorkerCacheStorageRequest::Open {
                cache_name: "runtime".into()
            }
        );
        runtime
            .complete_cache_storage(
                request_id,
                Ok(ServiceWorkerCacheStorageResult::Open {
                    cache_name: "runtime".into(),
                    cache_name_units: "00720075006e00740069006d0065".into(),
                    cache_id: 10,
                }),
            )
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing CacheStorage.has request");
        };
        assert_eq!(
            request,
            ServiceWorkerCacheStorageRequest::StorageHas {
                cache_name: "runtime".into()
            }
        );
        runtime
            .complete_cache_storage(request_id, Ok(ServiceWorkerCacheStorageResult::Bool(true)))
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing CacheStorage.keys request");
        };
        assert_eq!(request, ServiceWorkerCacheStorageRequest::StorageKeys);
        runtime
            .complete_cache_storage(
                request_id,
                Ok(ServiceWorkerCacheStorageResult::StorageKeys(vec!["runtime".into()])),
            )
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing Cache.delete request");
        };
        let ServiceWorkerCacheStorageRequest::Delete {
            cache_name,
            cache_id,
            request,
            options,
        } = request
        else {
            panic!("expected Cache.delete request");
        };
        assert_eq!(cache_name, "runtime");
        assert_eq!(cache_id, Some(10));
        assert_eq!(request.url, "https://example.test/app/delete?version=1");
        assert_eq!(
            options,
            ServiceWorkerCacheQueryOptions {
                ignore_search: true,
                ignore_method: false,
                ignore_vary: false,
            }
        );
        runtime
            .complete_cache_storage(request_id, Ok(ServiceWorkerCacheStorageResult::Bool(true)))
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing CacheStorage.delete request");
        };
        assert_eq!(
            request,
            ServiceWorkerCacheStorageRequest::StorageDelete {
                cache_name: "runtime".into()
            }
        );
        runtime
            .complete_cache_storage(request_id, Ok(ServiceWorkerCacheStorageResult::Bool(true)))
            .unwrap();

        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 47,
                request_url: "https://example.test/app/delete?version=1".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "true|runtime|true|true".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn worker_global_fetch_and_cache_add_roundtrip() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith((async () => {
                     const direct = await fetch('./direct.txt', {headers: {'X-Test': 'yes'}});
                     if ((await direct.text()) !== 'direct-body') throw new Error('direct fetch body');
                     const cache = await caches.open('runtime');
                     await cache.add('./add-a.txt');
                     await cache.addAll(['./add-b.txt']);
                     const one = await cache.match('https://example.test/app/add-a.txt');
                     const two = await cache.match('https://example.test/app/add-b.txt');
                     return new Response([one.headers.get('content-type'), await one.text(), await two.text()].join('|'));
                   })());
                 });",
                "https://example.test/app/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                45,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/page".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                },
            )
            .unwrap();

        let ServiceWorkerEvent::FetchRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing worker global fetch request");
        };
        assert_eq!(request.url, "https://example.test/app/direct.txt");
        assert_eq!(request.method, "GET");
        assert_eq!(request.headers, [("x-test".into(), "yes".into())]);
        runtime
            .complete_fetch(
                request_id,
                Ok(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: "OK".into(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "direct-body".into(),
                }),
            )
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing CacheStorage.open request");
        };
        assert_eq!(
            request,
            ServiceWorkerCacheStorageRequest::Open {
                cache_name: "runtime".into()
            }
        );
        runtime
            .complete_cache_storage(
                request_id,
                Ok(ServiceWorkerCacheStorageResult::Open {
                    cache_name: "runtime".into(),
                    cache_name_units: "00720075006e00740069006d0065".into(),
                    cache_id: 11,
                }),
            )
            .unwrap();

        for (expected_url, expected_body) in [
            ("https://example.test/app/add-a.txt", "alpha"),
            ("https://example.test/app/add-b.txt", "beta"),
        ] {
            let ServiceWorkerEvent::FetchRequested { request_id, request } =
                runtime.recv_timeout(Duration::from_secs(5)).unwrap()
            else {
                panic!("missing Cache.add fetch request");
            };
            assert_eq!(request.url, expected_url);
            assert_eq!(request.method, "GET");
            runtime
                .complete_fetch(
                    request_id,
                    Ok(ServiceWorkerFetchResponse {
                        status: 200,
                        status_text: "OK".into(),
                        response_type: "default".into(),
                        headers: vec![("content-type".into(), "text/plain".into())],
                        body: expected_body.into(),
                    }),
                )
                .unwrap();

            let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
                runtime.recv_timeout(Duration::from_secs(5)).unwrap()
            else {
                panic!("missing Cache.add put request");
            };
            let ServiceWorkerCacheStorageRequest::Put {
                cache_name,
                cache_id,
                request,
                response,
            } = request
            else {
                panic!("expected Cache.put request");
            };
            assert_eq!(cache_name, "runtime");
            assert_eq!(cache_id, Some(11));
            assert_eq!(request.url, expected_url);
            assert_eq!(response.status, 200);
            assert_eq!(response.body, expected_body);
            runtime
                .complete_cache_storage(request_id, Ok(ServiceWorkerCacheStorageResult::Done))
                .unwrap();
        }

        for expected_body in ["alpha", "beta"] {
            let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
                runtime.recv_timeout(Duration::from_secs(5)).unwrap()
            else {
                panic!("missing Cache.match request");
            };
            assert!(matches!(
                request,
                ServiceWorkerCacheStorageRequest::Match {
                    cache_name: Some(_),
                    cache_id: Some(11),
                    ..
                }
            ));
            runtime
                .complete_cache_storage(
                    request_id,
                    Ok(ServiceWorkerCacheStorageResult::Match(Some(
                        ServiceWorkerFetchResponse {
                            status: 200,
                            status_text: "OK".into(),
                            response_type: "default".into(),
                            headers: vec![("content-type".into(), "text/plain".into())],
                            body: expected_body.into(),
                        },
                    ))),
                )
                .unwrap();
        }

        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 45,
                request_url: "https://example.test/app/page".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "text/plain|alpha|beta".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn fetch_event_rejects_duplicate_respond_with() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith(new Response('first'));
                   event.respondWith(new Response('second'));
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                42,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/dup".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                },
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 42,
                response: None,
                message,
                ..
            } if message.contains("respondWith already called")
        ));
    }

    #[test]
    fn fetch_event_invalid_respond_with_value_fails_fetch() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith(new Object());
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                49,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/invalid".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                },
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 49,
                response: None,
                failed: true,
                message,
                ..
            } if message.contains("must resolve with a Response")
        ));
    }

    #[test]
    fn fetch_event_throw_after_respond_with_keeps_committed_response() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith(new Response('intercepted'));
                   throw new Error('after respondWith');
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                53,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/throw-after-respond".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                },
            )
            .unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 53,
                request_url: "https://example.test/app/throw-after-respond".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "intercepted".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn fetch_event_prevent_default_without_respond_with_fails_fetch() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.preventDefault();
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                51,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/prevent-default".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                },
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 51,
                response: None,
                failed: true,
                message,
                ..
            } if message.contains("prevented without respondWith")
        ));
    }

    #[test]
    fn fetch_event_used_fetched_response_body_fails_fetch() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith(fetch('./other.html').then(function(response) {
                     response.text();
                     return response;
                   }));
                 });",
                "https://example.test/app/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                52,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/page".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                },
            )
            .unwrap();
        let ServiceWorkerEvent::FetchRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing worker global fetch request");
        };
        assert_eq!(request.url, "https://example.test/app/other.html");
        runtime
            .complete_fetch(
                request_id,
                Ok(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: "OK".into(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "other".into(),
                }),
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 52,
                response: None,
                failed: true,
                message,
                ..
            } if message.contains("body has already been used")
        ));
    }

    #[test]
    fn fetch_event_allows_microtask_respond_with_but_rejects_task() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "let reportResult = null;
                 addEventListener('message', event => {
                   reportResult = result => event.source.postMessage(result);
                 });
                 function tryRespondWith(event) {
                   try {
                     event.respondWith(new Response('ok'));
                     reportResult({didThrow: false});
                   } catch (error) {
                     reportResult({didThrow: true, error: error.name});
                   }
                 }
                 addEventListener('fetch', event => {
                   if (event.request.url.endsWith('/task')) {
                     setTimeout(() => tryRespondWith(event), 0);
                   } else {
                     Promise.resolve().then(() => tryRespondWith(event));
                   }
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();
        runtime
            .dispatch_message(50, "null", "client-1", "https://example.test/app/page")
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::MessageDispatched { .. }
        ));

        runtime
            .dispatch_fetch(
                51,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/task".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                },
            )
            .unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::ClientMessagesEmitted {
                outbound: vec![ServiceWorkerOutboundMessage {
                    data_json: r#"{"didThrow":true,"error":"InvalidStateError"}"#.into(),
                    port_id: None,
                    transferred_port_ids: Vec::new(),
                    data_port_index: None,
                    target_client_id: Some("client-1".into()),
                }],
            }
        );
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 51,
                request_url: "https://example.test/app/task".into(),
                response: None,
                failed: false,
                message: String::new(),
            }
        );

        runtime
            .dispatch_fetch(
                52,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/microtask".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                },
            )
            .unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::ClientMessagesEmitted {
                outbound: vec![ServiceWorkerOutboundMessage {
                    data_json: r#"{"didThrow":false}"#.into(),
                    port_id: None,
                    transferred_port_ids: Vec::new(),
                    data_port_index: None,
                    target_client_id: Some("client-1".into()),
                }],
            }
        );
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 52,
                request_url: "https://example.test/app/microtask".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "ok".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }
}
