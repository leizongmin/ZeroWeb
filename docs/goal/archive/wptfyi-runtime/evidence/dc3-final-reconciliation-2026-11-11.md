# DC-3 · 终局对账（run 42843305476，净增 76,029）

**2026-11-11** · 终局快照 [coverage-2026-11-10](../../../../compat/wptfyi/coverage-2026-11-10.md)
（generated 2026-11-10T17:04:32Z，commit `3e2f0e417` 落库）· 对账 HEAD `91ab28c56`

## 数据源与产物有效性

终局快照来自 weekly dispatch run
[42843305476](https://github.com/leizongmin/ZeroWeb/actions/runs/42843305476)
（HEAD `91ab28c56`，15:20:29Z–17:09:11Z）的 `wptfyi-coverage-aggregate`
job（success）。run 整体 conclusion=failure 系 net 分片 workers 腿被
test-guard 4G 帽口杀死无 JSON（归因见
[workers 帽口证据](dc3-workers-shard-cap-attribution-2026-11-11.md)，
修复 `434dd2f89` 已落 6G）；workers 腿服务 workers topdir（不在本目标
test262/encoding/wasm 计分面），**三目录计分不受影响**，快照判定有效。

## 三目录净增对账

对预算基线 [coverage-2026-11-07](../../../../compat/wptfyi/coverage-2026-11-07.md)
（发布快照，1,317,955 / 1,398,399）：

| 目录 | 基线 11-07 | 终局 11-10 | 净增 | 分母变化 | 定性 |
|---|---:|---:|---:|---|---|
| encoding | 1,128,551 | 1,151,530 | **+22,979** | 1,152,398 不变；未覆盖 23,847→868 | 与 DC-2 账面 +22,979 精确对上 |
| wasm | 189,404 | 190,526 | **+1,122** | 192,207 不变 | 含 −7 上游漂移（已入册） |
| third_party (test262) | 0 | 51,928 | **+51,928** | 53,794 不变 | run-1 51,838 + source-phase 切片 +90 |
| **合计** | **1,317,955** | **1,393,984** | **+76,029** | | |

- 合计 1,393,984 ≥ 验收线 1,393,955，余量 **+29**。
- 净增 76,029 ≥ 验收净增 76,000。
- 上游漂移另报：wpt.fyi aligned run revision `8c04fb8d43`→`77787fe5fe`
  （2026-11-10T10:16–10:44Z 三家 stable）；encoding/wasm/test262 分母子
  测试数不变（1,152,398 / 192,207 / 53,794），漂移仅体现在 wasm 已过
  路径子测试构成（−7）。

## 产物核验（验收第 2 条）

- **非空、真实路径与变体**：三 test262 分片产物
  （`zw-results-test262-{builtins,language,misc}`）合计 53,162 文件 /
  51,949 Pass；快照计 53,141 / 51,928——差 21 条为 runner 产物中上游
  aligned 清单外路径被对账器丢弃（快照口径：570 条清单外条目不进分子
  分母），非数据缺失。
- **分片无截断**：三分片文件数 run-1 vs run-2 **逐字一致**
  （builtins 23,508 / language 23,645 / misc 6,009）——builtins 9000s
  预算内无截断；language 分片 full-pass 22,962→23,052（**+90**）即
  source-phase imports 切片 F2P，零回退。
- **每计分路径只贡献一次**：快照器口径固有（跨套件重复 65 条取最完整
  一次、可疑 2 条剔除、分片收录归组记一次），首跑已核验，终局沿用。

## 邻域回退复核（终局快照 vs run-1 快照，文件级下钻）

方法：两 run 的 `zw-results-*` artifacts 逐套件下钻，凡组级 zw_passes
回退者逐文件、逐子测比对。**三目录零回退**（encoding/wasm 逐字一致、
test262 +90 F2P）。全部回退项定性：

| 回退 | 量 | 归因 |
|---|---:|---|
| workers/*（5 组） | −784 | workers 腿被 4G 帽口杀（无 JSON），环境面；`434dd2f89` 6G 修复已落，下轮 weekly 恢复（[帽口证据](dc3-workers-shard-cap-attribution-2026-11-11.md)） |
| shadow-dom/declarative | −372 | **执行截断**：gethtml.html 单文件子测试总数 3,459→3,087 同步缩水（Pass 3,458→3,086，两轮均仅 1 Timeout）——runner 中途断，非行为 F2P |
| dom/ranges | −93 | **执行截断**：仅 2 文件，子测试总数同步缩水（Range-mutations-dataChange 713→653、replaceData 751→682），同上 |
| css/selectors | −4 | **dom 流跨流位移**：parse-not.html 4 子测（`:not(.a .b ~ c)` 等 valid-selector 形）Pass→Fail，对应 `:not` selector-list 语义收口 `61c148a7b`/`a1a2f1678`（3bfccdfa3..91ab28c56 区间），归 dom/html-semantics 流记账，本流不动 |
| css/css-values | −3 | **css 流组合态中间态**：clamp-color-computed 1 + random-serialize 2 子测 Pass→Fail，对应色彩数学管线施工 `1cc2f4e0f`/`c658ca70c`，归 wptfyi-css 流记账（该流后续提交 af02f52bf 等继续收敛），本流不动 |

增益项（不计本流账）：css 流 F2P——css-color +3,527、cssom-view +545、
css-animations +225、geometry +209、css-transitions +185、
css-properties-values-api +107 等；html/semantics +3 等零星。组合态
方向正常，双流并行无共享面损坏。

## 收口状态

DC-3 判据核验完毕：同口径净增 76,029 ≥ 76,000、最新周跑有效产物落库、
邻域无可归因于本流的回退、守卫证据（`91ab28c56` 轮 script-sandbox 452
测 + runner 319 测 + 定向 clippy，见
[source-phase 证据](dc3-source-phase-imports-2026-11-10.md)）在案。
遗留动作（goal 收口轮）：全仓守卫 build/test/clippy 取证 + DONE 判定 +
归档流程（goal README 第 4 节清单）。
