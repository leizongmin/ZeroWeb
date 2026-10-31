# 编辑命令兼容 — editing/run + editing/other

**版本**: v1.32
**日期**: 2026-10-27（立项，coverage 对账驱动）；2026-10-30（M1 收口）；2026-10-30（R5331 + M2 首轮）；2026-10-30（M2 次轮双窗折叠：预算归因 + other 面复跑收口；自旋簇三分类 + R5340）；2026-10-30（M2 次轮收环：缺陷 #2 已修 R5341 + other 面墙钟 R5342，三例 Timeout 全收口）；2026-10-30（预算窗 run 面全分母重跑收口 39,885P/47,804F/0T）；2026-10-30（簇 A 归因收口：填充/物化逐节点 JS glue 定量，修复 scope 定案）；2026-10-30（自旋四案新二进制复跑收口：零翻转，+687 行，残余挂簇 A）；2026-10-30（簇 A 修复落地：R263b/R262b 预筛门，fill/clone 3.6-9.3×）；2026-10-30（自旋四案簇 A 修复后复测收口：+2,113 行，缺口 -8,289→-6,176，insertparagraph/multitest 全分母）；2026-10-30（簇 A trap 层四笔落地：globalThis miss + R155 反查 + 键控根验证 + innerHTML CE 门，与 R263b/R262b 合并，S5 317ms）；2026-10-30（white-space join 实现落地+语义证明：探针直调三形态逐字命中语料期望；0P 根因=语料管线阻塞 D1/D2/D3 非 join 语义，簇序 scoping 修正）；2026-10-30（D1/D2/D3 落地 + 第 4 应用路径漏换代通知补账 + R382 条目级 ghost 兜底，join 语料 0P/7,518F→603P/6,915F）；2026-10-31（join 语料收口：双 wrapper 家族视图滞后根因 + plain 寻址补全，0P→3,108P/4,410F 零回归，与并行流 D1/D2/D3 撞车合并——D1 写通道统一 sel-path wire + caret 两级重锚）；2026-10-31（撞车仲裁终验：一级重锚让位 + 回灌 stamping 旁路两门落地，join 语料对 fix7 7,518 案零漂移恢复）；2026-10-31（行内壳变体收敛：ws 摘离快照 + per-run 兄弟壳 + 行内 run 升位 + ended 出参，join 三案 3,108P→5,698P/1,820F 零回归）；2026-10-31（join paragraphs 簇清零：skipWs 方向反转根因 + wire 探针定位，5,698P→6,330P/1,188F 零回归）；2026-10-31（attr 门落地：融合序列化纳入纯 attr 变更 + R383/R383a 跨代际桶门 + truthy-"0" 双修，inline-host 0P/880F→80P/800F）；2026-10-31（lrlp 壳丢失两笔落地 + R383×CE walk 撞车仲裁：join 三案 6,330P→7,254P/264F 零回归，union 树 bisect 归因 daf3bc4c5、`__zwCeWalkMerging` 域门旁路）；2026-10-31（lrlp 残余 264F 收口：壳保留等值降级——源行 ws vs 落位上下文有效 ws 判定 + 裸文本壳序列化臂，join 三案 7,254P→**7,518P/0F 全扫**，F→P 264/P→F 0 零回归）；2026-10-31（ghost text 宿主镜像补发 + R383a 桶级跳并域收缩与 R4620 撞车合并：inline-host 80P→170P/710F 零新败，域收缩吸收 `__zwCeWalkMerging` 旁路臂）；2026-10-31（inline-host ①簇碰头仲裁：并行流 ghost text 宿主镜像补发（落地 f8fe57abd）与本流独立实现（R129 RemoveChildAt 寻址 + 代内墓碑 + R47 摘除帧 + 建时数据登记）同根同判撞车，语料等额（各自独立验证均 80P→160P），按「消费域收敛、不回退对方修复面」先例采纳先落地对方版本，本流同面机制撤改零残留；part92 引擎 apply 层正确性钉测保留 + 两层根因归因与本流 r259/r2823 bisect 独立佐证「视图补偿不落」决策入库，见 [evidence](editing-compat/evidence/2026-10-31-ihost-cross-batch-text-remove.md)）；2026-10-31（R5356 inline-host 710F 主簇双缺陷归因收口：序列化 walk 过时 R5273 单文本种子放行改基座 e9fce9466 + Actions 键盘链修饰键 PUA 字面插入抑制 f93f9484b，170P→720P/160F 零新败）；2026-10-31（pre `{}<br>` 簇收口：setupEditingHost 标记删除同 turn 双入账——R5340 臂 `RemoveChildAt(span,0)` × D1 印章 accessor `RemoveChildAtPath(span,[0])`，第二笔误删同父 br；修 part04 R5340 臂对 `__zwSelPathWrite` 子让位，inline-host 720P→**780P/100F** 零位移）；2026-10-31（insertlinebreak 臂接入 + 共享路径双修：execCommand 零变更臂补分派 + 元素边界 offset 位插入 + 同型拆分头段全切摘空节点，ilb 0P/552F→**528P/24F**，wss 616P→**848P/256F**）；2026-10-31（typing-state 追踪 + div 宿主 inline 盒拆分 + 占位 br 消费：ilb 528P/24F→**552P/0F 全扫**，wss 848P→**912P/192F**，残余收敛 wrap 族单族）；2026-10-31（②c inline 盒 wrap 拆分：wss wrap 族 192F 全清，语料全扫 **1104P/0F**）；2026-10-31（void 簇收口：querySelector 解析子树下钻 + void caret 两臂，insert-paragraph-in-void 2P/70F→**72P/0F**、insert-text-in-void 0P/48F→**48P/0F** 双语料全扫）；2026-10-31（renderer R383a 对账：配对环境判定第二通道——host 换代标记，renderer 私有队列 paired 恒 false 缺口收口，R382 兜底下未证双计形态、判定源对齐定性）；2026-10-31（shift×normal 幽灵 br 碰头仲裁：本流独立归因「陈旧裸 tag selector 重建重绑改锚——RemoveHandle 删错节点」与并行流 R5356b 345140946「RemoveHandle 换代 no-op 宿主镜像补发」同洞两面撞车（对方=滞留面，本流=错锚错删面），part04 同面机制与翻译歧义守卫两笔撤改（后者按语义保守性撤改：join 语料存在失唯一下 RemoveHandle wire、既有语义为 query_selector 首见命中移除，唯一性门使其静默 no-op 致引擎树滞留已移除节点，行为分歧风险与超时无关；当时 Timeout 差经基线复证定谳为预算墙随机截断非该门因果）采纳对方语义零残留；保留 rebind 歧义守卫 + 钉测独立面：堵住刷新环 query_selector 首见猜测锚定把 pn 写歪的污染源头（[rebind] 两跳实录），使 RemoveHandle 恒 lenient 落空，组合态复验见 [evidence](editing-compat/evidence/2026-10-31-ihost-shift-ghost-br-rebind.md)）；2026-10-31（簇 A glue 残余 S8 移除侧收口：`_zwPAIdRemove` 空表预筛 + R52/pb1 密集重建，S8 567→410ms −27.7%、S8/S5 1.62→**1.18 ≤ 1.5 达标**，bold 墙钟 ~300→117s；inline-host 宽 filter 8F 定谳既有失败非本轮引入）；2026-10-31（簇 A removeChild 融合列表同调用复用收刀：tape 归因 sel 域 removeChild 每 removal 恰 3 次 `_childNodeList` 全量物化（R126 子判定 + R5340 索引扫描 + R5340 体尾 LiveNLSync）占墙钟 98% O(n²)，R126→R5340 同调用背靠背零 mutation 段 `_r126FuseMemo` 复用落地（part04 +23 行），探针 w2 均值 6661→4109ms −38%、物化 6001→4001 次恰 −1/3，前轮 §4 ~650µs text-child 挂账定谳历史态关闭；四门零位移——engine lib 3,635P/1F s24 已知红唯一、webview lib 832P/0F、inline-host 880P/0F、join 三案 7,518P/0F/0T，见 [evidence](editing-compat/evidence/2026-10-31-cluster-a-removechild-fused-rebuild.md)）
**上一状态（2026-10-31，shift×normal 幽灵 br 碰头仲裁轮）：part04 同面机制与 bridge 翻译歧义守卫两笔撤改采纳对方语义零残留，保留 pipeline rebind 歧义守卫 + 引擎钉测（38772a918）**。
> 定谳：本流「陈旧裸 tag selector 重建重绑改锚——RemoveHandle 删错节点」与并行流 R5356b 345140946「RemoveHandle 换代 no-op 宿主镜像补发」同洞两面撞车——补断恒落 span 内部，`</span>` 后可见 `<br>` 是语料自插 brElement 幽灵（刷新环 `query_selector` 首见猜测锚定把 pn 写歪，[rebind] 两跳实录）；撤改翻译歧义守卫按语义保守性（join 语料存在失唯一下 RemoveHandle wire，唯一性门静默 no-op 致引擎树滞留已移除节点；当时 Timeout 差经基线复证定谳为 R5342 预算墙随机截断非该门因果）。
> join-different 残余 T 归因：对方树基线 aa4e9d190 直跑同得 3455P/0F/1T，四轮超时变体逐轮漂移无固定受害者——3,456 变体恰在 R5342 自适应墙沿的随机截断，0F 零失败，按预算边距记档不作回归挂账。
> **门禁（终树 38772a918 单干净二进制逐文件分跑）**：inline-host **880P/0F**、join lrlp 2607P/0F + lprl 1455P/0F、chd 420P/0F、wss 1104P/0F、ilb 552P/0F 逐位恒等；join-different 3447P/0F/2T（0F）；engine lib **3,626P/10F**（10 已知红逐名 identical）；webview lib **830P/0F**。合并树复测（再并入 R54xx/s177/s178 九笔，1155ace7f）：inline-host 880P/0F、join-different 3454P/0F/1T、engine lib 3,626P/10F 零位移。
> 明细见 [evidence](editing-compat/evidence/2026-10-31-ihost-shift-ghost-br-rebind.md)。

