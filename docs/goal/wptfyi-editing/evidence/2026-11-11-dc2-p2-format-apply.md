# DC-2 · P2 apply 面 vendored 引擎落地（2026-11-11）

## 修复面

六 inline format 命令（bold/italic/underline/strikethrough/subscript/
superscript）的非 collapsed execCommand 从 flat 模型（宿主直子文本
splice，嵌套/跨段落零变更面）整体切换到 vendored `setSelectionValue`
全流程引擎（spec action 直译，`part01d __zw_ce_format_apply`）：

- **apply 入口**：spec action 原文（bold/italic state 真落 normal、
  underline/strike 落 null、sub/sup 两段式 null→目标值）；调 `part01c
  P1 __zw_ce_format_query` 读 toggle 前状态。
- **state override 写面**（P1 挂账接通）：`_zwVaSetStateOverride/
  GetStateOverride` + `__zw_ce_value_state_override` 导出；无
  formattable 臂按 inlineCommandActivatedValues 记账（sub/sup 互斥
  unset 对侧）；读序按 spec myQueryCommandState（override 只盖返回值
  .state，indeterm 纯 DOM——`part01c __zw_ce_format_query` 与
  `part06 _zwQueryFormatState` 两处消费）。
- **值面六命令扩展**：`relevantCssProperty` 两处映射（bold
  font-weight / italic font-style / u+s text-decoration / sub+sup
  vertical-align）、`equivalentValues` bold 臂（bold≡700、normal≡400）、
  forceValue 元素臂（flag=false 恒 `<b>/<i>/<s>/<u>`，sub/sup 恒元素
  形无 flag 门）、UA weight 表补 h1-h6/th。
- **共享路径 Chrome delta**：sibCrit css 门（css flag true 不并 tag 形
  兄弟，multitest css-true 族根因）、push-down 重裹臂跳 br（⑨）、
  选区重绑保 `__zwStyleWithCSS`（swcss 清除臂绕开）。
- **misc P3**：styleWithCSS exec 出口补拍 `__zwStyleWithCSSQuery`
  （misc 单命令案 qS-after 滞后读，28F 实证面）。

## 二轮修正（同轮回归归因后落地）

首轮 after 对跑暴露 23 断言 P→F，四簇归因：

1. **cef 端点门**（11，bold ?1-1000）：边界位置入 contenteditable=false
   子树 → 整条命令 no-op。Chrome 对端点入非编辑区恒不应用（`fo[o<span
   cef>b]ar</span>baz` 期望原样）；vendored 会剥可编辑片段落值。
   `__zw_ce_format_apply` 入口边界位置解析门（文本容器即本节点；元素
   容器取 offset 处子节点，offset=末端不触发——`{…}` 选形端点在
   editing host 元素上，`_zwJusIsEditable` 对宿主本体返 false 故必须
   解析到位置而非容器）。
2. **peel 残料溶解**（2，superscript `<sup>[bar]<br></sup>`）：end 侧
   bare-move 后壳内残余全不可见（extraneous br/空白）→ 壳溶解。
   `foo<sup>[bar]<br></sup>` 解除录形 `foobar<br>`，Chrome 不留空壳；
   vendored/首轮留 `<sup><br></sup>`。`_zwVaPeelOne` 两臂对称接
   `_zwVaPeelDissolveInvisibleRemnant`。zz-probe-p2supbr B 变体
   `foobar<sup><br></sup>`→`foobar<br>` 实证。
3. **sibCrit sub/sup 放行**（4，sub/sup `<span>[foo</span>
   <span>bar]</span>` css-true）：css 门误伤元素壳合并——Chrome
   css-true 亦合并单壳（期望 `<sub><span>foo</span>
   <span>bar</span></sub>`），首轮门后裂成 per-span 双壳、qI/qS 下游
   翻红。门补 sub/sup 例外。
4. **作者样式载体**（8，strike/underline `class="line-through"` /
   `class="underline-and-line-through"` span）：Chrome 对作者样式表
   携带 decoration 的 span 原位并 style / 识别已激活 no-op，我方 gCS
   无作者规则臂 → state 读 false 误应用外裹。**本轮不修**（gCS 作者
   样式表规则臂 = P1 FIXME 挂账面，范围大于本轮），挂 P2F 已知账。

