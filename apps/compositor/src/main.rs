//! ZeroWeb 合成器进程（C2 骨架）— 接收渲染进程的图元帧，BackingStore 双缓冲管理。
//!
//! 对照 Ladybird 2026-05 合成器独立进程（调研报告 §3.3/§3.4）：合成与
//! backing store 管理从渲染进程移出。本骨架实现：
//!   - stdio 管道 + bincode IPC（与 image-decoder 同款）
//!   - 接收 `CompositorFrame`（PaintSnapshotParams 图元快照）
//!   - BackingStoreManager 双缓冲：写 back → swap → 保留 front（供显示消费方读取）
//!   - 回复 `CompositorFrameResult`（帧已合成确认）
//!
//! 后续切片（RFC compositor-process-rfc C2/C3）：
//!   - renderer 帧传输接线（当前 renderer 仍直发 browser）
//!   - GPU 光栅化上下文迁移（C3：wgpu 在合成器进程内）
//!   - seccomp 沙箱

use zero_protocol::message::{IpcMessage, IpcMessageKind};
use zero_protocol::transport::stdio_transport;
use zero_protocol::{IpcChannel, is_disconnected_channel_message};
use zero_render_foundation::backing_store::BackingStoreManager;

use std::io;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(io::stderr)
        .init();

    let mut transport = stdio_transport().unwrap_or_else(|e| panic!("compositor: stdio transport: {e}"));

    // 双缓冲：尺寸随首帧初始化
    let mut backing: Option<BackingStoreManager> = None;
    let mut frame_count: u64 = 0;

    tracing::info!("zero-compositor 就绪（C2 骨架：BackingStore 双缓冲）");

    loop {
        let msg: IpcMessage = match transport.recv() {
            Ok(m) => m,
            Err(e) => {
                if is_disconnected_channel_message(&e.to_string()) {
                    tracing::info!("compositor: 通道关闭，退出");
                    break;
                }
                tracing::warn!("compositor: 读取失败: {e}");
                continue;
            }
        };

        match msg.kind {
            IpcMessageKind::CompositorFrame(frame) => {
                frame_count += 1;
                let w = frame.viewport_width.max(1);
                let h = frame.viewport_height.max(1);
                let store = backing.get_or_insert_with(|| BackingStoreManager::new(w, h));
                store.resize(w, h);
                // 骨架：写 back（清空示意）→ swap → front 为最新帧（供显示消费方读取）
                store.back_mut().data.clear();
                store.back_mut().data.resize((w as usize) * (h as usize) * 4, 0);
                store.swap();
                tracing::info!("compositor: 帧 #{frame_count} 已合成（{w}x{h}），front 就绪");

                let resp = IpcMessage {
                    id: msg.id,
                    kind: IpcMessageKind::CompositorFrameResult { frame_id: frame_count },
                };
                if let Err(e) = transport.send(resp) {
                    tracing::warn!("compositor: 响应失败: {e}");
                    break;
                }
            }
            _ => {
                tracing::warn!("compositor: 忽略未知消息");
            }
        }
    }
}
