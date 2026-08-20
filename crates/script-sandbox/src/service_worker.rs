//! Typed Service Worker script runtime.

use crate::threaded_runtime::ThreadedRuntimeCore;
use crate::{
    ModuleRegistry, Sandbox, SandboxConfig, ScriptError, compile_module_script, extract_dynamic_import_specifiers,
    extract_static_module_import_specifiers,
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
    },
    Shutdown,
}

enum ServiceWorkerImportResponse {
    Completed { request_id: u64, sources: Vec<String> },
    Failed { request_id: u64, message: String },
    Shutdown,
}

const SERVICE_WORKER_BOOTSTRAP: &str = r#"
(function() {
  const listeners = Object.create(null);
  let currentWaitUntil = null;
  let skipWaitingRequested = false;
  let claimClientsRequested = false;

  class ExtendableEvent {
    constructor(type) { this.type = type; }
    waitUntil(value) {
      if (typeof currentWaitUntil !== 'function') {
        throw new Error('InvalidStateError: waitUntil called outside dispatch');
      }
      currentWaitUntil(value);
    }
  }
  class InstallEvent extends ExtendableEvent {}
  const outboundMessages = [];
  const clientToken = {};
  class Client {
    constructor(id, url, token) {
      if (token !== clientToken) throw new TypeError('Illegal constructor');
      Object.defineProperties(this, {
        id: {value: id, enumerable: true},
        url: {value: url, enumerable: true},
        type: {value: 'window', enumerable: true},
        frameType: {value: 'top-level', enumerable: true}
      });
    }
    postMessage(data) {
      const dataJSON = JSON.stringify(data);
      if (dataJSON === undefined) throw new Error('DataCloneError: message could not be cloned');
      outboundMessages.push({dataJSON: dataJSON});
    }
  }
  class MessageEvent {
    constructor(type, init) {
      this.type = type;
      this.data = init.data;
      this.origin = '';
      this.source = null;
      this.ports = [];
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
  globalThis.DOMException = globalThis.DOMException || DOMException;
  globalThis.URLSearchParams = globalThis.URLSearchParams || URLSearchParams;
  globalThis.WorkerLocation = WorkerLocation;
  globalThis.Client = Client;
  globalThis.oninstall = null;
  globalThis.onactivate = null;
  globalThis.onmessage = null;
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
  class Clients {
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
  function importScriptsNetworkError(message) {
    return new globalThis.DOMException(String(message), 'NetworkError');
  }
  globalThis.importScripts = function() {
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
  globalThis.__zwDispatchMessage = function(eventId, data, clientId, clientURL) {
    outboundMessages.splice(0, outboundMessages.length);
    const event = new MessageEvent('message', {data: data});
    event.source = new Client(clientId, clientURL, clientToken);
    try {
      const callbacks = (listeners.message || []).slice();
      for (let i = 0; i < callbacks.length; i++) callbacks[i].call(globalThis, event);
      if (typeof globalThis.onmessage === 'function') {
        globalThis.onmessage.call(globalThis, event);
      }
      return String(eventId);
    } catch (error) {
      outboundMessages.splice(0, outboundMessages.length);
      throw error;
    }
  };
  globalThis.__zwTakeOutboundMessages = function() {
    return outboundMessages.splice(0, outboundMessages.length);
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
    /// A classic worker `importScripts()` call requires host-owned fetching.
    ImportScriptsRequested {
        /// Runtime-local request ID used to correlate the blocking response.
        request_id: u64,
        /// String-converted URL arguments in call order.
        specifiers: Vec<String>,
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
}

impl ServiceWorkerRuntime {
    /// Start a Service Worker engine thread and wait for engine initialization.
    pub fn new(config: SandboxConfig) -> Result<Self, ScriptError> {
        let config = normalize_config(config);
        let lifecycle_timeout_ms = config.timeout_ms;
        let (init_sender, init_receiver) = mpsc::sync_channel(1);
        let (import_response_sender, import_response_receiver) = mpsc::channel();
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
                if let Err(error) = sandbox.execute(SERVICE_WORKER_BOOTSTRAP) {
                    let _ = init_sender.send(Err(error));
                    return;
                }
                let _ = init_sender.send(Ok(()));

                while let Ok(command) = command_receiver.recv() {
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
                                Ok(()) => ServiceWorkerEvent::Evaluated { script_url },
                                Err(error) => ServiceWorkerEvent::ScriptError {
                                    script_url,
                                    kind: script_error_kind(&error),
                                    message: error.to_string(),
                                },
                            };
                            let _ = event_sender.send(event);
                        }
                        ServiceWorkerCommand::DispatchLifecycle { event_id, phase } => {
                            let event = dispatch_lifecycle(sandbox.as_mut(), event_id, phase, lifecycle_timeout_ms);
                            let _ = event_sender.send(event);
                        }
                        ServiceWorkerCommand::DispatchMessage {
                            event_id,
                            data_json,
                            client_id,
                            client_url,
                        } => {
                            let dispatch = format!(
                                "globalThis.__zwDispatchMessage({}, {}, {}, {});",
                                event_id,
                                data_json,
                                serde_json::to_string(&client_id).unwrap(),
                                serde_json::to_string(&client_url).unwrap()
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
        serde_json::from_str::<serde_json::Value>(data_json)
            .map_err(|error| ScriptError::InvalidInput(format!("invalid Service Worker message JSON: {error}")))?;
        self.core
            .send(ServiceWorkerCommand::DispatchMessage {
                event_id,
                data_json: data_json.to_string(),
                client_id: client_id.to_string(),
                client_url: client_url.to_string(),
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

    /// Shut down the engine thread with a bounded join.
    pub fn shutdown(&mut self) {
        let _ = self.import_response_sender.send(ServiceWorkerImportResponse::Shutdown);
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

fn import_failure_json(message: &str) -> String {
    serde_json::json!({"ok": false, "error": message}).to_string()
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

fn dispatch_lifecycle(
    sandbox: &mut dyn Sandbox,
    event_id: u64,
    phase: ServiceWorkerLifecyclePhase,
    timeout_ms: u64,
) -> ServiceWorkerEvent {
    let dispatch = format!(
        "globalThis.__zwDispatchLifecycle({}, {}); 'dispatched';",
        serde_json::to_string(phase.as_str()).expect("static phase is serializable"),
        event_id
    );
    if let Err(error) = sandbox.execute(&dispatch) {
        return ServiceWorkerEvent::LifecycleSettled {
            event_id,
            phase,
            succeeded: false,
            skip_waiting: false,
            claim_clients: false,
            message: error.to_string(),
        };
    }

    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
    loop {
        match sandbox.execute("JSON.stringify(globalThis.__zwLifecycleResult)") {
            Ok(result) => match serde_json::from_str::<serde_json::Value>(&result.value) {
                Ok(value) if value["settled"].as_bool() == Some(true) => {
                    return ServiceWorkerEvent::LifecycleSettled {
                        event_id,
                        phase,
                        succeeded: value["succeeded"].as_bool() == Some(true),
                        skip_waiting: value["skipWaitingRequested"].as_bool() == Some(true),
                        claim_clients: value["claimClientsRequested"].as_bool() == Some(true),
                        message: value["message"].as_str().unwrap_or_default().to_string(),
                    };
                }
                Ok(_) => {}
                Err(error) => {
                    return ServiceWorkerEvent::LifecycleSettled {
                        event_id,
                        phase,
                        succeeded: false,
                        skip_waiting: false,
                        claim_clients: false,
                        message: format!("invalid lifecycle result: {error}"),
                    };
                }
            },
            Err(error) => {
                return ServiceWorkerEvent::LifecycleSettled {
                    event_id,
                    phase,
                    succeeded: false,
                    skip_waiting: false,
                    claim_clients: false,
                    message: error.to_string(),
                };
            }
        }
        if std::time::Instant::now() >= deadline {
            return ServiceWorkerEvent::LifecycleSettled {
                event_id,
                phase,
                succeeded: false,
                skip_waiting: false,
                claim_clients: false,
                message: format!("lifecycle event exceeded {timeout_ms}ms"),
            };
        }
        let _ = sandbox.execute("'checkpoint'");
        std::thread::sleep(std::time::Duration::from_millis(1));
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
            Ok(ServiceWorkerOutboundMessage { data_json })
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
}
