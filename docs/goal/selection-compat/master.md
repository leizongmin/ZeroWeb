# Selection 兼容二期运行时控制面板

**状态**: Active——M1 主体完成（DC-1/2/3/4 全部落账，2026-11-04）；M2 待启动：
就地收敛 `Selection.modify()`（C2 簇 190F 判据内）
**最后更新**: 2026-11-04

---

## 当前状态

M1 主体完成。入口文档 [`../selection-compat.md`](../selection-compat.md)（v2.1）为
唯一目标契约。

- **DC-1 完成**：selection/ 语料 191 blob 与 pin `315976933870` 逐哈希核对
  0 mismatch（2026-10-30 深拉落盘）；14 项 pin 漂移 universe 文件按
  net-api M2-S12 先例对齐上游 HEAD `43f66fcb72f7`（8 文件补拉，idlharness
  记账排除先例）。runner 通道 `selection/` 根改递归扫描（skip 规则：
  manual/ref/crashtests；editing 两目录保持一期 flat 锚面）。证据：
  [`evidence/2026-11-04-m1-corpus-expansion.md`](evidence/2026-11-04-m1-corpus-expansion.md)。
- **DC-2 完成**：扩面基线首跑落账（guard 包裹，墙钟 ~11 分钟）——
  总账 35,637P/16,116F/4N（51,757 subtests，675 案，68.86%）；selection 面
  22,303P/13,756F/3N（36,062）。一期锚面 25 案 subtest 计数与 wpt.fyi 三家
  browser 逐案一致，一期两处修复无回退；对一期锚新增 4F 全部内容归因
  33 号流共享路径（见 T1）。逐 subtest 锚：
  [`evidence/2026-11-04-m1-baseline.json.gz`](evidence/2026-11-04-m1-baseline.json.gz)。
- **DC-3 完成**：失败簇互斥分区与 selection 面 F 闭合（13,756F）——
  C1 边界点校验族 13,277F（超限→D2）、C2 modify 缺失 190F（M2 就地收敛）、
  C3 contenteditable 其余 153F、C4 shadow/getComposedRanges 64F（新引擎
  概念域）、C5 拖选/布局命中 49F+锚面 1F（runner 基建）、C6 caret 键盘 18F、
  C7 anonymous 3F、C8 details 空壳 1F（33 号回归，见 T1）。
- **DC-4 完成**：coverage 复测（wpt.fyi aligned stable 三家并集同口径）——
  universe 194 文件覆盖 158；未覆盖 = idlharness 112 子测试（记账排除）+
  crashtests 35 文件（0 权重），**plain/query-variant 未覆盖 = 0，清零路径
  成立**；selection 通过率 61.63%（立项 8.09%）。
  三项综合证据：
  [`evidence/2026-11-04-m1-baseline-attribution.md`](evidence/2026-11-04-m1-baseline-attribution.md)。

**下一步（M2）**：就地收敛 `Selection.modify()`（C2 簇 190F ≤300 且零新
引擎概念，符合 M2 判据）——实现后幂等重跑基线对照零回归；C1 超限簇已挂
D2 待用户决策。

**与兄弟 goal 的边界**：

- 33 号 editing-compat——**ZeroWeb-3 工作区并行在跑**：selection/Range 的
  editing 依赖簇归 33 号，本 goal 只挂账登记、rally 不写对方控制面
  （2026-10-31 用户决策，避免跨工作区提交撞车）。
- 34 号 dom-expansion（dom/ranges）——共享 Range 底座，收敛轮核对互不回归。
- 共享底座跨界事项按 run-rules §9 先归口再动手。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| S1 | selection 语料 171/194 文件未覆盖（31,638 子测试，10-31 快照） | **已关闭**（DC-1+DC-2：语料全量+递归通道+基线落账，universe 覆盖 158/194，可执行面清零） |
| S2 | 扩面后失败簇未归因（selection/editing/runner 分列） | **已关闭**（DC-3：互斥分区 8 簇与 13,756F 闭合） |
| S3 | coverage 复测未做（未覆盖清零路径） | **已关闭**（DC-4：plain/query-variant 未覆盖 = 0） |

## 里程碑状态表

| 里程碑 | 内容 | 状态 |
|---|---|---|
| M1 | 语料扩面 + 扩面基线 + 归因表（DC-1/2/3） | **完成**（2026-11-04，DC-4 一并收口） |
| M2 | 语义簇收敛（≤300 子测试且零新引擎概念判据，超限挂账）/ coverage 复测收口（DC-4） | 待启动——就地收敛 C2 `Selection.modify()`（190F）；DC-4 已提前完成 |

## 碰撞管理记录（run-rules §9）

- 2026-10-31：立项时无同域在跑碰撞；33 号 editing-compat 在 ZeroWeb-3 并行
  推进（编辑域底座共享，收敛轮核对三域互不回归）。
- 一期零回归底线锚：
  [`../archive/selection-compat/evidence/2026-10-14-m1-baseline.md`](../archive/selection-compat/evidence/2026-10-14-m1-baseline.md)。

## 待用户决策 / 待转登记

| # | 事项 | 状态 |
|---|------|------|
| T1-a | **33 号回归挂账**：`selection/deleteFromDocument-HTMLDetails.html :: abc{<details><summary>…`——期望 `abc<details></details>jkl` 得 `abcjkl`（details 空壳未保留）。内容归因 `7ddf00c39`（part01f deleteSelection 忠实移植，deleteFromDocument 走同一删除算法）；一期终态（10-15）全过。复验归 33 号侧 | 待用户转登记 33 号 |
| T1-b | **33 号回归挂账**：`editing/event.html :: Command fontSize value ""/"quasit"/"15px" input event` ×3——期望 input 事件 1 次实得 0。内容归因 `90c992231`（style-with-value 实应用改造后 `__zw_ce_value_validate` 拒绝路径零事件，与 event.html「无效值也派 input」期望冲突）。复验归 33 号侧 | 待用户转登记 33 号 |
| D2 | 超限语义簇立项建议（M1 归因产出）：**C1 Selection API 边界点校验族 13,277F**（addRange 5,104/collapse 5,772/extend 2,342 等——spec 边界点有效性校验：非法/畸形 Range 应拒绝或不动 anchor/focus，shim 直接应用）。修复面 ≫300 且涉及 Range 有效性判定，不在 M2 就地收敛判据内；C4 shadow/getComposedRanges 64F（tentative API + composed range 机制 = 新引擎概念域）一并挂账。**待用户决策**：是否立项 | 待用户决策 |
