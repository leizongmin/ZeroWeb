//! WidgetSpec 加载器（spec IF-005 `WidgetSpecLoader` / `ExpressionEngine` / FR-008 / DC-6）。
//!
//! [`YamlLoader`] 把受限 YAML（[`crate::yaml`]）解析为中间 [`YamlValue`]，再递归转换为强类型
//! [`WidgetSpec`]。表达式承载字段（`visible_when`/`enabled_when`/`for_each.source`/`binding.source`）
//! 在 `ui/core` 侧以**原文 `CompactString`** 存储（保持 `ui/core` 不依赖 `ui/dsl`，依赖方向 dsl → core）；
//! strict 模式下加载时调 [`Engine::parse`] 提前捕获表达式语法错误（spec §8.4.7 Validate 阶段）。
//! `i18n:` 对象以 [`Value::Object`] 原样保留，运行期由 i18n 解析层处理（DC-10 / DC-6 后续）。

use crate::diagnostics::DslError;
use crate::engine::Engine;
use crate::expression::Expression;
use crate::yaml::{self, YamlValue};
use compact_str::CompactString;
use hashbrown::HashMap;
use zero_ui_core::action::{ActionBinding, ActionId, ActionPayload};
use zero_ui_core::binding::{Binding, PropsMap, Value};
use zero_ui_core::widget::{ComponentType, ControlDirectives, ForEachSpec, WidgetId, WidgetSpec};

/// YAML/字符串 → WidgetSpec 加载器（spec IF-005）。
pub trait WidgetSpecLoader {
    fn load_str(&self, source: &str) -> Result<WidgetSpec, DslError>;
}

/// 表达式引擎（spec IF-005 `ExpressionEngine`）。
pub trait ExpressionEngine {
    fn parse(&self, source: &str) -> Result<Expression, DslError>;
    fn typecheck(
        &self,
        expr: &Expression,
        schema: &zero_ui_core::binding::BindingSchema,
    ) -> Result<zero_ui_core::binding::ValueType, DslError>;
    fn eval(&self, expr: &Expression, ctx: &EvalContext) -> Result<Value, DslError>;
}

/// 表达式求值上下文（持有状态快照 + 权限边界，spec 约束 7）。
///
/// `vars` 提供路径根（`$state`/`$props`/`$theme`/`$env` 等）的值快照；
/// 求值无副作用、确定性、可缓存。
#[derive(Debug, Clone)]
pub struct EvalContext {
    /// 是否允许调用纯函数（沙箱可关闭：false 时所有 Call → ForbiddenCapability）。
    pub allow_functions: bool,
    /// AST 最大节点数（防恶意 DSL 消耗 CPU，spec FR-008）。
    pub max_nodes: usize,
    /// 求值最大递归深度（防深度嵌套 AST 栈溢出）。
    pub max_depth: usize,
    /// 集合函数最大累计迭代数（防超长数组消耗 CPU）。
    pub max_iterations: usize,
    /// 路径根 → 值快照（如 `"state" → Value::Object{...}`）。
    pub vars: HashMap<String, Value>,
}

impl Default for EvalContext {
    fn default() -> EvalContext {
        EvalContext {
            allow_functions: true,
            max_nodes: 1024,
            max_depth: 64,
            max_iterations: 10_000,
            vars: HashMap::new(),
        }
    }
}

impl EvalContext {
    /// 注入一个路径根的值快照（builder 风格）。
    pub fn with_var(mut self, root: &str, value: Value) -> EvalContext {
        self.vars.insert(root.to_string(), value);
        self
    }
}

/// YAML → WidgetSpec 加载器（spec IF-005 / DC-6）。
///
/// `strict = true`（默认）时，加载阶段即校验所有表达式字符串可被 [`Engine`] parse，
/// 提前暴露 `visible_when`/`enabled_when`/`for_each.source`/`binding.source` 的语法错误。
pub struct YamlLoader {
    engine: Engine,
    strict: bool,
}

impl Default for YamlLoader {
    fn default() -> YamlLoader {
        YamlLoader {
            engine: Engine,
            strict: true,
        }
    }
}

impl YamlLoader {
    /// 创建 strict 加载器（默认；加载时校验表达式语法）。
    pub fn new() -> YamlLoader {
        YamlLoader::default()
    }

    /// 创建 lenient 加载器（不校验表达式语法；适合已知安全的静态资源或调试）。
    pub fn lenient() -> YamlLoader {
        YamlLoader {
            engine: Engine,
            strict: false,
        }
    }

    /// 校验单条表达式字符串（strict 模式下被加载器调用）。
    fn validate_expr(&self, src: &str) -> Result<(), DslError> {
        if self.strict {
            self.engine
                .parse(src)
                .map(|_| ())
                .map_err(|e| DslError::Validate(format!("表达式语法错误 ({src}): {e}")))?;
        }
        Ok(())
    }

