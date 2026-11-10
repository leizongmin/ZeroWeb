# DC-2 color 批：light-dark() 建模 + system color dark 面板 + color-scheme 级联读取

日期：2026-11-11　批次：DC-2 color 上下文后续（light-dark/system-color/scheme 通道）　基线：results-v8（color 63 页 10,782P / values 256 页 2,213P）

## 1. 勘根（v8 失败归类，62 失败 subtest 五组）

| 组 | 页面 | 失败 | 通道 | 根因 |
|---|---|---|---|---|
| A | light-dark-image.html | 31F | shim plain 臂 | light-dark() 在 **image 属性**上下文无解析臂（解析失败整值丢弃→computed none）。css-color-5 color 上下文之外的泛化面，需 background-image 等属性 computed 序列化配套——挂账不追 |
| B | system-color-compute.html | 14F | 宿主 Rust cascade | 14 案全由 Menu 差分驱动：`named_colors.rs` menu→白（→Canvas 组静态映射），parent(light)/child(dark) 两 scheme 同白，`assert_not_equals` 失败。spec 语义：system color 在 computed 值时间解析为绝对色（css-color-4 §resolving-color-values），自计算则 inherit 链失效 |
| C | light-dark-currentcolor-in-color.html | 1F | 宿主+shim 双链 | `color: light-dark(red, currentColor)`——shim `_zwCCParseFn` 无 light-dark 分支（解析 null→宿主）；宿主侧 Rust 取支后 currentColor 实参 parse_color 不认→apply 拒→初始黑。spec：color 属性内 currentColor 自引用=继承值（css-color-3/4），期望 parent green |
| D | system-color-support.html | 12F | shim plain 臂 | 动态 createElement 元素走 plain 臂（`_zwCCSystem` 表 light 面板大多 ''）——scheme-aware 12 键 `assert_not_equals(light, dark)` 两读数同 ''（或同静态值）失败 |
| E | relative-color-with-system-color.html | 2F | Typed-OM | `hsl(from light-dark(currentColor, red) ...)` 走 `computedStyleMap` 相对色序列化，light-dark origin 折叠未建模（got ""）——Typed-OM 面挂账不追 |

## 2. 通道分野定谳（本批安全边界）

同一 system color 关键字读数走两条互斥通道（页面装配形态决定）：

- **宿主 Rust cascade**（静态 `<style>` 块 + 元素落宿主 selector）：system-color-compute 页、system-color-consistency 页。
- **shim plain 臂**（`createElement` 动态元素，`query` 闭包返 ''）：system-color-support 页。

证据：consistency 页 dark Field P/FieldText F 的读数形态只有宿主 Rust dark 臂能产（field named 白）；support 页读数恰为 shim `_zwCCSystem` 表值。由此本批改动互不越界：

- shim `_zwCCSystemDark` 表（仅 dark 面板 12 键）只影响 plain 臂 → support 页，**不碰** consistency 页（其 light 侧 ''=='' 等值钉面依赖空串口径，填 light 实值会成批翻红）。
- Rust dark 臂只扩 `menu`（compute 页驱动，consistency 页无 Menu 断言）。

## 3. 修复内容

1. `part01c.js`：`__zwPlainComputed` 加 `color-scheme` 属性分支——inline → 作者臂（qvMode）→ 沿父链首声明原文返回，token 判定归消费方。
2. `part11-computed-color.js`：
   - `_zwCCUsedSchemeDark(el)`：`__zwPlainComputed(el,'color-scheme',true)` 一次爬升 + 空格分词 token 判定（`light dark` 双 token → 保守 light，对齐 style-system `parse_color_scheme_dark` 的 `(true,true)→prefers_dark=false`；仅 dark → true；无声明/normal/only → light）。`\b` 判词在 `light-dark` 内会误断词，须空格分词。
   - `_zwCCParseFn` 加 `light-dark` 分支：恰好两组（`_zwCCTopSplit` 括号感知），支值暂存 AST `{kind:'lightdark', light, dark}` 延迟 resolve（currentColor 支须按属性上下文求值）。
   - `_zwCCResolve` case `lightdark`：used-scheme 选支递归——currentcolor 支走既有 currentcolor 臂（propCtx 'color' 父链语义保持，C 组钉面）。
   - `_zwCCSystemDark` dark 面板表（12 scheme-aware 键，值对齐 css-parser dark 臂近似：canvas 族 rgb(32,33,36)、文本族白）+ case `system` 分派（light 面板值优先，空串且 dark 面板有值且 used scheme dark 时取 dark）。
