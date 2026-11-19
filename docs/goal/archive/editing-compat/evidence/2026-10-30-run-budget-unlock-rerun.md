# editing-compat M2 次轮 — 自旋簇归因（预算截断）与全分母重跑（2026-10-30）

M2 首轮后 run 面分母 96,663→15,343 的「自旋簇」（大案变体 `declared tests: 0`、
无子测试行）本轮归因完成：**非死循环，是 runner V8 看门狗预算截断**。corpus
脚本预算口径已修（0acc26e5a），全分母重跑入账；other 面终账在本文件，run 面
待分片收口补记。

## 归因机制

- **现象**（M2-r1 遗留）：run 面大案（bold/justify 等）`?3001-last` 变体形态
  90s 后整变体中止、无子测试行——分母收缩 96,663→15,343 的主因。
- **机制**：runner 内部 V8 看门狗 `script_timeout_ms()`（env
  `ZW_SCRIPT_TIMEOUT_MS`，缺省 90_000，`crates/engine/src/testharness.rs`）
  是同步脚本段的实际预算边界——外层案超时只在 `run_page_scripts` 返回后
  tick，拦不到同步段。run 面大案每变体 ~1000 子测试同步段 ~200-300s（bold
  `browserTests.length=213`，上游 variant meta `?1-1000` 过量供给），90s 内
  跑不完整变体被 abort。
- **实证**（bold 单案隔离跑，唯一变量为脚本预算）：90s 预算 → 12 结果行；
  300s 预算 → **3,048 结果行全出（1,138P/1,910F，~0.2s/子测试）**。同一
  二进制同一案，预算放开即全量发射——归因闭合。

## 新预算口径（corpus 脚本 0acc26e5a）

- run 通道缺省 `ZW_SCRIPT_TIMEOUT_MS=300000`（显式 env 优先）。
- `CASE_WALL_S` 未显式设置时按案自适应：`60 + nv×330 + 90`（nv = 案内
  `meta name="variant"` 数；每变体 300s 脚本预算 + 30s 余量，60/90 为进程
  启停头尾）。multitest 10 变体需 ~3,400s，统一 420s 口径必然外部截断。
- **已知待办**：nv=0 下限 150s 偏紧——300s 脚本预算下无变体大单页案
  （createlink-with-selecting-img.html，s2 分片 150s external kill 实证）
  真实耗时超 150s（M2-r1 时代是 90s 看门狗先截断、才显得 420s 够用）；
  分片退出后修 420 下限并重跑误杀案。

## 自旋三分类（单案 600s 放宽探针，ZW_SCRIPT_TIMEOUT_MS=300000）

| 类 | 案 | 形态 | 处置 |
|---|---|---|---|
| ① 预算截断 | run 面全部变体案 | 同步段超 90s 被 abort，调大预算即全量发射 | 已解锁（新口径） |
| ② 真引擎 runaway | other 面 delete-without-unwrapping-first-line-of-child-block / inserttext-last-mutation / insertparagraph-in-inline-editing-host | 300s 脚本预算下 ~3min 吃满 4G scope OOM、transcript 0 字节（runner 按案缓冲、OOM 前无一字落盘）、零子测试行——编辑循环不收敛且 DOM 无界增长 | 预算不覆盖，归 delete/inserttext 收敛性引擎簇 |
| ③ 有限慢 | other 面 join-different-white-space-style-paragraphs（white-space 簇旗舰） | 300s 预算内完整跑完 3,456 结果行（0P/3,456F，exit=1 正常带 Fail） | 仅需墙钟 ≥ 实际耗时 |

②使 M1 悬置的「list/inline delete 循环疑似不终止」成立为**真引擎簇**；
③证明 white-space join 簇主体是**有限慢**、随预算解锁，非引擎簇。

## other 面同预算复跑终账

96 块 / 86 有效，**2,404P / 11,159F / 3T**
（[JSON 存档](2026-10-30-run-budget-unlock-rerun-other.json)）：

