# R359 — d3d-r3 blast-radius 探针复核（实用收口裁定）+ iframe 工厂元素 compound QSA 收口

**日期**: 2026-08-29
**切片**: M1/L2 d3d-r3 领取前复核（同 R356 对 d3d-r2 的处理）+ 探针暴露缺口的轻量修复
**改动面**: `part05.js`（iframe 工厂元素 own QSA compound 支持）+ `part24.rs`（+1 单测）

## 1. d3d-r3 blast-radius 探针复核（三域 × compound/identity/性能三点）

WPT 同构探针（临时 fixture，跑完即删）在 detached 工厂 / iframe 工厂 / 主文档三域实测：

| 面 | detached 工厂 | iframe 工厂 | 主文档 |
|----|--------------|------------|--------|
| compound 查询（`p#pa`/`.x`/`.x.y`）identity | ✅ identity | ❌ MISS（qClsMulti/qsaCmp/sameTurn 全 0） | ✅ identity（append 域 host 快照语义=R309 刻意取舍） |
| `querySelector === querySelectorAll[0]` | ✅ same | ✅ same | ✅ same |
| 查询单价（1k 次 compound） | — | — | 19ms/1k ≈ **19µs/次**（与 R353 W1 0.014ms 同量级——查询不是瓶颈） |

**裁定**：R171「element 上下文本树化」的两条立论均失效——① 正确性面：detached/主文档域
compound 消费面已由 R167/R308 桥归一 + R322 归并覆盖（identity 全绿），唯一实缺口是
**iframe 工厂元素 own QSA 不支持 compound**（R181 简单形态匹配器 + 工厂域不经
`_zwMQueryAll` JSON 往返）——这是补面修复不是本树化；② 性能面：19µs/次 × 真实页面
查询频度 = 无可测痛点（R353 同结论）。**d3d-r3「element/fragment 本树化」按 R356 对
d3d-r2 的同款处理记「实用收口点」结项**；d3e（组合器本树化）随本裁定一并失去独立收益
面（组合器在 iframe 域维持 headless 近似、其余域 host JSON 权威可达）。

## 2. 探针暴露缺口的修复：iframe 工厂元素 own QSA compound

`_zwIframeCreateElement` 的 `querySelectorAll`/`querySelector`（own 简单形态匹配器，
R181/R199）扩展：简单形态 miss 时先走 **`_zwParseCompoundSel`**（R171 共用解析器：
tag/#id/.class×n/[attr]/[attr=v] 组合，含空白/组合器/伪类返 undefined）——解析成功则
本树 walk 逐节点 compound 判定（与 detached-doc `_queryTreeByCompound` 同语义：tag
大小写折叠 + id + class 空白分词全含 + attr `=`/存在性）；解析仍失败（组合器/伪类）
回落空（headless 近似不变）。属性值无引号形态（`[id=pb]`）在调用前本地预加引号——
共用解析器值文法只认引号形式，doc 级消费面落 JSON 由 host 权威处理，本 face 无 JSON
回落，本地归一零共享面风险。

## 3. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 全量 dom sweep（polyfill，333 文件） | **55484P/17F/14T——真实 Fail 集合 15=15 恒等零回归，Pass +4 净正、Timeout -4** |
| WPT 探针复跑（iframe 域） | qCmp/qClsMulti **identity**、qsaCmp=1、qAttrV=1、sameTurn=2、qSame=some——探针 MISS 面全闭 |
| Range-mutations 全族 A/B | appendData 384P / deleteData 564P / insertData 382P / 其余全绿；dataChange/replaceData 2F = 已知集 |
| 文件级门 | QSA 1975 / matches 669 / MO-attributes 42 / getElementsByTagName 19 / getElementsByClassName 3 / Element-children 2 / ParentNode-children 1 全持平 |
| engine 单测 | v8 2485（+1 `test_iframe_factory_element_compound_qsa_r359`：iframe 工厂域 compound 全形态 + 组合器伪类回落 + qS===QSA[0] 断言）/ quickjs 1471 全绿 |
| clippy / fmt | v8 + quickjs 双矩阵 `-D warnings` 零警告 / 无 diff |

## 4. 教训

1. **域误判的系统性形态**（第三次）：R357 首版修 sel 分支而 WPT 用例走 handle 域；本轮
   单测首版在主文档域验证 iframe 工厂面——`document.createElement`（handle proxy 走
   part04 trap→host）与 `iframeDoc.createElement`（plain 字面量走 own QSA）是两个
   querySelectorAll 实现，验证面必须与修复面同域。
2. **复合正确性收益的负判定要随新证据复核**：d3d-r3 的「深结构」标签经探针复核降级为
   「已收口 + 一处补面」——与 d3d-r2 同构（R356 裁定），L2-d3 路线 A 的 d3d 系全部
   实用收口。

## 5. 后续

L2-d3 RFC 状态更新：d3d 系（r1 键统一 / r2 树源 / r3 本树化）全部收口，d3e 随裁定失去
独立收益面备档；**M1（polyfill-live 合一）的 JS 侧实用收口达成**——剩余 = M2（S6 高层
API 去字符串）/M5（V8 default-on，待用户点名）主线。已知 Fail 集合余 15（realm/adoption
族 6 为主力）。
