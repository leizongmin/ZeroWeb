//! Chrome → Scene 渲染桥（DC-7 完整验收）。
//!
//! 把 adaptive shell 产出的 `browser.*` [`WidgetSpec`] 声明树，经 [`WidgetHost`] 注册的
//! chrome 绘制工厂 paint 进统一 [`Scene`]（走 `ui/render` 的 `PaintRecorder`，**不绕过** ui/render）。
//!
//! - **容器节点**（shell 根 / ToolbarRow / Phone 顶部）由 shell 在 `props.layout` 声明为
//!   `column`/`row`（见 `ui/runtime::host` 的 `node_container_kind`），无 widget —— host 据此布局子节点。
//! - **叶子节点**（AddressBar/SecurityBadge/NavigationButtons/BrowserMenu/BrowserTabStrip/
//!   BookmarksBar/PageViewportFrame/FindBar）注册绘制工厂 [`ChromePanel`]，按 props 语义色 + 文案 paint。
//!
//! 本模块是「chrome 组件 → 可渲染 Scene」的桥，证明浏览器领域组件输出进统一 SDK scene（DC-7）。
//! 真实主题色解析在 DC-5 主题系统接入后由 semantic token 解析；此处用 [`chrome_color`] 语义色名映射。

use zero_ui_core::action::EventResult;
use zero_ui_core::binding::Value;
use zero_ui_core::event::UiEvent;
use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
use zero_ui_core::image::ImageRef;
use zero_ui_core::theme::{Color, SemanticTokens};
use zero_ui_core::widget::{
    EventCtx, LayoutCtx, MountCtx, PaintCtx, PaintRecorder, Props, SemanticsCtx, UpdateCtx, Widget, WidgetSpec,
};
use zero_ui_render::Scene;
use zero_ui_runtime::WidgetHost;

/// chrome 色名 → semantic token 名（组件消费 semantic token，不硬编码浏览器色值；DC-5）。
///
/// chrome 领域别名（`chrome`/`accent`/`secure`/`insecure`/`mixed`/`dangerous`/`viewport`）
/// 映射到通用 semantic token；非别名（含标准 token 名 `surface`/`primary`/... 与未知名）
/// 返回 `None`，由 [`chrome_color_themed`] 直接当 token 名解析或回落 `surface`。
fn chrome_alias_token(name: &str) -> Option<&'static str> {
    match name {
        "chrome" | "toolbar_bg" => Some("surface"),
        "tab_strip_bg" => Some("surface"),
        "address_bar_bg" => Some("background"),
        "accent" => Some("primary"),
        "secure" => Some("success"),
        "insecure" | "mixed" => Some("warning"),
        "dangerous" => Some("error"),
        "viewport" => Some("background"),
        _ => None,
    }
}

/// chrome 色名 + 主题 token → [`Color`]（DC-5：组件经 semantic token 解析，随主题切换）。
///
/// 先按 chrome 别名映射，否则直接当 token 名（`surface`/`primary`/`on_surface`/...）解析；
/// 未知名回落 `surface`。
pub fn chrome_color_themed(name: &str, tokens: &SemanticTokens) -> Color {
    let token = chrome_alias_token(name).unwrap_or(name);
    tokens.color_for(token).unwrap_or(tokens.surface)
}

/// chrome 色名 → [`Color`]（浅色基线；等价 `chrome_color_themed(name, &SemanticTokens::light())`）。
pub fn chrome_color(name: &str) -> Color {
    chrome_color_themed(name, &SemanticTokens::light())
}

/// chrome 安全状态 → 语义色名。
pub fn security_color_name(state: crate::security_badge::SecurityState) -> &'static str {
    use crate::security_badge::SecurityState;
    match state {
        SecurityState::Secure => "secure",
        SecurityState::Insecure => "insecure",
        SecurityState::Mixed => "mixed",
        SecurityState::Dangerous => "dangerous",
    }
}

/// chrome 叶子绘制控件：填充背景（语义色）+ 可选文案。
///
/// `fill_viewport = true` 时占满可用宽高（用于 PageViewportFrame）；否则占满宽、固定高（bars）。
/// `fill_background = false` 时跳过 `fill_rect`（DC-14 替换式迁移：PageViewportFrame 不填底色，
/// 页面内容由 WebView 绘制，避免 SDK chrome 的 viewport 底色覆盖页面内容）。
/// paint 读 `ctx.clip` 得节点可视尺寸，以局部坐标 (0,0) 填充 —— host 按节点 origin 平移后覆盖节点 rect。
pub struct ChromePanel {
    bg: Color,
    text: Option<String>,
    text_color: Color,
    height: f32,
    fill_viewport: bool,
    fill_background: bool,
    /// 占满可用宽度（cross-axis 拉伸）。用于全宽 bar（TabStrip / BookmarksBar）——
    /// 这些 bar 在 column 容器里须铺满窗口宽，而非按文案宽度收缩。
    fill_width: bool,
    /// 背景圆角半径（逻辑像素，四角同）。> 0 时用 `fill_rounded_rect`（地址栏 pill 等）。
    corner_radius: f32,
}

impl ChromePanel {
    /// 由声明节点构造：`bg`/`text`/`text_color`/`height` 从 props 读，颜色经 semantic token
    /// 解析（`tokens`，DC-5）；缺省 bg 用 `default_bg` 别名。
    pub fn from_spec(
        spec: &WidgetSpec,
        default_bg: &str,
        default_height: f32,
        fill_viewport: bool,
        tokens: &SemanticTokens,
    ) -> ChromePanel {
        Self::from_spec_with_fill(spec, default_bg, default_height, fill_viewport, true, tokens)
    }

    /// 同 [`from_spec`]，但 `fill_background` 控制是否绘制底色（DC-14 替换式迁移：
    /// PageViewportFrame 设 `false` 不填底色，页面内容由 WebView 绘制在上层）。
    pub fn from_spec_with_fill(
        spec: &WidgetSpec,
        default_bg: &str,
        default_height: f32,
        fill_viewport: bool,
        fill_background: bool,
        tokens: &SemanticTokens,
    ) -> ChromePanel {
        let bg_name = match spec.props.get("bg") {
            Some(Value::Text(s)) => s.as_str(),
            _ => default_bg,
        };
        let bg = chrome_color_themed(bg_name, tokens);
        let text = match spec.props.get("text") {
            Some(Value::Text(s)) => Some(s.clone()),
            _ => None,
        };
        let text_color = match spec.props.get("text_color") {
            Some(Value::Text(s)) => chrome_color_themed(s, tokens),
            _ => tokens.on_background,
        };
        let height = match spec.props.get("height") {
            Some(Value::Float(f)) => *f as f32,
            Some(Value::Int(i)) => *i as f32,
            _ => default_height,
        };
        let fill_width = match spec.props.get("fill_width") {
            Some(Value::Bool(b)) => *b,
            _ => false,
        };
        let corner_radius = match spec.props.get("corner_radius") {
            Some(Value::Float(f)) => *f as f32,
            Some(Value::Int(i)) => *i as f32,
            _ => 0.0,
        };
        ChromePanel {
            bg,
            text,
            text_color,
            height,
            fill_viewport,
            fill_background,
            fill_width,
            corner_radius,
        }
    }
}

impl Widget for ChromePanel {
    fn mount(&mut self, _ctx: &mut MountCtx) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _props: &Props) {}

    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        if self.fill_viewport {
            Size::new(constraints.max_width, constraints.max_height)
        } else {
            let height = self.height.clamp(constraints.min_height, constraints.max_height);
            let width = if self.fill_width {
                // 全宽 bar（TabStrip / BookmarksBar）：占满 column 容器可用宽。
                constraints.max_width
            } else if let Some(text) = &self.text {
                let char_count = text.chars().count().max(1) as f32;
                let approx = (char_count * 14.0 * 0.55 + 16.0).ceil();
                approx.clamp(constraints.min_width, constraints.max_width)
            } else {
                let icon = (height * 0.75).ceil().max(24.0);
                icon.clamp(constraints.min_width, constraints.max_width)
            };
            Size::new(width, height)
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        // ctx.clip = 祖先 clip ∩ 节点 rect（绝对坐标）；其 size 即节点可视尺寸。
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(400.0, self.height));
        if self.fill_background {
            let rect = Rect::from_ltrb(0.0, 0.0, size.width, size.height);
            if self.corner_radius > 0.0 {
                ctx.recorder.fill_rounded_rect(rect, self.corner_radius, self.bg);
            } else {
                ctx.recorder.fill_rect(rect, self.bg);
            }
        }
        if let Some(text) = &self.text {
            // 文本基线：bars 在 height-8 处；viewport 顶部 18px。
            let baseline = if self.fill_viewport {
                18.0
            } else {
                (self.height - 8.0).max(10.0)
            };
            ctx.recorder
                .draw_text(text, Point::new(6.0, baseline), 14.0, self.text_color);
        }
    }

    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

