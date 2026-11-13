# Editing、Selection 与 Input Events控制面

**2026-11-14 · Active**。DC-1 落定：weekly 快照 109,059/154,595，对
11-07 基线净增 +92,875（CI
[run 42947347500](https://github.com/leizongmin/ZeroWeb/actions/runs/42947347500)）。
**d15 strike S2a 并拢门 + P5 td span retag 批**：face 1 断言级
**F2P=2/P2F=0**（22F→20F：strikethrough 7F→5F，bold/italic/underline/
sub/sup 名集逐文件恒等）。两刀全落 strike ADD css-false 门内：① S2a
sibCrit 同 tag 名并拢门——`fo[o<s>b]ar</s>baz` 期望
`fo<strike>o</strike><s>bar</s>baz`（:635），fresh strike 不并入相邻
`<s>`（bold/strong 排除门同款规则面，门限 strike——underline 对称行
录形相反就地并入）；② P5 td decl span 原位 retag 裸 `<s>`
（createElement 干净壳迁子节点）+ **retag 后选区端点重锚**——multitest
序列上下文下 splitText 重绑后 range 位移槽与面值脱钩，move 补偿把
start 端点折叠进随迁节点（探针面包屑实证 s=e=#b@1、CONTAINED 空、
apply 空转；孤立探针同码绿不可复现），retag 前 setStart/setEnd 快照
同值重写刷新槽 + 重挂后回写兜底。哨兵 34 名集恒等；钉测 7/7；常驻
探针 zz-probe-d15strike 两 case 入库 2/2 绿；selection 语料未重跑
（改动全在 strike ADD 门内不可达其他命令，d14 全 selection P2F=0
基线维持）。上轮 64 行 strike REMOVE 方向规则草稿移除未验证，S2e
（:640 mixed qS=false 而方向 REMOVE）为下批方向规则候选。证据见
[dc2-strike-s2a-p5-retag](evidence/2026-11-13-dc2-strike-s2a-p5-retag.md)。
上一批 **d14 strike 内部界 peel + sub/sup 残段保面批**：face 1 断言级
**F2P=15/P2F=0**（37F→22F：strike 16F→7F、sub 3F→0F、sup 3F→0F）。
四刀：S1 族 composite end 侧内部界解析 + peelOne 按壳值补裹拖出段；
三门控收面（命令门/longhand 序列化门/drag 端点门）；sub/sup 残段 retag
最深 va 壳面（vaInT）+ 贴边 sandwich 门；s 壳收编序列化槽（S2c longhand）。
方法学教训入档：探针期望必须逐条从 data/*.js 直读。证据见
[dc2-va-peel-interior-strike-shell](evidence/2026-11-13-dc2-va-peel-interior-strike-shell.md)。
更早批次：**d13 sub/sup va 壳批**（F2P=20/P2F=0，57F→37F；同批定谳
d9 账目②瓦解 = 语料切片割裂伪失败——run 页 meta variant 割断条目，
Chrome 于 wpt.fyi 同红；六文件全谱重分类，证据
[dc2-sub-sup-va-shell](evidence/2026-11-13-dc2-sub-sup-va-shell.md)、
learnings bugs 2026-11-13-wpt-editing-corpus-truth-sources）；**d12
桥层修复批**（bold 700/900 残余根因 = plain 树插入侧 live-range 边界
补偿缺失，plain mutator 接 R263，F2P=4，61F→57F，证据
[dc2-bold-toggleoff-universe-diag](evidence/2026-11-13-dc2-bold-toggleoff-universe-diag.md)）；
**d9 italic ADD 壳内嵌套 + collapsed qS 记账**（F2P=5，证据
[dc2-italic-add-shell-collapsed-qs](evidence/2026-11-13-dc2-italic-add-shell-collapsed-qs.md)）；
**d8 italic toggle-off 值面**（F2P=12，证据
[dc2-italic-toggleoff-face](evidence/2026-11-13-dc2-italic-toggleoff-face.md)）；
③ bold/italic 载体吸收与边界挖入（d7 批 +55，bold 42F→13F、italic
24F→18F）、P1 读面 +114、P2 apply 面 +1,647、P3 作者载体 +26、strike
新裹标签 +37、主循环 ⑤ 元素载体折叠 +10、① decl 面壳多值偏序 +70、
② bold 数值权重边界 + cef 门 rc 面 +60（证据见下表与
[evidence/](evidence/) 目录）。
selection 扩面已接线待下周跑生效：fetch 切 deep（184 案）、独立
selection-face 片——本地 census 51,695/35,631（
[census](evidence/2026-11-11-dc2-selection-face-census.json)）。
**卡点** 无阻塞；距 146,184 验收线差 37,125，大头在 selection deep
面下周跑生效（census 口径 +~33k）与 input-events（201/3,378）等 0 覆盖面
接线。六 format 文件残余 20F（bold 8F、italic 3F、u 4F、strike 5F、
sub/sup 0F；d15 收账后），d13 重分类为 **伪 ~17F**（切片割裂 harness
固有——bold 6F = 条目 139/75/204、italic 1F = 条目 131、u 2F = 条目
65/133、strike 1-2F，任何浏览器同红，不进修复队列）；**真 ~11F**：
strike 5F = S2e 方向族（:640 REMOVE 方向，方向规则草稿候选）+ M1
multipara 裸 span 溶解 + qState 三条（`<strike>foo[bar]baz</strike>`
qS-after、`foo<s>[bar]</s>baz` qS-after/qS(stylewithcss)-before，
切片伪失败族待复核）；b-in-i/b-in-s 块边界嵌套 bold 2F + italic 2F、
multipara u 2F + u-qS 条目 134 维持。桥层支线三 bug（tc.querySelector
innerHTML 重建后空包装、addRange 后 getRangeAt(0) 越界、B2 臂双落地）
未动，主链经 R263 补偿后已与 B2 臂解耦。
**下一步** ①strike 残余 5F（S2e 方向规则——上轮 backup 草稿
/tmp/d15cmp/part01d-d15-backup.js 64 行起界两侧判向、M1 multipara）
与 multipara/跨块族（M1/M2/B1-B4 per-p 拆壳）评估开修，selection
editing/other ~47 慢页补全；②selection bidi（0/92）、shadow-dom（27/113）、
input-events 小桶按产物归类开修；③下周快照落做 DC-3 同口径对账（净增
≥130,000 + 邻域回退核对，P2/P3 语料全量增量随周跑入账；face 哨兵
探针：d5decl/d5diag/d6cef/d7diag/d10diag/d11tree/d13vashell 全绿、
d6diag 24/25、d8diag 11/11、d9diag 10 case 全绿、d15strike 2/2）。

[执行契约](../wptfyi-editing.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成（11/11 片；快照 109,059/154,595） | [dc1-run-shards](evidence/2026-11-11-dc1-run-shards.md) + [dc2-selection-wiring](evidence/2026-11-11-dc2-selection-wiring.md) |
| DC-2 按簇收敛 | P1 读面 +114；P2 apply 引擎（多面 +1,647 断言级零新增失败）；P3 作者载体臂 + strike/⑤/①/② 各批 + ③ 载体吸收（+55）+ ⑥ italic toggle-off（+12）+ ⑥ ADD 壳内嵌套 + collapsed qS 记账（+5）；均零回归；selection 扩面已接线 | [triage](evidence/2026-11-11-dc2-fail-cluster-triage.md) + [dc2-p2-format-apply](evidence/2026-11-11-dc2-p2-format-apply.md) + [dc2-us-carrier-author-arm](evidence/2026-11-12-dc2-us-carrier-author-arm.md) + [dc2-strike-wrap-tag](evidence/2026-11-12-dc2-strike-wrap-tag.md) + [dc2-element-carrier-fold](evidence/2026-11-12-dc2-element-carrier-fold.md) + [dc2-decl-shell-partial-order](evidence/2026-11-12-dc2-decl-shell-partial-order.md) + [dc2-bold-value-face](evidence/2026-11-12-dc2-bold-value-face.md) + [dc2-bold-italic-carrier-merge](evidence/2026-11-13-dc2-bold-italic-carrier-merge.md) + [dc2-italic-toggleoff-face](evidence/2026-11-13-dc2-italic-toggleoff-face.md) + [dc2-italic-add-shell-collapsed-qs](evidence/2026-11-13-dc2-italic-add-shell-collapsed-qs.md) + [dc2-bold-toggleoff-universe-diag](evidence/2026-11-13-dc2-bold-toggleoff-universe-diag.md)（d10–d12：宇宙面定谳 + 桥层修复落地，bold 12F→8F 零回归）+ [dc2-sub-sup-va-shell](evidence/2026-11-13-dc2-sub-sup-va-shell.md)（d13：sub/sup va 冲突壳 + 语料红账重分类，F2P=20 零回归，六文件 57F→37F）+ [dc2-va-peel-interior-strike-shell](evidence/2026-11-13-dc2-va-peel-interior-strike-shell.md)（d14：strike S1 内部界 peel + sub/sup 残段保面 + s 壳收编槽，F2P=15 零回归，六文件 37F→22F）+ [dc2-strike-s2a-p5-retag](evidence/2026-11-13-dc2-strike-s2a-p5-retag.md)（d15：strike S2a 同 tag 名门 + P5 td span retag 端点重锚，F2P=2 零回归，六文件 22F→20F） |
| DC-3 全量对账 | 对账口径就绪（净增 +92,875，距线 37,125） | coverage-2026-11-11 快照 + 验收线 146,184 |

碰撞记录：本批 pin 钉测与并行流 D1 的 workspace 全量 test+clippy 共
享 build 目录锁（2026-11-13），改用独立 CARGO_TARGET_DIR 完成钉测，
未触碰对方进程；后续重型编译撞锁时同法分流或等待。
