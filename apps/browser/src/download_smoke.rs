//! M3 下载管理器真实窗口 smoke（触发下载 / 进度记账 / 在文件夹中显示）的分步状态机。
//!
//! 与 tab/addressbar smoke 同模式：真实输入路径驱动——页面链接点击触发 attachment
//! 响应（本地 fixture 服务器带 Content-Disposition），下载经 backend 拦截 → 应用层
//! 落盘 → 下载管理器记账 → 面板/下载页 UI 呈现；「Show in folder」经面板按钮真实
//! 点击触发。状态断言经 app_smoke_state 只读面。

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use zero_render_foundation::surface::FrameBuffer;

use crate::app::BrowserApp;
use crate::smoke_capture::{self, PixelRegion};

const STEP_TIMEOUT: Duration = Duration::from_secs(60);
const PAGE_PATH: &str = "/page.html";
const FILE_PATH: &str = "/file.zip";

/// 下载 smoke 配置。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownloadSmokeConfig {
    /// 本地 fixture 服务 origin（如 http://127.0.0.1:PORT）。
    pub base: String,
    /// 分步截图输出目录。
    pub output_dir: PathBuf,
}

impl DownloadSmokeConfig {
    /// 创建并校验 smoke 配置。
    pub fn new(base: String, output_dir: PathBuf) -> Result<Self, String> {
        if !base.starts_with("http://") && !base.starts_with("https://") {
            return Err("--download-smoke-base requires an http:// or https:// origin".to_string());
        }
        if output_dir.as_os_str().is_empty() {
            return Err("--download-smoke-dir requires a directory path".to_string());
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
    /// 下载触发页加载（整页大链接），随后点击链接触发下载。
    WaitingPageLoad,
    /// 下载完成断言（管理器状态 + 文件落盘字节 + 面板可见），随后开下载页。
    WaitingDownloadDone,
    /// zero://downloads 页呈现断言 + 截图，随后点击面板「Show in folder」。
    WaitingDownloadsPage,
    /// 「在文件夹中显示」动作断言（最近打开目录 == 下载目录）。
    WaitingFolderOpen,
    Complete,
}

/// 下载 smoke 执行状态。
pub struct DownloadSmoke {
    config: DownloadSmokeConfig,
    stage: Stage,
    deadline: Instant,
}

impl DownloadSmoke {
    /// 创建尚未启动的 smoke。
    pub fn new(config: DownloadSmokeConfig) -> Self {
        Self {
            config,
            stage: Stage::Pending,
            deadline: Instant::now() + STEP_TIMEOUT,
        }
    }

    /// 在浏览器窗口与默认标签页就绪后开始流程。
    pub fn start(&mut self, app: &mut BrowserApp) {
        if self.stage != Stage::Pending {
            return;
        }
        tracing::info!("DL_SMOKE_START base={}", self.config.base);
        // 下载落盘隔离到演示流目录（避免写用户真实 Downloads）；流程结束前恢复空值。
        let target = self.config.output_dir.join("files");
        app.shell
            .apply_settings(|settings| settings.download_directory = target.display().to_string());
        app.navigate_to(&self.config.url(PAGE_PATH));
        self.advance(Stage::WaitingPageLoad);
    }

    /// 检查当前步骤是否超过内部墙钟上限。
    pub fn check_timeout(&self) -> Result<(), String> {
        if self.stage != Stage::Complete && Instant::now() > self.deadline {
            return Err(format!("Download smoke timed out in stage {:?}", self.stage));
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
        // 每个阶段在同一帧内完成断言/截图/下一步输入后，也要保证还有下一帧可消费。
        app.needs_redraw = true;
        let active = app.smoke_active_tab_id().ok_or_else(|| "no active tab".to_string())?;

        match self.stage {
            Stage::WaitingPageLoad => {
                let expected = self.config.url(PAGE_PATH);
                let url = app
                    .smoke_tab_url(active)
                    .ok_or_else(|| "page has no snapshot url".to_string())?;
                if url != expected {
                    return Err(format!("seed page url is {url}, expected {expected}"));
                }
                tracing::info!("DL_SMOKE_STEP step=page-load status=passed");

                // 整页大链接：页面区中心点击即命中（真实输入路径触发下载）。
                let (px, py, pw, ph) = app.page_content_rect_for(framebuffer.width, framebuffer.height);
                let x = f64::from(px + pw * 0.5);
                let y = f64::from(py + ph * 0.5);
                app.handle_mouse_click(x, y, true, "Left");
                app.handle_mouse_click(x, y, false, "Left");
                tracing::info!("DL_SMOKE_ACTION action=click-download-link status=executed");
                self.advance(Stage::WaitingDownloadDone);
            }
            Stage::WaitingDownloadDone => {
                let file_url = self.config.url(FILE_PATH);
                let downloads = app.smoke_downloads();
                let Some((_url, filename, state, downloaded, total)) =
                    downloads.iter().find(|(url, ..)| url == &file_url)
                else {
                    return Ok(false); // 下载尚未入账，等下一帧
                };
                if state != "Completed" {
                    return Err(format!("download state is {state}, expected Completed"));
                }
                if *total != Some(*downloaded) || *downloaded == 0 {
                    return Err(format!(
                        "download bytes inconsistent: downloaded={downloaded} total={total:?}"
                    ));
                }

                // 文件真实落盘且字节一致。
                let path = std::path::Path::new(&app.smoke_download_dir()).join(filename);
                let on_disk = std::fs::read(&path)
                    .map_err(|err| format!("downloaded file unreadable at {}: {err}", path.display()))?;
                if on_disk.len() as u64 != *downloaded {
                    return Err(format!(
                        "file size {} != recorded bytes {downloaded} at {}",
                        on_disk.len(),
                        path.display()
                    ));
                }
                self.capture_step(app, framebuffer, source, "01-download-panel.png", "download-panel")?;
                tracing::info!(
                    "DL_SMOKE_STEP step=download-done status=passed file={} bytes={downloaded}",
                    path.display()
                );
                if !app.smoke_download_panel_visible() {
                    return Err("download panel not visible after download".to_string());
                }

                // 打开浏览器内下载页（列表呈现）。
                app.open_downloads_page();
                self.advance(Stage::WaitingDownloadsPage);
            }
            Stage::WaitingDownloadsPage => {
                let url = app
                    .smoke_tab_url(active)
                    .ok_or_else(|| "downloads page has no snapshot url".to_string())?;
                if url != "zero://downloads" {
                    return Ok(false); // 内部页加载中
                }
                self.capture_step(app, framebuffer, source, "02-downloads-page.png", "downloads-page")?;
                tracing::info!("DL_SMOKE_STEP step=downloads-page status=passed");

                // 面板「Show in folder」按钮真实点击（矩形与渲染同源）。
                let Some((bx, by, bw, bh)) = app.download_panel_action_rect_for(framebuffer.width, framebuffer.height)
                else {
                    return Err("download panel action button unavailable".to_string());
                };
                let x = f64::from(bx + bw * 0.5);
                let y = f64::from(by + bh * 0.5);
                app.handle_mouse_click(x, y, true, "Left");
                app.handle_mouse_click(x, y, false, "Left");
                tracing::info!("DL_SMOKE_ACTION action=click-show-in-folder status=executed");
                self.advance(Stage::WaitingFolderOpen);
            }
            Stage::WaitingFolderOpen => {
                let dir = app.smoke_download_dir();
                let opened = app
                    .smoke_last_opened_download_dir()
                    .ok_or_else(|| "show-in-folder action did not run".to_string())?;
                if opened != dir {
                    return Err(format!("opened dir {opened:?} != download dir {dir:?}"));
                }
                self.capture_step(app, framebuffer, source, "03-show-in-folder.png", "show-in-folder")?;
                // 恢复用户设置（persist_user_data 在进程退出时落盘，不留演示流目录）。
                app.shell
                    .apply_settings(|settings| settings.download_directory = String::new());
                tracing::info!(
                    "DL_SMOKE_COMPLETE base={} steps=trigger,progress-record,file,on-page,show-in-folder dir={dir}",
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
            "DL_SMOKE_STEP step={step} screenshot={} status=captured",
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
        assert!(DownloadSmokeConfig::new("http://127.0.0.1:9".into(), "target/dl".into()).is_ok());
        assert!(DownloadSmokeConfig::new("zero://newtab".into(), "target/dl".into()).is_err());
        assert!(DownloadSmokeConfig::new("http://127.0.0.1:9".into(), PathBuf::new()).is_err());
    }
}
