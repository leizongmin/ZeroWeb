# R5411 证据：no-ratio SVG flex item 三断点修复（flex-svg-no-intrinsic-column-001 ✗4.69%→✓0.00%）

日期：2026-11-04。驱动案：`css/css-flexbox/flex-svg-no-intrinsic-column-001.html`
（column 容器 150×0 + 无声明 `<img src="data:image/svg+xml,<svg xmlns=…/>">`，`img{background:green}`，ref 应 150×150 绿方块；rel=help → css-flexbox #min-size-auto）。

## 1. 归因：bogus 解码 pixmap 三处泄漏

no-ratio SVG（CSS2 §10.3.2：width/height 非双绝对且无 viewBox）无固有尺寸、无固有比；
css-sizing-3 §5.1.1 default object size 300×150 兜底。runner `extract_image_metrics`
经 usvg 解码无固有 SVG 时回报默认 pixmap 100×100（bogus），与权威 `no_ratio (None,None)`
信号**并存**于 `image_sizes` 缓存——三处消费点把 bogus 尺寸/比当真值：

| # | 消费点（修复前） | 症状（驱动案 A/E 象限） |
|---|---|---|
| ① | `tree.rs` R4996 `set_replaced_content_suggestion_min` 读 bogus sizes (100,100) → `min_size.height=100` | 内容建议地板 100 ≠ default object size 150，shrink 后 h=100 |
| ② | `tree.rs` R2440/R4160 sizes 臂把 bogus (100,100) 作比注入 `aspect_ratio=1.0`（`aspect_ratio_auto` 消费） | flex-start 形态 w300 × ar1 → h300（应 300×150） |
| ③ | `postprocess.rs` R1411 `fix_column_flex_nonstretch_replaced_main` 非 stretch 臂 `expected_main = w×(ih/iw) = 300×1` 改写 item.height | LayoutBox 层把 taffy 已定的 150 高改成 300（H1/H4 探针定谳） |

探针隔离链（`probe_r5411b`，用后即删）：H1（sizes+no_ratio 双 map，flex-start，容器 200）
渲 300×300；H5（仅 no_ratio）渲 300×150；两态 taffy 终态 style 逐字节一致
（w=300/h=150/minh=150/ar=None）→ 差异必在 taffy 之外的 LayoutBox 域 → R5411FINAL
插桩（engine.rs merge 块前）排除 re-run 组 → BOX walk + 比值 1:1 签名锁定 ③。

## 2. 修复（kill-switch `ZW_FLEX_NO_RATIO_ITEM=0`，default-on，四处同门）

1. `set_replaced_content_suggestion_min`：no-ratio 权威信号**优先**于 sizes——
   命中即取「显式固有主维或 default object size 主维（宽 300/高 150）」+ 无比
   （`intrinsic_ratio=None`，eff_ratio 回落 taffy style 已有值）。
2. R2440/R4160 sizes 臂：no_ratio 命中即不注入 `aspect_ratio`（无固有比，CSS §10.3.2）。
3. no-ratio 臂 both-auto 腿（R5411 新增轴纪律）：flex item 的 default object size
   **只入主轴**（flex base 语义）；交叉轴仅当 align-stretch 确将生效（stretchy ∧
   容器 cross definite，与 sizes 臂 cross_stretch_definite 同判据）留 auto 让位
   stretch——否则写 default（CSS2 §10.3.2，flex-start 不塌 0）。非 flex item
   块流双维 default 不变（R2428 判例维持）。
4. R1411 非 stretch 臂：`img_no_ratio.contains_key(&id)` 命中即 continue（bogus 比
   不得传递；taffy 面已按 default object size 给 150）。cross_stretch 臂（R4224
   width:stretch 传递）维持现状——无驱动案，同族毒性记挂账。

## 3. 探针 A/B（修复后，runner 同语义 map 填充）

| 形态 | 修复前 | 修复后 | 应 |
|---|---|---|---|
| stretch 容器 h=0（驱动案） | ✗ | (8,50,150,150) ✓ | 150×150 |
| flex-start 容器 h=0 | — | (8,8,300,150) ✓ | 300×150 |
| flex-start 容器 h=200 双 map | (8,8,300,300) | (8,8,300,150) ✓ | 300×150 |
| stretch 容器 h=200 仅 no_ratio | — | (8,8,150,150) ✓ | 150×150 |
| sizes_only（无 no_ratio 信号） | — | 100×100（旧行为不变） | 信号缺失域 |

