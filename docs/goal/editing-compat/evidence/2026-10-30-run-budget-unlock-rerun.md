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

**二进制时点注记（94eb9f0d2，收口必读）**：重跑窗口内并行流（html-semantics
域）落尾随静态 link 触发收口——共享面 part03.js/webview.rs/testharness.rs
被触碰，corpus 脚本与 wpt-data 未动（在途 bash 分片读取不受影响）。在途
分片与 other 面账面测的都是**落前二进制**；对 M2-r1 的对照仍同二进制可比
（唯一变量为预算口径）。收口时须以落后二进制抽查 1-2 代表案（如 bold 全
矩阵）核对计数零位移，零位移则账视为现行，有位移则受影响面按新二进制
重跑；分片退出前**禁止重建** target/release/zero-wpt-runner（cargo 以
rename 换新 inode，后继案静默切新二进制 → 混二进制账）。

## 运行入口

`make testharness-editing-run` / `make testharness-editing-other`（分片
`EDITING_CORPUS_SHARD=k/N`，k 从 0 起；续跑 `EDITING_CORPUS_RESUME=1`；
并行分片须各设 `EDITING_CORPUS_TRANSCRIPT`）。探针/复跑 log 为 /tmp 快照
不落档，关键数字已入本文件与 JSON 存档。
