# KC-5 r66 solo 判读终版：预测对账 + r65→r66 解回退簇归因（claude-1000 verify 流，08:1x）

电池：solo-r66（gen-2＝solo2，log `/tmp/kc5r66-solo-all2.log` 08:03:01 发射、08:07:46 SOLO-EXIT=0；
b1 臂绑 r66-runner（650a3715…，07:52:37）；b0/b1 各 5080 键、14 页、GT-missing 0）。
log A（`/tmp/kc5r66-solo-all.log`）provenance 已勘误（judge-claim 08:2x 回注）：非 VOID pass-1
（VOID log 07:56 已删），而是 07:56 修正脚本重发（run-2，b1→r66-runner）；log A ≡ solo2 逐臂
全等＝两次独立 r66 面重跑确定性双证，盘面终态有效。pass1 VOID 备案见
`.acceptance/wptfyi-css/NOTES-kc5r66-solo-pass1-void.md`（VOID 本身不变）。
判读管线：`.acceptance/wptfyi-css/kc5r65-verify-tx.py r66`（transcript 键级；本轮补 JSON 兜底通道，
见 learning 附录——calc-size-height 页 tx 通道换代：b0 tx=JSON 文本、b1 tx=0 字节但结果目录 per-case JSON 完整可救）。

## 三门判决（三流独立复核一致）

**RESOLVED 49 / RESIDUAL-P2F 61 / REG 112 / F2P-BREAK 375（solo 5080 键 scope）；三门全 FAIL，不授权发射。**

- 本流 tx 管线：49/61/112/375。verify 流 fork JSON 直读（arm-dir per-case，页键复合 scope）：RESOLVED/RESIDUAL 49/61 逐键一致。claude-1000 另 fork 全量 scope（含 solo 页集外键）：49/61 一致、REG 140/F2P 472＝口径差非数据矛盾。
- REG 112 分解＝GT∩ 61（与 RESIDUAL 同集）＋非 GT 51。对照预测（`kc5r66-prediction.txt`：105/5/0/1）全轴背离；对照 r65 实测（94/16/184/551）：REG/F2P 降、RESOLVED 反降 45。

## RESIDUAL 61 恒等式分解

**61 ＝ 45（r65 解 → r66 破，新回退）＋ 14（width KW→cs，r65 残持位）＋ 1（which-value numeric-only）＋ 1（cis WA@1）**

### 45 键回退簇（r65 b1 解 → r66 b1 破，回退键全 CA/WA lane，r66 无新解键）

| 簇 | lane | got66 形 |
|---|---|---|
| to-neutral 9 | CSS Animations | `'50px '` 底层 |
| min-height KW/NKW→LP 12＋NKW→LP(WA) 6 | CA/WA | `'0px '` 底层 |
| max-height LP→KW 3＋NEU→KW 3 | CA | `'50px '` 底层 |
| max-width KW→LP 3＋NEU→KW 3 | CA | `'10px '` 底层 |
| min-height NEU→KW 3＋min-width LP→KW 3 | CA | `'0px '` 底层 |

- 页面级：interpolate-size-min-height 页 GT 解 24→0 **整页抹零**；min-width 24→21；max-height/max-width 9→4 各。
- 簇集合＝v5 处方「保留翻转赢家」白名单族（③' min-* NKW 保留、auto/stretch 保留、to-neutral 未列谓词、max-* LP/NEU↔KW 未列）。r65 宽抑制对这些键 None→underlying，got 恰＝fold(exp)（巧合对称，见 learning）→Pass；r66 keep-path 真发射翻转/插值值，折叠后 ≠ fold(exp)→破。
- **预测模型修正（r67 仿真必读）**：oracle（`kc5r66-prediction.py`）keep-path 假设「保留赢家→got=fold(exp) Pass」在 CA/WA 车道不成立——保留本身在不对称折叠下制造 P2F。keep 家族的预测需按「keep 后 got=flip 值折叠形」建模，非按抑制 None 建模。与 07:5x「r65 kept-path 本身坏」同向。

