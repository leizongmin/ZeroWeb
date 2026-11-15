# WAAPI 门刀落地——Web Animations face 全域解锁

2026-11-15。承接 [逐组转化率实测](2026-11-15-texp-groups-conversion.md)
§3/§5 第一刀。改 2 shim 文件（part03/part05），无 Rust 侧改动。

## 1. 根因与修法

门：`interpolation-testcommon.js:179`
`webAnimationsInterpolation.isSupported = 'animate' in Element.prototype`。
探针实证（刀前同二进制）：`typeof el.animate === 'function'` 且真跑通
（`animate()` → paused / currentTime=500 → computed opacity=1），但
`'animate' in Element.prototype` 与 `'animate' in el` 均 false——R2965
animate / R3067 getAnimations 只在实例 get trap 面，原型与 has 面缺失。

修法（两切片）：

- **part03.js**（`_makeAnimation` 后置块）：`Element.prototype` 挂
  `animate`/`getAnimations`（`!prototype.animate` 守卫防重复接线），
  identity 经 R351 顶部短路键 `this.__zwSelector`/`this.__zwHandle` 反解
  （part01b preservesPitch accessor 同款先例）委托既有 `_makeAnimation`
  与 `_elementAnimations` 注册表。实例 get trap 闭包优先命中，行为不变。
- **part05.js** has 白名单：补 `animate`/`getAnimations` 两名（R317/R152
  同款先例形态）——`'animate' in el` 实例面翻真。

## 2. 验证链

- concat 7 片 `node --check` OK；`make test`（守卫，systemd scope）：engine lib
  **4,032P/0F**，workspace 全绿，唯一 F =
  `image_batch_e2e::renderer_publish_batch_image_fetch_reaches_concurrent_in_flight`
  ——在册负载脆弱 flake（R5295/R5310/s437 同族，本次与 MyKernel 构建同机并发窗），
  隔离复跑（test-guard 包裹）**1 passed 绿**，记账不阻断。
- zz-mprobe-waapi（刀后二进制 09:53:08）：proto-has=true、inst-has=true、
  proto-typeof=function、anim-run 链路不损（paused/opacity=1）。
- A/B（五组 p1/p2 复跑，B0=results-texp-b1-{p1,p2} 门未开基线 /
  B1=results-waapi-b1-{p1,p2}，comparator 逐页 Pass 集比对）：

| 组 | B0 P | B1 P | Δ | 回归页 | WAAPI 门子测 B0→B1 |
|---|---|---|---|---|---|
| css-transforms | 265 | 282 | +17 | 0 | 897→0 |
| motion | 0 | 0 | +0 | 0 | 929→0 |
| css-backgrounds | 787 | 840 | +53 | 0 | 952→0 |
| css-values | 515 | 606 | +91 | 0 | 867→0 |
| css-fonts | 313 | 336 | +23 | 0 | 426→0 |
| **合计** | **1880** | **2064** | **+184** | **0** | **4071→0** |

签名迁移：waapi 门 4071→**0**（门全开，face 真跑）；supports
8565→10836（+2271，WAAPI face 复用同一 CSS.supports 前置门，门杀迁移
入该桶）；语义 7002→8618（+1616，face 真跑后的插值语义残差）。净转化
+184P 零回归——门面残差大头移交给 supports 覆盖刀族与插值语义刀。

- 附带归一：2 页分母差（text-decor discrete-no-interpolation 252/336、
  css-text hyphen-no-interpolation 126/168，恰为 25% WAAPI face）刀后
  声明数应达 chrome 分母（WAAPI face 子测注册恢复）。
- 主链一次失误记录：首轮 A/B 链首组误写 corpus 名 `transforms`（合法名
  `css-transforms`）→ usage 退出零产出，其余四组正常；复跑修复后比对
  数据完整。教训：链脚本 corpus 名以 wpt-data 顶层目录名为准。

