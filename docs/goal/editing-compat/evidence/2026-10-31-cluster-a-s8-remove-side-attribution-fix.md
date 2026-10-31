# 簇 A glue 残余：S8 重设移除侧相归因收口与修复（S8 ≤ 1.5×S5 达标）

日期：2026-10-31（R54xx 轮，接 renderer R383a 对账轮控制面挂账「簇 A glue 残余」）
范围：`crates/engine/src/js_dom_shim/part05.js`（三笔，+53/−4）
基线：8143d70be（R54xx 记账后，工作树含本轮改动）
验收线：S8 ≤ 1.5×S5（10-30 簇 A evidence §8.1 下半）；本轮任务只压移除侧，S5 ≤ 300 仍未达（见 §8）

## 1. 挂账

10-30 簇 A 归因轮（§8）账面：S5 首设 317.2ms、S8 重设 854.9ms（比 2.70），R263b/R262b 预筛门 + trap 层四笔只压了插入侧与 range 注册表，移除侧每子 ~250µs 未归因。本轮 tape 收口。

## 2. 归因方法

innerHTML setter 逐相时间戳 tape（临时插桩，`globalThis.__ZW_PROBE_TAPE===true` 门控，本轮已全剥），探针页 `zz-clone-cost-probe.html`（S5-set/S8-reset/S8b-reset2/S5c-fresh-set 四次 2000 span innerHTML 计时，用完已删）。三轮收敛：

1. 粗粒度 9 相（A childNodeList 读 / B、B2 摘链 / C、D 物化 / E CE 断连 / D2 sel stamp / F `_mo_notify` / G PAIdRemove 循环）——S8 特有成本集中在 **F ~110ms + G ~108ms**（E 17ms、A 13.6ms 为小项）。
2. F/G 内部子相（HC 子 tape）：F 内 = r2b（R52 purge）63-84ms + r2（host 快照门）8.7ms + collections 尾 10.4ms + 反链簿记 ~20ms；G 内 = `_zwPAIdRemove` ×4000 ≈ 97.6ms。
3. PA 子 tape（`nd.id` 读链单独计）：**单一最大 villain = `_zwPAIdRemove` 的 `nd.id` get-trap/attr 读链，20-32µs/次 ×6,000 调**（setter R125 尾扫 2×2000 + R52 purge 2000），跨 tape 轮次 idRead 累计 122-195ms——无 id 页面 `_zwPendingAddedById` Map 恒空，get 恒 miss，全数纯浪费。次 villain = **R52 稀疏假设在 bulk 移除下反转**：innerHTML 整体重设替换全部旧子 → 每子全命中 pa 表 → 逐命中 `indexOf+splice` O(n²)（4,000 remFlat × 4,000 pa 表）；pb1 桶对冲同构（探针 pb1 实测 13.4ms/次且随桶规模线性涨）。

## 3. 修复三笔（全在 part05.js）

① **`_zwPAIdRemove` 空表预筛**：`if (!_zwPendingAddedById.size) return;`——Map 空时任意 id 的 get 恒 miss（本函数唯一效果是命中条目剔除），O(1) 等价快出；有 pending id 时走原路径零变化。
② **r2b R52 purge 密集重建**：命中收集到数组，密集（≥8 命中）单遍 Set-filter 重建 `_zwPendingAdded`（按命中 Set 精确剔除，成员与序等价——非命中条目原样保留），稀疏保持原逐命中定点 splice。数组与 `_zwPASet()` 成员一致性保持：命中已从 Set 删除，重建只移除同一批命中。
③ **pb1 桶对冲同构密集重建**：同②作用于 `_pb.added`。

不采用全量 Set 化 `_zwPendingAdded`：全局表消费面（R51c 消零判定、slice45 豁免压实等）依赖数组序与原地 splice 语义，换结构风险大于收益；密集重建只在 bulk 形态触发，稀疏路径零改动。

## 4. A/B 验证

同日同负载（并行流 debug 构建结束后，`uptime`/`ps` 核净）三样本中位数，最终净树二进制（bisect 残留清理后复测，与清理前一致）：

| 案 | 修复前（同日对） | 修复后（净树中位） | Δ |
|---|---|---|---|
| S5-set | 351 | **346.6** | 不变（修复面只在移除侧，符合预期） |
| S8-reset | 567 | **410.0** | **−27.7%** |
| S8b-reset2 | 742 | **525.4** | **−29.2%** |
| S5c-fresh-set | — | 574.8 | 对照（全新容器首设） |

**S8/S5 = 410.0/346.6 = 1.18 ≤ 1.5 ✓**（修复前同日比 1.62）。探针 sanity-child-count 钉 Pass（重设后恰 2000 新子）。

注：S5 346.6 对 10-30 账 317.2 高 ~9%，为日间机器状态漂移（并行流负载史），非本轮回退——S5 路径未触碰（预筛 O(1) ×2000 可忽略）。

