//! 图像引用——SDK 层的不透明图像 key（DC-1：`ui/render` 不依赖 render-foundation）。
//!
//! 控件经 [`crate::widget::PaintRecorder::draw_image`] 引用一张由宿主/桥接预注册的图像
//! （典型 = 浏览器 SVG 图标经 resvg 光栅后的 alpha 掩码），由后端按 key 取回位图、按调用方
//! 给的 `tint` 着色光栅。本类型刻意与 render-foundation 的 `ImageKey` 解耦：SDK 只持轻量 u64
//! 引用，桥接层（`zero-ui-adapter-render-foundation`）负责把 `ImageRef` 映射到真实缓存。

use serde::{Deserialize, Serialize};

/// 不透明图像引用（宿主分配的稳定 id）。
///
/// 典型用法：浏览器把 SVG 图标经 resvg 光栅为单通道 alpha 掩码，注册到桥接后端并指定/取得
/// `ImageRef`，再经 chrome 工厂 props 传给控件；控件 paint 时
/// `draw_image(rect, image_ref, tint)` 引用之，桥接按 `tint` 着色（与 glyph 路径一致）。
///
/// 与文本 glyph 路径的对称：glyph = 字体里的 alpha 掩码 + 文本色；本机制把「任意宿主提供的
/// alpha 掩码」（图标 / 自定义符号 / 图片轮廓）以同样的 tint 模型暴露给 SDK 控件。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImageRef(pub u64);

impl ImageRef {
    /// 创建图像引用。
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_ref_is_copy_eq_hash() {
        let a = ImageRef::new(7);
        let b = ImageRef::new(7);
        assert_eq!(a, b);
        assert_ne!(a, ImageRef::new(8));
        // Copy：按值传递后仍相等。
        let c = a;
        assert_eq!(a, c);
    }
}
