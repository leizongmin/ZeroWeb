# M3 切片 R93 — CE 原型方法派发（get trap 原型链回落放宽）

**日期**: 2026-08-17
**Commit**: `7469a804`
**Milestone**: M3 Web Components 端到端（DC-2）
**前置**: R90（getPrototypeOf trap CE registry 分支）、R92（lit 风格模板渲染进 shadow root）

## 问题

R90 让 `getPrototypeOf` trap 对命中 CE registry 的 tag 返回 `ctor.prototype`，`instanceof` 因此通过；但 part04 的 get trap 对属性名的原型链回落**仅限 SCREAMING_SNAKE 常量**（`ELEMENT_NODE` 族，R80 加）。结果：custom element 的用户原型方法（`MyEl.prototype.bump`，lit/stencil 组件的标准形态）经 `el.bump` 查找时恒返 `undefined`——`el.bump is not a function`，方法不可达。

次要缺陷：链上命中若直接取 `_pchain[prop]`，accessor getter 的 `this` 落在 prototype 对象上，读 `this._count` 得 `undefined`（WC e2e 组 7 的 `doubled` NaN 实证）。

## 根因

- get trap 的兜底分支历史上是**为 Node 接口常量打的补丁**（R80），不是通用原型链回落——CE 场景需要完整回落。
- spec 语义：原型 accessor getter 以**实例**（此处为元素 proxy）为 this 求值；data property（方法引用）取值即可，this 在调用期自然绑定。

## 修复（part04.js get trap）

1. 回落条件从 `/^[A-Z][A-Z_]+$/` 放宽到**全部未命中的非空字符串属性**（expando / shim 已知面优先命中，回落只在 miss 时触发——per-instance 重写仍胜出）。
2. 沿链用 `getOwnPropertyDescriptor` 只取 **own 命中**，限 8 层（防循环链）。
3. accessor：`_pdesc.get.call(_makeProxy(sel, handle))`；data：`return _pdesc.value`。

## 测试资产

`tests/integration/src/e2e_web_components.rs` 断言组 7 `wc_prototype_method_dispatch`：
method typeof / 无参调用 / 带参调用 / this 持久化（expando 写读）/ 原型 getter / per-instance 覆盖优先 / 未定义成员 undefined。

## 验证

- integration WC 组 **7/7 全绿**（含新组 7）
- engine v8 **2188** / quickjs **1427** 全绿
- WPT dom 回归（release runner，test-guard 包裹）：
  - traversal **1593P/11F**（与 R92 逐字节一致；fresh run 的第 12F 是上轮中断 session 遗留在 ignored wpt-data 的 `zz-probe-r93.html` 诊断探针，取证后删除）
  - collections **48P/0F/1neutral**（一致）
  - nodes **6654P/1568F/13neutral**；stash A/B 重建干净 R92 二进制核对：`Node-textContent.html 77P/4F` 与 `Document-createElementNS.html 206P/390F` 在干净基线**同样如此**——非本切片回归（master.md R80/R81 的旧数字已过时，本轮勘误）
- fmt 无 diff；clippy（engine + integration）零警告；pre-commit-guard PASS

## 探针取证（上一轮中断 session 的遗留）

`zz-probe-r93.html`（已被上轮 session 写入 ignored wpt-data）断言输出证实了 Proxy-ctor 桥的可行性信号：bridge ctor 被调用（`customElements.getName(new.target)` 可用）、`Object.getPrototypeOf(el) === new.target.prototype` 在 createElement 后已为 true（R90 路径生效）、`new MyEl()` 直接构造时用户 ctor 体执行（state=5、bump()=6）。这是下轮 Proxy-ctor 桥切片的设计输入。

## 延后

- **Proxy-ctor 桥**（解 lit constructor 内初始化）：R92 延后项，探针信号已取证，下轮候选。
- lit 真库 e2e：Proxy-ctor 桥后。
