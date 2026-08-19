use std::collections::HashMap;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use zero_engine::IndexedDbHandler;
use zero_protocol::{
    IndexedDbRequestParams, IndexedDbResponseParams, IpcChannel, IpcMessage, IpcMessageKind, PipeTransport,
};

use crate::compositor_publish_thread::SharedWriter;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_PENDING_REQUESTS: usize = 64;

type IndexedDbResult = Result<String, String>;

/// Browser IPC reader 使用的 IndexedDB response router。
pub(crate) struct IndexedDbResponseRouter {
    pending: Mutex<HashMap<u64, Sender<IndexedDbResult>>>,
}

impl IndexedDbResponseRouter {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            pending: Mutex::new(HashMap::new()),
        })
    }

    pub(crate) fn route(&self, message: IpcMessage) -> Option<IpcMessage> {
        let IpcMessage { id, kind } = message;
        let IpcMessageKind::IndexedDbResponse(params) = kind else {
            return Some(IpcMessage { id, kind });
        };
        let result = response_result(params);
        if let Ok(mut pending) = self.pending.lock()
            && let Some(sender) = pending.remove(&id)
        {
            let _ = sender.send(result);
        }
        None
    }

    pub(crate) fn fail_all(&self, error: String) {
        let Ok(mut pending) = self.pending.lock() else {
            return;
        };
        for (_, sender) in pending.drain() {
            let _ = sender.send(Err(error.clone()));
        }
    }

    fn register(&self, id: u64, sender: Sender<IndexedDbResult>) -> Result<(), String> {
        let mut pending = self
            .pending
            .lock()
            .map_err(|_| "UnknownError: IndexedDB response router lock is poisoned".to_string())?;
        if pending.len() >= MAX_PENDING_REQUESTS {
            return Err("UnknownError: IndexedDB pending request limit reached".to_string());
        }
        if pending.insert(id, sender).is_some() {
            return Err("UnknownError: duplicate IndexedDB request id".to_string());
        }
        Ok(())
    }

    fn remove(&self, id: u64) {
        if let Ok(mut pending) = self.pending.lock() {
            pending.remove(&id);
        }
    }
}

pub(crate) fn spawn_ipc_inbound<R>(reader: R) -> (Receiver<IpcMessage>, JoinHandle<()>)
where
    R: io::Read + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let join = thread::Builder::new()
        .name("renderer-ipc-in".into())
        .spawn(move || {
            let mut transport = PipeTransport::new(reader, io::empty());
            loop {
                match transport.recv() {
                    Ok(message) => {
                        if tx.send(message).is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        tracing::warn!("Renderer stdin IPC reader stopped: {error}");
                        break;
                    }
                }
            }
        })
        .expect("spawn renderer ipc inbound reader");
    (rx, join)
}

pub(crate) fn route_browser_ipc_inbound(
    source: Receiver<IpcMessage>,
    indexed_db_responses: Arc<IndexedDbResponseRouter>,
    service_worker_responses: Arc<crate::ipc_service_worker::ServiceWorkerResponseRouter>,
) -> (Receiver<IpcMessage>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let join = thread::Builder::new()
        .name("renderer-ipc-router".into())
        .spawn(move || {
            while let Ok(message) = source.recv() {
                if let Some(message) = service_worker_responses.route(message)
                    && let Some(message) = indexed_db_responses.route(message)
                    && tx.send(message).is_err()
                {
                    break;
                }
            }
            indexed_db_responses.fail_all("UnknownError: browser IPC disconnected".to_string());
            service_worker_responses.fail_all("browser IPC disconnected");
        })
        .expect("spawn renderer ipc router");
    (rx, join)
}

pub(crate) fn indexed_db_handler(writer: SharedWriter, router: Arc<IndexedDbResponseRouter>) -> IndexedDbHandler {
    let outbound = Arc::new(Mutex::new(PipeTransport::new(io::empty(), writer)));
    let next_id = Arc::new(AtomicU64::new(1 << 63));
    Arc::new(move |_origin, request| {
        let id = next_id.fetch_add(1, Ordering::Relaxed);
        let (reply_tx, reply_rx) = mpsc::channel();
        router.register(id, reply_tx)?;
        let message = IpcMessage {
            id,
            kind: IpcMessageKind::IndexedDbRequest(IndexedDbRequestParams {
                request: request.to_string(),
            }),
        };
        let send_result = outbound
            .lock()
            .map_err(|_| "UnknownError: IndexedDB outbound lock is poisoned".to_string())?
            .send(message)
            .map_err(|error| format!("UnknownError: IndexedDB IPC send failed: {error}"));
        if let Err(error) = send_result {
            router.remove(id);
            return Err(error);
        }
        match reply_rx.recv_timeout(REQUEST_TIMEOUT) {
            Ok(result) => result,
            Err(error) => {
                router.remove(id);
                Err(format!("UnknownError: IndexedDB IPC response failed: {error}"))
            }
        }
    })
}

fn response_result(params: IndexedDbResponseParams) -> IndexedDbResult {
    match (params.response, params.error) {
        (Some(response), None) => Ok(response),
        (None, Some(error)) => Err(error),
        _ => Err("UnknownError: invalid IndexedDB IPC response".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::thread;
    use std::time::{Duration, Instant};

    use super::*;

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

    #[test]
    fn ipc_handler_waits_for_matching_routed_response() {
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let (writer, _) = SharedWriter::new(Box::new(CapturedWriter(Arc::clone(&bytes))));
        let router = IndexedDbResponseRouter::new();
        let handler = indexed_db_handler(writer, Arc::clone(&router));
        let join = thread::spawn(move || handler("https://ignored.example", r#"{"op":"databases"}"#));

        let deadline = Instant::now() + Duration::from_secs(2);
        let request = loop {
            let frame = bytes.lock().unwrap().clone();
            if frame.len() >= 4 {
                let length = u32::from_le_bytes(frame[..4].try_into().unwrap()) as usize;
                if frame.len() >= length + 4 {
                    break zero_protocol::serialize::deserialize(&frame[4..length + 4]).unwrap();
                }
            }
            assert!(Instant::now() < deadline, "IndexedDB IPC request was not written");
            thread::sleep(Duration::from_millis(1));
        };
        assert!(matches!(
            &request.kind,
            IpcMessageKind::IndexedDbRequest(IndexedDbRequestParams { request })
                if request == r#"{"op":"databases"}"#
        ));
        assert!(
            router
                .route(IpcMessage {
                    id: request.id,
                    kind: IpcMessageKind::IndexedDbResponse(IndexedDbResponseParams {
                        response: Some(r#"{"databases":[]}"#.to_string()),
                        error: None,
                    }),
                })
                .is_none()
        );
        assert_eq!(join.join().unwrap().unwrap(), r#"{"databases":[]}"#);
    }

    #[test]
    fn router_preserves_non_indexed_db_messages() {
        let router = IndexedDbResponseRouter::new();
        let message = router
            .route(IpcMessage {
                id: 7,
                kind: IpcMessageKind::Heartbeat,
            })
            .unwrap();
        assert_eq!(message.id, 7);
        assert!(matches!(message.kind, IpcMessageKind::Heartbeat));
    }
}
