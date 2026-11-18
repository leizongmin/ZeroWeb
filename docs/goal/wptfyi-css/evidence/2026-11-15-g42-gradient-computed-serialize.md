# g42 渐变 computed 通道 canonical 串化刀

2026-11-14 深夜开工，11-15 rollover 轮收口。driving: css-images
gradient-interpolation-method-computed 881F 主收割面（g41 evidence §5 G2 挂账），
顺带清偿 valid 残面 264F（方法子句族）。

## 1. 刀面

- css-parser `values/parse_transform.rs`：
  - `ColorInterpolationSpace` 模型身份保留——新增 `Hsl`/`Hwb`/`XyzD50`/`XyzD65`
    变体，`parse_interpolation_method` 识别入枚举（xyz→xyz-d65 别名在 color 侧
    同则；display-p3/a98-rgb/rec2020/prophoto-rgb 与未知空间仍归一 Srgb）。
    R2288 渲染回退移交 engine paint `map_interpolation`（hsl/hwb/xyz* → Srgb）。
  - `GradientSpecifiedParts` 文本承载（g41 已有）接线 radial/conic/linear computed：
    `canonicalize_radial_head`（`ellipse <lp>{2}` 双长度、`circle <length>` 单长度
    省形状关键字）；size-only 纯长度形首参派发（`radial-gradient(50px,…)` 颜色名
    /函数永不命中 parse_length 不误入）；`parse_position_pair` 4-token 缘偏移对
    （`at left 10px top 50em`）。
- css-parser `values/types.rs`：`ColorValue::ColorFunction(Box<ColorFunctionSpec>)`
  新变体——color() 预定义空间**未解析保留**（canonical 空间名 + 三分量 + alpha），
  parse 期不再坍缩 Rgba（css-color-4 §9.1 computed 值不坍缩 legacy rgb）；
  `xyz`→`xyz-d65` 别名 parse 期展开；percent 分量 /100 归一 0-1 域；
  白名单 = convert_predefined_to_srgb 全支持集（含线性光变体），白名单外
  fail-closed 拒收同既有；`to_srgb_rgba()` 供 paint 回退。
- css-parser `values/color.rs`：`parse_color_function` 改产
  `ColorFunctionSpec`（canonical 空间名 + 分量域归一）。
- engine `js_dom_bridge/computed_style.rs` computed 串化双路：
  - 带 specified 的渐变走 `gradient_specified_to_css` 骨架——canonical 配置头
    （linear 角词序 as-written 保真 `to right bottom` 不重排；radial 经
    `radial_computed_head`：size token 单位归一 `40em`→`640px`、`at` 位置模型化
    `at right center`→`at 100% 50%`、缺省 center 省略（R2751 口径显式
    `at center` computed 同省）；conic 经 `conic_computed_head`：from 模型角度
    归一 deg、缺省 0deg 省略）+ canonical `in <method>`（冗省判定
    `gradient_method_redundant_default`：站含 predefined color() → oklab、
    否则 srgb；显式 hue 法恒非缺省）+ computed 色标段（legacy 色 → rgb()、
    `color()` 站经 `color_function_to_css` 保真 `color(srgb 1 0 0)`，分量/alpha
    最短十进制）。
  - 程序化构造（specified None）回退模型枚举路径（part04 R2750 口径不变，
    11 断言全数维持）。
- engine paint 消费面：`paint/color.rs` resolve_color_current ColorFunction 臂
  + `paint/svg_filter_functions.rs` flood-color 臂 + `transition.rs` 串化臂
  （canonical 形）——渲染/滤镜回退 sRGB u8（R2288 无色彩管理口径），未知空间
  parse 期已拒、None 不可达防御性回落黑。

## 2. 单测与门禁

- css-parser lib **3,220P/0F**（+3 对 g41 基线 3,217：radial head canonical/
  size-only、4-token 位置对、interpolation 模型身份；color() 面旧 Rgba 坍缩
  等价断言改经分量域 + to_srgb_rgba 渲染等价断言）。
