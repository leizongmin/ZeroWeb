# DC-2 · 切片 3（nodes/ranges 失败面：shim CharacterData 三 setter replace-data 边界，2026-11-22）

日期：2026-11-22。基线：切片 2（`ec4ef28dc`）。目标：`dom/ranges/Range-mutations-dataChange.html` 单腿失败面清零（诊断归因 `js_dom_shim` CharacterData 家族三 setter 丢 replace-data 边界调整，190F/腿量级）。

## 诊断（切片 2 dom 全腿遗留，本轮收口）

失败三分（按节点族计数，`(.*?)\.(data|nodeValue|textContent)` 全量正则）：

1. **data= / nodeValue=（accessor → `_write` 直写 `__nv`，无边界调整）**：sanity 过而 "Wrong start/end offset expected 0 but got 1/112"——live range 边界未按 replace(0, len, 新值) 收拢。
2. **textContent=（构造字面 plain 槽，赋值不触 `__nv`）**："Sanity check failed: data not changed as expected"——`data` accessor 读 `__nv` 与槽位失同步。
3. attached textContent 的 16F Wrong offset：`=` 变体先触发偏移断言、`+=` 变体偏移断言通过后死在 sanity——同机制两表象，无遗留分支。

**基线页预算截断定性（本轮新证）**：基线两腿同文件上报 854P/188F 与 921P/192F（Range-mutations 双页 ±45P 截断抖动带内），上报止于 detachedForeignTextNode 带（分别 1042/1113 of 1168）——**detachedXmlTextNode 族（216 案）在任何基线腿中从未运行**（切片 2 诊断「detachedXml 计 0 未解释」破案：runner 页超时预算截断，非 shim 缺口）。`detachedForeignTextNode` 基线 2F 落在已上报带（textContent sanity 面）。

spec 定谳（DOM Standard）：三 setter 语义统一为 replace(0, len, value)——`CharacterData.data` = "replace data of this with 0, this's length, and the given value"；`Node.nodeValue` CharacterData 分支同式（null → 空串）；`Node.textContent` CharacterData 分支 = "Replace data of node with 0, node's length, and value"。identity 替换（`data = node.data`）同样迁移边界（测试实证 expected 0 got 1/112），不得加「值相同跳过」捷径。可达性三证：`_zwAdjustRangesForData` 身份命中（part03.js `if (cont === node260) return true;`）先于根验证返回，foreign range 同节点调整不受跨树守卫拦截；`__zwLiveRanges` 无条件注册全部 Range（part06 `_makeRange` push，cap 8192）；offset accessor 恒读 `_startOffsetBase`/`_endOffsetBase` 槽。

## 修复（part03.js，R353；同享 `_zwAttachCharacterDataMethods` 全调用面：`_zwMText`/`_zwMComment` 工厂、engine-realm 物化 n4/n7/n8、part05 tn 本地家族）

1. `_write` 删 `n.textContent = nv` 槽位同步行：textContent 转 accessor 后读 `__nv` 天然同步；槽位直写在转换后触发自身 setter 递归、转换前直写绕开 `__nv` 使 data 失同步。
2. 新共享 `_setData`（`adjust(0, _cur().length, 新值.length)` → `_write`；`_write` 变量晚绑定引用保 D1 持久化链）；insertData/deleteData/replaceData/splitText 既有显式调整路径不变（`_write` 本身不含调整，双重调整不存在）。
3. data/nodeValue setter 改走 `_setData`；新增 textContent accessor（get 读 `__nv`，set 走 `_setData`）。
4. spec 链接：`#concept-cd-replace` / `#dom-characterdata-data` / `#dom-node-nodevalue` / `#dom-node-textcontent`。

模式与 R260 前作两家族一致（part05 D1 桥接家族、part06 MO 镜像家族各已有同款 textContent accessor + adjust）；本刀补齐第三家族（纯 JS shim 家族）。

## 验证（守卫入口）

探针腿（`make testharness-dom FILTER=Range-mutations-dataChange TIME_LIMIT=3600`）：**1167P/0F/1T** vs 基线两腿 854P/188F、921P/192F——F 清零；上报 1167 超两基线（1042/1113），页超时签名（`pending:1, tests:1168`）三腿恒等＝预算截断既存；0F 覆盖基线全部 F 族（foreign 216/216、xml 216/216、detachedText 216/216 满额，detachedForeign 已达 87 案含基线 2F 位形）。55.34s 增量 release 编译干净（zero-engine → zero-wpt-runner 全链）。

预算未达面补证（探针页 `zzz-dc2s3-probe.html` 复用语料 `doTests(dataChangeTests.filter(...))` 机制，只跑 detachedForeign + detachedXml 两族 432 案）：**432P/0F/0T**（满额零截断——两族全绿，页预算解除后无残余）。探针页已归档 `.acceptance/dps/dc2s3-probe-detached.html`（md5 `d4c94b44a55b7d6e039d3ccd7a5d1bf4`），不入语料树。dataChange 面全量 1,080 案（foreign 216 + xml 216 + detachedText 216 + detachedForeign 216 + detachedXml 216）实证全绿。

dom 全腿（`make testharness-dom TIME_LIMIT=3600`）：**57,877P/495F/11T** vs 切片 2 基线 57,472P/683F/12T（+405P/−188F/−1T）：

- F 集合迁移：comm GONE=188/NEW=0，GONE 全落 dataChange——385 文件零 Pass→Fail 迁移。
- 逐文件 P 对账 total-drop 0（无任何文件隐性丢 Pass）。
- 归因分解：dataChange 854P/188F/1T → 1164P/0F/1T（+310P/−188F，其中 +188 为基线 F 转 P，+122 为预算带内原截断案完成）；`Range-mutations-replaceData` 1051P/0F/Timeout → 1146P/0F/完成（+95P/−1T，页预算带完成，F 恒 0——非语义面，记账同 replaceChildren 波动带先例）。

wc 全腿（`make testharness-web-components TIME_LIMIT=3600`）：**20,653P/1,235F/17T** vs 切片 2 基线 20,667P/1,239F/17T（−14P/−4F）：

- 差量恰为切片 2 两探针页退场（probe-dc2s2 14P/2F + zzz-dc2 0P/2F，19:37 已归档 `.acceptance/dps/`）——非语料面 11,915P/1,194F 与切片 2 恒等（20,653−8,738=11,915）。
- awk 全量 F 集合 NEW=0/GONE=4（全为探针页行）、逐文件 P drop=0：真实面零变化。
- 判读事故注：wc 日志触发 grep 二进制失效（`grep '^Fail'` 双侧同静默返回空），首轮 comm 假绿——awk 全量重提取定谳，证据以 awk 口径为准。

## 剩余失败面

- 页预算截断带（detachedForeign 尾部 + detachedXml 216 案在全腿中不计分）：runner 页超时预算域，非 shim 面；探针 432P/0F 补证后挂账（全腿计数受页预算带影响，波动同 Range-mutations 截断抖动族）。
- RT 深语义 41F / accname testdriver 通道 37F + 3 Unsupported 案：切片 2 遗留泊位不动。
- `ParentNode-replaceChildren` ±3P/±1T 波动带：DC-3 空载 solo 复锚。
