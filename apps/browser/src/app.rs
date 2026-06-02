//! 浏览器应用核心状态和事件处理

use std::collections::HashMap;

use zero_browser_shell::{BrowserShell, ContextMenu, ContextType, SuggestionSource, TabId};
use zero_render_foundation::color::Color;
use zero_render_foundation::config::RenderMode;
use zero_render_foundation::cpu::render_scene_to_framebuffer;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::gpu::renderer::{GlyphDraw, GpuRenderer};
use zero_render_foundation::primitive::{FillPrimitive, RenderPrimitives};
use zero_webview::WebViewBuilder;

use crate::colors;
use crate::layout;
use crate::pages;

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

/// 右键上下文菜单状态
pub struct ContextMenuState {
    /// 是否显示
    pub visible: bool,
    /// 菜单类型（预留用于区分不同场景的菜单行为）
    #[allow(dead_code)]
    pub context_type: ContextType,
    /// 菜单项标签列表
    pub items: Vec<String>,
    /// 悬停索引
    pub hovered_index: Option<usize>,
    /// 菜单左上角物理像素坐标
    pub x: f32,
    pub y: f32,
}

impl ContextMenuState {
    fn new() -> Self {
        Self {
            visible: false,
            context_type: ContextType::Page,
            items: Vec::new(),
            hovered_index: None,
            x: 0.0,
            y: 0.0,
        }
    }

    fn close(&mut self) {
        self.visible = false;
        self.items.clear();
        self.hovered_index = None;
    }
}

/// 浏览器应用状态
pub struct BrowserApp {
    /// 浏览器 Shell（标签页、书签、历史）
    pub shell: BrowserShell,
    /// 每个标签页对应的 WebView
    webviews: HashMap<TabId, zero_webview::WebView>,
    /// GPU 渲染器
    gpu_renderer: Option<GpuRenderer>,
    /// 渲染模式
    render_mode: RenderMode,
    /// 字体加载器
    font_loader: FontLoader,
    /// Glyph 缓存
    glyph_cache: GlyphCache,
    /// 已加载的系统字体 ID
    font_id: Option<u32>,
    /// 是否已初始化 GPU 表面
    pub surface_configured: bool,
    /// 窗口是否获得焦点（Wayland 下失焦时 surface 可能挂起）
    pub window_focused: bool,
    /// 地址栏当前文本
    address_bar_text: String,
    /// 地址栏是否获得焦点
    pub address_bar_focused: bool,
    /// 窗口物理像素尺寸
    pub physical_size: (u32, u32),
    /// 窗口缩放因子
    pub scale_factor: f32,
    /// 是否需要重绘
    pub needs_redraw: bool,
    /// 鼠标位置（用于悬停检测）
    pub mouse_pos: (f64, f64),
    /// Ctrl 键是否按住
    ctrl_pressed: bool,
    /// Shift 键是否按住
    shift_pressed: bool,
    /// 自动补全状态
    autocomplete: AutocompleteState,
    /// 查找栏输入文本
    find_input: String,
    /// 标签页布局缓存：每个标签页的 (x, width) 位置信息
    tab_layout: Vec<(TabId, f32, f32)>,
    /// 右键上下文菜单状态
    context_menu: ContextMenuState,
    /// 页面滚动偏移（物理像素）
    scroll_offset: HashMap<TabId, f32>,
}

