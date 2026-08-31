# R117 evidence — nodes：ChildNode/ParentNode mutation 族（before/after/replaceWith/replaceChildren 等）

**日期**: 2026-08-19
**里程碑**: M4（WPT dom 上游基线按聚类驱动修复）
**基线**: R116（nodes 7366P/831F；events 419P/32F；traversal 1595P/10F；`make test` 18,240P）

## 修复（七件 + 三次回归修正）

1. **`_zwDetachFromRegistry` helper**（part05）：spec pre-insert 步骤「node 有旧父先移除」——移动非复制；`_handleChildren` + `_zwNodeParent` 反链清理。
2. **before/after 重写为 spec viable-sibling 顺序 pre-insert 模型**（part04 handle 路径）：viablePrev/Next = 上下文前/后第一个不在参数集的兄弟；每参数 pre-insert 到固定 ref 前（self/兄弟参数先移除再插）。三旧缺口闭合：参数树内残留复制 / `after(self)` 语义（`'text<!--test-->'` = self 移除后重插末尾段）/ 参数顺序保持。**教训**：真实 DOM `[x,c,y].before(y,x)` 结果是 `[y,x,c]`（不是保持原序 `[x,y,c]`）——期望值以实测为准。
3. **replaceWith handle 路径**（part04）：detached 容器内 comment/text/元素的 sel-null 整体 no-op → 自身位置顺序 pre-insert + 移除自身；**self 作参数 = 移动重插**（末段移除被重插抵消，WPT `[x,c,text].replaceWith(x, c)` 期望 `[x,c,text]`）。
4. **层级校验族**（`_zwValidatePreInsert` part05 + detached doc 安装器 part03 + 主文档 append/prepend part06）：含 host 包容祖先环 / Document 收 Text·CDATA·Document → HRE / 非 Document 收 Document·DocumentType → HRE / Document 单元素规则（frag 多元素 / 已有元素 + frag 含元素）。**Document 可收 PI(7)/Comment(8)**——第一版把 7/8 也拒了，NodeIterator-removal 的 PI/comment 回归暴露（spec：Document 子节点允许 Doctype/Element/PI/Comment/Fragment）。
5. **`Node.prototype` 泛型变异族**（part03）：replaceChild/insertBefore/removeChild——WPT `Node.prototype.replaceChild.call(任意 parent)` 形态。校验顺序 spec 对齐：parent-type(HRE) → ancestor(HRE) → child NotFound → node 类型（WPT pre-insertion-validation-notfound 顺序断言族）。**doctype/text/PI/comment/CDATA 节点原型链接 `Node.prototype`**（part03 工厂 + part06 主 doctype）——泛型方法可达。
6. **null/undefined → WebIDL 文本转换**（`_appendVariadic` + `_insertAdjacentVariadic`）：`append(null)` 插入文本 `'null'`（旧 skip）。
7. **detached doc/主文档补齐变异族**（prepend/append/replaceChildren + doctype.cloneNode/remove + doc.replaceChild/insertBefore + `_zwMEl` 节点全家族）——`pre-insertion-validation-hierarchy` 的 createHTMLDocument 载体。

**三次回归修正（lenient 收窄教训）**：
- **removeChild NotFound 校验**：sel-based 父的融合 childNodes 视图不完整（注册文本子/pending 混合）——NodeIterator-removal 14F 回归 → **回退校验**（L2 live 视图后收口）。
- **element-proxy replaceChild NotFound**：webview 集成路径（execute_script_with_dom）的 pending 子不在 JS registry——`test_webview_dom_replace_child` 2 失败 → 回退（泛型 prototype 版保留，WPT 验证用例经 `.call` 走泛型）。
- **泛型 insertBefore refNode NotFound**：browser 加载路径经 insertBefore 挂 pending ref——IndexedDB owner 测试 blank 页加载回归 → lenient。

## A/B 结果（WPT testharness）

| 项 | R116 基线 | R117 | 净 |
|---|---|---|---|
| ChildNode-before/after/replaceWith | 54P/69F | **123P/0F（100%）** | +69 |
| ParentNode-append | 8P/17F | **21P/4F** | +13 |
| ParentNode-replaceChildren | 6P/25F | **16P/13F** | +10 |
| ParentNode-prepend | 2P/20F | **9P/13F** | +7 |
| Node-replaceChild | 9P/51F | **30P/28F** | +21 |
| Node-removeChild | 0P/28F | **12P/16F** | +12（余为 frames 域） |
| dom/nodes 全量 | 7366P/831F | **7479P/716F** | **+113 净** |
| dom/events | 419P/32F | 419P/32F | 0 |
| dom/collections | 48P/1F | 48P/1F | 0 |
| dom/traversal | 1595P/10F | 1595P/10F | 0（中途回归已修平） |

## 单测（part20.rs +1）

- `test_child_parent_node_mutation_family_r117`：after(self) 移动语义 + before 兄弟移动（`[x,c,y].before(y,x)` → `[y,x,c]`）+ replaceWith 文本替换 + append(null/undefined) 文本转换 + Document 收 Text → HRE + 泛型 replaceChild 非 parent → HRE。

## 验证

- `make test` **18,244 passed / 0 failed**（exit 0；含 webview 集成 + browser IndexedDB owner 回归修正后复验）
- `cargo fmt --all -- --check` 无 diff；workspace clippy 零警告
- engine js_dom_bridge 604 单测全绿（含 R117 +1）

## 教训

1. **lenient-to-strict 校验要按视图权威性分层**：JS 侧 childNodes 融合视图（pending/注册文本/registry 混合）不是权威——对它做 NotFound 严格校验必在某个内部路径误抛。校验挂在原型泛型层（测试用例显式 `.call` 消费），元素 trap 层保持 lenient，L2 live 视图后统一收口。
2. **WPT 验证顺序断言是精确的算法步骤测试**：parent-type → ancestor → child NotFound → node 类型——顺序错了类型对了也 fail；反之顺序对了能赚整族。
3. **Document 的可插入类型**：Text/CDATA/Document 拒绝；PI/Comment/Doctype/Element/Fragment 允许——「注释和 PI 可以进 Document」容易记错。
4. **before/after 的 viable-sibling 算法**：固定 ref + 逐参数移动（remove-then-insert）——期望值用真实浏览器语义推导，参数「保持原序」只在参数不在树中时成立。
5. **回归要当场修平**：本切片三次回归（traversal 24F / webview 集成 / browser IndexedDB）都在全量门禁暴露后立即归因修正，最终 traversal 逐值回到基线、make test exit 0。
