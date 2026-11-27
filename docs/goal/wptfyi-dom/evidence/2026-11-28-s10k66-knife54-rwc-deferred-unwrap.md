# s10k66 刀54：sanitizer rWC string-face 延迟展开（table-family 目标）＋写后真树移位 pass——sethtml-tree-construction replaceWithChildren #71 清零

日期：2026-11-28（UTC 链窗 20:08:28Z→20:26:40Z）。刀体 commit `0cba7aabc`（发射判读号 `2cfbef40e`，rebase 重放号变〔docs 对账 4db6d81a0 并入〕，三 blob ac1993506／0b42593f／7884f8f4 恒等复核＝判读域不变；首发射号 `a9d3ca7d4`＝宽域版已 ABORT 不判读）（三文件 `crates/engine/src/js_dom_shim/part04.js`＋`part05.js`＋`part13-sanitizer.js`，+104/−8）。发射 bin `970c05f87b77`（提交域字节，链后 md5 复核恒等）。刀号：**刀54**（十进制续编，上一轮 429 中断前已定刀号＋完成根因定谳与 v1-v4 探针链，本轮收养 WIP 完成验证落库）。

## 刀位与基线

控制面挂账池：san 残池 6F 中 `sethtml-tree-construction.html` **Testcase #71**（`<table><div><td>`，config `{ "replaceWithChildrenElements": ["table"] }`，期望树 `| <div>`＋`| <tbody>/tr/td`——unwrap table 后 family 子以兄弟身份存活）（锚 772P/6F/0T＝chain65 刀53 收账态）。

## 根因（上一轮探针链，v1→v4 迭代在案于刀体注释；本轮收养核证）

rWC 走 string-face（sel/handle 写链）时，过滤引擎 unwrap 在物化片段就地 splice → serialize 净串 → 既有写链 reparse。`<table>` 的子（tbody/tr/td/caption 等）是 body-context 解析的**上下文敏感标签**：裸串进 reparse 被丢弃/重构（#71 期望 `tbody>tr>td` 回环后仅剩 `div`，family 全灭）。结构性修复＝**延迟展开**：unwrap 目标保形序列化（标记 `data-zw-rwc-unwrap`），写入真树后由写后 pass 以子节点原位替换——无 reparse 无丢面。

上一轮 v1→v4 探针链（zz 探针＋/tmp/s10k66-bundle.js node 面，日志未留档、结论在刀体注释）：

1. **v1 真树臂挂死**（probe A2 定谳）：childNodes 读＝mock 数组而真树 insertBefore 不回写 JS 视图，真树移位循环 firstChild 永不推进＝挂死 → 改 JS 视图 mock 移位。
2. **v2 种子臂再订正**（probe dbg=2/0 reg=undefined）：createElement 真元素写链走 handle 域（sel=null），顶层读容器是 `_handleChildren[handle]` 重建岛数组而非 A-DC2 种子 → 双域分治。
3. **v3 双域落位**：handle 域 splice `_handleChildren[handle]`；sel 域 splice A-DC2 种子 `e.nodes`（R5356b `_zwLocalChildNodes` 源数组）。
4. **v4 活数组订正**（PROBE B1）：handle 域 childNodes getter 返回 `.slice()` 副本（part04 融合 childNodes），副本上 splice 不回写＝内层 unwrap 静默失效（仅标记摘除、结构不变）→ 递归与展开改走 `_handleChildren[el.__zwHandle]`／`_zwTextElsBySel.get(sel).nodes` 活数组，无身份兜底才回落读面副本。
5. **R184 normalize 判定谓词订正**（PI battery agnostic/4 实证）：展开后经 normalize 权威数组合并相邻文本，裸 `||` 判 `__zwIsText` 会把 nodeType 7（PI）误并入 Text 兜接（双 PI data 拼接＋后者移除）→ 按节点型排除 7/8。

本轮收养核证：中断前 bin（03:52Z 构建）`919bc8427393` 特征串 grep 实证（`data-zw-rwc-unwrap` ×5／`_zwSanRwcPostPass` ×4／normalize 修 `__zwIsText === true` ×1）＝树内 v4+normalize 修字节；单页 probe：sethtml-tree-construction **85P/0F**（#71 翻面）＋sanitizer-processing-instructions **45P/0F**（normalize 修防回潮）。

## 链上 battery 抓获回潮与收窄（chain66 ABORT）

首射 chain66（bin `919bc8427393`＝commit `a9d3ca7d4` 宽域版，20:04:20Z）：san 腿 **772P/6F**——P/F 计数与锚同但构成漂移：#71 翻面（−1）同时 `sethtml-with-custom-elements.html`「Custom element via is-attribute, where is attribute is allowed, but the element is replaced.」**新回潮（expected 0 but got 1）**。定谳：该钉面断言 rWC 后 `<div is=x-div>` **从未物化**（constructed/connected 恒 0）；宽域 defer 把标记元素保形写入 → 引擎写链经 is= 升级 machinery 物化 → constructed=1。

