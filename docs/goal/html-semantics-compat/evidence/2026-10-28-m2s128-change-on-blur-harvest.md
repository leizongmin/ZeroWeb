# M2 s128 · the-textarea-element 域残余同域收割（change-on-blur 共享面 + textLength/type 单点 + maxlength/minLength NaN 折叠——定向 9F→0F 搭车 input 域 4F，真翻红 0）

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s127（TEXTAREA value/defaultValue 读面 live 化）

## 入场定刀位

s127 后全量基线（/tmp/s127-full.log）翻拣 the-textarea-element 域残余
8F 逐点勘根：type/textLength = IDL getter 缺失单点；maxlength/minlength
= IDL setter 非数值 NaN 折叠缺失；change-to-empty = blur→change
提交面（勘根发现 the-input-element 域 change-set-value /
change-to-empty-value 同根——共享面）；wrapping ×2 + placeholder-
lineheight = 渲染流面让位。

## 勘根定谳

1. **change-on-blur 共享面**（3 断言，input 2 + textarea 1）：语料定
   模型——change 触发 = 用户编辑置脏（`__zw_text_input`/
   `__zw_text_delete`/计划管道 `mark_user_edited` 置 `_userEdited`；
   script `value=` 不算）**且** blur 时现值 ≠ focus 基线（净值比较：
   change-set-value 的 restore-then-set 负案「净值回 a 不触发」、
   script-set 负案「脚本改值不触发」双双依赖）。基建已有：`_userEdited`
   （part01，validity tooShort 消费中）+ `_zwSelNormValue`（R3906，
   TEXTAREA 双形态 live 读 s127 前已备）。
2. **textLength**：textarea.value= "Hello, World!" → `.textLength`
   undefined（getter 缺失）；语义 = API 值 UTF-16 长度（同 value getter
   读路径）。
3. **type**：textarea `.type` 返 ''（generic 反射落空）；spec 恒
   'textarea'。
4. **maxLength/minLength NaN 折叠**：IDL set "not-a-number" →
   `Number()` = NaN → `String(NaN)`="NaN" 写属性 → getter 解析失败返
   -1；WebIDL long NaN → 0 → attr "0" → getter 0。getter 三负案（缺省/
   负/非数值 → -1）已对，仅 setter 缺折叠。input 域 maxlength/minlength
   同断言搭车（共享反射层）。
5. **渲染流面让位**：wrapping ×2（hard wrap 依布局软换行位置——
   cols=10 提交值换行需渲染宽度）+ placeholder-lineheight（bounding
   rect 高 19.5 = 分数 line-height vs shim 19）。同 naturalWidth/
   current-pixel-density 惯例让位挂账。

## 施工（engine shim part01/03/04/05.js，无 Rust 变更）

1. part01：`_s128ChgBase` 声明 + reset 链；`__zw_text_change_baseline`
   （focus 记 `_zwSelNormValue` 基线）+ `__zw_text_change_due`（脏旗 +
   净值 ≠ 基线 → 重置基线/清旗/返 true）两钩子。
2. part04 focus 臂：获焦相位落新后记基线；焦点迁移旧元素 blur→focusout
   派发后同语义提交（blur() 之外的失焦路线 spec 同源）。
3. part04 blur 臂：blur → focusout 派发后 `__zw_text_change_due` →
   `_dispatchWithBubble('change', bubbles)`（序 blur → focusout →
   change，Chromium/Firefox 互操作序）。detached（MEl）臂不接（语料
   静默）。
4. part03：TEXTAREA `textLength` getter 臂（同 value getter 双路径：
   脏 → dirty 缓存归一形；非脏 → `__zw_get_text`/`_zwTaLiveText` 归一
   形）。
5. part04 type getter：TEXTAREA → 'textarea'（OUTPUT 臂后、枚举反射
   前）。
6. part05 R5009 setter：maxLength/minLength limited-long 对 NaN → 0
   折叠（负值 throw IndexSizeError 既有路径不动）。

## 验证

- shim 拼接（js_dom_bridge.rs 装配序）node --check 过。
- 定向 `FILTER=forms/the-textarea-element`：**38P/3F**（8F→3F；
  type/textLength/maxlength/minlength/change-to-empty 5F 全收口，残余
  3F = 让位的渲染流面）；`FILTER=forms/the-input-element`：**1496P/
  145F/3T**（join 翻动恰 4 行 = change ×2 + maxlength/minlength 搭车，
  无真翻红）。
- 全量 A/B × /tmp/s127-full.log（原始行口径 6749P/1060F/103T →
  **6758P/1051F/103T**）：键级 join 恰 **9 Fail→Pass**（4 input +
  5 textarea，全 `1F→1P`），**真翻红 0、Fail→Timeout 0、gone/new 0**。
  本轮无 guard 内存杀（s127 的并行摆动未复现）。
- engine lib **3570/0**（+11 单元 bin 全过）；wpt-runner **256/0**
  （76 ignored）；clippy（engine+wpt-runner --all-targets -D warnings）
  零告警；fmt 净。
- 语料零改动、无探针页（勘根经语料读码 + shim 调用链闭合）；runner
  侧无代码改动——常驻语料 change-set-value/change-to-empty-value/
  textarea-type/textarea-textLength/textarea-maxlength/textarea-
  minlength 9 断言即回归面（与修复同提交）。渲染面零依赖——reftest/
  product-smoke 未跑。

## 挂账（本切片更新）

- **textarea 域残余收割收口移出候选**（change-on-blur/textLength/
  type/maxlength/minLength 五面）。
- **渲染流面让位（承袭累积）**：wrapping hard-wrap ×2（布局软换行
  位置依赖）+ placeholder-lineheight 1F（分数 line-height 渲染）+
  naturalWidth SVG viewBox 22F + current-pixel-density 11F——均待
  css-parser/style-system/layout-engine/render-foundation 流。
- **change-on-blur 未接路线**（语料静默，未施工）：宿主 `__zw_host_focus`
  （Tab 键驱动失焦）、checkable/checkbox 即时 change（click 语义另有
  面——checkbox.html 'fires click → input → change' 3F 在账）。
- 承袭不变：form-submission-0 186F、execution-timing 构成面、
  lazy-loading 26F、meta color-scheme 21F、写流执行身份盖章断链
  FIXME、part02 setAttribute 绕 notify 臂、041 nested write、radio
  组排他其余置真现象、deep clone 非 HTML ns 面、css-module corpus 门
  （待决）、www2 可达性、module-import-referrer TLA、dynamic import()
  with 透传、classic 外链 SRI FIXME、143 write、defer 多写完成序、
  接口对象基建面、FIXME 组、/tmp/zw-hb*.txt、FileList 缺索引访问、
  live 树 document 级逗号列表 qSA 根缺口；**待决（须用户拍板）**：
  css/text 数据模块语料资源缺失、green-2x2.png 语料资源分发、parser
  form element pointer 4F 缓行（2026-10-23）。

## 下一步候选（s129）

挂账序复核（form-submission-0 186F POST 传输契约勘根大基建评估 /
execution-timing 构成面单根）或全量基线翻拣下一单根面（域翻拣刀法
同款，整页整簇 Fail 勘根优先）。
