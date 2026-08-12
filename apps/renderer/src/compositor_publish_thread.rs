//! RFC 4.1：renderer compositor 帧 IPC 发布线程（默认启用；`ZW_RENDERER_COMPOSITOR_THREAD=0` 禁用）。
//!
//! 主线程录制完成后将 `CompositorFrame` 提交到队列，由专用线程执行 PipeTransport
//! 写入，避免大图元序列化阻塞 layout/JS 路径。
//!
//! R3254 改造（审查修复）：
//! - L1 保序：单一 FIFO 队列 + 帧尾合并——帧与 regular 消息（Title/Url/LoadComplete/
//!   DispatchResult/FocusOwnerChanged）严格按入队序发送；帧仅在**队尾**时被新帧替换
//!   （latest-wins 保留），不越过其后入队的 regular 消息。
//! - M6 死亡感知：worker 因写错误退出时 `close()` mailbox，入队返回 false → 主线程
//!   回退同步发送（不再静默吞帧）。
//! - M1 保守标记：worker 实际写出成功后回传帧携带的图片 key 列表，主线程 drain 后
//!   标记 sent——被 latest-wins 替换的帧不标记（下帧重传像素，无害），永不丢失。

use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

use zero_protocol::message::IpcMessageKind;
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
        // R3254-L2：失败路径也清空本帧缓冲——否则吞错后重试会从头重复写入残留字节
        // （IPC 流损坏）。清空即丢弃本帧（写失败通常意味着连接损坏，调用方应退出/回退）。
        let result = (|| {
            let mut writer = self
                .inner
                .lock()
                .map_err(|_| io::Error::other("SharedWriter lock poisoned"))?;
            writer.write_all(&self.frame)?;
            writer.flush()
        })();
        self.frame.clear();
        result
    }
}

/// 异步 compositor IPC 发布队列。
pub struct CompositorPublishThread {
    mailbox: Arc<LatestFrameMailbox>,
    /// R3254-M1：worker 实际写出成功后回传的图片 key 列表（主线程 drain 后标记 sent）。
    sent_rx: mpsc::Receiver<Vec<u64>>,
    join: Option<JoinHandle<()>>,
}

#[derive(Default)]
struct MailboxState {
    queue: VecDeque<IpcMessage>,
    closed: bool,
}

#[derive(Default)]
struct LatestFrameMailbox {
    state: Mutex<MailboxState>,
    ready: Condvar,
}

fn is_frame_message(message: &IpcMessage) -> bool {
    matches!(
        message.kind,
        IpcMessageKind::ViewPainted(_) | IpcMessageKind::CompositorFrame { .. }
    )
}

