# 簇 A removeChild 残余 O(n²)：融合视图 overlay 单遍化——探针 w2 −60%，十门零位移

**日期**: 2026-11-01
**前置**: [R5340-glue 收刀](2026-10-31-cluster-a-removechild-fused-rebuild.md) §6 挂账——每
removal 余 2 次 `_childNodeList` 全量物化（R126 子判定 + LiveNLSync），总量 O(n²)。
**结果**: tape v3 内部插桩钉死剩余墙钟 ~96% 在 `_zwOverlayPendingChildNodes` 两循环内
（added 合并 ~70% + removed 逆序扫 ~18%），基底缓存恒命中（hit=4001/miss=0，rebuild≈0）
——host 往返无关；overlay 三笔单遍化（removedSet 过滤 / `_resSeen` Set 判定 / 尾插 push）
每调用 O(added×res)→O(added+res)，探针 w2 均值 3,832→1,506ms（同带 tape A/B −61%）、
净树 1,564ms（对 R5340-glue 净树 4,076ms −62%，对两轮前 6,661ms 累计 −77%）。七语料 +
双 lib + bench-gate + web-components 十门零位移。

## 1. 方法（tape v3）

沿前两轮 tape 方法（shim 顺序段带 + 闭包累计 + `__zwTkReset/__zwTkDump` 两读出），本轮
插桩面收窄到 `_childNodeList`（cache hit/miss、rebuild 段）与 `_zwOverlayPendingChildNodes`
（总时长 + removed 逆序扫段 + added 合并段；第 4 参传调用起点时刻以拆
rebuild = 总 − overlay）。探针复用 `zz-probe-rm-glue.html`（w2 = sel 域 2000 span
removeChild×2000），3 样本 ×（修复前/修复后）同二进制类；测完 tape 整体剥离（标记
TAPE-v3 全量清除，`grep -c` 归零复核），净树重建复跑核数。

## 2. 归因（tape v3，3 样本）

| 指标（w2 窗累计） | 3 样本 | 占比 |
|---|---|---|
| wall | 3,843/3,740/3,913（3,832） | — |
| `_childNodeList` 调用数 | 4,001（= 2/removal + 1 初始读） | — |
| cache hit / miss / rebuildMs | **4,001 / 0 / 0-2** | rebuild ≈ 0 |
| overlay 总（ovMs） | 3,614-3,764 | **~96%** |
| added 合并段（addMs） | 2,548-2,704（addN=4.0M 条） | ~70% |
| removed 逆序扫段（rmMs） | 639-729（rmN=1.0M 条） | ~18% |

四点结论：

1. **host 往返无关**：`_zwChildBaseCache` 恒命中——innerHTML= setter 置空基底
  （part04 R304 同款 `set(sel, [])`）后 overlay 供全视图，本窗零 JSON.parse 零
  wrapper 重建。R5340-glue 轮「每 mutation 失效基底」的表述对本形态不适用（该失效面
  在 apply 换代与特定 setter 臂，不在 removeChild 循环内）。
2. **added 合并主项**：逐条 `seen` 检查与 `nextSibling` 定位各 O(res) 线性扫，bulk 视图
  （第 k 次 remove 时 added≈n−k × res≈n−k）单次调用 O(added×res)。
3. **removed 逆序扫次项**：`bucket.removed` 随 removal 累积（第 k 次已 k 条），陈旧
  条目（早已不在 res）每次调用仍全扫 O(res)。
4. **splice memmove 与 Proxy trap 读非主项**：中段微修（尾插 push + trap 读局部复用）
  单独 A/B 实测——push 零增量（1,526→1,529 同带）、trap 读复用零增量（910-987 vs
  948-967 噪声带）；后者已回退（未兑现的机制不写收益注记），前者保留（等价零险）。

## 3. 修复（三笔，part05，+40/−9）

全部在 `_zwOverlayPendingChildNodes` 单调用内，**比较语义逐位保留**（每次调用仍全量
重算，per-call-recompute 架构不变）：

1. **removed 剔除单遍化**：`bucket.removedSet` 在时单遍 `has` 原地压缩（读指针写回，
   免第二份分配）。等价论证：res 条目 identity 唯一（基底 wrapper 唯一 + 并入
   seen/R182 去重），旧逆序首见剔除恰 = 剔除属于 removed 集的条目；`removedSet` 与
   `removed` 数组由记账侧恒成对维护（`_zwHCLiveInvalidate` push/add、splice/delete、
   512 压实三处；桶构造器恒建，R371 只重键不新建）。Set 缺失（防御口径，现无此形态）
   回落旧循环。
2. **`_resSeen` Set**：进 added 循环前建一次 res identity Set——`seen` 判定 O(1)；
   `nextSibling` 先 O(1) 成员判定、命中才做原定位扫（miss = 旧扫空落尾插
   `res.length`，等价且省全扫）。插入即 add，Set 与 res 成员恒一致（R182 重定位摘旧位
   后立即回插，其间无 has 消费）。added 空不建 Set（纯基底读快路径零新增成本）。
3. **尾插 push**：`pos >= res.length` 时 `push` 替代 `splice(pos,0,nd)`（后者在
   pos===res.length 时与 push 逐位等价），bulk 合并常态免 O(res) memmove。

## 4. A/B（tape 同在 3+3 样本，净树 3 样本）

