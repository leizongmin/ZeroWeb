# KC-5-r4＋fame2b 电池 verdict-red：P2F=557 REDLINE fail-stop——刀 1「echo 优先」证伪：rect 不携带动画帧值，echo 优先反噬 consult 正确车道

时间：电池 2026-11-22 22:35:56 发射（unit zw-knife-kc5r4-fame2b）→
23:04:28 EXIT rc=0（八腿全 rc=0：contain/sizing/values/transitions
b0+b1）；发射流 session 1f26333c 于 22:55 触 5h rate-limit 上限死亡，
电池由 systemd unit 存活跑毕，本文件为轮转新 session 2026-11-23 00:2x
的判读收账（开工三件套核证：无在飞构建/unit、树态＝face 发射态残留
＋es_module unstaged ride 原态、HEAD d150322ce 后 pull rebase 至
c22c3c0de 零域内冲突）。

face：11 文件 patch `knife-kc5r4-fame2b-face.patch` md5
`f0b44fca14d21d640bd2c5cf330a167b`＝r3 patch 745a6c4d 复贴＋刀 1
echo 对称化三 hunk（H1 callbacks.rs cache-hit 臂 used_size_echo 物理前
置、H2 miss 臂 width/height 借用早退、H3 computed_style_cache.rs
handle 通道 `echo_wh.or(consult)` 门序切换；门面＝prop width/height 二
值，min/max 等 prop 次序不变）。b0-base commit d150322ce；bin
b0 `017d81194…`／b1 `c5a09813…`。判读后 face 已剥（`git apply -R`
REVERSE-CHECK-OK，树回 HEAD＋ride 原态逐项核证），patch 留
.acceptance 存档（**不复贴**——见下文 r5 方向：刀 1 读序翻转弃用）。

## 判定

**REDLINE 触发（TOTAL P2F=557＞0）＝ fail-stop，face 不落库，tally
不动。** r3 池（F2P=689 挂账）维持原状挂 r5。

## RECONCILE 官方账（键级）

- css-contain：F2P=0 P2F=0（域外零损持位）。
- css-sizing：F2P=396 P2F=342（r3: 442/134）。
- css-values：F2P=227 P2F=207（r3: 247/201）。
- css-transitions：F2P=0 P2F=8（r3: 0/0——**新红**）。
- TOTAL：F2P=623 P2F=557（r3: 689/335）。

键级 diff 对 r3（case-name keyed，非页级净额）：

- sizing：new P2F 218（width-interp 91＋height-interp 85＋height-comp
  26＋width-comp 16）／resolved 10；F2P lost 52（width-comp 23＋
  height-comp 19＋width-interp 10）／new 6。
- values：new P2F 41（calc-size-h 20＋calc-size-w 10＋interp-size-h 9
  ＋interp-size-w 1＋which-value 1）／resolved 35；F2P lost 76／new 56。
- transitions：new P2F 8（changing-while-transition-001..004 各 1＋
  shadow-root-insertion 1＋transition-duration-shorthand 1＋
  all-interpolates-same-as-explicit-property 2）。
- contain：零变动。

## 主发现：刀 1 设计假设「rect 已含动画戳记值」被电池证伪

d150322ce r4 方向精化 ③ 的机制前提是动画值经 apply 通道进 layout 后
rect 即动画帧真值，故 width/height 门内 echo 优先于 consult。电池四
类损伤共享同一反证——**rect 在三类动画车道下均停在静态值，从不携带
动画帧**：

1. **transitions 车道 in-flight 插值**：got＝恒定端点值（height
   from neutral to 20px 全 at 点 got "20px"；width 形 "40px"）＝rect
   停在 transition 目标态；r3 consult 返回 in-flight 插值串（7px/10px/
   13px…）是对的，被 echo 优先反噬（含 css-transitions 独立 suite 的
   changing-while-* 4 键＋shadow-root-insertion 等 8 键新红）。
2. **cssa/WAAPI 车道 px/关键字轴**：got＝恒定 10px＝`.target` 底层
   `height:10px`（fixture 两段声明后者胜）＝静态底层值；且
   **Web Animations 车道 r3/r4 双侧同形 got=10px**（刀 1 根本未改变
   该车道读数）——r3 该车道靠 consult（WAAPI 预合成链）返回值，r4
   echo 优先后双路都只是静态 rect。
3. **composition 车道**：`underlying [50px] from add [100px] to
   add [200px]` r3 全 at 点 pass（consult 合成插值），r4 got＝50px/
   100px＝静态底层（height-comp new P2F 26＋width-comp 16 的全部门形）。

结论：**动画值→style→layout→rect 的 L2 同步在 gCS 读路径上不存在**
（或存在但 runner 不 flush）；探针 A1 的 b0 0px 是 cssa 预合成 add 关
键字解析败→'0px' 串→写串巧合链，不是「apply 已同步 layout」的通证。
echo 优先在动画元素上只能读到静态值＝先手毒化 consult 正确车道
（r3 已对的 ~250 键 P→F），而 r3 残面（关键字轴 fold 缺口）原样
F→F（sizing 残留 124＋values 残留 166）。**r4 net：F2P 689→623 但
P2F 335→557＝净损伤。**

## r5 方向（本轮收账同时精化，取代 d150322ce 修订序的 ①）

刀 1 读序翻转弃用（echo 优先=静态毒源）；r3 patch 745a6c4d 仍是复贴
基。主攻改 **consult 侧端点解析＋折叠（刀 K-fold）**，读序不动：

1. **关键字轴 discrete＋静态折叠**：consult 命中且 canonical 串为
   sizing 关键字（auto/initial/unset/min-content/max-content/
   fit-content/stretch）∩ prop width/height ∩ 元素渲染中 → 不返
   字面串，改走静态 used-size 折叠（与 harness 参考元素静态折叠同源
   ＝CSSOM-1 §resolved-value width/height）。此为 r3 主力 P2F 形
   （transitions 车道 from [initial]/[auto]/[max-content] got 字面串
   vs expected 静态折叠 0px）的正修，也是 cssa 预合成 '0px' 巧合链
   的原则化替换（同值不同源，非空内容页从巧合变真值）。
2. **inherit/unset(inherit 形) 端点解析**：inherit 解析为父 used px
   后进 px 插值（`from [inherit] to [20px] at(0.3) = 146px` 形），
   非 discrete。
3. **负外推钳 0**：height/width 不可负，`from [inherit] at(1.5)
   = -70px → 0px` 形（r3 B 层 16 键），入 px 插值出口。
4. 前置 probe（r5 探针轮）：① used_size 折叠机制可否参数化假想值
   （现 used_size_for_selector 折叠的是当前静态值；需 fold(keyword,
   element) 变体——engine js_dom_bridge 域）；② inherit 端点父值
   解析通道现形；③ 钳 0 出口位置（registry 串化层 vs 委托臂）。
5. 排队不变：composition % 基 8（r3 C 层）、γ at(1) 1、values
   calc-size/interp-size 族（关键字轴同族，K-fold 覆盖后复判）。
   L2 全量同步（动画值→layout，echo 优先前提真实化）为深水区备选，
   需 layout 通道改造，非 r5 首选。

## 完整性

S1/S3 ride 窗内零扰动（crates-dirty-s1/s2＝es_module 单件 ride）；
八腿 rc=0 无 LEG-EMPTY；per-page 双侧断言数对称非截断（键级 diff 基
于完整 JSON）；det-triple 不跑（无落库面）。电池产物
`results-kc5r4-fame2b/`（b0/b1 JSON＋log＋bin md5）存 .acceptance。
