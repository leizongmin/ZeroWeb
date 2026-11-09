# F7m 六键四微机制刀（2026-11-09，⑦ 车道）

mt 残 15F（[delmerge 收口档](2026-11-09-f7-delmerge-knife.md) ⑦ 车道）首位六键：host-div-wrap 1（mt 483）+ ip-placeholder 4（485/486/487/488 双臂）+ trailing-empty-p 1（626），护栏 9 案（482 blockquote / 484 双形式 / 489-492 文本 caret 占位 br 现径 / 495/497 跨壳 IP / 627 wss A4 护栏）。

## 刀体（commit 298fbc8a6 + 边界守卫修正 a8fb68673，两 shim + part01f + fixture）

- **A1 host-div-wrap（part01h 消解卫星 + part01f 双调用点）**：`__zw_ce_dissolve_emptied_div_tail(ar0)`——deleteSelection 全删空宿主直子 div 时 Chrome 消解壳提宿主（子提宿主保序 + 块删除 + caret 经 `__zw_ce_caret_in_leading_shell` 提壳链首，483 typing `<b>a</b>` 落壳）。四门：tag 收窄 div（482 blockquote 全删空不消解反例）∧ 宿主级无其他元素子/可见内容 ∧ **ar0 前选区 containment**（删除前选区完全含于块内某内联壳子树——⑥ delmerge 边界祖先语义，见下节）∧ **display∈{'',block}**（flex/grid 保留，与 ① 臂 dispHost 门同语义）。调用点自限非 collapsed 面：delete_apply 尾钩（传 entry ar）+ ipara_apply 前导 delete 后钩（捕获 `_zwIpAr0` 传入）重派 `__zw_ce_paragraph_split_at` ① 宿主级臂（485/487）。
- **A2 ip-placeholder ①（part01 ① 臂扩门）**：`arm1EmptyShellEl` 空壳元素边界（`block===boundaryNode` 非块 tag 且无可见子）并入 ① gate 的 `(block===host||arm1EmptyShellEl)` + `(isText||boundaryNode===null||arm1EmptyShellEl)`；`refAfter1` 元素臂修正（boundaryNode 原位非 childNodes[so]）；头垫（sep1 前向兄弟链反向判空垫裸 `__zwPlaceholderBr`，`_zwJusIsVisible` 块恒真故不能用 some 谓词）+ caret 经壳链下钻首面（文本/占位 br 头面 sh1 null 原位零漂移）+ ipara_wants 让位门（元素位 caret 前方无可见内容让 false——typing 落 ① 臂）。485/487 typing `<b>a</b>` 入壳。
- **A3 ip-placeholder ②（part01f 垫底收窄）**：ipara 尾段垫底 br 前置 `_zwIpShell` 壳链判定（`__zw_ce_caret_in_leading_shell(newContainer)`），空壳形不误垫独立占位块（486/488 `<div>abc</div><div><b>d</b></div>` 中 typed 块尾无幸存 br）；处方 b 原重复计算移除复用。
- **A4 trailing-empty-p（part01h adjunct + part01f FD walk cond1/cond2）**：`_zwA4TrailWsInvisible(nd)` 行尾 ws 前向行界直判（ws 文本 + 前向文档序扫描遇块/br/块界爬升耗尽即折叠真，遇非 ws 文本/含可见子树 inline 即假；`_zwJusIsWhitespaceNode` 父链 white-space 上下文判定使 pre/break-spaces 面天然不入——627 护栏零改动）。并入 walk cond1（invisible 兄弟子）与 cond2（invisible 子）判定。626 全链：adjunct 吸收块内尾 ws + 块间 ws → walk 续进 → 既有 final-range 跨块合并径 → `<p>abc</p>` → typing `<p>abcd</p>`。

## 根因链

- **A4 双层分歧**（探针 side-band trace 定谳，[learning](../../../learnings/bugs/2026-11/2026-11-09-fd-walk-execution-path-trace-before-absorb-arm.md)）：spec visible 对 ws 文本走文档序**回看**（`_zwJusIsCollapsedWhitespaceNode` 前一路径遇非空白文本判行中可见），walk 先吃块尾 br 后 ws 回看命中块内正文误判行中 → walk cond4 下降进 ws → text 分支当字符单删即 return，跨块合并径不达；Chrome 按**前向**行界折叠。执行路径模型同样证伪：初版吸收臂落 walk 后 br-branch，trace 四变体全空——walk cond1 在 text 分支前已移除块尾 br，br-branch 对本形不可达，死臂撤除。
- **A1 边界祖先语义**（门禁批 v1 probe-f7 腿 2P/7F 归因）：初版消解条件缺选区几何门，把 ⑥ delmerge 边界祖先保留形（`<div style=display:flex><span>{abc</span><span>def</span>}</div>` 删除中间态 exp `<div …><span></span></div>` 族 + K3 `{abc}` div 子边界位 + K1/K2 尾越块界形）误消解（got `<span></span>`/`<b>gh</b>`）。Chrome 分界：删除前选区完全含于块内唯一内联壳子树（483 `[abc]` 文本位）→ 消解；选区触及块边界（子边界位）或尾越块界入宿主级 → 块为选区边界祖先，保留（exp `<div>gh</div>` 族）。修正 a8fb68673 补 ar0 containment + display 双门，ipara 调用点补前选区捕获。

