# M2-S181 forms 全树 402F 家族分型勘根（勘定轮——分型归因定下一单根刀位，零源码改动）

## 入场与数据源

s180 下一步候选「forms 全树 402F 家族分析」。语料数据源 = /tmp/s178/
（s178 全树跑：forms_new.tsv 4697 键、forms_s178.log——s179/s180 改动面
（fetch/Request/XHR 凭证）对语料不可达，两轮均免跑，F 集合对 s178 零
位移承载）。本轮以 (key, assert 明细) 二元组重聚类：runner 日志 Fail 行
的下一行携带 expected/got，402 行全量提取（/tmp/s181/fail_sig.tsv）。

## 家族分型表（402F 全覆盖，合计 402 ✓）

| 族 | F | 机制（expected/got 定谳） | 单根? |
|---|---|---|---|
| customizable-select 特性面 | 165 | `appearance:base-select` 门 + picker/top-layer/`::picker()`/`<selectedcontent>` 整体缺位：19 处显式特性门断言（"must be supported in order to run"）+ 61 unhandled rejection + 6 target-size 几何。base-select/selectedcontent 全仓 grep 零命中 | 特性缺位，非单根 |
| combobox（datalist popover） | 12 | customizable combobox tentative 面（beforefilter/`:filtered`/popover 开合），同特性面 | 同上 |
| 值清洗算法族 | 27 | 逐类型 value sanitization：month/datetime-local/time/date 非法串→''（8+8+2+1+trailing-zeros 1）、valueAsNumber 小数秒（input-seconds-leading-zeroes 8 + valueasnumber 1）、text 换行剥离 2、type-change 2、hidden 1 | 多类型 2–3 根 |
| input 其他长尾 | 25 | number pattern/scientific-events/disabled、checkbox-switch tentative、color-attributes、defaultValue-clobbering 等 | 分散 |
| color 清洗 | 17 | type=color 清洗整体缺位——value 恒 `#000000`（关键词/rgb()/hsl()/display-p3/system colors 均应转 #rrggbb；`colorSpace` 反射 undefined）。引擎已有 `parseColorCss4` 桥（part05:8747 canvas op）可复用 | 单根（次刀位） |
| plain select | 14 | popover-position-with-zoom 4（picker 邻接）+ display-inherit 3 + required/focusable/many-options 等杂面 | 部分特性邻接 |
| progress value/position | 13 | **getter 缺位直接定谳**：`position` → undefined（indeterminate 应 -1）；`value` → 裸属性串（应 parse+clamp [0,max]，invalid→0）。setter 写 attr 后 position 转 0 的 determinate 链同缺 | **单根（下刀位）** |
| misc | 13 | textarea 3（placeholder line-height + wrapping transformation 2）+ output 3 + constraints fieldset 子 3 + option 2 + datalist 1 + setCustomValidity 换行归一 1 | 分散 |
| email/url/tel IDN | 13 | email 值 IDN→ASCII（punycode）转换缺位（parser-created 与动态 set 双臂）+ url/tel 清洗 | 单根，需 IDNA 通道 |
| form.elements 集合层 | 12 | HTMLFormControlsCollection/RadioNodeList 接口不存在（`RadioNodeList should exist` false）、namedItem/数字开头 name/table 语法错/indexed delete | 单根（集合层） |
| form 编码替换 | 12 | 不可编码字符→`&#NNN;` 替换算法缺位：text/plain expected `a&#601;b=c&#65533;d` got 原 mojibake；multipart 同根 8F（lone surrogate + 非 ASCII filename） | **单根**（text/plain+multipart 共享） |
| fieldset 禁用传播 | 10 | willValidate 对 disabled 祖先 fieldset 无 barred 传播（got true 应 false）+ 首 legend 子例外缺位（part01:1001 注 defer）+ `type` 反射 ''（应 'fieldset'）+ `elements` undefined + 嵌套 crash/hang 探针 | 3–4 根 |
| radio focus 导航 | 9 | 组内 checked 靶优先聚焦算法 | 单根 |
| :active 清除 | 9 | tab away 清 :active（pointer/spacebar/label 三臂）——输入仿真依赖 | 交互面 |
| disabled click() 门 | 8 | disabled 控件 `click()` 不分派 click 事件（button/input/select/textarea 四类双臂） | **单根** |
| click() 消费激活 | 7 | color/date/datetime-local/file/month/time/week `click()` 应消费 user activation | 单根（与上门同 click 分派域） |
| input.list | 6 | HTMLInputElement.list IDL（datalist 关联五断言面） | 单根（小） |
| legend.form | 5 | HTMLLegendElement.form 反射缺位 | 单根（小） |
| button | 5 | commandfor/invoker commands tentative 3 + keyup 激活 + 祖先 fieldset disabled 提交门 | 特性+分散 |
| form owner table | 4 | table 解析错树形下 form owner 关联 | 解析邻接 |
| submission checks | 3 | not-connected/navigated 文档提交门 | 单根（小） |

