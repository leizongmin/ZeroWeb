# reftest-upstream 大目录触发 test-guard OOM 杀进程（fail-list 捕获空致误判）

**日期**：2026-07-29
**相关模块**：`tests/wpt-runner`（`cmd_reftest_upstream`）、`scripts/test-guard.rs`（OOM 包裹器）
**触发轮**：R2160（css-text A/B fail-list 捕获空）

## 问题描述

R2160 对 Phase A slice 2 probe 跑 css-text self-source reftest A/B（`make reftest-upstream`
等价的 `cargo run --release --bin zero-wpt-runner -- reftest-upstream css/css-text`）。第一次
用默认并发（15 jobs）跑 OFF，想捕获 ✗ fail 列表对比 ON，结果 **fail-list 文件 0 行**——
但 per-dir 摘要（Total/Passed/Failed）能拿到（OFF 1742/84、ON 1727/99）。

起初怀疑 ✗ 行 grep 模式（`^  ✗` 多字节 UTF-8）问题，换 `^  (✓|✗)` / perl `\s+✗` 均返 0。
最终查原始 stdout 发现 test-guard 杀进程日志：

```
test-guard: 单进程内存超限 (6302848 KB > 6291456 KB)，已杀死进程树 (root pid ...)
```

## 根因分析

- `test-guard.rs` 默认内存上限 **6291456 KB = 6 GB**（OOM 防护，防单个内存型 bug 吃光内存
  连累 rally/tmux；见 `docs/rally/oom-guard.md`）。
- `cmd_reftest_upstream` 默认 `jobs = effective_jobs()`（CPU 数，本机 15）。每个 job 并行
  加载 + 渲染一个上游 reftest 页（含 DOM 解析 / style 计算 / layout / paint / 像素对比），
  单进程内存峰值高。
- **大目录**（css-text 1826 案 / css-multicol 452 案 / CSS2 全量 6283 案）× 15 jobs 并发
  → 总内存峰值超 6 GB → test-guard 杀整个进程树。
- 被杀后：**输出截断**——per-dir 摘要有时在杀之前已 flush（拿到部分数字），但逐案 ✗ 列表
  在杀之后未 flush → 捕获空。**故「拿到 Total/Passed 数字」≠「run 完整」**，fail-list 空
  可能是被杀而非真无 fail。

## 解决方案 / 如何避免

**跑 reftest-upstream 大目录（css-text / css-multicol / CSS2 全量等）时，显式降并发到
`--jobs 4`（或更低），稳留 6 GB 上限内：**

```bash
# 大目录 A/B（jobs 4 稳定，不触 OOM）
./target/test-guard -- cargo run --release --bin zero-wpt-runner -- \
  reftest-upstream css/css-text --jobs 4
```

判定 reftest-upstream run 是否被 OOM 杀的信号（按可信度）：

1. **grep 原始输出 `单进程内存超限` / `已杀死进程树`** = 被 test-guard 杀的铁证。
2. **fail-list 捕获 0 行但 Total/Failed 摘要显示有 fail** = 被杀截断（fail 行未 flush）。
3. **Duration 异常短**（大目录 < 几秒）= 早期被杀。

关键 A/B 数字（尤其 fail-list）**须用 `--jobs 4` 复验**，勿信被杀 run 的空捕获或单次摘要。

小目录（css/CSS2/box-display 120 案、css-grid 49 案、css-position 97 案等）默认 15 jobs
不触上限，无须降并发。

## 关联

- test-guard OOM 兜底设计：`docs/rally/oom-guard.md`、`docs/rally/run-rules.md`
  （本仓选定 `make test` / `make reftest` 经 scripts/test-guard.rs 包裹）。
- R2160 evidence §方法论(2)：`docs/goal/rendering-compat/evidence/r2160-phase-a-slice2-multi-inline-probe-netneg-2026-07-29.txt`（gitignored 本地）。
