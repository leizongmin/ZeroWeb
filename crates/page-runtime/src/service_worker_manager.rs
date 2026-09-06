//! Service Worker registration and lifecycle coordination.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use zero_script_sandbox::{
    SandboxConfig, ServiceWorkerCacheStorageRequest, ServiceWorkerCacheStorageResult, ServiceWorkerClientInfo,
    ServiceWorkerEvent, ServiceWorkerFetchRequest, ServiceWorkerFetchResponse, ServiceWorkerLifecyclePhase,
    ServiceWorkerMessagePorts, ServiceWorkerMessageSource, ServiceWorkerOutboundMessage, ServiceWorkerPeerInfo,
    ServiceWorkerRegistrationPeers, ServiceWorkerRuntime, ServiceWorkerScriptErrorKind, ServiceWorkerWorkerMessage,
};
use zero_storage::{
    Cache, CacheQueryOptions, CacheRequest, CacheResponse, CacheStorage, CacheStorageSnapshot,
    ServiceWorkerRegistration, ServiceWorkerRegistry, ServiceWorkerScriptType, ServiceWorkerState,
    ServiceWorkerUpdateViaCache,
};

const DEFAULT_RUNTIME_LIMIT: usize = 32;
const MAX_SERVICE_WORKER_CLIENTS: usize = 128;
const MAX_CLIENTS_PER_VERSION: usize = 256;
const MAX_MESSAGES_PER_CLIENT: usize = 1024;
const MAX_URL_BYTES: usize = 64 * 1024;
const MAX_SCRIPT_BYTES: usize = 16 * 1024 * 1024;
const MAX_SCRIPT_GRAPH_BYTES: usize = 64 * 1024 * 1024;
const MAX_IMPORTED_SCRIPTS_PER_VERSION: usize = 1024;
const MAX_PENDING_FETCH_EVENTS: usize = 1024;
const CACHE_NAME_DOMSTRING_PREFIX: &str = "__zw_domstring16:";

fn is_service_worker_window_frame_type(frame_type: &str) -> bool {
    matches!(frame_type, "top-level" | "auxiliary" | "nested")
}

fn service_worker_registration_scope_url(
    key: &ServiceWorkerRegistrationKey,
) -> Result<String, ServiceWorkerManagerError> {
    let mut base = url::Url::parse(&key.origin)
        .map_err(|_| ServiceWorkerManagerError::InvalidInput("Service Worker origin is invalid".into()))?;
    base.set_path("/");
    base.set_query(None);
    base.set_fragment(None);
    let mut scope = base
        .join(&key.scope)
        .map_err(|_| ServiceWorkerManagerError::InvalidInput("Service Worker scope URL is invalid".into()))?;
    scope.set_fragment(None);
    Ok(scope.to_string())
}

fn service_worker_response_from_cache(
    response: &CacheResponse,
) -> Result<ServiceWorkerFetchResponse, ServiceWorkerManagerError> {
    let mut headers: Vec<(String, String)> = response
        .headers
        .iter()
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect();
    if !response.url.is_empty() {
        headers.push(("x-zero-final-url".into(), response.url.clone()));
    }
    Ok(ServiceWorkerFetchResponse {
        status: response.status,
        status_text: response.status_text.clone(),
        response_type: response.response_type.clone(),
        headers,
        body: String::from_utf8(response.body.clone()).map_err(|_| {
            ServiceWorkerManagerError::InvalidInput("cached Service Worker response body is not UTF-8".into())
        })?,
    })
}

fn cache_response_from_service_worker(response: ServiceWorkerFetchResponse) -> CacheResponse {
    let mut headers = response.headers;
    let url = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("x-zero-final-url"))
        .map(|(_, value)| value.clone())
        .unwrap_or_default();
    headers.retain(|(name, _)| !name.eq_ignore_ascii_case("x-zero-final-url"));
    CacheResponse {
        url,
        status: response.status,
        status_text: response.status_text,
        response_type: response.response_type,
        headers: headers.into_iter().collect(),
        body: response.body.into_bytes(),
    }
}

fn cache_query_options_from_service_worker(
    options: zero_script_sandbox::ServiceWorkerCacheQueryOptions,
) -> CacheQueryOptions {
    CacheQueryOptions {
        ignore_search: options.ignore_search,
        ignore_method: options.ignore_method,
        ignore_vary: options.ignore_vary,
    }
}

fn service_worker_request_from_cache(request: &CacheRequest) -> ServiceWorkerFetchRequest {
    ServiceWorkerFetchRequest {
        url: request.url.clone(),
        method: request.method.clone(),
        headers: request.headers.clone(),
        body: None,
        credentials: None,
        client_id: None,
        resulting_client_id: None,
        referrer: None,
        is_reload_navigation: request.is_reload_navigation,
        is_history_navigation: request.is_history_navigation,
    }
}

fn cache_request_from_service_worker(request: ServiceWorkerFetchRequest) -> CacheRequest {
    let mut cache_request = CacheRequest::with_method_and_headers(&request.url, &request.method, request.headers);
    cache_request.is_reload_navigation = request.is_reload_navigation;
    cache_request.is_history_navigation = request.is_history_navigation;
    cache_request
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

#[derive(Default)]
struct CacheStorageRuntimeState {
    next_cache_id: u64,
    active_cache_ids: HashMap<(u64, String), u64>,
    doomed_caches: HashMap<(u64, u64), Cache>,
}

impl CacheStorageRuntimeState {
    fn ensure_active_cache_id(&mut self, registration_id: u64, name: &str) -> u64 {
        let key = active_cache_key(registration_id, name);
        if let Some(id) = self.active_cache_ids.get(&key) {
            return *id;
        }
        self.next_cache_id = self.next_cache_id.saturating_add(1).max(1);
        let id = self.next_cache_id;
        self.active_cache_ids.insert(key, id);
        id
    }

    fn remove_active_cache_id(&mut self, registration_id: u64, name: &str) -> Option<u64> {
        self.active_cache_ids.remove(&active_cache_key(registration_id, name))
    }

    fn active_cache_id_matches(&self, registration_id: u64, name: &str, cache_id: u64) -> bool {
        self.active_cache_ids
            .get(&active_cache_key(registration_id, name))
            .is_some_and(|active| *active == cache_id)
    }

    fn selected_cache_ref<'a>(
        &'a self,
        registration: &'a ServiceWorkerRegistration,
        name: &str,
        cache_id: Option<u64>,
    ) -> Option<&'a Cache> {
        match cache_id {
            Some(cache_id) => self
                .doomed_caches
                .get(&cache_instance_key(registration.id, cache_id))
                .or_else(|| {
                    if self.active_cache_id_matches(registration.id, name, cache_id) {
                        registration.cache_storage.get(name)
                    } else {
                        None
                    }
                }),
            None => registration.cache_storage.get(name),
        }
    }

    fn selected_cache_mut<'a>(
        &'a mut self,
        registration: &'a mut ServiceWorkerRegistration,
        name: &str,
        cache_id: Option<u64>,
    ) -> Option<&'a mut Cache> {
        match cache_id {
            Some(cache_id) => {
                if self
                    .doomed_caches
                    .contains_key(&cache_instance_key(registration.id, cache_id))
                {
                    self.doomed_caches
                        .get_mut(&cache_instance_key(registration.id, cache_id))
                } else if self.active_cache_id_matches(registration.id, name, cache_id) {
                    registration.cache_storage.get_mut(name)
                } else {
                    None
                }
            }
            None => registration.cache_storage.get_mut(name),
        }
    }
}

fn active_cache_key(registration_id: u64, name: &str) -> (u64, String) {
    (registration_id, name.to_string())
}

fn cache_instance_key(registration_id: u64, cache_id: u64) -> (u64, u64) {
    (registration_id, cache_id)
}

struct EvaluationOptions {
    update_via_cache: ServiceWorkerUpdateViaCache,
    script_type: ServiceWorkerScriptType,
    restoring_active: bool,
    restored_imported_scripts: Vec<ServiceWorkerImportedScript>,
    restored_cache_storage: CacheStorage,
}

#[derive(Debug, Clone)]
struct ClientRecord {
    info: ServiceWorkerClientInfo,
    controller_registration_id: Option<u64>,
    creation_sequence: u64,
    last_focus_sequence: Option<u64>,
}

