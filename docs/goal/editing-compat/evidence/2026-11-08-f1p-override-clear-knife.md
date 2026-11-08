# F1'-A override 清除刀（2026-11-08）

## 根因链（三层证据）

1. **Chrome 行为侧证**：multitest qV-before 恒读默认值 `rgba(0, 0, 0, 0)`——即便前一子测刚执行过 backcolor #00FFFF；全语料零条 expected 含链外 monospace。Chrome 端程序化选区变更（每案 setup 的 removeAllRanges+addRange）清空 typing style + spec overrides，无跨子测污染。
2. **引擎考古**：`_zwSync`（part06 Selection shim）只清 typing tags（Tags/TagsOff/CmdArmed/Link/LinkCarry），不清 `_zwVaStateOverrides`/`_zwVaValueOverrides`（part01d）。值 override 唯一清除点是惰性 staleness reset（`_zwVaOverridesReset` part01d.js:1535，读时 stored≠current 则清）——而 F6 delete 路径 take/putRaw + reanchor（part01f.js:1303-1325，preservesOverrides 语义）把 stored 重锚到新位置，使跨子测陈旧 override 在下次读时**免于 staleness 判清**（保鲜）。insertText 变异核快照读（part01.js:9628 `__zw_ce_value_override_snapshot`）消费陈旧值 → typed 段以 monospace/7/#ff0000/#00dddd 陈旧壳重裹。
3. **绿面对照**：inline delete 路径 override 位置派生已正确；F1' 族败面全部集中在 block-merge + 跨案污染叠加形态。

## F1' 29 键分解（f3-mt-post.log 基线 144 Fail 内）

- **A 族 25**（override 污染，本刀）：[3]-[18]（16 键，[swcss, delete/forwarddelete, insertText]）、[22]-[26]+3（8 键 fs5 族，[swcss, delete/fd, fontSize(5)(, foreColor), insertText b]）、[1]（fs4 单命令）。
- **B 族 4**（typing wrap 放置/合并，下一刀）：[2]（swcss(true) italic span merge）、[19][20][21]（bold>italic 出壳/前向壳并入）。
- 注：[1] 实际 got `<font size="4">a</font><font size="7"><br></font>` 为 br 残壳并列形（fresh 裹已成功），败因属占位 br/残壳面非 override 污染——本刀不翻，挂 F5/F6 域。

## 刀体

- part01d.js：`globalThis.__zw_ce_value_overrides_clear` 导出——清 `_zwVaStateOverrides`/`_zwVaValueOverrides`，`_zwVaStoredRange = null`（下一次读走既有 staleness reset 重锚）。
- part06.js `_zwSync`：清除序列尾挂 typeof 守卫调用（part01d 晚于 part06 加载，typeof 保调用序安全）。
- 内部路径不经 `_zwSync`（不受影响）：typed caret `_ranges` 直写（part01.js:10066）、delete reanchor+putRaw（F6 preservesOverrides 语义保全 g17 qV-after-delete 绿面）、结构命令 take/put 包裹尾回写恢复。
- md5 锚：part01d=5ab2368ec895e74527c510847e5a9f83、part06=a6461624a02e25a6c5988b779368131a。

## solo 对照实验（zz-probe-f1ov.html，pre/post-knife 双跑）

探针 5 案（污染四值 override → addRange 建案选区 → 命令链），断言口径 = 刀契约（S1-S3 陈旧壳负断言 + S4/S5 fresh 正断言）：

| 案 | pre-knife | post-knife | 判读 |
|---|---|---|---|
| S1 mono-clear | 红：`<font face="monospace" size="7" color="#ff0000" style="background-color:rgb(0, 221, 221)">A</font>`（与语料 [3] got 签名逐字节一致） | **绿**：零陈旧、裸 A | 清除生效 ✓ |
| S2 fs5 | 红：裸 `b` | 红：got 逐字节同 pre | fresh-after-delete 不裹=既有缺口（探针缺 typing 遗留态），刀零扰动 ✓ |
| S3 fc | 红：裸 `b` | 红：同 | 同 |
| S4 fs4-single | 绿 | 绿 | fresh fontsize 裹不受损 ✓ |
| S5 对照 | 红：裸 `d`（`foodbar`） | 红：同 | fresh collapsed foreColor+type 不裹=既有行为，非刀回归 |

首版探针 bug 记录：`setupCaseV2` marker 剥除 `deleteData` 用原始偏移未同步 `]` 左移 → 案面残 `]`、邻字符误删，首跑作废；修正后（先算调整后偏移再统一剥除）重跑。pre-knife 对照经 stash 往返（快照槽位 SHA b6b8d0c9 / ae5b2825，pop 后 md5 复核）。

