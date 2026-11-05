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
