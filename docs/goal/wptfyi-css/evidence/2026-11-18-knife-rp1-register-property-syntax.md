# 刀 RP1 · registerProperty 语法严格门（2026-11-18）

## 认领与背景

- 认领：[CLAIM-knife-rp1-register-property-syntax.txt](../../../.acceptance/wptfyi-css/CLAIM-knife-rp1-register-property-syntax.txt)。
- 池：css-properties-values-api `register-property-syntax-parsing.html`（292 行语料，
  assert_valid/assert_invalid 全矩阵）。fv1 电池窗内 registered_properties.rs 严格 face
  以 ride-along 同态在场（不入 fv1 账），本刀单独立账。
- 前窗（fv1 收口前）Rust 侧已绿（4 钉测/fmt/clippy 零新增）；fv1 落库（rebase 前
  `3a0a573d8`，活号 `7effb5d25`）后接 shim WebIDL 面与 wire 面。

## 码面（3 文件）

1. `registered_properties.rs`——注册门严格 face（`validate_initial_strict` 族）：
   - typed 形 initial 必备 + 严格分型解析；universal 形带值过 `<declaration-value>`
     生产式（bracket 栈配对、顶层 `;`/`!` 拒、EOF 串合法、尾空白容忍）；normal 禁带值。
   - calc 严格分型（同族 ±、单侧标量 ×÷、len↔% 混合臂 MixedLenPct、嵌套 calc
     ci 前缀、var() 拒）；font-relative 长度族注册 initial 拒（Em/Rem/Ch/…）、
     视口族按名义窗口 FIXME(regprop-vp)；resolution 负值拒。
   - `<custom-ident>` 值 ci 禁 CSS 宽关键字 + `default`；字符串/URL/image/
     transform 全校验；乘子邻接（`<length> +` 拒）；混合 run 逐项同族门。
   - **emoji strip 回归修**：`strip_css_comments` else 臂 `unwrap_or(&s[i..i+1])`
     兜底参数急切求值——多字节字符（语料 `\1F914`→🤔 族）即便 `get(i..end)`
     成功也先切 `[i..i+1]` 越界 panic。改直接切 `&s[i..end]`（utf8_char_len
     合法首字节恒边界）。钉 5 枚转义/emoji pin。
   - 禁名 ci 面：valid_ident_literal 六禁名改 ASCII ci（语料 INHERIT/unsEt/
     REVert/deFAUlT 四形锚定）。
   - `<number>` 号符：valid_number_token 补 ± 消费（语料 `-109` 锚定；
     valid_integer_token 已有同形）。
   - **gen3 引号 url 函数形续扫修**（gen2 电池 P2F×6 定谳）：gen2 电池暴露
     P2F×6（custom-property-animation-image/url 各 3 子测 P→F，失败信息
     `initial-value does not parse for this syntax`——语料只锚未引号 `url(a)`
     形，测试用引号全 URL `url("https://…")`）。根因＝`scan_url_after_open`
     引号分支只把 `url(` 入栈即返，`valid_url_value` 单发 `i == b.len()` 判定
     漏扫余段（串 token + 闭括号从未扫描恒 false；通用形只在
     `valid_declaration_value` 循环内生效，universal 面因循环续扫侥幸绿）。
     修复：循环体析出 `scan_declaration_tail` 共享面（串跳读/url( ci 特判/
     括号栈/顶层 `;``!` 拒），引号 handoff 续扫收尾；未引号形路径逐位不变。
     钉 3 枚引号形 pin（`<image>`/`<url>` 引号全 URL + `<image>+` 混合 run）。
     gen2 证据存档 `results-knife-rp1/gen2-p2f/`。
2. `callbacks.rs`——syntax wire 保真：空串不再映射 None（空串保真过 parse 判
   非法）；WebIDL 缺省 `*` 由 shim 兑现。
3. `part06.js`——shim WebIDL 字典转换：`syntax === undefined`（含缺席）→ `'*'`
   （PropertyDescriptor syntax = "*"）；`null` → DOMString `"null"`（语料
   `assert_valid(null,"null")` 非缺省通道）；inherits 前门改按兑换后 syntax 判
   （verdict 等价矩阵核过：absent/'*'/'<length>'/'' 同判，null 提前至 JS 同
   TypeError）。

## 探针定谳（results-knife-rp1/bin/b1-runner，3 代演进）

- 首代 panic：registerProperty `🤔` 族 emoji 触 strip_css_comments 越界
  （见上）；修后二代探针 241P/5F。
- 残面 5 项两机制：禁名 ci 4 形 + `<number>` 负号 1 形——同轮修复。
- 三代（终态）：**主靶 246P/0F 全绿**（probe-rp1/pva-syntax.b1.json）。
- gen3 runner（引号 url 修后，md5 `619b7962`）复验：246P/0F 零回归
  （probe-rp1/pva-syntax.gen3.json）。

## A/B 定谳（五语料 × b0/b1，results-knife-rp1/）

电池 gen3（引号 url 修复在 face）20:23:22–20:36:49（GATE-DONE，双建 31s/30s，
M2 patch 往返/S2 ride-along 窗内零扰动法证绿，fp-count b0=1 b1=1，
b0 md5 `d12d87bd`/b1 md5 `619b7962`，全腿 rc=0/1 零 LEG-FAIL）。
gen2（修复前窗）曾 P2F=6 → gen3 修复复跑 **P2F=0 全域**；gen2 pva 双 JSON
存档 results-knife-rp1/gen2-p2f/。

| 语料 | b0 | b1 | ΔP | F2P | P2F | NEW/LOST |
|---|---|---|---|---|---|---|
| css-properties-values-api | 562P/799F | 653P/718F | **+91** | 89 | 0 | 10/0 |
| css-typed-om | 11074P/1285F | 同 b0 | 0 | 0 | 0 | 0/0 |
| css-animations | 516P/787F | 同 b0 | 0 | 0 | 0 | NEW=1/LOST=1（见异常 1） |
| css-variables | 301P/271F | 同 b0 | 0 | 0 | 0 | 0/0 |
| cssom | 1968P/1511F | 同 b0 | 0 | 0 | 0 | 0/0 |

**净账（认领口径 F2P−P2F）= +89，P2F=0 全域。**

F2P 分页：主靶 `register-property-syntax-parsing` 87（严格门 + shim WebIDL +
wire 保真全链）、`registered-property-initial` 2（strict initial 分型）。
NEW=10：animation-color-comma-list/color-space-list 两页——b0 页级早退收成
单 FAIL 条目（每页 1 条目），b1 展开 6 子测（含 1 PASS/页）＝NEW 含 FAIL×8
＋PASS×2，零 P2F 零 LOST，不入认领账；b0 页级早退机制未定谳（非本刀面，
挂观察）。

异常一案法证：
1. css-animations `display-none-dont-cancel.tentative.html` NEW=1/LOST=1：
   fv1 电池窗同页同类墙边陲 flake（60s CASE_WALL 边陲 synthetic 条目），
   ΔP=0，判 flake 级。

## 门禁

- engine lib 4188 全量过滤跑 registered_properties 4 钉测全过（gen3 修复后
  复跑含 3 枚引号形 pin）；fmt 清；定向 clippy 零新增（在案 lint 均域外：
  animation.rs unnecessary_cast ×2 既有提交态、part115 s424d 探针
  doc_lazy_continuation、es_module s419 too_many_arguments——对端在飞 WIP）。

## 残面池

pva b1 残面 718F（111 页条目，含残面页 97；census 于收账 JSON），头十二页：

| 页 | F |
|---|---|
| typedom.html | 166 |
| font-property-unit-cycles.tentative.html | 79 |
| at-property.html | 43 |
| registered-property-computation.html | 33 |
| at-property-cssom.html | 33 |
| custom-property-transition-mismatched-list.html | 24 |
| unit-cycles.html | 23 |
| registered-property-initial.html | 19 |
| at-property-animation.html | 19 |
| url-resolution.html | 18 |
| registered-properties-inheritance.html | 8 |
| var-reference-registered-properties.html | 7 |

头十二页 472F（unit-cycle 检测族 102F、@property 规则/CSSOM 族 76F、
typed-om set/computed 缺口 166F）；尾 85 残面页 ~246F 长尾。
