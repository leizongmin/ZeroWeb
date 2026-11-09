# R5433 归因轮 — misc 账本批量取证（corpus 卫生收口 + 8 案定谳全深/环境域，零代码改动）

日期：2026-11-06。上游：R5432（9fd67c408）下一步① misc 账本续扫择轻。
本轮性质：纯归因轮（无产品代码改动 → 无 corpus A/B / bench-gate 义务）。

## 1. corpus 卫生收口（本轮唯一账面变化）

上轮（R5426 期）探针页遗留清理：`tests/wpt-runner/wpt-data/css/CSS2/generated-content/zz5426{b-p1..p11,c-p1}`
13 个探针页（gitignored、不在 reftest-manifest，guidelines #31 属本地残留）删除。

| 窗口 | 案数 | failed | 真通过-可信 |
|---|---|---|---|
| w0（清前） | 16606 | 709 | 10392 |
| w1（清后） | 16594 | **696** | **10393** |

清后基线 = **16594 案 / 696F**（−13 案 −13F 全为探针残留，−1F 系 zz5426c-p1 清前通过）。
后续轮次基线以 696F 为准。

## 2. 八案批量定谳（无一落入单案轻修象限）

### 2.1 floats-wrap-bfc-006（1.18%）→ 弃案，新增挂账㉕

案形：td 内 ~30 个 `float:left; clear:left` 蓝条（宽 150→25 递减、高 1px）+ 两个嵌套表格
（紫 bg + 黄 caption）。REFTEST_DUMP：ZW 第 2/4 嵌套表格全宽（300px），ref 中缩窄到 float
带右侧剩余宽（~122px）。归因：CSS2 §9.5——BFC 盒（表格）border box 不与同 BFC float 重叠时
可缩窄贴排到 float 旁；ZW 的 float exclusion 仅覆盖 inline 行盒层（`FloatExclusion` 断行域），
块级 BFC 盒放置/宽度不感知 float 带 → 一律全宽。缺 float-aware BFC shrink-to-fit 排布特性，
深域。同族：floats-wrap-bfc-with-margin-006.tentative（1.82%）/007.tentative（2.08%）并入。

### 2.2 s-11-1-1b-005（1.52%）→ 弃案，新增挂账㉖

`html{overflow:hidden; display:table}` 根元素表格化 + `head{display:caption}` + body cell
负 margin。断言 = css2 errata §11.1.1b：根表格上的 overflow 作用于**视口**而非表格盒。
缺根盒 display:table 化 + 视口级 overflow 传播机制，深域（根盒传播域）。

### 2.3 html-precedence-004（2.48%）→ 环境案弃案，新增挂账㉗

`flags="userstyle"`：需 harness 注入用户样式表（`#hixie-cascade-tests .instructions
{display:none}` 等 4 条）才能成立。自动化 reftest 环境先天不可过，非渲染缺口
（font-family-rule-002a 同型环境案先例，R5425②）。

### 2.4 c548-ln-ht-000（2.57%）→ 弃案，新增挂账㉘（chromium oracle 定谳）

案形：Ahem 30px `.test`（line-height 60px）vs `<pre class="control">`（30px/1）两侧像素等价。
ZW 差异：control 块首行 aqua 贴顶（无上方 teal 带），ref（表格模拟）首行上方有 ~28px 空带。
**oracle 定谳**（本机 chrome headless，CDATA 剥离修正后）：HTML 解析下 chrome 的 test 页
control 块**同样**首行贴顶 4 行 120px —— 与 ZW 一致；ref 表格 150px 带空带 → **chrome
HTML 模式下 test≠ref**。原案 .xht 在真 WPT 走 `application/xhtml+xml` XML 解析：XML 不适用
「`<pre>` 起始标签后首个换行剥离」（HTML parser 专属规则），control 5 行 150px 才与 ref 匹配。
ZW 按 HTML 解析剥离换行 = 正确 HTML 行为；差异根源 = **.xht XML 解析模式缺失**（挂账㉘：
XML parser/XHTML 域基建，影响面全语料依赖 XML 特有语义的 .xht 案）。
取证技法注记：oracle 复现时 `<style><![CDATA[…]]></style>` 在 text/html 模式下整段失效
（chrome 实证：teal/navy/字体全不生效），须先剥 CDATA 壳再渲染。

### 2.5 vertical-align-baseline-003（2.90%）→ 弃案，新增挂账㉙（chromium oracle 定谳）

