# 编辑命令兼容 — editing/run + editing/other

**版本**: v1.45
**日期**: 2026-10-27（立项，coverage 对账驱动）；2026-10-30（M1 收口）；2026-10-30（R5331 + M2 首轮）；2026-10-30（M2 次轮双窗折叠：预算归因 + other 面复跑收口；自旋簇三分类 + R5340）；2026-10-30（M2 次轮收环：缺陷 #2 已修 R5341 + other 面墙钟 R5342，三例 Timeout 全收口）；2026-10-30（预算窗 run 面全分母重跑收口 39,885P/47,804F/0T）；2026-10-30（簇 A 归因收口：填充/物化逐节点 JS glue 定量，修复 scope 定案）；2026-10-30（自旋四案新二进制复跑收口：零翻转，+687 行，残余挂簇 A）；2026-10-30（簇 A 修复落地：R263b/R262b 预筛门，fill/clone 3.6-9.3×）；2026-10-30（自旋四案簇 A 修复后复测收口：+2,113 行，缺口 -8,289→-6,176，insertparagraph/multitest 全分母）；2026-10-30（簇 A trap 层四笔落地：globalThis miss + R155 反查 + 键控根验证 + innerHTML CE 门，与 R263b/R262b 合并，S5 317ms）；2026-10-30（white-space join 实现落地+语义证明：探针直调三形态逐字命中语料期望；0P 根因=语料管线阻塞 D1/D2/D3 非 join 语义，簇序 scoping 修正）；2026-10-30（D1/D2/D3 落地 + 第 4 应用路径漏换代通知补账 + R382 条目级 ghost 兜底，join 语料 0P/7,518F→603P/6,915F）；2026-10-31（join 语料收口：双 wrapper 家族视图滞后根因 + plain 寻址补全，0P→3,108P/4,410F 零回归，与并行流 D1/D2/D3 撞车合并——D1 写通道统一 sel-path wire + caret 两级重锚）；2026-10-31（撞车仲裁终验：一级重锚让位 + 回灌 stamping 旁路两门落地，join 语料对 fix7 7,518 案零漂移恢复）；2026-10-31（行内壳变体收敛：ws 摘离快照 + per-run 兄弟壳 + 行内 run 升位 + ended 出参，join 三案 3,108P→5,698P/1,820F 零回归）；2026-10-31（join paragraphs 簇清零：skipWs 方向反转根因 + wire 探针定位，5,698P→6,330P/1,188F 零回归）；2026-10-31（attr 门落地：融合序列化纳入纯 attr 变更 + R383/R383a 跨代际桶门 + truthy-"0" 双修，inline-host 0P/880F→80P/800F）；2026-10-31（lrlp 壳丢失两笔落地 + R383×CE walk 撞车仲裁：join 三案 6,330P→7,254P/264F 零回归，union 树 bisect 归因 daf3bc4c5、`__zwCeWalkMerging` 域门旁路）；2026-10-31（lrlp 残余 264F 收口：壳保留等值降级——源行 ws vs 落位上下文有效 ws 判定 + 裸文本壳序列化臂，join 三案 7,254P→**7,518P/0F 全扫**，F→P 264/P→F 0 零回归）；2026-10-31（ghost text 宿主镜像补发 + R383a 桶级跳并域收缩与 R4620 撞车合并：inline-host 80P→170P/710F 零新败，域收缩吸收 `__zwCeWalkMerging` 旁路臂）；2026-10-31（inline-host ①簇碰头仲裁：并行流 ghost text 宿主镜像补发（落地 f8fe57abd）与本流独立实现（R129 RemoveChildAt 寻址 + 代内墓碑 + R47 摘除帧 + 建时数据登记）同根同判撞车，语料等额（各自独立验证均 80P→160P），按「消费域收敛、不回退对方修复面」先例采纳先落地对方版本，本流同面机制撤改零残留；part92 引擎 apply 层正确性钉测保留 + 两层根因归因与本流 r259/r2823 bisect 独立佐证「视图补偿不落」决策入库，见 [evidence](editing-compat/evidence/2026-10-31-ihost-cross-batch-text-remove.md)）；2026-10-31（R5356 inline-host 710F 主簇双缺陷归因收口：序列化 walk 过时 R5273 单文本种子放行改基座 e9fce9466 + Actions 键盘链修饰键 PUA 字面插入抑制 f93f9484b，170P→720P/160F 零新败）；2026-10-31（pre `{}<br>` 簇收口：setupEditingHost 标记删除同 turn 双入账——R5340 臂 `RemoveChildAt(span,0)` × D1 印章 accessor `RemoveChildAtPath(span,[0])`，第二笔误删同父 br；修 part04 R5340 臂对 `__zwSelPathWrite` 子让位，inline-host 720P→**780P/100F** 零位移）；2026-10-31（insertlinebreak 臂接入 + 共享路径双修：execCommand 零变更臂补分派 + 元素边界 offset 位插入 + 同型拆分头段全切摘空节点，ilb 0P/552F→**528P/24F**，wss 616P→**848P/256F**）；2026-10-31（typing-state 追踪 + div 宿主 inline 盒拆分 + 占位 br 消费：ilb 528P/24F→**552P/0F 全扫**，wss 848P→**912P/192F**，残余收敛 wrap 族单族）；2026-10-31（②c inline 盒 wrap 拆分：wss wrap 族 192F 全清，语料全扫 **1104P/0F**）；2026-10-31（void 簇收口：querySelector 解析子树下钻 + void caret 两臂，insert-paragraph-in-void 2P/70F→**72P/0F**、insert-text-in-void 0P/48F→**48P/0F** 双语料全扫）；2026-10-31（renderer R383a 对账：配对环境判定第二通道——host 换代标记，renderer 私有队列 paired 恒 false 缺口收口，R382 兜底下未证双计形态、判定源对齐定性）；2026-10-31（shift×normal 幽灵 br 碰头仲裁：本流独立归因「陈旧裸 tag selector 重建重绑改锚——RemoveHandle 删错节点」与并行流 R5356b 345140946「RemoveHandle 换代 no-op 宿主镜像补发」同洞两面撞车（对方=滞留面，本流=错锚错删面），part04 同面机制与翻译歧义守卫两笔撤改（后者按语义保守性撤改：join 语料存在失唯一下 RemoveHandle wire、既有语义为 query_selector 首见命中移除，唯一性门使其静默 no-op 致引擎树滞留已移除节点，行为分歧风险与超时无关；当时 Timeout 差经基线复证定谳为预算墙随机截断非该门因果）采纳对方语义零残留；保留 rebind 歧义守卫 + 钉测独立面：堵住刷新环 query_selector 首见猜测锚定把 pn 写歪的污染源头（[rebind] 两跳实录），使 RemoveHandle 恒 lenient 落空，组合态复验见 [evidence](editing-compat/evidence/2026-10-31-ihost-shift-ghost-br-rebind.md)）；2026-10-31（簇 A glue 残余 S8 移除侧收口：`_zwPAIdRemove` 空表预筛 + R52/pb1 密集重建，S8 567→410ms −27.7%、S8/S5 1.62→**1.18 ≤ 1.5 达标**，bold 墙钟 ~300→117s；inline-host 宽 filter 8F 定谳既有失败非本轮引入）；2026-10-31（簇 A removeChild 融合列表同调用复用收刀：tape 归因 sel 域 removeChild 每 removal 恰 3 次 `_childNodeList` 全量物化（R126 子判定 + R5340 索引扫描 + R5340 体尾 LiveNLSync）占墙钟 98% O(n²)，R126→R5340 同调用背靠背零 mutation 段 `_r126FuseMemo` 复用落地（part04 +23 行），探针 w2 均值 6661→4109ms −38%、物化 6001→4001 次恰 −1/3，前轮 §4 ~650µs text-child 挂账定谳历史态关闭；四门零位移——engine lib 3,635P/1F s24 已知红唯一、webview lib 832P/0F、inline-host 880P/0F、join 三案 7,518P/0F/0T，见 [evidence](editing-compat/evidence/2026-10-31-cluster-a-removechild-fused-rebuild.md)）；2026-10-31（nth-child rebind 探针撞头仲裁：本流簇 A 插入侧轮独立实现的查询作用域 sibling 位置索引与 site-opt PR #263 同根同判撞车，撤改零残留存档 wip 分支，探针 A/B 42,217ms→217ms 作对方修复面编辑域佐证入库，本轮认领 execCommand 簇序）；2026-10-31（簇 A 流插入侧残余归因 + CE entry 空表快路：tape v2 细粒度归因 + s111 臂定谳历史态 + `_ceEntryFor` gen-0 快路 seg15 −27%、fill wall −7%，六门达标零位移，见 [evidence](editing-compat/evidence/2026-10-31-insert-side-attribution-and-ce-entry-fast-path.md)）；2026-11-01（簇 A removeChild 残余 O(n²) overlay 单遍化：tape v3 内部插桩钉死剩余墙钟 ~96% 在 `_zwOverlayPendingChildNodes` 两循环（added 合并 ~70% + removed 逆序扫 ~18%，基底缓存 hit=4001/miss=0 host 往返无关），三笔单遍化 removedSet 过滤 / `_resSeen` Set 判定 / 尾插 push 每调用 O(added×res)→O(added+res)，探针 w2 均值 3,832→1,506ms（tape A/B −61%）、净树 1,564ms（对上轮 4,076ms −62%、两轮前累计 −77%），十门零位移，见 [evidence](editing-compat/evidence/2026-11-01-removechild-residual-overlay-singlepass.md)）；2026-11-01（插入侧解析残余归因轮：tape v4/v4b 两级细分定谳 seg55 的 99% 在 `_zwMEl` 逐实例安装面、host 桥 + JSON.parse 合计 <2ms 免费、无隐藏二次项——安全切片收益噪声带不落代码，结构性收口与 t8r 物化面合并立项「wrapper 安装面」深水轮（own-property 语义审计清单入 evidence），见 [evidence](editing-compat/evidence/2026-11-01-insert-parse-construction-attribution.md)）；2026-11-01（CE 环 details 全树扫归因 + 成员资格快路轮：scan 非逐子触发——每个 innerHTML 物化尾单次调用 ≈9.5µs/根 handle trap 读，构建期 census 单调闩 `__zwDetailsEver4100` 只门解析/物化面，S8 scan 19ms→~0、zz-s5 wall 中位 133→107ms −20%、移动面不变，九语料 + 双 lib + bench 零位移，见 [evidence](editing-compat/evidence/2026-11-01-details-scan-membership-fast-path.md)）；2026-11-01（justify 簇首簇落地：blockExtend 两档 Chrome delta + BpCompare equality + flat 模型 splice wrap + 查询面三笔，justifycenter 真页 **4,728P/1,359F/0T**、bold +464P 零新增失败、七门其余零位移，见 [evidence](editing-compat/evidence/2026-11-01-justify-cluster-first-landing.md)）；2026-11-01（「wrapper 安装面」深水轮：`_zwMEl` 逐实例安装面共享化（part03 +2,017/−1,783，73 items），seg55+seg54 同源合并收口，zz-s5 中位 107→68ms **−36%**、zz-reset ~85→30ms、zz-ap 恒等；七语料 + 双 lib + bench + wcomp + hsem 门禁零位移，⑥ the-details-element 语料 21 案续拉销账 42P/1F，见 [evidence](editing-compat/evidence/2026-11-01-mel-wrapper-shared-install-face.md)）；2026-11-01（justify 查询面 uplink 断裂兜底轮：setter 孤儿 wrapper 读数链修复——jr/jf 各 +220P 零回归、jl 逐位恒等、jc 交集 P→F=0，jc 全分母环境截断经 pristine A/B 定谳与本轮无关，见 [evidence](editing-compat/evidence/2026-11-01-justify-query-face-orphan-uplink.md)）；2026-11-01（style-with-value 簇 setup-exception 族收口：物化元素补 style IDL——`HTMLElement.prototype` 单点 style accessor + `_zwMStyleProxy` 属性反射面板，五语料 5,056P/4,162F→**7,041P/2,177F**（F→P +1,985 逐语料恰等额、P→F 0、T 0→0），`reading 'color'` 族全清零；七位移语料 + wcomp + 双 lib + bench 零位移，见 [evidence](editing-compat/evidence/2026-11-01-sv-setup-exception-mstyle-idl.md)）；2026-11-01（style-with-value 簇值查询面收口：gCS plain 四属性解析器 + 五值命令引擎面接线——五语料 7,041P/2,177F→**7,562P/1,656F**（F→P 609、P→F 88 全数定谳 apply 缺口②下游），合并树复测五语料+九位移+wcomp+双 lib+bench 零位移、hsem +23 F→P pristine A/B 定谳并行流修复面与 :active flakes 族，见 [evidence](editing-compat/evidence/2026-11-01-sv-queryvalue-plain-computed.md)）；2026-11-01（style-with-value 簇实应用面收口：execCommand 五值命令 setSelectionValue 引擎移植（part01d 新文件）——五语料 7,562P/1,656F→**8,266P/952F**（F→P 705、P→F 1 渲染流域挂账），中途修复五笔含色三非法色返 true/钳位归一、a[href] 零 DOM 变更、enabled 端点门三笔 Chrome delta 口径，九位移语料+wcomp+双 lib+bench 零位移、hsem +8 全 F→P 逐案 commit-mapped（s203/s204/s205 并行流 + 既有 flakes）非本 diff 面，见 [evidence](editing-compat/evidence/2026-11-01-sv-apply-face-execcommand.md)）；2026-11-01（setup-exception 族查询索引失效轮：R307 attr 变体——M-form 树五个 attr 写入点补 `_zwMElQIdxInvalidate` 祖先失效，五语料 8,266P/952F→**8,717P/501F**，F→P 451 零回归，setup 异常族全清）；2026-11-01（compare innerHTML 空白族收口轮：五命令 FV 空白门 + wrap 空白携带 + br 抑制 + formattable 双向 collapsed 四笔——五语料 8,717P/501F→**8,922P/296F**（F→P 205 零回归），429 中断面 splitText 原位写验收保留 + invisible skip 撤改重构，hsem +8 定谳 s209/s210 rebase 窗吸收，见 [evidence](editing-compat/evidence/2026-11-01-sv-compare-innerhtml-whitespace-family.md)）；2026-11-01（s212 handle 子 parentNode 反链 + isAllowedChild 表族名单 + inserttext 变异核与 typing 值 override 轮：三族并发归因两批收口——五语料 8,922P/296F→**8,986P/232F**（F→P 64 逐语料 5/16/8/12/12、P→F 0），insertText 波及语料首测 inserttext 2,777P/232F、multitest 9,331P/761F、ipara 6,720P/475F、iloop 1,438P/323F、illist 1,721P/344F，双 lib + bench rc=0、hsem 648F 恒等（2↔2 :active flakes），见 [evidence](editing-compat/evidence/2026-11-01-s212-parentlink-allowedchild-and-inserttext-typing-override.md)）；2026-11-01（insertText 波及语料 F 族普查 + 状态命令 typing 面轮：五语料 2,135F 普查定案（F1 list 零应用 ~600F/F2 host-split 前段不包裹 ~490F/D1 delete 撤壳/D4 尾 br 边界/D5 形态矛盾扩面）+ sub/sup tag 表互斥与 qState typing-state fallback 落地（Dom-split 双执行修正）——multitest 9,331P/761F→**9,486P/606F**（F→P 155、P→F 0），iloop/illist/forecolor 恒等、位移电池十案逐位恒等、engine lib rc=0，见 [evidence](editing-compat/evidence/2026-11-01-inserttext-corpus-f-survey-and-state-typing-face.md)）；2026-11-02（F1 list 命令簇轮：toggleLists/getSelectionListState/splitParent/outdentNode ol-ul 臂移植 part01e 新分片 + part06 应用与读面三笔 + WrapViaHtml 脱挂返回与 tag 大小写与 mixed 判序三修正——iloop 1,438P/323F→**1,619P/142F**、illist 1,721P/344F→**1,910P/155F**、multitest 9,486P/606F→**9,538P/554F** 合计 F→P 422/P→F 0，位移电池零位移、engine lib rc=0，见 [evidence](editing-compat/evidence/2026-11-02-f1-list-command-apply-and-qstate-face.md)）；2026-11-02（F2 insertParagraph host-level split 前段包裹轮：part01 ① 臂 boundaryText 路径双块输出改造（Chrome delta——语料期望 caret 行整体成块 `<div>foo</div><div>baz</div>`，vendored splitText 语义前段留宿主裸层）+ engine lib 钉测双块语义同步——ipara 6,720P/475F→**6,728P/467F**、inserttext 2,777P/232F→**2,779P/230F** 合计 F→P 10、P→F 0，multitest/wss-ip/位移电池十案恒等零位移，engine lib rc=0（钉测同步后 3,684P/0F），见 [evidence](editing-compat/evidence/2026-11-02-f2-insertparagraph-host-split-head-wrap.md)）；2026-11-02（F6 delete/forwardDelete 撤壳轮：part01f 新分片 deleteSelection+两 action 忠实移植 + Chrome delta 四笔（strip-wrappers 幸存壳/overrides 原始快照回写/幸存壳占位门/typing 包裹态快照回写），multitest 9,538P/554F→**9,590P/502F** F→P 59、P→F 7 全数挂账族，13 面 + engine lib 恒等，见 [evidence](editing-compat/evidence/2026-11-02-f6-delete-forwarddelete-shell-boundary.md)）；2026-11-02（D4 typing 拆壳重挂轮：part01 offSplit 臂（尾段手工爬升 + 非 off 链重包裹 + 占位尾段丢弃）+ part06 `__zwTypingTagsOff` 记账/_zwSync 清除 + part01f 快照扩三表——过程根因两笔（shim DocumentFragment.appendChild 无移动语义 X4 实证、尾段 null scan 抛 TypeError F3 形），multitest 桶内 F→P 11（D4 六 fixture 主翻转 F1-F5 全数 + F6 qS）、P→F 0、溢出桶 18 断言消失定谳环境截断（⊂ g18 缺失集、随无关 diff 翻转）、恒等电池 15 面 + engine lib 3,684P/0F 用例级零位移，见 [evidence](editing-compat/evidence/2026-11-02-d4-typing-off-shell-rehang.md)）；2026-11-02（ipara 忠实移植轮：insertParagraph 换代 vendored 忠实移植 + 混合派发——other 103 案逐位持平 13,134P/1,600F/0T rc=0、engine lib 3,707P/0F rc=0、run 电池收口（gate27 落地见 2026-11-03 条），见 [evidence](editing-compat/evidence/2026-11-02-ipara-faithful-port-nested-faces.md)）；2026-11-03（gate27 run 电池收口：六腿有效——insertparagraph F→P 净移 256、inserttext/insertlinebreak/forecolor 三腿逐位恒等、typing-text 6P/10F 首记账；multitest 缺 85 待 solo 归因；delete/fd 两腿零断言产出死亡无效账，直跑重跑定谳中）；2026-11-03（gate28 双壳终点边界捕获轮：五笔 c8188f02c + Edit F 整含两判方向修复 13017c0a3——E2fd/E4 判读收口 multitest −85 零行为差，探针首判 8 案 typing 面裸奔→capture 恒空归因→修复→终判读 delete/fd 面 35/35 全 PASS，10:09 放 zw-g28-battery，双实例碰头仲裁乙让位）；2026-11-03（multitest 腿 10:40:43 落地判读**不过**：P→F 22 全同形 `[foo<shell>bar]</shell>baz`+delete+inserttext——got 恒吞 typed "a"（exp `abaz` got `baz`）＝五笔 delta ① 链外边界 keep-shell 置空形态，探针缺 offset-0 起点形漏拦；F→P 23 翻转正常，余腿续跑全账）
**上一状态（2026-10-31，shift×normal 幽灵 br 碰头仲裁轮）：part04 同面机制与 bridge 翻译歧义守卫两笔撤改采纳对方语义零残留，保留 pipeline rebind 歧义守卫 + 引擎钉测（38772a918）**。
> 定谳：本流「陈旧裸 tag selector 重建重绑改锚——RemoveHandle 删错节点」与并行流 R5356b 345140946「RemoveHandle 换代 no-op 宿主镜像补发」同洞两面撞车——补断恒落 span 内部，`</span>` 后可见 `<br>` 是语料自插 brElement 幽灵（刷新环 `query_selector` 首见猜测锚定把 pn 写歪，[rebind] 两跳实录）；撤改翻译歧义守卫按语义保守性（join 语料存在失唯一下 RemoveHandle wire，唯一性门静默 no-op 致引擎树滞留已移除节点；当时 Timeout 差经基线复证定谳为 R5342 预算墙随机截断非该门因果）。
> join-different 残余 T 归因：对方树基线 aa4e9d190 直跑同得 3455P/0F/1T，四轮超时变体逐轮漂移无固定受害者——3,456 变体恰在 R5342 自适应墙沿的随机截断，0F 零失败，按预算边距记档不作回归挂账。
> **门禁（终树 38772a918 单干净二进制逐文件分跑）**：inline-host **880P/0F**、join lrlp 2607P/0F + lprl 1455P/0F、chd 420P/0F、wss 1104P/0F、ilb 552P/0F 逐位恒等；join-different 3447P/0F/2T（0F）；engine lib **3,626P/10F**（10 已知红逐名 identical）；webview lib **830P/0F**。合并树复测（再并入 R54xx/s177/s178 九笔，1155ace7f）：inline-host 880P/0F、join-different 3454P/0F/1T、engine lib 3,626P/10F 零位移。
> 明细见 [evidence](editing-compat/evidence/2026-10-31-ihost-shift-ghost-br-rebind.md)。

