//! IPC 传输层 — 基于管道的实际进程间通信实现。
//!
//! 使用长度前缀的二进制帧协议，通过 `std::io::{Read, Write}` 实现 IPC 消息传输。
//! 适用于父子进程间的 stdio 管道通信，也可用于 TCP socket。

use std::io::{self, Read, Write};
use std::sync::Arc;

use crate::serialize;
use crate::{IpcChannel, IpcMessage, ProtocolError};

/// 消息帧的最大长度（64 MiB），防止恶意或错误数据导致内存爆炸。
///
/// Compositor 的 UI 上传和 present 回读可能携带整张物理像素位图；高 DPI
/// 窗口已超过旧的 16 MiB 上限。64 MiB 可容纳单张 4K RGBA 帧，同时保持有界分配。
const MAX_FRAME_SIZE: usize = 64 * 1024 * 1024;

fn validate_frame_size(len: usize) -> Result<(), ProtocolError> {
    if len > MAX_FRAME_SIZE {
        return Err(ProtocolError::Channel(format!(
            "帧过大: {len} 字节（上限 {MAX_FRAME_SIZE}）"
        )));
    }
    Ok(())
}

// ── 基于管道的传输 ──────────────────────────────────────────────

/// 基于管道的 IPC 传输实现。
///
/// 使用 4 字节长度前缀 + bincode 载荷的帧协议。
/// 任何实现了 `Read + Write` 的类型都可以作为底层传输。
pub struct PipeTransport<R: Read, W: Write> {
    reader: R,
    writer: W,
}

impl<R: Read, W: Write> PipeTransport<R, W> {
    /// 创建新的管道传输。
    pub fn new(reader: R, writer: W) -> Self {
        Self { reader, writer }
    }

    /// 发送原始字节帧（4 字节 LE 长度前缀 + 载荷）。
    fn send_frame(&mut self, data: &[u8]) -> Result<(), ProtocolError> {
        validate_frame_size(data.len())?;
        let len = data.len() as u32;
        self.writer
            .write_all(&len.to_le_bytes())
            .map_err(|e| ProtocolError::Channel(format!("写入帧头失败: {e}")))?;
        self.writer
            .write_all(data)
            .map_err(|e| ProtocolError::Channel(format!("写入帧体失败: {e}")))?;
        self.writer
            .flush()
            .map_err(|e| ProtocolError::Channel(format!("flush 失败: {e}")))?;
        Ok(())
    }

    /// 接收原始字节帧。
    fn recv_frame(&mut self) -> Result<Vec<u8>, ProtocolError> {
        let mut len_buf = [0u8; 4];
        self.reader
            .read_exact(&mut len_buf)
            .map_err(|e| ProtocolError::Channel(format!("读取帧头失败: {e}")))?;
        let len = u32::from_le_bytes(len_buf) as usize;
        validate_frame_size(len)?;
        let mut data = vec![0u8; len];
        self.reader
            .read_exact(&mut data)
            .map_err(|e| ProtocolError::Channel(format!("读取帧体失败: {e}")))?;
        Ok(data)
    }
}

/// 判断通道错误消息是否表示 IPC 对端已断开（管道关闭 / Broken pipe）。
pub fn is_disconnected_channel_message(message: &str) -> bool {
    let m = message.to_ascii_lowercase();
    m.contains("failed to fill whole buffer")
        || m.contains("unexpected end of file")
        || m.contains("broken pipe")
        || m.contains("connection reset")
        || m.contains("connection aborted")
        || m.contains("os error 109")
        || m.contains("os error 232")
        || m.contains("os error 54")
        || m.contains("os error 10053")
        || m.contains("os error 10054")
        || m.contains("管道已结束")
        || m.contains("管道正在被关闭")
        || m.contains("ipc 通道已关闭")
        || m.contains("通道已关闭")
}

impl<R: Read, W: Write> IpcChannel for PipeTransport<R, W> {
    fn send(&mut self, msg: IpcMessage) -> Result<(), ProtocolError> {
        let data = serialize::serialize(&msg)?;
        self.send_frame(&data)
    }

    fn recv(&mut self) -> Result<IpcMessage, ProtocolError> {
        let data = self.recv_frame()?;
        serialize::deserialize(&data)
    }

    fn try_recv(&mut self) -> Result<Option<IpcMessage>, ProtocolError> {
        // 管道传输不支持非阻塞读取
        Err(ProtocolError::Channel("PipeTransport 不支持非阻塞接收".into()))
    }

    fn close(&mut self) {
        // 管道在 Drop 时自动关闭
    }
}

