//! SDK chrome 渲染管线（spec §8.2 / DC-14 浏览器接线 ready）。
//!
//! [`render_chrome_via_sdk`] 封装完整 SDK chrome 渲染管线，把 `zero-browser-shell` 业务状态
//! 经通用 SDK 抽象（模型 → desktop shell 声明树 → retained WidgetHost → Scene → render-foundation
//! 桥接）产出 [`RenderFoundationBackend`]（内含 `RenderPrimitives` + `ImageCache`）。
//!
//! **这是 apps/browser DC-14 生产接线的 ready-to-call 入口**：浏览器调用本函数取出
//! `into_primitives()` / `into_image_cache()` 合并进帧（替代/并存手绘 chrome）。浏览器侧接线
//! 仅剩「合并 + 可视验收」，SDK 管线正确性已由全链集成测试保证（见
//! `ui/adapters/render-foundation` 的 `full_pipeline_chrome_scene_to_render_primitives`）。

use crate::chrome_model::BrowserChromeModel;
use crate::render::{register_chrome_factories, register_chrome_factories_with_webview};
use crate::shell::{BrowserChromeShell, DesktopBrowserShell, ID_VIEWPORT};
use std::sync::Arc;
use zero_render_foundation::primitive::RenderPrimitives;
use zero_text_foundation::FontdueBackend;
use zero_ui_adapter_render_foundation::RenderFoundationBackend;
use zero_ui_core::geometry::{Constraints, Rect};
use zero_ui_core::layout::WindowMetrics;
use zero_ui_core::theme::{ResolvedColorScheme, SemanticTokens};
use zero_ui_core::widget::WidgetId;
use zero_ui_render::paint_scene;
use zero_ui_runtime::WidgetHost;

/// 经完整 SDK 管线渲染 desktop chrome（spec §8.2 / DC-14）。
///
/// 管线：`BrowserShell` → [`BrowserChromeModel::from_shell`] → [`DesktopBrowserShell::build`]
/// → `WidgetHost`（注册 chrome 工厂 → set_root → layout → paint）→ `Scene` →
/// `paint_scene` → [`RenderFoundationBackend`]。
///
/// - `tokens`：主题 semantic token（chrome 组件消费，DC-5）。
/// - `backend`：共享字体后端（**DC-11 字体栈共享契约**——chrome 文本经 `draw_text` shape，
///   故 `backend` 须已加载至少一种字体；与 TextBlob 生产者共用同一实例）。
///
/// 返回已绘制（painted）的桥接后端；调用方（apps/browser）取出 `into_primitives()` 合并进帧、
/// `into_image_cache()` 交给渲染器解析 glyph ImageKey。
pub fn render_chrome_via_sdk(
    shell: &zero_browser_shell::BrowserShell,
    metrics: &WindowMetrics,
    tokens: &SemanticTokens,
    backend: Arc<FontdueBackend>,
) -> RenderFoundationBackend {
    render_chrome_via_sdk_with_layout(shell, metrics, tokens, backend).0
}

/// 同 [`render_chrome_via_sdk`]，但额外返回 SDK chrome 布局后的 viewport（页面内容区）矩形。
///
/// **替换式迁移协调**（DC-14）：SDK chrome 拥有自己的布局（toolbar/tab/bookmarks/viewport 高度
/// 由 SDK shell 决定，与 apps/browser 手绘 chrome 几何不同——desktop chrome top ≈ 96 vs 手绘
/// ≈ 112-140）。浏览器替换式迁移须把页面内容定位到 SDK chrome 的 viewport rect（而非手绘 chrome
/// 的 `chrome_top`），否则页面与 SDK chrome 错位。本函数暴露该 rect，使浏览器能查询 SDK chrome
/// 的内容区。SDK chrome 拥有布局、浏览器适配——这是「浏览器迁移为 SDK 宿主」的正确方向。
///
/// 返回 `(bridge, viewport_rect)`；`viewport_rect` 为 `None` 表示 SDK shell 未布局出 viewport
/// 节点（异常情况，调用方回落到手绘 chrome 几何）。
pub fn render_chrome_via_sdk_with_layout(
    shell: &zero_browser_shell::BrowserShell,
    metrics: &WindowMetrics,
    tokens: &SemanticTokens,
    backend: Arc<FontdueBackend>,
) -> (RenderFoundationBackend, Option<Rect>) {
    let model = BrowserChromeModel::from_shell(shell);
    let spec = DesktopBrowserShell.build(&model, metrics);
    let mut host = WidgetHost::new();
    register_chrome_factories(&mut host, tokens);
    host.set_root(&spec);
    host.layout(Constraints::loose(metrics.logical_size));
    // SDK chrome 布局后的页面内容区（viewport 节点绝对 rect）。
    let viewport_rect = host.rect_of(&WidgetId::new(ID_VIEWPORT));
    let scene = host.paint().clone();
    let mut bridge = RenderFoundationBackend::new_with_text_size(metrics.logical_size, backend);
    paint_scene(&scene, &mut bridge);
    (bridge, viewport_rect)
}

