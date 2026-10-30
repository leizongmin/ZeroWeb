# editing-compat — 簇 A 修复落地：R263b/R262b range 注册表预筛门（2026-10-30）

承接 [簇 A 归因收口](2026-10-30-cluster-a-clone-perf-attribution.md)（§5.4 修复 scope 立项）。
本轮完成归因收口的最后一环（trap 内部插桩）+ 修复落地 + A/B 验证。

## 1. 方法：临时 shim tape 插桩（round-5/6）

原探针（裸回调拦截）无法进入闭包内部（`_mo_notify`/CE/details 均 IIFE 作用域，
非 call-time 全局查找）。改用**临时 shim 源码插桩**：part01/part04 加 `__zwTk(i)`
顺序 delta 计账 helper + 13 个检查点（innerHTML setter 头 s0、appendChild trap
s1-s8、`_mo_notify` s21-s24），复跑后细分 s7 窗口再加 s31-s35 五点。计时经
`performance.now()`（part01b 页面域实现），聚合 `globalThis.__zwTapeState.sum`，
结果放 test name。**测完 `git checkout` 整体回退，探针页已删**（残留会污染
全量基线扫描；重建方式即本节描述）。检查点插入用 `typeof __zwTk === 'function'`
守卫，旧二进制下零影响。

## 2. Villain 定位（修复前 tape 账，2000 span/text 定标）

| 相（检查点间） | TEXT | SPAN | IH | CLONE | 判定 |
|---|---:|---:|---:|---:|---|
| s35 = **R263 `__zwAdjustRangesForInsert` 循环** | 295.3ms | 807.8ms | 1403.9ms | 3571.8ms | **主项** |
| 占 phase wall | 63% | 73% | 73% | **86%** | |
| s24 = `_mo_notify` s111 script 扫描臂 | 48.3 | 77.8 | 93.6 | 159.0 | 次项 |
| s32 = CE loop（`_ceApplyConn`+iframe） | 25.2 | 40.0 | 56.1 | 96.2 | 次项 |
| s33 = R4100 details 扫描（span 属性读） | 5.4 | 26.1 | 4.6 | 7.2 | 免罪 |
| s31/s34/s1-s8/s21-23 各段 | 均 ≤50ms | | | | 免罪 |

细节分（s31-s35 五点切分）把 s7 窗口唯一重项钉死在 R263 循环；s31（cePconn）、
s34（slotchange）均 ~1ms/2000 免罪。

## 3. 根因链

1. part06 R183 Range 工厂把每个 range push 进 `globalThis.__zwLiveRanges`
   （**环形缓冲 8192，基本只进不出**，9785-9787）。
2. `_zwAdjustRangesForInsert`（part03 R263）gate 只查 `regs.length` 非空——
   **非空即无条件**读 `parentNode` + **`childNodes` live-NL refresh（O(n)，
   R140 每读全量 slice+concat）** + O(n) 索引扫，然后才逐条目过滤。
3. 故只要注册表非空，**每次 appendChild 付 O(n) 兄弟数 → 填充序列整体 O(n²)**。
   探针环境 R=1（load 期残留 1 个 range 即足）；editing 真实语料 Selection/Range
   操作频繁，注册表常驻数百条（tax 更重）。
4. `_zwAdjustRangesForRemove`（R262，remove 侧）同构：gate 后无条件 childNodes
   O(n) + 索引扫。innerHTML= 重设的移除侧同病。
5. 修复前归因文档 §2 的 S4 桶微增（134→179）、S8 温和超线性、resetBody 3.7×
   全部由此 O(n²) 解释。

## 4. 修复：pass1 预筛门 + 原路径逐字保留

两处对称改造（part03.js），冷路径（pass2）逐字保留：

- **R263b**：gate 后先读 `parentNode`（1 次 proxy 读）+ 父键 hoist（同款键域
  快道）+ O(R) 扫注册表：`startContainer`/`endContainer` 过 R352 dead 快道 +
  `sameParent`（identity/键）廉判，**无命中直接 return**；命中才进原路径
  （`_zwRangeRootOf` + O(n) childNodes/index + 逐条目 verified 调整）。
  语义等价论证：R263 规则匹配面仅「boundary node 逻辑等于 newParent」（无
  子孙判定），预筛谓词 = 同款 sameParent + dead-check，是候选的保守超集；
  newIndex 仅被命中条目消费；t8c 契约循环内树不变。
