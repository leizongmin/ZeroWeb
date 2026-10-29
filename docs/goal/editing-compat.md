# 编辑命令兼容 — editing/run + editing/other

**版本**: v1.6
**日期**: 2026-10-27（立项，coverage 对账驱动）；2026-10-30（M1 收口）；2026-10-30（R5331 + M2 首轮）；2026-10-30（M2 次轮双窗折叠：预算归因 + other 面复跑收口；自旋簇三分类 + R5340）；2026-10-30（M2 次轮收环：缺陷 #2 已修 R5341 + other 面墙钟 R5342，三例 Timeout 全收口）；2026-10-30（簇 A 归因收口：填充/物化逐节点 JS glue 定量，修复 scope 定案）
**状态**: Active——M2 次轮完成并收环；簇 A（cloneNode 性能）归因收口（修复下轮立项）。预算窗：自旋簇主体系 runner V8 看门狗 `ZW_SCRIPT_TIMEOUT_MS` 90s 预算截断（非死循环，corpus 脚本已修 300s/变体 + CASE_WALL_S 自适应），other 面同预算复跑收口 **2,404P/11,159F/3T**；三分类窗：cloneNode 前置成本性能簇（挂账）、removeChild 对 sel 父 pending 解析子静默穿透真自旋（**已修 R5340**）、runner 完成接线过早置 all_loaded（缺陷 #2，**已修 R5341**：静态尾改 load+setTimeout(0) 门控镜像原生时序，write 注入路径维持直调；回归钉两枚 + 探针双 Pass + scripting-1 verdict 流零影响）。R5340+R5341 双落复跑：预算窗三例 Timeout **全部收口**（① delete-without-unwrapping 200F/0T、② inserttext-last-mutation 64F/0T、③ white-space join 三案实为 other 面平墙 120s 过紧的外部截断假象——corpus 脚本 **R5342** other 面接线自适应墙 60+nv×15+90；R5342 后 other 面全量复跑终账 **2,406P/12,308F/0T**，Timeout 清零、分母完整、总账逐位核对：F = 基线 11,159 + R5340 解锁 1,144 + R5341 回归 5）
**执行模式**: WPT 驱动（上游 editing corpus 为验收标尺）+ 长线实现推进
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——编辑面）

> **M2 首轮当前状态（2026-10-30，R5331 之后的同日续环）**：js_dom_shim
> 四笔共享路径修复（matches 自含语义 / doc 级查询深层解析产物 in-doc 链上行
> ——并行撞车后折叠进 R5331 的 `_zwPendingNodeInDoc` 作 fallback / cloneNode
> 融合源读 / 子树脏门 ID 锚定感知），闭合 R5331 指认的 cloneNode 断点环。
> 探针 F/G/E 全绿，全量重跑 run 48.78% / other 32.03%（**分母收缩**
> 96,663→15,343：setup 解锁后变体级 90s 自旋中止取代全分母 setup 快败）。
> **自旋簇已归因（同日二轮，并行撞车折叠一笔后本侧核验推进）**：非死循环
> ——run 面大案每变体 ~1000 子测试同步脚本段 ~200-300s，runner 内部 V8
> 看门狗 `ZW_SCRIPT_TIMEOUT_MS`（缺省 90s）是同步段实际预算边界（外层
> CASE_WALL_S 拦不到同步段），90s 内跑不完整变体 aborted 无子测试行。
> 实证：bold 四变体 90s 预算 12 行 → 300s 预算 3048 行全出（1138P/1910F，
> 单案隔离跑）。corpus 脚本已改：run 通道缺省预算 300s + CASE_WALL_S 按案
> 内变体数自适应。全分母重跑（3 分片）进行中（bold 已按新预算全矩阵
> 发射）；other 面同预算复跑同窗启动。**自旋分类三例已出（单案 600s
> 放宽探针）**：① delete-without-unwrapping-first-line-of-child-block 与
> ② inserttext-last-mutation 非
> 预算族——300s 脚本预算下 ~3min 吃满 4G scope OOM、150s 零子测试行，
> 编辑循环不收敛且 DOM 无界增长，M1「list/inline delete 循环疑似
> 不终止」成立——真引擎簇（delete/inserttext 收敛性），预算解锁不覆盖，
> 归入 execCommand/delete 语义簇序列；③ join-different-white-space-style-
> paragraphs（white-space 簇旗舰）**有限慢**：300s 预算内跑完整案 3,456
> 子测试行（exit=1 正常带 Fail）——white-space join 簇随预算解锁，仅需
> 墙钟 ≥ 实际耗时，非引擎簇。**other 面同预算复跑已收口（2026-10-30 三轮）**：
> 96 块/86 有效，账面 **2,404P / 11,159F / 3T**（transcript
> /tmp 快照，JSON 按脚本 summarizer 再生）——Pass 与 M2-r1 完全一致
> （2,404=2,404，同 shim 只解锁发射的零回归信号）；Fail 5,096→11,159 为
> 分母 7,505→13,566 纯发射解锁（原被 90s 截断的变体现全量出断言行），
> 3 个 Timeout 即上述①②③ runaway 三例（timeout-only 块逐一核对）。
> 曾记「双实例案块双倍」经核查为误读：840 体行 = 420 唯一 Fail + 420 缩进
> 断言续行（单实例天然形态），块替换 + JSON 再生后总额逐位不变、无重复
> 案块——账面无需修正。**重跑在途余一笔待办**：自适应墙钟 nv=0 下限
> 150s 偏紧——300s 脚本预算下无变体大单页案（createlink-with-selecting-img
> 实证）真实耗时超 150s 被 externals kill（M2-r1 时代是 90s 看门狗先截断
> 才显得 420 够用），分片退出后修 420 下限 + 重跑误杀案（run 面 s2 在途）。
> run 面全分母 3 分片重跑进行中；窗口内并行流落 94eb9f0d2（共享面
> part03/webview/testharness）——在途账面为落前二进制、对 M2-r1 对照仍
> 可比，收口时按 evidence 时点注记以新二进制抽查核账。失败簇按新账重排后定
> white-space join / insertparagraph 入簇序。归因机制、三分类与 other 面
> 终账明细见
> [evidence/2026-10-30-run-budget-unlock-rerun.md](editing-compat/evidence/2026-10-30-run-budget-unlock-rerun.md)；首轮修复与 M2-r1 账面见
> [evidence/2026-10-30-m2-r1-innerhtml-materialization.md](editing-compat/evidence/2026-10-30-m2-r1-innerhtml-materialization.md)。

