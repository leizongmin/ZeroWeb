# style-with-value 簇值查询面收口——gCS plain 四属性解析器 + 五值命令接线（2026-11-01）

## 0. 结论速览

| 门禁 | 本轮（gate6 终二进制） | 轮前基线（gate3） |
|---|---|---|
| fontsize 真页 | **2,028P/394F**（+161P） | 1,867P/555F |
| forecolor 真页 | **1,960P/367F**（+80P） | 1,880P/447F |
| fontname 真页 | **1,801P/424F**（+134P） | 1,667P/558F |
| backcolor 真页 | **803P/223F**（+67P） | 736P/290F |
| hilitecolor 真页 | **970P/248F**（+79P） | 891P/327F |
| 五语料合计 | **7,562P/1,656F**（+521P） | 7,041P/2,177F |

逐语料 A/B：F→P=169/88/188/76/88（计 **609**）、P→F=8/8/54/9/9（计 **88**）、Timeout 0→0。88 条 P→F 全数定谳为 **apply 缺口②下游**（§5.2），零纯值面回归。join 位移语料 7,518P/0F/0T 逐位恒等、七 disp 臂全 exit=0、wcomp 红名集合 gate3↔gate6 **完全一致**（1,135=1,135，diff 空）——gCS 四属性 plain 回落对 wcomp 零影响。hsem/engine lib/webview lib/bench-gate 终数见 §5.3。

上轮（[setup-exception 族收口](2026-11-01-sv-setup-exception-mstyle-idl.md)）清掉 `reading 'color'` setup 例外后，本轮把五语料剩余失败主体——`queryCommandValue` 恒 `''`/`queryCommandIndeterm` 恒 `false` 桩——接通。

## 1. 现象与立项

五语料每案 6 条 query 断言行（Indeterm/State/Value × before/after）中，引擎面 `document.queryCommandValue` 五值命令恒返 `''`、`queryCommandIndeterm` 恒返 `false`（part06.js 旧桩），before 列（Chrome 期望值面读数）整族 Fail。上轮收口后该族是五语料失败主体。

## 2. 归因链

### 2.1 语料比较列真相（关键定谳）

`include/tests.js` 断言三列，此前误判「vendored 直呼其函数服务一切」：

- `tests.js:5595/5627` before/after 列读 **`document.queryCommandIndeterm/State/Value`（引擎面）**——引擎面桩恒 false/''，before 列整族挂；
- apply 列与 spec 列走 vendored `myExecCommand`/`myQueryCommandValue`（`tests.js:5010` 直呼）——vendored 值面本来就在。

gate4 批五语料**零变化**实证：只接 vendored 面不移动断言列；最终双轨都接（vendored 面此前已通，本轮补引擎面）。

### 2.2 探针两轮（gitignored 临时页，不入库）

**探针一 `zz-probe-qv.html`**（t1–t5）：

- t1/t2：M 形 plain 元素（font/span）gCS 四属性全 miss——host gCS 通道经 `__zwSelector/__zwHandle` 只覆盖 wrapper 形（`__zwIsMEl` 形无 selector/handle，通道不可达）→ **plain 计算值解析器缺口**；
- t3：裸文本五命令 eng 恒 ''/spec 正常——引擎面桩缺席；
- t4：font[size] 1-7 → px 表底数核对（Chrome 口径 10.08/13.12/16/18.08/24/32/48px，parseInt 取整 10/13/16/18/24/32/48）；
- t5：语料 setupDiv 全流复刻后 gCS/qV 形状——mixed indeterm 取样。

**探针二 `zz-probe-qv2.html`**（tA–tF，host 通道语法覆盖）：

- tA：颜色 12 形态（named/hex 三态/rgb/rgba/hsl/hsla/transparent/currentColor）host 序列化形态全对，唯一 brown 缺口（host 色表缺 brown，渲染流面，未动）；
- tB/tC：font[size] hint 与 font-size 声明族（pt/em/%/关键字）px 换算口径；
- tD：裸 font-family="serif"、UA mono 族（kbd/tt/code/... → monospace）；
- tE：裸 background-color="rgba(0, 0, 0, 0)"、透明色上行；
- tF：vendored normalizeColor（`implementation.js:415`）经引擎 gCS 探针 span 归一两侧颜色——**值面 state/indeterm 是原始串等值比较（无归一）**，故解析器必须输出 Chrome 序列化形态（`rgb(r, g, b)` 逗号式、alpha=1 时 rgb() 形）。

## 3. 修复（三文件，均在 `crates/engine/src/js_dom_shim/`）

### 3.1 part01c.js 块一：`__zwPlainComputed(elt, prop)` plain 计算值解析器

四属性 color/font-size/font-family/background-color 的 plain 形解析（kebab prop）：