// ── NavigationButtonsWidget（真实图标，DC-14 chrome 功能等价）──────────────────────

/// 导航按钮几何（与 apps/browser/src/layout.rs 手绘 chrome 对齐，DC-14 像素级等价）。
const NAV_BUTTON_WIDTH: f32 = 36.0;
const NAV_ICON_SIZE: f32 = 16.0;
const NAV_BAR_HEIGHT: f32 = 44.0; // = ADDRESS_BAR_HEIGHT。
/// nav 段左侧留白（= apps/browser layout::NAV_SECTION_LEADING_PAD）：手绘 chrome 第一个
/// nav 按钮距 toolbar 左缘 10px，控件据此对齐图标 x 位置。
const NAV_LEADING_PAD: f32 = 10.0;
/// nav 段右侧留白 = NAV_SECTION_TRAILING_GAP(10) + ADDRESS_BAR_PADDING(10) = 20：手绘 chrome
/// `bar_x = nav_section_width(164) + ADDRESS_BAR_PADDING(10) = 174`。nav 控件宽含此留白使
/// 后续 AddressBar 左缘对齐手绘 bar_x（nav 图标仍在 [10,154]，[154,174] 为 toolbar_bg 不可见）。
const NAV_TRAILING_PAD: f32 = 20.0;

/// 导航图标 [`ImageRef`]——宿主（apps/browser）须按相同 id 把 SVG 图标的 alpha 掩码注册到
/// 桥接 `image_masks`（`render_chrome_via_sdk_with_webview_surface` 的 `image_masks` 参数）。
/// id 从 1 起（0 保留）。顺序 = back / forward / reload / home（手绘 chrome nav 段顺序）。
pub const NAV_ICON_BACK: ImageRef = ImageRef::new(1);
pub const NAV_ICON_FORWARD: ImageRef = ImageRef::new(2);
pub const NAV_ICON_RELOAD: ImageRef = ImageRef::new(3);
pub const NAV_ICON_HOME: ImageRef = ImageRef::new(4);
/// menu（更多）按钮 MoreVertical 三点图标。手绘 chrome menu 是 MoreVertical 图标（非 "Menu" 文本）。
pub const MENU_ICON_MORE: ImageRef = ImageRef::new(5);
/// menu 按钮宽（= apps/browser layout::TOOLBAR_MENU_BUTTON_WIDTH）。
const MENU_BUTTON_WIDTH: f32 = 32.0;
/// 窗口控制 close（X）图标（min/max 用 fill_rect 画线/方块，close 是 SVG 图标需 ImageRef）。
pub const WC_ICON_CLOSE: ImageRef = ImageRef::new(6);
/// 窗口控制按钮宽（= apps/browser layout::WINDOW_CONTROL_BTN_WIDTH）。
const WINDOW_CONTROL_BTN_WIDTH: f32 = 46.0;
/// menu 按钮右侧留白 = ADDRESS_BAR_PADDING(10)：手绘 `menu_btn_x = width - ADDRESS_BAR_PADDING -
/// btn_w(32) = 1238`，menu [1238,1270] + 右侧 10px padding。控件宽含此留白使 flex-end 摆放时
/// menu 按钮左缘对齐手绘 1238（图标在按钮 [0,32] 中心 local x=16，trailing [32,42] 为 toolbar_bg）。
const MENU_TRAILING_PAD: f32 = 10.0;

/// 导航按钮组绘制控件（DC-14：真实图标替换 ChromePanel 占位）。
///
/// paint：填 nav 段背景（toolbar bg）+ 4 图标（back/forward/reload/home，各 `NAV_BUTTON_WIDTH` 宽，
/// 图标 `NAV_ICON_SIZE` 居中）。disabled 按钮（无历史 back/forward）用更淡 tint。图标经
/// [`PaintRecorder::draw_image`] 引用宿主预注册的 alpha 掩码（`NAV_ICON_*`），桥接按 tint 着色光栅
/// （与 glyph 文本路径对称）。hover 圆盘为指针态视觉，静态/无指针时不画（与手绘一致）。
pub struct NavigationButtonsWidget {
    bg: Color,
    icon_tint: Color,
    disabled_tint: Color,
    can_back: bool,
    can_forward: bool,
}

impl NavigationButtonsWidget {
    /// 由声明节点构造：`bg`（默认 `chrome`）+ `can_back`/`can_forward` bool props；
    /// 图标 tint 由 semantic token 派生（`on_surface`，disabled = `on_surface.mix(surface,0.5)`）。
    pub fn from_spec(spec: &WidgetSpec, tokens: &SemanticTokens) -> NavigationButtonsWidget {
        let bg_name = match spec.props.get("bg") {
            Some(Value::Text(s)) => s.as_str(),
            _ => "chrome",
        };
        let can_back = match spec.props.get("can_back") {
            Some(Value::Bool(b)) => *b,
            _ => false,
        };
        let can_forward = match spec.props.get("can_forward") {
            Some(Value::Bool(b)) => *b,
            _ => false,
        };
        NavigationButtonsWidget {
            bg: chrome_color_themed(bg_name, tokens),
            icon_tint: tokens.on_surface,
            // disabled tint ≈ on_surface.mix(surface, 0.62)：当 tokens 经 ChromePalette 映射
            // （on_surface=nav_button 95,99,104 / surface=toolbar_bg 248,249,250）→ (190,191,195)，
            // 对齐手绘 nav_button_disabled (189,193,198)。
            disabled_tint: tokens.on_surface.mix(tokens.surface, 0.62),
            can_back,
            can_forward,
        }
    }
}

impl Widget for NavigationButtonsWidget {
    fn mount(&mut self, _ctx: &mut MountCtx) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _props: &Props) {}

    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        // leading pad + 4 按钮 × NAV_BUTTON_WIDTH + trailing pad（含 NAV_SECTION_TRAILING_GAP +
        // ADDRESS_BAR_PADDING，对齐手绘 nav_section_width + padding → 后续 AddressBar 左缘 bar_x=174）。
        let width = (NAV_LEADING_PAD + 4.0 * NAV_BUTTON_WIDTH + NAV_TRAILING_PAD)
            .clamp(constraints.min_width, constraints.max_width);
        let height = NAV_BAR_HEIGHT.clamp(constraints.min_height, constraints.max_height);
        Size::new(width, height)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        let size = ctx.clip.map(|r| r.size).unwrap_or_else(|| {
            Size::new(
                NAV_LEADING_PAD + 4.0 * NAV_BUTTON_WIDTH + NAV_TRAILING_PAD,
                NAV_BAR_HEIGHT,
            )
        });
        // nav 段背景（toolbar bg，与相邻 AddressBar/Menu 连续）。
        ctx.recorder
            .fill_rect(Rect::from_ltrb(0.0, 0.0, size.width, size.height), self.bg);
        // 4 图标：back / forward / reload / home。手绘 chrome 图标 x = NAV_LEADING_PAD
        // + i*NAV_BUTTON_WIDTH + (NAV_BUTTON_WIDTH-NAV_ICON_SIZE)/2（与 app_render.rs nav 段对齐）。
        let icons = [
            (NAV_ICON_BACK, self.can_back),
            (NAV_ICON_FORWARD, self.can_forward),
            (NAV_ICON_RELOAD, true),
            (NAV_ICON_HOME, true),
        ];
        let icon_extent = NAV_ICON_SIZE.min(NAV_BUTTON_WIDTH);
        let y = ((size.height - icon_extent) / 2.0).max(0.0);
        for (i, (key, enabled)) in icons.iter().enumerate() {
            let x = NAV_LEADING_PAD + i as f32 * NAV_BUTTON_WIDTH + (NAV_BUTTON_WIDTH - icon_extent) / 2.0;
            let tint = if *enabled { self.icon_tint } else { self.disabled_tint };
            ctx.recorder
                .draw_image(Rect::from_ltrb(x, y, x + icon_extent, y + icon_extent), *key, tint);
        }
    }

    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

// ── MenuButtonWidget（真实 MoreVertical 图标，DC-14 chrome 功能等价）──────────────

/// menu（更多）按钮控件（DC-14：真实 MoreVertical 三点图标替换 ChromePanel "Menu" 文本占位）。
///
/// paint：填 toolbar bg + 居中 MoreVertical 图标（[`MENU_ICON_MORE`]，tint = on_surface）。
/// 图标经 `draw_image` 引用宿主预注册 alpha 掩码（与 NAV_ICON_* 同模式）。layout 宽 32
/// （TOOLBAR_MENU_BUTTON_WIDTH）+ 高 44（toolbar 行）。
pub struct MenuButtonWidget {
    bg: Color,
    icon_tint: Color,
}

