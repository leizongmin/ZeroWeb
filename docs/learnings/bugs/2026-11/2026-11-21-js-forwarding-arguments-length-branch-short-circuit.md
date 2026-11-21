---
date: 2026-11-21
modules: engine
---

# JS shim 转发函数形参恒绑定使 arguments.length 判分支恒真，显式传 undefined 参短路被调方默认查找

## 问题描述

刀 OM-16（typed-om CEReactions 影子派发）v3 电池 19/20 靶过、唯一
interleave 靶 callback 收到 `oldValue=null`。该靶语义：style 属性串与
attributeStyleMap 交替写后派发 attributeChangedCallback，old 值须为属性
串现态（影子优先查找＋attr 回落）。静态审查转发链、影子表、值门均无缺陷，
三处派发全部带错误 old。

## 根因分析

转发包装函数的形参是**恒绑定**的：

```js
function _zwStyleCeSyncDeferred(sel, handle, text, realOld) {
  if (_zwStyleCeWindowDepth <= 0) {
    _zwStyleCeSync(sel, handle, text, realOld);  // ← 恒 4 参
  }
}
```

被调方 `_zwStyleCeSync` 以 `arguments.length >= 4` 判「调用方是否传了
真实 old」走 realOld 分支（否则影子优先查找）。转发侧哪怕调用方只传 3 参，
`realOld` 形参仍存在（值为 undefined），显式转发后 `arguments.length`
恒为 4 → 恒走 realOld 分支，realOld=undefined 归一 null → 影子优先查找
永不执行。这类分支判据在 JS shim 里常见（可选参数语义），包装层显式转发
即破坏它。

## 解决方案

条件转发——未收到第 4 参时不得显式传 undefined：

```js
if (arguments.length >= 4) _zwStyleCeSync(sel, handle, text, realOld);
else _zwStyleCeSync(sel, handle, text);
```

## 避免方式

- 写转发包装函数前先查被调方是否用 `arguments.length`/`in` 判可选参
  分支；是则条件转发，不透传未绑定形参。
- 此类 bug 静态审查难以命中（语法、数据流都「正确」），怀疑可选参链路
  时直接加 trace 插桩打印实参个数定谳，比逐层读代码快。
- 同族陷阱：显式传 `null` 当「缺省」也不等价于不传——判分支若区分
  absent/null（如本刀共享门 absent≠''），透传语义须逐参核对。
