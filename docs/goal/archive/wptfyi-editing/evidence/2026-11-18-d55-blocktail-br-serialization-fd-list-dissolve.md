# d55 块尾 br 序列化判定＋FD 列表壳溶解刀 A/B 证据（2026-11-18）

## 码面（part01.js +198/−28，git numstat；上轮 session 429 中断，本轮回填定谳落树）

1. **`_zwCeSerTailBrInfo` 块尾 br 序列化判定**：block 的 innerHTML 序列化尾
   是 br → 真行尾（引擎权威序——gEditor.innerHTML 反映引擎树，bs2 探针实证
   本地 splice 不入读）。hasVis = br 前缀去标签后残留非空白文本（纯 br 占位
   块 false——br 即占位语义不消费，JoinStartOf 同口径）；解析失败保守
   {tail:false} 维持旧口径。根因：childNodes fused 视图跨案叠 pending ghost
   子（zz-probe-d55a face 188 `fv=B/1@P` 实证：早前 i/b join 案移入的 `<b>`
   壳出现在 p1 尾 fused 视图，`_zwCeBrFirstVisibleAfter` 误判行中 br）。
2. **`_zwCeSubtreeHasBr` 廉价预检**（深度限 8 防病态嵌套）：序列化判据的跳
   过门——fused 视图无任何 br 后代则序列化尾必非 br，直接回落 JoinStartOf
   （＝base 行为）。OPTIMIZATION：引擎 innerHTML 往返每案 ~百 ms 级，jdle
   880 案放大撞文件墙吞键——预检跳过后 jdle/jdla/dil 专项轮全零翻面。
3. **BS 块界 join 缝尾随 br 预消费**（③ join 臂）：bp 为块、序列化尾 br、
   块内有可见内容 → JoinMutate 提取前摘尾 br，gtr 起点落被消费 br 索引（单
   br `<p>abc<br></p>` 界 (p,1) 与 JoinStartOf d44 同值；双 br 形上游删恰好
   一枚保前一枚，界 (p,2)——JoinStartOf 落首个 br 位 (p,1) 欠账即此）。
   d55b 列表容器父门：ul/ol/dl 父域整门关断（li 形双 br 契约＝消费首个 br
   留占位尾 br，base JoinMutate br-eat 既有行为，dil 语料 base 全绿）。
4. **BS/FD 非块壳 caret 解析**（元素锚臂镜像对）：caret 落非块 inline 壳边
   界且壳无前/后子（`<p><span>{}</span><br></p>` marker 摘后空壳形）→ 上溯
   最近块祖先（guard 16）——归下方 K 族整块摘除口径，否则 silent return。
5. **FD 行尾 br 臂序列化行尾判据**（元素锚 walk + 文本锚 walk 两处）：
   `fc` 序列化尾 br → 行尾（消费+join），ghost 子不构成行中内容；非行尾保
   持 BrFirstVisibleAfter 旧口径。
6. **FD d55b 块形后邻门**：join 只收块形后邻（镜像 walk `!isBlock return`
   守卫）——裸文本后邻形（`<div><p><img>{}<br></p>def</div>` 语料 1060）的
   跨容器搬行触发引擎侧池 rebake 毒（d49 FD elem-join 同域：变异自身全对、
   下一案 initializeTest 后锚解析混陈旧代际；前缀二分定谳毒源即 1060），
   回退 N 派发章程（+2F/+2F 持位挂 ③）。
7. **FD dst 归属修正**：行尾 br 消费后 caret 块自身存活（可视内容非空）接
   收后邻首行，dst=caret 块 sc（旧口径 dst=Jc 把行壳落宿主尾错形）；src 块
   提线后全空同摘（`<p>def</p>` 空壳不残留）。
8. **FD 列表壳溶解 join 臂**（`_fdLdLi`，walk 守卫前插入）：caret 行外层后
   邻为 ul/ol 且 li 首子为纯单文本行（`abc[]<ul><li>def</li></ul>ghi` 界
   (abc,3)-(def,0) 语料形）→ li 首行溶入外层流（引擎 takeLine 准入路径，
   跨容器移动 shim 直挂 insertBefore 抛 HierarchyRequest——BS T7/T8 同修）；
   li/空 ul 摘除、ul 后继保留（第二 li 不动）；caret 侧尾随折叠空白同摘
   （normal 域门）。