| 指标 | 修复前（tape） | 修复后（tape） | 净树（tape 剥离） |
|---|---|---|---|
| w2 wall | 3,843/3,740/3,913（3,832） | 1,492/1,554/1,531（1,506） | 1,497/1,567/1,629（**1,564**） |
| ovMs | 3,614-3,764 | 1,373-1,416 | —（未插桩） |
| 其余 7 窗 | 73-100 / 887-1,057 / 991-1,025 | 73-94 / 1,006-1,057 / 1,342-1,356 | 73-77 / 92-93 / 1,069 / 1,042 同带 |

净树 1,564ms 对照系：R5340-glue 轮净树 4,076ms（−62%）；两轮前原始 6,661ms（累计 −77%）。
残余 ~1.4s = 2 次/removal 的 O(n) 重算常数（Set 构建 + 过滤 + 逐条 ~240ns 循环体）+
R5340 臂簿记/notify/LiveNLSync 同步拷贝。

## 5. 门禁（净树 8c617647d + 本修复，systemd 单串行批 `zw-rm-gate`）

| 面 | 结果 | 对现行账 |
|---|---|---|
| join-different-white-space-style（TIME_LIMIT=2400） | **7,518P/0F/0T** | 零位移 |
| insertparagraph-in-inline-editing-host | **880P/0F** | 零位移 |
| insertparagraph-with-white-space-style | **1,104P/0F** | 零位移 |
| insertlinebreak-with-white-space-style | **552P/0F** | 零位移 |
| insertparagraph-in-editing-host-cannot-have-div | **420P/0F** | 零位移 |
| insert-paragraph-in-void-element | **72P/0F** | 零位移 |
| insert-text-in-void-element | **48P/0F** | 零位移 |
| engine lib（make test 主矩阵，fail-fast） | **3,641P/1F** | 1F = s24 已知红唯一（红名 identical），本轮零漂移 |
| webview lib（test-guard 包裹 `-p zero-webview --lib`） | **832P/0F** | 零位移 |
| bench-gate（`ZERO_WEB_BENCH_CRATES=zero-engine` 定向） | **PASS**（页面绝对预算过；硬件基线 WARN 既有账） | PASS |
| testharness-web-components | **4,022P/1,146F/5T** | CE 快路轮三跑同计数——计数级恒等（失败集未逐位 diff，基线清单不在本轮日志；overlay 为全消费面共享机制，计数+分母+T 全同为零位移判据） |

执行形态：全批单 systemd user service 串行（探针×3 → 七语料 → make test → webview
lib → bench-gate → web-components），规避并发 make 重编译 `target/test-guard` 的自
exec ENOENT 竞态（[[test-guard-self-exec-race]]）。

## 5.1 组合态复测（推送时 rebase 并入并行流 s186-s189 后）

推送 rebase 将并行流 7 笔（s186 click 分派域 + s187 color IDL + s188 frame
navigable + s189 range value 消毒 + layout float/intrinsic 两笔 + weekly cron 基线）
并入组合树 445e89f70——其中 part01/03/04/05 同 shim 拼接二进制。合并语义核对：本提交
对 3be31af13 的 part05 diff 恰为 +40/−9（hunks 完整落位），并行流 part05 hunks 在
536/855/1126 行（click 事件面），与本轮 overlay 区（~10,868+）文本与语义均不相交。
组合树复测（systemd 单批 00:40-00:48）：join 三案 3,456+2,607+1,455 = **7,518P/0F**
+ inline-host **880P/0F** 逐位恒等，探针 w2 1,522/1,502/1,582（均值 1,535，落在门禁
净树带 1,497-1,629 内），0F/0T——**零位移**。

## 6. 残余与挂账

- **每 removal 仍 2 次物化**（R126 判定 + LiveNLSync 刷新），单次调用已 O(n)，总量
  O(n²)。下一刀 = 单物化/视图缓存面——LiveNLSync 增量化或融合视图缓存需 R55/R304
  身份稳定副作用与 `_zwHCLiveInvalidate` 失效覆盖重审（`_r140Refresh` 的 held-array
  语义靠 mutation 时点同步维持，惰性化会让页面留存数组失真；R382 惰性 ghost 确认使
  「上次视图 − 1」推导不等价）——既有「独立整轮」scoping 不变。
- 单调用 O(n) 常数 ~240ns/条（4M 条·窗 → ~0.9s）：逐条分支 + Set 操作的循环体开销，
  非可再单遍化的线性扫。
- web-components 1,146F 为计数级恒等判定（见 §5 表注），失败集逐位 diff 留待该面
  下次基线复验时一并做。
- 插入侧残余地图（[CE 快路轮 evidence §4](2026-10-31-insert-side-attribution-and-ce-entry-fast-path.md)）
  五项 scoping 不受本轮影响。

## 7. 复现入口

- 探针页：`tests/wpt-runner/wpt-data/editing/run/zz-probe-rm-glue.html`（gitignore 域
  保留；w2 = sel 域 removeChild×2000 主窗）。
- 单案通道：`make testharness-editing-run-one FILTER=zz-probe-rm-glue TIME_LIMIT=600`。
- tape 复建：本文 §1 检查点清单（`_childNodeList` 入口/缓存分支 + overlay 第 4 参
  拆段模式）。
