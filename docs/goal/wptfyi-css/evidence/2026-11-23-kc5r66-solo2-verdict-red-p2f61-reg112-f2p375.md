# KC-5-r66 solo2 判读 verdict-red（verify 流，2026-11-23 08:1x）

## 判定：三门全红，r66 face（knife-kc5r66-fame2b-face.patch md5 `d59c3594`）不落库

- **RESOLVED 49 / RESIDUAL-P2F 61 / REG 112 / F2P-BREAK 375，GT-missing 0**（5080 键两侧齐，14 页）。
- 对 v5 预测（105/5/0/1，`kc5r66-prediction.py`）三面全失准：RESIDUAL 61 vs 5、REG 112 vs 0、F2P-BREAK 375 vs 1。
- 判读工具 `kc5r65-verify-tx.py r66`（批次参数化），全量输出 `.acceptance/wptfyi-css/kc5r66-verdict.txt`（face 流独立判读与 verify 流 judge 输出逐簇同值互证）。
- face 流已按 fail-stop 协议剥除 r66 增量 hunks（08:11:33 批量 mtime，`r66_pair_suppress` 全树负检 =0），r5 基 face 保持驻树作 r7 基座；r66 patch 全文留档。

## 双臂 provenance

- b0 臂 = `probe-bin/b0-runner` md5 `16ce5984`（与 GT 面板/r5 电池 b0 同 md5，基线可比）；b1 臂 = `probe-bin/r66-runner` md5 `650a3715`（07:52 构建，face 文件 mtime 最新 07:51:31 早于构建时刻，无 stale-bin）。
- 28/28 腿 rc=0，08:03:xx 发射→08:08:31 收官（`/tmp/kc5r66-solo-all2.log`）。

## pass1/runA 事故与 provenance 勘误（判读样本史，不入账）

- 首发 solo（07:53，pass1）脚本模板 `$ARM-runner` 使 b1 臂命中 `b1-runner`＝r65 代 bin（contain b1 415P/17F 与 solo-r65 同页逐位全等暴露），其日志与 tx 已于 07:56 删除；脚本修正后 face 流重发＝log A（`/tmp/kc5r66-solo-all.log`，08:01:46 DONE，b1 臂绑定 r66-runner 650a3715，有效）。
- 勘误（face 流 08:2x 回注，本流认可入账）：log A 曾被误记为 pass1 VOID——实测 contain b1=431P/1F ≠ VOID 签名 415P/17F；**log A ≡ solo2 逐臂全等＝同一 r66 面两次独立重跑的确定性双证**。判读输入（solo2 现盘）不变，数据有效性反而更强。
- 交接窗瞬态读数（width-b1 853P/335F 单次，quiet 双证恒 922P/266F）存档不归因（并发 runner 负载敏感形，可能性存疑）。
- 教训：①同名 `*-runner` 模板在多代 bin 并存时是双臂 A/B 的结构性陷阱——发射前静态核脚本绑定，首页 fail 计数与已知代际签名全等即 VOID 信号；②脚本修正对在跑进程无效（bash 函数已解析）——改脚本必须重发射；③judge 输入先与代际签名核对再判读。

## transcript 形态插曲（判读工具链）

- corpus 脚本在 `CSS_CORPUS_JSON_DIR` 设置时 transcript 落 **JSON 块体**（`==== <rel>` + runner `--json` 数组；stderr 丢弃保 JSON 可解析），未设置时落 `Pass/Fail <rel> :: <name>` 文本行。r66 solo 两者兼有（JSON 形态），judge（按 r65 文本形态写）初跑 0 键＝空洞假绿，转换后方可判读。
- 转换器 `/tmp/kc5r66-tx-convert2.py`（非破坏版：已文本跳过、解析失败不改写；raw JSON 先存 `tx-json-r66/`）。转换后 28 文件 Pass/Fail 计数与 RUN 行逐一核对无缺臂。
- 教训：judge 输入非空/形态校验前置——0 键不是绿灯，先核 transcript 形态与块体完整性再出三门。

## 三门失准三分类（CLAIM 07:6x 协议）

