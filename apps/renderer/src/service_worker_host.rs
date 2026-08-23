//! Renderer-side Service Worker runtime host.
//!
//! Browser owner 把 SW 脚本求值下放到 renderer 进程（browser 主进程不链接
//! JS 引擎）。本模块按 browser 分配的 registration id 托管 script-sandbox
//! [`ServiceWorkerRuntime`] 线程，并把 runtime 事件转成 wire 消息回传。
//!
//! 托管运行在**独立线程**：命令由 IPC reader 线程直接投递（不经 renderer
//! 主循环——主循环可能被同步 automation 请求长期占住，而页面 JS 的
//! `navigator.serviceWorker.register` 正在等这里的求值结果，经主循环会
//! 互等死锁）；事件由本线程直接经共享 writer 写回 browser。

use std::collections::HashMap;
use std::io;
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::time::Duration;

use zero_protocol::message::{
    ServiceWorkerCacheQueryOptionsWire, ServiceWorkerCacheStorageRequestWire, ServiceWorkerCacheStorageResultWire,
    ServiceWorkerFetchRequestWire, ServiceWorkerFetchResponseWire, ServiceWorkerHostCommand,
    ServiceWorkerHostCommandParams, ServiceWorkerHostEvent, ServiceWorkerHostEventParams, ServiceWorkerLifecycleWire,
    ServiceWorkerScriptErrorKindWire, ServiceWorkerScriptTypeWire,
};
use zero_protocol::transport::PipeTransport;
use zero_protocol::{IpcChannel, IpcMessage, IpcMessageKind};
use zero_script_sandbox::{
    SandboxConfig, ServiceWorkerCacheQueryOptions, ServiceWorkerCacheStorageRequest, ServiceWorkerCacheStorageResult,
    ServiceWorkerClientInfo, ServiceWorkerEvent, ServiceWorkerFetchRequest, ServiceWorkerFetchResponse,
    ServiceWorkerLifecyclePhase, ServiceWorkerMessagePorts, ServiceWorkerRuntime, ServiceWorkerScriptErrorKind,
};

use crate::compositor_publish_thread::SharedWriter;

/// 命令通道闲置时的轮询间隔（仅影响 runtime 事件回传延迟）。
const COMMAND_POLL_INTERVAL: Duration = Duration::from_millis(8);

/// 按注册版本托管 SW runtime 线程的 renderer 侧宿主（命令投递端句柄）。
pub(crate) struct RendererServiceWorkerHost {
    commands: Option<Sender<ServiceWorkerHostCommandParams>>,
}

impl RendererServiceWorkerHost {
    /// 启动托管线程并返回投递句柄。
    pub(crate) fn new(outbound: SharedWriter) -> Self {
        let (commands, receiver) = mpsc::channel();
        std::thread::Builder::new()
            .name("sw-runtime-host".into())
            .spawn(move || HostThread::run(receiver, outbound))
            .expect("spawn Service Worker runtime host thread");
        Self {
            commands: Some(commands),
        }
    }

    /// 处理 browser 下发的托管命令（非阻塞；实际执行在托管线程）。
    pub(crate) fn handle_command(&self, params: ServiceWorkerHostCommandParams) {
        if let Some(commands) = &self.commands
            && commands.send(params).is_err()
        {
            tracing::warn!("Service Worker host thread has exited; command dropped");
        }
    }
}

impl Drop for RendererServiceWorkerHost {
    fn drop(&mut self) {
        // Drop 关闭命令通道，托管线程 drain 完队列后退出（runtimes Drop 时停引擎线程）。
        self.commands = None;
    }
}

/// 托管线程主体：消费命令 + drain runtime 事件 + 回传。
struct HostThread {
    runtimes: HashMap<u64, ServiceWorkerRuntime>,
    pending_events: Vec<ServiceWorkerHostEventParams>,
    outbound: PipeTransport<io::Empty, SharedWriter>,
}

impl HostThread {
    fn run(receiver: mpsc::Receiver<ServiceWorkerHostCommandParams>, outbound: SharedWriter) {
        let mut host = Self {
            runtimes: HashMap::new(),
            pending_events: Vec::new(),
            outbound: PipeTransport::new(io::empty(), outbound),
        };
        loop {
            loop {
                match receiver.try_recv() {
                    Ok(params) => host.handle_command(params),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return,
                }
            }
            host.tick();
            // 无新命令时阻塞等待，避免空转；等待期间到达的命令同样处理（不可丢弃）。
            if let Ok(params) = receiver.recv_timeout(COMMAND_POLL_INTERVAL) {
                host.handle_command(params);
            }
        }
    }

    fn handle_command(&mut self, params: ServiceWorkerHostCommandParams) {
        if let Err(message) = params.validate() {
            tracing::warn!("invalid Service Worker host command: {message}");
            return;
        }
        match params.command {
            ServiceWorkerHostCommand::Evaluate {
                script_url,
                script,
                script_type,
            } => {
                self.evaluate(params.registration_id, &script_url, &script, script_type);
            }
            ServiceWorkerHostCommand::DispatchLifecycle { phase } => {
                self.dispatch_lifecycle(params.registration_id, wire_phase(phase));
            }
            ServiceWorkerHostCommand::DispatchMessage {
                event_id,
                data_json,
                client_id,
                client_url,
                transferred_port_ids,
                data_port_index,
                target_port_id,
            } => self.dispatch_message(
                params.registration_id,
                event_id,
                &data_json,
                &client_id,
                &client_url,
                &ServiceWorkerMessagePorts {
                    transferred_port_ids,
                    data_port_index,
                    target_port_id,
                },
            ),
            ServiceWorkerHostCommand::DispatchFetch { event_id, request } => {
                self.dispatch_fetch(params.registration_id, event_id, fetch_request_from_wire(request));
            }
            ServiceWorkerHostCommand::CompleteImportScripts { request_id, result } => {
                self.complete_import_scripts(params.registration_id, request_id, result);
            }
            ServiceWorkerHostCommand::CompleteUpdate { request_id, result } => {
                self.complete_update(
                    params.registration_id,
                    request_id,
                    result.map_err(|error| (error.exception_name, error.message)),
                );
            }
            ServiceWorkerHostCommand::CompleteClientsMatchAll { request_id, result } => {
                self.complete_clients_match_all(
                    params.registration_id,
                    request_id,
                    result.map(|clients| {
                        clients
                            .into_iter()
                            .map(|client| ServiceWorkerClientInfo {
                                id: client.id,
                                url: client.url,
                                client_type: client.client_type,
                                frame_type: client.frame_type,
                                visibility_state: client.visibility_state,
                                focused: client.focused,
                            })
                            .collect()
                    }),
                );
            }
            ServiceWorkerHostCommand::CompleteClientsGet { request_id, result } => {
                self.complete_clients_get(
                    params.registration_id,
                    request_id,
                    result.map(|client| {
                        client.map(|client| ServiceWorkerClientInfo {
                            id: client.id,
                            url: client.url,
                            client_type: client.client_type,
                            frame_type: client.frame_type,
                            visibility_state: client.visibility_state,
                            focused: client.focused,
                        })
                    }),
                );
            }
            ServiceWorkerHostCommand::CompleteCacheStorage { request_id, result } => {
                self.complete_cache_storage(
                    params.registration_id,
                    request_id,
                    result.map(cache_storage_result_from_wire),
                );
            }
            ServiceWorkerHostCommand::CompleteFetch { request_id, result } => {
                self.complete_fetch(params.registration_id, request_id, result.map(fetch_response_from_wire));
            }
            ServiceWorkerHostCommand::Shutdown => {
                // 不在此处移除：tick 先 drain `Closed` 事件再按 is_running 回收槽位。
                if let Some(runtime) = self.runtimes.get_mut(&params.registration_id) {
                    runtime.shutdown();
                }
            }
        }
    }

