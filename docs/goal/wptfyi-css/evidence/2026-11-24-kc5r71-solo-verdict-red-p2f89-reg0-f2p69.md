# kc5r71 solo 判读——案 2 主刀＋F2P 钳全红：RESOLVED 0／RESIDUAL 89／非 GT REG 0／F2P 69，REDLINE 双触发 face 不落库（Delta A 净负、Delta B 死码、Delta C 兑现）

日期：2026-11-24 · 流：KC-5-r71（R5 基上案 2 pair 折叠 lerp＋六长手透传＋回落钳同 face）· 判读器 `.acceptance/wptfyi-css/kc5r71/judge.py` · raw：`/tmp/zw-kc5r71-run{1,2}-20261124.log`

## 电 provenance

- 三臂三 bin：b0-runner md5 c745bee4（HEAD face＋sibling WIP 同载）／r71r5-runner md5 d2ef864b（纯 R5 worktree，GT 再生臂）／r71-runner md5 df3a72ce（R5＋r71 三 delta，判读臂）。bin 代际：face 编辑（rustfmt 01:12:47＋clippy 显式 deref 修复 01:13:54）全早于 b1 bin（01:15:13）；deref 修复 codegen 恒等不构成行为变量。
- 电池 18 页窗（css-sizing/animation 15 页＋calc-size/animation 18 页中同目录 18 页）× 双臂，systemd unit zw-kc5r71-run{1,2}（MemoryMax 16G），run 1（b0＋b1r5）01:17:31 ALL-DONE，run 2（b0＋b1）01:29:03 ALL-DONE，rc 全 0。跨 run 确定性：run 2 b0 页级与 run 1 逐字同（css-sizing 2092P/959F＋css-values 1758P/2355F）。结果快照 results-{b0-run1,b1r5,b0,b1} 于 `.acceptance/wptfyi-css/kc5r71/`。
- 页级锚点对质 r70 逐字同：calc-size-width b0 291P/551F、b1r5 371P/471F；min/max-interp 四页 b1r5 fail 28-35；height/min/max-height-composition 三页聚合 b0 69P/111F、b1r5 155P/25F（r70 证据「height-composition」行＝三页聚合口径，单页 60 键＝6 组×5 t×2 lane 与页文本硬证一致非截断）；min-height-interp b0 250P/116F（r69 同）。

## GT 口径（与 r65-r70 轨迹的分歧，如实记）

r65-r70 的「GT-110」＝r6 时代预注册固定键单（r6-p2f-groundtruth.txt，工件已被清库，本机三 clone 均无），本轮不可逐键对表。本轮 GT＝当日经验再生（b0 Pass ∧ b1r5 Fail）：full 89／win14 22。再生口径与固定键单的差集不可重建；轨迹对照（r70 RESOLVED 20→r71 0）带口径 caveat。GT 形族与 r68 考古 r5 形逐字同（exp '0px' vs got '200px' ×14＝calc-size-width），face 面连续性可证。

## 三门判读

| 门 | r70（R5 形） | r71（R5＋三 delta） | 方向 |
|---|---|---|---|
| RESOLVED | 20（固定键单口径） | **0**（再生口径） | GT 键一个未回收 |
| RESIDUAL-P2F | 90 | **89**（full）／22（win14） | 持平（口径分歧下） |
| 非 GT REG | 0 | **0** | 保持 |
| F2P | 4 | **69** | 新代价，单 delta 肇事（下节） |
| GT-missing | 0 | 0 | 恒等式 89=89／22=22 双 ✓ |

**REDLINE 双触发（P2F 89>0 ∧ F2P 69>0）＝face 不落库**（fail-stop）。净账：b1r5 2592F → b1 2647F（−69 F2P＋14 增益）。R5 形（b1r5）仍是历史最优工作基。

## delta 归因（键级硬证，r72 交棒核心）

- **Delta A（canonicalize 六长手透传）＝69 F2P 唯一肇事**（css-sizing 44＝41 CSSA＋3 WA；css-values 25＝23 CSSA＋2 WA，全在 interpolate-size 页）。三机制形：① `fit-content`→`fit-content(100%)` mangle 本是 gCS 序列化正确形（harness exp 经参考元素折叠同形），裸 token ≠ exp——r70 勘误四「mangle 断根 9 键」为真但 69 键反噬；② `stretch` 裸字面 ≠ exp `0px`（r5 echo 折叠形）；③ KW↔KW 翻转字面钉死（got `fit-content` vs exp `max-content`）。**净严格负收益，r72 首刀全 revert**。
- **Delta B（pair fold-lerp consult）＝死码，全程未发射**：GT-89 中 74 键 b1 got 与 b1r5 逐字节同（15 键形变全为 Delta A）。死链双重：① 接线只在 selector 键两 consult 位（callbacks.rs :2583/:2740），WPT harness target 全是 createElement 产物无 `__zwSelector` → gCS 走 `__zw_get_computed_style_by_handle`（js_dom_shim/part01.js:15799）→ handle 路径 consult 链无 pair 域；② 动态 target 的 animate() 注册条目 sel=''（handle 键），selector 键 query 结构性匹配不上。r72：`query_sizing_interp_pair(None, Some(hnd), prop)`＋handle 视图折叠变体，接线 `query_override_for_handle` 同位。
- **Delta C（width/height 回落臂非负钳）＝兑现且零损伤**：`-15px` 形全库清零（r70 F2P 形根除）；14 增益（height-interp 8＋width-interp 6，exp `0px` 负 t 键翻 P）；8 键 `-15px`→`0px` 仍 FAIL（exp `300px`/`50px`/`200px`＝非 0 t 点，stretch→calc-size 区间值整体错——transition discrete 门域，b0=fail 非 GT 非 REG 中性）。

## r72 交棒面

1. revert Delta A（js_anim_registry canonicalize 六长手透传 hunk）。
2. Delta B 重接线 handle 路径（死链两处；发射前加「consult 已发射」探针页防再死码）。
3. cssa clock 端点 accessor（原 r72 主刀不变——GT-89 中 68 键 CSSA lane 结构外）。
4. discrete 细判：KW↔cs spec 可插值对（css-sizing-4）不进 discrete 门；multicol P2F 36 防线按端点形态保留，不全局拆门。
5. width '16px ' 单位解析刀维持挂账（'1em→10vw' 键本轮 exp 形 `0.00px` 小数——折叠器整数 i64 形待对齐）。
6. GT 口径：固定键单已失，再生口径成为后续轮事实标准（页级锚点逐字可对质 r70），固定口径仅存历史轨迹参考。

## 下一步

r72＝revert Delta A＋Delta B handle 重接线＋cssa accessor＋discrete 细判，同 face 四刀；发射前探针页证明 pair consult 已发射（防死码二演）。
