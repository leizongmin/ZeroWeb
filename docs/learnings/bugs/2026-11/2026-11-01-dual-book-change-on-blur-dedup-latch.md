---
date: 2026-11-01
modules: js_dom_shim,renderer
---

# change-on-blur 双账并存：shim 新臂与宿主 legacy 账同失焦双派 change

## 问题描述

ZRG 2026-10-31 巡检 parity 门捕获：generic-page toggle/button 步事件序失配——
同一 blur 失焦 ZeroWeb 派 2 次 `change#name`，Chrome 1 次（r34@6cc1ecbe5 同面
PASS，定谳窗口内新回归，双腿复现）。页面消费面（jQuery `.change()` 等监听）
对同一交互收到两次回调。

## 根因分析

form 控件的 change-on-blur 存在**两条独立派发账**：

- shim 臂（s128 193bde7cb + s129 23717fa34，2026-10-28）：`part04` blur()/
  焦点迁移臂、`part06` `__zw_host_blur` 臂，按 `__zw_text_change_due` 净值
  判定派 change；
- 宿主 legacy 账：renderer `runtime.rs` `blur_target`/`sync_js_blur` 在
  `value_changed` 时经 `page_scripts::dispatch_dom_event` 补派 change。

两条账各自正确、各自带净值去重，但**互不感知**：s128/s129 落地时只加了
shim 臂，没有审计宿主账是否已覆盖同一失焦 transition——同一次失焦两账各派
一次。回归暴露延迟 3 天的原因：parity 事件序门只在巡检轮跑，窗口内的
make test/wpt 面没有覆盖「shim 臂+宿主账同 transition」的组合形态。

## 解决方案

transition 去重闩（`globalThis._zwBlurChangeFiredSel`，修复 5bc4acc6d）：

1. **派发点置闩**：shim 三臂（part04 blur()、迁移臂，part06 `__zw_host_blur`）
   派 change 后写 `= sel`；
2. **消费点查闩**：宿主账派发改走 `script_dispatch_change_on_blur`——脚本先查
   闩，命中则只清闩不派发，未命中照常派（legacy 兜底语义不变）；
3. **相位边界清闩**：获焦相位落定（focus 臂 `_activeElKey` 赋值后、
   `__zw_host_focus` 同位）清闩——陈旧闩不得压掉下一个 transition 的宿主
   兜底补派。

## 避免方式

- 给既有用户可见事件（change/input/focus 系）**新增第二条派发路径**前，先
  grep 宿主侧（renderer `runtime.rs`、`page_scripts.rs`）同名事件的既有派发
  账；两账并存必须配对交付去重机制，不能各自默认对方不存在。
- 去重键用「transition 语义」（本次失焦是否已派）而非元素级布尔——元素级
  标志会把下一次合法 transition 一起压掉；闩的生命周期必须绑定相位边界
  （失焦→获焦），在获焦提交点显式清零。
- 回归测试沿三层各钉一点：shim 臂派发置闩、宿主查闩跳过、清闩后兜底照派
  （`js_dom_bridge::tests::test_change_on_blur_dedup_latch_zrg`）。
