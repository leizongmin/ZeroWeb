# 刀 CH-METRIC-2 · style-system 进程级度量兜底（2026-11-21）

## 缘起与认领

CHMETRIC r1 电池（face 1cf2563bb、收账 fccb49c0c，累计 +12020）主靶
line-break-ch-unit **97F 持平未翻**：testharness 注入面（resolver+metric
map 接 pipeline 实例）只覆盖初始布局通道，JS DOM-mutation 径的真因
在收账笔定界为「mutation 后 restyle 的样式解析丢 metric map」并登记
**CH-METRIC-2（engine mutation 径 metrics 传递，97F 真因恢复靶）**。
本刀即该挂账的实现位，本线（CHMETRIC 原刀线）root-cause 后续做。

## 机制（root-cause 全链）

A/B 探针五轮（zz-chprobe v1-v5，b0/b1 runner 单页 FILTER 形——filter
为**位置参数**非 `--filter`，见笔误记录）二分出全貌：

| 臂 | 表现 |
|---|---|
| 静态声明（stylesheet / markup style 属性） | b1 盒宽 15px（3ch@10px，真实 advance 0.5em），"0 0000" 换行 oh=20 ✓ |
| JS `createElement`+`style.width='3ch'` / 载入后 `setAttribute` | 双臂恒 30px/oh=10（ch 塌 1em）——与收账笔三探针一致 |

**双 StyleSystem 世界**是形态学根因：

1. **pipeline 实例通道**（r1 已接线）：`webview.set_font_metric_map` →
   `pipeline.style_system`——初始级联+layout 有度量；
2. **独立 StyleSystem 通道**（r2 靶）：engine
   `js_dom_bridge/computed_style.rs` `compute_styles_for_doc` 每次
   `StyleSystem::new()` 独立构造（函数头注自证「静态/handle 视图/
   全量兜底三通道共汇点」）——gCS、s421 pending view 现算布局
   （`computed_style_cache::pending_view_handle_rect` →
   `computed_style_by_handle` → 1470/1524 调 `compute_document_styles_
   with_inline_overrides_css`）、全量兜底全部经此。真页 194 div 全为
   `createElement` 产物（handle 身份 `__n{n}`），测量落 pending window
   现算臂 → 独立通道 ch 无度量 → `resolve_length(None, None)` 1em
   兜底 → 盒宽 i×10px，i+1 字（真实 advance ~0.5em）恒容纳 → 不换行
   → 97F。

旁证：pending view 文本 advance 已走真实字体（p3 六字形恰合 30px 盒
→ advance 5px/char）——只有盒宽 ch 解析丢度量，与「文本/layout 同源
已通、样式解析径不通」的收账定界一致。

## face（3 文件，r2 增量）

- `crates/style-system/src/lib.rs`：`PROCESS_FONT_METRIC_MAP` 进程级
  `OnceLock<RwLock<HashMap>>` + `pub fn set_process_font_metric_map` +
  `StyleSystem::new()` 注入臂（走实例 `set_font_metric_map` 以继承
  `ZW_FIRST_AVAILABLE_FONT_METRICS` 灭活开关与 context_version 语义）；
  未接线进程（生产/单测）零行为变更；
- `tests/wpt-runner/src/reftest/reftest_fonts.rs`：
  `build_style_metric_map()`——同源 FontLoader 派生
  FontFamilyMetrics→FontRelativeMetrics 换算（与 pipeline
  `set_font_metric_map` 字段换算一致，f32→f64 五字段）；
- `tests/wpt-runner/src/testharness.rs`：接线块追加一行
  `set_process_font_metric_map(&build_style_metric_map())`——与
  webview 注入同 loader 派生（metrics 单一 owner，zeroweb-guidelines #3）。

r1 face（reftest.rs 可见性、钉测、marker）已随 1cf2563bb 落库，不再入
r2 face 清单；FM 门改双臂对称记录（marker 在 HEAD），判别主证 =
M2（md5 异）+ S1/S2/S3 ride census。

## 预期兑现核对（预注册不预设）

