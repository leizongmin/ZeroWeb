# perf/wpt-runner-testharness-speed——html-semantics 执行提速：插桩归因 + 热路径清除 + adopted 隔离 + 并行化（2026-10-27）

## 范围

weekly coverage 流水线墙钟 3–3.5h，瓶颈是 html-sem shard：testharness-html-semantics
本地全量 79min（2594 案含 variant）。分支 perf/wpt-runner-testharness-speed
五笔提交（原计划 6 笔，字体共享按 Commit 1 数据否决砍掉），验收目标：全量本地
≤25min、结果与串行等价、CI 4vCPU 默认自动 jobs=3 零 workflow 改动。

## 插桩归因（ae34ad8dd）

`ZW_TESTHARNESS_TIME_LOG=1` 每案 stderr 分段计时（new/font/load/first_exec/
setup/media/run+rounds）。结论修正了两个预设：

- **字体共享无收益**：font 段实测 ≈0ms（原计划 Commit 3 的 SANS_FONT_LOADER
  跨案共享被否决，未实现）——语料无 @font-face 且 canvas 系统字体解析非瓶颈。
- **固定成本占比小**：普通案 ~0.1s/案（first_exec 55–67ms 最大头 = shim V8
  编译）；墙钟大头在 run 段等待——deadline 案（style-error-01、style_events
  等）烧满 30s 死线（~5.4ms/round 探针忙等）。→ 主杠杆是并行化，不是缓存。

## 落地（按提交序）

| 提交 | 内容 |
|---|---|
| ae34ad8dd | 每案分段计时插桩（诊断，env 门控，stderr） |
| c33eeabb8 | 热路径杂音清除：4 处 /tmp 心跳写删除（并行硬前置——固定路径多线程互覆写）+ 9 处 env::var 收敛 RunnerFlags OnceLock（learnings 2026-09-14 反模式） |
| 56f3bcc5a | adopted CSS 线程局部 runner 作用域：engine 侧 RUNNER_LOCAL_ADOPTED + enter/exit（镜像 ADOPTED_CSS_RENDER_OVERRIDE），runner 每案 RAII guard——串行下修复上一案 adopted 文本经进程槽残留到下一案的既有污染，并行下消除互覆写 |
| ec0b26764 | NATIVE_STATE_GEN 线程局部化（并行化前置依赖，见下「并行暴露的引擎缺陷」） |
| 13cea2f62 | html-semantics collect/execute 拆分：枚举半拆出（同递归/同 skip 谓词/同 variant 展开逐字不动），执行入 `crate::parallel_map` 闭包（reftest 先例）；jobs<=1 同闭包串行无第二代码路径；cmd 传 effective_jobs，stderr `Using N test job(s).` |

## 并行暴露的引擎缺陷（ec0b26764）

A/B 首轮发现 jobs>=2 下 forms the-label-element clicking/proxy 两案 6 子测
**确定性** Fail（`cloneNode is not a function` / 断言 false；jobs=1 稳 Pass，
j2/j3/j8 差同一批键）。根因：`NATIVE_STATE_GEN` 进程级 AtomicU64 守护
thread_local 模板缓存——并行 worker 线程的 WebView Drop `fetch_add` 全局代际，
在跑案的下一次 install（每次 execute_script_with_dom 都调）误判「缓存属于已
销毁 Isolate」走 reset+全量重装，打碎在跑案 JS 已捕获对象的模板/身份映射。
thread_local Cell 修复后：判别实验 103 键 j1 vs j2 零差异；同线程语义逐位
不变（engine dom_bindings 225 + webview 全量 814 passed，含 R384 双 WebView
共存 panic 面）。

## 对账（等价性）

判据 `(case, 子测名→status)` 集合恒等 + 逐键 status 相同；全部经 test-guard
（per-proc 4G / total 8G）包裹，双二进制 A/B 或同二进制跨 jobs。

| 对比 | 键数 | 差异 |
|---|---:|---:|
| Commit 4 A/B：css-module 26 / the-style-element 48 / forms 2745（A0=Commit 2 二进制） | 26/48/2745 | 0/0/0 |
| fix 判别：the-label-element j1 vs j2（修复前 6 差异→修复后） | 103 | 0 |
| Commit 5：forms j1↔j3↔j8 两两 | 2745 | 0 |
| Commit 5：css-module j1 vs j3；the-style-element j1 vs j8 | 26/48 | 0 |

## 墙钟与内存

| 切片 | j1 | j3 | j8(auto) | 加速 |
|---|---|---|---|---|
| forms/（268 案） | 7:15 | 3:19 | 1:36 | 2.2× / 4.5× |
| the-style-element（25 案） | 5:14 | — | 1:00 | 5.2× |
| css-module（14 案） | 0:52 | 0:21 | — | 2.5× |

峰值 RSS：forms j8 2.46GB、css-module j3 1.83GB——test-guard per-proc 4GB 限内。

## 全量对账

无 FILTER 全量（939 案枚举 / 5306 子测试键，同二进制两遍，test-guard 包裹）：

| 臂 | 墙钟 | 峰值 RSS | 差异 |
|---|---|---|---:|
| `--jobs 1`（串行基线） | 56:38.87 | 1.23GB | — |
| 默认 jobs（本机 auto=8） | **7:45.63** | 3.43GB | **0** |

加速 7.3×，达成「全量本地 ≤25min」验收（7:46）。串行基线较历史记录 79min
（2026-10-23 evidence 口径，含 variant 2594 案）快，含 Commit 2 杂音清除与语料
快照差异，等价性判据不受绝对口径影响（同分支同二进制 j1 vs auto）。峰值
3.43GB 贴近但未触 test-guard per-proc 4GB；CI 4vCPU 默认 jobs=3 内存更低。

## 已知边界

- 30s deadline burn 案并行下仍各自烧满死线（结构性挂账，learnings 2026-10-15
  单案超时网不修）——j8 4.5× 而非 8× 的主因之一。
- 挂账 I 不变：--json 缓冲到结束单次输出，进程超时被杀时结果全丢；无人值守
  长跑须 time-limit 大于套件最坏墙钟。
- 其余 50 个 testharness 套件未并行化（拆分模式与 checklist 复用，各自独立
  立项）。
- `--jobs 1` 即回滚开关，无新增 env。

## 门禁

fmt + clippy（zero-engine/-webview/-wpt-runner 全目标 0 warning）；cargo test
-p zero-engine（dom_bindings 225）/ -p zero-webview（814）/ -p zero-wpt-runner
（236）全绿；每提交独立过 lei-pre-commit-guard PASS。Makefile 与 weekly 矩阵
零改动。
