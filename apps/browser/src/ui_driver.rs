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

/// 滚动帧率基准收尾静置：等最后一次滚动的平滑动画帧落盘再统计。
const SCROLL_BENCH_SETTLE: Duration = Duration::from_millis(300);

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
    /// 写入剪贴板文本（T46b 书签导入等以剪贴板为输入源的用例探针）。
    SetClipboard {
        text: String,
    },
    /// 全库书签断言（T46b 导入）：`count` 精确总数（含文件夹内条目）；
    /// `contains` 任一条目标题或 URL 子串命中即可。至少给一个。
    AssertBookmarks {
        count: Option<usize>,
        contains: Option<String>,
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
    /// 点击地址栏前导 shield/lock 图标（T27-T30 站点信息面板；几何取自渲染同源 helper）。
    ClickSiteInfoIcon,
    /// 点击缩放百分比浮层（T27-T30 点击复位；浮层未显示时报错）。
    ClickZoomIndicator,
    /// 缩放百分比浮层显示断言：`visible` 为期望显示状态（3 秒窗口内）。
    AssertZoomIndicator {
        visible: bool,
    },
    /// 历史条目数断言（T27-T30 清除浏览数据）。`count` 精确匹配，`min`/`max` 边界，
    /// 至少给一个。
    AssertHistory {
        count: Option<usize>,
        min: Option<usize>,
        max: Option<usize>,
    },
    /// 页面焦点选择器断言：`contains` 子串匹配（Tab 焦点导航）。
    AssertFocus {
        pattern: String,
    },
    /// 点击查找栏选项按钮：`"case"` = Aa（区分大小写），`"whole"` = W•（全字匹配）。
    /// 几何与查找栏点击处理同源（bar 右缘偏移 160/130）。
    ClickFindOption {
        option: String,
    },
    /// 点击地址栏尾随 shield 图标（站点权限菜单；既有功能回归）。
    ClickSitePermissionsIcon,
    /// 上下文菜单项 id 断言：`contains` 须匹配任一菜单项 id。
    AssertMenuItems {
        pattern: String,
    },
    /// 共享 Cookie jar 条目数断言（清除浏览数据）。
    AssertCookies {
        count: usize,
    },
    /// 下载完成 toast 断言（T34）：`visible` 为期望可见性；`contains` 可选校验文件名。
    AssertDownloadToast {
        visible: bool,
        contains: Option<String>,
    },
    /// 关闭标签撤销 toast 断言（t47）：`visible` 为期望可见性；`contains` 可选校验文案。
    AssertClosedTabToast {
        visible: bool,
        contains: Option<String>,
    },
    /// 点击关闭标签撤销 toast 的「撤销」动作（t47；toast 未显示时报错）。
    ClickClosedTabToast,
    /// 压缩标签悬停 tooltip 断言（T35）：`visible` 为期望可见性；`contains` 可选
    /// 校验标题/URL。
    AssertTabTooltip {
        visible: bool,
        contains: Option<String>,
    },
    /// 可视标签条顺序断言（T54 溢出拖拽落点）：`contains[i]` 与第 i 个可视槽位
    /// 标签的 URL 子串一一对应，长度精确相等（tab_layout 与渲染同源，帧后有效）。
    AssertTabOrder {
        patterns: Vec<String>,
    },
    /// 悬停第 `index` 个标签槽位中心（几何取自渲染缓存 tab_layout；只移动不点击）。
    HoverTab {
        index: usize,
    },
    /// 点击第 `index` 个标签槽位的 close 按钮（t47；几何取自渲染缓存 tab_layout，
    /// 与 app_input 关闭按钮命中同源：`tab_x + tab_w - 24s` 起 TAB_CLOSE_SIZE 方块）。
    ClickTabClose {
        index: usize,
    },
    /// 地址栏编辑聚焦态断言（T50 关闭标签失焦）：`focused` 为期望状态。
    AssertAddressBarFocus {
        focused: bool,
    },
    /// 点击上下文菜单中 id 匹配的项（行中心，几何与 context_menu_hit_test 同源：
    /// 分隔行按紧凑高度累加）。菜单未开或无此项时报错。
    ClickMenuItem {
        id: String,
    },
    /// 点击标签栏「+」新标签按钮（T52 右键菜单验证；`button` 为 "Right"/"Left"，
    /// 几何与 app_input 命中同源：new_tab_button_x 起 NEW_TAB_BTN_WIDTH 方块）。
    ClickNewTabButton {
        button: String,
    },
    /// 滚轮滚动帧率基准（T36）：`steps` 次滚轮按 `interval_ms` 间隔派发，统计区间
    /// 呈现帧数与墙钟得平均 fps；`min_fps` > 0 时低于下限报错。结果写入 bench-scroll.json。
    BenchScroll {
        x: f64,
        y: f64,
        dy: f32,
        steps: usize,
        interval_ms: u64,
        min_fps: f32,
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
            "click_site_info_icon" => Ok(Self::ClickSiteInfoIcon),
            "click_zoom_indicator" => Ok(Self::ClickZoomIndicator),
            "assert_zoom_indicator" => Ok(Self::AssertZoomIndicator {
                visible: value
                    .get("visible")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| bad("assert_zoom_indicator requires boolean field \"visible\""))?,
            }),
            "assert_history" => Ok(Self::AssertHistory {
                count: value.get("count").and_then(Value::as_u64).map(|v| v as usize),
                min: value.get("min").and_then(Value::as_u64).map(|v| v as usize),
                max: value.get("max").and_then(Value::as_u64).map(|v| v as usize),
            }),
            "assert_focus" => Ok(Self::AssertFocus {
                pattern: text("contains")?,
            }),
            "click_find_option" => Ok(Self::ClickFindOption {
                option: text("option")?,
            }),
            "click_site_permissions_icon" => Ok(Self::ClickSitePermissionsIcon),
            "assert_menu_items" => Ok(Self::AssertMenuItems {
                pattern: text("contains")?,
            }),
            "assert_cookies" => Ok(Self::AssertCookies {
                count: num("count")? as usize,
            }),
            "assert_download_toast" => Ok(Self::AssertDownloadToast {
                visible: value
                    .get("visible")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| bad("assert_download_toast requires boolean field \"visible\""))?,
                contains: value.get("contains").and_then(Value::as_str).map(str::to_string),
            }),
            "assert_closed_tab_toast" => Ok(Self::AssertClosedTabToast {
                visible: value
                    .get("visible")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| bad("assert_closed_tab_toast requires boolean field \"visible\""))?,
                contains: value.get("contains").and_then(Value::as_str).map(str::to_string),
            }),
            "click_closed_tab_toast" => Ok(Self::ClickClosedTabToast),
            "assert_tab_order" => {
                // TV-4：非字符串项报错拒绝——静默降级空串会让 contains 恒假
                //（断言假绿），脚本笔误必须在解析期暴露。
                let items = value
                    .get("contains")
                    .and_then(Value::as_array)
                    .ok_or_else(|| bad("assert_tab_order requires string array field \"contains\""))?;
                let mut patterns = Vec::with_capacity(items.len());
                for (index, item) in items.iter().enumerate() {
                    let text = item.as_str().ok_or_else(|| {
                        bad(format!("assert_tab_order contains[{index}] must be a string, got {item}").as_str())
                    })?;
                    patterns.push(text.to_string());
                }
                Ok(Self::AssertTabOrder { patterns })
            }
            "assert_tab_tooltip" => Ok(Self::AssertTabTooltip {
                visible: value
                    .get("visible")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| bad("assert_tab_tooltip requires boolean field \"visible\""))?,
                contains: value.get("contains").and_then(Value::as_str).map(str::to_string),
            }),
            "hover_tab" => Ok(Self::HoverTab {
                index: num("index")? as usize,
            }),
            "click_tab_close" => Ok(Self::ClickTabClose {
                index: num("index")? as usize,
            }),
            "assert_addressbar_focus" => Ok(Self::AssertAddressBarFocus {
                focused: value
                    .get("focused")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| bad("assert_addressbar_focus requires boolean field \"focused\""))?,
            }),
            "click_menu_item" => Ok(Self::ClickMenuItem { id: text("id")? }),
            "click_new_tab_button" => Ok(Self::ClickNewTabButton {
                button: value
                    .get("button")
                    .and_then(Value::as_str)
                    .unwrap_or("Left")
                    .to_string(),
            }),
            "bench_scroll" => Ok(Self::BenchScroll {
                x: num("x")?,
                y: num("y")?,
                dy: value.get("dy").and_then(Value::as_f64).unwrap_or(-3.0) as f32,
                steps: value.get("steps").and_then(Value::as_u64).unwrap_or(60) as usize,
                interval_ms: value.get("interval_ms").and_then(Value::as_u64).unwrap_or(16),
                min_fps: value.get("min_fps").and_then(Value::as_f64).unwrap_or(0.0) as f32,
            }),
            "assert_tabs" => Ok(Self::AssertTabs {
                count: num("count")? as usize,
            }),
            "assert_clipboard" => Ok(Self::AssertClipboard {
                pattern: text("contains")?,
            }),
            "set_clipboard" => Ok(Self::SetClipboard { text: text("text")? }),
            "assert_bookmarks" => {
                let count = value.get("count").and_then(Value::as_u64).map(|v| v as usize);
                let contains = value.get("contains").and_then(Value::as_str).map(str::to_string);
                if count.is_none() && contains.is_none() {
                    return Err(format!(
                        "script line {line_no}: assert_bookmarks requires \"count\" or \"contains\""
                    ));
                }
                Ok(Self::AssertBookmarks { count, contains })
            }
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
    /// 滚轮帧率基准进行态（T26 平滑滚动同帧循环驱动）。
    BenchScroll {
        x: f64,
        y: f64,
        dy: f32,
        remaining: usize,
        interval: Duration,
        next_fire: Instant,
        first_fire: Option<Instant>,
        last_fire: Option<Instant>,
        start_frames: u64,
        min_fps: f32,
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
        if matches!(self.pending, Some(Pending::BenchScroll { .. })) {
            // 基准完成时清 pending 并落回下方主流程继续下一步；未完成直接等下一 tick。
            if self.advance_scroll_bench(app)? {
                self.pending = None;
            } else {
                app.needs_redraw = true;
                return Ok(false);
            }
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
                // BenchScroll 在上方分支独立推进，不会落入本 match。
                Pending::BenchScroll { .. } => true,
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
            Step::SetClipboard { text } => {
                if !crate::clipboard::write_text(text) {
                    return Err("set_clipboard: clipboard unavailable".to_string());
                }
                tracing::info!("UI_DRIVER_STEP step={} set_clipboard len={}", self.next + 1, text.len());
                Ok(())
            }
            Step::AssertBookmarks { count, contains } => {
                let items = app.smoke_bookmarks_all();
                if let Some(want) = count
                    && items.len() != *want
                {
                    return Err(format!(
                        "assert_bookmarks failed: store has {} bookmarks, expected {want}",
                        items.len()
                    ));
                }
                if let Some(pattern) = contains
                    && !items
                        .iter()
                        .any(|(title, url)| title.contains(pattern.as_str()) || url.contains(pattern.as_str()))
                {
                    return Err(format!(
                        "assert_bookmarks failed: no bookmark title/url contains {pattern:?} ({} total)",
                        items.len()
                    ));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_bookmarks total={} status=passed",
                    self.next + 1,
                    items.len()
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
            Step::ClickSiteInfoIcon => {
                let (slot_x, slot_y, slot_w, slot_h) = app.smoke_address_bar_leading_slot_rect();
                let (cx, cy) = ((slot_x + slot_w / 2.0) as f64, (slot_y + slot_h / 2.0) as f64);
                app.handle_mouse_click(cx, cy, true, "Left");
                app.handle_mouse_click(cx, cy, false, "Left");
                tracing::info!(
                    "UI_DRIVER_STEP step={} click_site_info_icon cx={cx} cy={cy}",
                    self.next + 1
                );
                Ok(())
            }
            Step::ClickZoomIndicator => {
                let Some((bx, by, bw, bh)) = app.zoom_indicator_rect_for(app.physical_size.0, app.physical_size.1)
                else {
                    return Err("click_zoom_indicator: zoom indicator is not visible".to_string());
                };
                let (cx, cy) = ((bx + bw / 2.0) as f64, (by + bh / 2.0) as f64);
                app.handle_mouse_click(cx, cy, true, "Left");
                app.handle_mouse_click(cx, cy, false, "Left");
                tracing::info!(
                    "UI_DRIVER_STEP step={} click_zoom_indicator cx={cx} cy={cy}",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertZoomIndicator { visible } => {
                let actual = app.smoke_zoom_indicator_visible();
                if actual != *visible {
                    return Err(format!(
                        "assert_zoom_indicator failed: visible={actual}, expected {visible}"
                    ));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_zoom_indicator visible={actual} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertHistory { count, min, max } => {
                if count.is_none() && min.is_none() && max.is_none() {
                    return Err("assert_history requires one of count/min/max".to_string());
                }
                let actual = app.smoke_history_len();
                if let Some(want) = count
                    && actual != *want
                {
                    return Err(format!("assert_history failed: len is {actual}, expected {want}"));
                }
                if let Some(want) = min
                    && actual < *want
                {
                    return Err(format!("assert_history failed: len is {actual}, expected >= {want}"));
                }
                if let Some(want) = max
                    && actual > *want
                {
                    return Err(format!("assert_history failed: len is {actual}, expected <= {want}"));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_history len={actual} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertFocus { pattern } => {
                let Some(actual) = app.smoke_focus_target() else {
                    return Err(format!(
                        "assert_focus failed: no focus target, expected to contain {pattern}"
                    ));
                };
                if !actual.contains(pattern.as_str()) {
                    return Err(format!(
                        "assert_focus failed: focus target is {actual:?}, expected to contain {pattern}"
                    ));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_focus target={actual:?} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::ClickFindOption { option } => {
                let s = app.scale_factor;
                let offset = match option.as_str() {
                    "case" => 160.0,
                    "whole" => 130.0,
                    other => return Err(format!("click_find_option: unknown option {other:?}")),
                };
                let (bar_x, bar_y, bar_w, bar_h) = app.find_bar_rect_for(app.physical_size.0, app.physical_size.1);
                let (cx, cy) = (
                    (bar_x + bar_w - offset * s + 14.0 * s) as f64,
                    (bar_y + bar_h / 2.0) as f64,
                );
                app.handle_mouse_click(cx, cy, true, "Left");
                app.handle_mouse_click(cx, cy, false, "Left");
                tracing::info!(
                    "UI_DRIVER_STEP step={} click_find_option={option} cx={cx} cy={cy}",
                    self.next + 1
                );
                Ok(())
            }
            Step::ClickSitePermissionsIcon => {
                let (slot_x, slot_y, slot_w, slot_h) = app.smoke_address_bar_trailing_slot_rect(1);
                let (cx, cy) = ((slot_x + slot_w / 2.0) as f64, (slot_y + slot_h / 2.0) as f64);
                app.handle_mouse_click(cx, cy, true, "Left");
                app.handle_mouse_click(cx, cy, false, "Left");
                tracing::info!(
                    "UI_DRIVER_STEP step={} click_site_permissions_icon cx={cx} cy={cy}",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertMenuItems { pattern } => {
                let ids = app.smoke_context_menu_item_ids();
                if !ids.iter().any(|id| id.contains(pattern.as_str())) {
                    return Err(format!(
                        "assert_menu_items failed: items are {ids:?}, expected one to contain {pattern}"
                    ));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_menu_items contains={pattern} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertCookies { count } => {
                let actual = app.smoke_cookie_count();
                if actual != *count {
                    return Err(format!(
                        "assert_cookies failed: cookie jar len is {actual}, expected {count}"
                    ));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_cookies len={actual} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertDownloadToast { visible, contains } => {
                let actual = app.smoke_download_toast();
                match (visible, actual.as_ref()) {
                    (true, None) => {
                        return Err("assert_download_toast failed: toast not visible".to_string());
                    }
                    (false, Some(name)) => {
                        return Err(format!(
                            "assert_download_toast failed: toast still visible with {name:?}"
                        ));
                    }
                    (true, Some(name)) => {
                        if let Some(pattern) = contains
                            && !name.contains(pattern.as_str())
                        {
                            return Err(format!(
                                "assert_download_toast failed: filename {name:?} does not contain {pattern:?}"
                            ));
                        }
                    }
                    (false, None) => {}
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_download_toast visible={visible} name={actual:?} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertTabOrder { patterns } => {
                let ids = app.smoke_tab_layout_ids();
                if ids.len() != patterns.len() {
                    return Err(format!(
                        "assert_tab_order failed: {} visible tabs, expected {} (urls: {:?})",
                        ids.len(),
                        patterns.len(),
                        ids.iter()
                            .map(|id| app.smoke_tab_url(*id).unwrap_or_default())
                            .collect::<Vec<_>>()
                    ));
                }
                for (index, pattern) in patterns.iter().enumerate() {
                    let url = app.smoke_tab_url(ids[index]).unwrap_or_default();
                    if !url.contains(pattern.as_str()) {
                        return Err(format!(
                            "assert_tab_order failed: visible slot {index} is {url}, expected to contain {pattern}"
                        ));
                    }
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_tab_order slots={} status=passed",
                    self.next + 1,
                    patterns.len()
                );
                Ok(())
            }
            Step::AssertTabTooltip { visible, contains } => {
                let actual = app.smoke_tab_tooltip();
                match (visible, actual.as_ref()) {
                    (true, None) => {
                        return Err("assert_tab_tooltip failed: tooltip not visible".to_string());
                    }
                    (false, Some(content)) => {
                        return Err(format!(
                            "assert_tab_tooltip failed: tooltip still visible with {content:?}"
                        ));
                    }
                    (true, Some((title, url))) => {
                        if let Some(pattern) = contains
                            && !title.contains(pattern.as_str())
                            && !url.contains(pattern.as_str())
                        {
                            return Err(format!(
                                "assert_tab_tooltip failed: tooltip ({title:?}, {url:?}) does not contain {pattern:?}"
                            ));
                        }
                    }
                    (false, None) => {}
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_tab_tooltip visible={visible} content={actual:?} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertClosedTabToast { visible, contains } => {
                let actual = app.smoke_closed_tab_toast();
                match (visible, actual.as_ref()) {
                    (true, None) => {
                        return Err("assert_closed_tab_toast failed: toast not visible".to_string());
                    }
                    (false, Some(label)) => {
                        return Err(format!(
                            "assert_closed_tab_toast failed: toast still visible with {label:?}"
                        ));
                    }
                    (true, Some(label)) => {
                        if let Some(pattern) = contains
                            && !label.contains(pattern.as_str())
                        {
                            return Err(format!(
                                "assert_closed_tab_toast failed: label {label:?} does not contain {pattern:?}"
                            ));
                        }
                    }
                    (false, None) => {}
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_closed_tab_toast visible={visible} label={actual:?} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::ClickClosedTabToast => {
                let Some((bx, by, bw, bh)) =
                    app.closed_tab_toast_undo_rect_for(app.physical_size.0, app.physical_size.1)
                else {
                    return Err("click_closed_tab_toast: undo toast is not visible".to_string());
                };
                let (cx, cy) = ((bx + bw / 2.0) as f64, (by + bh / 2.0) as f64);
                app.handle_mouse_move(cx, cy);
                app.handle_mouse_click(cx, cy, true, "Left");
                app.handle_mouse_click(cx, cy, false, "Left");
                tracing::info!(
                    "UI_DRIVER_STEP step={} click_closed_tab_toast cx={cx} cy={cy}",
                    self.next + 1
                );
                Ok(())
            }
            Step::HoverTab { index } => {
                let Some((cx, cy)) = app.smoke_tab_center(*index) else {
                    return Err(format!(
                        "hover_tab: no tab at index {index} (rendered layout has {} tabs)",
                        app.smoke_tab_ids().len()
                    ));
                };
                app.handle_mouse_move(cx, cy);
                tracing::info!(
                    "UI_DRIVER_STEP step={} hover_tab index={index} cx={cx} cy={cy}",
                    self.next + 1
                );
                Ok(())
            }
            Step::ClickTabClose { index } => {
                let id = app
                    .smoke_tab_ids()
                    .get(*index)
                    .copied()
                    .ok_or_else(|| format!("click_tab_close: no tab at index {index}"))?;
                let Some((tx, tw)) = app.smoke_tab_rect(id) else {
                    return Err(format!("click_tab_close: no rendered rect for tab at index {index}"));
                };
                let s = app.scale_factor;
                // 命中区与 app_input 标签 close 按钮同源：close_x = tx + tw - 24s，
                // y 以标签条中线为准（±TAB_CLOSE_SIZE/2 内）。
                let cx = tx + tw - 24.0 * s + crate::layout::TAB_CLOSE_SIZE * s * 0.5;
                let cy = (crate::layout::TAB_BAR_TOP_INSET + crate::layout::TAB_BAR_HEIGHT * 0.5) * s;
                app.handle_mouse_move(cx as f64, cy as f64);
                app.handle_mouse_click(cx as f64, cy as f64, true, "Left");
                app.handle_mouse_click(cx as f64, cy as f64, false, "Left");
                tracing::info!(
                    "UI_DRIVER_STEP step={} click_tab_close index={index} cx={cx} cy={cy}",
                    self.next + 1
                );
                Ok(())
            }
            Step::AssertAddressBarFocus { focused } => {
                let actual = app.smoke_address_bar_focused();
                if actual != *focused {
                    return Err(format!(
                        "assert_addressbar_focus failed: focused={actual}, expected {focused}"
                    ));
                }
                tracing::info!(
                    "UI_DRIVER_STEP step={} assert_addressbar_focus focused={actual} status=passed",
                    self.next + 1
                );
                Ok(())
            }
            Step::ClickMenuItem { id } => {
                if !app.smoke_context_menu_visible() {
                    return Err(format!("click_menu_item: context menu is not open (id {id:?})"));
                }
                let ids = app.smoke_context_menu_item_ids();
                let Some(idx) = ids.iter().position(|item| item == id) else {
                    return Err(format!("click_menu_item: no item {id:?} in menu {ids:?}"));
                };
                // 行中心与 context_menu_hit_test 同源：分隔行按紧凑高度累加。
                let s = app.scale_factor;
                let row_h = crate::layout::CONTEXT_MENU_ROW_HEIGHT * s;
                let sep_h = crate::layout::CONTEXT_MENU_SEPARATOR_HEIGHT * s;
                let Some((mx, my)) = app.smoke_context_menu_origin() else {
                    return Err("click_menu_item: menu has no origin".to_string());
                };
                let offset: f32 = ids[..idx]
                    .iter()
                    .map(|item| if item == "-" { sep_h } else { row_h })
                    .sum();
                let cx = (mx + crate::layout::CONTEXT_MENU_WIDTH * s * 0.5) as f64;
                let cy = (my + offset + row_h * 0.5) as f64;
                app.handle_mouse_click(cx, cy, true, "Left");
                app.handle_mouse_click(cx, cy, false, "Left");
                tracing::info!(
                    "UI_DRIVER_STEP step={} click_menu_item={id} cx={cx} cy={cy}",
                    self.next + 1
                );
                Ok(())
            }
            Step::ClickNewTabButton { button } => {
                let (bx, by, bw, bh) = app.smoke_new_tab_button_rect();
                let (cx, cy) = ((bx + bw / 2.0) as f64, (by + bh / 2.0) as f64);
                app.handle_mouse_move(cx, cy);
                app.handle_mouse_click(cx, cy, true, button);
                app.handle_mouse_click(cx, cy, false, button);
                tracing::info!(
                    "UI_DRIVER_STEP step={} click_new_tab_button button={button} cx={cx} cy={cy}",
                    self.next + 1
                );
                Ok(())
            }
            Step::BenchScroll {
                x,
                y,
                dy,
                steps,
                interval_ms,
                min_fps,
            } => {
                self.pending = Some(Pending::BenchScroll {
                    x: *x,
                    y: *y,
                    dy: *dy,
                    remaining: *steps,
                    interval: Duration::from_millis(*interval_ms),
                    next_fire: Instant::now(),
                    first_fire: None,
                    last_fire: None,
                    start_frames: app.smoke_presented_frames(),
                    min_fps: *min_fps,
                });
                tracing::info!(
                    "UI_DRIVER_STEP step={} bench_scroll x={x} y={y} dy={dy} steps={steps} interval_ms={interval_ms} min_fps={min_fps}",
                    self.next + 1
                );
                Ok(())
            }
        }
    }

    /// 推进滚动帧率基准：按间隔派发到期滚轮事件；全部派发并静置后统计区间
    /// 呈现帧数与墙钟，写入 bench-scroll.json 并按 `min_fps` 下限裁决。
    /// 返回 `true` 表示基准完成。
    fn advance_scroll_bench(&mut self, app: &mut BrowserApp) -> Result<bool, String> {
        {
            let Some(Pending::BenchScroll {
                x,
                y,
                dy,
                remaining,
                interval,
                next_fire,
                first_fire,
                last_fire,
                ..
            }) = &mut self.pending
            else {
                return Ok(true);
            };
            let now = Instant::now();
            while *remaining > 0 && now >= *next_fire {
                let delta = zero_host_runtime::event::MouseScrollDelta::LineDelta(0.0, *dy);
                app.handle_scroll(delta, *x, *y);
                if first_fire.is_none() {
                    *first_fire = Some(now);
                }
                *last_fire = Some(now);
                *remaining -= 1;
                *next_fire += *interval;
            }
        }
        let Some(Pending::BenchScroll {
            remaining,
            first_fire,
            last_fire,
            start_frames,
            min_fps,
            ..
        }) = &self.pending
        else {
            return Ok(true);
        };
        let (Some(first), Some(last)) = (*first_fire, *last_fire) else {
            return Ok(false);
        };
        if *remaining > 0 || Instant::now() < last + SCROLL_BENCH_SETTLE {
            return Ok(false);
        }
        let frames = app.smoke_presented_frames().saturating_sub(*start_frames);
        let elapsed = Instant::now().duration_since(first);
        let fps = frames as f32 / elapsed.as_secs_f32().max(0.001);
        tracing::info!(
            "UI_DRIVER_SCROLL_BENCH frames={frames} elapsed_ms={} fps={fps:.1}",
            elapsed.as_millis()
        );
        self.write_bench_result(frames, elapsed, fps);
        if fps < *min_fps {
            return Err(format!(
                "bench_scroll failed: {frames} frames in {} ms = {fps:.1} fps, below min_fps {min_fps}",
                elapsed.as_millis()
            ));
        }
        Ok(true)
    }

    fn write_bench_result(&self, frames: u64, elapsed: Duration, fps: f32) {
        let path = self.output_dir.join("bench-scroll.json");
        let payload = serde_json::json!({
            "schema_version": 1,
            "frames": frames,
            "elapsed_ms": elapsed.as_millis() as u64,
            "fps": (fps * 10.0).round() / 10.0,
        });
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, serde_json::to_vec_pretty(&payload).unwrap_or_default());
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

    /// TV-4：assert_tab_order 非字符串项解析期报错（不静默降级空串）。
    #[test]
    fn parse_assert_tab_order_rejects_non_string_items() {
        assert_eq!(
            Step::parse(r#"{"op":"assert_tab_order","contains":["a.html","b.html"]}"#, 1),
            Ok(Step::AssertTabOrder {
                patterns: vec!["a.html".into(), "b.html".into()]
            })
        );
        assert!(Step::parse(r#"{"op":"assert_tab_order","contains":["a.html",42]}"#, 1).is_err());
        assert!(Step::parse(r#"{"op":"assert_tab_order","contains":["a.html",null]}"#, 1).is_err());
        assert!(Step::parse(r#"{"op":"assert_tab_order"}"#, 1).is_err());
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
