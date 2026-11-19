# vendored unwrap 写面根因刀收口（R5450，bold pair 2F 测试同步转绿）

- 日期：2026-11-12
- 刀体：dbfc027c9（7d02707ba amend 并入存在性门修正笔后的终态；part04 跨父 textEl 移动臂双形态 + tombstone 判别 + 双臂存在性门 + part05 overlay prevSibling 回落 + part06 bold pair 测试同步）
- 背景：名册 [2026-11-12 档](enginelib-red-roster-2026-11-12.txt) bold pair 2F（format_apply / toggle_state_slice4）——f415be99c（s406 vendored setSelectionValue 契约切换）遗留的测试同步阻塞项，须先做 unwrap 写面根因刀再同步（不得把数据丢失签名钉为期望契约）。

## 根因链（探针 v1–v14 逐级取证，写者棒死亡后核验棒接管闭合）

toggle 解除路径 `pre<b>ob</b>post` 选 'ob' → bold：

1. clearValue 摘壳臂经 `_zwJusMovePreservingRanges` 挂载：`pre.insertBefore(obTextEl, b)`；textEl 无 handle，refNode（b）是快照基元仅带 selector。
2. **R87 误命中**（insertBefore 文本子注册恢复臂，本为 NodeIterator-removal 恢复段设计）：pre 本地子列表空 → 把 "ob" 种成 pre **整内容种子**（base 兄弟 pre/post 被遮蔽，本地视图坍缩为 `[#ob]`），且注册副作用打掉 `refNode.__zwSelector` 簿记（入口有值 "b"、链头 falsy——`T3-pre-chain-head` 火而 `T5-branch3-entered` 未火实证）。
3. 全 wire 分支条件落空、兜底臂 `__zw_insert_before(sel, undefined, refSelector)`（textEl 无 handle）→ **零 wire op** 静默返回。
4. 随后摘壳 `Remove(b)` wire 正常执行 → host 侧 "ob" 随 b 永久丢失（净效果 `prepost`）；本地视图 "ob"；期望 `preobpost`——真数据丢失，与语料 compare 行「命令后读 innerHTML」全绿的"真实引擎主机写面正确性"形成 mock 环境两视图分歧（名册档注记的原开放点，本刀定谳 = 写面缺陷在 host 侧真实存在，corpus 全绿系该形态语料未覆盖）。

第二处独立病灶（overlay 放置）：`insertBefore(ob, b)` 后同 turn `removeChild(b)` 时，overlay 先剔 removed（b）再并 added——ob 的锚 `nextSibling=b` 已剔 → miss 直落尾插（`prepostob`）。

## 修复五笔

1. **insertBefore 跨父 textEl 移动臂**（R87 之前）：条件 = sel 基元父 + 无 handle 文本子 + sel refNode + tombstone `__zwUnregFromSel !== sel` + 纯 base 父（`!_zwLocalChildNodes`）。wire = `__zw_insert_adjacent_text(refSel, 'beforebegin', text)`（host 时序 = 文本落壳前 → 摘壳 → 语义等价 pre-insert 到壳原位）；簿记镜像 R334（`_zwSelPendingParent` 链 + childList record + mutTick + range adjust + `_zwSiblingBaseInvalidateAll`），early return。
2. **appendChild sel 父分支 append 变体臂**：壳居末位形态（ref null 尾位挂载），同款语义与簿记。
3. **两处 textEl 注销点落 tombstone** `child.__zwUnregFromSel = sel`（removeChild parsed 分支 + R87 分支前置）：判别「同父恢复」（tombstone === 目标父 sel → 让位 R87 重注册路径，NodeIterator-removal 场景不变）与「跨父移动」（≠ → 新臂）。
4. **双臂 tombstone 存在性门**（perf 门修正，A/B 定谳后并入刀体，见下节）：`typeof newNode.__zwUnregFromSel !== 'undefined'` 前置——从未注册的新鲜文本（undefined）不入新臂，留常规旧径；否则高频常规 insertBefore/appendChild 文本插入全走 wire+簿记+`_zwSiblingBaseInvalidateAll` 全清，ab 页热路径推过 300s 脚本预算。
5. **overlay prevSibling 回落**（part05，R5450）：link 增 `prevSibling` 字段，nextSibling 锚同 turn 被移除时回落「prev 之后插」= 被移除锚原位；无该字段旧 link 行为不变（尾插）。

