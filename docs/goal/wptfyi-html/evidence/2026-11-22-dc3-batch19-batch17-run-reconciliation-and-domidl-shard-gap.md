# DC-3 · 批十九：批十七 run 45008050513 收账＋dom.idl 分片缺档定谳与补档＋批十八 run 派发

2026-11-22。run 45008050513（deb7fbfb0，批十七码）aggregate 快照落 main
（9a30b4394，11:20Z）逐通道收账；fix＝40 脚本补拉 `interfaces/dom.idl`
（+6 行）；run 45017894086（4fb8af586，承载本修复＋批十八 scripting-1
语料）11:33Z 派出。

## 批十七 run 逐通道收账（44996021189 → 45008050513）

| 通道 | 批十六 | 批十七 run | Δ | 判定 |
|---|---:|---:|---:|---|
| html/webappapis | 586 | 1,537 | +951 | 恰兑现计划面（键级证明见下） |
| html/syntax | 2,747 | 2,790 | +43 | speculative 瞬态窗回位（126 键 43P 面恢复） |
| html/infrastructure | 629 | 635 | +6 | messagechannel.any.worker.js 109→115 恢复带（键级核对 115P/21F/1PrecondF） |
| html/semantics | 10,466 | 10,464 | −2 | checkable-active-onblur-with-click.html 5P→3P（onblur flaky 族，html-semantics-compat 邻流已知摆动键 s635 同族），归抖动 |
| html/dom | 59,634 | 59,634 | 0 | 三查询键 +~1,394 落空 → dom.idl 缺档（本轮主项，见下） |
| browsers/rendering/xhtml/canvas | 持位 | 同 | 0 | 持位 |

目标级账（快照口径）：html 面 106,394→**107,392**（净 **+36,296**）；
全量 1,917,532→1,918,680（+1,148）。DC-3 验收 ≥152,096 → **余量 44,704**。

### webappapis +951 键级兑现（syntax-dom 分片 A/B）

- 8 个 event-handler 页 0P→919P：all-global-events 267、frameset-window 117、
  body-window 117、window 112、window-frameset 112、body-alt 97、
  frameset-alt 97——html.idl 运行期 fetch 兑现（批十六 −919 面全数回位）。
- base64.any.js 278→310（**+32**）＝勘误补档 base64.json 兑现（atob 节
  94 子测试注册回位）。

### dom 三查询键落空定谳（批十六「零注册」初判的真相）

syntax-dom 分片产物逐键核对：三键**并非零注册**，各注册 1 子测试
`idl_test setup` **Fail**，message＝`Error fetching /interfaces/dom.idl.`
——批十六 A/B delta=0（两轮同形）掩盖真形。

根因＝分片语料隔离律（批十五已立）第 4 例：批十六补拉 13 个 IDL 时
「除 dom.idl 外逐个补拉」，前提「dom.idl 同目录在册」本地为真
（2026-10-27 webgl M2 由 99-webgl-compat.sh 补拉带入，23,163B 在盘），
CI 为假——99-webgl-compat.sh 不在 syntax-dom 分片 fetch 清单，CI 分片
只跑本 goal fetch 恒缺。旁证：批十七修复 WebIDLParser 别名随 57/59 入
分片后，setup 链只卡 dom.idl 一件（WebIDLParser/idlharness.js/html.idl
全部在位）。

### 邻居观察（不入本流账）

dom/ranges −106＝Range-mutations-replaceData −76／Range-mutations-dataChange
−30（两键 Timeout 截断形，批十六 −13 同族扩形），归 wptfyi-dom 账本；
editing/run、css 面随全量 +1,148 另账。

## 修复：40 脚本补拉 interfaces/dom.idl

`fetch_raw "interfaces/dom.idl"`（+6 行，注释记根因链）。验证按批十七
教训①口径：`bash -n` 通过；**挪走**本地 dom.idl 后实跑脚本 exit=0、
文件重拉回且与原文件 `cmp` 字节级相同——幂等跳过路径零验证力教训
不再犯。

## 批十八 run 派发（45017894086，4fb8af586）

批次十七收账完成后派出（concurrency cancel-in-progress 风险解除），
承载：批十八 scripting-1 .any.js window 形语料（预期 html/semantics
+~234P，本地基线 234P/20F）＋本批 dom.idl 修复（预期 html/dom 三键
+~1,394P，CI 90s 级页预算观察点——5,822 子测试注册若超预算 TO 则
键级定谳续批）。预期 html 面 ~109,020（净 ~+37,900，余量 ~43,100）。

## 前置账勘误承接

safe-passing 286S 重分类（本轮前置账勘误）按本快照统一重排：跑道主序
缺失子测试 14,413→**14,127**（文件 2,768 持位）；safe-passing 移入行为
刀位（MessagePort instanceof 接口对象缺位 ~150–200S＋iframe 内容门
117S，待 engine 碰头）。跑道逐簇拉取顺延批二十起。

[逐文件 JSON](2026-11-22-dc3-batch19-batch17-run-reconciliation-and-domidl-shard-gap.json) ·
[批十八证据](2026-11-22-dc3-batch18-scripting1-module-anyjs-corpus.md) ·
[批十七证据](2026-11-22-dc3-batch17-webidlparser-alias-and-rerun.md) ·
[前置账](2026-11-22-dc3-goal-accounting-and-corpus-runway.md)
