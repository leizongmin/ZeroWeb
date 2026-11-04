# dom-ops — 跨浏览器 DOM 与脚本桥接操作基准

目标 [docs/goal/dom-operation-bench.md](../../../docs/goal/dom-operation-bench.md) 的 M1/M2 落地：固定同一页面与操作协议，比较 ZeroWeb 与 Chrome 的 DOM、样式/几何查询和 Canvas 状态操作成本。协议版本 **v1**（2026-11-04 首次落地；数值常量的权威来源是 `run-dom-ops.mjs` 内的 `PROTOCOL` 表，本文件描述并引用它，不复制两份）。

## 范围与边界

- **测什么**：页面 JS 语境下的操作成本——`getComputedStyle`、`getBoundingClientRect`、`replaceChildren`/列表插删、Canvas `fillStyle` 状态与真实绘制、`getImageData`。
- **不测什么**：CDP/桥接往返成本（每个子场景一次 `Runtime.evaluate`，循环在页面内跑完）；criterion 既有微基准（不替换）；V8 单独性能；Speedometer 全量。
- 内部 shim 桥接计数当前**未从产品侧暴露**，报告记录 `bridge_counters: "not_exposed"`；桥接成本通过 DOM 操作耗时间接体现。M3 热点归因如需内部计数再议产品侧暴露。

## 固定协议 v1

| 常量 | 值 | 说明 |
|---|---|---|
| 视口 | 1280×720 | 两侧一致（`page.setViewportSize`） |
| 预热 | `min(opsPerSample, 100)` 次操作 | 计时前执行，不计样本 |
| 样本数 | 12 / 子场景 / 顺序遍历 | forward + reverse 双序（DC-4 正反序） |
| 样本间隔 | 25 ms（`setTimeout`） | 让事件循环排空 |
| 单样本上限 | 4000 ms | 超限截断（`truncated`），样本数不足一半 → INCONCLUSIVE |
| 顺序 | forward 后 reverse，各自重新 `goto` | 中位数比值 >3 → 比较行 INCONCLUSIVE（顺序效应） |
| 比较口径 | 每操作时间 p50 比值 ZeroWeb/Chrome | ≥10x 标 `ANOMALY`（数量级异常），≥3x 标 `notable` |
| 定时器 | 页面经 COOP/COEP 头服务 → Chrome `crossOriginIsolated`（如实记录）；分辨率探针取相邻 `performance.now` 最小正差 | per-op 值由样本时长派生：任一侧分辨率误差超过其最小样本的 10% → 该比较行 INCONCLUSIVE（实测 Chrome ~5µs、ZeroWeb ~100µs 粒度） |

子场景与每样本操作数（`opsPerSample`，固定表）：

| 场景 | 子场景 | ops/样本 | 语义 |
|---|---|---|---|
| style-geometry | `style.read.same` | 2000 | 同元素重复 `getComputedStyle().color`（缓存读） |
| | `style.read.rotate` | 2000 | 轮转 600 元素读（缓存压力） |
| | `style.write.color` | 1000 | `style.color` 赋值（样式失效，不读几何） |
| | `geom.read.same` | 2000 | 同元素重复 `getBoundingClientRect`（缓存读） |
| | `geom.read.rotate` | 1000 | 轮转几何读（无失效间隔） |
| | `geom.write_read.thrash` | 50 | 写 `style.width` 后读下一元素 rect（强制布局） |
| mutation | `mut.replace.pool` | 20 | 固定 50 节点池 `replaceChildren` |
| | `mut.build.append` | 20 | `createElement`+`appendChild` 真实建树 |
| | `mut.remove.children` | 15 | 预置 1500 节点逐个 `removeChild(last)` |
| | `mut.move.identity` | 100 | `insertBefore(last, first)` 移位，标记节点身份保持 |
| canvas | `cv.state.fillStyle` | 2000 | `fillStyle` 四色轮转赋值（状态解析） |
| | `cv.state.saveRestore` | 500 | `save()`/赋色/`restore()` |
| | `cv.draw.rect` | 200 | 赋色 + `fillRect` 8×8 真实栅格化 |
| | `cv.read.imageData` | 200 | `getImageData` 32×32 回读 |

