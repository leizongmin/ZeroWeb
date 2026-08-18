//! C3 GPU 隔离策略：页面 wgpu 上下文仅 compositor 进程持有。

#[test]
fn renderer_crate_has_no_gpu_raster_imports() {
    // renderer 只发布 PaintSnapshot；GPU 光栅在 zero-compositor（ZW_COMPOSITOR_GPU=1）。
    let src = include_str!("runtime.rs");
    assert!(
        !src.contains("GpuRenderer"),
        "renderer 不得直接持有 GpuRenderer（C3 隔离）"
    );
    assert!(!src.contains("wgpu::"), "renderer 不得直接依赖 wgpu（C3 隔离）");
}
