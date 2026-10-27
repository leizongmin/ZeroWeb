# R5300 — 嵌套 abs-pos shrink-to-fit 链 CB 宽传播 + css-text-decor-3 §2.6 decorating box（色/位/厚）

**日期**: 2026-10-27
**驱动案**: `css/css-text-decor/text-decoration-subelements-002.html` ✗1.07% → **✓0.00%**
**corpus**: 交付态 **16591/15734P/857F**（探针页删除后全量复跑定版）；失败集 vs R5299 基线
（16591/15733P/858F）归一化 diff = {−002} 恰一案移出零翻转。轮中取证期本地探针页在场时
失败集 diff = {−002, +r5300-probe-shrink（gitignored 本地探针，见下）}，非 corpus 回归。

## 缺陷 1（layout-engine）：嵌套 abs-pos 链的过期 CB 宽钳死

### 根因

ref 页构造 `abs(wrapper, 仅 abs 子) > abs(overflow:hidden, in-flow 文本 + 元素子) > abs(top:0 覆盖层)`。
`apply_abspos_shrink_to_fit_width` 单趟 walk 用 **taffy 首趟布局宽**作子代 `cb_width`；当父 CB
本身也是未解析的 shrink-to-fit abs-pos 时，子代 `target = min(measured, stale_cb)` 被钳死
（stale_cb = 父代塌缩宽 0 或最宽元素子），写回门 `b.width < target_bw - 0.5` 判负不写，
单趟 walk 不迭代 → overflow:hidden 盒停在 13px（= 最宽元素子 `<i>` 宽）裁掉全部内容。

取证（临时插桩 trace）：wrapper A cb=800 正确写 746；中层 B cb_w=**0**（过期）钳死不写；
修复后 B cb=746（A 写定回写）→ B 写 338 → 覆盖层 C cb=338。

### 实现（sizing.rs，kill-switch `ZW_ABSPOS_STF_CB_PROP=0`）

- `propagate_resolved_cb_width(stack, b, child_is_cb, resolved)`：本 pass 给父代写定的
  border-box 宽即时回写子代栈槽 `cb_width`。栈顶 `b.children.len()` 项即循环顶刚 push 的
  子代（处理中无其他 push），LIFO 保证子代紧随本盒弹出。仅 `child_is_cb` 时有意义；
  kill-switch OFF 或 None 不回写，子代维持旧行为。
- 三个写定点：R4238 stretch 臂（写后、continue 前）、shrink-to-fit 写回臂、臂尾兜底。

### 回归（tests/r5300_abspos_stf_cb_prop_tests.rs，R5236 模式）

- `r5300_nested_abspos_branch_text_shrinks_like_top_level`：嵌套 branch 内层应与顶层参照
  同宽单行（旧行为塌 13 = 最宽元素子）。
- `r5300_three_level_abspos_chain_propagates_cb_width`：ref 页全形态三层链，中层 = in-flow
  文本 shrink-to-fit。
- A/B：默认 ON 双绿；`ZW_ABSPOS_STF_CB_PROP=0` 双红（旧行为复现）。

## 缺陷 2（engine paint）：css-text-decor-3 §2.6 decorating box

test 页 `div.test{text-decoration:underline}`（fs=32）直系+`<i>` 后代文本连续下划线；
ref 页 in-flow 部分无装饰，用 `span{underline} > span.hide{color:transparent}` 合成连续
黑下划线，被 overflow:hidden 收缩包络裁到文本宽。

### 2a. 装饰色取声明盒（text.rs，kill-switch `ZW_DECOR_ANCESTOR_BOX=0`）

- **容器臂**：`render_fragment!` 容器/声明盒自身装饰色由片段色（`frag_color`，含
  selection/clip:text 覆写）改为**声明盒解析色**（`color`）。透明文字片段的装饰线随片段
  色整条隐形是 002 ref 侧直接病灶；直系同色文本二者相等（字节不变）。
- **祖先链臂**（新）：owner→容器祖先链上每个声明 `text-decoration` 的盒各绘一条独立线
  （CSS 2.1 §16.3.1 + §2.6 传播语义，原 FIXME「更浅祖先链缺失」）。装饰色解析序：
  显式 `text-decoration-color` → 声明盒解析色 → currentColor 沿祖先链取首个显式色，
  兜底容器解析色。链走到 `box_node` 自身即止（容器臂已覆盖）。旧行为（owner 自声明 +
  片段色）保留在 kill-switch OFF 臂。

### 2b. 下划线位置/厚度取字体 post 表（forensics 主线）

颜色修复后残差 0.28%（1346px）：test 侧下划线占 3 设备行（118–120），ref 侧仅 1 行（118）。

