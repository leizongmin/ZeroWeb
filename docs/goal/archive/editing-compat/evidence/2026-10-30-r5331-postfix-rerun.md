# R5331 修复后复测 — run/other 全量（2026-10-30）

**修复**: 3d173dccc（`js_dom_shim/part06.js` R5331——查询回落 in-doc 门对 innerHTML
解析产物经 `_zwSelPendingParent` 槽锚点放行）。上游 M1 基线（dc719ecf0，
`2026-10-30-m1-*-baseline.json`）跑在**修复前** shim 上；本复测用同一分片器
（run-editing-testharness-corpus.sh，每案独立进程 + 外部墙钟兜底）+ 同一 skip
规则复测，pre/post 同口径对照。

## 结果（pre → post）

| 通道 | Pass | Fail | Timeout | 说明 |
|---|---:|---:|---:|---|
| editing/run | 7 → 7 | 96,656 → **49,778** | 0 → 7 | Fail -46,878（-48.5%） |
| editing/other | 2,316 → **2,325** | 7,747 → 7,782 | 10 → **4** | +9P / -6T / +35F |

**零回归**：逐案对照无一案 Fail 增加（top regressions 全为 0）；run 面 +7 Timeout
与 other 面 -6 Timeout 见下。

## 机制（run 面 Fail 减半的构成）

R5331 放行 `querySelector("div[contenteditable]")`（setup 首环），setup 链推进：
第一环（innerHTML setter 的 null 目标）已由上游基线时点的部分修复通过，本修复
打通第二环（compound 查询命中 + `setupDiv` 活体遍历），链断点后移到第三环
——`document.documentElement.cloneNode(true)` 产物树上的
`querySelector("[contenteditable]")`：克隆读链走视图物化子树（R3029 不重放
SetInnerHtml）→ 同 turn innerHTML 产物缺席（探针实证 live 容器 childNodes=2、
克隆容器 0）。全案 Fail 消息形态自 "Cannot set properties of null (setting
'innerHTML')" 转移为 "Cannot read properties of null (reading 'parentNode')"。

8 个大案（multitest 10,092F / delete 7,842F / forwarddelete 7,491F /
insertparagraph 7,195F / justifycenter 6,203F / inserttext / formatblock /
undo-redo）setup 全通后**首次进入真实命令执行面**，子测试量超出 420s/案外部
墙钟 → 页级超时（0 输出，rc 124 记 Timeout）——**不是语义通过**，是执行面
首次运行墙钟不够。其余案 Fail 数与修复前一致（如 backcolor 1,026=1,026、
bold 3,048=3,048），仅消息形态转移。

other 面 +9P / -6T / +35F：部分案 setup 通过转真实断言（净 +9P）；白空间/
insertparagraph 族执行面加深带 35 新 Fail（M2 簇信号）。

## 对 M2 指针的影响

1. **cloneNode 视图/pending 身份分裂 = M2 首簇不变**（全 run 面 49,778F 的
   公共断点；修法需动共享克隆/视图读链，Range/restoreIframe 回归面宽，
   独立立项）。
2. **8 大案执行面墙钟**：真实命令执行首跑即超时——CASE_WALL_S 调优
   （run 通道 420s → 按案需求）或命令执行面性能治理，runner 基建/执行优化
   二选一，M2 首轮一并评估。
3. execCommand 命令语义簇（justify* / stylewithcss 矩阵）在 8 大案解除墙钟
   后才有完整信号——顺序：cloneNode 簇 → 墙钟调优 → 命令语义簇。

逐案四态明细见同目录 `2026-10-30-r5331-postfix-rerun.json`；transcript
（/tmp/editing-{run,other}-corpus.log）不入库，重跑两条 make target 可复现。
