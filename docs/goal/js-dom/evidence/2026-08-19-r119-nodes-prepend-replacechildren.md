# R119 — M4 nodes：prepend/replaceChildren handle 容器路径（双用例 100%，+25 净双路径同步）

**日期**: 2026-08-19
**里程碑**: M4（WPT dom 上游基线建立与扩展）
**驱动用例**: `dom/nodes/ParentNode-prepend.html`（8F→21P 100%）+ `dom/nodes/ParentNode-replaceChildren.html`（13F→29P 100%）
**规范**: https://dom.spec.whatwg.org/#dom-parentnode-prepend / #concept-node-replace-all + https://github.com/whatwg/dom/issues/1045

## 结果摘要

| 路径 | 前 | 后 | 净 |
|------|----|----|----|
| polyfill nodes 全量 | 7526P | 7551P | +25（26F→P，零回归） |
| native nodes 全量 | 6047P | 6072P | +25（同步） |

traversal 1595 / events 419 / collections 48 双路径不变（per-case 零差异）。

## 根因与修复（四层）

1. **handle 容器 prepend 无实现**（part04 `prepend` 分支仅 sel-based，createElement('div')/
   DocumentFragment/cloneNode 容器静默 no-op——用例族 `Cannot read properties of undefined`）。
   新 `_prependHandleVariadic`（part05）：registry 头插保持**参数序**（物化后逆序 unshift，
   [t1,t2] 正序头插）+ host mutation 经 R101 全 handle wire（`__zw_insert_before_handle_handle`，
   ref=原首子；ref miss 降级 append——prepend 到空容器 == appendChild 语义）+ fragment
   flatten + `_zwDetachFromRegistry` 移动语义 + null/undefined → WebIDL 文本。
2. **handle 容器 replaceChildren 无实现**。新 handle 分支：清空（`_zwRemoveHandleNode`
   = registry 剔除 + 反链清 + `__zw_remove_handle` host wire）+ 追加（`_appendVariadic`）+
   **移动记账**（参数来自其他 handle 容器时：旧父 registry 剔除 + 旧父 observer 每子一条
   removed record——WPT move-order previousParent 断言 mutations.length=2、每条 1 子）+
   本容器**单条合成 record**（spec replace-all steps 6-7 一次 queue added+removed——
   首版逐子 notify 致 parent observer 收 3 条被 A/B 抓到同轮修）。
3. **detached doc replaceChildren 三缺口**（part03 `_r117Install`）：① 清空改 firstChild-while
   （旧快照 for 循环在 removeChild 状态分裂时漏删——`replaceChildren()` 后残留 1 子）；
   ② 校验移到清空**后**（whatwg/dom#1045：replace-all 先移除现有子再做 pre-insert 校验——
   「with an element, replacing an existing doctype and element」期望成功，旧先校验误抛
   「more than one Element」）；③ 字符串参数在 Document 目标抛 HRE（Text 不可进 Document，
   `_r117Validate` 只查 object 参数）。附带：detached doc `appendChild` 补 fragment flatten。
4. **doc prepend 缺 doctype-vs-doctype 校验**（part06）：spec pre-insert 步骤 6 II——doctype
   参数 + doc 已有另一 doctype → HierarchyRequestError（WPT pre-insertion-validation-hierarchy
   共享用例，经 ParentNode-prepend 载入）。

## 验证

- 两个 driving 用例 polyfill/native 双路径全绿（21P/29P）
- engine 单测 `test_prepend_replace_children_handle_paths_r119`（part20.rs，9 断言组：
  文本/null/参数序+identity/清空/移动记账断链/doc 清空/doc 元素替换/字符串 HRE）
- `make test` 全绿 exit 0；fmt 无 diff；clippy `-D warnings` 零警告
- 账本：`tests/wpt-runner/imported-tests.txt`（R119 条目）

## 过程记录

- 探针踩坑 ×2（均取证后即时修正）：① 探针顶层 `const` 箭头函数跨 `<script>` 不可见——
  R3258 既有设计（classic 脚本 try 包装使顶层 let/const/class 成块作用域），探针改单
  script + `var`；② assert_equals(actual, expected) 参数序写反导致读数不可见。
- 首版 replaceChildren 移动记账逐子 notify 被 move-order 用例当场否决（parent observer
  期望 1 条合成 record、previousParent observer 期望 2 条逐子 record——**旧父与新父的
  记账粒度不同**：旧父走 pre-insert remove、新父走 replace-all 单 record）。