    /// Drain runtime 事件并回传 browser（`Closed` 同时回收 runtime 槽位）。
    fn tick(&mut self) {
        let mut output = std::mem::take(&mut self.pending_events);
        for (&registration_id, runtime) in &self.runtimes {
            while let Some(event) = runtime.try_recv() {
                output.push(ServiceWorkerHostEventParams {
                    registration_id,
                    event: host_event(event),
                });
            }
        }
        self.runtimes.retain(|_, runtime| runtime.is_running());
        for event in output {
            let message = IpcMessage {
                id: event.registration_id,
                kind: IpcMessageKind::ServiceWorkerHostEvent(event),
            };
            if let Err(error) = self.outbound.send(message) {
                tracing::warn!("Service Worker host event send failed: {error}");
            }
        }
    }

    fn evaluate(
        &mut self,
        registration_id: u64,
        script_url: &str,
        script: &str,
        script_type: ServiceWorkerScriptTypeWire,
    ) {
        // 同 id 重复求值不应发生（browser 分配唯一 id）；防御性先回收旧 runtime。
        if let Some(mut runtime) = self.runtimes.remove(&registration_id) {
            runtime.shutdown();
        }
        match ServiceWorkerRuntime::new(SandboxConfig::default()) {
            Ok(mut runtime) => {
                let evaluation = match script_type {
                    ServiceWorkerScriptTypeWire::Classic => runtime.evaluate(script, script_url),
                    ServiceWorkerScriptTypeWire::Module => runtime.evaluate_module(script, script_url),
                };
                if let Err(error) = evaluation {
                    tracing::warn!("Service Worker evaluate queue failed: {error}");
                }
                self.runtimes.insert(registration_id, runtime);
            }
            Err(error) => {
                // 引擎初始化失败：合成 ScriptError 回传，browser 端据此判 installing 版本失败。
                tracing::warn!("Service Worker runtime spawn failed: {error}");
                self.pending_events.push(ServiceWorkerHostEventParams {
                    registration_id,
                    event: ServiceWorkerHostEvent::ScriptError {
                        script_url: script_url.to_string(),
                        kind: ServiceWorkerScriptErrorKindWire::EngineUnavailable,
                        message: error.to_string(),
                    },
                });
            }
        }
    }

    fn dispatch_message(
        &mut self,
        registration_id: u64,
        event_id: u64,
        data_json: &str,
        client_id: &str,
        client_url: &str,
        ports: &ServiceWorkerMessagePorts,
    ) {
        let Some(runtime) = self.runtimes.get_mut(&registration_id) else {
            tracing::warn!("Service Worker message for unknown registration {registration_id}");
            return;
        };
        if let Err(error) = runtime.dispatch_message_with_ports(event_id, data_json, client_id, client_url, ports) {
            tracing::warn!("Service Worker message dispatch failed: {error}");
        }
    }

    fn dispatch_fetch(&mut self, registration_id: u64, event_id: u64, request: ServiceWorkerFetchRequest) {
        let Some(runtime) = self.runtimes.get_mut(&registration_id) else {
            tracing::warn!("Service Worker fetch for unknown registration {registration_id}");
            return;
        };
        if let Err(error) = runtime.dispatch_fetch(event_id, request) {
            tracing::warn!("Service Worker fetch dispatch failed: {error}");
        }
    }

    fn dispatch_lifecycle(&mut self, registration_id: u64, phase: ServiceWorkerLifecyclePhase) {
        let Some(runtime) = self.runtimes.get_mut(&registration_id) else {
            tracing::warn!("Service Worker lifecycle for unknown registration {registration_id}");
            return;
        };
        let result = match phase {
            ServiceWorkerLifecyclePhase::Install => runtime.dispatch_install(registration_id),
            ServiceWorkerLifecyclePhase::Activate => runtime.dispatch_activate(registration_id),
        };
        if let Err(error) = result {
            tracing::warn!("Service Worker lifecycle dispatch failed: {error}");
        }
    }

    fn complete_import_scripts(&mut self, registration_id: u64, request_id: u64, result: Result<Vec<String>, String>) {
        let Some(runtime) = self.runtimes.get(&registration_id) else {
            tracing::warn!("Service Worker import response for unknown registration {registration_id}");
            return;
        };
        if let Err(error) = runtime.complete_import_scripts(request_id, result) {
            tracing::warn!("Service Worker import response failed: {error}");
        }
    }

    fn complete_update(&mut self, registration_id: u64, request_id: u64, result: Result<(), (String, String)>) {
        let Some(runtime) = self.runtimes.get(&registration_id) else {
            tracing::warn!("Service Worker update response for unknown registration {registration_id}");
            return;
        };
        if let Err(error) = runtime.complete_update(request_id, result) {
            tracing::warn!("Service Worker update response failed: {error}");
        }
    }

    fn complete_clients_match_all(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<Vec<ServiceWorkerClientInfo>, String>,
    ) {
        let Some(runtime) = self.runtimes.get(&registration_id) else {
            tracing::warn!("Service Worker clients response for unknown registration {registration_id}");
            return;
        };
        if let Err(error) = runtime.complete_clients_match_all(request_id, result) {
            tracing::warn!("Service Worker clients response failed: {error}");
        }
    }

