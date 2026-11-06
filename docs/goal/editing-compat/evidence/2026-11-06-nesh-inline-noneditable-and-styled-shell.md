# 2026-11-06 noneditable 整行单 li + styled 并 li div 壳溶解 两刀——NE 主循环碎片化三重缺陷定谳 + ST 复用合并源块溶解

日期：2026-11-06。基线 = divb 刀后账面（iol 6F / iul 7F，`batch-divb-*-post.log`）。目标族 = 控制面「下一步①」：styled compare ×1（iol `:688`）+ noneditable compare ×2（iol `:768` / iul `:860`，双语料同形）。**终判 = 两面三刀全修，探针五案全绿。**

## 语料面核对

- NE 形：`'{ <span contenteditable="false">A</span> ; <span contenteditable="false">B</span> ; <span contenteditable="false">C</span> }'` 整行选区 → 期望（双备选）`<ul><li> <span>A</span> ; <span>B</span> ; <span>C</span> </li></ul>` / tight 变体。census：`contenteditable="false"` 行双语料**各恰此一行**（:860/:768）。
- ST 形：`'<div><ol style='font-family: times; color: blue'><li>bar</li></ol><span style='font-family: italic; color: green'>[baz]</span></div>'` → 期望 **assert_equals 单备选** `<ol style=…><li>bar</li><li><span>baz</span></li></ol>`（无 div）。兄弟行 `:678`（fresh 壳形）期望同样无 div 但**已绿**（走 retag 臂）；`:683` 守卫（baz 在宿主层）已绿。

## 隔离探针（zz-probe-ne-sh.html，wpt-data 本地 gitignored）

五案（三失败形 + 两守卫），修复前复跑逐字复现语料 got：

- A-ne：` <span>A</span><ul><li> ; </li></ul><span>B</span> ; <span>C</span> `（span A 前置裸露、唯中段 ` ; ` 成 li）
- B-ne-ol：同构 ol 形
- C-st：`<div><ol…><li>bar</li><li><span>baz</span></li></ol></div>`（并 li 正确、div 壳残留）
- D-st-ctl/E-plain 守卫恒绿。

## 机制一（NE :860/:768）：主循环逐 run wrap 碎片化三重缺陷

DBG 插桩（`NL/SUB/FS-FRESH/DPS/END` 五时点，判读后剥净）实锤链条：

1. **nodeList 滤除非可编辑**（vendored 语义）：NL = 4 个 text（`" "`/`" ; "`/`" ; "`/`" "`），三枚 span 全滤——主循环视角整行只剩孤离 text run。
2. **首 run allInvisible 返 null**：wrap 头部 `allInvisible` 检查对纯空白 run 返 null（vendored 语义），`" "` 不包装。
3. **ViaHtml 重建 → 文本成员幽灵重定位不可达**：`" ; "` wrap 走 flat 臂 → `host.innerHTML` 整段重建，残余 text 成员 detached；`_zwListRelocateWrapGhosts` 按 `outerHTML` + `nodeType === 1` 元素门匹配，**text 成员 `ancHtml` 恒 undefined 直跳** → iter3 `wrap([text3])` 检 `!nodeList[0].parentNode` 返 null → **sublist 空，`sublist[0]` TypeError 被调用点 try 吞、toggle 中止**——TRACE `SUB n=1 [LI]` 后 `SUB n=0 []` 实证单 li 烂尾。

Chrome 录音语义：扁平行整行（含 non-editable span + 空白 run）收进**单 li**（`foo[bar]baz` 整行单 call 已绿行同源语义——全 editable 行 ViaHtml 成员链连续故不碎，non-editable 割断兄弟链才暴露）。

**刀**（`part01e.js` `_zwListToggle` enable，bqHost 赋值后新臂）：NE 整行单 li 臂——nodeList 成员全为宿主直下（任一非直下即不触发）、covered span（成员首末索引间）全 inline 无块无 br、span 内 ≥1 non-editable 元素子（族门，census 双语料各恰一行）且 ≥1 可见子 → 新建 li 按 `_zwListMovePreservingRanges` 逐子收编 span 段全体 → `nodeList = [li]` 交回主循环（gather 直收 → freshShell wrap → ul → dps 出口全复用）。门不含块/br 形（eb/br 预遍域不重叠，`lineScope` 存在即跳）；全 editable 行（现存绿面零命中族门）零扰动。

## 机制二（ST :688）：复用合并后源块 div 壳残留

DBG 实锤：NL = [SPAN]（ol 不在 contained），inline 臂 fresh li 后 **freshShell wrap `reusePrev` 复用既有同层 ol**（`newParent = li.previousSibling`，criteria `isHtmlElement(ol, 'ol')` 命中）——li 并入 ol 正确，但 `freshShell` 恒 false → dps 归并出口跳过 → 源块 div（现仅剩 ol）无人溶解。

census 防扩面：`<div><ol/ul…</div>` 保壳期望行分三类——①双备选行（:775/:780/:786/:791 族，保壳/溶解二选一，溶解形本就合法）；②eb1 空体族单备选保壳（`<div>{}</div>`/`<div>{}<br></div>` → `<div><ol><li>abc</li></ol></div>` :701/:707/:751/:757）——fresh 壳**无既有列表可复用**，门 (a) 天然排除；③:688 本行 assert_equals 无 div。

