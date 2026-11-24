# kc5r72 solo 判读——刀②透传净严格负＋刀③门域自锁打错靶：RESOLVED 0／RESIDUAL 89／非 GT REG 35／F2P 90，REDLINE 双触发 face 剥除（核心勘误：GT 期望全折叠 px，MIX 形只活在断言名）

日期：2026-11-24 · 流：KC-5-r72（R5 基＋三刀：Delta C 钳回写／canonicalize calc-size( 端点透传／calc-size MIX 串咨询）· 判读器 `.acceptance/wptfyi-css/kc5r72/judge.py` · raw：`/tmp/zw-kc5r72-solo-20261124a.log`
注：本笔为 resume-swarm 双 fork 同轮收敛仲裁稿（02:3x 两 fork 各判一轮后按 JSON 键级 census 合并；冲突点及裁决见 NOTES-kc5r72.md §9）。

## 电池 provenance 与有效性

- 三臂三 bin：b0-runner md5 c745bee4（HEAD face，r71 复用）／r71r5-runner md5 d2ef864b（纯 R5 face，GT 再生臂，r71 复用）／r72-runner md5 aebcbcd9（R5＋r72 三刀，判读臂）。b1 provenance 双通道：timeline 法证（face 末波 mtime 02:17:18 < build unit 02:23 < bin 02:24:11，窗内零编辑；六 face 文件发射前后 md5 逐字节同）＋运行时 marker（`zw-calcsize-mix-r72-2026-11-24` strings 命中 1——NOTES §3 strings 通道教训后改 tracing::debug! 运行时引用保全 .rodata）。
- 电池 15＋18 页窗（css-sizing/animation 15 页＋calc-size/animation 目录 18 页）× 三臂，单实例锁串行，02:24:27 START → 02:33:25 ALL-DONE，rc 全 0。PAGE-CENSUS 三臂各 15＋18 页逐页同集（ONLY_SUBSTR 未落空）。跨 run 确定性第三演：b0（2092P/959F＋1758P/2355F）与 b1r5（2588P/463F＋1984P/2129F＝2592F）页级账与 r71 电池逐字同。
- GT 再生入预注册带：full 89 ∈ [84,94] ✓、win14 22 ∈ [20,24] ✓＝电池有效，判读为 face 真红非环境伪红。形状 census 三臂各 7164 键零 ASYMMETRY（完备性对称）。
- NOTES §4 页数口径勘误：预注册写「15+2 页」，实为 15+18（calc-size/animation 目录全量，win14 判读器侧滤 R70_CSS_VALUES_INCLUDE 两页保口径）——与 r71 电池 18 页窗一致，非缺陷。

## 三门判读（REDLINE 双触发：P2F 89>0 ∧ F2P 90>0）

| 门 | r71（R5＋三 delta） | r72（R5＋三刀） | 预注册带 |
|---|---|---|---|
| RESOLVED | 0 | **0**（full）／0（win14） | [50,62]／[10,16] **全空** |
| RESIDUAL-P2F | 89 | **89**（full）／22（win14） | [27,39] full 口径外溢 |
| 非 GT REG | 0 | **35** | 0-2 **爆带** |
| F2P | 69 | **90** | 0-2 **爆带** |
| GT-missing | 0 | 0 | 恒等式 89=89／22=22 双 ✓ |

净账：b1 2703F vs b1r5 2592F＝页级 +111F（劣于 r71 2647F；历史最差仍 r68 2812F）。face 已剥（13 码文件 checkout HEAD，复贴源 face-r72-code-only.patch md5 675cfa73 apply --check OK，全树快照 face-r72-worktree-snapshot.patch md5 17eb0bc0 仅档案；姊妹流 html-semantics-compat.md 未卷）。

## 核心勘误：期望形前提系统性误读（两 fork 独立复核一致）

r71 硬锚表的「calcsize-MIX `calc-size(KW, Npx + size * t)` 66 键」**把 subtest NAME 内嵌的 from/to 声明串误读为期望形**。本轮键级实证（b1r5 fail message exp 字段全量提取）：GT-89 的 expected 形族＝`10px`×48／`0px`×22／`100px`×10／`150px`×9——**全数折叠 px，零 MIX 串**。MIX 形只出现在断言名（如 `from [calc-size(auto, size * 2)] to [50%] at (1) should be [0px]`——名内声明形 ≠ 期望值；期望恒经参考元素 static gCS 折叠为引擎自身输出形）。r71「MIX 方向代数残差 60px/15px/40px 全对上」核对的是名内声明串形状，不是 expected——假锚上的合成公式再准也回收不了任何键。

## 三刀键级归因

**刀 A（Delta C width/height 钳回写）＝兑现保持零损伤**：css-sizing 463F→449F（−14），r71 的 14 增益（exp `0px` 负 t 键）全保持（width-interp 6＋height-interp 8，逐键同族）；css-sizing 零 REG 零 F2P＝multicol P2F 36 防线（min/max 臂未动＋裸 KW 对门）保持。连续两轮实证净严格正，唯净正项。

**刀 B（canonicalize calc-size( 端点透传）＝净严格负，REG 35＋F2P 90 唯一肇事**：REG 35 全在 calc-size-height（33）＋calc-size-width（2）；F2P 90 全在 calc-size-width（49）＋calc-size-height（41）。got 形全族＝裸 `calc-size(...)` 串（`calc-size(50px, size)`×43／`calc-size(any, 50px)`×20／`calc-size(auto, size)`×7／`calc-size(min-content, size)`×8 等），exp 全为折叠形（`0px`／`50px`／`200px`）。**「透传恒端点形零伪源」前提证伪**：got 侧是直接串比较，exp 经参考元素 static gCS 折叠（fold(calc-size(50px, size))=`50px`、fold(auto)=`0px`）——裸函数串永不等折叠 px；被绕开的塌 0px／echo 通道正是 125 个 passing 键的承载机制。PURE 端点键零回收：width 页 14 GT 键中 8 键 got 由 `200px` 变形为裸 `calc-size(fit-content/min-content, size)`（仍 F，exp `0px`）＋6 键 stale `200px` echo 未变＝透传触达面页内分叉（stale echo 源未经 canonicalize 路径，待探针定位）；height 页 2 GT 键未变。

**刀 C（calc-size MIX 串咨询）＝发射链活了但门域自锁打错靶（0 回收＋带外损伤）**：双层事实（resume-swarm 两 fork 数据合成）——① 发射链非死码：127 键 got 含 `size *` MIX 串（探针页防死码目标达成，r71 Delta B 死码已修复），但 127 键全部落在 `prop∈{width,height}` 门域内（calc-size 两页非 GT 键），**0 键属于 GT-89**、127 全 FAIL（合成串天生 ≠ 折叠 expected，损伤面入 REG 4＋ 键如 `calc-size(auto, size * 3)`）；② GT MIX 域 67 键（min/max-width/height 页 66＋which-value 1）got 与 r71 residual 67/67 逐字节同（静态 underlying echo）＝门谓词 `prop∈{width,height}` 与 GT MIX 键分布（100% 在 min/max 页）**结构性不相交**，自锁致 GT 域零发射。双层合并即：链路通、门锁向、且**门解锁也不回收**——min/max 页键的 expected 同为折叠 px（exp census 覆盖全部 89 GT 键），扩门只会把 MIX 串喷向 px 期望键＝复演本轮 125 键反噬。RESIDUAL 89 结构与 r71 硬锚表同形：MIX 67＋PURE 14＋composition 5＋PX 3＋which-value 1（PX/OTHER 归页与 r71 表微差，总量恒等）。

## r73 交棒面

1. **刀 B revert（透传臂），「折叠形出口」重设计**：canonicalize 出口不产裸函数串，产折叠形（calc-size( 内联 px 字面直出 `50px`、KW/auto 端点按参考折叠语义求值）。发射前探针页须先证 got 通道（含 width 页 6 键 stale `200px` echo 的路径分叉定位）。
2. **刀 C 门域扩 min/max＝本笔证据下禁止**（exp census 全折叠 px，扩门＝复演反噬）；consult 基建（pair consult/handle 接线/peek 探针）留管道复用，MIX 字符串合成目标废弃。
3. **方向重定**：GT 回收路径＝让 tick/gCS 产出与 expected 同形折叠值。探针页先行：钉死 b0 对 static calc-size 元素的 gCS 输出形（expected 产生通道定谳），再决定插值算术落点。
4. 刀 A（Delta C 钳）保持，连续两轮 +14 零损伤，r73 可单刀复贴先行（css-sizing 全谱 P2F=0 门禁）。
5. 发射链探针页常态化：发射前现地证明 peek→合成→serialize 全链在目标页真 firing（防死码三演）。
6. composition 5（ANIMCOMP）／PX 3（tick 负进度 fill）／which-value 1 域外维持不接；GT 口径沿用再生口径（b0P∧b1r5F，full/win14 双带），页级锚点 b0/b1r5 与 r71 逐字同可续对质。

## 下一步

r73＝revert 刀 B＋折叠形出口探针页先行（b0 static calc-size gCS 输出形定谳）＋刀 A 单刀复贴评估；MIX 合成路线废弃（门域扩面禁止）；发射前探针证明目标通道真 firing。
