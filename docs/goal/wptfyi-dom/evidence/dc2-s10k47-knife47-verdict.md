# 刀㊼ 判读（s10k47）——S1 eval RO 报告边双面＝runner `.sub.headers` 侧车回退查找＋shim eval 全包装 RO 监测臂（TT 语料 +2P/−2F，池 31→29F）

日期：2026-11-27 · 域：trusted-types S1 eval 违规报告边 · 树锚：刀体 patch md5 `6d09b95fa36e8be3cb8ad3a73cd0336b`（发射 TREE-MD5-OK＋TT 重放 REAPPLY-MD5-OK 双断言；恰两文件 3145 B）。电池树态声明：本电池树＝HEAD＋本刀两文件＋兄弟席在飞 part02/part06 字节（刀㊻ S2 execCommand＝码刀 `b3dca53f0`＋判读 `e00280515` 已落库，本电池发射时其字节尚在飞未提交）；判读按 scoped A/B 精确归因（基线＝k46 六日志，TT-base 腿隔离兄弟席域），兄弟席效应不计本刀账。席位/波及审计：[s10k47-coord.md](../../../../.acceptance/dps/s10k47-coord.md)。

## 目标簇（census S1 三面中报告边两面；第三面定谳改挂）

census S1 原判「三面同管线宜同刀」经判据体细读**修正**：第三面 reporting-for-Window-eval「No violation reported for eval with TrustedScript.」失败 assert 为 `assert_equals(beacon, 'i ran')`（页面局部 `let beacon` 经 eval 赋值）＝**直接 eval 间接化边界**（包装在位→native 尾调 global 落点，k29 在案 FIXME「enforced 页内直接 eval 声明落点仍间接化」同族）非报告边——维持挂账不入本刀。本刀收两面：

| 页 | 面 | 根因链 |
|---|---|---|
| reporting-check-report-Window-sink-mismatch | 1F「Passing a plain string to eval throws.」 | 双头页（enforcing `script-src http: https: 'nonce-123'`＋RO `require-trusted-types-for 'script'`）sidecar 命名 `{page}.html.sub.headers`——runner `inject_csp_sidecar_meta` 只查 `{page}.headers` 落空＝页零 CSP→无门→eval 不抛 |
| trusted-types-eval-reporting-report-only | 1F「Trusted Type violation report: evaluating a string.」 | RO meta 自刀㉖ 起已注入、全包装 RO 装置臂在位，但 String 臂只认 `_zwTtParseCsp().enforced`——RO 页 TT face 整体跳过＝违规从不派发（`__zwMaybeInstallEvalGate` RO 装置臂成死装） |

## 修复（两文件两面）

- **face ① `.sub.headers` 侧车回退（testharness.rs `inject_csp_sidecar_meta`）**：`{page}.headers` 落空回试 `{page}.sub.headers`。波及面审计：TT 语料 `.sub.headers` 仅 sink-mismatch＋create-policy 两页；模板值行（`{{$id:uuid()}}`）键解析不受影响、非 CSP 键照跳；套件旗标面（仅 trusted-types 开启）其余 corpus 零变化。页 1 注入后＝RO TT meta（本刀 face ② 派 report 违规）＋enforcing script-src meta（无 'unsafe-eval'→eval CSP face 抛 EvalError＝assert_throws_js 臂）；harness 脚本全外链（scheme-source `http:`/`https:` 面，shim 无外链加载执法）零阻面。
- **face ② eval RO 监测臂（part14 全包装 String 臂）**：`enforced` 分支后接 `else if (_zwTtParseCsp().reportOnly)` → `_zwTtRoSink('TrustedScript', s, 'eval')`（刀㉖ sink 管道 RO 臂直接复用）：默认政策存在则运行 createScript（成功返值继续、无违规）；无政策/返 null·undefined → disposition 'report' 违规（sample `'eval|'` 前缀＝sinkName 载体、originalPolicy＝roPolicy）＋原串透传不执法不抛；回调异常原样传播。后续 CSP face 照走。规范 TT §eval-RO 语义与 sink RO 臂同源（§process-value-with-default-policy 管道）。https://w3c.github.io/webappsec-trusted-types/dist/spec/#eval-dom

## 电池六腿＋TT scoped A/B（solo 串行，2026-11-27 21:27:59→22:02:51 UTC，墙钟 34m52s，无 ABORT；k42→k46 协议，scoped 断言/checkout＝恰本刀两文件；发射前置 k46 DONE 门＋rustfmt --check RC=0 预检）

| 腿 | 计数 | 判 |
|---|---|---|
| sanitizer-api | 713P/65F/0T | F/T SET-IDENTICAL vs k46 |
| domparsing | 1314P/221F/1T | SET-IDENTICAL |
| web-components | 16525P/920F/20T | F/T SET-IDENTICAL；P −1157＝gethtml.html 单页（3421→2264，已知巨页截断抖动族，F 集零差排除） |
| dom | 57436P/362F/12T | F/T SET-IDENTICAL；P −31/−43＝Range-mutations 双页（已知截断抖动族） |
| TT base（scoped checkout＝HEAD 含刀㊺，兄弟席字节保留） | 2235P/32F/19T | vs k46-tt-post F/T **SET-IDENTICAL**（兄弟席域隔离实证） |
| TT post（重放 md5 断言过） | **2237P/30F/19T** | **post − base＝本刀效应 +2P/−2F**：GONE 恰＝两面目标；F NEW=0；T IDENTICAL（19 恒） |

判读行提取字节级解析（无 grep 路径）；四腿 rc=2 均为既有基线失败集（rc 只留痕）。

## 目标页逐面对账（k47-post vs k46-post base）

- report-only 页三子测全 Pass（「evaluating a string」F→P；「evaluating a Trusted Script」「default policy runs in report-only mode」保 P＝RO 臂对 TS 臂与 default-policy 臂零扰动）。
- sink-mismatch 页三子测全 Pass（eval throw F→P；两 setTimeout 子测保 P＝`.sub.headers` 注入对 sink 路径零扰动——setTimeout RO 臂走既有 `_zwTtRoSink` 管道）。
- create-policy 页两 P 保（`.sub.headers` 注入其 RO trusted-types 名单 meta：RO 政策不触 enforced 名单域＝createPolicy 行为零变化，审计预判兑现）。

## 结论

刀㊼ 净增 **+2**（S1 报告边两面收敛），TT 电池域套件 F **32→30**，Timeout 19 恒，**累计 84,272→84,274（距 ≥99,947 差 15,673）＝本刀 +2；84,270→84,272＝刀㊻ 兄弟席（码刀 `b3dca53f0`＋判读 `e00280515` 先落，本刀基线域已含）**。池余：census 口径 31→29F＝module 16F（R1 一体化刀位单独立项，gate 位置定谳已落 4ab38ae8d）＋散布余 S1 第三面（直接 eval 间接化族挂账）＋S3 2F（页级 T 耦合先解）＋S4 5F＋兄弟席 navigate 5F（其轮收账）。
