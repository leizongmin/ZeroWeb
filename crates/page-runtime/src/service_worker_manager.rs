//! Service Worker registration and lifecycle coordination.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use zero_script_sandbox::{
    SandboxConfig, ServiceWorkerEvent, ServiceWorkerLifecyclePhase, ServiceWorkerRuntime, ServiceWorkerScriptErrorKind,
};
use zero_storage::{
    ServiceWorkerRegistration, ServiceWorkerRegistry, ServiceWorkerScriptType, ServiceWorkerState,
    ServiceWorkerUpdateViaCache,
};

const DEFAULT_RUNTIME_LIMIT: usize = 32;
const MAX_CLIENTS_PER_VERSION: usize = 256;
const MAX_MESSAGES_PER_CLIENT: usize = 1024;
const MAX_URL_BYTES: usize = 64 * 1024;
const MAX_SCRIPT_BYTES: usize = 16 * 1024 * 1024;
const MAX_SCRIPT_GRAPH_BYTES: usize = 64 * 1024 * 1024;
const MAX_IMPORTED_SCRIPTS_PER_VERSION: usize = 1024;

struct EvaluationOptions {
    update_via_cache: ServiceWorkerUpdateViaCache,
    script_type: ServiceWorkerScriptType,
    restoring_active: bool,
    restored_imported_scripts: Vec<ServiceWorkerImportedScript>,
}

/// Stable registration key within one storage partition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceWorkerRegistrationKey {
    /// Serialized origin (`scheme://host:port`).
    pub origin: String,
    /// Normalized registration scope.
    pub scope: String,
}

impl ServiceWorkerRegistrationKey {
    /// Construct a registration key after basic non-empty validation.
    pub fn new(origin: &str, scope: &str) -> Result<Self, ServiceWorkerManagerError> {
        if origin.trim().is_empty() {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker origin is empty".into(),
            ));
        }
        if scope.trim().is_empty() {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker scope is empty".into(),
            ));
        }
        if origin.len() > MAX_URL_BYTES || scope.len() > MAX_URL_BYTES {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker origin or scope exceeds the length limit".into(),
            ));
        }
        Ok(Self {
            origin: origin.to_string(),
            scope: scope.to_string(),
        })
    }
}

/// Observable version slots for one registration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ServiceWorkerVersionSlots {
    /// Version currently evaluating or running its install event.
    pub installing: Option<u64>,
    /// Installed version waiting for activation.
    pub waiting: Option<u64>,
    /// Version currently active for this key.
    pub active: Option<u64>,
}

/// Result of comparing a fetched update script with the current version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceWorkerUpdateOutcome {
    /// The fetched script is byte-for-byte identical to the current version.
    Unchanged {
        /// Existing registration version ID.
        registration_id: u64,
    },
    /// A changed script started a new installing version.
    Started {
        /// New registration version ID.
        registration_id: u64,
    },
}

/// Persistable active registration inputs owned by the browser process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceWorkerPersistentRegistration {
    /// Canonical top-level script URL.
    pub script_url: String,
    /// Canonical registration scope.
    pub scope: String,
    /// Serialized origin.
    pub origin: String,
    /// UTF-8 top-level script source used to recreate the runtime.
    pub script_source: String,
    /// Script update HTTP cache policy.
    #[serde(default)]
    pub update_via_cache: ServiceWorkerUpdateViaCache,
    /// Top-level script type.
    #[serde(default)]
    pub script_type: ServiceWorkerScriptType,
    /// Canonical imported classic script URLs and source bytes.
    #[serde(default)]
    pub imported_scripts: Vec<ServiceWorkerImportedScript>,
}

/// One imported classic script or static module dependency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceWorkerImportedScript {
    /// Canonical request URL with no fragment.
    pub url: String,
    /// UTF-8 script source.
    pub source: String,
}

/// Typed manager event produced while polling worker runtimes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceWorkerManagerEvent {
    /// Top-level script evaluation completed.
    ScriptEvaluated {
        /// Registration version ID.
        registration_id: u64,
    },
    /// A persisted active version rebuilt its runtime without lifecycle events.
    RestorationCompleted {
        /// Recreated registration version ID.
        registration_id: u64,
    },
    /// Evaluation or runtime execution failed.
    ScriptFailed {
        /// Registration version ID.
        registration_id: u64,
        /// Stable failure category.
        kind: ServiceWorkerScriptErrorKind,
        /// Engine diagnostic that does not contain script source.
        message: String,
    },
    /// Runtime lifecycle dispatch and all `waitUntil()` promises settled.
    LifecycleSettled {
        /// Registration version ID.
        registration_id: u64,
        /// Install or activate phase.
        phase: ServiceWorkerLifecyclePhase,
        /// Whether dispatch and all lifetime promises fulfilled.
        succeeded: bool,
        /// Whether the worker requested immediate activation.
        skip_waiting: bool,
        /// Whether the worker requested control of matching clients.
        claim_clients: bool,
        /// Rejection or dispatch error diagnostic.
        message: String,
    },
    /// Manager could not apply or dispatch an internal lifecycle transition.
    CoordinationFailed {
        /// Registration version ID.
        registration_id: u64,
        /// Internal transition diagnostic.
        message: String,
    },
    /// Install result moved the version to waiting or redundant.
    InstallCompleted {
        /// Registration version ID.
        registration_id: u64,
        /// Whether all install lifetime promises fulfilled.
        succeeded: bool,
    },
    /// Activate result moved the version to active or redundant.
    ActivationCompleted {
        /// Registration version ID.
        registration_id: u64,
        /// Whether all activate lifetime promises fulfilled.
        succeeded: bool,
    },
    /// A page-to-worker message event was dispatched.
    MessageDispatched {
        /// Registration version ID.
        registration_id: u64,
        /// Host-assigned message event ID.
        event_id: u64,
        /// Number of worker-to-client messages emitted by the handler.
        outbound_count: usize,
    },
    /// A page-to-worker message handler threw.
    MessageFailed {
        /// Registration version ID.
        registration_id: u64,
        /// Host-assigned message event ID.
        event_id: u64,
        /// Browser-owned identity of the originating client.
        client_id: String,
        /// Handler diagnostic.
        message: String,
    },
    /// A runtime is blocked in `importScripts()` pending host-owned fetches.
    ImportScriptsRequested {
        /// Registration version ID.
        registration_id: u64,
        /// Runtime-local request ID.
        request_id: u64,
        /// Canonical script URLs in execution order.
        urls: Vec<String>,
        /// Whether these requests must bypass a fresh HTTP cache entry.
        bypass_cache: bool,
    },
    /// A module worker is blocked while its static dependency graph is fetched.
    ModuleScriptsRequested {
        /// Registration version ID.
        registration_id: u64,
        /// Runtime-local request ID.
        request_id: u64,
        /// Canonical module URLs in source order.
        urls: Vec<String>,
        /// Whether these requests must bypass a fresh HTTP cache entry.
        bypass_cache: bool,
    },
    /// A complete update candidate graph was compared with the current version.
    UpdateChecked {
        /// Installing candidate version ID.
        candidate_registration_id: u64,
        /// Version ID returned to the page API.
        registration_id: u64,
        /// Whether main or imported script bytes changed.
        changed: bool,
    },
}

/// Service Worker manager operation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceWorkerManagerError {
    /// Host input failed validation.
    InvalidInput(String),
    /// Another installing version already owns this registration key.
    JobInProgress(u64),
    /// Install completion was requested before script evaluation completed.
    EvaluationPending(u64),
    /// The manager reached its live runtime budget.
    CapacityExceeded {
        /// Maximum number of live runtimes.
        limit: usize,
    },
    /// One worker version reached its tracked client budget.
    ClientCapacityExceeded {
        /// Maximum tracked clients per version.
        limit: usize,
    },
    /// One client reached its retained message-event budget.
    ClientMessageCapacityExceeded {
        /// Maximum retained event batches per client.
        limit: usize,
    },
    /// The requested version does not exist.
    UnknownRegistration(u64),
    /// The operation does not match the version's current lifecycle state.
    InvalidState {
        /// Registration version ID.
        registration_id: u64,
        /// Required lifecycle state.
        expected: ServiceWorkerState,
        /// Actual lifecycle state.
        actual: ServiceWorkerState,
    },
    /// The script runtime could not be started or commanded.
    Runtime(String),
}

impl std::fmt::Display for ServiceWorkerManagerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(formatter, "invalid input: {message}"),
            Self::JobInProgress(id) => {
                write!(formatter, "registration {id} is already installing")
            }
            Self::EvaluationPending(id) => {
                write!(formatter, "registration {id} script evaluation is pending")
            }
            Self::CapacityExceeded { limit } => {
                write!(formatter, "Service Worker runtime limit reached ({limit})")
            }
            Self::ClientCapacityExceeded { limit } => {
                write!(formatter, "Service Worker client limit reached ({limit})")
            }
            Self::ClientMessageCapacityExceeded { limit } => {
                write!(formatter, "Service Worker client message limit reached ({limit})")
            }
            Self::UnknownRegistration(id) => {
                write!(formatter, "registration {id} does not exist")
            }
            Self::InvalidState {
                registration_id,
                expected,
                actual,
            } => write!(
                formatter,
                "registration {registration_id} is {actual}, expected {expected}"
            ),
            Self::Runtime(message) => write!(formatter, "runtime error: {message}"),
        }
    }
}

impl std::error::Error for ServiceWorkerManagerError {}

/// Single owner for registration slots and Service Worker runtimes.
///
/// This manager owns the storage registry instance. Callers can inspect
/// registrations but cannot mutate lifecycle state outside manager methods.
pub struct ServiceWorkerManager {
    registry: ServiceWorkerRegistry,
    slots: HashMap<ServiceWorkerRegistrationKey, ServiceWorkerVersionSlots>,
    registration_keys: HashMap<u64, ServiceWorkerRegistrationKey>,
    state_changes: HashMap<u64, Vec<ServiceWorkerState>>,
    claimed_clients: HashSet<u64>,
    client_messages: HashMap<(u64, String), Vec<Vec<String>>>,
    pending_client_messages: HashMap<(u64, String), usize>,
    script_sources: HashMap<u64, Vec<u8>>,
    imported_scripts: HashMap<u64, HashMap<String, Vec<u8>>>,
    pending_import_requests: HashMap<(u64, u64), Vec<String>>,
    update_predecessors: HashMap<u64, u64>,
    host: Box<dyn ServiceWorkerRuntimeHost>,
    evaluated: HashSet<u64>,
    restoring_active: HashSet<u64>,
    runtime_limit: usize,
}