- **Pass 与 M2-r1 完全一致**（2,404 = 2,404）——同 shim 只解锁发射的零回归
  信号；被解锁的变体只新增 Fail 行，无一例新增 Pass 或反转既有 Pass。
- Fail 5,096→11,159 为分母 7,505→13,566 纯发射解锁（原被 90s 截断的变体
  现全量出断言行），非质量回退；通过率 32.03%→17.72% 的表观下降同为
  分母效应。
- 3 Timeout = 上表②三例（timeout-only 块逐一核对）。
- **勘误**：前轮记「双实例共写 transcript 一案块双倍（420 行）」经核查为
  误读——840 体行 = 420 唯一 Fail + 420 缩进断言续行（单实例天然形态），
  块替换 + JSON 再生后总额逐位不变、`grep '==== ' \| sort \| uniq -d` 无
  重复案块。

## run 面全分母重跑（在途，收口时补记）

3 分片 scope（`zw-editing-run-s0/s1/s2`，`EDITING_CORPUS_SHARD=0/3..2/3`，
transcript/JSON 各自独立——并行分片共享 transcript 会互相截断，M2-r1 实测
坑）。s2 有一笔 nv=0 误杀案待修下限后重跑。账面待三分片退出后合并补记，
并入 M2 失败簇重排。

**分片中途账（2026-10-30 06:0x 快照）**：s1 已退出（exit=0，14 案全出）
**10,937P / 14,437F / 0T**——bold/formatblock/delete 等 nv≥2 案在新预算下
全量发射、零 Timeout，预算解锁主簇验证成立；s0 在 justifycenter（10/14）、
s2 在 insertparagraph（8/14）。s2 误杀案即
createlink-with-selecting-img.html（150s external kill，transcript 第 6-8 行）。
二进制时点实测：runner mtime 04:51:27 < 94eb9f0d2 落时 05:42:54，在途分片
确为落前二进制。

## 自旋四案新二进制复跑终账（R5340 收敛轮，2026-10-30 收口）

四案以新二进制（mtime 07:30:11，含 94eb9f0d2 + R5340 + R5341）+
ZW_SCRIPT_TIMEOUT_MS=300000 + 按案自适应墙单案直跑（delete/forwarddelete
镜像流跑，multitest/insertparagraph 本侧）。**逐测试 ID 对照零 verdict
翻转**——四案全部新行与旧账同行同位同判定，全部增减均为截断前沿移动：

| 案 | 分片账 | R5340 后 | 行差 | 脚本超时变体 old→new |
|---|---|---|---:|---|
| delete | 3,713r 1,863P/1,850F | 4,483r 2,014P/2,469F | +770 | 3→3 |
| forwarddelete | 4,225r 1,948P/2,277F | 4,582r 2,028P/2,554F | +357 | 3→3 |
| insertparagraph | 6,705r 3,546P/3,159F | 6,531r 3,423P/3,108F | -174 | 1→1 |
| multitest | 9,001r 5,292P/3,709F | 8,735r 5,109P/3,626F | -266 | 1→1 |
| 合计 | 23,644r | 24,331r | +687 | 8→8 |

归因两条（互不矛盾，不同案不同主因）：

1. **removeChild 自旋解锁**（delete/forwarddelete +770/+357）：R5340 前
   自旋变体零行/少行，修复后出真实断言行。但对 M1 基线（7,842/7,491）
   仍差近半——每案各 3 变体在 300s 脚本预算内 declared-0，与
   insertparagraph ?7001-last、multitest 各 1 变体同属预算前沿未竟。
2. **R5340 分支开销的截断前沿左移**（insertparagraph -174/multitest
   -266）：零翻转 + 前缀包含证明纯截断——R5340 在 part04.js 分支链新增
   逐操作 identity 检查，重型编辑矩阵变体吞吐降 ~25%（insertparagraph
   ?6001-7000 每 300s 预算 704→530 行），同一预算窗内完成的测试变少。
   正确性无损（P/F 分布同前缀一致），是簇 A（cloneNode 逐节点 glue）
   性能簇定量的同向佐证：分母全面回收系于簇 A 修复，非预算再调优。

