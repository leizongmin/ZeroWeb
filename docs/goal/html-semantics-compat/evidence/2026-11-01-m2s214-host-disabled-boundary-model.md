# M2-S214：host 侧 is_effectively_disabled 边界集模型统一（DOM/CSS/自由函数三面）

- **日期**：2026-11-01
- **目标面**：s212 挂账承袭——host 侧 disabled 判定仍是平面走查
  （select/optgroup 边界语义未分型），与 s212 语料验证过的 shim
  `_zwFeDisabled` 边界集模型存在三处分歧
- **状态**：收口——zero-dom 881P+0F（新增 R5381 钉面测试）、zero-engine
  3673P+0F（与 s213 基线持平）、html_compat 集成 22P+0F；clippy 0 警告、
  fmt 过。**本切片零 shim 改动**，forms 语料路径（纯 JS shim）不受扰，
  零漂移由构造保证（engine lib 全绿佐证），corpus 不重跑（取舍见验证节）

## 勘根

- **两套 host 实现分歧定位**：
  1. `crates/dom/src/document/form_state.rs` `Document::
     is_effectively_disabled`（:disabled/:enabled 选择器权威，DOM query +
     style-system CSS + 约束校验三消费面）——单合并走查：legend 旗标
     `in_first_legend_of_disabled_fieldset` 一旦置位即屏蔽**所有**后续
     disabled fieldset（内层 fieldset 的 legend 豁免错误豁免外层 disabled
     fieldset，bug）；禁用源 = fieldset|select|optgroup 平面命中（嵌套
     optgroup 级联、select 不截断 optgroup 传播——两处与 shim 边界集分歧）。
  2. `crates/dom/src/focus.rs` 自由函数（tab 序 + webview 点击激活
     DisabledTarget + collect_form_data 三消费面）——自身 disabled 检查在
     tag 门**之前**（div[disabled] 也 true 的 legacy quirk，保留）；option
     分支平面走查任意 optgroup|select disabled 祖先（嵌套级联 + 不截断，
     同两处分歧）；fieldset 分支 per-fieldset `is_descendant_of_first_
     legend` 正确。
- **shim 参考模型逐行定谳**（part03 `_zwFeDisabled` s212，两臂相加）：
  ① 边界集臂（OPTION/OPTGROUP 一次性，自父起上行走查）——SELECT 查其
  disabled 属性即断（无属性截断）；OPTGROUP 仅向 option 传播（自身属性
  直判，无属性取 computed = 其 select 链上首个 SELECT 的 disabled，嵌套
  optgroup 不级联；对 OPTGROUP 自身截断）；HR/DATALIST/OPTION 截断；包裹
  层（div 等）穿行。② fieldset 臂（每层独立）——disabled fieldset 命中时
  豁免判据 = 走查位（链上下方直接元素子）为该 fieldset 首个 legend 元素
  子（`_fdWalk` 随链推进实现 per-fieldset 语义），豁免后继续上行（外层
  disabled fieldset 仍禁用）。spec 锚：#concept-option-disabled +
  #attr-fe-disabled。
- **消费面全景**：Document 方法 → query.rs element_matches_selector
  （DOM :disabled/:enabled）+ style-system matcher（CSS :disabled）+
  validation.rs 约束校验候选门；自由函数 → focus.rs is_sequentially_
  focusable（tab 序）+ engine js_dom_bridge → webview user_actions 点击
  激活门 + collect_form_data 表单条目构造。
- **既有锚点兼容**：R3277（tests_1a.rs）钉单层 fieldset/select/optgroup
  形态全兼容；html_compat `disabled_fieldset_blocks_interaction_and_
  submission` 钉单层 fieldset + legend 豁免全兼容。

## 施工

（host Rust 2 文件 + 测试 1 文件；shim 零改动）

