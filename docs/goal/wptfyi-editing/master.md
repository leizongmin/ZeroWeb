# Editing、Selection 与 Input Events控制面

**2026-11-14 · Active**。DC-1 落定：weekly 快照 109,059/154,595，对
11-07 基线净增 +92,875（CI
[run 42947347500](https://github.com/leizongmin/ZeroWeb/actions/runs/42947347500)）。
**d16 strike REMOVE 方向规则 + u/s css-true 裸 span 吸收批**：face 1
断言级 **F2P=2/P2F=0**（20F→17F：strike 5F→4F、underline 4F→3F，
bold/italic/sub/sup 名集逐文件恒等）。两刀全落 part01d.js：① strike
toggle 方向门改判——REMOVE ⟺ 选区起点位置（文本尾边界前移规范化、
off=0 头边界取父链）祖先链 line-through 面命中，不用 fq.state（S2e
:640 stateBefore=false 而方向 REMOVE；方向与 qS 读数分途；门限
strike——underline 同形录形相反维持原臂）；② u/s css-true ADD 面全含
裸 span（无 text-decoration decl + spec 无命令值）run 单成员并值不外
裹——run 连接按兄弟链空白扫描存在性判定（成员判定被 Range 半开界
:20 双 span 行打回 P2F）。S2e innerHTML 转绿但同行 qS-after 翻红净
零，与 :336/:685/:690 同族（exec 后选区位移映射 + Chrome 塌缩 qS
读面；:336 与 S2e 读数互为镜像证实该族系真实分叉非切片伪失败，解除
d13 待复核标签）。哨兵 34 名集 + 2 页级 Timeout（fncorpus 两窗，d15
已有）恒等；钉测 7/7；常驻探针 zz-probe-d16strike 两 case 入库 2/2 绿
（:640 孤立跑双 b 冷路径分叉，face 序列行为为语料权威）；mt 慢页
1500s 守卫内首窗未完零输出（d13 systemd-run 口径亦 7,865/9,832 截
断），本批零回归由 face1 顺序面 + 哨兵 40 页承担，mt 长预算对拍顺延；
selection 语料未重跑（改动全在 strike/underline execCommand 门内不可
达 Selection API 语料）。上轮 64 行方向规则草稿条款 2（起点前命中）
经语料核对撤——其证据行是把期望输出串误读为输入行（:527/:581 真输入
为 style 全含形，stateB=true 条款 1 即覆盖）。证据见
[dc2-strike-direction-m1-absorb](evidence/2026-11-14-dc2-strike-direction-m1-absorb.md)。
上一批 **d15 strike S2a 并拢门 + P5 td span retag 批**：face 1 断言级
F2P=2/P2F=0（22F→20F：strikethrough 7F→5F）。S2a 同 tag 名并拢门
（fresh strike 不并入相邻 `<s>`，bold/strong 排除门同款、门限
strike）+ P5 td decl span 原位 retag 裸 `<s>` + retag 后选区端点重锚
（multitest 序列上下文 splitText 重绑后位移槽与面值脱钩，retag 前同
值重写刷新槽 + 回写兜底）。哨兵 34 恒等；钉测 7/7；探针
zz-probe-d15strike 2/2。证据见
[dc2-strike-s2a-p5-retag](evidence/2026-11-13-dc2-strike-s2a-p5-retag.md)。
更早批次：**d14 strike 内部界 peel + sub/sup 残段保面批**（F2P=15/
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
selection 扩面已接线待下周跑生效：fetch 切 deep（184 案）、独立
selection-face 片——本地 census 51,695/35,631（
[census](evidence/2026-11-11-dc2-selection-face-census.json)）。
**卡点** 无阻塞；距 146,184 验收线差 37,125，大头在 selection deep
面下周跑生效（census 口径 +~33k）与 input-events（201/3,378）等 0 覆盖面
接线。六 format 文件残余 17F（bold 8F、italic 3F、u 3F、strike 4F、
sub/sup 0F；d16 收账后），d13 重分类延续：**伪失败**（切片割裂
harness 固有，任何浏览器同红，不进修复队列）= bold 条目 139/75/204
6F + italic 条目 131 1F + u qS-after 两变体（条目 65/133）2F；
**真失败 ~9F**：strike 4F 全 qS 读面族（S2e qS-after + :336/:685
qS-after + :690 qS-swc-before——d13 "切片伪失败待复核"标签经 :336/
S2e 读数互为镜像反证解除，根因层级 = exec 后选区位移映射 + Chrome
塌缩 qS 语义）+ u-qS 条目 134 + b-in-i/b-in-s 块边界嵌套 bold 2F +
italic 2F。哨兵 fncorpus 两窗页级 Timeout（tests:0 挂起，d15 已有）
属 fontname 域待查小桶。桥层支线三 bug（tc.querySelector
innerHTML 重建后空包装、addRange 后 getRangeAt(0) 越界、B2 臂双落地）
未动，主链经 R263 补偿后已与 B2 臂解耦。
**下一步** ①mt 慢页 systemd-run 长预算（d13 口径，MemoryMax 封顶）
补跑 strike/u/bold/italic 链对拍 + qS 读面族独立批（选区位移映射
补偿/读面分层，strike 4F 同根因）与 multipara/跨块族（M2/B1-B4
per-p 拆壳）评估开修，selection editing/other ~47 慢页补全；
②selection bidi（0/92）、shadow-dom（27/113）、input-events 小桶按
产物归类开修；③下周快照落做 DC-3 同口径对账（净增 ≥130,000 + 邻域
回退核对，P2/P3 语料全量增量随周跑入账；face 哨兵探针：d5decl/
d5diag/d6cef/d7diag/d10diag/d11tree/d13vashell 全绿、d6diag 24/25、
d8diag 11/11、d9diag 10 case 全绿、d15strike 2/2、d16strike 2/2）。

[执行契约](../wptfyi-editing.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成（11/11 片；快照 109,059/154,595） | [dc1-run-shards](evidence/2026-11-11-dc1-run-shards.md) + [dc2-selection-wiring](evidence/2026-11-11-dc2-selection-wiring.md) |
| DC-2 按簇收敛 | P1 读面 +114；P2 apply 引擎（多面 +1,647 断言级零新增失败）；P3 作者载体臂 + strike/⑤/①/② 各批 + ③ 载体吸收（+55）+ ⑥ italic toggle-off（+12）+ ⑥ ADD 壳内嵌套 + collapsed qS 记账（+5）；均零回归；selection 扩面已接线 | [triage](evidence/2026-11-11-dc2-fail-cluster-triage.md) + [dc2-p2-format-apply](evidence/2026-11-11-dc2-p2-format-apply.md) + [dc2-us-carrier-author-arm](evidence/2026-11-12-dc2-us-carrier-author-arm.md) + [dc2-strike-wrap-tag](evidence/2026-11-12-dc2-strike-wrap-tag.md) + [dc2-element-carrier-fold](evidence/2026-11-12-dc2-element-carrier-fold.md) + [dc2-decl-shell-partial-order](evidence/2026-11-12-dc2-decl-shell-partial-order.md) + [dc2-bold-value-face](evidence/2026-11-12-dc2-bold-value-face.md) + [dc2-bold-italic-carrier-merge](evidence/2026-11-13-dc2-bold-italic-carrier-merge.md) + [dc2-italic-toggleoff-face](evidence/2026-11-13-dc2-italic-toggleoff-face.md) + [dc2-italic-add-shell-collapsed-qs](evidence/2026-11-13-dc2-italic-add-shell-collapsed-qs.md) + [dc2-bold-toggleoff-universe-diag](evidence/2026-11-13-dc2-bold-toggleoff-universe-diag.md)（d10–d12：宇宙面定谳 + 桥层修复落地，bold 12F→8F 零回归）+ [dc2-sub-sup-va-shell](evidence/2026-11-13-dc2-sub-sup-va-shell.md)（d13：sub/sup va 冲突壳 + 语料红账重分类，F2P=20 零回归，六文件 57F→37F）+ [dc2-va-peel-interior-strike-shell](evidence/2026-11-13-dc2-va-peel-interior-strike-shell.md)（d14：strike S1 内部界 peel + sub/sup 残段保面 + s 壳收编槽，F2P=15 零回归，六文件 37F→22F）+ [dc2-strike-s2a-p5-retag](evidence/2026-11-13-dc2-strike-s2a-p5-retag.md)（d15：strike S2a 同 tag 名门 + P5 td span retag 端点重锚，F2P=2 零回归，六文件 22F→20F）+ [dc2-strike-direction-m1-absorb](evidence/2026-11-14-dc2-strike-direction-m1-absorb.md)（d16：strike REMOVE 方向规则 + u/s css-true 裸 span 吸收，F2P=2 零回归，六文件 20F→17F） |
| DC-3 全量对账 | 对账口径就绪（净增 +92,875，距线 37,125） | coverage-2026-11-11 快照 + 验收线 146,184 |

碰撞记录：本批 pin 钉测与并行流 D1 的 workspace 全量 test+clippy 共
享 build 目录锁（2026-11-13），改用独立 CARGO_TARGET_DIR 完成钉测，
未触碰对方进程；后续重型编译撞锁时同法分流或等待。
