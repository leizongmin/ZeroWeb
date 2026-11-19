# 第十刀推送树复绑与名册漂移（knife10c，2026-11-07 11:56-12:27）

## 背景：推送域两层漂移

knife10b 复跑把 mt VERDICT 绑定到 b5323c70@49abc3e63 暂存态后，推送链出现两层
跨流漂移（主流程 d0b035cb9 由并行会话 11:46 提交、11:49 补状态行 ea35f6e9c、
12:2x 推送 + lark 落账 4ac6d8787）：

1. **SMIL 015172475**（svg-compat-v2 M1，推送序在 knife10 提交之前入历史）：
   part06 +5（文档 load 钩子 `_zwSmilDocStart`）/ part03 +23（animVal 只读
   consult 入 attribute getter）/ part09 +630（SMIL API 单点）/ js_dom_bridge.rs
   +14。shim 组合 md5 b5323c70→bfc57a5f。
2. **knife10 提交基座**：d0b035cb9 的 parent 已含 SMIL+R5447，提交信息
   「head=49abc3e63」指复跑锚定域而非提交树——字节断链由本批收口。

刀体完整性独立核验（49abc3e63→HEAD diff）：part01.js 68 行改动 = 复跑锚定刀体
原样；part06.js 25 行 = 刀体 20 + SMIL 5；marker 单拷贝（_rrKeep×3 行 /
_zwSyncCaretLost×1 / _sel57×1 源码面）。

## knife10c 复绑批（batch-knife10c-rebind.log，11:56:23 起跑，12:27:32 ALL-DONE）

head=c9b7a2416，enginelib 腿先行 → release 构建 35.7s → 字节锚断言
bfc57a5fd27239502a03f89e1fd05711 过 + dbg-absent 双检 0 + marker 核验
（_rrKeep×3/_syncCaret×9/_zwSyncCaretLost×3/_sel57×2）→ mt 全分母双判读：

- **红线 vs knife9 基线 VERDICT OK**：rows 10092→10092 / P 9856→9926 / F 236→166
  / F2P=70 / P2F=0 / post-only 0 / lost 0（raw 19,854P/330F，与 knife10b 逐位一致）。
- **漂移中性 vs knife10b VERDICT OK**：key 级恒等（P/F 全恒一）——SMIL 域漂移
  （part06 load 钩子 + part03 animVal consult）对编辑语料语义零差，预测命中。
- mt 红线 VERDICT 自此绑定推送树组合字节态（bfc57a5f@c9b7a2416）。其后
  83539714c（SMIL seek B3）再入漂移层，由 knife11 批 mt 判读覆盖。

## enginelib 名册漂移 4→6 与 checkout A/B 定谳

knife10c enginelib 腿（c9b7a2416，SMIL+R5447+刀体组合态）：**3793P/6F**——
对 4 红档多出 2 红：

- `img_runtime_deferred_settle_r5275`（part85.rs:200，c2「load 事件后续任务派发」
  left 0 / right 1）
- `svg_img_settle_density_r5277`（part89.rs:387，h1「load 不重派」left
  `300/100/0/...` / right `300/100/1/...`）

**归因（checkout A/B，detached 定向复跑）**：015172475（SMIL 入 / R5447 未入）
两测试即红（0 passed / 2 failed / 3797 filtered）；015172475^ 引擎态 ≈49abc3e63
（中间仅 docs 提交）上两测试通过（11:41 roster 批 3795P/4F 同证）——**引入者唯
SMIL 015172475**；R5447（table cell content-box）与 knife10 刀体均洗清。两红
同征（load 后续任务派发计数 1→0），SMIL 面候选机制：part06:6865 文档 load 钩子
/ part03 getter 热路径 consult / part09 全局单点——机制定位与修复权移交 svg 流
（run-rules §9 共享面纪律：不硬解）。新名册档
[enginelib-red-roster-2026-11-07-knife10c.txt](enginelib-red-roster-2026-11-07-knife10c.txt)。

## 并发事件记档：detached checkout 撞车与 merge-carry 恢复

knife10c 批退出后本会话 detached 到 015172475 做定向复跑（~1 分钟窗），期间
knife11 现役轮（认领 bfef95693）开始向工作区写入刀体（part01/part01e/part01f/
part06 +32/-8）——编辑落在 detached 基座上（该基座无 knife10 刀体）。恢复：
`git checkout -m main` merge-carry，三文件零冲突合并回 c9b7a2416→4ac6d8787 基座；
合并后核验：knife10 body 三件套在位（_rrKeep/_zwSyncCaretLost/_sel57），knife11
编辑全量保留为未暂存改动。**致 knife11 现役轮：基座已复原到 main（4ac6d8787），
贵轮编辑已按正确基线保留，继续施工前请重新 diff 核对工作区。**

## 资产索引

- 复绑批：`.acceptance/dps/batch-knife10c-rebind.sh` + `batch-knife10c-rebind.log`
  + `knife10c-mt-post.log` + `knife10c-vs-knife9.txt` / `knife10c-vs-knife10b.txt`
  （judge 输出，仅红面时写）。
- A/B 腿：detached 015172475 定向两测试 run（0 passed/2 failed）。
- knife11 普查准备：`.acceptance/dps/knife11-census-prep.txt`（166F judge 口径
  分桶：ip 触及 48 = 链 36 + compare 12、ihtml 12、ilb 10、createlink 30，与控制
  面账目吻合；knife11 批出数后以新基线重跑分拣）。
