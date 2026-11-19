# d62 · DC-3 coverage 判决（141,068 < 146,184 不达线）＋ d57 C-view 回归归因 ＋ d61 刀 F 验证

**2026-11-19 · d62 对账轮**。上轮 dispatch 的 DC-3 coverage 跑（weekly.yml
`run_wptfyi_coverage=true`，run [44486440940](https://github.com/leizongmin/ZeroWeb/actions/runs/44486440940)，
基 d59=4f1964625，2026-11-19T08:18–09:46Z success）产物已由 weekly cron 落库
[coverage-2026-11-19.json](../../../compat/wptfyi/coverage-2026-11-19.json)/[.md](../../../compat/wptfyi/coverage-2026-11-19.md)。
本轮下载两代 run 全部 editing/selection/ui-edit 分片原始 JSON（CI artifacts，
11-17=run 44092733559 对照）逐键对账，完成 DC-3 判决与回归归因；同轮补验上会话
中断遗留的 d61 刀 F WIP（工作树 part04.js，未提交）。

## 1. DC-3 判决：三目录 141,068 < 146,184，不达线

| topdir | 11-17 快照 | 11-19 快照 | Δ | 分母 |
|---|---:|---:|---:|---:|
| editing | 109,522 | 104,648 | **−4,874** | 117,246（两代同） |
| selection | 34,041 | 34,041 | 0 | 34,532（同） |
| input-events | 1,476 | 2,379 | **+903** | 3,378（同） |
| **三目录合计** | **145,039** | **141,068** | **−3,971** | 距线 146,184 差 5,116 |

上游 aligned run 换代（chrome 157→158、revision 81999813d5→71b58bb191）但三目录
分母恒等——editing −4,874 是**真实回归**，非分母漂移。全过文件 editing 376→214
（−162），失败集中于 join-white-space 族（§2）。

净增口径（对 2026-11-07 立项基线 16,184）：141,068 − 16,184 = **净 +124,884**，
距 ≥130,000 验收线差 5,116。**DC-3 不判 DONE**。

关键推算：join-white-space 回归（−5,192，§2）全部落 editing topdir；若回收该
回归并保住 d57–d59 input-events 增益，三目录 ≈ 146,260 ≥ 146,184——**修复该回归
是达线的唯一已知路径，且余量仅 ~76**（d61 刀 F +10 输入域增益未入本轮跑，可补
少量余量）。

## 2. editing −4,874 归因：d57 刀 C-view 粘性 canonical 重现 d47 族毒

selection 套件原始分片逐键对账（selection-face + selection-ws 合并）：

- 11-17：53,216 键 / 48,960 Pass
- 11-19：53,168 键 / 43,768 Pass（**净 −5,192**）

按路径归 family：

| family | ΔPass |
|---|---:|
| join-different-white-space-style-paragraphs.html | −2,707 |
| join-different-white-space-style-left-paragraph-and-right-line.html | −1,296 |
| join-different-white-space-style-left-line-and-right-paragraph.html | −1,127 |
| join-pre-and-other-block.html | −65 |
| delete-without-unwrapping-first-line-of-child-block.html | −20 |
| 零散增益 | +23 |

**与 d47 判负指纹逐族吻合**（d47 证据：join white-space 三族 −5,449 +
join-pre-and-other-block −64 + dwuf/plaintext-only insertLineBreak 各 −24）。
失败形抽查：`delete at <div style=normal>abc</div>\n<div style=pre>def</div>`
断言 join 后 white-space 由 span 保持，got=**原样未 join 的 pre 态**——join 变异
不进读取投影，即 d47 定谳的「join 族 innerHTML 读旧投影大面积红」。

**本地 bisect 定代**（`testharness-selection "join-different-white-space-style-paragraphs"`
过滤，test-guard 包裹，同一 wpt-data）：

| runner | 代 | Pass/键 |
|---|---|---|
| d56-clean-runner | d56 终态（pre-d57） | **3,456/3,456** |
| d57-final-runner | d57 落地（C-view+刀 A 重放） | **749/3,456** |

→ 回归由 **d57 刀 C-view**（`crates/engine/src/js_dom_bridge/css_wire.rs`
`STICKY_QUERY` 粘性 canonical 层）引入。机制与 d47 判负同解域：粘性 memo 跨
materialize 存续保 wrapper 身份连续（dil 族/harness churn 面的收益来源），但
join-white-space 族要求每次 mutation 换代后查询走新投影——两族要求相反，
resolution 自验证（memo 串仍解析回本次节点）不保证投影新鲜度。d57 当时的哨兵面
（dom/uievents/selce CE 过滤 430 键 + input-events 全套件）不覆盖 join-white-space
族（其属于 selection 通道 editing/other 语料），致毒性漏检；DC-3 coverage 跑
首次曝光。

d58/d59/d61 刀（part04 wire 面）同过滤腿 bisect 数字见 §4（后台腿，均预期与
d57-final 同位——它们不触碰查询返回点）。

## 3. d61 刀 F（T1 dead-anchor 章对齐）A/B——上会话 WIP 补验

工作树遗留 WIP（part04.js +29/−3，三 hunk）：R125 remove 臂 stamp
`__zwLastRemoveWireSel`（摘挂 wire 章键）＋ R334/R334b 重插臂在 removed 标记仍活
的重插窗口消费摘挂章改发 adjacent wire（引擎 R361 detached-stash 键对齐，移动
语义复用被摘 NodeId）；非摘挂窗口维持身份章零行为变化。对应 d60 §2 T1 死锚
wire op 定谳的修向（adjacent-move 锚已在 remove 集时以摘挂章寻址）。

- **bs15 隔离页**：T1 join `"<div><b>abc</b></div>"`（def 丢失）→
  `"<div><b>abc</b><b>def</b></div>"`（def 保位；两 `<b>` 壳未合一属 join 收口
  另面，T2/T4/T7 恒定）。`/tmp/d61-bs15-{base,knife}.json`。
- **input-events 全套件**（base=d61-base-runner 即 d59 HEAD stash roundtrip，
  wip=d61-knife-runner）：共键口径 **F2P=10/P2F=0/other-flip=0**，base
  4,529P → wip 4,565P（+36）。F2P 全落在 bold 壳 no-join 簇：真语料
  backspace +1 / forwarddelete +2（`<div><b>abc</b><p><b>[]def</b></p></div>`
  主形），zz-diag-d56 T1、zz-iso-bs-06、zz-probe-b3p/q/r、zz-probe-d56j/jt1
  探针镜像各 +1。new=34/gone=2 = deleting-in-list-items 变体超时截断伪影
  （base 侧 2 Timeout 键在 knife 侧展开为 34 键，8 键 NonPass）。
- **哨兵**：knife 侧 dom 57,918 键（57,219P）/uievents 2,401（2,096P）/selce 430
  （311P）已跑；base 侧 A/B 腿与 join-white-space 过滤腿本轮补跑（§4）。

## 4. 后台腿（本轮补跑，全部收口）

- join-white-space 过滤腿 bisect（`/tmp/d62-jws-*.json`，test-guard 900s）：
  **d58-knife 749/3,456、d59-final 749/3,456、d61-base 749/3,456、d61-knife
  749/3,456**——d57 后各刀（含 d61 刀 F）在该面全部持位（中性互证，exit=1 系
  套件含失败键的预期退出码）。
- d61 base 侧哨兵 A/B（`/tmp/d61-*-base.json` × knife 同 wpt-data）：
  **dom 共键 F2P=0/P2F=0/other-flip=0**（new=111/gone=0，new 键 106P+5F 全落
  Range-mutations-dataChange 风暴页截断伪影）；**uievents 2,400 共键
  F2P=0/P2F=0 new=0/gone=0**；**selce 430 共键 F2P=0/P2F=0 new=0/gone=0**。
- enginelib（d61 树＝main 5a5e34a7c＋d61 diff，独立 CARGO_TARGET_DIR）：
  **4,230P/0F**（exit=0，40.78s；+3 测试＝并行流 css animation-range 入树）。

## 5. 下一步（下轮主线）

join-white-space 回归回收——查询视图代际分桶（master ③ 重设计域，d42/d47 两次
入口级判负后仅存解域）或 C-view 粘性层的 mutation 点失效面收窄；约束：
(a) 保住 d57–d59 input-events +903 与 editing-run +318 增益；
(b) 哨兵集必须加 join-white-space 过滤腿（本轮教训：selce-only 哨兵漏检族毒）；
(c) 回收后三目录预估 ≈146,260，达线余量 ~76，d61 刀 F 落地可再补 ~10。
