//! WAAPI JS 动画注册表——JS `Element.animate()` 产物的引擎侧求值通道
//!（wptfyi-css「动画值进 gCS」刀，2026-11-15）。
//!
//! **断点**：shim `Animation` 对象此前纯 JS 态（playState/currentTime 齐全但引擎
//! 无感知）；渲染管线 AnimationClock 的插值只进渲染层副本，gCS 回调的级联路径
//! （parse+cascade）零动画钟参与——同步 `getComputedStyle` 对 paused+seek 形态
//! （WPT interpolation harness Web Animations 方法 100% 形态：
//! `animate(...)` → `pause()` → `currentTime = t`）恒读 underlying 值。
//! finished+fill forwards 形态另有 `_applyEndState` inline 写通道（R2965），不受本刀影响。
//!
//! **解法**：shim 经 `__zw_js_anim_*` 回调把 keyframes/timing/播放态登记到本表
//! （thread_local——host 回调仅 js worker 单线程执行，与代际槽同架构）；gCS 两通道
//!（selector / handle）序列化前咨询：命中属性返回插值串（applier 覆盖属性过
//! [`serialize_computed_property_with_sibling`] 规范化——与期望元素同序列化器，
//! 格式一致由构造保证；未覆盖属性原样透传，离散翻转端点串本就近乎规范形）。
//!
//! **零干扰**：表空（无 JS animate）时查询 O(1) 返 None，行为与刀前逐字节一致；
//! 独立 [`AnimationClock`] 实例，渲染管线时钟/事件派发零触及。
//!
//! spec：Web Animations §3.6 KeyframeEffect / §5.5 Element.animate。
//! https://drafts.csswg.org/web-animations-1/

use std::cell::{Cell, RefCell};
use std::time::Instant;

use serde_json::Value as Json;

use zero_css_parser::values::{AnimationDirectionValue, AnimationFillModeValue, TimingFunctionValue};
use zero_style_system::ComputedStyle;

use super::computed_style::serialize_computed_property_with_sibling;
use crate::animation::{AnimationClock, AnimationConfig, InterpolatedProperty};

/// 进程起点（单调墙钟锚——running 动画的求值时基；corpus 的 WAAPI 面全为
/// paused+seek 形态，running 精度仅服务真实页面近似）。
static MONOTONIC_START: std::sync::LazyLock<Instant> = std::sync::LazyLock::new(Instant::now);

#[derive(Debug, Clone)]
struct JsAnimEntry {
    /// 注册表 id（shim 持有，seek/playpause/cancel 回调键）。
    id: u64,
    /// 静态快照元素身份（`__zwSelector`）。selector 形 gCS 通道匹配键。
    sel: Option<String>,
    /// 动态元素身份（`__zwHandle`，createElement 产物）。handle 形 gCS 通道匹配键。
    hnd: Option<String>,
    /// 独立时钟内的元素键（每动画唯一，动画间零串扰）。
    clock_id: u64,
    /// 播放态。paused 时以 [`JsAnimEntry::hold_time`] 冻结求值（None = 冻结于 0——
    /// fresh pause 近似，FIXME(pause-mid-flight)：无 seek 的运行中暂停应冻结于
    /// pause 时刻，v1 以 0 近似，harness 面恒「先 pause 后 seek」不受影响）。
    paused: bool,
    /// seek 后的冻结求值点（秒，local time 含 delay）。`currentTime = t` 写入。
    hold_time: Option<f64>,
    /// play 时的墙钟锚（秒）。running 求值 = now − anchor。
    start_anchor: Option<f64>,
    /// 端点原文表（prop → tick 域 p=0/p=1 输出串去重集）——fallback 检测基准
    /// （与查询 tick 同源：合成 `@keyframes` 解析后的串化形，非 spec JSON 原文）。
    endpoints: std::collections::HashMap<String, Vec<String>>,
    /// 时钟域 duration/delay（秒）——开区间进度计算。
    dur_s: f64,
    delay_s: f64,
}

thread_local! {
    /// 注册序排列（同属性后注册覆盖先注册——Web Animations composite order 近似）。
    static ENTRIES: RefCell<Vec<JsAnimEntry>> = const { RefCell::new(Vec::new()) };
    /// 独立动画时钟——keyframes 注册 + `start_animation`/`tick` 全复用
    ///（AnimationState.start_time 由 config.current_time 锚定，paused_at 由
    /// [`AnimationClock::set_hold`] 写入；渲染管线时钟独立实例零干扰）。
    static CLOCK: RefCell<AnimationClock> = RefCell::new(AnimationClock::new());
    static NEXT_ID: Cell<u64> = const { Cell::new(1) };
}

