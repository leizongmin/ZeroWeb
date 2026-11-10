# DC-3 · net 分片 workers 套件空 JSON 归因——test-guard 4G 帽口 chronic 贴线（非行为面）

**2026-11-11** · 修复 `434dd2f89`（`.github/workflows/weekly.yml` wptfyi-coverage
Run suites 步 `--per-proc-mem 4`→`6`）

## 现象

run [42843305476](https://github.com/leizongmin/ZeroWeb/actions/runs/42843305476)
（HEAD `91ab28c56`）net 分片（net-api encoding wasm workers webmessaging
page-state navigation）**failure**，Run suites 步报
`testharness-workers 无 JSON 输出`。产物 [zw-results-net](https://github.com/leizongmin/ZeroWeb/actions/runs/42843305476/artifacts/13777599913)
逐套件核验：workers JSON 0 字节，其余六套件非空。

job 墙钟 29.6min < run-1（3bfccdfa3）同类 job 36.5min，差值 ≈ workers
套件自身耗时——排除 per-suite time_limit 3600s 截断，定性为 runner 进程
中途死。

## 归因：4G 单进程帽 chronic 贴线，机器方差决定单次绿红

本地 test-guard 复现（`./target/test-guard --per-proc-mem 4 --total-mem 8
--time-limit 1800 -- ./target/release/zero-wpt-runner testharness-workers
--json`）：

| HEAD | 结果 | 证据 |
|---|---|---|
| `f0a23470f`（现 main） | **4,200,476 KB > 4,194,304 KB 杀树**，exit 124，JSON 0 字节 | test-guard 单进程帽采样命中，与 CI 空 JSON 同型 |
| `3bfccdfa3`（run-1 绿 HEAD） | 12G 帽下完赛，RSS 采样峰值 **4,356,828 KB** | 结果 175 案/696 Pass，与 run-1 CI 产物逐字一致 |

两 HEAD 结果与产物完全一致（语料 175 案同集、696 Pass/18 非 Pass 同分布）
→ **行为面零差异**；workers 通道 175 案累计 RSS 天然 ~4.2–4.36GB，贴着
4G 帽口，CI 机器的采样时点/内存布局方差决定单次绿（run-1）或红（run-2）。
语料侧排除：本地 fetch（今日 lib.sh）与 run-1 CI 产物同集同量，无 probe
污染（`git ls-files wpt-data/workers` 0 条）。

## 修复与预期

`434dd2f89` 帽口 4→6G（余量 ~40%），只动 wptfyi-coverage Run suites 步，
wptfyi-gap job（`weekly.yml:696`）与 test-guard 本体不动。下一 weekly
起 workers 面恢复 696 过；本 run 快照 workers 记 0（分母设计「与未跑同
态」），邻域 −696 按本证据归因，不入 ZeroWeb 行为面回退。

本目标三目录（encoding/wasm/third_party）计分不受影响：encoding/net、
mb-decode、wasm 两腿、form-encode 十二分片全绿且与 run-1 逐字或同量。
