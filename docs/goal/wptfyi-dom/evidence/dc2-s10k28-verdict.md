# DC-2 切片 10 刀㉗收账——TT 违规 sample 形（sink 名前缀＋40 单元截断）＋createPolicy 名单/重名 block 违规派发（TT P 2044→2070＝+26＝GONE 27／NEW 0／Timeout 键集恒 20=20）

**日期** 2026-11-26　**判定** PASS（判读记录腿＝组合树 GONE 27／NEW 0；首轮 solo 腿 GONE 27/NEW 1 与 Timeout +1 两 delta 均为 gitignored 探针页噪声〔组合树腿复跑即消〕，语料回归 0）

## 提交与判读绑定

| 提交 | 内容 | 判读域 |
|---|---|---|
| 刀体 `a09bdd83c`（5 文件 206+/22-：part14 sample 形/派发签名/政策归属＋part12 block 违规派发＋part175 断言随新形＋part176 新钉两测＋registry 登记） | sample 裁剪形＋createPolicy block 面 | TT 全腿 solo vs v12 键集 |
| 前置 `d10adbfcd`（1 文件：style-system custom_function.rs `let_and_return`） | k131 携入 clippy 红点解堵（署名 9ef9851b4 域，gen27 撤码复验未复跑 clippy 遗留） | style-system 2731P/0F 复证零语义 |

谱系：`8aa750cf4` → pull --rebase --autostash 携入 css 流 k131 双提交（`9ef9851b4` code＋`a3d2f3be8` docs，stash 7→7，四脏文件 marker 逐项核在）→ `d10adbfcd` → `a09bdd83c`。记录腿发射器 `/tmp/s10k28/launch-combined.sh`（HEAD 断言 a09bdd83c，setsid detached），20:33:21–20:42:29Z（9min，含 release 重建——组合树二进制含 k131 part04/05/pipeline 全量）。

## 根因链（三段）

1. **sample 形**（part14）：spec TT §create-a-trusted-type-violation-record——sink 面 sample＝「sink group 名|前 40 UTF-16 码元」，尾单元（第 40）落 lone 高代理（D800–DBFF）时延至 41 保代理对完整（Firefox 口径，w3c/webappsec-csp#704）；`_zwTtClipSample` 实现＋`_zwTtViolation` 扩五参（sinkName/sample/originalPolicy/disposition/directive），sink 名照引擎既有 sinkName 实参（'Element innerHTML'/'HTMLScriptElement src' 等＝spec sink group 列同形），createPolicy block 面传空 sinkName＝无前缀。
2. **政策归属**（part14）：k27 的 enforcePolicy「首见非 RO 政策」口径在多 meta 页把名单/连接政策错标为执法政策（clipping 页三 meta：`trusted-types 'none'`＋`require-trusted-types-for 'script'`＋`connect-src 'none'`——原口径 originalPolicy 锚到名单政策）→改在解析分支内按指令归属记录（`enforcePolicy` 于 require-trusted-types-for 分支、`listPolicy` 于 trusted-types 分支，均取首见政策原文）。
3. **block 面派发**（part12）：createPolicy 名单 block（gate 拒）与重名 block（`_zwTtCreatedNames` 命中且无 allow-duplicates）两面此前只抛 TypeError 不派发违规；语料 cspTests(-none) 与 clipping createPolicy 面均断言违规事件（directive 'trusted-types'＋originalPolicy.includes/=名单政策原文＋sample=名截断无前缀＋disposition 'enforce'）→两面统一先派发后抛（Chrome 对两 block 面同发违规实证）。

## 判读（锚＝v12 终态 TT F 206/P 2044/Timeout 20；基线键集 /tmp/s10k28/baseline-allF.keys）

| 项 | solo 腿（预 rebase 二进制） | 组合树记录腿（a09bdd83c 全量） |
|---|---|---|
| GONE | 27 | **27**（clip 20＝正 12+tent 8；cspTests 2＋cspTests-none 2；svg-script-set-href 1；source-file-path 1；zz-knife27-probe 1〔gitignored〕） |
| NEW | 1（zz-s10k27-ro-probe 失败＝探针噪声） | **0** |
| Fail | 206→180 | **206→179** |
| Pass | 2044→2068（+24） | **2044→2070（+26）** |
| Timeout | 20→21（+1＝ro-probe2 探针） | **20→20 键集恒（comm 0/0）** |

clipping 残留 F＝0（20/20 目标簇全清）。cspTests 双页 4 键全清（Non-allowed＋Duplicate＋none 页两名）。svg-script-set-href 残 7 键／source-file-path 残余／navigate-001/003/004/008 5 键／eval-reporting 3 键为后续簇（需各面独立派发通道，sample 形只是其断言面之一）。

## 钉测与语料

- **part176_s10k28_sample_clip_pins**（新文件，两测 6 面）：`test_s10k28_sample_clip_pins`＝三 meta 执法页 sink 面截断（41 单元→40／40 不裁／42 单元 surrogate→40）＋createPolicy 名单 block（dir 'trusted-types'＋op=名单原文＋sample 无前缀＋disp enforce）；`test_s10k28_policy_block_pins`＝名单页（trusted-types DupName）重名 block＋名单外 block 两面违规字段序。
- **part175 五钉**断言随新形改带 sink 前缀（'Element innerHTML|abc'／'HTMLScriptElement src|support/namespaces.js#abc' 等），头注记 k28 形变更。
- 组合树钉测：s10k2x 全套 20/20＋kc5r131 4/4（k131 域交叉核）；engine lib 全量 **4323P/3F**（3F＝对端预存红名单 dialog_focus_s330/expando_r3046/p99s_inline_custom_property 恒定；总数 4326＝4320+2+4 对账合）。
- 语料 solo：clipping 正页 **12/12**（RC=0）＋tentative 页 **8/8**（RC=0，surrogate 四边界案全过：𝐆×20 40 不裁／H+𝐇×20 41 延至 41／𝐈×20+I 41→40／𝐉×21 42→40）。

## 门禁

fmt（engine+style-system --check）绿；guarded clippy `-p zero-engine --all-targets -D warnings` 绿（k131 红点修复后）；style-system lib 2731P/0F。跑批纪律：part12 二次编辑曾触发在飞 release 混合二进制风险→杀链重跑（正页 12/12 结果出自杀链前完好二进制，该页与重名面无关；记录腿为提交后组合树重建）。

## 累计与缺口

TT P 2044→2070＝**累计 83,910→83,936（距 ≥99,947 差 16,011）**。TT residual 池：179F＋Timeout 20 页页行（Element-setAttribute 9／svg-script-set-href 残 7／function-constructor+eval 族 11／script-enforcement-008/009 各 5／createHTMLDocument 5／navigate-001/003/004/008 5／eval-reporting 3／其余散布）。