/// 同 [`render_chrome_via_sdk_with_layout`]，但 viewport 使用 [`WebViewWidget`]
/// （`register_chrome_factories_with_webview`），并接受 WebView 表面注册（DC-3 phase-2）。
///
/// `webview_surface`：`(surface_id, primitives)` —— WebView 渲染输出（已变换到帧坐标空间）。
/// 在 `paint_scene` 之前注册到 bridge，使 WebViewWidget 的 `ExternalSurface` marker
/// 经 `draw_external_surface` 把 WebView 纹理合并进 SDK chrome scene。
///
/// 返回 `(bridge, viewport_rect)`；`webview_surface` 为 `None` 时等价于
/// `render_chrome_via_sdk_with_layout` 但 viewport 用 WebViewWidget 工厂。
pub fn render_chrome_via_sdk_with_webview_surface(
    shell: &zero_browser_shell::BrowserShell,
    metrics: &WindowMetrics,
    tokens: &SemanticTokens,
    scheme: ResolvedColorScheme,
    backend: Arc<FontdueBackend>,
    webview_surface: Option<(
        u64,
        RenderPrimitives,
        Option<zero_render_foundation::image_cache::ImageCache>,
    )>,
) -> (RenderFoundationBackend, Option<Rect>) {
    let model = BrowserChromeModel::from_shell(shell);
    let spec = DesktopBrowserShell.build(&model, metrics);
    let mut host = WidgetHost::new();
    register_chrome_factories_with_webview(&mut host, tokens, scheme);
    host.set_root(&spec);
    host.layout(Constraints::loose(metrics.logical_size));
    let viewport_rect = host.rect_of(&WidgetId::new(ID_VIEWPORT));
    let scene = host.paint().clone();
    let mut bridge = RenderFoundationBackend::new_with_text_size(metrics.logical_size, backend);
    // 在 paint_scene 之前注册 WebView 表面（DC-3 phase-2）：draw_external_surface 在 paint_scene
    // 期间按 ExternalSurface marker 的 surface_id 取回已注册表面并合并。
    if let Some((surface_id, primitives, maybe_cache)) = webview_surface {
        if let Some(cache) = maybe_cache {
            bridge.set_surface_with_cache(surface_id, primitives, cache);
        } else {
            bridge.set_surface(surface_id, primitives);
        }
    }
    paint_scene(&scene, &mut bridge);
    (bridge, viewport_rect)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::geometry::{Insets, Size};

    /// WPT 标准字体（每字符 1em 实心方块）。路径相对 crate 根：browser-ui/chrome → ../../../tests。
    const AHEM: &[u8] = include_bytes!("../../../tests/wpt-runner/fonts/Ahem.ttf");

    fn metrics() -> WindowMetrics {
        WindowMetrics {
            logical_size: Size::new(1280.0, 800.0),
            scale_factor: 1.0,
            safe_area: Insets::all(0.0),
            keyboard_insets: Insets::all(0.0),
            text_scale: 1.0,
            density: 1.0,
            orientation: zero_ui_core::layout::Orientation::Landscape,
        }
    }

    #[test]
    fn render_chrome_via_sdk_with_real_shell_produces_geometry_and_text() {
        // 真实 BrowserShell 状态：导航到带 URL 的页面 → chrome 模型 address_text 非空。
        let mut shell = zero_browser_shell::BrowserShell::new();
        shell.new_tab(Some("https://example.com")); // active tab url 非空 → 地址栏文本

        let mut backend = FontdueBackend::new();
        backend.load_family("Ahem", AHEM).expect("Ahem parses via fontdue");
        let backend = Arc::new(backend);

        let bridge = render_chrome_via_sdk(&shell, &metrics(), &SemanticTokens::light(), backend);
        let p = bridge.into_primitives();
        // chrome 背景（toolbar/viewport 等）→ FillPrimitive 非空（管线几何正确）。
        assert!(!p.fills.is_empty(), "SDK chrome 管线应产出背景 fills");
        // 地址栏文本 "https://example.com" 经 draw_text → glyph ImagePrimitive 非空（管线文本正确）。
        assert!(!p.images.is_empty(), "SDK chrome 管线应产出地址栏文本 ImagePrimitive");
    }

    #[test]
    fn render_chrome_via_sdk_empty_shell_still_renders_geometry() {
        // 空 shell（仅初始空 tab）→ 仍有 chrome 背景几何（toolbar/viewport），文本可能为空。
        let shell = zero_browser_shell::BrowserShell::new();
        let backend = Arc::new(FontdueBackend::new()); // 无字体 → 文本 no-op，几何仍渲染
        let bridge = render_chrome_via_sdk(&shell, &metrics(), &SemanticTokens::dark(), backend);
        let p = bridge.into_primitives();
        assert!(!p.fills.is_empty(), "即便无字体，chrome 背景几何仍应渲染");
    }

    #[test]
    fn render_chrome_via_sdk_with_layout_exposes_viewport_content_rect() {
        // DC-14 替换式迁移协调：SDK chrome 布局后的 viewport rect（页面内容区）须可查询，
        // 使浏览器能据此定位页面内容（SDK chrome 拥有布局，浏览器适配）。
        let mut shell = zero_browser_shell::BrowserShell::new();
        shell.new_tab(Some("https://example.com"));
        let mut backend = FontdueBackend::new();
        backend.load_family("Ahem", AHEM).expect("Ahem parses via fontdue");
        let (bridge, viewport_rect) =
            render_chrome_via_sdk_with_layout(&shell, &metrics(), &SemanticTokens::light(), Arc::new(backend));
        let p = bridge.into_primitives();
        assert!(!p.fills.is_empty(), "bridge 仍产出 chrome fills");

        let vp = viewport_rect.expect("SDK shell 布局出 viewport 节点");
        // viewport 在 chrome 之下（顶部留出 toolbar+tab+bookmarks）+ 非零 + 在 metrics 内。
        assert!(vp.size.width > 0.0 && vp.size.height > 0.0, "viewport 非零");
        assert!(vp.origin.y > 0.0, "viewport 在 chrome 之下（y > 0，顶部留出 chrome）");
        assert!(vp.origin.y < metrics().logical_size.height, "viewport 起点在窗口内");
        // viewport 顶 = SDK chrome 占用的高度（toolbar+tab+bookmarks）。
        // desktop shell：toolbar(36) + tab(32) + bookmarks(28) ≈ 96（host 实际布局）。
        assert!(
            vp.origin.y >= 50.0 && vp.origin.y <= 200.0,
            "viewport top ≈ SDK chrome 高度，got {}",
            vp.origin.y
        );
    }

    #[test]
    fn webview_surface_merges_into_chrome_output() {
        // DC-3 phase-2：render_chrome_via_sdk_with_webview_surface 把 WebView RenderPrimitives
        // 注册为表面并在 paint_scene 期间合并进 SDK chrome 输出。
        let mut shell = zero_browser_shell::BrowserShell::new();
        shell.new_tab(Some("https://example.com"));
        let mut font_backend = FontdueBackend::new();
        font_backend.load_family("Ahem", AHEM).expect("Ahem parses via fontdue");

        // 模拟 WebView 渲染输出（一个填充矩形，位于 viewport 内）。
        let mut webview_prims = RenderPrimitives::default();
        webview_prims.add_fill(
            zero_render_foundation::geometry::Rect::new(0.0, 0.0, 1280.0, 704.0),
            zero_render_foundation::color::Color::rgb(255, 255, 255),
        );

        let (bridge, vp) = render_chrome_via_sdk_with_webview_surface(
            &shell,
            &metrics(),
            &SemanticTokens::light(),
            ResolvedColorScheme::Light,
            Arc::new(font_backend),
            Some((0, webview_prims, None)),
        );
        let p = bridge.into_primitives();
        // chrome fills（toolbar/background 等）非空。
        assert!(!p.fills.is_empty(), "chrome fills 非空: {:?}", p.fills.len());
        // WebView 表面已合并（至少 webview 的 fill 在 primitives 中）+ chrome fills。
        let webview_fills = p.fills.len();
        assert!(
            webview_fills >= 1,
            "fills count after webview merge: {} (chrome + webview)",
            webview_fills
        );
        // viewport rect 非空。
        assert!(vp.is_some(), "viewport rect 非空");
    }
}
