# R5331 跨域回归对照 — dom + html-syntax 通道（2026-10-30）

**背景**：M2 指针（[m1-corpus-baseline.md](2026-10-30-m1-corpus-baseline.md)）要求首轮
引擎修复跨域核对 dom-expansion 34 号与 html-syntax 通道基线不回归（run-rules §9）。
修复 = 3d173dccc（shim 查询回落 in-doc 门 `_zwPendingNodeInDoc` 放行）。本对照在
f488fc8cf 同树上以 `git revert --no-commit 3d173dccc` 隔离单变量，pre/post 双跑对照；
跑后 `git revert --abort` 复原。

## 通道与口径

- **testharness-dom**（368 案，含 dom/ranges = 34 号语料面 + js-dom M4 五子域）。
- **testharness-html-syntax**（html/syntax 137 + html/dom 89 = 226 案，@ pin
  315976933——本轮经代理全量补拉零 403；此前本地无该语料）。

## 结果

| 通道 | post（HEAD） | pre（revert R5331） | 差值 |
|---|---|---|---|
| testharness-dom | 56,049P / 576F / 15T | 56,057P / 571F / 14T | −8P +5F +1T |
| testharness-html-syntax | 61,341P / 62F / 7T / 1U | **逐位一致** | 0 |

**html-syntax 腿：pre/post 逐位一致 → R5331 零影响。**

**dom 腿：逐案 diff 仅 6 案，全为 timing/发射形态，无查询身份语义翻转**：

| 案 | 差异 | 判定 |
|---|---|---|
| Event-timestamp-high-resolution（.html + .https） | 全通道 post 一次 0P/5F | **负载 timing 噪声**——单案隔离重跑两侧全 5P/0F（post ×4、pre ×1 逐位一致）；uievents 收口单案回退对照同款判据 |
| ParentNode-replaceChildren | post +1 案级 Timeout | 全通道负载下页级超时，单案形态无差异 |
| Range-mutations-dataChange / deleteData / replaceData | −5P / +5P / −3P | 内部超时边界的子测试发射截断 jitter——双向对称（post 侧 deleteData 反而多发射 5P），非单向变慢 |

**结论：R5331 dom / html-syntax 双腿零回归**，与 dnd（+1P）、uievents（5 处翻转判
噪声，3d173dccc 提交说明）合并，跨域核对收口。

## html-syntax 现行账（pin 315976933）

vs 2026-10-20 收口账（61,347P / 61F / 13T / 1U = 61,422，旧 pin）：本轮
**61,341P / 62F / 7T / 1U = 61,411**——分母 −11、html5lib 3T→3F、render-blocking
58F+10T→55F+7T。差异源自 pin 滚动与 10-20 以来他流引擎落版（1cf6843ba
innerHTML 身份双表示、bf9f3c41d link stylesheet 事件面、29923b68f stylesheet
MIME 矩阵等），与 R5331 无关（同 pin pre/post 逐位一致实证）。**61,341/61,411
为当前 pin 参照账**，`docs/compat/trends/wpt-suites.csv` 已记行；归档 master 的
10-20 账保留为历史事实不改写。

## 附记

- `waits_for_active_step_timeout_before_accepting_completion` 单测在 02:30 时点
  （0e3cc0c46 + 并行流 staged 树）确定性红（results[0].name 返文件级名），当前
  HEAD 复验已绿——中间 fd013ee0d（css M2 收口）触达 testharness.rs，瞬态不挂账。
- 对照产物（4 份全量日志 + 3 次单案重跑）在 /tmp，不入库；复现：revert 3d173dccc
  后跑 `make testharness-dom` / `make testharness-html-syntax` 对照即可。