    /// 把节点转换为 [`WidgetSpec`]（节点须为映射）。
    fn convert_spec(&self, node: &YamlValue) -> Result<WidgetSpec, DslError> {
        let entries = node
            .as_map()
            .ok_or_else(|| DslError::Parse("组件节点必须是映射".into()))?;

        // component（必需）。
        let component = entries
            .iter()
            .find(|(k, _)| k == "component")
            .and_then(|(_, v)| v.as_text())
            .ok_or_else(|| DslError::Parse("WidgetSpec 缺少 'component' 字段".into()))?;
        let mut spec = WidgetSpec::new(component);
        spec.component = ComponentType::new(component);

        for (key, val) in entries {
            match key.as_str() {
                "component" => {}
                "id" => {
                    spec.id = Some(
                        val.as_text()
                            .map(WidgetId::new)
                            .ok_or_else(|| DslError::Parse("'id' 必须是文本".into()))?,
                    );
                }
                "props" => {
                    spec.props = self.convert_props(val)?;
                }
                "bindings" => {
                    spec.bindings = self.convert_bindings(val)?;
                }
                "actions" => {
                    spec.actions = self.convert_actions(val)?;
                }
                "control" => {
                    spec.control = self.convert_control(val)?;
                }
                "children" => {
                    spec.children = self.convert_children(val)?;
                }
                // 未知顶层键忽略（前向兼容；spec 未要求严格拒绝）。
                _ => {}
            }
        }
        Ok(spec)
    }

    fn convert_props(&self, val: &YamlValue) -> Result<PropsMap, DslError> {
        let entries = val
            .as_map()
            .ok_or_else(|| DslError::Parse("'props' 必须是映射".into()))?;
        let mut props = PropsMap::new();
        for (k, v) in entries {
            props.insert(k, yaml_to_value(v));
        }
        Ok(props)
    }

    fn convert_bindings(&self, val: &YamlValue) -> Result<Vec<Binding>, DslError> {
        let seq = val
            .as_seq()
            .ok_or_else(|| DslError::Parse("'bindings' 必须是序列".into()))?;
        let mut out = Vec::with_capacity(seq.len());
        for item in seq {
            let map = item
                .as_map()
                .ok_or_else(|| DslError::Parse("binding 项必须是映射".into()))?;
            let target = map
                .iter()
                .find(|(k, _)| k == "target")
                .and_then(|(_, v)| v.as_text())
                .ok_or_else(|| DslError::Parse("binding 缺少 'target'".into()))?;
            let source = map
                .iter()
                .find(|(k, _)| k == "source")
                .and_then(|(_, v)| v.as_text())
                .ok_or_else(|| DslError::Parse("binding 缺少 'source'".into()))?;
            self.validate_expr(source)?;
            out.push(Binding {
                target: CompactString::new(target),
                source: CompactString::new(source),
            });
        }
        Ok(out)
    }

    fn convert_actions(&self, val: &YamlValue) -> Result<Vec<ActionBinding>, DslError> {
        let seq = val
            .as_seq()
            .ok_or_else(|| DslError::Parse("'actions' 必须是序列".into()))?;
        let mut out = Vec::with_capacity(seq.len());
        for item in seq {
            let map = item
                .as_map()
                .ok_or_else(|| DslError::Parse("action 项必须是映射".into()))?;
            let trigger = map
                .iter()
                .find(|(k, _)| k == "trigger")
                .and_then(|(_, v)| v.as_text())
                .ok_or_else(|| DslError::Parse("action 缺少 'trigger'".into()))?;
            let action = map
                .iter()
                .find(|(k, _)| k == "action")
                .and_then(|(_, v)| v.as_text())
                .ok_or_else(|| DslError::Parse("action 缺少 'action'".into()))?;
            let payload = map
                .iter()
                .find(|(k, _)| k == "payload")
                .map(|(_, v)| yaml_to_payload(v));
            out.push(ActionBinding {
                trigger: CompactString::new(trigger),
                action: ActionId::new(action),
                payload,
            });
        }
        Ok(out)
    }

    fn convert_control(&self, val: &YamlValue) -> Result<ControlDirectives, DslError> {
        let map = val
            .as_map()
            .ok_or_else(|| DslError::Parse("'control' 必须是映射".into()))?;
        let mut cd = ControlDirectives::default();
        for (k, v) in map {
            match k.as_str() {
                "visible_when" => {
                    if let Some(s) = v.as_text() {
                        self.validate_expr(s)?;
                        cd.visible_when = Some(CompactString::new(s));
                    }
                }
                "enabled_when" => {
                    if let Some(s) = v.as_text() {
                        self.validate_expr(s)?;
                        cd.enabled_when = Some(CompactString::new(s));
                    }
                }
                "for_each" => {
                    cd.for_each = Some(self.convert_for_each(v)?);
                }
                _ => {}
            }
        }
        Ok(cd)
    }

