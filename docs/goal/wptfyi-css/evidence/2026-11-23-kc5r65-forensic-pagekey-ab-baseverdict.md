# KC-5-r6.5（w5）face 流 forensic——页键复合键判读＋r5-b1 二进制 A/B＋HEAD cssa 无门定谳＋基座裁定（2026-11-23 07:4x）

r6.5（输出侧门 face，patch md5 `12c824e8592377e18473ef22dd7eeec3`，13 文件，rebase 至 HEAD b3c452bee）
28 腿 solo（06:58–07:05）verdict-red 之后，本流（r65 实现流）追加三轮取证并裁定 r66 基座选择。
与 verify 流 07:1x 判读笔（master.md Active 行、commit 152289e17/946339250）同账互补，不覆盖。

## 1. solo 三门账（页键复合键口径）

判读源＝transcript 键级（`Pass|Fail|Timeout <全路径> :: <subtest>`），键＝`(basename(page), subtest 全名)`
复合键——GT 文件第 2 列是 basename 而 transcript 是全路径，纯名字键存在 588–732 跨页碰撞（本流
曾因键口径给出假 RESIDUAL=0，探针单键对质 b0 Pass/b1 Fail 确证后修正）。

| 门 | 页键口径（本流） | 名字键口径（verify 流 07:1x） |
|---|---|---|
| P2F（fresh b0P→b1F） | 269 | — |
| RESIDUAL-P2F（∩GT 110） | **16** | 16（一致） |
| REG 名义 | — | 184 |
| F2P-BREAK（r5-b1P→fresh-b1F ∩ r5F2P 集） | 455 | 551 |
| F2P（fresh b0F→b1P） | 203 | — |
| r5-gains-kept | 170/625 | — |

口径差＝跨页同名键碰撞；判据结论同向：**三门全红，电池不授权落库**。GT 110 键 fresh-b0P
核对 GT-missing=0（b0 键级全覆盖）。残 16 三簇与 07:1x 笔一致（14× width KW→cs(KW-basis)
no-interp 对＋1× interpolate-size numeric-only consult 泄漏＋1× WA cis neutral 键）。

## 2. r5-b1 二进制 A/B——基座裁定直接证据

r5 电池原版 b1 二进制（`results-kc5r5-fame2b/bin/b1-runner`，01:24 代，r5 face `9b4d528b` 驻树期
构建）单页复跑 `css/css-sizing/animation/min-width-interpolation.html`（JSON_DIR 键级 366 键），
对 r65 fresh b1（`probe-bin/b1-runner` md5 c7269790）同页（366 键）：

- **LOST（r5P→r65F）＝76**：r65 输出侧门把 face 栈本可产的正确值挡死。got census＝
  42× `auto` ＋13× `fit-content` ＋9× `min - content` ＋8× `max - content`（72 KW 字面 echo）
  ＋4× `0px`（cssa canonicalize 折叠）——全部是 r5 代的正确产出。
- GAINED（r5F→r65P）＝4；双 fail 35。
- r5 页 P=327 vs r65 页 P=255（差 72，与 LOST 76 减 GAINED 4 恰等）。

**裁定＝甲（r5 face 原基重建）**。verify 流 07:2x 预测表（RESOLVED 100/RESIDUAL 10/REG 0/
F2P-BREAK 153）的成立前提「r66＝r5 基重建＋门控」与本证据一致。

## 3. HEAD cssa 无门定谳（推翻 r6.x 设计前提之一）

`git show HEAD:crates/engine/src/js_dom_bridge/cssa_registry.rs` query_core 直读：

```rust
for p in clock.tick(clock_id, now) {
    if p.name != prop { continue; }
    if let Some(v) = canonicalize(prop, &p.value, style, None) { return Some(v); }
}
None
```

无任何 `parse_lp_terms` 门——该谓词是 face `animation.rs` fame2b 刀新增（HEAD 无此符号）。
r6.1–r6.4 设计笔记中「HEAD cssa 咨询门＝parse_lp_terms 双端 LP」前提失真。由此重定性：
b0 的 cssa 对**全部对**走 canonicalize 直通；fresh-b0F 键＝HEAD canonicalize None
（HEAD applier 拒收该值形）→ underlying。r6.x「抑制复现 b0」只对「b0 本来就 None」的键成立。

