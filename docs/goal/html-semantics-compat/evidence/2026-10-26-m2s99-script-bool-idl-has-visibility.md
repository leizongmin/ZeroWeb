# M2 切片 99：HTMLScriptElement 布尔反射 IDL 的 `in` 可见性（has trap tag 门）

日期：2026-10-26。范围：`crates/engine/src/js_dom_shim/part05.js` 元素 proxy has trap——nomodule-set-on-async-classic-script 2F 销账。上轮：[M2 切片 98](2026-10-26-m2s98-write-harness-runner-face.md)。

## 1. 入场核账与定谳

- s98 基线（scripting-1 口径 574 行）：312P/229F/15T + 18 NotRun。
- nomodule-set-on-async-classic-script 两行同报 `assert_true: expected true got false`——两测试首断言同为 `assert_true(supportsNoModule)`，`supportsNoModule = "noModule" in document.getElementsByTagName("script")[0]`。若 load 门（`window.onload = resolve` promise）失灵则两测试应为 Timeout 而非断言失败——门正常，缺口在 `in` 可见性。
- 根因：元素 proxy has trap（part05 生效版，R129 定谳）白名单无 _REFLECTED_BOOL 面——get trap（part04:11747 R3038/R3040）值面覆盖全表、has 探测恒 false（既有 documented 限制「'id' in el 仍 false」的同类面）。
- 先例口径：R5249（track/option label tag 门）/ R5266（tr 族 tag 门）/ 尾簇 33（'value' tag 门）——接口成员归属按 tag 门控，防无条件列名假阳性（R5249 menuitem 实证）。

## 2. 接线

has trap 白名单新增 R5300 分支：`async`/`defer`/`noModule` 三名 **tag 门 SCRIPT**（`_realTag` ':' 后缀容差同 R5266）。全员表列名会重蹈 R5249 假阳性；三名恰为 HTMLScriptElement 接口成员（值面 R3038/R3040 presence 读已覆盖，spec：`noModule` reflected boolean，https://html.spec.whatwg.org/multipage/scripting.html#dom-script-nomodule）。

## 3. 验证

- FILTER=nomodule：nomodule-set-on-async-classic-script **2F→2P**——测试 1（noModule=false）四断言全过（动态 appendChild src 执行 + onload 投递 + load 门 resolve）；测试 2（noModule=true）!executed/!loaded/!errored 全过（s96 prepare 门阻塞执行且不投递 load/error）。
- 全子树终扫（scripting-1 口径 574 行）：**314P/227F/15T + 18 NotRun**，净 +2 Pass，行级 diff **仅本两行 F→P**、零 P→非P（/tmp/s98-s1-rows.txt vs /tmp/s99-s1-rows.txt）。
- zero-engine lib **3530/3530**（guard）；shim 拼接 node --check OK。
- 纯 shim .js 变更：fmt/clippy 不涉 render/layout 面，reftest/product-smoke 沿 s91-s98 口径未跑。

## 4. 范围与挂账

- 触碰面：part05.js ×1（has trap 白名单一分支）；runner/语料零变更。
- nomodule 族残余 2F（另族根因，归 load-event 保真切片）：①nomodule-set-on-synchronously-loaded-classic-scripts「must run」——parser 外链被 runner 内联投递时 `onload`/`onerror` 内容属性被丢弃（s94 保真只留 nonce/nomodule/type），load 不投递 → `assert_true(loaded)` 假红（脚本本身执行到位）；②nomodule-set-on-external-module-script——module 编译错误（缺 default export）冒出为 page script threw，spec 语义须落 Script error 事件面。
- 承袭：116 frameset、063 Timeout、module 尾片（.py 前置）、charset 12F（.py）、svg-script-fragment-clone 3F、load-error-events 族（脱离树副本元素事件保真——与本轮残余①同族汇入下一刀）、FIXME 组、接口对象基建面。
