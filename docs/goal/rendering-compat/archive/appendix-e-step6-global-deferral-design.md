# Appendix E step 6 — 全局 positioned-descendant 延迟设计

**状态**: ✅ 已实现（R504 LANDED，commit 见 master.md；net +7 / 0 回归）
**日期**: 2026-06-22
**承接**: R503（commit 270c410f，painter `child_paint_sort_key` z-index:auto positioned (1,0)→(3,0)）
**目标**: 恢复 R503 的 2 个回归（`abspos/static-inside-inline-001/003`）并完整实现 CSS 2.1 Appendix E step 6 的全局 tree-order 语义

> **R504 实现笔记**：本设计的「两趟 pre-order 收集」落地为 `collect_positioned_descendants`（收集**所有** positioned 非 auto-only——见 §4.6 修订），3 段 flush（step 2/6/7）。**关键修订**：step 7 排序方向 = **升序**（低 z 先绘、高 z 居上），与 R503 per-node `(key,z_index)` 升序一致；初版误用降序致 z-index-009/010/011 回归。`is_root_scope` 参数线程化（入口 true）。详见 `evidence/r504-appendix-e-global-deferral-landed-2026-06-22.txt`。

---

## 1. 问题陈述

### 1.1 R503 的成就与局限

R503 把 z-index:auto positioned 元素在 `child_paint_sort_key` 中从 `(1,0)`（与 in-flow 并列、tree-order 交错）改为 `(3,0)`（晚于 in-flow(1)/float(2)、早于 SC(4)）。这正确实现了 **direct-child** 层面的 step 6：positioned 直接子元素绘制在其后的 in-flow 兄弟之上。

**6案新 PASS**（spec-correct，positioned 覆于其后 in-flow 兄弟之上）：
`abspos-016`（4.27→0.78%，`position:fixed` green 覆 static red）/ `abspos-overflow-011` / `position-absolute-001/007` / `visuren/position-absolute-008a` / `abspos/between-float-and-text`（8.33→0.83%）。

### 1.2 R503 的 2 个回归（根因 = per-node 排序的内在局限）

`abspos/static-inside-inline-001/003`（0.00%→2.08%）。结构：

```html
<body>
  <div id="red"></div>           <!-- abspos red，body 较早子节点 -->
  <div id="wrapper">              <!-- in-flow，overflow:hidden -->
    <span id="inline">             <!-- inline（line-height:100px）-->
      <div id="abspos"></div>       <!-- abspos green（嵌套，经 R109 block-in-inline 拆分）-->
      X
    </span>
  </div>
</body>
```

**期望（Appendix E step 6）**：`#red` 与 `#abspos` 同属 body 堆叠上下文的 positioned descendants，按 **tree order** 绘制 → `#red`（较早）先、`#abspos` green（较晚）覆之 → 无红。

**R503 实际**：painter 为 per-node 递归（`ordered_child_indices` 只排每个节点的**直接**子节点）。body 层排序：`#wrapper`(1,0 in-flow) → `#red`(3,0 positioned) → 先绘 `#wrapper` 整棵子树（含 `#abspos` green），再绘 `#red` → 红覆绿 → FAIL。

### 1.3 为什么 per-node 排序不可解

step 6 要求**全局** tree-order 收集：一个堆叠上下文内**所有** z-index:auto positioned descendants（不论嵌套多深）在 normal flow（steps 3-5）之后、正 z-index SC（step 7）之前，按 tree order 统一绘制。

per-node 排序只能保证「直接子元素」的相对顺序，无法把「嵌套在 in-flow 后代里的 positioned descendant」上提到 scope 根的 step 6。两个模式在 per-node 下不可兼得：
- **abspos-016 模式**（common）：positioned 应覆于其**后**的 in-flow 兄弟 → 需 (3,0) 延迟。
- **static-inside-inline 模式**（rare）：positioned 应在**其后** in-flow 兄弟的**嵌套** positioned 后代**之前** → 需全局 tree-order。

→ **唯一正确修复 = 全局 positioned-descendant 延迟**。

---

## 2. CSS 2.1 Appendix E 正确语义（本设计的目标）

一个堆叠上下文（SC，real 或 z-index:auto 的 pseudo-SC）的绘制顺序：

