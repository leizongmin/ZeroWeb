# 历史轮次归档：R569–R572（CSS2 LANLED-impacted 子目录 DC-14 oracle 重跑 era）

> 归档自 `docs/goal/rendering-compat/master.md`（2026-06-25，R594 doc-maintenance 轮迁出，超出「最近 20 轮」窗口）。内容 100% 保留。综合结论见 master.md「综合裁决」point 3 + R580（CSS2 aggregate finalized 46.0%）+ R589。

---

**R569-R572（CSS2 LANLED-impacted 子目录 DC-14 oracle 重跑：borders/normal-flow/positioning/tables；3 真 win + 1 flat，净 +62 case；★证实 CSS2 sizing/border LANLED fix 真改善 chr<1%；零代码变更）**：CSS2 全量 6055 OOM-prone，本轮攻 4 LANLED-impact 子目录（复用 R484 per-case 基线 r484-css2-crossval-raw 算 delta）。**结果**：borders 377/506=74.5%（**+1.8pp**，R549/R550）/ normal-flow 555/746=74.4%（**+4.4pp 最大**，R544 ex + R546 ref 解封 +149 + R545/R548）/ positioning 206/520=39.6%（**+2.2pp**，R502/R549）/ tables 44/361=12.2%（-0.3pp FLAT，272/361 near-miss 主导）。**净 +62 case**（+9/+42/+12/-1）。★ **证实 CSS2 LANLED fix 真改善 DC-14 真目标**——CSS2 fix 多为 sizing/border-family（ex/inherit/refs/border-w/currentColor/presentational/col-w）= DC-14 高 yield 谱系，vs 非 CSS2 text-family（grid/flexbox/text）jitter-flat 低 yield，两类 yield 两极分化在 CSS2 vs 非 CSS2 再证。★ borders self-vs-chr gap 仅 11pp（layout/sizing 类 self 可信）vs text 目录 64-72pp。CSS2 全量仍 OOM-prone（剩 ~30 子目录分批）；本轮 4 子目录（2133 case=CSS2 35%）证 R484 44.1% 将上升。详见 [`evidence/r569-r572-css2-subdirs-dc14-revalidate-2026-06-24.txt`](../evidence/r569-r572-css2-subdirs-dc14-revalidate-2026-06-24.txt)。