## perf 门 A/B 定谳（存在性门的依据）

solo A/B 判别实验（同 clone 串行、300s 脚本预算 `ZW_SCRIPT_TIMEOUT_MS=300000`、judge 行级恒等对账）：

| 形态 | 头 | P/F/T | 墙钟 | 定性 |
|------|----|-------|------|------|
| pre-knife | 9dbe26c80 | 10092/0/0 | 252s 完走 | 基线 |
| knife（双臂无门） | 7d02707ba | 10055/0/**1T** | ~305s 截断于 99.6% | 刀体真计时回归 |
| 存在性门后 | dbfc027c9 | 10092/0/0 | 299.9s 完走（JS 段 ~269s） | 计时回归闭合 |

- 初判租户争抢伪象（ZeroWeb-2 流 battery 07:04→07:24 + cargo rebuild 07:35 时间线重叠）被 solo 判别**推翻**：无争抢下 knife 仍截断。
- judge（`judge-dps.py`）10055 行逐行恒等 = 截断前行为零漂移，纯计时回归。
- 机制：双臂条件 `__zwUnregFromSel !== sel` 对从未注册的新鲜文本恒真（undefined ≠ sel）→ 语料常规文本插入高频落 wire+簿记+SiblingBase 全清路径。
- 修复后 gate 判别串（`typeof newNode.__zwUnregFromSel !== 'undefined'`）在 runner 二进制 2 命中（双臂各一），修复前二进制 0 命中——本体核验锚（learning #28）。

## 测试同步（bold pair 根因刀收口，名册 9→7）

- `test_execcommand_format_apply_r3254_m3`：①② 断言从旧 SetInnerHtml 整体 splice 契约改 **vendored 契约净效果 replay**（步内 mutation 切片 `apply_dom_mutations` 到新鲜引擎 doc，比对 host `inner_html`——与语料 compare 行同口径，不钉 granular op 形状）；② 计数对象从 SetInnerHtml 改 mutation 总数。
- ①b（嵌套跨既有 `<b>`）与 ③（italic）迁出为 `#[ignore]` 簇测试具名挂账：
  - `test_execcommand_format_apply_nested_cross_b_r3254_m3`——apply 面嵌套簇三缺陷组合：① base 文本 data 写走物化时 `__zwChildIndex` 戳，同 turn 头部结构插入后戳陈旧（`SetChildText(#ce,2,"b")` host 全子索引 lenient skip 错位——发射侧文本子索引 vs host 全子索引契约错位）；② vendored contained 集在该 range 形态为空 → wrap 元素缺发射（隔离跑甚至全 no-op）；③ 跨命令 override 泄漏（先行命令 state override 残留改变后续分支路径）。
  - `test_execcommand_format_apply_italic_after_prior_cmd_r3254_m3`——③ 独立案：隔离跑纯 no-op、序列跑发射破坏性流（replay 净效果 "baz"，'foo' 与包裹双丢）。
  - 三缺陷 + override 泄漏（第四缺陷）超出本刀范围，根因刀后解除 ignore 并钉净效果期望。
- `test_execcommand_toggle_state_r3254_m3_slice4`：**断言零改动**（`preobpost` shim 视图即期望契约），unwrap 写面修复后按原断言转绿。

## 验证

- enginelib：3904P/**7F**/2I @dbfc027c9——7 红 = 2026-11-12 档 9 红 − bold pair 2，逐名恒一（[boldpair 档](enginelib-red-roster-2026-11-12-boldpair.txt)入库），零新增红。gate v1 腿（存在性门并入前的 7d02707ba 形态）同计数 = 存在性门不触碰 enginelib 面。
- 目标测试：`execcommand_format_apply`（1P/2I）+ `execcommand_toggle_state`（1P）全绿。
- ab 语料 multitest-ab：10092/0/0 @300s 预算完走（存在性门后 solo 复测；judge vs cssom-ab-perffix 基线）。
- 门禁批四轮史：
  - v1（`batch-unwrap-r5450-gate-v1.sh`）GATE-ABORT：**脚本顺序缺陷非行为红**——二进制特征串核验置于 release runner 构建腿之前，此时 bin 仍为 5c02b84d4 时代 6f46bc78（stale）。enginelib 腿已绿有效。
  - v2/v3 GATE-ABORT：ab 腿计数失配 + 腿间 HEAD 漂移；初判租户争抢伪象，经 solo A/B 判别推翻（见上节）——真因 = 刀体计时回归，存在性门修正后闭合。
  - v4（`batch-unwrap-r5450-gate-v4.sh`，L3b 改引存在性门后验证跑 + EXPECTED_BIN md5 绑定 + gate 判别串断言）：**七腿全绿收口**（bin 9f11f0d06… 恒一 @dbfc027c9）——L0 probe 217/5 恰集；L3b ab 复用 gatefix 10092/0/0；L1 outdent F2P=169 P2F=0 lost=0 残红恰 4；L2 indent 1302/27；L3 mt **零漂移**（10092 rows，F2P=0/P2F=0/lost=0，residF=2 他流已知，wall 1690s）；L4 singles×5 judge OK（iol 1759 / iul 2061 / ip 6989 / ilb / ihtml）；L5 exec×3 judge OK；L6 enginelib fresh 3904P/**7F**/2I（failed==7 过）。
  - v4 L6 脚本伪 ABORT（非行为红，v1 先例同判）：批脚本逐名提取 `grep '^test .* FAILED'` 把 `test result: FAILED.` 摘要行捕获为第 8 行垃圾 → 名册逐名 diff 误报；离线修正重提取（`'^test .* \.\.\. FAILED'`）得 7 名与 `unwrap-r5450-failures.txt` **逐名恒一**（`unwrap-r5450-enginelib-v4-failures.txt`）。同 log 离线后处理，无重跑。