**状态**: Active——**当前（2026-10-31，簇 A removeChild 融合列表复用收刀轮）：R126→R5340 同调用 `_childNodeList` 全量物化复用（R5340-glue，part04 +23 行）落地——tape 归因 sel 域 removeChild 每 removal 恰 3 次物化占墙钟 98%（O(n²)），修复后探针 w2 均值 6661→4109ms（−38%）、物化 6001→4001 次恰 −1/3；四门零位移（engine lib 3,635P/1F s24 已知红唯一、webview lib 832P/0F、inline-host 880P/0F、join 三案 7,518P/0F/0T）**：

**归因与修复（R5340-glue）**：簇 A 前轮 §4/§5 挂账的 tape 插桩收口轮——24 检查点 + 8 探针窗口×3 样本：sel 域 removeChild×2000 主项钉死为每 removal 恰 3 次 `_childNodeList(sel, handle)` 全量物化（① trap 入口 `_r126IsChildOf` 融合子判定 ② R5340 臂 pending 索引扫描 ③ R5340 体尾 LiveNLSync 重建；每次 = host 串回环 + `JSON.parse` ~2000 条 + 逐子 wrapper 构造 ~1.1ms），占墙钟 98%；`_zwChildBaseCache` 因每 mutation 失效设计与 mutation 循环相克而无效。前两次物化同调用背靠背、中间零 mutation（只有 stamp/门条件纯读）——`_r126FuseMemo`/`_r126FuseMemoKey` get-trap 闭包体作用域 memo（每次属性访问独立实例天然隔离）+ removeChild 入口防御清 + R5340 消费即清，同调用同数组逐位同结果；stamp+bucket 快路因 overlay 全语义不等价（R383/R5273/R5356/R382/R4620 门在编辑语料高发）否决留档。§4 ~650µs text-child 差额复测 +12% 不复现，定谳历史态，挂账关闭。残余每 removal 仍 2 次物化（R126 判定 + LiveNLSync），彻底 O(n) 化需 R55/R304 身份稳定副作用与 `_zwHCLiveInvalidate` 失效覆盖重审，留独立整轮。见 [evidence](editing-compat/evidence/2026-10-31-cluster-a-removechild-fused-rebuild.md)。

**上一状态（2026-10-31，R54xx 轮）：簇 A glue 残余 S8 重设移除侧相归因收口与修复——S8 ≤ 1.5×S5 验收线达标（eb141f7e1）**：

**归因（innerHTML setter 逐相 tape 三轮收敛，临时插桩已剥）**：S8 特有成本集中在 F（`_mo_notify`）~110ms + G（PAIdRemove 循环）~108ms（E CE 断连 17ms、A childNodeList 读 13.6ms 为小项）；**单一最大 villain = `_zwPAIdRemove` 的 `nd.id` get-trap/attr 读链 20-32µs/次 ×6,000 调**（setter R125 尾扫 2×2000 + R52 purge 2000，idRead 累计 122-195ms）——无 id 页 by-id Map 恒空、get 恒 miss，全数纯浪费；次 villain = **R52 稀疏假设在 bulk 移除下反转**——innerHTML 整体重设每子全命中 pa 表，逐命中 `indexOf+splice` O(n²)（4,000 remFlat × 4,000 表），pb1 桶对冲同构（探针 13.4ms/次线性涨）。

**修复三笔（part05.js，+53/−4）**：①`_zwPAIdRemove` 空表预筛（`!_zwPendingAddedById.size` O(1) 快出；有 pending id 走原路径零变化）；②r2b R52 purge 密集重建——命中收集，≥8 命中单遍 Set-filter 重建数组（成员与序等价），稀疏保持逐命中定点 splice；③pb1 桶对冲同构密集重建。不改全局表 Set 化（消费面依赖数组序与原地 splice，风险大于收益）。

**A/B（同日同负载三样本中位，净树二进制）**：S5 351→**346.6 不变**（修复面只在移除侧）、**S8 567→410.0（−27.7%）、S8b 742→525.4（−29.2%）、S8/S5 1.62→1.18 ≤ 1.5 达标**；S5c 574.8 对照；sanity-child-count 钉 Pass。

**门禁（净树）**：inline-host **窄口径** 880P/0F、void 72P/0F+48P/0F、ilb 552P/0F、wss 1104P/0F、chd 420P/0F、join 三案 7,518P/0F 全零位移；bold run 1,138P/1,910F 失败集对 10-30 基线**逐位 identical**、**墙钟 117s（前账 ~300s，−61%，bold 重设密集直接兑现）**；engine lib 3,625P/10F 已知红集 identical（修复态 bisect 前所跑，净树与之行为等价——清理只删死读 + 还原阈值）；webview lib 830P/0F。**假回归归因附账**：宽 filter `inline-editing-host` 888 案 8F（shadow-root delete 4F + placeholder 伪元素 4F，`expected "X" got "Type something"` 占位镜像形态）为**既有失败**——pristine HEAD stash 同集逐名 identical、昨账 880P/0F 口径为窄 filter 单案族（本轮窄口径复测恰合，3 失败文件从不在门禁集）；嫌疑并行流 3bbc7ca56/f8fe57abd，记档不回退待认领；**方法论：门禁 filter 口径必须记 evidence 原文**。**合并树复测（pull 拉入 shift×normal 碰头仲裁 38772a918 + s177-s180 后终验）**：探针比 1.18 保持、语料七面全零位移、bold 1,910F 仍逐位 identical（墙钟 118s）、engine lib **3,635P/1F**（唯一红 s24 flip 渲染流债——并行流 s179 fetch 根修清掉 fetch/Request 8 红 + iframe SW 红，外部改善）、webview lib 832P/0F。明细见 [evidence](editing-compat/evidence/2026-10-31-cluster-a-s8-remove-side-attribution-fix.md)。

**下一步**：① 簇 A removeChild 残余 O(n²)（R126 子判定 + LiveNLSync 每 removal 2 次物化）独立整轮、插入侧残余（S5 ≤300 线未达 346.6：s111 臂 34-46µs/append / CE loop 20-40µs）、execCommand 簇序 → ② run 面残余大簇按现行账重排。（removeChild trap glue 首刀 R5340 同调用复用已落地收口）

**上一状态（2026-10-31，R54xx 轮）：renderer R383a 对账——配对环境判定第二通道（host 换代标记），判定源对齐定性（挂账实质完成）**。
> 对账结论：cursor 通道覆盖 webview 形态（含 runner，语料基线 paired=true 下取得）；engine 纯测试 cursor 恒 0 → 恒并（t8r-4 臂②既有保护语义）；renderer 私有队列路径 apply 后发换代通知但无游标发布（`publish_mut_applied_cursor` 为 webview 线程 thread_local，发布不到 js_worker；journal drain 后 cursor 数值对重放/attr 扫描本无意义），唯一受影响面 = JS 侧 `_zwApplyPairedEnv` 与 R5356 种子过时判定。
> 实现（两笔）：①shim `_zwApplyPairedEnv` 加第二通道 `globalThis.__zwHostApplyBumpSeen === true` 置闩；②renderer `notify_shim_apply_generation` 通知脚本升级置标记（js_worker 线程执行，只在 apply 成功分支发）。
> 行为面如实评估：Arm3×跨代际桶组合 mirror 行为钉序列化恒正确、stash A/B 不红（R382 条目级兜底已防双计）——非「已证双计修复」。
> **门禁**：新钉两枚；renderer lib 355P/5F（5 红既有逐名 identical，+2P = 新钉）；engine lib 3,623P/10F 红集 identical；语料抽查零位移（void 72+48、inline-host 880P/0F、join paragraphs 3456P/0F）。
> 明细见 [evidence](editing-compat/evidence/2026-10-31-renderer-r383a-paired-env.md)。

**上一状态（2026-10-31，R5398 轮）：void 簇收口——querySelector 解析子树下钻 + void caret 两臂，insert-paragraph-in-void 2P/70F→**72P/0F**、insert-text-in-void 0P/48F→**48P/0F** 双语料全扫**。
> 前置归因（分层探针定谳，修正上轮「引擎树快照读盲」假设）：host 快照层无读盲——innerHTML 替换经 host 异步 apply（R309 既有语义），同 turn 快照本就只有空 editor div；真根因在 JS 面——querySelector 的 R322 回落只 compound 匹配桶 added **顶层**代理，`<div><br></div>` 顶层 div 命中、子代 br/img 永不达（querySelectorAll 的 R309 臂全量重建故命中；handle 分支既有下钻，解析分支缺失）→ caret 不落位假失败。
> 实现（三笔）：①R54xx 前号（part04 R322 解析分支）：顶层不命中时 `_zwHCCollectSubtree` 下钻，首个命中后代原样返回 plain 代理（不物化）；②⓪ void 拆分臂（part01 `_zwCeParagraphMutate`）：void caret → 容器拆分，void 及后兄弟移新块，新块标签 li→li、标题按语料分支（可见 void 或有后继→同标签，否则 div）、空半占位 br 打标；跨域摘挂——plain 子先 removeChild 再挂 handle sep（宿主桥 appendChild 对 plain 子序列化拷贝不摘原父）；③⓪ void 插入臂（part01 `__zw_ce_insert`）：可见性回溯——非空文本接尾/带文本元素末文本追加（hr 例外 void 前），不可见 void 跳 ws-only 与空 inline，img/input 遇 ws 停，hr 消费 ws，耗尽回落 embed/wbr/img 插首个跳过元素前、br/input 插 void 前。
> **门禁（净树）**：void 双语料 **72P/0F + 48P/0F 全扫**；ilb 552P/0F、wss 1104P/0F、chd 420P/0F、inline-host 880P/0F、join 三案 7,518P/0F、bold run 1,138P/1,910F 零位移；engine lib 3,617P/10F 已知红集逐名 identical；webview lib 830P/0F；非基线面 undo/cloning 等经 stash 基线对照两态同态。
> 明细见 [evidence/2026-10-31-void-element-caret-and-query.md](editing-compat/evidence/2026-10-31-void-element-caret-and-query.md)。

