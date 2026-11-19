# ③' forecolor 嵌套色壳 compare 7 行归因（只读窗——零源码修改）

日期：2026-11-05　证据源：/tmp/qv-fix4/fix12b-forecolor.log（fix10 基线同 17F，fix12b 零漂移）　语料权威：editing/include/tests.js :955-956 + include/reset.css :13

## 17F 结构

forecolor 现行 17F = **compare 7（③' 本窗）+ 读数面 10（④ 族域）**。7 条 compare
按机制四分（⑪ 轮「purple=样式表有效值 + ghost br」两分法的细化）：

| 组 | 行（log） | fixture / 模式 | 机制 |
|----|-----------|----------------|------|
| a | :1179/:1292 | `<span blue>foo<span brown>[bar]</span>baz</span>` sw=false / `<font color=blue>foo<font color=brown>[bar]</font>baz</font>` sw=true | 全含冲突壳摘除后等价早退漏包 + 外层等值壳不分裂 |
| b | :1341 | `<span blue>foo<span brown>b[ar]</span>baz</span>` sw=false | tag 模式 peel 残段 carry 形制 |
| c | :1394/:1412 | `<span blue>foo<span id=purple>b[a]r</span>baz</span>` 双模式 | 作者样式表有效值盲区 |
| d | :2292/:2309 | `<span brown>fo[o</span><span blue>b]ar</span>` 双模式 | ghost br（= ⑦ 时代挂账「wrap br 插入面」的 forecolor 语料形） |

## 机制定谳（got/expected 实证）

**③'-a 等价早退漏包 + 外层等值壳不分裂（2 行）**：全选冲突内壳（brown）摘除后，
幸存外壳（blue）使选中面有效值已等于新值 → 引擎等价早退跳过包裹（got
`<span style="color:rgb(0, 0, 255)">foobarbaz</span>` / `<font color="blue">foobarbaz</font>`，
壳内三段并拢）。Chrome expected = **新 font 壳照常物化 + 外层等值壳分裂**：
`<span blue>foo</span><font color="#0000ff">bar</font><span blue>baz</span>`（sw=true 族
expected 双形含 `<font style=color>` / `<font color=blue>foo</font><span blue>bar</span><font color=blue>baz</font>`，
assert_in_array）。规则面：**祖先壳命令等价面 == 新值 → 不并值、分裂外层让新壳落顶层**；
与 ⑫ Arm1 bg-merge 并值规则（祖先壳缺该命令面 → 并值入载体）互补成对。

**③'-b tag 模式 peel 残段 carry 形制（1 行）**：头段残迹 "b" 的 carry，expected
`<font color="#a52a2a">b</font>`——**span+rgb 形重发为 legacy font + 关键字 hex 归一**
（brown→#a52a2a）；got 保留原壳形 `<span style="color:rgb(165, 42, 42)">b</span>`。
sw=true 对应 fixture 无红 → CSS 模式 span carry 形引擎已对；差异仅在 tag 模式重发形制。
与 ⑫ Arm2 carry-clone（fontsize 载体克隆保原属性）不冲突——本行载体是普通冲突壳非
多属性载体，Chrome 在 tag 模式对残段 carry 走 font 重发。

**③'-c 作者样式表有效值盲区（compare 2 行 + ④ 读数 6 行同根）**：reset.css :13
`#purple { color: purple }`（`<link>` 作者样式表，全 run 壳页加载）。⑦ `_zwPlainComputed`
自面走查只认自身 style/font 属性 + UA 规则，**不查作者样式表** → `#purple` 元素
自面无声明：外层有 blue 壳时继承 blue（got `rgb(0, 0, 255)`）→ apply 面等价早退
跳过包裹（got `…<span id="purple">bar</span>…` 无包裹，expected 内层包
`<span blue>a</span>`/`<font color=#0000ff>a</font>`）；无外层壳时回落默认黑（④ qV
got `rgb(0, 0, 0)`，expected `rgb(128, 0, 128)`；qI expected true got false 同根）。
修复方向：自面走查补**扁平作者样式表查找**（reset.css 全规则均为无后代组合器的简单
选择器：元素 `b,strong`/`dfn`/`blockquote`/`quasit`、类 `.bold`/`.notbold`/`.underline`/
`.line-through`/`.underline-and-line-through`、id `#purple`、伪类 `:link,:visited`——
有界集合）。波及面：reset.css 全 run 壳页共享，qV/qI/qS 读面与 apply 等价判全面
换基，须整语料门禁窗；`quasit { text-align: inherit }` 与 ⑥ justifycenter 族谱相关，
修 ③'-c 时应一并核 ⑥ 的 quasit fixture 读数。

**③'-d ghost br（2 行）**：跨元素选中（brown 尾 + blue 头）merge 后 got
`<span blue>o<br>bar</span>`——"o" 并入 blue 壳正确，但 **多插一枚 `<br>`**（双模式
got 同形；sw=true expected `<span blue>obar</span>`；sw=false expected
`<font color="#0000ff">ob</font><span blue>ar</span>` 另含 tag 模式 font 重发面）。
= ⑦ 时代挂账「wrap br 插入面」残差（RUNSTATE xel fixture `o<br>bar` 同形）在
forecolor 语料的落点，同根同修面。

## 判读与门禁预记（下轮实施窗）

四机制独立可分刀：③'-c（样式表查找，影响读+apply 双面、波及全语料）建议独立窗
先行整语料门禁；③'-a/③'-b/③'-d 局部于 apply/peel/merge 面，刀后跑 forecolor 五
模式 + multitest 守卫腿。实施窗须避开并行轮 fix14（teal [a]bc）门禁窗口后再开。