- **R262b**：同构。预筛谓词取保守超集 `sameNode262 ∨ sameParent262 ∨ 同根`
  （rule① 子孙命中必与 removed 同树——`_zwEntryRootOf` 条目级根缓存 identity
  比较）+ R352 dead 快道；O(n) 快照与完整调整逐字保留在 pass2。
- R260（CharacterData 侧）已有键域快道且无 O(n) 前置，未动。

## 5. 收益（同探针 before/after，干净态无 tape）

| phase（2000 节点） | 修复前（tape 贮 ∼150ms 载） | 修复后 | 提升 |
|---|---:|---:|---:|
| SPAN appends | 1111.1ms（s35=807.8） | 298.3-308.5ms | 3.6× |
| IH 全链 set | 1912.9ms（s35=1403.9） | 465.1-480.0ms | 4.0× |
| CLONE deep | 4136.9ms（s35=3571.8） | 434.1-444.3ms | **9.1-9.3×** |
| RESET（重设） | 13217ms（前轮 S8） | 1253.2ms | 10.5× |

s35 段修复后 4.3-8.3ms（原 295-3572ms）；R263b pass1 实测 ~2.4µs/append。
CLONE 现快于 IH——归因文档「clone ≈ 2×set」病征消除。

## 6. 验证链

- `make test`（DISPLAY=:20）：**3601P/9F，与已知红集逐一对应**（fetch/Request/
  Response/iframe-service-worker 域，与本次改动无交集），零新增。
- bold 单案全矩阵（簇 A 最大消费者）：**1138P/1910F 计数零位移**（对账
  run-budget-unlock-rerun 现行账）；墙钟 3m29s（预算窗占用 ~300s → 209s）。
- dom/ranges 子集 A/B（新旧二进制各一遍，40,993 vs 41,041 行）：**零 Pass↔Fail
  翻转**；行数 ±8~22 漂移经同版二进制两次运行自对照证实为 90s 看门狗截断点
  方差（ mutations 族恒带 `Timeout 1` 长尾变体，行数非确定）；insertData 旧版
  Timeout+341P → 新版全量 382P（截断解锁向）。
- dom 全面（399 案 56,426 行）：新二进制 **900s guard 内完整出账**；旧二进制
  同面 900s 被外杀零输出——性能解锁直接证据。
- selection 单案（addRange-00）正常完成；探针五相全 Pass。

## 7. 验收对照与残余

- S7 ≈ S5 同量级 ✓（434 vs 467ms）。
- S5 ≤300ms/2000 目标：实测 465-480ms——残差 = s24（s111 臂 34-46µs/append）
  + s32（CE loop 20-40µs）+ s2（dispatch 23µs）+ 每调用 glue，**下轮候选**。
- S8 ≤1.5×S5 目标：实测 2.7×（1253ms）——残差在 **removeChild trap 自身
  glue ~390µs/次**（R262b 仅 ~10ms/2000），镜像 append 的分布式成本，
  **下轮同法 tape 归因候选**。
- bold <100s 目标：未达（209s）——受上两项 + execCommand 面 glue 限制；
  S5/S8 残差收敛后复测。
- 修复属纯性能余量项（R5342 后非解锁项），但 dom 面 900s 解锁为额外收益。

## 8. 复验入口

- 探针已删；重建按 §1（tape 检查点布局）+ 前文 §1-§4 页面形态。
- 单案通道 `make testharness-editing-run-one FILTER=<案>`；dom 面
  `make testharness-dom`（新二进制 ~14-15min，旧二进制 900s 不可完）。
- R263b/R262b 位于 part03.js `_zwAdjustRangesForInsert` / `_zwAdjustRangesForRemove`
  函数头部，pass2 原文逐字保留（review 时对照 R263b 注释块后的原文段）。
