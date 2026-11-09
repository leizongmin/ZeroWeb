# M1+M2：dom-ops 协议 v1 落地与首轮 ZeroWeb/Chrome 实测

**日期**：2026-11-04 / **结论**：M1（协议）、M2（真实数据采集与对比报告）交付；语义断言在 ZeroWeb 上发现 **2 个真实缺陷**，14 行中 4 行数量级异常（ANOMALY）、2 行 notable、7 行 INCONCLUSIVE（原因逐行标注）。Firefox 缺测（环境无安装，NOT_AVAILABLE，不伪造）。

## 版本与环境

- 仓库 HEAD：`0230c679a`（采集时工作树含本次未提交的 runner/fixtures——schema 与代码在 `tests/benchmarks/dom-ops/`，与本证据同提交入库）。
- Chrome：`156.0.8078.79`（`/usr/bin/google-chrome`，headless，临时 user-data-dir）。
- ZeroWeb：`target/release/zero-browser`，本次按 HEAD 重建（`cargo build --release -p zero-browser -p zero-renderer -p zero-compositor -p zero-image-decoder -p zero-media-host`，55.5s，BUILD-EXIT=0）。
- 平台：linux-x86_64 / i5-13500H 16 核；视口 1280×720；双浏览器均 headless、同机顺序采集（A/B 同环境）。
- 计时：`performance.now` 分辨率探针 Chrome ~5µs（crossOriginIsolated=true，COOP/COEP 头）、**ZeroWeb ~100µs**（平台事实，见发现 3）。

## 隔离与动作

- runner 每次 spawn 全新浏览器进程（freePort + `/json/version` 等待），结束时 SIGTERM→SIGKILL；Chrome 临时目录 `fs.mkdtemp` 用后删除；本地 HTTP 服务 fixtures（COOP/COEP），端口随机。
- 无人值守包裹：runner 经 `./target/test-guard --time-limit 540 --per-proc-mem 4 --total-mem 8`；release 构建经 `systemd-run --user --scope -p MemoryMax=24G -p RuntimeMaxSec=2400`。
- 标定探针（`probes/`）同样经 test-guard 跑，ZeroWeb/Chrome 各一份进程，用后即杀。

## 官方采集（`reports/report.json`）

命令：`node tests/benchmarks/dom-ops/run-dom-ops.mjs --browser both --out tests/benchmarks/dom-ops/results/2026-11-04_official-v1`（results/ 为 gitignore，正式副本在 `reports/`；首轮采集后按 SL-008 把报告 `binary` 字段改为仓内相对路径并重新生成，两轮异常结构一致，下表为最终轮）。

| 子场景 | ZeroWeb p50 | Chrome p50 | 比值 | 判定 |
|---|---|---|---|---|
| style.read.same | 1.35µs | 0.47µs | 2.9x | OK |
| style.read.rotate | 1.72µs | 0.49µs | 3.5x | OK [notable] |
| style.write.color | 27.88µs | 0.15µs | 182.8x | OK [ANOMALY] |
| geom.read.same | 21.18µs | 0.43µs | 49.8x | OK [ANOMALY] |
| geom.read.rotate | 29.27µs | 0.62µs | 47.1x | OK [ANOMALY] |
| geom.write_read.thrash | 54.50µs | 1.50µs | 36.3x | INCONCLUSIVE（ZeroWeb 语义 FAIL，见发现 1） |
| mut.replace.pool | 3207.50µs | 13.00µs | 246.7x | INCONCLUSIVE（ZeroWeb 语义 FAIL，见发现 2） |
| mut.build.append | 4445.00µs | 2.25µs | 1975.6x | INCONCLUSIVE（顺序效应） |
| mut.remove.children | 260.00µs | 2.00µs | 130.0x | INCONCLUSIVE（Chrome 侧分辨率误差 >10% 最小样本） |
| mut.move.identity | 54.50µs | 0.61µs | 89.0x | INCONCLUSIVE（同上，边界样本量） |
| cv.state.fillStyle | 0.30µs | 0.04µs | 7.2x | INCONCLUSIVE（两侧样本 <0.5ms，分辨率误差超阈） |
| cv.state.saveRestore | 3.05µs | 0.37µs | 8.2x | OK [notable] |
| cv.draw.rect | 2.13µs | 0.20µs | 10.6x | INCONCLUSIVE（同 fillStyle） |
| cv.read.imageData | 115.50µs | 3.82µs | 30.2x | OK [ANOMALY] |

