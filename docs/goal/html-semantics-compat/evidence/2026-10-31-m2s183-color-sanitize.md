# M2-S183 type=color 值清洗 CSS `<color>` 解析臂落地——14F 单根刀位（s182 后备序首刀），shim 1 + rust 2

## 入场定刀位

s182 账本后备序首位：color 清洗族（s181 分型 17F = color.html 14 +
color-attributes 2 + color.window.js 1）。清洗点唯一 = part01
`_zwSanitizeInputValue` case 'color'（part01:2312），旧臂仅
`/^#[0-9a-f]{6}$/` 校验——非法串面（#gggggg/foobar/8 位/trailing junk）
已绿，CSS 颜色面（关键词/rgb()/hsl()/display-p3/系统颜色/3 位 hex/
大写归一/首尾空白）整体缺位落 '#000000'。

## 勘根（覆盖面与调用面）

1. **调用面**：`_zwSanitizeInputValue` 三调用点同源——part01:2345
   （type 内容属性变更）、part04:14238（value IDL setter 净化后入脏
   缓存）、part03:17409（`_zwSelNormValue` R3906 写前校准基准读）。
   color.html 27 断言全走 IDL set（`input.value = w.value`）→ setter
   路径即全覆盖，**无需**挂解析期创建/内容属性变更臂（FIXME 注记）。
2. **解析覆盖面预证**（不写探针，读码定谳）：
   `zero_css_parser::values::parse_color`（css-parser values/color.rs）
   分派链覆盖 transparent/currentColor → hex（3/4 位展开；6/8 位
   `from_str_radix(16)` 大小写无关；7 位长 `#fffffff`/`#ffffff\0`/
   `#ffffff;` 落 `_ => None` 恰合三钉案）→ rgb()/hsl()/hwb()/color()
  （display-p3 经 convert_predefined_to_srgb）→ lab/lch/oklab/oklch →
   light-dark()/color-mix()/contrast-color() → 系统颜色；"inherit" 非
   颜色关键字拒绝 ✓。`crate::color_value_to_render` → render Color
  （u8 sRGB；CurrentColor→黑 fallback、Transparent→rgba(0,0,0,0)）
   恰承载 currentColor/transparent → '#000000' 钉案。
3. **复用而非新写**：canvas.rs `try_parse_canvas_color` =
   `parse_color(s.trim()) → color_value_to_render` 同源封装（私有），
   升 `pub(super)` 供 callbacks 复用——新回调零解析代码。

## 施工（shim 1 + rust 2）

- **callbacks.rs**：新注册 `__zw_parse_color_rgba`（`__zw_canvas_op`
  邻位）——入参 shim 已 strip 的原串，`super::canvas::
  try_parse_canvas_color` 成功返 `"r,g,b,a"`（u8 sRGB），失败/空返
  空串。纯函数无 DOM 状态。
- **canvas.rs**：`try_parse_canvas_color` 私有 → `pub(super)`（一行
  可见性，复用通道）。
- **part01.js**：case 'color' 重写——`String(v)` strip newlines（全串
  `[\n\r]`，spec strip newlines 语义）→ trim（ASCII 空白首尾，url/
  email 同款正则）→ `/^#[0-9a-f]{6}$/` 原样返回（快径）；否则回调
  解析 → 小写 `#rrggbb` 序列化（`('0'+c.toString(16)).slice(-2)`，
  丢 alpha——语料未钉 alpha<255 面）；回调缺席/解析失败/返串非四段
  → '#000000'。FIXME：解析期创建/内容属性变更臂未挂（语料仅 IDL
  set 面钉态）。spec：HTML §4.10.5.1.19 color state（value
  sanitization algorithm）。

## 验证

- 定向：`make testharness-html-semantics FILTER=the-input-element/color`
  —— **color.html 27/27 全 Pass**（14 靶案 F→P：3 位 hex 展开、大写
  归一、首尾空白三形态、crimson/bisque、rgb() 双形态、hsl(150deg
  100 53.5)→#12ff88、color(display-p3) 双案（.5 0 0→#8c0000、1 0 0
  →#ff0000）、System colors not_equals）；残留 3F 均非本刀面——
  color-attributes 2F（input.alpha/colorSpace IDL 反射独立缺位）+
  color.window.js 1F（语料资产 /css/support/color-testcommon.js 缺
  口，gitignore 语料，补拉属基准变更待授权，s108 先例）。
- **forms 全树 join**（8 job，/tmp/s183/forms_s183.log）：**4711/4711
  键 (key,state) 严格 join 对 s182 基线（/tmp/s181/f182.norm）——
  F2P = 14 恰靶案、P2F = 0、T 位移 0、其他 0**；全树 4325P/375F/12T
 （389F − 14 = 375F ✓；本轮零截断方差——12T 与 s182 同数，raw 日志
  仅 1 条精确重复 (key,Pass) 行，s139 同键多跑已知伪影，(key,state)
  对 sort -u 消化）。
- **interactive-elements 483 键 join 零位移**（439P/44F 两侧 bit-exact）。
- **engine lib 3635P/1F**（test-guard 包裹 540s；1F = ua_display_li
  渲染流在册，s180/s182 同键同态延续）。
- shim 装配序拼接 node --check SYNTAX_OK（114090 行）；cargo fmt
  （zero-engine）check 通过；`make guarded-clippy` workspace 全绿；
  webview/renderer lib 免跑（rust diff 仅 engine 内 2 文件，两语料
  不可达，s182 先例）。

## 边界

- alpha<255 序列化按丢 alpha（Chrome 观测行为同向），语料未钉——
  spec 对 CSS 色进 simple color 的 alpha 语义留白，不擅自加门。
- 解析期创建（`<input type=color value="crimson">` parser 路）与
  value 内容属性变更臂未挂清洗——语料无钉案，FIXME 在位。
- color-attributes input.alpha/colorSpace 2F = 独立 IDL 反射面
  （tentative 特性），不在本刀范围；已记下一步候选搭车位。
- 分型表后备序不变：form 编码替换 12 → form.elements 集合层 12 →
  disabled click 8 + click 消费激活 7。

## 中间产物

/tmp/s183/：color_targeted.log、forms_s183.log、f183.norm、
joined.tsv、inter_s183.log、inter183.norm、inter.join、engine_lib.log、
concat.js（shim 拼装）。