impl MenuButtonWidget {
    pub fn from_spec(spec: &WidgetSpec, tokens: &SemanticTokens) -> MenuButtonWidget {
        let bg_name = match spec.props.get("bg") {
            Some(Value::Text(s)) => s.as_str(),
            _ => "chrome",
        };
        MenuButtonWidget {
            bg: chrome_color_themed(bg_name, tokens),
            icon_tint: tokens.on_surface,
        }
    }
}

impl Widget for MenuButtonWidget {
    fn mount(&mut self, _ctx: &mut MountCtx) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _props: &Props) {}

    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        // 按钮宽 32 + 右侧 padding 10 = 42（flex-end 摆放时按钮左缘对齐手绘 menu_btn_x=1238）。
        let width = (MENU_BUTTON_WIDTH + MENU_TRAILING_PAD).clamp(constraints.min_width, constraints.max_width);
        let height = NAV_BAR_HEIGHT.clamp(constraints.min_height, constraints.max_height);
        Size::new(width, height)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        let size = ctx
            .clip
            .map(|r| r.size)
            .unwrap_or_else(|| Size::new(MENU_BUTTON_WIDTH + MENU_TRAILING_PAD, NAV_BAR_HEIGHT));
        ctx.recorder
            .fill_rect(Rect::from_ltrb(0.0, 0.0, size.width, size.height), self.bg);
        let icon_extent = NAV_ICON_SIZE.min(MENU_BUTTON_WIDTH);
        // 图标在按钮 [0, MENU_BUTTON_WIDTH] 中心（local x = 16），对齐手绘 menu_btn_cx。
        let x = (MENU_BUTTON_WIDTH - icon_extent) / 2.0;
        let y = ((size.height - icon_extent) / 2.0).max(0.0);
        ctx.recorder.draw_image(
            Rect::from_ltrb(x, y, x + icon_extent, y + icon_extent),
            MENU_ICON_MORE,
            self.icon_tint,
        );
    }

    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

// ── BrowserTabStripWidget（真实 tab 形状，DC-14 chrome 功能等价）───────────────────

/// 浏览器 tab 专属色（DC-14 parity）。
///
/// 这些色（active/inactive tab 底、分隔线、strip 底）在浏览器 [`ChromePalette`] 里有精确值
/// （light：tab_active_bg=255 / tab_bar_bg=222,225,230 / tab_separator=148,152,160 / toolbar_bg=248），
/// 但**不是**通用 `SemanticTokens` 的标准 slot——通用 token 集无法承载浏览器专属 chrome 语义。
/// 故本结构作为 tokens 的**补集**经工厂签名传入（apps/browser 从 `ChromePalette` 构造），
/// 使 SDK chrome 组件既保持 token 驱动（DC-5 通用色）又产出与手绘相同 tab 色值（DC-14 parity）。
/// 与 [`NavigationButtonsWidget`] 的 disabled tint（token 推导近似）不同：tab 形状是 chrome 最显眼
/// 结构，近似色会产生肉眼可见 diff，故走精确调色板注入。
///
/// [`ChromePalette`]: crate::chrome_model
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChromeTabColors {
    /// tab strip 背景（toolbar_bg；= tokens.surface，此处冗余携带避免 widget 再查 token）。
    pub strip_bg: Color,
    /// 激活 tab 底色（tab_active_bg；light=白 255）。
    pub active_bg: Color,
    /// 非激活 tab 底色（tab_bar_bg；light=222,225,230）。
    pub bar_bg: Color,
    /// 相邻非激活 tab 间分隔线（tab_separator；light=148,152,160）。
    pub separator: Color,
    /// 地址栏 pill 边框色（address_bar_border；light=218,220,224）。手绘 chrome 地址栏 pill 为
    /// 双层 fill_rounded_rect（外 border + 内 bg inset 1px），非 stroke。AddressBarWidget 据此画 border。
    pub address_border: Color,
    /// 窗口控制按钮图标色（window_control_icon；light=95,99,104 = nav_button）。
    /// BrowserTabStripWidget 据此画 min/max/close 图标（DC-14 窗口控制 parity）。
    pub window_icon: Color,
}

impl ChromeTabColors {
    /// SDK-only 默认（无浏览器调色板时从 token 近似；仅测试 / 非 webview 路径用）。
    /// 生产路径（`compose_sdk_chrome_replacement_with_webview`）从 `ChromePalette` 构造精确值。
    pub fn from_tokens(tokens: &SemanticTokens) -> ChromeTabColors {
        ChromeTabColors {
            strip_bg: tokens.surface,
            // active tab 在 light 主题为纯白；dark 主题为深灰，token 近似取 surface 向白提亮。
            active_bg: tokens.surface.lighten(1.0),
            bar_bg: tokens.on_surface.mix(tokens.surface, 0.86),
            separator: tokens.on_surface.mix(tokens.surface, 0.6),
            // address border ≈ on_surface 向 surface 提亮 0.84（light≈224 vs 调色板 218；测试近似）。
            address_border: tokens.on_surface.mix(tokens.surface, 0.84),
            // window icon = on_surface（light=nav_button 95,99,104；手绘 window_control_icon 同色）。
            window_icon: tokens.on_surface,
        }
    }
}

/// tab 几何常量（与 apps/browser/src/layout.rs 手绘 chrome 对齐，DC-14 像素级等价）。
const TAB_BAR_TOP_INSET: f32 = 6.0;
const TAB_BAR_HEIGHT: f32 = 34.0;
const TAB_STRIP_HEIGHT: f32 = 40.0; // = TAB_BAR_TOP_INSET + TAB_BAR_HEIGHT。
const TAB_MIN_WIDTH: f32 = 100.0;
const TAB_MAX_WIDTH: f32 = 240.0;
const TAB_TOP_RADIUS: f32 = 7.0;
const TAB_FOOT_RADIUS: f32 = 7.0;
const TAB_SEPARATOR_INSET: f32 = 8.0;
/// 「新建标签」按钮预留宽（手绘 chrome tab 区右侧；SDK 不画该按钮但需预留使 tab 宽一致）。
const NEW_TAB_BTN_WIDTH: f32 = 34.0;

/// 顶部圆角矩形扫描线（非激活 tab）——镜像 apps/browser/src/tab_chrome.rs::push_top_rounded_rect_fill，
/// 逐行 1px `fill_rect`（局部坐标），与手绘像素级一致（同一扫描线算法，DC-14 parity）。
fn paint_top_rounded_scanlines(rec: &mut dyn PaintRecorder, x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let r = radius.min(w * 0.5).min(h);
    if r <= f32::EPSILON {
        rec.fill_rect(Rect::from_ltrb(x, y, x + w, y + h), color);
        return;
    }
    let min_y = y.floor() as i32;
    let max_y = (y + h).ceil() as i32;
    let r_sq = r * r;
    for row in min_y..max_y {
        let yf = row as f32 + 0.5;
        if yf >= y + h {
            break;
        }
        let mut x_start = x;
        let mut x_end = x + w;
        if yf < y + r {
            let dy = (y + r) - yf;
            let dx = (r_sq - dy * dy).max(0.0).sqrt();
            x_start = x + r - dx;
            x_end = x + w - r + dx;
        }
        if x_end > x_start {
            rec.fill_rect(Rect::from_ltrb(x_start, row as f32, x_end, row as f32 + 1.0), color);
        }
    }
}

/// Chrome 风格激活 tab 扫描线（顶部圆角 + 底部 foot 二次曲线外扩）——镜像
/// apps/browser/src/tab_chrome.rs::push_active_tab_fill。逐行 1px `fill_rect`（`tab_y` 为 strip
/// 局部 y 偏移），与手绘像素级一致。foot 使激活 tab 底部与下方工具栏「连成一片」的经典 Chrome 形状。
#[allow(clippy::too_many_arguments)]
fn paint_active_tab_scanlines(
    rec: &mut dyn PaintRecorder,
    x: f32,
    tab_y: f32,
    w: f32,
    h: f32,
    r_top_in: f32,
    r_foot_in: f32,
    color: Color,
) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let r_top = r_top_in.min(w * 0.5).min(h);
    let r_foot = r_foot_in.min(w * 0.5).min(h * 0.5);
    let bottom_y = h;
    let foot_top = bottom_y - r_foot;
    let r_top_sq = r_top * r_top;
    let min_y = 0;
    let max_y = bottom_y.ceil() as i32;
    for row in min_y..max_y {
        let yf = row as f32 + 0.5;
        if yf >= bottom_y {
            continue;
        }
        let mut x_start = x;
        let mut x_end = x + w;
        if yf < r_top && r_top > f32::EPSILON {
            let dy = r_top - yf;
            let dx = (r_top_sq - dy * dy).max(0.0).sqrt();
            x_start = x + r_top - dx;
            x_end = x + w - r_top + dx;
        }
        if yf >= foot_top && r_foot > f32::EPSILON {
            let foot_span = (bottom_y - foot_top - 0.5).max(f32::EPSILON);
            let progress = ((yf - foot_top) / foot_span).clamp(0.0, 1.0);
            let foot_extend = r_foot * progress * progress;
            x_start -= foot_extend;
            x_end += foot_extend;
        }
        if x_end > x_start {
            rec.fill_rect(
                Rect::from_ltrb(x_start, tab_y + row as f32, x_end, tab_y + row as f32 + 1.0),
                color,
            );
        }
    }
}

