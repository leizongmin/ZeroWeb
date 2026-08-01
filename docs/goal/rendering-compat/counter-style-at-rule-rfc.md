# RFC：`@counter-style` 自定义计数器样式 at-rule（CSS Counter Styles 3）

**日期**: 2026-08-01
**状态**: Design；slice 1 ✅ landed（R2392）；**slice 2（additive/extends/range）应用经 R2394 A/B 量证 net-negative → 应用 defer，parse-retain landed**（见 `evidence/counter-style-slice2-ab-2026-08-01.md`）
**承接**: R2278（css-counter-styles 73.6%，`@counter-style` at-rule 完全未实现）/ R2390（自主 clean-lever 穷尽，feature 为唯一剩余自主面）
**WPT footprint**: `css/css-counter-styles/counter-style-at-rule/`（~40 reftest）+ 部分 Latin-symbol 预定义样式

## 动机

CSS Counter Styles 3 §3 `@counter-style` at-rule 允许作者定义自定义 `list-style-type` 计数器样式（符号系统、前缀/后缀等）。ZW 当前完全未实现 → `list-style-type: <自定义名>` 被当未知值丢 → `<ol>` 回退 decimal。css-counter-styles 73.6% 残余主因之一（另一为 CJK/Indic 预定义样式需 script 字体 = font-wall gated，本 RFC 不覆盖）。

**真 spec gap**（非推测性，有 driving WPT）— 同 R2279（CSS Values L4 数学）谱系，可自主推进。

## 范围（最小可工作切片）

支持自定义计数器样式，使 `list-style-type: <name>` 经 `@counter-style <name> {…}` 定义后正确生成 marker。

### 实施（slice 1）
- **descriptors**：`system`、`symbols`、`prefix`、`suffix`、`fallback`、`negative`、`pad`、`range`（parse 全保留，应用按系统选择性消费）。
- **systems**：`cyclic`、`fixed`、`symbolic`、`alphabetic`、`numeric`（5 系统；位置/重复算法）。
- **marker 渲染**：复用既有 list-marker 文本渲染路径（`text_list.rs` Decimal/Roman 分支同款字符串→text）。

### Deferred（slice 2+，记 follow-up 不阻 slice 1）

> **R2394 A/B 结论**：`additive` / `extends` / `range` 的**应用**（渲染）经 scoped reftest A/B 量证 **net-negative**（driving WPT 案全 font-wall dice/triangle 字形 + system-additive ref 依赖 `document.write` JS + system-extends nbsp/marker 渲染差 + empty-string mismatch 阈值）。**parse-retain 已 land**（`additive_symbols`/`range` 字段 + 解析 + 单测），**应用 defer** 至字体栈补字形 / ref 不依赖 JS。算法实现（additive 贪心 / extends resolve / builtin 表）在 git 历史（R2394 工作树）可复活。

- `additive` 系统（Roman 式加法表，复杂；内置 Roman 已有独立实现）。— **应用 defer（R2394 net-negative）；parse-retain landed**
- `extends`/`extends <builtin>` system（依赖 builtin 解析）。— **应用 defer（R2394 net-negative）；Extends 变体 + parse landed**
- `range`/`fallback`/`pad`/`negative` 完整应用语义（slice 1 parse 保留，应用简化）。— `range` parse-retain landed（R2394），应用 defer
- descriptor 内 `calc()`（descriptor-calc WPT 案）。
- CJK/Arabic/Hebrew/Japanese 等**预定义**样式（需 script 字体 = font-wall gated，独立于 at-rule）。

## 架构与关键决策

### 1. 解析（css-parser）
新增 at-rule 分支（`consume_at_rule` 按 `@counter-style` 名分发，镜像 `@keyframes`/`@layer` 模式）：
- prelude = `<ident>`（counter style 名）。
- body = declaration block（descriptors），用既有 `consume_style_block` 类似机制解析为 `HashMap<String, String>`（descriptor → value）。
- 产物：`Rule::CounterStyle { name: String, descriptors: HashMap<String,String> }`（新增 Rule 变体）。

