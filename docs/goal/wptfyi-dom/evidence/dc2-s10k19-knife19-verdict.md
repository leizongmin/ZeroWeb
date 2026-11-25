# DC-2 切片 10 DOM 刀⑲ 判读证据（TT script 身份门 provenance 快照等值模型）

日期：2026-11-25 · 判读域：trusted-types/script-enforcement-001..004 族（master.md 下一步 A 选项）
码刀：`a1bddb52a`〔rebase 前 13f57fa56〕（R521 TT script 身份门）＋判读补丁 `fe1ee9ed1`〔rebase 前 a4a1f1bae〕（CCF parser 投递信任 + registry splitText live-range retarget）

## 刀体（a1bddb52a〔rebase 前 13f57fa56〕）

TT provenance snapshot-equality 模型：script 信任是**身份**不是值——快照只在两处盖章：

1. **声明 sink setter 成功**（part14 §6 HTMLScriptElement.prototype text/textContent/innerText accessor；spec enforcement-in-scripts）；
2. **parser 投递**（part06 `__zw_stamp_parser_scripts`，全部解析期 script 含 type=unknown；FIXME document.write 中段）。

快照＝盖章时点 fused 文本（`_zwTtScriptFusedText` el.text→textContent 同源融合）；任何后续文本变更（无论写面）自动失配。门＝`_zwTtScriptExecGate`（part03 `_zwScriptMaybeStartDynamic` prepare 时点，空体检查后）：undefined=可信照常执行 / string=救援（返回串替换执行体、树文本不变）/ null=阻断（不执行**不置 run-once 记账**——spec prepare 即置 already-started，阻断仍置章防对臂重复执行）。R387/_r377/s97ib/s4p 副本执行臂以 run-once 表为门。

- part14 §6.2b `_zwTtChildTextFused` 自足 childNodes 走（R93 原型回落递归防护）；§6.4 原型分层安装（HTMLScriptElement 链接＋Node.prototype.textContent / HTMLElement.prototype.innerText **通用** descriptor——语料 001/002 `getOwnPropertyDescriptor` 直调面）；§6.3 管道异常（无政策 TypeError/回调抛错/返 undefined）一律归阻断，不向页面抛（violation report 挂账）。
- part04 R521 派发臂（R98 前）：accessor setter 经 `__zwTtAccBypass` **计数器**旗（嵌套写不丢旗）+ p∈{text,textContent,innerText} + SCRIPT + XHTML ns → 原型链走首个含 setter 的 descriptor；SVG script + 原型缺席保 legacy 臂（sink 分型 'SVGScriptElement text'）。
- part06 本地文本视图变异族补法（语料 001/002 "script.firstChild.splitText is not a function"）：splitText（IndexSizeError 越界抛；head 写回 + tail 兄弟插入条目 `nodes` 数组升级，_zwLocalChildNodes R5356b 多节点形消费）/ remove / replaceWith / before / after（外部节点 `_k19Carrier` 自属载体包装——createTextNode 代理 set-trap 派发落不进 plain 字段）+ `_k19SyncWritten` 条目 fused 回写 `_zwTextWritten`。
- part164_s10k19 钉测 v1 11 段（声明 sink 保信/通用 descriptor bypass 阻断/createHTML 落树阻断/splitText 保信/nodeValue 失配阻断/cloneNode 丢信/default policy prepare 救援序/SVG 救援/setter 期管道序/before/after 变异救援）。

## solo TT 判读（二进制 `edebb8652f98d4a60335c4a7727f3ecf`，HEAD 13f57fa56）

锚＝k18b v2 终态 /tmp/s10k18b-tt-v2.log（F 集 320；raw Fail 行 362）。发射器 .acceptance/dps/s10k19-tt-solo.sh（EXPECT_MD5 断言），日志 /tmp/s10k19-tt-solo2.log：

- **P2F 0 / GONE 85**（F 集 320→235；Pass 1728→1805）
- 逐页：001 **27F→0F 收官**；002 30F→3F；003 16F→5F；004 31F→20F；005 −2；006/007/014 各 −1；secondary-document −2；block-text-node-insertion(html/svg) −2
- 暂缓域（刀⑳ 料）：002 残 3F（setHTMLUnsafe + replaceChild + insertAdjacentText）；003 残 5F／004 残 20F（SVG created-script 子视图 + parser-script splitText 域）

钉测 13 段全绿（v2 补 ⑫⑬）；fmt RC=0；clippy engine lib 0 warning；全 shim 拼接 node --check OK。

## 电池 v1 判读（同二进制 edebb8652，HEAD 13f57fa56）——两处 P2F 定谳

发射器 .acceptance/dps/s10k19-battery.sh（EXPECT_MD5+EXPECT_HEAD 断言），05:14:14Z 发射 05:46:15Z 收：

| 腿 | 锚（k18b） | v1 | 判定 |
|---|---|---|---|
| WC | F 1118 | F 1116，P2F 0，GONE 2 | ✓ |
| DP | F 222 md5 `f70f8f97335497bfb0494833669bb208` | md5 逐字节恒等 | SET-IDENTICAL |
| SAN | F 50 | **F 51＝P2F 1** | ✗ 修复 |
| TT | F 320（F 集） | F 235，P2F 0，GONE 85 | ✓ |
| DOM | F 364 | **F 386＝P2F 22** | ✗ 修复 |

**P2F ①（SAN）**：sanitizer-api/sethtml-with-trustedtypes.tentative.html「Element: createContextualFragment runs scripts by default」——CCF 产物是 `_zwMBuildDeepEntry`/`_zwMBuildDeepTreeNode` 工厂代理（无 sel/handle，R368 身份在插入时点才赋予），身份门快照表无法键入 → prepare 按不可信处置；页面 default policy 只有 createHTML/createParserOptions 无 createScript → 救援 TypeError → 阻断 → `window.did_run` false。真实浏览器语义：CCF 内部走 HTML fragment 解析、脚本文本由 parser 投递 → **信任直执行**（对照 innerHTML/DOM API 新建脚本无投递 → 照常救援/阻断）。

