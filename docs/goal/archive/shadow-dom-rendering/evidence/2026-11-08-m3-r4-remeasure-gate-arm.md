# M3 R4 第二臂：IFC inline-only 重测匿名叶 gate（`ZW_SHADOW_TEXT_BOX` 翻默认开）

**日期**: 2026-11-08
**轮次**: 45 号 M3 R4 第二片（paint 侧收口 + 默认翻转）
**提交**: 01dfd0dc8
**结论**: slot-fallback-content-004 严格比对 ✗1.75%→✓0.00%（首次像素级
真通过），`ZW_SHADOW_TEXT_BOX` 翻默认开，shadow-dom 门控矩阵 32P/3F，
零 P→F，kill-switch `=0` 腿与 flag-off 基线全等。

## 1. 根因改判（推翻前片 §3 定谳）

前片把 004 回落归因「inline_finalization 行级 paint 行走不感知 shadow 匿名叶」。
本片 band 取证 + `ZW_TAFFY_TREE_DUMP` + extract 前后盒转储三层法证推翻：

- taffy 树层：正确（host 高度含嵌套匿名叶）；
- extract_layout 层：正确（`is_anonymous_text_item` 按 taffy 几何产盒）；
- **坏点在后处理 step 6.5 `remeasure_inline_only_containers`**：plain-block
  shadow host 命中 inline-only 重测，`collect_inline_items` 只走 light DOM
  （文本已随 slot 分配移入 shadow 树，仅剩空白 `\n  ` 文本节点），测得行数
  total = 0 → 「允许减小高度」收缩臂把 host 高度清 0，匿名叶随宿主塌陷；
  paint 忠实跟随坏几何，前片观测的「行带整体下移一行」是 host1 高度被清 0
  后 sibling 位移的下游表象。

即三层中坏的既不是 taffy 也不是 paint，而是中间的 IFC 重测后处理对
「shadow 化的 plain block」误判为空。

## 2. 修复（`crates/layout-engine/src/inline_finalization.rs`）

`remeasure_inline_only_containers` 的 `has_inline_children` 判定增加兜底臂：

```rust
let has_dom_anon_text_leaves = box_node.children.iter().any(|c| {
    c.is_anonymous_text_item && !c.is_absolute && !c.is_fixed
});
// has_inline_children = 既有 inline 判定 || (has_dom_anon_text_leaves && has_dom_non_ws_text)
```

- 匿名文本叶存在 ⇒ 该容器的行内容活在 taffy 匿名项里，IFC total=0 不再
  触发收缩清 0（taffy 已正确解出几何，重测结果不与之冲突）。
- **R3848 display:contents 文本上提页保护**：这些页同样产生匿名叶（flag
  无关路径），但 DOM 侧留有真（非空白）文本，`has_dom_non_ws_text` 为真
  ⇒ 重测资格保留，行为不变。首版 gate 用 `has_dom_text`（含空白节点）
  时 004 host1 仍塌（空白 `\n  ` 误判有内容），收紧为非空白口径后通过。
- 顺带把 R105 的重复 `has_dom_text` 块上移合并（净 +56/-25 行）。

`tree.rs` / `tree/runtime_flags.rs`：`ZW_SHADOW_TEXT_BOX` 注释更新 + 默认
翻转（`opt_in`→`enabled`，`=0` 杀）。

## 3. 门控矩阵（shadow-dom 35 案，DC-14 三态）

| 腿 | 004 | P/F | 真通过-可信 | 其余 F |
|---|---|---|---|---|
| flag off（前片基线） | ✓ 0.86% | 32/3 | 8 | 同右 3 案 |
| **default on（本片）** | **✓ 0.00%** | **32/3** | **9** | 同右 3 案 |
| `ZW_SHADOW_TEXT_BOX=0`（kill-switch） | ✓ 0.86% | 32/3 | 8 | 同右 3 案 |

- 004 的 0.00% 为像素逐位一致严格通过；**flag-off 基线的「✓0.86%」实为
  欠阈值假象**——slot 回退文本当时根本没画出来（近似通过桶），本次才是
  该案首次真通过。
- F 面恰为 3 案具名挂账：display-contents-shadow-dom-1 1.97%、
  vw-units 2.08%、slot-fallback-content-006 4.90%（R3 臂范围）。
- 逐案 A/B（flag-off vs default-on）：shadow-dom 35 案 + display-contents
  80 案零 P→F；唯一增量是 shadow-dom-1 的 1.95%→1.97%（同案同档微移）。

## 4. 全量门禁

- `make reftest`（本地 725）：725/725 全绿。
- `make test`：workspace 3814P/6F（6 案 fetch/websocket/img 域，与 HEAD
  detached worktree 探针全同，环境既有失败）；renderer 腿 0F；browser 腿
  742P/1F——`default_actions_work_without_javascript` 经 `git stash` 探针
  证实 HEAD 同败（fetch/credentials 桥通道域），非本片引入。
- `make product-smoke`：PASS。
- `make bench-gate` 定向（ZERO_WEB_BENCH_CRATES=zero-layout-engine）：
  microbench + 4 项绝对预算全过（welcome p95 7.83ms / morning 48.82ms /
  medium 268.5ms / form-input 0.0136ms）。

## 5. 下一步

1. **R3 变异应用臂**：`resolve_slots` 统一挂接（`crates/dom/src/document/
   shadow.rs` ~269 FIXME：attachShadow/childList/slot 属性变异后不重跑
   resolve_slots），006 及命令式 ~26 案翻绿面。
2. B6 补充池通道（testharness 渲染耦合 10 案计门挂此）。
