//! 渲染表面 — 管理渲染目标

use crate::geometry::Size;

/// 表面描述符 — 描述渲染目标的属性
#[derive(Debug, Clone)]
pub struct SurfaceDescriptor {
    /// 表面宽度（像素）
    pub width: u32,
    /// 表面高度（像素）
    pub height: u32,
    /// 是否支持透明
    pub transparent: bool,
}

impl SurfaceDescriptor {
    /// 创建新的表面描述符
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            transparent: false,
        }
    }

    /// 设置透明
    pub fn with_transparency(mut self) -> Self {
        self.transparent = true;
        self
    }

    /// 转换为几何尺寸
    pub fn to_size(&self) -> Size {
        Size::new(self.width as f32, self.height as f32)
    }
}

/// 帧缓冲 — CPU 侧的 RGBA 像素数据
#[derive(Debug, Clone)]
pub struct FrameBuffer {
    /// 像素数据（RGBA，行优先）
    pub data: Vec<u8>,
    /// 宽度
    pub width: u32,
    /// 高度
    pub height: u32,
}

impl FrameBuffer {
    /// 创建新的帧缓冲（全黑不透明）
    pub fn new(width: u32, height: u32) -> Self {
        let data = vec![0u8; (width * height * 4) as usize];
        Self { data, width, height }
    }

    /// 创建指定颜色填充的帧缓冲（一次 memset，免 new + clear 两遍全缓冲写）。
    pub fn new_filled(width: u32, height: u32, r: u8, g: u8, b: u8, a: u8) -> Self {
        let data = [r, g, b, a].repeat((width * height) as usize);
        Self { data, width, height }
    }

    /// 从 RGBA 数据创建帧缓冲
    pub fn from_rgba(data: Vec<u8>, width: u32, height: u32) -> Result<Self, String> {
        let expected = (width * height * 4) as usize;
        if data.len() != expected {
            return Err(format!("数据大小不匹配: 期望 {}, 实际 {}", expected, data.len()));
        }
        Ok(Self { data, width, height })
    }

