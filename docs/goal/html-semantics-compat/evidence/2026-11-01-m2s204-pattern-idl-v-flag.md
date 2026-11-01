# M2-S204：pattern IDL 反射 + v flag 编译判定——pattern_attribute/v_flag 2F 收口

- **日期**：2026-11-01
- **目标面**：`html/semantics/forms/the-input-element/pattern_attribute.html`
  1F（basic 子测——IDL `in` 可见性缺列）+ `pattern_attribute_v_flag.html`
  1F（breaking-changes-from-u-to-v 27 案——v 编译非法应忽略属性）（s181
  分型「input 其他长尾」pattern 面）
- **状态**：收口——两文件 7 子测全 Pass，forms 全树 keyed diff 净
  F→P=5（2 靶键 + onblur flake 域 3 键顺向回摆）P→F=0 键漂移 0

## 勘根

- **pattern_attribute.html basic**：`assert_idl_attribute(input, 'pattern')`
  = `!hasOwnProperty + in`（testharness.js:2077 `_assert_inherits` 实读）——
  `'pattern' in input` 恒 false。值面 `pattern` 已在
  `_REFLECTED_STRING_FLAT`（part01.js:1905）——get（part03 R80 臂）与
  set（part05 R5249 reflected 臂）通道已通，**唯 has 白名单缺列**
  （R5249/R5266/R5300/media M2a 同款根因，s202 form 同刀形）。同 test 的
  校验断言（validity.valid false/patternMismatch true/:invalid）在
  `[a-z]{3}` 基础案现势正确（其余 4 子测 unicode 面全 Pass）。
- **pattern_attribute_v_flag.html**：v_flag 面的编译判定链
  （part03 R57 两处——sel/handle 形 13000 + plain node 形 18798）为
  `new RegExp(x, 'v')` throw 时**回退 u 编译**——v 非法形（`[(]`/`[}]`/
  `[&&]` 等 u 合法 v 非法）回退后约束误生效 → invalid，而 spec §4.10.5.2.5
  编译非法 = pattern 属性整体忽略（valid 不受 pattern 影响）。既有
  `_isVInvalidPattern` 启发式（类内 `( [ { /` + 未配对组）只覆盖少数案；
  前 2 子测（set difference/`\q{}` v-only 语法）Pass 证明运行引擎
  （rusty_v8）支持 v flag——R57 时代注释「V8 无 v flag 支持」认定有误。
  探针勘根中自伤一课：首版探针 `[\q{ab}\--[cd]]` 语法错乱（`\-` 转义后
  `-[cd]` 悬挂）恒 false → 单验 21 案假 APPLY；修正为 `[\q{ab}]`
  （u THROWS / v OK）后 33 案全对。

## 施工

（shim 2 文件，净 +43/−5；rust/语料零改动）

- **part03**：`_zwVFlagSupported()` v flag 引擎可用性一次性探测（合法
  v-only 语法探针 `[\q{ab}]`，缓存）；两处编译回退分支改「v 可用引擎
  throw 即忽略属性（`_re` 置 null），仅 v 不可用引擎保留 u 回退」；
  R57 时代错误注释更正（v 支持实证 + 本启发式降级为快速路径说明）。
- **part05**：has 白名单 R5300 块后插 `pattern` tag 门列（INPUT/TEXTAREA，
  SCRIPT ':' 后缀容差同 R5266；FLAT 表全 tag 反射的接口成员归属门——
  `'pattern' in div` 应 false）。
- spec：#dom-input-pattern（input.pattern 接口成员）+
  §4.10.5.2.5（pattern 匹配与编译非法忽略）。gOPD 臂不加——语料无
  assert_readonly 面（assert_idl_attribute 只断 `in`，勘根实读）。

## 验证

- 拼接 shim（14 part）node --check SYNTAX_OK；编译判定单验 33 案全对
  （27 breaking-changes 案 IGNORE + v 合法案 APPLY + 未配对组 IGNORE）。
- 靶面：pattern_attribute.html 4/4 + pattern_attribute_v_flag.html 3/3
  全 Pass（stash 基线同两键 F 确认）；同 FILTER 命中的 canvas
  security.pattern.* 10 键双侧同败在册面零变化。
- forms 全树 stash A/B：**净 F→P=5 P→F=0 键漂移 0 timeout 位移 0**——
  2 靶键 + onblur flake 域 3 键顺向回摆（上轮 4 键下摆本轮全部回升，
  双向摆动再证在册 flake 域归因）。
- interactive 域 pattern 使用面核查：语料唯一命中 =
  dialog-popover-closedby-complex.html JS 模板串（非 pattern 属性，
  渲染流域在册挂账面），改动不可达。
- zero-engine 全量 3664P+1F（1F = ua_display_li 渲染流域在册键，基线
  同败非本刀）；fmt --check 过；guarded-clippy 过。

## 挂账

- pattern IDL gOPD 描述符未加（语料无 assert_readonly 面；后续若现
  assert_readonly(pattern) 面按 s202 form 同形补）。
- `_isVInvalidPattern` 启发式保留为快速路径（v throw 判定为权威判据后
  语义一致冗余；精准修改纪律不删）。
- onblur flake 域持续采样（本轮 +3 顺向样本）。
- number B 簇编辑路径语义（s203 挂账）与 misc 残面
  （setCustomValidity 换行归一/output mutations/form action base-url/
  form owner table）按序待刀。
