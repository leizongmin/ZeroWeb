# compare innerHTML 残余空白族收口——五命令 forceValue 空白门 + wrap 空白携带 + br 抑制 + formattable 双向 collapsed

日期：2026-11-01（editing 流，compare innerHTML 残余 256F 空白族归因轮 + 429 中断面处置）

## 0. 429 中断面处置（上一 session 遗留工作区验收）

上轮 gate9 提交（e73eb4cbd）后 API 限流中断，工作区遗留 part01d.js 两笔在途：

1. **splitText start 侧原位写**（选区重绑舞撤改）：vendored `removeAllRanges/addRange`
   舞蹈在本仓无 globalRange 面为无效操作——半成品 `newActiveRange`（不同容器臂
   end 未设）经重绑后 `rng` 丢失真端点 → contained 集空 → 整段 apply 零变异
   （`foo[bar<b>baz]qoz</b>quz` 族）。改为对 active range 全程原位写。
   **验收**：forecolor gate9 2,233P/94F → 中断面二进制 g10 2,255P/72F，F→P=22
   （16 qV-after + 6 compare innerHTML）、P→F=0——成立，保留。
2. **MAIN 循环 invisible skip**（`node.nodeType === 3 && !_zwJusIsVisible(node) continue`）：
   判定条件失效——`_zwJusIsVisible` 依赖 `_zwJusIsCollapsedWhitespaceNode`，而该
   helper 是 vendored :912-1001 的「结构化重写」且**只保留后向走查**；w1 的
   ` " "@DIV`（前向紧邻 foo 文本）后向走查断于非空白文本 → 判可见 → 门不触发
   （探针 LOG：`MAIN #" "@<DIV> | PD | FV #" "@<DIV>` 无 skip 记录）。g10 实证
   六行空白族 compare innerHTML 全部未回收 → 撤改，重构为下述三笔 + 一笔查询面。

## 1. 探针定谳（gitignore 临时页 `zz-probe-ws.html`，五形 + _zwDbg 临时插桩）

| 形 | Chrome 期望 | 修复前 got | 探针 LOG 关键证据 |
|---|---|---|---|
| w1 `<p>foo</p> <p>bar</p>` | 块间空白裸 | `<span style=color> </span>` 独立包裹 | `" "@DIV` 判可见（后向断于 foo）→ FV → 独立 wrap |
| w2 同 w1 styleWithCSS=false | 空白裸 | `<font color=#0000ff> </font>` | 同上 |
| w3 `<span>foo</span> <span>bar</span>` | 单一外层 wrapper 含空白 | 双兄弟 span + 多余 `<br>` | wrap([span2]) 不拉可见空白 → sibCrit 错过外层 wrapper → tail 独立包 + br 插入 |
| w4 `<p>foo</p><p> <span>bar</span> </p><p>baz</p>` | 块内两空白裸 | 尾空白独立包裹 | 尾空白在 span wrap 后判可见（前向断于 bar 文本）→ FV 独立包 |
| w5 `<p>foo</p><p><br></p><p>bar</p>` | br 与尾块包值 | `<p><br></p><p>bar</p>` 零变异 | apply 中止于 `FV <BR>→WRAP` 内部异常（part06 `catch (_eVa) {}` 吞没）——独立族，见 §6 |

插桩版三形修复后 innerHTML 与录制逐字节一致（w5 除外，独立族）。

## 2. 修复四笔

### 2.1 forceValue 入口空白门（part01d）

`_zwVaForceValue` 入口 `node.nodeType === 3 && _zwJusIsWhitespaceNode(node)` →
return。Chrome delta：空白-only 文本节点不强制赋值（不获独立包裹、不进 children
递归臂）。门置入口统一覆盖主循环/pushDownValues 兄弟臂/children 递归各调用点；
pushDownValues 本体维持 vendored 全量。`_zwJusIsWhitespaceNode` 感知
white-space 属性，pre 格式化空白不受门。

### 2.2 wrap 拉邻放宽（part01d 两处 while）

`_zwVaWrap` 拉邻条件 vendored 只拉 invisible 兄弟，放宽为 invisible **或
whitespace-only 文本**。w3 合并机制：wrap([span2]) 先拉 " " 入列，
`sibCrit(nodeList[0].previousSibling)` 才能命中已样式化外层 wrapper（simple
modifiable + specified 等价）→ newParent = 外层 wrapper → 空白与 span2 一并并入
→ 单一外层 span 形 = 录制。仅随携带不独获值（2.1 门）。

### 2.3 br 插入抑制（part01d 三臂）

`_zwVaWrap` 三处 br 插入臂（pre-move 前后两臂 + nextSibling 合并臂）补连接点
空白隔断判定：`!_zwVaIsWsText(nodeList[0])`（前向臂）/ `!_zwVaIsWsText(nodeList
[len-1])`（后向臂）/ `!_zwVaIsWsText(newParent.lastChild)`（合并臂）。vendored
br 防行内 run 连字；连接点被空白-only 成员隔断时空白已防连字，br 成多余可见断行
（w3 录制无 br 实证）。

### 2.4 formattable 可见面双向 collapsed（part01c）

