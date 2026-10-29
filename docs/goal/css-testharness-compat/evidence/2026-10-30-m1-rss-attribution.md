# css-testharness-compat M1 — RSS 归因实验（2026-10-30）

## 结论（先答 M1 立项问题）

**单进程长套件不存在 ~35MB/案的线性 RSS 累积。** 四组同进程连跑序列（跨三个
套件、197 案）实测斜率 0.06–0.82 MB/案；单案 ~60–76MB 的工作集峰值被后续案
**复用**（累计增量 ≪ 逐案 delta 之和）。历史 css-variables 全语料单进程
>8.8GB 的修正解释：run 中途命中 `variable-exponential-blowup.html` 单案
4.9GB（此前独立实证）→ Oilpan/V8 笼高位水位线抬升后不归还，把含病态案的全量
总增量按 249 案平摊才得出"35MB/案"——是口径错误，不是均匀累积。8.8GB 本身未
直接复测（病态案会顶穿守卫，正是 M2 排除的对象）；该解释由（a）4.9GB 单案实证
+（b）73 案安全子集实测斜率 0.37MB/案 两点共同约束。

**治理选型落定**：内存维度不需要"套件内分批 / 子进程分片"治理——进程内连跑
在守卫内天然有界。每案独立进程通道保留的真实理由是**每案隔离**：病态内存案
（4–5GB 单案）与原生同步死循环案（engine 侧 `:has()` 匹配自旋，任何内部超时
拦不住）都需要外部兜底杀单案，这正是分片 corpus 脚本已有的形态。DC-2 的实质
交付因此是：分片通道产出 runner 形状 `--json` + 合并等价复验 + coverage 管道
免特调消费（见下"通道交付物"）。

## 测量协议

- 插桩：runner 侧 env 门控逐案 RSS trace（`ZW_TESTHARNESS_MEM_LOG=1` → stderr
  `[th-rss] <case> | rss_kb= delta_kb= hwm_kb=`），`HarnessCaseTimer` RAII 起止
  各读一次 `/proc/self/status` VmRSS/VmHWM（Drop 点在 WebView 析构之后，delta
  即该案拆除后仍驻留的增量）；镜像 main.rs `REFTEST_MEM_LOG` 先例与
  `ZW_TESTHARNESS_TIME_LOG` 快照式开关（guideline #9）。
- 三臂同条件对照（guideline #11/35）：a1/a2 基线两轮（稳定性），b1 = glibc
  主动归还 env（`MALLOC_ARENA_MAX=1 MALLOC_TRIM_THRESHOLD_=4096
  MALLOC_MMAP_THRESHOLD_=131072`），全部同二进制、同案序；每臂经 test-guard
  （per-proc 4G / total 8G）。
- 子集：selectors `i18n/`（43 run-face 案）+ `parsing/`（22 案）、
  css-variables `variable-s` 前缀（73 案，按 e 序天然排除 patho）、css-values
  `calc` 前缀（59 案）。均避开 invalidation/ 等同步死循环案。selectors-4/
  实测仅 1 案 run-face（其余为 reftest 形态），无斜率数据。
- 正确性断言：各臂子测试 Pass/Fail 计数逐一相等（env 变化不得改行为）——
  全部成立。

## 数据

斜率 = 逐案 RSS（MB）对案序最小二乘；rss0/end = 首末案边界 RSS；sumDelta =
逐案正负 delta 代数和；hwmPk = 全程 VmHWM 峰值。

