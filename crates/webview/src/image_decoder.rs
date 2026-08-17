//! 图像解码进程代理（D1）— webview 侧的解码分发。
//!
//! Browser renderer 显式启用隔离模式时，
//! PNG/JPEG/WebP 解码转发到独立 `zero-image-decoder` 进程（stdin/stdout
//! 管道 + bincode IPC），隔离编解码器漏洞（对照 Ladybird ImageDecoder 进程）。
//!
//! 降级路径：
//!   - 未启用隔离模式 / 非栅格字节（SVG 等）→ 进程内解码（SVG 依赖资源加载）
//!   - 隔离进程不可用或崩溃 → 返回资源加载错误；不得绕过隔离边界。

use std::path::{Path, PathBuf};
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

fn image_decoder_candidates_near_executable(exe: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    #[cfg(target_os = "macos")]
    for contents_dir in exe
        .ancestors()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("Contents"))
    {
        candidates.push(
            contents_dir
                .join("Frameworks")
                .join("ZeroBrowser Helper (Image Decoder).app")
                .join("Contents")
                .join("MacOS")
                .join("ZeroBrowser Helper (Image Decoder)"),
        );
    }

    if let Some(parent) = exe.parent() {
        candidates.push(parent.join(image_decoder_binary_filename()));
        if let Some(grandparent) = parent.parent() {
            candidates.push(grandparent.join(image_decoder_binary_filename()));
        }
    }
    candidates
}

