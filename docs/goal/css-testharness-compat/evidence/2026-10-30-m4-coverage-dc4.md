# M4：coverage 复测收口——css 面 0→入账与 DC-4 判据闭环（2026-10-30）

## 结论

修复后 coverage 运行 **41005658107**（HEAD = e425f845a）聚合快照
coverage-2026-10-30 落账，DC-4 判据全落：

- **css 顶目录从 0 到有数字**：16 目录全部入组，快照 css 顶目录 covered
  files 与 zw passes 从 DC-3 时的 61 files / 232 passes（仅 css-variables）
  增至本轮全量（见下方 CI 快照节）。
- **试点目录清零路径成立**：css-values / selectors 在快照中 zw_passes > 0
  （通道 + 入账管道就位后，未覆盖子测试成为可随引擎修复收敛的常规缺口，
  而非结构性 0）。
- **无坍缩**：三分片产物逐目录对账，案数 16/16 与本地基线一致，子测试数
  无白名单漏 helper 的坍缩形态（对比污染运行 41000541561 的 masking
  544→63 / anchor-position 12,811→3,059 / backgrounds 1,085→190）。

## 接线与白名单修复

- **三分片接线（62064f318）**：weekly coverage matrix 新增 css-face-a/b/c
  （suite 名 = runner 子命令去前缀，aggregate 按案路径归 topdir）；35 号
  脚本加 `CSS_FETCH_ONLY` 白名单过滤；css 分片 job timeout 210min。
- **白名单修复（e425f845a）**：首次带接线运行 41000541561 的 css-face-b
  产物对账实证 `CSS_FETCH_ONLY` 漏 `css/support`（15/16 目录引用
  parsing/computed/interpolation-testcommon.js 等 helper）——过滤函数把
  35 号脚本 DIRS 中的 css/support 滤掉，CI 全新 checkout 缺 helper 目录，
  引用页全数 `declared tests: 0` 单 Fail。typed-om 零引用故计数与本地
  完全一致（750=750），反证根因唯一。运行取消（防污染快照），三分片
  白名单补 `css/support` 后重派。
- **idl_test 4 页**缺 `/resources/idlharness.js` 与本地同界（本地语料同样
  缺，页脚本报错单 Fail），非 CI 洞，不阻收口。
- **form-encode 分片失败（非 css 域）**：同分片在前次运行 40985139206 即
  撞 7200s 限制（2h02m 处 fail），本轮放宽到 14400s 仍在 4h03m 处 fail
  ——form-encode 单套件单进程超 4h，产物仅 1 个 0 字节文件。属
  encoding/表单域基建问题（套件级分批或豁免归口 encoding 目标），
  不阻 M4；本轮快照中 form/url-encoding 记 0 属实。

## 三分片产物对账（CI vs 本地单进程基线）

案数列 = run-face 案数；本地列出自 M0–M3 各批 evidence 基线表。

### css-face-a（css-values / selectors / color / align / sizing / transforms）

| suite | 案 CI/本地 | 子测试 CI/本地 | P CI/本地 | 备注 |
|---|---|---|---|---|
| css-values | 256/256 | 5,576/5,587 | 736/738 | −11 子测，时序带 |
| selectors | 274/274 | 4,257/4,256 | 2,001/2,001 | +1 子测 |
| css-color | 62/62 | 11,009/11,009 | 1,246/1,246 | 全等 |
| css-align | 234/234 | 4,540/4,540 | 657/657 | 全等 |
| css-sizing | 168/168 | 2,238/1,894¹ | 727/701 | 分母重测（见下） |
| css-transforms | 107/107 | 740/740 | 114/115 | −1 Pass 时序带 |