**碰头仲裁补记（同日，本流 shift×normal 幽灵 br 轮）**：本流独立归因 R5356「补断落点」定性证伪——补断恒落 span 内部，`</span>` 后可见 `<br>` 是语料自插 brElement 幽灵；根因链=裸 tag selector 注册时唯一、Enter 后失唯一不更新 → 刷新环重建 `rebind_handle_node` 首见命中猜测锚定（[rebind] 两跳 `parent=DIV→SPAN` 实录）→ RemoveHandle 经 pn 命中删 span 无辜 br（[apply-rh] 实录）→ 真 br 永生经基底重建每子测重吸收（拷贝指纹 `["span#-@S","br#-@S"]`）。与 R5356b 345140946 同洞两面并存：对方面=Actions 链排水后 pn miss 删不掉（滞留），本流面=刷新环 pn 写歪删错节点（错删无辜+真目标永生）。撤改两笔采纳对方语义零残留：part04 同面块撤下（采纳对方宿主镜像补发版本）；bridge 翻译歧义守卫（曾给既有 `RemoveHandle→Remove` 翻译臂加「失唯一跳过」门）组合态复验二次撤改（语义保守性定案）——join 语料存在失唯一下 RemoveHandle wire、既有语义为 query_selector 首见命中移除，唯一性门使其静默 no-op 致引擎树滞留 JS 侧已移除节点（行为分歧风险，与超时无关）；当时 Timeout 差（含门 2T/撤门 1T）经基线复证定谳为预算墙随机截断非该门因果（evidence §6），撤改以语义论据独立成立。保留一笔：rebind 歧义守卫（`query_selector_all != 1` 跳过重绑）堵截猜测锚定把 pn 写歪的入口 + 引擎钉测 `rebind_handle_node_skips_ambiguous_selector_ihost_ghost_br` 锚定。合并树门禁（对方 R5356b/R5371/s175 面）：inline-host 880P/0F、join lrlp 2607P/0F、join lprl 1455P/0F、chd 420P/0F、wss 848P/256F、ilb 528P/24F 逐位恒等；join-different 3450P/0F/1T（1T=预算边距截断，基线复证定谳见 evidence §6）；engine lib 3,618P/10F（+1P=本流钉测，10 已知红恒等）、webview lib 830P/0F；终树（并入 R5392/R5395/R5398/s176）组合态复验见当前轮状态块；明细与归因链见 [evidence](editing-compat/evidence/2026-10-31-ihost-shift-ghost-br-rebind.md)；**上一轮（同日，R5392 轮）typing-state 追踪 + div 宿主 inline 盒拆分 + 占位 br 消费：ilb 528P/24F→**552P/0F 全扫**，wss 848P/256F→**912P/192F**（+64 零回归，残余收敛 wrap 族单族）**：①typing-state：collapsed caret format 命令（italic/bold）切 `globalThis.__zwTypingTags` 不动 DOM，`__zw_ce_insert` 三插入臂按记录嵌套包裹插入，`_zwSync` 入口单点清除（11 调用点全为页面可见 Selection 方法；内部重锚 `_ranges` 直赋存活）；②②b inline 盒拆分：div 宿主（`hostTag==='div'` 与 ① 守卫对齐）caret 在非块直子 inline 盒 → 截尾段复制壳；③占位 br 打标 `__zwPlaceholderBr` + inserttext 进场从 caret 上行扫描移除（不门 typing state——preserving 组同样消费，首版错误门在 typing state 致 preserving bold 16F）。回归归因：②b 首版守卫只看 display 截胡 chd `p/span display:block + 内层 span` 16F，补 `hostTag==='div'` 后 420P/0F 恢复。门禁（14045bc7c）：inline-host 880P/0F、join 7,518P/0F、chd 420P/0F 零位移；bold run 1,138P/1,910F 逐位恒等；engine lib 3,617P/10F 已知红集逐条 identical；webview lib 830P/0F；见 [evidence](editing-compat/evidence/2026-10-31-typing-state-and-inline-box-split.md)；**上一轮（同日，R5371 轮）insertlinebreak execCommand 臂接入 + 共享路径双修（c81777413）：ilb 0P/552F→**528P/24F**、wss 616P/488F→**848P/256F**（+232 零回归）**：臂缺失根因——`_zwExecCmdInputType` 有 inputType（通用臂照派事件）但实应用分派四族无 insertlinebreak，零变更臂与 `expected "abc<br><br>" got "abc"` 吻合；三笔 = `_zwCeParagraphMutate` lineBreakOnly 模式 + `__zw_ce_line_break_mutate` 桥（ws 显著性/可见性补断复用 `_zwCeLineBreakAt`）、`__zw_ce_insert` 元素边界 offset 位插入（+8）、同型拆分头段全切摘空文本节点（+224）；门禁全绿零位移，见 [evidence](editing-compat/evidence/2026-10-31-insertlinebreak-arm-and-shared-path-fixes.md)；**再上一轮（同日，R5356b 轮）inline-host B 簇清零：无 id pending 元素跨批 remove 宿主镜像（780P→820P/60F，345140946）+ frag 元素种子与 live-list 同步（820P→**880P/0F 全扫**，93c02bd3c）**：`{}<br>` 形 setup 宿主 runner 引擎读永久盲（`with_query_view_doc` 基座钉定，engKids 恒 0）——frag 种子三层：注册（sel 形 markup innerHTML= 登记解析顶层代理列表 + **叶内容门**：含容器元素不登记，健全基座域零种子）、消费（注册序整体替换 + 摘离成员 `parentNode != null` 过滤 + 豁免 R5356 过时门）、live-list 同步（种子宿主 insertBefore splice 进 `entry.nodes`——管线 gen bump 清桶后种子仍持完整子列表）；**撞车复归归因**：首轮「全面注册+序列化无条件吐种子+过时门豁免」遮蔽健全基座（join 7,518 全 F，got=setup 原文）、「out 空门」被管线流证伪——终案注册面叶内容收窄；门禁 inline-host 880P/0F、join 7,518P/0F、wss 616P/488F、chd 420P/0F、engine lib 3,614P/10F、webview lib 830P/0F，见 [evidence](editing-compat/evidence/2026-10-31-r5356b-frag-seed-and-live-list.md)；**上一轮（同日）pre `{}<br>` 簇收口：setupEditingHost 标记删除同 turn 双入账互斥修复——inline-host 720P→**780P/100F** 零位移**：根因（apply trace 实证）——语料 setupEditingHost 对 D1 印章文本（`__zwSelPathWrite`）的同步 `node.remove()` 同 turn 触发两条 wire：R5340 臂（part04）`RemoveChildAt(span,0)` 删 T"{}" + 印章 accessor 自带移除面（part03）`RemoveChildAtPath(span,[0])` 再删发射序 0 号位=同父 `<br>` → br 引擎侧消失 → CE 空宿主插双 breaker `[T("\n\n")]`≠期望 `[T("\n"), BR]`。手工探针 t18-c 此前不现形因 `await tick()` 已 drain journal、`alive()` 假→印章面跳发，差异在删除与 drain 时序；R5058 嫌疑排除（只丢锚不删 br，t19 实测）；R5356「30 enter+130 shift-enter 补断落点」的 enter 侧 30F 实为本簇。修复=part04 R5340 臂 wire 发射加 `!child.__zwSelPathWrite` 让位（桶 retract/通知不动，与 D1 臂身份印排除同一互斥先例，+9/-1）。**门禁（修复二进制逐文件分跑）**：inline-host **780P/100F**（F→P 60/P→F 0，剩余 100F 逐行核对全 shift×normal 补断簇）、join 三案 3456+2607+1455=7,518P/0F、cannot-have-div 420P/0F、wss 616P/488F、ilb 0P/552F 逐位恒等、engine lib **3614P/10F** 已知红恒等（fetch 契约簇+s24 渲染流债）、webview lib 825P/0F；最小复现页 wire=`[T("\n"), BR]`/`\n<br>` 与语料期望逐字一致（补断未过发）；探针两页保持未跟踪，见 [evidence](editing-compat/evidence/2026-10-31-ihost-pre-br-double-remove.md)；2026-10-31（本流两笔收口：无 id pending 元素跨批 remove 宿主镜像 780P→820P/60F + R5356b frag 元素种子与 live-list 同步 820P→**880P/0F 全扫**，inline-host 清零）；**上一轮（同日）R5356 inline-host 710F 主簇双缺陷归因收口：序列化 walk 过时 R5273 单文本种子放行改基座 + Actions 键盘链修饰键字面插入抑制——inline-host 170P→**720P/160F** 零新败**：两簇此前叠加在同一批失败行上，缺陷 1 揭开视图层毒源后缺陷 2 才在账面可见——**缺陷 1（engine shim，e9fce9466）**：`setupEditingHost("abcd")` 纯文本经 part04 `_zwRegisterTextEl` 登记 span→T:"abcd" 种子（gen=G0）；CE enter 拆文本经 R48 CharacterData 写路径**原地**改注册节点 data 为 ""（引擎树 apply 后已 `[br, T:abcd]`，probe9 引擎快照直读证伪此前「行断语义缺失/基底缓存 stale」假设）；`removeAttribute(style)` 开 `_zwPendingAttrDirty` 祖先融合臂后，序列化 walk 内 R383a 跳并拦掉桶修正，R5273 种子替换在桶检查前**无条件**吐过时单节点 → container 视图丢子。修复=登记条目加 `gen` 代际戳（part06）+ 种子块过时判定「`_zwSerSkipCrossGen` 域内 ∧ gen≠当前 ∧ 配对环境」放行改读基座（part05）；live 视图与 engine 纯测试环境不变；**缺陷 2（wpt-runner，f93f9484b）**：缺陷 1 落地后残余 470F 字节对比全部 got = 可接受项 + 前缀 U+E008——testdriver Actions 链逐 op 落 `apply_testdriver_command` 时修饰键 PUA 码点（E008/E009/E00A/E03D）落单字符 InsertText 分支字面插入（send_keys 循环有修饰键表、Actions 链没有）。修复=keydown/keyup 臂补映射：事件 key/code 用 WebDriver 键名 + 修饰键步 prevented 检查后早退；FIXME 留置=Actions 无修饰键**状态**，块级 CE 宿主 kShift+Enter 与裸 Enter 同汇 Submit（语料只压行内宿主不触碰）；**门禁（干净树/等价语义树）**：join 三案 7,518P/0F、white-space-style 616P/488F 与 cannot-have-div 420P/0F 零位移、engine lib 3612P/11F 已知红逐条 identical、webview lib **824P/0F 新基线**（旧 r3259 红已被并行流 s170 b100c742a 修掉）、inline-host 终树复验 **720P/160F**、对原 710F 集 comm -13 = 0 新败；记账说明：join/wss 账出自惰性日志+变量改名插桩二进制，probe/chd/inline-host/engine lib/webview lib 已在剥离树复验、旗标引用 grep 清零、part04 无 diff，见 [evidence](editing-compat/evidence/2026-10-31-r5356-stale-seed-and-actions-modifier.md)；**剩余 160F 单一机制**=可见性补断 `<br>` 落宿主 span **之后**（container 子级）而非内部（EXP `<span…><br><br></span>` vs GOT `<span…><br><br></span><br>`；30 enter + 130 shift-enter），属 insertLineBreak 落位语义；**下一步**=① inline-host 剩余 160F 落位锚点归因收口 → ② white-space-style 488F（italic/bold 组）→ ③ part04 BUILD void 挂账（70F）→ ④ renderer 侧 R383a 对账 → ⑤ 簇 A glue 残余、execCommand 簇序。**上一轮（同日）ghost text 宿主镜像补发 + R383a 桶级跳并域收缩（与并行流 R4620 撞车合并）：inline-host 跨子测文本渗漏根修 80P→170P/710F 零新败 + attr 门轮 join 回归第三度独立归因**：两条线——**缺陷 A（56045919e）**：`followed by text` 族 finally-remove 后宿主基座滞留幽灵文本（part04 R129 ChildNode.remove sel 父形态仅单 wire `__zw_remove_handle`，跨批句柄表换代解析失败温和 no-op），补发 `__zw_remove_child_at(父 sel, 基底索引)` 宿主镜像（`_zwChildBaseCache` 扫 nodeType+data 首见索引 + R383 同款跨代际门；同代不补发防融合物化虚影误删，视图半区补偿 R294 两版证伪不落），zz-probe-abc t1–t4 F→P；**缺陷 B（cb3cff932，rebase 后 4c6653b70）**：复跑 join 三案暴露 attr 门轮引入回归 4,959P/2,559F（对基线 6,330P/1,188F −1,371P 全集中 lrlp）——本流 bisect + zz-probe-lrlp 三轮循环 + `ZW_DBG_LOG` 打点独立归因与并行流 R4620 同源同判（R383a 桶级早退在配对闩下对所有 stamp 滞后桶跳并，apply 后桶 identity 记账 stamp 必滞后 → live 视图消费面致盲）；修复=跳并域收缩到序列化 walk（`_zwSerSkipCrossGen` 标志，`_zwFusedInnerHtmlSel`/part04 outerHTML sel 分支 try/finally 置位），**rebase 撞车合并吸收并行流 R4620 `__zwCeWalkMerging` 旁路臂**（CE walk 不置序列化标志，域收缩覆盖其诉求，按「消费域收敛、不回退对方修复面」仲裁先例）；本流分支门禁：join fail 集对 22b4 基线逐位 identical、inline-host **170P/710F**（710F ⊂ 800F 零新败）、white-space-style 616P/488F 与 cannot-have-div 420P/0F 零位移、engine lib 3611P/11F 已知红集 identical；**组合态（域收缩 × lrlp 两轮修复）复验**：join 三案 **7,518P/0F 全扫保持**（fix13 账零位移）、inline-host **170P/710F**（= 分支账，零新败）、white-space-style 616P/488F 与 cannot-have-div 420P/0F 零位移、engine lib 3612P/11F 已知红集 identical、webview lib 822P/1F 已知红唯一，见 [evidence](editing-compat/evidence/2026-10-31-inline-host-ghost-text-host-mirror.md)；卡点=join 三案全扫无一级簇；**inline-host 710F 主簇**（insertLineBreak/insertParagraph 行内 host 行断插入语义缺失，LF+`<br>`，display 五型对称分布）；下一步=①inline-host 705F 主簇 → ②white-space-style 488F（italic/bold 组）→ ③part04 BUILD void 挂账（70F）→ ④renderer 侧 R383a 对账（域收缩后仅序列化域跳并，范围收窄）。**上一轮（同日，并行流）lrlp 残余 264F 收口：壳保留等值降级——join 三案 7,254P→7,518P/0F 全扫（96.5%→100%）零回归**：根因修正——上轮「normal 时降级」预判不完整，语料源码（paragraphs 案 expectedHTML 为 `aAttrsInLeftBlock` 参数化函数，accept 恒 `style="${rightWhiteSpace}"`）揭示真正规则是**壳只在源行有效 ws ≠ 落位上下文有效 ws 时才需要**：paragraphs 落位左块恒异→恒壳（无条件 normal 降级会倒灌），lrlp 落位宿主（normal 上下文）right=normal 时等值→裸（264F 唯一 delta）。落地三笔全在 part01 `_zwCeJoinApply`：①落位修正块上提至壳构造前 + `_wsNeedsShell` 等值判定；②`_stampWs` 等值早退 + 文本节点等值裸迁入；③裸文本壳序列化臂（shim 文本节点无 outerHTML 此前静默跳过，nodeType===3 走转义 textContent → insertAdjacentHTML 同 move 语义）；对 fix13 逐测试 ID **F→P 264 / P→F 0**、7,518 行恒等，insertparagraph 三案逐位恒等，engine lib 11 红清单恒等、webview lib 822P/1F 已知红唯一，见 [evidence](editing-compat/evidence/2026-10-31-lrlp-normal-ws-shell-degradation.md)；卡点=**join 三案全扫后残余面无一级簇**——按语料账重排：inline-host 800F（attr 门采样：跨子测文本渗漏 + D1 族 text-view 生命周期）> white-space-style 488F（italic/bold temporary-inline 保留组）> execCommand 簇序；下一步=inline-host 800F 残余簇收口（attr 门轮已采样失败行）→ white-space-style 488F → execCommand 簇序。**上一轮（同日）lrlp 壳丢失两笔落地 + R383×CE walk 撞车仲裁：join 三案 6,330P→7,254P/264F（84.2%→96.5%）零回归**：①上轮 429 中断遗留 R4620 join 落位两笔验证落地——js_dom_bridge InsertBefore ref 预解析兜底（根因=plain 壳 iah 兄弟插入使同 ref `:nth-child` 索引后移，handle 壳 ref selector apply 时失配 lenient skip → 壳丢失；批前 ins_targets 预解析 + apply 失配查表须仍挂树 + InsertAdjacentHtml 后刷新表）+ part01 壳 caret 落位钳制（plain 壳融合视图缺席 `indexOf=-1` 负 offset → setStart IndexSizeError → 整批 wire 永不 drain；缺席落容器尾），lrlp 三方法 396F/方法→88F/方法、对 fix10 逐测试 ID **F→P 924 / P→F 0**；②rebase 拉入 attr 门后 union 树全量曾跌 4,959P/2,559F（lrlp P→F 2,295，失败形态升级为整动作 no-op），三跳 bisect 归因 **daf3bc4c5 R383 桶级跳并**——CE delete 族对融合视图对象 identity 消费（caret 锚链持 setup innerHTML 物化 pending 壳对象），R382 条目级 ghost 确认被桶级粗跳前置短路从未运行；修 = `__zwCeWalkMerging` 域门旁路（part01 两入口 impl 换名 + wrapper 置位/复位，part05 R383 条件加旁路臂），同 `__zwCeWorldSyncing` 撞车仲裁先例按消费域收敛、不回退对方修复面；join 对 fix11 逐测试 ID **零翻转**、insertparagraph 三案逐位零位移（attr 门收益保全）；③webview 钉子欠账认领清偿：`execute_dom_script_path_generation_bump_call_site_present_d2` 红 = 本流 0920f3fd8 三条配对面而钉子写死 `==1`（make test 中止 workspace 段掩盖至今），改 `>= 3` 存在性钉复绿；见 [evidence](editing-compat/evidence/2026-10-31-lrlp-shell-loss-and-r383-collision-adjudication.md)；卡点/下一步已被本轮收口取代（lrlp 264F → 0F，join 三案全扫，见当前块）。**上一轮（同日）join paragraphs 簇清零：join 三案 5,698P→6,330P/1,820F（75.8%→84.2%）零回归**：`_zwCeSkipWs` 方向反转根因（恒走 previousSibling 的前邻语义被三个后邻臂复用——fd 块界/fd select/backspace select 在 `</div>\n<div>` 块间空白几何下从空白文本 backward 回走命中**前邻**，startNode=caret 侧块自提取，引擎删 caret 侧内容）定位（单案探针 + `ZW_WIRE_TRACE` wire 追踪 + JS 阶段标记，两轮 trace 分开「寻址错/入参错」）→ 方向参数化修复（backspace 前邻臂传 true，后邻臂默认 forward）；对 fix9 逐测试 ID **F→P 632 / P→F 0**（= paragraphs fd 316 + select-boundary 316 全清），结果行 7,518/7,518 恒等；分面账 paragraphs 三面 **0/0/0**、lprl 0/0/0；插桩移除后干净二进制逐位复现 drift=0，见 [evidence](editing-compat/evidence/2026-10-31-join-skipws-direction-paragraphs-sweep.md)；**并行流（2026-10-31）attr 门落地**：融合序列化纳入纯 attr 变更（`__zw_pending_attr_sels` 桥 + 门第三臂）+ R383/R383a 跨代际桶门（apply 落座条目幽灵双计，apply 配对环境闩护 t8r-4 裸 bump 语义）+ `__zw_contains` truthy-"0" 双修，inline-host **0P/880F→80P/800F**、white-space-style 616P/488F 与 cannot-have-div 420P/0F 零位移、engine lib 失败集与基线逐条 identical（t8r4_k3 复绿）、跨域电池 dom 55,948P/527F/14T（F 对 R5331 −49）selection 27,589P/23,591F/5T（P −1 零回归，101 文件全分母首个全面账），归因链与中间账（per-parent 清单方案 20P/860F 撤改闩）见 [evidence/2026-10-31-attr-gate-land.md](editing-compat/evidence/2026-10-31-attr-gate-land.md)；卡点/下一步已被本轮收口取代（lrlp 1,188F → 264F，见当前块）。**上一轮（同日）行内壳变体收敛**：join 三案 3,108P→**5,698P/1,820F（41.3%→75.8%）**零回归：六笔修复全落 part01.js——ws 摘离前快照（takeLineFrom wsVal 出参，B 族错印根因：先 removeChild 后记 srcEl 走断祖先链）、混合行 per-run 兄弟壳（单一外包 span 双层嵌套拒收）、行内 run 升位（fd/select 臂 dst=caret 侧 inline run 嵌套根因，壳落行容器 sibling 位）、落位序列化逐壳交替（混合 plain/handle 壳尾批逆序实证）、块间分隔空白消费（`\n` 变体 1,027 案）、`ended` 出参（**自实现即缺失**——克隆臂 r2.ended 恒 undefined，行界在壳内消费后循环越界提取；walk 形状 Node 最小 DOM 复算锚定因果）；对 fix7 7,518 案逐测试 ID **+2,590P / P→F=0**；paragraphs backspace 676F→**0**、lprl 三方法全清零、select-boundary 三案全清，见 [evidence](editing-compat/evidence/2026-10-31-join-inline-shell-family-convergence.md)。**上一轮（同日）white-space join 语料收口 + 撞车仲裁终验**：join 三案 **0P→3,108P/4,410F（41.3%）**零回归（双 wrapper 家族视图滞后根因 + 世界同步 + plain 寻址补全，见 [evidence](editing-compat/evidence/2026-10-31-join-corpus-family-resync-and-wired-addressing.md)）；rebase 撞车合并（并行流 D1/D2/D3）组合态曾跌 1,395P，两处域交互缺陷已仲裁终验——**登记表重锚×世界同步换代**（一级重锚 regIdx 指向换代前位次，`_zwSynced` 门让位二级 sel-path 重解析）+ **D1 stamping×回灌子树**（stamp 域移除覆盖缺口致行界 `<br>` 残留，`__zwCeWorldSyncing` 门旁路），落地后 join 语料复跑 **3,108P/4,410F 对 fix7 7,518 案零漂移**；make test engine lib 3,611P/11F=9 已知红零新增 + 2 渲染流债（r113 cssText 尾分号、s24 ua-display flip，stash A/B 实证归 rebase 拉入提交，记档待渲染流认领），见 [evidence §8-§9](editing-compat/evidence/2026-10-31-join-corpus-family-resync-and-wired-addressing.md)。M2 次轮完成并收环；white-space join 簇**实现已落地并证明**（2026-10-30 七轮：part01 CE 块界 join 三臂——backspace 前邻/forwarddelete 跨容器空界选区+collapsed 终点、首行提取（`\n`/br 行界、inline 克隆壳随迁）、white-space 保留壳（attr 原始串→class 映射→normal fallback）；探针直调三形态与语料 expectedHTML 逐字吻合。**0P 根因修正**：语料走 send_keys 管线非 execCommand，fail 链=setupEditingHost 标记剥离对解析文本节点不持久化（纯快照无 selector/handle，D1）→ send_keys 引擎动作触发 fused-view 重同步标记回滚（D2）→ caret/listener 陈旧走分离树（D3）——全部 marker+send_keys 编辑语料同链受阻，execCommand 族（run 面）不受影响；修复切入见 [evidence](editing-compat/evidence/2026-10-30-ws-join-impl-and-pipeline-blocker.md)，**D1/D2/D3 已落地**（同日八轮，见下方状态块与[落地 evidence](editing-compat/evidence/2026-10-30-d1d2d3-land-join-corpus-acceptance.md)，join 三案 **0P/7,518F→603P/6,915F**）；2026-10-31（insertParagraph 簇落地：live-DOM 降解模型）；簇 A（cloneNode 性能）**修复已落地**（tape 插桩收口 villain=R263/R262 range 注册表 O(n²)，pass1 预筛门 + 原路径逐字保留；SPAN/IH/CLONE 3.6-9.3×、RESET 10.5×；bold 209s 计数零位移、make test 9F 已知红集零新增、ranges A/B 零 verdict 翻转；残余 s111 臂/CE loop/removeChild glue 下轮候选）。**trap 层四笔叠加落地（6e294cd8a，与 R263b/R262b 合并互补）**：①part06 `__zwSelectionFollowRemoval` 注册提升（未注册 globalThis 属性 `typeof` 解析 miss **17.6ms/次**，惰性注册页面永不注册；remDbg 17.3→0.009ms/call，p3t move -97.5%）；②part04 innerHTML CE 环抬入 `_ihPc===true` 门（批量桶子代反链缺失，per-child `_elConnected` 落 R155 全表反查 6022 次 0 中 ~2.5ms/次；spec concept-connected 父断⇒子断）；③part03 键控根验证 + indexOf 快道（S4 -87%）；④合并冲突解法保预筛全文 + rmRoot 惰性化（R351 根缓存写侧同步，直引变量惰性化后缓存 null 隐患消除）。合并基座探针三样本：**S5 317.2ms / S8 854.9ms / p1-clone 18.1ms / R155 0×3**，无叠加回退；43 案零回归 + dom/selection/web-components/html-syntax 跨域电池 systemd 在途（结果入簇 A evidence §9）。预算窗：自旋簇主体系 runner V8 看门狗 `ZW_SCRIPT_TIMEOUT_MS` 90s 预算截断（非死循环，corpus 脚本已修 300s/变体 + CASE_WALL_S 自适应），other 面同预算复跑收口 **2,404P/11,159F/3T**；三分类窗：cloneNode 前置成本性能簇（挂账）、removeChild 对 sel 父 pending 解析子静默穿透真自旋（**已修 R5340**）、runner 完成接线过早置 all_loaded（缺陷 #2，**已修 R5341**：静态尾改 load+setTimeout(0) 门控镜像原生时序，write 注入路径维持直调；回归钉两枚 + 探针双 Pass + scripting-1 verdict 流零影响）。R5340+R5341 双落复跑：预算窗三例 Timeout **全部收口**（① delete-without-unwrapping 200F/0T、② inserttext-last-mutation 64F/0T、③ white-space join 三案实为 other 面平墙 120s 过紧的外部截断假象——corpus 脚本 **R5342** other 面接线自适应墙 60+nv×15+90；R5342 后 other 面全量复跑终账 **2,406P/12,308F/0T**，Timeout 清零、分母完整、总账逐位核对：F = 基线 11,159 + R5340 解锁 1,144 + R5341 回归 5）。**run 面全分母 3 分片重跑收口（同日四轮）**：43 案全出零 case 级 Timeout，**39,885P / 47,804F / 0T（87,689 行，45.51%）**——分母对 M2-r1 15,343 解锁 5.7×、Pass 7,485→39,885（率降为分母效应）；对 M1 96,663 差 -8,974 全部定位自旋四案 delete/forwarddelete/multitest/insertparagraph（分片二进制无 R5340），四案新二进制复跑归 R5340 收敛轮；误杀案 createlink-with-selecting-img 新二进制直跑收口（3 子测试 0P/3F，createLink 未包裹 img，归 createLink 簇）；bold 全矩阵新二进制抽查逐位零位移，账视为现行；分片+续跑 idx 槽位 bug 已修（resume 跳过保序）。明细见 [evidence/2026-10-30-run-budget-unlock-rerun.md](editing-compat/evidence/2026-10-30-run-budget-unlock-rerun.md)。**自旋四案新二进制复跑收口（同日五轮）**：delete/forwarddelete/multitest/insertparagraph 以 R5340+R5341 二进制直跑，**逐测试 ID 对照零 verdict 翻转**——delete +770 行、forwarddelete +357（removeChild 自旋解锁出真实断言）；insertparagraph -174、multitest -266 为 R5340 分支开销致重型变体吞吐降 ~25%、300s 预算窗内完成测试变少的纯截断（前缀包含证明）；四案残余分母缺口（对 M1 -8,289 行）挂簇 A 修复后复测。**四案簇 A 修复后复测已收口（同日六轮，[rerun evidence 终账节](editing-compat/evidence/2026-10-30-run-budget-unlock-rerun.md)）**：fc12a1ff8 二进制单案直跑，**26,444r 13,935P/12,509F（+2,113 行）**——insertparagraph 7,195r **全分母**（script timeout 清零）、multitest 9,497r 全变体完成（-595）、forwarddelete +816（-2,093）；delete 无实质恢复（4,354r，-129 中止集位移，对 M1 -3,488）——重区实测 ~2.3s/行，瓶颈证非 range 注册表，归簇 A glue 桶（removeChild trap/s111 臂/CE loop/execCommand glue/setup parse），下轮候选同前。run 面现行账更新：**41,171P / 49,318F / 90,489 行（45.50%）**，对 M1 96,663 缺口 -6,174。**insertParagraph 簇落地（2026-10-31）**：flat outerHTML-rewrite 模型换代 live-DOM in-place 降解模型（part01 三分支——host 分家/blockish 同标签分家带 style 复制/行断 ws 声明链，事件契约 beforeinput→textInput→mutate→input，insertText `\n` 等价路由，defaultParagraphSeparator，键盘 send_keys+Actions 双路接线）；同驱动同参语料账：cannot-have-div **0P/420F→420P/0F 全扫**、white-space-style **36P/1068F→616P/488F**、inline-host **0P/880F 封顶**（既有融合序列化 attr 层 gap：attr 读 pending 感知 vs 序列化回落基树快照两层不同步，HEAD stash A/B 同证；失败行唯一 delta=stale style 属性，语义结构零失败，下轮首位候选）；void 2P/70F DEFERRED；run 面 insertparagraph 回归核对与门禁见 [evidence](editing-compat/evidence/2026-10-31-insertparagraph-cluster-land.md)
**执行模式**: WPT 驱动（上游 editing corpus 为验收标尺）+ 长线实现推进
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——编辑面）