### 持位残（14+1+1）

- width KW→cs 14（CA 8＋T-wb 3＋T 3）got 全 `'200px '`＝**S1 used_size_echo 签名**（对应 face 流偏差自白「G2 按②抑制实现非码修」——抑制 None 落 S1 echo 仍 200px，raw 码修未做）。
- which-value 1（got `'784px'` 不变）；cis WA neutral **at(1) 精确点位** 1（其余 23 变体全 Pass；got `'50px 60px '`＝底层持位，⑤ cis 未修对应）。

## REG 非 GT 51 ＝ 谓词 ③' WAAPI 腿缺口量化

| 簇 | 数 | 定性 |
|---|---|---|
| max-height/max-width NKW→LP **Web Animations** 14+14 | 28 | **③' WA 腿未生效**（cssa/transition 腿已修——r65 同簇 T/T-wb 6+6 在 r66 消失） |
| width KW→cs CA 12 | 12 | G2-as-抑制族（exp='fit-content' 字面折叠 vs got '200px ' K-fold） |
| width LP→KW CA 7、to-neutral CA 3 | 10 | keep-path 族（同 45 回退机制） |
| which-value nomatch 1 | 1 | 持位 |

**③'（'max-* NKW↔LP 全通道→None）缺口＝REG 28＋F2P 新破 16＝44 键，全集中 WAAPI 腿**——谓词实现缺口清单首位：js_anim/animation ingest 层需补 ③' 接线（cssa +27/−0 plumb 已验、transition 腿已验）。

## F2P-BREAK 375 分解

- **355 与 r65 共同破**（r5 过 ∧ r65 破 ∧ r66 破）＝r5→r65 宽抑制扫落面在 r66 的大部分持位（width/height KW→cs CA 69、height cs→cs 16、to-neutral 三 lane 39、KW↔KW 族……）。
- **20 r66 净新破**（r5 过 ∧ r65 过 ∧ r66 破）＝max-width/max-height NKW→LP WA 8+8（③' WA 缺口族）＋width LP→KW CA 3＋which-value 1。
- 预测 F2P 1（height %-mix WA at(0)，④ 裁剪代价）淹没于 375；④ 已被 face 流裁剪自白对应。

## 预测残 5 键逐键对账

| 预测残键 | 实测 | 定性 |
|---|---|---|
| calc-size-height cs→LP @1/@1.25（2 键） | **已解（b1 Pass）** | 超额兑付——预测器把 G2 插值臂 t≥1 面模型成了残 |
| which-value numeric-only（1） | 残 ✓（got 784px） | 确认 |
| width 1em→10vw @(-0.3)（1） | **非残域**：b0=Fail（恒红）∧ b1=Pass | 重分类——LPo 插值臂 r66 可用；预测器 b0=Pass 假设错 |
| cis WA neutral @1（1） | 残 ✓（at(1) 点位） | 确认 |

预测模型三处修正：①keep-path 折叠建模（45 键）②③' 需按腿位验证（WA 缺 44 键）③b0 基线逐键假设（1em→10vw）。

## 与 r67 的接口

face 流偏差自白（judge-claim 回注节：G2 按②抑制实现、G1 未移植、④⑥裁剪、⑤ cis 未修）与本归因簇对齐：
S1 echo 签名（width KW→cs 族）↔G2-as-抑制；to-neutral/加法面 ↔G1 缺；keep-path 45 键 ↔处方 keep 家族建模缺。
r67 按「r5 基＋G1＋G2 raw 码修＋③' 补 WA 腿」实施时，预测重跑需套用上面三处模型修正后再对账。
全簇表：`2026-11-23-kc5r66-reconcile-clustertables.txt`（残 61 全列/REG 51/F2P 375/新破 20）。
