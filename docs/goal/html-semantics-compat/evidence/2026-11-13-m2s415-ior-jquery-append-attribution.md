# M2 切片 415（round-1→leg-3 收口）— ior jQuery append() 新红归因：B1 批内同锚倒转定谳 + 修复

日期：2026-11-13 凌晨（承 s414 下一步首位：ior 新红三跑归因；leg-3 收口同日）
commit：round-1/腿 1/腿 2 随 docs commit b9058372b（零代码变更轮）；腿 3 + 修复随 s415 施工 commit

> **⚠ round-1 结论更正（腿 3 定谳后）**：round-1 归因表 s413 批一行的「树 0079443c6」系**误标**——s413 批二进制 20:04 构建，早于 B1 patch commit 5d26a28eb（20:13）与 s413 收口 commit 0079443c6（22:04）本身，实为 **pre-B1 树**。「同源不同构建翻转，源归因排除」不成立：两批二进制**异源**（pre-B1 vs post-B1），源归因**成立**（B1 即根因引入点，见腿 3 谱系定谳）。腿 1 的 rustc sha256 非确定性为真实事实但**退出因果链**（ior 翻转无需构建布局漂移即可解释）。

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

> **⚠ 腿 3 更正（11-13 05:10）**：本节「机制定谳 1/2」与「发射点与根因域」已被推翻——dump 中「JS plain 批正确归一 Second」实为**下一 subtest（#by-innerHTML innerHTML parse）**的记录（leg-2 探针事件无 ts/target 字段，相邻 subtest 同类事件误绑）；fragment appendChild 流程全程**只有一条 R47 批记录**（腿 3 v2 探针 moq ts 字段实证）。「native 延迟排空双通道重复处理」经 kill-switch A/B 证伪（`ZW_MO_HOST_TRIGGER=0` 下 Fail 持续，腿 3）。真机制见腿 3 节。

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

## 腿 3 终局定谳 + 修复（11-13 04:30–05:10，/tmp/s415/leg3-* fix-*）

### 3a. native 排空证伪（kill-switch A/B）

`ZW_MO_HOST_TRIGGER=0`（env 经 `WebView::new` webview.rs:934 生效，runner 不覆盖）关断 `drain_native_mutations_to_mo` 后探针 ior ×3 **全 Fail 同签名**——native 排空通道与翻转无关，腿 2「双通道双记录」归因撤回（leg3-mooff-*.log）。

### 3b. v2 探针（事件流加 moq ts/tag 字段）——fragment 流单记录实证

`__zwP415` 事件带 ts + r47 tag 重跑（leg3-dump.json 158 事件）：`target.appendChild(fragment)` 只产生**一条** `moq type=childList r47 added=[First,Second]`（文档序），其后 selectedness 走查（`sel-in`/`write` 事件）在该记录处理完毕时**枚举序已倒转**——翻转不发生在记录发射/处理层，而在记录消费后的**选项列表视图层**。腿 2 dump 的「JS plain 批正确归一」确认为下一 subtest 事件（无 ts 字段误绑，教训入账）。

### 3c. v3 探针（`_zwListOfOptions` 视图源 dump）——同锚顺插倒转机制定谳

selectedness 走查时点 dump 视图源（leg3-dump2.json 160 事件）：sel 的 base cache = `[]`、pending bucket `added = [p11(文本), First{pv:p11}, First-dup{ps:"null"}, ws{pv:p11}, Second{pv:p11}, Second-dup{ps:"null"}]`（pv/ps = 记录级 prevSibling/nextSibling 镜像；dup 条目 parentSel 为字符串 `"null"`，合并以 `parentSel !== sel` 跳过——inert，挂账）。`_zwOverlayPendingChildNodes` 按 prevSibling 锚定位插入：p11 → [p11]；First（pv=p11）插锚后 → [p11, First]；ws（pv=p11）插锚后 → [p11, **ws, First**]；Second（pv=p11）插锚后 → [p11, **Second, ws, First**]。**同锚顺插 = LIFO 倒转**——文档序 [First, ws, Second] 的批视图变 [Second, ws, First]，s236 last-wins 走查读 First → `got First`，Fail 签名闭合。

