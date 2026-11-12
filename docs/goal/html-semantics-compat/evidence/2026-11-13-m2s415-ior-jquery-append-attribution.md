# M2 切片 415（round-1）— ior jQuery append() 新红归因：同源二进制翻转，源归因排除

日期：2026-11-13 凌晨（承 s414 下一步首位：ior 新红三跑归因）
commit：本账本随 docs commit（零代码变更轮）

## referent

`the-select-element/inserted-or-removed.html :: The last selected OPTION should win; Inserted by jQuery append()`——s414 勘根定谳的真新红（+1F）。失败签名 `assert_equals: expected "Second" but got "First"`：测试纯同步（s414 已核源码——模拟 jQuery 1.x/2.x DOM 移动：detached select innerHTML → fragment 逐个 appendChild → target.appendChild(fragment)，断言 last-wins selectedness），无 jQuery 库依赖、无异步面。该子测 s236（2026-11-02）前 1F 修复后 **11 天稳定 Pass**（s236–s413 数十轮 sweep 全 Pass），2026-11-12 23:12 s414 批起确定性 Fail。

## round-1 归因腿（/tmp/s415/，全部 test-guard 包裹 + RAF_FRAME_DRIVEN）

| 批次 | 树 | 二进制 | 条件 | 结果 |
|---|---|---|---|---|
| s413 批（11-12 20:04） | 0079443c6 | 当批构建 | 全域 sweep ×2 | **Pass ×2** |
| s414 批（11-12 23:12） | 0079443c6+注释 | 当批构建（freshness 门过） | sweep ×2 | Fail ×2 |
| flake3（11-12 23:20） | 同上 | 同上 | 负载窗（maketest 编译并发） | Fail ×3 |
| flake4（11-13 02:00） | bd41f9bd2 | 重建（freshness 门过） | 静机 load 10 | Fail ×3 |
| **attr（11-13 02:10，本轮）** | **纯 0079443c6**（detached checkout，零注释零上游） | 重建 BUILD_EXIT=0 | 静机 | **Fail ×3** |

## 定谳

**源归因排除**：纯 s413 源重建二进制 Fail ×3 vs 同源 s413 批二进制 Pass ×2——同源不同构建/不同时点翻转。s414 注释（已证 inert：纯注释 + attr 树无注释同态）与上游五 commit（attr 树不含）双双排除。翻转维度残留在**源码之外**：构建产物布局（rustc 并行 codegen 非逐位可复现 → codegen 单元划分/内联决策漂移 → 堆布局/分配序漂移）或运行时环境（同二进制跨时点翻转假设待检验——20:04 批二进制已被 23:12 重建覆盖，无法直接复跑）。

**勘根方向收窄（腿排程）**：
1. ~~同源连构建二进制 sha256 对比~~ **腿 1 已落锤（11-13 03:00，/tmp/s415/leg1-*）**：main 树（9204a0b31 基）同源 touch-rebuild 双构建 sha256 **A=af305840… / B=44a383b1… 不同**——rustc 输出非逐位可复现实证；双构建各 ior ×3 全 Fail（连前计 6 构建连 Fail：23:12 批 / attr / A / B——随机 regime 假设概率坍缩）。产物非确定 + 全近期构建同态 → 「20:04 Pass」 outlier 解释收窄为构建布局域（某 layout regime 触发/豁免 bug，20:04 构建落豁免侧）或机器时点状态。
2. ~~腿 2（首位）失败面探针插桩~~ **腿 2 已落锤（11-13 04:20，见下节）**：双通道键分裂 + 重复处理定谳——JS plain 批正确归一 Second 后，native 延迟排空以身份键重跑插入臂翻转 First；flip 维度 = 双记录处理序/键面随构建布局漂移。
3. 该子测断言对象为 fragment 双 option 迁移后的 selectedness 重算——与 s236 subtest 5 同根机制域（`_mo_notify` 钩挂位次/affKey 逐点臂），探针复用其插桩法。

## 腿 2 探针插桩定谳（11-13 03:40–04:20，/tmp/s415/leg2-*）