1. 自身背景/边框（SC 根）。
2. **负 z-index** 子 SC（按 z-index、再 tree order）。
3. in-flow、非 inline-level、非 positioned 块级后代。
4. 非 positioned float。
5. in-flow、inline-level、非 positioned 后代。
6. **所有 z-index:auto/0 的 positioned descendants**（pseudo-SC），按 **tree order**。
7. **正 z-index** 子 SC（按 z-index、再 tree order）。

**关键**：step 6 收集的是该 SC 子树中**所有** z-index:auto positioned 后代（不含被另一个 positioned 元素或 real SC 截住的——那些属于各自更近的 scope）。每个 step-6 元素自身又是一个 pseudo-SC，对其后代重复 step 1-7。

---

## 3. 现有 painter 结构（修改基线）

- `painter/mod.rs:66 child_paint_sort_key`：返回 `(u8, i32)`，当前值 neg-z`(0,z)` / in-flow`(1,0)` / float`(2,0)` / positioned-auto`(3,0)` / SC`(4,z)`。
- `painter/mod.rs:92 ordered_child_indices`：按 key 排**直接**子节点、tree order tiebreak。
- `painter/mod.rs:347 paint_node`：核心递归。line 537-543 主子循环（按排序序遍历直接子节点、递归 `paint_node`）。
- `painter/mod.rs:210 paint_node_in_rect`：dirty-rect 路径，有自己的子循环（line 323-325）。
- `painter/mod.rs:535 defer_abspos`：overflow 裁剪专用——非 positioned overflow 元素把 abspos/fixed 直接子元素移到裁剪之后绘制（line 620-625）。**这是局部 2-phase 先例**，可参考但语义不同（裁剪 vs 绘制顺序）。
- `layout-engine/engine.rs:733 creates_stacking_context`：`= positioned && z-index:Integer`。**z-index:auto positioned → false**（pseudo-SC，非 real-SC）。⚠️ 注意：此判定**不含** opacity<1 / transform≠none / filter / will-change / isolation 等 SC 触发器——这是独立既有缺口，本设计不修，但实现时须知晓（这些元素当前不被当 SC，其 positioned 后代会错误上提到祖先 scope；属 pre-existing，不计入本设计回归）。
- 偏移约定：子元素以 `child_offset_x/y`（父 content origin）传入 `paint_node`，内部 `abs_x = offset_x + box_node.x`。positioned 元素的 `box_node.x/y` 由 layout 解析为使其在父 content origin 基准下落到正确绝对位置（R98/R500 abspos 后处理已校准）。

---

## 4. 设计：scope-list 线程化 + 两趟收集

### 4.1 核心数据

```rust
/// 一个被延迟到 step 6 绘制的 positioned descendant。
struct DeferredPositioned<'a> {
    node: &'a LayoutBox,
    abs_x: f32,
    abs_y: f32,
}
```

收集时记录**已累积的绝对坐标**（offset 链已算好），flush 时以 `offset = abs - node.xy` 调用 paint，使内部 `offset + node.xy = abs` 还原。

### 4.2 scope 判定

`is_scope(box) = box.is_absolute || box.is_fixed || box.is_relative || box.is_sticky`（即任意 positioned 元素都是其 descendants 的 scope；real-SC 与 z-index:auto pseudo-SC 均是）。根节点（initial containing block / html）也是 scope（paint 入口视为 scope 根）。

### 4.3 算法（每个 scope 根的 paint_node）

把 paint_node 主子循环从「按 sort key 单趟递归」改为**两趟 + flush**：

