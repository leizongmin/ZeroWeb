# R5295 — 伪元素 inline-block+文本盒化 + inline 水平百分比 padding 解析 + 跨行 padded 行内盒 x 写回——c547-indent-001 ✗2.23%→✗1.17%

日期：2026-10-27。改动：`crates/engine/src/pipeline/mod.rs`（+40/−1，needs_box 新臂 + 测试）、`crates/layout-engine/src/inline/collect_items.rs`（+38/−3，pct resolver + 测试）、`crates/layout-engine/src/inline_finalization.rs`（+19/−8，min_box_x 两口径）、`crates/layout-engine/src/engine/tests/r5295_wrapped_padding_box_x_tests.rs`（新 +71）。新 kill-switch `ZW_PSEUDO_INLINEBLOCK_TEXT_BOX=0`。

## 案面

`css/CSS2/css1/c547-indent-001`（8 段落 text-indent 矩阵）：第 8 段 `p.eight:before { color: red; content: "generated content before"; display: inline-block; width: 50%; }`（无 background）——ref 期望伪元素为 50% 宽（392px）原子 inline-box 参与首行缩进，ZW 中伪文本 shrink-to-fit（~162px）冒充盒宽，首行缩进全错。

## 取证链

1. 15-probe 二分（css1/ 目录自建 probe 页 + REFTEST_DUMP_PASS 像素取证）：同规则**含 `background: yellow` 时伪元素正确盒化**（style-system 侧伪样式两变体均正确产出 content/display:inline-block/width:50%，scratch 测试定谳）→ 差异在 engine 页面管线的**盒化门控**。
2. 无 background 变体像素：伪文本以 inline 文本绘制（红色 x[9,164]，shrink-to-fit 宽）——`needs_box` 全臂落空走 text-node 注入路径，`display`/`width` 结构性丢失（text-node 无盒模型）。
3. 宽度侧 probe B（196px div + span `padding-left:50%` 跨 4 行）：文本 advance 全零——`resolve_inline_padding` 对 `LengthValue::Percentage` 归 0.0，与盒宽侧另一条百分比感知路径不一致（盒 w=211.6 vs 文本不动）。

## 根因（三处，两修一顺带）

1. **needs_box 门控缺口**（engine/pipeline/mod.rs）：R5238 臂族要求「display≠Inline **或装饰**」才盒化，`display:inline-block + 真文本 + 无装饰` 落空。CSS2 §9.2.1：atomic inline 有自身盒模型，text-node 路径容不下 display/width——R3928 块级无装饰保守门（盒化收益域未证）不覆盖 inline-block。
2. **inline 水平 padding 百分比归零**（collect_items.rs）：CSS2 §8.4 inline padding 百分比按包含块内联轴宽解析，IFC `container_width` 在 collect 期可用而未用。
3. **跨行 padded 行内盒 x 写回负偏**（inline_finalization.rs，px padding 同构既有缺口）：盒 x = min(全部 fragment x) − first_pl 统一扣——续行 fragment x 不含 pl 推进（padding 只在 run 首缘落账一次），跨行时 min 命中续行缘再扣首行 pl → 负偏（196px 容器 50% padding 跨 4 行 → x=−98；px 形态 −40）。

## 实现

1. **needs_box 新臂**：`display==InlineBlock && !text_blank && !parent_is_ruby_family && ZW_PSEUDO_INLINEBLOCK_TEXT_BOX!=0` → element 盒化。ruby 族宿主排除同 R4581（inlinize 语义未实现）；块级无装饰臂维持 R3928 保守门不动。
2. **`resolve_inline_padding_pct`**（collect_items.rs）：`Percentage(p) → container_width * p / 100`，其余走原 resolver；仅替换 4 处水平调用点（child_id/elem_id 两组 pl/pr）。容器宽未知（内在尺寸趟 container_width=0）→ 0 ≡ css-sizing-3 §5.2 不定基准。**不动**：mod.rs:1336 垂直 padding-bottom clamp 域、collect_items.rs:2054 原子 frame 累加、全部 margin 解析（R5068 负 margin 校准域留专项）。
3. **FragAgg `min_box_x` 两口径分离**（inline_finalization.rs）：内容缘 `min_x` 逐字节保留原语义（宽度 union `union_width = (max_x − min_x) + frames` 与 R5232 祖先链消费——首建处曾误改其语义致单行 decorated 盒宽 48.8→88.8 双计 padding，测试当场抓获后回退收窄）；新增 `min_box_x`（首片段 x − 自身 pl、续行/空片段原值收敛，仅 x 写回消费），写回 `child.x = min_box_x − fold_landed`，废除统一扣 first_pl。

