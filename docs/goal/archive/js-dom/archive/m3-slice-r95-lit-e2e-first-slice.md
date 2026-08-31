# M3 切片 R95 — 真实 lit 库 e2e 首切片（组件链 + template.content）

**日期**: 2026-08-17
**Milestone**: M3 Web Components 端到端（DC-2）
**前置**: R94（Proxy-ctor 桥——lit 依赖面的前提）

## lit 引入

- **来源**：上游官方发布产物（jsdelivr CDN 快照）——`@lit/reactive-element 2.1.1` +
  `lit-html 3.3.2` + `lit-element 4.2.1`（BSD-3-Clause，license 头保留在 bundle 内）。
- **形态**：五模块（css-tag/reactive-element/lit-html/is-server/lit-element）打包为
  单一 classic script——ESM import/export 剥离 + 每模块独立 IIFE 作用域 + 显式绑定
  交接（reactive-element 需要 css-tag 的 getCompatibleStyle/adoptStyles；lit-element
  需要 ReactiveElement/render/noChange）。产物 `fixtures/lit/lit.bundle.js`（~17.8KB），
  页面侧经 `globalThis.lit` 消费，与 `import { LitElement, html } from 'lit'` 组件代码
  面等价。
- **选择 bundle 而非原生 ESM**：shim 的 es_module 转换器不处理 minified
  `import{...}from"..."`（无空格形式，lit 发布产物标准形态）——bundle 是零引擎改动的
  等价路径（node 侧全套探针验证 bundle 语义与模块原形一致）。

## 真实 lit 暴露的两个 shim 缺口（均已修）

1. **`constructor` 读（part03 get trap 顶部短路）**：lit ReactiveElement 的实例方法
   `_$E_` 读 `this.constructor.elementProperties`——旧 get trap 对 'constructor' 落到
   中间分支返 undefined → `undefined.elementProperties` TypeError → **ctor 链中断，
   用户 ctor 体不执行**（R93 通用回落太靠后赶不上）。修复：trap 顶部原型链 own 命中
   （8 层）短路。
2. **`<template>`.content（part04 get trap）**：lit-html Template.createElement 路径
   `createElement('template'); t.innerHTML=html`，随后 `t.content` 取解析子树走 parts
   管线——content 缺失则整个 render 管线死寂。修复：轻量 fragment 视图（nodeType 11 /
   childNodes 直读 innerHTML 解析树 / firstChild/lastChild 派生）。

## 验收资产（make test 内）

- **组 A `lit_component_chain`**：bundle 在 shim 完整求值；LitElement 子类 define +
  createElement 升级；ctor 体以元素为 this 执行（R94 桥）；`el.constructor === 用户类`
  （R95 短路）；instanceof LitElement/HTMLElement 双层；lit 内部状态（_$ES/renderRoot）；
  shadow root 建立。
- **组 B `template_content_fragment_view`**：fragment 语义（nodeType/nodeName/
  childNodes/firstChild/空模板）。

## 验证

- integration **777**（+2 lit）/ engine v8 **2188** / quickjs **1427** / webview **601** 全绿
- WPT dom：nodes/collections/traversal per-case **逐字节一致**；events **净 +46P**
  （`passive-by-default` 整用例崩溃（"page script threw"）→ 执行 100 subtest 43P——
  constructor 修复解锁页面执行；`disabled-elements` 0P→3P）
- fmt 无 diff；clippy 零警告；pre-commit-guard PASS

## 诊断归档：异步 update 链剩余缺口（下一切片输入）

- **现象**：lit 的 `await this._$ES` 永不 resume——首渲染不落地（shadow root 恒空）。
- **根因链**（三轮探针实证）：ctor 内 `this.enableUpdating = resolveFn`（Promise
  executor 赋值，经 set trap 落 expando）→ connectedCallback 读
  `this.enableUpdating` 得**原型 noop**（`function(){[native...`）→ resolve 不发生。
- **expando-first 尝试（已回退）**：把 expando own 读提到 trap 最顶——group A 反而回归
  （`this.hasOwnProperty is not a function`）。暴露第二层缺口：**`hasOwnProperty` 在
  get trap 长链上本身不可达**（bisect：part03 的 53 个 prop 分支标记 0..52 全部到达，
  part04 的 expando/R93 计数器从未到达——中间有分支吞掉了控制流，具体位置在
  part03→part04 交接区域，待查）。
- **define 期 observedAttributes（lit finalize 触发）尝试（已回退）**：elProps 0→2
  （finalize 生效）但立即暴露上述 hasOwnProperty 缺口（`_$Ei` 内
  `this.hasOwnProperty(sym)`）。
- **下一切片计划**：① 定位吞控制流的分支（在 part04 区域加密 bisect 标记）；
  ② expando own-shadow 语义（修 enableUpdating 类实例属性）；③ 重 land define 期
  finalize；④ reactive property round-trip + 首渲染断言（`hits/1 then 42` 形态已备好）。

## 对 M3 的意义

DC-2 的「lit 之一」验收面从零到有：真实 lit bundle 在 ZeroWeb shim 上求值、组件
定义/升级/ctor/shadow root 全链路通过。剩余 = 异步 update 链（上记诊断）——
预计 1-2 个切片内收口。
