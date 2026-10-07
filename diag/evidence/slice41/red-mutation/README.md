# slice41 变异 RED 证据：钉 `lingering_bump_dropped_by_navigation_reset_seq_sweep_s41`

- 树：origin/main @ `9799ce551` + 钉（bounded_wait_tests.rs，本切片未提交时点实测）
- 判别代码：`apps/renderer/src/js_worker.rs` `survives_document_reset` seq 模式臂（:830）
- 变异 M-S41：`*seq > reset_seq` → `true`（注入点与行号偏移栏见 `mutation.diff`）

## 结果

| 阶段 | 命令（均经 test-guard 包裹，--test-threads=1） | 结果 |
|---|---|---|
| 变异 RED（v8） | `cargo test -p zero-renderer --lib lingering_bump_dropped_by_navigation_reset_seq_sweep_s41` | FAILED——`assertion left == right failed: 滞留 bump 须被导航 seq 清扫丢弃（不执行）：ec 4→6, left: 6, right: 5`（`red-M-s41-v8.log`） |
| 还原 | cp 备份逐字节恢复 | sha256 `ac8b7bb5f3c5a566f0d9cb46cfebe95e783ddad745f79eaaea7b505737b6add5` 前后一致（`sha256-before-mutation.txt` / `sha256-after-restore.txt`）；`git diff` 空 |
| 还原 GREEN（v8） | 同上 scoped | ok, 1 passed（`green-restored-v8.log`） |
| 还原 GREEN（quickjs） | `--no-default-features --features quickjs` 同 scoped | ok, 1 passed（`green-restored-quickjs.log`） |

## 断言级击杀语义

变异 M-S41（清扫失效）下滞留 bump 于 reset 臂完结后续派——reset 臂内 `reset_context` 与
shim 重装同臂同步、先于续派（js_worker.rs:1471/:1492 先于下一轮分派环顶部 :1158 的
retained 消费）——落新 context 带 shim 执行：gen 会读 1、gen 断言同样会红，并非「仍绿」。
唯 `execution_count` 判别断言（Execute 臂每命令恰 +1：丢弃形态忙臂后仅屏障探针 +1 = 5；
补执行形态 +2 = 6）把失败位次直接落在「滞留命令被执行」这一判别点，且对执行时序排序
无关。ec 对 slice40 汇总 xI-3 申报的观测盲区形态——「bump 于 reset 处理前已在旧 context
执行、随后 context 销毁，gen 归零不可辨」——保持判别（该形态 ec 同样 +2）。本钉以
cfg(test) `execution_count` 补齐该观测面。

（勘误 2026-10-08：初版本段误称「fresh context 尚未装 shim、gen 恒 0」，与 reset 臂
代码结构矛盾——PR #102 缺陷轮首轮评审 S-1 指认，主控核实后更正。）
