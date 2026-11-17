# Editing、Selection 与 Input Events控制面

**2026-11-18 · Active**。**d53 列表项选删归一＋占位 br 换代幽灵滤除刀（dil 四 variant 净 +72）**：part01.js +83/−5——①`_sdLiKid` 列表容器元素锚单 li 跨度归一（sc===ec ul/ol 跨恰一 li：有后继摘 li 壳、无后继保壳＋占位 br，gtr 随动作形）②BS/FD 文内臂塌缩门（重读覆盖限 `range.collapsed`，非塌缩 `[list-item1]` 全文形不再删坍成 1 单元）③文内臂整节点摘除占位（清空所在块 → `<ul><li><br></li></ul>` 期望形）④`_zwCePlaceholderBr` 换代记账门（`__zwPbrGenByHandle`，handle 持久而 expando 被 set trap 丢弃——d53b 序贯毒化定谳）+ part05 +19（融合视图并桶点跨代占位 br 条目滤除，dil B 族 6 面 P2F 清零）。**A/B（base=old-base 9404d6d41）**：dil 四 variant 232P→304P **F2P=72/P2F=0**，backspace/forwarddelete/non-collapsed/joining-dl 共键全恒等；input-events 全套件（1500s 完整轮）F2P=108/P2F=0，对 HEAD 净树（e4fb2c42e suite）共键 6475 恒等 F2P=199/P2F=0；selection 全语料 53,317 键 F2P=6/P2F=0；editing/other F2P=1/P2F=0；editing/run 全量四分片共键 99,474 F2P=0/P2F=0（new/gone 全为分片尾截断伪影）；enginelib 4137P/0F。**③ 域持位**：LI 幽灵跨代并入在册域不动（本刀只滤占位 br 记账条目）。证据见
[evidence/2026-11-17-d53-listitem-selectdelete-and-placeholder-ghost.md](evidence/2026-11-17-d53-listitem-selectdelete-and-placeholder-ghost.md)。

**d52 join 缝空白序言刀（d51 停车场塌方收口，input-events 核心 +9）**：JoinMutate 提取序言——跨容器块界 join 的源行首文本节点**前导折叠空白在提取前摘除**（守卫 `srcParent !== dst` + 双侧 normal 域 + 落位块元素；先摘后提，C-strip 头部支自然 no-op）。**根因（zz-probe-d52a 形状隔离 + J0-J5 shim trace 一轮，backup 零残回退）**：塌方唯一判别面 = 提取源行首文本节点带前导折叠空白（T5 `<li>def[]</li>` + `  ghi` 单独复现 content `<div>ghi</div>`）；shim 侧全程正确（takeLine n=1、shellDst=LI、ser sel=li beforeend），塌方在**引擎侧 drain**——removeChild(前导空白文本子) + insertAdjacent wire 序被引擎解释为前序内容整体摘除；去前导空白后同 wire 序恒正确。**A/B（base=61ebaf369）：backspace 79→81P（caret-after-ul ws 形 content ×2，bi=2 双派 promise 键持位 ③）、forwarddelete 98→100P（def[] ws 形 content ×2）、non-collapsed 101→106P（列表选区面 ×3 + `<p>abc   [</p>]   def` 块界选区 ×2）、joining-dl 1174 共键恒等，全腿 P2F=0**。哨兵：enginelib 4134P/0F；selection CE 414 全同；suite 对 d51 净差 F2P=20/P2F=0（核心 +9 + 探针镜像自增 11）。探针九面 content 全契约形（T1 bi=0 首位鬼持位 ③）。证据见
[evidence/2026-11-17-d52-join-seam-ws-prelude.md](evidence/2026-11-17-d52-join-seam-ws-prelude.md)。

**d51 列表壳溶解收口 + join 界规范化刀（input-events 核心净 +6）**：part01.js 三处——FD 溶解闭包 takeLineFrom 重写（直挂 insertBefore 跨容器移动 shim 内抛 HierarchyRequest＝BS 臂 T7/T8 探针实证的 li 连行带走根因，改引擎 takeLine 提取后逆序挂回）+ B2 双臂 join 界终文本锚前向规范化（`_gtrEndF1` 非 collapsed 跨容器 / `_gtrEndF` collapsed 块界：JoinEndOf 裸 (leaf,0) 对后邻前导折叠空白欠账，元素锚不变 preserve/preLine 自守恒）。**A/B（base=45051c160）：backspace 主文件 78→79P（F2P=1 ws 面 innerHTML）；forwarddelete 主文件 96→98P（F2P=2 `def[]` ws 形 promise 键）；non-collapsed 对 d50 wip 98→101P（F2P=3 ?Delete 列表选区 content——FD 溶解臂在保守 join bail 前截获，`<div>abcdefghi</div>`/二 li 保留形全对）；全腿 P2F=0**。探针 zz-probe-d51a：T8 列表界 repeat 位语料契约全对（takeLine 路径合流派发下兑现）；T7/T9 首位鬼（T9 entry `(t"",3)` 陈旧代际）归 ③。列表面残余分型入册：plain 形 bi=0 鬼 + ws 形 bi=2 双派簇（d40-II 族，溶解臂使新面进入停场簇、无新增红键）+ FD ws 形块界 join 内容塌方（content `<div>ghi</div>` base 继承逐字同消息、共享 JoinApply 96P 高危挂停车场）。哨兵：enginelib 4131P/0F；selection CE 过滤 414 共键全同 F2P=0/P2F=0；input-events 全套件对 d50 wip 净差 F2P=12/P2F=0（核心 +6 + zz-probe 镜像面自增 6）。证据见
[evidence/2026-11-17-d51-list-dissolve-gtr-canonical.md](evidence/2026-11-17-d51-list-dissolve-gtr-canonical.md)。

