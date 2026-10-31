# CE 环 details 全树扫归因：单点 `_ihMat` 尾扫 ≈9.5µs/根（handle trap 读），成员资格快路落地——bulk fill 墙钟 −20%

**日期**: 2026-11-01
**前置**: [插入侧残余地图](2026-10-31-insert-side-attribution-and-ce-entry-fast-path.md) §4 末项——CE 环残余（seg15 后段）~15µs/子，`__zwDetailsScan4100` 全树扫属 R4100 details 面，独立候选；本轮任务 = tape 细分其触发域与扫描面，语义门内找成员资格快路。
**结果**: ① 归因定谳——scan **不是逐子触发**而是**每个 innerHTML 物化尾单次调用**（part04 `_ihMat` 尾，2000 roots 一把扫），~19ms/2000 根 ≈ **9.5µs/根**，成本 = handle wrapper 逐根 trap 读（nodeType×2 + tagName + childNodes，collect 内三读合计 ~5µs/根 + 根循环 nodeType ~2.5µs/根）；② 成员资格快路落地（part03 +15/part05 +3）：解析/物化面构建期 census 单调闩 `__zwDetailsEver4100`，`isParse && !闩` 直返——**A/B：S8 scan 19ms→~0、zz-s5 wall 中位 133→107ms（−20%）**，移动面照旧（设计不门）。

## 1. 触发域（8 调用点全枚举，tape v5 site 标记）

| 站 | 面 | roots 来源 | isParse |
|---|---|---|---|
| S1 part03 `__zwCeAttachForAdded` 尾 | markup 构造（IAH/innerHTML/outerHTML 全 caller，8 处均解析产物） | `_zwMEl` 系解析树 | true |
| S2 part03 R181 plain innerHTML/outerHTML setter | plain 世界本地解析 | `_zwMBuildBodyTree` 产物 | true |
| S3 part06 `createContextualFragment` | Range 解析 | `_zwMBuildDeepEntry` 产物 | true |
| S4 part05 声明式 shadow root 构建 | `_dsBuildHandleChild` | 主世界 createElement | true |
| S5 part04 R145 template contents 视图 | `_zwMBuildDeepEntry` | `_zwMEl` 产物 | true |
| S6 part04 appendChild 尾 | 脚本移动 | 既有 wrapper | false |
| S7 part04 insertBefore 尾 | 脚本移动 | 既有 wrapper | false |
| S8 part04 innerHTML-into-handle 物化尾（`_ihMat`） | t8r 物化 | `_wcRebuildAsHandle`（源树 `_zwMEl`） | true |

## 2. 归因（tape v5：site 计时 + collect walk 三读分列，3 样本，测后全剥）

| 窗口 | wall 中位 | scan 触发 | scan 耗时 | 判读 |
|---|---|---|---|---|
| zz-s5（handle fill ×2000） | 133ms | **S8 ×1（2000 roots）** + S6 ×1 | **15/23/19ms** | 主项 = S8 单次全扫 |
| zz-reset（sel 重设 ×2000） | ~85ms | S1 ×1（2000 roots） | 1/1/4ms | **同形扫 5-19× 便宜**——roots 是 plain `_zwMEl` wrapper（本地属性读） |
| zz-ap（appendChild ×250） | 44ms | S6 ×251（1 root/次） | 2-3ms 合计 | 次要（~10µs/调用，含 wrapper/tape 开销） |

S8 内部（walk.visits=2001，三读计时）：nodeType 1-6ms + tagName 4-7ms + childNodes 4-6ms ≈ 5µs/根，加根循环 `r.nodeType`（collect 外）~2.5µs/根 ≈ 9.5µs/根——全部是 **handle wrapper 的 trap 读**（t8r 物化产物为 identity proxy），与 [R350 先例](../../../../docs/learnings/bugs/2026-10/2026-10-14-js-dom-wrapper-family-identity.md) #26 trap 域成本模型一致。**无 details 页 frag 恒空，scan 全程 no-op 但逐根 trap 照付**。

上轮地图「seg15 残余 ~30ms ≈ details 全树扫 15µs/子」据此**修正**：~15µs/子的口径把 `_ceApplyConn` walk 常数与 scan 合计摊到每子；本轮钉死 scan 自身 = 单次调用 ~19ms（≈9.5µs/根），CE 环其余为 `_ceApplyConn` 常数（无 host 往返，已净）。