案形：`#span2{display:inline-block; width:5em}` 6 个 Filler Text，断言 inline-block 末行
基线对齐（§10.8.1）。dump：**ZW test 侧语义已对**（橙块 80px 宽 6 行、箭头对齐末行，
与 chrome bbox (32,50,110,157) 逐像素同构）；差异全在 **ref 侧** —— ref 用
`<col id="middle">{width:80px}` 表格模拟，chrome 尊重 col 宽 80px（内容换行 6 行，
橙 bbox 与 test 侧同构），ZW 的 auto table-layout 给中列 max-content（430px 单行橙带）。
归因：col 指定宽列的 auto-layout 贡献应为 used width（钳 min-content 下限），ZW 只作
floor、max-content 仍按内容最大宽参与（`collect_col_widths`→`compute_column_widths`，
R2045 收集面已正确）。改动落 R1146/R4479 列宽分布核心；波及面 = 全语料含 `<col>` 案
**189 例**（含 applies-to-005/006 家族）→ 须专项轮全矩阵 corpus A/B，非轻量。

### 2.6 floats-placement-vertical-001b（3.64%）→ 弃案，新增挂账㉚

案形：6 个 `p{overflow:auto}`（BFC）各含 `Hello<span>内 50×50 float</span>Kitty` 组合
（left/right × ltr/rtl × text-align:right）。dump：ZW 全复合失真——float 高度塌缩
（~50×14）、"Hello" 文本缺失、RTL 未反排、text-align 失效、x 位错乱。ref 展示 §9.5.1
最高位放置 + 行内 BFC 收纳 + RTL 反排 + 对齐联动多规则正确叠加。多规则复合缺口，深域。

### 2.7 c5525-fltmrgn-000（3.37%）→ 弃案，并入挂账㉚（float margin 域）

CSS1 float margin 复合案：6 子案 float `<img>` 四边 ±30px margin，红底图案作 fail 探测器
（Ahem + image）。float margin box（含负 margin）排斥几何 × 图案底逐像素判据，多子案复合，
与 2.1/2.6 同属 float 排斥几何域。

### 2.8 background-intrinsic-006（本批首个已定谳，衔接记录）

SVG `width="40%" height="60%"` 百分比内在尺寸背景 → ZW 红 L 形泄漏。intrinsic sizing
深域（deep intrinsic 48 变体矩阵挂起域邻接），弃案并入挂起矩阵，不另立条目。

## 3. 扫面收口结论

- 亚 3% 段（w1-fails 排序）：0.00% 带 = 3D transforms 在册域 + slider/file-selector-button
  /highlight 伪元素等特性域；0.02–0.36% 带 = scroll-markers/selection/fullwidth/corner-shape
  /letter-spacing-ligatures（R5423 域）/backdrop-filter，全部在册或特性域。
- 3–4.5% 段核心案（abspos-inline-002/position-absolute-in-inline-005 = R5427⑭ 邻近；
  box-shadow-table-border-collapse = R5425⑦ 邻近；ruby/content-visibility/flexbox/
  run-in/aspect-ratio/scrollbar-gutter/filter/pagination = 特性或深域）已尽数定性。
- **结论：非在册小差带（≤4.5%）无可立轻修候选；misc 轻端扫面按本轮收口。** 后续轮次
  从挂账深域择优开案（专项轮 + 全矩阵 A/B 预算）或处置待用户决策项。

## 4. 侦查技法沉淀（oracle 侧）

- `<style><![CDATA[…]]></style>`（.xht 原文直拷 .html）在 text/html 下整段 CSS 失效
  （chrome 实证）——oracle 用 .xht 案必须先剥 CDATA 壳。
- chromium oracle + REFTEST_DUMP 双侧对照可一步分清「test 侧缺陷 vs ref 侧模拟失真」
  （2.5 实证：ZW test 与 chrome test 同构 → 归因反转到 ref 侧表格模拟语义）。

## 5. 门禁与观察

- 零代码改动 → make test / make reftest / product-smoke / bench-gate 无义务（R5425 先例）。
- 单案 dump 取证均经 `./target/test-guard`（REFTEST_DUMP + --time-limit 300）。
- 对端 js_dom_bridge 五案红腿观察维持（零改动 → 与 R5432 lib 腿同名单不变）。
- /tmp oracle 截图与探针文件不进证据（规则 8）；reftest-dump PNG 属本地缓存不清理义务。
