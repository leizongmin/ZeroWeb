# 经验：reftest 布局诊断必须用 empirical ZW-output 验证，不能只靠 code-trace

**日期**：2026-07-20（R1820 修正 R1819 错误根因分析）
**相关模块**：tests/wpt-runner（reftest harness）、crates/layout-engine（multicol 等）
**来源轮次**：R1817（code-trace 诊断）→ R1818（实现 + A/B 零效果 + revert）→ **R1820（empirical 验证推翻 R1817/R1818 全部假设 → 真 fix LANDED net +1）**

## 问题描述

R1817 通过**纯代码追踪**（读 multicol forced-break 推进守卫）诊断 `multicol-fill-auto-005`（1.87% diff）
为「forced-break overflow column 不创建」bug，声称「clean lever 首次浮现、fully root-caused」并写出完整
fix sketch。

R1818 按 fix sketch **完整实现**（kill-switch + 两 assign 函数 + overflow mirror），A/B
（`ZW_MULTICOL_FORCED_OVERFLOW=1 make reftest-oracle DIR=css-multicol`）= **零效果**（181/452 byte-identical）。

## 根因分析（★ R1820 修正：R1819 此节原写的「font-wall」根因本身也是错的）

R1817 的 code-trace 诊断**未经 empirical 验证**，属假阳性。但 R1819（本文档原版）的「回溯哪一步错」**又一次
靠推理猜**——猜「1.87% 是 `<p>` font-wall、geometry 已正确」，**这个猜测本身也未经验证，而且是错的**。
R1820 用 `REFTEST_DEBUG=1 make reftest-oracle DIR=multicol-fill-auto-005` 实测 ZW fill 几何：

- `fill[1]: (8,50.6,100,160) rgba(0,128,0,255)` —— 绿 multicol 容器 **160px 高**（= 20+40+100 全堆一列）
- chromium = 100×100 → **1.87% diff = 真 geometry bug（绿溢出红 60px ≈ 视口 1.25% + `<p>` 文本），非 font-wall**

临时 dispatch probe（`ZW_MULTICOL_DEBUG`，已移除）确证 **R1817 code-trace 全部假设错**：
- `height_limit=160 非 0`（R1817 说 height_limit=0）→ 走 line 690 第一分支，**非** R1817 说的 `_sequential`
- 全 3 子 `forced=true`（R1817 隐含「未检测」错）→ 走 `_with_breaking`（line 723），**非** `_sequential`
- ⇒ **R1817 诊断的函数根本没被调用**，R1818 改错函数故 A/B 零效果（非 env 传播——R1820 实测 env 正常 propagate）

**真根因两层**（R1820 确证）：① 主路径 `let _ = position_multicol_children`（multicol.rs:797）**丢弃 region_height**
→ 容器高永不按列分配重算，保持 taffy 预算自然和（160）；② `_with_breaking` 末列 forced break 不创建溢出列。
两层须同修。R1820 LANDED fix → css-multicol **181→182 net +1 零回归**（multicol-fill-auto-005 1.87%→0.62% FLIP）。

## 解决方案 / 可复用模式

**reftest 布局诊断的强制 empirical 验证清单**（实现 fix 前必做）：

1. **empirical geometry 优先（最高优先级）**：用 `REFTEST_DEBUG=1 make reftest-oracle DIR=<case-substring>`
   跑单 case，读 `fill[...]` / `image[...]` 实际 origin+size+color（reftest.rs:518 stderr dump），**确认 ZW 输出
   的真实几何**。这是区分「geometry bug」vs「font-wall」vs「code-trace 假阳性」的**决定性工具**。
   - R1820 正是靠这一步推翻 R1817/R1818/R1819 三轮推测（code-trace 140 / font-wall 100 / 实际 160 三者全不同）。
2. **临时 dispatch probe**：在疑似函数入口加 `if std::env::var("ZW_X_DEBUG").is_ok()` 临时 eprintln 关键变量
   （分支选择、forced_breaks、height_limit 等），单 case 跑一次定位真路径。**验证后必须移除**（非 production code）。
3. **diff 量级 + Ahem 判断仅作辅助筛**：high-diff（>5%）Ahem/无文本案更可能是真 geometry bug；但 near-pass
   （1-3%）**也可能是真 geometry bug**（multicol-fill-auto-005 = 1.87% 真 bug！），**不可仅凭量级跳过**——
   必须用第 1 步 empirical geometry 确认。R1819 原版「near-pass + 无 Ahem = font-wall 跳过」**被 R1820 证伪**。
4. **kill-switch A/B 的 env 传播**：`VAR=1 make target` 经 make → test-guard（scripts/test-guard.rs:165
   `Command::new` 不 sanitize）→ cargo run → binary **正常 propagate**（R1820 实测确认）；A/B 零效果时
   **默认是「诊断错」而非「env 未到」**，先做第 1 步 empirical geometry 验证。

## 如何避免

- **不要把 code-trace 推测当「fully root-caused」**：code-trace 给的是「假设」，须 empirical（REFTEST_DEBUG
  实际 geometry）才升为「确诊」。
- **不要用推理「回溯」诊断错在哪**：A/B 零效果后，**不要猜**「大概是 font-wall」——这本身又是一个未经验证的
  诊断（R1819 的错）。必须用 REFTEST_DEBUG empirical 查 ZW 真实输出。
- **A/B 零效果即 revert**：fix 实现后 A/B 若零 flip，不 land；但 revert **不等于 vein 关闭**——可能是 fix 目标
  错（错函数/错理解），empirical 重查可能找到真根因（R1820 即如此，R1818 revert 后 R1820 同 vein net +1）。
- **near-pass 带（1-3%）不全是 font-wall**：multicol-fill-auto-005（1.87%）是真 geometry bug。near-pass 须
  empirical 确认后再决定跳不跳。

## 关联

- memory `[[r1155-nearpass-band-fontwall-instruction-floor]]`：near-pass 带 = `<p>` font-wall（**注意：R1820
  证伪其普适性**——near-pass 也可能是真 geometry bug，须 empirical 区分）。
- master.md R740/R1817/R1818/R1820：code-trace 假阳性四证；R1820 = empirical 验证的正例（同 vein net +1）。
- evidence/r1820-multicol-forced-break-overflow-landed-2026-07-20.txt：R1820 完整诊断 + fix + A/B。
