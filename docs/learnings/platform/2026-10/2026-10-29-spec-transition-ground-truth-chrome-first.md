---
date: 2026-10-29
modules: engine,tests/wpt-runner
---

# 规范迁移期 JS 语义以 Chrome 实测为地面真值——node 旧 V8 是反例

## 问题

M2-S16 收敛 `ReadableStream.from` 时，从 2026-10 HEAD 的 streams spec +
WebIDL 抽取的「async sequence」原语文本中，「converting V to an IDL value
of type T（T=any）」一步字面不含 value 展平（await）；而 WPT 语料
`from.any.js` 要求 `from([Promise.resolve('a')])` 读出 `'a'`（展平）。用
node 22 实测 `ReadableStream.from`：array of promises **不展平**、
`from('ab')` **不抛 TypeError**——两处都与语料期望相反，spec 字面与 node
实测互相「印证」成一个错误结论。

## 根因

该面处于 WHATWG 规范原语迁移期（GetIteratorFlattened 旧模型 → WebIDL
async-sequence 新模型）。node 22 内嵌的 V8 落后于 Chrome 154（约 20 个
Chrome 大版本），实现的是**旧代模型**：`ReadableStream.from('ab')` 收留
字符串（旧 GetIterator 不做 Object 门）、value 不展平。spec 文本本身也仍
在演进（WebIDL HEAD 的 get-next-value 步字面尚未反映 Chrome 的展平行为，
或由「IDL any 转换」的实现约定承载）。拿「旧引擎 + 演进中文本」互证，会
把两边的滞后叠成看起来自洽的假地面真值。

## 解决方案

验收标尺固定为 **WPT 语料 + 目标档位 Chrome 实测**，spec 文本只作步骤
结构与腿归因参考：

1. spec 抽取定位语义分歧点（本例：value 转换步是否 await）后，直接写
   DOM 探针页在仓库 oracle Chrome（`google-chrome --headless=new
   --dump-dom`）上标定争议腿——Chrome 154 实测 `from('ab')` → TypeError、
   value 展平，与语料一致。
2. node / 旧 V8 探针结果只用于暴露分歧，不用于裁决；版本号差（node 22 ≈
   Chrome 134 代）先查再信。
3. 一致后按 Chrome 行为实现，并在实现注释里记「Chrome xxx 实测」口径，
   不留 spec 字面依赖（spec 落后于实现是迁移期常态）。

适用面：所有 WHATWG/TC39 迁移期特性（async sequence、Iterator helpers、
explicit resource management 等）；zeroweb 的 WPT pin 语料 + Chrome 对拍
是既有验证模式，分歧腿多出在「spec 文本 ≠ 落地行为」的窗口期。
