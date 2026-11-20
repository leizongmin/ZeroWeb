# 第十刀并发轮协调备忘（399090bd → 484b0799，2026-11-07 10:56）

写者角色让位：本会话（399090bd，压缩续轮，knife10 刀体与 mt 批的原执行者）停止写树、转核验方；
484b0799 为现役写者。控制面 v1.67（本会话起稿 + F2P 复跑定谳句 raw 147=77+70 精化）、
fix.md 现版、roster 现版均照单接受，不回退不改写。

## 提交前必须处理的核验发现

**shim md5 断链（红线）**：现树 part01*/part06 md5 = `b5323c701d8be72fb806fccfa21a70b2`，
mt 批锚定（batch-knife10-mt.log 10:07 build-anchored，VERDICT OK 10:39:46 判读域）=
`09cf49724835c41cca419d86aebad4bf`。diff 行数一致（part01 +68/-1、part06 +20）且刀体
marker 在位，但字节不同（10:54:11 恢复动作引入）——**现树 VERDICT OK 判读不绑定**。
提交前二者其一：

- (a) 恢复至 09cf 字节态（10:54:11 前的本会话树态；对照来源：本会话 transcript 或
  target/release/zero-wpt-runner 批二进制内嵌 shim），直接引用批判读；
- (b) 以现树复跑 mt 终判（flock /tmp/zw-dps/batch.lock + judge vs knife9-mt-post.log，
  红线 P2F=0/lost=0/rows 恒一），批日志更新后落账。

次要不一致：fix.md 验证链 F2P 键级对账句（raw last-wins 81 = 70 + 11 口径差）与控制面
复跑定谳句（raw 147 = 77 + 70 跨文件域）数字体系不同，收口时统一或注明域差
（ab 变体域 vs multitest.html?N-M 主语料窗口族）。

## 资产索引（本会话产出，均在盘）

- mt 终判：`.acceptance/dps/knife10-mt-post.log` + `batch-knife10-mt.log`（VERDICT OK，
  10:39:46）+ 批脚本 `batch-knife10-mt.sh`；census pickle `knife10-census.pkl`。
- make test 红面：`batch-knife10-maketest.log`（10:47:49，3795P/4F = 名册外债四红，零新增红）。
- probe（gitignored 不入库）：`tests/wpt-runner/wpt-data/editing/run/zz-probe-knife10*.html`，
  判读日志 `knife10-slice2/16.log`。
- stash@{0} 评估结论：patch 文本过期（part01f.js:428 context 漂移）apply 失败、语义被
  `_zwDelKeepShellOf` 重构与 784e52047 吸收，留 stash 不销毁。

## 399090bd 核验附录（2026-11-07 11:05）

1. **刀体语义完整性**：现树三修逐字在位——part06 尾簇 33 让位门（_sel57 walk + return
   false）、part01 `_rrKeep` 门（:9304）、`_zwSyncCaretLost` 链（:9205/:9275/:9304）与批
   锚定语义一致；:9603 注释行仍为 6 空格 09cf 态（批前缩进回滚未被回退）。字节差定位无
   09cf 参照不可得（见 3），「白噪字节差」仅为假设。
2. **gates 批核验**（batch-knife10-gates.log，10:56:43-11:00:46 ALL-DONE）：FMT OK；
   enginelib 4F 逐名=新名册档（enginelib-red-roster-2026-11-07-knife10.txt）——其 DRIFT
   判定系脚本对照旧九红档（EXPECT 取 …2026-11-07.txt）所致，实际零新红；make test rc=2
   四红等同外债（与 10:45 enginelib 批、10:47 maketest 批三代一致）。注：gates enginelib
   腿编译窗与 3 提交 pull（~10:57-59 落树）重叠，两代结果一致故不要求重跑，记录备查。
3. **md5 断链现状**：批二进制 target/release/zero-wpt-runner 已被清理——(a) 恢复路的提取
   源失效（重建只能走本会话 transcript 重放，重路径）；(b) mt 复跑**须待 shadow-m3 gethtml
   24G scope runner（本 clone PID 340702，TIME_LIMIT 1800s）退出后再起**——机器负载会推高
   ?9001-last 页 300s 预算慢化截断伪影（ledger 已记录的假红发生器），负载下复跑的判读
   不可用作绑定证据。
4. **复活版 doc1（10:58）三处事实修正建议**：
   - 「lost 33 + new 12 全落 ?9001-last 尾页超时截断边界」→ 复跑定谳：lost 33 全部为
     multitest-ab.html 族 delete 表格链未跑键，new 12 为 ?9001-last 窗界（11 Pass + 1
     Fail postF）；行差 -21 = -33 ab + 12 窗界。
   - 「enginelib 九红名册恒一（ROSTER OK）」→ 现树 9→4（五 CSSOM 红随 T45/sr 合入转绿，
     4F=新名册档逐名；10:45/10:56 两代一致）——以 fix.md 9→4 表述为准。
   - 「77 = 11 值/样式 × 7 结构命令目标子形 100% 全翻」网格口径成立且比 fix.md 对账段更
     精确，建议 fix.md 吸收（两档并存亦可）；「judge F2P=70 取 mt 侧 last-wins」为未验证
     解释，建议标（解释性）或核 judge-dps.py 归一化口径后再断言。
5. gates 脚本小疵（后续轮顺手修）：EXPECT 应对照新档 enginelib-red-roster-2026-11-07-knife10.txt。
