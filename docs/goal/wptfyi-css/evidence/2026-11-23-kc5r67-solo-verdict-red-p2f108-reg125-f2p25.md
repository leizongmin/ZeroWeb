# KC-5-r67 solo 判读 verdict-red（verify 流，2026-11-23 08:2x）

## 判定：三门全红且 RESOLVED 崩盘，r67 face 不落库（REDLINE P2F>0 fail-stop）

- **RESOLVED 2 / RESIDUAL-P2F 108 / REG 125 / F2P-BREAK 25，GT-missing 0**（5080 键两侧齐，14 页×双臂）。
- 对 v5 预测（105/5/0/1）再度三面全失准：RESOLVED 2 vs 105、RESIDUAL 108 vs 5、REG 125 vs 0；F2P 25 vs 1 是唯一接近轴。
- 判读工具 `kc5r66-verify-json.py solo-r67-snap-verify0823`（JSON 直读链，先快照后判读）；输出 `/tmp/kc5r67-judge-out.txt`。
- 与 r66（49/61/112/375）对照：**RESOLVED 49→2 崩盘、RESIDUAL 61→108 恶化**，但 **F2P-BREAK 375→25 大幅收敛**（r5 基恢复兑现了「r5 赢面 parity」设计目标）——单刀未同时压三门的历史第三次重演。

## 双臂 provenance

- b0 臂 = `probe-bin/b0-runner` md5 `16ce5984`（GT 基线同 md5）；b1 臂 = `probe-bin/r67-runner` md5 `a8c9e799`（08:18:20 构建，mtime 晚于 face 文件末次编辑 08:16:00，无 stale-bin；strings 正负检 trio 标记齐：r66_pair_suppress b1=2/b0=0、pair_endpoint_class b1=1/b0=0、fold_consult_for_selector b1=4/b0=0、parse_lp_terms b1=3/b0=0）。
- 脚本绑定静态核：`kc5r67-solo-verify.sh:24` b1 臂显式绑 `$PB/r67-runner`（不依赖 `$ARM-runner` 模板——r66 pass1 同型陷阱已防御）。
- solo 发射 08:18:25（本流实例，flock 持有），08:23:27 收官；竞争实例 08:18:36 撞单实例锁 exit 42（防双写门生效，其 SOLO-EXIT=42 行混入共享日志曾致 watcher 早触发误报——实例存活核验后排除）。

## 形态定谳：token 化 keyword echo 是 r5 原生伪影（初判「r67 新形」勘误）

- 初判曾把 r67 残面 token 化 keyword echo（`'min - content '`/`'auto '`/`'fit - content ( 100 % ) '`，token 间带空格＝逐 token 序列化）当作 r66 px echo → r67 的新形态漂移——**判后零构建分析证伪并勘误**：GT 文件（`results-kc5r5-fame2b/r6-p2f-groundtruth.txt`）自带 r5 代 B1STATUS 列＝110/110 全 fail（r5 时点 GT 轴已全塌）；r67-b1 对 GT-110 逐键 got-form 比对 r5 B1MSG＝**form flips 0**（40 px echo＋54 token 化 keyword echo＋9 fit-content＋4 pct echo＋1 cis 双侧同形，仅 2 键经 cssa 透传回收）。r66 的 px echo 单一形是 r65 基坏栈的特定伪影；r67 恢复 r5 基＝逐键恢复 r5 自己的残面形。
- 机制指向（不变且加固）：插值复合塌缩到 from 端点 keyword 的机制在 r5 hunks 本体（apply/cascade/animation 系）；r5 patch 在自基与现 HEAD 上行为逐键一致＝**非 base 交互**，现 HEAD 上做二分无交互噪声。「cssa 回 HEAD 恢复透传＝GT 面可过性」假设证伪维持——透传只救回 2 键（calc-size-height-interpolation），通道对称化（apply-往返 vs cascade 不对称，GT-110 根机制）是必要刀，透传非充分条件。

## F2P 轴：r5 基恢复基本兑付

