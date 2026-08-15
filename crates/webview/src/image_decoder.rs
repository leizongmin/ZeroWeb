//! 图像解码进程代理（D1）— webview 侧的解码分发。
//!
//! 多进程模式（默认启用，env `ZW_IMAGE_DECODER_PROCESS=0` 关闭）下，
//! PNG/JPEG/WebP 解码转发到独立 `zero-image-decoder` 进程（stdin/stdout
//! 管道 + bincode IPC），隔离编解码器漏洞（对照 Ladybird ImageDecoder 进程）。
//!
//! 降级路径：
//!   - env 关闭 / 非栅格字节（SVG 等）→ 进程内解码（SVG 依赖资源加载）
//!   - 代理 spawn 失败或解码中进程崩溃 → 进程内解码回退（fail-open，
//!     保证图像加载不被多进程路径阻断）

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

/// image-decoder 二进制文件名（平台后缀）。
fn image_decoder_binary_filename() -> &'static str {
    #[cfg(windows)]
    {
        "zero-image-decoder.exe"
    }
    #[cfg(not(windows))]
    {
        "zero-image-decoder"
    }
}

/// 解析 zero-image-decoder 可执行文件路径。
///
/// 与 renderer（process_backend）/ compositor（compositor_client）同模式——
/// 打包产物中子进程与宿主二进制同目录，裸名走 PATH 找不到：
/// 查找顺序：`ZW_IMAGE_DECODER_BIN` → `CARGO_BIN_EXE_zero-image-decoder` →
/// current_exe 同目录（测试二进制 `target/debug/deps/` 上溯 `target/debug/`）→
/// PATH 兜底。
fn resolve_image_decoder_bin() -> std::path::PathBuf {
    if let Some(bin) = zero_runtime_config::optional_path("ZW_IMAGE_DECODER_BIN") {
        return bin;
    }
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_zero-image-decoder")
        && std::path::Path::new(&path).is_file()
    {
        return std::path::PathBuf::from(path);
    }
    let name = image_decoder_binary_filename();
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        let candidate = parent.join(name);
        if candidate.is_file() {
            return candidate;
        }
        if let Some(grandparent) = parent.parent()
            && grandparent.join(name).is_file()
        {
            return grandparent.join(name);
        }
    }
    std::path::PathBuf::from(name)
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
        let bin = resolve_image_decoder_bin();
        let mut cmd = Command::new(&bin);
        for arg in child_process_args(ProcessRole::ImageDecoder, 0) {
            cmd.arg(arg);
        }
        // Windows：阻止子进程分配控制台窗口（双保险：即使子系统是 CUI 也不弹窗；
        // 同时不影响 stdin/stdout/stderr 管道继承）。与 zero-protocol spawn 同款。
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
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

/// spawn 已失败（如宿主环境无 zero-image-decoder 可执行文件）——不再重试，
/// 后续请求直接进程内解码，避免每次栅格图解码都重复 spawn 尝试与告警。
static SPAWN_FAILED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// 多进程解码是否启用（默认开；env `ZW_IMAGE_DECODER_PROCESS=0`/`false` 关闭）。
fn proxy_enabled() -> bool {
    // 单测默认关闭（仅显式 `=1` 启用，与 ZW_RENDER_THREAD 的测试语义一致）：
    // 测试进程无法保证 zero-image-decoder 二进制已构建，保持既有测试确定性。
    #[cfg(not(test))]
    {
        zero_runtime_config::enabled_by_default("ZW_IMAGE_DECODER_PROCESS")
    }
    #[cfg(test)]
    {
        zero_runtime_config::enabled_when_true("ZW_IMAGE_DECODER_PROCESS")
    }
}

