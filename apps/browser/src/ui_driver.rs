//! 脚本化真实窗口 UI 驱动器：JSONL 动作序列经真实输入路径驱动浏览器壳层。
//!
//! 与 smoke 家族同模式：动作经 `handle_mouse_click`/`handle_mouse_move`/`handle_key`
//! 进入与 winit 事件相同的处理路径；截图用生产场景 GPU 离屏渲染读回；状态断言经
//! `app_smoke_state` 只读面。供 GUI 兼容性探索与回归复现使用：
//!
//! ```jsonl
//! {"op":"wait_ms","ms":800}
//! {"op":"wait_idle","ms":15000,"settle":400}
//! {"op":"click","x":640,"y":75,"button":"Left"}
//! {"op":"move","x":640,"y":75}
//! {"op":"down","x":40,"y":160}
//! {"op":"move","x":300,"y":160}
//! {"op":"up","x":300,"y":160}
//! {"op":"key","key":"l","mods":["ctrl"]}
//! {"op":"type","text":"http://127.0.0.1:18080/page2.html"}
//! {"op":"shot","name":"01-home.png"}
//! {"op":"assert_url","contains":"page2.html"}
//! {"op":"assert_tabs","count":3}
//! {"op":"assert_url","contains":"view-source:http://…","mode":"prefix"}
//! {"op":"assert_menu","open":true}
//! {"op":"assert_nav","back":true,"forward":false}
//! {"op":"assert_status","contains":"http://127.0.0.1:18080/page2.html"}
//! {"op":"log","msg":"menu opened"}
//! ```

use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::app::BrowserApp;

const DRIVER_TIMEOUT: Duration = Duration::from_secs(600);

/// UI 驱动器配置。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiDriverConfig {
    /// JSONL 动作脚本路径。
    pub script: PathBuf,
    /// 截图与 result.json 输出目录。
    pub output_dir: PathBuf,
}

impl UiDriverConfig {
    /// 创建并校验驱动器配置。
    pub fn new(script: PathBuf, output_dir: PathBuf) -> Result<Self, String> {
        if script.as_os_str().is_empty() {
            return Err("--ui-driver-script requires a JSONL script path".to_string());
        }
        if output_dir.as_os_str().is_empty() {
            return Err("--ui-driver-out requires a directory path".to_string());
        }
        Ok(Self { script, output_dir })
    }
}