> **当前状态（2026-10-31，shift×normal 幽灵 br 碰头仲裁轮）：同洞两面撞车定案——part04 同面机制与 bridge 翻译歧义守卫两笔撤改采纳对方语义零残留，保留 pipeline rebind 歧义守卫 + 引擎钉测；终树 38772a918 组合态复验全绿，join-different 残余 T 定谳预算边距截断**。
> 落地两笔（净树）：pipeline `rebind_handle_node` 歧义守卫（selector 新 doc `query_selector_all != 1` 跳过重绑——堵刷新环 `query_selector` 首见猜测锚定把 pn 写歪的污染源头，RemoveHandle 恒 lenient 落空）+ 引擎钉测 `rebind_handle_node_skips_ambiguous_selector_ihost_ghost_br`（唯一 selector 正控回填 + 失唯一跳过）。撤改两笔：part04「元素 remove 无 id 臂择路」采纳对方 345140946 宿主镜像补发版本；bridge 翻译歧义守卫（失唯一跳过门）组合态复验证伪撤下——join 语料依赖翻译臂失唯一首见命中语义，唯一性门致引擎树滞留、join-different 重型变体挂起（2T→1T）。
> join-different 残余 T 归因：对方树基线 aa4e9d190（零本流改动，独立 clone 构建）同语料直跑 **3455P/0F/1T**；五轮超时变体逐轮漂移（forwarddelete pre-line×pre / pre×pre-line / select-boundary nowrap×pre-line / pre-wrap×pre / 终树 backspace pre-line×pre-wrap + forwarddelete pre-line×pre），无固定受害者 = R5342 自适应墙墙沿随机截断；所有轮次 0F——非回归，记档挂账（全分母复验需单语料直跑或调高 CASE_WALL_S）。
> **门禁（终树电池，单干净二进制逐文件分跑，systemd MemoryMax=24G）**：inline-host 880P/0F、join lrlp 2607P/0F、join lprl 1455P/0F、chd 420P/0F、wss 1104P/0F、ilb 552P/0F 逐位恒等；join-different 3447P/0F/2T（0F，2T=墙沿随机截断第五轮新变体集）；engine lib 3,626P/10F（10 已知红逐名 identical）、webview lib 830P/0F（电池账见 [evidence §5.3](editing-compat/evidence/2026-10-31-ihost-shift-ghost-br-rebind.md)）。
> 明细与归因链见 [evidence/2026-10-31-ihost-shift-ghost-br-rebind.md](editing-compat/evidence/2026-10-31-ihost-shift-ghost-br-rebind.md)。
> **下一步**：① 簇 A glue 残余、execCommand 簇序 → ② run 面残余大簇按现行账重排（renderer R383a 对账已由并行流 R54xx 轮落地收口；合并树复测抽检零位移）。
>
> **上一状态（2026-10-31，R5395 轮）：②c inline 盒 wrap 拆分——wss 912P/192F→**1104P/0F 全扫**（wrap 族 +192 全清，insertparagraph-with-white-space-style 语料整体清零）**。
> 规范事实（语料注释即 spec）：inline 段落直接拆分是「duplicate the inlined element」，Chromium 包进新匿名块且 separator **恒 div**（p 不能含 p/div，忽略 defaultParagraphSeparator）。实现（b380a98d6，part01 +31 单分支一次过）：②c 臂落 `_zwCeParagraphMutate` ② 内 ②b 后——`hostBlockish2b`（与 ②b 共用，hostTag==='div'）且壳为块标签且文本边界：at start → 新 div 块（壳副本+占位 br）插原盒**前**原盒不动；其余 → 头段留原盒、尾段进壳副本（尾空补占位 br）新块插原盒后；非 div 宿主仍落 ③（chd 期望行断形）。
> **门禁（净树）**：wss **1104P/0F 全扫**；ilb 552P/0F、chd 420P/0F、inline-host 880P/0F、join 7,518P/0F、bold run 1,138P/1,910F 零位移；engine lib 3,617P/10F 已知红集 md5 identical；webview lib 830P/0F。
> 明细见 [evidence/2026-10-31-inline-box-wrap-split.md](editing-compat/evidence/2026-10-31-inline-box-wrap-split.md)。
>
> **上一状态（2026-10-31，pre `{}<br>` 簇收口轮）：setupEditingHost 标记删除同 turn 双入账互斥修复——inline-host 720P→**780P/100F**，pre 簇 60F 全翻零位移**。
> 根因（apply trace 实证）：语料 setupEditingHost 的 marker 删除（`node.remove()`）对 D1 印章
> 文本节点（`__zwSelPathWrite`）同 turn 触发两条 wire——R5340 臂（part04）`RemoveChildAt(span,0)`
> + 印章 accessor 自带移除面（part03）`RemoveChildAtPath(span,[0])`；按发射序 apply，第一笔删
> T"{}" 后第二笔 path 末位 0 号位=同父 `<br>` → br 引擎侧消失 → CE 空宿主插双 breaker
> `[T("\n\n")]` ≠ 期望 `[T("\n"), BR]`。手工探针 t18-c 此前不现形因 `await tick()` 已 drain
> journal、`alive()` 为假→印章面跳发——差异在删除与 drain 时序非删除方式；R5058 嫌疑排除
> （只丢锚不删 br，t19 实测）；R5356「30 enter+130 shift-enter 补断落点」的 enter 侧 30F 实为
> 本簇。修复=part04 R5340 臂 wire 发射加 `!child.__zwSelPathWrite` 让位（桶 retract/通知不动，
> 与 D1 臂「身份印排除」同一互斥先例，+9/-1）。
> **门禁（修复二进制，逐文件分跑）**：inline-host **780P/100F**（F→P 60 / P→F 0，剩余 100F
> 逐行核对全部 shift×normal 补断簇）；join 三案 3456+2607+1455=7,518P/0F、
> cannot-have-div 420P/0F、wss 616P/488F、ilb 0P/552F 逐位恒等；engine lib **3614P/10F**
> 已知红恒等（fetch 契约簇 9 + s24 渲染流债，无编辑面红）；webview lib 825P/0F。
> 最小复现页修复后 wire=`[T("\n"), BR]`/`\n<br>` 与语料期望逐字一致（补断未过发：br 存活后
> `brk3.nextSibling` 非空）；探针两页保持未跟踪。
> 明细与归因链见
> [evidence/2026-10-31-ihost-pre-br-double-remove.md](editing-compat/evidence/2026-10-31-ihost-pre-br-double-remove.md)。
> **卡点/剩余**：inline-host 100F 单簇=shift×normal 补断落点（`<br>` 落宿主 span 之后
> container 子级，EXP `</span>` 前），insertLineBreak 落位锚点语义。
> **下一步**：① inline-host shift×normal 100F 补断落点归因收口 → ② insertlinebreak-with-
> white-space-style 0P/552F → ③ white-space-style 488F（italic/bold 组）→ ④ part04 BUILD
> void 挂账（70F）→ ⑤ renderer R383a 对账 → ⑥ 簇 A glue 残余、execCommand 簇序。
>
> **上一状态（2026-10-31，R5356 双缺陷收口轮）：inline-host 710F 主簇归因闭环 170P→720P/160F 零新败——过时 R5273 单文本种子 + Actions 修饰键字面插入两簇叠加剥离**。
> ① engine shim（e9fce9466）：`_zwRegisterTextEl` 条目加 `gen` 代际戳（part06）+
> 序列化 walk 内种子过时判定「`_zwSerSkipCrossGen` ∧ gen≠当前 ∧ 配对环境」放行改读
> 基座（part05 R5273 种子块）——归因链：setupEditingHost 纯文本登记 span→T:"abcd" →
> CE enter 拆文本经 R48 写路径**原地**清空注册节点 data（引擎树 apply 后已
> `[br, T:abcd]`，引擎快照读正确）→ `removeAttribute(style)` 开 attr 祖先融合臂 →
> R383a 跳并拦桶修正 + R5273 无条件吐过时单节点 → container 丢子。live 视图与
> engine 纯测试环境不变；② wpt-runner（f93f9484b）：缺陷 1 揭开后残余 470F 字节对比
> 全部 got = 可接受项 + 前缀 U+E008——Actions 键盘链修饰键 PUA 码点（E008/E009/
> E00A/E03D）落单字符 InsertText 分支字面插入；修 keydown/keyup 臂补 WebDriver 键名
> 映射 + 修饰键步早退。**FIXME 留置**：Actions 无修饰键状态，块级 CE 宿主 kShift+Enter
> 同汇 Submit（语料只压行内宿主）。
> **门禁（干净树/等价语义树）**：join 三案 7,518P/0F、white-space-style 616P/488F 与
> cannot-have-div 420P/0F 零位移、engine lib 3612P/11F 已知红逐条 identical、webview lib
> **824P/0F 新基线**（旧 r3259 红已被并行流 s170 修掉）、inline-host 终树复验 **720P/160F**、
> 对原 710F 集 comm -13 = 0 新败；插桩树记账说明（join/wss 出插桩二进制，其余剥离树复验）
> 见 evidence。**rebase 撞车合并组合态复验**（远端 7721142c4 ghost text ①簇碰头仲裁轮，
> shim 面零碰撞）：join 7,518P/0F、wss 616P/488F、chd 420P/0F、inline-host 720P/160F
> 零新败、engine lib 3614P/10F=远端新基线（r113 已销账）、webview lib 825P/0F，全绿。
> 明细与归因链见
> [evidence/2026-10-31-r5356-stale-seed-and-actions-modifier.md](editing-compat/evidence/2026-10-31-r5356-stale-seed-and-actions-modifier.md)。
> **卡点/剩余**：160F 单一机制=可见性补断 `<br>` 落宿主 span **之后**（container 子级）
> 而非内部（EXP `<span…><br><br></span>` vs GOT `<span…><br><br></span><br>`；30 enter +
> 130 shift-enter），属 insertLineBreak 落位锚点语义。
> **下一步**：① inline-host 剩余 160F 落位锚点归因收口（碰头补记佐证：**分簇**——
> shift×normal 100F 补断落点；enter×pre* 30F 引擎线 br 消失疑 part04 R5058，「单一机制」
> 概括对此 30F 存疑）→ ② insertlinebreak-with-white-space-style 首录 0P/552F
> （insertLineBreak 语义族，碰头补记轮新增分母）→ ③ white-space-style 488F
> （italic/bold 组）→ ④ part04 BUILD 路径 void 处理挂账（70F）→ ⑤ renderer 私有队列
> 路径 R383a 对账（挂账）→ ⑥ 簇 A glue 残余、execCommand 簇序。
> **碰头补记（同日，本流 inline-host 断行主簇轮）：独立实现同判撞车，采纳 R5356 先落地版本，本流同面机制撤改零残留**。本流自同基线 170P/710F 独立归因同两根因并独立实现（R5273 遮罩换代分叉——
> t12 探针证 rust 读链各时点全对、仅 getter 融合序列化陈旧；Actions 臂修饰键 PUA 抑制——
> R5356 键名映射+早退的子集），独立验证终点 **720P/160F 恒等**；rebase 撞车按「消费域收敛、
> 不回退对方修复面」采纳 3b25bb2c7+78601a924，本流 part05 跨代遮罩分叉（含 live 视图内容
> 相等退役臂）、part06 gen 戳、runner 护栏三处撤改零残留——live 视图语义分叉留档：R5356 判
> live 种子恒保留为真值，本流相等护栏为已验证候选臂（live 出过时种子反例时启用）。独立佐证
> 与增量诊断见 [evidence](editing-compat/evidence/2026-10-31-ihost-r5273-mask-and-actions-pua.md)：
> ① A 簇语料 finally 清场 → 逐测真实非页内级联 + t16 新页 shift 正确 → 状态/落位依赖；
> ② enter×pre* 30F 引擎线 `[T("\n\n")]` 既有 br 消失（嫌疑 part04 R5058 文本 insertBefore
> 退化丢锚）；③ **首录 insertlinebreak-with-white-space-style 0P/552F**（合并树复跑见下轮账）；
> ④ runner 实证 testharness-editing-other **多文件调用只跑首个文件**（join 三案分跑
> 3456+2607+1455=7,518 对账教训，位移核对须逐文件分跑）。

