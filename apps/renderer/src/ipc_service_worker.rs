use std::collections::HashMap;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use zero_protocol::{
    IpcChannel, IpcMessage, IpcMessageKind, PipeTransport, ServiceWorkerError, ServiceWorkerErrorCode,
    ServiceWorkerOperation, ServiceWorkerRequestParams, ServiceWorkerResult, ServiceWorkerStateWire,
};

use crate::compositor_publish_thread::SharedWriter;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_PENDING_REQUESTS: usize = 64;

type ServiceWorkerResponse = Result<ServiceWorkerResult, ServiceWorkerError>;

pub(crate) struct ServiceWorkerResponseRouter {
    pending: Mutex<HashMap<u64, Sender<ServiceWorkerResponse>>>,
}

impl ServiceWorkerResponseRouter {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            pending: Mutex::new(HashMap::new()),
        })
    }

    pub(crate) fn route(&self, message: IpcMessage) -> Option<IpcMessage> {
        let IpcMessage { id, kind } = message;
        let IpcMessageKind::ServiceWorkerResponse(params) = kind else {
            return Some(IpcMessage { id, kind });
        };
        if let Ok(mut pending) = self.pending.lock()
            && let Some(sender) = pending.remove(&id)
        {
            let _ = sender.send(params.result);
        }
        None
    }

    pub(crate) fn fail_all(&self, message: &str) {
        let Ok(mut pending) = self.pending.lock() else {
            return;
        };
        for (_, sender) in pending.drain() {
            let _ = sender.send(Err(ServiceWorkerError {
                code: ServiceWorkerErrorCode::Internal,
                message: message.to_string(),
            }));
        }
    }

    fn register(&self, id: u64, sender: Sender<ServiceWorkerResponse>) -> Result<(), String> {
        let mut pending = self
            .pending
            .lock()
            .map_err(|_| "Service Worker response router lock is poisoned".to_string())?;
        if pending.len() >= MAX_PENDING_REQUESTS {
            return Err("Service Worker pending request limit reached".into());
        }
        if pending.insert(id, sender).is_some() {
            return Err("duplicate Service Worker request id".into());
        }
        Ok(())
    }

    fn remove(&self, id: u64) {
        if let Ok(mut pending) = self.pending.lock() {
            pending.remove(&id);
        }
    }
}

#[derive(Clone)]
pub(crate) struct ServiceWorkerIpcClient {
    outbound: Arc<Mutex<PipeTransport<io::Empty, SharedWriter>>>,
    router: Arc<ServiceWorkerResponseRouter>,
    next_id: Arc<AtomicU64>,
}

impl ServiceWorkerIpcClient {
    pub(crate) fn new(writer: SharedWriter, router: Arc<ServiceWorkerResponseRouter>) -> Self {
        Self {
            outbound: Arc::new(Mutex::new(PipeTransport::new(io::empty(), writer))),
            router,
            next_id: Arc::new(AtomicU64::new(1 << 62)),
        }
    }