> **M2 次轮·三分类窗（2026-10-30，与预算窗并行折叠、本侧收环）**：自旋簇
> 归因深化为**三个独立事实**（非单一根因）。簇 A：run 面墙钟杀 =
> cloneNode 前置成本 ~54ms/次的线性放大——runConformanceTest 每行数据
> `documentElement.cloneNode(true)`（tests.js 5582 行），bold 四变体 3048
> 行 = 深克隆物化 × 数据行数 × 变体页数的线性放大，非死循环，性能簇与
> 墙钟调优一并挂账。簇 B：真自旋 = removeChild 对 sel 父 pending 解析子
> 静默穿透——innerHTML 同 turn 解析产物（无 handle/sel/`__zwIsText` 印）
> 在 removeChild 分支网既有四分支全不命中，末尾静默 `return child` 后父
> 融合视图仍含失效子，删除标记循环无限重扫同一 firstChild；**R5340 已修**
> （part04.js 分支链新增 sel 父 + 融合视图 identity 分支 → host
> `__zw_remove_child_at` + 桶 retract + 迭代器/range 通知），delete-in-last
> 家族两案修复前 90s 脚本中止、修复后 44 断言全出真实语义 Fail 0 Timeout，
> createlink-with-selecting-img 出真实语义断言——预算窗三例 Timeout 中的
> delete 族案待新二进制（94eb9f0d2 + R5340）复跑核对是否由此解除；
> ②inserttext OOM 簇（DOM 无界增长）非本根因、仍开放，归 execCommand/
> delete 语义簇序列待立项。簇 C（缺陷 #2）：runner
> `apply_harness_completion_patch` 尾调在页 load listener 之前置
> all_loaded，load handler 内同步 `test()`（声明即同步执行）首测 done 即
> 提前 complete，后续声明被 `Tests.push` 静默丢弃（真实浏览器原生 load+
> setTimeout(0) 时序严格晚于全部 load handler，无此形态）——other 面
> 29/96 load 形态案中 handler 内同步 test() 子集只报 ≤1 测试、分母低估；
> 修复受 270896e25（write 注入路径 102 伪 Timeout 翻绿）约束不能简单
> 删除。归因全文、R5340 验证与缺陷 #2 机制链见
> [evidence/2026-10-30-m2-r2-spin-cluster-attribution.md](editing-compat/evidence/2026-10-30-m2-r2-spin-cluster-attribution.md)。
> 下一步（2026-10-30 收环更新）：缺陷 #2 **已修 R5341**（静态尾 load+
> setTimeout(0) 门控镜像 vendored harness 原生时序，write 注入路径维持
> eval 尾直调——270896e25 依赖不动；行为钉+形态钉两枚回归钉常驻 make
> test；探针双 Pass 验收；scripting-1 verdict 流零影响、102 Fail 核实为
> 先于 R5341 的既有账归并行流）。预算窗三例 Timeout 全收口（①②
> R5340 解锁出真实断言、③ 实为 other 面平墙 120s 过紧的截断假象，
> corpus 脚本 R5342 接线 other 面自适应墙 60+nv×15+90——join 三案
> nv=60 单案 ~150-170s 双二进制同值实证）。其余同上：cloneNode
> 性能簇 → white-space join → insertparagraph → execCommand 逐簇立项；
> run 面全分母复跑与收口抽查按预算窗时点注记执行。R5341 修复全文与
> 三例收口账见
> [evidence/2026-10-30-r5341-defect2-completion-wiring.md](editing-compat/evidence/2026-10-30-r5341-defect2-completion-wiring.md)。
> **簇 A 归因收口（2026-10-30 四轮探针 + 同日二轮修正）**：cloneNode/innerHTML
> 填充的 2 秒级成本全部在 JS shim glue 层——host 解析（1.3ms/2000 span）、
> 裸宿主队列（0.23µs/调）、createElement wrap（13µs 线性）、属性读（2-6µs）
> 全部免罪；主项 = t8r 逐节点物化的 appendChild trap 全链 glue（element 子
> ~286µs/次，text 子一侧 +~650µs 差额待 trap 内部插桩归因——初稿 plain 分支
> 机制断言经 part06.js:4014 复核证伪，text 子实为 handle 形），重设 2-4×
> （移除侧逐子 + 全局状态温和超线性）。消费者换算与修复约束（guideline #35
> 整体替换 A/B + dom/html-syntax/selection/web-components 跨域抽查）见
> [evidence/2026-10-30-cluster-a-clone-perf-attribution.md](editing-compat/evidence/2026-10-30-cluster-a-clone-perf-attribution.md)
> ——修复下轮立项（候选：循环不变量提出 1.5-2× / 常见形态快速门 5-10×）；
> R5342 后本簇为纯性能余量项，语料已解锁不受其阻。

