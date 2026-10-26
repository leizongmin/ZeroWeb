# M2-s85 定证轮：s84「gBCR materialize 滞后」根因证伪 + closedby 红图修正（零生产代码）

日期：2026-10-25 ｜ 轮次：s85（定证，s53/s84 先例）｜ 前序：[s84](2026-10-25-m2s84-closedby-parity-recheck.md)

## 1. 结论：s84 根因不成立，「23F 可施工」重归挂账

s84 假设「showModal/showPopover 后 mutation 批 materialize（apply→render→rect 快照）在 rAF 驱动节拍整链滞后 ~5+ 探测轮」经三路独立定证**证伪**——当前 main 的 take_probe flush/version-gate/refresh 链逐轮即拍、无漏批点；rect 首拍即非零。s84 的 px=0/py=0 观测与其红图（含本轮实测复绿的 simple 2F）不可复现，判定为 s84 自身插桩/产物态伪值（见 §4）。

## 2. 三路独立定证（插桩临时已全部回退/删除）

1. **take_probe 链路直插**（`ZW_PROBE_TRACE=1`：flush 前后 `mutation_version`、refresh 门、`refresh_if_html_changed` changed/lengths、apply tail——corpus `dialog-popover-closedby-complex.html` 实测）：逐轮序列 `flush 即 apply（[zw-apply] tail=2 applied_cursor=4 queue_len=6）→ version 同拍 bump（ver_pre_flush=2 ver_post_flush=2 ver=2 last=1）→ 门即开 → serialize（html_len=240609）→ changed=true → refresh fired=true`，ver 1→7 逐轮即拍；GATE MISS 0 次；`fired=false` 仅出现在 html 未变轮（240617==240617，正确 no-op）。链上不存在 s84 猜测的三处漏批候选（mutation_version 门时序 / runner_probe_html 基线 / timer 泵与 rAF 拍批次归属）任一成立。
2. **Rust 复现测试**（`tests/wpt-runner/tests/s85_rect_repro.rs`，临时已删）：复刻 probe loop（`__zw_raf_tick`→flush→timers→modal sels→version 门→refresh）对 normal 与 `<template shadowrootmode=open>` 双结构逐拍采样 `getBoundingClientRect`——**round 0 rect 即非零**：`normal[dialogA@8,100,784,18 popoverA@8,118,784,18 dialogB@8,200,784,18 popoverB@8,218,784,18]`、`slots[dialogA@8,100,0,36 popoverA@8,26,784,18 dialogB@8,200,0,36 popoverB@8,44,784,18]`，12 轮与 +100ms 臂稳定不变。s84「sync/2rAF/4rAF 三拍恒零、+100ms 才有效」在当前 main 不存在。
3. **TD 点击坐标实测**（`ZW_TD_DEBUG`，corpus complex）：normal 嵌套 20 子测试全 Pass，clickOn 落点非零且命中正确目标（如 (400,227)→popoverB）——「首腿 px=0/py=0 → backdrop retarget 误隐」不存在。

slots 结构观测（dialog 宽 0、popover 平铺位置）同时把失败面钉回 s51 结论：slots 红是 declarative shadow 未实现（template 拆包平铺、shadow 几何缺席），不是全局 materialize 滞后。

## 3. 红图修正（当前 main，make 目标 + FILTER 当拍实测）

| 文件 | 实测 | 归账 |
|------|------|------|
| dialog-closedby.html | 27P/0F | 与 chrome-158 持平 |
| dialog-popover-closedby-show-stacked | 4P/0F | 同上 |
| dialog-popover-closedby-start-open | 2P/0F | 同上 |
| dialog-popover-closedby-simple | **6P/0F** | s84 记 4P/2F 过时（复绿） |
| dialog-popover-closedby-corner-cases | 29P/1F | s53 spec-consistent keep（ESC re-entrant showModal，crbug 417571176 族） |
| dialog-closedby-bounds-clicking | 12P/2F | dialog UA `position:absolute` 未上（R-s31，渲染流：layout abspos shrink-to-fit 前置，已挂账） |
| dialog-popover-closedby-complex | 24P/16F | 16F 全 shadow-slots 变体；normal 嵌套 20/20 Pass |

家族合计 **104P/19F**（s84 记「23F」过时）：16F = declarative shadow parser 提升 + shadow 几何渲染流联合切片（s51 用户决策门禁维持）；2F = 渲染流 R-s31（已在 style-system 挂账）；1F = s53 keep。**closedby 家族在本流工作面（engine shim/webview/runner）无剩余可施工面。** chrome-158 全绿参照（s84 §1）不受影响，受影响的是「本地可施工」判定。

## 4. s84 伪值来源推断（存疑不下死论）

s84 三步探针为临时 JS 插桩（包 `__zw_dialog_light_dismiss_check`），其红图含本轮实测复绿的 simple 2F——与陈旧产物态症状一致（先例均已在案：`make test` 后 target-disk-guard 清 release 产物需重建、裸跑缺 `ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1` 假红）。具体肇因不定论；教训补一条：**挂「根因已隔离」前，红图数字须当拍 main 复核**——本次 FILTER 复跑数分钟即证伪 simple 2F，若先复核可省整轮定证。

## 5. 范围与记账

- 零生产代码轮（s53 先例）：`tests/wpt-runner/src/testharness.rs`、`crates/webview/src/webview.rs` 插桩与复现测试临时已全部回退/删除，工作树回 s84 基线。
- s84「23F 转可施工」改记回挂账：16F 用户门禁联合切片 + 2F 渲染流 R-s31 + 1F keep + 2F 已复绿。
- corpus 不可改维持（popover-utils.js 2-rAF 采样为上游行为；正常面下我方 rect 同步可见，无需我方「新鲜度」修复）。
- 本流下一步转 testdriver-Actions 支持面（s78 挂账：hash-name-dynamic 5F + 指针/键盘合成输入族——runner 语料 testdriver.js 缺席面，runner 面属本流工作面）。
