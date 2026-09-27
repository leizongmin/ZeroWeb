//! M4 数据面真实窗口 smoke（收藏/历史/设置/主页）的分步状态机。
//!
//! 与既有 smoke 同模式：真实输入路径驱动——工具栏菜单按钮点击加书签、书签栏
//! 条目点击打开与右键删除、地址栏键入内部设置/历史 URL（真实导航路径）执行
//! 设置变更与历史清除、Alt+Home 验证主页导航。状态断言经 app_smoke_state
//! 只读面；流程结束恢复用户设置（自清洁，不留痕）。

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use zero_browser_shell::SearchEngine;
use zero_render_foundation::surface::FrameBuffer;

use crate::app::BrowserApp;
use crate::layout;
use crate::smoke_capture::{self, PixelRegion};

const STEP_TIMEOUT: Duration = Duration::from_secs(60);
const PAGE_PATH: &str = "/page.html";

/// 数据面 smoke 配置。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataSmokeConfig {
    /// 本地 fixture 服务 origin（如 http://127.0.0.1:PORT）。
    pub base: String,
    /// 分步截图输出目录。
    pub output_dir: PathBuf,
}

impl DataSmokeConfig {
    /// 创建并校验 smoke 配置。
    pub fn new(base: String, output_dir: PathBuf) -> Result<Self, String> {
        if !base.starts_with("http://") && !base.starts_with("https://") {
            return Err("--data-smoke-base requires an http:// or https:// origin".to_string());
        }
        if output_dir.as_os_str().is_empty() {
            return Err("--data-smoke-dir requires a directory path".to_string());
        }
        Ok(Self {
            base: base.trim_end_matches('/').to_string(),
            output_dir,
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Pending,
    /// 数据页加载（历史/书签的种子内容）。
    WaitingSeedLoad,
    /// 浏览器菜单点击「添加书签」：根书签 + 栏可见断言。
    WaitingBookmarkAdded,
    /// 书签栏条目点击：同 URL 重载（epoch 前进）。
    WaitingBarOpen,
    /// 书签栏条目右键 → 删除：根书签清空断言。
    WaitingBookmarkDeleted,
    /// zero://history 呈现断言（历史有记录）。
    WaitingHistoryPage,
    /// zero://history/clear 后历史清空断言 + 空态截图。
    WaitingHistoryCleared,
    /// 设置搜索引擎切 DuckDuckGo 断言。
    WaitingSearchEngine,
    /// 隐私开关 do_not_track 断言。
    WaitingDnt,
    /// 设置 home_url 为本地 fixture 断言。
    WaitingHomeSet,
    /// Alt+Home 主页导航落地断言（主页按钮收口）。
    WaitingHomeNav,
    Complete,
}

/// 数据面 smoke 执行状态。
pub struct DataSmoke {
    config: DataSmokeConfig,
    stage: Stage,
    deadline: Instant,
    pre_bar_open_epoch: u64,
}

impl DataSmoke {
    /// 创建尚未启动的 smoke。
    pub fn new(config: DataSmokeConfig) -> Self {
        Self {
            config,
            stage: Stage::Pending,
            deadline: Instant::now() + STEP_TIMEOUT,
            pre_bar_open_epoch: 0,
        }
    }

    /// 在浏览器窗口与默认标签页就绪后开始流程。
    pub fn start(&mut self, app: &mut BrowserApp) {
        if self.stage != Stage::Pending {
            return;
        }
        tracing::info!("DATA_SMOKE_START base={}", self.config.base);
        app.navigate_to(&self.config.url(PAGE_PATH));
        self.advance(Stage::WaitingSeedLoad);
    }

    /// 检查当前步骤是否超过内部墙钟上限。
    pub fn check_timeout(&self) -> Result<(), String> {
        if self.stage != Stage::Complete && Instant::now() > self.deadline {
            return Err(format!("Data smoke timed out in stage {:?}", self.stage));
        }
        Ok(())
    }

    /// 消费一个已真实呈现的 compositor/GPU framebuffer。
    ///
    /// 返回 `true` 表示全部步骤完成，调用方应正常关闭子进程并退出。
    pub fn on_presented_frame(
        &mut self,
        app: &mut BrowserApp,
        framebuffer: &FrameBuffer,
        source: &str,
    ) -> Result<bool, String> {
        if self.stage == Stage::Pending || self.stage == Stage::Complete || app.any_tab_loading() {
            // 未完成前自驱动下一帧（页面静止/加载期事件循环不自发重绘）。
            app.needs_redraw = true;
            return Ok(self.stage == Stage::Complete);
        }
        app.needs_redraw = true;
        let active = app.smoke_active_tab_id().ok_or_else(|| "no active tab".to_string())?;

        match self.stage {
            Stage::WaitingSeedLoad => {
                let expected = self.config.url(PAGE_PATH);
                let url = app
                    .smoke_tab_url(active)
                    .ok_or_else(|| "seed page has no snapshot url".to_string())?;
                if url != expected {
                    return Err(format!("seed page url is {url}, expected {expected}"));
                }
                tracing::info!("DATA_SMOKE_STEP step=seed-load status=passed");

                // Ctrl+D 为此标签页添加书签（标准快捷键，真实键盘路径）。
                // show_bookmarks_bar 默认 true：书签入账后栏即呈现（无需开关）。
                app.handle_key("Control", true, None);
                app.handle_key("d", true, None);
                app.handle_key("d", false, None);
                app.handle_key("Control", false, None);
                tracing::info!("DATA_SMOKE_ACTION action=ctrl-d-add-bookmark status=executed");
                self.advance(Stage::WaitingBookmarkAdded);
            }
            Stage::WaitingBookmarkAdded => {
                let bookmarks = app.smoke_bookmarks();
                let expected = self.config.url(PAGE_PATH);
                if bookmarks.len() != 1 || bookmarks[0].1 != expected {
                    return Ok(false); // 书签入账传播中
                }
                if !app.smoke_bookmarks_bar_visible() {
                    return Err("bookmarks bar not visible after add".to_string());
                }
                self.capture_step(app, framebuffer, source, "01-bookmark-added.png", "bookmark-added")?;
                tracing::info!(
                    "DATA_SMOKE_STEP step=bookmark-added status=passed title={:?}",
                    bookmarks[0].0
                );

                // 书签栏条目点击（真实点击，条目几何与渲染同源）→ 同 URL 重载。
                self.pre_bar_open_epoch = app.smoke_navigation_epoch(active);
                let (item_x, y) = self.bookmark_bar_first_item_center(app);
                app.handle_mouse_click(item_x, y, true, "Left");
                app.handle_mouse_click(item_x, y, false, "Left");
                tracing::info!("DATA_SMOKE_ACTION action=bar-item-open status=executed");
                self.advance(Stage::WaitingBarOpen);
            }
            Stage::WaitingBarOpen => {
                let epoch = app.smoke_navigation_epoch(active);
                if epoch <= self.pre_bar_open_epoch {
                    return Ok(false); // 重载尚未落地
                }
                tracing::info!("DATA_SMOKE_STEP step=bar-open status=passed epoch={epoch}");

                // 书签栏条目右键 → 删除行 → 根书签清空。
                let (item_x, y) = self.bookmark_bar_first_item_center(app);
                app.handle_mouse_click(item_x, y, true, "Right");
                if !app.smoke_context_menu_visible() {
                    return Err("bookmark bar right click did not open the menu".to_string());
                }
                let ids = app.smoke_context_menu_item_ids();
                if !ids.iter().any(|id| id == "bookmark_delete") {
                    return Err(format!("bookmark menu misses delete: {ids:?}"));
                }
                self.click_menu_item(app, "bookmark_delete")?;
                tracing::info!("DATA_SMOKE_ACTION action=menu-bookmark-delete status=executed");
                self.advance(Stage::WaitingBookmarkDeleted);
            }
            Stage::WaitingBookmarkDeleted => {
                if !app.smoke_bookmarks().is_empty() {
                    return Ok(false); // 删除传播中
                }
                tracing::info!("DATA_SMOKE_STEP step=bookmark-deleted status=passed");

                // 历史页呈现（此时流程已产生多条历史）。
                self.type_internal_url(app, "zero://history");
                self.advance(Stage::WaitingHistoryPage);
            }
            Stage::WaitingHistoryPage => {
                let url = app.smoke_tab_url(active).unwrap_or_default();
                if url != "zero://history" {
                    return Ok(false); // 内部页加载中
                }
                let history_len = app.smoke_history_len();
                if history_len == 0 {
                    return Err("history empty before clear".to_string());
                }
                self.capture_step(app, framebuffer, source, "01-history-page.png", "history-page")?;
                tracing::info!("DATA_SMOKE_STEP step=history-page status=passed entries={history_len}");

                self.type_internal_url(app, "zero://history/clear");
                self.advance(Stage::WaitingHistoryCleared);
            }
            Stage::WaitingHistoryCleared => {
                // 清除后 zero://history 自身会被重新记录（内部页也走 on_page_loaded），
                // 断言锚 = fixture 条目（127.0.0.1）全部消失，而非零长度。
                let fixture_left = app
                    .smoke_history_urls()
                    .iter()
                    .filter(|url| url.starts_with(&self.config.base))
                    .count();
                if fixture_left > 0 {
                    return Ok(false); // 清除传播中
                }
                self.capture_step(app, framebuffer, source, "02-history-cleared.png", "history-cleared")?;
                tracing::info!(
                    "DATA_SMOKE_STEP step=history-cleared status=passed fixture_entries_left={fixture_left} len={}",
                    app.smoke_history_len()
                );

                self.type_internal_url(app, "zero://settings/set/search_engine/DuckDuckGo");
                self.advance(Stage::WaitingSearchEngine);
            }
            Stage::WaitingSearchEngine => {
                let (engine, _dnt, _home, _bar) = app.smoke_settings();
                if !engine.contains("DuckDuckGo") {
                    return Ok(false); // 设置传播中
                }
                tracing::info!("DATA_SMOKE_STEP step=search-engine status=passed engine={engine}");

                self.type_internal_url(app, "zero://settings/toggle/do_not_track");
                self.advance(Stage::WaitingDnt);
            }
            Stage::WaitingDnt => {
                let (_engine, dnt, _home, _bar) = app.smoke_settings();
                if !dnt {
                    return Ok(false); // 开关传播中
                }
                tracing::info!("DATA_SMOKE_STEP step=do-not-track status=passed");

                let home = format!(
                    "zero://settings/set/home_url/{}",
                    urlencode(&self.config.url(PAGE_PATH))
                );
                self.type_internal_url(app, &home);
                self.advance(Stage::WaitingHomeSet);
            }
            Stage::WaitingHomeSet => {
                let (_engine, _dnt, home, _bar) = app.smoke_settings();
                let expected = self.config.url(PAGE_PATH);
                if home != expected {
                    return Ok(false); // 设置传播中
                }
                tracing::info!("DATA_SMOKE_STEP step=home-set status=passed home={home}");

                // Alt+Home：主页导航（DC-2 主页按钮项收口）。
                app.handle_key("Alt", true, None);
                app.handle_key("Home", true, None);
                app.handle_key("Home", false, None);
                app.handle_key("Alt", false, None);
                tracing::info!("DATA_SMOKE_ACTION action=alt-home status=executed");
                self.advance(Stage::WaitingHomeNav);
            }
            Stage::WaitingHomeNav => {
                let expected = self.config.url(PAGE_PATH);
                let url = app.smoke_tab_url(active).unwrap_or_default();
                if url != expected {
                    return Ok(false); // 主页导航尚未落地
                }
                self.capture_step(app, framebuffer, source, "03-home-nav.png", "home-nav")?;
                tracing::info!("DATA_SMOKE_STEP step=home-nav status=passed url={url}");

                // 恢复用户设置（自清洁：引擎/隐私/主页/书签栏开关回默认，不留痕）。
                app.shell.apply_settings(|settings| {
                    settings.search_engine = SearchEngine::Google;
                    settings.do_not_track = false;
                    settings.home_url = "https://example.com".to_string();
                    settings.show_bookmarks_bar = false;
                });
                tracing::info!(
                    "DATA_SMOKE_COMPLETE base={} steps=bookmark,history,settings,home",
                    self.config.base
                );
                self.stage = Stage::Complete;
                return Ok(true);
            }
            // 函数入口已对 Pending/Complete 提前返回，此处仅为穷尽性。
            Stage::Pending | Stage::Complete => {}
        }

        Ok(false)
    }

    fn advance(&mut self, stage: Stage) {
        self.stage = stage;
        self.deadline = Instant::now() + STEP_TIMEOUT;
    }

    /// Ctrl+L 聚焦地址栏 + 键入内部 URL + Enter（真实导航路径）。
    fn type_internal_url(&self, app: &mut BrowserApp, url: &str) {
        app.handle_key("Control", true, None);
        app.handle_key("l", true, None);
        app.handle_key("l", false, None);
        app.handle_key("Control", false, None);
        for ch in url.chars() {
            let key = ch.to_string();
            app.handle_key(&key, true, None);
            app.handle_key(&key, false, None);
        }
        app.handle_key("Enter", true, None);
        app.handle_key("Enter", false, None);
        app.needs_redraw = true;
    }

    /// 点击上下文菜单指定 id 的行（行几何与渲染/命中测试同源累积）。
    fn click_menu_item(&self, app: &mut BrowserApp, id: &str) -> Result<(), String> {
        let Some((mx, my)) = app.smoke_context_menu_origin() else {
            return Err("context menu origin unavailable".to_string());
        };
        let ids = app.smoke_context_menu_item_ids();
        let s = app.scale_factor;
        let normal_h = layout::CONTEXT_MENU_ROW_HEIGHT * s;
        let sep_h = layout::CONTEXT_MENU_SEPARATOR_HEIGHT * s;
        let mut row_center = None;
        let mut offset = 0.0_f32;
        for item in &ids {
            let row_h = if item == "-" { sep_h } else { normal_h };
            if item == id {
                row_center = Some(offset + row_h * 0.5);
                break;
            }
            offset += row_h;
        }
        let Some(center) = row_center else {
            return Err(format!("menu misses {id}: {ids:?}"));
        };
        let x = f64::from(mx + 20.0 * s);
        let y = f64::from(my + center);
        app.handle_mouse_move(x, y);
        app.handle_mouse_click(x, y, true, "Left");
        app.handle_mouse_click(x, y, false, "Left");
        Ok(())
    }

    /// 书签栏首条目中心（几何与 bookmark_bar_item_at 同源：bx=8s，宽=字符数×12s×0.6+24s）。
    fn bookmark_bar_first_item_center(&self, app: &BrowserApp) -> (f64, f64) {
        let s = app.scale_factor;
        let item_w = "M4 Data Page".len() as f32 * 12.0 * s * 0.6 + 24.0 * s;
        let x = f64::from(8.0 * s + item_w * 0.5);
        let toolbar_h = layout::TOOLBAR_HEIGHT * s;
        let chrome_top = app.chrome_top_y_for(s);
        let y = f64::from((toolbar_h + chrome_top) * 0.5);
        (x, y)
    }

    fn capture_step(
        &self,
        _app: &BrowserApp,
        framebuffer: &FrameBuffer,
        source: &str,
        filename: &str,
        step: &str,
    ) -> Result<(), String> {
        let path = self.config.output_dir.join(filename);
        let full = PixelRegion {
            x: 0,
            y: 0,
            width: framebuffer.width,
            height: framebuffer.height,
        };
        smoke_capture::capture_presented_frame(
            &path,
            framebuffer,
            full,
            full,
            "compositor",
            &self.config.base,
            source,
        )?;
        tracing::info!(
            "DATA_SMOKE_STEP step={step} screenshot={} status=captured",
            display_path(&path)
        );
        Ok(())
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// 内部设置 URL 的 query 段编码（与设置页 action_link 同款百分号编码）。
fn urlencode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => out.push(byte as char),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_requires_http_origin_and_output_directory() {
        assert!(DataSmokeConfig::new("http://127.0.0.1:9".into(), "target/data".into()).is_ok());
        assert!(DataSmokeConfig::new("zero://newtab".into(), "target/data".into()).is_err());
        assert!(DataSmokeConfig::new("http://127.0.0.1:9".into(), PathBuf::new()).is_err());
    }

    #[test]
    fn urlencode_keeps_safe_and_encodes_rest() {
        // 与设置页 action_link 同款：':' 编码为 %3A，'/' 保留（strip_prefix 先行）。
        assert_eq!(
            urlencode("http://127.0.0.1:9/page.html"),
            "http%3A//127.0.0.1%3A9/page.html"
        );
        // 本流程编码器保留 "/"（strip_prefix 先于解码，值内裸 "/" 安全）。
        assert_eq!(urlencode("zero://newtab"), "zero%3A//newtab");
    }
}
