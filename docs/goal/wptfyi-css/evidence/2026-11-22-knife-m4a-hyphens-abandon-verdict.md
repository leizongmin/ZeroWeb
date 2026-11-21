# 刀 M4-a hyphens 初始值 v1 弃置笔——witness 剥除＋判读改判（2026-11-22 06:4x，witness 接管实例）

## 判定 TOTAL：**弃置（v1 face 目标错位，不入库）**——gained=0 lost=0 **P2F=0**（四处全谱零位移）

> 本笔含一次判读改判：本实例首判 P2F=13 系**误计**（详见下节改判）；
> 权威 v1 判定笔为同窗独立判读实例所出
> [2026-11-22-knife-m4a-hyphens-initial-v1-verdict.md](2026-11-22-knife-m4a-hyphens-initial-v1-verdict.md)，
> 两笔判定 TOTAL 收敛一致（弃置＋零位移），本笔补 witness 剥除与误计法证。

## 判读改判（P2F=13 → P2F=0）

- 首判报 P2F=13（animations 5＋transitions 8），归因 flake 候选待 solo。
- 同数据双快照字节复核：`snap-knife-m4a-0631/` 与
  `snap-knife-m4a-hyphens-0632/` 八腿 JSON **md5 全恒等**（同源数据）。
- 逐案 status 重判：**P2F=0、F2P=0**。13 行实为 **b1 报告截断缺案**
  （missing_b1=12＋missing_b0=1：event-dispatch.tentative 7、
  event-order.tentative 3、animate-with-color-mix 1、
  display-none-dont-cancel 双侧各 1）——**missing≠Fail**，不入账；
  与「报告完备性不对称=phantom gain」判例同形。首判脚本把缺案计成
  翻红＝判读器缺陷，非电池回归。solo A/A 判别因此**不再需要**。
- 钉测 DOA 法证（v1 判定笔揭出，本笔采认）：`test_hyphens_initial_manual`
  首执 panic＝`ComputedStyle::default().hyphens` 仍 None——初始值真实源为
  `default_impl.rs:426` 手工 `impl Default for ComputedStyle`；registry
  `initial_value` 臂仅服务 `hyphens: initial` 声明解析，两源独立。
  v1 单改 registry ⇒ gCS 无声明读数恒 None ⇒ observed 回声不变。
  **主判据 B 分支（observed 未变）由此定谳＝registry 默认臂非回声源**。

## 电池档案（与 v1 判定笔同源，互证）

- unit `zw-knife-m4a-hyphens-initial`（甲 fork 05:58 发射，甲 session EXIT
  后零收账迹象，06:31 起 witness 接管）；EXIT rc=0 06:31:03，
  八腿（values/text/animations/transitions × b0/b1）全 rc=0。
- face v1：md5 `0564ed63647c8d8964929309dc12dc69`（2 文件：registry.rs
  None→Manual＋apply_coverage.rs 钉测）。
- bins：b0 `4ebcfe8ee221699e42acac7fbcdca363` / b1 `556ee2fca88471886997484c5d8a8a98`
  （md5 异=M2 门形）；build 05:58/05:59 早于 D1 首写 face 窗＝零污染。
- 快照：`.acceptance/wptfyi-css/snap-knife-m4a-0631/`（独立判读实例）＋
  `snap-knife-m4a-hyphens-0632/`（本实例），字节恒等。

## witness 剥除与 v2 衔接

- 本实例按 claim 预注册弃置纪律执行 face 剥除：`git apply -R` v1 patch
  （md5 恒等＋干跑绿后执行），registry.rs 回 None、apply_coverage.rs
  哨兵清零，为 v2 face 铺树（strip 时点在 v2 face 应用之前）。
- v2 face：`knife-m4a-hyphens-initial-face-v2.patch` md5
  `4a877c80e04308f2a8ae0f48c13981c5`（5 文件：registry.rs 承 v1＋
  **default_impl.rs None→Manual 主修**＋types.rs 枚举 doc＋extended.rs
  既有测期望更新＋apply_coverage.rs 钉测），r2 电池
  `zw-knife-m4a-hyphens-initial-r2` 06:42 发射在飞（serialize 门 06:42:37
  过，dev 测 `test_hyphens*` 5 passed＝DOA 解消）。
- v1 face 形不入库（零增益＋错位目标）；tally 全程不动。

## 处置与后续

1. M4-a v1 弃置入册（本笔＋v1 判定笔双存档）；主判据证伪价值已兑现
   （回声源定位到 Default impl）。
2. 后续挂 M4-a v2 判读（r2 电池 EXIT 后按 claim 判据收账）；
   M4-c「none 真来源」刀随 v2 主修大概率闭合，待 v2 结果定去留。
3. 判读器教训入账：逐案 diff 必须 status 三态（P/F/missing）分列，
   missing 不得折算翻红；双侧缺案页（display-none 形）单列挂起。