### 2. 存储与 threading
- 收集：stylesheet 解析期把所有 `Rule::CounterStyle` 聚合为 `HashMap<String, CounterStyleDefinition>`。
- **threading 模式 = @keyframes registration**（pipeline.rs:305 在样式计算后「注册 @keyframes 到动画时钟」）：同点注册 `CounterStyleRegistry` 到 engine/painter 上下文。
- `CounterStyleDefinition { system, symbols: Vec<String>, prefix, suffix, fallback, … }`（预解析 system enum + symbols 切分）。

### 3. `list-style-type` 自定义值
- `ListStyleTypeValue` 加 `Custom(String)` 变体（**或** ComputedStyle 加并行字段 `list_style_type_name: Option<String>` — 决策点，倾向 Custom 变体保单字段）。
- apply.rs：`list-style-type: <name>` 若非 builtin → `Custom(name)`（不丢，不回退 decimal）。

### 4. marker 生成（engine `paint_list_marker`）
`text_list.rs:141` match 加 `Custom(name)` arm：
- 查 `CounterStyleRegistry`（经 painter 上下文）；未命中 → fallback（默认 decimal）。
- 按 system 生成 marker 字符串：
  - `cyclic`: `symbols[(i-1) % len]`
  - `fixed`: `symbols[i-1]`（i ≤ len，否则 fallback）
  - `symbolic`: `symbols[(i-1) % len]` × i（如 `*`/`**`/`***`）
  - `alphabetic`: 位置制 (i-1) base-len（如 a-z/aa-zz）
  - `numeric`: 位置制含 0-symbol（如 0-9/00-99）
- 拼 `prefix + marker + suffix`，复用既有 Decimal 分支的文本渲染（font/位置）。

## 算法正确性要点（CSS Counter Styles 3 §3）
- `symbols` 长度限制：`alphabetic`/`numeric` 须 ≥1（alphabetic 须 ≥2 才能表示大数，但 spec 允许 ≥1）；不足按 spec 错误处理（at-rule 无效 → 丢）。
- `symbolic` 大 i 重复（`"*".repeat(i)`）。
- index 1-based（list-item counter 从 1 起）。
- `fixed` 系统 `first-symbol-value` 默认 1（`system: fixed N?` N 可调起始）。

## 门禁 / A/B
- kill-switch `ZW_COUNTER_STYLE=0`（default-on；关闭则 `@counter-style` 不解析 = 旧行为，零回归回退）。
- TDD：css-parser（@counter-style 解析 + descriptors + Custom list-style-type）+ engine（5 系统 marker 生成单测，i=1..N 覆盖循环/进位/边界）。
- scoped reftest A/B：`css-counter-styles/counter-style-at-rule/` self-source，`ZW_COUNTER_STYLE=0` baseline vs default-on，net≥0 才 land。
- 全量 `make test` + `cargo clippy --workspace --all-targets -D warnings` + `cargo fmt`。
- product-smoke / product-smoke-legacy（list marker 变更可能影响产品 fixture）。

## 风险
- **registry threading blast radius**：painter 上下文加 CounterStyleRegistry 须经 engine 管线多函数签名传递 — 与 @keyframes 同模式但触及 painter 调用链（moderate，A/B 守）。
- **ListStyleTypeValue::Custom 变体**：exhaustive match 多 site 须加 arm（与 R2215 NthChildOf 新变体同 surgical 法，零既有调用点变更）。
- **system 算法边界**：alphabetic/numeric 进位易错，须单测覆盖 i=len, i=len+1, i=len²。

## 估时
~250-350 行跨 css-parser（at-rule + Rule 变体 + descriptors）/ style-system（Custom 变体 + apply）/ engine（registry threading + 5 系统 marker 生成）。建议拆 2 提交：① parse + registry + Custom + cyclic/fixed/symbolic（最小可 land）；② alphabetic/numeric + descriptor 应用完善。

## 下一轮入口
按本 RFC slice 1 实施（cyclic/fixed/symbolic + parse/registry/threading），TDD red→green + scoped reftest A/B net≥0 + 全量门禁。