## 2.1 提交前去冗余与复验（11-15 收口轮）

初版改动含两处同轮冗余，提交前清理：part03 第二原型接线块
（`try{ if(... && !prototype.animate) ...}` 被 `!prototype.animate` 守卫
永不可达——首块已 `Object.defineProperty`，系早期版本残片，整块删除，
保留带 `__zw_selector_for_handle` 反解的完整版）；part05 has 白名单
`animate`/`getAnimations` 两名在**同一 has-trap OR 链**重复出现两次，删
后者（同链非双 trap 面）。清理为死代码/重复条件删除，行为零改动。

复验（去冗余后重建 runner，14:04:46 > shim 编辑 14:01:37/40）：
`zz-mprobe-waapi` 探针 proto-has/inst-has 双 true、anim-run 链路不损
（paused/opacity=1），与刀后初版探针逐项一致。

## 3. provenance

刀后二进制 mtime 09:53:08（> part03/part05 编辑 09:49–09:52）；
`cargo build --release -p zero-wpt-runner` 57.75s（systemd scope
MemoryMax=12G，显式注入 cargo PATH——systemd service 无 login PATH，
首跑 `cargo: command not found` 记忆项复证）。B1 结果 JSON 在
`.acceptance/wptfyi-css/results-waapi-b1-{p1,p2}/`，探针页留 wpt-data
（gitignored），comparator `.acceptance/wptfyi-css/waapi-ab-compare.py`
（本机产物不入库）。

## 5. 地图校正（11-15 九轮收口，改下一步方向）

收口轮校准探针（B0 态二进制 14:04:46）定谳两条前轮误读：

1. **§2「动画链路不损」过读**：anim-run 探针 `animate([{opacity:0},
   {opacity:1}])` currentTime=500（t=0.5）读 opacity=**1**——若值已应用应为
   0.5。Animation 对象链路通（playState/currentTime 正确），**动画值未进
   同步 gCS**。
2. **texp evidence §3「keyframe-inherit-anim 证 from 帧被丢」是探针伪账**：
   kw2 定谳探针（对照组分解）六个变体全 `auto`——JS 建 `<style>` +
   `from{height:100px}→200px` 普通值与 static `<style>` 普通值**同样不
   应用**，非全局关键字特有；同步 gCS 读根本不反映 @keyframes 动画值。
3. corpus 现有插值通过面走 **CSS Transitions 离散路径**（grid-template-
   rows-interpolation 单页 574P，Pass 签名集中 allow-discrete flip）；
   min-height-composition 0P/60F 证 additive composition 同样不反映。
4. harness 门面核查：interpolation-testcommon 仅门 WAAPI `isSupported`
   （本轮已修）与 custom-props；`supportsValue` 对 css 方法恒 true——
   texp §3 的「CSS.supports 覆盖门 25,135」来自**页级前置调用**而非
   harness。

结论：残差主矛盾从「supports 刀族」上移为**动画值进 gCS 同步读链**
（WAAPI KeyframeEffect→引擎动画时钟→computed 通道断点待定位）。校准
探针页 zz-mprobe-kw2 留 wpt-data（gitignored）；A/B 比对器与各探针
transcript 在 `.acceptance/wptfyi-css/`（本机产物不入库）。

## 4. 后续

1. **supports 覆盖刀族**（[转化率 evidence](2026-11-15-texp-groups-conversion.md)
   §5 排刀序）：全局关键字切片先行（cascade 实证全通，`decl_supported`
   + @keyframes apply 两拒点），再 shape() / calc-size（peer WIP 复用）/
   mask-border / backdrop-filter / offset-* 族逐个 A/B。
2. 插值语义长尾（WAAPI/supports 刀后占比上移为主矛盾）。
3. 全 42 组 WAAPI face 增量收口留 DC-3 全量对账（本轮五组抽样 A/B 已证
   门面归零、零回归）。