## 3. 快路设计与等价性

```js
// __zwDetailsScan4100 入口：
if (isParse && !globalThis.__zwDetailsEver4100) return;
// census 两点：_zwMEl 构造（tag 已知）+ _dsBuildHandleChild（主世界 createElement 面）
if (tag === 'details' || tag === 'DETAILS') globalThis.__zwDetailsEver4100 = true;
```

**等价性**：scan 行为是 frag（roots 子树内 DETAILS 集）的纯函数，frag 空时无任何副作用（toggle notify/互斥关闭全在 frag 循环内）。闩 false ⟹ 本世界从未构建过 DETAILS ⟹ 解析面批次（构建产物）必无 DETAILS ⟹ 跳过等价。**单调不清零**：任一 build 见过 details 后永久全扫（原行为），零回归方向。

**移动面（isParse=false）不门**——roots 为既有 wrapper，主文档初始树是 host 侧懒投影（querySelector/getElementById/childNodes 首触投影），**不经任何构建工厂**（分离文档/解析产物才走 shim 构建），census 对其不可靠；zz-ap 的 S6 成本本就次要（2-3ms/251 调），不门即零风险。

**census 完备性（门内六面逐一核验）**：S1 全部 8 个 caller（part04:10253/10294/13409/13434/13798/13845/13863、part03:19586）均传解析产物；S2/S3/S5 = `_zwMBuildBodyTree`/`_zwMBuildDeepEntry` → `_zwMEl`；S8 `_ihAdded` = `_zwFragmentAdded` 产物（13186→13247 只读不复赋，R294 纯文本重赋在物化之后且非元素）→ `_wcRebuildAsHandle` 源树亦 `_zwMEl`；S4 = `_dsBuildHandleChild` 单独 census。克隆/adopt 传递（源元素已 census）；`createElementNS` HTML-ns details 只经移动面进批次（不门），不需 census。

## 4. A/B（同探针同页，tape 在树，3 样本）

| 指标 | 修复前 | 修复后 | Δ |
|---|---|---|---|
| zz-s5 S8 scan | 15/23/19ms | **1/2/2ms**（Date.now 量化底） | **→~0** |
| zz-s5 walk.visits | 2001 | **1**（仅 S6 容器根） | 全扫消除 |
| zz-s5 wall | 198/133/110（中位 133） | 152/107/99（中位 107） | **−20%** |
| zz-reset S1 scan | 1/1/4ms | 2/2/2ms | 量化噪声带 |
| zz-ap S6 scan | 2-3ms/251 | 1-3ms/251 | 不变（不门 ✓） |

## 5. 门禁

| 门 | 结果 |
|---|---|
| editing 九语料面 | join 7,518P/0F + ihost 880P/0F + wss 1104P/0F + ilb 552P/0F + chd 420P/0F + void 72P/0F + 48P/0F，**零位移** |
| engine lib | 3,642P/1F（s24 已知红唯一，逐名 identical） |
| webview lib | 834P/0F |
| bench-gate | PASS（自链收尾 load 高 ABORT 一次，load 降后复跑 PASS） |
| html-semantics（新面，首跑无基线） | 5,170P/558F——`the-details-element` 语料 0 案（GitHub API 间歇 403，fetch 设计可续拉，挂下轮续跑）；Fail 集中 img 解码（EncodingError）/ customizable-select / textarea wrapping，无 details 相关案 |

快路面旁证：html-semantics 全量（表单/元数据/嵌入式内容，本面首跑无基线）Fail 案无一涉及 details/summary 语义；census 单调闩语义 = 见过 details 即永久原行为，行为分叉仅存在于「全窗未构建过 details」前缀段，该段 scan 本为 no-op。

**rebase 复验**：收口前并入远端 6 提交（s192-s194 engine shim + R5369 layout，part03 自动合并零冲突），合并树复验——parts 拼接 `node --check` PASS、三处插桩完整、ilb-wss 552P/0F 恒等、join-different-white-space-style-paragraphs 3,456P/0F/0T（全过，含既往预算墙 Timeout 案清零）。

## 6. 复现入口

- 探针页：`tests/wpt-runner/wpt-data/editing/run/zz-probe-s5-insert.html`（gitignore 域保留）。
- tape v5 复建：§1 site 标记清单 + §2 三读计时点（scan 入口/出口包装 + `_zwDetailsCollect4100` walk 内 nt/tag/kids 三段）。
