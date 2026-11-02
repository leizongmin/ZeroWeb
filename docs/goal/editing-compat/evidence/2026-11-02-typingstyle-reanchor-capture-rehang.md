# TypingStyle 轮：reanchor 层位修正 + 幸存壳 typing 链捕获 + F6 嵌套尾段重挂

日期：2026-11-02（editing 流，控制面下一步 ①，g19 后挂账 7 案收尾 + F6 FIXME 闭环）

## 0. 范围

g19 multitest P→F 7 案（CE-false span ×4、全段 `<b><i>` 双壳 ×2、styleWithCSS-ipara
qS ×1）+ F6 嵌套尾段 FIXME（D4 轮遗留）一次性归因与修复。探针：
`zz-probe-tstyle.html`（T1-T7 + T1MID/T1B/T1C）、`zz-probe-tstylediag.html`
（D1-D4 raw/caretRange 对账 + kill-switch）、`zz-probe-d4br.html`（D4 六 fixture）。

## 1. 归因

### 1.1 CE-false span ×4 → reanchor top===el 层位位移（Fix A）

zz-probe-tstylediag 实证：delete 后内部 raw selection 已正确落 `(b,0)`，
`__zw_ce_caret_range` 却返 `(SPAN,0)`（span = b 的 contenteditable=false 子）。
`ZW_SEL_PATH_MUTATION=0` kill-switch 下 caretRange 正确 → 定位 reanchor 路径。

机制：delete 动作管线把**宿主本体**登记进 `__zwSelPendingTops`（action 根形态），
`_zwCeChainTop` 命中登记表返 `top===el`、`path` 语义是 el→node 的全链索引；
`_zwCeRemapEndpoint` 旧码恒自 `kids[hit.regIdx]`（el 的下一层）起走 path，整条
链下移一层——`(b,0)` 被走成 `b.childNodes[0]`（CE-false span）@0。隐性存续原因：
错层下钻多数形态越界返 null（落到 reresolve 身份兜底），仅当目标层恰有该位子
元素时"成功"产出错点。

### 1.2 双壳 ×2 + ipara qS ×1 → delete 后 typing 态无载体（Fix B）

- `<div>abc<b><i>[def]</i></b>ghi</div>`+delete+inserttext：keep-shell delta ① 留
  最外层 b、内层 i 按空摘；caret 落 b 内，typed 段无 i 包裹得 `<b>d</b>`，期望
  `<b><i>def</i></b>`（Chrome recording :2521-:2536 四变体）。
- `<div><b>[abc]</b></div>`+delete+insertparagraph+inserttext（:2429）：caret 经
  ipara 拆分离壳入新块，typed 段应仍着 b（`<br><div><b>a</b></div>` 形）、
  qS(bold)=true；g19 后 `__zwTypingTags` 为空、caret 又不在 b 链上，qS 得 false。

Chrome 语义：typing style 跨 delete 存续（g18/g19 delta ④ 实证过显式 toggle 面），
作者 DOM 链形在删除后同样进入 typing 载体（recording 注释自认 "just based on
Chrome, some of them may not be reasonable"）。

### 1.3 F6 嵌套尾段 → offSplit 纯占位尾段整丢（Fix C）

`<div><b><i>abc[]<br></i><br></b></div>`+bold+inserttext 期望
`<div><b><i>abc</i></b><i>d<br></i><b><br></b></div>`（:2479，"second line text
should keep bold style"——末行占位 br 着 off 壳 b、typed 行终结 br 随 typed 段着
i）。D4 offSplit 尾段收集产出 `seqT=[<i><br_i></i>, br_b]` 后 tailHasText=false
走整丢臂，双占位形缺失。

## 2. 实现（part01.js 两笔 + part01f.js 两笔）

- **Fix A**（part01 `_zwCeRemapEndpoint`）：登记 top 为宿主本体时自 `el` 起走
  path（`cur = el` 进入既有下钻循环），其余形态照旧自 `kids[regIdx]` 起走。
- **Fix B**（part01f）：
  - 留壳预扫抽 `_zwDelKeepShellOf` 共享（deleteSelection 原位换调用，语义不变）；
  - 新 `__zw_ce_capture_typing_chain(ar)`：自 startContainer 上行至编辑宿主收
    整链；链上任一 `<a>` → 捕获空（:2163 `<a><u>` / :2168 `<u><a>` 双向实证
    typed 段裸奔）；否则内→外收集 format 族（part06 `_zwExecCmdFormatTag` 同表
    b/i/u/s/sub/sup），行至幸存壳（含）止（幸存壳之上删除时已脱离继承链）；
    过滤 D4 off 表；记录序外层在前；
  - `__zw_ce_delete_apply` 动作前捕获（仅非 collapsed），动作后显式 typing 态
    （delta ④ 回写）存活优先、态空且捕获非空时补位。
  - 消费侧（part01 新 `_zwCeTypingTagsAt`）：三处 ttags 读取（text splice /
    元素 target / 元素边界臂）滤除 caret 位祖先链（至宿主、含容器自身）既有
    tag——幸存壳由 DOM 自然承接，不双裹（T5 得 `<b><i>d</i></b>` 而非
    `<b><i><b>d</b></i></b>`；:2418 壳内 typing、:2173 sub 壳内 typing 各形同
    门）。显式 collapsed toggle 态不受影响（toggle-on 时 tag 必不在链上，滤除
    为恒等）。
