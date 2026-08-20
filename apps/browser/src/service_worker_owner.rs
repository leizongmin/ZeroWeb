//! Browser-process Service Worker registration owner.

use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use url::Url;
use zero_browser_shell::TabId;
use zero_net::{HttpResponse, is_javascript_mime};
use zero_page_runtime::{
    ServiceWorkerImportedScript, ServiceWorkerManager, ServiceWorkerManagerError, ServiceWorkerManagerEvent,
    ServiceWorkerPersistentRegistration, ServiceWorkerRegistrationErrorKind, ServiceWorkerRuntimeHost,
    ServiceWorkerUpdateOutcome, validate_service_worker_registration,
};
use zero_protocol::message::{
    ServiceWorkerClientMessages, ServiceWorkerError, ServiceWorkerErrorCode, ServiceWorkerHostCommand,
    ServiceWorkerHostCommandParams, ServiceWorkerHostEvent, ServiceWorkerHostEventParams, ServiceWorkerLifecycleWire,
    ServiceWorkerOperation, ServiceWorkerRequestParams, ServiceWorkerResponseParams, ServiceWorkerResult,
    ServiceWorkerScriptErrorKindWire, ServiceWorkerSnapshot, ServiceWorkerStateChanges, ServiceWorkerStateWire,
    ServiceWorkerUpdateViaCacheWire,
};
use zero_script_sandbox::{ServiceWorkerEvent, ServiceWorkerLifecyclePhase, ServiceWorkerScriptErrorKind};
use zero_storage::{ServiceWorkerRegistration, ServiceWorkerState, ServiceWorkerUpdateViaCache};

const MAX_SCRIPT_BYTES: usize = 16 * 1024 * 1024;
const MAX_PERSISTED_FILE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_PERSISTED_REGISTRATIONS: usize = 32;
const MAX_PERSISTED_IMPORTS_PER_REGISTRATION: usize = 1024;
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

/// 待下发 renderer 的托管命令（`tab_id` 定位宿主 renderer 连接）。
pub(crate) struct ServiceWorkerHostOutgoing {
    pub(crate) tab_id: TabId,
    pub(crate) params: ServiceWorkerHostCommandParams,
}

/// 单一 profile 的 browser↔renderer host 通道（`IpcServiceWorkerHost` 与 owner 共享）。
///
/// `pending_tab` 是一次性的「下一次 `start_evaluation` 的宿主 tab」——owner 在
/// `complete_fetch` 里于调用 manager 前设置，host 的 `evaluate` 消费。
#[derive(Clone, Default)]
struct SharedHostChannels {
    inbox: Arc<Mutex<Vec<(u64, ServiceWorkerEvent)>>>,
    outbox: Arc<Mutex<Vec<ServiceWorkerHostOutgoing>>>,
    pending_tab: Arc<Mutex<Option<TabId>>>,
    /// registration_id → 托管 renderer tab（runtime 存活集合；断连时反查注入 Closed）。
    owned: Arc<Mutex<HashMap<u64, TabId>>>,
}

impl SharedHostChannels {
    fn set_pending_tab(&self, tab_id: TabId) {
        *self.pending_tab.lock().unwrap_or_else(|error| error.into_inner()) = Some(tab_id);
    }

    fn push_event(&self, registration_id: u64, event: ServiceWorkerEvent) {
        self.inbox
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .push((registration_id, event));
    }

    fn push_outgoing(&self, outgoing: ServiceWorkerHostOutgoing) {
        self.outbox
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .push(outgoing);
    }

    fn take_outgoing(&self) -> Vec<ServiceWorkerHostOutgoing> {
        std::mem::take(&mut *self.outbox.lock().unwrap_or_else(|error| error.into_inner()))
    }

    fn take_owned_tab(&self, registration_id: u64) -> Option<TabId> {
        self.owned
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(&registration_id)
            .copied()
    }

