# M2 s245 — img SVG 密度族勘根分型 + SVG payload percent-decode 弹性补面（37F 家族面级归属定谳，4F 翻绿）

日期：2026-11-02。切片号 s245（承 s244）。根：s244 下一步候选 img SVG
密度族独立勘根轮（naturalWidth-naturalHeight-width-height 22F +
current-pixel-density 13F + svg-img-density-vs-zoom 2F，s219 密度切片
遗留 SVG/decoder 挂账族）。

## 入场定刀位

s244 下一步候选：img SVG 密度族独立勘根轮。

## 勘根（探针两轮 zz-s245-probe.html 验后删）

- **家族分型**（s244-ab.log 37F 逐行提取）：naturalWidth-naturalHeight
  22F 全部是六类 SVG 双轴缺席+viewBox 场景（no-dims/percentage/negative-
  percentage/negative-width/negative-height/both-negative）×（plain/1x/2x
  ×rendered/not-rendered）变体；current-pixel-density basic 6F = 3 SVG
  viewBox 2x（width=20 期望 10）+ 3 PNG expect-0（10000x/9e99…x/256w+0px）；
  error.html 7F = 404 破图族；svg-img-density-vs-zoom 2F = no-intrinsic
  density+gBCR、intrinsic-width zoom gBCR。
- **probe1 关键发现（先翻案再定根）**：六类场景的 plain 变体 **not-rendered
  全 Pass**（assert naturalWidth 300/100 先行）→ SVG 固有尺寸核
  `__zwSvgNaturalDims5277`（s71 组合表）与 settle 消费链**已正确**；失败面
  不在 natural 值。probe1 同时给出 layout 车道地面真值：plain vb img
  gBCR=300×100、clientWidth=300（layout 消费 reported dims，**无
  containing-block 拉伸**——测试期望 img.width=720）、srcset 变体
  gBCR=**784×0**（无 intrinsic → 拉伸+高 0）、zoom 容器内 gBCR=300×150
  （CSS zoom 几何未实现）、10000x PNG clientWidth=1（sizing.rs `w.max(1.0)`
  钳）、containingBlock gBCR=784×0。
- **probe2 定谳（精确 fixture + 传参修正）**：percentage+viewBox srcset 1x/2x
  natural=**300×150**（期望 300×100）复现失败面；decode 链三证——
  `pv.pay=RAW-ENC`（payload 整串未解码）→ `core={300,150,none}`（属性/
  viewBox 全部找不到——`%20` 非空白，`_svgAttr` 空白前置匹配落空）；对照
  `nvb.pay=decoded`（无字面 % 的 srcset）→ `core={300,100,none}` ✓、
  `pvPlain.pay` 解码 ✓（src 原生空格，非法 `%'` 抛出后 raw 已可用）。
- **根因链单点**：`__zwSvgPayload5277` 非 base64 臂整串
  `decodeURIComponent`——遇单非法 % 序列即抛（percentage src 的 `%'`），
  catch 后 raw 保持全编码态。srcset 克隆变体把空格编码成 %20（测试
  `replaceAll(" ","%20")`），百分比 payload 两者叠加 → 属性全部缺席 →
  300×150 无 AR。plain 变体 raw 天然带空白 → 抛出后仍可解析 → 通过——
  这解释了「plain not-rendered 过、srcset not-rendered 挂」的分型。

## 37F 面级归属（本轮定谳，车道分工）

- **shim 车道（本轮翻绿 4F）**：percentage/negative-percentage +viewBox
  srcset 1x/2x not-rendered 四面（payload percent-decode 单点）。
- **渲染流车道（33F 挂账，ground truth 已移交）**：① naturalWidth 文件
  18F rendered 面 = img.width/height IDL 期望 720/240（data-width——
  containing-block 拉伸的**渲染尺寸**；layout 无拉伸语义 gBCR=300×100，
  IDL 宽 getter headless 近似回自然尺寸——s71 FIXME 真实几何归 layout
  集成面）；② svg-img-density-vs-zoom 2F = gBCR 渲染尺寸 + CSS zoom 面
  （zoom 几何未实现）；③ current-pixel-density basic 3 SVG = clientWidth
  （784×0 拉伸盒 vs 期望 10——layout 的 img intrinsic 未消费 SVG 固有
  尺寸，**zero-engine build_img_intrinsic_all SVG 面 = 可行翻绿候选**）；
  ④ basic 3 PNG expect-0 = clientWidth 1px 钳（layout-engine sizing.rs
  `w.max(1.0)`）；⑤ error.html 7F = 404 破图 alt 盒渲染面。

## 施工（一处文件一块，零 rust）

- part03 `__zwSvgPayload5277`：decodeURIComponent 抛出臂加 WHATWG
  percent-decode 弹性回退——非法 `%`（后随非双 hex）转义 `%25` 再解码，
  合法 %XX 与多字节 UTF-8 序列路径不变（url.spec #percent-decode 锚；
  s245 注记块）。

## 验证

- node --check 全 shim concat（bridge include 序 18 片）干净；
  `make zero-wpt-runner-release` 重建 28.5s。
- 探针复测（验后删）：pv/pv1 `pay=decoded` → `core={300,100,none}` →
  natural 300×100 ✓；nvb/pvPlain 对照零位移。
- solo 三门：naturalWidth-natural-height-width-height **22F→18F（恰 −4）**、
  current-pixel-density 13F 零位移、svg-img-density-vs-zoom 2F 零位移。
- 全树 A/B（evidence/logs/s245-ab.log vs s244-ab.log）：见下节。

## 全树 A/B（evidence/logs/s245-ab.log vs s244-ab.log）

- 行级 7709P/506F/81T → **7712P/503F/81T**（ΔP=+3、ΔF=−3）。(key,state)
  对（expected 尾截断 + LC_ALL=C sort/comm 同序）7832/7832，LOST 5 /
  GAINED 5：**4 目标键精确 F→P**（percentage/negative-percentage +viewBox
  srcset 1x/2x not-rendered 恰四面）；另 1 对 = checkbox `:active` 交互
  时序族 P→F——**同跑日志内 2F+2P 混态自证 flaky**（s244 同日志 1F+3P），
  s238/s239 在册漂移族同形（checkable-active 家族，跨二进制复现），非本切
  通道（本切改动仅 data:SVG payload 解码，该测试无 SVG 面）——按 s239
  先例记基线态销账。
- 改动臂单点零通道：仅 decodeURIComponent 抛出路径生效（解码成功面不变），
  与 A/B 无其他位移一致。

## 残余挂账（本切不动）

- 渲染流车道 33F（面级归属与 ground truth 见上节——拉伸/zoom/1px 钳/
  alt 盒归 layout-engine 流；basic 3 SVG clientWidth 归 zero-engine
  build_img_intrinsic_all SVG 面候选）。
- s246 候选：zero-engine `build_img_intrinsic_all` SVG payload 面（Rust
  侧 SVG 固有尺寸解析喂 layout img intrinsic——3 SVG clientWidth 可翻，
  需核对 extract→layout 消费契约）。
- s241 存量（直写臂同步清单/`_zwOptionValue` 键派生）、s240 存量（plain
  detached select.value/option.text 字面差）维持。

## 下一步候选

s246 zero-engine build_img_intrinsic_all SVG 面（basic 3 SVG clientWidth，
需独立勘根轮核 extract→layout 契约）；customizable-select 177F（挂起待
用户拍板）；UA modal dialog 居中 sheet 缺位（渲染流车道）。
