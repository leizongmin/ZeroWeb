# font-* 数值槽 calc 接入批：font-width 级联槽位分裂定谳 + 父级字体基准 + calc 负 % 域门

2026-11-14。承接 [trig 型别门](2026-11-13-trig-typegate-font-unit-cycles.md) §5 残余桶
第 1 项（G1 其余 6 槽位 calc 接入）的 font-weight / font-size-adjust /
font-stretch 三槽首刀，同窗口顺带收编 font-width 新名属性全链。

## 1. 起点

- `font-property-unit-cycles.tentative.html`（95 子测）g26 后 5P/90F：font-size
  五腿已回位，余 6 槽 math 形声明整体被丢（font-weight 恒 400 / font-stretch
  恒 100 默认值回退）。
- `font-width-*.html` 三页（css-fonts-4 §font-width-prop font-stretch 新名）：
  valid 28F 量级、computed 基线 12P、invalid 大面积丢。
- css-fonts 全量基线（g29 前，本窗口 A/B 锚）：**2,375P/2,627F/4T**（163 页）。

## 2. 归因链（font-width 级联槽位分裂）

gCS 直读探针（engine shim setProperty → host doc 重放链）三轮切面：

1. **t1**：`font-width: ultra-condensed` 单独声明 → 写面成功（mutation 队列
   正常落 host），gCS 读回 100%（父值）——apply 成功后又被覆盖。
2. **t3/t4**：`font-width` + `font-stretch` 双声明 → 终值在两声明值间硬币翻转
   （50 残留实证）。
3. **定谳**：font-width 与 font-stretch 未 canonical 化时**各占独立级联槽位**，
   写同一存储字段 `style.font_stretch`。font-width 单独声明时主循环 apply 成功，
   随后继承填充循环（inheritance.rs known_properties 全表迭代）发现
   `font-stretch` 槽「未出现在级联表」→ 用父值覆盖已应用的元素自身值；两声明
   并存时终值由 HashMap 迭代序决定。与 cascade.rs 既有 line-clamp-019 硬币翻转
   先例同型（canonical_property_name 注释有案）。

次级归因（calc 负 % 腿）：css-parser 把 `calc(-100%)` 归一成
`BinaryOp(Px(0.0), Subtract, Percentage(100.0))`（Px 锚伪影）；tree_valid 原拒
一切非 % Length 叶 → `calc(-100%)` 整式被拒（spec 期 valid，
css-values-4 §10.5 calc 解析期不 clamp）；arm 原 `pct >= 0.0` 域门又把已放行的
负 calc 值在存储期拒掉（computed 期应钳 0，font-width-computed 0% 腿 oracle）。

## 3. 修复（分文件）

- **cascade.rs**：`canonical_property_name` 新增 `"font-width" => "font-stretch"`
  （css-fonts-4 §font-width-prop 同一属性新名，word-wrap→overflow-wrap 同例）。
  canonical 化后同槽位按 CascadeOrder（position）竞争，后声明胜。
- **inheritance.rs**：继承填充循环跳过 canonical 别名——font-width 仍注册
  （decl_supported 尾门 `known_properties().contains` 驱动 CSS.supports/valid
  语料，注销即全拒），靠 `canonical_property_name(property) != *property →
  continue` 保证别名不入填充。parent_font_size 预应用注入（font 槽 math eval
  的 font 相对单位按父级字体解析，css-properties-values-api
  #dependency-cycles-via-relative-units；独立 apply 调用无父上下文回落
  `style.font_size`）。
- **apply.rs**：`eval_font_slot_math_percentage`（% 保持型 eval，叶直取 +
  `eval_calc_with_context` 兜底）；`font_stretch_percentage_tree_valid` 增
  `percent_ctx` 参数——Px 叶仅 Add/Sub 且兄弟侧含 % 叶时合法（css-values-4
  §10.5 型别统一锚语义），乘除子树 percent_ctx=false 保住 `calc(1px * 2)` 拒面；
  font-stretch/font-width 臂存储钳 `pct.max(0.0)`（解析期不 clamp、computed 钳
  0）。font_slot_math_write_gate 测试补 calc(-100%) 双属性永久断言。