**d50 select-boundary 中段原子/空白带刀落树（non-collapsed 双变体净 +10）**：part01.js +82/−2——④ 空界选区臂「run 中段」保守 bail（BS 13034/FD 镜像 13330）替换为带消费：sc/ec 同容器文本、带内限非块元素+折叠空白文本（normal 域）、摘带+缝规范化（gtr 起跳尾随/终跳前导折叠空白，`(abc,3)-(def,1)` 契约形）+双端原位截断 live 端点。**A/B：non-collapsed Backspace 29→34P、Delete 29→34P（F2P=10/P2F=0，共键 207 全同）；backspace/fd 主文件与 joining-dl 共键逐键恒等零翻面**。F2P=img 形全键 ×2 变体 + hr 三形 content 子测 ×2；**hr 形主键残余 bi=0 = 派发鬼（content 变异正确、事件不达页监听）——探针定谳 hr 形重锚产物陈旧代际（`(t"",3)` 鬼锚）、img 形引擎视图全活，同臂同路径差异仅在宿主代际解析，归 ③**；元素锚 `(div,1)-(div,2)` 形 sc-out-of-host 维持停场；TypingA 变体走 insertText 路径不入 delete 臂恒等。哨兵：enginelib 4125P/0F；selection 全套件（跑完）CE 过滤共键 414 恒等 F2P=0/P2F=0；input-events 全套件共键 5442 P2F=0；hs 可达性论证（语料 6 文件含 CE 无键删路径）+ CE 切片腿跑通无 crash。证据见
[evidence/2026-11-17-d50-select-boundary-midrun-band-knife.md](evidence/2026-11-17-d50-select-boundary-midrun-band-knife.md)。

**d49 可见单元删除核落树（BS/FD 塌缩界单元臂 + 缝规范化，input-events 双腿净 +75）**：part01.js +763/−39——①BS/FD walk 臂可见单元臂（img/hr 原子消费、行中 br
消费+次行前导折叠空白吸收、行尾 br 先探后并、inline 壳端点叶消费+空壳剪枝、
宿主缘 N 派发）+ 元素锚臂（img 后退消费、可视空块整摘、表格单元/caption 缘
no-op、FD 可视块行尾 br 消费+块界 join）②walk 臂 ws 头/尾进入门（`   []def`/
`abc[]   ` 形 normal 域）③E 族 pre-line \n 组（BS 末组摘除 + 派发后重读镜像）
④B 族壳内整节点 gtr 升界+空壳剪枝 ⑤C 族 join 缝规范化（C-strip：跨容器块
join 双侧 normal 域摘 dst 深末叶尾随/源行首前导折叠空白，FD/BS 共用，jdwsp
4,312P 反证面 preserve/preLine 守卫挡）。**双腿专项 A/B：backspace 44→78P
（+34）、forwarddelete 55→96P（+41），F2P=75/P2F=0**；joining-dl 共键零翻面
（套件级 1398→1518P 含 zz-probe 探针自增与并行截断噪声不作账——ledger 级
A/B 定为专项单文件双腿口径）。**同会话三轮闭环修复**：J 族块元素前邻
catch-all 吞块 join（块形归块 join 臂，zz-probe-d49s 隔离实证 base 吸收/
修前全静/修后恒等）、ce=false 子树守卫 `_zwCeInCeFalseSubtree`（BS/FD 单元
臂不得吃不可编辑壳，FD 语料 span 形 P2F 实证）、尾随 br picker 回退取首枚
（**语料 p/li gtr 起点契约分歧** (p1,2)/(li1,1) 单 picker 不可两全，last-br
改动净亏损回退）。**哨兵全绿**：enginelib 4099P/0F；html-semantics 共键
F2P=0/P2F=0；selection-CE 过滤腿 214P 恒等（全 selection 套件 >520s 守卫截，
CE 键改动不过 Selection API 不追）。**停车场**：①文件内跨测引擎池 rebake 毒
（zz-probe-d49r：FD elem-join 后 BS ws-seam gtr 混陈旧锚 + post 树复现旧池
p#2 + 宿主属性全 null → _stampWs 假设证伪，毒在引擎侧 innerHTML 换代 rebake；
语料 BS 文件无 FD key 零命中，归 master ③ generation-aware 归一刀）②pre 双
br p 形 (p1,2) 契约未达（base 同红）③N3/I1/C5 既有红维持。证据见
[evidence/2026-11-17-d49-visible-unit-knife.md](evidence/2026-11-17-d49-visible-unit-knife.md)。

**d48 join 壳吸收深化 A+B1+B2 落树（B3 判不落，
input-events 净 +223）**：part01.js +130/−5——①A `_zwCeJoinDstDescend`
（11782 区）块界 join 接收容器两级下降（dst 为列表容器降末行块 li/dt/dd +
行块唯一 p/h1-6/div 内层壳且壳内非空活内容再降；`_zwCeJoinApply` 4 调用点
统一过降；纯空白文本不算内容守卫，jdwsp 反证面不受影响）②B1 FD 块界臂行尾
br 透明消费 + **先探后删**（消费前只读探针镜像 re-resolve 确认链上有可并后
邻，末块行尾 br 不消费——A/B 首轮 P2F=1 `Delete at "<p>abc[]<br></p>"` 期望
保持 br 占位、got `<p>abc</p>` 即此形，补丁后 fd 探针 P2F=0）③B2
select-delete 左空摘除判空改「无可见内容」+ 摘空文本子 + 左向上剪枝
（lca/el 止，g<16）。input-events 全语料 1114P/2176F/4T → **1364P/1964F/3T，
F2P=223/P2F=0**：joining-dl-elements Backspace +66/Delete +36 与
joining-dl-and-another-list +20/+20 = A；dil Backspace ol/ul +31/+31 = B2；
dil Delete ol/ul +8/+8 = B1；forwarddelete +2/backspace +1 外溢。P 差
+250 = 翻面 223 + ?Backspace 案基线 case 级超时截断消失的尾段新入账 27（T 4→3，
尾段键不属翻面不计对账）。哨兵：html-semantics **P 恒等 5378**（F2P=0/
P2F=0，键差 0，1 键 T→F 瞬态）；selection **49,006P/4,298F/9T 逐键恒等零翻
面**（F2P=0/P2F=0 双链同窗口；基线侧较 d47 账 48,942P/4,312F 漂 +64P/−14F
系语料/runner 换代窗口，非本刀）。**B3 同块全选
splice 占位 br 判不落**：+8F/−2P 净 +6 但 pair 页 `{}`-form innerHTML 两面
翻红（隔离页干净、页侧 pre-run 选区/DOM 断言全过 → 变异在 shim 内段），
切片二分定谳 B3 元凶；replaceChild 与 removeChild+appendChild 两浅修变体
均证伪。机理 = innerHTML 重置 ghost 投影（part01.js:12298 遗留）：同页序贯
[]-form 遗留 detached 节点投影中 `[br]` 元素毒化 `{}`-form splice 占位
（遗留 `[text('')]` 不毒化），解域并入 master ③ generation-aware（与 d47
别名族同域），遗留 8 键 `[list-item1]` 全选形待该刀。证据见
[d48 join-absorption-ab](evidence/2026-11-17-d48-join-absorption-ab-slices.md)。

