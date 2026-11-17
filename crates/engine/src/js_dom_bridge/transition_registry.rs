//! CSS Transition gCS 注册表——`transition-*` 触发的属性过渡在**同步读窗口**的
//! 求值通道（wptfyi-css「Transition 通道可见性」刀，2026-11-16）。
//!
//! **断点**：渲染管线 `TransitionClock` 的插值只进渲染层副本（泵帧 `tick` +
//! `apply_to_computed_style`），且 `start_transitions` 挂在泵首帧——corpus T 测试
//! （interpolation-testcommon.js CSS Transitions 方法）在**同一脚本轮次**内
//! `setProperty(from)` → gCS 读（flush 基线）→ 写 `transition-*` + `setProperty(to)`
//! → gCS 读断言，无泵帧介入：基线 seed 后 `start_transitions` 要等下一帧，断言读
//! 恒先发生 → gCS 恒落 cascaded 终值（六套件 T 面 7,846F 的根因，十八轮 §7 定谳）。
//!
//! **解法**（镜像 jsanim 十轮刀已证模式）：gCS 两通道（selector/handle）读点维护
//! **per-identity 最近观测 computed 快照**（`LAST_STYLE`——css-transitions §2
//! before-change style 近似；**必须按 identity 键而非文档级槽 diff**：corpus
//! interpolation harness 两阶段结构——全部 target 先 interpolate 写后 measure 读——
//! 下，文档级 handle 视图槽/per-selector map 会被穿插的其他 target 写+读重建/清空
//! 到终态，前值捕获恒落 to 态；per-identity 快照只在本元素读点更新，穿插写零干扰），
//! 读点比较快照与新 computed，对 `transition-property` 声明且值变化的属性登记端点对
//! （harness 的显式 flush 读保证快照即 from 态）；序列化前咨询——活跃
//! 条目按墙钟求 `progress`（harness `duration=100s, delay=-50s` 钉死 progress=0.5；
//! `at` 编码进 timing 函数，eased(0.5)=at 精确——对称 bezier 代数 y(0.5)=0.75c+0.125，
//! createEasing 控制点 c=(8at−1)/6 精确反解），经宽松插值器产出覆盖值，
//! `canonicalize`（jsanim 共享）施于基样式 clone 过 gCS 序列化器，applier 不支持的
//! 形（如 shape() 串）恒等返 None 回落 echo（零回归方向）。
//!
//! **零干扰**：非插值对（css-transitions §starting-of-transitions 非 animatable
//! 对不启动过渡）登记期即拒——条目表只含严格可插值对，echo 池（终值巧合 P）
//! 结构零扰动；表空查询 O(1) 返 None。渲染管线 `TransitionClock`/事件派发零触及
//!（独立 thread_local 实例，导航换代清表）。
//!
//! spec：css-transitions-1 §starting-of-transitions / §animations（transition 行为）、
//! css-values-4 §8.5.6（离散插值）、css-easing-4（timing 求值）。
//! https://drafts.csswg.org/css-transitions-1/

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Instant;

use zero_css_parser::values::{StepPosition, TimingFunctionValue};
use zero_dom::Document;
use zero_style_system::ComputedStyle;

use super::computed_style::serialize_computed_property_with_sibling;
use super::js_anim_registry::canonicalize;
use crate::animation::{apply_timing_function, interpolate_property_value, interpolate_property_value_strict};

/// 进程起点（单调墙钟锚——running 过渡求值时基；corpus 同轮次注册→咨询 elapsed≈0）。
static MONOTONIC_START: std::sync::LazyLock<Instant> = std::sync::LazyLock::new(Instant::now);

/// 条目世界域——gCS 两通道各自的视图状态独立（NodeIds 跨代际槽/视图槽两份 doc
/// 各自编号，同一逻辑元素在两域身份不同）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum World {
    /// selector 通道（per-selector 缓存域，`__zw_get_computed_style`）。
    Gen,
    /// handle 通道（handle 视图槽域，`__zw_get_computed_style_by_handle`）。
    View,
}