**R5340 收敛轮结论**：removeChild 自旋族解锁成立（delete/forwarddelete
真实断言出账），无任何正确性位移；行数增减均为簇 A 成本维度现象。
四案残余分母缺口（对 M1 -8,289 行）挂簇 A 修复后复测。

## run 面全分母重跑终账（2026-10-30 收口）

三分片全部 exit=0（43 案全出，零 case 级 Timeout），合并终账
**39,885P / 47,804F / 0T（87,689 行，45.51%）**，逐案 JSON 已存档
（[2026-10-30-run-budget-unlock-rerun-run.json](2026-10-30-run-budget-unlock-rerun-run.json)）：

| 分片 | 案 | Pass | Fail | Timeout |
|---|---:|---:|---:|---:|
| s0 | 15 | 13,941 | 18,202 | 0 |
| s1 | 14 | 10,937 | 14,437 | 0 |
| s2 | 14 | 15,007 | 15,165 | 0 |

对照 M2-r1（15,343 分母 / 7,485P / 48.78%）：分母 **5.7× 解锁**
（15,343→87,689）、case 级 Timeout 20→0、Pass 7,485→39,885；通过率
48.78%→45.51% 的表观下降同 other 面一样是分母效应，非质量回退。

**分母对 M1（96,663）尚差 -8,974 行，全部定位在自旋四案**：
delete.html -4,129、forwarddelete.html -3,266、multitest.html -1,091、
insertparagraph.html -490——与 R5340（removeChild 对 sel 父 pending
解析子静默穿透）受害家族完全重合：分片二进制无 R5340，这些案在编辑
循环自旋到案墙钟被外部截断（部分变体无行）。M1 时代 setup 快败不进
命令执行面故无自旋、全分母记账。四案在新二进制（R5340 后）复跑归
R5340 收敛轮，预期收回分母缺口并出真实断言。

**误杀案收口（新二进制直跑）**：createlink-with-selecting-img 以
ZW_SCRIPT_TIMEOUT_MS=300000 + 420s 墙钟（R5342 同款下限）单案直跑：
**3 子测试 0P/3F/0T、数秒完成**（旧二进制 150s 零行——removeChild 自旋
即该案 >150s 的全部原因）。3 Fail 均为 createLink 未包裹 `<img>`
（inline/block/inline-block 三形态）——归 createLink 语义簇。该案为账内
唯一新二进制案（42 案为分片二进制），混二进制由下述零位移抽查背书。

**新二进制零位移抽查（收口义务履行）**：bold 全矩阵（nv=4，3,048 行）
新二进制（07:27:58，含 94eb9f0d2 + R5340 + R5341）复跑 =
**1,138P / 1,910F，与在途 s1 账逐位一致**——按本文件时点注记的判据，
账视为现行；R5340 影响面已由分母 delta 定位在自旋四案（上节），
R5341 load 形态缺陷不影响 run 面变体案（bold 零位移为证）。

**分片 + 续跑组合 bug（已修，本 goal 通道脚本）**：resume 跳过已完成案
时不消费 idx 槽位，`EDITING_CORPUS_RESUME=1` + `EDITING_CORPUS_SHARD`
组合下后续案 `idx % shard_n` 归属整体左移——13/14 完成态续跑单案实测
把 shard 2 名册错位成全名册扫描（s2 transcript 曾被灌入 5 个他分片案，
已从 transcript 剔除并从备份重建，分片 JSON 账不受影响）。修复：resume
跳过分支补 `idx++`。教训：分片成员资格由名册序位决定，任何跳过都必须
保序消费槽位。

