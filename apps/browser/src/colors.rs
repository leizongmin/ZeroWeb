//! 浏览器 Chrome UI 颜色（随 `prefers-color-scheme` / 系统主题切换）

use zero_engine::PrefersColorSchemeValue;
use zero_render_foundation::color::Color;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color { r, g, b, a: 255 }
}

const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color { r, g, b, a }
}

/// 浏览器外壳配色方案。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChromePalette {
    pub background: Color,
    pub tab_bar_bg: Color,
    pub toolbar_bg: Color,
    pub tab_active_bg: Color,
    pub tab_hover_bg: Color,
    pub tab_text: Color,
    pub tab_close: Color,
    /// 相邻非激活标签之间的竖线分隔
    pub tab_separator: Color,
    pub tab_crashed: Color,
    pub tab_attention: Color,
    pub address_bar_bg: Color,
    pub address_bar_bg_focused: Color,
    pub address_bar_border: Color,
    pub address_bar_border_focused: Color,
    pub address_bar_text: Color,
    pub address_bar_placeholder: Color,
    pub address_bar_selection_bg: Color,
    pub address_bar_insecure: Color,
    pub address_bar_internal: Color,
    pub address_bar_secure: Color,
    pub text_selection_bg: Color,
    pub page_bg: Color,
    pub nav_button: Color,
    pub nav_button_disabled: Color,
    pub nav_button_pressed: Color,
    pub separator: Color,
    pub loading_indicator: Color,
    pub page_title: Color,
    pub page_url: Color,
    pub page_hint: Color,
    pub status_text: Color,
    pub autocomplete_bg: Color,
    pub autocomplete_hover_bg: Color,
    pub autocomplete_selected_bg: Color,
    pub autocomplete_text: Color,
    pub autocomplete_url: Color,
    pub autocomplete_bookmark: Color,
    pub find_bar_bg: Color,
    pub find_bar_border: Color,
    pub find_bar_text: Color,
    pub find_match_text: Color,
    pub find_active_option_bg: Color,
    pub find_active_option_text: Color,
    pub new_tab_button: Color,
    pub window_control_hover: Color,
    pub window_control_close_hover: Color,
    pub window_control_icon: Color,
    pub context_menu_bg: Color,
    pub context_menu_hover_bg: Color,
    pub context_menu_text: Color,
    pub context_menu_separator: Color,
    pub bookmarks_bar_bg: Color,
    pub bookmarks_bar_text: Color,
    pub bookmarks_bar_hover_bg: Color,
    pub bookmarks_bar_icon: Color,
    pub download_bar_bg: Color,
    pub download_bar_fill: Color,
    pub download_bar_text: Color,
    /// 页面滚动条轨道（overlay 风格默认透明）
    pub scrollbar_track: Color,
    /// 页面滚动条滑块
    pub scrollbar_thumb: Color,
    /// 页面滚动条滑块 hover
    pub scrollbar_thumb_hover: Color,
    /// 页面滚动条滑块拖拽
    pub scrollbar_thumb_active: Color,
    /// Wayland 无系统装饰时，非最大化窗口外框描边
    pub window_frame_border: Color,
    /// 窗口非激活（失焦）时的 chrome 背景色（标签栏 + 工具栏统一变灰）
    pub chrome_inactive_bg: Color,
    /// 窗口非激活时的地址栏边框色（弱化焦点感知）
    pub address_bar_border_inactive: Color,
}

impl ChromePalette {
    /// 按颜色方案返回 Chrome 配色。
    pub fn for_scheme(scheme: PrefersColorSchemeValue) -> Self {
        match scheme {
            PrefersColorSchemeValue::Dark => Self::dark(),
            PrefersColorSchemeValue::Light => Self::light(),
        }
    }

