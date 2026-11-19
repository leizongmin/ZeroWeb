# s212 handle 子 parentNode 反链 + isAllowedChild 名单收口 + inserttext 变异核与 typing 值 override

日期：2026-11-01（editing 流，forecolor 语料 w3/w4 嵌套包裹族 + table 族 + a[]c insertText 族三轮归因合并提交）

## 0. 背景与三族并发

上一轮（空白族收口，8,922P/296F）提交后残余普查锁定 compare innerHTML 167F 主刀族；
本轮对 forecolor 语料逐族归因，发现三族同源于引擎 DOM/编辑两层缺口：

| 族 | 症状 | 根因层 |
|---|---|---|
| w3/w4 嵌套包裹（`<span>foo</span> <span>bar</span>` 全选 foreColor） | 产出嵌套双 span 形，期望合并单 span | part04 appendChild/insertBefore handle 子臂缺 parentNode 反链 |
| table 族（`<table>` 内 foreColor 等） | `<font>` 包裹 tbody 非法形态 | part01d `_zwVaProhibitedParagraphChildren` 名单缺 table 族 + 缺 spec 终检 |
| a[]c insertText（`a[]c` + foreColor collapsed + insertText） | 插入零变异 / override 不落 | part06 branch3 缺 inserttext 实应用臂 + 值 override 无 typing 应用面 |

## 1. s212：handle 子 appendChild/insertBefore parentNode 反链（part04 两臂）

**归因链**（TAIL trace → gCS 对照探针 → live-path 日志 → MV 步进 → 对象形态日志）：

1. live 文档元素为 handle 背书 Proxy；`removeChild` 的 `child.parentNode = null`
   落 expando own 值属性。
2. get trap expando 优先于 `prop === 'parentNode'` 内置分支（part04:2094）→
   后续 appendChild 到 handle 父只更新 `_handleChildren` registry +
   `_zwNodeParent` 槽，stale expando null 遮蔽读面 → `parentNode` 恒 null。
3. span1 孤儿 → `_zwValEffectiveValue` gCS 空 → UA 默认 rgb(0,0,0) → vendored
   :762「eff ≠ newValue」成立 → UNDO（移回 + 删壳 + children 递归）误触发 →
   嵌套包裹形。

**修复**：appendChild trap handle 臂与 insertBefore trap refNode==null 臂补
`_zwForceParentLink(child, _makeProxy(sel, handle))`（R84 plain 子臂 / R245
MEl 臂同款对偶，defineProperty 强写可配置 parentNode）。

## 2. s212：isAllowedChild 表族名单收口（part01d 两笔）

修复 1 使 eff 读数正确后，`forceValue(tbody)` 因名单缺 tbody/td/tr 误判
allowed → 走 vendored :754 包裹臂产出 `<table><font><tbody>` 非法形态——此前
靠孤儿 bug + UNDO「碰巧」正确（forecolor 67F→131F 回归态实证，P→F 71）。

1. `_zwVaProhibitedParagraphChildren` 对齐 spec 名单（44 名：补 caption/col/
   colgroup/dt/listing/plaintext/tbody/td/tfoot/th/thead/tr/xmp；移除非 spec
   的 canvas/dialog）。
   https://w3c.github.io/editing/execCommand.html#prohibited-paragraph-child-names
2. `_zwVaIsAllowedChild` 补 spec 终检：body/caption/col/colgroup/frame/frameset/
   head/html/tbody/td/tfoot/th/thead/tr 对任意 parent 恒 false（参考实现
   :1682-1689；prohibited∩inline 面只覆盖 inline 父，span 字面父臂经此收口）。

## 3. inserttext：branch3 零变异 + typing 值 override 双缺口

**探针**（gitignore `zz-probe-typing.html`，修复前三形全零变异）：

```
A(plain)="ac" sel=3:1 || B(override) qv=rgb(0, 0, 255)="ac" || C(swcss-on)="ac"
```

- part06 execCommand R3254-M2 branch3（选区爬链 editing-host 分支）无 inserttext
  实应用臂——事件照派、DOM 零变异；branch2 需 activeElement 在 CE 内，WPT 环境
  activeElement=body 落 branch3。
