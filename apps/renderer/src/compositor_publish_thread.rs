//! RFC 4.1：renderer compositor 帧 IPC 发布线程（`ZW_RENDERER_COMPOSITOR_THREAD=1`）。
//!
//! 主线程录制完成后将 `CompositorFrame` 提交到队列，由专用线程执行 PipeTransport
//! 写入，避免大图元序列化阻塞 layout/JS 路径。

use std::io::{self, Write};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};

use zero_protocol::transport::PipeTransport;
use zero_protocol::{IpcChannel, IpcMessage};

/// 是否启用 compositor 发布线程（默认关；`=1` 启用）。
pub fn compositor_publish_threading_enabled() -> bool {
    std::env::var("ZW_RENDERER_COMPOSITOR_THREAD").is_ok_and(|v| v == "1")
}

/// 共享 stdout/pipe writer（主线程与发布线程各持一个 `PipeTransport`）。
#[derive(Clone)]
pub struct SharedWriter(Arc<Mutex<Box<dyn Write + Send>>>);

impl SharedWriter {
    /// 包装已有 writer（`Arc` 可在主线程 transport 与发布线程间共享）。
    pub fn new(inner: Box<dyn Write + Send>) -> (Self, Arc<Mutex<Box<dyn Write + Send>>>) {
        let arc = Arc::new(Mutex::new(inner));
        (Self(Arc::clone(&arc)), arc)
    }
}

impl Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .map_err(|_| io::Error::other("SharedWriter lock poisoned"))?
            .write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0
            .lock()
            .map_err(|_| io::Error::other("SharedWriter lock poisoned"))?
            .flush()
    }
}

/// 异步 compositor IPC 发布队列。
pub struct CompositorPublishThread {
    tx: Option<mpsc::SyncSender<IpcMessage>>,
    join: Option<JoinHandle<()>>,
}

impl CompositorPublishThread {
    /// 启动发布线程；`writer` 须与主线程 `PipeTransport` 共享同一底层 writer。
    pub fn spawn(writer: Arc<Mutex<Box<dyn Write + Send>>>) -> Self {
        let (tx, rx) = mpsc::sync_channel::<IpcMessage>(8);
        let join = thread::Builder::new()
            .name("zero-compositor-publish".into())
            .spawn(move || {
                let mut transport = PipeTransport::new(io::empty(), SharedWriter(Arc::clone(&writer)));
                while let Ok(msg) = rx.recv() {
                    if transport.send(msg).is_err() {
                        break;
                    }
                }
            })
            .expect("compositor publish thread spawn");
        Self {
            tx: Some(tx),
            join: Some(join),
        }
    }

    /// 非阻塞提交；队列满时回退为 `false`（调用方可同步发送）。
    pub fn try_enqueue(&self, msg: IpcMessage) -> bool {
        self.tx.as_ref().is_some_and(|tx| tx.try_send(msg).is_ok())
    }
}

impl Drop for CompositorPublishThread {
    fn drop(&mut self) {
        self.tx.take();
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn publish_thread_delivers_message_to_shared_writer() {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let writer: Box<dyn Write + Send> = Box::new(SharedBuf(Arc::clone(&buf)));
        let arc = Arc::new(Mutex::new(writer));
        let pt = CompositorPublishThread::spawn(Arc::clone(&arc));
        let msg = IpcMessage {
            id: 1,
            kind: zero_protocol::IpcMessageKind::RequestFrame,
        };
        assert!(pt.try_enqueue(msg));
        drop(pt);
        assert!(!buf.lock().unwrap().is_empty());
    }

    #[derive(Clone)]
    struct SharedBuf(Arc<Mutex<Vec<u8>>>);

    impl Write for SharedBuf {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
}