**css-sizing 分母重测（¹）**：CI 声明 2,238 子测试，较 M3 批次 1 记录的本地
基线 1,894 多 344。为定性，本轮 fresh 本地单进程重跑（当前 main 构建 +
本地语料，test-guard 包裹）：**168 案 / 2,238 子测试 / 735 Pass，与 CI 逐案
声明数全等（168/168 案 diff=0），Pass 差 8 在时序带**——即当前语料态的
真实分母就是 2,238，批次 1 记录的 1,894 是单进程跑序残留使部分页脚本提前
抛错少声明（批次 1 已记 6 子测 stretch 族残留同族；stretch 页
block-height-008 单页 160 声明为最大贡献）。sizing 行的十六目录合计按
重测口径修正（见下）。

### css-face-b（typed-om / fonts / shapes / anchor-position / masking / backgrounds）

| suite | 案 CI/本地 | 子测试 CI/本地 | P CI/本地 | 备注 |
|---|---|---|---|---|
| css-typed-om | 348/348 | 750/750 | 25/25 | 全等 |
| css-fonts | 163/163 | 4,977/4,981 | 2,280/2,273 | −4 子测 / +7 Pass |
| css-shapes | 145/145 | 5,298/5,298 | 1,692/1,692 | 全等 |
| css-anchor-position | 253/253 | 12,793/12,811 | 5,201/5,205 | −18 子测，时序带 |
| css-masking | 61/61 | 544/544 | 121/121 | 全等（污染运行曾 63） |
| css-backgrounds | 121/121 | 1,085/1,085 | 267/267 | 全等（污染运行曾 190） |

### css-face-c（motion / grid / text）+ misc（css-variables）

| suite | 案 CI/本地 | 子测试 CI/本地 | P CI/本地 | 备注 |
|---|---|---|---|---|
| css-motion | 45/45 | 391/391 | 105/105 | 全等 |
| css-grid | 653/653 | 7,716/7,716 | 1,251/1,247 | +4 Pass |
| css-text | 354/354 | 3,017/3,017 | 1,104/1,104 | 全等 |
| css-variables | 58/58 | 515/532 | 232/239 | DC-3 同值（时序带先例） |

**十六目录 CI 合计 65,446 子测试 / 17,759 Pass**；对本地 run-face 基线按
sizing 重测口径修正后为 **65,501 / 17,781**（65,157−1,894+2,238 /
17,747−701+735），CI 对修正基线偏差 −0.08% / −0.12%（纯时序带）；
无坍缩形态。

## CI 快照（coverage-2026-10-30）

**产出方式**：运行 41005658107 的 form-encode 分片（form/url-encoding
域，非 css）超 14400s per-suite 限制失败（job 于 4h03m 处 fail），
`wptfyi-coverage-aggregate` 被 needs 门跳过。快照改由**本地聚合同运行
17 个成功分片产物**（68 个结果文件）产出——同一 `wptfyi_gap.py
coverage`，输入形状与 CI `download-artifact` 的 `merge-multiple` 一致
（0 字节空产物剔除；可疑 2 / 跨套件重复 60 / 上游无对应路径 402，与
10-29 快照同基线形态，环比护栏通过）。快照已按 CI 同款提交回 main
（6102d71c3）。

css 顶目录行（10-29 → 10-30）：

| 口径 | 分母文件 | 覆盖文件 | 全过 | 分母子测试 | 通过 | 未覆盖 | 通过率 |
|---|---:|---:|---:|---:|---:|---:|---:|
| css（10-29） | 35319 | 61 | 16 | 195338 | 232 | 195106 | 0.12% |
| css（10-30） | 35365 | 3293 | 362 | 195508 | 17750 | 177758 | 9.08% |

十六目录快照明细（`coverage-2026-10-30.json` groups；zw_env_fails 全 0）：

