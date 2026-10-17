// 图片/页面“另存为”消费（start_image_save / save_image_fetch_result /
// save_page_as / save_page_fetch_result）。
// 从 app.rs 拆分以控制单文件体积，经 `include!` 文本包含进 app.rs 模块作用域，
// 与 app_input_context_menus.rs / app_page_fonts.rs 同模式。

impl BrowserApp {
    /// 右键“将图片另存为”：后台线程拉取图片原始字节，完成后走下载管线落盘。
    fn start_image_save(&mut self, tab_id: Option<TabId>, url: &str) {
        let Some(tab_id) = tab_id.or_else(|| self.shell.active_tab_id()) else {
            return;
        };
        self.image_save_fetch.request(tab_id, url);
        self.needs_redraw = true;
    }

    /// 图片另存为抓取完成：成功走 attachment 下载管线，失败记一条失败下载项
    ///（与写盘失败同口径，用户在下载面板可见，不静默吞掉）。
    fn save_image_fetch_result(&mut self, tab_id: TabId, url: String, result: Result<Vec<u8>, String>) {
        let filename = crate::process_backend::parse_attachment_filename("", &url);
        match result {
            Ok(body) => {
                self.save_attachment_download(crate::process_backend::PendingAttachmentDownload {
                    tab_id,
                    url,
                    filename,
                    body,
                });
            }
            Err(error) => {
                tracing::warn!("save_image fetch failed url={url} err={error}");
                let id = self.shell.downloads_mut().start_download(&url, &filename);
                self.shell.downloads_mut().mark_failed(id);
                self.download_panel_open = true;
                self.needs_redraw = true;
            }
        }
    }

    /// 页面右键“另存为”（T51）：对当前标签 URL 发起后台抓取（与图片另存为同一
    /// 抓取基础设施：独立线程 + 30s 超时 + 在途去重），完成后经下载管线落盘。
    /// URL 取 shell 标签模型（与菜单可用性 page_inspectable 判定同源）；
    /// zero:// / about:blank 等无“可保存源”的页面已在菜单侧禁用，此处兜底跳过。
    fn save_page_as(&mut self, tab_id: TabId) {
        let Some(url) = self
            .shell
            .tab(tab_id)
            .and_then(|tab| tab.url())
            .map(str::to_string)
            .filter(|url| !url.is_empty() && url.as_str() != "about:blank")
        else {
            return;
        };
        self.page_save_fetch.request(tab_id, &url);
        self.needs_redraw = true;
    }

    /// 页面另存为抓取完成：成功走 attachment 下载管线（文件名按页面规则
    /// save_page_filename），失败记失败下载项（与图片另存为同口径）。
    fn save_page_fetch_result(&mut self, tab_id: TabId, url: String, result: Result<Vec<u8>, String>) {
        let filename = pages::save_page_filename(&url);
        match result {
            Ok(body) => {
                self.save_attachment_download(crate::process_backend::PendingAttachmentDownload {
                    tab_id,
                    url,
                    filename,
                    body,
                });
            }
            Err(error) => {
                tracing::warn!("save_page fetch failed url={url} err={error}");
                let id = self.shell.downloads_mut().start_download(&url, &filename);
                self.shell.downloads_mut().mark_failed(id);
                self.download_panel_open = true;
                self.needs_redraw = true;
            }
        }
    }
}