- 主靶 line-break-ch-unit：b0（HEAD+ride，含 r1 face）= 97P/97F 基线
  → b1 = 194P（wrap 97F→P；fit 97P 保持——i 字形恰合 i×ch 盒，等式
  边界依赖 advance 与盒宽同源同精度）；
- gCS 读回径**预期不动**（收账笔定界 computed_style.rs:2656 ch 臂
  硬编码 1em 兜底，独立于级联度量——本刀不触序列化层，gCS ch 仍
  30px 形）；
- blast radius 不预设：mutation-restyle 径上任何 ch/ex/ic 断言页
  （七语料）有位移可能，逐案归因后入账；text corpus advance 未动，
  SET-IDENTICAL 为佳。

## 电池结果（unit zw-knife-chmetric-r2-102102，发射 10:21:02，GATE-DONE 10:41:57）

脚本 `.acceptance/wptfyi-css/knife-chmetric-battery.sh`（r2 改参：FACE_FILES
去 reftest.rs（r1 已落库）；FM 门改双臂对称记录）。results 目录
`results-knife-chmetric/`（r1 原目录改存 `results-knife-chmetric-r1/`）。

门组全绿：

- FACE-MARKER b0=1 b1=1（marker 随 1cf2563bb 在 HEAD，双臂对称，非判别）；
- M2：b0 md5 `9ea01abbb13dc103f26ebe5692850263` ≠ b1 md5
  `4978d6667c94b8853a2e547259c80761` ✓；
- S1/S3 ride census 零扰动（9 个他流 WIP 文件三窗 md5 同；worktree 内
  `zz-chmetric-probe*` 未跟踪探针页不属 ride 清单）；
- TRAP-NOOP rc=0；14 腿（7 语料 × 双臂）全完成。

逐语料 case 键级（P 数变化）：

| 语料 | gained | lost | 明细 |
|---|---|---|---|
| values | **+100** | 0 | **line-break-ch-unit 97→194 (+97)**；cap/rcap/rex-invalidation 0→1 (+3) |
| text | 0 | 0 | SET-IDENTICAL（advance 未动，符合预期） |
| fonts / sizing / motion / typed-om / color | 0 | 0 | SET-IDENTICAL（四哨兵全净） |

主靶兑现：b0 97P/97F → b1 **194P/0F**——wrap 97 全翻 P、fit 97 全保持
（i 字形恰合 i×ch 盒的等式边界无浮点翻车）。cap/rcap/rex-invalidation
+3 同机制（mutation-restyle 径 font-relative 单位真实解析）。

**ASYM −25 定界（非回归）**：RECONCILE 报 10 个 `zz-chmetric-probe*`
页 b0=N/b1=0——收账笔对流的调查探针（未跟踪 worktree 文件）在双腿
之间（values b0 腿后、b1 腿前，10:32 窗）被对流删除：b0 JSON 含 10 个
探针页键、b1 JSON 0 个键，文件现已不在 wpt-data。真实上游页双腿同
枚举、账面 +100/−0 不受影响；探针页不计入净账（非上游语料）。

钉测：`base_loader_publishes_monospace_ch_metric` 1 passed（发射前
10:19，r2 face 编译门后复跑）；`cargo check -p zero-wpt-runner`
（test-guard 4.03s）+ rustfmt 三 face 文件 CLEAN + clippy 本刀文件
零告警（style-system 6 missing_docs 告警系他流 730dbfc2a 挂账域）。

## 域界与挂账

- 本刀只接级联度量解析（style-system 消费侧）；gCS 序列化 ch 硬编码
  臂（computed_style.rs:2656）不触——另一刀（收账笔已登记 gCS ch 桥
  层读回）；GL/U+202F 禁则断行器（baspglwj 121/124 +4 恢复靶）另行。
- style-system/src/lib.rs 5210 行超 2000 行准则（既有体量，本刀
  +55 行；拆分另案不混入本刀）。
- 挂账维持：CALC-SIZE-INTERP、INTRINSIC ~209F、calc-interpolation
  117F、if-cycle 38F（同 r1 收账笔）。
