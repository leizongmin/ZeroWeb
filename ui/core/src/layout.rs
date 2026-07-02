//! 响应式/自适应布局输入（spec FR-015 / IF-009 / DC-12）。
//!
//! `WindowMetrics` 描述窗口物理/逻辑尺寸、scale、safe area、软键盘遮挡；
//! `ViewportClass` 把宽度映射到 Compact/Medium/Expanded，驱动 adaptive shell 选择。
//! 详细 adaptive shell（desktop/tablet/phone）与 `BrowserChromeModel` 共享合约在 M2/M4。

use crate::geometry::{Insets, Size};
use serde::{Deserialize, Serialize};

/// 设备像素 → 逻辑像素的缩放因子（HiDPI）。
pub type ScaleFactor = f32;

/// 用户文本字号缩放的默认值（1.0 = 不放大）。
///
/// spec IF-009：`WindowMetrics` 承载 `text_scale`（移动端无障碍「更大字体」/系统字号设置）。
/// 有效范围 `> 0.0`；`1.0` 为基线，`> 1.0` 放大字号（触发 layout 失效），`< 1.0` 缩小。
pub const DEFAULT_TEXT_SCALE: f32 = 1.0;

/// 布局密度缩放的默认值（1.0 = 标准密度）。
///
/// spec IF-009：`WindowMetrics` 承载 `density`（移动端「compact/comfortable」布局密度）。
/// 与 `scale_factor`（HiDPI 设备像素比）不同——density 缩放**间距/图标尺寸**
/// （`SpacingTokens`），不改变文本测量。`1.0` 标准、`>1.0` 舒朗、`<1.0` 紧凑。
pub const DEFAULT_DENSITY: f32 = 1.0;

/// 屏幕方向（spec IF-009 `Orientation`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation {
    /// 竖屏：高 > 宽。
    Portrait,
    /// 横屏：宽 > 高。
    Landscape,
}

impl Orientation {
    /// 由逻辑尺寸推断方向（宽 < 高 → Portrait；否则 Landscape；正方形按 Landscape）。
    pub fn from_size(size: Size) -> Orientation {
        if size.height > size.width {
            Orientation::Portrait
        } else {
            Orientation::Landscape
        }
    }
}

/// 窗口度量（spec IF-009 `WindowMetrics`）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowMetrics {
    /// 逻辑像素尺寸。
    pub logical_size: Size,
    pub scale_factor: ScaleFactor,
    /// 系统 safe area（刘海/任务栏/圆角避让）。
    pub safe_area: Insets,
    /// 软键盘/IME 当前遮挡区域（逻辑像素，无遮挡为 0）。
    pub keyboard_insets: Insets,
    /// 用户文本字号缩放（spec IF-009，移动端无障碍/系统字号；`DEFAULT_TEXT_SCALE`=1.0）。
    ///
    /// 改变本值会缩放 `TypographyTokens`（→ 触发 `needs_layout`），是 DC-15 移动端
    /// 「text scale」适配的数据入口。桌面端固定 1.0；移动端由 M4 runtime 从系统设置探测。
    pub text_scale: f32,
    /// 布局密度（spec IF-009，移动端「compact/comfortable」间距密度；`DEFAULT_DENSITY`=1.0）。
    ///
    /// 缩放 `SpacingTokens`（→ 触发 `needs_layout`），不影响文本测量（区别于 `text_scale`）。
    /// 桌面端固定 1.0；移动端由 M4 runtime 按用户密度偏好或设备 bucket 探测。
    pub density: f32,
    /// 屏幕方向（spec IF-009，Portrait/Landscape；由 `logical_size` 派生，M4 移动端 adaptive 消费）。
    pub orientation: Orientation,
}

impl WindowMetrics {
    pub fn physical_size(self) -> Size {
        Size::new(
            self.logical_size.width * self.scale_factor,
            self.logical_size.height * self.scale_factor,
        )
    }

    /// 典型手机 metrics（DC-15 移动测试/示例用）：390×844 逻辑像素（iPhone 12 级），
    /// scale 3.0，safe_area top 47（刘海/状态栏）+ bottom 34（home indicator），无键盘，
    /// 默认 text_scale/density。`ViewportClass::Compact`。
    pub fn phone() -> WindowMetrics {
        let logical_size = Size::new(390.0, 844.0);
        WindowMetrics {
            logical_size,
            scale_factor: 3.0,
            safe_area: Insets {
                left: 0.0,
                top: 47.0,
                right: 0.0,
                bottom: 34.0,
            },
            keyboard_insets: Insets::all(0.0),
            text_scale: DEFAULT_TEXT_SCALE,
            density: DEFAULT_DENSITY,
            orientation: Orientation::from_size(logical_size),
        }
    }

    /// 典型平板 metrics（DC-15）：768×1024 逻辑像素（iPad 级），scale 2.0，无 safe_area/键盘，
    /// 默认 text_scale/density。`ViewportClass::Medium`（→ tablet shell）。
    pub fn tablet() -> WindowMetrics {
        let logical_size = Size::new(768.0, 1024.0);
        WindowMetrics {
            logical_size,
            scale_factor: 2.0,
            safe_area: Insets::all(0.0),
            keyboard_insets: Insets::all(0.0),
            text_scale: DEFAULT_TEXT_SCALE,
            density: DEFAULT_DENSITY,
            orientation: Orientation::from_size(logical_size),
        }
    }
}

/// 视口分级（spec IF-009，Material/Bootstrap 风格断点）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewportClass {
    /// 窄屏（手机）：< 600 逻辑像素。
    Compact,
    /// 中屏（小平板/折叠）：600..840。
    Medium,
    /// 宽屏（桌面/大平板）：≥ 840。
    Expanded,
}

