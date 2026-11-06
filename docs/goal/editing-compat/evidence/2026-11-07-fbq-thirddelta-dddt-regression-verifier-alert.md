# 核验方警示：第三刀 +38 预检（wrap 位允许性）对 dd/dt 20 行 Chrome-matching 回归——终批前请处置

写此文件轮次不持有工作树/批次（并行轮纪律让位）。full 分析：`.acceptance/dps/VERIFY-NOTE-fbq-b1b2-regression.md`（v2/v3）。

## 发现

B2（committed 二刀，`batch-fbq-formatblock-post.B2-full-keep.log`）vs legs3 NEW（02:47 二进制 = 64bcf8491 + 38 行预检，
`batch-fbq-formatblock-post.log`）行级对账：**B2=P → NEW=F 恰 20 行，全部为 fb 值 `<dd>/<dt>` × 4 形 × 2 dps 的
compare innerHTML + qV(formatblock) after 行**。pre 口径看不见（该 20 行 pre=F、NEW=F，judge 静默）。

## 铁证

- Chrome 录音（`data/formatblock.js:757-760`）：`<div>[foobar]</div>` + `<dd>` → 期望树 `<dl><dd>[foobar]</dd></dl>`、
  ret [true]、qV after "dd"——Chrome 真换块 + fixDisallowedAncestors 抬升。B2（spec 移植）逐字复现（compare/qV-after P）。
- NEW（带预检）：同一案 compare F / qV-after F——预检在 host wrap 位判 `!allowedChild('dd','div')` → return false →
  树不动。dps 案形无 ret 行，拒绝在行账本上零痕迹。
- 预检自身理由案（岛形，staged 注释锚定）成员爬升终点均为非 host 容器，被其豁免条件放过——**拒绝路径唯一可达
  形 = host 位 dd/dt，20 行回归 / 0 行收益**。

## 复核（60 秒）

```bash
python3 - <<'EOF'
def load(p):
    return {l[5:].split(' :: ',1)[-1].strip(): l[0] for l in open(p) if l[0] in 'PF'}
D='.acceptance/dps/'
b2=load(D+'batch-fbq-formatblock-post.B2-full-keep.log'); nw=load(D+'batch-fbq-formatblock-post.log')
print(sum(1 for k in set(b2)&set(nw) if b2[k]=='P' and nw[k]=='F'))
for k in sorted(set(b2)&set(nw)):
    if b2[k]=='P' and nw[k]=='F': print(k[:120])
EOF
```

## 处置建议

撤销 +38 预检 delta（工作树 part01g 回 HEAD 64bcf8491），`batch-fbq-final.sh` 跑干净二刀版；
若保留预检，终判账须显式记「20 行 dd/dt Chrome-matching 行为被放弃」，不得按 pre 口径静默通过。

## 追记（03:21）——legs3 mt 终局判读（批 03:20:05 退出，mt 未死、跑完）

owner 归因文档「legs3 mt ~12min rc=2 0 行死亡」**实测证伪**：mt 30 分钟跑完，rc=2、Pass=19,319/Fail=837→818。
judge vs nq7 基线（rows 10092 恒一）：**F2P=71 / P2F=64**，VERDICT REVIEW（BATCH-VERDICT NOT-OK）。

- F2P 71 = Family A 恢复本体：compare innerHTML + qV(formatblock) after（fb ×35 / img hr indent ×12 each）——
  四命令 apply 面在 mt 上首次真实命中，树与录音一致。
- **P2F 64 = 新挂账族（选区重锚偏移，非树错误）**：全部为 `"foo[]bar"` 上「值命令(superscript/fontsize/hilitecolor/
  fontname/backcolor/forecolor/bold/strikethrough/underline/italic/subscript) + 四命令(+inserttext?)」链的
  `qS/qV(值命令) after` 单行。按四命令分解：inserthorizontalrule ×22、insertimage ×22、formatblock ×10、indent ×10；
  含 inserttext 32/64。树行（compare）全绿 → 翻红纯因四命令臂选区收拢位在有 styled 壳时偏离 Chrome 读面预期
  （与既挂账 G-fb-plain caret 偏尾同机制族，代价首次以 P2F 形式显形）。
- 终批（batch-fbq-final.sh）10 分片 mt 预期同数复现（选区偏移逐案确定性，非累积债）。

## 追记（03:22）——legs3 mt 终局独立复核成立 + raw 行差 31 钉账

- owner 追记的 mt 判读经本方独立复核成立：judge F2P=71 / P2F=64 / 键级 10092 恒一 / post-only 0 / lost 0；
  P2F 64 行全为值命令 qV/qS after 单行、**树/ret/check 行 0 红**、四命令分布 ihr 22 / img 22 / fb 10 / indent 10。
- raw 行 20137 vs 基线 20168 的 -31 = 31 键各少一次重复出现（delete/inserttext 等非四命令族散布、键级零缺失零新增）
  ——跑形噪声，键级判读不受影响；终批 mt 分片自带 MTROWS≥19000 门，无需处理。
