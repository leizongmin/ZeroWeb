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

use zero_css_parser::values::{
    AnimationCompositionValue, AnimationDirectionValue, AnimationFillModeValue, TimingFunctionValue,
};
use zero_style_system::ComputedStyle;

use super::computed_style::serialize_computed_property_with_sibling;
use crate::animation::{AnimationClock, AnimationConfig, InterpolatedProperty, apply_timing_function};

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
    /// 全局 easing（合成规则帧不携带 animation-timing-function——`_zwWaapiSpecKeyframes`
    /// 丢帧级 easing 键，引擎段内恒用全局 timing， eased 重算同源）。
    timing: TimingFunctionValue,
    /// 离散翻转采纳形态门：恰 2 帧、端点 offset 0/1（选择器域）、direction
    /// normal——eased 一致性重算仅在整体进度 ≡ 段归一化进度（span 1）且方向
    /// 不反演时精确。
    flip_ok: bool,
    /// 离散翻转端点对（prop → (from, to)，按帧选择器 0%/100% 归侧而非数组序）。
    flip_pairs: std::collections::HashMap<String, (String, String)>,
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
    // 条目侧留存（AnimationConfig 构造 move 走原值——登记闭包借用序先 clone）。
    let entry_timing = timing_function.clone();
    // iterations：null（JS Infinity）/<=0 → None（infinite）；其余 Some(n)。
    let iteration_count = spec
        .get("iter")
        .and_then(json_f64)
        .filter(|n| *n > 0.0 && n.is_finite());
    let direction = parse_direction(spec.get("dir").and_then(Json::as_str));
    let fill_mode = parse_fill(spec.get("fill").and_then(Json::as_str));
    // 刀 ANIMCOMP S2：合成操作 + shim 预读 underlying（css-animations-2
    // §animation-composition / web-animations-1 §5.4.4）。shim 均匀 add 影族走
    // 引擎合成通道：`comp` = 动画级 op（add/accumulate），`und` = 属性 → gCS
    // computed 串映射；缺省/畸形回落 replace（刀前行为恒等）。
    let composition = match spec.get("comp").and_then(Json::as_str) {
        Some("add") => AnimationCompositionValue::Add,
        Some("accumulate") => AnimationCompositionValue::Accumulate,
        _ => AnimationCompositionValue::Replace,
    };
    let mut underlying_values = std::collections::HashMap::new();
    if let Some(und) = spec.get("und").and_then(Json::as_object) {
        for (k, v) in und {
            if let Some(s) = v.as_str()
                && !s.is_empty()
            {
                underlying_values.insert(k.clone(), s.to_string());
            }
        }
    }

    let clock_id = id;
    // easing 感知离散翻转采纳（css-values-4 §8.5.6 / Web Animations §5.6.2）：
    // 恰 2 帧 [0,1] 单向动画的整体进度即段归一化进度，eased 位置可精确重算；
    // 多帧（段选址扭曲整体进度）与 reverse 族（方向反演）形态不进门。
    let kf_offset = |b: &zero_css_parser::ast::KeyframeBlock| -> Option<f64> {
        match b.selectors.first() {
            Some(zero_css_parser::ast::KeyframeSelector::Percentage(p)) => Some(*p / 100.0),
            Some(zero_css_parser::ast::KeyframeSelector::From) => Some(0.0),
            Some(zero_css_parser::ast::KeyframeSelector::To) => Some(1.0),
            _ => None,
        }
    };
    let mut flip_ok = rule.keyframes.len() == 2 && direction == AnimationDirectionValue::Normal;
    let mut flip_pairs: std::collections::HashMap<String, (String, String)> = std::collections::HashMap::new();
    // 端点偏移门按集合成员判定（kc5r86 刀面修正）：shim neutral 材料化把预读
    // underlying 帧追加在 to 帧之后＝数组序反置（css-animations-1 #keyframes
    // 缺帧=underlying 的单帧输入物化成 2 实块），位置式检查恒拒＝neutral 族
    // flip 通道全灭、at==endTime 端点输出被 fallback 误杀。flip_pairs 归侧本就
    // 不信任数组序，此门与其对齐；{0,1} 双成员也排除 0/0 同侧退化。
    if flip_ok {
        let offs: Vec<f64> = rule.keyframes.iter().filter_map(&kf_offset).collect();
        flip_ok = offs.contains(&0.0) && offs.contains(&1.0);
    }
    if flip_ok {
        // 端点对按选择器归侧（数组序不信任——kc5r86 起真按选择器：neutral 材料
        // 化帧序反置，首帧可能持 100% 块）：0% → from、100% → to，同帧重复声明
        // 首值胜（与 endpoints 去重首值胜一致）。
        let mut fside: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let mut tside: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        for b in &rule.keyframes {
            let side = match kf_offset(b) {
                Some(0.0) => &mut fside,
                Some(1.0) => &mut tside,
                _ => continue,
            };
            for d in &b.declarations {
                if !d.property.eq_ignore_ascii_case("animation-timing-function") {
                    side.entry(d.property.clone()).or_insert_with(|| d.value.clone());
                }
            }
        }
        for (k, fv) in fside {
            if let Some(tv) = tside.get(&k) {
                flip_pairs.insert(k, (fv, tv.clone()));
            }
        }
    }
    // 登记面诊断（ZW_BOUNDARY_DEBUG 门， kc5r86）：块序/归侧/flip 门形态直读——
    // neutral 材料化反置序先例在案（PROBE FAIL→dump 定谳）。
    if std::env::var("ZW_BOUNDARY_DEBUG").is_ok() {
        let dump: Vec<String> = rule
            .keyframes
            .iter()
            .map(|b| {
                format!(
                    "[{:?}:{}{}]",
                    b.selectors,
                    b.declarations
                        .iter()
                        .map(|d| format!("{}={}", d.property, d.value))
                        .collect::<Vec<_>>()
                        .join(","),
                    if b.declarations.is_empty() { "EMPTY" } else { "" }
                )
            })
            .collect();
        eprintln!(
            "ZW-BOUNDARY-REG id={id} blocks={} flip_ok={flip_ok} dir={direction:?} pairs={} dump={}",
            rule.keyframes.len(),
            flip_pairs.len(),
            dump.join(" ")
        );
    }
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
                composition,
                underlying_values,
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
            timing: entry_timing,
            flip_ok,
            flip_pairs,
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