impl LatestFrameMailbox {
    /// 帧入队：队尾是帧则替换（latest-wins），否则 push。closed 时返回 false。
    fn enqueue_frame(&self, message: IpcMessage) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        if state.closed {
            return false;
        }
        if let Some(tail) = state.queue.back_mut()
            && is_frame_message(tail)
        {
            *tail = message;
        } else {
            state.queue.push_back(message);
        }
        self.ready.notify_one();
        true
    }

    /// regular 消息入队（FIFO，恒 push_back）。closed 时返回 false。
    fn enqueue_regular(&self, message: IpcMessage) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        if state.closed {
            return false;
        }
        state.queue.push_back(message);
        self.ready.notify_one();
        true
    }

    fn receive(&self) -> Option<IpcMessage> {
        let mut state = self.state.lock().ok()?;
        loop {
            if let Some(message) = state.queue.pop_front() {
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

/// 提取帧消息携带的图片 payload key 列表（M1 回传用）；非帧消息返回空。
fn frame_image_keys(message: &IpcMessage) -> Vec<u64> {
    match &message.kind {
        IpcMessageKind::ViewPainted(paint) | IpcMessageKind::CompositorFrame { paint, .. } => {
            paint.image_payloads.iter().map(|payload| payload.image_key).collect()
        }
        _ => Vec::new(),
    }
}

impl CompositorPublishThread {
    /// 启动发布线程；`writer` 须与主线程 `PipeTransport` 共享同一底层 writer。
    pub fn spawn(writer: Arc<Mutex<Box<dyn Write + Send>>>) -> Self {
        let mailbox = Arc::new(LatestFrameMailbox::default());
        let worker_mailbox = Arc::clone(&mailbox);
        let (sent_tx, sent_rx) = mpsc::channel();
        let join = thread::Builder::new()
            .name("zero-compositor-publish".into())
            .spawn(move || {
                let mut transport = PipeTransport::new(io::empty(), SharedWriter::from_arc(Arc::clone(&writer)));
                while let Some(msg) = worker_mailbox.receive() {
                    let keys = frame_image_keys(&msg);
                    if transport.send(msg).is_err() {
                        // R3254-M6：写失败（peer 断开）——close mailbox 让主线程入队失败，
                        // 回退同步发送（否则帧被无限静默吞掉且无信号）。
                        worker_mailbox.close();
                        break;
                    }
                    // R3254-M1：实际写出成功后才回传图片 key——被 latest-wins 替换的帧
                    // 从未发送、不产生回传，其像素下帧重传（保守，正确）。
                    if !keys.is_empty() && sent_tx.send(keys).is_err() {
                        // 主线程已退出：继续发送剩余消息或退出均可（进程将结束）。
                        break;
                    }
                }
            })
            .expect("compositor publish thread spawn");
        Self {
            mailbox,
            sent_rx,
            join: Some(join),
        }
    }

    /// 非阻塞提交帧；尚未发送的旧帧在队尾时被新帧替换（R3254-L1 帧尾合并）。
    /// 返回 false 表示发布线程已死亡/关闭——调用方应回退同步发送。
    pub fn try_enqueue(&self, msg: IpcMessage) -> bool {
        self.mailbox.enqueue_frame(msg)
    }

    /// 非阻塞提交 regular 消息（FIFO，与帧严格保序——R3254-L1）。
    /// 返回 false 表示发布线程已死亡/关闭——调用方应回退同步发送。
    pub fn enqueue_regular(&self, msg: IpcMessage) -> bool {
        self.mailbox.enqueue_regular(msg)
    }

    /// R3254-M1：drain 发布线程已实际写出成功的帧携带的图片 key 列表（主线程据此标记 sent）。
    pub fn drain_sent_image_keys(&self) -> Vec<u64> {
        let mut out = Vec::new();
        while let Ok(keys) = self.sent_rx.try_recv() {
            out.extend(keys);
        }
        out
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
            assert!(mailbox.enqueue_frame(IpcMessage {
                id,
                kind: zero_protocol::IpcMessageKind::ViewPainted(Default::default()),
            }));
        }
        assert_eq!(mailbox.receive().map(|message| message.id), Some(3));
        mailbox.close();
        assert!(mailbox.receive().is_none());
    }

    /// R3254-L1：regular 消息插在帧之间时不被后续帧替换——严格 FIFO 保序。
    #[test]
    fn frames_between_regular_messages_stay_fifo() {
        let mailbox = LatestFrameMailbox::default();
        let frame = |id| IpcMessage {
            id,
            kind: zero_protocol::IpcMessageKind::ViewPainted(Default::default()),
        };
        let regular = |id| IpcMessage {
            id,
            kind: zero_protocol::IpcMessageKind::TitleChanged("t".into()),
        };
        assert!(mailbox.enqueue_frame(frame(1)));
        assert!(mailbox.enqueue_regular(regular(2)));
        assert!(mailbox.enqueue_frame(frame(3)));
        // 队尾是 regular——帧 3 不能替换帧 1，只能追加。
        assert_eq!(mailbox.receive().map(|m| m.id), Some(1));
        assert_eq!(mailbox.receive().map(|m| m.id), Some(2));
        assert_eq!(mailbox.receive().map(|m| m.id), Some(3));
    }

    /// R3254-M1：worker 实际写出成功后回传帧携带的图片 key（主线程据此标记 sent）。
    #[test]
    fn sent_keys_reported_after_actual_write() {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let writer: Box<dyn Write + Send> = Box::new(SharedBuf(Arc::clone(&buf)));
        let pt = CompositorPublishThread::spawn(Arc::new(Mutex::new(writer)));
        let frame = IpcMessage {
            id: 1,
            kind: zero_protocol::IpcMessageKind::ViewPainted(Box::new(
                zero_protocol::paint_snapshot::PaintSnapshotParams {
                    image_payloads: vec![zero_protocol::IpcImagePayload {
                        image_key: 7,
                        width: 1,
                        height: 1,
                        rgba: vec![0, 0, 0, 255],
                    }],
                    ..Default::default()
                },
            )),
        };
        assert!(pt.try_enqueue(frame));
        // 等待 worker 写出并回传（最多 2s）。
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            let sent = pt.drain_sent_image_keys();
            if sent.contains(&7) {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "worker 写出后必须回传图片 key");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        drop(pt);
    }

    /// R3254-M6：worker 因写失败死亡后，入队返回 false（主线程据此回退同步发送）。
    #[test]
    fn enqueue_fails_after_worker_death() {
        struct FailWriter;
        impl Write for FailWriter {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("boom"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let pt = CompositorPublishThread::spawn(Arc::new(Mutex::new(Box::new(FailWriter))));
        let msg = || IpcMessage {
            id: 1,
            kind: zero_protocol::IpcMessageKind::RequestFrame,
        };
        // 首次入队可能抢在 worker 死亡前成功；循环直到失败（worker 已 close mailbox）。
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while pt.try_enqueue(msg()) {
            assert!(std::time::Instant::now() < deadline, "worker 死亡后入队必须返回 false");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
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
