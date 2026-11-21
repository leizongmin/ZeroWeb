# 刀 GLBREAK：U+2007/U+202F 出可折叠空白集——GL∩White_Space 三员齐出（2026-11-21）

## 池与根因

- 恢复靶＝CHMETRIC 收账揭显的 P2F −4（baspglwj-121/124 各 2 断言）＋
  css3-text-line-break-baspglwj 全族 non-preserve 臂同缺口。
- 根因：`layout-engine/src/inline/inline_types.rs:584 is_collapsible_ws`
  ＝`char::is_whitespace() && != U+00A0 && != U+3000`——Rust
  `White_Space` 含 U+202F（NARROW NO-BREAK SPACE）/U+2007（FIGURE
  SPACE）（GL∩White_Space 全集 = {U+00A0 已例外, U+2007, U+202F}）：
  分词 split 在 GL 处开断点（collapse_split_words
  `text.split(is_collapsible_ws)`）＋折叠吸收（is_atomic_gl_suppressing
  注记自证「断行层身份消失」）。CSS Text 3 §3 document white space＝
  窄集（SPACE/TAB/LF/CR），U+202F/U+2007 非 document white space →
  应保留 advance 且无断点（UAX#14 GL：LB12/LB13 两侧禁断）。
- 页面形态：50ch 盒内 `39a+NNBSP(202F)+30b`——b0 假度量盒不溢出→
  「无需断行」平凡通过；b1 真度量（CHMETRIC-2 后）文本溢出，breaker
  在 GL 处误断 → normal/pre-line 两臂断层（2/4）；pre-wrap/
  break-spaces 两臂 preserve 路径语义本正确（不断，单行溢出）。

## face

| 文件 | 内容 |
|---|---|
| `inline_types.rs` | is_collapsible_ws 出表 U+2007/U+202F（U+00A0/U+3000 既有出表员保持）＋GL∩White_Space 三员齐出注记＋规范链接＋is_atomic_gl_suppressing 陈旧注记换新＋GLBREAK_FACE_MARKER（`#[used]` rodata） |
| 同文件 tests | `gl_class_chars_not_collapsible` 钉测：出表三员＋出表先例回归＋document white space 主员＋非 GL 成员原口径四组断言 |

- 探测性编辑注记：钉测落位的三次锚点尝试（02:53Z 两次锚
  invalid_utf8 fixture 不匹配弃用，第三次 אב fixture 锚成功）——仅
  末次生效态入库。
- 恢复注记：本 face 于 11-21 13:2x 因 `git checkout -- .` 误清扫，
  从 session transcript（d846aee9，记录级 timestamp 当日过滤＋
  已应用跳过重放）字节级恢复，钉测复跑全绿后入库。

## 预注册预期（发射前）

- css-text baspglwj-121/124：2/4→4/4（+4 恢复靶，non-preserve 两臂
  白空间断层修复）；121=U+202F、124=U+202F 族页。
- baspglwj 全族其余页（BA/SP/WJ 类对）：SET-IDENTICAL 为佳——0/4×~75
  是另一 UAX14 大刀域，本刀域外；位移逐案归因后才可入账。
- white-space-collapse-001：8/8 保持（NNBSP 保留经正确机制同形）。
- text 语料其余 + fonts/sizing/values 三哨 SET-IDENTICAL。
- reftest 腿另行：`make reftest` 全量（trailing-other-space-separators
  族＝reftest 页 css-text/white-space，pre 模式本就保留判零位移——
  电池探不到，reftest 兜 P2F 面）。

## 电池结果（unit zw-knife-glbreak-battery-1322，13:22–13:47）

- 门：FM `GLBREAK collapsible-gl-exclusion` b0=0/b1=1 ✓；M2 md5 异
  （b0 `4962d7e3d2f0518751d858482150690e` vs b1
  `44fe4ee1ee21641367613b703f533303`）✓；S1/S3 ride 窗内零扰动 ✓；
  八腿 JSON 全非空，EXIT rc=0。
- RECONCILE（13:47:12）：text gained=4 lost=0——
  `baspglwj-121` 2→4（+2）、`baspglwj-124` 2→4（+2），与预注册
  恢复靶逐键吻合；baspglwj 全族其余 111 页逐键 SET-IDENTICAL
  （016–052/060–118 的 0/4 UAX14 大刀域未动、062–071/092–096/
  120–131 的 4/4 平凡页不动）；`white-space-collapse-001` 8/8
  保持；fonts/sizing/values 三哨 gained=0 lost=0；全语料无 ASYM。

## 收账结论

- 判定：+4/−0 净增，P2F=0，无 ASYM，预注册全兑现——入库。
- CHMETRIC 收账时挂账的 P2F −4（baspglwj-121/124）至此清偿；
  baspglwj 全族剩余 ~0/4×74 页为 UAX#14 BA/B2/WJ/SY 类断行
  大刀域，另行挂账后续刀位。
- reftest 兜面：`make reftest` 全量 725/725（Layout 521/521、Text
  204/204）零位移——trailing-other-space 族 P2F 面清偿，face 在树态
  13:4x 跑。