**P2F ②（DOM ×22）**：dom/ranges/Range-mutations-splitText.html 22 案 Pass→Fail——common.js `paras[x].textContent=` 建的注册文本节点，k18b 基线 splitText 缺法（harness `catch` 分支简易通过＝假绿），刀⑲ 补法后真判读面开启但缺 spec concept-text-split 末段 live-range retarget（failing 脸两类：父边界 offset 未 +1（"expected 2 but got 1"）＋节内边界未移尾节点（"expected Text 'b̈c̈…' but got 'Ä'"））。判读纪律注：逐标题 join 须以「页面路径+标题」全串为键——Range-mutations 双页共享 harness 标题集，仅按标题键 join 得 flips=0 假阴（双页互相覆写掩蔽）。

## 补丁（fe1ee9ed1〔rebase 前 a4a1f1bae〕，3 文件 +141/−1）

1. **CCF parser 投递信任**：part06 CCF 构建期 walker（镜像刀⑩ `_ccfStampK10` 先例，TT enforced 门 + `__zwFragNoExec` 排除）＋part14 §6.0b `_zwTtScriptTrustPlain` WeakMap（对象身份寄存；唯一写入方＝CCF walker）。gate 兜底臂在 **sel/hdl 双表未中时**查 plain 表——不能以 sel/hdl 键缺席为前置条件（插入后对象已获键、键在表空）。
2. **registry splitText R261 镜像 retarget**：原节点边界 offset > split 点 → 移尾节点 offset−=split（harness 相位一 clamp 与末段 move 合并语义＝move 用 original offset，clamp 值仅在 parent 为 null 时是终值）；父边界 offset ≥ 1+idx 恰一次 +1（harness 相位二 `>` 与终段 `==` 合并）。条目字面量恒等直比；父容器 el 为 setter 期新建代理、页面持有的 paras[x] 是另一实例 → sel/handle 键串比较（R261 同款防单次 get trap identity 不稳）；偏移权威面 `_startOffsetBase`/`_endOffsetBase`（R183 Range getter 直读）。挂 `globalThis.__zwLiveRanges` 全表。
3. 钉测 ⑫（retarget 三态：节内移尾/父边界 +1/前段不动）＋⑬（CCF parser 投递直执行：default policy 在场无救援、原串保留非 RUNX 改写）→ 13/13。

## 电池 v2 终判读（二进制 `31fd0c73d3f95e3b3d2e0affbfb27286`，HEAD a4a1f1bae）

发射器 .acceptance/dps/s10k19-battery-v2.sh（EXPECT_MD5+EXPECT_HEAD 断言，test-guard 包裹），05:55:51Z 发射 06:26:58Z 收（31 min）：

| 腿 | 锚 | v2 | 判定 |
|---|---|---|---|
| WC | F 1118 | F 1116，P2F **0**，GONE 2 | ✓ |
| DP | md5 f70f8f97 | md5 **逐字节恒等** | SET-IDENTICAL |
| SAN | F 50 | F 50=50，P2F 0，GONE 0，P 706=706 | SET-IDENTICAL（① 修复兑现） |
| TT | F 320 | F 235，P2F **0**，GONE 85 | ✓（GONE 保位） |
| DOM | F 364 | F 364=364，P2F **0**，GONE 0，P 57313→57417 | SET-IDENTICAL（② 修复兑现） |

- splitText 页 116 标题逐串 join 与基线同判 0 diff（22 案全翻回 Pass）；enforcement-001 0F 持位、002 残 3F 持位。
- WC GONE 2：custom-elements/builtin-coverage「script: innerHTML should instantiate a customized built-in element」＋shadow-dom/declarative/declarative-shadow-dom-opt-in「XMLHttpRequest document.open()…」。
- WC P 18260→17685 漂移＝F 集零差（P2F 0/GONE 2）下截断落位族，非回归（dom-leg 同款纪律）。

## 账目

- **刀⑲ 净增 +87**（TT GONE 85 + WC GONE 2；DP/SAN/DOM SET-IDENTICAL 不计）→ **累计 83,670 / 107,971**，距 ≥99,947 差 **16,277**。
- 判读绑定：solo/v1 二进制 `edebb8652f98d4a60335c4a7727f3ecf`（提交 a1bddb52a〔rebase 前 13f57fa56〕）、v2 二进制 `31fd0c73d3f95e3b3d2e0affbfb27286`（提交 fe1ee9ed1〔rebase 前 a4a1f1bae〕）。
- 日志：/tmp/s10k19-{tt-solo2,wc,domparsing,sanitizer,tt,dom}.log（v1）、/tmp/s10k19v2-{wc,domparsing,sanitizer,tt,dom}.log（v2）；主账 /tmp/s10k19-battery{,-v2}-main.log。

## 刀⑳ 挂账（本刀暂缓域）

- 002 残 3F：setHTMLUnsafe script 保信面 + replaceChild + insertAdjacentText 变异域；
- 003 残 5F／004 残 20F：SVG created-script 子视图（_zwRegisterTextEl SVG 臂）＋parser-script splitText（004 语料 `create_svg_script_with_trusted_source_text` parser-script 复用面）；
- 次选：WC adopted-callback 66F（custom-elements upgrade 序，原 master 下一步 B 选项）。
- 卡点（不属本刀）：WPT_REV 漂移 bump（跨流共享面，DC-3 前统一处理）。
