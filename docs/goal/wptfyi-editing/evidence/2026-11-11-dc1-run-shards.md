# DC-1 · editing/run 分片落地与探针陷阱修复（2026-11-11）

## 背景定谳：为什么周跑分片只有探针

11-10 快照（coverage-2026-11-10.json）editing 三面账面不对称：

| 面 | 覆盖文件 | 通过 | 原因 |
|---|---|---:|---|
| editing/run | 0/120 | 0 | 随库探针骗过 fetch 幂等快路径，上游语料从未拉取 |
| editing/other | 462/512 | 12,997 | 无随库探针，同款快路径正常列目录拉取 |

根因：`goals/lib.sh` 旧幂等快路径按「目录已有 .html 即跳过」判，而
`wpt-data/editing/run/` 有 28 个随库 `zz-probe-*.html`（回归见证探针，
`git ls-files` 实证）；干净 checkout（CI）上目录永远"已有 .html"，
上游 96,745 子测试的语料从未进入 CI（run 42499865933 的 editing-run JSON
只有 20 探针案、上游 0 命中）。`editing/other` 零随库文件（`git ls-files`
实证），同款快路径每次正常走 API，故 11-10 快照 other 面 12,997 过——
不对称本身即根因的对照实验。

## 修复面（4 文件）

1. `tests/wpt-runner/scripts/goals/lib.sh`：幂等判据改拉取完成标记
   `.goal-fetched-${WPT_REV}`（历史已拉目录首跑重列一次后补标记）；
   FORCE=1 先摘旧标记防强拉失败残留；有失败不写标记（下次重跑续拉）。
   本地验证：`make fetch-wpt-editing` 四目录全部按标记跳过、零网络。
2. `tests/wpt-runner/src/testharness_editing_corpus.rs`：filter 支持 `|`
   分隔多段任一命中（test262 通道先例）。
3. `tests/wpt-runner/ci-shards.yml`：editing/run 按文件全路径 pipe 组拆
   **11 片**（er0~er10，result_tag 互异防 merge-multiple 覆盖），组间互斥
   保证同名 case 不跨片重复；探针 zz-probe-* 不进任何片。
   `script_timeout: 300000`（corpus 脚本 run 通道同款）、
   `case_timeout: 3600`（editing 通道按案计 corpus_case_timeout() 缺省
   30s——本地校准 bold 单案 4 变体 ~60s 已超缺省，不提额 CI 整面逐案掐死）、
   `time_limit: 10800`。editing-other 面留 catch-all（registry 已映射
   fetch-wpt-editing，11-10 实证 12,997 过、90s 看门狗足够）。
4. `Makefile`（editing -one 两目标）：FILTER 单引号包裹——pipe filter 经
   make 展开后被 /bin/sh 当管道符（本地实证 rc=127）；CI 侧 weekly.yml
   先赋变量再展开不经 shell 重解析，无此问题。

## 分片覆盖核对（42/42）

python 核对 11 片 pipe 组 vs `wpt-data/editing/run/` 实际文件（43 上游 −
undo-redo.html iframe 案 = 42，runner 侧 editing_case_skipped 跳 iframe，
M1 基线同口径）：无重复、无遗漏、无探针混入。

## 管道 filter 行为验证（守卫内实跑）

```
ZW_SCRIPT_TIMEOUT_MS=300000 ./target/test-guard --per-proc-mem 4 --total-mem 8 \
  --time-limit 900 -- ./target/release/zero-wpt-runner testharness-editing-run \
  'editing/run/underline.html|editing/run/italic.html' --json
```

- 两文件都进（`|` 任一命中生效）：
  - italic.html：3 变体（1-1000 / 1001-2000 / 2001-last）2,073 子测试，87.2% 过
  - underline.html：3 变体同构 2,147 子测试，87.1% 过
- 完整变体进入聚合 JSON（[产物](2026-11-11-dc1-pipe-filter-check.json)）——
  DC-1「至少一个上游文件的完整变体进入聚合」验收本地成立。

## 静态门禁

- `cargo fmt --all -- --check` 无 diff。
- `cargo clippy -p zero-wpt-runner --all-targets -- -D warnings` 干净。
- `bash -n` lib.sh / 33 脚本通过。
- `ci_discover_shards.py` 本地跑：41 手调分片覆盖 128 套件，editing-run
  11 片解析正常、catch-all 排除 editing-run/editing-other。

## CI 验证（待跑）

分片 push 后 dispatch weekly（run_wptfyi_coverage）即可取 CI 证据；
预算失败形态安全（超预算片记 0 + 分片红，下轮再拆）。job 级静态硬顶
360 min ≥ time_limit 10800s + build/fetch。
