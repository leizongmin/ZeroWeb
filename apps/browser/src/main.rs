//! ZeroBrowser — 基于 Rust 的跨平台浏览器应用
//!
//! M11 里程碑：完整浏览器应用，连接 BrowserShell（数据模型）、
//! WebView（页面渲染）和 HostRuntime（窗口管理）。

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;

use softbuffer::{Context as SoftbufferContext, Surface as SoftbufferSurface};
use zero_browser_shell::{BrowserShell, SuggestionSource, TabId};
use zero_host_runtime::event::AppEvent;
use zero_host_runtime::window::{HostRuntime, WindowConfig};
use zero_render_foundation::color::Color;
use zero_render_foundation::config::RenderMode;
use zero_render_foundation::cpu::render_scene_to_framebuffer;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::gpu::renderer::{GlyphDraw, GpuRenderer};
use zero_render_foundation::primitive::FillPrimitive;
use zero_webview::WebViewBuilder;

type CpuSurface = SoftbufferSurface<Arc<winit::window::Window>, Arc<winit::window::Window>>;

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
    /// 单个标签最小宽度
    pub const TAB_MIN_WIDTH: f32 = 100.0;
    /// 单个标签最大宽度
    pub const TAB_MAX_WIDTH: f32 = 240.0;
    /// 标签关闭按钮大小
    pub const TAB_CLOSE_SIZE: f32 = 16.0;
    /// 自动补全下拉最大显示条数
    pub const AUTOCOMPLETE_MAX_VISIBLE: usize = 6;
    /// 自动补全下拉行高
    pub const AUTOCOMPLETE_ROW_HEIGHT: f32 = 28.0;
    /// 查找栏高度
    pub const FIND_BAR_HEIGHT: f32 = 36.0;
    /// 状态栏高度
    pub const STATUS_BAR_HEIGHT: f32 = 22.0;
}

/// 浏览器 UI 颜色
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
    /// 非活跃标签悬停背景色
    pub const TAB_HOVER_BG: Color = Color {
        r: 50,
        g: 50,
        b: 50,
        a: 255,
    };
    /// 标签文字颜色
    pub const TAB_TEXT: Color = Color {
        r: 200,
        g: 200,
        b: 200,
        a: 255,
    };
    /// 标签关闭按钮颜色
    pub const TAB_CLOSE: Color = Color {
        r: 150,
        g: 150,
        b: 150,
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
    /// 自动补全下拉背景色
    pub const AUTOCOMPLETE_BG: Color = Color {
        r: 45,
        g: 45,
        b: 45,
        a: 255,
    };
    /// 自动补全悬停背景色
    pub const AUTOCOMPLETE_HOVER_BG: Color = Color {
        r: 60,
        g: 60,
        b: 60,
        a: 255,
    };
    /// 自动补全文字颜色
    pub const AUTOCOMPLETE_TEXT: Color = Color {
        r: 220,
        g: 220,
        b: 220,
        a: 255,
    };
    /// 自动补全 URL 颜色
    pub const AUTOCOMPLETE_URL: Color = Color {
        r: 140,
        g: 140,
        b: 140,
        a: 255,
    };
    /// 自动补全书签标记颜色
    pub const AUTOCOMPLETE_BOOKMARK: Color = Color {
        r: 255,
        g: 193,
        b: 7,
        a: 255,
    };
    /// 查找栏背景色
    pub const FIND_BAR_BG: Color = Color {
        r: 50,
        g: 50,
        b: 50,
        a: 245,
    };
    /// 查找栏文字颜色
    pub const FIND_BAR_TEXT: Color = Color {
        r: 220,
        g: 220,
        b: 220,
        a: 255,
    };
    /// 查找栏匹配数颜色
    pub const FIND_MATCH_TEXT: Color = Color {
        r: 160,
        g: 160,
        b: 160,
        a: 255,
    };
    /// 新建标签按钮颜色
    pub const NEW_TAB_BUTTON: Color = Color {
        r: 160,
        g: 160,
        b: 160,
        a: 255,
    };
}

/// 自动补全建议缓存
struct AutocompleteState {
    /// 当前显示的建议列表
    suggestions: Vec<zero_browser_shell::Suggestion>,
    /// 鼠标悬停的索引
    hovered_index: Option<usize>,
}

impl AutocompleteState {
    fn new() -> Self {
        Self {
            suggestions: Vec::new(),
            hovered_index: None,
        }
    }

    fn clear(&mut self) {
        self.suggestions.clear();
        self.hovered_index = None;
    }
}

