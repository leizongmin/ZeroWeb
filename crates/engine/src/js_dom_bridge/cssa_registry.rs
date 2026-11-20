//! CSS Animations 声明式 @keyframes 的 gCS 同步读咨询注册表（wptfyi-css 刀 E，
//! 2026-11-17）。
//!
//! **断点**：渲染管线 `AnimationClock` 的插值只进渲染层 styles 副本
//!（`apply_animation_overrides` 泵帧路径），且 gCS 桥层重建 computed 时独立级联。
//! corpus CSSA 测试（interpolation-testcommon.js `CSS Animations` 方法）在**同一
//! 脚本轮次**内追加 `@keyframes` 文本 + 写 `animation-name/duration/delay/-timing-
//! function` → gCS 断言读——泵帧不介入，声明式动画对 gCS 零效果（8 个 rule-inset
//! interpolation 槽 CSSA 通道 312F 全挂、got 恒类底层值的根因）。
//!
//! **解法**（transition_registry 十轮刀已证模式镜像）：gCS 两通道（selector/
//! handle）读点维护 **per-identity 动画状态**（thread_local `AnimationClock`）。
//! keyframes 经 [`super::computed_style_cache`] 代际重建喂入（`@keyframes` 文本在
//! stylesheet 解析产物内）；元素动画配置从 ComputedStyle `animation-*` 字段读取，
//! **首咨询锚 start**（`current_time = now`——harness `duration=100s, delay=-50s`
//! 钉死 progress≈0.5；同步轮次内 start→read 间隔微秒级，eased(0.5)=at 精确），
//! tick 产出经 [`super::js_anim_registry::canonicalize`] 施于基样式 clone 过 gCS
//! 序列化器。渲染管线 `AnimationClock`/事件派发零触及（独立 thread_local 实例，
//! 导航换代清表——transition_registry 同约束）。
//!
//! **零干扰**：`animation-name` 空的元素 O(1) None；keyframes 未注册（动画名不在
//! 喂入面）None 回落级联序列化。CSSA 语料外的声明式动画（真站 @keyframes 渲染）
//! 走管线泵帧路径不变——本注册表只在 gCS 读点**叠加**咨询，返回 None 时行为与
//! 刀前逐位一致。
//!
//! spec：css-animations-1 §animations（active interval/delay 负值 seek）、
//! css-values-4 §8.5.6（离散插值）、css-easing-4（timing 求值）。
//! https://drafts.csswg.org/css-animations-1/

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Instant;

use zero_style_system::ComputedStyle;

use super::js_anim_registry::canonicalize;
use super::transition_registry::World;
use crate::animation::{AnimationClock, AnimationConfig};
use zero_css_parser::values::{AnimationCompositionValue, AnimationDirectionValue, AnimationFillModeValue};

/// 进程起点（单调墙钟锚——running 动画求值时基；corpus 同轮次 start→read elapsed≈0）。
static MONOTONIC_START: std::sync::LazyLock<Instant> = std::sync::LazyLock::new(Instant::now);

#[derive(Debug, Clone)]
struct CssaEntry {
    /// 当前动画名（style.animation_name 逐名对位；名变即换代重锚）。
    anim_name: String,
    /// AnimationClock 内的动画键（per-entry 独立 clock_id，互不串扰）。
    clock_id: u64,
}

thread_local! {
    /// 声明式动画时钟（keyframes registry + active animations；喂入面唯一写者）。
    static CLOCK: RefCell<AnimationClock> = RefCell::new(AnimationClock::new());

    /// per-(world, identity) 当前动画条目——同名动画只锚一次（多次读不重锚，
    /// elapsed 自然推进）；动画名变更（harness 逐 at 换名）即换代重锚。
    static ENTRIES: RefCell<HashMap<(World, String), CssaEntry>> = RefCell::new(HashMap::new());

    static NEXT_CLOCK_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(1) };
}

/// 页面换代清表（`clear_generation_cache` 同点调用——导航/新 WebView 装新快照，
/// 旧页声明式动画不跨文档泄漏；与 js_anim/transition registry 同约束）。
pub fn clear_all() {
    CLOCK.with(|cell| *cell.borrow_mut() = AnimationClock::new());
    ENTRIES.with(|cell| cell.borrow_mut().clear());
}

/// stylesheet 产物喂入（computed_style_cache 代际重建两分支调用——
/// `compute_document_styles_with_inline_overrides_css` 产出的 sheets 即当前代际
/// 权威 stylesheet 集）。`register_from_stylesheets` 按规则名 insert-or-replace，
/// 同名重喂幂等；默认 MediaContext（corpus 面无条件包裹 @keyframes，条件块支持
/// 由管线 R3881 面承担——FIXME(cssa-conditional)：gCS 咨询面 @media 内 keyframes
/// 未跟管线条件评估对齐）。
pub fn note_stylesheets(stylesheets: &[zero_css_parser::Stylesheet]) {
    let media_ctx = zero_css_parser::media_query::MediaContext::new(1280.0, 720.0);
    CLOCK.with(|cell| {
        cell.borrow_mut().register_from_stylesheets(stylesheets, &media_ctx);
    });
    // keyframes 内容换代失效条目：活跃 AnimationState 持 start 时克隆的旧帧表，
    // 同名换内容（harness 每 at 重注入 @keyframes）不换名——ENTRIES 不清则永远
    // 跑旧帧。清条目强制下次咨询重锚（新内容新 start；corpus 面重锚即本轮，
    // elapsed≈0 progress 恒 0.5 不漂移）。
    ENTRIES.with(|cell| cell.borrow_mut().clear());
}

