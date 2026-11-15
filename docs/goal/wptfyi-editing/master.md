# Editing、Selection 与 Input Events控制面

**2026-11-16 · Active**。**d39 B3 诊断轮（零净码变）**：beforeinput 计数 908F
重分类定谳——**主簇 = 元素锚 range 选区 silent return**（list-items 592 +
dl-join 152 + non-collapsed 76 ≈ 820F，`__zw_ce_delete_impl` 12026/12155
sc 非 text 直接 return、树零变，select-delete 语义缺失），**事件丢失集群占比
极小**（此前 round-3 日志的主簇判断是误诊）。真事件丢失根因链全探针定谳
（b3bi→b3k→b3l→b3m→b3n 六页）：join 形同页**首触发**必走 caret_range 世界
同步回灌+回植 → impl 世界内 querySelector/锚链上溯全落 query 家族代理、
页面家族 listener 挂点不可达（N1 `sameNode=false`、docCap=1 而 bi=0、壳段
跨家族抛异常丢 input）→ 二触起家族合流全通（N2 same=true）。修复实验
4 刀递进（re-query 无效 → 换锚链宿主 → `__zwCeOrigEvHost` 回植前捕获
[isolated N1 bi=1 input=1 全兑现] → isSameNode 条件化）：**isolated 全好、
语料面 P 473→471 净 -2**——每案 initializeTest 使未合流态恒存，打通派发后
多案上下文暴露 **got=2 双派**（div-p 案 0→2、O1 isolated 单收 corpus 双收），
**与既有 got=2 342F 集群机制同源嫌疑高**。shim 改动已 `git checkout` 回滚
（基线二进制 N1 复现丢失态确认零净码变）。**d40 B3P 注入语料探针补测**：
hookCalls=72 ≤ cases=86（impl 无双调实锤、webview 重试假设排除）、双派案簇 =
join/br 空白形与 got=0 同簇（同簇案跨环境呈现 got=0/got=2，非两个独立 bug）、
注入敏感性定谳（hook 跨包装使家族判定漂移——计数探针有效、行为探针失真）、
listener 表= `_listenerStore['@'+handle]` 键存储 + `_zwQueryWrapIdentity`
身份桥定位。下轮刀序：①双派根因（**先修双派再落事件丢失刀**，否则正确性
修复会把 got=0 成批变 got=2）——跨案 dump `_listenerStore` 键序列 + gEditor
handle 轨迹，分辨同 handle 双 listener vs 双 handle 双表 ②元素锚 range
选区删除语义（820F 主簇）③空 range beforeinput 静默臂（语义已定谳
`checkGetTargetRangesOfInputOnDoNothing`，小刀独立可落）。
证据见 [b3-gtr-event-loss-diagnosis](evidence/2026-11-16-b3-gtr-event-loss-diagnosis.md)。

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
**卡点** 无阻塞。11-12 快照口径：目标三目录合计 130,245、距 146,184
差 15,939（前文"差 37,125"系 11-07 基线口径未刷新，勘误）；d22 后
selection 全语料 47,716P（含探针页断言面；较 d21 后 47,326 净 +366 =
F2P 367 − P2F 1）、input-events 211P 恒等；d23–d31 批净 +52（selection
交集 +18、input-events +13、unlink.html +21），d33 批净 +50
（delete/insertparagraph-in-child-of-html +10/+40），三目录预估 ~142.7k、
差 ~3.5k（预估口径，正式落账待下周 DC-3 快照；d34 切片 +10 后
~142.75k；d35 批 +216 后 ~142.97k；d36 批 +8 后 ~142.98k；d37 批
+66 后 ~143.0k；d38 批 P +184（变体展开口径同批扩大，P 增量含展开
分母新增案，跨口径不直加——正式落账以 DC-3 快照统一口径为准））。
子文档域残余：
typing-around-link **CE（主窗）变体**
collapsed ~122F + non-collapsed ~77F + after-joining-paragraphs 48F（边界
切片已入执行链，tal 内壳形 `child=b`/`parent=b&child=i` ×4 Timeout——收敛
前置=单一补偿面 + caret 引擎同位快照（d24 切片））+
delete-in-child-of-html 12F（8F J2 跨 body 边界块合并缺口——吸收臂展平
单侧、无跨界块合并，独立后续切片 + 4F XOR 伪失败出队）/
insertparagraph-in-child-of-html 8F（跨文档收养缺口，引擎级 adoptNode
挂账 FIXME(d33)）+ d22 记档
deleteFromDocument `[head,1]` 越界 P2F=1 当前账面无对应红项（d34 headEl
持真后 deleteFromDocument 族全 Pass，账面清）+ unlink 余样式重裹 4F
（FIXME(d24)）。
modify 残余 ~57F（定向 probe 口径）= 鼠标驱动 modify（2/7×2+3/7，事件
集成面）、line-flex/grid 0/2×3（FIXME(visual-modify) 行盒几何）、
inline-boundary 18/36、bidi character 22/28×2（RTL 视觉轴近似）、word
细分（korean 2/4、symbol 2/6、generated-content 1/2）——近似面已标
FIXME，按簇评估是否进精化批。其余桶：text-control 期望矩阵 ~474F（B3）+
input-events getTargetRanges（B2，切片 1 已落 d37 +66：gtr 家族 9 文件
余 743F/1T = list-items/dl-join/across-boundaries join 面细化 + 主文件
gtr.html Timeout 案；修饰组合词删语义 deleteWordBackward/Forward、
range-exceptions InputEvent 构造器 targetRanges dict + StaticRange
IndexSizeError 校验、typing DELETE join-walk 分离 sc 崩点守卫挂 B2
后续刀）+ shadow-dom getComposedRanges
62F（B4）——修复队列 B1–B5 见
[dc2-residual-bucket-triage](evidence/2026-11-14-dc2-residual-bucket-triage.md)。六 format 文件残余
19F（HEAD 基线口径，d17 账 18F 勘误 +1）：bold 9F、italic 3F、
strike 4F、underline 3F、sub/sup 0F。重分类终账：伪失败 13F（切片
割裂 harness 固有，不进修复队列）= strike/underline 条目 65/133/134 +
bold 条目 75/139/204 + italic 条目 131；真失败 6F：strike S2e qS-after
1F（挂账 Chrome-quirk 家族）+ bold b-in-i / italic i-in-b 块边界
innerHTML 2+2F + bold notbold 条目 1F（sw=true innerHTML 内层裸 span，
prefix 对拍定谳 HEAD 既有欠账非 d18 引入，溯源指向 02:12 前落地批
20521a2ba/8add42713 一带，待 bisect）。哨兵 fncorpus 两窗页级 Timeout
（d15 已有，fontname 域待查小桶）；f6bwin 两窗发射深度=共享超时预算非
门面（d22 bisect 定谳：HEAD/d22 二进制同病灶，重叠 240 断言 0 错配）。
桥层支线三 bug（tc.querySelector innerHTML 重建后空包装、
addRange 后 getRangeAt(0) 越界、B2 臂双落地）未动，主链经 R263 补偿
后已与 B2 臂解耦。
**下一步** ①**B1 切片 3 收尾**（headEl 持真 title 已落 d34 切片——
del-html +10、title 门控清账；边界臂/通道门/unlink 臂/html 宿主
join/enter 精化已落，见 d33 顶格段；plaintext-only P2F=2 已落 d35 批
清账；J2 跨 body 边界块合并已落 d36 批——del-html 42P/4F，余 4F =
d17 XOR 伪失败族出队）：tal 内壳形收敛（前置
d24 单一补偿面 + caret 引擎同位快照）+ 跨文档收养（引擎级 adoptNode，
IP-html 余 8F，FIXME(d33)）+ execCommand
insertParagraph per-doc 分隔符消费；②**B2 切片 1 已落 d37 批**
（Actions 编辑键分流 + getTargetRanges 快照面，input-events 净 +66、
P2F=0）→ B2 切片 2 残余刀：修饰组合词删语义
（deleteWordBackward/Forward，delete-selection 2 Ctrl 面）、
range-exceptions InputEvent 构造器 targetRanges dict + StaticRange
IndexSizeError 校验（4F）、typing DELETE join-walk 分离 sc 崩点守卫
（4 笔 unhandled rejection）、gtr 家族 join 面真实语义刀
（**d38 双根因刀已落**：crash 家族 321F→1F + Unhandled variant 488→0，
新账 = innerHTML-mismatch 1480 + beforeinput 计数 908 +
targetRanges-assert 409——join/merge 语义细化与 getTargetRanges 值面，
见顶格 d38 段；**d39 诊断轮**：908F 主簇重分类 = 元素锚 range 选区
silent return ≈820F + join 首触发事件丢失根因链定谳（修复实验语料面
净 -2 已回滚）+ got=2 双派同源嫌疑——刀序见顶格 d39 段）→ **B3 text-control execCommand 期望矩阵**（~474F，修前逐命令核对语料期望表）
→ B4 shadow-dom getComposedRanges（62F）→ B5 小簇（cloning-attributes
null 崩溃 17F 等）；③mt 慢页 systemd-run 长预算（d13 口径，MemoryMax
封顶）补跑 strike/u/bold/italic 链对拍 + multipara/跨块族（M2/B1-B4
per-p 拆壳，真失败 b-in-i/i-in-b 块边界 innerHTML 4F 同域）评估开修，
bold notbold 1F bisect 溯源（20521a2ba/8add42713 一带），selection
editing/other ~47 慢页补全；④下周快照落做 DC-3 同口径对账（净增
≥130,000 + 邻域回退核对，P2/P3 语料全量增量随周跑入账；face 哨兵探针：
d5decl/d5diag/d6cef/d7diag/d10diag/d11tree/d13vashell 全绿、d6diag
22/25（两轮恒等，旧记 24/25 笔误已勘）、d8diag 11/11、d9diag 10 case
全绿、d15strike 2/2、d16strike 2/2、d18sel 3/3、d19focus 5/5、d19word 3
探针、d22b1s2 24/24；组成漂移后哨兵判读改 ledger 集合判读
P2F/new-fail）。