/// KC-5-r72 pair 端点对咨询（handle 通道，2026-11-24）：命中 width/height 动画
/// 的开区间可插值端点对 `(from, to, eased)`——eased 与 tick 段内映射同源（全局
/// timing/段 timing、[`apply_timing_function`]），求值时间 t-math 与
/// [`query_core`] 恒等（`set_hold`/`set_start_time` 幂等，后续单值链 tick 净态
/// 不变）。端点对自 clock 注册态纯读（[`AnimationClock::peek_sizing_interp_pair`]）。
///
/// 折叠需求门（r72 领域收窄）：两端皆 LP 数值形（px/%/calc 家族）→ None——
/// 既有 tick+roundtrip 通道已正确，经布局折叠反引入 border-box 膨胀回归面；
/// 裸关键字↔裸关键字对 → None——spec 离散交既有 flip 通道（css-values-4
/// §8.5.6）。r71 证据域（got=静态回声）= 关键字涉入混形对，唯此域入 fold。
pub fn query_sizing_interp_pair_for_handle(hnd: &str, prop: &str) -> Option<(String, String, f64)> {
    use super::computed_style_cache::{is_bare_sizing_keyword, is_plain_lp_form};
    if !ENTRIES.with(|cell| !cell.borrow().is_empty()) {
        return None;
    }
    let now = MONOTONIC_START.elapsed().as_secs_f64();
    let mut out: Option<(String, String, f64)> = None;
    ENTRIES.with(|entries| {
        CLOCK.with(|clock_cell| {
            let mut clock = clock_cell.borrow_mut();
            for e in entries.borrow().iter() {
                if e.hnd.as_deref() != Some(hnd) {
                    continue;
                }
                // 求值时间 t-math 同 query_core（hold 冻结 / paused 零点 / running 墙钟）。
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
                let prog = if e.dur_s > 0.0 { (t - e.delay_s) / e.dur_s } else { 1.0 };
                if !(prog > 0.0 && prog < 1.0) {
                    continue;
                }
                let Some((from, to, eased)) = clock.peek_sizing_interp_pair(e.clock_id, prop, t) else {
                    continue;
                };
                // 后值覆盖先值（注册序，query_core latest 同序）。
                if is_plain_lp_form(&from) && is_plain_lp_form(&to) {
                    continue;
                }
                if is_bare_sizing_keyword(&from) && is_bare_sizing_keyword(&to) {
                    continue;
                }
                out = Some((from, to, eased));
            }
        });
    });
    out
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
                    let endpoint_hit = e
                        .endpoints
                        .get(&p.name)
                        .is_some_and(|vals| vals.iter().any(|v| v == &p.value));
                    // easing 感知离散翻转采纳（wptfyi-css easing 刀 2026-11-15）：
                    // 2 端点 [0,1] 单向动画的开区间端点恒等输出是 spec 离散/步进
                    // 行为（css-values-4 §8.5.6：eased < 0.5 → from，≥ 0.5 → to；
                    // step-start/end eased 钳 0/1 同形），eased 位置与端点侧一致
                    // 则采纳——v2 全折检测器把该面 ~50 真增益一并回落（十轮账）。
                    // eased 与引擎段内映射同源（全局 timing、span 1 归一化恒等），
                    // 不一致（spec 可插值而引擎退化为 flip 的 echo，如 calc/track
                    // 列表）维持回落。NaN eased 双臂皆 false → 保持回落零回归方向。
                    let flip_adopted = endpoint_hit
                        && open_interval
                        && e.flip_ok
                        && e.flip_pairs
                            .get(&p.name)
                            .is_some_and(|(f, tv)| flip_eased_adopts(f, tv, &p.value, prog, &e.timing));
                    // 单块精确终点落地采纳（kc5r86，css-animations-1 #keyframes
                    // 缺帧=underlying）：单块规则 eased==1.0 时 tick 输出＝末端帧
                    // 精确 to 值，对 interpolable（进度 1 恰 to）与 discrete
                    // （css-values-4 §8.5.6 ≥0.5→to）皆 spec 正确——不再作离散
                    // echo 回落。eased<1.0 中程行零触及（插值中值非端点，
                    // endpoint_hit 本就 false）；多块规则（2026-11-15 REGRESSED=245
                    // fallback 防线面）不入本门。steps(1, start) 在 prog=0.5 处
                    // 精确 1.0（apply_timing_function 单测钉值），浮点容差 1e-6。
                    // 求值面诊断（ZW_BOUNDARY_DEBUG 门，kc5r86）：端点命中/开区间/
                    // eased 形直读——at==endTime 误杀先例在案（got=underlying 回声）。
                    if std::env::var("ZW_BOUNDARY_DEBUG").is_ok() {
                        eprintln!(
                            "ZW-BOUNDARY-Q prop={} raw={:?} ehit={endpoint_hit} ointerval={open_interval} flipok={} eased={:?} prog={prog}",
                            p.name,
                            p.value,
                            e.flip_ok,
                            apply_timing_function(prog, &e.timing),
                        );
                    }
                    let fallback = endpoint_hit && open_interval && !flip_adopted;
                    if !fallback {
                        latest = Some(p.value);
                    }
                }
            }
        });
    });
    latest.and_then(|raw| canonicalize(prop, &raw, base, sibling))
}

