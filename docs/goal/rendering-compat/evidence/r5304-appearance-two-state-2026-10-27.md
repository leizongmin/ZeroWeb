# R5304 — css-ui-4 §appearance-switching 两态模型 + menulist-button 解析（webkit-appearance 家族 6/12→11/12，corpus 15738P/856F→15742P/850F 净 +4；002.tentative 一案论证接受翻转）

**日期**: 2026-10-27
**驱动案**: `css/css-ui/webkit-appearance-{button,checkbox,radio,searchfield,textarea,textfield}-001.html`
基线 ✗0.52/0.27/0.17/5.00/1.80/1.64 → **全部 ✓**；`webkit-appearance-menulist-button-001.html`
中段回归 ✗2.20 → **✓0.00**。
**corpus**: **16592/15742P/850F**；失败集 vs 基线（15738P/856F）归一化 diff 恰
{−6 家族翻绿， +webkit-appearance-menulist-button-002.tentative（论证接受翻转，见下），
 −incremental-scroll（R5303 已记账 flake，本次通过，方向无害）}，**零意外翻转**。

## 取证链

1. **定性**：ZW 绘制侧 appearance 是「声明值逐字匹配」非 css-ui-4 §appearance-switching
   两态模型。基线 dispatch skip set = {None, Auto, Meter, ProgressBar, Listbox, Menulist}
   ——`auto` 不绘原生外观（素盒）；legacy 值仅在 `appearance_element_matches` 判「适配
   元素」时绘制。后果：家族 test/ref 两页部件渲染不对称——ref（作者 `appearance: auto`
   覆盖 UA 值）素盒、test（legacy 值或 UA 值）原生，6 案翻红。
2. **规范依据**（css-ui-4 §appearance-switching）：`none` 关原生外观；**auto 与全部
   legacy widget 值互为兼容别名，used value = 元素自身原生外观**。WPT 断言组
   webkit-appearance-radio/checkbox/button-001 直接断言「任意非 none 值 ≡ auto ref」
   像素一致（fuzzy totalPixels 0-2）。R4249 四值（meter/progress-bar/listbox/menulist）
   保留 skip 不冲突。
3. **裸 input 的 computed appearance 是 `auto`**（getComputedStyle 探针页实证：
   `range-input=auto | search-text-input=auto | text-input=auto`；types.rs 注释
   「None（默认值）」已过时）。基线全量裸 input 靠 dispatch 的 Auto-skip 免疫原生绘制。
4. **首版两态实现的 corpus 扰动（+21 案）与修正**：首版 resolver 把
   input[text/search/range]→Textfield/Searchfield/SliderHorizontal、textarea→Textarea，
   且 effects.rs 保留 Textfield/Textarea 臂（白底+#787878 边框，叠 UA 样式成双边框）与
   `_` catchall「灰色背景指示器」rgba(230,230,230,200)（合成白底恰为 (235,235,235)，
   ~4040px/元素）。Auto 移出 skip 后全部裸 input 进入 paint_appearance 落入这些臂 →
   grouped-fallback ×10、trailing-space-and-text-alignment ×10、textarea-pre-wrap-013、
   blocks-026 成片扰动。**取证链**：corpus diff → 单案复跑（负载 2.7 排除 flake）→
   stash 单因子 dump（基线两页近素 0.27% pass）→ PNG 直方图锁定 (235,235,235) →
   kill-switch `ZW_APPEARANCE_NATIVE_USED=0` 二分（灰消失 → 两态侧）→ 探针页逐元素
   二分（textarea/range/search 单元素复现）→ getComputedStyle 探针定谳 computed=auto →
   源码定位 catchall 发射点。
5. **002.tentative 翻转的联合不可满足证明**：对 drop-down select 的渲染，001（match，
   initial-select ≡ auto-select）+ compute-kind fallback 家族（match，menulist-button-
   select ≡ initial-select）+ 002（mismatch，menulist-button-select ≠ auto-select）
   传递矛盾，任何渲染器无法同时满足。基线 002 pass 机制（PNG A/B + stash 单因子）：
   ref 页作者 auto → skip 素盒，test 页其余 15 部件保留 UA `appearance: checkbox/…` →
   原生——两页差异全部来自**其余 15 部件**（diff bbox (19,21)-(230,116) 覆盖
   button/input-text/textarea/checkbox/radio 区域），被测命题「menulist-button ≠ auto
   on select」**并未被基线 pass 验证**。当前树两页 PNG 字节级相同（md5 一致）。该
   不对称正是 6 案家族翻红的根因；WPT 源码自注 tentative（spec 明言 "there is no
   requirement that this rendering is any different"）；ZW 无 OS 原生 menulist chrome，
   忠实实现无差异可用。**选择接受 002 一案翻转**。