[执行契约](../wptfyi-editing.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 完成（11/11 片；快照 109,059/154,595） | [dc1-run-shards](evidence/2026-11-11-dc1-run-shards.md) + [dc2-selection-wiring](evidence/2026-11-11-dc2-selection-wiring.md) |
| DC-2 按簇收敛 | P1 读面 +114；P2 apply 引擎（多面 +1,647 断言级零新增失败）；P3 作者载体臂 + strike/⑤/①/② 各批 + ③ 载体吸收（+55）+ ⑥ italic toggle-off（+12）+ ⑥ ADD 壳内嵌套 + collapsed qS 记账（+5）；均零回归；selection 扩面已接线；d19 focus 族 + modify 简形（+168）；d21 B1 切片 1 子文档编辑面（setup 三层断链解除，零 P2F，+7 新键 P）；d22 B1 切片 2 子文档键路由（子窗 test_driver 面，selection 全语料 F2P=367/P2F=1，净 +366） | [triage](evidence/2026-11-11-dc2-fail-cluster-triage.md) + [dc2-p2-format-apply](evidence/2026-11-11-dc2-p2-format-apply.md) + [dc2-us-carrier-author-arm](evidence/2026-11-12-dc2-us-carrier-author-arm.md) + [dc2-strike-wrap-tag](evidence/2026-11-12-dc2-strike-wrap-tag.md) + [dc2-element-carrier-fold](evidence/2026-11-12-dc2-element-carrier-fold.md) + [dc2-decl-shell-partial-order](evidence/2026-11-12-dc2-decl-shell-partial-order.md) + [dc2-bold-value-face](evidence/2026-11-12-dc2-bold-value-face.md) + [dc2-bold-italic-carrier-merge](evidence/2026-11-13-dc2-bold-italic-carrier-merge.md) + [dc2-italic-toggleoff-face](evidence/2026-11-13-dc2-italic-toggleoff-face.md) + [dc2-italic-add-shell-collapsed-qs](evidence/2026-11-13-dc2-italic-add-shell-collapsed-qs.md) + [dc2-bold-toggleoff-universe-diag](evidence/2026-11-13-dc2-bold-toggleoff-universe-diag.md)（d10–d12：宇宙面定谳 + 桥层修复落地，bold 12F→8F 零回归）+ [dc2-sub-sup-va-shell](evidence/2026-11-13-dc2-sub-sup-va-shell.md)（d13：sub/sup va 冲突壳 + 语料红账重分类，F2P=20 零回归，六文件 57F→37F）+ [dc2-va-peel-interior-strike-shell](evidence/2026-11-13-dc2-va-peel-interior-strike-shell.md)（d14：strike S1 内部界 peel + sub/sup 残段保面 + s 壳收编槽，F2P=15 零回归，六文件 37F→22F）+ [dc2-strike-s2a-p5-retag](evidence/2026-11-13-dc2-strike-s2a-p5-retag.md)（d15：strike S2a 同 tag 名门 + P5 td span retag 端点重锚，F2P=2 零回归，六文件 22F→20F）+ [dc2-strike-direction-m1-absorb](evidence/2026-11-14-dc2-strike-direction-m1-absorb.md)（d16：strike REMOVE 方向规则 + u/s css-true 裸 span 吸收，F2P=2 零回归，六文件 20F→18F）+ [dc2-window-math-reclassification](evidence/2026-11-14-dc2-window-math-reclassification.md)（d17：窗口数学重分类，零引擎改动，伪失败 13F 定谳出队、真失败收敛 5F，d16 账目勘误 20F→18F）+ [dc2-selection-validation-gates](evidence/2026-11-14-dc2-selection-validation-gates.md)（d18：Selection collapse/extend/addRange 三规范门，受控六文件 F2P=6,146、全语料对拍 F2P=11,481、vanished 1,740，零回归）+ [dc2-focus-noop-selection-modify](evidence/2026-11-14-dc2-focus-noop-selection-modify.md)（d19：focus no-op 族 + Selection.modify 简形，全语料 F2P=168、P2F=0、真 new-fail=0，face/哨兵/input 零回归） | + [dc2-b1-slice1-child-doc-editing-surface](evidence/2026-11-14-dc2-b1-slice1-child-doc-editing-surface.md)（d21：detached doc parentNode 链 + child execCommand 薄面 + child collapse 空选区臂，selection/input 全语料零 P2F，face/哨兵/钉测恒等）+ [dc2-b1-slice2-child-doc-key-routing](evidence/2026-11-14-dc2-b1-slice2-child-doc-key-routing.md)（d22：子窗 test_driver 面 + 键语义 shim 直应用，selection 全语料 F2P=367/P2F=1 净 +366，input/face/哨兵/钉测恒等，P2F 根修挂账 headEl 持真 title）+ [dc2-b1-slice3-typing-around-link](evidence/2026-11-15-dc2-b1-slice3-typing-around-link.md)（d23–d29+d31：tal 边界臂 + 读断逃逸 + 输入通道门，selection 交集 F2P=18/P2F=0；unlink 实应用臂 370P/25F→391P/4F；createLink 包裹臂落地暴露 HEAD 空转假绿 P2F=1 并以 stamped 家族工厂重建配方修复，input-events 全量 223P 净 +1）+ [dc2-plaintext-only-styling-gate](evidence/2026-11-15-dc2-plaintext-only-styling-gate.md)（d35：plaintext-only 宿主 styling 命令门，styling-commands 双 variant 42P/108F→150P/0F，d34 披露 unlink P2F=2 清账，全语料 F2P=290/P2F=0 含 delete.html +9 窗口补账）+ [dc2-j2-cross-body-block-merge](evidence/2026-11-15-dc2-j2-cross-body-block-merge.md)（d36：J2 跨 body 边界块合并，del-html 34P/12F→42P/4F 净 +8 零回归）+ [dc2-input-events-get-target-ranges](evidence/2026-11-15-dc2-input-events-get-target-ranges.md)（d37：B2 切片 1，Actions 编辑键分流 + beforeinput StaticRange 快照面，input-events 223P/941F→289P/875F 净 +66、F2P=66/P2F=0，修饰组合 no-op 定谳失步幻影，selection 对拍零漂移除已知间歇 1 面） |
| DC-3 全量对账 | 对账口径就绪（11-12 快照三目录 130,245、距线 15,939；d31 后预估差 ~3.6k，正式落账待下周快照） | coverage-2026-11-11 快照 + 验收线 146,184 + [d22 selection 全语料对拍](evidence/2026-11-14-dc2-b1-slice2-child-doc-key-routing.md) |

碰撞记录：本批 pin 钉测与并行流 D1 的 workspace 全量 test+clippy 共
享 build 目录锁（2026-11-13），改用独立 CARGO_TARGET_DIR 完成钉测，
未触碰对方进程；后续重型编译撞锁时同法分流或等待。