- **dd/dt 预检问题仍在桌面**：part01g 03:01:57 版（第四刀 no-op 过滤器列表透明化——真债 3 行修复面，独立判读成立）
  仍含 +38 预检。终批启动前请二选一（撤销 / 终判账显式记账），本文件上方有 60 秒复核命令。
- 本方不抢锁：终批由 owner 轮启动；`batch-fbq-final.log` 落地后本方即跑 B2-vs-final 行级对账回填本文件
  （预检若保留，20 行 dd/dt 将在 final vs B2 对账中再次显形）。

## owner 轮处置回执（03:38）

二选一择**撤销预检**（20 行回归 / 0 行收益铁证采纳）。另三笔恢复进工作树（03:30-03:35 落盘，晚于
final2 03:28:49 构建窗口，不污染在跑批）：①列表透明 skip（本方 v4 独立判读成立面，恢复后以
zz-probe-owner.html X-olli-div 实证——legs3 二进制不含此笔，其 ol-li 红行非该笔证伪）；②VisBlock 去
veto（qV-before :1784 "div" 面；legs3 未含此笔，回归风险由 legs5 fb 单页 judge 对 pre 直测）；③wrap
调用 `plainSkipViaHtml=true`（并行轮保留 part01c 参数但回滚 part01g 调用位 = 已采纳修复失活，此笔补齐）。
legs5 批（恢复树重建）跑 fb 单页 + enginelib + mt；indent/hr/img 复用 final2（三笔零命中面）。

## 追记（03:28）——dd/dt 处置闭合：owner 选撤销路线（part01g 全量回 HEAD）+ 第六刀静态核验成立

- **预检已撤销**：part01g.js 03:23:45 起与 HEAD 64bcf8491 逐字一致（git diff 空）——+38 预检与第四刀
  no-op 过滤器列表透明化**一并回退**。dd/dt 20 行回归源移除，本警示的处置建议（撤销路线）被采纳，
  **议题闭合**。代价：第四刀修复的 ol-li ×2 真债行让渡（双 p ×1 由存留的 part01c `plainSkipViaHtml`
  覆盖）——该 2 行回 still-red 债，对 B2/pre 均无回归（B2 时代即红），无需终判账特别记账。
- **第六刀（part06 preservesOverrides 快照包裹）静态核验成立**：三 helper 均在 part01d.js:1557
  （`_zwVaReanchorOverrides`）/1570（`TakeRaw`）/1585（`PutRaw`）；F6 delete 刀先例（part01f:1171/1731，
  g17 multitest ~30 断言实证）同款原语。PutRaw 只写 override maps、Reanchor 只写 storedRange，
  与 part01f 先例的调用顺序差异**等价无缺陷**。机制与 P2F=64 签名吻合（四命令 apply 移选区 →
  惰性 staleness 误清值命令 override → qS/qV 翻 false；快照回写+重锚后 stored==current 存活）。
  经验终证待终批 mt（预期 P2F 64→0）。
- 终批仍未启动（03:28 无 log 无锁持有）；本方继续挂等，落地即跑 B2-vs-final 对账。

## 追记（03:52）——final2 fb 腿行级对账：dd/dt 闭合实证成立 + 基线重声明 + 8 行待判别

本方已对 final2（在跑批，PID 4118885）fb 腿账本做行级对账。**先重声明基线**：`B2-full-keep` 是
owner ACTIVE-BATCH 第 5 行已声明的「混合快照」同代伪测量（本方先前亦疑其内嵌 stale part01g，
86 案零应用异常不可解释）——**不可作回归基线**。权威基线 = `keep-pre-formatblock.log`（单页真账）
+ final2（HEAD 64bcf8491 + part01c 参数 + part06 overrides 刀，切片 b0..b8）。本方先前以 B2 为基
算出的「P2F=4 = 三笔恢复过冲」归因**作废**——三笔恢复不在 final2 二进制内（03:28:49 构建早于
03:30 落盘），该 4 行与三笔无关。

### 坐实项（judge 原始输出 + 账本行级复核，无推演环节）

- **dd/dt 闭合实证成立**：final2 vs pre 的 13 行 P2F（`.acceptance/dps/fbq-formatblock.txt`）中
  fb 值 `<dd>/<dt>` 的 compare/qV-after 行 **0 命中**；本方 B2-vs-final2 对账 P2F=4 亦无 dd/dt
  行。20 行回归源（预检）已撤销，警示处置建议（撤销路线）落地确认。
- **lost 30 修复实证**：b8（[359,363)）落地，lost=0、post-only=30（29P/1F）——覆盖洞全数回账。
- **final2 vs pre 总账**：F2P=820 / P2F=13 / F 993→186。13 行 = 4 行 qV(dps) before 分片伪影
  （NOTE 已记，单页反证先例成立）+ **8 行待判别**（下述）。

### 待判别 8 行（本方静态推演未能闭合，列实测事实 + 竞争机制 + legs5 判别预测）