### 3d. B1 谱系定谳（根因引入点）

git 谱系实证：B1 patch = **5d26a28eb（11-12 20:13）**「嵌套簇 B1 根因刀」在 `_mo_notify` childList 臂给批内每个子镜像 `prevSibling: baseRecord.previousSibling`（commit hunk 原注释「record 的 previousSibling 本就是 spec 字段，镜像即得」——漏了 record 级 sibling 字段是**整批首尾锚**非逐子锚）。时点链：s236（11-02）→ s413 批二进制（20:04 构建，**pre-B1**：批子只带 `nextSibling: record.nextSibling`（fragment 批 = null）→ 合并尾插保序 → ior 11 天稳定 Pass 与 20:04 批 Pass ×2 同因）→ B1（20:13）→ s414 批（23:12）起全部构建含 B1 → 确定性 Fail，连 6 构建连 Fail 无一例外。窗口（5d26a28eb~1..0079443c6）shim 面唯有 B1 触及（git diff --stat 实证）；「同源翻转」系 round-1 树误标（更正见卷首），翻转轴退化为纯源差，**零构建非确定性依赖**。

### 3e. 修复（施工，四件套）

真锚语义统一施工——`_zwNodeParent` 链接与 `_zwSelPendingParent` 槽两条锚通道全改**逐子真锚**，批外锚消费方按新语义适配：

1. **part01.js:8794**（`_mo_notify` childList 臂链接写入，根因刀）：record 级镜像改逐子真锚——`prevSibling = added[i-1]`（首子接 record.previousSibling）、`nextSibling = added[i+1]`（末子接 record.nextSibling），spec 兄弟语义本如此；锚按节点 identity 消费（R5450 同口径）；B1 的 prev 兜底意图保留且加强。单子批与 B1 逐字节等价（i=0 双向接 record 锚），零行为漂移。
2. **part05.js `_zwHCTreeOrderSync`（13686）**：scoped 集合重排改**倒序处理 addFlat**——真锚下批内成员 next 锚 = 邻位兄弟，倒序使后位兄弟先落位、前位锚它即插其前；正序在锚兄弟未重排时跳过移动（初版修复实测 t8e 多节点批红，探针实序 [a,n2,b,n1] = v1 缺陷形复原）。锚不在集合的走查链经真锚逐位逃逸；单子批倒序=正序。
3. **part05.js `_zwBaseIdxShiftForAdded`（12972）**：next miss 回落 prev 双查（else-if 串联改顺序双查）——B1 apply 时坐标平移契约在真锚下批内非末子 next 锚 = pending 兄弟（不在基底），漏计平移；批首经 prev=record 外锚解析、末子 next 直命中，计数与 B1 共享锚时代逐条对齐。
4. **part04.js iAH 槽锚（11755）**：insertAdjacentHTML 解析子挂父槽同改逐子真锚（`_iahRef` 是 record 级整批共享后继，afterbegin=旧首子）——初版只改链接面时 t8e iAH 批钉探针实序 [I,B,SPAN,EM]（槽臂共享锚 + 倒序重排反转）；批内子后继 = 邻位子（R4600 variadic 槽「项 i 后继 = 项 i+1」既有同型先例），末子/无后继接 record 锚。

10 文件拼装 `node --check` 过；探针插桩全还原（part41.rs 零 delta、shim 三件 sha 对 orig 待终验）。

### 3f. 验证（终稿树）

