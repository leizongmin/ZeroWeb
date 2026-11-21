# 刀 GENFAM：generic family keywords quoted/unquoted 三点联动（2026-11-21）

认领：`.acceptance/wptfyi-css/NOTES-knife-genfam-quoted-generic-claim.md`（14:2x）。
本文在发射前冻结预注册，RECONCILE 后补齐实测数字。

## Census 定位（14:17–14:18，results-knife-glbreak/bin/b1-runner 快照 + test-guard）

- css-backgrounds：124 页 3495P/2751F——animations 插值合成域 2243F/26 页
  （A3r/MOTION-3 经营域，避让）、parsing 482F/57 页（css-parser
  property_validate/style-system validate 双在飞域，避让）、他域 26F。
- css-fonts：163 页 6654P/1590F——**generic-family-keywords-001/003 两页
  各 15 case 全灭（30F）**，全挂 quoted 臂（"quoted X matches @font-face
  rule expected 50 but got 25"）、unquoted 臂全过。唯一零撞面内聚刀位。
- 其余候选（palette ~205F/font-synthesis ~38F/font-variant 21F/font-face-src
  ~57F）逐一撞在飞面（apply.rs 混合区 S2a/S2b、at_rules.rs import 区 URLS
  face）排除。

## 机制（源码追踪链定谳）

页面机制（css-fonts-4 #family-name-syntax 测试）：无 @font-face 时 unquoted
keyword 宽度 expected；插入 unquoted+quoted 两条同 keyword @font-face（src
`local(Ahem), url(/fonts/Ahem.ttf)`）后 unquoted 须保持 generic 宽度、quoted
须匹配 @font-face（Ahem）。001 页 offsetWidth 臂、003 页 canvas ctx.font 臂。

三点联动缺陷：

1. **C（css-parser at_rules.rs consume_font_face_rule）**：@font-face
   font-family 描述符 unquoted generic keyword 未拒收（css-fonts-4
   #descdef-font-family：generic keyword 形 → 整条规则无效）。语料零使用
   （grep 证）→ 拒收 P2F 风险面空。
2. **A（webview.rs load_page_font_faces）**：src 选择取 `sources.first()`
   ＝`local(Ahem)` 直接当 URL fetch → 必败 → 整条 face 未注册。live 路径
   （async_load.rs:943-959）按扩展名 find 链（.ttf/.otf → .woff → first）
   早已正确跳 local 项，runner 路径未镜像。
3. **B（三 resolve 点）**：quoted generic 分流缺失——text_shaping.rs:136
   与 layout-engine font_resolution.rs:61 无条件 `continue`（R3249-F
   8cb279a67 引入，动机＝quoted-generic-ignored reftest：quoted "fantasy"
   须落后续 serif）；canvas context_impl.rs:220 family 带引号原串查表必
   miss → 通用族回落。

键空间定谳：build_font_resolver 末尾 generic 映射无条件覆盖基础键＝
"serif" 基础键恒 generic 映射（Liberation Serif）；@font-face alias id
恒存活于 `{family}:face=N` 键（loader.rs:305，expose_alias_faces 默认开）。

## face 设计（七文件 15 hunks，patch md5 dfa1207c3d970556daf46be497c08ed3）

1. **C**：at_rules.rs font-face 区拒收臂＋私有判定（与 URLS 流 import 区
   hunk 零重叠坐标共存）。
2. **A**：webview.rs src 选择镜像 live find 链。
3. **Y**：generic 名 alias **带引号键**注册（`"serif"`/`"serif":face=N`）——
   与 generic 映射基础键自然隔离：unquoted 查无引号键恒 generic 映射、
   quoted 查带引号键命中 alias，lookup_faces 的 `:face=` 展开零互扰。
4. **B1/B2/B3**：三 resolve 点 quoted-generic 臂改「带引号判据键
   `{quoted}:face=0` 存在才按带引号名查表，缺失维持 skip」——reftest
   语义保持＋alias 命中翻面；primary/shaping 同步防 T2 分歧。
5. **单点导出**：render-foundation face_match `is_generic_family_keyword`
   （13 员全集），webview/engine/layout/canvas 四处收敛（layout-engine
   与 engine 私有集删除）；css-parser 低层不依赖自持同型集。
6. **FACE-MARKER**：`GENFAM_FACE_MARKER = "GENFAM quoted-generic-alias-split"`
   （#[used] rodata 代码字面 ASCII）。
7. **D 零改动**：build_font_resolver generic 映射覆盖语义不动。

钉测（发射前实测）：css-parser `test_font_face_unquoted_generic_family_rule_invalid`
＋`test_font_face_quoted_generic_family_rule_valid`（unquoted generic 8 形
全拒＋quoted 三形存活）；layout-engine
`quoted_generic_matches_alias_unquoted_stays_generic_mapper`（quoted 命中
alias 9／unquoted 恒 generic 4／无注册 skip 落 initial serif 三臂）。
engine text_shaping 与 canvas 无独立钉（Painter/CanvasContext 构造成本高，
与 B2 同构逻辑，001/003 页电池端到端覆盖）。

## 预注册预期（发射前冻结）

- 主靶：generic-family-keywords-001/003 各 15 case → 30F 翻面（quoted 命中
  Ahem；unquoted 保持 generic 宽度）。
