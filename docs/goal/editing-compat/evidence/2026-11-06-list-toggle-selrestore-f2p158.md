# list toggle 语料收敛 F1 链——knifeA/B' + selrestore 收口（F2P 158 / P2F 0）

日期：2026-11-06。域：insertorderedlist / insertunorderedlist 语料（+ multitest list 面）。f1b 基线 iloop 132F / illist 145F，经 q9→q10→q11 三代门禁收口至 **iloop 61F / illist 58F，行级元组 F2P 158 / P2F 0 / newF 0 / lost 0**（[judge-q11.txt](2026-11-06-judge-q11.txt)，f1b 对 q11）。

## 收口链

- **q9（f1 首代四刀）**：`_zwListIsAllowedChild` 忠实尾段（vendored :1629 每亲 switch + 配对表）、`_zwListNormalizeSublists`（:3904）+ items 前奏、toConvert/disable/otherTagName/outdent 四位点 record-restore、缩进合并新父臂。132→71F / 145→67F，但 **P2F 17 红线**——items 前奏对「选区自 li 直接子文本跨入嵌套子列表」（CAC==li 或在 li 之上）形状过度 hoist，语料 expected 保持嵌套（证据：`<ol><li>foo<ol><li>[bar]<li>baz</ol>` 行 Chrome expected 即 hoist 形、f1b 未归并失败；17 行 P2F 全部 CAC 边界外形状）。首代 CAC 门（`_zwListStrictlyContains`）探明边界后被并行实例 knifeB 评估为粒度过粗回退。
- **q10（knifeA/B'，第三实例 f1-resume）**：`_zwListRetagPreservingSelection`（retag 选区保持包装 4 位点：fixDisallowed div/sep、toConvert utag、enable li 直收）+ items 前奏无条件归并对齐 vendored forEach。F2P=156 保持，P2F 17→9；残 9 全部 qInd-after 读面（7 straddle + 2 plain-retag），树面自 q9 起全绿——读面依赖恢复选区几何而非树形（f1b 树错却 Pass 的行自此归因）。
- **q11（selrestore 两笔，本轮）**：
  1. **disable 出口端点身份回植**：toggle 入口录 (textNode, offset)，出口双端点仍在文档时按身份原位重设（offset 越界钳长）。setTagName（part01c :709）/splitParent 均按 movePreservingRanges 逐子迁移、文本节点身份存活已核。探针 i5-straddle：恢复选区 `@2..@2` 塌缩 → `@1..@2` 跨区，qInd true 命中语料。
  2. **enable-retag 交集钳制**：toConvert retag 后活动选区与换壳子树求交，壳外端点钳到壳内最深文本端。探针 absorb：qIndA true→false 命中语料（全跨度回植会混入壳外文本给 mixed——读面 admission 含 li 允许子 div/text，跨区恢复自然给 mixed，机制闭环）。

## 门禁

| 腿 | q10 | q11 | Δ |
|---|---|---|---|
| iloop | 1,761 行 / 67F | 1,761 行 / **61F** | F2P +4 |
| illist | 2,065 行 / 63F | 2,065 行 / **58F** | F2P +5 |
| 元组（f1b 对 q11） | F2P 156 / P2F 9 | **F2P 158 / P2F 0 / newF 0 / lost 0** | 红线清 |

q11 相对 q10 的 9 行 P2F 全部回植基线（9/9 门行 verifier 逐行确认 Pass）；F2P 158 比 q10 多 2 为 dd/dt compare 与 blockquote 选区面随 selrestore 附带翻绿。multitest 十片：1-9 片 fail 计数 30/24/34/24/50/49/59/47/14 逐位恒定三代（q9/q10/q11），尾片 9001-last 覆盖 958 行（预算截断带），对 jccut 基线元组 **F2P 0 / P2F 0 / newF 0 / lost 15**——lost 全部为截断带内 delete/forwardDelete 家族（fontSize 载体 + table/grid straddle），与 toggleLists 无共享路径，符合尾带 doctrine（q10 轮 verifier 同判）。守卫腿 enginelib（make guarded-test）：**3,740P/5F**，5 红 = js_dom_bridge 五已知恒定零新增。

中途事件入账：q11 批中段 01:15 rally rule-13 同步触发 cargo relink，双 verifier 席独立审计（marker 核对 `_zwListNodeInDocument`=2 / `_zwListRetagPreservingSelection`=5 / `__zwListNewLi`=0 + pull range diff 逐 hunk）确认内容零漂移，1-8 片与 9-10 片+enginelib 同内容可比。

