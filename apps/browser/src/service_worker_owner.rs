//! Browser-process Service Worker registration owner.

use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, TryRecvError};

use serde::{Deserialize, Serialize};
use url::Url;
use zero_browser_shell::TabId;
use zero_net::HttpResponse;
use zero_page_runtime::{
    ServiceWorkerManager, ServiceWorkerManagerError, ServiceWorkerManagerEvent, ServiceWorkerPersistentRegistration,
    ServiceWorkerRegistrationErrorKind, ServiceWorkerUpdateOutcome, validate_service_worker_registration,
};
use zero_protocol::message::{
    ServiceWorkerClientMessages, ServiceWorkerError, ServiceWorkerErrorCode, ServiceWorkerOperation,
    ServiceWorkerRequestParams, ServiceWorkerResponseParams, ServiceWorkerResult, ServiceWorkerSnapshot,
    ServiceWorkerStateChanges, ServiceWorkerStateWire, ServiceWorkerUpdateViaCacheWire,
};
use zero_script_sandbox::SandboxConfig;
use zero_storage::{ServiceWorkerRegistration, ServiceWorkerState, ServiceWorkerUpdateViaCache};

const MAX_SCRIPT_BYTES: usize = 16 * 1024 * 1024;
const MAX_PERSISTED_FILE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_PERSISTED_REGISTRATIONS: usize = 32;
const PERSISTENCE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct PersistedServiceWorkers {
    version: u32,
    registrations: Vec<ServiceWorkerPersistentRegistration>,
}

