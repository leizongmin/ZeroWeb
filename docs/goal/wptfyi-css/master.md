# CSS 子测试控制面

**2026-11-10 · Active**。目标净增 153,000 项（绝对线 182,392）。11-09 发布
快照基线 29,958 / 196,461（15.25%），待回收 152,434。
**DC-1 完成**：四焦点目录本地重跑复现快照（typed-om 25 / color 1,293 逐位
一致），缺口三分类入账——运行面断言失败 ~139.5k（Chrome 同面过 95%）、
未接语料 ~8.1k+缺文件 ~10.5k、通道外形态余量。
**DC-2 首批完成**：Typed OM API 入口面（part10 ~1,810 行：CSSStyleValue
类族 + CSS 工厂 + StylePropertyMap + attribute/computed/rule styleMap +
225 属性语法表；part06 styleMap 挂钩）——css-typed-om 簇全量重跑
**Pass 25 → 9830**（365 页，65 页零失败），残余 2,383 归四桶（简写长槽
缺面/computed 颜色管线/computed transform/`.any.js` 通道）。
**DC-2 次批完成**：css-color declared/computed 通路（part11 色值管线接线 +
part06 null 即拒 + part01 computed `has` 陷阱/宿主抢答覆盖）——css-color 簇
62 页 **Pass 1,293 → 7,229**（+5,936），逐页不降核对 33 增/28 平/1 回归
（relative-color -14 记账，次批后半恢复）；T48 五钉 Blink 对齐翻正。
**DC-2 次批后半完成**：computed 真色彩数学管线（part11-computed-color.js
1,871 行接线：13 空间全精度矩阵/relative color 解析/color-mix 混合/none
策略/序列化）+ part03 raw 影子/part01c raw 读取分流 + NeedsPlain 关键字
排除——css-color 簇 63 页 **Pass 7,229 → 9,318**（+2,089），逐页不降
**9 增/54 平/0 回归**（前轮 -14 豁免页恢复持平，computed-relative 0→1,100、
computed-mix 0→930）。
**DC-2 再批完成**：declared 声明面归一（part11-computed-color.js 覆盖段
+527/-17：relative origin 折叠/color-mix method 与权重规范化/calc 规范形/
rest alpha 行为表/浅面基座委托）——css-color 簇 63 页 **Pass 9,318 →
10,530**（+1,212），逐页不降 **2 增/61 平/0 回归**（contrast 透传修复后
恢复）。
**DC-2 四批完成**：语法面收批（part11-computed-color.js +205/-3：relative
通道表槽类型校验/color-mix hue 方法空间门/alpha() 声明面/color-layers
声明面与 computed 入口）——css-color 簇 63 页 **Pass 10,530 → 10,756**
（+226），逐页不降 **5 增/58 平/0 回归**（cc7 校验过严 -20 回归修正后）。
残余 254 归三桶：computed 校验/序列化残余（relative 69、rgb 21 等）、
light-dark 32、system-color 55。
**DC-2 语料扩面批完成**：DC-1 缺语料桶 13 目录一次接线（41-sh DIRS 扩面 +
runner 13 wrapper/dispatch 臂/CLI 子命令 + corpus 脚本显式映射 + Makefile
守卫 target + ci-shards css-v2-face-d 分片，提交 0d66d7997），全量 missing=0
（pin 315976933，blob sha 逐文件复核）。基线记录轮 13 corpus **8,562 子测试
/ Pass 1,381**（transitions 185 / om-view 545 / geometry 209 / animations 225
等），失败分桶全为引擎能力缺口（startViewTransition / registerProperty /
WAAPI / 过渡插值 / 布局 overflow 几何），通道本身零缺陷账。
**卡点**：无阻塞。
**下一步**：grid G2b——grid-template-* used 值（taffy detailed_layout_info
接线，gCS 通道需 layout 树，架构面独立批）；DC-3 全量对账（等批次平台期
dispatch 周跑，HEAD 已含 color/values/grid 各批增量）；light-dark/
system-color 建模残余。

