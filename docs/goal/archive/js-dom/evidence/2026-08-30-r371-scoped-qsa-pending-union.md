# R371 — 元素作用域 QSA 的空 host 归并 + replaceWith pending 重键

**日期**: 2026-08-30
**切片**: 已知 Fail 集合 8→7——`querySelector-mixed-case` 整文件转绿（41 断言全
Pass）；`remove-next-sibling-during-replace-with` 收窄到 R328 遗留深项
**改动面**: `js_dom_shim/part04.js`（QSA 空结果门 + replaceWith 重键）

## 1. querySelector-mixed-case 根因（探针复刻归因）

用例：JS 建 19 节点混合 HTML/SVG/MathML detached 树 → `runTests(tree)`（41
断言全过）→ `container.appendChild(tree)` → `runTests(container)`——第二遍
Test 1 `[viewBox]` 期望 4 得 **0**（逐轮探针：detached 树查询恒对、attach 后
的**作用域**查询恒 0、document GEBI 却 hit）。

归因：`#test-container` 是 sel-based 解析元素，append 的树是 R368 盖章的
handle 子（整棵子树只在 JS registry，host 侧仅有空 wrapper）——作用域 QSA 的
host 子树查询返**空串**（合法 0 命中），而 R322 pending 归并段被
`if (all)` 门**短路**：空串 falsy → 归并循环（R322 探针复刻 mg=4/link=1/
flat=19 全部正常）根本不执行 → 恒返空。空串是合法结果非错误。

**修**：`if (all)` → `if (all !== null && all !== undefined)`——空 host 结果
继续走 R310 pending-removed 过滤 + R322 归并（桶空时行为不变：空列表归并仍
空；非空 host 结果路径完全不动）。

## 2. remove-next-sibling 收窄（负结果备档）

用例链：template content clone（含 script 节点）→ `target.replaceWith(frag)`
→ 插入的 script 执行（移除 `<b>`）→ 容器 querySelector('script') → 移除 →
innerHTML = 两 span。本轮探针剥出两层：

1. **`container.querySelector('script')` null**（根因已修）——replaceWith 的
   前插参数经 `_insertAdjacentVariadic('#target','beforebegin')` 入桶记在
   **被替换者**桶/反链下（parentSel='#target'），随后 `__zw_remove` 移除
   target，真父是容器——桶键/反链仍指已移除的 '#target' → 容器 R322 归并查
   '#container' 桶恒 miss。**修**：移除前取父 sel，移除后桶 added 项与反链
   parentSel 重键到父（桶合并 + `_zwNodeParent` 改写；仅 sel 父形态，缺项
   no-op）。
2. **剩余缺口 = 插入期 script 执行**（R328 遗留，深结构转档确认）：host 对
   pending 插入不执行脚本 → 脚本内 `b.remove()` 未发生；且 sel 域 innerHTML
   为纯 host 读（pending 插入的 span/target 移除均不反映）。两项合并构成该
   文件转绿的剩余前置——插入期脚本执行属 R328「克隆 script 插入期执行」遗留
   深项（转档 L2/基建域），不在轻量切片范围。

## 3. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 目标件 | `querySelector-mixed-case` 整文件 Pass（41 断言含 SVG/MathML/foreignObject 大小写矩阵 + `s`/`i` flag） |
| 全量 dom sweep（polyfill，TIME_LIMIT=2400） | **55495P/9F/15T——Fail 文件集合 8→7 零新增**（remove-next-sibling 收窄后仍 Fail[深项]，Timeout 集合噪声轮转） |
| 哨兵套件 | ParentNode-querySelector 2055P、Element-matches 675P、MO 族 135P/3F 恒等（QSA 路径零回归）；engine v8 2498 / quickjs 1473；integration 784P |
| clippy / fmt | v8 + quickjs 双矩阵 `-D warnings` 零警告 / 无 diff |

**过程教训**：① 「空串 vs null」的语义分叉——`if (x)` 门把合法空结果当缺席
处理，归并/补偿逻辑全部短路；探针复刻（mg=4）与实际路径（n=0）的差值直接
指向门条件。② 插桩改变行为的假象——首轮 instrumentation 后用例「通过」，
逐层剥后确认是一次孤例（同内容重跑恒 Fail），归因必须以**稳定复现的失败态**
为准。

## 4. 后续

- 已知 Fail 集合余 **7**：MutationObserver-document 3F（parse-time 基建）、
  remove-and-adopt-thcrash（window.open 无 popup 通道）、
  remove-next-sibling-during-replace-with（插入期脚本执行[R328 遗留] +
  sel 域 fused innerHTML——两项合并前置）、events 2F（onerror 跨 realm /
  Chromium 专有 pseudo）、ranges dataChange/replaceData 2F（文件级 Timeout
  尾批，R353 游离树堆积域）。
- 主线剩余：M5/M7 default-on（待用户点名，改 Mission 级单向门）；M3 已达成；
  M4 基线持续维护；M2 已收口；M8/DC-8 已收敛。
