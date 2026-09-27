//! M3 右键上下文菜单 + 缩放联动真实窗口 smoke 的分步状态机。
//!
//! 与既有 smoke 同模式：真实输入路径驱动——页面区右键弹菜单、菜单行真实点击分发
//! （reload / inspect）、拖拽选中文本后 Selection 菜单复制（arboard 剪贴板读回断言）、
//! Ctrl± 缩放（shell 缩放状态 + 页面区 reflow 像素签名断言）。状态断言经
//! app_smoke_state 只读面。

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use zero_render_foundation::surface::FrameBuffer;

use crate::app::BrowserApp;
use crate::layout;
use crate::smoke_capture::{self, PixelRegion, RegionStats};

const STEP_TIMEOUT: Duration = Duration::from_secs(60);
const MENU_PATH: &str = "/menu.html";

/// 菜单/缩放 smoke 配置。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuZoomSmokeConfig {
    /// 本地 fixture 服务 origin（如 http://127.0.0.1:PORT）。
    pub base: String,
    /// 分步截图输出目录。
    pub output_dir: PathBuf,
}

impl MenuZoomSmokeConfig {
    /// 创建并校验 smoke 配置。
    pub fn new(base: String, output_dir: PathBuf) -> Result<Self, String> {
        if !base.starts_with("http://") && !base.starts_with("https://") {
            return Err("--menu-zoom-smoke-base requires an http:// or https:// origin".to_string());
        }
        if output_dir.as_os_str().is_empty() {
            return Err("--menu-zoom-smoke-dir requires a directory path".to_string());
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
    /// 文本页加载（选择/检查元素的目标）。
    WaitingSeedLoad,
    /// 右键弹出 Page 菜单：可见性 + 条目断言 + 截图，随后点击 reload 行。
    WaitingMenuOpen,
    /// 菜单 reload 分发落地：菜单关闭 + 导航 epoch 前进 + URL 不变。
    WaitingMenuReload,
    /// 再次右键 → 点击 inspect 行：检查元素新标签打开。
    WaitingMenuOpen2,
    /// inspect 标签落地断言 + 截图，随后 Ctrl+W 关闭返回文本页。
    WaitingInspectTab,
    /// 拖拽选中页面文本 → 右键 Selection 菜单 → 点击 copy 行。
    WaitingSelectionMade,
    /// Selection 菜单断言 + copy 分发 → 剪贴板非空断言。
    WaitingSelectionMenu,
    /// 记录页面区基线 → Ctrl+＋ 放大。
    WaitingZoomBaseline,
    /// 缩放状态 > 1 + 页面区 reflow 像素签名变化断言 + 截图 → Ctrl+0 复位。
    WaitingZoomedIn,
    /// 复位断言（缩放回 1.0）。
    WaitingZoomReset,
    Complete,
}

/// 菜单/缩放 smoke 执行状态。
pub struct MenuZoomSmoke {
    config: MenuZoomSmokeConfig,
    stage: Stage,
    deadline: Instant,
    pre_reload_epoch: u64,
    seed_tab_id: Option<zero_browser_shell::TabId>,
    page_baseline: Option<RegionStats>,
}

impl MenuZoomSmoke {
    /// 创建尚未启动的 smoke。
    pub fn new(config: MenuZoomSmokeConfig) -> Self {
        Self {
            config,
            stage: Stage::Pending,
            deadline: Instant::now() + STEP_TIMEOUT,
            pre_reload_epoch: 0,
            seed_tab_id: None,
            page_baseline: None,
        }
    }

    /// 在浏览器窗口与默认标签页就绪后开始流程。
    pub fn start(&mut self, app: &mut BrowserApp) {
        if self.stage != Stage::Pending {
            return;
        }
        tracing::info!("MZ_SMOKE_START base={}", self.config.base);
        app.navigate_to(&self.config.url(MENU_PATH));
        self.advance(Stage::WaitingSeedLoad);
    }

    /// 检查当前步骤是否超过内部墙钟上限。
    pub fn check_timeout(&self) -> Result<(), String> {
        if self.stage != Stage::Complete && Instant::now() > self.deadline {
            return Err(format!("Menu/zoom smoke timed out in stage {:?}", self.stage));
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
                let expected = self.config.url(MENU_PATH);
                let url = app
                    .smoke_tab_url(active)
                    .ok_or_else(|| "seed page has no snapshot url".to_string())?;
                if url != expected {
                    return Err(format!("seed page url is {url}, expected {expected}"));
                }
                self.seed_tab_id = Some(active);
                tracing::info!("MZ_SMOKE_STEP step=seed-load status=passed");

                // 页面区右键（真实输入路径）→ Page 上下文菜单。
                let (x, y) = self.page_center(framebuffer);
                app.handle_mouse_click(x, y, true, "Right");
                if !app.smoke_context_menu_visible() {
                    return Err("right click did not open the context menu".to_string());
                }
                let ids = app.smoke_context_menu_item_ids();
                for required in ["reload", "inspect"] {
                    if !ids.iter().any(|id| id == required) {
                        return Err(format!("context menu misses {required}: {ids:?}"));
                    }
                }
                self.capture_step(app, framebuffer, source, "01-context-menu.png", "context-menu")?;
                tracing::info!("MZ_SMOKE_STEP step=context-menu status=passed items={ids:?}");

                // 点击 reload 行（真实行内点击，hover 由点击路径按坐标解析）。
                // 基线必须先于点击记录——refresh_page 可能同步推进导航 epoch。
                self.pre_reload_epoch = app.smoke_navigation_epoch(active);
                self.click_menu_item(app, "reload", framebuffer)?;
                tracing::info!("MZ_SMOKE_ACTION action=menu-reload status=executed");
                self.advance(Stage::WaitingMenuReload);
            }
            Stage::WaitingMenuReload => {
                if app.smoke_context_menu_visible() {
                    return Ok(false); // 等待菜单关闭传播
                }
                let url = app.smoke_tab_url(active).unwrap_or_default();
                if url != self.config.url(MENU_PATH) {
                    return Err(format!("menu reload changed the url to {url}"));
                }
                let epoch = app.smoke_navigation_epoch(active);
                if epoch <= self.pre_reload_epoch {
                    return Ok(false); // 重载尚未落地
                }
                tracing::info!("MZ_SMOKE_STEP step=menu-reload status=passed epoch={epoch}");

                // 再次右键 → inspect 行 → 检查元素新标签。
                let (x, y) = self.page_center(framebuffer);
                app.handle_mouse_click(x, y, true, "Right");
                if !app.smoke_context_menu_visible() {
                    return Err("second right click did not open the context menu".to_string());
                }
                self.click_menu_item(app, "inspect", framebuffer)?;
                tracing::info!("MZ_SMOKE_ACTION action=menu-inspect status=executed");
                self.advance(Stage::WaitingMenuOpen2);
            }
            Stage::WaitingMenuOpen2 => {
                let Some(seed) = self.seed_tab_id else {
                    return Err("seed tab missing".to_string());
                };
                if app.smoke_active_tab_id() == Some(seed) {
                    return Ok(false); // inspect 标签尚未打开
                }
                let url = app.smoke_tab_url(active).unwrap_or_default();
                if !url.starts_with("zero://inspect") {
                    return Ok(false); // 内部检查页加载中
                }
                self.capture_step(app, framebuffer, source, "02-inspect-tab.png", "inspect-tab")?;
                tracing::info!("MZ_SMOKE_STEP step=inspect-tab status=passed url={url}");

                // Ctrl+W 关闭检查元素标签，回到文本页。
                app.handle_key("Control", true, None);
                app.handle_key("w", true, None);
                app.handle_key("w", false, None);
                app.handle_key("Control", false, None);
                self.advance(Stage::WaitingInspectTab);
            }
            Stage::WaitingInspectTab => {
                let Some(seed) = self.seed_tab_id else {
                    return Err("seed tab missing".to_string());
                };
                if app.smoke_active_tab_id() != Some(seed) {
                    return Ok(false); // 关闭传播中
                }
                tracing::info!("MZ_SMOKE_STEP step=inspect-closed status=passed");

                // 拖拽选中文本（真实拖拽路径）：页面文本区按下 → 右移 → 释放。
                let (x, y) = self.page_center(framebuffer);
                let start_x = x - 160.0;
                app.handle_mouse_click(start_x, y, true, "Left");
                for step in 1..=8 {
                    let px = start_x + 220.0 * (step as f64) / 8.0;
                    app.handle_mouse_move(px, y);
                }
                app.handle_mouse_click(start_x + 220.0, y, false, "Left");
                tracing::info!("MZ_SMOKE_ACTION action=drag-select status=executed");
                self.advance(Stage::WaitingSelectionMade);
            }
            Stage::WaitingSelectionMade => {
                // 选区落地后右键 → Selection 菜单（copy 行）。
                if app.smoke_context_menu_visible() {
                    return Err("context menu unexpectedly open before selection right click".to_string());
                }
                let (x, y) = self.page_center(framebuffer);
                app.handle_mouse_click(x, y, true, "Right");
                if !app.smoke_context_menu_visible() {
                    return Err("right click after selection did not open the context menu".to_string());
                }
                let ids = app.smoke_context_menu_item_ids();
                if !ids.iter().any(|id| id == "copy") {
                    return Err(format!("selection menu misses copy: {ids:?}"));
                }
                self.click_menu_item(app, "copy", framebuffer)?;
                tracing::info!("MZ_SMOKE_ACTION action=menu-copy status=executed");
                self.advance(Stage::WaitingSelectionMenu);
            }
            Stage::WaitingSelectionMenu => {
                if app.smoke_context_menu_visible() {
                    return Ok(false);
                }
                let Some(text) = app.smoke_clipboard_text() else {
                    return Err("clipboard empty after selection copy (arboard write failed?)".to_string());
                };
                if text.trim().is_empty() {
                    return Err("clipboard text empty after selection copy".to_string());
                }
                tracing::info!("MZ_SMOKE_STEP step=selection-copy status=passed bytes={}", text.len());

                // 记录页面区基线，Ctrl+＋ 放大。
                self.page_baseline = Some(self.page_region_stats(framebuffer));
                app.handle_key("Control", true, None);
                app.handle_key("+", true, None);
                app.handle_key("+", false, None);
                app.handle_key("Control", false, None);
                tracing::info!("MZ_SMOKE_ACTION action=zoom-in status=executed");
                self.advance(Stage::WaitingZoomBaseline);
            }
            Stage::WaitingZoomBaseline => {
                let zoom = app.smoke_page_zoom();
                if zoom <= 1.0 {
                    return Ok(false); // 缩放尚未落地
                }
                let after = self.page_region_stats(framebuffer);
                let changed = crate::gui_smoke::require_visual_change(self.page_baseline.as_ref(), &after, "zoom_in");
                if changed.is_err() {
                    return Ok(false); // reflow 帧未到，等下一帧
                }
                self.capture_step(app, framebuffer, source, "03-zoomed.png", "zoomed")?;
                tracing::info!("MZ_SMOKE_STEP step=zoom-in status=passed zoom={zoom}");

                app.handle_key("Control", true, None);
                app.handle_key("0", true, None);
                app.handle_key("0", false, None);
                app.handle_key("Control", false, None);
                self.advance(Stage::WaitingZoomedIn);
            }
            Stage::WaitingZoomedIn => {
                let zoom = app.smoke_page_zoom();
                if (zoom - 1.0).abs() > f32::EPSILON {
                    return Ok(false); // 复位尚未落地
                }
                tracing::info!(
                    "MZ_SMOKE_COMPLETE base={} steps=menu-open,menu-reload,menu-inspect,selection-copy,zoom-in,zoom-reset",
                    self.config.base
                );
                self.stage = Stage::Complete;
                return Ok(true);
            }
            // 入口已提前返回 Pending/Complete；WaitingMenuOpen 折叠进 SeedLoad、
            // WaitingZoomReset 折叠进 ZoomedIn（按键+断言+截图同帧完成）。
            Stage::Pending | Stage::WaitingMenuOpen | Stage::WaitingZoomReset | Stage::Complete => {}
        }

        Ok(false)
    }

    fn advance(&mut self, stage: Stage) {
        self.stage = stage;
        self.deadline = Instant::now() + STEP_TIMEOUT;
    }

    fn page_center(&self, framebuffer: &FrameBuffer) -> (f64, f64) {
        // 页面中心偏上（避开可能的滚动条/底部覆盖层），仍在整页链接/文本区内。
        let (x, y, w, h) = (0.0_f64, 0.0_f64, framebuffer.width as f64, framebuffer.height as f64);
        (x + w * 0.5, y + h * 0.4)
    }

    /// 点击上下文菜单指定 id 的行（真实鼠标点击；行几何与渲染/命中测试同源：
    /// 普通行 CONTEXT_MENU_ROW_HEIGHT，分隔行 CONTEXT_MENU_SEPARATOR_HEIGHT 累积）。
    fn click_menu_item(&self, app: &mut BrowserApp, id: &str, framebuffer: &FrameBuffer) -> Result<(), String> {
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
            return Err(format!("context menu misses {id}: {ids:?}"));
        };
        let x = f64::from(mx + 20.0 * s);
        let y = f64::from(my + center);
        let _ = framebuffer;
        app.handle_mouse_move(x, y);
        app.handle_mouse_click(x, y, true, "Left");
        app.handle_mouse_click(x, y, false, "Left");
        Ok(())
    }

    fn page_region_stats(&self, framebuffer: &FrameBuffer) -> RegionStats {
        let region = PixelRegion {
            x: 0,
            y: 0,
            width: framebuffer.width,
            height: framebuffer.height,
        };
        smoke_capture::analyze_region(framebuffer.width, framebuffer.height, &framebuffer.data, region).unwrap_or_else(
            |_| RegionStats {
                pixels: 0,
                opaque_pixels: 0,
                unique_bins: 0,
                dominant_ratio: 0.0,
                luma_min: 0,
                luma_max: 0,
                dark_pixels: 0,
                dark_ratio: 0.0,
                signature: Vec::new(),
            },
        )
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
            "MZ_SMOKE_STEP step={step} screenshot={} status=captured",
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
        assert!(MenuZoomSmokeConfig::new("http://127.0.0.1:9".into(), "target/mz".into()).is_ok());
        assert!(MenuZoomSmokeConfig::new("zero://newtab".into(), "target/mz".into()).is_err());
        assert!(MenuZoomSmokeConfig::new("http://127.0.0.1:9".into(), PathBuf::new()).is_err());
    }
}