**d47 查询返回点身份别名刀判不落（selection 哨兵
否决）**：上 session 429 中断遗留的 selection 腿补测定谳——querySelector
返回点方言归并别名（宿主 `__zw_same_node_sel` 同视图 NodeId 同节点门 +
part05 别名表 512 上限 + part04 qSA 容器作用域门/part06 qS 返回点门，4 文件
+95/−2，**未上树**）input-events 全语料 1135P→1466P 净 +331（bi=2 dil 双派
族 +339 翻绿，二次复跑逐键复现），但 selection 全语料 **48,942P → 43,386P
净 −5,556（F2P=6/P2F=5,562）**：join white-space 三族 −5,449 + 
join-pre-and-other-block 净 −64 + dwuf/plaintext-only insertLineBreak 各
−24。机理与同刀收敛实验（impl 入口统一 el 身份，net −386 已回退）同解域：
innerHTML 换代后旧方言仍解析出同位置节点 → 同节点门放行旧查询视图代理 →
join 族 innerHTML 读旧投影大面积红；dil 族要身份连续、join 族要换代新投影，
两族要求相反，查询返回点一刀同判负——master ③ 入口级归一刀两处负样本
（d42 shim 方言归一 −27、d47 别名 −5,556），正确解域收敛到查询视图代际
分桶。WIP 四文件 stash（配方保留），码态回 d46 终态；`/tmp/d46-tgt` 二进制
已污染（现含别名刀），A/B 基线二进制须重建勿复用。证据见
[d47 query-identity-alias](evidence/2026-11-17-d47-query-identity-alias.md)。

**d46 B4 键入替换臂：非塌缩选区键入先删后插 + gtr
端点直取空白门**：part01.js +115/−9——①`__zw_ce_insert` 替换拦截（非塌缩 +
非 `\n` 键入改道 d43 `_zwCeSelectDelete` 摘除核新增 typingData 第 4 参，
摘除+join+插入单 beforeinput 承载）+ ②`_sdFinish` 键入尾（删除点插 data +
caret 落插入段尾 + input 收束；元素锚两侧**非空活文本**拼接——空文本 ghost
锚定谳：initializeTest innerHTML 重置留 shim/engine 分裂投影，parentNode 检查
不滤，拼接 wire drain 静默丢弃，probe console 通道 trace 实证；join 臂
`silent` 参让位 caret/input，返回 `{node, offset}` join 点）+ ③`_zwCeCanonGtrStart/
End` 直取臂（元素锚 child 非空白文本 → (text,0)/(text,len)）+ **纯空白文本子
除外**（`div{  <ol>…  }` 装饰空白保持元素锚）。input-events 全语料
（zz-probe 排除）1099P/2229F/3T → **1135P/2193F/3T 净 +36，F2P=36/P2F=0**：
non-collapsed?TypingA 69F→43F（+26，`{abc}` 系/跨 p join/caret host/img/
`{<img>}<br>`/`{ abc }`/li 系 gtr+content 双面）、non-collapsed?Backspace/
Delete +3/+3、deleting-in-list-items ×4 +1×4（`<li>{list-item1}</li>` gtr 直
取）。A/B 判据定谳：直取门=子文本纯空白 `/^\s*$/` 而非选区形状（v5
singleChild 限制 1098P P2F=37 joining-dl 回红弃——跨元素直取需求实证，与 d44
克隆/持活两族并存同构）；v4 全语料 1131P P2F=4 空白装饰形回红同证。
哨兵：html-semantics **P 恒等 5378**（F2P=0/P2F=0）；selection **48,914P →
48,942P 净 +27，F2P=32/P2F=4**（dwuf Typing 面 +24 首次成批翻绿——master 原
「dwuf +22 名义红」清偿 Typing 变体；typing-around-link Direct typing +4、
insertHTML ws 四域 +4；P2F=4 = Backspace/Delete/execCommand 两阶段变体
键入 XY 丢失——phase-1/2 代码与 d45 逐字节同、Direct typing 改核后同文档
残留态分岔，initializeTest 分裂投影 chaos 族与 img 三案同根，d42 A/B 前例
接受记档不追）。**dwuf caret/deleteCommand 臂 +22 名义红未清偿**（Typing
变体已翻绿，其余七臂两侧全红持位）。TypingA 残余
43F = bi=2 身份双派族（sameObj=true，d41/d42 已记档前置病非本刀引入）+
`{<hr>}` 纯元素选区核拒收 + table 跨 cell bi=3。证据见
[d46 typing-replace-arm](evidence/2026-11-16-d46-typing-replace-arm.md)。

**卡点** 无阻塞。口径持位：11-12 快照三目录合计 130,245、距 146,184 差
15,939；d53 后预估 ~144.0k（跨口径不直加，正式落账待 DC-3 快照统一口径）。
环境记档：main 有一处 fmt 既有漂移（`crates/dom/src/tests/
tests_9_query_coverage.rs`，并行流 3eddbadbb 带入，非本流写面，未触碰）；
runner console 通道仅 `ZW_SHAPED_ADVANCE_TRACE=1` 下可见（tracing subscriber
条件 init，console.log→stdout，d46 记档）。