1. **cssa 门残留（残留主簇）**：RESIDUAL 61 中 ~59 键 got 全为静态/echo 形（`'50px '`/`'200px '`/`'0px '`/`'10px '`，带尾空格）而 exp 为复合 calc-size 串（`'calc-size(min-content, 0px + size * 1)'` 等）。GT 键的可过性 = b0 HEAD cssa query_core 无门直通 canonicalize 的复合串透传（exp 恰与该串全等）；r66 保留 r5 代 canonicalize 门/used 解析即破坏透传。＝GT-110 根机制（apply-往返 vs cascade 静态折叠通道不对称）在 r66 的重演，**谓词抑制对这些键结构性无效**（抑制→None→级联→静态折叠形 ≠ 复合串 exp）。
2. **谓词实现缺口（REG 主簇）**：28× max-*:NKW->LP WA（exp=`'initial'` got=`'10px '`）＝③' 辖区——若 ③' 命中且抑制恢复 b0 parity（b0 got=`'initial'`）应全过；实测产出 LP 静态形。与 CLAIM 07:3x 实现注意（E5 ingest 解析后对级门看不见 NKW）吻合：NKW 可见性 plumb 未达，③' 实际未拦。
3. **G2 域 + 仿真模型错（F2P-BREAK 主簇）**：375 全落在 r5 赢面形（KW->cs 69、to-neutral 39、KW->KW 39、cs->cs 24、NKW->LP WA 28）＝模型假设「不抑制→r5 parity、抑制→b0 parity」双分支皆被实测推翻：抑制分支落级联形（≠exp），非抑制分支被 r5 cssa 门改值。仿真 oracle（全键静态推演）在多层 fallback 链下再次连环失准（a72d71657「抑制非万能」同根，教训三演）。
- 残池另核：which-value 数字模式 1 键（got=`'784px'`，真门按对级判未覆盖）＋WA cis neutral 1 键（`'50px 60px '`，cis 刀域）——与 r65 残 16 中该两键同位持续。

## 对照池（GT 三门轨迹）

| 轮 | RESOLVED | RESIDUAL | REG | F2P-BREAK |
|---|---|---|---|---|
| r6.3 (w3) | 93 | 17 | 211 | 573 |
| r6.4 (w4) | 49 | 61 | 73 | 332 |
| r65 | 94 | 16 | 184 | 551 |
| **r66** | **49** | **61** | **112** | **375** |
| v5 预测 | 105 | 5 | 0 | 1 |

- r66 面貌与 r6.4 几乎同构（RESIDUAL 61 同值）——谓词抑制把 r5 基的 GT 轴打到与 r6.4 相同的残面，且 REG/F2P 未达 w4 水位。GT 轴突破轮（r65 的 94/16）靠的是输出侧门+抑制的组合，但其 REG/F2P 轴未达零；**单刀同时压三门的组合未出现**。

## r7 方向（与 a1a2d962c 处方一致，本轮实测加固）

- cssa_registry.rs 回 HEAD 原样不贴（恢复复合串透传＝GT 面可过性），通道对称化按残面定靶；谓词刀在「逐键 b0 双机制（consult 命中 vs None 级联）定谳」之前不再整簇发射——本轮 375 F2P-BREAK 是该教训的第三次实证。
- NKW 对级门实现若复用，须先解 E5 ingest 可见性（时钟态保 raw 或门置于 keyframe 解析层）。

## 工件索引

- 判读输出：`.acceptance/wptfyi-css/kc5r66-verdict.txt`（face 流）/ judge 互证同值（verify 流，本 doc 数字源）
- 判读工具：`.acceptance/wptfyi-css/kc5r65-verify-tx.py`（`r66` 参数）；转换器 `/tmp/kc5r66-tx-convert2.py`
- transcript raw：`.acceptance/wptfyi-css/results-kc5r6-fame2b/tx-json-r66/`（run2）；pass1 VOID 样本 `tx-json-r66-pass1-VOID/`
- 仲裁：`NOTES-kc5r66-solo-pass1-void.md`、`NOTES-kc5r66-solo2-judge-claim.md`
- face patch：`.acceptance/wptfyi-css/knife-kc5r66-fame2b-face.patch`（md5 `d59c3594036f48d0e0c98af249a1a086`，13 文件）
