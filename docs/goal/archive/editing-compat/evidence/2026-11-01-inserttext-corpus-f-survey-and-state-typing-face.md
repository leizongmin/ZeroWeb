# insertText 波及语料首测 F 族普查 + 状态命令 typing 面（sub/sup + qState fallback）

日期：2026-11-01（editing 流，控制面下一步 ① 普查轮 + F3/F4 落地，基于 gate13 干净二进制产物聚类）

## 0. 背景与范围

上一轮（s212 + inserttext 变异核，2675ec0b5）gate13 对 insertText 波及语料完成首测：
inserttext 2,777P/232F、multitest 9,331P/761F、insertparagraph 6,720P/475F、
insertorderedlist 1,438P/323F、insertunorderedlist 1,721P/344F（均无既有基线）。
本轮对五语料 761+475+323+344+232=2,135 F 做家族聚类（断言类型 × 命令签名 ×
expected/got 形态，产物 /tmp/zw-edit-v/g13-*.out），定 mid-command 缺口臂优先级，
并落地普查直接锁定的状态命令 typing 面（F3+F4）。

## 1. 五语料 F 族普查

### 1.1 inserttext（232F）

断言面：compare innerHTML 230 + qV/qState before 各 1。签名：单 `inserttext` 146、
`stylewithcss+inserttext` 49、`inserttext×2-4` 28、`defaultparagraphseparator+inserttext` 9。

形态抽样：`\n` 分段族仅 2（`foo[]bar` got `foo<p>bar</p>` vs expected
`<p>foo</p><p>bar</p>`——host-level split 前段不包裹，同 ipara 主刀机制，见 1.3）；
`abc\ndef` 双行族 got `fooabc\ndefbar` 字面 splice（同机制）；`&nbsp;` 空格族 2
（`foo ` vs `foo&nbsp;`——尾空格 nbsp 化）；`foobarbaz` vs `foobaz` 1（非空选区
空串 deleteSelection 前置，§5 挂账）；其余为白空间 style 容器内 typing 形态。

### 1.2 multitest（761F）

断言面：compare innerHTML 463 + qState-after 202（bold 25/subscript 38/superscript 34/
insertorderedlist 24/insertunorderedlist 24/italic 19/strikethrough 19/underline 19）+
其余 qState/qV/qIndeterm 零散。compare 主刀签名：`delete+inserttext` 60+17+7、
`insertparagraph+inserttext^n` 21、`forwarddelete+inserttext^n` 22、
`styleWithCSS+delete/forwarddelete+insertText` 16、`styleWithCSS+bold+inserttext` 6、
`createlink+forecolor+inserttext` 8。五族：

- **D1 delete 撤壳族（~100 compare F）**：`exp 'abaz' got 'a<a href…><font…'`——
  delete 跨包裹边界后壳未消费、typing 落进残壳。Chrome delete 语义独立轮。
- **D2 ipara 形族（~21 compare F）**：`exp '<div>foo<b>a</b>bar</div>' got
  'foo<b>a</b>bar'`——insertParagraph host-level split 前段不包裹（1.3 同机制）。
- **D3 状态命令 typing 族（sub/sup compare ~20 + qState-after 202 主体）**：
  `exp 'foo<sub>a</sub>bar' got 'fooabar'`（sub/sup 不在 format tag 表、collapsed
  臂 no-op 不入 typing 集）；qState-after 面两类——caret 未动类（`bold+
  insertorderedlist`、`bold+delete`、`bold+formatblock`：ttags 有 b 但 DOM 无包裹，
  读面只走祖先链恒 false，Chrome stateOverride 语义 expected true）+ 落点在包裹内类
  （修复 D3 tag 表后由祖先链自然命中）。**本轮 F3+F4 修复面，见 §3。**