## 实现（最终设计）

- **`effective_appearance()`**（painter/mod.rs，pub(super)）：绘制侧 used value。None →
  None；其余 → `native_appearance_for_element()`。kill-switch
  `ZW_APPEARANCE_NATIVE_USED=0` 回退声明值逐字语义。仅绘制侧消费（getComputedStyle
  序列化不经此路径）。
- **`native_appearance_for_element()`**：仅映射 ZW 有真实原生 chrome 的元素——
  `input[type=checkbox|radio]` → Checkbox/Radio（填充臂）、`button` 标签与
  `input[button|submit|reset]` → Button（border.rs 原生 2px 边框臂 + text.rs +2 基线）；
  **其余一律 Auto**（paint_appearance 早退素盒 ≡ R5304 前 UA 样式渲染）。
- **dispatch skip set 移除 Auto**：Auto 进入 paint_appearance 按元素解析（家族 ref 页
  作者 auto → 原生 ≡ test legacy）。
- **effects.rs 臂收敛**：Textfield/Textarea 臂（白底+#787878 边框）与 `_` catchall 灰色
  指示器删除——css-ui-4 兼容别名下 ZW 无该 widget chrome 的值按普通盒渲染（素）；
  Checkbox/Radio 臂保留（corpus 全量验证的 UA 值域）；Button 臂保持 `{}`（chrome 由
  border/text 通道承担）。
- **`menulist-button` 入 parse 表**（css-parser `AppearanceValue::MenulistButton` +
  style-system types/apply_advanced 映射 + js_dom_bridge 序列化 + apply_coverage 值表 +
  tests_9 解析判例）：001 修复关键（此前声明无效被丢，computed 停在级联前值 none →
  测试页 15 部件素盒 vs ref 原生 ✗2.20）。两态结构下自动获得「非 drop-down select 上
  ≡ auto」语义，engine 侧零特判。
- **paint_borders 参数化**（border.rs 加 `doc: Option<&Document>`，11 处调用点同步），
  Button 族原生边框臂与 text.rs 按钮文本 +2 基线改用 `effective_appearance`；
  `appearance_element_matches` 冗余 gate 删除。

## 门禁

- fmt 0 diff；clippy 4 crate（engine/style-system/css-parser/layout-engine）
  `--all-targets -D warnings` 0 告警；engine lib **3565P/0F**（+2 回归守卫：
  裸 input auto 素盒 + input[type=checkbox] auto 解析原生；5 个旧语义单测按两态模型
  更新——checkbox/radio 判例补 type 属性、textfield 判例改断言素盒）、style-system
  **2450P/0F**、css-parser **3003P/0F**（+menulist-button 解析断言）、layout-engine
  **2185P/0F**。
- 家族 FILTER=webkit-appearance **11/12**（基线 6/12）；FILTER=grouped-kind-of-widget-
  fallback **10/10** ✓0.24%；FILTER=menulist 45/46（唯一 ✗ 即 002.tentative）。
- 全量 corpus **16592/15742P/850F**，失败集 diff 零意外（见头部）。
- product-smoke PASS（welcome oracle **15.50% 持平** + struct ×6）。
- bench-gate 定向 zero-engine **GATE PASS**（19 指标 NEW=7 全在预算内）。
- make test 全量未跑（改动面 = engine paint appearance + style-system/css-parser
  appearance 解析 + 同 crate 测试；消费面由 corpus 全量 + product-smoke + 4 crate lib
  覆盖，R5301/R5302/R5303 同判例记录在案）。

## 挂账

- 002.tentative：若后续实现 OS 原生 select chrome（menulist 专属绘制）或
  appearance-disabling properties（css-ui-4 §appearance-disabling-properties）语义，
  可用「auto 可忽略 border/背景、menulist-button 不可」重开。
- types.rs `AppearanceComputedValue::None` 文档注释「默认值」过时（实证裸 input
  computed=auto，初值来源未溯）——低危记录，后续轮顺带修正。
- R5299 三项、013 ellipsis host 传播、vertical-rl 域、flake 账本（size-adjust.tentative、
  incremental-scroll 先单案复跑再定性）维持。
