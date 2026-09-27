# M1 / DC-1 — hr-time window 子集通过率基线

**日期**: 2026-09-27
**套件**: `make testharness-hr-time`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-27-m1-hr-time-baseline.json](2026-09-27-m1-hr-time-baseline.json)

## 结果

| 指标 | 值 |
|---|---|
| 导入面 | 12 案（10 top-level html + 2 `.any.js` window 变体）|
| 执行 | 9 案（3 案内容级 skip：iframe ×2 / manual ×1）|
| subtests | **7/15 = 46.7% Pass**（Fail × 6、Timeout × 2）|
| 全绿用例 | timing-attack.html、basic.any.js、monotonic-clock.any.js |

## 通道形态（M1 新增）

- fetch：`scripts/goals/10-timing-animation-compat.sh`（目录清单 + `.any.js` 显式清单
  补拉——`lib.sh` fetch 侧排除 `.any.js`，wrapper harness 形态经 runner
  `any_js_window_wrapper` 以 window 变体执行，wasm/fs/indexeddb 先例）
- runner：`testharness-hr-time` 子命令（testharness.rs `run_timing_subdirs` 四 corpus
  共用扫描器：`.html` 直跑 + `.any.js` window 变体 + 绝对路径 helper inline_extras）
- Makefile：`fetch-wpt-timing-animation` + `testharness-hr-time`（test-guard 包裹）

## 失败聚类（M2 修齐方向）

| 簇 | 案数 | 形态 | 归属 |
|---|---|---|---|
| `performance.toJSON` 缺失 | 1 | `expected "function" but got "undefined"` | M2 |
| `performance.addEventListener` 缺失 | 1 | Performance 未继承 EventTarget（basic.any.js）| M2 |
| timeOrigin epoch 语义 | 1+1T | shim `timeOrigin: 0`（相对原点）vs `Date.now()` epoch 对齐断言；worker 子测试 Timeout（runner 无 worker 管道，见下）| M2 |
| `DocumentTimeline is not defined` | 1 | raf-coarsened-time（rAF 时间戳粗化 + timeline 语义）——WAAPI timeline 构造器缺位 | M3 |
| crossOriginIsolated = false | 2 | clamped-time-origin-isolated / cross-origin-isolated-timing-attack 需 COOP/COEP 隔离上下文（跨域记账：安全/网络域 infra）| 记账 |
| clamped-time-origin.html Timeout | 1 | completion 未达（pending 1 test——异步等待语义待 M2 拆解）| M2 |

已实现的真面（基线绿）：`performance.now` 数值/正值/单调性（basic + monotonic-clock
全绿）、100μs 非隔离分辨率语义（timing-attack 全绿）。

## 导入面与排除

- 拉：top-level 10 html + `resources/` helper + `.any.js` 2 案
- 不拉（记账）：`idlharness.any.js`——依赖 `/resources/WebIDLParser.js`（上游
  resources/webidl2 的 build 期生成资产，repo raw 拉不到，实测 404；clipboard
  idlharness 先例）
- 运行面 skip ×3：`navigation-start-post-before-unload.html` /
  `test_cross_frame_start.html`（iframe 依赖：literal `<iframe` 与
  `createElement('iframe')` 双形态规则）、`unload-manual.html`（manual）
- 例外放行：`timeOrigin.html` 含 `new Worker(blob)`——首子测试为 window timeOrigin
  核心断言（DC-2 面），worker 子测试按真实缺口 Timeout（hr-time 通道不施 worker
  skip 规则的唯一案）

## 下一步（M2）

1. `performance.toJSON` + Performance EventTarget 继承（addEventListener 等）
2. timeOrigin epoch 语义（`__zw_performance_now` 共享 origin 侧的 epoch 锚定）
3. clamped-time-origin Timeout 拆解（异步等待语义）