**DC-2 values 批完成（11-11 凌晨）**：L4 数学函数进 transform/数值槽位 +
T54 inline style 写面值校验（三写臂：native CSD / shim writeDecl / shim
sel-handle 代理——corpus 实际写臂为 sel-handle 面）——values corpus
594 页 **Pass 1,540 → 1,770**（vs 基线 1,379 净增 +391），逐页不降 **0 回归**
（oracle 收窄后 v5f 口径；v5e 曾 -12 回归已修）。invalid 族 9 页翻正
（acos 族 63 全翻）；typed-om `.any.js` 变体通道同步落地（v5b 9,872 / +43）。

**DC-2 grid 批完成（11-11 凌晨）**：CSSOM 声明面校验 + canonical 序列化 +
supported property names has 面（native 17f30543c grid_cssom.rs + shim 面
接线：`_styleProxy`/`_zwMStyleProxy` 共享 `_zwStyleHas` 臂 + grid 归一臂
宿主回调，与 grid_cssom 单实现双通道）——锚页 grid-layout-properties
**13P → 94P（+81）**（`in` 门全解锁），探针页 20P/0F；残余 46F 分桶：
computed 值发射臂缺失（grid-line 族 computed 返 initial）、stylesheet
cascade 应用缺失（reset 不回落页面规则值）、span 形状序列化。
[证据](evidence/2026-11-11-dc2-grid-cssom-decl-face.md)

**DC-2 values serialize 批完成（11-11 凌晨）**：specified 值读面常量折叠
（trig/inverse-trig/minmax/clamp/round-mod-rem/hypot + canonical 序列化）三写臂
接线 + computed 值臂 NaN 规约（CSS Values 4 §10.12 censor-to-zero / §10.13
`calc(NaN)`）——values corpus 对重录基线（results-v5 v5d，oracle 收窄）
**1,548 → 2,213（净 +665），逐页不降 0 回归**（对上批账面 1,770 净 +443；
sin-cos-tan-serialize 40→270 全绿、acos 两页 0→62/60、minmax 族 +170、
round-mod-rem-computed +54）。**DC-2 color 上下文批完成**（提交 d1dd30e7a，三修合并）：currentcolor 三参
属性上下文（`__zwComputedColor(raw, el, prop)`，非 color 属性先试本元素，
css-color-4 §5.3）——currentColor 子测试 +26（relative-color +21、alpha +5）；
part03 raw 影子放宽（归一变形且原值含 none 即存原值）+ srcBare 分量语法跟随
证伪回退（declared 写侧剥 % 后 store 无语法）——hsl/hwb computed none 组
保形 8 案恢复（cc8 通过系旧 serial 强加 % 与影子漏存两错误抵消的 accidental
pass）；color-mix 插值空间别名折叠 `'in xyz' ≡ 'in xyz-d65'`——xyz none 组
13 案恢复（v6 记录轮为 pre-alias 测量态 −13）。v7 HEAD 复录终局核对。
[证据](evidence/2026-11-11-dc2-values-serialize-r2320-color-cc-ctx.md)
（values 侧 8b5fc9ebd：values corpus v5d 1,548 → 2,213 净 +665 逐页不降；
已知缺口挂账：atan2 混型实参、trig 实参时间单位各 1F。color 侧收口探针
法证与 v8 终账（对 cc8 净 +26 逐页 0 回归）见
[证据](evidence/2026-11-11-dc2-color-raw-shadow-mix-alias.md)。）

