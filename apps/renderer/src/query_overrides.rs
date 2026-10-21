//! M2-S2a：gCS 查询覆盖采样 provider（renderer 侧，goal：compositor-animation-continuity）。
//!
//! flag 开启且动画经 compositor 采样时，`getComputedStyle` 对已导出通道（opacity/
//! transform）返回与合成视觉一致的采样值（DC-2 查询一致性，RFC §2.1）。真相源 =
//! 已发送的协议声明集 + `CLOCK_MONOTONIC` + `zero_protocol::animation::sample_declaration`
//! ——与 compositor Get 采样完全同源（同函数、同钟域、无 IPC）。
//!
//! 线程边界：表由 renderer 主线程在声明代际发布点构建；js worker 线程的 gCS 宿主
//! 回调（engine `query_style_override`）只读哈希表，零 doc 访问（`Rc<RefCell<Document>>
//! 不可跨线程）。元素身份 = 选择器串（engine 导出侧双形态产出：`unique_selector_for_node`
//! ——shim 元素代理 `__zwSelector` 的同源函数，gCS 回调传入串直查命中——+
//! `structural_path_selector` 恒唯一结构路径兜底；跨文档 NodeId 不可比）。
//!
//! 序列化口径（镜像 engine gCS 惯例，spec-correct 函数列表 diverge 不扩大）：
//! - opacity 整数无小数点（`"1"`/`"0"`；engine `serialize_computed_property` 同则）；
//! - transform 返**函数列表**（非折叠 matrix——engine `transform_to_css` 对非 none
//!   transform 返函数列表；compositor 侧的仿射折叠发生在像素应用，查询侧不折叠）；
//! - 数字 3 位舍入、整数无小数点、px/deg 后缀（engine `format_num` 镜像）；
//! - 单参归一：协议 `Scale(sx, sy)`/`Skew(ax, ay)` 已丢失元数（`Scale(sx, None)` →
//!   `Scale(sx, sx)` 等），sx==sy / ay==0 时返单参形与 engine 单参序列化惯例对齐
//!   （两种写法 CSS 语义等价，视觉无差）。
//!
//! RFC：`docs/goal/compositor-animation-continuity/m2-slice-query-consistency.md`。

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use zero_engine::animation::compositor_export::ExportedAnimationDeclaration;
use zero_protocol::animation::{
    AnimationPropertyKind, AnimationPropertyValue, CompositorAnimationDeclaration, TransformFunctionValue,
    monotonic_now_ns, sample_declaration,
};

/// 单元素查询通道：每 (元素, 属性) 至多一条声明（engine 导出规则 5：单通道单动画）。
#[derive(Default)]
struct ElementQueryChannels {
    opacity: Option<Arc<CompositorAnimationDeclaration>>,
    transform: Option<Arc<CompositorAnimationDeclaration>>,
}

/// 结构路径 → 通道表。主线程构建、js worker 线程只读；查询 O(1) 哈希 + 采样。
#[derive(Default)]
pub struct QueryOverrideTable {
    elements: HashMap<String, ElementQueryChannels>,
}

impl QueryOverrideTable {
    /// 从「engine 声明（携元素选择器候选）+ 协议声明（可采样域）」构建表。按
    /// animation_id 配对（两侧 1:1，`animation_declaration_to_ipc` 失败的引擎声明整条
    /// 不发）；**双形态选择器各登记一键**（`unique_selector_for_node` 形态 = shim
    /// `__zwSelector` 主形态，结构路径兜底），gCS 回调传入串直查命中。候选为空的
    /// 声明（仅测试构造）不进表——查询侧保持底层值。
    pub fn build(engine: &[ExportedAnimationDeclaration], ipc: &[CompositorAnimationDeclaration]) -> Self {
        let by_id: HashMap<u64, &CompositorAnimationDeclaration> = ipc.iter().map(|d| (d.animation_id, d)).collect();
        let mut elements: HashMap<String, ElementQueryChannels> = HashMap::new();
        for declaration in engine {
            let Some(ipc_declaration) = by_id.get(&declaration.animation_id) else {
                continue;
            };
            for selector in &declaration.element_selectors {
                let entry = elements.entry(selector.clone()).or_default();
                let slot = match ipc_declaration.property {
                    AnimationPropertyKind::Opacity => &mut entry.opacity,
                    AnimationPropertyKind::Transform => &mut entry.transform,
                };
                *slot = Some(Arc::new((*ipc_declaration).clone()));
            }
        }
        Self { elements }
    }