    fn convert_for_each(&self, val: &YamlValue) -> Result<ForEachSpec, DslError> {
        let map = val
            .as_map()
            .ok_or_else(|| DslError::Parse("'for_each' 必须是映射".into()))?;
        let source = map
            .iter()
            .find(|(k, _)| k == "source")
            .and_then(|(_, v)| v.as_text())
            .ok_or_else(|| DslError::Parse("for_each 缺少 'source'".into()))?;
        self.validate_expr(source)?;
        let item_alias = map
            .iter()
            .find(|(k, _)| k == "item_alias")
            .and_then(|(_, v)| v.as_text())
            .unwrap_or("item");
        Ok(ForEachSpec {
            source: CompactString::new(source),
            item_alias: CompactString::new(item_alias),
        })
    }

    fn convert_children(&self, val: &YamlValue) -> Result<Vec<WidgetSpec>, DslError> {
        let seq = val
            .as_seq()
            .ok_or_else(|| DslError::Parse("'children' 必须是序列".into()))?;
        seq.iter().map(|c| self.convert_spec(c)).collect()
    }
}

impl WidgetSpecLoader for YamlLoader {
    fn load_str(&self, source: &str) -> Result<WidgetSpec, DslError> {
        let root = yaml::parse(source)?;
        if matches!(root, YamlValue::Null) {
            return Err(DslError::Parse("YAML 为空".into()));
        }
        self.convert_spec(&root)
    }
}

/// [`YamlValue`] → [`Value`]（props / payload 用）。
fn yaml_to_value(v: &YamlValue) -> Value {
    match v {
        YamlValue::Null => Value::Null,
        YamlValue::Bool(b) => Value::Bool(*b),
        YamlValue::Int(i) => Value::Int(*i),
        YamlValue::Float(f) => Value::Float(*f),
        YamlValue::Text(s) => Value::Text(s.clone()),
        YamlValue::Seq(items) => Value::Array(items.iter().map(yaml_to_value).collect()),
        YamlValue::Map(entries) => {
            let mut obj = HashMap::with_capacity(entries.len());
            for (k, vv) in entries {
                obj.insert(k.clone(), yaml_to_value(vv));
            }
            Value::Object(obj)
        }
    }
}

