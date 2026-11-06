# 2026-11-06 div 边界 compare ×3 刀——blockExtend list-wide 块 start 边界排除臂 + ViaHtml 孪生定位位优先 + 幽灵重定位 detached 放行

日期：2026-11-06。基线 = bq 刀后 fail 面（iol 1755P/6F / iul 2055P/10F，`batch-bq-{iol,iul}-post.log`）。目标族 = 控制面「下一步③」首选项：iul div 边界 compare ×3（`:775` `<div>[foo</div>bar]<div>baz</div>` / `:803` `<div>[abc</div><div>def</div><div>}<br></div>` / `:809` `<div>[abc</div><div>def</div><div>]ghi</div>`，全 `?2001-last`，全仅 compare 断言红）。**终判 = 三行全真实引擎债，两机制三刀全修，探针六案全绿。**

## 语料面核对

- 三形在 insertorderedlist.js **缺席**（仅 iul 语料有）——无跨命令矛盾面。
- census：`]`/`}` 紧贴 block 开标签的几何在 iol/iul 两语料**仅 ：803/:809 两行**——端点排除规则的语料 blast radius 即目标两行。
- 兄弟行 `:772` `<div>[foo</div><div>bar]</div><div>baz</div>`（端点偏移>0）与 `:806`/`:800`（端点在文本末/起点侧对称位）**基线即绿**——判据对照面。

## 隔离探针（zz-probe-divb.html，wpt-data 本地 gitignored）

六案（三失败形 + 三 PASS 兄弟形对照），修复前复跑逐字复现语料 got：

- A-775：`<li>foo</li><li>bar</li><div>baz</div>`（ul 壳灭失）+ sel 塌缩 `host@3`
- C-803：`<ul><li>abc</li><li>def</li><li><br></li></ul>`（br 块过吞）
- D-809：`<ul><li>abc</li><li>def</li><li>ghi</li></ul>`（ghi 块过吞）
- B/E/F 守卫恒绿。

## 机制一（:803/:809）：blockExtend list-wide 端点块过吞

原始 end 落在 block start 边界（offset0 沿首子链爬升到最近 block 仍在 offset0）时，既有 do-while + at-length 爬升把该 block **整体纳入含域**（newRange end 爬到 block 后），`_zwJusGetContainedNodes` 收进端点块 → 换型吞并。语料明注 *"don't listify the last empty <div> if only its start boundary is in the range"*——Chrome 录音排除端点块。

**刀**（`part01c.js` `_zwJusBlockExtend` list-wide 臂）：改写前捕获 `origEndN/origEndO`；list-wide 分支先判几何——offset0 爬升到最近 block 且该 block 无列表祖先（li 臂整行收编语义域不触发）、非宿主、非起始块（`exB !== sbX`，eb1 空体形 `<div>{}</div>` start=end 同块不触发）、非 td/th → `end = (exB.parentNode, idx(exB))` 钳到 block 前。对照面：`:806` end=offset3 不触发、`:800` start 侧对称位不触发（规则仅 end 侧）。

## 机制二（:775）：ViaHtml 重建后 sublist 幽灵 + 孪生定位歧义（两刀）

DBG 插桩六轮定位（`BH/RG/RGD/RGMISS/WRAP/SHELL` 时点，判读后剥净）全链：

1. 主循环 gather：div1 retag→li1 入 sublist；紧随 `bar`（host 直下裸 text）走 inline-wrap 臂 → `_zwJusWrap` flat 边界臂（`part01c.js:1008`）→ `_zwJusWrapViaHtml` **host.innerHTML 整段重写**——li1 原引用 detached，且 np=li 本身无 ul。
2. 幽灵重定位对 sublist 从未跑：climb 循环条件 `anc.parentNode !== host` 对 detached（null ≠ host）成立 → `anc.parentNode.firstChild` **TypeError 被调用点 try 吞**，重定位整体中止（sublist 侧根本没接）。
3. 接上重定位后仍失败一层：ViaHtml 返回值孪生定位按 **open-tag 串首命中**——`<li>` 前缀歧义面（重建子列 [li-foo, li-bar, div]）把 liEl 定位到 liFooTwin 而非真包装 liBarTwin；重锚后 sublist = [liFooTwin, liFooTwin] 重复引用同一节点，套壳只装一次，liBarTwin 永远 bare。DBG 铁证：`RGD m=LI det=true path=0 ancHtml="<li>foo</li>"` → `SHELL=UL/fresh html=<ul><li>foo</li></ul><li>bar</li>…`。

**刀**（三处）：
- `part01e.js` `_zwListRelocateWrapGhosts`：入口 `!m.parentNode` 跳过删（detached 放行）；climb 循环加 `anc.parentNode &&` 真值守卫；新增 detached 分支（顶层幽灵候选，路径空，outerHTML 匹配重锚——路径非空时早停层即最深存活祖先，孪生下行同样成立）。
- `part01e.js` `_zwListToggle` 主循环 inline-wrap 臂后：`_zwListRelocateWrapGhosts(bqHost, sublist)` 重锚本迭代套壳成员。
- `part01c.js` `_zwJusWrapViaHtml` 孪生定位**位优先**：wrapper 恰占重建子列 `firstIdx` 位（prefix 前缀 = 成员首索引，html 拼装序即 DOM 序），位不可靠再串扫描兜底。同 open-tag 前缀歧义面串首命中恒取错——位优先对现存绿面零扰动（歧义几何 census 零命中，唯一命中即红着的 :775）。

## 修复后探针（六案全绿）

