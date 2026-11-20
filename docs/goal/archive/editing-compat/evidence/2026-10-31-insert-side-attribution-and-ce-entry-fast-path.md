# 插入侧残余归因（细粒度 tape）+ s111 残余定谳历史态 + CE entry 空表快路落地

**日期**: 2026-10-31
**前置**: [簇 A removeChild 融合列表复用收刀](2026-10-31-cluster-a-removechild-fused-rebuild.md)
下一步①的「插入侧残余（S5 ≤300 线未达 346.6：s111 臂 34-46µs/append / CE loop 20-40µs）」挂账。
**结果**: ① s111 臂残余**定谳历史态**（tape 三窗复测 ≤1-3ms/2000 子、≤4µs/append，被 10-30 簇 A
t8f 印章前移等修复吸收，与 §4 ~650µs text-child 差额同款关闭先例）；②「CE loop 20-40µs」仍在
（bulk ~21µs/子），**主项 = `_ceEntryFor` 未缓存 key 的 host `is` 属性往返**，空表快路落地后
seg15 41.3→30.3ms（−27%）、探针 fill 墙钟 −7%；③ 逐项刷新插入侧残余地图（见 §4），R126/
LiveNLSync、trap glue、本地解析、t8r 物化四项维持既有「独立整轮」scoping。

## 1. 方法

沿 [removeChild 收刀轮](2026-10-31-cluster-a-removechild-fused-rebuild.md) §1 的 tape 方法：
shim 顺序 delta 段带（`__zwTk` 段差 + `__zwTc` 计数，`Date.now()`），本轮两版检查点——

- **v1（20-28/41-44/10-18）**：`_mo_notify` 入口/SCRIPT 臂/HCLiveInvalidate/s111 臂/NA 同步
  + innerHTML setter 主相（R125 快照/解析/host set/R5241+注册/CE/R304/notify）；
- **v2（细分 30-34/50/51/53/54/55）**：seg20 切四段（ParentLinkBump/s109 臂/`_mo_id`+R188/
  deliver/subtree）、setter 解析与 s118 拆分（55）、R5241 与 t8r 物化拆分（53/54）、slotchange
  与 CE 断连拆分（50/51）。

探针页 `zz-probe-s5-insert.html`（editing/run/，gitignore 域保留复用；前轮已建）：
w0 `zz-s5` = createElement 容器（**handle 域**）innerHTML= 2000 span、w1 `zz-ap` = handle 容器
createElement+appendChild ×250、w2 `zz-reset` = `getElementById` 容器（**sel 域**）重设
innerHTML=。每窗 3 样本，`make testharness-editing-run-one FILTER=zz-probe-s5-insert` 单案通道。
测完 tape 整体 `git checkout` 回退（v1/v2 两版均未入库），修复净树跑门禁。

## 2. 归因（tape v2，3 样本均值）

| 窗口 | wall | 主项（seg） |
|---|---|---|
| zz-s5 handle fill ×2000 | 133-174（~146） | seg55 解析 `_zwFragmentAdded` ~44ms、seg54 R5241+**t8r 物化** ~42ms、seg15 **CE 环** ~41ms、seg24 HCLiveInvalidate ~10ms |
| zz-reset sel 重设 ×2000 | 76-108（~89） | seg55 解析 ~39ms、seg24 HCLiveInvalidate ~22ms、seg11 R125 removed 快照 ~13ms、seg51 CE 断连 ~5ms |
| zz-ap appendChild ×250 | 44-51（~46） | **seg20 ~85µs/append**（= notify 入口前后的 appendChild trap glue）、**seg24 ~73µs/append**（HCLiveInvalidate）；seg30-34（s109 臂/`_mo_id`+R188/deliver/subtree）合计 ≤5µs/append |

四点结论：

1. **s111 臂定谳历史态**：三窗 seg25/26（s111 臂全程）≤1-3ms/2000 子、zz-ap ≤4µs/append
   ——「34-46µs/append」系 10-30 归因时点数字，其后落地的 t8f 创建印章前移 + `_realTag`
   handle-only O(1) 表查（[簇 A 修复](2026-10-30-cluster-a-clone-perf-attribution.md) §7）已吸收。
   `n111.rec/walk` 计数正常（每 record 1 walk、2001 walk/2000 span），臂逻辑无异常放大。
