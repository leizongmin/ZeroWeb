---
date: 2026-11-06
modules: zero-engine, tests/wpt-runner
---

# WPT runner 完成竞态：module script unhandled rejection 上报与结果收集差一 tick

## 问题描述

shadow-dom M3 尾段小刀（js_dom_shim 纯 JS diff，+146 行）落地时全语料 A/B 显示
`shadowrootadoptedstylesheets` 族（modulepreload/async-fetch，24 文件约 28 行 F）整文件
消失（0 行上报），而 09:00 基线（同一 HEAD、改动前 shim）同样环境下稳定上报。初步结论
「新 shim 弄崩页面」，二分到一个仅 3 行对象字面量的 innocent 编辑仍 100% 复现，矛盾到达顶点。

## 根因分析

1. **页面级失败是同态的**：对所有二进制（改动前/后），该族页面都死在同一个
   unhandled rejection（`getElementById` 同步不可见 setHTMLUnsafe pending 突变 →
   `null.shadowRoot` TypeError）。差异只在「这行失败记录是否被收进结果 JSON」。
2. **runner 结果收集与 rejection 上报是一竞态**：testharness 的
   `promise_test: Unhandled rejection` 行在页面完成路径上落袋；module script 的求值/
   reject 时刻与 runner 收集点相差一个 task tick。
3. **shim 体积/脚本数微扰时序**：给页面 `<body>` 后追加任一 `<script>`（哪怕
   `void 0;`）即把上报翻回来；shim 增减数百字节经 V8 解析时长同样能翻转方向。与
   shim 语义无关的纯体积变化即可让整族页面在 A/B 中「消失」。
4. **取证教训**：交接摘要/记忆对「冻结二进制含哪些编辑」的声称不可信——本次
   `zw-runner-wip-m3residual` 摘要称含 part03 编辑，实际 marker 计数为 0（构建时刻早于
   编辑写入）。对 shim 二进制做 provenance 判读必须直接对二进制数独有 marker 字符串
   （python `bytes.count`，注意 grep 对二进制行计数有歧义）。

## 解决方案

- **鉴别法**：A/B 出现整文件行消失时，先做页内扰动实验（复制页面追加空 `<script>` 重跑）
  与跨二进制错误消息比对（页面加 `window.onerror`/`unhandledrejection` 陷阱把错误变成
  一行测试输出）；若两二进制 reject 同态且扰动可翻转上报，判为收集竞态 artifact，不是
  语义回归。
- **门语义**：A/B 硬门只看逐名 P→F 与 after-only 新增 Fail 名；整文件 vanish 与
  gethtml 预算漂移、zz-* 探针（子测试名带时间戳，逐轮身份不稳定）都不进语义门。
- **最小改动联动**：无 corpus 收益的 shim 增量（本例 MEl 域克隆臂 + 工厂三字段）会以
  体积扰动既有竞态族，无收益不落地；俟其服务的物化面（m3c2）落地后随复测一起进。
- 该族本身在簇 D triage 已记「module/fetch 载入链环境受限」——本次只是给该结论补了
  机制解释。