## 残差 119F 与后续

iloop 61F + illist 58F：absorb compare 族稳定红（`foo<ol><li>[bar</ol>baz]` 类期望尾段并入 retag 壳 `<ol><li>bar</li><li>baz</li></ol>`，f1b 即红非本轮回归，q11 轮 verifier 已证 got 树逐字节恒等基线）——static 归因 enable 主循环 contained walk 在 pre-retag 端点上漏收尾段兄弟，probe 17-19 sel dump 定位后为下一主刀；余为 straddle 周边读面零星。UL-I5-div 一行 f1b 稳定红不在门行集。

## 协作账

本链为多实例接力：f1 首代四刀与探针案 7-12（实例 A）、knifeA/B' 与 census 案 13-17（实例 B）、selrestore 两笔与收口（实例 C）同账本（/tmp/qv-fix4/jccut-status.txt）协同，verifier 席（1d32a3a5 / rollover-0114）全程 provenance 与门行复核。判读脚本 judge-q9.py（行名元组口径：variant+description 键，family 剥命令数组前缀）。

## q12-q13 追账（2026-11-06）：strip 补门 + absorb qState 伴行收口（F2P 167 / P2F 0 / enginelib 五红恒定）

### A. q12 补门（strip+absorb 最终内容门禁）

445307514 落地内容含 absorb 臂与 `__zwListDbgTrace` 调试残留（landing 窗口与 strip 竞速，残留未及入 commit）。补门批 q12（01:38-01:43）：q12 二进制 01:38:41 由剥离后 src 构建（bin-mtime > src-mtime，单构建无中途换源）——q12 全腿直接门禁剥离后最终内容：

- 语料 f1b-vs-q12 双腿 **F2P 158 / P2F 0 / newF 0 / lost 0**（行集与 q11 的 158 一致，契约红线清）；
- enginelib js_dom_bridge **3,746P/5F 五已知红逐名恒定**（p56 / p99s-rule-style / r3276 / r2810 / r2809）；
- 探针 absorb 行树 corpus 精确 `foo<ul><li>bar</li><li>baz</li></ul>`（qSA=false 留读，见 B 交换账）。

### B. q11→q12 交换账（净零，非契约回归）

q12-vs-q11 元组 **F2P 2 / P2F 2 / newF 0 / lost 0**：+2 compare 行（iol `foo<ul><li>[bar</ul>baz]` compare innerHTML + iul `<ol>` 镜像 = ⑯ 卡点所指 iol:43/iul:43 absorb 稳定红，absorb 臂翻绿）；−2 同输入 qState-after 伴行回落 f1b 态 Fail（吸收流灭失重建入口文本节点、引擎活范围钩子把端点重锚列表壳沿 UL@2，选中区脱离 li 内容致 qState 读 false）。vs f1b 两行本就 Fail → 非契约 P2F，fails 计数 61/58 恒定。

### C. ⑯ 勘误（两处与日志不符，verifier 席独立判读）

1. 上节「树面自 q9 起全绿」不成立：q10 前奏 CAC 门回归 **10 行 nested compare 树面**（5 iol + 5 iul，家族 `<ul|ol><li>foo<ul|ol><li>[bar]<li>baz</ul|ol><li>quz</ul|ol>` / `[bar]baz</ul|ol>` / `b[ar..b]az` / `[foo..baz]+extra` / `bar<li>[baz]`，跨 ?1-1000 + ?1001-last），q11 selrestore 未愈、q12/q13 未触，仍挂（下一主刀；nested-split 边界根因准备见账本 note-418）。
2. 上节「F2P 158 比 q10 多 2 为 dd/dt compare 与 blockquote 选区面」不成立：全集未截断组成 = **156(q10) + 12 新翻（absorb qS-after ×4 + straddle qI/qS 读面 ×6 + `<p>{}</p>` 空选区 compare ×2）− 10 nested 债 lost**。逐代 direct tuple：q9→q10 iol/iul 各 F2P 10/P2F 6（12 P2F = 10 nested compares + 2 absorb qInd reads，qInd 对由 q11 selrestore 愈，compares 未愈）；q9→q11 iol F2P 15/P2F 5、iul F2P 14/P2F 5。fail 计数序列对账：iol 71→67→61、iul 67→63→58、q12 交换中性 61/58。

### D. qState 伴行收口刀（v1→v3 迭代，enable 出口列表内收钳）

