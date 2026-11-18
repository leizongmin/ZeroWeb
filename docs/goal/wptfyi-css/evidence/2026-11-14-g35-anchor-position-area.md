# g35 anchor 批：position-area wm/dir 映射 + §4.1 对齐面 + inset-IMCB 撤除定谳

日期：2026-11-14。批次目标：position-area 写作模式/方向逻辑轴解析（CB 写作
模式参照 plain logical 关键字、anchored 自身写作模式参照 self-* 关键字）+
§4.1 区域默认对齐表 + 显式 align/justify-self 折算 + anchor-center。

## 1. 实现

- `css-parser::values::anchor_area`：`AreaWmDir { vertical, block_rl, rtl }`
  双坐标系单实现，`position_area_cells_wm(canonical, cb, self_box)`——
  inline_low = vertical ? !block_rl : !rtl，block_low = !block_rl；12 单测。
- `layout-engine::engine::anchor_pos`：pass2 区域定位接 `cells_wm`（CB wm 查
  CB style，非 styled 盒缺省 HTB_LTR）+ 对齐面——
  - `axis_align`：显式 self-alignment 优先，否则 §4.1 区域默认表（仅 start
    轨→end、仅 end 轨→start、仅 center 轨→center、三轨全选→anchor-center；
    span-all 默认即 anchor-center）；
  - `align_from_justify`/`align_from_align`：stretch 满填、物理/逻辑关键字按
    自身写作模式折算（Flex* 视同 start/end，Safe* 按底层对齐）；
  - margin-box 口径对齐（css-align）：Fill+auto 尺寸满填区域、显式尺寸只
    定位不覆写；AnchorCenter = margin-box 中心对锚盒中心（§4.2）。
- 14 个 g1f 单测全绿（含新钉 g1f_position_area_default_table_end_pin、
  g1f_position_area_span_all_anchor_center），clippy 零告警、rustfmt clean。

## 2. 三轮 corpus 账（css-anchor-position 259 案）

| 轮 | Pass/Fail/Timeout | vs 基线 |
|---|---|---|
| triage 基线（g34 提交态二进制） | 11,869/967/4 | — |
| AB1（wm+align+inset-IMCB 面） | 11,805/1,031/4 | 净 −64 |
| AB2（inset-IMCB 撤除） | **11,870/966/4** | **净 +1，case 级逐页 REGRESSED=0** |

- AB1 REGRESSED：position-area-in-position-try.html 64→0 + 探针页
  zz-g6probe-mut6 −1；GAINED：anchor-in-anchor-positioned +1。
- AB2 GAINED 仅 anchor-in-anchor-positioned ".target 1"；in-position-try
  64/64 恢复。

## 3. inset-IMCB 撤除归因

position-area-in-position-try.html 等值断言页：#target `left: 200px`（force
fallback）与 #ref 共享 base `position-area: bottom right`，逐测注入
`@position-try --pt` + `#target { position-try-fallbacks: --pt }` +
`#ref { position-area: X }`，断言 `target.offsetLeft == ref.offsetLeft`。
#target 带 fallbacks 声明 → 区域 pass 按「任一 fallback 不介入」门跳过（保
持 left:200）；#ref 无 fallbacks → 介入。旧码（AB1 前后均）target/ref 同几
何 → 64P；AB1 给 #ref 建模 inset-IMCB（§3.1 区域成为 CB、inset 相对区域）
后 #ref 对齐落点分叉 → 等值破 64F。等值语义要汔回到「target/ref 同几何」，
单独建模 inset 必破；撤除（区域即对齐容器，inset 属性不介入，
`axis_align` 同时去 §4.1 单 auto inset 例外支），复录 AB2 确认恢复且无新
回归。inset 面后续需与 position-try fallback 联动建模（同页 fallback 声明
下 target 经 fallback 取 X 与 ref 直接 X 同几何）才可重试，挂账。

## 4. 目标桶定谳（triage 967F 中 position-area 族 30 页 236F）

**CSSOM 声明层缺口 ~190F（8 页，先行挡板）**：页面用
`element.style.positionArea = …` 设置（grep 实证：basic、align-justify、
align-justify-wm-dir、wm-dir、anchor-outside、anchor-partially-outside、
with-insets、in-grid），runner/宿主未注册 position-area CSSOM 属性 → 声明
即被丢弃、元素未被区域定位拦截（"expected offsetWidth 400 but got 0"）。
分账：basic 47 + align-justify-wm-dir 56 + wm-dir 44 + align-justify 14 +
anchor-outside 11 + anchor-partially-outside 11 + with-insets 5 + in-grid 2
= **190F**。布局面（wm 映射/对齐表）改动无法翻绿这些页，声明面支持另刀。
（align-justify-wm-dir/wm-dir 内另叠 sideways-lr/ltr R1785 parse 期归一丢
inline 反转 ~12F，被同一挡板遮蔽，非独立可计账。）

**其余 ~46F（布局面/邻面挂账）**：scrolling 族 23（001/002/005/006/007——
滚动坐标建模，挂账 scroll offset 感知）、anchor-getComputedStyle 4（§3.1
区域成为 CB 后 inset 使用值 0px 的 computed 面）、anchored-fallback
@container anchored(fallback) 查询 4、last-successful 3（fallback 状态机
挂账）、scrollable-containing-block 4、chain 1、computed-insets 1（Typed
OM styleMap position-area 槽）、interpolation 1（interpolation-testcommon
TypeError——属性插值注册面）。

语料族外残余桶（anchor()/position-try/fallback 族）维持既有挂账，本批无
回归（anchor-in-anchor-positioned +1 为 wm 面真实增益）。

## 5. 挂账（新增/更新）

1. **CSSOM position-area 声明面**：`style.positionArea` 属性注册（runner
   shim + 宿主桥），~190F 先行缺口，另刀。
2. **inset 相对区域 IMCB**：需 fallback×inset 联动建模（见 §3），单独建模
   破等值页已实证。
3. §4.1 单 auto inset 例外（随 inset 撤除同去）。
4. scrolling 族 23F（滚动上下文折算，模块头既有 FIXME 扩面）。
5. getComputedStyle inset 使用值（区域成为 CB → auto inset 使用值 0px）4F。
6. @container anchored(fallback) 查询面 4F；Typed OM position-area 槽 1F；
   属性插值注册 1F。
7. 既有：sideways-lr/ltr R1785 归一、fallback 状态机、baseline 族对齐、
   safe/unsafe 溢出折让。

## 6. 证据路径

- 基线：`.acceptance/wptfyi-css/results-g35-anchor-triage/css-anchor-position.json`
- AB1：`.acceptance/wptfyi-css/results-g35-anchor-ab/css-anchor-position.json`
- AB2：`.acceptance/wptfyi-css/results-g35-anchor-ab2/css-anchor-position.json`
- 脚本/守卫：`.acceptance/wptfyi-css/run-g35-anchor-ab.sh`（ab2 复录用），
  systemd user service `zw-g35-anchor-ab2.service`（test-guard 包裹构建与
  corpus，MemoryMax=10G）。
