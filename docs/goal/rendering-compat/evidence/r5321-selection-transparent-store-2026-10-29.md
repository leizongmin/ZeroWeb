# R5321 — 作者 ::selection transparent 背景声明存储探针（css-pseudo-4 §highlight-cascading）

- **日期**: 2026-10-29
- **驱动案**: `css/css-pseudo/selection-background-color-transparent.html` ✗0.00%（mismatch 型）→ **✓3.18%**
- **corpus**: 817F → **816F**（16592 案，FIXED=1，REGRESSED=0）
- **域**: css-pseudo-4 §highlight-cascading（作者 origin 压过 UA 默认高亮回落）× R5052/R5065/R5272 selection 存储链

## 定性（817F 账本 0.00% 段复查，R5320 下一步 ②css-pseudo 域邻案）

`selection-background-color-transparent` 是 **mismatch 型 reftest**（`rel=mismatch`
notref）：页声明 `div::selection { background-color: transparent }` + 脚本选区，断言
test（无高亮底）与 notref（UA 默认 Highlight 蓝）**必须相异**。ZW 实渲染 0 像素差 =
两页都铺 UA 默认高亮 = 作者透明声明被吞。根因：`compute_styles_recursive` 的
selection 存储门是纯后值判据 `color 差异 || bg 非 Transparent`——「作者显式
transparent/同值声明」与「无规则」后值同形（Transparent/同 color），误不存储 →
`selection_pseudo = None` → paint 侧 R5272 UA 默认高亮回落（`selection_pseudo.is_some()
|| ZW_SELECTION_HIGHLIGHT_DEFAULT` 门）给作者已声明页再铺 OS 高亮底，违反
§highlight-cascading 的 origin 序（作者声明面压过 UA fallback 面）。

同域邻案 `highlight-paired-cascade-004`（同 0.00% mismatch 段）为 ::target-text
paired cascade 域（UA Mark 默认底 + 配对级联），ZW 无 target-text UA 默认底机制，
**不适用本修复另案**（记录不挂账）。

## 实现（style-system 单侧，paint 零改动；无 kill-switch——R5320 切片先例）

存储门加 collect 探针腿：后值歧义形（color 同 + bg Transparent）才走
`collect_pseudo_declarations_with_media(..., "selection")`（≡ R4257 scroll-button 位
同构：媒体+容器链顶；仅 `has_selection_rules` 页的 selection host 到达），命中
**background 面字面 `transparent` 值声明**（`background-color`/`background`）即存储。
三道收窄（每道都有 corpus 实测红背书）：

1. **只在后值歧义形走探针**——非歧义形（color 差异/bg 非透明/R5065 种子后非透明）
   零新增开销；
2. **只认 background 面属性名**——探针返回未过滤原始声明（未知属性 `foo: bar`
   R5065 ③ pin 族在列，级联期才丢弃），全量判存复活 invalid 规则族；
3. **只认字面 `transparent` 值**——`background-color: bar`（invalid value，
   active-selection-054）与 `background-color: revert`（回落 UA origin，
   selection-background-color-revert）按名判存实测 +2 红，按值字面收窄后双绿。

注：`rgb(0,0,0,0)` 等数值透明写法不在字面匹配内（corpus 无此类案，需要时按值域
扩展）；invalid-value/revert 语义级判别需级联期 declaration 溯源，超出本轮切片。

## 回归测试

`crates/engine/src/paint/tests/r5321_selection_transparent_bg_tests.rs` +3 pin：
①作者 transparent 声明 → 零 Highlight fill（目标腿）；②同页无规则 → Highlight
fill 照铺（R5272 控制腿）；③作者非透明背景 → 照铺作者色（既有存储腿）。**mutant**
（stash lib.rs 回存储门实测）：仅① FAIL，②③ PASS——双向可判别。

## 门禁

- fmt 0；clippy zero-engine + zero-style-system `-D warnings` 0。
- **zero-style-system lib 2450P/0F**（R5065 存储判据 pin 保持）；**zero-engine lib
  3588P/8F**（8F = R5320 已记账 js_dom_bridge fetch 既有红，clean HEAD 同红非本轮）；
  zero-layout-engine lib 2225P/0F（涟漪保险腿）。
- engine selection 族 44P/0F（r5256/r5269/r5270/r5271/r5272/r5273 + 新增 3 pin 全绿）。
- 本地 723 套件唯一红 = R5311 既有记账；product-smoke 全腿 PASS + legacy 42 fixtures
  struct 0 失败（battery 以本轮最终树跑）。
- corpus 全量：**816F**，FIXED=1（driving ✓3.18%）REGRESSED=0；中间版（按属性名判存）
  实测 active-selection-054 + selection-background-color-revert +2 红，收窄后复绿
  （收窄必要性 corpus 背书）。
- **bench-gate 判定不适用**：探针仅在「::selection 规则页 × selection host × 后值
  歧义形」三重门后触发（产品页几乎恒零路径）；不在 bench 测量的布局/解析热循环。

## 挂账

- **收口**：selection-background-color-transparent mismatch 族（817F 账本 0.00% 段
  本域首案）。
- **维持**：bench-gate 静窗复测（跨 R5317/R5318 两轮）、非 img inline-block
  content/border 口径分裂、content-none-select-1 负载脆弱族、c44-003 0.47% 残差、
  va 族余 122/baseline-003/intrinsic-size-009/R5314/R5315；zero-engine js_dom_bridge
  fetch 8 案（R5320 记账，net-api 流域）。
- **邻案观察（不挂账）**：`highlight-paired-cascade-004`（::target-text paired
  cascade，ZW 无 target-text UA 默认底机制，深结构另案）；`::selection` 数值透明写法
  （`rgb(0,0,0,0)`）不在字面探针内，corpus 无案，未来入账按值域扩展。
- **下一步 R5322**：①816F 账本续扫择轻（0.00% 段 + 低 diff 段，避开 va 域与 OOF
  锚域）②CSS2/css1 邻案（c548-ln-ht-000、c5525-fltmrgn-000、floats-031）③css-multicol
  103F / css-transforms 67F / css-flexbox 52F 域内挑轻。
