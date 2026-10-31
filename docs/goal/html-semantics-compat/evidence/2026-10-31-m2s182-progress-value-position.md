# M2-S182 progress value/position 实际值 getter 落地——13F 单根刀位（s181 分型定谳承接），shim 2 文件零 rust

## 入场定刀位

s181 分型表定谳的下一单根刀位：HTMLProgressElement `value`/`position`
IDL 缺位（forms 全树 13F）。日志 expected/got 直接定谳无需探针再证：
`position` → undefined（indeterminate 应 -1）；`value` → 裸属性串
（应 parse + clamp [0,max]，非法 → 0）。

## 勘根（拦截链）

1. `position`：无任何分支 → undefined。part04 反射区 R5009 `max` 分支
   （part04:12471）已落 max 反射（失败/≤0 → 1），但 value/position 缺位。
2. `value`：part03 属性 getter 陷阱 `prop === 'value'` 块（part03:26285）
   的 R5009 片 d `_rvSkip3` 名单（BUTTON/LI/METER/PROGRESS/PARAM/OPTION）
   直读属性面**先于** part04 数值分支接手——PROGRESS 落 `attr 缺省 ''`
   直读出口，part04 永不可达。LI/METER 既有 `!== 'LI' && !== 'METER'`
   放行先例。
3. indeterminate 门第一版误用 get 族判缺属性——lw/get 读器缺属性回落
   空串非 null（`__zw_get_attr_lw` RemoveAttr 命中返空串），`raw != null`
   恒真 → position 恒 0。改 has 族（`__zw_has_attr_lw`/`__zw_has_attr`/
   `__zw_has_attr_handle` 返 '1'/'0'，R3901 INPUT value 模式面同款口径）。

## 施工（shim 2 文件，零 rust 零协议）

- **part04.js**：`_zwMeterActualValues`（R4002）邻位新增
  `_zwProgressActualValues`——value = value attr 前缀浮点解析（缺省/失败
  → 0）clamp [0,max]；position = value attr 缺席（indeterminate）→ -1，
  否则 clamp 值/max；max 同款前缀解析（失败/≤0 → 1，与 R5009 max 分支
  同款 trailing-junk-忽略——reflection-forms 'progress.max setAttribute
  ("7\v")' 钉态）。R5009 max 分支邻位新增 PROGRESS `value`/`position`
  getter 分支（setter 走既有泛型反射写内容属性——setter 后 attr 在即
  determinate 化，progress-2 'position equals 0' 链）。
- **part03.js**：`_rvSkip3` 放行门加 `&& _rvTag3 !== 'PROGRESS'`（LI/
  METER 同款），R5009 片 d 注释同步（PROGRESS 数值分支在 part04）。
- spec：HTML §4.10.14 the progress element
  （https://html.spec.whatwg.org/multipage/form-elements.html#the-progress-element）。

## 验证

- 定向：`make testharness-html-semantics FILTER=the-progress-element`
  **18/18 全 Pass**（13 靶案 F→P + progress-max-setting 5 既有 Pass）。
- **forms 全树 join**（8 job，/tmp/s181/forms_s182.log）：4686 common 键
  (key,state) 严格 join 对 s178 基线（/tmp/s178/forms_new.tsv 4697 键）
  ——**F2P = 13 恰靶案、P2F = 0、其他位移 0**；全树 4310P/389F/12T
  （402F − 13 = 389F ✓）。两侧提取空白归一后 join（s178 提取做过空白
  归一，GAINED/LOST 首轮 94/80 为提取口径假位移，归一后残差 = 文件级
  截断族：12T 新报 = beforeinput 1 + form-submission-target 5 +
  button-click-submits 1 + maxlength-number 1 + radio-focused 1 +
  customizable-select optional 2——s178 同日 0T，负载偶发；solo 复跑
  the-button-element 49P/6F 与 s178 基线同键同态钉死截断非回归）。
- **interactive-elements 483 键 join 零位移**（439P/44F 两侧同数）。
- **engine lib 3635P/1F**（test-guard 包裹；1F = ua_display_li 渲染流
  在册，s180 账本同键同态——fetch 族清零延续）。
- shim 装配序拼接（part01,01b,02-07,08a,08,08c,08b,08d）node --check
  SYNTAX_OK（114024 行）；rust 零 diff（fmt/clippy 无对象）；webview/
  renderer lib 免跑（零 rust diff；progress value 于两语料不可达，
  s178/s180 先例）。

## 边界

- value setter 面未动（既有泛型反射写内容属性已覆盖 corpus 断言——
  window.js setter+clamp 链经靶案 Pass 承载）。
- `position` 对「value attr 存在但非法」的返回（0/max）无语料钉子，
  按 spec indeterminate 门 = attr 缺席口径实现（attr 在 → determinate）。
- 分型表后备序不变：color 清洗 17 → form 编码替换 12 → form.elements
  集合层 12 → disabled click 8 + click 消费激活 7。

## 中间产物

/tmp/s181/：forms_s182.log、f178/f182.norm、join.norm、inter_s182.log、
inter178/182.norm、inter.join、button_solo.log、engine_lib.log、
shim_concat.js、fail_sig.tsv（s181 分型原始）。
