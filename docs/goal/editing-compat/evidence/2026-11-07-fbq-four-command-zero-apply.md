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

G-typing 三连（bold → formatblock `<div>` → inserttext 'a'）：树与 qV 正确，但 'a' 落在文本尾（`<div>foobara</div>`@7）而非 typing 壳内（期望 `<div>foo<b>a[]</b>bar</div>`）。定位：fb_apply 自身落位正确（sel=text@3），偏移发生在 inserttext 经 `__zw_ce_caret_range` 的世界同步/两级重锚对 wrap 后树的落位。该行为刀前已红（Family A 行本就是全红行），非本刀回归；typing 链挂账收口另立项。

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
| mt | <!-- MT-RESULT --> | |

四语料净减红 1,086 行（1,329F→287F）。mt 语料 formatblock 值分布核实：全 24 处均为 `<div>`（spec 12 名内）——扩 8 名对 mt 零行为影响，mt 批即终判数据。批脚本 `.acceptance/dps/batch-fbq-legs.sh`（head=5d26dc17b 工作树含刀，二进制 marker `fb_apply=4`；与 commit 6afc857bf 内容一致）。

indent 剩 27F 零散边界族（不可编辑 span 边界 `{ <span contenteditable=false>A…}`、`<p>[abc]</p>` 全选 wrap、br 交错 `foo[bar]<br>baz`、blockquote 相邻聚合、table data-start）——非 CSS margin-div 主族，后续小刀多批。img 剩 23F 零散边界族（`foo{<span>bar</span>}baz` inline 全选、跨 color span 边界、dps p + 跨块 style 形）。

## 结论

<!-- CONCLUSION -->