/// Engine-runtime operations delegated by [`ServiceWorkerManager`].
///
/// The manager owns registration state only; worker runtimes live behind this
/// trait. [`LocalServiceWorkerHost`] runs script-sandbox engine threads
/// in-process (webview / WPT / tests). The multi-process browser implements
/// the trait over IPC so script evaluation happens in renderer processes and
/// the browser binary links no JavaScript engine.
pub trait ServiceWorkerRuntimeHost: Send {
    /// Spawn the runtime for `registration_id` and queue `script` evaluation.
    fn evaluate(
        &mut self,
        registration_id: u64,
        script_url: &str,
        script: &str,
        script_type: ServiceWorkerScriptType,
    ) -> Result<(), ServiceWorkerManagerError>;
    /// Dispatch the install or activate event inside one live runtime.
    fn dispatch_lifecycle(
        &mut self,
        registration_id: u64,
        phase: ServiceWorkerLifecyclePhase,
    ) -> Result<(), ServiceWorkerManagerError>;
    /// Dispatch one JSON-compatible page message into a live runtime.
    fn dispatch_client_message(
        &mut self,
        registration_id: u64,
        event_id: u64,
        data_json: &str,
        client_id: &str,
        client_url: &str,
    ) -> Result<(), ServiceWorkerManagerError>;
    /// Complete one blocking `importScripts()` request.
    fn complete_import_scripts(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<Vec<String>, String>,
    ) -> Result<(), ServiceWorkerManagerError>;
    /// Stop one runtime and release its resources.
    fn shutdown(&mut self, registration_id: u64);
    /// Drain all currently available runtime events.
    fn poll_events(&mut self) -> Vec<(u64, ServiceWorkerEvent)>;
    /// Number of live runtimes, for capacity accounting.
    fn runtime_count(&self) -> usize;
}

/// In-process [`ServiceWorkerRuntimeHost`] backed by script-sandbox engine threads.
pub struct LocalServiceWorkerHost {
    config: SandboxConfig,
    runtimes: HashMap<u64, ServiceWorkerRuntime>,
}

impl LocalServiceWorkerHost {
    /// Create a host whose runtimes evaluate with `config`.
    pub fn new(config: SandboxConfig) -> Self {
        Self {
            config,
            runtimes: HashMap::new(),
        }
    }
}

impl ServiceWorkerRuntimeHost for LocalServiceWorkerHost {
    fn evaluate(
        &mut self,
        registration_id: u64,
        script_url: &str,
        script: &str,
        script_type: ServiceWorkerScriptType,
    ) -> Result<(), ServiceWorkerManagerError> {
        let mut runtime = ServiceWorkerRuntime::new(self.config.clone())
            .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))?;
        match script_type {
            ServiceWorkerScriptType::Classic => runtime.evaluate(script, script_url),
            ServiceWorkerScriptType::Module => runtime.evaluate_module(script, script_url),
        }
        .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))?;
        self.runtimes.insert(registration_id, runtime);
        Ok(())
    }

    fn dispatch_lifecycle(
        &mut self,
        registration_id: u64,
        phase: ServiceWorkerLifecyclePhase,
    ) -> Result<(), ServiceWorkerManagerError> {
        let runtime = self
            .runtimes
            .get_mut(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        let result = match phase {
            ServiceWorkerLifecyclePhase::Install => runtime.dispatch_install(registration_id),
            ServiceWorkerLifecyclePhase::Activate => runtime.dispatch_activate(registration_id),
        };
        result.map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))
    }

    fn dispatch_client_message(
        &mut self,
        registration_id: u64,
        event_id: u64,
        data_json: &str,
        client_id: &str,
        client_url: &str,
    ) -> Result<(), ServiceWorkerManagerError> {
        let runtime = self
            .runtimes
            .get_mut(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        runtime
            .dispatch_message(event_id, data_json, client_id, client_url)
            .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))
    }

    fn complete_import_scripts(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<Vec<String>, String>,
    ) -> Result<(), ServiceWorkerManagerError> {
        self.runtimes
            .get(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?
            .complete_import_scripts(request_id, result)
            .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))
    }

    fn shutdown(&mut self, registration_id: u64) {
        // 不在此处移除：poll_events 先 drain `Closed` 事件再按 is_running 回收槽位
        // （提前移除会让 Closed 成为孤儿事件，manager 无法据此推进状态机）。
        if let Some(runtime) = self.runtimes.get_mut(&registration_id) {
            runtime.shutdown();
        }
    }

    fn poll_events(&mut self) -> Vec<(u64, ServiceWorkerEvent)> {
        let mut pending = Vec::new();
        for (&registration_id, runtime) in &self.runtimes {
            while let Some(event) = runtime.try_recv() {
                pending.push((registration_id, event));
            }
        }
        self.runtimes.retain(|_, runtime| runtime.is_running());
        pending
    }

    fn runtime_count(&self) -> usize {
        self.runtimes.len()
    }
}

impl Drop for LocalServiceWorkerHost {
    fn drop(&mut self) {
        for runtime in self.runtimes.values_mut() {
            runtime.shutdown();
        }
    }
}

impl ServiceWorkerManager {
    /// Create an empty manager with in-process runtimes and the default
    /// sandbox configuration.
    pub fn new() -> Self {
        Self::with_local_host(SandboxConfig::default())
    }

    /// Create an empty manager whose runtimes are delegated to `host`.
    pub fn with_host(host: Box<dyn ServiceWorkerRuntimeHost>) -> Self {
        Self {
            registry: ServiceWorkerRegistry::new(),
            slots: HashMap::new(),
            registration_keys: HashMap::new(),
            state_changes: HashMap::new(),
            claimed_clients: HashSet::new(),
            client_messages: HashMap::new(),
            pending_client_messages: HashMap::new(),
            script_sources: HashMap::new(),
            imported_scripts: HashMap::new(),
            pending_import_requests: HashMap::new(),
            update_predecessors: HashMap::new(),
            host,
            evaluated: HashSet::new(),
            restoring_active: HashSet::new(),
            runtime_limit: DEFAULT_RUNTIME_LIMIT,
        }
    }

    /// Create an empty manager with in-process runtimes using `config`.
    pub fn with_local_host(config: SandboxConfig) -> Self {
        Self::with_host(Box::new(LocalServiceWorkerHost::new(config)))
    }

    /// Start evaluating one fetched script as the installing version.
    ///
    /// URL fetching and security validation remain host responsibilities in
    /// M1-2. The manager rejects overlapping jobs for the same origin/scope.
    pub fn start_evaluation(
        &mut self,
        script_url: &str,
        scope: &str,
        origin: &str,
        script: &str,
    ) -> Result<u64, ServiceWorkerManagerError> {
        self.start_evaluation_with_update_via_cache(
            script_url,
            scope,
            origin,
            script,
            ServiceWorkerUpdateViaCache::Imports,
        )
    }

    /// Start evaluation with the registration's update cache policy.
    pub fn start_evaluation_with_update_via_cache(
        &mut self,
        script_url: &str,
        scope: &str,
        origin: &str,
        script: &str,
        update_via_cache: ServiceWorkerUpdateViaCache,
    ) -> Result<u64, ServiceWorkerManagerError> {
        self.start_evaluation_internal(
            script_url,
            scope,
            origin,
            script,
            EvaluationOptions {
                update_via_cache,
                script_type: ServiceWorkerScriptType::Classic,
                restoring_active: false,
                restored_imported_scripts: Vec::new(),
            },
        )
    }

    /// Start evaluation with explicit script type and update cache policy.
    pub fn start_evaluation_with_options(
        &mut self,
        script_url: &str,
        scope: &str,
        origin: &str,
        script: &str,
        script_type: ServiceWorkerScriptType,
        update_via_cache: ServiceWorkerUpdateViaCache,
    ) -> Result<u64, ServiceWorkerManagerError> {
        self.start_evaluation_internal(
            script_url,
            scope,
            origin,
            script,
            EvaluationOptions {
                update_via_cache,
                script_type,
                restoring_active: false,
                restored_imported_scripts: Vec::new(),
            },
        )
    }

    /// Recreate one persisted active runtime without replaying install/activate.
    pub fn start_restored_active(
        &mut self,
        registration: ServiceWorkerPersistentRegistration,
    ) -> Result<u64, ServiceWorkerManagerError> {
        let ServiceWorkerPersistentRegistration {
            script_url,
            scope,
            origin,
            script_source,
            update_via_cache,
            script_type,
            imported_scripts,
        } = registration;
        self.start_evaluation_internal(
            &script_url,
            &scope,
            &origin,
            &script_source,
            EvaluationOptions {
                update_via_cache,
                script_type,
                restoring_active: true,
                restored_imported_scripts: imported_scripts,
            },
        )
    }