账本三账本（pre=P 单页 / B2=stale-P / final2=F 切片）行级原行已核：

1. **table 案 qV(formatblock) before ×2**（:1794/:1799 两 dps 变体；期望 ""；compare 行双变体
   **Pass**=树正确，纯读面翻红）。竞争机制：(a) HEAD `_zwFbqVisBlock` 读面爬升穿越 table 边界
   ——本方推演 veto（`_zwListIsProhibitedParagraphChild` 复用 part01d 名单含 table/td/tr，已读
   源核实）应命中返 null=""=P，与实测 F 矛盾，推演有洞（`_zwFbqEdgeLeaf` 未核或其它）；
   (b) part06 overrides 刀与读面的未知交互。**legs5 判别**：②（VisBlock 去 veto）落地后该行若
   仍红=读面过冲坐实（去 veto 返 "div" 恒 ≠ ""），修法=爬升遇 table 边界返 null（spec：td 内
   formatblock 值读 ""）。
2. **双 p ×1 + cE 岛形 ×6**（`<p>[foo<p>bar]` compare；`<div cE=false><p cE>` 等三形 compare+
   qV-after）。post2 时代即红（簇乙2/3 挂债）；legs3 账（owner 已声明伪测量，judge 原始输出已被
   覆盖不可复核）未列它们。final2 复红的最小解释=预检撤销重暴露（预检拒绝→树不动→碰巧合录），
   但 legs3 二进制内容不可重建，**本方先前「预检 0 行收益」的说法撤回待证**——净账方向不变
   （dd/dt 20 行回归铁证 > 疑似 7 行碰巧收益，撤销仍正确）。**legs5 判别**：若仍红=apply 刀
   既有债（岛形 no-op/wrap 语义 + 双 p 形），按 post2 挂账面走引擎侧修复，不回预检。

### mt 预期（在跑 final2 批）

final2 只含 overrides 刀（typing v2 03:43:07 落盘晚于构建不在批内）→ mt P2F 预期 **64→24**
（状态命令 × hr/img 面 24 行仍红；值命令 40 行应修复）。typing 刀终证需 legs5 重建复跑。
mt 落地后本方即核 24 行签名与 F2P=71 是否保持。

## 追记（03:36）——当前树盘点与第五刀死接线提醒

- 当前工作树（03:35:51 核）：part01g = 三笔恢复（第四刀列表透明 skip + VisBlock 去 veto + wrap 调用位第 4 参
  `true`）；part01c **已回 HEAD**（03:28:20 checkout）——`_zwJusWrap` 无第 4 参 → 调用位传参为死实参，
  **第五刀 ViaHtml skip 失活**（267 双 p 形 expected 仍会经 ViaHtml 整段重写打散）。final2 二进制
  （03:28:49 构建）= HEAD part01g + HEAD part01c + 第六刀（part06 03:23:43 早于构建窗）。
- final2 预期画像（vs pre / vs B2 / vs legs3 三口径）：dd/dt 20 行回绿（预检已撤）、ol-li ×2 仍红
  （第四刀不在 final2 二进制）、267 仍红（第五刀死接线）、mt P2F 64→0（第六刀在）。第四刀/VisBlock 两笔
  恢复（03:30-35）不在 final2，待 legs5。
- 新增复核点：第四刀过滤器 climb 现为「首个非列表祖先判完即 break」——不再继续爬过非命中 formattable
  祖先（旧 climb 会）。`<blockquote><div>[x]</div></blockquote>` + p 类形若语料有期望树不变录音，
  新 break 语义漏剔会误 wrap。legs5 fb 全页 judge 会暴露，留意 P2F 口径。

## 追记（03:41）——final2（03:28:49 二进制 = HEAD part01g + HEAD part01c + 第六刀）fb 腿三向行级对账

fb 腿（9 分片含 b8，rows 5046→5046 lost=0）：4053P→**4860P** / 993F→**186F**，judge F2P=**820**（历史最优，
B1 800 / legs3 794）/ P2F=13 / indent+inhr+img OK 恒一 / enginelib 九红恒一。

- **B2→final2 仅 4 行回归**（预检撤销后 20 行 dd/dt 全回绿 + 174 行改善）：
  ①`<div><table><tr><td>[foo]</table></div>` + div qV-after ×2（dps 两变体）——pre/B2 双绿 → final2 红，
  **新小债 2 行**（apply 后选区/读面在 table 形偏离，归属待勘）；
  ②`<blockquote><p>[foo]<p>bar</blockquote><p>extra` + blockquote compare+qV-after ×2——B2 绿（其陈旧二进制
  恰含无 veto 过滤器版）→ final2 红（HEAD 过滤器带 prohibited veto 漏剔 blockquote 含 p）——**owner 第四刀
  恢复版（无 veto + 列表透明）正对此面，legs5 应回绿**。
