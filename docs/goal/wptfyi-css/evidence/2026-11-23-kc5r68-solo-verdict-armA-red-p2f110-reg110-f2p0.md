# KC-5-r68 solo 判读 verdict-red（verify 流 judge，2026-11-23 09:4x）——臂 A（纯去 trio）实测证伪预配表

## 判定：三门全红（RESOLVED 崩盘至 0），r68 臂 A face 不落库（REDLINE fail-stop）

- **RESOLVED 0 / RESIDUAL-P2F 110 / REG 110 / F2P-BREAK 0，GT-missing 0**（5080 键两侧齐，14 页×双臂）。
- 对照 r67（2/108/125/25）：RESOLVED 2→0、RESIDUAL 108→110、REG 125→110、**F2P 25→0**。
- 判读输入：快照 `results-kc5r6-fame2b/solo-r68-snap-verify094x/`（28 臂 JSON，指纹 md5 `d86a27a6bceadd5a`，09:43 收官即抢，防并发重发覆写）；canonical `solo-r68/` 09:38 发射 09:43 收官。
- 判读工具：`kc5r66-verify-json.py solo-r68-snap-verify094x`（三门）＋本流逐键评分器 `/tmp/kc5r68-score.py`（GT-110 × `r68-armA-predict.json` 交叉评分）。

## 双臂 provenance（三角定谳：b1＝真臂 A 面，结果非 stale-bin 伪影）

- b0 臂 = `probe-bin/b0-runner` md5 `16ce5984`（GT 基线同 md5）；b1 臂 = `probe-bin/r68-runner` md5 **`eda6c938`**（09:38:20 构建，`/tmp/kc5r68-b1-build.log` Finished 40.19s BUILD-RC=0，≠r67-runner `a8c9e799`＝全新生代）。
- 脚本绑定静态核：`kc5r68-solo-verify.sh:24` b1 显式绑 `$PB/r68-runner`（模板陷阱已防）。
- **strings 代际正负检**：`r66_pair_suppress`/`KFOLD_R5`/`pair_endpoint_class` 三标记 b1=**0**/b0=0 ＝ **trio 剥离在二进制层兑现**；`parse_lp_terms` b1=3 ＝ r5 hunks 保留。臂 A 切割（r5 patch 35 hunks 全量 − r67 delta 6 hunks）与跑面一致。

## 预配表证伪记分（judge 前置 falsifiable 表 vs 实测）

| 预测项 | 预言 | 实测 | 判 |
|---|---|---|---|
| RESIDUAL | ~13-17 | 110 | **证伪** |
| RESOLVED | ~95-99 | 0 | **证伪** |
| REG | ~0-5 | 110 | **证伪** |
| F2P | ~25 持位 | 0 | **证伪（方向反）** |
| 2 键翻转（r67 抑制→echo 巧合 Pass） | 翻 Fail | 翻 Fail | ✓ |
| provenance（真臂 A 面） | — | 三角兑现 | ✓ |

- 逐键转移实测：Pass→Fail 2（预言✓）、Fail→Pass **0/97**（预言全灭）、Fail→Fail 11（预言✓）。
- F2P 25→0 反转的机制修正：r67 的 25 F2P-BREAK 是 **trio 因果**（抑制打落 r5 赢面），非 intake/LP 臂域——去 trio 即愈合。08:6x/勘误四的「intake 激活型」归属勘误。

## 机制图回修（证伪驱动的结论替换）

- **「模式 A 死点在 cssa consult 上游态（r5 patch 外）」假说被实测处决**：trio 剥离＋cssa 回 HEAD 后，b1 consult 交付仍是端点 echo 而非复合串（RESIDUAL trace：exp=`'calc-size(min-content, -30px + size * 1.3)'` 型复合＝b0 直通形，got=`'min - content '`/`'auto '`/`'200px '` 端点 echo/K-fold 折叠形）。**复合交付死亡在 35 hunks 的运行时链内**——cssa(HEAD) 在 r5 面树上产出与 HEAD 二进制不同的 consult 值。
- got 形签名分组（定位输入）：`'200px '`＝consult 返回可折叠关键字端点→K-fold 假想折叠（width/height 键）；`'min - content '`＝consult 返回关键字端点原样→非折叠门 passthrough→gCS token 化（min-*/max-* 键）；`'auto '`/`'50 % '`＝同 passthrough 形。→ **cssa(HEAD) 的复合合成链在 r5 面树上被短路成端点 echo**，毒点候选（按 hunks 组）：
  1. `js_anim_registry::canonicalize`（r5 hunk，consult 值 apply+gCS 往返＋K-fold 关键字透传臂）——consult 返回值经此改形；
  2. `AnimationClock` keyframe 预合成换道（r5 hunk，注册期 U⊕端点经 LP 臂改写关键帧端点——cssa 合成的输入被改）；
  3. `interpolate_property_value` LP 臂＋`compose_animation_value` LP 臂（端点拒析→离散翻转 echo，若 cssa 合成路径间接消费插值输出）；
  4. `apply_single_property` 六长手委托 style-system（apply+gCS 往返通道的写面变形）。