/// 解码图像字节（webview 侧统一入口；D1 分发）。
///
/// - 默认：栅格图像走 image-decoder 进程
/// - `ZW_IMAGE_DECODER_PROCESS=0`：进程内解码
/// - SVG/data URI/降级路径回退进程内
pub fn decode_image(bytes: &[u8]) -> Result<ImageData, String> {
    if !proxy_enabled() || !is_raster_image_bytes(bytes) {
        return decode_inline(bytes);
    }
    if SPAWN_FAILED.load(std::sync::atomic::Ordering::Relaxed) {
        return decode_inline(bytes);
    }

    let mut guard = PROXY.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        match ImageDecoderProxy::spawn() {
            Some(proxy) => *guard = Some(proxy),
            None => {
                SPAWN_FAILED.store(true, std::sync::atomic::Ordering::Relaxed);
                tracing::warn!("image-decoder 进程 spawn 失败，后续图像回退进程内解码");
                return decode_inline(bytes);
            }
        }
    }
    let Some(proxy) = guard.as_mut() else {
        return decode_inline(bytes);
    };
    proxy.decode(bytes).or_else(|e| {
        tracing::warn!("image-decoder 进程解码失败，回退进程内: {e}");
        decode_inline(bytes)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webview::base64_decode;

    /// 1x1 红色 PNG（最小合法 PNG）。
    const ONE_PX_RED_PNG_B64: &str =
        "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

    fn test_png() -> Vec<u8> {
        base64_decode(ONE_PX_RED_PNG_B64).expect("测试 PNG 解码")
    }

    fn set_env_process(on: bool, bin: Option<&std::path::Path>) {
        // 重置 spawn 失败缓存，保证本测试的代理路径不被先前测试的
        // spawn 失败短路（真实子进程优先，失败则回退进程内）。
        SPAWN_FAILED.store(false, std::sync::atomic::Ordering::Relaxed);
        // edition 2024：env 修改为 unsafe（测试单线程环境无并发，安全）
        unsafe {
            if on {
                std::env::set_var("ZW_IMAGE_DECODER_PROCESS", "1");
            } else {
                std::env::remove_var("ZW_IMAGE_DECODER_PROCESS");
            }
            match bin {
                Some(b) => std::env::set_var("ZW_IMAGE_DECODER_BIN", b),
                None => std::env::remove_var("ZW_IMAGE_DECODER_BIN"),
            }
        }
    }

    #[test]
    fn non_raster_svg_goes_inline() {
        // SVG（依赖资源加载）不启代理，进程内解码（C4 分发验证）
        set_env_process(true, None);
        let svg = b"<svg xmlns='http://www.w3.org/2000/svg' width='2' height='2'/>";
        let img = decode_image(svg).expect("SVG 应进程内解码成功");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        set_env_process(false, None);
    }

    #[test]
    fn proxy_mode_decodes_png_correctly() {
        // 代理模式（env 开）下 decode_image 全链路像素正确。
        // 注意：PROXY 为进程级 static（跨测试共享），本测试不依赖
        // spawn 成功/失败的具体路径——核心验证是「代理模式启用时
        // 解码结果仍正确」；fail-open 回退由代码路径（match None →
        // decode_inline）保证，并有 apps/image-decoder 集成测试兜底。
        set_env_process(true, None);

        // 真实二进制（同 target 目录）优先走代理；未构建时 spawn 失败
        // 回退进程内——两条路径都应产出正确像素。
        let exe = std::env::current_exe().expect("current_exe");
        if let Some(bin) = exe
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join(image_decoder_binary_filename()))
            .filter(|p| p.exists())
        {
            set_env_process(true, Some(&bin));
        }

        let img = decode_image(&test_png()).expect("代理模式解码成功");
        assert_eq!(img.width, 1);
        assert_eq!(img.height, 1);
        // 1x1 红色 PNG（alpha=127）：红色通道 255 即像素正确
        assert_eq!(img.pixels[0], 255, "R 通道应为 255");
        set_env_process(false, None);
    }

    #[test]
    fn env_off_uses_inline_decode() {
        // 单测构建默认关闭（cfg(test) 语义）；env 未设置 → 进程内解码
        set_env_process(false, None);
        let img = decode_image(&test_png()).expect("进程内解码成功");
        assert_eq!(img.width, 1);
        assert_eq!(img.pixels[0], 255, "R 通道应为 255");
    }
}