v1 enable 出口端点身份回植不可行——探针实证 barAlive=false：吸收流经 wrap 的 flat-model HTML 重写臂（editing host 直子文本成员）把换壳列表与端点文本整体灭失重建（`capS.n` 脱离文档；retaggedFinal 于主循环后脱挂，探针 `pg rF=UL/-`），身份原位重设无靶。v3 改树位收钳（part01e 出口臂）：端点容器祖先链无列表内容（`_zwListInListContent`）时收钳——容器自身为顶层列表壳（引擎重锚 `(ul,2)` 尾沿形）直接钳入该壳最深首/尾端；否则回退换壳列表（retaggedFinal 存活常形，与 mid-toggle 交集钳制同型）；端点已在任一列表内容内不动（nested/换型行保位，nested ul 判 in-content 不触臂）。探针常驻面（zz-probe-list c16 增 sel dump）：absorb sel=`LI<UL>@0..LI<UL>@1` qIndA=false qSA=true（q12 时 `UL@2..UL@2` qSA=false）、plain-retag/straddle 控制面全绿。

### E. q13 门禁账（02:06-02:08 语料批 + q13c enginelib 补腿）

- 语料 f1b-vs-q13 双腿 **F2P 167 / P2F 0 / newF 0 / lost 0**（行数 1761/2065 恒定）；fails iol 61→**57**、iul 58→**53**。
- q12-vs-q13 **F2P 9 / P2F 0**，9 行全为读面（零 compare innerHTML 位移 → 树中性行级实证）：absorb fixture qState-after ×4（双命令双 fixture）+ quz fixture 读面 ×3（`foo<ol><li>bar</ol><ul><li>[baz</ul>quz]` iol qInd+qState、iul qState）+ fontSize-div 读面 ×2（iul ?1001-2000 dps div/p {1.3em/1.1em}——收钳 sel 几何外溢红利，超 LAUNCHED 预测 +2）；absorb compare :33/:43 保持 Pass（不回赎交换）。
- 探针见 D；mt 免跑：树中性已行级数据背书（零 compare 位移，五席判读一致）。
- enginelib（make guarded-test，q13c 补腿 02:13-02:16）：**3,746P/5F** 五已知红逐名恒一（p56_adopted / p99s-rule-style / r3276 / r2810 / r2809），judge-q13.sh 红集 diff q12→q13 为空；rc=2 系五已知红致 make 非零、非门禁失败。
- 事故：q13 首批 enginelib 腿死于 T58 fmt 前置门（fc10ac3ce gap-batch1 落树未 fmt，testharness.rs 4 hunks 21 行，0 result-lines 5s）——修复属 fmt 席独立 chore 提交（非本链内容，rule-9 归 gap-batch1 流），q13c 对修复后树重跑补腿。

### 协作账（增补）

q12/q13 链多席协同：判读五席一致（rollover-0114 三实例 + 1d32a3a5 + strip-seat，judge-q9.py 独立重跑逐行一致）；seat-boundary 仲裁（fmt 席 = 1d32a3a5 独立提交、enginelib 补腿 + q13 落地 = 本席、blob audit 交叉核）见账本 note-454/458。

### F. q14 nested 前奏归并几何门（enable 逐 item 跳过条件）

q10 disable-only 门的对偶补齐：enable 归并按选区几何逐 item 门控。新增 `_zwListEndpointPathKind(n, item)`（part01e.js，沿 parent 链爬升 ≤256 步：先达 item → 'direct'（端点在 item 直属内容），途中先命中 ol/ul → 'list'（端点在 item 子列表内），不达 → 'outside'）。前奏对每个 item：`起点 kind=direct ∧ 终点 kind=list` 时跳过 normalizeSublists，否则归并；disable 模式保持无条件归并（top-level 合并形依赖）。语义：`[foo<ol><li>bar]` 跨入形（q9 P2F 17 行族，:233 expected 保持嵌套换壳 li）不拆；其余形状（subset-of-N :178、trailing-text :253、trailing-list :208、straddle :240/:245）归并外提——q9 F2P 行集恢复。探针 fixture 首跑踩坑自纠：ul 底必须配 ul 嵌套（iol 语料 ol↔ul 镜像），首版 ol 嵌套令 state=ol=iol tagName 误入 disable 模式，L 族面不可判读；q14b 复跑（fresh binary 未变，仅 probe 数据）取正确 enable 面。

### G. q14 门禁账（03:22-03:29 批 + q14b probe 复跑）