**二进制时点注记（94eb9f0d2，收口必读）**：重跑窗口内并行流（html-semantics
域）落尾随静态 link 触发收口——共享面 part03.js/webview.rs/testharness.rs
被触碰，corpus 脚本与 wpt-data 未动（在途 bash 分片读取不受影响）。在途
分片与 other 面账面测的都是**落前二进制**；对 M2-r1 的对照仍同二进制可比
（唯一变量为预算口径）。收口时须以落后二进制抽查 1-2 代表案（如 bold 全
矩阵）核对计数零位移，零位移则账视为现行，有位移则受影响面按新二进制
重跑；分片退出前**禁止重建** target/release/zero-wpt-runner（cargo 以
rename 换新 inode，后继案静默切新二进制 → 混二进制账）。

## 四案簇 A 修复后复测终账（2026-10-30 收口）

上节「四案残余分母缺口（对 M1 -8,289 行）挂簇 A 修复后复测」收口。二进制
fc12a1ff8（R5340+R5341+R263b/R262b 预筛门）+ ZW_SCRIPT_TIMEOUT_MS=300000 +
按案自适应墙单案直跑（实测墙钟 delete 43min / forwarddelete 42min /
insertparagraph 31min / multitest 52min）。逐案账（变体级 P/F 见
[JSON 存档](2026-10-30-cluster-a-4case-retest.json)）：

| 案 | R5340 轮 | 本轮（簇 A 修复后） | Δ行 | 对 M1 缺口 |
|---|---|---|---:|---:|
| delete | 4,483r 2,014P/2,469F | 4,354r 2,061P/2,293F | -129 | -3,488 |
| forwarddelete | 4,582r 2,028P/2,554F | 5,398r 2,447P/2,951F | +816 | -2,093 |
| insertparagraph | 6,531r 3,423P/3,108F | 7,195r 3,774P/3,421F | +664 | **0（全分母）** |
| multitest | 8,735r 5,109P/3,626F | 9,497r 5,653P/3,844F | +762 | -595 |
| 合计 | 24,331r | **26,444r 13,935P/12,509F** | +2,113 | **-8,289→-6,176** |

结构性结论：

1. **insertparagraph/multitest 全分母收回**：script timeout 1→0 / 1→0，
   全部变体预算内完成（insertparagraph ?7001-last 195 行、multitest
   ?9001-last 497 行均自然收尾）——证实 R5340 轮归因：两案缺口主体是
   R5340 分支开销 × 300s 预算的吞吐截断，簇 A 修复购回吞吐即全额恢复。
2. **forwarddelete 部分恢复（+816）**：frontier 推进至重区（4001-5000
   @852/1000、5001-6000 @544/1000），仍 4 变体 300s 中止。
3. **delete 无实质恢复（-129）**：重区实测 ~2.3s/行（完成区 <0.3s/行），
   中止集 {4001,5001,6001,7001}。瓶颈已证非 range 注册表——R263b/R262b
   pass1 预筛为 O(R≤8192) 廉判、命中时增量可忽略；残余归簇 A fix
   evidence §7 分布式 glue 桶（removeChild trap ~390µs/次、s111 臂、
   CE loop、execCommand glue、逐断言 setup parse）。M1 全分母系 setup
   快败产物（断言快速 Fail 不做真功），真执行下全分母回收须重区每行
   成本降约一个量级。
4. **跨截断集不做翻转推断**（与上节同纪律）：delete 对 R5340 轮 -129
   为中止集位移（两轮中止变体集合不同），完成区确定性由 bold 全矩阵
   零位移先例 + 本轮完成变体零 timeout 背书。
5. **run 面现行账**（43 案分片账的四案位替换为本轮）：
   **41,171P / 49,318F / 90,489 行**，对 M1 96,663 缺口 **-6,174**。

## 运行入口

`make testharness-editing-run` / `make testharness-editing-other`（分片
`EDITING_CORPUS_SHARD=k/N`，k 从 0 起；续跑 `EDITING_CORPUS_RESUME=1`；
并行分片须各设 `EDITING_CORPUS_TRANSCRIPT`）。探针/复跑 log 为 /tmp 快照
不落档，关键数字已入本文件与 JSON 存档。