/// 浏览器应用状态
struct BrowserApp {
    /// 浏览器 Shell（标签页、书签、历史）
    shell: BrowserShell,
    /// 每个标签页对应的 WebView
    webviews: HashMap<TabId, zero_webview::WebView>,
    /// GPU 渲染器
    gpu_renderer: Option<GpuRenderer>,
    /// CPU 软件渲染窗口 surface
    cpu_surface: Option<CpuSurface>,
    /// 渲染模式
    render_mode: RenderMode,
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
    /// 窗口物理像素尺寸
    physical_size: (u32, u32),
    /// 窗口缩放因子
    scale_factor: f32,
    /// 是否需要重绘
    needs_redraw: bool,
    /// 鼠标位置（用于悬停检测）
    mouse_pos: (f64, f64),
    /// 自动补全状态
    autocomplete: AutocompleteState,
    /// 查找栏输入文本
    find_input: String,
    /// 标签页布局缓存：每个标签页的 (x, width) 位置信息
    tab_layout: Vec<(TabId, f32, f32)>,
}

impl BrowserApp {
    /// 创建新的浏览器应用
    fn new(render_mode: RenderMode) -> Self {
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
            cpu_surface: None,
            render_mode,
            font_loader,
            glyph_cache: GlyphCache::new(8192),
            font_id,
            surface_configured: false,
            address_bar_text: String::new(),
            address_bar_focused: false,
            window_size: (1024, 768),
            physical_size: (1024, 768),
            scale_factor: 1.0,
            needs_redraw: true,
            mouse_pos: (0.0, 0.0),
            autocomplete: AutocompleteState::new(),
            find_input: String::new(),
            tab_layout: Vec::new(),
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
        let url = normalize_url(url, &self.shell);
        tracing::info!("Navigating to: {url}");

        self.shell.navigate(&url);
        self.address_bar_text = url.clone();
        self.autocomplete.clear();

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

    /// 关闭指定 ID 的标签页
    fn close_tab_by_id(&mut self, id: TabId) {
        self.webviews.remove(&id);
        self.shell.close_tab(id);

        if self.shell.is_empty() {
            self.new_tab(None);
        }

        self.update_address_bar_from_active_tab();
        self.needs_redraw = true;
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
        if self.shell.find_state().is_active() {
            // 查找栏获得焦点
            match key {
                "Enter" => {
                    if self.find_input.is_empty() {
                        self.shell.find_close();
                    } else if self.shell.find_state().total_matches() == 0 {
                        self.shell.find_start(&self.find_input.clone());
                    } else {
                        self.shell.find_next();
                    }
                    self.needs_redraw = true;
                }
                "Escape" => {
                    self.shell.find_close();
                    self.find_input.clear();
                    self.needs_redraw = true;
                }
                "Backspace" => {
                    self.find_input.pop();
                    if self.find_input.is_empty() {
                        self.shell.find_close();
                    } else {
                        self.shell.find_start(&self.find_input);
                    }
                    self.needs_redraw = true;
                }
                _ => {
                    if key.len() == 1 {
                        self.find_input.push_str(key);
                        self.shell.find_start(&self.find_input);
                        self.needs_redraw = true;
                    }
                }
            }
        } else if self.address_bar_focused {
            match key {
                "Enter" => {
                    let url = self.address_bar_text.trim().to_string();
                    if !url.is_empty() {
                        // 如果有高亮的自动补全建议，使用建议的 URL
                        let nav_url = if let Some(idx) = self.autocomplete.hovered_index {
                            self.autocomplete
                                .suggestions
                                .get(idx)
                                .map(|s| s.url().to_string())
                                .unwrap_or(url)
                        } else {
                            url
                        };
                        self.navigate_to(&nav_url);
                    }
                    self.address_bar_focused = false;
                    self.autocomplete.clear();
                }
                "Escape" => {
                    self.address_bar_focused = false;
                    self.autocomplete.clear();
                    self.update_address_bar_from_active_tab();
                }
                "Backspace" => {
                    self.address_bar_text.pop();
                    self.update_autocomplete();
                    self.needs_redraw = true;
                }
                "Down" => {
                    // 选择下一个自动补全建议
                    if !self.autocomplete.suggestions.is_empty() {
                        let next = self
                            .autocomplete
                            .hovered_index
                            .map(|i| (i + 1).min(self.autocomplete.suggestions.len() - 1))
                            .unwrap_or(0);
                        self.autocomplete.hovered_index = Some(next);
                        self.needs_redraw = true;
                    }
                }
                "Up" => {
                    // 选择上一个自动补全建议
                    if let Some(i) = self.autocomplete.hovered_index {
                        if i > 0 {
                            self.autocomplete.hovered_index = Some(i - 1);
                        } else {
                            self.autocomplete.hovered_index = None;
                        }
                        self.needs_redraw = true;
                    }
                }
                "Tab" => {
                    // Tab 键补全第一个建议
                    if let Some(sug) = self.autocomplete.suggestions.first() {
                        self.address_bar_text = sug.url().to_string();
                        self.autocomplete.clear();
                        self.needs_redraw = true;
                    }
                }
                _ => {
                    // 单字符输入
                    if key.len() == 1 {
                        self.address_bar_text.push_str(key);
                        self.update_autocomplete();
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
                "f" => {
                    // Ctrl+F: 打开查找栏
                    self.find_input.clear();
                    self.shell.find_close();
                    self.needs_redraw = true;
                }
                "+" | "=" => {
                    self.shell.zoom_in();
                    self.needs_redraw = true;
                }
                "-" => {
                    self.shell.zoom_out();
                    self.needs_redraw = true;
                }
                "0" => {
                    self.shell.zoom_reset();
                    self.needs_redraw = true;
                }
                "n" => {
                    // 查找下一个
                    self.shell.find_next();
                    self.find_input = self.shell.find_state().query().to_string();
                    self.needs_redraw = true;
                }
                _ => {}
            }
        }
    }

    /// 更新自动补全建议
    fn update_autocomplete(&mut self) {
        let query = self.address_bar_text.trim();
        if query.is_empty() {
            self.autocomplete.clear();
            return;
        }
        self.autocomplete.suggestions = self.shell.suggest(query);
        self.autocomplete.hovered_index = None;
    }

    /// 处理鼠标移动
    fn handle_mouse_move(&mut self, x: f64, y: f64) {
        let old_pos = self.mouse_pos;
        self.mouse_pos = (x, y);

        // 检查自动补全悬停
        if self.address_bar_focused && !self.autocomplete.suggestions.is_empty() {
            let hovered = self.autocomplete_hit_test(x, y);
            if hovered != self.autocomplete.hovered_index {
                self.autocomplete.hovered_index = hovered;
                self.needs_redraw = true;
            }
        }

        // 鼠标移动才重绘（优化：只在有悬停变化时重绘）
        if (old_pos.0 - x).abs() > 1.0 || (old_pos.1 - y).abs() > 1.0 {
            // 悬停效果需要重绘
            if (y as f32) < layout::TOOLBAR_HEIGHT {
                self.needs_redraw = true;
            }
        }
    }

    /// 处理鼠标点击
    fn handle_mouse_click(&mut self, x: f64, y: f64, pressed: bool) {
        if !pressed {
            return;
        }

        let y_f = y as f32;
        let x_f = x as f32;
        let width = self.window_size.0 as f32;

        // 1. 自动补全下拉区域点击
        if self.address_bar_focused && !self.autocomplete.suggestions.is_empty() {
            if let Some(idx) = self.autocomplete_hit_test(x, y) {
                let url = self.autocomplete.suggestions.get(idx).map(|s| s.url().to_string());
                if let Some(url) = url {
                    self.navigate_to(&url);
                    self.address_bar_focused = false;
                    self.autocomplete.clear();
                    return;
                }
            }
            // 点击自动补全区域外时关闭下拉
            let addr_bar_bottom = layout::TAB_BAR_HEIGHT + layout::ADDRESS_BAR_HEIGHT;
            let autocomplete_top = addr_bar_bottom;
            let autocomplete_height = self
                .autocomplete
                .suggestions
                .len()
                .min(layout::AUTOCOMPLETE_MAX_VISIBLE) as f32
                * layout::AUTOCOMPLETE_ROW_HEIGHT;
            if y_f >= autocomplete_top && y_f < autocomplete_top + autocomplete_height {
                return; // 点击在自动补全内但没命中建议，忽略
            }
            // 点击自动补全外，关闭它
            self.autocomplete.clear();
        }

        // 2. 标签栏区域点击
        if y_f < layout::TAB_BAR_HEIGHT {
            // 检查是否点击了新建标签按钮 (+)
            let new_tab_x = width - 32.0;
            if x_f >= new_tab_x && x_f <= width {
                self.new_tab(None);
                return;
            }

            // 检查是否点击了标签页
            for &(id, tab_x, tab_w) in &self.tab_layout {
                if x_f >= tab_x && x_f < tab_x + tab_w {
                    // 检查关闭按钮
                    let close_x = tab_x + tab_w - 24.0;
                    let close_y_center = layout::TAB_BAR_HEIGHT / 2.0;
                    if x_f >= close_x
                        && x_f <= close_x + layout::TAB_CLOSE_SIZE
                        && (y_f - close_y_center).abs() <= layout::TAB_CLOSE_SIZE / 2.0
                    {
                        self.close_tab_by_id(id);
                        return;
                    }
                    // 切换标签页
                    if Some(id) != self.shell.active_tab_id() {
                        self.shell.switch_tab(id);
                        self.update_address_bar_from_active_tab();
                        self.needs_redraw = true;
                    }
                    return;
                }
            }
            return;
        }

        // 3. 地址栏区域点击
        if y_f < layout::TAB_BAR_HEIGHT + layout::ADDRESS_BAR_HEIGHT {
            let nav_w = layout::NAV_BUTTON_WIDTH * 4.0 + 16.0;
            let addr_bar_x = nav_w + layout::ADDRESS_BAR_PADDING;

            // 导航按钮区域
            if x_f < nav_w {
                let button_index = ((x_f - 8.0) / layout::NAV_BUTTON_WIDTH) as i32;
                match button_index {
                    0 => self.go_back(),
                    1 => self.go_forward(),
                    2 => self.refresh_page(),
                    3 => {
                        let home = self.shell.settings().home_url.clone();
                        self.navigate_to(&home);
                    }
                    _ => {}
                }
                return;
            }

            // 地址栏输入区域
            if x_f >= addr_bar_x && x_f <= width - layout::ADDRESS_BAR_PADDING {
                if !self.address_bar_focused {
                    self.address_bar_focused = true;
                    self.needs_redraw = true;
                }
                return;
            }
        }

        // 4. 查找栏区域点击（如果活跃）
        if self.shell.find_state().is_active() {
            let find_y = layout::TOOLBAR_HEIGHT;
            if y_f >= find_y && y_f < find_y + layout::FIND_BAR_HEIGHT {
                // 点击关闭按钮
                let close_x = width - 40.0;
                if x_f >= close_x {
                    self.shell.find_close();
                    self.find_input.clear();
                    self.needs_redraw = true;
                    return;
                }
                // 点击上一个/下一个
                let prev_x = width - 100.0;
                let next_x = width - 70.0;
                if x_f >= prev_x && x_f < prev_x + 28.0 {
                    self.shell.find_previous();
                    self.needs_redraw = true;
                    return;
                }
                if x_f >= next_x && x_f < next_x + 28.0 {
                    self.shell.find_next();
                    self.needs_redraw = true;
                    return;
                }
                return;
            }
        }

        // 5. 页面内容区域 — 取消地址栏焦点
        if y_f >= layout::TOOLBAR_HEIGHT && self.address_bar_focused {
            self.address_bar_focused = false;
            self.autocomplete.clear();
            self.needs_redraw = true;
        }
    }

    /// 自动补全下拉命中检测
    fn autocomplete_hit_test(&self, x: f64, y: f64) -> Option<usize> {
        let nav_w = layout::NAV_BUTTON_WIDTH * 4.0 + 16.0;
        let bar_x = nav_w + layout::ADDRESS_BAR_PADDING;
        let bar_w = self.window_size.0 as f32 - bar_x - layout::ADDRESS_BAR_PADDING;

        let autocomplete_top = layout::TAB_BAR_HEIGHT + layout::ADDRESS_BAR_HEIGHT;
        let y_f = y as f32;
        let x_f = x as f32;

        if x_f < bar_x || x_f > bar_x + bar_w || y_f < autocomplete_top {
            return None;
        }

        let row_offset = y_f - autocomplete_top;
        if row_offset < 0.0 {
            return None;
        }

        let index = (row_offset / layout::AUTOCOMPLETE_ROW_HEIGHT) as usize;
        if index
            < self
                .autocomplete
                .suggestions
                .len()
                .min(layout::AUTOCOMPLETE_MAX_VISIBLE)
        {
            Some(index)
        } else {
            None
        }
    }

    /// 初始化 GPU 渲染器
    fn init_gpu(&mut self, window: &Arc<winit::window::Window>) {
        if matches!(self.render_mode, RenderMode::Cpu) {
            return;
        }

        match GpuRenderer::new_for_window(Arc::clone(window)) {
            Ok(renderer) => {
                tracing::info!("GPU renderer initialized (format: {:?})", renderer.surface_format());
                self.gpu_renderer = Some(renderer);
                self.surface_configured = false;
                self.needs_redraw = true;
            }
            Err(e) => {
                if matches!(self.render_mode, RenderMode::Gpu) {
                    tracing::error!("GPU renderer init failed: {e}");
                } else {
                    tracing::warn!("GPU renderer init failed: {e}; using CPU renderer");
                }
            }
        }
    }

    /// 初始化 CPU 软件渲染 surface
    fn init_cpu_surface(&mut self, window: &Arc<winit::window::Window>) {
        if self.cpu_surface.is_some() {
            return;
        }

        match SoftbufferContext::new(Arc::clone(window))
            .and_then(|context| SoftbufferSurface::new(&context, Arc::clone(window)))
        {
            Ok(surface) => {
                tracing::info!("CPU renderer initialized");
                self.cpu_surface = Some(surface);
                self.surface_configured = false;
                self.needs_redraw = true;
            }
            Err(err) => {
                tracing::error!("CPU renderer init failed: {err}");
            }
        }
    }

    /// 执行渲染
    fn render(&mut self, width: u32, height: u32) {
        let mut gpu = self.gpu_renderer.take();
        if let Some(ref mut renderer) = gpu {
            self.render_frame(renderer, width, height);
        } else if self.cpu_surface.is_some() {
            self.render_cpu(width, height);
        }
        self.gpu_renderer = gpu;
    }

    /// 渲染一帧
    fn render_frame(&mut self, gpu: &mut GpuRenderer, width: u32, height: u32) {
        let (fills, glyphs) = self.build_scene(width, height);
        gpu.render_scene_scaled(
            &fills,
            &self.font_loader,
            &mut self.glyph_cache,
            &glyphs,
            self.scale_factor,
        );
    }

    /// CPU 软件渲染一帧
    fn render_cpu(&mut self, width: u32, height: u32) {
        let (fills, glyphs) = self.build_scene(width, height);
        let fb = render_scene_to_framebuffer(
            width,
            height,
            self.scale_factor,
            &fills,
            &self.font_loader,
            &mut self.glyph_cache,
            &glyphs,
        );
        let Some(surface) = self.cpu_surface.as_mut() else {
            return;
        };

        let sw = match NonZeroU32::new(fb.width) {
            Some(width) => width,
            None => return,
        };
        let sh = match NonZeroU32::new(fb.height) {
            Some(height) => height,
            None => return,
        };

        if let Err(err) = surface.resize(sw, sh) {
            tracing::error!("CPU surface resize failed: {err}");
            return;
        }

        let mut buffer = match surface.buffer_mut() {
            Ok(buffer) => buffer,
            Err(err) => {
                tracing::error!("CPU surface buffer failed: {err}");
                return;
            }
        };

        for (dst, rgba) in buffer.iter_mut().zip(fb.data.chunks_exact(4)) {
            *dst = ((rgba[0] as u32) << 16) | ((rgba[1] as u32) << 8) | rgba[2] as u32;
        }

        if let Err(err) = buffer.present() {
            tracing::error!("CPU surface present failed: {err}");
        }
    }

    /// 构建浏览器 UI 渲染图元
    fn build_scene(&mut self, width: u32, height: u32) -> (Vec<FillPrimitive>, Vec<GlyphDraw>) {
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

        // 3. 标签内容（带布局缓存）
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

        // 11. 查找栏（覆盖在页面内容上方）
        if self.shell.find_state().is_active() {
            self.render_find_bar(&mut fills, &mut glyphs, width, font_size);
        }

        // 12. 自动补全下拉（覆盖在页面内容上方）
        if self.address_bar_focused && !self.autocomplete.suggestions.is_empty() {
            self.render_autocomplete(&mut fills, &mut glyphs, width, font_size);
        }

        // 13. 状态栏
        self.render_status_bar(&mut fills, &mut glyphs, width, height, font_size);

        (fills, glyphs)
    }

    /// 渲染标签页（带完整标签条）
    fn render_tabs(&mut self, fills: &mut Vec<FillPrimitive>, glyphs: &mut Vec<GlyphDraw>, width: u32, font_size: f32) {
        let active_id = self.shell.active_tab_id();
        let tab_count = self.shell.tab_count();
        if tab_count == 0 {
            return;
        }

        // 计算每个标签宽度（均分，限制最大/最小宽度）
        let new_tab_btn_w = 32.0_f32;
        let available_width = width as f32 - new_tab_btn_w;
        let tab_w = (available_width / tab_count as f32).clamp(layout::TAB_MIN_WIDTH, layout::TAB_MAX_WIDTH);

        // 更新标签布局缓存
        self.tab_layout.clear();
        let mut x = 0.0_f32;

        for tab in self.shell.tabs() {
            let is_active = Some(tab.id()) == active_id;
            let is_hovered = !is_active && {
                let mx = self.mouse_pos.0 as f32;
                let my = self.mouse_pos.1 as f32;
                mx >= x && mx < x + tab_w && my < layout::TAB_BAR_HEIGHT
            };

            // 标签背景
            let bg = if is_active {
                colors::TAB_ACTIVE_BG
            } else if is_hovered {
                colors::TAB_HOVER_BG
            } else {
                colors::TAB_BAR_BG
            };
            fills.push(rect_fill(x, 0.0, tab_w - 1.0, layout::TAB_BAR_HEIGHT, bg));

            // 标签文本（截断显示）
            if let Some(fid) = self.font_id {
                let label = tab.title().unwrap_or_else(|| tab.url().unwrap_or("New Tab"));
                let max_chars = ((tab_w - 40.0) / (font_size * 0.6)).max(3.0) as usize;
                let truncated: String = label.chars().take(max_chars).collect();
                draw_text(&truncated, x + 10.0, 8.0, font_size, colors::TAB_TEXT, fid, glyphs);
            }

            // 关闭按钮（×）
            if let Some(fid) = self.font_id {
                let close_x = x + tab_w - 24.0;
                let close_y = 8.0_f32;
                glyphs.push(GlyphDraw {
                    ch: '×',
                    x: close_x,
                    baseline_y: close_y + font_size,
                    color: colors::TAB_CLOSE,
                    font_id: fid,
                    font_size: font_size * 0.8,
                });
            }

            // 保存布局信息
            self.tab_layout.push((tab.id(), x, tab_w));
            x += tab_w;
        }

        // 新建标签按钮 (+)
        if let Some(fid) = self.font_id {
            let btn_x = width as f32 - new_tab_btn_w;
            let is_hovered = {
                let mx = self.mouse_pos.0 as f32;
                let my = self.mouse_pos.1 as f32;
                mx >= btn_x && my < layout::TAB_BAR_HEIGHT
            };
            if is_hovered {
                fills.push(rect_fill(
                    btn_x,
                    0.0,
                    new_tab_btn_w,
                    layout::TAB_BAR_HEIGHT,
                    colors::TAB_HOVER_BG,
                ));
            }
            let text_x = btn_x + (new_tab_btn_w - font_size * 0.6) / 2.0;
            draw_text("+", text_x, 8.0, font_size, colors::NEW_TAB_BUTTON, fid, glyphs);
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

            // 光标（地址栏聚焦时闪烁效果）
            if self.address_bar_focused {
                let cursor_x = bar_x + 10.0 + self.address_bar_text.len() as f32 * font_size * 0.6;
                fills.push(rect_fill(
                    cursor_x,
                    bar_y + 4.0,
                    1.5,
                    bar_h - 8.0,
                    colors::ADDRESS_BAR_TEXT,
                ));
            }
        }
    }

    /// 渲染页面内容
    fn render_page_content(&mut self, glyphs: &mut Vec<GlyphDraw>, _width: u32, page_y: f32, font_size: f32) {
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

        // 查找栏打开时页面内容下移
        let content_y_offset = if self.shell.find_state().is_active() {
            layout::FIND_BAR_HEIGHT
        } else {
            0.0
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

        let mut y = page_y + content_y_offset;

        if !title.is_empty() {
            draw_text(&title, 20.0, y + 20.0, 24.0, colors::PAGE_TITLE, fid, glyphs);
            y += 52.0;
        }

        if !url.is_empty() {
            draw_text(&url, 20.0, y, 12.0, colors::PAGE_URL, fid, glyphs);
            y += 28.0;
        }

        if is_loading {
            draw_text("Loading...", 20.0, y, font_size, colors::PAGE_HINT, fid, glyphs);
        } else if title.is_empty() && url.is_empty() {
            draw_text(
                "Welcome to ZeroBrowser — Press L to focus address bar, T for new tab",
                20.0,
                y,
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
                draw_text(&info, 20.0, y, font_size, colors::PAGE_BODY, fid, glyphs);
            }
        }
    }

    /// 渲染查找栏
    fn render_find_bar(&self, fills: &mut Vec<FillPrimitive>, glyphs: &mut Vec<GlyphDraw>, width: u32, font_size: f32) {
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

        let y = layout::TOOLBAR_HEIGHT;
        let bar_w = 320.0_f32;
        let bar_x = width as f32 - bar_w - 10.0;

        // 背景
        fills.push(rect_fill(bar_x, y, bar_w, layout::FIND_BAR_HEIGHT, colors::FIND_BAR_BG));

        // 输入框文本
        let display = if self.find_input.is_empty() {
            "Find...".to_string()
        } else {
            self.find_input.clone()
        };
        let text_color = if self.find_input.is_empty() {
            colors::FIND_MATCH_TEXT
        } else {
            colors::FIND_BAR_TEXT
        };
        draw_text(&display, bar_x + 10.0, y + 5.0, font_size, text_color, fid, glyphs);

        // 匹配计数
        let find_state = self.shell.find_state();
        if find_state.total_matches() > 0 {
            let match_text = format!("{}/{}", find_state.current_match(), find_state.total_matches());
            let match_x = bar_x + bar_w - 130.0;
            draw_text(
                &match_text,
                match_x,
                y + 5.0,
                font_size,
                colors::FIND_MATCH_TEXT,
                fid,
                glyphs,
            );
        } else if !self.find_input.is_empty() {
            let no_match_x = bar_x + bar_w - 130.0;
            draw_text(
                "No matches",
                no_match_x,
                y + 5.0,
                font_size,
                colors::FIND_MATCH_TEXT,
                fid,
                glyphs,
            );
        }

        // 上一个/下一个/关闭按钮
        let btn_y = y + 5.0;
        let prev_x = bar_x + bar_w - 100.0;
        let next_x = bar_x + bar_w - 70.0;
        let close_x = bar_x + bar_w - 40.0;
        draw_text("↑", prev_x, btn_y, font_size, colors::FIND_BAR_TEXT, fid, glyphs);
        draw_text("↓", next_x, btn_y, font_size, colors::FIND_BAR_TEXT, fid, glyphs);
        draw_text("×", close_x, btn_y, font_size, colors::FIND_BAR_TEXT, fid, glyphs);
    }

    /// 渲染自动补全下拉
    fn render_autocomplete(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        font_size: f32,
    ) {
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

        let nav_w = layout::NAV_BUTTON_WIDTH * 4.0 + 16.0;
        let bar_x = nav_w + layout::ADDRESS_BAR_PADDING;
        let bar_w = width as f32 - bar_x - layout::ADDRESS_BAR_PADDING;
        let dropdown_y = layout::TAB_BAR_HEIGHT + layout::ADDRESS_BAR_HEIGHT;

        let visible_count = self
            .autocomplete
            .suggestions
            .len()
            .min(layout::AUTOCOMPLETE_MAX_VISIBLE);
        let dropdown_h = visible_count as f32 * layout::AUTOCOMPLETE_ROW_HEIGHT;

        // 下拉背景
        fills.push(rect_fill(bar_x, dropdown_y, bar_w, dropdown_h, colors::AUTOCOMPLETE_BG));

        for (i, sug) in self.autocomplete.suggestions.iter().take(visible_count).enumerate() {
            let row_y = dropdown_y + i as f32 * layout::AUTOCOMPLETE_ROW_HEIGHT;
            let is_hovered = self.autocomplete.hovered_index == Some(i);

            // 悬停高亮
            if is_hovered {
                fills.push(rect_fill(
                    bar_x,
                    row_y,
                    bar_w,
                    layout::AUTOCOMPLETE_ROW_HEIGHT,
                    colors::AUTOCOMPLETE_HOVER_BG,
                ));
            }

            // 书签图标
            let source_label = match sug.source() {
                SuggestionSource::Bookmark => "★",
                SuggestionSource::History => "🕐",
            };
            let text_x = bar_x + 10.0;
            draw_text(
                source_label,
                text_x,
                row_y + 5.0,
                font_size * 0.85,
                if sug.source() == SuggestionSource::Bookmark {
                    colors::AUTOCOMPLETE_BOOKMARK
                } else {
                    colors::AUTOCOMPLETE_URL
                },
                fid,
                glyphs,
            );

            // 标题
            let title = sug.title();
            let max_title_chars = ((bar_w - 180.0) / (font_size * 0.6)).max(10.0) as usize;
            let truncated_title: String = title.chars().take(max_title_chars).collect();
            draw_text(
                &truncated_title,
                text_x + 24.0,
                row_y + 5.0,
                font_size * 0.85,
                colors::AUTOCOMPLETE_TEXT,
                fid,
                glyphs,
            );

            // URL（右侧截断）
            let url = sug.url();
            let url_x = bar_x + bar_w - 10.0;
            let max_url_chars = ((bar_w * 0.4) / (font_size * 0.5)).max(8.0) as usize;
            let truncated_url: String = url.chars().take(max_url_chars).collect();
            // 右对齐 URL：估算宽度并从右边开始
            let url_display_width = truncated_url.len() as f32 * font_size * 0.5;
            draw_text(
                &truncated_url,
                url_x - url_display_width,
                row_y + 5.0,
                font_size * 0.75,
                colors::AUTOCOMPLETE_URL,
                fid,
                glyphs,
            );
        }

        // 边框
        fills.push(rect_fill(bar_x, dropdown_y + dropdown_h, bar_w, 1.0, colors::SEPARATOR));
    }

    /// 渲染状态栏
    fn render_status_bar(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        height: u32,
        _font_size: f32,
    ) {
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

        let status_y = height as f32 - layout::STATUS_BAR_HEIGHT;

        // 状态栏背景
        fills.push(rect_fill(
            0.0,
            status_y,
            width as f32,
            layout::STATUS_BAR_HEIGHT,
            colors::BACKGROUND,
        ));

        // 分隔线
        fills.push(rect_fill(0.0, status_y, width as f32, 1.0, colors::SEPARATOR));

        // 左侧：缩放信息
        let zoom = self.shell.zoom();
        if (zoom - 1.0).abs() > f32::EPSILON {
            let zoom_text = format!("{}%", (zoom * 100.0) as u32);
            draw_text(&zoom_text, 10.0, status_y + 3.0, 11.0, colors::STATUS_TEXT, fid, glyphs);
        }

        // 右侧：标签页数量
        let tab_count = self.shell.tab_count();
        let tabs_text = format!("Tabs: {tab_count}");
        let tabs_width = tabs_text.len() as f32 * 11.0 * 0.6;
        draw_text(
            &tabs_text,
            width as f32 - tabs_width - 10.0,
            status_y + 3.0,
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

/// URL 规范化 — 支持 URL 和搜索引擎回退
fn normalize_url(input: &str, shell: &BrowserShell) -> String {
    if input.starts_with("http://") || input.starts_with("https://") {
        return input.to_string();
    }
    if input.starts_with("ftp://") || input.starts_with("file://") || input.starts_with("data:") {
        return input.to_string();
    }
    // 包含点且无空格 → 可能是域名
    if input.contains('.') && !input.contains(' ') {
        return format!("https://{input}");
    }
    // 看起来不像 URL → 使用搜索引擎
    shell.settings().search(input)
}

/// 去除 URL 协议前缀
#[allow(dead_code)]
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

fn parse_render_mode_from_args() -> Result<RenderMode, String> {
    let mut args = std::env::args().skip(1);
    let mut cli_mode = None;

    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            print_usage();
            std::process::exit(0);
        }

        if let Some(value) = arg.strip_prefix("--renderer=") {
            cli_mode = Some(value.parse()?);
            continue;
        }

        if arg == "--renderer" {
            let value = args
                .next()
                .ok_or_else(|| format!("--renderer requires {}", RenderMode::values()))?;
            cli_mode = Some(value.parse()?);
        }
    }

    Ok(cli_mode.or(RenderMode::from_env()?).unwrap_or_default())
}

fn print_usage() {
    println!("Usage: zero-browser [--renderer {}]", RenderMode::values());
    println!("Environment: {}={}", RenderMode::ENV_VAR, RenderMode::values());
}

fn logical_size_from_window(window: &winit::window::Window) -> ((u32, u32), f32) {
    let physical = window.inner_size();
    let scale = normalized_window_scale(window.scale_factor());
    let logical_width = ((physical.width as f32 / scale).round() as u32).max(1);
    let logical_height = ((physical.height as f32 / scale).round() as u32).max(1);
    ((logical_width, logical_height), scale)
}

fn normalized_window_scale(scale: f64) -> f32 {
    if scale.is_finite() && scale > 0.0 {
        scale as f32
    } else {
        1.0
    }
}

fn main() {
    // 初始化日志
    tracing_subscriber::fmt().init();

    tracing::info!("ZeroBrowser starting...");

    let render_mode = match parse_render_mode_from_args() {
        Ok(mode) => mode,
        Err(err) => {
            eprintln!("{err}");
            print_usage();
            std::process::exit(2);
        }
    };
    tracing::info!("Renderer mode: {render_mode}");

    let config = WindowConfig::new("ZeroBrowser")
        .with_size(1024, 768)
        .with_resizable(true);

    let runtime = HostRuntime::new(config);
    let mut app = BrowserApp::new(render_mode);

    // 加载欢迎页
    app.new_tab(None);

    tracing::info!("Entering event loop...");

    if let Err(e) = runtime.run_with_window(move |event, window| {
        match event {
            AppEvent::RedrawRequested => {
                if !app.surface_configured {
                    if let Some(ref win) = window
                        && app.gpu_renderer.is_none()
                        && app.cpu_surface.is_none()
                    {
                        let (logical_size, scale_factor) = logical_size_from_window(win);
                        let physical_size = win.inner_size();
                        app.window_size = logical_size;
                        app.physical_size = (physical_size.width, physical_size.height);
                        app.scale_factor = scale_factor;

                        match app.render_mode {
                            RenderMode::Cpu => app.init_cpu_surface(win),
                            RenderMode::Gpu | RenderMode::Auto => {
                                app.init_gpu(win);
                                if app.gpu_renderer.is_none() && matches!(app.render_mode, RenderMode::Auto) {
                                    app.init_cpu_surface(win);
                                }
                            }
                        }
                    }
                    if let Some(ref mut gpu) = app.gpu_renderer {
                        let (w, h) = app.physical_size;
                        gpu.configure_surface(w, h);
                        app.surface_configured = true;
                    } else if app.cpu_surface.is_some() {
                        app.surface_configured = true;
                    }
                }
                app.render(app.window_size.0, app.window_size.1);
                app.needs_redraw = false;
            }
            AppEvent::Resized { width, height } if width > 0 && height > 0 => {
                tracing::debug!("Window resized: {width}x{height}");
                app.physical_size = (width, height);
                if let Some(ref win) = window {
                    let (logical_size, scale_factor) = logical_size_from_window(win);
                    app.window_size = logical_size;
                    app.scale_factor = scale_factor;
                } else {
                    app.window_size = (width, height);
                    app.scale_factor = 1.0;
                }
                if let Some(ref mut gpu) = app.gpu_renderer {
                    gpu.configure_surface(width, height);
                }
                app.needs_redraw = true;
            }
            AppEvent::ScaleFactorChanged { scale_factor } => {
                tracing::debug!("Window scale factor changed: {scale_factor}");
                if let Some(ref win) = window {
                    let physical_size = win.inner_size();
                    let (logical_size, normalized_scale) = logical_size_from_window(win);
                    app.physical_size = (physical_size.width, physical_size.height);
                    app.window_size = logical_size;
                    app.scale_factor = normalized_scale;
                    if let Some(ref mut gpu) = app.gpu_renderer {
                        gpu.configure_surface(physical_size.width, physical_size.height);
                    }
                } else {
                    app.scale_factor = normalized_window_scale(scale_factor);
                }
                app.needs_redraw = true;
            }
            AppEvent::CloseRequested => {
                tracing::info!("Window closed");
            }
            AppEvent::KeyboardInput { key, pressed } if pressed => {
                app.handle_key(&key, true);
            }
            AppEvent::MouseMoved { x, y } => {
                app.handle_mouse_move(x / app.scale_factor as f64, y / app.scale_factor as f64);
            }
            AppEvent::MouseInput { button: _, pressed } => {
                app.handle_mouse_click(app.mouse_pos.0, app.mouse_pos.1, pressed);
            }
            AppEvent::MouseWheel { delta: _ } => {
                // 未来：页面滚动
            }
            AppEvent::Focused => {
                tracing::debug!("Window focused");
            }
            AppEvent::Unfocused => {
                app.address_bar_focused = false;
                app.needs_redraw = true;
            }
            _ => {}
        }

        if app.needs_redraw
            && let Some(ref win) = window
        {
            win.request_redraw();
        }
    }) {
        tracing::error!("Event loop error: {e}");
        std::process::exit(1);
    }

    tracing::info!("ZeroBrowser exited");
}