9. `_zwCeBlockVisuallyEmpty` 补 img/hr 可见原子门（`<p><img><br></p>` 非空
   块契约）；JoinMutate 缝 br-eat 维持 try/catch 原形（wip8 构建源带守卫，
   16:30 后的摘守卫手改无 A/B 覆盖，恢复 HEAD 原形）。

## 根因链（zz-probe-d55a 定谳）

1. **fused 视图 ghost 叠影**：同一文件内早前案的 join 变异在 childNodes
   fused 视图留 pending ghost 子——p1 尾出现早前案移入的 `<b>` 壳（face 188
   `fv=B/1@P`），`_zwCeBrFirstVisibleAfter(br)` 返回该 ghost → 误判行中 br
   → 行尾 br 臂不触发。序列化尾判据读引擎树权威序，ghost 不入序——一刀
   双臂（元素锚 walk + 文本锚 walk）统一换判。
2. **JoinMutate 缝 br-eat 同病**：br-eat 走块尾 fused 视图（face 176
   `dlast=B`），ghost 挡真 br——BS 侧改在 face 内预变异期预消费（此时视图
   干净），JoinMutate br-eat 原样保留兜底。
3. **双 br gtr 起点欠账**：`<p>abc<br><br></p>` JoinStartOf 落首个 br 位
   (p,1)，上游期望删尾 br 后界 (p,2)——预消费臂记录被消费 br 索引即得。
4. **FD 裸文本后邻搬行毒**：`<div><p><img>{}<br></p>def</div>` 跨容器搬行
   wire 序触发引擎侧池 rebake 毒（同页后续案锚解析陈旧代际），d49 同域第二
   负样本——门控回退 N 派发，净 +2F/+2F 挂 ③ generation-aware 归一刀。

## 权威账（专项 A/B，base=HEAD（d54 终态）release /tmp/d55-base2-runner，
wip=d55 终态 release /tmp/d55-final-runner）

| 腿 | base | wip | 共键 F2P | P2F | 备注 |
|---|---|---|---|---|---|
| gt-backspace | 81P (163) | 86P (163) | 5 | 0 | fin-vs-wip8 **163 键逐字节恒等**；release 重建轮（d55r-bs）再证恒等 |
| gt-forwarddelete | 100P (176) | 98P (149†) | 12 | 0 | † fin 轮 300s 尾部修饰键截断；共键 148 F2P=12/P2F=0 与 wip8/release 轮一致 |
| joining-dl（双文件 1176） | 942P | 942P | 0 | 0 | 含 jdla（296 键 216P 恒等） |
| joining-dl-elements（880） | 726P | 726P | 0 | 0 | d55b 列表父门＋预检跳过门零扰动 |
| deleting-in-list-items（1264） | 504P | 504P | 0 | 0 | 同上 |
| gt-non-collapsed（207） | 110P | 110P | 0 | 0 | |
| input-events 全套件 | 3507P（共键 6318） | 3541P | 34 | 0 | F2P 面＝bs 5 形 ×4 variant＋fd 14 面；fin/release-vs-wip8 共键 4075 全恒等 P 2313 双侧 |
| selection 全语料（53,312 共键） | 49,025P | 49,025P | 0 | 0 | gone=5/new=1 stringifier click 族带伪影（d54 同谱） |
| editing/run 回填（8 分片） | 90,360P（共键 91,311） | 90,360P | 0 | 0 | only-in-base=9（全 Timeout 截断桶）/only-in-wip=4690（base 侧文件级截断桶在 wip 全展开，Pass 4497＋Fail 193），共键零位移 |
| enginelib | — | **4187P/0F**（release 重建轮再跑 **4205P/0F**，计数漂移＝并行流 pin 自增） | — | — | d54 记档的 r2761 font 简写红已被并行流清偿 |

**刀净 +17（bs +5、fd +12；F2P=17/P2F=0）**。fd 12 面分解：序列化行尾判据
+ dst 修正 6 面（`<p><img>[]<br></p>`、`<p><img>{}<br></p><p>def</p>` ×2、
`<p>abc<br>{}<br></p><p>def</p>` ×2、`<div><img>{}<br><hr>def</div>`）＋
列表壳溶解臂 6 面（`abc[]<ul>…` 三形 content+gtr 双键）。bs 5 面分解：缝
尾 br 预消费 4 面（单/双 br content+gtr）＋非块壳 caret 解析 1 面
（`<p><span>{}</span><br></p>` content）。

## 哨兵

- selection 全语料 53,312 共键 F2P=0/P2F=0（见上表）。
- enginelib 4187P/0F（release 轮 4205P/0F；test-guard --per-proc-mem 4
  --total-mem 8）。