- **D4 typing 尾 br 边界族（~12-24 F）**：`<div><b>abc[]<br></b></div>` typing
  'd' expected `<div><b>abc</b>d</div>`（assert_in_array 含 `<b>abc</b>d<br></div>`
  变体）、qState(bold) after expected false——caret 在包裹尾 br 前时 Chrome 弃包裹
  落位（`<b><i>abc[]<br></i></b>` 形 expected `<b><i>abc</i></b><i>d</i>` 出包 i 留
  b 造新 i，Chrome typing-style 重挂语义）；本实现 splice 在包裹内（got
  `<b>abcd<br></b>`）+ qState true。独立轮（Chrome 边界规则未建模）。
- **D5 形态矛盾族（挂账扩面）**：fontsize+sub 复合案实证矛盾扩到值命令——
  `[["fontsize","2"],…inserttext]` expected `foo<sub>a</sub>bar`（size 2 零包裹）、
  `[["fontsize","3"],…]` expected `<font size="3"><sub>a</sub></font>`（font 元素
  形），而本实现 flag-true（bootstrap hack）产 `<span style="font-size:small">`；
  与五值语料 flag-true→span 期望同构矛盾，multitest bootstrap `styleWithCSS=true`
  与录制环境不一致定谳维持，不 chase。

### 1.3 insertparagraph（475F）

compare 465 主刀单机制：**host-level split 前段不包裹**——`foo[bar]baz`（非坍缩）
got `foo<div>baz</div>` vs expected `<div>foo</div><div>baz</div>`；`foo[]bar` inserttext
`\n` 同形。table 跨块形 `fo[o<table>…<td>b]ar</table>` got `fo<div>oo</div>…<td>bar`
vs expected `<div>fo</div><div><br></div><table>…<td>ar`（选区入格删除面 + 空段
占位 br）。签名 `defaultparagraphseparator+insertparagraph` 364 + swcss 变体 69 +
单命令 37。**F2 修复面（独立轮）**。

### 1.4 insertorderedlist / insertunorderedlist（323F / 344F）

**list 命令零应用**：compare F got 与输入恒等（`foo[]bar` got `foobar` vs expected
`<ol><li>foobar</li></ol>`；iloop 150、illist 166 全族）；qState before/after 面
（iloop 103+40、illist 87+66）——命令不应用则 after 恒 false、before 部分形
（`<ol>` 已存在时 expected true）也因读面未接 list 命令而 false。qV/qIndeterm 零散
（iloop 29、illist 24）。**F1 修复面（最大单族，fresh 实现轮）**：branch3 需
insertorderedlist/insertunorderedlist 实应用臂（vendored insert list 算法 ~1,000
行面：toggle 语义 + 邻接 list 合并 + li 拆分 + 与 indent 交互）+ qState/qV 读面。

## 2. mid-command 缺口臂优先级定案

普查后 branch3 无实应用臂命令的语料权重（compare+qState 断言面）：

| 优先级 | 命令 | 语料面 | 量级 |
|---|---|---|---|
| F1 | insertorderedlist/insertunorderedlist | iloop/illist 全语料 + multitest 24+24 qState | ~600F |
| F2 | insertParagraph 前段包裹（split 语义非缺臂） | ipara 465 + multitest D2 21 + inserttext `\n` 族 | ~490F |
| F5 | typing 尾 br 边界（非缺臂，边界规则） | multitest D4 | ~24F |
| F6 | delete/forwardDelete 撤壳边界 | multitest D1 | ~100F |

F1/F2 为下一轮主刀候选；F5/F6 挂后续。

## 3. F3+F4 落地：sub/sup tag 表 + qState typing-state fallback

### 3.1 实现（part06 三笔）

1. `_zwExecCmdFormatTag` 补 `subscript: 'sub', superscript: 'sup'`——Chrome 对
   sub/sup 恒元素形（styleWithCSS 置位也产 `<sub>/<sup>`，multitest bootstrap
   flag=true 下录制仍 `<sub>` 实证），无 CSS 臂。