    fn record_owned(&self, registration_id: u64, tab_id: TabId) {
        self.owned
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .insert(registration_id, tab_id);
    }

    fn remove_owned(&self, registration_id: u64) -> Option<TabId> {
        self.owned
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(&registration_id)
    }

    /// 移除并返回 `tab_id` 托管的全部 registration（renderer 死亡时用）。
    fn remove_owned_by_tab(&self, tab_id: TabId) -> Vec<u64> {
        let mut owned = self.owned.lock().unwrap_or_else(|error| error.into_inner());
        let ids: Vec<u64> = owned
            .iter()
            .filter(|(_, owner)| **owner == tab_id)
            .map(|(id, _)| *id)
            .collect();
        for id in &ids {
            owned.remove(id);
        }
        ids
    }

    fn owned_count(&self) -> usize {
        self.owned.lock().unwrap_or_else(|error| error.into_inner()).len()
    }
}

/// Browser 侧 [`ServiceWorkerRuntimeHost`]：命令经 IPC 转发给宿主 renderer 进程，
/// 事件由 process_backend 从 renderer 消息流注入 inbox。JS 引擎只存在于 renderer。
struct IpcServiceWorkerHost {
    channels: SharedHostChannels,
}

impl IpcServiceWorkerHost {
    fn new(channels: SharedHostChannels) -> Self {
        Self { channels }
    }
}

impl ServiceWorkerRuntimeHost for IpcServiceWorkerHost {
    fn evaluate(
        &mut self,
        registration_id: u64,
        script_url: &str,
        script: &str,
    ) -> Result<(), ServiceWorkerManagerError> {
        let Some(tab_id) = self
            .channels
            .pending_tab
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take()
        else {
            return Err(ServiceWorkerManagerError::Runtime(
                "Service Worker renderer host tab is unknown".into(),
            ));
        };
        self.channels.push_outgoing(ServiceWorkerHostOutgoing {
            tab_id,
            params: ServiceWorkerHostCommandParams {
                registration_id,
                command: ServiceWorkerHostCommand::Evaluate {
                    script_url: script_url.to_string(),
                    script: script.to_string(),
                },
            },
        });
        self.channels.record_owned(registration_id, tab_id);
        Ok(())
    }

    fn dispatch_lifecycle(
        &mut self,
        registration_id: u64,
        phase: ServiceWorkerLifecyclePhase,
    ) -> Result<(), ServiceWorkerManagerError> {
        let Some(tab_id) = self.channels.take_owned_tab(registration_id) else {
            return Err(ServiceWorkerManagerError::UnknownRegistration(registration_id));
        };
        self.channels.record_owned(registration_id, tab_id);
        self.channels.push_outgoing(ServiceWorkerHostOutgoing {
            tab_id,
            params: ServiceWorkerHostCommandParams {
                registration_id,
                command: ServiceWorkerHostCommand::DispatchLifecycle {
                    phase: wire_phase(phase),
                },
            },
        });
        Ok(())
    }

    fn dispatch_client_message(
        &mut self,
        registration_id: u64,
        event_id: u64,
        data_json: &str,
        client_id: &str,
        client_url: &str,
    ) -> Result<(), ServiceWorkerManagerError> {
        let Some(tab_id) = self.channels.take_owned_tab(registration_id) else {
            return Err(ServiceWorkerManagerError::UnknownRegistration(registration_id));
        };
        self.channels.record_owned(registration_id, tab_id);
        self.channels.push_outgoing(ServiceWorkerHostOutgoing {
            tab_id,
            params: ServiceWorkerHostCommandParams {
                registration_id,
                command: ServiceWorkerHostCommand::DispatchMessage {
                    event_id,
                    data_json: data_json.to_string(),
                    client_id: client_id.to_string(),
                    client_url: client_url.to_string(),
                },
            },
        });
        Ok(())
    }

