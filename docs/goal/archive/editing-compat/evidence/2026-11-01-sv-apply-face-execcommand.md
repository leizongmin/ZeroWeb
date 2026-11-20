# style-with-value 簇实应用面收口——execCommand 五值命令 setSelectionValue 引擎移植（2026-11-01）

## 0. 结论速览

| 门禁 | 本轮 | 轮前基线（gate7） | 位移 |
|---|---|---|---|
| fontsize 真页 | **2,211P/211F/0T** | 2,028P/394F | F→P 184、P→F 1（§5 挂账）、分母逐位恒等 |
| forecolor 真页 | **2,142P/185F/0T**（v4） | 1,960P/367F | F→P 182、P→F 0（v2 中途 3 例 §6.4 已回收）、分母逐位恒等 |
| fontname 真页 | **1,984P/241F/0T**（v3） | 1,801P/424F | F→P 183、P→F 0（v2 中途 2 例 §6.5 已回收）、分母逐位恒等 |
| backcolor 真页 | **875P/151F/0T** | 803P/223F | F→P 72、P→F 0 |
| hilitecolor 真页 | **1,054P/164F/0T** | 970P/248F | F→P 84、P→F 0 |

五语料合计 **8,266P/952F** 对 gate7 7,562P/1,656F：F→P **705**（184/182/183/72/84 逐语料）、P→F **1**（fontsize `+1` 渲染流域挂账 §5；v2 批中途 3 例 a[href] + 2 例 tt P→F 已在 v3/v4 回收，终版 P→F 逐语料 0/0/0/0/1），分母逐位恒等（onlyOld=0/onlyNew=0 五语料全零）。

上两轮（setup-exception 族收口 + 值查询面收口）后五语料失败主体 = compare innerHTML **567F**（apply 缺口②——引擎 execCommand 五命令零变更臂）。本轮移植 vendored `setSelectionValue` 全流程到引擎面。

**终版残余族普查（952F，五语料合计）**：queryCommandValue **312**、compare innerHTML **296**（上轮 567 → 296）、queryCommandIndeterm **129**、queryCommandState **125**、return value 60、checks for modifications 30。

## 1. 实现（part01d.js 新文件 936 行 + part06 接线两笔 + part01c override 消费 + bridge 挂载）

### 1.1 part01d.js（`__zw_ce_value_apply` + 36 个 `_zwVa*` 机具）

参考实现（tests/wpt-runner/wpt-data/editing/include/implementation.js）忠实移植：

- `setSelectionValue`（:2965）：无 contained formattable → value override 记账臂（collapsed 选区）；边界 splitText 两臂（引擎安全拆分：data 写 + createTextNode + insertBefore——`_zwVaSplitText`，part04 splitText 仅 handle-based wrapper 可用）；clearValue pass（contained 元素）+ pushDownValues/forceValue pass（参考实现 :2586/:2696——forceValue 的 CSS flag false 臂落 font color/face/size attr，fontsize xxx-large 恒 font attr 特例照录）；
- `wrap`（:1377）/`reorderModifiableDescendants`（:2345）/`removeExtraneousLineBreaks*`（:1305-1377）——共享机具复用 part01c `_zwJus*` 闭包（move/range/tree/visible/editable/cs），`_zwVaWrap` 的 globalRange 簿记换 `sel._ranges[0]` 单 range 补偿（新 parent 直接插入臂边界 offset +1；移动臂 `_zwJusMovePreservingRanges` 自带补偿）；
- `isModifiableElement`/`isSimpleModifiableElement`（:1833/:1883）、`isAllowedChild`（:1572）、`areEquivalentValues`（色三命令 normalizeColor 等值）/`areLooselyEquivalentValues`（fontsize sizeMap）、`normalizeFontSize`（:3192 ±3 相对模式 + clamp 1-7）、`getSpecifiedCommandValue`（:2201 font color/face/size 表现提示臂）；
- state/value override（:1102-1171）：`_zwVaValueOverrides` map + 存储范围快照失配即重置（选区变动作废）+ backcolor/hilitecolor 双向镜像。

