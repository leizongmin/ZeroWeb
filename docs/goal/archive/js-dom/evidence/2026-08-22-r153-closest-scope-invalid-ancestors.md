# R153 — Element-closest 的 :scope 作用域语义 + :invalid form/fieldset 祖先形态

**日期**: 2026-08-22
**里程碑**: M4（WPT dom 上游基线扩展）
**commit**: `e2a17e0cf`
**驱动用例**: `dom/nodes/Element-closest.html`（29 subtest）

## 核实：并行 R152 click 激活对 single-activation 的削减

R151 记录 19P/113F → 本轮实测 **85P/47F**（并行 R152 `242b9b7d1` click() 激活三段
模型削减 66F）。剩余 47F 分布（empty-activation 归因）：
- `child <A>/<AREA>` 18（hash 导航 activation——detached clone 子树上 location.hash
  + window.onhashchange newURL 字符串形态）
- `child <LABEL><INPUT><SPAN>` 10（LABEL 转发激活）
- `child <INPUT checkbox/radio>` in FORM 父 12（FORM submit/reset 联动 / radio 组互斥
  在 clone 子树）
- 4 个 a-area 类型错配（activated 收到元素对象而非 href 字符串——`e.href` 映射
  `getExpectedActivations` 的 `endsWith("testN_link")` 判定失败）

仍属 detached clone 子树激活深水区（R151 评估不变）。

## 根因与修复（两件）

### ① `:scope` 的 closest 作用域语义（4F → 0F）

**根因**：spec selectors-4 §6.4 + `dom-element-closest`——`:scope` = 作用域根，closest
的 scoping root = **调用元素自身**。现实现 `closest_matching_selector`（selector_match.rs）
经文档级 `querySelector_all` 求全匹配集，`:scope` 由 dom crate `is_scope_element` 判为
**文档根元素 html** → `test4.closest(':scope')` 命中 html 而非自身（沿 parent 链遇 html
前的所有站都不在匹配集 → 返空）。

**修复**（最小正解，不动选择器引擎）：`closest_matching_selector` 把 test_sel 中的
**独立 `:scope` token** 文本替换为 start 元素的唯一选择器
（`unique_selector_for_node`），再走既有全匹配集。`replace_scope_tokens` 做 token 边界
判定（后随空白/组合器 `>` `+` `~`/逗号/`)`/串尾才算独立 token；`:scope-x` 等前缀同名
防御性不替换）。四形态验证：
- `:scope` → `#opt`（自身）
- `select > :scope` → `#opt`（组合器左段匹配父）
- `div > :scope` → 空（父是 select）
- `:has(> :scope)` → `#sel`（拥有 scope 为直接子的祖先，替换后 `:has(> #opt)` 走
  dom crate 既有 `:has` child_scope 求值）

**备注**：`el.querySelector(':scope ...')` 的 scope 语义（scoping root = 查询元素）
不经本路径（`__zw_query_match_sub` 子树作用域天然等价），无回归面。

### ② `:invalid` 的 form/fieldset 祖先形态（1F → 0F）

**根因**：HTML spec `selector-invalid` 定义 `:invalid` 匹配**三类**：
1. 候选元素自身不满足约束（已实现）
2. **form 元素**是 ≥1 个无效候选的 form owner（未实现）
3. **fieldset 元素**拥有 ≥1 个无效候选**后代**（未实现）

WPT `test11.closest(':invalid')` 期望 `fieldset#test2`——其内有 `input#test9 required`
（text 无 value = valueMissing 无效候选）。

**修复**（dom crate `validation.rs`，DOM 选择器与 style-system CSS 匹配共享同一权威
判定）：
- `is_invalid_element` 非候选分支按 tag 分派：form → `has_invalid_candidate_in(node,
  false)`（候选的 form owner 须是 root 本身，嵌套 form 不算）；fieldset →
  `has_invalid_candidate_in(node, true)`（任意后代，spec 措辞即 descendant）。
- `is_valid_element` 保持仅候选形态（spec `selector-valid` 只定义「candidates that
  satisfy their constraints」——无祖先形态），判定体内联避免与祖先形态的
  `is_invalid_element` 互递归。

## A/B 验证

| 项 | polyfill | native |
|----|----------|--------|
| Element-closest | **29P/0F**（vs R152b 24P/5F） | **29P/0F** |
| Element-matches | 6P/1F（Element-matches.html 整页 error pre-exists，stash 基线核实） | — |
| 全量 dom 套件 | **6257P/297F/18T**（vs R152b 6253P/301F：净 +4P/-4F，fail 集合 diff 仅 Element-closest 四条消失，**零新增**） | — |
| dom crate 单测 | 837+1 全绿（新增 form/fieldset 祖先形态断言：两类匹配/div 不匹配/:valid 无祖先/移除无效候选后失效） | — |
| style-system | 2254 全绿（CSS matcher `:invalid` 消费同一权威判定，无回归） | — |
| `make test` | 66 套件全绿 | — |
| fmt / clippy | 零警告 | — |

## 未收（记入 R154 候选）

- single-activation 剩余 47F（上述四簇，detached clone 子树激活深水区）
- Attr-prefix 2F / MO-document 3F / realm·adopt 族
- Element-matches.html 整页 error（pre-exists，`Cannot read properties of null
  (reading 'appendChild')`——用例 setup 阶段 null，独立诊断）
