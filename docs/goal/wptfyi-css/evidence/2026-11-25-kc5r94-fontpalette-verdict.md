# kc5r94 verdict——font-palette 属性＋palette-mix calc 折叠刀（10 触点）（2026-11-25）

## 结论

- **刀面**（指纹 `ZW-KC5R94-FONTPALETTE-FACE`，电池代 b1 bin md5
  8f0f1f01af4ce7b9a01a6fc1d552a2aa＝R94/R93 双指纹并存 strings=1/1；b0=
  r93-vtn-runner 14794f737b1eefd5f08c3ff90580e1cc 指纹=0）：
  1. registry.rs 三表：known_properties + "font-palette"（CSS.supports
     属性名面＋`all` 展开清单 +1）＋initial_value
     `OptionalString(Some("normal"))`＋is_inherited **true**。
  2. computed_style.rs 字段 `pub font_palette: String`（css-fonts-4
     #font-palette-prop，Inherited: yes——与 vtn 反向形）。
  3. default_impl.rs "normal"。
  4. inherit.rs 两臂：inherit_property 父→子 clone（继承属性加臂）＋
     apply_initial_value 复位臂。
  5. apply.rs 求值臂：substitution 词放行（oracle 防线）∥
     normal/light/dark ∥ 裸 dashed-ident（`is_custom_ident_token`
     r93 资产复用）∥ palette-mix( 前缀＋**单层 `calc(<percentage> *
     <number>)` 折叠小臂**（`fold_percent_times_number`：f64 相乘
     `format!("{}%", l*r)` 最短形→'90%'；解析失败原文保）。
  6. apply.rs validate_decl_value：白名单 [&str; 110] + "font-palette"
     ＋分派臂（substitution 放行 ∥ 三 keyword ∥ dashed-ident ∥
     palette-mix( 前缀）。
  7. matcher/mod.rs is_property_supported 臂（r93 教训①预判兑现——
     布防在先，as-run 零落空）。
  8. engine js_dom_bridge.rs decl_supported 前置臂（r93 教训②预判兑现，
     委托 validate_decl_value）。
  9. engine js_dom_bridge/computed_style.rs gCS 串化臂（:390
     random_computed 表查询属性无关在前，% 保留形 echo 自然承接）。
  10. lib.rs `#[used]` 指纹（**random_integer_typed 不加＝font-palette
      无 integer 位**，r92/r93 教训反向应用；继承×挂表子元素链
      FIXME 域外）。
- **布防预测账 as-run 零修订全兑现**（r92 三缺口／r93 三缺口修订之后
  首轮预注册零落空）：

  | 门 | 布防预测 | as-run | 判 |
  |---|---|---|---|
  | G-F2P | =2 精确（:430 calc 折叠键＋:470 shared calc 键） | **=2** 刀内精确，out-of-registry=0 missing=0 | 兑现 |
  | G-P2F | =0 fail-stop（键 4 % 形 shared 持 P 最高危位） | **=0**，hold-P 显式守卫双臂 Pass | 兑现 |
  | 哨兵 | 'Shared by property' Fail 集 3→2 | **=2**（color-mix＋animation-timeline 持；font-palette calc 移出） | 兑现 |
  | held-F | 键 2 % 保留形＋descriptor 4 键持 F | **5 键全持 F**（held-F-bad=0） | 兑现 |
  | G4 | SET-IDENTICAL 判据无修订 | **非豁免页 diff=0** | 兑现 |
  | G5 | GONE/NEW=0 | **9882/9882 = 0/0** | 兑现 |
  | NET | +2（6741→6743） | **6741→6743 = +2** | 兑现 |

- **探针门 probe-impl 八门双臂三跑定谳**：run1 b0 8F 未知属性形正确
  ＋b1 五门翻 P（E1/E5/E6/E7/E8）＋E2 Fail＝held-F 守卫符合预测；
  **E3/E4 Fail＝探针页自身断言形错误**（v1 element-scoped 形误做
  assert_equals——两元素异 R 异 draw 异串；battery 真身 :470/:474
  equality 走 v2 property-index-scoped 变体，与引擎行为无关）＝r93
  「工件面非引擎面」同型勘误。run2 修断言形（v2 scoped equals＋v1
  not-equals 复刻 battery 断言链）后全对齐预测：b0 8F（E4 新形转
  **P**＝双 P 守卫前半）＋b1 七翻一守（E1/E3/E5/E6/E7/E8 P＋E4 P
  ＋E2 双 F 守卫＝% 保留形 echo 实显 `random(fixed 0.789…, …)`）。
  探针页电池发射前 trap-rm（PROBE-PAGE-CLEAN=0）。
- **电池双腿** css-values RC=0/0 满量 257 页/9882 case（b0 03:15:39
  →03:18:53、b1 03:18:53→03:22:06）＝**judge VERDICT PASS /
  verify VERIFY-PASS**（judge 五门零修订；恒等式 +2==2−0；基线
  6741 与 r93 收账逐位钉死）。judge 预注册硬码零 bug 一次性通过
  （r93 三 bug 教训前置兑现：rcs[-2:] 代际段＋F2P/held-F 互斥
  显式集）。
- **cascade-precheck 6-run**：all-prop-revert-layer 页 b0
  97/100/98、b1 97/97/100 分布互叠＝r92/r93 同款 HashMap 迭代序
  flake，非 font-palette 注册回归＝all 展开第 4 采样点 watch 结案。
- **门禁**：clippy 三包（zero-css-parser＋zero-style-system＋
  zero-engine --all-targets -D warnings）RC=0 零告警；fmt 归因核
  前置（r93 教训兑现——收账前先扫）：残面 2 位点（engine
  js_dom_bridge/computed_style.rs:32＋style-system lib.rs:3375）
  均在刀 hunk（engine @792、lib.rs @4982）之外＝既有债区，**本刀
  自有 fmt 债 = 0**（六触点文件 registry/computed_style/
  default_impl/inherit/apply/matcher 全 CLEAN；apply.rs 折叠
  helper、ident.rs 复用位点全 rustfmt 干净）。
- **代账**：GENERATIONS.txt r94 行（布防轮已落）；工件
  .acceptance/wptfyi-css/kc5r94/ 本地不入库（probe 三跑 JSON＋
  battery 双腿 JSON＋judge/verify＋cascade-precheck＋bins 双代
  md5＋NOTES §零-§五）。

## 挂账（域外移交）

1. 键 2 `random(90%, 10%)` % 叶 fixed-echo 保留形（§9.3
   unresolvable percentage）——引擎保留形 vs 上游直求值结构分歧，
   引擎级刀域（random.rs :933 保留形全域依赖，不可单属性刀）。
2. descriptor 层 4 键＝@font-palette-values parser 刀域
   （random-in-descriptors 2＋sibling-function-descriptors 2）。
3. 继承×挂表子元素链：random_computed 表「非继承，每元素独立」
   注释 vs font-palette 继承属性 % 叶挂表后子元素行为，battery
   语料零消费键，FIXME 域外。
4. fmt 清偿笔 10 处既有债（random.rs 8＋engine :32＋lib.rs :3375）
   独立提交延续挂账。

## 下一步

wrapper 族余 2 键刀序：animation-timeline（view() 语法面）＞
color 相对色（css-color-5 域最大）。同构流程：布防预注册＋实施＋
探针门＋电池双腿＋judge/verify＋收账三件。DC-3 周账 12-01 窗口。
