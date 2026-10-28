# M3：`_zwMBuildDeepEntry` details 裁决态甄别收口

日期：2026-10-28。挂账来源：M1 evidence「门禁拦截的回归与修复（R4100）」——
深臂初版 setAttribute 灌属性触发 details open 互斥写尾（R4100 others-prong）
破坏解析产物标记态，改工厂 attrs 直灌修复；既有 `_zwMBuildDeepEntry`
（WC-M2 template contents 路径，早于本 goal）同为 setAttribute 灌法，理论
同暴露，挂账 M3 甄别。

## 甄别方法

engine lib 探针测试（`js_dom_bridge_tests/part60.rs`
`test_details_name_exclusivity_template_deep_entry`，R4100 同款 V8 harness）：
页面 markup `<template>` 内放同名 `name=a open` DETAILS ×2，首读 `.content`
触发 sel 视图物化（`__zw_template_contents_deep` → `_zwMBuildDeepEntry`），
直读两元素 `.open`。

## 实证结论

1. **host 解析产物不裁决互斥**：`__zw_parse_html_subtree` 深 JSON 直读实证——
   同名双 DETAILS 的 `open` 属性**双双在列**（原样解析态）；template contents
   序列化（`parse_template_contents_deep_json`）逐属性原样直发。即「树序第一
   open 胜出」的裁决责任在 plain 世界物化面，不在 host。
2. **修复前三面终态**：
   - body innerHTML（深臂 `_zwMBuildDeepTreeNode` 直灌 + `__zwCeAttachForAdded` /
     本地 innerHTML 的 `__zwDetailsScan4100(…, true)` 扫描）：1,0,1 ✓（R4100 T1）。
   - template content 视图（part04 `_tcArr.map(_zwMBuildDeepEntry)`，构建后
     **无扫描**）：**1,1 无裁决**（探针实测）。构建期写尾未触发反转的原因：
     物化数组 map 完成后才挂到 fragment 视图，构建中组员互相不可见，
     others-prong 找不到同组——但也没有任何裁决，双 open 原样直读。
   - createContextualFragment（part06，`_zwMBuildDeepEntry` 渐进
     `frag.childNodes.push`）：**0,1 反转（定性分析，未单独探针）**——条目 k
     构建时 frag 已含 0..k-1（parent=frag 链可达），写尾 others-prong
     「后者关前者」，与树序裁决正好相反。
3. **浅臂 template 分支**（part03 `_zwMBuildNode` TEMPLATE 臂，同 builder）：
   构建期反转与 ccf 同机制，但终态由调用方扫描承担（innerHTML 本地扫描
   `__zwDetailsScan4100(kids, true)` 的全子树 DFS 覆盖 template childNodes）
   ——终态正确，本轮不改。

## 修复（与甄别同提交）

- **builder 直灌**（part03 `_zwMBuildDeepEntry`）：setAttribute 逐个灌 → 工厂
  attrs 映射直灌（null 原型映射 + id/cls 反射字段，逐镜像深臂同款）。构造期
  写尾（R4100 互斥、slot name slotchange 队列）全部不再触发——slotchange 对
  template contents / 未插入 fragment 符合 spec（inert、无 shadow 分配）；
  toggle 排程改由各物化面扫描统一承担（R4009 队列 per-element 去重语义不变）。
- **template content 视图补扫描**（part04）：`_r145Built` 构建后
  `__zwDetailsScan4100(_r145Built, true)`——parse-face 裁决 + parser-open
  toggle 排程，对齐其余 markup 面。
- **ccf 补扫描**（part06）：`return frag` 前
  `__zwDetailsScan4100(frag.childNodes, true)`——builder 直灌消掉反转后，
  由此扫描补树序裁决（直接消费态 1,0；经插入路径消费的面另有插入扫描，
  R4009 去重吸收不重复）。

## 验证

- engine lib 全量 **3571P/0F**（含新测试两面断言 2:1,0 + 原 R4100
  `test_details_name_exclusivity_r4100` 全绿）；fmt 零 diff；clippy
  `-p zero-engine --all-targets -D warnings` 零 warning。
- 通道复验（test-guard 包裹 + 负载门，产物 `/tmp/ab-legacy-mb/m3-verify/`、
  `/tmp/ab-legacy-mb/noM3-htmlsem.json`）：
  - euc-jp 族（深臂锚点）：43 case 40,388P/0F、exit 0 维持。
  - web-components 全通道 vs base（c1482756a）：265/265 `=`、5178/5178 `=`、
    状态差 0。
  - html-semantics 全通道：case 939/939 `=`、subtest 5340/5340 `=`；vs base
    状态差 19 **全部**集中在 `forms/input-change-event-properties.html`
    （Pass→Fail）。**stash A/B 定责**：去本改动 shim 的 HEAD 二进制
    （`/tmp/zero-wpt-runner-noM3`）整通道重跑 vs base 差 26 = 同 19 +
    **7 处 dialog/details**（noM3 Fail、base Pass）→ 19 与本改动无关（并行流
    s129/s130 组合态或通道环境属性：两二进制逐字一致，且该文件 standalone
    双二进制均 11/11 Pass，仅通道上下文失败——input/change composed 断言面
    恰为 s129 改动域，归 html-semantics 流账本，本流不修）；**7 处为本修复
    翻绿**（noM3 Fail → 本改动 Pass，base 与本改动同 Pass，净回归 0、
    净修复 +7——ccf/template 扫描对 dialog/popover fixture 内 details 开态
    的树序裁决生效）。
  - dom 全通道 vs base：368/368 `=`、+335/−2 全 Range-mutations-*
    超时截断家族、状态差 3 全为 Event-timestamp-high-resolution
    Fail→Pass（base 二进制同机复跑同案散布于不同子测试，记账计时 flake）。
    cloneNode-svg 回归翻转维持消失。

## 挂账余项

- form-encode 面可跑性甄别（M3 另一项，本轮未动）。
- DC-3 全量基线 / DC-4 coverage 复测：语料补全卡点维持（本轮 FORCE=1 复试
  仍 136 html，网络 degraded）。
- **跨流红转记（非本流域）**：`forms/input-change-event-properties.html`
  通道上下文 19 子测试失败（composed 断言面），去本改动二进制同现，归
  html-semantics 流 s129/s130 组合态账本；standalone 双二进制全 Pass，复现
  需通道上下文（前置用例后的窗口态）。
