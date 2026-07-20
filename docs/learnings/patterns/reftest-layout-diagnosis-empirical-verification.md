# 经验：reftest 布局诊断必须用 empirical ZW-output 验证，不能只靠 code-trace

**日期**：2026-07-20
**相关模块**：tests/wpt-runner（reftest harness）、crates/layout-engine（multicol 等）
**来源轮次**：R1817（code-trace 诊断）→ R1818（实现 + A/B 证伪 + revert）

## 问题描述

R1817 通过**纯代码追踪**（读 `assign_children_to_columns_sequential` / `_with_breaking` 的
forced-break 推进守卫 `current_col + 1 < col_count`）诊断 `multicol-fill-auto-005`（1.87% diff）
为「forced-break overflow column 不创建」bug，并写出完整 fix sketch，声称「clean lever 首次浮现、
fully root-caused」。

R1818 按 fix sketch **完整实现**（kill-switch + 两 assign 函数改 + multicol_overflow_column_count mirror），
`cargo check` clean + lib 测试 1223/0（default-off byte-equivalent），然后 A/B
（`ZW_MULTICOL_FORCED_OVERFLOW=1 make reftest-oracle DIR=css-multicol`）：

**结果：零效果**——181/452 (40.0%) 与 OFF baseline 字节一致，strict/near/mismatch 全 identical。
fix 对 corpus **零 case flip、零 diff 变化**。

## 根因分析

R1817 的 code-trace 诊断**未经 empirical ZW-output 验证**，属**假阳性**。multicol-fill-auto-005：
- **无 Ahem flag**（`<meta name="flags" content="ahem"/>` 不存在），内容是空 `<div>` + 顶部
  `<p>Test passes if...</p>` **默认字体**描述文本。
- 1.87% diff **很可能就是 `<p>` 描述文本 font-wall**（ZW Liberation 文本 vs chromium 文本 raster/
  度量子像素差），**geometry ZW 已正确**（容器 100px，非 code-trace 推测的 140px）。
- 故 fix 即使逻辑正确，对像素输出**零影响**。

这与 R740 早已发现的「doc-side 只读 lever 分析假阳性率高」、以及 [[r1155]]「near-pass（~1-3%）
diff 主导 = 测试页顶部默认字体指令 `<p>` 文本非 geometry」**完全一致**——R1817 重蹈覆辙。

## 解决方案 / 可复用模式

**reftest 布局诊断的强制 empirical 验证清单**（实现 fix 前必做）：

1. **确认测试是 Ahem / 无文本**：grep `<meta name="flags" content="ahem"/>` 或读测试内容。
   - 有 Ahem flag → 文本是 Ahem，**无 font-wall**，diff 是真 geometry → 可信诊断。
   - **无 Ahem flag + 有 `<p>` 描述文本** → diff **主导是 `<p>` font-wall**（near-pass 带），
     **不要当 geometry bug 攻**（[[r1155]] 方法论：勿挖 near-pass 带）。
2. **diff 量级判断**：1-3% near-pass 带 = `<p>` font-wall 嫌疑大；>5% 才可能是真 geometry bug。
3. **empirical geometry 验证**：实现前用 `LAYOUT_DUMP=1`（reftest.rs:513 `dump_layout_tree`）
   跑该 case，**肉眼看容器/子元素实际 abs_y / height**，确认 code-trace 推测的「错误几何」真实存在
   （如本例：确认容器 140 vs 100），而非 ZW 已正确。
   - 单 case LAYOUT_DUMP：harness 现按 dir 全量 dump（452 案输出巨大），可加 case 名 filter 或
     临时缩 scope。
4. **kill-switch A/B 的 env 传播核查**：`VAR=1 make target` 经 make → test-guard（继承 env，
   scripts/test-guard.rs:165 `Command::new` 不 sanitize）→ cargo run → binary **应当 propagate**
   （POSIX 标准）；若 A/B 零效果，先区分「env 未到」vs「诊断错」——hardcode kill-switch ON 重跑
   可区分（本例经推理判定为 font-wall，env 传播正常）。

## 如何避免

- **不要把 code-trace 推测当「fully root-caused」**：code-trace 给的是「假设」，须 empirical 验证
  （LAYOUT_DUMP 实际 geometry）才升为「确诊」。
- **near-pass（1-3%）+ 无 Ahem + `<p>` 描述文本 = font-wall**，直接跳过，转挖 high-diff（>5%）
  Ahem / 无文本案（[[r1155]]）。
- **A/B 零效果即 revert**：fix 实现后 A/B 若零 case flip / 零 diff 变化，**不 land**（违
  code-guidelines「不做零价值修改」+「目标驱动验证」），并回溯诊断哪一步错（通常是无 Ahem =
  font-wall）。

## 关联

- memory `[[r1155-nearpass-band-fontwall-instruction-floor]]`：near-pass 带 = `<p>` font-wall 方法论。
- master.md R740：doc-side 只读 lever 分析假阳性率高。
- master.md R1817/R1818：本经验的具体案例（multicol forced-break overflow 假阳性 + A/B 证伪 + revert）。
