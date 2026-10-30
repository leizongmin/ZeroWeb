# R5339 证据 — backdrop-filter 滤区 plain 圆角回退（css-filters-2 #BackdropFilter）

日期：2026-10-30 · 轮次：R5339 · 基线：R5338 交付态 796F @ `e3fe18574` · 代码 commit：见轮次记录

## 0. 结论速览

- backdrop-filter 滤区多边形单语义源 `backdrop_filter_clip_polygon`：shaped 形角优先（R5183 既有），corner-shape 初始态（全 round）回退 **plain 圆角多边形**（`rounded_corner_polygon_in_box`，任意半径 > 0），无半径保持矩形滤区（既有行为）。
- 驱动案 backdrop-filter-clip-rect-2 **✗0.40% → ✓0.01%**（fuzzy 阈内）：`.bf` 行三盒（border-style none/solid/dotted + `border-radius: 10px 20px 30px 40px`）backdrop 反色区此前方角，现随圆角轮廓（像素探针：左缘曲线 15→11→9→8 与背景行一致）。
- corpus A/B：796F → **795F**，FIXED={backdrop-filter-clip-rect-2}，REGRESSED=∅，通过率 15795/16590 = 95.2%。
- **backdrop-root 族定性为结构域挂账**（本轮不改）：backdrop-filter-backdrop-root-{opacity,filter,mask,mix-blend-mode,clip-path-*} 族 = Backdrop Root 隔离组采样语义缺失，CPU 平面 framebuffer 需透明底组层机制。

## 1. 择轻与定性

796F 账本分带扫描（避开 va/OOF 锚/writing-mode 域）：0.00% 带 35 页 = 既有特性缺失/flake 带（历轮已定性，不重开）；text-transform-fullwidth-004/005（0.12/0.17%）= vertical-rl 写向测试（排除域）；letter-spacing-ligatures-005（0.36%）= Arabic shaping 近似域（shaped kerning 挂账同族）；outline-028（0.31%）= auto outline 外观域。取 **backdrop-filter-clip-rect-2（0.40%）**——单语义缺口、机制链清晰。

定形：product-smoke dump（`/tmp/r5339-bf1-test.png`）+ 像素探针——`.no-bf` 行（背景黑）圆角 ✓（左缘 15→11→9→8 曲线），`.bf` 行（backdrop-filter: invert）方角 ✗（恒 8）。R5183 已给 FilterPrimitive 挂 shaped clip_polygon，但 `shaped_corner_polygon_in_box` 对 `corner_shape.is_all_round()`（**所有未显式设 corner-shape 元素的初始态**）返回 None → apply_backdrop_filter 滤区退矩形。R5183 注释「全 round 无形角返回 None（矩形滤区，既有行为）」将该 None 语义误推及 backdrop 消费端——plain border-radius 元素滤区本应随圆角轮廓（filter-effects-2 #BackdropFilter：backdrop 裁剪于 border box 含圆角）。

## 2. 消费端审计（共享路径边界）

`shaped_corner_polygon{,_in_box}` 五消费端逐一核对 None 语义：

- border.rs:250（形角边框带）/ mod.rs:5457（形角背景填充）/ mod.rs:5566（`prepare_corner_shape_clip` 显式早退）：None → 走 plain round 既有路径，语义正确，**不动**。
- collapse_phase.rs:240（overflow: clip 形角裁剪）：同型盲区（plain radius + overflow:clip 裁剪方角）但属 overflow 裁剪域、corpus 无当前红案指向，**另案挂账**不顺手扩。
- effects.rs `apply_backdrop_filter`：**本轮修**——None 应回退 plain 圆角（任意半径 > 0）。

单消费端修法：新 helper `backdrop_filter_clip_polygon`（helpers.rs，shaped 优先 → plain 圆角回退 → 无半径 None），apply_backdrop_filter 改单点调用。

## 3. 实现与回归