```
paint_node(box, offset_x, offset_y, parent_scope):
    abs = offset + box.xy
    # 1. 自身背景/边框/文本（不变）
    paint_background_borders_text(box)

    # 2. 确定 scope：positioned → 自身新 scope；in-flow → 沿用 parent_scope
    let mut own_scope: Vec<DeferredPositioned> = if is_scope(box) { vec![] } else { vec::placeholder };
    let scope: &mut Vec<_> = if is_scope(box) { &mut own_scope } else { parent_scope };

    # 3. 第一趟：steps 2,3,4,5（neg-z SC、in-flow、float）
    #    按 sort key 遍历直接子节点中 key ∈ {0,1,2} 的：
    #      - in-flow/float 子：递归 paint_node(child, scope)   ← 沿用 scope，nested positioned 上提
    #      - neg-z SC 子（key 0）：递归 paint_node(child, fresh)  ← real-SC 自成 scope
    #    注意：若该子是 in-flow 且自身含 positioned 后代，递归时它们会被 push 进 scope（上提）。
    for child in sorted_children where key(child) in {0,1,2}:
        if is_real_sc(child) || key(child)==0:  paint_node(child, fresh_scope)
        else:                                    paint_node(child, scope)

    # 4. 第二趟收集：把直接 positioned-auto 子（key 3）按 tree order 追加到 scope
    #    （此时 scope 已含第一趟递归上提的 nested positioned，需保证 tree order——见 4.4）
    for child in sorted_children where key(child) == 3:
        scope.push(DeferredPositioned { node: child, abs: child_abs })

    # 5. step 6 flush：仅当 box 是 scope 根时，按 tree order 绘制 scope
    if is_scope(box):
        sort scope by tree order  # 见 4.4
        for item in scope:
            paint_node(item.node, offset = item.abs - item.node.xy, fresh_scope)

    # 6. step 7：正 z-index SC（key 4）——按 z、tree order 递归，各自 fresh scope
    for child in sorted_children where key(child) == 4:
        paint_node(child, fresh_scope)
```

### 4.4 tree-order 保证（关键难点）

scope 在第一趟（nested 上提）与第二趟（直接子追加）两阶段填充，**直接追加会破坏 tree order**（例：body 的 `#red` 直接子 vs `#wrapper` 内嵌套的 `#abspos`——`#abspos` 在第一趟先入 scope，`#red` 在第二趟后入，flush 顺序变 `#abspos, #red`，错）。

**两种解法（择一）**：

- **(A) 单趟 pre-order 预扫描**：进入 scope 根后，先做一次纯遍历（不绘制），按 pre-order 收集所有 positioned-auto 后代（遇 real-SC 或 positioned-auto 元素则**不**下钻——它们自成 scope），得到 tree-order 列表。然后第一趟绘制 in-flow（**跳过** positioned-auto 子）、step 6 flush 预扫描列表、step 7。**优点**：tree order 天然正确。**缺点**：两遍遍历（性能 + 需在 in-flow 绘制时跳过 positioned-auto）。
- **(B) 带深度/序号的稳定排序**：每个 DeferredPositioned 携带 pre-order 序号（进入 scope 根时用 Cell/计数器分配），flush 时按序号排序。单趟即可。**优点**：单趟。**缺点**：需线程化一个序号计数器。

**推荐 (A)**：清晰、tree order 无歧义；性能开销可接受（paint 非 hot path 瓶颈）。

### 4.5 与现有机制的交互

- **overflow `defer_abspos`（mod.rs:535）**：语义正交（裁剪 vs 顺序）。`defer_abspos` 决定 abspos 直接子是否被本 overflow 裁剪；本设计决定 positioned 后代何时绘制。实现时：`defer_abspos` 的排除/追加逻辑保留，本设计的 scope 收集在其之上叠加。需 A/B 验证 `abspos-overflow-*` 不回归。
- **multicol（mod.rs:549-605）**：列片段内的 positioned 后代——理想应在该片段所属 scope 的 step 6 绘制（含列位移）。MVP 可先让 multicol 路径沿用旧行为（不参与全局延迟），A/B 看 multicol 是否回归；若回归再细化。`css-multicol` 当前 chr<1% 23.5% 本就结构性低，paint-order 不是主因。
- **R109 block-in-inline（static-inside-inline 的关键路径）**：~~`#abspos` 在 `<span id=inline>` 内，inline 被 §9.2.1.1 拆成匿名块片段~~ **★ LAYOUT_DUMP 实证（2026-06-22）已推翻此担忧**：`#abspos` 为 `position:absolute`（out-of-flow），按 §9.2.1.1 **不拆分** 包裹的 inline——LayoutBox 树为 `body > div(#red abspos) / div(#wrapper) > span(#inline) > div(#abspos)`，`#abspos` 作为 `span` 的**普通结构子节点**出现（无匿名片段包裹、无 `is_r109_split`）。painter 主子循环（`paint_node` 无条件遍历 `box_node.children`）经 `body>wrapper>span>abspos` 递归**可达** `#abspos`。**故本设计的 scope-list 线程化能回收 static-inside-inline**——R109/IFC 路径**非**阻塞，原「实现第一步决定性未知」已 RESOLVED（ favorable）。附带正确性收益：`#abspos` 的 CB=viewport（无 positioned 祖先），本不应被 `#wrapper{overflow:hidden}` 裁剪；将其从 `#wrapper` 递归中上提到 body step 6，恰好使它脱离 `#wrapper` 的 clip 包裹——比现状更正确。
- **paint_node_in_rect（dirty-rect 路径，mod.rs:210）**：MVP 可暂不改造（保留旧行为），仅改 `paint_node`；A/B（reftest 走 `paint_node` 主路径）验通过后再同步 `paint_node_in_rect`。