/// 页面换代清表（[`crate::js_dom_bridge::clear_generation_cache`] 同点调用——
/// 导航/新 WebView 装新快照，旧页动画不跨文档泄漏）。
pub fn clear_all() {
    ENTRIES.with(|cell| cell.borrow_mut().clear());
    CLOCK.with(|cell| *cell.borrow_mut() = AnimationClock::new());
}

/// shimming 侧 `__zw_js_anim_register`：登记动画（keyframes 文本经合成
/// `@keyframes` 解析进独立时钟）。返回注册表 id（0 = 失败）。
pub fn register(sel: Option<&str>, hnd: Option<&str>, spec_json: &str) -> u64 {
    let Ok(spec) = serde_json::from_str::<Json>(spec_json) else {
        return 0;
    };
    let id = NEXT_ID.with(|cell| {
        let v = cell.get();
        cell.set(v + 1);
        v
    });
    let name = format!("__zw_js_anim_{id}");
    let Some(rule) = build_keyframes_rule(&name, spec.get("kf")) else {
        return 0;
    };
    CLOCK.with(|cell| cell.borrow_mut().register_keyframes(&rule));

    // timing（JS ms → 时钟域秒）。duration 'auto'/缺省 → 0（`max(0.001)` 防除零，
    // 进度溢出走 finished+fill 通道 = 瞬间完成语义，与 spec duration 0 一致）。
    let duration_s = spec.get("dur").and_then(json_f64).unwrap_or(0.0) / 1000.0;
    let delay_s = spec.get("delay").and_then(json_f64).unwrap_or(0.0) / 1000.0;
    let easing = spec.get("easing").and_then(Json::as_str).unwrap_or("linear");
    let timing_function =
        zero_css_parser::values::parse_transform::parse_timing_function(easing).unwrap_or(TimingFunctionValue::Linear);
    // iterations：null（JS Infinity）/<=0 → None（infinite）；其余 Some(n)。
    let iteration_count = spec
        .get("iter")
        .and_then(json_f64)
        .filter(|n| *n > 0.0 && n.is_finite());
    let direction = parse_direction(spec.get("dir").and_then(Json::as_str));
    let fill_mode = parse_fill(spec.get("fill").and_then(Json::as_str));

    let clock_id = id;
    let started = CLOCK.with(|cell| {
        cell.borrow_mut().start_animation(
            clock_id,
            &AnimationConfig {
                name: name.clone(),
                duration: duration_s,
                delay: delay_s,
                timing_function,
                iteration_count,
                direction,
                fill_mode,
                current_time: 0.0,
            },
        )
    });
    if !started {
        return 0;
    }
    // 端点原文捕获：直读合成规则的帧声明串（`block_to_point` 保真直通 tick 端点
    // 输出，同源同格式）。不走 tick 探针——tick 会把状态推进到 finished
    //（fill forwards 后续查询恒返回终帧，探针翻红实证），污染后续求值。
    let mut endpoints: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for frame in &rule.keyframes {
        for d in &frame.declarations {
            if d.property.eq_ignore_ascii_case("animation-timing-function") {
                continue;
            }
            let vals = endpoints.entry(d.property.clone()).or_default();
            if !vals.contains(&d.value) {
                vals.push(d.value.clone());
            }
        }
    }
    ENTRIES.with(|cell| {
        cell.borrow_mut().push(JsAnimEntry {
            id,
            sel: sel.map(str::to_string),
            hnd: hnd.map(str::to_string),
            clock_id,
            paused: false,
            hold_time: None,
            start_anchor: None,
            endpoints,
            dur_s: duration_s,
            delay_s,
        })
    });
    id
}

/// shimming 侧 `__zw_js_anim_seek`：`currentTime = t`（ms）→ 冻结求值点。
/// v1 近似：seek 即按 hold 求值（running seek 的继续推进语义 FIXME(seek-running)，
/// harness 面恒「先 pause 后 seek」不受影响）。time 空串 = null currentTime，忽略。
pub fn seek(id: u64, time_ms: &str) {
    let Ok(ms) = time_ms.parse::<f64>() else {
        return;
    };
    ENTRIES.with(|cell| {
        for e in cell.borrow_mut().iter_mut() {
            if e.id == id {
                e.hold_time = Some(ms / 1000.0);
                e.paused = true;
            }
        }
    });
}

