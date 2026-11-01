# M2-S212：disabled 语义面三缺口——焦点门 + matches 四伪类臂 + select/optgroup 传播边界走查

- **日期**：2026-11-01
- **目标面**：input-disabled-fieldset-dynamic（fieldset 动态翻转 +
  matches 同 turn 可见性 + focus 不可聚焦）+ number-disabled（自身
  disabled focus 不落焦）+ option-disabled-optgroup（concept-option-
  disabled 六测）+ option-disabled-optgroup-wrapper（wrapper 穿行三测）
  合计 4 份语料 11 键在册 F
- **状态**：收口——四份靶语料全 Pass（11 键 F→P 全闭合）；forms 全域
  A/B 净 F→P=7 恰 7 靶键（3 键 HEAD 已 P 承前保持），P→F=1 在册
  onblur flake 族，键漂移 0 其他位移 0

## 勘根

- **入场**：s210 后 misc 残面盘点（s211 定按钮激活门后余面）——
  disabled 语义面四份语料 11 键。
- **缺口一（焦点门）**：mousedown 默认动作聚焦路径（part06
  `_zwIsFocusableSel`）与宿主 pre-focus 通道对 INPUT/TEXTAREA/SELECT/
  BUTTON 无 actually-disabled 判——disabled 控件可落焦。spec
  §6.6.1 focusing steps 对 actually disabled 元素 no-op（§4.10.5.1.1
  #concept-fe-disabled + §4.10.15 fieldset 传染）。
- **缺口二（matches 伪类臂）**：`:disabled`/`:enabled`/`:read-only`/
  `:read-write` 在 matches trap 无 JS 臂——`__zw_matches` 读 dom_html
  applied 快照（callbacks.rs:1340），同 turn setAttribute/
  removeAttribute 写 lw 层不可见（R4200 同款机制），fieldset 动态
  翻转断言假红。selectors-4 #enableddisabled：`:enabled` = disableable
  且非 actually disabled；#ro-rw：`:read-write` = textarea 或文本
  可编辑 input type 且非 readonly 非 disabled（13 type 全表：
  ""/text/search/url/tel/email/password/date/month/week/time/
  datetime-local/number，与 host `is_text_editable_input_type` 同表）。
- **缺口三（select/optgroup 传播）**：`_zwFeDisabled` s211 版只有自身
  disabled + fieldset 传染——select.disabled 后 option/optgroup 不禁用。
- **传播模型三轮收敛（勘根转折）**：①首版平面走查（select/optgroup/
  fieldset 混走）→ option-disabled-optgroup "nested optgroup"/
  "select in optgroup" 2 案真 P→F——语料钉死嵌套 optgroup 不级联、
  optgroup 内 select 截断传播，平面走查过度；②改父级单层 computed
  （OPTION = 自身 | 父 OPTGROUP computed | 父 SELECT）→ 六测全过但
  wrapper 语料 2 案回红——`<optgroup disabled><div><option>` 期望
  option :disabled **true**，直接父是 div 非边界；③取 spec
  concept-option-disabled 原文定谳**边界集走查**：OPTION 自身属性后
  走查祖先——SELECT 查其 disabled 属性、首个 OPTGROUP 即决（查
  computed 禁用 = 自身属性 | 其 select 链，嵌套不级联）、HR/DATALIST/
  OPTION 截断 false、其余包裹层（div 等）穿行；OPTGROUP 自身不从
  OPTGROUP 级联（OPTGROUP 祖先即截断）。包裹层穿行与嵌套不级联在
  同一模型下同时成立——九断言 + wrapper 三测全验算通过。
  https://html.spec.whatwg.org/multipage/form-elements.html#concept-option-disabled
- **不能伤的键**：R5378 激活门（s211，`_zwFeDisabled` 供体面——本轮
  只扩其语义不换签名）；R5328 validity 面（s210）；R313 自身
  disabled click 臂；R3902 :focus / R4200 :modal/:popover-open /
  R57 :invalid/:valid 既有 matches 臂（本轮新增独立分支不重排）。

## 施工

（shim 3 文件 + rust 测试 1 文件；语料零改动——四份靶语料上游既有）

- **part03 `_zwFeDisabled` 扩臂**：外层 fieldset 平面走查内新增
  depth-0 OPTION/OPTGROUP 边界走查臂（SELECT 查 disabled 属性 /
  OPTGROUP 对 OPTION 查 computed 两级（自身属性 | 其 select 链——
  同边界集递归一层）、对 OPTGROUP 自身截断 / HR、DATALIST、OPTION
  截断 / 包裹层穿行 / guard 64）；fieldset 传染由外层平面走查独立
  承担（两臂相加）。头注释同步 s212 口径。
- **part04 焦点门**（模态 inert 门后、detached 臂前）：disableable
  七族 tag 门 + `_zwFeDisabled` 命中 return（focusing steps no-op）。
