//! ZeroBrowser — 基于 Rust 的跨平台浏览器应用
//!
//! M11 里程碑：完整浏览器应用，连接 BrowserShell（数据模型）、
//! WebView（页面渲染）和 HostRuntime（窗口管理）。

use std::collections::HashMap;
use std::sync::Arc;

use zero_browser_shell::{BrowserShell, TabId};
use zero_host_runtime::event::AppEvent;
use zero_host_runtime::window::{HostRuntime, WindowConfig};
use zero_render_foundation::color::Color;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::gpu::renderer::{GlyphDraw, GpuRenderer};
use zero_render_foundation::primitive::FillPrimitive;
use zero_webview::WebViewBuilder;

/// 浏览器 UI 布局常量
mod layout {
    /// 标签栏高度
    pub const TAB_BAR_HEIGHT: f32 = 36.0;
    /// 地址栏高度
    pub const ADDRESS_BAR_HEIGHT: f32 = 36.0;
    /// 地址栏内边距
    pub const ADDRESS_BAR_PADDING: f32 = 8.0;
    /// 工具栏总高度
    pub const TOOLBAR_HEIGHT: f32 = TAB_BAR_HEIGHT + ADDRESS_BAR_HEIGHT;
    /// 导航按钮宽度
    pub const NAV_BUTTON_WIDTH: f32 = 32.0;
}

/// 浏览器 UI 颜色（使用 const 构造 Color）
mod colors {
    use super::Color;
    /// 窗口背景色（深灰）
    pub const BACKGROUND: Color = Color {
        r: 30,
        g: 30,
        b: 30,
        a: 255,
    };
    /// 标签栏背景色
    pub const TAB_BAR_BG: Color = Color {
        r: 40,
        g: 40,
        b: 40,
        a: 255,
    };
    /// 活跃标签背景色
    pub const TAB_ACTIVE_BG: Color = Color {
        r: 60,
        g: 60,
        b: 60,
        a: 255,
    };
    /// 标签文字颜色
    pub const TAB_TEXT: Color = Color {
        r: 200,
        g: 200,
        b: 200,
        a: 255,
    };
    /// 地址栏背景色
    pub const ADDRESS_BAR_BG: Color = Color {
        r: 50,
        g: 50,
        b: 50,
        a: 255,
    };
    /// 地址栏聚焦背景色
    pub const ADDRESS_BAR_BG_FOCUSED: Color = Color {
        r: 60,
        g: 60,
        b: 60,
        a: 255,
    };
    /// 地址栏文字颜色
    pub const ADDRESS_BAR_TEXT: Color = Color {
        r: 240,
        g: 240,
        b: 240,
        a: 255,
    };
    /// 地址栏占位文字颜色
    pub const ADDRESS_BAR_PLACEHOLDER: Color = Color {
        r: 160,
        g: 160,
        b: 160,
        a: 255,
    };
    /// 页面内容区域背景色（白色）
    pub const PAGE_BG: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    /// 导航按钮颜色
    pub const NAV_BUTTON: Color = Color {
        r: 180,
        g: 180,
        b: 180,
        a: 255,
    };
    /// 分隔线颜色
    pub const SEPARATOR: Color = Color {
        r: 70,
        g: 70,
        b: 70,
        a: 255,
    };
    /// 加载指示器颜色
    pub const LOADING_INDICATOR: Color = Color {
        r: 66,
        g: 133,
        b: 244,
        a: 255,
    };
    /// 页面标题颜色
    pub const PAGE_TITLE: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    /// 页面 URL 颜色
    pub const PAGE_URL: Color = Color {
        r: 100,
        g: 100,
        b: 100,
        a: 255,
    };
    /// 页面内容提示颜色
    pub const PAGE_HINT: Color = Color {
        r: 150,
        g: 150,
        b: 150,
        a: 255,
    };
    /// 页面正文颜色
    pub const PAGE_BODY: Color = Color {
        r: 80,
        g: 80,
        b: 80,
        a: 255,
    };
    /// 状态栏文字颜色
    pub const STATUS_TEXT: Color = Color {
        r: 120,
        g: 120,
        b: 120,
        a: 255,
    };
}

