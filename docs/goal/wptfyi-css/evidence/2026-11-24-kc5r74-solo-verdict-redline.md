# kc5r74 solo 判读收账——REDLINE 触发 fail-stop（REG=522，刀F① 判死，刀E 需判别式）

日期：2026-11-24。前序：[探针定谳](2026-11-24-kc5r74-probe-tick-channel-verdict.md)（commit 6b71f0740）。

## 0. 结论

三臂 solo 电池（css-sizing 15 页＋css-values 18 页 × b0/b1r5/b1）04:31:01→04:38:16 全绿
（SOLO_RC=0，PAGE-CENSUS 15+18×3 齐，PROBE-PAGE-IN-CORPUS 零命中）。判读（judge.py
勘误后口径）对预注册 8 门：

| 门 | 预注册 | 实测（full / win14） | 判 |
|---|---|---|---|
| GT 再生带 | [84,94] / [20,24] | 89 / 22 | ✓ |
| RESOLVED | 82 / 16 | **78 / 12** | ✗ −4 |
| RESIDUAL | 7 / 6 | **11 / 10** | ✗ +4 |
| 非 GT REG | 0 | **522 / 350** | ✗ **REDLINE** |
| 非 GT F2P | 0 | **56 / 122** | ✗ |
| css-sizing 页级 Δ | 0±2 | **+342**（449→791） | ✗ |
| 净账 | ≈2496F | **2980F**（对 r73 工作基 2578F＝+402） | ✗ |
| 恒等式 | 双 ✓ | 89=78+11+0 / 22=12+10+0 | ✓ |
| GT-missing | 0 | 0 | ✓ |

**REDLINE 触发（REG>0）→ face 不落库、刀E/刀F 不转正。** worktree 已复原 r73 工作基
（13 码文件 md5 与 /tmp/zw-r73-face-md5-prejudge.txt 逐字节同，见 §2）。

## 1. provenance

- face 三点冻结：prebuild 04:28:53＝发射印＝postbuild 04:30（13 文件逐字节同，
  `face-md5-prebuild-0428.txt`/`face-md5-postbuild-0430.txt`），电池窗复核同值。
- bins：b0 c745bee4（HEAD face）／b1r5 d2ef864b（R5 face，复用 kc5r73）／
  b1(r74) 14a7ff92（r73 工作基＋刀E＋刀F①＋刀F②；build 1m05s RC=0）。
- 判后处置：本 session 7 对 Edit（`anim-knife-edit-pairs-r74.json` 存档）逆序反演，
  animation.rs md5 复原 **76e1d5c3**＝r73 快照 MATCH；其余 12 文件零触碰核同。
  该字节态即编译出 r73 bin ee308dad 的 provenance 态，免重复编译验证。

## 2. judge.py 勘误（口径修正）

初版 f2p 过滤器误写 `b0F∧b1r5P∧b1F`（也是 P→F 子集），导致打印 REG=242/F2P=280——
两口径实为真 REG 522 的 b0P/b0F 两半。修正为 REG=`b1r5P∧b1F` 全量（按 b0 分半）、
F2P=`b1r5F∧b1P` 且非 GT 后，键级清单与三方交叉表算术互核闭合：
b1 fails 2980 = b1r5 2592 − F2P全 134 + P→F全 522 ✓；pass 4572−522+134=4184 ✓。

波及核验（kc5r73 结果重算真账）：GT=89／RESOLVED=0／RESIDUAL=89／**真 REG=0**——
r73「REG 0」结论在修正口径下成立；真非 GT F2P=14＝刀A「css-sizing −14 恰中」同源
（r73 原始计数账 449/2129/2578 均不受影响，仅标签定义勘误；r73 收账结论不变）。

## 3. 刀分立归因（键级，全闭环）

对 r73 工作基净效应（vs b1r5 口径 +388＝F①+408 / E−18 / F②−2；加 r73 对 R5 的 −14
＝总 +402 ✓）：