### 4.6 step 6 与 R503 (3,0) 的关系

本设计**取代** R503 的 `(3,0)` 排序值：positioned-auto 子不再在主循环里按 (3,0) 绘制，而是被收集进 scope、在 step 6 flush。`(3,0)` 可回退为占位（或保留为「未启用全局延迟时的 fallback」）。实现完成后 R503 的 6 案 PASS 保持，2 案回归恢复，**且**可能额外修复更多 nested-positioned 案（须 A/B 量化）。

---

## 5. 实现草图（文件级）

1. `painter/mod.rs`：新增 `DeferredPositioned` 结构 + `is_scope()` 辅助。
2. `paint_node` 签名增 `scope: &mut Vec<DeferredPositioned>` 参数（入口 `paint()`/`paint_node_in_rect` 传初始 scope）。
3. `paint_node` 主子循环（537-543）重构为 4.3 的两趟 + flush（推荐方案 A 预扫描）。
4. （后续）`paint_node_in_rect` 同步改造。
5. 保留 `defer_abspos`、multicol 专用循环；A/B 验证交互。

预估改动：paint_node 主循环 ~60-100 行重构 + 签名/调用点 ~10 处。**单文件 painter/mod.rs（现 1232 行）仍在 2000 行限内。**

---

## 6. 验证计划（实现会话执行）

1. **targeted**：`make reftest`（test-guard 包裹）跑 `abspos/static-inside-inline-001/003` → 须恢复 ≤1%（目标 0.00%）。
2. **blast-radius A/B**：`git stash`/`pop` 对比 7 position-sensitive CSS2 子目录（positioning/abspos/zindex/visuren/normal-flow/floats/visufx，1491 案）pass-list set diff：
   - 须 **net ≥ +2**（恢复 2 回归 + R503 的 6 案保持）；
   - **零新回归**（任何 newly FAIL 须逐案根因，不接受 paint-order 退化）。
3. **broader spot-check**：tables/backgrounds/borders/margin-padding-clear/selectors（positioned 较少，验交互无溢出回归）。
4. **multicol/overflow 交互**：`abspos-overflow-*`、`css-multicol/` 抽样验零回归。
5. **make test**（test-guard）：全量 12286/0/72（ignored = real_website）。
6. **make reftest**（test-guard）：inline reftest smoke 全绿。
7. **clippy**：`cargo clippy -p zero-engine --all-targets` 零警告。

**成功标准**：static-inside-inline-001/003 恢复 PASS + position-sensitive A/B net ≥ +2 零新回归 + make test/make reftest/clippy 全绿。**不达标即整体回退**（恢复 R503 (3,0) 态），按本项目的反回归纪律。

---

## 7. 风险与回退

- **主要风险**：~~R109/IFC 路径不可达~~ **已 RESOLVED**（LAYOUT_DUMP 证 `#abspos` 经主子循环可达，见 4.5）。**现主要风险**转为：tree-order 收集正确性（4.4，须 pre-order 扫描或稳定 tree-order 索引，collection-order 因 sort 重排会错）+ overflow/multicol 交互（4.5）+ paint_node_in_rect 同步。**缓解**：A/B set diff 逐案核 + 回退纪律。
- **次要风险**：step 6/7 顺序、overflow/multicol 交互回归。**缓解**：A/B set diff 逐案核，回退纪律。
- **回退**：单 commit，`git revert` 即恢复 R503 (3,0) 态，零残留。

---

## 8. 不在本设计范围

- `creates_stacking_context` 补齐 opacity/transform/filter 等 SC 触发器（独立既有缺口）。
- z-index:0 与 z-index:auto 的 SC 判定差异（CSS3 细化，本设计按 CSS2.1 把两者均作 pseudo-SC）。
- 3D transform / will-change / isolation 的 SC 语义。