**下一步** ①**B4-S2 第三刀候选（d48 后残余簇，见 d48/d46 证据遗留节）**：
(a) ~~join 壳吸收深化~~ **已由 d48 收**（A+B1+B2 落树净 +223；残留 = B1 两
p 形 `Delete at "<p>abc[]<br></p><p>def</p>"` F→F 未解 + B3 判不落移③）；
(a') ~~dil 列表项选删归一＋占位 br 换代幽灵~~ **已由 d53 收**（净 +72；
LI 幽灵跨代并入仍挂③）；
(b) backspace/fd 主文件 243F：跨元素边界 caret
join（`<p>a<span>b</span>[]c</p>` bi=0）+ 文首空删 beforeinput 形；(c) TypingA
残余 43F 中可独立修面：`{<hr>}` 纯元素选区核拒收（bi=0，核放行臂缺口）——
bi=2 双派族挂③不动（d47 别名刀判负后无入口级现成臂）；②dwuf 余臂（caret
join/deleteCommand 七变体）跨
child-block gtr 端点规范化持位——Typing 变体 24F 已由 d46 清偿；
③generation-aware 归一刀重设计（d42 A/B 与 d47 查询返回点别名两处入口级
判负后挂起：端点报告语义与身份稳定成对修、查询视图代际分桶是仅存解域；
d46 非空锚守卫是键入面局部免疫；**d48 B3 同块占位 br +8/−2 与 8 键
`[list-item1]` 全选形并入此域；d50 ④ 臂 hr 形派发鬼（content 正确 bi=0、
重锚 `(t"",3)` 陈旧代际）并入此域；d51 列表面派发分型（plain bi=0 鬼、
ws 形 bi=2 双派入簇、T7/T9 首位鬼探针）并入此域；**d53 占位 br 换代记账
滤除已落树（br 条目子域清偿、LI 幽灵与重锚陈旧代际仍在册）**）④B3 text-control execCommand 期望
矩阵（~474F，修前逐命令核对语料期望表）→ B5 小簇；⑤DC-3 周跑快照同口径
对账（净增 ≥130,000 + 邻域回退核对；face 哨兵探针清单见 d39 前账）。

**残余账本（持位）**：子文档域——tal CE（主窗）变体 collapsed ~122F +
non-collapsed ~77F + after-joining-paragraphs 48F（内壳形 `child=b`/
`parent=b&child=i` ×4 Timeout，前置=单一补偿面 + caret 引擎同位快照）+
delete-in-child-of-html 4F（d17 XOR 伪失败族出队）/insertparagraph-in-
child-of-html 8F（跨文档收养，引擎级 adoptNode FIXME(d33)）+ unlink 余样式
重裹 4F（FIXME(d24)）；modify 残余 ~57F（定向 probe 口径）= 鼠标驱动
modify、line-flex/grid（FIXME(visual-modify) 行盒几何）、inline-boundary
18/36、bidi character 22/28×2（RTL 视觉轴近似）、word 细分——按簇评估；
input-events 桶 = B4-S1 select-delete 残余形（d43 主臂落树后名义账 2613F
→ 剩左块尾随 `<br>` 消费/嵌套深锚/dwuf gtr 期望 +22 等，见 d43 证据遗留节；
d50 后 non-collapsed 三 hr 形主键 bi=0 派发鬼挂 ③ + TypingA 带形归 insert
路径另开面，见 d50 证据；d51 列表面派发分型挂 ③ + FD ws 形块界 join
内容塌方（**d52 缝空白序言刀已收口：塌方根因 = 提取 wire 对前导空白文本
子的引擎侧解释，见 d52 证据**；promise 键已 d51 翻 P、content 键 d52 清偿，
caret-after-ul bi=2 双派 promise 键持位 ③）
+ 修饰组合词删
（deleteWordBackward/Forward）+ range-exceptions 构造器 targetRanges dict +
IndexSizeError（4F）+ typing DELETE join-walk 分离 sc 崩点守卫（4 笔
unhandled rejection）+ shadow-dom getComposedRanges 62F——修复队列 B1–B5
见 [dc2-residual-bucket-triage](evidence/2026-11-14-dc2-residual-bucket-triage.md)；
六 format 文件真失败 6F（strike S2e qS-after 1F Chrome-quirk 挂账 + bold
b-in-i / italic i-in-b 块边界 2+2F + bold notbold 1F 待 bisect
20521a2ba/8add42713 一带）+ 伪失败 13F 出队；哨兵 fncorpus 两窗页级
Timeout（d15 已有）、f6bwin 发射深度=共享超时预算非门面（d22 bisect
定谳）；桥层支线三 bug 未动（主链经 R263 补偿与 B2 臂解耦）。

**d39 B3 诊断轮（零净码变）**：beforeinput 计数 908F 重分类定谳——**主簇 =
元素锚 range 选区 silent return**（list-items 592 + dl-join 152 +
non-collapsed 76 ≈ 820F，`__zw_ce_delete_impl` sc 非 text 直接 return、树零
变，select-delete 语义缺失），**事件丢失集群占比极小**（此前 round-3 日志的
主簇判断是误诊）。真事件丢失根因链全探针定谳（b3bi→b3k→b3l→b3m→b3n 六页）：
join 形同页**首触发**必走 caret_range 世界同步回灌+回植 → impl 世界内
querySelector/锚链上溯全落 query 家族代理、页面家族 listener 挂点不可达
（N1 `sameNode=false`、docCap=1 而 bi=0、壳段跨家族抛异常丢 input）→ 二触
起家族合流全通（N2 same=true）。修复实验 4 刀递进（re-query 无效 → 换锚链
宿主 → `__zwCeOrigEvHost` 回植前捕获 [isolated N1 bi=1 input=1 全兑现] →
isSameNode 条件化）：**isolated 全好、语料面 P 473→471 净 -2**——每案
initializeTest 使未合流态恒存，打通派发后多案上下文暴露 **got=2 双派**，
与既有 got=2 342F 集群机制同源（d40-II 定谳：harness 每案
`gEditor !== querySelector(...)` 身份检查上游 [SameObject] 下恒 false 从不
重挂；ZeroWeb querySelector 代理身份不稳定 → 误判换代 → remove 不中 →
listener 遗留 → 双触发；修复刀 = querySelector 同 sel 代理身份稳定性，d42
A/B 判决后转 generation-aware 挂起）。shim 改动已回滚（基线二进制 N1 复现
丢失态确认零净码变）；d40 补测 hookCalls=72 ≤ cases=86（impl 无双调实锤）、
listener 表= `_listenerStore['@'+handle]` 键存储。证据见
[b3-gtr-event-loss-diagnosis](evidence/2026-11-16-b3-gtr-event-loss-diagnosis.md)。

**d38 gtr 家族双根因刀（查询链形 overlay 匹配 + runner 变体展开）**：
gtr 家族 743F 拆解定谳两独立根因，共享路径双刀落树：

① **shim 链形 overlay 匹配**（part05 `_zwOverlayChainAll` 四组合器链求值 helper +
part04 qSA R309 臂/qS R322 臂各补 complex 链分支 + `_readCompoundToken` break 条件补
`+`/`>`/`~` 组合器字符）——根因 A（P12 探针定谳）：同 turn innerHTML 后 mutation 经
host 异步 apply，快照查询对刚写入子树恒 miss（同 turn 链查询 null → 60ms 后自愈命中
= 宿主视图滞后），简单形有 R309/R161 pending 归并臂可看 overlay、链形无归并臂 →
`querySelector("dl + dl > dt")`/`querySelector("P + p > b")` null → 语料
`.firstChild` 崩；根因 B（P14 四象限定谳）：`_readCompoundToken` 不含组合器字符，
无空格链 `p+p>b` 被单 token 吞成伪 tag `'P+P>B'` 的 supported compound → compound
臂恒 miss（空格版 unsupported 正确落链臂，hit/null 四象限恒定）。引擎侧全链
exonerated（parse/match/大小写折叠/子树作用域 fresh-parse 单测 3 件入树：
dom `diag_chain_sibling_match` + engine `diag_bridge_query_all_chain_sibling`/
`diag_parse_html_json_chain_sibling`）。

② **runner 变体展开**（`run_corpus_subdirs_ext` R329 同款：`case_variants` 逐
query 跑 + case 名带 query 与上游 dashboard 对齐；基础 URL 上游 harness 不注册 =
空转）——4 文件 `<meta name=variant>` 矩阵（joining-dl ×2、deleting-in-list-items
×4、joining-dl-and-another-list ×2、non-collapsed ×3）`location.search` 分派，
query 缺席 → "Unhandled variant" 恒崩。**口径变化记档**：input-events case 分母
25→32（1164→3350 子测）。

数字：input-events **289P/875F/3T（1164 子测）→ 473P/2877F（3350 子测）**，
P +184、crash 家族 **321F→1F**、Unhandled variant **488→0**；新账结构 =
innerHTML-mismatch 1480 + beforeinput 事件计数 908 + targetRanges-assert 409
（真实编辑语义面 = B2 剩余/B3）。哨兵：html-semantics 新旧二进制对拍
**P 恒等 5378**、F 461 vs 464（3 差异全落 select-home-end-pagedown 键盘发射带内
flaky，s491 形差带同族；本地 wpt-data 部分拉取态基线 5378P 与 s492 满额 8307P
口径差异系语料不全、非代码回归——记档防误读）；桥测试 **1657P/0F**（含新 diag 2）；
探针 P1-P7/P13/P14 全绿（P8 doc 级链、P9 matches 为范围外已知项）。证据见
[gtr-chain-query-root-cause](evidence/2026-11-16-dc2-gtr-chain-query-root-cause.md)。
**d37 input-events getTargetRanges 切片 1**：
三件套（part01 CE 删除路径 beforeinput 前置 StaticRange 快照
`__zwCePendingTargetRanges` + 传播后重读选区、part05
`InputEvent.prototype.getTargetRanges` getter、runner Actions 键盘链
Backspace/Delete 裸键分流 DeleteBackward/ForwardDelete + 事件键名
UI Events 化）——input-events 全语料 **223P/941F/3T → 289P/875F/3T
净 +66（F2P=66/P2F=0）**：forwarddelete.tentative +34、backspace
+25、during-and-after-dispatch +2、探针 b2face +5；gtr 家族
21P→82P。修饰组合（Shift/Ctrl/Meta/Alt + 编辑键）定谳**完全 no-op**
（探针定谳 HEAD 同位绿 = 失步幻影 PUA：bi=0、落不可见视图、同链隔离
跑 bi=1 漂移不可复现；no-op 对失步免疫，18 修饰面 head→WIP 恒等），
Alt 补 `td_alt_sticky` 粘滞位。哨兵恒等：exec-command 198P/105F、
typing 2P/11F、delete-selection 2P/4F、cut-paste/gtr.html/
select-event-drag T 桶同案、range-exceptions 4F、spin/arrow 4F。
selection 全语料对拍 F2P=0/P2F=1（delete-chars 单面已知间歇，同二进制
复跑自翻；上 session 13 面 whitespaces 词删翻转由 no-op 守卫清偿），
engine lib 4033P/0F、clippy/fmt 干净。证据见
[input-events-get-target-ranges](evidence/2026-11-15-dc2-input-events-get-target-ranges.md)。
**d36 J2 跨 body 边界块合并切片**：
part05 两处（`_b1BlockTags` 36 标签块面表 + `_b1JoinMembers` body 吸收臂
J2 分支：body 可见内容恰单块且 src 可见块 → 靠前块存续 innerHTML 重写并块 +
缝 caret 落靠前块文本末端，零 Rust 变更；上一 session 429 中断遗留 WIP，
本 session 完成验证与落账）——`delete-in-child-of-html` **34P/12F → 42P/4F
净 +8**（8 个 div↔div 跨界并块形 F2P=8/P2F=0，HEAD 基线双二进制定谳
base 34P/12F；余 4F = d17 XOR 伪失败族，出队不动）。tal 三文件 base vs WIP
status+name 级逐行恒等（DM 125P+84P、内壳形 child=b/parent=b&child=i ×4
Timeout 恒）；守护 del-head 64P/4F、IP-html/IP-head 64P/8F 恒、b1s3 四姊妹
+ d31-unlink 探针恒、engine lib 4033P/0F、clippy/fmt 干净。通道勘误记档：
child-doc 语料归 selection 通道（editing-other 通道 `<iframe` 整文件跳过）。
证据见 [j2-cross-body-block-merge](evidence/2026-11-15-dc2-j2-cross-body-block-merge.md)。
**d35 plaintext-only 宿主 styling 命令门批**：
part06.js 四处（`_zwExecCmdStylingCmds` 26 命令表 + `_zwExecCmdFindPlaintextRootEditable`
根可编辑宿主爬升判定 + execCommand 入口门 + queryCommandEnabled 面，零 Rust
变更）——`styling-commands.html` 双 variant **42P/108F ×2 → 150P/0F ×2**，
d34 披露的 unlink P2F=2 清账；门语义取 Blink root editable element：po 嵌
true 内不门、true-in-po 门、ce=false 截断、designMode=on 不门（nested/
designMode 语料逐案定谳表见证据）。selection 全语料 vs d30 对拍
**F2P=290 / P2F=0**（48,668P → 48,958P；证据初稿 281 系预估、已勘正）：
+216 本批、+65 d34 账持位、**+9 delete.html = 04369d39a ip 垫刀窗口增益**
（9 子测全为 `<p><br></p>` 空垫段 join 形，共享 `_zwJusIsExtraneousLineBreak`
判定路径归因定谳，首现于本批对拍属窗口补账）。守护面零漂移：unlink
391P/4F、b1s3/d31 哨兵恒、del/IP-in-child-of-{html,head} 恒、input-events
223P 恒、engine lib 4033P/0F。证据见
[plaintext-only-styling-gate](evidence/2026-11-15-dc2-plaintext-only-styling-gate.md)。
**d34 headEl 持真 title 切片**：part05 R159 分支
head 段解析前剥离（存 `doc._r159HeadHtml` 槽）+ headEl 独立解析入链（s408
trio + R173 盖章，快照迭代避 reparent splice 活数组陷阱）+ part03 detHtml
doc 槽补发 + R167 桥 headEl 真节点搜（qSA("script") 产物 `.remove` 断链
修复）——`delete-in-child-of-html` **24P/22F → 34P/12F 净 +10**，d33 挂账
18F title 门控清账；`doc.title` getter head 快道免修自愈（d22 记档缺口
闭合）。全语料护航零 P2F（本切片）：engine lib 4032P/0F、input-events
223P 恒、unlink 391P/4F 恒、head 姐妹面/b1s3/d31 哨兵恒、selection 全量
对拍 vs d30 账 F2P=65（del-html +20 含 d33 批 +10、IP-html +40 d33 批、
editing-div-outside-body +5 d33 窗口）；P2F=2 归 d31 unlink 臂既有欠账
（stash 基线二进制定谳，非本切片）。余 12F = 8F J2 跨 body 边界块合并
缺口 + 4F S8 XOR 伪失败（d17 族）。证据见
[headel-title-relocation](evidence/2026-11-15-dc2-headel-title-relocation.md)。
**d33 子文档 join/enter html 形精化批**：
part05 四修（doc 级拆半落位按 childNodes 位次 +1 同槽位插、body-caret
直落臂含换向判据、visible head join 双臂（walk 边缘成员 + `root===doc.head`
边界升级）、空体占位尾缀剥离）——`insertparagraph-in-child-of-html`
24P/48F → **64P/8F 净 +40**、`delete-in-child-of-html` 14P/32F →
**24P/22F 净 +10**，批净 **+50**；head 姐妹面与 b1s3/d31 哨兵零漂移，
engine lib 4032P/0F。余 8F = 跨文档收养缺口（引擎级 adoptNode 挂账
FIXME(d33)）；余 22F = 18F title 门控（head 段提升切片）+ 4F XOR 伪失败
（d17 族出队，错误面自证 `expected true got 1`）。证据见
[child-doc-join-enter](evidence/2026-11-15-dc2-child-doc-join-enter.md)。
**d23–d29+d31 B1 切片 3 批**：①typing-around-link
边界臂 + 读断逃逸（`_zwCeReresolveCaret`，caret 快照锚读短按"读已断"放行
重映射）+ 输入通道门（execCommand insertText 让位、键盘径保边界外落）——
selection 全语料交集 **F2P=18/P2F=0**（tal +4；join-pre ×6、editing-div ×5、
end-key ×1 走 caret-move 门 `script_ce_caret_move`）。②unlink 实应用臂
（此前仅 inputType 记名零变更）：`unlink.html` **370P/25F → 391P/4F 净
+21**（余 4F = FIXME(d24) 余样式重裹）；三层分诊（仅 href 整摘/余属性保壳
摘 href/无 href untouched）。③createLink 非 collapsed 包裹臂（IAH
`InsertAdjacentHtmlByPath` + detached FIFO 摘除暂存）落地后暴露 HEAD
**空转假绿 P2F=1**（input-events-exec-command "should remove the link"）：
d31 三轮探针定谳 stamped/wire-only 家族变异原语——工厂产物 fresh 插入收、
parse 节点 cloneNode 产物整类拒收、既有子移动静默 no-op、本地操作不发树
wire——unlink 臂改 **childNodes 数组遍历 + 工厂重建直插 + 首轮落地校验
回退**，P2F 转 Pass，input-events 全量 **223P/936F/3T（净 +1）**、交集
F2P=12+1。新驻留探针 zz-probe-d31-unlink 8P/0F（P1–P4 原语语义 + T1–T4
端到端）。回归面零漂移：unlink.html 391P/4F、bold.html 3039P/9F、
b1s3 4P/9F 恒等。证据见
[dc2-b1-slice3-typing-around-link](evidence/2026-11-15-dc2-b1-slice3-typing-around-link.md)。
**d22 B1 切片 2 子文档键路由批**：子窗 `test_driver` 面 shim 直应用
（part03 +17/part05 +946）——iframe 子文档不经 webview 键管线（选区/树在
shim 工厂域），`send_keys`/`Actions`（PUA 键码 runner 同形映射，方向/行端
键光标移动）+ 键入 typing-around-link 边界面（链接端点键入不扩进链接、
内层剥出重裹）+ 删除块级 join（可见门/族标签 no-op/body 吸收臂/缝上不可
见元素回插）+ insertParagraph（pre `\n`/块拆分克隆）+ 结构面补齐（headEl
属性面、docEl 虚树 innerHTML 读面按 head 族提升、insertBefore 虚参照、
body 树根上行链接、`_zwMakeIframeDoc(undefined)` 除空壳 title、工厂
focus/isConnected、子 execCommand 折叠臂 `__zwB1KeyFace` hook）。runner 侧
selectorFor 文档限定因此无需。验证：selection 全语料
**F2P=367/P2F=1**（typing-around-link DM 变体 collapsed +121/non-collapsed
+80，delete/insertparagraph-in-child-of-{head,html} 64+14+64+24；P2F=1 =
deleteFromDocument Range 18 `[head,1]` 越界——`undefined` title 门后
headEl 视图空，真修属 headEl 持真 title 切片挂账；new-P=24 本刀探针页
zz-probe-b1s2 全绿）；input-events 全语料恒等；face 六文件 19F 逐名恒等
（strike 对拍勘误：b1 轮 `FILTER=strike` 连带探针页 4 断言，精确
strikethrough.html 过滤 2147/2147 恒等）；哨兵 selection 14 文件 + editing
54 文件对 d19 终账恒等（f6bwin 两变体发射深度=共享超时预算非门面，重叠
240 断言 0 错配，HEAD bisect 同病灶）；引擎 lib 钉测 ×2（终源复跑）=
3944 passed / 7 已知红恒等。证据见
[dc2-b1-slice2-child-doc-key-routing](evidence/2026-11-14-dc2-b1-slice2-child-doc-key-routing.md)。
**d21 B1 切片 1 子文档编辑面批**：三修复（part03/part05）——detached doc
parentNode 链（R3019 getter-only 静默失效→own defineProperty）+ iframe 子
文档 execCommand 薄面（333F setup TypeError 解除）+ 子窗 Selection.collapse
空选区臂；六族 676 断言 setup 三层全通。selection/input 全语料 F2P=0/P2F=0
（new-P=7 探针面）；input-events 副效应 bisect 定谳修复 1：8 行硬错转断言
阶段 1 行转 Pass。证据见
[dc2-b1-slice1-child-doc-editing-surface](evidence/2026-11-14-dc2-b1-slice1-child-doc-editing-surface.md)。
**d19 focus no-op 族 + Selection.modify 简形批**：selection 全语料
F2P=168/P2F=0/真 new-fail=0；face/哨兵/input 恒等。证据见
[dc2-focus-noop-selection-modify](evidence/2026-11-14-dc2-focus-noop-selection-modify.md)。
**d18 Selection 校验门批**：collapse/extend/addRange 三道规范校验门落
part06.js 主窗口 Selection。同机受控六文件 before/after
**F2P=6,146/P2F=0/new-fail=0**；selection 全语料对拍
**F2P=11,481/P2F=0/new-fail=0**。证据见
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
[执行契约](../wptfyi-editing.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成（11/11 片；快照 109,059/154,595） | [dc1-run-shards](evidence/2026-11-11-dc1-run-shards.md) + [dc2-selection-wiring](evidence/2026-11-11-dc2-selection-wiring.md) |
| DC-2 按簇收敛 | P1 读面 +114；P2 apply 引擎（多面 +1,647 断言级零新增失败）；P3 作者载体臂 + strike/⑤/①/② 各批 + ③ 载体吸收（+55）+ ⑥ italic toggle-off（+12）+ ⑥ ADD 壳内嵌套 + collapsed qS 记账（+5）；均零回归；selection 扩面已接线；d19 focus 族 + modify 简形（+168）；d21 B1 切片 1 子文档编辑面（setup 三层断链解除，零 P2F，+7 新键 P）；d22 B1 切片 2 子文档键路由（子窗 test_driver 面，selection 全语料 F2P=367/P2F=1，净 +366） | [triage](evidence/2026-11-11-dc2-fail-cluster-triage.md) + [dc2-p2-format-apply](evidence/2026-11-11-dc2-p2-format-apply.md) + [dc2-us-carrier-author-arm](evidence/2026-11-12-dc2-us-carrier-author-arm.md) + [dc2-strike-wrap-tag](evidence/2026-11-12-dc2-strike-wrap-tag.md) + [dc2-element-carrier-fold](evidence/2026-11-12-dc2-element-carrier-fold.md) + [dc2-decl-shell-partial-order](evidence/2026-11-12-dc2-decl-shell-partial-order.md) + [dc2-bold-value-face](evidence/2026-11-12-dc2-bold-value-face.md) + [dc2-bold-italic-carrier-merge](evidence/2026-11-13-dc2-bold-italic-carrier-merge.md) + [dc2-italic-toggleoff-face](evidence/2026-11-13-dc2-italic-toggleoff-face.md) + [dc2-italic-add-shell-collapsed-qs](evidence/2026-11-13-dc2-italic-add-shell-collapsed-qs.md) + [dc2-bold-toggleoff-universe-diag](evidence/2026-11-13-dc2-bold-toggleoff-universe-diag.md)（d10–d12：宇宙面定谳 + 桥层修复落地，bold 12F→8F 零回归）+ [dc2-sub-sup-va-shell](evidence/2026-11-13-dc2-sub-sup-va-shell.md)（d13：sub/sup va 冲突壳 + 语料红账重分类，F2P=20 零回归，六文件 57F→37F）+ [dc2-va-peel-interior-strike-shell](evidence/2026-11-13-dc2-va-peel-interior-strike-shell.md)（d14：strike S1 内部界 peel + sub/sup 残段保面 + s 壳收编槽，F2P=15 零回归，六文件 37F→22F）+ [dc2-strike-s2a-p5-retag](evidence/2026-11-13-dc2-strike-s2a-p5-retag.md)（d15：strike S2a 同 tag 名门 + P5 td span retag 端点重锚，F2P=2 零回归，六文件 22F→20F）+ [dc2-strike-direction-m1-absorb](evidence/2026-11-14-dc2-strike-direction-m1-absorb.md)（d16：strike REMOVE 方向规则 + u/s css-true 裸 span 吸收，F2P=2 零回归，六文件 20F→18F）+ [dc2-window-math-reclassification](evidence/2026-11-14-dc2-window-math-reclassification.md)（d17：窗口数学重分类，零引擎改动，伪失败 13F 定谳出队、真失败收敛 5F，d16 账目勘误 20F→18F）+ [dc2-selection-validation-gates](evidence/2026-11-14-dc2-selection-validation-gates.md)（d18：Selection collapse/extend/addRange 三规范门，受控六文件 F2P=6,146、全语料对拍 F2P=11,481、vanished 1,740，零回归）+ [dc2-focus-noop-selection-modify](evidence/2026-11-14-dc2-focus-noop-selection-modify.md)（d19：focus no-op 族 + Selection.modify 简形，全语料 F2P=168、P2F=0、真 new-fail=0，face/哨兵/input 零回归） | + [dc2-b1-slice1-child-doc-editing-surface](evidence/2026-11-14-dc2-b1-slice1-child-doc-editing-surface.md)（d21：detached doc parentNode 链 + child execCommand 薄面 + child collapse 空选区臂，selection/input 全语料零 P2F，face/哨兵/钉测恒等）+ [dc2-b1-slice2-child-doc-key-routing](evidence/2026-11-14-dc2-b1-slice2-child-doc-key-routing.md)（d22：子窗 test_driver 面 + 键语义 shim 直应用，selection 全语料 F2P=367/P2F=1 净 +366，input/face/哨兵/钉测恒等，P2F 根修挂账 headEl 持真 title）+ [dc2-b1-slice3-typing-around-link](evidence/2026-11-15-dc2-b1-slice3-typing-around-link.md)（d23–d29+d31：tal 边界臂 + 读断逃逸 + 输入通道门，selection 交集 F2P=18/P2F=0；unlink 实应用臂 370P/25F→391P/4F；createLink 包裹臂落地暴露 HEAD 空转假绿 P2F=1 并以 stamped 家族工厂重建配方修复，input-events 全量 223P 净 +1）+ [dc2-plaintext-only-styling-gate](evidence/2026-11-15-dc2-plaintext-only-styling-gate.md)（d35：plaintext-only 宿主 styling 命令门，styling-commands 双 variant 42P/108F→150P/0F，d34 披露 unlink P2F=2 清账，全语料 F2P=290/P2F=0 含 delete.html +9 窗口补账）+ [dc2-j2-cross-body-block-merge](evidence/2026-11-15-dc2-j2-cross-body-block-merge.md)（d36：J2 跨 body 边界块合并，del-html 34P/12F→42P/4F 净 +8 零回归）+ [dc2-input-events-get-target-ranges](evidence/2026-11-15-dc2-input-events-get-target-ranges.md)（d37：B2 切片 1，Actions 编辑键分流 + beforeinput StaticRange 快照面，input-events 223P/941F→289P/875F 净 +66、F2P=66/P2F=0，修饰组合 no-op 定谳失步幻影，selection 对拍零漂移除已知间歇 1 面）+ [dc2-gtr-chain-query-root-cause](evidence/2026-11-16-dc2-gtr-chain-query-root-cause.md)（d38：shim 链形 overlay 匹配补 + runner 变体展开，input-events 473P、crash 321F→1F、分母 25→32 案记档）+ [d42 sel-dialect-knife AB and remap order guard](evidence/2026-11-16-d42-sel-dialect-knife-ab-and-remap-order-guard.md)（d42：sel 方言归一刀 A/B 净 -27 不落 + `_zwCeReanchorRange` 重映射序守卫落树，BASE 恒等纯加固）+ [d43 B4-S1 select-delete](evidence/2026-11-16-d43-b4-s1-select-delete.md)（d43：元素锚/跨容器 range select-delete 主臂，input-events 461P→715P 净 +254 F2P=254/P2F=0，html-semantics P 恒等 5378、selection F2P=+10/真 P2F=0、engine lib 4059P/0F）+ [d44 join-br-gtr-endpoint](evidence/2026-11-16-d44-b4-s2-join-br-gtr-endpoint.md)（d44/d45：join 缝尾随 br 消费 + gtr 端点克隆/持活分流，input-events 715P→1099P 净 +384 F2P=384/P2F=0，html-semantics P 恒等、selection P/F 集双重恒等，dwuf 未清偿持位）+ [d47 query-identity-alias 判决不落](evidence/2026-11-17-d47-query-identity-alias.md)（d47：查询返回点身份别名，input-events +331 但 selection 净 −5,556，与 d42 同为入口级归一刀负样本，WIP stash 配方保留零码上树） |
| DC-3 全量对账 | 对账口径就绪（11-12 快照三目录 130,245、距线 15,939；d31 后预估差 ~3.6k，正式落账待下周快照） | coverage-2026-11-11 快照 + 验收线 146,184 + [d22 selection 全语料对拍](evidence/2026-11-14-dc2-b1-slice2-child-doc-key-routing.md) |

碰撞记录：本批 pin 钉测与并行流 D1 的 workspace 全量 test+clippy 共
享 build 目录锁（2026-11-13），改用独立 CARGO_TARGET_DIR 完成钉测，
未触碰对方进程；后续重型编译撞锁时同法分流或等待。