/// 浏览器应用状态
struct BrowserApp {
    /// 浏览器 Shell（标签页、书签、历史）
    shell: BrowserShell,
    /// 每个标签页对应的 WebView
    webviews: HashMap<TabId, zero_webview::WebView>,
    /// GPU 渲染器
    gpu_renderer: Option<GpuRenderer>,
    /// 字体加载器
    font_loader: FontLoader,
    /// Glyph 缓存
    glyph_cache: GlyphCache,
    /// 已加载的系统字体 ID
    font_id: Option<u32>,
    /// 是否已初始化 GPU 表面
    surface_configured: bool,
    /// 地址栏当前文本
    address_bar_text: String,
    /// 地址栏是否获得焦点
    address_bar_focused: bool,
    /// 窗口尺寸
    window_size: (u32, u32),
    /// 是否需要重绘
    needs_redraw: bool,
}

impl BrowserApp {
    /// 创建新的浏览器应用
    fn new() -> Self {
        let mut font_loader = FontLoader::new();
        let font_id = load_system_font(&mut font_loader);

        if font_id.is_some() {
            tracing::info!("System font loaded");
        } else {
            tracing::warn!("No system font found, text rendering will be limited");
        }

        Self {
            shell: BrowserShell::new(),
            webviews: HashMap::new(),
            gpu_renderer: None,
            font_loader,
            glyph_cache: GlyphCache::new(8192),
            font_id,
            surface_configured: false,
            address_bar_text: String::new(),
            address_bar_focused: false,
            window_size: (1024, 768),
            needs_redraw: true,
        }
    }

    /// 获取或创建活跃标签页的 WebView
    fn ensure_webview(&mut self, tab_id: TabId) {
        self.webviews
            .entry(tab_id)
            .or_insert_with(|| WebViewBuilder::new().build());
    }

    /// 导航到指定 URL
    fn navigate_to(&mut self, url: &str) {
        let url = normalize_url(url);
        tracing::info!("Navigating to: {url}");

        self.shell.navigate(&url);
        self.address_bar_text = url.clone();

        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return,
        };
        self.ensure_webview(tab_id);

        // 尝试通过 WebView 加载页面
        let result = match self.webviews.get_mut(&tab_id) {
            Some(wv) => wv.fetch_url(&url),
            None => return,
        };

        if let Err(e) = result {
            tracing::warn!("Failed to fetch URL: {e}, loading error page");
            let error_page = generate_error_page(&url, &e.to_string());
            if let Some(wv) = self.webviews.get_mut(&tab_id) {
                wv.load_html(&error_page, None);
            }
        }