#[derive(Debug, Clone)]
struct TransitionEntry {
    world: World,
    /// 身份串：Gen = 查询 selector 原文；View = handle 原文。
    identity: String,
    /// 过渡属性名（kebab，与咨询 `prop` 同域精确匹配）。
    property: String,
    /// 前值（css-transitions §2 before-change style——gCS 序列化形）。
    from: String,
    /// 终值（gCS 序列化形）。
    to: String,
    /// 时长（秒）。
    duration_s: f64,
    /// 延迟（秒，可为负——harness -50s 钉 progress=0.5）。
    delay_s: f64,
    timing: TimingFunctionValue,
    /// 登记时刻（[`MONOTONIC_START`] 域秒）。
    start_s: f64,
}

thread_local! {
    /// 登记序排列（同 world+identity+property 后注册覆盖先注册——新过渡替换旧过渡，
    /// css-transitions §starting-of-transitions 同属性替换语义）。
    static ENTRIES: RefCell<Vec<TransitionEntry>> = const { RefCell::new(Vec::new()) };

    /// per-identity 最近一次 gCS 观测的 computed（before-change style 近似——
    /// css-transitions §2）。**必须按 identity 键而非文档级槽 diff**：corpus
    /// interpolation harness 两阶段结构（全部 target 先 interpolate 写、后
    /// measure 读）下，文档级 handle 视图槽/per-selector map 会被穿插的其他
    /// target 写+读重建/清空到终态，前值捕获恒落 to 态（from==to 全拒）；
    /// per-identity 快照只在**本元素**的读点更新，穿插写零干扰。
    static LAST_STYLE: RefCell<HashMap<(World, String), ComputedStyle>> =
        RefCell::new(HashMap::new());
}

/// 页面换代清表（`clear_generation_cache` 同点调用——导航/新 WebView 装新快照，
/// 旧页过渡不跨文档泄漏）。
pub fn clear_all() {
    ENTRIES.with(|cell| cell.borrow_mut().clear());
    LAST_STYLE.with(|cell| cell.borrow_mut().clear());
}

/// 取走该 identity 的最近观测 computed（读点前值——css-transitions §2
/// before-change style 基线；None = 本轮次首次读，无登记基线）。
pub(crate) fn take_last_style(world: World, identity: &str) -> Option<ComputedStyle> {
    LAST_STYLE.with(|cell| cell.borrow_mut().remove(&(world, identity.to_string())))
}

/// 存回该 identity 本次观测 computed（登记判定后调用——本轮读即成下一轮前值）。
pub(crate) fn store_last_style(world: World, identity: &str, style: &ComputedStyle) {
    LAST_STYLE.with(|cell| cell.borrow_mut().insert((world, identity.to_string()), style.clone()));
}

/// 换代边界登记（gCS 读路径刷新调用）：比较前值/新值 computed，对
/// `transition-property` 声明且值变化的属性登记过渡条目。非插值对 / duration≤0 /
/// `none` 属性 / 端点序列化同值或空值一律跳过（= 刀前行为，echo 池零扰动）；
/// `all` 按可插值清单通配展开（css-transitions §2）。
pub(crate) fn register_from_style_change(
    world: World,
    identity: &str,
    prev: &ComputedStyle,
    new: &ComputedStyle,
    doc: &Document,
    nid: zero_dom::NodeId,
) {
    if new.transition_property.is_empty() {
        return;
    }
    let sibling = Some(zero_style_system::sibling_position(doc, nid));
    for (i, property) in new.transition_property.iter().enumerate() {
        if property == "none" || property.is_empty() {
            continue;
        }
        // `all` 通配（css-transitions §2：`all` 匹配一切可过渡 longhand）——按
        // 可插值属性清单逐项探测端点变化并各自登记（duration/delay/timing 取
        // `all` 项自身位）。非插值对/端点同值在逐项门过滤，echo 池零扰动。
        if property == "all" {
            for prop in TRANSITIONABLE_PROPERTIES {
                register_property_if_changed(
                    world,
                    identity,
                    prop,
                    prev,
                    new,
                    sibling,
                    new.transition_duration.get(i).copied().unwrap_or(0.0),
                    new.transition_delay.get(i).copied().unwrap_or(0.0),
                    new.transition_timing_function
                        .get(i)
                        .cloned()
                        .unwrap_or(TimingFunctionValue::Ease),
                );
            }
            continue;
        }
        register_property_if_changed(
            world,
            identity,
            property,
            prev,
            new,
            sibling,
            new.transition_duration.get(i).copied().unwrap_or(0.0),
            new.transition_delay.get(i).copied().unwrap_or(0.0),
            new.transition_timing_function
                .get(i)
                .cloned()
                .unwrap_or(TimingFunctionValue::Ease),
        );
    }
}

