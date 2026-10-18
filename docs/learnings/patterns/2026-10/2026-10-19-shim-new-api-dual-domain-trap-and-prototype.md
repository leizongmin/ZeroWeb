---
date: 2026-10-19
modules: js_dom_shim
---

# shim 新 API 需 trap 分支 + 原型回落双层，否则克隆树/实例域元素静默缺方法

## 问题

pointerlock-compat M3 s4 落地 FR-7 shadow retarget（`pointerlock_shadow` WPT
案）时，`requestPointerLock` 与 `ShadowRoot.pointerLockElement` 只实现在
`_makeProxy`/`_wrapHandle` 产物（sel/handle 代理）的 proxy get trap 分支上，
实跑整案 `expected null but got undefined`。逐层探针定位：`createTestTree(host)`
惯例（`walk(node.cloneNode(true))` + 逐 template `attachShadow`）构建的整棵
克隆树是 **R307 实例域**——内部节点是普通 JS 对象，`__zwHandle === undefined`，
属性/方法解析走 shim 自建原型链（`_zwBuiltNodeChain`），**从不经过 proxy trap**。

两处静默失配：

1. `canvas.requestPointerLock()`：trap 分支只服务代理元素，实例域 canvas 读到
   undefined（若实跑走到会 TypeError）。本次红在更早的默认 null 面，故表现为
   undefined 而非 not-a-function。
2. `attachShadow` 双路径分族：克隆树父节点无 handle → `parent.attachShadow`
   解析到 `Element.prototype.attachShadow` 轻量路径（part03 R179），产物是
   `[object Object]` 平面对象（无 handle、无 trap）；同一函数里克隆树根节点
   （`cloneNode` 返回值，有 handle）走 part04 trap 路径产 `_wrapHandle` 代理。
   `ids.root` 与 `ids.root2` 因此分属两族，`pointerLockElement` 只在 trap 族
   有答。

## 根因

同一 DOM 语义在 shim 里有两条属性解析路径：proxy get trap（sel/handle 代理域）
与 shim 自建原型链（R307 实例域 + 轻量 shadow）。新 API 只落 trap 分支 = 只
覆盖「从活树查询面拿到的代理」；`cloneNode`/`importNode` 产物、innerHTML 解析
经 named-prop 注册的节点、detached 树——全部经原型链解析。learning #32 的
多族并存不是边界情形，克隆树惯例（WPT shadow-dom 测试的标准写法）天然整树
落入实例域。

`_elKey(null, null)` 返 null 还有次生坑：实例域元素无代理坐标，锁态键若直接
沿用 `_elKey` 会把「已锁」真值判定毁掉（`_plKey = null` 即未锁）——需换对象
身份盖章（`_ceConnKeyFor` 的 `plain:p<N>` 模式）。

## 解决

新 API 按「双层」落地，参照 `attachShadow` 既有先例：

- **trap 分支**（服务代理域）保持，grant 流抽成共享函数（如 `_plRequestLock`），
  两个调用点同层语义，不复制 ①-⑤ 校验链。
- **`Element.prototype` 原型回落**：guard 幂等
  （`typeof ... !== 'function'`）+ `own` 委托检查（`getPrototypeOf(this)` 上有
  非 prototype 自身的 own value 则委托——照抄 part03 attachShadow 原型面形态）；
  defineProperty 不带 `enumerable`（WebIDL 原型成员缺省非 enumerable，R3046
  for-in 零变化先例）。
- **`ShadowRoot.prototype` accessor**：轻量 shadow 的 `setPrototypeOf` 目标即
  此，一个 accessor 同时覆盖所有轻量产物；代理域由 trap 先答（proxy trap 先于
  原型查找），不达本面，无双答。
- 需要按「节点身份」记账/比对的键，实例域用对象盖章（`_ceConnKeyFor`），不
  用 `_elKey`。

## 验证方式

探针链定位族属：`Object.prototype.toString.call(x)`（`[object HTMLDivElement]`
= 代理族 / `[object Object]` = 实例族）、`x.__zwHandle` 有无、
`x.attachShadow === globalThis.Element.prototype.attachShadow`（true = 原型轻量
路径将接管）。修复后 `grep -ac <特征串> target/release/zero-wpt-runner` 核验
shim 进了最终二进制（learning #28），再跑语料全案。

## 关联

- learning #32（js_dom_shim 多族包装 identity）——本案例是其「克隆树整树实例
  域」的具体化与新 API 落点规则。
- learning #28（执行体以最终链接产物为证据）。
