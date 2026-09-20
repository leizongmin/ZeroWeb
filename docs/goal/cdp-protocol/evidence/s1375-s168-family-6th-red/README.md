# S1375 — S168 形态第 6 例 RED 捕获与 #0 深排查立案（2026-09-20）

## 来源

S1375 轮（树变化刷新轮双腿活跑 + #0 升级后首复查窗）门腿**首调即红**，二调收口绿。
本目录为红例全件保全（本轮起报告不依赖转录）：`gate-call1-red.log`（首调全输出，
路径已按 S1343 口径通用化）、`gate-call2-closure.log`（复跑收口全输出）、
`determinism-report-red.json` / `steps-report-red.json`（红例报告 verbatim 副本，
复跑覆盖前拷出）。

## 时间线

| 时刻 | 事件 |
|------|------|
| 09:29:37 | 开工快照：load **6.15**/3.51（高位段 兄弟流编译活跃）、盘 448G、零 zombie、922x 全 free、jsonl md5 2c2cdb6d 恒值；五点锚零漂移 |
| 09:29:58 | 门腿首调起跑（R4550 新树态：3 Compiling = zero-engine → zero-page-runtime → zero-browser 预期兑现；dead_code warning match_media_to_json 真重编第八次再发射） |
| 09:32:07 | 首调收口 **EXIT=2，wall ~2min09s（短延长形态——较第 5 例 ~7min37s 大幅回缩），deterministic NO** |
| 09:33:10 | 复跑收口起跑（0 Compiling 全缓存） |
| 09:33:40 | 复跑收口 **EXIT=0，wall ~30s（常态），deterministic YES，green 33 = expected 33，regressions 空** |
| 09:33:53 | mt 腿错峰起跑 → 09:45:07 收口 EXIT=0 19,376P/0F 68 组（29 Compiling） |

## 红例形态（报告 verbatim）

- **run 1：全绿** — ok 33，failed 仅 `frames.click+evaluate`（期望失败步骤既有形态）。
- **run 2：单步失败** — ok 32，failed 2 步：
  - `frames.click+evaluate`（期望失败步骤，10002ms locator timeout 既有形态）
  - **`emulation.userAgentOverride` — watchdog 60s（60002ms）**
- green 33 = expected 33 对称差 none，regressions 空；deterministic NO 纯由 run 2
  该单步新增失败构成。
- wall 核算：run1 ~30s + watchdog 60s + run2 ~40s ≈ 2min09s 与实测吻合。

## 家族记账更新（第 6 例）

- 样本链：S168（run1 11 步）→ S268（**run2 单步 emulation.userAgentOverride**）→
  S1342（**run2 单步同步骤**）→ S1366（run2 三步聚簇 watchdog）→ S1374（run2 八步
  聚簇 watchdog+locator）→ **本例（run2 单步 emulation.userAgentOverride，回落
  S268/S1342 同型）**。
- 间隔序列：~100 → ~1074 → 24 → 8 → **1 轮**（第 5/6 例连续两轮再现）。
- **run 2 发作偏置 = 5/6**（第 2/3/4/5/6 例全 run 2；run 1 仅首例）。
- 负载窗两极对照：第 5 例低位窗（0.95）发作、本例高位窗（6.15）发作——负载相关性
  进一步弱化（#0 取证）。

## #0 深排查立案（S1374 预埋条款触发）

S1374 记录条款：「若三例内再现或聚簇再扩大即 #0 深排查（双跑进程/端口/临时态残留
取证面）」——**第 5/6 例连续两轮再现（间隔 1 轮）即触发**。

### 本轮事实基线（runner 隔离面实测，scripts/verify-deterministic.mjs 171 行通读）

1. 每跑 spawn 全新 `zero-browser --headless --remote-debugging-port <freePort()>`
   （临时端口，竞态窗源码自述「可接受」）；跑间**零冷却、零残留探针**。
2. terminate = SIGTERM → 5s 背板 → SIGKILL；SIGKILL 启用本身即「优雅退出受阻」信号，
   现行不记录。
3. 首发步骤 `emulation.userAgentOverride`（第 2/3/6 例）= 全流唯一 mid-flow 二次
   CDP session attach 步（`context.newCDPSession(page)` → `Emulation.
   setUserAgentOverride`）；但第 5 例聚簇先行于 `screenshot.element` → 共同形态 =
   **run 2 下半程服务端整体停摆，首发观察点漂移**（步骤视图 ≠ 停摆根因视图）。
4. 残留候选面：run 1 进程树未净退出（render 子进程/orphan）、临时端口/监听残留、
   服务端临时态（profile/缓存/shm）。

### 下轮动作（S1376 首轮，patch verbatim 如下）

`scripts/verify-deterministic.mjs` runOnce 的 `finally` 块（现仅 `await terminate(proc)`）
替换为（log-only，判定语义不变）：

```js
  } finally {
    // #0 深排查残留探针（S1375 立案）：terminate 耗时（接近 5s SIGKILL 背板 =
    // 优雅退出受阻直接信号）+ spawn pid /proc 复核 + headless 实例残留扫描。
    // 仅落 run log 供红例归因，判定语义不变。
    const pid = proc.pid
    const t0 = Date.now()
    await terminate(proc)
    const teardownMs = Date.now() - t0
    let leftover = 'none'
    try {
      leftover = execFileSync('pgrep', ['-a', '-f', 'remote-debugging-port'], { encoding: 'utf8' }).trim()
    } catch { /* pgrep exit 1 = 无命中 */ }
    const pidGone = !fs.existsSync(`/proc/${pid}`)
    console.log(`  run ${index}: teardown ${teardownMs}ms, pid ${pid} gone=${pidGone}, headless leftovers: ${leftover}`)
  }
```

（`fs`/`execFileSync` 已在文件头 import，零新增依赖。）落 patch 时按流程走全
Rust 门禁（含非 docs 文件不适用 docs 豁免：`cargo fmt --check` + `cargo clippy
--workspace --all-targets -- -D warnings`——本轮 docs-only 收口即为规避本 clone
冷 clippy 全量重编成本，预埋记档），再以带探针门腿采集首个 instrumented 样本。

## 保全产物

`gate-call1-red.log`、`gate-call2-closure.log`、`determinism-report-red.json`、
`steps-report-red.json`、本 README。
