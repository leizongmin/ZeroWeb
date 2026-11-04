# M2 s280 — runner bin 内存大户二分定位（mini_harness_stateful phase-4 根修 + workspace 腿阈值回归常态）

日期：2026-11-04。切片号 s280（承 s279 下一步候选① = s270 挂账①：
runner 全量钉内存大户二分定位，基础设施轮次）。2 文件（runner
钉面 1 行 + Makefile 阈值回归），零引擎零 shim 改动。

## 勘根（探针分组 → 逐钉普查 → 根因链）

1. **探针分组**（test-guard 6G 兜底 + `/usr/bin/time -v` 峰值
   RSS，全部通过 guard 非裸跑；日志 s280-bin-{full,engine,
   nonengine}-6g.log）：
   - 全量 362 钉（286P/0F/76I）：峰 **4,212,376 KB（4.02G）**、
     墙钟 17.2s——恰超 4G 门限（4,194,304 KB）18MB，即 s270
     「--per-proc-mem 4 杀树」面；
   - engine 群（`testharness*` 过滤 85 钉）单独 **4,656,060 KB
     （4.44G）**；非 engine 群（277 钉含 76 ignored）仅
     **820,624 KB（0.78G）**——内存大户在 engine 群。
2. **逐钉普查**（85 钉逐个单跑 + time -v，/tmp/s280-pin-peaks
   过程表）：65 钉 <150MB、4 钉 150-200MB、12 钉 300-400MB
   （wall 全部 ≈10.4s）、4 钉 ≥500MB——**wall 10.4s 钉恰 14 个，
   全为 `mini_harness_stateful()` 调用钉**（峰 3×~900MB：
   s110-children/s118-innerhtml/s109-textcontent，11×362-375MB）。
   另两枚 ≥300MB 快钉（no_harness_test_wait 506MB/1.53s、
   send_keys 多字符 314MB/1.28s）为合法泵工作面，非缺陷群。
3. **10.4s = CASE_TIMEOUT（10s, testharness.rs:13）到期墙**：
   断言全部 Pass、结果已收，但泵循环空转到 deadline。根因链：
   - `apply_harness_completion_patch`（testharness.rs:10471）对
     以 `\n})(self);` 结尾的 harness 源注入 `__zw_harness_state`；
   - `MINI_HARNESS` 无 IIFE 尾 → 注入空转 → 探针 state=null →
     terminal 判据 arm3（`state.is_null() && complete`）早收
     （0.5s 快钉群）；
   - `mini_harness_stateful` 包装层自带 `})(self);` 尾 → 注入
     成功，但读到的 `tests` 是模拟物：`phase: 0` 恒置 +
     `complete` 空操作 → terminal 三臂全不满足（arm1 需
     phase==4 && pending==0 && 配平；arm2 需 complete && phase==4；
     arm3 需 state null）→ **永 terminal 不达，泵满 CASE_TIMEOUT**。
4. **内存形态定性**：10s 泵窗 = JS 堆增长窗（V8 堆 + allocator
   留存随泵 tick 增长）——迷你页单钉长到 300-930MB；16 并流
   （默认 test-threads=16，本机 16 CPU）聚合 runner bin 单进程
   RSS 4.0-4.4G 超 4G 门限。R5389 workspace 腿 6G/12G 校准与
   s270 挂账①同因同源（真实非「并发引擎工作集结构」，是
   terminal 缺陷 × 10s 泵窗）。
5. **探针注记**：修后普查表 `anchor_url_decomposition_s121`
   首跑采到 3.48G 假峰——改码后首跑含 debug 重建，rustc/link
   编译期 RSS 被 time -v 进程树聚合捕获（wall 4.09s = 编译段）；
   单跑复测 3 连 149MB 稳定（149,564/149,328/149,144 KB）。
   编译期 RSS 与钉内存分型无关，后续普查表应剔除首跑样本。

## 施工（2 文件）

1. **testharness.rs `mini_harness_stateful`**（1 行 + 注记）：
   包装层 `complete: function() {}` → `complete: function() {
   tests.phase = 4; }`——镜像上游 Tests.complete 语义（
   https://github.com/web-platform-tests/wpt/blob/master/resources/testharness.js
   phase COMPLETE）。completion 回调（reporter
   `__zw_harness_complete`）已发、`__zw_mark_harness_loaded`
   → `all_done()` → `complete()` 置 phase=4 → terminal arm1/2
   达成早收。pending 门由 `__completeSoon` 的 `__pending===0`
   与探针 pending==0 双重保持——不完整页（pending 卡死）维持
   泵满 deadline 的既有语义不变。
2. **Makefile workspace 腿**：per-proc-mem 6→4 / total-mem
   12→8 回归常态口径（R5389 校准解除），R5389 注记块补 s280
   真修依据。仅此腿；renderer/browser 各腿不动。

## 验证

- **14 慢钉逐钉复测**（s280 验证轮逐钉表）：**10.4s→0.51s、
  300-930MB→148-185MB** 全表收敛；断言全 Pass。
- **runner bin 全量 @ 4G 常态阈值**（s280-bin-full-4g-after.log）：
  **RC=0、286P/0F/76I 零位移**，峰 **4,212,376→1,235,072 KB
  （4.02G→1.18G，余量 2.8G）**、墙钟 17.2→8.4s。
- **make test @ 回归后 4G/8G**（s280-maketest.log）：74 腿
  **逐二进制 (P/F/I) 全等零位移**（P 总 9082 / F 总 5 / I 总 7
  双侧逐位同）；engine 3724P+5F = R-t2qq adopted-stylesheets
  既有 5 红同集，exit 101 同因（s279 基线形态一致——workspace
  腿后置腿本就不达，见 s279 账本）；workspace 腿 4G/8G 无杀树；
  并行 clippy 臂 `-D warnings` 零告警。
- **quickjs 特性腿**（s280-quickjs-runner.log）：RC=0、
  **216P/0F**——fix 为 feature 无关 JS 文本，quickjs 构建面同收。
- fmt：`cargo fmt -p zero-wpt-runner` 干净 + make test fmt 门禁
  前置通过。

## 挂账更新

- **s270 挂账①（runner bin 内存大户二分定位）销案**：大户=
  14 个 stateful 钉的 10s 泵窗（terminal 缺陷），根修 1 行；
  4G 常态口径恢复。
- **R5389「真修归 runner 通道」碰头账销案**：workspace 腿
  6G/12G 校准解除（4G/8G 回归）；runner 测试并发工作集修后
  1.18G，余量充足，无需腿拆分/并发限流。
- 新注记：**普查表首跑样本剔除**（编译期 rustc/link RSS 混入
  time -v 进程树聚合——s280 实证 3.48G 假峰，后续内存普查以
  二跑起算）。
- 承前不动：root margin 净速分档（s271 挂账①——无上游断言
  面，维持弱候选）；below/in-viewport-dynamic Chrome-only 口径
  （s272 挂账延续）；multiple-times 销案（签名级钉死）；
  dynamic import() referrerpolicy 穿参；decode() 兜底元组细化
  （s276 注记）；lazy 族既有红群（in-scroller ×5 / base-url ×2
  等——视口相交模型面候选待勘根分型）。

## 下一步候选

1. lazy 族既有红群勘根分型（image-loading-lazy-in-scroller ×5
   + base-url ×2 等——shim 车道，语义兼容主战场回归）。
2. below/in-viewport-dynamic Chrome-only 口径勘根（s272 挂账
   延续——inflight 阻塞链调度 vs spec delay load 字面冲突面）。
3. root margin 净速分档（s271 挂账①——仅当语料扩出连接类型
   依赖断言面时重估）。
