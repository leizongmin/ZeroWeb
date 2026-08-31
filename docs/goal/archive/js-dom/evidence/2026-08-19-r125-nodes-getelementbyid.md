# R125 — M4 nodes：Document-getElementById 动态 id 语义（12F→0F 全 100%，+13 净双路径）

**日期**: 2026-08-19
**Commit**: 本切片提交
**Driving WPT**: `dom/nodes/Document-getElementById.html`（18 subtest，双路径全绿）
**账本**: `tests/wpt-runner/imported-tests.txt`（R125 条目）

## 根因

WPT Document-getElementById 12F 簇的共同根因：**getElementById 的查询源是 host 快照 +
pending-ID 索引，两层都不反映同 execute 内的 id 变更与子树移除**。

1. **sel-based 元素 id 变更 stale**：`querySelector('[id="…"]')` 读 host 快照——同批
   `setAttribute('id', …)`/`removeAttribute('id')`/`Attr.value=` 改 id 后快照仍持旧值
   （mutation 在 execute 结束才 apply），旧 id 误命中 / 新 id 漏检。
2. **pending-ID 索引无 in-document 门**：`<div id=x>` 挂 detached 容器
   （`createElement('div').appendChild(x)`）时 pending 表有条目但元素不在主文档——spec
   getElementById 只返**树中**节点（tree order 首个 in-document 命中）。
3. **祖先移除不可见**：`outer.removeChild(middle)` 后 `inner.appendChild(h1)`——h1 的
   祖先 middle 已移除，整棵子树 out-of-document，但快照仍含（快照未换代）。
4. **innerHTML/outerHTML 替换不剔旧子 pending**：`t9f.innerHTML = ''` 后旧子 `test9`
   仍在 pending-ID 索引 → stale 命中。
5. **handle 容器 outerHTML= 无实现**：`createElement+appendChild` 后赋 outerHTML（WPT
   "add/remove id attribute via outerHTML" 形态）静默 no-op。
6. **append 父 selector miss 硬错中止整批**：同批 Remove 摘除的子树内元素再 append 时
   `find_by_selector` miss → 整批 mutation 以 error 失败（页面脚本全挂）。
7. **解析本地元素无接口原型**：`t8.firstChild instanceof HTMLDivElement` 断言失败
   （`_zwMBuildNode` 产物是 plain object）。
8. **（回归根因）`_zwMEl` 节点缺 namespaceURI**：R125 原型链接后
   `node instanceof Element` 为真——DOMPurify `_checkValidNamespace` 开始消费
   `element.namespaceURI`，undefined 不在 ALLOWED_NAMESPACES → 元素被误杀
   （`sanitize('<img src=x onerror=alert(1)>')` 返空串，r3019 回归）。

## 修复面

| 层 | 修复 |
|----|------|
| part05 | `_zwIdOverrides`（elKey → 新 id \| null）latest-wins 覆盖表 + 三访问器（Set/Get/Entries） |
| part04 | setAttribute/removeAttribute id 分支摘旧键挂新键（写前摘——写后摘会摘到新键）+ sel-based 记覆盖表；innerHTML 替换的 `_ihRemoved` 基底改 `_handleChildren` 快照 + 旧子及孙代 pending-ID 剔除；outerHTML= handle 路径（父 sel InsertAdjacentHtml 'afterend' 先插后摘）；sel-based 子元素 removeChild 三件（host Remove mutation + `_zwMarkRemoved` + childList record） |
| part06 | getElementById 空串早退 null；快照命中先验覆盖表/pending-removed/`_r125AncestorRemoved`（沿 `_zwNodeParent` 反链/`__zw_parent` 上行 32 层）；快照 miss 正向查覆盖表；pending-ID 条目 in-doc 门（`_zwMutationInDoc`）+ 树序近似；pending 子树扫描同门 |
| part03 | `_zwMBuildNode` 接口原型链接（`__zwHtmlTagIface` 表 → `HTML*Element.prototype`，miss 回落 HTMLElement.prototype）；`_zwMEl` 补 `namespaceURI`（`snap.ns` 回落 HTML ns） |
| engine 桥 | `apply_dom_mutations_full` AppendChild 父 miss → lenient `continue`（child 仍由 handles 表登记不断链） |

## A/B 验证

- **Document-getElementById.html**：polyfill 6P/12F → **18P/0F（100%）**；native 同步
  **18P/0F（100%）**。
- **dom/nodes 全量**：polyfill 7825P → **7838P（+13）**；native 6107P → **6120P（+13
  同步）**；fail 文件集与 R124 基线逐文件比对**零新增**（连接修复连带 insert-adjacent
  ×1 + name-validation ×3 + create-element-realm-after-adoption flake 池）。
- **回归面**：traversal 1595P/9F、collections 48P、events 419P/27F、Element-classlist
  1420P 全绿、MutationObserver 102P/10F、NodeIterator 794P/1F、TreeWalker 804P/8F、
  Node-contains 1482P 全绿——与 R124 同值零回归；createHTMLDocument 2P/13F 与 clean-HEAD
  基线逐值一致（既存缺口）。
- **DOMPurify 回归**：`test_sanitize_dompurify_real_r3019` 修复后通过（namespaceURI
  根因，见上 §根因 8）；`test_sanitize_dompurify_style_r3018` 持续绿。
- **engine 单测**：`test_get_element_by_id_dynamic_id_r125`（14 断言段：空串/
  setAttribute 改名/移除/detached/Attr.value/innerHTML 增删/outerHTML 引号形态/
  子树移除可见性）；js_dom_bridge 全量 **617P/0F**。
- `make test` 全绿 exit 0（v8 + quickjs 双矩阵；一次跑观测到 762P/1F 为 flake 池成员，
  复跑全绿）；`cargo fmt --check` 无 diff；clippy 双矩阵零警告。

## 教训

1. **「查询源的时效」是查询类 API 的第一设计问题**——本切片的覆盖表/in-doc 门/祖先
   判定三层都在回答「查询源何时反映变更」；后续查询族（matches/closest 的 live 化）
   同样先画查询源时效矩阵再动手。
2. **原型链接是行为面放大的开关**——setPrototypeOf 之前 `instanceof Element` 恒 false
   使库（DOMPurify）的整段校验被跳过；链接后 namespaceURI 等字段从「可选装饰」变
   「被消费的契约」。给 plain object 补原型时，同步盘点该原型链上所有 getter 的
   消费方（本次 instanceof → namespaceURI → ALLOWED_NAMESPACES 三跳定位）。
3. **摘除-再挂回的键序**：id 变更先摘旧键再写新键（写后摘会摘到新键）——latest-wins
   覆盖表的写序是正确性的一部分，不是实现细节。
