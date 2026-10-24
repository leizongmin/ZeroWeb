# M1 第一切片：style 层确定性双路径快照与失效反例（INC-01/INC-02 style 面）

**运行身份**：基线 HEAD `332c9a9cc`，工作区含本轮改动（`crates/style-system/src/property/computed_style.rs`、`property/types.rs` 补 `PartialEq`；新增 `src/tests/incremental_oracle.rs`；`tests/mod.rs` 注册）。goal 启动门控于本轮解除（本次 Rally 运行即文档所述明确启动）。
**环境**：Linux x86_64（WSL2 6.18）；`make target/test-guard` 后 `./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-style-system incremental`；8/8 通过（0.01s）。
**隔离**：单进程单测，双路径各持独立 `StyleSystem` 实例（互不污染），同一 `Document`；无网络/字体/GPU 依赖。

## 入口与缓存盘点（M1 盘点产物，style/engine 面）

| 项 | 位置 | 说明 |
|---|---|---|
| 全量入口 | `crates/style-system/src/lib.rs:388` `compute_styles` | 文档 DFS；S11 样式键缓存 + S10 伪元素规则预扫描均为每调用局部 |
| 增量入口 | `crates/style-system/src/lib.rs:464` `compute_styles_incremental` | changed 每项：祖先链单节点重算（`compute_element_style_internal`，**不含伪元素**）+ 变更子树 `compute_styles_recursive`；样式表含 `:has()` 时整帧退化全量 |
| 生产消费者 1 | `crates/engine/src/pipeline/mod.rs:2334` `paint_only_incremental` | paint-only 属性白名单 mutation（`is_paint_only_property`，mod.rs:2304）→ 增量 style（base=`cached_styles`）+ 复用 `cached_layout` + 全量 paint；changed = mutation selector 目标 |
| 生产消费者 2 | `crates/engine/src/js_dom_bridge/computed_style_cache.rs` | getComputedStyle 代际缓存：drain_gen/style_version 前进时对 cached doc 幂等 replay + 变更子树增量 cascade（同样调 `compute_styles_incremental`） |
| 布局/帧缓存 | `pipeline/mod.rs:176/265/280` `cached_styles` / `cached_layout` / `cached_stylesheets` | paint-only 臂布局整体复用；布局失效粒度本轮未测（M1 布局切片） |
| 脏区 | `crates/engine/src/dirty.rs` `DirtyTracker` | rect 级，与样式失效正交 |

## 比较契约（浮点容差决策，M1 按契约固定）

style 层**精确相等，不设容差**：`ComputedStyle` 本轮补 `PartialEq`（全部字段为可观察 CSS 值；`CustomProperties` 为 `Arc<HashMap>`，比较按内容与插入顺序无关）——两路径同进程对同一输入运行同一计算代码，位级相同是应有语义，任何差异即真实分歧。几何/像素容差待布局切片按输出契约另行固定，不提前放宽。

## 序列与结果（8/8）

基座 DOM：`html(lang)>body>(div#card-a.card>(p#p1.text+p#p2.text+span#s1)+section#sec-b)`；每步同一 CSS 输入喂两条路径，逐步逐元素比对（含条目集合——残留条目也算分歧）。

| 序列（测试名） | 步骤 | 期望 | 结果 |
|---|---|---|---|
| inc01_class_and_ancestor_attr_changes_consistent | p1 class 追加 → 祖先增删 `data-x` | Equal | PASS |
| inc01_remove_last_declaration_consistent | `p` 规则逐条移除声明至规则清空 | Equal | PASS |
| inc01_move_subtree_consistent | span 移入/移出 `#sec-b`（继承切换） | Equal | PASS |
| inc02_inherited_font_change_consistent | `.card` 增 `font-size:20px`（后代继承） | Equal | PASS |
| inc02_has_selector_fallback_matches_full | `:has()` 规则移除/恢复（增量入口退化全量） | Equal | PASS |
| 首帧基线（所有 run 的 frame 0） | 双路径同为全量 | Equal | PASS |
| inc01_sibling_invalidation_gap_detected | `p:only-child`，q2 移出（changed 只含 q2） | **Divergent** | 检出 ✓（反例 A） |
| inc01_ancestor_pseudo_loss_on_chain_recompute_detected | 祖先 `body::before` + p1 inline style 变更 | **Divergent** | 检出 ✓（反例 B） |

## 失效反例（本轮核心产物）

**反例 A（`:only-child` 兄弟失效缺口）**：结构型伪类依赖兄弟存在性；`compute_styles_incremental` 只重算祖先链 + changed 子树，留存的兄弟不重算——全量判 `:only-child` 成立、增量保留旧值。**生产不可达**（`paint_only_incremental` 只传单目标 paint-only mutation，不改树结构；生产可达性再收窄需 mutation 路由变更）——属 API 契约缺口：调用方必须把受 restyle 影响的兄弟纳入 changed。

**反例 B（祖先链伪元素丢失）**：祖先链单节点重算走 `compute_element_style_internal`，不含 `compute_styles_recursive` 中的伪元素计算块——含 `::before/::after` 等规则的祖先在每次增量后被无伪样式覆盖。**生产增量臂可达**（消费者 1、2 均以祖先链重算恢复父样式；站点 CSS 含 `body::before` 等装饰/counter 用法时，paint-only mutation 或 gCS 增量 cascade 后伪样式丢失）。FIXME 已标于测试；修复（祖先链重算补伪元素计算）留待 M2 切片，修复后该步骤改为 Equal。

## 过程记录

开发中一次 Equal 失败经最小复现定性为 fixture 笔误（`CSS_WITH_HIGHLIGHT` 漏写 `#sec-b` 规则，全量路径输出正确）——oracle 对期望错误同样敏感，修正后 8/8。差异报告含字段路径上下文（pretty-Debug 行对齐在首个差异处截断）。

## 边界与未覆盖

- 覆盖面：style 可观察输出。**未覆盖**：布局几何（缓存失效输入 DC-3）、paint/帧复用与缓存生命周期（DC-4）、生产默认路径成本（DC-5）——分别对应 M1 布局切片、M3、M4。
- `:has()` 全量回退**有行为断言、无计数器**（DC-2"全量回退有计数"待 M2 补 instrumentation）。
- 反例 B 的修复会改变 `compute_styles_incremental` 行为，需 style-system 全量回归 + 消费者侧（paint-only 增量臂、gCS 缓存）行为核对后落 Equal。
