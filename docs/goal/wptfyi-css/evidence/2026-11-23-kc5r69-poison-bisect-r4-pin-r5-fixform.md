# kc5r69 毒点组二分收官——毒点钉死在 animation.rs apply_single_property 委托 hunk，R5 委托＋拒收回落＝两全修复形

日期：2026-11-23 · 流：KC-5-r69 bisect（接续 executor 流 r68 判后切割权）· 单实例锁 /tmp/kc5r69-bisect.lock
账本：`.acceptance/wptfyi-css/NOTES-kc5r69-bisect.md` · 判读器：`kc5r69-judge-v2.py`（GT 页名 .html 修正版＋solo-r68 b1 基线 fail-set 差分）

## 背景与切割协议

r68 臂 A 判读 verdict-red（RESIDUAL-P2F 110 / 非 GT REG 110 / F2P 0）后，r69 执行毒点组二分：每臂单变量剥一组 face hunk→独立代号 bin→两页探针（css-sizing min-height-interpolation＋width-interpolation）→GT 键 silenced 判定（成功判据＝GT 键发射静默化 deferral→b0 Pass，非 echo 复活）。face/臂 A 驻留态不动（手术只动 worktree，`git checkout -- crates/engine/src/animation.rs` 收臂复位 index 臂 A 面 8265bda75）。

## 法证前置（改变毒点图的两个发现）

1. **apply.rs face hunks 不在 worktree**：worktree blob==HEAD（90d71104），face hunks 仅在 index；mtime 04:43 早于 r66 以来全部 build ⇒ r66 以来全部电池/探针 bin 均不含 apply.rs face hunks。09:5x 笔「毒点=apply 委托 hunk(apply.rs)」归因错文件，本流推翻。
2. **GT got 形重分组**：min-height 0px→100px at(-0.5) 期望 0px（非负截断），got '10px '（CSSA/WA，＝测试页静态底层值）或 '100px '（Transitions，＝cascade 已落新值）；width 1em→10vw at(-0.3) 期望 0px，got '16px '（=1em 解析值=from 端点回声）。三形＝**负 progress 截断族＋端点回声族**，非 calc-size 复合串族。

## 二分结果（R1→R5）

| 臂 | 剥除对象 | GT silenced | fail-set vs arm A 基线 | 判定 |
|---|---|---|---|---|
| R1 | js_anim canonicalize hunks | 0/5 | min-height 39=39 恒等；width broke=3（stretch→auto 新败） | 负 |
| R2 | Clock 域门 sizing 六属性行 | 0/5 | 两页逐键恒等（39=39、155=155）零运行时足迹 | 负 |
| R3 | LP 臂三 hunks（compose 加法＋lenient 插值＋min/max 归位） | 0/5 | min-height 恒等；width broke=6（10px→100% 混合对＝lenient lane 在径的活体证据） | GT 负 |
| R4 | apply_single_property 六长手委托 hunk→HEAD 原生臂 | **5/5** | min-height 250P/116F==b0 完全同值 healed=4 broke=81；width healed=16 broke=124 | **毒点钉死** |
| R5 | 委托保留＋拒收回落（修复形） | 5/5（width GT 键除外，见挂账） | min-height healed=4 **broke=0**；width healed=12 **broke=0** | **修复形成立** |

bin 代号与 md5：r69r1 4eb46082（前 session 遗产）/ r69r2 b0b545c5 / r69r3 5166d06e（strings parse_lp_terms=1＝剥除二进制级兑现）/ r69r4 185b867e / r69r5 0feac854。

## 毒点机制链（静态＋实证交叉定谳）

harness（interpolation-testcommon.js）at() 恒经 delay=-duration/2＋createEasing cubic-bezier 近似＋currentTime=50000 ⇒ 插值 lane 收到 t=easing(0.5)=**at 本身（可为负）**；期望值＝线性外插＋非负钳（css-values-4 §8.6 / css-sizing-3 propdef <length [0,∞]>）。tick 产出负外插串（如 "-50.00px"）→fallback 检测器不触发（非端点）→采纳→**canonicalize apply 段决定生死**：

- HEAD 原生臂：width/height 宽松 parse_px、min/max 严格臂负值照样落槽→serialize 计算期非负钳→"0px "==期望→PASS。
- r3 fame2b 委托臂：`apply_single_property` 六长手→style-system `apply_property_value`，静态有效性门负值拒收槽位不动（注释自证「宁缺勿错」）→ser1==ser0→canonicalize None→底层回声 '10px '/'16px '/cascade '100px ' FAIL。

三个 got 形由同一 hunk 统一解释。R4 预注册预言（5/5 silenced）发射前落 NOTES、结果全兑现＝机制定谳非事后拟合。

## R5 修复形（r70 face 候选）

委托臂保留（非负域照常受理 calc-mix/keyword，保住 fame2c 增益），`apply_property_value` 返 false 时回落 HEAD 原生写槽臂（width/height 宽松、min/max 严格＋parse_max_sizing_anim_value）；负外插值走回落→落槽→serialize 钳 0。探针实测：R4 裸剥的 205 broke 全部消除，GT clamp 族 silenced 保全，另白得 16 键回收（min-height 4＋width 12）。预注册预言 ①②③ 发射前落 NOTES，结果全兑现（③ 更优：fail-set 143≠155，差异方向为 recovered 非回归）。

## 挂账（域外，如实记录）

width 1em→10vw at(-0.3) GT 键在 R5 下仍 FAIL got '16px '：离散翻转采纳的 raw "1em" 端点被静态 applier 的 em 受理落槽（16px）。真修＝插值通道单位解析（em/vw→px 按元素上下文解析成 px 再 lerp），不入本刀。

## 下一步

solo-r70 全量电池（14 页×双臂，b1 显式绑 r69r5-runner，脚本 kc5r70-solo-verify.sh）对 GT-110/非 GT REG/F2P 三门验账；三门达标则 r70 face 落库提案（含 GT 1 键域外挂账声明）。