fixture 放 `fixtures/`（style-geometry / mutation / canvas），由 runner 的本地 HTTP 服务以 COOP/COEP 头提供，两侧同一 URL。

**引擎预算标定（v1）**：ZeroWeb CDP `Runtime.evaluate` 有同步执行 30s 与 `awaitPromise` 8s 上限（`apps/renderer/src/automation.rs` `AWAIT_PROMISE_TIMEOUT_SECS`）；超时的脚本继续在页面内运行，其晚到的响应可能串扰后续 evaluate（首轮 smoke 实测）。因此每个子场景的同步阶段（setup+预热）与异步阶段（样本+间隔）预算都必须留在此上限内——`mut.build.append` 20、`mut.remove.children` 15 的操作数按 ZeroWeb 实测的单操作成本（append ~0.7–2.8ms/op、remove ~2.4–12.7ms/op，随容器规模增长）标定。改操作数前先跑慢侧估算，超预算会同时破坏该行测量与后续子场景。

## 语义断言（DC-1/2/3）

每个子场景计时结束后执行 `verify()`（不计入样本）：计算值与预期三元组比对（`#hex`/`rgb()` 双形态解析）、几何有限且 `top` 随写入变化（±2px）、`replaceChildren` 后数量/顺序/节点身份（`dataset.seq` 与 `===`）正确、Canvas 像素中心点颜色 ±2 容差、`measureText` 有限正值。任一断言失败 → 该子场景 `status: FAIL`，运行退出码 1。

## 污染与错误注入（DC-4）

- 样本截断 / 顺序效应 / 定时器粗化 → 对应行 `INCONCLUSIVE`，不冒充 PASS。
- `--self-test`：对 `mut.replace.pool` 注入错误期望（children 数 51 ≠ 实际 50），断言 harness 判 FAIL。注入被捕获（FAIL + 非零裁决）→ 退出 0 = harness 自检通过；未捕获 → 退出 2。
- 报告含 `git_sha`、浏览器 product/UA、二进制路径、Chrome flags、`crossOriginIsolated`，样本原始值全保留——操作数与版本可追溯。

## 用法

```sh
# 双浏览器配对采集（默认出对比报告）
node tests/benchmarks/dom-ops/run-dom-ops.mjs --browser both

# 单浏览器 / 附着既有 CDP 端点
node tests/benchmarks/dom-ops/run-dom-ops.mjs --browser zeroweb
node tests/benchmarks/dom-ops/run-dom-ops.mjs --browser chrome --cdp-endpoint http://127.0.0.1:9223

# harness 自检（注入错误期望必须被判 FAIL）
node tests/benchmarks/dom-ops/run-dom-ops.mjs --self-test
```

- 依赖 `playwright-core@1.63.0`，从 `tests/playwright-matrix/node_modules` 解析（不新增 npm 安装）；Node ≥ 20。
- ZeroWeb 二进制默认 `target/release/zero-browser`（`ZERO_WEB_DOM_OPS_BIN` 覆盖）；Chrome 默认 `google-chrome`（`ZERO_WEB_DOM_OPS_CHROME` 覆盖）。runner 自行 spawn/清理浏览器进程，只清理本次创建的进程与临时目录。
- 无人值守跑 runner 用 `target/test-guard` 包裹（内存 + 墙钟）；ZeroWeb release 构建走 `make browser-build` 同款入口并置于 `systemd-run` scope（`make dev-guard` 同参数）。

## 报告

`results/<timestamp>/`（gitignore）：`<browser>.json`（原始样本 + 断言）与 `report.json`（合并 + `comparison` 比值表）。schema `dom-ops-report` v1：`runs[].scenarios{}` 记 `ops_per_sample`/样本原始毫秒/per-op 分位数（p05/p50/p95）/断言/status；`comparison{}` 记两侧 p50 µs、`ratio_p50`、分类与 INCONCLUSIVE 原因。与既有 `bench-report`（criterion 微基准）schema 互不替换；M4 若将本基准接入 `perf-gate`，以新增条目方式进基线，不修改既有配置。
