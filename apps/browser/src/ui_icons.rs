//! 浏览器 Chrome 图标：运行时以 `resvg` 光栅化 `assets/icons/*.svg`，
//! 经 glyph atlas 的 alpha 遮罩绘制（与文字相同的抗锯齿路径）。
//!
//! # SVG 图标资源索引
//!
//! 所有语义图标都存放在 [`assets/icons/`]，通过 [`Icon`] 枚举 + `svg_bytes()` 统一加载：
//!
//! | [`Icon`] 变体          | SVG 文件              | 用途                 |
//! |------------------------|-----------------------|----------------------|
//! | `ChevronLeft`          | `chevron-left.svg`    | 导航后退、查找上一个 |
//! | `ChevronRight`         | `chevron-right.svg`   | 导航前进             |
//! | `ChevronUp`            | `chevron-up.svg`      | 查找上一个（竖排）   |
//! | `ChevronDown`          | `chevron-down.svg`    | 查找下一个（竖排）   |
//! | `Refresh`              | `refresh.svg`         | 刷新页面             |
//! | `Home`                 | `home.svg`            | 回主页               |
//! | `Close`                | `close.svg`           | 关闭标签、关闭浮层   |
//! | `Plus`                 | `plus.svg`            | 新建标签             |
//! | `MoreVertical`         | `more-vertical.svg`   | 全局菜单（地址栏外） |
//! | `Star`                 | `star.svg`            | 收藏当前页（未收藏） |
//! | `StarFilled`           | `star-filled.svg`     | 已收藏当前页         |
//! | `Lock`                 | `lock.svg`            | HTTPS 安全锁         |
//! | `Download`             | `download.svg`        | 下载管理             |
//! | `Shield`               | `shield.svg`          | 站点权限             |
//! | `VolumeOff`            | `volume-off.svg`      | 标签静音状态         |
//! | `AlertTriangle`        | `alert-triangle.svg`  | 标签崩溃状态         |
//! | `Sun` / `Moon` / `SunMoon` | `sun.svg` / `moon.svg` / `sun-moon.svg` | 主题切换（亮/暗/自动） |
//! | `Clock`                | `clock.svg`           | 自动补全历史来源     |
//!
//! 另有一个 [`globe.svg`] 不走 [`Icon`] 枚举，作为默认 favicon 占位符由
//! [`tab_favicon`] 直接加载。
//!
//! # 仍用代码绘制的图形（非图标，不转 SVG）
//!
//! 以下图形因动态变形或性能原因保持代码绘制，**不应**迁移到 SVG：
//! - 窗口控制按钮（最小化/最大化/关闭）——形状随 `window_is_maximized` 动态变形
//! - 加载旋转环——角度每帧变化，需高频重绘
//! - 标签形状——宽度/圆角随布局动态变化
//! - 分隔线、hover 圆形背景、书签 pill 等纯装饰几何

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg;
use tracing::warn;
use zero_render_foundation::color::Color;
use zero_render_foundation::font::GlyphBitmap;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::gpu::renderer::GlyphDraw;

/// 图标专用虚拟 font id（不对应真实 fontdue 字体）。
pub const ICON_FONT_ID: u32 = 0xFFFF_FFFE;

const ICON_BASE_CODEPOINT: u32 = 0xE000;

/// Chrome 工具栏图标。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    ChevronLeft,
    ChevronRight,
    Refresh,
    Home,
    Close,
    ChevronUp,
    ChevronDown,
    Plus,
    MoreVertical,
    Star,
    StarFilled,
    Lock,
    Download,
    Shield,
    VolumeOff,
    AlertTriangle,
    Search,
    Sun,
    Moon,
    SunMoon,
    Clock,
}

