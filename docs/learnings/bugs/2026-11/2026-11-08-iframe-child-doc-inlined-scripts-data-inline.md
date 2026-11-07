---
date: 2026-11-08
modules: crates/engine/src/js_dom_shim, tests/wpt-runner
---

# shim 侧 iframe 子文档脚本段的 src 检测在 runner 语料面恒空——s261 预内联投递剥 src，须读 data-inline

## 问题描述

s339（iframe 域动态 import 模块体 window 身份桥）在 `_zwRunIframeScripts`（js_dom_shim part01 R206 通道）注入注册包装段时，以「partSrcs 槽记 `<script src>` URL + `/testharness/` 正则命中 harness 段」做注入锚点检测。注入代码进了 release 产物（strings 可见），但 corpus 面（dynamic-imports-credentials 族等全部 runTestsFromIframe 形态）12 子测零变化——探针钉出桥位在测试体执行窗恒 undefined、注册 API 是原版 harness 函数（包装未生效）。

## 根因分析

仪表化（channel 计数 + partSrcs join + body 转储）定谳：runner 投递给 iframe 子文档的 body 里，**外链脚本已被预内联**——`<script src="/resources/testharness.js">` 变形为

```html
<script data-zw-inlined data-inline="resources/testharness.js">/*原 harness 源码*/</script>
```

（s261 语料投递形态，engine extract.rs `inlined_src` 与 webview.rs 按 data-inline 恢复原始 src 语义——shim 提取循环是唯一漏网面）。该形态下 `\bsrc\s*=` 正则不命中 → 落 inline 分支 → partSrcs 全空 → harness 检测恒 -1 → 注入永不 splice。探针另一证据：`promise_test` 在合并 Function 内是 **IIFE 内部绑定**（harness 自包裹），裸赋值/裸引用走作用域链落宿主全局——这正是 s147 注记的既有承重契约（嵌套 test() 落宿主 harness），注入包装若不配恢复会跨页泄漏。

## 解决方案

1. 提取循环 inline 分支加 `data-inline` 属性读取作 partSrcs 槽值（槽语义 =「原外链 URL 或 ''（真内联）」），锚点检测不变。
2. 注入包装对宿主全局 `promise_test`/`test`/`async_test` 的写入，在 `fn.call` 外配 pre-call 快照 + finally 恢复（注册在 fn 同步窗内已完成、体已被包装，恢复不撤已注册面）。
3. 排查手法沉淀：shim 注入不生效时，先 `strings <bin> | grep` 核验代码在产物内（learning 2026-10-07 同款），再做**通道仪表化**（对两个 iframe 脚本通道各埋计数/early-return/partSrcs 转储）+ **setter 陷阱**（Object.defineProperty 捕获全局写入者栈）——一步定位「谁在哪个通道、以哪条语句」写了全局，避免对多通道 iframe 面做静态推演。

## 避免方式

shim 内凡按 `<script>` 属性分类子文档脚本段的代码，src 判据必须同时认 `data-inline`（s261 形态）；两类形态的判据差异在主文档管道（extract.rs/webview.rs 已双认）与 shim 通道之间此前未对齐。
