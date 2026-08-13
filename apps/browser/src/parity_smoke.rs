//! Chrome 一致性验收的真实窗口交互与 production GPU 证据生产器。

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use zero_browser_shell::TabId;
use zero_render_foundation::surface::FrameBuffer;

use crate::app::BrowserApp;
use crate::smoke_capture::{self, PixelRegion};

const STEP_TIMEOUT: Duration = Duration::from_secs(30);
const HIT_SCAN_STEP: u32 = 2;
const TITLE_STATE_PREFIX: &str = "ZERO_TEST_STATE:";

/// 一致性场景配置。
#[derive(Clone, Debug)]
pub struct ParitySmokeConfig {
    scenario: Scenario,
    output_dir: PathBuf,
}

impl ParitySmokeConfig {
    /// 从 Skill 场景文件加载并校验配置。
    pub fn load(scenario_path: PathBuf, output_dir: PathBuf) -> Result<Self, String> {
        let source = std::fs::read_to_string(&scenario_path)
            .map_err(|error| format!("failed to read parity scenario {}: {error}", scenario_path.display()))?;
        let mut scenario: Scenario = serde_json::from_str(&source)
            .map_err(|error| format!("failed to parse parity scenario {}: {error}", scenario_path.display()))?;
        if scenario.version != 1 {
            return Err(format!("unsupported parity scenario version: {}", scenario.version));
        }
        if scenario.steps.is_empty() {
            return Err("parity scenario requires at least one step".to_string());
        }
        if scenario.viewport.width == 0 || scenario.viewport.height == 0 || scenario.viewport.dpr != 1.0 {
            return Err("production parity currently requires a non-empty DPR=1 viewport".to_string());
        }
        scenario.url = expand_repo_url(&scenario.url)?;
        Ok(Self { scenario, output_dir })
    }

    /// 场景页面 URL。
    pub fn url(&self) -> &str {
        &self.scenario.url
    }

