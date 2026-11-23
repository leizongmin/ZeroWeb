# kc5r75 solo 判读收账——刀F② 单刀净改善轮（REG=0，净账 2574F 新最优工作基）

日期：2026-11-24。前序：[r74 REDLINE fail-stop](2026-11-24-kc5r74-solo-verdict-redline.md)
（commit e6fe96f76）。

## 0. 结论

三臂 solo 电池（css-sizing 15 页＋css-values 18 页 × b0/b1r5/b1）05:02:26→05:09:49
全绿（SOLO_EXIT=0，PAGE-CENSUS 15+18×3 齐，PROBE-PAGE-IN-CORPUS 零命中）。判读：

| 门 | 预注册 | 实测（full / win14） | 判 |
|---|---|---|---|
| GT 再生带 | [84,94] / [20,24] | 89 / 22 | ✓ |
| RESOLVED | 2 / 2 | **4 / 4** | ✗ +2（预注册归因勘误，见 §2） |
| RESIDUAL | 87 / 20 | **85 / 18** | ✗ −2（同上对偶） |
| 非 GT REG | 0 | **0 / 0** | ✓ |
| 非 GT F2P | 0 | 14 / 14 | ✗ 门参照系勘误（本刀新翻 0，见 §3） |
| css-sizing 页级 Δ | −2±2 | **−2**（449→447） | ✓ |
| 净账 | ≈2576F | **2574F** | ✓ ±2 带（−4＝RESOLVED +2 对偶） |
| 恒等式 | 双 ✓ | 89=4+85+0 / 22=4+18+0 | ✓ |
| GT-missing | 0 | 0 | ✓ |

**REDLINE：REG=0 达标；P2F 85>0 → face 不落库（r73/r74 先例），驻留 r76 工作基。**
净账 2574F＝**新历史最优工作基**（轨迹 r71 2647→r72 2703→r73 2578→r75 2574）；
vs r73 工作基增量＝**4 改善 0 恶化**（首个零回归净改善轮）。

## 1. provenance

- face 13 文件快照 prebuild 05:02＝电池发射现印逐字节同
  （`face-md5-prebuild-0502.txt`）；build 43s RC=0（05:01 systemd scope 24G）。
- bins：b0 c745bee4（HEAD face）／b1r5 d2ef864b（R5 face，复用 kc5r73）／
  b1(r75) 253ed910（r73 工作基＋刀F②）。
- 刀 hunks：animation.rs 离散臂 height-% guard 臂＋`is_bare_percentage_literal`
  helper＋apply_single_property 空串契约门＋钉测 2（test-guard 包裹定向跑 2 passed
  实数核对）。

## 2. RESOLVED 4 键级与预注册勘误

- height-composition 2 键（`underlying [10%] from add [100px] to add [20%]` 离散
  % 端点）＝预注册内。
- calc-size-height-interpolation 2 键（`from [calc-size(auto, size * 2)] to [50%]`
  t≥0.5 采样）＝**预注册漏计**：r74 联合 face 时这 2 键已 RESOLVED，r74 verdict
  §3 把它们归刀E（RESOLVED 10＝width 8＋height 2）系归因笔误——它们走的是刀F②
  通道（to=50% 裸 % → 不写 → 静态折叠两错相消），本轮 F② 单刀同代码同键集命中
  即归因实证。预注册「RESOLVED 2」按错误归因推演，实际刀域命中面为 4。

## 3. F2P 14 键级＝门参照系勘误（非行为偏离）

- 14 键（width-interpolation 6＋height-interpolation 8）**全部**在 r73 face 下已
  pass（继承存量＝刀A「css-sizing −14」同族；14/14 零例外）；本刀新翻＝0。
- 预注册「F2P=0」误以 b1r5 为参照（b1r5F∧b1P 口径天然含 r73 已入账增益）；
  正确门＝**vs r73 工作基增量 F2P=0**——实测增量 4 改善 0 恶化达标。下轮预注册
  统一用 vs-工作基增量口径记 F2P。

## 4. 对 r73 工作基增量分解（键级全列）

4 键 fail→pass，零反向：height-composition 2＋calc-size-height 2；css-sizing
Δ=−2（449→447）、css-values Δ=−2（2129→2127）。

## 5. 剩余 GT 85 残键构成（刀域判定）

- **min/max 66**＝刀F① 判死域（r74 判定：离散 echo 是规格正确通道；66 键真通道
  ＝interpolate-size 关键字插值实现——大件，spec 级 calc-size 端点对插值）。
- **width 14**＝r74 判定引擎 tick 可观测空间与 no-interpolation 参考通道重叠，
  零伤害切法不存在——挂账。
- composition 3＋PX 1（width `1em→10vw` 负 t 钳）＋which-value 1＝域外残键。

## 6. r76 决策点（处方）

KC-5 动画刀轮 r65→r75 共 11 轮，本刀域（sizing 动画 tick/apply）可安全入账面已
榨干（余键全需大件或不可达信号）。候选：

1. **KC-5 收口**：r75 工作基（r73 face＋刀F②，净 2574F）定档；刀F② face 保持
   驻留不落库（P2F 85 fail-stop 条款持续生效）；KC-5 总结归档（11 轮轨迹
   RESOLVED 16→20→0→0→2→0→20→0→89→0→4，净账 2703→2574）。
2. **interpolate-size 关键字插值大件立项**（min/max 66 键）：探针先行
   （allow-keywords 页面端点对可插值性＋参考折叠通道定谳）→ 评估工程量后决定。
3. **转域**：CSS 其余兼容性域（按 wptfyi_gap coverage 管道选下一个高收益域）。

默认走 1＋3（KC-5 收口＋选域评估）；2 是否立项留 r76 探针定谳后决定。