## 4. 常驻测试

`crates/engine/tests/r5411_flex_svg_no_intrinsic.rs` 2 针：
- `r5411_no_ratio_column_stretch_to_container` ★load-bearing：驱动案形态
  (8,50,150,150)——bogus 100 pixmap 不再遮蔽（min 地板 150）+ 交叉轴让位 stretch。
- `r5411_no_ratio_non_stretch_keeps_default_cross`：flex-start 形态 (8,8,300,150)——
  交叉轴保持 default 300（让位逻辑不塌 0）+ R1411 臂不传 bogus 比。2/2 绿。

## 5. corpus 验证

- 单案 ON ✓0.00%（修复前 ✗4.69%）；kill-switch OFF 同二进制精确回 ✗（R3858 口径）。
- 全量 A/B 双窗（16594 案，BASE=同二进制 `ZW_FLEX_NO_RATIO_ITEM=0`）：
  - 窗1 OFF 15875 / ON 15872：FIXED {001, intrinsic-percent-replaced-dynamic-005}、
    REGRESSED 候选 5（insert-block-in-blocks-middle-001、border-collapse-dynamic-column-002、
    table-anon-128/151/152）。
  - 窗2 OFF 15874 / ON 15876：FIXED {001 + insert-end-002 + conic-repaint}、
    REGRESSED 候选 dynamic-008。
  - 交叉定谳：001 两窗均 FIXED＝唯一稳定归属；其余 8 个候选全部单案隔离
    ON/OFF 双侧 3×3 全绿＝在册抖动族（table-anon、insert-block、css-sizing dynamic-00x、
    repaint；guidelines #35）。**真态 FIXED={flex-svg-no-intrinsic-column-001}
    REGRESSED=∅，corpus 真态 716F→715F。**
- 附带红利（不可归属）：intrinsic-percent-replaced-dynamic-005 窗1 FIXED、窗2/单案
  双侧均过＝dynamic-00x 抖动族成员，不计 FIXED。

## 6. 门禁

fmt ✓；guarded-clippy 全工作区 EXIT 0（R1411 既有单测调用点补第 4 参）；make test
唯一红腿＝js_dom_bridge CSSOM 对端账在册 5 案（p56-adopted/p99s-rule-style/
r3276-add-rule/r2810-rule-style/r2809-write，名单与 R5408-R5410 同）其余 66 腿全绿
——`git diff HEAD -- crates/engine/` 为空位元证明 HEAD 既有非本轮引入；product-smoke
主腿全 struct-check PASS + form-input-perf-gate PASS（exit 0）、legacy struct-check
failures: 0；bench-gate 定向 zero-layout-engine GATE PASS 25 指标（NEW=0）×2 ON 腿
＋同窗 kill-switch OFF 配对腿（medium/layout ON 157.56/154.87 vs OFF 163.75——环境
漂移非本轮负载，ON 不劣于 OFF；R5410 配对对照口径）。

## 7. 挂账（本轮新增）

1. **img 臂 R4159 continue 纪律缺失**（`pipeline/mod.rs` build_img_intrinsic_all
   757-792）：img 路径 sizes/ratios/no_ratio 三 map 并行 insert，无 object/embed 臂
   的 continue 互斥——bogus sizes 与 no_ratio 并存即本轮三断点的上游源头。深修需
   核对 R695 quirk fallback 与 R2172 attr-ar 消费者（`<img src=no-ratio-svg width=100>`
   attr 域 100×100→100×0 变化，缺驱动案）。
2. **R1411 cross_stretch 臂同族毒性**：width:stretch no-ratio SVG 会被 bogus 比传
   递 main（flexbox-auto-minimum-001/002 为 canvas attr 合法比判例，无 no-ratio
   驱动案，本轮不动）。
3. **G 形态观察**：显式 `height:100px` + bogus sizes 的非 flex 域形态（探针 H3 谱系）
   旧行为维持，无 corpus 驱动，暂不动。

## 8. 工件

- 修复：`tree.rs`（no-ratio 优先/AR 排除/轴纪律）、`engine.rs`（R1411 装填 no_ratio
  副本）、`engine/postprocess.rs`（R1411 非 stretch 臂 no-ratio 门）。
- 全部 TEMP 插桩（R5411DBG×2 / R5411PASS×4 / R5411R695 / R5411FINAL）已摘除；
  探针 probe_r5411b.rs 用后即删；`engine/sizing.rs` 回到 HEAD 位元。