>
> **上一状态（2026-10-31，ghost text + 域收缩轮，与并行流 R4620 撞车合并）：inline-host 渗漏根修 80P→170P/710F 零新败 + R383a 域收缩吸收 `__zwCeWalkMerging` 旁路臂**。
> ① R129 ChildNode.remove sel 父形态跨批补发宿主镜像 `__zw_remove_child_at`（56045919e，
> 基底缓存索引 + 跨代际门；视图半区补偿 R294 两版证伪不落）；② R383a 跨代际桶跳并收缩至
> 序列化 walk（cb3cff932，`_zwSerSkipCrossGen` 标志）——本流 bisect + zz-probe-lrlp 三轮
> 循环 + `ZW_DBG_LOG` 打点独立归因，与并行流 R4620 union 树 bisect **同源同判**（attr 门轮
> 桶级早退致盲 live 视图消费面）；rebase 合并取域收缩单门（CE walk 不置序列化标志，覆盖
> R4620 诉求，按消费域收敛先例不回退对方修复面，`!__zwCeWalkMerging` 臂移除）。
> **门禁（本流分支账）**：engine lib 3611P/11F 已知红集 identical、join 三案对 22b4f1f2a
> 基线 fail 集逐位 identical、white-space-style 616P/488F 与 cannot-have-div 420P/0F 零位移、
> inline-host **170P/710F**（710F ⊂ 800F 零新败）；探针 zz-probe-abc t1–t4、zz-probe-lrlp
> 三轮循环全 Pass。**组合态（域收缩 × lrlp 两轮）复验**：join 三案 **7,518P/0F 全扫保持**、
> inline-host **170P/710F**（= 分支账零新败）、white-space-style 616P/488F 与
> cannot-have-div 420P/0F 零位移、engine lib 3612P/11F 已知红集 identical、webview lib
> 822P/1F 已知红唯一。
> 明细与归因链见
> [evidence/2026-10-31-inline-host-ghost-text-host-mirror.md](editing-compat/evidence/2026-10-31-inline-host-ghost-text-host-mirror.md)。
> **卡点**：join 三案全扫无一级簇；**inline-host 710F 主簇**——insertLineBreak/insertParagraph
> 行内 host 行断插入语义（LF+`<br>`）缺失，display 五型对称分布。
> **下一步**：① inline-host 710F 主簇收口 → ② white-space-style 488F（italic/bold 组）→
> ③ part04 BUILD 路径 void 处理挂账（70F）→ ④ renderer 私有队列路径 R383a 对账（域收缩后
> 仅序列化域跳并，范围收窄，挂账）→ ⑤ 簇 A glue 残余、execCommand 簇序。按语料账重排。
>
> **碰头补记（同日，本流 inline-host 轮）**：本流曾独立实现同根修复（R129 sel 父臂
> RemoveChildAt 寻址 + 代内墓碑视图过滤 + extractContents 摘除帧 R47 同帧扩展 +
> `_zwTextCreateData` 建时数据登记，独立验证 80P→160P/720F、engine lib r259/r2823
> 回归 bisect 修正），rebase 撞车按「消费域收敛、不回退对方修复面」采纳上方先落地
> 对方版本，同面机制撤改零残留——本流 r259（消零隐藏 sim 中间态 pending-added 文本）
> 与 r2823（建时播种遮蔽 splitText wire 写）bisect 独立佐证对方「视图补偿不落」决策；
> part92 引擎 apply 层钉测
> `ihost_cross_batch_text_remove_handle_resolves_via_persistent_nodes` 保留（双方修复
> 共同依赖的引擎侧前提）。归因链与撤改面清单见
> [evidence/2026-10-31-ihost-cross-batch-text-remove.md](editing-compat/evidence/2026-10-31-ihost-cross-batch-text-remove.md)。
>
> **上一状态（2026-10-31，lrlp normal 降级轮，并行流）：join 三案 7,518P/0F 全扫（100%）**。
> 根因修正：壳保留判定 = **源行有效 ws ≠ 落位上下文有效 ws** 才需要壳（非无条件
> normal 降级——paragraphs 案 accept 数组恒 `style="${rightWhiteSpace}"`，落位左块
> 恒异恒壳；lrlp 落位宿主 normal 上下文，right=normal 等值 → 裸 `def`/`<b>def</b>`，
> 264F 唯一 delta）。三笔全在 part01 `_zwCeJoinApply`：落位修正上提 +
> `_wsNeedsShell` 等值判定、`_stampWs` 早退 + 文本节点裸迁入、裸文本壳序列化臂
> （转义 textContent → insertAdjacentHTML）。**门禁**：对 fix13 逐测试 ID F→P 264 /
> P→F 0 零回归；insertparagraph 三案逐位恒等；engine lib 3,611P/11F 清单恒等、
> webview lib 822P/1F 已知红唯一、fmt clean。见
> [evidence/2026-10-31-lrlp-normal-ws-shell-degradation.md](editing-compat/evidence/2026-10-31-lrlp-normal-ws-shell-degradation.md)。
>
> **上一状态（2026-10-31，lrlp 壳丢失 + 撞车仲裁轮，并行流）：join 三案 7,254P/264F（96.5%）零回归**。
> 三段账：①上轮 429 遗留 R4620 落位两笔验证——InsertBefore ref 预解析兜底（plain 壳
> iah 兄弟插入后移同 ref 索引 → handle 壳 ref 失配 lenient skip 壳丢失）+ 壳 caret
> 落位钳制（融合视图缺席 IndexSizeError 整批 wire 不 drain），lrlp 396F/方法→88F/方法
> （对 fix10 F→P 924/P→F 0）；②union 树三跳 bisect 归因 daf3bc4c5 R383 桶级跳并——
> CE delete 族对象 identity 消费融合视图，R382 条目级确认被前置短路 → 整动作 no-op；
> 修 `__zwCeWalkMerging` 域门（按消费域收敛，不回退对方修复面；**ghost 轮 rebase 后由
> `_zwSerSkipCrossGen` 域收缩吸收，见当前状态**），join 对 fix11 零翻转、
> insertparagraph 三案零位移；③webview 钉子 `==1` 欠账认领（0920f3fd8 配对面扩至 3 处、
> make test 中止 workspace 段掩盖），改 `>= 3` 存在性钉复绿。
> **门禁**：engine lib 3,611P/11F 已知红恒等、webview lib 822P/1F 已知红唯一、
> fmt/clippy clean。明细与撞车 bisect 全表见
> [evidence/2026-10-31-lrlp-shell-loss-and-r383-collision-adjudication.md](editing-compat/evidence/2026-10-31-lrlp-shell-loss-and-r383-collision-adjudication.md)。
>
> **上一状态（2026-10-31，attr 门轮）：融合序列化 attr 层 gap 落地——inline-host 0P/880F → 80P/800F 解封**。
> 四笔共享路径修复：① `__zw_pending_attr_sels` 桥（journal 未应用尾 attr 变更去重清单，
> `len:cursor` 版本戳）+ 融合序列化门第三臂（R380/R3254-E2 outerHTML sel 分支同步）——纯
> attr 变更开门，读写两层同步；② R383 跨代际桶不并入 overlay（apply 落座条目基底 + 幽灵
> 双计，t4 实证 4 br）+ **R383a apply 配对环境闩**（游标 >0 置闩恒跳并；engine 纯测试环境
> 游标恒 0 恒并——t8r-4 臂②裸 bump 语义保护，A/B stash 实证唯一翻红案；**域收缩修订见
> 当前状态**——闩语义保留，桶级早退限序列化域）+ webview 两
> apply 点游标即时发布（R348 重绑前窗口收口）；③④ `__zw_contains` truthy-"0" 双修（本轮
> 新臂 + 既有 PSD）——`"0"` 真值使非前缀键恒命中，语料慢 6×+ 与 `<br></br>` 非闭合串页级
> 扩散的第二根因，均改 `=== '1'`。**中间账留档**：R383a 首版 per-parent 已应用结构清单方案
> 退化 20P/860F（动态 host = handle 寻址父，sel 键清单覆盖不到），已撤桥收敛为闩。
> **门禁**：engine lib 3617P/11F 失败集与 HEAD 逐条 identical（t8r4_k3 复绿）、webview lib
> 822P/1F 已知红、fmt/clippy clean；white-space-style 616P/488F、cannot-have-div 420P/0F
> 零位移。跨域电池 dom/selection 面终账见
> [evidence/2026-10-31-attr-gate-land.md](editing-compat/evidence/2026-10-31-attr-gate-land.md)。
> **上一状态（2026-10-31，attr 门轮之前）：insertParagraph 簇落地——live-DOM 降解模型上线，cannot-have-div 全扫**。
> 本轮换代：旧 flat outerHTML-rewrite 模型（host 分离、turn 内不可见、无降解规则）→
> **live-DOM in-place 三分支降解**（part01 `_zwCeParagraphMutate`）：① host div + block-ish +
> host 级 caret → separator 元素（defaultParagraphSeparator，默认 div）分家，空侧 `<br>` 填充，
> 内容头 caret 空占位插前；② blockish 后代（显式 display:block 或无声明 + div/p/h1-h6/li）
> 同标签分家复制 `style` 属性；③ 其余行断——white-space 声明链（声明 normal 即停）判 `\n` vs
> `<br>`，inline 子代无自有声明 forceLf（Blink quirk）。事件契约 beforeinput→textInput(`\n`)→
> mutate→input；insertText `\n` 等价路由；键盘 send_keys + Actions keydown 双路接线（FIXME：
> 无 modifier payload，Shift+Enter 未区分）；execCommand 臂换调核心 + defaultparagraphseparator
> 案；`__zw_ce_enter` 折叠 caret 尾部双写顺手修（同根因共享路径）。
> **语料账（同驱动同参 apples-to-apples）**：cannot-have-div **0P/420F → 420P/0F 全扫**；
> white-space-style **36P/1068F → 616P/488F**（残余主属 italic/bold temporary-inline 保留组）；
> inline-host **0P/880F 封顶**——既有融合序列化 attr 层 gap（HEAD stash A/B 同证）：
> attr 读路径 `__zw_has_attr_lw` pending 感知返 false，而 outerHTML 序列化门（part04 桶非空/
> subtreeDirty 才开）对纯 attr 变更不开 → 回落 `__zw_get_outer_html` 基树快照 stale；
> 失败行唯一 delta = stale style 属性，语义结构零失败——**下轮首位候选**；void 2P/70F
> DEFERRED；child-of-head/-html/non-splittable 三案 runner 零结果行（r5342 基线同，非本轮回归）。
> 门禁：engine lib 3619P/9F 已知红集零新增、webview lib 822P/1F pre-existing、fmt/clippy clean、
> bench-gate 见 evidence §6；run 面 insertparagraph 逐测试 ID 对照 **F→P 61 / P→F 2**（净 +59，
> -638 为重型变体预算截断，终版为基线前缀子集）。探针矩阵 + A/B + 失败行逐条核验明细见
> [evidence/2026-10-31-insertparagraph-cluster-land.md](editing-compat/evidence/2026-10-31-insertparagraph-cluster-land.md)。
> **下一步**：① 融合序列化 attr 层 gap（inline-host 880F 解封，D1 族 attr 维度）；
> ② join 精细化三族（inline run 残壳/host 级位次/块间 \n 几何）与簇 A glue 残余
> （s111 臂/CE loop/removeChild trap）；③ white-space-style italic/bold 组、void-element、
> 空 li no-op（run 面仅余 2 处 P→F 根因）、Shift+Enter modifier。按语料账重排。

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
> 【已收口 2026-10-30 四轮：420 下限落地、误杀案新二进制直跑 3F 归
> createLink 簇、三分片终账 39,885P/47,804F/0T——见 evidence 终账节】
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
> ——**修复已落地（同日第 5/6 轮 tape 插桩 + R263b/R262b）**：villain =
> R183 range 注册表（8192 环形只进不出）非空时 R263/R262 gate 后无条件
> childNodes O(n) refresh + 索引扫 → 每 append O(n)（探针 R=1 即足）。
> 修法 = pass1 保守超集预筛门（无命中直返）+ 原路径逐字保留，SPAN/IH/CLONE
> 3.6-9.3×、RESET 10.5×；bold 1138P/1910F 零位移 + make test 零新增 +
> ranges A/B 零 verdict 翻转，见
> [evidence/2026-10-30-cluster-a-r263-ring-gate-fix.md](editing-compat/evidence/2026-10-30-cluster-a-r263-ring-gate-fix.md)。
> 残余（下轮候选）：s111 script 扫描臂 34-46µs/append、CE loop 20-40µs、
> removeChild trap glue ~390µs/次（S8 比值 2.7 主项）；R5342 后本簇为纯
> 性能余量项，语料已解锁不受其阻。