2. **CE loop（seg15）主项 = `_ceEntryFor` cold key**：连接环对每个物化子调 `_ceApplyConn`
   → `_ceEntryFor(key,…)`；未缓存 key 走 `_realTag` + `_ce_registry[tag]` miss + **host
   `__zw_has_attr_handle(handle,'is')` 往返**（customized built-in 反查，每元素一次）——
   2000 子 = 2000 次纯浪费往返（无 CE 页面 registry 结构性空，两查恒 miss）。
3. **notify 内部机制段全净**：`_mo_deliverToId`/`_mo_any_subtree`（无 observer 零迭代）、
   `__zw_parent`（host 注册回调 ns 级命中，非 §7.1 未注册 miss 形态）、R188 `_makeProxy`
   均 ≤1-2ms/窗——10-30 trap 层四笔（6e294cd8a）效果保持。
4. **bulk fill 大头在架构常数项**：本地解析 `_zwMBuildBodyTree`（~21µs/span，纯 JS 解析器）
   + t8r 物化 `_wcRebuildAsHandle`（~21µs/span，bilibili `.style` TypeError 语义承担者）
   合计 ~85ms/2000——两者均为既有正确性修复面，非本轮 scope。

## 3. 修复：`_ceEntryFor` 空表快路（part03，+10 行）

`_ceEntryFor` 缓存 miss 后加 `if (_ce_registry_gen === 0) return null;`——gen 只在
`_ceDefine` 递增、从不清零，=0 ⟺ 从未 define ⟹ `_ce_registry` 结构性空，tag 直查与
`__zwCERegistryLookup`（同读 `_ce_registry`）双双恒 null，`_realTag` + host is 读全可省。
**先例**：part04:12920 `_ce_registry_gen ? _ceEntryFor(…) : null`、part05:1536 同款注记——
本修复把同款门下沉到 `_ceEntryFor` 本体，覆盖 `_ceApplyConn` 连接/断连 walk、part05 attr
路径（1911/1949/5192）、part04 属性变更面（3742/4115/4414/4561/5224）全部调用点。

语义面：① gen=0 期不写 `_ceEntryByKey` 缓存、不盖 `_ceIsChecked` 章（缺席 ≠ 已查）——define
后 gen bump 走原链，is 反查照常发生；② **顺带修一潜伏边**：旧代码 `_ceIsChecked` 章跨
gen bump 存活，gen=0 期插入的元素 define（customized built-in）后 is 反查被旧章跳过——
本修复该形态改走完整反查（更正确）；③ `__zwCeInvalidateEntry` 对缺席 key delete 为 no-op。

### A/B（同 tape v2 在树，3 样本）

| 指标 | 修复前 | 修复后 | Δ |
|---|---|---|---|
| zz-s5 wall | 162/142/132（145.3） | 163/132/113（136.0） | **−7%** |
| zz-s5 seg15（CE 环） | 38/44/42（41.3） | 33/34/24（30.3） | **−27%** |
| zz-reset wall | 76/87/103（88.7） | 84/87/102（91.0） | 噪声带 |
| zz-reset seg51（CE 断连） | 3/2/10（5.0） | 2/2/8（4.0） | 噪声带（walk 自身为主） |
| zz-ap wall | 46/47/51（48.0） | 66/44/45（51.7，s0 离群） | 噪声带（热点在 trap glue+HCLiveInvalidate，非 CE） |

seg15 残余 ~30ms = `__zwDetailsScan4100` 全树扫（R4100 details name 组，内置语义非 CE）
+ `_ceApplyConn` walk 常数（`_elKey`/`'prop'` 记账/childNodes 读）——前者属 details 语义
独立面，后者已无 host 往返。

## 4. 插入侧残余地图（本轮后现行）

