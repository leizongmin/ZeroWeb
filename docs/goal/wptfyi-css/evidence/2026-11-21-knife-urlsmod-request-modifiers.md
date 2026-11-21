# 刀 URLSMOD：url() 请求修饰符接入（css-values-5 §request-url-modifiers）

日期：2026-11-21
face：css-parser tokenizer＋values 层＋style-system 校验门/计算值＋engine computed/paint（16 二进制面文件）
判别：FACE-MARKER `URLSMOD request-url-modifiers`（b0=0/b1=1）＋M2（bin md5 异）＋S1/S3 ride 零扰动
电池：`.acceptance/wptfyi-css/knife-urlsmod-battery.sh`（values/backgrounds/fonts/images/text 五语料×双臂）

## 机制（规范锚）

css-values-5 [§request-url-modifiers](https://drafts.csswg.org/css-values-5/#request-url-modifiers)：
`<url>` 引号形可携带请求修饰符序列 `url("u" <url-modifier>#)`——

- known 三修饰符（function 形专属文法）：`cross-origin(anonymous | use-credentials)`、
  `integrity(<string>`（空串合法）、`referrer-policy(<ident>)`；
- unknown 单现容忍（ident/function 两形）；重复＝同名且同形二元键
  （ident/function 异形合法并存；大小写不敏感；function 形异参同拒）；
- 非 ident/function 词元＝parse error；无引号 url 不能接受修饰符（css-values-4
  §url-modifiers）；修饰符须居 url() 括号内、字符串之后；
- CSSOM 序列化 canonical：已知修饰符字母序重组（cross-origin < integrity <
  referrer-policy）、未知丢弃、url 串恒双引号（`\`/`"` 转义）。

## 词法单源与三通道同源

- tokenizer `parse_url_request_modifiers_at`：修饰符序列文法单函数（known_seen
  名键 / unknown_seen (名,形态) 键）；违例 → bad-url remnants + `Token::Error`
  （词法层拒收）；合法 → `Token::UrlWithModifiers(String, Vec<UrlRequestModifier>)`。
- 值层 `split_url_request_payload`（parse_background_image url 臂）复用词法
  单源（切片补 `)` 终止符）；无修饰符形与旧 `parse_css_url_payload` 等价
  （引号串同位闭引号；无引号含空白/括号/引号两侧同拒）。
- 写面 `validate_decl_value`（style-system apply.rs，白名单检查前置、不纳
  白名单）与 CSS.supports `decl_supported` 同源 `url_modifier_declaration_valid`；
  序列化 `fold_decl_specified` URLS pre-gate（`value_has_url_request_modifiers`
  判定 → `canonicalize_url_modifier_value`）——setProperty/supports/序列化
  三通道同文法（guidelines #41）。
- computed 发射（engine js_dom_bridge/computed_style.rs）与 image-set 串化
  （serialize_gradient.rs）同 `canonical_url_modifier_list` 单源。
- paint 降级：painter text/effects 三 match 臂 `Url(..) | UrlWithModifiers(.., _)`
  同径取内层 url（绘制语义＝修饰符请求通道，不影响图元）。

## 消费位

- @font-face src（parser/at_rules.rs）与 @import（parser/mod.rs prelude +
  at_rules consume_import_rule）：规则存活语义（提取内层 url）。
- property_validate is-image 判定、pipeline/extract.rs 图片抽取（@import
  除外守卫保持）。

## 钉测

`crates/css-parser/src/tests/url_request_modifiers_tests.rs`（5 测）：
invalid 页 38 形矩阵、serialize 页 33 形 canonical 矩阵（含 integrity("")
与三修饰符四序字母序）、值层拆分 round-trip、canonical_list 单元（排序/
剥除/转义）、词元级变体/Error 臂。

旧文法钉更新（css-values-5 语义，尾随 ident 由拒改容忍剥离，各附规范注）：
coverage_round5（quoted url 尾随）、tests_10（@font-face src payload）、
values/tests/advanced（background-image invalid）、style-system matcher
coverage_round2（CSS.supports）、shorthand basic（background 展开）。

## 电池结果（19:50:41 GATE-DONE，rc=0）

- 判别门全绿：FACE-MARKER b0=0/b1=1、M2 bin md5 异、RIDE 2 文件
  （rendering-compat 两 docs，S1/S2/S3 census 零扰动）、PATCH-MD5
  4404c208 绑定吻合（hunksplit 16 文件/24 hunks，generator 重跑
  ASSERTS-OK md5 逐字节复现＝树态与电池验证面一致）。
- **净增 +76 / 零 P2F**（values +76；backgrounds +1 探针定谳 flake 剔除，
  fonts/images/text SET-IDENTICAL）：
  - serialize 页 P 17→33（+16；终态 33＝预注册终态数吻合）
  - computed 页 P 1→23（+22；终态 23≈预注册 22 吻合）
  - **invalid 页 P 0→38（+38，超出预注册）**——预注册 v2 判「SET-IDENTICAL
    （b0 引号臂尾随恒 Error＝38 形全 P）」前提有误：b0 实测 0/38（HEAD
    对引号+修饰符形接受的语义与 WPT 期望相反），face 文法落库后全页
    F2P。预注册方向判断翻转但零损失，实账取代预估。
  - background-computed.html P 24→25（+1）**探针复跑定谳 flake 不入账**：
    同双 bin 复跑（19:58）双向对翻 b0 P=25/b1 P=24（calc 案 b0 Pass/b1
    Fail）；position-x 多值列表解析存在共享态污染（幽灵值
    `12px, 50%, 30px, -7px` 在页内不同案间随机现身）——与 url 域正交
    （face hunks 全 url 语义，fv1 position-y calc 案同页同源旧笔旁证）。
    证据：`.acceptance/wptfyi-css/probe-urlsmod-bgposition-flake-{b0,b1}.json`
    ＋NOTES-knife-urlsmod-bgposition-flake-requery.md。域外新 bug 信号
    （position 列表解析共享态）挂账独立刀域。
- **挂账（域外缺口非本刀回归；v1/v2 轮语料不同各自独立成立）**：
  - v2 轮（19:50 hunksplit）fonts 腿全零位移：font-face-src-list.html
    12P/5F 双臂同态，5 fail 全为 should-be-invalid 宽松接受
    （`expected false but got true`：`local()` 空、`format(unknown), tech(broken)`
    组合、junk 尾随等）＝src 描述符验证宽松既有债；预注册 6 valid F2P
    未兑现＝at_rules.rs 单 hunk src 臂对该页 wire 断言形无效，消费位
    有效性待查。
  - v1 轮（15:29 旧全量 patch 语料）font-face 页 0/8 SET-IDENTICAL
    （预注册 +6 未兑现）：8 失败全为 JS 桥错误（`cssRules` 的 @font-face
    rule 无 `.style` accessor——「Cannot read properties of undefined
    (reading 'style')」族），解析侧规则存活语义正常，CSSOM FontFace
    rule 物化另刀挂账。
  - import 页 3/10 SET-IDENTICAL（预注册吻合）：7 valid 恒 F＝@import
    CSSOM wire 物化缺口，维持挂账另刀。
  - url-import-referrer-policy 0/1 SET-IDENTICAL。
- 语料修正案：v1 电池（1529）cssom 腿 corpus 名 `testharness-css-cssom`
  不存在空跑；本刀五语料无 cssom 腿，客户端面哨兵由 values 主靶覆盖。

## 预注册（发射前落账）

- invalid 页 38：SET-IDENTICAL（HEAD 引号臂尾随恒 Error＝38 形 b0 全 P；
  b1 文法保 P）；任何 P2F＝文法回归信号。
- serialize 页 33：+32~33（case 0 无修饰符形可能 b0 已 P）。
- computed 页 22：+21~22（case 0 同上）。
- font-face 页 9：+6 F2P（6 valid：wire cssText 合成 src 链路成立；3 invalid
  SET-P 恒 ""）。
- import 页 10：SET-IDENTICAL（@import CSSOM wire 物化缺口域外，3 invalid
  空洞 P、7 valid 恒 F——挂账另刀）。
- 远哨（backgrounds/fonts/images/text）：SET-IDENTICAL；位移逐案归因。
