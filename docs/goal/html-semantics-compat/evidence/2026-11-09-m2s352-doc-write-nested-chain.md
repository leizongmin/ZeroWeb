# M2 切片 352：写流嵌套链插入点序——execution-timing 041 F→P

**日期**: 2026-11-09
**目标对**: `scripting-1/the-script-element/execution-timing/041.html`（s351 基线 15F 中单 F，document.write 嵌套链序）
**结果**: 041 F→P；域 join det F→P=1 / P→F=∅ / 新增 P=∅ / removed-F=∅ / T=8 同集双向 ∅ / 双态键 0；语料口径 **15F→14F，1107P→1108P**（distinct-pair 口径；149.html `for='  WINdow\t` 变体在 A/B 两日志同形重复 12 行，`(key,state)` sort -u 剔除后 headline 不受影响）

## 上游语义（勘根）

[HTML §4.3 document.write](https://html.spec.whatwg.org/multipage/dom.html#document.write)：write 插入位置是**当前执行中 script 的 parser insertion point**，不是缓冲区尾部——嵌套 write 的 src script 在此前已写入但未执行的 src script **之前**执行。041 场景：外层 write 三段（inline 探针 + include-1 + include-2 src 链），include-2 的 eval 窗口内再 write include-3 src——期望序 = include-3 先于 include-1/include-2 之外的 pending 面之前插入执行（11 元素精确序断言）。

## 执行时机族勘根结论：非单一根因（4 案分诊）

| 案 | 旧红形态 | 根因定性 | 处置 |
|---|---|---|---|
| **041** | 11 元素序断言失配（include-3 落尾） | 引擎 doc.write 嵌套链插入点序缺失——**本切片修复** | F→P |
| 068 | `found #foo element: NO` | 同 chunk 内 script 之后的 markup 在 script 执行前一次性物化——需增量物化（replay 模型手术），独立切片 | 挂账 |
| 056 | 空 `<script></script>` + appendChild 文本未同步执行 | mutation 触发的 script prepare 面缺失（独立面） | 挂账 |
| 143 | 期望 [#1,#2,#3] 引擎日志 [#1,#2] | SVG script tokenizer 双重转义 → 外层 script 文本含内层 `<script>` 字面 + `assert_unreached():` 冒号笔误 → spec 严格解析下 SyntaxError——疑上游自身失败 | upstream 真值挂账待查 |

## 施工（engine shim 单面，part06.js 七处）

doc.write 面从「扁平追加缓冲」扩出**嵌套链驱动器**：

1. **状态复位三处**（open() 复位块 + 首写 init 块 + close() face 接管块）：`__zwDocWriteNestedQueue/NestedBusy/NestedMark/NestedDone/SlotCount/NewFrom` 六全局清零——与 s114 defer 复位同层同形。
2. **face pass 头**：`__zwDocWriteSlotCount = list.length` 快照；非 throwaway pass 先 `__zwDocWriteNestedDrive()`，busy 即 return（嵌套链独占执行窗口）。
3. **src 分支重构**：defer/async 检测上提到 throwaway 分支前（s114 redirect 分支复用提升变量）；新 throwaway 臂——defer 非穿跳过；`NestedMark/NestedDone` 已记跳过；**仅 `slot >= NewFrom`（本 write 新物化）入队**并即时 `NestedDrive()`，旧 slot 留主链。
4. **主链**：`NestedDone` skip 标记插在 pendingSlot 重入门之前；copy 构建重构为 `__zwDocWriteBuildCopy` 助手（script 副本全属性透传 + xlink:href→src 回写 + catch 落 `_el` 本体）。
5. **`__zwDocWriteNestedDrive` 驱动器**：busy → 队空 → `pendingSlot >= 0` 让位（eval 窗口内 pendingSlot 即正在求值的 script）→ prepare 可用，四道门；出队即 `_zwPrepareSrcScript(copy, consume)`，consume 标记 done + 清 pendingSlot + 解 busy + 递归驱动 + 空闲时补 `__zwDocWriteRunFace()`。
6. **write() replay**：replay 前 `__zwDocWriteNewFrom = __zwDocWriteSlotCount` 快照——throwaway pass 只排本 write 新物化的 src slot。

**两条设计不变式**（实现前推演钉死）：

- **嵌套链复用单一 `__zwDocWritePendingSlot`**：part03 `_zwExecFetchedImpl` 以 pendingSlot 推导 `__zwDocWriteEvalSlot`（fetch 求值前后夹取）——独立嵌套 pending 会断 eval 窗口推导。
- **consume 不推进 watermark**：`max(wm, slot+1)` 跳法会略过未认领主链 slot（trace：consume(slot3) 时 wm=2 → 跳 4 → include-1(slot2) 永不执行）——主链步行靠 NestedDone skip 标记消费已认领 slot。

## Blast radius 普查

- 行为变化只在 **eval 窗口内嵌套 write 外链** 形触发；全语料 `document.write(<script src=` 形仅 13 文件（execution-timing 11 案 + history/008 + resource-timing/initiator-type）。
- 资源外溢双点逐验：resource-timing solo exit 0；history/008 Timeout **双树同态**（跨域 script 不可达既有面）。
- 既有红 A/B 定谳（stash 前后双跑）：output.html Fail 双树同态（0 处 doc.write）；engine `--lib` 3816P + 7 env 红双树逐名同构（img_runtime_deferred_settle_r5275、svg_img_settle_density_r5277、fetch_filtered_response_type_matrix、iframe_sw_messages_dispatch、response_body_readable_stream_r2967、websocket_error_nonterminal、websocket_event_flow）。
- shim 拼装语法检查：21 part 按构建序拼接（js_dom_bridge.rs 装配序）`node --check` SYNTAX-OK。

## 验证

- **solo 041 全 Pass**（`FILTER=execution-timing/041`，exit 0；evidence/logs/s352-solo-041.log）。红态证据：s351-b 域跑 041 Fail 行（序断言失配）。终设计全 11 元素执行 trace 与期望序逐位吻合。
- **域 join**（LC_ALL=C + `(key,state)` sort -u + 双态键核查）：A=s351-b **1107P/15F/8T** vs B=s352-b **1108P/14F/8T**——det F→P=1 恰 041；new-F=∅；removed-F=∅；P→F=∅；T=8 同集双向 ∅；双态键 0。
- **clippy** zero-engine `--all-targets -D warnings` 0 告警；`cargo fmt --all -- --check` 净（本切片纯 JS shim，无 Rust 改动，双证兜底）。
- 域日志 evidence/logs/s352-b-script-element.log（B 面）；基线 A 面 s351-b-script-element.log（s351 既有）。

## 残余 14F（B 基线在册）

execution-timing 056/063/068/143/probe12-defer-slow（自建 probe 单列口径）、fetch-src/alpha/base、json-module/parse-error-location.tentative、module/custom-element-exception、module/dynamic-import/{basic-deferred-evaluation,cyclic-tla-defer-eval,tla-deferred-evaluation}.tentative（import.defer JS 语言特性面独立切片）、remove-src-attr-prepare-a-script、script-not-found-not-executed、module/probe13-async-srcfast（自建 probe 单列口径）。T=8 同前长尾在册。

## 挂账 / 下一步

- execution-timing 068 同 chunk 增量物化（script 截断后续 markup 物化，replay 模型手术）——下一候选勘根。
- 143 upstream 真值：wpt.fyi summary_v2 与 per-test results 端点 2026-11-09 四次取回空对象/超时（网络面不通）——真值未取得，不臆断，挂账待网络面可用复核。
- 056 mutation 触发 script prepare 面；import.defer 三案独立切片；fetch-src/alpha/base、json-module parse-error-location、custom-element-exception、remove-src-attr、script-not-found 逐案勘根。
- probe12/probe13 自建 probe 单列口径不入 headline F。

## 日志

- evidence/logs/s352-solo-041.log（solo 041 Pass）
- evidence/logs/s352-b-script-element.log（域跑 B 面）
- s351 基线 A 面：evidence/logs/s351-b-script-element.log（既有）
