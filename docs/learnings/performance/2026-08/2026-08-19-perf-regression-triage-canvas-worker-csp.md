---
date: 2026-08-19
modules: zero-canvas, zero-script-sandbox, zero-security, 性能门禁体系
---

# 三例「合规修复顺手引入性能回归」的定位与修复（canvas/worker/CSP）

## 问题

08-11 性能基线捕获后至 08-19，perf-gate 全量门禁 15 个指标 FAIL。逐项 git bisect /
探针分解定位出三例同模式回归——**都是必要的 spec 合规/安全修复，但实现引入了
非必要的热路径成本**：

| 指标 | 基线 | 回归至 | 倍数 | 引入提交 | 根因 |
|---|---|---|---|---|---|
| canvas/stroke_rect_1000 | 1.13ms | 18.8ms | 16.7x | 61a52486b (R34xx) | 每次 stroke 分配+清零全画布去重 mask（480KB）且用完丢弃 |
| script-sandbox/worker_create_terminate | 2.57ms | 22.5ms | 8.8x | a8d5a22d (R3399) | terminate 的 join 轮询固定 sleep(20ms)——正常退出也等满一轮 |
| security/csp_resource_check_1000 | 142µs | 324µs | 2.1x | cd9e0c9e4+faabd5429 (R3389/R3342) | check_source_list 每次 5 轮 values 扫描 + 每源重复解析 expr |

## 修复与效果

1. **canvas stroke mask**（9f905de4b）：mask 常驻 + 脏索引回滚——懒分配一次，
   stroke 结束按 `stroke_dirty` 记录的线性索引只清触达像素（O(触达) 而非
   O(全画布)）。同机背靠背探针 17.5ms → 3.9ms（4.5x）。
2. **worker terminate**（20c291e59）：join 轮询改指数退避（1ms 起步 ×2 封顶
   20ms）——正常路径 1-2 轮即 join，卡死路径仍受 5s 上限 + detach 兜底
   （防 DoS 语义不变）。22.5ms → 3.12ms（7.2x），回基线容忍范围。
3. **CSP 源列表预计算**（9a2deff0e）：`SourceListSummary` 惰性预分类
   （OnceLock，parse 零成本）+ `ParsedOriginExpr` 预解析（下标+字节区间，
   零字符串拷贝）。324µs → 162µs（2.0x）。

## 方法论（可复用）

1. **定位**：先写微探针分解（new vs 各操作分开计时），确认「哪一段变慢」再
   bisect「哪个提交」。canvas 例：探针证明 Context::new 仅 37µs/千次、开销全在
   stroke 本身 → bisect 判据才有聚焦点。
2. **bisect 判据**用探针而非 bench（秒级 vs 分钟级，且判据阈值取基线与现状的
   中界留 5 倍抗噪裕度）。
3. **测量验证**在共享机上受另一条流干扰时，**同机背靠背 A/B**（git stash 切换
   实现、最小暴露窗口）比等待「真空闲窗口」更可信。

## 如何避免

1. **修复/合规变更必须过一次 perf-gate 定向测量**（`ZERO_WEB_BENCH_CRATES=<crate>
   make bench-gate`）——三例回归都在合入后一周才被周期性全量门禁抓住，定向
   测量本可在提交前暴露。
2. **每次调用都分配大缓冲的模式的三大治法**（按侵入度递增）：
   - 常驻复用 + 脏区回滚（canvas mask）
   - 固定轮询 → 指数退避（worker join）
   - 不变量预计算 + 惰性缓存（CSP summary）
3. **预计算要审「成本挪移」**：CSP 首版把 String clone 放 parse 使 csp_parse
   从 562µs 涨到 941µs——检查路径赢的钱在解析路径输光。下标/区间引用 +
   OnceLock 惰性派生才两全。**一个对象的两个基准（parse/check）会互相约束，
   优化必须看总账。**
