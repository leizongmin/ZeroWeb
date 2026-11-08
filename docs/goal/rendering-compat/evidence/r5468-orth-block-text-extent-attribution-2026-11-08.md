# R5468 证据档：缺陷 2 正交纯文本臂实现→oracle 几何达标→corpus 净负→整体回退（fsa 度量单 owner 阻塞定谳）

日期：2026-11-08。R5467 下一步①（缺陷 2 归因开案：htb 父＋vertical 子测量 w=0 域，R5456 镜像）。

## 0. 轮次结论

- **臂已实现并几何定谳达标**：R4445 域新增正交纯文本 auto 高臂（depth 单列 INFINITY ＋ 行内 extent fit-content 回写，kill-switch 设计 `ZW_ORTHO_BLOCK_TEXT_EXTENT`），driver ref 臂 cell 内 vertical p **w=0 h=20 → w=20 ch=95.1/106.6**（LAYOUT_DUMP 实测，chrome oracle 20×94.17 / 20×106.61，残差 0.93/0.01px＝shaping 域）——缺陷 2 盒级修复成立。
- **corpus 净负，整体回退**（树回到 `9cd55425d`，零代码残留）：全量 A/B（同二进制，臂 ON 692F / OFF 688F）＋逐案 solo ×3：**真回归 1**（font-size-adjust-ic-height 0.25%→11.60%，solo ×3 恒 11.60%、kill-switch solo ×2 恒 ✓0.25%）；driver vrl-007 **3.55%→3.58%**（solo ×3 双侧采样，像素中性偏负——残余被缺陷 3 的 16/48px 结构列位差支配）；✗→✓ **0 案**。净负 ＋ 准则 6（压制症状不如修共享路径）→ 不合入。
- **根因定谳（fsa 度量单 owner 双侧破缺）**：fsa（font-size-adjust）在 ZW 两条文本度量路径上**各自坏法不同**——
  1. **渲染竖排 IFC 整体忽略 fsa**：ic-height 测试页 `.adjusted`（100px＋ic-height 2.0）旧径 ch=**116**（=100px×line-height 1.16；fsa 生效应为 200px→232）、`.fallback`（ic-height 0.5）ch=**100**（fsa 生效应 50px→58）；
  2. **探针横排 IFC 把 ic-height 按 ex-height 回退解**：fit 度量 144.8 = 0.724（NotoSansCJK ex）× 200（fsa 目标值已生效但 metric 错）——R4469 同案「144.8 vs 200」分歧的本体。
  - 后果：臂 ON 时 ref 臂页（显式 200px、无 fsa）正常进臂（w=232 h=232 ch=200，探针 FSA-REF 实测），测试页被 fsa 域染毒——**self-source 双侧对称破缺**（learning #33：配对型 reftest 只让单侧走新路径必失配，可行方向只有双侧同修到一致）。
  - **fsa 容器 exclusion gate 不可救**（本轮实证）：gate 只排测试侧，ref 侧（无 fsa）照进臂 → w=0-vs-232 反向失配，对称性恢复不了；`text-orientation` 未实现（ComputedStyle 无字段，engine 仅注释提及）亦不可作域闸。唯一收敛路径＝**fsa 度量单 owner 深修**（渲染竖排 IFC 接入 fsa ＋ 探针 metric 解析与 shaper 同源，css-fonts-5 域切片）→ 挂账待开案，本轮不动。

## 1. 新发现：缺陷 4（viv 行内 size fill 域，挂账）

