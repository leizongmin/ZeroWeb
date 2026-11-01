# M2-S207：form.action 文档 URL 反射特例——form-action-reflection-with-base-url 2F 收口

- **日期**：2026-11-01
- **目标面**：`html/semantics/forms/the-form-element/form-action-reflection-with-base-url.html`
  2F（empty-string action / missing action 应返 document's URL 而非
  base URL）（s206 后 misc 残面次刀候选序首位）
- **状态**：收口——靶面 3/3 Pass（2 靶键翻绿），forms 全树 keyed diff
  净 F→P=2（恰两靶键）+ onblur flake 域 ±2 双向摆动（在册），键漂移 0

## 勘根

- **spec 落点**：HTML §4.10.3 #dom-fs-action——action IDL 属性反射同名
  content attribute，**例外**：content attribute **缺失或空串**时 getter
  返 **node document's URL**（非文档基 URL，`<base href>` 不影响）；非空
  值对文档基 URL 解析（reflected URL 语义，net-api M5-S7 已承载）。
- **根因（回归考古）**：R5009 片 d 曾实现「missing/空 → 文档 URL」特例，
  但只活在 `__zw_parse_url` **不可用**回退分支（`else if (fv === '')`）；
  net-api M5-S7 引入解析优先分支后，空串落入 `__zw_parse_url('', 基URL)`
  → 解析结果 = 基 URL 本身——`<base href>` 存在时即偏离（无 `<base>` 场景
  基 URL == 文档 URL，form-action-reflection.html 3/3 Pass 掩面，语料
  base 变体才现形）。
- **语料锚定**：`<base href="/common/blank.html">` + 非根文档 URL 拉开
  两值——`action="a.html"` 对基解析（Pass，非本刀）；`action=""`/缺失
  期望 document.URL（两 F，本刀靶）。

## 施工

（shim 1 文件 + rust 测试 1 文件；语料零改动）

- **part03.js FORM action getter 分支重构**（:28014 区）：`fv === ''`
  先判——返 `_zwCurrentHref()`（文档 URL 通道：location.href 页面地址，
  `<base>` 无关；桥环境回落 `__zw_get_page_url()`）；非空才走
  `__zw_parse_url` 对 `_zwDocBaseURL()`（文档基 URL）解析——M5-S7 解析
  语义原样保留。旧死路回退分支删除。
- **part09.rs R5325 新桥测试**
  （test_r5325_form_action_reflection_document_url_vs_base）：`<base
  href>` + 非根 page_url 拉开两值，5 断言镜像语料三断言 + document.URL
  sanity + 空串与 document.URL 全等——语料 gitignore，「兼容性修复附
  常驻测试与修复同一提交」由已提交桥测试等价承载（s205/s206 先例）。

## 验证

- 拼接 shim（14 part include 序）node --check SYNTAX_OK。
- 靶面：`make testharness-html-semantics FILTER=the-form-element` →
  form-action-reflection-with-base-url.html 3/3 Pass（stash 基线同两键
  F）；form-action-reflection.html 3/3 Pass 持平；form-nameditem
  override builtins 1F 承袭保持（fix#19c 挂待决独立面）。
- forms 全树 stash A/B（FILTER=forms）：**净 F→P=4，其中恰 2 靶键**；
  另 2 键 F→P + 2 键 P→F 全落 **onblur flake 域双向摆动**（checkable-
  active-onblur 1 上、with-click 1 上 + active-onblur button 1 下、
  with-click 1 下；s204 +3/s205 +4 顺向后本轮 ±2 双向，在册 flake 归因
  再采样——本刀仅动 action getter 字符串分支，与 :active/onblur 时序域
  无交互路径）。键漂移 0 timeout 位移 0，两侧 (key,state) 对各 4711
  覆盖全同。
- zero-engine lib：**3666P+1F**（1F = ua_display_li 渲染流域在册键，基线
  同败非本刀；R5325 新测 +1 过）。
- fmt 本刀零 diff；guarded-clippy workspace 过（零警告）。

## 挂账

- **input/button formAction 同形特例缺口**：spec #dom-fs-action 同款
  「缺失/空串 → document's URL」例外，shim 无 formAction 反射臂
  （grep 空）；现有语料 formAction_document_address.html /
  formaction.html 全 Pass 系 document URL == base URL（无 `<base>`
  形语料），`<base>` 下潜在同形偏差不可见——待 `<base>` 变体语料或
  反射需求进场再勘。
- form-nameditem override builtins 1F（fix#19c 挂待决承袭）。
- onblur flake 域持续采样（本轮 ±2 双向样本）。