| 序列 | 案数 | 臂 | slope MB/案 | rss0→end MB | sumDelta MB | hwmPk MB | P/F |
|---|---:|---|---:|---|---:|---:|---|
| selectors i18n | 43 | a1 | 0.12 | 75.9→85.2 | 74.8 | 101.4 | 41/2 |
| | | a2 | 0.09 | 76.3→85.8 | 75.4 | 101.9 | 41/2 |
| | | b1 | 0.06 | 73.7→78.2 | 67.9 | 95.0 | 41/2 |
| selectors parsing | 22 | a1 | 0.78 | 82.5→106.1 | 95.7 | 123.0 | 140/264 |
| | | a2 | 0.82 | 82.3→107.8 | 97.3 | 124.6 | 140/264 |
| | | b1 | 0.36 | 79.6→93.4 | 82.9 | 110.7 | 140/264 |
| css-variables variable-s | 73 | a1 | 0.37 | 76→101 | 91 | 117 | 160/22 |
| | | a2 | 0.39 | 76→102 | 92 | 118 | 160/22 |
| | | b1 | 0.32 | 73→94 | 84 | 111 | 160/22 |
| css-values calc | 59 | a1 | 0.30 | 87→114 | 104 | 131 | 101/924 |
| | | b1 | 0.11 | 83→94 | 84 | 113 | 101/924 |

- 单案峰值 delta（各序列 top1）：`css3-selectors-lang-014` +65.5MB、
  `parse-focus-visible` +72.1MB、`variable-supports-13` +66MB、
  `calc-serialization-002` +76MB——均被后续案复用回收，未进入累计线。
- 每案独立进程地板（arm C）：`css3-selectors-lang-001` rss_end 76.5MB /
  hwm 92.8MB；`parse-anplusb` 87.3 / 104.2MB（第三样 `details-open-pseudo-002`
  非 run-face 无 trace）——分片通道单进程峰值 ~0.1GB 量级，并行度受 test-guard
  total 上限约束宽松。
- trim env 效应 = 地板下移 7–14MB，无斜率级差异；无正确性影响。不构成治理手段
  （无线性项可治），也无必要进默认配置（收益毫级、改动面全局）。

## 通道交付物（DC-2）

1. **分片 JSON 通道**：`run-css-testharness-corpus.sh` 增 `CSS_CORPUS_JSON_DIR`
   （每案改跑 `--json`，块体进 transcript，收尾合并为 runner 形状
   `[[case, [{name,status,message}...]]...]` 数组 `<JSON_DIR>/<corpus>[-shardKofN].json`）
   与 `CSS_CORPUS_ONLY_SUBSTR`（子集过滤）。外部墙钟杀的案以合成单子测 Timeout
   条目入账；块体非法即失败退出（不静默缺账）。文本通道行为不变。
2. **恒等判据复验**：`verify-css-corpus-json-parity.sh`（单进程 `--json` vs
   分片合并，(case, 子测名→status) 集合恒等；Timeout 语义差异单列不在判据内）
   + `make testharness-css-parity`（CORPUS/FILTER 可换，M3 模板复用）。
   实测：selectors `i18n/` 43/43 **IDENTICAL**、css-values `calc` 59/59
   **IDENTICAL**（2026-10-30，test-guard 内）。
3. **管道契约核对**：`wptfyi_gap.py coverage` 按 results 目录 `*.json` 通配
   消费 runner 形状数组，`merge_zw_results` 同案取 (total, passes) 最大——
   分片文件可直接投喂，新套件零特调纳入（DC-2 后半句成立）。
4. **全量端到端（2026-10-30，test-guard 内 rc=0）**：两试点目录全量分片 JSON
   各一轮——selectors 274 案 2,001P/2,249F/6TO（4m11s）、css-values 256 案
   738P/4,837F/12TO（2m47s），四态计数与 M0 文本通道基线**逐项一致**；合并
   产物为 runner 形状数组（selectors.json 813KB / css-values.json），直接满足
   `wptfyi_gap.py coverage` 消费契约。三通道互证：M0 文本 transcript → M1
   子集 --json → M1 全量 --json，同一 (case, 子测→status) 面。

## 遗留与边界

- RSS 读数为 /proc VmRSS 案边界采样（WebView 析构后），含 allocator 未归还
  驻留——这正是归因对象；不含进程间共享页差异。
- 斜率样本为轻中量案；病态案（4–5GB 单案）独立于斜率问题，由 M2 skip 规则与
  分片通道外部兜底处理，不混入本实验。
- M1 收口后剩余（入 M2/M4）：css 套件接 weekly coverage results 目录与复测
  合并执行（M4）；css-variables 病态案 runner 谓词排除 + 分片 JSON 接 misc
  分片（M2）。