**DC-2 color 通道批完成（11-11 上午）**：light-dark() 双通道建模 + system
color dark 面板 + color-scheme 级联读取（提交 639bed48a）——shim part11
`_zwCCParseFn` light-dark 分支（支值暂存 AST 延迟 resolve，currentColor 支
走属性上下文）+ `_zwCCUsedSchemeDark` 沿父链合成（空格分词，light dark 双
token 保守 light 对齐 parse_color_scheme_dark）+ `_zwCCSystemDark` 12 键
dark 面板 + part01c color-scheme 级联分支（inline→作者臂→父链）+ Rust
dark 臂 menu 扩键——color corpus **10,782 → 10,801（净 +19），逐页 0 回归**
（system-color-compute +8、system-color-support +9、parsing/relative-color
+2 意外收；values 2,213P 零漂移）。通道分野定谳：support 页（动态元素）走
shim plain 臂、compute/consistency 页走宿主 cascade，两改动面互不越界
（consistency 页 ''=='' 空串钉面零变化）。
**DC-2 shadow/caret dark 贯穿批完成（11-11 上午，提交 6dd926911）**：
parse_color_with_scheme dark 贯穿 text-shadow/box-shadow/caret-color/
column-rule-color 四属性解析链（apply_advanced 四臂读
style.color_scheme_dark、supports 语法判定传 false、测试调用点适配）+
case 'system' 分派 dark 优先（LinkText/VisitedText 非空 light 静态值键
light 优先则 dark 分派永不生效）——color corpus **10,801 → 10,808（净
+7），两批合计 10,782 → 10,808 净 +26 逐页 0 回归**（compute shadow/caret
差分 4F、support LinkText/VisitedText/SelectedItem 3F 全数兑现；values
零漂移；干净树 cargo check EXIT=0——zwv10-check.log 101 为提交前旧态
伪信号）。残余挂账：light-dark image 上下文 31F、Typed-OM 相对色 2F、
currentcolor-in-light-dark 宿主黑 1F、fill/stroke gCS 读数 2F、consistency
dark FieldText 5F（UA 表单样式建模）。
[证据](evidence/2026-11-11-dc2-lightdark-scheme-system.md)

**DC-2 light-dark image 批完成（11-11 午后）**：css-color-5 §funcdef-light-dark
图像上下文贯穿四属性（提交 8c166641d）——background-image/mask-image 共用 +
list-style-image + border-image-source 三解析器 LightDark 变体（恰两参、支内
none 合法、色值支/arity 递归拒）+ cascade 期按 style.color_scheme_dark 选支
展开（computed 一次定案，inherit 继承选中支）+ 支内 none 展开 TransparentImage
（序列化 image(rgba(0, 0, 0, 0)) 区别顶层 none）+ paint 五臂补 exhaustive +
shim 写面 light-dark 前缀浅校验（invalid 组拒，白名单外 passthrough 零回归）。
color corpus **10,808 → 10,840（净 +32）逐页零回归**（共享树提前兑现，v11=70
记录轮 build 含本批在树 hunk：light-dark-image 1/32 → 30/32 本批 +29 全数兑现；
compute fill/stroke +2 与 currentcolor +1 归同树 70 的 fill/stroke hunk 非本批；
values 2,213P 零漂移；纯净 HEAD 终账归 DC-3）。挂账：content 图像 light-dark
1F、image-set/cross-fade 嵌套 1F、UA 表单 dark FieldText 5F。
[证据](evidence/2026-11-11-dc2-lightdark-image.md)

**DC-2 fill/stroke gCS + currentColor 宿主解析批完成（11-11 午后）**：fill/stroke
三层缺位补齐 + color 属性 currentColor 宿主后置解析（提交 179d04529）——apply
fill/stroke 两臂 parse_color_with_scheme（system color dark 传播，同
apply_advanced 四臂形态）+ inherit_property fill/stroke 继承臂（SVG2
Inherited: yes，R4280/R5187「继承由后续切片补」落地）+ inheritance.rs 继承完成后置
解析 `style.color == CurrentColor → parent.color`（css-color-3/5；shim plain 臂
已有同语义，宿主 cascade 面补齐——双通道一致）+ computed_style fill/stroke 序列化
臂（None→SVG2 初始 fill 黑/stroke none）——system-color-compute **25P/2F →
27P/0F 页面清零**、light-dark-currentcolor-in-color 1F→1P，本批面净 +3（v11
同轮 10,808→10,840 净 +32 含 e2 image 链 +29，逐页零回归按面分页归属；values
2,213P 零漂移）。残余挂账：consistency dark FieldText 5F（UA 表单样式建模）、
Typed-OM 相对色 2F、fill/stroke none/transparent 存储串区分（存储层变体）。
[证据](evidence/2026-11-11-dc2-fill-stroke-gcs-currentcolor-host.md)

