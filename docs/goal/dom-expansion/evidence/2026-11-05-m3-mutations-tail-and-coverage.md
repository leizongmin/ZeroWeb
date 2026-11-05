# M3 续 mutations 尾簇收口 + coverage 复测（DC-4 收口，2026-11-05）

## mutations 尾簇 8F 根因与修复

M2 挂账尾簇：`Range-mutations-appendChild` 4F + `Range-mutations-insertBefore`
4F，全部为**尾部插入**（`paras[0].appendChild(paras[1])` /
`insertBefore(paras[1], null)`）下边界点在插入位次的 range 被误 bump
（expected 1 got 2）。探针页（`dom/__m3mutprobe.html`，包装 R262/R263 记
每次调用前后 `_startOffsetBase`/`_endOffsetBase`，已删）证认根因：

**R263 `_zwAdjustRangesForInsert` 的 `ensureNewIndex263` registry 快道位次
≠ DOM 位次**。textContent= 建的 textEl 文本子不入 `_handleChildren`，混装
箱（[textEl 文本子, handle 子]）append 后 registry=[a1]、regIdx 0，而融合
视图 [text, a1]、DOM idx 1。「登记序 = DOM 序」前提只在 registry 覆盖全部
视图子时成立；快道 fallback 条件（「registry 缺该子才退 live NL」）漏掉
「registry 有该子但位次错」形态。`offset(1) > regIdx(0)` 误成立 → bump 到 2。
oracle `modifyForInsert` 用融合视图 indexOf=1，1>1 false → 恒 1。

探针同时证认 **R262 `_zwAdjustRangesForRemove` 同款隐患**（同一条注释、同
款快道、同款 fallback 条件）：混装箱 remove 的 oldIndex 取 regIdx 0 而
DOM idx 1，`offset > 0` 的误 −1 属对称潜伏面。

修复（crates/engine/src/js_dom_shim/part03.js 两处，规格锚不变）：
两快道加**完整序前提校验**——registry 子数 == 融合视图子数才可信 indexOf，
不齐退回原 live NL 扫描（原 fallback，语义权威）。快道 O(1) 性质保留在
完整覆盖容器（物化后 R265 形态 registry 即视图，长度相等照走快道）；
不齐形态本就走 O(n) fallback，校验只是前置了同一读。

## 验证

| 层 | 结果 |
|---|---|
| 探针（6 例） | 尾插 unsel/selected ×2 案 + R262 混装 remove 例全绿；前插对照组（须 bump 到 2）维持绿 |
| 定向 mutations 10 案 | **2127P / 0F** / 4T（head 56098 全通道 399F 内该族 8F → 0F）；4T = appendData/dataChange/deleteData/replaceData CASE_TIMEOUT=10s 截断（M2 挂账不变，M3 evidence 同款截断深度波动注记适用） |
| 全 dom 通道 A/B（385 案，head `682f0632d` 二进制 vs 修复二进制，均 test-guard 包裹 --time-limit 1500） | head 56098P/399F/16T → wip 56206P/386F/15T；逐子测试对拍 **0 Pass→Fail、0 新增 Fail**；Fail→Pass 13 = 目标 8 + 探针页 5（head 上探针预期红）；余量 +96/−1 Pass 全落 replaceData/dataChange/insertData 截断深度波动（M2/M3 同款，语义差只取集合差） |
| 全 html-syntax 通道 A/B（同双二进制，--time-limit 1800） | head/wip **逐子测试集合完全一致**（0 翻转、0 缺失）——修复对本通道零影响 |

- A/B 原始 log（gz）：[head-dom](2026-11-05-m3-mutations-ab-head-dom.log.gz) /
  [wip-dom](2026-11-05-m3-mutations-ab-wip-dom.log.gz) /
  [head-html](2026-11-05-m3-mutations-ab-head-html.log.gz) /
  [wip-html](2026-11-05-m3-mutations-ab-wip-html.log.gz)
- 机时注记：html 通道双轮各 ~2min（反射面快案为主，M1 的 ~7min 为另一日
  负载口径）；dom 双轮各 ~14.5min 持平——快道校验无墙钟回退信号。

## coverage 复测（DC-4）

终版二进制全通道数（= 上表 wip 轮）：

**dom/ranges 面（testharness-dom 通道，61 案）**：40,985P / 189F / 4T。
mutations 族 2,213P/**0F**/4T（尾簇收口）；top-level 已实现 API 面语义
正确率 **40,985/41,064 = 99.81%**（M1 扩面基线 99.07%，dataChange 369F +
尾簇 8F 收口贡献）；189F 全部为 tentative 19 案 OpaqueRange/createValueRange
提案接口缺失（M1 挂账不变，0P/189F）。

**html/dom 面（testharness-html-syntax 通道）**：

| 面 | 案 | P | F | T | (M1 基线) |
|---|---:|---:|---:|---:|---|
| 旧面（40 号既有域） | 79(html/dom 前缀口径) | 58,486 | 55 | 6 | 61F 挂账构成不变 |
| 新面（10 叶子） | 149 | 621 | 484 | 1 | 469P/636F |
| 全通道 | 272 | 61,823 | 545 | 7 | 61,671P/697F/7T+1U |

全通道 **+152P/−152F 与 M3 收口簇精确对账**（documents 136 + dataset 16），
旧面 61F 挂账构成零变化（55 html/dom 前缀 + 6 通道其余挂账案）——DC-4
「40 号已收口面零回归」成立（叠加 html 通道 A/B 集合恒等实证）。

**新面 484F 挂账构成（全量归类，DC-4「未覆盖子测试清零路径」口径）**：

| 簇 | F | 处置 |
|---|---:|---|
| innerText 面（getter/setter/outertext/dynamic 四案） | 400 | 渲染依赖型接口，与渲染流域划界后处置（M1 挂账候选转正式挂账） |
| partial-updates 提案面（streamAppendHTMLUnsafe/Sanitizer/`template for=`） | 56 | WICG 提案面挂账不收敛 |
| dir 动态语义（dir-slots-directionality 11 + dir-auto-div-append-child + dir-bdi-script） | 13 | computed 方向语义，25 号 html-semantics 域边界，挂账 |
| documents 残余（embeds/plugins 树序、scripts live、lastModified header） | 3 | runner 管道挂账（M3 documents 收口注记不变） |
| 全局属性散案（id-attribute 4、translate 3、lang 2、historical/custom-attrs/mapped-adopt 各 1） | 12 | 历史/idl 残差散面挂账 |

ranges tentative 189F：OpaqueRange/createValueRange 提案接口缺失挂账，
不做语义收敛（M1 归因不变）。

## 依赖约束核对（run-rules §9）

- 与 33 号 editing 共享 selection/Range 底座：R262/R263 是 selection follow
  removal 的上游，dom 通道 A/B 0 回归覆盖 selection 面 use case；本流只动
  part03 两快道，未触 editing 域文件。
- 与 25 号 html-semantics 划界：dir 13F 维持挂账归 25 号域，本流未动。

## 残余挂账汇总（M3 收口态）

`.window.js` 7 案 / `.xhtml` 12 案形态缺口（M1 结构性挂账不变）；
上表 484F + tentative 189F 提案面；documents 3F runner 管道。本 goal
四里程碑全收口，无进行中项。
