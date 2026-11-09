# 2026-11-07 fbq 四命令零应用族 apply 刀（part01g）——multitest 837F 普查 Family A 收口

日期：2026-11-07。目标族 = mt 837F 普查 [Family A](../../editing-compat.md)（formatblock/indent/inserthorizontalrule/insertimage 四命令 apply 面零应用，mt 241 行）+ 四独立语料页。**pre 基线合计 1,329F**：formatblock 4,053P/993F、indent 1,183P/146F、inserthorizontalrule 1,234P/114F、insertimage 886P/76F（`batch-fbq-pre-*.log`，刀前 nq7 commit 5d26dc17b 同二进制采）。mt 基线 = `batch-nq7-mt-post.log`（19,331P/837F）。

## 病灶

四命令此前仅 `_zwExecCmdInputType` 有名（事件照派、execCommand 返 true），apply 面缺失——DOM 零变更，`<div>foo[]bar</div>` 类语料期望全数落红。formatBlock qV 读面同时缺位（qV 恒空串，:2521 陈旧注释）。

## 刀（part01g.js 新文件 473 行 + part06 四 apply 臂 + 前置校验 + qV 读面）

忠实移植 vendored implementation.js（`third_party/wpt/editing/` 参考行号见文件头注释）：

- **formatBlock action**（:6580-6698）：白名单 normalize → blockExtend → contained 可编辑元素过滤 → recordValues/splitParent 环/restoreValues → wrap 环（single-line-container 解包臂 ∨ sibling 聚合）→ fixDisallowedAncestors。**Chrome delta（宿主豁免）**：splitParent 循环判据 spec 为 `inSameEditingHost(ancestor, node)`，裸宿主形下 host 自身满足判据会被拆产双层 contenteditable——录音铁证（multitest `"foo[]bar"` exp `<div>foo[]bar</div>` 单层）定谳祖先爬升跳过 editing host，刀口加 `!_zwJusIsEditingHost(anc)` 前置。
- **formatBlock qV 读面**（:7052-7120）：blockExtend → `_zwJusCollectContainedNodes` 全树叶（visible+editable+无子）→ 爬升（parent 可编辑同宿主非 formattable）→ 终判 editable formattable 无 prohibited paragraph child 后代 → `_zwJusTag`。
- **indent**（:5482-5517 indentNodes + :7141-7230 action）：items 树序收集（end/start/cca 三爬升）→ normalizeSublists → contained（allowedChild div/ol）→ 首可见 li 前驱 normalize → sublist 聚合 → indentNodes（ol/ul 父同型 tag-wrap ∨ blockquote simpleIndentationElement + fixDisallowed；CSS margin-div 臂 FIXME 未移植）。
- **insertHorizontalRule**（:7316-7367）：offset 爬升归一 → sel collapse/extend → deleteSelection(blockMerging:false) → Text 重锚 → createElement('hr') → fixDisallowed → collapse(parent, 1+idx)。
- **insertImage**（:6773-6805）：空值返 false → deleteSelection(stripWrappers:false) → block 单 br 子臂 → img+src 插入 → 选区重锚。
- **前置校验（事件前拒绝）**：formatblock 非法值（白名单外，语料 `<del>/<dl>/<ins>/<li>/<ol>/<ul>/<quasit>` 录音 exec 返 [false] 树不变）+ insertimage 空值（语料 :18-22 [false]）→ Chrome 口径零事件返 false。
- part06：四 apply 臂（事件口径同 inserttext 臂，`__zw_ce_*` 单点调用）+ value_validate 块后前置校验臂 + queryCommandValue formatblock 臂。

## 探针（zz-probe-fbq.html，wpt-data 本地 gitignored）

首跑（01:52 `probe-fbq-build.log`，11 案版）判读（树 + 选区 + qreads 三方对齐）：

