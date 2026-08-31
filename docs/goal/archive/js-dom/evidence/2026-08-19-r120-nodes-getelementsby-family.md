# R120 — M4 nodes：getElementsBy* 族 NS 感知匹配（四文件全 100%，+48 净双路径）

**日期**: 2026-08-19
**里程碑**: M4（WPT dom 上游基线建立与扩展）
**驱动用例**: `Element-getElementsByTagName.html`（4F→19P）、`Document-getElementsByTagName.html`（6F→18P）、`Element-getElementsByTagNameNS.html`（3F→16P）、`Document-getElementsByTagNameNS.html`（14F→14P）——四文件 100%；`case.html` 285P（非 ASCII 名 2F 修复）
**规范**: https://dom.spec.whatwg.org/#concept-getelementsbytagname / #concept-getelementsbytagnamens

## 结果摘要

| 路径 | 前 | 后 | 净 |
|------|----|----|----|
| polyfill nodes 全量 | 7551P | 7599P | +48（50F→P，零新增 fail） |
| native nodes 全量 | 6072P | 6112P | +40 |
| case.html | 283P | 285P | +2 |

traversal 1595 / events 419 / collections 48 不变（中途回归 9F 当轮修平）。

## 根因与修复（六层）

1. **匹配算法缺失 NS/大小写语义**：旧实现直接委托 `querySelectorAll(tag)`（host 查询不区分
   ns/大小写）。新 `_zwFilterByTagNameNS` 统一匹配器（part05，Element/Document/NS 三处共用）：
   - 非 NS 变体：qualifiedName（tagName 含 prefix）**双方 ASCII 小写**比较（HTML ns）或
     原样精确（非 HTML ns——'ST' 命中 'ST' 不命中 'st'）；HTML ns 元素 localName 非纯
     ASCII 小写 → 永不匹配（WPT「uppercase tagName never matches」：createElementNS
     (HTMLNS,'I') 的 ('I')/('i') 都不命中）。
   - NS 变体：localName **原样精确**比较（createElementNS(HTMLNS,'ABC') 只被
     ('HTMLNS','ABC') 命中）；ns 匹配先行（'*' 任意 / null 匹配无 ns / 精确串）。
2. **document 级枚举缺动态子**：新 `_zwDocAllElements`（快照 `__zw_query_all('*')` ∪
   `_zwPendingAdded` 动态子 + in-doc 门；快照不支持 '*' 时回落 documentElement/html/body
   子树展开）。回落与 pending **并存**（首版回落条件 `!out.length` 被 pending 独占——A/B 抓到）。
3. **live collection 三处缺失**：`liveSpec.matches` 闭包（`_zwLiveMatchesFor`）接到
   element/document/NS 全部入口；`_zwHCLiveInvalidate` 的 add 段增**作用域放行**
   （`scopeHandle/scopeSel`——detached handle 容器上的 element 级集合按容器匹配放行，
   文档级集合维持 R54 in-doc 门）。
4. **named 暴露不对称**（ownKeys/namedFor/prototype namedItem 三处同款）：**id 暴露对
   所有元素**（document-wide named lookup）、**name 暴露仅限 HTML ns 元素**
   （WPT own-props：z=createElementNS('') 的 id 暴露、w=同款元素的 name 不暴露）。
   `_zwIsHTMLNamespace` 增 `_nsHandles` registry 判定（createElementNS('') 产物非 HTML）。
5. **NodeList/HTMLCollection 构造器缺失**（Interfaces/ReferenceError 崩簇）+ prototype
   `item`/`namedItem`/`length` 接线——expando 断言 `fn === HTMLCollection.prototype.item`
   要求 identity 相同（p 层转发构造器 prototype 的同一函数，不定义副本）。
6. **tagName Unicode 大写化 bug**（case.html non-ASCII 2F）：createElementNS(HTMLNS,'ä')
   的 tagName 走 `.toUpperCase()` 把 'ä'→'Ä'（spec 是 ASCII uppercase）——改 ASCII-only
   转换，连带修复 getElementsByTagName('ä'/'Ä') 的匹配分叉。

## 回归与修复（A/B 门当轮抓到）

- HTMLCollection-iterator -2F / HTMLCollection-empty-name -7F：R120 段 python 替换误删
  p 层 `Symbol.iterator`/`Symbol.toPrimitive` 定义（原版形态恢复）+ prototype namedItem
  补空串早退。
- r54_detached 引擎测试失败：pending 并入缺 in-doc 门（R54 防泄漏语义保持）。
- r12 引擎测试失败：旧断言固化「null ns 忽略」的 polyfill 语义——按 spec 纠正
  （null 只匹配无 ns 元素，改用 HTMLNS 查询断言）。

## 验证

- 四 driving 用例 + case.html polyfill 全绿；native nodes 6112P 同步
- engine 单测 `test_get_elements_by_tag_name_family_r120`（13 断言组）
- `make test` 65 套件全绿 exit 0；fmt 无 diff；clippy `-D warnings` 零警告
- 账本：`tests/wpt-runner/imported-tests.txt`（R120 条目）

## 教训

1. python 大段替换前必须核对切除边界（本次误删相邻的 Symbol 定义块——A/B 门是唯一安全网）。
2. WPT 期望表是语义仲裁者：null-ns 匹配、id/name 暴露不对称、uppercase-never-match
   三处都与「直觉的 spec 读法」相反，逐用例对期望表而非凭记忆写规则。
3. live collection 的「作用域」与「文档级」是两个独立观察面——in-doc 门对文档级集合
   是防泄漏，对容器作用域集合是误伤，须按集合建立时的容器区分。
