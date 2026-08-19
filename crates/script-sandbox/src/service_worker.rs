//! Typed Service Worker script runtime.

use crate::threaded_runtime::ThreadedRuntimeCore;
use crate::{Sandbox, SandboxConfig, ScriptError};
use std::sync::mpsc;

const ENGINE_INIT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
const MAX_HEAP_BYTES: usize = 64 * 1024 * 1024;
const DEFAULT_SCRIPT_TIMEOUT_MS: u64 = 5_000;

enum ServiceWorkerCommand {
    Evaluate {
        script: String,
        script_url: String,
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

  globalThis.self = globalThis;
  globalThis.ServiceWorkerGlobalScope = function ServiceWorkerGlobalScope() {};
  globalThis.ExtendableEvent = ExtendableEvent;
  globalThis.InstallEvent = InstallEvent;
  globalThis.MessageEvent = MessageEvent;
  globalThis.Client = Client;
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
    /// The runtime thread exited.
    Closed,
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
}

impl ServiceWorkerRuntime {
    /// Start a Service Worker engine thread and wait for engine initialization.
    pub fn new(config: SandboxConfig) -> Result<Self, ScriptError> {
        let config = normalize_config(config);
        let lifecycle_timeout_ms = config.timeout_ms;
        let (init_sender, init_receiver) = mpsc::sync_channel(1);
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
                if let Err(error) = sandbox.execute(SERVICE_WORKER_BOOTSTRAP) {
                    let _ = init_sender.send(Err(error));
                    return;
                }
                let _ = init_sender.send(Ok(()));

                while let Ok(command) = command_receiver.recv() {
                    match command {
                        ServiceWorkerCommand::Evaluate { script, script_url } => {
                            let source = if script.trim().is_empty() { ";" } else { script.as_str() };
                            let event = match sandbox.execute(source) {
                                Ok(_) => ServiceWorkerEvent::Evaluated { script_url },
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
            Ok(Ok(())) => Ok(Self { core }),
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

    /// Shut down the engine thread with a bounded join.
    pub fn shutdown(&mut self) {
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

fn script_error_kind(error: &ScriptError) -> ServiceWorkerScriptErrorKind {
    match error {
        ScriptError::CompileError(_) => ServiceWorkerScriptErrorKind::Compile,
        ScriptError::RuntimeError(_) | ScriptError::NotInitialized => ServiceWorkerScriptErrorKind::Runtime,
        ScriptError::Timeout(_) => ServiceWorkerScriptErrorKind::Timeout,
        ScriptError::InvalidInput(_) => ServiceWorkerScriptErrorKind::InvalidInput,
        ScriptError::EngineUnavailable(_) => ServiceWorkerScriptErrorKind::EngineUnavailable,
    }
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
    const MAX_OUTBOUND_MESSAGES: usize = 64;
    const MAX_MESSAGE_BYTES: usize = 1024 * 1024;
    let result = sandbox.execute("JSON.stringify(globalThis.__zwTakeOutboundMessages())")?;
    let values = serde_json::from_str::<Vec<serde_json::Value>>(&result.value)
        .map_err(|error| ScriptError::RuntimeError(format!("invalid outbound message list: {error}")))?;
    if values.len() > MAX_OUTBOUND_MESSAGES {
        return Err(ScriptError::InvalidInput(
            "Service Worker emitted too many messages in one event".into(),
        ));
    }
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
}