/// 解析 zero-image-decoder 可执行文件路径。
///
/// 与 renderer（process_backend）/ compositor（compositor_client）同模式——
/// macOS 发布包优先使用独立 Helper app，其他平台及本地构建使用宿主同目录：
/// 查找顺序：`ZW_IMAGE_DECODER_BIN` → `CARGO_BIN_EXE_zero-image-decoder` →
/// macOS Helper app → current_exe 同目录（测试二进制 `target/debug/deps/` 上溯
/// `target/debug/`）→ PATH 兜底。
fn resolve_image_decoder_bin() -> PathBuf {
    if let Some(bin) = zero_runtime_config::optional_path("ZW_IMAGE_DECODER_BIN") {
        return bin;
    }
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_zero-image-decoder")
        && Path::new(&path).is_file()
    {
        return PathBuf::from(path);
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(candidate) = image_decoder_candidates_near_executable(&exe)
            .into_iter()
            .find(|path| path.is_file())
    {
        return candidate;
    }
    PathBuf::from(image_decoder_binary_filename())
}

/// image-decoder 子进程代理。
struct ImageDecoderProxy {
    _child: Child,
    transport: PipeTransport<std::process::ChildStdout, std::process::ChildStdin>,
    next_id: u64,
    /// 进程已失效（崩溃/通道断开）——不再尝试，直接返回资源加载错误。
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
static PROCESS_ISOLATION_ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// 为 browser renderer 启用独立图像解码进程。
///
/// 嵌入式 WebView 默认在宿主进程内工作；该函数只应由受控的 browser renderer
/// 调用，且一旦启用不会在进程生命周期内关闭。
pub fn enable_isolated_image_decoder() {
    PROCESS_ISOLATION_ENABLED.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// 是否由 browser renderer 启用独立图像解码进程。
fn proxy_enabled() -> bool {
    PROCESS_ISOLATION_ENABLED.load(std::sync::atomic::Ordering::Relaxed)
}

/// 解码图像字节（webview 侧统一入口；D1 分发）。
///
/// - renderer 隔离模式：栅格图像走 image-decoder 进程
/// - 嵌入式 WebView：进程内解码
/// - SVG/data URI 始终进程内解码
pub fn decode_image(bytes: &[u8]) -> Result<ImageData, String> {
    if !proxy_enabled() || !is_raster_image_bytes(bytes) {
        return decode_inline(bytes);
    }
    if SPAWN_FAILED.load(std::sync::atomic::Ordering::Relaxed) {
        return Err("image-decoder 进程不可用".to_string());
    }

    let mut guard = PROXY.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        match ImageDecoderProxy::spawn() {
            Some(proxy) => *guard = Some(proxy),
            None => {
                SPAWN_FAILED.store(true, std::sync::atomic::Ordering::Relaxed);
                tracing::warn!("image-decoder 进程 spawn 失败");
                return Err("无法启动 image-decoder 进程".to_string());
            }
        }
    }
    let Some(proxy) = guard.as_mut() else {
        return Err("image-decoder 进程不可用".to_string());
    };
    proxy.decode(bytes)
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

    #[test]
    fn local_build_uses_sibling_image_decoder() {
        let executable = Path::new("/workspace/target/release/zero-renderer");
        let candidates = image_decoder_candidates_near_executable(executable);

        assert_eq!(
            candidates.first(),
            Some(&PathBuf::from("/workspace/target/release").join(image_decoder_binary_filename()))
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_renderer_finds_image_decoder_helper_in_outer_app() {
        let executable = Path::new(
            "/Applications/ZeroBrowser.app/Contents/Frameworks/ZeroBrowser Helper (Renderer).app/Contents/MacOS/ZeroBrowser Helper (Renderer)",
        );
        let candidates = image_decoder_candidates_near_executable(executable);

        assert!(candidates.contains(&PathBuf::from(
            "/Applications/ZeroBrowser.app/Contents/Frameworks/ZeroBrowser Helper (Image Decoder).app/Contents/MacOS/ZeroBrowser Helper (Image Decoder)"
        )));
    }

    fn set_process_isolation(on: bool, bin: Option<&std::path::Path>) {
        // 重置 spawn 失败缓存，保证本测试的代理路径不被先前测试的
        // spawn 失败短路（真实子进程优先，失败则回退进程内）。
        SPAWN_FAILED.store(false, std::sync::atomic::Ordering::Relaxed);
        // edition 2024：env 修改为 unsafe（测试单线程环境无并发，安全）
        unsafe {
            PROCESS_ISOLATION_ENABLED.store(on, std::sync::atomic::Ordering::Relaxed);
            match bin {
                Some(b) => std::env::set_var("ZW_IMAGE_DECODER_BIN", b),
                None => std::env::remove_var("ZW_IMAGE_DECODER_BIN"),
            }
        }
    }

    #[test]
    fn non_raster_svg_goes_inline() {
        // SVG（依赖资源加载）不启代理，进程内解码（C4 分发验证）
        set_process_isolation(true, None);
        let svg = b"<svg xmlns='http://www.w3.org/2000/svg' width='2' height='2'/>";
        let img = decode_image(svg).expect("SVG 应进程内解码成功");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        set_process_isolation(false, None);
    }

    #[test]
    fn proxy_mode_decodes_png_correctly() {
        // 代理模式（env 开）下 decode_image 全链路像素正确。
        // 注意：PROXY 为进程级 static（跨测试共享），本测试不依赖
        // 核心验证是隔离模式下的端到端像素正确性；进程不可用不会回退到
        // 进程内解码。
        set_process_isolation(true, None);

        // 真实二进制（同 target 目录）优先走代理。
        let exe = std::env::current_exe().expect("current_exe");
        if let Some(bin) = exe
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join(image_decoder_binary_filename()))
            .filter(|p| p.exists())
        {
            set_process_isolation(true, Some(&bin));
        }

        let img = decode_image(&test_png()).expect("代理模式解码成功");
        assert_eq!(img.width, 1);
        assert_eq!(img.height, 1);
        // 1x1 红色 PNG（alpha=127）：红色通道 255 即像素正确
        assert_eq!(img.pixels[0], 255, "R 通道应为 255");
        set_process_isolation(false, None);
    }

    #[test]
    fn embedded_webview_uses_inline_decode() {
        set_process_isolation(false, None);
        let img = decode_image(&test_png()).expect("进程内解码成功");
        assert_eq!(img.width, 1);
        assert_eq!(img.pixels[0], 255, "R 通道应为 255");
    }
}
