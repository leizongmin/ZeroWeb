# ip C2/F/I6 方向性壳携刀收口——16 键 + a-outer 精化 2 键（v1 ABORT → 精化 → v2 ALL-DONE）

- 日期：2026-11-15；刀体 `fb81665dd`（16 键三臂 + part118 16 钉）→ 精化 `b64549664`（最外收壳之上扫 a 清链 + ⑰⑱ 双裸垫钉，18 钉全绿）→ pull --rebase 并入 f389ee645（g44 色标/style-attr 写面拒收，crates/ 入站唯一代码提交，editing 语料 innerHTML 建树不走 setAttribute 写面）内容恒等重放 `f6a70ef86` 已推 origin。fixdiff crates/ md5 `793027a5d54790a1bad87808059b2cf1`（rebase 70689b3ce 上内容恒等）。
- 前置普查：[c2fi6-shell-carry-census-16f](2026-11-15-c2fi6-shell-carry-census-16f.md)（16 键 got/exp 全解析）。基线：`ipm3-b-v6-ip.log`（B 刀 v6 终判，ip 残 44F）。

## 刀体三臂（part01f.js，装配点实证由探针定谳）

- **左臂**（offset-0 左空块垫点，v6 日志 16 键 brCreated=1 栈行实录）：垫底 br 新建时携 caret 祖先样式壳链（fmt+span 逐层 cloneNode(false) 保 attrs、id 剥除、内→外正序包裹保输入链方向）；携链不带 v3.2 重开门。
- **右臂-弃**（extract 携带链归一，spine 含 `a`）：右空块 spine 全弃裸垫（F 6 键：a-外/b-外两方向同弃，语料 exp 备选两形右块均裸）。
- **右臂-扩**（纯样式浅链）：spine 无 a 且浅于拆分位祖先链时按 tag+attrs 前缀等值扩全链重建（I6 2 键双 span 半携转全携）。

## v1 批 ABORT 定谳（a-outer 左垫过携）

- `batch-ipm3-c2fi6-gate-v1` @fb81665dd（bin 3e4ef736）03:17 L1 judge：**F2P=16 恰等（缺=0 多=0）、residF=28 恰等，唯 P2F=2**——`<p><a href=foo><b>[]foo</b></a></p>` / `<div><a href=foo><b>[]foo</b></a></div>`（:1072/:1077，v6 期裸垫即绿的通行键，普查只解剖 44F 失败键故漏）。
- 根因：左臂 skip-a 收链把 b 携进左垫，语料 exp 裸垫 `<p><br></p>`。
- 精化（b64549664）：携集精化门——收集链最外层壳之上扫至 container，命中 `a` 即清链裸垫。运行时分派实证：:1072/:1077（a-外-b-内）b 之上即 a → 清空裸垫 ✓；:1092/:1097（b-外-a-内，exp 左垫**携 b** `<p><b><br></b></p>`）拆分后 a 已随内容移入右块、b 之上无 a → 携 b 保持 ✓——同一道门对两朝向给出语料两侧正确答案（v2 批 8 个 a 族键全绿实证）。part118 增 ⑰⑱ 双裸垫钉（p/div 宿主）锁语义，18 钉全绿。

## v2 门禁批 ALL-DONE 五腿全绿（2026-11-15T04:29:01+0800）

`batch-ipm3-c2fi6-gate-v2` @f6a70ef86 bin `d6169b75d7d3503f8977c29a2871ccf3` ksig=10/10：

| 腿 | 判据 | 结果 |
|---|---|---|
| L1 ip fresh | vs 44F 基线 F2P=16 恰等 / P2F=0 / lost=0 / newF=0 / residF=28 恰等 | rows 7195 恒一，**F 44→28**，wall=949s ✓ |
| L2 ab | 10092/0/0 严格 + judge-dps vs e1-ab | P=10092 F=0 T=0 wall=271s ✓ |
| L3 mt | vs e1-mt 四零 + 白名单 | rows 10092 恒一 F 2→2 全零漂移 residF=2（X37 旧账）✓ |
| L4 enginelib | 名册零新增 | **4009 passed / 0 failed**（3984+16+2+7 恒一对账）✓ |
| L5 ilb | vs e1-ilb 33F 逐位恒等 | rows 1691 F 33→33 全恒等＝右臂插入点 ilb 零暴露 ✓ |

## 收口与挂账

- ip 残 44F→**28F**，族图闭合：C1 容器尾序反转 8 ＞ qV-dPS 读面 8 ＞ C5 2＋C6 2＋C3 2＋I4 2＋I5 1＋C4 越界 3。下一刀面=C1（census 次序最大余簇）。
- 监视器教训：批腿头静态文本含「新增=ABORT」字样，grep "ABORT" 全文判终态的观察器会假阳性——终态判定应锚行首 `^ABORT` 或 `ALL-DONE` 精确标记。
- 双执行体协调：v1 中止诊断、精化修复与 v2 发射由同 session 并行写者轮完成（commit b64549664 注释引 v1 批 P2F 即证据），本流按核验方协议让位观察（协调信道 `.acceptance/dps/ipm3-c2fi6-verifier-note.md`），批终态后接管 docs 收尾。