**状态**: Active——**当前（2026-11-03，gate28 双壳终点边界捕获轮，电池在跑）**：五笔 delta（c8188f02c）+ **Edit F 整含两判方向修复（13017c0a3 已推送）**；E 腿判读收口——**E2fd fd 腿 6,809P/250F/2T**（32.7 min 干净全量，fd leg8 有效账成立；差 gate21 6,831P/258F/1T = −22P/−8F/+1T 恰 ?6001-7000 新增截断片 flake）、**E4 multitest 腿 9,546P/428F/1T**（同 bin 切片级 diff：?9001-last 外全部切片逐位恒等、fail 集 ⊆ 电池腿——**multitest 缺 85/110 = ?9001-last 页超时截断点漂移 flake 族，零真实行为差**），见 [evidence §4](editing-compat/evidence/2026-11-03-dshell-endpoint-capture.md)；prep 双探针首判 8 案 typing 面未翻转（壳摘 ✓ caret 落位 ✓ 但 typed 段裸奔）→ zz-probe-dshell3 三段二分定位 capture 直调恒空 → 根因 `_zwDelFirstVisibleInRange` 整含两判方向全反（`_zwDelGetPosition` 首参相对次参方位下 'after'/'before' 反写，判定实际要求候选横跨全选区恒无候选）→ **Edit F 修复 → 语料终判读 delete/fd 面 35/35 全 PASS**（C01/C02/C05/C06/C11-C14 typing 重裹逐字 `abc<b><i>def</i></b>ghi`；页直驱与语料驱动双绿，prep 后出现的「harness 驱动 vs 页直驱分歧」假设作废）+ 残余 21F 全 ipara/ilb 独立轮面（第十五轮认领）、dshell2 九案 DOM 全合修复预期（M1/M3/M5 caret=div@1 系 t:abc@3 边界等价点表示差异，探针表期望待改非引擎缺陷，§6 记档）；**10:09:27 放 zw-g28-battery**（multitest → insertparagraph → inserttext → typing-text → insertlinebreak → forecolor → delete.html → forwarddelete 八腿 TIME_LIMIT=10800 → other → enginelib，~3h，gate28-done.txt；multitest/delete/fd 腿判读按 fail 行集 diff + ?9001-last/?6001-7000 截断容噪，P→F 必须零）。双实例同修碰头仲裁：乙实例独立归因逐字同判按「消费域收敛、不回退对方修复面」让位源码/探针/放电/commit 全链（COORD 十六/十七轮），甲实例（本流）认领 fix、终判读、放电与 §5 收口；family 基线行集存 `g28-bi-family-baseline.txt` 45 行 + `g28-mt-baseline-all.txt` 430 行；battery 窗口内勿改 crates/engine js 源（八腿逐腿 make 拾取中途变更毒化 leg 可比性，COORD 十七轮）。**multitest 腿已落地（10:41，9,596P/433F/0T，?9001-last 本轮全片跑完 T 1→0）**：F→P 23（`<b><i>` 双壳 delete 10 + fd 10 主目标面全翻转 ✓ + ipara img 3 连带回收）但 **P→F 26 违门**——22 案真回归 `[foo<X>bar]</X>baz` 族（?8001-9000 完整片）签名 22/22 恒一 `exp "abaz" got "baz"`（typed 段整丢：caret 落原边界两臂前提「原边界节点在删除链外必存活」对 origStart 本身在删程内的形态不成立，collapse 塌到 detached 节点致 inserttext 落空；目标面 `abc{...}ghi` 形边界在删程外恒存活故探针 35/35 不踩），+ 4 案 ?9001-last 截断新暴露跨块双壳形（`<b></b><b>gh</b>`）非回归归因；判读=主面达成、门禁违例——**gate28 批不得判绿**，修复笔（caret 两臂 + same-node strip 臂加 origStartContainer 存活门、不存活回落 legacy forward 分支；capture 臂语义正确不在修复面）待电池落地后执行，multitest/delete/fd 三腿复跑，详录见 [evidence §5](editing-compat/evidence/2026-11-03-dshell-endpoint-capture.md)。**（前一状态：ipara 忠实移植轮——insertParagraph 换代忠实移植 + 混合派发；other 13,134P/1,600F 逐位持平 = gate21，engine lib 3,709P/0F，run 电池六腿有效——insertparagraph F→P 净移 256、三腿恒等、typing-text 6P/10F 首记账；multitest 缺 85 已由 E4 收口，delete/fd 零断言死亡无效账已由 E1/E2fd 补齐）**：

