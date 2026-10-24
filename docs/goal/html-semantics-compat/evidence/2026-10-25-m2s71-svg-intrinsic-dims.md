# M2 切片 71（s71）——F1b SVG 固有尺寸族：data:SVG 尺寸核 + settle 覆写面 + 密度逐轴折算 + 未渲染 IDL 臂（R5277）

日期：2026-10-25。s67 八族地图 F1b 续片，主体是
naturalWidth-naturalHeight-width-height（s70 域内 77P/181F）与
svg-img-density-vs-zoom（1P/8F）两个 SVG data URL 语料文件。

## 1. 根因

1. **runner 位图解码面不解 SVG**：host `__zw_get_image_size` 对
   `data:image/svg+xml` 返回 viewBox 原始宽高（无则 100×100），不是
   CSS-images natural dimensions 组合表语义——viewBox-only 图期望
   300×100（AR 套默认 concrete object size），host 给 600×200；百分比/
   负值 attr（无效缺席 → 落 AR/默认尺寸）同样偏离。静态 sweep 面的
   resourceState 全部由该 host 快照写入。
2. **resourceState 无 SVG 固有轴元数据**：语料 `data-density` 语义是
   「attr 派生轴做密度校正、默认尺寸/AR 拟合轴不校正」，无 axes 字段时
   naturalWidth 密度校正只能全开或全关，逐轴期望不可表达。
3. **未渲染 IDL 面（whatwg#11287）**：display:none img 的 `.width/.height`
   应回原始未校正 intrinsic；s70 恒密度校正 → 2x srcset 未渲染行全 F
   （s70 偶有蒙对：host viewBox 原始值恰被校正回期望值的行）。

## 2. 修复面（R5277 五刀）

1. **`__zwSvgPayload5277`**（part03）：`data:image/svg+xml` payload 提取
   ——percent 编码、base64（atob 失败 → null）、raw 回落。
2. **`__zwSvgNaturalDims5277`**（part03）：根 `<svg>` attr 提取（单/双/
   无引号；HTML tokenizer 无引号值不在 `/` 截断——`height=40/>` 值为
   `40/` 非法即缺席）；length 解析（非负 number|px，`0` 合法，负/百分比/
   不可解析缺席）；viewBox 4 数且 w/h>0 → AR；组合表：w+h→(w,h,both)、
   仅 w→AR?(w,w/ar,width):(w,150,width)、仅 h 对称、全无→
   AR?(300,300/ar,none):(300,150,none)。spec：CSS-images §natural
   dimensions + HTML density-corrected intrinsic width/height。
3. **sweep SVG 核前置 + 覆写面**（part04 `__zwFireStaticImgSettle`）：
   payload 解析 → 核值写 resourceState + avail（携 axes）。幂等三分：
   携 axes（本核已写）跳过；既有 host 态（无 axes，含运行期臂与 broken
   态）**原位覆写**不重派 load；非 SVG payload 维持 host 幂等语义。
4. **密度逐轴折算**（part04 `_zwImgNaturalDim`）：state.axes —— attr
   派生轴按选中密度校正、默认/AR 轴恒原始；无 axes 旧态默认 both
   （raster 语义不变，s70 八行 F→P 即由该默认 + 未渲染臂合力）。
5. **未渲染 IDL 臂**（part04 width/height getter）：内联
   `style.display === 'none'` 且 loaded → 回原始未校正 intrinsic
   （FIXME：样式表/继承 none 不在近似面——动态创建元无布局几何，
   gBCR 零框 R4600 实证，故用内联样式直读）；detached 密度语义维持 s70。
6. **avail 传播 axes**（part03 avail 命中臂）：resourceState 与
   available-images 同携 axes，运行期新建 img 命中后逐轴折算生效。

## 3. 运行期 fast path 撤除（留档）

初版曾在运行期 fetch 臂加同款 SVG 前置：`inflight+1` 后 `_defer(load)`
—— 模板克隆风暴（43 模板 ×3 密度 = 129 img）超 `_deferBudget=256`，
回调被静默丢弃 → inflight 泄漏 → `__zwImgLoadBlocked()` 恒真 → runner
loadStep5275 以 100ms 重试至文件 Timeout（域 run5 全域 1001/1258 行截断、
naturalWidth 单文件 258→1 行实证）。改同步 dispatch（无簿记）后仍抖
（77P/181F，差于撤除后 84P/174F 且复现不稳）。终版**撤除运行期臂**，
settle 面确定性 3× 复现；运行期 SVG 语义归 host 解码器切片收口（解码器
语义统一后 shim 前置自然不需要）。

## 4. 桥接契约（part89 两测试）

`svg_intrinsic_dims_r5277`（19 断言）：payload 三形态解码 + 非 SVG
证伪 null；组合表 b1–b7；length 边界（负/百分比/不可解析缺席、px 后缀、
`"0"` 合法）；退化 viewBox（0 宽、3 数）；无引号 attr（含 `/>` 截断语义
——断言用无斜杠形，注释记 tokenizer 依据）。

`svg_img_settle_density_r5277`（8 臂）：(a) sweep 前置压过宿主证伪桩
999；(a2) avail 携 axes；(b) srcset 2x 密度校正；(c1–c3) 仅 w/仅 viewBox/
仅 h 逐轴折算；(d1–d6) display:none 后 .width/.height 回原始、
naturalWidth 仍校正（SVG + raster 双钉）；(e) raster 同语义；
(f) 运行期 avail 传播；(g) 预置 axes 的 avail 项传播；(h) 覆写面——
无 axes host 形态（999 证伪）被核原位覆写为组合表值 + axes、load 不重派。
断言翻转：SVG 前置撤 (a) 翻 999；axes 折算撤 (b)(c) 翻原始；未渲染臂撤
(d)(e) 翻折算值；avail 传播撤 (f)(g) 翻 miss/默认轴；覆写撤 (h) 翻回 999
且双派 load。

## 验证

- 域（embedded-content 231 文件子集）run7：**907P/320F/31T** vs s70
  747P/479F/32T —— **160 F→P / 0 P→F**。唯一 T 变化
  video-loading-lazy-autoplay-when-visible T→F 为 s70 账本已记的环境
  抖动行（s69 F → s70 T → s71 F 翻回，video lazy 族无 SVG 关联）。
- naturalWidth 文件：77P/181F → **236P/22F**（单文件 3× 稳定同值）；
  svg-img-density-vs-zoom：1P/8F → 2P/7F。
- 全域逐子测试 kv diff：`/tmp/s70-run4-kv.txt` vs
  `/tmp/s71-run7-full-kv.txt`（160 行 Pass 翻转、0 行 Pass 消失）。
- interactive-elements 域 pin **439P/44F/0T 逐位持平**。
- zero-engine lib **3496P/0F**；clippy/fmt clean（test-guard 包裹）。

## 未解决项

- naturalWidth 文件残余 22F 全部是 720-stretch 渲染展开行（`data-width=
  720`：img 铺满 containing block，`.width` 需布局几何）→ layout 集成面
  收口，控制面既列遗留。
- 运行期（动态创建/src 变更后 settle 前）SVG img 读值仍走 host 语义
  （avail 传播命中除外）→ host 解码器 SVG 语义统一后自然收口。
- display:none 近似面：样式表/继承 none 不可见（仅内联样式直读）。
- 控制面既列遗留不变：sizes-auto→F5、empty-picture 请求对象模型
  （issue 10531）、sizes 数学函数 FIXME、parser form element pointer
  4F（用户决策捆绑 declarative shadow DOM 批次）。