- F2P-BREAK 375→25：残 25 簇＝width:LP→KW 7、width:KW→cs 6、width:NEU→KW 4、height:KW→cs 4、to-neutral 3、nomatch 1。r66 的 375 破坏主簇（r65 基坏栈拖累）随 r5 基回植消解。
- 对照：预测残 5 键（calc-h cs→LP 2 超额兑付／which-value＋cis 持位／1em→10vw 重分类）在 r67 残面全部复现同位（nomatch 784px、WA cis `50px 60px ` 在 RESIDUAL 列表可见）。

## REG 轴

- REG 112→125（+13），可见簇＝width:LP→KW 7、width:KW→cs 6、to-neutral 3、nomatch 1（显示截断，全量以 JSON 快照为准）；r66 的 max-*:NKW→LP WA 28 键簇未进显示前列。

## 判读样本与工序注记

- **transcript 尾段丢失**：本流日志 `/tmp/kc5r67-solo-all.log` 于发射窗内被并发流 rm 重建（fd 1 指向 deleted inode），RUN 行 24/28 经 `/proc/<pid>/fd/1` 快照抢救（`/tmp/kc5r67-solo-mine-final.log`）；末 4 腿（min-width/width 两页）RUN 行随进程退出丢失，**键级数据无损**（judge 直读 arm-dir JSON，5080×2 齐）。
- 结果快照：`results-kc5r6-fame2b/solo-r67-snap-verify0823/`（28 JSON 全非空，指纹 md5 `3d1564239dfb6c4dc3311b86509ffcbd`），canonical `solo-r67/` 同内容；快照先行防并发流重发覆写。
- 单轮判读即 RED（P2F=108 远超瞬态噪声域，无需确定性双跑；r66 的双证协议用于绿门验证，红门 fail-stop 不依赖重跑）。
- 教训：①共享日志名＋并发流清理＝deleted-inode 输出丢失，唯一日志名按流绑定（`solo-<face>-<stream>.log` 级）；②watcher 完成判据不得裸 grep 共享日志退出标记（他流撞锁行混入），以 PID 存活＋自身 RUN 计数为准；③JSON 直读判读链对 transcript 形态/丢失免疫，arm-dir JSON 才是判读权威输入。

## 对照池（GT 三门轨迹累计）

| 轮 | RESOLVED | RESIDUAL | REG | F2P-BREAK |
|---|---|---|---|---|
| r6.3 (w3) | 93 | 17 | 211 | 573 |
| r6.4 (w4) | 49 | 61 | 73 | 332 |
| r65 | 94 | 16 | 184 | 551 |
| r66 | 49 | 61 | 112 | 375 |
| **r67** | **2** | **108** | **125** | **25** |
| v5 预测 | 105 | 5 | 0 | 1 |

- **RESIDUAL 单调最优仍是 r65（16）**：r65 的输出侧门＋宽抑制组合至今是 GT 轴唯一有效解；r5 基（r67）与 r6.4 基（r66）都不承载 GT 轴。r68 若延续 face 路，基座选择须先回答「r5 patch 哪些 hunks 塌缩插值复合」——逐 hunk 二分计划已入 `NOTES-kc5r66-solo2-judge-claim.md` 尾节（hunk 分布＋探针形族＋轮次建议），谓词/抑制层在其上才有意义。

## r68 靶面决策输入（verify 流判后追加，零构建实证，非静态推演）

1. **失败两模式（1883 b1 败键全量 got-form census）**：
   - **模式 A＝复合未产出**（~1877 键）：got 全为动画缺席时的元素裸值（top 形 `'0px '`×716/`'10px '`×291/`'200px '`×215/`'50px '`×128…＝width/height 用值、min-*/max-* 计算值），含 calc-size 复合串的 got 仅 6 键——KW 涉足 shape 的插值复合根本没到元素上，gCS 返回的是无动画 computed/used 值（token 化串化）。
   - **模式 B＝复合已产未折叠**（恰 6 键，全在 calc-size-height/width-interpolation 页 csKW↔LP 形）：got=`'calc ( 3 % + 74px + 17px ) '` 线性组合未折叠 vs exp=`'125px '`/`'100px '`/`'75px '` 已折叠值——r5 面 applier 接受并串化了未解析 calc，HEAD 同键解析到用值。**模式 B 直指 applier calc-size used 解析臂（07:4x ⑤ 案＝靶面案 2 的辖区）**。