    /// 亮色 Chrome 风格配色。
    pub const fn light() -> Self {
        Self {
            background: rgb(241, 243, 244),
            tab_bar_bg: rgb(211, 227, 253),
            toolbar_bg: rgb(255, 255, 255),
            tab_active_bg: rgb(255, 255, 255),
            tab_hover_bg: rgb(232, 240, 254),
            tab_text: rgb(32, 33, 36),
            tab_close: rgb(95, 99, 104),
            tab_separator: rgb(148, 152, 160),
            tab_crashed: rgb(217, 48, 37),
            tab_attention: rgb(234, 134, 0),
            address_bar_bg: rgb(255, 255, 255),
            address_bar_bg_focused: rgb(255, 255, 255),
            address_bar_border: rgb(218, 220, 224),
            address_bar_border_focused: rgb(26, 115, 232),
            address_bar_text: rgb(32, 33, 36),
            address_bar_placeholder: rgb(128, 134, 139),
            address_bar_selection_bg: rgb(168, 199, 250),
            address_bar_insecure: rgb(234, 134, 0),
            address_bar_internal: rgb(95, 99, 104),
            address_bar_secure: rgb(26, 127, 55),
            text_selection_bg: rgb(180, 215, 255),
            page_bg: rgb(255, 255, 255),
            nav_button: rgb(95, 99, 104),
            nav_button_disabled: rgb(189, 193, 198),
            nav_button_pressed: rgb(218, 220, 224),
            separator: rgb(218, 220, 224),
            loading_indicator: rgb(66, 133, 244),
            page_title: rgb(32, 33, 36),
            page_url: rgb(95, 99, 104),
            page_hint: rgb(128, 134, 139),
            status_text: rgb(95, 99, 104),
            autocomplete_bg: rgb(255, 255, 255),
            autocomplete_hover_bg: rgb(241, 243, 244),
            autocomplete_selected_bg: rgb(232, 240, 254),
            autocomplete_text: rgb(32, 33, 36),
            autocomplete_url: rgb(95, 99, 104),
            autocomplete_bookmark: rgb(234, 134, 0),
            find_bar_bg: rgba(255, 255, 255, 245),
            find_bar_border: rgb(218, 220, 224),
            find_bar_text: rgb(32, 33, 36),
            find_match_text: rgb(128, 134, 139),
            find_active_option_bg: rgb(26, 115, 232),
            find_active_option_text: rgb(255, 255, 255),
            new_tab_button: rgb(95, 99, 104),
            window_control_hover: rgb(197, 213, 237),
            window_control_close_hover: rgb(196, 43, 28),
            window_control_icon: rgb(95, 99, 104),
            context_menu_bg: rgb(255, 255, 255),
            context_menu_hover_bg: rgb(241, 243, 244),
            context_menu_text: rgb(32, 33, 36),
            context_menu_separator: rgb(218, 220, 224),
            bookmarks_bar_bg: rgb(248, 249, 250),
            bookmarks_bar_text: rgb(60, 64, 67),
            bookmarks_bar_hover_bg: rgb(232, 234, 237),
            bookmarks_bar_icon: rgb(234, 134, 0),
            download_bar_bg: rgb(241, 243, 244),
            download_bar_fill: rgb(66, 133, 244),
            download_bar_text: rgb(32, 33, 36),
            scrollbar_track: rgba(0, 0, 0, 0),
            // overlay 风格半透明滑块：默认更柔和（Chrome 风格浅灰），hover/active 递进加深。
            scrollbar_thumb: rgba(0, 0, 0, 60),
            scrollbar_thumb_hover: rgba(0, 0, 0, 110),
            scrollbar_thumb_active: rgba(0, 0, 0, 160),
            window_frame_border: rgb(160, 164, 169),
            chrome_inactive_bg: rgb(235, 238, 242),
            address_bar_border_inactive: rgb(224, 226, 230),
        }
    }