**实现（part01f 新分片 +541 行 + part06 接线一笔）**：insertParagraph 从 live 就地拆分近似（`__zw_ce_paragraph_split_at`，F2 双块改造，保留供 insertText `\n` 分流与键盘面）换代 vendored implementation.js :7486-7826 忠实移植（part01f `__zw_ce_ipara_apply`：deleteSelection 前导 → 文本归一 → 单行容器爬升 → p/div 外爬 dd/dt/li → 非 SLC wrap / pre br / 空 li dt dd splitParent（rename walk 扩 li）→ 主拆分支）。shim 缺陷绕行三笔：mid-text splitText 臂弃用（shim 包装层 splitText 后选区重锚与 `.data` 双态不可信，trace 三步实证，`_zwIparaExtractRange` first-partial cloneSlice+deleteData 原生消化）；extract/insertNode 自实现（shim Range.extractContents 尾兄弟不迁移 X1、DocumentFragment.appendChild 无移动语义 X4 双父窗——contained 子迁移 removeChild 先于 appendChild 消双父窗，共享根修复仍挂账）；collapsed 守卫点等价自算（个别 delete 路径遗留 `collapsed` 字段过期，C5 实证）。**混合派发**：part06 臂 `__zw_ce_ipara_wants` 只读预检——container 为宿主内元素走移植；裸文本/宿主本体/void caret/宿主 SLC 非 div/white-space 或 inline(-block) 链回落 legacy（gate21 已知绿辖区构造上复位）；样式读数 style 面板优先 + attr 兜底（wss 语料 IDL 设样式 attr 扫描漏接，探针 v13 定谳）；part01.js IIFE 私有作用域教训本地复制 void/display helpers。endOfLine 判定补 startInTail + 第三段提升臂限 comment/0 位文本（不 splitText 下 spec 前提修正）。override 快照/重锚/回填同 `__zw_ce_delete_apply` 口径。

**门禁（gate22b→26 五批，.acceptance/ipara-port/，详录见 evidence §4）**：B1 纯移植全量 run 90,158P/6,025F/3T rc=0（run ipara F→P 244）但 other 卫星面净 −442P + enginelib 1F（F2 钉测按设计失败）；混合派发三轮收敛 other 12,805→12,834→13,005（对 gate21 恰 −129P/+129F = echd 57F + wss-ip 72F 两面，cmp-other.py 定谳）；B1→B2 style-panel 修复后 13,134P/1,600F。终批（22:39 pull 合并树 cc097dbc3 首次全量）：other **13,134P/1,600F/0T rc=0，103 案 PER-FACE identical = gate21**（wss-ip 1,104P/0F、echd 420P/0F 另 `-one` 直跑全分母双验；103 案恒 10 案 runner 筛减零输出——8 `<iframe` + 2 `.https.`）+ engine lib **3,707P/0F rc=0**（较 gate25 +8 系 pull 并入并行流补面测试，本轮零测试新增）。edob 25F/restoration 24F 定谳 gate21 既有失败。探针 zz-probe-ipara2 九族 + ipara4 十二族终批干净二进制复验全绿。gate27 终验（23:53-23:56 pull 新基 58de3d5ec+5e694d5a7，唯一记账源）：enginelib **3,709P/0F rc=0**、other **13,134P/1,600F/0T PER-FACE identical**（`cmp-other.py`，gate21/gate26/gate27 三基树逐位一致，s244/s245 并入中性）。run 电池（v1 截断账作废——`-one` 缺 `ZW_SCRIPT_TIMEOUT_MS=300000` 导出 90s 看门狗截断 insertparagraph，Makefile 内联导出根因修复已落工作区另行提交；v2 补导出 + TIME_LIMIT=10800 重跑 00:31-02:10，delete 腿修精确路径过滤）：六腿有效——insertparagraph **6,984P/211F/0T**（F→P 净移 256，ΔP=+256/ΔF=−256 恰互偿）、inserttext 2,782P/227F/insertlinebreak 1,641P/50F/forecolor 2,279P/48F 三腿逐位=gate21、typing-text 6P/10F/0T 首记账；multitest 9,569P/430F/1T 缺 85（P/F 同降=输出缺失）待 solo 切片归因；delete.html/forwarddelete 两腿零断言产出死亡（31/11 min 满载后 make Error 1、transcript 零 flush、test-guard 无击杀留痕；机制定谳（2026-11-03 03:16 勘误：本三证推断作废——runner 断言收尾批量打印，零行中期为正常缓冲形态，LineWriter 逐行即刷与应用层收尾前无行可刷不矛盾；E1/diag 双 bin 干净全量反证 + E3/E24 系外部 stop，勘误详见 evidence §4）；gate21 同通道曾完整记账 7,339P/296F/1T、6,831P/258F/1T——崩溃归属终局（03:16 勘误）：battery v1 双腿 SIGSEGV/GPF 为唯一内核实证崩溃对，判环境性/低频一次性，重跑法归因穷尽无 bisect 靶），直跑重跑归因进行中（g27-chain：del→fd→multitest solo 顺序链，落地前树上钉）。见 [evidence](editing-compat/evidence/2026-11-02-ipara-faithful-port-nested-faces.md)。

**下一步**：① gate28 电池余腿落地判读（ipara/inserttext/typing-text/insertlinebreak/forecolor/delete/fd 八腿计数对账 evidence §5 基线 + other/enginelib 逐位恒等复核；delete/fd 腿重点核 `[foo<X>bar]</X>baz` 族 P→F 扩散面，?6001-7000 截断容噪）→ ② caret 臂存活门修复笔（selExit/same-node 两 caret 臂 + same-node strip 臂加 origStartContainer 删后存活检查，不存活回落 legacy forward 分支；探针补 `[foo<b>bar]</b>baz` 回归形并复验双壳目标面 35/35 不退）→ ③ multitest/delete/fd 三腿复跑行集 diff 判读（P→F 归零、F→P 23 保持）→ ④ evidence §5 补录 + 本文收口 + pre-commit-guard 提交推送 → ⑤ qV 残余 ~109F（挂账不变）→ ⑥ list FIXME + delete li/dt/dd FIXME 臂 + shim DocumentFragment.appendChild 移动语义/Range.extractContents 共享根 + Range `collapsed` 字段过期根源（独立轮，影响面广）→ ⑦ ipara non-compare 174（in_array 63 多形态允许值 + equals 111 读面）+ :2429 空壳弃形 → ⑧ w5 void-br / jc 墙钟 / merge FIXME（挂账不变）。

**上一状态（2026-11-02，TypingStyle 挂账收尾轮，editing 流）：g19 后挂账 7 案全数翻转 + F6 嵌套尾段 FIXME 闭环；multitest F→P 59（486F→437F/1T），inserttext F→P 3，其余面恒等**：

**实现（三笔，part01.js 两处 + part01f.js 两处）**：**Fix A reanchor 层位修正**——delete 动作管线把宿主本体登记进 `__zwSelPendingTops` 时 `_zwCeRemapEndpoint` 旧码恒自 `kids[regIdx]`（el 的下一层）起走 path，整链下移一层：CE-false 语料 `(b,0)` 被走成 `b.childNodes[0]`（contenteditable=false span）@0、typed 段误入不可编辑壳（multitest :3206-:3229 四臂；zz-probe-tstylediag raw=B@0 vs caretRange=SPAN@0 + kill-switch 定谳）；修法 top===el 时自 el 起走。**Fix B 幸存壳 typing 链捕获（delta ⑤）**——留壳预扫抽 `_zwDelKeepShellOf` 共享；新 `__zw_ce_capture_typing_chain`：start 链上任一 `<a>` → 捕获空（:2163/:2168 双向实证），否则内→外收集 format 族（part06 同表 b/i/u/s/sub/sup）行至幸存壳（含）止、滤 D4 off 表、记录序外层在前；`__zw_ce_delete_apply` 动作前捕获（仅非 collapsed）、显式 typing 态存活优先捕获补位；消费侧 part01 新 `_zwCeTypingTagsAt` 于三处 ttags 读取滤 caret 祖先链既有层（幸存壳自然承接不双裹——T5 得 `<b><i>d</i></b>` 非 `<b><i><b>d</b></i></b>`；显式 toggle-on 态 tag 必不在链上，滤除恒等）。**Fix C F6 嵌套尾段重挂**——offSplit 纯占位尾段含 ≥2 br 时：首个 br（caret 行终结符）并入 typed 段壳（`<i>d<br></i>`）、其余 br 各以 off 壳 clone 回挂（`<b><br></b>`）；单 br 维持丢弃（F1-F5 形不变）。探针全绿：zz-probe-tstyle T1-T6 精确期望形、T7 qS_after=true；zz-probe-d4br F6 精确全形 `<b><i>abc</i></b><i>d<br></i><b><br></b>`、F1-F5 已锁形不变。

**门禁（gate21 批，test-guard 4G/8G，run 通道全量首次 44 文件全记账）**：multitest 9,588P/486F→**9,647P/437F/1T**（?9001-last 整变体墙钟截断 1T，前轮同款 flake 签名）；挂账行逐条：CE-false 行 Fail=0、`[def]` 全含双壳 delete/forwarddelete compare 全 Pass、:2429 qS Pass（compare 留挂）、F6 行全 Pass；inserttext 2,782P/227F（F→P 3）；ipara/forecolor 恒等。other 通道 13,134P/1,600F rc=0 锁面逐位恒等：wss-ip 1,104P/0F、wss-ilb 552P/0F、join 三案 3,456/2,607/1,455P/0F、ipihost 880P/0F、chd 420P/0F、void 72+48P/0F、run-ilb 1,641P/50F。engine lib rc=0 **3,686P/0F**。delete.html/forwarddelete.html 首次全文件记账 7,339P/296F/1T、6,831P/258F/1T——失败簇全为 collapsed 跨块合并 compare + foreColor qV（本轮臂惰性：collapsed 不捕获、qV 读 override 仓）。残余挂账：双壳终点边界捕获面（`abc{<b><i>def</i></b>}ghi` 部分含形 ~25 断言，Chrome "last visible thing" 语义）、:2429 compare（ipara 拆分臂空 inline 壳弃壳形）、T1B/D4 诊断案元素位直插 throw（非语料面）。见 [evidence](editing-compat/evidence/2026-11-02-typingstyle-reanchor-capture-rehang.md)。

**下一步**：① ipara other-cmp 153 嵌套面（table/dl/blockquote/list 内段落 split；含 ：2429 空壳弃形）→ ② 双壳终点边界捕获面（part 含形 ~25 断言）→ ③ qV 残余 ~109F（挂账不变）→ ④ list FIXME + delete li/dt/dd FIXME 臂 + shim DocumentFragment.appendChild 移动语义/Range.extractContents 尾兄弟共享根修复（独立轮，影响面广）→ ⑤ w5 void-br / jc 墙钟 / merge FIXME（挂账不变）。

**上一状态（2026-11-02，D4 typing 拆壳重挂轮，editing 流）：multitest 桶内 F→P 11（D4 六 fixture 主翻转 F1-F5 全数 + F6 qS）、P→F 0，恒等电池 15 面 + engine lib 用例级零位移**：

**实现（D4，part01 offSplit 臂 + part06 toff 记账 + part01f 快照扩三表）**：Chrome typing-style 重挂语义三则——① 拆最深 off 壳（显式 toggle-off 的 tag 仍在 caret 祖先链时 typed 段不继承该壳，`<b>abc[]<br></b>`+bold+inserttext 期望 `<b>abc</b>d` 非 in-place `<b>abcd<br></b>`）；② typed 段按壳内非 off 链 `_zwCeTypingInsert` 重包裹（F4 `<b><i>abc[]</i></b>` → `<i>d</i>`）；③ 纯占位尾段丢弃 + off 壳直后裸 br 同消费。part01.js `__zw_ce_inserttext_mutate` 文本分支 offSplit 臂：尾段手工爬升摘取（自 sc 逐级摘后兄弟至 offSplit，每级容器空壳 clone 承接、空级不建壳、文档序深层在前）+ 丢弃臂。part06 collapsed format toggle 臂 `__zwTypingTagsOff` 记账（on→off 表入、off→回正表，sub/sup 互斥同步）+ `_zwSync` 同生命周期清除；part01f `__zw_ce_delete_apply` 快照扩 tt+ttoff 三表回写（同 F6 Fix C 语义）。过程根因两笔：shim DocumentFragment.appendChild 无移动语义（X4 实证 host 桥 append 不摘源父——同根解释 extractContents 尾兄弟不迁移 X1/X2，爬升改显式 removeChild 绕过，共享根修复独立轮挂账）+ 尾段 null 时 `_zwD4Scan` 抛 TypeError（F3 形臂中断、dispatcher 吞异常回落 `d<br>` 滞留）。

**门禁（gate20 批，systemd user unit + test-guard）**：multitest 9,590P/502F→**9,588P/486F**——D4 六 fixture 桶内主翻转 F1-F5 compare+qS-after 全数 Fail→Pass（11 断言恰 F→P 全额）、F6 compare 保持 Fail（挂账一致）+ qS Fail→Pass；主 case 行零位移。溢出桶 18 断言（5F+13P）消失定谳环境截断非回归——消失集 ⊂ g18 缺失集，随无关 diff 翻转即 flake 签名（jc 墙钟截断同款先例）。恒等电池用例级 verdict diff=0：inserttext 2,779P/230F/ipara 6,728P/467F/forecolor 2,279P/48F/wss 双语料/run-ilb/join 三案/chd/ipihost/void 双语料/iloop/illist；engine lib rc=0 3,684P/0F。语义探针 zz-probe-d4br 七案 5/6 语料逐字，qS_after 全 false。见 [evidence](editing-compat/evidence/2026-11-02-d4-typing-off-shell-rehang.md)。

