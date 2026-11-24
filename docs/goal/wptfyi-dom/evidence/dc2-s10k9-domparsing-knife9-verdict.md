# dc2-s10k9 · domparsing 刀⑨ verdict（IAH/CCF 修形刀）

日期：2026-11-24　判读绑定二进制 md5：`5c13109d5befc76ead0db045b1bb7d80`（target/release/zero-wpt-runner）

## 修复面全景（七件）

| # | 修复 | 文件 | 机制 |
|---|------|------|------|
| A | IAH 兄弟链视图（R3254-K3 清基底行拆除＋槽锚合并消费） | part04.js / part05.js | 见 §1 |
| B | IAH 校验恒开＋NMA 双臂 | part04.js | 见 §2 |
| C | ccf isEqualNode | part06.js | 见 §3 |
| D | ccf 产物 appendChild 展平执行（sel 父 plain fragment 臂） | part04.js / part03.js | 见 §4 |
| E | html context 剥壳（insert_adjacent_html 消费端） | js_dom_bridge.rs | 见 §5 |
| F | IAH 产物 script already-started 章（FIXME 收口） | part04.js | 见 §6 |
| G | handle-only 容器臂同款盖章 | part04.js | 见 §6 |

## §1 Fix A——兄弟链视图（insert_adjacent_html.html 10F＋insert-adjacent.html 2F）

根因双层：
1. **R3254-K3 清基底行**（前段已拆）：`_zwChildBaseCache.set(parentSel, [])` 置空后缓存命中空数组恒不回源 host——旧子从融合视图消失、`_sbByPos` indexOf 找不到目标回落 stale pair。
2. **合并消费批内锚 miss**（本轮定谳新断点）：桶登记序＝文档序，s415 批内锚「项 i 后继＝项 i+1」指同批 pending 后继——`_zwOverlayPendingChildNodes`（part05.js）按桶序先处理前驱时后继尚未并入 `_resSeen`，next 定位 miss → 前驱尾插、后继再插基床锚位 → 批内序散架。钉测 DIAG 链 `[b,i]`/`[u,div,div,script,a]` 与模型逐位吻合（批首孤悬尾）。
   修法：两轮处理（第一轮并 next 锚不在本桶 added 集的条目；第二轮按锚依赖拓扑序并入剩余；环回落登记序）。ghost 摘位（R382/d53 splice）的索引序列同步修正 `_k9fixOrd`。

## §2 Fix B——校验恒开＋NMA 双臂（insert_adjacent_html.html 7F 校验脸）

- position 四值校验移出 TT enforcement 门（spec 步骤 2 无条件；"bar"/土耳其 "beforebegİn" 展开两码点 ≠ 11 字符/"beforebegın" 点less ı）→ SyntaxError。
- beforebegin/afterend 且 parent null 或 nodeType===9 → NoModificationAllowedError（spec 步骤 3；detached 元素＋documentElement 父为 Document 面）。

## §3 Fix C——ccf isEqualNode（createContextualFragment.html 等价表脸）

frag 字面量补 `isEqualNode: function (other) { return _zwIsEqualNode(frag, other); }`——plain nodeType 11 wrapper 递归比对既有 helper。ownerDocument 断言同 test 函数紧随自然成立（ccf 产物 ownerDocument＝range 文档）。

## §4 Fix D——ccf 产物 appendChild 展平执行（createContextualFragment.html script-run 脸）

根因链（两件）：
1. **sel 父 plain fragment 展平臂**（part04.js appendChild trap）：`document.body.appendChild(ccfFrag)`（plain nodeType 11、无 handle 无 selector）旧版全分支 miss 静默丢落。新增臂（`!handle && sel`）与 R97 handle 父臂互补：parentNode 反链＋`_zwSelPendingParent` 槽（noWire）＋childList record（addedNodes 平铺子列表）。
2. **plain wrapper 体融合读断链**（part03.js `_zwScriptMaybeStartDynamic`）：`el.text != null ? el.text : ''` 对 plain wrapper（`_zwMBuildDeepEntry` 产物无 text getter）读 undefined 得空串，textContent 回落分支 catch-only 永不触达 → prepare 静默中止。修法：text 读数 null/undefined **或空串**时回落 textContent（spec script text IDL 与 textContent 同源融合）。钉测 DIAG 定谳：`slot:unsealed:pn-body=true:text="∅":tc="globalThis.__k9ranD = tr"`。
   innerHTML 产物带 `__zwFragNoExec` 章同流免执行（章门早退不受 text 读影响）——两语义单通道分流。