    /// 场景要求的页面 viewport。
    pub fn viewport(&self) -> (u32, u32) {
        (self.scenario.viewport.width, self.scenario.viewport.height)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Scenario {
    version: u32,
    name: String,
    url: String,
    viewport: Viewport,
    observe: Observe,
    steps: Vec<ScenarioStep>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct Viewport {
    width: u32,
    height: u32,
    dpr: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Observe {
    selectors: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ScenarioStep {
    id: String,
    action: ScenarioAction,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ScenarioAction {
    Snapshot,
    Click {
        selector: String,
        #[serde(default)]
        offset: Option<Point>,
        #[serde(default)]
        jitter: Option<Point>,
    },
    Type {
        text: String,
    },
    Key {
        key: String,
    },
    Wait {
        milliseconds: u64,
    },
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    schema_version: u32,
    scenario: String,
    engine: &'static str,
    engine_version: &'static str,
    capture_path: &'static str,
    input_path: &'static str,
    viewport: Viewport,
    steps: Vec<ManifestStep>,
}

#[derive(Clone, Debug, Serialize)]
struct ManifestStep {
    id: String,
    action: ScenarioAction,
    screenshot: String,
    regions: HashMap<String, String>,
    state: Value,
    events: Vec<Value>,
    geometry: HashMap<String, Geometry>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct Geometry {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

/// 真实窗口一致性场景执行状态。
pub struct ParitySmoke {
    config: ParitySmokeConfig,
    started: bool,
    next_step: usize,
    deadline: Instant,
    captured: Vec<ManifestStep>,
    stable_geometry: Option<HashMap<String, Geometry>>,
    initial_frame_retry_requested: bool,
}

impl ParitySmoke {
    /// 创建未启动的场景。
    pub fn new(config: ParitySmokeConfig) -> Self {
        Self {
            config,
            started: false,
            next_step: 0,
            deadline: Instant::now() + STEP_TIMEOUT,
            captured: Vec::new(),
            stable_geometry: None,
            initial_frame_retry_requested: false,
        }
    }

    /// 浏览器窗口就绪后导航到目标页面。
    pub fn start(&mut self, app: &mut BrowserApp) {
        if self.started {
            return;
        }
        self.started = true;
        tracing::info!("PARITY_SMOKE_NAVIGATE url={}", self.config.url());
        app.navigate_to(self.config.url());
        self.deadline = Instant::now() + STEP_TIMEOUT;
    }

    /// 检查当前动作是否超时。
    pub fn check_timeout(&self) -> Result<(), String> {
        if self.next_step < self.config.scenario.steps.len() && Instant::now() > self.deadline {
            return Err(format!(
                "parity smoke timed out before step {}",
                self.config.scenario.steps[self.next_step].id
            ));
        }
        Ok(())
    }

    /// 消费一次真实呈现并严格 GPU readback 的产品帧。
    pub fn on_presented_frame(
        &mut self,
        app: &mut BrowserApp,
        framebuffer: &FrameBuffer,
        source: &str,
    ) -> Result<bool, String> {
        if !self.started || app.any_tab_loading() {
            return Ok(false);
        }
        if source != "compositor_bitmap" {
            return Err(format!("production parity requires compositor_bitmap, got {source}"));
        }
        let initial_page_region = page_region(app, framebuffer);
        if !visible_page(framebuffer, initial_page_region)? {
            if !self.initial_frame_retry_requested {
                app.sync_webview_viewport();
                app.needs_redraw = true;
                self.initial_frame_retry_requested = true;
                tracing::info!("PARITY_SMOKE_RETRY reason=blank_initial_frame action=resync_viewport");
            }
            return Ok(false);
        }

        while self.next_step < self.config.scenario.steps.len() {
            let step_index = self.next_step;
            if matches!(self.config.scenario.steps[step_index].action, ScenarioAction::Snapshot) {
                let tab_id = app
                    .parity_active_tab_id()
                    .ok_or_else(|| "parity smoke has no active tab".to_string())?;
                let Ok(current_state) = serialized_state(app, tab_id) else {
                    if !self.initial_frame_retry_requested {
                        app.sync_webview_viewport();
                        self.initial_frame_retry_requested = true;
                        tracing::info!("PARITY_SMOKE_RETRY reason=state_unavailable action=resync_viewport");
                    }
                    app.needs_redraw = true;
                    return Ok(false);
                };
                let baseline_frame_id = app.parity_compositor_frame_id(tab_id);
                app.sync_webview_viewport();
                let settle_deadline = Instant::now() + Duration::from_secs(2);
                while app.parity_compositor_frame_id(tab_id) <= baseline_frame_id {
                    if Instant::now() > settle_deadline {
                        return Ok(false);
                    }
                    std::thread::sleep(Duration::from_millis(5));
                    app.poll_tab_fetch();
                }
                let fresh_frame = app.render_full_scene_gpu_capture(app.physical_size.0, app.physical_size.1)?;
                if !visible_page(&fresh_frame, page_region(app, &fresh_frame))? {
                    return Ok(false);
                }
                self.capture_step(app, &fresh_frame, step_index, &current_state)?;
                self.next_step += 1;
                continue;
            }
            let tab_id = app
                .parity_active_tab_id()
                .ok_or_else(|| "parity smoke has no active tab".to_string())?;
            let baseline_state = serialized_state(app, tab_id)?;
            self.perform_action(app, tab_id, step_index)?;
            self.deadline = Instant::now() + STEP_TIMEOUT;
            let mut settled_state = loop {
                app.poll_tab_fetch();
                if let Ok(state) = serialized_state(app, tab_id)
                    && state != baseline_state
                {
                    break state;
                }
                if Instant::now() > self.deadline {
                    return Err(format!(
                        "parity action did not change state before timeout: {}",
                        self.config.scenario.steps[step_index].id
                    ));
                }
                std::thread::sleep(Duration::from_millis(5));
            };
            let settle_deadline = Instant::now() + Duration::from_secs(2);
            let mut stable_polls = 0;
            while stable_polls < 20 {
                std::thread::sleep(Duration::from_millis(5));
                app.poll_tab_fetch();
                let state = serialized_state(app, tab_id)?;
                if state == settled_state {
                    stable_polls += 1;
                } else {
                    settled_state = state;
                    stable_polls = 0;
                }
                if Instant::now() > settle_deadline {
                    return Err(format!(
                        "parity state did not stabilize after step {}",
                        self.config.scenario.steps[step_index].id
                    ));
                }
            }
            let mut fresh_frame = None;
            for _ in 0..3 {
                let baseline_frame_id = app.parity_compositor_frame_id(tab_id);
                app.sync_webview_viewport();
                let settle_deadline = Instant::now() + Duration::from_secs(2);
                while app.parity_compositor_frame_id(tab_id) <= baseline_frame_id {
                    if Instant::now() > settle_deadline {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(5));
                    app.poll_tab_fetch();
                }
                let candidate = app.render_full_scene_gpu_capture(app.physical_size.0, app.physical_size.1)?;
                if visible_page(&candidate, page_region(app, &candidate))? {
                    fresh_frame = Some(candidate);
                    break;
                }
            }
            let fresh_frame = fresh_frame.ok_or_else(|| {
                format!(
                    "full-frame resync did not produce a visible page after step {}",
                    self.config.scenario.steps[step_index].id
                )
            })?;
            settled_state = serialized_state(app, tab_id)?;
            self.capture_step(app, &fresh_frame, step_index, &settled_state)?;
            self.next_step += 1;
        }

        self.write_manifest()?;
        tracing::info!(
            "PARITY_SMOKE_COMPLETE scenario={} steps={} source={source}",
            self.config.scenario.name,
            self.captured.len()
        );
        Ok(true)
    }

    fn perform_action(&self, app: &mut BrowserApp, tab_id: TabId, step_index: usize) -> Result<(), String> {
        let step = &self.config.scenario.steps[step_index];
        match &step.action {
            ScenarioAction::Snapshot => {}
            ScenarioAction::Click {
                selector,
                offset,
                jitter,
            } => {
                let geometry = scan_geometry(app, tab_id, std::slice::from_ref(selector))?
                    .remove(selector)
                    .ok_or_else(|| format!("no hit-test geometry for {selector}"))?;
                let offset = offset.unwrap_or(Point { x: 0.5, y: 0.5 });
                let document_x = geometry.x + geometry.width * offset.x;
                let document_y = geometry.y + geometry.height * offset.y;
                let (content_x, content_y, _, _) = app.page_content_rect();
                let physical_x = f64::from(content_x + document_x * app.scale_factor);
                let physical_y = f64::from(content_y + document_y * app.scale_factor);
                let jitter = jitter.unwrap_or_default();
                app.handle_mouse_move(physical_x, physical_y);
                app.handle_mouse_click(physical_x, physical_y, true, "Left");
                app.handle_mouse_move(physical_x + f64::from(jitter.x), physical_y + f64::from(jitter.y));
                app.handle_mouse_click(
                    physical_x + f64::from(jitter.x),
                    physical_y + f64::from(jitter.y),
                    false,
                    "Left",
                );
            }
            ScenarioAction::Type { text } => {
                for character in text.chars() {
                    app.handle_key(&character.to_string(), true, None);
                }
            }
            ScenarioAction::Key { key } => app.handle_key(key, true, None),
            ScenarioAction::Wait { milliseconds } => {
                std::thread::sleep(Duration::from_millis(*milliseconds));
                app.needs_redraw = true;
            }
        }
        tracing::info!("PARITY_SMOKE_ACTION step={} action={:?}", step.id, step.action);
        Ok(())
    }

    fn capture_step(
        &mut self,
        app: &mut BrowserApp,
        framebuffer: &FrameBuffer,
        step_index: usize,
        serialized_state: &str,
    ) -> Result<(), String> {
        let step = &self.config.scenario.steps[step_index];
        let tab_id = app
            .parity_active_tab_id()
            .ok_or_else(|| "parity smoke has no active tab".to_string())?;
        let mut state: Value = serde_json::from_str(serialized_state)
            .map_err(|error| format!("invalid parity state in HTML snapshot: {error}"))?;
        let events = state
            .get("events")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if let Some(state) = state.as_object_mut() {
            state.remove("events");
        }
        let geometry = match &self.stable_geometry {
            Some(geometry) => geometry.clone(),
            None => {
                let geometry = scan_geometry(app, tab_id, &self.config.scenario.observe.selectors)?;
                self.stable_geometry = Some(geometry.clone());
                geometry
            }
        };
        let page_region = page_region(app, framebuffer);
        let screenshot = format!("{}.png", step.id);
        smoke_capture::capture_region(&self.config.output_dir.join(&screenshot), framebuffer, page_region)?;

        let mut regions = HashMap::new();
        for (selector, rect) in &geometry {
            let filename = format!("{}.region-{}.png", step.id, hex_name(selector));
            let region = PixelRegion {
                x: page_region.x.saturating_add(rect.x.floor().max(0.0) as u32),
                y: page_region.y.saturating_add(rect.y.floor().max(0.0) as u32),
                width: (rect.x + rect.width).ceil().max(0.0) as u32 - rect.x.floor().max(0.0) as u32,
                height: (rect.y + rect.height).ceil().max(0.0) as u32 - rect.y.floor().max(0.0) as u32,
            };
            smoke_capture::capture_region(&self.config.output_dir.join(&filename), framebuffer, region)?;
            regions.insert(selector.clone(), filename);
        }

        self.captured.push(ManifestStep {
            id: step.id.clone(),
            action: step.action.clone(),
            screenshot,
            regions,
            state,
            events,
            geometry,
        });
        tracing::info!("PARITY_SMOKE_STEP step={} status=captured", step.id);
        Ok(())
    }

    fn write_manifest(&self) -> Result<(), String> {
        std::fs::create_dir_all(&self.config.output_dir)
            .map_err(|error| format!("failed to create parity output directory: {error}"))?;
        let manifest = Manifest {
            schema_version: 1,
            scenario: self.config.scenario.name.clone(),
            engine: "zeroweb",
            engine_version: zero_product_version::VERSION,
            capture_path: "production-window-gpu",
            input_path: "browser-pointer",
            viewport: self.config.scenario.viewport,
            steps: self.captured.clone(),
        };
        let data = serde_json::to_vec_pretty(&manifest)
            .map_err(|error| format!("failed to serialize parity manifest: {error}"))?;
        std::fs::write(self.config.output_dir.join("manifest.json"), data)
            .map_err(|error| format!("failed to write parity manifest: {error}"))
    }
}

fn expand_repo_url(input: &str) -> Result<String, String> {
    const PREFIX: &str = "file://${REPO_ROOT}/";
    if let Some(relative) = input.strip_prefix(PREFIX) {
        let (path, query) = relative
            .split_once('?')
            .map_or((relative, None), |(path, query)| (path, Some(query)));
        let root = std::env::var_os("PARITY_REPO_ROOT")
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| "failed to resolve parity repository root".to_string())?;
        let mut url = url::Url::from_file_path(root.join(path))
            .map_err(|_| "failed to convert parity fixture path to file URL".to_string())?;
        url.set_query(query);
        return Ok(url.to_string());
    }
    Ok(input.replace(
        "${REPO_ROOT}",
        &std::env::current_dir().unwrap_or_default().to_string_lossy(),
    ))
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

fn visible_page(framebuffer: &FrameBuffer, region: PixelRegion) -> Result<bool, String> {
    let stats = smoke_capture::analyze_region(framebuffer.width, framebuffer.height, &framebuffer.data, region)?;
    Ok(stats.opaque_pixels > 0 && stats.luma_max.saturating_sub(stats.luma_min) >= 12)
}

fn serialized_state(app: &BrowserApp, tab_id: TabId) -> Result<String, String> {
    if let Some(state) = app
        .parity_page_title(tab_id)
        .and_then(|title| title.strip_prefix(TITLE_STATE_PREFIX).map(str::to_string))
    {
        return Ok(state);
    }
    if let Some(html) = app.parity_page_html(tab_id) {
        let state = zero_engine::query_text_from_html(&html, "#test-state");
        if !state.is_empty() {
            return Ok(state);
        }
    }
    Err("parity state is unavailable from title and HTML snapshot".to_string())
}

fn scan_geometry(
    app: &mut BrowserApp,
    tab_id: TabId,
    selectors: &[String],
) -> Result<HashMap<String, Geometry>, String> {
    let wanted = selectors
        .iter()
        .map(|selector| {
            selector
                .strip_prefix('#')
                .map(|id| (id.to_string(), selector.clone()))
                .ok_or_else(|| format!("ZeroWeb parity currently requires id selectors: {selector}"))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;
    let (_, _, physical_width, physical_height) = app.page_content_rect();
    let logical_width = (physical_width / app.scale_factor).ceil().max(0.0) as u32;
    let logical_height = (physical_height / app.scale_factor).ceil().max(0.0) as u32;
    let mut result = HashMap::new();
    for y in (0..logical_height).step_by(HIT_SCAN_STEP as usize) {
        for x in (0..logical_width).step_by(HIT_SCAN_STEP as usize) {
            let Some(hit) = app.parity_hit_test_element(tab_id, x as f32, y as f32) else {
                continue;
            };
            let Some(id) = hit.id else {
                continue;
            };
            if !wanted.contains_key(&id) {
                continue;
            }
            let selector = wanted.get(&id).expect("wanted id exists").clone();
            result.entry(selector).or_insert(Geometry {
                x: hit.x,
                y: hit.y,
                width: hit.width,
                height: hit.height,
            });
            if result.len() == wanted.len() {
                return Ok(result);
            }
        }
    }
    Ok(result)
}

fn hex_name(selector: &str) -> String {
    selector.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_name_matches_skill_contract() {
        assert_eq!(hex_name("#name"), "236e616d65");
    }

    #[test]
    fn sparse_control_page_is_visible() {
        let mut frame = FrameBuffer::new_filled(100, 100, 255, 255, 255, 255);
        for x in 10..90 {
            let offset = ((10 * frame.width + x) * 4) as usize;
            frame.data[offset..offset + 4].copy_from_slice(&[128, 128, 128, 255]);
        }
        assert!(
            visible_page(
                &frame,
                PixelRegion {
                    x: 0,
                    y: 0,
                    width: 100,
                    height: 100,
                }
            )
            .unwrap()
        );
    }
}
