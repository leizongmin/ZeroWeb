# R126 — M4 nodes：Node-removeChild spec pre-remove 校验族（28F→9F，+19 净）

**日期**: 2026-08-19
**Driving WPT**: `dom/nodes/Node-removeChild.html`（polyfill 0P/28F → 19P/9F；剩 9F 全部为
frame 域 R117 既知 iframe 跨文档深结构缺口）
**账本**: `tests/wpt-runner/imported-tests.txt`（R126 条目）

## 根因

shim 的 `removeChild` 全层 lenient——任意 child（非子节点/null/非 Node）都静默返 child：

1. **proxy 层**（part04 get trap）：无任何校验，detached/非本父子节点照常走移除记账。
2. **`Node.prototype` 泛型层**（R117）：lenient 注释明示「sel 父融合视图不完整会误抛」——
   但真实缺口只是「无 childNodes 视图的透明父」，有完整视图的叶子节点（_zwMText/
   _zwMComment childNodes 恒 []）可以安全校验。
3. **`_zwMEl`/detached docEl 层**：无校验 + docEl 缺 appendChild/insertBefore mutation 面。
4. **校验顺序错误**：泛型层先抛「父类型不能有子」（HierarchyRequestError），spec
   `dom-node-pre-remove` 步骤 1 是 child 包含检查（NotFoundError）——WPT synthetic
   text/comment `s.removeChild(doc)` 期望 NOT_FOUND_ERR。

## 修复面

| 层 | 修复 |
|----|------|
| part04 proxy removeChild | ① WebIDL Node 类型校验（null/非 Node → TypeError）② `_r126IsChildOf` 融合视图子判定（handle registry / `_childNodeList` 快照∪pending）③ 已移除节点（`_zwIsRemovedNode`）lenient 返 child——同批移动后 parentNode 读 host 快照仍指旧父，调用方按旧父重试是 shim 视图限制下的合法形态 ④ parentNode 融合视图残留（`target.parentNode === this` 但子列表已剔除）lenient——同视图体系自洽，真 detached 不命中照抛 |
| part03 Node.prototype 泛型 | 步骤顺序纠正（包含检查先于类型检查）+ WebIDL TypeError + **own-property 判定**（`typeof this.removeChild` 命中原型方法自身 → 无限递归栈溢出 RangeError，探针实证）+ 有 childNodes 视图（叶子节点）就地校验 |
| part03 `_zwMEl` removeChild | NotFound + TypeError 校验（synthetic createElement 产物） |
| part03 detached docEl | 补 appendChild/removeChild/insertBefore mutation 面（旧缺 appendChild 直接 TypeError） |
| part05 `_zwDomException` | `globalThis.DOMException`（原词法引用——R6/R9 已有教训的复发面；WPT `(doc.defaultView \|\| self).DOMException` identity 断言） |
| dom_bindings native removeChild | 无 slot 参数分级：null/非对象 → TypeError；有 nodeType 无 slot（叠加路径 polyfill 节点）→ NotFoundError |

## 连带修复

- **Event-dispatch-target-moved 回归**（本切片引入→当轮修复）：严格校验使 listener 内
  `parent.removeChild(target)` 的重试（parentNode 快照残留守卫 `parent === target.parentNode`
  旧值）抛 NotFoundError → lenient 两分支（已移除标记 / parentNode 残留）收口。教训：
  **加严校验前先枚举既有调用方对 leniency 的隐性依赖**——lenient 不是 bug 是部分调用方
  的（stale 视图下的）契约。

## A/B 验证

- **Node-removeChild**：polyfill 0P/28F → **19P/9F**（9F 全 frame 域 R117 既知缺口）；
  native 0P→**13P/15F**（+13；6F 为叠加路径 polyfill document 无 internal slot 的
  NotFound 形态——master.md 未解决问题 #9 的范畴；3F TypeError 用例 native 已过）。
- **dom/nodes 全量**：polyfill fail 集 261→238（**逐文件 diff 零新增**；-23 = removeChild
  19 + 连带）；native 6120→**6137P（+17）**。
- **回归面**：events 419（target-moved 修复后含）、traversal 1595、collections 48、
  MutationObserver 102、Element-classlist 1420、Node-appendChild 2 同值零回归。
- **单测**：engine `test_remove_child_not_found_validation_r126`（11 断言段）+ native
  `native_remove_child_type_and_slotless_validation_r126`（3 分支）。
- `make test` 全绿 exit 0（v8+quickjs 双矩阵 66 套件）；fmt 无 diff；clippy 双矩阵零警告。

## 教训

1. **own vs inherited**：`typeof this.method === 'function'` 不区分自有/继承——在原型方法
   内做「有自身实现则委托」判定必用 `Object.prototype.hasOwnProperty`，否则自引用无限
   递归（本切片 RangeError 实证；V8 栈溢出报 RangeError 不报 call stack exceeded）。
2. **加严校验的回归面 = 既有调用方的 leniency 依赖清单**：lenient removeChild 承载了
   「stale parentNode 视图下重试」的合法调用形态；先枚举（dispatch listener 重试 /
   移动后旧父再删）再设计 lenient 分支，而非全局加严后逐个救火。
3. **spec 步骤顺序即断言顺序**：pre-remove 的包含检查在类型检查前——「先抛哪个错」
   由 spec 算法步骤序决定，不是实现自由。
