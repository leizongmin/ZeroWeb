# bolder/lighter 相对权重表：Rust 计算臂 + JS plain 通道 font-weight 臂

2026-11-14。承接 [font-* 数值槽 calc 接入批](2026-11-14-font-slot-calc-parent-basis.md)
§5 残余桶第 2 项（g31）。双臂批：Rust 继承完成期解析表 + shim gCS 通道
font-weight plain 回落（克隆件 host 视图 miss 面）。

## 1. 起点

- g29 后 css-fonts 挂账：font-weight-lighter-bolder 22F（css-fonts-4
  §relative-weights 新表页）+ font-weight-matching 14F +
  font-weight-matching-installed-fonts 6F（共 42F 桶）。
- css-fonts 全量基线（g30 后）**2,472P/2,526F/4T**（163 页）。
- WIP 起点（rate-limit 前轮遗留工作树件，本 session 法证采纳）：
  inheritance.rs resolve_relative_font_weight 已按 css-fonts-4 §5.2
  表重写（旧 css-fonts-3 400/600/800 边界整表淘汰）+ 新增单测钉面。

## 2. 归因链（双臂缺失，两条独立传导面）

**臂 1（Rust 计算臂，+8P）**：`resolve_relative_font_weight` 旧表按
css-fonts-3 边界（bolder <400→400/<600→700/≥600→900；lighter <600→100/
<800→400/≥800→700），css-fonts-4 §5.2 收窄为 <350→400/<550→700/
<900→900/≥900 保持（lighter <100 保持/<550→100/<750→400/≥750→700）。
边界 350/550/900 与 100/550/750 由 font-weight-lighter-bolder 页 22 腿
期望值实证钉面（base 99 lighter 保 99 / 901 bolder 保 901）。修复落在
computed 通道：font-weight-computed 48P/10F → **56P/2F**（+8，relative
关键字页腿回位）。

**臂 2（JS plain 通道，+22P）**：lighter-bolder 页 22 腿全败、gCS 返
**空串**（非错值）。五轮探针切面定谳传导链：

1. 静态 HTML inline `lighter/bolder` + gCS 直读（探针 t1）——**全对**：
   解析管线本身正确（解析表已由臂 1 兑现）。
2. 页面复刻（cloneNode(true) 模板 + 预挂载 style 写入 + appendChild）：
   22 腿全空。差异锁定 **cloneNode 时序**。
3. 身份法证：base/c0 均为纯 handle 元素（`__zwSelector=null`，
   `__zwHandle=__n0/__n1`）；裸调 `__zw_get_computed_style_by_handle`：
   base 出值（handle 在 `HANDLE_VIEW_CACHE.nodes` 表——CreateElement/
   SetStyle 变异重放时注册），**c0/c1 恒空**——innerHTML 批量物化的
   克隆子件不在 nodes 表、无 selector 可走正置表回落（s166/R51c miss
   路径），host 视图对该子树 face 盲。
4. 通道分解：`__zwPlainComputed(c0,'font-weight')` 裸调空——part01c
   plain 计算器有 font-size/opacity/anchor 族/position-area/color-scheme/
   color·font-family·background-color 各臂，**无 font-weight 臂**，
   尾部 `return ''`。
5. gCS 直取形（`cs.fontWeight`）回落白名单六项（color/font-size/
   font-family/background-color/opacity/position-area），font-weight
   不在列 → `_csV` 空串直返（getPropertyValue 形有通用 plain 回落，
   直取形是白名单式——两臂不对称即页腿死点）。

## 3. 修复（分文件）

- **inheritance.rs**（前轮 WIP 采纳 + fmt 收口）：css-fonts-4 §5.2 表
  重写（见臂 1）+ `test_relative_font_weight_table_css_fonts_4` 单测
  （bolder/lighter 各 10 边界腿，覆盖 99/100/349/350/549/550/749/750/
  899/900/901 期望值矩阵）。
- **part01c.js**：`__zwPlainComputed` 新增 font-weight 臂——继承走查
  （`_zwPlainDecl` 自 elt 上行首个可解析声明）+ `zwFwResolved` 递归
  （relative 关键字按父链已解析字重查 css-fonts-4 表，父链无源回落
  初始值 400；invalid/不可解析声明视同无声明继续上行——级联语义；
  css-wide initial/unset/revert 族 → 400，inherit 即上行同形）。序列化
  对齐宿主 `font_weight_str`（normal→400/bold→700/number 原值）。
  gCS 通道无源返空契约保持（P9-9/R5107 同形）。
- **part01.js**：gCS 直取形 `_csV === ''` plain 回落白名单补 `font-weight`
  （第 7 项，与 getPropertyValue 通用回落臂对齐）。

## 4. 验证

- 单测：zero-style-system **2,533 passed / 0 failed**（+1 权重表钉测；
  test-guard 包裹）。clippy workspace 0 告警；rustfmt clean。
- 探针（本地 zz-g31-probe，跑后删除不入库）：cloneNode 时序复刻 22 腿
  （11 base × lighter/bolder 全边界矩阵）**22/22 全绿**（臂 2 落地前
  0/22）。
- css-fonts 全量（02:08 二进制，systemd unit zw-g31-verify-0210）：
  见 §6 终账。
- 残余桶（g29 §5 第 2 项收口后的分账）：font-weight-matching 14F 已由
  g32 归因定谳为运行时粗体键塌缩（font_load_bridge R2417 按 ≥600 二值
  键选择，无 face 级 §5.4 匹配，faces 注册表架构面批已入册）；installed
  fonts 6F 宿主测试字体环境依赖定性不计缺口。二者均非权重表/通道域，
  挂账不闭；font-weight-computed 残 2F（150.25 u16 存储精度 + sign 容器
  实参）均为在册桶。

## 5. 传导链考古（handle 视图机制，防再推导）

`__zw_get_computed_style_by_handle`（callbacks.rs:2484 →
computed_style_cache.rs:855）：HANDLE_VIEW_CACHE 键 =（基座 html，
drain_gen，style_version，view_gen，applied，css/adopted/外链/shadow
条目）。miss 时 parse 基座 + 权威 applier 重放 `[applied..count)`，**仅
变异条目直接引用的 handle 注册进 nodes 表**；`nodes.get(handle)` miss →
正置表（sel_handle_map，handle→selector）回定位 → 仍 miss 返空串。
克隆子件经由 `__zw_set_inner_html_handle`（批量 SetInnerHtml 变异）物化，
其 JS 侧 handle（R151 `_handleChildren` registry 解析产物）从不入
nodes/正置表 → 该子树 gCS host face 永久 miss → 只有 shim plain 臂可救。
本质 = handle 身份与 host 视图节点身份的弱映射（drain 路径靠惰性 wrap
合并，视图路径无此合并）。

## 6. 终账

- **css-fonts 全量 2,472 → 2,502P（净 +30，页级零回归）**：
  Rust 计算臂 +8（font-weight-computed 48→56P）+ JS plain 通道臂 +22
  （lighter-bolder 页 22F→22P 全页清零）。
- 对账载体：zw-g31-final 七套件链（本批 attr 修复同链终验，
  [attr 修复批终账 §6](2026-11-14-attr-font-write-gate-deferred.md)）
  ——css-fonts 2,502、css-color 10,972 恒一、css-values 3,935、
  其余四套件页级零漂移。
- 单测：zero-style-system 2,533 / zero-css-parser 3,190 全绿（终数，
  含本批钉测）。