/// 严格可插值属性清单（[`interpolate_property_value_strict`] 臂全集）——`all`
/// 通配展开的探测面。清单外属性 pair 落 None 即不启动过渡（css-transitions
/// §starting-of-transitions），与显式声明同口径。
const TRANSITIONABLE_PROPERTIES: [&str; 60] = [
    "backdrop-filter",
    "background",
    "background-color",
    "border-color",
    "bottom",
    "clip-path",
    "color",
    "column-count",
    "column-height",
    "column-rule-color",
    "column-rule-inset-cap-end",
    "column-rule-inset-cap-start",
    "column-rule-inset-junction-end",
    "column-rule-inset-junction-start",
    "column-rule-width",
    "columns",
    "column-width",
    "filter",
    "font-size",
    "font-size-adjust",
    "font-weight",
    "height",
    "left",
    "letter-spacing",
    "margin-bottom",
    "margin-left",
    "margin-right",
    "margin-top",
    "max-height",
    "max-width",
    "min-height",
    "min-width",
    "offset-anchor",
    "offset-distance",
    "offset-path",
    "offset-position",
    "offset-rotate",
    "opacity",
    "padding-bottom",
    "padding-left",
    "padding-right",
    "padding-top",
    "right",
    "rotate",
    "row-rule-color",
    "row-rule-inset-cap-end",
    "row-rule-inset-cap-start",
    "row-rule-inset-junction-end",
    "row-rule-inset-junction-start",
    "row-rule-width",
    "scale",
    "shape-image-threshold",
    "shape-margin",
    "shape-outside",
    "text-indent",
    "top",
    "transform",
    "translate",
    "width",
    "word-spacing",
];

/// 单属性登记（显式声明与 `all` 通配展开共用）：duration≤0 / 端点空串同值 /
/// 严格门不可插值对跳过；同 (world, identity, property) 条目替换。
fn register_property_if_changed(
    world: World,
    identity: &str,
    property: &str,
    prev: &ComputedStyle,
    new: &ComputedStyle,
    sibling: Option<(u64, u64)>,
    duration: f64,
    delay: f64,
    timing: TimingFunctionValue,
) {
    if duration <= 0.0 {
        return;
    }
    // 端点取 gCS 序列化形（与断言读同序列化器——格式一致由构造保证）。
    let from = serialize_computed_property_with_sibling(prev, property, sibling);
    let to = serialize_computed_property_with_sibling(new, property, sibling);
    if from.is_empty() || to.is_empty() || from == to {
        return;
    }
    // 仅登记严格可插值对（css-transitions §starting-of-transitions：非
    // animatable 对不启动过渡；t=0.5 仅作端点解析探针，进度无关）。
    if interpolate_property_value_strict(property, &from, &to, 0.5).is_none() {
        return;
    }
    let entry = TransitionEntry {
        world,
        identity: identity.to_string(),
        property: property.to_string(),
        from,
        to,
        duration_s: duration,
        delay_s: delay,
        timing,
        start_s: MONOTONIC_START.elapsed().as_secs_f64(),
    };
    ENTRIES.with(|cell| {
        let mut entries = cell.borrow_mut();
        entries.retain(|e| e.world != entry.world || e.identity != entry.identity || e.property != entry.property);
        entries.push(entry);
    });
}

