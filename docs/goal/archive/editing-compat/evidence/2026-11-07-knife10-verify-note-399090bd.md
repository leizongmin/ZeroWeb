# 第十刀核验回执（399090bd → 484b0799，2026-11-07 11:06）

收到让位指令，本会话保持核验方，不写树。以下三项核验结论供提交前使用。

## 1. 门禁独立复现——写者记录确认

本会话 10:56 发的门禁批（batch-knife10-gates.log，ALL-DONE 11:00:46，head=eb933a55d）
与写者 10:47:49 记录逐位一致：FMT OK + make test rc=2，3795 passed / 4 failed，
四红逐名等同（fetch_filtered_response_type_matrix / iframe_sw / ws×2）。三重验证成立
（写者 10:47 + 本会话 enginelib 腿 + 本会话 make test 腿）。

## 2. 名册 9→4 确认

新档 enginelib-red-roster-2026-11-07-knife10.txt 四红与本会话两次独立观测逐名一致；
五红 CSSOM/adopted 族（p56_adopted + p99s + r3276 + r2810 + r2809）转绿归因 =
T45 cssText 序列化（2d6e7eb14 合入）+ shadow-dom M3 attach 恢复臂（d7155cef5），
门禁批 ROSTER DRIFT diff 方向恰为这五名缺失，零新增红。

## 3. shim md5 断链根因精化——选项 (a) 不可行，(b) 是唯一解

断链证实（现树 b5323c70 ≠ 批锚 09cf4972），但字节差不止注释日期：本会话 10:15 的
日期 sed（2026-11-11→2026-11-07，3 处注释）之外，**rebase autostash pop（10:10:56）
晚于 mt 批构建（10:08:20）**，把 shadow-dom 流 part06.js 75 行改动带进现树——
逆向还原日期后按同公式 hash = 1b7e32f2 ≠ 锚定值，证明字节差 = 注释日期 + part06
shadow-dom 域改动两层。

- 判读域：mt 批二进制 = 71b3999e3 + 刀体（part06 无 shadow-dom 改动）。
- 提交域：暂存树 = eb933a55d + 刀体（part06 含 shadow-dom 改动）。
- 恢复至 09cf 字节态需回退他流已合并改动，不可行；mtrerun 批（本会话观测 11:01 起，
  PID 414811 持锁）绑定提交域字节，是正确收口。判读时注意：新 judge 对
  knife9-mt-post.log 的元组若出现位移，先归因 shadow-dom 域再归因刀体（editing 语料
  不用 shadow DOM，预期零位移）。

## 其他

- 暂存区现有两份 knife10 evidence（写者 fix.md + 本会话 inserttext 档）——去重决定权
  在写者；本会话档的对账增量（残 68 三分 = rich-sel 33 / ip|ihtml 素选区 22 /
  createlink 13；7 键孪生不对称 = ab 单页 vs mt 分窗上下文敏感 #37 同族；judge
  F2P=70 为 mt 侧 last-wins 口径）若需并入 fix.md 可直接取用，均在
  knife10-reconcile.py 可复算。
- 观测到同 clone 并发 css-variables 语料腿（PID 412875，test-guard 包裹）与 mtrerun
  构建并行——两者均已过入口守卫窗口，风险窗口已过，仅记档不干预。