impl Icon {
    /// 所有图标变体，用于测试与一致性校验。新增变体时只需在此处追加。
    pub const ALL: [Icon; 21] = [
        Icon::ChevronLeft,
        Icon::ChevronRight,
        Icon::Refresh,
        Icon::Home,
        Icon::Close,
        Icon::ChevronUp,
        Icon::ChevronDown,
        Icon::Plus,
        Icon::MoreVertical,
        Icon::Star,
        Icon::StarFilled,
        Icon::Lock,
        Icon::Download,
        Icon::Shield,
        Icon::VolumeOff,
        Icon::AlertTriangle,
        Icon::Search,
        Icon::Sun,
        Icon::Moon,
        Icon::SunMoon,
        Icon::Clock,
    ];

    fn svg_bytes(self) -> &'static [u8] {
        match self {
            Icon::ChevronLeft => include_bytes!("../assets/icons/chevron-left.svg"),
            Icon::ChevronRight => include_bytes!("../assets/icons/chevron-right.svg"),
            Icon::Refresh => include_bytes!("../assets/icons/refresh.svg"),
            Icon::Home => include_bytes!("../assets/icons/home.svg"),
            Icon::Close => include_bytes!("../assets/icons/close.svg"),
            Icon::ChevronUp => include_bytes!("../assets/icons/chevron-up.svg"),
            Icon::ChevronDown => include_bytes!("../assets/icons/chevron-down.svg"),
            Icon::Plus => include_bytes!("../assets/icons/plus.svg"),
            Icon::MoreVertical => include_bytes!("../assets/icons/more-vertical.svg"),
            Icon::Star => include_bytes!("../assets/icons/star.svg"),
            Icon::StarFilled => include_bytes!("../assets/icons/star-filled.svg"),
            Icon::Lock => include_bytes!("../assets/icons/lock.svg"),
            Icon::Download => include_bytes!("../assets/icons/download.svg"),
            Icon::Shield => include_bytes!("../assets/icons/shield.svg"),
            Icon::VolumeOff => include_bytes!("../assets/icons/volume-off.svg"),
            Icon::AlertTriangle => include_bytes!("../assets/icons/alert-triangle.svg"),
            Icon::Search => include_bytes!("../assets/icons/search.svg"),
            Icon::Sun => include_bytes!("../assets/icons/sun.svg"),
            Icon::Moon => include_bytes!("../assets/icons/moon.svg"),
            Icon::SunMoon => include_bytes!("../assets/icons/sun-moon.svg"),
            Icon::Clock => include_bytes!("../assets/icons/clock.svg"),
        }
    }

    pub(crate) fn glyph_id(self) -> u32 {
        ICON_BASE_CODEPOINT + self as u32
    }

    pub(crate) fn as_char(self) -> char {
        char::from_u32(self.glyph_id()).unwrap_or('\0')
    }
}

/// 在 `(cx, cy)` 居中绘制图标。`size` 为物理像素边长。
pub fn render_icon(
    font_loader: &mut FontLoader,
    glyphs: &mut Vec<GlyphDraw>,
    icon: Icon,
    cx: f32,
    cy: f32,
    size: f32,
    color: Color,
) {
    if size <= 0.0 {
        return;
    }

    ensure_icon_bitmap(font_loader, icon, size);

    glyphs.push(GlyphDraw {
        ch: icon.as_char(),
        font_glyph_index: None,
        x: cx - size * 0.5,
        baseline_y: cy + size * 0.5,
        color,
        font_id: ICON_FONT_ID,
        font_size: size,
        rotation: 0.0,
    });
}

fn ensure_icon_bitmap(font_loader: &mut FontLoader, icon: Icon, size_px: f32) {
    let glyph_id = icon.glyph_id();
    if font_loader.has_bitmap_glyph(ICON_FONT_ID, glyph_id, size_px) {
        return;
    }

    match rasterize_icon_svg(icon.svg_bytes(), size_px) {
        Ok(bitmap) => {
            font_loader.register_bitmap_glyph(ICON_FONT_ID, glyph_id, size_px, bitmap);
        }
        Err(err) => {
            warn!(?icon, %size_px, %err, "failed to rasterize chrome icon");
        }
    }
}