    fn complete_import_scripts(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<Vec<String>, String>,
    ) -> Result<(), ServiceWorkerManagerError> {
        let Some(tab_id) = self.channels.take_owned_tab(registration_id) else {
            return Err(ServiceWorkerManagerError::UnknownRegistration(registration_id));
        };
        self.channels.record_owned(registration_id, tab_id);
        self.channels.push_outgoing(ServiceWorkerHostOutgoing {
            tab_id,
            params: ServiceWorkerHostCommandParams {
                registration_id,
                command: ServiceWorkerHostCommand::CompleteImportScripts { request_id, result },
            },
        });
        Ok(())
    }

    fn shutdown(&mut self, registration_id: u64) {
        if let Some(tab_id) = self.channels.remove_owned(registration_id) {
            self.channels.push_outgoing(ServiceWorkerHostOutgoing {
                tab_id,
                params: ServiceWorkerHostCommandParams {
                    registration_id,
                    command: ServiceWorkerHostCommand::Shutdown,
                },
            });
        }
    }

    fn poll_events(&mut self) -> Vec<(u64, ServiceWorkerEvent)> {
        std::mem::take(&mut *self.channels.inbox.lock().unwrap_or_else(|error| error.into_inner()))
    }

    fn runtime_count(&self) -> usize {
        self.channels.owned_count()
    }
}

/// profile 的 manager 构造方式。
#[derive(Clone, Copy)]
enum ProfileHostKind {
    /// 求值经 IPC 下放 renderer（生产；browser 主进程不链接 JS 引擎）。
    Ipc,
    /// runtime 在本进程求值（单测；引擎经 dev-dependency feature unification 引入）。
    Local,
}

fn profile_manager(kind: ProfileHostKind, channels: &SharedHostChannels) -> ServiceWorkerManager {
    match kind {
        ProfileHostKind::Ipc => ServiceWorkerManager::with_host(Box::new(IpcServiceWorkerHost::new(channels.clone()))),
        ProfileHostKind::Local => ServiceWorkerManager::new(),
    }
}

fn wire_phase(phase: ServiceWorkerLifecyclePhase) -> ServiceWorkerLifecycleWire {
    match phase {
        ServiceWorkerLifecyclePhase::Install => ServiceWorkerLifecycleWire::Install,
        ServiceWorkerLifecyclePhase::Activate => ServiceWorkerLifecycleWire::Activate,
    }
}

fn sandbox_phase(phase: ServiceWorkerLifecycleWire) -> ServiceWorkerLifecyclePhase {
    match phase {
        ServiceWorkerLifecycleWire::Install => ServiceWorkerLifecyclePhase::Install,
        ServiceWorkerLifecycleWire::Activate => ServiceWorkerLifecyclePhase::Activate,
    }
}

fn sandbox_event(event: ServiceWorkerHostEvent) -> ServiceWorkerEvent {
    match event {
        ServiceWorkerHostEvent::Evaluated { script_url } => ServiceWorkerEvent::Evaluated { script_url },
        ServiceWorkerHostEvent::ScriptError {
            script_url,
            kind,
            message,
        } => ServiceWorkerEvent::ScriptError {
            script_url,
            kind: match kind {
                ServiceWorkerScriptErrorKindWire::Compile => ServiceWorkerScriptErrorKind::Compile,
                ServiceWorkerScriptErrorKindWire::Runtime => ServiceWorkerScriptErrorKind::Runtime,
                ServiceWorkerScriptErrorKindWire::Timeout => ServiceWorkerScriptErrorKind::Timeout,
                ServiceWorkerScriptErrorKindWire::InvalidInput => ServiceWorkerScriptErrorKind::InvalidInput,
                ServiceWorkerScriptErrorKindWire::EngineUnavailable => ServiceWorkerScriptErrorKind::EngineUnavailable,
            },
            message,
        },
        ServiceWorkerHostEvent::LifecycleSettled {
            phase,
            succeeded,
            skip_waiting,
            claim_clients,
            message,
        } => ServiceWorkerEvent::LifecycleSettled {
            event_id: 0,
            phase: sandbox_phase(phase),
            succeeded,
            skip_waiting,
            claim_clients,
            message,
        },
        ServiceWorkerHostEvent::MessageDispatched {
            event_id,
            client_id,
            outbound,
        } => ServiceWorkerEvent::MessageDispatched {
            event_id,
            client_id,
            outbound: outbound
                .into_iter()
                .map(|data_json| zero_script_sandbox::ServiceWorkerOutboundMessage { data_json })
                .collect(),
        },
        ServiceWorkerHostEvent::MessageFailed {
            event_id,
            client_id,
            message,
        } => ServiceWorkerEvent::MessageFailed {
            event_id,
            client_id,
            message,
        },
        ServiceWorkerHostEvent::ImportScriptsRequested { request_id, specifiers } => {
            ServiceWorkerEvent::ImportScriptsRequested { request_id, specifiers }
        }
        ServiceWorkerHostEvent::Closed => ServiceWorkerEvent::Closed,
    }
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
}