#[derive(Debug, Clone)]
struct PendingFetchRecord {
    registration_id: u64,
    request_url: String,
    client_id: Option<String>,
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

/// Result of queuing a fetch through an active Service Worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceWorkerFetchDispatch {
    /// A matching active worker accepted the fetch event.
    Dispatched {
        /// Active registration version that owns the fetch event.
        registration_id: u64,
        /// Host-assigned fetch event ID.
        event_id: u64,
    },
    /// No same-origin active worker controls this request.
    PassThrough,
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
    /// Registration-local CacheStorage owned by this Service Worker.
    #[serde(default)]
    pub cache_storage: CacheStorageSnapshot,
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
    /// Activate result moved the version to active.
    ActivationCompleted {
        /// Registration version ID.
        registration_id: u64,
        /// Whether activation completed.
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
    /// An active worker requested a new browser-owned update fetch.
    WorkerUpdateRequested {
        /// Worker version that called `registration.update()`.
        caller_registration_id: u64,
        /// Runtime-local request ID.
        request_id: u64,
        /// Current registration version whose script must be fetched.
        target_registration_id: u64,
    },
    /// A worker-global `registration.unregister()` request completed.
    WorkerUnregistered {
        /// Worker version that called `registration.unregister()`.
        registration_id: u64,
        /// Runtime-local request ID.
        request_id: u64,
        /// Whether a registration was removed.
        removed: bool,
    },
    /// A worker-global `fetch()` call is blocked on a browser-owned network request.
    WorkerFetchRequested {
        /// Active registration version that called `fetch()`.
        registration_id: u64,
        /// Runtime-local request ID.
        request_id: u64,
        /// Pure-value network request.
        request: ServiceWorkerFetchRequest,
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
    /// A fetch event settled after optional `respondWith()` handling.
    FetchSettled {
        /// Active registration version that handled the fetch event.
        registration_id: u64,
        /// Host-assigned fetch event ID.
        event_id: u64,
        /// Request URL associated with this fetch event.
        request_url: String,
        /// Source client identity, when known.
        client_id: Option<String>,
        /// Response supplied through `respondWith()`, or `None` for pass-through/failure.
        response: Option<ServiceWorkerFetchResponse>,
        /// True when `respondWith()` was called but failed, producing a network error.
        failed: bool,
        /// Handler or response-conversion diagnostic. Empty means success or pass-through.
        message: String,
    },
    /// A registration-local CacheStorage mutation completed successfully.
    CacheStorageMutated {
        /// Registration version whose CacheStorage changed.
        registration_id: u64,
    },
    /// An active worker requested control of matching clients.
    ClientsClaimed {
        /// Registration version whose matching clients are now controlled.
        registration_id: u64,
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
    client_messages: HashMap<(u64, String), Vec<Vec<ServiceWorkerOutboundMessage>>>,
    pending_client_messages: HashMap<(u64, String), usize>,
    message_ports: HashSet<(u64, String, u64)>,
    worker_message_ports: HashMap<(u64, u64), u64>,
    clients: HashMap<String, ClientRecord>,
    unregistered_active: HashSet<u64>,
    next_client_sequence: u64,
    next_client_focus_sequence: u64,
    script_sources: HashMap<u64, Vec<u8>>,
    imported_scripts: HashMap<u64, HashMap<String, Vec<u8>>>,
    pending_import_requests: HashMap<(u64, u64), Vec<String>>,
    update_predecessors: HashMap<u64, u64>,
    pending_worker_updates: HashMap<u64, (u64, u64)>,
    pending_fetch_events: HashMap<(u64, u64), PendingFetchRecord>,
    cache_storage_state: CacheStorageRuntimeState,
    host: Box<dyn ServiceWorkerRuntimeHost>,
    evaluated: HashSet<u64>,
    restoring_active: HashSet<u64>,
    runtime_limit: usize,
    next_worker_message_event_id: u64,
}

/// One page-to-worker message dispatch with manager-owned state attached.
pub struct ServiceWorkerClientMessageDispatch<'a> {
    /// Host-assigned event ID.
    pub event_id: u64,
    /// JSON structured-message payload.
    pub data_json: &'a str,
    /// Browser-owned identity of the originating client.
    pub client_id: &'a str,
    /// Originating client URL.
    pub client_url: &'a str,
    /// Frame type of the originating window client.
    pub client_frame_type: &'a str,
    /// Whether the originating window client currently has focus.
    pub client_focused: bool,
    /// MessagePort metadata for transferred or addressed endpoints.
    pub ports: &'a ServiceWorkerMessagePorts,
    /// Whether `clients.claim()` is allowed for this dispatched event.
    pub clients_claim_allowed: bool,
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
        scope_url: &str,
        script: &str,
        script_type: ServiceWorkerScriptType,
        initial_peers: ServiceWorkerRegistrationPeers,
    ) -> Result<(), ServiceWorkerManagerError>;
    /// Dispatch the install or activate event inside one live runtime.
    fn dispatch_lifecycle(
        &mut self,
        registration_id: u64,
        phase: ServiceWorkerLifecyclePhase,
        clients_claim_allowed: bool,
    ) -> Result<(), ServiceWorkerManagerError>;
    /// Dispatch one JSON-compatible page message into a live runtime.
    fn dispatch_client_message(
        &mut self,
        registration_id: u64,
        message: ServiceWorkerClientMessageDispatch<'_>,
    ) -> Result<(), ServiceWorkerManagerError>;
    /// Dispatch one worker-to-worker message into a live runtime.
    fn dispatch_worker_message(
        &mut self,
        registration_id: u64,
        event_id: u64,
        message: ServiceWorkerWorkerMessage,
        clients_claim_allowed: bool,
    ) -> Result<(), ServiceWorkerManagerError>;
    /// Synchronize one runtime's visible registration slot projections.
    fn sync_registration_peers(
        &mut self,
        registration_id: u64,
        peers: ServiceWorkerRegistrationPeers,
    ) -> Result<(), ServiceWorkerManagerError>;
    /// Dispatch one fetch event into a live runtime.
    fn dispatch_fetch(
        &mut self,
        registration_id: u64,
        event_id: u64,
        request: ServiceWorkerFetchRequest,
        clients_claim_allowed: bool,
    ) -> Result<(), ServiceWorkerManagerError>;
    /// Forward one page-side fetch body cancel/abort to the runtime.
    ///
    /// Default no-op: hosts whose transport cannot reach the runtime after
    /// settlement (browser IPC page fetches) silently ignore the signal —
    /// in-process hosts override to make worker-side cancel observable.
    fn cancel_fetch_body(
        &mut self,
        _registration_id: u64,
        _event_id: u64,
        _reason: &str,
    ) -> Result<(), ServiceWorkerManagerError> {
        Ok(())
    }
    /// Complete one blocking `importScripts()` request.
    fn complete_import_scripts(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<Vec<String>, String>,
    ) -> Result<(), ServiceWorkerManagerError>;
    /// Complete one worker-global `registration.update()` request.
    fn complete_update(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<(), (String, String)>,
    ) -> Result<(), ServiceWorkerManagerError>;
    /// Complete one worker-global `registration.unregister()` request.
    fn complete_unregister(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<bool, (String, String)>,
    ) -> Result<(), ServiceWorkerManagerError>;
    /// Complete one worker-global `clients.matchAll()` request.
    fn complete_clients_match_all(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<Vec<ServiceWorkerClientInfo>, String>,
    ) -> Result<(), ServiceWorkerManagerError>;
    /// Complete one worker-global `clients.get()` request.
    fn complete_clients_get(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<Option<ServiceWorkerClientInfo>, String>,
    ) -> Result<(), ServiceWorkerManagerError>;
    /// Complete one worker-global CacheStorage request.
    fn complete_cache_storage(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<ServiceWorkerCacheStorageResult, String>,
    ) -> Result<(), ServiceWorkerManagerError>;
    /// Complete one worker-global ordinary fetch request.
    fn complete_fetch(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<ServiceWorkerFetchResponse, String>,
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
        scope_url: &str,
        script: &str,
        script_type: ServiceWorkerScriptType,
        initial_peers: ServiceWorkerRegistrationPeers,
    ) -> Result<(), ServiceWorkerManagerError> {
        let mut runtime = ServiceWorkerRuntime::new(self.config.clone())
            .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))?;
        match script_type {
            ServiceWorkerScriptType::Classic => {
                runtime.evaluate_with_scope_and_peers(script, script_url, scope_url, initial_peers)
            }
            ServiceWorkerScriptType::Module => {
                runtime.evaluate_module_with_scope_and_peers(script, script_url, scope_url, initial_peers)
            }
        }
        .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))?;
        self.runtimes.insert(registration_id, runtime);
        Ok(())
    }

    fn dispatch_lifecycle(
        &mut self,
        registration_id: u64,
        phase: ServiceWorkerLifecyclePhase,
        clients_claim_allowed: bool,
    ) -> Result<(), ServiceWorkerManagerError> {
        let runtime = self
            .runtimes
            .get_mut(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        let result = match phase {
            ServiceWorkerLifecyclePhase::Install => runtime.dispatch_lifecycle_with_claim_allowed(
                registration_id,
                ServiceWorkerLifecyclePhase::Install,
                clients_claim_allowed,
            ),
            ServiceWorkerLifecyclePhase::Activate => runtime.dispatch_lifecycle_with_claim_allowed(
                registration_id,
                ServiceWorkerLifecyclePhase::Activate,
                clients_claim_allowed,
            ),
        };
        result.map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))
    }

    fn dispatch_client_message(
        &mut self,
        registration_id: u64,
        message: ServiceWorkerClientMessageDispatch<'_>,
    ) -> Result<(), ServiceWorkerManagerError> {
        let runtime = self
            .runtimes
            .get_mut(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        runtime
            .dispatch_message_with_ports_with_claim_allowed(
                message.event_id,
                message.data_json,
                ServiceWorkerMessageSource {
                    client_id: message.client_id,
                    client_url: message.client_url,
                    client_frame_type: message.client_frame_type,
                    client_focused: message.client_focused,
                },
                message.ports,
                message.clients_claim_allowed,
            )
            .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))
    }

    fn dispatch_worker_message(
        &mut self,
        registration_id: u64,
        event_id: u64,
        message: ServiceWorkerWorkerMessage,
        clients_claim_allowed: bool,
    ) -> Result<(), ServiceWorkerManagerError> {
        let runtime = self
            .runtimes
            .get_mut(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        runtime
            .dispatch_worker_message(
                event_id,
                &message.data_json,
                message.source,
                &ServiceWorkerMessagePorts {
                    transferred_port_ids: message.transferred_port_ids,
                    data_port_index: message.data_port_index,
                    target_port_id: message.target_port_id,
                },
                clients_claim_allowed,
            )
            .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))
    }

    fn sync_registration_peers(
        &mut self,
        registration_id: u64,
        peers: ServiceWorkerRegistrationPeers,
    ) -> Result<(), ServiceWorkerManagerError> {
        let runtime = self
            .runtimes
            .get_mut(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        runtime
            .sync_registration_peers(registration_id, peers)
            .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))
    }

    fn dispatch_fetch(
        &mut self,
        registration_id: u64,
        event_id: u64,
        request: ServiceWorkerFetchRequest,
        clients_claim_allowed: bool,
    ) -> Result<(), ServiceWorkerManagerError> {
        let runtime = self
            .runtimes
            .get_mut(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        runtime
            .dispatch_fetch_with_claim_allowed(event_id, request, clients_claim_allowed)
            .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))
    }

    fn cancel_fetch_body(
        &mut self,
        registration_id: u64,
        event_id: u64,
        reason: &str,
    ) -> Result<(), ServiceWorkerManagerError> {
        let runtime = self
            .runtimes
            .get_mut(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        runtime
            .cancel_fetch_body(event_id, reason.to_string())
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

    fn complete_update(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<(), (String, String)>,
    ) -> Result<(), ServiceWorkerManagerError> {
        self.runtimes
            .get(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?
            .complete_update(request_id, result)
            .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))
    }

    fn complete_unregister(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<bool, (String, String)>,
    ) -> Result<(), ServiceWorkerManagerError> {
        self.runtimes
            .get(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?
            .complete_unregister(request_id, result)
            .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))
    }

    fn complete_clients_match_all(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<Vec<ServiceWorkerClientInfo>, String>,
    ) -> Result<(), ServiceWorkerManagerError> {
        self.runtimes
            .get(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?
            .complete_clients_match_all(request_id, result)
            .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))
    }

    fn complete_clients_get(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<Option<ServiceWorkerClientInfo>, String>,
    ) -> Result<(), ServiceWorkerManagerError> {
        self.runtimes
            .get(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?
            .complete_clients_get(request_id, result)
            .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))
    }

    fn complete_cache_storage(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<ServiceWorkerCacheStorageResult, String>,
    ) -> Result<(), ServiceWorkerManagerError> {
        self.runtimes
            .get(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?
            .complete_cache_storage(request_id, result)
            .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))
    }

    fn complete_fetch(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<ServiceWorkerFetchResponse, String>,
    ) -> Result<(), ServiceWorkerManagerError> {
        self.runtimes
            .get(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?
            .complete_fetch(request_id, result)
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
        self.runtimes.values().filter(|runtime| runtime.is_running()).count()
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
            message_ports: HashSet::new(),
            worker_message_ports: HashMap::new(),
            clients: HashMap::new(),
            unregistered_active: HashSet::new(),
            next_client_sequence: 1,
            next_client_focus_sequence: 1,
            script_sources: HashMap::new(),
            imported_scripts: HashMap::new(),
            pending_import_requests: HashMap::new(),
            update_predecessors: HashMap::new(),
            pending_worker_updates: HashMap::new(),
            pending_fetch_events: HashMap::new(),
            cache_storage_state: CacheStorageRuntimeState::default(),
            host,
            evaluated: HashSet::new(),
            restoring_active: HashSet::new(),
            runtime_limit: DEFAULT_RUNTIME_LIMIT,
            next_worker_message_event_id: 1,
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
                restored_cache_storage: CacheStorage::new(),
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
                restored_cache_storage: CacheStorage::new(),
            },
        )
    }

    /// Start a registration job, comparing it with the existing version for
    /// the same origin and scope when one exists.
    pub fn start_registration(
        &mut self,
        script_url: &str,
        scope: &str,
        origin: &str,
        script: &str,
        script_type: ServiceWorkerScriptType,
        update_via_cache: ServiceWorkerUpdateViaCache,
    ) -> Result<u64, ServiceWorkerManagerError> {
        let key = ServiceWorkerRegistrationKey::new(origin, scope)?;
        let predecessor = self.slots.get(&key).and_then(|slot| slot.waiting.or(slot.active));
        let registration_id =
            self.start_evaluation_with_options(script_url, scope, origin, script, script_type, update_via_cache)?;
        if let Some(predecessor) = predecessor {
            self.update_predecessors.insert(registration_id, predecessor);
        }
        Ok(registration_id)
    }

    /// Return the current version when a registration job is an exact option match.
    pub fn matching_registration(
        &self,
        script_url: &str,
        scope: &str,
        origin: &str,
        script_type: ServiceWorkerScriptType,
        update_via_cache: ServiceWorkerUpdateViaCache,
    ) -> Result<Option<u64>, ServiceWorkerManagerError> {
        let key = ServiceWorkerRegistrationKey::new(origin, scope)?;
        Ok(self
            .slots
            .get(&key)
            .and_then(|slot| slot.waiting.or(slot.active))
            .and_then(|registration_id| self.registration(registration_id))
            .filter(|registration| {
                registration.script_url == script_url
                    && registration.script_type == script_type
                    && registration.update_via_cache == update_via_cache
            })
            .map(|registration| registration.id))
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
            cache_storage,
        } = registration;
        let cache_storage = CacheStorage::from_snapshot(cache_storage).map_err(|error| {
            ServiceWorkerManagerError::InvalidInput(format!("invalid Service Worker CacheStorage: {error}"))
        })?;
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
                restored_cache_storage: cache_storage,
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
        registration.cache_storage = options.restored_cache_storage;
        let scope_url = service_worker_registration_scope_url(&key)?;
        let initial_peers = self.registration_peers_for_key(&key)?;
        if let Err(error) = self
            .host
            .evaluate(id, script_url, &scope_url, script, options.script_type, initial_peers)
        {
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

    /// Return an installing candidate that an explicit client update can reuse.
    ///
    /// The boolean reports whether the candidate is a changed replacement.
    /// Updating during the initial installation succeeds without creating a
    /// second job, but must not report another updatefound transition.
    pub fn coalesced_update_candidate(
        &self,
        registration_id: u64,
    ) -> Result<Option<(u64, bool)>, ServiceWorkerManagerError> {
        let key = self.key_for(registration_id)?;
        let slot = self
            .slots
            .get(key)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        Ok(slot
            .installing
            .map(|candidate_id| (candidate_id, slot.active.is_some() || slot.waiting.is_some())))
    }

    /// Complete the browser-owned fetch for a worker-global update request.
    pub fn complete_worker_update_fetch(
        &mut self,
        caller_registration_id: u64,
        request_id: u64,
        result: Result<String, (String, String)>,
    ) -> Result<(), ServiceWorkerManagerError> {
        let script = match result {
            Ok(script) => script,
            Err(error) => {
                return self
                    .host
                    .complete_update(caller_registration_id, request_id, Err(error));
            }
        };
        match self.start_update(caller_registration_id, &script) {
            Ok(ServiceWorkerUpdateOutcome::Unchanged { .. }) | Err(ServiceWorkerManagerError::JobInProgress(_)) => {
                self.host.complete_update(caller_registration_id, request_id, Ok(()))
            }
            Ok(ServiceWorkerUpdateOutcome::Started { registration_id }) => {
                self.pending_worker_updates
                    .insert(registration_id, (caller_registration_id, request_id));
                Ok(())
            }
            Err(error) => self.host.complete_update(
                caller_registration_id,
                request_id,
                Err(("TypeError".into(), error.to_string())),
            ),
        }
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
                        if let Some((caller_registration_id, request_id)) =
                            self.pending_worker_updates.remove(&registration_id)
                        {
                            let _ = self.host.complete_update(caller_registration_id, request_id, Ok(()));
                        }
                        let update_via_cache = self
                            .registration(registration_id)
                            .map(|registration| registration.update_via_cache);
                        if let (Some(update_via_cache), Some(current)) =
                            (update_via_cache, self.registry.get_mut(current_id))
                        {
                            current.update_via_cache = update_via_cache;
                        }
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
                    if let Some((caller_registration_id, request_id)) =
                        self.pending_worker_updates.remove(&registration_id)
                    {
                        let _ = self.host.complete_update(
                            caller_registration_id,
                            request_id,
                            Err(("TypeError".into(), message.clone())),
                        );
                    }
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
                ServiceWorkerEvent::UpdateRequested { request_id } => {
                    match self.worker_update_target(registration_id) {
                        Ok(Some(target_registration_id)) => {
                            output.push(ServiceWorkerManagerEvent::WorkerUpdateRequested {
                                caller_registration_id: registration_id,
                                request_id,
                                target_registration_id,
                            });
                        }
                        Ok(None) => {
                            let _ = self.host.complete_update(registration_id, request_id, Ok(()));
                        }
                        Err(error) => {
                            let _ = self.host.complete_update(
                                registration_id,
                                request_id,
                                Err(("InvalidStateError".into(), error.to_string())),
                            );
                        }
                    }
                }
                ServiceWorkerEvent::UnregisterRequested { request_id } => {
                    let removed = self.unregister_from_worker_global(registration_id);
                    if let Err(error) = self.host.complete_unregister(registration_id, request_id, Ok(removed)) {
                        output.push(ServiceWorkerManagerEvent::CoordinationFailed {
                            registration_id,
                            message: error.to_string(),
                        });
                    }
                    output.push(ServiceWorkerManagerEvent::WorkerUnregistered {
                        registration_id,
                        request_id,
                        removed,
                    });
                }
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
                    if self
                        .registry
                        .get(registration_id)
                        .is_some_and(|registration| registration.state == ServiceWorkerState::Redundant)
                    {
                        continue;
                    }
                    if phase == ServiceWorkerLifecyclePhase::Activate && succeeded && claim_clients {
                        self.claimed_clients.insert(registration_id);
                        self.claim_matching_clients(registration_id);
                    }
                    let lifecycle_succeeded = match phase {
                        ServiceWorkerLifecyclePhase::Install => succeeded,
                        // https://w3c.github.io/ServiceWorker/#activation-algorithm
                        ServiceWorkerLifecyclePhase::Activate => true,
                    };
                    let transition = match phase {
                        ServiceWorkerLifecyclePhase::Install => self.apply_install_result(registration_id, succeeded),
                        ServiceWorkerLifecyclePhase::Activate => {
                            self.apply_activation_result(registration_id, lifecycle_succeeded)
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
                    if let Err(error) = self.record_outbound_message_ports(registration_id, &client_id, &outbound) {
                        self.complete_client_message_batch(registration_id, &client_id, Vec::new());
                        output.push(ServiceWorkerManagerEvent::MessageFailed {
                            registration_id,
                            event_id,
                            client_id,
                            message: error.to_string(),
                        });
                        continue;
                    }
                    self.complete_routed_client_messages(registration_id, Some(&client_id), outbound);
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
                ServiceWorkerEvent::FetchSettled {
                    event_id,
                    request_url,
                    response,
                    failed,
                    message,
                } => {
                    let pending = self.pending_fetch_events.remove(&(registration_id, event_id));
                    output.push(ServiceWorkerManagerEvent::FetchSettled {
                        registration_id,
                        event_id,
                        request_url: pending
                            .as_ref()
                            .map(|record| record.request_url.clone())
                            .unwrap_or(request_url),
                        client_id: pending.and_then(|record| {
                            (record.registration_id == registration_id)
                                .then_some(record.client_id)
                                .flatten()
                        }),
                        response,
                        failed,
                        message,
                    });
                }
                ServiceWorkerEvent::ClientsMatchAllRequested {
                    request_id,
                    include_uncontrolled,
                    client_type,
                } => {
                    let result = self.clients_for_worker(registration_id, include_uncontrolled, &client_type);
                    let _ = self.host.complete_clients_match_all(
                        registration_id,
                        request_id,
                        result.map_err(|error| error.to_string()),
                    );
                }
                ServiceWorkerEvent::ClientsGetRequested { request_id, client_id } => {
                    let result = self.client_for_worker(registration_id, &client_id);
                    let _ = self.host.complete_clients_get(
                        registration_id,
                        request_id,
                        result.map_err(|error| error.to_string()),
                    );
                }
                ServiceWorkerEvent::CacheStorageRequested { request_id, request } => {
                    let mutates_cache_storage = matches!(
                        &request,
                        ServiceWorkerCacheStorageRequest::Open { .. }
                            | ServiceWorkerCacheStorageRequest::Delete { .. }
                            | ServiceWorkerCacheStorageRequest::Put { .. }
                            | ServiceWorkerCacheStorageRequest::StorageDelete { .. }
                    );
                    let result = self.cache_storage_for_worker(registration_id, request);
                    if mutates_cache_storage && result.is_ok() {
                        output.push(ServiceWorkerManagerEvent::CacheStorageMutated { registration_id });
                    }
                    let _ = self.host.complete_cache_storage(
                        registration_id,
                        request_id,
                        result.map_err(|error| error.to_string()),
                    );
                }
                ServiceWorkerEvent::FetchRequested { request_id, request } => {
                    output.push(ServiceWorkerManagerEvent::WorkerFetchRequested {
                        registration_id,
                        request_id,
                        request,
                    });
                }
                ServiceWorkerEvent::ClientMessagesEmitted { outbound } => {
                    let pending_fetch_client_id = self.pending_fetch_client_id(registration_id);
                    let (outbound, failures) = self.route_worker_port_messages(registration_id, outbound);
                    output.extend(failures);
                    if !outbound.is_empty()
                        && self
                            .record_fetch_outbound_message_ports(
                                registration_id,
                                pending_fetch_client_id.as_deref(),
                                &outbound,
                            )
                            .is_ok()
                    {
                        self.route_client_messages(registration_id, pending_fetch_client_id.as_deref(), outbound);
                    }
                }
                ServiceWorkerEvent::WorkerMessagesEmitted { messages } => {
                    for message in messages {
                        if let Err(error) = self.dispatch_worker_message(registration_id, message) {
                            output.push(ServiceWorkerManagerEvent::CoordinationFailed {
                                registration_id,
                                message: error.to_string(),
                            });
                        }
                    }
                }
                ServiceWorkerEvent::ClientsClaimRequested => {
                    if self
                        .registry
                        .get(registration_id)
                        .is_some_and(|registration| registration.state == ServiceWorkerState::Activated)
                    {
                        self.claimed_clients.insert(registration_id);
                        self.claim_matching_clients(registration_id);
                        output.push(ServiceWorkerManagerEvent::ClientsClaimed { registration_id });
                    }
                }
            }
        }
        output
    }

    fn script_graphs_equal(&self, left: u64, right: u64) -> bool {
        self.script_sources.get(&left) == self.script_sources.get(&right)
            && self.imported_scripts.get(&left) == self.imported_scripts.get(&right)
            && self
                .registration(left)
                .map(|registration| registration.script_url.as_str())
                == self
                    .registration(right)
                    .map(|registration| registration.script_url.as_str())
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

    /// Complete one browser-owned network request initiated by worker-global `fetch()`.
    pub fn complete_worker_fetch(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<ServiceWorkerFetchResponse, String>,
    ) -> Result<(), ServiceWorkerManagerError> {
        self.host.complete_fetch(registration_id, request_id, result)
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
            .map_err(|_| ServiceWorkerManagerError::InvalidInput("invalid Service Worker origin".into()))?;
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
                    || (registration_origin.scheme() == "https" && url.scheme() != "https")
                {
                    return Err(ServiceWorkerManagerError::InvalidInput(
                        "Service Worker module URL is not fetchable".into(),
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
        self.sync_runtime_registration_peers(registration_id)?;
        self.host
            .dispatch_lifecycle(registration_id, ServiceWorkerLifecyclePhase::Install, false)
    }

    /// Move a waiting version into the activating state.
    fn begin_activation(&mut self, registration_id: u64) -> Result<(), ServiceWorkerManagerError> {
        self.require_state(registration_id, ServiceWorkerState::Installed)?;
        let key = self.key_for(registration_id)?.clone();
        let old_active = {
            let slot = self
                .slots
                .get(&key)
                .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
            if slot.waiting != Some(registration_id) {
                return Err(ServiceWorkerManagerError::InvalidState {
                    registration_id,
                    expected: ServiceWorkerState::Installed,
                    actual: ServiceWorkerState::Redundant,
                });
            }
            slot.active
        };
        if old_active == Some(registration_id) {
            return Err(ServiceWorkerManagerError::InvalidState {
                registration_id,
                expected: ServiceWorkerState::Installed,
                actual: ServiceWorkerState::Activated,
            });
        }
        self.registry
            .get_mut(registration_id)
            .expect("validated registration must exist")
            .state = ServiceWorkerState::Activating;
        self.record_state_change(registration_id, ServiceWorkerState::Activating);
        if let Some(old_active) = old_active {
            // https://w3c.github.io/ServiceWorker/#activation-algorithm
            self.transfer_controlled_clients(old_active, registration_id);
        }
        Ok(())
    }

    /// Dispatch the real activate event for an activating version.
    fn dispatch_activate(&mut self, registration_id: u64) -> Result<(), ServiceWorkerManagerError> {
        self.require_state(registration_id, ServiceWorkerState::Activating)?;
        self.sync_runtime_registration_peers(registration_id)?;
        self.host
            .dispatch_lifecycle(registration_id, ServiceWorkerLifecyclePhase::Activate, true)
    }

    /// Apply the result of the activate event and its lifetime promises.
    fn apply_activation_result(
        &mut self,
        registration_id: u64,
        succeeded: bool,
    ) -> Result<ServiceWorkerManagerEvent, ServiceWorkerManagerError> {
        self.require_state(registration_id, ServiceWorkerState::Activating)?;
        let key = self.key_for(registration_id)?.clone();
        let old_active = {
            let slot = self.slots.get_mut(&key).expect("registration key must have slots");
            if slot.waiting != Some(registration_id) {
                return Err(ServiceWorkerManagerError::InvalidState {
                    registration_id,
                    expected: ServiceWorkerState::Activating,
                    actual: ServiceWorkerState::Redundant,
                });
            }
            slot.waiting = None;
            succeeded.then(|| slot.active.replace(registration_id)).flatten()
        };

        if succeeded {
            if let Some(old_active) = old_active {
                self.mark_redundant_and_stop(old_active);
            }
            self.registry
                .get_mut(registration_id)
                .expect("validated registration must exist")
                .state = ServiceWorkerState::Activated;
            self.record_state_change(registration_id, ServiceWorkerState::Activated);
        } else {
            if let Some(old_active) = self.slots.get(&key).and_then(|slot| slot.active) {
                self.transfer_controlled_clients(registration_id, old_active);
            }
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

    /// Remove a registration from future matching.
    ///
    /// https://w3c.github.io/ServiceWorker/#navigator-service-worker-unregister
    /// A controlled client keeps using its incumbent active worker until the
    /// client is discarded; later navigations no longer match this registration.
    pub fn unregister(&mut self, registration_id: u64) -> bool {
        let Some(key) = self.registration_keys.get(&registration_id).cloned() else {
            return false;
        };
        let Some(slots) = self.slots.remove(&key) else {
            return false;
        };
        let mut removed = false;
        for id in [slots.installing, slots.waiting].into_iter().flatten() {
            self.mark_redundant_and_stop(id);
            self.forget_registration(id);
            removed = true;
        }
        if let Some(active_id) = slots.active {
            if self.client_is_controlled_by(active_id) {
                self.unregistered_active.insert(active_id);
                removed = true;
            } else {
                self.mark_redundant_and_stop(active_id);
                self.forget_registration(active_id);
                removed = true;
            }
        }
        removed
    }

    fn unregister_from_worker_global(&mut self, registration_id: u64) -> bool {
        let Some(key) = self.registration_keys.get(&registration_id).cloned() else {
            return false;
        };
        let Some(slots) = self.slots.remove(&key) else {
            return false;
        };
        let mut removed = false;
        for id in [slots.installing, slots.waiting].into_iter().flatten() {
            if id == registration_id {
                self.mark_redundant_after_worker_unregister(id);
                removed = true;
            } else {
                self.mark_redundant_and_stop(id);
                self.forget_registration(id);
                removed = true;
            }
        }
        if let Some(active_id) = slots.active {
            if active_id == registration_id {
                if self.client_is_controlled_by(active_id) {
                    self.unregistered_active.insert(active_id);
                } else {
                    self.mark_redundant_after_worker_unregister(active_id);
                }
                removed = true;
            } else {
                if self.client_is_controlled_by(active_id) {
                    self.unregistered_active.insert(active_id);
                } else {
                    self.mark_redundant_and_stop(active_id);
                    self.forget_registration(active_id);
                }
                removed = true;
            }
        }
        removed
    }

    /// Inspect one registration version.
    pub fn registration(&self, registration_id: u64) -> Option<&ServiceWorkerRegistration> {
        self.registry.get(registration_id)
    }

    /// Insert one response into a registration's browser-owned CacheStorage.
    pub fn put_cached_response(
        &mut self,
        registration_id: u64,
        cache_name: &str,
        request: CacheRequest,
        response: CacheResponse,
    ) -> Result<(), ServiceWorkerManagerError> {
        self.registry
            .get_mut(registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?
            .cache_storage
            .open(cache_name)
            .put(request, response)
            .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))
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

    fn active_registration_id_for_url(&self, origin: &str, url: &str) -> Option<u64> {
        self.active_registration_for_url(origin, url)
            .map(|registration| registration.id)
    }

    fn client_is_controlled_by(&self, registration_id: u64) -> bool {
        self.clients
            .values()
            .any(|record| record.controller_registration_id == Some(registration_id))
    }

    /// Return whether any known client is controlled by this registration version.
    pub fn has_controlled_client(&self, registration_id: u64) -> Result<bool, ServiceWorkerManagerError> {
        if self.registry.get(registration_id).is_none() {
            return Err(ServiceWorkerManagerError::UnknownRegistration(registration_id));
        }
        Ok(self.client_is_controlled_by(registration_id))
    }

    fn forget_registration(&mut self, registration_id: u64) {
        self.registration_keys.remove(&registration_id);
        self.state_changes.remove(&registration_id);
        self.registry.unregister(registration_id);
        self.unregistered_active.remove(&registration_id);
    }

    fn claim_matching_clients(&mut self, registration_id: u64) {
        let Some(registration) = self.registry.get(registration_id).cloned() else {
            return;
        };
        let claimable_client_ids = self
            .clients
            .values()
            .filter(|record| {
                let Ok(url) = url::Url::parse(&record.info.url) else {
                    return false;
                };
                url.origin().ascii_serialization() == registration.origin
                    && registration.is_in_scope(&record.info.url)
                    && self
                        .registration_for_url(&registration.origin, &record.info.url)
                        .is_some_and(|matched| matched.id == registration_id)
            })
            .map(|record| record.info.id.clone())
            .collect::<Vec<_>>();
        for client_id in claimable_client_ids {
            let Some(record) = self.clients.get_mut(&client_id) else {
                continue;
            };
            record.controller_registration_id = Some(registration_id);
        }
    }

    fn transfer_controlled_clients(&mut self, old_registration_id: u64, new_registration_id: u64) {
        for record in self.clients.values_mut() {
            if record.controller_registration_id == Some(old_registration_id) {
                record.controller_registration_id = Some(new_registration_id);
            }
        }
    }

    fn clear_controller_for_registration(&mut self, registration_id: u64) {
        for record in self.clients.values_mut() {
            if record.controller_registration_id == Some(registration_id) {
                record.controller_registration_id = None;
            }
        }
    }

    fn collect_uncontrolled_unregistered_workers(&mut self) {
        let stale = self
            .unregistered_active
            .iter()
            .copied()
            .filter(|registration_id| !self.client_is_controlled_by(*registration_id))
            .collect::<Vec<_>>();
        for registration_id in stale {
            self.mark_redundant_and_stop(registration_id);
            self.forget_registration(registration_id);
        }
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
        self.post_message_with_ports(
            registration_id,
            event_id,
            data_json,
            client_id,
            client_url,
            &ServiceWorkerMessagePorts::default(),
        )
    }

    /// Queue a page or MessagePort message with transferred endpoint metadata.
    pub fn post_message_with_ports(
        &mut self,
        registration_id: u64,
        event_id: u64,
        data_json: &str,
        client_id: &str,
        client_url: &str,
        ports: &ServiceWorkerMessagePorts,
    ) -> Result<(), ServiceWorkerManagerError> {
        let known_client_before_dispatch = self.clients.contains_key(client_id);
        if !known_client_before_dispatch {
            self.observe_window_client(client_id, client_url)?;
        }
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
        if ports.transferred_port_ids.len() > 16
            || ports.transferred_port_ids.contains(&0)
            || ports.transferred_port_ids.iter().collect::<HashSet<_>>().len() != ports.transferred_port_ids.len()
            || ports
                .data_port_index
                .is_some_and(|index| index >= ports.transferred_port_ids.len())
            || ports.target_port_id == Some(0)
        {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "invalid Service Worker MessagePort metadata".into(),
            ));
        }
        if let Some(port_id) = ports.target_port_id
            && !self
                .message_ports
                .contains(&(registration_id, client_id.to_string(), port_id))
        {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker MessagePort endpoint does not exist".into(),
            ));
        }
        if ports.transferred_port_ids.iter().any(|&port_id| {
            self.message_ports
                .contains(&(registration_id, client_id.to_string(), port_id))
        }) {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker MessagePort endpoint was already transferred".into(),
            ));
        }
        let key = (registration_id, client_id.to_string());
        let client_frame_type = self
            .clients
            .get(client_id)
            .map(|record| record.info.frame_type.clone())
            .unwrap_or_else(|| "top-level".to_string());
        let has_focused_window = self
            .clients
            .values()
            .any(|record| record.info.client_type == "window" && record.info.focused);
        let client_focused = self.clients.get(client_id).is_some_and(|record| record.info.focused)
            || (!has_focused_window && client_frame_type == "top-level");
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
        self.message_ports.extend(
            ports
                .transferred_port_ids
                .iter()
                .map(|&port_id| (registration_id, client_id.to_string(), port_id)),
        );
        *self.pending_client_messages.entry(key.clone()).or_default() += 1;
        let clients_claim_allowed = state == ServiceWorkerState::Activated;
        self.sync_runtime_registration_peers(registration_id)?;
        let result = self.host.dispatch_client_message(
            registration_id,
            ServiceWorkerClientMessageDispatch {
                event_id,
                data_json,
                client_id,
                client_url,
                client_frame_type: &client_frame_type,
                client_focused,
                ports,
                clients_claim_allowed,
            },
        );
        if result.is_err() {
            self.release_client_message_reservation(&key);
            for port_id in &ports.transferred_port_ids {
                self.message_ports
                    .remove(&(registration_id, client_id.to_string(), *port_id));
            }
        }
        result
    }

    /// Queue a controlled fetch event on the active worker with the longest matching scope.
    pub fn dispatch_fetch(
        &mut self,
        origin: &str,
        event_id: u64,
        request: ServiceWorkerFetchRequest,
    ) -> Result<ServiceWorkerFetchDispatch, ServiceWorkerManagerError> {
        if origin.trim().is_empty() || origin.len() > MAX_URL_BYTES {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker fetch origin is invalid".into(),
            ));
        }
        if self.pending_fetch_events.len() >= MAX_PENDING_FETCH_EVENTS {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "too many pending Service Worker fetch events".into(),
            ));
        }
        let request_url = request.url.clone();
        let request_origin = url::Url::parse(&request_url)
            .map_err(|_| ServiceWorkerManagerError::InvalidInput("Service Worker fetch URL is invalid".into()))?
            .origin()
            .ascii_serialization();
        if request_origin != origin {
            return Ok(ServiceWorkerFetchDispatch::PassThrough);
        }
        let registration_id = if request.resulting_client_id.is_some() {
            self.active_registration_for_url(origin, &request_url)
                .map(|registration| registration.id)
        } else if let Some(client_id) = request.client_id.as_deref() {
            if self.clients.contains_key(client_id) {
                self.active_registration_for_client(origin, client_id)
                    .map(|registration| registration.id)
            } else {
                self.active_registration_for_url(origin, &request_url)
                    .map(|registration| registration.id)
            }
        } else {
            self.active_registration_for_url(origin, &request_url)
                .map(|registration| registration.id)
        };
        let Some(registration_id) = registration_id else {
            return Ok(ServiceWorkerFetchDispatch::PassThrough);
        };
        let client_id = request.client_id.clone();
        self.host.dispatch_fetch(registration_id, event_id, request, true)?;
        self.pending_fetch_events.insert(
            (registration_id, event_id),
            PendingFetchRecord {
                registration_id,
                request_url,
                client_id,
            },
        );
        Ok(ServiceWorkerFetchDispatch::Dispatched {
            registration_id,
            event_id,
        })
    }

    /// Forward one page-side fetch body cancel/abort for a dispatched fetch event.
    ///
    /// The worker-side stream `cancel()` callback becomes observable even though
    /// the response body was already serialized to the page (headless approximation;
    /// unknown registrations are ignored so late cancels after unregister stay quiet).
    pub fn cancel_fetch_body(&mut self, registration_id: u64, event_id: u64, reason: &str) {
        let Some(key) = self.registration_keys.get(&registration_id) else {
            return;
        };
        if !self.slots.contains_key(key) {
            return;
        }
        let _ = self.host.cancel_fetch_body(registration_id, event_id, reason);
    }

    /// Return worker messages for one browser-owned client after its cursor.
    pub fn client_messages_since(
        &self,
        registration_id: u64,
        client_id: &str,
        after_sequence: u64,
    ) -> (u64, Vec<ServiceWorkerOutboundMessage>) {
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

    fn record_outbound_message_ports(
        &mut self,
        registration_id: u64,
        client_id: &str,
        messages: &[ServiceWorkerOutboundMessage],
    ) -> Result<(), ServiceWorkerManagerError> {
        let registration_origin = self
            .registration(registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?
            .origin
            .clone();
        let mut transferred = HashSet::new();
        for message in messages {
            let target_client_id = message.target_client_id.as_deref().unwrap_or(client_id);
            let valid_target = self.clients.get(target_client_id).is_some_and(|record| {
                url::Url::parse(&record.info.url)
                    .is_ok_and(|url| url.origin().ascii_serialization() == registration_origin)
            });
            if target_client_id.is_empty() || !valid_target {
                return Err(ServiceWorkerManagerError::InvalidInput(
                    "outbound Service Worker target client does not exist".into(),
                ));
            }
            if message.transferred_port_ids.len() > 16
                || message.transferred_port_ids.contains(&0)
                || message
                    .data_port_index
                    .is_some_and(|index| index >= message.transferred_port_ids.len())
                || message.port_id == Some(0)
            {
                return Err(ServiceWorkerManagerError::InvalidInput(
                    "invalid outbound Service Worker MessagePort metadata".into(),
                ));
            }
            if let Some(port_id) = message.port_id
                && !self
                    .message_ports
                    .contains(&(registration_id, target_client_id.to_string(), port_id))
                && !transferred.contains(&(target_client_id.to_string(), port_id))
            {
                return Err(ServiceWorkerManagerError::InvalidInput(
                    "outbound Service Worker MessagePort endpoint does not exist".into(),
                ));
            }
            for &port_id in &message.transferred_port_ids {
                if self
                    .message_ports
                    .contains(&(registration_id, target_client_id.to_string(), port_id))
                    || !transferred.insert((target_client_id.to_string(), port_id))
                {
                    return Err(ServiceWorkerManagerError::InvalidInput(
                        "outbound Service Worker MessagePort endpoint was already transferred".into(),
                    ));
                }
            }
        }
        self.message_ports.extend(
            transferred
                .into_iter()
                .map(|(client_id, port_id)| (registration_id, client_id, port_id)),
        );
        Ok(())
    }

    fn record_fetch_outbound_message_ports(
        &mut self,
        registration_id: u64,
        default_client_id: Option<&str>,
        messages: &[ServiceWorkerOutboundMessage],
    ) -> Result<(), ServiceWorkerManagerError> {
        let mut targeted_messages = Vec::with_capacity(messages.len());
        for message in messages {
            let mut targeted = message.clone();
            if targeted.target_client_id.is_none()
                && let Some(port_id) = targeted.port_id
            {
                targeted.target_client_id = self.message_port_client_id(registration_id, port_id);
            }
            targeted_messages.push(targeted);
        }
        self.record_outbound_message_ports(registration_id, default_client_id.unwrap_or(""), &targeted_messages)
    }

    fn route_worker_port_messages(
        &mut self,
        registration_id: u64,
        messages: Vec<ServiceWorkerOutboundMessage>,
    ) -> (Vec<ServiceWorkerOutboundMessage>, Vec<ServiceWorkerManagerEvent>) {
        let mut client_messages = Vec::new();
        let mut failures = Vec::new();
        for message in messages {
            if let Some(port_id) = message.port_id
                && let Some(target_registration_id) =
                    self.worker_message_ports.get(&(registration_id, port_id)).copied()
                && let Some(source) = self.worker_peer_info(registration_id)
            {
                if let Err(error) = self.dispatch_worker_port_message(
                    registration_id,
                    ServiceWorkerWorkerMessage {
                        target_registration_id,
                        source,
                        data_json: message.data_json,
                        transferred_port_ids: message.transferred_port_ids,
                        data_port_index: message.data_port_index,
                        target_port_id: Some(port_id),
                    },
                ) {
                    failures.push(ServiceWorkerManagerEvent::CoordinationFailed {
                        registration_id,
                        message: error.to_string(),
                    });
                }
            } else {
                client_messages.push(message);
            }
        }
        (client_messages, failures)
    }

    fn complete_routed_client_messages(
        &mut self,
        registration_id: u64,
        source_client_id: Option<&str>,
        messages: Vec<ServiceWorkerOutboundMessage>,
    ) {
        if let Some(client_id) = source_client_id {
            self.release_client_message_reservation(&(registration_id, client_id.to_string()));
        }
        self.route_client_messages(registration_id, source_client_id, messages);
    }

    fn route_client_messages(
        &mut self,
        registration_id: u64,
        default_client_id: Option<&str>,
        messages: Vec<ServiceWorkerOutboundMessage>,
    ) {
        let mut batches: Vec<(String, Vec<ServiceWorkerOutboundMessage>)> = Vec::new();
        for message in messages {
            let client_id = message
                .target_client_id
                .clone()
                .or_else(|| {
                    message
                        .port_id
                        .and_then(|port_id| self.message_port_client_id(registration_id, port_id))
                })
                .or_else(|| default_client_id.map(str::to_string));
            let Some(client_id) = client_id else {
                continue;
            };
            if let Some((_, batch)) = batches.iter_mut().find(|(id, _)| id == &client_id) {
                batch.push(message);
            } else {
                batches.push((client_id, vec![message]));
            }
        }
        for (client_id, batch) in batches {
            self.append_client_message_batch(registration_id, &client_id, batch);
        }
    }

    fn pending_fetch_client_id(&self, registration_id: u64) -> Option<String> {
        let mut client_id = None;
        for pending in self.pending_fetch_events.values() {
            if pending.registration_id != registration_id {
                continue;
            }
            let Some(current) = pending.client_id.as_ref() else {
                continue;
            };
            if client_id.as_ref().is_some_and(|existing| existing != current) {
                return None;
            }
            client_id = Some(current.clone());
        }
        client_id
    }

    fn message_port_client_id(&self, registration_id: u64, port_id: u64) -> Option<String> {
        self.message_ports.iter().find_map(|(id, client_id, current_port_id)| {
            (*id == registration_id && *current_port_id == port_id).then(|| client_id.clone())
        })
    }

    fn complete_client_message_batch(
        &mut self,
        registration_id: u64,
        client_id: &str,
        batch: Vec<ServiceWorkerOutboundMessage>,
    ) {
        let key = (registration_id, client_id.to_string());
        self.release_client_message_reservation(&key);
        self.append_client_message_batch(registration_id, client_id, batch);
    }

    fn append_client_message_batch(
        &mut self,
        registration_id: u64,
        client_id: &str,
        batch: Vec<ServiceWorkerOutboundMessage>,
    ) {
        let key = (registration_id, client_id.to_string());
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

    /// Record one committed top-level window as a Service Worker client.
    pub fn observe_window_client(
        &mut self,
        client_id: &str,
        client_url: &str,
    ) -> Result<(), ServiceWorkerManagerError> {
        self.observe_window_client_with_frame_type(client_id, client_url, "top-level")
    }

    /// Record one committed window as a Service Worker client.
    pub fn observe_window_client_with_frame_type(
        &mut self,
        client_id: &str,
        client_url: &str,
        frame_type: &str,
    ) -> Result<(), ServiceWorkerManagerError> {
        if client_id.is_empty() || client_id.len() > MAX_URL_BYTES || client_url.len() > MAX_URL_BYTES {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker client fields are invalid".into(),
            ));
        }
        // https://w3c.github.io/ServiceWorker/#client-frametype
        if !is_service_worker_window_frame_type(frame_type) {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker window client frame type is invalid".into(),
            ));
        }
        let url = url::Url::parse(client_url)
            .map_err(|_| ServiceWorkerManagerError::InvalidInput("Service Worker client URL is invalid".into()))?;
        if !matches!(url.scheme(), "http" | "https") || !url.username().is_empty() || url.password().is_some() {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker client URL is not eligible".into(),
            ));
        }
        if !self.clients.contains_key(client_id) && self.clients.len() >= MAX_SERVICE_WORKER_CLIENTS {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker client limit exceeded".into(),
            ));
        }
        let existing = self.clients.get(client_id);
        let sequence = existing.map(|record| record.creation_sequence).unwrap_or_else(|| {
            let sequence = self.next_client_sequence;
            self.next_client_sequence = self.next_client_sequence.saturating_add(1);
            sequence
        });
        let focused = existing.is_some_and(|record| record.info.focused);
        let last_focus_sequence = existing.and_then(|record| record.last_focus_sequence);
        let controller_registration_id = existing
            .map(|record| record.controller_registration_id)
            .unwrap_or_else(|| self.active_registration_id_for_url(&url.origin().ascii_serialization(), url.as_str()));
        self.clients.insert(
            client_id.to_string(),
            ClientRecord {
                info: ServiceWorkerClientInfo {
                    id: client_id.to_string(),
                    url: url.to_string(),
                    client_type: "window".into(),
                    frame_type: frame_type.to_string(),
                    visibility_state: "visible".into(),
                    focused,
                },
                controller_registration_id,
                creation_sequence: sequence,
                last_focus_sequence,
            },
        );
        Ok(())
    }

    /// Update the current focus state for one known window client.
    pub fn set_window_client_focused(
        &mut self,
        client_id: &str,
        focused: bool,
    ) -> Result<(), ServiceWorkerManagerError> {
        if client_id.is_empty() || client_id.len() > MAX_URL_BYTES {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker client id is invalid".into(),
            ));
        }
        if focused {
            if !self
                .clients
                .get(client_id)
                .is_some_and(|record| record.info.client_type == "window")
            {
                return Ok(());
            }
            let focus_sequence = self.next_client_focus_sequence;
            self.next_client_focus_sequence = self.next_client_focus_sequence.saturating_add(1);
            for record in self.clients.values_mut() {
                if record.info.client_type == "window" {
                    let is_target = record.info.id == client_id;
                    record.info.focused = is_target;
                    if is_target {
                        record.last_focus_sequence = Some(focus_sequence);
                    }
                }
            }
        } else if let Some(record) = self.clients.get_mut(client_id)
            && record.info.client_type == "window"
        {
            record.info.focused = focused;
        }
        Ok(())
    }

    /// Clear the current focus state without forgetting historical focus order.
    pub fn clear_window_client_focus(&mut self) {
        for record in self.clients.values_mut() {
            if record.info.client_type == "window" {
                record.info.focused = false;
            }
        }
    }

    /// Remove a Document client after navigation replacement or disconnect.
    pub fn remove_client(&mut self, client_id: &str) {
        self.clients.remove(client_id);
        self.client_messages.retain(|(_, id), _| id != client_id);
        self.pending_client_messages.retain(|(_, id), _| id != client_id);
        self.message_ports.retain(|(_, id, _)| id != client_id);
        self.collect_uncontrolled_unregistered_workers();
    }

    fn clients_for_worker(
        &self,
        registration_id: u64,
        include_uncontrolled: bool,
        client_type: &str,
    ) -> Result<Vec<ServiceWorkerClientInfo>, ServiceWorkerManagerError> {
        if !matches!(client_type, "window" | "worker" | "sharedworker" | "all") {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker client type is invalid".into(),
            ));
        }
        let registration = self
            .registration(registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        let mut clients = self
            .clients
            .values()
            .filter(|record| {
                let client = &record.info;
                (client_type == "all" || client.client_type == client_type)
                    && url::Url::parse(&client.url)
                        .is_ok_and(|url| url.origin().ascii_serialization() == registration.origin)
                    && (include_uncontrolled || record.controller_registration_id == Some(registration_id))
            })
            .cloned()
            .collect::<Vec<_>>();
        // https://w3c.github.io/ServiceWorker/#clients-matchall
        clients.sort_by(|left, right| {
            let left_window = left.info.client_type == "window";
            let right_window = right.info.client_type == "window";
            match (left_window, right_window) {
                (true, true) => match (left.last_focus_sequence, right.last_focus_sequence) {
                    (Some(left_focus), Some(right_focus)) => right_focus
                        .cmp(&left_focus)
                        .then_with(|| left.creation_sequence.cmp(&right.creation_sequence)),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => left.creation_sequence.cmp(&right.creation_sequence),
                },
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                (false, false) => left.creation_sequence.cmp(&right.creation_sequence),
            }
        });
        Ok(clients.into_iter().map(|record| record.info).collect())
    }

    fn client_for_worker(
        &self,
        registration_id: u64,
        client_id: &str,
    ) -> Result<Option<ServiceWorkerClientInfo>, ServiceWorkerManagerError> {
        // https://w3c.github.io/ServiceWorker/#clients-get
        if client_id.is_empty() || client_id.len() > MAX_URL_BYTES {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker client id is invalid".into(),
            ));
        }
        let registration = self
            .registration(registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        let Some(record) = self.clients.get(client_id) else {
            return Ok(None);
        };
        let same_origin = url::Url::parse(&record.info.url)
            .is_ok_and(|url| url.origin().ascii_serialization() == registration.origin);
        if same_origin {
            Ok(Some(record.info.clone()))
        } else {
            Ok(None)
        }
    }

    fn cache_storage_for_worker(
        &mut self,
        registration_id: u64,
        request: ServiceWorkerCacheStorageRequest,
    ) -> Result<ServiceWorkerCacheStorageResult, ServiceWorkerManagerError> {
        // https://w3c.github.io/ServiceWorker/#cache-storage-interface
        let registration = self
            .registry
            .get_mut(registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        match request {
            ServiceWorkerCacheStorageRequest::Open { cache_name } => {
                registration.cache_storage.open(&cache_name);
                let cache_id = self
                    .cache_storage_state
                    .ensure_active_cache_id(registration_id, &cache_name);
                Ok(ServiceWorkerCacheStorageResult::Open {
                    cache_name: display_cache_name(&cache_name),
                    cache_name_units: encode_cache_name_units(&cache_name),
                    cache_id,
                })
            }
            ServiceWorkerCacheStorageRequest::Match {
                cache_name,
                cache_id,
                request,
                options,
            } => {
                let request = cache_request_from_service_worker(request);
                let options = cache_query_options_from_service_worker(options);
                let response = match cache_name {
                    Some(cache_name) => self
                        .cache_storage_state
                        .selected_cache_ref(registration, &cache_name, cache_id)
                        .and_then(|cache| cache.match_request_with_options(&request, options)),
                    None => registration.cache_storage.match_request_with_options(&request, options),
                }
                .map(service_worker_response_from_cache)
                .transpose()?;
                Ok(ServiceWorkerCacheStorageResult::Match(response))
            }
            ServiceWorkerCacheStorageRequest::MatchAll {
                cache_name,
                cache_id,
                request,
                options,
            } => {
                let request = request.map(cache_request_from_service_worker);
                let options = cache_query_options_from_service_worker(options);
                let responses = self
                    .cache_storage_state
                    .selected_cache_ref(registration, &cache_name, cache_id)
                    .map(|cache| match &request {
                        Some(request) => cache.match_all_with_options(request, options),
                        None => cache
                            .request_keys()
                            .into_iter()
                            .filter_map(|request| cache.match_request(request))
                            .collect(),
                    })
                    .unwrap_or_default()
                    .into_iter()
                    .map(service_worker_response_from_cache)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ServiceWorkerCacheStorageResult::MatchAll(responses))
            }
            ServiceWorkerCacheStorageRequest::Keys {
                cache_name,
                cache_id,
                request,
                options,
            } => {
                let request = request.map(cache_request_from_service_worker);
                let options = cache_query_options_from_service_worker(options);
                let requests = match self
                    .cache_storage_state
                    .selected_cache_ref(registration, &cache_name, cache_id)
                {
                    Some(cache) => match &request {
                        Some(request) => cache.request_keys_with_options(request, options),
                        None => cache.request_keys(),
                    }
                    .into_iter()
                    .map(service_worker_request_from_cache)
                    .collect(),
                    None => Vec::new(),
                };
                Ok(ServiceWorkerCacheStorageResult::Keys(requests))
            }
            ServiceWorkerCacheStorageRequest::Delete {
                cache_name,
                cache_id,
                request,
                options,
            } => {
                let request = cache_request_from_service_worker(request);
                let options = cache_query_options_from_service_worker(options);
                let deleted = self
                    .cache_storage_state
                    .selected_cache_mut(registration, &cache_name, cache_id)
                    .map(|cache| cache.delete_with_options(&request, options))
                    .unwrap_or(false);
                Ok(ServiceWorkerCacheStorageResult::Bool(deleted))
            }
            ServiceWorkerCacheStorageRequest::Put {
                cache_name,
                cache_id,
                request,
                response,
            } => {
                let request = cache_request_from_service_worker(request);
                let response = cache_response_from_service_worker(response);
                if let Some(cache_id) = cache_id {
                    let Some(cache) =
                        self.cache_storage_state
                            .selected_cache_mut(registration, &cache_name, Some(cache_id))
                    else {
                        return Err(ServiceWorkerManagerError::Runtime(
                            "InvalidStateError: Cache backing store is no longer available".into(),
                        ));
                    };
                    cache
                        .put(request, response)
                        .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))?;
                } else {
                    registration
                        .cache_storage
                        .open(&cache_name)
                        .put(request, response)
                        .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))?;
                }
                Ok(ServiceWorkerCacheStorageResult::Done)
            }
            ServiceWorkerCacheStorageRequest::StorageHas { cache_name } => Ok(ServiceWorkerCacheStorageResult::Bool(
                registration.cache_storage.has(&cache_name),
            )),
            ServiceWorkerCacheStorageRequest::StorageDelete { cache_name } => {
                let doomed = registration.cache_storage.get(&cache_name).cloned();
                let deleted = registration.cache_storage.delete(&cache_name);
                let removed = if deleted {
                    self.cache_storage_state
                        .remove_active_cache_id(registration_id, &cache_name)
                        .zip(doomed)
                } else {
                    None
                };
                if let Some((cache_id, cache)) = removed {
                    self.cache_storage_state
                        .doomed_caches
                        .insert(cache_instance_key(registration_id, cache_id), cache);
                }
                Ok(ServiceWorkerCacheStorageResult::Bool(deleted))
            }
            ServiceWorkerCacheStorageRequest::StorageKeys => Ok(ServiceWorkerCacheStorageResult::StorageKeys(
                registration
                    .cache_storage
                    .keys()
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
            )),
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

    fn worker_peer_info(&self, registration_id: u64) -> Option<ServiceWorkerPeerInfo> {
        let registration = self.registry.get(registration_id)?;
        Some(ServiceWorkerPeerInfo {
            id: registration.id,
            script_url: registration.script_url.clone(),
            state: registration.state.to_string(),
        })
    }

    fn registration_peers(
        &self,
        registration_id: u64,
    ) -> Result<ServiceWorkerRegistrationPeers, ServiceWorkerManagerError> {
        let key = self.key_for(registration_id)?;
        self.registration_peers_for_key(key)
    }

    fn registration_peers_for_key(
        &self,
        key: &ServiceWorkerRegistrationKey,
    ) -> Result<ServiceWorkerRegistrationPeers, ServiceWorkerManagerError> {
        let slots = self.slots.get(key).copied().unwrap_or_default();
        Ok(ServiceWorkerRegistrationPeers {
            installing: slots.installing.and_then(|id| self.worker_peer_info(id)),
            waiting: slots.waiting.and_then(|id| self.worker_peer_info(id)),
            active: slots.active.and_then(|id| self.worker_peer_info(id)),
        })
    }

    fn sync_runtime_registration_peers(&mut self, registration_id: u64) -> Result<(), ServiceWorkerManagerError> {
        let peers = self.registration_peers(registration_id)?;
        self.host.sync_registration_peers(registration_id, peers)
    }

    fn dispatch_worker_message(
        &mut self,
        source_registration_id: u64,
        message: ServiceWorkerWorkerMessage,
    ) -> Result<(), ServiceWorkerManagerError> {
        let target_registration_id = message.target_registration_id;
        if message.target_port_id.is_some() {
            return self.dispatch_worker_port_message(source_registration_id, message);
        }
        let mut client_port_transfers = Vec::new();
        let mut worker_port_transfers = Vec::new();
        for &port_id in &message.transferred_port_ids {
            if let Some(client_id) = self.message_port_client_id(source_registration_id, port_id) {
                client_port_transfers.push((client_id, port_id));
            } else {
                worker_port_transfers.push(port_id);
            }
        }
        let target_state = self
            .registration(target_registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(target_registration_id))?
            .state;
        self.sync_runtime_registration_peers(target_registration_id)?;
        let event_id = self.next_worker_message_event_id;
        self.next_worker_message_event_id = self.next_worker_message_event_id.saturating_add(1);
        self.host.dispatch_worker_message(
            target_registration_id,
            event_id,
            message,
            target_state == ServiceWorkerState::Activated,
        )?;
        self.message_ports.extend(
            client_port_transfers
                .into_iter()
                .map(|(client_id, port_id)| (target_registration_id, client_id, port_id)),
        );
        self.worker_message_ports.extend(
            worker_port_transfers
                .into_iter()
                .map(|port_id| ((target_registration_id, port_id), source_registration_id)),
        );
        if let Ok(()) = self.sync_runtime_registration_peers(source_registration_id) {}
        Ok(())
    }

    fn dispatch_worker_port_message(
        &mut self,
        source_registration_id: u64,
        message: ServiceWorkerWorkerMessage,
    ) -> Result<(), ServiceWorkerManagerError> {
        let Some(port_id) = message.target_port_id else {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "worker MessagePort target is missing".into(),
            ));
        };
        let Some(target_registration_id) = self
            .worker_message_ports
            .get(&(source_registration_id, port_id))
            .copied()
        else {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "worker MessagePort endpoint does not exist".into(),
            ));
        };
        let target_state = self
            .registration(target_registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(target_registration_id))?
            .state;
        self.sync_runtime_registration_peers(target_registration_id)?;
        let event_id = self.next_worker_message_event_id;
        self.next_worker_message_event_id = self.next_worker_message_event_id.saturating_add(1);
        self.host.dispatch_worker_message(
            target_registration_id,
            event_id,
            message,
            target_state == ServiceWorkerState::Activated,
        )?;
        if let Ok(()) = self.sync_runtime_registration_peers(source_registration_id) {}
        Ok(())
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

    /// Find the active registration controlling a known window client.
    pub fn active_registration_for_client(&self, origin: &str, client_id: &str) -> Option<&ServiceWorkerRegistration> {
        let client = self.clients.get(client_id)?;
        let client_origin = url::Url::parse(&client.info.url).ok()?.origin().ascii_serialization();
        if client_origin != origin {
            return None;
        }
        let registration_id = client.controller_registration_id?;
        let registration = self.registry.get(registration_id)?;
        (registration.state == ServiceWorkerState::Activated
            && registration.origin == origin
            && registration.is_in_scope(&client.info.url))
        .then_some(registration)
    }

    /// Find the registration currently exposed as one known window client's
    /// controller. During activation, clients that were already controlled by
    /// the replaced active worker observe the replacement as `activating`.
    pub fn controller_registration_for_client(
        &self,
        origin: &str,
        client_id: &str,
    ) -> Option<&ServiceWorkerRegistration> {
        let client = self.clients.get(client_id)?;
        let client_origin = url::Url::parse(&client.info.url).ok()?.origin().ascii_serialization();
        if client_origin != origin {
            return None;
        }
        let registration_id = client.controller_registration_id?;
        let registration = self.registry.get(registration_id)?;
        (matches!(
            registration.state,
            ServiceWorkerState::Activating | ServiceWorkerState::Activated
        ) && registration.origin == origin
            && registration.is_in_scope(&client.info.url))
        .then_some(registration)
    }

    /// Find the registration currently exposed as one known window client's
    /// controller when that controller also matches `url`.
    pub fn controller_registration_for_client_url(
        &self,
        origin: &str,
        client_id: &str,
        url: &str,
    ) -> Option<&ServiceWorkerRegistration> {
        let registration = self.controller_registration_for_client(origin, client_id)?;
        registration.is_in_scope(url).then_some(registration)
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
                    cache_storage: registration.cache_storage.snapshot(),
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

    fn worker_update_target(&self, registration_id: u64) -> Result<Option<u64>, ServiceWorkerManagerError> {
        let registration = self
            .registration(registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        if registration.state == ServiceWorkerState::Installing {
            return Err(ServiceWorkerManagerError::InvalidState {
                registration_id,
                expected: ServiceWorkerState::Activated,
                actual: registration.state,
            });
        }
        let key = self.key_for(registration_id)?;
        let slot = self
            .slots
            .get(key)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        if slot
            .installing
            .is_some_and(|candidate_id| candidate_id != registration_id)
        {
            return Ok(None);
        }
        slot.waiting
            .or(slot.active)
            .map(Some)
            .ok_or(ServiceWorkerManagerError::InvalidState {
                registration_id,
                expected: ServiceWorkerState::Activated,
                actual: registration.state,
            })
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
        self.mark_redundant_without_stopping(registration_id);
        self.host.shutdown(registration_id);
    }

    fn mark_redundant_after_worker_unregister(&mut self, registration_id: u64) {
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
        self.unregistered_active.remove(&registration_id);
        self.clear_controller_for_registration(registration_id);
        self.claimed_clients.remove(&registration_id);
        self.restoring_active.remove(&registration_id);
        self.pending_import_requests.retain(|(id, _), _| *id != registration_id);
        self.update_predecessors.remove(&registration_id);
        self.pending_worker_updates
            .retain(|candidate_id, (caller_id, _)| *candidate_id != registration_id && *caller_id != registration_id);
        self.pending_fetch_events
            .retain(|_, pending| pending.registration_id != registration_id);
        self.script_sources.remove(&registration_id);
        self.imported_scripts.remove(&registration_id);
    }

    fn mark_redundant_without_stopping(&mut self, registration_id: u64) {
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
        self.unregistered_active.remove(&registration_id);
        self.clear_controller_for_registration(registration_id);
        self.claimed_clients.remove(&registration_id);
        self.restoring_active.remove(&registration_id);
        self.client_messages.retain(|(id, _), _| *id != registration_id);
        self.pending_client_messages.retain(|(id, _), _| *id != registration_id);
        self.pending_import_requests.retain(|(id, _), _| *id != registration_id);
        self.update_predecessors.remove(&registration_id);
        self.pending_worker_updates
            .retain(|candidate_id, (caller_id, _)| *candidate_id != registration_id && *caller_id != registration_id);
        self.pending_fetch_events
            .retain(|_, pending| pending.registration_id != registration_id);
        self.script_sources.remove(&registration_id);
        self.imported_scripts.remove(&registration_id);
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
            assert!(
                Instant::now() < deadline,
                "manager state timed out while waiting for {expected:?}; events: {events:?}"
            );
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

    fn wait_for_fetch(manager: &mut ServiceWorkerManager, event_id: u64) -> ServiceWorkerManagerEvent {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            for event in manager.poll() {
                if let ServiceWorkerManagerEvent::FetchSettled {
                    event_id: returned_id, ..
                } = &event
                    && *returned_id == event_id
                {
                    return event;
                }
            }
            assert!(Instant::now() < deadline, "manager fetch event timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn repeated_registration_compares_script_type_and_graph() {
        let mut manager = manager_under_test();
        let script = "globalThis.version = 1;";
        let first = start_active(&mut manager, "/app/", script);
        assert_eq!(
            manager
                .matching_registration(
                    "https://example.test/sw.js",
                    "/app/",
                    "https://example.test",
                    ServiceWorkerScriptType::Classic,
                    ServiceWorkerUpdateViaCache::Imports,
                )
                .unwrap(),
            Some(first)
        );
        assert_eq!(
            manager
                .matching_registration(
                    "https://example.test/sw.js",
                    "/app/",
                    "https://example.test",
                    ServiceWorkerScriptType::Classic,
                    ServiceWorkerUpdateViaCache::None,
                )
                .unwrap(),
            None
        );

        let unchanged = manager
            .start_registration(
                "https://example.test/sw.js",
                "/app/",
                "https://example.test",
                script,
                ServiceWorkerScriptType::Classic,
                ServiceWorkerUpdateViaCache::None,
            )
            .unwrap();
        assert_eq!(wait_for_update_check(&mut manager, unchanged), (first, false));
        assert_eq!(
            manager.registration(first).unwrap().update_via_cache,
            ServiceWorkerUpdateViaCache::None
        );

        let changed = manager
            .start_registration(
                "https://example.test/sw.js",
                "/app/",
                "https://example.test",
                script,
                ServiceWorkerScriptType::Module,
                ServiceWorkerUpdateViaCache::None,
            )
            .unwrap();
        assert_eq!(wait_for_update_check(&mut manager, changed), (changed, true));
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
    fn concurrent_updates_share_the_installing_candidate() {
        let mut manager = manager_under_test();
        let active = start_active(&mut manager, "/", "globalThis.version = 1;");
        assert_eq!(manager.coalesced_update_candidate(active), Ok(None));

        let ServiceWorkerUpdateOutcome::Started {
            registration_id: candidate,
        } = manager.start_update(active, "globalThis.version = 2;").unwrap()
        else {
            panic!("changed update must start a replacement");
        };
        assert_eq!(manager.coalesced_update_candidate(active), Ok(Some((candidate, true))));
        assert_eq!(
            manager.coalesced_update_candidate(candidate),
            Ok(Some((candidate, true)))
        );

        assert_eq!(wait_for_update_check(&mut manager, candidate), (candidate, true));
        wait_for_state(&mut manager, candidate, ServiceWorkerState::Installed);
        assert_eq!(manager.coalesced_update_candidate(candidate), Ok(None));
    }

    #[test]
    fn client_update_reuses_initial_installing_candidate_without_reporting_change() {
        let mut manager = manager_under_test();
        let installing = start(&mut manager, "/", "new Promise(() => {});");

        assert_eq!(
            manager.coalesced_update_candidate(installing),
            Ok(Some((installing, false)))
        );
        assert_eq!(manager.runtime_count(), 1);
    }

    #[test]
    fn installing_worker_update_rejects_with_invalid_state() {
        let mut manager = manager_under_test();
        let installing = start(
            &mut manager,
            "/",
            "let finishInstall;
             addEventListener('install', event => {
               event.waitUntil(new Promise(resolve => { finishInstall = resolve; }));
             });
             addEventListener('message', event => {
               registration.update().then(
                 () => event.source.postMessage({success: true}),
                 error => event.source.postMessage({success: false, exception: error.name})
               );
             });",
        );
        while !manager.evaluated.contains(&installing) {
            let _ = manager.poll();
        }

        manager
            .post_message(installing, 80, "null", "client-1", "https://example.test/page")
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let _ = manager.poll();
            let (_, messages) = manager.client_messages_since(installing, "client-1", 0);
            if !messages.is_empty() {
                assert_eq!(
                    messages,
                    [ServiceWorkerOutboundMessage {
                        data_json: r#"{"success":false,"exception":"InvalidStateError"}"#.into(),
                        port_id: None,
                        transferred_port_ids: Vec::new(),
                        data_port_index: None,
                        target_client_id: Some("client-1".into()),
                    }]
                );
                break;
            }
            assert!(Instant::now() < deadline, "worker update rejection timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn active_worker_update_reuses_installing_replacement() {
        let mut manager = manager_under_test();
        let active = start_active(
            &mut manager,
            "/",
            "addEventListener('message', event => {
               registration.update().then(
                 () => event.source.postMessage({success: true}),
                 error => event.source.postMessage({success: false, exception: error.name})
               );
             });",
        );
        let ServiceWorkerUpdateOutcome::Started {
            registration_id: candidate,
        } = manager
            .start_update(
                active,
                "addEventListener('install', event => event.waitUntil(new Promise(() => {})));",
            )
            .unwrap()
        else {
            panic!("changed update must start a candidate");
        };
        assert_eq!(wait_for_update_check(&mut manager, candidate), (candidate, true));

        manager
            .post_message(active, 81, "null", "client-1", "https://example.test/page")
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let _ = manager.poll();
            let (_, messages) = manager.client_messages_since(active, "client-1", 0);
            if !messages.is_empty() {
                assert_eq!(messages[0].data_json, r#"{"success":true}"#);
                break;
            }
            assert!(Instant::now() < deadline, "worker update coalescing timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn active_worker_update_requests_and_completes_host_fetch() {
        let mut manager = manager_under_test();
        let active = start_active(
            &mut manager,
            "/",
            "addEventListener('message', event => {
               registration.update().then(
                 () => event.source.postMessage({success: true}),
                 error => event.source.postMessage({success: false, exception: error.name})
               );
             });",
        );
        manager
            .post_message(active, 82, "null", "client-1", "https://example.test/page")
            .unwrap();

        let deadline = Instant::now() + Duration::from_secs(5);
        let request_id = loop {
            if let Some(request_id) = manager.poll().into_iter().find_map(|event| match event {
                ServiceWorkerManagerEvent::WorkerUpdateRequested {
                    caller_registration_id,
                    request_id,
                    target_registration_id,
                } if caller_registration_id == active && target_registration_id == active => Some(request_id),
                _ => None,
            }) {
                break request_id;
            }
            assert!(Instant::now() < deadline, "worker update fetch request timed out");
            std::thread::sleep(Duration::from_millis(1));
        };
        manager
            .complete_worker_update_fetch(active, request_id, Ok("globalThis.version = 2;".into()))
            .unwrap();

        loop {
            let _ = manager.poll();
            let (_, messages) = manager.client_messages_since(active, "client-1", 0);
            if !messages.is_empty() {
                assert_eq!(messages[0].data_json, r#"{"success":true}"#);
                break;
            }
            assert!(Instant::now() < deadline, "worker update completion timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn active_worker_unregister_removes_future_matching_and_reports_result() {
        let mut manager = manager_under_test();
        let active = start_active(
            &mut manager,
            "/app/",
            "addEventListener('message', event => {
               registration.unregister().then(result => {
                 event.source.postMessage({removed: result});
               });
             });",
        );
        manager
            .observe_window_client("client-1", "https://example.test/app/page")
            .unwrap();
        assert_eq!(
            manager
                .active_registration_for_client("https://example.test", "client-1")
                .map(|registration| registration.id),
            Some(active)
        );
        manager
            .post_message(active, 83, "null", "client-1", "https://example.test/app/page")
            .unwrap();

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut unregister_reported = false;
        loop {
            for event in manager.poll() {
                if let ServiceWorkerManagerEvent::WorkerUnregistered {
                    registration_id,
                    removed,
                    ..
                } = event
                    && registration_id == active
                {
                    assert!(removed);
                    unregister_reported = true;
                    assert!(
                        manager
                            .registration_for_url("https://example.test", "https://example.test/app/next")
                            .is_none(),
                        "unregistered registration must not match later navigations"
                    );
                }
            }
            let (_, messages) = manager.client_messages_since(active, "client-1", 0);
            if unregister_reported && !messages.is_empty() {
                assert_eq!(messages[0].data_json, r#"{"removed":true}"#);
                assert_eq!(
                    manager
                        .active_registration_for_client("https://example.test", "client-1")
                        .map(|registration| registration.id),
                    Some(active),
                    "existing clients keep their controller after unregister"
                );
                manager
                    .observe_window_client("client-2", "https://example.test/app/next")
                    .unwrap();
                assert!(
                    manager
                        .active_registration_for_client("https://example.test", "client-2")
                        .is_none(),
                    "new clients must not be controlled by an unregistered registration"
                );
                return;
            }
            assert!(Instant::now() < deadline, "worker unregister completion timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn installing_worker_unregister_becomes_redundant_and_removes_future_matching() {
        let mut manager = manager_under_test();
        let id = start(
            &mut manager,
            "/app/",
            "registration.unregister();
             addEventListener('install', () => {});",
        );
        wait_for_state(&mut manager, id, ServiceWorkerState::Redundant);

        assert!(
            manager
                .registration_for_url("https://example.test", "https://example.test/app/next")
                .is_none()
        );
        assert!(manager.slots(&key("/app/")).is_none());
    }

    #[test]
    fn activating_worker_unregister_becomes_redundant_and_removes_future_matching() {
        let mut manager = manager_under_test();
        let id = start(
            &mut manager,
            "/app/",
            "addEventListener('activate', event => {
               event.waitUntil(registration.unregister());
             });",
        );
        wait_for_state(&mut manager, id, ServiceWorkerState::Redundant);

        assert!(
            manager
                .registration_for_url("https://example.test", "https://example.test/app/next")
                .is_none()
        );
        assert!(manager.slots(&key("/app/")).is_none());
    }

    #[test]
    fn evaluation_match_all_lists_same_origin_uncontrolled_client() {
        let mut manager = manager_under_test();
        manager
            .observe_window_client("client-1", "https://example.test/page")
            .unwrap();
        manager
            .observe_window_client("cross-origin", "https://other.test/page")
            .unwrap();
        let id = start(
            &mut manager,
            "/scope/",
            "clients.matchAll({includeUncontrolled: true}).then(clientList => {
               for (const client of clientList) {
                 client.postMessage({id: client.id, url: client.url});
               }
             });",
        );

        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let _ = manager.poll();
            let (_, messages) = manager.client_messages_since(id, "client-1", 0);
            if !messages.is_empty() {
                assert_eq!(messages.len(), 1);
                assert_eq!(
                    messages[0].data_json,
                    r#"{"id":"client-1","url":"https://example.test/page"}"#
                );
                assert_eq!(messages[0].target_client_id.as_deref(), Some("client-1"));
                assert_eq!(manager.client_messages_since(id, "cross-origin", 0).1, Vec::new());
                assert!(matches!(
                    manager.record_outbound_message_ports(
                        id,
                        "",
                        &[ServiceWorkerOutboundMessage {
                            data_json: "\"blocked\"".into(),
                            port_id: None,
                            transferred_port_ids: Vec::new(),
                            data_port_index: None,
                            target_client_id: Some("cross-origin".into()),
                        }],
                    ),
                    Err(ServiceWorkerManagerError::InvalidInput(_))
                ));
                break;
            }
            assert!(Instant::now() < deadline, "clients.matchAll message timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
        manager
            .observe_window_client("controlled", "https://example.test/scope/page")
            .unwrap();
        wait_for_state(&mut manager, id, ServiceWorkerState::Activated);
        assert!(
            manager.clients_for_worker(id, false, "window").unwrap().is_empty(),
            "an existing same-scope client is uncontrolled until clients.claim()"
        );
        manager.claim_matching_clients(id);
        assert_eq!(
            manager
                .clients_for_worker(id, false, "window")
                .unwrap()
                .into_iter()
                .map(|client| client.id)
                .collect::<Vec<_>>(),
            ["controlled"]
        );
        assert!(manager.clients_for_worker(id, true, "worker").unwrap().is_empty());
    }

    #[test]
    fn observed_window_clients_preserve_valid_frame_types() {
        let mut manager = manager_under_test();
        let id = start_active(&mut manager, "/", "globalThis.ready = true;");
        manager
            .observe_window_client("top", "https://example.test/top")
            .unwrap();
        manager
            .observe_window_client_with_frame_type("popup", "https://example.test/popup", "auxiliary")
            .unwrap();
        manager
            .observe_window_client_with_frame_type("frame", "https://example.test/frame", "nested")
            .unwrap();

        assert_eq!(
            manager
                .clients_for_worker(id, true, "window")
                .unwrap()
                .into_iter()
                .map(|client| (client.id, client.frame_type))
                .collect::<Vec<_>>(),
            [
                ("top".to_string(), "top-level".to_string()),
                ("popup".to_string(), "auxiliary".to_string()),
                ("frame".to_string(), "nested".to_string()),
            ]
        );
        assert_eq!(
            manager.client_for_worker(id, "frame").unwrap().unwrap().frame_type,
            "nested"
        );
    }

    #[test]
    fn observed_window_client_rejects_invalid_frame_type() {
        let mut manager = manager_under_test();

        assert_eq!(
            manager.observe_window_client_with_frame_type("client-1", "https://example.test/page", "none"),
            Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker window client frame type is invalid".into()
            ))
        );
        assert_eq!(
            manager.observe_window_client_with_frame_type("client-1", "https://example.test/page", "detached"),
            Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker window client frame type is invalid".into()
            ))
        );
    }

    #[test]
    fn evaluation_clients_get_returns_same_origin_client_only() {
        let mut manager = manager_under_test();
        manager
            .observe_window_client("client-1", "https://example.test/page")
            .unwrap();
        manager
            .observe_window_client("cross-origin", "https://other.test/page")
            .unwrap();
        let id = start(
            &mut manager,
            "/scope/",
            "Promise.all([
               clients.get('client-1'),
               clients.get('cross-origin'),
               clients.get('missing')
             ]).then(results => {
               results[0].postMessage({
                 hit: results[0].url,
                 hidden: results[1] === undefined,
                 missing: results[2] === undefined
               });
             });",
        );

        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let _ = manager.poll();
            let (_, messages) = manager.client_messages_since(id, "client-1", 0);
            if !messages.is_empty() {
                assert_eq!(messages.len(), 1);
                assert_eq!(
                    messages[0].data_json,
                    r#"{"hit":"https://example.test/page","hidden":true,"missing":true}"#
                );
                assert_eq!(messages[0].target_client_id.as_deref(), Some("client-1"));
                assert_eq!(manager.client_messages_since(id, "cross-origin", 0).1, Vec::new());
                break;
            }
            assert!(Instant::now() < deadline, "clients.get message timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn match_all_orders_focused_windows_before_creation_order() {
        let mut manager = manager_under_test();
        let id = start_active(&mut manager, "/", "globalThis.ready = true;");
        manager
            .observe_window_client("client-1", "https://example.test/app/1")
            .unwrap();
        manager
            .observe_window_client("client-2", "https://example.test/app/2")
            .unwrap();
        manager
            .observe_window_client("client-3", "https://example.test/app/3")
            .unwrap();

        assert_eq!(
            manager
                .clients_for_worker(id, true, "window")
                .unwrap()
                .into_iter()
                .map(|client| (client.id, client.focused))
                .collect::<Vec<_>>(),
            [
                ("client-1".to_string(), false),
                ("client-2".to_string(), false),
                ("client-3".to_string(), false),
            ]
        );

        manager.set_window_client_focused("client-1", true).unwrap();
        manager.set_window_client_focused("client-3", true).unwrap();
        manager.set_window_client_focused("client-2", true).unwrap();
        manager.set_window_client_focused("client-2", true).unwrap();
        assert_eq!(
            manager
                .clients_for_worker(id, true, "window")
                .unwrap()
                .into_iter()
                .map(|client| (client.id, client.focused))
                .collect::<Vec<_>>(),
            [
                ("client-2".to_string(), true),
                ("client-3".to_string(), false),
                ("client-1".to_string(), false),
            ]
        );

        manager.clear_window_client_focus();
        assert_eq!(
            manager
                .clients_for_worker(id, true, "window")
                .unwrap()
                .into_iter()
                .map(|client| (client.id, client.focused))
                .collect::<Vec<_>>(),
            [
                ("client-2".to_string(), false),
                ("client-3".to_string(), false),
                ("client-1".to_string(), false),
            ]
        );
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
    fn lifecycle_events_see_current_registration_slots() {
        let mut manager = manager_under_test();
        let id = start(
            &mut manager,
            "https://example.test/",
            "if (registration.installing !== null) throw new Error('initial installing');
             if (registration.waiting !== null) throw new Error('initial waiting');
             if (registration.active !== null) throw new Error('initial active');
             addEventListener('install', () => {
               if (registration.installing !== serviceWorker) throw new Error('install installing');
               if (registration.waiting !== null) throw new Error('install waiting');
               if (registration.active !== null) throw new Error('install active');
               if (registration.installing.state !== 'installing') throw new Error('install state');
             });
             addEventListener('activate', () => {
               if (registration.installing !== null) throw new Error('activate installing');
               if (registration.waiting !== null) throw new Error('activate waiting');
               if (registration.active !== serviceWorker) throw new Error('activate active');
               if (registration.active.state !== 'activating') throw new Error('activate state');
             });",
        );
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
                cache_storage: CacheStorageSnapshot::default(),
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
        let events = wait_for_state(&mut manager, id, ServiceWorkerState::Activated);
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
        assert!(events.contains(&ServiceWorkerManagerEvent::ActivationCompleted {
            registration_id: id,
            succeeded: true,
        }));
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
    fn activation_wait_until_rejection_still_replaces_existing_active() {
        let mut manager = manager_under_test();
        let first = start_active(&mut manager, "/", "globalThis.version = 1;");
        manager
            .observe_window_client("client-1", "https://example.test/app/page")
            .unwrap();

        let second = start(
            &mut manager,
            "/",
            "addEventListener('activate', event => {
                event.waitUntil(Promise.reject(new Error('activate rejected')));
            });",
        );
        wait_for_state(&mut manager, second, ServiceWorkerState::Installed);
        manager.activate_waiting(second).unwrap();
        wait_for_state(&mut manager, second, ServiceWorkerState::Activated);

        let slots = manager.slots(&key("/")).unwrap();
        assert_eq!(slots.active, Some(second));
        assert_eq!(slots.waiting, None);
        assert_eq!(
            manager
                .controller_registration_for_client("https://example.test", "client-1")
                .map(|registration| registration.id),
            Some(second),
            "activate waitUntil rejection should not restore the incumbent controller"
        );
        assert_eq!(
            manager.registration(first).unwrap().state,
            ServiceWorkerState::Redundant
        );
        assert_eq!(
            manager.registration(second).unwrap().state,
            ServiceWorkerState::Activated
        );
    }

    #[test]
    fn activating_replacement_is_visible_as_controlled_client_controller() {
        let mut manager = manager_under_test();
        let first = start_active(&mut manager, "/", "globalThis.version = 1;");
        manager
            .observe_window_client("client-1", "https://example.test/app/page")
            .unwrap();

        let second = start(&mut manager, "/", "globalThis.version = 2;");
        wait_for_state(&mut manager, second, ServiceWorkerState::Installed);
        manager.activate_waiting(second).unwrap();

        assert_eq!(
            manager.registration(second).unwrap().state,
            ServiceWorkerState::Activating
        );
        assert_eq!(
            manager
                .controller_registration_for_client("https://example.test", "client-1")
                .map(|registration| (registration.id, registration.state)),
            Some((second, ServiceWorkerState::Activating)),
            "controlled clients should observe the replacement controller during activation"
        );
        assert!(
            manager
                .active_registration_for_client("https://example.test", "client-1")
                .is_none(),
            "fetch dispatch should not treat an activating controller as active"
        );

        wait_for_state(&mut manager, second, ServiceWorkerState::Activated);
        assert_eq!(manager.slots(&key("/")).unwrap().active, Some(second));
        assert_eq!(
            manager.registration(first).unwrap().state,
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
    fn active_registration_for_client_requires_observed_control_or_claim() {
        let mut manager = manager_under_test();
        manager
            .observe_window_client("client-1", "https://example.test/app/page")
            .unwrap();
        let id = start_active(&mut manager, "/app/", "globalThis.ready = true;");

        assert!(
            manager
                .active_registration_for_client("https://example.test", "client-1")
                .is_none(),
            "an existing same-scope client stays uncontrolled until clients.claim()"
        );

        manager.claim_matching_clients(id);
        assert_eq!(
            manager
                .active_registration_for_client("https://example.test", "client-1")
                .map(|registration| registration.id),
            Some(id)
        );
    }

    #[test]
    fn repeated_observe_does_not_control_existing_uncontrolled_client() {
        let mut manager = manager_under_test();
        manager
            .observe_window_client("client-1", "https://example.test/app/page")
            .unwrap();
        let id = start_active(&mut manager, "/app/", "globalThis.ready = true;");
        manager
            .observe_window_client("client-1", "https://example.test/app/page")
            .unwrap();

        assert!(
            manager
                .active_registration_for_client("https://example.test", "client-1")
                .is_none(),
            "an existing same-document client must not become controlled just because it is observed again"
        );

        manager.claim_matching_clients(id);
        assert_eq!(
            manager
                .active_registration_for_client("https://example.test", "client-1")
                .map(|registration| registration.id),
            Some(id)
        );
    }

    #[test]
    fn newly_observed_matching_client_is_controlled_by_active_registration() {
        let mut manager = manager_under_test();
        let id = start_active(&mut manager, "/app/", "globalThis.ready = true;");
        manager
            .observe_window_client("client-1", "https://example.test/app/page")
            .unwrap();

        assert_eq!(
            manager
                .active_registration_for_client("https://example.test", "client-1")
                .map(|registration| registration.id),
            Some(id)
        );
    }

    #[test]
    fn clients_claim_does_not_control_client_with_longer_matching_registration() {
        let mut manager = manager_under_test();
        manager
            .observe_window_client("client-1", "https://example.test/app/deeper/page")
            .unwrap();
        let app = start_active(&mut manager, "/app/", "globalThis.app = true;");
        let deeper = start(&mut manager, "/app/deeper/", "globalThis.deeper = true;");

        assert_eq!(
            manager.registration(deeper).map(|registration| registration.state),
            Some(ServiceWorkerState::Installing)
        );

        manager.claim_matching_clients(app);
        assert!(
            manager
                .active_registration_for_client("https://example.test", "client-1")
                .is_none(),
            "clients.claim() must not control clients matched by a different longer registration"
        );
    }

    #[test]
    fn clients_claim_can_replace_shorter_controller_when_it_is_longest_match() {
        let mut manager = manager_under_test();
        let root = start_active(&mut manager, "/", "globalThis.root = true;");
        manager
            .observe_window_client("client-1", "https://example.test/app/page")
            .unwrap();
        assert_eq!(
            manager
                .active_registration_for_client("https://example.test", "client-1")
                .map(|registration| registration.id),
            Some(root)
        );

        let app = start_active(&mut manager, "/app/", "globalThis.app = true;");
        manager.claim_matching_clients(app);
        assert_eq!(
            manager
                .active_registration_for_client("https://example.test", "client-1")
                .map(|registration| registration.id),
            Some(app)
        );
    }

    #[test]
    fn clients_claim_can_replace_controller_for_query_scope() {
        let mut manager = manager_under_test();
        let root = start_active(&mut manager, "/app/", "globalThis.root = true;");
        manager
            .observe_window_client("client-1", "https://example.test/app/page.html?controlled")
            .unwrap();
        assert_eq!(
            manager
                .active_registration_for_client("https://example.test", "client-1")
                .map(|registration| registration.id),
            Some(root)
        );

        let query = start_active(&mut manager, "/app/page.html?controlled", "globalThis.query = true;");
        manager.claim_matching_clients(query);
        assert_eq!(
            manager
                .active_registration_for_client("https://example.test", "client-1")
                .map(|registration| registration.id),
            Some(query)
        );
    }

    #[test]
    fn clients_claim_can_replace_controller_for_full_url_query_scope() {
        let mut manager = manager_under_test();
        let root = start_active(&mut manager, "https://example.test/app/", "globalThis.root = true;");
        manager
            .observe_window_client("client-1", "https://example.test/app/page.html?controlled")
            .unwrap();
        assert_eq!(
            manager
                .active_registration_for_client("https://example.test", "client-1")
                .map(|registration| registration.id),
            Some(root)
        );

        let query = start_active(
            &mut manager,
            "https://example.test/app/page.html?controlled",
            "globalThis.query = true;",
        );
        manager.claim_matching_clients(query);
        assert_eq!(
            manager
                .active_registration_for_client("https://example.test", "client-1")
                .map(|registration| registration.id),
            Some(query)
        );
    }

    #[test]
    fn unregister_preserves_existing_controller_until_client_is_removed() {
        let mut manager = manager_under_test();
        let id = start_active(&mut manager, "/app/", "globalThis.ready = true;");
        manager
            .observe_window_client("client-1", "https://example.test/app/page")
            .unwrap();
        assert_eq!(
            manager
                .active_registration_for_client("https://example.test", "client-1")
                .map(|registration| registration.id),
            Some(id)
        );

        assert!(manager.unregister(id));
        assert!(
            manager
                .registration_for_url("https://example.test", "https://example.test/app/page")
                .is_none(),
            "unregistered registration must not match later navigations"
        );
        assert!(
            manager
                .active_registration_for_client("https://example.test", "client-1")
                .map(|registration| registration.id)
                == Some(id),
            "existing controlled clients keep their controller after unregister"
        );
        manager
            .observe_window_client("client-2", "https://example.test/app/next")
            .unwrap();
        assert!(
            manager
                .active_registration_for_client("https://example.test", "client-2")
                .is_none(),
            "new clients must not be controlled by an unregistered registration"
        );
        manager.remove_client("client-1");
        assert!(manager.registration(id).is_none());
        let _ = manager.poll();
        assert_eq!(manager.runtime_count(), 0);
    }

    #[test]
    fn unregister_collects_active_after_controlled_client_is_removed() {
        let mut manager = manager_under_test();
        let id = start_active(&mut manager, "/app/", "globalThis.ready = true;");
        manager
            .observe_window_client("client-1", "https://example.test/app/page")
            .unwrap();
        assert_eq!(manager.runtime_count(), 1);

        assert!(manager.unregister(id));
        assert_eq!(
            manager.registration(id).map(|registration| registration.state),
            Some(ServiceWorkerState::Activated),
            "controlled unregistered active worker is kept until the client is discarded"
        );
        assert_eq!(manager.runtime_count(), 1);

        manager.remove_client("client-1");
        let _ = manager.poll();

        assert!(manager.registration(id).is_none());
        assert_eq!(manager.runtime_count(), 0);
    }

    #[test]
    fn navigation_fetch_uses_resulting_client_scope_not_uncontrolled_source_client() {
        let mut manager = manager_under_test();
        manager
            .observe_window_client("client-1", "https://example.test/page")
            .unwrap();
        let id = start_active(
            &mut manager,
            "/app/",
            "addEventListener('fetch', event => {
               event.respondWith(new Response('nav:' + event.request.url));
             });",
        );

        assert_eq!(
            manager
                .dispatch_fetch(
                    "https://example.test",
                    118,
                    ServiceWorkerFetchRequest {
                        url: "https://example.test/app/frame.html".into(),
                        method: "GET".into(),
                        headers: Vec::new(),
                        body: None,
                        credentials: None,
                        client_id: Some("client-1".into()),
                        resulting_client_id: Some("client-1:nested:https://example.test/app/frame.html".into()),
                        referrer: Some("https://example.test/page".into()),
                        is_reload_navigation: false,
                        is_history_navigation: false,
                    },
                )
                .unwrap(),
            ServiceWorkerFetchDispatch::Dispatched {
                registration_id: id,
                event_id: 118,
            }
        );
        assert_eq!(
            wait_for_fetch(&mut manager, 118),
            ServiceWorkerManagerEvent::FetchSettled {
                registration_id: id,
                event_id: 118,
                request_url: "https://example.test/app/frame.html".into(),
                client_id: Some("client-1".into()),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "nav:https://example.test/app/frame.html".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );

        assert_eq!(
            manager
                .dispatch_fetch(
                    "https://example.test",
                    119,
                    ServiceWorkerFetchRequest {
                        url: "https://example.test/app/subresource.txt".into(),
                        method: "GET".into(),
                        headers: Vec::new(),
                        body: None,
                        credentials: None,
                        client_id: Some("client-1".into()),
                        resulting_client_id: None,
                        referrer: Some("https://example.test/page".into()),
                        is_reload_navigation: false,
                        is_history_navigation: false,
                    },
                )
                .unwrap(),
            ServiceWorkerFetchDispatch::PassThrough
        );
    }

    #[test]
    fn message_time_clients_claim_is_recorded_for_active_version() {
        let mut manager = manager_under_test();
        let id = start_active(
            &mut manager,
            "/app/",
            "addEventListener('message', event => {
               event.waitUntil(clients.claim());
               event.source.postMessage('claimed');
             });",
        );
        assert!(!manager.claims_clients(id));

        manager
            .post_message(id, 94, "null", "client-1", "https://example.test/app/page")
            .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        let mut saw_claim = false;
        let mut saw_message = false;
        while !saw_claim || !saw_message {
            for event in manager.poll() {
                match event {
                    ServiceWorkerManagerEvent::ClientsClaimed { registration_id } if registration_id == id => {
                        saw_claim = true;
                    }
                    ServiceWorkerManagerEvent::MessageDispatched {
                        registration_id,
                        event_id: 94,
                        outbound_count: 1,
                    } if registration_id == id => {
                        saw_message = true;
                    }
                    _ => {}
                }
            }
            assert!(std::time::Instant::now() < deadline, "message clients.claim timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(manager.claims_clients(id));
    }

    #[test]
    fn waiting_worker_rejected_clients_claim_can_reply_over_message_port() {
        let mut manager = manager_under_test();
        let active = start_active(&mut manager, "/app/", "globalThis.version = 1;");
        let ServiceWorkerUpdateOutcome::Started {
            registration_id: waiting,
        } = manager
            .start_update(
                active,
                "addEventListener('message', event => {
                   clients.claim().then(() => {
                     event.data.port.postMessage('PASS');
                   }, error => {
                     event.data.port.postMessage('FAIL: exception: ' + error.name);
                   });
                 });",
            )
            .unwrap()
        else {
            panic!("changed update must start a waiting candidate");
        };
        wait_for_state(&mut manager, waiting, ServiceWorkerState::Installed);

        manager
            .post_message_with_ports(
                waiting,
                95,
                r#"{"port":{"__zwServiceWorkerTransferredPortIndex":0}}"#,
                "client-1",
                "https://example.test/app/page",
                &ServiceWorkerMessagePorts {
                    transferred_port_ids: vec![2],
                    data_port_index: None,
                    target_port_id: None,
                },
            )
            .unwrap();

        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let _ = manager.poll();
            let (_, messages) = manager.client_messages_since(waiting, "client-1", 0);
            if !messages.is_empty() {
                assert_eq!(
                    messages,
                    [ServiceWorkerOutboundMessage {
                        data_json: "\"FAIL: exception: InvalidStateError\"".into(),
                        port_id: Some(2),
                        transferred_port_ids: Vec::new(),
                        data_port_index: None,
                        target_client_id: None,
                    }]
                );
                break;
            }
            assert!(
                Instant::now() < deadline,
                "waiting worker clients.claim rejection timed out"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(!manager.claims_clients(waiting));
    }

    #[test]
    fn worker_to_worker_message_can_reply_to_page_client() {
        let mut manager = manager_under_test();
        let active = start_active(
            &mut manager,
            "/app/",
            "addEventListener('message', event => {
               if (event.data.type === 'start') {
                 registration.waiting.postMessage({type: 'ping', client_id: event.source.id});
               } else if (event.data.type === 'pong') {
                 clients.get(event.data.client_id).then(client => {
                   client.postMessage({type: 'record', from: 'active', source: event.source.state});
                   client.postMessage({type: 'finish'});
                 });
               }
             });",
        );
        let ServiceWorkerUpdateOutcome::Started {
            registration_id: waiting,
        } = manager
            .start_update(
                active,
                "addEventListener('message', event => {
                   if (event.data.type === 'ping') {
                     clients.get(event.data.client_id).then(client => {
                       client.postMessage({type: 'record', from: 'waiting', source: event.source.state});
                       event.source.postMessage({type: 'pong', client_id: event.data.client_id});
                     });
                   }
                 });",
            )
            .unwrap()
        else {
            panic!("changed update must start a waiting candidate");
        };
        wait_for_state(&mut manager, waiting, ServiceWorkerState::Installed);

        manager
            .post_message(
                active,
                96,
                r#"{"type":"start"}"#,
                "client-1",
                "https://example.test/app/page",
            )
            .unwrap();

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut seen_events = Vec::new();
        loop {
            seen_events.extend(manager.poll());
            let (_, active_messages) = manager.client_messages_since(active, "client-1", 0);
            let (_, waiting_messages) = manager.client_messages_since(waiting, "client-1", 0);
            if active_messages.len() >= 2 && !waiting_messages.is_empty() {
                assert_eq!(
                    waiting_messages[0].data_json,
                    r#"{"type":"record","from":"waiting","source":"activated"}"#
                );
                assert_eq!(
                    active_messages[0].data_json,
                    r#"{"type":"record","from":"active","source":"installed"}"#
                );
                assert_eq!(active_messages[1].data_json, r#"{"type":"finish"}"#);
                break;
            }
            assert!(
                Instant::now() < deadline,
                "worker-to-worker page reply timed out: events={seen_events:?} active={active_messages:?} waiting={waiting_messages:?}"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn worker_to_worker_transferred_port_can_reply_to_page_client() {
        let mut manager = manager_under_test();
        let active = start_active(
            &mut manager,
            "/app/",
            "addEventListener('message', event => {
               if (event.data.port) {
                 const port = event.data.port;
                 const channel = new MessageChannel();
                 channel.port1.onmessage = event => {
                   if (event.data.pong) port.postMessage(event.data.pong);
                 };
                 registration.waiting.postMessage({ping: channel.port2}, [channel.port2]);
               }
             });",
        );
        let ServiceWorkerUpdateOutcome::Started {
            registration_id: waiting,
        } = manager
            .start_update(
                active,
                "addEventListener('message', event => {
                   if (event.data.ping) event.data.ping.postMessage({pong: 'OK'});
                 });",
            )
            .unwrap()
        else {
            panic!("changed update must start a waiting candidate");
        };
        wait_for_state(&mut manager, waiting, ServiceWorkerState::Installed);

        manager
            .post_message_with_ports(
                active,
                97,
                r#"{"port":{"__zwServiceWorkerTransferredPortIndex":0}}"#,
                "client-1",
                "https://example.test/app/page",
                &ServiceWorkerMessagePorts {
                    transferred_port_ids: vec![2],
                    data_port_index: None,
                    target_port_id: None,
                },
            )
            .unwrap();

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut seen_events = Vec::new();
        loop {
            seen_events.extend(manager.poll());
            let (_, messages) = manager.client_messages_since(active, "client-1", 0);
            if !messages.is_empty() {
                assert_eq!(
                    messages,
                    [ServiceWorkerOutboundMessage {
                        data_json: "\"OK\"".into(),
                        port_id: Some(2),
                        transferred_port_ids: Vec::new(),
                        data_port_index: None,
                        target_client_id: None,
                    }]
                );
                break;
            }
            assert!(
                Instant::now() < deadline,
                "worker MessagePort reply timed out: events={seen_events:?} active_messages={messages:?} worker_ports={:?}",
                manager.worker_message_ports
            );
            std::thread::sleep(Duration::from_millis(1));
        }
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
            manager.client_messages_since(id, "client-1", 0).1[0].data_json,
            r#"{"echo":"hello"}"#
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
    fn fetch_dispatch_uses_active_longest_scope_worker() {
        let mut manager = manager_under_test();
        let root = start_active(
            &mut manager,
            "/",
            "addEventListener('fetch', event => {
               event.respondWith(new Response('root', {status: 201}));
             });",
        );
        let app = start_active(
            &mut manager,
            "/app/",
            "addEventListener('fetch', event => {
               event.respondWith(new Response('app:' + event.request.url, {
                 status: 203,
                 statusText: 'Scoped',
                 headers: {'X-Scope': 'app'}
               }));
             });",
        );

        assert_eq!(
            manager
                .dispatch_fetch(
                    "https://example.test",
                    120,
                    ServiceWorkerFetchRequest {
                        url: "https://example.test/app/data".into(),
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
                .unwrap(),
            ServiceWorkerFetchDispatch::Dispatched {
                registration_id: app,
                event_id: 120,
            }
        );
        assert_eq!(
            wait_for_fetch(&mut manager, 120),
            ServiceWorkerManagerEvent::FetchSettled {
                registration_id: app,
                event_id: 120,
                request_url: "https://example.test/app/data".into(),
                client_id: Some("client-1".into()),
                response: Some(ServiceWorkerFetchResponse {
                    status: 203,
                    status_text: "Scoped".into(),
                    response_type: "default".into(),
                    headers: vec![("x-scope".into(), "app".into())],
                    body: "app:https://example.test/app/data".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );

        assert_eq!(
            manager
                .dispatch_fetch(
                    "https://example.test",
                    121,
                    ServiceWorkerFetchRequest {
                        url: "https://example.test/other".into(),
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
                .unwrap(),
            ServiceWorkerFetchDispatch::Dispatched {
                registration_id: root,
                event_id: 121,
            }
        );
        assert_eq!(
            wait_for_fetch(&mut manager, 121),
            ServiceWorkerManagerEvent::FetchSettled {
                registration_id: root,
                event_id: 121,
                request_url: "https://example.test/other".into(),
                client_id: None,
                response: Some(ServiceWorkerFetchResponse {
                    status: 201,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "root".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn fetch_dispatch_uses_controlled_client_registration_before_request_scope() {
        let mut manager = manager_under_test();
        let _root = start_active(
            &mut manager,
            "/",
            "addEventListener('fetch', event => {
               event.respondWith(new Response('root'));
             });",
        );
        let app = start_active(
            &mut manager,
            "/app/",
            "addEventListener('fetch', event => {
               event.respondWith(new Response('app:' + event.clientId));
             });",
        );
        manager
            .observe_window_client("client-1", "https://example.test/app/page")
            .unwrap();

        assert_eq!(
            manager
                .dispatch_fetch(
                    "https://example.test",
                    122,
                    ServiceWorkerFetchRequest {
                        url: "https://example.test/sibling".into(),
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
                .unwrap(),
            ServiceWorkerFetchDispatch::Dispatched {
                registration_id: app,
                event_id: 122,
            }
        );
        assert_eq!(
            wait_for_fetch(&mut manager, 122),
            ServiceWorkerManagerEvent::FetchSettled {
                registration_id: app,
                event_id: 122,
                request_url: "https://example.test/sibling".into(),
                client_id: Some("client-1".into()),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "app:client-1".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn fetch_handler_can_respond_with_cache_storage_match() {
        let mut manager = manager_under_test();
        let registration_id = start_active(
            &mut manager,
            "/app/",
            "addEventListener('fetch', event => {
               event.respondWith(caches.match(event.request));
             });",
        );
        manager
            .put_cached_response(
                registration_id,
                "runtime",
                CacheRequest::new("https://example.test/app/cached"),
                zero_storage::CacheResponse::ok(b"cached-body".to_vec()).with_header("X-Cache", "hit"),
            )
            .unwrap();

        assert_eq!(
            manager
                .dispatch_fetch(
                    "https://example.test",
                    124,
                    ServiceWorkerFetchRequest {
                        url: "https://example.test/app/cached".into(),
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
                .unwrap(),
            ServiceWorkerFetchDispatch::Dispatched {
                registration_id,
                event_id: 124,
            }
        );
        assert_eq!(
            wait_for_fetch(&mut manager, 124),
            ServiceWorkerManagerEvent::FetchSettled {
                registration_id,
                event_id: 124,
                request_url: "https://example.test/app/cached".into(),
                client_id: Some("client-1".into()),
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
    fn fetch_handler_can_write_and_read_named_cache_storage() {
        let mut manager = manager_under_test();
        let registration_id = start_active(
            &mut manager,
            "/app/",
            "addEventListener('fetch', event => {
               event.respondWith((async () => {
                 const cache = await caches.open('runtime');
                 await cache.put(event.request, new Response('stored-body', {
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
        );

        assert_eq!(
            manager
                .dispatch_fetch(
                    "https://example.test",
                    125,
                    ServiceWorkerFetchRequest {
                        url: "https://example.test/app/stored".into(),
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
                .unwrap(),
            ServiceWorkerFetchDispatch::Dispatched {
                registration_id,
                event_id: 125,
            }
        );
        assert_eq!(
            wait_for_fetch(&mut manager, 125),
            ServiceWorkerManagerEvent::FetchSettled {
                registration_id,
                event_id: 125,
                request_url: "https://example.test/app/stored".into(),
                client_id: Some("client-1".into()),
                response: Some(ServiceWorkerFetchResponse {
                    status: 201,
                    status_text: "Created".into(),
                    response_type: "default".into(),
                    headers: vec![("x-cache".into(), "put".into())],
                    body: "stored-body".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );

        let cached = manager
            .registration(registration_id)
            .and_then(|registration| registration.cache_storage.get("runtime"))
            .and_then(|cache| cache.match_request(&CacheRequest::new("https://example.test/app/stored")))
            .expect("Cache.put should store into the registration CacheStorage");
        assert_eq!(cached.status, 201);
        assert_eq!(cached.status_text, "Created");
        assert_eq!(cached.response_type, "default");
        assert_eq!(cached.headers.get("x-cache").map(String::as_str), Some("put"));
        assert_eq!(cached.body, b"stored-body");
    }

    #[test]
    fn fetch_handler_can_delete_cache_entries_and_named_cache_storage() {
        let mut manager = manager_under_test();
        let registration_id = start_active(
            &mut manager,
            "/app/",
            "addEventListener('fetch', event => {
               event.respondWith((async () => {
                 const cache = await caches.open('runtime');
                 await cache.put(event.request, new Response('stored-body'));
                 const names = await caches.keys();
                 const hasBefore = await caches.has('runtime');
                 const entryDeleted = await cache.delete(new Request(event.request.url + '?miss=1'), {
                   ignoreSearch: true
                 });
                 const hasAfterEntryDelete = await cache.match(event.request) === undefined;
                 const cacheDeleted = await caches.delete('runtime');
                 const hasAfterStorageDelete = await caches.has('runtime');
                 return new Response([
                   names.join(','),
                   hasBefore,
                   entryDeleted,
                   hasAfterEntryDelete,
                   cacheDeleted,
                   hasAfterStorageDelete
                 ].join('|'));
               })());
             });",
        );

        assert_eq!(
            manager
                .dispatch_fetch(
                    "https://example.test",
                    127,
                    ServiceWorkerFetchRequest {
                        url: "https://example.test/app/delete".into(),
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
                .unwrap(),
            ServiceWorkerFetchDispatch::Dispatched {
                registration_id,
                event_id: 127,
            }
        );
        assert_eq!(
            wait_for_fetch(&mut manager, 127),
            ServiceWorkerManagerEvent::FetchSettled {
                registration_id,
                event_id: 127,
                request_url: "https://example.test/app/delete".into(),
                client_id: Some("client-1".into()),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "runtime|true|true|true|true|false".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
        assert!(
            manager
                .registration(registration_id)
                .is_some_and(|registration| !registration.cache_storage.has("runtime")),
            "CacheStorage.delete should remove the named registration cache"
        );
    }

    #[test]
    fn persistent_registration_round_trips_cache_storage() {
        let mut manager = manager_under_test();
        let registration_id = start_active(&mut manager, "/app/", "globalThis.ready = true;");
        manager
            .put_cached_response(
                registration_id,
                "runtime",
                CacheRequest::new("https://example.test/app/persisted"),
                zero_storage::CacheResponse::ok(b"persisted-body".to_vec()).with_header("x-cache", "restored"),
            )
            .unwrap();

        let mut persisted = manager.persistent_active_registrations();
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].cache_storage.caches.len(), 1);
        assert_eq!(persisted[0].cache_storage.caches[0].name, "runtime");
        assert_eq!(persisted[0].cache_storage.caches[0].entries.len(), 1);

        let mut restored = manager_under_test();
        let restored_id = restored.start_restored_active(persisted.pop().unwrap()).unwrap();
        wait_for_state(&mut restored, restored_id, ServiceWorkerState::Activated);
        let matched = restored
            .registration(restored_id)
            .and_then(|registration| registration.cache_storage.get("runtime"))
            .and_then(|cache| cache.match_request(&CacheRequest::new("https://example.test/app/persisted")))
            .expect("restored CacheStorage entry should match");
        assert_eq!(matched.body, b"persisted-body");
        assert_eq!(matched.headers.get("x-cache").map(String::as_str), Some("restored"));
    }

    #[test]
    fn fetch_handler_cache_storage_respects_vary_request_headers() {
        let mut manager = manager_under_test();
        let registration_id = start_active(
            &mut manager,
            "/app/",
            "addEventListener('fetch', event => {
               event.respondWith((async () => {
                 const cache = await caches.open('runtime');
                 await cache.put(event.request, new Response('cookie-body', {
                   headers: [['vary', 'cookies']]
                 }));
                 const strict = await cache.match(new Request(event.request.url));
                 const ignored = await cache.match(new Request(event.request.url), {ignoreVary: true});
                 const keys = await cache.keys(new Request(event.request.url));
                 if (strict !== undefined) throw new Error('strict vary matched');
                 if (!ignored) throw new Error('ignoreVary did not match');
                 if (keys.length !== 0) throw new Error('keys vary matched');
                 return ignored;
               })());
             });",
        );

        assert_eq!(
            manager
                .dispatch_fetch(
                    "https://example.test",
                    126,
                    ServiceWorkerFetchRequest {
                        url: "https://example.test/app/vary".into(),
                        method: "GET".into(),
                        headers: vec![("cookies".into(), "is-for-cookie".into())],
                        body: None,
                        credentials: None,
                        client_id: Some("client-1".into()),
                        resulting_client_id: None,
                        referrer: None,
                        is_reload_navigation: false,
                        is_history_navigation: false,
                    },
                )
                .unwrap(),
            ServiceWorkerFetchDispatch::Dispatched {
                registration_id,
                event_id: 126,
            }
        );
        assert_eq!(
            wait_for_fetch(&mut manager, 126),
            ServiceWorkerManagerEvent::FetchSettled {
                registration_id,
                event_id: 126,
                request_url: "https://example.test/app/vary".into(),
                client_id: Some("client-1".into()),
                response: Some(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: String::new(),
                    response_type: "default".into(),
                    headers: vec![("vary".into(), "cookies".into())],
                    body: "cookie-body".into(),
                }),
                failed: false,
                message: String::new(),
            }
        );
    }

    #[test]
    fn install_worker_fetch_request_completes_through_manager() {
        let mut manager = manager_under_test();
        let registration_id = start(
            &mut manager,
            "/app/",
            "addEventListener('install', event => {
               event.waitUntil((async () => {
                 const response = await fetch('/app/install.txt', {headers: {'X-Test': 'yes'}});
                 if ((await response.text()) !== 'install-body') throw new Error('wrong body');
               })());
             });",
        );

        let deadline = Instant::now() + Duration::from_secs(5);
        let request_id = loop {
            if let Some(request_id) = manager.poll().into_iter().find_map(|event| {
                if let ServiceWorkerManagerEvent::WorkerFetchRequested {
                    registration_id: id,
                    request_id,
                    request,
                } = event
                    && id == registration_id
                {
                    assert_eq!(request.url, "https://example.test/app/install.txt");
                    assert_eq!(request.method, "GET");
                    assert_eq!(request.headers, [("x-test".into(), "yes".into())]);
                    return Some(request_id);
                }
                None
            }) {
                break request_id;
            }
            assert!(Instant::now() < deadline, "worker fetch request timed out");
            std::thread::sleep(Duration::from_millis(1));
        };
        manager
            .complete_worker_fetch(
                registration_id,
                request_id,
                Ok(ServiceWorkerFetchResponse {
                    status: 200,
                    status_text: "OK".into(),
                    response_type: "default".into(),
                    headers: Vec::new(),
                    body: "install-body".into(),
                }),
            )
            .unwrap();
        wait_for_state(&mut manager, registration_id, ServiceWorkerState::Activated);
    }

    #[test]
    fn fetch_dispatch_passes_through_without_same_origin_active_worker() {
        let mut manager = manager_under_test();
        start_active(
            &mut manager,
            "/app/",
            "addEventListener('fetch', event => {
               event.respondWith(new Response('app'));
             });",
        );

        assert_eq!(
            manager
                .dispatch_fetch(
                    "https://example.test",
                    122,
                    ServiceWorkerFetchRequest {
                        url: "https://other.test/app/data".into(),
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
                .unwrap(),
            ServiceWorkerFetchDispatch::PassThrough
        );
        assert_eq!(
            manager
                .dispatch_fetch(
                    "https://example.test",
                    123,
                    ServiceWorkerFetchRequest {
                        url: "https://example.test/outside".into(),
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
                .unwrap(),
            ServiceWorkerFetchDispatch::PassThrough
        );
    }

    #[test]
    fn message_port_batch_validation_is_transactional() {
        let mut manager = manager_under_test();
        let id = start_active(&mut manager, "/", "globalThis.version = 1;");
        manager.message_ports.insert((id, "client-1".into(), 4));

        assert_eq!(
            manager.post_message_with_ports(
                id,
                95,
                "null",
                "client-1",
                "https://example.test/page",
                &ServiceWorkerMessagePorts {
                    transferred_port_ids: vec![2, 4],
                    data_port_index: None,
                    target_port_id: None,
                },
            ),
            Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker MessagePort endpoint was already transferred".into()
            ))
        );
        assert!(
            !manager.message_ports.contains(&(id, "client-1".into(), 2)),
            "a rejected transfer batch must not partially register endpoints"
        );

        assert_eq!(
            manager.record_outbound_message_ports(
                id,
                "client-1",
                &[ServiceWorkerOutboundMessage {
                    data_json: "\"invalid\"".into(),
                    port_id: Some(6),
                    transferred_port_ids: vec![8],
                    data_port_index: None,
                    target_client_id: None,
                }],
            ),
            Err(ServiceWorkerManagerError::InvalidInput(
                "outbound Service Worker MessagePort endpoint does not exist".into()
            ))
        );
        assert!(
            !manager.message_ports.contains(&(id, "client-1".into(), 8)),
            "a rejected outbound batch must not register transferred endpoints"
        );
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
            manager.client_messages_since(id, "client-late", 0).1[0].data_json,
            r#"{"errorName":"NetworkError"}"#
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
