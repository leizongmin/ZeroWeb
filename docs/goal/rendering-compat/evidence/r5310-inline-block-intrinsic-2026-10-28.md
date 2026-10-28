# R5310 — 非 replaced 原子 inline 塌 0 域 max-content DOM 递归真测量 + definite max/min-width 钳制（css-sizing-3 §5.1/§5.2）——corpus 15752P/840F 与 R5309 稳定账本逐案一致零翻转；outline-028 ref 页测量缺口修复（TEST 页第二层 bug 独立挂账）；bench 同窗 A/B 捕获 +10.4% 回归并以塌 0 门消除

**日期**: 2026-10-28
**驱动案**: `css/css-ui/outline-028.html`（R5309 期 outline-028 取证链延续）
**corpus**: 修复后两次全量 **16592 / 15752P / 840F**，失败集逐案与 R5309 稳定账本（/tmp 对照 `r5309-failures-run2.txt` 840F）**完全一致**——净 0，零意外翻转。

## 定性

outline-028（✗0.70%）取证发现**两层缺陷**，本轮只修第一层（固有测量）：

1. **固有测量缺口（本轮修复）**：intrinsic 测量发生在 taffy 首趟前（R4355 同序），内容定尺寸 inline-block 的 `child.width` 彼时为 0——`width:max-content` 容器（`block_intrinsic_width_ex` 实际解析路径，R1018 分发）对 inline-block 子只算到 margin（ref 页 `<div class=inline style=margin-left:5px>XX</div>` 唯一子测 5 应 25）。R4032 在 `box_content_max_width_inner` 已有同款臂（`ZW_IB_DESTRETCH`，opt-in 未验证），但实际解析路径所在函数此前无接线。
2. **IFC float+inline 行布局（未修，挂账）**：TEST 页 outline 容器实测 20 应 25（float margin-left:5 缺失）——测量函数已算出 25，损失发生在 IFC float 回避行布局层（dual-page layout dump + PNG RGBA 逐带 diff 取证），与第 1 层正交，独立成轮。

## 实现（intrinsic_sizing.rs 一处新臂，kill-switch `ZW_IB_INTRINSIC_RECURSE=0`）

`block_intrinsic_width_ex` 在 R4355 纯 inline 臂之后新增：非 replaced 原子 inline（inline-block/inline-flex/inline-grid/inline-table，svg 已被 R4000 臂先行消费）**且 `child.width <= 0.5`（塌 0 门）** 且 max 模式时，改 `box_content_max_width` DOM 递归真测量，贡献再被自身 definite max/min-width 钳制（border-box 基 `len + frame`，与 `clamp_fit_content_child_contribution` 的 `limit` 同式）。

- **塌 0 门是本轮第三版**。第一版（无条件递归）全量 corpus A/B **+3 红**（absolute-non-replaced-width-017/018/021，各 3.00%）：abspos shrink-to-fit（CSS2 §10.3.7）的 R5236 D2 重测臂**只扩不缩**，taffy 首趟已按 span max-width 解出 120，被未钳制递归值 240「扩」掉；第二版加 definite max/min-width 钳制（css-sizing-3 §5.2；017 案 assert 明言「max-width 约束长度定义父 preferred 宽」）回收 3 案，但 bench 同窗 clean 树 A/B 捕获 **medium 页 layout +10.4%**（174.67→192.73ms，definite/已解析宽子被无谓递归文本测量）→ 第三版收窄到塌 0 域：post-taffy 重测（D2、caption、float 链）child.width 已解析走旧口径（017 的 D2 恰需 120 非递归值，两全），pre-taffy 内容定尺寸子仍递归（outline-028 ref 页缺口保留修复）。
- 钳制只对 definite 长度生效（`resolve_intrinsic_real_length` 对 Auto/Percentage/`min-content`/`max-content`/`fit-content` 全返 None）——关键字域已由递归内 `kw_aware_max_width`（R4151）消费，无双算。
- 规范链接：https://drafts.csswg.org/css-sizing-3/#intrinsic-sizes 、https://www.w3.org/TR/CSS21/visudet.html#shrink-to-fit-float 。

## 交付

