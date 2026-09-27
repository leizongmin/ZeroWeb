//! M2 地址栏真实窗口 smoke（键入导航 / 自动补全 / 加载指示）的分步状态机。
//!
//! 与 tab_smoke 同模式：真实输入路径驱动——Ctrl+L 聚焦地址栏、逐字符键入、
//! ArrowDown 选中建议、Enter 提交；加载指示腿经本地延迟页面（slow.html）在真实
//! loading 窗口内采样。状态断言经 app_smoke_state 只读面。

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use zero_render_foundation::surface::FrameBuffer;

use crate::app::BrowserApp;
use crate::smoke_capture::{self, PixelRegion};

const STEP_TIMEOUT: Duration = Duration::from_secs(60);
/// 本地延迟页面响应延迟（保证 loading 窗口可被帧采样）。
const SLOW_PATH: &str = "/slow.html";
const ONE_PATH: &str = "/one.html";
const TWO_PATH: &str = "/two.html";

/// 地址栏 smoke 配置。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddressbarSmokeConfig {
    /// 本地 fixture 服务 origin（如 http://127.0.0.1:PORT）。
    pub base: String,
    /// 分步截图输出目录。
    pub output_dir: PathBuf,
}

impl AddressbarSmokeConfig {
    /// 创建并校验 smoke 配置，派生三个 fixture URL。
    pub fn new(base: String, output_dir: PathBuf) -> Result<Self, String> {
        if !base.starts_with("http://") && !base.starts_with("https://") {
            return Err("--addressbar-smoke-base requires an http:// or https:// origin".to_string());
        }
        if output_dir.as_os_str().is_empty() {
            return Err("--addressbar-smoke-dir requires a directory path".to_string());
        }
        Ok(Self {
            base: base.trim_end_matches('/').to_string(),
            output_dir,
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    fn url_one(&self) -> String {
        self.url(ONE_PATH)
    }

    fn url_two(&self) -> String {
        self.url(TWO_PATH)
    }

    fn url_slow(&self) -> String {
        self.url(SLOW_PATH)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Pending,
    /// 历史种子页加载（one.html 进历史，供补全匹配）。
    WaitingSeedLoad,
    /// 完整 URL 键入落地断言 + 补全下拉截图，随后 Enter 提交。
    WaitingTypedUrl,
    /// 键入导航加载完成断言，随后键入 "two" 触发补全。
    WaitingTypedLoad,
    /// 建议列表断言 + ArrowDown 选中历史项 + 下拉截图，随后 Enter。
    WaitingSuggest,
    /// 建议导航落地断言，随后导航延迟页。
    WaitingSuggestNav,
    /// 延迟页加载完成断言（loading 窗口内证据在 gate 分支采样）。
    WaitingSlowLoad,
    Complete,
}

/// 地址栏 smoke 执行状态。
pub struct AddressbarSmoke {
    config: AddressbarSmokeConfig,
    stage: Stage,
    deadline: Instant,
    mid_load_captured: bool,
}

impl AddressbarSmoke {
    /// 创建尚未启动的 smoke。
    pub fn new(config: AddressbarSmokeConfig) -> Self {
        Self {
            config,
            stage: Stage::Pending,
            deadline: Instant::now() + STEP_TIMEOUT,
            mid_load_captured: false,
        }
    }

    /// 在浏览器窗口与默认标签页就绪后开始流程。
    pub fn start(&mut self, app: &mut BrowserApp) {
        if self.stage != Stage::Pending {
            return;
        }
        tracing::info!("ADDR_SMOKE_START base={}", self.config.base);
        app.navigate_to(&self.config.url_one());
        self.advance(Stage::WaitingSeedLoad);
    }

    /// 检查当前步骤是否超过内部墙钟上限。
    pub fn check_timeout(&self) -> Result<(), String> {
        if self.stage != Stage::Complete && Instant::now() > self.deadline {
            return Err(format!("Addressbar smoke timed out in stage {:?}", self.stage));
        }
        Ok(())
    }

    /// Poll tick 采样：延迟页 loading 窗口内抓一帧真实 loading 指示（标签条 spinner）。
    ///
    /// 不依赖帧回调——页面加载期事件循环可能整段空闲（无 OS 事件时 16ms Poll tick
    /// 是唯一保证的推进点，RedrawRequested 链会断），故经 Poll 主动渲染一帧捕获。
    pub fn sample_mid_load(&mut self, app: &mut BrowserApp) -> Result<(), String> {
        if self.stage != Stage::WaitingSlowLoad || self.mid_load_captured || !app.any_tab_loading() {
            return Ok(());
        }
        let (width, height) = app.physical_size;
        if let Ok(frame) = app.render_full_scene_gpu_capture(width, height) {
            self.mid_load_captured = true;
            self.capture_step(app, &frame, "poll-tick", "04-loading.png", "loading-indicator")?;
            tracing::info!("ADDR_SMOKE_STEP step=loading-indicator status=passed (mid-load sample)");
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
        if self.stage == Stage::Pending || self.stage == Stage::Complete {
            return Ok(self.stage == Stage::Complete);
        }
        // smoke 未完成前自驱动下一帧（页面静止后事件循环不自发重绘）。
        app.needs_redraw = true;

        // loading 窗口内帧回调链可能整段断流（见 sample_mid_load 注记），
        // 中途证据由主循环 Poll tick 的 sample_mid_load 采样；此处仅等待加载结束。
        if app.any_tab_loading() {
            return Ok(false);
        }
        let active = app.smoke_active_tab_id().ok_or_else(|| "no active tab".to_string())?;

        match self.stage {
            Stage::WaitingSeedLoad => {
                self.assert_tab_url(app, active, &self.config.url_one(), "seed-load")?;
                tracing::info!("ADDR_SMOKE_STEP step=seed-load status=passed");

                // Ctrl+L 聚焦地址栏（全选），逐字符键入完整 URL（真实键入路径）。
                self.press_ctrl_l(app);
                self.type_text(app, &self.config.url_two());
                self.advance(Stage::WaitingTypedUrl);
            }
            Stage::WaitingTypedUrl => {
                let typed = self.config.url_two();
                let text = app.smoke_address_bar_text().to_string();
                if text != typed {
                    return Err(format!("typed address bar text is {text:?}, expected {typed:?}"));
                }
                self.capture_step(app, framebuffer, source, "01-typed-url.png", "typed-url")?;
                tracing::info!("ADDR_SMOKE_STEP step=typed-url status=passed");

                // Enter 提交键入 URL（无选中建议 → 导航键入文本）。
                app.handle_key("Enter", true, None);
                app.handle_key("Enter", false, None);
                tracing::info!("ADDR_SMOKE_ACTION action=typed-enter status=executed");
                self.advance(Stage::WaitingTypedLoad);
            }
            Stage::WaitingTypedLoad => {
                self.assert_tab_url(app, active, &self.config.url_two(), "typed-load")?;
                tracing::info!("ADDR_SMOKE_STEP step=typed-nav status=passed");

                // 键入历史命中词：弹出补全（搜索建议 + 历史项）。
                self.press_ctrl_l(app);
                self.type_text(app, "two");
                self.advance(Stage::WaitingSuggest);
            }
            Stage::WaitingSuggest => {
                let suggestions = app.smoke_autocomplete_urls();
                // "two" 不像 URL → 顶部插入搜索建议，历史项 two.html 在其后的某位。
                let hit = suggestions
                    .iter()
                    .position(|url| url == &self.config.url_two())
                    .ok_or_else(|| format!("autocomplete misses {}: {suggestions:?}", self.config.url_two()))?;
                if suggestions.len() < 2 {
                    return Err(format!(
                        "autocomplete should offer a search suggestion plus history hit: {suggestions:?}"
                    ));
                }

                // ArrowDown 逐位选中历史项（highlight = hovered.or(selected)）。
                for _ in 0..=hit {
                    app.handle_key("ArrowDown", true, None);
                    app.handle_key("ArrowDown", false, None);
                }
                if app.smoke_autocomplete_highlight() != Some(hit) {
                    return Err(format!(
                        "ArrowDown selection is {:?}, expected Some({hit})",
                        app.smoke_autocomplete_highlight()
                    ));
                }
                self.capture_step(app, framebuffer, source, "02-suggest-popup.png", "suggest-popup")?;
                tracing::info!(
                    "ADDR_SMOKE_STEP step=suggest-popup status=passed selected={hit} suggestions={suggestions:?}"
                );

                // Enter 按选中建议导航。
                app.handle_key("Enter", true, None);
                app.handle_key("Enter", false, None);
                tracing::info!("ADDR_SMOKE_ACTION action=suggest-enter status=executed");
                self.advance(Stage::WaitingSuggestNav);
            }
            Stage::WaitingSuggestNav => {
                self.assert_tab_url(app, active, &self.config.url_two(), "suggest-nav")?;
                self.capture_step(app, framebuffer, source, "03-suggest-loaded.png", "suggest-nav")?;
                tracing::info!("ADDR_SMOKE_STEP step=suggest-nav status=passed");

                // 延迟页：真实 loading 窗口（gate 分支采样 spinner 帧）。
                app.navigate_to(&self.config.url_slow());
                tracing::info!("ADDR_SMOKE_ACTION action=navigate-slow status=executed");
                self.advance(Stage::WaitingSlowLoad);
            }
            Stage::WaitingSlowLoad => {
                if !self.mid_load_captured {
                    return Err("slow page finished loading without a mid-load sample".to_string());
                }
                self.assert_tab_url(app, active, &self.config.url_slow(), "slow-load")?;
                self.capture_step(app, framebuffer, source, "05-slow-loaded.png", "slow-loaded")?;
                tracing::info!(
                    "ADDR_SMOKE_COMPLETE base={} steps=typed-nav,suggest-nav,loading-indicator",
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

    fn assert_tab_url(
        &self,
        app: &BrowserApp,
        tab: zero_browser_shell::TabId,
        expected: &str,
        step: &str,
    ) -> Result<(), String> {
        let url = app
            .smoke_tab_url(tab)
            .ok_or_else(|| format!("{step}: tab has no snapshot url"))?;
        if url != expected {
            return Err(format!("{step}: tab url is {url}, expected {expected}"));
        }
        Ok(())
    }

    fn press_ctrl_l(&self, app: &mut BrowserApp) {
        app.handle_key("Control", true, None);
        app.handle_key("l", true, None);
        app.handle_key("l", false, None);
        app.handle_key("Control", false, None);
        app.needs_redraw = true;
    }

    fn type_text(&self, app: &mut BrowserApp, text: &str) {
        for ch in text.chars() {
            let key = ch.to_string();
            app.handle_key(&key, true, None);
            app.handle_key(&key, false, None);
        }
        app.needs_redraw = true;
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
            &self.config.url_one(),
            source,
        )?;
        tracing::info!(
            "ADDR_SMOKE_STEP step={step} screenshot={} status=captured",
            display_path(&path)
        );
        Ok(())
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_requires_http_origin_and_output_directory() {
        assert!(AddressbarSmokeConfig::new("http://127.0.0.1:9".into(), "target/addr".into()).is_ok());
        assert!(AddressbarSmokeConfig::new("zero://newtab".into(), "target/addr".into()).is_err());
        assert!(AddressbarSmokeConfig::new("http://127.0.0.1:9".into(), PathBuf::new()).is_err());
    }

    #[test]
    fn urls_derived_from_base_without_trailing_slash() {
        let cfg = AddressbarSmokeConfig::new("http://127.0.0.1:9/".into(), "target/addr".into()).unwrap();
        assert_eq!(cfg.url_two(), "http://127.0.0.1:9/two.html");
        assert_eq!(cfg.url_slow(), "http://127.0.0.1:9/slow.html");
    }
}
