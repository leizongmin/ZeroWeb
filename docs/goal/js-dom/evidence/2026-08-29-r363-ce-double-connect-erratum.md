# R363 — 勘误：R362「嵌套 insert 双 connect finding」为测试装配伪影（非 registry 簿记缺陷）

**日期**: 2026-08-29
**切片**: 勘误轮（R362 观测结论更正 + tests_ce.rs 期望修正；零生产代码改动）
**改动面**: `tests_ce.rs`（lifecycle 测试脚本两段 join 装配去除 + 期望 7→6 条目 + 注记勘误）

## 1. 勘误内容

R362 记档的「嵌套 insert（`ce.appendChild(inner)`）触发父子双 connect——is_custom_connected
检查疑未覆盖嵌套 append 路径」**不成立**。逐段读数复刻（body.append(ce) → body.append(div)
→ ce.append(inner) → removeChild → ce2 移动形态，最小序列 + 全序列双跑确定性验证）实证：

- `body.append(ce)` → `C:my-el`（恰一次）
- `ce.append(inner)` → `C:my-inner`（恰一次，无父元素重派发）
- `removeChild` / `移动形态` → 各元素恰一次 disconnect / 再 connect

全序列 6 条目：`C:my-el|C:my-inner|D:my-el|D:my-inner|C:my-el|D:my-el`——**mark/unmark
簿记 spec-correct，每连接态真转恰一次派发**。

## 2. 伪影机制

lifecycle 测试脚本用**两段 join 拼接**装配期望：`afterConnect = calls.join(',')`（截取
首段）在前，`afterDisconnect = calls.join('|')`（全量）在后，最终表达式
`afterConnect + '|' + afterDisconnect` 把首条 `connect:my-el` **双渲染**——6 条目数组
呈现为 7 段字符串，第 2 段的「多余 connect:my-el」即伪影。R362 首轮正是从这个 7 段
读数（当时恰好跑出含 ce2/det 的 7 条目形态）反推出「双 connect」的错误结论。

## 3. 更正

- `tests_ce.rs` lifecycle 测试：去除两段 join，全量单次 join 断言 6 条目；
- R362 evidence 的「观测 finding」段落以本文件为准（finding 撤销）；
- master.md 的 R362 记录注记勘误指向；
- **CE registry 专项材料中的「嵌套双 connect」项删除**（专项剩余内容 = per-realm
  registry 路由，realm 族 5 的前置）。

## 4. 教训

**分段 join 装配是观测伪影的经典来源**：首段 join(',') + 全量 join('|') 的拼接让首条
元素双渲染——读数与数组内容不符时，先做「最小序列逐段读数」排除装配层，再怀疑被观测
系统（本轮第一次误判方向正确但中途又自我推翻，多绕一轮）。与 R362 的「验证面必须与
修复面同域」同属探针纪律。

## 5. 验证

- engine v8 2490（tests_ce 3 测试全绿，期望修正）/ quickjs 1472；
- clippy 双矩阵零警告 / fmt 无 diff；零生产代码改动。