/// shimming 侧 `__zw_js_anim_playpause`：`'1'` = pause（无 hold 时冻结于
/// now − anchor / 0），`'0'` = play（重锚墙钟、解冻结）。`now_ms` = Date.now()。
pub fn play_pause(id: u64, paused: &str, now_ms: &str) {
    let now = now_ms.parse::<f64>().unwrap_or(0.0) / 1000.0;
    ENTRIES.with(|cell| {
        for e in cell.borrow_mut().iter_mut() {
            if e.id != id {
                continue;
            }
            if paused == "1" {
                e.paused = true;
                // fresh pause（无 seek）：冻结于当前运行相位（无 anchor = 0 近似）。
                if e.hold_time.is_none() {
                    e.hold_time = Some(e.start_anchor.map_or(0.0, |a| (now - a).max(0.0)));
                }
            } else {
                e.paused = false;
                e.hold_time = None;
                e.start_anchor = Some(now);
            }
        }
    });
}

/// shimming 侧 `__zw_js_anim_cancel`：移除条目（时钟残条不再被查询，无害）。
pub fn cancel(id: u64) {
    ENTRIES.with(|cell| cell.borrow_mut().retain(|e| e.id != id));
}

/// gCS selector 通道咨询（`__zw_get_computed_style` 序列化点前）。命中返回覆盖值。
pub fn query_override_for_sel(
    sel: &str,
    prop: &str,
    base: &ComputedStyle,
    sibling: Option<(u64, u64)>,
) -> Option<String> {
    query_core(Some(sel), None, prop, base, sibling)
}

/// gCS handle 通道咨询（`__zw_get_computed_style_by_handle` 视图序列化点前）。
pub fn query_override_for_handle(
    hnd: &str,
    prop: &str,
    base: &ComputedStyle,
    sibling: Option<(u64, u64)>,
) -> Option<String> {
    query_core(None, Some(hnd), prop, base, sibling)
}

/// 求值核心：匹配条目按注册序逐个 tick（后值覆盖先值），命中 `prop` 经
/// 规范化返回。表空快速路径 O(1)。
fn query_core(
    sel: Option<&str>,
    hnd: Option<&str>,
    prop: &str,
    base: &ComputedStyle,
    sibling: Option<(u64, u64)>,
) -> Option<String> {
    if !ENTRIES.with(|cell| !cell.borrow().is_empty()) {
        return None;
    }
    let now = MONOTONIC_START.elapsed().as_secs_f64();
    let mut latest: Option<String> = None;
    ENTRIES.with(|entries| {
        CLOCK.with(|clock_cell| {
            let mut clock = clock_cell.borrow_mut();
            for e in entries.borrow().iter() {
                let matches = if let Some(s) = sel {
                    e.sel.as_deref() == Some(s)
                } else {
                    e.hnd.as_deref() == hnd
                };
                if !matches {
                    continue;
                }
                // 求值时间：hold（含 delay 的 local time）冻结；running 墙钟推进。
                let t = if let Some(h) = e.hold_time {
                    clock.set_hold(e.clock_id, Some(h));
                    h
                } else if e.paused {
                    clock.set_hold(e.clock_id, Some(0.0));
                    0.0
                } else {
                    clock.set_hold(e.clock_id, None);
                    if let Some(a) = e.start_anchor {
                        clock.set_start_time(e.clock_id, a);
                    }
                    now
                };
                // fallback 检测（A/B 定谳 2026-11-15，REGRESSED=245 归因）：开区间
                // 进度内 raw 恒等端点原文 = 引擎未真插值（关键字/calc-size/url 等
                // 离散 fallback，canonicalize 序列化还会变异裸串）——不采纳，回落
                // 既有 B0 通道（级联/终态 inline 写），零回归方向。端点钳制（p≤0
                // 或 p≥1）保留采纳：B1 对插值形端点子测的增益来源。
                let prog = if e.dur_s > 0.0 { (t - e.delay_s) / e.dur_s } else { 1.0 };
                let open_interval = prog > 0.0 && prog < 1.0;
                for p in clock.tick(e.clock_id, t) {
                    if p.name != prop {
                        continue;
                    }
                    let fallback = open_interval
                        && e.endpoints
                            .get(&p.name)
                            .is_some_and(|vals| vals.iter().any(|v| v == &p.value));
                    if !fallback {
                        latest = Some(p.value);
                    }
                }
            }
        });
    });
    latest.and_then(|raw| canonicalize(prop, &raw, base, sibling))
}