fn rasterize_icon_svg(svg: &[u8], size_px: f32) -> Result<GlyphBitmap, String> {
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_data(svg, &options).map_err(|e| e.to_string())?;

    let side = size_px.ceil().max(1.0) as u32;
    let mut pixmap = Pixmap::new(side, side).ok_or_else(|| "pixmap allocation failed".to_string())?;

    let view_w = tree.size().width();
    let view_h = tree.size().height();
    let scale = side as f32 / view_w.max(view_h);
    resvg::render(&tree, Transform::from_scale(scale, scale), &mut pixmap.as_mut());

    let data: Vec<u8> = pixmap.pixels().iter().map(|px| px.alpha()).collect();

    Ok(GlyphBitmap {
        data,
        width: side as u16,
        height: side as u16,
        x_offset: 0,
        y_offset: 0,
        advance: size_px,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rasterize_all_icons() {
        let mut loader = FontLoader::new();
        for &icon in Icon::ALL.iter() {
            render_icon(&mut loader, &mut Vec::new(), icon, 16.0, 16.0, 24.0, Color::BLACK);
            assert!(loader.has_bitmap_glyph(ICON_FONT_ID, icon.glyph_id(), 24.0));
        }
    }

    #[test]
    fn icon_bitmap_has_alpha_edges() {
        let bitmap = rasterize_icon_svg(Icon::Close.svg_bytes(), 24.0).expect("rasterize close");
        assert!(bitmap.width > 0 && bitmap.height > 0);
        let opaque = bitmap.data.iter().filter(|&&a| a > 200).count();
        let fringe = bitmap.data.iter().filter(|&&a| (1..200).contains(&a)).count();
        assert!(opaque > 0, "icon should have opaque pixels");
        assert!(fringe > 0, "icon should have antialiased fringe pixels");
    }

    #[test]
    fn download_icon_has_arrow_and_tray() {
        let bitmap = rasterize_icon_svg(Icon::Download.svg_bytes(), 32.0).expect("rasterize download");
        let w = bitmap.width as usize;
        let h = bitmap.height as usize;
        assert_eq!(w, 32);
        assert_eq!(h, 32);
        // 上 1/3 与下 1/3 都应有实质像素（箭头在上、托盘在下），避免图标过扁
        let row_has_ink = |start: usize, end: usize| -> bool {
            (start..end).any(|r| {
                let row = &bitmap.data[r * w..(r + 1) * w];
                row.iter().filter(|&&a| a > 64).count() > 0
            })
        };
        assert!(
            row_has_ink(0, h / 3),
            "download icon should have ink in top third (arrow)"
        );
        assert!(
            row_has_ink(2 * h / 3, h),
            "download icon should have ink in bottom third (tray)"
        );
    }

    #[test]
    fn sun_moon_icon_is_half_filled_to_signal_auto() {
        // Auto 主题图标应是半日半月：左半实心、右半描线，与纯 Sun / 纯 Moon 区分。
        let bitmap = rasterize_icon_svg(Icon::SunMoon.svg_bytes(), 32.0).expect("rasterize sun-moon");
        let w = bitmap.width as usize;
        let h = bitmap.height as usize;
        // 中线左右两侧都应有像素
        let mid = w / 2;
        let left_ink: usize = (0..h)
            .map(|r| bitmap.data[r * w..r * w + mid].iter().filter(|&&a| a > 64).count())
            .sum();
        let right_ink: usize = (0..h)
            .map(|r| {
                bitmap.data[r * w + mid..(r + 1) * w]
                    .iter()
                    .filter(|&&a| a > 64)
                    .count()
            })
            .sum();
        assert!(left_ink > 0, "sun-moon left half should have ink");
        assert!(right_ink > 0, "sun-moon right half should have ink");
        // 左半（实心）应明显多于右半（描线），体现半填充
        assert!(
            left_ink > right_ink,
            "sun-moon left half (filled) should have more ink than right half (outline)"
        );
    }
}
