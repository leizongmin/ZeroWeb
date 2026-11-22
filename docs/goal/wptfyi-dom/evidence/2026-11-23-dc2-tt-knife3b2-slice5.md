# DC-2 切片 5 刀③b② 证据：TT HTML sink 族全钩（2026-11-23）

## 交付面（part02/03/04/05/06 五件 13 钩点 + part14 划界更新）

sink 名与 null 语义均按语料 ground truth（`support/helper.sub.js` 断言函数 +
18 页 block-string-assignment-to-*.html default-policy 断言面）钉死；全部钩点
enforced 门控（非 enforcement 页零变化）。

1. **part04 innerHTML**（setter，`_ihVal` 归一后）：sink 名容器分型
   `Element innerHTML`/`ShadowRoot innerHTML`（handle 经 `_shadowHandles` 判）。
   LegacyNullToEmptyString：null → '' 仍进管道（无政策 TypeError 于解析前，
   树不变）。`__zwDocWriteReplay` 门：write 流回放（part06 基线 innerHTML= /
   VisExtend insertAdjacentHTML，R5206 既有跨件旗）的值已在 write() 入口过
   管道，回放面不再二次处理。
2. **part04 outerHTML**（`_ohVal` 后、parent 判定**前**）：spec compliantString
   先于 parent 判定——parent 为 Document 时 string 抛 TypeError 而非
   NoModificationAllowedError（语料 Element-outerHTML 页双臂实证）。
   LegacyNullToEmptyString（default-policy null 臂读回 ""）。
3. **part04 insertAdjacentHTML**（函数入口）：spec 步骤 1 compliantString 先于
   position/context 判定（TT 先于 SyntaxError）。plain union：null → "null"
   进管道。含 replay 门（write 流 VisExtend 通道）。
4. **part04 setHTMLUnsafe**（返回函数入口；setHTML 消毒面非 TT sink 不挂钩）：
   容器分型 `Element setHTMLUnsafe`/`ShadowRoot setHTMLUnsafe`。plain union。
5. **part04 script.text**（SCRIPT 门 TT 分支重写）：`TrustedScript` 管道先行
   （sink `HTMLScriptElement text`），变换值置单次透传旗标委托
   `.textContent=`（防 SCRIPT+textContent 钩二次管道）。SVG script 非 sink
   （HTML ns 门）。**自审修复**：初版注释声称持旗标但未置位——TT 实例相变
   字符串后委托会二次入管道（无政策页 TypeError）。
6. **part04 textContent/innerText SCRIPT 门**（分支头，`p !== 'innerHTML'`）：
   sink 名 `HTMLScriptElement ` + p；LegacyNullToEmptyString null → '' 进管道。
   div 非 sink、innerHTML 走下方分支同名 `Element innerHTML`。
7. **part05 script.src 专用臂**（`_s96Val` 写属性前）：`__zwTtAttrSinkData`
   判定（`HTMLScriptElement src`，TrustedScriptURL）→ 管道后写。null 读回
   URL-resolved "null"（plain union）。
8. **part05 generic reflected 臂**（`_refAttr` 写前）：同款判定——覆盖
   iframe.srcdoc 反射通道（`HTMLIFrameElement srcdoc`，TrustedHTML）。
9. **part06 document.write**（TT 管道替换 concat 循环）：spec #document-write-
   steps 步骤 1–5——TT 实例取关联数据串直拼；任一参非 TT（null → "null"）
   置 `_dwTtHasPlain`，拼接后整体过**一次** default policy 管道（sink
   `Document write`）；无政策 TypeError 时 buffer 不动＝写入未发生。TT 检查
   先于解析/写入面。else 分支保持既有 null → '' 口径。
10. **part06 document.writeln**（重写）：同构逐参管道（sink
    `Document writeln`）→ 置 bypass 旗 → 委托 write；`'\n'` 管道**后**追加
    （spec 步骤 5 在步骤 4 后；语料 assertSinkEqualsDocumentWriteLn 实证 \n
    不进 policy 回调）。
11. **part06 Range.createContextualFragment**（arity 检查后）：spec 步骤 1
    compliantString 先于 context 解析；plain union。
12. **part02 DOMParser.parseFromString** / **part03 Document.parseHTMLUnsafe**
    （各自入口）：sink 名按语料同名页 default-policy 断言；plain union，
    既有 null 口径保持。
13. **part14 头注划界更新**：③b② 全清单入册（钩点在消费面五件，part14 只持
    管道核心）。

