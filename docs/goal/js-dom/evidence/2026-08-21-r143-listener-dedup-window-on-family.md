# R143 — handler-count 2F→0F（重复 listener 丢弃 + window on* 全族 IDL 属性）

**日期**: 2026-08-21
**里程碑**: M4（WPT dom 上游基线扩展）
**驱动用例**: `dom/events/handler-count.html`（2 subtest，R142 Actions 白名单放行后新可见）

## 根因（两层）

1. **重复 listener 不丢弃**：spec「add an event listener」步骤 4——同 target 已有
   （type, callback, capture）相同的 listener 时静默 no-op。shim 四个注册面
   （`_globalAddEventListener` window / document.addEventListener / element proxy
   addEventListener / `_zwMEl` plain-node addEventListener）全部无条件 push →
   "Duplicate listener is discarded" 断言 3≠2。
2. **window on* 全族缺失**：`_defineWinOnHandler` 只覆盖 WindowEventHandlers 子集
   （load/error/message/...）+ focus/blur，缺 GlobalEventHandlers 的鼠标/键盘/输入/
   动画/过渡等全族——`window.onclick = fn` 落 plain 属性，派发不触发（"After adding
   listener expected 1 but got 0"）。

## 修复（part03/04/06）

- 四个注册面各加 dedup（同 type + 同 fn + 同 capture + 同槽位 tgt → return）：
  - part03 `_globalAddEventListener`（window，tgt='win'）
  - part06 `document.addEventListener`（tgt='doc'）
  - part04 element proxy `addEventListener`（tgt=undefined 元素槽位）
  - part03 `_zwMEl` plain-node `addEventListener`（按 node._zwEvLs）
- part06 `_defineWinOnHandler` 类型表扩到 GlobalEventHandlers 全族
  （click/dblclick/auxclick/contextmenu/mouse*/pointer*/key*/input/beforeinput/change/
  submit/reset/wheel/drag*/copy/cut/paste/media 全族/animation*/transition*/toggle/
  slotchange/securitypolicyviolation/scriptexecute 等 ~70 类型）——
  setter 移旧注册新、getter 返存储 fn（既有 R2932 机制，纯表扩展）。

## A/B 结果（polyfill / native 双路径）

| 套件 | 结果 |
|---|---|
| handler-count | **2P/0F 双路径 100%** |
| dom/events 全量 | 417P/32F（fail 集 vs R142：仅 handler-count 消失，零新增） |
| dom/traversal | fail 集与基线一致（50P/6F） |
| dom/collections | 49P/0F 全绿 |
| `make test` | 66 套件全绿（双矩阵） |
| fmt / clippy | 零 diff / 零警告 |

## 单元测试

`test_listener_dedup_and_window_on_family_r143`（part20.rs，6 断言段）：window 重复
丢弃 / capture 不匹配非重复 / document 丢弃 / element 丢弃 / window.onclick 替换
语义（旧不触发新触发 + typeof function）/ onclick=null 移除。

**过程教训**：Rust 字符串 line-continuation（`\`+换行）与 JS `//` 行注释互斥——
注释会把合并行后的全部剩余脚本吞掉（首个 // 后整段失效，完成值落 undefined）。
单测 JS 载荷内不用 `//` 注释（既有测试全遵守，本次首版违反即触）。
