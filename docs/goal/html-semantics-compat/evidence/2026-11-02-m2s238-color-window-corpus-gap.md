# M2 s238 — color.window.js 语料缺口补入 + color-well 宽域序列化（1F→196P）

日期：2026-11-02。切片号 s238（承 s237）。两层根：语料资产缺口（s227 同族）+ 引擎 color-well 序列化面缺位（part01 sanitize case 'color' 重构 + part03 getter 读侧清单）。

## 入场定刀位

s237 下一步候选：color.window.js 语料缺口（全树仅 1 行 Fail「load support
/css/support/color-testcommon.js」——文件级形态，s227 已有同族定性先例）。

## 勘根

- **语料缺口层**：color.window.js 顶部 META script 引
  `/css/support/color-testcommon.js`，语料包无该文件 → 404 → 顶层
  `set_up_fuzzy_color_test` ReferenceError → 全文件 192 表行 + 4 独立子测
  零注册，全树只剩 1 行 Fail。定性 = 语料缺口非引擎缺陷（s227
  update-media 同构）。补入：raw.githubusercontent.com 与 HTTPS github.com
  本环境超时（exit 28），走 SSH（origin 同路）sparse clone 上游原件——
  `git clone --filter=blob:none --no-checkout --depth 1 --sparse
  git@github.com:web-platform-tests/wpt.git` + `sparse-checkout set
  css/support`，169 行自包含 JS（fuzzy_compare_colors：数字段 ε=0.01 容差、
  非数字段**逐字符精确**——序列化格式必须逐位吻合含空格）落
  `tests/wpt-runner/wpt-data/css/support/`（gitignored 本地数据，升级新
  ref 后需重新补；`scripts/update-wpt-data.sh` known-gaps 挂账）。
- **引擎层（补资产后现形，双层根）**：首跑失败矩阵两形态——
  ①`expected "color(display-p   )" got "color(srgb   )"`（格式面）+
  `lengths differ, expected 3 got 0`（getter 返 ''）。勘定：
  - **根 1**：INPUT value getter（part03）读侧消毒清单止于 email
    （s194，FIXME 在账）无 color 臂——IDL setter 存净化形（s183 算法），
    但「先设 value 后改 colorSpace/alpha 属性」序列要求按**最新属性态**
    重序列化（color.window.js「Setting colorspace/alpha by setAttribute
    should update the value」钉面），getter 无臂则改属性后读数不变；
    `setAttribute('value')` 存原串（非 dirty 不净化）直接漏原样。
  - **根 2**：color-well 宽域序列化面缺位——s183 只做 limited-srgb 无
    alpha 的 `#rrggbb` 臂。color.window.js 钉态：colorspace=display-p3 →
    `color(display-p3 R G B)`；alpha 属性在场保留 alpha 通道且
    limited-srgb 也转 `color(srgb R G B)` 形；α=1 省略 alpha 分量、α<1
    ` / a`；越界分量保真（`color(display-p3 3 none .2 / .6)` 期望保
    `3`、none→0）——宿主 `__zw_parse_color_rgba` 经 u8 rgba 钳制
    （3→255）不可达。

## 桥力学

- **幂等闭环**：IDL setter 存「sanitize 时属性态」的净化形 → part03
  getter 读侧重净化（新增 color 入清单）→ 属性变更后按最新属性态重序列化；
  setter 存的旧净化形重净化幂等（display-p3 输入 `color(display-p3 …)`
  串被预解析臂直解，`_cvSrcP3` 位保持，不再二次转换）。
  `setAttribute('value')` 存原串走同臂补面（s183 FIXME 内容属性变更面）。
- **shim 侧 color() 预解析**：`/^color\(\s*(srgb|display-p3)[\s/]+([^)]*)\)$/i`
  → `/` 分段 → 分量 `/^[+-]?(?:\d+(?:\.\d+)?|\.\d+)$/` 原值保留（不钳制）、
  `none`→0、可选 `/alpha`（alpha none→0）；不匹配回落宿主 u8 解析
  （s183 族面）。
- **色域转换**（CSS Color 4 矩阵对偶；gamma 曲线两空间同形 1/2.4 +
  12.92 线性段）：sRGB→display-p3 = gamma decode → XYZ(D65) → lin_P3 →
  gamma encode；逆向 p3→srgb 语料无钉面，按逆矩阵对偶实现（lin_P3→XYZ +
  XYZ→lin_sRGB）。sRGB ⊂ display-p3 正向无 gamut clip。