    /// 深色 Chrome 风格配色。
    pub const fn dark() -> Self {
        Self {
            background: rgb(30, 30, 30),
            tab_bar_bg: rgb(40, 40, 40),
            toolbar_bg: rgb(52, 53, 56),
            tab_active_bg: rgb(60, 60, 60),
            tab_hover_bg: rgb(50, 50, 50),
            tab_text: rgb(200, 200, 200),
            tab_close: rgb(150, 150, 150),
            tab_separator: rgb(120, 120, 120),
            tab_crashed: rgb(242, 139, 130),
            tab_attention: rgb(255, 193, 7),
            address_bar_bg: rgb(46, 47, 50),
            address_bar_bg_focused: rgb(32, 33, 36),
            address_bar_border: rgb(72, 74, 77),
            address_bar_border_focused: rgb(138, 180, 248),
            address_bar_text: rgb(240, 240, 240),
            address_bar_placeholder: rgb(160, 160, 160),
            address_bar_selection_bg: rgb(70, 110, 180),
            address_bar_insecure: rgb(255, 193, 7),
            address_bar_internal: rgb(160, 160, 160),
            address_bar_secure: rgb(129, 201, 149),
            text_selection_bg: rgb(180, 215, 255),
            page_bg: rgb(255, 255, 255),
            nav_button: rgb(180, 180, 180),
            nav_button_disabled: rgb(90, 90, 90),
            nav_button_pressed: rgb(72, 74, 77),
            separator: rgb(70, 70, 70),
            loading_indicator: rgb(66, 133, 244),
            page_title: rgb(0, 0, 0),
            page_url: rgb(100, 100, 100),
            page_hint: rgb(150, 150, 150),
            status_text: rgb(120, 120, 120),
            autocomplete_bg: rgb(45, 45, 45),
            autocomplete_hover_bg: rgb(60, 60, 60),
            autocomplete_selected_bg: rgb(48, 58, 78),
            autocomplete_text: rgb(220, 220, 220),
            autocomplete_url: rgb(140, 140, 140),
            autocomplete_bookmark: rgb(255, 193, 7),
            find_bar_bg: rgba(50, 50, 50, 245),
            find_bar_border: rgb(72, 74, 77),
            find_bar_text: rgb(220, 220, 220),
            find_match_text: rgb(160, 160, 160),
            find_active_option_bg: rgb(138, 180, 248),
            find_active_option_text: rgb(32, 33, 36),
            new_tab_button: rgb(160, 160, 160),
            window_control_hover: rgb(60, 60, 60),
            window_control_close_hover: rgb(196, 43, 28),
            window_control_icon: rgb(200, 200, 200),
            context_menu_bg: rgb(48, 48, 48),
            context_menu_hover_bg: rgb(65, 65, 65),
            context_menu_text: rgb(220, 220, 220),
            context_menu_separator: rgb(70, 70, 70),
            bookmarks_bar_bg: rgb(52, 53, 56),
            bookmarks_bar_text: rgb(190, 190, 190),
            bookmarks_bar_hover_bg: rgb(50, 50, 50),
            bookmarks_bar_icon: rgb(255, 193, 7),
            download_bar_bg: rgb(40, 40, 40),
            download_bar_fill: rgb(66, 133, 244),
            download_bar_text: rgb(220, 220, 220),
            scrollbar_track: rgba(0, 0, 0, 0),
            scrollbar_thumb: rgba(255, 255, 255, 70),
            scrollbar_thumb_hover: rgba(255, 255, 255, 120),
            scrollbar_thumb_active: rgba(255, 255, 255, 170),
            window_frame_border: rgb(90, 90, 90),
            chrome_inactive_bg: rgb(40, 41, 44),
            address_bar_border_inactive: rgb(60, 62, 65),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_and_dark_palettes_differ() {
        let light = ChromePalette::light();
        let dark = ChromePalette::dark();
        assert_ne!(light.tab_bar_bg, dark.tab_bar_bg);
        assert_ne!(light.tab_text, dark.tab_text);
        assert_ne!(light.background, dark.background);
    }

    #[test]
    fn light_address_bar_connects_to_active_tab_surface() {
        let light = ChromePalette::light();
        assert_eq!(light.toolbar_bg, light.tab_active_bg);
        assert_eq!(light.address_bar_bg, light.tab_active_bg);
    }

    #[test]
    fn for_scheme_selects_palette() {
        assert_eq!(
            ChromePalette::for_scheme(PrefersColorSchemeValue::Light),
            ChromePalette::light()
        );
        assert_eq!(
            ChromePalette::for_scheme(PrefersColorSchemeValue::Dark),
            ChromePalette::dark()
        );
    }
}
