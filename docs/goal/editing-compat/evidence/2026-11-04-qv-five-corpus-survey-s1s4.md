# ⑦ 值命令五语料普查 + S1-S4 qV 读面修复笔（2026-11-04，乙续）

## 普查（04:36–04:41，zw-qv-survey，五语料全量）

现行残差 **232F/8,986P**（gate13 挂账 ~109F 已陈旧，语料亦扩容），162 不同失败案。
构成：exec 包裹形状 ~126（assert_equals，push-down/add-wrapper 内层缺口）+
in_array 多形 15（归 ⑨）+ qV 读面 75。qV 面 75 行拆解：**58 行 exec 失败下游**
（exec 未包裹 → 读面走查落 UA 缺省，不可独立修）、**17 行 standalone**。
族表与逐案 expected/got 证据见 `.acceptance/ipara-port/qv-survey/qv-survey-verdict.md`
（原始 log 含每 Fail 的 assert 明细）。

standalone 17 行归四缺陷：

| 族 | 形态 | 行数 | 根因 |
|---|---|---|---|
| S1 | `<a href>foo[bar]baz</a>` qV forecolor before 期望 rgb(0,0,255) 得 rgb(0,0,0) | 3 | plain 回落无 UA stylesheet a:link 规则 |
| S2 | `<font color="0000ff">` 期望 rgb(0,0,255) 得 rgb(0,0,0) | 2 | legacy 属性色无 `#` hex 形态未补 `#` |
| S3 | `rgba(0, 0, 255, 0.5)` 期望原样得 `...0.502` | 4 | computed color 序列化 alpha 3 位四舍五入（u8 127.5→0.502），Chrome 口径 = 最短 roundtrip 十进制 |
| S4 | `foo<font size=+1>[bar]</font>baz` qV fontsize before 期望 4 得 1 | 2（连带 exec 面 1） | `parseInt('+1')` 错当绝对档位 1；相对值 ±N 未实现（原 FIXME 注记「语料无消费面」已过时） |
| S5 | `<p>[foo<p><br><p>bar]` qV 得空串；`[foo<pre>bar</pre>baz]` fontname qV-after 得 serif 期望 sans-serif；table qV-after 得 3 期望 4 | ~6 | 选区/走查面（疑似多块部分含选区 walk 异常→catch 落空串；或 exec 后选区态与 Chrome 漂移）——本轮未动，待探针定位 |

S3 另有 exec 已通过案的 qV-after 形（`foo[bar]baz` rgba 0.5，2 行）。

## 修复笔（本轮，4 处）

1. **S3** `crates/engine/src/js_dom_bridge/computed_style.rs` `color_to_css`：
   alpha 序列化改最短 roundtrip 十进制，复用 `canvas.rs serialize_alpha`
   （`pub(super)`；R34xx 同算法——canvas 侧 2d.fillStyle.get.halftransparent
   先例）。 Chrome 计算值序列化口径；u8 通道无法精确保存 0.5×255=127.5。
2. **S2** `crates/engine/src/js_dom_shim/part01c.js` `__zwPlainComputed`
   font 属性臂：legacy 属性色（color/bgcolor）无 `#` 的 3/6 位 hex 补 `#`
   （HTML 渲染节 body 属性映射 legacy parse；仅属性臂消费面，inline style
   同形仍非法）。
3. **S4** `part01c.js` `_zwPlainFontSizePx` font[size] 臂新增相对值 ±N：
   `/^([+-])([1-6])$/` → 父链首个可解析 font-size 过 legacy 中点表定基准档
   （缺省 16px→档 3）± N，clamp 1..7；绝对值臂原样后置。
4. **S1** `part01c.js` `__zwPlainComputed` color 臂：UA stylesheet
   `a:link { color: -webkit-link }`（HTML 渲染节 15.3.1）→ 带 href 的 a
   返 `rgb(0, 0, 255)`（无 href 不命中；继承链命中=文字继承 UA 链色，与
   Chrome 一致）。