- **form_state.rs 重写**：门 is_disableable_tag → ①自身 disabled 属性 →
  ②边界集臂（新私有 helper `option_boundary_disabled`，镜像 shim 边界臂
  逐行：SELECT/OPTGROUP/HR/DATALIST/OPTION/包裹穿行/guard 64，OPTGROUP
  computed 两级内循环）→ ③fieldset 臂（while parent_element_node 上行，
  per-fieldset 豁免复用既有 `is_first_legend_of`，豁免继续上行）。修复
  旧合并走查的 legend 旗标跨层屏蔽 bug。
- **focus.rs 自由函数委托**：可禁用族 → `doc.is_effectively_disabled`
  委托（消除第二套走查实现）；自身 disabled 前置检查 quirk 保留
  （div[disabled] → true 的 R3056 legacy 形态——`Document` 权威门仅认可
  禁用族，两面在非可禁用族上有意分歧，注释言明）；删除孤儿
  `is_descendant_of_first_legend`（唯一调用方即被替换的 fieldset 臂）。
- **tests_1a.rs R5381**：边界集七形态钉面（嵌套 optgroup 不级联 option+
  optgroup 自身两断言 / select 截断 / 包裹层穿行 / 禁用 select 直判 /
  optgroup 直接子传播 / optgroup 经 select 传播 / fieldset 逐层豁免——
  in-inner-lg 断言即旧模型返 enabled 的 bug 修复锚）+ 自由函数委托等价
  三断言 + div[disabled] quirk 保留断言。

## 验证

- zero-dom lib 全量 **881P+0F**（880 既有全绿 + R5381 一次过；日志
  evidence/logs/s214-host-rust-suites.log）。
- zero-engine lib 全量 **3673P+0F**（与 s213 基线持平——shim 零改动，
  bridge 消费面 collect_form_data/is_control_disabled 全绿）。
- html_compat 集成 **22P+0F**（DisabledTarget 点击门 + 单层 fieldset
  形态兼容实证）。
- guarded-clippy 0 警告；`cargo fmt -p zero-dom -p zero-engine --check`
  过。
- **forms corpus 不重跑（取舍）**：本切片零 shim 改动，corpus 走纯 JS
  shim 路径与 host walk 无交集，零漂移由构造保证；engine lib 3673P +
  bridge 消费面全绿为佐证。非零 shim 改动的切片仍按惯例跑 A/B join。

## 行为变更清单（新旧对照）

- option 在嵌套 optgroup 内（外层禁用内层未禁用）：三面由禁用改启用
  （spec 对齐）。
- option 在禁用 optgroup 内的未禁用 select 中：由禁用改启用（select 边界
  截断）。
- optgroup 自身在禁用 select 内：由启用改禁用（边界集传播补齐）。
- 内层 fieldset legend 子树内控件、外层 disabled fieldset 内：由启用改
  禁用（legend 豁免不再跨层屏蔽——bug 修复）。
- 自由函数面 option/optgroup 形态随委托同步收敛；button/input/textarea/
  select 形态逐 case 等价（fieldset 臂两实现本就同义）。
- 消费面影响：tab 序（option/optgroup 不可聚焦，无实效）、点击激活门
  （option 形态对齐 shim）、collect_form_data（option 非表单条目，无实效）、
  约束校验候选（嵌套 fieldset 形态修正）。

## 挂账

- part04 willValidate 臂内联 fieldset 走查共享化（s211 承袭——本切片
  host 模型已定谳，后续可指 `Document::is_effectively_disabled` 同源）。
- `__zw_has_attr` 非 lw 存量点位审计（s211 承袭）。
- onblur flake 族持续采样；beforeinput.tentative.html 16F 既有面（录入
  事件族独立评估）。
- 上游 customizable select 特性面 177F 挂起待拍板承袭（GB-20261031）。
- form owner table / parser form element pointer 缓行族（用户决策承袭）。
- form-nameditem override builtins 待决（承袭）。
- input-type-button click() 返回值面（共享通路高风险，未入刀，承袭）。
- 多进程 renderer SetDomSnapshot entries 化（s174 低优先承袭）。