/// 从标准 I/O 构建管道传输，用于子进程端。
///
/// 子进程通过 stdin/stdout 与父进程通信。
pub fn stdio_transport() -> Result<PipeTransport<io::Stdin, io::Stdout>, ProtocolError> {
    Ok(PipeTransport::new(io::stdin(), io::stdout()))
}

// ── 共享内存通道（测试和同进程模拟）──────────────────────────────

/// 共享消息队列（VecDeque 实现 FIFO 语义）。
type SharedQueue = Arc<std::sync::Mutex<std::collections::VecDeque<IpcMessage>>>;

/// 基于 `Arc<Mutex<Vec>>` 的内存 IPC 通道，用于测试和同进程多线程模拟。
pub struct SharedMemoryChannel {
    inbox: SharedQueue,
    peer_inbox: SharedQueue,
}

impl SharedMemoryChannel {
    /// 创建新通道。
    pub fn new(inbox: SharedQueue, peer_inbox: SharedQueue) -> Self {
        Self { inbox, peer_inbox }
    }
}

impl IpcChannel for SharedMemoryChannel {
    fn send(&mut self, msg: IpcMessage) -> Result<(), ProtocolError> {
        self.peer_inbox
            .lock()
            .map_err(|e| ProtocolError::Channel(format!("锁失败: {e}")))?
            .push_back(msg);
        Ok(())
    }

    fn recv(&mut self) -> Result<IpcMessage, ProtocolError> {
        let mut inbox = self
            .inbox
            .lock()
            .map_err(|e| ProtocolError::Channel(format!("锁失败: {e}")))?;
        inbox
            .pop_front()
            .ok_or_else(|| ProtocolError::Channel("没有可用消息".into()))
    }

    fn try_recv(&mut self) -> Result<Option<IpcMessage>, ProtocolError> {
        let mut inbox = self
            .inbox
            .lock()
            .map_err(|e| ProtocolError::Channel(format!("锁失败: {e}")))?;
        Ok(inbox.pop_front())
    }

    fn close(&mut self) {
        if let Ok(mut v) = self.inbox.lock() {
            v.clear();
        }
    }
}