入口三导出：`__zw_ce_value_apply`（校验+应用，返 false = 无效值）、`__zw_ce_value_validate`（纯校验，part06 事件前前置）、`__zw_ce_value_override_value`（override 读数，fontsize 经 `_zwVaLegacyValueOf` 两段式转换后返——part01c `_zwValLegacyFontSize` 只吃 px 串，关键字 override 会错转，转换须在导出侧）。

### 1.2 part06 接线两笔

- **校验前置**（编辑分支 host 判定后、beforeinput 派发前）：五命令 `__zw_ce_value_validate` 返 false → `return false` 零事件（Chrome 口径——vendored action 在 setSelectionValue 前做值校验，失败直接 false）；
- **应用臂**（justify 臂后、inserthtml 臂前，notCanceled 块内）：`__zw_ce_value_apply(cmd, arguments[2])`，事件口径同 justify 臂（beforeinput 已派，input 在块尾统一派）。

### 1.3 part01c `__zw_ce_value_query` override 消费

入口 five 检查后先查 `__zw_ce_value_override_value`（未设返 undefined 走原链）：命中即返（fontsize 的 legacy 映射在导出侧完成）——vendored myQueryCommandValue :655-663 同序；Indeterm/State 无 override 面（五命令无 inlineCommandActivatedValues，collapsed 只记 value override）。

### 1.4 bridge 挂载

part01c 后、part07 前拼接（同闭包可见 `_zwJus*`/`_zwVal*`/plain 解析器机具）。

## 2. 探针（gitignored 临时页 `zz-probe-fs.html`，不入库）

- 接线验证：t4 无效值 return false 零事件（fontsize 'foo'/forecolor 'notacolor' → false、beforeinput 计数 0、DOM 不变）一轮过；
- 读数探针（SENTINEL fail 形态打读数）：sizeMap 表 `font[size=1..7]` gCS 直调 = 10.08/13.12/16/18.08/24/32/48px（**plain 回落小数表**——createElement 自建 wrapper 无 host 映射），语料形态 `<font size=4>`（innerHTML 物化 handle-proxy）= **18px**（host 通道整数表）——两链异值。

## 3. 中途修复（三笔，均在 part01d.js）

### 3.1 `_zwVaSizeMap` 动态读改静态表（fontsize 5 例 P→F → 4 例回收）

初版照录 vendored 动态 gCS 读法（createElement font + setAttribute(size,N) + gCS fontSize）——本仓两读数链异值（§2）：sizeMap 走 plain 回落 `18.08px`，`_zwValEffectiveValue` 走 host 通道 `18px`——`_zwVaLooselyEquivalent('fontsize','18px','large')` 判 false，同值选区（`<font size=4>foo[bar]baz</font>` fontsize 4）误走 wrap 链多包 `<font size=4>bar</font>`。Chrome 两侧同走原生 gCS 恒同值，我方等价面 = 与 effectiveValue 链一致——改静态表 `{x-small:10px … xxx-large:48px}`（与 part01c `_zwValLegacyBounds` 中点标度同构的 host 口径）。

### 3.2 `_zwVaIsAllowedChild` 字符串 parent 分支（fontsize 39 例回收主刀）

vendored 尾段组合表 `[namesOfElementsWithInlineContents, prohibitedParagraphChildNames]` **字符串 parent 参与判定**（'span'×'p' → false）；初版字符串 parent 直接进 DOM 爬升，`ancestor.nodeType !== 1` 即 break 恒 true → 跨 p 选区的 p 元素被 forceValue 误 wrap（got `<span> <p>bar</p></span>` 结构错乱）。补字符串 parent 分支：a×a / inline-contents×prohibited / h1-6×h1-6 三门 + 落 true。五命令调用面全部 `'span'` 字面量。

### 3.3 `_zwVaSetStyleProp` 读核验改 attr 权威面（fontsize 连带收口）

初版 setProperty 后以 `el.style.getPropertyValue` 核验——host setProperty 通道对未注册 handle 的自建 wrapper 只写内部样式面、**attr 未落**，核验通过即返 → innerHTML 序列化（读 attr）丢 `style` 声明（got `<span>foo</span>` 无 style；qV plain 回落同样读 attr → 16px → qV-after '3' ≠ '4'）。改核验面 = style attr（`_zwPlainStyleDecls` 读数），不过即回落 attr 手工合并；清空值从 attr 面删声明（`_zwVaClearValue` 依赖）。

## 4. fontsize 终数（全三变体）

