# s424b：selector 臂结构性放行实验——双门实验定谳与回退

日期：2026-11-14。承接 s424a 挂账（evidence
2026-11-14-s424a-class-gate-widening-experiment.md §3「清偿需 selector 臂
结构性放行」假设）。**本刀全程未提交**：engine 已回 HEAD 零残留，A/B 工件
存 `.acceptance/wptfyi-css/results-s424b-structural/`。

## 1. 刀口

假设：s423 selector rect 臂撤 family 谓词（对齐 s421 handle 臂无门全重放
语义），anchor-name-mutation 15F 的结构性队列（template clone Append +
class）即可进视图重放翻绿。

实施：callbacks.rs s423 臂 gate 改 `style_version > applied` 单窗口判
（9+/12−，已回退）。

## 2. 受控 A/B（同 peer g38 提交态基座，对照 = s424a 轮 postrevert 跑）

基座：peer g38 五文件 19:02:37 内容 = 已提交 b0e5ac2da（本轮期间 peer 落
远端）；两轮二进制同基座受控可比（s424b 二进制 19:19:04，基座 mtime 全程
未动）。

**实验组（结构性放行）**：265 页 **10,996P/1,857F/4T**——对对照
（12,061P/792F）净 **−1,065**；对 s424a class 拓宽面（11,199P）再 −203
= 结构性额外暴露。

逐案归因（vs postrevert）：

- **−864** writing-modes-001 + −1 金丝雀：class 拓宽已量面（窗口判开臂
  无族过滤，全数复现）；
- **−203 结构性新暴露**：try-tactic-position-area **131** 案 +
  position-area-in-position-try **64** 案 + position-area-value 7 +
  position-try-tree-scoped 1 + zz-g6probe-mut6 1——正是 s423 收窄注释
  在册「position-try fallback 状态机 ~131F」池的实测全形（快照陈旧读
  伪过被视图真读暴露，新失败形 "expected 95 but got 200"）；
- **+4**：dynamic-001/004 + fallback-invalidation（class 面同回）+
  zz-g6probe-mut5 1；
- **anchor-name-mutation 纹丝不动 15F（假设证伪）**：结构性放行后门开、
  视图槽重放照跑，页面读取仍 0——瓶颈不在 gate，在视图重放/物化链上游
  （shim template `.content.cloneNode(true)` + `append` 产出的变异形与
  `apply_dom_mutations_full` 的 `CreateDocumentFragment`/
  `AppendFragmentChildren(ByHandle)` 族支撑是否对上、handle/id 在视图 doc
  的解析位，未再深挖——挂账）。

## 3. 回退决策

净 −1,065 且目标面零交付；双门实验（class 族 −862 / 结构性 −1,065）
定谳：s423 收窄的家族门是伪过池（writing-mode 865 + try-tactic 203+）的
门面依赖，暴露面与 153,000 指标方向冲突；结构性放行亦不服务目标页。
engine `git checkout` 回 HEAD；s423 收窄约定维持。

## 4. 挂账更新

- anchor-name-mutation 15F 清偿前提更正：非「结构性放行」而系**视图重放/
  物化链上游缺口**（template clone 变异形 ↔ applier 支撑面对账 + handle
  视图 doc 解析位）——独立挂账；
- try-tactic/position-try 伪过池实测 203+（131+64+7+1）：清偿 =
  position-try fallback 状态机建模（master 在册挂账域）落地后同批转真；
- 双门补丁形态存档：s424a `.acceptance/wptfyi-css/s424a-class-gate-
  widening.patch`；s424b 形（gate 单窗口判）见本 evidence §1 描述 +
  results-s424b-structural 工件。