- 五值命令 collapsed 记账的 `_zwVaValueOverrides` 仅 qV 消费（part01c），无
  typing 插入应用面。

**spec 语义照录**（vendored implementation.js）：

- insertText action（:7831-7960）：deleteSelection(stripWrappers false) → 邻接
  文本节点吸附 → **recordCurrentOverrides（:4351，插入前）** → insertData →
  **restoreStatesAndValues（:4458，插入后）**——对覆盖插入段的 active range 逐条
  值 override 等值比较（areEquivalentValues 门），非等值按 command action
  （= setSelectionValue 主流程，边界 splitText 只裹插入段）重放 → collapseToEnd。
- forceValue（:2696）本体无等值早退（wrap 臂在前）——缺等值门会对既有同值内容
  产生冗余包裹。

**修复三处**：

1. **part01 变异核拆分**：`__zw_ce_inserttext_mutate(el, ins)` 导出（caret 世界
   同步 + '\n' 分流 + 文本/元素/void 三臂 + 占位 br 消费 + caret 重锚），事件序
   留在键盘 `__zw_ce_insert`（beforeinput → textInput → 核 → input）与 branch3
   （beforeinput 已派 → 核 → input 由既有事件序尾部派发）各入口——核零事件，
   不双派。返回 `{ ok, text }`。
2. **part01d 快照/重放导出**：`__zw_ce_value_override_snapshot`（核头捕获——
   读侧 reset：世界同步与选区移动令 stored range 失配清空 override，事后读恒
   空）+ `__zw_ce_value_restore_overrides`（变异尾：临时 range `(node, off0)-(
   node, off1)` 覆盖插入段进 `_zwVaSetSelectionValue` 主流程——就地 splice 使
   caret 锚为整节点，直接 forceValue 会过裹全串，边界 splitText 消化；等值门
   `_zwVaEquivalentValues`；尾塌 range 终点 = vendored collapseToEnd 面）。
3. **part06 branch3 臂**：`cmd === 'inserttext' && rng2 && host2` → 核直调
   （事件口径同 insertparagraph 臂——beforeinput 已派、input 走既有事件序）。

**探针修复后三形逐字命中**：

```
A(plain)="abc" sel=3:1
B(override) qv=rgb(0, 0, 255)="a<font color=\"#0000ff\">b</font>c"   ← = forecolor.js a[]c 期望形
C(swcss-on)="a<span style=\"color: rgb(0, 0, 255)\">b</span>c"
```

## 4. 对账

### 4.1 gate12（s212 单独，批 20:22-20:40，systemd user unit zw-gate12）

| 语料 | g10f 基线 | gate12 | Δ |
|---|---|---|---|
| fontsize | 2,329P/93F | 2,334P/88F | F→P 5 |
| forecolor | 2,260P/67F | 2,271P/56F | F→P 11 |
| fontname | 2,150P/75F | 2,166P/59F | F→P 16 |
| backcolor | 1,002P/24F | 1,014P/12F | F→P 12 |
| hilitecolor | 1,181P/37F | 1,193P/25F | F→P 12 |
| 合计 | 8,922P/296F | 8,978P/240F | F→P 56、P→F 0 |

位移电池 join 三案 3,456/2,607/1,455P/0F、wss-ilb 552P/0F、wss-ip 1,104P/0F、
chd 420P/0F、ipihost 880P/0F、void 72P/0F + 48P/0F、run-ilb 1,641P/50F 逐位恒等。
hsem 6,913P/648F（g10f 652F → −4，全 F→P 方向）。

unit 内 cargo 不在 PATH → enginelib/webviewlib/bench rc=127（基建缺口非代码回
归；gate13 补 `PATH=$HOME/.cargo/bin` 前缀后重跑收口，见 §4.2）。

### 4.2 gate13（s212 + inserttext 合并，干净二进制，批 20:56 起）

| 语料 | gate12 | gate13 | Δ |
|---|---|---|---|
| fontsize | 88F | 2,334P/88F | F-list 逐字节恒等 |
| forecolor | 56F | 2,279P/48F | F→P 8（恰 a[]c 族）、P→F 0 |
| fontname | 59F | 2,166P/59F | F-list 逐字节恒等 |
| backcolor | 12F | 1,014P/12F | F-list 逐字节恒等 |
| hilitecolor | 25F | 1,193P/25F | F-list 逐字节恒等 |