- `_zwPlainNamedColor`：全 148 色 CSS named 表（name→`'r, g, b'` 串，含 brown）；
- `_zwPlainFontAttrKw`（font[size] 1-7→x-small…xxx-large）+ `_zwPlainFontKwPx`（xx-small:9 … xxx-large:48，Chrome 口径）；
- `_zwPlainUaMono = { tt, code, kbd, samp, pre, listing, xmp, plaintext }`——UA 规则层 font-family: monospace（二次修复补入，§4.3）；
- `_zwPlainStyleDecls`（style attr 朴素扫描）+ `_zwPlainDecl`（style IDL getPropertyValue 优先、attr 扫描兜底）；
- `_zwPlainHslToRgb` + `_zwPlainColorSpec`：transparent→`rgba(0, 0, 0, 0)`、currentColor→父链递归、named/#hex/rgb(a)/hsl(a)→Chrome 序列化（alpha=1→rgb() 形）；
- `_zwPlainFontSizePx`：px/pt/pc/in/cm/mm 直换、em/%×父 px、关键字表、font[size] attr；
- 入口走查：font-size 继承链 + r3 序列化；color/font-family/background-color 走查——font 表现提示 color/face/bgcolor、UA mono 臂、bg 透明继续上行；缺省 `rgb(0, 0, 0)`/`serif`/`rgba(0, 0, 0, 0)`。

FIXME 挂账（块头注释）：仅逗号式 rgb()/空格式语法与 font 简写未解析；smaller/larger/rem 未实现；CSS 系统色未覆盖。

### 3.2 part01c.js 块二：`__zw_ce_value_query(cmd)` 引擎值面

vendored `implementation.js` 忠实移植：

- `_zwValIsEffectivelyContained`（:1807 isEffectivelyContained：collapsed false、isContained、start/end TEXT 臂、全子递归+端偏移守卫）、`_zwValGetAllEffectivelyContained`（起点爬升 256 守卫+文档序走查 65536 守卫）；
- `_zwValIsFormattable`（editable+visible+(TEXT||img/br)，:1997 isFormattableNode 对齐）；
- `_zwValLegacyFontSize`（:3309 getLegacyFontSize：`_zwValLegacyBounds=[10,13,16,18,24,32]` 中点映射返 '1'-'7'）；
- `_zwValEffectiveValue`（:2069 getEffectiveCommandValue：TEXT 归父、backcolor/hilitecolor transparent 爬升、fontsize→font-size/fontname→font-family/else color）；
- 入口（fontsize/fontname/forecolor/backcolor/hilitecolor 五命令）：选区取 `globalThis.getSelection` 惯用法（`_getSelection` 声明域不可见，part01c 文件头既有约定）、`sel._ranges[0]`；`getAllEffectivelyContained(rng, isFormattable)` 取节点集，>1 互异判定→indeterm，值取首节点（无则 startContainer），fontsize 过 legacy 映射。

FIXME 挂账：**值 override 机制未接**——vendored myExecCommand 的 override 写私有闭包 map 引擎不可见，collapsed 选区 qV-after 残余（Chrome 期望来自其原生 override），族挂账待后续轮。

### 3.3 part01.js：gCS 两钩子（plain 回落）

- `getPropertyValue` 尾：query 空时 `__zwPlainComputed` 兜底；
- Proxy get 尾：`_camelToKebab(p)` 归一后判四属性 kebab 名回落（`_zwJusCs` 内部机具以 kebab 名访问 gCS Proxy get，vendored 以 camelCase——不归一则 fontsize/fontname 链断，§4.2）。

`--` 自定义属性跳过；异常静默回落原通道。

### 3.4 part06.js：两查询面接线

`queryCommandIndeterm`（justify 臂后）与 `queryCommandValue`（defaultparagraphseparator 臂后）各加五命令分流 `__zw_ce_value_query`；:2330 陈旧注释更新为接线记录（formatBlock 等其余值面未接，documented）。

## 4. 中途修复（三轮）

1. **探针 qV 口径错**：初版探针调引擎面恒 ''→改 vendored 验链路通→gate4 五语料零变化定谳比较列真相（§2.1），引擎面+vendored 双轨都接；
2. **`_getSelection` 闭包不可见**：值面首版直呼 `_getSelection()` 取区落空返 ''（forecolor 意外通是 rng null 时 '' 恰合期望）——改 `globalThis.getSelection` 惯用法；
3. **UA monospace 缺失（真分歧）**：fontname 2 条 before 行 P→F（`<span style=font-family:monospace>fo[o</span><kbd>b]ar</kbd>` Chrome false 我方 true）——数据挖掘 BEFORE-TRUE 行集定谳：kbd/tt/code 等 UA mono 族行 Chrome indeterm=false（两端 computed font-family 同为 monospace）。补 `_zwPlainUaMono` 表后 fontname 1,756→1,801P（**+45** 连带解锁 kbd 族 qV）。

## 5. 门禁

### 5.1 五语料终数（gate6 终二进制全量；UA mono 修复落地后 gate5 在途批作废，证据绑最终工件）

见 §0 表。A/B 逐行比对：F→P=609、P→F=88、Timeout 0→0。

### 5.2 P→F 定谳（88/88 = apply 缺口②下游，零纯值面回归）

python 逐行关联（needle = `<params+html> compare innerHTML`，此前两轮 grep 关联因行尾语义截断与双引号追加错判，作废重做）：

