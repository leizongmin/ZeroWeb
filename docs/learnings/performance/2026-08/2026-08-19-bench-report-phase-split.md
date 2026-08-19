---
date: 2026-08-19
modules: scripts, ci
---

# bench-report.sh 编译/测量相位分离：批量 cargo 调用消掉串行编译开销

## 问题

`bench-report.sh` 对 16 个 crate 的 criterion 微基准逐个 `cargo bench -p X --bench Y`，
编译与测量交错在同一循环里。QUICK 模式（PR CI 编译检查）暖缓存下 16 次 `--no-run`
竟耗时 **~297s**——几乎零编译工作，纯粹是 16 次 cargo 进程启动 + workspace metadata
解析 + fingerprint 校验的固定开销叠加。真实测量模式下编译段同样无法满核并行
（每次只编一个目标），且第 N 个 crate 编译失败要等前面所有测量跑完才暴露。

关键约束：**微基准测量阶段不能并行**（ns 级测量对同机负载极敏感——脚本自身的
负载守卫 / suspect 标记 / 1.35 因子都是为隔离这种污染而存在）。可并行的只有编译。

## 根因

cargo 的目标选择参数 `-p` / `--bench` 是「沿命令行流式解析」的：`-p A --bench a
-p B --bench b` 会精确选择 a、b 两个目标（各 bench 名在本仓唯一）。因此 N 次
`cargo bench -p X --bench Y --no-run` 可以合并为**一次**调用，cargo 内部即可满核
并行编译链接全部 bench 二进制。旧脚本没利用这一点。

## 解决方案（scripts/bench-report.sh，2026-08-19）

1. **相位 0（新增）**：进入测量前，一次 `cargo bench ${全部 -p/--bench 对} --no-run`
   + 一次 `cargo build --release --bin zero-wpt-runner --bin form-input-perf`
   预编译全部目标。预编译失败则记满 error 条目、清空测量列表、报告仍产出
   （error 条目在场 → perf-gate 直接 FAIL，与旧「bench 非零」语义一致）。
2. **测量循环零改动**：循环体内 `cargo bench` 此时不触发任何编译，机器静默。
3. **QUICK 模式**：批量 `--no-run` 成功即全过（一次调用）；失败回退逐 crate
   循环定位失败者（保留归因语义）。
4. 注意 `-p` 与 `--bin` **不要混排**（`--bin` 归属会产生歧义）；裸 `--bin` 从
   workspace 根解析即可（weekly.yml 同款模式，实测两二进制都建）。

## 效果（本机 16 核，16 crate）

| 口径 | 旧 | 新 | 提速 |
|---|---|---|---|
| QUICK 暖缓存（PR CI 编译检查每次都付） | 297.4s | 0.6s | ~500x |
| QUICK 增量重编译（touch 16 个 bench 源） | 18.8s | 9.2s | 2x |
| QUICK 近冷（删 target/release 全量重建） | ~297s（冷估） | 206.2s | ~1.4x |
| 全量真实测量（含测量本身 ~7min） | — | 444s | 编译段收益同上 |

暖缓存 500x 的构成：16 次 cargo 固定开销（各 ~18s 的 metadata 解析 + 锁 + freshness
检查）合并为 1 次。CI 有 cargo 缓存时 QUICK 接近暖缓存口径，收益最直接。

## 测量等价性验证

同 HEAD 上分别用旧脚本（git stash 还原）和新脚本跑同一批 crate，失败指标数字
同量级（stroke_rect 旧 17.5ms vs 新 18.8ms，均在正常运行间噪声内）——相位分离
不改变测量语义。全量报告的 microbenches 指标集合与历史报告逐 id 一致，
config_hash 不变（基线无需重新 capture）。

## 如何避免

- 多目标 cargo 操作（bench/test/build 检查）优先合并为单次调用，让 cargo 内部
  并行；N 次外部串行调用每次都重复付 metadata 解析 + 锁 + fingerprint 的固定成本。
- 「编译」与「测量/执行」分相位是性能测量脚本的通用纪律：既压缩墙钟，又保证
  测量相位机器静默。
