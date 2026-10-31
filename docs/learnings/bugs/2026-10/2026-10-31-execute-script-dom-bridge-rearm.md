---
date: 2026-10-31
modules: webview,js_dom_bridge
---

# 裸 execute_script 逐调用重注册 DOM 桥撞毁 iframe 文档异步物化（form 面 0P 根因）

## 问题描述

wptfyi weekly 全量对账 form-url-encoding 面（encoding/legacy-mb-*，约 112 万
子测试）整面 0P，头条通过率钉在 42.1%。失败签名清一色为 Timeout：
`testharness completed without reporting registered tests
state={"loaded":true,"pending":1000,"phase":4}`——页面已 loaded、注册的
1000 个 async_test 一个都没完成，全部挂到 testharness.js 页级超时被强收。
同一 runner、同一语料，decode 面（encoding 与 legacy-mb decode 簇）正常出分。

## 根因分析

`60ce14698`（T26 Cache API 修复）把裸 `WebView::execute_script` 入口改成
**每次调用**都执行 `register_dom_callbacks`。而 runner 的探测环
（tests/wpt-runner/src/testharness.rs probe loop）恰恰高频裸调用
execute_script 轮询 harness 状态（数十次/秒）。

`register_dom_callbacks`（crates/engine/src/js_dom_bridge/callbacks.rs）的
语义是「注册即 dom_html 换代」：发布新 `Arc<Mutex<String>>` dom_html 快照 +
`clear_generation_cache()` + `_zwChildBaseInvalidateAll` /
`_zwSiblingBaseInvalidateAll` 全量基底缓存失效。探测环每次调用都重绑这套
状态，encode-form-common.js 依赖的 iframe 文档**异步物化**被反复打断：
`form.submit()` 后 target iframe 的导航 onload 永不派发 → runNext 链断 →
1000 个 async_test 全 pending → 页级超时 phase 4 → s149 早收。

三个放大器：

- **常规页不受影响**：`run_page_scripts` 每页只武装一次，问题只在「裸入口
  高频调用」路径显形——手工单次 execute_script 一切正常。
- **失败形状误导方向**：Timeout + 0P 看似 runner 超时/早收（s149）问题，
  嫌疑先落在探测环裁决逻辑上，实际页面 JS 从未真正开始跑测试。
- **语料缺 blank.html 假象**：排查时先撞上语料目录缺文件，但 runner 内嵌
  `/common/blank.html` fixture（testharness.rs `clean == "common/blank.html"`），
  语料层白查一轮。

## 解决方案

`dom_bridge_host_armed` 旗标（修复 `84811327c`）：裸入口首次调用完整武装
（publish cursor + register_dom_callbacks + identity/pointer 桥），稳态调用
只保留尾部 `apply_pending_shared_mutations()`；`ensure_sandbox` 重建沙箱时
复位；`run_page_scripts` 每页常规武装不受影响。

注意 tail apply **不能一并 gate**：`e31527218` 的 mixed-batch 两步 pin 依赖
逐调用 tail apply，trial 中曾把尾部 apply 也只在首次执行，对应测试立刻红。

两条规则：

- **带副作用的注册/武装操作不得放在逐调用入口的 hot path**。「武装一次」
  语义要显式旗标化，且各类入口（页面脚本/裸调用/沙箱重建）的复位时机各自
  写明。
- **面级 0P + `loaded:true` + pending 等于注册总数** → 先分辨「测试没跑」
  还是「跑了全挂」：探测环 `take_probe` 后 eprintln state，并在可疑 JS 里埋
  `__dbg_submit`/`__dbg_onload` 旗标、经 execute_script 读回——一轮即可定位
  断在哪环（本例 submit=2、onload=0 直接钉死 iframe onload 未派发）。