- **刀F①（min/max 轴非纯端点不写）：净 +408，判死。** RESOLVED 66／REG 516
  （max-height 152＋max-width 144＋min-width 115＋min-height 105）。逐键证据：
  同形 `from[20px] to[min-content]`，WPT 期望＝from 回声 20px（规格离散翻转正确预期），
  刀F① 不写→underlying 10px 显出→打破规格正确通过。66 RESOLVED 判为**两错相消**：
  `from[150px] to[min-content]` 类的测试声明 underlying≡from（150px），不写→underlying
  显出恰同回声→侥幸命中；无规格正确赢面通道。探针定谳「exp 恒声明值」系只采 failing
  键的以偏概全（r72 勘误同型重演：静态机制推演在多层 fallback 链下连环失准）。
- **刀E（width/height calc-size intrinsic 折叠 0px）：净 −18（+10R/−6REG/+14F2P），
  需判别式后重施。** RESOLVED 10（calc-size-width 8＋calc-size-height 2，exp=0px 通道
  折叠命中）＋非 GT F2P 14（height-interpolation 8＋width-interpolation 6）；REG 6
  全部 calc-size-width **test_no_interpolation 形**（from[calc-size(any,50px)]/裸 KW →
  to[calc-size(intrinsic,·)]，exp=200px 静态回声，折叠 0px 打破 r73 两错相消）。
  判别式数据支撑：折叠应仅在**另一端点为纯数值**（px/%/em/vw）时触发；另一端点亦
  KW/calc-size 形＝真离散→echo 才是规格通道。
- **刀F②（height 轴未解析 % 不写）：净 −2（2 RESOLVED/0 REG），纯净保留候选。**
  height-composition 2 键命中，REG 零。

## 4. RESIDUAL 11 键构成（对预注册 7 的 +4 全归因）

- 预期内 5：composition 3（height-composition，其中 2 键被刀F② 提走故 5→3）＋
  PX 1（width-interpolation `1em→10vw` exp 0px got 16px）＋which-value 1。
- 超预期 6→即 width 族 GT 14 键的另一半：calc-size-width **Transitions lane**
  （allow-discrete）6 键 exp 0px got 200px——刀E 折叠只触达 CSS Animations lane，
  未触达 transitions 离散路径。width 14＝Animations lane 8 折叠命中＋Transitions lane 6 残留。
- 每个预注册桶键级分解闭合：RESOLVED 78＝F① 66＋E 10＋F② 2；RESIDUAL 11＝
  width 6＋composition 3＋PX 1＋which 1；GT 89＝78+11+0。

## 5. 挂账（不变＋新增）

- render_lp_terms 混合形缺 `%` 后缀（face 既有域外勘误，b0 亦 F＝GT 口径外）。
- zero-engine lib 单测 `wc_m1_ce_define_domexception_types`（part28）域外红待全量窗归因。
- judge.py f2p 定义勘误（本档 §2；r73 结论复核不受影响）。

## 6. r75 处方（预注册草案）

1. 基座＝r73 工作基（已复原驻留，13 文件快照核验）。
2. 刀E'：折叠加判别式——picked 为 calc-size(intrinsic,·) 且**另一端点纯数值**才写 0px；
   另一端点 KW/calc-size 形维持 echo（6 REG 键复活回 PASS）。预测 RESOLVED full 8~12。
3. 刀F② 原样重施（+2 纯净）。刀F① 不复活；min/max 真通道＝interpolate-size 关键字
   插值实现（大件另立项，不在 r75）。
4. 探针先行（K-E-DISCRIM）：calc-size-width 6 REG 键 + 8 RESOLVED 键对 r73 bin 重放，
   验证「另一端点纯数值」判别式零重叠；顺核 transition_registry 离散路径是否可达
   sizing 臂（Transitions lane 6 键，不塞 r75 主刀）。
5. solo 电池同协议。预注册：GT 带 [84,94]/[20,24] 不变；RESOLVED full 12±2 / win14
   10~14；RESIDUAL full 9±2（7＋Transitions lane 2?）——以探针实测修边；REG=0；
   F2P=0（刀E'/F② 域外零翻动）；css-sizing Δ=−2~−4（width-interpolation 6 F2P 若复活
   则 −8±2）；净账 2566±4F；恒等式双 ✓。
6. REDLINE 同款：REG>0 或 F2P>0 → fail-stop 逐键归因，face 不落库。