| 案 | 判据 | 结果 |
|---|---|---|
| fb-div | `<div>foobar</div>` + sel text@3 + qV ""→"div" | ✓ |
| fb-ph1（:1554） | `<h1>foo</h1><p>bar</p>` + qV p→h1 + sel 重锚 h1 | ✓ |
| fb-invalid | rets=[false] 树不变 qV 恒 div（前置校验零事件） | ✓ |
| hr-basic | `foo<hr>bar` + sel host@2（{}bar 位） | ✓ |
| hr-crossblock | `<p>foo</p><hr><p>quz</p>` + sel host@2 | ✓ |
| img-basic | img 插入 + 选区重锚 | ✓（笔误源版；foobarbaz 修正版复跑见下） |
| img-empty | rets=[false] 树不变（空值拒） | ✓ |
| indent-basic | `<blockquote>foobar</blockquote><p>extra</p>` + sel blockquote 内 | ✓ |
| indent-ol | `<ol><li>foo</li><ol><li>bar</li></ol><li>baz</li></ol>` 同型嵌套 tag-wrap 臂 + qS-iol=true 恒一 | ✓ |
| G-typing | 树 `<div>foobar<b>a</b></div>` 正确（typing 壳保持守卫过）+ qV=div | 树 ✓ / caret 偏尾挂账（见下） |
| G-38-iol | qSA=true 恒一（四臂不扰 list toggle 面） | ✓ |

待补跑：img-basic 修正源（foobarbaz）版、G-fb-only/G-fb-plain（G-typing caret 定位案）。

## 首跑 P2F=44 三族归因（formatblock 腿 4,053P→4,809P / 993F→237F，F2P=800）

| 族 | 行数 | 机制 | 处置 |
|---|---|---|---|
| 白名单外值 ret | 28 | `<article>/<aside>/<blockquote>/<footer>/<header>/<hgroup>/<nav>/<section>` 语料 ret 期望 true 且树真 wrap（compare 行刀前全红＝录音树已变）——spec 白名单（implementation.js :6577-6580 仅 12 名）外，Chrome 真实 Blink 名单另收 8 名。首跑我按 spec 拒 → ret 巧合翻转（pre 零应用返 true 巧合过 / post 白名单拒 false） | **名单扩 8 名**（`_zwFbqFormattableNames` Chrome 口径，apply/前置校验/qV 读面三处共用单点自动生效；`<del>/<dl>/<ins>/<li>/<ol>/<ul>/<quasit>` 录音 [false] 维持拒） |
| qV before 激活面 | 12 | 6 案 ×2 dps 前置，语料 qV before 录音全为 "" + qI 录音 true（跨块/跨宿主边界选区 Chrome 判「不定且无值」签名）；spec 算法（implementation.js :6767-6798 first-leaf 爬升）返真值 "div"/"h1"/"p"。刀前引擎 qV 无臂恒 "" 巧合过，刀后 qV 臂接通返 spec 真值 ≠ 录音 | **挂账**：qV/qI 读面 Chrome delta 族（qv13 轮录音 provenance 同族），需专项按录音定 Chrome 口径（跨边界选区 → indeterm true + value ""） |
| 裸名/内容模型边界 | 4 | :1874 p→div、:1864 span→p 形 Chrome ret=[false] do-nothing（注释「`<p>` cannot have `<div>`」＝内容模型边界），:1879 div→p、:1884 div→div wrap true——apply 面需内容模型检查 delta；compare + qV after 行翻红 | **挂账**：apply 内容模型边界族 |

白名单修复预期联动翻绿：28 ret 行 + 8 名案的 compare/qV after 行（扩名后 qV 爬升返 "article" 等新名）。qI 行（qI 臂未接，期望 false）维持既有红。

F2P=800 断言类型分布：Value-after 290 + Value-before 239（qV 臂接通：before 录音真值行 + after wrap 后真值行）+ compare 222（apply 树变更命中）+ ret 49（前置校验命中——白名单内但 Chrome 拒值如 `<li>/<ol>` 录音 false，pre 零应用返 true 红 / post 事件前拒返 false 绿）。

## 挂账：inserttext 对 wrap 后树 caret 偏尾（typing 链既有红，无 P2F 风险）

G-typing 三连（bold → formatblock `<div>` → inserttext 'a'）：树与 qV 正确，但 'a' 落在文本尾（`<div>foobara</div>`@7）而非 typing 壳内（期望 `<div>foo<b>a[]</b>bar</div>`）。定位：fb_apply 自身落位正确（sel=text@3），偏移发生在 inserttext 经 `__zw_ce_caret_range` 的世界同步/两级重锚对 wrap 后树的落位。~~该行为刀前已红（Family A 行本就是全红行），非本刀回归~~——mt P2F=64 族定谳后定性修正：四命令重锚对 inline 壳内 caret 落位/样式读数的扰动是**本刀激活引入**（见上节），本探针形为同族 caret 面；typing 链挂账收口另立项。