vrl-007 **test 臂**双 p（viv 域：父 div 亦 vertical-rl，不在正交臂域）ZW **h=752 fill**，chrome oracle test/ref 双臂 p **逐位一致 20×94.17**（R5467 oracle 表）——chromium 对**不定 CB**（div height:auto）下的行内 size:auto 竖排块做 **shrink-to-fit**，ZW viv 既有径填满（且 fill 源 752 = 800−16−32 疑走物理宽轴，VIV 探针 ch=752 > 视口高 600，轴源待归因）。本次臂的 parent_wm gate（viv 排除）正确防了此域扩散（VIV-LONG 长文本 w=60 fill 哨兵探针臂 ON/OFF 同值），但 viv 行内 fill 语义本身与 chromium 的偏差是新缺口：行内 size auto＝fill 的横排类比只在 CB inline-size **definite** 时成立，indefinite 时 chromium 落 shrink-to-fit。 blast radius＝全部含 auto 高竖排块页（vrl/vlr 近似通过带全动）→ 深结构挂账，与 R5459/R5460 viv fill 域账本合并管理。

## 2. corpus 批跑噪声指纹家族扩员（＋3）

A/B 新臂 4 apparent 回归 solo ×3 定谳：font-size-adjust-ic-height **真回归**（上）；insert-block-in-blocks-n-inlines-middle-001（batch ✗1.58% / solo ✓0.00% ×3）、table-anonymous-objects-130（✗4.03% / ✓0.00% ×3）、table-anonymous-objects-153（✗1.17% / ✓0.00% ×3）＝**噪声扩员**（R5464/R5466/R5467 同族第四批：movers solo×3 纪律再证）。

## 3. 几何证据（臂 ON 版 LAYOUT_DUMP，已随回退消失，留档）

`LAYOUT_DUMP=1 make reftest-upstream FILTER=sizing-orthog-htb-in-vrl-007`（臂 ON）ref 臂：

```
td [212,264] w=52  → p w=20 h=20 ch=95.1  （oracle 20×94.17 ✓ 残差 0.93px）
td#data [264,664] w=400 ch=120 h=126     （R5467 缺陷 1 修复维持）
td [664,716] w=52  → p w=20 h=20 ch=106.6 （oracle 20×106.61 ✓ 残差 0.01px）
```

test 臂：p h=752 fill（缺陷 4，见 §1）、ob div [280,680] w=400 h=126（x 差 16＝缺陷 3 待决策⑥）。

臂实现要点（供深修复用，git 无残留）：depth 三臂 `(Px, has_atomic, ortho-text)`；正交域 gate `parent_wm==HorizontalTb && !(sideways-lr+vlr)`；fit = min(横排 IFC 单行 advance, `icb_inline_avail`=viewport_height 递归下传)；回写仅 content_height（height 不动，UA p margin 物理 `1em 0` 下双写会撑高 cell 行高 12.6px——缺陷 3 ⑥ UA 表逻辑化后改双写）。

## 4. 验证矩阵（本轮收尾态）

- 回退后 `git status` clean（`9cd55425d`），layout-engine 无代码变化；tmp 探针/判例文件全部删除。
- 全量 corpus 双窗（臂 ON/OFF 同二进制 16594）：ON 692F/10332 可信、OFF 688F/10331 可信（差异即 §0/§2 四案）。
- ic-height solo：ON ×3 恒 ✗11.60%、OFF ×2 恒 ✓0.25%（定谳真回归）；vrl-007 solo：ON ×3 恒 3.58%、OFF ×3 恒 3.55%（采样稳定，非 flake）。

## 5. 下一步（R5469 候选）

1. **c5525-fltmrgn-000 ✗3.92% 归因探针**（R5467 下一步③，本轮顺延优先——缺陷 2/4 修复均阻塞深结构，轻量归因先行）。
2. fsa 度量单 owner 切片（css-fonts-5：渲染竖排 IFC fsa 接入＋探针 metric 同源 shaper）——深结构，有本档 §0 双侧实证可直开。
3. 缺陷 4 viv 行内 fill 语义（indefinite CB shrink-to-fit＋fill 轴源归因）——深结构，与 R5459/R5460 账本合并。
4. 缺陷 3（UA p margin 逻辑化＋竖排块流边距折叠）维持待用户决策⑥。