/// gCS selector 通道咨询（`__zw_get_computed_style` 序列化点，jsanim consult 位旁）。
pub fn query_override_for_sel(
    sel: &str,
    prop: &str,
    base: &ComputedStyle,
    sibling: Option<(u64, u64)>,
) -> Option<String> {
    query_core(World::Gen, sel, prop, base, sibling)
}

/// gCS handle 通道咨询（`__zw_get_computed_style_by_handle` 视图序列化点）。
pub fn query_override_for_handle(
    hnd: &str,
    prop: &str,
    base: &ComputedStyle,
    sibling: Option<(u64, u64)>,
) -> Option<String> {
    query_core(World::View, hnd, prop, base, sibling)
}

/// 条目 timing 本地求值——[`apply_timing_function`] 的 Steps 臂未吃 position
/// 参数（恒 jump-end floor，`steps(1, start)` 在 t=0.5 误落 0）。本表咨询按
/// css-easing-1 §step-easing-functions 规范本地求值；共享函数修正是全局行为
/// 面（jsanim 同载），独立刀验证，此处不动。其余 timing 形直接委托。
// https://drafts.csswg.org/css-easing-1/#step-easing-functions
fn entry_eased(t: f64, tf: &TimingFunctionValue) -> f64 {
    match tf {
        TimingFunctionValue::Steps(n, pos) => {
            let n = (*n).max(1) as f64;
            match pos {
                Some(StepPosition::Start) if t > 0.0 => ((t * n).ceil() / n).min(1.0),
                Some(StepPosition::Start) => 0.0,
                Some(StepPosition::Both) if t <= 0.0 => 0.0,
                Some(StepPosition::Both) if t >= 1.0 => 1.0,
                Some(StepPosition::Both) => ((t * n).ceil() / n).min(1.0),
                Some(StepPosition::None) => {
                    let m = (n - 1.0).max(1.0);
                    ((t * m).round() / m).clamp(0.0, 1.0)
                }
                // End（默认 jump-end，与引擎既有语义一致）。
                _ => ((t * n).floor() / n).min(1.0),
            }
        }
        _ => apply_timing_function(t, tf),
    }
}

/// filter 量钳制——滤单内 [`CLAMPED_UNIT_INTERVAL`] 族量域 [0,1]（css-filters-1
/// §filter-functions 量值界），eased 外推越界（overshoot bezier / 负向外推）时
/// 钳回界内。其余族不钳：brightness/contrast/saturate 域 [0,∞)、blur/hue-rotate/
/// drop-shadow 无界（外推真增益面，v3 A/B 实测 63 例全在此侧零冲突）。
// https://drafts.fxtf.org/css-filters-1/#filter-functions
fn clamp_filter_unit_interval_amounts(raw: &str) -> String {
    const CLAMPED_UNIT_INTERVAL: [&str; 4] = ["grayscale", "sepia", "invert", "opacity"];
    let bytes = raw.as_bytes();
    let mut out = String::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        // 逐字节位判定：当前位命中钳制族函数调用（词边界 + 紧跟 '('）即钳制
        // 其量——跳转式 find 会把同轮前缀里的其他族名原样推入（单测实证）。
        let hit = CLAMPED_UNIT_INTERVAL.iter().find(|name| {
            raw[i..].starts_with(*name)
                && raw[i + name.len()..].starts_with('(')
                && (i == 0 || !bytes[i - 1].is_ascii_alphabetic())
        });
        let Some(name) = hit else {
            let ch = raw[i..].chars().next().expect("i 在串内非空");
            out.push(ch);
            i += ch.len_utf8();
            continue;
        };
        let after_name = i + name.len();
        let Some(close_rel) = raw[after_name..].find(')') else {
            out.push_str(&raw[i..]);
            break;
        };
        let close = after_name + close_rel;
        out.push_str(&raw[i..after_name + 1]);
        let amount = raw[after_name + 1..close].trim();
        match amount.parse::<f64>() {
            // 钳制后按数值形回写（f64 Display 整值无小数点，与序列化形一致）。
            Ok(x) => out.push_str(&format!("{}", x.clamp(0.0, 1.0))),
            // 非数值量（% 形等）原样保留（FIXME(percent-amount)）。
            Err(_) => out.push_str(amount),
        }
        i = close;
    }
    out
}

