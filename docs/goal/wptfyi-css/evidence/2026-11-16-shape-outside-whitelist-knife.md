# shape-outside 门线刀——白名单五站 + 4 值 edge-offset 位置 + supports 全局关键字附带臂

日期：2026-11-16（十九轮，wptfyi-css；rate-limit 双 spawn 对称收敛第三例——
twin 实例 06:06 5 小时限额中断，WIP 静默 >2h 无提交，本实例按磁盘终态仲裁
认领收口：孤儿核查（无进程存活、无在飞 unit）→ WIP 完整性审读 → 最终态
（twin b1v2 批后追加的两臂）重建重跑权威 A/B）。

## 改动面（12 文件 +265/−5，es_module.rs s419 peer WIP 恒不入按惯例排除）

- **style-system 五站**：`types.rs`（`ShapeOutsideComputedValue` 三变体
  None/Shape/Box + `ShapeOutsideBoxKind` 四关键字 + `PropertyValue` 变体——
  独立 wrapper 不用 ClipPathValue 别名，文法多裸 `<shape-box>` 形不波及
  clip-path paint/interp 面）+ `registry.rs`（initial_value 表 +
  known_properties 白名单 `"shape-outside"` 两处）+ `computed_style.rs`
  存储槽 + `default_impl.rs` 缺省 + `inherit.rs` initial 臂 +
  `apply_advanced.rs` apply 文法门（`<basic-shape>` 复用 parse_clip_path
  裁决单点 + 裸 `<shape-box>` 四关键字独立受理，大小写不敏感；
  FIXME `<basic-shape> <shape-box>` 后缀组合形未建模，corpus 现值均裸形态）。
- **engine 三站**：`js_dom_bridge/computed_style.rs` gCS 串化臂
  （`shape_outside_to_css`：Shape 复用 clip_path_to_css、Box 关键字原文、
  none→"none"）+ `transition.rs` get 臂（clip-path canonical 比较同型）+
  `js_dom_bridge_tests/part03.rs` shape-outside 钉（四关键字/非法拒收/
  缺省 none，56 行）。
- **css-parser 刀 B（4 值 edge-offset）**：`parse_extended_border.rs`
  `parse_position_pair` 4 值形（css-values-4 §position：首词定轴序、同轴
  双对拒收、right/bottom 偏移 100%−o Calc 折算存储——style-system
  parse_origin_edge_offsets 同构）；`parse_basic_shape/tests.rs`
  circle at 4 值钉（直读/折算/同轴拒收三钉 + `<offset> <edge>` 乱序形
  FIXME）。
- **附带臂（decl_supported 全局关键字门）**：`js_dom_bridge.rs` ——
  `CSS.supports(prop, initial|inherit|unset|revert|revert-layer)` 对已知
  属性恒真（CSS Cascade 5 §css-wide-keywords，值文法无关），**臂置顶于
  各属性文法早退之前**（首建版被 counter-increment 文法门截胡——
  `CSS.supports('counter-increment','inherit')` 恒假，违反 Cascade 5；
  置顶修正属行为修正，已测五腿逐腿全同证中性）。此前全局关键字走 apply
  oracle 值校验恒拒 → interpolation-testcommon
  `from [initial]`/`to [inherit]` 前置门整组连坐（offset-position 刀
  残面「initial/unset/inherit 端点组 108」定谳的跨 corpus 形，本臂一次
  清偿）。

参考：css-shapes-1 §shape-outside / §shape-box-value；css-values-4
§typedef-position；CSS Cascade 5 §default-keyword。

## A/B 权威账（B0=results-shape-b1v3 ≡ HEAD ff29d95be 码态采认不重跑；
B1=bin-shapeout-b1v4 md5 70f90c73，08:36 终态树重建——先建 b1v3 41ef3431
后 decl_supported 关键字臂置顶（counter-increment 等属性文法早退不再截胡
keyword 值，CSS.supports('counter-increment','inherit') 由恒假修正为
Cascade 5 语义恒真），属行为修正故重跑 v4 权威批；es_module s419 WIP 两侧
同载零贡献在案；合成探针页 2 页批前删除——twin b1v2 批曾卷入
zz-probe-so-computed/zzz-zw-probe-offsetpath 两页 +11P 假增益，页集合
不等假差按 g41 先例出清。臂置顶行为修正对已测五腿中性实证：v3/v4 两批
五腿逐腿 P 全同（3089/1379/1488/4966/9872），corpus 现值无 counter 族
属性走该前置门）

| corpus | B0 P | B1 P | net | UPS/DOWNS | 页集合 |
|---|---|---|---|---|---|
| css-shapes | 1894 | 3089 | **+1195** | 1195/0 | 145=145 |
| css-masking | 995 | 1379 | **+384** | 384/0 | 61=61 |
| motion | 1134 | 1488 | **+354** | 354/0 | 45=45 |
| css-values（B0=results-flip-b1，见下注） | 4813 | 4966 | **+153** | 153/0 | 256=256 |
| css-typed-om（护栏） | 9872 | 9872 | 0 | 0/0 | 359=359 |
| **合计** | | | **+2086** | 2086/0 | |

- css-values 腿 B0 用 results-flip-b1（11-15 19:25）——其后已提交五刀对
  values 面唯一已知增益 = offset 刀 +1（random-computed offset-distance
  % 槽），实测 UPS 第 153 项恰为该页 +1，归因闭环无漂移。