**实现（F6，part01f 新分片 975 行 + part01d 两笔 + part06 调用点置换）**：vendored deleteSelection（:4550-5058）+ delete/forwarddelete 两 action（:6193-7052）忠实移植（equivalent point、canonicalizeWhitespace、块合并、recordValues/restoreValues 9 命令表、combining marks 正则）；part06 flat `_zwExecCmdApplyDelete` 删除换 `__zw_ce_delete_apply` 单点调用。Chrome delta 四笔：① strip-wrappers 幸存壳——起点行内链最外层 `<a>`（无 a 则最外层链包裹）保留、计长 0、包含态 `<a>` 整体摘（`[foo<a>bar]</a>baz`→`abaz` 无壳实证）；② preservesOverrides 等价面——part01d `_zwVaOverridesTakeRaw/PutRaw` 双仓原始快照回写 + `_zwVaReanchorOverrides` 重锚（动作内 override 读触发 staleness reset 误清，值命令族 g17 实证）；③ 幸存壳即占位——块空 br 臂加 `parent_ !== delKeepShell` 门（blockquote>font 双占位实证）；④ typing 包裹态 `__zwTypingTags` 同面快照回写——动作内 caret 落位走 Selection 公共面触发 `_zwSync` 清除（防跨子测泄漏语义保留），状态命令族 g18 实证仍红、g19 复。FIXME：delete action li/dt/dd first-child 臂 + 缩进元素臂 + restoreStatesAndValues 非 collapsed 臂（当前调用域全 collapsed）。

**门禁（gate17/18/19 三批，systemd user unit + test-guard）**：g17 移植未修 multitest 9,535P/557F（P→F 50 聚类归因：override 误清 ~45 + 壳双占位 1 + 挂账 6）；g18 +FixA/B multitest 9,538P/527F（值命令族复、状态族仍红 → delta④ 定谳）；g19 终批 multitest 9,538P/554F→**9,590P/502F**（F→P 59、P→F 7 全数挂账），inserttext（fail 集 comm 空）/ipara/forecolor/wss 双语料/run-ilb/join 三案/chd/ipihost/void 双语料/iloop/illist/engine lib 3,684P/0F 逐位恒等（F1 面保持）。P→F 7 挂账：CE-false span ×4 + 全段 `<b><i>` 双壳 ×2 + styleWithCSS-ipara qS ×1 → TypingStyle 轮（前两者 vendored 忠实 = Chrome 错位，修复面在 typing 重挂语义非 delete 侧）。见 [evidence](editing-compat/evidence/2026-11-02-f6-delete-forwarddelete-shell-boundary.md)。

**下一步**：① F5 typing 尾 br 边界（D4 ~24F，Chrome typing-style 重挂语义，可并 TypingStyle 挂账 7 案同轮）→ ② ipara other-cmp 153 嵌套面（table/dl/blockquote/list 内段落 split）→ ③ qV 残余 ~109F（挂账不变）→ ④ list FIXME + delete li/dt/dd FIXME 臂（normalizeSublists/outdentNode/restoreStatesAndValues 恢复臂，嵌套与缩进交互面）→ ⑤ w5 void-br / jc 墙钟 / merge FIXME（挂账不变）。

**上一状态（2026-11-02，F2 insertParagraph 前段包裹轮，editing 流）：ipara/inserttext F→P 10、P→F 0，multitest/位移电池恒等零位移**：

**实现（F2，part01.js ① 臂一笔 + 钉测同步）**：`__zw_ce_paragraph_split_at` ① 臂（宿主级拆分）boundaryText 路径重写双块输出——前块（sepTag）= boundaryText 前兄弟逆序迁入 + head 段（`slice(0, boundaryOff)`），空则占位 `<br>`（`__zwPlaceholderBr` 打标同款）；后块（sep1）= tail 段 + 后兄弟顺迁，空则占位 `<br>`；boundaryText 摘除，caret 落后块内容起点。归因 Chrome delta：vendored insertParagraph（:7486）splitText 后前段留宿主直子**裸层**（extractContents 只摘尾段），语料期望 caret 行整体成块——`foo[bar]baz` expected `<div>foo</div><div>baz</div>` 实证。宿主元素边界路径（boundaryText null）与 ② 臂（块级后代同型拆分，wss-ip 全绿面）不动。multitest D2 重查定谳 typing×split 交互形（b 壳 caret 夹位 `<div>abc<b><br>d<br></b></div>`），非纯 ① 臂面挂账。engine lib 钉测 `test_insert_paragraph_split_r3254_m3_slice5` 锁旧 SetChildText 序，按 F2 双块语义同步更新（CreateElement×2 + CreateTextNode head/tail + RemoveChildAt 摘旧文本 + AppendChild），行为变更记档注释。

**门禁（gate16 串行批，systemd user unit + test-guard）**：ipara 6,720P/475F→**6,728P/467F**（F→P 8 恰裸文本 host 直子面 8 案）、inserttext 2,777P/232F→**2,779P/230F**（F→P 2）——合计 **F→P 10、P→F 0**；multitest 9,538P/554F、wss-ip 1,104P/0F、wss-ilb 552P/0F、run-ilb 1,641P/50F、join 三案 3,456/2,607/1,455P/0F、chd 420P/0F、ipihost 880P/0F、void 72+48P/0F、forecolor 2,279P/48F、iloop 1,619P/142F、illist 1,910P/155F 全恒等零位移（F1 面保持）；engine lib rc=0（钉测同步后 3,684P/0F）。探针 zz-probe-ipara 六案逐字语料期望（mid/range/start/end/inblock/p-sep，gitignore add -f）。残余：ipara other-cmp 153（table 跨块选区入格形、dl/dt/dd 嵌套形、blockquote 形、list 内段落 split 形）+ non-compare 174（in_array 63 多形态允许值 + equals 111 读面）。见 [evidence](editing-compat/evidence/2026-11-02-f2-insertparagraph-host-split-head-wrap.md)。

**上一状态（2026-11-02，F1 list 命令簇轮，editing 流）：iloop/illist/multitest 合计 F→P 422、P→F 0**：

**实现（F1，三文件）**：① part01e 新分片（挂 part01d 后，348 行）——`_zwListSelectionState`（getSelectionListState 移植：blockExtend + contained 过滤 → ol/ul/mixed 系/none）+ `_zwListToggle`（toggleLists 移植：enable other-tag 换型/邻接并入 + sublist 聚合（p/div→li 换型、li/ol/ul 直收、inline wrap 成 li）套壳；disable outdent ol/ul 臂（摘 reversed/start/type → preserve-descendants 撤壳 → 子 fixDisallowedAncestors）+ splitParent + fixDisallowedAncestors）+ `_zwListSplitParent`（:5058 忠实移植：尾段独在臂逆序后移/首段缺位 clone 壳承接/通用前移 + followsLb/precedesLb br 保全）+ `_zwListFixDisallowedAncestors`；FIXME：normalizeSublists/recordValues/restoreValues/outdent 缩进臂/isSimpleIndentationElement。② part01c 一笔——`_zwJusWrapViaHtml` fresh np 返回 host 内 open-tag 重定位新包元素（原引用脱挂，list enable 套壳断链实证；wrap 返回值消费面仅 list 一处，justify 簇弃返回值不受影响）。③ part06 三笔——branch3 list 应用臂（justify 臂模式，事件口径同）+ queryCommandState/queryCommandIndeterm list 读面（`__zw_ce_list_state`：state = list state 恰为本命令 tag，indeterm = mixed | mixed ol/ul）。**探针轮三 bug 修正**：tag 常量大小写（otherTagName 大写传小写比较机具恒 miss——ol→ul 换型恒等实证）、WrapViaHtml 脱挂返回（裸 `<li>` 无壳实证）、mixed 判序双反（`hasUl ? 'mixed ol' : 'mixed ul'` → vendored hasOl/hasUl 分立 + 'none' 尾）。

**门禁（gate15 串行批，systemd user unit）**：iloop 1,438P/323F→**1,619P/142F**（F→P 181）、illist 1,721P/344F→**1,910P/155F**（F→P 189）、multitest 9,486P/606F→**9,538P/554F**（F→P 52）——合计 **F→P 422、P→F 0**；inserttext/ipara/forecolor/wss-ilb/wss-ip/join 三案/chd/ipihost/void×2/run-ilb 恒等零位移；indent 1,182P/147F 首轮基线（list×indent 交互面）；engine lib rc=0。残余定谳（iloop 142F）：dd/dt 子树入列形（`<dd><ol><li>…` 期望 dd 保留壳）+ 缩进 blockquote 合并形（isSimpleIndentationElement 臂）+ 聚合尾边界形 + qV/qState 零散（49）。探针 zz-probe-list 六案逐字 vendored 忠实行（含 toggle-off 裸 `<li>` = isAllowedChild(li,div)=true 忠实产物）。见 [evidence](editing-compat/evidence/2026-11-02-f1-list-command-apply-and-qstate-face.md)。

**下一步**：① F2 insertParagraph host-level split 前段包裹（ipara 465F + multitest D2 21F + inserttext `\n` 族，`<div>foo</div><div>baz</div>` 双包裹语义）→ ② F6 delete/forwardDelete 撤壳边界（multitest D1 ~100F）→ ③ F5 typing 尾 br 边界（D4 ~24F，Chrome typing-style 重挂语义）→ ④ qV 残余 ~109F（挂账不变）→ ⑤ list FIXME（normalizeSublists/recordValues/restoreValues/outdent 缩进臂/isSimpleIndentationElement，嵌套与缩进交互面）→ ⑥ w5 void-br / jc 墙钟 / merge FIXME（挂账不变）。

**上一状态（2026-11-01，insertText 波及语料 F 族普查 + 状态命令 typing 面轮，editing 流）：五语料 2,135F 普法定案 + multitest 761F→606F（F→P 155、P→F 0）**：

**普查（gate13 首测 2,135F 聚类，断言 × 签名 × 形态）**：① iloop/illist 323+344F = list 命令**零应用**（compare got 与输入恒等 + qState before/after 面）——F1 修复面（branch3 insertorderedlist/insertunorderedlist 实应用臂，vendored insert list ~1,000 行：toggle + 邻接合并 + li 拆分 + indent 交互，~600F 最大单族）；② ipara 465F + multitest D2 21F + inserttext `\n` 2F = **host-level split 前段不包裹**单机制（`foo[bar]baz` got `foo<div>baz</div>` vs expected `<div>foo</div><div>baz</div>`；table 跨块 got `fo<div>oo</div>` 形）——F2（~490F）；③ multitest 761F 五族：D1 delete 撤壳 ~100F（`exp 'abaz' got 'a<a href…><font…'`，Chrome delete 语义独立轮）、D2 前述、D3 状态命令 typing 族（sub/sup 不在 format tag 表 + qState 读面无 typing 兜底，**本轮修复**）、D4 typing 尾 br 边界 ~24F（`<b>abc[]<br></b>` typing expected `<b>abc</b>d` 弃包裹，`<b><i>abc[]<br></i></b>` 形出包 i 留 b 造新 i——Chrome typing-style 重挂未建模，独立轮）、D5 形态矛盾挂账扩面（fontsize+sub 复合案实证：size2 零包裹 vs size3 `<font size=3>` 形 vs 本实现 flag-true span 形，不 chase）；④ inserttext 232F 余族：`&nbsp;` 尾空格 2、非空选区空串 deleteSelection 前置 1、白空间容器 typing 形态。

**实现（F3+F4，part06 三笔）**：① `_zwExecCmdFormatTag` 补 subscript/superscript（Chrome 恒元素形，bootstrap flag=true 下录制仍 `<sub>` 实证，无 CSS 臂）；② collapsed 臂 ttags 补 sub/sup 互斥（spec action 先关对侧；交叉序 expected `foo<sub>a</sub>bar`/`foo<sup>a</sup>bar` 实证）；③ `_zwQueryFormatState` 补 typing-state fallback（祖先链 miss 后查 `__zwTypingTags`——Chrome stateOverride 语义：collapsed 执行不改 DOM 但 qState 立即 true；caret 未动类 `bold+insertorderedlist`/`bold+delete` 语料 expected true）。**gate14 弯路修正（Dom-split）**：ttags fallback 只进 qState 读面，臂内 toggle 判定拆 `_zwQueryFormatStateDom` 纯祖先链——双执行 underline 语料实证 Chrome 第二次 collapsed 执行**不关** typing 态（fallback 进判定会误关）。

**门禁（gate14 + gate14b 串行批，systemd user unit）**：gate14：multitest 9,331P/761F→9,484P/608F（F→P 155、P→F 2 双执行对）、iloop/illist/forecolor 恒等、位移电池 wss-ilb 552+wss-ip 1,104+join 三案 3,456/2,607/1,455+chd 420+ipihost 880+void 72+48+run-ilb 1,641（50F）逐位恒等、engine lib rc=0；gate14b（Dom-split 干净二进制）：multitest **9,486P/606F——对轮初基线 F→P 155、P→F 0**，wss-ilb/wss-ip/join-paragraphs 恒等；探针 zz-probe-subsup 六案逐字（sub/sup typing 包裹 + 互斥 + qState 三面）。

**下一步**：① F1 list apply 臂（iloop/illist ~600F：branch3 insertorderedlist/insertunorderedlist 实应用 + qState/qV 读面，vendored insert list 算法移植）→ ② F2 insertParagraph host-level split 前段包裹（ipara 465F + multitest D2 + inserttext `\n`，`<div>foo</div><div>baz</div>` 双包裹语义）→ ③ F6 delete/forwardDelete 撤壳边界（multitest D1 ~100F）→ ④ F5 typing 尾 br 边界（D4 ~24F，Chrome typing-style 重挂语义）→ ⑤ qV 残余 ~109F（挂账不变）→ ⑥ w5 void-br / jc 墙钟 / merge FIXME（挂账不变）。