- cis 组（style-system 9+bridge 2+animation cis 臂+trans 登记）维持惰性判定（本轮无反证）；shim intake 3 hunks 升为候选（consult 激活面的上级开关）。
- **下一刀（r69）建议＝毒点组二分**：R1＝剥 js_anim canonicalize r5 hunks（保 Clock/LP/apply/shim）solo 探针（宽 min-height:KW→LP CA 族＋width:KW→cs 族两形）；R2 按 R1 红臂剥 Clock 域门组；R3 LP 臂。每次切割保持 cssa=HEAD、无 trio、其余 hunks 原样（单变量）。

## 工序注记

- 本流 judge 监视器（28 臂＋进程退出判据）09:43:09 触发即抢快照；canonical 目录此后若被并发流重发覆写，以本快照为准。
- 键级实核（快照逐键重算）：**b0P→b1F 全量 110 全为 GT 键，非 GT REG = 0**——r67 的 17 非 GT 回归键（16 trio 因果 + 1 which-value）在臂 A 全回收；REG 125→110 = −17 非 GT 回收 +2 GT 翻转（与预言 2 键翻转恒等）。REG 门数字含 GT 双计入，三court 判读时以「非 GT REG」为独立轴。
- 预配 falsifiable 表整表入档（含证伪行）＝机制图迭代的对照锚；下轮预配须先解本表三处证伪再定量。

## 工件索引

- 快照：`.acceptance/wptfyi-css/results-kc5r6-fame2b/solo-r68-snap-verify094x/`（md5 `d86a27a6bceadd5a`）
- 判读输出：`kc5r66-verify-json.py solo-r68-snap-verify094x`（本 doc 数字源）；逐键评分器 `/tmp/kc5r68-score.py`
- provenance：`/tmp/kc5r68-bin-md5.txt`、`/tmp/kc5r68-b1-build.log`、`kc5r68-solo-verify.sh`
- 预配输入：`results-kc5r6-fame2b/r68-armA-predict.json`（110 键逐键预言，勘误四精化版）
- 前轮 verdict：`2026-11-23-kc5r67-solo-verdict-red-p2f108-reg125-f2p25.md`（其 §量化预言/§勘误四 即本轮被证伪对象）

## 判后追加（09:5x）——LP 插值臂静态免责，毒点收窄至 apply 串化折叠缺口（三流收敛）

上文候选 #3（LP 臂致复合不产出）经 r5 patch 原文核实**出局**，候选表读法修正：

- r5 face sizing 六长手臂（patch interpolate hunk 原文）：`parse_lp_terms` 双拒 → `_ if t >= 0.5 => to.to_string()`；HEAD 同属性臂：`parse_px_strict` 双拒 → 同形 flip（≥0.5→to / <0.5→from）。**对复合端点两代逐字同构**——calc-size/fit-content/关键字端点在两代都原样穿过插值臂进 canonicalize。GT 面 harness 钉 progress 0.5（`query_core` 注释锚：-50s 钉 50/100），at(1)/at(1.25) 同 ≥0.5 → tick 产出恒 `to` 侧复合串，两代一致。
- 故复合串在两代都完整抵达 canonicalize，死亡发生在 canonicalize→apply→gCS 串化段：
  - b0：HEAD applier 拒收复合串 → canonicalize None → consult miss → cascade 静态折叠 == exp（07:4x face 流实证链「HEAD canonicalize None（applier 拒收）→ underlying」；GT exp 本身＝参考元素 cascade 静态 gCS 折叠形，同通道恒等）；
  - r5 面：apply 委托 hunk 令 applier **接受**复合串（勘误四「生产增量」同一 hunk）→ canonicalize 命中 → gCS 串化出 token 化函数形（`'fit - content ( 100 % )'`/`'calc ( 3 % + 74px + 17px )'`）≠ 折叠 used px == exp → FAIL。**consult 命中在 r5 面反而遮蔽了本会折叠成功的 cascade 静态通道**。
- **毒点收窄＝apply 串化折叠缺口**（上文候选 #4 为首，候选 #1 的 js_anim 共享 canonicalize 若仅透传臂改动则同样免责，R1 可并测）。与 judge 流 09:5x 笔「97 键 r68-b1/r67-b1 got 字节级全等 ⇒ 死点 = canonicalize→apply 复合折叠缺口」及勘误四「生产增量 = apply 委托 hunk」三点收敛。
- **回收杠杆修正**：trio 抑制路线彻底出局——抑制→None 后 r5 callbacks/K-fold 层 echo ≠ b0 的 cascade 静态折叠（r67 RESOLVED 2 为巧合通道已验），r67/r68 两态 got 字节级全等＝抑制与否对复合键输出零效应。主刀＝**applier 复合串折叠到 used 值**（案 2 推广：calc-size 族全覆盖，fit-content() 仅子集）；上文「LP 臂二分（R3）」按本法证降级，R1 剥 js_anim canonicalize 仍可先测（与 apply 折叠族同链）。which-(b) KW-from 端点离散落值族独立维持另刀。