        self.needs_redraw = true;
    }

    /// 创建新标签页
    fn new_tab(&mut self, url: Option<&str>) {
        let tab_id = self.shell.new_tab(url);
        let webview = WebViewBuilder::new().build();
        self.webviews.insert(tab_id, webview);

        if let Some(url) = url {
            self.address_bar_text = url.to_string();
        } else {
            self.address_bar_text.clear();
            // 加载欢迎页
            if let Some(wv) = self.webviews.get_mut(&tab_id) {
                wv.load_html(WELCOME_HTML, None);
            }
        }

        self.needs_redraw = true;
    }

    /// 关闭活跃标签页
    fn close_active_tab(&mut self) {
        if let Some(tab_id) = self.shell.active_tab_id() {
            self.webviews.remove(&tab_id);
            self.shell.close_tab(tab_id);

            if self.shell.is_empty() {
                self.new_tab(None);
            }

            self.update_address_bar_from_active_tab();
            self.needs_redraw = true;
        }
    }

    /// 刷新当前页面
    fn refresh_page(&mut self) {
        self.shell.refresh();

        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return,
        };

        let url = match self
            .webviews
            .get(&tab_id)
            .and_then(|wv| wv.url().map(|s| s.to_string()))
        {
            Some(u) => u,
            None => return,
        };

        if let Some(wv) = self.webviews.get_mut(&tab_id) {
            let _ = wv.fetch_url(&url);
        }

        self.needs_redraw = true;
    }

    /// 执行后退导航
    fn go_back(&mut self) {
        if !self.shell.go_back() {
            return;
        }

        let url = match self.shell.active_tab().and_then(|t| t.url().map(|s| s.to_string())) {
            Some(u) => u,
            None => return,
        };

        self.address_bar_text = url.clone();
        let tab_id = self.shell.active_tab_id().unwrap();
        self.ensure_webview(tab_id);

        if let Some(wv) = self.webviews.get_mut(&tab_id) {
            let _ = wv.fetch_url(&url);
        }

        self.needs_redraw = true;
    }

    /// 执行前进导航
    fn go_forward(&mut self) {
        if !self.shell.go_forward() {
            return;
        }

        let url = match self.shell.active_tab().and_then(|t| t.url().map(|s| s.to_string())) {
            Some(u) => u,
            None => return,
        };

        self.address_bar_text = url.clone();
        let tab_id = self.shell.active_tab_id().unwrap();
        self.ensure_webview(tab_id);

        if let Some(wv) = self.webviews.get_mut(&tab_id) {
            let _ = wv.fetch_url(&url);
        }

        self.needs_redraw = true;
    }

    /// 处理键盘输入
    fn handle_key(&mut self, key: &str, _pressed: bool) {
        if self.address_bar_focused {
            match key {
                "Enter" => {
                    let url = self.address_bar_text.trim().to_string();
                    if !url.is_empty() {
                        self.navigate_to(&url);
                    }
                    self.address_bar_focused = false;
                }
                "Escape" => {
                    self.address_bar_focused = false;
                    self.update_address_bar_from_active_tab();
                }
                "Backspace" => {
                    self.address_bar_text.pop();
                    self.needs_redraw = true;
                }
                _ => {
                    // 单字符输入
                    if key.len() == 1 {
                        self.address_bar_text.push_str(key);
                        self.needs_redraw = true;
                    }
                }
            }
        } else {
            // 全局快捷键（暂不检查 Ctrl 修饰键，因为 host-runtime 尚未传递修饰键）
            match key {
                "l" => {
                    self.address_bar_focused = true;
                    self.needs_redraw = true;
                }
                "t" => {
                    self.new_tab(None);
                }
                "w" => {
                    self.close_active_tab();
                }
                "r" => {
                    self.refresh_page();
                }
                "Left" => {
                    self.go_back();
                }
                "Right" => {
                    self.go_forward();
                }
                "Home" => {
                    self.navigate_to("https://example.com");
                }
                _ => {}
            }
        }
    }

    /// 初始化 GPU 渲染器
    fn init_gpu(&mut self, window: &Arc<winit::window::Window>) {
        match GpuRenderer::new_for_window(Arc::clone(window)) {
            Ok(renderer) => {
                tracing::info!("GPU renderer initialized (format: {:?})", renderer.surface_format());
                self.gpu_renderer = Some(renderer);
                self.surface_configured = false;
                self.needs_redraw = true;
            }
            Err(e) => {
                tracing::error!("GPU renderer init failed: {e}");
            }
        }
    }

    /// 执行渲染
    fn render(&mut self, width: u32, height: u32) {
        let mut gpu = self.gpu_renderer.take();
        if let Some(ref mut renderer) = gpu {
            self.render_frame(renderer, width, height);
        }
        self.gpu_renderer = gpu;
    }

    /// 渲染一帧
    fn render_frame(&mut self, gpu: &mut GpuRenderer, width: u32, height: u32) {
        let mut fills = Vec::new();
        let mut glyphs = Vec::new();
        let font_size = 14.0_f32;

        // 1. 整体背景
        fills.push(rect_fill(0.0, 0.0, width as f32, height as f32, colors::BACKGROUND));

        // 2. 标签栏背景
        fills.push(rect_fill(
            0.0,
            0.0,
            width as f32,
            layout::TAB_BAR_HEIGHT,
            colors::TAB_BAR_BG,
        ));

        // 3. 标签内容
        self.render_tabs(&mut fills, &mut glyphs, width, font_size);

        // 4. 地址栏背景
        let addr_y = layout::TAB_BAR_HEIGHT;
        fills.push(rect_fill(
            0.0,
            addr_y,
            width as f32,
            layout::ADDRESS_BAR_HEIGHT,
            colors::TAB_BAR_BG,
        ));

        // 5. 导航按钮
        self.render_nav_buttons(&mut glyphs, addr_y, font_size);

        // 6. 地址栏
        self.render_address_bar(&mut fills, &mut glyphs, width, addr_y, font_size);

        // 7. 分隔线
        fills.push(rect_fill(
            0.0,
            layout::TOOLBAR_HEIGHT - 1.0,
            width as f32,
            1.0,
            colors::SEPARATOR,
        ));

        // 8. 页面内容区域
        let page_y = layout::TOOLBAR_HEIGHT;
        let page_h = height as f32 - page_y;
        fills.push(rect_fill(0.0, page_y, width as f32, page_h, colors::PAGE_BG));

        // 9. 加载指示器
        if self.shell.active_tab().is_some_and(|t| t.is_loading()) {
            fills.push(rect_fill(
                0.0,
                layout::TOOLBAR_HEIGHT,
                width as f32,
                2.0,
                colors::LOADING_INDICATOR,
            ));
        }

        // 10. 页面内容
        self.render_page_content(&mut glyphs, width, page_y, font_size);

        gpu.render_scene(&fills, &self.font_loader, &mut self.glyph_cache, &glyphs);
    }

    /// 渲染标签页
    fn render_tabs(&mut self, fills: &mut Vec<FillPrimitive>, glyphs: &mut Vec<GlyphDraw>, width: u32, font_size: f32) {
        let active_id = self.shell.active_tab_id();
        let tab_count = self.shell.tab_count();
        if tab_count == 0 {
            return;
        }

        // 活跃标签
        if let Some(tab) = self.shell.active_tab() {
            let is_active = Some(tab.id()) == active_id;
            let tab_w = (width as f32 / tab_count as f32).min(240.0);

            fills.push(rect_fill(
                0.0,
                0.0,
                tab_w - 1.0,
                layout::TAB_BAR_HEIGHT,
                if is_active {
                    colors::TAB_ACTIVE_BG
                } else {
                    colors::TAB_BAR_BG
                },
            ));

            let label = tab.url().unwrap_or("New Tab");
            let display = strip_protocol(label);
            let truncated: String = display.chars().take(20).collect();

            if let Some(fid) = self.font_id {
                draw_text(&truncated, 12.0, 10.0, font_size, colors::TAB_TEXT, fid, glyphs);
            }
        }

        // 多标签提示
        if tab_count > 1
            && let Some(fid) = self.font_id
        {
            let x = 244.0_f32;
            let text = format!("+{} tabs", tab_count - 1);
            draw_text(&text, x, 10.0, font_size, colors::TAB_TEXT, fid, glyphs);
        }
    }

    /// 渲染导航按钮
    fn render_nav_buttons(&mut self, glyphs: &mut Vec<GlyphDraw>, y: f32, font_size: f32) {
        if let Some(fid) = self.font_id {
            let baseline_y = y + (layout::ADDRESS_BAR_HEIGHT + font_size) / 2.0;
            let x = 8.0_f32;
            let w = layout::NAV_BUTTON_WIDTH;

            for (i, ch) in ['←', '→', '↻', '⌂'].iter().enumerate() {
                glyphs.push(GlyphDraw {
                    ch: *ch,
                    x: x + w * i as f32,
                    baseline_y,
                    color: colors::NAV_BUTTON,
                    font_id: fid,
                    font_size,
                });
            }
        }
    }

    /// 渲染地址栏
    fn render_address_bar(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        y: f32,
        font_size: f32,
    ) {
        let nav_w = layout::NAV_BUTTON_WIDTH * 4.0 + 16.0;
        let bar_x = nav_w + layout::ADDRESS_BAR_PADDING;
        let bar_w = width as f32 - bar_x - layout::ADDRESS_BAR_PADDING;
        let bar_y = y + 4.0;
        let bar_h = layout::ADDRESS_BAR_HEIGHT - 8.0;

        let bg = if self.address_bar_focused {
            colors::ADDRESS_BAR_BG_FOCUSED
        } else {
            colors::ADDRESS_BAR_BG
        };
        fills.push(rect_fill(bar_x, bar_y, bar_w, bar_h, bg));

        let display_text = if self.address_bar_text.is_empty() && !self.address_bar_focused {
            "Search or enter URL...".to_string()
        } else {
            self.address_bar_text.clone()
        };

        if let Some(fid) = self.font_id {
            let color = if self.address_bar_focused {
                colors::ADDRESS_BAR_TEXT
            } else if self.address_bar_text.is_empty() {
                colors::ADDRESS_BAR_PLACEHOLDER
            } else {
                colors::ADDRESS_BAR_TEXT
            };
            draw_text(&display_text, bar_x + 10.0, bar_y + 3.0, font_size, color, fid, glyphs);
        }
    }

    /// 渲染页面内容
    fn render_page_content(&mut self, glyphs: &mut Vec<GlyphDraw>, width: u32, page_y: f32, font_size: f32) {
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

        // 收集活跃标签信息
        let (title, url, is_loading) = match self.shell.active_tab() {
            Some(tab) => (
                tab.title().unwrap_or("").to_string(),
                tab.url().unwrap_or("").to_string(),
                tab.is_loading(),
            ),
            None => return,
        };

        if !title.is_empty() {
            draw_text(&title, 20.0, page_y + 20.0, 24.0, colors::PAGE_TITLE, fid, glyphs);
        }

        if !url.is_empty() {
            draw_text(&url, 20.0, page_y + 52.0, 12.0, colors::PAGE_URL, fid, glyphs);
        }

        let content_y = page_y + 80.0;

        if is_loading {
            draw_text("Loading...", 20.0, content_y, font_size, colors::PAGE_HINT, fid, glyphs);
        } else if title.is_empty() && url.is_empty() {
            draw_text(
                "Welcome to ZeroBrowser — Press L to focus address bar, T for new tab",
                20.0,
                content_y,
                font_size,
                colors::PAGE_HINT,
                fid,
                glyphs,
            );
        } else {
            // 尝试显示 WebView 的 URL 信息
            let tab_id = match self.shell.active_tab_id() {
                Some(id) => id,
                None => return,
            };
            if let Some(wv) = self.webviews.get(&tab_id) {
                let info = format!("Content from: {}", wv.url().unwrap_or("(none)"));
                draw_text(&info, 20.0, content_y, font_size, colors::PAGE_BODY, fid, glyphs);
            }
        }

        // 底部状态栏
        let status_y = self.window_size.1 as f32 - 24.0;
        let status = format!("Tabs: {}", self.shell.tab_count());
        draw_text(
            &status,
            width as f32 - 100.0,
            status_y,
            11.0,
            colors::STATUS_TEXT,
            fid,
            glyphs,
        );
    }

    /// 从活跃标签更新地址栏文本
    fn update_address_bar_from_active_tab(&mut self) {
        if let Some(tab) = self.shell.active_tab() {
            self.address_bar_text = tab.url().unwrap_or("").to_string();
        }
    }
}

