# DC-2 color 残余批：fill/stroke gCS 读出面 + color 属性 currentColor 宿主后置解析

日期：2026-11-11　批次：DC-2 color 通道批后续（lightdark-scheme-system 证据 §5 挂账两项收口）　基线：results-v10（color 63 页 10,808P / values 256 页 2,213P）

## 1. 勘根（v10 失败三案归因）

| 案 | 页面 | 失败形态 | 根因分层 |
|---|---|---|---|
| fill/stroke ×2 | system-color-compute.html | `getPropertyValue("fill"/"stroke")` 双元素同返 `""`——assert_not_equals 双空串相等失败（`assert_not_equals: got disallowed value ""`） | **三层缺位叠加**：①apply.rs 两臂 `parse_color` 无 scheme（dark 传播缺，同 compute 页 box/text-shadow 4F 根因）；②`inherit_property` 无 fill/stroke 臂（`fill: inherit`/`stroke: inherit` 声明 no-op，inherit 子元素落初始）；③computed_style.rs 无序列化臂（gCS 读数恒 ''）。次断言 computedStyleMap `/menu/i` 空串 vacuous pass 不计 |
| currentColor ×1 | light-dark-currentcolor-in-color.html | `color: light-dark(red, currentColor)`（#child color-scheme: dark → dark 支选中 currentColor）期望 parent green `rgb(0,128,0)` 得黑 | 宿主 cascade `style.color = CurrentColor` 后无后置解析——gCS `resolve_color_current` 自引用（element_color = &style.color 即 CurrentColor 本身）回落黑（paint/color.rs 旧行为）。spec：`color` 属性内 currentColor（含 light-dark 选中分支）computed 值 = inherit（css-color-3 §currentColor / css-color-5 §light-dark）。shim plain 臂已有同语义（`__zwComputedColor` propCtx 'color' 父链；parsing/color-computed-named-color.html currentColor→parent 色 Pass 为证）——宿主面缺失，双通道分野又一例 |

## 2. 修复（四刀，与 apply_advanced dark 四臂同形态）

1. `style-system/property/apply.rs` fill/stroke 两臂：`parse_color` → `parse_color_with_scheme(v, style.color_scheme_dark)`（css-color-4 §resolving-color-values：system color computed 值时间解析为绝对色）。
2. `style-system/property/inherit.rs` `inherit_property` 加 fill/stroke 臂（SVG2 §fill-stroke Inherited: yes；R4280/R5187 注的「继承由后续切片补」落地）。
3. `style-system/inheritance.rs` 继承完成后置解析：`style.color == CurrentColor && parent_style.is_some()` → 取 `parent.color`（`resolve_relative_font_weight`/text_align MatchParent 同型后置位；根无父保持 CurrentColor 黑回落旧行为；parent.color 已递归归一）。
4. `engine/js_dom_bridge/computed_style.rs` 加 fill/stroke 序列化臂：Some → `color_to_css(resolve_color_current(c, element_color))`，None → SVG2 初始值（fill 黑 / stroke none）。FIXME：`none` 声明存储为 Transparent（apply 臂），与显式 transparent 同串——区分需存储层新变体，语料未钉暂记。

通道分野：support 页无 fill/stroke 断言（grep 空），本批纯宿主通道，shim 表面零改动。

## 3. 测量（v11 记录轮）

results-v11/（json 通道，v10 同口径；日志 zwv11-build/zwv11-css-values/zwv11-css-color；unit zwv11-run-b systemd service，test-guard per-proc-mem 4G total 8G limit 3600s）：

- **css-color：63 页 10,840P / 170F（对 v10 10,808P 净 +32，逐页零回归）**。
- **css-values：256 页 2,213P / 3,380F 对 v10 页级全等零漂移**。
- 本批面 **+3 精确命中链预测**：system-color-compute 25P/2F → **27P/0F（页面清零）**、light-dark-currentcolor-in-color 0P/1F → **1P/0F**。
- 同树异面增量（provenance 注记：v11 build 取当时工作树全态，三面在树未提交 hunk 并存——本批 fill/stroke+currentColor＝主 session、light-dark image＝zeroweb-2-e2、grid G2a＝zeroweb-2-fe）：e2 面 **+29**（light-dark-image 1P/31F → 30P/2F，残余 2F 归 e2 账）；fe 的 grid 面不入 css-values/css-color 两 corpus（零影响）。逐页归属按面分列互不混算。

## 4. 门禁

clippy --workspace --all-targets EXIT=0（target-clippy 隔离目录，test-guard per-proc-mem 4G total 8G limit 2400s；唯一 warning 在 fe 的 parse.rs 半成品，归其面）；四改动文件 rustfmt --check 干净（工作区级 fmt 唯一 diff 在 fe grid.rs 半成品行）。

## 5. 挂账

- light-dark-image 残余 2F（e2 面在其证据补记）。
- consistency 页 dark FieldText 5F（UA 表单样式建模）维持挂账。
- Typed-OM 相对色 light-dark origin 折叠 2F 维持挂账。
- fill/stroke `none`→Transparent 存储与显式 transparent 的 gCS 串区分（存储层变体，语料未钉）。