- 语料 f1b-vs-q14 双腿 **F2P 175 / P2F 0 / newF 0 / lost 0**（行数 1761/2065 恒定，F2P>167 达标、红线零回归）；fails iol 57→**53**、iul 53→**49**。
- q13-vs-q14 **F2P 8 / P2F 0 / newF 0 / lost 0**：8 行全为 compare innerHTML 树面，两命令镜像各半——iol `foo<ul><li>[bar]<li>baz</ul><li>quz` 与 `[bar<li>baz]` 变体（subset-of-N，:178/:180）、`foo<ul><li>b[ar</ul><li>b]az</ul>`（straddle-out :240）、`foo<ul><li>[bar]</ul>baz</ul>`（trailing-text :253）；iul 为 ol 基镜像。q9 P2F 17 行族（:233 keep 面）零位移。
- 探针 corpus-exact 4 面（q14b enable 模式）：L1 got `<ul><li>foo</li><ol><li>bar</li></ol><ul><li>baz</li></ul><li>quz</li></ul>` = :178 镜像逐字一致；L2 = :253 镜像一致（尾随文本成新 li）；K1 = :240 镜像一致（双壳 `<ol><ol>` 形）；K3 keep 面不变（门跳过实证）。L3/K2 非 :208/:245 期望形（trailing-list 外提后换壳 li 应保嵌套子列表、straddle-extra 应整体换壳分裂）——该两族仍在 F2P=8 之外，属 nested-split 边界残债余量。
- enginelib js_dom_bridge（make guarded-test，03:26-03:28）：**3,746P/5F** 五已知红逐名恒一，红集与 q13c 日志直接 diff 为空（q13-reds.txt 缺失由本证补齐重建）；rc=2 系五已知红致 make 非零、非门禁失败。
- judge 脚本 keep-family 控制段 grep 空系行格式不匹配（f-string 面名不落日志行），P2F=0 双 bar 已行级覆盖同一断言，不另补腿。

### H. q15 nested-split 三臂（残债八面 census + 门几何修正）

上轮 ⑰ 卡点两族普查纠偏：残债实为 **enable 嵌套八面**（:373/:407/:418/:458/:463/:473/:478/:488 compare），q10 起 stable debt（q13/q14 双日志核对全红非回归）。独立探针页 zz-probe-list15.html（常驻页 dump 消息 3.3KB runner 截断顶格、追加块落末尾 assert 后为死代码——已回退；新页带 PRE 结构 dump + case 级 try/catch）。八面按 vendored toggleLists 全流程模拟分解为三臂：

1. **臂 1 helper 语义修复**：`_zwListEndpointPathKind` 原实现链上先撞无关列表即判 'list'（straddle 端点撞外层壳误判）——改 item 可达才分类，列表介于端点与 item 之间才 'list'，不可达一律 'outside'。:458 li(foo) 端点几何（direct∧outside）恢复归并触发，vendored lift→换型→splitParent+wrap 全流程模拟即 expected。
2. **臂 2 enable 门收窄**：跳过条件从 direct∧list 扩为 **start=direct ∧ end≠outside**（端点不离开该 item 一律保嵌套：跨入形 ：448 与单 li 直选含嵌套尾随形 ：407/:418/:463/:473 同门）；start=list 或 end=outside 仍归并（q14 已绿 8 行几何不变）。
3. **臂 3 换型门**：normalize 跳过后嵌套列表仍被 newRange 包含，enable 换型 pass 会 retag 与 ：407/:463/:473 expected（嵌套保 UL 原型）矛盾——跳过 retag/merge iff list.parent 为 li 且两端点均不在列表内（三分判据：:448 选区跨入换型、:458 外提升 sibling 位 parent 非 li 换型、单 li 直选保型）。

q15 首批（仅臂 1+2）停批重跑（TaskStop 无孤儿锁自由，臂 3 模拟 ：473 retag 矛盾实证后并入，避免混合二进制）；第二批会话终止仅杀记账壳，批进程树存活跑完（判活后让位，LOCK-BUSY 退出按设计让路）。

### I. q15 门禁账（03:55-03:59 批，三臂单构建）

