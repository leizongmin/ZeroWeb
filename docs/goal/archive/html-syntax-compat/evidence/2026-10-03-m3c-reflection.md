# M3 片 c——reflection IDL 反射系统面（2026-10-03）

## 结果

**全通道 50380/61303 = 82.18%**（片 b 75.55%，+3833 passes），对 M1 基线 per-case
逐案比对 **0 回归**。reflection 域从 ~0% 级联跃升：reflection-sections
**5604/5604 全绿**、reflection-text 9897/10202（97%）、reflection-forms
7029/8271（85%）、reflection-misc 4231/4877（87%）、reflection-grouping
4883/5358（91%）、reflection-tabular 4030/6116（66%）、reflection-embedded
6260/8922（70%）、aria-enumerated 521/1722（30%）。DC-4 实测：make test 68
suites 全绿（含按 spec 更新的 R3185 单测）+ clippy `-D warnings` 干净 + fmt +
reftest **704/704**。

## 系统性根因（一改千测）

| 根因 | 修法 | 影响面 |
|---|------|--------|
| R3042 expando 分支拦截一切非原始值赋值（含 null/undefined/object）——反射 DOMString 属性的 `IDL set to null` 属性未写 | set trap 加 `_reflectedStringAttr(p) === null` 豁免——反射属性走 WebIDL DOMString 转义（null→"null"、对象走 ToPrimitive toString/valueOf） | reflection-* 'IDL set to null/object' 全簇 |
| R122 `_zwAttrInstances` 实例登记不知道 IDL boolean 移除——hasAttribute 实例短路恒 true | 新增 `_zwAttrInstanceRemoveKey`（qname/local 双匹配 + Attr 绑定摘除），hidden/checked/disabled/selected、_REFLECTED_BOOL 表、autofocus/inert 三处 falsy 移除分支接同步 | boolean IDL set→hasAttribute 往返 |
| autofocus/inert falsy 分支只移除 sel 路径——createElement detached 元素（reflection-* 全域装配形态）属性残留 | 补 handle 路径 `__zw_remove_attr_handle` | detached boolean 全簇 |
| title/lang/accessKey 误按 [LegacyNullToEmptyString]（null→''）——实为普通 DOMString（spec：仅 id 与 body 颜色族是 LegacyNull） | title/lang/accessKey setter `String(value)`（null→"null"）；R3185 单测按 spec 更新 | title/lang/accessKey null 面 |
| tabIndex 反射 "-0" 返 -0（assert Object.is 区分 ±0）——WebIDL long 的 Int32 转换归一 +0 | getter/setter 命中后 `| 0`（**NaN 保持**——首版误全局 `| 0` 使 NaN→0 破坏 native-focusability 默认链，tabindex_defaults_follow_native_focusability 单测实证后修） | tabIndex "-0" 面 |
| body 颜色族（text/link/vLink/aLink/bgColor）与 document 颜色别名缺 [LegacyNullToEmptyString]；颜色族/align/version 等 legacy 反射串表外 | 表补 `_REFLECTED_STRING_NULL_EMPTY`（null→''）+ `_REFLECTED_STRING_FLAT/MAP` 扩 align/version/background/text/link/scroll/颜色族/margin 族/trueSpeed + part06 document 颜色/dir 别名（fgColor↔body text 等 + dir 枚举归一） | body/document legacy 反射面 |

## 诊断记账

- **探针三级跳**：probe-refl（基础语义）→ probe-refl2（复现 harness 序列
  setAttribute→IDL set→hasAttribute）→ probe-refl3（直读宿主回调区分移除/读取
  两侧）——定位「移除已入队但 hasAttribute 实例短路」与「我的错误插入分支遮蔽
  sel 移除路径」双根因。
- **set trap 结构**：`_makeProxy` handler 字面量跨 part03:15185→part05:849 拼接
  （get trap 巨函数体横跨三文件）；set trap 唯一（part04:8024），其链尾延伸至
  part05。修复时插入的重复 `!handle` 分支遮蔽 sel 移除（R2997/R2998/R2999/R3048
  七个单测红了七个）——removeAttribute 语义分支改动必须同时核对 sel/handle 双路径。

## 残差（M3 片 d 输入）

- **per-interface IDL 清单**：reflection-embedded（base/link.href URL 解析反射、
  img 维度族）、reflection-tabular（table 章节 attrs）、reflection-forms 尾簇
  （17%）、aria-enumerated（70%，ARIA 反射枚举语义）、ol.start/reversed、
  li.value 等 long/unsigned 反射数值面。
- aria-attribute-reflection.html（0/41）与 aria-element-reflection 面（4/27）。

## 门禁

- make test（TIME_LIMIT=1800）：68 suites 全绿（0 failed）。
- make guarded-clippy（-D warnings）：干净（含按 spec 更新的
  part15.rs R3185 断言）；cargo fmt --check 干净。
- make reftest：**704/704**（Layout 502 + Text 202）零不一致。
- 全通道 50380/61303 = 82.18%；M1 基线逐案比对 0 回归。
