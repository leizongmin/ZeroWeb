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
        Self {
            data,
            width,
            height,
        }
    }

    /// 从 RGBA 数据创建帧缓冲
    pub fn from_rgba(data: Vec<u8>, width: u32, height: u32) -> Result<Self, String> {
        let expected = (width * height * 4) as usize;
        if data.len() != expected {
            return Err(format!(
                "数据大小不匹配: 期望 {}, 实际 {}",
                expected,
                data.len()
            ));
        }
        Ok(Self {
            data,
            width,
            height,
        })
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
        for chunk in self.data.chunks_exact_mut(4) {
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
}
