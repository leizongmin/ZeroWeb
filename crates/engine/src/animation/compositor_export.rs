//! M2-S1b：可合成动画声明导出（renderer 静态路径 → compositor 采样域）。
//!
//! 从 `AnimationClock` 的 @keyframes 注册表与元素 underlying 样式导出可合成
//! （opacity / 2D transform）动画声明，供 renderer 经 zero-protocol 发送
//! `CompositorAnimationState`；同时产出「元素 → 已导出通道」映射，painter 据此
//! 发射 paint 动画作用域、静态路径据此跳过对已导出属性的 t=0 叠加（compositor
//! 采样在干净基线上进行，RFC §2.3）。
//!
//! 分类规则（不满足则**不导出**——动画保持现状冻结，与 flag 关一致）：
//! 1. 通道均质：每个关键帧（含补齐的 0%/100% 边界帧）都声明该属性且可解析。
//!    engine 插值按 to 侧属性集驱动（`interpolate_between_with_timing`），缺端点帧
//!    的段零输出（如 from-only opacity）；导出侧不镜像该怪癖，直接拒绝（冻结 =
//!    engine 无输出的同视觉结果）。
//! 2. 2D 可采样子集：transform 函数限 translate/rotate/scale/skew 的 px/deg/无量纲
//!    形态；`%` 分量（协议无盒参照）、matrix（engine matrix↔matrix 走 2D unmatrix
//!    分解而协议逐分量 lerp，数学不同径）、3D 函数一律拒绝。
//! 3. 端点同构：帧间函数列表等长且逐位同变体（镜像 `transform_interp` 异构离散
//!    规则——离散切换语义协议端不承载）；`none` 帧按对侧形状恒等展开（镜像
//!    `identity_list`）。
//! 4. underlying 干净门：opacity 通道要求 underlying opacity ≥ 1；transform 通道
//!    要求 underlying transform/individual transforms 均无声明——compositor 在干净
//!    基线上覆写采样值（paint 侧中性化的前提）。
//! 5. 单通道单动画：同元素同属性有多个活跃动画时全部拒绝（engine 按声明序覆盖，
//!    协议侧多声明独立采样无法复现覆盖序）。
//! 6. t=0 已结束的动画（负 delay 越过末轮）不导出——静态路径照常烘焙终帧值。
//!
//! 导出侧不镜像的引擎语义（如实记录，FR-4）：individual transform 属性动画
//!（`translate`/`rotate`/`scale` 关键帧）不导出（协议通道只承载 `transform`）；
//! 不可插值属性离散切换不导出。
//!
//! RFC：`docs/goal/compositor-animation-continuity/m2-slice-compositor-sampling.md`。

use std::collections::{HashMap, HashSet};

use slotmap::Key;
use zero_css_parser::values::{
    AnimationDirectionValue, AnimationFillModeValue, TimingFunctionValue, TransformFunction, TransformValue,
    parse_transform,
};
use zero_dom::NodeId;
use zero_style_system::{ComputedStyle, MixBlendModeComputedValue};

use super::{AnimationClock, AnimationConfig, KeyframePoint};

/// 导出通道属性（engine 域；renderer 换算 protocol `AnimationPropertyKind`——
/// engine 不反依赖 zero-protocol）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportedProperty {
    /// opacity（区间内图元 alpha 缩放）。
    Opacity,
    /// transform（恒等 TP 矩阵就地改写）。
    Transform,
}

/// 键帧端点的采样域值。
#[derive(Debug, Clone, PartialEq)]
pub enum ExportedAnimationValue {
    /// opacity（无量纲）。
    Opacity(f64),
    /// transform：2D 函数列表（已过同构/子集判定）。
    Transform(Vec<TransformFunction>),
}

/// 键帧端点（offset 升序；easing = 段起帧 timing，None 回落全局——engine
/// `KeyframePoint.timing` 同语义）。
#[derive(Debug, Clone, PartialEq)]
pub struct ExportedAnimationKeyframe {
    /// 偏移 [0,1]。
    pub offset: f64,
    /// 端点值。
    pub value: ExportedAnimationValue,
    /// 段起帧 timing。
    pub easing: Option<TimingFunctionValue>,
}

