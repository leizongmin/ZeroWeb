# R383 — M5 V8 default-on 启动：触发条件复核 + 步骤① default-off 基线（部分闭合）

**日期**: 2026-08-30
**执行序**: zero-web ⚡ 块（2026-08-19 批复）——「先 default-off 全量基线 → 翻开关 → 双流守门 A/B net≥0」；方向已批，无需再征询
**改动面**: 零源码改动（纯测量 + 控制面记录轮）

---

## 1. 触发条件复核（M5 前置勘误）

master.md 待决策清单 M5 行的触发条件「M1–M4 完成、V8 native 路径生产就绪」**已满足**：

| 里程碑 | 状态 | 证据 |
|--------|------|------|
| M1 | ✅ JS 侧实用收口达成 | R359：d3d-r3 blast-radius 探针复核 + d3e 失去独立收益面备档——L2-d3 路线 A 的 d3d 系全部收口 |
| M2 | ✅ 收口 | R361：S6「shim 改调 native」superseded-by-default-on（ZW_NATIVE_DOM=1 实测：native 工厂面已装但页面 document 仍 shim 所有[R9 维持]——在将被删除的路径上建一次性桥不可达） |
| M3 | ✅ 达成 | R100 Vue 3 e2e（mount/reactive+event/reconciliation）+ R339 quickjs 同页对齐（DC-2 双 feature 验证） |
| M4 | ✅ DC-3 交付面闭合 | 基线建立（55808P/11F）、按子域通过率报告、driving 用例账本（imported-tests.txt）、evidence/ 持久化全部落地；「持续维护」是开放态姿态——基线数字随每轮 sweep 自然刷新，非完成阻塞项 |
| V8 native 生产就绪 | ✅ | M7 同款 A/B 事实：testharness-dom-native 双路径对等差 0.02pp（R76）+ R3334/R3335 多 WebView 回归门 + M7 期 flip 暴露的缺陷全在测试/工具面而非 native 行为 |

## 2. 步骤① default-off 基线（本树 `fdecb21d4`，V8 native_dom 默认 false）

| 门 | 结果 | 判定 |
|----|------|------|
| make test（v8 全量） | 508P / 1F | 1F = `window_surface_present_smoke` XOpenDisplayFailed——R355/R340 多轮记录的无头环境预存项，非代码回归 ✅ |
| product-smoke | diff 23.37% struct PASS | 与渲染流 R3830-F 逐字节同值（md5 b0ceaa）——既存 oracle 慢性项（ZRG hmtx 族），oracle re-capture 已归 rendering-compat 流决策；struct-check 全 PASS ✅ |
| bench-gate 定向（engine/webview/script-sandbox 42 指标） | **未闭合** | 晚间时段第三方负载持续（ZeroSeed llvm-cov 常驻 ~9 进程 + 峰值 load1 11.7）；空窗轮询脚本两版均被余波污染（17 FAIL / 11 FAIL）——按 M7 空窗法判据（load1≤1 起跑 + 单跑全 PASS 才采纳）**本轮不采纳噪声数据**，延后到真空窗轮（M7 先例：基线跨两轮）⏳ |

### bench-gate 噪声跑记录（不采纳为基线）

| 跑次 | 起跑条件 | 结果 | 判定 |
|------|---------|------|------|
| 1 | load1=1 但 ZeroSeed 9 进程存活 | 17 FAIL | 测量中途负载爬升，余波污染 |
| 2 | load1≤1 持续 3 检查（v2 脚本） | IDLE CONFIRMED 后 11 FAIL | loadavg 5/15min 仍 4-5（11.67 峰值余波），load1 恢复快于系统真实排空 |

**教训**：`load1 ≤ 1` 是必要的非充分条件——负载突降后 loadavg 中长窗仍高，CPU 频率/调度余波继续压制微基准。空窗判据需升级为「load1、load5 均 ≤2 且 15min 无近期峰值」或直接以「GATE PASS 单跑」为自证采纳（M7 闭合路径）。

## 3. 下一步（M5 步骤②③ 前置）

1. **真空窗补跑 bench-gate 基线**（自证采纳：GATE PASS 单跑即闭合——负载噪声不会产生假 PASS，只产生假 FAIL，故 PASS 结果天然可信）。
2. bench 基线闭合后执行 flip（`WebViewConfig::default()` v8 分支 `native_dom: true`，cfg 门控与 M7 对偶）。
3. A/B 守门：make test 全量 + reftest + product-smoke + bench-gate（空窗），net≥0 才 land。
4. M5 land 后单片删除 kill-switch（R382 勘误的耦合时序）。
