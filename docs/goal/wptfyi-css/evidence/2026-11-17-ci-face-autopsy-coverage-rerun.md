# CI 执行面验尸＋coverage 重跑 dispatch（2026-11-17）

## 背景

11-14 weekly coverage 快照（coverage-2026-11-14.json）：css 分子 62,896 /
197,163（31.90%），缺口 134,267。缺口为何这么大——本轮对 CI 工件全量验尸。

## 数据源

- run 43615261973（11-14 weekly）`zw-results-css-face-{a,b,c}` +
  `zw-results-css-v2-face-{a,b,c,d}` + `wptfyi-coverage` 工件：
  `gh run download 43615261973 -D <scratch-dir> -p 'zw-results-css*'`
  （注意 `-p` glob 不跨页，artifacts 默认 per_page=30 有翻页截断，
  v2 分片首轮漏看）。
- 本地五代 runner 对照：`.acceptance/wptfyi-css/results-cssa-h1-full/*.json`
  （刀 H1 收账轮五 corpus，per-case wrapper 通道，与 CI 同子命令+filter 同码路径）。

## 普查结论（73 css 套件）

- 7,081 页 / 114,088 子测 / P=62,687；快照权威账 css zw_passes=62,896、
  covered_files 7,036/35,983、den_subtests 197,163。差 209 = aggregate
  合并含 face 分片外语料（selection-face/forms-clip 等 shard 内 css 面），
  方向健康不细究。
- **402 页全 stub 死页**：每页恰 1 条 Fail，message 形如
  `page script threw (declared tests: 0): TypeError: Cannot read properties
  of undefined (reading 'setProperty')`（masking clip-composition 等）＝
  11-14 CI 旧 runner 页脚本崩溃（0 declared tests，runner 包装层兜底 stub）。
  死页 top：properties-values-api 40、transforms 32、backgrounds 30、
  masking 22、motion 22、values 22、fonts 18、sizing 17、text 12、
  filter-effects 12。
- **live 页零截断**：5 corpus（gaps/multicol/typed-om/shapes/masking）
  CI live 页子测和与本地五代 runner 逐数 IDENT（1768/341/12357/5291/522），
  页集合恒等（masking 61=61）——**无 fetch 缺口**，runner 对拉到的页忠实执行。
- masking 帐目核验：CI 544 = 22 stub + live 39 页 522；本地同 61 页 6,341 =
  dead 22 页 5,819 + live 39 页 522。live 侧逐数恒等。

## 缺口 134,267 三分解

1. **死页面 402 页**（runner 代次问题）：5 corpus 死页本地 yield
   P=6,158（gaps 2,168 / masking 2,173 / shapes 1,198 / multicol 619 /
   typedom 0——typedom 两死页本地同挂非 runner 类）；余 68 套件 356 死页
   无本地账，按 5-corpus 均值 ~134 P/页上限估 ~54k（下限 6.2k，重跑实测）。
2. **covered 页 F ~51.4k**：真实兼容缺口，刀群主战场不变。
3. **未拉取文件面 28,947/35,983（80.4%，子测 ~83k）**：DC-2/DC-3
   corpus 扩面主杠杆（fetch 白名单/分片扩面）。

## 行动与验收

- weekly `run_wptfyi_coverage=true` 已 dispatch：run **44017408145**
  （2026-11-17 04:28 UTC；runner 自 origin/main 当前代构建，刀群修复自动入账）。
- 下轮验收：coverage 快照 css 分子 62,896 → 预期 73k+（死页回收实测值）；
  快照内仍崩死页清单 = 当前 runner 残余崩溃类（下轮刀面线索）。
- 预算闭合粗算：62,896 + 死页回收（6.2k~54k）+ live F 51.4k×修复率 +
  未拉取 83k×yield —— +153k 需三杠杆并用，机械杠杆（死页+扩面）优先。

## 复现命令

```bash
# 普查（每页子测数/P/死页）：json.load 每套件工件，遍历
# [[page, [assertion,...]],...] 顶层列表；口径：all-stub = 全部 message 含 'page script threw'
# 5 corpus 对照
# CI:  <scratch-dir>/zw-results-css-*/testharness-<suite>.json
# 本地: .acceptance/wptfyi-css/results-cssa-h1-full/<suite>.json
```

## 环境注记

- 本轮零码面（fork 群 K1/H2/J2 三链在飞，K orch unit zw-cssa-k-orch-120100
  在跑；watch-k-land-1205.sh 42min 无事件到期正常回收）。
- CI dispatch 前核 `gh run list`（仅 site-opt PR Precheck 完成态，无竞争）。