探针保真度边界：探针上下文（scratch host 污染、无 typing 遗留态）无法复现 multitest 页的 caret 入壳行为——S2 pre-knife 裸形与语料 [24] pre-knife 裹壳形的分歧证明落点由 typing 态（tags/carry）驱动而非 override；故探针只判 override 面契约，语料翻转预测依赖 mt 批。

## 机制预测（fs5 族 8 键 got 落位推演）

pre-knife got 落位：text-shell 变体（[22]=20209、[26']=20238）b 在壳内 text 尾（splice 面）；color 变体（20217/20246）b 在壳外兄弟位；`<b>` 变体（20224/20231/20253/20260）b 在 `</b>` 后。post-knife 推演：壳内 splice 位等值跳裹（eff size==fresh x-large）→ `ab` 合并形 = expected → **翻 2**（双页至 4）；其余裸/独立裹形 ≠ expected → 内容漂移不翻。[3]-[18] 无位置派生（replay 快照空 → 不裹）→ 漂移不翻。**F2P 预期 2-4，窗口 [2,16] + 翻键 ⊆ fs5 签名**。

## 门禁批判读

- 批锚：HEAD=f1fb5956b（pull 先于发射；incoming 域不触 shim——批后入站 2 提交 = layout-engine R5465 abspos + net 46 号 M1 wss TLS fixture，均带各自落基门禁，与刀 hunk 域不相交，VERDICT 传递先例条件成立）。shim md5 锚 part01d=5ab2368e / part06=a6461624 双门过。
- 红线：probe-f2 14/0、probe-f3 7/0、probe 224/17、ip/iol/iul 零漂移、inserthtml/exec×3 vs f3base P2F 0、mt vs f3-mt-post P2F/post-only/lost 0 + rows 恒一 + F2P ∈ [2,16]、enginelib 6 红名册、battery 七腿。
- v1 首发 ABORT 记录（15:56 发射，probe 腿止）：rc=2 而 P=224 F=17 与期望一致、内容 judge vs 基线零漂移——定谳 **make/test-guard 的 rc 是坏门信号**（r3 日志自身即证：probe-f2 全绿 rc=1、probe 17F rc=0、mt 144F rc=2、enginelib 6 红名册 rc=101 常态）。v1.1 修：probe-f2/probe-f3/probe/电池四腿 rc 门撤除，计数 + probe 腿内容 judge（vs knife11-v3-probe-post）双证为门；15:5x+ 重发，md5 锚不变，日志同文件覆盖。

### v1.1 判读（16:00 发射，calibrate 段 ABORT 手审）

- 前段七腿全绿：probe-f2 14/0、probe-f3 7/0、probe 224/17 + judge vs knife11-v3-probe-post 零漂移、ip 7195 rows 恒一全零漂移、inserthtml 1565 恒一、exec×3（1806/1/6 rows）全零漂移。
- mt：rows 10092 恒一，F 73→70，judge **F2P 3（assertion 级）** P2F 0 post-only 0 lost 0 VERDICT OK（∈[2,16] 机制预期窗）。
- calibrate：entry 级翻键 **6** ∈[2,16]——fs5 签名 4 = 预测 text-shell 双键（census 204 delete + 212 forwardDelete，`<p><font size="5">a</font></p>` splice 面）×双页，**机制预测命中**；非 fs5 翻键 2 触发 ABORT 红线（脚本 predicate 非 fs5 须 0）。
- **手审定谳（ABORT 复盘，翻键合法域内）**：2 翻键 = 同一断言双页（`[["delete",""],["inserttext","g"],["inserttext","h"]] "<div>{abc</div><div>def</div>}" compare innerHTML`，multitest-ab + ?9001-last）。pre-knife got（f3-mt-post.log ab :10101 / main :20266 assert_in_array 明细）= `<div><font face="monospace" size="5" color="#ff0000">gh</font></div>`——恰为 F1'-A 根因链陈旧 override 壳（monospace/size5/#ff0000 污染族与 A 族 [3]-[18] 同源，值来自早前子测残留非本链命令），post-knife 翻绿 `<div>gh</div>` = expected 命中（execCommand 返回值三行双侧恒 Pass，唯 compare innerHTML 翻转）。**刀体无域外行为变更**；census 归簇 F7 delete-merge-div 系树形聚类口径非根因口径。机制预测漏判处：[3]-[18]「无位置派生→漂移不翻」推演对 delete-merge 落点变体不成立（陈旧壳确有包裹）。
- 工具瑕疵披露（非门）：f2ga-calibrate.py 系 F2 轮工具原样复用，family universe 过滤硬编码 F2 域（`createlink` in sig / delete+`<a `），故 F1' 翻键恒 out-family、stay-red 78 = F2 族键非本轮账——in-family/out-family/stay 三行本轮不可用；批门以脚本 grep 签名自证。
- ab 页尾截断：post log ab 页 10034/10092（−58 尾段）= 已知预算墙截断伪象族（F2 轮两发同签名第三发消失），main 域判读不受影响（红线以 judge-dps main 域为准）；翻键行位于截断点前，为真实状态变更非伪影。
- v1.1 中止点：iol/iul/enginelib/battery 未跑。v1.1 产物留档 `-v11` 后缀（f1p-mt-post-v11.log / f1p-mt-flip-v11.txt / f1p-mt-stay-v11.txt / batch 日志 batch-f1p-override-clear-gate-v11.log）。
- **尾链收敛（并行判读轮同判）**：并行续轮独立手审同判（回执 [f1p-mt-abort-manual-review.txt](../../../../.acceptance/dps/f1p-mt-abort-manual-review.txt)——逐键 pre-got 实证与上文同字节签名，fs5 双键 got 嵌套壳 `<p><font size="5">a<font face="monospace" size="5" color="#ff0000">b</font></font>c</p>` 隔断 splice 逐键落位），并落三件：①尾腿批 batch-f1p-tail-legs.sh（16:53 发射持锁：iol/iul/enginelib/battery×7，门禁与主批同源，mt 产物复用）；②resume.sh（calibrate 复核修正谓词 = 钉本节白名单 2 键 + fs5 insertText 签名，锁blocked 未跑）；③multitest-ab 单页带刀复跑（f1p-mt-ab-rerun.log，TIME_LIMIT 3600，核 −58 尾截=预算墙伪象，F2 先例同法）。本判读轮自拟 v1.2 全量重发草稿与 ①② 冗余且产物文件名共槽，未发射即删除；ABORT 复盘结论两轮独立一致。
- **发射权/判读权裁定**（ACTIVE-BATCH 17:0x 核验轮条目）：写者轮 3250910 死于 16:58:33（末动作 = -v11 存档在位；其 evidence 填充未落盘，现文件仅本判读轮节无合并残留），核验轮按回执代跑认领——**v1.2 按写者设计备好**（batch-f1p-override-clear-gate.sh 17:04:48 改制，v1.1 原脚本存 -v11.sh；calibrate 白名单门与本判读轮独立设计等价，另加 ab 截断不设门由 ③ 复跑兜底），ab 复跑落地后发射，判读权归核验轮。tail-legs 批 6/7 电池腿全绿（残 1 腿）。
- v1.2 全量批 ALL-DONE（18:17:00，head=f1fb5956b 全程零换基）：probe-f2 14/0、probe-f3 7/0、probe 224/17 + judge vs knife11-v3-probe-post 零漂移、ip 7195 / inserthtml 1565 / exec×3（1806/1/6）恒一零漂移。
- mt 判读（main 域 judge + 独立 per-assertion 全行级 diff 双证）：rows 10092 恒一，F 73→70，**F2P 3（assertion 级）** P2F 0 post-only 0 lost 0——fs5 delete + fs5 forwardDelete（text-shell splice 双变体，机制预测命中）+ div-merge 裸链白名单键；ab 页同三键 realFlips 3 + lost 5 = 300s 脚本级预算墙截断伪象（Makefile:1079 ZW_SCRIPT_TIMEOUT_MS 独立于 TIME_LIMIT，v1.1 三方回执定位口径；ab-rerun 17:06 同二进制 solo 10092 全账恰 3 翻键零其他漂移兜底）。calibrate entry 级翻键 6 ∈[2,16] = fs5 4 + 白名单 2 全命中 OK。iol/iul 恒一零漂移、enginelib 3815P/6F 红名册逐名恒等、battery 七腿 F=0 T=0。
- F1' 记账：mt 残 73→70（main）/144→138（双页实域，ab 5 lost 为伪象非实残）；A 族余键 [3]-[18] 按机制预测漂移不翻——污染层已清（got 告别陈旧 monospace/fs5/#ff0000 壳签名），无位置派生缺陷面留 F1' 后续刀；[1] br 残壳面挂 F5/F6；B 族 4 键（[2][19][20][21]）= 下一刀（基线签名已取证：[2] swcss(true) style-attr 并壳+占位 br、[19][20] bold>italic 出壳裸文本、[21] 前向壳并入分裂形）。
- 收口执行体：17:4x 注册轮（d86ef4ba）按账本串行门（18:22 条目）于 18:47 窗持 /tmp/zw-dps/close.lock 代跑——发射轮会话死亡实证（18:22 条目）+ 17:33/84 轮窗口零笔迹；判读独立复核（per-assertion 全行级 diff）与批 judge 同判，与本执行体同一，无并发冲突。