/// A validated imported classic script batch owned by one blocked runtime.
pub(crate) struct ServiceWorkerImportFetchPlan {
    tab_id: TabId,
    profile: ProfileKey,
    registration_id: u64,
    request_id: u64,
    urls: Vec<String>,
    bypass_cache: bool,
}

impl ServiceWorkerImportFetchPlan {
    pub(crate) fn tab_id(&self) -> TabId {
        self.tab_id
    }

    pub(crate) fn urls(&self) -> &[String] {
        &self.urls
    }

    pub(crate) fn bypass_cache(&self) -> bool {
        self.bypass_cache
    }
}

struct PendingImportFetch {
    plan: ServiceWorkerImportFetchPlan,
    receivers: Vec<Option<Receiver<Result<HttpResponse, String>>>>,
    scripts: Vec<Option<ServiceWorkerImportedScript>>,
}

/// Browser-process single owner for Service Worker managers and runtimes.
pub(crate) struct BrowserServiceWorkerOwner {
    normal: ServiceWorkerManager,
    normal_channels: SharedHostChannels,
    private: HashMap<TabId, ServiceWorkerManager>,
    private_channels: HashMap<TabId, SharedHostChannels>,
    host_kind: ProfileHostKind,
    pending_fetches: Vec<PendingScriptFetch>,
    pending_evaluations: HashMap<(ProfileKey, u64), PendingEvaluation>,
    import_fetch_plans: Vec<ServiceWorkerImportFetchPlan>,
    pending_import_fetches: Vec<PendingImportFetch>,
    persistence_path: Option<PathBuf>,
    restoring: HashSet<u64>,
    /// IPC host 启动时无 renderer：持久化记录延迟到首个 renderer 接入时恢复。
    deferred_restores: Vec<ServiceWorkerPersistentRegistration>,
}

impl BrowserServiceWorkerOwner {
    /// 生产构造：脚本求值经 IPC 下放宿主 renderer（browser 主进程不链接 JS 引擎）。
    pub(crate) fn new() -> Self {
        Self::build(ProfileHostKind::Ipc, None)
    }

    /// 生产构造 + 持久化恢复（延迟到首个 renderer 接入，见 `flush_deferred_restores`）。
    pub(crate) fn with_persistence(path: PathBuf) -> Self {
        let mut owner = Self::build(ProfileHostKind::Ipc, Some(path.clone()));
        match load_persisted_service_workers(&path) {
            Ok(registrations) => {
                owner.deferred_restores = registrations;
            }
            Err(error) => {
                tracing::warn!("Service Worker persistence load failed: {error}");
            }
        }
        owner
    }