**DC-2 grid-line 组合形批完成（11-11 午后）**：`<grid-line>` 组合形模型贯穿
（提交 c535b64b6 + fmt 收敛 9f025bc66）——GridLineValue 结构变体
（Line{line,name}/Span{count,name}）+ parse_grid_line 全文法重写
（css-grid-2 §7.1：`[<integer> && <custom-ident>?]` 任意序、`[ span &&
<integer>? && <custom-ident>? ]`、span2 无空格 token 边界纠正为 custom-ident）
+ grid_line_to_css canonical 序列化 + converter 落位 NamedSpan——勘根修正
G1 批猜测（46F 实为组合形 21F + template used 24F 两族，发射臂/cascade 臂
早已存在非缺口）；css-grid corpus v13 全量 635 页逐页对基线 **净 +26 零回归**
（锚页 grid-layout-properties 94P/46F → 115P/25F：3 组合形 × 7 属性全翻，
pair/area shorthand 走同一解析路径连带；grid-area-computed 8→13 含 unicode
ident）；锚页残余 25F 全为 grid-template-* used 值族挂账 G2b（taffy
detailed_layout_info 接线，架构面独立批）。
[证据](evidence/2026-11-11-dc2-grid-line-composite-model.md)

[执行契约](../wptfyi-css.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | **完成** | [基线与三分类](evidence/2026-11-10-dc1-baseline-and-classification.md)；快照基线 29,958/196,461（[coverage-2026-11-09](../../compat/wptfyi/coverage-2026-11-09.md)） |
| DC-2 按簇收敛 | 十三批完成 | [Typed OM API 入口面](evidence/2026-11-10-dc2-part10-typed-om-api.md)：typed-om 簇 25 → 9830；[css-color 色值管线](evidence/2026-11-10-dc2-part11-color-pipeline.md)：color 簇 1,293 → 7,229；[computed 数学管线](evidence/2026-11-10-dc2-part11-computed-color.md)：color 簇 7,229 → 9,318；[declared 声明面归一](evidence/2026-11-10-dc2-declared-normalization.md)：color 簇 9,318 → 10,530；[语法面收批](evidence/2026-11-10-dc2-color-layers-alpha-decl.md)：color 簇 10,530 → 10,756；[语料扩面 13 目录](evidence/2026-11-10-dc2-corpus-expansion.md)：新 corpus 8,562 子测试 / Pass 1,381；[values 数学槽位 + T54](evidence/2026-11-11-dc2-values-math-slots-t54.md)：values 簇 1,540 → 1,770；[grid 声明面 + has 面](evidence/2026-11-11-dc2-grid-cssom-decl-face.md)：锚页 13 → 94；[values serialize r2320 + color currentcolor ctx](evidence/2026-11-11-dc2-values-serialize-r2320-color-cc-ctx.md)：values v5d 1,548 → 2,213、color +26 子测试；[light-dark + system dark 面板](evidence/2026-11-11-dc2-lightdark-scheme-system.md)：color 10,782 → 10,808 逐页 0 回归（含 shadow/caret dark 贯穿批）；[light-dark 图像上下文](evidence/2026-11-11-dc2-lightdark-image.md)：color 10,808 → 10,840 逐页 0 回归（本批 +29，同树 fill/stroke +3 归 70）；[fill/stroke gCS + currentColor 宿主解析](evidence/2026-11-11-dc2-fill-stroke-gcs-currentcolor-host.md)：v11 10,808 → 10,840（本批面 +3，compute 页清零）；[grid-line 组合形模型](evidence/2026-11-11-dc2-grid-line-composite-model.md)：css-grid v13 净 +26 零回归、锚页 94 → 115（fmt 收敛 9f025bc66） |
| DC-3 全量对账 | 待执行 | 验收线 182,392 项通过，同 revision 周跑产物待补 |

碰撞记录：11-11 凌晨 union 树 engine 全量 7F（img/svg settle + websocket
族）——values/T54 批写面与该 7 测试零交集（part85/part45 无 style 用法），
归因待 color/grid 链在途 hunks 或 main 漂移。**定谳（11-11 凌晨，HEAD
worktree 判定）**：HEAD be6703d58 冷编译后同 7 测试全红（0.17s、同断言：
img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 /
fetch_filtered_response_type_matrix / iframe_service_worker_messages /
response_body_readable_stream_r2967 / t8k_websocket_error_nonterminal /
t8k_websocket_event_flow）——**main 既有欠账，与任何在途 hunk 无关**。
r2320/color ctx 批 make test 全量 3835P/7F，7F 即该既有组，本批新增测试
全绿，不随本批携带。

遇到共享文件或语料 pin 冲突时，记下对应 goal、范围和可并行工作。
