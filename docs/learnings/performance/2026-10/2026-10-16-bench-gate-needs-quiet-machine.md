---
date: 2026-10-16
modules: bench-gate, rally
---

# bench-gate 必须静机跑——与其他重负载并行 = 测量污染假 FAIL

## 问题

R5149 在 corpus 全量 reftest（nohup 后台）运行期间跑了定向 bench-gate，连续两跑
1-2 项超预算（`mb/block_layout_1000_elements` 基线 2.17ms → 实测 3.01ms，+38%），
两次 FAIL 集还不一致——先怀疑代码回归排查方向完全错误。

## 根因

bench 门禁比较的是纳秒级微基准 + e2e 页面耗时，基线在静机录制。corpus 全量跑批
吃满 CPU/内存带宽，被测进程被抢占 → 实测值整体抬升且逐次漂移。两跑 FAIL 集不同
本身就是污染信号（真回归对同一代码是稳定的）。

## 解决方案

1. bench-gate 前确认机器安静：无并行跑批/构建/浏览器（`ps` 查自己起的 nohup 任务）。
2. 排查顺序：FAIL 集**逐跑漂移** → 先怀疑污染；稳定复现同一指标超阈 → 再看代码。
3. corpus reftest 与 bench-gate 排串行队列，不同时跑（两者都是必跑门禁，先后即可）。
4. 污染跑的 FAIL 不得触发 record-bench-baseline --relax（那会把污染当合法变化写进基线）。
