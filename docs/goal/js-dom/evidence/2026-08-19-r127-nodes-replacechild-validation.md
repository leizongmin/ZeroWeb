# R127 — M4 nodes：Node-replaceChild spec replace-child 校验族 + replace 语义（15F→0F，+16 净）

**日期**: 2026-08-19
**Driving WPT**: `dom/nodes/Node-replaceChild.html`（15F → 0F，29P/29P 双路径 100%）
**连带**: `dom/nodes/MutationObserver-childList.html` 两个悬挂 async_test（replacement/internal
replacement）被解锁后修平；`Node-childNodes-cache-2.html` 1F 意外修复。
**账本**: `tests/wpt-runner/imported-tests.txt`（R127 条目）

## 根因（四层）

1. **泛型 `Node.prototype.replaceChild` 的 own-delegation bug**（R126 同款教训的 replaceChild
   复发面）：`typeof this.replaceChild === 'function'` 命中原型方法自身 → `own.call` 无限递归
   栈溢出。R126 修了 removeChild，replaceChild 漏修——本轮 own-property 委托判定补齐
   （`Object.prototype.hasOwnProperty`）。
2. **Document pre-insert step 6「给定当前子」校验全缺**（8F）：fragment 含 text/多元素、
   element 与既有 element/doctype 位置冲突、doctype 重复/element 之后——spec 检验表未实现。
3. **replace 语义错误**（3F）：旧实现「先定位 old index 再 adopt new」在 new 是 old 的
   兄弟时 adopt 移除使 old 的 index 前移 → splice 错位/残留；replace-with-self 无短路。
4. **proxy 层 selector-based newChild 无路径**（1F + MO 2 悬挂）：主文档内既有元素作
   newChild（wire 只支持 handle 子插入）旧静默 no-op——`b.parentNode` 残留旧值、MO record
   永不产生（async_test 悬挂不跑，R127 加严后被解锁暴露）。

## 修复面

| 层 | 修复 |
|----|------|
| part03 泛型 replaceChild | own-property 委托判定 + `_r127DocPreInsertCheck`（kids+oldChild 位置感知：element 看 doctype-after，doctype 看 element-before）+ 先 adopt 后定位的本地替换语义 + replace-with-self 短路 |
| part03 detached doc replaceChild | step 6 校验 + fragment flatten（df 子展开进 doc.childNodes）+ adopt ownerDocument 重指（`concept-node-adopt`——跨 detached doc 移动后 `doctype2.ownerDocument === doc`）+ children 同步 |
| part03 `_zwMEl` replaceChild | replace-with-self 短路 |
| part04 proxy replaceChild | `_r126IsChildOf` 提升到 get trap 作用域（removeChild/replaceChild 共用融合视图子判定）+ NotFound 校验 + replace-with-self 短路 + R100 handle 路径 adopt-first（先移除 new 自身条目再定位 old）|
| part04 selector-newChild 路径 | 新增：① adopt record（newChild 旧父发 removed + prev/next 上下文——n52 双 record 期望）② replace record（removed=[old]+added=[new] 单条——n50 期望）③ old 走 remove 同款记账（wire remove + 标记 + pending + 迭代器通知 + CE 断连）|

## 位置感知校验的关键纠正

首版 `_r127DocPreInsertCheck` 用「rest 中有 doctype → element 不可插」——错。spec 检验按
**插入位（oldChild 原位）** 判方向：element 不可插当且仅当「文档另有 element（≠old）」或
「有 doctype 在插入位**之后**」；doctype 对称（「另有 doctype」或「有 element 在插入位
**之前**」）。"Replacing the document element with a single element should work"（doctype 在
插入位之前）正是方向反了才误抛。

## A/B 验证

- **Node-replaceChild**：polyfill 14P/15F → **29P/0F（100%）**；native 0F。
- **dom/nodes 全量**：polyfill 7857→**7876P（fail 集 326→310，逐文件 diff 零新增）**；
  native 6375P。
- **MO-childList**：19P/1F → **22P/1F**（两个悬挂 async_test 解锁修平；剩 1F 为既知
  surroundContents prevSibling 缺口，基线同值）。
- **回归面**：events 419P/27F、traversal 1595P/9F、collections 48P/0F、MO 全族
  105P/10F（文件级分布与基线逐项一致）、Element-classlist 1420P/0F——零回归。
- **单测**：engine `test_replace_child_validation_and_semantics_r127`（9 断言段）。
- `make test` 全绿 exit 0（v8+quickjs 双矩阵 66 套件）；fmt 无 diff；clippy 零警告。

## 教训

1. **R126 的 own-property 教训有同族复发面**——原型方法内「有自身实现则委托」判定的
   bug 会在同族方法（removeChild/replaceChild/insertBefore...）逐个复发，修复时应主动
   枚举同族方法一并收口，而非等下轮 WPT 暴露。
2. **spec 位置约束的方向性**：「element 须在 doctype 后」的校验必须以**插入位**为参照
   判 before/after，不能用「存在性」近似——存在性版本会把合法的 replace-docElement
   误拦。
3. **静默 no-op 的 MO 测试形态是「悬挂」不是「通过」**：async_test 的 mutationFunction
   no-op → record 永 empty → callback 在 microtask 检查 0≠N 应 FAIL——但悬挂的
   async_test 在 runner turn 结束前不结算 = 看不见。加严修复解锁后 fail 数先涨后修平，
   这是「reveal then fix」的正常形态，不是回归。
