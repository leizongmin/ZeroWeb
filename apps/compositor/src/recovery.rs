//! RFC §五：compositor GPU 设备丢失与恢复辅助。

use zero_render_foundation::gpu::renderer::GpuRenderer;

/// 测试/诊断：`ZW_COMPOSITOR_GPU_SIMULATE_LOST=1` 时模拟 GPU 设备丢失。
pub fn gpu_simulate_device_lost_enabled() -> bool {
    std::env::var("ZW_COMPOSITOR_GPU_SIMULATE_LOST").is_ok_and(|v| v == "1")
}

/// 若启用模拟丢失，丢弃 GPU 上下文并强制回退 CPU（返回 true）。
pub fn take_simulated_device_lost(gpu_renderer: &mut Option<GpuRenderer>) -> bool {
    if !gpu_simulate_device_lost_enabled() {
        return false;
    }
    if gpu_renderer.is_some() {
        tracing::warn!("compositor: 模拟 GPU 设备丢失，回退 CPU 光栅");
        *gpu_renderer = None;
    }
    if std::env::var("ZERO_BROWSER_PRODUCT_SMOKE").as_deref() == Ok("1") {
        tracing::info!("SMOKE_EVENT component=compositor event=gpu_device_lost_fallback");
    }
    true
}

/// R3281（#3）：真实设备丢失（wgpu set_device_lost_callback）检测——丢弃上下文
/// 强制回退 CPU（返回 true），下帧由调用方重建 GPU renderer。
pub fn take_real_device_lost(gpu_renderer: &mut Option<GpuRenderer>) -> bool {
    let lost = gpu_renderer.as_ref().is_some_and(|g| g.is_device_lost());
    if lost {
        tracing::warn!("compositor: GPU 设备丢失（真实），回退 CPU 光栅，下帧重建");
        *gpu_renderer = None;
        if std::env::var("ZERO_BROWSER_PRODUCT_SMOKE").as_deref() == Ok("1") {
            tracing::info!("SMOKE_EVENT component=compositor event=gpu_device_lost_fallback");
        }
    }
    lost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulate_lost_forces_cpu_path() {
        unsafe {
            std::env::set_var("ZW_COMPOSITOR_GPU_SIMULATE_LOST", "1");
        }
        let mut slot = Some(zero_render_foundation::gpu::renderer::GpuRenderer::new_headless(4, 4).unwrap());
        assert!(take_simulated_device_lost(&mut slot));
        assert!(slot.is_none());
        assert!(take_simulated_device_lost(&mut slot));
        unsafe {
            std::env::remove_var("ZW_COMPOSITOR_GPU_SIMULATE_LOST");
        }
    }
}

#[cfg(test)]
mod real_lost_tests {
    use super::*;

    /// R3281（#3）：真实设备丢失 → take_real_device_lost 丢弃 renderer；
    /// 重建后（新 renderer）不再丢失。
    #[test]
    fn real_device_lost_drops_and_rebuilds() {
        let mut slot = Some(GpuRenderer::new_headless(4, 4).unwrap());
        // 注入丢失（模拟 wgpu 回调置位）
        slot.as_ref().unwrap().simulate_device_lost();
        assert!(take_real_device_lost(&mut slot), "丢失后应检测并丢弃");
        assert!(slot.is_none(), "丢失后 renderer 应被丢弃");
        // 重建（下帧 new_headless）→ 不再丢失
        slot = Some(GpuRenderer::new_headless(4, 4).unwrap());
        assert!(!take_real_device_lost(&mut slot), "新 renderer 不应丢失");
        assert!(slot.is_some(), "未丢失时 renderer 保留");
    }
}
