//! `for_each` 列表渲染（spec §8.4.7 / FR-008 / DC-6）。
//!
//! 把带 `control.for_each` 的声明节点展开为 N 个具体子节点：
//! 1. 在外层 [`EvalContext`] 中求值 `for_each.source` 得到集合（`Value::Array`）。
//! 2. 对每个元素构造 **item 作用域**：把 `item_alias` 作为路径根注入（克隆后的）子 `EvalContext`。
//! 3. 在 item 作用域内求值子模板的 `bindings`（写入对应 props）、`visible_when`（决定是否纳入）、
//!    `enabled_when`（解析为字面布尔，供下游 host 在无 item 作用域时也能求值）。
//! 4. 产出具体 [`WidgetSpec`]，稳定 id = `<模板 id 或组件名>@<index>`。
//!
//! 安全（spec 约束 7 / DC-6 资源上限）：迭代数受 `EvalContext::max_iterations` 约束，
//! 超限 → [`DslError::EvalResourceLimit`]；求值无副作用、确定性（item 作用域是只读快照叠加，
//! 不修改原上下文）。表达式四阶段管线（parse/validate/typecheck/eval）复用 [`Engine`]，
//! 不引入新的可计算能力（无 lambda / 递归 / 状态写入）。

use crate::Engine;
use crate::diagnostics::DslError;
use crate::expression::Expression;
use crate::loader::{EvalContext, ExpressionEngine};
use compact_str::CompactString;
use zero_ui_core::binding::Value;
use zero_ui_core::widget::{WidgetId, WidgetSpec};

/// 在 item 作用域内求值已解析的表达式。
///
/// 以 `ctx` 为底克隆出子作用域，注入 `alias → item`；原 `ctx` 不受影响（无副作用）。
fn eval_in_item_scope(
    engine: &Engine,
    expr: &Expression,
    ctx: &EvalContext,
    alias: &str,
    item: &Value,
) -> Result<Value, DslError> {
    let mut scope = ctx.clone();
    scope.vars.insert(alias.to_string(), item.clone());
    engine.eval(expr, &scope)
}

