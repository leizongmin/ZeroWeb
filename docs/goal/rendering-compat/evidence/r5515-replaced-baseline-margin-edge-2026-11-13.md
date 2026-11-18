# R5515 轮次证据：img 原子基线取 margin box 底边（CSS2 §10.8.1）——R5418 挂账①字段面销账＋009 残差面再归因（行进域）

日期：2026-11-13
轮次：R5515（R5514 后继）
择案：css-ui/box-sizing-009 残差（✗2.50%，R5418 挂账①「行内替换元素 baseline 应取下
margin edge（CSS2 §10.8.1，IFC 行内 baseline 独立语义面，chrome 双页 diff 0 铁证）」——
账本记名候选，账本 grep 含 evidence/*.md 简称形）。
结论：①baseline 字段面按挂账处方实现（collect_items.rs img 臂 `baseline: atom_h` →
`atom_h + margin_bottom`）——spec-真、锚 4 针、lib 2502P/0F、corpus 真态 685F 持平
（FIXED=0 REGRESSED=0）；②**归因细化（本轮新账）**：009 残差不在 baseline 字段——
ZW 案内 img 原子落位本就正确（同线同顶、同底差 20 均对），残差实为**行进
（line advance）面**：ZW 行盒 advance 120（mt+atom_h 尺度）vs chromium 110 档
（R5418 gBCR「bb 120−10＝div 100＋10 行进等」），第 3 行起累计 +10px。挂账①
「行内 baseline 域」定名收窄为「IFC 行进/break 侧行高计账域」。

## 一、取证链（零编译）

- 案面：body max-width 700，3 行原子带——L1 ref div(300×100 m10) + t00；L2 t01
  （pl20 ml−10，mb+10）+ t02（pb20 bb120，mb−10）；L3 t03 + t04（h120 pb20 mb−10）。
- LAYOUT_DUMP 几何：ZW 同线 img 同顶（270/270、500/500 ✓）、同底差 20（bb120 vs 100）
  ✓——案内落位与 chromium 一致；行距 60→270→500（Δ210/230）非均匀＝行进域缺陷
  （chromium 依 R5418 gBCR：L3 应 −10）。
- ZW_DEBUG_IFC（runner layout-dump）：img 臂字段已折叠 mb（t02 h120 field 110＝
  120−10 ✓ 新代码生效；t00 field 110＝100+10 ✓）；行盒 h=120/baseline_y=120——
  字段变化被行高恒等吸收（落位 mt-不变量：img 支配 ascent 时 y = baseline_y − field
  = mt 与字段值无关）→ 像素 12000/480000 (2.50%) 与修前**逐位同值**。
- 探针双 pass 形态：measure/final 两趟 IFC（runs=4 h100 field90 vs runs=2 h120
  field110）并存——行进计账属 break 侧/两趟衔接域，非本字段面。

## 二、修复（collect_items.rs 单表达式 +15/−5）

- img 臂：`let img_mb = resolve_inline_margin(&s.margin_bottom, s)`；字段
  `baseline: atom_h + img_mb`、`margin_bottom: img_mb`（复用局部）。注释锚 §10.8.1
  ＋R3904 字段口径（border 顶→基线距离，margin_top 由 ascent 循环另加）＋mb=0 恒等
  零扰动＋驱动案注记。
- 消费面核查（全量 grep）：ascent 循环（mod.rs:3049，仅 Baseline|Sub|Super 参与——
  c44 族 text-bottom/top 不参与零影响；正 mb 行 ascent 计入全 margin box＝chromium
  行为）；va=Baseline 落位（mod.rs:3115）；refresh_reused_inline_block_metrics 臂
  （ib_sizes 只收 DisplayValue::InlineBlock，img UA display:inline 不入表→不覆盖）；
  其余＝debug dump 与透传（para_bidi 为测试直构）。写点唯一。

## 三、锚测试（r5515_replaced_baseline_margin_edge_tests，4 针）

①mb−10 单图：行基线 = mt+(50−10) = 50（旧 60，红态由值差算术确立——旧代码字段
border 底不可能产出 50/70/Δ20）；②同带 ±mb 双图（box-sizing-009 行 2 形）：基线 70、
b.y−a.y = 20（旧 60/0）；③mb=0 守卫：基线 60/y10 与旧逐字节同值；④正 mb 单图：
基线 70＝margin box 全高参与 ascent（R3904/R5394 同语境）。

## 四、验收

- lib：**2502P/0F**（2498 + 4 新锚，fmt 后态由 make test 复盖）。
- solo：box-sizing-009 ✗2.50% **纹丝未动**（12000px 逐位同值）＝字段非残差面的
  直接证伪——残差归行进域（见 §一/§五）。
- corpus 全量 A/B（16594 vs /tmp/r5514 轮 685F）：**TRUE-DIFF=8 全在册互翻族**——
  GONE=4（table-anon-128/run-in-inline-block-between-003/selection-input-011/
  intrinsic-percent-replaced-dynamic-007，恰 R5514 solo 已证 flake 集落绿相）、
  NEW=4（table-anon-129/149/150/153，在册 127/128/129/148/151/152/153/154 互翻族
  本窗相位）；**FIXED=0 REGRESSED=0，685F 真态持平**。
- make reftest：725/725（521 layout + 204 text）。
- make test：见 /tmp/r5515/make-test.log（后台跑，交付前核账）。
- product-smoke：welcome chromium oracle **17.23%**（R5501 基线同值中性）+ struct 全
  PASS；legacy 42 fixture struct-check **0 失败**。
- fmt（净）+ clippy `-p zero-layout-engine --all-targets -D warnings` 净。

## 五、挂账与下一步

- **挂账面修正**：R5418 挂账①「行内替换元素 baseline 取边语义」的字段面本轮销账；
  009 残差 ✗2.50% 转记新面——**IFC 行进域**：带 ±垂直 margin 的行内替换元素行的
  advance 计账（ZW 120 vs chromium 110 档；ZW_DEBUG_IFC 双 pass 形态在案，
  measure h100/field90 vs final h120/field110 两趟 split 不一致待查）。非轻量
  （跨 break 侧 margin-box 计账＋两趟衔接＋树回填三处口径），按深域记档。
- bench-gate 静窗收账（R5513 起跨轮挂账）**本轮销账**：watcher2 静窗 16:21 触发，
  GATE PASS（25 指标全绿）BENCH-EXIT=0，/tmp/r5514/bench-gate2.log 在案；
  target-disk-guard 全量清 target/（97GB 超阈）＝重腿前预编译产物尽失教训在案
  （零编译取证窗口需重估）。
- 下一步 R5516：①corpus FAIL 盘点续择轻（685F 清单复用 /tmp/r5515/fails-r5515.txt；
  避开行进新域/在册深域/待决策③④面/动态域；候选先查在册互翻记录与阈值盲区）②
  行进域深评（若择重再开：先单图 3 行最小形 LAYOUT_DUMP 定位树回填口径）③维持项
  ＋对端挂账观察＋重腿静窗纪律维持。

## 六、本轮动作与卫生

- 候选盘点续：css-display/color/conditional/ui/contain 带账本交叉全归档
  （flow-root float-avoidance 深域/t425 runner XML 分支域/opacity-overlap 特性级/
  currentcolor 复合特性域均排除在册；box-sizing-009 经 R5418 evidence 命中择中）。
- 临时探针全走零编译（layout-dump + ZW_DEBUG_IFC），无源码残留；探针留 /tmp/r5515/。
- bench-gate2 静窗期间零编译纪律保持（无重腿撞窗，GATE PASS 干净收账）。