/// 一条可合成动画声明（engine 域；时间单位秒——renderer 换算协议毫秒与单调钟锚点）。
#[derive(Debug, Clone, PartialEq)]
pub struct ExportedAnimationDeclaration {
    /// renderer 域稳定 id（(element_key, name, property) FNV-1a；paint 作用域以它绑定）。
    pub animation_id: u64,
    /// 元素 key（NodeId::as_ffi，诊断用）。
    pub element_key: u64,
    /// @keyframes 名称。
    pub name: String,
    /// 单次迭代时长（秒，engine `AnimationState.duration` 已 clamp ≥ 0.001）。
    pub duration_s: f64,
    /// 启动延迟（秒）。
    pub delay_s: f64,
    /// 迭代次数；None = infinite。
    pub iterations: Option<f64>,
    /// 方向。
    pub direction: AnimationDirectionValue,
    /// 填充模式。
    pub fill: AnimationFillModeValue,
    /// 全局 timing。
    pub easing: TimingFunctionValue,
    /// engine 时钟域 start（`AnimationState.start_time`；renderer 锚点换算输入，
    /// t=0 求值域恒 0.0）。
    pub start_time_s: f64,
    /// 键帧端点列（已排序、已补界）。
    pub keyframes: Vec<ExportedAnimationKeyframe>,
}

/// 元素的一条已导出通道（painter 作用域绑定 + 静态路径跳写依据）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExportedChannel {
    /// 通道属性。
    pub property: ExportedProperty,
    /// 绑定的声明 id。
    pub animation_id: u64,
}

/// 导出全集：声明列表（按 animation_id 排序，跨帧稳定序供 renderer 代际比较）+
/// 元素通道映射。
#[derive(Debug, Clone, Default)]
pub struct ExportedAnimationSet {
    /// 当前可合成声明全集。
    pub declarations: Vec<ExportedAnimationDeclaration>,
    /// 元素 key → 已导出通道（每通道至多一条，分类规则 5）。
    pub channels: HashMap<u64, Vec<ExportedChannel>>,
}

impl ExportedAnimationSet {
    /// 查元素某通道的声明 id。
    pub fn channel(&self, element_key: u64, property: ExportedProperty) -> Option<u64> {
        self.channels
            .get(&element_key)?
            .iter()
            .find(|c| c.property == property)
            .map(|c| c.animation_id)
    }

    /// 静态路径跳写判定：该元素的该插值属性（"opacity"/"transform"）已导出时
    /// t=0 叠加不得写入样式（compositor 在干净基线上采样）。
    pub fn skips_property(&self, element_key: u64, property: &str) -> bool {
        let prop = match property {
            "opacity" => ExportedProperty::Opacity,
            "transform" => ExportedProperty::Transform,
            // individual transform 属性不导出（模块头记档），照常叠加。
            _ => return false,
        };
        self.channel(element_key, prop).is_some()
    }

    /// 从声明集重建通道表（M2-S1b：增量臂复用最近全量帧的导出——不重跑分类，
    /// painter 照常发射作用域，draw 区间随本帧 paint 期自然生成）。属性通道取
    /// 首帧值变体（导出规则 1 保证端点均质）。
    pub fn channels_from_declarations(
        declarations: &[ExportedAnimationDeclaration],
    ) -> HashMap<u64, Vec<ExportedChannel>> {
        let mut channels: HashMap<u64, Vec<ExportedChannel>> = HashMap::new();
        for declaration in declarations {
            let property = match declaration.keyframes.first().map(|kf| &kf.value) {
                Some(ExportedAnimationValue::Opacity(_)) => ExportedProperty::Opacity,
                Some(ExportedAnimationValue::Transform(_)) => ExportedProperty::Transform,
                None => continue,
            };
            channels
                .entry(declaration.element_key)
                .or_default()
                .push(ExportedChannel {
                    property,
                    animation_id: declaration.animation_id,
                });
        }
        channels
    }
}

/// FNV-1a 64（跨进程确定——id 须在 renderer 锚点缓存与 compositor 声明替换间稳定）。
fn animation_id(element_key: u64, name: &str, property: ExportedProperty) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut feed = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    feed(&element_key.to_le_bytes());
    feed(name.as_bytes());
    feed(&[match property {
        ExportedProperty::Opacity => 0u8,
        ExportedProperty::Transform => 1u8,
    }]);
    hash
}

