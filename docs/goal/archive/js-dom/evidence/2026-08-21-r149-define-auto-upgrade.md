# R149 — EventTarget-add-listener-platform-object 1F→0F（customElements.define 既有元素自动升级）

**日期**: 2026-08-21
**里程碑**: M4（WPT dom 上游基线扩展）
**驱动用例**: `dom/events/EventTarget-add-listener-platform-object.html`（1 subtest）

## 根因与修复（两层）

### ① define 不自动升级文档中既有元素

用例：parser 先建 `<my-custom-click id=click>` → 脚本 `customElements.define(...)` →
既有元素应按 spec `custom-element-registration` define 末步**自动升级**（ctor 体 +
connectedCallback 立即触发）。旧 define 只注册——connectedCallback 永不跑 →
`this.addEventListener("click", this)` 未注册 → `customElement.click()` 后
`dataset.yay` undefined（"expected 'It worked!' but got undefined"）。

修复：define 末尾 `_ceUpgradeSubtree(document)`（同步执行；spec 是 upgrade queue
微任务——headless 同步等价，whenDefined waiter 在 resolve 前，时序无冲突）。
复用 R3269/R94 的既有升级管线（ctor 体重放 + R3274 初始 attr change +
connectedCallback）。

### ② upgrade 初始 attr change 仅对**存在**的属性派发

自动升级暴露了 `_ceFireInitialAttrChanges`（R3274）的 best-effort 偏差：对
observedAttributes **每项**无条件派发（缺失属性 null→null 回调）。真实浏览器/
spec（custom-elements upgrade enqueue「if its value is not null」）仅对**存在**
的属性派发。R3205 既有测试（define 时 foo 未设 → 首回调应为后续 setAttribute 的
null->a）与此一致——旧实现在自动升级下多出一条 `foo:null->null`。

修复：getAttribute 返 null 的 observed 属性跳过（不派发）。**两处既有测试断言
更新**：R3274 webview multi-attr 测试（原断言 `a=1,b=null,c=null` 的 best-effort
行为）改 spec 语义（仅 `a=1`）；R3205 engine 测试经修复自然通过。

## 不追项（记档）

event-global 2F（`event-global-is-still-set-when-coercing-beforeunload-result` /
`...when-reporting-exception-onerror`）：依赖 cross-realm `frames[0].Function(...)`
构造跨 realm onerror handler——iframe 跨 realm 基建深项（与 handleEvent-cross-realm
5F 同簇），本轮不追。

## A/B 结果（polyfill / native 双路径）

| 套件 | 结果 |
|---|---|
| EventTarget-add-listener-platform-object | **1P 双路径** |
| dom/events 全量 | **441P/15F/9T**（vs R148 440P/16F/9T：+1P/-1F fail 集仅该件消失）；native 441P/15F/9T 逐文件一致 |
| CE 回归 | lit e2e 6P + custom element e2e 11P 不回归（define 自动升级触达全部 CE 路径） |
| dom/nodes 全量 | 5472P/230F/10T——Node-parentNode-iframe 隔离复跑在 **stash 基线同 Timeout**（既存 flake 非回归） |
| `make test` | 66 套件全绿 |
| fmt / clippy | 零 diff / 零警告（v8 + quickjs 双矩阵） |

## 单元测试（part21.rs 追加）

`test_define_auto_upgrade_existing_elements_r149`：既有 `<auto-ce greet="hi">`
define 后 ctor 跑 + connectedCallback 跑 + 初始 attr change 仅 greet（`greet:null->hi`；
后续 `setAttribute('foo','a')` 的 `foo:null->a` 是 foo 首回调）+ `instanceof` 成立
四段断言。
