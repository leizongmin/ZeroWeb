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
}