    pub(crate) fn register_callbacks(&self, sandbox: &mut dyn zero_script_sandbox::Sandbox) {
        let register_client = self.clone();
        sandbox.register_callback(
            "__zw_sw_register",
            Box::new(move |args| {
                let script_url = args.first().cloned().unwrap_or_default();
                let scope =
                    (args.get(3).map(String::as_str) == Some("true")).then(|| args.get(1).cloned().unwrap_or_default());
                let document_url = args.get(2).cloned().unwrap_or_default();
                match register_client.request(ServiceWorkerOperation::Register {
                    script_url,
                    scope,
                    document_url,
                }) {
                    Ok(ServiceWorkerResult::Registered { registration_id }) => {
                        serde_json::json!({"ok": true, "id": registration_id}).to_string()
                    }
                    Ok(_) => error_wire("invalid register response"),
                    Err(error) => response_error_wire(error),
                }
            }),
        );

        let snapshot_client = self.clone();
        sandbox.register_callback(
            "__zw_sw_snapshot",
            Box::new(move |args| {
                let Some(registration_id) = parse_registration_id(args) else {
                    return error_wire("invalid registration id");
                };
                match snapshot_client.request(ServiceWorkerOperation::Snapshot { registration_id }) {
                    Ok(ServiceWorkerResult::Snapshot(snapshot)) => {
                        let mut wire = snapshot_wire(snapshot);
                        wire["ok"] = serde_json::Value::Bool(true);
                        wire.to_string()
                    }
                    Ok(_) => error_wire("invalid snapshot response"),
                    Err(error) => error_wire(error.message),
                }
            }),
        );

        let state_changes_client = self.clone();
        sandbox.register_callback(
            "__zw_sw_state_changes",
            Box::new(move |args| {
                let Some(registration_id) = parse_registration_id(args) else {
                    return error_wire("invalid registration id");
                };
                let after_sequence = args.get(1).and_then(|value| value.parse::<u64>().ok()).unwrap_or(0);
                match state_changes_client.request(ServiceWorkerOperation::StateChanges {
                    registration_id,
                    after_sequence,
                }) {
                    Ok(ServiceWorkerResult::StateChanges(changes)) => serde_json::json!({
                        "ok": true,
                        "latestSequence": changes.latest_sequence,
                        "states": changes.states.into_iter().map(state_wire).collect::<Vec<_>>(),
                        "claimClients": changes.claim_clients,
                    })
                    .to_string(),
                    Ok(_) => error_wire("invalid state changes response"),
                    Err(error) => error_wire(error.message),
                }
            }),
        );

        let unregister_client = self.clone();
        sandbox.register_callback(
            "__zw_sw_unregister",
            Box::new(move |args| {
                let Some(registration_id) = parse_registration_id(args) else {
                    return "false".into();
                };
                match unregister_client.request(ServiceWorkerOperation::Unregister { registration_id }) {
                    Ok(ServiceWorkerResult::Boolean(removed)) => removed.to_string(),
                    _ => "false".into(),
                }
            }),
        );

        let get_registration_client = self.clone();
        sandbox.register_callback(
            "__zw_sw_get_registration",
            Box::new(move |args| {
                let client_url = args.first().cloned().unwrap_or_default();
                match get_registration_client.request(ServiceWorkerOperation::GetRegistration { client_url }) {
                    Ok(ServiceWorkerResult::OptionalSnapshot(snapshot)) => serde_json::json!({
                        "ok": true,
                        "registration": snapshot.map(snapshot_wire),
                    })
                    .to_string(),
                    Ok(_) => error_wire("invalid getRegistration response"),
                    Err(error) => error_wire(error.message),
                }
            }),
        );

        let controller_client = self.clone();
        sandbox.register_callback(
            "__zw_sw_controller",
            Box::new(
                move |_args| match controller_client.request(ServiceWorkerOperation::Controller) {
                    Ok(ServiceWorkerResult::OptionalSnapshot(snapshot)) => serde_json::json!({
                        "ok": true,
                        "controller": snapshot
                            .filter(|snapshot| snapshot.state == ServiceWorkerStateWire::Activated)
                            .map(snapshot_wire),
                    })
                    .to_string(),
                    Ok(_) => error_wire("invalid controller response"),
                    Err(error) => error_wire(error.message),
                },
            ),
        );

        let get_registrations_client = self.clone();
        sandbox.register_callback(
            "__zw_sw_get_registrations",
            Box::new(
                move |_args| match get_registrations_client.request(ServiceWorkerOperation::GetRegistrations) {
                    Ok(ServiceWorkerResult::Snapshots(snapshots)) => serde_json::json!({
                        "ok": true,
                        "registrations": snapshots.into_iter().map(snapshot_wire).collect::<Vec<_>>(),
                    })
                    .to_string(),
                    Ok(_) => error_wire("invalid getRegistrations response"),
                    Err(error) => error_wire(error.message),
                },
            ),
        );
    }

    fn request(&self, operation: ServiceWorkerOperation) -> ServiceWorkerResponse {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (reply_tx, reply_rx) = mpsc::channel();
        self.router.register(id, reply_tx).map_err(internal_error)?;
        let message = IpcMessage {
            id,
            kind: IpcMessageKind::ServiceWorkerRequest(ServiceWorkerRequestParams { operation }),
        };
        let send_result = match self.outbound.lock() {
            Ok(mut outbound) => outbound
                .send(message)
                .map_err(|error| internal_error(format!("Service Worker IPC send failed: {error}"))),
            Err(_) => Err(internal_error("Service Worker outbound lock is poisoned")),
        };
        if let Err(error) = send_result {
            self.router.remove(id);
            return Err(error);
        }
        match reply_rx.recv_timeout(REQUEST_TIMEOUT) {
            Ok(result) => result,
            Err(error) => {
                self.router.remove(id);
                Err(internal_error(format!("Service Worker IPC response failed: {error}")))
            }
        }
    }
}

fn parse_registration_id(args: &[String]) -> Option<u64> {
    args.first()?.parse().ok()
}

