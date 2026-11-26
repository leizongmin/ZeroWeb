# kc5r136 verdict——other-slots 刀：异种伪元素槽表＋first-line/first-letter part 通道 plumbing（2026-11-26）

## 结论

- **刀域**（css-parser＋style-system＋engine 三 crate＋shim 一臂；判据页
  css-shadow part/pseudo-elements-after-part＋part/interaction-with-pseudo-
  elements＋css-pseudo highlight-cascade/first-line/first-letter 族）：
  ① **css-parser**：`is_known_functional_pseudo_element` 收 `"highlight"`＋
  `is_valid_functional_pseudo_element_arg` 新臂 `highlight => arg != "*"`
  （css-pseudo-4 §highlight-selectors，高亮名 = `<ident>` 非通配；`*` 是
  view-transition 专属形走 Delim 臂显式拒）。此前 `::highlight(x)` 规则整条
  被弃（is_known 名单缺位）＝伪样式存储/投影读无源。k133 钉
  kc5r133_part_parse_invalid_forms 的 `::part(x)::highlight(n)` 暂拒断言转正
  （语料 pseudo-elements-after-part.html `test_valid_selector` 钉该形可解析，
  移 suffix_forms 断言存活 Functional{highlight,n}）——k133 旧断言为偏差，
  其 FIXME(k134+)「函数伪元素缀后扩面」由本刀兑现。dom/query.rs 选择器
  合法性走 k132 既有表不受影响。
  ② **style-system**：`ComputedStyle.other_pseudos: Option<Box<Vec<(Box<str>,
  Box<ComputedStyle>)>>>` 异种槽合一载荷（R3867 单指针＋Vec 仅存储命中分配）
  ＋`OTHER_PSEUDO_NAMES: [&str; 8]`（backdrop/details-content/file-selector-
  button/grammar-error/highlight/search-text/spelling-error/target-text）＋
  `other_pseudo_key`（split_once 三臂：highlight 闭合校验全形键；静态名裸键）
  ＋`stylesheet_other_pseudo_keys` 页扫描（S10/R5054 同型零成本门禁，全量
  shadow_scopes 并集/增量单表两入口）＋compute_styles_recursive/
  attach_pseudo_elements 签名透传＋每元素存储块（门 = color 相异或背景非
  Transparent——background 非 inherited 不可比 elem_style，R5052/k135 三门
  同型；键 = 归一小写名，函数形带实参全形）＋**first-line/first-letter
  part 通道 plumbing**（compute helper 加 shadow_view 末参＝`::part(x)::first-
  line` 缀后形可存；探针放宽＝std 探针空且无 part 属性早退 None；first-letter
  存储门扩 bg 位镜像 R5320）。
  ③ **engine**：`serialize_pseudo_element_property` 扩臂（first-line/
  first-letter 存储槽＋other exact-key 查表——k133 名单外硬 `return None`
  移除，槽 miss 统一本体回落；k133 P3 钉 is_none 断言按新契约更新＝first-line
  回落元素值，API 终值与旧 shim 丢第三参行为等价）＋
  `normalize_pseudo_element_arg`（14 名单直通＋`highlight(<arg>)` 臂 trim
  外围空白＋CSS escape 解码＋小写＋`is_pseudo_arg_ident` 校验——CSSOM API
  语法容忍无闭合括号 `::highlight(name\t` parsable 形；`::highlight(1)`/`($)`
  nonParsable 拒）＋callbacks.rs 两通道五槽名单块统一走该归一单点＋shim
  part01.js 伪元素归一名单 14 名＋highlight( 前缀透传（实参归一 host 单点）。
  ④ **P2F 根修**（r2 电池暴露→r4 面内修复）：`registered_properties::
  computed_value` 声明值 CSS-wide 关键字 `initial` → 归入 unset 臂路由注册
  初值（css-properties-values-api-1 §3 computed value——关键字语义先于
  syntax 分型，大小写不敏感；`inherit`/`unset`/`revert` 需父链 FIXME(k136+)）。
  b0 偶然绿根因＝`::spelling-error` 规则被解析弃置→gCS 第二参读形回落
  元素级 unset 兜底；k136 规则存活后槽侧字面 `initial` 直入分型失败。
  ⑤ 指纹 `ZW-KC5R136-OTHER-SLOTS`（css_wire.rs `[u8;22]`）。
- **电池**（b0=k135 归档 bin 505036c1／b1=新代 150390c2，指纹 strings=1；
  锁键 zw-k136-battery.lock；**18 腿全 RC=0**＝9 suite×2：css-shadow/css-pseudo/
  css-values/css-variables/cssom/css-position/css-conditional/css-anchor-position/
  css-properties-values-api；POST MD5 与发射前恒等）：
  **全量 F2P=59 P2F=0 NEW=0 GONE=0＝净 +59 红线成立**。
- **F2P 分解**（59 键；键级差分 /tmp/zw-k136/diff-k136-r4.txt）：
  - css-shadow **P 259→271（+12）**：pseudo-elements-after-part 10 键
    （::part()::backdrop/details-content/file-selector-button/first-letter/
    first-line/grammar-error/highlight(myhighlight)/search-text/spelling-error/
    target-text computed style＝挂账异种槽位 10 键全数兑现）＋
    interaction-with-pseudo-elements 2 键（::first-line/::first-letter in
    selected host is styled＝part 通道 plumbing 目标兑现）。
  - css-pseudo **P 127→161（+34）**：highlight-cascade 25 键
    （highlight-currentcolor-computed 10＋highlight-pseudos-computed 5 含
    `Different getComputedStyle() for ::highlight(bar)` 双名键＋
    highlight-pseudos-visited-computed-001 10）＋first-line-allowed-properties
    6 键（block/inline/flex × color/backgroundColor）＋
    first-letter-allowed-properties 2 键（color/backgroundColor＝bg 位兑现）＋
    file-selector-button-inherit 1 键。
  - css-values 6844→6847（+3）：attr-css-wide-keywords 3 键（`--y` 值含字面
    `initial`/attr fallback initial 形＝P2F 根修超预期面，§3 关键字解析
    attr() 产物同享）。
  - css-variables 304→308（+4）：variable-first-letter/variable-first-line
    color＋nested color（legacy 单冒号读形投影翻绿）。
  - cssom 2326→2327（+1）：getComputedStyle-pseudo `::file-selector-button
    resolves to the file input button element`。
  - css-position 517→518（+1）：backdrop-inherit-computed（继承 custom
    property 到 background-color）。
  - css-conditional 1538→1542（+4）：container-queries/backdrop-invalidation
    1 键＋pseudo-elements-004 3 键（::first-line/::first-letter/::backdrop
    Initial color）。
  - css-anchor-position 12447→12447＋css-properties-values-api 957→957
    双零漂移（后者＝P2F 根修机器域定向核）。
- **r2 电池（根修前 face c6f23ed4）P2F=1 发现与归因**：
  highlight-cascade-010 `the custom property receives its initial value from
  a deferred registration`——r2 面 F2P=56 P2F=1 停发，P2F 键级归因→根修
  ④→r3 面重发（两次被 10 分钟后台包装器默认限杀树，r4 2h 限时完成）→
  P2F=0 定谳。b0 偶然绿机理见 ④。
- **双 fork 收敛**（rally 同 session 双拉起，fork 2367883/2368020）：2367883
  完成 ①②③ 实现七面＋门禁四项＋电池 v1 十六腿（face c6f23ed4）＋r2
  P2F=1 键级停发归因；2368020 独立诊断同一 P2F 键，先手落 ④ 根修＋单元
  钉＋电池脚本扩 css-properties-values-api（18 腿）＋r3/r4 重发收全终账。
  收敛走 /tmp/zw-k136/NOTES.md CONVERGENCE 声明＋快照恒等核验
  （zombie-face-snapshot-regprops.patch）＋先手提交仲裁。
- **钉测**：style-system kc5r136_other_slots 8 钉（backdrop_bg_slot／
  highlight_slots_keyed 双名键控／highlight_arg_normalized_key escape 解码
  归一／no_diff_no_misstore Transparent 守卫／first_letter_part_bg_arm／
  first_line_part_channel／first_line_no_part_attr_none 外树 normal 胜依赖面／
  shadow_scope_rule_collected）＋engine kc5r136_other_slots_pins 5 钉
  （H1 backdrop exact-key＋跨键不串／H2 highlight 五别名同键读绿含 escape 与
  无闭合括号形／H3 nonParsable 回落／H4 part 通道 first-line/first-letter 含
  legacy 单冒号读形／H5 无规则零漂移）＋registered_properties
  test_computed_value_initial_keyword_resolves_registration_initial 1 钉
  ＋css-parser k133 缀后形转正断言＋k133 P3 钉契约更新复验绿。
- **门禁**：clippy 3 crate（style-system/engine/css-parser）-D warnings 绿；
  style-system 全量 2753P 0F；css-parser 全量 3422P 0F；engine lib
  4347P 3F＝红名单三键 s330/r3046/p99s 恒对账零新红（k135 账面同键）。
- **残 F 挂账 k137+**：::part 伪元素盒布局 7 键（getBoundingClientRect rect）
  ＋exportparts 跨树转发＋:has-slotted 22 键＋深链 FIXME（嵌套 shadow 逐跳）
  ＋shim gCS `.length`/item 语义（getComputedStyle-pseudo-with-argument
  parsable 5 断言中 length!=0 面仍红——shim `length: 0` 硬编码，color 断言
  已绿）＋computed_value `inherit`/`unset`/`revert` 关键字父链解析。
- 电池工件：`.acceptance/wptfyi-css/k136/results-{b0,b1}-{9 suite}`；差分
  脚本 /tmp/zw-k136/diff-k136.py（键级 Counter，status 判读 lower().startswith）；
  运行注记 /tmp/zw-k136/NOTES.md（r2→r4 三轮发射记录＋后台包装器限时教训）。
