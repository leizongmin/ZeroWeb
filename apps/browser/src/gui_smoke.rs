//! 真实网站 compositor GUI smoke 的分步状态机。

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use zero_host_runtime::event::MouseScrollDelta;
use zero_render_foundation::surface::FrameBuffer;

use crate::app::BrowserApp;
use crate::smoke_capture::{self, PixelRegion, RegionStats};

const STEP_TIMEOUT: Duration = Duration::from_secs(90);

/// 真实网站 GUI smoke 配置。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuiSmokeConfig {
    /// 待验收的真实 HTTP(S) 网站。
    pub url: String,
    /// 分步截图输出目录。
    pub output_dir: PathBuf,
}

impl GuiSmokeConfig {
    /// 创建并校验 GUI smoke 配置。
    pub fn new(url: String, output_dir: PathBuf) -> Result<Self, String> {
        if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("file://") {
            return Err("--gui-smoke-url requires an http://, https://, or file:// URL".to_string());
        }
        if output_dir.as_os_str().is_empty() {
            return Err("--gui-smoke-dir requires a directory path".to_string());
        }
        Ok(Self { url, output_dir })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Pending,
    WaitingInitialFrame,
    WaitingScrolledFrame,
    WaitingZoomedFrame,
    WaitingReloadedFrame,
    Complete,
}

/// 真实窗口 GUI smoke 执行状态。
pub struct GuiSmoke {
    config: GuiSmokeConfig,
    stage: Stage,
    deadline: Instant,
    previous_page: Option<RegionStats>,
    initial_frame_retry_requested: bool,
}

impl GuiSmoke {
    /// 创建尚未启动的 GUI smoke。
    pub fn new(config: GuiSmokeConfig) -> Self {
        Self {
            config,
            stage: Stage::Pending,
            deadline: Instant::now() + STEP_TIMEOUT,
            previous_page: None,
            initial_frame_retry_requested: false,
        }
    }

    /// 在 Browser 窗口和默认标签页就绪后导航到待验收网站。
    pub fn start(&mut self, app: &mut BrowserApp) {
        if self.stage != Stage::Pending {
            return;
        }
        tracing::info!("GUI_SMOKE_NAVIGATE url={}", self.config.url);
        app.navigate_to(&self.config.url);
        self.advance(Stage::WaitingInitialFrame);
    }

    /// 检查当前步骤是否超过内部墙钟上限。
    pub fn check_timeout(&self) -> Result<(), String> {
        if self.stage != Stage::Complete && Instant::now() > self.deadline {
            return Err(format!("GUI smoke timed out in stage {:?}", self.stage));
        }
        Ok(())
    }

