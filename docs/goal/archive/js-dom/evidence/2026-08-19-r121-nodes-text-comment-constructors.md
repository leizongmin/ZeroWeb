# R121 — M4 nodes：Text/Comment 构造器 + CharacterData 孤立代理保真（三文件全 100%，+32 净）

**日期**: 2026-08-19
**里程碑**: M4（WPT dom 上游基线建立与扩展）
**驱动用例**: `Text-constructor.html`（13F→15P）、`Comment-constructor.html`（13F→15P）、`CharacterData-surrogates.html`（6F→8P）——三文件 100%
**规范**: https://dom.spec.whatwg.org/#dom-text / #dom-characterdata-replacedata

## 结果摘要

| 路径 | 前 | 后 | 净 |
|------|----|----|----|
| polyfill nodes 全量 | 7599P | 7631P | +32（44F→P，零新增 fail） |
| native nodes 全量 | 6112P | 6144P | +32（同步） |

traversal 1595 / events 419 / collections 48 不变。

## 根因与修复（两层）

1. **Text/Comment 构造器是空 stub**（`function Text() {}`——`new Text().data` undefined、
   instanceof 原型链断，两文件 26F）。真构造器经 `_zwMText`/`_zwMComment` 轻量构建器：
   `data === undefined ? '' : String(data)`（null→'null'）、`ownerDocument` getter →
   document、`setPrototypeOf(n, Text.prototype)`（原型链 Text→CharacterData→Node 三层
   instanceof 真）。R108 的 dispatchEvent（prototype 上）保留。
2. **CharacterData 孤立代理在 wire 层变 U+FFFD**（R118 记档同族的可达绕过）：spec 允许
   replaceData/deleteData/insertData 按 **UTF-16 code unit 偏移切开代理对**，切开的
   孤立代理在读回时保真（WPT surrogates：`"\uD83Cst 🌠 TEST"` 非 FFFD）。wire
   （`to_rust_string_lossy`）必然替换——**JS 侧覆盖缓存** `_zwTextDataCache`（Map，键
   handle）：写双写（JS 保真 + wire 尽力供 host 渲染），读缓存优先（miss 回落 wire）。
   十处统一接线：data/nodeValue/textContent/wholeText/length 读 + setter +
   appendData/deleteData/insertData/replaceData/substringData。

## 验证

- 三 driving 用例 polyfill/native 双路径全绿（15/15/8P）
- engine 单测 `test_text_comment_constructors_and_surrogates_r121`（6 断言组：构造器
  参数转换/ownerDocument/三层 instanceof/孤立代理保真/方法族组合）
- `make test` 65 套件全绿 exit 0；fmt 无 diff；clippy `-D warnings` 零警告
- 账本：`tests/wpt-runner/imported-tests.txt`（R121 条目）

## 设计注记

- 覆盖缓存是 R118「wire 协议孤立代理」深结构的**局部可达绕过**：text/comment 的 data
  语义面（CharacterData 族）在 JS 侧保真，host 渲染侧接受 FFFD（渲染孤立代理无意义）。
  wire 协议的 WTF-16 保真改造（全回调面）仍是深结构记档项。
- `_zwMText` 原型链原本指向 Node.prototype——构造器路径显式 setPrototypeOf 到
  Text.prototype（parsed 文本节点的链不迁移，两路径语义独立）。