/// [`YamlValue`] → [`ActionPayload`]（标量优先映射，复合用 [`ActionPayload::Value`]）。
fn yaml_to_payload(v: &YamlValue) -> ActionPayload {
    match v {
        YamlValue::Null => ActionPayload::Unit,
        YamlValue::Bool(b) => ActionPayload::Bool(*b),
        YamlValue::Int(i) => ActionPayload::Int(*i),
        YamlValue::Float(f) => ActionPayload::Float(*f),
        YamlValue::Text(s) => ActionPayload::Text(s.clone()),
        YamlValue::Seq(_) | YamlValue::Map(_) => ActionPayload::Value(yaml_to_value(v)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::binding::ValueType;

    fn load(src: &str) -> WidgetSpec {
        YamlLoader::new()
            .load_str(src)
            .unwrap_or_else(|e| panic!("load failed: {e:?}\n--- source ---\n{src}"))
    }

    // ── 基本结构 ──────────────────────────────────────────────────────
    #[test]
    fn loads_minimal_component() {
        let spec = load("component: Button");
        assert_eq!(spec.component, ComponentType::new("Button"));
        assert!(spec.id.is_none());
        assert!(spec.children.is_empty());
    }

    #[test]
    fn loads_id_and_props() {
        let spec = load("component: Button\nid: confirm_btn\nprops:\n  label: OK\n  count: 3\n  enabled: true\n");
        assert_eq!(spec.id, Some(WidgetId::new("confirm_btn")));
        assert_eq!(spec.props.get("label"), Some(&Value::Text("OK".into())));
        assert_eq!(spec.props.get("count"), Some(&Value::Int(3)));
        assert_eq!(spec.props.get("enabled"), Some(&Value::Bool(true)));
    }

    #[test]
    fn loads_nested_children() {
        let spec = load(
            "component: Column\nchildren:\n  - component: Text\n    props:\n      text: hi\n  - component: Button\n",
        );
        assert_eq!(spec.children.len(), 2);
        assert_eq!(spec.children[0].component, ComponentType::new("Text"));
        assert_eq!(spec.children[0].props.get("text"), Some(&Value::Text("hi".into())));
        assert_eq!(spec.children[1].component, ComponentType::new("Button"));
    }

    // ── bindings / actions / control ─────────────────────────────────
    #[test]
    fn loads_bindings_and_actions() {
        let spec = load(
            "component: TextInput\nbindings:\n  - target: text\n    source: $state.query\nactions:\n  - trigger: submit\n    action: app.search\n    payload: go\n",
        );
        assert_eq!(spec.bindings.len(), 1);
        assert_eq!(spec.bindings[0].target.as_str(), "text");
        assert_eq!(spec.bindings[0].source.as_str(), "$state.query");
        assert_eq!(spec.actions.len(), 1);
        assert_eq!(spec.actions[0].trigger.as_str(), "submit");
        assert_eq!(spec.actions[0].action, ActionId::new("app.search"));
        assert_eq!(spec.actions[0].payload, Some(ActionPayload::Text("go".into())));
    }

    #[test]
    fn loads_control_directives() {
        let spec = load(
            "component: Button\ncontrol:\n  visible_when: $state.tabs.count > 0\n  enabled_when: $state.can_go_back\n  for_each:\n    source: $state.items\n    item_alias: tab\n",
        );
        assert_eq!(spec.control.visible_when.as_deref(), Some("$state.tabs.count > 0"));
        assert_eq!(spec.control.enabled_when.as_deref(), Some("$state.can_go_back"));
        let fe = spec.control.for_each.expect("for_each");
        assert_eq!(fe.source.as_str(), "$state.items");
        assert_eq!(fe.item_alias.as_str(), "tab");
    }

    #[test]
    fn for_each_default_item_alias() {
        let spec = load("component: Row\ncontrol:\n  for_each:\n    source: $state.items\n");
        assert_eq!(spec.control.for_each.unwrap().item_alias.as_str(), "item");
    }

    // ── i18n 对象保留 ─────────────────────────────────────────────────
    #[test]
    fn i18n_prop_preserved_as_object() {
        let spec = load(
            "component: Text\nprops:\n  text:\n    i18n: browser.address.placeholder\n    params:\n      origin: $browser.active_origin\n",
        );
        let text = spec.props.get("text").expect("text prop");
        let obj = match text {
            Value::Object(o) => o,
            _ => panic!("expected object, got {text:?}"),
        };
        assert_eq!(
            obj.get("i18n"),
            Some(&Value::Text("browser.address.placeholder".into()))
        );
        assert!(obj.get("params").is_some());
    }

    // ── strict 表达式校验 ─────────────────────────────────────────────
    #[test]
    fn strict_rejects_bad_expression() {
        let loader = YamlLoader::new();
        let err = loader
            .load_str("component: Button\ncontrol:\n  visible_when: $state.tabs.count >")
            .unwrap_err();
        assert!(matches!(err, DslError::Validate(_)), "got {err:?}");
    }

    #[test]
    fn lenient_accepts_bad_expression() {
        let spec = YamlLoader::lenient()
            .load_str("component: Button\ncontrol:\n  visible_when: not valid expr (")
            .unwrap();
        // lenient 不校验，原文照存。
        assert_eq!(spec.control.visible_when.as_deref(), Some("not valid expr ("));
    }

    #[test]
    fn strict_validates_binding_source() {
        let loader = YamlLoader::new();
        let err = loader
            .load_str("component: TextInput\nbindings:\n  - target: text\n    source: '@@@ bad'")
            .unwrap_err();
        assert!(matches!(err, DslError::Validate(_)));
    }

    // ── 端到端：DSL 文本 → eval ───────────────────────────────────────
    #[test]
    fn end_to_end_load_then_eval() {
        // 加载后，把 visible_when 原文交给 Engine eval，验证整条 DSL 管线贯通。
        let spec = load("component: Button\ncontrol:\n  visible_when: $state.count > 0\n");
        let expr_src = spec.control.visible_when.unwrap();
        let engine = Engine;
        let expr = engine.parse(&expr_src).unwrap();
        let mut state = HashMap::new();
        state.insert("count".to_string(), Value::Int(5));
        let ctx = EvalContext::default().with_var("state", Value::Object(state));
        assert_eq!(engine.eval(&expr, &ctx).unwrap(), Value::Bool(true));
        // typecheck 也能跑通。
        let schema = zero_ui_core::binding::BindingSchema::default();
        assert_eq!(engine.typecheck(&expr, &schema).unwrap(), ValueType::Bool);
    }

    // ── 错误路径 ──────────────────────────────────────────────────────
    #[test]
    fn errors() {
        let loader = YamlLoader::new();
        assert!(loader.load_str("").is_err()); // 空
        assert!(loader.load_str("a: 1").is_err()); // 根无 component
        assert!(loader.load_str("component: 5").is_err()); // component 非文本
        assert!(loader.load_str("- a\n- b").is_err()); // 根是序列不是映射
    }
}