## 验证（qv-fix1 轮，04:57 起，五语料重跑行级差 judge-qvfix1.py 元组口径）

**13 F→P / 0 P→F / VERDICT OK**，伪 T 行 0：

| 语料 | 行数 | F | F→P | P→F |
|---|---|---|---|---|
| fontsize | 2,422 | 88→84 | 4 | 0 |
| fontname | 2,225 | 59→59 | 0 | 0 |
| forecolor | 2,327 | 48→39 | 9 | 0 |
| backcolor | 1,026 | 12→12 | 0 | 0 |
| hilitecolor | 1,218 | 25→25 | 0 | 0 |

逐族对账（与修复笔一一对应）：

- **S4**（4 行）：`foo<font size=+1>[bar]</font>baz` qV before ×2 variant + `<font size=+1>foo[bar]baz</font>` qV before + **连带 exec 面 compare innerHTML ×1**（exec 对相对值 font 的 push-down 判定读到正确有效尺寸后翻转）——比预期多收 1 行。
- **S1**（3 行）：`<a href=http://www.google.com>foo[bar]baz</a>` qV forecolor before ×3 variant（blue/#0000ff/rgb(0,0,255)）。
- **S2**（2 行）：`<font color="0000ff">[foo]</font>` qV before ×2 variant。
- **S3**（4 行）：`<span style="color: rgba(0, 0, 255, 0.5)">[foo]</span>` qV before ×2 + `foo[bar]baz` rgba 0.5 qV after ×2。

fontname/backcolor/hilitecolor 零位移符合预期（其残差全在 exec 下游面与 S5 族，
本轮未动）。残差更新：**232F → 219F**（分母不变，五语料）。

