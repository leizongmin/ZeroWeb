# ipara 忠实移植轮：vendored insertParagraph 换代 + 嵌套面探针九族

日期：2026-11-02（editing 流，控制面下一步 ①，ipara other-cmp 153 嵌套面）

## 0. 范围

insertParagraph 从 live 就地拆分近似（`__zw_ce_paragraph_split_at`，F2 轮双块改造）
换代为 vendored implementation.js :7486-7826 忠实移植（part01f 新
`__zw_ce_ipara_apply`），覆盖 other-cmp 153 嵌套面（table/dl/blockquote/list 内段
落 split）。`__zw_ce_paragraph_split_at` 保留供 insertText `\n` 分流与键盘面。

## 1. 设计与过程归因

- **忠实移植决策**：嵌套面（dt/dd 换名、单行容器爬升、空 li/dt/dd splitParent、
  非 SLC wrap）在就地拆分近似里无对应臂；vendored 算法是唯一一致参考。本仓 shim
  与 vendored 的两处语义差异按本仓约定处理：`_zwJusIsEditable(host)` 返 false
  （vendored isEditable(host)=true，首版移植曾在爬升条件尝试 `isEditable ||
  isEditingHost` 组合，gate22b 实证该组合不通，见 §1.1 混合派发）；`_zwVaIsAllowedChild`（VA 语义）与 `_zwJusIsAllowedChild`（vendored 矩
  阵）在 li-in-div 形分歧，移植统一取 JUS 表。

### 1.1 宿主边界面回退：混合派发（gate22b 回归归因 + 修正）

gate22b 全量实证纯移植净回退：主语料 467F→223F（F→P 244）但卫星面 echd −210P、
wss-ip −208P、ipiv −18P、edob −5P、restoration −1P（净 −442P）。探针 v3（zz-probe-
ipara3）定谳根因：**`_zwJusIsEditable(host)` 返 false 使 SLC 爬升在宿主边界卡死**
——caret 直挂宿主（裸文本/inline 链/元素偏移）时 container 恒停在非 SLC 节点，落
spec wrap 臂产出 `<div>整段</div><div><br></div>` 形；而 Chrome/F2 语义为宿主内双
块拆分（`foo[]bar` → `<div>foo</div><div>bar</div>`）。对照实验：手工预删后走移植
（Q2）产出正确，移植自调 `_zwDelDeleteSelection`（Q4）caret 落旧 END——delete 动
作臂 `__zw_ce_delete_apply` 的 Fix A 重锚是 caret 正确性的必要件，移植裸调
deleteSelection 绕开了它。

**混合派发**（`__zw_ce_ipara_wants` 只读预检 + part06 臂分派）：container 是宿主
内**元素**（`nodeType===1 && _zwJusIsEditable && 同宿主`）→ 忠实移植（嵌套 SLC 走
主分支、宿主直子 inline 链走 wrap 臂，两态探针实证皆 Chrome 形）；裸文本（walk 卡
停在 TEXT）或宿主本体（isEditable=false）→ 回落旧 `__zw_ce_paragraph_split_at`
（gate21 已知绿，echd/wss/void/edob/restoration 各面构造上复位）。SLC 判定对路由
非必要——判定容器是「宿主内元素」即可，非 SLC wrap 臂自会正确处理 inline 链
（探针 v4 P3 定谳：首版判据含 SLC 条件把 P3 误判 legacy 得 `<b>foo</b><div>…`，
去 SLC 条件后转 `<div><b>foo</b></div><div><b>bar</b></div>`）。

### 1.2 弃用 splitText 归一臂：shim 文本双态缺陷（临时 trace 定谳）

gate24 前 edob 五变体 `ab[]c`（宿主内嵌套 div 形，wants 判真走移植）产出
`<div><br></div><div>abcc</div>`（尾字符翻倍 + 空块前置），且 apply 单次调用即
坏（探针 v8 计数 apply=1/split=0，非双派发）。移植内加临时 trace 实证三步：

1. `splitText(2)` 真实分裂成功（parent kids=2）；
2. 分裂后活动 range 被重锚到 `(div,0)` 而非 spec 的 `(textL,2)`；
3. `_zwIparaExtractRange` 读到**包装层 stale `.data`**（旧串 "abc"）而定位走真
   树双节点——clone 用旧串、定位用真树，产出 `abcc`。

