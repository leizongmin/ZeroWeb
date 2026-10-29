---
date: 2026-10-30
modules: wpt-runner, testing
---

# 守卫杀树总增量按案平摊会伪造"线性内存累积"结论

## 问题

css-variables 全语料单进程跑批三次被 test-guard 杀（>8.8GB / 249 案），按
8.8GB÷249 得出"每案 ~35MB 线性累积、分配器驻留"的结论，并据此把"套件内分批
vs 子进程分片"列为内存治理里程碑（css-testharness-compat M1 立项前提）。
2026-10-30 插桩逐案 RSS（`ZW_TESTHARNESS_MEM_LOG=1`，案边界 VmRSS 采样）后
实测：四组跨套件序列（selectors i18n/parsing、css-variables variable-s、
css-values calc，197 案）斜率仅 0.06–0.82 MB/案，单案 60–76MB 峰值被后续案
复用——线性累积前提不成立。

## 根因

8.8GB 的构成里有一枚病态单案：`variable-exponential-blowup.html` 单案
4.9GB（此前已独立实证）。run 中途命中它后，Oilpan/V8 笼的高位水位线抬升且
不归还 OS，后续所有案的 RSS 都在这条高基线上采样。把"含一枚 4.9GB 异常点的
总增量"按案数平均，异常值被平摊成均匀的"每案 35MB"，恰好长得像线性累积——
均值对重尾分布无解释力，但叙事上极其顺口。

## 解决与避免

- 守卫杀树证据只证明"峰值超限"，不证明"逐案均匀泄漏"。归因斜率前先分离
  已知病态单案（skip 后重测安全子集），再在案边界采样拟趋势；单点 4.9GB
  支配下任何平均数都是口径错误。
- 逐案 delta 之和 ≫ 累计增量，是分配器复用的正常信号（本次 sumDelta
  ~75–104MB vs 累计 9–38MB），不是泄漏证据；泄漏判据是累计线斜率，不是
  单案峰值。
- 治理选型前先做归因实验：本次结论是内存维度不需要分批/分片治理；每案独立
  进程的真实价值是病态内存案与原生同步死循环案的外部兜底隔离（内部超时拦不住
  engine 侧原生自旋），与内存无关。
- 对照实验配平：同二进制、同案序、基线重复两轮、跨臂断言子测试计数相等
  （glibc trim env 只降地板 7–14MB，不改斜率、不改行为）。

实验记录：`docs/goal/css-testharness-compat/evidence/2026-10-30-m1-rss-attribution.md`。