/// 单条脚本动作。
#[derive(Clone, Debug, PartialEq)]
enum Step {
    WaitMs {
        ms: u64,
    },
    WaitIdle {
        ms: u64,
        settle: u64,
    },
    Click {
        x: f64,
        y: f64,
        button: String,
    },
    /// 左/右键按下（不释放）：与 `up` 组合成拖拽（如页面文本拖选）。
    Down {
        x: f64,
        y: f64,
        button: String,
    },
    /// 按键释放：与 `down` 组合成拖拽。
    Up {
        x: f64,
        y: f64,
        button: String,
    },
    Move {
        x: f64,
        y: f64,
    },
    Key {
        key: String,
        mods: Vec<String>,
    },
    Type {
        text: String,
    },
    /// 滚轮滚动（真实 `handle_scroll` 路径）：`line` 模式（默认）传 LineDelta，
    /// `pixel` 模式传 PixelDelta；`dy` 正值向上（winit 符号约定）。
    Wheel {
        x: f64,
        y: f64,
        dx: f32,
        dy: f32,
        pixel: bool,
    },
    Shot {
        name: String,
    },
    Log {
        msg: String,
    },
    // `mode` 为 `"prefix"` 时按前缀匹配（可判别 `https://view-source:…` 类错误形态），
    // 缺省 `"contains"` 按子串匹配。不用 doc 注释：variant doc 会使 rustfmt 把整个
    // enum 强制展开为多行。
    AssertUrl {
        pattern: String,
        prefix: bool,
    },
    AssertMenu {
        open: bool,
    },
    /// 导航箭头启用态断言（app_smoke_state 只读面，与渲染同源）。
    AssertNav {
        back: bool,
        forward: bool,
    },
    /// 悬停链接状态栏断言：`contains` 按子串匹配显示文本；`absent` 断言已清除。
    AssertStatus {
        pattern: String,
        absent: bool,
    },
    /// 页面缩放断言（与 Ctrl± 同一状态源）。
    AssertZoom {
        zoom: f32,
    },
    /// 页面查找状态断言（T15）：给定字段逐一校验，缺省字段不校验。
    AssertFind {
        active: Option<bool>,
        query: Option<String>,
        current: Option<usize>,
        total: Option<usize>,
    },
    /// 标签溢出断言（T18）：`open=下拉展开`；`hidden_min` 为隐藏标签数下限。
    AssertTabOverflow {
        open: bool,
        hidden_min: usize,
    },
    /// 标签数断言（T20 会话恢复）：`count` 为期望的标签页数量。
    AssertTabs {
        count: usize,
    },
    /// 剪贴板文本断言：`contains` 子串匹配（读回系统剪贴板）。
    AssertClipboard {
        pattern: String,
    },
    /// 地址栏文本断言：`contains` 子串匹配。
    AssertAddressBar {
        pattern: String,
    },
    /// 垂直滚动偏移断言（T26 平滑滚动）：当前偏移须落在 `[min, max]`（物理像素）。
    AssertScrollY {
        min: f32,
        max: f32,
    },
    /// 崩溃恢复气泡断言（T20）：`visible` 为期望可见性。
    AssertSessionRestore {
        visible: bool,
    },
    /// 点击崩溃恢复气泡「恢复标签」按钮（T20；气泡未显示时报错）。
    ClickSessionRestoreButton,
    /// 打印预览断言（T17）：`on=true` 时渲染媒体类型应为 Print。
    AssertPrintPreview {
        on: bool,
    },
    /// 隐私开关断言（T16）：给定字段逐一校验，缺省不校验。
    AssertSettingsToggles {
        js: Option<bool>,
        cookies: Option<bool>,
        block_3p: Option<bool>,
        dnt: Option<bool>,
    },
    /// 点击标签溢出 ▾ 按钮（T18；几何取自渲染缓存，未渲染时报错）。
    ClickOverflowButton,
    /// 点击溢出下拉第 `index` 行（T18；要求下拉已展开）。
    ClickOverflowRow {
        index: usize,
    },
}