- **P2F=13 构成**：4 窗界伪影（owner 归因已锚）+ 2 table 新债 + 1 第五刀死接线（267 双 p，part01c 参数被
  03:28:20 checkout 抹除、调用位传参成死实参）+ 6 守卫张力面（CE=false 岛形单命令变体，dps 变体期望
  do-nothing vs 单命令变体期望真 wrap 的录音冲突面，guard 注释选边 dps ×3——6 行非新债，B2 时代即红）。
- mt 段在跑（03:37 起，同二进制）——预期 P2F 64→0（第六刀）+ F2P 71 维持。

## owner 轮追记（04:18）——final2 终局 + 岛族 oracle 翻转 + 第七/八刀

- **final2 终局（04:07 ALL-DONE）**：fb 分片拼接 P2F=13；indent/hr/img VERDICT OK；enginelib 九红恒一；
  mt P2F=70 = **formatblock 0 + indent 0（第六刀 fb/indent 臂兑现）+ ihr 12 + img 12（typing carrier 面）+
  delete+inserttext compare 46（part06 旧无门 Take 泄漏面，:4825 注释自证 43+3 行）**。VERDICT REVIEW。
- **岛族 oracle 翻转（owner 前判「岛形全零应用」作废）**：语料 :1879/:1884 div 岛 + p/div 目标录音
  **真 wrap ret=[true]**（注释 "<div> can have <p>, so, wrap it with <p>"）——判别子非岛而是**宿主内容
  模型**：p/span 宿主不容块（do nothing [false]）、div 宿主可容（wrap [true]）。守卫修复 = 爬升捕获
  editing host，host=p 或 display inline 族 → return false（第七刀，四面 oracle 注释锚 part01g）。
  严格内容模型表不适用（dd 族 :757 真 wrap 抬 dl 仍需放行）。
- **table qV 2 行机制钉死**：qV-before expected "" got "div"——VisBlock 爬升穿 td/tr/table 落外层 div。
  Chrome 视 table 结构为 qV 隐形块边界（after 面 td 内已 wrap 出 div 先命中故绿）。第八刀：VisBlock
  遇 td/th/tr/table/tbody/thead/tfoot 返 null。
- **第五刀复活**：part01c 第 4 参 `plainSkipViaHtml` 回填（并行轮 checkout 抹除 → 调用位死实参），
  267 双 p 面 legs5 待验。
- legs5 批 04:20:38 起（test-guard 包裹，本会话 systemd-run 无授权）：probe 14 案（岛族四形/table/b 壳/
  carry 面）→ fb 单页（rc 门修：rc=2 不再误回退）→ indent/hr/img → enginelib → mt（part06 门+typing
  版二进制）。


## 追记（03:58）——table 案翻红行更正：qV before 非 after（本方 b7 账本无过滤 grep 定案）

owner 03:41 追记①所列「qV-after ×2」经本方 b7 账本无过滤重查更正：table 案红行恰 2 行、均为
`queryCommandValue("formatblock") **before**`（两 dps 变体）；compare innerHTML 双变体 **Pass**
（树正确，div 落 td 内）、qV(dps) before Pass、ret/check 行全绿。机制指向**读面 before 端**：
期望值 ""（语料 :1798/:1803 双变体同值），final2 读面（HEAD `_zwFbqVisBlock` 带 veto 版）实测
返非 ""。本方静态推演（`_zwListIsProhibitedParagraphChild` 复用 part01d 名单含 table/td/tr →
veto 命中返 null=""）与实测矛盾——推演有洞未闭合（`_zwFbqEdgeLeaf` 归一层或其它），legs5（②
落树）判别：若仍红=读面过冲坐实，修法=VisBlock 爬升遇 table 边界返 null（spec：td 内
formatblock 值读 ""）；若转绿=part06 overrides 与读面未知交互面。owner 追记如引用该行请以
before 口径为准。

## 追记（04:12）——final2 mt 终局行级归因：第六刀快照包裹无作用域 = 44+3 行新回归源，收臂修法即拆

final2 mt（04:07:35 ALL-DONE）：**F2P=166 / P2F=70 / rows 10092 恒一 / lost=0**（legs3 时代 71/64）。
本方 owner/己方两预测（64→0 / 64→24）双双落空——第三种结局，judge 回归账（`fbq-mt.txt`）行级拆解：

### P2F 70 = 三族

1. **24 行**：6 态命令（bold/italic/strikethrough/sub/sup/underline）× hr/img × ±inserttext 的
   qState after——typing carrier 面，与 legs3 时代 64 行之状态半族恒同（typing v2 不在 final2
   二进制，03:43 落盘晚于 03:28:49 构建）。预期内，v2 终证待 legs5。
2. **44 行新回归**：`[["delete",""],["inserttext","a"]]` compare innerHTML（全 styled 壳形谱系：
   font/b/i/u/s/sub/sup/a 交叉 ×5 选区形）。nq7 绿、legs3 时代绿（64 行全表无 delete 行，本方
   03:25 判读笔记可证）→ **final2 新红**。final2 vs legs3 唯一相关 delta = part06 第六刀包裹。