    fn start_evaluation_internal(
        &mut self,
        script_url: &str,
        scope: &str,
        origin: &str,
        script: &str,
        options: EvaluationOptions,
    ) -> Result<u64, ServiceWorkerManagerError> {
        if script_url.trim().is_empty() {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker script URL is empty".into(),
            ));
        }
        if script_url.len() > MAX_URL_BYTES {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker script URL exceeds the length limit".into(),
            ));
        }
        if script.len() > MAX_SCRIPT_BYTES {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker script exceeds the size limit".into(),
            ));
        }
        let restored_imported_scripts =
            Self::validate_restored_imported_scripts(script, options.restored_imported_scripts)?;
        let key = ServiceWorkerRegistrationKey::new(origin, scope)?;
        if let Some(id) = self.slots.get(&key).and_then(|slot| slot.installing) {
            return Err(ServiceWorkerManagerError::JobInProgress(id));
        }
        if self.host.runtime_count() >= self.runtime_limit {
            return Err(ServiceWorkerManagerError::CapacityExceeded {
                limit: self.runtime_limit,
            });
        }

        let id = self.registry.register(script_url, scope, origin);
        let registration = self.registry.get_mut(id).expect("new registration must exist");
        registration.state = ServiceWorkerState::Installing;
        registration.update_via_cache = options.update_via_cache;
        registration.script_type = options.script_type;
        if let Err(error) = self.host.evaluate(id, script_url, script, options.script_type) {
            self.registry.unregister(id);
            return Err(error);
        }
        self.slots.entry(key.clone()).or_default().installing = Some(id);
        self.registration_keys.insert(id, key);
        self.state_changes.insert(id, Vec::new());
        self.script_sources.insert(id, script.as_bytes().to_vec());
        self.imported_scripts.insert(id, restored_imported_scripts);
        if options.restoring_active {
            self.restoring_active.insert(id);
        }
        Ok(id)
    }

    fn validate_restored_imported_scripts(
        main_script: &str,
        scripts: Vec<ServiceWorkerImportedScript>,
    ) -> Result<HashMap<String, Vec<u8>>, ServiceWorkerManagerError> {
        if scripts.len() > MAX_IMPORTED_SCRIPTS_PER_VERSION {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker import graph has too many scripts".into(),
            ));
        }
        let mut graph = HashMap::with_capacity(scripts.len());
        let mut total_bytes = main_script.len();
        for script in scripts {
            if script.url.len() > MAX_URL_BYTES || script.source.len() > MAX_SCRIPT_BYTES {
                return Err(ServiceWorkerManagerError::InvalidInput(
                    "Service Worker imported script exceeds the size limit".into(),
                ));
            }
            let url = url::Url::parse(&script.url).map_err(|_| {
                ServiceWorkerManagerError::InvalidInput("invalid Service Worker imported script URL".into())
            })?;
            if url.as_str() != script.url
                || url.fragment().is_some()
                || !matches!(url.scheme(), "http" | "https" | "data")
                || !url.username().is_empty()
                || url.password().is_some()
            {
                return Err(ServiceWorkerManagerError::InvalidInput(
                    "Service Worker imported script URL is not canonical".into(),
                ));
            }
            total_bytes = total_bytes.checked_add(script.source.len()).ok_or_else(|| {
                ServiceWorkerManagerError::InvalidInput("Service Worker script graph size overflow".into())
            })?;
            if total_bytes > MAX_SCRIPT_GRAPH_BYTES || graph.insert(script.url, script.source.into_bytes()).is_some() {
                return Err(ServiceWorkerManagerError::InvalidInput(
                    "Service Worker imported script graph is invalid or exceeds the size limit".into(),
                ));
            }
        }
        Ok(graph)
    }

    /// Resolve the current waiting or active version for a registration key.
    pub fn update_target(&self, registration_id: u64) -> Result<&ServiceWorkerRegistration, ServiceWorkerManagerError> {
        let key = self.key_for(registration_id)?;
        let slot = self
            .slots
            .get(key)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        if let Some(installing) = slot.installing {
            return Err(ServiceWorkerManagerError::JobInProgress(installing));
        }
        let Some(current_id) = slot.waiting.or(slot.active) else {
            let actual = self
                .registration(registration_id)
                .map(|registration| registration.state)
                .unwrap_or(ServiceWorkerState::Redundant);
            return Err(ServiceWorkerManagerError::InvalidState {
                registration_id,
                expected: ServiceWorkerState::Activated,
                actual,
            });
        };
        self.registration(current_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(current_id))
    }

    /// Compare a fetched script and start an installing replacement when changed.
    pub fn start_update(
        &mut self,
        registration_id: u64,
        script: &str,
    ) -> Result<ServiceWorkerUpdateOutcome, ServiceWorkerManagerError> {
        if script.len() > MAX_SCRIPT_BYTES {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker script exceeds the size limit".into(),
            ));
        }
        let registration = self.update_target(registration_id)?.clone();
        let current_id = registration.id;
        let registration_id = self.start_evaluation_with_options(
            &registration.script_url,
            &registration.scope,
            &registration.origin,
            script,
            registration.script_type,
            registration.update_via_cache,
        )?;
        self.update_predecessors.insert(registration_id, current_id);
        Ok(ServiceWorkerUpdateOutcome::Started { registration_id })
    }

    /// Drain all currently available runtime events and apply state changes.
    pub fn poll(&mut self) -> Vec<ServiceWorkerManagerEvent> {
        let mut output = Vec::new();
        for (registration_id, event) in self.host.poll_events() {
            match event {
                ServiceWorkerEvent::Evaluated { .. } => {
                    if let Some(current_id) = self.update_predecessors.remove(&registration_id) {
                        let changed = !self.script_graphs_equal(current_id, registration_id);
                        let returned_id = if changed { registration_id } else { current_id };
                        output.push(ServiceWorkerManagerEvent::UpdateChecked {
                            candidate_registration_id: registration_id,
                            registration_id: returned_id,
                            changed,
                        });
                        if !changed {
                            self.fail_installing_version(registration_id);
                            continue;
                        }
                    }
                    output.push(ServiceWorkerManagerEvent::ScriptEvaluated { registration_id });
                    if self.restoring_active.remove(&registration_id) {
                        match self.complete_active_restoration(registration_id) {
                            Ok(()) => {
                                output.push(ServiceWorkerManagerEvent::RestorationCompleted { registration_id });
                            }
                            Err(error) => {
                                self.fail_installing_version(registration_id);
                                output.push(ServiceWorkerManagerEvent::CoordinationFailed {
                                    registration_id,
                                    message: error.to_string(),
                                });
                            }
                        }
                    } else {
                        self.evaluated.insert(registration_id);
                        if let Err(error) = self.dispatch_install(registration_id) {
                            self.fail_installing_version(registration_id);
                            output.push(ServiceWorkerManagerEvent::CoordinationFailed {
                                registration_id,
                                message: error.to_string(),
                            });
                        }
                    }
                }
                ServiceWorkerEvent::ScriptError { kind, message, .. } => {
                    self.fail_installing_version(registration_id);
                    output.push(ServiceWorkerManagerEvent::ScriptFailed {
                        registration_id,
                        kind,
                        message,
                    });
                }
                ServiceWorkerEvent::ImportScriptsRequested { request_id, specifiers } => {
                    match self.resolve_import_script_urls(registration_id, &specifiers) {
                        Ok(urls) => {
                            if self.restoring_active.contains(&registration_id) {
                                let result = self.restored_import_sources(registration_id, &urls);
                                let _ = self.host.complete_import_scripts(registration_id, request_id, result);
                            } else if let Some(sources) = self.cached_import_sources(registration_id, &urls) {
                                let _ = self
                                    .host
                                    .complete_import_scripts(registration_id, request_id, Ok(sources));
                            } else if self
                                .registration(registration_id)
                                .is_none_or(|registration| registration.state != ServiceWorkerState::Installing)
                            {
                                let _ = self.host.complete_import_scripts(
                                    registration_id,
                                    request_id,
                                    Err("importScripts cannot fetch a new script after installation".into()),
                                );
                            } else {
                                self.pending_import_requests
                                    .insert((registration_id, request_id), urls.clone());
                                let bypass_cache = self.registration(registration_id).is_some_and(|registration| {
                                    registration.update_via_cache == ServiceWorkerUpdateViaCache::None
                                });
                                output.push(ServiceWorkerManagerEvent::ImportScriptsRequested {
                                    registration_id,
                                    request_id,
                                    urls,
                                    bypass_cache,
                                });
                            }
                        }
                        Err(error) => {
                            let _ =
                                self.host
                                    .complete_import_scripts(registration_id, request_id, Err(error.to_string()));
                        }
                    }
                }
                ServiceWorkerEvent::ModuleScriptsRequested {
                    request_id,
                    referrer_url,
                    specifiers,
                } => match self.resolve_module_script_urls(registration_id, &referrer_url, &specifiers) {
                    Ok(urls) => {
                        if self.restoring_active.contains(&registration_id) {
                            let result = self.restored_import_sources(registration_id, &urls);
                            let _ = self.host.complete_import_scripts(registration_id, request_id, result);
                        } else if let Some(sources) = self.cached_import_sources(registration_id, &urls) {
                            let _ = self
                                .host
                                .complete_import_scripts(registration_id, request_id, Ok(sources));
                        } else if self
                            .registration(registration_id)
                            .is_none_or(|registration| registration.state != ServiceWorkerState::Installing)
                        {
                            let _ = self.host.complete_import_scripts(
                                registration_id,
                                request_id,
                                Err("Service Worker module graph cannot fetch after installation".into()),
                            );
                        } else {
                            self.pending_import_requests
                                .insert((registration_id, request_id), urls.clone());
                            let bypass_cache = self.registration(registration_id).is_some_and(|registration| {
                                registration.update_via_cache == ServiceWorkerUpdateViaCache::None
                            });
                            output.push(ServiceWorkerManagerEvent::ModuleScriptsRequested {
                                registration_id,
                                request_id,
                                urls,
                                bypass_cache,
                            });
                        }
                    }
                    Err(error) => {
                        let _ = self
                            .host
                            .complete_import_scripts(registration_id, request_id, Err(error.to_string()));
                    }
                },
                ServiceWorkerEvent::LifecycleSettled {
                    phase,
                    succeeded,
                    skip_waiting,
                    claim_clients,
                    message,
                    ..
                } => {
                    output.push(ServiceWorkerManagerEvent::LifecycleSettled {
                        registration_id,
                        phase,
                        succeeded,
                        skip_waiting,
                        claim_clients,
                        message,
                    });
                    if phase == ServiceWorkerLifecyclePhase::Activate && succeeded && claim_clients {
                        self.claimed_clients.insert(registration_id);
                    }
                    let transition = match phase {
                        ServiceWorkerLifecyclePhase::Install => self.apply_install_result(registration_id, succeeded),
                        ServiceWorkerLifecyclePhase::Activate => {
                            self.apply_activation_result(registration_id, succeeded)
                        }
                    };
                    match transition {
                        Ok(completed) => {
                            output.push(completed);
                            if phase == ServiceWorkerLifecyclePhase::Install
                                && succeeded
                                && (skip_waiting
                                    || self
                                        .key_for(registration_id)
                                        .ok()
                                        .and_then(|key| self.slots.get(key))
                                        .is_some_and(|slot| slot.active.is_none()))
                                && let Err(error) = self.activate_waiting(registration_id)
                            {
                                self.mark_redundant_and_stop(registration_id);
                                output.push(ServiceWorkerManagerEvent::CoordinationFailed {
                                    registration_id,
                                    message: error.to_string(),
                                });
                            }
                        }
                        Err(error) => {
                            self.mark_redundant_and_stop(registration_id);
                            output.push(ServiceWorkerManagerEvent::CoordinationFailed {
                                registration_id,
                                message: error.to_string(),
                            });
                        }
                    }
                }
                ServiceWorkerEvent::Closed => {
                    self.complete_pending_client_message_batches(registration_id);
                    if self.is_installing(registration_id) {
                        self.fail_installing_version(registration_id);
                        output.push(ServiceWorkerManagerEvent::ScriptFailed {
                            registration_id,
                            kind: ServiceWorkerScriptErrorKind::EngineUnavailable,
                            message: "Service Worker runtime closed".into(),
                        });
                    }
                }
                ServiceWorkerEvent::MessageDispatched {
                    event_id,
                    client_id,
                    outbound,
                } => {
                    let outbound_count = outbound.len();
                    self.complete_client_message_batch(
                        registration_id,
                        &client_id,
                        outbound.into_iter().map(|message| message.data_json).collect(),
                    );
                    output.push(ServiceWorkerManagerEvent::MessageDispatched {
                        registration_id,
                        event_id,
                        outbound_count,
                    });
                }
                ServiceWorkerEvent::MessageFailed {
                    event_id,
                    client_id,
                    message,
                } => {
                    self.complete_client_message_batch(registration_id, &client_id, Vec::new());
                    output.push(ServiceWorkerManagerEvent::MessageFailed {
                        registration_id,
                        event_id,
                        client_id,
                        message,
                    });
                }
            }
        }
        output
    }

    fn script_graphs_equal(&self, left: u64, right: u64) -> bool {
        self.script_sources.get(&left) == self.script_sources.get(&right)
            && self.imported_scripts.get(&left) == self.imported_scripts.get(&right)
            && self.registration(left).map(|registration| registration.script_type)
                == self.registration(right).map(|registration| registration.script_type)
    }

    /// Complete one host-owned `importScripts()` fetch batch.
    pub fn complete_import_scripts(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<Vec<ServiceWorkerImportedScript>, String>,
    ) -> Result<(), ServiceWorkerManagerError> {
        let expected = self
            .pending_import_requests
            .remove(&(registration_id, request_id))
            .ok_or(ServiceWorkerManagerError::InvalidInput(
                "unknown Service Worker importScripts request".into(),
            ))?;
        let sources = match result {
            Ok(scripts) => match self.validate_imported_scripts(registration_id, &expected, scripts) {
                Ok(scripts) => {
                    let graph = self
                        .imported_scripts
                        .get_mut(&registration_id)
                        .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
                    let mut sources = Vec::with_capacity(scripts.len());
                    for script in scripts {
                        sources.push(script.source.clone());
                        graph.insert(script.url, script.source.into_bytes());
                    }
                    Ok(sources)
                }
                Err(error) => Err(error.to_string()),
            },
            Err(message) => Err(message),
        };
        self.host.complete_import_scripts(registration_id, request_id, sources)
    }

    fn resolve_import_script_urls(
        &self,
        registration_id: u64,
        specifiers: &[String],
    ) -> Result<Vec<String>, ServiceWorkerManagerError> {
        let registration = self
            .registration(registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        let base = url::Url::parse(&registration.script_url)
            .map_err(|_| ServiceWorkerManagerError::InvalidInput("invalid Service Worker script URL".into()))?;
        specifiers
            .iter()
            .map(|specifier| {
                if specifier.len() > MAX_URL_BYTES {
                    return Err(ServiceWorkerManagerError::InvalidInput(
                        "Service Worker imported script URL exceeds the length limit".into(),
                    ));
                }
                let mut url = base.join(specifier).map_err(|_| {
                    ServiceWorkerManagerError::InvalidInput("invalid Service Worker imported script URL".into())
                })?;
                if !matches!(url.scheme(), "http" | "https" | "data") {
                    return Err(ServiceWorkerManagerError::InvalidInput(
                        "Service Worker imported script URL uses an unsupported scheme".into(),
                    ));
                }
                if !url.username().is_empty() || url.password().is_some() {
                    return Err(ServiceWorkerManagerError::InvalidInput(
                        "Service Worker imported script URL contains credentials".into(),
                    ));
                }
                url.set_fragment(None);
                Ok(url.to_string())
            })
            .collect()
    }

    fn resolve_module_script_urls(
        &self,
        registration_id: u64,
        referrer_url: &str,
        specifiers: &[String],
    ) -> Result<Vec<String>, ServiceWorkerManagerError> {
        let registration = self
            .registration(registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        if registration.script_type != ServiceWorkerScriptType::Module {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "static module fetch requested by a classic Service Worker".into(),
            ));
        }
        let referrer = url::Url::parse(referrer_url)
            .map_err(|_| ServiceWorkerManagerError::InvalidInput("invalid Service Worker module referrer".into()))?;
        let referrer_url = referrer.to_string();
        if referrer_url != registration.script_url
            && !self
                .imported_scripts
                .get(&registration_id)
                .is_some_and(|graph| graph.contains_key(&referrer_url))
        {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker module referrer is not in the script graph".into(),
            ));
        }
        let registration_origin = url::Url::parse(&registration.origin)
            .map_err(|_| ServiceWorkerManagerError::InvalidInput("invalid Service Worker origin".into()))?
            .origin();
        specifiers
            .iter()
            .map(|specifier| {
                if specifier.len() > MAX_URL_BYTES {
                    return Err(ServiceWorkerManagerError::InvalidInput(
                        "Service Worker module URL exceeds the length limit".into(),
                    ));
                }
                let mut url = referrer
                    .join(specifier)
                    .map_err(|_| ServiceWorkerManagerError::InvalidInput("invalid Service Worker module URL".into()))?;
                if !matches!(url.scheme(), "http" | "https")
                    || !url.username().is_empty()
                    || url.password().is_some()
                    || url.origin() != registration_origin
                {
                    return Err(ServiceWorkerManagerError::InvalidInput(
                        "Service Worker module URL must be same-origin http(s)".into(),
                    ));
                }
                url.set_fragment(None);
                Ok(url.to_string())
            })
            .collect()
    }

    fn restored_import_sources(&self, registration_id: u64, urls: &[String]) -> Result<Vec<String>, String> {
        let graph = self
            .imported_scripts
            .get(&registration_id)
            .ok_or_else(|| "persisted Service Worker import graph is missing".to_string())?;
        urls.iter()
            .map(|url| {
                graph
                    .get(url)
                    .ok_or_else(|| format!("persisted Service Worker imported script is missing: {url}"))
                    .and_then(|source| {
                        String::from_utf8(source.clone())
                            .map_err(|_| format!("persisted Service Worker imported script is not UTF-8: {url}"))
                    })
            })
            .collect()
    }

    fn cached_import_sources(&self, registration_id: u64, urls: &[String]) -> Option<Vec<String>> {
        let graph = self.imported_scripts.get(&registration_id)?;
        urls.iter()
            .map(|url| String::from_utf8(graph.get(url)?.clone()).ok())
            .collect()
    }

    fn validate_imported_scripts(
        &self,
        registration_id: u64,
        expected: &[String],
        scripts: Vec<ServiceWorkerImportedScript>,
    ) -> Result<Vec<ServiceWorkerImportedScript>, ServiceWorkerManagerError> {
        if scripts.len() != expected.len()
            || scripts
                .iter()
                .zip(expected)
                .any(|(script, expected_url)| script.url != *expected_url)
        {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker imported script response does not match the request".into(),
            ));
        }
        let mut graph = self.imported_scripts.get(&registration_id).cloned().unwrap_or_default();
        for script in &scripts {
            if script.source.len() > MAX_SCRIPT_BYTES {
                return Err(ServiceWorkerManagerError::InvalidInput(
                    "Service Worker imported script exceeds the size limit".into(),
                ));
            }
            graph.insert(script.url.clone(), script.source.as_bytes().to_vec());
        }
        if graph.len() > MAX_IMPORTED_SCRIPTS_PER_VERSION {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker import graph has too many scripts".into(),
            ));
        }
        let imported_bytes = graph
            .values()
            .try_fold(0usize, |total, source| total.checked_add(source.len()));
        let total_bytes = imported_bytes.and_then(|bytes| {
            self.script_sources
                .get(&registration_id)
                .and_then(|source| bytes.checked_add(source.len()))
        });
        if total_bytes.is_none_or(|bytes| bytes > MAX_SCRIPT_GRAPH_BYTES) {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker imported script graph exceeds the size limit".into(),
            ));
        }
        Ok(scripts)
    }

    /// Promote a recreated runtime directly to its persisted active slot.
    fn complete_active_restoration(&mut self, registration_id: u64) -> Result<(), ServiceWorkerManagerError> {
        self.require_state(registration_id, ServiceWorkerState::Installing)?;
        let key = self.key_for(registration_id)?.clone();
        let previous_active = {
            let slot = self.slots.get_mut(&key).expect("registration key must have slots");
            if slot.installing != Some(registration_id) {
                return Err(ServiceWorkerManagerError::JobInProgress(
                    slot.installing.unwrap_or(registration_id),
                ));
            }
            slot.installing = None;
            slot.active.replace(registration_id)
        };
        if let Some(previous_active) = previous_active {
            self.mark_redundant_and_stop(previous_active);
        }
        self.registry
            .get_mut(registration_id)
            .expect("validated registration must exist")
            .state = ServiceWorkerState::Activated;
        Ok(())
    }

    /// Apply the result of the install event and its lifetime promises.
    fn apply_install_result(
        &mut self,
        registration_id: u64,
        succeeded: bool,
    ) -> Result<ServiceWorkerManagerEvent, ServiceWorkerManagerError> {
        self.require_state(registration_id, ServiceWorkerState::Installing)?;
        if !self.evaluated.remove(&registration_id) {
            return Err(ServiceWorkerManagerError::EvaluationPending(registration_id));
        }
        let key = self.key_for(registration_id)?.clone();
        let old_waiting = {
            let slot = self.slots.get_mut(&key).expect("registration key must have slots");
            if slot.installing != Some(registration_id) {
                return Err(ServiceWorkerManagerError::JobInProgress(
                    slot.installing.unwrap_or(registration_id),
                ));
            }
            slot.installing = None;
            if succeeded {
                slot.waiting.replace(registration_id)
            } else {
                None
            }
        };

        if succeeded {
            self.registry
                .get_mut(registration_id)
                .expect("validated registration must exist")
                .state = ServiceWorkerState::Installed;
            self.record_state_change(registration_id, ServiceWorkerState::Installed);
            if let Some(old_waiting) = old_waiting {
                self.mark_redundant_and_stop(old_waiting);
            }
        } else {
            self.mark_redundant_and_stop(registration_id);
        }

        Ok(ServiceWorkerManagerEvent::InstallCompleted {
            registration_id,
            succeeded,
        })
    }

    /// Dispatch the real install event for an evaluated installing version.
    fn dispatch_install(&mut self, registration_id: u64) -> Result<(), ServiceWorkerManagerError> {
        self.require_state(registration_id, ServiceWorkerState::Installing)?;
        if !self.evaluated.contains(&registration_id) {
            return Err(ServiceWorkerManagerError::EvaluationPending(registration_id));
        }
        self.host
            .dispatch_lifecycle(registration_id, ServiceWorkerLifecyclePhase::Install)
    }

    /// Move a waiting version into the activating state.
    fn begin_activation(&mut self, registration_id: u64) -> Result<(), ServiceWorkerManagerError> {
        self.require_state(registration_id, ServiceWorkerState::Installed)?;
        let key = self.key_for(registration_id)?;
        if self.slots.get(key).and_then(|slot| slot.waiting) != Some(registration_id) {
            return Err(ServiceWorkerManagerError::InvalidState {
                registration_id,
                expected: ServiceWorkerState::Installed,
                actual: ServiceWorkerState::Redundant,
            });
        }
        self.registry
            .get_mut(registration_id)
            .expect("validated registration must exist")
            .state = ServiceWorkerState::Activating;
        self.record_state_change(registration_id, ServiceWorkerState::Activating);
        Ok(())
    }

    /// Dispatch the real activate event for an activating version.
    fn dispatch_activate(&mut self, registration_id: u64) -> Result<(), ServiceWorkerManagerError> {
        self.require_state(registration_id, ServiceWorkerState::Activating)?;
        self.host
            .dispatch_lifecycle(registration_id, ServiceWorkerLifecyclePhase::Activate)
    }

    /// Apply the result of the activate event and its lifetime promises.
    fn apply_activation_result(
        &mut self,
        registration_id: u64,
        succeeded: bool,
    ) -> Result<ServiceWorkerManagerEvent, ServiceWorkerManagerError> {
        self.require_state(registration_id, ServiceWorkerState::Activating)?;
        let key = self.key_for(registration_id)?.clone();
        let slot = self.slots.get_mut(&key).expect("registration key must have slots");
        if slot.waiting != Some(registration_id) {
            return Err(ServiceWorkerManagerError::InvalidState {
                registration_id,
                expected: ServiceWorkerState::Activating,
                actual: ServiceWorkerState::Redundant,
            });
        }
        slot.waiting = None;

        if succeeded {
            if let Some(old_active) = slot.active.replace(registration_id) {
                self.mark_redundant_and_stop(old_active);
            }
            self.registry
                .get_mut(registration_id)
                .expect("validated registration must exist")
                .state = ServiceWorkerState::Activated;
            self.record_state_change(registration_id, ServiceWorkerState::Activated);
        } else {
            self.mark_redundant_and_stop(registration_id);
        }

        Ok(ServiceWorkerManagerEvent::ActivationCompleted {
            registration_id,
            succeeded,
        })
    }

    /// Activate a waiting replacement version.
    ///
    /// The first successful installation activates automatically. A replacement
    /// remains waiting until the host determines it can activate or implements
    /// `skipWaiting()` semantics.
    pub fn activate_waiting(&mut self, registration_id: u64) -> Result<(), ServiceWorkerManagerError> {
        self.begin_activation(registration_id)?;
        if let Err(error) = self.dispatch_activate(registration_id) {
            self.registry
                .get_mut(registration_id)
                .expect("validated registration must exist")
                .state = ServiceWorkerState::Installed;
            return Err(error);
        }
        Ok(())
    }

    /// Remove a registration and stop every version associated with its key.
    pub fn unregister(&mut self, registration_id: u64) -> bool {
        let Some(key) = self.registration_keys.get(&registration_id).cloned() else {
            return false;
        };
        let version_ids = self
            .registration_keys
            .iter()
            .filter_map(|(&id, version_key)| (version_key == &key).then_some(id))
            .collect::<Vec<_>>();
        let mut removed = false;
        for id in version_ids {
            self.evaluated.remove(&id);
            self.restoring_active.remove(&id);
            self.mark_redundant_and_stop(id);
            removed |= self.registry.unregister(id);
            self.registration_keys.remove(&id);
            self.state_changes.remove(&id);
        }
        self.slots.remove(&key);
        removed
    }

    /// Inspect one registration version.
    pub fn registration(&self, registration_id: u64) -> Option<&ServiceWorkerRegistration> {
        self.registry.get(registration_id)
    }

    /// Return lifecycle states recorded after a caller-owned sequence cursor.
    ///
    /// Sequence zero starts immediately after the initial `installing` state.
    /// The log is immutable for the lifetime of a version so independent
    /// renderer clients cannot consume each other's events.
    pub fn state_changes_since(
        &self,
        registration_id: u64,
        after_sequence: u64,
    ) -> Option<(u64, &[ServiceWorkerState])> {
        let changes = self.state_changes.get(&registration_id)?;
        let start = usize::try_from(after_sequence).unwrap_or(usize::MAX).min(changes.len());
        Some((changes.len() as u64, &changes[start..]))
    }

    /// Return whether this version requested `clients.claim()` while activating.
    pub fn claims_clients(&self, registration_id: u64) -> bool {
        self.claimed_clients.contains(&registration_id)
    }

    /// Queue a page message on an evaluated installing, waiting, or active worker runtime.
    pub fn post_message(
        &mut self,
        registration_id: u64,
        event_id: u64,
        data_json: &str,
        client_id: &str,
        client_url: &str,
    ) -> Result<(), ServiceWorkerManagerError> {
        let state = self
            .registration(registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?
            .state;
        let can_receive = matches!(
            state,
            ServiceWorkerState::Installed | ServiceWorkerState::Activating | ServiceWorkerState::Activated
        ) || (state == ServiceWorkerState::Installing && self.evaluated.contains(&registration_id));
        if !can_receive {
            return Err(ServiceWorkerManagerError::InvalidState {
                registration_id,
                expected: ServiceWorkerState::Activated,
                actual: state,
            });
        }
        let key = (registration_id, client_id.to_string());
        let pending = self.pending_client_messages.get(&key).copied().unwrap_or(0);
        let known_client = self.client_messages.contains_key(&key) || self.pending_client_messages.contains_key(&key);
        if let Some(batches) = self.client_messages.get(&key)
            && batches.len().saturating_add(pending) >= MAX_MESSAGES_PER_CLIENT
        {
            return Err(ServiceWorkerManagerError::ClientMessageCapacityExceeded {
                limit: MAX_MESSAGES_PER_CLIENT,
            });
        }
        if !known_client
            && self
                .client_messages
                .keys()
                .chain(self.pending_client_messages.keys())
                .filter(|(id, _)| *id == registration_id)
                .map(|(_, client_id)| client_id)
                .collect::<HashSet<_>>()
                .len()
                >= MAX_CLIENTS_PER_VERSION
        {
            return Err(ServiceWorkerManagerError::ClientCapacityExceeded {
                limit: MAX_CLIENTS_PER_VERSION,
            });
        }
        *self.pending_client_messages.entry(key.clone()).or_default() += 1;
        let result = self
            .host
            .dispatch_client_message(registration_id, event_id, data_json, client_id, client_url);
        if result.is_err() {
            self.release_client_message_reservation(&key);
        }
        result
    }

    /// Return worker messages for one browser-owned client after its cursor.
    pub fn client_messages_since(
        &self,
        registration_id: u64,
        client_id: &str,
        after_sequence: u64,
    ) -> (u64, Vec<String>) {
        let batches = self
            .client_messages
            .get(&(registration_id, client_id.to_string()))
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let start = usize::try_from(after_sequence).unwrap_or(usize::MAX).min(batches.len());
        (
            batches.len() as u64,
            batches[start..].iter().flatten().cloned().collect(),
        )
    }

    fn complete_client_message_batch(&mut self, registration_id: u64, client_id: &str, batch: Vec<String>) {
        let key = (registration_id, client_id.to_string());
        self.release_client_message_reservation(&key);
        let known_client = self.client_messages.contains_key(&key);
        let client_count = self
            .client_messages
            .keys()
            .filter(|(id, _)| *id == registration_id)
            .count();
        if known_client || client_count < MAX_CLIENTS_PER_VERSION {
            let batches = self.client_messages.entry(key).or_default();
            if batches.len() < MAX_MESSAGES_PER_CLIENT {
                batches.push(batch);
            }
        }
    }

    fn release_client_message_reservation(&mut self, key: &(u64, String)) {
        let Some(pending) = self.pending_client_messages.get_mut(key) else {
            return;
        };
        *pending = pending.saturating_sub(1);
        if *pending == 0 {
            self.pending_client_messages.remove(key);
        }
    }

    fn complete_pending_client_message_batches(&mut self, registration_id: u64) {
        let pending = self
            .pending_client_messages
            .iter()
            .filter(|((id, _), _)| *id == registration_id)
            .map(|((_, client_id), count)| (client_id.clone(), *count))
            .collect::<Vec<_>>();
        for (client_id, count) in pending {
            let key = (registration_id, client_id);
            self.pending_client_messages.remove(&key);
            let batches = self.client_messages.entry(key).or_default();
            let available = MAX_MESSAGES_PER_CLIENT.saturating_sub(batches.len());
            batches.extend(std::iter::repeat_with(Vec::new).take(count.min(available)));
        }
    }

    /// Inspect version slots for one origin/scope key.
    pub fn slots(&self, key: &ServiceWorkerRegistrationKey) -> Option<ServiceWorkerVersionSlots> {
        self.slots.get(key).copied()
    }

    /// Find the active registration with the longest matching scope.
    pub fn active_registration_for_url(&self, origin: &str, url: &str) -> Option<&ServiceWorkerRegistration> {
        self.slots
            .iter()
            .filter(|(key, slot)| key.origin == origin && slot.active.is_some())
            .filter_map(|(key, slot)| {
                let registration = self.registry.get(slot.active?)?;
                registration.is_in_scope(url).then_some((key.scope.len(), registration))
            })
            .max_by_key(|(scope_length, _)| *scope_length)
            .map(|(_, registration)| registration)
    }

    /// Find the representative registration with the longest matching scope.
    ///
    /// One manager key represents one web-visible registration. The active
    /// version remains representative while a replacement installs or waits.
    pub fn registration_for_url(&self, origin: &str, url: &str) -> Option<&ServiceWorkerRegistration> {
        self.slots
            .iter()
            .filter(|(key, _)| key.origin == origin)
            .filter_map(|(key, slot)| {
                let registration = self.representative_registration(slot)?;
                registration.is_in_scope(url).then_some((key.scope.len(), registration))
            })
            .max_by_key(|(scope_length, _)| *scope_length)
            .map(|(_, registration)| registration)
    }

    /// List one representative registration per scope for an origin.
    pub fn registrations_for_origin(&self, origin: &str) -> Vec<&ServiceWorkerRegistration> {
        let mut registrations: Vec<_> = self
            .slots
            .iter()
            .filter(|(key, _)| key.origin == origin)
            .filter_map(|(key, slot)| {
                self.representative_registration(slot)
                    .map(|registration| (key.scope.as_str(), registration))
            })
            .collect();
        registrations.sort_unstable_by_key(|(scope, _)| *scope);
        registrations
            .into_iter()
            .map(|(_, registration)| registration)
            .collect()
    }

    /// Export active registration inputs for browser-owned persistence.
    pub fn persistent_active_registrations(&self) -> Vec<ServiceWorkerPersistentRegistration> {
        let mut registrations = self
            .slots
            .iter()
            .filter_map(|(key, slot)| {
                let registration = self.registry.get(slot.active?)?;
                let script_source = String::from_utf8(self.script_sources.get(&registration.id)?.clone()).ok()?;
                let mut imported_scripts = self
                    .imported_scripts
                    .get(&registration.id)?
                    .iter()
                    .map(|(url, source)| {
                        Some(ServiceWorkerImportedScript {
                            url: url.clone(),
                            source: String::from_utf8(source.clone()).ok()?,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                imported_scripts.sort_unstable_by(|left, right| left.url.cmp(&right.url));
                Some(ServiceWorkerPersistentRegistration {
                    script_url: registration.script_url.clone(),
                    scope: key.scope.clone(),
                    origin: key.origin.clone(),
                    script_source,
                    update_via_cache: registration.update_via_cache,
                    script_type: registration.script_type,
                    imported_scripts,
                })
            })
            .collect::<Vec<_>>();
        registrations.sort_unstable_by(|left, right| {
            (&left.origin, &left.scope, &left.script_url).cmp(&(&right.origin, &right.scope, &right.script_url))
        });
        registrations
    }

    /// Return the number of live worker runtimes.
    pub fn runtime_count(&self) -> usize {
        self.host.runtime_count()
    }

    /// 测试钩子：强制停掉一个 runtime（模拟引擎线程退出，触发 Closed 事件）。
    #[cfg(test)]
    fn shutdown_runtime_for_test(&mut self, registration_id: u64) {
        self.host.shutdown(registration_id);
    }

    fn key_for(&self, registration_id: u64) -> Result<&ServiceWorkerRegistrationKey, ServiceWorkerManagerError> {
        self.registration_keys
            .get(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))
    }

    fn require_state(
        &self,
        registration_id: u64,
        expected: ServiceWorkerState,
    ) -> Result<(), ServiceWorkerManagerError> {
        let registration = self
            .registry
            .get(registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        if registration.state != expected {
            return Err(ServiceWorkerManagerError::InvalidState {
                registration_id,
                expected,
                actual: registration.state,
            });
        }
        Ok(())
    }

    fn is_installing(&self, registration_id: u64) -> bool {
        self.registry
            .get(registration_id)
            .is_some_and(|registration| registration.state == ServiceWorkerState::Installing)
    }

    fn representative_registration(&self, slot: &ServiceWorkerVersionSlots) -> Option<&ServiceWorkerRegistration> {
        let id = slot.active.or(slot.waiting).or(slot.installing)?;
        self.registry.get(id)
    }

    fn fail_installing_version(&mut self, registration_id: u64) {
        self.evaluated.remove(&registration_id);
        if let Some(key) = self.registration_keys.get(&registration_id)
            && let Some(slot) = self.slots.get_mut(key)
            && slot.installing == Some(registration_id)
        {
            slot.installing = None;
        }
        self.mark_redundant_and_stop(registration_id);
    }

    fn mark_redundant_and_stop(&mut self, registration_id: u64) {
        let changed = if let Some(registration) = self.registry.get_mut(registration_id) {
            let changed = registration.state != ServiceWorkerState::Redundant;
            registration.mark_redundant();
            changed
        } else {
            false
        };
        if changed {
            self.record_state_change(registration_id, ServiceWorkerState::Redundant);
        }
        self.claimed_clients.remove(&registration_id);
        self.restoring_active.remove(&registration_id);
        self.client_messages.retain(|(id, _), _| *id != registration_id);
        self.pending_client_messages.retain(|(id, _), _| *id != registration_id);
        self.pending_import_requests.retain(|(id, _), _| *id != registration_id);
        self.update_predecessors.remove(&registration_id);
        self.script_sources.remove(&registration_id);
        self.imported_scripts.remove(&registration_id);
        self.host.shutdown(registration_id);
    }

    fn record_state_change(&mut self, registration_id: u64, state: ServiceWorkerState) {
        if let Some(changes) = self.state_changes.get_mut(&registration_id) {
            changes.push(state);
        }
    }
}

impl Default for ServiceWorkerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn key(scope: &str) -> ServiceWorkerRegistrationKey {
        ServiceWorkerRegistrationKey::new("https://example.test", scope).unwrap()
    }

    fn manager_under_test() -> ServiceWorkerManager {
        ServiceWorkerManager::with_local_host(SandboxConfig {
            timeout_ms: 200,
            ..Default::default()
        })
    }

    fn wait_for_event(manager: &mut ServiceWorkerManager) -> ServiceWorkerManagerEvent {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(event) = manager.poll().into_iter().next() {
                return event;
            }
            assert!(Instant::now() < deadline, "manager event timed out");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn start(manager: &mut ServiceWorkerManager, scope: &str, script: &str) -> u64 {
        manager
            .start_evaluation("https://example.test/sw.js", scope, "https://example.test", script)
            .unwrap()
    }

    fn wait_for_state(
        manager: &mut ServiceWorkerManager,
        registration_id: u64,
        expected: ServiceWorkerState,
    ) -> Vec<ServiceWorkerManagerEvent> {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut events = Vec::new();
        loop {
            events.extend(manager.poll());
            if manager
                .registration(registration_id)
                .is_some_and(|registration| registration.state == expected)
            {
                return events;
            }
            assert!(Instant::now() < deadline, "manager state timed out");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn start_active(manager: &mut ServiceWorkerManager, scope: &str, script: &str) -> u64 {
        let id = start(manager, scope, script);
        wait_for_state(manager, id, ServiceWorkerState::Activated);
        id
    }

    fn wait_for_import_request(manager: &mut ServiceWorkerManager, registration_id: u64) -> (u64, Vec<String>, bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            for event in manager.poll() {
                if let ServiceWorkerManagerEvent::ImportScriptsRequested {
                    registration_id: id,
                    request_id,
                    urls,
                    bypass_cache,
                } = event
                    && id == registration_id
                {
                    return (request_id, urls, bypass_cache);
                }
            }
            assert!(Instant::now() < deadline, "manager importScripts request timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn wait_for_module_request(manager: &mut ServiceWorkerManager, registration_id: u64) -> (u64, Vec<String>, bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            for event in manager.poll() {
                if let ServiceWorkerManagerEvent::ModuleScriptsRequested {
                    registration_id: id,
                    request_id,
                    urls,
                    bypass_cache,
                } = event
                    && id == registration_id
                {
                    return (request_id, urls, bypass_cache);
                }
            }
            assert!(Instant::now() < deadline, "manager module request timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn wait_for_update_check(manager: &mut ServiceWorkerManager, candidate_id: u64) -> (u64, bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            for event in manager.poll() {
                if let ServiceWorkerManagerEvent::UpdateChecked {
                    candidate_registration_id,
                    registration_id,
                    changed,
                } = event
                    && candidate_registration_id == candidate_id
                {
                    return (registration_id, changed);
                }
            }
            assert!(Instant::now() < deadline, "manager update comparison timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn successful_version_moves_through_slots() {
        let mut manager = manager_under_test();
        let id = start(&mut manager, "/app/", "globalThis.ready = true;");
        assert_eq!(manager.registration(id).unwrap().state, ServiceWorkerState::Installing);
        assert_eq!(manager.slots(&key("/app/")).unwrap().installing, Some(id));

        let events = wait_for_state(&mut manager, id, ServiceWorkerState::Activated);
        assert!(events.contains(&ServiceWorkerManagerEvent::ScriptEvaluated { registration_id: id }));
        assert!(events.contains(&ServiceWorkerManagerEvent::InstallCompleted {
            registration_id: id,
            succeeded: true,
        }));
        assert!(events.contains(&ServiceWorkerManagerEvent::ActivationCompleted {
            registration_id: id,
            succeeded: true,
        }));
        assert_eq!(manager.registration(id).unwrap().state, ServiceWorkerState::Activated);
        assert_eq!(manager.slots(&key("/app/")).unwrap().active, Some(id));
        assert_eq!(
            manager.state_changes_since(id, 0),
            Some((
                3,
                [
                    ServiceWorkerState::Installed,
                    ServiceWorkerState::Activating,
                    ServiceWorkerState::Activated,
                ]
                .as_slice()
            ))
        );
        assert_eq!(
            manager.state_changes_since(id, 1),
            Some((
                3,
                [ServiceWorkerState::Activating, ServiceWorkerState::Activated].as_slice()
            ))
        );
    }

    #[test]
    fn update_compares_script_bytes_before_starting_replacement() {
        let mut manager = manager_under_test();
        let first = start_active(&mut manager, "/", "globalThis.version = 1;");
        let runtime_count = manager.runtime_count();

        let ServiceWorkerUpdateOutcome::Started {
            registration_id: unchanged_candidate,
        } = manager.start_update(first, "globalThis.version = 1;").unwrap()
        else {
            panic!("update comparison must evaluate a candidate graph");
        };
        assert_eq!(wait_for_update_check(&mut manager, unchanged_candidate), (first, false));
        let _ = manager.poll();
        assert_eq!(manager.runtime_count(), runtime_count);
        assert_eq!(manager.slots(&key("/")).unwrap().installing, None);

        let ServiceWorkerUpdateOutcome::Started {
            registration_id: replacement,
        } = manager.start_update(first, "globalThis.version = 2;").unwrap()
        else {
            panic!("changed update must start a replacement");
        };
        assert_ne!(replacement, first);
        assert_eq!(manager.slots(&key("/")).unwrap().active, Some(first));
        assert_eq!(manager.slots(&key("/")).unwrap().installing, Some(replacement));
        assert_eq!(
            manager.registration(replacement).unwrap().script_url,
            "https://example.test/sw.js"
        );
        assert_eq!(wait_for_update_check(&mut manager, replacement), (replacement, true));
        wait_for_state(&mut manager, replacement, ServiceWorkerState::Installed);
        manager.activate_waiting(replacement).unwrap();
        wait_for_state(&mut manager, replacement, ServiceWorkerState::Activated);
        let ServiceWorkerUpdateOutcome::Started {
            registration_id: unchanged_candidate,
        } = manager.start_update(first, "globalThis.version = 2;").unwrap()
        else {
            panic!("update comparison must evaluate a candidate graph");
        };
        assert_eq!(
            wait_for_update_check(&mut manager, unchanged_candidate),
            (replacement, false)
        );

        let next = manager
            .start_evaluation(
                "https://example.test/sw-v3.js",
                "/",
                "https://example.test",
                "globalThis.version = 3;",
            )
            .unwrap();
        wait_for_state(&mut manager, next, ServiceWorkerState::Installed);
        manager.activate_waiting(next).unwrap();
        wait_for_state(&mut manager, next, ServiceWorkerState::Activated);
        assert_eq!(
            manager.update_target(first).unwrap().script_url,
            "https://example.test/sw-v3.js"
        );
        let ServiceWorkerUpdateOutcome::Started {
            registration_id: unchanged_candidate,
        } = manager.start_update(first, "globalThis.version = 3;").unwrap()
        else {
            panic!("update comparison must evaluate a candidate graph");
        };
        assert_eq!(wait_for_update_check(&mut manager, unchanged_candidate), (next, false));
    }

    #[test]
    fn update_compares_imported_script_graph_when_main_bytes_match() {
        let mut manager = manager_under_test();
        let script = "importScripts('./dependency.js');";
        let first = start(&mut manager, "/", script);
        let (request_id, urls, bypass_cache) = wait_for_import_request(&mut manager, first);
        assert!(!bypass_cache);
        manager
            .complete_import_scripts(
                first,
                request_id,
                Ok(vec![ServiceWorkerImportedScript {
                    url: urls[0].clone(),
                    source: "globalThis.dependencyVersion = 1;".into(),
                }]),
            )
            .unwrap();
        wait_for_state(&mut manager, first, ServiceWorkerState::Activated);

        let ServiceWorkerUpdateOutcome::Started {
            registration_id: unchanged_candidate,
        } = manager.start_update(first, script).unwrap()
        else {
            panic!("update comparison must evaluate imported scripts");
        };
        let (request_id, urls, _) = wait_for_import_request(&mut manager, unchanged_candidate);
        manager
            .complete_import_scripts(
                unchanged_candidate,
                request_id,
                Ok(vec![ServiceWorkerImportedScript {
                    url: urls[0].clone(),
                    source: "globalThis.dependencyVersion = 1;".into(),
                }]),
            )
            .unwrap();
        assert_eq!(wait_for_update_check(&mut manager, unchanged_candidate), (first, false));

        let ServiceWorkerUpdateOutcome::Started {
            registration_id: changed_candidate,
        } = manager.start_update(first, script).unwrap()
        else {
            panic!("update comparison must evaluate imported scripts");
        };
        let (request_id, urls, _) = wait_for_import_request(&mut manager, changed_candidate);
        manager
            .complete_import_scripts(
                changed_candidate,
                request_id,
                Ok(vec![ServiceWorkerImportedScript {
                    url: urls[0].clone(),
                    source: "globalThis.dependencyVersion = 2;".into(),
                }]),
            )
            .unwrap();
        assert_eq!(
            wait_for_update_check(&mut manager, changed_candidate),
            (changed_candidate, true)
        );
        wait_for_state(&mut manager, changed_candidate, ServiceWorkerState::Installed);
    }

    #[test]
    fn module_graph_resolves_transitive_urls_and_persists_sources() {
        let mut manager = manager_under_test();
        let id = manager
            .start_evaluation_with_options(
                "https://example.test/workers/sw.js",
                "/",
                "https://example.test",
                "import { doubled } from './lib/entry.js'; if (doubled !== 14) throw new Error('wrong value');",
                ServiceWorkerScriptType::Module,
                ServiceWorkerUpdateViaCache::Imports,
            )
            .unwrap();

        let (request_id, urls, bypass_cache) = wait_for_module_request(&mut manager, id);
        assert!(!bypass_cache);
        assert_eq!(urls, ["https://example.test/workers/lib/entry.js"]);
        manager
            .complete_import_scripts(
                id,
                request_id,
                Ok(vec![ServiceWorkerImportedScript {
                    url: urls[0].clone(),
                    source: "import { value } from './value.js'; export const doubled = value * 2;".into(),
                }]),
            )
            .unwrap();

        let (request_id, urls, _) = wait_for_module_request(&mut manager, id);
        assert_eq!(urls, ["https://example.test/workers/lib/value.js"]);
        manager
            .complete_import_scripts(
                id,
                request_id,
                Ok(vec![ServiceWorkerImportedScript {
                    url: urls[0].clone(),
                    source: "export const value = 7;".into(),
                }]),
            )
            .unwrap();
        wait_for_state(&mut manager, id, ServiceWorkerState::Activated);

        let persisted = manager.persistent_active_registrations();
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].script_type, ServiceWorkerScriptType::Module);
        assert_eq!(persisted[0].imported_scripts.len(), 2);
    }

    #[test]
    fn module_update_compares_static_dependency_bytes() {
        fn complete_module_graph(manager: &mut ServiceWorkerManager, registration_id: u64, dependency_source: &str) {
            let (request_id, urls, _) = wait_for_module_request(manager, registration_id);
            manager
                .complete_import_scripts(
                    registration_id,
                    request_id,
                    Ok(vec![ServiceWorkerImportedScript {
                        url: urls[0].clone(),
                        source: dependency_source.into(),
                    }]),
                )
                .unwrap();
        }

        let mut manager = manager_under_test();
        let main = "import { value } from './dependency.js'; globalThis.value = value;";
        let first = manager
            .start_evaluation_with_options(
                "https://example.test/sw.js",
                "/",
                "https://example.test",
                main,
                ServiceWorkerScriptType::Module,
                ServiceWorkerUpdateViaCache::Imports,
            )
            .unwrap();
        complete_module_graph(&mut manager, first, "export const value = 1;");
        wait_for_state(&mut manager, first, ServiceWorkerState::Activated);

        let ServiceWorkerUpdateOutcome::Started {
            registration_id: unchanged_candidate,
        } = manager.start_update(first, main).unwrap()
        else {
            panic!("module update must evaluate a candidate graph");
        };
        complete_module_graph(&mut manager, unchanged_candidate, "export const value = 1;");
        assert_eq!(wait_for_update_check(&mut manager, unchanged_candidate), (first, false));

        let ServiceWorkerUpdateOutcome::Started {
            registration_id: changed_candidate,
        } = manager.start_update(first, main).unwrap()
        else {
            panic!("module update must evaluate a candidate graph");
        };
        complete_module_graph(&mut manager, changed_candidate, "export const value = 2;");
        assert_eq!(
            wait_for_update_check(&mut manager, changed_candidate),
            (changed_candidate, true)
        );
    }

    #[test]
    fn unregister_removes_active_and_waiting_versions_for_registration_key() {
        let mut manager = manager_under_test();
        let active = start_active(&mut manager, "/", "globalThis.version = 1;");
        let ServiceWorkerUpdateOutcome::Started {
            registration_id: waiting,
        } = manager.start_update(active, "globalThis.version = 2;").unwrap()
        else {
            panic!("changed update must start a replacement");
        };
        assert_eq!(wait_for_update_check(&mut manager, waiting), (waiting, true));
        wait_for_state(&mut manager, waiting, ServiceWorkerState::Installed);
        assert_eq!(
            manager.slots(&key("/")),
            Some(ServiceWorkerVersionSlots {
                installing: None,
                waiting: Some(waiting),
                active: Some(active),
            })
        );

        assert!(manager.unregister(active));
        assert!(manager.slots(&key("/")).is_none());
        assert!(manager.registration(active).is_none());
        assert!(manager.registration(waiting).is_none());

        let next = start_active(&mut manager, "/", "globalThis.version = 3;");
        assert_eq!(manager.slots(&key("/")).unwrap().active, Some(next));
    }

    #[test]
    fn repeated_import_uses_version_resource_map_without_refetch() {
        let mut manager = manager_under_test();
        let id = start(
            &mut manager,
            "/",
            "importScripts('./dependency.js');
             const first = globalThis.version;
             globalThis.version = null;
             importScripts('./dependency.js');
             if (globalThis.version !== first) throw new Error('resource map was not reused');",
        );
        let (request_id, urls, _) = wait_for_import_request(&mut manager, id);
        assert_eq!(urls, ["https://example.test/dependency.js"]);
        manager
            .complete_import_scripts(
                id,
                request_id,
                Ok(vec![ServiceWorkerImportedScript {
                    url: urls[0].clone(),
                    source: "globalThis.version = 'first-response';".into(),
                }]),
            )
            .unwrap();

        wait_for_state(&mut manager, id, ServiceWorkerState::Activated);
        assert_eq!(
            manager
                .imported_scripts
                .get(&id)
                .and_then(|graph| graph.get("https://example.test/dependency.js"))
                .map(Vec::as_slice),
            Some(b"globalThis.version = 'first-response';".as_slice())
        );
    }

    #[test]
    fn install_event_can_fetch_new_import_before_updated_flag() {
        let mut manager = manager_under_test();
        let id = start(
            &mut manager,
            "/",
            "addEventListener('install', () => {
               importScripts('/install-import.js');
               if (globalThis.installImported !== true) throw new Error('install import missing');
             });",
        );
        let (request_id, urls, _) = wait_for_import_request(&mut manager, id);
        assert_eq!(urls, ["https://example.test/install-import.js"]);
        manager
            .complete_import_scripts(
                id,
                request_id,
                Ok(vec![ServiceWorkerImportedScript {
                    url: urls[0].clone(),
                    source: "globalThis.installImported = true;".into(),
                }]),
            )
            .unwrap();
        wait_for_state(&mut manager, id, ServiceWorkerState::Activated);
    }

    #[test]
    fn restored_active_runtime_does_not_replay_install_or_activate() {
        let mut manager = manager_under_test();
        let id = manager
            .start_restored_active(ServiceWorkerPersistentRegistration {
                script_url: "https://example.test/sw.js".into(),
                scope: "/".into(),
                origin: "https://example.test".into(),
                script_source: "addEventListener('install', () => { throw new Error('install replayed'); });
                         addEventListener('activate', () => { throw new Error('activate replayed'); });"
                    .into(),
                update_via_cache: ServiceWorkerUpdateViaCache::Imports,
                script_type: ServiceWorkerScriptType::Classic,
                imported_scripts: Vec::new(),
            })
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut events = Vec::new();
        loop {
            events.extend(manager.poll());
            if manager
                .registration(id)
                .is_some_and(|registration| registration.state == ServiceWorkerState::Activated)
            {
                break;
            }
            assert!(Instant::now() < deadline, "active restoration timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(events.contains(&ServiceWorkerManagerEvent::RestorationCompleted { registration_id: id }));
        assert!(!events.iter().any(|event| matches!(
            event,
            ServiceWorkerManagerEvent::LifecycleSettled { .. }
                | ServiceWorkerManagerEvent::InstallCompleted { .. }
                | ServiceWorkerManagerEvent::ActivationCompleted { .. }
        )));
        let (latest_sequence, states) = manager.state_changes_since(id, 0).unwrap();
        assert_eq!(latest_sequence, 0);
        assert!(states.is_empty());
    }

    #[test]
    fn script_failure_clears_installing_slot() {
        let mut manager = manager_under_test();
        let id = start(&mut manager, "/", "function(");
        assert!(matches!(
            wait_for_event(&mut manager),
            ServiceWorkerManagerEvent::ScriptFailed {
                registration_id,
                kind: ServiceWorkerScriptErrorKind::Compile,
                ..
            } if registration_id == id
        ));
        assert_eq!(manager.registration(id).unwrap().state, ServiceWorkerState::Redundant);
        assert_eq!(manager.slots(&key("/")).unwrap().installing, None);
        // runtime 槽位随下一次事件轮询回收（Closed 先 drain 再 retain）。
        let _ = manager.poll();
        assert_eq!(manager.runtime_count(), 0);
    }

    #[test]
    fn lifecycle_runtime_outcomes_are_forwarded_typed() {
        let mut manager = manager_under_test();
        let id = start(
            &mut manager,
            "/",
            "addEventListener('install', event => {
                event.waitUntil(Promise.resolve());
            });
            addEventListener('activate', event => {
                event.waitUntil(Promise.reject(new Error('activate rejected')));
            });",
        );
        let events = wait_for_state(&mut manager, id, ServiceWorkerState::Redundant);
        assert!(events.contains(&ServiceWorkerManagerEvent::LifecycleSettled {
            registration_id: id,
            phase: ServiceWorkerLifecyclePhase::Install,
            succeeded: true,
            skip_waiting: false,
            claim_clients: false,
            message: String::new(),
        }));
        assert!(matches!(
            events.iter().find(|event| matches!(
                event,
                ServiceWorkerManagerEvent::LifecycleSettled {
                    phase: ServiceWorkerLifecyclePhase::Activate,
                    ..
                }
            )),
            Some(ServiceWorkerManagerEvent::LifecycleSettled {
                registration_id,
                phase: ServiceWorkerLifecyclePhase::Activate,
                succeeded: false,
                message,
                ..
            }) if *registration_id == id && message.contains("activate rejected")
        ));
    }

    #[test]
    fn install_failure_preserves_existing_active() {
        let mut manager = manager_under_test();
        let first = start_active(&mut manager, "/", "globalThis.version = 1;");

        let second = start(
            &mut manager,
            "/",
            "addEventListener('install', event => {
                event.waitUntil(Promise.reject(new Error('install rejected')));
            });",
        );
        wait_for_state(&mut manager, second, ServiceWorkerState::Redundant);

        let slots = manager.slots(&key("/")).unwrap();
        assert_eq!(slots.active, Some(first));
        assert_eq!(slots.installing, None);
        assert_eq!(
            manager.registration(first).unwrap().state,
            ServiceWorkerState::Activated
        );
        assert_eq!(
            manager.registration(second).unwrap().state,
            ServiceWorkerState::Redundant
        );
    }

    #[test]
    fn activation_failure_preserves_existing_active() {
        let mut manager = manager_under_test();
        let first = start_active(&mut manager, "/", "globalThis.version = 1;");

        let second = start(
            &mut manager,
            "/",
            "addEventListener('activate', event => {
                event.waitUntil(Promise.reject(new Error('activate rejected')));
            });",
        );
        wait_for_state(&mut manager, second, ServiceWorkerState::Installed);
        manager.activate_waiting(second).unwrap();
        wait_for_state(&mut manager, second, ServiceWorkerState::Redundant);

        let slots = manager.slots(&key("/")).unwrap();
        assert_eq!(slots.active, Some(first));
        assert_eq!(slots.waiting, None);
        assert_eq!(
            manager.registration(first).unwrap().state,
            ServiceWorkerState::Activated
        );
        assert_eq!(
            manager.registration(second).unwrap().state,
            ServiceWorkerState::Redundant
        );
    }

    #[test]
    fn activation_replaces_only_same_scope() {
        let mut manager = manager_under_test();
        let root = start_active(&mut manager, "/", "globalThis.scope = 'root';");
        let app = start_active(&mut manager, "/app/", "globalThis.scope = 'app';");

        let replacement = start(&mut manager, "/app/", "globalThis.scope = 'app-v2';");
        wait_for_state(&mut manager, replacement, ServiceWorkerState::Installed);
        manager.activate_waiting(replacement).unwrap();
        wait_for_state(&mut manager, replacement, ServiceWorkerState::Activated);

        assert_eq!(manager.slots(&key("/")).unwrap().active, Some(root));
        assert_eq!(manager.slots(&key("/app/")).unwrap().active, Some(replacement));
        assert_eq!(
            manager
                .active_registration_for_url("https://example.test", "https://example.test/app/page.html",)
                .unwrap()
                .id,
            replacement
        );
        assert_eq!(
            manager
                .active_registration_for_url("https://example.test", "https://example.test/other/page.html",)
                .unwrap()
                .id,
            root
        );
        assert!(
            manager
                .active_registration_for_url("https://other.test", "https://other.test/app/page.html",)
                .is_none()
        );
        assert_eq!(manager.registration(root).unwrap().state, ServiceWorkerState::Activated);
        assert_eq!(manager.registration(app).unwrap().state, ServiceWorkerState::Redundant);
    }

    #[test]
    fn skip_waiting_activates_replacement_without_host_command() {
        let mut manager = manager_under_test();
        let first = start_active(&mut manager, "/", "globalThis.version = 1;");
        let replacement = start(
            &mut manager,
            "/",
            "addEventListener('install', event => {
                event.waitUntil(skipWaiting());
            });",
        );

        let events = wait_for_state(&mut manager, replacement, ServiceWorkerState::Activated);
        assert!(events.iter().any(|event| matches!(
            event,
            ServiceWorkerManagerEvent::LifecycleSettled {
                registration_id,
                phase: ServiceWorkerLifecyclePhase::Install,
                succeeded: true,
                skip_waiting: true,
                ..
            } if *registration_id == replacement
        )));
        assert_eq!(manager.slots(&key("/")).unwrap().active, Some(replacement));
        assert_eq!(
            manager.registration(first).unwrap().state,
            ServiceWorkerState::Redundant
        );
    }

    #[test]
    fn clients_claim_is_recorded_for_activated_version() {
        let mut manager = manager_under_test();
        let id = start(
            &mut manager,
            "/app/",
            "addEventListener('activate', event => {
                event.waitUntil(clients.claim());
            });",
        );

        let events = wait_for_state(&mut manager, id, ServiceWorkerState::Activated);
        assert!(events.iter().any(|event| matches!(
            event,
            ServiceWorkerManagerEvent::LifecycleSettled {
                registration_id,
                phase: ServiceWorkerLifecyclePhase::Activate,
                succeeded: true,
                claim_clients: true,
                ..
            } if *registration_id == id
        )));
        assert!(manager.claims_clients(id));
    }

    #[test]
    fn page_message_dispatches_to_active_worker() {
        let mut manager = manager_under_test();
        let id = start_active(
            &mut manager,
            "/",
            "addEventListener('message', event => {
                globalThis.message = event.data.value;
                event.source.postMessage({echo: event.data.value});
            });",
        );

        manager
            .post_message(id, 91, r#"{"value":"hello"}"#, "client-1", "https://example.test/page")
            .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let events = manager.poll();
            if events.contains(&ServiceWorkerManagerEvent::MessageDispatched {
                registration_id: id,
                event_id: 91,
                outbound_count: 1,
            }) {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "message dispatch timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(
            manager.client_messages_since(id, "client-1", 0),
            (1, vec![r#"{"echo":"hello"}"#.to_string()])
        );

        manager.client_messages.insert(
            (id, "near-full-client".into()),
            vec![Vec::new(); MAX_MESSAGES_PER_CLIENT - 1],
        );
        manager
            .post_message(id, 92, "null", "near-full-client", "https://example.test/page")
            .unwrap();
        assert_eq!(
            manager.post_message(id, 93, "null", "near-full-client", "https://example.test/page",),
            Err(ServiceWorkerManagerError::ClientMessageCapacityExceeded {
                limit: MAX_MESSAGES_PER_CLIENT,
            })
        );

        manager
            .client_messages
            .insert((id, "full-client".into()), vec![Vec::new(); MAX_MESSAGES_PER_CLIENT]);
        assert_eq!(
            manager.post_message(id, 92, "null", "full-client", "https://example.test/page"),
            Err(ServiceWorkerManagerError::ClientMessageCapacityExceeded {
                limit: MAX_MESSAGES_PER_CLIENT,
            })
        );

        let closed_key = (id, "closed-client".to_string());
        manager.pending_client_messages.insert(closed_key.clone(), 2);
        manager.shutdown_runtime_for_test(id);
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while manager.pending_client_messages.contains_key(&closed_key) {
            let _ = manager.poll();
            assert!(std::time::Instant::now() < deadline, "runtime close timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(manager.client_messages_since(id, "closed-client", 0), (2, Vec::new()));
    }

    #[test]
    fn active_message_rejects_new_import_after_updated_flag() {
        let mut manager = manager_under_test();
        let id = start_active(
            &mut manager,
            "/",
            "addEventListener('message', event => {
               let errorName = null;
               try {
                 importScripts('/late-import.js');
               } catch (error) {
                 errorName = error && error.name;
               }
               event.source.postMessage({errorName: errorName});
             });",
        );

        manager
            .post_message(id, 94, "null", "client-late", "https://example.test/page")
            .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let events = manager.poll();
            assert!(
                !events.iter().any(|event| matches!(
                    event,
                    ServiceWorkerManagerEvent::ImportScriptsRequested {
                        registration_id,
                        ..
                    } if *registration_id == id
                )),
                "late import must not reach the host fetch layer"
            );
            if events.contains(&ServiceWorkerManagerEvent::MessageDispatched {
                registration_id: id,
                event_id: 94,
                outbound_count: 1,
            }) {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "late import rejection timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(
            manager.client_messages_since(id, "client-late", 0),
            (1, vec![r#"{"errorName":"NetworkError"}"#.to_string()])
        );
    }

    #[test]
    fn discovery_returns_one_representative_version_per_scope() {
        let mut manager = manager_under_test();
        let root = start_active(&mut manager, "/", "globalThis.scope = 'root';");
        let app = start_active(&mut manager, "/app/", "globalThis.scope = 'app';");
        let replacement = start(&mut manager, "/app/", "globalThis.scope = 'app-v2';");

        assert_eq!(
            manager
                .registration_for_url("https://example.test", "https://example.test/app/page")
                .unwrap()
                .id,
            app,
            "active version remains representative while replacement installs"
        );
        assert_eq!(
            manager
                .registrations_for_origin("https://example.test")
                .into_iter()
                .map(|registration| registration.id)
                .collect::<Vec<_>>(),
            vec![root, app]
        );
        assert!(manager.registrations_for_origin("https://other.test").is_empty());
        assert_eq!(
            manager.registration(replacement).unwrap().state,
            ServiceWorkerState::Installing
        );
    }

    #[test]
    fn discovery_exposes_first_installing_version() {
        let mut manager = manager_under_test();
        let installing = start(&mut manager, "/app/", "void 0;");

        assert_eq!(
            manager
                .registration_for_url("https://example.test", "https://example.test/app/page")
                .unwrap()
                .id,
            installing
        );
    }

    #[test]
    fn overlapping_job_for_same_key_is_rejected() {
        let mut manager = manager_under_test();
        let first = start(&mut manager, "/", "void 0;");
        let result = manager.start_evaluation("https://example.test/sw-v2.js", "/", "https://example.test", "void 0;");
        assert_eq!(result, Err(ServiceWorkerManagerError::JobInProgress(first)));
    }

    #[test]
    fn different_scope_jobs_can_evaluate_concurrently() {
        let mut manager = manager_under_test();
        let root = start(&mut manager, "/", "globalThis.scope = 'root';");
        let app = start(&mut manager, "/app/", "globalThis.scope = 'app';");
        assert_ne!(root, app);
        assert_eq!(manager.runtime_count(), 2);
        assert_eq!(manager.slots(&key("/")).unwrap().installing, Some(root));
        assert_eq!(manager.slots(&key("/app/")).unwrap().installing, Some(app));

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut evaluated = HashSet::new();
        while evaluated.len() < 2 {
            for event in manager.poll() {
                if let ServiceWorkerManagerEvent::ScriptEvaluated { registration_id } = event {
                    evaluated.insert(registration_id);
                }
            }
            assert!(Instant::now() < deadline, "parallel manager events timed out");
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(evaluated, HashSet::from([root, app]));
    }

    #[test]
    fn waiting_activation_rejects_installing_version() {
        let mut manager = manager_under_test();
        let id = start(&mut manager, "/", "void 0;");
        assert!(matches!(
            manager.activate_waiting(id),
            Err(ServiceWorkerManagerError::InvalidState {
                registration_id,
                actual: ServiceWorkerState::Installing,
                ..
            }) if registration_id == id
        ));
        assert_eq!(manager.registration(id).unwrap().state, ServiceWorkerState::Installing);
    }

    #[test]
    fn capacity_rejection_does_not_create_registration() {
        let mut manager = manager_under_test();
        manager.runtime_limit = 1;
        let first = start(&mut manager, "/", "void 0;");
        let result = manager.start_evaluation(
            "https://example.test/app-sw.js",
            "/app/",
            "https://example.test",
            "void 0;",
        );
        assert_eq!(result, Err(ServiceWorkerManagerError::CapacityExceeded { limit: 1 }));
        assert_eq!(manager.runtime_count(), 1);
        assert!(manager.slots(&key("/app/")).is_none());
        assert!(manager.registration(first + 1).is_none());
    }

    #[test]
    fn oversized_input_is_rejected_before_runtime_creation() {
        let mut manager = manager_under_test();
        let oversized_url = "x".repeat(MAX_URL_BYTES + 1);
        assert!(matches!(
            manager.start_evaluation(&oversized_url, "/", "https://example.test", "void 0;"),
            Err(ServiceWorkerManagerError::InvalidInput(_))
        ));

        let oversized_script = "x".repeat(MAX_SCRIPT_BYTES + 1);
        assert!(matches!(
            manager.start_evaluation(
                "https://example.test/sw.js",
                "/",
                "https://example.test",
                &oversized_script
            ),
            Err(ServiceWorkerManagerError::InvalidInput(_))
        ));
        assert_eq!(manager.runtime_count(), 0);
        assert!(manager.slots(&key("/")).is_none());
        assert!(manager.registration(0).is_none());
    }
}
