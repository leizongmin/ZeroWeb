# R244 Evidence — Document 容器 doctype contained surround HRE（25/26,x 12F 簇全解）

**日期**: 2026-08-25
**切片**: M4——R244(a) 25/26,x Document 容器 HRE 12F
**改动面**: `part06.js`（surroundContents 入口 doctype-contained 检查）+ `part23.rs`（r244 单测）
**commit**: 见 master.md 本轮记录

## 一、根因（R244-probe 探针 24 行实证）

WPT `Range-surroundContents` 25/26,x（`[document,0,document,1/2]` ×
元素 newParent j=0,4,6,9,11,13）期望 HIERARCHY_REQUEST_ERR 而 host
NO_THROW。探针（restoreIframe 后 dump sim/host 结果 + doc 子形态）：

- iframe doc = `[doctype(#10), html(#1)]`（restoreIframe 保 doctype）。
- range `[document,0,document,1/2]` 的 contained children 含 **doctype**
  （25,x：doctype contained；26,x：doctype+html contained）。
- sim（common.js `myExtractContents`）步骤 9：「If any member of contained
  children is a DocumentType → HIERARCHY_REQUEST_ERR」——surroundContents
  步骤 3 调 extractContents，HRE 原样上抛且**树不变**（spec
  `dom-range-extract-contents` 同款规则）。
- host 对元素 newParent 无任何拦截：partial 检查不命中（sc===ec 无
  partial）、nodeType 白名单过（元素合法）→ 走 `_coveredChildren` 路径
  实际变更树/空转——不抛。

探针行示例：`25,0 sim=[HIERARCHY_REQUEST_ERR] host=[NO_THROW] …
range=[[document, 0, document, 1]] node=[paras[0]]`；j 非 6 元素族
（0,4,6,9,11,13）全部同形态；文本族（1,2,3,5,7,14–19）host 已因
Text-入-Document 校验抛 HRE（基线即过）。

## 二、修复

`part06.js` `surroundContents` 入口（Attr 检查后、`_coveredChildren` 前）
新增 `_r244DoctypeCheck`：

- **阈值门**：仅 `commonAncestorContainer.nodeType === 9`（Document）时
  扫描——doctype 只能挂在 Document 下，其余容器零成本跳过。
- **contained 判定**：sc===ec===cac 时区间算术（`idx >= so && idx+1 <= eo`）；
  跨容器形态 sideIdx（边界容器是 cac 直接子 k 的后代 → 落点该子；是 cac
  自身 → 落点 offset）。
- 命中 → 抛 `'A DocumentType node cannot be extracted.'`
  HierarchyRequestError，**树不变**（与 sim 步骤 3 先抛语义一致——
  extract 未执行任何树变更）。

## 三、验证链（vs R243 基线）

| 项 | R243 | R244 | Δ |
|---|---|---|---|
| Range-surroundContents | 1794P/46F | **1806P/34F** | **+12 纯增**（F2P=12 / P2F=0 / P→F flips=0） |
| ranges 全量（除 probe） | 38664P/1416F | **38676P/1404F** | set-diff **12 Fail→Pass / 0 Pass→Fail** |
| Range-insertNode | 1841P/0F | 1841P/0F | 0（100% 保持） |
| dom/nodes 失败集 | 57 | 57 | **逐条一致**（diff exit 0） |
| dom/events | 578P/7F | 578P/7F | 基线值不变 |
| dom/collections | 49P/0F | 49P/0F | 基线值不变 |
| dom/traversal | 1602P/2F | 1602P/2F | 基线值不变 |

- **native 同值**：ZW_NATIVE_DOM=1 surround 1806P/34F，与 polyfill
  逐 subtest 一致（sort+diff exit 0）。
- **engine 单测**：2391 全绿（新增 r244 单测：A/B 形态 HRE 树不变 +
  C 正常 wrap 零误伤 + D 跨容器零误伤四断言）。
- `make test` 1F 为 XOpenDisplayFailed 环境项（run-rules §10，历轮一致）。
- fmt/clippy（`-D warnings`）干净。

## 四、12 个 fixed subtests

25/26,x × {paras[0], foreignPara1, detachedPara1, detachedDiv,
foreignPara2, xmlElement} 的「resulting DOM」行（每行即
`assert_throws_dom` 的 did-not-throw 消除；对应 position 行本就因
assert 序 Pass）。

## 五、R245 靶点（34F 重聚类）

| 簇 | 计数 | 行 | 备注 |
|---|---|---|---|
| differing | 13 | 17,x 9 + 13/14,x 4 | 17,x position（cDP 解锁后新前沿——expected "[object Object]" 形态） |
| startOffset | 11 | 16,x | harness-iframe index 算术 |
| other | 6 | 17,x 3 + 30,x 2 + 28,x 1 | 残余 |
| assert_unreached | 2 | 18/19,x | 残余 |
| HRE | 2 | 18/19,x 各 1 | 残余 |

- **首选**：17,x position 9F（`[foreignDoc.documentElement,0,…,1]`——
  cDP 解锁后暴露的 host/sim 深分歧）。
- 次选：16,x startOffset 11F（harness-iframe index 算术）。
