# R5298 证据：混合 line-height 行盒 strut 参与（CSS2 §10.8.1）

**日期**: 2026-10-27
**轮次**: R5298
**驱动案**: css/CSS2/css1/c42-ibx-ht-000.xht ✗1.19 → ✓0.83
**语料**: 16591 案（含自家 gitignored probe）15732P/859F → **15733P/858F**，失败集归一化 diff 恰 {−c42-ibx-ht-000}，零新增翻转

## 根因

CSS2 §10.8.1：块容器的 strut（假想零宽 inline 盒，字体/行高取容器自身样式）参与**每一**行盒。引擎原以「本行 run 的 line-height 最大值」近似 strut——纯小行高 run 行 strut 蒸发：

- c42：div `font: 10px/12px Ahem` 折 5 行，em `font: 1em/1`（lh 10px）词独占 L2。近似令 L2 行盒缩至 10、基线 8，行距 11 vs chromium 12（div 高 58 vs 60）。
- LAYOUT_DUMP 实证（`ZW_DEBUG_IFC` 链路）：TEST div ch=58（11+11+12+12+12）vs REF 60。

## 实现（layout-engine/inline，无新 kill-switch）

1. **捕获**（`inline/mod.rs` `layout()`）：`resolve_font_metrics_with_provider`（run 侧同源解析器，normal 整数化 / Number / Length 同口径）取容器自身 used line-height 存 IFC 新字段 `container_strut_line_height` + `container_strut_is_ahem`。**恒覆写**（0 也写入）并在入口先复位——IFC 实例跨容器复用 / anon 容器时残留值会把外层 strut 泄漏给内嵌行。
2. **胜出才接管**（`apply_vertical_alignment` strut 臂）：仅当容器 strut ascent 严格大于 legacy 近似（本行 run 的 line-height/fs max）时替换 strut ascent；否则（含相等）返回 None 完全走旧行为。legacy 近似会「骑」本行最高 run（含 va-shifted 等不入 ascent 环的 run），corpus 已按其校准——容器 strut 只托底、绝不贬基/混搭抬升。
3. **descent 侧行盒下限 + 归因行顶平移**：接管行补 `line.height ≥ baseline_y + strut_descent`（strut 盒随基线悬挂）；净抬升量记入 `r5298_lifts`，循环后按累计增量下移后续行顶。legacy 高度变化（under_need / R822 extend）从不传播行顶——corpus 按此校准，平移混入即整体错位。
4. **全 lh:0 run 行不接管**：行内全部 text run `height == 0`（line-height: 0 内容）时 legacy 兜底（strut_lh=0 → 行高）语义保持。

## 试错三轮（corpus 仲裁）

| 版本 | 结果 | 定谳 |
|---|---|---|
| v1 无条件接管 + break_lines 快照全量平移 | 15728P/863F：6 新红 / 2 翻绿 | legacy-wins 行被贬基 + legacy 抬升被平移 |
| v2 max-guard（max(container, legacy)） | vertical-align-121 复绿，4 案残红同值 | 残红扰动源 = 快照平移非 strut 本体 |
| v3 胜出接管 + 归因平移 + 捕获恒覆写 + lh:0 门 | **15733P/858F，0 新红** | 归因成立 |

ruby-align-001 三变体同为 ✗1.02% 的探针定谳（`ZW_R5298_DEBUG` 临时行级探针，已移除）：h=20 内嵌行（runs fs16/**h0**）被容器 48 接管抬高——anon 容器样式继承自 body 而非 ruby（style-system 域结构缺口，见挂账），`line-height:0` 门修复本侧面。

## 门禁

- make reftest **716/716**；layout-engine lib **2173P**(+2，含 r5298 两判例)；fmt 0 diff；clippy（zero-layout-engine --all-targets）0 告警；product-smoke 6/6 struct PASS。
- bench-gate：对端流 `zeroweb-regression-guard` bench-chain 并行窗（loadavg 5→9.75），四窗口多跑 FAIL 集 1→8→5→2 非定（噪声签名），同二进制 wide_tree 187µs→220µs/234µs（3 分钟内纯负载 delta，该路径本轮零改动）、stash 干净 HEAD 同负载自超预算（deep_nesting 306µs > 289.9、block_layout +18%）——R5212 判例同构「机器比记录基线慢 = 记录基线陈旧环境漂移」。同负载 criterion A/B：WIP vs HEAD block +3.6%/flex +3.8%（ms 级噪声带内）；页面级 layout_ms 预算 PASS。**按 R5210/R5212 环境噪声判例交付，零 relax 零基线改动；R5299 步骤①空闲窗口 bench-gate 复测补门。**

## 常驻测试

`crates/layout-engine/src/inline/tests/r5298_mixed_lh_line_pitch_tests.rs`：
- `r5298_strut_holds_line_box_on_small_lh_only_lines`——c42 原样页 5 行逐行 h=12/基线 9/行顶 12k，div 高 60。
- `r5298_normal_line_height_paragraph_unchanged`——Ahem 16px normal 段落 3 行 h=16、p 高 48（容器 strut 捕获 normal 整数化同口径，零扰动锚）。

## 挂账（新矿井）

1. **「容器 strut < legacy 但应接管」域**：first-line-line-height-003（1.89% 既有红）在 v1 无条件接管下曾翻绿——::first-line 伪元素行需要 strut 全量语义，与 va-shifted 校准面冲突；须 va-shift 感知的 strut 归因（判别特征待独立轮）。
2. **ruby anon 容器样式继承结构缺口**：anon 盒 computed line-height 继承自 body（48）而非 ruby（0）——style-system 域，属对端流工作面（碰头信号，记 master.md）。
3. 既有挂账维持：单片段/orphan inline 装饰 legacy 轨转正、selection fill 逐词 run 域、c43/c44 replaced-element 入行盒域。
