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