## 门禁批内存爆定谳与修复（part01e 进度防御刀）

首跑门禁批 formatblock 腿被 test-guard SIGTERM（`单进程内存超限 (4256184 KB > 4194304 KB)`）。定位链三轮收敛：

1. **黑盒层**：zz-fbq-full 单页（359 条）155s 4.3GB 复现；8 分片 fresh 进程全绿；c120（0-120）完成 125s、c240（0-240）175s 被杀；段页 s1224（[120,240] fresh）85s / s2436（[240,359] fresh）37s 均正常完成 → 表象 = 单进程连续 ~200+ 条真实 apply 后爆，pre 零应用 4,053 条不爆（apply 缺失时死循环路径不可达）。
2. **精确定位**：`_zwListFixDisallowedAncestors` 允许子臂（part01e :527+）——`_zwListSplitParent` 走 plain 域拷贝路径时**身份不迁**（副本上移、原 node 留原地），while 判定 `node.parentNode && !allowed(node,parent)` 引用永不变 → pg++<256 上界内每轮重克隆整棵子树，分配指数膨胀。formatblock 语料条目 360（display:block span）/ 361（不可编辑 span 混合形）单案复现实证。
3. **修复（part01e 单笔 +7）**：splitParent 前快照 `prevParent`，调用后 `node.parentNode === prevParent` 即 break——真移动路径必迁位不受影响，list 刀既有调用零行为变化（mt 腿 judge P2F=0 佐证 list toggle 面恒等）。

跑法绕过双保险：formatblock 腿按 8 分片（zz-fbq-b0..b7，每片 ~45 条 fresh 进程）跑，judge key 剥页名拼接与 pre 语料 log key 一致；分片 bisect 页含 `/common/subset-tests.js` include（首版 bisect 页缺 include 致 `subsetTest is not defined` 零用例执行假绿，已修正复跑）。修复后四语料腿 5046/1329/1348/962 行全部完整出账。

## 门禁（batch-fbq-legs，systemd user 单元 + flock + MemoryMax=24G + TimeoutStartSec=90min）

