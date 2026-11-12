# M2 s416 勘根收口账——current-pixel-density 10F 双面归因渲染流（勘根不施工移交）

- 日期: 2026-11-13；基线: HEAD edfdccbd1（s415 收口），树净，开工 pull --rebase 无新提交。
- 驱动: current-pixel-density 10F（chronic 在册，s416 施工候选——勘根结论变**移交收口**）。

## solo 复现（/tmp/s415/s416-cpd-solo.log，与在册一致）

- basic.html 14 子测 11P/3F，三案同签名 `assert_equals: clientWidth expected 0 but got 1`：
  1. `srcset="green-256x256.png 10000x"`（data-expect=0）
  2. `srcset="green-256x256.png 9e99999999999999999999999x"`（data-expect=0）
  3. `srcset="green-256x256.png 256w" sizes="0px"`（data-expect=0）
- error.html 7 子测全 F 首断言 `assert_not_equals: expected_width got disallowed value 0`；ref=`<img id=ref src="404" alt="testing">`，7 变体（`src=404`/`srcset=404`/`0.5x`/`2x`/`100w`/`100w sizes=500px`/`src="404 100w"`）全 404+alt="testing"。

## 断言结构（basic.html 全读）

- 每 img 六断言: width/height/clientWidth/clientHeight/naturalWidth/naturalHeight 全 == expected（**naturalWidth 断言值即密度折算值**——2x→128、10000x→0，spec HTML embedded-content §img processing-model）。
- 本库 11P 含全部非边界案（1x→256、1.6x→160、2x→128、512w/256px→128、256w/512px→512、256w/1px→1、SVG data-URI 三案→10）——width/height/naturalWidth 三对全过，**唯一败口 = clientWidth/clientHeight**。

## basic 3F 根因（engine 折算正确，layout 地板抬值）

- engine 侧密度折算链正确（s219 基座）：`img_selected_pixel_density`（extract.rs:770 起）三路——
  - `10000x` → Some(10000)；
  - `9e999…x` → x 描述符 HTML 浮点非有限 → error 候选弃（extract.rs:778 与 shim isFinite 同门，extract.rs:2879 单测在案）→ 无 src broken 形态 → Some(+Inf)（build_img_intrinsic_all 首候选回落保键 dim 0，防 NaN 面，mod.rs 注释在案）；
  - `256w sizes=0px` → dv = 256/0 = +Inf（extract.rs:809 合法计算态）→ 选 Inf。
  - mod.rs:783-804 除法 `*s = (s.0/ds, s.1/ds)` → dims 0.0256/0/0 正确落 map。
- **layout-engine tree.rs:1704-1709 both-abs 双维臂**：`let w = w.max(1.0); let h = h.max(1.0);`——三路折算 dims（0.0256/0/0）全被 1px 地板抬成 1×1 → clientWidth 1 ≠ 0。
- spec: 密度折算固有维 = natural dims / density，可为亚像素/0；Chrome 布局 0.0256px 盒 → clientWidth round → 0，测试期望 0 即此。
- 同族位点核查（移交方参考）: tree.rs `.max(1.0)` 共 10+ 处（1333/1334、1375/1376、1468、1478、1482、1494、1498、1524、1708/1709），驱动案命中 1704-1709 无属性 both-abs 臂（测试 img 无 width/height 属性、CSS 双 auto）。

## error 7F 根因（broken img alt 盒呈现缺失）

- spec: img broken 态（404）且 alt 非空 → 呈现 alt 文本盒（尺寸非零；测试口径宽松——仅断 ref 尺寸非零 + 各变体与 ref 同宽高 + naturalWidth/Height == 0，无 Chrome 精确形态断言）。
- 本库 broken img 无 fallback content 呈现（无占位/alt 文本盒）→ 盒 0×0 → ref.width = 0 → 首断言即败，7 案同因。

## 归属与移交

- run-rules 9: zero-web 流 = engine/dom/script-sandbox/net/webview；layout-engine = 渲染流工作面。两面根修均不在本流——**勘根不施工，移交渲染流挂账 5 面→6 面**。
- 移交方向注记（供渲染流）：
  - basic 3F: engine 已折算写入 image_sizes 的亚像素/0 值不应被 1px 地板——floor 收窄（按折算来源区分）或折算值专用通道；主位点 tree.rs:1704-1709，同族 `.max(1.0)` 位点需一并核查（防单点收窄后姊妹案翻面）。
  - error 7F: img broken+alt 盒 = fallback content sizing/paint 面。

## 口径

- 在册红 212F+16T 不变（移交不除账，承 s410 checkable 父 label 2F 渲染域移交先例）；NotRun 5 不变。
- 无施工零行为变化（solo 复现形态即在册形态）。
- 过程修正: s415 Active 下一步「constructed-stylesheets 1T 勘根候选」系**过期指针**——s413 已收口（tentative/ 11 文件 37 子测全 Pass），s415 收口时未同步摘除；本轮摘除并记此。

## 下一步

- form-submission-target 6T 勘根首位（engine/dom 面，挂账池既有项）；其余挂账承 s415。
