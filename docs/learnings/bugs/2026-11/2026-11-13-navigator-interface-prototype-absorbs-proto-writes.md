---
date: 2026-11-13
modules: engine
---

# 暴露给页面 JS 的平台对象必须带接口原型——平面对象让原型定向写污染 Object.prototype

## 问题描述

T65（site-optimizer perf-mem-20261023）诊断链定谳：掘金 rc-verifycenter/bdms 风控 SDK
在 zw 上每轮必中同一失败签名——cookie 链断（s_v_web_id / `__tea_cookie_tokens_*` 全缺）、
feed 45s 满窗 cards=0；chrome 同站同 SDK 通过。zw 页面行为与 chrome 完全一致（最小复现
页 caseB 双臂同 THROW），分叉点不在脚本执行路径，而在接收者品牌。

## 根因分析

bdms.js（VM 混淆 bundle）含「从根对象沿 `.p`（原型）走 s 步得到目标对象，再以键存储」
的**原型定向写** opcode（活体栈 + 分发环字节实拍，双臂无条件执行）。落点由接收者的原型
链决定：

- chrome：`navigator` 的 `__proto__` 是 `Navigator.prototype` → canary 落接口原型对象，
  任何页面对象的 `for-in` 都看不到（不在普通对象的原型链上）→ 无害吸收。
- zw：`navigator` 是平面对象，`__proto__` 即 `Object.prototype` → canary 落
  **Object.prototype 可枚举数据属性**，全局可见。页面自带 js-cookie 写循环
  `for(var s in i)` 枚举到继承键 `vendorSubs`，对非字符串真值 `{ink}` 调 `.split`
  抛 TypeError，被页面 try/catch 静默吞 → cookie 写入链断裂 → SDK init 停摆。

直接判别读数（probe6，about:blank 零站点干扰）：`Object.prototype.toString.call(navigator)`
zw `[object Object]` vs chrome `[object Navigator]`；同一 `navigator.vendorSubs={probe:1}`
的 setter 帧名 zw `Object.set` vs chrome `Navigator.set`（V8 从接收者原型链推断帧品牌）。

教训：shim 暴露给页面 JS 的全局平台对象（navigator/location/history/storage 家族）若只建
值平面对象、不挂接口原型，任何 SDK 的原型定向写都会落 Object.prototype——污染面是
**所有页面对象**，且错误在远处的 for-in 消费点爆出，与写入点跨站点脚本分离，无法从
症状定位到落点。

## 解决方案

`js_dom_shim/part02.js`（T66，commit de8c25511）补标准面三件套：

1. `window.Navigator` 接口对象：`function Navigator() { throw new TypeError("Illegal
   constructor"); }`（WebIDL §interface-object，无 [Constructor]）。
2. 接口原型品牌：`Object.defineProperty(Navigator.prototype, Symbol.toStringTag,
   { value: 'Navigator', configurable: true })`（w/e 缺省 false 与 WebIDL 形一致）。
3. 平面对象升级：`globalThis.navigator = Object.assign(Object.create(Navigator.prototype),
   {...原字面量})`——own 键序/可枚举性/值恒等，仅 `[[Prototype]]` 换位。

验收钉（`js_dom_bridge_tests/part95.rs` `navigator_platform_object_t66`）：构造器类型、
`getPrototypeOf(navigator) === Navigator.prototype`、`[object Navigator]` 品牌、
`for (var k in {})` 零继承键（污染形态不复发）、own 键面不变。T66 probe6 双臂对照四项
全 SAME、活体 zw 臂 enumProto 全程空 + cookie 链落地 + feed 挂载。

避免方式：shim 新增任何页面可见全局对象时按 WebIDL 接口对象/原型对象语义对齐（构造器
暴露、`@@toStringTag` 品牌、原型链挂接三件一起做）；判别探针用「接收者品牌 + 帧名 +
自然落点」三读数，单看属性值定位不到落点分叉。

## 参考

- spec：HTML §the-navigator-object；WebIDL §interface-object / §interface-prototype-object
- 证据：运行目录 perf-mem-20261023 evidence/t65-diagnosis.md（定谳）、bench/results/t65/
  （probe4/5/6 对照）、bench/results/t66/（修复后验收）