/// 标签栏绘制控件（DC-14：真实 tab 形状替换 ChromePanel 占位）。
///
/// paint：填 strip 背景（toolbar bg 全宽）+ 逐 tab 几何（active=Chrome foot 形状白底 /
/// inactive=顶部圆角 bar 底 / 相邻 inactive 间分隔线）。tab 几何算法镜像
/// apps/browser/src/app_render.rs::render_tabs（无 pinned、Windows leading=0 简化；fresh app 默认）。
/// 图标 / 标签文案 / close 按钮 / hover 为后续轮次（本轮先闭合形状 parity，diff 主因）。
pub struct BrowserTabStripWidget {
    strip_bg: Color,
    active_bg: Color,
    bar_bg: Color,
    separator: Color,
    window_icon: Color,
    tab_count: usize,
    active_index: Option<usize>,
    /// 自定义窗口控制按钮区宽度（Windows 138；0 = 不画）。手绘在此区画 tab_bar_bg 底（DC-14 parity）。
    window_controls_width: f32,
    /// 各标签标题（DC-14 标签文本对齐手绘）。
    titles: Vec<String>,
    /// 标签文本色（取自 token，active 用 on_surface，inactive 调低对比度）。
    text_color: Color,
    muted_text_color: Color,
}

impl BrowserTabStripWidget {
    /// 由声明节点构造：`tab_count`（Int）/ `active_tab_index`（Int，-1 = 无）/
    /// `window_controls_width`（Float，0 = 无自定义窗口控制）props；tab 色经
    /// [`ChromeTabColors`] 注入（生产从 `ChromePalette`，测试从 token 近似）。
    /// `titles` 为 `Value::Array` 可选 prop（标签标题列表，长度不必匹配 tab_count）。
    pub fn from_spec(spec: &WidgetSpec, tab_colors: ChromeTabColors) -> BrowserTabStripWidget {
        let tab_count = match spec.props.get("tab_count") {
            Some(Value::Int(i)) => (*i).max(0) as usize,
            _ => 0,
        };
        let active_index = match spec.props.get("active_tab_index") {
            Some(Value::Int(i)) if *i >= 0 => Some(*i as usize),
            _ => None,
        };
        let window_controls_width = match spec.props.get("window_controls_width") {
            Some(Value::Float(f)) => *f as f32,
            Some(Value::Int(i)) => *i as f32,
            _ => 0.0,
        };
        let titles = match spec.props.get("titles") {
            Some(Value::Array(arr)) => arr
                .iter()
                .filter_map(|v| match v {
                    Value::Text(s) => Some(s.clone()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };
        // text_color 用 on_surface（≈ 手绘 tab_text）；muted_text_color ≈ 手绘 page_hint。
        let text_color = tab_colors.window_icon; // on_surface
        let muted_text_color = text_color.mix(tab_colors.strip_bg, 0.45);
        BrowserTabStripWidget {
            strip_bg: tab_colors.strip_bg,
            active_bg: tab_colors.active_bg,
            bar_bg: tab_colors.bar_bg,
            separator: tab_colors.separator,
            window_icon: tab_colors.window_icon,
            tab_count,
            active_index,
            window_controls_width,
            titles,
            text_color,
            muted_text_color,
        }
    }
}

impl Widget for BrowserTabStripWidget {
    fn mount(&mut self, _ctx: &mut MountCtx) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _props: &Props) {}

    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        // 全宽 bar（铺满 column 容器宽）+ 固定 TAB_STRIP_HEIGHT（clamp 到约束）。
        let width = constraints.max_width;
        let height = TAB_STRIP_HEIGHT.clamp(constraints.min_height, constraints.max_height);
        Size::new(width, height)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        let size = ctx
            .clip
            .map(|r| r.size)
            .unwrap_or_else(|| Size::new(1280.0, TAB_STRIP_HEIGHT));
        let width = size.width;
        // 1. strip 背景全宽（toolbar bg）。
        ctx.recorder
            .fill_rect(Rect::from_ltrb(0.0, 0.0, width, TAB_STRIP_HEIGHT), self.strip_bg);
        // 1b. 自定义窗口控制按钮区（Windows）：tab_bar_bg 底 + 3 图标（min/max/close），
        // 镜像手绘 render_window_controls（每按钮 bg=tab_bar_bg + 图标 window_control_icon）。
        if self.window_controls_width > 0.0 {
            let wc_x = (width - self.window_controls_width).max(0.0);
            ctx.recorder.fill_rect(
                Rect::from_ltrb(wc_x, TAB_BAR_TOP_INSET, width, TAB_BAR_TOP_INSET + TAB_BAR_HEIGHT),
                self.bar_bg,
            );
            // 3 按钮各 WINDOW_CONTROL_BTN_WIDTH(46) 宽，图标在按钮中心。
            // cy = tab bar 垂直中心；thickness = 1（与手绘 1px 描边一致）。
            let cy = TAB_BAR_TOP_INSET + TAB_BAR_HEIGHT * 0.5;
            let thickness = 1.0_f32;
            for i in 0..3u32 {
                let btn_cx = wc_x + (i as f32 + 0.5) * WINDOW_CONTROL_BTN_WIDTH;
                match i {
                    0 => {
                        // minimize：10px 横线，居中于 cy。
                        let line_w = 10.0_f32;
                        ctx.recorder.fill_rect(
                            Rect::from_ltrb(
                                btn_cx - line_w * 0.5,
                                cy - thickness * 0.5,
                                btn_cx + line_w * 0.5,
                                cy + thickness * 0.5,
                            ),
                            self.window_icon,
                        );
                    }
                    1 => {
                        // maximize：10px 空心方块（4 条 1px 边）。
                        let sz = 10.0_f32;
                        let left = btn_cx - sz * 0.5;
                        let top = cy - sz * 0.5;
                        // 上、下、左、右 四条边。
                        for (r, color) in [
                            (Rect::from_ltrb(left, top, left + sz, top + thickness), self.window_icon),
                            (
                                Rect::from_ltrb(left, top + sz - thickness, left + sz, top + sz),
                                self.window_icon,
                            ),
                            (Rect::from_ltrb(left, top, left + thickness, top + sz), self.window_icon),
                            (
                                Rect::from_ltrb(left + sz - thickness, top, left + sz, top + sz),
                                self.window_icon,
                            ),
                        ] {
                            ctx.recorder.fill_rect(r, color);
                        }
                    }
                    2 => {
                        // close：X 图标（SVG），12px，经 draw_image 着色（与 NAV_ICON 同模式）。
                        let icon_extent = 12.0_f32;
                        let x = btn_cx - icon_extent * 0.5;
                        let y = cy - icon_extent * 0.5;
                        ctx.recorder.draw_image(
                            Rect::from_ltrb(x, y, x + icon_extent, y + icon_extent),
                            WC_ICON_CLOSE,
                            self.window_icon,
                        );
                    }
                    _ => {}
                }
            }
        }
        if self.tab_count == 0 {
            return;
        }
        // 2. tab 宽布局（镜像 app_render.rs render_tabs：Windows leading=0、无 OS 窗口控件、无 pinned）。
        let leading = 0.0_f32;
        let window_controls_w = 0.0;
        let tabs_max_width = (width - window_controls_w - NEW_TAB_BTN_WIDTH - leading).max(0.0);
        let normal_count = self.tab_count as f32;
        let ideal = if normal_count > 0.0 {
            tabs_max_width / normal_count
        } else {
            0.0
        };
        let tab_w = ideal.clamp(TAB_MIN_WIDTH, TAB_MAX_WIDTH).max(0.0);
        let tab_body_w = (tab_w - 1.0).max(0.0); // 手绘 tab_body_w = tab_w - scale。
        let tab_y = TAB_BAR_TOP_INSET;
        let active = self.active_index;
        // 3. 非激活 tab（顶部圆角 bar 底）先画。
        for i in 0..self.tab_count {
            if Some(i) == active {
                continue;
            }
            let x = leading + i as f32 * tab_w;
            paint_top_rounded_scanlines(
                ctx.recorder,
                x,
                tab_y,
                tab_body_w,
                TAB_BAR_HEIGHT,
                TAB_TOP_RADIUS,
                self.bar_bg,
            );
        }
        // 4. 激活 tab（Chrome foot 形状白底）画在 inactive 之上（foot 覆盖相邻 inactive 边）。
        if let Some(ai) = active.filter(|&a| a < self.tab_count) {
            let x = leading + ai as f32 * tab_w;
            paint_active_tab_scanlines(
                ctx.recorder,
                x,
                tab_y,
                tab_body_w,
                TAB_BAR_HEIGHT,
                TAB_TOP_RADIUS,
                TAB_FOOT_RADIUS,
                self.active_bg,
            );
        }
        // 5. 相邻非激活 tab 间分隔线（在 tab 底色之上；与手绘一致：仅相邻均非激活时画）。
        let sep_w = 1.0_f32;
        for i in 0..self.tab_count.saturating_sub(1) {
            if Some(i) == active || Some(i + 1) == active {
                continue;
            }
            let gap_center = leading + (i + 1) as f32 * tab_w - 0.5;
            ctx.recorder.fill_rect(
                Rect::from_ltrb(
                    gap_center - sep_w * 0.5,
                    tab_y + TAB_SEPARATOR_INSET,
                    gap_center + sep_w * 0.5,
                    tab_y + TAB_BAR_HEIGHT - TAB_SEPARATOR_INSET,
                ),
                self.separator,
            );
        }
        // 6. 标签标题文本（对齐手绘 render_tabs 的 draw_ui_text 路径）。
        // 手绘 text_left_inset = 12*s + icon_size(16) + 8*s = 36（s=1）。
        // 可用文本宽 = tab_w - 36 - 32（close 按钮预留）= tab_w - 68。
        // baseline 计算：手绘经 ui_text_centered_in_height → text_top + ascent。
        const TAB_TEXT_LEFT_INSET: f32 = 36.0;
        const TAB_TEXT_RIGHT_RESERVE: f32 = 32.0;
        let tab_text_baseline = tab_y + TAB_BAR_HEIGHT * 0.5 + 13.0 * 0.35;
        for i in 0..self.tab_count {
            let label = self.titles.get(i).map(|s| s.as_str()).unwrap_or("");
            let tab_x = leading + i as f32 * tab_w;
            let useable_w = (tab_body_w - TAB_TEXT_LEFT_INSET - TAB_TEXT_RIGHT_RESERVE).max(0.0);
            // char-count 近似截断（~8px/char at 13px）。
            let max_chars = (useable_w / 8.0).floor() as usize;
            let display = if label.chars().count() > max_chars && max_chars > 1 {
                format!("{}…", label.chars().take(max_chars.saturating_sub(1)).collect::<String>())
            } else {
                label.to_string()
            };
            if !display.is_empty() {
                let color = if Some(i) == self.active_index { self.text_color } else { self.muted_text_color };
                ctx.recorder.draw_text(
                    &display,
                    Point::new(tab_x + TAB_TEXT_LEFT_INSET, tab_text_baseline),
                    13.0,
                    color,
                );
            }
        }
    }

    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

// ── BookmarksBarWidget（真实书签栏，DC-14 chrome 功能等价）──────────────────────

const BOOKMARKS_BAR_HEIGHT: f32 = 28.0;

/// 书签栏绘制控件：填充背景 + 书签标题列表（bookmarks prop）。
pub struct BookmarksBarWidget {
    bg: Color,
    bookmarks: Vec<String>,
    text_color: Color,
}

impl BookmarksBarWidget {
    /// 由声明节点构造：`bg` + `bookmarks`（`Value::Array`，可选）。
    pub fn from_spec(spec: &WidgetSpec, tokens: &SemanticTokens) -> BookmarksBarWidget {
        let bg_name = match spec.props.get("bg") {
            Some(Value::Text(s)) => s.as_str(),
            _ => "chrome",
        };
        let bookmarks = match spec.props.get("bookmarks") {
            Some(Value::Array(arr)) => arr
                .iter()
                .filter_map(|v| match v {
                    Value::Text(s) => Some(s.clone()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };
        BookmarksBarWidget {
            bg: chrome_color_themed(bg_name, tokens),
            bookmarks,
            text_color: tokens.on_surface,
        }
    }
}

impl Widget for BookmarksBarWidget {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, _ctx: &mut UpdateCtx, _props: &Props) {}
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        let width = constraints.max_width;
        let height = BOOKMARKS_BAR_HEIGHT.clamp(constraints.min_height, constraints.max_height);
        Size::new(width, height)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        let size = ctx.clip.map(|r| r.size).unwrap_or_else(|| Size::new(400.0, BOOKMARKS_BAR_HEIGHT));
        ctx.recorder
            .fill_rect(Rect::from_ltrb(0.0, 0.0, size.width, size.height), self.bg);
        if !self.bookmarks.is_empty() {
            let baseline = size.height * 0.5 + 13.0 * 0.32;
            let mut x = 8.0;
            for bm in &self.bookmarks {
                if x >= size.width - 8.0 {
                    break;
                }
                ctx.recorder.draw_text(bm, Point::new(x, baseline), 13.0, self.text_color);
                x += 8.0 + bm.chars().count() as f32 * 7.5;
            }
        }
    }

    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

// ── FindBarWidget（真实查找栏，DC-14 chrome 功能等价）────────────────────────

const FIND_BAR_HEIGHT: f32 = 36.0;

/// 查找栏绘制控件：背景 + 查询文本 + 匹配计数。
pub struct FindBarWidget {
    bg: Color,
    query: String,
    match_index: Option<i64>,
    match_count: Option<i64>,
    text_color: Color,
}

impl FindBarWidget {
    pub fn from_spec(spec: &WidgetSpec, tokens: &SemanticTokens) -> FindBarWidget {
        let bg_name = match spec.props.get("bg") {
            Some(Value::Text(s)) => s.as_str(),
            _ => "chrome",
        };
        let query = match spec.props.get("query") {
            Some(Value::Text(s)) => s.clone(),
            _ => String::new(),
        };
        let match_index = match spec.props.get("match_index") {
            Some(Value::Int(i)) if *i >= 0 => Some(*i),
            _ => None,
        };
        let match_count = match spec.props.get("match_count") {
            Some(Value::Int(i)) if *i >= 0 => Some(*i),
            _ => None,
        };
        FindBarWidget {
            bg: chrome_color_themed(bg_name, tokens),
            query,
            match_index,
            match_count,
            text_color: tokens.on_surface,
        }
    }
}

impl Widget for FindBarWidget {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, _ctx: &mut UpdateCtx, _props: &Props) {}
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        let width = constraints.max_width;
        let height = FIND_BAR_HEIGHT.clamp(constraints.min_height, constraints.max_height);
        Size::new(width, height)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        let size = ctx.clip.map(|r| r.size).unwrap_or_else(|| Size::new(400.0, FIND_BAR_HEIGHT));
        ctx.recorder
            .fill_rect(Rect::from_ltrb(0.0, 0.0, size.width, size.height), self.bg);
        let label = if let (Some(idx), Some(cnt)) = (self.match_index, self.match_count) {
            if self.query.is_empty() {
                format!("Find  ({}/{cnt})", idx + 1)
            } else {
                format!("\"{}\" ({}/{cnt})", self.query, idx + 1)
            }
        } else if self.query.is_empty() {
            "Find".to_string()
        } else {
            format!("\"{}\"", self.query)
        };
        let baseline = size.height * 0.5 + 13.0 * 0.35;
        ctx.recorder.draw_text(&label, Point::new(12.0, baseline), 13.0, self.text_color);
    }

    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

// ── SecurityBadgeWidget（真实安全徽章，DC-14 chrome 功能等价）───────────────────

const SECURITY_BADGE_HEIGHT: f32 = 44.0;

/// 安全徽章绘制控件：根据安全状态显示彩色背景条 + 状态文本。
pub struct SecurityBadgeWidget {
    bg: Color,
    text: Option<String>,
    text_color: Color,
}

impl SecurityBadgeWidget {
    pub fn from_spec(spec: &WidgetSpec, tokens: &SemanticTokens) -> SecurityBadgeWidget {
        let bg_name = match spec.props.get("bg") {
            Some(Value::Text(s)) => s.as_str(),
            _ => "secure",
        };
        let text = match spec.props.get("text") {
            Some(Value::Text(s)) => Some(s.clone()),
            _ => None,
        };
        SecurityBadgeWidget {
            bg: chrome_color_themed(bg_name, tokens),
            text,
            text_color: tokens.on_surface,
        }
    }
}

impl Widget for SecurityBadgeWidget {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, _ctx: &mut UpdateCtx, _props: &Props) {}
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        let width = constraints.max_width;
        let height = SECURITY_BADGE_HEIGHT.clamp(constraints.min_height, constraints.max_height);
        Size::new(width, height)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        let size = ctx.clip.map(|r| r.size).unwrap_or_else(|| Size::new(400.0, SECURITY_BADGE_HEIGHT));
        ctx.recorder
            .fill_rect(Rect::from_ltrb(0.0, 0.0, size.width, size.height), self.bg);
        if let Some(text) = &self.text {
            let baseline = size.height * 0.5 + 13.0 * 0.35;
            ctx.recorder.draw_text(text, Point::new(8.0, baseline), 13.0, self.text_color);
        }
    }

    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

// ── AddressBarWidget（真实地址栏 pill，DC-14 chrome 功能等价）──────────────────────

/// 地址栏几何（与 apps/browser/src/layout.rs 手绘 chrome 对齐，DC-14 像素级等价）。
///
/// 手绘 `address_bar_layout`：toolbar 行高 ADDRESS_BAR_HEIGHT=44，地址栏垂直 inset
/// ADDRESS_BAR_INPUT_V_INSET=6 → bar_h = 44 - 2*6 = 32，bar_y = toolbar_top + 6。
/// radius = bar_h * 0.5 = 16（完整半圆 pill）。border 1px（失焦态）。
const ADDRESS_BAR_HEIGHT: f32 = 32.0;
const ADDRESS_BAR_RADIUS: f32 = 16.0; // = ADDRESS_BAR_HEIGHT * 0.5。
const ADDRESS_BAR_BORDER_WIDTH: f32 = 1.0;

/// 地址栏 pill 绠制控件（DC-14：真实 pill border 替换 ChromePanel 占位）。
///
/// paint 用**双层 fill_rounded_rect** 镜像手绘 `render_address_bar`：外层 address_bar_border(218)
/// 与内层 bg(248) inset border_width（手绘如此实现 border，非 stroke；SDK 复用 fill_rounded_rect
/// 无需新原语）。layout 高 32（toolbar 44 内 inset 6），宽填满（flex）；垂直居中由 shell 在
/// toolbar 行声明 cross_axis_align=center 提供（bar_y = toolbar_top + 6）。
///
/// URL 文本经 `draw_text` 渲染在 security slot 之后（text_x = border + INNER_PAD_H + LEADING_SLOT）。
/// 文本为空时渲染 placeholder "Search or enter URL..."（对齐手绘 `render_address_bar`）。
/// security 状态图标 / focus 态 为后续轮次。
pub struct AddressBarWidget {
    bg: Color,
    border_color: Color,
    text: Option<String>,
    text_color: Color,
    /// placeholder 文本色（手绘 `address_bar_placeholder`：muted gray，≈ on_surface 混 surface）。
    placeholder_color: Color,
    /// security 状态名（"secure"/"not-secure"/"mixed-content"/"dangerous"；默认 "secure"）。
    /// 手绘 chrome：Secure 态不画图标（slot 空），非 Secure 在 slot 画 "!" 等。SDK 复此逻辑。
    security_state: String,
    slot_divider_color: Color,
    insecure_color: Color,
}

impl AddressBarWidget {
    /// 由声明节点构造：`bg` prop（默认 `address_bar_bg`）经 semantic token 解析；
    /// border 色经 [`ChromeTabColors::address_border`]（生产从 `ChromePalette` 精确注入）；
    /// `text` prop（URL 文案）+ 文案色 = on_background token（对齐手绘 address_bar_text）。
    pub fn from_spec(spec: &WidgetSpec, tokens: &SemanticTokens, tab_colors: ChromeTabColors) -> AddressBarWidget {
        let bg_name = match spec.props.get("bg") {
            Some(Value::Text(s)) => s.as_str(),
            _ => "address_bar_bg",
        };
        let text = match spec.props.get("text") {
            Some(Value::Text(s)) => Some(s.clone()),
            _ => None,
        };
        let security_state = match spec.props.get("security_state") {
            Some(Value::Text(s)) => s.to_string(),
            _ => "secure".to_string(),
        };
        // placeholder 色 ≈ 手绘 chrome `address_bar_placeholder`（muted gray）：
        // on_surface 混 surface 约 0.55 在 light 下 ≈ rgb(128,134,139)，dark 下 ≈ rgb(160,160,160)。
        let placeholder_color = tokens.on_surface.mix(tokens.surface, 0.55);
        AddressBarWidget {
            bg: chrome_color_themed(bg_name, tokens),
            border_color: tab_colors.address_border,
            text,
            text_color: tokens.on_background,
            placeholder_color,
            security_state,
            slot_divider_color: tokens.on_surface.mix(tokens.surface, 0.6),
            insecure_color: tokens.warning,
        }
    }
}

impl Widget for AddressBarWidget {
    fn mount(&mut self, _ctx: &mut MountCtx) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _props: &Props) {}

    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        // 宽填满（flex 由 shell 设；layout 取 max_width）+ 固定 32 高。
        let width = constraints.max_width;
        let height = ADDRESS_BAR_HEIGHT.clamp(constraints.min_height, constraints.max_height);
        Size::new(width, height)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        let size = ctx
            .clip
            .map(|r| r.size)
            .unwrap_or_else(|| Size::new(400.0, ADDRESS_BAR_HEIGHT));
        let w = size.width;
        let h = size.height;
        let bw = ADDRESS_BAR_BORDER_WIDTH;
        // 外层 border（圆角 pill，border 色）。
        ctx.recorder
            .fill_rounded_rect(Rect::from_ltrb(0.0, 0.0, w, h), ADDRESS_BAR_RADIUS, self.border_color);
        // 内层 bg（inset border_width，radius 减 border_width）。
        let inner = Rect::from_ltrb(bw, bw, (w - bw).max(bw), (h - bw).max(bw));
        ctx.recorder
            .fill_rounded_rect(inner, (ADDRESS_BAR_RADIUS - bw).max(0.0), self.bg);
        // URL 文本：security slot 之后（border + INNER_PAD_H + LEADING_SLOT_WIDTH = 1+12+28 = 41）。
        // 基线 ≈ bar_h/2 + font_ascent 偏移（对齐手绘 ui_text_centered_in_height，font 13）。
        const ADDRESS_BAR_INNER_PAD_H: f32 = 12.0;
        const ADDRESS_BAR_LEADING_SLOT_WIDTH: f32 = 28.0;
        let slot_x = bw + ADDRESS_BAR_INNER_PAD_H; // slot 起（图标区）
        let text_x = slot_x + ADDRESS_BAR_LEADING_SLOT_WIDTH; // slot 后文本起
        let baseline = h * 0.5 + 13.0 * 0.35;
        // security slot 分隔线（1px 竖线，对齐手绘 slot_divider； inset 6 顶/底）。
        let divider_inset = 6.0_f32;
        ctx.recorder.fill_rect(
            Rect::from_ltrb(
                text_x,
                divider_inset,
                text_x + 1.0,
                (h - divider_inset).max(divider_inset),
            ),
            self.slot_divider_color,
        );
        // security 状态图标：Secure = 无（对齐手绘 Secure→None）；非 Secure 在 slot 中心画 "!"。
        // 图标中心 = slot_x + LEADING_SLOT/2 = slot_x + 14。
        if self.security_state != "secure" {
            let icon_x = slot_x + ADDRESS_BAR_LEADING_SLOT_WIDTH * 0.5;
            ctx.recorder
                .draw_text("!", Point::new(icon_x - 3.0, baseline), 13.0, self.insecure_color);
        }
        // 文本渲染：若 text 非空渲染 URL（用 text_color），否则渲染 placeholder 文案
        //（用 placeholder_color，对齐手绘 address_bar_placeholder 色）。
        // DC-11 per-char 文本路径与手绘 `draw_ui_text` 的 advance 逐位一致（含空格）。
        let (display_text, display_color) = match &self.text {
            Some(t) if !t.is_empty() => (t.as_str(), self.text_color),
            _ => ("Search or enter URL...", self.placeholder_color),
        };
        ctx.recorder
            .draw_text(display_text, Point::new(text_x, baseline), 13.0, display_color);
    }

    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// 把 chrome `browser.*` 叶子组件工厂注册到 host。
///
/// 容器节点（shell 根 / ToolbarRow）不注册 widget —— 它们经 `props.layout` 由 host 布局。
/// 调用方仍需先 `host.set_root(&shell.build(...))` 注入声明树。
///
/// `tab_colors`：tab 专属色（active/inactive/separator/strip bg），生产从浏览器 `ChromePalette`
/// 构造（DC-14 parity）；测试可用 [`ChromeTabColors::from_tokens`] 近似。
pub fn register_chrome_factories(host: &mut WidgetHost, tokens: &SemanticTokens, tab_colors: ChromeTabColors) {
    // bars：固定高度的语义色条 + 可选文案；颜色经 semantic token 解析（DC-5）。
    // 高度对齐 apps/browser/src/layout.rs 手绘 chrome（DC-14 像素级等价）：
    //   TAB_STRIP_HEIGHT = TAB_BAR_TOP_INSET(6) + TAB_BAR_HEIGHT(34) = 40
    //   ADDRESS_BAR_HEIGHT = 44（地址行：AddressBar / NavigationButtons / BrowserMenu / SecurityBadge）
    //   BOOKMARKS_BAR_HEIGHT = 28
    // SemanticTokens / ChromeTabColors 是 Copy：每个 move 闭包各持一份副本。
    let t = *tokens;
    let tc = tab_colors;
    host.register("browser.AddressBar", move |s| {
        Box::new(AddressBarWidget::from_spec(s, &t, tc))
    });
    host.register("browser.NavigationButtons", move |s| {
        Box::new(NavigationButtonsWidget::from_spec(s, &t))
    });
    host.register("browser.BrowserMenu", move |s| {
        Box::new(MenuButtonWidget::from_spec(s, &t))
    });
    host.register("browser.BrowserTabStrip", move |s| {
        Box::new(BrowserTabStripWidget::from_spec(s, tc))
    });
    host.register("browser.BookmarksBar", move |s| {
        Box::new(BookmarksBarWidget::from_spec(s, &t))
    });
    host.register("browser.SecurityBadge", move |s| {
        Box::new(SecurityBadgeWidget::from_spec(s, &t))
    });
    host.register("browser.FindBar", move |s| {
        Box::new(FindBarWidget::from_spec(s, &t))
    });
    // 视口：占满剩余区域，**不填底色**（DC-14 替换式迁移：页面内容由 WebView 绘制在上层，
    // SDK chrome viewport 只负责布局空间，不覆盖页面像素）。
    host.register("browser.PageViewportFrame", move |s| {
        Box::new(ChromePanel::from_spec_with_fill(s, "viewport", 0.0, true, false, &t))
    });
}

/// 同 [`register_chrome_factories`]，但 viewport 使用 [`WebViewWidget`] 替代 [`ChromePanel`]
/// 占位符（DC-3 phase-2）。WebViewWidget paint 产出 `ExternalSurface` marker，供后端按
/// `surface_id` 合成 WebView 纹理。
///
/// props 约定：`surface_id`（u64，默认 0）由宿主分配；`scale_factor`（f32，默认 1.0）。
/// viewport rect 由 layout 填充可用空间确定。
pub fn register_chrome_factories_with_webview(
    host: &mut WidgetHost,
    tokens: &SemanticTokens,
    scheme: zero_ui_core::theme::ResolvedColorScheme,
    tab_colors: ChromeTabColors,
) {
    register_chrome_factories(host, tokens, tab_colors);
    // 替换 PageViewportFrame：使用 WebViewWidget 产出 ExternalSurface marker。
    let theme = zero_ui_core::theme::ThemeResolver::build_theme(
        zero_ui_core::theme::ThemeId::new("zero"),
        "Zero",
        scheme,
        zero_ui_core::theme::ColorPalette::default(),
    );
    host.register("browser.PageViewportFrame", move |s| {
        let surface_id = match s.props.get("surface_id") {
            Some(zero_ui_core::binding::Value::Int(v)) => *v as u64,
            _ => 0,
        };
        let scale_factor = match s.props.get("scale_factor") {
            Some(zero_ui_core::binding::Value::Float(v)) => *v as f32,
            _ => 1.0,
        };
        Box::new(
            zero_ui_adapter_webview::WebViewWidget::new(
                zero_ui_core::geometry::Rect::from_ltrb(0.0, 0.0, 0.0, 0.0),
                scale_factor,
                theme.clone(),
            )
            .with_surface_id(surface_id),
        )
    });
}

/// 取 Scene 中所有文本图元的文案（断言用）。
pub fn scene_texts(scene: &Scene) -> Vec<String> {
    use zero_ui_render::render_node::RenderPrimitive;
    scene
        .entries
        .iter()
        .filter_map(|e| match &e.primitive {
            RenderPrimitive::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BrowserChromeShell;
    use crate::chrome_model::BrowserChromeModel;
    use crate::shell::DesktopBrowserShell;
    use zero_ui_core::geometry::{Insets, Size};
    use zero_ui_core::layout::WindowMetrics;
    use zero_ui_core::widget::WidgetId;
    use zero_ui_render::render_node::RenderPrimitive;

    fn metrics(w: f32, h: f32) -> WindowMetrics {
        let logical_size = Size::new(w, h);
        WindowMetrics {
            logical_size,
            scale_factor: 1.0,
            safe_area: Insets::all(0.0),
            keyboard_insets: Insets::all(0.0),
            text_scale: 1.0,
            density: 1.0,
            orientation: zero_ui_core::layout::Orientation::from_size(logical_size),
        }
    }

    #[test]
    fn navigation_buttons_widget_draws_bg_and_four_icons() {
        // DC-14 真实 nav 图标控件：填 nav 段背景 + 4 图标（back/forward/reload/home）。
        // can_back=true / can_forward=false → forward 用 disabled tint（不同 cache key）。
        use zero_ui_adapter_render_foundation::RenderFoundationBackend;
        use zero_ui_core::binding::Value;
        use zero_ui_core::geometry::Constraints;
        use zero_ui_core::widget::WidgetSpec;
        use zero_ui_render::paint_scene;

        let mut nav = WidgetSpec::new("browser.NavigationButtons");
        nav.id = Some(WidgetId::new("nav"));
        nav.props.insert("bg", Value::Text("toolbar_bg".into()));
        nav.props.insert("can_back", Value::Bool(true));
        nav.props.insert("can_forward", Value::Bool(false));

        let tokens = SemanticTokens::light();
        let mut host = WidgetHost::new();
        register_chrome_factories(&mut host, &tokens, ChromeTabColors::from_tokens(&tokens));
        host.set_root(&nav);
        let sz = host.layout(Constraints::loose(Size::new(400.0, 44.0)));
        // leading pad(10) + 4 按钮 × 36px + trailing pad(20) = 174（含 NAV_SECTION_TRAILING_GAP +
        // ADDRESS_BAR_PADDING，对齐手绘 bar_x=174）。
        assert!(
            (sz.width - (NAV_LEADING_PAD + 4.0 * NAV_BUTTON_WIDTH + NAV_TRAILING_PAD)).abs() < 0.01,
            "nav width 174, got {}",
            sz.width
        );
        let scene = host.paint().clone();

        // 注册 4 图标 alpha 掩码到桥接 + paint_scene。
        let mut bridge = RenderFoundationBackend::new_with_text_size(
            Size::new(400.0, 44.0),
            std::sync::Arc::new(zero_text_foundation::FontdueBackend::new()),
        );
        for key in [NAV_ICON_BACK, NAV_ICON_FORWARD, NAV_ICON_RELOAD, NAV_ICON_HOME] {
            bridge.register_image_mask(key, vec![255], 1, 1);
        }
        paint_scene(&scene, &mut bridge);
        let p = bridge.primitives();
        // 1 个背景 fill（nav 段）。
        assert_eq!(p.fills.len(), 1, "nav bg fill, got {:?}", p.fills.len());
        // 4 个 ImagePrimitive（back/forward/reload/home）。
        assert_eq!(p.images.len(), 4, "4 nav icons, got {}", p.images.len());
        // 自左向右排列：每槽 36px，首槽图标 x ≈ NAV_LEADING_PAD + (36-16)/2 = 20。
        let mut xs: Vec<f32> = p.images.iter().map(|i| i.rect.origin.x).collect();
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!((xs[0] - 20.0).abs() < 0.01, "first icon x≈20, got {}", xs[0]);
        assert!(
            xs[3] - xs[0] >= (3.0 * NAV_BUTTON_WIDTH) - 0.01,
            "icons span 4 slots, first={} last={}",
            xs[0],
            xs[3]
        );
    }

    #[test]
    fn desktop_shell_paints_into_unified_scene() {
        // DC-7：chrome 组件输出进统一 Scene（不绕过 ui/render）。
        let mut model = BrowserChromeModel::new();
        model.address_text = "https://example.com".into();
        model.security = crate::security_badge::SecurityState::Secure;
        model.tabs = vec![crate::BrowserTab {
            id: zero_browser_shell::TabId(1),
            title: "Example".into(),
            loading: false,
        }];
        model.active_tab_index = Some(0);

        let m = metrics(1280.0, 800.0);
        let spec = DesktopBrowserShell.build(&model, &m);

        let mut host = WidgetHost::new();
        register_chrome_factories(
            &mut host,
            &zero_ui_core::theme::SemanticTokens::light(),
            ChromeTabColors::from_tokens(&zero_ui_core::theme::SemanticTokens::light()),
        );
        host.set_root(&spec);
        let root_size = host.layout(zero_ui_core::geometry::Constraints::loose(Size::new(
            m.logical_size.width,
            m.logical_size.height,
        )));
        // shell 根为 column 容器，铺满视口。
        assert!((root_size.width - 1280.0).abs() < 0.01);
        let scene = host.paint().clone();

        // 1. 地址栏 URL 文案进 Scene。
        let texts = scene_texts(&scene);
        assert!(
            texts.iter().any(|t| t.contains("example.com")),
            "address URL must render into scene, got {texts:?}"
        );
        // 2. 至少有 toolbar/tabstrip/bookmarks/address 的背景 fill + 视口 fill。
        let fills = scene
            .entries
            .iter()
            .filter(|e| matches!(e.primitive, RenderPrimitive::FillRect { .. }))
            .count();
        assert!(fills >= 4, "expected several chrome bar fills, got {fills}");
        // 3. 视口节点（fill_viewport）铺满剩余高度：rect 高接近 800-顶部bars。
        let vp = host.rect_of(&WidgetId::new("viewport")).expect("viewport laid out");
        assert!(
            vp.size.height > 600.0,
            "viewport should fill remaining height, got {}",
            vp.size.height
        );
    }

    #[test]
    fn security_state_drives_badge_color() {
        // 安全状态 → chrome 色名 → semantic token → Color（DC-5：经 token 解析）。
        let light = zero_ui_core::theme::SemanticTokens::light();
        let dark = zero_ui_core::theme::SemanticTokens::dark();
        assert_eq!(
            security_color_name(crate::security_badge::SecurityState::Secure),
            "secure"
        );
        // dangerous → error token（浅色/深色不同色值，证明经主题解析而非硬编码）。
        assert_eq!(chrome_color_themed("dangerous", &light), light.error);
        assert_eq!(chrome_color_themed("dangerous", &dark), dark.error);
        // secure → success token。
        assert_eq!(chrome_color_themed("secure", &light), light.success);
        // chrome → surface；viewport → background；未知 → surface 回落。
        assert_eq!(chrome_color_themed("chrome", &light), light.surface);
        assert_eq!(chrome_color_themed("viewport", &light), light.background);
        assert_eq!(chrome_color_themed("unknown", &light), light.surface);
        // accent → primary。
        assert_eq!(chrome_color_themed("accent", &light), light.primary);
    }

    #[test]
    fn chrome_panel_default_text_color_uses_token() {
        // ChromePanel 默认文案色 = on_background token（DC-14：chrome 文案色经 ChromePalette 映射
        // on_background ← address_bar_text；不硬编码，随主题切换）。
        let light = zero_ui_core::theme::SemanticTokens::light();
        let mut spec = WidgetSpec::new("browser.AddressBar");
        spec.props.insert("bg", Value::Text("chrome".into()));
        let panel = ChromePanel::from_spec(&spec, "chrome", 36.0, false, &light);
        assert_eq!(panel.text_color, light.on_background);
        assert_eq!(panel.bg, light.surface);
    }

    #[test]
    fn viewport_frame_skips_background_fill() {
        // DC-14 替换式迁移：PageViewportFrame 不填底色（页面内容由 WebView 绘制）。
        let light = zero_ui_core::theme::SemanticTokens::light();
        let spec = WidgetSpec::new("browser.PageViewportFrame");
        let panel = ChromePanel::from_spec_with_fill(&spec, "viewport", 0.0, true, false, &light);
        assert!(!panel.fill_background, "PageViewportFrame fill_background = false");
        assert!(panel.fill_viewport, "fill_viewport 仍为 true（占满剩余空间）");
    }

    #[test]
    fn bars_keep_background_fill_default() {
        // 非视口 chrome 控件（AddressBar/TabStrip 等）默认 fill_background = true（向后兼容）。
        let light = zero_ui_core::theme::SemanticTokens::light();
        let mut spec = WidgetSpec::new("browser.AddressBar");
        spec.props.insert("bg", Value::Text("chrome".into()));
        let panel = ChromePanel::from_spec(&spec, "chrome", 36.0, false, &light);
        assert!(panel.fill_background, "bars 默认 fill_background = true");
    }

    #[test]
    fn webview_viewport_paints_external_surface() {
        // DC-3 phase-2：register_chrome_factories_with_webview 注册 WebViewWidget 为
        // PageViewportFrame，paint 产出 ExternalSurface marker（而非 ChromePanel 的 fill_rect）。

        use zero_ui_core::geometry::Size;
        use zero_ui_render::render_node::RenderPrimitive;
        use zero_ui_runtime::WidgetHost;

        let tokens = zero_ui_core::theme::SemanticTokens::light();
        let mut host = WidgetHost::new();
        register_chrome_factories_with_webview(
            &mut host,
            &tokens,
            zero_ui_core::theme::ResolvedColorScheme::Light,
            ChromeTabColors::from_tokens(&tokens),
        );

        // 构造含 surface_id 的 PageViewportFrame spec。
        let mut spec = WidgetSpec::new("browser.PageViewportFrame");
        spec.id = Some(zero_ui_core::widget::WidgetId::new("viewport"));
        spec.props.insert("surface_id", Value::Int(42));
        spec.props.insert("scale_factor", Value::Float(2.0));
        host.set_root(&spec);
        host.layout(zero_ui_core::geometry::Constraints::loose(Size::new(1280.0, 800.0)));

        let scene = host.paint().clone();
        let external = scene
            .entries
            .iter()
            .find(|e| matches!(e.primitive, RenderPrimitive::ExternalSurface { .. }));
        assert!(
            external.is_some(),
            "WebViewWidget factory should produce ExternalSurface marker"
        );
    }

    #[test]
    fn bookmarks_bar_widget_paints_bg_and_text() {
        let tokens = zero_ui_core::theme::SemanticTokens::light();
        let mut spec = WidgetSpec::new("browser.BookmarksBar");
        spec.id = Some(zero_ui_core::widget::WidgetId::new("bookmarks_bar"));
        spec.props.insert("bg", Value::Text("toolbar_bg".into()));
        spec.props.insert("bookmarks", Value::Array(vec![
            Value::Text("MDN".into()),
            Value::Text("GitHub".into()),
        ]));
        let mut host = zero_ui_runtime::WidgetHost::new();
        let t = tokens;
        host.register("browser.BookmarksBar", move |s| {
            Box::new(BookmarksBarWidget::from_spec(s, &t))
        });
        host.set_root(&spec);
        host.layout(zero_ui_core::geometry::Constraints::loose(Size::new(1280.0, 800.0)));
        let scene = host.paint().clone();
        let texts = scene_texts(&scene);
        assert!(texts.iter().any(|t| t.contains("MDN")), "MDN label, got {texts:?}");
        assert!(texts.iter().any(|t| t.contains("GitHub")), "GitHub label, got {texts:?}");
    }

    #[test]
    fn find_bar_widget_paints_bg_and_label() {
        let tokens = zero_ui_core::theme::SemanticTokens::light();
        let mut spec = WidgetSpec::new("browser.FindBar");
        spec.id = Some(zero_ui_core::widget::WidgetId::new("find_bar"));
        spec.props.insert("bg", Value::Text("toolbar_bg".into()));
        spec.props.insert("query", Value::Text("hello".into()));
        spec.props.insert("match_index", Value::Int(0));
        spec.props.insert("match_count", Value::Int(5));
        let mut host = zero_ui_runtime::WidgetHost::new();
        let t = tokens;
        host.register("browser.FindBar", move |s| {
            Box::new(FindBarWidget::from_spec(s, &t))
        });
        host.set_root(&spec);
        host.layout(zero_ui_core::geometry::Constraints::loose(Size::new(1280.0, 800.0)));
        let scene = host.paint().clone();
        let texts = scene_texts(&scene);
        assert!(texts.iter().any(|t| t.contains("hello")), "query text, got {texts:?}");
        assert!(texts.iter().any(|t| t.contains("1/5")), "match count, got {texts:?}");
    }

    #[test]
    fn security_badge_widget_paints_bg_and_label() {
        let tokens = zero_ui_core::theme::SemanticTokens::light();
        let mut spec = WidgetSpec::new("browser.SecurityBadge");
        spec.id = Some(zero_ui_core::widget::WidgetId::new("security_badge"));
        spec.props.insert("bg", Value::Text("secure".into()));
        spec.props.insert("text", Value::Text("Secure".into()));
        let mut host = zero_ui_runtime::WidgetHost::new();
        let t = tokens;
        host.register("browser.SecurityBadge", move |s| {
            Box::new(SecurityBadgeWidget::from_spec(s, &t))
        });
        host.set_root(&spec);
        host.layout(zero_ui_core::geometry::Constraints::loose(Size::new(1280.0, 800.0)));
        let scene = host.paint().clone();
        let texts = scene_texts(&scene);
        assert!(texts.iter().any(|t| t.contains("Secure")), "security label, got {texts:?}");
    }
}
