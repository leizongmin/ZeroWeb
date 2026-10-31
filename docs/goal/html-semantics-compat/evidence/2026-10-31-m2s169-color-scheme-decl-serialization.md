# M2-S169 color_scheme_decl 序列化面勘根（s165 挂账收口）

**日期**: 2026-10-31
**刀位**: s165 挂账「color_scheme_decl 继承序列化偏差——无声明后代经 parent-copy 获 decl（序列化现父值），spec 无声明时应表达 used/normal 回落语义」。s168c 后候选①。

## 勘根定谳（两分：销账 + 真修点）

**parent-copy 非偏差，销账**。css-color-adjust-1 §color-scheme-prop：`Inherited: yes`，无声明后代的 computed value = 父 computed value——gCS 序列化输出父值**正是继承语义**；「decl None→normal」回落只该发生在根/链首未声明（meta color-scheme 只入 used-scheme fallback 不改 computed，语料 `gCS(root).colorScheme === 'normal'` 钉死）。s165 挂账时「应表达 used/normal 回落」的疑虑混淆了两个面：used scheme（合成位 `color_scheme_dark`，normal 声明回落文档 scheme，s165 已按语料钉死）与 computed 序列化（继承父值）。形态矩阵 3 案实证现行为正确：

- 父显式 `dark` + 子无声明 → 子 decl=Some("dark") + dark 位 true（继承 computed 值 + used 随父）；
- 全链无声明 → decl=None（序列化 normal）+ used 回落文档 scheme；
- 父显式 `normal`（meta dark）+ 子无声明 → 子 decl=Some("normal")（computed 面继承 normal）+ dark 位 true（used 面回落文档 scheme）——**computed 与 used 两面分离且各自正确**。

**真偏差：显式声明值未归一化**。`resolve_keyword` Concrete 臂 decl 存原形串（`color-scheme: DARK   Light` → decl "DARK Light"），gCS computed 值断言面原样输出。CSSOM computed value 序列化要求 keyword 小写、token 单空格连接（cssom-1 §serializing-css-values）；used 面 token 解析 `eq_ignore_ascii_case` 不受影响，纯序列化面偏差。语料 14 文件无任何 `colorScheme` gCS 断言（全部走系统色渲染面），css/css-color-adjust 域语料不在本地——s165「语料未覆盖该形态」即指此。

## 施工（2 文件 rust，零 shim）

- `style-system/inheritance.rs`：Concrete 臂 decl 归一化——`split_whitespace` + token 小写 + 单空格 join（注释带 cssom-1 锚）。
- `engine/js_dom_bridge/computed_style.rs`：序列化臂注释对齐勘根定谳（decl=computed 值串语义、继承面/回落面分界）。
- `style-system/tests/color_scheme_synthesis.rs`：+4 形态矩阵案（继承父值 / 全链 None / 继承显式 normal / 归一化规范形），锚 css-color-adjust-1 §color-scheme-prop + cssom-1。

## 验证

- style-system lib **2467P/0F**（含新 4 案；归一化案修前红 `Some("DARK Light")` → 修后绿）。
- engine lib **3612P/11F**——失败集合与 s168c 基线逐案相同（fetch 族 9 + ua_display_li + style_sheet_from_handle_r113），零新增。
- color-scheme 语料 **24P/0F/0T**（s165 面 25P 含已删探针，24P 即基线）——归一化修零回归（语料输入全小写）。
- fmt / guarded-clippy workspace 全绿。
- reftest/product-smoke 未跑：decl 归一化仅 gCS 序列化通道（渲染用 used 合成位 `color_scheme_dark`），无渲染管线变更，沿 s165 先例记账。

## FIXME 销账

- s165 记账面 1「color_scheme_decl 继承序列化偏差」——**resolved**：parent-copy=继承语义（本账本形态矩阵实证），归一化面已修（CSSOM 规范形）。