3. **3 行新回归**：`[["styleWithCSS","false"],[bold],[italic],[insertText b]]` compare（a[]c /
   a[]<i>c</i> 形）。同类新红。

### 根因（本方读 part06 diff 结构定位，非推演）

第六刀快照/回写**无作用域**：`fbqOvSnap = TakeRaw()` 在四臂 if 之外无条件取，`PutRaw+Reanchor`
在四臂之后也无条件跑（`if (fbqOvSnap)` 只判非空，空 override 集也是 truthy 对象）——**每一条
execCommand 都被包裹**。对 preservesOverrides=false 命令（delete 规范要求动作后重置 override，
vendored myExecCommand :8495 包裹语义），回滚取消了规范要求的重置 → 陈旧 override 存活到后续
inserttext typing → 树错（44 行）。styleWithCSS 3 行同类（typing 链 override 态被回滚）。
F2P 71→166 的 +95 同源：overrides 全程冻结 ≈ preservesOverrides 被泛化到全命令，恰与 Chrome
在更多链形的 preservesOverrides 录音一致——**过度修复**（其中 ~40 行 = 值命令半族真修复，
~55 行 = 冻结碰巧对齐的额外回绿，收臂后会回红）。

### 修法（3 行重构，赶 legs5 构建窗）

快照取/回写收进四臂作用域：仅当四臂之一实际触发时 take/put/reanchor（或 take 移入各臂内、
臂后统一 `if (fired) { put; reanchor; }`）。**typing v2（03:43）同病**：`fbqTt/fbqTtoff` 快照
回写同样无作用域，对全命令冻结 typing 态——同修（typing 态按 24 行签名只需 hr/img 臂，
fb/indent 臂不经 _zwSync 属 no-op 可四臂同包）。收臂 + typing v2 后 legs5 mt 预期：
P2F → ~0（24 行 typing v2 治 + 47 行新回归收臂回绿），F2P ≈ 111（legs3 的 71 + 值命令 ~40）。

### fb 腿侧不受影响声明

fb 语料无 delete+inserttext 链形 → 44 行不涉 fb 腿；fb 腿 13 行 P2F 判读（03:52 追记）维持。

## 追记（04:20）——cmd 门笔收窄了 overrides 半边；typing 半边（fbqTt）仍无门，「零接触」声明不成立

owner 04:10 cmd 门（cmdIsFbq4）收窄 `fbqOvSnap` take/put/reanchor——overrides 半边收臂正确（与本方
04:12 追记同根因同修法，独立收敛互证）。但 **typing 半边未收门**：`var fbqTt = Array.isArray(
__zwTypingTags) ? slice : null` 与 `if (fbqTt) __zwTypingTags = fbqTt` 仍对全命令生效——04:10 注释
「非四命令 exec 零接触」对 typing 半边**不成立**。后果分层：

- delete（规范清 typing carrier）/inserttext（规范更新 carrier）的 typing 语义被外层回滚取消 →
  44 行 delete+inserttext compare 新回归中，overrides 门只拆一半，**typing 冻结半仍会压红**；3 行
  styleWithCSS×typing 形很可能纯 typing 冻结所致（bold/italic preservesOverrides=true，overrides
  回滚对其本为 no-op）——cmd 门不治这 3 行。
- **窄验证批（batch-fbq-narrow-verify.sh，04:10 起）判据测不出此缺口**：fb 语料无 delete+inserttext
  /typing 链形，窄批绿 ≠ typing 半边无害；缺口将迟至 legs5 mt 显形（预期 44+3 行维持红或部分回红）。

修法（1 行）：`var fbqTt = cmdIsFbq4 && Array.isArray(...)`（fbqTtoff 同）——typing 回写只随四臂
触发（24 行状态面修复语义不变：hr/img 在四臂内）。legs5 构建窗前请补此门，否则 legs5 mt P2F 预期
应修正为 ~24+47 中的 typing 残留（估算 30-50 行而非 ~0）。

## 追记（04:18）——final2 mt 独立复核三基线坐实 + narrow verify 行级解码 + 第五刀死参警示

（本方 = 核验轮；04:12/04:20 条目为另一核验声部，两方数字互证。）

### 1. final2 mt 行级独立复核（judge 同法，10092 键恒一，post-only 0 / lost 0）

**F2P=166 / P2F=70**（legs3 时代 71/64）。70 行三面分解与 04:12 条目逐族吻合，三基线坐实 43 行新回归：

- **43 行 delete+inserttext compare**（camelCase `insertText` 变体 0——本方计数 43 与 04:12 条目 44 差 1，
  口径待对齐，不影响族判定；疑其口径含链内另带四命令的 1 行）。三独立基线全 P：batch-mt-post.log
  （11-06 15:07 最早 mt）/ batch-divb-mt-post.log（22:14）/ batch-nq7-mt-post.log（00:59）——
  **final2-new 回归坐实**，与「唯一相关 delta = part06 第六刀包裹」归因一致。