/// 插值串规范化：施加到基样式 clone 后过 gCS 序列化器——与期望元素（测试侧
/// inline 设值读 gCS）同通道，格式一致由构造保证（"0.5000" → "0.5"）。序列化
/// 无变化两种形态都返 None 回落既有 gCS 通道（零回归方向）：
/// ① applier 未覆盖该属性（引擎不支持的属性，期望元素 gCS 为空串——B1 实测
///    font-variant-ligatures 期望 "" 对 raw 透传裸串 = REGRESSED 伪源）；
/// ② 施加后序列化恒等（动画值 ≡ 基值，回落同一序列化结果语义不变）。
/// A/B 定谳 2026-11-15：raw 透传臂在端点钳制形态（p≥1）撞空串期望。
fn canonicalize(prop: &str, raw: &str, base: &ComputedStyle, sibling: Option<(u64, u64)>) -> Option<String> {
    let mut tmp = base.clone();
    let ser0 = serialize_computed_property_with_sibling(&tmp, prop, sibling);
    AnimationClock::apply_to_computed_style(
        &[InterpolatedProperty {
            name: prop.to_string(),
            value: raw.to_string(),
        }],
        &mut tmp,
    );
    let ser1 = serialize_computed_property_with_sibling(&tmp, prop, sibling);
    (ser1 != ser0).then_some(ser1)
}

/// 合成 `@keyframes` 文本并解析出规则（`0%{prop:val;...}100%{...}`——offset 百分
/// 比三位小数去尾零）。帧集空 / 解析无 keyframes → None。
fn build_keyframes_rule(name: &str, kf: Option<&Json>) -> Option<zero_css_parser::ast::KeyframesRule> {
    let frames = kf?.as_array()?;
    let mut css = format!("@keyframes {name}{{");
    for frame in frames {
        let offset = frame.get("o").and_then(json_f64).unwrap_or(0.0);
        let props = frame.get("p").and_then(Json::as_object);
        let Some(props) = props else { continue };
        css.push_str(&format!("{}%{{", trim_float(offset * 100.0)));
        for (k, v) in props {
            let value = match v {
                Json::String(s) => s.clone(),
                Json::Number(n) => n.to_string(),
                _ => continue,
            };
            // 声明值内 `}` 破坏文本结构——保守跳过该帧（解析失败面，宁缺勿错）。
            if value.contains('}') || value.contains('{') {
                continue;
            }
            css.push_str(&format!("{k}:{value};"));
        }
        css.push('}');
    }
    css.push('}');
    let sheet = zero_css_parser::Parser::parse_stylesheet(&css);
    sheet.rules.into_iter().find_map(|rule| match rule {
        zero_css_parser::ast::Rule::Keyframes(kf) => Some(kf),
        _ => None,
    })
}

/// f64 抽取（JSON number；字符串形态数字也收——shim `String(...)` 化字段）。
fn json_f64(v: &Json) -> Option<f64> {
    match v {
        Json::Number(n) => n.as_f64(),
        Json::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

/// offset 百分比文本：整数无小数点，小数去尾零（0.5 → "50"，0.123 → "12.3"）。
fn trim_float(v: f64) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" || s.is_empty() {
        "0".to_string()
    } else {
        s.to_string()
    }
}

fn parse_direction(s: Option<&str>) -> AnimationDirectionValue {
    match s.unwrap_or("normal") {
        "reverse" => AnimationDirectionValue::Reverse,
        "alternate" => AnimationDirectionValue::Alternate,
        "alternate-reverse" => AnimationDirectionValue::AlternateReverse,
        _ => AnimationDirectionValue::Normal,
    }
}

fn parse_fill(s: Option<&str>) -> AnimationFillModeValue {
    match s.unwrap_or("none") {
        "forwards" => AnimationFillModeValue::Forwards,
        "backwards" => AnimationFillModeValue::Backwards,
        "both" => AnimationFillModeValue::Both,
        _ => AnimationFillModeValue::None,
    }
}