- 全量 corpus 两次独立复跑 **16592/15752P/840F**，失败集逐案与 R5309 稳定账本一致，零意外翻转零新增红（第一版 +3 红在收窄后全部回收）。净 0 轮：价值在测量语义正确化 + outline-028 ref 页半修复 + 三枚回归钉。
- 定向复跑 absolute-non-replaced-width-01x 全族 **24/24 ✓**（017/018/021 收窄后保持绿）；outline-028 维持 ✗0.70%（第二层 bug 域，与测量层正交——ref 页容器测量已正确，TEST 页 IFC 行布局缺陷独立挂账）。
- 回归测试 +3（`crates/engine/tests/r5310_inline_block_intrinsic_max_width.rs`，引擎级 RenderPipeline API）：①`width:max-content` 容器 + 内容定尺寸 inline-block（旧口径容器塌 5 → 递归后 23 ≈ 文本 17.6 + margin 5）②同容器 + `max-width:120px` + 文本（递归裸值 ~240 → 钳制 120；断言取容器 div 行——span 自身布局期 max-width 也是 120，全文 contains 匹配曾假阳性，mutant 复验改结构选择器后红靶达成）③abspos shrink-to-fit + max-width 子（R5236 D2 只扩不缩不放大，post-taffy 旧口径域钉）。kill-switch 下 ①② 红（递归域钉）、③ 绿；去钳 mutant 下 ② 红（240 未钳）。
- **跨流发现**：`fetch-wpt-data`（make reftest-upstream 依赖）在 reftest-manifest.json 缺失时 `rm -rf` 整个 wpt-data 目录再克隆——**把对端 encoding 流手工拉取的 legacy-mb 语料一并清掉**（`wpt_file_loader::legacy_mb_chars_decode_semantics_euc_jp_and_iso_2022_jp` 红：「两案语料均缺失」）。经 `tests/wpt-runner/scripts/goals/31-encoding-legacy-mb.sh` 幂等恢复。移交对端流知悉/碰头：语料易被常规 reftest 依赖链无意摧毁。

## 门禁

- layout-engine 全套 **2206P/0F**（lib 2189 + 集成子套件）；r5310 引擎回归 ×3 绿 + kill-switch ×2 红 + mutant ×1 红证据在案。
- fmt 0 diff；clippy zero-layout-engine + zero-engine `--all-targets -D warnings` 0 告警。
- product-smoke：struct ×6 全 PASS + form-perf PASS；welcome oracle **15.68%**（75254px 与 R5308/R5309 同值零位移）。
- **make test 全量 ×3**：首跑缺 `target/debug/zero-browser`（前次被内存守卫中断的编译遗留，`cargo build -p zero-browser -p zero-renderer` 补建后过 image_batch 腿）；次跑缺 legacy-mb 语料（跨流发现，恢复后过）；末两跑 793P/**1F**——`image_batch_e2e::renderer_publish_batch_image_fetch_reaches_concurrent_in_flight` 两次**不同断言**翻红（:317 fetched-exactly-once / :324 peak 7 > limit 6），隔离复跑 **3/3 绿**——多进程并发限流 e2e 的负载脆弱签名，属对端 renderer/net 域，记账不阻断。
- **bench-gate 定向 zero-layout-engine**：pre-gate WIP GATE FAIL 6 指标 → **同窗 clean 树 stash A/B 定谳 medium/layout_ms 真回归**（clean 174.67 PASS vs WIP 192.73 FAIL，+10.4%；block/flex 微基准双树同超 = 对端负载签名）→ 塌 0 门后同法复测 **157.77ms**（低于 clean 同窗值，回归消除；block 4.28/flex 3.85ms 同窗落预算内）。正式 perf-gate 判 **INCONCLUSIVE**（测量期 loadavg 13，报告标 suspect）——空闲窗并账复跑（R5307 残余 + R5308 + R5309 + 本轮），零 relax 零基线改动。
- 环境注：外层 test-guard 包裹 `make test` 会与其 compile-first 设计冲突（Makefile 明言「cargo test 先无约束编译再监管运行阶段；clippy 不受内存阈值限制」，双腿并行编译 >25GB 触外层总内存上限）——`make test` 本身即守卫入口（每腿内层自守卫），直接调用。

## 挂账

- **outline-028 第二层：IFC float+inline 行布局**（TEST 页容器 20 应 25，float margin 丢失在行布局层）——候选下轮结构案。
- bench-gate 空闲窗并账复跑（R5307 flex 残余 + R5308 + R5309 + R5310 四项并账）。
- image_batch_e2e 负载脆弱（对端 renderer/net 域）：两次全量不同断言红、隔离 3× 绿——移交对端流。
- fetch-wpt-data `rm -rf` 摧毁对端手工语料（跨流 footgun）：移交碰头，建议语料目录隔离或 fetch 前存在性探测。
- R5309 挂账全维持：text-wrap:balance 家族 9F（balance 算法 + 004 强制断行分组，中复杂度）；R5307/R5308 挂账（flat 兄弟 float 叠压结构轮、earliest-fit 带集 FIXME、float_positioning.rs 超限）；welcome 15.68% 移交对端流。
- flake 账本无新增（content-none-select-1 / size-adjust.tentative / multicol-fill-auto-block-children-002 维持既有双向摆动记账）。

## 下一步 R5311

①840F 账本续扫择轻 ②outline-028 第二层（IFC float+inline 行布局）评估 ③bench-gate 空闲窗并账复跑 ④text-wrap:balance 供给评估（若账本轻案不足）。
