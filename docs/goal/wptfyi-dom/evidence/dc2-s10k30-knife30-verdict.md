# DC-2 切片 10 刀㉚ verdict——CE adoptedCallback 跨文档派发闭环＋reactions 跨文档 prepend·insertNode 簇收口

**2026-11-26 · 记录腿 HEAD `e60306098`（单提交刀体，rebase 于他流 k136 `1ad3cf356` 之上；guard PASS 0 发现 0 提醒在案）**

## 结论

- **WC 语料净 +92**（F 1115→1023，raw P 17573→17522＋gethtml 截断带 −143 见下）：三页对称簇 89 行（adopted 66＋connected 19＋disconnected 4）＋reactions 跨文档 3 行（Range insertNode／ParentNode prepend／Node insertBefore——后者为本刀 hook 意外面，同根因真增益）。
- **NEW=0、T 20==20 键集 SET-IDENTICAL**（solo gethtml T=1 为 solo 墙钟伪影，全腿未现）。
- **四邻电池 @e60306098**（battery-k30 四腿 vs k29 电池 @a842bf953）：DP FT 223==223（P1313/F222/T1 精确）＋SAN FT 51==51＋DOM FT 376==376（F/T 364/12 恒等）＝三邻腿 SET-IDENTICAL；DOM 腿 Pass 漂移仅 Range-mutations-replaceData +54／dataChange −8＝master.md 碰撞记录在案的停摆漂移族（双向历史观测，非本刀非 k136）。
- gethtml（shadow-dom/declarative）P 3505→3362（−143）：巨页截断落位噪声（F 集 1==1 恒等，页键零 Fail/Timeout 漂移），与刀㉙ 同注记口径。
- **累计 84,090（距 ≥99,947 差 15,857）**。

## 码刀清单（4 文件 389+/27−）

| 面 | 刀 |
|---|---|
| part03 `_ceApplyConn` | adoptOnly 第 4 参（walk 栈 `{n,d,ao}` 子代继承：_ceConn 不标、ccb 不派、_ceConnDoc 记账对齐）＋registry 空表快径（全局 `_ce_registry` for-in＋`_zwCeRegEntries.size` 双源 O(1)——零 CE 大 shadow 树页 O(n²) walk 根除，gethtml WC 腿 2736→3345+）＋TEMPLATE 宿主下钻门（contents inert 不传播） |
| part03 `_ceParentConnected` | marks 优先＋proxy parentNode 结构上行兜底（快径跳过 define 前 'prop' 记账 → define 后首插误判父断开＝reactions 首子测 got [] 根因） |
| part03 派发面 | R194b light shadow.appendChild CE 派发（host 上行连接性；detached→adoptOnly）＋R2815 docEl appendChild/insertBefore CE hook（先派后 adoptAll330＝spec `concept-node-adopt` pre-insert 序，oldDoc 不得读 adopt 后 ownerDocument）＋`_zwMElAppendChild`/`_zwMElInsertBefore` 环 A 及对偶 hook（限定树文档 ≠ 主文档） |
| part05 | R177 合成 docEl appendChild（CE hook＋fragment 展平臂＋removeChild 断开臂）＋R179 factory shadow 同臂（iframe host 自带 attachShadow 产物） |
| part02 | `_zwXmlNode.appendChild` 先摘旧父（Moving 形 disconnected）＋fragment 展平臂；`createElementNS` 补缺（XHR doc 无该方法＝probe6 nsErr 实证；ownerDocument 接线＋attachShadow 委托） |
| part04 | template content 视图 appendChild/insertBefore 改 adopt-only（spec the-template-element：contents inert 只 adopt 进 contents owner doc 不派 connected——旧全量派发污染 `_ceConn` 使真移出被 `!was` 门吞） |

## 回归插曲（v2 registry 快径 → reactions 6 页 9 行红 → 双 hook 收口）

- 归因链：v2 快径（gethtml 回绿所需）跳过 define 前 'prop' 记账引发两类断链——① `_ceParentConnected` marks 缺失（marks-first＋结构兜底修）；② **跨文档 prepend/insertNode 的真落点＝plain docEl 自带 insertBefore（R2815 字面量，R219 own 委派直达，非 `_zwMElInsertBefore`）＋_zwMElInsertBefore 同缺 CE hook**——v2 期归因 `_prependHandleVariadic`（handle 容器路径）有误已订正。
- 收口：R2815 docEl insertBefore 补 `_wcCe330i` hook（环 B 对偶，hintDoc=doc 直传）＋`_zwMElInsertBefore` 尾部环 A 对偶 hook。六回归页行级 multiset vs k29 基线：Range 唯一 flip＝insertNode 跨文档 Fail→Pass、余页行集全等（FieldSet name／OptGroup label 的 attributeChanged F＝基线既有面非本刀域挂账）。
- 判读工具坑双踩在案：基线 battery 日志 grep 二进制失效（提取一律 `LC_ALL=C awk`）＋页名子串误配（`connected-callbacks` 撞 `disconnected-callbacks`，页锚须含 `custom-elements/` 前缀精确匹配）。

## 验证面

- adopted-callback solo 71P/0F（基线 5P/66F；v7/v8 恒）；connected/disconnected solo＝基线形＋19/4 翻绿，残 8F＝shadow host 簇基线恒（本刀域外挂账）。
- 探针定根因五站（zz-s10k30-adopt-probe1..7，故意失配内嵌 JSON 走 Fail 行；交付前已全删）。
- 质量门：lei-pre-commit-guard PASS（4 文件 389+/27−，0 发现 0 提醒）；fmt/clippy 不适用（纯 .js shim 改动）；battery 脚本单锁内层化＋DIRTY=0 门（/tmp/s10k30/battery-k30.sh，HEAD/DIRTY 断言在 driver log）。

## 残池移交

- connected-callbacks shadow host 簇 8F＋FieldSet/OptGroup name/label attributeChanged F（基线既有，attribute reaction 面独立刀位）＋Node.html replaceChild 跨文档 F（replaceChild 路径未接 hook，语料有钉可后续收）。
- gethtml 巨页截断带（−143 至 −714 历史带内）与 Range-mutations 停摆族为判读噪声面，不计净增。
