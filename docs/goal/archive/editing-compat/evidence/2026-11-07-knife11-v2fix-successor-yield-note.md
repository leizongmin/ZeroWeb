# knife11 v2 fix-gate rally 后继轮让位核验记录（2026-11-07 16:5x）

rally 驱动后继轮（16:46 起，前轮 d0476931 于 16:41 rate-limit 终止）开工归因：

- fix-gate 批（16:45:49 起飞，shim 组合 md5 57db60a18b5c89100a155cce2ffc9e3d
  build 时锚，marker 双检全中）归属施工轮（root-cause 笔记 §4 作者，
  d081bc66 谱系，part01f 16:34:42 施工为证）。本轮**让位不动树**，转被动
  核验方：批链静默前不 pull、不 commit、不 launch 批，防双执行体撞车。
- part01f.js/part06.js 修复字节已由本轮先行提交为 **17c4a4bde**
  （lei-pre-commit-guard PASS；字节=批锚树态，SHIM-DRIFT 与 marker 断言
  不受影响）——施工轮落地提交按此收敛为 docs+evidence+goal doc 尾链，
  代码无需重提。
- 让位依据：census 笔记 16:49 后新 mtime（v2b-verdict/probe-e 脚本 +
  census 笔记勘误节追加）证明谱系轮活跃；共享树收敛（pull --rebase，
  behind 10）按既定纪律**待批链静默后**由落地执行体进行。
- 本轮以只读 watcher 监视 fix-gate→supp→(v2b 若起飞) 全链；出数后按
  红线（mt F2P=18 恒一 + P2F=0/post-only=0/lost=0/rows 恒一；ip 腿
  P2F=0 复位；iol/iul/battery 恒一；enginelib 名册恒一）独立判读，
  回执附于本笔记。若谱系轮在批链完成后长时静默（无 commit 无笔记），
  本轮按 root-cause §4 回执代跑落地尾链（代跑前重核 flock/git 状态）。

## 判读回执（后继轮，2026-11-07 19:0x，fix-gate 出数后首个判读方）

背景：批链脱离会话存活至 17:45 完成（fix-gate 16:45:49→17:30:52、supp
17:30:52→17:43:57、probe-e 17:43:57→17:45:17，三批 shim 锚 57db60a1
build 时同值 + marker 复检全中）；施工轮与本轮前驱双双 rate-limit 终止
（17:18 前），批后 80 分钟无判读无 commit（.rally/events 与 evidence
mtime 核对，19:04 重置后仅本后继轮一个执行体），按本笔记预告条款由
后继轮代跑判读与落地尾链（flock 空闲、part0*.js 树态=17c4a4bde 已核）。

- **fix-gate 红线全绿**：ip 单腿 rows 7195 恒一，F 210→**208**，
  shared F2P=0/**P2F=0**/post-only 0/lost 0 VERDICT OK——P2F×2 td 宿主
  复位恰如 root-cause 笔记处方预期（dps-unset 门生效）；mt 全分母 rows
  10092 恒一，F2P=**18** 恒一 / P2F=0 / post-only 0 / lost 0 VERDICT
  OK（F 143→125，Δ=18；raw 19936P/248F）；enginelib **3794P/6F** 名册
  逐名恒一（6-red 外债档：fetch/iframe/ws×2/img/svg，ROSTER OK）。
- **supp 恒一红线全绿**（head=17c4a4bde，shim md5 同锚）：iol
  1759P/2F、iul 2061P/4F rows 恒一零翻转 VERDICT OK；ambient 七腿电池
  F=0（880/420/1104/552/3456/2607/1455）零位移。
- **probe-e 220P/8F 零 UNEXPECTED**：8 Fail 行全 496/499 族（CASE 行 +
  DUMP + STAGE-D ×2 + 语料 de]f/def 两行）——DC-4 记账域，与
  arc-verify 笔记「残红恰 {496,499}」判读一致。
- **v2b-verdict 起飞条件不成立**：fix-gate（mt+ip+enginelib）+ supp
  （iol/iul+battery）+ probe 已覆盖本笔记全红线，全量 verdict 重跑冗余，
  脚本归档不跑。
- **换基传递**：落地前 pull --rebase（19:14，behind 13：SMIL
  B5/B6a/B6b、s330/s332、R5449、shadow-dom M1、cssom、站点 PR #281
  并入），17c4a4bde → **c5a28c114**；刀体 diff（part01f+part06）对账
  逐字节恒等（唯一差异 = part06 hunk 头行偏移 4753→4776，incoming
  +23 行所致；part01f 远端零 hunk）+ 基座增量各自落基门禁独立验证
  （各 commit 自带零回归证据）⇒ 57db60a1 锚 VERDICT 传递随附；四文件
  recipe md5 随基座漂移为 3b7a64a5（内容绑定锚失效属预期，绑定句不再
  声称现树=57db60a1），新锚重钉见 rebase-gate 批
  （batch-knife11-v2-rebase-gate.sh：marker + probe 复跑 + enginelib
  名册换基后复核，结果见下节补记）。
- **落地尾链**：v2 证据集（typed-fate/ab-yield/arc-verify/landing-verify/
  双 verdict-verify/本笔记 7 文件）+ goal doc v1.69（含 72da4f974 落地
  提交正文方向勘误 per landing-verify §2）由后继轮提交推送，lark 通知。

## rebase-gate 补记（19:15:02→19:17:44，新锚 3b7a64a5 build-anchored）

- marker 双检全中（paren old=0/new=3、noparen 1/1，v2 四标记 tcap=7/
  dive=4/wraptag=4/delead=1）——刀体在换基二进制内字节面在位。
- probe 复跑 **220P/8F**，8 Fail 行与换基前逐案恒一（496/499 ×
  CASE/DUMP/STAGE-D + 语料 de]f/def 两行）——行为面换基恒一。
- enginelib **3798P/6F** 名册逐名恒一（ROSTER OK；P 3794→3798 系基座
  增量新增测试，6-red 名单零变动）。
- mt/ip 全腿不重跑：刀体 diff 恒等 + 基座增量各自落基门禁 +
  marker/probe/名册三面复钉，按 anchor-rebase 笔记既立先例判 VERDICT
  传递成立；57db60a1 锚判读域=16:45 批锚树（=17c4a4bde 提交字节），
  现树绑定以 3b7a64a5 为准。
