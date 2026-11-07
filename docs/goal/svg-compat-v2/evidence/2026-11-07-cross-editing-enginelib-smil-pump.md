# [editing 流移交] SMIL M1 泵饿死 _defer 预算——enginelib 名册 4F→6F 两 settle 红归因 M1（2026-11-07 12:35）

来源：editing-compat knife10c 推送树复绑批 enginelib 名册复核发现 4F→6F，双流独立
bracket + 核验方机制链静态闭环。完整档案（机制链 file:line、时间线、复核配方、附带
r3254 发现）：
`.acceptance/dps/VERIFY-NOTE-enginelib-6f-m1-smil-pump-defer-starvation.md`

## 结论速览

- **归因**：015172475（M1）。双流 bracket 一致——M1 树 6F 红在案（knife10c rebind
  11:56 + 核验方 detached 015172475 纯 M1 7F 含 r3254）、49abc3e63（pre-M1）4F 绿、
  49abc3e63+knife10 刀体（无 SMIL）绿。R5447/刀体洗清。
- **两红**：`img_runtime_deferred_settle_r5275` (c2)、`svg_img_settle_density_r5277`
  (h1)——同为 `_defer` 延迟交付饿死（load 计数 0≠1）。
- **机制**：part06:6865 load 挂钩 `_zwSmilDocStart` → part09:602 `_scheduleTick(5)`
  **零动画元素也无条件武装** → part09:570-591 `_pumpTick` 0 元素仍 50ms 保底自续 →
  enginelib 裸沙箱无宿主定时器，setTimeout 走 part01:7377 fallback `_defer` 微任务
  链 → 单次 execute 内烧穿 `_deferBudget=256`（part01:3694，注释防线原文即「避免
  setTimeout 轮询在 checkpoint 中无限链式调度」）→ 裸 execute 不经 `__zw_begin_script`
  重置（part01:3734）→ 预算停 0，后续 `_defer` 静默灭（part01:3697）。
- **mt runner 面零扰**：knife10c mt 漂移中性 judge F2P=0/P2F=0/lost=0 VERDICT OK
  （有宿主定时器路径无键级行为差）。生产面遗留：load 后每页常驻 50ms 真定时器轮询
  （零 SMIL 内容也转永不停，无界 CPU 税）。
- **修法建议**（svg 流裁量）：`_pumpTick` 尾 `_scanDoc(doc)` 0 元素时不再自续，唤醒面
  load/beginElement/`_wake`（元素插入感知可走 MO——part01:8132 注 MO flush 绕过
  `_deferBudget`）。复核配方三步见完整档案。
- **附带**：`test_insert_paragraph_split_r3254_m3_slice5` M1 引入确定性翻红（p 块
  mutation 序）、knife10 part01 改动顺带治愈、现役树全绿——机制未定谳留档不阻塞；
  另有 knife11 现役轮预警（该测断言 div、第十一刀 ① 翻 p 后将按设计复红，须同提交
  改期望），与 svg 流无涉，见完整档案末节。

## 时间线

M1 11:29:29 → knife10c rebind enginelib 11:56:36（c9b7a2416，6F）→ mt 腿
11:57-12:27（19,854P/330F 恒一，双 judge VERDICT OK）→ 核验方 bracket 12:28
（detached 015172475）→ 冻结二进制复跑 12:37（三红确定性）→ 本档案 12:37。