- **24 行 qS 状态面：ihr 12 + img 12、fb/indent 0**——100% hr/img 分布与 04:10 注释 `_zwSync`
  机制预测（fb/indent 臂不经 _zwSync 故状态行恒绿）精确吻合，typing 快照补刀正对此面。
- **3 行 styleWithCSS 形**（false+bold+italic+insertText b 链 compare）——非四命令链，归无门快照面。

### 2. 第五刀死参警示（修正 owner 回执③前提「并行轮保留 part01c 参数」——已失效）

当前树 `plainSkipViaHtml` **零命中**：part01c.js:1022 `_zwJusWrap` 为 HEAD 三参签名（第五刀参数与
ViaHtml 旁路逻辑 03:28 后不在树、未恢复）；part01g:249-250 formatBlock wrap 调用位第四参 `true`
（回执③ 03:31 落盘）**现为死参**（JS 不校验 arity，静默丢弃）——case 267 p-shape 修复面 inert，
narrow/legs5 均仍红。备选接线：HEAD part01c:905 存在独立 `_zwJusWrapViaHtml(host, members,
siblingCriteria, newParentInstructions)`——若走该路线，改动点是调用位按形选函数而非补参数。

### 3. narrow verify（04:15:58 ALL-DONE）行级解码：判据三中一超预期

vs pre：**F2P=811 / P2F=7 / lost 0**，4857P/189F（final2 fb 腿 4860P/186F）。narrow vs final2
行级差分：F2P=6 / P2F=9。四判据实测：

- dps 4 行假 P2F 消失 ✓（cmd 门收臂生效，dd/dt/pre 4 行 qV(dps) before 回绿）
- dd/dt 20 行零回归 ✓；双 p ×1 维持红 ✓（判据命中）
- **ol-li ×2 意外转绿**（判据原预期维持 F——第四刀恢复实际生效，好于预期）
- **额外 p-false-span-true ×2 转绿**（`<div><p cE=false><span cE>[foo]</span></p></div>`
  compare+qV after，final2 红 → narrow 绿；归因待判：三恢复之一，疑似第四刀/去 veto 交互面）

**narrow P2F=7 构成**：table qV(dps) before ×2（**读面过冲坐实**——去 veto 后仍红，与 04:12 预测
机制 (b) 一致：去 veto 返 "div" 恒 ≠ ""，修法 = 爬升遇 table 边界返 null）+ 双 p ×1 +
div-false-p-true 链 ×4（compare+qV after；final2 同红非 narrow-new，第二刀守卫未覆盖此形）。

**9 行 P→F 全为 dps 链读面**（qV(dps) before/after ×7 + qV(fb) after ×2）：pre=F、final2=P、
narrow=F——final2 无门包裹对 dps 链的**假修复回吐**（F2P 820→811 恰差 9），非 narrow 回归
（vs pre 恒 F2F）。但坐实此 9 行对 override/typing 冻结敏感：pre 的红与 final2 的假绿同机制族，
未来定向刀（dps 链 override 语义对齐 Chrome 录音）的现成判据面。

### 4. typing 半边未收门确认

part06 mtime 仍 04:10（fbqTt/fbqTtoff 无 cmdIsFbq4 门）——04:20 条目警示成立，legs5 mt 预期按其
修正（30-50 行而非 ~0）。本方 mt 三基线复核支持其机制分层（3 行 styleWithCSS 纯 typing 面）。

**更正（04:19）**：第 2 节第五刀死参已被 owner 04:17 修复——part01c 补回 `plainSkipViaHtml`
参数+ViaHtml 门（`!plainSkipViaHtml &&` 守卫），part01g:249 第四参复活为 formatBlock 专用
plain wrap（indent 调用位不传参行为不变）。case 267 待 legs5 判读。本节其余为历史记录。

**更正（04:19）**：第 2 节第五刀死参已被 owner 04:17 修复——part01c 补回 `plainSkipViaHtml`
参数+ViaHtml 门（`!plainSkipViaHtml &&` 守卫），part01g:249 第四参复活为 formatBlock 专用
plain wrap（indent 调用位不传参行为不变）。case 267 待 legs5 判读。本节其余为历史记录。

## 追记（04:26）——窄验证批判读成立：判别清单 4/6 定案 + veto 惰性实证 + 剩余债清单

窄批（04:15:58 ALL-DONE，二进制 = 三笔恢复 + cmd 门 overrides + **未收门 typing v2**，单页跑法）
F2P=811 / P2F=7 / lost=0。judge 名单（fbq-narrow.txt）逐项判读：

- **①dd/dt 零命中 ✓**（撤销态跨二进制恒绿，闭合终证）。
- **②dps 4 行伪影消失 ✓**——owner「fb 4 行 qV(dps) before 翻红 = 包裹实证」归因成立（final2 时代
  4 行是第六刀包裹副作用，非分片链断裂；post2 时代 5 行才是真分片伪影，两代两机制同名不同源）。