## 回归测试

- engine 桥测 +1（`inject_inlineblock_text_pseudo_as_element_node_r5295`）：phase① inline-block+真文本+无装饰 → ELEMENT zw-pseudo（style 保 display:inline-block + width:50%）+ 文本子节点；phase② kill-switch=0 → text-node。单测试函数双相位（env-var 竞态纪律）。
- resolver 单测 +3（collect_items.rs `r5295_inline_pct_padding_tests`）：50%×196=98 ✓、legacy Percentage=0 路径回归 ✓、px/em 不变 ✓。
- 盒 x 锚 +2（r5295_wrapped_padding_box_x_tests）：换行 px padding 盒 x=0（修前 −40）+ 单行场景 x=0 且盒宽含 padding 不双计（48.8，断言区间字体无关 40<w<70）。测试样式显式 `CssParser::parse_stylesheet` 传入（`<style>` 不被测试管线自动抽取——vacuous 假绿教训）。

## 交付

- 全量 corpus **16590/15731P/859F**，失败集与 R5294 交付态 **859=859 逐案零翻转**（diff 空）；全 corpus 唯一百分比变化即驱动案 c547 **✗2.23%→✗1.17%**（残余为 atomic inline-block 基线模型 2px 垂直位移——REF 原子盒基线对齐 vs strut 行高，独立 fidelity 域挂账）。
- kill-switch 判别：`ZW_PSEUDO_INLINEBLOCK_TEXT_BOX=0` 定向单案 → ✗2.22%（回修复前），OFF 轮归因成立。

## 门禁

- make reftest 716/716（0 failed）；engine lib **3534P**(+1)、layout-engine lib **2168P**(+5：盒锚 2 + resolver 3)、style-system lib 2450P；fmt 0 diff；clippy（zero-engine + zero-layout-engine --all-targets）0 告警。
- product-smoke 6/6 struct PASS + welcome oracle 15.50% 持平；bench-gate 定向 `ZERO_WEB_BENCH_CRATES=zero-engine,zero-layout-engine` GATE PASS（25 指标，NEW=7）。
- **make test 跨流散布红记录**（5 跑 4 观察案，全部非本流域且 standalone 复验绿）：
  - image_batch_e2e（净域，对端流 10-25 新增）：本树并行负载下 2 红（37≠36 重复 GET）→ 安静窗 1 绿 + 单测隔离 5/5 + 全量 lib 独立 3/3 绿。页面为裸 img 无伪元素/无 padded span，本修两改动结构惰性。
  - cdp busy-arm（CDP 域，R5293 同族）：1 红 → standalone 5/5 绿。
  - page_scripts mutation_arm（renderer 域）：本树 1 红，**干净基线 HEAD（stash 后）同案同断言逐字节复现**（page_scripts.rs:1787「未观测到滞留窗，时序假设失效」；该腿负载下 316-319s vs standalone 1.6s）→ 预存时序敏感，非本修引入。
  - 归因：对端流 testharness 重腿同机并行窗（load 2.4-3.4）为共同背景；本流触达各腿（engine/layout-engine/style-system/workspace 大腿/integration 794 腿）全绿。

## 挂账

- **c547 残余 1.17%**：atomic inline-block 基线模型 2px 垂直位移（行 21-31 vs 23-33）——REF 原子盒 baseline 对齐模型 vs strut，独立域。
- userstyle 注入域（html-precedence-004）：runner 无 user stylesheet 注入面，ref 硬编码 user 样式后状态。
- inline 百分比 **margin** 解析：R5068 负 margin 校准与 word-space-transform-012 补偿形态耦合，留专项不并入。
- 垂直 padding（mod.rs:1336 clamp 域）与原子 frame 累加（collect_items.rs:2054）仍 legacy resolver。
- 未装饰 inline 盒宽不含 padding（slice13 gBCR 近似挂账，既有）。

## 教训

- **聚合字段单口径承载多消费方 = 回归温床**：min_x 同时被 x 写回（需盒缘）与宽度 union/祖先链（需内容缘）消费，改语义修 x 时宽度当场双计。正确做法是新字段新语义（min_box_x），旧字段逐字节不动——改动面收窄到单条写回式。
- 测试环境与被测管线字体度量不同：绝对宽度断言（49.8）在直接测试环境不成立（48.8），结构性区间断言（padding 下界 + 双计上界）跨环境成立。