## 4. 残 16 的 got 200px 归因修正（勘 07:1x「K-fold 伪影」）

K-fold（`fold_consult_for_node`）需 `consult_raw` Some 才可达（callbacks.rs fold-or-drop 链：
consult_raw None 直接跳过 K-fold 落 echo 链）。r65 三 registry 输出侧门全关后 consult_raw=None，
K-fold 不可达。200px 的双代双源：

- **r5 代**：cssa 无门直通 canonicalize，face applier/serializer 对
  `calc-size(fit-content, size)` 做 used 值解析 → 在 .target 上下文（block auto 填满
  parent 200px，见 `css-values/calc-size/animation/calc-size-width-interpolation.html`
  样表）产出 200px。
- **r65 代**：三门 None 后落 `used_size_echo`（callbacks.rs S1，独立于 consult 的 width 通道）
  → 同样 200px。
- **b0**：HEAD applier 接受 calc-size 串、HEAD serializer 折叠不支持形 → canonicalize 产
  `'0px'` → cssa hit 先于 echo 返回——这是 exp 折叠 `'0px'` 的来源。

**推论：抑制路线（verify 流处方② KW↔cs(KW-basis) 门）达不到 exp '0px'**——cssa None 后
S1 仍 200px；连 S1 一起抑制则落静态 `'auto'`，仍非 0px。残 14 键簇的唯一收敛路径是
**通道对称化**（见 §5），不是更细的抑制谓词。

## 5. GT-110 根机制重定性＋r7 处方

expected＝harness 参考元素的 **cascade 静态 gCS 折叠**；consult/echo 产出＝**applier apply
后串化**。双通道对纯 KW 字面（`auto`/`stretch`）同形（＝r5 增益引擎，76 键实证）；对含解析/
折叠语义的值形（min-content/max-content/calc-size）分裂：apply-往返产 used 解析形（200px）
或 KW 字面（`min - content`），cascade 折叠产 px 或 `0px`。GT census 双簇：
77 键 exp=LP got=LP（上下文不对称：target used vs reference 折叠）＋33 键 exp=LP got=KW
（字面 echo vs px 折叠）——同一根因两面。

**r7 核心刀＝通道对称化**：canonicalize（或 consult 出口）产出收敛到 cascade 静态折叠形，
或 applier 的 calc-size used 解析臂回退。非「更细抑制谓词」（verify 流 r66 处方①②③⑤对
残 14 簇不够：②达不到 0px，①③针对的 REG 面在甲案基下另算）。

KW↔LP 混合对 t<0.5 expected=from 字面（'auto'，76 LOST 键实证）＝甲案「翻转形保留」直接
证据；verify 流处方①③的 REG 归因（r65 宽抑制扫落 r5 翻转赢家）与甲案方向一致。

## 6. r66/r7 执行序（face 流）

1. r5 patch（`9b4d528b`，12 文件，**不含 cssa_registry.rs**）对现 HEAD `apply -3` 重贴；
   cssa_registry.rs 回 HEAD 原样不贴。
2. 双臂 solo 复验（判据同三门：RESIDUAL-P2F=0 ∧ REG=0 ∧ F2P-BREAK=0；键级口径
   (basename, subtest) 复合键）。
3. 通道对称化刀按 solo 残面定靶（预期残面＝GT-110 中含解析语义值形的子集）。

r65 patch `12c824e8` 与电池 script `knife-kc5r65-fame2b-battery.sh` 留档未发射
（solo 三门未过，RESULTS-NOT-FRESH 门未解除）。

## 7. 本地工件（.acceptance/wptfyi-css/，不入库 provenance）

- `kc5r65-pagekey-judge.py`＋`results-kc5r65-fame2b/pagekey-ledger/`：
  `keys-five-lists.json`（p2f/gres/fbreak/f2p/lost 五清单）、`p2f-269.txt`、
  `r5-gains-lost-455.txt`、`r5b1-vs-r65b1-min-width-ab.txt`（76/4/35 逐键＋got）、
  `diag/`（r5b1 与 r65b1 四页 JSON_DIR 键级）。
- 判读脚本修正记录：walrus 清理、tag() 前置、`gt & set(p2f)`、basename 归一复合键。
