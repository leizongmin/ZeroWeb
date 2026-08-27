# R311 Evidence — CDATA 计入 textContent 联接（Node-properties 2F 全解；MO 4F 确认深结构域）

**日期**: 2026-08-27
**切片**: M4/L2——R311(b) Node-properties 2F 归因修复 + MO 4F 域确认
**改动面**: `part04.js`（textContent 联接的 CDATA 纳入）+ `part24.rs`（+1 单测）

## 一、成果

| 套件 | 基线 | R311 | Δ |
|---|---|---|---|
| Node-properties | 724P/2F | **726P/0F（100%）** | +2P/-2F |
| Node-textContent | 81P/0F | 同 | 持平（PI/Comment 排除语义零冲突） |
| Node-cloneNode / ParentNode-querySelector / Element-matches / MutationObserver | 基线 | 同 | 持平（cloneNode 145P 复核无漂移） |
| engine 单测 --lib | 2448 | **2449** | +1（r311 CDATA/Comment/PI 三态断言） |
| make test | 1F 环境项 | 同 | 持平 |
| fmt / clippy | — | 干净 | — |

## 二、根因与修复

**用例**（WPT dom/common.js setupTestNodes）：`paras[5].appendChild
(xmlDocument.createCDATASection("1234"))` ×2 + `paras[5].append("9012")` →
`testDiv.textContent` 期望 `"123456789012"`（CDATA+CDATA+Text 拼接），旧只拼 Text
得 `"9012"`。

**根因**：R184 把 **normalize 的「exclusive Text」口径**误套到 textContent——spec
`dom-node-textcontent` 的字符数据联接含 **CDATASection**（`CDATASection : Text`，
字符数据语义），comment/PI 才是排除项。R184 当时依据的 Node-textContent 套件只测
PI/Comment 排除（无 CDATA 元素内形态），与本修零冲突。

**修复**：part04 textContent 联接 wire 的 `_tcn.nodeType === 3` 扩为 `=== 3 || === 4`
（一处）；R184 注释勘误说明。

## 三、MO 4F 域确认（不修，深结构备档维持）

- **document.html 3F**（parser insertion / script insertion / removal during
  parsing）：**parse-time MO**——host 解析期间的 mutation 不产 MO record（JS 侧
  observe 早于解析完成），已知 parse-time 深结构（master.md 备档维持）。
- **cross-realm-callback-report-exception 1F**：`frames[0].document.body`（iframe
  工厂 body 视图）不可 observe（工厂 append 无 `_mo_notify` + 节点无 handle/sel 使
  `_mo_id` null → observe 静默丢弃）——R302 已归因 **R220 工厂节点可观察 id 深结构域**
  （机制件 R302 已全通，断点正是工厂可观察性），备档维持。

## 四、sweep 状态

R310 后台全量 sweep 持续运行中（96.7% CPU 正常推进，~30 分钟量级）——结果核对转
R312(a)。

## 五、教训

套件驱动的 spec 修正（R184「按 WPT 为准」）要核对**该套件是否真的覆盖了被收窄的
语义面**——Node-textContent 无 CDATA 元素内用例，R184 的收窄是「未被测试检验的
过度泛化」；两个套件的期望差异（Node-properties vs Node-textContent 的隐含口径）
正是 spec 原文的精确分界（character data vs exclusive Text）。
