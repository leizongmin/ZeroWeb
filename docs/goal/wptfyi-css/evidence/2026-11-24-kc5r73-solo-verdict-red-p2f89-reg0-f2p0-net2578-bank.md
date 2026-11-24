# kc5r73 solo 判读——刀A 单刀银行轮预注册 7 项全数命中：RESOLVED 0／RESIDUAL 89（win14 22）／非 GT REG 0／F2P 0／净账 2578F 历史最优工作基，REDLINE 仍触发 face 不落库（探针折叠表定谳 expected 产生通道）

日期：2026-11-24 · 流：KC-5-r73（R5 基＋刀A 钳单刀＋刀B 删除＋刀C 闸死）· 判读器 `.acceptance/wptfyi-css/kc5r73/judge.py` · raw：`/tmp/zw-kc5r73-solo-20261124a.log`
注：本笔为 resume-swarm 三 fork 对称收敛合成稿（判读/电池/verify 位各自独立判读，三门数值与预注册对账全同向；按 r72 先例合并单稿，分歧零——收敛记录见 NOTES-kc5r73.md §6＋§8）。

## face 设计与 provenance

r73 face（worktree 13 码文件）＝HEAD d09d301dc ＋ r72 复贴源 face-r72-code-only.patch（md5 675cfa73，REVERSE-OK 验证精确 post-image）＋ 三处外科修改：**刀B 整块删除**（js_anim canonicalize calc-size( 透传臂，r72 净严格负唯一肇事）；**刀C 发射闸关死**（computed_style_cache.rs `const MIX_CONSULT_ENABLED: bool = false` 门住 pair_px 接线，管道/三 lane accessor/peek/合成器/测试整体保留供 r74）；**刀A 保持**（animation.rs width/height 臂 `parse_px(value).max(0.0)`）。行为面目标＝R5＋刀A 同形。cargo check 绿零警告。

三臂 bin：b0 c745bee4（HEAD face，r71→r72→r73 三轮复用）／r71r5 d2ef864b（R5 face，GT 再生臂）／r73 ee308dad（本 face，unit zw-r73-b1-build 03:19:30 落盘）。b1 provenance＝timeline 法证（face 末波 mtime 03:16:46 < build 发射 < bin 03:19:30，窗内零 crates 编辑）＋ face md5 三重同值对质（prebuild 快照 /tmp/zw-r73-face-md5-prebuild.txt＝solo-verify.sh 发射现印＝判读时复核，13 文件逐字节同）。strings 标记通道弃用（r72 假阴性教训）。

## 探针定谳（expected 产生通道，先行落档 .acceptance/wptfyi-css/kc5r73/probe/）

两探针页 114 tests 对 b0 实测 static gCS 折叠表（harness expected＝参考元素 static gCS，interpolation-testcommon.js measure() 通道）：

- **width/height 轴＝真求值**：calc-size(auto, 6px + size*0.7)→146px（auto→CB 宽 200px）；算术含 MIX 加式全求值；intrinsic 关键字（min/fit/max-content）在 calc-size 内→**0px**；auto/stretch→CB 尺寸（W 200px；H 300px，height:auto parent→0px）；% 原样显 %。
- **min/max 轴＝声明 DROP**：calc-size MIX 形声明被 stylesheet 解析丢弃→声明值显出（MW/MH→10px＝stylesheet 声明、XW→150px、XH→100px）。判别器：calc-size(min-content, -45px + size*1.3)→150px（stylesheet 值）非 85px（真折叠值）＝drop 非 fold 铁证。
- GT-89 expected 全折叠 px（10px×48／0px×22／100px×10／150px×9）与本表逐族吻合＝r72 核心勘误（期望恒折叠形、MIX 形只活在断言名）的通道级闭环。

## 电池有效性与预注册判读（2026-11-24 03:30:09 START → 03:37:43 ALL-DONE，SOLO-RC=0）

- 发射形＝resume-swarm 双 fork 仲裁（NOTES §6）：电池位 fork 于 03:23:57 发射 unit zw-r73-solo（systemd --user，MemoryMax 24G，flock 单实例，内置 ZeroWeb-3 drain-wait 实耗 6m12s），判读位 fork 零双发（同名 unit 已载入即止）；SOLO-RC 后判读工件 ~6 分钟内三方齐落，接管条款未触发。
- 三臂 15＋18 页（css-sizing/animation 15＋calc-size/animation 18），PAGE-CENSUS 逐臂逐页同集、PROBE-PAGE-IN-CORPUS 零命中（探针页发射前已核删 find=0）。形状 census 三臂各 7164 键零 ASYMMETRY。电池本体 7m34s 六腿 rc 全 0。
- **GT 再生双口径入带**：full 89 ∈ [84,94] ✓、win14 22 ∈ [20,24] ✓。页级锚点 b0/b1r5 与 r71/r72 电池逐字同（calc-size-width 291P/551F、b1r5 371P/471F，页总 842 断言＝P/F 标注收账时键级核正）＝跨 run 确定性第四演。
- **预注册 7 项全数命中（zero miss，三 fork 判读各自对账同值）**：

| 门 | 预注册 | full 实测 | win14 实测 |
|---|---|---|---|
| GT 再生 | [84,94]／[20,24] | **89** ✓ | **22** ✓ |
| RESOLVED | 0 | **0** ✓ | **0** ✓ |
| RESIDUAL-P2F | 89／22 | **89** ✓ | **22** ✓ |
| 非 GT REG | 0 | **0** ✓ | **0** ✓ |
| F2P | 0 | **0** ✓ | **0** ✓ |
| 恒等式 | GT 口径 ✓ | 89=89 ✓ | 22=22 ✓ |
| css-sizing Δ | −14±2 | **−14** ✓ 恰中 | 同 |
| 净账 | b1 2578F | **2578F** ✓ | 同 |

- **净账 b1 2578F**（b1r5 2592F − 14）＝历史最优工作基（r71 2647F→r72 2703F→**r73 2578F**）。轨迹：刀A 钳三轮（r71/r72/r73）同一 14 键族净正。

## 三通道键级直证（行为面 R5+刀A 同形）

- **css-values 全语料键级 status diff = 0**（b1 vs b1r5 1984P/2129F 逐键全同）＝刀B 删除＋刀C 闸死把 r72 的 REG 35＋F2P 90 全数撤净，calc-size 页行为面精确回到 R5 形（页级印证：calc-size 两页 b1≡b1r5 逐字同 727F/471F）。
- **css-sizing gained=14／lost=0**（b1r5 2588P→b1 2602P，height-interp 8＋width-interp 6，exp `0px` 负 t 端点外推簇）＝刀A −14 页级分解与 r71/r72 同族同页。
- **RESIDUAL 89 结构与 r71/r72 恒等**：MIX 67（min/max 四页 66：MW 24＋MH 24＋XW 9＋XH 9＋which-value 1）＋PURE 端点 16（calc-size-width 14＋height 2）＋composition 5＋PX 1；got 串形族 census auto 33／200px 14／min-content 12／fit-content 9（静态 echo 多数簇，与 r71/r72 同形）。
- multicol P2F 36 防线（min/max 臂 parse_px_strict 未动）保持＝REG/F2P 双零的构造面。

## REDLINE 与 face 处置

**P2F 89>0 → face 不落库**（预注册预期触发＝银行轮性质）。13 码文件 r73 face 驻留 worktree 作 r74 工作基；本轮 commit 仅 docs（pathspec 双文件）。

## r74 交棒面（K-FOLD-EXIT＋K-MIX，探针处方直引）

1. **K-FOLD-EXIT（width/height PURE 14＋2 键，win14 GT 主体）**：canonicalize 出口不产裸函数串，产**折叠形**（与探针折叠表同源）：calc-size 内联 px 基→px 直出（calc-size(50px, size)→50px）；intrinsic 关键字→**0px**（探针实证）；auto/stretch→CB 尺寸（需 substrate 上下文注入，或回落不写让 echo 通道承责——r72 教训：echo/塌 0px 通道正是 125 passing 键承载机制，恢复不写即回该通道）。发射前探针页证明目标通道真 firing＋width 页 6 键 stale 200px echo 的路径分叉定位（r72 遗留疑点）。
2. **K-MIX（min/max MIX 簇 66＋which-value 1 键，full GT 主体）**：探针铁证＝min/max 页 calc-size 声明被 DROP、GT 期望＝stylesheet 声明值（10/150/100px）。刀法＝动画 tick 对 min/max 轴 calc-size MIX 端点**不写值**（让 underlying 显出）——期望通道即声明值，非合成。刀C 管道（三 lane accessor/peek）可复用于端点对识别（开闸 `MIX_CONSULT_ENABLED` 即可），但发射产物从「合成串」改「None/不写」。
3. 域外维持：composition 5（ANIMCOMP）／PX 1-3（tick 负进度 fill）／which-value 1 不接；GT 口径沿用再生口径（b0P∧b1r5F，双带 ±5/±2）；页级锚点 b0/b1r5 与 r71→r73 逐字同可续对质。
4. 协议沉淀：发射闸模式（const 门）是刀搁置不删管道的落法范本；provenance＝timeline＋face md5 多快照对质（本轮三重同值实证零缺口）；发射前探针钉 firing 通道纪律续（防死码三演已立）。

## 下一步

r74＝K-FOLD-EXIT＋K-MIX 双刀同 face（基座＝r73 工作基 R5＋刀A），探针页先行钉 firing 通道，GT 回收路径＝tick/gCS 产出与 expected 同形折叠值／不写值让声明值显出。
