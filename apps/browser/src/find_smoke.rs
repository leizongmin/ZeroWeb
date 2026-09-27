//! M3 页面查找真实窗口 smoke（Ctrl+F 键入 → 计数/高亮 → Enter 逐项滚动 → 关闭）
//! 的分步状态机。状态断言经 app_smoke_state 只读面（find_state + scroll_y）。

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use zero_render_foundation::surface::FrameBuffer;

use crate::app::BrowserApp;
use crate::smoke_capture::{self, PixelRegion};

const STEP_TIMEOUT: Duration = Duration::from_secs(60);
const PAGE_PATH: &str = "/find.html";
/// 查询词（fixture 页内出现 5 次，含首屏外）。
const QUERY: &str = "needle";

/// 查找 smoke 配置。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindSmokeConfig {
    /// 本地 fixture 服务 origin（如 http://127.0.0.1:PORT）。
    pub base: String,
    /// 分步截图输出目录。
    pub output_dir: PathBuf,
}

impl FindSmokeConfig {
    /// 创建并校验 smoke 配置。
    pub fn new(base: String, output_dir: PathBuf) -> Result<Self, String> {
        if !base.starts_with("http://") && !base.starts_with("https://") {
            return Err("--find-smoke-base requires an http:// or https:// origin".to_string());
        }
        if output_dir.as_os_str().is_empty() {
            return Err("--find-smoke-dir requires a directory path".to_string());
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
    /// 文本页加载（多处 needle，含首屏外）。
    WaitingSeedLoad,
    /// Ctrl+F + 键入 needle：计数落地断言（total >= 2）+ 高亮/计数截图。
    WaitingTyped,
    /// Enter 逐项：current 前进 + 滚动偏移变化断言 + 截图。
    WaitingFirstNext,
    /// 再 Enter：current 再前进（跨匹配推进）。
    WaitingSecondNext,
    /// Escape 关闭：find_state 复位断言。
    WaitingClosed,
    Complete,
}

/// 查找 smoke 执行状态。
pub struct FindSmoke {
    config: FindSmokeConfig,
    stage: Stage,
    deadline: Instant,
    scroll_before_next: f32,
}

impl FindSmoke {
    /// 创建尚未启动的 smoke。
    pub fn new(config: FindSmokeConfig) -> Self {
        Self {
            config,
            stage: Stage::Pending,
            deadline: Instant::now() + STEP_TIMEOUT,
            scroll_before_next: 0.0,
        }
    }

    /// 在浏览器窗口与默认标签页就绪后开始流程。
    pub fn start(&mut self, app: &mut BrowserApp) {
        if self.stage != Stage::Pending {
            return;
        }
        tracing::info!("FIND_SMOKE_START base={}", self.config.base);
        app.navigate_to(&self.config.url(PAGE_PATH));
        self.advance(Stage::WaitingSeedLoad);
    }

    /// 检查当前步骤是否超过内部墙钟上限。
    pub fn check_timeout(&self) -> Result<(), String> {
        if self.stage != Stage::Complete && Instant::now() > self.deadline {
            return Err(format!("Find smoke timed out in stage {:?}", self.stage));
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
                tracing::info!("FIND_SMOKE_STEP step=seed-load status=passed");

                // Ctrl+F 打开查找栏 + 逐字符键入查询词（真实键入路径）。
                app.handle_key("Control", true, None);
                app.handle_key("f", true, None);
                app.handle_key("f", false, None);
                app.handle_key("Control", false, None);
                for ch in QUERY.chars() {
                    let key = ch.to_string();
                    app.handle_key(&key, true, None);
                    app.handle_key(&key, false, None);
                }
                tracing::info!("FIND_SMOKE_ACTION action=type-query status=executed query={QUERY}");
                self.advance(Stage::WaitingTyped);
            }
            Stage::WaitingTyped => {
                let (active, query, _current, total) = app.smoke_find_state();
                if !active {
                    return Ok(false); // 查找栏激活传播中
                }
                if query != QUERY {
                    return Err(format!("find query is {query:?}, expected {QUERY:?}"));
                }
                if total < 2 {
                    if app.smoke_scroll_y() == 0.0 && total == 0 {
                        return Ok(false); // 匹配计算尚未落地
                    }
                    return Err(format!("expected >=2 matches, got {total}"));
                }
                self.capture_step(app, framebuffer, source, "01-find-count.png", "find-count")?;
                tracing::info!("FIND_SMOKE_STEP step=find-count status=passed total={total}");

                // Enter：推进到匹配 2（首屏内，预期不滚动）。
                self.scroll_before_next = app.smoke_scroll_y();
                app.handle_key("Enter", true, None);
                app.handle_key("Enter", false, None);
                tracing::info!("FIND_SMOKE_ACTION action=find-next-1 status=executed");
                self.advance(Stage::WaitingFirstNext);
            }
            Stage::WaitingFirstNext => {
                let (_active, _query, current, _total) = app.smoke_find_state();
                if current < 2 {
                    return Ok(false); // 推进尚未落地
                }
                self.capture_step(app, framebuffer, source, "02-find-next.png", "find-next")?;
                tracing::info!(
                    "FIND_SMOKE_STEP step=find-next status=passed current={current} scroll_y={:.1}",
                    app.smoke_scroll_y()
                );

                // 再 Enter：推进到匹配 3（fixture 900px 上边距，首屏外 → 必触发滚动定位）。
                self.scroll_before_next = app.smoke_scroll_y();
                app.handle_key("Enter", true, None);
                app.handle_key("Enter", false, None);
                tracing::info!("FIND_SMOKE_ACTION action=find-next-2 status=executed");
                self.advance(Stage::WaitingSecondNext);
            }
            Stage::WaitingSecondNext => {
                let (_active, _query, current, _total) = app.smoke_find_state();
                if current < 3 {
                    return Ok(false); // 推进尚未落地
                }
                // 第三个匹配在首屏之外：推进必须触发滚动定位。
                let scrolled = (app.smoke_scroll_y() - self.scroll_before_next).abs() > 1.0;
                if !scrolled {
                    return Err(format!(
                        "advancing to the below-fold match did not scroll (scroll_y={:.1}, before={:.1})",
                        app.smoke_scroll_y(),
                        self.scroll_before_next
                    ));
                }
                tracing::info!(
                    "FIND_SMOKE_STEP step=find-advance status=passed current={current} scroll_y={:.1}",
                    app.smoke_scroll_y()
                );

                // Escape 关闭查找栏：状态复位。
                app.handle_key("Escape", true, None);
                app.handle_key("Escape", false, None);
                self.advance(Stage::WaitingClosed);
            }
            Stage::WaitingClosed => {
                let (active, query, current, total) = app.smoke_find_state();
                if active {
                    return Ok(false); // 关闭传播中
                }
                if !query.is_empty() || current != 0 || total != 0 {
                    return Err(format!(
                        "find state not reset: query={query:?} current={current} total={total}"
                    ));
                }
                self.capture_step(app, framebuffer, source, "03-find-closed.png", "find-closed")?;
                tracing::info!(
                    "FIND_SMOKE_COMPLETE base={} steps=type-query,count,next,advance,close",
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
            "FIND_SMOKE_STEP step={step} screenshot={} status=captured",
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
        assert!(FindSmokeConfig::new("http://127.0.0.1:9".into(), "target/find".into()).is_ok());
        assert!(FindSmokeConfig::new("zero://newtab".into(), "target/find".into()).is_err());
        assert!(FindSmokeConfig::new("http://127.0.0.1:9".into(), PathBuf::new()).is_err());
    }
}
