# 刀 MOTION-1：css-motion offset-rotate/ray/offset 简写 supports 门 + deg 等价插值（2026-11-20）

## 池与根因链

css-motion 全套 45 页 2,410F（motion.json 池化，probe results-motion-probe/
motion-full.json 19:45 b1 代 binary）。三机制：

1. **supports 门 570F**——offset-rotate 288F / offset-path 246F / offset 简写
   36F：parse 通道缺形（rotate `reverse <angle>` 复合与 `||` 乱序、calc() 角、
   非 deg 单位；ray `contain` 关键词 + calc 角 + `ray(...) coord-box` 组合
   （ray 臂 early-return 先于 split_coord_box_suffix 全拒）；offset 简写
   registry 无 `offset` 属性名）。
2. **值断言 1,827F**——rotate 复合形端点/插值（canonical 折叠与 deg 等价
   lerp 缺）、path() 命令插值（656F，MOTION-2）、anchor/position
   `<position>` 对（266F）、composition add 机制。
3. OTHER 13F。

## 码面（face 5 文件，+703/−137）

- **types.rs**：`OffsetRotateValue` 扩 `ReverseAngle(f64)`（gCS 回显折叠
  `auto (N+180)deg`，语料 block10-12 端点钉）+ `Verbatim(Box<str>)`（`||`
  乱序/calc/非 deg 单位端点 verbatim 回显）；Copy→Clone。
- **registry.rs**：known_properties 收 `offset` 简写（initial 借形
  Transform(None)，展开臂在 apply_advanced）。
- **apply_advanced.rs**：`parse_offset_path_fn` 收 `<url>` 形（文法裁决单点
  parse_clip_path 既有 `<clip-source>` 臂；FIXME(motion-url) 渲染联动未接）、
  coord-box 后缀先剥再裁决核心（ray 组合形解锁）、`parse_ray_decl` 重写
  `&&` 全序受理（contain 任意位 + 五 ray-size + calc 角 + 括号感知
  split_paren_aware）；`parse_offset_rotate_value` 两词复合 `[auto || reverse]
  <angle>` 双序受理（canonical 全单位归 deg 语义形、乱序/calc Verbatim）、
  calc() 角 Number|Angle 维度门 + eval（font 相对按所属元素字号）、单词非
  deg 单位 Verbatim；`offset` 简写 apply 臂（path/none/shape + coord-box +
  距离 + rotate 复合乱序预合并；`<position>` 分量语料零出现 FIXME 挂
  MOTION-2）。
- **animation.rs**：`classify_rotate_endpoint` deg 等价分类插值（关键词类
  reverse ≡ N+180/纯 reverse ≡ 180deg 按 canonical auto 词 lerp；跨类离散
  css-values-4 §8.5.6；rad/grad/turn/calc 归 deg）；strict 臂同口径
  None=非插值对；`parse_offset_rotate_computed` canonical 复合全单位归 deg
  （语料 `auto 5turn`→`auto 1800deg`、`reverse -50grad`→`auto 135deg` 钉）、
  乱序/calc/非 deg Verbatim roundtrip 保真。
- **computed_style.rs**：`offset_rotate_to_css` 六臂（ReverseAngle 折叠 +
  Verbatim 直通）。

