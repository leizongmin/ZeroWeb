# R5178 替换元素 min/max 尺寸 content 关键字 transferred/attr 固有

日期：2026-10-18 ｜ 前轮：R5177（inline 壳 color 折回归因拆分） ｜ 基线 corpus：1,077（R5177 ON 终态）

## 案形与域核

css-sizing 6 案族（单案 0.29-8.33% 不等）：替换元素（canvas/svg 等）设 `min-width:min-content` / `max-width:min-content` / `min-height:min-content` / `max-height:min-content` 配合另一轴 definite 尺寸，期望盒钳到 100×100，ZW 保持声明尺寸（10×100 / 1000×100 / 100×10 / 100×1000）。svg 变体 attr `width="0" height="100"`（width=0 → 无有效固有比）退 attr 固有高。

根因链：converter 把 min 侧 content 关键字映射 `length(0)`、max 侧映射 `auto`（taffy Dimension 语义），且 R4149 主臂 `!b.is_replaced` 排除替换元素 → min 钳成 0 等于没有、max 钳成 auto 也没有 → content 关键字在替换元素上完全失效。规范依据：css-sizing-3 §5.1 + csswg #3973——替换元素 min/max 尺寸的 content 关键字按 **transferred size** 解析（固有比 × 正交轴 definite 尺寸），无 definite 正交轴时退固有尺寸；css-sizing-4 §3.2——CSS `aspect-ratio` 生效时自然尺寸弃用，固有块尺寸 = 固有宽 ÷ 比。

排除面：flex/grid item 的块轴 min-*/max-* content 关键字走 css-flexbox §4.5 自动最小尺寸（content-based min(内容建议, 指定建议)），非 §5.1 固有/转递——`!b.is_flex_grid_item` 双臂排除（corpus flex-minimum-height-flex-items-023 判例，首次迭代翻红后加闸）。IFC 内联原子仅做 min-height floor（max 侧归 R5110 postprocess `clamp_replaced_min_content_keywords`，不重复）。

## 实现（三处）

- **宽轴 taffy 臂**（`crates/layout-engine/src/engine/sizing.rs` `apply_intrinsic_content_sizing`）：R4215 prescan 命中替换元素含 min/max-width content 关键字时，比 = CSS `aspect_ratio` → `replaced_intrinsic` 表（attr 双值），transferred = definite 高 × 比；无 definite 高退 attr 宽。写 taffy `min_size.width` / `max_size.width` 后走既有 rerun。
- **高轴 taffy 臂**（同文件 `apply_block_axis_content_sizing`）：对称臂，transferred = definite 宽 ÷ 比，HorizontalTb 限定（垂直块轴 = R4560 别名域）。固有块尺寸退让链：有比 → `attr_w / ratio`（css-sizing-4 §3.2，R3796 canvas 判例：attr 100×50 + aspect-ratio:1 时 max-content 高 = 100 非 50）→ 无比 → attr 高。两臂均签名加 `doc + &intrinsic_for_r695`（复用 R695 attr 表，零新增扫描）。
- **IFC 原子 floor**（`crates/layout-engine/src/inline/collect_items.rs`）：`remeasure_inline_only_containers` 同步 IFC 原子高度会覆盖 taffy min_size（svg min-height 案单测根因；runner 页因 R4108 mixed_all_replaced_inline 门跳过 remeasure 才假绿）——原子盒 min-height content 关键字时按 `w / ratio`（退 attr 高）floor `h`，仅水平书写、仅 min 侧、限 svg/canvas/video/iframe/embed/object/applet。

kill-switch：无独立开关（臂内联在既有 sizing pass 与 collect_items），回退需还原三处。

## 迭代定谳（corpus A/B 同 binary）

| 迭代 | 结果 | 归因 |
| --- | --- | --- |
| ON v1 | 3 案翻绿但 **flex-minimum-height-flex-items-023 翻红**（8.33%） | flex item 块轴 min-content = §4.5 自动最小尺寸非 §5.1 转递——双臂加 `!is_flex_grid_item` |
| ON v2 | r3796 max-height:max-content 单测翻红（100×100 → 50×50） | attr 高退让先于比推导触发；改 `intrinsic_h = attr_w / ratio`（自然高弃用）后回绿 |
| ON v2 重编 release | **1,074 翻绿恰 3（3 案 0.29-8.33% 全→0.00%）翻红 0** | 逐项 diff 干净；svg-no-ar 两案（min/max-height）同轮转绿 |
| ON v3（探针清除） | 仍 1,074，失败集与 v2 逐项一致 | 临时 eprintln 探针为纯诊断输出，行为零变化 |

## bench 判别（A/B/A 同刻对照）

bench-gate 首窗 FAIL 31 项，失败横跨本轮未触碰 crate（zero-net parse_url +78%、zero-script-sandbox +115%、zero-render-foundation raster +106%）→ 疑环境。stash 基线窗自身 FAIL 12 项（含 layout trio block_layout/flex_layout/incremental 超预算）→ 门禁预算与当前机器状态失配。

A/B/A 同 binary 交替窗（ON-A1 / BASE-B2 / ON-A2）判别：**每个指标 ON ≤ BASE**，且 A2 在与 A1 相同 binary 上复现尖峰 → 机器噪声非代码回归。判负：环境性 FAIL，零 relax、零基线改（R5172-R5177 噪声判例延续；不为过门禁改测量配置）。

## 验证

- unit：`r5178_replaced_content_kw_tests.rs` ×7——min/max-width、min/max-height transferred 四案（canvas attr 10×10 → 100×100）、svg 无比 attr 退让两案（attr 0×100 → 100×100）、flex item 排除守卫（负空间断言 h<149，本 harness 无解码像素；完整语义由 corpus flex-minimum-height-flex-items-023 钉住）。
- 既有钉维持：zero-layout-engine **1,997 绿**（1,990 + 7 新）；zero-engine lib **3,170 绿**。fmt / clippy（workspace --all-targets -D warnings）干净。
- corpus 16,590 A/B 同 binary：基线 1,077 / ON **1,074**（翻绿恰 3：replaced-min-width-min-content、replaced-min-height-min-content、svg-no-ar-min-height-min-content / 翻红 0，逐项 diff）；svg-no-ar-max-height 与 replaced-max-width/max-height 原绿同轮 import 钉住（共 6 案入常驻集）。
- make reftest **712/712** EXIT=0（含 6 新案后）。
- product-smoke **EXIT=0** 全 struct PASS（layout 变更必跑）。
- bench-gate：见上节，FAIL 归环境（基线窗自身 FAIL + A/B/A ON ≤ BASE），零 relax。

## 挂账（新增/延续）

- **IFC 宽轴 min 臂未做**：内联原子 `min-width:min-content` 走 IFC 行内盒宽推导（当前无翻红 driving case），与本轮块轴 floor 不对称，待有 case 再域核。
- **outline-028 仍红**（R5177 延续）：floated descendants + max-content sizing 案形，独立域核。
- backdrop-root 结构性挂账（R5176 tail 定谳，延续）：filter/opacity/mask/mix-blend-mode 4 案需三层联动。
- 延续：transforms 余 44、filter-effects 20、cascade 16、dirty-rect 相位不一致、body-background 传播缺陷、preserve-3d 完整域、rotateY 域 reveal、apply_transform_post 下层恢复根修、perspective-origin-scroll 恒值簇、R5172 挂账组。