- engine lib **3,991P/0F**（rebase 后口径，含 g42 钉 6 + 对端 s440/insertParagraph
  新钉；上轮手术前 3,966 → 还原 part116 C4 钉后 3,974）；all-targets 47 target
  全绿 0F（含 js_dom_bridge_tests 桥测面）。
- fmt `--all --check` EXIT=0；定向 clippy css-parser+engine `--all-targets
  -D warnings` EXIT=0（systemd scope）。
- make test canonical 轮（上轮 make-test-g42c-235459.log）：fmt 前置过 +
  161 套件除一环境性失败全绿——`image_batch_e2e::…_reaches_concurrent_in_flight`
  系 disk-guard 清库删 zero-browser 产物（"build first" panic 793P/1F，
  非行为失败；`cargo build -p zero-browser -p zero-renderer` 后重跑 1P/0F
  闭账）。

## 3. 受控 A/B（B0 = results-g41-b2 g41 提交态；B1 = results-g42-b1 quiescent 重录）

| corpus | B0（g41-b2） | B1（g42-b1） | 净 |
|---|---|---|---|
| css-images | 1439P/1776F | 2584P/631F | **+1145P/−1145F** |
| css-values | 4041P/1546F/6T | 4041P/1546F/6T | **±0 零回归** |

- 逐页（ab-g42.py，案数恒同 css-images 3215 / css-values 5593）：
  gradient-interpolation-method-computed **F 881→0**、
  gradient-interpolation-method-valid **F 264→0**；其余页零漂移。
  **REGRESSED=0，grand net +1145。**
- B1 口径注：results-g42-b1 首录（00:10 前段）被僵尸 resume 轮的
  target-disk-guard 三级清库焚毁（release runner 被删，css-images exit=2、
  css-values 179/256 页截断 exit=0 假绿）作废；杀 8 僵尸 + 在跑链后 00:16
  quiescent 同脚本重录（run-g42-computed.sh：flock 互斥 + 同 clone
  cargo/rustc 活跃守卫 + test-guard 包裹），00:18 落盘为终账。

## 4. 会话事故记（429 rate-limit rollover 多重 spawn 爆发收敛）

- rate-limit 中断触发 N×resume 爆发（ps 计 13+ claude 同树并发）：gate 重试
  互撞（disk-guard 清库删在跑编译 ×3）、/tmp 日志并发覆盖截断。仲裁：保留
  唯一在跑 canonical gate，杀重复 make test ×2；本 rollover 轮 pull --rebase
  （远端 15 提交：对端 editing 修复 + wpt 周基线 docs，与本流 10 文件
  零交集）后干净复验单测/fmt/clippy/corpus 全链。
- 树收敛发现编辑流废弃 revert 残片（part116_c4_block_boundary_pins.rs +
  part01.js/part01f.js C4 代码删除，源出 21:42 autostash）与上游方向相悖
  （对端 fix-forward）——checkout HEAD 还原 6 路径，不卷入本流提交。
- s424d 判别探针（part115 +42 临时钉）两轮 ok 定谳（delta_y=1/101 →
  anchor-name-mutation 恒 +1px 残差归 shim offset 链 opBorder 项），提交前
  已撤除，工作树仅存本流 10 文件。

## 5. 残面挂账（g41 §5 更新后）

- ~~G2 computed 刀~~（本刀 +1145 清偿 computed 881F + valid 264F）。
- A 刀写面文法门：gradient-interpolation-method-invalid 292F（0P）——
  invalid 值现全放行（本刀探针实录 `in hsl longer`/`90deg in hsl shorter` 等
  hue 法缺参形仍被接受），validate 白名单接入 + parse 严格性盘点待刀。
- 解析缺口：color-stops-parsing 60F；image-set-parsing 114F +
  image-set-computed 36F（6b 指派 C+D 刀）。
- 独立探针 anomaly：runner 对 color-stops-parsing/image-set-parsing 单页
  直跑零输出（corpus 路径正常计数），待查。
