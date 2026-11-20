# exec2/exec3 双门刀收口（2026-11-19）

DC-4 普查定谳的两枚真差异刀候选落地。刀体 `1383ac371`＝part06.js
execCommand 入口双门＋四派发臂递归深度标记＋part144 钉测六面（详述见
commit 正文）。

## 一、刀形与 Chrome 对账

| 键 | Chrome master（71b58bb1） | 刀前我方 | 刀形 |
|---|---|---|---|
| exec2：Recursive `Document.execCommand()` shouldn't be supported | 1/1 OK | 0/1（递归重入逐层插字 `0, 1, ..., 9`） | `_zwExecCmdDepth` 派发窗门：四派发臂（clipboard/text-control/CE-insert/editing）try 置 1、finally 同步清零，嵌套入口返 false 零事件零变更 |
| exec3：Testing execCommand with unknown command | 6/6 OK | 5/1（execCommand 表外返 true） | 未知命令入口即返 false（已知集＝inputType 表 ∪ clipboard ∪ dps/stylewithcss/usecss/selectall/undo/redo/enableObjectResizing）；五读面（qCE/qCI/qCS/qCSupported/qCV）刀前已绿零改动 |

钉测 part144（`test_exec_gate_pins`）六面：unknown×3（ret=false 不抛零
事件零 DOM）＋递归形（outer=true nested=[false] counter=1 text="0"）＋
深度清零控制键（outer2=true text2="1"）＋bold 已知命令控制键（ret=true
`<b>` 包裹派 1 input）。

## 二、门禁批 e23：目标腿全绿＋L8 捕获跨流回归

批链 e23（01:51 起跑，head `1383ac371`，bin `41ac09e8da3d33d99558c4cecab49d8c`）：

| 腿 | 结果 | 判定 |
|---|---|---|
| L1 exec2 | rc=0 **1/0** wall=1s | 目标键翻绿 ✓ |
| L2 exec3 | rc=0 **6/0** wall=1s | 目标键翻绿 ✓ |
| L3 fb | 5043/3/0 wall=840s | SET-IDENTICAL 对 r21a 锚 ✓ |
| L4 fs | 2419/3/0 wall=53s | SET-IDENTICAL ✓ |
| L5 event | 176/4/0 wall=2s | SET-IDENTICAL ✓ |
| L6 indent | 1329/0/0 wall=23s | 清零域维持 ✓ |
| L7 img | 962/0/0 wall=12s | 清零域维持 ✓ |
| L8 ip | **ABORT：7183/12**（期望 7185/10） | 新增 2 红＝R22 I4 双键 P2F，伪影 10 键恒一 |

L9 ilb / L10 lib / L11 mt 未达（L8 计数门 ABORT 截停）。

## 三、L8 ABORT 归因链（跨流 d55 误删 ⑩ 臂）

1. e23-ip failset 对账（12 键 vs ip1 伪影集 10 键）：新增 2 红恰为 R22
   I4 刀翻绿的 CE-island 双键（`<dl contenteditable=false><dd
   contenteditable>ab[]cd</dd></dl>`＋`<ol contenteditable=false><li
   contenteditable>ab[]cd</li></ol>` compare innerHTML），其余 10 键逐行恒一。
2. 嫌疑窗收敛：22:26 组合 spot check（树含 {R22+TD1+R2}）ip 7185/10 绿；
   led1 普查（4932e5d95，含 TP/RC）未跑 ip 腿 → 嫌疑⊆{TP, RC×2, d55, exec}。
3. 钉测判位：part143（`test_ip_island_li_dd_pins`）在 d55 树（part06.js
   处于上轮 429 遗留的 A/B revert 态，blob `82df69c5d` 恒等于 d55 树）
   **红**（p-li-i4/p-dd-i4 got `ab<br>cd` ③ 行断退化形）→ d55
   （`a763e74b1`，对端 wptfyi-editing 流 input-events 域刀，part01.js
   +226 行）在同文件 arm1 域**误删 ⑩ 臂扩域＋sepTag 直通化**（
   `sepTag = hostTag` 回退覆写＋`arm1IslandLid` 门整段缺失）＝regression
   源。exec 双门刀对 I4 无涉（L1/L2 目标腿＋L3-L7 全绿旁证）。
4. part143 在 HEAD 树亦红（d55 的 part01.js 缺 ⑩ 臂，与 part06.js 无关）
   ——即 e23 批即使无 exec 刀也会在 L8 同点 ABORT。

## 四、修复（对端流响应，跨流协作闭环）

wptfyi-editing 流读 e23 ABORT 账本后按原 I4 语义恢复 ⑩ 臂（part01.js
+26/-2，恢复注两处；arm1IslandLid 门＋sepTag 门与 `b86912191` 原刀**逐
字节恒等**，仅注释追加恢复注；d55 其余 BS/FD 域改动不动）。part143 钉测
修复树翻绿（双证：独立复跑＋resume 轮 bg 任务 bturyzk4b）。

修复提交：`1cbcb7420`（对端流域，已推 origin；提交正文含完整归因链与
e23 L8 捕获记录）。

## 五、e24 重锚批（EXPECTED_HEAD=1cbcb7420）判读

e24 六腿收口结构（03:39 起跑，bin `40014d7d0bb16b2fa906bd74f45df478`）＝
exec2/exec3 绿域重锚＋ip 腿（恢复 7185/10 伪影集恒一）＋ilb（同函数
lineBreakOnly 邻接面）＋lib strict（part143/144 钉测在册）＋mt 压轴
（10090/2 X37 恒一）；fb/fs/event/indent/img 五面 e23 已 SET-IDENTICAL
且修复域不相交（⑩ 臂 insertParagraph 域局部），不再重跑。

**e24 终值（05:49:23 ALL-DONE，六腿全绿——writer 轮 429 亡故后批 setsid
存活自跑完，收养轮判读）**：

| 腿 | 结果 | 判定 |
|---|---|---|
| L1 exec2 | rc=0 **1/0/0** wall=1s | 双门刀绿域重锚 ✓ |
| L2 exec3 | rc=0 **6/0/0** wall=1s | 双门刀绿域重锚 ✓ |
| L3 ip | rc=2 **7185/10/0** wall=953s，failset==ip1 伪影集逐行恒一 | ⑩ 臂恢复，I4 双键 F2P 复绿 ✓ |
| L4 ilb | rc=0 **1691/0/0** wall=33s | ✓ |
| L5 lib | rc=0 **4214P/0F** wall=17s，part143/144 双钉测在册 | ✓ |
| L6 mt | rc=2 **10090/2/0** wall=1633s，X37 SET-IDENTICAL | ✓ |

判读附加对账：批 head `1cbcb7420` 至判读树漂移仅 docs 三笔 .md（零代码
漂移，bin 锚仍绑现树）；e24-ip-failset 与重射 e23-ip-failset 字节恒一。
双门刀收口正式生效：exec2/exec3 真差异清零。

## 六、残面全图更新（post-双门刀）

| 面 | 计数 | 归类 |
|---|---|---|
| exec2/exec3 | 0 | **真差异清零**（双门刀翻绿） |
| fb 3 / fs 3 / event 4 / ip 10 / mt 2 | 不变 | artifact（协议伪影/窗 harness/同红挂账族，failset 全链恒一） |
| exec1 474 | 不变 | handoff（user 域决策门，user 留置） |
| ie 残 | 不变 | handoff（对端流域） |

DC-4 验收账本终态：自认领真红面**清零**（exec2/exec3 收口，I4 双键经
e23 捕获＋跨流修复维持绿）；伪影挂账面 failset 全链恒一。