- **apply_advanced.rs**：font-stretch/font-width 双名臂 math 形接入
  （`eval_font_slot_math_percentage(value, font_slot_font_px(style))`）。
- **css-parser**：`parse_font_weight` 域放宽 `[100,900] 百倍数` → 连续
  `<number [1,1000]>`（css-fonts-4 §font-weight-numeric，`1`/`101`/`150.25`
  语料腿；非整数 u16 round 存储）；`parse_font_stretch` % 过滤 `> 0.0` →
  `>= 0.0`（0% valid）；`fold_decl_specified` 接线 font-weight（Number 折）/
  font-stretch|font-width（Length 折，`calc(100% + 100%)` → `calc(200%)`）。
- **registry.rs / inherit.rs**：font-width 注册保留（初始值 100%、is_inherited
  两臂按 font_stretch 字段承接——显式 inherit/initial 关键字直应用面）。

## 4. 验证

- 单测：zero-css-parser lib **3,188 passed / 0 failed**；zero-style-system
  lib **2,531 passed / 0 failed**（test-guard 包裹，断言补齐后复跑）。
- 探针页五腿全 Pass（本地 zz-g29-probe，不入库）：font-width uc→50 双读、
  75%、234.5%、`calc(100% + 100%)` → 200%、
  `calc(1% * tan(atan2(1em, 1px)))` → 30（父级基准）。
- 单页：`font-property-unit-cycles` **20P/75F**（g26 后 5P → +15，直置 math
  腿全回位；G1 直置 35 腿全绿）。
- css-fonts 全量（rebase origin/main 2957c3e0a 后重建二进制，round-2）：
  **2,411P/2,587F/4T**，基线 2,375P/2,627F → **净 +36P**。页级：
  font-width-computed 12P→**28P**（calc(-100%) 双腿回位）、font-width-invalid
  **8P/0F 全页清零**、font-weight-invalid 0→**2P** 全清、font-weight-valid
  **21P/1F**、font-weight-computed **48P/10F**、font-size-adjust-valid
  **14P/5F**、font-weight-sign-function 3P/0F。
- round-1 复测（修复末段二进制）2375/2623 与 2369/2629 两轮 6P 摆动为
  修复中态伪红，不作账；round-2 为权威终账。

## 5. 残余桶（下批候选，页账驱动）

1. **sign(20cqw) 容器实参 + 乘法重排串化**：font-width-valid 2F /
   font-weight-valid 1F / font-width-computed 2F /
   font-size-adjust-valid 1F——`calc(100% + (sign(20cqw - 10px) * 5%))` 族，
   期望 `5% * sign(...)` canonical 序（g27 counter 容器参数同源）。
2. **bolder/lighter 相对权重表**（css-fonts-4 §relative weights）：
   font-weight-lighter-bolder 22F、font-weight-matching 14F、
   installed-fonts 6F。
3. **font-size-adjust-invalid 写面门 57F**：invalid 域全放行（`e.style`
   应拒未拒）。
4. **font-size-adjust computed/specified 串化**：computed 10F +
   valid 4F（`ex-height 0.5` 冗余 metric 剥离 canonical 形）。
5. **G2/G3 registered custom property typed compute**（unit-cycles 余
   75F 主体，60 腿 + font 单位基准 var() 腿）。
6. **inline decl alias 腿**：font-stretch-alias 1F——设置 font-width 后
   font-stretch 应显 inherit（CSSOM 双名 alias 完整性，级联槽位已统一、
   inline 声明槽独立）。
7. 150.25 串化保真（Absolute(u16) round 既有结构）+ gCS text-size-adjust
   computed 串化臂（g28 挂账）。