**上一状态（2026-11-01，s212 handle 子反链 + inserttext 变异核轮，editing 流）：三族并发归因两批收口——五语料 8,922P/296F→**8,986P/232F**（F→P 64、P→F 0）+ insertText 波及语料首测**：

**实现（两组四文件）**：① s212 part04 两臂——appendChild/insertBefore handle 子补 `_zwForceParentLink` 反链（removeChild 的 `parentNode=null` 落 expando own 值、get trap expando 优先于 `_zwNodeParent` 槽读面 → 重挂后 parentNode 恒 null → forceValue vendored :762 UNDO 误触发，w3/w4 嵌套包裹族根因；R84/R245 同款对偶）；② s212 part01d——`_zwVaProhibitedParagraphChildren` 对齐 spec 44 名（补 tbody/td/tr 等 table 族、移除 canvas/dialog）+ `_zwVaIsAllowedChild` step-6 终检（骨架/table 名恒 false，参考实现 :1682-1689）——修复 ① 后 eff 读面正确暴露 ② 缺口（forecolor 中态 67F→131F 回归态实证，此前靠孤儿 + UNDO 碰巧正确）；③ inserttext part01——`__zw_ce_inserttext_mutate` 变异核导出（caret 世界同步 + '\n' 分流 + 三臂 + caret 重锚；事件序留键盘 `__zw_ce_insert` 与 branch3 各入口，核零事件不双派）；④ inserttext part01d + part06——`__zw_ce_value_override_snapshot`（核头捕获，读侧 reset：变异移动选区后 stored range 失配）+ `__zw_ce_value_restore_overrides`（变异尾：临时 range 覆盖插入段进 `_zwVaSetSelectionValue` 主流程——就地 splice 整节点过裹由边界 splitText 消化；等值门）+ branch3 inserttext 实应用臂（此前事件照派零变异；vendored recordCurrentOverrides :4351 / restoreStatesAndValues :4458 照录）。

**验证**：zz-probe-typing 三形逐字（A plain `"abc"` / B `a<font color="#0000ff">b</font>c` / C `a<span style="color: rgb(0, 0, 255)">b</span>c`）；forecolor a[]c 8 案双向齐收——flag-false-at-insert 四案 font 形 + flag-true-at-insert 四案 span 形，typing 重放跟随 CSS flag（= forceValue `!cssStylingFlag` 同门，vendored :2795）。

**门禁（gate12 → gate13 串行批，systemd user unit）**：gate12（s212 单独）：五语料 8,922P/296F→**8,978P/240F**（F→P 56），位移电池九案逐位恒等，hsem 652→648F；gate13（合并干净二进制）：fontsize 2,334P/88F F-list 逐字节恒等、forecolor 2,279P/48F（F→P 8 恰 a[]c 族）、fontname/backcolor/hilitecolor F-list 逐字节恒等——五语料 **8,986P/232F**；inserttext 2,777P/232F、multitest 9,331P/761F、ipara 6,720P/475F、iloop 1,438P/323F、illist 1,721P/344F 首测；engine lib + webview lib（env -u）+ bench **rc=0**（gate12 三项 rc=127 系 unit PATH 无 cargo 基建缺口，gate13 脚本补 `$HOME/.cargo/bin` 收口）；hsem 复跑 **6,917P/648F**（F 数恒等，2↔2 :active flakes 族零真实回归；批内首跑 8 job 零输出即死一次性观测）。

**挂账**：multitest typing 形态矛盾族——multitest.html bootstrap `execCommand("styleWithCSS", false, "true")`（录制环境 hack）下 fontname/forecolor+insertText 期望 `<font>` 形，与现代五值语料 flag-true→`<span>` 期望矛盾（~16-24 案，取 flag-following 不 chase）+ mid-command 缺口族（formatblock/indent/insertorderedlist/insertimage/inserthorizontalrule branch3 无实应用臂）。

**下一步**：① insertText 波及语料首测 F 族普查（ipara 475F/iloop 323F/illist 344F 家族聚类 + mid-command 缺口臂优先级）→ ② qV 残余 ~109F 归因（qV-before 30 + qV-after 79，挂账）→ ③ w5 void-br 族（`<p>[foo<p><br><p>bar]` apply 内部异常中止，挂账）→ ④ inserttext 语料 `\n` 族（引擎委托 insertParagraph 对录制不分段）+ 空串非空选区 deleteSelection 前置序归因 → ⑤ jc `?5001-6000` 墙钟异动归因（挂账不变）→ ⑥ merge FIXME 族（挂账不变）。

**上一状态（2026-11-01，compare innerHTML 空白族收口轮，editing 流）：五命令空白族三笔 apply 面 Chrome delta + formattable 查询面双向 collapsed——五语料 8,717P/501F→8,922P/296F（F→P 205、P→F 0、分母逐位恒等）+ 429 中断面 splitText 修复验收**：

**实现（part01d 三笔 + part01c 一笔）**：①`_zwVaForceValue` 入口空白门——whitespace-only 文本节点 return（Chrome 不给空白独获值：跨块 `<p>foo</p> <p>bar</p>`、块内 `<p>…<span>bar</span>…</p>`、行内 `<span>foo</span> <span>bar</span>` 三形录制空白恒裸/仅随携带；门置入口统一覆盖主循环/pushDownValues 兄弟臂/children 递归，pushDownValues 本体维持 vendored 全量；`_zwJusIsWhitespaceNode` white-space 感知，pre 不受门）；②`_zwVaWrap` 拉邻放宽——invisible 之外 whitespace-only 文本兄弟随携带（空白先入列 sibCrit 才能沿 `nodeList[0].previousSibling` 命中已样式化邻接 wrapper 合并出单一外层 span 形）；③`_zwVaWrap` 三处 br 插入臂补连接点空白隔断判定（vendored br 防 run 连字，空白已防则 br 成多余可见断行——w3 录制无 br）；④part01c `_zwJusFormattableVisible`——formattable 消费面文本节点 collapsed 判定补**前向走查**（vendored isCollapsedWhitespaceNode :912-1001 双向，既有 `_zwJusIsCollapsedWhitespaceNode` 结构化重写只保留后向→w1 ` " "@DIV` 判可见被 forceValue 误包/裸空白被 formattable 集误收→indeterm true≠录制 false），仅 qV/indeterm/apply-entry 两消费点取双向语义，结构面（block boundary、follows/precedesLineBreak）单向语义不动（wss/ilb/join 基线锁定）。

**429 中断面处置**：上轮提交后限流中断遗留 part01d 两笔——splitText start 侧原位写（选区重绑舞撤改）验收保留（forecolor 中断面二进制 +22 F→P 零回归：16 qV-after + 6 compare，`foo[bar<b>baz]qoz</b>quz` 族零变异洞证实收窄）；MAIN 循环 invisible skip 撤改（判定依赖单向 collapsed 重写失效——探针 LOG 实证 `" "@DIV` 无 skip 直达 FV，重构为上述四笔）。

**门禁（gate10f 串行批，干净二进制 18:33-19:01）**：fontsize **2,329P/93F**、forecolor **2,260P/67F**、fontname **2,150P/75F**、backcolor **1,002P/24F**、hilitecolor **1,181P/37F**——合计 **8,922P/296F**，F→P=205、P→F 全零、分母逐位恒等；干净二进制 forecolor 失败集与中途逐位 identical。位移电池 join 三案 **3,456/2,607/1,455P/0F** + wss-ilb **552P/0F** + wss-ip **1,104P/0F** + chd **420P/0F** + ipihost **880P/0F** + void **72P/0F + 48P/0F** + ilb run **1,641P/50F** 逐位恒等；engine lib **3,670P/0F**（+3 同窗并行流新增）+ webview lib **834P/0F**（env -u 清代理跑法）+ wcomp **4,022P/1,146F/5T** 计数级恒等 + bench **PASS**；hsem **6,913P/652F/162T**（总 7,727 恒等，+8 全 F→P 逐行定谳 rebase 窗并行流修复面吸收——s209 number 编辑器 16:32:51 四行 + s210 FORM 约束校验 16:56:51 四行，两笔提交时刻晚于 gate9 hsem 跑完时点 16:56:16，g8/g9 快照该八行均败本轮转 Pass，非本 diff 面）。残余族普查（296F）：compare innerHTML **167**、qV-after ~79、qV-before ~30、stylewithcss State before 5。明细见 [evidence](editing-compat/evidence/2026-11-01-sv-compare-innerhtml-whitespace-family.md)。

**下一步**：① compare innerHTML 残余 **167F** 归因（font rewrap 拆分 run 族 ~12 行/`a[]c` insertText multitest 8 行为主刀）→ ② qV 残余 ~109F 归因（qV-before 30 + qV-after 79，fontname sans-serif 族已接 override 读数面）→ ③ w5 void-br 族（`<p>[foo<p><br><p>bar]`——apply 于 `FV <BR>→WRAP` 内部异常中止、part06 catch 吞没，抛点独立轮）→ ④ enabled 端点门扩其余语料（bold tt 同形同期望已实证）→ ⑤ jc `?5001-6000` 墙钟异动归因（挂账不变）→ ⑥ merge FIXME 族（挂账不变）。

**上一状态（2026-11-01，setup-exception 族查询索引失效轮，editing 流）：R307 attr 变体——M-form 树 attr 写入点补祖先查询索引失效——五语料 8,266P/952F→**8,717P/501F**（F→P 451 逐语料 90/91/90/90/90、P→F 0、分母逐位恒等，setup 异常族每语料恒 96 → 0 全清）**：

**根因（part03.js 单机制洞）**：setupDiv 选区构建三步——`querySelector('[data-start]')` 命中真节点 → `removeAttribute('data-start')` 改 attrs 数组**不失效**祖先挂槽查询索引 `_zwNodeIdx`/包装缓存 `_zwQWrapMap`（键含 outer——R355 统一键构造）→ `querySelector('[data-end]')` 新键对陈旧索引恒 miss，`_zwMWrapCached` 回落 detached `_zwParseEl` 孤儿 wrapper（parentNode null、tr 上下文解析 0 子）→ `range.setEnd(orphan, 2)` 命中 `_nodeLength=0` 界检抛 IndexSizeError（part06 R288），该 browserTest 行全部 setup 断言败。R307 既有失效只覆盖 append/remove 两树结构写点，attr 面同族缺陷漏网——「靠 outer-key 差异近似」只保证旧键 miss，不保证新键命中真节点。

**修复（part03.js +21/−0）**：`_zwMElQIdxInvalidate(node)` helper（R307 同款沿父链遍历，depth 64 防环）挂五个 attr 写入点——`_zwMElSetAttribute`/`_zwMElRemoveAttribute`（NS 变体经委托同覆盖）+ IDL 直写三件 `_zwIdSet`/`_zwClassNameSet`/`_zwSlotSet`（classList 写面同经此）。失效后下次查询重建索引命中真节点（R167 归一 + R308 miss 补登语义），查询产物与 traverse/mutation 面 identity 统一。定谳手法与 ZZDBG 诊断翻转（孤儿 `el:0/eParent:null/IndexSizeError` → 真节点 `el:3/eParent:TBODY/se:ok`）见 [evidence](editing-compat/evidence/2026-11-01-setup-family-query-idx-invalidate.md)。

**门禁（gate9 串行批）**：五语料 fontsize **2,301P/121F**、forecolor **2,233P/94F**、fontname **2,074P/151F**、backcolor **965P/61F**、hilitecolor **1,144P/74F**——合计 **8,717P/501F**，F→P=90/91/90/90/90（计 451）、P→F 全零、分母逐位恒等；setup 异常断言五语料 grep 全零。位移电池 join 三案 **3,456/2,607/1,455P/0F** + ihost **880P/0F** + wss-ilb **552P/0F** + wss-ip **1,104P/0F** + chd **420P/0F** + ipihost **880P/0F** + void **72P/0F + 48P/0F** 逐位恒等；ilb（run 面）**1,641P/50F**（gate8 1,633P/58F → F→P 8 正向连带，同族 setup 形连带回收）；wcomp **4,022P/1,146F/5T** 计数级恒等；engine lib **3,667P/0F**（gate8 3,664P/1F 的 s24 红已由 rebase 窗 PR #267 修复，差额 +3 测试为同窗并行流新增）+ webview lib **834P/0F**（env -u 清代理跑法）+ bench rc=0（absolute budgets 达标，硬件 WARN 既有）；hsem **6,905P/660F/162T**（总 7,727 恒等：+4 F→P form-action-reflection ×2 + output mutations ×2——attr/mutation 变异邻近面因果相容；P→F 2 = checkable-active-onblur ×2 `:active` 时序 flakes 既有家族，单语料复跑 Checkbox 案转 Pass/Radio 案仍 Fail 两跑异果定谳间歇，非确定性回归）。残余族普查（501F）：compare innerHTML **256**、queryCommandValue **231**、queryCommandIndeterm **9**、queryCommandState **5**。明细见 [evidence](editing-compat/evidence/2026-11-01-setup-family-query-idx-invalidate.md)。

**下一步**：① compare innerHTML 残余 **256F** 归因（apply 缺口②续刀）→ ② qV 残余 **231F** 归因（fontname sans-serif 族为主刀，override 读数面已接）→ ③ enabled 端点门扩其余语料（bold tt 同形同期望已实证）→ ④ jc `?5001-6000` 墙钟异动归因（挂账不变）→ ⑤ merge FIXME 族（挂账不变）。