## 探针迭代（zz-probe-f7m，faithful 权威面 + 直调 trace 不判）

r1 六目标全红 + 九护栏全绿基线（trace 观测垫底 br 出生点在 IP 步）→ r2 A1 helper 落点 part01f 行数违约迁 part01h（186 行卫星，④ 刀先例），146P/4F → r3 `_zwJusIsVisible` 块恒真打穿头垫/空源块两判据（改 previousSibling 反向判空 + 占位语义直判），148P/2F → r4 ws 折叠判据改兄弟链行界直判（回看谓词 deleteSelection 后时序失配）→ r5 A4TRACE 全空证伪 br-branch 假设 → r6 加 `da/fd pre/逐迭代/walk` 四点定位 trace 定谳执行路径 → A4 落点改 walk adjunct → 六案全绿 → 认证清理（四点 trace 撤除、DIAG/DIAG2 临时段移除、TRACEDUMP 转 trace-only；CASE483 直调选区寻址伪影 caret 溢宿主树落探针 script 文本，mt harness 同案绿，沿 F4 CASE530 先例 trace 不判）。终态 **149P/0F**（fixture md5 d79c5136 入库）。

## 门禁批（batch-f7m-gate-v1）

v1 首射（12:16，head 298fbc8a6）probe-f7 腿 2P/7F ABORT → A1 边界守卫修正 a8fb68673（九 probe fixture 本地全绿复验：f7 9/0、f7m 149/0、f4ilb 99/0、f1shell 27/0、f1b 7/0、f2 14/0、f3 7/0、f6b 17/0、k11 224/17）→ 重锚重发射（12:41，head a8fb68673）。

### 终态判读（ALL-DONE 14:10:08，head a8fb68673，pid 709622）

| 腿 | 结果 | 门 |
|---|---|---|
| probe-f7m | 149/0 | 六案+九护栏 |
| probe-f7 / f4ilb / f1shell / f1b / f2 / f3 / f6b | 9/0, 99/0, 27/0, 7/0, 14/0, 7/0, 17/0 | 刀间护栏全绿 |
| probe-k11 | 224/17 + judge F2P 0 P2F 0 lost 0 | 基线恒等零漂移 |
| ilb / ip / ihtml / exec×3 / iol / iul | 全 F2P 0 P2F 0 post-only 0 | vs ⑥/F4 终态零漂移 |
| mt | 20168P/16F（基线 20156P/28F）；行级 F2P 12 P2F 0 postF 0 lost 0；entry 级 F2P 恰 6 | 四微机制族翻绿 |
| mt 校准 | entry 翻键 12（白名单内：483=2 + 485/486=4 + 487/488=4 + 626=2）；白名单外 0 | 白名单外 0 |
| enginelib | 3820 passed / 7 failed | 7-red 名册 OK（6 先例 + r2967） |
| battery 七腿 | 420/880/1104/552/3456/2607/1455 全 F=0 T=0 | ⑥ 批同数 |
| binary sig | 9/9 命中（`__zw_ce_dissolve_emptied_div_tail` `_zwA4TrailWsInvisible` `arm1EmptyShellEl` `_zwIpShell` 等） | 产物内刀体身份 |

断言级 28F→16F（−12 = 六 entry 翻键），残 16F = 行级 9F：F6b handoff 4（N1/N2/N4/N5）+ F6a 2 + F1' fontsize(4) 1 + X37 2，与 ⑥ 收口档预测一致。ip/ilb F2P 0——A2/A3 域收敛无外溢。

## 工程记账

- part01f.js 2075 行（>2000 锚定）——A4 判据落 part01h 卫星规避新增违约；存量超限沿 ④ 刀记账，拆分留 ⑦ 车道收口后。
- 门禁批先入库协议首次触发实践：批脚本树不净门使刀体必须先 commit 再发射（首射 ABORT 即协议现象，非脚本缺陷）。
- 批日志 `.acceptance/dps/batch-f7m-gate-v1.log`（141 行）留档 gitignored 区。