/// selector 通道咨询（callbacks.rs gCS per-selector 命中/重算两路径）。
/// `parent`：父元素 computed（刀 K1——关键帧 'inherit' 端点解析源，
/// css-cascade-4 §inheritance；None 回落 initial 同式解析）。
pub(crate) fn query_override_for_sel(
    sel: &str,
    prop: &str,
    style: &ComputedStyle,
    parent: Option<&ComputedStyle>,
) -> Option<String> {
    query_core(World::Gen, sel, prop, style, parent)
}

/// handle 通道咨询（computed_style_cache.rs 动态元素 gCS 路径）。
pub(crate) fn query_override_for_handle(
    handle: &str,
    prop: &str,
    style: &ComputedStyle,
    parent: Option<&ComputedStyle>,
) -> Option<String> {
    query_core(World::View, handle, prop, style, parent)
}

/// 咨询核：元素有 animation-name 时逐名保证动画已锚（首咨询 start、elapsed 自
/// 锚点推进），tick 产出匹配属性 → canonicalize（apply + gCS 串化往返，applier
/// 未覆盖/恒等 None 回落）。keyframes 未注册（`start_animation_with_underlying`
/// 返 false）→ 无条目无产出。
fn query_core(
    world: World,
    identity: &str,
    prop: &str,
    style: &ComputedStyle,
    parent: Option<&ComputedStyle>,
) -> Option<String> {
    if style.animation_name.is_empty() {
        return None;
    }
    let now = MONOTONIC_START.elapsed().as_secs_f64();
    // 动画名列表首名为准（harness 单名形态；多名列表现实罕见，逐名对位见
    // pipeline start_from_computed_style——v1 取首名，FIXME(cssa-multi-name)）。
    let name = style
        .animation_name
        .iter()
        .find(|n| !n.is_empty() && n.as_str() != "none")?
        .clone();

    CLOCK.with(|clock_cell| {
        ENTRIES.with(|entries| {
            let mut clock = clock_cell.borrow_mut();
            let mut entries = entries.borrow_mut();
            let key = (world, identity.to_string());
            let need_start = match entries.get(&key) {
                Some(e) => e.anim_name != name,
                None => true,
            };
            if need_start {
                let clock_id = NEXT_CLOCK_ID.with(|cell| {
                    let v = cell.get();
                    cell.set(v + 1);
                    v
                });
                // 配置从 ComputedStyle animation-* 字段对位读取（管线
                // start_from_computed_style 同映射）。current_time = now 锚 start：
                // active_elapsed = (now - now) - delay = -delay，harness -50s 钉
                // progress = 50/100 = 0.5（css-animations-1 §animations 负 delay）。
                let config = AnimationConfig {
                    name: name.clone(),
                    duration: style.animation_duration.first().copied().unwrap_or(0.0),
                    delay: style.animation_delay.first().copied().unwrap_or(0.0),
                    timing_function: style
                        .animation_timing_function
                        .first()
                        .cloned()
                        .unwrap_or(zero_css_parser::values::TimingFunctionValue::Ease),
                    iteration_count: style.animation_iteration_count.first().and_then(|v| *v),
                    direction: style
                        .animation_direction
                        .first()
                        .cloned()
                        .unwrap_or(AnimationDirectionValue::Normal),
                    fill_mode: style
                        .animation_fill_mode
                        .first()
                        .cloned()
                        .unwrap_or(AnimationFillModeValue::None),
                    current_time: now,
                    // 刀 ANIMCOMP S2：动画级合成操作（css-animations-2 §animation-composition；
                    // 单动画面 first 对位，空列表回落 replace）
                    composition: style
                        .animation_composition
                        .first()
                        .cloned()
                        .unwrap_or(AnimationCompositionValue::Replace),
                    underlying_values: std::collections::HashMap::new(),
                };
                // 刀 K1：parent 沿 gCS 读点（doc+styles 在场）传入——'inherit'
                // 端点解析为父 computed（interpolation corpus CSS Animations
                // 半面 50F 主靶）；None 回落 initial（E5 同式）。
                if !clock.start_animation_with_underlying(clock_id, &config, Some(style), parent) {
                    return None; // keyframes 未注册（喂入面无该名）
                }
                entries.insert(
                    key.clone(),
                    CssaEntry {
                        anim_name: name,
                        clock_id,
                    },
                );
            }
            let entry = entries.get(&key)?;
            let clock_id = entry.clock_id;
            for p in clock.tick(clock_id, now) {
                if p.name != prop {
                    continue;
                }
                // canonicalize：apply + gCS 串化往返（js_anim 共享——applier 未覆盖
                // 或施前后串化恒等返 None 回落级联，零回归方向）。
                if let Some(v) = canonicalize(prop, &p.value, style, None) {
                    return Some(v);
                }
            }
            None
        })
    })
}