## 5. 门禁矩阵（最终净树二进制）

- inline-host（**窄口径 `insertparagraph-in-inline-editing-host`**）**880P/0F** 零位移
- void 双语料 **72P/0F + 48P/0F**、ilb **552P/0F**、wss **1104P/0F**、chd **420P/0F**、join 三案 **7,518P/0F**——全零位移
- bold run **1,138P/1,910F**，失败集对 10-30 基线（`bold-attrib.log`）与同日修复前（`bold-run2.log`）**逐位 identical**；墙钟 **117s**（前账 ~300s，−61%——bold 重型变体重设密集，S8 修复直接兑现）
- webview lib **830P/0F**；engine lib **3,625P/10F**（修复态二进制、bisect 前所跑，红集 = fetch/Request 族 8 + iframe SW + s24 flip 逐名 identical；净树二进制与之行为等价——清理只删 bisect 死读 + 还原密集阈值 8，语义零变化，探针与语料门禁已在净树复验）

## 5.1 合并树复测（撞车合并后终验）

push 前 pull 拉入并行流新账（shift×normal 幽灵 br 碰头仲裁 38772a918 + s177-s180 + docs），part05.js 另入 s178 radio sweep 净件臂（~1672 行，与本轮三笔 ~11680-12260 无交集；stash 自动合并干净，三笔复核在位）。合并树重编复验：

- 探针三样本中位：S5 352 / S8 416.6 / S8b 531.7 / S5c 596.3——**S8/S5 = 1.18 保持 ≤ 1.5**。
- 语料门禁全零位移：inline-host 窄口径 880P/0F、join 三案 7,518P/0F/0T、void 72P/0F + 48P/0F、ilb 552P/0F、wss 1104P/0F、chd 420P/0F。
- bold run 1,138P/1,910F 失败集仍对 10-30 基线逐位 identical，墙钟 118s。
- engine lib **3,635P/1F**——唯一红 = s24 flip（渲染流既有债）；拉入的并行流 s179 fetch 契约族根修清掉 fetch/Request 族 8 红 + iframe SW 红（对 §5 的 10F 账为外部改善，非本轮动作）。
- webview lib **832P/0F**（+2P = 并行流新钉）。

## 6. inline-host 8F 假回归归因（既有失败，非本轮引入）

复跑门禁时以宽 filter `inline-editing-host` 代窄口径，888 案 880P/**8F** 对昨账 880P/0F 触发假回归警报。四轮归因：

- 8F = `delete-in-inline-editing-host-under-shadow-root.html`（4F）+ `inline-editing-host-has-placeholder-with-{after,before}-pseudo-element.html`（各 2F），断言形态 `expected "X" but got "Type something"`（`:empty::after` 占位伪元素镜像文本机制）。
- **pristine HEAD（git stash A/B）同 8F，失败集逐名 identical** → 非本轮（修复或 bisect）引入；语料文件 mtime 10-27 未变。
- 根因 = **filter 口径**：昨账「inline-host 880P/0F」的 filter 是窄口径单案族（本轮窄口径复测恰 880P/0F 坐实），3 失败文件从不在门禁集，8F 与昨账 0F 无矛盾。
- 8F 本身为**既有失败**：嫌疑并行流提交 3bbc7ca56（s176 shadow 域定位寻址——shadow-root delete 案相关）/ f8fe57abd（ghost text 宿主镜像——placeholder 镜像相关）。按「并行流引入的不回退、只记账」记入待认领，未做提交 bisect（与两流当前工作面均无交集证据）。

方法论教训：门禁 filter 口径必须记入 evidence 原文（本次凭「inline-host」记法重跑宽 filter，代价 4 轮 bisect）；宽口径 8F 已知红在此记档，后续全量基线扫描若用宽 filter 以本账为准。

## 7. 探针方法论（承接 10-30 evidence §7，新增两条）

- test() 的 name 参数在**调用时求值**——测量须在顶层先跑完再调 test()，name 里拼测量值（`test(fn, 'ms=' + x)` 在 fn 内赋值 x 会 ReferenceError）。
- runner 语料通道按目录分：探针页在 `editing/run/` 走 `testharness-editing-run` 通道，`testharness-editing-other` 查不到（0 案静默 exit 0）。

## 8. 残余与下一步

- 本轮验收线 S8 ≤ 1.5×S5 **达标（1.18）**。S5 ≤ 300（10-30 线）未达（346.6）——插入侧残余 glue 小项：s111 臂（34-46µs/append）、CE loop（20-40µs）、removeChild trap glue，下轮按账推进。
- G 相残余：空表预筛后 `_zwPAIdRemove` 在无 id 页已近零成本；有 pending id 页面的 bulk 形态（id 页 × innerHTML 重设）未单独建账（语料面未见，暂不立项）。
