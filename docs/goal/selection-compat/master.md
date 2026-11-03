# Selection 兼容二期运行时控制面板

**状态**: Active——M1 完成（DC-1/2/3/4 全部落账，2026-11-04；含
deleteFromDocument-HTMLDetails 翻转就地修复，一期锚净零回归）；M2 判据落定
（2026-11-04 收口轮）：就地收敛目标 = C3 initial-selection-on-focus
（149F，独立语义）；C2 整簇挂账立项建议（D3）
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
- **DC-2 完成**：扩面基线落账（guard 包裹，墙钟 ~11 分钟）——同日两轮：
  首轮 01:32 落 35,637P/16,116F 并发现 HTMLDetails 翻转，就地修复后复跑
  01:43 为 canonical：**总账 35,638P/16,115F/4N（51,757 subtests，675 案，
  68.87%）；selection 面 22,304P/13,755F/3N（36,062）**。一期锚面 25 案
  subtest 计数与 wpt.fyi 三家 browser 逐案一致，一期两处修复无回退；首轮
  4F 翻转清算：HTMLDetails ×1 本轮就地修复转绿（净零），fontSize ×3 归因
  33 号流共享路径（见 T1）。逐 subtest 锚：
  [`evidence/2026-11-04-m1-baseline.json.gz`](evidence/2026-11-04-m1-baseline.json.gz)。
- **DC-3 完成**：失败簇互斥分区与 selection 面 F 闭合（13,755F）——
  C1 边界点校验族 13,277F（超限→D2）、C2 modify 缺失 190F（复判定稿：
  整簇挂账→D3）、C3 contenteditable 其余 153F（细拆定谳：initial-selection
  -on-focus 149F 独立语义 = M2 就地目标）、C4 shadow/getComposedRanges
  64F（新引擎概念域）、C5 拖选/布局命中 49F+锚面 1F（runner 基建）、
  C6 caret 键盘 18F、C7 anonymous 3F、C8 details 空壳 **0F——本轮就地修复**
  （deleteContents R279 反 spec 特判撤销）。簇明细与处置：
  [`evidence/2026-11-04-m1-baseline-attribution.md`](evidence/2026-11-04-m1-baseline-attribution.md)。
- **DC-4 完成**：coverage 复测（wpt.fyi aligned stable 三家并集同口径）——
  universe 194 文件覆盖 158；未覆盖 = idlharness 112 子测试（记账排除）+
  crashtests 35 文件（0 权重），**plain/query-variant 未覆盖 = 0，清零路径
  成立**；selection 通过率 61.63%（立项 8.09%）。
  三项综合证据：
  [`evidence/2026-11-04-m1-baseline-attribution.md`](evidence/2026-11-04-m1-baseline-attribution.md)。

**下一步（M2）**：就地收敛 C3 `initial-selection-on-focus.tentative.html`
?div/?span（149F；失败形态 99/99+50/50 全为「caret 期望 1 实得 0」=
编辑宿主 focus 未挂初选区，shim 级 focus 钩子 + caret 置位，零新引擎
概念）——实现后幂等重跑基线对照零回归；同轮复核同文件 collapse 3F +
selection-outside-host 1F。C2 判据复判定稿：整簇挂账立项建议（D3）——
bidi 92F 视觉序 + line/lineboundary 17F 行盒 = 引擎概念域，余 ~80F shim
无现成字符/词移动底座；C1/C4 超限簇已挂 D2 待用户决策。

**与兄弟 goal 的边界**：

- 33 号 editing-compat——**已迁 ZeroWeb-2 工作区**（2026-11-04 进程面核实，
  其 rally 与实例 cwd 均为 ZeroWeb-2；本仓 origin/main 当日仍见其 3 个
  docs 提交）：selection/Range 的 editing 依赖簇归 33 号，本 goal 只挂账
  登记、rally 不写对方控制面（2026-10-31 用户决策，避免跨工作区提交撞车）。
- 34 号 dom-expansion（dom/ranges）——共享 Range 底座，收敛轮核对互不回归。
- 共享底座跨界事项按 run-rules §9 先归口再动手。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| S1 | selection 语料 171/194 文件未覆盖（31,638 子测试，10-31 快照） | **已关闭**（DC-1+DC-2：语料全量+递归通道+基线落账，universe 覆盖 158/194，可执行面清零） |
| S2 | 扩面后失败簇未归因（selection/editing/runner 分列） | **已关闭**（DC-3：互斥分区 8 簇与 13,755F 闭合） |
| S3 | coverage 复测未做（未覆盖清零路径） | **已关闭**（DC-4：plain/query-variant 未覆盖 = 0） |

