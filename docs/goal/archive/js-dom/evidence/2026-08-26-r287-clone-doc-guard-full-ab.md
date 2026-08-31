# R287 Evidence — clone 侧 doc 守卫 + 九轮全量 A/B（cloneContents 100%）

**日期**: 2026-08-26
**切片**: M4——R287(a) clone docEl 域 + (b) dom 全量 A/B 复核
**改动面**: `part06.js`（cloneContents 的 scParOk 守卫）+ `part23.rs`（+1 单测）
**commit**: `480d6f96f`

## 一、clone 29/31,x：R282 修正的漏移植

R282 修的「doc 的 parentNode 恒 null 是合法形态」**只落了 extract 侧**——
cloneContents 仍持旧守卫使 doc-sc 路径从未执行（sandbox probe：frag=0）。
对称移植 `scParOk` 后：`frag=2[HTML(fc=HEAD,nk=2), #comment]`（HTML 深克
隆 + comment 头克隆，均 plain）。

**教训**：双侧同构的 spec 步骤修正（守卫/抛/塌缩类）须双侧同步落——
extract/clone 的修正清单互为 checklist。

## 二、验证

| 项 | R286 | R287 | Δ |
|---|---|---|---|
| Range-cloneContents | 185P/2F | **187P/0F（100%）** | +2（29/31,x 全解） |
| 其余五套件 | 全 100%（125/4/1840/1840/187） | 同 | 持平 |
| engine 单测 | 2419 | **2420** | +1（r287 doc-sc clone 单测） |

**ranges 域终态：六套件全部 100%**（deleteContents 125P /
deleteContents-in-ShadowRoot 4P / insertNode 1840P / surroundContents
1840P / extractContents 187P / cloneContents 187P）。

## 三、dom 全量 A/B 复核（R278-R287 九轮累计，vs R279 基线双跑）

| 域 | R279 基线 | R287 | Δ |
|---|---|---|---|
| dom 全量 | 52717P | **52778P** | +61，set-diff **0 回归 / 62 fail 消失** |
| dom/nodes | 12663P | 12662P | -1 = 已知 flaky crash 用例（单跑 Pass） |
| dom/events | 579P | 579P | 持平 |
| dom/traversal | 1603P | 1603P | 持平 |
| dom/collections | 49P | 49P | 持平 |
| dom/ranges | 37823P | **37885P** | +62 |

九轮（R278-R287）跨域影响经全量 set-diff 验证零回归；nodes/events/
traversal/collections 域与基线持平。

## 四、R288 靶点

ranges 域六套件 100% 收口后，M4 的 ranges 域剩余失败面只有
compareBoundaryPoints（592F）/ set（240F）/ comparePoint（124F）三大多
未动域 + mutations 超时族（环境慢）。按 ROI：
- **(a) Range-comparePoint 124F 重聚类**（中等簇、可能与既有 cDP 域
  修复有连带）。
- **(b) Range-set 240F / compareBoundaryPoints 592F 重聚类**（大片域，
  先取样归因）。
- **(c) mutations 超时族**（低 ROI 备档）。
