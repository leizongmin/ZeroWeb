# R350 — Range-mutations data 族超时根因：live-range 注册表扫描的每条目 host 往返

**日期**: 2026-08-29
**影响**: `dom/ranges/Range-mutations-{dataChange,appendData,deleteData,insertData,replaceData}.html` 五用例自导入起文件级 Timeout（90s 截断，declared 212-344 test 仅完成 88 个）

## 1. 背景与上轮误判勘误

R349 在 `testharness.rs` script_timeout 注释中留下的中断前认知（未经验证的探针假设）为：
「per-op 线性成本 ~0.6ms/op × ~500k ops，放宽超时无解，维持 90s 截断待 L2 闭合」。
本轮探针实测推翻：

- **非 per-op 固定成本**：单次 `p.firstChild.data = "x"` 写仅 0.004ms；单次 setStart/setEnd + offset 读 0.019ms（300 次均值）。
- **增长源 = shim `__zwLiveRanges` 注册表**（part06 `_makeRange` push，8192 环形缓冲）随用例序列线性累积后，R260（data 变更）/R262（节点移除）/R263（节点插入）三处 adjust 函数对**全部历史条目**逐条做身份/键比对，且比对中的属性读（`__zwHandle`/`__zwSelector`/`parentNode`）走 **proxy get trap → host 回调往返**。

## 2. 探针链（全部经 test-guard 单文件复现）

| 探针 | 形态 | 结果 |
|---|---|---|
| COST1/COST2 | 300 次纯写 / 纯 range 操作 | 0.004 / 0.019 ms/op——单 op 便宜 |
| A（无 range） | 重建树循环 150 轮 | cost(i)=2.4+0.026·i，近恒定 |
| B2（有 range 无写） | 同 A + 每轮 createRange+setStart/setEnd | cost(i)=2.4+0.790·i，**斜率 30x** |
| Q 分段 | 重建+range 分段计时 | rebuild 段 6.7→96.9ms/iter（range 存在使其涨 14x），range 段恒 1ms |
| S（每轮清空注册表） | 同 Q + `regs.length=0` | 3ms/iter 恒定 ✓ 决定性 |
| C | 注册表长度 | 60 轮 61 条——无 trim（8192 才截断） |
| R | removeChild 单独计时 | 9.8→32ms（涨） |
| F/L/M | createRange vs setStart 分离 | 各自 O(1)；成本在「注册表 × adjust 扫描」 |
| W6/W7 | adjust ON/OFF A/B | ON 18.8ms/append，OFF 0.47ms；只关 insert → 0.08ms（insert adjust 是主体） |
| W13 | native vs 等价轻量 stub | 11.46ms vs 0.10ms（**115x**） |
| W14 | 逐段加回：root-walk-only 0.39 / +键读 0.60 / 全循环 0.60 | 单独都不慢 |
| W17/W20 | proxy trap 键读微基准 | 0.49µs/读（活节点热路径）、跨树旧节点同量级 |
| W21 | proxy parentNode 上行 | 3 跳 root walk 13.8µs（每跳 host 往返 ~5µs）|
| W18 | 等价实现 + newParent 键读提出循环外 + 无键快道 | **0.21ms vs native 11.93ms（57x）** ✓ 修复形态确定 |

## 3. 根因三层

1. **每条目键读走 proxy trap**：`sameNode/sameParent` 闭包内每条目 2 容器 × 1-2 键读，每次读穿 part04 get trap（数百行分支）→ ~0.5µs；`sameParent263` 还在**循环内每条目重读 newParent263 的键**（恒定值重复 trap）。
2. **前置 root walk 穿 proxy**：跨树守卫若前置，每条目 2 容器 × O(深度) 次 `parentNode` 读，每跳 host 往返 ~5µs（W21）——比键读贵一个量级。
3. **文本域键匹配分支**：`sameNode` 的 textEl 分支命中 `cont.parentNode`（host 往返）在 p260sel 存在时每条目触发。

三者相乘：每用例 O(注册表条目 × 每轮 ops × 每条目 host 往返 µs)——300 轮循环后期 97ms/iter（探针 Q），五用例 90s 只跑 28%（88/304）。

## 4. 修复（part03.js 三处 adjust）

- **键域快道**：mutation 节点（R260 node260 / R262 removed262）或插入父（R263 newParent263）无 handle/sel 键时（plain 工厂域），跨域键比对必不命中 → identity-only O(1)/条目，键读全部跳过。
- **键读提出循环外**：R263 newParent263 / R262 oldParent262 的 handle/sel 键从每条目重读改为函数级一次。
- **键命中后根验证**：前置 root walk 改为 `sameNodeVerified`/`sameParentVerified`——identity/键比对 miss 短路（常态 O(1)），仅键字符串相等（罕见）时才做 root walk 验证跨树。**语义不变 + 顺带修真 bug**：跨树同 selector 字符串的旧死条目此前可被键匹配误命中并篡改 offset（守卫后不可能）。
- 守卫保守性：同树（含 detached 子树与 mutation 节点同根，如 detachedPara1 域）不跳过，行为与旧路径一致。

## 5. A/B 结果

**微基准**（60 条目 × 30 append）：native 11.93ms → 修复后 0.21ms/append（57x）；探测循环 97ms/iter → 3ms/iter（32x，清空注册表形态等价对照）。

**全量 dom sweep**（`testharness-dom`，3300s guard）：

- **Fail 集合恒等**：真实 Fail 20 = 20（排除两侧探针文件），零新增零丢失。
- Pass 集合净 +8（仅 R350 有 10 / 仅 baseline 有 2）——全为已知并发 Timeout 轮转族（Event-dispatch-on-disabled-elements 等本轮跑完产生 Pass）。
- data 族完成 subtest：dataChange 88→273（3.1x）、appendData →198、deleteData →225、insertData →217（全量完成）、replaceData →316。五文件仍文件级 Timeout（完成率 90-100% 差最后 6-30 test）——残余为注册表条目 GC 压力 + textEl 分支的 `cont.parentNode` host 往返（每条目 ~5µs × 20-60 条 × 7 写/轮），随 M1 L2 live Document 落地（proxy/host 往返消失）自然消除。
- 五文件合计完成 subtest 从 ~440 提升到 ~1229（+789 test 在 90s 窗口内实际执行）。

## 6. 残余与下一步

- 五文件最后 6-30 test 的收口 = 注册表**条目级键缓存**（建 range 时缓存容器键，避免扫描期 trap 读）或 L2 落地后自然消解——记 master.md 下一步。
- 8192 环形缓冲收紧无效果（W26/W27：真实瓶颈在扫描内 host 往返而非条目数本身）。
