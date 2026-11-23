# KC-5-r68 臂 A solo 判读 verdict-red（executor 流，2026-11-23 09:4x）

## 判定：三门 P2F FAIL / REG FAIL / F2P PASS——r68 臂 A（r5 基去 trio）不落库（REDLINE P2F>0 fail-stop）

- **RESOLVED 0 / RESIDUAL-P2F 110 / REG 110（非 GT 0）/ F2P-BREAK 0，GT-missing 0**（5080 键两侧对称，14 页×双臂，28 entries 齐）。
- 勘误四精化表预言（RESIDUAL ~13-17／RESOLVED ~95-99／REG ~0-4／F2P ~25）与 oracle 预注册（point 13/97/14）在 GT 轴全面失准：RESIDUAL 110 vs 13-17、RESOLVED 0 vs 95-99；**REG 与 F2P 两轴兑现且更优**（非 GT REG 0 ∈ 预言带；F2P 0 < ~25）。
- 失准模式＝方向性证伪：**suppress-echo 理论（trio 抑制打落 r5 巧合对称 echo ⇒ 解抑回收 97 键）死亡**——97/97 键在臂 A 下仍以 r5 原生 echo 形败（`'200px '`/`'min - content '`/`'max - content '`/`'fit - content ( 100 % ) '`/`'50 % '`），无一翻转 Pass。
- 对 r67（49→2→0 RESOLVED 轨迹）与 r66（49）对照：臂 A 是 GT 轴历史最差代——**r5 基 35 hunks 本体就是 GT 塌缩机制**（端点 echo ≠ 折叠 exp），trio 非 GT 败因（反而是 r67 仅有的 2 键 RESOLVED 来源）。

## 双臂 provenance

- b0 臂 = `probe-bin/b0-runner` md5 `16ce5984`（历代同款基线）；b1 臂 = `probe-bin/r68-runner` md5 `eda6c938`（09:32 构建 `cargo build --release -p zero-wpt-runner`，BUILD-RC=0，40.19s，log /tmp/kc5r68-b1-build.log）。
- 分代 strings 检（`/tmp/kc5r68-gen-strings.txt`）：`parse_lp_terms` b1=3＝r5 LP 臂在位；`r66_pair_suppress` b1=0＝trio 全剥；对照 r67-runner 3/2、b0-runner 0/0——臂 A 代际签名三向分离成立。
- 面驻留独立复证（发射前，只读）：staged 12 文件 35 hunks 与 `knife-kc5r5-fame2b-face.patch`（md5 `9b4d528b` ✓）hunk 体逐一全等（python 逐 hunk 比对 0 mismatch）；hunk 分布逐文件全对（animation 11/callbacks 4/computed_style 2/cache 2/jsanim 2/trans 2/part03 3/cascade 1/apply_advanced 4/cs(style) 1/default_impl 1/inherit 2）；trio 标记 staged diff 0 命中；cssa_registry.rs 干净＝HEAD；worktree==index 零未暂存差。
- 脚本绑定静态核：`kc5r68-solo-verify.sh:24` b1 臂显式绑 `$PB/r68-runner`（r66 pass1 模板陷阱防御沿用）；单实例锁 `/tmp/kc5r68-solo-instance.lock` 持有发射，无双写。
- solo 发射 09:33（本 executor fork，后台任务），09:43:04 `SOLO-VERIFY-ALL-DONE`，`SOLO-ALL-RC=0`；快照先行 `solo-r68-snap-exec1123094338`（28 entries）后判读；判读链 `kc5r66-verify-json.py`（JSON 直读）＋oracle 逐键 diff，输出 `/tmp/kc5r68-judge-out.txt`。

## 预注册对账（发射前落盘，发射后即验）

- 发射前已将 oracle 全 110 键对 r5 代 B1MSG（`r6-p2f-groundtruth.txt` 自带列）做形对账并存证 `.acceptance/wptfyi-css/kc5r68-pre-reg-r5form-census.txt`：**110/110 全部 R5FORM-FAIL**（r5 代 expected≠got，97 suppress-echo 键无一例外，如 exp `'0px '` vs r5got `'200px '`、exp `'100px '` vs r5got `'min - content '`）。
- 判读实证：110/110 残面 got 形与 r5 代 B1MSG 同形（form flips 0 维持，08:3x 原生性定谳在臂 A 上第三次复现）——**预注册 r5 原生假说全胜，oracle 假说全灭**。这条对账在发射前已把「oracle 97 Pass 预言 vs r5 代 110/110 fail 旧证」的矛盾摆上台面，solo 裁决取后者。
- REG 轴预言（09:2x 键级锚：17 非 GT 回归键＝16 trio 簇＋which-(a)）兑现：非 GT REG 0，17 键全回收（which-(a) r5-runner 探针 PASS 预言同向兑现）。
- F2P 轴预言（~25 持位）超额兑付：0 break——r67 的 25 破坏（WA 缺口族）随 trio 剥除全回收。

