# R5513 轮次证据：flex item calc margin 固有测量 px 分量缺失——R4136 口径入 flex 臂

日期：2026-11-13
轮次：R5513（corpus FAIL 盘点择案轻量，R5512 后继）
择案：css-sizing/calc-margins-flex（✗2.08%，685F 在册零提及，684F 时代即稳红）
结论：根因＝`flex_row_intrinsic_width`/`flex_column_intrinsic_width` 裸读 post-taffy
`child.margin_*`，calc(P%±Npx) margin 的 px 分量在 convert 层被剥离、% 分量在固有
语境无基准解析为 0 → 容器 intrinsic 塌 0；修复＝两臂接入既有 R4136
`intrinsic_margin_contribution`（calc 取 px 分量、普通 margin 照用）。002 案
✗2.08% → ✓0.00%（3× solo 稳）。

## 一、缺陷签名

- `div{display:flex; width:min-content; height:100px; background:green}` +
  空 item `margin-left: calc(10% + 100px)`；ref=ref-filled-green-100px-square-only。
  css-sizing-3 #intrinsic-sizes + csswg #823066：内容尺寸测量中 margin 的 % 分量
  对零解析，calc 长度分量存活 → item 贡献 = 0 + 100px，容器 min-content = 100。
- ZW 容器宽 0、绿面整缺：diff = 10000px² / 480000（800×600）= 2.083% ≈ 报告
  2.08% 精确吻合（缺失面积恒等驱动面，无需 PNG 佐证）。

## 二、归因链

1. convert 层：`convert_length_to_lpa` 对 Calc margin 只提取 % 分量喂 taffy
   （`calc(10% + 100px)` → Percent(0.1)），px 偏移静默丢弃（在册注释：px 部分由
   R4136 布局后处理补）——taffy 侧 LPA（本地版 CompactLength）无 calc 组合表示，
   opaque calc 指针路线需覆写 `resolve_calc_value`（两处默认 0.0，未接线）。
2. intrinsic 语境：`apply_intrinsic_content_sizing`（sizing.rs，flex/grid 容器宽度
   提升臂）对 width:MinContent flex 容器走 `flex_row_intrinsic_width` 测量——该臂
   item 贡献 = `flex_item_base_size` + **裸 `child.margin_left + child.margin_right`**
   （post-taffy 值；首趟 taffy % 无基准 → margin 0）→ 容器 intrinsic 测 0 →
   promotion 写 0 → 绿面整缺。
3. R4136 已为 block 域建 `intrinsic_margin_contribution`（calc 取 px 分量、普通
   margin 照用 post-taffy；消费点 box_content_max_width_inner:716 /
   block_intrinsic_width_ex:1582）——flex 两臂漏接。grid 臂（grid_intrinsic_width_ex
   两处裸 margin）同病，本轮不动、记账挂账。

## 三、修复（intrinsic_sizing.rs 两臂各 1 行 + 注释）

- `flex_row_intrinsic_width`/`flex_column_intrinsic_width`：margin 项改经
  `intrinsic_margin_contribution(child, styles)` 取 (ml, mr)。非 Calc margin 逐字节
  不变（该 helper 对普通长度返回原 post-taffy 值）；仅 Calc(P%±Npx) 形在固有测量
  切换为 px 分量口径（R4136 既有契约，负 px 分量 .max(0.0) 同既有口径）。
- promotion 链路零改动：flex+MinContent 走既有 `b.width < intrinsic` grow 臂写
  定宽重排（definite 布局相位 % 分量由 R4136 apply_calc_margin_adjustments 照旧）。

## 四、锚测试

`r5513_flex_calc_margin_intrinsic_tests`（3 案）：row 容器 min-content = 100
（WPT 驱动案形）、column 容器 cross = 100、普通 px margin 40 不变（回归守卫）。

## 五、验收

- lib：2495P/0F（2492 + 3 新锚）。
- solo：calc-margins-flex ✗2.08% → ✓0.00%，3× 复跑稳（#35 协议）。
- corpus 全量 A/B（16594 vs /tmp/r5512/new-fails-r5512-v3.txt 685F）：
  **FIXED={calc-margins-flex} REGRESSED=∅**；批跑表观 3 红
  （border-collapse-dynamic-table-002 + table-anonymous-objects-152/154）solo 2×2
  全 ✓0.00%＝表族互翻 flaky 态（R5418/R5420/R5421 同族先科），非本轮因果。
- make reftest：725/725 100%。
- make test：3911P/0F 主腿 + **7F＝js_dom_bridge 对端在册族恰集复现**（img/svg
  settle 2 + fetch 2 + websocket 2 + iframe-sw 1，与 R5512 同集；本轮改动面
  intrinsic_sizing.rs 布局测量零涉网络桥面，R5504「7F＝对端在册族 base 复现归因」
  先例口径；renderer/browser 腿未跑属对端门禁债）。首轮 timeout 3600 EXIT=124＝
  对端 clone 构建风暴（host b=245）击穿 integration-tests 编译相，缓存热重跑即净。
- product-smoke：form-input-perf PASS + welcome chromium oracle **17.23%**（R5501
  基线同值）+ welcome/morning ×2 全 struct PASS。
- fmt + clippy：`cargo fmt -p zero-layout-engine`；`cargo clippy -p
  zero-layout-engine --all-targets -D warnings` 净；workspace guarded-clippy 见
  /tmp/r5513/clippy.log。

## 六、本轮动作与卫生

- 候选盘点（R5513①）：685F 小差带全 ledger 对账排除——slice-inline-fragmentation-003
  （R4409 遗留① css-break-3 paint 特性工作）、border-radius-clip-001（R5421 残差＝
  设防角裁剪臂，挂账⑤ backdrop/filter 前置）、featureless-001..005（R5344 shadow
  DOM scoped 结构域）、display-contents-shadow-dom-1（R4324 ③）、display-change-iframe
  （iframe 重排动态域）、block-size-with-min-or-max-content-table-1a（R5128
  ZW_LOGICAL_SIZE 机制挂账）、fullwidth-004/005（0.12/0.17% 阈值盲区带）、
  textarea-pre-wrap-013（textarea 处置域）、letter-spacing-ligatures-005（待决策③）、
  slider/file-selector 伪元素族（表单域）、ortho-writing-mode 族（正交深域）、
  background-color-animation（动画时钟域）。calc-margins-flex 账本零提及、案面
  最窄，选中。
- 临时探针无（归因全走代码链 + 数值自洽），无诊断单测残留。