/// 展开带 `control.for_each` 的节点为具体子节点列表（列表渲染，spec §8.4.7）。
///
/// - `spec`：带 `control.for_each` 的容器节点；取其 `children[0]` 作为**每项模板**
///   （多模板 / 更复杂模板编排留作 follow-up）。
/// - `engine` / `ctx`：表达式引擎与外层求值上下文；item 作用域基于 `ctx` 克隆叠加。
///
/// # 行为
/// - `source` 求值为 `Value::Array` → 逐项展开；`Value::Null`（路径缺失等）→ 空列表；
///   其它类型 → [`DslError::Typecheck`]。
/// - 每项：克隆模板 → 稳定 id `<模板id>@<idx>`（模板无 id 时用组件名）→
///   在 item 作用域求值每个 `binding.source` 写入 `binding.target` prop。
/// - 模板的 `visible_when`：在 item 作用域求值，**假值项跳过**（条件纳入）。
/// - 模板的 `enabled_when`：在 item 作用域求值，结果以字面 `"true"`/`"false"` 字符串写回
///   子节点（使下游 host 在无 item 作用域时也能正确求值）。
/// - 迭代数受 `ctx.max_iterations` 约束（DC-6 资源上限）。
///
/// # 错误
/// - 节点缺少 `control.for_each`、缺少子模板 → [`DslError::Validate`]。
/// - `source` 非数组（且非 Null）→ [`DslError::Typecheck`]。
/// - 迭代超 `max_iterations` → [`DslError::EvalResourceLimit`]。
/// - 任一表达式 parse/eval 错误透传。
pub fn materialize_for_each(
    spec: &WidgetSpec,
    engine: &Engine,
    ctx: &EvalContext,
) -> Result<Vec<WidgetSpec>, DslError> {
    let fe = spec
        .control
        .for_each
        .as_ref()
        .ok_or_else(|| DslError::Validate("materialize_for_each: 节点缺少 control.for_each".into()))?;
    let template = spec
        .children
        .first()
        .ok_or_else(|| DslError::Validate("for_each 节点缺少子模板（children[0]）".into()))?;

    // ── 求值数据源 ──────────────────────────────────────────────────────
    let source_expr = engine.parse(fe.source.as_str())?;
    let collection = engine.eval(&source_expr, ctx)?;
    let items = match collection {
        Value::Array(a) => a,
        Value::Null => return Ok(Vec::new()),
        other => {
            return Err(DslError::Typecheck(format!(
                "for_each source 必须为数组，got {:?}",
                other.value_type()
            )));
        }
    };

    // ── 预解析模板表达式（避免每项重复 parse）────────────────────────────
    let parsed_bindings: Vec<(&CompactString, Expression)> = template
        .bindings
        .iter()
        .map(|b| Ok((&b.target, engine.parse(b.source.as_str())?)))
        .collect::<Result<_, DslError>>()?;
    let parsed_visible = match &template.control.visible_when {
        Some(s) => Some(engine.parse(s.as_str())?),
        None => None,
    };
    let parsed_enabled = match &template.control.enabled_when {
        Some(s) => Some(engine.parse(s.as_str())?),
        None => None,
    };

    // 稳定 id 基名：模板 id 优先，否则组件名。
    let base_id = template
        .id
        .as_ref()
        .map(|id| id.0.as_str().to_string())
        .unwrap_or_else(|| template.component.0.as_str().to_string());

    let mut out = Vec::with_capacity(items.len().min(ctx.max_iterations));
    for (idx, item) in items.iter().enumerate() {
        if idx >= ctx.max_iterations {
            return Err(DslError::EvalResourceLimit(format!(
                "for_each 迭代数 > max_iterations {}",
                ctx.max_iterations
            )));
        }

        // visible_when：item 作用域求值，假值跳过（条件纳入）。
        if let Some(vw) = &parsed_visible {
            let v = eval_in_item_scope(engine, vw, ctx, fe.item_alias.as_str(), item)?;
            if !v.is_truthy() {
                continue;
            }
        }

        let mut child = template.clone();
        child.id = Some(WidgetId::new(&format!("{base_id}@{idx}")));
        // 已在 item 作用域解析为纳入/启用决策，清除模板继承的控制指令（防下游重复展开/误判）。
        child.control.for_each = None;
        child.control.visible_when = None;

        // bindings：item 作用域求值 → 写入对应 prop（列表渲染的核心数据通路）。
        for (target, expr) in &parsed_bindings {
            let v = eval_in_item_scope(engine, expr, ctx, fe.item_alias.as_str(), item)?;
            child.props.insert(target.as_str(), v);
        }

        // enabled_when：item 作用域求值，结果编码为字面布尔字符串（下游 host 无 item 作用域亦可求值）。
        if let Some(ew) = &parsed_enabled {
            let v = eval_in_item_scope(engine, ew, ctx, fe.item_alias.as_str(), item)?;
            child.control.enabled_when = Some(CompactString::new(if v.is_truthy() { "true" } else { "false" }));
        }

        out.push(child);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::EvalContext;
    use zero_ui_core::binding::{Binding, Value};
    use zero_ui_core::widget::{ComponentType, ControlDirectives, ForEachSpec, WidgetId, WidgetSpec};

    fn eng() -> Engine {
        Engine
    }

    /// 构造一个含 for_each 的容器：模板为 `Tab`，绑定 title ← `$tab.title`。
    fn tab_list_spec(alias: &str) -> WidgetSpec {
        let mut spec = WidgetSpec::new("TabList");
        spec.component = ComponentType::new("TabList");
        let mut tmpl = WidgetSpec::new("Tab");
        tmpl.component = ComponentType::new("Tab");
        tmpl.id = Some(WidgetId::new("tab"));
        tmpl.bindings = vec![Binding {
            target: CompactString::new("title"),
            source: CompactString::new("$tab.title"),
        }];
        spec.children = vec![tmpl];
        spec.control = ControlDirectives {
            for_each: Some(ForEachSpec {
                source: CompactString::new("$state.tabs"),
                item_alias: CompactString::new(alias),
            }),
            ..Default::default()
        };
        spec
    }

    fn tabs_ctx() -> EvalContext {
        // $state.tabs = [ {title: A}, {title: B}, {title: C} ]
        let tabs = Value::Array(vec![
            Value::Object([("title".to_string(), Value::Text("A".into()))].into_iter().collect()),
            Value::Object([("title".to_string(), Value::Text("B".into()))].into_iter().collect()),
            Value::Object([("title".to_string(), Value::Text("C".into()))].into_iter().collect()),
        ]);
        EvalContext::default().with_var(
            "state",
            Value::Object([("tabs".to_string(), tabs)].into_iter().collect()),
        )
    }

    #[test]
    fn materializes_one_child_per_item_with_stable_ids() {
        let out = materialize_for_each(&tab_list_spec("tab"), &eng(), &tabs_ctx()).expect("materialize");
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].id.as_ref().unwrap().0.as_str(), "tab@0");
        assert_eq!(out[1].id.as_ref().unwrap().0.as_str(), "tab@1");
        assert_eq!(out[2].id.as_ref().unwrap().0.as_str(), "tab@2");
        // 每个 title prop 已按 item 作用域解析。
        assert_eq!(out[0].props.get("title").unwrap(), &Value::Text("A".into()));
        assert_eq!(out[1].props.get("title").unwrap(), &Value::Text("B".into()));
        assert_eq!(out[2].props.get("title").unwrap(), &Value::Text("C".into()));
        // 展开后子节点不再带 for_each（防重复展开）。
        assert!(out[0].control.for_each.is_none());
    }

    #[test]
    fn custom_item_alias_resolves() {
        // 使用 `it` 作为别名，绑定也用 $it。
        let mut spec = tab_list_spec("ignored");
        spec.children[0].bindings = vec![Binding {
            target: CompactString::new("title"),
            source: CompactString::new("$it.title"),
        }];
        spec.control.for_each = Some(ForEachSpec {
            source: CompactString::new("$state.tabs"),
            item_alias: CompactString::new("it"),
        });
        let out = materialize_for_each(&spec, &eng(), &tabs_ctx()).expect("materialize");
        assert_eq!(out.len(), 3);
        assert_eq!(out[1].props.get("title").unwrap(), &Value::Text("B".into()));
    }

    #[test]
    fn nested_path_and_null_coalesce_in_item_scope() {
        // $tab.meta.title ?? 'New Tab'：缺 meta 字段时回落默认文案（spec 示例模式）。
        let mut spec = tab_list_spec("tab");
        spec.children[0].bindings = vec![Binding {
            target: CompactString::new("title"),
            source: CompactString::new("$tab.meta.title ?? 'New Tab'"),
        }];
        let ctx = EvalContext::default().with_var(
            "state",
            Value::Object(
                [(
                    "tabs".to_string(),
                    Value::Array(vec![
                        Value::Object(
                            [(
                                "meta".to_string(),
                                Value::Object(
                                    [("title".to_string(), Value::Text("Real".into()))]
                                        .into_iter()
                                        .collect(),
                                ),
                            )]
                            .into_iter()
                            .collect(),
                        ),
                        Value::Object([].into_iter().collect()), // 无 meta → 回落
                    ]),
                )]
                .into_iter()
                .collect(),
            ),
        );
        let out = materialize_for_each(&spec, &eng(), &ctx).expect("materialize");
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].props.get("title").unwrap(), &Value::Text("Real".into()));
        assert_eq!(out[1].props.get("title").unwrap(), &Value::Text("New Tab".into()));
    }

    #[test]
    fn visible_when_skips_falsy_items() {
        // 仅纳入 active == true 的项。
        let mut spec = tab_list_spec("tab");
        spec.children[0].control.visible_when = Some(CompactString::new("$tab.active"));
        spec.children[0].bindings.push(Binding {
            target: CompactString::new("title"),
            source: CompactString::new("$tab.title"),
        });
        let ctx = EvalContext::default().with_var(
            "state",
            Value::Object(
                [(
                    "tabs".to_string(),
                    Value::Array(vec![
                        Value::Object(
                            [
                                ("title".to_string(), Value::Text("A".into())),
                                ("active".to_string(), Value::Bool(true)),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                        Value::Object(
                            [
                                ("title".to_string(), Value::Text("B".into())),
                                ("active".to_string(), Value::Bool(false)),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                        Value::Object(
                            [
                                ("title".to_string(), Value::Text("C".into())),
                                ("active".to_string(), Value::Bool(true)),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                    ]),
                )]
                .into_iter()
                .collect(),
            ),
        );
        let out = materialize_for_each(&spec, &eng(), &ctx).expect("materialize");
        // B 被跳过；A、C 纳入。
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].props.get("title").unwrap(), &Value::Text("A".into()));
        assert_eq!(out[1].props.get("title").unwrap(), &Value::Text("C".into()));
        // 纳入项的 visible_when 已清除（已决策为纳入）。
        assert!(out[0].control.visible_when.is_none());
    }

    #[test]
    fn enabled_when_resolved_to_literal_per_item() {
        let mut spec = tab_list_spec("tab");
        spec.children[0].control.enabled_when = Some(CompactString::new("$tab.active"));
        spec.children[0].bindings.push(Binding {
            target: CompactString::new("title"),
            source: CompactString::new("$tab.title"),
        });
        let ctx = EvalContext::default().with_var(
            "state",
            Value::Object(
                [(
                    "tabs".to_string(),
                    Value::Array(vec![
                        Value::Object(
                            [
                                ("title".to_string(), Value::Text("A".into())),
                                ("active".to_string(), Value::Bool(true)),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                        Value::Object(
                            [
                                ("title".to_string(), Value::Text("B".into())),
                                ("active".to_string(), Value::Bool(false)),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                    ]),
                )]
                .into_iter()
                .collect(),
            ),
        );
        let out = materialize_for_each(&spec, &eng(), &ctx).expect("materialize");
        assert_eq!(out.len(), 2);
        // 解析为字面布尔字符串，下游 host 无需 item 作用域即可求值。
        assert_eq!(out[0].control.enabled_when.as_ref().unwrap().as_str(), "true");
        assert_eq!(out[1].control.enabled_when.as_ref().unwrap().as_str(), "false");
    }

    #[test]
    fn source_null_yields_empty() {
        // 路径缺失 → Null → 空列表（不报错）。
        let ctx = EvalContext::default();
        let out = materialize_for_each(&tab_list_spec("tab"), &eng(), &ctx).expect("materialize");
        assert!(out.is_empty());
    }

    #[test]
    fn source_non_array_is_typecheck_error() {
        let ctx = EvalContext::default().with_var(
            "state",
            Value::Object([("tabs".to_string(), Value::Int(3))].into_iter().collect()),
        );
        let err = materialize_for_each(&tab_list_spec("tab"), &eng(), &ctx).unwrap_err();
        assert!(matches!(err, DslError::Typecheck(_)), "got {err:?}");
    }

    #[test]
    fn iteration_limit_enforced() {
        // max_iterations = 2，数组 3 项 → 第 3 项触发 EvalResourceLimit。
        let ctx = EvalContext {
            max_iterations: 2,
            ..tabs_ctx()
        };
        let err = materialize_for_each(&tab_list_spec("tab"), &eng(), &ctx).unwrap_err();
        assert!(matches!(err, DslError::EvalResourceLimit(_)), "got {err:?}");
    }

    #[test]
    fn missing_for_each_is_error() {
        let spec = WidgetSpec::new("Row");
        let err = materialize_for_each(&spec, &eng(), &tabs_ctx()).unwrap_err();
        assert!(matches!(err, DslError::Validate(_)), "got {err:?}");
    }

    #[test]
    fn missing_template_is_error() {
        let mut spec = WidgetSpec::new("TabList");
        spec.control = ControlDirectives {
            for_each: Some(ForEachSpec {
                source: CompactString::new("$state.tabs"),
                item_alias: CompactString::new("tab"),
            }),
            ..Default::default()
        };
        let err = materialize_for_each(&spec, &eng(), &tabs_ctx()).unwrap_err();
        assert!(matches!(err, DslError::Validate(_)), "got {err:?}");
    }

    #[test]
    fn stable_id_falls_back_to_component_name_without_template_id() {
        let mut spec = tab_list_spec("tab");
        spec.children[0].id = None; // 模板无 id
        let out = materialize_for_each(&spec, &eng(), &tabs_ctx()).expect("materialize");
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].id.as_ref().unwrap().0.as_str(), "Tab@0");
        assert_eq!(out[2].id.as_ref().unwrap().0.as_str(), "Tab@2");
    }

    #[test]
    fn end_to_end_dsl_yaml_loader_to_materialize() {
        // 端到端：YAML → WidgetSpec → materialize_for_each，验证 spec §8.4.7 列表渲染闭环。
        use crate::loader::{WidgetSpecLoader, YamlLoader};
        let yaml = r#"
component: TabList
control:
  for_each:
    source: $state.tabs
    item_alias: tab
children:
  - component: Tab
    id: tab
    bindings:
      - target: title
        source: $tab.title
"#;
        let spec = YamlLoader::new().load_str(yaml).expect("load");
        let out = materialize_for_each(&spec, &eng(), &tabs_ctx()).expect("materialize");
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].props.get("title").unwrap(), &Value::Text("A".into()));
        assert_eq!(out[1].props.get("title").unwrap(), &Value::Text("B".into()));
        assert_eq!(out[0].id.as_ref().unwrap().0.as_str(), "tab@0");
    }
}