2. **r65-vs-r5 cascade 通道判别器**：r65-b1 的 94 GT 解键＝r65 面 None→cascade→apply→串化管线曾把 exp 级值送达元素（94 键 got==exp）；r5 面（r67）同通道产出 mangle 形（`'min - content '` 等）——**mangle 由 r5 patch 非 cssa hunks 携带**（applier/cascade/animation 侧），cssa 回 HEAD（r67 已做）只修 cssa 腿不修 cascade 腿。r65 的 94/16＝输出侧门在输出位 masking 此 mangle，非修复。
3. **靶面案映射**：案 1（canonicalize 产出收敛 cascade 静态折叠形）辖模式 A；案 2（applier calc-size used 解析臂回退）辖模式 B＋mangle 根除。若静态定位证实 mangle 在 applier/cascade 层，案 2 是根修案 1 是补丁案；两案对 6 键模式 B 均需 applier 层动作。
4. 二分锚点建议：模式 B 6 键（calc-size-*/csKW↔LP）作定位探针最锐——mangle 复现/消失的 hunk 即案 2 根因；模式 A 用 min-height:KW→LP CA（got=`'auto '` 型）族作第二探针。静态定位若与此实证冲突，按既有教训（静态机制推演在多层 fallback 链下连环失准，got/expected 样本是硬锚）以实测优先。

## r5 patch 全 hunk 静态定位（verify 流判后追加，只读零树变更零碰撞）

35 hunks 全量通读完毕（12 文件；face 流认领二分切割，本流为支持性机制地图，结论与其静态定位冲突时以其实测优先）。

### hunk 全分类

- **cis 工作（KC-4/KC-5-A，对 calc-size 复合路径惰性）**：style-system 侧 9 hunks（apply_advanced 4＝cis auto 旗标/长手解析、computed_style(style) 2＝cis struct 字段+聚合臂、default_impl 1、inherit 2、cascade 1＝KC-5-B cis 槽确定性合并）＋bridge computed_style.rs 2（cis_axis_to_css 串化/简写聚合）＋animation cis 臂组（interpolate_cis_value/parse_cis_axes/CisAxis/strict 登记臂/apply 写面）＋transition 登记 cis 三属性＋shim cis ④ 豁免。**全部 contain-intrinsic 域，非本案机制位**。
- **K-fold consult 层（r5 独有，HEAD 无）**：callbacks 2 hunks＝cache-hit 臂与 miss 臂的三咨询（jsanim→trans→cssa）直通 return 改道 `fold_consult_for_selector`；computed_style_cache 2 hunks＝handle 通道同门＋`fold_sizing_keyword_core`（clone styles→静态 apply 写关键字→全量布局→rect→used px）/`is_foldable_sizing_keyword` 实现；js_anim_registry canonicalize 关键字透传臂（width/height ∩ 折叠集关键字→原串透传交 gCS 折叠）＋KFOLD_R5_MARKER。
- **插值 LP 臂（r5 独有）**：animation `interpolate_property_value` 六长手 fame2b 臂——`parse_lp_terms` 域＝裸 px/裸 %/裸 0/恰好双 term `calc(Apx ± B%)`，**calc-size 复合/多 term calc/关键字端点全拒→离散翻转端点原样 echo**；compose LP 臂（U⊕端点逐 term 加法，不可加回落原样）；Clock 六长手域门＋keyframe 预合成换道；`apply_single_property` 六长手委托 style-system `apply_property_value`（width/height 旧宽松 parse_px 臂、min/max 旧 parse_px_strict 臂移除）。
- **intake 门（r5 独有）**：shim 3 hunks＝`_zwIsLp`/`_zwIsSizingLp` 六长手进 WAAPI 合成域门＋④ 单位 multiset 闸豁免（此前 KW↔cs 对被误杀不登记）；transition 登记数 89→92（cis 三属性，域内）。