/// 创建一对通过共享内存连接的 IPC 通道。
///
/// 返回 `(client, server)`，client 发送的消息 server 可接收，反之亦然。
/// 适用于测试和同进程内的多线程通信模拟。
pub fn shared_channel_pair() -> (SharedMemoryChannel, SharedMemoryChannel) {
    let a_inbox: SharedQueue = Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new()));
    let b_inbox: SharedQueue = Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new()));

    let a = SharedMemoryChannel::new(a_inbox.clone(), b_inbox.clone());
    let b = SharedMemoryChannel::new(b_inbox, a_inbox);

    (a, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::IpcMessageKind;

    /// 测试帧协议的基本序列化/反序列化。
    #[test]
    fn test_frame_roundtrip() {
        let msg = IpcMessage {
            id: 42,
            kind: IpcMessageKind::Heartbeat,
        };

        let serialized = serialize::serialize(&msg).unwrap();
        let mut pipe = Vec::new();

        // 写入帧
        let len = serialized.len() as u32;
        pipe.write_all(&len.to_le_bytes()).unwrap();
        pipe.write_all(&serialized).unwrap();

        // 读回帧
        let mut reader = &pipe[..];
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf).unwrap();
        let read_len = u32::from_le_bytes(len_buf) as usize;
        let mut data = vec![0u8; read_len];
        reader.read_exact(&mut data).unwrap();

        let decoded: IpcMessage = serialize::deserialize(&data).unwrap();
        assert_eq!(decoded.id, 42);
        assert!(matches!(decoded.kind, IpcMessageKind::Heartbeat));
    }

    /// 测试 PipeTransport 读写往返。
    #[test]
    fn test_pipe_transport_roundtrip() {
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::Ok,
        };

        let serialized = serialize::serialize(&msg).unwrap();
        let mut write_buf = Vec::new();
        let len = serialized.len() as u32;
        write_buf.write_all(&len.to_le_bytes()).unwrap();
        write_buf.write_all(&serialized).unwrap();

        let mut transport = PipeTransport::new(&write_buf[..], io::empty());
        let received = transport.recv().unwrap();
        assert_eq!(received.id, 1);
        assert!(matches!(received.kind, IpcMessageKind::Ok));
    }

    /// 测试 PipeTransport 发送。
    #[test]
    fn test_pipe_transport_send() {
        let mut output = Vec::new();
        {
            let mut transport = PipeTransport::new(io::empty(), &mut output);
            transport
                .send(IpcMessage {
                    id: 99,
                    kind: IpcMessageKind::Heartbeat,
                })
                .unwrap();
        }
        // 验证输出不为空（4 字节头 + 载荷）
        assert!(output.len() > 4);
        let len = u32::from_le_bytes(output[..4].try_into().unwrap()) as usize;
        assert_eq!(output.len(), 4 + len);
    }

    /// 测试共享内存通道对双向通信。
    #[test]
    fn test_shared_channel_pair_bidirectional() {
        let (mut a, mut b) = shared_channel_pair();

        let msg_a = IpcMessage {
            id: 10,
            kind: IpcMessageKind::Heartbeat,
        };
        a.send(msg_a).unwrap();

        let received = b.recv().unwrap();
        assert_eq!(received.id, 10);
        assert!(matches!(received.kind, IpcMessageKind::Heartbeat));

        // 反向
        let msg_b = IpcMessage {
            id: 20,
            kind: IpcMessageKind::Ok,
        };
        b.send(msg_b).unwrap();

        let received = a.recv().unwrap();
        assert_eq!(received.id, 20);
        assert!(matches!(received.kind, IpcMessageKind::Ok));
    }

    /// 测试共享通道 try_recv 在无消息时返回 None。
    #[test]
    fn test_shared_channel_try_recv_empty() {
        let (mut a, _b) = shared_channel_pair();
        let result = a.try_recv().unwrap();
        assert!(result.is_none());
    }

    /// 测试共享通道 recv 在无消息时返回错误。
    #[test]
    fn test_shared_channel_recv_empty_error() {
        let (mut a, _b) = shared_channel_pair();
        assert!(a.recv().is_err());
    }

    /// 测试帧大小超过限制时报错。
    #[test]
    fn test_frame_too_large() {
        let mut huge_buf = Vec::new();
        let huge_len: u32 = (MAX_FRAME_SIZE + 1) as u32;
        huge_buf.write_all(&huge_len.to_le_bytes()).unwrap();
        huge_buf.extend_from_slice(&[0u8; 64]); // 不需要真的分配 16MB+

        let mut transport = PipeTransport::new(&huge_buf[..], io::empty());
        let result = transport.recv();
        assert!(result.is_err());
        if let Err(ProtocolError::Channel(msg)) = result {
            assert!(msg.contains("帧过大"), "实际消息: {msg}");
        } else {
            panic!("期望 Channel 错误");
        }
    }

    #[test]
    fn compositor_ui_frame_above_legacy_limit_is_accepted() {
        const OBSERVED_COMPOSITOR_FRAME_SIZE: usize = 21_797_029;

        let payload = vec![0; OBSERVED_COMPOSITOR_FRAME_SIZE];
        let mut transport = PipeTransport::new(io::empty(), io::sink());
        transport.send_frame(&payload).unwrap();
        validate_frame_size(MAX_FRAME_SIZE).unwrap();
        assert!(validate_frame_size(MAX_FRAME_SIZE + 1).is_err());
    }

    /// 测试通道关闭后清空 inbox。
    #[test]
    fn test_close_clears_inbox() {
        let (mut a, mut b) = shared_channel_pair();
        b.send(IpcMessage {
            id: 1,
            kind: IpcMessageKind::Ok,
        })
        .unwrap();
        assert!(a.try_recv().unwrap().is_some());
        a.close();
        assert!(a.try_recv().unwrap().is_none());
    }

    /// 测试多消息发送（栈序 LIFO）。
    #[test]
    fn test_message_ordering() {
        let (mut a, mut b) = shared_channel_pair();
        for i in 0..5 {
            a.send(IpcMessage {
                id: i,
                kind: IpcMessageKind::Heartbeat,
            })
            .unwrap();
        }
        // VecDeque push_back/pop_front → FIFO
        for i in 0..5 {
            let msg = b.recv().unwrap();
            assert_eq!(msg.id, i);
        }
    }

    /// 测试 PipeTransport try_recv 返回不支持错误。
    #[test]
    fn test_pipe_try_recv_unsupported() {
        let mut transport = PipeTransport::new(io::empty(), Vec::new());
        let result = transport.try_recv();
        assert!(result.is_err());
        if let Err(ProtocolError::Channel(msg)) = result {
            assert!(msg.contains("不支持非阻塞"));
        }
    }

    #[test]
    fn disconnected_channel_message_detection() {
        assert!(is_disconnected_channel_message(
            "flush 失败: 管道正在被关闭。 (os error 232)"
        ));
        assert!(is_disconnected_channel_message(
            "写入帧体失败: 管道已结束。 (os error 109)"
        ));
        assert!(!is_disconnected_channel_message("帧过大: 999 字节"));
    }
}