**上一状态（2026-11-01，style-with-value 簇实应用面收口轮，editing 流）：execCommand 五值命令 `setSelectionValue` 引擎移植——五语料 7,562P/1,656F→**8,266P/952F**（F→P 705、P→F 1、Timeout 0→0，分母逐位恒等）**：

**实现（part01d.js 新文件 1,041 行 + part06 两笔 + part01c override 消费 + bridge 挂载）**：vendored `setSelectionValue` 全流程忠实移植（collapsed 值 override 记账带选区快照失配重置 + backcolor/hilitecolor 镜像；边界 splitText 安全拆分；clearValue + pushDownValues/forceValue 三 pass；wrap/reorder/removeExtraneousLineBreaks 共享机具复用 `_zwJus*` 闭包）。入口三导出 `__zw_ce_value_{apply,validate,override_value}`：part06 校验前置（返 false 零事件，Chrome 口径）+ apply 臂（justify 臂后、inserthtml 臂前）接线；part01c `__zw_ce_value_query` 入口 override 优先消费（fontsize legacy 映射在导出侧两段式转换）。中途修复五笔（全 part01d）：①sizeMap 静态表（plain 回落与 host 通道双读数链异值，探针二值化定谳）；②`_zwVaIsAllowedChild` 字符串 parent 分支（跨 p 选区 forceValue 误 wrap 主刀）；③style attr 权威核验（host setProperty 通道不落 attr，innerHTML 序列化与 qV-after 连带）；④色三 Chrome 口径——非法色返 true 零变更、`_zwVaNormalizeCssColor` rgb 分量钳位归一、a[href] 内 forecolor 零 DOM 变更 + 值 override 记账；⑤forceValue 插入点选区边界补偿（手工探针不可复现 → 切片页定谳：裸 insertBefore 端点漂移 + movePreservingRanges 随动进包）+ enabled 门（选区端点不可编辑返 false 零事件，fontname tt 两形实证，bold 同形挂下轮）。

**门禁（gate8 串行批 + hsem 补跑）**：五语料 fontsize **2,211P/211F**、forecolor **2,142P/185F**、fontname **1,984P/241F**、backcolor **875P/151F**、hilitecolor **1,054P/164F**——合计 **8,266P/952F**，整行 ID 对账 F→P=184/182/183/72/84（计 705）、P→F=1/0/0/0/0（fontsize `<font size=+1>` 相对 size 渲染流域挂账）、onlyOld=onlyNew=0 分母逐位恒等；v2 批中途 3 例 a[href] + 2 例 tt P→F 已在 v3/v4 回收（tt/a[href] 形在 fontsize/backcolor/hilitecolor 语料零暴露 grep 实证，v2 数即终二进制数）。位移语料 join 三案 **7,518P/0F/0T** + ihost **880P/0F** + wss **1,104P/0F** + ilb **552P/0F** + chd **420P/0F** + void **72P/0F + 48P/0F** 逐位恒等（插入补偿前 build 位移电池，五值命令域外零暴露）；wcomp **4,022P/1,146F/5T** 计数级恒等；engine lib **3,664P/1F**（s24 已知红唯一，红名 identical）+ webview lib **834P/0F**（shell 代理环境首跑 833P/1F `fetch_url_invalid_host` 假红——gate8 自 shell 继承 HTTP(S)_PROXY 致 `http://0.0.0.0:1` 经代理应答不再连接失败，clean-env 复跑对齐 gate7）+ bench-gate **PASS**（absolute budgets 达标，硬件 WARN 同 gate7）；hsem **6,903P/662F/162T**（总 7,727 恒等，+8 全 F→P 正向漂移逐案 commit-mapped 定谳：s203 number 值消毒 4 案（66f4871c1——number.html ×3 + number-constraint-validation ×1，恰等额）+ s204 pattern 面 2 案（61720d309 本次窗口子集）+ s205 setCustomValidity 换行归一 1 案（7c681d5c6）+ inert-svg-hittest 1 案 `:active` 时序 flakes 既有家族——四笔均 gate7 基线（5ee3d31f9）后 rebase 窗口吸收的并行流修复与既有 flakes，非本 diff 面；P→F 0）。终版残余族普查（952F）：queryCommandValue **312**、compare innerHTML **296**（上轮 567→296）、queryCommandIndeterm **129**、queryCommandState **125**、return value 60、checks for modifications 30。明细见 [evidence](editing-compat/evidence/2026-11-01-sv-apply-face-execcommand.md)。

**下一步**：① qV 残余 **312F** 归因（override 读数面本轮已接，qV-after expected/实际分歧逐案定谳，上轮 652F 挂账转入）→ ② enabled 端点门扩其余语料（bold tt 同形同期望已实证）+ compare innerHTML 残余 **296F** 归因 → ③ Indeterm **129F** / State **125F** 族归因 → ④ 表 fixture IndexSizeError setup 形（挂账不变）→ ⑤ jc `?5001-6000` 墙钟异动归因（挂账不变）→ ⑥ merge FIXME 族（挂账不变）。

**上一状态（2026-11-01，style-with-value 簇值查询面收口轮，editing 流）：gCS plain 四属性解析器 + 五值命令引擎面接线——五语料 7,041P/2,177F→**7,562P/1,656F**（F→P 609、P→F 88 全数定谳 apply 缺口②下游，零纯值面回归）**：

**根因与修复（part01c.js +417 / part01.js +26 / part06.js +21，三文件）**：上轮 setup-exception 族收口后五语料失败主体 = `queryCommandValue` 恒 `''`/`queryCommandIndeterm` 恒 `false` 桩。**比较列真相定谳**（关键弯路）：tests.js:5595/5627 before/after 断言读**引擎面** `document.queryCommand*`，apply/spec 列才走 vendored——gate4 批只接 vendored 面五语料零变化实证后双轨都接。修复三笔：①`__zwPlainComputed(elt, prop)` plain 形四属性（color/font-size/font-family/background-color）JS 侧级联解析器——inline style → font 表现提示（size/color/face/bgcolor + UA mono 族 tt/code/kbd/samp/pre/listing/xmp/plaintext）→ 继承走查（bg 透明继续上行）→ 148 named/hex/rgb(a)/hsl(a) → Chrome 序列化口径（alpha=1→rgb() 形、transparent→`rgba(0, 0, 0, 0)`、currentColor 父链递归、非整 px 保 3 位小数）；host gCS 通道只覆盖 sel/handle 注册元素，M 形 plain 面不可达（探针 t1/t16）。②`__zw_ce_value_query(cmd)` 引擎值面——参考实现忠实移植（isEffectivelyContained :1807 / getAllEffectivelyContained 文档序走查 / isFormattable / getEffectiveCommandValue :2069 含 TEXT 归父 + backcolor/hilitecolor transparent 爬升 / getLegacyFontSize :3309 中点映射；UA 初值在值面消费点填——gCS 通道保 P9-9/R5107 合成源契约返空）。③part06 `queryCommandIndeterm`/`queryCommandValue` 两臂五命令分流 + part01 gCS 两钩子 plain 回落（camel/kebab 归一——`_zwJusCs` 以 kebab 访问）。中途修复三轮：探针 qV 口径错 → gate4 定谳比较列真相；`_getSelection` 声明域闭包不可见 → `globalThis.getSelection` 惯用法；UA monospace 缺失（fontname 2 行 P→F，BEFORE-TRUE 行集定谳 kbd 族 Chrome indeterm=false）补表后 fontname +45P 连带解锁 kbd 族。FIXME 挂账：值 override 机制未接（vendored myExecCommand 私有闭包 map 引擎不可见，collapsed 选区 qV-after 残余）；空格式 rgb 语法/font 简写/smaller/larger/rem/系统色未解析（语料无消费面）。

**门禁（gate6 本轮二进制全批 + gate7 合并树复测）**：五语料 fontsize **2,028P/394F**、forecolor **1,960P/367F**、fontname **1,801P/424F**、backcolor **803P/223F**、hilitecolor **970P/248F**——合计 **7,562P/1,656F**，逐语料整行 ID 对账 **F→P=169/88/188/76/88（计 609）、P→F=8/8/54/9/9（计 88）、Timeout 0→0**；88 条 P→F python 逐行关联定谳 85 条同 case `compare innerHTML` 同 Fail（apply 分歧 DOM 上旧恒-false 桩巧合命中、新诚实读数翻真）+ 3 条跨分片 carry-over 伪噪声，零纯值面回归。位移语料 join 三案 **7,518P/0F/0T** + ihost **880P/0F** + wss **1,104P/0F** + ilb **552P/0F** + chd **420P/0F** + void **72P/0F + 48P/0F** 逐位恒等；wcomp **4,022P/1,146F/5T** 计数级恒等（红名集合 diff 空）；engine lib **3,664P/1F**（s24 已知红唯一，红名 identical；P 3,642→3,664 并行流 part94 新增测试面）+ webview lib **834P/0F** + bench-gate（zero-engine 定向）**PASS**。**合并树复测（gate7，rebase 并入远端 14 笔——PR #265 bridge 7 文件 +1,127/−77、s199/s200 runner、s201 engine hit_test、s202 form IDL、shim part03 +88/part05 +38，systemd 单串行批）**：五语料 + 九位移文件 + wcomp + webview lib + bench 全部逐位恒等零位移；hsem **6,895P/670F/162T**（总 7,727 恒等，+23 全 F→P 正向漂移——pristine A/B 定谳 +22 系吸收的 s199-s202 并行流修复面，差值内净 ±1 落 `:active` 时序 flakes 家族（checkable-active-onblur{,-with-click}/inert-svg-hittest，同 diff 两跑漂移行数大于 pristine↔diff 差值、inert 定向复检 3/3 全绿），非本 diff 回归）。明细见 [evidence](editing-compat/evidence/2026-11-01-sv-queryvalue-plain-computed.md)。

**下一步**：① 五命令实应用臂 compare innerHTML（**567F**，apply 缺口②主刀）→ ② qV-after collapsed 选区值 override 族（**652F**，值 override 引擎面缺位）→ ③ Indeterm **214F** / State **125F** 族归因 → ④ 表 fixture IndexSizeError setup 形（other 98F 内独立面）→ ⑤ jc `?5001-6000` 墙钟异动归因（挂账不变）→ ⑥ merge FIXME 族（挂账不变）。

**上一状态（2026-11-01，style-with-value 簇 setup-exception 族收口轮，editing 流）：物化元素补 style IDL——五语料 5,056P/4,162F→7,041P/2,177F（F→P +1,985、P→F 0）**：

**根因与修复（part03.js +133 两 hunk + part18.rs t8r4 判别式契约更新）**：`normalizeSerializedStyle`（vendored tests.js `node.style.color` 抛点）setup-exception 族根因 = `_zwMEl` 物化 plain 形元素整族缺 style IDL——style 此前仅 `_makeProxy` get trap `'style'` 分支提供，document.querySelector 宿主物化命中及其 innerHTML 解析子树全为 M 形（WPT editing run 五语料 doc 级克隆/查询路径全量踩中）。归因链：探针页二值化三排除（stylewithcss 命令/字号值/整树深克隆）→ 逐阶段 spy 定位「setupDiv 后即坏」→ 文档级 vs 容器级查询分离最小复现 → t15/t16 形态取证（M 形 93 props 无 style、`SPAN(M---)>DIV(M--P)` 域标记链、`'style' in span` false）。修复：`HTMLElement.prototype` 单点 style accessor（`_zwBuiltNodeChain` 自建链门内——native prototype 不可覆盖 R3270；子类原型经 Object.create 全员继承；代理家族 get trap 先于原型链命中读数不变）+ `_zwMStyleProxy` 属性反射面板（读 getAttribute 末次非空声明 R3213、R3193 !important 剥离、`--` 自定义属性大小写敏感、写同名移除后尾部追加）；SameObject 槽 `__zwMStyleObj` defineProperty non-enumerable（wrapper 安装面 `__zw*` 槽 gPN 零进），accessor 本体 enumerable:false（R3046 for...in 零进先例，scrollTop/scrollLeft 同款）。三版演进：per-instance 弃（逐实例成本落「wrapper 安装面」深水轮刚优化 −36% 的解析构造面）→ 槽枚举性违例版弃（TaskStop 杀在途门禁批、撤码重建全量重跑——证据绑最终工件）。part18 t8r4 两臂形判别式 `typeof style==='object'` 随 style IDL 全员化失效，改域标记判别（`__zwIsMEl` M 形=代理面 vs `__zwSelector/__zwHandle` trap 应答=wrapper，两形取证 diag 入 evidence §4）；R3046 expando 枚举钉保真。

**门禁（最终二进制 gate3 全批；五语料 gate2 中途二进制已逐位一致）**：fontsize **1,867P/555F**、forecolor **1,880P/447F**、fontname **1,667P/558F**、backcolor **736P/290F**、hilitecolor **891P/327F**——合计 **7,041P/2,177F** 对基线 5,056P/4,162F **F→P +1,985 逐语料恰等额、P→F 0、T 0→0**，`reading 'color'` 族五语料 grep 清零；七位移语料 join **7,518P/0F/0T** + ihost **880P/0F** + wss **1,104P/0F** + ilb **552P/0F** + chd **420P/0F** + void **72P/0F + 48P/0F** 对上轮终树电池逐位恒等；wcomp **4,022P/1,146F/5T** 计数级恒等；hsem **6,872P/693F/162T**（总数 7,727 恒等，+3P/−2F/−1T 正向小漂移，style IDL 全员化合理外溢，无回归形态）；engine lib **3,642P/1F**（s24 已知红唯一）+ webview lib **834P/0F** + bench-gate **PASS**。enumerable:true 中途版曾在 engine lib 暴露 +2 新红（t8r4 判别式失效 + R3046 for...in 混入）修正复绿——门禁红逐名定谳不放行。残余族普查：queryCommandValue **1,173F**（下轮主刀）、compare innerHTML 567F、Indeterm 214F、State 125F、表 fixture IndexSizeError setup 形每语料恒 96 明细行（独立面）。明细见 [evidence](editing-compat/evidence/2026-11-01-sv-setup-exception-mstyle-idl.md)。