    /// 查询覆盖采样：该 (路径, 属性) 在当前时刻的序列化 CSS 值；`None` = 无覆盖
    /// （未导出 / 已结束且无 forwards fill / 延迟期且无 backwards fill——回落底层
    /// 级联，与 compositor「无覆盖 → 基线像素」镜像）。
    fn sample(&self, selector: &str, property: &str, now_ns: u64) -> Option<String> {
        let channels = self.elements.get(selector)?;
        let declaration = match property {
            "opacity" => channels.opacity.as_ref()?,
            "transform" => channels.transform.as_ref()?,
            _ => return None,
        };
        let sample = sample_declaration(declaration, now_ns);
        match sample.value? {
            AnimationPropertyValue::Opacity(o) => Some(format_opacity(o)),
            AnimationPropertyValue::Transform(fns) => Some(format_transform(&fns)),
        }
    }
}

/// opacity 序列化：整数无小数点（engine `serialize_computed_property` 同则，与
/// real browser getComputedStyle 一致）。
fn format_opacity(o: f64) -> String {
    if o == o.trunc() {
        format!("{}", o as i32)
    } else {
        format!("{o}")
    }
}

/// 数字格式：3 位舍入、整数无小数点、单位后缀（engine `format_num` 镜像）。
fn format_num(v: f64, suffix: &str) -> String {
    let r = (v * 1000.0).round() / 1000.0;
    if r == r.trunc() {
        format!("{}{}", r as i64, suffix)
    } else {
        format!("{r}{suffix}")
    }
}