## §5 Fix E——html context 剥壳（insert_adjacent_html.html implied head/body 脸）

根因完整闭环（spec 字面链＋Chrome 实证）：
- spec：fragment case reset insertion mode 步骤 3（node=fragment context element）＋步骤 14（html 分支 head 指针 null → before head）→ before-head anything else → implied head → after-head → implied body → html context `<p>` 产物＝[head, body(p)] 包裹。
- Chrome 158 master wpt.fyi：insert_adjacent_html.html 31/31 全 Pass；本机 headless 158 探针 13 面：IAH/ccf **消费端剥壳**（head/body 壳不落文档、children 展平保序、显式壳输入同样剥、纯 text 落顶层、仅 html context 生效）；innerHTML setter 不剥（解析产物保留壳）。
- 修法（js_dom_bridge.rs）：`insert_adjacent_html` 消费端 html context（context_ns==xhtml && context_local=="html"）展平一层（新 helper `flatten_head_body_children`——HTML ns 的 head/body 元素替换为其 children、壳属性丢弃）；innerHTML/共享 parse 层零动；ccf 零改（spec 步骤 6 html context 重定向 body——part06 既有回落臂 `ln !== 'html'` 天然剥）。

## §6 Fix F/G——IAH 产物 script already-started 章（part03 FIXME(spec-gap) 收口）

text 融合读修复暴露缺章：IAH plain 产物 script（textContent 有体）经 s111 prepare 可达即执行——spec HTML fragment parsing algorithm「script element created by the algorithm → set already started」不得执行（testPositions script_ran=false 断言面）。修法：part04 IAH 消费点 `_zwFragmentAdded` 产物 s118 同款递归盖章。**双臂盖章**（v1 电池 2 残 F 定谳后补）：sel 臂 `_k9fs118`＋handle-only 容器臂（`!sel && handle`，createElement 容器形）内联 `_k9fs118h` 镜像——「descendant script via IAH 后移动不得执行」脸经 handle-only 臂漏章，移动时 s111 walk 执行。

## v1 参考电池（5c13109d 二进制，14:07-14:21Z）

- domparsing 1264P/271F → **1310P/225F**＝净 +46P/−46F：ccf 24P/11F→35P/0F、insert-adjacent 2P/2F→4P/0F、IAH 13P/18F→30P/1F、**xhtml 挂账页 12P/18F→28P/2F（+16 外溢修复）**；残 3F＝2 条 handle-only 臂漏章脸（§6 已修，v2 验证）＋1 条 xhtml `insertAdjacentHTML in HTML` XML 域脸（挂账保持）。
- sanitizer **SET-IDENTICAL**（42 键零 diff、P=415/F=341/T=1 恒等）。
- WC 17687P/1118F → 17972P/1120F＝+285P/+2F **全限定截断抖动族**（gethtml +284P 页预算回摆、opt-in ±3）——零行为面漂移。
- dom 57341P/496F/12T → 57487P/496F/12T＝+146P **F/T 恒等**全限定 Range-mutations 双页截断抖动（+51/+95，±45P 已知噪声带）——零行为面漂移。

## 常驻钉测