## 方法与数字

同树前后二进制对跑（`dc2p2-base-runner` = HEAD 49a3b2db8，
`dc2p2-fix1-runner` = HEAD + P2 全量；test-guard/systemd scope 包裹，
`ZW_SCRIPT_TIMEOUT_MS=300000`），断言级（全子测试名）前后 diff。

面 1 · 六 format 命令文件 + 8 探针（qsq/qv5/vsel/tstyle/fmtstate/
swcss/mtmini/p2diag）：

| 文件 | base P/F | fix1 P/F | ΔP |
|---|---|---|---:|
| bold | 2673/375 | 2964/84 | +291 |
| italic | 1845/228 | 2049/24 | +204 |
| underline | 1894/253 | 2101/46 | +207 |
| strikethrough | 1895/252 | 2055/92 | +160 |
| subscript | 1080/161 | 1228/13 | +148 |
| superscript | 1112/161 | 1260/13 | +148 |
| 合计 | 10499/1430 | 11657/272 | **+1158** |

- base 六文件数与 [dc2-p1-format-state](2026-11-11-dc2-p1-format-state.md)
  P1 after 列逐文件相同（口径连续）。
- 断言级：**F2P=1,174 / P2F=8 / newF=0**；8 P2F 全为簇 4（作者样式
  载体，已知账）；probe qv5 2 断言名变化为 dump payload 携带型
  （`<font color>`→`<span style=color>` 载体形向 Chrome 靠拢），两轮
  全 PASS 良性。
- 探针见证：zz-probe-swcss 0/3→3/0（css-true 形修复）；p2supbr B
  `foobar<br>`（peel 溶解）；p2cef A/B/C no-op+qS=false、D 对照应用
  （cef 门）。

面 2 · font 命令族回归哨（backcolor/fontname/fontsize/forecolor/
hilitecolor，共享 push-down/sibCrit 路径）：

| 文件 | base P/F | fix1 P/F | ΔP |
|---|---|---|---:|
| backcolor | 1026/0 | 1026/0 | 0 |
| fontname | 2191/34 | 2222/3 | +31 |
| fontsize | 2381/41 | 2419/3 | +38 |
| forecolor | 2296/31 | 2322/5 | +26 |
| hilitecolor | 1218/0 | 1218/0 | 0 |
| 合计 | 9112/106 | 9207/11 | **+95** |

断言级 F2P=95 / P2F=0——sibCrit css 门与 br 跳对 font css-true 形同向
修正（font 语料零回归）。

面 3 · multitest + misc（最大消费面；after = 首轮二进制，fix1 = 二轮
修正后）：

| 文件 | base P/F | after P/F | fix1 P/F |
|---|---|---|---|
| multitest | 9647/2 | 9665/2 | **10027/2** |
| misc | 395/28 | 401/22 | 401/22 |

- multitest fix1 较 base **+380P**（F 恒 2 同名）：增量全为新注册子
  测试且全 Pass（引擎走通更深序列），失败名集合零新增零丢失。
- misc 6 个 `styleWithCSS` qS before/after 变体 F→P（TRUE/TrUe/true/
  "true " 大小写与尾空格形）——P3 出口补拍实证；F 28→22。
- 三面合计断言级净增 **+1,647**（F2P 1,275 + 新注册 ~380 − P2F 8）。

## 残余与下一步

- 簇 4 作者样式载体（8 P2F）：gCS 作者样式表规则臂 + 载体识别
  （state 读 + apply 原位并 style 双面），P1 FIXME 升级为 P3 待办。
- 六文件残余 272F 长尾：bold 数值权重边界 merge（500/600 邻接）、
  italic var/dfn 归一、u/s 多值 decoration 偏序删除（clearValue
  FIXME 挂账）、sub/sup vertical-align 邻 span。
- 语料全量增量（er0-er9 片 2,342 innerHTML 族 + state 下游）以下周
  CI weekly 对账。
