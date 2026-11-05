# M2 s294 — Image 具名构造器结构对齐 + plain IMG src IDL 反射对（Image-constructor 2F→0F、img-src-in-synthetic-document 1F→0F；全树 A/B 翻转恰 3 枚全 F→P 第 26 个连续完美；首轮 P2F=3 中途勘根——getter-only 断 expando 落点，setter 补齐闭环）

日期：2026-11-06。切片号 s294（承 s293 下一步候选之外的语料钉面——s293 B 侧
全树失败分布勘出两枚无主 img 域红：Image-constructor 2F + img-src-in-synthetic
-document 1F，候选①picture env 化/②物化缺口根修/③plain-src 触发缺口维持挂
账）。勘根+施工+验证+落账单 session 完成。

## 勘根

**面 1 Image-constructor.html 2F**（"share a prototype" + "NamedConstructor
creates the correct object structure."）：WebIDL 具名构造器结构缺位。旧
part02 R2834 普通函数版两处不合 spec（imagebitmap-and-images.html#image）：
(1) `Image.prototype` 与 `HTMLImageElement.prototype` 非同一对象；(2) 接口
原型描述符应 {writable:false, enumerable:false, configurable:false}——
function 的 prototype 属性 non-configurable 不可 defineProperty 改形，直接
赋值又留 writable:true（钉形 assert_false(desc.writable) 断）。

**方案勘根（含一轮死路）**：Proxy 包装 target 函数（gOPD/get 双 trap
'prototype' 臂报接口原型）——QuickJS 运行时抛 ES 内部方法不变式错
「trap reported non-configurable and non-writable for property 'prototype'
which is non-configurable, writable in the proxy target」（target 'prototype'
non-configurable + writable 时 trap 不得报 non-writable；失败 solo 存档
s294-image-constructor-solo.log 首轮——后被修正轮覆盖）。改 **接口原型对象
整体置换**：class 的 .prototype 天生该描述符形，置 `HTMLImageElement.
prototype` 指向 class 原型对象——三方同一化一步到位（Image.prototype /
HTMLImageElement.prototype / getPrototypeOf(new Image())；getPrototypeOf
trap part05 与 R125 _zwMEl 建链均运行时动态读，无旧对象残存观察者；part01c-f
/part02a/part07/part08 无加载期引用已核）。链修复：class 无 extends 原型落
Object.prototype → setPrototypeOf 接 HTMLElement.prototype（spec 接口原型
[[Prototype]] = 父接口原型）；class 原型自带 constructor=Image → 改指
HTMLImageElement（覆盖 R290 补装——swap 置 R290 IIFE 之后，先 swap 会被
R290 的 hasOwnProperty 幂等跳过）。工厂体逐字承 R2834（createElement('img')
proxy + width/height 属性）。

**面 2 img-src-in-synthetic-document.html 1F**：plain `_zwMEl` 工厂无 IMG
src accessor（s239 勘根既记「本工厂仅按 tag 装 href/contentDocument 少数
accessor」）。钉形：createHTMLDocument（doc URL = about:blank）img
setAttribute('src','/test') 后 `img.src` 期望 '/test'——url 型反射 getter
对 about:blank 解析失败**返回属性原值**。getter 本地反射实现（ownerDocument
.URL base + `__zw_parse_url` 失败回落原串），身份态委托 wrapper 链。

## 中途回归与修正（首轮全树 P2F=3 → 勘根 → setter 闭环）

首轮全树 B（s294-fulltree-b.log，getter-only 版二进制）A/B 出 **P2F=3**：
image-decode.html 两案 + forms checkable-active-onblur-with-click 一案。
按 s292 (二进制,run) 矩阵法勘根：

- **image-decode 两案确定性红**（solo 复跑同红，非 flake）：失败消息
  「Cannot set property src of #<HTMLImageElement> which has only a
  getter」——harness strict 模式下 `img.src=` 对 getter-only 抛 TypeError。
  s69 decode 候选读臂的 **expando 回落设计**（part03「plain 产物无 src IDL
  setter——`img.src=` 落 expando」注记）隐式钉住 setter 存在性；本片首轮
  「setter 无 in-channel 钉面」核验**误判**（只查了 html/dom reflection 簇，
  漏了本通道 decode 流的 strict 写点）。修正：getter 补 **spec 反射
  setter**（写内容属性，String(v) 原串入库不做 URL 解析，同 part05 DOMString
  口径；identity 态委托 wrapper set trap）。修正后 image-decode solo
  60P/0F 全绿（含两案），expando 回落注记同步更新（保留为旧形态兼容回落）。
- **checkable-active-onblur 一案时序 flake**：forms checkbox :active 指针
  交互域，与本片两改动无可达链；r1 红 → r2（终版二进制）绿（全树），
  solo 未复现。同码跨跑翻态 = flake（s292 矩阵判据），不入 REGRESSED。
- 另 make test 首轮暴露引擎内静态哨兵 `test_image_constructor_r2834` 末臂
  钉旧「无 new 亦返 proxy」行为——语料不钉且 spec 拒绝（WebIDL named
  constructor 不可无 new 调用）。按 s293 静态哨兵期望改正先例 + Audio
  R2835 同款先例（part09 「media-elements M3 扩批 III spec 纠正」注释同形）
  更新断言：无 new 抛 TypeError + 补 Image.prototype===HTMLImageElement.
  prototype identity 钉形。

## 施工（3 文件）

- `crates/engine/src/js_dom_shim/part02.js`（−13）：R2834 Image 函数定义整
  体迁出（定义点移 part03 接口环后——HTMLImageElement.prototype 届时就绪）。
