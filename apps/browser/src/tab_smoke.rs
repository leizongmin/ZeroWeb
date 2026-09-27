//! M2 标签族/导航控制真实窗口 smoke 的分步状态机。
//!
//! 与 gui_smoke 同模式：由主循环在真实窗口每帧呈现时驱动，全部交互走真实输入路径
//! （handle_key 快捷键 / handle_mouse_click+move 标签条点击与拖拽 / navigate_to 与
//! 地址栏提交同路径），状态断言经 app_smoke_state 只读面，步骤截图落盘供 evidence。

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use zero_browser_shell::TabId;
use zero_render_foundation::surface::FrameBuffer;

use crate::app::BrowserApp;
use crate::smoke_capture::{self, PixelRegion, RegionStats};

const STEP_TIMEOUT: Duration = Duration::from_secs(60);

/// 标签族 smoke 配置。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabSmokeConfig {
    /// 第一个标签导航的目标页（历史/前进断言的基点）。
    pub url_one: String,
    /// 第二个标签导航的目标页（跨标签切换断言）。
    pub url_two: String,
    /// 分步截图输出目录。
    pub output_dir: PathBuf,
}

impl TabSmokeConfig {
    /// 创建并校验 smoke 配置。
    pub fn new(url_one: String, url_two: String, output_dir: PathBuf) -> Result<Self, String> {
        for (name, url) in [("url-one", &url_one), ("url-two", &url_two)] {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(format!("--tab-smoke-{name} requires an http:// or https:// URL"));
            }
        }
        if output_dir.as_os_str().is_empty() {
            return Err("--tab-smoke-dir requires a directory path".to_string());
        }
        Ok(Self {
            url_one,
            url_two,
            output_dir,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Pending,
    /// 启动标签加载第一个页面。
    WaitingFirstLoad,
    /// Ctrl+T 新标签断言 + 截图，随后导航第二页。
    WaitingNewTab,
    /// 第二标签加载完成断言，随后 Ctrl+Tab 切回。
    WaitingSecondLoad,
    /// 切回断言，随后同标签二次导航（构造真实历史条目）。
    WaitingSwitchBack,
    /// 标签内二次导航落地断言，随后 Alt+Left 后退。
    WaitingInTabSecond,
    /// 后退到第一页断言，随后 Alt+Right 前进。
    WaitingBack,
    /// 前进恢复断言，记录 epoch 后 F5 刷新。
    WaitingForward,
    /// 重载 epoch 前进断言 + 记录标签条基线，随后拖拽重排。
    WaitingReload,
    /// 拖拽后顺序断言 + 标签条可视变化断言，随后 Ctrl+W 关闭。
    WaitingDragApplied,
    /// 关闭后剩余标签断言，最后截图。
    WaitingClose,
    Complete,
}

/// 标签族 smoke 执行状态。
pub struct TabSmoke {
    config: TabSmokeConfig,
    stage: Stage,
    deadline: Instant,
    first_tab_id: Option<TabId>,
    second_tab_id: Option<TabId>,
    pre_reload_epoch: u64,
    tab_strip_baseline: Option<RegionStats>,
}

impl TabSmoke {
    /// 创建尚未启动的 smoke。
    pub fn new(config: TabSmokeConfig) -> Self {
        Self {
            config,
            stage: Stage::Pending,
            deadline: Instant::now() + STEP_TIMEOUT,
            first_tab_id: None,
            second_tab_id: None,
            pre_reload_epoch: 0,
            tab_strip_baseline: None,
        }
    }

    /// 在浏览器窗口与默认标签页就绪后开始流程。
    pub fn start(&mut self, app: &mut BrowserApp) {
        if self.stage != Stage::Pending {
            return;
        }
        let first = app.smoke_active_tab_id().unwrap_or_else(|| {
            app.ensure_startup_tab();
            app.smoke_active_tab_id().expect("startup tab after ensure")
        });
        self.first_tab_id = Some(first);
        tracing::info!(
            "TAB_SMOKE_START first_tab={} url_one={} url_two={}",
            first.0,
            self.config.url_one,
            self.config.url_two
        );
        app.navigate_to(&self.config.url_one);
        self.advance(Stage::WaitingFirstLoad);
    }

    /// 检查当前步骤是否超过内部墙钟上限。
    pub fn check_timeout(&self) -> Result<(), String> {
        if self.stage != Stage::Complete && Instant::now() > self.deadline {
            return Err(format!("Tab smoke timed out in stage {:?}", self.stage));
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
            // 页面静止后事件循环不再自发重绘；smoke 未完成前必须自驱动下一帧，
            // 否则 Waiting* 阶段永远等不到呈现帧（步骤推进与超时都依赖帧回调）。
            app.needs_redraw = true;
            return Ok(self.stage == Stage::Complete);
        }
        // 每个阶段在同一帧内完成断言/截图/下一步输入后，也要保证还有下一帧可消费。
        app.needs_redraw = true;
        let first = self.first_tab_id.expect("start() recorded first tab");

        match self.stage {
            Stage::WaitingFirstLoad => {
                self.assert_tab_url(app, first, &self.config.url_one, "first-load")?;
                tracing::info!("TAB_SMOKE_STEP step=first-load status=passed");

                // Ctrl+T 经真实快捷键路径新开标签。
                self.press_with_ctrl(app, "t", false);
                let second = app
                    .smoke_active_tab_id()
                    .ok_or_else(|| "no active tab after Ctrl+T".to_string())?;
                if second == first {
                    return Err("Ctrl+T did not switch to a new tab".to_string());
                }
                self.second_tab_id = Some(second);
                let count = app.smoke_tab_ids().len();
                if count != 2 {
                    return Err(format!("tab count after Ctrl+T is {count}, expected 2"));
                }
                self.capture_step(app, framebuffer, source, "01-new-tab.png", "new-tab")?;
                tracing::info!("TAB_SMOKE_STEP step=new-tab status=passed tab_count=2");

                app.navigate_to(&self.config.url_two);
                self.advance(Stage::WaitingSecondLoad);
            }
            Stage::WaitingSecondLoad => {
                let second = self.second_tab_id.expect("second tab recorded");
                self.assert_tab_url(app, second, &self.config.url_two, "second-load")?;
                self.assert_address_bar(app, &self.config.url_two, "second-load")?;
                self.capture_step(app, framebuffer, source, "02-tab-two.png", "tab-two")?;

                // Ctrl+T 后地址栏处于聚焦态（新标签 Chrome 语义）；先点击页面区
                // 失焦（真实用户路径），否则按键会派发给地址栏编辑器而非全局快捷键。
                let (px, py, pw, ph) = app.page_content_rect_for(framebuffer.width, framebuffer.height);
                let page_x = f64::from(px + pw * 0.5);
                let page_y = f64::from(py + ph * 0.5);
                app.handle_mouse_click(page_x, page_y, true, "Left");
                app.handle_mouse_click(page_x, page_y, false, "Left");

                // Ctrl+Tab 循环切回第一个标签。
                self.press_with_ctrl(app, "Tab", false);
                if app.smoke_active_tab_id() != Some(first) {
                    return Err("Ctrl+Tab did not switch back to the first tab".to_string());
                }
                self.assert_address_bar(app, &self.config.url_one, "switch-back")?;
                self.capture_step(app, framebuffer, source, "03-switched-back.png", "switch-back")?;
                tracing::info!("TAB_SMOKE_STEP step=switch-back status=passed");

                // 同标签内二次导航（ONE -> TWO）：启动欢迎页不进历史（不经 shell.navigate），
                // 真实历史条目须由文档间导航构成——后退/前进断言基于这条两入口历史。
                app.navigate_to(&self.config.url_two);
                tracing::info!("TAB_SMOKE_ACTION action=in-tab-navigate status=executed");
                self.advance(Stage::WaitingInTabSecond);
            }
            Stage::WaitingInTabSecond => {
                // 标签内二次导航落地：第一个标签此刻显示 TWO（历史 [ONE, TWO]）。
                self.assert_tab_url(app, first, &self.config.url_two, "in-tab-second")?;
                tracing::info!("TAB_SMOKE_STEP step=in-tab-nav status=passed");

                // Alt+Left 浏览器后退：跨真实历史条目回到 ONE。
                self.press_with_alt(app, "ArrowLeft");
                self.advance(Stage::WaitingBack);
            }
            Stage::WaitingBack => {
                self.assert_tab_url(app, first, &self.config.url_one, "go-back")?;
                self.capture_step(app, framebuffer, source, "04-went-back.png", "go-back")?;
                tracing::info!("TAB_SMOKE_STEP step=go-back status=passed");

                // Alt+Right 浏览器前进：恢复 TWO。
                self.press_with_alt(app, "ArrowRight");
                self.advance(Stage::WaitingForward);
            }
            Stage::WaitingForward => {
                self.assert_tab_url(app, first, &self.config.url_two, "go-forward")?;
                self.capture_step(app, framebuffer, source, "05-went-forward.png", "go-forward")?;
                tracing::info!("TAB_SMOKE_STEP step=go-forward status=passed");

                // F5 刷新：记录 epoch，重载完成后断言前进。
                self.pre_reload_epoch = app.smoke_navigation_epoch(first);
                app.handle_key("F5", true, None);
                app.handle_key("F5", false, None);
                tracing::info!("TAB_SMOKE_ACTION action=reload status=executed");
                self.advance(Stage::WaitingReload);
            }
            Stage::WaitingReload => {
                self.assert_tab_url(app, first, &self.config.url_two, "reload")?;
                let epoch = app.smoke_navigation_epoch(first);
                if epoch <= self.pre_reload_epoch {
                    return Err(format!("reload did not advance navigation epoch ({epoch})"));
                }
                let strip = self.capture_step(app, framebuffer, source, "06-reloaded.png", "reload")?;
                self.tab_strip_baseline = Some(strip);
                tracing::info!(
                    "TAB_SMOKE_STEP step=reload status=passed epoch {} -> {}",
                    self.pre_reload_epoch,
                    epoch
                );

                // 标签拖拽重排：按住第一个标签中心，拖过第二个标签中点后释放。
                let second = self.second_tab_id.expect("second tab recorded");
                let (x1, w1) = app
                    .smoke_tab_rect(first)
                    .ok_or_else(|| "first tab rect unavailable for drag".to_string())?;
                let (x2, w2) = app
                    .smoke_tab_rect(second)
                    .ok_or_else(|| "second tab rect unavailable for drag".to_string())?;
                let y = tab_center_y(app);
                let start_x = f64::from(x1 + w1 * 0.5);
                let end_x = f64::from(x2 + w2 * 0.75);
                app.handle_mouse_click(start_x, y, true, "Left");
                let steps = 8;
                for step in 1..=steps {
                    let x = start_x + (end_x - start_x) * (step as f64) / (steps as f64);
                    app.handle_mouse_move(x, y);
                }
                app.handle_mouse_click(end_x, y, false, "Left");
                tracing::info!("TAB_SMOKE_ACTION action=tab-drag status=executed");
                self.advance(Stage::WaitingDragApplied);
            }
            Stage::WaitingDragApplied => {
                let ids = app.smoke_tab_ids();
                let second = self.second_tab_id.expect("second tab recorded");
                let first_id = self.first_tab_id.expect("first tab recorded");
                if ids != vec![second, first_id] {
                    return Err(format!(
                        "tab drag did not move the first tab to the end: {:?}",
                        ids.iter().map(|id| id.0).collect::<Vec<_>>()
                    ));
                }
                let strip = self.capture_step(app, framebuffer, source, "07-reordered.png", "tab-drag")?;
                crate::gui_smoke::require_visual_change(self.tab_strip_baseline.as_ref(), &strip, "tab_drag")?;
                tracing::info!("TAB_SMOKE_STEP step=tab-drag status=passed");

                // Ctrl+W 关闭活动标签（拖拽后的第一个标签）。
                self.press_with_ctrl(app, "w", false);
                let ids = app.smoke_tab_ids();
                if ids != vec![second] {
                    return Err(format!(
                        "Ctrl+W did not leave only the second tab: {:?}",
                        ids.iter().map(|id| id.0).collect::<Vec<_>>()
                    ));
                }
                self.advance(Stage::WaitingClose);
            }
            Stage::WaitingClose => {
                let second = self.second_tab_id.expect("second tab recorded");
                if app.smoke_active_tab_id() != Some(second) {
                    return Err("surviving tab is not active after Ctrl+W".to_string());
                }
                self.capture_step(app, framebuffer, source, "08-closed.png", "close-tab")?;
                tracing::info!(
                    "TAB_SMOKE_COMPLETE url_one={} url_two={} steps=new-tab,switch,in-tab-nav,back,forward,reload,drag,close",
                    self.config.url_one,
                    self.config.url_two
                );
                self.stage = Stage::Complete;
                return Ok(true);
            }
            // WaitingNewTab / WaitingSwitchBack 折叠进前序步骤（按键 + 断言 + 截图
            // 同帧完成），不单独等待帧。
            Stage::Pending | Stage::WaitingNewTab | Stage::WaitingSwitchBack | Stage::Complete => {}
        }

        Ok(false)
    }

    fn advance(&mut self, stage: Stage) {
        self.stage = stage;
        self.deadline = Instant::now() + STEP_TIMEOUT;
    }

    fn assert_tab_url(&self, app: &BrowserApp, tab: TabId, expected: &str, step: &str) -> Result<(), String> {
        let url = app
            .smoke_tab_url(tab)
            .ok_or_else(|| format!("{step}: tab {} has no snapshot url", tab.0))?;
        if url != expected {
            return Err(format!("{step}: tab url is {url}, expected {expected}"));
        }
        Ok(())
    }

    fn assert_address_bar(&self, app: &BrowserApp, expected: &str, step: &str) -> Result<(), String> {
        let text = app.smoke_address_bar_text();
        if text != expected {
            return Err(format!("{step}: address bar shows {text:?}, expected {expected:?}"));
        }
        Ok(())
    }

    fn press_with_ctrl(&self, app: &mut BrowserApp, key: &str, shift: bool) {
        app.handle_key("Control", true, None);
        if shift {
            app.handle_key("Shift", true, None);
        }
        app.handle_key(key, true, None);
        app.handle_key(key, false, None);
        if shift {
            app.handle_key("Shift", false, None);
        }
        app.handle_key("Control", false, None);
        app.needs_redraw = true;
    }

    fn press_with_alt(&self, app: &mut BrowserApp, key: &str) {
        app.handle_key("Alt", true, None);
        app.handle_key(key, true, None);
        app.handle_key(key, false, None);
        app.handle_key("Alt", false, None);
        app.needs_redraw = true;
    }

    fn capture_step(
        &self,
        app: &BrowserApp,
        framebuffer: &FrameBuffer,
        source: &str,
        filename: &str,
        step: &str,
    ) -> Result<RegionStats, String> {
        let path = self.config.output_dir.join(filename);
        let full = PixelRegion {
            x: 0,
            y: 0,
            width: framebuffer.width,
            height: framebuffer.height,
        };
        // 截整窗：标签族断言的主体是 chrome 标签条，页面区只是上下文。
        smoke_capture::capture_presented_frame(
            &path,
            framebuffer,
            full,
            full,
            "compositor",
            &self.config.url_one,
            source,
        )?;
        let strip = smoke_capture::analyze_region(
            framebuffer.width,
            framebuffer.height,
            &framebuffer.data,
            tab_strip_region(app, framebuffer),
        )?;
        tracing::info!(
            "TAB_SMOKE_STEP step={step} screenshot={} status=captured",
            display_path(&path)
        );
        Ok(strip)
    }
}

/// 标签条中心线的物理 y 坐标（与 tests.rs 的点击配方一致）。
fn tab_center_y(app: &BrowserApp) -> f64 {
    let s = app.scale_factor;
    f64::from((crate::layout::TAB_BAR_TOP_INSET + crate::layout::TAB_BAR_HEIGHT * 0.5) * s)
}

/// 标签条像素区（可视变化断言的作用域）。
fn tab_strip_region(app: &BrowserApp, framebuffer: &FrameBuffer) -> PixelRegion {
    let s = app.scale_factor;
    let height = ((crate::layout::TAB_BAR_TOP_INSET + crate::layout::TAB_BAR_HEIGHT) * s)
        .ceil()
        .max(1.0) as u32;
    PixelRegion {
        x: 0,
        y: 0,
        width: framebuffer.width,
        height: height.min(framebuffer.height),
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(signature: Vec<u8>) -> RegionStats {
        RegionStats {
            pixels: 64,
            opaque_pixels: 64,
            unique_bins: 8,
            dominant_ratio: 0.5,
            luma_min: 0,
            luma_max: 255,
            dark_pixels: 8,
            dark_ratio: 0.125,
            signature,
        }
    }

    #[test]
    fn config_requires_http_urls_and_output_directory() {
        assert!(
            TabSmokeConfig::new(
                "http://127.0.0.1:9/one.html".into(),
                "http://127.0.0.1:9/two.html".into(),
                "target/tab".into()
            )
            .is_ok()
        );
        assert!(
            TabSmokeConfig::new(
                "zero://newtab".into(),
                "http://127.0.0.1:9/two.html".into(),
                "target/tab".into()
            )
            .is_err()
        );
        assert!(
            TabSmokeConfig::new(
                "http://127.0.0.1:9/one.html".into(),
                "http://127.0.0.1:9/two.html".into(),
                PathBuf::new()
            )
            .is_err()
        );
    }

    #[test]
    fn visual_change_reuse_matches_gui_smoke_contract() {
        let baseline = stats(vec![100; 64]);
        let mut changed = vec![100; 64];
        changed[..4].fill(110);
        assert!(crate::gui_smoke::require_visual_change(Some(&baseline), &stats(changed), "tab_drag").is_ok());
    }
}
