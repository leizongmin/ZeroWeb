# R352 — data 族残余归因反转：adjust 扫描已非瓶颈（removed-marked 容器快道为防御性收尾）

**日期**: 2026-08-29
**前置**: R351（trap 键读顶部短路 + 条目级根缓存）后 dataChange/replaceData 仍文件级 Timeout。

## 1. 归因过程（探针链，含两次反转）

| 探针 | 结果 | 阶段结论 |
|---|---|---|
| R352 基线 | 300 轮尾部 93ms/iter（R350 时代 236 → 2.5x 已改善） | 残余仍随注册表增长 |
| W2/W3 data 写 × 注册表 | 0.001ms/op 恒定（同树/跨树皆然） | **R260 已完全修好** |
| W6 adjust ON/OFF（60 条 stale） | ON 47.8ms / OFF 1.5ms | 表象指向 R262 |
| W8/W9 removed 表快道（单层） | 无改善 | 标记只挂摘除顶点，子孙未标 |
| W11 工厂 plain 节点属性读 | nodeType 62ns / parentNode 32ns | 工厂域 plain 全快 |
| W12 插桩分账 | **R262 本体仅 25.8ms / rm 总 506ms** | 95% 在 R262 之外 |
| W13 直接引用排除 getElementById | rm 22.4ms，R262 占 21.3ms（95%） | **反转为 R262 本体** |
| W15 等价轻量 stub | **0.06ms** | **再反转：W13 的 21.3ms 是游离树堆积下的测量形态差异** |

**最终归因**：R262 对 stale 注册表的扫描在干净页面上为 **0.06ms**（R350/R351 修复已把 adjust
成本打掉）。残余瓶颈 = **setupRangeTests 每轮 `querySelector("#test")` + `removeChild` 在
游离树持续堆积的页面上的查询/文档生命周期成本**——非 range adjust 域，属 host doc/L2 邻域。

## 2. 本轮落地的防御性修复

`_zwDeadContainer352(cont)`：容器自身 sel/handle 或经 `_zwNodeParent` 反向链（plain 表，
64 跳防环）上行的祖先在 removed 标记表（`_zwRemovedSels`/`_zwRemovedHandles`）→ 判死树
容器。接入 R260 `sameNode260Verified`（identity 分支后）/ R262 `inRemovedSubtree262` +
`sameParent262Verified` / R263 `sameParent263Verified`。

**语义安全**：已摘子树的边界点在摘除那次 R262（pre-remove）已重写到活树父——重写过的
条目容器是活树节点（不在 removed 表）；未重写的孤儿条目键属死树，活树 mutation 的
identity/键命中不可能成立。identity 命中路径不经过本快道（调用序保证）。

## 3. 量化

- dataChange declared 426→448（含插桩）/ 428（纯净版，≈噪声级 +0.5%）——微幅正收益
- 三绿文件保持全绿：appendData 384P / deleteData 564P / insertData 382P
- 既有套件零回归：extractContents 192P、insertNode 1841P、MO-attributes 42P、surround 1840P

## 4. dataChange/replaceData 完整收口的前置条件（记 L2）

setupRangeTests 每轮游离树堆积 + `querySelector("#test")` 的查询成本随堆积线性增长。
真浏览器此成本不存在（游离节点被 GC、查询不扫）。shim 侧对应物 = host doc 生命周期
（游离子树可回收标记）或 L2 live Document（查询直读 live doc，无游离面）。
