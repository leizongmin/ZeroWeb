// 演示流（gui_smoke / tab_smoke）运行时状态读取面。
// 从 app.rs 通过 include! 引入模块作用域——私有字段（shell/tabs/address_bar/tab_layout）
// 仅在 app 模块树内可见，故经此处暴露最小只读访问器；与 `#[cfg(test)]` 的 *_for_test
// 族不同，这些方法不带 cfg 门，release 二进制的真实窗口演示流同样可用。

impl BrowserApp {
    /// 演示流用：标签条顺序的全部标签 id（与 tab_layout 同序）。
    pub fn smoke_tab_ids(&self) -> Vec<zero_browser_shell::TabId> {
        self.shell.tabs().map(|tab| tab.id()).collect()
    }

    /// 演示流用：当前活动标签 id。
    pub fn smoke_active_tab_id(&self) -> Option<zero_browser_shell::TabId> {
        self.shell.active_tab_id()
    }

    /// 演示流用：标签最近快照 URL。
    pub fn smoke_tab_url(&self, tab_id: zero_browser_shell::TabId) -> Option<String> {
        self.tabs.page_url(tab_id)
    }

    /// 演示流用：地址栏当前文本。
    pub fn smoke_address_bar_text(&self) -> &str {
        self.address_bar.text()
    }

    /// 演示流用：标签在标签条中的 `(x, width)` 布局（帧渲染时随 build_scene 刷新）。
    pub fn smoke_tab_rect(&self, tab_id: zero_browser_shell::TabId) -> Option<(f32, f32)> {
        self.tab_layout
            .iter()
            .find(|(id, _, _)| *id == tab_id)
            .map(|&(_, x, w)| (x, w))
    }

    /// 演示流用：标签最近快照的导航 epoch（刷新后前进，用于断言重载确实发生）。
    pub fn smoke_navigation_epoch(&self, tab_id: zero_browser_shell::TabId) -> u64 {
        self.tabs.smoke_navigation_epoch(tab_id)
    }

    /// 演示流用：当前补全建议 URL 列表（建议弹出的状态源）。
    pub fn smoke_autocomplete_urls(&self) -> Vec<String> {
        self.autocomplete
            .suggestions
            .iter()
            .map(|s| s.url().to_string())
            .collect()
    }

    /// 演示流用：当前补全高亮位（hovered 优先，其次键盘选中）。
    pub fn smoke_autocomplete_highlight(&self) -> Option<usize> {
        self.autocomplete.highlight_index()
    }

    /// 演示流用：下载条目快照 `(url, filename, state, downloaded, total)`。
    pub fn smoke_downloads(&self) -> Vec<(String, String, String, u64, Option<u64>)> {
        self.shell
            .downloads()
            .iter()
            .map(|d| {
                (
                    d.url().to_string(),
                    d.filename().to_string(),
                    format!("{:?}", d.state()),
                    d.downloaded_bytes(),
                    d.total_bytes(),
                )
            })
            .collect()
    }

    /// 演示流用：下载目标目录（与落盘路径同源）。
    pub fn smoke_download_dir(&self) -> String {
        self.download_target_dir().display().to_string()
    }

    /// 演示流用：「在文件夹中显示」动作最近打开的目录。
    pub fn smoke_last_opened_download_dir(&self) -> Option<String> {
        self.last_opened_download_dir.clone()
    }

    /// 演示流用：下载面板是否可见（含活动下载自动展开语义）。
    pub fn smoke_download_panel_visible(&self) -> bool {
        self.should_show_download_panel()
    }

    /// 演示流用：上下文菜单是否可见。
    pub fn smoke_context_menu_visible(&self) -> bool {
        self.context_menu.visible
    }

    /// 演示流用：上下文菜单原点 `(x, y)`（可见时）。
    pub fn smoke_context_menu_origin(&self) -> Option<(f32, f32)> {
        self.context_menu
            .visible
            .then_some((self.context_menu.x, self.context_menu.y))
    }

    /// 演示流用：上下文菜单项 id 序列（与行序一致，分隔符以 "-" 占位）。
    pub fn smoke_context_menu_item_ids(&self) -> Vec<String> {
        self.context_menu
            .items
            .iter()
            .map(|item| {
                if item.is_separator() {
                    "-".to_string()
                } else {
                    item.id().to_string()
                }
            })
            .collect()
    }

    /// 演示流用：系统剪贴板文本（arboard；X11 selection 同进程读回）。
    pub fn smoke_clipboard_text(&self) -> Option<String> {
        crate::clipboard::read_text()
    }

    /// 演示流用：当前页面缩放（1.0 = 100%）。
    pub fn smoke_page_zoom(&self) -> f32 {
        self.shell.zoom()
    }

    /// 演示流用：页面查找状态 `(active, query, current, total)`。
    pub fn smoke_find_state(&self) -> (bool, String, usize, usize) {
        let state = self.shell.find_state();
        (
            state.is_active(),
            state.query().to_string(),
            state.current_match(),
            state.total_matches(),
        )
    }

    /// 演示流用：活动标签页垂直滚动偏移（物理像素，滚动定位断言面）。
    pub fn smoke_scroll_y(&self) -> f32 {
        self.shell
            .active_tab_id()
            .map(|tab| self.tab_scroll_state(tab).y)
            .unwrap_or(0.0)
    }

    /// 演示流用：根书签列表 `(title, url)` 快照。
    pub fn smoke_bookmarks(&self) -> Vec<(String, String)> {
        self.shell
            .bookmarks()
            .list_root()
            .iter()
            .map(|bm| (bm.title().to_string(), bm.url().to_string()))
            .collect()
    }

    /// 演示流用：历史条目数。
    pub fn smoke_history_len(&self) -> usize {
        self.shell.history().len()
    }

    /// 演示流用：历史条目 URL 列表。
    pub fn smoke_history_urls(&self) -> Vec<String> {
        self.shell.history().iter().map(|e| e.url().to_string()).collect()
    }

    /// 演示流用：设置快照 `(search_engine, do_not_track, home_url, show_bookmarks_bar)`。
    pub fn smoke_settings(&self) -> (String, bool, String, bool) {
        let settings = self.shell.settings();
        (
            format!("{:?}", settings.search_engine),
            settings.do_not_track,
            settings.home_url.clone(),
            settings.show_bookmarks_bar,
        )
    }

    /// 演示流用：工具栏浏览器菜单（三点）按钮矩形（物理像素）。
    pub fn smoke_toolbar_menu_button_rect(&self) -> (f32, f32, f32, f32) {
        self.toolbar_menu_button_rect()
    }

    /// 演示流用：书签栏可见性（= 设置开关 && 根书签非空）。
    pub fn smoke_bookmarks_bar_visible(&self) -> bool {
        self.bookmarks_bar_visible()
    }
}