/// 创建填充矩形图元
fn rect_fill(x: f32, y: f32, w: f32, h: f32, color: Color) -> FillPrimitive {
    FillPrimitive {
        rect: zero_render_foundation::geometry::Rect::new(x, y, w, h),
        color,
    }
}

/// 绘制文本（估算字符宽度）
fn draw_text(
    text: &str,
    start_x: f32,
    start_y: f32,
    font_size: f32,
    color: Color,
    font_id: u32,
    glyphs: &mut Vec<GlyphDraw>,
) {
    let mut x = start_x;
    for ch in text.chars() {
        glyphs.push(GlyphDraw {
            ch,
            x,
            baseline_y: start_y + font_size,
            color,
            font_id,
            font_size,
        });
        x += if ch.is_ascii() { font_size * 0.6 } else { font_size };
    }
}

/// URL 规范化
fn normalize_url(input: &str) -> String {
    if input.starts_with("http://") || input.starts_with("https://") {
        input.to_string()
    } else if input.contains('.') && !input.contains(' ') {
        format!("https://{input}")
    } else {
        input.to_string()
    }
}

/// 去除 URL 协议前缀
fn strip_protocol(url: &str) -> &str {
    if let Some(rest) = url.strip_prefix("https://") {
        rest
    } else if let Some(rest) = url.strip_prefix("http://") {
        rest
    } else {
        url
    }
}

