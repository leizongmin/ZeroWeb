//! Typed Service Worker script runtime.

use crate::threaded_runtime::ThreadedRuntimeCore;
use crate::{
    ModuleRegistry, Sandbox, SandboxConfig, ScriptError, compile_module_script, expose_classic_script_lexicals,
    extract_static_module_import_specifiers, rewrite_dynamic_imports,
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
const SERVICE_WORKER_DYNAMIC_IMPORT_PRELUDE: &str = "\
globalThis.__zw_dynamic_import = function() {\
  return Promise.reject(new TypeError('dynamic import is unavailable in Service Worker scripts'));\
};";

enum ServiceWorkerCommand {
    Evaluate {
        script: String,
        script_url: String,
        scope_url: String,
        initial_peers: ServiceWorkerRegistrationPeers,
        is_module: bool,
    },
    DispatchLifecycle {
        event_id: u64,
        phase: ServiceWorkerLifecyclePhase,
        clients_claim_allowed: bool,
    },
    DispatchMessage {
        event_id: u64,
        data_json: String,
        client_id: String,
        client_url: String,
        client_frame_type: String,
        client_focused: bool,
        ports: ServiceWorkerMessagePorts,
        clients_claim_allowed: bool,
    },
    DispatchWorkerMessage {
        event_id: u64,
        data_json: String,
        source: ServiceWorkerPeerInfo,
        ports: ServiceWorkerMessagePorts,
        clients_claim_allowed: bool,
    },
    SyncRegistrationPeers {
        registration_id: u64,
        peers: ServiceWorkerRegistrationPeers,
    },
    DispatchFetch {
        event_id: u64,
        request: ServiceWorkerFetchRequest,
        clients_claim_allowed: bool,
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

enum ServiceWorkerUnregisterResponse {
    Completed {
        request_id: u64,
        removed: bool,
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
  let clientsClaimAllowed = false;
  let activeEventClaimClientsRequested = false;
  const timerTasks = [];
  let nextTimerId = 1;

  class Event {
    constructor(type) {
      this.type = type;
      // https://dom.spec.whatwg.org/#concept-event-initialize
      this.bubbles = false;
      this.cancelable = false;
      this.defaultPrevented = false;
      this._propagationStopped = false;
      this._immediateStopped = false;
    }
    preventDefault() {
      if (this.cancelable) this.defaultPrevented = true;
    }
    stopPropagation() {
      this._propagationStopped = true;
    }
    stopImmediatePropagation() {
      this._immediateStopped = true;
      this._propagationStopped = true;
    }
  }
  class ExtendableEvent extends Event {
    constructor(type) {
      super(type);
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
  const workerMessages = [];
  const portEndpoints = Object.create(null);
  const serviceWorkersById = Object.create(null);
  let nextWorkerPortId = 1;
  const transferredPortMarker = '__zwServiceWorkerTransferredPortIndex';
  function cloneWithTransferredPortMarkers(value, ports, seen) {
    if (value === null || typeof value !== 'object') return value;
    const portIndex = ports.indexOf(value);
    if (portIndex >= 0) {
      const marker = {};
      marker[transferredPortMarker] = portIndex;
      return marker;
    }
    if (seen.has(value)) return seen.get(value);
    const out = Array.isArray(value) ? [] : {};
    seen.set(value, out);
    const keys = Object.keys(value);
    for (let i = 0; i < keys.length; i++) {
      out[keys[i]] = cloneWithTransferredPortMarkers(value[keys[i]], ports, seen);
    }
    return out;
  }
  function cloneMessage(data) {
    const dataJSON = JSON.stringify(data);
    if (dataJSON === undefined) throw new DOMException('message could not be cloned', 'DataCloneError');
    return dataJSON;
  }
  function cloneWithLocalTransferredPorts(value, ports, replacements, seen) {
    if (value === null || typeof value !== 'object') return value;
    const portIndex = ports.indexOf(value);
    if (portIndex >= 0) return replacements[portIndex];
    if (seen.has(value)) return seen.get(value);
    const out = Array.isArray(value) ? [] : Object.create(Object.getPrototypeOf(value));
    seen.set(value, out);
    const keys = Object.keys(value);
    for (let i = 0; i < keys.length; i++) {
      out[keys[i]] = cloneWithLocalTransferredPorts(value[keys[i]], ports, replacements, seen);
    }
    return out;
  }
  function prepareLocalPortTransfer(data, transfer) {
    const ports = transfer !== undefined && transfer !== null ? Array.from(transfer) : [];
    const replacements = [];
    for (const item of ports) {
      if (!(item instanceof MessagePort) || item._closed || item._detached || !item._other) {
        throw new DOMException('invalid MessagePort transfer', 'DataCloneError');
      }
      if (ports.indexOf(item) !== ports.lastIndexOf(item)) {
        throw new DOMException('duplicate MessagePort transfer', 'DataCloneError');
      }
      const counterpart = item._other;
      const replacement = new MessagePort();
      replacement._other = counterpart;
      counterpart._other = replacement;
      item._other = null;
      item._detached = true;
      replacements.push(replacement);
    }
    return {
      data: cloneWithLocalTransferredPorts(data, ports, replacements, new Map()),
      ports: replacements
    };
  }
  function preparePortTransfer(data, transfer, workerTransferTargetId) {
    const ids = [];
    let dataPortIndex = null;
    const ports = transfer !== undefined && transfer !== null ? Array.from(transfer) : [];
    if (transfer !== undefined && transfer !== null) {
      for (const item of ports) {
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
          item._other._workerRemote = workerTransferTargetId !== null && workerTransferTargetId !== undefined;
          item._other._workerRegistrationId =
            workerTransferTargetId === null || workerTransferTargetId === undefined ? '' : String(workerTransferTargetId);
          portEndpoints[String(portId)] = item._other;
        }
        if (data === item) dataPortIndex = ids.length;
        ids.push(portId);
        item._other = null;
        item._detached = true;
      }
    }
    if (ids.length > 0 && dataPortIndex === null) {
      data = cloneWithTransferredPortMarkers(data, ports, new Map());
    }
    return {
      dataJSON: dataPortIndex === null ? cloneMessage(data) : 'null',
      transferredPortIds: ids,
      dataPortIndex: dataPortIndex
    };
  }
  function queueOutbound(data, transfer, portId, targetClientId) {
    const wire = preparePortTransfer(data, transfer, null);
    wire.portId = portId;
    wire.targetClientId = targetClientId;
    outboundMessages.push(wire);
  }
  function queueWorkerMessage(targetRegistrationId, data, transfer) {
    const wire = preparePortTransfer(data, transfer, String(targetRegistrationId));
    wire.targetRegistrationId = String(targetRegistrationId);
    wire.targetPortId = null;
    wire.source = {
      id: currentServiceWorker._id || '',
      scriptURL: currentServiceWorker.scriptURL,
      state: currentServiceWorker.state
    };
    workerMessages.push(wire);
  }
  globalThis.queueMicrotask = globalThis.queueMicrotask || function(callback) {
    if (typeof callback !== 'function') {
      throw new TypeError('queueMicrotask callback must be callable');
    }
    Promise.resolve().then(callback);
  };
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
  // https://w3c.github.io/ServiceWorker/#windowclient-interface
  class WindowClient extends Client {}
  function clientFromInfo(info) {
    return info && info.type === 'window'
      ? new WindowClient(info, clientToken)
      : new Client(info, clientToken);
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
        if (this._workerRemote) {
          const wireTargetRegistrationId = this._workerRegistrationId === null || this._workerRegistrationId === undefined
            ? ''
            : String(this._workerRegistrationId);
          const wire = preparePortTransfer(data, transfer, wireTargetRegistrationId);
          wire.targetRegistrationId = wireTargetRegistrationId;
          wire.targetPortId = this._hostPortId;
          wire.source = {
            id: currentServiceWorker._id || '',
            scriptURL: currentServiceWorker.scriptURL,
            state: currentServiceWorker.state
          };
          workerMessages.push(wire);
        } else {
          queueOutbound(data, transfer, this._hostPortId, null);
        }
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
  function eventInit(init) {
    return init === undefined || init === null ? {} : Object(init);
  }
  function isMessageEventSource(value) {
    return value === null || value instanceof MessagePort || value instanceof Client || value instanceof ServiceWorker;
  }
  function messageEventInit(type, init) {
    init = eventInit(init);
    const event = {
      type: String(type),
      bubbles: Boolean(init.bubbles),
      cancelable: Boolean(init.cancelable),
      defaultPrevented: false,
      data: init.data === undefined ? null : init.data,
      origin: init.origin === undefined ? '' : String(init.origin),
      lastEventId: init.lastEventId === undefined ? '' : String(init.lastEventId),
      source: init.source === undefined ? null : init.source,
      ports: init.ports === undefined ? [] : init.ports
    };
    if (!isMessageEventSource(event.source)) {
      throw new TypeError('MessageEvent source must be Client, ServiceWorker, MessagePort, or null');
    }
    if (!Array.isArray(event.ports)) {
      throw new TypeError('MessageEvent ports must be an array');
    }
    for (let i = 0; i < event.ports.length; i++) {
      if (!(event.ports[i] instanceof MessagePort)) {
        throw new TypeError('MessageEvent ports entries must be MessagePort');
      }
    }
    return event;
  }
  class MessageEvent extends ExtendableEvent {
    constructor(type, init) {
      const event = messageEventInit(type, init);
      super(event.type);
      this.bubbles = event.bubbles;
      this.cancelable = event.cancelable;
      this.data = event.data;
      this.origin = event.origin;
      this.lastEventId = event.lastEventId;
      this.source = event.source;
      this.ports = event.ports.slice();
    }
  }
  // https://w3c.github.io/ServiceWorker/#extendablemessageevent-interface
  class ExtendableMessageEvent extends ExtendableEvent {
    constructor(type, init) {
      const event = messageEventInit(type, init);
      super(event.type);
      this.bubbles = event.bubbles;
      this.cancelable = event.cancelable;
      this.data = event.data;
      this.origin = event.origin;
      this.lastEventId = event.lastEventId;
      this.source = event.source;
      this.ports = event.ports.slice();
    }
  }
  function originOf(url) {
    try {
      return new URL(String(url)).origin;
    } catch (_error) {
      return '';
    }
  }
  class DOMException extends Error {
    constructor(message, name) {
      super(String(message));
      this.name = name === undefined ? 'Error' : String(name);
      this.code = this.name === 'AbortError' ? 20 : this.name === 'InvalidStateError' ? 11 : this.name === 'NetworkError' ? 19 : 0;
    }
  }
  Object.defineProperty(DOMException, 'ABORT_ERR', {value: 20});
  Object.defineProperty(DOMException.prototype, 'ABORT_ERR', {value: 20});
  Object.defineProperty(DOMException, 'INVALID_STATE_ERR', {value: 11});
  Object.defineProperty(DOMException.prototype, 'INVALID_STATE_ERR', {value: 11});
  Object.defineProperty(DOMException, 'NETWORK_ERR', {value: 19});
  Object.defineProperty(DOMException.prototype, 'NETWORK_ERR', {value: 19});

  // https://dom.spec.whatwg.org/#abortcontroller
  class AbortSignal {
    constructor() {
      this._aborted = false;
      this._reason = undefined;
      this._listeners = [];
    }
    get aborted() { return this._aborted; }
    get reason() { return this._reason; }
    addEventListener(type, listener) {
      if (type === 'abort' && typeof listener === 'function') this._listeners.push(listener);
    }
    removeEventListener(type, listener) {
      if (type !== 'abort') return;
      const index = this._listeners.indexOf(listener);
      if (index >= 0) this._listeners.splice(index, 1);
    }
    dispatchEvent() {
      return true;
    }
    throwIfAborted() {
      if (this._aborted) throw this._reason;
    }
    static abort(reason) {
      const signal = new AbortSignal();
      abortSignal(signal, reason);
      return signal;
    }
  }
  function abortSignal(signal, reason) {
    if (signal._aborted) return;
    signal._aborted = true;
    signal._reason = reason === undefined
      ? new DOMException('signal is aborted without reason', 'AbortError')
      : reason;
    const listeners = signal._listeners.slice();
    signal._listeners = [];
    for (const listener of listeners) {
      try {
        listener.call(signal, {type: 'abort', target: signal});
      } catch (_error) {}
    }
  }
  class AbortController {
    constructor() {
      this._signal = new AbortSignal();
    }
    get signal() { return this._signal; }
    abort(reason) {
      abortSignal(this._signal, reason);
    }
  }

  function utf8Encode(value) {
    const text = String(value);
    const bytes = [];
    for (let i = 0; i < text.length; i++) {
      const c = text.charCodeAt(i);
      if (c < 0x80) {
        bytes.push(c);
      } else if (c < 0x800) {
        bytes.push(0xc0 | (c >> 6), 0x80 | (c & 0x3f));
      } else if (c >= 0xd800 && c <= 0xdbff && text.charCodeAt(i + 1) >= 0xdc00 && text.charCodeAt(i + 1) <= 0xdfff) {
        const lo = text.charCodeAt(++i);
        const cp = 0x10000 + ((c & 0x3ff) << 10) + (lo & 0x3ff);
        bytes.push(0xf0 | (cp >> 18), 0x80 | ((cp >> 12) & 0x3f), 0x80 | ((cp >> 6) & 0x3f), 0x80 | (cp & 0x3f));
      } else {
        bytes.push(0xe0 | (c >> 12), 0x80 | ((c >> 6) & 0x3f), 0x80 | (c & 0x3f));
      }
    }
    return new Uint8Array(bytes);
  }
  function utf8Decode(bytes) {
    let text = '';
    for (let i = 0; i < bytes.length;) {
      const b = bytes[i];
      if (b < 0x80) {
        text += String.fromCharCode(b);
        i += 1;
      } else if (b < 0xc2 || i + 1 >= bytes.length) {
        text += '\uFFFD';
        i += 1;
      } else if (b < 0xe0) {
        text += String.fromCharCode(((b & 0x1f) << 6) | (bytes[i + 1] & 0x3f));
        i += 2;
      } else if (b < 0xf0) {
        if (i + 2 >= bytes.length) {
          text += '\uFFFD';
          i += 1;
        } else {
          text += String.fromCharCode(((b & 0x0f) << 12) | ((bytes[i + 1] & 0x3f) << 6) | (bytes[i + 2] & 0x3f));
          i += 3;
        }
      } else if (i + 3 >= bytes.length) {
        text += '\uFFFD';
        i += 1;
      } else {
        let cp = ((b & 0x07) << 18) | ((bytes[i + 1] & 0x3f) << 12) | ((bytes[i + 2] & 0x3f) << 6) | (bytes[i + 3] & 0x3f);
        cp -= 0x10000;
        text += String.fromCharCode(0xd800 + (cp >> 10), 0xdc00 + (cp & 0x3ff));
        i += 4;
      }
    }
    return text;
  }
  // https://encoding.spec.whatwg.org/#interface-textencoder
  class TextEncoderPolyfill {
    get encoding() {
      return 'utf-8';
    }
    encode(value) {
      return utf8Encode(value === undefined ? '' : value);
    }
    encodeInto(value, destination) {
      const text = String(value === undefined ? '' : value);
      const bytes = utf8Encode(text);
      const written = Math.min(bytes.length, destination && destination.length || 0);
      for (let i = 0; i < written; i++) destination[i] = bytes[i];
      return {read: text.length, written};
    }
  }
  // https://encoding.spec.whatwg.org/#interface-textdecoder
  class TextDecoderPolyfill {
    constructor() {
      this.encoding = 'utf-8';
      this.fatal = false;
      this.ignoreBOM = false;
    }
    decode(input) {
      if (input === undefined || input === null) return '';
      return utf8Decode(bodyPartBytes(input));
    }
  }
  function bodyPartBytes(part) {
    if (part === undefined || part === null) return new Uint8Array(0);
    if (typeof part === 'string') return utf8Encode(part);
    if (part instanceof ArrayBuffer) return new Uint8Array(part);
    if (ArrayBuffer.isView(part) && part.buffer instanceof ArrayBuffer) {
      const offset = part.byteOffset || 0;
      return new Uint8Array(part.buffer.slice(offset, offset + (part.byteLength || 0)));
    }
    if (part instanceof Blob) return blobBytes(part);
    return utf8Encode(String(part));
  }
  function blobBytes(blob) {
    const parts = blob._parts || [];
    const chunks = [];
    let total = 0;
    for (const part of parts) {
      const bytes = bodyPartBytes(part);
      chunks.push(bytes);
      total += bytes.length;
    }
    const out = new Uint8Array(total);
    let offset = 0;
    for (const bytes of chunks) {
      out.set(bytes, offset);
      offset += bytes.length;
    }
    return out;
  }
  // https://w3c.github.io/FileAPI/#blob
  class Blob {
    constructor(parts, options) {
      this._parts = parts === undefined || parts === null ? [] : Array.from(parts);
      this.size = this._parts.reduce((sum, part) => sum + bodyPartBytes(part).length, 0);
      this.type = options && options.type !== undefined ? String(options.type).toLowerCase() : '';
    }
    slice(start, end, contentType) {
      let from = start === undefined ? 0 : Number(start);
      let to = end === undefined ? this.size : Number(end);
      if (!Number.isFinite(from)) from = 0;
      if (!Number.isFinite(to)) to = 0;
      if (from < 0) from = Math.max(0, this.size + from);
      if (to < 0) to = Math.max(0, this.size + to);
      from = Math.min(Math.max(0, from), this.size);
      to = Math.min(Math.max(0, to), this.size);
      if (from > to) from = to;
      return new Blob([blobBytes(this).slice(from, to)], {
        type: contentType === undefined ? this.type : String(contentType)
      });
    }
    text() {
      return Promise.resolve(utf8Decode(blobBytes(this)));
    }
    arrayBuffer() {
      const bytes = blobBytes(this);
      const copy = new Uint8Array(bytes.length);
      copy.set(bytes);
      return Promise.resolve(copy);
    }
  }
  Object.defineProperty(Blob.prototype, Symbol.toStringTag, {value: 'Blob'});
  // https://w3c.github.io/FileAPI/#FileReader-interface
  class FileReader {
    constructor() {
      this.readyState = 0;
      this.result = null;
      this.error = null;
      this.onloadstart = null;
      this.onprogress = null;
      this.onload = null;
      this.onerror = null;
      this.onabort = null;
      this.onloadend = null;
      this._total = 0;
    }
    _fire(type, loaded, total) {
      const event = {
        type,
        target: this,
        lengthComputable: total !== undefined && total >= 0,
        loaded: loaded || 0,
        total: total === undefined ? 0 : total
      };
      const handler = this['on' + type];
      if (typeof handler === 'function') handler.call(this, event);
    }
    _start(blob) {
      this.readyState = 1;
      this.result = null;
      this.error = null;
      this._total = blob && blob.size !== undefined ? blob.size : 0;
      this._fire('loadstart', 0, this._total);
    }
    _done(result) {
      this.readyState = 2;
      this.result = result;
      this._fire('progress', this._total, this._total);
      this._fire('load', this._total, this._total);
      this._fire('loadend', this._total, this._total);
    }
    _fail(error) {
      this.readyState = 2;
      this.error = error;
      this._fire('error', 0, this._total);
      this._fire('loadend', 0, this._total);
    }
    readAsText(blob) {
      this._start(blob);
      blob.text().then(result => this._done(result), error => this._fail(error));
    }
    readAsArrayBuffer(blob) {
      this._start(blob);
      blob.arrayBuffer().then(result => this._done(result), error => this._fail(error));
    }
    readAsBinaryString(blob) {
      this._start(blob);
      blob.arrayBuffer().then(bytes => {
        let text = '';
        for (let i = 0; i < bytes.length; i++) text += String.fromCharCode(bytes[i]);
        this._done(text);
      }, error => this._fail(error));
    }
    abort() {
      if (this.readyState === 0 || this.readyState === 2) return;
      this.readyState = 2;
      this.result = null;
      this._fire('abort', 0, this._total);
      this._fire('loadend', 0, this._total);
    }
  }
  FileReader.EMPTY = 0;
  FileReader.LOADING = 1;
  FileReader.DONE = 2;
  Object.defineProperties(FileReader.prototype, {
    EMPTY: {value: 0},
    LOADING: {value: 1},
    DONE: {value: 2},
    [Symbol.toStringTag]: {value: 'FileReader'}
  });

  function formDataEntry(name, value, filename) {
    if (value instanceof Blob) {
      return [String(name), value, filename === undefined ? (value.name || 'blob') : String(filename)];
    }
    return [String(name), String(value), undefined];
  }
  let formDataCounter = 0;
  // https://xhr.spec.whatwg.org/#interface-formdata
  class FormData {
    constructor() {
      this._pairs = [];
    }
    append(name, value, filename) {
      this._pairs.push(formDataEntry(name, value, filename));
    }
    delete(name) {
      name = String(name);
      this._pairs = this._pairs.filter(pair => pair[0] !== name);
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
    has(name) {
      name = String(name);
      return this._pairs.some(pair => pair[0] === name);
    }
    set(name, value, filename) {
      const entry = formDataEntry(name, value, filename);
      const index = this._pairs.findIndex(pair => pair[0] === entry[0]);
      if (index < 0) {
        this._pairs.push(entry);
        return;
      }
      this._pairs[index] = entry;
      this._pairs = this._pairs.filter((pair, i) => pair[0] !== entry[0] || i === index);
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
    [Symbol.iterator]() {
      return this.entries();
    }
  }
  Object.defineProperty(FormData.prototype, Symbol.toStringTag, {value: 'FormData'});
  function formDataMultipart(formData) {
    const boundary = '----ZeroWebSWForm' + formDataCounter++;
    const bytes = [];
    const pushText = function(text) {
      const encoded = utf8Encode(text);
      for (let i = 0; i < encoded.length; i++) bytes.push(encoded[i]);
    };
    for (const entry of formData._pairs) {
      const name = entry[0];
      const value = entry[1];
      const filename = entry[2];
      pushText('--' + boundary + '\r\n');
      if (value instanceof Blob) {
        pushText('Content-Disposition: form-data; name="' + name + '"; filename="' + filename + '"\r\n');
        pushText('Content-Type: ' + (value.type || 'application/octet-stream') + '\r\n\r\n');
        const blob = blobBytes(value);
        for (let i = 0; i < blob.length; i++) bytes.push(blob[i]);
        pushText('\r\n');
      } else {
        pushText('Content-Disposition: form-data; name="' + name + '"\r\n\r\n');
        pushText(value);
        pushText('\r\n');
      }
    }
    pushText('--' + boundary + '--\r\n');
    const body = new Uint8Array(bytes.length);
    for (let i = 0; i < bytes.length; i++) body[i] = bytes[i];
    return {
      body,
      contentType: 'multipart/form-data; boundary=' + boundary
    };
  }

  // https://fetch.spec.whatwg.org/#headers-class
  function isHeaderNameChar(code) {
    return (code >= 0x30 && code <= 0x39)
      || (code >= 0x41 && code <= 0x5a)
      || (code >= 0x61 && code <= 0x7a)
      || code === 0x21 || code === 0x23 || code === 0x24 || code === 0x25
      || code === 0x26 || code === 0x27 || code === 0x2a || code === 0x2b
      || code === 0x2d || code === 0x2e || code === 0x5e || code === 0x5f
      || code === 0x60 || code === 0x7c || code === 0x7e;
  }
  function normalizeHeaderName(name) {
    name = String(name).toLowerCase().trim();
    // https://fetch.spec.whatwg.org/#concept-header-name
    if (name.length === 0) throw new TypeError('invalid header name');
    for (let i = 0; i < name.length; i++) {
      if (!isHeaderNameChar(name.charCodeAt(i))) throw new TypeError('invalid header name');
    }
    return name;
  }
  function normalizeHeaderValue(value) {
    value = String(value);
    // https://fetch.spec.whatwg.org/#concept-header-value
    for (let i = 0; i < value.length; i++) {
      const code = value.charCodeAt(i);
      if ((code < 0x20 && code !== 0x09) || code === 0x7f) {
        throw new TypeError('invalid header value');
      }
    }
    return value;
  }
  function isForbiddenResponseHeader(name) {
    return name === 'set-cookie' || name === 'set-cookie2';
  }
  function isInternalResponseHeader(name) {
    return name === 'x-zero-final-url' || name === 'x-zero-response-type' || name === 'x-zero-body-error';
  }
  function isHiddenResponseHeader(headers, name) {
    return headers._guard === 'response' && (isForbiddenResponseHeader(name) || isInternalResponseHeader(name));
  }
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
      name = normalizeHeaderName(name);
      value = normalizeHeaderValue(value);
      if (isHiddenResponseHeader(this, name)) return;
      this._pairs.push([name, value]);
    }
    delete(name) {
      if (this._guard === 'immutable') {
        throw new TypeError('Headers are immutable');
      }
      name = normalizeHeaderName(name);
      if (isHiddenResponseHeader(this, name)) return;
      this._pairs = this._pairs.filter(pair => pair[0] !== name);
    }
    get(name) {
      name = normalizeHeaderName(name);
      if (isHiddenResponseHeader(this, name)) return null;
      const values = this._pairs.filter(pair => pair[0] === name).map(pair => pair[1]);
      return values.length === 0 ? null : values.join(', ');
    }
    getSetCookie() {
      return this._pairs.filter(pair => pair[0] === 'set-cookie').map(pair => pair[1]);
    }
    has(name) {
      name = normalizeHeaderName(name);
      if (isHiddenResponseHeader(this, name)) return false;
      return this._pairs.some(pair => pair[0] === name);
    }
    set(name, value) {
      if (this._guard === 'immutable') {
        throw new TypeError('Headers are immutable');
      }
      name = normalizeHeaderName(name);
      value = normalizeHeaderValue(value);
      if (isHiddenResponseHeader(this, name)) return;
      this._pairs = this._pairs.filter(pair => pair[0] !== name);
      this._pairs.push([name, value]);
    }
    forEach(callback, thisArg) {
      for (const pair of this._pairs) {
        if (!isHiddenResponseHeader(this, pair[0])) callback.call(thisArg, pair[1], pair[0], this);
      }
    }
    entries() {
      return this._pairs
        .filter(pair => !isHiddenResponseHeader(this, pair[0]))
        .map(pair => [pair[0], pair[1]])[Symbol.iterator]();
    }
    [Symbol.iterator]() {
      return this.entries();
    }
  }
  Object.defineProperty(Headers.prototype, Symbol.toStringTag, {value: 'Headers'});

  function normalizeBody(body) {
    if (body === undefined || body === null) return '';
    return utf8Decode(bodyPartBytes(body));
  }
  const MAX_FETCH_BODY_BYTES = 16 * 1024 * 1024;
  if (typeof globalThis.ReadableStream !== 'function') {
    globalThis.ReadableStream = class ReadableStream {
      constructor(underlyingSource) {
        const source = underlyingSource || {};
        const queue = [];
        const waiting = [];
        let state = 'readable';
        let errorValue;
        let pulling = false;
        const flushPull = function() {
          if (pulling || state !== 'readable' || typeof source.pull !== 'function') return;
          pulling = true;
          try { source.pull(controller); } catch (error) { errorStream(error); }
          pulling = false;
        };
        const enqueueChunk = function(chunk) {
          if (state !== 'readable') return;
          if (waiting.length > 0) {
            waiting.shift().resolve({done: false, value: chunk});
          } else {
            queue.push(chunk);
          }
        };
        const closeStream = function() {
          if (state !== 'readable') return;
          state = 'closed';
          while (waiting.length > 0) waiting.shift().resolve({done: true, value: undefined});
        };
        const errorStream = function(error) {
          if (state !== 'readable') return;
          errorValue = error;
          state = 'errored';
          while (waiting.length > 0) waiting.shift().reject(error);
        };
        const controller = {
          enqueue: enqueueChunk,
          close: closeStream,
          error: errorStream,
          get desiredSize() { return state === 'errored' ? null : (state === 'closed' ? 0 : 1); }
        };
        // https://streams.spec.whatwg.org/#rs-constructor
        if (typeof source.start === 'function') {
          try { source.start(controller); } catch (error) { errorStream(error); }
        }
        this.getReader = function() {
          return {
            read: function() {
              return new Promise(function(resolve, reject) {
                if (state === 'errored') { reject(errorValue); return; }
                if (queue.length > 0) {
                  resolve({done: false, value: queue.shift()});
                  flushPull();
                  return;
                }
                if (state === 'closed') { resolve({done: true, value: undefined}); return; }
                waiting.push({resolve, reject});
                flushPull();
              });
            },
            releaseLock: function() {}
          };
        };
      }
    };
  }
  function isReadableStreamLike(body) {
    return body && typeof body.getReader === 'function';
  }
  function collectReadableStreamBody(stream) {
    const reader = stream.getReader();
    const chunks = [];
    let total = 0;
    function pump() {
      return reader.read().then(function(result) {
        if (result.done) {
          const out = new Uint8Array(total);
          let offset = 0;
          for (const chunk of chunks) {
            out.set(chunk, offset);
            offset += chunk.length;
          }
          return utf8Decode(out);
        }
        // https://fetch.spec.whatwg.org/#concept-bodyinit-extract
        // Response body streams must yield Uint8Array chunks; other chunk
        // types error the transferred body rather than being stringified.
        if (!(result.value instanceof Uint8Array)) {
          throw new TypeError('ReadableStream response body chunk is not a Uint8Array');
        }
        const bytes = bodyPartBytes(result.value);
        chunks.push(bytes);
        total += bytes.length;
        if (total > MAX_FETCH_BODY_BYTES) {
          throw new TypeError('Service Worker fetch response body exceeds the size limit');
        }
        return pump();
      });
    }
    function releaseReader() {
      if (reader && typeof reader.releaseLock === 'function') {
        try { reader.releaseLock(); } catch (_error) {}
      }
    }
    return pump().then(function(body) {
      releaseReader();
      return body;
    }, function(error) {
      releaseReader();
      return {__zwBodyError: String(error && error.message || error)};
    });
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
        this.signal = init.signal === undefined ? input.signal : init.signal;
        this.isReloadNavigation = !!input.isReloadNavigation;
        this.isHistoryNavigation = !!input.isHistoryNavigation;
        this.bodyUsed = false;
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
        this.signal = init.signal === undefined ? input.signal : init.signal;
        this.isReloadNavigation = !!input.isReloadNavigation;
        this.isHistoryNavigation = !!input.isHistoryNavigation;
        if (input.headerGuard === 'immutable' && init.headers === undefined) {
          this.headers._guard = 'immutable';
        }
        this.bodyUsed = false;
      } else {
        this.url = normalizeRequestURL(input);
        this.method = init.method === undefined ? 'GET' : String(init.method).toUpperCase();
        this.headers = new Headers(init.headers);
        this._body = normalizeBody(init.body);
        this.mode = init.mode === undefined ? 'cors' : String(init.mode);
        this.credentials = init.credentials === undefined ? 'same-origin' : String(init.credentials);
        this.redirect = init.redirect === undefined ? 'follow' : String(init.redirect);
        this.referrer = init.referrer === undefined ? '' : String(init.referrer);
        this.signal = init.signal;
        this.isReloadNavigation = false;
        this.isHistoryNavigation = false;
        this.bodyUsed = false;
      }
      if (!(this.signal instanceof AbortSignal)) this.signal = null;
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
      this.headers._guard = 'response';
      if (body instanceof FormData) {
        const multipart = formDataMultipart(body);
        if (!this.headers.has('content-type')) {
          this.headers.set('content-type', multipart.contentType);
        }
        this._body = utf8Decode(multipart.body);
      } else if (isReadableStreamLike(body)) {
        this._body = '';
        this._bodyStream = body;
      } else {
        this._body = normalizeBody(body);
      }
      this._bodyError = '';
      this.bodyUsed = false;
      this.ok = status >= 200 && status <= 299;
      this.type = 'default';
      this.url = init.url === undefined ? '' : String(init.url);
    }
    text() {
      this.bodyUsed = true;
      return Promise.resolve(this._body);
    }
    json() {
      this.bodyUsed = true;
      return Promise.resolve(JSON.parse(this._body));
    }
    blob() {
      this.bodyUsed = true;
      const contentType = this.headers.get('content-type') || '';
      return Promise.resolve(new Blob([this._body], {type: contentType}));
    }
    arrayBuffer() {
      this.bodyUsed = true;
      return Promise.resolve(utf8Encode(this._body));
    }
    clone() {
      if (this.bodyUsed) throw new TypeError('Response body has already been used');
      if (this.type === 'error') return Response.error();
      const cloned = new Response(this._body, {
        status: this.status === 0 ? 200 : this.status,
        statusText: this.statusText,
        headers: this.headers,
        url: this.url
      });
      cloned.status = this.status;
      cloned.statusText = this.statusText;
      cloned.ok = this.ok;
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
      response.url = '';
      return response;
    }
    // https://fetch.spec.whatwg.org/#dom-response-redirect
    static redirect(url, status) {
      status = status === undefined ? 302 : Number(status);
      if ([301, 302, 303, 307, 308].indexOf(status) < 0) {
        throw new RangeError('Invalid redirect status');
      }
      return new Response(null, {
        status,
        headers: [['Location', String(url)]]
      });
    }
    static _from(value) {
      if (value instanceof Response) return value;
      throw new TypeError('FetchEvent.respondWith must resolve with a Response');
    }
    static _serialize(response, options) {
      if (response.bodyUsed) throw new TypeError('Response body has already been used');
      options = options || {};
      const finish = function(body) {
        const headers = Array.from(response.headers);
        let bodyText = body;
        if (body && typeof body === 'object' && body.__zwBodyError !== undefined) {
          if (options.rejectBodyError) throw new TypeError(String(body.__zwBodyError));
          bodyText = '';
          headers.push(['x-zero-body-error', String(body.__zwBodyError)]);
        }
        if (response.url) headers.push(['x-zero-final-url', response.url]);
        return {
          status: response.status,
          statusText: response.statusText,
          type: response.type || 'default',
          headers,
          body: bodyText
        };
      };
      if (response._bodyStream) {
        return collectReadableStreamBody(response._bodyStream).then(finish);
      }
      return finish(response._body);
    }
  }
  Object.defineProperty(Response.prototype, Symbol.toStringTag, {value: 'Response'});

  function takeHeader(headers, name) {
    const normalized = normalizeHeaderName(name);
    let found = '';
    const kept = [];
    for (const pair of headers._pairs) {
      if (pair[0] === normalized) found = pair[1];
      else kept.push(pair);
    }
    headers._pairs = kept;
    return found;
  }
  function urlOrigin(url) {
    try {
      return new URL(url, globalThis.location && globalThis.location.href ? String(globalThis.location.href) : '').origin;
    } catch (_error) {
      const match = String(url).match(/^([A-Za-z][A-Za-z0-9+.-]*:\/\/[^\/?#]*)/);
      return match ? match[1] : '';
    }
  }
  function applyOpaqueFilter(response) {
    response.type = 'opaque';
    response.status = 0;
    response.statusText = '';
    response.ok = false;
    response.headers = new Headers();
    response.headers._guard = 'response';
    response._body = '';
    return response;
  }

  function cacheRequestWire(input) {
    const request = input instanceof Request ? input : new Request(input);
    return {
      url: request.url,
      method: request.method,
      headers: Array.from(request.headers),
      body: request._body,
      clientId: null,
      resultingClientId: null,
      isReloadNavigation: !!request.isReloadNavigation,
      isHistoryNavigation: !!request.isHistoryNavigation
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
  function cacheRequestCacheKey(request) {
    let url = String(request && request.url || '');
    try {
      const parsed = new URL(url);
      parsed._hash = '';
      parsed._sync();
      url = parsed.href;
    } catch (_error) {
      url = url.split('#')[0];
    }
    return String(request && request.method || 'GET').toUpperCase() + ' ' + url;
  }
  function cacheRequestHeader(request, name) {
    if (!request || !request.headers || typeof request.headers.get !== 'function') return null;
    const value = request.headers.get(name);
    return value === null ? null : String(value);
  }
  function cacheRequestsMatchByResponseVary(cachedRequest, queryRequest, response) {
    if (cacheRequestCacheKey(cachedRequest) !== cacheRequestCacheKey(queryRequest)) return false;
    if (!response || String(response.type || 'default').toLowerCase() === 'opaque') return true;
    const vary = response.headers && typeof response.headers.get === 'function' ? response.headers.get('vary') : null;
    if (vary === null) return true;
    let hasField = false;
    for (const field of String(vary).split(',').map(field => field.trim()).filter(field => field !== '')) {
      if (field === '*') return false;
      hasField = true;
      if (cacheRequestHeader(cachedRequest, field) !== cacheRequestHeader(queryRequest, field)) {
        return false;
      }
    }
    return hasField || String(vary).trim() === '';
  }
  function cacheAddAllHasDuplicate(entries) {
    // https://w3c.github.io/ServiceWorker/#batch-cache-operations
    for (let i = 0; i < entries.length; i++) {
      for (let j = 0; j < i; j++) {
        if (cacheRequestsMatchByResponseVary(entries[j].request, entries[i].request, entries[i].response)
            || cacheRequestsMatchByResponseVary(entries[i].request, entries[j].request, entries[j].response)) {
          return true;
        }
      }
    }
    return false;
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
    const headers = new Headers(response.headers || []);
    const finalURL = takeHeader(headers, 'x-zero-final-url');
    const metadataType = takeHeader(headers, 'x-zero-response-type');
    const responseType = String(metadataType || response.type || 'default').toLowerCase();
    if (responseType === 'error') {
      return Response.error();
    }
    const result = new Response(response.body || '', {
      status: response.status === 0 ? 200 : response.status,
      statusText: response.statusText || '',
      headers,
      url: finalURL
    });
    result.type = responseType;
    if (response.status === 0 || responseType === 'opaque') {
      result.status = response.status || 0;
      result.statusText = response.statusText || '';
      result.ok = result.status >= 200 && result.status <= 299;
    }
    return responseType === 'opaque' ? applyOpaqueFilter(result) : result;
  }
  function cachedRequestFromWire(request) {
    return new Request({
      url: request.url,
      method: request.method || 'GET',
      headers: request.headers || [],
      body: request.body == null ? undefined : request.body,
      isReloadNavigation: !!request.isReloadNavigation,
      isHistoryNavigation: !!request.isHistoryNavigation
    }, {
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
    };
    cacheSetNameWire(hostRequest, 'cacheName', cache._name);
    cacheSetIdWire(hostRequest, cache);
    const serialized = Response._serialize(cacheResponse, {rejectBodyError: true});
    if (serialized && typeof serialized.then === 'function') {
      return serialized.then(function(response) {
        hostRequest.response = response;
        return hostRequest;
      });
    }
    hostRequest.response = serialized;
    return hostRequest;
  }
  // https://w3c.github.io/ServiceWorker/#cache-interface
  class Cache {
    constructor(name, cacheId, liveCheck) {
      this._name = String(name);
      this._cacheId = cacheId === undefined || cacheId === null ? null : Number(cacheId);
      this._liveCheck = typeof liveCheck === 'function' ? liveCheck : null;
    }
    _assertLive() {
      if (this._liveCheck && !this._liveCheck()) throw cacheBucketDeletedError();
    }
    match(input, options) {
      this._assertLive();
      return cacheStorageHost(cacheMatchRequest(input, this._name, options, this)).then(function(response) {
        return response.response === null ? undefined : cachedResponseFromWire(response.response);
      });
    }
    matchAll(input, options) {
      this._assertLive();
      return cacheStorageHost(cacheMatchAllRequest(this, input, options)).then(function(response) {
        const responses = Array.isArray(response.responses) ? response.responses : [];
        return responses.map(cachedResponseFromWire);
      });
    }
    put(input, response) {
      this._assertLive();
      let request;
      try {
        request = cachePutRequest(this, input, response);
      } catch (error) {
        return Promise.reject(error);
      }
      return Promise.resolve(request).then(cacheStorageHost).then(function() {
        return undefined;
      });
    }
    add(input) {
      this._assertLive();
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
      this._assertLive();
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
        const seen = {};
        for (const request of requests) {
          const key = cacheRequestCacheKey(request) + ' ' + Array.from(request.headers).map(pair => pair[0] + '\u001e' + pair[1]).join('\u001f');
          if (seen[key]) {
            throw new DOMException('Cache.addAll duplicate requests', 'InvalidStateError');
          }
          seen[key] = true;
        }
        return Promise.all(requests.map(function(request) {
          return fetch(request.clone()).then(function(response) {
            if (!response || !response.ok) {
              throw new TypeError('Cache.addAll fetch response is not ok');
            }
            validateCachePut(request, response);
            return {request, response};
          });
        })).then(function(entries) {
          if (cacheAddAllHasDuplicate(entries)) {
            throw new DOMException('Cache.addAll duplicate requests', 'InvalidStateError');
          }
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
      this._assertLive();
      return cacheStorageHost(cacheKeysRequest(this, input, options)).then(function(response) {
        const requests = Array.isArray(response.requests) ? response.requests : [];
        return requests.map(cachedRequestFromWire);
      });
    }
    delete(input, options) {
      this._assertLive();
      const hasInput = arguments.length >= 1;
      let request;
      try {
        if (!hasInput) throw new TypeError('Cache.delete requires a request');
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

  // https://wicg.github.io/storage-buckets/#storagebucket
  function cacheBucketDeletedError() {
    try {
      return new DOMException('Storage bucket is deleted', 'UnknownError');
    } catch (_error) {
      const error = new Error('Storage bucket is deleted');
      error.name = 'UnknownError';
      return error;
    }
  }

  // https://w3c.github.io/ServiceWorker/#cache-storage-interface
  class CacheStorage {
    constructor(namePrefix, liveCheck, keyFromHostName) {
      this._namePrefix = typeof namePrefix === 'function' ? namePrefix : null;
      this._liveCheck = typeof liveCheck === 'function' ? liveCheck : null;
      this._keyFromHostName = typeof keyFromHostName === 'function' ? keyFromHostName : null;
    }
    _assertLive() {
      if (this._liveCheck && !this._liveCheck()) throw cacheBucketDeletedError();
    }
    _nameForHost(name) {
      return this._namePrefix ? this._namePrefix(String(name)) : name;
    }
    open(name) {
      const storage = this;
      const hasName = arguments.length >= 1;
      return new Promise(function(resolve, reject) {
        try {
          storage._assertLive();
          if (!hasName) throw new TypeError('CacheStorage.open requires a name');
          const request = {op: 'open'};
          const fallback = String(name);
          cacheSetNameWire(request, 'name', storage._nameForHost(name));
          cacheStorageHost(request).then(function(response) {
            resolve(new Cache(cacheNameFromResult(response, fallback), response.cacheId, storage._liveCheck));
          }, reject);
        } catch (error) {
          reject(error);
        }
      });
    }
    match(input, options) {
      this._assertLive();
      const cacheName = options === undefined || options === null ? undefined : Object(options).cacheName;
      const hostName = cacheName === undefined ? undefined : this._nameForHost(cacheName);
      return cacheStorageHost(cacheMatchRequest(input, hostName, options)).then(function(response) {
        return response.response === null ? undefined : cachedResponseFromWire(response.response);
      });
    }
    has(name) {
      const storage = this;
      const hasName = arguments.length >= 1;
      return new Promise(function(resolve, reject) {
        try {
          storage._assertLive();
          if (!hasName) throw new TypeError('CacheStorage.has requires a name');
          const request = {op: 'has'};
          cacheSetNameWire(request, 'name', storage._nameForHost(name));
          cacheStorageHost(request).then(function(response) {
            resolve(Boolean(response.value));
          }, reject);
        } catch (error) {
          reject(error);
        }
      });
    }
    delete(name) {
      const storage = this;
      const hasName = arguments.length >= 1;
      return new Promise(function(resolve, reject) {
        try {
          storage._assertLive();
          if (!hasName) throw new TypeError('CacheStorage.delete requires a name');
          const request = {op: 'storageDelete'};
          cacheSetNameWire(request, 'name', storage._nameForHost(name));
          cacheStorageHost(request).then(function(response) {
            resolve(Boolean(response.value));
          }, reject);
        } catch (error) {
          reject(error);
        }
      });
    }
    keys() {
      const storage = this;
      storage._assertLive();
      return cacheStorageHost({op: 'storageKeys'}).then(function(response) {
        let keys;
        if (Array.isArray(response.cacheNameUnits)) {
          keys = response.cacheNameUnits.map(cacheDomStringFromWire);
        } else {
          keys = Array.isArray(response.cacheNames) ? response.cacheNames.map(String) : [];
        }
        if (storage._keyFromHostName) {
          keys = keys.map(storage._keyFromHostName).filter(function(name) { return name !== null; });
        }
        return keys;
      });
    }
  }
  Object.defineProperty(CacheStorage.prototype, Symbol.toStringTag, {value: 'CacheStorage'});

  // https://wicg.github.io/storage-buckets/#storagebucket
  function storageBucketCachePrefix(bucketName) {
    return '__zw_storage_bucket__' + cacheDomStringWire(bucketName) + ':';
  }
  class StorageBucket {
    constructor(name, owner) {
      this.name = String(name);
      this._owner = owner;
      this._deleted = false;
      const prefix = storageBucketCachePrefix(this.name);
      const bucket = this;
      this.caches = new CacheStorage(
        function(cacheName) { return prefix + cacheName; },
        function() { return !bucket._deleted && owner._bucketExists(bucket.name); },
        function(hostName) {
          hostName = String(hostName);
          return hostName.indexOf(prefix) === 0 ? hostName.slice(prefix.length) : null;
        }
      );
    }
  }
  Object.defineProperty(StorageBucket.prototype, Symbol.toStringTag, {value: 'StorageBucket'});
  class StorageBucketManager {
    constructor() {
      this._buckets = {};
      this._order = [];
    }
    _bucketExists(name) {
      return Object.prototype.hasOwnProperty.call(this._buckets, String(name));
    }
    open(name) {
      const manager = this;
      return new Promise(function(resolve) {
        name = String(name);
        if (!manager._bucketExists(name)) {
          manager._buckets[name] = new StorageBucket(name, manager);
          manager._order.push(name);
        }
        resolve(manager._buckets[name]);
      });
    }
    keys() {
      const manager = this;
      return Promise.resolve(manager._order.filter(function(name) {
        return manager._bucketExists(name);
      }));
    }
    delete(name) {
      const manager = this;
      return new Promise(function(resolve, reject) {
        try {
          name = String(name);
          if (!manager._bucketExists(name)) {
            resolve(false);
            return;
          }
          const prefix = storageBucketCachePrefix(name);
          cacheStorageHost({op: 'storageKeys'}).then(function(response) {
            const keys = Array.isArray(response.cacheNameUnits)
              ? response.cacheNameUnits.map(cacheDomStringFromWire)
              : (Array.isArray(response.cacheNames) ? response.cacheNames.map(String) : []);
            let chain = Promise.resolve();
            keys.forEach(function(cacheName) {
              cacheName = String(cacheName);
              if (cacheName.indexOf(prefix) === 0) {
                const request = {op: 'storageDelete'};
                cacheSetNameWire(request, 'name', cacheName);
                chain = chain.then(function() { return cacheStorageHost(request); });
              }
            });
            return chain.then(function() {
              manager._buckets[name]._deleted = true;
              delete manager._buckets[name];
              manager._order = manager._order.filter(function(bucketName) { return bucketName !== name; });
              resolve(true);
            });
          }, reject);
        } catch (error) {
          reject(error);
        }
      });
    }
  }
  Object.defineProperty(StorageBucketManager.prototype, Symbol.toStringTag, {value: 'StorageBucketManager'});

  // https://w3c.github.io/ServiceWorker/#fetch-event-interface
  class FetchEvent extends ExtendableEvent {
    constructor(type, init) {
      super(type);
      // https://webidl.spec.whatwg.org/#required-dictionary-member
      init = Object(init);
      const request = init.request;
      if (!(request instanceof Request)) {
        throw new TypeError('FetchEventInit.request must be a Request');
      }
      this.request = request;
      this.clientId = init.clientId || '';
      this.resultingClientId = init.resultingClientId || '';
      this._respondWith = null;
      this._handledSettled = false;
      let handledResolve;
      let handledReject;
      // https://w3c.github.io/ServiceWorker/#dom-fetchevent-handled
      Object.defineProperty(this, 'handled', {
        value: new Promise(function(resolve, reject) {
          handledResolve = resolve;
          handledReject = reject;
        }),
        enumerable: true
      });
      this._settleHandled = function(succeeded, error) {
        if (this._handledSettled) return;
        this._handledSettled = true;
        if (succeeded) {
          handledResolve(undefined);
        } else {
          handledReject(error || new TypeError('FetchEvent handling failed'));
        }
      };
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
      this._protocol = response.protocol;
      this._hostname = response.hostname;
      this._port = response.port;
      this._pathname = response.pathname;
      this._search = response.search;
      this._hash = response.hash;
      this._sync();
    }
    _sync() {
      this.host = this._port ? this._hostname + ':' + this._port : this._hostname;
      this.origin = this._protocol + '//' + this.host;
      this.href = this.origin + this._pathname + this._search + this._hash;
      this.protocol = this._protocol;
      this.port = this._port;
      this.pathname = this._pathname;
      this.search = this._search;
      this.hash = this._hash;
      this.searchParams = new URLSearchParams(this._search.startsWith('?') ? this._search.slice(1) : this._search);
    }
    get hostname() {
      return this._hostname;
    }
    // https://url.spec.whatwg.org/#dom-url-hostname
    set hostname(value) {
      this._hostname = String(value);
      this._sync();
    }
    toString() {
      return this.href;
    }
    toJSON() {
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
    if (globalThis.serviceWorker &&
        Object.prototype.hasOwnProperty.call(globalThis.serviceWorker, '_scriptURL') &&
        globalThis.serviceWorker._scriptURL === '') {
      globalThis.serviceWorker._scriptURL = String(parts && parts.href || '');
    }
  };

  function WorkerGlobalScope() {}
  WorkerGlobalScope.prototype = Object.create(Object.getPrototypeOf(globalThis));
  // https://w3c.github.io/webappsec-secure-contexts/#dom-windoworworkerglobalscope-issecurecontext
  Object.defineProperty(WorkerGlobalScope.prototype, 'isSecureContext', {
    get: function() { return true; },
    configurable: true
  });
  function ServiceWorkerGlobalScope() {}
  ServiceWorkerGlobalScope.prototype = Object.create(WorkerGlobalScope.prototype);
  Object.defineProperty(ServiceWorkerGlobalScope.prototype, 'constructor', {
    value: ServiceWorkerGlobalScope,
    configurable: true,
    writable: true
  });
  Object.setPrototypeOf(globalThis, ServiceWorkerGlobalScope.prototype);
  // https://webidl.spec.whatwg.org/#dfn-immutable-prototype-exotic-object
  const immutablePrototypeObjects = [
    globalThis,
    ServiceWorkerGlobalScope.prototype,
    WorkerGlobalScope.prototype,
    Object.getPrototypeOf(WorkerGlobalScope.prototype),
    Object.prototype
  ];
  const originalObjectSetPrototypeOf = Object.setPrototypeOf;
  Object.setPrototypeOf = function(target, prototype) {
    if (immutablePrototypeObjects.indexOf(target) >= 0 &&
        Object.getPrototypeOf(target) !== prototype) {
      throw new TypeError('Immutable prototype object');
    }
    return originalObjectSetPrototypeOf(target, prototype);
  };
  if (typeof Reflect === 'object' && Reflect && typeof Reflect.setPrototypeOf === 'function') {
    const originalReflectSetPrototypeOf = Reflect.setPrototypeOf;
    Reflect.setPrototypeOf = function(target, prototype) {
      if (immutablePrototypeObjects.indexOf(target) >= 0 &&
          Object.getPrototypeOf(target) !== prototype) {
        return false;
      }
      return originalReflectSetPrototypeOf(target, prototype);
    };
  }
  globalThis.self = globalThis;
  globalThis.WorkerGlobalScope = WorkerGlobalScope;
  globalThis.ServiceWorkerGlobalScope = ServiceWorkerGlobalScope;
  globalThis.Event = Event;
  globalThis.ExtendableEvent = ExtendableEvent;
  globalThis.InstallEvent = InstallEvent;
  globalThis.ExtendableMessageEvent = ExtendableMessageEvent;
  globalThis.MessageEvent = MessageEvent;
  globalThis.MessagePort = MessagePort;
  globalThis.MessageChannel = MessageChannel;
  globalThis.TextEncoder = globalThis.TextEncoder || TextEncoderPolyfill;
  globalThis.TextDecoder = globalThis.TextDecoder || TextDecoderPolyfill;
  globalThis.Blob = globalThis.Blob || Blob;
  globalThis.FileReader = globalThis.FileReader || FileReader;
  globalThis.FormData = globalThis.FormData || FormData;
  globalThis.Headers = globalThis.Headers || Headers;
  globalThis.Request = globalThis.Request || Request;
  globalThis.Response = globalThis.Response || Response;
  globalThis.Cache = globalThis.Cache || Cache;
  globalThis.CacheStorage = globalThis.CacheStorage || CacheStorage;
  globalThis.StorageBucket = globalThis.StorageBucket || StorageBucket;
  globalThis.StorageBucketManager = globalThis.StorageBucketManager || StorageBucketManager;
  globalThis.caches = globalThis.caches || new CacheStorage();
  // https://html.spec.whatwg.org/multipage/workers.html#the-workernavigator-object
  // https://wicg.github.io/storage-buckets/#extensions-to-the-navigator-and-workernavigator-interfaces
  globalThis.navigator = globalThis.navigator || {};
  if (!globalThis.navigator.storageBuckets) {
    globalThis.navigator.storageBuckets = new StorageBucketManager();
  }
  globalThis.FetchEvent = FetchEvent;
  globalThis.DOMException = globalThis.DOMException || DOMException;
  globalThis.AbortController = globalThis.AbortController || AbortController;
  globalThis.AbortSignal = globalThis.AbortSignal || AbortSignal;
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
  globalThis.__zwHasQueuedTask = function() {
    return timerTasks.length > 0;
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
  function workerGlobalFetch(input, init) {
    let request;
    try {
      request = new Request(input, init);
    } catch (error) {
      return Promise.reject(error);
    }
    const signal = request.signal;
    return new Promise(function(resolve, reject) {
      if (signal && signal.aborted) {
        reject(signal.reason);
        return;
      }
      let settled = false;
      const abort = function() {
        if (settled) return;
        settled = true;
        reject(signal.reason);
      };
      if (signal) signal.addEventListener('abort', abort);
      const finish = function(response) {
        if (settled) return;
        settled = true;
        if (signal) signal.removeEventListener('abort', abort);
        if (!response || response.ok !== true) {
          reject(new TypeError(response && response.error || 'Service Worker fetch failed'));
          return;
        }
        try {
          const fetchResponse = cachedResponseFromWire(response.response);
          if (request.mode === 'no-cors' && urlOrigin(request.url) !== urlOrigin(globalThis.location && globalThis.location.href || '')) {
            applyOpaqueFilter(fetchResponse);
          }
          resolve(fetchResponse);
        } catch (_error) {
          reject(new TypeError('invalid Service Worker fetch response'));
        }
      };
      try {
        const hostRequest = cacheRequestWire(request);
        hostRequest.credentials = request.credentials;
        const response = JSON.parse(globalThis.__zwFetch(JSON.stringify(hostRequest)));
        if (signal) {
          setTimeout(function() { finish(response); }, 0);
        } else {
          finish(response);
        }
      } catch (_error) {
        if (!settled) {
          settled = true;
          if (signal) signal.removeEventListener('abort', abort);
          reject(new TypeError('invalid Service Worker fetch response'));
        }
      }
    });
  }
  // https://fetch.spec.whatwg.org/#fetch-method
  Object.defineProperty(WorkerGlobalScope.prototype, 'fetch', {
    value: workerGlobalFetch,
    configurable: true,
    writable: true
  });
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
      return Promise.resolve(response.client === null ? undefined : clientFromInfo(response.client));
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
        return clientFromInfo(info);
      }));
    }
    claim() {
      // https://w3c.github.io/ServiceWorker/#clients-claim
      if (!clientsClaimAllowed) {
        return Promise.reject(new DOMException(
          'clients.claim() is only available to an active Service Worker',
          'InvalidStateError'));
      }
      claimClientsRequested = true;
      if (globalThis.__zwLifecycleResult &&
          globalThis.__zwLifecycleResult.settled !== true) {
        globalThis.__zwLifecycleResult.claimClientsRequested = true;
      } else {
        activeEventClaimClientsRequested = true;
      }
      return Promise.resolve();
    }
  }
  globalThis.Clients = Clients;
  globalThis.WindowClient = WindowClient;
  globalThis.clients = new Clients();
  const serviceWorkerToken = {};
  class ServiceWorker {
    constructor(scriptURL, state, token) {
      if (token !== serviceWorkerToken) throw new TypeError('Illegal constructor');
      Object.defineProperties(this, {
        _listeners: {value: Object.create(null), writable: true},
        _scriptURL: {value: String(scriptURL), writable: true},
        _state: {value: String(state), writable: true},
        scriptURL: {get: function() { return this._scriptURL; }, enumerable: true},
        state: {get: function() { return this._state; }, enumerable: true}
      });
      this.onstatechange = null;
    }
    addEventListener(type, listener) {
      if (typeof listener !== 'function') return;
      const key = String(type);
      const list = this._listeners[key] || (this._listeners[key] = []);
      if (list.indexOf(listener) < 0) list.push(listener);
    }
    removeEventListener(type, listener) {
      const list = this._listeners[String(type)] || [];
      const index = list.indexOf(listener);
      if (index >= 0) list.splice(index, 1);
    }
    dispatchEvent(event) {
      if (!event || !event.type) throw new TypeError('Invalid event');
      const callbacks = (this._listeners[String(event.type)] || []).slice();
      for (let i = 0; i < callbacks.length; i++) callbacks[i].call(this, event);
      const handler = this['on' + String(event.type)];
      if (typeof handler === 'function') handler.call(this, event);
      return !event.defaultPrevented;
    }
    postMessage(data, transfer) {
      if (this === globalThis.serviceWorker) {
        const wire = prepareLocalPortTransfer(data, transfer);
        const source = this;
        queueMicrotask(function() {
          // https://w3c.github.io/ServiceWorker/#extendablemessageevent-interface
          const event = new ExtendableMessageEvent('message', {
            data: wire.data,
            origin: originOf(globalThis.location && globalThis.location.href || ''),
            ports: wire.ports,
            source: source
          });
          const callbacks = (listeners.message || []).slice();
          for (let i = 0; i < callbacks.length; i++) callbacks[i].call(globalThis, event);
          if (typeof globalThis.onmessage === 'function') {
            globalThis.onmessage.call(globalThis, event);
          }
        });
        return;
      }
      if (this._id !== undefined && this._id !== null) {
        queueWorkerMessage(this._id, data, transfer);
        return;
      }
      queueOutbound(data, transfer, null, null);
    }
  }
  Object.defineProperty(ServiceWorker.prototype, Symbol.toStringTag, {value: 'ServiceWorker'});
  const workerScriptURL = String(globalThis.location && globalThis.location.href || '');
  const currentServiceWorker = new ServiceWorker(workerScriptURL, 'parsed', serviceWorkerToken);
  const registrationListeners = Object.create(null);
  function serviceWorkerFromInfo(info) {
    if (!info) return null;
    const id = String(info.id);
    const worker = id === String(currentServiceWorker._id || '')
      ? currentServiceWorker
      : (serviceWorkersById[id] || new ServiceWorker(info.scriptURL || '', info.state || 'parsed', serviceWorkerToken));
    worker._id = id;
    worker._scriptURL = String(info.scriptURL || worker._scriptURL || '');
    if (worker !== currentServiceWorker) {
      setServiceWorkerState(worker, String(info.state || worker._state || 'parsed'));
    }
    serviceWorkersById[id] = worker;
    return worker;
  }
  const registration = {
    scope: '',
    installing: null,
    waiting: null,
    active: null,
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
    },
    unregister: function() {
      let response;
      try {
        response = JSON.parse(globalThis.__zwRequestUnregister());
      } catch (_error) {
        response = {ok: false, name: 'TypeError', message: 'invalid Service Worker unregister response'};
      }
      if (response && response.ok === true) return Promise.resolve(!!response.removed);
      return Promise.reject(new globalThis.DOMException(
        response && response.message || 'Service Worker unregister failed',
        response && response.name || 'TypeError'
      ));
    }
  };
  registration.addEventListener = function(type, listener) {
    if (typeof listener !== 'function') return;
    const key = String(type);
    const list = registrationListeners[key] || (registrationListeners[key] = []);
    if (list.indexOf(listener) < 0) list.push(listener);
  };
  registration.removeEventListener = function(type, listener) {
    const list = registrationListeners[String(type)] || [];
    const index = list.indexOf(listener);
    if (index >= 0) list.splice(index, 1);
  };
  registration.dispatchEvent = function(event) {
    if (!event || !event.type) throw new TypeError('Invalid event');
    const callbacks = (registrationListeners[String(event.type)] || []).slice();
    for (let i = 0; i < callbacks.length; i++) callbacks[i].call(registration, event);
    const handler = registration['on' + String(event.type)];
    if (typeof handler === 'function') handler.call(registration, event);
    return !event.defaultPrevented;
  };
  registration.onupdatefound = null;
  globalThis.ServiceWorker = ServiceWorker;
  // https://w3c.github.io/ServiceWorker/#serviceworkerglobalscope-serviceworker
  Object.defineProperty(globalThis, 'serviceWorker', {
    value: currentServiceWorker,
    enumerable: true,
    configurable: true
  });
  globalThis.registration = registration;
  globalThis.__zwSyncRegistrationPeers = function(currentId, peers) {
    if (currentId !== null && currentId !== undefined) {
      currentServiceWorker._id = String(currentId);
    }
    if (currentServiceWorker._id !== undefined && currentServiceWorker._id !== null) {
      serviceWorkersById[String(currentServiceWorker._id)] = currentServiceWorker;
    }
    const currentIdString = String(currentServiceWorker._id || '');
    if (peers && peers.installing && String(peers.installing.id) === currentIdString) {
      currentServiceWorker._state = String(peers.installing.state || currentServiceWorker._state || 'installing');
    }
    const hadInstalling = !!registration.installing;
    registration.installing = serviceWorkerFromInfo(peers && peers.installing);
    registration.waiting = serviceWorkerFromInfo(peers && peers.waiting);
    registration.active = serviceWorkerFromInfo(peers && peers.active);
    if (!hadInstalling && registration.installing) {
      registration.dispatchEvent(new Event('updatefound'));
    }
  };
  function setServiceWorkerState(worker, state) {
    if (!worker) return;
    const next = String(state);
    const changed = worker._state !== next;
    worker._state = next;
    if (changed) worker.dispatchEvent(new Event('statechange'));
  }
  function setCurrentServiceWorkerState(state) {
    setServiceWorkerState(currentServiceWorker, state);
  }
  function setRegistrationLifecyclePhase(type, running) {
    if (type === 'install') {
      registration.installing = running ? currentServiceWorker : null;
      registration.waiting = running ? null : currentServiceWorker;
      setCurrentServiceWorkerState(running ? 'installing' : 'installed');
      return;
    }
    if (type === 'activate') {
      registration.installing = null;
      registration.waiting = null;
      registration.active = currentServiceWorker;
      setCurrentServiceWorkerState(running ? 'activating' : 'activated');
    }
  }
  function importScriptsNetworkError(message) {
    return new globalThis.DOMException(String(message), 'NetworkError');
  }
  function rewriteServiceWorkerDynamicImport(source) {
    return String(source).split('import(').join('__zw_dynamic_import(');
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
      (0, eval)(rewriteServiceWorkerDynamicImport(response.sources[i]));
    }
  };
  globalThis.__zwDispatchLifecycle = function(type, eventId) {
    const pending = [];
    claimClientsRequested = false;
    setRegistrationLifecyclePhase(type, true);
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
      setRegistrationLifecyclePhase(type, false);
      return;
    }
    currentWaitUntil = null;
    Promise.all(pending).then(function() {
      result.settled = true;
      result.succeeded = true;
      setRegistrationLifecyclePhase(type, false);
    }, function(error) {
      result.settled = true;
      result.message = String(error && error.message || error);
      setRegistrationLifecyclePhase(type, false);
    });
  };
  function materializeTransferredPorts(portIds, workerOwnerRegistrationId) {
    return portIds.map(function(portId) {
      const key = String(portId);
      if (portEndpoints[key]) throw new DOMException('MessagePort already exists', 'DataCloneError');
      const port = new MessagePort();
      port._hostPortId = portId;
      port._remote = true;
      port._workerRemote = workerOwnerRegistrationId !== null && workerOwnerRegistrationId !== undefined;
      port._workerRegistrationId =
        workerOwnerRegistrationId === null || workerOwnerRegistrationId === undefined ? '' : String(workerOwnerRegistrationId);
      portEndpoints[key] = port;
      return port;
    });
  }
  function reviveTransferredPorts(value, ports) {
    if (value === null || typeof value !== 'object') return value;
    if (Object.keys(value).length === 1 &&
        Object.prototype.hasOwnProperty.call(value, transferredPortMarker)) {
      const index = Number(value[transferredPortMarker]);
      if (Number.isInteger(index) && index >= 0 && index < ports.length) return ports[index];
    }
    const keys = Object.keys(value);
    for (let i = 0; i < keys.length; i++) {
      value[keys[i]] = reviveTransferredPorts(value[keys[i]], ports);
    }
    return value;
  }
  globalThis.__zwDispatchMessage = function(
      eventId, data, clientId, clientURL, clientFrameType, clientFocused, portIds, dataPortIndex, targetPortId) {
    outboundMessages.splice(0, outboundMessages.length);
    const ports = materializeTransferredPorts(portIds || [], null);
    const eventData = dataPortIndex === null ? reviveTransferredPorts(data, ports) : ports[dataPortIndex];
    const EventClass = targetPortId !== null ? MessageEvent : ExtendableMessageEvent;
    const event = new EventClass('message', {data: eventData, origin: originOf(clientURL), ports: ports});
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
        event.source = clientFromInfo({
          id: clientId,
          url: clientURL,
          type: 'window',
          frameType: clientFrameType || 'top-level',
          visibilityState: 'visible',
          focused: clientFocused === true
        });
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
  globalThis.__zwDispatchWorkerMessage = function(eventId, data, sourceInfo, portIds, dataPortIndex, targetPortId) {
    outboundMessages.splice(0, outboundMessages.length);
    const ports = materializeTransferredPorts(portIds || [], sourceInfo && sourceInfo.id);
    const eventData = dataPortIndex === null ? reviveTransferredPorts(data, ports) : ports[dataPortIndex];
    const source = serviceWorkerFromInfo(sourceInfo);
    const event = new ExtendableMessageEvent('message', {
      data: eventData,
      origin: originOf(source && source.scriptURL || ''),
      ports: ports,
      source: source
    });
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
      workerMessages.splice(0, workerMessages.length);
      throw error;
    }
  };
  globalThis.__zwTakeOutboundMessages = function() {
    return outboundMessages.splice(0, outboundMessages.length);
  };
  globalThis.__zwTakeWorkerMessages = function() {
    return workerMessages.splice(0, workerMessages.length);
  };
  globalThis.__zwTakeClientsClaimRequested = function() {
    const requested = activeEventClaimClientsRequested;
    activeEventClaimClientsRequested = false;
    return requested;
  };
  globalThis.__zwSetClientsClaimAllowed = function(allowed) {
    clientsClaimAllowed = allowed === true;
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
          event._settleHandled(false, new DOMException('respondWith already called', 'InvalidStateError'));
          throw new DOMException('respondWith already called', 'InvalidStateError');
        }
        respondWithCalled = true;
        // https://w3c.github.io/ServiceWorker/#fetch-event-respondwith
        event.stopImmediatePropagation();
        pending.push(Promise.resolve(value).then(function(response) {
          if (result.failed) return;
          response = Response._from(response);
          return Promise.resolve(Response._serialize(response)).then(function(serialized) {
            result.responded = true;
            result.response = serialized;
          });
        }));
      };
      const callbacks = (listeners.fetch || []).slice();
      for (let i = 0; i < callbacks.length; i++) {
        callbacks[i].call(globalThis, event);
        if (event._immediateStopped) break;
      }
      if (!event._immediateStopped && typeof globalThis.onfetch === 'function') {
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
        if (event && typeof event._settleHandled === 'function') event._settleHandled(!result.failed, error);
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
        event._settleHandled(false, new TypeError(result.message));
      } else {
        event._settleHandled(true);
      }
      result.settled = true;
    }, function(error) {
      result.failed = true;
      result.response = null;
      result.responded = false;
      result.settled = true;
      result.message = String(error && error.message || error);
      event._settleHandled(false, error);
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
    /// The worker global called `registration.unregister()`.
    UnregisterRequested {
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
    /// A worker emitted messages targeting another Service Worker version.
    WorkerMessagesEmitted {
        /// Worker-to-worker messages with explicit target version IDs.
        messages: Vec<ServiceWorkerWorkerMessage>,
    },
    /// The worker called `clients.claim()` outside lifecycle event settlement.
    ClientsClaimRequested,
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

/// Pure-value Service Worker object projection for worker-to-worker messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceWorkerPeerInfo {
    /// Registration version ID represented by the worker object.
    pub id: u64,
    /// Worker script URL.
    pub script_url: String,
    /// Worker lifecycle state exposed to script.
    pub state: String,
}

/// Current visible Service Worker version slots for one registration object.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServiceWorkerRegistrationPeers {
    /// Installing worker projection.
    pub installing: Option<ServiceWorkerPeerInfo>,
    /// Waiting worker projection.
    pub waiting: Option<ServiceWorkerPeerInfo>,
    /// Active worker projection.
    pub active: Option<ServiceWorkerPeerInfo>,
}

impl ServiceWorkerRegistrationPeers {
    fn is_empty(&self) -> bool {
        self.installing.is_none() && self.waiting.is_none() && self.active.is_none()
    }
}

/// One worker-to-worker message emitted during a worker event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceWorkerWorkerMessage {
    /// Target registration version ID.
    pub target_registration_id: u64,
    /// Source worker projection exposed through `ExtendableMessageEvent.source`.
    pub source: ServiceWorkerPeerInfo,
    /// JSON-compatible structured payload.
    pub data_json: String,
    /// MessagePort endpoint IDs transferred with this message.
    pub transferred_port_ids: Vec<u64>,
    /// Index of the transferred port used as the payload itself.
    pub data_port_index: Option<usize>,
    /// Existing worker-owned endpoint addressed by this message.
    pub target_port_id: Option<u64>,
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
    /// Fetch credentials mode (`same-origin`, `omit`, or `include`).
    pub credentials: Option<String>,
    /// Browser-owned source client identity, when the request has one.
    pub client_id: Option<String>,
    /// Browser-owned resulting client identity for navigation requests, when known.
    pub resulting_client_id: Option<String>,
    /// Fetch request referrer exposed to `FetchEvent.request`.
    pub referrer: Option<String>,
    /// Whether this request was created by a reload navigation.
    pub is_reload_navigation: bool,
    /// Whether this request was created by a history navigation.
    pub is_history_navigation: bool,
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

/// Browser-owned source metadata for one page-to-worker message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceWorkerMessageSource<'a> {
    /// Browser-owned identity of the originating client.
    pub client_id: &'a str,
    /// Originating client URL.
    pub client_url: &'a str,
    /// Frame type of the originating window client.
    pub client_frame_type: &'a str,
    /// Whether the originating window client currently has focus.
    pub client_focused: bool,
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
    unregister_response_sender: mpsc::Sender<ServiceWorkerUnregisterResponse>,
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
        let (unregister_response_sender, unregister_response_receiver) = mpsc::channel();
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
                let unregister_event_sender = event_sender.clone();
                let unregister_response_receiver = Arc::new(Mutex::new(unregister_response_receiver));
                let next_unregister_request_id = Arc::new(AtomicU64::new(1));
                sandbox.register_callback(
                    "__zwRequestUnregister",
                    Box::new(move |_args| {
                        let request_id = next_unregister_request_id.fetch_add(1, Ordering::Relaxed);
                        if unregister_event_sender
                            .send(ServiceWorkerEvent::UnregisterRequested { request_id })
                            .is_err()
                        {
                            return update_failure_json("InvalidStateError", "Service Worker host disconnected");
                        }
                        let deadline =
                            std::time::Instant::now() + std::time::Duration::from_millis(lifecycle_timeout_ms);
                        loop {
                            let now = std::time::Instant::now();
                            if now >= deadline {
                                return update_failure_json("TimeoutError", "Service Worker unregister timed out");
                            }
                            let response = unregister_response_receiver
                                .lock()
                                .expect("unregister response lock")
                                .recv_timeout(deadline.saturating_duration_since(now));
                            match response {
                                Ok(ServiceWorkerUnregisterResponse::Completed {
                                    request_id: response_id,
                                    removed,
                                }) if response_id == request_id => {
                                    return serde_json::json!({"ok": true, "removed": removed}).to_string();
                                }
                                Ok(ServiceWorkerUnregisterResponse::Failed {
                                    request_id: response_id,
                                    exception_name,
                                    message,
                                }) if response_id == request_id => {
                                    return update_failure_json(&exception_name, &message);
                                }
                                Ok(ServiceWorkerUnregisterResponse::Shutdown) => {
                                    return update_failure_json(
                                        "InvalidStateError",
                                        "Service Worker runtime is shutting down",
                                    );
                                }
                                Ok(_) => continue,
                                Err(mpsc::RecvTimeoutError::Timeout) => {
                                    return update_failure_json("TimeoutError", "Service Worker unregister timed out");
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
                    emit_queued_messages(sandbox.as_mut(), &event_sender);
                    if let Some(pending) = pending_fetch.as_ref()
                        && let Some(event) = poll_fetch(sandbox.as_mut(), pending, lifecycle_timeout_ms)
                    {
                        emit_queued_messages(sandbox.as_mut(), &event_sender);
                        emit_clients_claim_requested(sandbox.as_mut(), &event_sender);
                        let _ = event_sender.send(event);
                        pending_fetch = None;
                    }
                    let has_queued_task =
                        pending_lifecycle.is_none() && pending_fetch.is_none() && has_queued_task(sandbox.as_mut());
                    let ran_idle_task = if pending_lifecycle.is_none() && pending_fetch.is_none() {
                        run_one_queued_task(sandbox.as_mut())
                    } else {
                        false
                    };
                    emit_queued_messages(sandbox.as_mut(), &event_sender);
                    emit_clients_claim_requested(sandbox.as_mut(), &event_sender);

                    let command =
                        if pending_lifecycle.is_some() || pending_fetch.is_some() || has_queued_task || ran_idle_task {
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
                            scope_url,
                            initial_peers,
                            is_module,
                        } => {
                            let source = if script.trim().is_empty() { ";" } else { script.as_str() };
                            let evaluation = set_worker_registration_scope(sandbox.as_mut(), &scope_url)
                                .and_then(|()| {
                                    if initial_peers.is_empty() {
                                        Ok(())
                                    } else {
                                        sync_worker_registration_peers(sandbox.as_mut(), None, &initial_peers)
                                    }
                                })
                                .and_then(|()| set_worker_location(sandbox.as_mut(), &script_url))
                                .and_then(|()| {
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
                                        evaluate_classic_script(sandbox.as_mut(), source)
                                    }
                                });
                            let event = match evaluation {
                                Ok(()) => {
                                    emit_queued_messages(sandbox.as_mut(), &event_sender);
                                    emit_clients_claim_requested(sandbox.as_mut(), &event_sender);
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
                        ServiceWorkerCommand::DispatchLifecycle {
                            event_id,
                            phase,
                            clients_claim_allowed,
                        } => {
                            let result = if pending_lifecycle.is_some() {
                                Err(ScriptError::RuntimeError(
                                    "Service Worker lifecycle event is already pending".into(),
                                ))
                            } else {
                                set_clients_claim_allowed(sandbox.as_mut(), clients_claim_allowed)
                                    .and_then(|()| begin_lifecycle(sandbox.as_mut(), event_id, phase))
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
                            client_frame_type,
                            client_focused,
                            ports,
                            clients_claim_allowed,
                        } => {
                            let dispatch = format!(
                                "globalThis.__zwDispatchMessage({}, {}, {}, {}, {}, {}, {}, {}, {});",
                                event_id,
                                data_json,
                                serde_json::to_string(&client_id).unwrap(),
                                serde_json::to_string(&client_url).unwrap(),
                                serde_json::to_string(&client_frame_type).unwrap(),
                                serde_json::to_string(&client_focused).unwrap(),
                                serde_json::to_string(&ports.transferred_port_ids).unwrap(),
                                serde_json::to_string(&ports.data_port_index).unwrap(),
                                serde_json::to_string(&ports.target_port_id).unwrap(),
                            );
                            let event = match set_clients_claim_allowed(sandbox.as_mut(), clients_claim_allowed)
                                .and_then(|()| sandbox.execute(&dispatch))
                            {
                                Ok(_) => match take_outbound_messages(sandbox.as_mut()) {
                                    Ok(outbound) => {
                                        emit_queued_worker_messages(sandbox.as_mut(), &event_sender);
                                        emit_clients_claim_requested(sandbox.as_mut(), &event_sender);
                                        ServiceWorkerEvent::MessageDispatched {
                                            event_id,
                                            client_id,
                                            outbound,
                                        }
                                    }
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
                        ServiceWorkerCommand::DispatchWorkerMessage {
                            event_id,
                            data_json,
                            source,
                            ports,
                            clients_claim_allowed,
                        } => {
                            let dispatch = format!(
                                "globalThis.__zwDispatchWorkerMessage({}, {}, {}, {}, {}, {});",
                                event_id,
                                data_json,
                                serde_json::to_string(&service_worker_peer_json(&source)).unwrap(),
                                serde_json::to_string(&ports.transferred_port_ids).unwrap(),
                                serde_json::to_string(&ports.data_port_index).unwrap(),
                                serde_json::to_string(&ports.target_port_id).unwrap(),
                            );
                            let event = match set_clients_claim_allowed(sandbox.as_mut(), clients_claim_allowed)
                                .and_then(|()| sandbox.execute(&dispatch))
                            {
                                Ok(_) => {
                                    emit_queued_messages(sandbox.as_mut(), &event_sender);
                                    emit_clients_claim_requested(sandbox.as_mut(), &event_sender);
                                    ServiceWorkerEvent::MessageDispatched {
                                        event_id,
                                        client_id: String::new(),
                                        outbound: Vec::new(),
                                    }
                                }
                                Err(error) => ServiceWorkerEvent::MessageFailed {
                                    event_id,
                                    client_id: String::new(),
                                    message: error.to_string(),
                                },
                            };
                            let _ = event_sender.send(event);
                        }
                        ServiceWorkerCommand::SyncRegistrationPeers { registration_id, peers } => {
                            let _ = sync_worker_registration_peers(sandbox.as_mut(), Some(registration_id), &peers);
                        }
                        ServiceWorkerCommand::DispatchFetch {
                            event_id,
                            request,
                            clients_claim_allowed,
                        } => {
                            let request_url = request.url.clone();
                            let result = if pending_fetch.is_some() {
                                Err(ScriptError::RuntimeError(
                                    "Service Worker fetch event is already pending".into(),
                                ))
                            } else {
                                set_clients_claim_allowed(sandbox.as_mut(), clients_claim_allowed)
                                    .and_then(|()| begin_fetch(sandbox.as_mut(), event_id, &request))
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
                unregister_response_sender,
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
        self.evaluate_with_scope(script, script_url, script_url)
    }

    /// Queue a script for evaluation with the registration scope exposed in the worker global.
    pub fn evaluate_with_scope(&mut self, script: &str, script_url: &str, scope_url: &str) -> Result<(), ScriptError> {
        self.evaluate_with_scope_and_peers(script, script_url, scope_url, ServiceWorkerRegistrationPeers::default())
    }

    /// Queue a script for evaluation with the initial registration slot projection.
    pub fn evaluate_with_scope_and_peers(
        &mut self,
        script: &str,
        script_url: &str,
        scope_url: &str,
        initial_peers: ServiceWorkerRegistrationPeers,
    ) -> Result<(), ScriptError> {
        if self.core.is_terminated() {
            return Err(ScriptError::InvalidInput(
                "Cannot evaluate script on terminated Service Worker runtime".into(),
            ));
        }
        if script_url.trim().is_empty() {
            return Err(ScriptError::InvalidInput("Service Worker script URL is empty".into()));
        }
        if scope_url.trim().is_empty() {
            return Err(ScriptError::InvalidInput("Service Worker scope URL is empty".into()));
        }
        self.core
            .send(ServiceWorkerCommand::Evaluate {
                script: script.to_string(),
                script_url: script_url.to_string(),
                scope_url: scope_url.to_string(),
                initial_peers,
                is_module: false,
            })
            .map_err(|_| ScriptError::RuntimeError("Service Worker runtime disconnected".into()))
    }

    /// Queue a JavaScript module graph for evaluation in the persistent Service Worker global.
    pub fn evaluate_module(&mut self, script: &str, script_url: &str) -> Result<(), ScriptError> {
        self.evaluate_module_with_scope(script, script_url, script_url)
    }

    /// Queue a JavaScript module graph with the registration scope exposed in the worker global.
    pub fn evaluate_module_with_scope(
        &mut self,
        script: &str,
        script_url: &str,
        scope_url: &str,
    ) -> Result<(), ScriptError> {
        self.evaluate_module_with_scope_and_peers(
            script,
            script_url,
            scope_url,
            ServiceWorkerRegistrationPeers::default(),
        )
    }

    /// Queue a JavaScript module graph with the initial registration slot projection.
    pub fn evaluate_module_with_scope_and_peers(
        &mut self,
        script: &str,
        script_url: &str,
        scope_url: &str,
        initial_peers: ServiceWorkerRegistrationPeers,
    ) -> Result<(), ScriptError> {
        if self.core.is_terminated() {
            return Err(ScriptError::InvalidInput(
                "Cannot evaluate script on terminated Service Worker runtime".into(),
            ));
        }
        if script_url.trim().is_empty() {
            return Err(ScriptError::InvalidInput("Service Worker script URL is empty".into()));
        }
        if scope_url.trim().is_empty() {
            return Err(ScriptError::InvalidInput("Service Worker scope URL is empty".into()));
        }
        self.core
            .send(ServiceWorkerCommand::Evaluate {
                script: script.to_string(),
                script_url: script_url.to_string(),
                scope_url: scope_url.to_string(),
                initial_peers,
                is_module: true,
            })
            .map_err(|_| ScriptError::RuntimeError("Service Worker runtime disconnected".into()))
    }

    /// Dispatch an install event.
    pub fn dispatch_install(&mut self, event_id: u64) -> Result<(), ScriptError> {
        self.dispatch_lifecycle_with_claim_allowed(event_id, ServiceWorkerLifecyclePhase::Install, false)
    }

    /// Dispatch an activate event.
    pub fn dispatch_activate(&mut self, event_id: u64) -> Result<(), ScriptError> {
        self.dispatch_lifecycle_with_claim_allowed(event_id, ServiceWorkerLifecyclePhase::Activate, true)
    }

    /// Dispatch one JSON-compatible page message.
    pub fn dispatch_message(
        &mut self,
        event_id: u64,
        data_json: &str,
        client_id: &str,
        client_url: &str,
    ) -> Result<(), ScriptError> {
        self.dispatch_message_with_ports_and_frame_type(
            event_id,
            data_json,
            client_id,
            client_url,
            "top-level",
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
        self.dispatch_message_with_ports_and_frame_type(event_id, data_json, client_id, client_url, "top-level", ports)
    }

    /// Dispatch a page message from a specific window-client frame type.
    pub fn dispatch_message_with_ports_and_frame_type(
        &mut self,
        event_id: u64,
        data_json: &str,
        client_id: &str,
        client_url: &str,
        client_frame_type: &str,
        ports: &ServiceWorkerMessagePorts,
    ) -> Result<(), ScriptError> {
        self.dispatch_message_with_ports_with_claim_allowed(
            event_id,
            data_json,
            ServiceWorkerMessageSource {
                client_id,
                client_url,
                client_frame_type,
                client_focused: true,
            },
            ports,
            true,
        )
    }

    /// Dispatch a page message with explicit `clients.claim()` eligibility.
    pub fn dispatch_message_with_ports_with_claim_allowed(
        &mut self,
        event_id: u64,
        data_json: &str,
        source: ServiceWorkerMessageSource<'_>,
        ports: &ServiceWorkerMessagePorts,
        clients_claim_allowed: bool,
    ) -> Result<(), ScriptError> {
        serde_json::from_str::<serde_json::Value>(data_json)
            .map_err(|error| ScriptError::InvalidInput(format!("invalid Service Worker message JSON: {error}")))?;
        if !matches!(source.client_frame_type, "top-level" | "auxiliary" | "nested") {
            return Err(ScriptError::InvalidInput(
                "invalid Service Worker client frame type".into(),
            ));
        }
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
                client_id: source.client_id.to_string(),
                client_url: source.client_url.to_string(),
                client_frame_type: source.client_frame_type.to_string(),
                client_focused: source.client_focused,
                ports: ports.clone(),
                clients_claim_allowed,
            })
            .map_err(|_| ScriptError::RuntimeError("Service Worker runtime disconnected".into()))
    }

    /// Synchronize the Service Worker global registration's visible worker slots.
    pub fn sync_registration_peers(
        &mut self,
        registration_id: u64,
        peers: ServiceWorkerRegistrationPeers,
    ) -> Result<(), ScriptError> {
        if self.core.is_terminated() {
            return Err(ScriptError::InvalidInput(
                "Cannot synchronize Service Worker peers on terminated runtime".into(),
            ));
        }
        self.core
            .send(ServiceWorkerCommand::SyncRegistrationPeers { registration_id, peers })
            .map_err(|_| ScriptError::RuntimeError("Service Worker runtime disconnected".into()))
    }

    /// Dispatch one worker-to-worker message.
    pub fn dispatch_worker_message(
        &mut self,
        event_id: u64,
        data_json: &str,
        source: ServiceWorkerPeerInfo,
        ports: &ServiceWorkerMessagePorts,
        clients_claim_allowed: bool,
    ) -> Result<(), ScriptError> {
        serde_json::from_str::<serde_json::Value>(data_json).map_err(|error| {
            ScriptError::InvalidInput(format!("invalid Service Worker worker message JSON: {error}"))
        })?;
        if ports.transferred_port_ids.len() > MAX_MESSAGE_PORTS
            || ports.transferred_port_ids.contains(&0)
            || ports.transferred_port_ids.iter().collect::<HashSet<_>>().len() != ports.transferred_port_ids.len()
            || ports
                .data_port_index
                .is_some_and(|index| index >= ports.transferred_port_ids.len())
            || ports.target_port_id == Some(0)
        {
            return Err(ScriptError::InvalidInput(
                "invalid worker-to-worker Service Worker MessagePort metadata".into(),
            ));
        }
        self.core
            .send(ServiceWorkerCommand::DispatchWorkerMessage {
                event_id,
                data_json: data_json.to_string(),
                source,
                ports: ports.clone(),
                clients_claim_allowed,
            })
            .map_err(|_| ScriptError::RuntimeError("Service Worker runtime disconnected".into()))
    }

    /// Dispatch one fetch event into the persistent Service Worker global.
    pub fn dispatch_fetch(&mut self, event_id: u64, request: ServiceWorkerFetchRequest) -> Result<(), ScriptError> {
        self.dispatch_fetch_with_claim_allowed(event_id, request, true)
    }

    /// Dispatch one fetch event with explicit `clients.claim()` eligibility.
    pub fn dispatch_fetch_with_claim_allowed(
        &mut self,
        event_id: u64,
        request: ServiceWorkerFetchRequest,
        clients_claim_allowed: bool,
    ) -> Result<(), ScriptError> {
        validate_fetch_request(&request)?;
        if self.core.is_terminated() {
            return Err(ScriptError::InvalidInput(
                "Cannot dispatch fetch on terminated Service Worker runtime".into(),
            ));
        }
        self.core
            .send(ServiceWorkerCommand::DispatchFetch {
                event_id,
                request,
                clients_claim_allowed,
            })
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

    /// Complete one blocking worker-global `registration.unregister()` request.
    pub fn complete_unregister(
        &self,
        request_id: u64,
        result: Result<bool, (String, String)>,
    ) -> Result<(), ScriptError> {
        let response = match result {
            Ok(removed) => ServiceWorkerUnregisterResponse::Completed { request_id, removed },
            Err((exception_name, message)) => ServiceWorkerUnregisterResponse::Failed {
                request_id,
                exception_name,
                message,
            },
        };
        self.unregister_response_sender
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
            .unregister_response_sender
            .send(ServiceWorkerUnregisterResponse::Shutdown);
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

    /// Dispatch a lifecycle event with explicit `clients.claim()` eligibility.
    pub fn dispatch_lifecycle_with_claim_allowed(
        &mut self,
        event_id: u64,
        phase: ServiceWorkerLifecyclePhase,
        clients_claim_allowed: bool,
    ) -> Result<(), ScriptError> {
        if self.core.is_terminated() {
            return Err(ScriptError::InvalidInput(
                "Cannot dispatch event on terminated Service Worker runtime".into(),
            ));
        }
        self.core
            .send(ServiceWorkerCommand::DispatchLifecycle {
                event_id,
                phase,
                clients_claim_allowed,
            })
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
        credentials: value["credentials"].as_str().map(str::to_string),
        client_id: value["clientId"].as_str().map(str::to_string),
        resulting_client_id: value["resultingClientId"].as_str().map(str::to_string),
        referrer: value["referrer"].as_str().map(str::to_string),
        is_reload_navigation: value["isReloadNavigation"].as_bool().unwrap_or(false),
        is_history_navigation: value["isHistoryNavigation"].as_bool().unwrap_or(false),
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
    if request
        .credentials
        .as_deref()
        .is_some_and(|credentials| !matches!(credentials, "omit" | "same-origin" | "include"))
    {
        return Err(ScriptError::InvalidInput(
            "Service Worker fetch request credentials mode is invalid".into(),
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
        validate_fetch_header_name(name)?;
        validate_fetch_header_value(value)?;
    }
    Ok(())
}

// https://fetch.spec.whatwg.org/#concept-header-name
fn validate_fetch_header_name(name: &str) -> Result<(), ScriptError> {
    if name.is_empty() || !name.bytes().all(is_http_token_byte) {
        return Err(ScriptError::InvalidInput(
            "Service Worker fetch response header name is invalid".into(),
        ));
    }
    Ok(())
}

// https://fetch.spec.whatwg.org/#concept-header-value
fn validate_fetch_header_value(value: &str) -> Result<(), ScriptError> {
    if value.bytes().any(|byte| (byte < 0x20 && byte != b'\t') || byte == 0x7f) {
        return Err(ScriptError::InvalidInput(
            "Service Worker fetch response header value is invalid".into(),
        ));
    }
    Ok(())
}

fn is_http_token_byte(byte: u8) -> bool {
    matches!(
        byte,
        b'0'..=b'9'
            | b'A'..=b'Z'
            | b'a'..=b'z'
            | b'!'
            | b'#'
            | b'$'
            | b'%'
            | b'&'
            | b'\''
            | b'*'
            | b'+'
            | b'-'
            | b'.'
            | b'^'
            | b'_'
            | b'`'
            | b'|'
            | b'~'
    )
}

fn fetch_request_json(request: &ServiceWorkerFetchRequest) -> serde_json::Value {
    let is_navigation = request.resulting_client_id.is_some();
    serde_json::json!({
        "url": &request.url,
        "method": &request.method,
        "headers": &request.headers,
        "body": &request.body,
        "credentials": request.credentials.as_deref().unwrap_or(if is_navigation { "include" } else { "same-origin" }),
        "clientId": &request.client_id,
        "resultingClientId": &request.resulting_client_id,
        "mode": if is_navigation { "navigate" } else { "cors" },
        "redirect": if is_navigation { "manual" } else { "follow" },
        "referrer": &request.referrer,
        "isReloadNavigation": request.is_reload_navigation,
        "isHistoryNavigation": request.is_history_navigation,
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

fn has_queued_task(sandbox: &mut dyn Sandbox) -> bool {
    sandbox
        .execute("globalThis.__zwHasQueuedTask && globalThis.__zwHasQueuedTask() ? 'true' : 'false';")
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
        validate_fetch_header_name(&name)?;
        validate_fetch_header_value(&value)?;
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
    let mut main_url = url::Url::parse(script_url)
        .map_err(|error| ScriptError::InvalidInput(format!("invalid Service Worker module URL: {error}")))?;
    main_url.set_fragment(None);
    let main_url = main_url.to_string();
    if service_worker_module_source_has_top_level_await(source) {
        // https://w3c.github.io/ServiceWorker/#service-worker-script-request
        // FIXME: Replace this fail-closed check once Service Worker module evaluation has
        // spec-accurate asynchronous module job rejection instead of the current synchronous runner.
        return Err(ScriptError::RuntimeError(
            "Service Worker module scripts with top-level await are not supported".into(),
        ));
    }
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
    // https://w3c.github.io/ServiceWorker/#service-worker-script-request
    // Service Worker scripts do not perform dynamic imports; preserve evaluation and reject at call sites.
    sandbox.execute(&format!(
        "globalThis.__zwModuleScriptMode = true;{SERVICE_WORKER_DYNAMIC_IMPORT_PRELUDE}"
    ))?;
    sandbox.execute(&compiled).map(|_| ())
}

fn evaluate_classic_script(sandbox: &mut dyn Sandbox, source: &str) -> Result<(), ScriptError> {
    // https://w3c.github.io/ServiceWorker/#service-worker-script-request
    // Dynamic import is unavailable in Service Worker scripts; V8's host hook reports a generic
    // Error, so rewrite import() to the same TypeError rejected promise used by module workers.
    sandbox.execute(SERVICE_WORKER_DYNAMIC_IMPORT_PRELUDE)?;
    let source = rewrite_dynamic_imports(source);
    sandbox.execute(&source).map(|_| ())
}

fn service_worker_module_source_has_top_level_await(source: &str) -> bool {
    source.contains("await")
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

fn set_worker_registration_scope(sandbox: &mut dyn Sandbox, scope_url: &str) -> Result<(), ScriptError> {
    let url = url::Url::parse(scope_url)
        .map_err(|error| ScriptError::InvalidInput(format!("invalid Service Worker scope URL: {error}")))?;
    let scope = serde_json::to_string(url.as_str()).expect("URL string is serializable");
    sandbox
        .execute(&format!("globalThis.registration.scope = {scope};"))
        .map(|_| ())
}

fn set_clients_claim_allowed(sandbox: &mut dyn Sandbox, allowed: bool) -> Result<(), ScriptError> {
    let script = format!(
        "globalThis.__zwSetClientsClaimAllowed({});",
        if allowed { "true" } else { "false" }
    );
    sandbox.execute(&script).map(|_| ())
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
    let _ = sandbox.execute("globalThis.__zwRunOneTask && globalThis.__zwRunOneTask();");
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

fn take_clients_claim_requested(sandbox: &mut dyn Sandbox) -> Result<bool, ScriptError> {
    sandbox
        .execute("globalThis.__zwTakeClientsClaimRequested && globalThis.__zwTakeClientsClaimRequested() ? 'true' : 'false';")
        .map(|result| result.value == "true")
}

fn emit_clients_claim_requested(sandbox: &mut dyn Sandbox, event_sender: &mpsc::Sender<ServiceWorkerEvent>) {
    if take_clients_claim_requested(sandbox).unwrap_or(false) {
        let _ = event_sender.send(ServiceWorkerEvent::ClientsClaimRequested);
    }
}

fn emit_queued_messages(sandbox: &mut dyn Sandbox, event_sender: &mpsc::Sender<ServiceWorkerEvent>) {
    if let Ok(outbound) = take_outbound_messages(sandbox)
        && !outbound.is_empty()
    {
        let _ = event_sender.send(ServiceWorkerEvent::ClientMessagesEmitted { outbound });
    }
    emit_queued_worker_messages(sandbox, event_sender);
}

fn emit_queued_worker_messages(sandbox: &mut dyn Sandbox, event_sender: &mpsc::Sender<ServiceWorkerEvent>) {
    if let Ok(messages) = take_worker_messages(sandbox)
        && !messages.is_empty()
    {
        let _ = event_sender.send(ServiceWorkerEvent::WorkerMessagesEmitted { messages });
    }
}

fn sync_worker_registration_peers(
    sandbox: &mut dyn Sandbox,
    registration_id: Option<u64>,
    peers: &ServiceWorkerRegistrationPeers,
) -> Result<(), ScriptError> {
    let current_id = registration_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "null".to_string());
    let script = format!(
        "globalThis.__zwSyncRegistrationPeers({}, {});",
        current_id,
        serde_json::to_string(&service_worker_peers_json(peers)).unwrap(),
    );
    sandbox.execute(&script).map(|_| ())
}

fn service_worker_peer_json(peer: &ServiceWorkerPeerInfo) -> serde_json::Value {
    serde_json::json!({
        "id": peer.id,
        "scriptURL": peer.script_url,
        "state": peer.state,
    })
}

fn service_worker_peers_json(peers: &ServiceWorkerRegistrationPeers) -> serde_json::Value {
    serde_json::json!({
        "installing": peers.installing.as_ref().map(service_worker_peer_json),
        "waiting": peers.waiting.as_ref().map(service_worker_peer_json),
        "active": peers.active.as_ref().map(service_worker_peer_json),
    })
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

fn take_worker_messages(sandbox: &mut dyn Sandbox) -> Result<Vec<ServiceWorkerWorkerMessage>, ScriptError> {
    const MAX_WORKER_MESSAGES: usize = 1024;
    const MAX_MESSAGE_BYTES: usize = 1024 * 1024;
    const MAX_WORKER_BATCH_BYTES: usize = 16 * 1024 * 1024;
    let result = sandbox.execute("JSON.stringify(globalThis.__zwTakeWorkerMessages())")?;
    let values = serde_json::from_str::<Vec<serde_json::Value>>(&result.value)
        .map_err(|error| ScriptError::RuntimeError(format!("invalid worker message list: {error}")))?;
    if values.len() > MAX_WORKER_MESSAGES {
        return Err(ScriptError::InvalidInput(
            "Service Worker emitted too many worker messages in one event".into(),
        ));
    }
    let mut total_bytes = 0usize;
    values
        .into_iter()
        .map(|value| {
            let data_json = value["dataJSON"]
                .as_str()
                .ok_or_else(|| ScriptError::RuntimeError("worker message data is missing".into()))?
                .to_string();
            serde_json::from_str::<serde_json::Value>(&data_json)
                .map_err(|error| ScriptError::RuntimeError(format!("invalid worker message data: {error}")))?;
            if data_json.len() > MAX_MESSAGE_BYTES {
                return Err(ScriptError::InvalidInput(
                    "Service Worker worker message exceeds the size limit".into(),
                ));
            }
            total_bytes = total_bytes.saturating_add(data_json.len());
            if total_bytes > MAX_WORKER_BATCH_BYTES {
                return Err(ScriptError::InvalidInput(
                    "Service Worker worker message batch exceeds the size limit".into(),
                ));
            }
            let target_registration_id = value["targetRegistrationId"]
                .as_str()
                .and_then(|id| id.parse::<u64>().ok())
                .ok_or_else(|| ScriptError::RuntimeError("worker message target is missing".into()))?;
            let target_port_id = value["targetPortId"].as_u64();
            let source = &value["source"];
            let source_id = source["id"]
                .as_str()
                .and_then(|id| id.parse::<u64>().ok())
                .ok_or_else(|| ScriptError::RuntimeError("worker message source is missing".into()))?;
            let transferred_port_ids = value["transferredPortIds"]
                .as_array()
                .map(|values| values.iter().filter_map(serde_json::Value::as_u64).collect::<Vec<_>>())
                .unwrap_or_default();
            let data_port_index = value["dataPortIndex"]
                .as_u64()
                .and_then(|index| usize::try_from(index).ok());
            if transferred_port_ids.len() > MAX_MESSAGE_PORTS
                || transferred_port_ids.contains(&0)
                || transferred_port_ids.iter().collect::<HashSet<_>>().len() != transferred_port_ids.len()
                || data_port_index.is_some_and(|index| index >= transferred_port_ids.len())
                || target_port_id == Some(0)
            {
                return Err(ScriptError::InvalidInput(
                    "invalid worker-to-worker Service Worker MessagePort metadata".into(),
                ));
            }
            Ok(ServiceWorkerWorkerMessage {
                target_registration_id,
                source: ServiceWorkerPeerInfo {
                    id: source_id,
                    script_url: source["scriptURL"].as_str().unwrap_or_default().to_string(),
                    state: source["state"].as_str().unwrap_or("parsed").to_string(),
                },
                data_json,
                transferred_port_ids,
                data_port_index,
                target_port_id,
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
    use std::time::{Duration, Instant};

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
    fn fetch_lives_on_worker_global_scope_prototype() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "if (self.hasOwnProperty('fetch')) throw new Error('fetch is an own property');
                 const workerProto = Object.getPrototypeOf(Object.getPrototypeOf(self));
                 if (!Object.prototype.hasOwnProperty.call(workerProto, 'fetch')) {
                   throw new Error('WorkerGlobalScope.prototype missing fetch');
                 }
                 if (self.fetch !== workerProto.fetch) throw new Error('fetch identity mismatch');",
                "https://example.test/sw.js",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn service_worker_global_is_secure_context() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "if (self.hasOwnProperty('isSecureContext')) throw new Error('isSecureContext is an own property');
                 const workerProto = Object.getPrototypeOf(Object.getPrototypeOf(self));
                 if (!Object.prototype.hasOwnProperty.call(workerProto, 'isSecureContext')) {
                   throw new Error('WorkerGlobalScope.prototype missing isSecureContext');
                 }
                 if (self.isSecureContext !== true) throw new Error('wrong secure context value');",
                "https://example.test/sw.js",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn service_worker_global_prototype_chain_is_immutable() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "const protected = [
                   self,
                   ServiceWorkerGlobalScope.prototype,
                   WorkerGlobalScope.prototype,
                   Object.getPrototypeOf(WorkerGlobalScope.prototype),
                   Object.prototype
                 ];
                 for (const item of protected) {
                   const original = Object.getPrototypeOf(item);
                   let threw = false;
                   try {
                     Object.setPrototypeOf(item, {});
                   } catch (error) {
                     threw = error instanceof TypeError;
                   }
                   if (!threw) throw new Error('Object.setPrototypeOf accepted protected object');
                   if (Object.getPrototypeOf(item) !== original) throw new Error('prototype changed');
                   if (Reflect.setPrototypeOf(item, {}) !== false) {
                     throw new Error('Reflect.setPrototypeOf accepted protected object');
                   }
                   if (Object.getPrototypeOf(item) !== original) throw new Error('reflect changed prototype');
                 }
                 const ordinary = {};
                 const proto = {marker: true};
                 Object.setPrototypeOf(ordinary, proto);
                 if (Object.getPrototypeOf(ordinary) !== proto) throw new Error('ordinary object blocked');",
                "https://example.test/sw.js",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn fetch_event_constructor_requires_request_member() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "function assertTypeError(callback, label) {
                   let threw = false;
                   try {
                     callback();
                   } catch (error) {
                     threw = error instanceof TypeError;
                   }
                   if (!threw) throw new Error(label);
                 }
                 assertTypeError(() => new FetchEvent('FetchEvent'), 'missing init was accepted');
                 assertTypeError(() => new FetchEvent('FetchEvent', {}), 'missing request was accepted');
                 assertTypeError(
                   () => new FetchEvent('FetchEvent', {request: null}),
                   'null request was accepted'
                 );
                 const request = new Request('https://example.test/data');
                 const event = new FetchEvent('FetchEvent', {request, clientId: 'client-1'});
                 if (event.type !== 'FetchEvent') throw new Error('wrong type');
                 if (event.request !== request) throw new Error('wrong request');
                 if (event.clientId !== 'client-1') throw new Error('wrong clientId');
                 if (event.cancelable !== false) throw new Error('wrong cancelable');
                 if (event.bubbles !== false) throw new Error('wrong bubbles');
                 if (event.isReload !== undefined) throw new Error('unexpected isReload');",
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
    fn extendable_message_event_constructor_applies_default_and_init_values() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "const empty = new ExtendableMessageEvent('type');
                 if (empty.type !== 'type') throw new Error('wrong type');
                 if (empty.bubbles !== false) throw new Error('wrong default bubbles');
                 if (empty.cancelable !== false) throw new Error('wrong default cancelable');
                 if (empty.data !== null) throw new Error('wrong default data');
                 if (empty.origin !== '') throw new Error('wrong default origin');
                 if (empty.lastEventId !== '') throw new Error('wrong default lastEventId');
                 if (empty.source !== null) throw new Error('wrong default source');
                 if (!Array.isArray(empty.ports) || empty.ports.length !== 0) {
                   throw new Error('wrong default ports');
                 }
                 const channel = new MessageChannel();
                 const payload = {value: 7};
                 const filled = new ExtendableMessageEvent('type', {
                   bubbles: 1,
                   cancelable: true,
                   data: payload,
                   origin: 123,
                   lastEventId: null,
                   source: registration.active,
                   ports: [channel.port1]
                 });
                 if (filled.bubbles !== true) throw new Error('wrong bubbles');
                 if (filled.cancelable !== true) throw new Error('wrong cancelable');
                 if (filled.data !== payload) throw new Error('wrong data');
                 if (filled.origin !== '123') throw new Error('wrong origin');
                 if (filled.lastEventId !== 'null') throw new Error('wrong lastEventId');
                 if (filled.source !== registration.active) throw new Error('wrong source');
                 if (filled.ports.length !== 1 || filled.ports[0] !== channel.port1) {
                   throw new Error('wrong ports');
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
    fn extendable_message_event_constructor_rejects_invalid_source_and_ports() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "let invalidSource = false;
                 try {
                   new ExtendableMessageEvent('type', {source: self});
                 } catch (error) {
                   invalidSource = error instanceof TypeError;
                 }
                 if (!invalidSource) throw new Error('invalid source was accepted');
                 let invalidPorts = false;
                 try {
                   new ExtendableMessageEvent('type', {ports: [1]});
                 } catch (error) {
                   invalidPorts = error instanceof TypeError;
                 }
                 if (!invalidPorts) throw new Error('invalid ports were accepted');
                 let getterThrown = {name: 'Error'};
                 try {
                   new ExtendableMessageEvent('type', {get data() { throw getterThrown; }});
                 } catch (error) {
                   if (error !== getterThrown) throw new Error('wrong getter error');
                   getterThrown.caught = true;
                 }
                 if (!getterThrown.caught) throw new Error('getter throw was not propagated');",
                "https://example.test/sw.js",
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
    fn module_dynamic_import_rejects_at_runtime() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate_module(
                "import('./late.js').then(
                   () => { throw new Error('dynamic import resolved'); },
                   error => {
                     if (!(error instanceof TypeError)) throw error;
                   }
                 );",
                "https://example.test/workers/sw.js",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn service_worker_module_top_level_await_rejects_evaluation() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate_module(
                "await Promise.resolve(1);
                 globalThis.ready = true;",
                "https://example.test/workers/sw.js",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::ScriptError {
                kind: ServiceWorkerScriptErrorKind::Runtime,
                message,
                ..
            } if message.contains("top-level await")
        ));
    }

    #[test]
    fn classic_dynamic_import_rejects_with_type_error() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "import('./late.js').then(
                   () => { throw new Error('dynamic import resolved'); },
                   error => {
                     if (!(error instanceof TypeError)) throw error;
                   }
                 );",
                "https://example.test/workers/sw.js",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn classic_imported_script_dynamic_import_rejects_with_type_error() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate("importScripts('./helper.js');", "https://example.test/workers/sw.js")
            .unwrap();
        let ServiceWorkerEvent::ImportScriptsRequested { request_id, .. } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing importScripts request");
        };
        runtime
            .complete_import_scripts(
                request_id,
                Ok(vec![
                    "import('./late.js').then(
                       () => { throw new Error('dynamic import resolved'); },
                       error => {
                         if (!(error instanceof TypeError)) throw error;
                       }
                     );"
                    .into(),
                ]),
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
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
                    if (event.type !== 'install') throw new Error('wrong event type');
                    if (event.bubbles !== false) throw new Error('wrong bubbles');
                    if (event.cancelable !== false) throw new Error('wrong cancelable');
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
    fn service_worker_global_exposes_current_worker_and_lifecycle_registration() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "if (!('serviceWorker' in self)) throw new Error('missing serviceWorker');
                 if (registration.installing !== null) throw new Error('initial installing');
                 if (registration.waiting !== null) throw new Error('initial waiting');
                 if (registration.active !== null) throw new Error('initial active');
                 if (serviceWorker.state !== 'parsed') throw new Error('initial state');
                 var desc = Object.getOwnPropertyDescriptor(self, 'serviceWorker');
                 if (!desc || desc.writable !== false) throw new Error('serviceWorker not readonly');
                 globalThis.__initialServiceWorker = serviceWorker;
                 addEventListener('install', event => {
                   if (serviceWorker !== globalThis.__initialServiceWorker) throw new Error('install identity');
                   if (registration.installing !== serviceWorker) throw new Error('installing mismatch');
                   if (registration.waiting !== null) throw new Error('install waiting');
                   if (registration.active !== null) throw new Error('install active');
                   if (serviceWorker.state !== 'installing') throw new Error('install state');
                 });
                 addEventListener('activate', event => {
                   if (serviceWorker !== globalThis.__initialServiceWorker) throw new Error('activate identity');
                   if (registration.installing !== null) throw new Error('activate installing');
                   if (registration.waiting !== null) throw new Error('activate waiting');
                   if (registration.active !== serviceWorker) throw new Error('active mismatch');
                   if (serviceWorker.state !== 'activating') throw new Error('activate state');
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));

        runtime.dispatch_install(71).unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::LifecycleSettled {
                event_id: 71,
                phase: ServiceWorkerLifecyclePhase::Install,
                succeeded: true,
                skip_waiting: false,
                claim_clients: false,
                message: String::new(),
            }
        );

        runtime.dispatch_activate(72).unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::LifecycleSettled {
                event_id: 72,
                phase: ServiceWorkerLifecyclePhase::Activate,
                succeeded: true,
                skip_waiting: false,
                claim_clients: false,
                message: String::new(),
            }
        );

        runtime
            .evaluate(
                "if (serviceWorker.scriptURL !== 'https://example.test/sw.js') throw new Error('scriptURL changed');
                 if (registration.active !== serviceWorker) throw new Error('final active');
                 if (serviceWorker.state !== 'activated') throw new Error('final state');",
                "https://example.test/check.js",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn service_worker_global_registration_scope_is_available_during_events() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate_with_scope(
                "const expectedScope = 'https://example.test/app/';
                 if (registration.scope !== expectedScope) throw new Error('initial scope');
                 addEventListener('install', event => {
                   if (registration.scope !== expectedScope) throw new Error('install scope');
                 });
                 addEventListener('activate', event => {
                   if (registration.scope !== expectedScope) throw new Error('activate scope');
                 });
                 addEventListener('fetch', event => {
                   if (registration.scope !== expectedScope) throw new Error('fetch scope');
                   event.respondWith(new Response(registration.scope));
                 });",
                "https://example.test/app/sw.js",
                "https://example.test/app/",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));

        runtime.dispatch_install(81).unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::LifecycleSettled {
                event_id: 81,
                phase: ServiceWorkerLifecyclePhase::Install,
                succeeded: true,
                ..
            }
        ));

        runtime.dispatch_activate(82).unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::LifecycleSettled {
                event_id: 82,
                phase: ServiceWorkerLifecyclePhase::Activate,
                succeeded: true,
                ..
            }
        ));

        runtime
            .dispatch_fetch(
                83,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/page".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
                },
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 83,
                response: Some(ServiceWorkerFetchResponse { body, .. }),
                ..
            } if body == "https://example.test/app/"
        ));
    }

    #[test]
    fn service_worker_global_self_post_message_dispatches_message_event() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('message', event => {
                   if (!event.data || event.data.messageTest !== true) return;
                   if (event.source !== serviceWorker) throw new Error('wrong self source');
                   globalThis.__sawSelfMessage = true;
                 });
                 serviceWorker.postMessage({messageTest: true});",
                "https://example.test/sw.js",
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::Evaluated { .. }
        ));

        runtime
            .evaluate(
                "if (globalThis.__sawSelfMessage !== true) throw new Error('self message missing');",
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
                    if (!(event instanceof ExtendableMessageEvent)) throw new Error('wrong event');
                    globalThis.messageValue = event.data.name + ':' + event.data.items[1];
                    if (!(event.source instanceof WindowClient)) throw new Error('wrong source');
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
    fn page_message_timer_emits_client_message_after_dispatch() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('message', event => {
                   setTimeout(() => event.source.postMessage({delayed: event.data.name}), 0);
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_message(62, r#"{"name":"page"}"#, "client-1", "https://example.test/page")
            .unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::MessageDispatched {
                event_id: 62,
                client_id: "client-1".into(),
                outbound: Vec::new(),
            }
        );
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::ClientMessagesEmitted {
                outbound: vec![ServiceWorkerOutboundMessage {
                    data_json: r#"{"delayed":"page"}"#.into(),
                    port_id: None,
                    transferred_port_ids: Vec::new(),
                    data_port_index: None,
                    target_client_id: Some("client-1".into()),
                }],
            }
        );
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
    fn worker_registration_unregister_round_trips_through_host() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('message', event => {
                   registration.unregister().then(
                     result => event.source.postMessage({success: result}),
                     error => event.source.postMessage({success: false, exception: error.name})
                   );
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_message(21, "null", "client-1", "https://example.test/page")
            .unwrap();
        let ServiceWorkerEvent::UnregisterRequested { request_id } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing worker unregister request");
        };
        runtime.complete_unregister(request_id, Ok(true)).unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::MessageDispatched {
                event_id: 21,
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
    fn worker_message_dispatch_transfers_message_ports() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('message', event => {
                   if (event.data.ping) {
                     event.data.ping.postMessage({pong: 'OK'});
                   }
                 });",
                "https://example.test/pong.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();
        runtime
            .sync_registration_peers(
                8,
                ServiceWorkerRegistrationPeers {
                    active: Some(ServiceWorkerPeerInfo {
                        id: 8,
                        script_url: "https://example.test/pong.js".into(),
                        state: "activated".into(),
                    }),
                    ..Default::default()
                },
            )
            .unwrap();

        runtime
            .dispatch_worker_message(
                40,
                r#"{"ping":{"__zwServiceWorkerTransferredPortIndex":0}}"#,
                ServiceWorkerPeerInfo {
                    id: 7,
                    script_url: "https://example.test/ping.js".into(),
                    state: "activated".into(),
                },
                &ServiceWorkerMessagePorts {
                    transferred_port_ids: vec![2],
                    data_port_index: None,
                    target_port_id: None,
                },
                true,
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        let messages = loop {
            match runtime.recv_timeout(Duration::from_secs(5)).unwrap() {
                ServiceWorkerEvent::WorkerMessagesEmitted { messages } => break messages,
                ServiceWorkerEvent::MessageDispatched { .. } if Instant::now() < deadline => {}
                event => panic!("missing worker MessagePort reply: {event:?}"),
            }
        };
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].target_registration_id, 7);
        assert_eq!(messages[0].target_port_id, Some(2));
        assert_eq!(messages[0].data_json, r#"{"pong":"OK"}"#);

        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::MessageDispatched {
                event_id: 40,
                outbound,
                ..
            } if outbound.is_empty()
        ));
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
    fn message_dispatch_reports_clients_claim_request() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('message', event => {
                   clients.claim().then(() => {
                     event.source.postMessage('claimed');
                   });
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_message(22, "null", "client-1", "https://example.test/page")
            .unwrap();

        let mut claim_reported = false;
        let mut message_dispatched = false;
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !claim_reported || !message_dispatched {
            match runtime.recv_timeout(deadline.saturating_duration_since(std::time::Instant::now())) {
                Ok(ServiceWorkerEvent::ClientsClaimRequested) => {
                    claim_reported = true;
                }
                Ok(ServiceWorkerEvent::MessageDispatched {
                    event_id: 22, outbound, ..
                }) => {
                    assert_eq!(
                        outbound,
                        vec![ServiceWorkerOutboundMessage {
                            data_json: "\"claimed\"".into(),
                            port_id: None,
                            transferred_port_ids: Vec::new(),
                            data_port_index: None,
                            target_client_id: Some("client-1".into()),
                        }]
                    );
                    message_dispatched = true;
                }
                Ok(other) => panic!("unexpected runtime event: {other:?}"),
                Err(error) => panic!("runtime event timed out: {error}"),
            }
        }
    }

    #[test]
    fn message_dispatch_rejects_clients_claim_when_not_active() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('message', event => {
                   clients.claim().then(() => {
                     event.source.postMessage('PASS');
                   }, error => {
                     event.source.postMessage('FAIL: exception: ' + error.name);
                   });
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_message_with_ports_with_claim_allowed(
                23,
                "null",
                ServiceWorkerMessageSource {
                    client_id: "client-1",
                    client_url: "https://example.test/page",
                    client_frame_type: "top-level",
                    client_focused: true,
                },
                &ServiceWorkerMessagePorts::default(),
                false,
            )
            .unwrap();

        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::MessageDispatched {
                event_id: 23,
                client_id: "client-1".into(),
                outbound: vec![ServiceWorkerOutboundMessage {
                    data_json: "\"FAIL: exception: InvalidStateError\"".into(),
                    port_id: None,
                    transferred_port_ids: Vec::new(),
                    data_port_index: None,
                    target_client_id: Some("client-1".into()),
                }],
            }
        );
    }

    #[test]
    fn message_port_reply_after_rejected_clients_claim_when_not_active() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('message', event => {
                   clients.claim().then(() => {
                     event.data.port.postMessage('PASS');
                   }, error => {
                     event.data.port.postMessage('FAIL: exception: ' + error.name);
                   });
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_message_with_ports_with_claim_allowed(
                24,
                r#"{"port":{"__zwServiceWorkerTransferredPortIndex":0}}"#,
                ServiceWorkerMessageSource {
                    client_id: "client-1",
                    client_url: "https://example.test/page",
                    client_frame_type: "top-level",
                    client_focused: true,
                },
                &ServiceWorkerMessagePorts {
                    transferred_port_ids: vec![2],
                    data_port_index: None,
                    target_port_id: None,
                },
                false,
            )
            .unwrap();

        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::MessageDispatched {
                event_id: 24,
                client_id: "client-1".into(),
                outbound: vec![ServiceWorkerOutboundMessage {
                    data_json: "\"FAIL: exception: InvalidStateError\"".into(),
                    port_id: Some(2),
                    transferred_port_ids: Vec::new(),
                    data_port_index: None,
                    target_client_id: None,
                }],
            }
        );
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
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
    fn fetch_event_rejects_invalid_response_header_value() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith(new Promise(resolve => {
                     const headers = new Headers();
                     headers.append('foo', 'foo');
                     headers.append('foo', 'b\\0r');
                     resolve(new Response('bad', {headers}));
                   }));
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                44,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/invalid-header".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
                },
            )
            .unwrap();
        let event = runtime.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(matches!(
            event,
            ServiceWorkerEvent::FetchSettled {
                event_id: 44,
                response: None,
                failed: true,
                ref message,
                ..
            } if message.contains("invalid header value")
        ));
    }

    #[test]
    fn fetch_response_validation_rejects_invalid_header_wire_fields() {
        let invalid_name = ServiceWorkerFetchResponse {
            status: 200,
            status_text: "OK".into(),
            response_type: "default".into(),
            headers: vec![("bad name".into(), "ok".into())],
            body: String::new(),
        };
        assert!(validate_fetch_response(&invalid_name).is_err());

        let invalid_value = ServiceWorkerFetchResponse {
            status: 200,
            status_text: "OK".into(),
            response_type: "default".into(),
            headers: vec![("x-test".into(), "bad\r\nvalue".into())],
            body: String::new(),
        };
        assert!(validate_fetch_response(&invalid_value).is_err());
    }

    #[test]
    fn fetch_event_respond_with_serializes_buffer_source_and_form_data_response() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   const type = new URL(event.request.url).searchParams.get('type');
                   if (type === 'buffer') {
                     const bytes = new TextEncoder().encode('PASS');
                     event.respondWith(new Response(bytes.buffer));
                   } else if (type === 'buffer-view') {
                     event.respondWith(new Response(new Uint8Array([80, 65, 83, 83])));
                   } else if (type === 'form-data') {
                     const body = new FormData();
                     body.set('result', 'PASS');
                     event.respondWith(new Response(body));
                   }
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        for (event_id, response_type) in [(41, "buffer"), (42, "buffer-view")] {
            runtime
                .dispatch_fetch(
                    event_id,
                    ServiceWorkerFetchRequest {
                        url: format!("https://example.test/app/data.txt?type={response_type}"),
                        method: "GET".into(),
                        headers: Vec::new(),
                        body: None,
                        credentials: None,
                        client_id: Some("client-1".into()),
                        resulting_client_id: None,
                        referrer: None,
                        is_reload_navigation: false,
                        is_history_navigation: false,
                    },
                )
                .unwrap();
            assert_eq!(
                runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
                ServiceWorkerEvent::FetchSettled {
                    event_id,
                    request_url: format!("https://example.test/app/data.txt?type={response_type}"),
                    response: Some(ServiceWorkerFetchResponse {
                        status: 200,
                        status_text: String::new(),
                        response_type: "default".into(),
                        headers: Vec::new(),
                        body: "PASS".into(),
                    }),
                    failed: false,
                    message: String::new(),
                }
            );
        }

        runtime
            .dispatch_fetch(
                43,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/data.txt?type=form-data".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
                },
            )
            .unwrap();
        let event = runtime.recv_timeout(Duration::from_secs(5)).unwrap();
        let ServiceWorkerEvent::FetchSettled {
            response: Some(response),
            failed: false,
            ..
        } = event
        else {
            panic!("expected FormData response, got {event:?}");
        };
        assert_eq!(response.status, 200);
        assert!(
            response
                .headers
                .iter()
                .any(|(name, value)| name == "content-type" && value.starts_with("multipart/form-data; boundary=")),
            "Response(FormData) should set a multipart Content-Type: {:?}",
            response.headers
        );
        assert!(
            response
                .body
                .contains("Content-Disposition: form-data; name=\"result\"")
                && response.body.contains("PASS"),
            "Response(FormData) body should preserve the field: {}",
            response.body
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
                    credentials: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
                     if (!event.request.isReloadNavigation || event.request.isHistoryNavigation) {
                       throw new Error('event request navigation flags');
                     }
                     if (!requests[0].isReloadNavigation || requests[0].isHistoryNavigation) {
                       throw new Error('stored request navigation flags');
                     }
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
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: true,
                    is_history_navigation: false,
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
        assert!(request.is_reload_navigation);
        assert!(!request.is_history_navigation);
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
        assert!(request.is_reload_navigation);
        assert!(!request.is_history_navigation);
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
                    credentials: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: true,
                    is_history_navigation: false,
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
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
                    credentials: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
    fn storage_bucket_caches_use_prefixed_registration_cache_namespace() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith((async () => {
                     const inbox = await navigator.storageBuckets.open('inbox');
                     const cache = await inbox.caches.open('attachments');
                     await cache.put('receipt.txt', new Response('bread'));
                     await navigator.storageBuckets.delete('inbox');
                     try {
                       await cache.match('receipt.txt');
                       throw new Error('deleted bucket cache stayed live');
                     } catch (error) {
                       if (error.name !== 'UnknownError') throw error;
                     }
                     return new Response('done');
                   })());
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                48,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/bucket".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
                },
            )
            .unwrap();

        let bucket_cache_name = "__zw_storage_bucket__0069006e0062006f0078:attachments";
        let bucket_cache_name_units = "005f005f007a0077005f00730074006f0072006100670065005f006200750063006b00650074005f005f00300030003600390030003000360065003000300036003200300030003600660030003000370038003a006100740074006100630068006d0065006e00740073";

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing bucket CacheStorage.open request");
        };
        assert_eq!(
            request,
            ServiceWorkerCacheStorageRequest::Open {
                cache_name: bucket_cache_name.into()
            }
        );
        runtime
            .complete_cache_storage(
                request_id,
                Ok(ServiceWorkerCacheStorageResult::Open {
                    cache_name: bucket_cache_name.into(),
                    cache_name_units: bucket_cache_name_units.into(),
                    cache_id: 11,
                }),
            )
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing bucket Cache.put request");
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
        assert_eq!(cache_name, bucket_cache_name);
        assert_eq!(cache_id, Some(11));
        assert_eq!(request.url, "https://example.test/receipt.txt");
        assert_eq!(response.body, "bread");
        runtime
            .complete_cache_storage(request_id, Ok(ServiceWorkerCacheStorageResult::Done))
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing StorageBucketManager delete keys request");
        };
        assert_eq!(request, ServiceWorkerCacheStorageRequest::StorageKeys);
        runtime
            .complete_cache_storage(
                request_id,
                Ok(ServiceWorkerCacheStorageResult::StorageKeys(vec![
                    bucket_cache_name.into(),
                    "outside".into(),
                ])),
            )
            .unwrap();

        let ServiceWorkerEvent::CacheStorageRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing bucket CacheStorage.delete request");
        };
        assert_eq!(
            request,
            ServiceWorkerCacheStorageRequest::StorageDelete {
                cache_name: bucket_cache_name.into()
            }
        );
        runtime
            .complete_cache_storage(request_id, Ok(ServiceWorkerCacheStorageResult::Bool(true)))
            .unwrap();

        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 48,
                request_url: "https://example.test/app/bucket".into(),
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
    fn cache_delete_missing_argument_rejects_and_error_response_clone_roundtrips() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith((async () => {
                     const cache = await caches.open('runtime');
                     let deleteRejected = false;
                     try {
                       await cache.delete();
                     } catch (error) {
                       deleteRejected = error instanceof TypeError;
                     }
                     const cloned = Response.error().clone();
                     await cache.put(event.request, cloned);
                     return new Response(String(deleteRejected));
                   })());
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                48,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/error-clone".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
                    cache_id: 12,
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
            response,
            ..
        } = request
        else {
            panic!("expected Cache.put request");
        };
        assert_eq!(cache_name, "runtime");
        assert_eq!(cache_id, Some(12));
        assert_eq!(response.status, 0);
        assert_eq!(response.response_type, "error");
        runtime
            .complete_cache_storage(request_id, Ok(ServiceWorkerCacheStorageResult::Done))
            .unwrap();

        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 48,
                request_url: "https://example.test/app/error-clone".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "true".into(),
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
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
    fn cache_add_rejects_aborted_request_signal() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith((async () => {
                     const cache = await caches.open('runtime');
                     const alreadyController = new AbortController();
                     alreadyController.abort();
                     let already = 'resolved';
                     try {
                       await cache.add(new Request('./already.txt', {signal: alreadyController.signal}));
                     } catch (error) {
                       already = error.name + ':' + error.code;
                     }
                     const laterController = new AbortController();
                     const laterPromise = cache.add(new Request('./later.txt', {signal: laterController.signal}));
                     laterController.abort();
                     let later = 'resolved';
                     try {
                       await laterPromise;
                     } catch (error) {
                       later = error.name + ':' + error.code;
                     }
                     return new Response(already + '|' + later);
                   })());
                 });",
                "https://example.test/app/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                59,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/page".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
                    cache_id: 15,
                }),
            )
            .unwrap();

        let ServiceWorkerEvent::FetchRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing Cache.add fetch request");
        };
        assert_eq!(request.url, "https://example.test/app/later.txt");
        runtime
            .complete_fetch(
                request_id,
                Ok(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: "OK".into(),
                    response_type: "default".into(),
                    headers: vec![("content-type".into(), "text/plain".into())],
                    body: "later".into(),
                }),
            )
            .unwrap();

        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 59,
                request_url: "https://example.test/app/page".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "AbortError:20|AbortError:20".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn cache_add_preserves_signal_through_wrapped_fetch() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "const nativeFetch = globalThis.fetch;
                 globalThis.fetch = function(input, init) {
                   const signal = input && input.signal ? input.signal : init && init.signal;
                   if (String(input.url || input).includes('slow.txt')) {
                     return new Promise((resolve, reject) => {
                       signal.addEventListener('abort', () => reject(signal.reason));
                     });
                   }
                   return nativeFetch(input, init);
                 };
                 addEventListener('fetch', event => {
                   event.respondWith((async () => {
                     const cache = await caches.open('runtime');
                     const controller = new AbortController();
                     const request = new Request('./slow.txt', {signal: controller.signal});
                     const promise = cache.add(request);
                     controller.abort();
                     try {
                       await promise;
                       return new Response('resolved');
                     } catch (error) {
                       return new Response(error.name + ':' + error.code);
                     }
                   })());
                 });",
                "https://example.test/app/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                60,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/page".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
                    cache_id: 16,
                }),
            )
            .unwrap();

        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 60,
                request_url: "https://example.test/app/page".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "AbortError:20".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn cache_add_abort_after_polling_stash_open_rejects() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "const nativeFetch = globalThis.fetch;
                 const stash = {};
                 globalThis.fetch = function(input, init) {
                   const raw = input && typeof input === 'object' && input.url !== undefined ? input.url : input;
                   const url = new URL(String(raw), location.href);
                   const signal = input && input.signal ? input.signal : init && init.signal;
                   if (signal && signal.aborted) return Promise.reject(signal.reason);
                   if (url.pathname === '/fetch/api/resources/stash-take.py') {
                     const key = url.searchParams.get('key') || '';
                     const value = Object.prototype.hasOwnProperty.call(stash, key) ? stash[key] : null;
                     delete stash[key];
                     return Promise.resolve(new Response(JSON.stringify(value), {headers: {'content-type': 'application/json'}}));
                   }
                   if (url.pathname === '/fetch/api/resources/stash-put.py') {
                     const key = url.searchParams.get('key') || '';
                     if (key) stash[key] = 'done';
                     return Promise.resolve(new Response('done'));
                   }
                   if (url.pathname === '/fetch/api/resources/infinite-slow-response.py') {
                     const stateKey = url.searchParams.get('stateKey') || '';
                     if (stateKey) stash[stateKey] = 'open';
                     return new Promise((resolve, reject) => {
                       signal.addEventListener('abort', () => reject(signal.reason));
                     });
                   }
                   return nativeFetch(input, init);
                 };
                 addEventListener('fetch', event => {
                   event.respondWith((async () => {
                     const cache = await caches.open('runtime');
                     const controller = new AbortController();
                     const stateKey = 'state';
                     const request = new Request(
                       '../../../fetch/api/resources/infinite-slow-response.py?stateKey=' + stateKey,
                       {signal: controller.signal});
                     const promise = cache.add(request);
                     const response = await fetch('../../../fetch/api/resources/stash-take.py?key=' + stateKey);
                     const body = await response.json();
                     if (body !== 'open') return new Response('not-open:' + body);
                     await new Promise(resolve => setTimeout(resolve, 250));
                     controller.abort();
                     try {
                       await promise;
                       return new Response('resolved');
                     } catch (error) {
                       await fetch('../../../fetch/api/resources/stash-put.py?key=abort');
                       return new Response(error.name + ':' + error.code);
                     }
                   })());
                 });",
                "https://example.test/app/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                61,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/page".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
                    cache_id: 17,
                }),
            )
            .unwrap();

        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 61,
                request_url: "https://example.test/app/page".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "AbortError:20".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn cache_add_all_duplicate_request_rejects_before_fetch() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith((async () => {
                     const cache = await caches.open('runtime');
                     const request = new Request('./same.txt');
                     try {
                       await cache.addAll([request, request]);
                       return new Response('resolved');
                     } catch (error) {
                       return new Response(error.name);
                     }
                   })());
                 });",
                "https://example.test/app/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                57,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/page".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
                    cache_id: 13,
                }),
            )
            .unwrap();

        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 57,
                request_url: "https://example.test/app/page".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "InvalidStateError".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn cache_add_all_vary_duplicate_rejects_without_put() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith((async () => {
                     const cache = await caches.open('runtime');
                     const requests = [
                       new Request('./vary.txt', {headers: {'x-shape': 'circle', 'x-size': 'big'}}),
                       new Request('./vary.txt', {headers: {'x-shape': 'square', 'x-size': 'big'}})
                     ];
                     try {
                       await cache.addAll(requests);
                       return new Response('resolved');
                     } catch (error) {
                       return new Response(error.name);
                     }
                   })());
                 });",
                "https://example.test/app/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                58,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/page".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
                    cache_id: 14,
                }),
            )
            .unwrap();

        for expected_shape in ["circle", "square"] {
            let ServiceWorkerEvent::FetchRequested { request_id, request } =
                runtime.recv_timeout(Duration::from_secs(5)).unwrap()
            else {
                panic!("missing Cache.addAll fetch request");
            };
            assert_eq!(request.url, "https://example.test/app/vary.txt");
            assert_eq!(request.method, "GET");
            assert!(
                request
                    .headers
                    .iter()
                    .any(|(name, value)| name == "x-shape" && value == expected_shape)
            );
            assert!(
                request
                    .headers
                    .iter()
                    .any(|(name, value)| name == "x-size" && value == "big")
            );
            runtime
                .complete_fetch(
                    request_id,
                    Ok(ServiceWorkerFetchResponse {
                        status: 200,
                        status_text: "OK".into(),
                        response_type: "default".into(),
                        headers: vec![("vary".into(), "x-size".into())],
                        body: expected_shape.into(),
                    }),
                )
                .unwrap();
        }

        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 58,
                request_url: "https://example.test/app/page".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "InvalidStateError".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn worker_global_fetch_preserves_response_metadata_and_blob_surface() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith((async () => {
                     const response = await fetch('./asset.html');
                     const blob = await response.blob();
                     const sliceText = await new Promise((resolve) => {
                       const reader = new FileReader();
                       reader.onloadend = e => resolve(e.target.result);
                       reader.readAsText(blob.slice(1, 5));
                     });
                     return new Response([
                       response.url,
                       response.type,
                       String(response.headers.get('set-cookie')),
                       response.headers.get('content-type'),
                       blob.type,
                       sliceText
                     ].join('|'));
                   })());
                 });",
                "https://example.test/app/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                54,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/page".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
                },
            )
            .unwrap();
        let ServiceWorkerEvent::FetchRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing worker global fetch request");
        };
        assert_eq!(request.url, "https://example.test/app/asset.html");
        runtime
            .complete_fetch(
                request_id,
                Ok(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: "OK".into(),
                    response_type: "default".into(),
                    headers: vec![
                        ("content-type".into(), "text/html".into()),
                        ("set-cookie".into(), "a=1".into()),
                        ("x-zero-final-url".into(), "https://example.test/app/asset.html".into()),
                        ("x-zero-response-type".into(), "cors".into()),
                    ],
                    body: "abcdef".into(),
                }),
            )
            .unwrap();

        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 54,
                request_url: "https://example.test/app/page".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "https://example.test/app/asset.html|cors|null|text/html|text/html|bcde".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn worker_global_fetch_no_cors_cross_origin_returns_opaque_response() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith(fetch('https://remote.example/data.txt', {mode: 'no-cors'}).then(function(response) {
                     return new Response([
                       response.url,
                       response.type,
                       response.status,
                       response.ok,
                       response.headers.has('vary'),
                       String(response.headers.get('content-type'))
                     ].join('|'));
                   }));
                 });",
                "https://example.test/app/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                55,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/page".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
                },
            )
            .unwrap();
        let ServiceWorkerEvent::FetchRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing worker global fetch request");
        };
        assert_eq!(request.url, "https://remote.example/data.txt");
        runtime
            .complete_fetch(
                request_id,
                Ok(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: "OK".into(),
                    response_type: "default".into(),
                    headers: vec![
                        ("content-type".into(), "text/plain".into()),
                        ("vary".into(), "foo".into()),
                        ("x-zero-final-url".into(), "https://remote.example/data.txt".into()),
                    ],
                    body: "hidden".into(),
                }),
            )
            .unwrap();

        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 55,
                request_url: "https://example.test/app/page".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "https://remote.example/data.txt|opaque|0|false|false|null".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn worker_global_fetch_no_cors_request_object_after_url_hostname_mutation_is_opaque() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   event.respondWith((async () => {
                     const url = new URL('./data.txt', location.href);
                     url.hostname = 'remote.example';
                     const request = new Request(url, {mode: 'no-cors'});
                     const response = await fetch(request);
                     return new Response([
                       request.url,
                       response.type,
                       response.status,
                       response.ok,
                       String(response.headers.get('content-type'))
                     ].join('|'));
                   })());
                 });",
                "https://example.test/app/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                56,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/page".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
                },
            )
            .unwrap();
        let ServiceWorkerEvent::FetchRequested { request_id, request } =
            runtime.recv_timeout(Duration::from_secs(5)).unwrap()
        else {
            panic!("missing worker global fetch request");
        };
        assert_eq!(request.url, "https://remote.example/app/data.txt");
        runtime
            .complete_fetch(
                request_id,
                Ok(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: "OK".into(),
                    response_type: "default".into(),
                    headers: vec![
                        ("content-type".into(), "text/plain".into()),
                        ("x-zero-final-url".into(), "https://remote.example/app/data.txt".into()),
                    ],
                    body: "hidden".into(),
                }),
            )
            .unwrap();

        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 56,
                request_url: "https://example.test/app/page".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "https://remote.example/app/data.txt|opaque|0|false|null".into(),
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
                    credentials: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
                    credentials: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
                    credentials: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
                    credentials: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
    fn fetch_event_handled_reports_final_fetch_settlement() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "let port;
                 addEventListener('message', event => { port = event.data.port; });
                 addEventListener('fetch', event => {
                   event.handled.then(
                     () => port.postMessage('RESOLVED:' + new URL(event.request.url).search),
                     () => port.postMessage('REJECTED:' + new URL(event.request.url).search));
                   const search = new URL(event.request.url).search;
                   if (search === '?prevent-default') {
                     event.preventDefault();
                   } else if (search === '?invalid-response') {
                     event.respondWith(Promise.resolve('invalid response'));
                   }
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_message_with_ports(
                25,
                r#"{"port":{"__zwServiceWorkerTransferredPortIndex":0}}"#,
                "client-1",
                "https://example.test/page",
                &ServiceWorkerMessagePorts {
                    transferred_port_ids: vec![2],
                    data_port_index: None,
                    target_port_id: None,
                },
            )
            .unwrap();
        assert!(matches!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::MessageDispatched {
                event_id: 25,
                outbound,
                ..
            } if outbound.is_empty()
        ));

        for (event_id, query, expected) in [
            (57, "", "\"RESOLVED:\""),
            (58, "?prevent-default", "\"REJECTED:?prevent-default\""),
            (59, "?invalid-response", "\"REJECTED:?invalid-response\""),
        ] {
            runtime
                .dispatch_fetch(
                    event_id,
                    ServiceWorkerFetchRequest {
                        url: format!("https://example.test/app/data{query}"),
                        method: "GET".into(),
                        headers: Vec::new(),
                        body: None,
                        credentials: None,
                        client_id: Some("client-1".into()),
                        resulting_client_id: None,
                        referrer: None,
                        is_reload_navigation: false,
                        is_history_navigation: false,
                    },
                )
                .unwrap();

            let mut saw_fetch = false;
            let mut saw_handled = false;
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            while !saw_fetch || !saw_handled {
                match runtime.recv_timeout(deadline.saturating_duration_since(std::time::Instant::now())) {
                    Ok(ServiceWorkerEvent::FetchSettled {
                        event_id: settled_id,
                        failed,
                        ..
                    }) if settled_id == event_id => {
                        assert_eq!(failed, expected.contains("REJECTED"));
                        saw_fetch = true;
                    }
                    Ok(ServiceWorkerEvent::ClientMessagesEmitted { outbound }) => {
                        if outbound
                            .iter()
                            .any(|message| message.port_id == Some(2) && message.data_json == expected)
                        {
                            saw_handled = true;
                        }
                    }
                    Ok(other) => panic!("unexpected runtime event: {other:?}"),
                    Err(error) => panic!("runtime event timed out: {error}"),
                }
            }
        }
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
                    credentials: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
    fn fetch_event_readable_stream_body_error_is_serialized() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   let counter = 0;
                   const stream = new ReadableStream({ pull: controller => {
                     counter += 1;
                     if (counter === 1) {
                       controller.enqueue(new Uint8Array([80, 65, 83, 83]));
                     } else {
                       setTimeout(() => controller.error('stream failed'), 0);
                     }
                   }});
                   event.respondWith(new Response(stream));
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                53,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/page".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
                },
            )
            .unwrap();
        let event = runtime.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(
            matches!(
                event,
                ServiceWorkerEvent::FetchSettled {
                event_id: 53,
                response: Some(ServiceWorkerFetchResponse { ref headers, ref body, .. }),
                failed: false,
                ..
            } if body.is_empty()
                && headers.iter().any(|(name, value)| name == "x-zero-body-error" && value == "stream failed")
            ),
            "unexpected event: {event:?}"
        );
    }

    #[test]
    fn fetch_event_readable_stream_start_body_is_serialized() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   const encoder = new TextEncoder();
                   const stream = new ReadableStream({ start: controller => {
                     controller.enqueue(encoder.encode('PASS'));
                     controller.close();
                   }});
                   event.respondWith(new Response(stream));
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                53,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/body-stream-start".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
                },
            )
            .unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 53,
                request_url: "https://example.test/app/body-stream-start".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "PASS".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn fetch_event_readable_stream_body_chunks_are_serialized() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   let counter = 0;
                   const encoder = new TextEncoder();
                   const stream = new ReadableStream({ pull: controller => {
                     counter += 1;
                     if (counter === 1) {
                       controller.enqueue(encoder.encode('chunk #1'));
                     } else if (counter === 2) {
                       controller.enqueue(encoder.encode(' chunk #2'));
                     } else {
                       controller.close();
                     }
                   }});
                   event.respondWith(new Response(stream));
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                54,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/body-stream".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
                },
            )
            .unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 54,
                request_url: "https://example.test/app/body-stream".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "chunk #1 chunk #2".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn fetch_event_readable_stream_invalid_chunk_errors_body() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "addEventListener('fetch', event => {
                   const stream = new ReadableStream({ start: controller => {
                     controller.enqueue('not bytes');
                   }});
                   event.respondWith(new Response(stream));
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                55,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/body-stream-invalid-chunk".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
                },
            )
            .unwrap();
        let event = runtime.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(
            matches!(
                event,
                ServiceWorkerEvent::FetchSettled {
                    event_id: 55,
                    response: Some(ServiceWorkerFetchResponse { ref headers, ref body, .. }),
                    failed: false,
                    ..
                } if body.is_empty()
                    && headers.iter().any(|(name, value)| {
                        name == "x-zero-body-error" && value.contains("Uint8Array")
                    })
            ),
            "unexpected event: {event:?}"
        );
    }

    #[test]
    fn fetch_event_respond_with_stops_later_listeners() {
        let mut runtime = ServiceWorkerRuntime::new(test_config()).unwrap();
        runtime
            .evaluate(
                "let result = 'unset';
                 addEventListener('fetch', event => {
                   if (result === 'unset') result = 'PASS';
                   event.respondWith(new Response(result));
                 });
                 addEventListener('fetch', () => {
                   result = 'FAIL: fetch event propagated';
                 });",
                "https://example.test/sw.js",
            )
            .unwrap();
        let _ = runtime.recv_timeout(Duration::from_secs(5)).unwrap();

        runtime
            .dispatch_fetch(
                53,
                ServiceWorkerFetchRequest {
                    url: "https://example.test/app/page".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: None,
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
                },
            )
            .unwrap();
        assert_eq!(
            runtime.recv_timeout(Duration::from_secs(5)).unwrap(),
            ServiceWorkerEvent::FetchSettled {
                event_id: 53,
                request_url: "https://example.test/app/page".into(),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "PASS".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
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
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                    is_reload_navigation: false,
                    is_history_navigation: false,
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