## 里程碑状态表

| 里程碑 | 内容 | 状态 |
|---|---|---|
| M1 | 语料扩面 + 扩面基线 + 归因表（DC-1/2/3） | **完成**（2026-11-04，DC-4 一并收口；含 HTMLDetails 翻转就地修复） |
| M2 | 语义簇收敛（≤300 子测试且零新引擎概念判据，超限挂账） | 判据落定（2026-11-04 收口轮）：就地收敛目标 = C3 initial-selection-on-focus 149F；C2 整簇挂账 D3；C1/C4 已挂 D2。待实现收敛 |

## 碰撞管理记录（run-rules §9）

- 2026-10-31：立项时无同域在跑碰撞；33 号 editing-compat 并行推进（编辑域
  底座共享，收敛轮核对三域互不回归）。
- 2026-11-04：M1 收口轮无同域碰撞（33 号已迁 ZeroWeb-2 工作区）。本 session
  曾出现同 session 并行实例竞态（master.md/证据文件被并行改写、探针被并行
  清理），按既定预案清理重复实例后以 canonical 轮落账（两轮面集恒等，账取
  修复后轮）。竞态处置收尾（收口轮）：01:45 追加的 canonical 确认复跑在跑
  期间被同树 make 重建 `target/test-guard` 触发自 exec 竞态杀死（整链无
  exit 落盘，canonical 账不依赖该轮）；并行实例经进程树核实（00:56 resume
  实例与 01:51 新轮实例并存）后按预案终止前者。连带修复：被杀构建遗留
  `target/debug` 构件投毒（css-parser 陈旧构件指纹自称新鲜）曾使 make test
  编译腿假红（`AlignmentValue::Safe*`/`CaretShapeValue` 缺失型陈旧诊断，
  单包 check 全绿反证），`cargo clean -p` 三包后复跑转真。
- 2026-11-04（收口轮门禁账）：make test workspace 腿 3722P/5F——5 红 = s249
  `652a78e00` CSSOM 序列化尾分号过期期望（渲染流域在册红，webcrypto-v3
  2026-11-03 收口账逐名同族），按 run-rules §9 不代修，goal 面自身全绿；
  fmt/clippy（默认 feature 全量 -D warnings）零告警。
- 一期零回归底线锚：
  [`../archive/selection-compat/evidence/2026-10-14-m1-baseline.md`](../archive/selection-compat/evidence/2026-10-14-m1-baseline.md)。

## 待用户决策 / 待转登记

| # | 事项 | 状态 |
|---|------|------|
| T1 | **33 号回归挂账**：`editing/event.html :: Command fontSize value ""/"quasit"/"15px" input event` ×3——期望 input 事件 1 次实得 0。内容归因 `90c992231`（style-with-value 实应用改造后 `__zw_ce_value_validate` 拒绝路径零事件，与 event.html「无效值也派 input」期望冲突）。复验归 33 号侧 | 待用户转登记 33 号 |
| T2 | **撤账更正（2026-11-04）**：已推送的 `9406a986f` 曾将 deleteFromDocument-HTMLDetails 翻转挂为 T1-a、内容归因 `7ddf00c39`→33 号；本轮探针隔离 + 逐提交 diff 实证根因为本 goal 域内 `a8ec673b1` 反 spec 特判（Range deleteContents ec 尾边界整删，命中面依赖解析 wrapper childNodes 同步态），已就地修复转绿（C8=0F，详见总证据文锚面对照节）——33 号侧勿接此票 | **已关闭**（就地修复） |
| D2 | 超限语义簇立项建议（M1 归因产出）：**C1 Selection API 边界点校验族 13,277F**（addRange 5,104/collapse 5,772/extend 2,342 等——spec 边界点有效性校验：非法/畸形 Range 应拒绝或不动 anchor/focus，shim 直接应用）。修复面 ≫300 且涉及 Range 有效性判定，不在 M2 就地收敛判据内；C4 shadow/getComposedRanges 64F（tentative API + composed range 机制 = 新引擎概念域）一并挂账。**待用户决策**：是否立项 | 待用户决策 |
| D3 | 语义簇立项建议（M2 判据复判定稿，2026-11-04 收口轮）：**C2 `Selection.modify()` 190F**——非标 tentative API：bidi 92F 需视觉序字符移动（引擎无 bidi 文本布局）、line/lineboundary 17F 需行盒查询（layout 桥新面）、余 ~80F shim 无现成字符/词移动底座且 left/right 语义本就视觉序。**待用户决策**：是否立项 | 待用户决策 |
