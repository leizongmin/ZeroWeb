//! Browser 侧 Linux dma-buf / memfd 导入（P0：mmap 直传 wgpu，跳过 Vec/ImageCache）。

use std::os::fd::AsRawFd;
use std::sync::Arc;

use wgpu::Device;

use super::texture_export::{DRM_FORMAT_ABGR8888, ExportedGpuFrame};

/// 是否启用 Browser GPU dma-buf 导入（`ZW_BROWSER_GPU_DMABUF_IMPORT=1`）。
pub fn browser_gpu_dmabuf_import_enabled() -> bool {
    std::env::var("ZW_BROWSER_GPU_DMABUF_IMPORT").is_ok_and(|v| v == "1")
}

/// 从 compositor fd 创建 wgpu 纹理（mmap → queue.write_texture，无中间 Vec）。
pub fn try_import_linear_dmabuf(
    device: &Arc<Device>,
    queue: &Arc<wgpu::Queue>,
    export: &ExportedGpuFrame,
) -> Result<wgpu::Texture, String> {
    if export.drm_modifier != 0 {
        return Err(format!("非线性 modifier {} 尚未支持", export.drm_modifier));
    }
    if export.drm_fourcc != DRM_FORMAT_ABGR8888 {
        return Err(format!("fourcc {:#x} 尚未支持", export.drm_fourcc));
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
    let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, expected) };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("compositor-import"),
        size: wgpu::Extent3d {
            width: export.width.max(1),
            height: export.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        slice,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(export.stride),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width: export.width.max(1),
            height: export.height.max(1),
            depth_or_array_layers: 1,
        },
    );
    unsafe {
        libc::munmap(ptr, expected);
    }
    Ok(texture)
}