/// 欢迎页 HTML
const WELCOME_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><title>ZeroBrowser</title></head>
<body style="font-family: sans-serif; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background: #f8f9fa;">
  <div style="text-align: center;">
    <h1 style="color: #333; font-size: 48px;">ZeroBrowser</h1>
    <p style="color: #666; font-size: 18px;">基于 Rust 的跨平台浏览器</p>
    <p style="color: #999; font-size: 14px;">在地址栏输入 URL 开始浏览</p>
  </div>
</body>
</html>"#;

/// 生成错误页面 HTML
fn generate_error_page(url: &str, error: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>加载失败</title></head>
<body style="font-family: sans-serif; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background: #fff3f3;">
  <div style="text-align: center;">
    <h1 style="color: #c62828;">页面加载失败</h1>
    <p style="color: #555;">无法加载: <code>{url}</code></p>
    <p style="color: #888; font-size: 14px;">错误: {error}</p>
  </div>
</body>
</html>"#
    )
}

/// 尝试加载系统字体
fn load_system_font(font_loader: &mut FontLoader) -> Option<u32> {
    let font_paths = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "C:\\Windows\\Fonts\\arial.ttf",
    ];

    font_paths.iter().find_map(|path| {
        std::fs::read(path)
            .ok()
            .and_then(|data| font_loader.load_font(&data).ok())
    })
}