即 shim 文本包装层 splitText 后 selection 重锚与 `.data` 双态皆不可信（Q5 页面
侧 wrapper 直调 splitText 缺失为同族缺陷）。老 `split_at` 以引擎 SetChildText
重写文本绕开了同一雷区。

**修法**：弃用 spec 归一的中点 splitText 臂——mid-text 起点由
`_zwIparaExtractRange` first-partial CharData 路径（cloneSlice+deleteData）与
`_zwIparaInsertNode` 原生消化，语义等价。连带两处 spec 前提修正：第三段提升臂
（text/comment→parent@index）只对 comment 与 0 位文本生效（mid-text 保持文本偏
移，否则拆分点塌到 0）；endOfLine 判定补 `startInTail`（起点在文本中段即有尾随
内容——spec 下尾段必为 splitText 拆出的整 contained 节点，不 split 则 contained
恒 0，h1-6/dt/dd 改名会误触发，探针 P8 `dt[]` mid-caret 实证 `dt→dd` 误改名，
修正后 dt→dt ✓）。`__zw_ce_ipara_wants` 镜像归一同步。

### 1.3 路由三规则（gate24/25 other 对账残差逐面收敛）

混合派发落定后 other 通道三轮对账（gate23 → gate24 → gate25）逐面收敛残差，
追加三类 legacy 回落规则（`__zw_ce_ipara_wants` 内，均 gate21 已知绿辖区）：

1. **void caret**（ipiv −18 面）：caret 挂 void 元素（hr/embed/...）落 legacy
   （移植 wrap 臂产 div 壳错形）；
2. **宿主为不可容块的行内容器**（echd 面）：宿主本体是 p/h1-6 等 SLC 且非
   div 落 legacy（Chrome 退化 `<br>` 插入而非块拆；div 宿主可容块不在此列）；
3. **white-space / display:inline(-block) 样式族**（wss 面）：宿主带
   white-space 声明或 caret 祖先链 display:inline/inline-block 落 legacy（移植
   wrap/主分支丢样式继承与空壳延续）。

两条工程教训：**样式读数必须 style 面板优先**（wss 语料经
`editingHost.style.whiteSpace = ...` IDL 设置，style attr 为空——attr 扫描全数
漏接；探针 v13 X1/X2 定谳，panel 优先 + attr 兜底复位）；**part01.js 是 IIFE
私有作用域**（`_zwCeIsVoidElementTag`/`_zwCeDisplayOf` 不可达——首版路由调用
即 ReferenceError 被外层 catch 吞成恒 false，part01e 的 `_zwList*` 顶层声明则
可见；本地复制 `_zwIparaIsVoidTag`/`_zwIparaDisplayOf` 同构解决）。

探针矩阵（v4 P1-P12 / v2 C1-C9 / v10 V1-V7 / v13 X1-X3 / v8 U2 / v6 S2，r5 全
绿）：legacy 侧 P1 `foo[]bar` 双块、P7 非塌缩、P6 echd 退化 `a<br>1b`、P9 wss
宿主、V1-V5 echd/wss/void；port 侧 P2/P4/P5/P8 嵌套 p/li/h3/dt（dt mid-caret
不改名）、P3 inline 链 wrap、P10/P12/V6/V7 嵌套块与空 li splitParent、P11 pre
支路、C1-C9 九族、X3 面板样式嵌套块。
- **extract/insertNode 本仓 spec 自实现**（`_zwIparaExtractRange`/
  `_zwIparaInsertNode`）：shim `Range.extractContents`（part06 :7883）带 form 节流
  且 X1 形（尾兄弟不迁移）实证不达 spec；shim `DocumentFragment.appendChild` 无移
  动语义（X4 实证：host 桥 append 不摘源父，下游 parent-walk 双父窗栈溢出）。
  移植内 extract 的 contained 子迁移采用 **removeChild 先于 appendChild** 序消双
  父窗（共享根修复仍挂账④ 独立轮）。
- **collapsed 守卫自算**（`sc===ec && so===eo || ar0.collapsed`）：shim 个别
  delete 路径留下的活动 range `collapsed` 字段可过期（sc===ec 同偏移而
  collapsed=false，C5 实证），spec 首步「collapsed → 不删前导直接继续」须以点等
  价自算，不信缓存字段。staleness 根源（哪条 delete 路径漏 recalc）未定位，挂账。
