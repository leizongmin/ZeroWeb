# R5315 — inline-block OOF 行内流静态位锚 + 锚消费流基修正（css-position-3 §static-position × CSS2 §10.3.7：vertical-align-baseline-004a/005a ✗5.70% ×2 → ✓0.80%；abspos-inline-008 同轮入账并当场收口；corpus 829F→827F FIXED=2 零新增红）

**日期**: 2026-10-28
**驱动案**: `css/CSS2/linebox/vertical-align-baseline-004a.xht` / `-005a.xht`（829F 账本 vertical-align 族 7F 择轻：像素走查锁定 2 案同源）
**corpus**: 同树 A/B——基线 **829F**（R5314 交付态 `80ee2c69e` + rebind `0ca7f8419`）vs 修复态 **827F**，逐案 diff = FIXED 2 案（`vertical-align-baseline-004a`/`-005a`，✓0.80%）；**零新增红**。

## 定性（诊断链）

1. **现象**：004a 蓝盒（空 inline-block，height/width:2em）test 侧可见 bbox x63-107（45 宽）vs ref x8-107（100 宽），**垂直 extent 完全一致**（y72-171——空 ib 基线 = bottom margin edge 规则 ZW 已正确）；005a 同值同形。族内另 5 案（117a/118a/122/baseline-003/negative-leading-001）全源 grep 无 abspos——不同子域（text-bottom/text-top half-leading 盒、行盒高），非本轮域。
2. **像素定谳**：ruler-only 带（y275-320）dark 列组 test=x58-62 vs ref=x158-162；100px/200px 刻度线带同分化——**abspos ruler img test 渲在 x8-63（行首）、ref 在 x108-163（ib 之后）**，img 左半压住蓝盒。差异全为水平一维。
3. **机理**：R5166 行内流 OOF 静态位锚（`collect_items.rs` OofAnchor 发射 → `break_lines.rs` 消费期记录行内光标 → `abspos.rs fix_abspos_inline_static_position` 覆盖 x/y）的**发射门仅 `display:inline`**；UA replaced 组（img/canvas/input…）在 ZW 判 `InlineBlock`（`style-system/lib.rs:89`），abspos img 无锚 → taffy 块流静态位（容器内容左缘）→ 盖住前驱 atomic inline-block。spec：css-position-3 §static-position——inline-level OOF（含 inline-block）静态位 = 行内光标位；chromium 对 abspos img 即此行为。

## 实现（两处，一处收口同轮入账案）

1. **发射门扩 InlineBlock**（`collect_items.rs`）：`is_inline_level_oof` 加 `DisplayValue::InlineBlock`。ZW 无 abspos display 块化（仅 -webkit-box legacy 块化），abspos img 计算值保持 InlineBlock，门可达。kill-switch 沿用 `ZW_OOF_INLINE_ANCHOR`。
2. **锚消费流基修正**（`abspos.rs` walk child_frame 累计）：锚 x/y 是断行期**未施加 relative 偏移**的行内流坐标，而累计路径上盒 x/y 已含 relative inset（display:inline 由 taffy 单次施加、inline-block/replaced 由 R5054 臂单次施加）——中间 relative inline wrapper（abspos-inline-008 的 `span left:-100px`）按位移后 x 累计使 `ax − dx` 把 −100 反号 +100 二次施加。修正 = 累计扣回 `resolve_relative_inset`（仅 Px；非 Px inset 两臂施加口径本就分裂，维持旧行为）。
3. **同轮入账案 abspos-inline-008**（首轮 corpus +1 红 `positioning/abspos-inline-008.xht`，绿→红 ✗4.17%）：绿盒 test x108-207 vs ref x8-107（恰 +100）——即修正 2 的活体反证；落地后 ✓0.00%。该案 R5225 注释已留名（span 位移 2×−100px 旧症），本轮为同坐标基问题的锚侧再现。

## 回归钉（3 件，mutant 红在案）

`crates/layout-engine/src/engine/tests/r5315_inline_block_oof_static_pos_tests.rs`：

1. `r5315_inline_block_oof_anchor_follows_atomic_predecessor`（主钉）：author inline-block abspos 盒锚在前驱 atomic 推进位 50px（修复前无锚落 0）；
2. `r5315_abspos_img_after_inline_block_gets_inline_anchor`：UA 默认分类路径（无 author display）img 锚 x=ib 宽 60——004a 形态直钉；
3. `r5315_anchor_accum_skips_relative_wrapper_inset`：relative inline wrapper 内 inline-block OOF 三段合成位（host +100 + wrap −100 + oof 0）= 流位 0——修正 2 直钉（修复前 oof.x=+100 合成 100）。

**mutant 因果**：`ZW_OOF_INLINE_ANCHOR=0`（= 旧门 InlineBlock 无锚语义）双钉红于 x=0 断言（taffy 块游标位），修复态全绿——双向可判别。

## 门禁

- fmt 0 diff；clippy `zero-layout-engine --all-targets -D warnings` 0 告警；**layout-engine lib 2201P/0F**（2200 + 新增 3 钉）。
- corpus 同树 A/B（见题头）829F→**827F** 零新增红；驱动族 FILTER vertical-align 7F→**5F**（004a/005a ✓0.80%，余 5 案无 abspos 不同域维持）；abspos-inline-008 复验 ✓0.00%。
- 本地 720 套件 722/723 唯一红 `float-shrink-inline-children-zw-001`（R5311 既有记账，对端 d68f3aa16 域）零位移。
- product-smoke：struct ×6 全 PASS；welcome oracle **15.68%（75254/480000px）与 R5308-R5314 零位移同值**；form-input-perf-gate PASS。
- bench-gate 定向 `ZERO_WEB_BENCH_CRATES=zero-layout-engine`：**GATE PASS**，25 指标全在预算内 NEW=0（锚发射为 OOF 元素冷路径、锚 pass 仅 anchors 非空时走全局 walk；medium/layout 149.17/预算 181.9ms，跑时 loadavg ~10 有背景负载噪声）。

## 挂账

- vertical-align 族余 5 案（117a/118a text-bottom/text-top half-leading 盒、122、baseline-003、negative-leading-001 ✗19.98%）——与 R5258 first-line 切片 2 同域（bg/font/vertical-align 需 IFC token 切片，结构性）；negative-leading-001 差异量最大（19.98%），账本独立定性。
- `resolve_relative_inset` 仅 Px：非 Px relative inline inset 的锚消费基差维持旧行为（两臂施加口径本就分裂，corpus 无此形态翻红）。
- R5314 挂账全维持（bookmark 既有红移交对端、SYSTEM-only doctype、multicol/Path B 让位门、float walk y 双补偿、Path B border-box 带、outline-028 lime 域 0.31%、form 族/负载脆弱双族/balance 9F/welcome 对端流/corner-shape 族）。

## 下一步 R5316

①827F 账本续扫择轻（避开本轮 OOF 锚域与 R5314 XHTML 自闭合域）②negative-leading-001（✗19.98%）独立定性 ③outline-028 lime 域 / balance 供给评估维持。

## 域收口核验

修复后账本全量复扫：无新增 inline-block OOF 静态位形态余案（首轮 corpus +1 的 abspos-inline-008 已当场收口 ✓0.00%）；`ZW_OOF_INLINE_ANCHOR` 域内 Inline/InlineBlock 两级均已锚定，**行内流 OOF 静态位域收口**。余 vertical-align 族 5 案属 IFC 切片结构域（非本锚域）。
