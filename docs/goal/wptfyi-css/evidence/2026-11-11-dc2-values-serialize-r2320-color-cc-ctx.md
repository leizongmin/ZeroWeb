# DC-2 values serialize 批（r2320 + NaN 规约）与 color currentcolor 上下文批

日期：2026-11-11 · 分支 main · 范围 css-values 数学序列化 + css-color computed 上下文

## 批次内容（两个功能提交 + 一个 fmt 提交）

1. **r2320 values serialize 批**：specified 值读面常量折叠（cos/sin/tan/acos/asin/
   atan/atan2/min/max/clamp/mod/rem/round/hypot 常量参数折叠，calc 树化与
   canonical 序列化）——`css-parser` serialize_math.rs（+339）、parse_transform.rs、
   transform_helpers.rs 三写臂接线（native CSD 写时折叠 css_style_declaration.rs /
   shim sel 臂 callbacks.rs / shim handle 臂 js_dom_bridge.rs）+ computed 值臂
   NaN 规约（computed_style.rs）。测试：css-parser r2280/r2320 两文件 + engine
   js_dom_bridge_tests/part03.rs（f1–f11 + g1/g2 + NaN censor 组）。
2. **color currentcolor 上下文批**：shim part11-computed-color.js `_zwCCPropCtx`
   模块态 + `__zwComputedColor(raw, el, prop)` 三参（part01c.js 传递）+ 非 color
   属性 currentcolor 先试本元素（css-color-4 §5.3）。
3. **fmt 批**：`cargo fmt --all` 副产物（script-sandbox/v8_runtime.rs、
   style-system/property/apply.rs 数组 packed→逐行），无行为变化。

## NaN 规约（computed 值臂）

CSS Values 4 §10.12：顶层计算结果不外泄——NaN **censored into a zero value**（±∞
clamp 到上下文允许范围，individual transform 数值域无界不 clamp）；§10.13 序列化
NaN → `calc(NaN)`。WPT sin-cos-tan-serialize NaN 组断言
`computed(scale, NaN) === computed(scale, "0")` 且 ≠ defaultValue，驱动
computed_style.rs `individual_transform_to_css` 各臂 `n()` 闭包
（`is_nan → 0.0`）规约后序列化；Scale 两值相等折叠单值形在规约后比较。

## srcBare 分量语法跟随：证伪回退

color 批曾给 hsl/hwb computed 生产者加 `srcBare` 三件套（跟随输入语法裸/百分比
形）——v6 首跑回归 8 案（hsl/hwb computed none 混 % 槽丢 %）。三轮探针页定位：
declared 面在写侧已剥 %（color-valid-hsl/hwb corpus 锚
`'hsl(120 80% none)' → 'hsl(120 80 none)'`，parsing-testcommon = 声明面），
computed 面读同一 store——store 已丢输入语法，srcBare 只能跟随 mangled 输入
复刻剥 %，无条件补 % 恰复 computed 期望。srcBare 零增益（v6 对账确认），回退
producer 数组/serial 分支/_zwCCCompBare 三件套，保留 currentcolor ctx 修复。

## v6 记录轮对账（cmp-v6.py，results-v6/ vs results-v5 css-values + results-cc8 css-color）

- **values：净 +665，逐页不降 0 回归**，18 升页。top：sin-cos-tan-serialize
  **40→270（全绿）**、acos-invalid 0→62、acos-serialize 0→60、
  round-mod-rem-computed +54、minmax-angle-invalid 0→48、
  minmax-percentage-invalid 0→42、sin-cos-tan-invalid 0→42、
  minmax-angle-serialize 0→38、minmax-angle-computed 1→27、
  minmax-length-percent-invalid 0→16、calc-infinity-nan-serialize-angle 0→14、
  clamp-integer-invalid 0→12。两轮 values 总数逐字一致（2213P/3373F）证确定性。
