# 刀㊻ 判读（s10k46）——S2 execCommand sink 门：主文档 execCommand insertHTML 命令值入 TrustedHTML 管道（TT 语料 +2P/−2F，S2 簇全收，真实池 33→31F）

日期：2026-11-27 · 域：trusted-types 散布收敛（census bb1d0aea0 §1 S2 簇）· 树锚：刀体 patch md5 `27b8588122caa623948637838b668a6e`（发射 TREE-MD5-OK＋REAPPLY-MD5-OK 双断言；patch 常驻 `/tmp/s10k46/knife46-part06.patch`）· 文件域＝`part06.js` 单点 18 行。

## 目标簇（S2，2F 两页）

| 页 | 面 | 根因链 |
|---|---|---|
| block-Document-execCommand | `Document.execCommand("insertHTML") throws.` | 主文档 execCommand（part06）对 insertHTML 命令值整体无 TT 门——enforced 页 plain string 不进管道不抛（block 页非默认政策存在仍须 TypeError：只有 default policy 转换 plain string） |
| trusted-types-reporting-for-Document-execCommand | `Violation report for plain string (insertHTML command).` | 同根因违规边：enforce 模式 throw 时须随附 SecurityPolicyViolationEvent（sample＝`Document execCommand|`＋clip 值，blockedURI＝trusted-types-sink） |

页内其余 7 面修 = 既有绿（TrustedHTML 直通、paste 非 TT sink、default policy 面 assert sink 名）。

## 修复（part06.js 单点）

主文档 `execCommand` 入口已知命令门（DC-4 exec3，未知命令返 false 不抛）之后、justify 快照之前插 TT 门：`cmd === 'inserthtml'` 且 `__zwTtEnforced()` 且 `__zwDocWriteReplay !== true`（write 回放窗豁免）→ `String(__zwTtSinkValue('TrustedHTML', arguments[2], 'Document execCommand'))`。管道语义全部复用 part14 `__zwTtSinkValue`（TrustedHTML 实例 isHTML 直通／default policy 缺失 → violation 派发＋TypeError／default policy 存在 → createHTML(input,type,sink) 转换）。同款门先例：detached doc 薄面＝刀㉒（part03，s10k22）；iframe 子文档薄面（part05）语料无 TT 消费面不接门。paste 非 TT sink（block 页 tt_commands 仅 insertHTML 实证）。

spec：https://w3c.github.io/webappsec-trusted-types/dist/spec/#enforcement-in-documents

## solo 页级（post 树，电池前先行）

`make testharness-trusted-types FILTER=execCommand` 全绿 11P/0F：block 6P（含 throws 面）、reporting 3P（含 violation 面）、Document-execCommand 2P（既有绿恒）。

## 电池六腿（solo 串行，2026-11-27 20:54:47→21:27:46 UTC，墙钟 33m0s，无 ABORT）

| 腿 | 计数 | 判 |
|---|---|---|
| sanitizer-api | 713P/65F | F/T SET-IDENTICAL vs k45 |
| domparsing | 1314P/222F | SET-IDENTICAL |
| web-components | 17682P/940F | F/T SET-IDENTICAL；P +182＝gethtml.html 单页（3239→3421，已知巨页截断抖动族，F 集零差排除） |
| dom | 57510P/374F | F/T SET-IDENTICAL；P +14＝Range-mutations 双页（dataChange +12／replaceData +2，已知截断抖动族） |
| TT base（scoped checkout＝恰 part06 回 HEAD） | 2233P/34F/19T | vs k45-tt-post **多重集恒等**（gone=0 new=0）＝字节形状锚：base 树＝HEAD(㊺)+part02 与 k45 post 树同构 |
| TT post（reapply md5 断言过） | **2235P/32F/19T** | **post − base ＝本刀效应 +2P/−2F**：GONE 恰 S2 两面（多重集 1+1）；F NEW=0；T 恒（19） |

判读行提取 LC_ALL=C awk＋wc 对账＋awk 多重集（comm 协作性教训守恒）；P 漂移页级 join 归因仅已知噪声族两页。四腿 rc=2 均为既有基线失败集（rc 只留痕）。

## 并行席字节洁净声明

写者席 s10k47 字节（part14＋testharness.rs）apply 落树于 21:27:59Z＝**晚于本刀全腿完成 21:27:46Z**（其发射回执自证「k46 DONE → 本席 apply 落树」）；TT base 与 k45-tt-post 多重集恒等反证该字节不在本刀任何腿的树态。本刀日志域零污染；s10k47 电池（写者席）树态含本刀 part06 字节，其 scoped A/B 自会归因（与 k45 电池树态含 part02 同构）。

## 归属与计账边界

- **本刀计账：TT 语料 +2P/−2F（S2 簇全收），真实池 33→31F（散布 12→10F：S1 3＋S3 2＋S4 5），套件 F 34→32，累计 84,270→84,272（距 ≥99,947 差 15,675）**。
- part02 navigate 五面（+5P/−5F）＝写者链自家前轮字节（归属定谳：0f85e51f transcript tool_use 审计 8 次 Edit，19:53:40Z＝其 coord 账本「03:53:40 落树」同刻；见仓库根 [.acceptance/dps/s10k45-coord.md](../../../../.acceptance/dps/s10k45-coord.md) 04:53 追加段），归其 s10k47 轮收编，本席不 commit 不计账。

## 结论

刀㊻ 净增 **+2**（S2 execCommand sink 门两面收敛），TT 真实池 **33→31F**，Timeout 19 恒，**累计 84,270→84,272**。池余：module 子系统 16F（R1 一体化刀位定谳已落档）＋散布 10F（S1/S3/S4）＋navigate 5F（写者链域）。下一刀位：写者席 s10k47 在飞（part14＋testharness.rs 域，其轮自结）；S1 eval 报告边簇与 module R1 依席位节奏排期。