3. `css-parser/values/color.rs`：dark 臂扩 `menu` 键（rgb(43,43,43)，light 静态白不变）。遵循既有「无钉面不扩刀」原则：scrollbar 等宿主侧无钉面键不扩（既有 Scrollbar 恒等测试保留）。

## 4. 测量（v9 记录轮）

results-v9/（json 通道，v8 同口径；日志 zwv9-*）：

- **css-color：63 页 10,801P / 209F（对 v8 10,782P 净 +19，逐页零回归）**。
- **css-values：256 页 2,213P / 3,380F 对 v8 逐页全等零漂移**（注：v8 values F 实为 3,380——前档 3,373 系笔误，两轮页级全同）。
- 翻绿：system-color-compute 13→21（+8）、system-color-support 9→18（+9）、parsing/color-computed-relative-color +2（light-dark 解析臂意外收）。
- 剩余缺口（v9 后定谳）：compute 6F（box/text-shadow 2F＝颜色分量走无 scheme 的 parse_color；column-rule/caret-color 2F 同型；fill/stroke 2F＝gCS 读取层缺位）；support 3F（LinkText/VisitedText dark 分派被 light 静态值短路＋SelectedItem light 表缺键——补丁已落待 v10 测量）；currentcolor-in-color 1F（宿主 CurrentColor 序列化黑直出，paint/color.rs:35；shim 抢答面不覆盖作者臂声明）。
- 组 A（light-dark-image 31F）、组 E（Typed-OM 2F）、consistency dark FieldText 5F 维持挂账。
- 门禁：clippy CLIPPY-EXIT=0（target-clippy）、fmt 干净；§3 刀单 1/2/3 与本测量对应。

## 4b. 碰撞与接手（同 clone 姊妹 zeroweb-2-b0）

本批 v9 测量落定后，姊妹在同一工作树开始 text-shadow dark 贯穿改造（parse_transform.rs 05:36 半成品：parse_text_shadow(_list) 加 dark 参数+内部 parse_color_with_scheme，调用点未接线）——与 §3 刀单的 shadow 链同族。分工对齐：**姊妹接手** parse_box_shadow(_list)/parse_caret_color 签名+内部 dark、apply_advanced 四臂传 style.color_scheme_dark、supports.rs 三处传 false、shim selecteditem dark 分派接管与 v10 测量；**本批只提交** v9 已测量面（color.rs menu 扩键＋part01c color-scheme 分支＋part11 light-dark/system 建模与 selecteditem 补键——补键未经测量，随姊妹 v10 记录轮兜底）。compute 页 box/text-shadow/column-rule/caret 4F 归姊妹链。

## 4c. shadow 链落地（姊妹 b0 实测段）

链完成并推送（commit 6dd926911，13 文件 +154/−133）：parse_box_shadow(_list)/parse_caret_color 签名+内部 dark 贯穿（box-shadow 2150 色点、caret parse_one 闭包）、apply_advanced 四臂读 style.color_scheme_dark（column-rule-color 换 parse_color_with_scheme）、supports.rs 三处传 false（语法判定 scheme 无关）、测试调用点适配 7 文件（css-parser 5 + values/tests 2；tests/wpt-runner/reftest/tests.rs 最后一个 E0061 由姊妹先修，check --all-targets EXIT=0 确认）。多实例窗口取证：同树 4 姊妹（b9/e2/e5/68）交错编辑互补合并（text-shadow 单影链 4 hunk 由 b9 补齐我的 list 签名缺口；case 'system' dark 优先分派由 b9 落），cargo check 0.34s 缓存命中假象经 touch 强制单 crate 复检验证真态。主体批随 rebase 重写 640d5f4e4→639bed48a。分工终局：v10 记录轮+clippy 复检+测量回填归 b9（本段作者只补链与推送）；期望 v10 收 compute 4F＋support 3F，fill/stroke 2F 与 currentcolor-in-color 1F 维持挂账。

## 5. 挂账

- 组 A light-dark image 上下文（31F）：需泛化 light-dark + background-image 等属性 computed 序列化建模。
- 组 E Typed-OM 相对色 light-dark origin 折叠（2F）。
- consistency 页 dark FieldText 5F（UA 表单样式建模——element 侧读数缺口）。
- fill/stroke 属性 gCS 支持（compute 页 2F）。