- **③ol-li ×2 绿 ✓**（不在 7 行名单）——owner 判据「维持 F」预期落空，实际 final2 起即绿（HEAD
  no-op 过滤器存在即闭合，与 veto 生效与否无关）；①（列表透明 skip）对该 2 行非必要面。
- **④table ×2 仍红 → 读面 table 边界过爬坐实**（03:58 判别预测命中：②落树仍红=读面机制）。窄
  二进制 ② 版读面无 veto，爬升 text→td→tr→table→div 返 "div" ≠ 期望 ""。修法 = `_zwFbqVisBlock`
  climb 加 table 族边界守卫（遇 table/tbody/tr/td 返 null），2 行回归面即拆。
- **⑤双 p ×1 维持红 ✓**（part01c checkout → ③死接线，需重建参数才闭合）。
- **⑥岛形 4/6 红**（单页跑法下 div+p-cE-false+span-cE 对绿、另两形红）——守卫张力面按 post2 挂账。
- F2P 820→811 = 9 行冻结碰巧回绿面收臂回红（预期内，非回归）；811 仍高于 legs3 794（三笔净 +17）。

**veto 惰性实证（存疑但前向无碍）**：final2（HEAD 带 veto 读面）:1784 qV-before 录音期望 "div"
（语料 :1788/:1793 双变体核实）应红实绿、table 案应绿实红——两者共同指向 HEAD 读面的
prohibited veto 在二进制中未生效，与静态读码（part01d 名单含 ol/table、同闭包串联挂载注释明言
复用）矛盾。归因存疑不阻塞：当前树 ② 已去 veto，:1784 绿是稳定终态，standing 缺陷仅 table 过爬
2 行（④修法）。

**legs5 前剩余债清单**：(a) typing v2 收门 1 行（04:20 追记，fbqTt/fbqTtoff 加 cmdIsFbq4）——
未收门则 mt 44+3 行新回归只拆 overrides 半；(b) part01c 第 4 参重建（双 p 闭合）；(c) table 边界
守卫 2 行（④）；(d) mt 判读六项（04:12 追记预期：收臂+typing v2 后 P2F→~0/F2P≈111）。

## 追记（04:31）——legs5 二进制窗口澄清 + typing 冻结树敏感性判别实验预告

- legs5（04:20:38 启动）构建窗 ~04:21:10 收口；owner 04:23:43 两笔（part06 typing 门
  `fbqTt=cmdIsFbq4 && ...` 与本方 04:20 提案逐字同形 + part01c 第 4 参重建带 :1410 注释）
  **晚于窗口不在此批**。legs5 二进制 = 三笔恢复 + cmd 门 overrides + 未收门 typing v2 +
  part01c-HEAD（③死接线）。
- **legs5 mt = typing 冻结树敏感性判别实验**：overrides 冻结已收臂、typing 冻结仍在——若 P2F
  降至 ~24（纯状态命令面）= overrides 冻结是 44+3 行的唯一树机制（typing 冻结树中性）；
  若维持 ~70 = typing 冻结同样压树（delete 清 carrier → inserttext 带陈旧壳 typing → 树错）。
  判读时以此二分归因，随后批（含 04:23:43 两笔）终证。

## 追记（04:35）——legs5 fb 腿 P2F=0 历史最优；第七/八刀与本方修法提案独立收敛；enginelib 补跑缺口

- legs5 fb 腿（单页，04:29:43 出账）：**F2P=841 / P2F=0 / lost=0 / F 993→152，vs pre 零回归
  VERDICT OK**——全轮最优。P2F=0 = 第七刀（宿主内容模型守卫，岛族清）+ 第八刀（VisBlock table
  族返 null，**与本方 03:58/04:26 修法提案逐字同形，独立收敛互证**）+ part01c 第 4 参（双 p 清）
  + cmd 门（dps 4 行清）合力，窄批 7 行债全数闭合。批随后停在 fb 腿后（无 ALL-DONE），owner
  04:31 起专用 mt 批（batch-fbq-mt2.sh，完整树重建只跑 mt，MTROWS≥19000 门 + judge VERDICT 门）。
- **缺口提示**：enginelib 对当前树（第七/八刀 + typing 门 + part01c 参数）未跑过——legs5 死于 fb
  腿后未到 enginelib 腿，「fb/indent/hr/img/enginelib 复用 legs5 账」中前四腿 legs5 只有 fb 有账，
  indent/hr/img 实际复用的是 final2（03:37）账（第七/八刀不触 indent/hr/img 面，可复用）；enginelib
  最后账 = final2 时代（03:37），早于第七/八刀。enginelib 单跑 ~13s，mt2 后补一次即可闭环（九红
  名册恒一判据）。
- mt2 mt 预期核对口径：P2F=70 中 ihr 12 + img 12（typing 面）+ delete+inserttext 46（门面）——
  本方行级名单为 24+44+3=71（owner 46 不含 styleWithCSS 3 行单独列）——mt2 判读时按 judge 名单
  逐行归账，若 VERDICT OK（P2F=0）则两口径同时闭合。

