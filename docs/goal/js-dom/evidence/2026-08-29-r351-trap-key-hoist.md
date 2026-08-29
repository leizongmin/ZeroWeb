# R351 — proxy get trap 内部键读的 R98 分支成本与顶部短路 + 条目级根缓存

**日期**: 2026-08-29
**前置**: R350（Range-mutations data 族 Timeout 根因第一层修复——adjust 扫描的键域快道 + 键读提出循环外 + 键命中后根验证）。R350 后 data 族五文件仍文件级 Timeout（每文件差最后 6-30 test）。

## 1. 归因续挖（探针链）

| 探针 | 结果 | 结论 |
|---|---|---|
| R351 基线分段 | rm 5.4→40.3ms/iter、build 12.7→76.0ms/iter（100 轮） | 增长确在 rm/build 段 |
| W3 单次 removeChild（100 条跨树条目） | 2.9ms ≈ 每容器 15µs | 与 root walk（3 跳 × 5µs）吻合——但只是表层 |
| W5 缓存字段检查 | `_rzk0/_rzr0` 正确回填 | 根缓存生效 |
| W4 缓存后复测 | rm 36.8、build 70.3（仅 -9%） | **成本不在根 walk** |
| W6 adjust ON/OFF（注册表 ~220 条） | OFF：rm 1.8、build 5.3ms（**35x 差**） | 成本确在三个 adjust 函数内 |
| W8 三分解 | sc 读 20ns、identity 比较 21ns、**+nodeType 读 78,212ns** | 唯一贵点 = **proxy trap 属性读本身** |
| W9 顶部短路后 nodeType | 仍 83µs | nodeType 路径仍过 R98（未提升面）；键读已快 |
| W10 全循环复测 | **rm -72%、build -84%** | 修复净效果确认 |

## 2. 根因

part03 `_makeProxy` 的 get trap 中，R98 分支（CE 用户类首层 accessor 优先）对**每个字符串属性读**都先执行：

```js
var _r98Proto = Object.getPrototypeOf(_makeProxy(sel, handle));  // getPrototypeOf trap
// + customElements.getName(_r98Ctor)（CE 时）
```

而 `__zwHandle`/`__zwSelector` 的返回分支在 **10360 行**（R98 之后）。R350 修复后 adjust 扫描的每条目键读虽然已「提出循环外/快道化」，但每次键读仍要先付 R98 的 getPrototypeOf + CE 检查 ≈ **78µs**（W8 实测）。100+ 条注册表 × 8 op/轮 × 2 容器 × 1-2 键 → 每轮数十 ms，随注册表线性增长。

## 3. 修复（part03.js）

1. **`__zwHandle`/`__zwSelector` 顶部短路**：提到 get trap 最顶（R95 constructor 短路之前）。两键是 shim 私有锚定约定（消费者全在 shim 内部：adjust 扫描、identity 桥、testdriver stub），与 CE accessor / 反射属性语义零交集——顶部直返零行为变化。原 10360 行分支删除（同值，仅位置前移）。
2. **条目级根缓存 `_zwEntryRootOf(rg, cont, slot)`**：根比较结果挂 range 条目（`_rzk0/_rzr0` = startContainer 及其根，`_rzk1/_rzr1` = endContainer 侧），读取侧 identity 校验防 stale（容器被 R262/R191 重写后自动重算回填）；R262 重写 container 时写侧直更。root walk 从每次比较降为每 (条目, 容器) 一次。

## 4. A/B

**全量 dom sweep**（3300s guard，与 R350 轮同环境）：

| 指标 | R350 | R351 | Δ |
|---|---|---|---|
| Pass（非探针口径） | 54,057 | 55,390 | **+1,333** |
| 真实 Fail 文件 | 20 | 17 | **-3（全为 data 族转绿）** |
| 仅一侧有的 Pass subtest | — | 仅 baseline 有 = **空集** | 零 subtest 丢失 |

**三文件历史性全绿**（自 R42 导入 dom/ranges 以来从未 Pass）：
- `Range-mutations-appendData.html` 384P
- `Range-mutations-deleteData.html` 564P
- `Range-mutations-insertData.html` 382P

`dataChange`/`replaceData` 仍文件级 Timeout，但 declared tests 273→426、316→701（**2.2x**）——doTests 循环在 90s 内推进距离翻倍。

微基准：探测循环（100 轮重建+range）last-20 均值 rm 96.6→27.2ms/iter（-72%）、build 187.3→30.4ms/iter（-84%）。

既有套件零回归：extractContents 192P、insertNode 1841P、MO-attributes 42P、Node-textContent 81P、adopt-test 4P、deleteContents 129P、surroundContents 1840P——全部与基线一致。

## 5. 残余与下一步

- dataChange/replaceData 收口：残余 = R98 分支对其他 trap 读（`__zwIsText`/`nodeType`/`parentNode` host 往返 ~5µs/跳）+ 注册表 GC 压力。可选：顶部短路扩展到 `__zwIsText`/`__zwChildIndex`（需先验证 part05 wrapper 产物的消费路径）；或随 M1 L2 落地自然消解。
- R98 分支本身可优化（CE 检查结果缓存 per-proxy），但影响面大（所有属性读），须独立评估——记 L2/深结构邻域。
