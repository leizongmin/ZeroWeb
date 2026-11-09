---
date: 2026-11-07
modules: crates/engine
---

# element proxy 的 get trap 拦截全部属性读——原型侧 accessor 对 proxy 恒不可达

## 问题描述

svg-compat-v2 B6b（svgstring-1）排查中实证：给 `SVGAElement` 接
`target` 属性（SVGAnimatedString，SameObject 语义）时，先按传统思路在
原型上挂 accessor（`Object.defineProperty(SVGAElement.prototype,
'target', {get...})`），WPT 断言 animVal 恒为 `undefined`——accessor
从未执行。同样的 accessor 逻辑移入 element proxy 的 get trap 中间分支
（per-key SameObject 缓存）后立即通过。

## 根因分析

element proxy 的 get trap 里已有一组「通用反射」中间分支，在尾部回落
`Reflect.get` 之前就把 `prop === 'target'` 这类属性名读拦下并按内容属
性直答。JS proxy 语义：get trap 一旦返回值（不回落 Reflect.get），
原型链查找根本不会发生。于是原型 accessor 是死代码，且无任何报错——
只在断言失败时才显形。

这与 [shim 新 API 需 trap 分支 + 原型回落双层](../patterns/2026-10/2026-10-19-shim-new-api-dual-domain-trap-and-prototype.md)是互补而非同
一条：那条说方法需要「trap + 原型」双层保证克隆树可达；本条说的是反
向陷阱——**属性读侧原型单层恒死**，因为 get trap 的中间分支先行拦截。

## 解决方案

- SVG 反射属性接线统一走 get trap 中间分支：命中即直答，配 per-key
  SameObject 缓存（`className` 分支是现成模板——缓存键 `_elKey(sel,
  handle) + prop`，value 缓存 AnimatedString 实例）。
- 原型侧不挂同语义 accessor（避免双实现漂移）；基类实例（非 proxy）
  若确需 accessor，另行确认读路径无 trap 拦截。

## 验证方式

在原型 accessor 内放一次性日志/探针即可证实「从未执行」；接线到 get
trap 后 `svgstring-1` animVal 断言转 P，全通道零 P→F。

## 避免方式

- 给 shim 元素新增反射属性时，第一落点永远是 get trap 中间分支（先查
  通用反射分支是否已把该属性名吞掉）。
- 「属性没生效且无报错」的排查第一假设：原型路径被 trap 拦截，探针打
  在原型 accessor 上验证是否执行。