- **solo**：ior 文件修复前同法 ×5 全 Fail → 修复后 ×3 全 Pass exit 0（fix3-ior-*.log；中间态两轮 ×5+×3 同 Pass）；t8e 全家 7/7 绿（fix3-t8e.log，初版修复单点时 multi_node 红、探针定位槽臂后四件套收口）。
- **消费方全量审计**：`_zwNodeParent`/`_zwSelPendingParent` sibling 字段读方逐一核过——overlay 合并（真锚正确，ior 定靶）、TreeOrderSync（倒序适配）、BaseIdxShift（prev 回落适配）、B1 锚重定向 hunk（纯 node-ref 语义无涉）、R91 清理/R92 模板 info/R4600 variadic（自建锚或 parent 反链，无涉）。innerHTML sel 路径（15148 槽锚恒 null 尾插）无共享锚无涉。
- **sweep ×2**（终稿树，test-guard + RAF_FRAME_DRIVEN，对 /tmp/s414 基线 join）：run1 8277P/214F/15T + run2 8277P/212F/17T。join-b1（对 s414 run1）：**state-diff=2**——`ior … jQuery append()` **Fail→Pass（修复定靶）** + `select-home-end-pagedown-pageup.optional.html` 文件键 **Timeout→Fail**；**new=0 gone=0** 零键集位移。join-b2（对 s414 run2）：**state-diff=2**——ior Fail→Pass + `select-home-end-pagedown-pageup-detailed.optional.html` **Fail→Timeout**（home-end 双文件族内换位：s414 detailed=F/非detailed=T → 终稿 run1 双 F / run2 双 T，T↔F 跨跑漂移全落历史观测集——s406/s411 双跑 Fail、s412–s414 Timeout、s414 detailed 静机连跑展开↔塌缩双向翻转在案，红集不变）；new=0 gone=0。join-cross：stable=8502 state-diff=2 仅 home-end 双文件族内漂移、new=0 gone=0。**数字闭合**：run2 212F = 214 −1(ior) −1(detailed 形迁出 F)、17T = 16 +1(detailed 入 T)；run1 214F = 214 −1(ior) +1(非detailed 形迁入 F)、15T = 16 −1——逐项无残差。（s415 首轮 sweep2/maketest 期 release 重建窗口 127 ×2 与 target-disk-guard 收尾删 release 产物 127 ×1 三连——重建后补跑收口，make-test 后须重建 release runner 已有在案。）中间态轮（part01 单点）sweep 曾见 new=6 探针语料残留键，终稿轮语料已删归零。
- **make test 全量**（终稿树，B1 nested_cross_b 域跨域门禁）：**155 腿 154 绿 + 仅 engine lib 在册红腿**（3911P/**7F**，失败清单与 s414 maketest 逐名 diff **IDENTICAL** 零位移〔img×2/fetch+stream×2/websocket×2/iframe-service-worker×1〕；3910→3911 +1P = t8e multi_node 回绿）。首轮 maketest（中间态树 part01 单点）engine lib 3910P/8F = 在册 7F + t8e 新红——即 3e-2/3e-4 两适配的动因，二次消费方审计闭环。

### 3g. 残留与挂账新增

- bucket.added **重复条目**（First-dup/Second-dup，parentSel 为字符串 `"null"`）：合并 inert 跳过，无行为影响；计数/记账面 bug 候选，挂账待勘根。
- 「20:04 批为何 Pass」考古**结案**：pre-B1，非构建布局（3d）。
- target-s415/（2G 硬链接克隆探针底座）与 /tmp/s415/ 工件留存备复验。

## 口径影响

在册红 s414 收口 213F+16T → **212F+16T**：−1F = ior 解挂（s236 机制域批视图根因修复，非 flake 治理）。home-end 双文件族内 T↔F 漂移红集不变不在册计位（measured/在册差额规则同 s414：run2 measured 17T − detailed-T 1 = 在册 16T、measured 212F = 在册 212F）。tentative/ 口径不变。ior 键勘根收口转修复解挂。

## 下一步

constructed-stylesheets 1T 勘根候选 / 挂账池承 s404/s405/s411/s412/s413/s414 + 新增（bucket 重复条目 parentSel 字符串 `"null"` inert 记账面）；checkable 父 label 2F 渲染域移交承 s410；select/combobox（簇 D）与 A2+D 簇立项征询仍待用户决策。本切片教训（record 级 sibling 字段 = 整批锚不可逐子镜像、共享/真锚双读方需同语义归一；探针 dump 必带 ts/target 防跨 subtest 误绑；归因表树列须核构建时刻 vs commit 时刻）已具 learning 候选。