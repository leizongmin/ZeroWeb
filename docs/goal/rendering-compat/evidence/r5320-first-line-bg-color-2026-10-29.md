# R5320 — ::first-line background-color 首格式行带（R5258 切片 2 bg-color 位收口）

- **日期**: 2026-10-29
- **驱动案**: `css/CSS2/css1/c25-pseudo-elmnt-000.xht` ✗2.80% → **✓0.32%**；连带 `css/css-pseudo/first-line-and-marker.html` ✗ → **✓0.50%**
- **corpus**: 819F → **817F**（16592 案，FIXED=2，REGRESSED=0）
- **域**: CSS2 §5.12.1（::first-line 适用属性——background 类）× R5258 切片 1 color-only 实现

## 定性

c25 页形态：`.test { color: red }` + `p:first-line { background: green }` +
`p.test:first-line { color: white }`（+ 含 `.two` 子类的 invalid 规则，flags=invalid）。
ref 期望两段白字绿带。实渲染：**白字已应用（R5258 切片 1），绿带缺失**——段落
白底白字不可见。INVALID 规则 `p:first-line.two` 被 matcher
`SubclassSelector::PseudoElement(_) => false` 无害丢弃（伪元素永不匹配 DOM 元素），
非缺口。根因 = R5258 切片 1 FIXME 注释自述「bg/font 类属性不应用」：

1. **style-system 存储门 color-only**：`compute_first_line_pseudo` 仅在
   `first_line.color != elem_style.color` 时存储伪样式——bg-only 页
   （`p:first-line { background: green }`，color 同元素）存 `None`，paint 侧无伪样式可取。
   c25 因 color:white≠red 恰好存储，暴露的缺口在 paint 侧。
2. **paint 臂只发字形 color**：R5258 臂解构 `((owner, ltb), _, _, _, _)` 丢弃行几何
   min_x/max_xw，无任何 background 通路。

## 实现（两处，无 kill-switch——沿用 R5258 切片 1 域先例，FIXME 切片结构即回退边界）

1. `style-system/lib.rs compute_first_line_pseudo` 存储门扩位：
   `color 差异 || background_color 差异` 才存储（collect 探针前置与帧体量纪律不变，
   R5258 文档记载的 background-image-first-line 误存回归不复现——探针空仍 None）。
2. `engine/paint/painter/text.rs` R5258 臂加 bg-color 位：首格式行（line_top == 组内
   最小，与切片 1 color 位同行判定）铺 `resolve_color_current(fl.background_color)` 带
   ——横向 = 行内全部片段范围并集（≡ R5054 选区逐行片段横向范围口径），高 = 盒
   line_height（≡ 选区行高解析），透明（a=0）不铺；带推入既有 `sel_line_fills`
   向量走既有 paint 位（先于字形，层序与选区高亮一致）。其余 bg 类
   （background-image 等）与 font 类仍不应用（FIXME 切片 2 维持）。

## 回归测试

`crates/engine/src/paint/tests/r5320_first_line_bg_tests.rs` +3 pin：
①首行单带 + 次行无带 + bg-only 不改字形 color；②bg-only 页（伪样式 color==元素）
仍存储铺带——**style-system 存储门专腿**；③透明 background 零带 + color 位照常
（切片 1 不回退）。**mutant 双腿（stash 单文件回退实测）**：仅回 paint 臂 → ①②
FAIL；仅回存储门 → ①② FAIL（bg-only 级联 None → line_ranges 不构建）——双向可判别。

## 门禁

- fmt 0；clippy zero-engine + zero-style-system `-D warnings` 0。
- **zero-engine lib 3585P/8F**：8F 全部为 js_dom_bridge fetch/Request/Response 契约案，
  **clean HEAD stash A/B 复现同红 = 既有记账**（归属并行流 net-api 提交 cc944c667
  URL 校验语义，非本轮修改面；paint:: 全绿含新增 3 pin + R5258 切片 1 8 案）。
- **zero-style-system lib 2450P/0F**；**zero-layout-engine lib 2225P/0F**（2 ignored 既有，
  style-system→layout 涟漪保险腿）。
- 本地 723 套件 722/723，唯一红 = `float-shrink-inline-children-zw-001`（R5311 既有记账）。
- corpus 全量：**817F**，FIXED=2（c25 ✓0.32% + first-line-and-marker ✓0.50%，后者
  `li::first-line{background:cyan}` bg-only 形同门受益）REGRESSED=0。
- product-smoke 全腿 PASS（welcome 15.68% ≤ 20% 预算；struct-check 0 issue；
  form-input-perf PASS）；product-smoke-legacy 42 fixtures struct 0 失败。
- **bench-gate 判定不适用**：新增计算全部在既有 `first_line_pseudo.is_some()` 门内
  （R5258 切片 1 已建），且再经 `fl_bg.a > 0` 短路——无 ::first-line 规则的元素零
  新增工作；不在 bench 测量的 block/flex_layout 布局热循环。

## 挂账

- **收口**：::first-line bg-color 位（切片 2 的 background-color 部分）；c25 +
  first-line-and-marker 双案落账。
- **维持**：bench-gate 静窗复测（跨 R5317/R5318 两轮）、非 img inline-block
  content/border 口径分裂、content-none-select-1 负载脆弱族、c44-003 0.47%
  length-va 残差、va 族余 122/baseline-003/intrinsic-size-009/R5314/R5315。
- **新增记账**：zero-engine js_dom_bridge fetch 契约 8 案 clean HEAD 同红（net-api
  并行流 cc944c667 域，渲染流不修，提请 net-api 流自查）。
- **FIXME 维持（切片 2 余量）**：::first-line 其余 bg 类（background-image）、font 类
  属性；::first-letter 同存首字符 interplay；竖排；跨行 inline 次行回落。
- **下一步 R5321**：①817F 账本续扫择轻（避开 va 域与 OOF 锚域）②css-pseudo 域
  邻案复查（本轮域连带面）③CSS2/css1 邻案（c548-ln-ht-000、c5525-fltmrgn-000、
  floats-031）。
