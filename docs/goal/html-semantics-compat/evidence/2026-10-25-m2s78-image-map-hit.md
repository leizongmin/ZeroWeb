# M2 切片 78（s78）——F4 image-map 命中测试：coords/shape 解析核 + elementFromPoint 缺席臂命中面（R5278）

日期：2026-10-25。s67 八族地图 F4 收口，主对象
the-area-element 语料三文件（area-coords 32F + area-shape 17F +
area-processing 14F = 63F），全部为
「`document.elementFromPoint(x, y)` expected `<area>`/`<img>` but got
null」。

## 1. 根因

runner headless 无渲染管线——host `__zw_elementFromPoint`（R2924
HitTestCache，仅 renderer/browser 注册）与 `__zw_getBoundingClientRect`
在 runner 沙箱均缺席，elementFromPoint 缺席臂此前只有 SVG 形状回落
（svg-compat M3 S1），usemap img 的形状命中面为空 → 63F 全 null。
失败面修正：控制面原记 84F 系旧账——实测 68F 中 5F 是
hash-name-reference-dynamic.html 的 testdriver-Actions 面（runner 语料无
testdriver.js，`test_driver.Actions` ReferenceError，与本切片无关），
image-map 实际可收面 63F。

## 2. 修复面（R5278 三刀）

1. **解析核**（part03，R5277 核邻位）：
   - `__zwImgMapFloatList5278`：「rules for parsing a list of
     floating-point numbers」逐字——分隔集 = ASCII 空白（TAB LF FF CR SP；
     `\v` U+000B 非空白）+ 逗号 + 分号、连续分隔一体跳过（`2,,10,10` →
     [2,10,10] 不产 0 项）、垃圾字符跳读（`="2` → 2）、token 浮点前缀截断
     （`2in` → 2、`.4` → 0.4、`1e1` → 10）、error → 0（`--5`）。
   - `__zwImgMapShape5278`：shape 枚举属性 ASCII case-insensitive
     （`_zwASCIILower`——Unicode 折叠会把 `CİRCLE` 误判 circle）；missing/
     invalid 值均落 rectangle 缺省。
   - `__zwImgMapShapeAt5278`：各态最小计数（circle 3/poly 6/rect 4）不足
     → 空形；circle r ≤ 0 空、d² ≤ r² 含边界；rect min/max 交换归一 +
     `[x1,x2)×[y1,y2)`（corpus 无 rect 边界定谳行，FIXME 取 CSS 半开惯例）；
     poly 奇数丢尾 + on-edge 预检（**边界含入**——corpus 定谳面：star
     (101,101) 在闭合边 y=x、(199,101) 在 x+y=300、hitPoly (119,101) 在
     x+y=220 均期望命中；语料全整数坐标外积精确判 0，FIXME 非整数边界 FP
     噪声落 even-odd 仍确定性）+ even-odd 半开边射线（min(y)≤y<max(y) +
     x ≤ 交点）；default 整图恒真。
   - `__zwImgMapHashRef5278`：hash-name reference——无 `#`/`#` 末字符 →
     null，首个 `#` 后到尾。
2. **命中面**（part06 `_zwImgMapHitSel`，`_zwSvgHitSel` 邻位）：挂
   elementFromPoint 的 host-bridge **缺席臂**（SVG 命中后置臂——产品路径
   host 命中优先，零波及）。img 框 (0,0) 起点 + IDL width/height 近似
   （FIXME: containing block 真实几何归 layout 集成面）；usemap →
   hash-name → 树序首个 name/id 匹配 map → `map.querySelectorAll('area')`
   树序首个覆盖形状 → 无形状覆盖（含 usemap 不可解析/map 缺失）→ img
   自身。事件派发路径 `_zwHitTestSel` 不动（避免波及 interactive pin）。
3. **契约**（part90 两测试，新增 55+10 断言）：
   `image_map_parse_r5278`（解析核真值点 (a) float list 15 行 (b) shape
   枚举 13 行 (c) shape-at 22 行含 poly 边界三行/star 四行 (d) hash-ref
   5 行）；`image_map_element_from_point_r5278`（静态 DOM 命中 area/img/
   null + IDL 反射改 shape 随变 + map id 匹配 + usemap 无 `#` 回落 img +
   st() 错误捕获钉块内无异常）。断言翻转：解析核撤 (a)–(d) 全红；
   `_zwImgMapHitSel` 撤 (a)–(e) 全 null 红。

## 3. 排障留档：裸跑 runner 与 make 目标的环境差

初轮域/pin 直接 `cargo build --release` + 裸 `zero-wpt-runner`（test-guard
包裹但缺 Makefile 目标的 `ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`）——interactive
pin 假红 7 行（dialog-popover-closedby 6F + auto-expand-details-fragment
1F：滚动/点击时序敏感面），且与 s71 基线 diff 呈 P→F。排障：stash 全部
改动后**基线同样红** → 排除代码回归 → 对照 Makefile 发现 env 差 →
make 目标重跑后 **pin 439P/44F/0T 与 s71 逐位持平**（kv diff 空）。
教训入账：runner 数果实测必须走 Makefile 目标（test-guard 之外还有
RAF 帧驱动开关），裸跑二进制的域内数字（embedded-content）两法一致、
交互时序面不可比。

## 4. 验证

- 域（make 目标 + FILTER=embedded-content，RAF 帧驱动）run3：**971P/
  257F/30T** vs s71 run7 907P/320F/31T —— **64 F→P / 0 P→F**。翻转构成：
  area 三文件 63 行全翻 + `usemap-casing.html` 1 行加成（image-map 面解析
  后命中路径打通）；`audio-loading-lazy-autoplay-when-visible` T→P 为
  s69/s70 已记环境抖动族（audio 无 image-map 关联，顺风向）。
- run2（make 目标无 FILTER 全域跑，5400s 超时截断于 text-level-semantics，
  embedded-content 子树完整 1258 行与 s71 基线同规模）与裸跑 run1 的
  embedded-content 数字**逐位一致**（971/257/30 三方复现）。
- interactive pin（make 目标 FILTER=interactive-elements）run4：
  **439P/44F/0T 逐位持平**。
- zero-engine lib **3499P/0F**（含 part90 新增两测试）；clippy/fmt clean。
- image-maps 目录残 5F（hash-name-reference-dynamic，testdriver-Actions
  面）与 s71 持平，3P 面不变。

## 未解决项

- hash-name-reference-dynamic 5F：testdriver-Actions 面（runner 语料缺
  testdriver.js，指针/键盘合成输入族）→ testdriver 支持切片，非 image-map。
- img 框 (0,0) + IDL 尺寸近似：多 usemap img 重叠/非零原点页面不可表达
  → layout 集成面（与 720-stretch 同批）。
- rect 半开边界、circle 含边界、poly 边界含入为 corpus 定谳面选型，spec
  无逐字边界条款（「covering the point」未定义开闭）——上游若加边界行
  需复核。
- 控制面既列遗留不变：720-stretch 22F、sizes-auto→F5（用户门）、
  empty-picture 请求对象模型（issue 10531）、sizes 数学函数 FIXME、
  parser form element pointer 4F（用户决策捆绑 declarative shadow DOM
  批次）。
