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
}

impl WindowMetrics {
    pub fn physical_size(self) -> Size {
        Size::new(
            self.logical_size.width * self.scale_factor,
            self.logical_size.height * self.scale_factor,
        )
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
}
