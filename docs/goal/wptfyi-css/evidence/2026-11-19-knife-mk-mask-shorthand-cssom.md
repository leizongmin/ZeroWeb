# 刀 MK：mask 简写 `<mask-layer>#` 全文法（CSSOM 双车道）

**状态：电池已收账 + 迁移收口重验过（mk-reval：b1v2 net=+83、P2F=0、
零回归——P2F=30 机制归因获清偿实证，见重验节）**

## 机制

- 池＝css-masking/parsing mask-valid.sub.html 36F + mask-invalid.html 13F
  ＝49F（CLAIM-knife-mask-shorthand 23:25 让位条，基线 pre-MK probe 22:4x
  核实：mask-valid 19P/36F + mask-invalid 0P/13F）。
- 刀 B inline CSSOM 单源同构：shim writeDecl/readProp 通用 miss 兜底链任意名
  直达两站（part03.js L23479/L23413，零 JS 编辑）→ engine 两站 mask 分派臂
  （css_style_declaration.rs，fork C 笔）→ style-system `expand_mask` 全文法
  展开（shorthand/mask.rs 新模块，本流笔）。V8 native 车道 decl_supported
  mask 臂（js_dom_bridge.rs，fork C）parity。
- engine 写面站语义（fork C）：`mask_shorthand_expand_json`＝空值→13 名
  remove 列表；css-wide 整值→13 对透传；var()/attr() 替换函数形放行 store
  原文；展开失败→`[["mask",""]]` 自清拒收；成功→13 对 JSON。读面站
  `mask_family_value` canonical 重组＝border 五非初值→"" 不可表达、
  8 longhand 缺任一→""、逐层 canonical 序。
- 文法＝css-masking-1 §3.10 `<mask-layer>#`：`<mask-reference> || <position>
  [/ <bg-size>]? || <repeat-style> || <geometry-box> || [no-clip ||
  <geometry-box>]? || <compositing-operator> || <masking-mode>`；括号感知
  逗号分层复用 background `split_bg_layer_list`。
- 拒收集 13 形（mask-invalid.html 定谳）：双 reference 两形（none+image /
  渐变+url，图像函数括号感知提取后残 token 拒 + none 臂互斥）、双 mode、
  双 composite、双 `/` size 段（首段增量消费后溢出 `/` token 文法外拒）、
  三四裸 length（parse_position_strict 终验）、三盒、no-clip+双盒三形、
  `margin-box`（§3.7 无此值）、单轴 repeat 配对（`repeat-y repeat-x`）。
- 投影 13 longhand（§3.10 reset 名单）：8 mask 逐层 join（缺省层补初值）+
  mask-border 五长手初值（source none / slice 0 / width auto / outset 0 /
  repeat stretch——§3.4-3.9，slice/width 与 border-image 简写不同值）。
  clip 派生：no-clip 旗标字面 > 双盒次盒 > 单盒同 origin（§3.10 prose 单盒
  同时定 origin 与 clip）。
- css-wide 门（fork C 03:11 补笔，P2F 阻塞项）：`expand_mask` 入口
  `matches_css_wide_keyword` 短路 → 13 名单逐 longhand 同值透传
  （`wide_keyword_to_longhands` 复用，Cascading-4 §7.3 整值截获面）。无门则
  接线后 `mask: inherit` token 分类失败声明丢弃回显空 = P→F（b0 raw-store
  基线回显 ✓）。

## 本轮真文法 bug 修一笔（语料驱动）

`split_bg_position_and_size` 返 `/` 后整段为 size，而 `<mask-layer>` 文法
size 位仅 1-2 组件、余 token（repeat/盒/composite/mode）属 `||` 主流——
整段严门误拒 valid 形 `1px 2px / contain space round view-box no-clip
intersect`。修＝size 段增量消费（2 token 优先/1 token 回退，
`parse_background_size` 同源严门）+ 溢出 token 归主流分类；`/` 后无任一
合法 bg-size 前缀＝拒（invalid 形 `1px 2px / contain / cover` 溢出 `/`
token 文法外拒保持）。

## 验证（电池前静态面）

- style-system：`knife_mask` 钉测 6 passed 0 failed（本流 5 枚：13 投影
  初值/reset、盒三形、全槽乱序层〔size 增量消费实证〕、多层 join〔computed
  形 `0% 0%, 0% 0%` / `add, add` 等〕、13 拒收形逐条；fork C 1 枚：css-wide
  门 5 关键字 ×13 透传）——cargo test -p zero-style-system --lib knife_mask，
  test-guard 包裹，首执行即绿非 DOA。
- 全量 lib 2578 passed 0 failed（含旧单值版钉测翻转：`tests/mask.rs`
  FIXME(mask-shorthand-grammar) 限单值拒收集由全文法替换，四形翻转断言
  展开）；既有 `test_mask_shorthand_reset_cascade_wins` §3.10 级联语义钉
  零漂移；rustfmt --check 清。
- engine 侧（fork C）：knife_mk_expand_json_verdicts /
  knife_mk_reconstruct_verdicts 两钉在档（css_style_declaration.rs
  knife_mk_cssom_tests），门 tripwire inherit→13 透传断言。

## 协作分工（NOTES-knife-ts2-twin-arbitration 仲裁在案）

- 本流：style-system 面（mask.rs 全文法+钉测 5 枚 / mod.rs 接线+旧单值版
  删除 / background.rs 两 helper pub(super) / tests/mask.rs 翻转）。
