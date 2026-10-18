# R5171 证据：单行 inline box-level 装饰内容区锚定（inline-non-replaced-height-002/003 翻绿，corpus 净 −2）

日期：2026-10-18。基线：R5170 tip（5f6ae5752）。kill-switch：`ZW_INLINE_CA_PAINT=0` 回退行盒域绘制（Painter 构造期单读，R3858 教训；判别经外部 env corpus OFF 轮验证）。

## 域核

inline-non-replaced-height-002/003（各 1.04%）：`font:100px Ahem; line-height:150px` 的 span，002 `padding:50px 0; background:blue`、003 `border-bottom:50px solid blue`。CSS2 §10.6.2/§10.8.1：inline 非替换盒 content area = 元素主字体 A+D（Ahem 100px → 100px），**与 line-height 无关**，半 leading（(150−100)/2 = 25px×2）属行盒。真实装饰矩形 = 内容区锚定 border-box = 200px；旧实现按树几何（行盒 150 + padding/border 100）= 250px，多涂整条半 leading 带（003 border-bottom 落点低 25px）。

布局树保持行盒域（流 bookkeeping，r3773 判例：锚定 y 入树破坏收缩回收/兄弟位移）；但 `inline_reported_rect`（R4379/R4383 记录臂，默认 on，只记录不上树）已持有内容区锚定 border-box `(top − pad_t − bt, content + pads + borders)`——paint 侧从未消费。R5120 flag-on 全量 A/B 已否决 per-fragment 臂整旗翻默认（1,112 净 +4），002/003 不受益（不过 1.5×fs 门，实际路径 = box-level 站）。

## 实现（painter/mod.rs，+约 60 行）

1. **`inline_ca_paint_override` helper**：门 = kill-switch + `display == Inline` + 非定位（`is_absolute`/`is_fixed`）+ `inline_reported_rect` 存在 + **收缩轴**（`reported_h ≤ tree_h − 0.25`）+ 报告值差 >0.25px。命中则 clone 树盒仅覆写 y/height，返回调整后 abs_y。
2. **只收缩不扩张**（关键设计，见下「027 归因」）：扩张轴（显式 line-height < A+D）不动——与 highlight overlay 行盒缘语义（css-pseudo-4 highlight-bounds，R5052/R5054 选区站按行盒口径）及 R639/R1442 fragment 站相交。
3. **消费面**：`paint_node_in_rect` 与 `paint_node_inner` 双路径对称——bg 色两处（含 R5086 z 序 splice 内）、bg-image（`_fixed` 两态）、`paint_borders`（fieldset 分支不动）。outline/box-shadow/spanner 保持树几何（无案驱动，最小面）。
4. **零扰动论证**：normal line-height 页引擎常数行高 ≡ A+D（系统 0.928+0.236=1.164、Ahem 1.0）→ hl=0 → 报告值 ≡ 树值 → ≤0.25px 门内不 clone；provider 入树臂（webfont）树值本就是内容区，同门内零差。bleed 不双计：树上 padding/border 域已是声明值（inline_finalization），R4297 缺失量口径 `inline_box_vertical_bleed` 输入不变，缺失量恒 0。

## 027 归因记录（设计收敛依据）

首轮（无收缩门）corpus 翻红 `active-selection-027`（R5086 driving case，基线「近似通过」>strict）。插桩链（ZW_R5171_DEBUG 临时 eprintln，已移除）+ Build A/B 二分实证：

- 该页 `.highlight_reftest { line-height:1 }`（48px 行高 < 48px 字体 A+D 55.9）= **扩张轴**页面；`div::selection` 黄底被 ZW 以「选中盒自身 background」承载，box-level 站发射。
- override 把 test 页黄底从树几何 [116,164] 扩到 [116,171.9]；而 **ref 页同几何 span 的黄底走 fragment 站（绕过 paint_background，插桩零命中）仍 48**——两侧分化 1.16% 翻红。字形色紫/绿为两侧共同预存差（::first-line 未实现，基线即「近似通过」），与本改动零归因。
- 结论：扩张轴须与 fragment/selection 站同切片校准（FIXME 挂账），本切片收缩门后 027 回绿。