单测：engine `test_offset_rotate_interp_and_discrete` 扩 7 断言（block1/3/4/
5/10 语料钉 + 离散钉 + 单位换算）、`test_transform_offset_rotate_no_offset`；
style-system offset 13P（含 `offset_path_none_and_path_forms` url(#clip)
契约翻转——原拒收断言写于 CLIP-URL url 文法落库前）。clippy 两 crate
--all-targets 0 错；fmt 我方 hunks 零 diff（HEAD 既有漂移 11 处不动）。
验证线 receipt：23:34:19 静默判据达成后 r1-launcher fork 复验
（compile 2.07s + 双 crate 单测 + clippy + fmt 全绿，NOTES-knife-motion1-
align-r1launcher.md）。

## A/B 电池（r2 converged，knife-motion1-battery-r2.sh，23:35:37→23:49:19）

- b0 = HEAD `ab904e542` + ride（face 剥除）md5 `97328d055060041bc0c5656a9b5e508d`
  **三代恒**（r1 23:00 / 部分批 23:27 / r2 23:35 三次独立构建字节相同＝构建
  确定性三证，HEAD+ride 窗内零扰动）；b1 = HEAD + ride + face（face v2 含
  23:24 C 修 animation.rs CSSOM 通道）md5 `3acc230becdd9ea1dced774379188568`
  两代恒（部分批 23:29 / r2 23:36）。M2 异 md5 ✓；FACE-MARKER
  `MOTION1 offset-rotate-ray-shorthand-forms` b0=0/b1=1 ✓；S1/S2/S3 ride
  census es_module.rs（对端 s419 WIP）md5 `8c77f2ad` 恒 ✓；TREE-COMPILABLE
  dev 0.22s ✓；12 腿全 JSON 非空；TRAP-NOOP rc=0。
- 前代 r1（face v1，results-knife-motion1-r1 归档）：gained=356 lost=8，
  三 loss 机制定谳（url blanket 拒收 ×6 / 5turn Verbatim 过宽 ×1 / ui
  outline-017 载荷 ×1）→ 修面 v2（url 收 + keyword-first canonical +
  animation.rs CSSOM 通道）后 r2 重跑全 12 腿。

## r2 结果（RECONCILE stdout 与独立重算 motion1-r2-collect.py 全等互证）

| 语料 | asserts | pass | 页 |
|---|---|---|---|
| motion | 4955→4955（对称零 ASYM） | 2545→**2891（+346）** | 45→45 |
| borders | 1553→1553 | 842→857（+15 flake 带不入账） | 79→79 |
| masking | 6355→6355 | 5520→5520 SET-IDENTICAL | 62→62 |
| shapes | 7515→7515 | 6310→6310 SET-IDENTICAL | 145→145 |
| ui | 1940→1940 | 1342→1341（−1 flake 不入账） | 119→119 |
| typed-om | 12406→12406 | 12184→12184 SET-IDENTICAL | 359→359 |

motion 分页恰集：rotate-interp 262→490（+228）/ path-interp-005 194→262
（+68）/ offset-interp 0→36（+36）/ rotate-composition 19→31（+12）/
path-computed 27→28（+1）/ rotate-computed 2→3（+1）。
F2P 348 / P2F 2（bare-reverse at(1) 端点 echo ×2＝canonical lerp 与离散
echo 的设计权衡，B fork 语料钉决策：block5 CSS Transitions at(1) ×2 让渡）。

**r1→r2 修向回执（r1 b1 vs r2 b1，motion 全语料）**：fixes=8 regs=0——
interp-007 url 对子 ×6（`'to' value should be supported`）+ computed 页
`auto 5turn`/`reverse -50grad` ×2。r1 lost 8 → r2 lost 1（仅余 flake）。

## flake 定谳（双向不入账）

- **borders corner-shape-interpolation +15**：CS-1c 已证 flake 带
  ±17~20（r1 +11 / r2 +15 / clipurl 电池 −14 双向；notch/square/round/
  bevel/superellipse 值级噪声同签名），双向不入账。
- **ui outline-017 'outline-offset is animated as a length' −1**：升级定谳
  ——r2b 配对探针（results-knife-motion1-solo-ui/r2b-b{0,1}-ui-outline.json
  23:55:58，qemu 负载窗）b0/b1 双臂同败 0P/3F（`expected 2px got 2.01px`
  同 message），而同一 b0 binary（97328d05 三代恒）在 r2 电池窗内该断言
  PASS——**同 binary 跨窗 P↔F 翻转＝载荷敏感非 face 归因**。终证：r1-b1
  binary（3678a4a6）23:55 solo 窗 JSON 与 r2-b1 电池 JSON **md5
  `a3a6cee6` 字节相同**（runner 载荷窗确定性），而 r1-b1 于 23:25 静默窗
  solo PASS（1342P＝battery 1341P +1）——二进制代际在本断言上无差异，
  差异全在机器载荷。不挂回归。

## 残面挂账（MOTION-2+）

- calc 复合声明 parse 门：`auto calc(...)` 多 token split 拒收（interp 页
  24F + computed 页 1F）——split_paren_aware 两词化 +30~40 估（MOTION-2
  候选）。
- composition add 机制 41F（rotate-composition 页 under-add 群）。
- bare-reverse at(1) echo ×2（权衡保留）；css-wide inherit 端点 10F；
  WAAPI stale-underlying 臂 14F（引擎动画域）。
- path() 命令插值 + 绝对化 canonical（001-004 656F）；anchor/position
  `<position>` 对（266F）；shape() arc（008 204F）；offset-path composition
  add 84F；`<position>` 简写分量（FIXME 已注）。

## 账

**入账 +346（motion 恰集），局部账 9886→10232（本刀时点口径；OM-8 +4
插账后链上终值 +10236）。** 零回归（哨兵四语料
SET-IDENTICAL；borders/ui flake 域双向不入账，证据同窗配对探针 +
byte-identical determinism 双证）。face 提交三代 rebase 后活 hash
`988a1aefc`（pre-rebase 代 `2304f834b`/`d808cb16d`，patch-id
`b05437c6` 恒＝电池证据跨代成立）。