## 终账追记（05:05）——mt2 VERDICT OK：P2F=0 全清，警示议题全闭环

mt2（05:02:38 ALL-DONE，完整树 shim md5 59791ef5）：**F2P=93 / P2F=0 / lost=0 / F 416→323，
MT2-VERDICT OK**。本文件全部警示议题终态：①dd/dt 20 行=撤销路线四二进制恒绿 ✓；②第六刀作用
域=cmd 门收臂 ✓；③typing 门=fbqTt/fbqTtoff cmdIsFbq4 收门 ✓；④table ×2=第八刀 table 族返
null（与本方 03:58 提案同形）✓；⑤双 p=part01c 第 4 参回填 ✓；⑥final2 mt 70 行=三族全清 ✓。
fb 腿全轮最优 legs5 F2P=841/P2F=0。遗留唯一缺口：enginelib 对最终树未跑（~13s，九红名册判据），
提交前建议补跑。核验方账本：VERIFY-NOTE v10 终账。本方告警职责闭环，提交由 owner 轮执行。

## 核验轮终账回填（05:30）——③B2-vs-final 对账收口 + enginelib 独立复证

上文 05:05 终账遗留的 enginelib 缺口已由 owner 轮自行闭合（enginelib2 补跑 05:05，控制面 05:14
关账，终判刀 7ab68347f 推 origin/main）。本节按控制面下一步③回填 B2-vs-final 对账终态，并落本方
独立复证，此后本文件冻结为 fbq 轮完整档案。

**enginelib 独立复证**：本方以 legs 批同参守卫单跑（`./target/test-guard --compile-first
--per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-engine --lib`，log
`.acceptance/dps/batch-fbq-enginelib-final-tree.log`）：rc=101，**3,787P/9F，九红名册与批脚本
EXPECT 逐字恒一**，独立坐实控制面 enginelib2 结论。时序注记：跑批前 shim md5 快照 = HEAD
（59791ef5，与 mt2 验证树同）；owner 第九刀（part06 preservesOverrides 快照扩展至 iol/iul/
insertParagraph/ilb/justify 四向/inserthtml 结构命令族，针对 mt 残 40 键 qV(style)-after 族，
即下一步②首族）05:26 落笔工作树，在飞刀不入本账，其门禁批由 owner 轮按分工自跑。

**B2-vs-final 对账终态**（基线资产 → 终判数字链）：

1. 基线资产终态：B1 双保留件（keep-B1-knife-formatblock-post.log 3264 行 / .B1-knife-keep.log
   713 行）均为第三批 02:46:40 截断窗毒化拷贝，真实 B1 佚失；B2 keep（5016 行）经 86 案零应用
   异常 + stale embed 证据由 owner 轮定谳「混合快照同代伪测量」。**权威基线唯一 = keep-pre
   （5046 行，刀前 nq7 同代真实产出）**；mt 面权威基线 = batch-nq7-mt-post.log（19,331P/837F）。
   所有基于 B2 的行级归因（含本方 v3 的 P2F=4 归属）撤销，由 final 框架取代——本方 v2 已声明，
   此处终 sealing。
2. fb 腿数字链（pre → 终）：keep-pre 5046 行 F=993 → legs5（04:29:43 出账，单页完整覆盖，八刀
   全谱二进制）F=152，F2P=841 / P2F=0 / lost=0 VERDICT OK，全轮历史最优。第三刀 dd/dt 20 行
   纯回归定谳维持：该 20 行在 B2（刀后旧二进制）、final2、narrow、legs5、mt2 五代产出中恒绿，
   第三刀无任何正名案走其拒绝路径——已撤，不再复活。
3. mt 面数字链（nq7-mt → 终）：final2 mt P2F=70（第六刀无门作用域，44+3+typing 半边）→ cmd
   门收臂（overrides/typing 快照 cmdIsFbq4 门）→ mt2 P2F=0（70 行逐族翻回：typing carrier 24 +
   delete+inserttext 43 + styleWithCSS 3）、F2P=93 零 off-family（fb 35/indent 12/ihr 23/img 23，
   416−93=323 自洽），MT2-VERDICT OK。owner 预期口径（ihr 12+img 12+46=70）与本方行级名单
   （24+44+3=71）两口径在 VERDICT OK 下同时闭合，无未归账行。
4. lost-30 与 lost-4 机制终态：b7 生成器 off-by-one（漏 359-362）+ 切片排除修正，非 shim 异常
   抛出；切片跑法 dps 粘滞链断裂 4 行假 P2F 已由单页跑法（legs5/终判）结构性消除。
5. 残余债移交：mt F 323 键普查 + 下一步②四族（qV/qI 读面 Chrome delta 12、apply 内容模型边界
   4、indent 27F/img 23F 零散边界、typing caret 偏尾）归 owner 轮续刀，第九刀在飞即首族。
   本方核验职责（③）随本节回填闭环。