**2,209P/213F/0T** 对基线 2,028P/394F：F→P **182**、P→F **1**、Timeout 0→0、分母 2,422 行逐位恒等（onlyOld=0/onlyNew=0）。中途两版（sizeMap 修复前 2,172P/250F → 修复后 2,170P/252F 出 qV-after 6 例新回归 → §3.2/§3.3 两笔后 2,209P/213F 净收 39 例零回归）证据绑最终工件。

## 5. 残余 P→F 定谳（1 例）

`<font size="+1">foo[bar]baz</font>` fontsize 4：`+1` 相对 size 属性（HTML 规范 size 解析——符号前缀基准 3 增减）host 引擎 UA 属性面不识别 → effective value 读缺省 16px ≠ 18px → 误包。**渲染流域挂账**（gCS host 通道 font[size] 符号前缀解析，非编辑命令域）。

## 6. 中途修复二批（四语料首轮 8,161P/1,057F 定谳后，四笔）

### 6.1 色三命令 Chrome 口径（forecolor return 5 例 + 蓝值归一主刀）

语料实证与 vendored 相异的 delta 面：

- **非法色返 true 零变更**：`potato quiche`/`f`/`000000fff`/`#f`/`#000000fff` 期望 `[true]` + DOM 不变 + qV-after 缺省黑（Chrome 不拒绝；vendored return false）。`currentColor` 独走 spec 拒绝（`[false]`，vendored 同）。
- **CSSOM 口径颜色归一**：`_zwVaNormalizeCssColor`（`_zwPlainColorSpec` 后补 rgb 分量 clamp——`rgb(15, -10, 375)` → `rgb(15, 0, 255)` 语料实证，gCS 通道原生钳位而 plain 解析未钳）；`_zwVaParseSimpleColor` 改经归一面（rgb()/hsl()/named/hex 全入口 → `#rrggbb`，rgba(α<1) → null 落 span 臂——语料 backcolor 无 font[bgcolor] 期望，span+style 形唯一）；`_zwVaSetStyleProp` color 系写入即归一（语料期望 `color:rgb(0, 0, 255)` 而 got `color:#0000FF`——裸 attr 合并丢归一）；override 读数（collapsed qV-after）同归一（语料 collapsed `#0000FF` → qV `rgb(0, 0, 255)`）。

### 6.2 forceValue 插入点选区边界补偿（qV-after 黑值 2 例主刀）

`foo{<font color=blue>bar</font>}baz` swC 两形 qV-after got `rgb(0, 0, 0)`：单案探针（手工选区 + 手工序列）全绿不可复现，切片页（真实 harness `browserTests.slice(72,75)`）复现。**归因链**：`_zwVaForceValue` 尾段 `insertBefore(newParent, node)` 裸插入无边界补偿（真浏览器原生调整、`_zwVaWrap` 有等价门而 forceValue 漏）→ 端点 (host,2) 漂到 newParent 与 node 之间 → 随后 `movePreservingRanges` 端点随动规则（offset === oldIndex 臂）把终点带进 `(newParent, 0)`（bar 之前）→ qV contained 集空、落端点链读缺省黑。Chrome 应落 (newParent, 1)（语料标点 `bar}` 实证）。补 `_zwVaWrap` 同款补偿（`insIdx = _zwJusNodeIndex(node)`，≥ 判定 +1）。tests.js `setupDiv` 每案 `test-container.innerHTML = '<div contenteditable></div><p>test'` 重建（div 物化形与 createElement 探针异链）是探针不复现根因。

### 6.3 位移电池（batch1，插入补偿前 build；五值命令域外零暴露）

join 三案 7,518P/0F/0T + wss-ilb 552P/0F + wss-ip 1,104P/0F + chd 420P/0F + ipihost 880P/0F + void 双案 72P/0F、48P/0F 全绿零位移。run 面 ilb 1,633P/58F 定谳既有失败（表格族 insertlinebreak，五值命令域外）。`inline-host` 宽 filter 定谳伪欠账——goal doc 简称即 ipihost 语料，无独立文件，battery 已覆盖。

### 6.4 a[href] Chrome delta（forecolor compare 3 例）

