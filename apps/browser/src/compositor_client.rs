//! 合成器客户端（C2 显示接线）— browser spawn zero-compositor，
//! 转发 renderer 帧 + 拉取已合成帧。
//!
//! env `ZW_COMPOSITOR_PROCESS=1` 启用（默认关 = 零行为变更）：
//!   - browser 收 ViewPainted 时同步转发 CompositorFrame 到合成器进程
//!   - `get_frame()` 拉取最新已合成帧（front 像素）——显示消费方
//!   - 失败静默（fail-open：帧转发失败不阻断主显示通路）
//!
//! 显示切换（render_cpu 用 compositor 帧 + chrome 合成）需窗口环境
//! A/B 验证，为后续切片（RFC compositor-process-rfc C2 剩余）。

use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

use zero_protocol::message::{IpcMessage, IpcMessageKind};
use zero_protocol::paint_snapshot::PaintSnapshotParams;
use zero_protocol::transport::PipeTransport;
use zero_protocol::{IpcChannel, ProcessRole, child_process_args};

struct Client {
    _child: Child,
    transport: PipeTransport<std::process::ChildStdout, std::process::ChildStdin>,
    failed: bool,
}

impl Client {
    fn spawn() -> Option<Self> {
        let bin = std::env::var("ZW_COMPOSITOR_BIN").unwrap_or_else(|_| "zero-compositor".to_string());
        let mut cmd = Command::new(&bin);
        for arg in child_process_args(ProcessRole::ImageDecoder, 0) {
            cmd.arg(arg);
        }
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .ok()?;
        let stdout = child.stdout.take()?;
        let stdin = child.stdin.take()?;
        Some(Self {
            _child: child,
            transport: PipeTransport::new(stdout, stdin),
            failed: false,
        })
    }

    fn send_frame(&mut self, paint: PaintSnapshotParams) {
        if self.failed {
            return;
        }
        let msg = IpcMessage {
            id: 1,
            kind: IpcMessageKind::CompositorFrame(Box::new(paint)),
        };
        if let Err(e) = self.transport.send(msg) {
            self.failed = true;
            tracing::warn!("compositor 帧转发失败（已禁用代理）: {e}");
        }
    }

    /// 拉取最新已合成帧（front 像素）。失败返回 None（fail-open）。
    fn get_frame(&mut self) -> Option<(u64, u32, u32, Vec<u8>)> {
        if self.failed {
            return None;
        }
        let msg = IpcMessage {
            id: 2,
            kind: IpcMessageKind::GetCompositorFrame,
        };
        if self.transport.send(msg).is_err() {
            self.failed = true;
            return None;
        }
        match self.transport.recv() {
            Ok(resp) => match resp.kind {
                IpcMessageKind::CompositorFrameData {
                    frame_id,
                    width,
                    height,
                    rgba,
                } => Some((frame_id, width, height, rgba)),
                _ => None,
            },
            Err(_) => {
                self.failed = true;
                None
            }
        }
    }
}

static CLIENT: Mutex<Option<Client>> = Mutex::new(None);

/// 启用开关（env `ZW_COMPOSITOR_PROCESS=1`）。
pub fn enabled() -> bool {
    std::env::var("ZW_COMPOSITOR_PROCESS").is_ok_and(|v| v == "1")
}

/// 转发一帧到合成器进程（启用时）。失败静默。
pub fn forward_frame(paint: PaintSnapshotParams) {
    if !enabled() {
        return;
    }
    let mut guard = CLIENT.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        *guard = Client::spawn();
    }
    if let Some(client) = guard.as_mut() {
        client.send_frame(paint);
    }
}

/// 拉取最新已合成帧（启用时）。返回 (frame_id, width, height, rgba)。
pub fn get_frame() -> Option<(u64, u32, u32, Vec<u8>)> {
    if !enabled() {
        return None;
    }
    let mut guard = CLIENT.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        *guard = Client::spawn();
    }
    guard.as_mut()?.get_frame()
}