## 机制定谳（三线收敛）

1. **GT 塌缩机制在 r5 hunks 本体，与 trio 无关**：臂 A（纯 r5 面）GT 110/110 全 P2F，got 形逐键同 r5 代——端点 echo（from-keyword token 化串／from-LP used px）经 cascade/apply 投递后 ≠ 折叠 exp，r67 期勘误已定位的「投递形」机制即本案主犯。trio 只是在 r5 之上对个别键（csKW↔LP 2 键）做巧合翻转、对非 GT 面（REG 17＋F2P 25）做净破坏。
2. **suppress-echo 理论证伪过程**：claude-1000 流 08:3x 回注三曾以「抑制→echo→巧合 Pass」重释 r67 残 108 主簇并支撑 oracle 97 Pass——本跑 97/97 不翻转，且 r5 代 B1MSG（无 trio 时代）got 形与臂 A 全同形，证明该批键的败因从来不是「echo 被抑制」而是「echo 本身 ≠ 折叠 exp」。勘误四的 mode B 6 键（b0=Fail 双臂预存败）与本跑无交集，维持原判。
3. **GT 三门轨迹全景（r6-p2f GT-110 口径）**：r65 RESIDUAL 16（单调最优）→ r66 61 → r67（r5+trio）108 → r68 臂 A（纯 r5）110；F2P-BREAK 轨迹 r66 375 → r67 25 → r68 0。r5 基换入 = F2P 赢面全额兑付 + GT 面崩盘，纯 r5 基不可落库；r65 基 GT 最优但 F2P 缺口另计。

## 下一步刀位建议（face 流定夺；judge 流按新 solo 再判）

- **主刀＝r5 LP 臂投递形手术**：35-hunk 静态图（r67 evidence doc §r5 patch 全 hunk 静态定位）已把 hunks 分为 cis 组／K-fold consult 层／插值 LP 臂／intake 门四类；本跑把 GT 塌缩钉死在「插值 LP 臂端点 echo＋intake 激活投递」子集——hunk 级二分（原 08:3x 降级计划）在「r5 基＋cssa HEAD＋无 trio」当前驻留态上做单变量切割的条件已成熟：每臂 build＋GT 页探针即可判塌缩归属，勿整 14 页。
- **案 2（applier fit-content() 串化解析回退）维持独立价值**：fit-content-apply-form 9 键在臂 A 仍败（r5got `'fit - content ( 100 % ) '` 形），勘误四的修键条件判定不受本跑影响。
- **r65×r5 组合刀提上**：GT 单调最优（r65 残 16）× F2P 赢面（r5 pool 0 break）的组合，即「保留 r5 的 F2P/intake 增益面＋剥 r5 的 GT-echo 投递 hunks」——与本跑后成熟的 hunk 二分同一张工作面。

## 工件索引

- 快照：`.acceptance/wptfyi-css/results-kc5r6-fame2b/solo-r68-snap-exec1123094338/`（28 entries）
- 原始跑目录：`results-kc5r6-fame2b/solo-r68/`；tx 日志 `/tmp/kc5r68-tx-<page>-<arm>.log`
- 判读输出：`/tmp/kc5r68-judge-out.txt`（三门＋got-trace＋簇表）
- bin：`probe-bin/r68-runner` md5 `eda6c938`；md5/strings 存证 `/tmp/kc5r68-bin-md5.txt`、`/tmp/kc5r68-gen-strings.txt`
- 预注册存证：`.acceptance/wptfyi-css/kc5r68-pre-reg-r5form-census.txt`（发射前）、`kc5r68-staged-diff.txt`（面纯度）
- 发射脚本：`.acceptance/wptfyi-css/kc5r68-solo-verify.sh`；构建日志 `/tmp/kc5r68-b1-build.log`
- 认领与对账注记：`.acceptance/wptfyi-css/NOTES-kc5r66-solo2-judge-claim.md`（executor 流 09:3x 认领节）