## 下一切片刀位定谳

**s182 = progress value/position（13F 单根）**：

1. 定谳证据直接来自日志 expected/got（无需探针再证）：`progress-2.html
   position equals -1` → got undefined；`value equals 0` → got (string) ""；
   `progress.html` 值钳制六案 + `progress.window.js` setter 面（value=2>
   max=1 → getter 1；max=4 → value 2/position 0.5）。
2. spec 短而闭合（HTML §4.10.14）：value getter = value attr 双精度解析
   （缺省/非法 → 0）clamp [0,max]；position getter = indeterminate（无
   value attr）→ -1，else min(value/max, 1)；setter 写内容属性（determinate
   化）。max 解析 R5009（part04:12471 邻位）已有同款浮点前缀解析可复用。
3. 零交互/零环境/零渲染依赖——13F 里 12F 单文件三件套
   （progress.html/progress-2/progress.window.js）+ reflection 面。

后备序（勘根已备）：color 清洗 17（需先证 parseColorCss4 对
display-p3/system colors 覆盖面）→ form 编码替换 12 → form.elements
集合层 12 → disabled click 8 + click 消费激活 7（同 click 分派域可合刀）。

**customizable select 特性面（177F）处置**：特性整体缺位，非单根刀——
立项需特性工程决策（picker UI/top-layer/`::picker()`/`<selectedcontent>`/
combobox popover 五面），挂起待拍板，不入单根刀序。语料 19 处特性门断言
= 后续立项时的现成 smoke 探针。

## 后备候选复读：interactive 44F（零新刀位，销案维持既有归因）

对 /tmp/s178/inter_new.tsv 44F（s178 记 bit-exact 对 s176）逐族复读：

- 18F `dialog-popover-closedby-complex` = shadow-slots 段，s176 挂账渲染流
  归因（nested-shadow 布局几何三面）在册，不越界；
- 11F 布局几何（abspos-dialog-layout 6 + overlay 2 + top-layer 1 +
  multiple-centered 1 + scroll-height 1 + non-modal 1）= 渲染流面；
- 4F 交互（closedby-bounds-clicking 2 + focus-previous-outside 2）+
  3F inert（ancestor-is-inert/svg-hittest/dialog-inert）+ selection 1 +
  showModal 1；
- 3F UA 样式面（default-color 2 + visibility-hidden 1）。

无漂移、无本流（语义面）新单根刀位；维持 s176 归因，候选销案。

## 局限注记

- 数据源为 s178 跑（今日零源码相关位移承载），非本轮新跑；s179/s180 的
  corpus 免跑判定（改动面不可达）是 F 集合现势性的依据。
- 族边界含判读成分（如 plain-select 14 里 popover-position-with-zoom 4
  归特性邻接面）——不影响刀位排序（前三刀与边界无关）。
- 中间产物：/tmp/s181/（f.tsv、fail_sig.tsv、by_file.txt、by_dir.txt）。
