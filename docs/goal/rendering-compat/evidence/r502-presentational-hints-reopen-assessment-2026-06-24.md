# R502 presentational-hints 重开评估 — R504 paint-order 修复可能已解禁 block A（doc-side read-only）

**日期**: 2026-06-24
**性质**: read-only 历史复盘 + 机制比对（未跑 reftest——agent 正跑 R548 13-job normal-flow A/B，并发 OOM 风险）
**相关**: master.md 下一步 item 2（presentational hints REFUTED）；archive/rounds-r491-r526 R502 条目；MEMORY [[r502-presentational-hints-zero-yield-refuted-code-reverted]]

## 背景：R502 为何 REFUTED

R502（presentational hints → CSS2 App D `<img width=/height=>` → CSS）实现 **VERIFIED CORRECT**：
LAYOUT_DUMP 实证 absolute-replaced-width-006 `<img width="50%">` 计算尺寸 WITHOUT hint=15×15（intrinsic）
→ WITH hint=**96×96**（50%×#div1 192px ✓，spec-correct）。**但 reftest ZERO 像素效果 / ZERO yield**。

R502 IMGDBG 定位**双重阻塞**（任一不修则案不翻）：
- **block A「draw_order 覆写」**：abspos `<img>` 图元**确实被正确绘制**（primitive 存在 + image_cache HIT + fill loop
  实写 225px 到 (8..23,150..165)），但 `render_full_scene` 生产路径走 `render_draw_order`（R155），该 img 图元在
  draw_order 中**位置过早**，被其后某图元（疑 #div1 border/bg）**整片覆写** → 最终 fb blue_px=0。
  **本质 = abspos 元素 paint-order 错位**（CSS Appendix E：positioned descendants 应晚于 CB bg/border 绘制）。
- **block C「margin-collapse-through-border」**：006 test img abs_y=150 vs ref 70（80px 垂直偏差）=
  `div div{margin-top:1in}` 穿透 #div1 border 冒泡（§8.3.1 父 border-top 应阻断），**R490-entangled 死路**。

→ R502 按零 yield revert（R370 先例 + code-guidelines §2/§4），seam 保留待双阻塞解后重投。

## ★ 关键时间线发现：R502 在 R503/R504（paint-order 修复）之前 REFUTED

R502 REFUTED 后，R503/R504 **LANDED CSS Appendix E paint-order 修复**：
- **R504（commit ea08de51）global positioned-descendant deferral**：painter scope 收集**所有 positioned 后代**，
  step 2/6/7 flush（positioned descendants 晚于 in-flow content + CB bg/border 绘制）。supersede R503。
- MEMORY [[r504-appendix-e-global-deferral-landed]]。

**block A 的本质 = "abspos img（positioned descendant）在 draw_order 中绘制过早，被 in-flow 图元覆盖"**。
R504 的 global positioned-descendant deferral **正是修这个**：positioned 后代现统一收集到 step 6 flush，
绘制**晚于** in-flow content → abspos img 不再被 #div1 border/bg 覆盖 → **block A 很可能已由 R504 解除**。

## 重开评估（须 reftest 确认，本轮 agent 占用未跑）

- **block A**：很可能 R504 已解（机制直接对应）。须 retest absolute-replaced-width-006 IMGDBG 确认 img blue_px>0。
- **block C**：仍 R490-entangled（margin-collapse-through-border），**只阻塞带此结构的特定案**（006 子组 B ~7 案）。
- **★ 广 yield**：CSS2 全量 **285 文件**用 `<img width=/height=>`（positioning 52/normal-flow 35/backgrounds 22/borders 19），
  多数**无 margin-collapse-through-border 结构** → block A 解除后这些案 hint 应可见 yield。
  R502 当时只 A/B positioning/absolute-replaced-width-*（11 案，受 block C 拖累），未扫 285 文件 broad yield。

## 结论与建议（高价值重开候选）

R502 presentational-hints 是 **R491–R511 CSS-correctness lever era 中唯一被「后续 paint-order 修复」
部分解禁的 verified-correct lever**。block A（draw_order 覆写）很可能已由 R504 解除，剩余 block C（R490-entangled）
仅阻塞特定子集。

**建议**（code agent，R548 后 / agent idle 时）：
1. retest `reftest-upstream absolute-replaced-width --jobs 3` 确认 block A 是否真解（img blue_px>0）。
2. 若 block A 解：重投 R502 实现（seam 仍 verified-localizable：`style-system/lib.rs` 步 1.5a + `tree.rs:191` partial % path），
   A/B 扫 CSS2 broad（normal-flow/backgrounds/borders 用 img w/h 的案，非仅 positioning），量真实 yield。
3. 006 子组 B（margin-collapse）仍 FAIL = block C 独立，不阻塞 broad yield。
4. 预期：若 block A 真解，R502 可能 yield **broad +N**（285 文件中无 margin-collapse 的子集，远超原 positioning 11 案）。

## 不改 master.md 的原因

agent 正活跃验证 R548（13-job normal-flow A/B + 即将提交 R548+master.md 条目）。本评估写 evidence
（additive、零冲突），待 R548 落地、master.md 安全后折进 item 2（把 presentational hints 从「⛔ REFUTED blocked 重投」
降级标注「★ block A 疑被 R504 解除，待 retest 重开」）。

★ 此发现纠正 MEMORY [[r502-presentational-hints-zero-yield-refuted-code-reverted]]「seam 待两阻塞解后重投」的悲观框架——
两阻塞之一（block A）很可能已解，重投门槛降低。