- editing/run 全量回填：8 分片 ×2 侧共键 91,311 键 F2P=0/P2F=0 双侧
  90,360P 恒等（分片账：sh1 5702/sh2 7065/sh3 13602/sh4 9813/sh5 8766/
  sh6 14849/sh7 16608/sh8 14906，F2P/P2F 全零）。

## 回填轮次勘误（本轮定谳记录）

- 回填轮 1（`/tmp/d55-er-sh*-base/wip.json`）全 0 字节：脚本引用
  `/tmp/d55-base2-runner`、`/tmp/d55-final-runner` 于 runner 拷贝完成前启动
  （test-guard 起进程即失败），产物作废。
- 回填轮 2 wip 侧（`/tmp/d55f-er-sh*-wip.json`）用 `/tmp/d55-wip-runner`
  ＝13:32 早迭代 debug 二进制（非 d55 终态）；其与轮 3 终态 wip 在 sh1 共键
  5759 F2P=0/P2F=0——editing/run 面 d55 早迭代与终态无行为差，互证。
- **权威对**＝轮 2 base（release HEAD d55-base2-runner）× 轮 3 wip（release
  d55-final-runner，`/tmp/d55g-er-sh*-wip.json`）。release 重建轮
  （d55r-*）在 bs/fd/joining/dil/nc/suite 六腿对 fin 轮全共键恒等
  （F2P=0/P2F=0），二进制换代伪影排除。
- fin 轮 selection 腿（rel-verify 链内）3000s 墙钟 rc=124 截断作废，哨兵以
  wip8 轮 `d55-selection-{base,wip8}.json` 全语料对拍为准。

## 残留与归域

1. **③ generation-aware 域**（挂账累计）：FD 裸文本后邻形 +2F/+2F（本刀
   门控回退章程入册，毒机理 d49/d55 两负样本同域）；d50 hr 形派发鬼、d51
   列表面派发分型、d48 B3 占位 br、8 键 `[list-item1]` 全选形等旧账不动。
2. **runner 文件级超时吞键族带**：fin suite/joining/dil 轮在并发负载下截
   断（gone=2487/338/112）——共键账与 wip8 全键集账互证，伪影不入账。
3. **⑩ CE-island li/dd host WIP 未上树**：上轮 session 429 中断前
   （23:21）在 arm1 sepTag 缺省行与 arm1 入口门留下 li/dd host 直子文本
   中点 caret 拆分 hunks（editing-compat ip I4 族方向）——未构建未 A/B，
   本轮落树前剥离（part01.js 恢复与 release 二进制内嵌 shim 逐字节一致），
   hunks 存 /tmp/d55-shi-wip-hunks.txt 待单独验证。
4. ①(b) 跨元素边界 caret join、①(c) `{<hr>}` 核拒收、④ B3 text-control
   期望矩阵、⑤ DC-3 周跑快照对账照旧挂账。

## 运行产物

- /tmp/d55-{bs,fd,joining,dil,nc}-base.json ＋ /tmp/d55-{bs,fd,joining,dil,
  nc}-wip8.json（wip8 轮）；/tmp/d55-fin-{bs,fd,joining2,dil2,nc,suite}.json
  （fin 轮）；/tmp/d55-suite-base.json、/tmp/d55-suite-wip8.json（suite 对）
- /tmp/d55f-er-sh{1..8}-base.json（轮 2 base）× /tmp/d55g-er-sh{1..8}-wip.json
  （轮 3 wip）＝editing/run 权威对；记账脚本 /tmp/d55-er-merge.py
- /tmp/d55-selection-{base,wip8}.json（selection 全语料对）、
  /tmp/d55f-selection.json（轮 2 复跑）
- /tmp/d55-final-enginelib.log、/tmp/d55g-enginelib.log（enginelib 两轮）
- 二进制：/tmp/d55-base2-runner（release HEAD stash roundtrip）、
  /tmp/d55-final-runner（release d55 终态，内嵌 shim 与落树 part01.js 逐字节
  一致）；/tmp/d55-wip{2..8}-runner（迭代轮 debug）；探针
  zz-probe-d55a.html（wpt-data 本地不入树，9 variant 计数面）
- 关键脚本：/tmp/d55-flips.py、/tmp/d55-keysets.py、/tmp/d55-rel-build.sh
  （release 重建 sequencer）、/tmp/d55-rel-verify.sh