门禁：cargo fmt --check 过；zero-engine clippy `-D warnings` 干净；make test
（zw-qvfix1-maketest，05:02-05:06）workspace 测试腿 3,724P/**5F**——5 案全落
`js_dom_bridge::tests`（p56_adopted_snapshot_restore_hook_keeps_page_value、
test_p99s_rule_style_preserves_custom_property_case、
test_stylesheet_add_rule_remove_rule_r3276、test_stylesheets_rule_style_r2810、
test_stylesheets_write_r2809），`git stash` A/B 双向证明为 HEAD（f888736f9）
既有红与本笔无关（± 本笔零差，pop 后账同）；判定依据 = 本笔改动面（shim plain
回落三臂 + computed_style alpha 序列化）不触 stylesheet/rule-style/adopted 面，
且语料判决 13 F→P/0 P→F 独立成立。既有红跨席归因登记 COORD（非本席面）。
常驻回归面：editing/run 五语料为 pinned 语料常驻 make 入口（wpt-suites.csv
editing-compat 行），失败案翻转即回归断言，无需新增 import。

## 残差与新账（后续轮工作面，fix1 时点）

1. **exec 包裹形状 ~126 行**（跨五语料，骨架 `exp /font+/span+font+span vs got font`
   单族 55 行）——push-down/add-wrapper 内层语义，⑦ 下一步主攻。
2. **S5 qV 得空串/UA 缺省族 ~25 行**：`<p>[foo<p><br><p>bar]` qV before/after 得
   `''`（indeterm 同案 false——异常路径签名一致，疑似多块部分含选区 walk 异常
   或 exec 后选区态漂移）；`[foo<pre>bar</pre>baz]` fontname qV-after 得 serif；
   table `<tbody data-start data-end>` qV-after 得 3。需探针定位于
   `_zwValGetAllEffectivelyContained` 走查/exec 选区恢复，与 ① 有交叉（部分案
   exec 同败）。
3. in_array 多形 15 行归 ⑨ 合流不变。

## ⑦ 残余第一笔落地：minified joiner + descend 臂自面读（fix2→fix3，2026-11-04 乙续落地、判读收口）

针对 ① exec 包裹形状族先做两笔（`crates/engine/src/js_dom_shim/part01d.js`，
探针 `tests/wpt-runner/wpt-data/editing/run/zz-probe-vawrap.html` 五案
A1/A2/C1 + D1/D2）：

1. **`_zwVaSetStyleProp` 手工合并序列化改 minified**：`k: v` + `'; '` joiner →
   `k:v` + `';'`。语料期望 Chrome recorded 无空格形（compare innerHTML 裸串
   等值无归一；host style.setProperty 通道同形；justify 簇 wrap 样式
   `text-align:center` 先例同口径）。
2. **add-wrapper descend 臂条件读换自面走查**：spec 条件读 move 后
   effective(node)——真 Chrome computed 自 node 自面起算（自身 style/font 属性
   + UA stylesheet 元素规则，无自面声明才继承新 styled wrapper，等价跳臂）。
   旧 `_zwValEffectiveValue` 对 sel 注册（M 形）元素走 host 通道，plain 新建
   styled 祖先不在宿主树 → 读旧继承 ≠ 新值 → 臂误触发：span1 被移出
   styled wrapper 逐子重包（fontsize 4 双层套，期望单层外裹）。

**fix2 首版回归撤改**：descend 臂改挂 `_zwVaSpecifiedValue` null 门只覆盖
author-specified 面，漏 UA 规则面误跳臂——fontname 语料 tt/code/kbd/samp 8 案
回归（`judge-qvfix2.py`：F→P 6 / P→F+newF 8，VERDICT REGRESSION，语料账
`.acceptance/ipara-port/qv-fix2/`）。fix3 改读 `__zwPlainComputed(node,
property)` 自面走查（plain 通道同时看见 styled wrapper 与 UA 规则表：
plain span 爬升命中 wrapper 新值 → 等价跳臂；tt 族首步命中 UA mono 规则
→ 臂照走，包裹收进 tt 内层）；自面无源（''）回落 host effective 保 UA
初值口径；等价比较走 `_zwVaLooselyEquivalent`（自面 fontsize 返 px 形，
sizeMap 桥接关键字）。

**验证（qv-fix3 轮，judge-qvfix3.py 元组口径，对 qv-fix1 基线）**：

- probe-vawrap3 五案 5/5 PASS（A1 `<span style="font-size:large"><span>foo</span>
  <span>bar</span></span>` 单层外裹 + A2 font 形 + C1 多块 + D1/D2 tt 内层
  双 flag）。
- 五语料：fontsize F 84→78（**F→P 6**：`<span>[foo</span> <span>bar]</span>` /
  `<p>[foo</p><p> <span>bar</span> </p><p>baz]</p>` compare innerHTML 双 flag
  ×2 形 + qI ×2）、fontname 59→59（fix2 的 8 例回归消除）、forecolor
  39→39、backcolor 12→12、hilitecolor 25→25——**合计 F→P 6 / P→F+newF 0，
  VERDICT OK**；伪 T 行 0。
- 残差更新：**219F → 213F**（fontsize 78 / fontname 59 / forecolor 39 /
  backcolor 12 / hilitecolor 25；compare/equals 136 + qV/qS/qI 读面 77）。
- engine lib：test-guard 包裹 `cargo test -p zero-engine --lib`
  （zw-qvfix3-enginelib，06:02-06:03）**3,724P/5F**——5 红逐名恒 fix1 轮
  stash A/B 定谳的 HEAD（f888736f9）既有名单
  （p56_adopted_snapshot_restore_hook_keeps_page_value /
  test_p99s_rule_style_preserves_custom_property_case /
  test_stylesheet_add_rule_remove_rule_r3276 / test_stylesheets_rule_style_r2810 /
  test_stylesheets_write_r2809），P 数 3,724 与 fix1 轮持平，零位移。

descend 臂本轮对语料零净翻转（+6 全归 joiner 笔），其价值在探针 D1/D2 锁
tt 内层包裹语义 + 双层套误触发面修复（未来 fontsize push-down 案不再误摘
wrapper）。①族残余 ~120 行主攻（`exp /font+/span+font+span vs got font`
单族 55 行）不变。


