//! RFC 4.1：renderer compositor 帧 IPC 发布线程（默认启用；`ZW_RENDERER_COMPOSITOR_THREAD=0` 禁用）。
//!
//! 主线程录制完成后将 `CompositorFrame` 提交到队列，由专用线程执行 PipeTransport
//! 写入，避免大图元序列化阻塞 layout/JS 路径。

use std::io::{self, Write};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

use zero_protocol::transport::PipeTransport;
use zero_protocol::{IpcChannel, IpcMessage};

/// 是否启用 compositor 发布线程（默认开；仅精确值 `0` 禁用）。
pub fn compositor_publish_threading_enabled() -> bool {
    compositor_publish_threading_enabled_from_env(std::env::var("ZW_RENDERER_COMPOSITOR_THREAD").ok().as_deref())
}

fn compositor_publish_threading_enabled_from_env(value: Option<&str>) -> bool {
    value != Some("0")
}

/// 共享 stdout/pipe writer（主线程与发布线程各持一个 `PipeTransport`）。
pub struct SharedWriter {
    inner: Arc<Mutex<Box<dyn Write + Send>>>,
    frame: Vec<u8>,
}

impl Clone for SharedWriter {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            frame: Vec::new(),
        }
    }
}

impl SharedWriter {
    /// 包装已有 writer（`Arc` 可在主线程 transport 与发布线程间共享）。
    pub fn new(inner: Box<dyn Write + Send>) -> (Self, Arc<Mutex<Box<dyn Write + Send>>>) {
        let arc = Arc::new(Mutex::new(inner));
        (Self::from_arc(Arc::clone(&arc)), arc)
    }

    fn from_arc(inner: Arc<Mutex<Box<dyn Write + Send>>>) -> Self {
        Self {
            inner,
            frame: Vec::new(),
        }
    }
}

impl Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.frame.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut writer = self
            .inner
            .lock()
            .map_err(|_| io::Error::other("SharedWriter lock poisoned"))?;
        writer.write_all(&self.frame)?;
        writer.flush()?;
        self.frame.clear();
        Ok(())
    }
}

/// 异步 compositor IPC 发布队列。
pub struct CompositorPublishThread {
    mailbox: Arc<LatestFrameMailbox>,
    join: Option<JoinHandle<()>>,
}

#[derive(Default)]
struct MailboxState {
    pending: Option<IpcMessage>,
    closed: bool,
}

#[derive(Default)]
struct LatestFrameMailbox {
    state: Mutex<MailboxState>,
    ready: Condvar,
}

impl LatestFrameMailbox {
    fn replace(&self, message: IpcMessage) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        if state.closed {
            return false;
        }
        state.pending = Some(message);
        self.ready.notify_one();
        true
    }

    fn receive(&self) -> Option<IpcMessage> {
        let mut state = self.state.lock().ok()?;
        loop {
            if let Some(message) = state.pending.take() {
                return Some(message);
            }
            if state.closed {
                return None;
            }
            state = self.ready.wait(state).ok()?;
        }
    }

    fn close(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.closed = true;
            self.ready.notify_one();
        }
    }
}

impl CompositorPublishThread {
    /// 启动发布线程；`writer` 须与主线程 `PipeTransport` 共享同一底层 writer。
    pub fn spawn(writer: Arc<Mutex<Box<dyn Write + Send>>>) -> Self {
        let mailbox = Arc::new(LatestFrameMailbox::default());
        let worker_mailbox = Arc::clone(&mailbox);
        let join = thread::Builder::new()
            .name("zero-compositor-publish".into())
            .spawn(move || {
                let mut transport = PipeTransport::new(io::empty(), SharedWriter::from_arc(Arc::clone(&writer)));
                while let Some(msg) = worker_mailbox.receive() {
                    if transport.send(msg).is_err() {
                        break;
                    }
                }
            })
            .expect("compositor publish thread spawn");
        Self {
            mailbox,
            join: Some(join),
        }
    }

    /// 非阻塞提交；尚未发送的旧帧会被新帧替换。
    pub fn try_enqueue(&self, msg: IpcMessage) -> bool {
        self.mailbox.replace(msg)
    }
}

impl Drop for CompositorPublishThread {
    fn drop(&mut self) {
        self.mailbox.close();
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn publish_thread_defaults_on_and_exact_zero_disables_it() {
        for value in [None, Some(""), Some("1"), Some("true"), Some("01")] {
            assert!(compositor_publish_threading_enabled_from_env(value));
        }
        assert!(!compositor_publish_threading_enabled_from_env(Some("0")));
    }

    #[test]
    fn pending_frames_are_latest_wins_instead_of_fifo() {
        let mailbox = LatestFrameMailbox::default();
        for id in 1..=3 {
            assert!(mailbox.replace(IpcMessage {
                id,
                kind: zero_protocol::IpcMessageKind::RequestFrame,
            }));
        }
        assert_eq!(mailbox.receive().map(|message| message.id), Some(3));
        mailbox.close();
        assert!(mailbox.receive().is_none());
    }

    #[test]
    fn shared_writers_commit_complete_frames_atomically_on_flush() {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let inner: Box<dyn Write + Send> = Box::new(SharedBuf(Arc::clone(&buf)));
        let (mut first, arc) = SharedWriter::new(inner);
        let mut second = SharedWriter::from_arc(arc);

        first.write_all(b"head-one").unwrap();
        second.write_all(b"head-two").unwrap();
        first.write_all(b"-body-one").unwrap();
        second.write_all(b"-body-two").unwrap();
        second.flush().unwrap();
        first.flush().unwrap();

        assert_eq!(&*buf.lock().unwrap(), b"head-two-body-twohead-one-body-one");
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