- **空 li/dt/dd 分支 rename walk 扩 li**：vendored 只对 dd/dt 做「非任何同宿主祖
  先 allowed child → 换 default 名」，因 vendored `fixDisallowedAncestors` 用
  VA 语义（li 在含块 div 里算 allowed）且本仓 `_zwJusIsEditable` 门挡掉 while 爬
  升，li 形（`<ol><li><br></li></ol>` 空拆）在本仓语义下不换名留
  `<ol><li><br></li></ol>`；语料期望 `<div><br></div><ol>...`。移植将 allBad 同
  宿主祖先 walk 扩到 li（li/dt/dd 同表）。

## 2. 实现（part01f.js +541 行，part06.js 接线一笔）

- part01f 追加 `__zw_ce_ipara_apply`（deleteSelection 前导 → 文本归一（尾偏移
  升父/0 位文本与注释升父；中点 splitText 臂弃用，见 §1.2）→ 单行容器爬升 →
  非 SLC 分支
  blockExtend+首 contained wrap / nodeList 空 createElement+insertNode+br →
  pre/address/listing br 支路（含非块尾双 br 垫底）→ 空 li/dt/dd splitParent +
  rename walk（li/dt/dd）+ fixDisallowedAncestors → 主分支
  newLineRange 归一爬升（prohibited 边界）+ endOfLine 判定 + H1-6→sep/dt→dd/
  dd→dt/else 本名 + attr 拷贝去 id + insertAfter + extract + frag 后代 id 清除
  （非 containedNodes）+ 双 lastChild prohibited 下钻 + 双不可见 br 垫底 +
  collapse(newContainer,0)）。helpers：`_zwIparaIsSingleLineContainer`/
  `_zwIparaDeleteData`/`_zwIparaHasVisibleChild`/`_zwIparaGetDescendants`/
  `_zwIparaExtractRange`/`_zwIparaInsertNode`。
- part06 insertparagraph 应用臂换混合派发（`__zw_ce_ipara_wants()` 真值走
  `__zw_ce_ipara_apply`，否则旧 `__zw_ce_paragraph_split_at(host2, rng2, false)`
  原参回落；异常吞没同邻臂口径）。`__zw_ce_ipara_wants` 只读预检（镜像文本归
  一不 splitText + SLC 爬升 + 元素/可编辑/同宿主判定，异常返 false 走 legacy）。
- override 快照/重锚/回填（`_zwVaOverridesTakeRaw`/`_zwVaReanchorOverrides`/
  `_zwVaOverridesPutRaw`）同 `__zw_ce_delete_apply` 口径（动作内 override 读触
  发 staleness reset 误清，g17 实证面）。

## 3. 探针（zz-probe-ipara2.html 九族，重建后全绿）

C1 `<b>foobar</b>`+div → `<div><b>foo</b></div><div><b>bar</b></div>`；C2/C8
`<p><b>foobar</b></p>` div/p sep → `<p><b>foo</b></p><p><b>bar</b></p>`；C3
`<ul><li><p>foobar</p></li></ul>` → `<ul><li><p>foo</p></li><li><p>bar</p></li>
</ul>`；C4 `<ol><li>abc</li><li>def</li></ol>` delete 全 abc + ipara →
`<div><br></div><ol><li>def</li></ol>`；C5 `<h3>abc</h3>` delete + ipara →
`<h3><br></h3><div><br></div>`；C6 `<td>foobar</td>` caret 3 →
`<td><div><br></div><div>foobar</div></td>`（**vendored 参考形**：wrap 后 active
range 塌 (wrap,0)、div prohibited 主分支不爬升 → extract 全含清空 + br 垫底；
corpus 无 td fixture，本族为参考推演记录）；C7 `<dl><dt>foobar</dt>` caret 3 →
`<dl><dt>foo</dt><dt>bar</dt></dl>`（corpus dt→dt 形：`_zwJusBpCompare`
child-vs-parent 约定与 vendored getPosition CONTAINS 分支逐一核对同构，bar 文本
contained → endOfLine=false → dt 不改名）；C9 `<i><b>foobar</b></i>` caret 3 →
双 div 包裹（爬升止于非 editable host 边界，非 SLC wrap 形）。

诊断探针（zz-probe-iparac45/c6/c7/iparamin）一次性归因用，存 `.acceptance/
ipara-port/` 不入库。