处置：chain66 ABORT（script 先杀防新腿＋腿链叶先 -9，pid 2800024/2800496/2800563/2800564 全数验尸干净；宽域版字节永不提交）＋收窄——**延迟展开仅对 table-family unwrap 目标生效**（`_ZW_SAN_RWC_DEFER_TAGS`：table/thead/tbody/tfoot/tr/td/th/caption/colgroup/col，其子为松散 family 成员＝reparse 必丢面）；其余目标（div/b/i/custom-element）回环本无损，就地 splice 维持 CE 未物化语义。语料 rWC 目标普查：tree-construction #74-76（div，template content 域＝defer 本就不传）、modifiers/basic-filtering（b）、custom-elements（is-div）——收窄后全部回锚行为，唯 #71 table 走 defer。

## 修复（三文件）

- part13：`_zwSanFilterList` 新增 deferRwc 参数——unwrap 臂对 family 目标改标 `data-zw-rwc-unwrap`（attrs 未过滤面无碍：标记元素写后随移位移除不进终树）；子树递归同传（嵌套 rWC 保形）；template content 递归不传（content 不进序列化面，标记会失联）。string-face 漏斗（`_zwSanSetHtmlImpl` sel/handle 臂）传 true＋写后调 `_zwSanBridge.rwcPostPass(sel, handle)`。target 域／sanitizeString 串消费面／parseHTML 漏斗不传＝零变化。
- part05：`_zwSanRwcPostPass`（`__zwSanBridge.rwcPostPass` 导出）——递归 unwrapKids 走活数组（`_handleChildren[handle]`／A-DC2 种子 `e.nodes`），标记元素摘标＋splice 换子，展开后经 R184 normalize 权威数组合并相邻文本（真浏览器 serialize-reparse 回环自然合并同形）。
- part04：R184 normalize 判定谓词 `_r184IsText` 按节点型排除 PI/Comment（`https://dom.spec.whatwg.org/#concept-tree-normalize`）。

## 判读（chain66b，20:08:28Z→20:26:40Z，pid 2802687，四腿）

| 臂 | 锚（chain65） | 实跑 | 归因 |
|---|---|---|---|
| san 全臂 | 772P/6F/0T | **773P/5F/0T** | ΔP+1=ΔF−1 闭合；翻面恰 sethtml-tree-construction #71、**新回潮 0**；余 5F＝parseHTML 2＋svg-animate 1〔SMIL 挂账〕＋inert-document 2 |
| dom 臂 | 57467P/361F/12T | **57489P/361F/12T** | F 页集恒等＋T 集恒等；P+22＝Range-mutations 截断噪声族（dataChange +6／replaceData +16，已知带内）＝刀零 dom 影响 |
| tt 臂 | 2290P/14F/18T | **2290P/14F/18T** | F/T 集逐字节恒等（diff 双空）；14F 全 zz 探针家族＝真实池 0F 维持 |
| ref 臂 | 725/725 | **725/725**（Layout 521/521＋Text 204/204） | 恒等 |

腿 rc 留痕：san rc=2／dom rc=2／tt rc=2／ref rc=0（rc 非门信号）；发射断言 `970c05f87b77`+head `2cfbef40e` 过＋链后 md5 恒等。提交前单页四臂：tree-construction 85P/0F、custom-elements 13P/0F（宽域版 1F 回潮已收）、modifiers 19P/0F（rWC b 就地臂）、processing-instructions 45P/0F（normalize 修）。语料页即常驻回归。

census：+1 → **84,392，距 DC-3 校准线 ≥94,093＝9,701**（原线 ≥99,947＝15,555）。

## 残相挂账

- table-family 目标带 `is=`（如 `<td is=x-td>`）假想形：defer 保形写入仍会物化（constructed≥1）——无语料钉面，accepted 近似（inline 展开该形本就丢 family 结构，两向皆损）。
- E1 附带发现承刀53：rWC tbody verdict 未命中疑 mock 域 table-family namespaceURI 形，无语料钉面，不在本刀范围。
- 引擎树保持标记元素暂态（语料读面恒 JS 容器视图）——若未来出现绕过 JS 视图的读面（如引擎侧直接序列化消费），wrapper 会泄漏；现语料无此面。
- san 余 5F：parseHTML 2（parseHTMLUnsafe 树形 `<script>`/`<html onload>` 面）＋svg-animate 1（SMIL 动画引擎缺口，页可达上限 21/22）＋inert-document 2（inert 文档 image 请求泄漏）。
