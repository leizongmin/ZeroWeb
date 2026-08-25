# R265 Evidence — textEl ref 的 registry 插入（mutations-insertBefore 0P/超时→76P/0F 100%）

**日期**: 2026-08-26
**切片**: M4——R265(a) R264 定位的死循环根因修复
**改动面**: part04 insertBefore 新增两分支（textEl 注册表形态 + 物化后形态）+
part23.rs（+1 回归单测）
**commit**: 见本轮 commit

## 一、修复机制（物化 + 前插）

R264 定位的根因：refNode 是 textEl 包装（textContent= 建，无 selector 无
handle）+ handle 父时，insertBefore 的三个 wire 分支全不命中 → 插入静默不入
registry → 调用方 common.js indexOf（`while (node != parent.childNodes[i])
i++` 无终止）在融合视图 miss 上自旋（Range-mutations-insertBefore 超时族，
entry 7 单点 90s 复现）。

**修法两分支**：

1. **textEl 注册表形态**（`_zwTextEntryForEl(容器)` 有条目）：
   - 物化——把 textEl 包装从注册表域移入 `_handleChildren`（融合序
     `[textEl…handles]` → registry 完整序），注销 textEl 注册（融合视图
     单源化防 text 双计）；
   - 前插——newNode splice 到 text 位前；host 侧降级 append（无 text ref
     wire 能力，JS 视图权威）。
   - **注销安全性**（handle-only 形态——harness 的 paras 即此）：node 的
     data/appendData 族是 node 自身闭包（`_regWrite` 只写 node.__nv，不查
     注册表）；firstChild/childNodes 走融合视图返**同一 node 对象**
     （identity 保持）。
2. **物化后二次插入形态**（注册表已注销但 refNode 已在 `_handleChildren`）：
   按 registry 内位次直接 splice（无 `__zwHandle` 的 plain 包装形态）。

**过程教训**：首版只做分支 1——回归单测场景④（物化后二次 insert）立即抓到
位次错（`P1;TXT` 漏 P2）：物化后的 refNode 无 handle 无注册表，两分支都
miss 仍静默。**物化是把节点从一个域搬到另一个域，所有消费该域形态的分支
都要补位**。

## 二、验证（vs R264 基线）

| 项 | R264 | R265 | Δ |
|---|---|---|---|
| Range-mutations-insertBefore | 0P/超时 | **76P/0F** | **+76（100%，死循环族解锁）** |
| Range-mutations-removeChild | 20P/0F | 20P/0F | 持平（100%） |
| Range-mutations-appendChild | 70P/0F | 70P/0F | 持平（100%） |
| Range-mutations-replaceChild | 60P/0F | 60P/0F | 持平（100%） |
| Range-mutations-splitText | 116P/0F | 116P/0F | 持平（100%） |
| Range-insertNode | 1841P/0F | 1841P/0F | 持平（100%——text-ref 形态密集，零 P2F） |
| Range-surroundContents | 1840P/0F | 1840P/0F | 持平（100%） |
| Range-delete/extract/clone | 80/160/162P | 同 | 持平（预存簇不变） |
| engine 单测 | 2403 | **2404** | +1（r265 回归单测）全绿 |
| fmt / clippy（workspace） | 干净 | 干净 | — |

**mutations 八套件状态**：removeChild 20 / appendChild 70 / replaceChild 60 /
insertBefore 76 / deleteData 564 / insertData 382 / appendData 384 /
splitText 116 —— **全部 100%**（insertBefore 从死循环超时直接到 100%）。
剩余超时族仅 replaceData/dataChange（累积型慢，R261(a) 归因，无正确性影响）。

## 三、R266 靶点

- **(a) deleteContents 49F / extractContents 32F / cloneContents 29F 重聚类**
  （ranges 域最后三个大失败簇；R260-R265 行为面六轮变化后取样）。
- (b) replaceData/dataChange 超时（累积型慢，低 ROI 备档）。
- (c) mutations 全 100% 后的整域 set-diff 复核（make testharness-dom 全量
  dom/ranges sweep 一次，确认无跨文件漂移）。
