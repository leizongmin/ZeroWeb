# DC-2 css-color 组E：域外相对色保形序列化 + opacity 数学形读面（css-color 10,905 → 10,948 净 +43）

日期：2026-11-11。前序：[组F 布局刀3 anchor() inset](../../..//docs/goal/wptfyi-css/master.md) 同日提交
（405b12a3c，另一流）。本批只动 engine shim 两文件，与布局流零交集。
归属记账（对称 resume 仲裁定案）：fix1-3 + part01 双门 = color 流前期批
（5a/83/d3 对称 resume 共同继承，作者归属 siblings 间不可裁，按 face 记账）；
fix4-5 + 证据终稿 + 提交 = d3（live 提交 owner）；fd1-gate2 全量与语料账 = fd；
bin leg 319P/0F 与 engine leg 账 = color 流 resume 节点递交（5a 回执）。

三桶归类（g3 基线 106F：system-color-consistency 27F、opacity-computed 20F、
relative-color-out-of-gamut 20F、长尾 39F）中本批取后两桶 + 连带长尾。

## 根因与修法（五处）

### part11-computed-color.js 四修（写面/序列化面/投影面）

1. **相对色序列化全面取消通道钳制**（原 ~1931-1937）：css-color-4
   §outside-gamut + css-color-5 §serial-relative-color——域外值不 gamut-map
   不钳，color(srgb) 保形回读；mozilla wg-decisions#1125 color() 与 legacy
   rgb/hsl/hwb 目标一律不钳。hsl/hwb 经 hue 回读展开后负通道原样携带。
   out-of-gamut 页 20F（hsl/hwb 目标钳致 (0,1,0) 形）钉面。
2. **hwb+hsl resolve-tail 解除 s/l/w/b floor**（原 1811）：域外转换
   s = C/(1−|2L−1|) 为负、hwb w/b 携负，floor 会毁 color(srgb) 回读展开。
   语料双锚：`hwb(from oklab(0 …) h w b)` w=−0.0456（floor 毁回读 g 通道）；
   `hsl(from lab(100 104.3 -50.9) h s l)` want color(srgb 1.5935 0.58776
   1.40555)——s=−5.5486 floor 成 0 即灰 1.09（g4 轮实残留 2F 根因，g5 刀
   撤除后翻绿）。hue 归一 0..360 不变；域外 hue 有定，非 powerless。
3. **oklab origin 投影换数值真逆**（原 1223）：OkM1i/OkM2i 发表逆常数 10 位
   舍入偏差 7.6e-4，relative-color-out-of-gamut 的 1e-4 容差超差；换真逆
   OkM1t/OkM2t（13 位，f64 往返 ~1e-15，Python 法证：发表逆路径逐位复现错值
   1.5930931、真逆路径复现 Chrome 期望 1.59297，差 8.8e-6）。自形序列化面
   `_zwCCOklabToXyz` 保发表常数口径不变（防自形序列化锚漂移）。
4. **`_zwCCMixSide` hsl powerless hue 判据收窄为真消色**（原 1359）：原式
   `v[1] <= 1e-9 || v[2] <= 0 || v[2] >= 1` 把域外负 s、l>1 一并坍缩
   h=none（css-color-4 §hsl powerless = s=0 或 l 恰 0/1；域外色相有定，
   语料锚 h=311.2°）。none 化致 color-mix 结果走 hsl 自形保形分支——
   `color-mix(in hsl, lab(100 104.3 -50.9) 100%)` 实测
   `hsl(none -554.86 109.06)`（color-mix-out-of-gamut 2F，g3 即有）。
   收窄后 hue 参与回读展开，mix 结果落 color(srgb 1.59343 …) 期望形。
5. **同 2**（g5 轮补刀，撤 hsl floor 使 2 号修覆盖 hsl）。

### part01.js 双臂 opacity 数学形抢答门（读面）

宿主 `parse_opacity`（css-parser parse_basic.rs:682）拒 calc()/min()/max()/
clamp() 数学形 → 级联落初始 '1' 抢答失真；写面已常量折叠存 calc(N) 形
（part11 `_zwCOpacNormalize`，opacity-valid 30/30 已通）。gCS 两臂
（getPropertyValue + 直取 proxy）各加一臂：inline 声明（getPropertyValue +
`__zwRawDecl` 影子优先）命中 `/^(calc|min|max|clamp)\(/i` 即落
`__zwPlainComputed` 重读（part01c opacity 臂 + part11 `__zwOpacityComputed`
clamp [0,1]，css-values-4 §10.12/§10.13 NaN 规约在管线内）。裸 number/% 形
宿主可正确产出不接管（零漂移）。

宿主 parse_opacity 本体不改（style-system/apply.rs opacity 臂为另一流六文件
面，避让）。

## 验证

### g4 轮（19:38，results-g4，fix1-3 + part01 双门）

- **css-color 语料 64 页：10,905P/106F → 10,943P/68F（净 +38），逐页零回归**：
  - opacity-computed.html：20F → **0F**（全绿）。
  - relative-color-out-of-gamut.html：20F → **2F**（7p/20f → 25p/2f）。
  - 其余 62 页逐页 pass/fail 精确不变（compare-g4.py 对账，零漂移）。
- **css-anchor-position 语料 253 页：11,862P/953F → 11,863P/953F**：预期 +3
  精确落齐（anchor-position-inline-002 0p/4f→2p/2f、anchor-scroll-js-expose
  +1P），证明 color 面与布局面零耦合；inline-001/003 各 1p→0p 为
  405b12a3c（19:27 提交）对 g3 基线（18:48 提交前工作树二进制）的布局流自身
  delta，非本批（本批门仅在 opacity 数学形 inline 触发，与 anchor 几何无通路）。
- make test 全量：fmt 前置门禁于 anchor_pos.rs:253 格式漂移快速失败
  （405b12a3c 漏过 fmt，另一流已补 be4c89a60），g5 轮补跑。

### g5 轮（终版，fix4-5 补刀后）

本流自起 g5 链因 systemd-run 相对路径启动失败（status=127，journal 可证，
链未执行任何阶段）；fd1-gate2（zeroweb-fd1-gate2 unit，run-fd1.sh）等待环见
场清后接跑同树 make test + release 重建 + css-color 语料，其账经 provenance
注记复用为本批 gate。anchor 语料零漂移对账由本流 run-g5-anchor.sh 补跑
（复用 fd1-gate2 二进制，不重建；results-g5/css-anchor-position.json）。

- **make test 全量（五修树）：3875P/7F**——7F 与 main 既有欠账组逐名一致
  （img/svg settle、websocket 族、fetch/iframe/stream，见控制面碰撞记录
  be6703d58 冷编译定谳段，不随本批携带）；fmt 门过（be4c89a60）。
- **wpt-runner bin leg（五修态）：319P/0F**（logs/wptrunner-bin-g4.log，
  19:57 编译 provenance）。
- **css-color 语料 64 页：10,943P/68F → 10,948P/63F（净 +5），逐页零回归**：
  - relative-color-out-of-gamut.html：2F → **0F**（页全绿，fix5）。
  - color-mix-out-of-gamut.html：2F → **0F**（页全绿，fix4）。
  - color-computed-relative-color.html：1F → 0F（尾页连带清零——翻绿案
    `color(from hsl(from color(xyz-d65 0.99 0.88 0.77) h s l) xyz-d65 x y z)`
    嵌套 hsl 中继形：floor 撤除让域外 s 穿过中继腿，外层 xyz 往返收回 0.99，
    fix1 钳制往返族的 hsl 中继变体）。
  - 其余 61 页逐页 pass/fail 精确不变（compare-corpus.py 对账）。
  - 批次累计：10,905 → **10,948（净 +43）**。
- **css-anchor-position 语料 253 页（20:14，results-g5/css-anchor-position.json）**：
  **11,863P/953F，对 g4 账面逐页零漂移**（zw-g5-anchor-2136 unit，
  run-g5-anchor.sh，复用 fd1-gate2 二进制不重建；fix4/5 仅触色值管线，
  anchor 几何零通路实证）。

## 残留与后续刀

- system-color-consistency 27F、长尾 ~39F 未动（另桶归因）。
- 宿主 parse_opacity 拒数学形为共享路径根因，读面门为本批规避式修法；
  根因修（apply.rs/parse_basic.rs）留挂账给布局流合并后统一处理。
