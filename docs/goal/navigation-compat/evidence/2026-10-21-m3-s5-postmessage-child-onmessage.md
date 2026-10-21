# M3-S5 parent→child postMessage + 子文档 onmessage 绑定（2026-10-21）

M3 第五切片：the-iframe-element/iframe_sandbox_block_modals-1..4 四案簇
（父窗向 sandbox 子窗投递 modal 名 → 子窗 onmessage 执行 alert/confirm/
prompt/print → 回程 postMessage 报值；断言「无 allow-modals 时调用即时
返回 + 返回值 undefined/false/null/undefined」）。两处 shim 变更，零
runner 变更，零既有测试改写。上轮 429 断点遗留 WIP（part63 测试 +
part01/part05 雏形）经核对语义后收口。

## 根因

子文档裸 `onmessage = function(e){...}` 赋值此前落**父窗 globalThis**
（子脚本体在 `new Function(...)` 内执行，非严格裸赋值沿作用域链穿透到
宿主 global）——R2932 的 body/frameset `on*` setter 面把它注册成**父窗**
监听；子窗 `win.onmessage` 永远缺席。父→子 `contentWindow.postMessage`
投递（part05 子窗 postMessage stub → `win.dispatchEvent(MessageEvent)`
→ `win['on' + type]` 直调）找不到处理器，子窗永不回程 → 上游
`t.step_func_done` 门铃永挂 → 四案 Timeout。

## 实现

- **(a) 子窗 stub 预置 `onmessage: null` 槽**（part05 iframe 子窗对象
  字面量）：spec
  [WindowEventHandlers](https://html.spec.whatwg.org/multipage/webappapis.html#windoweventhandlers)
  onmessage 未设时 getter 返 null；槽位在场是 `with(window)` 裸赋值落
  with 对象的前提（JS 语义：with 对象上**已存在**的属性才被赋值捕获，
  否则穿透到外层）。
- **(b) 子文档脚本条件 `with(window)` 包合**（part01 child script 合成）：
  combined206 命中裸赋值正则 `/(^|[^\w.$])onmessage\s*=[^=]/` 时整体包
  `with(window){ combined206 + exportSuffix206 }`——裸 `onmessage =` 绑定
  子窗属性，处理器经既有 `win['onmessage']` 直调路径派发。正则排除成员
  赋值（`.onmessage =`，前置 `[^\w.$]`）与 `==`/`=>`（后置 `[^=]`）；
  **未命中的子文档脚本文本逐字节不变**（corpus 其余 425 路径零漂移的
  结构保证）。exportSuffix 的 `window["name"]=name` 导出显式引用形参，
  with 块内语义不变。
- 集成测试 `js_dom_bridge_tests/part63.rs`
  m3s5_parent_to_child_postmessage_roundtrip：四段投递断言回程序列
  `["undefined","false","null","undefined"]`（modal 桩 R2979 语义）+
  子文档裸 onmessage 绑定子窗（`contentWindow.onmessage` function /
  父窗 `window.onmessage` null——R2932 误注册面防回环）。

## FIXME 记录（最小面边界）

- with 包合为文本启发式：仅裸 `onmessage =` 触发；子文档
  `addEventListener('message',…)` 走既有 listeners 路径本就正确，不触碰。
- modal 桩恒即时返回（R2979 headless dismiss 语义）——「allow-modals
  在场时应阻塞」的反向行为无实现面；上游四案只断言**无 allow-modals
  即时返回**，桩语义恰好满足（headless 无 UI，真实阻塞上游亦无可执行案）。
- 子→父回程经 `parent.postMessage` 既有 self 投递面；跨源 origin 校验、
  MessageEvent `source`/`origin` 字段按既有 stub 语义（source=调用方
  globalThis=父窗）。
- parent→child 通道不覆盖 MessageChannel port 迁移（webmessaging 域
  已收口，子窗 port 面未触）。

## 验证

- 域内：`make testharness-navigation FILTER=block_modals` 四案
  **Timeout→Pass ×4**。
- 全量 corpus：`make testharness-navigation` **468P/69F/44T**（581 案行）
  对 S4 全量（464P/69F/48T）逐路径 diff：**恰 4 行变化**（block_modals
  1..4 Timeout→Pass），**其余 577 路径逐行恒等零回归**；域内
  the-iframe-element 28P/32F/27T→**32P/32F/23T**。
- 引擎库：`cargo test -p zero-engine --lib` **3358/3358**（含新
  m3s5 案；S4 态 3357 + 1）。
- 跨套件回归臂：`make testharness-webmessaging` **132P/3F**——对已收口
  账本（archive/webmessaging-compat 2026-10-12，133/135，挂账 2）的差项
  全为数据/域面而非本切片行为面：user-activation（navigator.userActivation
  缺失，账本既有挂账）；garbage-collected 与 Channel_postMessage_Blob 两案
  `load support /common/gc.js`——**wpt-data 2026-10-20 整目录重建后
  /common/gc.js 未补拉**（2026-10-20 恢复只补了 navigation corpus 四项；
  css/css-nesting、navigation-api 同消费）。共享资产洞记入 master.md
  待办，不在本切片硬解（归 webmessaging/渲染流账本）。
- fmt 干净（另：`apps/compositor/src/frame_dump.rs` 发现兄弟流 9799e6435
  带入的 fmt 漂移阻塞 T58 前置门——机械 `cargo fmt -p zero-compositor`
  修复，独立提交）；clippy workspace --all-targets -D warnings 零 warning。
- `make test`（test-guard，DISPLAY=:20）76 suites 全绿（见 master.md
  本轮记录）。
- reftest 未跑：变更面（子脚本文本包合 + 子窗槽位）零布局/绘制触达
  （S1~S4 同理由）。

原始输出：[2026-10-21-m3-s5-postmessage-child-onmessage-family.txt](2026-10-21-m3-s5-postmessage-child-onmessage-family.txt)
（block_modals 过滤）、[2026-10-21-m3-s5-postmessage-child-onmessage-full.txt](2026-10-21-m3-s5-postmessage-child-onmessage-full.txt)
（全量）、[2026-10-21-m3-s5-webmessaging-crosscheck.txt](2026-10-21-m3-s5-webmessaging-crosscheck.txt)
（跨套件回归臂）。