- 语料 f1b-vs-q15 双腿 **F2P 185 / P2F 0 / newF 0 / lost 0**（行数 1761/2065 恒定，恰为预测 175+10）；fails iol 53→**48**、iul 49→**44**。
- q14-vs-q15 **F2P 10 / P2F 0 / newF 0 / lost 0**：十行全 compare 树面，五面 ×2 镜像逐字 = 预测清单（iol :407 `<ul><li>[foo]<ul><li>bar</ul><li>baz</ul>`、:418 `<li>[bar]<ul>baz`、:458 straddle-extra `<p>extra`、:463 `[foo]<ul>bar</ul>baz`、:473 `[baz]` 嵌套后直选；iul 为 ol 基镜像）。
- 控制行：keep 族四形（:448/:393/:453/:463-变体）Pass 恒定 + 预测五行全 Pass（judge-q15.sh 显式 grep，行名对齐语料 fixture 串）。
- 探针 5 面 corpus-exact（q15-2/3/4/5/6 = 语料 expected 形逐字一致）；q15-1（:373 双层换壳）/q15-7（:478 跨入+尾随换型双层）/q15-8（:488 尾文本吸收）未翻 = 预留 q16 转换主流程臂（fixpoint 换壳 retag、direct→nested 双层换型、跨列表 join 尾吸收）。
- enginelib js_dom_bridge（make guarded-test）：**3,746P/5F** 五已知红逐名恒一，红集 vs q14 日志 diff 空；rc=2 系五已知红致 make 非零、非门禁失败。

### J. q16 转换主流程 list-wide 档（:373 双壳 retag + :488 尾吸收落地，commit 05111fbf4）

- 根因两层：①派发层死码——'list-wide' 档插在 `if (chromeTight)` 真值判断之后，truthy 字符串被钳制臂截获，前四批门禁（177/97/97/97）实测全为钳制臂行为，重排 list-wide→钳制→旧 else 后首次真上机（教训入 docs/learnings/bugs/2026-11/2026-11-06-truthy-guard-eats-new-string-mode-branch.md）；②几何层——list-wide 档 end 界 vendored 忠实 at-length 爬升（宿主顶护栏），start 界 peek 纯壳提交（origStart 基准判分支归属）：:373 纯壳链 start 爬至 (host,0)、end 爬至 (host,1)，外层壳入含域 → lists-to-convert 双壳整壳换型。
- 语料（vs q15，批 5）：iol fails 48→**46**，F2P=2 = :373 `<ul><ul><li>[foo]</ul></ul>` + :488 `foo<ol><li>bar</ol><ul><li>[baz</ul>quz]`（尾文本吸收随档顺带落地，原 DEFERRED 面）；iul fails 44→**43**，F2P=1 = :373 镜像 `<ol><ol><li>[foo]</ol></ol>`；双语料 **P2F=0**、newF=0、lost=0；f1b bar **F2P=188≥185**、P2F=0。
- 探针 10 面（zz-probe-list16.html）：q16-1 = `<ol><ol><li>foo</li></ol></ol>`、q16-3 = `foo<ol><li>bar</li><li>baz</li><li>quz</li></ol>` 逐字期望形；六控制面（:448/:407/:463/:473/:458/跨入形）全保形；q16-2/:478 = `<ol><li><ol><li>foo</li><li>bar</li></ol>baz</li></ol>` 仍红（foo 被裹进内层 OL 首 li，期望保位）→ 残债：:478 整壳换型走 split-lift（期望 wholesale 保尾文本）+ qState 读面。
- enginelib js_dom_bridge（make guarded-test）：**3,751P/5F** 五已知红恒一。
- 批间波折存证：批 5 二进制在飞时本档 hunks 曾被并行席回退（04:56:46/04:59:03），验证席终判背书「批 5 源态 = 可落地增量」，工件抢救于 /tmp/qv-fix4/b5/（iul 行级完整、iol 行级以覆盖前判读转储 q16-move5.txt 为证）；落地 = 批 5 实测态逐字节恢复。并行席 sole-child 级联臂（part01e +25 行）留工作区未暂存，待其后批测量，勿随本提交误入。

### J 追记（批7，05:12-05:18）：行级补证闭环 + 级联臂处置