impl Step {
    fn parse(line: &str, line_no: usize) -> Result<Self, String> {
        let value: Value =
            serde_json::from_str(line).map_err(|error| format!("script line {line_no} is not valid JSON: {error}"))?;
        let bad = |what: &str| format!("script line {line_no}: {what}");
        let op = value
            .get("op")
            .and_then(Value::as_str)
            .ok_or_else(|| bad("missing string field \"op\""))?;
        let num = |field: &str| -> Result<f64, String> {
            value
                .get(field)
                .and_then(Value::as_f64)
                .ok_or_else(|| bad(format!("missing number field \"{field}\"").as_str()))
        };
        let text = |field: &str| -> Result<String, String> {
            value
                .get(field)
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| bad(format!("missing string field \"{field}\"").as_str()))
        };
        match op {
            "wait_ms" => Ok(Self::WaitMs { ms: num("ms")? as u64 }),
            "wait_idle" => Ok(Self::WaitIdle {
                ms: value.get("ms").and_then(Value::as_u64).unwrap_or(15_000),
                settle: value.get("settle").and_then(Value::as_u64).unwrap_or(400),
            }),
            "click" => Ok(Self::Click {
                x: num("x")?,
                y: num("y")?,
                button: value
                    .get("button")
                    .and_then(Value::as_str)
                    .unwrap_or("Left")
                    .to_string(),
            }),
            "down" | "up" => {
                let pressed = op == "down";
                Ok(if pressed {
                    Self::Down {
                        x: num("x")?,
                        y: num("y")?,
                        button: value
                            .get("button")
                            .and_then(Value::as_str)
                            .unwrap_or("Left")
                            .to_string(),
                    }
                } else {
                    Self::Up {
                        x: num("x")?,
                        y: num("y")?,
                        button: value
                            .get("button")
                            .and_then(Value::as_str)
                            .unwrap_or("Left")
                            .to_string(),
                    }
                })
            }
            "move" => Ok(Self::Move {
                x: num("x")?,
                y: num("y")?,
            }),
            "key" => Ok(Self::Key {
                key: text("key")?,
                mods: value
                    .get("mods")
                    .and_then(Value::as_array)
                    .map(|items| items.iter().filter_map(Value::as_str).map(str::to_string).collect())
                    .unwrap_or_default(),
            }),
            "type" => Ok(Self::Type { text: text("text")? }),
            "wheel" => Ok(Self::Wheel {
                x: num("x")?,
                y: num("y")?,
                dx: value.get("dx").and_then(Value::as_f64).unwrap_or(0.0) as f32,
                dy: value.get("dy").and_then(Value::as_f64).unwrap_or(0.0) as f32,
                pixel: value.get("mode").and_then(Value::as_str) == Some("pixel"),
            }),
            "shot" => Ok(Self::Shot { name: text("name")? }),
            "log" => Ok(Self::Log { msg: text("msg")? }),
            "assert_url" => {
                let mode = value.get("mode").and_then(Value::as_str).unwrap_or("contains");
                if mode != "contains" && mode != "prefix" {
                    return Err(format!(
                        "script line {line_no}: assert_url mode \"{mode}\" must be \"contains\" or \"prefix\""
                    ));
                }
                Ok(Self::AssertUrl {
                    pattern: text("contains")?,
                    prefix: mode == "prefix",
                })
            }
            "assert_menu" => Ok(Self::AssertMenu {
                open: value
                    .get("open")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| bad("assert_menu requires boolean field \"open\""))?,
            }),
            "assert_nav" => Ok(Self::AssertNav {
                back: value
                    .get("back")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| bad("assert_nav requires boolean field \"back\""))?,
                forward: value
                    .get("forward")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| bad("assert_nav requires boolean field \"forward\""))?,
            }),
            "assert_status" => {
                let absent = value.get("absent").and_then(Value::as_bool).unwrap_or(false);
                if !absent {
                    // absent 模式不要求 contains；非 absent 必须给出匹配串。
                    text("contains")?;
                }
                Ok(Self::AssertStatus {
                    pattern: value
                        .get("contains")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    absent,
                })
            }
            "assert_zoom" => Ok(Self::AssertZoom {
                zoom: num("zoom")? as f32,
            }),
            "assert_find" => Ok(Self::AssertFind {
                active: value.get("active").and_then(Value::as_bool),
                query: value.get("query").and_then(Value::as_str).map(str::to_string),
                current: value.get("current").and_then(Value::as_u64).map(|v| v as usize),
                total: value.get("total").and_then(Value::as_u64).map(|v| v as usize),
            }),
            "assert_tab_overflow" => Ok(Self::AssertTabOverflow {
                open: value.get("open").and_then(Value::as_bool).unwrap_or(false),
                hidden_min: value.get("hidden_min").and_then(Value::as_u64).unwrap_or(0) as usize,
            }),
            "assert_print_preview" => Ok(Self::AssertPrintPreview {
                on: value.get("on").and_then(Value::as_bool).unwrap_or(true),
            }),
            "assert_settings_toggles" => Ok(Self::AssertSettingsToggles {
                js: value.get("js").and_then(Value::as_bool),
                cookies: value.get("cookies").and_then(Value::as_bool),
                block_3p: value.get("block_3p").and_then(Value::as_bool),
                dnt: value.get("dnt").and_then(Value::as_bool),
            }),
            "click_tab_overflow_button" => Ok(Self::ClickOverflowButton),
            "click_tab_overflow_row" => Ok(Self::ClickOverflowRow {
                index: num("index")? as usize,
            }),
            "assert_tabs" => Ok(Self::AssertTabs {
                count: num("count")? as usize,
            }),
            "assert_clipboard" => Ok(Self::AssertClipboard {
                pattern: text("contains")?,
            }),
            "assert_addressbar" => Ok(Self::AssertAddressBar {
                pattern: text("contains")?,
            }),
            "assert_scroll_y" => {
                let min = value.get("min").and_then(Value::as_f64).unwrap_or(0.0) as f32;
                let max = value
                    .get("max")
                    .and_then(Value::as_f64)
                    .map(|v| v as f32)
                    .ok_or_else(|| bad("assert_scroll_y requires number field \"max\""))?;
                Ok(Self::AssertScrollY { min, max })
            }
            "assert_session_restore" => Ok(Self::AssertSessionRestore {
                visible: value
                    .get("visible")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| bad("assert_session_restore requires boolean field \"visible\""))?,
            }),
            "click_session_restore_button" => Ok(Self::ClickSessionRestoreButton),
            other => Err(bad(format!("unknown op \"{other}\"").as_str())),
        }
    }
}

