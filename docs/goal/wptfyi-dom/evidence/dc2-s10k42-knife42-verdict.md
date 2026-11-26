# 刀㊷ 判读（s10k42）——prepare 管道 TT 门序三修正＝script-enforcement-005 整页 4F 清零＋010/011 classic 臂 2F 清零（真实池 48→44F）

日期：2026-11-27 · 域：trusted-types / prepare-the-script-element · 树锚：刀体 patch md5 `79734a77fed4dbc0a57773e2bbb671c1`（发射前 TREE-MD5-OK＋重放后 REAPPLY-MD5-OK 双断言）

## 目标簇（池内入刀前 6F）

- `script-enforcement-005` 2F：空体脚本经 default policy 变非空须执行（判据 `assert_array_equals` 得 `[]`——policy 未被调用，html＋svg 双面）。
- `script-enforcement-010`/`011` 各 2F：default policy 回调内翻转 el type 属性（classic→module）后须按**转换后**类型分派（classic 臂得 `[C,S]` 缺尾消息；010 判据体 `undefined=0` 的 CLASSIC/MODULE 双消息＝strict/sloppy 直证）。

## 根因（三处同源，prepare-the-script-element 门序）

- **门序**（005）：part03 `_zwScriptMaybeStartDynamic` 内联臂把「空体早退」放在 TT 门**前**——TT spec §enforcement-in-scripts 的转换步先于「无 src 且 source text 为空 → return」空查（空查消费**变换后**文本，005 页注 PR 579 判据原文）。空体不可信脚本永不进门 → default policy 永不被调。
- **分类时点**（010/011）：type 门分类读于门**前**——policy 回调在转换窗 `removeAttribute/setAttribute('type')` 翻转 classic/module，经典尾/模块臂按**陈旧**分类分派。
- **回落读陈旧**（005 空体救援不可达的第二层，探针钉死）：入臂融合读 `el.text` 得 JS 真值 `""`，但 `el.textContent` 回落读在 sel 形代理的移除标记/pending 窗口落 **host 快照**（mutation-blind 陈旧 `";"`）→ 快照等值判 trusted 静默直 eval 抢章——探针 tap 序列 `[T] enter text="" → [T] gateline text=";"` 逐字节定位（R334 pending window 同族）。

## 修复（part03 门移位＋part14 门内豁免/导出）

- part03 门移位：TT 门移到空体早退**前**；门首查 `__zwTtScriptExecGate` 即返臂保持非 enforcement 页零路径变化（145 形）。救援返空＝prepare 已过转换步 → **置 run-once 章**（R387/s97ib 插入链直 eval 臂以此表为副本执行门，缺章时拿原 DOM 文本重跑——首版曾致 005 变空面 `[C,S,RUN]` 多余 RUN，二版修）；救援返非空 → `_zwScriptTypeClassify` 重分类（blocked → return；nomodule/for/event 不重查，语料无该面 FIXME）。
- part03 回落读：enforcement 页空 `el.text` 回落改走 `__zwTtChildTextFused`（part14 §6.2b JS 侧融合遍历导出）；非 enforcement 页 textContent 回落不变。
- part14 门内豁免：空体且无 default policy → **静默不执行零违规**（block-text spot 面判据：innerHTML TypeError 中止后 append 的空体脚本＋无政策＝零违规零执行）；**有政策时空体仍进转换**（005 判据，policy 以 `('',type,sink)` 调用）；非空无政策照走管道（违规+阻断，001 判据族）。

## 首版回归事故记（已修并入门序设计）

block-text spot 面 `no_trusted_type_violation_for` 得 1——空体＋无政策曾走 `_zwTtSinkValue` 违规派发；violations tap（`[V] Element innerHTML sample=2+2 → [G] enter text= → [V] HTMLScriptElement text sample=`）定位后落门内豁免臂，spot 回绿。教训：空体双态（有政策转换/无政策静默）是 TT spec 转换步与「空 → return」次序的自然推论，缺一即违一侧判据。

## solo 页级（post 树，终版字节）

| 页 | base | post | 判 |
|---|---|---|---|
| script-enforcement-005 | 2P/2F | **4P/0F（整页全绿）** | rc=0 |
| script-enforcement-010 | 0P/2F | 1P/**1F**（classic 臂绿；module 臂＝runner 通道缺） | rc=1 |
| script-enforcement-011 | 0P/2F | 1P/**1F**（同上 SVG 面） | rc=1 |

010/011 余 2F＝module 方向 `__zw_compile_inline_module` 仅 webview.rs 注册（runner 挂钩域 grep 零命中实证）——两面**归 module 子系统切片池**（不再计散布池）。

## 邻域回归扫（17 页全 baseline）

001 34P/002 33P/003 32P/004 31P 全 0F；006/007/008.https/009.https 各 3F（module 切片位，零漂移）；pwrv.sub 4P/0F；block-text html 9P/1F＋svg 8P/1F（余 1F＝module 面，k41 同带）；eval 族八页全绿（k41 6F 清零面保持）；inheriting-csp-for-local-schemes 1P/1F/1T 不变（iframe 域挂账另刀）。

## 电池四腿＋TT 同树 A/B（solo，串行，2026-11-27 16:14:00→16:48:24 UTC，墙钟 34m24s，无 ABORT）

| 腿 | 计数 | 判 |
|---|---|---|
| sanitizer-api | 713P/65F | F/T 多重集 SET-IDENTICAL vs k41-post（comm diff=0） |
| domparsing | 1314P/221F/1T | SET-IDENTICAL（comm diff=0） |
| web-components | 17487P/920F/20T | F/T SET-IDENTICAL（comm diff=0） |
| dom | 57183P/362F/12T | Fail 多重集 362=362 diff=0；T +3＝replaceChildren 单页停摆族（solo 串行已知时序伪影） |
| TT base（checkout＝HEAD 含刀㊶） | 2204P/55F/20T | vs k41-tt-post **SET-IDENTICAL diff=0**（对锚正确性自证） |
| TT post（重放 md5 断言过） | **2208P/51F/20T** | **+4P/−4F；GONE 恰＝005 html＋svg＋010 classic＋011 classic；NEW=0** |

四腿 rc=2 均为既有基线失败集（rc 只留痕）。TT 腿总 F 55→51；真实池口径沿刀㊶收账 48→44F（010/011 余 2F 归 module 切片池另册）。

## 结论

刀㊷ 净增 **+4**（005 ×2＋010 classic＋011 classic），TT 真实池 48→44F，**累计 84,250（距 ≥99,947 差 15,697）**。module 面归并后散布池余键：inheriting-csp-for-local-schemes（iframe CSP 继承域挂账）／SharedWorker 2＋ServiceWorkerContainer 2（worker 面）／005 已清零退出候选。
