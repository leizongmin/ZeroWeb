# 刀 Q：shape-outside specified 声明值 canonical 串化（css-parser 域，p1）

日期：2026-11-18 · 提交 `3788f6c99` · 域：css-parser（fold_decl_specified 新臂 + canonicalizer + 4 钉测，净 +297 行）

## 定谳

css/css-shapes `shape-outside/values/` inline（specified）读出通道失败根因 = `testInlineStyle`
经 `SetStyleOnHandle` → `style_from_mutations_lw` → `fold_decl_specified`，shape-outside
此前不在串化白名单 → 原文回显 ≠ oracle canonical 形（b0 基线 344F：circle/ellipse-002
位置补全 29F×2、circle/ellipse-004 位置组 143F×2、inset-003 slash 折叠 52F）。

修复 = 文本级单源 `canonicalize_shape_outside_specified`（css-shapes-1
§shape-outside-property + css-values-3 \<position\>）：

- 单 token 补全：len/pct → `{tok} center`；水平缘 → `{tok} center`；垂直缘 →
  `center {tok}`；center → `center center`。
- 双关键字水平先行重排：`top left`→`left top` 全四缘组合；**center 混合形**
  （探针 p1 余 4F 钉出的缺口）`top center`→`center top`、`center left`→
  `left center`（center 归缺失轴 + x 轴先行）；已 canonical 形 passthrough。
- 4-token edge-offset 对：首对垂直（`top 50% left 50cm`）→ 水平对先行；len 用
  `parse_shape_len` 共享解析器裁形。
- inset round：`/ <vertical>` 垂直组与水平组等值时折叠（`inset(10px round 5px / 5px)`
  → `inset(10px round 5px)`）；非等值与 1–4 文法外 passthrough。
- 守卫：空串/CSS-wide keyword/`var(`/`attr(`/shape-box 前缀（content-box 等，
  剥离保形回拼）passthrough；未知函数名 passthrough。

文本级而非 AST 往返的原因：物理单位 parse 期已折 px（types.rs `finite_px`，
50pt→66.666…px），AST 往返丢原单位；oracle specified 面保单位形（`50cm center`），
故只能文本级改写。先例复用：g47 object-position `canonicalize_position_decl`
（callbacks.rs）同语义 Rust canonicalizer。

## 探针账（单页 testharness-css-shapes，test-guard 包裹）

基线 b0 = results-knife-p-b2（shapes 全 corpus，md5 存档正式 bin）；p1 探针 bin
md5 `c0bbdb62`（canonicalizer 无 center 混合形）；p2 探针 bin md5 `76b768e6`
（补 center 混合形，提交态 3788f6c99 重建）。

| 页 | b0 | p1 | p2 | 净清 |
|----|----|----|----|------|
| circle-002 | 29F | 4F | **0F**/136 | 29 |
| ellipse-002 | 29F | （镜像） | **0F**/136 | 29 |
| circle-004 | 206F | 63F | （同 p1 面） | 143 |
| ellipse-004 | 206F | （镜像） | **63F**/780 | 143 |
| inset-003 | 292F | 240F | （同 p1 面） | 52 |

**净清 396F（逐页探针实测 416F 含镜像页重复计）**，inline 面全清；余 fail 全为
engine computed 读出通道（gCS）域。

## 余面分解（p2 候选，engine 域）

- **B 精度**：`format_num`（computed_style.rs:2092）3 位小数 vs oracle
  `parseFloat(n.toPrecision(6))` 6 位有效数字——`66.66666…`→我方 `66.667` vs
  oracle `66.6667`（ellipse-004 63F 样本实证）。全局影响需 A/B 计账。
- **C-computed**：inset 多半径截断（ClipPathValue::Inset 单 ClipPathRadius，
  parse 期丢垂直组）240F，需 AST 改型，与对端动画域 clip-path 插值高碰撞，
  PARKED 协调后另刀。
- **D calc 非 Px 叶**：`length_to_css` Calc 臂 `calc_computed_to_css(...).unwrap_or_default()`
  求值失败返空串（`circle(at 50% calc(100% - 896px))` → `circle(at 50% )`）。
- **E vmax 半值谜团**：plain `circle(at top 160vmax…)` 期望 1280px 我方 640px
  （uniform ×0.5），静态追踪穷尽未定位，需插桩定谳。
- **F ch 度量**：oracle ch=16px（Ahem 1em），我方 fallback 0.5em=8px。

## 门禁

zero-css-parser **3302 passed / 0 failed**（含新增 4 钉测：位置补全、关键字重排
+center 混合、inset slash 折叠、passthrough 守卫）；clippy 零告警；fmt clean。
提交 `3788f6c99` push 完成；探针工件 `.acceptance/wptfyi-css/results-knifeq-p{1,2}/`
（诊断产物不入库）。