| 腿 | 结果 | judge（vs pre 基线） | VERDICT |
|---|---|---|---|
| formatblock | 4,809P/237F（pre 4,053P/993F） | F2P=800 / P2F=44（三族归因见上节） / lost 0 | REVIEW→白名单修复 |
| indent | 1,302P/27F（pre 1,183P/146F） | F2P=119 / P2F=0 / lost 0 | OK |
| inserthorizontalrule | 1,348P/**0F**（pre 1,234P/114F） | F2P=114 / P2F=0 / lost 0——**腿全绿** | OK |
| insertimage | 939P/23F（pre 886P/76F） | F2P=53 / P2F=0 / lost 0 | OK |
| enginelib | 3,782P/9F | 九红名册恒一 | OK |
| mt | 首跑 mt 腿 02:23 被并行轮慢化定位实验 SIGTERM 终止（无数据），终判由 mt2 补跑（见下节）：19,485P/646F（rc=2 单页尾部 1 案 unreported 超时，先例签名） | vs nq7 基线 rows 10092 恒一，F2P=93 / P2F=0 / post-only 0 / lost 0 | OK |

四语料净减红 1,086 行（1,329F→287F）。mt 语料 formatblock 值分布核实：全 24 处均为 `<div>`（spec 12 名内）——扩 8 名对 mt 零行为影响，mt 批即终判数据。批脚本 `.acceptance/dps/batch-fbq-legs.sh`（head=5d26dc17b 工作树含刀，二进制 marker `fb_apply=4`；与 commit 6afc857bf 内容一致）。

indent 剩 27F 零散边界族（不可编辑 span 边界 `{ <span contenteditable=false>A…}`、`<p>[abc]</p>` 全选 wrap、br 交错 `foo[bar]<br>baz`、blockquote 相邻聚合、table data-start）——非 CSS margin-div 主族，后续小刀多批。img 剩 23F 零散边界族（`foo{<span>bar</span>}baz` inline 全选、跨 color span 边界、dps p + 跨块 style 形）。

## 白名单复跑中间态判读（02:20:57 二进制 = 8 名 + FBQ-STAGE 插桩残留）

formatblock 8 分片复跑（`batch-fbq-wl-rerun.sh`，行源页 zz-fbq-b0..b7）：**F2P=644 / P2F=13 / postF=0 / lost 30**，VERDICT REVIEW。归因（03:10 修订）：

- **lost 30 = 分片覆盖洞非插桩伪影**：post2 5016 行 vs pre 5046 行恰差 30，尾族（条目 359-362：cediv×2 + spanblk + `{ <span cE=false>A…`）在任何分片零行——b7 bisect 页生成器 off-by-one（`SLICE_B=359` 漏 359）+ 8×45 只盖 [0,360)，zz-fbq-b8.html（[359,363)，并行轮建）页头注释与补跑实证：b8 补跑 29P+1F 恰 30 行（1F = entry 362 compare，pre 同红 F2F）。FBQ-STAGE 插桩归因作废（抛错产生 Fail 行解释不了行缺失；post2 无 FBQ-STAGE 效应行；该批实为 8 分片跑法非整页）。切片页 zz-fbq-lost.html（6 案 verbatim + 同 runner 机具，fresh 进程）54 行全数落账（51P+3F），证条目本体健康。
- **P2F 13 三族**：①嵌套 contenteditable=false 链形 ×6（compare + qV after 成对）——并行轮第二刀可编辑链守卫已修（legs3 判读消失，切片页同形 compare/qV 全绿）；②同 tag no-op 形 ×2（`<div><ol><li>[foo]</ol></div>` compare）+ ③双 p 形 ×1（`<p>[foo<p>bar]` compare）——第三刀修复面（见下节）；④dps qV before ×4（`<dd>/<div>/<p>` 前置案）——8 分片 fresh 进程断 dps 粘滞链伪影（簇甲，判读滤键），非引擎债。
- formatblock 腿终判数据待第三刀单页跑法批；mt 腿不受影响（24 处值全 `<div>` 简单形，刀零命中，见上节核实）。

**探针腿作废**：复跑批探针腿跑的是并行轮 02:14:37 版探针页（23 案，每案 assert_false 信息行）——runner 每 test 只吐首个失败断言 message，post 态快照全部丢失，23 行 Fail 仅含 pre 快照，判读不可靠。并行轮 02:27:10 已重写为聚合行版（`results.join(' || ')` 单断言一次 dump 全部 pre/post 信息，18 案），探针复核待聚合版重跑判读。

### 探针复核（02:27:10 聚合版，第二刀版二进制，02:48 跑）——16/18 合

| 案 | 判据 | 结果 |
|---|---|---|
| fb-div / fb-ph1 / fb-invalid | wrap/qV 换值/前置拒三基线 | ✓✓✓ |
| hr-basic / hr-crossblock | hr 插入 + 选区重锚（`foo<hr>bar` / `<p>foo</p><hr><p>quz</p>`） | ✓✓ |
| img-basic（foobarbaz 修正源）/ img-empty | img 插入 / 空值拒 | ✓✓ |
| indent-basic / indent-ol | blockquote wrap / 同型嵌套 tag-wrap | ✓✓ |
| fb-dl | dt ∈ SLC 解包换块 `<div>foo</div><dl><dd>bar</dd></dl>`，qV "dt"→"div" | ✓ |
| fb-olbare | 无屏障 wrap `<ol><li><div>foobar</div></li></ol>`（与语料 exp 逐字一致） | ✓ |
| fb-cross | 跨块换块 + **pre qV-fb=""（引擎跨块形返空与录音一致——挂账 qV-before 12 行需按真实案形复查，跨块裸形已对）** | ✓ |
| fb-brace | brace 全含归一 → blockquote wrap | ✓ |
| G-typing | 树 `<div>foobar<b>a</b></div>`（'a' 入壳 ✓）+ caret 落尾（sel DIV@1 期望壳内）——caret 挂账维持 | 树 ✓ caret 挂账 |
| G-fb-only | formatblock 后 caret= text@3 ✓（fb_apply 重锚正常） | ✓ |
| G-fb-plain | **无 typing 壳也落尾**（`<div>foobara</div>`@7，期望 @3 插 `fooabar`）——挂账定位收紧：inserttext `__zw_ce_caret_range` 世界同步对 wrap 后 caret 失灵，与 typing 壳无关 | 挂账定位更新 |
| G-38-iol | qS-iol=true 恒一（list toggle 面不扰） | ✓ |
| fb-listbarrier | SETEXC IndexSizeError（探针 setSel setup 自身问题非引擎债；同 tag no-op 臂未由此案验证，第二刀批判读覆盖） | 探针 setup 小修案 |

## 第三刀（真残余 P2F 三行收口）——四笔草案 × 一笔撤销 × 三笔恢复（2026-11-07 02:44-03:35）

legs3 判读（02:47 二进制）：fb F2P 794 / P2F 7（4 dps 伪影 + ol-li ×2 + 双 p ×1）/ lost 30（覆盖洞，见上节）/ mt F2P 71 / **P2F 64（新族，见下节）**。

**legs3 二进制 = 部分快照**（03:30 定谳）：build 窗口（02:46:40 起）落在四笔编辑之间——dd/dt 20 行回归证明允许性预检在内（B2-vs-NEW 行级对账，核验方 `VERIFY-NOTE-fbq-b1b2-regression.md` v2/v3 + 告警文档）；三重 p got 树（`<p>bar</p><p>foo</p><p>bar</p>`）证明 wrap 第 4 参不在；ol-li got `<li><div>foo</div></li>` 证明列表透明 skip 不在。legs3 fb 行级结论对任何单一树状态均非有效测量，终判以干净重跑为准。

四笔草案终态（03:38 处置回执，详见 `2026-11-07-fbq-thirddelta-dddt-regression-verifier-alert.md` owner 回执）：

1. **允许性预检——撤销**：wrap 位 `!_zwJusIsAllowedChild(v, tag)` → return false。核验方 B2-vs-NEW 对账铁证：Chrome 对 `<div>[foobar]</div>`+`<dd>` 真换块（录音 `<dl><dd>[foobar]</dd></dl>`，`formatblock.js:757-760`，fixDisallowedAncestors 抬升），预检在 host 位拒绝 → 20 行（dd/dt × 4 形 × 2 dps 的 compare + qV-after）Chrome-matching 行为回归；其理由案（岛形）爬升终点均非 host、拒绝路径唯一可达形即 host 位 dd/dt——**20 行回归 / 0 行收益**。
2. **`_zwJusWrap` 第 4 参 `plainSkipViaHtml`**（part01c.js 参数 + part01g 调用传 true）——保留：ViaHtml 臂分派门加 `!plainSkipViaHtml`。裸 text 成员父 = editing host 时 innerHTML 整段重写使 formatBlock 多成员循环未处理成员游离，复制倒序产出三重 p 树（legs3 b5 detail 铁证）。仅 fb 调用传 true，justifycenter 等既有调用零扰动。
3. **同 tag skip 祖先链爬升列表透明**（part01g.js :145 环）——保留：爬升穿 ol/ul/li 不判（弃 prohibited-paragraph veto，li ∈ prohibited 名单使 :1784 外层 div 被误 veto 不剔除 → 误 wrap 出 `<li><div>foo</div></li>`）。legs3 未含此笔，ol-li 红行非其证伪；恢复后以 O-olli-div 探针实证。
4. **`_zwFbqVisBlock` 去 veto**（part01g.js）——保留：qV 读面命中块名只判 editable，`:1784` qV-before 录音 "div"（div 含 li 仍返名）。legs3 未含此笔，回归风险由语料腿 judge 对 pre 直测。
5. **批脚本 fb 腿改单页跑法**（batch-fbq-legs.sh）：part01e 堆债修复后单页可完整出账，免分片跑法 dps 粘滞链断裂伪影（4 qV(dps) before 假 P2F——窗界对齐即消，核验方案号归因 180/45/229/318）与 b7 覆盖洞；rc≠0 或 rows<5000 回退 9 分片（b0..b8 含尾片）。

## mt P2F=64 族（四命令 apply 激活引入，part06 preservesOverrides 修复）

legs3 mt 首次完整出账（30min rc=2，19,319P/818F——此前「mt ~12min 死亡」预测证伪，单进程 mt 方案可行）：judge vs nq7 基线 rows 10092 恒一，F2P=71（48 compare + 23 qV-fb-after，四命令 apply 首次真实命中录音）/ **P2F=64 单一家族** / lost=0。

64 行全部为 `[样式命令(b/i/strike/sub/sup/underline/fontname/fontsize/fore/back/hilite), 四命令(hr×22/img×22/fb×10/indent×10)(, inserttext)?]` 链的 `qS/qV(样式命令) after` 行——同案 compare/ret 行零翻红，树与返值全对唯样式读数失真。机制：四命令 spec 均 preservesOverrides（vendored :6577/:7052/:7141/:7316 各 action 头），但 apply 变更选区（wrap/hr/img/indent 重锚）后 override 惰性 staleness 判定把 stored range 快照判陈旧 → 下次 qV/qS 读误清 override（nq7 时代零应用选区不动 → 巧合绿；apply 激活后重锚 → 翻红）。**修正本证据文档早前「caret 偏尾挂账（刀前已红非本刀回归）」定性：mt 64 行是本刀激活引入的 P2F**；G-fb-plain caret 偏尾探针形（wrap 后 inserttext 落位）为同族另一面，另行挂账。

修复（并行轮，part06.js +14）：四 apply 臂 preservesOverrides 快照包裹——动作前 `_zwVaOverridesTakeRaw()` 快照、落定后 `_zwVaOverridesPutRaw()` + `_zwVaReanchorOverrides()` 重锚 stored range（F6 delete 刀 `__zw_ce_delete_apply` 先例模式，override 存储本身 document 级不动）。final2 批（03:28 二进制含此修复）mt 腿为验证跑。

## 第七刀（宿主内容模型守卫）+ 第八刀（table qV 隐形）——fb 腿 P2F=0 定谳（2026-11-07 04:21-04:30，legs5）

final2（03:28 二进制）fb 分片拼接 P2F=13 后归因五面：岛族 9 行（3 形 × compare/ret/qV-after）、table qV-before 2 行、267 双 p 1 行（第五刀死接线）、窗界伪影 4 行（分片跑法）、其余为 pre 同红 F2F。

**岛族 oracle 翻转**（owner「岛形全零应用」前判作废）：语料 :1879/:1884 div 岛 + p/div 目标录音**真 wrap ret=[true]**（注释 "<div> can have <p>, so, wrap it with <p>"）——判别子非「岛」而是**选区 editing host 的内容模型**：p/span 宿主不容块级子（`"<p> cannot have <div>, so, do nothing"` :1876 / `"<span> cannot have <p>"` :1866/:1871 → ret=false 树不变 qV 前后 ""）；div 宿主可容 → 照 wrap ret=true。严格 HTML 内容模型表不适用：`<div>[foobar]</div>`+dd（:757）真 wrap 后 fixDisallowedAncestors 抬 dl，dd∉div 内容模型仍需放行；b/i/font 样式壳不触发（壳非 host，宿主为外层块容器——mt `[["bold",""],["formatblock","<div>"]]` compare 全绿面）。

实装（part01g.js）：可编辑链守卫爬升捕获 host（`fbqHost`），host=p 或 computed display inline 族 → apply 整体 return false（无任何变异，qV 读面 VisBlock 遇 host 先返 null → 前后 ""）。**第八刀**（同文件 `_zwFbqVisBlock`）：爬升遇 td/th/tr/table/tbody/thead/tfoot 返 null——`<div><table><tr><td>[foo]</table></div>` qV-before 录音 ""（:1794/:1799），穿 table 落外层 div 曾误返 "div"；after 面因 td 内已 wrap 出 div 先命中不受扰。另回填 part01c `plainSkipViaHtml` 第 4 参（并行轮 03:28:20 checkout 抹除致第五刀失活，:1410 双 p 形 ViaHtml 整段重写游离复制倒序）。

**legs5 fb 单页判读（04:30，rc 门修复后单页完整出账）**：rows 5046 恒一，**P 4053→4894 / F 993→152，judge F2P=841（历史最优：B1 800 / legs3 794 / final2 820）/ P2F=0 / post-only 0 / lost 0 → VERDICT OK**。probe 14 案（zz-probe-owner.html，legs5 二进制）实证：table qV-before "" ✓、双 p 树不变 rets=[true] ✓、b 壳 + fb 不触发守卫 ✓、hr/img qS-bold carry ✓（part06 typing 半边）、spanblk wrap=录音 ✓、dl dt 换块 ✓、olbare 列表屏障 wrap ✓。

mt 面（final2 终局 04:07）：P2F=70 = formatblock 0 + indent 0（part06 快照包裹 fb/indent 臂兑现）+ ihr 12 + img 12（typing carrier 面）+ delete+inserttext compare 46（part06 旧无门 Take 泄漏 + typing 冻结，核验方 04:12/04:20 独立侦别）。修法两半收 `cmdIsFbq4` 门（overrides 半边 owner 04:10 / typing 半边核验方警示采纳），mt 由 batch-fbq-mt2.sh 单腿补跑（含门二进制）终判。

## mt2 终判（05:02 ALL-DONE，typing 门二进制）——VERDICT OK 全红线守

mt 单腿补跑（batch-fbq-mt2.sh，04:31:10 拿锁，head=08c7ea901）：shim md5=59791ef5=现树，二进制特征串 cmdIsFbq4×4 / fbqTtoff×2 / fbqOvSnap×4 实证（binary mtime 04:24:52 晚于 04:23:43 门笔收编，无混合窗口）；legs5 fb 账携带性经核验方 mtime 链 + 双 p compare 族 44 行翻绿独立实证。judge vs nq7 基线（batch-nq7-mt-post.log，**rows 10092→10092 键恒一**）：**P 9676→9769 / F 416→323，F2P=93 / P2F=0 / post-only 0 / lost 0 → VERDICT OK**（原始行 19,485P/646F，rc=2 语料完跑）。

行级对账（fbq-mt.final2-keep.txt 70 键直查 mt2 log）：

- **final2 P2F=70 全数翻回 Pass（70/70 零残余）**，族构成与 final2 分解逐族吻合：typing carrier 24 + delete+inserttext 43 + styleWithCSS 3——overrides 半边（fbqOvSnap null 保持式门自动限定）与 typing 半边（fbqTt/fbqTtoff 同门）各自兑现预期。
- **F2P=93 全落四命令族**：formatblock 35 + indent 12 + inserthorizontalrule 23 + insertimage 23，零 off-family 转化——apply 刀为唯一成因；F 母项自洽（416−93=323）。
- F2P 166→93 差额 73 行 = final2 旧无门版「冻结碰巧对齐」假转化回真红态（核验方预测 ~55、实际 73，F2F 非回归；P2F=0 为充要佐证，mt2 失败键 323 = final2 250 真债 + 73 回真红）。

enginelib2 补跑（05:05，test-guard 包裹，现树含第七/八刀 + 门笔）：**3,783P→3,787P/9F 九红名册恒一**（ENGINELIB2 ROSTER OK，final2 旧账 03:37 早于第七/八刀的账缺口闭合）。

腿账缺口显式处置：legs5 在 fb judge（04:29:50）后退出，indent/hr/img 语料单页腿未在现树重跑——final2 三腿账复用，可移植性论证（owner + 核验方双确认）：apply 臂自首刀后未动、dps setup 链差在 final2 三腿账 P2F=0 无伪影行；mt2 multitest 覆盖四命令全族终判。

## 结论

四命令 apply 面落地 + formatBlock qV 读面接通 + cmdIsFbq4 门收臂，终判批全绿收口：fb legs5 **F2P=841/P2F=0/lost 0**（历史最优，VERDICT OK）、mt2 **F2P=93/P2F=0/postF 0/lost 0**（rows 10092 恒一，final2 P2F=70 回归全数闭合 70/70、F2P 93 零 off-family）、enginelib **3,787P/9F 九红恒一**。四独立语料净减红 1,086 行（1,329F→287F）+ mt 面 F 416→323（nq7 基线口径）。挂账四族（不阻塞本批红线）：qV/qI 读面 Chrome delta（12 qV-before 行，跨边界选区录音 indeterm=true+value="" 签名）、apply 内容模型边界（4 行，p→div/span→p do-nothing 形）、typing 链 caret 偏尾、splitParent plain 域拷贝行为面（并行轮防御刀挂账）。