- fork C：engine 面（css_style_declaration.rs 两站 mask 分派臂 +
  mask_shorthand_expand_json + mask_family_value canonical 重组 +
  knife_mk 两钉 / js_dom_bridge.rs decl_supported 臂）+ css-wide 门补笔
  （style-system 面唯一越界笔，P2F 阻塞项事由在案）+ 门钉补笔 +
  knife-mk 电池发射权。
- provenance 观察（rb 流自查项，不仲裁）：knife-ts-rb b1 构建窗 02:47:52-
  02:48:38 早于 computed_style.rs 末写 02:50:34。
- 双预写件仲裁（NOTES 03:3x）：本文件 = canonical evidence（沿
  knife-ts-shorthand-cssom.md 命名惯例）；无后缀副本
  `2026-11-19-knife-mk-mask-shorthand.md` 机制段已并入本文件（engine 站
  语义 + pre-MK probe 22:4x 基线注记），电池绿后移除该 untracked 重复件
  （不入 commit）。resume-swarm 同 id 多 fork 署名漂移形态在案，收敛产物
  以本文件 + git 账面为准。

## 电池账（results-knife-decomp，2026-11-19 09:40 GATE-DONE）

分解电池（zw-knife-decomp-battery，15 腿 + unit pins；bin md5 b0
`54207468` / b0s `a473acbd` / b1 `4ca80d11`；S1==S3 全树零扰动——
S1-MISMATCH 行系脚本检查位错配假警，s2 剥离态 vs s1 全树态按构造必异，
仲裁条 NOTES-knife-decomp-post-collection.md 09:18 节）。

**css-masking b1 vs b0：F2P=83 P2F=30 net=+53**（双腿 rc=1 语料常态）：

- F2P 页级：mask-valid.sub.html **+36**（基线 36F 全清）、
  mask-no-interpolation.html **+28**（全 `transition-behavior:allow-discrete`
  面，TB face DISCRETE_TRANSITIONABLE 合法 win）、mask-invalid.html **+13**
  （基线 13F 全清）、mask-computed.html +6——基线池 49F 全部兑现，
  另加 no-interp/computed 超额面。
- P2F=30 **全落 mask-image-interpolation.html plain 形**（neutral/initial/
  inherit/unset→url 家族 t<0.5 采样，`expected green-20 got stripes-20`）。
  定谳（第三 fork 独立法证复核）：TB face 冻结版 `all` 通配按可插值清单
  逐项展开、未按 css-transitions-1 §starting-of-transitions **末位匹配**
  并入管辖——transition-property:all + 后位 transition-behavior:
  allow-discrete 时 mask-image 离散跳变面 30 子测 P→F；静态值 subtest
  零翻转（MK/ATP 静态解析无辜），翻转只在 b1-only face。清偿＝管辖归并
  表重写（transition_registry.rs，末位覆盖模型，TB 域归并重写流在飞，
  账见 TB evidence；pristine 账留 P2F 法证链）。**清偿已实证**：mk-reval
  b1v2（树含归并重写 + plain 门）该页 30 失败全翻 Pass、零回归，见重验节。
- 哨兵：css-properties-values-api 净 +80 零 P2F（ATP 账面）；
  css-typed-om 零位移。

## 迁移收口与重验（mk-reval，2026-11-19 10:52）

- style-system 面收口：`expand_mask` 独立模块化（`shorthand/mask.rs` 新文件，
  mod.rs `mod mask;` 接线 + background.rs 两 helper `pub(super)`）——迁移流
  模块重构 + 机械错修复笔入树（10:38 接管：三臂越界 `t`→`token`、
  `LengthValue::Percent`→`Percentage`、unused mut；3 处 clippy lint：
  mod.rs 孤儿 doc 转普通注释、bg-size 段增量消费改 `find`+`?` 同语义、
  doc 缩进；rustfmt 全绿）。语义零变钉测复验：knife_mask 6/6、
  style-system lib 2579 passed 0 failed。
- **mk-reval b1v2 电池腿**（systemd zw-mk-reval；bin md5 `3348c60d`；
  树态＝MK face + TB 归并重写 + plain 门；账走 mk-reval/ 侧目录不触
  canonical）：
  - **css-masking b1v2 vs b0：F2P=83 P2F=0 net=+83**——MK 双腿兑现全保留
    （valid +36 / no-interp +28 / invalid +13 / computed +6），较 battery#2
    权威账 +53 多出的 +30 即下条清偿面。
  - **vs battery2 pristine：regressed=0 gained=30**——mask-image-interpolation
    plain 形 30 失败全翻 Pass。P2F=30 机制归因（TB face `all` 通配）由此
    获清偿终证；「瞬态」假设作废（第 5 跑负载复刻经 md5 法证系副本、从未
    真跑，launcher-arbitration 10:3x 条）。
  - 门禁：engine clippy 本体零 warning（es_module.rs:2211 too_many_arguments
    系 RIDE 域外在途码，挂账待对端收口）；mask.rs 修复笔账目在
    launcher-arbitration NOTES。

## 残面

- mask-border 家族池（8 页 1791F/0P，master.md 观察面在册"下一刀候选"）
  ——mask 简写投影其五长手初值 reset，属性面 parse/apply/gCS/动画另刀。
- 六长手写值 apply 通道 FIXME(mask-longhands-unimplemented)：投影 reset 真、
  写值不生效，computed 面位移归其分诊池不入刀 MK 账。