    /// 首个 renderer 接入时恢复持久化记录（求值下放该 renderer）。
    ///
    /// 由 process_backend 在 `ensure_renderer` 成功后调用；多次调用安全
    /// （队列为空时 no-op）。全部记录恢复失败时清理持久化文件（与启动期
    /// 恢复语义一致）。
    pub(crate) fn flush_deferred_restores(&mut self, tab_id: TabId) {
        if self.deferred_restores.is_empty() {
            return;
        }
        let registrations = std::mem::take(&mut self.deferred_restores);
        let had_records = !registrations.is_empty();
        for registration in registrations {
            self.normal_channels.set_pending_tab(tab_id);
            match self.normal.start_restored_active(registration) {
                Ok(registration_id) => {
                    self.restoring.insert(registration_id);
                }
                Err(error) => {
                    tracing::warn!("Service Worker restore skipped: {error}");
                }
            }
        }
        if had_records
            && self.restoring.is_empty()
            && let Err(error) = self.persist_normal()
        {
            tracing::warn!("Service Worker persistence cleanup failed: {error}");
        }
    }

    /// 测试构造：runtime 在本进程求值（引擎经 dev-dependency feature unification 引入）。
    pub(crate) fn with_local_hosts() -> Self {
        Self::build(ProfileHostKind::Local, None)
    }