    fn complete_clients_get(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<Option<ServiceWorkerClientInfo>, String>,
    ) {
        let Some(runtime) = self.runtimes.get(&registration_id) else {
            tracing::warn!("Service Worker clients response for unknown registration {registration_id}");
            return;
        };
        if let Err(error) = runtime.complete_clients_get(request_id, result) {
            tracing::warn!("Service Worker clients response failed: {error}");
        }
    }

    fn complete_cache_storage(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<ServiceWorkerCacheStorageResult, String>,
    ) {
        let Some(runtime) = self.runtimes.get(&registration_id) else {
            tracing::warn!("Service Worker cache response for unknown registration {registration_id}");
            return;
        };
        if let Err(error) = runtime.complete_cache_storage(request_id, result) {
            tracing::warn!("Service Worker cache response failed: {error}");
        }
    }

    fn complete_fetch(
        &mut self,
        registration_id: u64,
        request_id: u64,
        result: Result<ServiceWorkerFetchResponse, String>,
    ) {
        let Some(runtime) = self.runtimes.get(&registration_id) else {
            tracing::warn!("Service Worker fetch response for unknown registration {registration_id}");
            return;
        };
        if let Err(error) = runtime.complete_fetch(request_id, result) {
            tracing::warn!("Service Worker fetch response failed: {error}");
        }
    }
}

fn wire_phase(phase: ServiceWorkerLifecycleWire) -> ServiceWorkerLifecyclePhase {
    match phase {
        ServiceWorkerLifecycleWire::Install => ServiceWorkerLifecyclePhase::Install,
        ServiceWorkerLifecycleWire::Activate => ServiceWorkerLifecyclePhase::Activate,
    }
}

fn fetch_request_from_wire(request: ServiceWorkerFetchRequestWire) -> ServiceWorkerFetchRequest {
    ServiceWorkerFetchRequest {
        url: request.url,
        method: request.method,
        headers: request.headers,
        body: request.body,
        credentials: request.credentials,
        client_id: request.client_id,
        resulting_client_id: request.resulting_client_id,
        referrer: request.referrer,
    }
}

fn fetch_response_from_wire(response: ServiceWorkerFetchResponseWire) -> ServiceWorkerFetchResponse {
    ServiceWorkerFetchResponse {
        status: response.status,
        status_text: response.status_text,
        response_type: response.response_type,
        headers: response.headers,
        body: response.body,
    }
}

fn cache_storage_request_to_wire(request: ServiceWorkerCacheStorageRequest) -> ServiceWorkerCacheStorageRequestWire {
    match request {
        ServiceWorkerCacheStorageRequest::Open { cache_name } => {
            ServiceWorkerCacheStorageRequestWire::Open { cache_name }
        }
        ServiceWorkerCacheStorageRequest::Match {
            cache_name,
            cache_id,
            request,
            options,
        } => ServiceWorkerCacheStorageRequestWire::Match {
            cache_name,
            cache_id,
            request: ServiceWorkerFetchRequestWire {
                url: request.url,
                method: request.method,
                headers: request.headers,
                body: request.body,
                credentials: request.credentials,
                client_id: request.client_id,
                resulting_client_id: request.resulting_client_id,
                referrer: request.referrer,
            },
            options: cache_query_options_to_wire(options),
        },
        ServiceWorkerCacheStorageRequest::MatchAll {
            cache_name,
            cache_id,
            request,
            options,
        } => ServiceWorkerCacheStorageRequestWire::MatchAll {
            cache_name,
            cache_id,
            request: request.map(|request| ServiceWorkerFetchRequestWire {
                url: request.url,
                method: request.method,
                headers: request.headers,
                body: request.body,
                credentials: request.credentials,
                client_id: request.client_id,
                resulting_client_id: request.resulting_client_id,
                referrer: request.referrer,
            }),
            options: cache_query_options_to_wire(options),
        },
        ServiceWorkerCacheStorageRequest::Keys {
            cache_name,
            cache_id,
            request,
            options,
        } => ServiceWorkerCacheStorageRequestWire::Keys {
            cache_name,
            cache_id,
            request: request.map(|request| ServiceWorkerFetchRequestWire {
                url: request.url,
                method: request.method,
                headers: request.headers,
                body: request.body,
                credentials: request.credentials,
                client_id: request.client_id,
                resulting_client_id: request.resulting_client_id,
                referrer: request.referrer,
            }),
            options: cache_query_options_to_wire(options),
        },
        ServiceWorkerCacheStorageRequest::Delete {
            cache_name,
            cache_id,
            request,
            options,
        } => ServiceWorkerCacheStorageRequestWire::Delete {
            cache_name,
            cache_id,
            request: ServiceWorkerFetchRequestWire {
                url: request.url,
                method: request.method,
                headers: request.headers,
                body: request.body,
                credentials: request.credentials,
                client_id: request.client_id,
                resulting_client_id: request.resulting_client_id,
                referrer: request.referrer,
            },
            options: cache_query_options_to_wire(options),
        },
        ServiceWorkerCacheStorageRequest::Put {
            cache_name,
            cache_id,
            request,
            response,
        } => ServiceWorkerCacheStorageRequestWire::Put {
            cache_name,
            cache_id,
            request: ServiceWorkerFetchRequestWire {
                url: request.url,
                method: request.method,
                headers: request.headers,
                body: request.body,
                credentials: request.credentials,
                client_id: request.client_id,
                resulting_client_id: request.resulting_client_id,
                referrer: request.referrer,
            },
            response: ServiceWorkerFetchResponseWire {
                status: response.status,
                status_text: response.status_text,
                response_type: response.response_type,
                headers: response.headers,
                body: response.body,
            },
        },
        ServiceWorkerCacheStorageRequest::StorageHas { cache_name } => {
            ServiceWorkerCacheStorageRequestWire::StorageHas { cache_name }
        }
        ServiceWorkerCacheStorageRequest::StorageDelete { cache_name } => {
            ServiceWorkerCacheStorageRequestWire::StorageDelete { cache_name }
        }
        ServiceWorkerCacheStorageRequest::StorageKeys => ServiceWorkerCacheStorageRequestWire::StorageKeys,
    }
}

fn cache_query_options_to_wire(options: ServiceWorkerCacheQueryOptions) -> ServiceWorkerCacheQueryOptionsWire {
    ServiceWorkerCacheQueryOptionsWire {
        ignore_search: options.ignore_search,
        ignore_method: options.ignore_method,
        ignore_vary: options.ignore_vary,
    }
}

