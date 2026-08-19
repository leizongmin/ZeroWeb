//! Browser-process Service Worker registration owner.

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, TryRecvError};

use url::Url;
use zero_browser_shell::TabId;
use zero_net::HttpResponse;
use zero_page_runtime::{ServiceWorkerManager, ServiceWorkerManagerError, ServiceWorkerManagerEvent};
use zero_protocol::message::{
    ServiceWorkerError, ServiceWorkerErrorCode, ServiceWorkerOperation, ServiceWorkerRequestParams,
    ServiceWorkerResponseParams, ServiceWorkerResult, ServiceWorkerSnapshot, ServiceWorkerStateWire,
};
use zero_script_sandbox::SandboxConfig;
use zero_storage::{ServiceWorkerRegistration, ServiceWorkerState};

const MAX_SCRIPT_BYTES: usize = 16 * 1024 * 1024;

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
                match validate_registration(&script_url, scope.as_deref(), &authority) {
                    Ok((script_url, scope, origin)) => ServiceWorkerRequestDisposition::Fetch(ServiceWorkerFetchPlan {
                        tab_id,
                        request_id,
                        profile,
                        script_url,
                        scope,
                        origin,
                    }),
                    Err(message) => {
                        self.error_disposition(tab_id, request_id, ServiceWorkerErrorCode::InvalidArgument, message)
                    }
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
        let result = self.manager_mut(plan.profile).start_evaluation(
            plan.script_url.as_str(),
            plan.scope.as_str(),
            &plan.origin,
            &script,
            SandboxConfig::default(),
        );
        match result {
            Ok(registration_id) => {
                self.pending_evaluations.insert(
                    (plan.profile, registration_id),
                    PendingEvaluation {
                        tab_id: plan.tab_id,
                        request_id: plan.request_id,
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
                        completed.push(success_response(
                            pending.tab_id,
                            pending.request_id,
                            ServiceWorkerResult::Registered { registration_id },
                        ));
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

fn validate_registration(
    script_url: &str,
    scope: Option<&str>,
    document: &Url,
) -> Result<(Url, Url, String), &'static str> {
    let secure = document.scheme() == "https"
        || (document.scheme() == "http"
            && document.host_str().is_some_and(|host| {
                host == "localhost" || host.parse::<std::net::IpAddr>().is_ok_and(|ip| ip.is_loopback())
            }));
    if !secure {
        return Err("Service Worker registration requires a secure context");
    }
    let mut script = document
        .join(script_url)
        .map_err(|_| "invalid Service Worker script URL")?;
    if !matches!(script.scheme(), "http" | "https") || script.origin() != document.origin() {
        return Err("Service Worker script URL must be same-origin http(s)");
    }
    if script.fragment().is_some() {
        return Err("Service Worker script URL must not contain a fragment");
    }
    script.set_fragment(None);

    let scope = match scope {
        Some(value) => document.join(value).map_err(|_| "invalid Service Worker scope")?,
        None => script.join("./").map_err(|_| "invalid default Service Worker scope")?,
    };
    if !matches!(scope.scheme(), "http" | "https") || scope.origin() != document.origin() {
        return Err("Service Worker scope must be same-origin http(s)");
    }
    if scope.fragment().is_some() {
        return Err("Service Worker scope must not contain a fragment");
    }
    Ok((script, scope, document.origin().ascii_serialization()))
}

fn snapshot(registration: ServiceWorkerRegistration) -> ServiceWorkerSnapshot {
    ServiceWorkerSnapshot {
        registration_id: registration.id,
        script_url: registration.script_url,
        scope: registration.scope,
        state: match registration.state {
            ServiceWorkerState::Registered | ServiceWorkerState::Installing => ServiceWorkerStateWire::Installing,
            ServiceWorkerState::Installed => ServiceWorkerStateWire::Installed,
            ServiceWorkerState::Activating => ServiceWorkerStateWire::Activating,
            ServiceWorkerState::Activated => ServiceWorkerStateWire::Activated,
            ServiceWorkerState::Redundant => ServiceWorkerStateWire::Redundant,
        },
    }
}

fn manager_error(error: ServiceWorkerManagerError) -> ServiceWorkerError {
    let code = match error {
        ServiceWorkerManagerError::InvalidInput(_) => ServiceWorkerErrorCode::InvalidArgument,
        ServiceWorkerManagerError::UnknownRegistration(_) => ServiceWorkerErrorCode::NotFound,
        ServiceWorkerManagerError::JobInProgress(_)
        | ServiceWorkerManagerError::EvaluationPending(_)
        | ServiceWorkerManagerError::InvalidState { .. } => ServiceWorkerErrorCode::InvalidState,
        ServiceWorkerManagerError::CapacityExceeded { .. } => ServiceWorkerErrorCode::Capacity,
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