**语料新定性**：③b① 证据文「DOMParser/parseHTMLUnsafe/setHTMLUnsafe/
createContextualFragment = API 缺失域」**过时**——四 API 实现都在
（part02:6399 / part03:2638 / part04:1918 / part06:11504），四页纳入本刀
（同族同机制小幅扩展）。

## 自审修复（3 处二次管道防护）

- script.text bypass 旗（上述第 5 条）。
- innerHTML + insertAdjacentHTML `__zwDocWriteReplay` 门（write 回放值已在
  write() 入口过管道）。
- open()/close() bypass 置位（enforced 门控）：open 清空 `innerHTML = ''` 与
  close 缓冲应用在 enforced 页会二次入管道。非 enforcement 页**不**置位——
  sinkValue 旗标消费先于 enforced 检查，非 enforced 页置旗会泄漏到下一次
  页面 sink 调用。

**零重入面排查**：outerHTML handle 路径走 native `__zw_insert_adjacent_html`
直调、setHTMLUnsafeImpl 走 `_zwFragmentAddedUnsafe`、testharness.js 仅
innerHTML 读（getter）、testharnessreport.js 零 innerHTML——native 直调不经
JS trap，零重入。

## 语料对账（第一轮 → 修复后第二轮）

**第一轮**：block-string 族 **68P/40F**（基线 31P/82F，+37P/-42F）+ 3 页
Timeout。③b① 回归面全恒：set-attributes-require-* 三页 **536P/0F**、
no-require **202P/0F**、双探针 16P/13P。

**探针定谳（zz-knife3b2-probe*.html 六件，临时件不入库）**：第一轮余 F 归因
两类——

- **本刀缺陷（TT 域，第二轮修复）**：四钩点把 TT 实例先 `String(value)` 归一
  再入管道——字符串化毁 isHTML/isScriptURL 品牌判定（管道 isTT 恒假）→ 无
  default policy 窗口（页首命名 policy 阶段）TypeError 炸测试与页级 cleanup
  → 级联（后续测试 cleanup 断链、ShadowRoot 两页 Timeout）。修复 = 管道收
  **raw** 值（null 预归一，管道内做 String 化）×4：part04 innerHTML
  （`_ihVal` 改传 `value`）、part04 outerHTML、part05 script.src（传 raw
  value）、part05 generic reflected（传 raw value + null-form 门复刻）。
  第五修复 = insertAdjacentHTML 补 position 四值校验（R293 同款成语，管道
  **后**判定——spec 步骤 1 compliantString 先于步骤 2 position parse；宿主
  不校验非法位置，探针 p08 零抛实证）。探针复验：p03/p04/p05/p07/p12/p13/
  p14/p15 PASS、p08 抛 SyntaxError DOMException（探针断言查 name 非
  message——引擎行为正确）。
- **预存引擎面（非本刀回归，probe 在非 enforced 页/基线恒等路径复现）**：
  ① IAH beforebegin/afterend 插入同步视图缺失——非 enforced 纯串同样全失
  （probe5：previousSibling null、cHTML 无插入物）；② outerHTML 替换同步视
  图缺失——移除生效、替换内容不落（probe7：cHTML=''）；③ doc-write 缓冲
  应用后 body.innerText 读空——非 enforced 基线路径 childCount=2、visLen=29
  而 innerText=''（probe2 c01，innerText getter 面）；④ 解析文档
  body.innerText undefined（probe1 p16，DOMParser 页 3F 根因）；
  ⑤ setAttributeNS null 非 sink 属性读回 ''（期望 "null"——k3b-bsa1.log
  ③b① 基线即 F）。

**第二轮**：block-string 族 **76P/39F + 1 Timeout**（run1 68P/40F + 3 TO →
**+8P/-1F**，Timeout 3→1——ShadowRoot 两页截断解除各多录 4 断言）。对基线
31P/82F **+45P/-43F**。逐页：HTMLElement-generic 6P/3F→**9P/0F**（F4
cleanup-TypeError 修复）、HTMLIFrameElement-srcdoc 4P/1F→**5P/0F**（F4）、
ShadowRoot-innerHTML 1P/1F+TO→**3P/2F** 无 TO（F1：管道不再炸，测试全录）、
ShadowRoot-setHTMLUnsafe 0P/1F+TO→**2P/3F** 无 TO（同）、setHTMLUnsafe/
parseHTMLUnsafe/createContextualFragment/text-and-url-sinks/setAttribute 全
绿持位、Document-write 10P/10F 持位（innerText 面）、IAH 0P/7F 持位
（beforebegin 面）、outerHTML 1P/6F 持位（同步替换面）、DOMParser 2P/3F 持位
（解析文档 innerText 面）、Window-setTimeout-setInterval 2P/5F+TO 持位
（刀③③ 域）、setAttributeNS 4P/1F 持位（null 语义面）。

