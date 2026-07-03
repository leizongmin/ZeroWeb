# zero-ui-dsl

通用 UI SDK 的声明式 DSL 层。把 YAML 文件解析为 `WidgetSpec` 声明树，并用受控表达式语言实现属性绑定、条件渲染、列表渲染等能力。

## 架构位置

```
ui/dsl ←── YAML 输入 → WidgetSpec → ui/runtime（WidgetHost 消费）
     │
     ├── ui/core（WidgetSpec / ActionId / Binding 等基础类型）
     ├── ui/i18n（`i18n:` 对象桥接）
     └── ui/assets（`asset:` 对象桥接）
```

## 模块

| 模块 | 职责 |
|------|------|
| `yaml.rs` | **受限 YAML 子集解析器**（仓内自实现，零新依赖；spec §8.4.7 倾向）。支持块映射/块序列/嵌套/流集合/注释/类型推断；不支持锚点/多文档/Tab。深度守卫 MAX_YAML_DEPTH=100。UTF-8 保真。 |
| `engine.rs` | **表达式引擎**（Pratt parser + typecheck + eval）。四阶段管线：parse → validate → typecheck → eval。字面量 / `$path` / 算术·比较·布尔·空值合并 / 条件 `?:` / 纯函数 / sandbox 防御。 |
| `expression.rs` | `Expression` 枚举定义（文法封闭，无 Lambda 变体） |
| `loader.rs` | `YamlLoader`（impl `WidgetSpecLoader`）：YAML→`WidgetSpec` 递归；strict 模式预校验表达式语法；responsive branch 收敛 |
| `for_each.rs` | `materialize_for_each`：列表渲染（item 作用域求值 bindings + visible/enabled + 稳定 id `id@idx`） |
| `diagnostics.rs` | DSL 错误诊断（`DslError`） |
| `i18n_bridge.rs` | `i18n_value_to_message`：把保留的 `i18n:` 对象→`LocalizedText`，参数表达式求值 |
| `asset_bridge.rs` | `asset_id_of` / `is_asset_object`：DSL `asset:` 引用→`AssetId` |

## 表达式能力

### 字面量与路径
- 整数/浮点/布尔/字符串/Null/数组/Object
- `$path` → `EvalContext.vars` 读取（如 `$tab.title`）
- 路径支持嵌套 + 空值合并 `??`

### 运算符
算术（+ - * / %）· 比较（== != < > <= >=）· 布尔（&& || !）· 空值合并（`??` 比 `||` 松，C# 语义）· 条件（`cond ? a : b` 右结合）· 一元负号（`checked_neg` + Float 兜底防 i64::MIN panic）

### 纯函数
`count` / `contains` / `any` / `all` / `min` / `max` / `clamp` / `concat` / `starts_with` / `ends_with` / `format` / `map($items, field)` / `filter($items, field)` / `field(obj, path)`

### Sandbox（约束 #7）
- 禁止：递归/无限循环/文件·网络·进程·时钟·随机数访问→`ForbiddenCapability`
- 未注册函数→`UnknownFunction`；`allow_functions=false`全禁
- 资源上限：max_nodes=1024, max_depth=64, max_iterations=10000, parse-depth=64

### Action 简写
- `command: <id>` → action=命令 id
- `navigate: <route>` → `nav.push` + payload
- `open_overlay: <id>` / `close_overlay: <id>`

## 依赖

- `zero-ui-core` / `zero-ui-i18n` / `zero-ui-assets`
- `compact_str` / `serde` / `thiserror` / `hashbrown`
- 零浏览器业务 crate 依赖（DC-1 cargo tree 机械验证）

## 测试

- `cargo test -p zero-ui-dsl` — 95 测
- 覆盖：表达式 parse/typecheck/eval/sandbox/资源上限/YAML 解析+loader/for_each 列表渲染/responsive branch/command·route·overlay·asset 简写/i18n+asset 桥接

## 深度审查

2026-07-03 全 crate 三轮深度审查（engine + yaml + loader），共修 5 bug：
- **engine**：unary neg i64::MIN panic → `checked_neg` + Float 兜底
- **yaml**：嵌套无守卫→栈溢出（+MAX_DEPTH=100 + expand_dash 迭代）；strip_comment + parse_quoted_scalar UTF-8 损坏→改切片/char 迭代
- **loader**：visible_when/enabled_when 非文本值静默忽略→ok_or_else 报错

约束 #7 经深度审查闭合；引擎对任意 host 状态值健壮（不 panic）。