fn cache_storage_result_from_wire(result: ServiceWorkerCacheStorageResultWire) -> ServiceWorkerCacheStorageResult {
    match result {
        ServiceWorkerCacheStorageResultWire::Done => ServiceWorkerCacheStorageResult::Done,
        ServiceWorkerCacheStorageResultWire::Open {
            cache_name,
            cache_name_units,
            cache_id,
        } => ServiceWorkerCacheStorageResult::Open {
            cache_name,
            cache_name_units,
            cache_id,
        },
        ServiceWorkerCacheStorageResultWire::Match(response) => {
            ServiceWorkerCacheStorageResult::Match(response.map(fetch_response_from_wire))
        }
        ServiceWorkerCacheStorageResultWire::MatchAll(responses) => {
            ServiceWorkerCacheStorageResult::MatchAll(responses.into_iter().map(fetch_response_from_wire).collect())
        }
        ServiceWorkerCacheStorageResultWire::Keys(requests) => {
            ServiceWorkerCacheStorageResult::Keys(requests.into_iter().map(fetch_request_from_wire).collect())
        }
        ServiceWorkerCacheStorageResultWire::Bool(value) => ServiceWorkerCacheStorageResult::Bool(value),
        ServiceWorkerCacheStorageResultWire::StorageKeys(cache_names) => {
            ServiceWorkerCacheStorageResult::StorageKeys(cache_names)
        }
    }
}