- **color：测量二进制含 raw 影子放宽（见下节），子测试级归因**：+26 currentcolor
  （computed-relative-color +21、alpha-color-computed +5，全部 currentColor
  子测试、无 none 槽，归因本批 ctx 修复）；−13 color-computed-color-mix-function
  xyz none 组（pre-alias 测量态，1cafca122 别名折叠恢复）；探针页
  zz-probe-hsl-none +1。hsl/hwb computed 3752/3753、55/56 与 cc8 持平——
  srcBare 回退成功恢复首跑 -8。终局记录见
  [color 批收口](2026-11-11-dc2-color-raw-shadow-mix-alias.md) v8 终账
  （color 63 页 10,782P/228F，对 cc8 净 +26 逐页 0 回归）。

## raw 影子放宽与 mix 空间别名折叠（合并提交 1cafca122）

part03.js raw 影子保存条件放宽（`!/none/i.test(core)` 项去除：归一变形且原值
含 none 即存原值）与本批 srcBare 回退是同一现象的两层修法，最终**合并提交**
1cafca122（currentcolor 三参上下文 + raw 影子放宽 + color-mix 插值空间别名折叠
`'in xyz' ≡ 'in xyz-d65'`）。法证记录：

- v6 记录轮二进制（04:29 rebuild3 态）已含 raw 影子放宽、不含 mix 别名折叠——
  color −13 mix xyz none 组（cc8 943→930，丢失子测试全部含 `color(xyz … none …)`）
  是 **pre-alias 测量态**：raw 影子让 computed 读回含 none 原文后，mix 侧空间
  别名未折叠走 rgb 回退丢 none 旗标；别名折叠（1cafca122）恢复该组。
- values 零影响：v6 两轮 values 总数逐字一致（2213P/3373F），首轮构建不含
  part03.js 改动。
- currentcolor +26 与 none 槽无关（全部新增子测试无 none），不受 raw 影子
  分支影响。
- cc8 hsl/hwb none 组的通过系「旧 serial 强加 % 与影子漏存两错误抵消」的
  accidental pass（详见
  [learning](../../../docs/learnings/bugs/2026-11/2026-11-11-css-color-raw-shadow-accidental-pass.md)）；
  最终态由 raw 影子保原形 + declared 写侧剥 % 双锚定。

## 已知缺口（挂账不追）

- C：atan2 混合单位型（angle/number 混槽）specified 折叠不覆盖。
- D：trig 函数时间单位（`sin(1s)` 类）折叠不覆盖。

## 探针卫生

zz-probe-hsl-none.html（本链三轮探针）已从仓库树删除；
zz-probe-cc-route.html（他流）不在本批范围。v6 记录轮 corpus 工作目录仍消费了
两探针（+1/0 pass），对账时已剔除口径说明。

## 验证

- `cargo fmt --all --check` 绿；clippy --workspace --all-targets -D warnings 绿
  （隔离 target-clippy，test-guard 守卫）。
- **make test 全量**（systemd cgroup MemoryMax=24G + Makefile 内建 per-leg
  守卫）：147 套件 **3835P/7F**。7F 为 main 既有欠账（img/svg settle + ws/net
  族），HEAD be6703d58 detached worktree 冷编译复跑同 7 测试全红（0.17s、
  同断言）定谳——与在途 hunk 无关（详见 master.md 碰撞记录定谳段）；本批
  新增测试组（r2280/r2320/part03.rs 折叠 + NaN censor）全绿。
- 单测：zero-css-parser r2280/r2320 组、engine part03.rs 折叠组、NaN censor 组
  （`individual_transform_censors_nan_at_computed`）全绿。
- corpus：`CSS_CORPUS_JSON_DIR=.acceptance/wptfyi-css/results-v6`，
  systemd scope `zeroweb-dev-guard-ZeroWeb-2-gates-v6` / `zw-v6run2`，
  二进制 target/release/zero-wpt-runner（04:29，rebuild3 态）。
- 门禁形态学习：make test 外层 RSS 求和守卫不配双腿并行编译（三连杀实录），
  顶层守卫归 systemd cgroup——
  [learning](../../../docs/learnings/platform/2026-11/2026-11-11-make-test-outer-guard-rss-vs-cgroup.md)。