## coverage 对账（coverage-2026-10-27 基线，47 套件干跑口径，终稿待 `docs/compat/wptfyi/coverage-2026-10-27.md` 复核）

| WPT 面 | 全量分母（子测试） | 覆盖 |
|---|---:|---:|
| editing/run | 96,745 | 0/120 文件 |
| editing/other | 16,339 | 2/510 文件 |

合计 **113,084 子测试**，是 encoding 之外最大的连片零覆盖域。

**M1 基线（2026-10-30，DC-1/DC-2/DC-3 落成）**：本地 pin 语料 run-face
128 案（run 42 + other 86，iframe/https 内容筛减后）106,736 子测试全量
干跑——run 7/96,663 = 0.007%、other 2,316/10,073 = 23.0%，合计 2.18%。
run 面是**单根因 setup 级联**（innerHTML 产物宿主侧身份失步——
`Element.matches` 解析不到 innerHTML 物化元素，占 Fail 99.96%，最小六探针
复现存档）；other 面白空间 join/insertparagraph/execCommand 三层真实语义簇。
通道（子命令 + 内容筛减 + Makefile 分片 target）、形态分簇、失败归簇与
M2 指针详见 [evidence/2026-10-30-m1-corpus-baseline.md](editing-compat/evidence/2026-10-30-m1-corpus-baseline.md)。
分母口径 = 本地 pin 语料 run-face 实测子测试（与 coverage 快照口径并行不
互换）；`docs/compat/trends/wpt-suites.csv` 已回填数据行。

**R5331 修复后复测（2026-10-30，3d173dccc）**：shim 查询回落 in-doc 门对
innerHTML 解析产物放行（`_zwSelPendingParent` 槽锚点爬升，R125 R5001 同款；
并行双流撞车后按上游版收敛，本 goal 侧只落引擎修复）。同分片器同口径复测：
run 面 96,656F → 49,778F（-48.5%，+7T）、other 面 2,316P → 2,325P（+9P/-6T），
逐案零 Fail 回归。setup 链断点后移至**第三环 cloneNode**（克隆读链走视图物化
子树，同 turn innerHTML 产物缺席）——M2 首簇不变；8 个大案（multitest/delete/
forwarddelete/insertparagraph/justifycenter 等）setup 全通后首次进入真实命令
执行面、超 420s 案墙钟转 Timeout，执行面信号待墙钟调优后完整暴露。明细见
[evidence/2026-10-30-r5331-postfix-rerun.md](editing-compat/evidence/2026-10-30-r5331-postfix-rerun.md)。
跨域核对（M2 指针要求）已落：dom + html-syntax 双腿同树 revert 对照零回归
（html-syntax 逐位一致 61,341P/62F/7T/1U；dom 6 案差异全为 timing/发射形态，
单案两侧隔离重跑判据），html-syntax 现行 pin 参照账 61,341/61,411 已记 CSV——
见 [evidence/2026-10-30-r5331-cross-domain-regression.md](editing-compat/evidence/2026-10-30-r5331-cross-domain-regression.md)。