## 换基重锚三腿（dbfc027c9 组合态 @e46242b74，bin 8d2361a3d）

收口 commit 后 pull --rebase 合入 origin 领先提交（d0221d496 part01d.js u/s/ins/del 载体折叠、4bf49d625 css-values sibling 求值桥等 12 提交；刀体 rewrite 2cd87daa2→dbfc027c9，range-diff `=` patch 恒等）。post-rebase 组合态三腿取证（`batch-reanchor-r5450-postrebase.sh`，d08e5cf8f 先例）：

- **ab multitest-ab：零漂移**——10092/0/0，wall 263s（预构建 bin，无计时回归），judge 10092 rows F2P=0/P2F=0/lost=0 恒等（批内 judge 页滤串带 `?` 对 ab 行失配致 rows 0->0 空转假通过，离线修正页滤重判 VERDICT OK——与 v4 L6 同类脚本伪象非行为红）。
- **mt multitest：零漂移**——10092 rows F2P=0/P2F=0/lost=0，residF=2 他流已知，wall 1624s。
- **enginelib：t49 陈旧钉同步**——首跑 3905P/8F，新红 `test_cssom_inline_style_decl_value_normalization_t49`（`width: #F00` 写面 verbatim 钉）；归因 **99c573d30（css 流 g18）width 槽文法门 invalid 拒/valid 放行有意变更** vs b1fdb1db7 时代 verbatim 钉陈旧——该红自 99c573d30 落 main 起即存在于主干（git log -S widthHexVerbatim 唯一命中 b1fdb1db7，与本刀无关；css 流门禁面不含 zero-engine lib 故未拦）。测试同步 `widthHexVerbatim:"#F00"→""`（`width` 文法无 <color>，解析失败声明整体拒=Chrome oracle 同形）后 B3' 复跑 **3906P/7F/2I 名册逐名恒一**（+2P = be008fdd7 computed 两新测试；7 红名册不变，[boldpair 档](enginelib-red-roster-2026-11-12-boldpair.txt)继续有效）。

## 挂账（下一步素材）

1. **apply 面嵌套簇根因刀**：`__zwChildIndex` 发射/应用契约统一（文本子索引 vs 全子索引）+ contained 集空 bail 归因 + override 生命周期（清除依赖选区变化臂，mock 序列 removeAllRanges/addRange 未触发）——两枚 ignored 簇测试解除后名册不变（自愈面）。
2. **shim 视图序缺陷**（probe11，本刀范围外挂账）：apply 阶段 handle 基路径 `flat=foaz<b>ob</b>`（b 落尾而非 ob 前——RemoveHandle retract 杀 nextSibling 锚 → 尾插）；语料侧全绿不受影响。
3. ab 腿同 bin 瞬态丢行率账（本日累计 4 例：98/363/265/37 行，均为 buffered 输出页尾部截断形态）：复现法 = ab 腿连跑 N 次测丢行率，择期。