/// 等待中的步骤：墙钟到期或页面静默后放行。
enum Pending {
    Until(Instant),
    Idle {
        deadline: Instant,
        settle: Duration,
        quiet_since: Option<Instant>,
    },
}

/// UI 驱动器执行状态。
pub struct UiDriver {
    steps: Vec<Step>,
    output_dir: PathBuf,
    next: usize,
    pending: Option<Pending>,
    deadline: Instant,
    finished: bool,
}

impl UiDriver {
    /// 解析脚本并创建驱动器（窗口打开前失败快速退出）。
    pub fn new(config: UiDriverConfig) -> Result<Self, String> {
        let raw = std::fs::read_to_string(&config.script)
            .map_err(|error| format!("cannot read ui driver script {}: {error}", config.script.display()))?;
        let mut steps = Vec::new();
        for (index, line) in raw.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            steps.push(Step::parse(line, index + 1)?);
        }
        Ok(Self {
            steps,
            output_dir: config.output_dir,
            next: 0,
            pending: None,
            deadline: Instant::now() + DRIVER_TIMEOUT,
            finished: false,
        })
    }

    /// 在浏览器窗口与默认标签页就绪后开始执行。
    pub fn start(&mut self, app: &mut BrowserApp) {
        tracing::info!("UI_DRIVER_START steps={}", self.steps.len());
        app.needs_redraw = true;
    }

    /// 检查整体墙钟上限；超时先落盘 result.json 再报错——无人值守 harness 在
    /// 超时场景同样能读到产物（与步骤失败/正常完成路径同一契约）。
    pub fn check_timeout(&self) -> Result<(), String> {
        if !self.finished && Instant::now() > self.deadline {
            let detail = format!("timed out at step {}/{}", self.next, self.steps.len());
            self.write_result(false, &detail);
            return Err(detail);
        }
        Ok(())
    }

    /// 一次 poll tick：放行到期的等待或执行下一步。
    ///
    /// 返回 `true` 表示脚本已完成（成功），调用方应正常退出。
    pub fn tick(&mut self, app: &mut BrowserApp) -> Result<bool, String> {
        if self.finished {
            return Ok(true);
        }
        if self.next >= self.steps.len() {
            self.finished = true;
            self.write_result(true, &format!("completed {} steps", self.steps.len()));
            tracing::info!("UI_DRIVER_COMPLETE steps={}", self.steps.len());
            return Ok(true);
        }
        if let Some(pending) = &mut self.pending {
            let due = match pending {
                Pending::Until(at) => Instant::now() >= *at,
                Pending::Idle {
                    deadline,
                    settle,
                    quiet_since,
                } => {
                    if app.any_tab_loading() {
                        *quiet_since = None;
                    } else if quiet_since.is_none() {
                        *quiet_since = Some(Instant::now());
                    }
                    let quiet = quiet_since.is_some_and(|since| since.elapsed() >= *settle);
                    quiet || Instant::now() >= *deadline
                }
            };
            if !due {
                app.needs_redraw = true;
                return Ok(false);
            }
            self.pending = None;
        }
        let step = self.steps[self.next].clone();
        match self.execute(app, &step) {
            Ok(()) => {
                self.next += 1;
                app.needs_redraw = true;
                Ok(false)
            }
            Err(error) => {
                self.finished = true;
                self.write_result(false, &format!("step {} failed: {error}", self.next + 1));
                Err(error)
            }
        }
    }

    fn execute(&mut self, app: &mut BrowserApp, step: &Step) -> Result<(), String> {
        match step {
            Step::WaitMs { ms } => {
                self.pending = Some(Pending::Until(Instant::now() + Duration::from_millis(*ms)));
                Ok(())
            }
            Step::WaitIdle { ms, settle } => {
                self.pending = Some(Pending::Idle {
                    deadline: Instant::now() + Duration::from_millis(*ms),
                    settle: Duration::from_millis(*settle),
                    quiet_since: None,
                });
                Ok(())
            }
            Step::Click { x, y, button } => {
                app.handle_mouse_click(*x, *y, true, button);
                app.handle_mouse_click(*x, *y, false, button);
                tracing::info!("UI_DRIVER_STEP step={} x={x} y={y} button={button}", self.next + 1);
                Ok(())
            }
            Step::Down { x, y, button } => {
                // 与 main 窗口事件同一顺序：先 move 再 press（保证 hover 状态一致）。
                app.handle_mouse_move(*x, *y);
                app.handle_mouse_click(*x, *y, true, button);
                tracing::info!("UI_DRIVER_STEP step={} down x={x} y={y} button={button}", self.next + 1);
                Ok(())
            }
            Step::Up { x, y, button } => {
                app.handle_mouse_move(*x, *y);
                app.handle_mouse_click(*x, *y, false, button);
                tracing::info!("UI_DRIVER_STEP step={} up x={x} y={y} button={button}", self.next + 1);
                Ok(())
            }
            Step::Move { x, y } => {
                app.handle_mouse_move(*x, *y);
                Ok(())
            }
            Step::Key { key, mods } => {
                let names: Vec<&str> = mods.iter().map(String::as_str).collect();
                press_chord(app, &names, key);
                tracing::info!("UI_DRIVER_STEP step={} key={key} mods={mods:?}", self.next + 1);
                Ok(())
            }
            Step::Type { text } => {
                for ch in text.chars() {
                    app.handle_key(&ch.to_string(), true, None);
                    app.handle_key(&ch.to_string(), false, None);
                }
                Ok(())
            }
            Step::Wheel { x, y, dx, dy, pixel } => {
                let delta = if *pixel {
                    zero_host_runtime::event::MouseScrollDelta::PixelDelta(*dx as f64, *dy as f64)
                } else {
                    zero_host_runtime::event::MouseScrollDelta::LineDelta(*dx, *dy)
                };
                app.handle_scroll(delta, *x, *y);
                tracing::info!(
                    "UI_DRIVER_STEP step={} wheel x={x} y={y} dx={dx} dy={dy} pixel={pixel}",
                    self.next + 1
                );
                Ok(())
            }
            Step::Shot { name } => {
                let (width, height) = app.physical_size;
                tracing::info!("UI_DRIVER_SHOT {name} find_state={:?}", app.smoke_find_state());
                let frame = app
                    .render_full_scene_gpu_capture(width, height)
                    .map_err(|error| format!("shot {name}: {error}"))?;
                self.save_png(&frame, name)?;
                tracing::info!("UI_DRIVER_STEP step={} shot={name}", self.next + 1);
                Ok(())
            }
            Step::Log { msg } => {
                tracing::info!("UI_DRIVER_STEP step={} log={msg}", self.next + 1);
                Ok(())
            }
            Step::AssertUrl { pattern, prefix } => {
                let id = app
                    .smoke_active_tab_id()
                    .ok_or_else(|| "assert_url: no active tab".to_string())?;
                let url = app
                    .smoke_tab_url(id)
                    .ok_or_else(|| "assert_url: active tab has no url".to_string())?;
                let matched = if *prefix {
                    url.starts_with(pattern.as_str())
                } else {
                    url.contains(pattern.as_str())
                };
                if !matched {
                    return Err(format!(
                        "assert_url failed: url is {url}, expected to {} {pattern}",
                        if *prefix { "start with" } else { "contain" }
                    ));
                }
                tracing::info!("UI_DRIVER_STEP step={} assert_url={url} status=passed", self.next + 1);
                Ok(())
            }
            Step::AssertMenu { open } => {
                let visible = app.smoke_context_menu_visible();
                if visible != *open {
                    return Err(format!("assert_menu failed: menu visible={visible}, expected {open}"));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_menu={visible} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertNav { back, forward } => {
                let (actual_back, actual_forward) = (app.smoke_can_go_back(), app.smoke_can_go_forward());
                if actual_back != *back || actual_forward != *forward {
                    return Err(format!(
                        "assert_nav failed: back={actual_back} forward={actual_forward}, expected back={back} forward={forward}"
                    ));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_nav back={actual_back} forward={actual_forward} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertStatus { pattern, absent } => {
                if *absent {
                    if let Some(url) = app.smoke_hovered_link_url() {
                        return Err(format!(
                            "assert_status failed: status still shows {url}, expected cleared"
                        ));
                    }
                    tracing::info!(
                        "UI_DRIVER_STEP step={} assert_status=cleared status=passed",
                        self.next + 1
                    );
                    return Ok(());
                }
                let Some(url) = app.smoke_hovered_link_url() else {
                    return Err(format!(
                        "assert_status failed: no hovered link, expected to contain {pattern}"
                    ));
                };
                if !url.contains(pattern.as_str()) {
                    return Err(format!(
                        "assert_status failed: status is {url}, expected to contain {pattern}"
                    ));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_status={url} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertZoom { zoom } => {
                let actual = app.smoke_page_zoom();
                if (actual - zoom).abs() > f32::EPSILON {
                    return Err(format!("assert_zoom failed: zoom is {actual}, expected {zoom}"));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_zoom={actual} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertFind {
                active,
                query,
                current,
                total,
            } => {
                let (a, q, c, t) = app.smoke_find_state();
                if let Some(want) = active
                    && a != *want
                {
                    return Err(format!("assert_find failed: active is {a}, expected {want}"));
                }
                if let Some(want) = query
                    && q != *want
                {
                    return Err(format!("assert_find failed: query is {q:?}, expected {want:?}"));
                }
                if let Some(want) = current
                    && c != *want
                {
                    return Err(format!("assert_find failed: current is {c}, expected {want}"));
                }
                if let Some(want) = total
                    && t != *want
                {
                    return Err(format!("assert_find failed: total is {t}, expected {want}"));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_find active={a} query={q:?} current={c} total={t} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertTabOverflow { open, hidden_min } => {
                let (o, hidden) = app.smoke_tab_overflow();
                if o != *open || hidden < *hidden_min {
                    return Err(format!(
                        "assert_tab_overflow failed: open={o} hidden={hidden}, expected open={} hidden>={hidden_min}",
                        open
                    ));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_tab_overflow open={o} hidden={hidden} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertTabs { count } => {
                let actual = app.smoke_tab_ids().len();
                if actual != *count {
                    return Err(format!("assert_tabs failed: tab count is {actual}, expected {count}"));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_tabs count={actual} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertClipboard { pattern } => {
                let actual = app.smoke_clipboard_text().unwrap_or_default();
                if !actual.contains(pattern.as_str()) {
                    return Err(format!(
                        "assert_clipboard failed: clipboard is {actual:?}, expected to contain {pattern}"
                    ));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_clipboard len={} status=passed",
                    self.next + 1,
                    actual.len()
                );
                Ok(())
            }
            Step::AssertAddressBar { pattern } => {
                let actual = app.smoke_address_bar_text().to_string();
                if !actual.contains(pattern.as_str()) {
                    return Err(format!(
                        "assert_addressbar failed: address bar is {actual:?}, expected to contain {pattern}"
                    ));
                }
                tracing::info!("UI_DRIVER_STEP step={} assert_addressbar status=passed", self.next + 1);
                Ok(())
            }
            Step::AssertScrollY { min, max } => {
                let actual = app.smoke_scroll_offset_y();
                if actual < *min || actual > *max {
                    return Err(format!(
                        "assert_scroll_y failed: scroll y is {actual}, expected within [{min}, {max}]"
                    ));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_scroll_y={actual} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertSessionRestore { visible } => {
                let actual = app.smoke_session_restore_prompt();
                if actual != *visible {
                    return Err(format!(
                        "assert_session_restore failed: prompt visible={actual}, expected {visible}"
                    ));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_session_restore visible={actual} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::ClickSessionRestoreButton => {
                let (bx, by, bw, bh) = app.session_restore_bubble_rect_for(app.physical_size.0, app.physical_size.1);
                if !app.smoke_session_restore_prompt() {
                    return Err("click_session_restore_button: restore prompt is not visible".to_string());
                }
                let s = app.scale_factor;
                let (restore, _) = BrowserApp::session_restore_button_rects((bx, by, bw, bh), s);
                let cx = restore.0 + restore.2 / 2.0;
                let cy = restore.1 + restore.3 / 2.0;
                app.handle_mouse_move(cx as f64, cy as f64);
                app.handle_mouse_click(cx as f64, cy as f64, true, "Left");
                app.handle_mouse_click(cx as f64, cy as f64, false, "Left");
                tracing::info!(
                    "UI_DRIVER_STEP step={} click_session_restore x={cx} y={cy}",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertPrintPreview { on } => {
                let media = app.smoke_media_type();
                let want = if *on { "Print" } else { "Screen" };
                if media != want {
                    return Err(format!(
                        "assert_print_preview failed: media type is {media}, expected {want}"
                    ));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_print_preview media={media} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertSettingsToggles {
                js,
                cookies,
                block_3p,
                dnt,
            } => {
                let (a_js, a_cookies, a_3p, a_dnt) = app.smoke_settings_toggles();
                for (name, actual, want) in [
                    ("js", a_js, *js),
                    ("cookies", a_cookies, *cookies),
                    ("block_3p", a_3p, *block_3p),
                    ("dnt", a_dnt, *dnt),
                ] {
                    if let Some(want) = want
                        && actual != want
                    {
                        return Err(format!(
                            "assert_settings_toggles failed: {name} is {actual}, expected {want}"
                        ));
                    }
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_settings_toggles js={a_js} cookies={a_cookies} block_3p={a_3p} dnt={a_dnt} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::ClickOverflowButton => {
                let Some((bx, by, bw, bh)) = app.smoke_tab_overflow_button_rect() else {
                    return Err("click_tab_overflow_button: no overflow button rendered".to_string());
                };
                let (cx, cy) = ((bx + bw / 2.0) as f64, (by + bh / 2.0) as f64);
                app.handle_mouse_click(cx, cy, true, "Left");
                app.handle_mouse_click(cx, cy, false, "Left");
                tracing::info!(
                    "UI_DRIVER_STEP step={} click_tab_overflow_button cx={cx} cy={cy}",
                    self.next + 1
                );
                Ok(())
            }
            Step::ClickOverflowRow { index } => {
                if !app.smoke_tab_overflow().0 {
                    return Err("click_tab_overflow_row: overflow dropdown is not open".to_string());
                }
                let Some((px, py, _, _)) = app.smoke_tab_overflow_panel_rect() else {
                    return Err("click_tab_overflow_row: no panel rect".to_string());
                };
                let row_h = crate::layout::TAB_OVERFLOW_ROW_HEIGHT * app.scale_factor;
                let (cx, cy) = ((px + 10.0) as f64, (py + row_h * (*index as f32 + 0.5)) as f64);
                app.handle_mouse_click(cx, cy, true, "Left");
                app.handle_mouse_click(cx, cy, false, "Left");
                tracing::info!(
                    "UI_DRIVER_STEP step={} click_tab_overflow_row index={index} cx={cx} cy={cy}",
                    self.next + 1
                );
                Ok(())
            }
        }
    }

    fn save_png(&self, frame: &zero_render_foundation::surface::FrameBuffer, name: &str) -> Result<(), String> {
        if name.contains("..") || name.contains('/') || name.contains('\\') {
            return Err(format!("invalid shot name {name}"));
        }
        std::fs::create_dir_all(&self.output_dir)
            .map_err(|error| format!("failed to create output directory: {error}"))?;
        let path = self.output_dir.join(name);
        let file =
            std::fs::File::create(&path).map_err(|error| format!("failed to create {}: {error}", path.display()))?;
        let mut encoder = png::Encoder::new(file, frame.width, frame.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|error| format!("failed to write PNG header: {error}"))?;
        writer
            .write_image_data(&frame.data)
            .map_err(|error| format!("failed to write PNG pixels: {error}"))?;
        Ok(())
    }

    fn write_result(&self, ok: bool, detail: &str) {
        let path = self.output_dir.join("result.json");
        let payload = serde_json::json!({
            "schema_version": 1,
            "ok": ok,
            "steps_total": self.steps.len(),
            "steps_done": self.next,
            "detail": detail,
        });
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, serde_json::to_vec_pretty(&payload).unwrap_or_default());
    }
}

/// 修饰键名映射到 handle_key 的键名。
fn modifier_key(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "ctrl" | "control" => Some("Control"),
        "shift" => Some("Shift"),
        "alt" => Some("Alt"),
        "super" | "meta" => Some("Super"),
        _ => None,
    }
}

/// 按下修饰键 → 按下并释放主键 → 释放修饰键（与既有 smoke 的和弦模式一致）。
fn press_chord(app: &mut BrowserApp, mods: &[&str], key: &str) {
    let held: Vec<&str> = mods.iter().filter_map(|name| modifier_key(name)).collect();
    for name in &held {
        app.handle_key(name, true, None);
    }
    app.handle_key(key, true, None);
    app.handle_key(key, false, None);
    for name in held.iter().rev() {
        app.handle_key(name, false, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_requires_script_and_output_paths() {
        assert!(UiDriverConfig::new(PathBuf::new(), PathBuf::from("out")).is_err());
        assert!(UiDriverConfig::new(PathBuf::from("script.jsonl"), PathBuf::new()).is_err());
        assert!(UiDriverConfig::new(PathBuf::from("script.jsonl"), PathBuf::from("out")).is_ok());
    }

    #[test]
    fn parse_covers_all_ops_and_rejects_unknown() {
        assert_eq!(
            Step::parse(r#"{"op":"wait_ms","ms":500}"#, 1),
            Ok(Step::WaitMs { ms: 500 })
        );
        assert_eq!(
            Step::parse(r#"{"op":"click","x":10,"y":20}"#, 1),
            Ok(Step::Click {
                x: 10.0,
                y: 20.0,
                button: "Left".into()
            })
        );
        assert_eq!(
            Step::parse(r#"{"op":"key","key":"l","mods":["ctrl"]}"#, 1),
            Ok(Step::Key {
                key: "l".into(),
                mods: vec!["ctrl".into()]
            })
        );
        assert!(Step::parse(r#"{"op":"nonsense"}"#, 1).is_err());
        assert!(Step::parse(r#"{"op":"click","x":10}"#, 1).is_err());
    }

    /// assert_url 缺省 contains；`mode:"prefix"` 按前缀匹配；未知 mode 拒绝。
    #[test]
    fn parse_assert_url_supports_prefix_mode() {
        assert_eq!(
            Step::parse(r#"{"op":"assert_url","contains":"page2.html"}"#, 1),
            Ok(Step::AssertUrl {
                pattern: "page2.html".into(),
                prefix: false
            })
        );
        assert_eq!(
            Step::parse(r#"{"op":"assert_url","contains":"view-source:","mode":"prefix"}"#, 1),
            Ok(Step::AssertUrl {
                pattern: "view-source:".into(),
                prefix: true
            })
        );
        assert!(Step::parse(r#"{"op":"assert_url","contains":"x","mode":"suffix"}"#, 1).is_err());
    }
}