    /// 测试构造 + 持久化恢复（本地 host 求值；drain 至恢复 settle——坏脚本经
    /// 异步 ScriptError 判失败并回写持久化文件，须轮询驱动）。
    #[cfg(test)]
    pub(crate) fn with_local_hosts_and_persistence(path: PathBuf) -> Self {
        let mut owner = Self::build(ProfileHostKind::Local, Some(path.clone()));
        let had_records = if let Ok(registrations) = load_persisted_service_workers(&path) {
            let had_records = !registrations.is_empty();
            for registration in registrations {
                match owner.normal.start_restored_active(registration) {
                    Ok(registration_id) => {
                        owner.restoring.insert(registration_id);
                    }
                    Err(error) => {
                        tracing::warn!("Service Worker restore skipped: {error}");
                    }
                }
            }
            had_records
        } else {
            false
        };
        if had_records
            && owner.restoring.is_empty()
            && let Err(error) = owner.persist_normal()
        {
            tracing::warn!("Service Worker persistence cleanup failed: {error}");
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

    fn build(host_kind: ProfileHostKind, persistence_path: Option<PathBuf>) -> Self {
        let normal_channels = SharedHostChannels::default();
        Self {
            normal: profile_manager(host_kind, &normal_channels),
            normal_channels,
            private: HashMap::new(),
            private_channels: HashMap::new(),
            host_kind,
            pending_fetches: Vec::new(),
            pending_evaluations: HashMap::new(),
            import_fetch_plans: Vec::new(),
            pending_import_fetches: Vec::new(),
            persistence_path,
            restoring: HashSet::new(),
            deferred_restores: Vec::new(),
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

    pub(crate) fn take_import_fetch_plans(&mut self) -> Vec<ServiceWorkerImportFetchPlan> {
        std::mem::take(&mut self.import_fetch_plans)
    }

    pub(crate) fn attach_import_fetches(
        &mut self,
        plan: ServiceWorkerImportFetchPlan,
        receivers: Vec<Receiver<Result<HttpResponse, String>>>,
    ) {
        let script_count = plan.urls.len();
        if receivers.len() != script_count {
            let _ = self.manager_mut(plan.profile).complete_import_scripts(
                plan.registration_id,
                plan.request_id,
                Err("Service Worker import fetch count mismatch".into()),
            );
            return;
        }
        self.pending_import_fetches.push(PendingImportFetch {
            plan,
            receivers: receivers.into_iter().map(Some).collect(),
            scripts: vec![None; script_count],
        });
    }

    pub(crate) fn poll(&mut self) -> Vec<CompletedServiceWorkerResponse> {
        let mut completed = Vec::new();
        self.poll_fetches(&mut completed);
        self.poll_import_fetches();
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
        self.fail_tab_hosted_runtimes(tab_id);
    }

    pub(crate) fn disconnect_tab(&mut self, tab_id: TabId) {
        self.pending_fetches.retain(|pending| pending.plan.tab_id != tab_id);
        self.pending_evaluations.retain(|_, pending| pending.tab_id != tab_id);
        let mut retained_plans = Vec::new();
        for plan in std::mem::take(&mut self.import_fetch_plans) {
            if plan.tab_id == tab_id {
                let _ = self.manager_mut(plan.profile).complete_import_scripts(
                    plan.registration_id,
                    plan.request_id,
                    Err("Service Worker import fetch client disconnected".into()),
                );
            } else {
                retained_plans.push(plan);
            }
        }
        self.import_fetch_plans = retained_plans;
        let mut retained = Vec::new();
        for pending in std::mem::take(&mut self.pending_import_fetches) {
            if pending.plan.tab_id == tab_id {
                let _ = self.manager_mut(pending.plan.profile).complete_import_scripts(
                    pending.plan.registration_id,
                    pending.plan.request_id,
                    Err("Service Worker import fetch client disconnected".into()),
                );
            } else {
                retained.push(pending);
            }
        }
        self.pending_import_fetches = retained;
    }

    pub(crate) fn remove_private_profile(&mut self, tab_id: TabId) {
        self.private.remove(&tab_id);
        self.pending_fetches
            .retain(|pending| pending.plan.profile != ProfileKey::Private(tab_id));
        self.pending_evaluations
            .retain(|(profile, _), _| *profile != ProfileKey::Private(tab_id));
        self.import_fetch_plans
            .retain(|plan| plan.profile != ProfileKey::Private(tab_id));
        self.pending_import_fetches
            .retain(|pending| pending.plan.profile != ProfileKey::Private(tab_id));
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

    fn poll_import_fetches(&mut self) {
        let mut retained = Vec::new();
        for mut pending in std::mem::take(&mut self.pending_import_fetches) {
            let origin = self
                .manager(pending.plan.profile)
                .and_then(|manager| manager.registration(pending.plan.registration_id))
                .map(|registration| registration.origin.clone());
            let Some(origin) = origin else {
                continue;
            };
            let mut failure = None;
            for index in 0..pending.receivers.len() {
                let Some(receiver) = pending.receivers[index].as_ref() else {
                    continue;
                };
                match receiver.try_recv() {
                    Ok(result) => {
                        pending.receivers[index] = None;
                        match validate_imported_script_response(&pending.plan.urls[index], &origin, result) {
                            Ok(script) => pending.scripts[index] = Some(script),
                            Err(error) => {
                                failure = Some(error);
                                break;
                            }
                        }
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => {
                        failure = Some("Service Worker import fetch worker exited".into());
                        break;
                    }
                }
            }
            if let Some(error) = failure {
                let _ = self.manager_mut(pending.plan.profile).complete_import_scripts(
                    pending.plan.registration_id,
                    pending.plan.request_id,
                    Err(error),
                );
            } else if pending.scripts.iter().all(Option::is_some) {
                let scripts = pending.scripts.into_iter().flatten().collect();
                let _ = self.manager_mut(pending.plan.profile).complete_import_scripts(
                    pending.plan.registration_id,
                    pending.plan.request_id,
                    Ok(scripts),
                );
            } else {
                retained.push(pending);
            }
        }
        self.pending_import_fetches = retained;
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
        // IPC host 需在 manager 分配 registration 前知道宿主 renderer tab——
        // `evaluate` 在 `start_evaluation` / `start_update` 内同步消费该一次性槽位。
        self.ensure_profile(plan.profile);
        if let Some(channels) = self.channels_for(plan.profile) {
            channels.set_pending_tab(plan.tab_id);
        }
        let result = match plan.purpose {
            ServiceWorkerFetchPurpose::Register { update_via_cache } => {
                self.manager_mut(plan.profile).start_evaluation_with_update_via_cache(
                    plan.script_url.as_str(),
                    plan.scope.as_str(),
                    &plan.origin,
                    &script,
                    update_via_cache,
                )
            }
            ServiceWorkerFetchPurpose::Update { registration_id, .. } => {
                match self.manager_mut(plan.profile).start_update(registration_id, &script) {
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
                    Ok(ServiceWorkerUpdateOutcome::Started { registration_id }) => Ok(registration_id),
                    Err(error) => Err(error),
                }
            }
        };
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
        let mut persistence_dirty = false;
        for event in events {
            match event {
                ServiceWorkerManagerEvent::ImportScriptsRequested {
                    registration_id,
                    request_id,
                    urls,
                    bypass_cache,
                } => {
                    let pending_tab = self
                        .pending_evaluations
                        .get(&(profile, registration_id))
                        .map(|pending| pending.tab_id);
                    let owned_tab = self
                        .channels_for(profile)
                        .and_then(|channels| channels.take_owned_tab(registration_id));
                    let tab_id = pending_tab.or(owned_tab);
                    if let Some(tab_id) = tab_id {
                        if let Some(channels) = self.channels_for(profile) {
                            channels.record_owned(registration_id, tab_id);
                        }
                        self.import_fetch_plans.push(ServiceWorkerImportFetchPlan {
                            tab_id,
                            profile,
                            registration_id,
                            request_id,
                            urls,
                            bypass_cache,
                        });
                    } else {
                        let _ = self.manager_mut(profile).complete_import_scripts(
                            registration_id,
                            request_id,
                            Err("Service Worker import fetch has no renderer host".into()),
                        );
                    }
                }
                ServiceWorkerManagerEvent::UpdateChecked {
                    candidate_registration_id,
                    registration_id,
                    changed,
                } => {
                    if let Some(pending) = self.pending_evaluations.remove(&(profile, candidate_registration_id)) {
                        completed.push(success_response(
                            pending.tab_id,
                            pending.request_id,
                            ServiceWorkerResult::Updated {
                                registration_id,
                                changed,
                            },
                        ));
                    }
                }
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
        self.ensure_profile(profile);
        match profile {
            ProfileKey::Normal => &mut self.normal,
            ProfileKey::Private(tab_id) => self.private.get_mut(&tab_id).expect("private manager ensured"),
        }
    }

    fn ensure_profile(&mut self, profile: ProfileKey) {
        if let ProfileKey::Private(tab_id) = profile
            && !self.private.contains_key(&tab_id)
        {
            let channels = SharedHostChannels::default();
            let manager = profile_manager(self.host_kind, &channels);
            self.private.insert(tab_id, manager);
            self.private_channels.insert(tab_id, channels);
        }
    }

    fn channels_for(&self, profile: ProfileKey) -> Option<&SharedHostChannels> {
        match profile {
            ProfileKey::Normal => Some(&self.normal_channels),
            ProfileKey::Private(tab_id) => self.private_channels.get(&tab_id),
        }
    }

    /// 注入 renderer 回传的 runtime 事件（process_backend 消息循环调用）。
    pub(crate) fn inject_host_event(&mut self, tab_id: TabId, private: bool, params: ServiceWorkerHostEventParams) {
        let profile = if private {
            ProfileKey::Private(tab_id)
        } else {
            ProfileKey::Normal
        };
        if let Some(channels) = self.channels_for(profile) {
            channels.push_event(params.registration_id, sandbox_event(params.event));
        }
    }

    /// 取出待下发宿主 renderer 的托管命令（process_backend 轮询后发送）。
    pub(crate) fn take_host_commands(&mut self) -> Vec<ServiceWorkerHostOutgoing> {
        let mut commands = self.normal_channels.take_outgoing();
        for channels in self.private_channels.values() {
            commands.extend(channels.take_outgoing());
        }
        commands
    }

    /// renderer 进程死亡/关闭：该 tab 托管的 runtime 注入 Closed，manager 据此把
    /// installing 版本判失败（active 版本状态保持——fetch 拦截接线为后续切片）。
    pub(crate) fn fail_tab_hosted_runtimes(&mut self, tab_id: TabId) {
        let mut failed: Vec<(ProfileKey, Vec<u64>)> = Vec::new();
        let normal_ids = self.normal_channels.remove_owned_by_tab(tab_id);
        if !normal_ids.is_empty() {
            failed.push((ProfileKey::Normal, normal_ids));
        }
        if let Some(channels) = self.private_channels.get(&tab_id) {
            let private_ids = channels.remove_owned_by_tab(tab_id);
            if !private_ids.is_empty() {
                failed.push((ProfileKey::Private(tab_id), private_ids));
            }
        }
        for (profile, ids) in failed {
            if let Some(channels) = self.channels_for(profile) {
                for registration_id in ids {
                    channels.push_event(registration_id, ServiceWorkerEvent::Closed);
                }
            }
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
        if registration.script_source.len() > MAX_SCRIPT_BYTES {
            return Err("state main script exceeds the size limit".into());
        }
        total_script_bytes = total_script_bytes
            .checked_add(registration.script_source.len())
            .ok_or_else(|| "state script size overflow".to_string())?;
        if registration.imported_scripts.len() > MAX_PERSISTED_IMPORTS_PER_REGISTRATION {
            return Err("state registration has too many imported scripts".into());
        }
        let mut imported_urls = HashSet::new();
        for imported in &registration.imported_scripts {
            total_script_bytes = total_script_bytes
                .checked_add(imported.source.len())
                .ok_or_else(|| "state script size overflow".to_string())?;
            if imported.source.len() > MAX_SCRIPT_BYTES {
                return Err("state imported script exceeds the size limit".into());
            }
            let url = Url::parse(&imported.url).map_err(|_| "state imported script URL is invalid".to_string())?;
            if url.as_str() != imported.url
                || url.fragment().is_some()
                || !matches!(url.scheme(), "http" | "https" | "data")
                || !url.username().is_empty()
                || url.password().is_some()
            {
                return Err("state imported script URL is not canonical".into());
            }
            if !imported_urls.insert(imported.url.as_str()) {
                return Err("state contains duplicate imported script URLs".into());
            }
        }
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

fn validate_imported_script_response(
    requested_url: &str,
    registration_origin: &str,
    result: Result<HttpResponse, String>,
) -> Result<ServiceWorkerImportedScript, String> {
    let response = result.map_err(|message| format!("Service Worker import fetch failed: {message}"))?;
    if !response.is_success() {
        return Err(format!(
            "Service Worker import fetch returned HTTP {}",
            response.status_code
        ));
    }
    if response.body.len() > MAX_SCRIPT_BYTES {
        return Err("Service Worker imported script exceeds the size limit".into());
    }
    let final_url =
        Url::parse(&response.url).map_err(|_| "Service Worker import fetch returned an invalid final URL")?;
    if !matches!(final_url.scheme(), "http" | "https" | "data")
        || !final_url.username().is_empty()
        || final_url.password().is_some()
    {
        return Err("Service Worker import fetch returned a disallowed final URL".into());
    }
    if Url::parse(registration_origin).is_ok_and(|origin| origin.scheme() == "https") && final_url.scheme() == "http" {
        return Err("Service Worker import redirect downgraded a secure context".into());
    }
    let mime = response
        .content_type_mime()
        .ok_or_else(|| "Service Worker imported script has no JavaScript MIME type".to_string())?;
    if !is_javascript_mime(mime) {
        return Err(format!(
            "Service Worker imported script has unsupported MIME type {mime}"
        ));
    }
    let source = String::from_utf8(response.body).map_err(|_| "Service Worker imported script is not valid UTF-8")?;
    Ok(ServiceWorkerImportedScript {
        url: requested_url.to_string(),
        source,
    })
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