取证链：
1. 行扫描：test 行 118/119/120 全暗（各 305px），ref 行 119/120 全白——同一段 x 域。
2. 红染探针：`paint_decoration_line` 填充全部染红 → 三行 ink 全部来自装饰线（无神秘路径）。
3. `add_fill`/`add_stroke` 图元探针：**无 y=118 的图元**；`fill_rect` 设备行探针：
   `css_top=118.996` → `floor(118.996)=118, ceil(120.916)=121` → 行 118–120。根因是
   **分数坐标 + floor/ceil 外扩**：基线 114.196 + 0.15×32 = 118.996。
4. 盒几何：ref 侧 B（overflow:hidden）高 38、底缘 = 81+38 = **119.0 = descent 底**
   （本引擎 descent 比例恰 0.15em）——下划线偏移 0.15em 恒落在行盒底缘之外，收缩包络
   祖先整条裁掉 119+，仅 floor 外扩的行 118 泄出。

修复（css-text-decor-3 #text-decoration-width/#text-underline-position，chromium 同源）：

- render-foundation `FontLoader::underline_metric_aspects(font_id)`：ttf-parser `post` 表
  `underline_metrics`（字体单位 y-up → em 相对，基线下方为正）。Liberation Serif
  （corpus 默认字体）：0.0601em / 0.0488em；Ahem 0.133em / 0.02em。表缺失/线厚非正 → None。
- engine `text_metrics.rs`：宿主回调 `set_underline_metrics_fn` +
  `underline_metrics_for_paint(font_id)`（OnceLock<fn>，同 char_measure 桥接口径）。
- effects.rs `paint_text_decoration_from_style` 新增 `font_id` 参数：underline 位置
  `post.underline_position`（回退 0.15em），auto 厚度 `post.underline_thickness`
  （回退 0.06em）；显式 Length 厚度与 R2522 offset 不变。overline/line-through 位置
  无字体度量对应，维持启发式。text.rs 五个调用点传 `frag_font_id.0`（祖先链臂用片段
  字体近似声明盒字体，二者继承同源时相等——注释在案）。
- 宿主注册三处：wpt-runner（runner_text_metrics + main.rs）、apps/renderer（text_metrics +
  runtime.rs）、apps/browser（text_metrics + app.rs + tab_worker.rs），同 MEASURE_CTX 模式。
- fs=32 效果：ink 116.12–117.68，行 116–117，全部在 B 底缘 119.0 之上 → ref 不再被裁。

### 试错记录

- 探针页命名坑：`r5300-probe-ref.html` 撞 wpt_file_loader 引用页跳过规则
  （stem 以 `-ref` 结尾不作为 case），改名 `-refpage`。
- 0.28% → 误读 1% 阈：002 fuzzy meta `maxDifference=0-80;totalPixels=0-42` 要求 ≤42px，
  比率启发式不适用。
- [r5300d]/[r5300a] 打印 `y=119.00`（{:.2} 舍入）掩盖 118.996 分数坐标——设备行探针
  （floor/ceil 之后打印）才定谳；取证打印须锚定最终消费坐标。

## 交付

- 002：✗1.07%（轮首）→ ✗0.42%（缺陷 1 修复）→ ✗0.28%（2a 色修复）→ **✓0.00%**（2b 位修复）。
- corpus 交付态 **16591/15734P/857F**（探针页删除后全量复跑定版，三态分类
  10260+531+4943+857 = 16591 对账）；失败集 vs R5299 基线（858）归一化 diff 恰 {−002}
  零翻转。轮中取证期失败集 diff = {−002, +r5300-probe-shrink.html}，后者是本轮 bug
  隔离用 gitignored 本地探针页（compare about:blank，布局修复后内容渲染变多 >1% 阈），
  非 corpus 回归；交付前已连同其余 r5300-* 探针页从 wpt-data 删除，corpus 恢复
  16591 基数。
- kill-switch A/B：`ZW_DECOR_ANCESTOR_BOX=0` → 002 回 ✗0.28%；`ZW_ABSPOS_STF_CB_PROP=0`
  → 回 ✗0.87%（两开关各证各的修复）。字体度量路径无开关（数值精化，corpus 零回归验证）。
- 门禁：make reftest 716/716、fmt 0 diff、clippy 0 告警（含 quickjs feature 组合
  13 crate `-D warnings` 腿）、product-smoke PASS（struct-check ×2 viewport）、
  make test 全量（集成腿 image_batch/network 并发窗 2 flake 隔离复跑 2/2 绿——R5295
  同族记录不修，其余腿全绿含 quickjs 测试腿）、bench-gate 定向
  `ZERO_WEB_BENCH_CRATES=zero-engine,zero-layout-engine` **GATE PASS**
  （25 指标 NEW=7 全在预算内，本轮无环境噪声争议）。
- 回归：+2（layout-engine r5300 链传播）；paint 测试签名适配 17 处（font_id 参数）。
- 单测无注册回调 → `underline_metrics_for_paint` 返 None → 启发式回退 → 既有装饰断言
  （0.15em/0.06em 域）全部不变。