- `crates/engine/src/js_dom_shim/part03.js`（+109 行两块）：①R290 IIFE 后
  class 版 Image 具名构造器（工厂体逐字承 R2834 + setPrototypeOf + 
  constructor 自反覆盖 + 接口原型置换 + globalThis.Image 挂载；行为收紧两
  处均 spec 正确面：无 new TypeError、`new Image() instanceof Image` 翻
  真——旧 documented false 注记随之失效，语料 Image( 全为 new 形零暴露）；
  ②_zwMEl IMG src 反射对（getter: 身份态委托 wrapper/本地 ownerDocument
  .URL 解析失败回落原串；setter: spec 反射写内容属性，identity 态委托
  wrapper set trap）。
- `crates/engine/src/js_dom_bridge_tests/part09.rs`（+24−5）：R2834 哨兵末
  臂改向（identity 钉形 + 无 new TypeError；见上）。
- 语料卫生（不入提交——wpt-data gitignored 本地数据）：删除
  zz-s272-probe2.html（s272 探针残留，自 s272 起每轮全树污染 1 枚常量
  Fail）→ 本轮 A/B 记 **LOST=1（良性卫生面）**；zz-cueprobe.html 维持
  （archived media 通道域 0 执行，规则 9）。

## 验证

- **前置**：concat node --check 18 片干净（canonical 序 = js_dom_bridge.rs
  include_str! 序）；`cargo fmt --all --check` clean；`make guarded-clippy`
  workspace -D warnings 干净（33.41s）。
- **solo 目标面**（终版二进制）：Image-constructor **5P/0F/0T**（含上轮恒
  红的结构钉形断言）、img-src-in-synthetic-document **1P/0F/0T**。
- **solo 回归修正面**：image-decode **60P/0F/0T**（getter-only 版 58P/2F →
  setter 版全绿）。
- **solo 哨兵门**（终版二进制，期望零位移）：naturalWidth
  **258P/0F/0T**（s293 B 侧基线精确同）、the-option-element **108P/0F/0T**
  （工厂 accessor 哨兵）。
- **全树 A/B**（A = s293-fulltree-b.log；B = s294-fulltree-b2.log 终版；
  双方 TIME_LIMIT=14400 RAF_FRAME_DRIVEN）：**8418 键双侧同集，LOST=1
  （zz 卫生面）GAINED=0，翻转恰 3 枚全 F→P**（Image-constructor ×2 +
  img-src-in-synthetic ×1），**P2F=0 / REGRESSED=0**，行级（原始行数）
  7982P/383F/57T → **7985P/379F/57T**（ΔP=+3/ΔF=−4=3 翻转+zz 除账/ΔT=0）
  ——**第 26 个连续完美 A/B**。首轮 getter-only 版（s294-fulltree-b.log）
  P2F=3 已全部勘根闭环（两案 setter 修复 + 一案 flake 判定），日志存证。
- **make test**（哨兵改向终轮）：**80 套件 9130P/5F** 与 s293 基线精确持
  平——5F 全为已知 CSSOM cssText 域红（p56/p99s/r2809/r2810/r3276，s288-
  s293 逐键同红），**本片零新增红**。中途两轮存证：r1 6F（+旧哨兵期望红
  ——改向对象）、r2 6F（+paint r5296_single_line_inflated 一枚——同码跨
  跑翻态 flake：r1 绿/r2 红/standalone 复跑 3/3 绿，本片无 paint 可达链，
  上游 ce83c0166..aa4fa35ce 六提交亦无 paint 改动）。
- **影响面豁免**（quality-gates 按影响选择）：本片 JS shim DOM API 语义 +
  引擎测试断言，无 layout/render/Rust 产品路径改动——`make reftest`、
  product-smoke、bench-gate 不跑；Web API 兼容修复的 WPT 附带面 = 本通道
  全树 A/B + 语料常驻钉形（Image-constructor.html 等既在语料集）。

## 下一步候选

1. picture 父 IMG 臂 env 化独立评估（s221 microtask 时序面，维持）。
2. 宿主 documentElement 物化缺口根修（s285 挂账 + s293 挂账②并案评估）。
3. plain-src 运行时 img 触发缺口根修（s293 挂账①——需先立 corpus 钉面或
   确认 runner 生命周期 sweep 已覆盖实际断言域）。
4. 余量无主 img 域面（s293 B 侧失败分布，均未立钉）：image-alternate-
   no-store EncodingError 1F、list-of-available-images coalesce 1F、
   natural-size-orientation / null-image-source / not-rendered-dimension-
   getter / update-src-complete Timeout ×4。

## 证据

- 本轮日志（evidence/logs/）：s294-image-constructor-solo.log（5P 终态轮；
  首轮 Proxy 不变式失败轮被修正轮覆盖，失败消息已录本账勘根节）、
  s294-img-src-synthetic-solo.log（1P）、s294-naturalwidth-solo.log（258P）、
  s294-option-solo.log（108P）、s294-image-decode-solo.log（60P 终态轮；
  getter-only 版 58P/2F 状态录本账）、s294-fulltree-b.log（首轮 getter-only
  版 P2F=3 存证）、s294-fulltree-b2.log（终版 7985P/379F/57T）、
  s294-make-test.log（r3 终态 80 套件 9130P/5F；r1/r2 状态录本账）、
  s294-clippy.log（workspace 干净）。
- 日志 md5：image-constructor-solo b5b00c85、img-src-synthetic 7419f0fa、
  naturalwidth 31184f12、option 7091958c、image-decode e1168414、
  fulltree-b 840ac010、fulltree-b2 68bae558、make-test f3a6b1b9、clippy
  df49fd1a。
