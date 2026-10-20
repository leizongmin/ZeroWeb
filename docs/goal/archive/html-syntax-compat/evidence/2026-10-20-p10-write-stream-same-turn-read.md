# P10：document.write 流同 turn 读收敛——ambiguous-ampersand 2F→0F（R5206 续二）

**日期**: 2026-10-20
**域**: html-syntax-compat（P9 遗留挂账的 async-apply 缝 document.write 子集）
**前置**: [P9 证据](2026-10-20-p9-aria-crossdoc-gEBI.md)——host 视图层已由 latest-wins
收敛，残差钉在 shim 侧读路径多层积累器（pending 桶/children 融合/tag 视图读数分裂
gEBT4/Kids6/Children8）。

## 目标

`html/syntax/parsing/ambiguous-ampersand.html` 2F 清零（「Number of elements
document.write」+「Check div structure: document.write」），全通道零回归。

## 根因（机制图谱补全）

P9 证据已定位「多层独立积累器」。本轮补全三缝的精确机理：

1. **raw wrapper 误入 removed 记账**：R5001 写流每写步 `body.innerHTML = 基线`
   重放，innerHTML setter 的 `_mo_notify(removedNodes=旧子快照 wrapper)` 把
   **基线内容 wrapper** 打进 `_zwPendingRemoved`——追加语义下基线内容由重放原样
   复活、非真移除。gEBT 集合构建按 identity splice 剔除 → static div 从 gEBT
   消失（id 形元素两条包装路径同落 `#id` 必命中）。
2. **补偿代理与视图回放双计**：写流每步 innerHTML+IAH 各挂一批解析补偿代理
   （`_zwSelPendingParent` 槽、无 handle/sel）进 `_zwPendingAdded`；而 P9 后文档级
   枚举源 `__zw_query_all_tagged` 已走 `with_query_view_doc`（latest-wins 收敛
   base + 末次满缓冲）——代理再并入即双计（gEBT 4 vs 2）。
3. **raw 缺席容器的子视图恒空**：写流新建容器 sel（written div）在 raw 快照恒
   缺席，`__zw_child_nodes` 独走 raw 快照 → childNodes 恒空（「Number of
   elements document.write」F）。R57 起 attr/text/tag 族已走视图回放，子视图是
   唯一独走 raw 的读路径。

## 修复（R5206 续二，五点）

| # | 文件 | 内容 |
|---|------|------|
| 1 | shim part06 | 写流重放门 `__zwDocWriteReplay`（try/finally 恢复前值）包住基线重放双调用 |
| 2 | shim part04 | innerHTML setter 消费门：写流门内 removedNodes 只保留解析补偿代理（写际消零收敛依赖），raw wrapper 不记账；**门外普通 innerHTML 不受影响**（真替换语义照常记账） |
| 3 | shim part04 | 创建站点打标 `_zwSelPendingParent.inViewReplay`：**IAH 站恒真**（InsertAdjacentHtml 被视图回放应用），**innerHTML 站仅写流门内真**（基线重放 = 基座内容） |
| 4 | shim part05 | 两处文档级并入点（`_zwMakeCollection` R333 臂 + `_zwDocAllElements` pending 扫）剔除打标代理；判据 = 打标 + K3 切片 C 同款（无 handle/sel、挂槽、非 noWire、parentSel） |
| 5 | bridge | `__zw_child_nodes` raw 缺席 sel 回落 `with_query_view_doc`（新增 `element_exists_doc`，与子列表读共用同一 `with_query_doc` 缓存文档，增量为一次 selector 游走） |

### 打标制的设计要点（本轮关键修正）

初版直接按 K3 切片 C 判据无差别过滤 sel 父补偿代理，探针证伪两处：

- **普通 innerHTML 误伤**（gEBT len 1→0）：innerHTML 只入队 `SetInnerHtml`——视图
  回放**不应用**（R3029 既有语义），补偿代理是其同 turn 文档级枚举**唯一可见面**。
  stash 对照实证：pre-WIP `len:1|0:ih1` → 初版过滤 `len:0`。
- **plain IAH 双计是既有 bug**（gEBT len 3 vs 2）：IAH 内容经队列回放进视图，代理
  再并入即 dup（stash 对照 pre-WIP `len:3|0:static|1:ia1|2:ia1`）。初版过滤顺带
  修复，但判据须精确到「内容在视图」——由此落定**创建站点打标**（表 #3）：
  视图回放是否包含代理内容在创建时刻即已知（IAH 队列必回放 / SIH 必不回放 /
  写流门内 innerHTML 的内容 = 基线已在基座），part05 只认标记，不猜。

组合序列（写流后续 plain innerHTML）的 gEBT 含 stale base 条目属 R3029 既有
限制（「基线 ≠ 基座内容的一般 SIH 形态不在本次修复面」，P9 已记），不在本轮
修复面；独立场景探针钉面见常驻测试。

## 常驻测试

`js_dom_bridge_tests/part24.rs` 两案：

- `r5206_write_stream_doc_query_no_dup_and_absent_child_view`：写流场景四断言
  （gEBT = [static, qw] 无 dup 无消失 + written div childNodes 回落视图）。
- `r5206_plain_path_compensation_stamp_semantics`：打标判据两侧语义（独立沙箱）——
  plain innerHTML 代理保留（插入内容同 turn gEBT 可见）+ plain IAH 双计修复
  （len 3→2，集合语义）+ handle 并入不受影响。

## 门禁

- `cargo test -p zero-engine --lib`：3301 passed / 0 failed（经 test-guard，
  `--compile-first --per-proc-mem 6 --total-mem 12`）。
- `cargo fmt -p zero-engine -- --check` 干净；`cargo clippy -p zero-engine
  --all-targets` 无 warning；shim 拼接 `node --check` 通过。
- 全通道复测：见下节。

## 通道验证

`make testharness-html-syntax` 全通道（test-guard 包裹）：

- ambiguous-ampersand **3/3 全 Pass**（2F→0F；过滤前后两版均验）。
- 全通道 **61347P/61F/13T/1U = 61422**（vs P9 基线
  61345P/63F/13T/1U：**+2P −2F**，恰为本案修复面）。Fail 集构成逐域核对：
  58 render-blocking（既有挂账）+ 2 unclosed-svg-script（html5ever tokenizer
  能力缝挂账）+ 1 document.all（挂账）= 61；Timeout 13（html5lib ×3 +
  render-blocking ×10）；Unsupported 1。**零漂移**。

## 余账

- R3029 既有限制（SIH 不被视图回放 → SIH 后视图含 stale base）维持 P9 记录，
  组合序列形态无案主张。
- bench：`__zw_child_nodes` 每次调用增加一次 selector 游走（存在性判定，与子
  列表读同一缓存文档）；dom_bindings_bench 不覆盖 shim 遍历回调路径（P9 同款
  记账）。写流门/打标为 O(1) 属性读写，无热路径新增游走。