混合派发路由探针（zz-probe-ipara4 十二族，gate23 前全绿）：P1 `foo[]bar` 裸文
本 caret → legacy `<div>foo</div><div>bar</div>`；P7 `foo[1b]baz` 非塌缩 →
legacy `<div>foo</div><div>baz</div>`；P6 `<p contenteditable>a[]1b</p>` echd
宿主面 → legacy `a<br>1b`（退化臂）；P9 wss 宿主面 → legacy 块拆 + br 垫；
P3 `<b>foo[]bar</b>` inline 链 → 移植 wrap `<div><b>foo</b></div><div><b>bar
</b></div>`；P2/P4/P5/P8/P10/P11/P12 嵌套 p/li/h3/dt/wss/pre/空 li → 移植主
分支/pre 支路/splitParent，各 Chrome 形。

## 4. 门禁（gate22b→26 五批，test-guard 4G/8G）

批次链（产物 `.acceptance/ipara-port/`；other 通道 summary 行 `cases=` 只计有断言
产出的实跑案——103 案中恒 10 案被 runner `editing_case_skipped` 筛减（8 含
`<iframe` + 2 `.https.`）零断言输出，JSON 仍按 transcript 头记 103；每轮干净跑
恒 93 实跑 + 10 筛减，gate22b 的 93 非异常）：

| 批（时点） | 树 | other 通道 | run 通道 | engine lib |
| --- | --- | --- | --- | --- |
| gate22b（15:41-18:04） | B1 纯移植（65cc6a4ca） | 12,692P/2,042F | 全量 90,158P/6,025F/3T rc=0（run ipara 467F→223F，F→P 244；卫星面净 −442P） | 3,698P/1F rc=101（1F = F2 钉测 `test_insert_paragraph_split_r3254_m3_slice5`，纯移植换拆分序按设计失败） |
| gate23（19:43-20:31） | 混合派发初版 | 12,805P/1,929F | — | 3,699P/0F rc=0（23b/c/d/e 四连同值） |
| gate24（20:32-20:50） | +void/echd 回落规则 | 12,834P/1,900F | — | — |
| gate25（20:58-21:28） | +wss 三规则终态（B1'） | 13,005P/1,729F | — | 3,699P/0F rc=0 |
| gate26 前轮（21:26-22:28） | B2（style-panel 修复落树，§1.3） | **13,134P/1,600F** = gate21 逐位 | b→formatblock 14 案与 gate21 零位移（delete 7,339P/296F/1T、forecolor 2,279P/48F 等），之后 run 腿中断 | — |
| gate26 终批（23:23-23:39） | B2 + 22:39 pull 合并树（cc097dbc3） | **13,134P/1,600F/0T rc=0，103 案逐位 = gate21**（`cmp-other.py` PER-FACE identical；wss-ip 1,104P/0F、echd 420P/0F 另经 `-one` 直跑全分母双验） | 电池未跑成：leg3 于 23:47 会话重启竞态对账中主动终止（ps 实证存活后 kill，非进程组退出；kill 前无断言产出）；孪生会话 23:53 补启的 zw-gate26b2 unit 因首案跑 pre-pull 旧基二进制、后续腿将撞新树混基，00:01 停止 | **3,707P/0F rc=0**（较 gate25 的 3,699 +8 系 pull 并入的并行流补面测试，本轮零测试新增） |
| gate27 终验（23:58-，钉基 58de3d5ec + commit 5e694d5a7） | 新基唯一记账源（23:53-23:56 并行流 s244/s245/R5394/PQC 五提交并入后 `pull --rebase`，commit 已 push） | **13,134P/1,600F/0T rc=0，103 案 PER-FACE identical = gate21**（00:14 落地，other 通道第四次确认） | 电池 v1 方法缺陷终止（账作废留档）：`-one` 路径缺 corpus 脚本的 `ZW_SCRIPT_TIMEOUT_MS=300000` 导出，90s 缺省看门狗截断大变体（insertparagraph 仅 6,512P/141F/1T rc=2，~542 行缺失，不可比；Makefile `-one` 内联导出根因修复已落工作区另行提交）；v2 补导出 + TIME_LIMIT=10800 重跑 8 腿（00:31-02:10 落定，8 腿 END 齐 + done 标记，分面终数 `gate27-battery-summary.txt`）。六腿有效：insertparagraph **6,984P/211F/0T vs gate21 6,728P/467F/1T——F→P 净移 256（ΔP=+256/ΔF=−256 恰互偿）**（B1 纯移植 223F → 本轮 211F 再 −12，混合派发+样式面板叠加信号；总数 7,195 vs 基线 7,195（6,728+467 ex-1T 口径）持平，gate21 侧无本地 transcript 不可逐面精核；8/8 chunk ?1-1000…?7001-last 行数 7×1000+195 全在——rc=2 系 runner text 模式退出策略 main.rs:487 failed→exit(1) 经 make Error 1 透传，非截断信号）；multitest **9,569P/430F/1T vs 基线 9,647P/437F/1T——缺 85（ΔP=−78/ΔF=−7，P/F 同降=输出缺失非行为位移）**，?9001-last 切片嫌疑最大但量级超单案 flake，solo 切片重跑归因后定谳（重点核尾部断言无隐藏 P→F）；inserttext 2,782P/227F、insertlinebreak 1,641P/50F、forecolor 2,279P/48F **三腿逐位=gate21**；typing-text-in-direct-child-of-editing-host 6P/10F/0T 首次单腿记账（gate21 无 per-file 基线，不判回归）。两腿无效账：delete.html（01:27-01:58 整 31 分钟）、forwarddelete（01:58-02:10 仅 11 分钟）均**零断言产出后 make Error 1**——transcript 零 flush、test-guard 三种击杀路径（超时/总内存/单进程）日志无留痕、TIME_LIMIT 未到。机制定谳（02:26 会话 B 码检三证）：**跑批中段静默原生信号崩溃**（SIGSEGV/SIGABRT）——① test-guard 对子进程信号死亡映射 rc 1（`status.code().unwrap_or(ExitCode::from(1))`，scripts/test-guard.rs:407-409），make Error 1 与信号死亡一致，非干净 exit 1 的证据；② runner stdout 是 Rust LineWriter 逐行即刷（文件同样），「尾声崩溃丢全量缓冲」不成立——零行 = 批量打印阶段（`run_editing_corpus_dir` 收集完毕 → `run_observers_cmd` 收口打印）根本未达；③ 两腿全程 ~100% CPU 31/11 min，空 cases 秒退路径排除 ⇒ 崩溃在逐案执行段。两腿全灭 + 其余六腿 transcript 完整 = 系统性、delete 语料特异（11 vs 31 分钟不对称 → 毒案位置相关非 RAM 累积型）；gate21 同 runner 子命令曾完整记账 7,339P/296F/1T、6,831P/258F/1T——崩溃为移植后树新行为，候选属主 ipara 移植 5e694d5a7 或并行 s243 cc097dbc3（同碰 part01f/part06），毒案定位后判；battery 原始账只记异常事实，两腿以直跑重跑数字记账（g27-del-rerun 02:14 起，绕 make 抓真实 rc） | **3,709P/0F rc=0**（较 gate26 +2 = s244/s245 桥测并入，0F 信号不变） |

