//! Browser-process Service Worker registration owner.

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, TryRecvError};

use url::Url;
use zero_browser_shell::TabId;
use zero_net::HttpResponse;
use zero_page_runtime::{
    ServiceWorkerManager, ServiceWorkerManagerError, ServiceWorkerManagerEvent, ServiceWorkerRegistrationErrorKind,
    ServiceWorkerUpdateOutcome, validate_service_worker_registration,
};
use zero_protocol::message::{
    ServiceWorkerClientMessages, ServiceWorkerError, ServiceWorkerErrorCode, ServiceWorkerOperation,
    ServiceWorkerRequestParams, ServiceWorkerResponseParams, ServiceWorkerResult, ServiceWorkerSnapshot,
    ServiceWorkerStateChanges, ServiceWorkerStateWire,
};
use zero_script_sandbox::SandboxConfig;
use zero_storage::{ServiceWorkerRegistration, ServiceWorkerState};

const MAX_SCRIPT_BYTES: usize = 16 * 1024 * 1024;

enum ServiceWorkerFetchPurpose {
    Register,
    Update { registration_id: u64 },
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
}

impl BrowserServiceWorkerOwner {
    pub(crate) fn new() -> Self {
        Self {
            normal: ServiceWorkerManager::new(),
            private: HashMap::new(),
            pending_fetches: Vec::new(),
            pending_evaluations: HashMap::new(),
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
                        purpose: ServiceWorkerFetchPurpose::Register,
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
                    .map(|_| ServiceWorkerResult::Boolean(self.manager_mut(profile).unregister(registration_id)));
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
                    purpose: ServiceWorkerFetchPurpose::Update { registration_id },
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
            ServiceWorkerFetchPurpose::Register => self
                .manager_mut(plan.profile)
                .start_evaluation(
                    plan.script_url.as_str(),
                    plan.scope.as_str(),
                    &plan.origin,
                    &script,
                    SandboxConfig::default(),
                )
                .map(|registration_id| (registration_id, false)),
            ServiceWorkerFetchPurpose::Update { registration_id } => {
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
                    if let Some(pending) = self.pending_evaluations.remove(&(profile, registration_id)) {
                        completed.push(error_response(
                            pending.tab_id,
                            pending.request_id,
                            ServiceWorkerErrorCode::Script,
                            message,
                        ));
                    }
                }
                _ => {}
            }
        }
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
        state: state_wire(registration.state),
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