- `paint/helpers.rs`：`backdrop_filter_clip_polygon(style, x0, y0, w, h) -> Option<Vec<(f32,f32)>>`（半径经 `shaped_radius_spec` 规范解析，含 %）。
- `paint/painter/effects.rs`：`apply_backdrop_filter` 滤区多边形改走该 helper（R5183 shaped 行为不变）。
- pin +2（`r5183_shaped_percent_radius.rs` 同族）：`test_r5339_backdrop_plain_radius_round_clip_fallback`（10/20/30/40px 半径 + 默认 corner_shape + backdrop_filter → 圆角多边形在场：TR 弧段 (80,0)→(100,20)、BR 起点 (100,70)；直角点不在场）+ `test_r5339_backdrop_no_radius_keeps_rect_region`（无半径 None 矩形滤区守卫——corpus 矩形路径零扰动哨）。
- 双向可判：修复前同 pin 驱动腿失败（`shaped_corner_polygon_in_box` 返 None）；驱动案 corpus 前后 ✗0.40 → ✓0.01。

## 4. backdrop-root 族结构域定性（挂账依据）

backdrop-filter-backdrop-root-opacity 定谳（`/tmp/r5339-bf3-test2.png`）：testcase 1（`opacity: 0.8` 祖先）+ testcase 2（`will-change: opacity`）的 backdrop 元素均反色出红（采样穿透 backdrop root 到容器青色）；ref 真值 = **青色**（root 内无内容 → backdrop 透明 → invert 不改 alpha → 透出容器青）。filter-effects-2 #BackdropRoot：opacity<1 / filter / will-change(相关属性) 等祖先形成隔离组，backdrop 采样限于组内。CPU 平面 framebuffer 实现需「进入 root 开透明底离屏层 → 子树绘入层 → backdrop 元素采样层 → 退出合成（带组 opacity/filter）」——结构域，挂账待专项轮次。家族余红（root-{filter,mask,mix-blend-mode,clip-path-*}、plus-mask/plus-opacity、reference-filter、svg、basic-opacity 等）同属此域或 SVG/mask 深片。

## 5. 门禁

- fmt 0 / clippy zero-engine --all-targets 0。
- corpus A/B：796F → **795F**，FIXED={backdrop-filter-clip-rect-2}，REGRESSED=∅，95.2%（15795/16590）；backdrop 家族全量核查：既有 ✓ 页（basic/blur-large/border-radius-change/clip-radius-zoom/clip-rect/box-shadow/clip-rect-zoom 等）零位移，既有 ✗ 页（backdrop-root 族、clip-rounded-clip 10.27 等）率值零位移。
- make test（guarded-test）：zero-engine lib **3603P/9F**（既有 fetch 红族零位移 + 本轮 pin 2 入账）；clippy quickjs 腿净、0 编译错误；其余 binary 全绿。
- product-smoke（paint 路径变更适用）：welcome **14.59%** ≤ 20% + struct PASS ×6（welcome 800/375/320 + article 双宽）+ form-input-perf PASS + legacy **42 fixtures 0**。
- bench-gate 定向 `ZERO_WEB_BENCH_CRATES=zero-engine`：**GATE PASS 19 指标（NEW=7 定向面新增，全部在预算内）**（morning paint 62.29→54.43 基线 / first_paint_wall 174.59 预算 241.51 内）。

## 6. 挂账 / 下一步

- **backdrop-root 隔离组采样**（结构域）：需透明底组层 + 退出合成机制；家族 root-* / plus-* / reference-filter / svg / mask 族 10+ 页。
- overflow:clip + plain border-radius 形角裁剪同型盲区（collapse_phase.rs:240，另案）。
- corner-shape-backdrop-filter-{overflow,video-overflow} ✓0.01 内维持；backdrop-filter-clip-rounded-clip ✗10.27（clip-path 交叉域）。
- 795F 账本续扫择轻（避开 va/OOF 锚/writing-mode/backdrop-root 域）。