- **Fix C**（part01 offSplit 纯占位尾段臂）：尾段无文本且含 ≥2 个 br（元素
  typed 段前提）→ 重挂：首个 br（文档序 = caret 行终结符）并入 typed 段壳，
  其余 br 各以 off 壳 clone 承接回挂 typed 段之后；单 br/空尾段维持丢弃 +
  off 壳直后裸 br 消费（F1-F5 形不变）。

## 3. 探针验证（零-wpt-runner release 重建后）

- zz-probe-tstyle：T1-T4 GOT `<b>Y<span contenteditable=false>abc</span></b>`
  `<b><span>abc</span>Y</b>` 双臂全翻转（typed 段入 b 非 CE-false span）；
  T5/T6 GOT `<div>abc<b><i>def</i></b>ghi</div>` 精确期望形；T7 GOT
  `<div><b></b></div><div><b>a</b><b></b></div>` qS_after=**true**（qS 断言翻
  转，compare 双 b 形留挂账见 §5）；T1MID anchor=B@0 inCEfalse=false；T1C
  caretRange=**B@0**（修前 SPAN@0）。
- zz-probe-tstylediag：D1/D3 caretRange B@0/B@1（修前 SPAN@0/SPAN@1）、D2
  raw=B@0 plain=true。D4（元素位直插诊断案）仍 throw "The given node is
  invalid."（T1B 同，非本轮 7 案，见 §5）。
- zz-probe-d4br：F1 `<b>abc</b>d`、F2/F3 `<i><b>abc</b>d</i>`、F4/F5
  `<b><i>abc</i></b><i>d</i>`（D4 已锁形不变）、**F6
  `<b><i>abc</i></b><i>d<br></i><b><br></b>` 精确期望全形**；qS_after 全 false
  合 state 表。

## 4. 门禁（gate21 批：run 通道全量 + other 通道 + engine lib）

**run 通道全量**（transcript + 逐案 JSON，test-guard 4G/8G/3h；首次全 44 文件记账）：

| 面 | 上一轮 | gate21 | 位移 |
| --- | --- | --- | --- |
| multitest | 9,588P/486F | **9,647P/437F/1T** | F→P 59（dP=+59/dF=−49，?9001-last 1 案 Timeout 挤占少量断言） |
| inserttext | 2,779P/230F | 2,782P/227F | F→P 3（typing 态同域激活面） |
| ipara / forecolor | 6,728P/467F / 2,279P/48F | 同 | 恒等 |
| delete.html / forwarddelete.html | （此前未记账） | 7,339P/296F/1T / 6,831P/258F/1T | 首记账；失败簇全为 collapsed 跨块合并 compare + foreColor qV（capture 不涉：collapsed 不捕获、qV 读 override 仓不读 typing 态） |

multitest 挂账行逐条：CE-false contenteditable 行 Fail=0（此前 ×4）；`[def]`
全含双壳 delete/forwarddelete compare 全 Pass（此前 ×2 挂）；:2429 ipara qS
Pass、compare 仍 Fail（§5 挂账）；F6 `abc[]<br></i><br>` 行全 Pass。双壳
`{...}`/`[<b>` 部分含形（`abc{<b><i>def</i></b>}ghi` 等 ~25 断言）仍 Fail——
start 边界在链外（abc 文本/壳前），捕获链为空、本轮各臂路径惰性，归既存 F
（Chrome "preserving inline style of the last visible thing in the selection"
语义，需终点边界捕获面，挂后续轮）。其余 F 簇（styleWithCSS span、img 选择、
table td、fontsize qV）皆既存形。

**other 通道**（13,134P/1,600F rc=0，锁面逐位恒等）：wss-ip 1,104P/0F、wss-ilb
552P/0F、join 三案 3,456/2,607/1,455P/0F、ipihost 880P/0F、chd 420P/0F、void
双语料 72+48P/0F、run-ilb（run 通道 insertlinebreak 1,641P/50F）恒等。

**engine lib**：`cargo test -p zero-engine --lib` rc=0，**3,686P/0F**（基线
3,684P/0F，+2 为期间新增钉测，0F 信号不变）。

## 5. 残余与挂账

- :2429 compare（T7）：ipara 在 `(b,0)`（b 为带占位 br 的幸存壳）拆分时把空壳
  b clone 进新块、旧壳 br 被 typed 段占位消费吞掉，得
  `<div><b></b></div><div><b>a</b><b></b></div>`；期望 `<br><div><b>a</b></div>`
  需 ipara 拆分臂对"空 inline 壳"做 Blink 形弃壳（新挂账，非本轮面）。
- T1B/D4 诊断案（元素位 (b,0) 直插 insertText）throw "The given node is
  invalid."——非 multitest 语料面（语料 caret 均落文本位），诊断探针独有；
  根因未追（疑 override 重放或 setSelectionValue splitText 面），不影响门禁面。
- qV 残余 ~109F、list FIXME、delete li/dt/dd FIXME、shim
  DocumentFragment.appendChild 移动语义/Range.extractContents 共享根（控制面
  下一步 ②-④ 原样顺延）；新增：双壳终点边界捕获面（`abc{<b><i>def</i></b>}ghi`
  部分含形 ~25 断言，Chrome "last visible thing" 语义）。