fn state_wire(state: ServiceWorkerStateWire) -> &'static str {
    match state {
        ServiceWorkerStateWire::Installing => "installing",
        ServiceWorkerStateWire::Installed => "installed",
        ServiceWorkerStateWire::Activating => "activating",
        ServiceWorkerStateWire::Activated => "activated",
        ServiceWorkerStateWire::Redundant => "redundant",
    }
}

fn snapshot_wire(snapshot: zero_protocol::ServiceWorkerSnapshot) -> serde_json::Value {
    serde_json::json!({
        "id": snapshot.registration_id,
        "scriptURL": snapshot.script_url,
        "scope": snapshot.scope,
        "state": state_wire(snapshot.state),
    })
}

fn error_wire(message: impl Into<String>) -> String {
    serde_json::json!({"ok": false, "error": message.into(), "errorName": "TypeError"}).to_string()
}

fn response_error_wire(error: ServiceWorkerError) -> String {
    serde_json::json!({
        "ok": false,
        "error": error.message,
        "errorName": match error.code {
            ServiceWorkerErrorCode::Security => "SecurityError",
            _ => "TypeError",
        },
    })
    .to_string()
}

fn internal_error(message: impl Into<String>) -> ServiceWorkerError {
    ServiceWorkerError {
        code: ServiceWorkerErrorCode::Internal,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::thread;
    use std::time::Instant;

    use super::*;
    use zero_protocol::{ServiceWorkerResponseParams, ServiceWorkerSnapshot};

    struct CapturedWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for CapturedWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _bytes: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("expected write failure"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn wait_for_request(bytes: &Arc<Mutex<Vec<u8>>>) -> IpcMessage {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let frame = bytes.lock().unwrap().clone();
            if frame.len() >= 4 {
                let length = u32::from_le_bytes(frame[..4].try_into().unwrap()) as usize;
                if frame.len() >= length + 4 {
                    return zero_protocol::serialize::deserialize(&frame[4..length + 4]).unwrap();
                }
            }
            assert!(Instant::now() < deadline, "Service Worker IPC request was not written");
            thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn client_correlates_typed_snapshot_response() {
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let (writer, _) = SharedWriter::new(Box::new(CapturedWriter(Arc::clone(&bytes))));
        let router = ServiceWorkerResponseRouter::new();
        let client = ServiceWorkerIpcClient::new(writer, Arc::clone(&router));
        let join = thread::spawn(move || client.request(ServiceWorkerOperation::Snapshot { registration_id: 7 }));

        let request = wait_for_request(&bytes);
        assert!(matches!(
            request.kind,
            IpcMessageKind::ServiceWorkerRequest(ServiceWorkerRequestParams {
                operation: ServiceWorkerOperation::Snapshot { registration_id: 7 }
            })
        ));
        assert!(
            router
                .route(IpcMessage {
                    id: request.id,
                    kind: IpcMessageKind::ServiceWorkerResponse(ServiceWorkerResponseParams {
                        result: Ok(ServiceWorkerResult::Snapshot(ServiceWorkerSnapshot {
                            registration_id: 7,
                            script_url: "https://example.test/sw.js".into(),
                            scope: "https://example.test/".into(),
                            state: ServiceWorkerStateWire::Activated,
                        })),
                    }),
                })
                .is_none()
        );
        assert!(matches!(
            join.join().unwrap(),
            Ok(ServiceWorkerResult::Snapshot(ServiceWorkerSnapshot {
                registration_id: 7,
                state: ServiceWorkerStateWire::Activated,
                ..
            }))
        ));
    }

    #[test]
    fn router_preserves_unrelated_messages() {
        let router = ServiceWorkerResponseRouter::new();
        let message = router
            .route(IpcMessage {
                id: 9,
                kind: IpcMessageKind::Heartbeat,
            })
            .unwrap();
        assert_eq!(message.id, 9);
        assert!(matches!(message.kind, IpcMessageKind::Heartbeat));
    }

    #[test]
    fn outbound_send_failure_removes_pending_request() {
        let (writer, _) = SharedWriter::new(Box::new(FailingWriter));
        let router = ServiceWorkerResponseRouter::new();
        let client = ServiceWorkerIpcClient::new(writer, Arc::clone(&router));

        let result = client.request(ServiceWorkerOperation::Snapshot { registration_id: 1 });

        assert!(matches!(
            result,
            Err(ServiceWorkerError {
                code: ServiceWorkerErrorCode::Internal,
                ..
            })
        ));
        assert!(router.pending.lock().unwrap().is_empty());
    }
}