- **序列化**：分量 6 位小数舍入去尾零（`Math.round(x*1e6)/1e6`），前缀
  `color(srgb ` / `color(display-p3 `，α<1 追加 ` / a`——fuzzy 比对非数字
  段逐字符精确，格式（空格/斜杠）逐位对齐。解析失败回落不透明黑按同
  模式序列化（spec「失败视同 #000000」锚点）。
- **s183 快臂不动**：`!_cvIsP3 && !_cvKeepA` 时原 `#rrggbb` 路径原样保留
  （color.html 27/27 钉面）。

## 施工（两文件，零 rust）

1. part01 `_zwSanitizeInputValue` case 'color' 重构：lw 融合读
   colorspace/alpha 属性态（`__zw_get_attr_lw`/`__zw_has_attr_lw`，
   handle/lw/裸三层回落，try/catch 包裹）→ 快臂分流 → color() 预解析 →
   转换 → 序列化。s238 注释 + HTML §color state / CSS Color 4 规范链。
2. part03 INPUT value getter 读侧清单 `|| _rvTyG === 'color'`（s238 注释
   记幂等闭环）。

## 验证

- node --check 全 shim concat（canonical 序自 js_dom_bridge.rs
  include_str! 提取）语法干净。
- 数学预验（node）：crimson/bisque/rgb(1,1,1) 三行 p3 转换 6 位小数精确
  吻合语料期望。
- solo（s238 二进制）：color.window.js **196P/0F/0T**（192 表行全绿 + 4
  独立子测：out-of-bounds / colorspace setAttribute / alpha setAttribute /
  non-dirty reparse whatwg/html#12057）；color-attributes.html 2P（s187
  IDL 反射钉面）；color.html 27P（s183 钉面）；default-color.html 2F =
  s237 基线同键同态（dialog 面存量）。
- **全树 A/B**（evidence/logs/s238-ab.log vs s237-ab.log，(key,state) 对
  sort -u join）：LOST 1 = color.window.js「load support …」Fail 行（语料
  缺口行，补资产后消失）；GAINED 196 = color.window.js 196 Pass 行——
  **恰靶面**。文件键形变注记（s227 同族）：该文件从 1 行 Fail 变 196 行
  Pass，join 键集 +195。
- **非靶位移 2 族，全定谳非回归**：
  - checkable-active-onblur-with-click.html「pressing the following
    label」×2 P→F：机器态时间漂移——s237 全树（今晨）该二键 Pass，此刻
    HEAD 二进制（stash 法对照重建）solo 同形状 Fail（following ×2 Fail /
    parent ×2 Fail / it ×2 Pass）+ s238 二进制 solo ×2 复现同形状（3 rep
    跨二进制一致）；语义隔离：改动臂严格 type 门（case 'color' /
    getter 清单 +color），checkbox/radio 不进任何改动臂，pointer/:active
    面无通道。
  - select-home-end-pagedown-pageup-detailed.optional.html 4 子测 Fail
    行 → 1 文件级 Fail 行（子测名=文件路径=文件整体未跑起的塌缩形）：
    并行 8 job 负载形；solo（s238 二进制）现基线 4 键 Fail 形（文件正常
    起跑逐子测红），净态双侧同红零翻转。
- 行级分布 7512P/508F/81T → 7706P/506F/81T：ΔP=+194（+196 靶面 −2
  checkable 漂移）、ΔF=−2（−1 语料缺口行 +1 select-home-end 塌缩行 −4
  原子测行 −2 checkable 漂移）精确闭合；Timeout 81→81 零位移。

## 合并基线复核（收口轮）

s238 验证原在 pull 前基线完成；收口轮 `git pull --rebase` 并入并发流 5 提交
（crypto part01b/02/02a + paint text.rs R5390，均不触 input value 通道）后
重建 release runner 复跑 solo：color.window.js **196P/0F/0T** 与原验证逐行
同态，全 shim concat node --check 干净（part01/part03 本地改动与上游
part01b/part02/part02a 拼接无冲突）。

## 残余挂账（本切不动，注释已注记）

- p3→srgb 逆向转换无语料钉面，按 CSS Color 4 逆矩阵对偶实现（正逆
  矩阵互为逆，未独立验证往返误差）。
- color() 的 percent 形与非 number token（如 `calc()`）未处理，回落宿主
  u8 解析（钳制面）。
- 表单提交入口读 raw `_controlValue` 非净化形（语料无钉面，s236 已记
  同族账）。

## 下一步候选

plain value 死路补面（s236 新账，与 s228 车道交叉评估）；
modal-dialog-selection（pointer 拖拽面存量，s237 已定谳基线）；
checkable-active-onblur-with-click 漂移族观察（若下轮基线同形状则记基线态销账）。
