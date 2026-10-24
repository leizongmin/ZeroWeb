# M2 切片 2：`:has()` 全量回退计数（DC-2）+ Shadow/lang/根度量失效序列固化

**运行身份**：基线 HEAD `834e90d59`（上轮 M2 切片 1），本轮改动 `crates/style-system/src/lib.rs`（回退计数 instrumentation）与 `crates/style-system/src/tests/incremental_oracle.rs`（计数断言 + 3 新序列）。
**环境/守卫**：`./target/test-guard --compile-first … cargo test -p zero-style-system`；fmt、`make guarded-clippy`、`make test` 全量、定向 bench-gate（见提交说明）。

## 内容

### 1. `:has()` 全量回退计数（DC-2"全量回退有计数"）

`compute_styles_incremental` 开头的 `stylesheet_has_has_selector` 回退分支（样式表含 `:has()` 时按契约退化 `compute_styles` 全量）原先是静默的——oracle 只能验证"结果一致"，无法区分"真增量"与"未声明的全量"。

instrumentation：`StyleSystem` 实例字段 `incremental_has_fallbacks: u64` + getter `incremental_has_fallback_count()`，回退分支递增。**选实例字段而非进程级 static**（`SCROLL_PSEUDO_PRESENT` AtomicBool 先例）：Rust 测试同进程并行，进程级计数会被其它测试推高，只能做下界断言；实例字段隔离，oracle 可精确等值断言。递增点在慢分支（页含 `:has` 才走），非热循环。

oracle 断言（`inc02_has_selector_fallback_matches_full` 扩展）：首帧全量 = 0 → 移除 `:has` 规则后的真增量步 = 0 → 恢复 `:has` 规则的步 = 1。钉死两件事：回退只在 `:has` 在页时发生；回退确实发生（而非静默走错路径）。

### 2. Shadow DOM 序列（内部等价基线）

`attach_shadow` 把 ShadowRoot 节点挂进 host 的 children 且 parent 指回 host（crates/dom/src/document/shadow.rs:19），因此全量 DFS 下探 shadow 子树；增量链走自 shadow 内元素向上**穿过 ShadowRoot 节点**（非 Element，链走跳过）达 host。三步序列（`inc02_shadow_tree_mutations_consistent`）：shadow 内元素 class 变更（链走穿出 shadow 树）→ host 属性变更（变更子树递归下探 shadow）→ CSS 同帧改 shadow 内规则色值（重算不沿用旧值）。全部 Equal。

**基线性质**：断言的是两路径对同一 Document 的遍历语义一致（内部等价），不是 shadow 封装/插槽分配的规范正确性——后者属 rendering-compat 范围。style-system 本身无 shadow/slot 概念（源码核查：仅 scroll-button slot u8 同名无关），两路径共享的 DFS 语义即当前唯一不变量。

### 3. lang → `<q>` 自动引号传播

R2856：`quotes: auto` 按父元素有效内容语言解析引号对；`effective_lang` 现读 Document、无 StyleSystem 实例状态，故语义上只需重算即可得新值——风险在增量路径是否真的重算 q。序列（`inc02_lang_change_propagates_to_quotes`）：`html lang` en→ja（changed=[html]，链走 + 变更子树递归覆盖 q，引号 " " → 「 」）→ lang 置空（空 lang = 无语言，回落英语 root）。两步 Equal。

### 4. 根字号度量刷新时序

根度量（`root_x_height` 等）是 **StyleSystem 实例状态**，与 lang 不同：全量入口先 reset（lib.rs compute_styles 开头）、根元素（parent_style=None）计算时回填；增量路径不 reset，靠链走重算 html 回填新值，且链走先于变更子树递归——时序错了 p 的根相对单位就吃旧度量。

序列（`inc02_root_metric_units_refresh_on_html_font_change`）：`html { font-size: 16→24px }`，changed=[html]，`p { line-height: 10rex }` 按根度量解析。Equal ⇒ 度量刷新时序正确。

**测量中发现**：`rem` 恒按常量 `ROOT_FONT_SIZE` 解析（computed.rs:172），不随 `html { font-size }` 变化——设计序列时先假设 rem 随根字号，源码核查否决后改用 `rex`（computed.rs:681 走 `root_metrics`）。rem 的规范正确性疑点（html font-size 应影响 rem？）属 rendering-compat，本对照只记录事实。

## 验证

| 检查 | 结果 |
|---|---|
| oracle 套件 | **12/12**（9 旧 + 计数断言扩展 + 3 新序列），计数精确断言 0→0→1 |
| style-system 全量 | 2448/2448 PASS（+3 测试） |
| fmt / guarded-clippy / make test | 见提交说明 |
| 定向 bench-gate | `ZERO_WEB_BENCH_CRATES=zero-style-system`（见提交说明；递增点在 `:has` 慢分支，无 `:has` 页零开销） |

## 边界与未覆盖

- 反例 A（changed 集完备性契约）保持开放，不变。
- slot 分配（`assign_slot`/`resolve_slots`）驱动的样式失效未固化——style-system 两路径都不感知 slot，序列无从差异；等 engine 层 flat-tree 切片再入账。
- `@font-face` 到达类字体失效序列未做：本轮用根字号度量序列覆盖"字体相关实例状态刷新"轴；`@font-face` 加载事件在生产走 engine 侧失效路由，style 层无独立入口，归入 M3 生产帧验证。
- 布局几何快照（M1 剩余）、帧复用/缓存生命周期（M3）、检查成本（M4）未动。