/// 求值核心：匹配条目按墙钟求 progress → eased → 插值 → 规范化。表空 O(1) None。
///
/// 进度语义（css-transitions-1 §animations——[delay, delay+duration] 活跃区间）：
/// delay 期（active<0）progress=0 → 值=from（start 值前填）；完成（progress≥1）
/// eased=1 → 值=to——canonicalize 对 to 与级联基值恒等返 None → 回落 echo，终态
/// 无须显式摘除条目（新过渡替换 / 换代清表负责生命周期）。
fn query_core(
    world: World,
    identity: &str,
    prop: &str,
    base: &ComputedStyle,
    sibling: Option<(u64, u64)>,
) -> Option<String> {
    if !ENTRIES.with(|cell| !cell.borrow().is_empty()) {
        return None;
    }
    let now = MONOTONIC_START.elapsed().as_secs_f64();
    let entries = ENTRIES.with(|cell| cell.borrow().clone());
    for e in entries.iter() {
        if e.world != world || e.identity != identity || e.property != prop {
            continue;
        }
        let active = now - e.start_s - e.delay_s;
        let progress = if e.duration_s > 0.0 {
            (active / e.duration_s).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let eased = entry_eased(progress, &e.timing);
        // eased==1.0（step 越点 / 进度完结）：过渡值恒等 `to`，级联基值即 `to`
        // ——回落 echo 恒正确，且规避本地插值端点序列化形差（v3 A/B 定谳：
        // grad→deg、cm→mm 归一形不一致 hue-rotate 2 REG + offset-path 端点
        // 8 DOWNS 全此类）。严格 `==`（steps 越点精确 1.0；bezier 外推 eased>1
        // 走插值+钳制臂，不落此门——63 例无界族外推真增益保持）。
        // rule-\* 四属性例外：插值器端点含广播/有限 repeat 展开语义（to 单值
        // 对 from 多值位输出 `to, to`；computed 串化已展开有限 repeat）——echo
        // 原样形欠广播/展开位，回落插值臂（css-gaps-1 §column-row-rule-width）。
        // rule-inset 八槽例外（刀 E v4）：跨单位端点 at(1) 须出 calc 混合终形
        // （css-values-4 §8.5.6 calc-mix——`-100%`→`1px` at(1) 期望
        // `calc(0% + 1px)`，echo 级联形 `1px` 欠混合形；同单位/关键字端点经
        // 插值臂 t=1 与 echo 数值恒等，canonicalize 归一零截尾后同形）。
        if eased == 1.0
            && !matches!(
                prop,
                "column-rule-width"
                    | "row-rule-width"
                    | "column-rule-color"
                    | "row-rule-color"
                    | "column-rule-inset-cap-start"
                    | "column-rule-inset-cap-end"
                    | "column-rule-inset-junction-start"
                    | "column-rule-inset-junction-end"
                    | "row-rule-inset-cap-start"
                    | "row-rule-inset-cap-end"
                    | "row-rule-inset-junction-start"
                    | "row-rule-inset-junction-end"
            )
        {
            return None;
        }
        let mut raw = interpolate_property_value(&e.property, &e.from, &e.to, eased);
        if e.property == "filter" || e.property == "backdrop-filter" {
            raw = clamp_filter_unit_interval_amounts(&raw);
        }
        return canonicalize(prop, &raw, base, sibling);
    }
    None
}

/// A/B 快照二进制 provenance 标记（strings 抽验 B0/B1 装载面；测试引用保存活）。
#[allow(dead_code)]
pub(crate) static TRANSITION_SYNC_READ_MARKER: &str = "zw-transition-sync-read-2026-11-16";

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::values::{LengthValue, TimingFunctionValue};
    use zero_dom::parse_html;

    fn make_pair(
        width_from: f64,
        width_to: f64,
        timing: TimingFunctionValue,
        delay: f64,
    ) -> (ComputedStyle, ComputedStyle) {
        let mut prev = ComputedStyle::default();
        let mut new = ComputedStyle::default();
        prev.width = LengthValue::Px(width_from);
        new.width = LengthValue::Px(width_to);
        new.transition_property = vec!["width".to_string()];
        new.transition_duration = vec![100.0];
        new.transition_delay = vec![delay];
        new.transition_timing_function = vec![timing];
        (prev, new)
    }

    /// harness 语义端到端钉：duration=100s + delay=-50s 钉 progress=0.5，linear
    /// 下中点=端点中值；at 经 timing 编码（bezier 对称点 y(0.5)=at 精确）。
    #[test]
    fn test_harness_sync_read_midpoint() {
        clear_all();
        let html = r#"<html><body><div id="t"></div></body></html>"#;
        let doc = parse_html(html);
        let nid = crate::find_by_selector(&doc, "#t").expect("target");
        let (prev, new) = make_pair(50.0, 150.0, TimingFunctionValue::Linear, -50.0);
        register_from_style_change(World::Gen, "#t", &prev, &new, &doc, nid);

        // 同轮次咨询（elapsed≈0 → progress≈0.5）：线性中点 100px。
        let base = new.clone();
        let v = query_override_for_sel("#t", "width", &base, None).expect("mid override");
        assert_eq!(v, "100px", "linear progress=0.5 中点，got {v}");
        clear_all();
    }

    /// at 编码进 timing：cubic-bezier 对称控制点 c=(8at−1)/6 在 progress=0.5 处
    /// eased=at（createEasing harness 反解一致）——at=0.3 → width 50→150 中值 80px。
    #[test]
    fn test_easing_encoded_at() {
        clear_all();
        let html = r#"<html><body><div id="t"></div></body></html>"#;
        let doc = parse_html(html);
        let nid = crate::find_by_selector(&doc, "#t").expect("target");
        // at=0.3 → c=(2.4-1)/6≈0.2333
        let timing = TimingFunctionValue::CubicBezier(0.0, (8.0 * 0.3 - 1.0) / 6.0, 1.0, (8.0 * 0.3 - 1.0) / 6.0);
        let (prev, new) = make_pair(50.0, 150.0, timing, -50.0);
        register_from_style_change(World::Gen, "#t", &prev, &new, &doc, nid);
        let base = new.clone();
        let v = query_override_for_sel("#t", "width", &base, None).expect("override");
        let px = v.trim_end_matches("px").parse::<f64>().expect("px");
        assert!((px - 80.0).abs() < 0.5, "eased(0.5)=at=0.3 → 50+100*0.3=80px，got {v}");
        clear_all();
    }

    /// 非插值对不登记（css-transitions §starting-of-transitions）：关键字端点
    /// `auto→100px` 宽松解析会 0 数值化产伪中值——严格门拒登记，咨询回落 None。
    #[test]
    fn test_non_interpolable_pair_not_registered() {
        clear_all();
        let html = r#"<html><body><div id="t"></div></body></html>"#;
        let doc = parse_html(html);
        let nid = crate::find_by_selector(&doc, "#t").expect("target");
        let mut prev = ComputedStyle::default();
        let mut new = ComputedStyle::default();
        prev.width = LengthValue::Auto;
        new.width = LengthValue::Px(100.0);
        new.transition_property = vec!["width".to_string()];
        new.transition_duration = vec![100.0];
        new.transition_delay = vec![-50.0];
        new.transition_timing_function = vec![TimingFunctionValue::Linear];
        register_from_style_change(World::Gen, "#t", &prev, &new, &doc, nid);
        let base = new.clone();
        assert!(
            query_override_for_sel("#t", "width", &base, None).is_none(),
            "非插值对（auto 端点）不登记不覆盖"
        );
        clear_all();
    }

    /// 同 (world, identity, property) 后注册替换先注册（css-transitions 同属性
    /// 新过渡替换旧过渡）；异属性共存。
    #[test]
    fn test_replace_and_coexist() {
        clear_all();
        let html = r#"<html><body><div id="t"></div></body></html>"#;
        let doc = parse_html(html);
        let nid = crate::find_by_selector(&doc, "#t").expect("target");
        let (prev, new) = make_pair(0.0, 100.0, TimingFunctionValue::Linear, -50.0);
        register_from_style_change(World::Gen, "#t", &prev, &new, &doc, nid);
        let (prev2, new2) = make_pair(200.0, 300.0, TimingFunctionValue::Linear, -50.0);
        register_from_style_change(World::Gen, "#t", &prev2, &new2, &doc, nid);
        let base = new2.clone();
        let v = query_override_for_sel("#t", "width", &base, None).expect("latest wins");
        assert_eq!(v, "250px", "后注册覆盖先注册，got {v}");
        clear_all();
    }

    /// 世界域隔离：Gen 登记对 View 通道查询不可见（两域身份/视图独立）。
    #[test]
    fn test_world_isolation() {
        clear_all();
        let html = r#"<html><body><div id="t"></div></body></html>"#;
        let doc = parse_html(html);
        let nid = crate::find_by_selector(&doc, "#t").expect("target");
        let (prev, new) = make_pair(0.0, 100.0, TimingFunctionValue::Linear, -50.0);
        register_from_style_change(World::Gen, "#t", &prev, &new, &doc, nid);
        let base = new.clone();
        assert!(
            query_override_for_handle("__n0", "width", &base, None).is_none(),
            "Gen 条目不进 View 通道"
        );
        clear_all();
    }

    /// eased==1.0（at=1 → steps(1,start) 越点）回落 echo：过渡值恒等 to == 级联
    /// 值，咨询返 None（v3 端点序列化形差 REG/DOWNS 类的回归钉）。
    #[test]
    fn test_eased_one_falls_back_to_echo() {
        clear_all();
        let html = r#"<html><body><div id="t"></div></body></html>"#;
        let doc = parse_html(html);
        let nid = crate::find_by_selector(&doc, "#t").expect("target");
        let (prev, new) = make_pair(
            50.0,
            150.0,
            TimingFunctionValue::Steps(1, Some(StepPosition::Start)),
            -50.0,
        );
        register_from_style_change(World::Gen, "#t", &prev, &new, &doc, nid);
        let base = new.clone();
        assert!(
            query_override_for_sel("#t", "width", &base, None).is_none(),
            "eased==1.0 恒等 to，回落 echo 不覆盖"
        );
        clear_all();
    }

    /// filter 量钳制（css-filters-1 量值界）：[0,1] 族外推量钳回界内，无界族
    /// （blur/hue-rotate/brightness）不钳。
    #[test]
    fn test_filter_amount_clamp() {
        let raw = "sepia(1.5) grayscale(2) invert(-0.5) opacity(0.75) blur(4px) hue-rotate(90deg) brightness(1.5)";
        assert_eq!(
            clamp_filter_unit_interval_amounts(raw),
            "sepia(1) grayscale(1) invert(0) opacity(0.75) blur(4px) hue-rotate(90deg) brightness(1.5)",
            "[0,1] 族钳制、无界族原样"
        );
        assert_eq!(clamp_filter_unit_interval_amounts("invert()"), "invert()", "空量形原样");
    }

    /// markers 存活（A/B provenance 抽验锚点）。
    #[test]
    fn test_marker_present() {
        assert!(!TRANSITION_SYNC_READ_MARKER.is_empty());
    }

    /// `all` 通配展开（css-transitions §2）：`transition-property: all` 下值变化
    /// 的 longhand 各自登记——width 10→40 linear 中点 25px；未变化属性无条目
    /// （echo 池零扰动）。
    #[test]
    fn test_all_wildcard_registers_changed_longhand() {
        clear_all();
        let html = r#"<html><body><div id="t"></div></body></html>"#;
        let doc = parse_html(html);
        let nid = crate::find_by_selector(&doc, "#t").expect("target");
        let mut prev = ComputedStyle::default();
        let mut new = ComputedStyle::default();
        prev.width = LengthValue::Px(10.0);
        new.width = LengthValue::Px(40.0);
        new.transition_property = vec!["all".to_string()];
        new.transition_duration = vec![100.0];
        new.transition_delay = vec![-50.0];
        new.transition_timing_function = vec![TimingFunctionValue::Linear];
        register_from_style_change(World::Gen, "#t", &prev, &new, &doc, nid);
        let base = new.clone();
        let v = query_override_for_sel("#t", "width", &base, None).expect("all 通配登记 width");
        assert_eq!(v, "25px", "all 通配 linear 中点，got {v}");
        assert!(
            query_override_for_sel("#t", "opacity", &base, None).is_none(),
            "端点同值属性无条目"
        );
        clear_all();
    }

    /// eased==1.0 rule-\* 族例外（css-gaps-1 列表端点广播/展开语义）：to 单值对
    /// from 多值位输出广播形（`repeat(2, 20px)`→`30px` at(1) = `30px, 30px`），
    /// 不落 echo 短路（语料 rule-width-interpolation-repeaters 页钉）。
    #[test]
    fn test_eased_one_rule_broadcast() {
        clear_all();
        let html = r#"<html><body><div id="t"></div></body></html>"#;
        let doc = parse_html(html);
        let nid = crate::find_by_selector(&doc, "#t").expect("target");
        let mut prev = ComputedStyle::default();
        let mut new = ComputedStyle::default();
        prev.column_rule_width = zero_css_parser::values::parse_rule_width_list("repeat(2, 20px)").expect("from");
        new.column_rule_width = zero_css_parser::values::parse_rule_width_list("30px").expect("to");
        new.transition_property = vec!["column-rule-width".to_string()];
        new.transition_duration = vec![100.0];
        new.transition_delay = vec![-50.0];
        new.transition_timing_function = vec![TimingFunctionValue::Steps(1, Some(StepPosition::Start))];
        register_from_style_change(World::Gen, "#t", &prev, &new, &doc, nid);
        let base = new.clone();
        let v = query_override_for_sel("#t", "column-rule-width", &base, None).expect("广播覆盖");
        assert_eq!(v, "30px, 30px", "at(1) 广播位输出，got {v}");
        clear_all();
    }

    /// eased==1.0 rule-inset 槽例外（刀 E v4）：跨单位端点 at(1) 须出 calc 混合
    /// 终形（css-values-4 §8.5.6 calc-mix）——`-100%`→`1px` at(1) =
    /// `calc(0% + 1px)`，不落 echo 短路（语料 rule-inset-interpolation 页钉：
    /// echo 级联形 `1px` 欠混合形）。
    #[test]
    fn test_eased_one_rule_inset_calc_mix() {
        clear_all();
        let html = r#"<html><body><div id="t"></div></body></html>"#;
        let doc = parse_html(html);
        let nid = crate::find_by_selector(&doc, "#t").expect("target");
        let mut prev = ComputedStyle::default();
        let mut new = ComputedStyle::default();
        prev.column_rule_inset_cap_start =
            zero_css_parser::values::RuleInsetComponent::Length(LengthValue::Percentage(-100.0));
        new.column_rule_inset_cap_start = zero_css_parser::values::RuleInsetComponent::Length(LengthValue::Px(1.0));
        new.transition_property = vec!["column-rule-inset-cap-start".to_string()];
        new.transition_duration = vec![100.0];
        new.transition_delay = vec![-50.0];
        new.transition_timing_function = vec![TimingFunctionValue::Steps(1, Some(StepPosition::Start))];
        register_from_style_change(World::Gen, "#t", &prev, &new, &doc, nid);
        let base = new.clone();
        let v = query_override_for_sel("#t", "column-rule-inset-cap-start", &base, None).expect("calc 混合覆盖");
        assert_eq!(v, "calc(0% + 1px)", "at(1) 跨单位混合终形，got {v}");
        clear_all();
    }
}
