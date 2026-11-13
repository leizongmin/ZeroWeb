# font-weight 相对权重表 css-fonts-4 边界：font-weight-computed +8P（g29 残余桶 2 部分）

2026-11-14。承接 [font-* 数值槽 calc 接入批](2026-11-14-font-slot-calc-parent-basis.md)
§5 残余桶 2（bolder/lighter 相对权重表），css-fonts face 同日批（g31）。

## 1. 起点（g30 后基线）

`css-fonts/variations/font-weight-lighter-bolder.html` 0P/22F、
`font-weight-matching.html` 0P/14F、`font-weight-matching-installed-fonts.html`
0P/6F。

## 2. 归因

- `resolve_relative_font_weight`（inheritance.rs）用旧 css-fonts-3 边界
  （bolder <400/<600/else、lighter <600/<800/else）；css-fonts-4 §5.2
  #relative-weights 表边界为 **bolder 350/550/900、lighter 100/550/750**，
  且 **≥900 bolder 与 <100 lighter 保持原值**（页 22 腿：base 901 bolder
  →"901"、base 99 lighter→"99" 实证）。
- lighter-bolder 页 22 腿不回位的**独立根因**（表刀之外）：页子元素经
  `cloneNode` 模板克隆插入，gCS fontWeight **空串**（got ""，22 腿全空）。
  探针二分钉面（zzz-bolder-probe 四变异）：cloneNode 克隆节点（V2/V4）
  gCS 空、直建节点（V3，含未知 `font-family: 'CSSTest Weights Full'`）
  正常出 "400"/"900"——**克隆节点样式合成缺口**（shim/桥面，引擎样式树
  无克隆节点条目），与权重表无关。
- font-weight-computed 页静态 HTML 嵌套腿走引擎初始解析链，表刀直接生效
  ——该页 48→56（+8）。

## 3. 修复

`resolve_relative_font_weight` 表更替（css-fonts-4 §5.2 链接注释 + 边界
实证注释）：Bolder `<350→400 / 350-549→700 / 550-899→900 / ≥900→保持
parent_weight`；Lighter `<100→保持 / 100-549→100 / 550-749→400 /
≥750→700`。单测 `test_relative_font_weight_table_css_fonts_4`（22 腿边界
全覆盖）。

## 4. 验证

- 单测：zero-style-system lib **2,533 passed / 0 failed**（+1 表测试）。
- css-fonts 全量对账（results-g31-bolder-lighter/css-fonts-full）：
  **2,472 → 2,480P 净 +8 零回归**（font-weight-computed 48→56）。
- lighter-bolder 页 0P/22F 不变（克隆通道缺口，见 §5 转桶）。

## 5. 残余桶（转挂，下批候选）

1. **cloneNode 克隆节点 gCS 样式合成缺口**（lighter-bolder 22F + 潜在
   跨页克隆面）：克隆节点引擎样式树无条目 → 继承属性 gCS 空串。修位在
   shim cloneNode/桥样式合成（DOM 面，非 css face）。探针四变异形态见
   本文档 §2。
2. **@font-face 描述符匹配算法**（font-weight-matching 14F，css-fonts-4
   §5.4 face matching：desired ≤500 先降序 ≤desired 再升序 >desired 族）。
3. **installed-fonts 6F 环境依赖**：宿主无 CSSTest Weights 测试字体族，
   代码不可修（定性，不计缺口）。
