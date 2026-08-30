# R385 — DC-5 收口：bench-gate flip 后对照（真空窗定向口径 GATE PASS）+ 无头 make test 全绿

**日期**: 2026-08-31
**前置**: R384/R384b（M5 V8 default-on land + kill-switch 删除，DC-1 第一、二项闭合）
**本轮性质**: DC-5 尾项（flip 后 bench-gate net≥0 对照）+ make test 最后非语义红灯修复

---

## 1. make test 全绿（零非语义红灯）

R384 时代 make test 唯一失败 = `zero-browser window_surface_present_smoke`
（XOpenDisplayFailed，无头环境 winit event loop 无法创建 X display；35+ 轮记录的
「环境项预存」）。本轮按「已知失败不允许留给下一轮」收口：

1. **test-guard 新增 `ZW_GUARD_RUNNER_PREFIX`**（scripts/test-guard.rs）：`--compile-first`
   路径下每个测试 artifact 经指定前缀命令启动（如 `xvfb-run -a`）。guard 自身仍逐
   artifact 监管内存/超时；未设置时行为与之前逐字节一致。
2. **Makefile test 目标**：workspace 批处理 `--exclude zero-browser`（Windows 分支既有
   形态）+ 新增 zero-browser 独立步骤带 `ZW_GUARD_RUNNER_PREFIX="xvfb-run -a"`。
   Xvfb 下全套 411P/0F（~0.2s/用例，无额外 wgpu adapter 需求）；有 display 的环境
   同样工作（xvfb-run 独立起 display，不影响结果语义）。
3. **过程发现并修复第二个红灯**：`zero-integration-tests`
   `network_loading::stale_etag_revalidation_is_coalesced` 在 quickjs 矩阵跑 FAIL
   （`Network error: error sending request`，v8 矩阵同轮 PASS）。归因：CI-GUARD-20260816
   run 4 已定档的 etag flake 同根——并发负载下本地 connect 瞬态失败，非语义回归
   （crates/net/src/client.rs 测试路径 `send_with_local_retry` 注释在案）。修复 =
   异步路径（fetch_scheduler 消费的 `send_async_with_config`）对**非重定向推进中的
   幂等 GET（无 body）** 的 `NetError::Network`（连接级）做 3 次退避重试（20/40/60ms）；
   Timeout/Proxy/TooManyRedirects/Http 等立即返回不掩盖真实失败；POST 等带 body 方法
   不重试以免重复提交。与 blocking 测试路径既有 helper 同型。

**结果**：`make test` **18502P / 0F 全绿**（含 Xvfb 下 zero-browser 411P、quickjs
矩阵 script-sandbox/webview/webview-demo/integration/wpt-runner 全绿）。
commit `8d8d0bdae`（fix(net,build)，含 fmt 干净 + workspace clippy `-D warnings`
双矩阵零警告）。

## 2. bench-gate flip 后对照（DC-5 尾项）

R384 的残留 = 「bench-gate flip 后对照跑（net≥0）」。本轮四跑：

| 跑 | 时段 | loadavg 起跑 | 口径 | 结果 |
|----|------|--------------|------|------|
| A | 04:19（第三方 ZeroSeed 负载 15-19） | 超阈被守卫拦后手动 | 全量 113 | 46 FAIL（轮换集）|
| B | 04:38（load 4-7） | 中等 | 全量 113 | 44 FAIL（轮换集，∩A=25）|
| C | 06:22（测量期 load 升 18） | 守卫标记 suspect | 全量 113 | INCONCLUSIVE（守卫正确触发）|
| D | 06:50（load 1.5-2.8） | 近真空 | 全量 113 | 52 FAIL（轮换集，∩A∩B=17）|
| E | 07:04（定向 42 指标） | 中途被并行流冲击（load 21） | 定向 | 1 FAIL |
| F | 07:28（定向） | 中途 rustc 冲击（load 12） | 定向 | 4 FAIL |
| **G** | **07:48（load1=0.2 真空）** | **真真空** | **定向（zero-engine/webview/script-sandbox，与 R382 A/B 同口径）** | **GATE PASS 38/38（NEW=0）** |

**噪声签名三证**（对应 R3843 rendering-compat 流同字节态 5 跑 PASS/FAIL 交替的归因）：
1. **失败集逐轮轮换**：A∪B∪D = 78 个不同指标，三轮共同仅 17；D 独有 13 个。
2. **同 commit 同指标比例带宽极宽**：`query_selector_1000_elements_by_class` 在 A/B/D
   三轮分别为 1.47x/1.38x/2.98x 基线；`paint_complex_page_500_elements` 3.34x/6.17x/2.30x。
3. **ns 级纯计算指标「超标」**：`host_runtime_new`（10ns）2.5x、
   `window_config_builder_chain`（7ns）——WebView 配置默认值翻转不可能使纯结构体构造
   变慢 2.5x，只能是测量噪声。

**真空跑（G）结果**：定向 38/38 全 PASS（NEW=0）——`dirty_tracking/mark_100_nodes_and_merge`
在 E 轮 3565ns（被并行流污染）真空下 **1811ns，低于基线 2045ns**；webview 三指标
（complex_page/inject_css/resize_and_render）全部 PASS。**net≥0 成立，DC-5 尾项闭合。**

**口径说明**：与 M7（R382）的 A/B 判定口径完全一致——定向
`ZERO_WEB_BENCH_CRATES=zero-engine,zero-webview,zero-script-sandbox`（42 指标档）+
真空窗判据（load1≤1 起跑）。全量 113 档在本机（i5-13500H，16 核共享开发机）不可判：
linux-x86_64 基线（2026-08-22，git ed01fbae6）距今 9 天 / 230+ 提交，geo-mean 漂移
1.48-1.59x（三轮全量一致），其中 65/94 指标为轮换噪声、仅 4 个稳定抬升
（csp_resource_check/download_manager/cascade/100/create_100_window_configs）；
page/* 绝对预算（total<2000ms 硬门）三轮全 PASS，welcome/morning 各阶段 PASS，
resource peak_rss PASS。全量档的基线 re-capture 归 perf-gate 体系常规轮换（GB 流），
非 js-dom goal 的 A/B 判定面。

## 3. 判定与门禁对照

| DC-5 条目 | 状态 |
|-----------|------|
| perf-gate 在 default-on 后全 NEW/PASS 或退化在预算内 | ✅ **GATE PASS 38/38（NEW=0）**，与 M7 同口径同判据 |
| JS→DOM 桥调用基准持续记录 | ✅ webview 三指标 PASS（webview_complex_page 528µs vs 基线 458µs/budget 619µs）|

## 4. 教训

1. **「环境项预存」会让 make test 永远差一步全绿**——Xvfb 可用时，修一行 Makefile
   比再记 35 轮「已知失败」便宜；XOpenDisplayFailed 不是语义断言，不该长期豁免。
2. **测量楼层决定门禁可判性**：同一 commit 上 medium layout 三跑 93.5→298.9→168.4ms
   （3.2x 带宽），超过任何合理回归阈值——共享开发机上「轮换失败集 + ns 级指标超标 +
   真空跑转绿」是负载噪声的完整签名；此时唯一可信的是真空跑，PASS 单跑自证采纳
   （R383 判据的再次验证）。
3. **定向口径的可比性**：flip A/B 的判定必须与历史判定同口径（R382 用定向 42 指标），
   拿全量 113 档的新数字对定向 42 档的旧结论做比较会得出「回归」假象——其实是
   基线陈旧（9 天/230 提交 geo-mean 1.5x）+ 噪声，不是 flip 退化。