/// 离散翻转 eased 一致性判定：`flip_ok` 条目的开区间端点恒等输出在 eased 位置
/// 与端点侧一致时采纳（`< 0.5` 归 from，`≥ 0.5` 归 to——css-values-4 §8.5.6）。
/// from ≡ to 恒值动画两侧一致恒采纳。非 from 非 to 的值拒（防御：端点对与
/// 命中值失配的形态保持回落）。NaN eased（退化 bezier 参数）双臂皆 false →
/// 拒（保持回落零回归方向）。
fn flip_eased_adopts(from: &str, to: &str, value: &str, prog: f64, timing: &TimingFunctionValue) -> bool {
    if value == to {
        if to == from {
            // 恒值动画：任何进度期望值恒同（spec 常值），无条件采纳。
            return true;
        }
        let eased = apply_timing_function(prog, timing);
        return eased >= 0.5;
    }
    let eased = apply_timing_function(prog, timing);
    value == from && eased < 0.5
}

/// A/B 快照二进制 provenance 标记（strings 抽验 B0/B1 装载面——shim JS 未动轮
/// 的源侧指纹，shim-include_str 陈旧二进制事故防线；测试引用保存活零运行时成本）。
#[allow(dead_code)]
pub(crate) static FLIP_EASED_ADOPT_MARKER: &str = "zw-easing-flip-adopt-2026-11-15";