2. collapsed 臂 ttags 维护补 sub/sup 互斥——activating 一方移除另一方（spec
   subscript/superscript action 先关对侧；multitest 交叉序 expected
   `foo<sub>a</sub>bar` / `foo<sup>a</sup>bar` 实证）。二次执行 toggle-off 经 3 的
   读数翻转自然生效（`_zwQueryFormatState` 见 ttags → splice）。
3. `_zwQueryFormatState` 补 typing-state fallback——祖先链 miss 后查
   `__zwTypingTags` 含 tag 返 true（Chrome stateOverride 语义：collapsed 执行
   format 命令不改 DOM 但 qState 立即 true，typing 包裹态存活到 toggle 关或落点
   离弃；caret 未动类 `bold+insertorderedlist`/`bold+delete` 语料 expected true
   实证）。

### 3.2 探针（zz-probe-subsup.html，gitignore）

六案逐字命中语料期望：`sub+type`→`foo<sub>a</sub>bar`+qStateSub=true、
`sup-then-sub+type`→`foo<sub>a</sub>bar`（互斥）、`sub-then-sup+type`→
`foo<sup>a</sup>bar`、`bold+iol`→DOM 不变+qStateBold=true、`bold+del`→
`fobar`+qStateBold=true、`bold-only`→qStateBold=true。

## 4. 对账（gate14 + gate14b，systemd user unit 串行批）

### 4.1 gate14（F3+F4 v1，批 22:41-23:25）

| 语料 | g13 基线 | gate14 | Δ |
|---|---|---|---|
| multitest | 9,331P/761F | 9,484P/608F | F→P 155、P→F 2（双执行 underline 对，§4.2 修复） |
| insertorderedlist | 1,438P/323F | 恒等 | 本轮不动面 |
| insertunorderedlist | 1,721P/344F | 恒等 | 本轮不动面 |
| forecolor | 2,279P/48F | 恒等 | 五值对照 |
| wss-ilb / wss-ip | 552P/0F / 1,104P/0F | 恒等 | typing ttags 消费面零位移 |
| join 三案 | 3,456/2,607/1,455P/0F | 恒等 | 零位移 |
| chd / ipihost / void×2 | 420/880/72+48P/0F | 恒等 | 零位移 |
| run-ilb | 1,641P/50F | 恒等 | 零位移 |
| engine lib | rc=0 | **rc=0** | |

multitest 155 F→P 逐案归 D3 族（sub/sup compare + qState-after caret-未动类 +
落点在包裹内类）；P→F 2 = `underline first+second application` 双执行对——

### 4.2 双执行修正（Dom-split，gate14b）

gate14 暴露语义弯路：`_zwQueryFormatState` 的 ttags fallback 若同时进臂内 toggle
判定，第二次 collapsed 执行会关闭 typing 态；语料实证 Chrome **第二次执行不关**
（`[["underline","first"],["underline","second"],["inserttext","a"]]`
qState(underline) after expected true）。修正：拆分 `_zwQueryFormatStateDom`
（臂内 toggle 判定 + 非 collapsed wrap/unwrap 判定，纯祖先链）与
`_zwQueryFormatState`（qState 读面 = Dom + ttags fallback）。

gate14b（干净二进制，批 23:32-00:07）：multitest **9,486P/606F**——对 g13 轮初
基线 **F→P 155、P→F 0**；wss-ilb 552P/0F、wss-ip 1,104P/0F、join-paragraphs
3,456P/0F 恒等。探针六案复验逐字不变。

## 5. 残余与挂账

- F1 list apply 臂（~600F，fresh 实现轮）→ F2 insertParagraph 前段包裹（~490F）。
- multitest D4 typing 尾 br 边界（Chrome typing-style 重挂语义未建模）。
- multitest D5 形态矛盾族（挂账扩面到值命令 fontsize 复合案，不 chase）。
- inserttext `&nbsp;` 尾空格族、非空选区空串 deleteSelection 前置族（§1.1）。
- sub/sup 非 collapsed 交叉切换（无语料面，FIXME 未做）。
