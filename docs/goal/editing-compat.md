# 编辑命令兼容 — editing/run + editing/other

**版本**: v1.17
**日期**: 2026-10-27（立项，coverage 对账驱动）；2026-10-30（M1 收口）；2026-10-30（R5331 + M2 首轮）；2026-10-30（M2 次轮双窗折叠：预算归因 + other 面复跑收口；自旋簇三分类 + R5340）；2026-10-30（M2 次轮收环：缺陷 #2 已修 R5341 + other 面墙钟 R5342，三例 Timeout 全收口）；2026-10-30（预算窗 run 面全分母重跑收口 39,885P/47,804F/0T）；2026-10-30（簇 A 归因收口：填充/物化逐节点 JS glue 定量，修复 scope 定案）；2026-10-30（自旋四案新二进制复跑收口：零翻转，+687 行，残余挂簇 A）；2026-10-30（簇 A 修复落地：R263b/R262b 预筛门，fill/clone 3.6-9.3×）；2026-10-30（自旋四案簇 A 修复后复测收口：+2,113 行，缺口 -8,289→-6,176，insertparagraph/multitest 全分母）；2026-10-30（簇 A trap 层四笔落地：globalThis miss + R155 反查 + 键控根验证 + innerHTML CE 门，与 R263b/R262b 合并，S5 317ms）；2026-10-30（white-space join 实现落地+语义证明：探针直调三形态逐字命中语料期望；0P 根因=语料管线阻塞 D1/D2/D3 非 join 语义，簇序 scoping 修正）；2026-10-30（D1/D2/D3 落地 + 第 4 应用路径漏换代通知补账 + R382 条目级 ghost 兜底，join 语料 0P/7,518F→603P/6,915F）；2026-10-31（join 语料收口：双 wrapper 家族视图滞后根因 + plain 寻址补全，0P→3,108P/4,410F 零回归，与并行流 D1/D2/D3 撞车合并——D1 写通道统一 sel-path wire + caret 两级重锚）；2026-10-31（撞车仲裁终验：一级重锚让位 + 回灌 stamping 旁路两门落地，join 语料对 fix7 7,518 案零漂移恢复）；2026-10-31（行内壳变体收敛：ws 摘离快照 + per-run 兄弟壳 + 行内 run 升位 + ended 出参，join 三案 3,108P→5,698P/1,820F 零回归）
**状态**: Active——**当前（2026-10-31）行内壳变体收敛：join 三案 3,108P→5,698P/1,820F（41.3%→75.8%）零回归**：六笔修复全落 part01.js——ws 摘离前快照（takeLineFrom wsVal 出参，B 族错印根因：先 removeChild 后记 srcEl 走断祖先链）、混合行 per-run 兄弟壳（单一外包 span 双层嵌套拒收）、行内 run 升位（fd/select 臂 dst=caret 侧 inline run 嵌套根因，壳落行容器 sibling 位）、落位序列化逐壳交替（混合 plain/handle 壳尾批逆序实证）、块间分隔空白消费（`\n` 变体 1,027 案）、`ended` 出参（**自实现即缺失**——克隆臂 r2.ended 恒 undefined，行界在壳内消费后循环越界提取；walk 形状 Node 最小 DOM 复算锚定因果）；对 fix7 7,518 案逐测试 ID **+2,590P / P→F=0**；paragraphs backspace 676F→**0**、lprl 三方法全清零、select-boundary 三案全清，见 [evidence](editing-compat/evidence/2026-10-31-join-inline-shell-family-convergence.md)；卡点=lrlp host 级 join `abcabc` 引擎双计（backspace/forwarddelete 各 ~396 案，fix7 起既有，JS 侧形状已正确、壳 insert 引擎丢失）+ paragraphs forwarddelete 316 案（dst=host 几何 per-run 文本壳落引擎丢失）；下一步=lrlp/forwarddelete 引擎收敛 → insertparagraph 精细化（inline-host 880F attr gap）→ execCommand 簇序。**上一轮（同日）white-space join 语料收口 + 撞车仲裁终验**：join 三案 **0P→3,108P/4,410F（41.3%）**零回归（双 wrapper 家族视图滞后根因 + 世界同步 + plain 寻址补全，见 [evidence](editing-compat/evidence/2026-10-31-join-corpus-family-resync-and-wired-addressing.md)）；rebase 撞车合并（并行流 D1/D2/D3）组合态曾跌 1,395P，两处域交互缺陷已仲裁终验——**登记表重锚×世界同步换代**（一级重锚 regIdx 指向换代前位次，`_zwSynced` 门让位二级 sel-path 重解析）+ **D1 stamping×回灌子树**（stamp 域移除覆盖缺口致行界 `<br>` 残留，`__zwCeWorldSyncing` 门旁路），落地后 join 语料复跑 **3,108P/4,410F 对 fix7 7,518 案零漂移**；make test engine lib 3,611P/11F=9 已知红零新增 + 2 渲染流债（r113 cssText 尾分号、s24 ua-display flip，stash A/B 实证归 rebase 拉入提交，记档待渲染流认领），见 [evidence §8-§9](editing-compat/evidence/2026-10-31-join-corpus-family-resync-and-wired-addressing.md)；卡点=行内壳变体与真跨块选区长尾；下一步=行内壳 join 形态 → insertparagraph 精细化 → execCommand 簇序。M2 次轮完成并收环；white-space join 簇**实现已落地并证明**（2026-10-30 七轮：part01 CE 块界 join 三臂——backspace 前邻/forwarddelete 跨容器空界选区+collapsed 终点、首行提取（`\n`/br 行界、inline 克隆壳随迁）、white-space 保留壳（attr 原始串→class 映射→normal fallback）；探针直调三形态与语料 expectedHTML 逐字吻合。**0P 根因修正**：语料走 send_keys 管线非 execCommand，fail 链=setupEditingHost 标记剥离对解析文本节点不持久化（纯快照无 selector/handle，D1）→ send_keys 引擎动作触发 fused-view 重同步标记回滚（D2）→ caret/listener 陈旧走分离树（D3）——全部 marker+send_keys 编辑语料同链受阻，execCommand 族（run 面）不受影响；修复切入见 [evidence](editing-compat/evidence/2026-10-30-ws-join-impl-and-pipeline-blocker.md)，**D1/D2/D3 已落地**（同日八轮，见下方状态块与[落地 evidence](editing-compat/evidence/2026-10-30-d1d2d3-land-join-corpus-acceptance.md)，join 三案 **0P/7,518F→603P/6,915F**）；2026-10-31（insertParagraph 簇落地：live-DOM 降解模型）；簇 A（cloneNode 性能）**修复已落地**（tape 插桩收口 villain=R263/R262 range 注册表 O(n²)，pass1 预筛门 + 原路径逐字保留；SPAN/IH/CLONE 3.6-9.3×、RESET 10.5×；bold 209s 计数零位移、make test 9F 已知红集零新增、ranges A/B 零 verdict 翻转；残余 s111 臂/CE loop/removeChild glue 下轮候选）。**trap 层四笔叠加落地（6e294cd8a，与 R263b/R262b 合并互补）**：①part06 `__zwSelectionFollowRemoval` 注册提升（未注册 globalThis 属性 `typeof` 解析 miss **17.6ms/次**，惰性注册页面永不注册；remDbg 17.3→0.009ms/call，p3t move -97.5%）；②part04 innerHTML CE 环抬入 `_ihPc===true` 门（批量桶子代反链缺失，per-child `_elConnected` 落 R155 全表反查 6022 次 0 中 ~2.5ms/次；spec concept-connected 父断⇒子断）；③part03 键控根验证 + indexOf 快道（S4 -87%）；④合并冲突解法保预筛全文 + rmRoot 惰性化（R351 根缓存写侧同步，直引变量惰性化后缓存 null 隐患消除）。合并基座探针三样本：**S5 317.2ms / S8 854.9ms / p1-clone 18.1ms / R155 0×3**，无叠加回退；43 案零回归 + dom/selection/web-components/html-syntax 跨域电池 systemd 在途（结果入簇 A evidence §9）。预算窗：自旋簇主体系 runner V8 看门狗 `ZW_SCRIPT_TIMEOUT_MS` 90s 预算截断（非死循环，corpus 脚本已修 300s/变体 + CASE_WALL_S 自适应），other 面同预算复跑收口 **2,404P/11,159F/3T**；三分类窗：cloneNode 前置成本性能簇（挂账）、removeChild 对 sel 父 pending 解析子静默穿透真自旋（**已修 R5340**）、runner 完成接线过早置 all_loaded（缺陷 #2，**已修 R5341**：静态尾改 load+setTimeout(0) 门控镜像原生时序，write 注入路径维持直调；回归钉两枚 + 探针双 Pass + scripting-1 verdict 流零影响）。R5340+R5341 双落复跑：预算窗三例 Timeout **全部收口**（① delete-without-unwrapping 200F/0T、② inserttext-last-mutation 64F/0T、③ white-space join 三案实为 other 面平墙 120s 过紧的外部截断假象——corpus 脚本 **R5342** other 面接线自适应墙 60+nv×15+90；R5342 后 other 面全量复跑终账 **2,406P/12,308F/0T**，Timeout 清零、分母完整、总账逐位核对：F = 基线 11,159 + R5340 解锁 1,144 + R5341 回归 5）。**run 面全分母 3 分片重跑收口（同日四轮）**：43 案全出零 case 级 Timeout，**39,885P / 47,804F / 0T（87,689 行，45.51%）**——分母对 M2-r1 15,343 解锁 5.7×、Pass 7,485→39,885（率降为分母效应）；对 M1 96,663 差 -8,974 全部定位自旋四案 delete/forwarddelete/multitest/insertparagraph（分片二进制无 R5340），四案新二进制复跑归 R5340 收敛轮；误杀案 createlink-with-selecting-img 新二进制直跑收口（3 子测试 0P/3F，createLink 未包裹 img，归 createLink 簇）；bold 全矩阵新二进制抽查逐位零位移，账视为现行；分片+续跑 idx 槽位 bug 已修（resume 跳过保序）。明细见 [evidence/2026-10-30-run-budget-unlock-rerun.md](editing-compat/evidence/2026-10-30-run-budget-unlock-rerun.md)。**自旋四案新二进制复跑收口（同日五轮）**：delete/forwarddelete/multitest/insertparagraph 以 R5340+R5341 二进制直跑，**逐测试 ID 对照零 verdict 翻转**——delete +770 行、forwarddelete +357（removeChild 自旋解锁出真实断言）；insertparagraph -174、multitest -266 为 R5340 分支开销致重型变体吞吐降 ~25%、300s 预算窗内完成测试变少的纯截断（前缀包含证明）；四案残余分母缺口（对 M1 -8,289 行）挂簇 A 修复后复测。**四案簇 A 修复后复测已收口（同日六轮，[rerun evidence 终账节](editing-compat/evidence/2026-10-30-run-budget-unlock-rerun.md)）**：fc12a1ff8 二进制单案直跑，**26,444r 13,935P/12,509F（+2,113 行）**——insertparagraph 7,195r **全分母**（script timeout 清零）、multitest 9,497r 全变体完成（-595）、forwarddelete +816（-2,093）；delete 无实质恢复（4,354r，-129 中止集位移，对 M1 -3,488）——重区实测 ~2.3s/行，瓶颈证非 range 注册表，归簇 A glue 桶（removeChild trap/s111 臂/CE loop/execCommand glue/setup parse），下轮候选同前。run 面现行账更新：**41,171P / 49,318F / 90,489 行（45.50%）**，对 M1 96,663 缺口 -6,174。**insertParagraph 簇落地（2026-10-31）**：flat outerHTML-rewrite 模型换代 live-DOM in-place 降解模型（part01 三分支——host 分家/blockish 同标签分家带 style 复制/行断 ws 声明链，事件契约 beforeinput→textInput→mutate→input，insertText `\n` 等价路由，defaultParagraphSeparator，键盘 send_keys+Actions 双路接线）；同驱动同参语料账：cannot-have-div **0P/420F→420P/0F 全扫**、white-space-style **36P/1068F→616P/488F**、inline-host **0P/880F 封顶**（既有融合序列化 attr 层 gap：attr 读 pending 感知 vs 序列化回落基树快照两层不同步，HEAD stash A/B 同证；失败行唯一 delta=stale style 属性，语义结构零失败，下轮首位候选）；void 2P/70F DEFERRED；run 面 insertparagraph 回归核对与门禁见 [evidence](editing-compat/evidence/2026-10-31-insertparagraph-cluster-land.md)
**执行模式**: WPT 驱动（上游 editing corpus 为验收标尺）+ 长线实现推进
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——编辑面）

> **上一状态（2026-10-31，join 收口之前）：insertParagraph 簇落地——live-DOM 降解模型上线，cannot-have-div 全扫**。现行态见上方状态行（white-space join 语料收口）。
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