- **两臂分解**（twin b1v2 批 2950P/146 页含探针 vs 本终态批）：五站刀体
  ≈+1046（shapes 面）、4 值 edge-offset +139（shapes ellipse-004/
  circle-004 位置 canonical 形）、css-wide 关键字臂 ≈+877 跨四 corpus
  （masking clip-interpolation 族 +384、motion offset-*/position/anchor/
  rotate +354、values calc-size interpolate-size +152、shapes
  interpolation 页 keyword 端点组）。
- UPS 头部页与设计稿残面三桶逐页吻合：inset-001 +296/ellipse-005 +178
  （串化 canonical 桶）、shape-outside-shape-interpolation +180 等三页
  （supports 前置门桶开闸后插值可跑）、clip-interpolation.html +108
  （keyword 端点组）。
- **DOWNS 0 全线**——echo 红线（texp 转化率轮 T-P 巧合池）零触碰，
  typed-om 白名单触面护栏 0/0/0 纯增量。
- 批 zw-so2-b1v4-ab（08:36:29 起，5 腿 test-guard 包裹
  `--per-proc-mem 4 --total-mem 8 --time-limit 3600`，transcript 唯一名
  /tmp/zw-so2-b1v4-\<corpus\>.txt，JSON 分目录 results-shapeout-b1v4）；
  前批 zw-so2-b1v3-ab（41ef3431，08:20:32–08:28:35）同法跑五腿已被 v4
  逐腿复证全同，账以 v4 为准。

## 单测与门禁

- engine lib **4060 passed / 0 failed**（含 part03 shape-outside 新钉，
  定向 1 passed 复核）；style-system **2546/0**；css-parser 钉见下
  （首轮 E0425 笔误 parse_clip_path 作用域，修后重跑）。
- fmt `cargo fmt -p zero-css-parser -p zero-style-system -p zero-engine
  -- --check` 三 crate 净（首轮 drift 仅本刀 tests.rs 两行 assert 展开，
  手修后净）。
- clippy：css-parser+style-system `-D warnings` 净；engine 腿唯一 error =
  peer es_module.rs:2217 `too_many_arguments(8/7)` s419 WIP 在册连坐
  （release 二进制编译不受影响，b1v4 构建即证）。
- make test 拆腿（zw-so2-mkt.service 08:45:15–09:03:08 GATE-DONE，12 腿）
  **刀域零红**：绿五腿 companion-build/renderer-bin/gpu 双腿/
  quickjs-renderer-bin 全 EXIT=0；红七腿全对号在册——workspace
  15,651P/6F 全 es_module::tests 六测（s360/s361/s337/s269/reexport 族，
  peer s419 WIP 树态红）、quickjs-clippy es_module.rs:2217 同 WIP lint
  连坐、renderer-lib 371P/1F + quickjs-renderer-lib 369P/1F worker-fetch
  在册镜像、browser-bin 741P/2F + browser-tests 772P/2F tab_js_worker
  fetch 镜像 + default_actions 在册对、quickjs-workspace 825P/1F
  image_batch 环境形（腿数为小目标合计）。
- make test 拆腿门禁（zw-so2-mkt systemd unit，08:45:15–09:03:08
  GATE-DONE，12 腿）：绿五腿 companion-build / renderer-bin / gpu-adapter /
  gpu-consistency / quickjs-renderer-bin 全 EXIT=0。红七腿逐名对号全在册
  **零本刀域红**：
  - workspace 15,651P/6F——六红全 es_module::tests（s360/s361×2/s337/
    s269/reexport-arms，peer s419 WIP 树态红，v8 形在册）；
  - quickjs-clippy 101——zero-script-sandbox too_many_arguments(8/7)
    es_module.rs:2217（同 s419 WIP lint 连坐）；
  - renderer-lib 371P/1F + quickjs-renderer-lib 369P/1F——js_worker
    fetch_without_handler_resolves_error（c6dcbce0a 在册 reject 语义
    过时镜像）；
  - browser-bin 741P/2F + browser-tests 741P/2F——tab_js_worker fetch
    镜像（同根）+ default_actions（在册三红账对）；browser-tests 侧
    cdp_focus 新鲜度 tripwire 本轮未现（companion build 刷新
    zero-renderer 二进制 mtime 顺带核销，先例同 jsanim 轮）。

## 残面（下刀队列）

- **shape-outside-interpolation 660F/780、shape-interpolation 366F/546、
  path-interpolation 340F/478**：门开插值可跑后的真插值/中点期望残面，
  归 Transition 通道可见性刀真插值池（census：shape-outside 351 直取）。
- **circle-004/ellipse-004 各 533F、inset-003 302F、circle/ellipse-003 各
  125F（P=0 全灭）**：串化 canonical 深形（缺分量补 center、edge 序
  canonical、calc 折算展开）——刀 B 已兑现 4 值形，剩余为 declared 级
  canonical 输出序与单位形。
- **shape-margin-interpolation 120F、shape-image-threshold-interpolation
  120F（P=0 全灭）**：shape 族另两属性未入白名单——backdrop-filter
  同型门线刀候选（shape-image-threshold `<number>` [0,1]、shape-margin
  `<length-percentage>`，apply/gCS/插值站需同步落）。
- **composition 138F**：shim composite-add 函数列表 all-or-nothing 主矛盾
  面（十轮已挂账）。
- decl_supported 全局关键字臂跨 corpus 全量兑现面（grid/fonts/
  backgrounds 等 census T 面 15,816F 中的 keyword 端点组）由全量 corpus
  跑自然入账，本刀 A/B 断言面仅四 corpus。