    /// 消费一个已真实呈现的 compositor framebuffer。
    ///
    /// 返回 `true` 表示全部步骤完成，调用方应正常关闭子进程并退出。
    pub fn on_presented_frame(
        &mut self,
        app: &mut BrowserApp,
        framebuffer: &FrameBuffer,
        source: &str,
    ) -> Result<bool, String> {
        if self.stage == Stage::Pending || self.stage == Stage::Complete || app.any_tab_loading() {
            return Ok(self.stage == Stage::Complete);
        }
        if matches!(self.stage, Stage::WaitingInitialFrame | Stage::WaitingReloadedFrame)
            && visible_page_stats(app, framebuffer)?.is_none()
        {
            // 网络导航会先发布空白过渡帧；只接受首个具备真实可见内容的 compositor 帧。
            if self.stage == Stage::WaitingInitialFrame && !self.initial_frame_retry_requested {
                app.sync_webview_viewport();
                app.needs_redraw = true;
                self.initial_frame_retry_requested = true;
                tracing::info!("GUI_SMOKE_RETRY reason=blank_initial_frame action=resync_viewport");
            }
            return Ok(false);
        }

        match self.stage {
            Stage::WaitingInitialFrame => {
                let page = self.capture_step(app, framebuffer, source, "01-loaded.png", "loaded")?;
                self.previous_page = Some(page);
                if self.config.url.starts_with("file://") {
                    tracing::info!("GUI_SMOKE_COMPLETE url={} steps=load", self.config.url);
                    self.stage = Stage::Complete;
                    return Ok(true);
                }

                let (x, y, width, height) = app.page_content_rect_for(framebuffer.width, framebuffer.height);
                app.handle_scroll(
                    MouseScrollDelta::LineDelta(0.0, -8.0),
                    f64::from(x + width / 2.0),
                    f64::from(y + height / 2.0),
                );
                tracing::info!("GUI_SMOKE_ACTION action=scroll status=executed");
                self.advance(Stage::WaitingScrolledFrame);
            }
            Stage::WaitingScrolledFrame => {
                let page = self.capture_step(app, framebuffer, source, "02-scrolled.png", "scrolled")?;
                require_visual_change(self.previous_page.as_ref(), &page, "scroll")?;
                self.previous_page = Some(page);

                app.handle_key("+", true, None);
                tracing::info!("GUI_SMOKE_ACTION action=zoom_in status=executed");
                self.advance(Stage::WaitingZoomedFrame);
            }
            Stage::WaitingZoomedFrame => {
                let page = self.capture_step(app, framebuffer, source, "03-zoomed.png", "zoomed")?;
                require_visual_change(self.previous_page.as_ref(), &page, "zoom_in")?;
                self.previous_page = Some(page);

                app.refresh_page();
                tracing::info!("GUI_SMOKE_ACTION action=reload status=executed");
                self.advance(Stage::WaitingReloadedFrame);
            }
            Stage::WaitingReloadedFrame => {
                self.capture_step(app, framebuffer, source, "04-reloaded.png", "reloaded")?;
                tracing::info!(
                    "GUI_SMOKE_COMPLETE url={} steps=load,scroll,zoom_in,reload",
                    self.config.url
                );
                self.stage = Stage::Complete;
                return Ok(true);
            }
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
        app: &BrowserApp,
        framebuffer: &FrameBuffer,
        source: &str,
        filename: &str,
        step: &str,
    ) -> Result<RegionStats, String> {
        let page_region = page_region(app, framebuffer);
        let chrome_height = page_region.y;
        let path = self.config.output_dir.join(filename);
        smoke_capture::capture_presented_frame(
            &path,
            framebuffer,
            PixelRegion {
                x: 0,
                y: 0,
                width: framebuffer.width,
                height: chrome_height,
            },
            page_region,
            "compositor",
            &self.config.url,
            source,
        )?;
        let page_path = self.config.output_dir.join(filename.replace(".png", "-page.png"));
        smoke_capture::capture_region(&page_path, framebuffer, page_region)?;
        let page =
            smoke_capture::analyze_region(framebuffer.width, framebuffer.height, &framebuffer.data, page_region)?;
        tracing::info!(
            "GUI_SMOKE_STEP step={step} status=passed screenshot={}",
            display_path(&path)
        );
        Ok(page)
    }
}

fn page_region(app: &BrowserApp, framebuffer: &FrameBuffer) -> PixelRegion {
    let (x, y, width, height) = app.page_content_rect_for(framebuffer.width, framebuffer.height);
    PixelRegion {
        x: x.floor().max(0.0) as u32,
        y: y.floor().max(0.0) as u32,
        width: width.ceil().max(0.0) as u32,
        height: height.ceil().max(0.0) as u32,
    }
}

fn visible_page_stats(app: &BrowserApp, framebuffer: &FrameBuffer) -> Result<Option<RegionStats>, String> {
    let stats = smoke_capture::analyze_region(
        framebuffer.width,
        framebuffer.height,
        &framebuffer.data,
        page_region(app, framebuffer),
    )?;
    if stats.validate_visible("page").is_err() || stats.dark_ratio < 0.0002 {
        return Ok(None);
    }
    Ok(Some(stats))
}

fn require_visual_change(before: Option<&RegionStats>, after: &RegionStats, action: &str) -> Result<(), String> {
    let before = before.ok_or_else(|| format!("{action} has no baseline frame"))?;
    let changed_samples = before
        .signature
        .iter()
        .zip(&after.signature)
        .filter(|(left, right)| left.abs_diff(**right) >= 2)
        .count();
    if changed_samples < 4 {
        return Err(format!(
            "{action} did not visibly change the page: changed_samples={changed_samples}/{}",
            before.signature.len()
        ));
    }
    tracing::info!(
        "GUI_SMOKE_ASSERT action={action} visual_change=passed changed_samples={changed_samples}/{}",
        before.signature.len()
    );
    Ok(())
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
    fn config_requires_real_web_url_and_output_directory() {
        assert!(GuiSmokeConfig::new("https://www.iana.org/domains/reserved".into(), "target/gui".into()).is_ok());
        assert!(GuiSmokeConfig::new("file:///tmp/form.html".into(), "target/gui".into()).is_ok());
        assert!(GuiSmokeConfig::new("zero://newtab".into(), "target/gui".into()).is_err());
        assert!(GuiSmokeConfig::new("https://example.com".into(), PathBuf::new()).is_err());
    }

    #[test]
    fn visual_change_requires_multiple_changed_samples() {
        let baseline = stats(vec![100; 64]);
        let mut changed = vec![100; 64];
        changed[..4].fill(110);
        assert!(require_visual_change(Some(&baseline), &stats(changed), "scroll").is_ok());

        let mut unchanged = vec![100; 64];
        unchanged[..3].fill(110);
        assert!(require_visual_change(Some(&baseline), &stats(unchanged), "scroll").is_err());
    }
}