- **85 条**：同 case `compare innerHTML` 行同 Fail——apply 分歧 DOM 上恒 false indeterm 旧桩巧合命中 Pass、新诚实读数翻真，非值面回归；
- **3 条**（fontsize `foo<font size=2>ba[r</font>b]az` / forecolor `fo[o<font color=brown>bar</font>b]az` / fontname `[foo<pre>bar</pre>baz]`）：跨分片 carry-over 伪噪声——该三 case 的 compare 检查在早分片（`?1001-2000`/`?1-1000`）运行且 Fail，晚分片（`?2001-last`/`?1001-2000`）只跑查询行，apply 分歧 DOM 状态跨分片持续到 after 读数。

### 5.3 其余门禁臂

- join 三案 7,518P/0F/0T 逐位恒等；七 disp 臂全 exit=0；
- wcomp：Fail 红名集合 gate3↔gate6 **完全一致**（1,135=1,135，diff 空）——gCS 四属性 plain 回落零影响；
- hsem / engine lib（基线 3,642P/1F s24 已知红唯一）/ webview lib（基线 834P/0F）/ bench-gate（zero-engine 定向）：终数见 gate6 日志与本轮 goal doc 记账。

### 5.4 残余分解（1,656F）

compare innerHTML 族（apply 缺口②）下游为主；collapsed 选区 override 族（§3.2 FIXME）；表 fixture IndexSizeError（族③）；state 面 25/语料未动。

## 6. 证据文件

- gate3/gate6 五语料+join+wcomp 日志：`/tmp/zw-edit/gate{3,6}-{fontsize,forecolor,fontname,backcolor,hilitecolor,join-*,wcomp}.log`（本轮收口后随 tmp 清理失效，终数已录 §0/§5 与 goal doc）；
- P→F 行集与关联脚本输出：`/tmp/zw-edit/pf-*.txt`（88 行）；
- 探针页两枚：`tests/wpt-runner/wpt-data/editing/run/zz-probe-qv{,2}.html`（gitignored 临时页，按惯例保留不入库）。

## 7. 合并树复测（gate7，rebase 后，本文件入库时补账）

轮内 gate6 全批收口后、入库前 `git pull --rebase` 并入远端 14 笔（PR #265 softnav busy-loop engine bridge 7 文件 +1,127/−77 含 part94 +413 钉测、s199 runner、s200 runner、s201 engine hit_test 3 文件、s202 form IDL、shim part03 +88/part05 +38），systemd 单串行批全电池复跑（日志 `/tmp/zw-edit-gate7/`）：

| 门禁 | gate7（合并树） | 对 gate6 | 位移 |
|---|---|---|---|
| fontsize / forecolor / fontname / backcolor / hilitecolor | 2,028P/394F、1,960P/367F、1,801P/424F、803P/223F、970P/248F | 同 | **逐位恒等** |
| join 三案 / ihost / wss / ilb / chd / void×2 | 3456+2607+1455=7,518P/0F/0T、880P/0F、1,104P/0F、552P/0F、420P/0F、72P/0F、48P/0F | 同 | **零位移** |
| wcomp | 4,022P/1,146F/5T | 同 | **计数级恒等** |
| engine lib | 3,664P/1F（s24 已知红唯一，红名 identical；P 3,642→3,664 并行流 part94 新增测试面） | 已知红唯一 | 达标 |
| webview lib | 834P/0F | 同 | **恒等** |
| bench-gate（zero-engine 定向） | PASS（absolute budgets；baseline 硬件 WARN 恒在） | PASS | 恒等 |
| hsem | **6,895P/670F/162T**（总 7,727 恒等） | 前账 6,872P/693F | +23F→P 正向漂移，见下 |

**hsem +23 归因（pristine A/B + 定向复检三连）**：裸 HEAD（5ee3d31f9，无本 diff）hsem 6,894P/671F/162T——+22 系吸收的并行流修复面（s199 button-activate-keyup-prevented / s200 keyboard-click / s201 svg-hittest / s202 legend-option-form-IDL，html-semantics 流自证 keyed diff），与本 diff 无关；差值内净 +1P/−1F 定谳为 `:active` 时序 flakes 家族（`checkable-active-onblur.html`、`checkable-active-onblur-with-click.html`、`inert-svg-hittest.html` pointer/keyboard 按住态断言）逐跑 ±数行漂移：**同 diff 两跑（gate7 vs 定向复检）间漂移行数大于 pristine↔diff 差值**（"following label" 对与 onblur Radios 在 gate7 P、复检 F，代码同态），且 `inert-svg-hittest` 定向复检 **3/3 全绿**（本 diff 在树）——非本 diff 回归，族级挂账归 html-semantics 流（其控制面记录）。

**残余失败族终普查（gate7 五语料 1,656F）**：compare innerHTML **567**（apply 缺口②，下轮主刀）+ queryCommandValue-after **652**（collapsed 选区 override 族，§3.2 FIXME）+ Indeterm **214** + State **125** + other **98**（表 fixture IndexSizeError setup 形等独立面）。