### 机制链（与三门实证逐项对账）

1. **K-fold 对复合透明**：非折叠集串（px/%/calc/calc-size 复合）Some 原样透传；折叠仅 width/height ∩ {auto,stretch,min-content,max-content,fit-content,fit-content(}。→ **r5 patch 35 hunks 不杀 consult 复合透传**——「mangle 由 r5 patch 携带」在此意义上收窄。
2. **模式 A（~1877 键）死点在 cssa consult 上游、非 35 hunks**：r5 代 cssa canonicalize 门（r5 patch 外的树态，r67 已回 HEAD）／r67 trio 谓词抑制 → consult None → fold(None) → **echo 静态通道** → got＝无动画静态值（px echo＝width/height used echo；keyword token 化＝min-*/max-* computed 关键字串化；min-height `'auto '` 族＝同通道）。r5 期 110/110 fail 与 r67 108 fail 同形（form flips 0）跨 cssa 态稳定＝此链的结构性证据；r67 RESOLVED 2＝trio 未辖的 2 键经 cssa 直通回收（透传恢复有效但被 trio 抵消）。
3. **r5 patch 的 P2F 贡献＝intake 激活＋投递形**：shim/Clock/transition 门放开让 KW↔cs 对进入引擎管线（b0 期 ④ 闸误杀→不登记→WAAPI 静态 underlying 与 harness 期望巧合 Pass）；激活后 LP 臂离散翻转端点 echo／K-fold 折叠 px 投递落形失配 → b0P→b1F。r65-vs-r5 判别器「mangle 在 applier/cascade/animation 侧」对账：94 键 folded-px-exp 类＝r5 面投递形失配（r65 面输出侧 fold 直连节点臂 +28 于 callbacks 等处投递 px 兑付）；复合 exp 类在 r65 面仍死（RESIDUAL 16/REG 184）。
4. **模式 B（6 键）＝intake 激活＋K-fold 透放，非 applier 单独可修**：b0 无咨询→echo 通道投递 used px == exp（`'125px '`）；r67 咨询激活（intake 门放开后 WAAPI 管线产出 blend/calc 复合串）→ K-fold 透传臂放行非折叠串 → got＝token 化未折叠 `'calc ( 3 % + 74px + 17px ) '`。applier used 解析臂只修串形；exp 来源是 b0 的巧合 echo 通道，故 revert intake 门（shim 3 hunks）即预言 6 键回 Pass。
5. **r65 输出侧门＝投递位 masking 非根修**：r65-only 增量 census——animation +126（parse_lp_terms 拓宽 lp_relative_px/括号深度拆分/CisAxis 逐轴/max(0.0)）、callbacks +28（fold_consult_for_node 直连节点臂）、cache +46、js_anim_registry +75、transition +41、shim +7（cis 白名单）＝投递覆盖面拓宽；上游 consult 死点（cssa 态）在 r65 面未除（复合 exp 类 REG 184/RESIDUAL 16 持位）。

### r68 决策输入修订（覆盖前节第 2-3 条的靶面映射）

- **复合 exp 键（模式 A 主簇）的最小刀＝cssa 层去 trio 保直通**：r67 已证 HEAD cssa 直通可回收（RESOLVED 2），trio 抑制是 r66→r67 RESOLVED 49→2 崩盘主因；35 hunks 无需动即回收复合 exp 类。
- **案 2（applier calc-size used 解析臂）辖区收窄**＝mode B 串形＋folded-px-exp 类投递形；其 exp 来源（b0 巧合 echo）意味着 intake revert 同样兑付该 6 键。
- **案 1（canonicalize 产出收敛 cascade 静态折叠形）** 辖 folded-px-exp 类的抑制分支残面；对复合 exp 键产出 echo 形恒失配（静态形≠复合串）。
- **hunk 级二分预言**（供 face 流切割参考，本流不做实测）：revert shim intake 3 hunks → 模式 B 6 键 + intake 激活型 P2F 回 b0 巧合；revert callbacks/cache K-fold 4 hunks → consult 投递回 HEAD 直通形（cssa 态允许时）；trio 剥离（cssa 侧，patch 外）单独可测。三臂互不纠缠，可单变量切割。

## trio 本体判读与量化预言（verify 流判后追加，judge 前置）

`knife-kc5r67-fame2b-face.patch`（md5 `37745a7d`，13 文件 41 hunks）对 r5 patch 的 delta 恰 6 hunks＝trio 全体：`computed_style_cache::r66_pair_suppress`（定义 +105 行，PairEndpointClass 七类＝Px/Lpo/Kwid/Nkw/CsKw/CsLp/Neu）＋四个挂点（cssa_registry 1／js_anim_registry 2＝cssa_or_waapi_lane=true，transition_registry 1＝lane=false）＋animation `AnimationClock::state()` 只读快照 accessor。抑制规则集：①interpolate-size:allow-keywords 元素 ∧ 对含 KWish/cs 端点；③width/height ∧ CSSA/WAAPI lane ∧ min/max-content/fit-content↔LP/px；③'max-* ∧ CSS-wide↔LP/px（全 lane）；G2 KW↔cs 裸关键字 basis；追加刀 csKW↔LP/px。**缺端点（时钟态关键帧无端=neutral 侧 None）规则不发射、照旧 canonicalize**。

- **RESOLVED 2 归属勘误（本 doc 前文「透传回收」修正）**：arb NOTES 08:3x「csKW↔LP 追加刀兑现」与抑制→echo→巧合 Pass 链对账成立——该 2 键（calc-size-height csKW↔LP）经抑制回落 echo 投 used px == exp 折叠值而 Pass；若去 trio 反而落 mode B 形（同页 6 键＝缺端点未抑制→复合透传→token 化≠exp 折叠值→Fail 实证）。即 **trio 对该 2 键是增益、对 ~106 复合 exp 键是死因**。
- **量化预言（falsifiable，solo-r68 判读即验）**：
  - **臂 A＝纯去 trio**（6 r67-only hunks revert，r5 hunks 原样）：cssa consult 回 HEAD 无门直通 canonicalize → 复合 exp 键直通回收；RESIDUAL 预言 ~10（mode B 6 未抑制持位＋原 2 键翻 Fail＋which-value/cis 残 ~2）；REG 预言 ~0（b0 consult 形恢复，125 全簇＝width LP→KW 7/width KW→cs 6/max-* NKW→LP WA 28 等 consult 命中形）；F2P 持位 ~25（intake/LP 臂域，trio 外）；RESOLVED 预言 ~92-96。
  - **臂 B＝纯 revert shim intake**（3 hunks）：mode B 6 键回 b0 巧合 Pass；但 r5 赢面形（F2P 域 intake 激活型）失活→F2P-BREAK 大幅回升；RESIDUAL/REG 基本持位（cssa 上游态未动）。**非独立臂，只能与臂 A 组合**。
  - **臂 C＝纯 revert K-fold**（callbacks 2+cache handle 门 2）：cssa 被 trio 抑制臂不受影响（None 链在 fold 上游）；jsanim 关键字端点改走 apply+串化往返＝r5 已知错值形（Stretch 塌 0px）→ 小幅恶化。**单独无收益**。
  - **案 2（applier calc-size used 解析臂回退）**＝修 mode B 6 键串形（复合→折叠 px == exp）；案 1（canonicalize 收敛静态折叠形）对复合 exp 键产出 echo≠复合串恒失配——**案 1 单独不解模式 A 主簇**。
  - **组合预言 A+案 2**：RESOLVED ~98-102／RESIDUAL ~2／REG ~0／F2P ~25——F2P 25（intake 激活型）成下一战线（需 shim/LP 臂域 r65 式赢面工程，另一战役）。
- 判读锚：solo-r68 若落臂 A，GT 面板三门对预言表逐项核；预言失准处按 got/expected 样本回修本图（静态推演三演失准教训仍有效，本表可证伪）。

## 勘误四（快照键级法证，2026-11-23 09:1x 追加）——mode B 归属与臂预言修正

快照键级提取（solo-r67-snap-verify0823，GT-110 全键 got/exp 对账）三修正：

1. **GT 级「复合已产未折叠」＝fit-content(100%) 9 键**（max-width:fit-content↔50px×6／max-width:neutral→fit-content×3／min-width:fit-content→neutral×3 中 GT 成员，exp=`'150px '`/`'10px '`＝b0 折叠 px）：got=`'fit - content ( 100 % ) '` token 化。**生产增量＝r5 apply 委托 hunk**（max-/min- 旧 parse_px_strict 臂→style-system apply_property_value 解析 fit-content 产出 100% 形）非 cssa/trio（页面无 allow-keywords→① 不发射＝trio 盲区，臂 A 不动其败）。前文「mode B＝calc-size-* 6 键」归属勘误。
2. **`'calc ( 3 % + 74px + 17px ) '` 6 键＝双臂预存败**（b0=Fail b1=Fail，非 P2F/REG）：b0 亦不 Pass，前文「b0 巧合 echo 通道/HEAD 同键解析到用值」对该 6 键不成立——**「revert shim intake 3 hunks → 6 键回 Pass」预言作废**（臂 B 对 GT 面修复≈0；其真实效应仅剩 r5 赢面失活→F2P 恶化，非独立臂结论加强）。1883 键面 taxonomy 作为败面描述仍有效，唯 GT 级驱动归此勘误。
3. **臂 A 残面精化 ~13-17**（9 fit＋which-value 1＋WA cis 1＋原 2 键翻败＋comp-add 0-4〔exp=`'calc ( 4 % + 130px ) '` 型加法复合，若 cssa HEAD 加法 consult 恢复则回收〕），RESOLVED ~95-99／REG ~0-5；**案 2 辖区扩条件**＝若 applier 回退覆盖 fit-content() 串化解析则修 9 键，仅 calc-size 则 0。
4. RESOLVED 2 实证锁定：`height csKW↔LP at(1)/at(1.25)` 两键 b1=Pass——追加刀兑现（抑制→echo）归属维持。

## 勘误五（solo-r68 判后追加，verify 流 judge 位）——臂 A 证伪与机制图三修

**判定：solo-r68（臂 A＝纯去 trio，r5 hunks 驻树）三门 0/110/110/0（RESOLVED 0／RESIDUAL-P2F 110／REG 110／F2P-BREAK 0，GT-missing 0，5080 键两侧齐），双臂预言（face 流预注册 97 suppress-echo→Pass／本流精化表 RESOLVED ~95-99）同向证伪，r68 face 不落库（REDLINE fail-stop）。** 判读输入＝快照 `results-kc5r6-fame2b/solo-r68-snap-verify0945/`（28 臂 JSON 全非空，指纹 `cc27801fc8bfea7e`），b1 显式绑 r68-runner（脚本 :24），树态核 r66_pair_suppress 全树 0（trio 剥除确认）。

### 键级法证（快照 t 桶判别，1870 composite-exp 败键）

- **got 形 t 无关**：t<0.5（615 键）与 t≥0.5（855 键）got 同族——静态 echo 主导（`'0px '`×705／`'200px '`×205／`'50px '`×128／`'20px '`×104／`'400px '`…）＋基关键字子集（`'auto '`、`'min - content '`×18、`'max - content '`×18 恒等双侧）。**排除 LP 臂离散翻转 echo 模型**（flip 预言 t<0.5=from 形/t≥0.5=to 形，实测无翻转）。
- 实态分解：①静态 echo 形多数簇（`'0px '`×705／`'200px '`…＝t 无关 underlying/静态值）——通道归属未定谳（canonicalize None 回落 vs consult 命中但串化产出静态形，两候选并存；executor 流 09:5x 毒点模型＝apply 委托接受→Some token 化，与本簇键级归属留 P 探针判别）；②基字形子集（`'auto '`＝`calc-size(auto,…)` basis、`'min - content '`）＝canonicalize Some(基字形)——style-system apply 解析 calc-size 产 basis 槽串化；③**LP 可解析键全数恢复**（REG 125→110、F2P 25→0＝r5 赢面池完好，cssa tick→canonicalize 链对 px 级值正常）＝consult 通道活的判据。
- r67→r68-A 轴位移对账：RESOLVED −2（csKW↔LP 追加刀 2 键翻败＝本流预言项 ✓）、RESIDUAL 108→110、REG −15、F2P −25（恢复）。REG 口径对齐 09:4x verify fork 单列笔：**REG 门 110 含 GT 双计入，非 GT REG 键级实核＝0**（r67 的 17 非 GT 回归键臂 A 全回收）＝LP 可解析赢面池完好判据维持。
- **机制图三修**：「模式 A 死点＝cssa consult 上游态（门/trio）」证伪——trio 剥除后复合对仍死（None→echo）＝死点在 suppression 下游的 tick/canonicalize/注册链内部；b0 的复合 exp 通道不被 consult 恢复单独兑付。cssa 链已静态全读（`query_core→clock.tick→animation.rs 插值→canonicalize(apply+串化往返)`，animation.rs **无 calc-size 复合臂**〔grep 零命中〕）——b0 复合产出位仍未定（cascade 侧复合臂嫌疑最大，cssa_registry 注释「canonicalize 复合串透传」与 tick 无复合臂矛盾未解，**静态定位在此深度挂起**，公开声明不做无证据推演）。
- **r69 单变量探针（falsifiable，face 流切割权）**：P1＝revert Clock 域门 2 hunks（:3947 continue 门＋:3985 预合成换道）——预言复合 exp 回收 ~97 键若「预合成 corrupt 关键帧态→cascade 复合消费位」模型真；P2＝revert apply 委托 1 hunk——诊断基字形子集行为；P3＝canonicalize `calc-size(` 前缀 raw 透传臂（~2 行，js_anim canonicalize）——预言基字形/None 两族直通复合串==exp 而 Pass（b0 gCS 复合透传能力已证），RESIDUAL → ~13-15（9 fit＋which 1＋cis 1＋comp-add 2±）。按（改动成本×预言产额）排序 P3>P1>P2；三探针互不纠缠可单变量验。P3 ⊂ executor/verify fork r69 建议 R1（剥 js_anim canonicalize）同族——P3 为靶向透传细化、R1 为整段剥除，切割权在 face/executor 流。
- 工序注记：judge 直读快照 JSON（多路 judge 并行快照 8 变体并存，本流快照先行于收官后 2 分钟；canonical `solo-r68/` 同源）。face 流预注册 census `kc5r68-pre-reg-r5form-census.txt` 与本判读同输入。

## 工件索引

- 判读输出：`/tmp/kc5r67-judge-out.txt`（本 doc 数字源）；判读工具 `.acceptance/wptfyi-css/kc5r66-verify-json.py`（`solo-r67-snap-verify0823` 参数）
- 结果快照：`.acceptance/wptfyi-css/results-kc5r6-fame2b/solo-r67-snap-verify0823/`（canonical `solo-r67/` 同内容）
- 本流日志快照：`/tmp/kc5r67-solo-mine-final.log`（24/28 RUN，deleted-inode 抢救）
- 判读脚本：`.acceptance/wptfyi-css/kc5r67-solo-verify.sh`（b1 显式绑 r67-runner）
- 前轮 verdict：`2026-11-23-kc5r66-solo2-verdict-red-p2f61-reg112-f2p375.md`