/// KC-5-r5 K-fold face provenance 标记（A/B strings 抽验 b1 装载面；同上防线，
/// 测试引用保存活零运行时成本）。
#[allow(dead_code)]
pub(crate) static KFOLD_R5_MARKER: &str = "zw-kfold-r5-consult-2026-11-23";

/// kc5r86 flip 端点偏移集合成员门＋选择器归侧 face provenance 标记（`#[used]`
/// static [u8;N] 防 release DCE——r82 先例：未引用 `&str`/pub const 被链接期剥
/// 离；A/B strings 抽验 b1 装载面，测试引用保存活零运行时成本）。
#[used]
#[allow(dead_code)]
static FLIP_ORDERLESS_R86_MARKER: [u8; 28] = *b"zw-flip-orderless-2026-11-24";

/// 插值串规范化：施加到基样式 clone 后过 gCS 序列化器——与期望元素（测试侧
/// inline 设值读 gCS）同通道，格式一致由构造保证（"0.5000" → "0.5"）。序列化
/// 无变化两种形态都返 None 回落既有 gCS 通道（零回归方向）：
/// ① applier 未覆盖该属性（引擎不支持的属性，期望元素 gCS 为空串——B1 实测
///    font-variant-ligatures 期望 "" 对 raw 透传裸串 = REGRESSED 伪源）；
/// ② 施加后序列化恒等（动画值 ≡ 基值，回落同一序列化结果语义不变）。
/// A/B 定谳 2026-11-15：raw 透传臂在端点钳制形态（p≥1）撞空串期望。
pub(super) fn canonicalize(prop: &str, raw: &str, base: &ComputedStyle, sibling: Option<(u64, u64)>) -> Option<String> {
    // KC-5-r5 K-fold（wptfyi-css，2026-11-23）：sizing 关键字端点透传——width/
    // height 套 apply+串化往返产错值串（Stretch 槽串化塌 "0px"、bare fit-content
    // 落 MaxContent 槽错映射），而 expected 侧为参考元素静态 used-value 折叠 px
    //（CSSOM-1 §resolved value width/height 特例）→ 字面/错值串恒 F（r3 interp
    // 族 327 P2F 根因）。透传关键字原串交 gCS 读点假想值折叠
    //（[`super::computed_style_cache::fold_consult_for_selector`]），折叠失配
    // 回落 echo 静态通道。min/max 族不进本门：resolved value = computed 字面，
    // apply+串化既有行为正确（r3/r4 双侧零红）。
    // https://drafts.csswg.org/css-sizing-3/#width-resolution
    if matches!(prop, "width" | "height") && super::computed_style_cache::is_foldable_sizing_keyword(raw) {
        return Some(raw.trim().to_ascii_lowercase());
    }
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
    let dbg = std::env::var("ZW_ANIM_VALIDATION_DEBUG").is_ok();
    if dbg {
        eprintln!("ZW-ANIM-BUILD-ENTRY name={name} frames={}", frames.len());
    }
    let mut css = format!("@keyframes {name}{{");
    for frame in frames {
        let offset = frame.get("o").and_then(json_f64).unwrap_or(0.0);
        let props = frame.get("p").and_then(Json::as_object);
        let Some(props) = props else { continue };
        css.push_str(&format!("{}%{{", trim_float(offset * 100.0)));
        // 刀 WAC（wptfyi-css，2026-11-20）：帧级 composite（shim 引擎合成模式保形
        // 透传，mask-border 混合 op 面）→ `animation-composition:` 帧内声明
        //（css-animations-2 §animation-composition；@keyframes 文本通道与 CSSA
        // 同法入 point.composition，引擎 §5.4.4 混合 op 端点预合成臂消费）。
        // 白名单外值不落（帧回落动画级 op，js_anim 面恒 replace 语义）。
        if let Some(c) = frame.get("c").and_then(Json::as_str)
            && matches!(c, "replace" | "add" | "accumulate")
        {
            css.push_str(&format!("animation-composition:{c};"));
        }
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
            // WAAPI keyframes 值校验（2026-11-17 B3 回归修复）：非法值**丢弃该属性**
            // （键塌缩 → 端点回落 underlying——Web Animations §5.3.11 keyframe
            // 转换对 invalid property value 的丢弃语义，Chrome parity）。合成文本
            // 通道此前零校验，声明文法宽松受理残缺形（括号未闭合照收）→ 端点值
            // 落盘放大为 B2→B3 真回归（repeaters-001 WAAPI at(1.5)）。与 CSSOM
            // setProperty 同一校验器（`validate_decl_value`），合法值零变化。
            // https://drafts.csswg.org/web-animations-1/#processing-a-keyframes-argument
            if !zero_style_system::validate_decl_value(k, &value) {
                if dbg {
                    eprintln!("ZW-ANIM-VALIDATION-DROP prop={k} value={value}");
                }
                continue;
            }
            if dbg && k.contains("rule") {
                eprintln!("ZW-ANIM-VALIDATION-PASS prop={k} value={value}");
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

#[cfg(test)]
mod tests {
    use super::FLIP_EASED_ADOPT_MARKER;
    use super::build_keyframes_rule;
    use super::flip_eased_adopts;
    use serde_json::Value as Json;
    use zero_css_parser::values::TimingFunctionValue;

    #[test]
    fn test_marker_present() {
        assert!(!FLIP_EASED_ADOPT_MARKER.is_empty());
    }

    /// linear：翻转点 eased = prog——前半 from 侧采纳 / 后半 to 侧采纳，异侧拒。
    #[test]
    fn test_linear_flip_sides() {
        assert!(flip_eased_adopts(
            "solid",
            "dashed",
            "solid",
            0.25,
            &TimingFunctionValue::Linear
        ));
        assert!(!flip_eased_adopts(
            "solid",
            "dashed",
            "dashed",
            0.25,
            &TimingFunctionValue::Linear
        ));
        assert!(flip_eased_adopts(
            "solid",
            "dashed",
            "dashed",
            0.75,
            &TimingFunctionValue::Linear
        ));
        assert!(!flip_eased_adopts(
            "solid",
            "dashed",
            "solid",
            0.75,
            &TimingFunctionValue::Linear
        ));
    }

    /// 翻转边界 eased = 0.5 归 to 侧（引擎 `t >= 0.5` 臂与 css-values-4 §8.5.6 同界）。
    #[test]
    fn test_boundary_half_to_side() {
        assert!(!flip_eased_adopts("a", "b", "a", 0.5, &TimingFunctionValue::Linear));
        assert!(flip_eased_adopts("a", "b", "b", 0.5, &TimingFunctionValue::Linear));
    }

    /// step-end：开区间 eased 钳 0 → 只认 from；step-start 钳 1 → 只认 to。
    #[test]
    fn test_steps_clamp() {
        assert!(flip_eased_adopts(
            "0px",
            "10px",
            "0px",
            0.4,
            &TimingFunctionValue::StepEnd
        ));
        assert!(!flip_eased_adopts(
            "0px",
            "10px",
            "10px",
            0.4,
            &TimingFunctionValue::StepEnd
        ));
        assert!(flip_eased_adopts(
            "0px",
            "10px",
            "10px",
            0.4,
            &TimingFunctionValue::StepStart
        ));
        assert!(!flip_eased_adopts(
            "0px",
            "10px",
            "0px",
            0.4,
            &TimingFunctionValue::StepStart
        ));
    }

    /// ease-in（bezier(0.42,0,1,1)）前半段 eased < 0.5——0.4 处 from 侧仍成立。
    #[test]
    fn test_ease_in_midpoint() {
        assert!(flip_eased_adopts("x", "y", "x", 0.4, &TimingFunctionValue::EaseIn));
    }

    /// 恒值动画（from ≡ to）：任何进度期望恒同，无条件采纳。
    #[test]
    fn test_constant_pair_always_adopts() {
        assert!(flip_eased_adopts(
            "5px",
            "5px",
            "5px",
            0.1,
            &TimingFunctionValue::Linear
        ));
        assert!(flip_eased_adopts(
            "5px",
            "5px",
            "5px",
            0.9,
            &TimingFunctionValue::StepEnd
        ));
    }

    /// 非 from 非 to 的命中值拒（防御：端点对失配形态保持回落）。
    #[test]
    fn test_foreign_value_rejects() {
        assert!(!flip_eased_adopts("a", "b", "c", 0.25, &TimingFunctionValue::Linear));
        assert!(!flip_eased_adopts("a", "b", "c", 0.75, &TimingFunctionValue::Linear));
    }

    /// WAAPI keyframes 值校验（B3 回归修复钉）：非法值丢属性——键塌缩后端点
    /// 回落 underlying（Web Animations §processing-a-keyframes-argument，
    /// Chrome parity）。合法值零变化（valid 帧保形入合成文本）。
    /// 语料形：repeaters-001 WAAPI `to` 值缺右括号（页 L69 上游 typo）。
    #[test]
    fn test_build_keyframes_rejects_invalid_value() {
        // 残缺 to 值（缺右括号）：属性被丢弃 → 该帧 properties 不含 column-rule-color。
        let kf: Json = serde_json::from_str(
            r#"[{"o":0,"p":{"column-rule-color":"black"}},{"o":1,"p":{"column-rule-color":"repeat(auto, rgb(0, 0, 255), rgb(255, 0, 0)"}}]"#,
        )
        .unwrap();
        let rule = build_keyframes_rule("zwtest", Some(&kf)).expect("合法帧在 → 规则应构建");
        let to_decls = &rule.keyframes.last().expect("to 帧在").declarations;
        assert!(
            !to_decls.iter().any(|d| d.property == "column-rule-color"),
            "残缺 to 值应被丢弃，实存 {to_decls:?}"
        );
        let from_decls = &rule.keyframes.first().expect("from 帧在").declarations;
        assert!(
            from_decls.iter().any(|d| d.property == "column-rule-color"),
            "合法 from 值应保形，实存 {from_decls:?}"
        );

        // 全合法值：双帧保形（零变化面）。
        let kf: Json = serde_json::from_str(
            r#"[{"o":0,"p":{"column-rule-color":"black"}},{"o":1,"p":{"column-rule-color":"repeat(auto, rgb(0, 0, 255), rgb(255, 0, 0))"}}]"#,
        )
        .unwrap();
        let rule = build_keyframes_rule("zwtest2", Some(&kf)).expect("合法帧在 → 规则应构建");
        let to_decls = &rule.keyframes.last().expect("to 帧在").declarations;
        assert!(
            to_decls.iter().any(|d| d.property == "column-rule-color"),
            "合法 to 值应保形，实存 {to_decls:?}"
        );
    }
}
