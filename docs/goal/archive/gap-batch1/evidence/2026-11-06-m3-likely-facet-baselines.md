# M3 通道基线 —— LIKELY 五面干跑（2026-11-06）

通道五件套落地（M0 甄别池逐面入账）：`fetch-gap-batch1-subset.sh` 扩
`common/security-features/**` 判据 helper（referrer-policy generic/css-integration
判据面）+ 五个 runner 子命令臂（`testharness-referrer-policy/input-events/
preload/wai-aria/storage`）+ skip 规则共用 `gap_batch1_case_skipped` + Makefile
target（test-guard 包裹，--per-proc-mem 4 --total-mem 8 --time-limit 1800）+
weekly 分片接线（catch-all 缺省约定自动发现，本地 ci_discover_shards 复核：
runner 套件 128 / 五新面入 catch-all-4/5/6/7/8、fetch 目标全解析、≥40 断言过）。

**谓词修复（随本轮落地）**：`gap_batch1_case_skipped` manual 规则从
`*-manual.html` 后缀匹配扩为 `*-manual.*`（`.https.html` 等后缀变体一并排除）
——storage `persist-permission-manual.https.html` 此前漏网实跑（M0 甄别明确
manual 形态应排除）。修复后 3 文件（−1 文件 −1F，P 账不变）；十一份基线中
仅此 1 文件受影响，M1 六面零影响。

## 干跑基线（pin 31597693，2026-11-06 复跑，JSON 同目录）

| 面 | 文件 | 子测试 | P | F | T | 通过率 | DC-4 数值门 |
|---|---:|---:|---:|---:|---:|---:|---:|
| referrer-policy | 37 | 41 | 1 | 31 | 9 | 2.4% | ≥ 1 |
| input-events | 23 | 1,003 | 188 | 812 | 3 | 18.7% | ≥ 188 |
| preload | 68 | 407 | 106 | 279 | 22 | 26.0% | ≥ 106 |
| wai-aria | 28 | 424 | 2 | 420 | 0（+2 Unsupported） | 0.5% | ≥ 2 |
| storage | 3 | 5 | 0 | 3 | 2 | 0% | 不设门（引擎缺口全域） |
| **合计** | **159** | **1,880** | **297** | **1,545** | **36** | **15.8%** | |

数值门按 control plane 预登记规则落定 = 各面干跑基线 P 数（零回落约束）。
复现性：四面重跑计数与首干跑逐位一致；storage 差异即谓词修复预期效果。

## 失败簇归因（bucket 级，引擎面归属按 goal 划界）

- **referrer-policy 31F+9T**：`.py` 判据端点仿真缺位为主簇（8F
  `r.json()` 回体是 Python 源码 + 13F Network Error 同源）——runner 无
  wptserve handler 臂（`cors-makeheader.py` 同簇，挂账 runner 基建）；
  9T = JS 动态建 iframe / 跨窗 messaging 页（内容规则只捕源码 `<iframe`
  字面，动态创建面漏过 → 挂起超时，如实落账）。
- **input-events 812F**：单簇 674F `gEditor is not defined` = 编辑管线
  依赖面（contenteditable 注入判据，M0 划界归 editing 域）；188P =
  InputEvent 构造器/IDL 静态面（M0 同类域参照预期兑现）；
  formatBack/FontColor 27F = execCommand input 事件派发面部分在册。
- **preload 279F+22T**：referrer 判据端点簇 49F（`NO-REFERER` 断言回体
  是 Python 源码）+ JSON parse 33F 同簇；`Invalid preload load succeeded`
  29F = 无效 preload 类型未阻断（preload.rs 提示队列未消费阻断面）；
  `expected timeout but got load` 20F = onload/fetchpriority 语义缺位。
- **wai-aria 420F**：单簇 402F `test_driver.get_computed_label/role` 缺位
  ——testdriver stub 无 aom 查询臂（挂账 runner 基建 + 无障碍树域）。
  M0「反射面预期高通过」假设证伪：判据本身依赖 computed role/label 查询，
  非纯 IDL 反射。2 Unsupported = 页面级 harness 不支持标记。
- **storage 3F+2T**：`buckets/detached-iframe` 3F = buckets API
  （open/keys/delete）引擎缺位；partitioned-estimate 2T = estimate 面等待
  ——M0 甄别全域 engine-gap（StorageManager estimate/persist/persisted
  全缺）如实兑现，本通道只做 covered>0 对账入账。

## 验证锚

- 单元测试：`cargo test -p zero-wpt-runner --bin zero-wpt-runner`（test-guard
  包裹）285 passed / 2 failed——s249/s250 语料缺文件环境性红，M1 已记账
  （clean main 复现同红），本轮零回归。
- `cargo clippy -p zero-wpt-runner --all-targets -- -D warnings` 干净；
  `cargo fmt --all -- --check` 干净（含 M1/M2 提交带入的 3 处既有漂移一并
  格式化修复）。
- 共享谓词改动（manual 变体）复跑 M3 五面确认仅 storage 受影响；M1 六面
  十一份基线 grep 核对无其他 `-manual.` 执行文件，分面门零冲击。