- 批7（⑲+级联臂混合态，单干净二进制 05:12:56，树上 HEAD 71dd00f8e）：iol vs q15 **F2P=2 行级实证** = `:373 <ul><ul><li>[foo]</ul></ul>` compare + `:488 foo<ol><li>bar</ol><ul><li>[baz</ul>quz]` compare——批5 iol 日志被批6 截断导致的行级身份悬案（§J 归因此前系批5 活日志判读推定）由混合态完整日志直接补证闭环；vs f1b F2P=86/P2F=0/newF=0/lost=0。iul vs f1b **F2P=102**/P2F=0/newF=0/lost=0、vs q15 F2P=1（:373 镜像）。双语料 fails 46/43 与 ⑲ 单独态逐值恒一、P2F=0；探针 10 面逐字节同批5（q16-1/3 精确形、六保面保形、q16-2/:478 仍红）；enginelib **3,754P/5F** 五红恒一（+3P 系他流 71dd00f8e EXIF 新增用例，红集 diff 空）。
- 级联臂（armC）处置：混合态对 iol/iul/probe **零增量零回归**——batch6 独立价值（无 list-wide 时经 sole-child 级联翻转 :373 镜像，iul F2P=1）在组合态被 list-wide 完全覆盖，按死代码不入库原则**不落地**；patch 存档 `/tmp/qv-fix4/cascadec-arm-on-⑲.patch`，工作树已还原 ⑲ 干净态。:478 刀法转 DBG 探针实证（前奏 normalize 门/主循环 merge-vs-retag 分派/ARM-B'/armC 四点计数，`__zwDbgQ16` 探针页设旗），foo 进内层首 li 的重挂站点待 trace 定位；纯读码初判嫌疑 = merge 臂（splitParent+wrap）与 normalizeSublists 中间内容聚入臂，retag 本体（纯改名+选区重锚）排除。

### K. q16 :478 direct→nested wholesale 收口（批9 DBG 探针 + 批10 shellArm 重锚）

- **DBG 探针（批9 前半，两轮 trace，旗门控零行为差）**：q16-dbg 面（:478 同参）四点计数 + nodeList/gather/sublistArm 六点扩展。定谳变异链：①`blockExtend('list-wide')` = `DIV@0..LI@2`——baz 尾随文本致 end 爬升止于外层 li 内（非 at-length），外层壳部分含域；②toConvert 只收内层 UL，ARM-B' 按判据 unshift 外层壳 → 双 retag 全干净（此刻树已期望形）；③**nodeList 平铺阶段 foo-text 以 allowedChild(li) 资格混入**（GetContainedNodes 整壳收集不递归内层、retag 后 innerOL 非 otherTagName 不收）→ inline-wrap-li 裹新 li → default-wrap-utag 下 `_zwJusWrap` part01c.js:1016-1025 **邻接 utag 收养臂**（新 li 的 nextSibling = 换型后 innerOL 命中判据）→ foo 收养进内层表 = split-lift 逐字节。retag 本体改名词材纯函数复证排除（:416-487 setTagName+路径重锚，零内容搬移）。
- **批9 v1 修复（shellArm nodeList 排除）判读 = 中性零回归但 :478 未翻**：vs f1b F2P=188/P2F=0、vs q15 F2P=3 恒一批5、enginelib 3,754P/5F 红集恒一、KEEP 九面 Pass、fails 46/43 恒一、probe q16-2 零位移。根因 = setTagName 先插后删换型致 shellArm 旧壳引用 **detached**，containsNode 恒 false 过滤恒空（v1 no-op；本轮门禁即中性证明）。verify-seat round-3 独立归因（retag 分支搬内容）由 trace+读码双重证伪，其 round-3 快照插桩（SNAP tree=undefined 恒 null 产出，爬顶谓词 `contenteditable==='true'` 与裸属性不符）未产出数据，随批10 撤除。
- **批10 v2 修复（shellArm 重锚换型后新壳，retag 分支 `list===shellArm` 时 `shellArm=retagged`）判读全绿**：iol fails 46→**45**、iul 43→**42**，vs f1b **F2P=190/P2F=0/newF=0/lost=0**；vs q15 **F2P=5** 恰 = :373 + :478 + :488（iol）+ :373 镜像 + :478 镜像（iul）五行 compare——预测集逐行命中。探针 q16-2 = `<ol><li>foo<ol><li>bar</li></ol>baz</li></ol>` sel=bar@3..bar@3 qIndA=false qSA=true **逐字期望形**，其余 9 面与批5 逐字节恒一；KEEP 九面 + iul 双面全 Pass；enginelib **3,754P/5F** 红集 diff 空。q16 三 compare 族（:373 双壳/:478 direct→nested/:488 尾吸收）全落地，fail 序列 iol 71→…→48→46→**45**、iul …→44→43→**42**。
- 残债：:478 qState 读面（语料 queryCommandState 行，compare 已翻后独立读面）+ straddle 周边零星（控制面 ②）。