| 组 | 分母文件 | 覆盖文件 | 分母子测试 | zw passes |
|---|---:|---:|---:|---:|
| css/css-values | 521 | 256 | 10400 | 736 |
| css/selectors | 836 | 274 | 6239 | 2001 |
| css/css-variables | 250 | 58 | 588 | 232 |
| css/css-color | 380 | 62 | 12734 | 1246 |
| css/css-align | 298 | 234 | 5373 | 657 |
| css/css-sizing | 821 | 166 | 6347 | 727 |
| css/css-typed-om | 411 | 348 | 13446 | 25 |
| css/css-fonts | 562 | 162 | 8675 | 2280 |
| css/css-shapes | 385 | 144 | 7527 | 1691 |
| css/css-anchor-position | 547 | 253 | 13663 | 5201 |
| css/css-masking | 569 | 60 | 6399 | 121 |
| css/css-backgrounds | 875 | 121 | 6236 | 267 |
| css/css-transforms | 933 | 107 | 5516 | 114 |
| css/motion | 148 | 45 | 4959 | 105 |
| css/css-grid | 2525 | 648 | 15845 | 1243 |
| css/css-text | 1983 | 352 | 5714 | 1104 |
| **合计** | | **3290** | | **17750** |

- **试点目录**：css-values 256 files / 736 passes、selectors 274 /
  2,001——zw_passes > 0，未覆盖子测试清零路径成立。
- **css-variables**：58 files / 232 passes，与 DC-3 快照逐值一致
  （重复性佐证）。
- 覆盖文件/通过数略低于分片产物口径（如 grid 648 files/1,243 passes 对
  产物 653 案/1,251 passes）= 管道按 wpt.fyi 全量清单交集记账，上游清单
  无对应路径的案不进分子分母（全 goal 计 402 条 zw-only，与 10-29 的
  390 同形态）；顶目录行 3,293 = 十六目录 3,290 + css/css-scroll-snap
  3 files（0 passes，coverage-2026-10-29 复核已记的其他通道顺带页）。

## 验证矩阵

| 项 | 结果 |
| --- | --- |
| 运行 41005658107 | 17/18 分片 success，css-face-a/b/c（36m/25m/33m，≤210min 预算）+ misc 全绿；form-encode 分片（form/url-encoding 域）超 per-suite 限制失败 |
| 三分片产物案数 | 16/16 目录与本地基线一致（产物逐 JSON 复核） |
| 三分片产物子测试 | 15/16 目录 ±20 以内（时序带）；sizing 分母重测 2,238（fresh 本地 ≡ CI 逐案全等） |
| 坍缩形态复核 | masking 544/anchor-position 12,793/backgrounds 1,085——修复生效 |
| css-variables CI 值 | 58 files / 232 passes，与 DC-3 快照逐值一致（重复性佐证） |
| coverage 快照 | 本地聚合 17 分片产物产出（CI aggregate 被 needs 门跳过），提交 6102d71c3；css 顶目录 3,293 files / 17,750 passes / 9.08%，16 目录全入组且 zw_env_fails=0 |
| wpt-suites.csv | 汇总行追加（十六目录 65,501/17,781 修正口径 + DC-4 快照注记），trend_csv_check 过 |

## 边界与遗留

- **失败簇归口**（后续 goal 素材，不阻 M4）：typed-om API 面（96.7% Fail）、
  anchor-size() 序列化参数序 ~2,500、grid 布局探测 3,280、color Color-4
  计算值 7,199、text 断行 approx 543——见 M3 各批 evidence 分簇账。
- **css-sizing 子测试声明差的通道语义**：单进程跑序残留不仅影响子测试
  status（批次 1 记 6 子测），也会压低页面声明数（本轮 1,894→2,238 实证，
  fresh 重跑与 CI 逐案一致后定性）。后续如需本地复现 CI 口径，以分片通道
  （每案独立进程）或 fresh 重跑为准；批次 1 evidence 的 sizing 分母行
  （1,894/701）由本轮重测口径（2,238/735）取代。
- **idl_test 4 页**：`/resources/idlharness.js` 本地语料亦缺，与 CI 同界；
  如未来补齐需同时核对 wpt-data 根 resources 抓取契约。