fn host_event(event: ServiceWorkerEvent) -> ServiceWorkerHostEvent {
    match event {
        ServiceWorkerEvent::Evaluated { script_url } => ServiceWorkerHostEvent::Evaluated { script_url },
        ServiceWorkerEvent::ScriptError {
            script_url,
            kind,
            message,
        } => ServiceWorkerHostEvent::ScriptError {
            script_url,
            kind: match kind {
                ServiceWorkerScriptErrorKind::Compile => ServiceWorkerScriptErrorKindWire::Compile,
                ServiceWorkerScriptErrorKind::Runtime => ServiceWorkerScriptErrorKindWire::Runtime,
                ServiceWorkerScriptErrorKind::Timeout => ServiceWorkerScriptErrorKindWire::Timeout,
                ServiceWorkerScriptErrorKind::InvalidInput => ServiceWorkerScriptErrorKindWire::InvalidInput,
                ServiceWorkerScriptErrorKind::EngineUnavailable => ServiceWorkerScriptErrorKindWire::EngineUnavailable,
            },
            message,
        },
        ServiceWorkerEvent::LifecycleSettled {
            phase,
            succeeded,
            skip_waiting,
            claim_clients,
            message,
            ..
        } => ServiceWorkerHostEvent::LifecycleSettled {
            phase: match phase {
                ServiceWorkerLifecyclePhase::Install => ServiceWorkerLifecycleWire::Install,
                ServiceWorkerLifecyclePhase::Activate => ServiceWorkerLifecycleWire::Activate,
            },
            succeeded,
            skip_waiting,
            claim_clients,
            message,
        },
        ServiceWorkerEvent::MessageDispatched {
            event_id,
            client_id,
            outbound,
        } => ServiceWorkerHostEvent::MessageDispatched {
            event_id,
            client_id,
            outbound: outbound
                .into_iter()
                .map(|message| zero_protocol::ServiceWorkerMessage {
                    data_json: message.data_json,
                    port_id: message.port_id,
                    transferred_port_ids: message.transferred_port_ids,
                    data_port_index: message.data_port_index,
                    target_client_id: message.target_client_id,
                })
                .collect(),
        },
        ServiceWorkerEvent::MessageFailed {
            event_id,
            client_id,
            message,
        } => ServiceWorkerHostEvent::MessageFailed {
            event_id,
            client_id,
            message,
        },
        ServiceWorkerEvent::FetchSettled {
            event_id,
            request_url,
            response,
            failed,
            message,
        } => ServiceWorkerHostEvent::FetchSettled {
            event_id,
            request_url,
            response: response.map(|response| ServiceWorkerFetchResponseWire {
                status: response.status,
                status_text: response.status_text,
                response_type: response.response_type,
                headers: response.headers,
                body: response.body,
            }),
            failed,
            message,
        },
        ServiceWorkerEvent::CacheStorageRequested { request_id, request } => {
            ServiceWorkerHostEvent::CacheStorageRequested {
                request_id,
                request: cache_storage_request_to_wire(request),
            }
        }
        ServiceWorkerEvent::FetchRequested { request_id, request } => ServiceWorkerHostEvent::FetchRequested {
            request_id,
            request: ServiceWorkerFetchRequestWire {
                url: request.url,
                method: request.method,
                headers: request.headers,
                body: request.body,
                credentials: request.credentials,
                client_id: request.client_id,
                resulting_client_id: request.resulting_client_id,
                referrer: request.referrer,
            },
        },
        ServiceWorkerEvent::ImportScriptsRequested { request_id, specifiers } => {
            ServiceWorkerHostEvent::ImportScriptsRequested { request_id, specifiers }
        }
        ServiceWorkerEvent::UpdateRequested { request_id } => ServiceWorkerHostEvent::UpdateRequested { request_id },
        ServiceWorkerEvent::ClientsMatchAllRequested {
            request_id,
            include_uncontrolled,
            client_type,
        } => ServiceWorkerHostEvent::ClientsMatchAllRequested {
            request_id,
            include_uncontrolled,
            client_type,
        },
        ServiceWorkerEvent::ClientsGetRequested { request_id, client_id } => {
            ServiceWorkerHostEvent::ClientsGetRequested { request_id, client_id }
        }
        ServiceWorkerEvent::ClientMessagesEmitted { outbound } => ServiceWorkerHostEvent::ClientMessagesEmitted {
            outbound: outbound
                .into_iter()
                .map(|message| zero_protocol::message::ServiceWorkerMessage {
                    data_json: message.data_json,
                    port_id: message.port_id,
                    transferred_port_ids: message.transferred_port_ids,
                    data_port_index: message.data_port_index,
                    target_client_id: message.target_client_id,
                })
                .collect(),
        },
        ServiceWorkerEvent::Closed => ServiceWorkerHostEvent::Closed,
        ServiceWorkerEvent::ModuleScriptsRequested {
            request_id,
            referrer_url,
            specifiers,
        } => ServiceWorkerHostEvent::ModuleScriptsRequested {
            request_id,
            referrer_url,
            specifiers,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    /// 内存 writer：捕获 host 线程回传的 IPC 字节。
    #[derive(Clone, Default)]
    struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

    /// 顺序消费 SharedBuffer 的阻塞 reader（等待新字节写入）。
    ///
    /// 超时返回 `Ok(0)`（EOF）：host 线程按整帧写入（flush 在锁内 write_all），
    /// reader 只会在帧边界处空等，EOF 不会撕裂帧；`wait_for_event` 借此让
    /// deadline 断言有机会执行（否则阻塞 read 永不返回）。
    struct BufferReader(SharedBuffer, usize);

    impl Read for BufferReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                {
                    let inner = self.0.0.lock().unwrap();
                    if self.1 < inner.len() {
                        let count = (inner.len() - self.1).min(buf.len());
                        buf[..count].copy_from_slice(&inner[self.1..self.1 + count]);
                        self.1 += count;
                        return Ok(count);
                    }
                }
                if Instant::now() >= deadline {
                    return Ok(0);
                }
                std::thread::sleep(Duration::from_millis(2));
            }
        }
    }

    impl io::Write for SharedBuffer {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// host 句柄 + 单个顺序 transport（多 reader 会重复从头读取）。
    type HostFixture = (RendererServiceWorkerHost, PipeTransport<BufferReader, io::Empty>);

    fn spawn_host() -> HostFixture {
        let buffer = SharedBuffer::default();
        let (writer, _shared) = SharedWriter::new(Box::new(buffer.clone()));
        (
            RendererServiceWorkerHost::new(writer),
            PipeTransport::new(BufferReader(buffer, 0), io::empty()),
        )
    }

    fn evaluate_command(registration_id: u64, script: &str) -> ServiceWorkerHostCommandParams {
        ServiceWorkerHostCommandParams {
            registration_id,
            command: ServiceWorkerHostCommand::Evaluate {
                script_url: "https://example.test/sw.js".into(),
                script: script.into(),
                script_type: ServiceWorkerScriptTypeWire::Classic,
            },
        }
    }

    fn wait_for_event(transport: &mut PipeTransport<BufferReader, io::Empty>) -> ServiceWorkerHostEventParams {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            assert!(Instant::now() < deadline, "host event timed out");
            if let Ok(message) = transport.recv() {
                let IpcMessageKind::ServiceWorkerHostEvent(params) = message.kind else {
                    panic!("expected ServiceWorkerHostEvent, got {:?}", message.kind);
                };
                return params;
            }
        }
    }

    #[test]
    fn evaluate_command_reports_evaluated_event() {
        let (host, mut transport) = spawn_host();
        host.handle_command(evaluate_command(7, "globalThis.ready = true;"));
        let event = wait_for_event(&mut transport);
        assert_eq!(event.registration_id, 7);
        assert_eq!(
            event.event,
            ServiceWorkerHostEvent::Evaluated {
                script_url: "https://example.test/sw.js".into(),
            }
        );
    }

    #[test]
    fn evaluate_command_reports_compile_error_event() {
        let (host, mut transport) = spawn_host();
        host.handle_command(evaluate_command(8, "function("));
        let event = wait_for_event(&mut transport);
        assert!(
            matches!(
                event.event,
                ServiceWorkerHostEvent::ScriptError {
                    kind: ServiceWorkerScriptErrorKindWire::Compile,
                    ..
                }
            ),
            "unexpected event: {:?}",
            event.event
        );
    }

    #[test]
    fn import_scripts_round_trips_through_renderer_host() {
        let (host, mut transport) = spawn_host();
        host.handle_command(evaluate_command(
            12,
            "importScripts('./dependency.js'); if (!globalThis.imported) throw new Error('missing import');",
        ));
        let request = wait_for_event(&mut transport);
        let ServiceWorkerHostEvent::ImportScriptsRequested { request_id, specifiers } = request.event else {
            panic!("expected ImportScriptsRequested");
        };
        assert_eq!(specifiers, ["./dependency.js"]);

        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 12,
            command: ServiceWorkerHostCommand::CompleteImportScripts {
                request_id,
                result: Ok(vec!["globalThis.imported = true;".into()]),
            },
        });
        assert!(matches!(
            wait_for_event(&mut transport).event,
            ServiceWorkerHostEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn module_graph_fetch_round_trips_through_renderer_host() {
        let (host, mut transport) = spawn_host();
        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 13,
            command: ServiceWorkerHostCommand::Evaluate {
                script_url: "https://example.test/workers/sw.js".into(),
                script: "import { value } from './dependency.js'; if (value !== 3) throw new Error('wrong');".into(),
                script_type: ServiceWorkerScriptTypeWire::Module,
            },
        });
        let request = wait_for_event(&mut transport);
        let ServiceWorkerHostEvent::ModuleScriptsRequested {
            request_id,
            referrer_url,
            specifiers,
        } = request.event
        else {
            panic!("expected ModuleScriptsRequested");
        };
        assert_eq!(referrer_url, "https://example.test/workers/sw.js");
        assert_eq!(specifiers, ["./dependency.js"]);

        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 13,
            command: ServiceWorkerHostCommand::CompleteImportScripts {
                request_id,
                result: Ok(vec!["export const value = 3;".into()]),
            },
        });
        assert!(matches!(
            wait_for_event(&mut transport).event,
            ServiceWorkerHostEvent::Evaluated { .. }
        ));
    }

    #[test]
    fn shutdown_command_stops_runtime() {
        let (host, mut transport) = spawn_host();
        host.handle_command(evaluate_command(9, "void 0;"));
        let _ = wait_for_event(&mut transport);

        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 9,
            command: ServiceWorkerHostCommand::Shutdown,
        });
        let event = wait_for_event(&mut transport);
        assert_eq!(event.event, ServiceWorkerHostEvent::Closed);
    }

    #[test]
    fn message_command_dispatches_and_returns_outbound_events() {
        let (host, mut transport) = spawn_host();
        host.handle_command(evaluate_command(
            10,
            "addEventListener('message', event => { event.source.postMessage({echo: event.data}); });",
        ));
        let evaluated = wait_for_event(&mut transport);
        assert!(
            matches!(evaluated.event, ServiceWorkerHostEvent::Evaluated { .. }),
            "{:?}",
            evaluated.event
        );

        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 10,
            command: ServiceWorkerHostCommand::DispatchMessage {
                event_id: 21,
                data_json: "\"ping\"".into(),
                client_id: "tab-1".into(),
                client_url: "https://example.test/page".into(),
                transferred_port_ids: Vec::new(),
                data_port_index: None,
                target_port_id: None,
            },
        });
        let dispatched = wait_for_event(&mut transport);
        match dispatched.event {
            ServiceWorkerHostEvent::MessageDispatched {
                event_id,
                client_id,
                outbound,
            } => {
                assert_eq!((event_id, client_id.as_str()), (21, "tab-1"));
                assert_eq!(outbound.len(), 1, "worker must echo one message: {outbound:?}");
                assert!(
                    outbound[0].data_json.contains("ping"),
                    "unexpected payload: {:?}",
                    outbound[0]
                );
            }
            other => panic!("expected MessageDispatched, got {other:?}"),
        }
    }

    #[test]
    fn fetch_command_dispatches_and_returns_response_event() {
        let (host, mut transport) = spawn_host();
        host.handle_command(evaluate_command(
            14,
            "addEventListener('fetch', event => {
               event.respondWith(new Response('from-sw', {
                 status: 202,
                 headers: {'X-Test': event.request.headers.get('x-test')}
               }));
             });",
        ));
        let evaluated = wait_for_event(&mut transport);
        assert!(
            matches!(evaluated.event, ServiceWorkerHostEvent::Evaluated { .. }),
            "{:?}",
            evaluated.event
        );

        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 14,
            command: ServiceWorkerHostCommand::DispatchFetch {
                event_id: 22,
                request: zero_protocol::message::ServiceWorkerFetchRequestWire {
                    url: "https://example.test/app/data".into(),
                    method: "GET".into(),
                    headers: vec![("x-test".into(), "yes".into())],
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                },
            },
        });
        let dispatched = wait_for_event(&mut transport);
        match dispatched.event {
            ServiceWorkerHostEvent::FetchSettled {
                event_id,
                request_url,
                response: Some(response),
                message,
                ..
            } => {
                assert_eq!(event_id, 22);
                assert_eq!(request_url, "https://example.test/app/data");
                assert_eq!(response.status, 202);
                assert_eq!(response.headers, [("x-test".into(), "yes".into())]);
                assert_eq!(response.body, "from-sw");
                assert!(message.is_empty());
            }
            other => panic!("expected FetchSettled response, got {other:?}"),
        }
    }

    #[test]
    fn cache_match_round_trips_through_renderer_host() {
        let (host, mut transport) = spawn_host();
        host.handle_command(evaluate_command(
            15,
            "addEventListener('fetch', event => {
               event.respondWith(caches.match(event.request));
             });",
        ));
        let evaluated = wait_for_event(&mut transport);
        assert!(
            matches!(evaluated.event, ServiceWorkerHostEvent::Evaluated { .. }),
            "{:?}",
            evaluated.event
        );

        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 15,
            command: ServiceWorkerHostCommand::DispatchFetch {
                event_id: 23,
                request: zero_protocol::message::ServiceWorkerFetchRequestWire {
                    url: "https://example.test/app/cached".into(),
                    method: "GET".into(),
                    headers: vec![("accept".into(), "text/plain".into())],
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                },
            },
        });
        let cache_request = wait_for_event(&mut transport);
        let ServiceWorkerHostEvent::CacheStorageRequested { request_id, request } = cache_request.event else {
            panic!("expected CacheStorageRequested, got {:?}", cache_request.event);
        };
        let ServiceWorkerCacheStorageRequestWire::Match {
            cache_name: None,
            request,
            options,
            ..
        } = request
        else {
            panic!("expected CacheStorage.match request");
        };
        assert_eq!(request.url, "https://example.test/app/cached");
        assert_eq!(request.method, "GET");
        assert_eq!(options, ServiceWorkerCacheQueryOptionsWire::default());

        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 15,
            command: ServiceWorkerHostCommand::CompleteCacheStorage {
                request_id,
                result: Ok(ServiceWorkerCacheStorageResultWire::Match(Some(
                    ServiceWorkerFetchResponseWire {
                        status: 200,
                        status_text: "OK".into(),
                        response_type: "default".into(),
                        headers: vec![("x-cache".into(), "hit".into())],
                        body: "cached-body".into(),
                    },
                ))),
            },
        });
        let settled = wait_for_event(&mut transport);
        match settled.event {
            ServiceWorkerHostEvent::FetchSettled {
                event_id,
                request_url,
                response: Some(response),
                message,
                ..
            } => {
                assert_eq!(event_id, 23);
                assert_eq!(request_url, "https://example.test/app/cached");
                assert_eq!(response.status, 200);
                assert_eq!(response.headers, [("x-cache".into(), "hit".into())]);
                assert_eq!(response.body, "cached-body");
                assert!(message.is_empty());
            }
            other => panic!("expected FetchSettled response, got {other:?}"),
        }
    }

    #[test]
    fn cache_storage_open_put_match_all_keys_round_trips_through_renderer_host() {
        let (host, mut transport) = spawn_host();
        host.handle_command(evaluate_command(
            16,
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
        ));
        let evaluated = wait_for_event(&mut transport);
        assert!(
            matches!(evaluated.event, ServiceWorkerHostEvent::Evaluated { .. }),
            "{:?}",
            evaluated.event
        );

        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 16,
            command: ServiceWorkerHostCommand::DispatchFetch {
                event_id: 24,
                request: zero_protocol::message::ServiceWorkerFetchRequestWire {
                    url: "https://example.test/app/stored".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                },
            },
        });

        let open_request = wait_for_event(&mut transport);
        let ServiceWorkerHostEvent::CacheStorageRequested { request_id, request } = open_request.event else {
            panic!("expected CacheStorage.open request");
        };
        assert_eq!(
            request,
            ServiceWorkerCacheStorageRequestWire::Open {
                cache_name: "runtime".into()
            }
        );
        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 16,
            command: ServiceWorkerHostCommand::CompleteCacheStorage {
                request_id,
                result: Ok(ServiceWorkerCacheStorageResultWire::Open {
                    cache_name: "runtime".into(),
                    cache_name_units: "00720075006e00740069006d0065".into(),
                    cache_id: 7,
                }),
            },
        });

        let put_request = wait_for_event(&mut transport);
        let ServiceWorkerHostEvent::CacheStorageRequested { request_id, request } = put_request.event else {
            panic!("expected Cache.put request");
        };
        let ServiceWorkerCacheStorageRequestWire::Put {
            cache_name,
            cache_id,
            request,
            response,
        } = request
        else {
            panic!("expected Cache.put payload");
        };
        assert_eq!(cache_name, "runtime");
        assert_eq!(cache_id, Some(7));
        assert_eq!(request.url, "https://example.test/app/stored");
        assert_eq!(response.status, 201);
        assert_eq!(response.status_text, "Created");
        assert_eq!(response.response_type, "default");
        assert_eq!(response.headers, [("x-cache".into(), "put".into())]);
        assert_eq!(response.body, "stored-body");
        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 16,
            command: ServiceWorkerHostCommand::CompleteCacheStorage {
                request_id,
                result: Ok(ServiceWorkerCacheStorageResultWire::Done),
            },
        });

        let match_all_request = wait_for_event(&mut transport);
        let ServiceWorkerHostEvent::CacheStorageRequested { request_id, request } = match_all_request.event else {
            panic!("expected Cache.matchAll request");
        };
        let ServiceWorkerCacheStorageRequestWire::MatchAll {
            cache_name,
            cache_id,
            request: Some(request),
            options,
        } = request
        else {
            panic!("expected named Cache.matchAll payload");
        };
        assert_eq!(cache_name, "runtime");
        assert_eq!(cache_id, Some(7));
        assert_eq!(request.url, "https://example.test/app/stored");
        assert_eq!(options, ServiceWorkerCacheQueryOptionsWire::default());
        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 16,
            command: ServiceWorkerHostCommand::CompleteCacheStorage {
                request_id,
                result: Ok(ServiceWorkerCacheStorageResultWire::MatchAll(vec![
                    ServiceWorkerFetchResponseWire {
                        status: 201,
                        status_text: "Created".into(),
                        response_type: "default".into(),
                        headers: vec![("x-cache".into(), "put".into())],
                        body: "stored-body".into(),
                    },
                ])),
            },
        });

        let keys_request = wait_for_event(&mut transport);
        let ServiceWorkerHostEvent::CacheStorageRequested { request_id, request } = keys_request.event else {
            panic!("expected Cache.keys request");
        };
        assert_eq!(
            request,
            ServiceWorkerCacheStorageRequestWire::Keys {
                cache_name: "runtime".into(),
                cache_id: Some(7),
                request: None,
                options: ServiceWorkerCacheQueryOptionsWire::default(),
            }
        );
        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 16,
            command: ServiceWorkerHostCommand::CompleteCacheStorage {
                request_id,
                result: Ok(ServiceWorkerCacheStorageResultWire::Keys(vec![
                    ServiceWorkerFetchRequestWire {
                        url: "https://example.test/app/stored".into(),
                        method: "GET".into(),
                        headers: Vec::new(),
                        body: None,
                        credentials: None,
                        client_id: None,
                        resulting_client_id: None,
                        referrer: None,
                    },
                ])),
            },
        });

        let settled = wait_for_event(&mut transport);
        match settled.event {
            ServiceWorkerHostEvent::FetchSettled {
                event_id,
                request_url,
                response: Some(response),
                message,
                ..
            } => {
                assert_eq!(event_id, 24);
                assert_eq!(request_url, "https://example.test/app/stored");
                assert_eq!(response.status, 201);
                assert_eq!(response.status_text, "Created");
                assert_eq!(response.headers, [("x-cache".into(), "put".into())]);
                assert_eq!(response.body, "stored-body");
                assert!(message.is_empty());
            }
            other => panic!("expected FetchSettled response, got {other:?}"),
        }
    }

    #[test]
    fn cache_delete_and_storage_listing_round_trips_through_renderer_host() {
        let (host, mut transport) = spawn_host();
        host.handle_command(evaluate_command(
            17,
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
        ));
        let evaluated = wait_for_event(&mut transport);
        assert!(
            matches!(evaluated.event, ServiceWorkerHostEvent::Evaluated { .. }),
            "{:?}",
            evaluated.event
        );

        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 17,
            command: ServiceWorkerHostCommand::DispatchFetch {
                event_id: 25,
                request: zero_protocol::message::ServiceWorkerFetchRequestWire {
                    url: "https://example.test/app/delete?version=1".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                },
            },
        });

        let open_request = wait_for_event(&mut transport);
        let ServiceWorkerHostEvent::CacheStorageRequested { request_id, request } = open_request.event else {
            panic!("expected CacheStorage.open request");
        };
        assert_eq!(
            request,
            ServiceWorkerCacheStorageRequestWire::Open {
                cache_name: "runtime".into()
            }
        );
        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 17,
            command: ServiceWorkerHostCommand::CompleteCacheStorage {
                request_id,
                result: Ok(ServiceWorkerCacheStorageResultWire::Open {
                    cache_name: "runtime".into(),
                    cache_name_units: "00720075006e00740069006d0065".into(),
                    cache_id: 8,
                }),
            },
        });

        let has_request = wait_for_event(&mut transport);
        let ServiceWorkerHostEvent::CacheStorageRequested { request_id, request } = has_request.event else {
            panic!("expected CacheStorage.has request");
        };
        assert_eq!(
            request,
            ServiceWorkerCacheStorageRequestWire::StorageHas {
                cache_name: "runtime".into()
            }
        );
        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 17,
            command: ServiceWorkerHostCommand::CompleteCacheStorage {
                request_id,
                result: Ok(ServiceWorkerCacheStorageResultWire::Bool(true)),
            },
        });

        let keys_request = wait_for_event(&mut transport);
        let ServiceWorkerHostEvent::CacheStorageRequested { request_id, request } = keys_request.event else {
            panic!("expected CacheStorage.keys request");
        };
        assert_eq!(request, ServiceWorkerCacheStorageRequestWire::StorageKeys);
        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 17,
            command: ServiceWorkerHostCommand::CompleteCacheStorage {
                request_id,
                result: Ok(ServiceWorkerCacheStorageResultWire::StorageKeys(vec!["runtime".into()])),
            },
        });

        let delete_request = wait_for_event(&mut transport);
        let ServiceWorkerHostEvent::CacheStorageRequested { request_id, request } = delete_request.event else {
            panic!("expected Cache.delete request");
        };
        let ServiceWorkerCacheStorageRequestWire::Delete {
            cache_name,
            cache_id,
            request,
            options,
        } = request
        else {
            panic!("expected Cache.delete payload");
        };
        assert_eq!(cache_name, "runtime");
        assert_eq!(cache_id, Some(8));
        assert_eq!(request.url, "https://example.test/app/delete?version=1");
        assert_eq!(
            options,
            ServiceWorkerCacheQueryOptionsWire {
                ignore_search: true,
                ignore_method: false,
                ignore_vary: false,
            }
        );
        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 17,
            command: ServiceWorkerHostCommand::CompleteCacheStorage {
                request_id,
                result: Ok(ServiceWorkerCacheStorageResultWire::Bool(true)),
            },
        });

        let storage_delete_request = wait_for_event(&mut transport);
        let ServiceWorkerHostEvent::CacheStorageRequested { request_id, request } = storage_delete_request.event else {
            panic!("expected CacheStorage.delete request");
        };
        assert_eq!(
            request,
            ServiceWorkerCacheStorageRequestWire::StorageDelete {
                cache_name: "runtime".into()
            }
        );
        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 17,
            command: ServiceWorkerHostCommand::CompleteCacheStorage {
                request_id,
                result: Ok(ServiceWorkerCacheStorageResultWire::Bool(true)),
            },
        });

        let settled = wait_for_event(&mut transport);
        match settled.event {
            ServiceWorkerHostEvent::FetchSettled {
                event_id,
                request_url,
                response: Some(response),
                message,
                ..
            } => {
                assert_eq!(event_id, 25);
                assert_eq!(request_url, "https://example.test/app/delete?version=1");
                assert_eq!(response.status, 200);
                assert_eq!(response.body, "true|runtime|true|true");
                assert!(message.is_empty());
            }
            other => panic!("expected FetchSettled response, got {other:?}"),
        }
    }

    #[test]
    fn cache_query_options_round_trip_through_renderer_host() {
        let (host, mut transport) = spawn_host();
        host.handle_command(evaluate_command(
            18,
            "addEventListener('fetch', event => {
               event.respondWith(caches.match(event.request, {
                 ignoreSearch: true,
                 ignoreMethod: true,
                 ignoreVary: true
               }));
             });",
        ));
        let evaluated = wait_for_event(&mut transport);
        assert!(
            matches!(evaluated.event, ServiceWorkerHostEvent::Evaluated { .. }),
            "{:?}",
            evaluated.event
        );

        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 18,
            command: ServiceWorkerHostCommand::DispatchFetch {
                event_id: 26,
                request: zero_protocol::message::ServiceWorkerFetchRequestWire {
                    url: "https://example.test/app/cached?from=fetch".into(),
                    method: "HEAD".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                },
            },
        });
        let cache_request = wait_for_event(&mut transport);
        let ServiceWorkerHostEvent::CacheStorageRequested { request_id, request } = cache_request.event else {
            panic!("expected CacheStorageRequested, got {:?}", cache_request.event);
        };
        let ServiceWorkerCacheStorageRequestWire::Match {
            cache_name: None,
            request,
            options,
            ..
        } = request
        else {
            panic!("expected CacheStorage.match request");
        };
        assert_eq!(request.url, "https://example.test/app/cached?from=fetch");
        assert_eq!(request.method, "HEAD");
        assert_eq!(
            options,
            ServiceWorkerCacheQueryOptionsWire {
                ignore_search: true,
                ignore_method: true,
                ignore_vary: true,
            }
        );

        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 18,
            command: ServiceWorkerHostCommand::CompleteCacheStorage {
                request_id,
                result: Ok(ServiceWorkerCacheStorageResultWire::Match(Some(
                    ServiceWorkerFetchResponseWire {
                        status: 200,
                        status_text: "OK".into(),
                        response_type: "default".into(),
                        headers: Vec::new(),
                        body: "cached-body".into(),
                    },
                ))),
            },
        });
        let settled = wait_for_event(&mut transport);
        assert!(matches!(
            settled.event,
            ServiceWorkerHostEvent::FetchSettled {
                event_id: 26,
                response: Some(_),
                ..
            }
        ));
    }

    #[test]
    fn worker_global_fetch_round_trips_through_renderer_host() {
        let (host, mut transport) = spawn_host();
        host.handle_command(evaluate_command(
            18,
            "addEventListener('fetch', event => {
               event.respondWith(fetch('./asset.txt', {headers: {'X-Test': 'yes'}}));
             });",
        ));
        let evaluated = wait_for_event(&mut transport);
        assert!(
            matches!(evaluated.event, ServiceWorkerHostEvent::Evaluated { .. }),
            "{:?}",
            evaluated.event
        );

        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 18,
            command: ServiceWorkerHostCommand::DispatchFetch {
                event_id: 26,
                request: zero_protocol::message::ServiceWorkerFetchRequestWire {
                    url: "https://example.test/app/page".into(),
                    method: "GET".into(),
                    headers: Vec::new(),
                    body: None,
                    credentials: None,
                    client_id: Some("client-1".into()),
                    resulting_client_id: None,
                    referrer: None,
                },
            },
        });
        let fetch_request = wait_for_event(&mut transport);
        let ServiceWorkerHostEvent::FetchRequested { request_id, request } = fetch_request.event else {
            panic!("expected FetchRequested");
        };
        assert_eq!(request.url, "https://example.test/asset.txt");
        assert_eq!(request.method, "GET");
        assert_eq!(request.headers, [("x-test".into(), "yes".into())]);

        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 18,
            command: ServiceWorkerHostCommand::CompleteFetch {
                request_id,
                result: Ok(ServiceWorkerFetchResponseWire {
                    status: 200,
                    status_text: "OK".into(),
                    response_type: "default".into(),
                    headers: vec![("content-type".into(), "text/plain".into())],
                    body: "from-network".into(),
                }),
            },
        });
        let settled = wait_for_event(&mut transport);
        match settled.event {
            ServiceWorkerHostEvent::FetchSettled {
                event_id,
                response: Some(response),
                ..
            } => {
                assert_eq!(event_id, 26);
                assert_eq!(response.status, 200);
                assert_eq!(response.body, "from-network");
            }
            other => panic!("expected FetchSettled response, got {other:?}"),
        }
    }

    #[test]
    fn invalid_command_is_rejected_without_runtime() {
        let (host, transport) = spawn_host();
        host.handle_command(ServiceWorkerHostCommandParams {
            registration_id: 1,
            command: ServiceWorkerHostCommand::Evaluate {
                script_url: String::new(),
                script: "void 0;".into(),
                script_type: ServiceWorkerScriptTypeWire::Classic,
            },
        });
        // 托管线程 drain 完命令队列后不应产生任何回传字节。
        drop(transport);
    }
}