**装备**：part03.js 临时插桩 5 点（`_zwOptStateKeyOf` mint 臂 / `_zwSelectednessSetting` 入口+出口+multiple 早退 / `_zwSelectResetOnMutation` 入口 / `_zwOptWriteSelState`）→ `globalThis.__zwP415` 事件数组；探针语料 `zz-s415-probe-ior.html`（ior 逐字节拷贝 + 尾部 dump 失败断言 `P415DUMP:JSON`，s414 probe3 同款出流）；10 文件拼装 `node --check` 过。构建走 `CARGO_TARGET_DIR=target-s415` 硬链接克隆隔离（33.7s——r5510-heavy 并行 product-smoke 正占用共享 target/release，跨流零干扰；`target-s415/` 挂 .git/info/exclude）。

**复现**：探针文件 3/3 Fail（签名与真实 ior 全同 expected "Second" got "First"）；真实 ior 在插桩构建上同步 Fail——插桩不改 regime，dump 直读失败现场。

**机制定谳**（dump 108 事件全量未截断，/tmp/s415/leg2-dump.json）：

1. `target.appendChild(fragment)` 的 **JS plain 批记录先处理**：`mut added=[First(p8), Second(p9)]` 文档序、`set-in aff=null` → 走查 last-wins → `write First=false` → **Second=true 正确归一**（11 天稳定 Pass 的既有路径）。
2. **其后第二条记录经 @-handle 身份键重处理**（`set-in aff=@__n8`）：该时点 `_zwListOfOptions` 枚举序**倒转 `[Second(@__n11), First(@__n8)]`** → affKey=First 绝对胜出语义 → `write Second=false` → **First 翻胜**。
3. 断言读 `[First]` → "got First"——与 Fail 签名逐字吻合。

**发射点与根因域**：第二条记录来自 native 通道**延迟排空**（`drain_native_mutations_to_mo` → `__zw_mo_notify_native` → `_mo_notify`，part12.rs:2020 / webview mo_host_trigger 先例）——**一次 appendChild(fragment) 双通道双记录**（JS wrapper 同步批 + engine native 排空）。native 臂重读 selectedness 时，JS 归一写的 clean 层条目在 `plainOpt:p8/p9` 键下，`@__n8/@__n11` 键不可见 → 属性回落 true → 插入臂重跑 → 翻转。**通道间键分裂 + 同一逻辑变异重复处理**坐实为根因域。

**翻转维度解释收窄**：Pass/Fail 由双记录的处理序与键面决定（JS 批是否写身份键 / native 排空是否含该事件 / 排空时点枚举序）——三者皆随构建布局/分配序漂移；20:04 s413 批二进制落「无翻转」侧，此后全部构建落翻转侧。invocation-mode delta 已排除（s413/s414 final.sh sweep 行逐字节相同，差异仅 bookkeeping）。

**修复方向候选（下轮腿 3 定夺，未施工）**：(a) native 排空去重/抑制——同逻辑变异已被 JS 记录归一时不再重发（engine/webview 桥共享管域，需全量 sweep 验证）；(b) 键面统一（JS 归一双写或 `_zwOptSelOf` 跨键读）；(c) 插入臂幂等门。倾向 (a) 域勘根先行：重复处理是根因，键分裂是放大器。

**还原与干净回钉**：part03.js 还原（sha256 c422fbcf… 逐字节一致，git 零 delta）、探针语料删除、清回构建 C（freshness bin>shim、`__zwP415` ×0 零残留）ior ×3 全 Fail——连 Fail 构建链延伸至 5（23:12 批 / attr / leg1-A / leg1-B / C），插桩构建独立同机制复现。target-s415/（2G 硬链接克隆）留作下轮探针 30s 复用底座。

**残差（腿 3 候选）**：native 排空的确切触发时点（dump 显示翻转落在同文件下一 subtest DOM 操作之前——同 turn 内嵌套排空或微任务边界，未分档）；「20:04 批为何无翻转」考古降为次级（修复优先于考古，维持腿排程既定原则）。

## 口径影响

s414 账本在册红 +1F（213F+16T）不变；本切片零代码变更，纯勘根轮。ior 键在勘根收口前按在册红记账（11 天稳定 Pass 前科 + 跨源翻转实证 → 按 flake 治理不抹红、不入 favorable 单列）。

## 下一步

**腿 3（下轮首位）＝修复方向定夺**：native 排空域勘根先行（`drain_native_mutations_to_mo` 触发时点与缓冲内容分档 → 去重/抑制面定稿 → 施工 + 全量 sweep 零位移验证）；键分裂面（b/c 候选）作备选。挂账池与立项征询不变。