a[]c 8 案双向齐收：flag-false-at-insert 四案（#0000FF/rgb/rgba/transparent →
`<font>` 形）+ flag-true-at-insert 四案（同值 → `<span style=color>` 形）——
typing 重放跟随 CSS flag 的实现即修复面本身（forceValue `!cssStylingFlag` font
臂，vendored :2795 同门）。

### 4.2a multitest 形态矛盾族（挂账，不 chase）

multitest.html bootstrap 显式 `execCommand("styleWithCSS", false, "true")`（录制
环境还原 hack，注释自述「every group of tests except the last … sets styleWithCSS
… left at true」），其 fontname/forecolor + insertText 组合案期望 `<font face>`/
`<font color>` 形；而 forecolor.js 现代 a[]c 族在 flag-true-at-insert 下钉死
`<span style=color>` 形（`[[swcss,false],[foreColor,#0000FF],[swcss,true],
[insertText,b]] → a<span style="color:rgb(0, 0, 255)">b</span>c`）——两语料对同
一语义给出矛盾期望。本实现取 flag-following（对齐 vendored forceValue 与现代五
值语料）；multitest 侧 ~16-24 形态 F 挂账不 chase。multitest 另一 F 族 = mid-
command 缺口（formatblock/indent/insertorderedlist/insertimage/
inserthorizontalrule 在 branch3 无实应用臂——`[["backcolor",…],["formatblock",
"<div>"],["inserttext","a"]]` 期望 `<div>foo<span…>a</span>bar</div>` got 无
`<div>` 包裹），backcolor span 形正确、inserttext 落位正确，与 inserttext 变更
无关。

insertText 波及语料首测（本 gate 系列无既有基线，家族分析附下）：

| 语料 | gate13 首测 | 残余主族 |
|---|---|---|
| inserttext | 2,777P/232F | `\n` 段落等价族（语料录制 Chrome 不分段）、空串+非空选区 deleteSelection 前置族、嵌套 inline 跨选区族 |
| multitest | 9,331P/761F | typing 形态矛盾族（§4.2a）+ mid-command 缺口族（formatblock/indent/list/hr/image branch3 无应用臂） |
| insertparagraph | 6,720P/475F | 待普查 |
| insertorderedlist | 1,438P/323F | 待普查 |
| insertunorderedlist | 1,721P/344F | 待普查 |

门禁（gate13 批 + hsem 单跑收口）：engine lib **rc=0**、webview lib **rc=0**
（env -u 清代理）、bench **rc=0**（perf-gate 达标）、hsem 复跑 **6,917P/648F**
（g12 6,913P/648F——F 数恒等，2↔2 翻转全在既有 `:active` flakes 族
checkable-active-onblur / radio-focus-navigation-checked，goal 史已定谳 flake
面，零真实回归）。hsem 首跑（批内 21:48）8 job 零输出即死（21 字节日志），
单跑复现窗口后复跑正常——记一次性观测，机制未追。gate12 批的 enginelib/
webviewlib/bench rc=127 系 unit 内 PATH 无 cargo（基建），gate13 脚本补
`export PATH="$HOME/.cargo/bin:$PATH"` 后三项全 rc=0。

## 5. 残余与挂账

- a[]c '\n' 臂不接值 override 重放（FIXME 已注——vendored 委托 insertParagraph
  自带 record/restore 周期，本仓拆分核未接）。
- inserttext 语料 `\n` 族：引擎 '\n' → insertParagraph 委托对录制行为（Chrome
  不分段）——独立归因轮。
- 空串 insertText 在非空选区上应先 deleteSelection（vendored :7834 delete 先于
  空值检查）——当前核 `ins === ''` 直返。
- forecolor 残余 48F：brown font 边界拆分族（~8）、pushDown 读黑族（2）、
  purple 误剥族（3）、o<br>bar br 误留族（2）、`<p>[foo<p><br><p>bar]` w5
  void-br 族、qV-after 残余。
- gate12 unit 教训：systemd user unit 默认 PATH 无 cargo——批脚本需
  `export PATH="$HOME/.cargo/bin:$PATH"`。