/// transform 序列化为函数列表（模块doc 口径；单参归一见模块头）。
fn format_transform(fns: &[TransformFunctionValue]) -> String {
    fns.iter()
        .map(|f| match f {
            TransformFunctionValue::Translate(x, y) => {
                format!("translate({}, {})", format_num(*x, "px"), format_num(*y, "px"))
            }
            TransformFunctionValue::TranslateX(x) => format!("translateX({})", format_num(*x, "px")),
            TransformFunctionValue::TranslateY(y) => format!("translateY({})", format_num(*y, "px")),
            TransformFunctionValue::Rotate(deg) => format!("rotate({}deg)", format_num(*deg, "")),
            TransformFunctionValue::Scale(sx, sy) => {
                if sx == sy {
                    format!("scale({})", format_num(*sx, ""))
                } else {
                    format!("scale({}, {})", format_num(*sx, ""), format_num(*sy, ""))
                }
            }
            TransformFunctionValue::ScaleX(x) => format!("scaleX({})", format_num(*x, "")),
            TransformFunctionValue::ScaleY(y) => format!("scaleY({})", format_num(*y, "")),
            TransformFunctionValue::Skew(ax, ay) => {
                if *ay == 0.0 {
                    format!("skew({}deg)", format_num(*ax, ""))
                } else {
                    format!("skew({}deg, {}deg)", format_num(*ax, ""), format_num(*ay, ""))
                }
            }
            TransformFunctionValue::Matrix(m) => {
                let components = m.iter().map(|v| format_num(*v, "")).collect::<Vec<_>>().join(", ");
                format!("matrix({components})")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// 发布新一代查询覆盖表（声明代际变化时；空声明发空表——清残留选择器键）。
/// 进程级表（renderer 主线程写、js worker gCS 回调线程读；`None` = 无覆盖，flag 关 /
/// 退出导出域 / 声明清空——查询路径零干扰）。engine 侧提供者槽
///（`set_query_style_override_provider`）经此闭包间接读表。
static QUERY_OVERRIDE_TABLE: RwLock<Option<Arc<QueryOverrideTable>>> = RwLock::new(None);

/// 发布新一代查询覆盖表（声明代际变化时；空声明发空表——清残留选择器键）。
fn publish(table: QueryOverrideTable) {
    if let Ok(mut slot) = QUERY_OVERRIDE_TABLE.write() {
        *slot = Some(Arc::new(table));
    }
}

/// 清空查询覆盖表（退出导出域 / flag 关初始化）。清后查询全回落底层级联。
fn clear() {
    if let Ok(mut slot) = QUERY_OVERRIDE_TABLE.write() {
        *slot = None;
    }
}

/// 注册查询覆盖 provider 到 engine 全局槽（flag 开启时 renderer 初始化调用一次；
/// flag 关不注册——FR-9 查询路径零注入）。provider 只读全局表，`Send + Sync`。
pub fn register_provider() {
    zero_engine::js_dom_bridge::set_query_style_override_provider(Some(Arc::new(|selector: &str, property: &str| {
        let table = QUERY_OVERRIDE_TABLE.read().ok()?.clone()?;
        table.sample(selector, property, monotonic_now_ns())
    })));
}

/// 声明代际发布点（renderer 主线程，`send_compositor_animation_state` advance 成功后）：
/// 由 engine 声明 + 协议声明构建并发布新表。
pub fn publish_declarations(engine: &[ExportedAnimationDeclaration], ipc: &[CompositorAnimationDeclaration]) {
    publish(QueryOverrideTable::build(engine, ipc));
}

/// 退出导出域清理（deactivate 发空集清理同点）：清表。
pub fn clear_overrides() {
    clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_protocol::animation::{
        AnimationDirection, AnimationEasing, AnimationFillMode, AnimationKeyframe, AnimationPropertyKind,
        AnimationTiming,
    };

    fn opacity_declaration(id: u64) -> CompositorAnimationDeclaration {
        CompositorAnimationDeclaration {
            animation_id: id,
            property: AnimationPropertyKind::Opacity,
            composable: true,
            start_monotonic_ns: 0,
            timing: AnimationTiming {
                duration_ms: 1000.0,
                delay_ms: 0.0,
                iterations: Some(1.0),
                direction: AnimationDirection::Normal,
                fill: AnimationFillMode::None,
                easing: AnimationEasing::Linear,
            },
            keyframes: vec![
                AnimationKeyframe {
                    offset: 0.0,
                    value: AnimationPropertyValue::Opacity(0.0),
                    easing: None,
                },
                AnimationKeyframe {
                    offset: 1.0,
                    value: AnimationPropertyValue::Opacity(1.0),
                    easing: None,
                },
            ],
        }
    }

    fn engine_declaration(id: u64, selectors: &[&str]) -> ExportedAnimationDeclaration {
        ExportedAnimationDeclaration {
            animation_id: id,
            element_key: id,
            element_selectors: selectors.iter().map(|s| (*s).to_string()).collect(),
            name: "fade".to_string(),
            duration_s: 1.0,
            delay_s: 0.0,
            iterations: Some(1.0),
            direction: zero_css_parser::values::AnimationDirectionValue::Normal,
            fill: zero_css_parser::values::AnimationFillModeValue::None,
            easing: zero_css_parser::values::TimingFunctionValue::Linear,
            start_time_s: 0.0,
            keyframes: Vec::new(),
        }
    }

    #[test]
    fn opacity_mid_sampling_formats_without_trailing_point() {
        let ipc = vec![opacity_declaration(7)];
        let engine = vec![engine_declaration(7, &["div:nth-child(1) > p:nth-child(2)"])];
        let table = QueryOverrideTable::build(&engine, &ipc);
        // t=0.5s → opacity 0.5（非整数控原样）；t=0 → "0"（整数无小数点）。
        assert_eq!(
            table.sample("div:nth-child(1) > p:nth-child(2)", "opacity", 500_000_000),
            Some("0.5".to_string())
        );
        assert_eq!(
            table.sample("div:nth-child(1) > p:nth-child(2)", "opacity", 0),
            Some("0".to_string())
        );
        // 已结束且无 forwards fill → 无覆盖回落底层（与 compositor 基线像素镜像；
        // 恰在终点 t=1s 也已 finished）。
        assert_eq!(
            table.sample("div:nth-child(1) > p:nth-child(2)", "opacity", 1_000_000_000),
            None
        );
        assert_eq!(
            table.sample("div:nth-child(1) > p:nth-child(2)", "opacity", 2_000_000_000),
            None
        );
    }

    #[test]
    fn transform_sampling_serializes_function_list() {
        let mut declaration = opacity_declaration(3);
        declaration.property = AnimationPropertyKind::Transform;
        declaration.timing.fill = AnimationFillMode::Forwards;
        declaration.keyframes = vec![
            AnimationKeyframe {
                offset: 0.0,
                value: AnimationPropertyValue::Transform(vec![TransformFunctionValue::Translate(0.0, 0.0)]),
                easing: None,
            },
            AnimationKeyframe {
                offset: 1.0,
                value: AnimationPropertyValue::Transform(vec![TransformFunctionValue::Translate(100.0, 50.5)]),
                easing: None,
            },
        ];
        let ipc = vec![declaration];
        let engine = vec![engine_declaration(3, &["div:nth-child(1)"])];
        let table = QueryOverrideTable::build(&engine, &ipc);
        assert_eq!(
            table.sample("div:nth-child(1)", "transform", 500_000_000),
            Some("translate(50px, 25.25px)".to_string())
        );
        // forwards fill：已结束保持终帧（整数/小数格式镜像 format_num）。
        assert_eq!(
            table.sample("div:nth-child(1)", "transform", 5_000_000_000),
            Some("translate(100px, 50.5px)".to_string())
        );
        // 未导出属性 / 未导出元素回落底层。
        assert_eq!(table.sample("div:nth-child(1)", "opacity", 0), None);
        assert_eq!(table.sample("other:nth-child(9)", "transform", 0), None);
        assert_eq!(table.sample("div:nth-child(1)", "color", 0), None);
    }

    #[test]
    fn pathless_engine_declarations_are_not_published() {
        let ipc = vec![opacity_declaration(7)];
        let engine = vec![engine_declaration(7, &[])];
        let table = QueryOverrideTable::build(&engine, &ipc);
        assert_eq!(table.sample("div:nth-child(1)", "opacity", 0), None);
    }

    /// RFC §2.3（FR-6 等价断言，renderer 侧）：同一声明扫 t，查询覆盖输出与共享采样器
    /// `sample_declaration` 输出同值（opacity 解析回数值比对）。compositor
    /// `animation_overrides::apply` 消费同一函数（apps/compositor/src/animation_overrides.rs
    /// apply 内逐声明采样）——单一实现即两侧等价的结构保证。
    #[test]
    fn query_output_matches_shared_sampler_across_scan() {
        let ipc = vec![opacity_declaration(7)];
        let engine = vec![engine_declaration(7, &["div:nth-child(1)"])];
        let table = QueryOverrideTable::build(&engine, &ipc);
        let sel = "div:nth-child(1)";
        for t_ms in [0u64, 1, 250, 500, 750, 999] {
            let now = t_ms * 1_000_000;
            let sample = sample_declaration(&ipc[0], now);
            let AnimationPropertyValue::Opacity(o) = sample.value.expect("活跃期有覆盖") else {
                panic!("fixture 是 opacity 通道");
            };
            let Some(text) = table.sample(sel, "opacity", now) else {
                panic!("t={t_ms}ms 查询应命中覆盖");
            };
            let parsed: f64 = text.parse().expect("opacity 序列化可解析");
            assert!((parsed - o).abs() < 1e-9, "t={t_ms}ms 查询 {text} 与采样器 {o} 不一致");
        }
    }
}