DC-4 自检（错误注入）：`node run-dom-ops.mjs --self-test` —— 注入错误期望（children 51≠50）在两侧均被判 FAIL（ZeroWeb checks 0/4、Chrome 2/4），退出 0 = harness 捕获有效。顺序效应与污染样本按协议标 INCONCLUSIVE，未混入 OK 行。

## 发现（M3 输入，复现探针在 `probes/`）

1. **几何读不反映样式写（同步即复现）**：`style.width='33px'` 写入后 `style.width` 存储与 `getComputedStyle().width` 均为 33px，但 `getBoundingClientRect().width`/`offsetWidth` 仍返回 24px（Chrome 返回 33）。→ `geom.write_read.thrash` 在 ZeroWeb 上测的不是强制布局而是空操作。探针 `probe-semantics-and-scaling.mjs` P1 输出：`{"r0":24,"stored":"33px","r1":24,"csWidth":"33px","offsetWidth":24}`。
2. **timer 回调内的 DOM 变更丢 data-\* 属性**：`replaceChildren` 在 setTimeout 回调中执行后，被移动节点的 `dataset.seq` 读为 undefined、`getAttribute('data-seq')` 为 null，JS 身份（`===`）保持；同步执行同操作完全正常。→ `mut.replace.pool` 身份/顺序断言 FAIL。探针 `probe7-dataset-timer-loss.mjs`：`after-replace-sync: seq0="0"` → `in-timer-0: seq0=undefined` → `after-async: attr0=null`。疑与 renderer automation 的 timer DOM 变更 drain/提交路径相关（`apps/renderer/src/automation.rs` `await_pending_operation` 附近）。
3. **ZeroWeb `performance.now` 粒度 ~100µs**（Chrome 5µs）——影响所有页面内计时精度，本身也是待修的平台项。

数量级异常（比值 ≥10x，OK 行）：style.write 183x、geom.read.same 50x、geom.read.rotate 47x、cv.read.imageData 30x；notable：style.read.rotate 3.5x、cv.saveRestore 8.2x。INCONCLUSIVE 行的比值（thrash 36x、replace.pool 247x、build.append 1976x、remove.children 130x、move.identity 89x）仅作参考，修复语义缺陷/重测后回填。两轮运行间 move.identity、saveRestore 在 OK/INCONCLUSIVE 边界摆动（Chrome 侧小样本分辨率误差 ±10% 边界），结构稳定。标定数据（P3/P4）：ZeroWeb append ~0.7→2.8ms/op、remove ~2.4→12.7ms/op，成本随容器规模增长；Chrome 同操作 0.4–0.5µs 平坦。

## 协议标定记录（v1 定稿依据）

- 首轮 smoke 暴露 ZeroWeb CDP evaluate 上限（同步 30s / awaitPromise 8s，`apps/renderer/src/automation.rs` `AWAIT_PROMISE_TIMEOUT_SECS=8`）；超时脚本继续运行，晚到响应会串扰后续 evaluate（smoke 实测：`mut.build.append` 超时后其 checks 落入 `mut.remove.children` 结果）。据此把 `mut.build.append` 200→20、`mut.remove.children` 100→15（预置 4000→1500）重标，官方采集无超时、无串扰。
- reverse 遍历同时逆转同组子场景顺序（首轮只逆转了场景列表，组内未逆转，已修）。
- 比较行 INCONCLUSIVE 判据从「per-op <100µs 且分辨率粗」修正为「分辨率误差 >10% 最小样本」（per-op 值由样本时长派生，首轮口径误伤全部 <100µs 行）。

## 清理

- runner 结束即杀本次 spawn 的浏览器进程并删除 Chrome 临时目录；content server 关闭。
- 构建scope 随命令退出；探针进程由脚本自身 kill。
- 残留：无（/tmp 下 smoke 与探针产物已清）。运行日志见本轮会话记录，正式数据以 `reports/` 为准。

## DC 对照

| DC | 状态 | 证据 |
|---|---|---|
| DC-1 | 场景已跑，语义断言发现缺陷 1 | reports/report.json + probes P1 |
| DC-2 | 场景已跑，语义断言发现缺陷 2 | reports/report.json + probes P7 |
| DC-3 | PASS（8/8 像素断言 + 状态往返，双浏览器） | reports/report.json |
| DC-4 | PASS（配对同环境、正反序、注入捕获、INCONCLUSIVE 判据齐备） | reports/report.json + 自检记录 |
| DC-5 | NOT_RUN | 待 M3 修复后接既有门禁 |