- A-775 = `<ul><li>foo</li><li>bar</li></ul><div>baz</div>`（期望第二备选逐字）；B 守卫不变
- C-803 = `<ul><li>abc</li><li>def</li></ul><div><br></div>`；D-809 = `<ul><li>abc</li><li>def</li></ul><div>ghi</div>`（期望第二备选逐字）；sel 保跨界 `LI@0..DIV@0`（qIndeterm 读面不变）；E/F 守卫不变。

## 门禁（batch-divb-legs，systemd detached + flock + 内存守卫）

批跑于 b79506697（修复在树），release 二进制双标记核对 `origEndN=2 _eRgS=1`：

| 腿 | 基线 | 批后 | 行级判读 | VERDICT |
|---|---|---|---|---|
| iol | 1755P/6F | 1755P/6F | F2P 0 / P2F 0 / post-only 0 / base-only 0 | OK |
| iul | 2055P/10F | 2058P/7F | **F2P 3 / P2F 0** / post-only 0 / base-only 0 | OK |
| mt | 19,314P/833F | 19,314P/833F | 四向全零 | OK |
| enginelib | 3,773P/5F 五红名册 | 3,774P/7F | 五红恒一 + ws×2 新红 | 见下裁定 |

iul F2P=3 恰三目标（`:775`/`:803`/`:809`），残余 7F 全为既有挂账族（qState/qV 读面 ×4 + noneditable ×1 + dps div ×2），零 P2F 零新红。

### enginelib ws×2 裁定（非本刀回归，跨面债）

批判读 enginelib 出现 ROSTER DRIFT（7F vs 五红名册 5F）→ 三层定谳：

1. **+3 测试数非异象**：全量测试名 diff，新增恰 `tests::pipeline::test_s311_textarea_*` ×3 = commit fa4696ebc（bq 基线 enginelib mtime 19:57 早于其落地 20:28）。算术闭合：3773 + 3 新绿 − 2 翻红 = 3774P。
2. **ws×2 翻红非本刀因果**：本刀只改 part01c/part01e 列表路径，零 ws 因果面；stash 实验（裸 HEAD 无本刀改动）同 7F 复现。
3. **元凶钉死 = ba03fd7ff（net-api M3 ws interfaces）**：bracket 归因实验（同脚本逐个 checkout + ws 过滤跑，test-guard 守护）——`727b7e163`（ba03fd7ff^）ws 6/6 **全绿** → `ba03fd7ff` 恰 `test_t8k_websocket_error_nonterminal`（got `msg:after-error`，want `error:1|msg:after-error`，error 事件未派发）+ `test_t8k_websocket_event_flow_open_message_close`（got `no` want `yes`）**同两测试翻红**。该 commit 211 行改 `part06.js`（CloseEvent 槽 + fail connection error 派发语义），其自称「+45P 零回归」审计面为 WPT harness（13,064→13,109P），不含 rust `js_dom_bridge` 桥测试——跨面盲区引入 main 常驻红。

**处置**：不属本工作面（net-api M3 另一席位在途），不动 `part06.js`；记控制面挂账并 lark 通报，归 net-api 面收口。本刀 enginelib 判据 = 五红名册恒一 + ws×2 已归因外部债，批 VERDICT NOT-OK 由此单点解除。

### rebase 后 tip 复核

rebase 97edcd983（远端 6 commit：fetch redirect f021e11fb / s312 textarea 4fbbce827 / R5436 layout / perf CI / docs ×2）后 tip 复核批（同守护，二进制标记 `origEndN=2 _eRgS=1`）：

| 腿 | 批判读（b79506697） | tip 复核（97edcd983） | 判读 |
|---|---|---|---|
| iol | 1755P/6F 四向零漂移 OK | 1755P/6F 四向零漂移 | OK 恒一 |
| iul | 2058P/7F F2P=3/P2F=0 OK | 2058P/7F F2P=3/P2F=0 | OK 恒一 |
| enginelib | 3,774P/7F | 3,775P/8F | 见下 |

tip 测试数 +2 = `tests::pipeline::test_s312_textarea_*`（4fbbce827，全绿）；新增第 8 红 = `js_dom_bridge::tests::test_fetch_filtered_response_type_matrix`（part10.rs:2605，response type 矩阵读面 got `undefined`）。

**fetch×1 bracket 归因**：`244e9a5c2`（f021e11fb^）该测试**绿** → `f021e11fb`（M3 fetch redirect 七簇）恰同测试**红**逐字同形——元凶钉死 = f021e11fb，与 ba03fd7ff 同一跨面盲区模式（「+102P 零回归」审计面为 WPT harness，rust `js_dom_bridge` 桥测试漏审）。同记跨面债，归 net-api 面收口。

**enginelib 终判据**：五红名册恒一 + ws×2/fetch×1 三红全数 bracket 归因外部跨面债（ba03fd7ff + f021e11fb），零本刀因果；语料三腿（iol/iul/mt）+ tip 复核全 OK。

## 零扰动面说明

- 机制一规则仅 end 侧 + 非列表块 + 非同块：iol 语料零命中该几何（census），iol 腿行级判读四向漂移全零实证。
- 机制二重定位调用 gated `bqHost`；活引用走 live-check 直过（零扰动）；detached 放行仅改变「此前必然 TypeError 中止」的死路径。ViaHtml 位优先：现存绿面歧义几何零命中（`foo[bar]baz` 族为整行单 call 单成员，无 prefix 歧义面；多轮 iter2/iter3 场景不存在）。
- DBG 插桩（六时点）判读后剥净，`grep __zwDbgListSel` 恒零；探针页 `__zwDbgListSel` 开关对剥桩后引擎 inert。