`crates/engine/src/js_dom_bridge_tests/part155_s10k9_domparsing_knife9_pins.rs`（include 于 js_dom_bridge_tests.rs）六面：①Fix A 兄弟链（四 position 二深链＋script 不执行＋连续插入基底不回退）②Fix B（SyntaxError ×2＋NMA ×3＋beforeend/afterBegin 次序对照）③Fix C（isEqualNode 等价/异输入＋ownerDocument）④Fix D（appendChild ccf 产物 script 执行＋innerHTML 章对照）⑤剥壳（head/body 计数恒 1 ×3 面）⑥testEquivalence 精确镜像（body 壳等价）。`cargo test -p zero-engine --lib test_s10k9_domparsing_knife9_pins` 1 passed。

## 电池判读（锚＝k8 v3 日志）

锚 md5：domparsing `492dc92343e774df4272e0e6a488e627`（1264P/271F/1T，80 键）、sanitizer `3e2fc2eb4651a8801a4535088f4345f7`（415P/341F/1T，42 键）、dom F-恒等 496、WC 17687P/1118F/20T。

## 终态电池 v2（ee76f4b5 二进制，14:22-14:37Z）

| 腿 | 锚 | v2 | 判读 |
|---|---|---|---|
| domparsing | 1264P/271F/1T | **1312P/223F/1T** | **净 +48P/−48F**：insert_adjacent_html 13P/18F→**31P/0F 全页收敛**、createContextualFragment 24P/11F→**35P/0F 全页收敛**、insert-adjacent 2P/2F→**4P/0F 全页收敛**、insert_adjacent_html-xhtml.xhtml 12P/18F→29P/1F（**外溢 +17**——剥壳/text/两轮序对 XML 域 IAH 同样生效）；残 1F＝`insertAdjacentHTML in HTML`（xhtml 文档 HierarchyRequestError 域，XML 管线挂账保持） |
| sanitizer | 415P/341F/1T | 415P/341F/1T | **键级 SET-IDENTICAL**（42 键零 diff）；行级唯一差异＝ccf 两条 script 脸互换翻转：`runs scripts by default` 锚 F→v2 **真 P**（text 融合读修复兑现）＋`createParserOptions(runScripts=false) does not run scripts` 锚**假 P**→v2 真 F（锚的不执行由 text 读 bug 静默实现＝假绿；修复暴露 runScripts options 章缺口——run_method 语料 JS 不传 options、ccf 无第二形参，ParserOptions 全链〔policy→sink→ccf→章〕属 sanitizer/TT 独立簇**挂账**） |
| WC | 17687P/1118F/20T | 18562P/1120F/20T | diff 全限定 3 键**页预算截断抖动族**（gethtml 3622→4496＝+874P 回摆、declarative-shadow-dom-opt-in 7F→9F/108P→109P）；F/T 恒等——零行为面漂移 |
| dom | 57341P/496F/12T | 57452P/496F/12T | diff 全限定 2 键 Range-mutations 截断抖动（dataChange 916→938、replaceData 859→948）；**F=496/T=12 恒等**——零行为面漂移（两轮序共享面无附带损伤实证） |

判读方法：逐页 (status,page) 键 join 对账（`LC_ALL=C awk` 聚合＋comm，脚本 `.acceptance/dps/s10k9-judge.sh`，自校验锚对锚零 diff）。注意 grep 行级对个别日志有静默零输出失效（bisect0 复核改用 awk）。

## v1→v2 差量（5c13109d→ee76f4b5）

handle-only 容器臂盖章（§6 Fix G）单件：IAH 残 1F→0F、xhtml 2F→1F；其余三腿键级恒等（sanitizer 行级亦恒等——v1/v2 sanitizer md5 逐字节同 `62775dc882beb31803edc994453fa90c`）。

## 挂账

1. `insert_adjacent_html-xhtml.xhtml` 残 1F：`insertAdjacentHTML in HTML`（xhtml 文档 XML 域 HierarchyRequestError）——XML 管线深项既有挂账域。
2. ccf `runScripts=false` ParserOptions 章（sanitizer/TT 簇）：run_method 语料 JS 不传 options、shim ccf 无第二形参、ParserOptions 消费链（part12 policy→sink→ccf→产物章）整链缺——独立刀位。
3. IAH handle-only 协议突变（getter 页 TO 族既有）、XML 域 ~200F 既有挂账——不动。