/// 导出主入口：遍历声明了 animation-name 的元素，按分类规则导出可合成通道。
///
/// `styles` 为 t=0 叠加**之前**的 underlying 快照（调用方在 R3882 静态求值前调用）；
/// `clock` 只读 @keyframes 注册表（边界补齐镜像 `start_animation_with_underlying`
/// 的 clone+resolve 序列，不启动动画——启动仍由静态求值在 clone 时钟上进行）。
/// 同名 animation-name 重复声明只分类一次（引擎 `start_from_computed_style`
/// 已活跃跳过的同帧镜像）。
pub fn export_composable_animations(
    clock: &AnimationClock,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> ExportedAnimationSet {
    let mut set = ExportedAnimationSet::default();
    for (node_id, style) in styles {
        if !style.animation_name.iter().any(|n| !n.is_empty() && n != "none") {
            continue;
        }
        let element_key = node_id.data().as_ffi();
        let mut seen = HashSet::new();
        for (idx, name) in style.animation_name.iter().enumerate() {
            if name.is_empty() || name == "none" || !seen.insert(name.clone()) {
                continue;
            }
            // 与 start_from_computed_style 同序取 per-animation 计时字段。
            let config = AnimationConfig {
                name: name.clone(),
                duration: style.animation_duration.get(idx).copied().unwrap_or(0.0),
                delay: style.animation_delay.get(idx).copied().unwrap_or(0.0),
                timing_function: style
                    .animation_timing_function
                    .get(idx)
                    .cloned()
                    .unwrap_or(TimingFunctionValue::Ease),
                iteration_count: style.animation_iteration_count.get(idx).and_then(|v| *v),
                direction: style
                    .animation_direction
                    .get(idx)
                    .cloned()
                    .unwrap_or(AnimationDirectionValue::Normal),
                fill_mode: style
                    .animation_fill_mode
                    .get(idx)
                    .cloned()
                    .unwrap_or(AnimationFillModeValue::None),
                // t=0 求值域：导出声明的时钟域 start 恒 0（`start_time_s`）。
                current_time: 0.0,
            };
            export_one(&mut set, clock, style, element_key, &config);
        }
    }
    // 跨帧稳定序（renderer 代际比较按 Vec 相等；HashMap 遍历序不定）。
    set.declarations.sort_by_key(|d| d.animation_id);
    set
}

/// 单动画分类导出（规则 1-6；任一规则不过即放弃，不留半导出状态）。
fn export_one(
    set: &mut ExportedAnimationSet,
    clock: &AnimationClock,
    style: &ComputedStyle,
    element_key: u64,
    config: &AnimationConfig,
) {
    // 规则 6：t=0 已结束（负 delay 越过末轮；engine tick 结束分支烘焙终帧，不走合成域）。
    if finished_at_t0(config.duration, config.delay, config.iteration_count) {
        return;
    }
    let Some(points) = clock.keyframes_registry.get(&config.name) else {
        return;
    };
    // 镜像 start_animation_with_underlying：clone 注册表端点并按 underlying 补界。
    let mut points = points.clone();
    super::resolve_implicit_boundaries(&mut points, style);
    if points.len() < 2 {
        return;
    }

    // 规则 4：underlying 干净门（透明度基线烘焙是采样域无法表达的成分）。
    // paint 效果面（blend/mask/filter）与 alpha 烘焙、图元重组（url() filter isolate
    // 会追加图元）弱合成不可交换，一并要求干净——两通道同门。
    let paint_effect_clean = style.filter.is_empty()
        && style.backdrop_filter.is_empty()
        && style.mask_image.is_empty()
        && matches!(style.mix_blend_mode, MixBlendModeComputedValue::Normal);
    let opacity_clean = paint_effect_clean && style.opacity >= 1.0 - 1e-6;
    let transform_clean = paint_effect_clean
        && matches!(style.transform, TransformValue::None)
        && style.individual_translate.is_none()
        && style.individual_rotate.is_none()
        && style.individual_scale.is_none();

    // 规则 1-3：逐通道提取端点列。
    let opacity_keyframes = if opacity_clean {
        extract_opacity_channel(&points)
    } else {
        None
    };
    let transform_keyframes = if transform_clean {
        extract_transform_channel(&points)
    } else {
        None
    };

    // 规则 5：单通道单动画——同元素同属性已有导出时新旧一并拒绝（引擎按声明序
    // 覆盖的语义不可采样，两侧都保持冻结基线）。
    for (property, keyframes) in [
        (ExportedProperty::Opacity, opacity_keyframes),
        (ExportedProperty::Transform, transform_keyframes),
    ] {
        let Some(keyframes) = keyframes else { continue };
        let channels = set.channels.entry(element_key).or_default();
        if let Some(existing) = channels.iter().find(|c| c.property == property).copied() {
            channels.retain(|c| c.animation_id != existing.animation_id);
            if channels.is_empty() {
                set.channels.remove(&element_key);
            }
            set.declarations.retain(|d| d.animation_id != existing.animation_id);
            continue;
        }
        let id = animation_id(element_key, &config.name, property);
        channels.push(ExportedChannel {
            property,
            animation_id: id,
        });
        set.declarations.push(ExportedAnimationDeclaration {
            animation_id: id,
            element_key,
            name: config.name.clone(),
            duration_s: config.duration.max(0.001),
            delay_s: config.delay,
            iterations: config.iteration_count,
            direction: config.direction.clone(),
            fill: config.fill_mode.clone(),
            easing: config.timing_function.clone(),
            start_time_s: config.current_time,
            keyframes,
        });
    }
}

/// t=0 相位判定：负 delay 使 active_elapsed = -delay 越过总活跃时长 → 已结束。
fn finished_at_t0(duration: f64, delay: f64, iterations: Option<f64>) -> bool {
    let Some(max_iter) = iterations else {
        return false; // infinite 永不结束
    };
    delay <= 0.0 && duration > 0.0 && ((-delay) / duration) >= max_iter
}

/// opacity 通道提取（规则 1：每帧都声明且可解析）。
fn extract_opacity_channel(points: &[KeyframePoint]) -> Option<Vec<ExportedAnimationKeyframe>> {
    let mut out = Vec::with_capacity(points.len());
    for point in points {
        let raw = point.properties.get("opacity")?;
        let value = super::parse_f64(raw)?;
        out.push(ExportedAnimationKeyframe {
            offset: point.offset,
            value: ExportedAnimationValue::Opacity(value),
            easing: point.timing.clone(),
        });
    }
    Some(out)
}

/// transform 通道提取（规则 1-3）：解析 → 2D 子集过滤 → none 恒等展开 → 同构判定。
fn extract_transform_channel(points: &[KeyframePoint]) -> Option<Vec<ExportedAnimationKeyframe>> {
    // individual transform 属性动画不导出（协议通道只承载 transform；引擎双通道
    // 叠加序不可采样）。
    for point in points {
        for name in point.properties.keys() {
            if matches!(name.as_str(), "translate" | "rotate" | "scale") {
                return None;
            }
        }
    }

    // 逐帧解析；记录首个非 none 形状作同构参照。
    let mut parsed: Vec<Option<Vec<TransformFunction>>> = Vec::with_capacity(points.len());
    let mut reference: Option<Vec<TransformFunction>> = None;
    for point in points {
        let raw = point.properties.get("transform")?;
        match parse_transform(raw)? {
            TransformValue::None => parsed.push(None),
            TransformValue::List(fns) => {
                if fns.iter().any(|f| !is_exportable_2d(f)) {
                    return None;
                }
                match &reference {
                    // 参照形状与端点列各持一份（帧值同构由 homogeneous 保证，克隆
                    // 仅发生在每动画首个非 none 帧）。
                    None => reference = Some(fns.clone()),
                    Some(r) if !homogeneous(r, &fns) => return None,
                    _ => {}
                }
                parsed.push(Some(fns));
            }
        }
    }
    // 全 none：无可采样成分（视觉恒等），拒绝。
    let reference = reference?;

    let mut out = Vec::with_capacity(points.len());
    for (point, functions) in points.iter().zip(parsed) {
        let value = match functions {
            Some(fns) => ExportedAnimationValue::Transform(fns),
            // none 帧 → 参照形状恒等展开（镜像 `transform_interp::identity_list`）。
            None => ExportedAnimationValue::Transform(identity_list(&reference)),
        };
        out.push(ExportedAnimationKeyframe {
            offset: point.offset,
            value,
            easing: point.timing.clone(),
        });
    }
    Some(out)
}

/// 2D 可采样函数（px / deg / 无量纲；`%` 分量与 matrix 拒绝，模块头规则 2）。
fn is_exportable_2d(f: &TransformFunction) -> bool {
    matches!(
        f,
        TransformFunction::Translate(_, _)
            | TransformFunction::TranslateX(_)
            | TransformFunction::TranslateY(_)
            | TransformFunction::Rotate(_)
            | TransformFunction::RotateZ(_)
            | TransformFunction::Scale(_, _)
            | TransformFunction::ScaleX(_)
            | TransformFunction::ScaleY(_)
            | TransformFunction::Skew(_, _)
    )
}

/// 同构判定：等长且逐位同变体（引擎 `lerp_lists` 离散回退规则的同义镜像；
/// rotate 与 rotateZ 视为异种——引擎逐位配对要求同变体）。
fn homogeneous(a: &[TransformFunction], b: &[TransformFunction]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b.iter())
            .all(|(x, y)| std::mem::discriminant(x) == std::mem::discriminant(y))
}

