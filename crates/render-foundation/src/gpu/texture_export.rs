//! Linux GPU 纹理导出（RFC 4.3-S5 / P0：memfd + Vulkan OPAQUE_FD 尝试）。

use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

use super::renderer::GpuRenderer;

/// DRM fourcc `ABGR8888`（对应 wgpu `Bgra8Unorm` 线性布局）。
pub const DRM_FORMAT_ABGR8888: u32 = 0x3432_4241;

/// 导出的 GPU/线性帧缓冲（fd 由接收方关闭）。
pub struct ExportedGpuFrame {
    /// dma-buf / memfd 文件描述符。
    pub fd: OwnedFd,
    /// 宽度（像素）。
    pub width: u32,
    /// 高度（像素）。
    pub height: u32,
    /// 行 stride（字节）。
    pub stride: u32,
    /// DRM fourcc。
    pub drm_fourcc: u32,
    /// DRM modifier（线性 = 0）。
    pub drm_modifier: u64,
    /// 可选 GPU fence sync_fd（Vulkan 导出路径；当前多为 None）。
    pub sync_fd: Option<OwnedFd>,
}

/// 是否启用 compositor GPU 纹理导出（Linux 默认开；`ZW_COMPOSITOR_GPU_TEXTURE_EXPORT=0` 禁用）。
pub fn gpu_texture_export_enabled() -> bool {
    zero_protocol::compositor_gpu_texture_export_enabled()
}

/// 尝试从无头渲染目标导出 fd；Vulkan 不可用时回退 memfd 线性缓冲。
pub fn try_export_headless(gpu: &GpuRenderer) -> Result<ExportedGpuFrame, String> {
    try_export_vulkan_dma_buf(gpu).or_else(|vulkan_err| {
        tracing::debug!("vulkan dma-buf 导出不可用 ({vulkan_err})，回退 memfd");
        export_via_memfd(gpu)
    })
}

fn try_export_vulkan_dma_buf(gpu: &GpuRenderer) -> Result<ExportedGpuFrame, String> {
    // Vulkan OPAQUE_FD 导出需 wgpu-hal Instance 私有句柄；当前经 memfd 回退。
    // Browser 侧 `ZW_BROWSER_GPU_DMABUF_IMPORT=1` 走 mmap→write_texture 跳过 Vec/ImageCache。
    let _ = gpu;
    Err("vulkan OPAQUE_FD 导出待 wgpu-hal Instance 句柄公开或 wgpu 30+ dma-buf API".into())
}

pub(crate) fn export_via_memfd(gpu: &GpuRenderer) -> Result<ExportedGpuFrame, String> {
    let pixels = gpu.read_pixels().ok_or("read_pixels 失败")?;
    let (width, height) = gpu.surface_size();
    if width == 0 || height == 0 {
        return Err("无效 surface 尺寸".into());
    }
    let stride = width.saturating_mul(4);
    let expected = (stride as usize).saturating_mul(height as usize);
    if pixels.len() != expected {
        return Err(format!("像素长度不匹配: 期望 {expected}, 实际 {}", pixels.len()));
    }

    let fd = unsafe { libc::memfd_create(c"zeroweb-gpu-export".as_ptr(), libc::MFD_CLOEXEC) };
    if fd < 0 {
        return Err(format!("memfd_create 失败: {}", std::io::Error::last_os_error()));
    }
    if unsafe { libc::ftruncate(fd, expected as libc::off_t) } != 0 {
        unsafe {
            libc::close(fd);
        }
        return Err(format!("ftruncate 失败: {}", std::io::Error::last_os_error()));
    }

    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            expected,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            0,
        )
    };
    if ptr == libc::MAP_FAILED {
        unsafe {
            libc::close(fd);
        }
        return Err(format!("mmap 失败: {}", std::io::Error::last_os_error()));
    }
    unsafe {
        std::ptr::copy_nonoverlapping(pixels.as_ptr(), ptr as *mut u8, expected);
        libc::munmap(ptr, expected);
    }

    Ok(ExportedGpuFrame {
        fd: unsafe { OwnedFd::from_raw_fd(fd) },
        width,
        height,
        stride,
        drm_fourcc: DRM_FORMAT_ABGR8888,
        drm_modifier: 0,
        sync_fd: None,
    })
}

/// Browser 侧：mmap 线性 dma-buf/memfd 并拷贝 RGBA（modifier=0；legacy 路径）。
pub fn map_linear_rgba(export: &ExportedGpuFrame) -> Result<Vec<u8>, String> {
    if export.drm_modifier != 0 {
        return Err(format!("非线性 modifier {} 尚未支持", export.drm_modifier));
    }
    let expected = (export.stride as usize).saturating_mul(export.height as usize);
    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            expected,
            libc::PROT_READ,
            libc::MAP_SHARED,
            export.fd.as_raw_fd(),
            0,
        )
    };
    if ptr == libc::MAP_FAILED {
        return Err(format!("mmap 失败: {}", std::io::Error::last_os_error()));
    }
    let rgba = unsafe { std::slice::from_raw_parts(ptr as *const u8, expected) }.to_vec();
    unsafe {
        libc::munmap(ptr, expected);
    }
    Ok(rgba)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::font::{FontLoader, GlyphCache};
    use crate::geometry::Rect;
    use crate::gpu::renderer::GpuRenderer;
    use crate::primitive::{FillPrimitive, RenderPrimitives};

    #[test]
    fn memfd_export_round_trip() {
        let mut gpu = match GpuRenderer::new_headless(8, 8) {
            Ok(g) => g,
            Err(_) => return,
        };
        gpu.configure_surface(8, 8);
        let mut primitives = RenderPrimitives::new();
        primitives.fills.push(FillPrimitive {
            rect: Rect::new(0.0, 0.0, 8.0, 8.0),
            color: Color::rgb(0, 255, 0),
        });
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        gpu.render_scene_ext(&primitives.fills, &font_loader, &mut glyph_cache, &[], &[], &[]);

        let exported = export_via_memfd(&gpu).expect("export");
        let rgba = map_linear_rgba(&exported).expect("map");
        assert_eq!(rgba.len(), 8 * 8 * 4);
        assert_eq!(rgba[0], 0);
        assert_eq!(rgba[1], 255);
    }
}