impl ViewportClass {
    /// 由逻辑宽度推断分级。
    pub fn from_width(width_px: f32) -> ViewportClass {
        if width_px < 600.0 {
            ViewportClass::Compact
        } else if width_px < 840.0 {
            ViewportClass::Medium
        } else {
            ViewportClass::Expanded
        }
    }
}

/// 平台类别（spec FR-015）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformClass {
    Desktop,
    Mobile,
    /// 嵌入式/其它。
    Embedded,
}

/// 主导输入类别（spec FR-015）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputClass {
    /// 精确指针（鼠标/触控笔）。
    Pointer,
    /// 触摸。
    Touch,
    /// 键盘主导。
    Keyboard,
}

/// adaptive 分支选择输入（M2 的 `Adaptive` widget 消费）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AdaptiveBranch {
    pub viewport: ViewportClass,
    pub platform: PlatformClass,
    pub input: InputClass,
}

impl AdaptiveBranch {
    pub fn from_metrics(metrics: &WindowMetrics, platform: PlatformClass, input: InputClass) -> AdaptiveBranch {
        AdaptiveBranch {
            viewport: ViewportClass::from_width(metrics.logical_size.width),
            platform,
            input,
        }
    }

    /// 是否应使用移动端 shell。
    pub fn is_mobile_shell(self) -> bool {
        matches!(self.viewport, ViewportClass::Compact) || self.platform == PlatformClass::Mobile
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(width: f32) -> WindowMetrics {
        WindowMetrics {
            logical_size: Size::new(width, 800.0),
            scale_factor: 1.0,
            safe_area: Insets::all(0.0),
            keyboard_insets: Insets::all(0.0),
            text_scale: DEFAULT_TEXT_SCALE,
            density: DEFAULT_DENSITY,
            orientation: Orientation::from_size(Size::new(width, 800.0)),
        }
    }

    #[test]
    fn viewport_class_breakpoints() {
        assert_eq!(ViewportClass::from_width(390.0), ViewportClass::Compact);
        assert_eq!(ViewportClass::from_width(600.0), ViewportClass::Medium);
        assert_eq!(ViewportClass::from_width(839.0), ViewportClass::Medium);
        assert_eq!(ViewportClass::from_width(840.0), ViewportClass::Expanded);
        assert_eq!(ViewportClass::from_width(1280.0), ViewportClass::Expanded);
    }

    #[test]
    fn phone_and_tablet_presets_match_expected_viewport_class() {
        // DC-15 presets：phone → Compact（手机 shell），tablet → Medium（tablet shell）。
        let phone = WindowMetrics::phone();
        assert_eq!(
            ViewportClass::from_width(phone.logical_size.width),
            ViewportClass::Compact,
            "phone preset → Compact"
        );
        assert_eq!(phone.scale_factor, 3.0);
        assert_eq!(phone.safe_area.top, 47.0, "phone 有刘海 safe_area");
        assert_eq!(phone.safe_area.bottom, 34.0, "phone 有 home indicator safe_area");
        assert_eq!(phone.orientation, Orientation::Portrait);

        let tablet = WindowMetrics::tablet();
        assert_eq!(
            ViewportClass::from_width(tablet.logical_size.width),
            ViewportClass::Medium,
            "tablet preset → Medium"
        );
        assert_eq!(tablet.scale_factor, 2.0);
        // 默认无 safe_area（平板通常无刘海）。
        assert_eq!(tablet.safe_area, Insets::all(0.0));
    }

    #[test]
    fn adaptive_branch_mobile_vs_desktop() {
        let m = metrics(390.0);
        let mobile = AdaptiveBranch::from_metrics(&m, PlatformClass::Mobile, InputClass::Touch);
        assert!(mobile.is_mobile_shell());

        let m2 = metrics(1280.0);
        let desktop = AdaptiveBranch::from_metrics(&m2, PlatformClass::Desktop, InputClass::Pointer);
        assert!(!desktop.is_mobile_shell());
    }

    #[test]
    fn physical_size_scales() {
        let m = metrics(500.0);
        let mut hidpi = m;
        hidpi.scale_factor = 2.0;
        assert_eq!(hidpi.physical_size().width, 1000.0);
    }

    #[test]
    fn text_scale_defaults_to_baseline() {
        // spec IF-009：WindowMetrics 必须承载 text_scale。默认 = 1.0（不放大）。
        let m = metrics(800.0);
        assert_eq!(m.text_scale, DEFAULT_TEXT_SCALE);
        assert_eq!(DEFAULT_TEXT_SCALE, 1.0);
    }

    #[test]
    fn density_defaults_to_baseline() {
        // spec IF-009：WindowMetrics 必须承载 density（布局密度）。默认 = 1.0（标准）。
        let m = metrics(800.0);
        assert_eq!(m.density, DEFAULT_DENSITY);
        assert_eq!(DEFAULT_DENSITY, 1.0);
    }

    #[test]
    fn orientation_derived_from_size() {
        // spec IF-009：WindowMetrics 承载 orientation（Portrait/Landscape），由尺寸派生。
        assert_eq!(Orientation::from_size(Size::new(390.0, 844.0)), Orientation::Portrait);
        assert_eq!(Orientation::from_size(Size::new(844.0, 390.0)), Orientation::Landscape);
        // 正方形按 Landscape（宽不严格小于高）。
        assert_eq!(Orientation::from_size(Size::new(600.0, 600.0)), Orientation::Landscape);
        // 测试 helper 按尺寸派生：宽 390<高 800 → Portrait；宽 1280>高 800 → Landscape。
        assert_eq!(metrics(390.0).orientation, Orientation::Portrait);
        assert_eq!(metrics(1280.0).orientation, Orientation::Landscape);
    }
}