## Mission

execCommand / queryCommand / queryCommandState 等编辑命令族语义面预置与收敛。
这是长线 goal：编辑命令语义面广（style/color/list/alignment/link/removeFormat
等数十命令 × HTML 片段矩阵），预期初始通过率低。策略是**先语料后引擎**：
M2 基线先把 Unsupported / Fail / Timeout 归簇摸清，再按簇立项收敛，不追求
一轮全绿。

## Done Criteria

- **DC-1（语料全量）**：33 号脚本拉全 editing/run + editing/other（deep 模式，
  include/ 共享 helper 随目录拉取）。
- **DC-2（runner 通道）**：testharness 子命令 + per-dir skip 规则 + Makefile
  target（M1 rally 轮落地）。
- **DC-3（基线分类）**：全量分类通过率落 `docs/goal/editing-compat/evidence/`；
  Unsupported 与 Fail 分列（能力缺失 ≠ 实现错误）；失败按命令族归簇；
  `docs/compat/trends/wpt-suites.csv` planned 行转数据行。
- **DC-4（收敛轮）**：按失败簇逐轮修齐，每轮零回归（逐 subtest 对照），
  evidence 落档；收敛目标与节奏按簇另行定稿。
- **DC-5（coverage 复测）**：editing 未覆盖子测试清零路径成立。

## 里程碑

- **M1**：语料预置 + runner 通道 + 基线分类（2026-10-30 完成——DC-1/DC-2/DC-3）。
- **M2**：按簇收敛（每簇一轮）。R5331（3d173dccc，查询回落槽锚点——并行
  双流撞车后按上游版收敛）+ 首轮（2026-10-30，innerHTML 物化身份失步全环：
  matches 自含 / 深层解析产物 in-doc / cloneNode 融合源读 / 子树脏门 ID 锚定
  ——R5331 指认的 cloneNode 断点环闭合；四笔 + 全量重跑 + 跨域零回归，见
  evidence）。次轮（2026-10-30，双窗并行折叠）：预算窗——run 面 ?3001-last
  形态实为 runner V8 看门狗 `ZW_SCRIPT_TIMEOUT_MS` 90s 预算截断（同步脚本
  段 ~200-300s/变体，bold 单案 12→3048 行实证，非死循环），corpus 脚本
  run 通道预算已修（300s/变体 + CASE_WALL_S 按变体数自适应），other 面同
  预算复跑收口 2,404P/11,159F/3T；三分类窗——自旋簇拆三事实：cloneNode
  前置成本性能簇（挂账）、removeChild pending 解析子静默穿透真自旋
  （R5340 修，delete-in-last 家族解锁验证，other 面 list/inline delete
  自旋案是否同根因就此收口）、runner 完成接线过早置 all_loaded（缺陷 #2，
  other 面 29/96 load 形态案账面不完整）。收环（2026-10-30 三轮）：缺陷 #2
  已修 R5341（testharness.rs 静态尾 load 门控 + 回归钉两枚 + scripting-1
  verdict 流零影响），预算窗三例 Timeout 全收口（①②R5340 解锁出真实
  断言、③ other 面平墙过紧截断假象由 R5342 修复——corpus 脚本 other 面
  接线自适应墙 60+nv×15+90，run 通道分支未动；该文件与预算窗挂账的
  420 下限重写共享，碰头信号记此）；其后 cloneNode 性能簇**归因已收口**
  （2026-10-30 四轮探针，逐节点物化 glue 定量，修复 scope 定案下轮按
  guideline #35 整体替换 A/B 立项——见簇 A evidence）→ white-space
  join → insertparagraph → execCommand 语义逐簇立项。
- **M3+**：扩面（edit-context / plaintext-only / whitespaces / chrome-compat
  等不在 DC-1 语料范围的邻接目录）与收口标准按簇定稿。

## 依赖约束（run-rules §9）

- 编辑命令实现与 uievents（50 号已收口，IME 挂账）、selection（23 号二期）
  共享编辑域路径——失败簇跨界时按 run-rules §9 先协调归口，不双边重复立项。
- 编辑面涉及 selection/Range 的部分与 34 号 dom-expansion（dom/ranges）有
  共享底座，收敛轮核对两域基线互不回归。