    /// 获取指定位置的像素（RGBA）
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let idx = ((y * self.width + x) * 4) as usize;
        [
            self.data[idx],
            self.data[idx + 1],
            self.data[idx + 2],
            self.data[idx + 3],
        ]
    }

    /// 设置指定位置的像素（RGBA）
    pub fn set_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        let idx = ((y * self.width + x) * 4) as usize;
        self.data[idx] = rgba[0];
        self.data[idx + 1] = rgba[1];
        self.data[idx + 2] = rgba[2];
        self.data[idx + 3] = rgba[3];
    }

    /// 清除为指定颜色
    pub fn clear(&mut self, r: u8, g: u8, b: u8, a: u8) {
        for chunk in self.data.as_chunks_mut::<4>().0 {
            chunk[0] = r;
            chunk[1] = g;
            chunk[2] = b;
            chunk[3] = a;
        }
    }

    /// 尺寸
    pub fn size(&self) -> Size {
        Size::new(self.width as f32, self.height as f32)
    }

    /// 总像素数
    pub fn pixel_count(&self) -> u32 {
        self.width * self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_descriptor_new() {
        let desc = SurfaceDescriptor::new(800, 600);
        assert_eq!(desc.width, 800);
        assert_eq!(desc.height, 600);
        assert!(!desc.transparent);
    }

    #[test]
    fn test_surface_descriptor_transparent() {
        let desc = SurfaceDescriptor::new(800, 600).with_transparency();
        assert!(desc.transparent);
    }

    #[test]
    fn test_frame_buffer_new() {
        let fb = FrameBuffer::new(100, 100);
        assert_eq!(fb.width, 100);
        assert_eq!(fb.height, 100);
        assert_eq!(fb.data.len(), 100 * 100 * 4);
        // 初始全黑
        assert_eq!(fb.get_pixel(0, 0), [0, 0, 0, 0]);
    }

    #[test]
    fn test_frame_buffer_set_get_pixel() {
        let mut fb = FrameBuffer::new(10, 10);
        fb.set_pixel(5, 5, [255, 128, 0, 255]);
        assert_eq!(fb.get_pixel(5, 5), [255, 128, 0, 255]);
    }

    #[test]
    fn test_frame_buffer_clear() {
        let mut fb = FrameBuffer::new(10, 10);
        fb.clear(255, 255, 255, 255);
        assert_eq!(fb.get_pixel(0, 0), [255, 255, 255, 255]);
        assert_eq!(fb.get_pixel(9, 9), [255, 255, 255, 255]);
    }

    #[test]
    fn test_frame_buffer_from_rgba() {
        let data = vec![255u8; 10 * 10 * 4];
        let fb = FrameBuffer::from_rgba(data, 10, 10);
        assert!(fb.is_ok());
    }

    #[test]
    fn test_frame_buffer_from_rgba_wrong_size() {
        let data = vec![255u8; 100];
        let fb = FrameBuffer::from_rgba(data, 10, 10);
        assert!(fb.is_err());
    }

    #[test]
    fn test_frame_buffer_pixel_count() {
        let fb = FrameBuffer::new(10, 10);
        assert_eq!(fb.pixel_count(), 100);
    }

    #[test]
    fn test_surface_descriptor_to_size() {
        let desc = SurfaceDescriptor::new(1024, 768);
        let size = desc.to_size();
        assert_eq!(size.width, 1024.0);
        assert_eq!(size.height, 768.0);
    }

    #[test]
    fn test_frame_buffer_size() {
        let fb = FrameBuffer::new(20, 30);
        let size = fb.size();
        assert_eq!(size.width, 20.0);
        assert_eq!(size.height, 30.0);
    }

    #[test]
    fn test_frame_buffer_clear_partial() {
        let mut fb = FrameBuffer::new(5, 5);
        fb.set_pixel(0, 0, [100, 100, 100, 100]);
        fb.set_pixel(4, 4, [200, 200, 200, 200]);
        // Clear to white
        fb.clear(255, 255, 255, 255);
        // All pixels should be white now
        assert_eq!(fb.get_pixel(0, 0), [255, 255, 255, 255]);
        assert_eq!(fb.get_pixel(4, 4), [255, 255, 255, 255]);
    }

    #[test]
    fn test_surface_descriptor_builder_pattern() {
        let desc = SurfaceDescriptor::new(800, 600).with_transparency();
        assert_eq!(desc.width, 800);
        assert_eq!(desc.height, 600);
        assert!(desc.transparent);
        // Non-transparent default
        let desc2 = SurfaceDescriptor::new(640, 480);
        assert!(!desc2.transparent);
    }

    #[test]
    fn test_frame_buffer_multiple_pixel_ops() {
        let mut fb = FrameBuffer::new(3, 3);
        fb.set_pixel(0, 0, [10, 20, 30, 40]);
        fb.set_pixel(1, 1, [50, 60, 70, 80]);
        fb.set_pixel(2, 2, [90, 100, 110, 120]);
        assert_eq!(fb.get_pixel(0, 0), [10, 20, 30, 40]);
        assert_eq!(fb.get_pixel(1, 1), [50, 60, 70, 80]);
        assert_eq!(fb.get_pixel(2, 2), [90, 100, 110, 120]);
        // Unset pixel should still be black (0,0,0,0)
        assert_eq!(fb.get_pixel(0, 1), [0, 0, 0, 0]);
    }

    /// 测试 GPU 与 CPU 渲染路径对纯黑/纯白填充产生一致的像素输出
    ///
    /// GPU 路径：通过 GpuRenderer 无头模式渲染 FillPrimitive 并回读像素
    /// CPU 路径：通过 FrameBuffer::clear 填充相同颜色
    /// 纯黑 (0,0,0) 和纯白 (255,255,255) 在 sRGB 下无 gamma 偏移，
    /// 因此 GPU 与 CPU 必须产生完全相同的 RGBA 像素值。
    #[test]
    fn test_gpu_cpu_rendering_consistency_solid_fill() {
        use crate::color::Color;
        use crate::font::cache::GlyphCache;
        use crate::font::loader::FontLoader;
        use crate::gpu::renderer::GpuRenderer;
        use crate::primitive::FillPrimitive;

        let width = 16u32;
        let height = 16u32;

        // 测试纯黑
        {
            let mut cpu_fb = FrameBuffer::new(width, height);
            cpu_fb.clear(0, 0, 0, 255);

            let mut gpu_renderer = GpuRenderer::new_headless(width, height).expect("headless");
            let fills = vec![FillPrimitive {
                rect: crate::geometry::Rect::new(0.0, 0.0, width as f32, height as f32),
                color: Color::BLACK,
            }];
            let font_loader = FontLoader::new();
            let mut glyph_cache = GlyphCache::new(64);
            gpu_renderer.render_scene(&fills, &font_loader, &mut glyph_cache, &[], &[]);
            let gpu_pixels = gpu_renderer.read_pixels().expect("read_pixels");

            assert_eq!(gpu_pixels.len(), cpu_fb.data.len());
            for (x, y) in [(0, 0), (8, 8), (15, 15)] {
                let cpu_pixel = cpu_fb.get_pixel(x, y);
                let idx = ((y * width + x) * 4) as usize;
                let gpu_pixel = &gpu_pixels[idx..idx + 4];
                assert_eq!(
                    cpu_pixel,
                    [gpu_pixel[0], gpu_pixel[1], gpu_pixel[2], gpu_pixel[3]],
                    "纯黑: GPU 和 CPU 在 ({x},{y}) 处的像素应一致"
                );
            }
        }

        // 测试纯白
        {
            let mut cpu_fb = FrameBuffer::new(width, height);
            cpu_fb.clear(255, 255, 255, 255);

            let mut gpu_renderer = GpuRenderer::new_headless(width, height).expect("headless");
            let fills = vec![FillPrimitive {
                rect: crate::geometry::Rect::new(0.0, 0.0, width as f32, height as f32),
                color: Color::WHITE,
            }];
            let font_loader = FontLoader::new();
            let mut glyph_cache = GlyphCache::new(64);
            gpu_renderer.render_scene(&fills, &font_loader, &mut glyph_cache, &[], &[]);
            let gpu_pixels = gpu_renderer.read_pixels().expect("read_pixels");

            assert_eq!(gpu_pixels.len(), cpu_fb.data.len());
            for (x, y) in [(0, 0), (8, 8), (15, 15)] {
                let cpu_pixel = cpu_fb.get_pixel(x, y);
                let idx = ((y * width + x) * 4) as usize;
                let gpu_pixel = &gpu_pixels[idx..idx + 4];
                assert_eq!(
                    cpu_pixel,
                    [gpu_pixel[0], gpu_pixel[1], gpu_pixel[2], gpu_pixel[3]],
                    "纯白: GPU 和 CPU 在 ({x},{y}) 处的像素应一致"
                );
            }
        }
    }

    // -- 边界条件测试 --
    /// 测试 1x1 FrameBuffer
    #[test]
    fn test_frame_buffer_1x1() {
        let mut fb = FrameBuffer::new(1, 1);
        fb.set_pixel(0, 0, [255, 0, 0, 255]);
        assert_eq!(fb.get_pixel(0, 0), [255, 0, 0, 255]);
        assert_eq!(fb.data.len(), 4);
    }

    /// 测试 FrameBuffer 角落像素
    #[test]
    fn test_frame_buffer_corner_pixels() {
        let mut fb = FrameBuffer::new(10, 10);
        fb.set_pixel(9, 9, [1, 2, 3, 4]);
        assert_eq!(fb.get_pixel(9, 9), [1, 2, 3, 4]);
    }

    /// 测试 FrameBuffer::from_rgba 零尺寸
    #[test]
    fn test_frame_buffer_from_rgba_zero_size() {
        let fb = FrameBuffer::from_rgba(vec![], 0, 0);
        assert!(fb.is_ok());
        let fb = fb.unwrap();
        assert_eq!(fb.width, 0);
        assert_eq!(fb.height, 0);
        assert!(fb.data.is_empty());
    }

    /// 测试 FrameBuffer clear 为透明黑色
    #[test]
    fn test_frame_buffer_clear_transparent() {
        let mut fb = FrameBuffer::new(3, 3);
        fb.set_pixel(1, 1, [255, 255, 255, 255]);
        fb.clear(0, 0, 0, 0);
        assert_eq!(fb.get_pixel(0, 0), [0, 0, 0, 0]);
        assert_eq!(fb.get_pixel(1, 1), [0, 0, 0, 0]);
        assert_eq!(fb.get_pixel(2, 2), [0, 0, 0, 0]);
    }

    /// 测试 SurfaceDescriptor 零尺寸
    #[test]
    fn test_surface_descriptor_zero_dims() {
        let desc = SurfaceDescriptor::new(0, 0);
        assert_eq!(desc.width, 0);
        assert_eq!(desc.height, 0);
        assert!(!desc.transparent);
    }

    /// 测试表面多次 resize 后最终尺寸正确
    ///
    /// 模拟连续多次调整表面尺寸：每次创建新的 FrameBuffer 并填充测试像素，
    /// 验证中间步骤和最终尺寸均为预期值。
    #[test]
    fn test_surface_multiple_resizes() {
        let sizes = [(100, 100), (200, 150), (50, 300), (1024, 768)];

        let mut fb = FrameBuffer::new(sizes[0].0, sizes[0].1);
        assert_eq!(fb.width, sizes[0].0);
        assert_eq!(fb.height, sizes[0].1);

        for &(w, h) in &sizes[1..] {
            // 模拟 resize：创建新的帧缓冲
            fb = FrameBuffer::new(w, h);
            assert_eq!(fb.width, w, "resize 后宽度应为 {w}");
            assert_eq!(fb.height, h, "resize 后高度应为 {h}");
            assert_eq!(fb.data.len(), (w * h * 4) as usize);
        }

        // 验证最终尺寸
        let (final_w, final_h) = sizes[sizes.len() - 1];
        assert_eq!(fb.width, final_w);
        assert_eq!(fb.height, final_h);
        // 验证最终帧缓冲可以正常读写像素
        fb.set_pixel(final_w - 1, final_h - 1, [255, 128, 64, 200]);
        assert_eq!(fb.get_pixel(final_w - 1, final_h - 1), [255, 128, 64, 200]);
    }

    /// 测试 FrameBuffer::from_rgba 使用恰好 1x1 像素（4 字节）的数据
    ///
    /// 验证最小有效帧缓冲的创建、读写和清除操作均正确，
    /// 确保单个像素边界条件下无越界访问。
    #[test]
    fn test_frame_buffer_single_pixel_from_rgba() {
        let data = vec![10, 20, 30, 40];
        let mut fb = FrameBuffer::from_rgba(data, 1, 1).expect("1x1 应创建成功");
        assert_eq!(fb.width, 1);
        assert_eq!(fb.height, 1);
        assert_eq!(fb.data.len(), 4);
        assert_eq!(fb.get_pixel(0, 0), [10, 20, 30, 40]);

        // 覆写并回读
        fb.set_pixel(0, 0, [255, 255, 255, 255]);
        assert_eq!(fb.get_pixel(0, 0), [255, 255, 255, 255]);

        // 清除
        fb.clear(0, 0, 0, 0);
        assert_eq!(fb.get_pixel(0, 0), [0, 0, 0, 0]);

        // pixel_count 应为 1
        assert_eq!(fb.pixel_count(), 1);
    }

    /// 测试 FrameBuffer resize 到相同尺寸后数据保持正确
    ///
    /// 创建帧缓冲后设置像素，再以相同尺寸创建新帧缓冲，
    /// 验证新帧缓冲为空（不保留旧数据）。
    #[test]
    fn test_frame_buffer_resize_same_size() {
        let mut fb = FrameBuffer::new(10, 10);
        fb.set_pixel(5, 5, [255, 0, 0, 255]);

        // 模拟 resize 到相同尺寸（实际是新建）
        fb = FrameBuffer::new(10, 10);
        assert_eq!(fb.get_pixel(5, 5), [0, 0, 0, 0], "新帧缓冲应为初始黑色");
        assert_eq!(fb.width, 10);
        assert_eq!(fb.height, 10);
    }

    /// 测试 FrameBuffer 缩小尺寸后数据正确
    ///
    /// 从较大帧缓冲缩小到较小尺寸，验证像素读写仅在有效范围内。
    #[test]
    fn test_frame_buffer_shrink_size() {
        let mut fb = FrameBuffer::new(100, 100);
        fb.set_pixel(50, 50, [128, 128, 128, 255]);

        // 缩小到 10x10
        fb = FrameBuffer::new(10, 10);
        assert_eq!(fb.data.len(), 10 * 10 * 4);
        assert_eq!(fb.get_pixel(0, 0), [0, 0, 0, 0]);
        fb.set_pixel(9, 9, [200, 100, 50, 255]);
        assert_eq!(fb.get_pixel(9, 9), [200, 100, 50, 255]);
    }

    /// 测试 FrameBuffer from_rgba 使用极大值 u32 验证不 panic
    ///
    /// 由于 u32 溢出风险，不应使用极大值创建。
    /// 此测试验证 from_rgba 对数据大小不匹配的错误处理。
    #[test]
    fn test_frame_buffer_from_rgba_mismatch_single_byte() {
        // 提供比期望少 1 个字节的数据
        let data = vec![0u8; 10 * 10 * 4 - 1];
        let fb = FrameBuffer::from_rgba(data, 10, 10);
        assert!(fb.is_err(), "数据少 1 字节应返回错误");
    }
}