**下一步**：① style-with-value 簇 queryCommandValue 值面（**1,173F**，effective command value + getLegacyFontSize 字号映射/computed color 反射/computed family 立项）→ ② 五命令实应用臂 compare innerHTML（≈567F）→ ③ 表 fixture IndexSizeError setup 形（独立面收口）→ ④ jc `?5001-6000` 墙钟异动归因（挂账不变）→ ⑤ merge FIXME 族（挂账不变）。

**上一状态（2026-11-01，justify 查询面 uplink 断裂兜底轮，editing 流）：setter 孤儿 wrapper 读数链修复——jr/jf 各 +220P 零回归、jl 逐位恒等、jc 交集 P→F=0**：

**根因与修复三笔（part01c.js +102/−3）**：上轮 `_zwJusWrapViaHtml` flat 模型 wrap 走 `host.innerHTML=` setter（sanctioned 写路径），setter 产物子 wrapper 是**孤儿**——`childNodes` 可枚举而 `parentNode=null`、兄弟链 null，`_zwJusGetAlignmentValue` 的 parentNode 爬链与 `_zwJusIsContained` 的 BpCompare 祖先链双双失效 → 查询面 Value 恒 `'left'`/State 恒 false（justifycenter 旧日志同构 127 例、right/full 同构——共享查询面残缺非 wrap 引入）。修复：①apply 入口宿主 stash（wrap 前上行链完好时点存 `__zwJusLastHost`，换代写覆盖、陈旧 stash 下钻不中保留直读缺省无副作用）；②`_zwJusQueryRoot` 读数爬链断时 stash 兜底；③`_zwJusChainDown`/`_zwJusAvFromChain`/`_zwJusQueryAlignment` 合成链读数——只对「直读返缺省 `'left'` 且上行链不可达宿主」触发 DFS 下钻（自宿主 `childNodes` 合成祖先链），自内向外最近块级 computed text-align + 缺字面值沿链外扩显式提示（style text-align/center 标签/align 映射集），无提示返 null 保留缺省（真左对齐零接触）。健康路径 O(depth) 可达性爬链即返，无 DFS 成本。

**门禁**：justifyright **3,667P/1,147F/0T**（+220P）、justifyfull **3,622P/1,140F/0T**（+220P）、justifyleft 2,290P/213F/0T **逐位恒等**（负负得正机理——jl 期望值即 `'left'`，断裂读数恰返缺省）；逐测试 ID 对账（整行减判定）jr/jf common=4,814/4,762 **F→P=220 各、P→F=0、onlyOld=0/onlyNew=0 分母逐位恒等**，+220 结构=110（命令表×setup）对 × 每对恰 2 条（Value after + State after）——修复面恰为查询面，Indeterm/innerHTML/return 全不动；justifycenter 交集 **F→P=222/P→F=0/onlyNew=0**（全分母当日环境截断——`?5001-6000` variant 墙钟异动，**五跑 + 双重 A/B 定谳与本轮 diff 无关**：DFS 关断仍截断且更重、1min-load-1.2 静窗仍截断、**裸 HEAD pristine 仅 171 行 vs 昨 884**，挂账下轮静窗 + 逐提交 bisect 归因）；七语料 join 三案 3456+2607+1455=**7,518P/0F/0T** + wss **1,104P/0F** + ilb **552P/0F** + chd **420P/0F** + void **72P/0F + 48P/0F** 全零位移；ihost **883P/74F**——共同分母 888 行逐位恒等（880P + 8F 既有），新可见 69 行 pristine A/B 定谳截断遮蔽既有失败族（54F/9F/1F/1F/1F 裸 HEAD 逐位同败）；engine lib **3,642P/1F**（s24 已知红唯一）+ webview lib **834P/0F**；探针 6 形状全绿（zz-jrprobe/zz-jcperf 已删）；剥桩后最终工件 jr 复跑 `cmp` **逐位 identical**。明细见 [evidence](editing-compat/evidence/2026-11-01-justify-query-face-orphan-uplink.md)。

**下一步**：① style-with-value 簇（5,890F）立项归因（簇序次轮后半）→ ② jc `?5001-6000` 墙钟异动归因（静窗 + revert dc49d1e7c 单跑 variant；跨域候选：dc49d1e7c CE 环快路 lineage / 当日负载浪潮，evidence §3.4a 已录归因材料）→ ③ merge FIXME 族（jr 38 + jf 36 + jl 15 setup，块级合并缺口）留簇序后续 → ④ 并行流 wrapper 安装面深水轮已落地（上一状态）不重叠。

**上一状态（2026-11-01，「wrapper 安装面」深水轮，簇 A 流）：`_zwMEl` 逐实例安装面共享化（part03 +2,017/−1,783），seg55+seg54 同源合并收口，zz-s5 墙钟中位 107→68ms（−36%）**：

**转换与等价边（73 items：44 引用替换 + 9 体搬移 + 4 getter 提取 + 16 显式改写）**：非 tag 门控成员提升模块级共享函数（前缀 `var node = this;`，体内既有闭包捕获局部 `node` 语义逐位保留）+ 共享描述符（own 可见面 gPN/hOP/gOPD 三查逐位保留，只换函数身份）；方法体逐字搬移，sanctioned 重写仅四类（R117 上移带 node 出参、移入体尾钩补参、`attrs` → `this[_zwAttrsSlotKey]`、`_zwMEvLs` 惰性建）。等价边逐条定案：分离调用 TypeError（对齐真实 DOM Illegal invocation）；描述符 getter `.call(他节点)` 读他节点（WebIDL 正语义）；id/className getter 改读 attrs 数组（R307 镜像退役，attrs 唯一权威）；ownerDocument 写旗 `_odWritten167` 删（`_zwOdOwn !== undefined` 等价）；新增非枚举槽 `__zwAttrsArr`/`__zwOwnerEl`/`__zwOwnerNode`（gPN own enumerable 零进，无 exact-gPN 消费面）；`_zwIdlSet` → 共享 no-op 维持 `_zwMReflectIdl` skip 分支（防 removeAttribute 回写复活属性）；tag 门控族（a/area href、iframe contentDocument/Window、input value、template content、媒体族、dialog、img decode）保留逐实例闭包。验证：八段拼接 `node --check` PASS；13 移入体 + 7 提取体与 pre-transform 快照逐行 content-only 全 OK（仅 sanctioned 差异）；行为探针 4 案全绿（id/class 不复活 + 反射、classList 落 attrs、`__zw*` 零 gPN 键、事件族）；runner 通道页面元素 = handle-proxy，pre/post 双二进制同探针同读数（own 面观感跨转换恒等，plain 面由静态审计定案）。A/B：zz-s5 中位 107→**68ms（−36%）**、zz-reset ~85→**30ms**、zz-ap 44ms 恒等。

**门禁（七语料 + 双 lib + bench + wcomp + hsem，systemd 单串行批，净树）**：七语料 join **7,518P/0F/0T** + ihost **880P/0F** + wss **1,104P/0F** + ilb **552P/0F** + chd **420P/0F** + void **72P/0F + 48P/0F** 全零位移；engine lib **3,642P/1F**（s24 已知红唯一，红名 identical）；webview lib **834P/0F**（workspace 腿 cargo fail-fast 止于 engine 红，单独 test-guard 补跑对账）；bench-gate **PASS**；web-components **4,022P/1,146F/5T** 计数级恒等；html-semantics 语料续拉后全量 **6,869P/695F/163T**——`the-details-element` 21 案补入 **42P/1F**（唯一 Fail = closed-details-layout-apis.tentative content-visibility 布局 API，engine 语义面之外），Fail 分布与上轮同簇 + 新补语料面，无 shim 回归形态。**⑥ the-details-element 语料续拉销账（R4100 语义 WPT 面无红灯）**。见 [evidence](editing-compat/evidence/2026-11-01-mel-wrapper-shared-install-face.md)。

**下一步**：①（簇 A 流）appendChild trap glue（seg20 ~85µs/append，guideline #35 整体替换 = 独立整轮）与 HCLiveInvalidate（seg24，R55/R304 重审 = 独立整轮）按地图顺序各自独立轮；② 移动面 S6/S7 ~10µs/调用残余记档（次要，懒投影无 census 覆盖，收口需 host 侧查询新桥面 = 不值）；③（execCommand 簇序流认领不动）run 面现行账按命令族重排定首簇；④ PR #263 已合入（9493a8d8a）——nth-child rebind 探针复验销账归撞头仲裁流执行。

**上一状态（2026-11-01，justify 簇首簇落地轮，editing 流）：justifycenter 全簇 + 查询面三笔落地——justifycenter 真页 4,728P/1,359F/0T（8 variant 全分母），bold +464P 零新增失败，七门其余全零位移**：

**实现六笔（part01c.js 新文件 1,286 行 + part06 +62/−3 + bridge +8）**：①`_zwJusBlockExtend` 两档（默认档=条件尾爬升+宿主夹取；chromeTight 档=无尾爬升+闭合容器/宿主下移）——参考实现无条件尾爬升在语料宿主形状产出空含域（vendored 直调实证），录制期望来自真实 Chrome，按 Chrome delta 移植；②`_zwJusBpCompare` branch 2 equality（`oB < i2`→`oB <= i2`，contained=0 全谜团根因——边界 offset==路径子 index 时该子内部节点严格在后）；③`_zwJusWrapViaHtml` flat 模型 splice 路径（宿主直子文本 model-backed、innerHTML getter 走 pending 模型，裸 DOM 搬运不回流读数——探针 w3/w5 实证；整段经 `host.innerHTML=` setter 重写 + 复用兄弟包 `<br>` 分隔 + 选区端点文本偏移映射）；④查询面三笔：styleWithCSS 滞后快照（每个 execCommand 入口从当前模式同步，justifycenter 语料 250/250 行实测该模型；Value 恒 `''`）、defaultparagraphseparator 值面（缺省 `'div'`）、justify 查询面 caret 锚 fallback（contained 叶为空→锚选区起点所在块）；⑤对齐读数两笔（start/end 先询祖先显式提示链——wrap 后 td 文本继承包 div center；align 属性限 HTML 对齐映射元素集——quasit 等未知元素录制 left）；⑥wrap 样式 `text-align:center` 无空格形。

**门禁**：justifycenter 真页 8 variant 全分母 **4,728P/1,359F/0T**；bold run 面 1,138P/1,910F→**1,602P/1,446F**（+464P，Y−X=464 恒等式两侧一致零新增失败位，增量=查询面三笔命中 bold 语料同名查询断言——查询面改动对 bold 语料的正向溢出，非 bold 命令面变更）；wss 1104P/0F、ilb 552P/0F、chd 420P/0F、join 三案 3456+2607+1455=7,518P/0F、inline-host 宽 filter 880P/8F（既有 8F shadow-root delete + placeholder 伪元素）、engine lib 3,641P/1F（s24 已知红唯一）、webview lib 832P/0F 全零位移；探针 zz-jcprobe 8 形状全绿（临时页 commit 前删）；bisect driver 逐行对账 P→F 0 / F→P 398。明细见 [evidence](editing-compat/evidence/2026-11-01-justify-cluster-first-landing.md)。

**方法论注记**：语料 case timeout（`ZW_CORPUS_CASE_TIMEOUT_SECS` 缺省 30s）小于单 variant 页墙钟时 runner 截断（`Timeout ... unreported`）致全页对账行数漂移——全页对账 case timeout 须 ≥ 页墙钟（本轮 150s 跑全 8 variant）；逐行对账只取双方已报告行交集，P→F 结论不受影响。参考实现 ≠ 录制期望已实证闭环（vendored implementation.js 直调同样不 wrap，非「移植走样」）。bold 失败集对本轮基线不再逐位 identical（查询面有意变更所致，余面恒等）。

**残余与挂账**：merge FIXME 族（跨 cell 选区行/三节点合并行/tbody-tr-attr selection 行/整表选择行部分变体，~57 形状行 ×4 variant，wrap DOM 未达录制，FIXME 已标 part01c wrap 节）；quasit align 残余 = 引擎 computed style 未知元素 `text-align: inherit` 求值债（style-system 面，渲染流域，4F 不跨界）；justifyright/full/left 与 justifycenter 共用 `_zwJus*` 实现（分派臂 part06），簇序次轮全页对账补账。

**下一步**：① 簇序次轮：justifyright/full/left 全页对账补账 + style-with-value 簇（5,890F）立项归因 → ② 并行流在途（wrapper 安装面深水轮）不重叠 → ③ PR #263 已合入（9493a8d8a）——nth-child rebind 探针复验销账归撞头仲裁流执行（前轮挂账不变）。