fn main() {
    // 初始化日志
    tracing_subscriber::fmt().init();

    tracing::info!("ZeroBrowser starting...");

    let config = WindowConfig::new("ZeroBrowser")
        .with_size(1024, 768)
        .with_resizable(true);

    let runtime = HostRuntime::new(config);
    let mut app = BrowserApp::new();

    // 加载欢迎页
    app.new_tab(None);

    tracing::info!("Entering event loop...");

    if let Err(e) = runtime.run_with_window(move |event, window| {
        match event {
            AppEvent::RedrawRequested => {
                if !app.surface_configured {
                    if let Some(ref win) = window
                        && app.gpu_renderer.is_none()
                    {
                        app.init_gpu(win);
                    }
                    if let Some(ref mut gpu) = app.gpu_renderer {
                        let (w, h) = app.window_size;
                        gpu.configure_surface(w, h);
                        app.surface_configured = true;
                    }
                }
                app.render(app.window_size.0, app.window_size.1);
                app.needs_redraw = false;
            }
            AppEvent::Resized { width, height } if width > 0 && height > 0 => {
                tracing::debug!("Window resized: {width}x{height}");
                app.window_size = (width, height);
                if let Some(ref mut gpu) = app.gpu_renderer {
                    gpu.configure_surface(width, height);
                }
                app.needs_redraw = true;
            }
            AppEvent::CloseRequested => {
                tracing::info!("Window closed");
            }
            AppEvent::KeyboardInput { key, pressed } if pressed => {
                app.handle_key(&key, true);
            }
            AppEvent::MouseInput { button: _, pressed: _ } => {
                // 未来：处理鼠标点击（地址栏聚焦、标签切换等）
            }
            AppEvent::Focused => {
                tracing::debug!("Window focused");
            }
            AppEvent::Unfocused => {
                app.address_bar_focused = false;
            }
            _ => {}
        }
    }) {
        tracing::error!("Event loop error: {e}");
        std::process::exit(1);
    }

    tracing::info!("ZeroBrowser exited");
}