**第二轮余 39F 全预存归因（零 TT 域残余）**：shadow 纯文本片段面
（probe9 非 enforced 复现 srHTML=''——标记片段 ✓ 纯文本片段丢，ShadowRoot
5F 根因）、innerText 空块语义（单空 div 子 innerText='\n'，Chrome 口径 ''
——setHTMLUnsafe 2F + outerHTML 页部分）、其余同上五预存面清单。

③b① 回归面第二轮复验恒等：require 三页 **536P/0F**、no-require **202P/0F**、
双探针 16P/13P、setAttribute 页 14P/0F 持位。

## 回归网（五证）

- zero-engine **lib unittests 4249P/3F 基线精确恒等**（3F = offset_path_m2 +
  min_max_single_arg + numeric_special 全预存挂账同名）✓——五件 shim 改动
  零 lib 回归。
- TT 语料全量两轮（见上）：③b① 面 536P/0F + 202P + 双探针 16P/13P 两轮恒
  等 ✓。
- san spot **323P/328F/1T ＝ ③b① 记录值精确恒等** ✓（sanitizer 语料与 TT
  钩 enforced 门零交涉实证）。
- 探针复验 5 修复（probe1 第二轮 p03/p04/p05/p07/p12/p13/p14/p15 PASS、
  p08 抛 SyntaxError DOMException）✓。
- 余 39F 全预存归因（六探针非 enforced 复现证据链）✓。

如实记档：`cargo test -p zero-engine` integration 面（43 个 CSS 流域 r5xxx
测试文件）中 r5309_text_wrap_nowrap_layout 两测失败——**CSS 流 WIP 在途破面**
（engine.rs/abspos.rs mtime 03:25 晚于 ③b① 基线，abspos +56/-15 正是 wrap
宽度几何域；本刀只改 JS shim 字符串，与布局几何零交涉），非本刀回归。

## 挂账

- **IAH beforebegin/afterend 同步视图缺失**（预存）：插入物不落 host/本地
  双视图（probe5/v4a 实证）——block-string IAH 页与 Element-insertAdjacentHTML
  页的 beforebegin 臂全部压此面；修复需 native `__zw_insert_adjacent_html`
  position 通道刀位。
- **outerHTML 替换同步视图缺失**（预存）：移除生效、替换内容不落（probe7）
  ——outerHTML 页 success/default-policy 臂压此面。
- **shadow 纯文本片段面**（预存）：ShadowRoot innerHTML/setHTMLUnsafe 对无
  标记纯文本片段丢内容（probe9 非 enforced 复现 srHTML=''；标记片段 probe8
  ✓）——ShadowRoot 两页成功读回臂 5F 根因。
- **innerText getter 语义面**（预存）：① doc-write 应用后 body.innerText 读
  空（probe2 c01：childCount=2/visLen=29 而 innerText=''）；② 空块元素换行
  语义（单空 div 容器 innerText='\n'，Chrome 口径 ''）——write/outerHTML/
  setHTMLUnsafe 页 readback 臂压此面。
- **解析文档 body.innerText undefined**（预存）：DOMParser 页 3F 根因
  （probe1 p16）。
- **setAttributeNS null 非 sink 属性**（预存，③b① 基线即 F）：读回 '' 期望
  "null"。
- **engine 内部其余 14 处 `.innerHTML =`**（part01c/f execCommand、part01 CE
  world-sync、part05 detached doc builder——如 iframe srcdoc 加载建 body 3101、
  part06:3706/3768 helpers）在 enforced 页会入管道；本刀语料零 exercising。
  若未来 enforced + 该 features 组合页出现需逐点挂门。
- **ordering caveat**：part04 innerHTML 钩 throw 路径前有
  `_zwRemoveIframeWindowClientsForNodes(_ihRemoved)` 预检——spec 要求 TT 检查
  先于一切树变更；当前容器无 iframe clients 时无影响，带 iframe clients 的
  enforced 页理论上先做了一次无害预检（读面非写面），记档。
- attribute-via-attribute-node 页 rc=1 零输出（404 载入挂起）继续挂账
  （③b① 同记录）。
- script sink wrap 重设计（刀③③）：block-string-assignment-to-*-setTimeout
  页族在册（本刀 Window-setTimeout-setInterval 页 F 形态不变，属 script sink
  域）。