**上一状态（2026-11-01，CE 环 details 全树扫归因 + 成员资格快路轮，簇 A 流）：`__zwDetailsScan4100` 归因定谳 + R4100 门内快路落地——bulk fill 墙钟 −20%**：归因反转地图假设——scan **不是逐子触发**，而是**每个 innerHTML 物化尾单次调用**（8 调用点全枚举：S1-S5/S8 解析物化面 isParse=true、S6/S7 脚本移动面 isParse=false）；主项 S8（`_ihMat` 尾，2000 roots 一把扫）~19ms ≈ **9.5µs/根**，成本 = handle wrapper 逐根 trap 读（nodeType/tagName/childNodes，guideline #26 trap 域成本模型）——「~15µs/子」旧口径系 `_ceApplyConn` walk 常数摊入；sel 域同形扫（plain wrapper 本地属性读）只 1-4ms 证贵在 trap 读非遍历逻辑。

**决策（快路落地，part03 +15/part05 +3）**：构建期 census 单调闩 `__zwDetailsEver4100`（`_zwMEl` + `_dsBuildHandleChild` 两处，never cleared 保守方向），`__zwDetailsScan4100` 入口 `isParse && !闩` 直返——闩 false ⟹ 本世界从未构建 DETAILS ⟹ 解析面批次（恒构建产物，六面 roots 溯源全验）必空 frag、scan 恒 no-op（frag 循环外无副作用），跳过等价；移动面不门（主文档初始树 = host 侧懒投影不经构建工厂，census 不可靠；S6/S7 本就次要 2-3ms/251 调）。A/B：S8 scan 19ms→~0（walk.visits 2001→1）、zz-s5 wall 中位 133→107ms（**−20%**）、zz-ap/zz-reset 不变（不门 ✓）。门禁零位移：九语料 + engine lib（3,642P/1F s24 已知红唯一）+ webview lib（834P/0F）+ bench-gate PASS；html-semantics 新面首跑 5,170P/558F——`the-details-element` 语料 0 案（GitHub API 间歇 403，fetch 可续拉挂下轮），Fail 集中 img 解码/customizable-select/textarea-wrapping，无 details 相关案。见 [evidence](editing-compat/evidence/2026-11-01-details-scan-membership-fast-path.md)。

**下一步**：①（簇 A 流）「wrapper 安装面」深水轮（合并 seg55+seg54，审计清单见上一状态）；② appendChild trap glue（seg20 ~85µs/append，guideline #35 整体替换 = 独立整轮）与 HCLiveInvalidate（seg24，R55/R304 重审 = 独立整轮）按地图顺序各自独立轮；③ 移动面 S6/S7 ~10µs/调用残余记档（次要，懒投影无 census 覆盖，收口需 host 侧查询新桥面 = 不值）；④（execCommand 簇序流认领不动）run 面现行账按命令族重排定首簇；⑤ PR #263 已合入（9493a8d8a）——nth-child rebind 探针复验销账归撞头仲裁流执行；⑥ html-semantics `the-details-element` 语料续拉续跑（R4100 语义 WPT 面补账，fetch 脚本可跨次续拉）。

**上一状态（2026-11-01，插入侧解析残余归因轮，簇 A 流）**：seg55「本地解析 ~21µs/span」定谳 = wrapper 逐实例安装面，非解析/传输（tape v4/v4b：host 桥 + JSON.parse 合计 <2ms 免费、构建段 99%、无隐藏二次项），不改代码轮；与 seg54 合并立项「wrapper 安装面」深水轮（审计清单：gPN/hOP/descriptor 三查 + R55/R304 身份稳定 + R309 视图语义 + R167 `_zwMEvLs` 双引用），seg55「解析器优化」原 scoping 关闭。见 [evidence](editing-compat/evidence/2026-11-01-insert-parse-construction-attribution.md)。

**上一状态（2026-11-01，簇 A removeChild 残余 O(n²) overlay 单遍化轮，簇 A 流）：`_zwOverlayPendingChildNodes` 内部两循环单遍化（part05 +40/−9）——tape v3 内部插桩钉死剩余墙钟 ~96% 在 overlay 内部（added 合并 ~70% + removed 逆序扫 ~18%），基底缓存恒命中（hit=4001/miss=0，rebuild≈0）host 往返无关；三笔修复：removedSet 单遍 `has` 原地压缩过滤（等价性 = res 条目 identity 唯一 + removedSet/removed 记账侧恒成对维护）、`_resSeen` res identity Set（seen O(1) 判定 + nextSibling O(1) 成员门，miss = 旧扫空落尾插等价）、尾插 push（splice 在 pos===length 时逐位等价）；每调用 O(added×res)→O(added+res)，探针 w2 均值 3,832→1,506ms（tape A/B −61%）、净树 1,564ms（对 R5340-glue 轮 4,076ms −62%、对两轮前 6,661ms 累计 −77%）**：

**归因与取舍（tape v3 三样本，插桩剥离后净树复跑核数）**：splice memmove 与 Proxy trap 读非主项——微修单独 A/B 实测零增量（push 1,526→1,529 同带保留；trap 读复用 910-987 vs 948-967 噪声带，**已回退**——未兑现机制不写收益注记）；残余 ~1.4s = 每 removal 仍 2 次物化的单调用 O(n) 常数（Set 构建 + 过滤 + 逐条 ~240ns 循环体），下一刀 = 单物化/视图缓存面——LiveNLSync 增量化或融合视图缓存需 R55/R304 身份稳定副作用与 `_zwHCLiveInvalidate` 失效覆盖重审（`_r140Refresh` held-array 语义靠 mutation 时点同步维持，惰性化失真 + R382 惰性 ghost 使「上次视图 −1」推导不等价），既有独立整轮 scoping 不变。见 [evidence](editing-compat/evidence/2026-11-01-removechild-residual-overlay-singlepass.md)。

**门禁（十面，systemd 单串行批，净树）**：七语料 join **7,518P/0F/0T** + inline-host **880P/0F** + wss **1,104P/0F** + ilb **552P/0F** + chd **420P/0F** + void 双语料 **72P/0F + 48P/0F** 全零位移；engine lib **3,641P/1F**（s24 已知红唯一，红名 identical）；webview lib **832P/0F**；bench-gate（zero-engine 定向）**PASS**；web-components **4,022P/1,146F/5T** 与 CE 快路轮三跑同计数——计数级恒等（失败集逐位 diff 留该面下次基线复验）。组合态复测（推送 rebase 并入并行流 s186-s189 同 shim 四文件 + layout-engine 两笔，part05 hunks 536/855/1126 与本轮 overlay 区不相交）：join 7,518P/0F + ihost 880P/0F 逐位恒等、探针 w2 均值 1,535 净树带内、0F/0T **零位移**。

**下一步**：①（簇 A 流）插入侧残余按 CE 快路轮 evidence §4 现行地图逐项独立轮（解析 `_zwMBuildBodyTree` / t8r 物化 / appendChild trap glue / HCLiveInvalidate / CE 环 details 全树扫五项）；removeChild 侧 O(n²) 已收口至算法等价底线，单物化/视图缓存面（R55/R304 重审）为深水区独立整轮；②（execCommand 簇序流认领不动）run 面现行账按命令族重排定首簇；③ PR #263 已合入（9493a8d8a）——nth-child rebind 探针复验销账归撞头仲裁流执行。

**上一状态（2026-10-31，插入侧残余归因 + CE entry 空表快路轮，簇 A 流）：`_ceEntryFor` 空表快路（part03 +10 行）落地——tape v2 归因钉死 CE 连接环主项 = 未缓存 key 的 `_realTag` + host `is` 属性往返（2000 子 bulk fill = 2000 次纯浪费往返，seg15 ~41ms）；`_ce_registry_gen` 只增不清（唯一写入口 `_ceDefine` 内 `reg[name]=` 后紧跟 `++`，无 reset 无旁路写）⟹ gen=0 ⟺ registry 结构性空 ⟹ 快路直返 null；A/B seg15 41.3→30.3ms（−27%）、zz-s5 fill wall 145.3→136.0ms（−7%）**：

**归因（tape v2 三窗×3 样本，临时插桩已剥）**：bulk fill 四主项 = 本地解析 `_zwMBuildBodyTree` ~21µs/span + t8r 物化 `_wcRebuildAsHandle` ~21µs/span（架构常数、既有正确性修复面）+ CE 环 seg15 ~41ms（本轮修复面）+ HCLiveInvalidate ~10ms；per-append 两主项 = appendChild trap glue ~85µs + HCLiveInvalidate ~73µs；notify 内部机制段全净 ≤5µs——10-30 trap 层四笔效果保持。**s111 臂残余定谳历史态**：三窗 ≤1-3ms/2000 子、≤4µs/append，「34-46µs/append」系 10-30 归因时点数字，已被 10-30 t8f 印章前移 + `_realTag` handle-only O(1) 表查吸收（与前轮 ~650µs text-child 同款关闭先例）。**顺带修一潜伏边**：旧 `_ceIsChecked` 章跨 gen bump 存活，gen=0 期插入元素 define（customized built-in）后 is 反查被旧章跳过——快路使该形态改走完整反查（更正确）。插入侧残余地图刷新（解析 / t8r / trap glue / HCLiveInvalidate / CE 环 details 全树扫五项），维持既有独立整轮 scoping。见 [evidence](editing-compat/evidence/2026-10-31-insert-side-attribution-and-ce-entry-fast-path.md)。

**门禁（六面，systemd 串行批）**：join 三案 **7,518P/0F/0T**、inline-host **880P/0F**、engine lib **3,641P/1F**（s24 已知红唯一，红名 identical；+6P 他流入账正漂移）、webview lib **832P/0F**、bench-gate（zero-engine 定向）**PASS**、web-components 跨域 pristine-A/B **4,022P/1,146F/5T 三跑逐位一致零位移**。合并树复测（ba268c969 = s185 三笔 part03/04/05 +414/−53 于门禁后并入同文件）：join 7,518P/0F/0T + inline-host 880P/0F **零位移**。

**下一步**：①（簇 A 流）removeChild 残余 O(n²)（R126 子判定 + LiveNLSync 每 removal 2 次物化）独立整轮 + 插入侧残余按 evidence §4 现行地图逐项独立轮；②（execCommand 簇序流认领不动）run 面现行账按命令族重排定首簇；③ PR #263 已合入（9493a8d8a）——nth-child rebind 探针复验销账归撞头仲裁流执行。

**上一状态（2026-10-31，nth-child rebind 探针撞头仲裁轮，execCommand 簇序流）：本流查询作用域位置索引独立实现与 site-opt PR #263 同根同判撞车——撤改零残留，探针 A/B 独立佐证入库，本轮认领 execCommand 簇序**：

**撞头仲裁（429 中断轮遗留工作面处置）**：簇 A 插入侧轮 rebind 环探测独立发现宽兄弟 `:nth-child` 链查询 O(候选×兄弟) 全扫（探针页 zz-att1600：1600 span 宿主 + 1601 对 handle 重绑，单查恒 16ms → rebind 42,217ms、completion Timeout），已实现 Document 字段查询作用域索引（A/B：单查 0ms、rebind **217ms（194×）**、Timeout→Pass）。rebase 后发现 site-opt PR #263（`p125-query-nth-child-wedge`，github trending 真站楔死同根因；thread_local 每查询索引 + 廉价成分先行 + shadow 变体覆盖 + merge-base 等值回归钉，先落地 18:32 commit / 19:14 PR，OPEN precheck-pass）——**同根同判撞车，按「消费域收敛、不回退对方修复面」先例采纳对方版本**：本流实现撤改零残留（实现全文存档本地分支 `wip/editing-nth-child-query-scope-index` b88d132f0 不 push，#263 弃置时可复活），A/B 与插桩归因作为对方修复面在编辑域 rebind 环的独立佐证入库。

**编辑域影响评估（性能余量项，非语料阻塞）**：现行语料宿主树窄（几-几十子），`:nth-child` 单扫 µs 级，bold 墙钟 117s 账内 rebind 占比可忽略，42s 形态仅 1600 兄弟压力页现形；簇 A 验收线不动（villain 仍 s111 臂 / CE loop / removeChild 残余）。#263 合并后编辑域顺带受益（rebind 环 / plain 寻址 / 真站宽页面）。**销账** = #263 合入后重建探针复验 rebind ≤ 1s 同带（探针页本轮已失：`ZW_PROBE_REFRESH` 为已剥临时臂净源码无此变量，重建配方见 [evidence §5](editing-compat/evidence/2026-10-31-nth-child-rebind-probe-collision-adjudication.md)）。

**分工认领**：execCommand 簇序（run 面现行账按命令族重排 + 首簇立项归因）归本流；并行流 removeChild 残余 O(n²) 独立整轮 + 插入侧残余不受影响。

**下一步**：① execCommand 簇序：现行 run 面账（bold 1,138P/1,910F、insertparagraph 13,935P/12,509F、delete/forwarddelete/multitest 等）按命令族聚类重排，定首簇单根刀位 → ②（并行流在途，不重叠）removeChild 残余 O(n²) 独立整轮、插入侧残余（S5 ≤ 300 线：s111 臂 / CE loop）→ ③ PR #263 合并后探针复验销账。

**上一状态（2026-10-31，簇 A removeChild 融合列表复用收刀轮，并行流）：R126→R5340 同调用 `_childNodeList` 全量物化复用（R5340-glue，part04 +23 行）落地——tape 归因 sel 域 removeChild 每 removal 恰 3 次物化占墙钟 98%（O(n²)），修复后探针 w2 均值 6661→4109ms（−38%）、物化 6001→4001 次恰 −1/3；四门零位移（engine lib 3,635P/1F s24 已知红唯一、webview lib 832P/0F、inline-host 880P/0F、join 三案 7,518P/0F/0T）**：

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