**归因链**：gate25 对 gate21 恰 −129P/+129F，`cmp-other.py` 对 gate25 JSON 复核差
异仅两面——echd 420→363P/0→57F + wss-ip 1,104→1,032P/0→72F；B1→B2 修复
（§1.3 style 面板优先）后 gate26 前轮即回 13,134P/1,600F。edob 25F/restoration
24F 定谳 gate21 既有失败（B1 前后恒等），非本轮引入。

**合并树中性**：HEAD reflog 实证 15:40→22:39 本树停 65cc6a4ca（gate22b-25 皆旧
树），22:39 `pull --rebase` 并入 s239-s243 / R5391 / R5392 / webcrypto XOF /
site-opt #273（引擎 src 并入 ~9 枚 `#[test]`）。终批 leg1/leg2 为合并树首次全量
门禁：other 逐位持平 + engine lib 0F——ipara diff 与并行流批次组合中性。run 电
池基线（gate21 系数）隔两波 pull，见位再按 insertParagraph 路径面归因（s243 同碰
part06；控制面「合并基线复核」先例 baec34fff）。

**探针复验**：zz-probe-ipara2 九族 + zz-probe-ipara4 十二族在终批干净二进制复验
全绿，值与 §3 逐字一致（dbg 临时导出已移除；shim 18 分片全拼接 `node --check`
通过）。

## 5. 残余与挂账

- ipara non-compare 174（in_array 63 多形态允许值 + equals 111 读面）非本路面。
- :2429 空壳弃形（ipara 拆分臂对空 inline 壳 Blink 形弃壳）挂账。
- shim Range `collapsed` 字段过期根源（delete 路径漏 recalc）+ shim
  DocumentFragment.appendChild 移动语义 / Range.extractContents 共享根修复（控
  制面④ 独立轮）。
- 双壳终点边界捕获面（~25 断言）、qV 残余 ~109F 挂账不变。