- 回归哨：quoted-generic-ignored reftest 保持（reftest 通道，不入本 census）。
- 哨兵：css-values/css-text SET-IDENTICAL 为佳。
- font-face-src-local/tech/format 三页（18F/25F/14F）因 A 修复可能邻面
  位移——逐案归因后入账，不预注册必得。
- P2F 逐案归因（页＋case＋消息形）；ASYM-COUNT（断言数不等）挂起不收账。

## 电池

knife-genfam-battery.sh（flock /tmp/knife-genfam.lock 单发；patch md5 绑定；
M2 distinct-md5 门；FACE-MARKER b0=0/b1>0 字面门；语料腿 css-fonts 主靶＋
css-values/css-text 哨兵 × b0/b1，test-guard 包裹 per-proc 4/total 8/3000s）。
发射经 launcher service（zw-knife-genfam-launcher-1443）错峰排队——在飞
zw-knife-uax14ba-battery-1431（语料腿）＋zw-knife-s2calcsize-battery-1437
（构建窗）收口＋树态双采样稳定后自射（共享树电池排队协议）。ride 相消面：
apply.rs 混合区（S2a/S2b）＋css-parser 8 文件 URLS face＋layout-engine
inline 三文件（UAX14BA）＋es_module.rs＋target-v2b1/。

## RECONCILE 实测（15:12 GATE-DONE，收账 15:2x）

### 电池门读数（results-knife-genfam/）

- M2 门：b0/b1 bin md5 异（b1 `c1f6e709f432672bc52783508515c7f7`）✓
- FACE-MARKER 门：`GENFAM quoted-generic-alias-split` b0=0 / b1=1 ✓
- 六腿全跑：css-fonts 163/163 页 8244 断言、css-values 256 页 9864、
  css-text 354 页 4518——两侧断言数全等（无 ASYM-COUNT 挂起）。

### census 差分（knife-genfam-judge.py）

```
[css-fonts]  F=1590->1590 gained=0 lost=0 net=0
[css-values] F=3998->3998 gained=0 lost=0 net=0
[css-text]   F=2467->2467 gained=0 lost=0 net=0
GRAND gained=0 lost=0 net=0
```

**零位移**：预注册 30F 主靶未翻，无 P2F、无 ASYM。哨兵腿 SET-IDENTICAL ✓。

### 零位移根因（源码追踪链，收账前独立核机制）

face 机制本身正确（钉测全绿），零位移＝face 在 runner 页面渲染路径上
**不可达**，三缺口：

1. **动态 @font-face 注入缺口**：001/003 页两条 @font-face 规则是页面 JS
   `insertAdjacentHTML('beforeend', '<style>…</style>')` 运行时插入（非
   初始 HTML）；load_page_font_faces 只在 load_html 前的
   fetch_page_images 通道跑一次（webview.rs prepare_page_subresources
   :1110），mutation 后路径（webview.rs :1526/:1596/:1672）只刷图片子
   资源，@font-face 永不增量注册。
2. **双 FontLoader 断层**：即使注册成功，alias 进的是 canvas registry
   loader；painter/layout 的 resolver 表来自 font_stack_loader
   （create_font_loader，runner 启动 webview.set_font_resolver 一次，
   testharness.rs:9628-9629），load_page_font_faces 无刷新通道。生产
   async_load 路径有 `drain_loaded_fonts → set_font_resolver →
   request_rerender` 刷新先例（async_load.rs 测试镜像），runner 路径缺环。
3. **promise 微任务风暴时序**：001 页断言在 `load` 事件后
   `document.fonts.ready.then(runTests)` 异步步内；shim fonts.ready 是
   一次性 settle（R2947 __zw_font_settle，finish_page_load 时 resolve），
   后续 `await document.fonts.ready` 立即解除——13 keyword × 2 断言在
   微任务风暴里连续同步读 offsetWidth，宿主无渲染机会。face 注册须挂在
   mutation apply 同步流内（insertAdjacentHTML 桥回调与 offsetWidth 读取
   之间），需要 engine↔webview 反向 provider 通道——超出本刀预注册范围。
   （font_stack_loader 直接预载 Ahem，reftest_fonts.rs:142-143——#ahem
   参照宽度正常而 quoted alias 永不可达，与断言消息形一致。）

### 收账决策（CHMETRIC 先例：机制落库、主靶未翻、gap 挂账）

- **face 入库**：C/A/Y/B1/B2/B3 机制正确且钉测全绿（css-parser 16
  font_face 测试含 2 新、layout-engine 7 passed 含 1 新三臂钉），双臂
  净=0 无 lost 零回归；是第二阶段的地基（alias 键空间、generic 拒收、
  quoted 分流判据已定）。
- **30F 主靶未翻**＝刀未完成：三缺口挂账第二阶段（动态 face 增量注册
  通道＋runner resolver 刷新环＋mutation 同步流挂点），不在此收账刀
  扩范围（准则 5）。
- css-fonts F 总量 1590 不变：font-face-src-local/tech/format 邻面
  预注册的「可能位移」也未发生（同根因——runner 页面 face 注册通道整体
  不可达，A 修复单点不生效）。