## 验证

### 回归测试（crates/engine/src/paint/tests/r5171_inline_ca_paint.rs，3 案）

- `r5171_single_line_inline_bg_uses_content_area_border_box`：002 形态——蓝 bg fill 顶 = 树 y+25、高 200（旧行为 fill 在树 y、高 250，测试红判别经 stash 验证：旧行为 2 红/normal 对照绿）。
- `r5171_single_line_inline_border_bottom_anchors_content_area`：003 形态——50px 底条带顶 = 树 y+125（内容区底），旧行为树 y+150。
- `r5171_normal_line_height_page_unchanged`：normal 行高对照——bg 仍在树盒位（h=200 = A+D+padding），锁定零扰动。

### corpus ON/OFF A/B（16,590 案，同 binary）

| 轮 | fail | 备注 |
|---|---|---|
| ON（收缩轴，默认） | **1,106** | **终态**（基线 R5170-ON 1,108，净 −2） |
| OFF（`ZW_INLINE_CA_PAINT=0`） | 1,108 | 恰等基线，fail 清单逐案零差异；002/003 回红 |

- ON ↔ 基线 fail 清单 diff（前缀归一化）：**翻绿恰 2**（inline-non-replaced-height-002/003）、**翻红 0**。

### 门禁

- fmt / clippy（-p zero-engine -p zero-render-foundation --all-targets -D warnings）：清。
- zero-engine lib：**3,113 全绿**（r5171 3 案新入库）。
- `make reftest`：**712/712**（100%）。
- `make test`：workspace 腿全绿；`zero-browser --tests` 腿预存环境红（R5168 挂账 5 同款）xvfb-run 前缀复跑绿（详见 rendering-compat.md 条目）。
- product-smoke：welcome oracle diff **17.07%**（≤20，R5166-R5170 同值）+ 全 struct PASS。
- bench-gate：**24 FAIL（113 检查，NEW=0）——负载签名判读，不 relax**。①FAIL 集横跨零变更 crate（layout 4.7×、webview_render 4.3×、security 2.6×、style-system/script-sandbox 1.3-3.3×），起跑 load 3.16（phase3c 基线重录浪潮退潮中）+ 兄弟流（html-semantics-compat clone）活跃；②唯一修改面指标 `paint_complex_page_500_elements` 666.85µs vs 预算 656.88µs（超 1.5%）经 **kill-switch 同窗 A/B 甄别：ON 528.13/531.14µs vs OFF 546.28/571.69µs（各 2 轮交错）——开关零因果甚至 ON 略快，读数随负载回落 21%**；③同日 cronjob clone 净窗守卫 capture（phase3c attempt 2，load>4.5 中夭折守卫零触发）已 `record --relax` 落盘新基线（a908338cf，paint_complex 净窗 p95 475.5k），净窗 verify 仅余 4 FAIL（style-system/wasm-sandbox 零变更 crate）——进一步佐证本机读数带宽由负载主导。按 R5170 判例噪声族延续判读，不改测量配置；空闲净窗复跑挂账。

## 挂账与下一步

1. **扩张轴**（line-height < A+D 的内容区锚定）：须与 R639/R1442 fragment 站、R5052/R5054 选区站（css-pseudo-4 highlight-bounds 行盒缘口径）同切片校准；active-selection-027 族为回归试金。
2. R5170 挂账延续：vertical 域表 rtl 列序、跨 white-space 折叠全局对齐、纯文本 run 匿名 cell 合成、boxless cell 网格定位边缘、vertical 表匿名 cell 域（本轮勘察无案驱动）。
3. `zero-browser --tests` 腿缺 `ZW_GUARD_RUNNER_PREFIX`（R5168 挂账 5 延续，测试基建另案）。
