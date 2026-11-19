# editing-compat M2 首轮 — innerHTML 物化身份失步修复（2026-10-30）

M1 定位的 run 面**单根因 setup 级联**（99.96% Fail = `Setup must not throw
an exception`）本轮修齐。根因不是单点，是 pending-apply 架构下四个共享路径
缺口的合成：

## 修复清单（js_dom_shim 四笔，全部探针复验）

1. **matches 自含语义**（part03 `_zwMQueryAll(n, sel, includeSelf)` +
   `Element.prototype.matches` 调用点）——innerHTML 顶层解析产物（
   `_zwMEl`，parentNode 是锚 proxy，root walk 第一跳即断于 this）查询面
   就是自身序列化，R188 的 QSA 根镜像自剔把唯一候选删光 → `el.matches`
   恒 false。matches 语义自含（`dom-element-matches`：context object
   匹配即真），此形态传 includeSelf。探针 F3/F4/F5。
2. **doc 级 querySelector 解析产物 in-doc 判定**（part06 R145/R3902b 门）——
   纯解析视图（无 sel 无 handle）在 `_zwMutationInDoc(undefined, undefined)`
   首跳 `!h` 即 false，document 级查询 fallback 链的 in-doc 门把同 turn
   innerHTML 产物全跳过 → `querySelector('div[contenteditable]')` 返
   null（runConformanceTest 开场）。本轮以 plain parentNode 链上行到最近锚
   祖先判定；**并行撞车收敛**：同日另一会话流已落 R5331（3d173dccc，同门
   的 `_zwSelPendingParent` 槽锚点形态，`_zwPendingNodeInDoc`），rebase 时
   两者折叠——槽锚点为主臂，链上行并入为深层视图（≥2 级、无槽无身份）的
   fallback，本侧独立实现删除。探针 G1。
3. **cloneNode 深克隆源读走 R380 融合**（part04 innerHTML getter 尾 +
   cloneNode deep 臂共用 part05 抽出的 `_zwFusedInnerHtmlSel`）——sel 源
   克隆此前读 host 直串（apply 滞后旧树），pending 子不进克隆。探针 G5。
4. **子树脏门 ID 锚定 sel 感知**（part05 `_zwPendingSubtreeDirty` 补
   `__zw_contains` 祖先判定 + `_zwPSDEpoch` 记账代际缓存）——host 静态
   元素 sel 方言按最近 id 祖先截断（`#wrap` 下的 `#tc`），R3254-E2 的
   `sel + ' > '` 前缀扫描对 ID 锚定后代天然失明 → 祖先层
   `innerHTML`/cloneNode 融合门 miss 回落 stale 串，**克隆内层 registry
   空**（G5 的第二层）。前缀命中仍走快路径零 host 调用；缓存按记账批次
   作废。诊断：template 深解析反证解析浅化（D16）、E1 wrap-clone A/B
   收敛变量、E3 `__zwSelector` 直读拿到 `#wrap`/`#tc` 形态定位。

探针：[2026-10-30-innerhtml-materialization-probe.html](2026-10-30-innerhtml-materialization-probe.html)
（F1-F6 存档 + 结果注记）；G/E 系列探针为临时文件已删，形态记录在本文件。

## M2 首轮基线（修复后全量重跑，对照 M1）

| 通道 | 案（有效/扫描） | 子测试 | Pass | Fail | Timeout | 通过率 | M1 对照 |
|---|---|---:|---:|---:|---:|---:|---|
| editing/run | 42/43 | 15,343 | 7,485 | 7,838 | 20 | **48.78%** | 0.007%（7/96,663） |
| editing/other | 86/96 | 7,505 | 2,404 | 5,096 | 5 | **32.03%** | 23.0%（2,316/10,073） |

**口径警示（分母不可比）**：run 面子测试分母 96,663 → 15,343。M1 形态是
「setup 每子测试快速失败」（全分母记账）；M2 形态是「setup 解锁后变体
要么跑真实断言、要么在编辑循环自旋到 90s 脚本超时被整变体中止」（
`declared tests: 0` unreported，如 bold.html?3001-last）——自旋中止的
变体不产子测试行。分母收缩本身就是**自旋簇浮出**的证据，不是回退。

other 面 Timeout 10 → 5（M1 的 4 案外部墙钟杀 + 6 自旋子测试，本轮
5 全部为子测试级 timeout——外部墙钟杀清零，同批 mutation 下编辑循环
不再无限滞留）。

## 失败簇重排（M2 后续入口）

1. **run 面自旋簇（新头号）**：编辑命令循环（bold/justify 等
   ?3001-last 变体形态）90s 脚本超时整变体中止——run 面分母收缩主因，
   下轮先归因（与 other 面 list/inline delete 自旋同根因概率高）。
2. **run 面真实语义 Fail 7,838**：setup 解锁后的真实编辑命令断言
   （compare innerHTML 矩阵、stylewithcss 族）——按 M1 既定顺序
   white-space join → insertparagraph → execCommand 归簇。
3. **other 面既有簇延续**：white-space join ~4,700 与 execCommand
   text-editor 簇形态未变（M1 分簇仍有效）。

## 跨域核对（run-rules §9）

- **html-syntax 通道（40 号，99.86% 收口态）**：全量 122,680 Pass /
  126 Fail / 14 Timeout = 99.90%，Fail 集中 render-blocking（挂账
  P-render-blocking）+ html5lib（挂账 Timeout）+ unclosed-svg-script
  ——与收口态残差同族，零回归。
- **dom-expansion（34 号）**：goal 尚在 M1 语料预置未启动，无基线可
  对照；其 html/dom 面与 html-syntax 通道重叠，由上条覆盖。
- **make test**：3600 passed / 9 failed——9 个失败全部是 fetch/Request/
  Response/ServiceWorker 族（`fetch_sync_wire_preserves_response_contract`
  等），`git stash` 隔离实测**在未含本轮改动的 main HEAD 上同样失败**
  （非本轮引入；该族属 net/service-worker 域，不动，留并行流域归因）。
  本轮触碰面的 lib 测试（js_dom_shim 相关套件）全绿；clippy
  `-p zero-engine --all-targets -D warnings` 过；fmt 过。
- 并行流碰头记录：本轮窗口内检测到 html-semantics 域语料跑批（另一
  会话流）与本人 shim 改动并行——js_dom_shim 是共享面，若该域本轮
  结果异常先核对本轮四笔（尤其 R188 includeSelf 与 R145 门放宽）；
  另请该流注意上述 fetch 族 HEAD 既有红。

## 运行与证据入口

`make testharness-editing-run` / `make testharness-editing-other`
（分片 `EDITING_CORPUS_SHARD=k/N`，**k 从 0 起**；续跑
`EDITING_CORPUS_RESUME=1`；并行分片须各设 `EDITING_CORPUS_TRANSCRIPT`
——JSON 汇总从 transcript 解析，共享 transcript 时后启实例截断先启
实例的账，本轮实测踩坑）。分片 k 语义（0-indexed）与 transcript 要求
已记入脚本头注释。