- **part06 `_zwIsFocusableSel`**：INPUT/TEXTAREA/SELECT/BUTTON 分支内
  `_zwFeDisabled(sel, null)` 门（mousedown 默认动作聚焦与宿主
  pre-focus 共用谓词；IFRAME/SUMMARY 分支拆出不受门——无 disabled
  语义）。
- **part04 matches 四伪类臂**（:popover-open 块后）：`:disabled` =
  `_zwFeDisabled`；`:enabled` = 七族可禁且非禁；`:read-write` =
  TEXTAREA 或文本可编辑 INPUT type 且非 readonly（lw 读）且非
  disabled；`:read-only` 补集。带 spec 链接注释
  （#selector-disabled、selectors-4 #enableddisabled #ro-rw）。
- **part09.rs R5379 六腿 → 九腿**：腿 A fieldset disabled 后代 focus
  不动；腿 B removeAttribute 可聚焦 + B2 同 turn matches 四伪类
  （':disabled'/'0'/':read-only'/'0'/':read-write'/'1'/':enabled'/'1'）
  + B3 setAttribute 重禁同 turn '1' 再解除；腿 C number-disabled 自身
  focus 不动；腿 D IDL disabled=false 解除；腿 E disabled select 内
  option matches true + 解除翻 false；腿 F 嵌套边界（disabled select
  直子 [_og1,_og2,_oIn1,_oIn2,_oDirect] 期望 "10101"——嵌套 optgroup
  及其 option 不级联；select-in-optgroup 期望 option '0'）；腿 G
  wrapper 穿行（div 包裹 option 于 disabled optgroup → true +
  optgroup.disabled=false 同 turn 翻 false + disabled select 内 div
  包裹 option → true）。

## 验证

- 拼接 shim（15 part include 序）node --check SYNTAX_OK。
- R5379 单测 **1 passed**（九腿；首轮 grandparent 臂条件 bug——
  `if (_fdTag === 'OPTGROUP')` 父 tag 判断使嵌套 OPTGROUP 也吃两级
  computed，腿 F 期望 "11101" ≠ "10101" 抓住，改
  `_fdSelfTag === 'OPTION' && _fdTag === 'OPTGROUP'`；随后 wrapper
  2 案回红驱动第三轮收敛至边界走查模型）。
- 四份靶语料全 Pass：option-disabled-optgroup **6/6**、option-
  disabled-optgroup-wrapper **3/3**（2F 翻绿）、input-disabled-
  fieldset-dynamic **1/1**、number-disabled **1/1**（终跑日志
  evidence/logs/s212-option-disabled-final.log、s212-input-disabled-
  fieldset-dynamic-final.log、s212-number-disabled-final.log）。
- forms 全域 A/B（WIP-final vs stash 基线，(key,state) 对全测试名
  sort -u join——带空格测试名须整行键提取，键级叉乘假 P→F 坑 s139
  在册）：两侧各 **4700 对零键漂移**；**净 F→P=7 恰 7 靶键**
  （wrapper 2 + option-disabled 3 + number-disabled 1 +
  input-disabled-fieldset-dynamic 1），**P→F=1**（checkable-
  active-onblur-with-click——在册 onblur flake 族 s209/s210/s211
  承袭摆动；上轮 WIP 日志复跑同键 F→P 反向回摆，族内闭环非回归）。
  日志 evidence/logs/s212-forms-wip-final.log、
  s212-forms-stash-baseline.log。
- zero-engine lib 全量 **3672P+0F**（R5379 计入）。
- guarded-clippy 0 警告；`cargo fmt -p zero-engine -- --check`
  EXIT=0（裸 `rustfmt --check` 对本仓测试文件报全文件重排噪声——
  不读 rustfmt.toml 配置所致，口径以 cargo fmt 为准；workspace 级
  唯一漂移 apps/compositor/seccomp_linux.rs 归口上游 d4768c825）。

## 挂账

- **传播模型 host 侧分歧**：crates/dom `form_state.rs
  is_effectively_disabled` 仍是平面走查（select/optgroup 边界语义
  未分型）——本轮 JS shim 臂先行定谳边界集模型，host 面与 CSS 面
  （selector engine 原生 :disabled）统一挂账独立切片。
- part04 willValidate 臂内联 fieldset 走查共享化（s211 挂账承袭，
  独立小刀评估）。
- `__zw_has_attr` 非 lw 存量点位审计（s211 挂账承袭）。
- onblur flake 族持续采样（本轮 with-click 摆入 1 键 + spacebar 2 键
  摆出，族内双向摆动闭环）。
- r5296_single_line_inflated paint 全量并行失败 solo 复绿（渲染流域
  观察，shim 字符串不入 paint 测试路径，非本轮机制关联）。
- button-click-submits 文件级 Timeout 伪影（HEAD 既有形态，本轮
  A/B 双侧同现 16 Pass + 同 Timeout，s211 口径承袭）。
- matches `:read-only`/`:read-write` 对 FIELDSET 自身 readonly 语义
  未建模（语料无形态，FIXME 注释已记）。
- 上游 customizable select 特性面 177F 挂起待拍板承袭（GB-20261031）。