> **white-space join 语料收口（2026-10-31，join 三案 0P → 3,108P/4,410F）**：
> D1/D2/D3 落地后 join 语料从管线阻塞中解锁。**根因定型（H 探针五步实证
> 链）**：CE 动作 `querySelector` 产物与页面脚本持有的 host wrapper 分属
> 同 context 内**两族 wrapper 对象图**——页面 setup 的 innerHTML 写走页面
> 家族后，query 家族子树视图滞留上一动作形态（kid1=上轮 shell span），
> fused 序列化读引擎态恒 fresh 掩盖陈旧 → 动作 walk 在陈旧树上早退
> （g6 守卫/recorded=0），同页第二动作起成片 no-op（guideline #32 多族
> 包装的新面：家族**子树视图滞后**，非仅 identity 失配）。修复链四笔：
> ①R380 用户动作私有批 apply 后换代通知（executor/sandbox 两分支对偶，
> 双计根因）；②`_zwCeCaretAnchorStale` + remap 门拓宽（stamped 但锚落点
> 非原对象同样重解析）；③`__zw_ce_caret_range` 世界同步（raw 选区锚链上
> 的 CE 宿主非 el 本体时以页面家族 innerHTML 回灌 el——两家族 fused 同源，
> 内容一致时幂等重建）；④plain 寻址补全：`_zwPosSelOf` 位置串渲染
> （元素序 nth-child 链，与引擎 structural fastpath 同口径）+ part04
> insertBefore plain-ref 臂 + join plain 壳序列化收敛（InsertAdjacentHtml
> beforebegin/beforeend，引擎侧 remove+copy=move）。量化：fix1 R380
> 1,044P → fix4 壳形态 1,278P → fix6 世界同步 1,701P → fix7 plain 壳
> **3,108P/4,410F（41.3%）**，逐测试 ID 对拍 fix6/fix4 均零回归；分方法
> backspace 502→1,228、forwarddelete 406→940、select-boundary 370→940
> （后两臂同消费 join apply，④两臂同收）。连带面：世界同步暴露 plain 壳
> append 缺口曾致 left-paragraph-and-right-line 363 案回归，④第三笔清零。
> 剩余长尾：forwarddelete/select-boundary 各 ~800F（行内壳变体+真跨块选区
> defer）、`<b>[]def<br>ghi</b>` backspace 形态。全文见
> [evidence/2026-10-31-join-corpus-family-resync-and-wired-addressing.md](editing-compat/evidence/2026-10-31-join-corpus-family-resync-and-wired-addressing.md)。
> 下一步：行内壳变体（b/i/span 嵌套）join 形态、真跨块选区、以及 other 面
> 剩余簇（insertparagraph → execCommand）按簇序立项。

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
  420 下限重写共享，碰头信号记此）。run 面预算窗重跑收口（2026-10-30）：
  3 分片 43 案 39,885P/47,804F/0T（87,689 行），分母 -8,974 差距
  定位自旋四案、bold 新二进制零位移背书账面；误杀案收口 3 Fail 归
  createLink 簇。其后自旋四案新二进制复跑已收口（2026-10-30，零翻转、
  +687 行、残余挂簇 A——见 evidence 终账节）。当前簇序：cloneNode
  性能簇**归因已收口**（2026-10-30 四轮探针，逐节点物化 glue 定量）且
  **修复已落地**（同日 R263b/R262b 预筛门 fill/clone 3.6-9.3× + trap 层
  四笔 6e294cd8a 合并互补——globalThis miss 17.6ms/次 + R155 反查 +
  键控根验证 + CE 门，组合基座 S5 317.2ms，见簇 A fix evidence §7-§8；
  四案复测已收口 +2,113 行、缺口 -6,176，delete/forwarddelete 残余归
  glue 桶；43 案零回归电池在途）
  → white-space join（**实现已落地+根因修正 2026-10-30**：part01 CE 块界
  join 三臂（backspace 前邻 join / forwarddelete select-boundary 空界选区
  + collapsed 终点），首行提取+保留壳，探针直调三形态逐字命中语料
  expectedHTML——语义面就绪。**语料 0P 根因非 join 语义**：scoping 时
  误记 execCommand flat 模型为根因，探针证明语料三方法全走 send_keys 键盘
  管线且死在 setup——标记剥离不持久化（解析文本节点纯快照，D1）→
  send_keys 引擎动作触发视图重建标记回滚（D2）→ caret 陈旧（D3）；
  3 案 7,518F 两预算轮稳定账不变，D1/D2 修复后复测
  （[evidence](editing-compat/evidence/2026-10-30-ws-join-impl-and-pipeline-blocker.md)；
  part06 execCommand flat delete 模型仍是独立语义缺口，归 execCommand 簇））
  → **语料管线阻塞修复（D1/D2，新簇，join/键盘驱动族前置）** →
  insertparagraph → execCommand 语义逐簇立项。
- **M3+**：扩面（edit-context / plaintext-only / whitespaces / chrome-compat
  等不在 DC-1 语料范围的邻接目录）与收口标准按簇定稿。

## 依赖约束（run-rules §9）

- 编辑命令实现与 uievents（50 号已收口，IME 挂账）、selection（23 号二期）
  共享编辑域路径——失败簇跨界时按 run-rules §9 先协调归口，不双边重复立项。
- 编辑面涉及 selection/Range 的部分与 34 号 dom-expansion（dom/ranges）有
  共享底座，收敛轮核对两域基线互不回归。