`_zwJusFormattableVisible` 新 helper：文本节点 collapsed 判定在后向走查之上补
**前向走查**（自 node 沿 nextNode 至块界 stop = `nextNodeDescendants(block
ancestor)`，遇 block/br → 折叠；遇非空白文本/img → 不折叠），`_zwValIsFormattable`
可见性消费面切换到该 helper。仅 formattable 消费面取双向语义；结构面（block
boundary、follows/precedesLineBreak）维持既有单向语义不动（wss/ilb/join 基线
锁定，避免全 helper 语义翻转的未评估扩散）。

**中途回归与回收**：仅 2.1-2.3 时 forecolor 2,256P/71F，四行
`queryCommandIndeterm("forecolor") after` 新败（w1/w4 × 2 变体）——裸空白被
`_zwValIsFormattable` 判 formattable（旧单向判定可见）且无色 → indeterm true ≠
录制 false；2.4 落地后四行全收。

## 3. 语料对账（中途二进制；终数见 gate10f 批）

forecolor：gate9 基线 2,233P/94F → 本轮 **2,260P/67F/0T**，F→P=27（修复行计
31、中途回归 4 行已回收）、P→F=0、分母恒等。分阶段归因：splitText 中断面
22 行（16 qV-after + 6 compare innerHTML）→ 本轮 apply 三笔 5 行（w1 compare
×2、w3 qV-after ×2 外层合并后选区端点落 wrapper 内、ce=false 尾空白 compare
×1）→ formattable 面 4 行（indeterm-after 回收）。

## 4. 其余四语料 + 位移电池 + 门禁（gate10f 串行批，干净二进制，18:33-19:01）

五语料（对 gate9 基线整行 ID 对账）：

| 语料 | gate9 | gate10f | F→P | P→F |
|---|---|---|---|---|
| fontsize | 2,301P/121F | **2,329P/93F** | 28（21 qV-after + 7 compare） | 0 |
| forecolor | 2,233P/94F | **2,260P/67F** | 27 | 0 |
| fontname | 2,074P/151F | **2,150P/75F** | 76 | 0 |
| backcolor | 965P/61F | **1,002P/24F** | 37 | 0 |
| hilitecolor | 1,144P/74F | **1,181P/37F** | 37 | 0 |
| **合计** | **8,717P/501F** | **8,922P/296F** | **205** | **0** |

干净二进制 forecolor 失败集与中途（插桩剥离前）逐位 identical（`diff` 空）。

位移电池逐位恒等：join 三案 **3,456/2,607/1,455P/0F**、wss-ilb **552P/0F**、
wss-ip **1,104P/0F**、chd **420P/0F**、ipihost **880P/0F**、void **72P/0F +
48P/0F**、ilb run **1,641P/50F**（gate9 同数）、ihost 宽 filter 同 gate9 空形。

门禁：engine lib **3,670P/0F**（gate9 3,667P/0F，+3 同窗并行流新增）、webview
lib **834P/0F**（env -u 清代理跑法）、wcomp **4,022P/1,146F/5T** 计数级恒等、
bench **PASS**（absolute budgets 达标）；hsem **6,913P/652F/162T**（总 7,727
恒等，+8 全 F→P——**逐行定谳 rebase 窗并行流修复面吸收非本 diff**：s209
number 编辑器 16:32:51（number-constraint ×2 + scientific ×1 +
defaultValue-clobbering ×1）+ s210 FORM 约束校验 16:56:51（form-validation-
validate fieldset ×3 + output-validity ×1），两笔提交时刻晚于 gate9 hsem 跑完
时点 16:56:16，g8/g9 快照（hsem-g{8,9}-fails.txt 662/660 行）该八行均败、本轮
转 Pass，P→F=0）。

残余 296F 普查：compare innerHTML **167**（上轮 256）、qV-after ~79、qV-before
~30、stylewithcss State before 5、其余零星。

## 5. 残余

- w5 void-br 族（`<p>[foo<p><br><p>bar]` × 2 变体/语料）：apply 于 `FV <BR>→WRAP`
  内部抛异常中止（探针 LOG 终止于 `WRAP n=1 <BR>@<P>`，后续 MAIN 不再执行；
  part06 apply 臂 catch 吞没故语料行表现为 compare innerHTML 差异非异常）。
  抛点未定位（`_zwJusIsInvisible(br)`→`_zwJusIsExtraneousLineBreak` 链初查无显性
  throw 源），独立轮。
- compare innerHTML 残余其余族（font rewrap 拆分 run 族 ~12 行、`a[]c` insertText
  multitest 8 行、stylewithcss State before 2 行等）→ 下轮 ① 续刀。
- qV before 11 行（forecolor）→ 下轮 ②。

## 6. 工件

- 探针页 `tests/wpt-runner/wpt-data/editing/run/zz-probe-ws.html`（gitignore 域）
- 中途对账工件 `/tmp/zw-edit-v/g10{,w,w2}-forecolor.out`、`g10f-*`（gate 批）
- 探针插桩版已剥（本 diff 无 _zwDbg 残留，`grep -c _zwDbg` = 0）