| 项 | 量级 | scoping |
|---|---|---|
| bulk fill 本地解析（seg55） | ~21µs/span | `_zwMBuildBodyTree` 解析器优化，独立轮（全 innerHTML 面共享，爆炸半径大） |
| bulk fill t8r 物化（seg54） | ~21µs/span | bilibili `.style` 语义承担者，惰性化需 R309 视图语义重审 |
| appendChild trap glue（seg20） | ~85µs/append | ~15 项语义义务 dispatch，guideline #35 整体替换=独立整轮（既有 scoping 不变） |
| `_zwHCLiveInvalidate`（seg24） | ~73µs/append、~10-31ms/bulk | R55/R304 身份稳定副作用重审，独立整轮（既有 scoping 不变） |
| CE 环残余（seg15 后段） | ~15µs/子 | `__zwDetailsScan4100` 全树扫属 R4100 details 面，独立候选 |
| S5 ≤300 验收线 | 现探针口径 136ms/2000 | 与 S8 轮 S5 探针（346.6 口径）不同负载，不可直比；机制残余已按本表重排 |

## 5. 门禁（净树：tape 剥离 + part03 快路在，tree 91bc5de42 + fix，2026-10-31 深夜批）

| 门禁 | 结果 | 判定 |
|---|---|---|
| join-different-white-space-style（TIME_LIMIT=2400） | **7,518P/0F/0T** | 零位移（对 R5340-glue 轮账） |
| insertparagraph-in-inline-editing-host（TIME_LIMIT=1200） | **880P/0F** | 零位移 |
| engine lib（make test workspace 主矩阵） | **3,641P/1F**——唯一红 `test_ua_display_li_list_item_canvas_inline_s24_flip`（s24 已知红，红名与前轮 identical；3,635→3,641 +6P 为 pull 入账他流提交正漂移） | 达标（cargo test fail-fast 止于 engine，webview 补跑） |
| webview lib（test-guard 包裹 `-p zero-webview --lib` 补跑） | **832P/0F** | 零位移 |
| bench-gate（`ZERO_WEB_BENCH_CRATES=zero-engine` 定向） | **PASS**——microbench 1/1 + 页面绝对预算过（dirty=true 修复在树测得；硬件基线错配 WARN 既有账） | PASS |
| testharness-web-components（CE 面跨域，非基线面 stash-A/B 先例） | fix 在树 **4,022P/1,146F/5T** vs pristine HEAD **4,022P/1,146F/5T**——三轮独立跑（pristine 腿 / fix 腿 / 并发方独立跑）计数逐位一致，文件差仅时序字段 | 零位移 |

执行形态：语料与 lib 门禁经 systemd user service（`zw-gate-chain-1031b`）串行跑批——本轮实证
**并发 make 目标会重编译 `target/test-guard`，运行中 guard 的 `current_exe()` 自 exec 落
`(deleted)` 路径 ENOENT（exit 127，cargo 编译全绿仍炸 launch 段）**；门禁批自此一律串行 +
脱离会话（systemd-run service 模式）。

**碰头附账（同刻双会话同面同批）**：web-components A/B 期另一支 Rally 会话在同一树上并行
编排同款 stash 基线跑——我方 23:09:24 pop 先行取走 fix stash，对方 23:10:20 设计上无条件
`git stash pop` 落到栈顶历史条目（ghost-br-round）产生 3 文件 UU 冲突（git 保留 stash 无丢失）；
恢复 = 冲突文件 `checkout -f HEAD`（stash 条目留置不动），我方 fix 与 evidence 完好。
stash 编排属共享面操作，后续轮次开工前按 [[rally-resume-check-concurrent-rounds]] 先盘
`systemctl --user list-units 'zw-*'` 与 stash 栈。

**合并树复测（2026-10-31 23:22-23:30，systemd）**：s185 三笔（form.elements 面，part03/04/05
+414/−53）于本轮六门之后并入同一提交树（ba268c969），组合态此前无人门禁——补跑 join
7,518P/0F/0T + inline-host 880P/0F **零位移**，合并面收口。

## 6. 复现入口

- 探针页：`tests/wpt-runner/wpt-data/editing/run/zz-probe-s5-insert.html`（gitignore 域保留）。
- 单案通道：`make testharness-editing-run-one FILTER=zz-probe-s5-insert TIME_LIMIT=600`。
- tape 复建：本文 §1 检查点清单 + [removeChild 轮](2026-10-31-cluster-a-removechild-fused-rebuild.md) §1 方法。
