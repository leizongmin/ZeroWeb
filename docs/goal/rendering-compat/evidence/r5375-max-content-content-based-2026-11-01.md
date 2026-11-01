# R5375 证据 — css-sizing §5.1/§5.2 content 关键字内容基测量（4 案收口）

日期：2026-11-01（R5375 轮；基线 = main@b26d5a5d6 + 本轮 fmt 前置 4c74562fd，干净树 **758F**）

## 0. 结论速览

| 案 | 修复前 | 修复后 | 定谳 |
|---|---|---|---|
| `css/css-sizing/block-size-with-min-or-max-content-7.html` | 2.08% | **PASS 0.00%** | 翻绿（本轮归因案①） |
| `css/css-sizing/fit-content-max-inline-size.tentative.html` | 2.08% | **PASS 0.00%** | 翻绿（本轮归因案②） |
| `css/CSS2/tables/table-anonymous-objects-151.xht` | ✗ | **PASS** | 连带翻绿（贡献臂） |
| `css/css-overflow/incremental-scroll.html` | ✗ | **PASS** | 连带翻绿（贡献臂） |

corpus 同树 A/B 三跑：RUN0（stash 基线）**758F** → RUN1 **756F** / RUN2 **757F**；
稳定交集 **755F**，FIXED = 4 案，**REGRESSED = 0**。RUN1/RUN2 超出稳定集的
`table-anonymous-objects-149/150/152/153` 为并行抖动面（R5374 已账 -127 同族），
隔离复跑 5/5 PASS。真通过-可信 10367（RUN2 静窗）。Total 16592（并行流 +2 案）。

## 1. 归因（css-sizing-3 §5.1 content-based size + §5.2 content-based contribution）

content 关键字（`min-content`/`max-content`/`fit-content(L)`）作 **cap** 使用时，测量基
应为**内容**：specified width 在 cap 使用侧钳制（used = min(W_max, specified)），不参与
W_max 自身。旧路径 `box_content_max_width`（R4149/R4151 口径）带 `include_own_width=true`
——`width:200px + max-width:max-content` 盒被 own_explicit 抬升测成 200，cap 恒 no-op：

- **block-size-with-min-or-max-content-7**：abspos 盒 `width:200px` + `max-width:max-content`
  含 100px 定宽子 → W_max 应 100，used = min(200,100) = 100；旧测 200 → cap 无操作。
- **fit-content-max-inline-size**：内盒 `width:200px` + `max-width:max-content`，其对父
  fit-content 的**贡献**同样应取内容基（cap 后 100 而非 specified 200）→ 走
  `kw_aware_max_width`（R4151 包装）own max 关键字臂，与 sizing.rs 的 cap 使用臂同根。

## 2. 修复（两臂同提交）

- `sizing.rs` `is_kw_clamp` 臂：纯 max 侧关键字（`kw_max && !kw_min`）cap 目标改
  `box_content_max_width_content_only`（R5037 既有 helper，content-only 只剥
  own_explicit，own_ar R3792 transferred 两口径都保留）；`kw_min` 收缩臂（含
  `max-width:min-content` 的 R4151 收缩路径）保持 full 口径。
- `intrinsic_sizing.rs` `kw_aware_max_width`：own `max-width:max-content` 的 intrinsic
  贡献 = `box_content_max_width_inner(..., false)`（内容基），不经本包装防递归重入；
  `.item`（border-box-and-max-content-002，无子 own_ar）transferred 保留不塌。

**敏感面注记**：`kw_aware_max_width` 贡献臂无独立 kill-switch（R4151 同口径，其
MinContent 臂亦无）；sizing.rs cap 使用臂仍由 `ZW_WIDTH_KEYWORD_CLAMP` 门控。
连带案 151/incremental-scroll 经贡献臂翻绿，`ZW_WIDTH_KEYWORD_CLAMP=0` 不影响
（A/B 同树已证因果，无需环境变量敏感臂）。

## 3. 门禁

- corpus 同树 A/B 三跑（R5373 教训：两侧都显式 `cargo build --release --bin
  zero-wpt-runner`）：758F → 稳定 755F，FIXED=4 REGRESSED=0；149/150/152/153 抖动
  隔离复跑全 PASS。
- layout-engine lib：**2310P/0F**（+2 `r5375_*`：abspos cap 翻案 + 贡献基单测）。
- make test workspace：**3664P/1F**——1F = `ua_display_li`（D4 既有账，与
  R5372/R5373/R5374 轮同位同签名）。
- clippy `--workspace --all-targets`（guarded-clippy）0 告警 + `cargo fmt --all
  -- --check` 清零（本轮先落 `style(browser)` fmt 前置提交清并行流遗留，见 §5）。
- product-smoke：struct-check PASS（welcome + article @320 双面 0 issues）。
- bench-gate：静窗 `ZERO_WEB_BENCH_CRATES=zero-layout-engine make bench-gate`
  **GATE PASS——25 指标全预算内 NEW=0**。改动面 = own content 关键字盒（rare 群）
  的测量基切换，常驻路径仅 is_kw_clamp 臂内一次布尔分支；前两跑 GATE FAIL（4/7
  指标、两次失败集不同成员）为并行流 loadavg 20+ 竞争（第 3 跑静窗 loadavg 5.9
  全绿复现），按 #35 跨窗漂移归因环境不归因代码。

## 4. 方法注记

- **busy 窗 bench-gate 不可信但也不误导**：两次 FAIL 的失败集互不相同（4 指标 vs 7
  指标、交集仅 2）——同一代码两次测量给出矛盾超预算集，本身就是环境噪声签名；
  静窗复跑全绿即收口，无需为单次 FAIL 做代码侧归因。
- **A/B 前 stash 记得显式重编 release**（R5373 教训再次生效）：RUN0 前手动
  `cargo build --release --bin zero-wpt-runner`，否则假对照。
- `table-anonymous-objects` 家族 8 并行 job 下抖动面比 R5374 记录的（-127 单案）
  更宽：本轮 149/150/152/153 四案先后在 RUN1/RUN2 出现又全部隔离复跑 PASS——
  全量跑出现该家族孤立 ✗ 时先隔离复跑再归因。

## 5. 并行碰头记

main 上并行流（renderer-trust-boundary / zero-web 域）b4f1b9079、b5ef2f0e8 等提交
遗留 4 文件 rustfmt 漂移，`make test` fmt 前置门对两流同时阻断；本轮以
`style(browser)` 独立提交（4c74562fd）机械清零（纯换行/导入序，pre-commit guard
PASS），未触碰语义。apps/browser 属对方域，后续轮次若再见 fmt 漂移宜由属主流自清。