`<a href>foo[bar]baz</a>` + forecolor 期望 DOM 完全不变 + `[true]` + qV-after 新值（Chrome 不改链内样式）。补 `_zwVaInHrefAnchor` 门（两端点最近 a[href] 祖先同且非空）→ 零 DOM 变更 + 值 override 记账（qV 读新值）；仅 forecolor 观测面（backcolor/hilitecolor/fontsize/fontname 语料无该形）。首版 `up()` 文本节点起点 while 条件首查即败恒 null——语料仍 wrap，探针（bracket 字面污染）假阳——修正沿父链逐级查后回收。

### 6.5 enabled 面：选区端点不可编辑（fontname tt 两形 4 面）

`foo<tt contenteditable=false>ba[r</tt>b]az`（起点入 tt）与 `fo[o<tt...b]ar</tt>`（跨 tt 交叉）均期望 `[false]` 零事件（Chrome enabled 门）。bold 语料同形同期望（跨命令一致面）。`__zw_ce_value_validate` 入口加双端点 editable 门（`_zwJusIsEditingHost || _zwJusIsEditable`——宿主自身算可编辑边界，tt[ce=false] 子内不算）→ 返 false 零事件。fontname +4P 回收（return/compare/qV-after/qS 面）；bold 等其余语料同形族挂下一轮（本轮仅五值命令域）。

## 7. 其余语料终数（v2/v3/v4，gate7 对账逐行恒等分母）

| 语料 | gate7 | 终版 | F→P | P→F |
|---|---|---|---|---|
| fontsize | 2,028P/394F | 2,211P/211F | 184 | 1（§5 挂账） |
| forecolor | 1,960P/367F | 2,142P/185F（v4） | 182 | 0（§6.4 回收） |
| fontname | 1,801P/424F | 1,984P/241F（v3） | 183 | 0（§6.5 回收） |
| backcolor | 803P/223F | 875P/151F | 72 | 0 |
| hilitecolor | 970P/248F | 1,054P/164F | 84 | 0 |

## 8. 证据文件

- 全批日志：`/tmp/zw-edit-v/`（run-*.out、o-*.out、status-v.log；本轮收口后随 tmp 清理失效，终数已录本文档与 goal doc）；
- 对账脚本：`/tmp/zw-edit-v/reconcile.py`（gate7 基线 `/tmp/zw-edit-gate7/`）；
- gate8 门禁批：`/tmp/zw-edit-v/g8-*.log`（engine lib / webview lib / wcomp / hsem / bench）。

## 9. gate8 门禁批补记

- engine lib **3,664P/1F**——唯一红 `js_dom_bridge::tests::test_ua_display_li_list_item_canvas_inline_s24_flip` 与 gate7 红名 identical（并行流 s24 已知红）；
- webview lib **834P/0F**——首跑 833P/1F（`test_webview_fetch_url_invalid_host`，basic.rs:578 `result.is_err()` 失败）定谳环境假红：gate8 自用户 shell 继承 `HTTP(S)_PROXY=<LAN-PROXY>:10801`，`http://0.0.0.0:1` 经代理改为代理应答不再连接失败；`env -u` 五代理变量清空复跑 834P/0F 对齐 gate7（gate7 系 systemd 净环境跑法）；
- wcomp **4,022P/1,146F/5T** 对 gate7 计数级恒等；
- v2 批（fontsize/backcolor/hilitecolor）先于 6.4/6.5 门跑的有效性补证：三语料日志 grep `contenteditable=false` 与 `<a href=` 均 0 命中（fontname 18 命中为对照），两门对三语料零暴露，v2 数即终二进制数；
- hsem **6,903P/662F/162T** 对 gate7 6,895P/670F/162T：总 7,727 恒等、onlyOld=onlyNew=0、P→F 0、+8 全 F→P 逐案定谳——s203 number 值消毒 4 案（66f4871c1：number.html ×3 + number-constraint-validation ×1，与该提交自账恰等额）+ s204 pattern 面 2 案（61720d309 本次窗口子集）+ s205 setCustomValidity 换行归一 1 案（7c681d5c6）+ inert-svg-hittest 1 案（gate7 账 `:active` 时序 flakes 既有家族在册成员）；四笔均在 gate7 基线 5ee3d31f9 之后的 rebase 窗口落 main，非本 diff 面（本 diff 五值命令域不触及 number/pattern/validityMessage 面）。