/// 对侧函数序列的恒等对应（与 `transform_interp::identity_list` 同构——该函数为
/// 模块私有，此处按已过 `is_exportable_2d` 过滤的输入展开，FIXME(m2-s2)：收敛
/// 到单一实现当协议通道需要 % / matrix 支撑时一并对齐）。
fn identity_list(fns: &[TransformFunction]) -> Vec<TransformFunction> {
    fns.iter()
        .map(|f| match f {
            TransformFunction::Translate(_, _) => TransformFunction::Translate(0.0, 0.0),
            TransformFunction::TranslateX(_) => TransformFunction::TranslateX(0.0),
            TransformFunction::TranslateY(_) => TransformFunction::TranslateY(0.0),
            TransformFunction::Rotate(_) | TransformFunction::RotateZ(_) => TransformFunction::Rotate(0.0),
            TransformFunction::Scale(_, _) => TransformFunction::Scale(1.0, Some(1.0)),
            TransformFunction::ScaleX(_) => TransformFunction::ScaleX(1.0),
            TransformFunction::ScaleY(_) => TransformFunction::ScaleY(1.0),
            TransformFunction::Skew(_, _) => TransformFunction::Skew(0.0, Some(0.0)),
            // 已过 is_exportable_2d 过滤，其余变体不可达。
            other => other.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use slotmap::Key;
    use zero_css_parser::values::{AnimationDirectionValue, AnimationFillModeValue, TimingFunctionValue};
    use zero_dom::{NodeId, parse_html};
    use zero_style_system::ComputedStyle;

    use super::*;
    use crate::animation::{AnimationClock, KeyframePoint};

    /// 直接构造注册表端点（绕过 CSS 解析；端点为已补界形态，与
    /// `AnimationState.keyframes` 一致）。
    fn registry_clock(entries: Vec<(&str, Vec<(f64, &[(&str, &str)])>)>) -> AnimationClock {
        let mut clock = AnimationClock::new();
        for (name, points) in entries {
            clock.keyframes_registry.insert(
                name.to_string(),
                points
                    .into_iter()
                    .map(|(offset, props)| KeyframePoint {
                        offset,
                        properties: props
                            .iter()
                            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                            .collect(),
                        timing: None,
                    })
                    .collect(),
            );
        }
        clock
    }

    /// 单元素导出（NodeId 取自真实文档——slotmap key 无任意构造）。
    fn export_one_elem(clock: &AnimationClock, style: &ComputedStyle) -> (ExportedAnimationSet, u64) {
        let doc = parse_html("<div></div>");
        let nid = doc.query_selector(doc.root(), "div").expect("div in fixture");
        let element_key = nid.data().as_ffi();
        let mut styles = HashMap::new();
        styles.insert(nid, style.clone());
        (export_composable_animations(clock, &styles), element_key)
    }

    fn style_with_animation(name: &str, duration: f64, delay: f64) -> ComputedStyle {
        let mut style = ComputedStyle::default();
        style.animation_name = vec![name.to_string()];
        style.animation_duration = vec![duration];
        style.animation_delay = vec![delay];
        style
    }

    #[test]
    fn opacity_channel_exports_both_endpoints() {
        let clock = registry_clock(vec![(
            "fade",
            vec![(0.0, &[("opacity", "0.0")]), (1.0, &[("opacity", "1.0")])],
        )]);
        let style = style_with_animation("fade", 2.0, 0.0);
        let (set, key) = export_one_elem(&clock, &style);
        assert_eq!(set.declarations.len(), 1);
        let decl = &set.declarations[0];
        assert_eq!(decl.element_key, key);
        assert_eq!(decl.duration_s, 2.0);
        assert_eq!(
            decl.keyframes,
            vec![
                ExportedAnimationKeyframe {
                    offset: 0.0,
                    value: ExportedAnimationValue::Opacity(0.0),
                    easing: None
                },
                ExportedAnimationKeyframe {
                    offset: 1.0,
                    value: ExportedAnimationValue::Opacity(1.0),
                    easing: None
                },
            ]
        );
        // 通道映射可查，跳写集命中 opacity、不命中 transform/individual。
        assert_eq!(set.channel(key, ExportedProperty::Opacity), Some(decl.animation_id));
        assert!(set.skips_property(key, "opacity"));
        assert!(!set.skips_property(key, "transform"));
        assert!(!set.skips_property(key, "translate"));
    }

    #[test]
    fn from_only_opacity_is_rejected() {
        // 100% 边界帧缺 opacity（resolve_implicit_boundaries 只补 transform/individual）：
        // 引擎 to 侧驱动零输出 → 导出拒绝（冻结同视觉）。
        let clock = registry_clock(vec![("fade", vec![(0.0, &[("opacity", "0.0")]), (1.0, &[])])]);
        let style = style_with_animation("fade", 2.0, 0.0);
        let (set, _) = export_one_elem(&clock, &style);
        assert!(set.declarations.is_empty());
    }

    #[test]
    fn from_only_transform_boundary_fills_with_identity() {
        // R5191：none 端点帧 → 恒等展开导出（对侧形状）。
        let clock = registry_clock(vec![(
            "slide",
            vec![
                (0.0, &[("transform", "translateX(40px)")]),
                (1.0, &[("transform", "none")]),
            ],
        )]);
        let style = style_with_animation("slide", 1.0, 0.0);
        let (set, _) = export_one_elem(&clock, &style);
        assert_eq!(set.declarations.len(), 1);
        let kf = &set.declarations[0].keyframes;
        assert_eq!(
            kf[1].value,
            ExportedAnimationValue::Transform(vec![TransformFunction::TranslateX(0.0)])
        );
    }

    #[test]
    fn three_d_and_percent_and_matrix_are_rejected() {
        let cases: Vec<Vec<(f64, &[(&str, &str)])>> = vec![
            vec![
                (0.0, &[("transform", "rotateX(30deg)")]),
                (1.0, &[("transform", "rotateX(60deg)")]),
            ],
            vec![
                (0.0, &[("transform", "translate(10%, 0)")]),
                (1.0, &[("transform", "translate(20%, 0)")]),
            ],
            vec![
                (0.0, &[("transform", "matrix(1,0,0,1,0,0)")]),
                (1.0, &[("transform", "matrix(2,0,0,2,5,5)")]),
            ],
        ];
        for points in cases {
            let clock = registry_clock(vec![("m", points)]);
            let style = style_with_animation("m", 1.0, 0.0);
            let (set, _) = export_one_elem(&clock, &style);
            assert!(set.declarations.is_empty(), "应拒绝不可采样端点");
        }
    }

    #[test]
    fn heterogeneous_lists_are_rejected() {
        let clock = registry_clock(vec![(
            "mix",
            vec![
                (0.0, &[("transform", "translateX(10px)")]),
                (1.0, &[("transform", "scale(2)")]),
            ],
        )]);
        let style = style_with_animation("mix", 1.0, 0.0);
        let (set, _) = export_one_elem(&clock, &style);
        assert!(set.declarations.is_empty());
    }

    #[test]
    fn individual_transform_keyframes_are_rejected() {
        let clock = registry_clock(vec![(
            "ind",
            vec![(0.0, &[("translate", "0px")]), (1.0, &[("translate", "10px")])],
        )]);
        let style = style_with_animation("ind", 1.0, 0.0);
        let (set, _) = export_one_elem(&clock, &style);
        assert!(set.declarations.is_empty());
    }

    #[test]
    fn underlying_dirty_gates_respectively() {
        let clock = registry_clock(vec![(
            "both",
            vec![
                (0.0, &[("opacity", "0.0"), ("transform", "translateX(0px)")]),
                (1.0, &[("opacity", "1.0"), ("transform", "translateX(100px)")]),
            ],
        )]);
        // underlying opacity 0.5 → opacity 通道拒绝；transform 通道仍导出。
        let mut style = style_with_animation("both", 1.0, 0.0);
        style.opacity = 0.5;
        let (set, key) = export_one_elem(&clock, &style);
        assert_eq!(set.declarations.len(), 1);
        assert!(set.skips_property(key, "transform"));
        assert!(!set.skips_property(key, "opacity"));

        // underlying transform 声明 → transform 通道拒绝；opacity 仍导出。
        let mut style = style_with_animation("both", 1.0, 0.0);
        style.transform = TransformValue::List(vec![TransformFunction::TranslateY(5.0)]);
        let (set, key) = export_one_elem(&clock, &style);
        assert_eq!(set.declarations.len(), 1);
        assert!(set.skips_property(key, "opacity"));
        assert!(!set.skips_property(key, "transform"));
    }

    #[test]
    fn duplicate_channel_rejects_both() {
        // 两动画同通道：引擎按声明序覆盖不可采样，双双拒绝。
        let clock = registry_clock(vec![
            ("a", vec![(0.0, &[("opacity", "0.0")]), (1.0, &[("opacity", "1.0")])]),
            ("b", vec![(0.0, &[("opacity", "1.0")]), (1.0, &[("opacity", "0.0")])]),
        ]);
        let mut style = style_with_animation("a", 1.0, 0.0);
        style.animation_name.push("b".to_string());
        style.animation_duration.push(1.0);
        style.animation_delay.push(0.0);
        let (set, _) = export_one_elem(&clock, &style);
        assert!(set.declarations.is_empty());
        assert!(set.channels.is_empty());
    }

    #[test]
    fn finished_at_t0_not_exported() {
        // 负 delay -3s × 时长 1s × 2 轮：t=0 已越过末轮 → 静态路径烘焙终帧，不导出。
        let clock = registry_clock(vec![(
            "fade",
            vec![(0.0, &[("opacity", "0.0")]), (1.0, &[("opacity", "1.0")])],
        )]);
        let mut style = style_with_animation("fade", 1.0, -3.0);
        style.animation_iteration_count = vec![Some(2.0)];
        let (set, _) = export_one_elem(&clock, &style);
        assert!(set.declarations.is_empty());
        // infinite 迭代永不结束，同样负 delay 仍导出。
        let mut style = style_with_animation("fade", 1.0, -3.0);
        style.animation_iteration_count = vec![None];
        let (set, _) = export_one_elem(&clock, &style);
        assert_eq!(set.declarations.len(), 1);
    }

    #[test]
    fn animation_id_is_deterministic_and_channel_separated() {
        let a = animation_id(7, "fade", ExportedProperty::Opacity);
        let b = animation_id(7, "fade", ExportedProperty::Opacity);
        let c = animation_id(7, "fade", ExportedProperty::Transform);
        let d = animation_id(8, "fade", ExportedProperty::Opacity);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }

    #[test]
    fn per_keyframe_timing_and_direction_fill_carry_through() {
        let mut clock = AnimationClock::new();
        clock.keyframes_registry.insert(
            "paced".to_string(),
            vec![
                KeyframePoint {
                    offset: 0.0,
                    properties: [("opacity".to_string(), "0.0".to_string())].into_iter().collect(),
                    timing: Some(TimingFunctionValue::EaseIn),
                },
                KeyframePoint {
                    offset: 1.0,
                    properties: [("opacity".to_string(), "1.0".to_string())].into_iter().collect(),
                    timing: None,
                },
            ],
        );
        let mut style = style_with_animation("paced", 1.0, 0.0);
        style.animation_direction = vec![AnimationDirectionValue::Alternate];
        style.animation_fill_mode = vec![AnimationFillModeValue::Both];
        let (set, _) = export_one_elem(&clock, &style);
        let decl = &set.declarations[0];
        assert_eq!(decl.direction, AnimationDirectionValue::Alternate);
        assert_eq!(decl.fill, AnimationFillModeValue::Both);
        assert_eq!(decl.keyframes[0].easing, Some(TimingFunctionValue::EaseIn));
        assert_eq!(decl.keyframes[1].easing, None);
    }

    // ── render_html 导出路径（flag 门控端到端） ──────────────────────────

    const FADE_HTML: &str = r#"<html><body><div class="a">x</div></body></html>"#;
    const FADE_CSS: &str = r#"
        @keyframes fade { from { opacity: 0.3 } to { opacity: 1 } }
        .a { width: 50px; height: 50px; background-color: rgb(10, 20, 30); animation: fade 1s linear; }
    "#;

    #[test]
    fn render_flag_on_exports_declaration_and_neutralizes_static_bake() {
        let mut pipeline = crate::pipeline::RenderPipeline::new(800.0, 600.0);
        pipeline.set_compositor_animations(true);
        let on = pipeline.render_html(FADE_HTML, FADE_CSS);
        assert_eq!(on.animation_declarations.len(), 1, "fade 声明应导出");
        let decl = &on.animation_declarations[0];
        assert_eq!(decl.name, "fade");
        assert!(
            matches!(decl.keyframes.first().map(|k| &k.value),
                Some(ExportedAnimationValue::Opacity(o)) if (*o - 0.3).abs() < 1e-9),
            "首帧应为 from opacity 0.3"
        );
        assert_eq!(on.animation_scopes.len(), 1, "opacity 通道应有一个作用域");
        let scope = &on.animation_scopes[0];
        assert_eq!(scope.animation_id, decl.animation_id);
        assert_eq!(
            scope.property,
            zero_render_foundation::primitive::PaintAnimationProperty::Opacity
        );
        assert!(scope.draw_end > scope.draw_begin, "作用域区间非空");

        // 对照帧（flag 关）：无导出，t=0 值 0.3 烘焙进 fill alpha。
        let off = crate::pipeline::RenderPipeline::new(800.0, 600.0).render_html(FADE_HTML, FADE_CSS);
        assert!(
            off.animation_declarations.is_empty() && off.animation_scopes.is_empty(),
            "flag 关导出恒空"
        );
        let on_alpha = on.primitives().fills.iter().map(|f| f.color.a).max().unwrap();
        let off_alpha = off.primitives().fills.iter().map(|f| f.color.a).max().unwrap();
        assert!(
            (f64::from(off_alpha) - 0.3 * 255.0).abs() < 2.0,
            "flag 关烘焙 t=0 值（alpha={off_alpha}）"
        );
        assert_eq!(
            on_alpha, 255,
            "flag 开保持 underlying 不透明（跳写 + 中性化，alpha={on_alpha}）"
        );
    }

    #[test]
    fn render_flag_on_transform_scope_binds_identity_transform_slot() {
        let css = r#"
            @keyframes slide { from { transform: translateX(0px) } to { transform: translateX(50px) } }
            .a { width: 50px; height: 50px; background-color: red; animation: slide 1s linear; }
        "#;
        let mut pipeline = crate::pipeline::RenderPipeline::new(800.0, 600.0);
        pipeline.set_compositor_animations(true);
        let result = pipeline.render_html(FADE_HTML, css);
        assert_eq!(result.animation_declarations.len(), 1, "slide 声明应导出");
        let decl = &result.animation_declarations[0];
        assert!(matches!(
            decl.keyframes.first().map(|k| &k.value),
            Some(ExportedAnimationValue::Transform(_))
        ));
        let scope = result
            .animation_scopes
            .iter()
            .find(|s| s.property == zero_render_foundation::primitive::PaintAnimationProperty::Transform)
            .expect("transform 作用域应发射");
        assert_eq!(scope.animation_id, decl.animation_id);
        let idx = scope.transform_index.expect("transform 通道必须带槽位");
        let tp = &result.primitives().transforms[idx as usize];
        // 恒等基线：compositor 采样后就地改写该槽位（RFC §2.3-2）。
        assert!(
            (tp.a - 1.0).abs() < 1e-6
                && tp.b.abs() < 1e-6
                && tp.c.abs() < 1e-6
                && (tp.d - 1.0).abs() < 1e-6
                && tp.tx.abs() < 1e-6
                && tp.ty.abs() < 1e-6,
            "槽位应为恒等矩阵，实际 {tp:?}"
        );
    }

    #[test]
    fn render_flag_on_rejected_matrix_animation_stays_statically_baked() {
        let css = r#"
            @keyframes mat { from { transform: matrix(1, 0, 0, 1, 0, 0) } to { transform: matrix(1, 0, 0, 1, 10, 0) } }
            .a { width: 50px; height: 50px; background-color: red; animation: mat 1s linear; }
        "#;
        let mut pipeline = crate::pipeline::RenderPipeline::new(800.0, 600.0);
        pipeline.set_compositor_animations(true);
        let result = pipeline.render_html(FADE_HTML, css);
        // matrix 关键帧超 2D 子集（分类规则 2）→ 不导出；静态路径照常烘焙 t=0 值
        //（跳写只作用于已导出通道），动画冻结在 t=0 形态与 flag 关一致。
        assert!(result.animation_declarations.is_empty(), "matrix 动画不导出");
        assert!(result.animation_scopes.is_empty(), "无导出即无作用域");
    }
}
