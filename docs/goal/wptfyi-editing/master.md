# Editing、Selection 与 Input Events控制面

**2026-11-14 · Active**。DC-1 落定：weekly 快照 109,059/154,595，对
11-07 基线净增 +92,875（CI
[run 42947347500](https://github.com/leizongmin/ZeroWeb/actions/runs/42947347500)）。
**d18 Selection 校验门批**：collapse/extend/addRange 三道规范校验门落
part06.js 主窗口 Selection（spec 步骤序：addRange 非 Range TypeError +
端点 root 非本文档完全 no-op + rangeCount≠0 no-op；collapse in-doc
no-op、doctype/offset 抛错先于 no-op；extend contains 门先于空选区
InvalidStateError）。验证：同机受控六文件 before/after
**F2P=6,146/P2F=0/new-fail=0**；selection 全语料对拍
**F2P=11,481/P2F=0/new-fail=0、vanished=1,740**（addRange second 测试
分母收缩非回归，净 -13,201F，16,004→2,803）；face 六文件与哨兵 53 页
ledger 零回归（prefix 对拍）。证据见
[dc2-selection-validation-gates](evidence/2026-11-14-dc2-selection-validation-gates.md)。
上一批 **d17 窗口数学重分类批**（零引擎改动）：六 format 文件残余全谱
重定谳——伪失败 13F（run 页 meta variant 切片割裂，任何浏览器同红）
出修复队列、真失败 5F（strike S2e qS-after 挂账 Chrome-quirk 家族 +
bold b-in-i / italic i-in-b 块边界 innerHTML 2+2F）；d16 账目勘误
face 20F→18F。证据见
[dc2-window-math-reclassification](evidence/2026-11-14-dc2-window-math-reclassification.md)。
**d16 strike REMOVE 方向规则 + u/s css-true 裸 span 吸收批**：
face 断言级 F2P=2/P2F=0。两刀落
part01d.js；常驻探针 zz-probe-d16strike 2/2。证据见
[dc2-strike-direction-m1-absorb](evidence/2026-11-14-dc2-strike-direction-m1-absorb.md)。
更早批次：**d15 strike S2a 并拢门 + P5 td span retag 批**（F2P=2，
22F→20F，
[证据](evidence/2026-11-13-dc2-strike-s2a-p5-retag.md)）；
**d14 strike 内部界 peel + sub/sup 残段保面批**（F2P=15/
P2F=0，37F→22F；方法学教训入档：探针期望必须逐条从 data/*.js 直读，
[证据](evidence/2026-11-13-dc2-va-peel-interior-strike-shell.md)）；
**d13 sub/sup va 壳批**（F2P=20/P2F=0，57F→37F；定谳 d9 账目②瓦解 =
语料切片割裂伪失败，Chrome 于 wpt.fyi 同红；六文件全谱重分类，
[证据](evidence/2026-11-13-dc2-sub-sup-va-shell.md)与 learnings bugs
2026-11-13-wpt-editing-corpus-truth-sources）；**d12 桥层修复批**
（plain mutator 接 R263，F2P=4，61F→57F，
[证据](evidence/2026-11-13-dc2-bold-toggleoff-universe-diag.md)）；
**d9 italic ADD 壳内嵌套 + collapsed qS 记账**（F2P=5，
[证据](evidence/2026-11-13-dc2-italic-add-shell-collapsed-qs.md)）；
**d8 italic toggle-off 值面**（F2P=12，
[证据](evidence/2026-11-13-dc2-italic-toggleoff-face.md)）；
③ bold/italic 载体吸收与边界挖入（d7 批 +55，bold 42F→13F、italic
24F→18F）、P1 读面 +114、P2 apply 面 +1,647、P3 作者载体 +26、strike
新裹标签 +37、主循环 ⑤ 元素载体折叠 +10、① decl 面壳多值偏序 +70、
② bold 数值权重边界 + cef 门 rc 面 +60（证据见下表与
[evidence/](evidence/) 目录）。
selection 扩面已生效：d18 三门为 deep 面首刀（全语料 +11,481）；11-12
快照已含 deep 接线（184 案 fetch 切 deep、独立 selection-face 片）——
本地 census 51,695/35,631（
[census](evidence/2026-11-11-dc2-selection-face-census.json)）。
**卡点** 无阻塞。11-12 快照口径：目标三目录合计 130,245、距 146,184
差 15,939（前文"差 37,125"系 11-07 基线口径未刷新，勘误）；d18 后
selection 全语料 47,147P（较 11-12 快照 +11,677），三目录预估 ~141.9k、
差 ~4.3k（预估口径，正式落账待下周 DC-3 快照）。六 format 文件残余
19F（HEAD 基线口径，d17 账 18F 勘误 +1）：bold 9F、italic 3F、
strike 4F、underline 3F、sub/sup 0F。重分类终账：伪失败 13F（切片
割裂 harness 固有，不进修复队列）= strike/underline 条目 65/133/134 +
bold 条目 75/139/204 + italic 条目 131；真失败 6F：strike S2e qS-after
1F（挂账 Chrome-quirk 家族）+ bold b-in-i / italic i-in-b 块边界
innerHTML 2+2F + bold notbold 条目 1F（sw=true innerHTML 内层裸 span，
prefix 对拍定谳 HEAD 既有欠账非 d18 引入，溯源指向 02:12 前落地批
20521a2ba/8add42713 一带，待 bisect）。哨兵 fncorpus 两窗页级 Timeout
（d15 已有，fontname 域待查小桶）；f6bwin 两窗（并行流新探针）页级
超时非门面。桥层支线三 bug（tc.querySelector innerHTML 重建后空包装、
addRange 后 getRangeAt(0) 越界、B2 臂双落地）未动，主链经 R263 补偿
后已与 B2 臂解耦。
**下一步** ①mt 慢页 systemd-run 长预算（d13 口径，MemoryMax 封顶）
补跑 strike/u/bold/italic 链对拍 + multipara/跨块族（M2/B1-B4
per-p 拆壳，真失败 b-in-i/i-in-b 块边界 innerHTML 4F 同域）评估开修，
bold notbold 1F bisect 溯源（20521a2ba/8add42713 一带），
selection editing/other ~47 慢页补全；②selection bidi（0/92）、
shadow-dom（27/113）、input-events 小桶按产物归类开修；③下周快照落做
DC-3 同口径对账（净增 ≥130,000 + 邻域回退核对，P2/P3 语料全量增量随
周跑入账；face 哨兵探针：d5decl/d5diag/d6cef/d7diag/d10diag/d11tree/
d13vashell 全绿、d6diag 24/25、d8diag 11/11、d9diag 10 case 全绿、
d15strike 2/2、d16strike 2/2、d18sel 3/3；组成漂移后哨兵判读改
ledger 集合判读 P2F/new-fail）。

[执行契约](../wptfyi-editing.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成（11/11 片；快照 109,059/154,595） | [dc1-run-shards](evidence/2026-11-11-dc1-run-shards.md) + [dc2-selection-wiring](evidence/2026-11-11-dc2-selection-wiring.md) |
| DC-2 按簇收敛 | P1 读面 +114；P2 apply 引擎（多面 +1,647 断言级零新增失败）；P3 作者载体臂 + strike/⑤/①/② 各批 + ③ 载体吸收（+55）+ ⑥ italic toggle-off（+12）+ ⑥ ADD 壳内嵌套 + collapsed qS 记账（+5）；均零回归；selection 扩面已接线 | [triage](evidence/2026-11-11-dc2-fail-cluster-triage.md) + [dc2-p2-format-apply](evidence/2026-11-11-dc2-p2-format-apply.md) + [dc2-us-carrier-author-arm](evidence/2026-11-12-dc2-us-carrier-author-arm.md) + [dc2-strike-wrap-tag](evidence/2026-11-12-dc2-strike-wrap-tag.md) + [dc2-element-carrier-fold](evidence/2026-11-12-dc2-element-carrier-fold.md) + [dc2-decl-shell-partial-order](evidence/2026-11-12-dc2-decl-shell-partial-order.md) + [dc2-bold-value-face](evidence/2026-11-12-dc2-bold-value-face.md) + [dc2-bold-italic-carrier-merge](evidence/2026-11-13-dc2-bold-italic-carrier-merge.md) + [dc2-italic-toggleoff-face](evidence/2026-11-13-dc2-italic-toggleoff-face.md) + [dc2-italic-add-shell-collapsed-qs](evidence/2026-11-13-dc2-italic-add-shell-collapsed-qs.md) + [dc2-bold-toggleoff-universe-diag](evidence/2026-11-13-dc2-bold-toggleoff-universe-diag.md)（d10–d12：宇宙面定谳 + 桥层修复落地，bold 12F→8F 零回归）+ [dc2-sub-sup-va-shell](evidence/2026-11-13-dc2-sub-sup-va-shell.md)（d13：sub/sup va 冲突壳 + 语料红账重分类，F2P=20 零回归，六文件 57F→37F）+ [dc2-va-peel-interior-strike-shell](evidence/2026-11-13-dc2-va-peel-interior-strike-shell.md)（d14：strike S1 内部界 peel + sub/sup 残段保面 + s 壳收编槽，F2P=15 零回归，六文件 37F→22F）+ [dc2-strike-s2a-p5-retag](evidence/2026-11-13-dc2-strike-s2a-p5-retag.md)（d15：strike S2a 同 tag 名门 + P5 td span retag 端点重锚，F2P=2 零回归，六文件 22F→20F）+ [dc2-strike-direction-m1-absorb](evidence/2026-11-14-dc2-strike-direction-m1-absorb.md)（d16：strike REMOVE 方向规则 + u/s css-true 裸 span 吸收，F2P=2 零回归，六文件 20F→18F）+ [dc2-window-math-reclassification](evidence/2026-11-14-dc2-window-math-reclassification.md)（d17：窗口数学重分类，零引擎改动，伪失败 13F 定谳出队、真失败收敛 5F，d16 账目勘误 20F→18F）+ [dc2-selection-validation-gates](evidence/2026-11-14-dc2-selection-validation-gates.md)（d18：Selection collapse/extend/addRange 三规范门，受控六文件 F2P=6,146、全语料对拍 F2P=11,481、vanished 1,740，零回归） |
| DC-3 全量对账 | 对账口径就绪（11-12 快照三目录 130,245、距线 15,939；d18 后 selection 全语料 47,147P，预估差 ~4.3k，正式落账待下周快照） | coverage-2026-11-11 快照 + 验收线 146,184 + [d18 selection 全语料对拍](evidence/2026-11-14-dc2-selection-validation-gates.md) |

碰撞记录：本批 pin 钉测与并行流 D1 的 workspace 全量 test+clippy 共
享 build 目录锁（2026-11-13），改用独立 CARGO_TARGET_DIR 完成钉测，
未触碰对方进程；后续重型编译撞锁时同法分流或等待。
