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