impl BrowserApp {
    /// 创建新的浏览器应用
    pub fn new(render_mode: RenderMode) -> Self {
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
            render_mode,
            font_loader,
            glyph_cache: GlyphCache::new(8192),
            font_id,
            surface_configured: false,
            window_focused: true,
            address_bar_text: String::new(),
            address_bar_focused: false,
            physical_size: (1024, 768),
            scale_factor: 1.0,
            needs_redraw: true,
            mouse_pos: (0.0, 0.0),
            ctrl_pressed: false,
            shift_pressed: false,
            autocomplete: AutocompleteState::new(),
            find_input: String::new(),
            tab_layout: Vec::new(),
            context_menu: ContextMenuState::new(),
            scroll_offset: HashMap::new(),
        }
    }

    /// 设置窗口逻辑尺寸
    pub fn set_window_size(&mut self, size: (u32, u32)) {
        // 保留兼容：仅记录但不用于关键路径
        let _ = size;
    }

    /// GPU 渲染器是否存在
    pub fn gpu_renderer_is_some(&self) -> bool {
        self.gpu_renderer.is_some()
    }

    /// GPU 渲染器是否不存在
    pub fn gpu_renderer_is_none(&self) -> bool {
        self.gpu_renderer.is_none()
    }

    /// 获取 GPU 渲染器可变引用
    pub fn gpu_renderer_as_mut(&mut self) -> Option<&mut GpuRenderer> {
        self.gpu_renderer.as_mut()
    }

    /// 获取当前渲染模式
    pub fn render_mode(&self) -> RenderMode {
        self.render_mode
    }

    /// 调整所有 WebView 视口尺寸
    pub fn resize_all_webviews(&mut self, w: u32, h: u32) {
        for wv in self.webviews.values_mut() {
            wv.resize(w, h);
        }
    }

    /// 测试用：构建场景（暴露私有方法给测试模块）
    #[cfg(test)]
    pub fn build_scene_for_test(&mut self, width: u32, height: u32) -> (Vec<FillPrimitive>, Vec<GlyphDraw>) {
        self.build_scene(width, height)
    }

    /// 测试用：向指定标签的 WebView 加载 HTML
    #[cfg(test)]
    pub fn load_webview_html(&mut self, tab_id: TabId, html: &str, css: Option<&str>) {
        if let Some(wv) = self.webviews.get_mut(&tab_id) {
            wv.load_html(html, css);
        }
    }

    /// 测试用：获取 Ctrl 修饰键状态
    #[cfg(test)]
    pub fn is_ctrl_pressed(&self) -> bool {
        self.ctrl_pressed
    }

    /// 测试用：获取地址栏文本
    #[cfg(test)]
    pub fn address_bar_text(&self) -> &str {
        &self.address_bar_text
    }

    /// 计算网页内容区域物理像素尺寸
    pub fn content_physical_size(&self) -> (u32, u32) {
        let s = self.scale_factor;
        let chrome_h = (layout::TOOLBAR_HEIGHT + layout::BOOKMARKS_BAR_HEIGHT + layout::STATUS_BAR_HEIGHT) * s;
        let content_w = self.physical_size.0;
        let content_h = (self.physical_size.1 as f32 - chrome_h).max(0.0) as u32;
        (content_w, content_h)
    }

    /// 创建指定视口尺寸的 WebView
    fn create_webview(&self) -> zero_webview::WebView {
        let (w, h) = self.content_physical_size();
        WebViewBuilder::new().width(w).height(h).build()
    }

    /// 获取或创建活跃标签页的 WebView
    pub fn ensure_webview(&mut self, tab_id: TabId) {
        if !self.webviews.contains_key(&tab_id) {
            let wv = self.create_webview();
            self.webviews.insert(tab_id, wv);
        }
    }

    /// 通过 WebView 加载指定标签页 URL
    fn fetch_tab_url(&mut self, tab_id: TabId, url: &str) {
        let result = match self.webviews.get_mut(&tab_id) {
            Some(wv) => wv.fetch_url(url),
            None => return,
        };

        match result {
            Ok(_) => {
                let title = self
                    .webviews
                    .get(&tab_id)
                    .and_then(|wv| wv.title().map(str::to_string))
                    .unwrap_or_else(|| url.to_string());
                self.shell.on_page_loaded(&title);
            }
            Err(e) => {
                tracing::warn!("Failed to fetch URL: {e}, loading error page");
                let error = e.to_string();
                let error_page = pages::generate_error_page(url, &error);
                if let Some(wv) = self.webviews.get_mut(&tab_id) {
                    wv.load_html(&error_page, None);
                }
                self.shell.on_page_error(&error);
            }
        }
    }

    /// 导航到指定 URL
    pub fn navigate_to(&mut self, url: &str) {
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

        // 重置滚动偏移
        self.scroll_offset.insert(tab_id, 0.0);

        self.fetch_tab_url(tab_id, &url);
        self.needs_redraw = true;
    }

    /// 创建新标签页
    pub fn new_tab(&mut self, url: Option<&str>) {
        let tab_id = self.shell.new_tab(url);
        let webview = self.create_webview();
        self.webviews.insert(tab_id, webview);

        if let Some(url) = url {
            self.address_bar_text = url.to_string();
        } else {
            self.address_bar_text.clear();
            if let Some(wv) = self.webviews.get_mut(&tab_id) {
                wv.load_html(pages::WELCOME_HTML, None);
            }
        }

        self.scroll_offset.insert(tab_id, 0.0);
        self.needs_redraw = true;
    }

    /// 关闭活跃标签页
    pub fn close_active_tab(&mut self) {
        if let Some(tab_id) = self.shell.active_tab_id() {
            self.webviews.remove(&tab_id);
            self.scroll_offset.remove(&tab_id);
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
        self.scroll_offset.remove(&id);
        self.shell.close_tab(id);

        if self.shell.is_empty() {
            self.new_tab(None);
        }

        self.update_address_bar_from_active_tab();
        self.needs_redraw = true;
    }

    /// 刷新当前页面
    pub fn refresh_page(&mut self) {
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

        self.fetch_tab_url(tab_id, &url);
        self.needs_redraw = true;
    }

    /// 执行后退导航
    pub fn go_back(&mut self) {
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

        self.fetch_tab_url(tab_id, &url);
        self.needs_redraw = true;
    }

    /// 执行前进导航
    pub fn go_forward(&mut self) {
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

        self.fetch_tab_url(tab_id, &url);
        self.needs_redraw = true;
    }

    /// 打开设置页面（about:preferences）
    pub fn open_settings_page(&mut self) {
        let html = pages::generate_settings_html(self.shell.settings());
        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return,
        };
        self.ensure_webview(tab_id);
        if let Some(wv) = self.webviews.get_mut(&tab_id) {
            wv.load_html(&html, None);
        }
        self.address_bar_text = "zero://settings".to_string();
        self.needs_redraw = true;
    }

    /// 处理鼠标滚轮滚动
    pub fn handle_scroll(&mut self, delta: zero_host_runtime::event::MouseScrollDelta) {
        // 上下文菜单打开时不滚动
        if self.context_menu.visible {
            return;
        }

        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return,
        };

        // 提取 Y 方向滚动量
        let delta_y = match delta {
            zero_host_runtime::event::MouseScrollDelta::PixelDelta(_, y) => y as f32,
            zero_host_runtime::event::MouseScrollDelta::LineDelta(_, y) => y * 40.0, // 每行约 40 像素
        };

        let content_h = self.content_physical_size().1 as f32;

        // 获取页面实际高度（如果有的话）
        let page_height = self
            .webviews
            .get(&tab_id)
            .and_then(|wv| wv.last_render())
            .map(|r| {
                r.primitives
                    .fills
                    .iter()
                    .map(|f| f.rect.origin.y + f.rect.size.height)
                    .fold(0.0f32, f32::max)
            })
            .unwrap_or(content_h);

        let max_scroll = (page_height - content_h).max(0.0);
        let offset = self.scroll_offset.entry(tab_id).or_insert(0.0);
        *offset = (*offset + delta_y).clamp(0.0, max_scroll);

        self.needs_redraw = true;
    }

    /// 处理键盘输入
    pub fn handle_key(&mut self, key: &str, pressed: bool) {
        // 追踪修饰键状态
        match key {
            "Control" => {
                self.ctrl_pressed = pressed;
                return;
            }
            "Shift" => {
                self.shift_pressed = pressed;
                return;
            }
            _ => {}
        }

        // 只处理按键按下事件
        if !pressed {
            return;
        }

        // 上下文菜单打开时，Escape 关闭菜单，其他按键忽略
        if self.context_menu.visible {
            match key {
                "Escape" => {
                    self.context_menu.close();
                    self.needs_redraw = true;
                }
                "Up" if !self.context_menu.items.is_empty() => {
                    let next = self
                        .context_menu
                        .hovered_index
                        .map(|i| {
                            if i > 0 {
                                i - 1
                            } else {
                                self.context_menu.items.len() - 1
                            }
                        })
                        .unwrap_or(self.context_menu.items.len() - 1);
                    self.context_menu.hovered_index = Some(next);
                    self.needs_redraw = true;
                }
                "Down" if !self.context_menu.items.is_empty() => {
                    let next = self
                        .context_menu
                        .hovered_index
                        .map(|i| (i + 1) % self.context_menu.items.len())
                        .unwrap_or(0);
                    self.context_menu.hovered_index = Some(next);
                    self.needs_redraw = true;
                }
                "Enter" => {
                    self.activate_context_menu_item();
                }
                _ => {}
            }
            return;
        }

        if self.shell.find_state().is_active() {
            self.handle_find_key(key);
        } else if self.address_bar_focused {
            self.handle_address_bar_key(key);
        } else {
            self.handle_global_key(key);
        }
    }

    fn handle_find_key(&mut self, key: &str) {
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
    }

    fn handle_address_bar_key(&mut self, key: &str) {
        match key {
            "Enter" => {
                let url = self.address_bar_text.trim().to_string();
                if !url.is_empty() {
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
                if let Some(sug) = self.autocomplete.suggestions.first() {
                    self.address_bar_text = sug.url().to_string();
                    self.autocomplete.clear();
                    self.needs_redraw = true;
                }
            }
            _ => {
                if key.len() == 1 {
                    self.address_bar_text.push_str(key);
                    self.update_autocomplete();
                    self.needs_redraw = true;
                }
            }
        }
    }

    fn handle_global_key(&mut self, key: &str) {
        // Ctrl 修饰键快捷键
        if self.ctrl_pressed {
            match key {
                "l" | "L" => {
                    self.address_bar_focused = true;
                    self.address_bar_text.clear();
                    self.needs_redraw = true;
                }
                "t" | "T" => {
                    self.new_tab(None);
                }
                "w" | "W" => {
                    self.close_active_tab();
                }
                "r" | "R" => {
                    self.refresh_page();
                }
                "f" | "F" => {
                    self.find_input.clear();
                    self.shell.find_close();
                    self.needs_redraw = true;
                }
                "d" | "D" => {
                    // 收藏当前页面
                    self.shell.add_bookmark();
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
                "," => {
                    // Ctrl+, 打开设置页面
                    self.open_settings_page();
                }
                _ => {}
            }
            return;
        }

        // 无修饰键的全局快捷键（保留兼容无 Ctrl 的单键模式）
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
                self.shell.find_next();
                self.find_input = self.shell.find_state().query().to_string();
                self.needs_redraw = true;
            }
            _ => {}
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
    pub fn handle_mouse_move(&mut self, x: f64, y: f64) {
        let old_pos = self.mouse_pos;
        self.mouse_pos = (x, y);

        // 上下文菜单悬停检测
        if self.context_menu.visible {
            let hovered = self.context_menu_hit_test(x, y);
            if hovered != self.context_menu.hovered_index {
                self.context_menu.hovered_index = hovered;
                self.needs_redraw = true;
            }
        }

        // 自动补全悬停
        if self.address_bar_focused && !self.autocomplete.suggestions.is_empty() {
            let hovered = self.autocomplete_hit_test(x, y);
            if hovered != self.autocomplete.hovered_index {
                self.autocomplete.hovered_index = hovered;
                self.needs_redraw = true;
            }
        }

        if (old_pos.0 - x).abs() > 1.0 || (old_pos.1 - y).abs() > 1.0 {
            let toolbar_h = (layout::TOOLBAR_HEIGHT + layout::BOOKMARKS_BAR_HEIGHT) * self.scale_factor;
            if (y as f32) < toolbar_h {
                self.needs_redraw = true;
            }
        }
    }

    /// 处理鼠标点击（物理像素坐标）
    pub fn handle_mouse_click(&mut self, x: f64, y: f64, pressed: bool, button: &str) {
        // 鼠标释放不做处理（右键除外）
        if !pressed {
            return;
        }

        // 右键 → 上下文菜单
        if button == "Right" {
            self.show_context_menu(x, y);
            return;
        }

        // 左键点击时关闭上下文菜单
        if self.context_menu.visible {
            if let Some(idx) = self.context_menu_hit_test(x, y) {
                // 点击菜单项
                self.context_menu.hovered_index = Some(idx);
                self.activate_context_menu_item();
                return;
            }
            // 点击菜单外关闭
            self.context_menu.close();
            self.needs_redraw = true;
            return;
        }

        let s = self.scale_factor;
        let y_f = y as f32;
        let x_f = x as f32;
        let width = self.physical_size.0 as f32;

        let tab_bar_h = layout::TAB_BAR_HEIGHT * s;
        let addr_bar_h = layout::ADDRESS_BAR_HEIGHT * s;
        let toolbar_h = layout::TOOLBAR_HEIGHT * s;
        let bookmarks_bar_h = layout::BOOKMARKS_BAR_HEIGHT * s;
        let chrome_top = toolbar_h + bookmarks_bar_h;
        let nav_w = (layout::NAV_BUTTON_WIDTH * 4.0 + 16.0) * s;
        let nav_btn_w = layout::NAV_BUTTON_WIDTH * s;
        let addr_padding = layout::ADDRESS_BAR_PADDING * s;
        let tab_close_size = layout::TAB_CLOSE_SIZE * s;
        let autocomplete_row_h = layout::AUTOCOMPLETE_ROW_HEIGHT * s;

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
            let autocomplete_top = tab_bar_h + addr_bar_h;
            let autocomplete_height = self
                .autocomplete
                .suggestions
                .len()
                .min(layout::AUTOCOMPLETE_MAX_VISIBLE) as f32
                * autocomplete_row_h;
            if y_f >= autocomplete_top && y_f < autocomplete_top + autocomplete_height {
                return;
            }
            self.autocomplete.clear();
        }

        // 2. 标签栏区域点击
        if y_f < tab_bar_h {
            let new_tab_x = width - 32.0 * s;
            if x_f >= new_tab_x && x_f <= width {
                self.new_tab(None);
                return;
            }

            for &(id, tab_x, tab_w) in &self.tab_layout {
                if x_f >= tab_x && x_f < tab_x + tab_w {
                    let close_x = tab_x + tab_w - 24.0 * s;
                    let close_y_center = tab_bar_h / 2.0;
                    if x_f >= close_x
                        && x_f <= close_x + tab_close_size
                        && (y_f - close_y_center).abs() <= tab_close_size / 2.0
                    {
                        self.close_tab_by_id(id);
                        return;
                    }
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
        if y_f < tab_bar_h + addr_bar_h {
            let addr_bar_x = nav_w + addr_padding;

            if x_f < nav_w {
                let button_index = ((x_f - 8.0 * s) / nav_btn_w) as i32;
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

            if x_f >= addr_bar_x && x_f <= width - addr_padding {
                if !self.address_bar_focused {
                    self.address_bar_focused = true;
                    self.needs_redraw = true;
                }
                return;
            }
        }

        // 4. 书签栏区域点击
        if y_f >= toolbar_h && y_f < chrome_top {
            self.handle_bookmark_bar_click(x_f, y_f, toolbar_h, width, s);
            return;
        }

        // 5. 查找栏区域点击
        if self.shell.find_state().is_active() && y_f >= chrome_top && y_f < chrome_top + layout::FIND_BAR_HEIGHT * s {
            let close_x = width - 40.0 * s;
            if x_f >= close_x {
                self.shell.find_close();
                self.find_input.clear();
                self.needs_redraw = true;
                return;
            }
            let prev_x = width - 100.0 * s;
            let next_x = width - 70.0 * s;
            if x_f >= prev_x && x_f < prev_x + 28.0 * s {
                self.shell.find_previous();
                self.needs_redraw = true;
                return;
            }
            if x_f >= next_x && x_f < next_x + 28.0 * s {
                self.shell.find_next();
                self.needs_redraw = true;
                return;
            }
            return;
        }

        // 6. 页面内容区域 — 取消地址栏焦点
        if y_f >= chrome_top && self.address_bar_focused {
            self.address_bar_focused = false;
            self.autocomplete.clear();
            self.needs_redraw = true;
        }
    }

    /// 处理书签栏点击
    fn handle_bookmark_bar_click(&mut self, x: f32, _y: f32, _bar_y: f32, _width: f32, s: f32) {
        let font_size = 12.0 * s;
        let mut bx = 8.0 * s;
        let mut target_url: Option<String> = None;

        let bookmarks = self.shell.bookmarks();
        for bm in bookmarks.list_root() {
            let label = bm.title();
            let item_w = label.len() as f32 * font_size * 0.6 + 24.0 * s;
            if x >= bx && x < bx + item_w {
                target_url = Some(bm.url().to_string());
                break;
            }
            bx += item_w + 8.0 * s;
        }

        if let Some(url) = target_url {
            self.navigate_to(&url);
        }
    }

    /// 显示右键上下文菜单
    fn show_context_menu(&mut self, x: f64, y: f64) {
        let s = self.scale_factor;
        let y_f = y as f32;
        let chrome_top = (layout::TOOLBAR_HEIGHT + layout::BOOKMARKS_BAR_HEIGHT) * s;

        // 工具栏区域不显示上下文菜单
        if y_f < chrome_top {
            return;
        }

        let context_type = ContextType::Page;
        let menu = ContextMenu::new(context_type);
        let items: Vec<String> = menu
            .items()
            .iter()
            .map(|mi| {
                if mi.is_separator() {
                    "---".to_string()
                } else {
                    mi.label().to_string()
                }
            })
            .collect();

        self.context_menu = ContextMenuState {
            visible: true,
            context_type,
            items,
            hovered_index: None,
            x: x as f32,
            y: y as f32,
        };
        self.needs_redraw = true;
    }

    /// 激活上下文菜单中选中的项
    fn activate_context_menu_item(&mut self) {
        let idx = match self.context_menu.hovered_index {
            Some(i) => i,
            None => return,
        };

        let label = match self.context_menu.items.get(idx) {
            Some(l) => l.clone(),
            None => return,
        };

        self.context_menu.close();
        self.needs_redraw = true;

        match label.as_str() {
            "后退" => self.go_back(),
            "前进" => self.go_forward(),
            "重新加载" => self.refresh_page(),
            _ => {}
        }
    }

    /// 上下文菜单命中检测
    fn context_menu_hit_test(&self, x: f64, y: f64) -> Option<usize> {
        if !self.context_menu.visible {
            return None;
        }

        let s = self.scale_factor;
        let menu_x = self.context_menu.x;
        let menu_y = self.context_menu.y;
        let row_h = 28.0 * s;
        let menu_w = 200.0 * s;
        let menu_h = self.context_menu.items.len() as f32 * row_h;

        let x_f = x as f32;
        let y_f = y as f32;

        if x_f < menu_x || x_f > menu_x + menu_w || y_f < menu_y || y_f > menu_y + menu_h {
            return None;
        }

        let idx = ((y_f - menu_y) / row_h) as usize;
        if idx < self.context_menu.items.len() {
            Some(idx)
        } else {
            None
        }
    }

    /// 自动补全下拉命中检测（物理像素坐标）
    fn autocomplete_hit_test(&self, x: f64, y: f64) -> Option<usize> {
        let s = self.scale_factor;
        let nav_w = (layout::NAV_BUTTON_WIDTH * 4.0 + 16.0) * s;
        let bar_x = nav_w + layout::ADDRESS_BAR_PADDING * s;
        let bar_w = self.physical_size.0 as f32 - bar_x - layout::ADDRESS_BAR_PADDING * s;

        let autocomplete_top = (layout::TAB_BAR_HEIGHT + layout::ADDRESS_BAR_HEIGHT) * s;
        let y_f = y as f32;
        let x_f = x as f32;

        if x_f < bar_x || x_f > bar_x + bar_w || y_f < autocomplete_top {
            return None;
        }

        let row_offset = y_f - autocomplete_top;
        if row_offset < 0.0 {
            return None;
        }

        let row_h = layout::AUTOCOMPLETE_ROW_HEIGHT * s;
        let index = (row_offset / row_h) as usize;
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
    pub fn init_gpu(&mut self, window: &std::sync::Arc<winit::window::Window>) {
        if matches!(self.render_mode, RenderMode::Cpu) {
            return;
        }

        match GpuRenderer::new_for_window(std::sync::Arc::clone(window)) {
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
    pub fn init_cpu_surface(
        &mut self,
        window: &std::sync::Arc<winit::window::Window>,
        cpu_surface: &mut Option<
            softbuffer::Surface<std::sync::Arc<winit::window::Window>, std::sync::Arc<winit::window::Window>>,
        >,
    ) {
        if cpu_surface.is_some() {
            return;
        }

        match softbuffer::Context::new(std::sync::Arc::clone(window))
            .and_then(|context| softbuffer::Surface::new(&context, std::sync::Arc::clone(window)))
        {
            Ok(surface) => {
                tracing::info!("CPU renderer initialized");
                *cpu_surface = Some(surface);
                self.surface_configured = false;
                self.needs_redraw = true;
            }
            Err(err) => {
                tracing::error!("CPU renderer init failed: {err}");
            }
        }
    }

    /// GPU 渲染一帧
    pub fn render_frame(&mut self, width: u32, height: u32) {
        let mut gpu = self.gpu_renderer.take();
        if let Some(ref mut renderer) = gpu {
            let (fills, glyphs) = self.build_scene(width, height);
            renderer.render_scene(&fills, &self.font_loader, &mut self.glyph_cache, &glyphs);
        }
        self.gpu_renderer = gpu;
    }

    /// CPU 软件渲染一帧
    pub fn render_cpu(
        &mut self,
        width: u32,
        height: u32,
        cpu_surface: &mut Option<
            softbuffer::Surface<std::sync::Arc<winit::window::Window>, std::sync::Arc<winit::window::Window>>,
        >,
    ) {
        use std::num::NonZeroU32;

        let (fills, glyphs) = self.build_scene(width, height);
        let fb = render_scene_to_framebuffer(
            width,
            height,
            1.0,
            &fills,
            &self.font_loader,
            &mut self.glyph_cache,
            &glyphs,
        );
        let Some(surface) = cpu_surface.as_mut() else {
            return;
        };

        let sw = match NonZeroU32::new(fb.width) {
            Some(w) => w,
            None => return,
        };
        let sh = match NonZeroU32::new(fb.height) {
            Some(h) => h,
            None => return,
        };

        if let Err(err) = surface.resize(sw, sh) {
            tracing::error!("CPU surface resize failed: {err}");
            return;
        }

        let mut buffer = match surface.buffer_mut() {
            Ok(b) => b,
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

    /// 构建浏览器 UI 渲染图元（物理像素坐标）
    fn build_scene(&mut self, width: u32, height: u32) -> (Vec<FillPrimitive>, Vec<GlyphDraw>) {
        let s = self.scale_factor;
        let mut fills = Vec::new();
        let mut glyphs = Vec::new();
        let font_size = 14.0 * s;

        // 1. 整体背景
        fills.push(rect_fill(0.0, 0.0, width as f32, height as f32, colors::BACKGROUND));

        // 2. 标签栏背景
        let tab_bar_h = layout::TAB_BAR_HEIGHT * s;
        fills.push(rect_fill(0.0, 0.0, width as f32, tab_bar_h, colors::TAB_BAR_BG));

        // 3. 标签内容（带布局缓存）
        self.render_tabs(&mut fills, &mut glyphs, width, font_size, s);

        // 4. 地址栏背景
        let addr_y = tab_bar_h;
        fills.push(rect_fill(
            0.0,
            addr_y,
            width as f32,
            layout::ADDRESS_BAR_HEIGHT * s,
            colors::TAB_BAR_BG,
        ));

        // 5. 导航按钮
        self.render_nav_buttons(&mut glyphs, addr_y, font_size, s);

        // 6. 地址栏
        self.render_address_bar(&mut fills, &mut glyphs, width, addr_y, font_size, s);

        // 7. 分隔线
        let toolbar_h = layout::TOOLBAR_HEIGHT * s;
        fills.push(rect_fill(0.0, toolbar_h - s, width as f32, s, colors::SEPARATOR));

        // 8. 书签栏
        let bookmarks_bar_y = toolbar_h;
        self.render_bookmarks_bar(&mut fills, &mut glyphs, width, bookmarks_bar_y, s);

        // 9. 页面内容区域
        let chrome_top = toolbar_h + layout::BOOKMARKS_BAR_HEIGHT * s;
        let page_h = height as f32 - chrome_top - layout::STATUS_BAR_HEIGHT * s;
        fills.push(rect_fill(0.0, chrome_top, width as f32, page_h, colors::PAGE_BG));

        // 10. 加载指示器
        if self.shell.active_tab().is_some_and(|t| t.is_loading()) {
            fills.push(rect_fill(
                0.0,
                chrome_top,
                width as f32,
                2.0 * s,
                colors::LOADING_INDICATOR,
            ));
        }

        // 11. 页面内容（含滚动偏移）
        self.render_page_content(&mut fills, &mut glyphs, width, chrome_top, font_size, s);

        // 12. 查找栏（覆盖在页面内容上方）
        if self.shell.find_state().is_active() {
            self.render_find_bar(&mut fills, &mut glyphs, width, chrome_top, font_size, s);
        }

        // 13. 自动补全下拉
        if self.address_bar_focused && !self.autocomplete.suggestions.is_empty() {
            self.render_autocomplete(&mut fills, &mut glyphs, width, font_size, s);
        }

        // 14. 上下文菜单（最上层覆盖）
        if self.context_menu.visible {
            self.render_context_menu(&mut fills, &mut glyphs, s);
        }

        // 15. 下载进度条（有活跃下载时显示在状态栏上方）
        if self.shell.downloads().active_count() > 0 {
            self.render_download_bar(&mut fills, &mut glyphs, width, height, font_size, s);
        }

        // 16. 状态栏
        self.render_status_bar(&mut fills, &mut glyphs, width, height, font_size, s);

        (fills, glyphs)
    }

    /// 渲染标签页
    fn render_tabs(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        font_size: f32,
        s: f32,
    ) {
        let active_id = self.shell.active_tab_id();
        let tab_count = self.shell.tab_count();
        if tab_count == 0 {
            return;
        }

        let new_tab_btn_w = 32.0 * s;
        let available_width = width as f32 - new_tab_btn_w;
        let tab_w = (available_width / tab_count as f32).clamp(layout::TAB_MIN_WIDTH * s, layout::TAB_MAX_WIDTH * s);

        self.tab_layout.clear();
        let mut x = 0.0_f32;

        for tab in self.shell.tabs() {
            let is_active = Some(tab.id()) == active_id;
            let is_hovered = !is_active && {
                let mx = self.mouse_pos.0 as f32;
                let my = self.mouse_pos.1 as f32;
                mx >= x && mx < x + tab_w && my < layout::TAB_BAR_HEIGHT * s
            };

            let tab_bar_h = layout::TAB_BAR_HEIGHT * s;
            let bg = if is_active {
                colors::TAB_ACTIVE_BG
            } else if is_hovered {
                colors::TAB_HOVER_BG
            } else {
                colors::TAB_BAR_BG
            };
            fills.push(rect_fill(x, 0.0, tab_w - s, tab_bar_h, bg));

            if let Some(fid) = self.font_id {
                let label = tab.title().unwrap_or_else(|| tab.url().unwrap_or("New Tab"));
                let max_chars = ((tab_w - 40.0 * s) / (font_size * 0.6)).max(3.0) as usize;
                let truncated: String = label.chars().take(max_chars).collect();
                draw_text(
                    &truncated,
                    x + 10.0 * s,
                    8.0 * s,
                    font_size,
                    colors::TAB_TEXT,
                    fid,
                    glyphs,
                );
            }

            if let Some(fid) = self.font_id {
                let close_x = x + tab_w - 24.0 * s;
                glyphs.push(GlyphDraw {
                    ch: '×',
                    x: close_x,
                    baseline_y: 8.0 * s + font_size,
                    color: colors::TAB_CLOSE,
                    font_id: fid,
                    font_size: font_size * 0.8,
                });
            }

            self.tab_layout.push((tab.id(), x, tab_w));
            x += tab_w;
        }

        // 新建标签按钮 (+)
        if let Some(fid) = self.font_id {
            let btn_x = width as f32 - new_tab_btn_w;
            let tab_bar_h = layout::TAB_BAR_HEIGHT * s;
            let is_hovered = {
                let mx = self.mouse_pos.0 as f32;
                let my = self.mouse_pos.1 as f32;
                mx >= btn_x && my < tab_bar_h
            };
            if is_hovered {
                fills.push(rect_fill(btn_x, 0.0, new_tab_btn_w, tab_bar_h, colors::TAB_HOVER_BG));
            }
            let text_x = btn_x + (new_tab_btn_w - font_size * 0.6) / 2.0;
            draw_text("+", text_x, 8.0 * s, font_size, colors::NEW_TAB_BUTTON, fid, glyphs);
        }
    }

    /// 渲染导航按钮
    fn render_nav_buttons(&mut self, glyphs: &mut Vec<GlyphDraw>, y: f32, font_size: f32, s: f32) {
        if let Some(fid) = self.font_id {
            let baseline_y = y + (layout::ADDRESS_BAR_HEIGHT * s + font_size) / 2.0;
            let x = 8.0 * s;
            let w = layout::NAV_BUTTON_WIDTH * s;

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
        s: f32,
    ) {
        let nav_w = (layout::NAV_BUTTON_WIDTH * 4.0 + 16.0) * s;
        let bar_x = nav_w + layout::ADDRESS_BAR_PADDING * s;
        let bar_w = width as f32 - bar_x - layout::ADDRESS_BAR_PADDING * s;
        let bar_y = y + 4.0 * s;
        let bar_h = layout::ADDRESS_BAR_HEIGHT * s - 8.0 * s;

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
            draw_text(
                &display_text,
                bar_x + 10.0 * s,
                bar_y + 3.0 * s,
                font_size,
                color,
                fid,
                glyphs,
            );

            if self.address_bar_focused {
                let cursor_x = bar_x + 10.0 * s + self.address_bar_text.len() as f32 * font_size * 0.6;
                fills.push(rect_fill(
                    cursor_x,
                    bar_y + 4.0 * s,
                    1.5 * s,
                    bar_h - 8.0 * s,
                    colors::ADDRESS_BAR_TEXT,
                ));
            }
        }
    }

    /// 渲染书签栏
    fn render_bookmarks_bar(
        &self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        y: f32,
        s: f32,
    ) {
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

        let bar_h = layout::BOOKMARKS_BAR_HEIGHT * s;
        fills.push(rect_fill(0.0, y, width as f32, bar_h, colors::BOOKMARKS_BAR_BG));

        let font_size = 12.0 * s;
        let mut bx = 8.0 * s;
        let by = y + 3.0 * s;

        let bookmarks = self.shell.bookmarks();
        for bm in bookmarks.list_root() {
            let label = bm.title();
            let item_w = label.len() as f32 * font_size * 0.6 + 24.0 * s;

            // 悬停效果
            let mx = self.mouse_pos.0 as f32;
            let my = self.mouse_pos.1 as f32;
            if mx >= bx && mx < bx + item_w && my >= y && my < y + bar_h {
                fills.push(rect_fill(bx, y, item_w, bar_h, colors::BOOKMARKS_BAR_HOVER_BG));
            }

            // 书签图标
            draw_text("★", bx, by, font_size, colors::BOOKMARKS_BAR_ICON, fid, glyphs);
            // 标签文本
            draw_text(
                label,
                bx + 14.0 * s,
                by,
                font_size,
                colors::BOOKMARKS_BAR_TEXT,
                fid,
                glyphs,
            );

            bx += item_w + 8.0 * s;
            if bx > width as f32 - 40.0 * s {
                break;
            }
        }
    }

    /// 渲染页面内容
    fn render_page_content(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        _width: u32,
        page_y: f32,
        font_size: f32,
        s: f32,
    ) {
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

        let content_y_offset = if self.shell.find_state().is_active() {
            layout::FIND_BAR_HEIGHT * s
        } else {
            0.0
        };

        let (title, url, is_loading) = match self.shell.active_tab() {
            Some(tab) => (
                tab.title().unwrap_or("").to_string(),
                tab.url().unwrap_or("").to_string(),
                tab.is_loading(),
            ),
            None => return,
        };

        let mut y = page_y + content_y_offset;

        // 获取当前标签的滚动偏移
        let tab_id = self.shell.active_tab_id().unwrap();
        let scroll_y = self.scroll_offset.get(&tab_id).copied().unwrap_or(0.0);

        if !is_loading && self.render_active_webview(fills, glyphs, y, fid, scroll_y) {
            return;
        }

        if !title.is_empty() {
            draw_text(
                &title,
                20.0 * s,
                y + 20.0 * s,
                24.0 * s,
                colors::PAGE_TITLE,
                fid,
                glyphs,
            );
            y += 52.0 * s;
        }

        if !url.is_empty() {
            draw_text(&url, 20.0 * s, y, 12.0 * s, colors::PAGE_URL, fid, glyphs);
            y += 28.0 * s;
        }

        if is_loading {
            draw_text("Loading...", 20.0 * s, y, font_size, colors::PAGE_HINT, fid, glyphs);
        } else if title.is_empty() && url.is_empty() {
            draw_text(
                "Welcome to ZeroBrowser — Press L to focus address bar, T for new tab",
                20.0 * s,
                y,
                font_size,
                colors::PAGE_HINT,
                fid,
                glyphs,
            );
        }
    }

    /// 渲染活跃 WebView 的页面图元。
    fn render_active_webview(
        &self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        y_offset: f32,
        fallback_font_id: u32,
        scroll_y: f32,
    ) -> bool {
        let tab_id = match self.shell.active_tab_id() {
            Some(id) => id,
            None => return false,
        };

        let primitives = match self
            .webviews
            .get(&tab_id)
            .and_then(|wv| wv.last_render())
            .map(|render| &render.primitives)
        {
            Some(primitives) => primitives,
            None => return false,
        };

        append_webview_primitives(
            primitives,
            fills,
            glyphs,
            0.0,
            y_offset - scroll_y,
            fallback_font_id,
            1.0,
        )
    }

    /// 渲染查找栏
    fn render_find_bar(
        &self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        chrome_top: f32,
        font_size: f32,
        s: f32,
    ) {
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

        let y = chrome_top;
        let bar_w = 320.0 * s;
        let bar_x = width as f32 - bar_w - 10.0 * s;

        fills.push(rect_fill(
            bar_x,
            y,
            bar_w,
            layout::FIND_BAR_HEIGHT * s,
            colors::FIND_BAR_BG,
        ));

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
        draw_text(
            &display,
            bar_x + 10.0 * s,
            y + 5.0 * s,
            font_size,
            text_color,
            fid,
            glyphs,
        );

        let find_state = self.shell.find_state();
        if find_state.total_matches() > 0 {
            let match_text = format!("{}/{}", find_state.current_match(), find_state.total_matches());
            let match_x = bar_x + bar_w - 130.0 * s;
            draw_text(
                &match_text,
                match_x,
                y + 5.0 * s,
                font_size,
                colors::FIND_MATCH_TEXT,
                fid,
                glyphs,
            );
        } else if !self.find_input.is_empty() {
            let no_match_x = bar_x + bar_w - 130.0 * s;
            draw_text(
                "No matches",
                no_match_x,
                y + 5.0 * s,
                font_size,
                colors::FIND_MATCH_TEXT,
                fid,
                glyphs,
            );
        }

        let btn_y = y + 5.0 * s;
        let prev_x = bar_x + bar_w - 100.0 * s;
        let next_x = bar_x + bar_w - 70.0 * s;
        let close_x = bar_x + bar_w - 40.0 * s;
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
        s: f32,
    ) {
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

        let nav_w = (layout::NAV_BUTTON_WIDTH * 4.0 + 16.0) * s;
        let bar_x = nav_w + layout::ADDRESS_BAR_PADDING * s;
        let bar_w = width as f32 - bar_x - layout::ADDRESS_BAR_PADDING * s;
        let dropdown_y = (layout::TAB_BAR_HEIGHT + layout::ADDRESS_BAR_HEIGHT) * s;

        let visible_count = self
            .autocomplete
            .suggestions
            .len()
            .min(layout::AUTOCOMPLETE_MAX_VISIBLE);
        let row_h = layout::AUTOCOMPLETE_ROW_HEIGHT * s;
        let dropdown_h = visible_count as f32 * row_h;

        fills.push(rect_fill(bar_x, dropdown_y, bar_w, dropdown_h, colors::AUTOCOMPLETE_BG));

        for (i, sug) in self.autocomplete.suggestions.iter().take(visible_count).enumerate() {
            let row_y = dropdown_y + i as f32 * row_h;
            let is_hovered = self.autocomplete.hovered_index == Some(i);

            if is_hovered {
                fills.push(rect_fill(bar_x, row_y, bar_w, row_h, colors::AUTOCOMPLETE_HOVER_BG));
            }

            let source_label = match sug.source() {
                SuggestionSource::Bookmark => "★",
                SuggestionSource::History => "🕐",
            };
            let text_x = bar_x + 10.0 * s;
            draw_text(
                source_label,
                text_x,
                row_y + 5.0 * s,
                font_size * 0.85,
                if sug.source() == SuggestionSource::Bookmark {
                    colors::AUTOCOMPLETE_BOOKMARK
                } else {
                    colors::AUTOCOMPLETE_URL
                },
                fid,
                glyphs,
            );

            let title = sug.title();
            let max_title_chars = ((bar_w - 180.0 * s) / (font_size * 0.6)).max(10.0) as usize;
            let truncated_title: String = title.chars().take(max_title_chars).collect();
            draw_text(
                &truncated_title,
                text_x + 24.0 * s,
                row_y + 5.0 * s,
                font_size * 0.85,
                colors::AUTOCOMPLETE_TEXT,
                fid,
                glyphs,
            );

            let url = sug.url();
            let url_x = bar_x + bar_w - 10.0 * s;
            let max_url_chars = ((bar_w * 0.4) / (font_size * 0.5)).max(8.0) as usize;
            let truncated_url: String = url.chars().take(max_url_chars).collect();
            let url_display_width = truncated_url.len() as f32 * font_size * 0.5;
            draw_text(
                &truncated_url,
                url_x - url_display_width,
                row_y + 5.0 * s,
                font_size * 0.75,
                colors::AUTOCOMPLETE_URL,
                fid,
                glyphs,
            );
        }

        fills.push(rect_fill(bar_x, dropdown_y + dropdown_h, bar_w, s, colors::SEPARATOR));
    }

    /// 渲染右键上下文菜单
    fn render_context_menu(&self, fills: &mut Vec<FillPrimitive>, glyphs: &mut Vec<GlyphDraw>, s: f32) {
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

        let menu_x = self.context_menu.x;
        let menu_y = self.context_menu.y;
        let row_h = 28.0 * s;
        let menu_w = 200.0 * s;
        let menu_h = self.context_menu.items.len() as f32 * row_h;
        let font_size = 13.0 * s;

        // 菜单背景
        fills.push(rect_fill(menu_x, menu_y, menu_w, menu_h, colors::CONTEXT_MENU_BG));

        // 菜单边框
        let border_w = 1.0 * s;
        fills.push(rect_fill(
            menu_x,
            menu_y,
            menu_w,
            border_w,
            colors::CONTEXT_MENU_SEPARATOR,
        ));
        fills.push(rect_fill(
            menu_x,
            menu_y + menu_h - border_w,
            menu_w,
            border_w,
            colors::CONTEXT_MENU_SEPARATOR,
        ));
        fills.push(rect_fill(
            menu_x,
            menu_y,
            border_w,
            menu_h,
            colors::CONTEXT_MENU_SEPARATOR,
        ));
        fills.push(rect_fill(
            menu_x + menu_w - border_w,
            menu_y,
            border_w,
            menu_h,
            colors::CONTEXT_MENU_SEPARATOR,
        ));

        for (i, label) in self.context_menu.items.iter().enumerate() {
            let row_y = menu_y + i as f32 * row_h;
            let is_hovered = self.context_menu.hovered_index == Some(i);

            if is_hovered {
                fills.push(rect_fill(
                    menu_x + border_w,
                    row_y,
                    menu_w - 2.0 * border_w,
                    row_h,
                    colors::CONTEXT_MENU_HOVER_BG,
                ));
            }

            // 分隔线项
            if label == "---" {
                let sep_y = row_y + row_h / 2.0;
                fills.push(rect_fill(
                    menu_x + 12.0 * s,
                    sep_y,
                    menu_w - 24.0 * s,
                    border_w,
                    colors::CONTEXT_MENU_SEPARATOR,
                ));
                continue;
            }

            draw_text(
                label,
                menu_x + 16.0 * s,
                row_y + 6.0 * s,
                font_size,
                colors::CONTEXT_MENU_TEXT,
                fid,
                glyphs,
            );
        }
    }

    /// 渲染状态栏
    fn render_status_bar(
        &mut self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        height: u32,
        _font_size: f32,
        s: f32,
    ) {
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

        let status_h = layout::STATUS_BAR_HEIGHT * s;
        let status_y = height as f32 - status_h;

        fills.push(rect_fill(0.0, status_y, width as f32, status_h, colors::BACKGROUND));
        fills.push(rect_fill(0.0, status_y, width as f32, s, colors::SEPARATOR));

        let zoom = self.shell.zoom();
        if (zoom - 1.0).abs() > f32::EPSILON {
            let zoom_text = format!("{}%", (zoom * 100.0) as u32);
            draw_text(
                &zoom_text,
                10.0 * s,
                status_y + 3.0 * s,
                11.0 * s,
                colors::STATUS_TEXT,
                fid,
                glyphs,
            );
        }

        let tab_count = self.shell.tab_count();
        let tabs_text = format!("Tabs: {tab_count}");
        let tabs_width = tabs_text.len() as f32 * 11.0 * s * 0.6;
        draw_text(
            &tabs_text,
            width as f32 - tabs_width - 10.0 * s,
            status_y + 3.0 * s,
            11.0 * s,
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

    /// 渲染下载进度条（状态栏上方）
    fn render_download_bar(
        &self,
        fills: &mut Vec<FillPrimitive>,
        glyphs: &mut Vec<GlyphDraw>,
        width: u32,
        height: u32,
        _font_size: f32,
        s: f32,
    ) {
        let fid = match self.font_id {
            Some(id) => id,
            None => return,
        };

        let bar_h = layout::DOWNLOAD_BAR_HEIGHT * s;
        let status_h = layout::STATUS_BAR_HEIGHT * s;
        let bar_y = height as f32 - status_h - bar_h;

        // 背景
        fills.push(rect_fill(0.0, bar_y, width as f32, bar_h, colors::DOWNLOAD_BAR_BG));

        // 显示第一个活跃下载的信息
        let downloads = self.shell.downloads();
        let active: Vec<_> = downloads.iter().filter(|d| d.is_active()).collect();
        if let Some(dl) = active.first() {
            let font_size = 11.0 * s;

            // 文件名
            let name_text = dl.filename();
            draw_text(
                name_text,
                10.0 * s,
                bar_y + 6.0 * s,
                font_size,
                colors::DOWNLOAD_BAR_TEXT,
                fid,
                glyphs,
            );

            // 进度条
            let progress = dl.progress();
            let bar_width = 120.0 * s;
            let bar_start_x = width as f32 - bar_width - 80.0 * s;
            let bar_top = bar_y + 8.0 * s;
            let bar_inner_h = 6.0 * s;

            // 进度条背景
            fills.push(rect_fill(
                bar_start_x,
                bar_top,
                bar_width,
                bar_inner_h,
                colors::SEPARATOR,
            ));
            // 进度条填充
            fills.push(rect_fill(
                bar_start_x,
                bar_top,
                bar_width * progress,
                bar_inner_h,
                colors::DOWNLOAD_BAR_FILL,
            ));

            // 百分比文字
            let pct_text = format!("{:.0}%", progress * 100.0);
            draw_text(
                &pct_text,
                bar_start_x + bar_width + 8.0 * s,
                bar_y + 6.0 * s,
                font_size,
                colors::DOWNLOAD_BAR_TEXT,
                fid,
                glyphs,
            );
        }
    }
}

// --- 工具函数 ---

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

/// 将 WebView 输出的基础图元追加到浏览器场景。
pub fn append_webview_primitives(
    primitives: &RenderPrimitives,
    fills: &mut Vec<FillPrimitive>,
    glyphs: &mut Vec<GlyphDraw>,
    x_offset: f32,
    y_offset: f32,
    fallback_font_id: u32,
    s: f32,
) -> bool {
    let fill_start = fills.len();
    let glyph_start = glyphs.len();

    for fill in &primitives.fills {
        let mut translated = fill.clone();
        translated.rect.origin.x = fill.rect.origin.x * s + x_offset;
        translated.rect.origin.y = fill.rect.origin.y * s + y_offset;
        translated.rect.size.width *= s;
        translated.rect.size.height *= s;
        fills.push(translated);
    }

    for glyph in &primitives.glyphs {
        let Some(ch) = char::from_u32(glyph.glyph_id) else {
            continue;
        };
        if ch == '\0' {
            continue;
        }
        glyphs.push(GlyphDraw {
            ch,
            x: glyph.x * s + x_offset,
            baseline_y: glyph.y * s + y_offset,
            color: glyph.color,
            font_id: if glyph.font_id.0 == 0 {
                fallback_font_id
            } else {
                glyph.font_id.0
            },
            font_size: glyph.font_size * s,
        });
    }

    fills.len() > fill_start || glyphs.len() > glyph_start
}

/// URL 规范化 — 支持 URL 和搜索引擎回退
pub fn normalize_url(input: &str, shell: &BrowserShell) -> String {
    if input.starts_with("http://") || input.starts_with("https://") {
        return input.to_string();
    }
    if input.starts_with("ftp://") || input.starts_with("file://") || input.starts_with("data:") {
        return input.to_string();
    }
    if input.contains('.') && !input.contains(' ') {
        return format!("https://{input}");
    }
    shell.settings().search(input)
}

/// 尝试加载系统字体
pub fn load_system_font(font_loader: &mut FontLoader) -> Option<u32> {
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