enum ServiceWorkerFetchPurpose {
    Register {
        update_via_cache: ServiceWorkerUpdateViaCache,
    },
    Update {
        registration_id: u64,
        update_via_cache: ServiceWorkerUpdateViaCache,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ProfileKey {
    Normal,
    Private(TabId),
}

/// A validated script fetch that must run through the browser network owner.
pub(crate) struct ServiceWorkerFetchPlan {
    tab_id: TabId,
    request_id: u64,
    profile: ProfileKey,
    script_url: Url,
    scope: Url,
    origin: String,
    purpose: ServiceWorkerFetchPurpose,
}

impl ServiceWorkerFetchPlan {
    pub(crate) fn tab_id(&self) -> TabId {
        self.tab_id
    }

    pub(crate) fn script_url(&self) -> &str {
        self.script_url.as_str()
    }

    pub(crate) fn bypass_cache(&self) -> bool {
        match self.purpose {
            ServiceWorkerFetchPurpose::Register { .. } => true,
            ServiceWorkerFetchPurpose::Update { update_via_cache, .. } => {
                update_via_cache != ServiceWorkerUpdateViaCache::All
            }
        }
    }
}

/// Result of accepting one renderer request.
pub(crate) enum ServiceWorkerRequestDisposition {
    /// The request completed without network work.
    Respond(CompletedServiceWorkerResponse),
    /// The caller must attach a browser-owned script fetch.
    Fetch(ServiceWorkerFetchPlan),
}

/// Response ready to send to one renderer with the original IPC ID.
pub(crate) struct CompletedServiceWorkerResponse {
    pub(crate) tab_id: TabId,
    pub(crate) request_id: u64,
    pub(crate) params: ServiceWorkerResponseParams,
}

struct PendingScriptFetch {
    plan: ServiceWorkerFetchPlan,
    receiver: Receiver<Result<HttpResponse, String>>,
}

struct PendingEvaluation {
    tab_id: TabId,
    request_id: u64,
    update: bool,
}

/// Browser-process single owner for Service Worker managers and runtimes.
pub(crate) struct BrowserServiceWorkerOwner {
    normal: ServiceWorkerManager,
    private: HashMap<TabId, ServiceWorkerManager>,
    pending_fetches: Vec<PendingScriptFetch>,
    pending_evaluations: HashMap<(ProfileKey, u64), PendingEvaluation>,
    persistence_path: Option<PathBuf>,
    restoring: HashSet<u64>,
}

impl BrowserServiceWorkerOwner {
    pub(crate) fn new() -> Self {
        Self::empty(None)
    }

    pub(crate) fn with_persistence(path: PathBuf) -> Self {
        let mut owner = Self::empty(Some(path.clone()));
        match load_persisted_service_workers(&path) {
            Ok(registrations) => {
                let had_records = !registrations.is_empty();
                for registration in registrations {
                    match owner.normal.start_restored_active(
                        &registration.script_url,
                        &registration.scope,
                        &registration.origin,
                        &registration.script_source,
                        registration.update_via_cache,
                        SandboxConfig::default(),
                    ) {
                        Ok(registration_id) => {
                            owner.restoring.insert(registration_id);
                        }
                        Err(error) => {
                            tracing::warn!("Service Worker restore skipped: {error}");
                        }
                    }
                }
                if had_records
                    && owner.restoring.is_empty()
                    && let Err(error) = owner.persist_normal()
                {
                    tracing::warn!("Service Worker persistence cleanup failed: {error}");
                }
            }
            Err(error) => {
                tracing::warn!("Service Worker persistence load failed: {error}");
            }
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !owner.restoring.is_empty() {
            let _ = owner.poll();
            if std::time::Instant::now() >= deadline {
                tracing::warn!(
                    "Service Worker persistence restore timed out with {} registrations pending",
                    owner.restoring.len()
                );
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        owner
    }

    fn empty(persistence_path: Option<PathBuf>) -> Self {
        Self {
            normal: ServiceWorkerManager::new(),
            private: HashMap::new(),
            pending_fetches: Vec::new(),
            pending_evaluations: HashMap::new(),
            persistence_path,
            restoring: HashSet::new(),
        }
    }

    pub(crate) fn begin_request(
        &mut self,
        tab_id: TabId,
        private: bool,
        request_id: u64,
        authority_url: Option<&str>,
        params: ServiceWorkerRequestParams,
    ) -> ServiceWorkerRequestDisposition {
        self.begin_request_for_client(
            tab_id,
            private,
            request_id,
            authority_url,
            &tab_id.0.to_string(),
            params,
        )
    }

    pub(crate) fn begin_request_for_client(
        &mut self,
        tab_id: TabId,
        private: bool,
        request_id: u64,
        authority_url: Option<&str>,
        client_id: &str,
        params: ServiceWorkerRequestParams,
    ) -> ServiceWorkerRequestDisposition {
        if let Err(message) = params.validate() {
            return self.error_disposition(tab_id, request_id, ServiceWorkerErrorCode::InvalidArgument, message);
        }
        let Some(authority) = authority_url.and_then(|value| Url::parse(value).ok()) else {
            return self.error_disposition(
                tab_id,
                request_id,
                ServiceWorkerErrorCode::InvalidArgument,
                "Service Worker is unavailable before navigation commit",
            );
        };
        let profile = if private {
            ProfileKey::Private(tab_id)
        } else {
            ProfileKey::Normal
        };

        match params.operation {
            ServiceWorkerOperation::Register {
                script_url,
                scope,
                document_url,
                update_via_cache,
            } => {
                let Ok(renderer_document) = Url::parse(&document_url) else {
                    return self.error_disposition(
                        tab_id,
                        request_id,
                        ServiceWorkerErrorCode::InvalidArgument,
                        "invalid Service Worker document URL",
                    );
                };
                if renderer_document != authority {
                    return self.error_disposition(
                        tab_id,
                        request_id,
                        ServiceWorkerErrorCode::InvalidArgument,
                        "Service Worker document URL does not match the committed navigation",
                    );
                }
                match validate_service_worker_registration(&script_url, scope.as_deref(), &authority) {
                    Ok((script_url, scope, origin)) => ServiceWorkerRequestDisposition::Fetch(ServiceWorkerFetchPlan {
                        tab_id,
                        request_id,
                        profile,
                        script_url,
                        scope,
                        origin,
                        purpose: ServiceWorkerFetchPurpose::Register {
                            update_via_cache: update_via_cache_storage(update_via_cache),
                        },
                    }),
                    Err(error) => self.error_disposition(
                        tab_id,
                        request_id,
                        match error.kind {
                            ServiceWorkerRegistrationErrorKind::Type => ServiceWorkerErrorCode::InvalidArgument,
                            ServiceWorkerRegistrationErrorKind::Security => ServiceWorkerErrorCode::Security,
                        },
                        error.message,
                    ),
                }
            }
            ServiceWorkerOperation::Snapshot { registration_id } => {
                let result = self
                    .authorized_registration(profile, registration_id, &authority)
                    .map(snapshot)
                    .map(ServiceWorkerResult::Snapshot);
                self.result_disposition(tab_id, request_id, result)
            }
            ServiceWorkerOperation::Unregister { registration_id } => {
                let result = self
                    .authorized_registration(profile, registration_id, &authority)
                    .map(|_| {
                        let removed = self.manager_mut(profile).unregister(registration_id);
                        if removed
                            && profile == ProfileKey::Normal
                            && let Err(error) = self.persist_normal()
                        {
                            tracing::warn!("Service Worker persistence after unregister failed: {error}");
                        }
                        ServiceWorkerResult::Boolean(removed)
                    });
                self.result_disposition(tab_id, request_id, result)
            }
            ServiceWorkerOperation::ActivateWaiting { registration_id } => {
                let result = self
                    .authorized_registration(profile, registration_id, &authority)
                    .and_then(|_| {
                        self.manager_mut(profile)
                            .activate_waiting(registration_id)
                            .map_err(manager_error)
                    })
                    .map(|()| ServiceWorkerResult::Empty);
                self.result_disposition(tab_id, request_id, result)
            }
            ServiceWorkerOperation::GetRegistration { client_url } => {
                let result = validate_client_url(&client_url, &authority)
                    .map_err(|message| ServiceWorkerError {
                        code: ServiceWorkerErrorCode::InvalidArgument,
                        message: message.into(),
                    })
                    .map(|client_url| {
                        self.manager(profile)
                            .and_then(|manager| {
                                manager.registration_for_url(
                                    &authority.origin().ascii_serialization(),
                                    client_url.as_str(),
                                )
                            })
                            .cloned()
                            .map(snapshot)
                    })
                    .map(ServiceWorkerResult::OptionalSnapshot);
                self.result_disposition(tab_id, request_id, result)
            }
            ServiceWorkerOperation::GetRegistrations => {
                let registrations = self
                    .manager(profile)
                    .map(|manager| {
                        manager
                            .registrations_for_origin(&authority.origin().ascii_serialization())
                            .into_iter()
                            .cloned()
                            .map(snapshot)
                            .collect()
                    })
                    .unwrap_or_default();
                self.result_disposition(tab_id, request_id, Ok(ServiceWorkerResult::Snapshots(registrations)))
            }
            ServiceWorkerOperation::StateChanges {
                registration_id,
                after_sequence,
            } => {
                let result = self
                    .authorized_registration(profile, registration_id, &authority)
                    .and_then(|_| {
                        let manager = self.manager(profile).ok_or_else(|| ServiceWorkerError {
                            code: ServiceWorkerErrorCode::NotFound,
                            message: "Service Worker registration does not exist".into(),
                        })?;
                        let (latest_sequence, states) = manager
                            .state_changes_since(registration_id, after_sequence)
                            .ok_or_else(|| ServiceWorkerError {
                                code: ServiceWorkerErrorCode::NotFound,
                                message: "Service Worker registration does not exist".into(),
                            })?;
                        Ok(ServiceWorkerResult::StateChanges(ServiceWorkerStateChanges {
                            latest_sequence,
                            states: states.iter().copied().map(state_wire).collect(),
                            claim_clients: manager.claims_clients(registration_id),
                        }))
                    });
                self.result_disposition(tab_id, request_id, result)
            }
            ServiceWorkerOperation::Controller => {
                let controller = self
                    .manager(profile)
                    .and_then(|manager| {
                        manager
                            .active_registration_for_url(&authority.origin().ascii_serialization(), authority.as_str())
                    })
                    .cloned()
                    .map(snapshot);
                self.result_disposition(
                    tab_id,
                    request_id,
                    Ok(ServiceWorkerResult::OptionalSnapshot(controller)),
                )
            }
            ServiceWorkerOperation::PostMessage {
                registration_id,
                data_json,
            } => {
                let result = self
                    .authorized_registration(profile, registration_id, &authority)
                    .and_then(|_| {
                        self.manager_mut(profile)
                            .post_message(registration_id, request_id, &data_json, client_id, authority.as_str())
                            .map_err(manager_error)
                    })
                    .map(|()| ServiceWorkerResult::Empty);
                self.result_disposition(tab_id, request_id, result)
            }
            ServiceWorkerOperation::ClientMessages {
                registration_id,
                after_sequence,
            } => {
                let result = self
                    .authorized_registration(profile, registration_id, &authority)
                    .map(|_| {
                        let (latest_sequence, data_json) = self
                            .manager(profile)
                            .map(|manager| manager.client_messages_since(registration_id, client_id, after_sequence))
                            .unwrap_or_default();
                        ServiceWorkerResult::ClientMessages(ServiceWorkerClientMessages {
                            latest_sequence,
                            data_json,
                        })
                    });
                self.result_disposition(tab_id, request_id, result)
            }
            ServiceWorkerOperation::Update { registration_id } => {
                if let Err(error) = self.authorized_registration(profile, registration_id, &authority) {
                    return self.result_disposition(tab_id, request_id, Err(error));
                }
                let registration = match self
                    .manager(profile)
                    .expect("authorized registration requires a manager")
                    .update_target(registration_id)
                    .map_err(manager_error)
                {
                    Ok(registration) => registration.clone(),
                    Err(error) => return self.result_disposition(tab_id, request_id, Err(error)),
                };
                let (Ok(script_url), Ok(scope)) =
                    (Url::parse(&registration.script_url), Url::parse(&registration.scope))
                else {
                    return self.error_disposition(
                        tab_id,
                        request_id,
                        ServiceWorkerErrorCode::Internal,
                        "Service Worker registration URLs are invalid",
                    );
                };
                ServiceWorkerRequestDisposition::Fetch(ServiceWorkerFetchPlan {
                    tab_id,
                    request_id,
                    profile,
                    script_url,
                    scope,
                    origin: registration.origin,
                    purpose: ServiceWorkerFetchPurpose::Update {
                        registration_id,
                        update_via_cache: registration.update_via_cache,
                    },
                })
            }
        }
    }

    pub(crate) fn attach_fetch(
        &mut self,
        plan: ServiceWorkerFetchPlan,
        receiver: Receiver<Result<HttpResponse, String>>,
    ) {
        self.pending_fetches.push(PendingScriptFetch { plan, receiver });
    }

    pub(crate) fn poll(&mut self) -> Vec<CompletedServiceWorkerResponse> {
        let mut completed = Vec::new();
        self.poll_fetches(&mut completed);
        self.poll_manager(ProfileKey::Normal, &mut completed);
        let private_tabs: Vec<_> = self.private.keys().copied().collect();
        for tab_id in private_tabs {
            self.poll_manager(ProfileKey::Private(tab_id), &mut completed);
        }
        completed
    }

    pub(crate) fn remove_tab(&mut self, tab_id: TabId) {
        self.private.remove(&tab_id);
        self.disconnect_tab(tab_id);
    }

    pub(crate) fn disconnect_tab(&mut self, tab_id: TabId) {
        self.pending_fetches.retain(|pending| pending.plan.tab_id != tab_id);
        self.pending_evaluations.retain(|_, pending| pending.tab_id != tab_id);
    }

    pub(crate) fn remove_private_profile(&mut self, tab_id: TabId) {
        self.private.remove(&tab_id);
        self.pending_fetches
            .retain(|pending| pending.plan.profile != ProfileKey::Private(tab_id));
        self.pending_evaluations
            .retain(|(profile, _), _| *profile != ProfileKey::Private(tab_id));
    }

    fn poll_fetches(&mut self, completed: &mut Vec<CompletedServiceWorkerResponse>) {
        let mut pending = Vec::new();
        for item in std::mem::take(&mut self.pending_fetches) {
            match item.receiver.try_recv() {
                Ok(result) => self.complete_fetch(item.plan, result, completed),
                Err(TryRecvError::Empty) => pending.push(item),
                Err(TryRecvError::Disconnected) => completed.push(error_response(
                    item.plan.tab_id,
                    item.plan.request_id,
                    ServiceWorkerErrorCode::Network,
                    "Service Worker script fetch worker exited",
                )),
            }
        }
        self.pending_fetches = pending;
    }

    fn complete_fetch(
        &mut self,
        plan: ServiceWorkerFetchPlan,
        result: Result<HttpResponse, String>,
        completed: &mut Vec<CompletedServiceWorkerResponse>,
    ) {
        let response = match result {
            Ok(response) => response,
            Err(message) => {
                completed.push(error_response(
                    plan.tab_id,
                    plan.request_id,
                    ServiceWorkerErrorCode::Network,
                    format!("Service Worker script fetch failed: {message}"),
                ));
                return;
            }
        };
        if !response.is_success() {
            completed.push(error_response(
                plan.tab_id,
                plan.request_id,
                ServiceWorkerErrorCode::Network,
                format!("Service Worker script fetch returned HTTP {}", response.status_code),
            ));
            return;
        }
        if response.redirect_count != 0
            || Url::parse(&response.url).ok().is_none_or(|final_url| {
                final_url.origin() != plan.script_url.origin() || final_url.fragment().is_some()
            })
        {
            completed.push(error_response(
                plan.tab_id,
                plan.request_id,
                ServiceWorkerErrorCode::Network,
                "Service Worker script fetch redirected",
            ));
            return;
        }
        if response.body.len() > MAX_SCRIPT_BYTES {
            completed.push(error_response(
                plan.tab_id,
                plan.request_id,
                ServiceWorkerErrorCode::Capacity,
                "Service Worker script exceeds the size limit",
            ));
            return;
        }
        let script = match String::from_utf8(response.body) {
            Ok(script) => script,
            Err(_) => {
                completed.push(error_response(
                    plan.tab_id,
                    plan.request_id,
                    ServiceWorkerErrorCode::Script,
                    "Service Worker script is not valid UTF-8",
                ));
                return;
            }
        };
        let result = match plan.purpose {
            ServiceWorkerFetchPurpose::Register { update_via_cache } => self
                .manager_mut(plan.profile)
                .start_evaluation_with_update_via_cache(
                    plan.script_url.as_str(),
                    plan.scope.as_str(),
                    &plan.origin,
                    &script,
                    update_via_cache,
                    SandboxConfig::default(),
                )
                .map(|registration_id| (registration_id, false)),
            ServiceWorkerFetchPurpose::Update { registration_id, .. } => {
                match self
                    .manager_mut(plan.profile)
                    .start_update(registration_id, &script, SandboxConfig::default())
                {
                    Ok(ServiceWorkerUpdateOutcome::Unchanged { registration_id }) => {
                        completed.push(success_response(
                            plan.tab_id,
                            plan.request_id,
                            ServiceWorkerResult::Updated {
                                registration_id,
                                changed: false,
                            },
                        ));
                        return;
                    }
                    Ok(ServiceWorkerUpdateOutcome::Started { registration_id }) => Ok((registration_id, true)),
                    Err(error) => Err(error),
                }
            }
        };
        match result {
            Ok((registration_id, update)) => {
                self.pending_evaluations.insert(
                    (plan.profile, registration_id),
                    PendingEvaluation {
                        tab_id: plan.tab_id,
                        request_id: plan.request_id,
                        update,
                    },
                );
            }
            Err(error) => {
                let error = manager_error(error);
                completed.push(error_response(plan.tab_id, plan.request_id, error.code, error.message));
            }
        }
    }

    fn poll_manager(&mut self, profile: ProfileKey, completed: &mut Vec<CompletedServiceWorkerResponse>) {
        let events = self.manager_mut(profile).poll();
        let mut persistence_dirty = false;
        for event in events {
            match event {
                ServiceWorkerManagerEvent::ScriptEvaluated { registration_id } => {
                    if let Some(pending) = self.pending_evaluations.remove(&(profile, registration_id)) {
                        let result = if pending.update {
                            ServiceWorkerResult::Updated {
                                registration_id,
                                changed: true,
                            }
                        } else {
                            ServiceWorkerResult::Registered { registration_id }
                        };
                        completed.push(success_response(pending.tab_id, pending.request_id, result));
                    }
                }
                ServiceWorkerManagerEvent::ScriptFailed {
                    registration_id,
                    message,
                    ..
                }
                | ServiceWorkerManagerEvent::CoordinationFailed {
                    registration_id,
                    message,
                } => {
                    if profile == ProfileKey::Normal
                        && self.restoring.remove(&registration_id)
                        && self.restoring.is_empty()
                    {
                        persistence_dirty = true;
                    }
                    if let Some(pending) = self.pending_evaluations.remove(&(profile, registration_id)) {
                        completed.push(error_response(
                            pending.tab_id,
                            pending.request_id,
                            ServiceWorkerErrorCode::Script,
                            message,
                        ));
                    }
                }
                ServiceWorkerManagerEvent::InstallCompleted {
                    registration_id,
                    succeeded: false,
                } => {
                    if profile == ProfileKey::Normal
                        && self.restoring.remove(&registration_id)
                        && self.restoring.is_empty()
                    {
                        persistence_dirty = true;
                    }
                }
                ServiceWorkerManagerEvent::ActivationCompleted {
                    registration_id,
                    succeeded,
                } if profile == ProfileKey::Normal => {
                    let restored = self.restoring.remove(&registration_id);
                    if self.restoring.is_empty() && (restored || succeeded) {
                        persistence_dirty = true;
                    }
                }
                ServiceWorkerManagerEvent::RestorationCompleted { registration_id }
                    if profile == ProfileKey::Normal
                        && self.restoring.remove(&registration_id)
                        && self.restoring.is_empty() =>
                {
                    persistence_dirty = true;
                }
                _ => {}
            }
        }
        if persistence_dirty && let Err(error) = self.persist_normal() {
            tracing::warn!("Service Worker persistence update failed: {error}");
        }
    }

    fn persist_normal(&self) -> Result<(), String> {
        let Some(path) = &self.persistence_path else {
            return Ok(());
        };
        let state = PersistedServiceWorkers {
            version: PERSISTENCE_VERSION,
            registrations: self.normal.persistent_active_registrations(),
        };
        let json = serde_json::to_string(&state).map_err(|error| format!("serialize state failed: {error}"))?;
        if json.len() as u64 > MAX_PERSISTED_FILE_BYTES {
            return Err("serialized state exceeds the size limit".into());
        }
        atomic_write_persistence(path, &json)
    }

    fn manager(&self, profile: ProfileKey) -> Option<&ServiceWorkerManager> {
        match profile {
            ProfileKey::Normal => Some(&self.normal),
            ProfileKey::Private(tab_id) => self.private.get(&tab_id),
        }
    }

    fn manager_mut(&mut self, profile: ProfileKey) -> &mut ServiceWorkerManager {
        match profile {
            ProfileKey::Normal => &mut self.normal,
            ProfileKey::Private(tab_id) => self.private.entry(tab_id).or_default(),
        }
    }

    fn authorized_registration(
        &self,
        profile: ProfileKey,
        registration_id: u64,
        authority: &Url,
    ) -> Result<ServiceWorkerRegistration, ServiceWorkerError> {
        let registration = self
            .manager(profile)
            .and_then(|manager| manager.registration(registration_id))
            .ok_or_else(|| ServiceWorkerError {
                code: ServiceWorkerErrorCode::NotFound,
                message: "Service Worker registration does not exist".into(),
            })?;
        if registration.origin != authority.origin().ascii_serialization() {
            return Err(ServiceWorkerError {
                code: ServiceWorkerErrorCode::NotFound,
                message: "Service Worker registration does not exist".into(),
            });
        }
        Ok(registration.clone())
    }

    fn result_disposition(
        &self,
        tab_id: TabId,
        request_id: u64,
        result: Result<ServiceWorkerResult, ServiceWorkerError>,
    ) -> ServiceWorkerRequestDisposition {
        ServiceWorkerRequestDisposition::Respond(CompletedServiceWorkerResponse {
            tab_id,
            request_id,
            params: ServiceWorkerResponseParams { result },
        })
    }

    fn error_disposition(
        &self,
        tab_id: TabId,
        request_id: u64,
        code: ServiceWorkerErrorCode,
        message: impl Into<String>,
    ) -> ServiceWorkerRequestDisposition {
        ServiceWorkerRequestDisposition::Respond(error_response(tab_id, request_id, code, message))
    }
}

impl Default for BrowserServiceWorkerOwner {
    fn default() -> Self {
        Self::new()
    }
}

fn load_persisted_service_workers(path: &Path) -> Result<Vec<ServiceWorkerPersistentRegistration>, String> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("read state metadata failed: {error}")),
    };
    if metadata.len() > MAX_PERSISTED_FILE_BYTES {
        return Err("state file exceeds the size limit".into());
    }
    let mut source = String::new();
    File::open(path)
        .map_err(|error| format!("open state failed: {error}"))?
        .take(MAX_PERSISTED_FILE_BYTES + 1)
        .read_to_string(&mut source)
        .map_err(|error| format!("read state failed: {error}"))?;
    if source.len() as u64 > MAX_PERSISTED_FILE_BYTES {
        return Err("state file exceeds the size limit".into());
    }
    let state = serde_json::from_str::<PersistedServiceWorkers>(&source)
        .map_err(|error| format!("parse state failed: {error}"))?;
    if state.version != PERSISTENCE_VERSION {
        return Err(format!("unsupported state version {}", state.version));
    }
    if state.registrations.len() > MAX_PERSISTED_REGISTRATIONS {
        return Err("state has too many registrations".into());
    }

    let mut keys = HashSet::new();
    let mut total_script_bytes = 0usize;
    for registration in &state.registrations {
        total_script_bytes = total_script_bytes
            .checked_add(registration.script_source.len())
            .ok_or_else(|| "state script size overflow".to_string())?;
        if total_script_bytes as u64 > MAX_PERSISTED_FILE_BYTES {
            return Err("state scripts exceed the size limit".into());
        }
        let document = Url::parse(&registration.origin).map_err(|_| "state origin is invalid".to_string())?;
        if document.origin().ascii_serialization() != registration.origin {
            return Err("state origin is not canonical".into());
        }
        let (script_url, scope, origin) =
            validate_service_worker_registration(&registration.script_url, Some(&registration.scope), &document)
                .map_err(|error| format!("state registration is invalid: {}", error.message))?;
        if script_url.as_str() != registration.script_url
            || scope.as_str() != registration.scope
            || origin != registration.origin
        {
            return Err("state registration URLs are not canonical".into());
        }
        if !keys.insert((registration.origin.clone(), registration.scope.clone())) {
            return Err("state contains duplicate registration keys".into());
        }
    }
    Ok(state.registrations)
}

fn atomic_write_persistence(path: &Path, content: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("state path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| format!("create state directory failed: {error}"))?;
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let mut file = File::create(&temporary).map_err(|error| format!("create temporary state failed: {error}"))?;
    file.write_all(content.as_bytes())
        .map_err(|error| format!("write temporary state failed: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("sync temporary state failed: {error}"))?;
    fs::rename(&temporary, path).map_err(|error| format!("replace state failed: {error}"))?;
    #[cfg(unix)]
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("sync state directory failed: {error}"))?;
    Ok(())
}

fn validate_client_url(client_url: &str, document: &Url) -> Result<Url, &'static str> {
    let mut client = document
        .join(client_url)
        .map_err(|_| "invalid Service Worker client URL")?;
    if !matches!(client.scheme(), "http" | "https") || client.origin() != document.origin() {
        return Err("Service Worker client URL must be same-origin http(s)");
    }
    client.set_fragment(None);
    Ok(client)
}

fn snapshot(registration: ServiceWorkerRegistration) -> ServiceWorkerSnapshot {
    ServiceWorkerSnapshot {
        registration_id: registration.id,
        script_url: registration.script_url,
        scope: registration.scope,
        update_via_cache: update_via_cache_wire(registration.update_via_cache),
        state: state_wire(registration.state),
    }
}

fn update_via_cache_storage(value: ServiceWorkerUpdateViaCacheWire) -> ServiceWorkerUpdateViaCache {
    match value {
        ServiceWorkerUpdateViaCacheWire::Imports => ServiceWorkerUpdateViaCache::Imports,
        ServiceWorkerUpdateViaCacheWire::All => ServiceWorkerUpdateViaCache::All,
        ServiceWorkerUpdateViaCacheWire::None => ServiceWorkerUpdateViaCache::None,
    }
}

fn update_via_cache_wire(value: ServiceWorkerUpdateViaCache) -> ServiceWorkerUpdateViaCacheWire {
    match value {
        ServiceWorkerUpdateViaCache::Imports => ServiceWorkerUpdateViaCacheWire::Imports,
        ServiceWorkerUpdateViaCache::All => ServiceWorkerUpdateViaCacheWire::All,
        ServiceWorkerUpdateViaCache::None => ServiceWorkerUpdateViaCacheWire::None,
    }
}

fn state_wire(state: ServiceWorkerState) -> ServiceWorkerStateWire {
    match state {
        ServiceWorkerState::Registered | ServiceWorkerState::Installing => ServiceWorkerStateWire::Installing,
        ServiceWorkerState::Installed => ServiceWorkerStateWire::Installed,
        ServiceWorkerState::Activating => ServiceWorkerStateWire::Activating,
        ServiceWorkerState::Activated => ServiceWorkerStateWire::Activated,
        ServiceWorkerState::Redundant => ServiceWorkerStateWire::Redundant,
    }
}

fn manager_error(error: ServiceWorkerManagerError) -> ServiceWorkerError {
    let code = match error {
        ServiceWorkerManagerError::InvalidInput(_) => ServiceWorkerErrorCode::InvalidArgument,
        ServiceWorkerManagerError::UnknownRegistration(_) => ServiceWorkerErrorCode::NotFound,
        ServiceWorkerManagerError::JobInProgress(_)
        | ServiceWorkerManagerError::EvaluationPending(_)
        | ServiceWorkerManagerError::InvalidState { .. } => ServiceWorkerErrorCode::InvalidState,
        ServiceWorkerManagerError::CapacityExceeded { .. }
        | ServiceWorkerManagerError::ClientCapacityExceeded { .. }
        | ServiceWorkerManagerError::ClientMessageCapacityExceeded { .. } => ServiceWorkerErrorCode::Capacity,
        ServiceWorkerManagerError::Runtime(_) => ServiceWorkerErrorCode::Internal,
    };
    ServiceWorkerError {
        code,
        message: error.to_string(),
    }
}

fn success_response(tab_id: TabId, request_id: u64, result: ServiceWorkerResult) -> CompletedServiceWorkerResponse {
    CompletedServiceWorkerResponse {
        tab_id,
        request_id,
        params: ServiceWorkerResponseParams { result: Ok(result) },
    }
}

fn error_response(
    tab_id: TabId,
    request_id: u64,
    code: ServiceWorkerErrorCode,
    message: impl Into<String>,
) -> CompletedServiceWorkerResponse {
    CompletedServiceWorkerResponse {
        tab_id,
        request_id,
        params: ServiceWorkerResponseParams {
            result: Err(ServiceWorkerError {
                code,
                message: message.into(),
            }),
        },
    }
}

#[cfg(test)]
#[path = "service_worker_owner/tests.rs"]
mod tests;