**刀**（`part01e.js` 主循环 freshShell wrap 位点 + 循环后新臂）：wrap 前捕获 `stBlock`（sublist[0] 父块，非宿主才录）；循环后若 `newShell && !freshShell`（复用合并形）且 `newShell.parentNode === stBlock`、stBlock 为 div/p 非列表元素非宿主、块内仅剩 newShell + 空白 → `insertBefore` 提位 + 移除空块。:683 守卫合并落宿主层（stBlock=null）不触发。

### ST 臂选区保全刀（首轮门禁 P2F=2 定谳后追加）

首轮门禁（batch-nesh-legs：iol 1755P/6F **F2P=2 P2F=2 VERDICT REVIEW**、iul 2059P/6F F2P=4 P2F=0 OK、enginelib 八红恒一 OK、mt 腿因改码重门中途停批作废）：ST 臂溶解树正确（:688 innerHTML F2P）但 **:688 同测试 `queryCommandState/Value("insertorderedlist") after` 双行 P2F**（期望 `true`/`"true"`，got `false`/`""`）。机制：`insertBefore` 子树搬移在引擎选区重锚语义下把搬移子树内端点甩到壳外——探针实证 pre-fix `sel=#text"baz"<SPAN<LI<OL@0..3`、post-fix `sel=DIV@1..1`（塌宿主层），qState/qV 壳标读面按端点爬祖先均无列表 → false/`""`。pre-fix State/Value 恰绿全凭保壳树下端点恰在 ol 内——树修对反而暴露选区丢失。

**保全刀**：挪壳前按 newShell 相对索引路径快照两端点（`_zwListPathFrom`），拆块后显式回写（`_zwListResolvePath` + `setStart/setEnd`）；端点不全在壳内则不回写（不半新半旧混搭）。iul 零触发（ST 几何全语料唯 :688 一行）。保全后探针复跑五案恒等且 C-st `sel=#text"baz"<SPAN<LI<OL@0..3` 回位。

## 修复后探针（五案全绿，净桩复跑恒等）

- A-ne = `<ul><li> <span contenteditable="false">A</span> ; <span contenteditable="false">B</span> ; <span contenteditable="false">C</span> </li></ul>`（期望变体一逐字，选区 li 内文本）
- B-ne-ol = ol 同构逐字
- C-st = `<ol style="font-family: times; color: blue"><li>bar</li><li><span style="font-family: italic; color: green">baz</span></li></ol>`（:688 期望树逐字，div 溶解；**首版探针判读只核 html、漏核 sel=DIV@1——语料 State/Value after 断言面由首轮门禁 P2F 暴露**，保全刀后 sel=#text"baz"<SPAN<LI<OL@0..3）
- D-st-ctl = `<div>bar</div><ol><li>baz</li></ol>`（守卫不变）；E-plain = `<ul><li>foobarbaz</li></ul>`（守卫不变）

## 门禁（batch-nesh2-legs，systemd detached + flock + 内存守卫）

| 腿 | 结果 | judge（vs bq 基线） | VERDICT |
|---|---|---|---|
| iol | 1,757P/**4F** | F2P=2（:688 innerHTML + :768）/ P2F=0 / lost 0 | OK |
| iul | 2,059P/**6F** | F2P=4（divb 3 + NE :860）/ P2F=0 / lost 0 | OK |
| enginelib | 3,777P/8F | 八红名册恒一（五红 + ws×2/fetch×1 外部跨面债） | OK |
| mt | 19,331P/837F | rows 10092→10092，F2P/P2F/postF/lost 全 0 键级恒等 | OK |

**BATCH-VERDICT OK**（2026-11-06T23:42 收口）。残差全部既有族零新增：iol 4F = 同型嵌套 qState 读面 ×3（?1001-last）+ qVb 窗界伪影 ×1；iul 6F = 同型嵌套 qState 读面 ×2（?1-1000）+ 跨列表读面 ×2（?1001-2000 qInd/qState）+ divbr straddle 窗界伪影 ×2（?2001-last）。账面 **iul 7→6F / iol 6→4F**。

首轮 batch-nesh-legs（无保全刀）账目（对照存证）：iol 1755P/6F F2P=2 **P2F=2 VERDICT REVIEW**（:688 qState/qV after 双行，机制与保全刀见上节）、iul 2059P/6F OK、enginelib OK，mt 腿因改码重门中途停批作废。

基线 = `batch-bq-*-post.log`（bq 刀后账面；divb 刀 iul 10→7F 已含在基线对比时序内）。二进制 marker：`neHasNe=3 stBlock=13`。批脚本 `.acceptance/dps/batch-nesh2-legs.sh`，systemd detached（MemoryMax=24G + TimeoutStartSec=90min）+ flock 串行，23:10:20→23:42:06。

## 零扰动面说明

- NE 臂族门（span 内 non-editable 直子存在）对现存绿面零命中（census）；触发形走 `nodeList=[li]` 主循环常规道（直收/freshShell/dps/sr1 全既有臂）。
- ST 臂三重门（复用合并 + 源块仅存列表 + 非宿主非列表块）：eb1 fresh 壳、:678 retag 形、宿主层合并形均不触发；双备选行溶解形本就合法不构成 P2F。
- DBG 插桩（五时点）判读后剥净，`grep __zwDbgListSel` 恒零；净桩复跑五案结果与带桩恒等。
