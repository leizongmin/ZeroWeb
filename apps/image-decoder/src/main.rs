//! ZeroWeb 图像解码进程（D1）— 独立进程解码 PNG/JPEG/WebP。
//!
//! 由渲染进程（apps/renderer 内 webview 的 ImageDecoderProxy）经
//! stdin/stdout 管道 spawn（`--type=image-decoder --instance-id=N`，
//! 与 renderer 同款零协议：bincode 序列化 IpcMessage）。职责：
//!   - 接收 `ImageDecodeRequest`（字节 + mime）
//!   - 调 zero-render-foundation 的解码器（与进程内路径同一实现，保证一致）
//!   - 返回 `ImageDecodeResult`（RGBA + 尺寸或错误）
//!
//! 动机：编解码器处理不可信输入（畸形图片），独立进程隔离解码器漏洞
//! （对照 Ladybird ImageDecoder 进程）。
//!
//! 注意：SVG 解码依赖资源加载（字体等），保持在渲染进程内完成，
//! 本进程仅处理 PNG/JPEG/WebP（mime 显式分派，见 webview 侧）。

use zero_protocol::message::{ImageDecodeParams, ImageDecodeResultParams, IpcMessage, IpcMessageKind};
use zero_protocol::transport::stdio_transport;
use zero_protocol::{IpcChannel, is_disconnected_channel_message};

use std::io;

/// 解码一份图像字节（与 webview 进程内路径同一实现）。
fn decode(mime: &str, bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), String> {
    let img = zero_render_foundation::image_cache::decode_image_bytes(bytes)
        .map_err(|e| format!("decode failed ({mime}): {e}"))?;
    Ok((img.width, img.height, img.pixels))
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .with_writer(io::stderr)
        .init();

    let mut transport = stdio_transport().unwrap_or_else(|e| panic!("image-decoder: stdio transport init: {e}"));

    tracing::info!("image-decoder: 就绪，等待解码请求");

    loop {
        let msg: IpcMessage = match transport.recv() {
            Ok(m) => m,
            Err(e) => {
                if is_disconnected_channel_message(&e.to_string()) {
                    tracing::info!("image-decoder: 通道关闭，退出");
                    break;
                }
                tracing::warn!("image-decoder: 读取失败: {e}");
                continue;
            }
        };

        match msg.kind {
            IpcMessageKind::ImageDecodeRequest(ImageDecodeParams {
                request_id,
                mime,
                bytes,
            }) => {
                let (width, height, rgba, error) = match decode(&mime, &bytes) {
                    Ok((w, h, rgba)) => (w, h, rgba, None),
                    Err(e) => (0, 0, Vec::new(), Some(e)),
                };
                let resp = IpcMessage {
                    id: msg.id,
                    kind: IpcMessageKind::ImageDecodeResult(ImageDecodeResultParams {
                        request_id,
                        width,
                        height,
                        rgba,
                        error,
                    }),
                };
                if let Err(e) = transport.send(resp) {
                    tracing::warn!("image-decoder: 响应写入失败: {e}");
                    break;
                }
            }
            _ => {
                tracing::warn!("image-decoder: 忽略未知消息");
            }
        }
    }
}
