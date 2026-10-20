# M2 首簇 — innerHTML/insertAdjacentHTML/outerHTML character reference 直通修齐

**日期**: 2026-10-02
**对应缺口**: P2 解析树一致性（首簇 NCR 表）| **基线**: [2026-10-02-m1-baseline.md](2026-10-02-m1-baseline.md)

## 根因（双向）

1. **宿主侧**：`js_dom_bridge.rs` 三处 setter 共用的「无 `<` → 整串造文本节点」快速路径
   （replace_inner_html / insert_adjacent_html / replace_outer_html_node）跳过
   fragment 解析，character reference（命名表/数字引用/C1 重映射）全部直通成字面文本。
2. **shim 侧**：`part04.js` innerHTML setter 的纯文本分支把**原始串**注册进本地文本
   视图（`_zwRegisterTextEl(..., _ihVal)`）——宿主 apply 异步，同 turn 读（textContent/
   childNodes 断言）命中的是本地视图，宿主侧修齐后仍读回字面串。

## 修复（共享路径，无实体表副本）

- 宿主：删三处快速路径——纯文本与 markup 一律走 `parse_html_fragment`（html5ever
  tokenizer charref 语义：2231 命名条目含 legacy 无分号形态 + 数字引用 + C1 重映射，
  零自维护表）。
- shim：纯文本注册文本改取 `_zwFragmentAdded`（宿主解析产物，已 trim + 展开）的
  单 text 子 data；宿主无解析器时回落原串。
- DOMParser（`parse_html` 全解析器）与 insertAdjacentHTML 本地视图（宿主
  `__zw_parse_html_child_nodes`）本就展开，未改。

## 结果（全通道复跑逐案对账，分母恒 113 案）

| 指标 | M1 基线 | M2 首簇 |
|---|---|---|
| Pass | 43884/61304 = 71.58% | **46122/61304 = 75.23%** |
| named-character-references | 0/2231 | **2231/2231（整簇清偿）** |
| zero.html（数字引用簇） | 1/14 | 8/14（+7） |
| 回归案数 | — | **0**（逐案对账零回归） |

## 门禁

- `make test` 68 suites 全绿；clippy -p zero-engine 零警告；fmt 干净
- 新单测 `test_apply_set_inner_html_character_reference_r5000`（engine part02.rs）
- 定向 bench-gate（zero-engine 26 指标）：首窗 1 指标超预算（输出未捕获），连续两窗
  **GATE PASS 全预算内**——判单窗事件，不改基线
