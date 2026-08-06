//! 图像解码进程代理（D1）— webview 侧的解码分发。
//!
//! 多进程模式（env `ZW_IMAGE_DECODER_PROCESS=1`）下，PNG/JPEG/WebP 解码
//! 转发到独立 `zero-image-decoder` 进程（stdin/stdout 管道 + bincode IPC），
//! 隔离编解码器漏洞（对照 Ladybird ImageDecoder 进程）。
//!
//! 降级路径：
//!   - 未启用 env → 进程内解码（现状，零行为变更）
//!   - 非栅格字节（SVG 等）→ 进程内解码（SVG 依赖资源加载）
//!   - 代理 spawn 失败或解码中进程崩溃 → 进程内解码回退（fail-open，
//!     保证图像加载不被多进程实验阻断）

use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

use zero_protocol::message::{ImageDecodeParams, ImageDecodeResultParams, IpcMessage, IpcMessageKind};
use zero_protocol::transport::PipeTransport;
use zero_protocol::{IpcChannel, ProcessRole, child_process_args};

use zero_render_foundation::image_cache::{ImageData, decode_image_bytes, is_raster_image_bytes};

/// 进程内解码（默认路径，零行为变更）。
fn decode_inline(bytes: &[u8]) -> Result<ImageData, String> {
    decode_image_bytes(bytes)
}

/// image-decoder 子进程代理。
struct ImageDecoderProxy {
    _child: Child,
    transport: PipeTransport<std::process::ChildStdout, std::process::ChildStdin>,
    next_id: u64,
    /// 进程已失效（崩溃/通道断开）——不再尝试，直接回退进程内。
    failed: bool,
}

impl ImageDecoderProxy {
    fn spawn() -> Option<Self> {
        let bin = std::env::var("ZW_IMAGE_DECODER_BIN").unwrap_or_else(|_| "zero-image-decoder".to_string());
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
            next_id: 1,
            failed: false,
        })
    }

    /// 同步解码（请求按序处理；解码通常 <100ms，第一版不做并发）。
    fn decode(&mut self, bytes: &[u8]) -> Result<ImageData, String> {
        if self.failed {
            return Err("image-decoder 进程已失效".to_string());
        }
        let request_id = self.next_id;
        self.next_id += 1;
        let msg = IpcMessage {
            id: request_id,
            kind: IpcMessageKind::ImageDecodeRequest(ImageDecodeParams {
                request_id,
                mime: String::new(),
                bytes: bytes.to_vec(),
            }),
        };
        self.transport
            .send(msg)
            .map_err(|e| format!("image-decoder send: {e}"))?;

        loop {
            let resp: IpcMessage = self.transport.recv().map_err(|e| {
                self.failed = true;
                format!("image-decoder recv: {e}")
            })?;
            if let IpcMessageKind::ImageDecodeResult(ImageDecodeResultParams {
                request_id: rid,
                width,
                height,
                rgba,
                error,
            }) = resp.kind
            {
                if rid != request_id {
                    continue; // 非本次请求的响应（不应出现，防御性跳过）
                }
                if let Some(e) = error {
                    // 解码失败多为畸形输入，不视为进程故障（进程保持存活）
                    return Err(e);
                }
                return ImageData::from_rgba(rgba, width, height);
            }
        }
    }
}

static PROXY: Mutex<Option<ImageDecoderProxy>> = Mutex::new(None);

/// 多进程解码是否启用（env `ZW_IMAGE_DECODER_PROCESS=1`）。
fn proxy_enabled() -> bool {
    std::env::var("ZW_IMAGE_DECODER_PROCESS").is_ok_and(|v| v == "1")
}

/// 解码图像字节（webview 侧统一入口；D1 分发）。
///
/// - 默认：进程内解码（与改造前完全一致）
/// - `ZW_IMAGE_DECODER_PROCESS=1`：栅格图像走 image-decoder 进程，
///   SVG/data URI/降级路径回退进程内
pub fn decode_image(bytes: &[u8]) -> Result<ImageData, String> {
    if !proxy_enabled() || !is_raster_image_bytes(bytes) {
        return decode_inline(bytes);
    }

    let mut guard = PROXY.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        *guard = ImageDecoderProxy::spawn();
    }
    match guard.as_mut() {
        Some(proxy) => proxy.decode(bytes).or_else(|e| {
            tracing::warn!("image-decoder 进程解码失败，回退进程内: {e}");
            decode_inline(bytes)
        }),
        None => {
            tracing::warn!("image-decoder 进程 spawn 失败，回退进程内");
            decode_inline(bytes)
        }
    }
}
