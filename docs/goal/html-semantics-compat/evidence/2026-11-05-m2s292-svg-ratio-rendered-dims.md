# M2 s292 — SVG 固有尺寸布局缺口三件 + img width/height IDL 渲染维几何臂（naturalWidth 18F→12F、density-vs-zoom 4F→3F；全树 A/B 翻转恰 7 枚全 F→P 第 24 个连续完美；upstream reftest 确定性修复 4 面 + 129/150 时序 flake 族定谳）

日期：2026-11-05。切片号 s292（承 s291 下一步候选①——SVG 纵横比-only
填充，rendering 域跨切）。勘根+施工主体为前一轮 session 产出（429 中断于
落账前），本轮接续完成：rebase 后全量重验（含今日 A/B 对、reftest 腿、
make test）+ 跨流归因（r3033）+ 落账。

## 勘根

1. **Gap-0 零维 viewBox 伪有效比**（`svg_intrinsic_kind`）：旧只护
   `vh > 0`，`vw = 0` 时 `ratio = 0/vh = 0` 伪有效 → ComputedIntrinsic
   `w/0 = +∞` 或 RatioOnly(0)，no_ratio 不落表 → 布局/naturalDims 双面
   失真。css-images-4 §5.4.1：零固有维图像视为无固有 → 落 NoRatio（attr
   维保留），R3760 早退臂据此设 no_ratio——R3760 判词「0 维 viewBox 无有
   效 ratio → NoRatio」的对跖半边（旧只护 vh 半边）。单测 ×2
   （vw=0 + width attr → NoRatio{Some(60),None}；无 attr →
   NoRatio{None,None}）。
2. **Gap-0b 负维属性 decode 全断**：usvg 对负 width/height 拒整文档解析
   → decode Err → fetch 侧丢弃 → 三表全空（naturalWidth 语料负维 4
   plain 面双失真）。SVG2 值校验：负值非法视同未指定 →
   `strip_negative_svg_dims`（引号感知 attr 切分，viewBox 值含空格不得
   裸 split；无负维原样返回零拷贝）。单测 ×2（负 px / 负 % 同剥 →
   RatioOnly(3)）。
3. **缺口1 ratio-only 约束方程 frame 缺口**（`apply_replaced_element_
   sizing` all-auto 臂）：R2054 C2 把 viewBox-only img 撑满父 content
   宽但未减自身 margin/border/padding——CSS2 §10.3.4 块级替换元素约束方
   程 margin-box = CB 宽 → used content 宽 = CB − 自身水平 frame
   （margin auto used = 0，% margin 相对 CB 宽；border/padding 复用
   frame_size，style none 归零）。naturalWidth 语料驱动形：740 CB + 5px
   border/margin img 应 content 720×240（chromium oracle border box
   730×250 实证），旧渲染 750×256.67。CB 不可解析（fallback 300 default
   object size）不减——default 非约束方程解。kill-switch
   `ZW_RATIO_CONSTRAINT_FRAME=0`。
4. **缺口2 no-ratio img attr 臂 bogus 比注入 + 缺口3（L1）注入误读
   bogus pixmap**：无 viewBox SVG 无自然比（CSS Sizing 4 §4.1）——
   (Some(w),None)/(None,Some(h)) attr 臂 R2172 注入 bogus 解码 pixmap 比
   （density-vs-zoom "intrinsic width only: baseline"：width=60 SVG 应
   60×150，bogus 比 60/100 → 旧 60×100）。修复：no_ratio 键命中时跳比注
   入，缺轴（CSS auto）落 default object size 对应轴（300/150，
   SVG_DEFAULT_W/H 同源）；对称臂同口径。L1：零维 viewBox R3760 早退
   1×1 形态 sizes 表存 bogus 1×1、no_ratio 存真维 (Some(60),None)——
   s283 auto 态固有维注入旧读 sizes 表 → 内容宽 1；修复后 noratio img
   注入先取 no_ratio 真维（密度折算同通道口径），无真维保持 auto。
   kill-switch `ZW_NORATIO_ATTR_GUARD=0`（缺口 2+3 共用）。守卫：Computed
   Intrinsic（有 viewBox）不入守卫，R2172 比注入保留（r5421
   computed_intrinsic_ratio_injection_kept 钉面）。
5. **shim IDL 渲染维臂 + 信号门**：img width/height IDL「已渲染」=content
   box 渲染维（HTML #dom-img-width rendered dimensions；chrome
   LayoutReplaced::ContentWidth 口径；CSS2 §10.3.4 约束方程解出的 used
   content 尺寸）——旧 dN==null 恒返密度校正自然尺寸（viewBox-only 撑满
   CB 面渲染 720×240 而 IDL 旧返 300×100）。门 = host
   `__zwImgHasIntrinsicSignal`（快照三表任一含该 img 资源键才进几何
   臂）：运行时创建/换源 img（script 注入、srcset 换源）不在 load 期
   fetch_image_subresources 清单，cached_* 布局表无该资源键——此时布局
   盒走无信号 auto 回落（CB − margin 拉伸），IDL 读几何会把布局缺口泄进
   断言面（srcset 克隆 62 P→F，replica 探针勘根 2026-11-05）；未命中 →
   回落 naturalDim 近似（旧行为零回归）。源键口径同 naturalDim 快照回落
   （R5275：选中源 raw URL 优先，缺失回落 raw src）。几何臂：近似链
   handle 身份先经 s284 `__zwImgResolveIdentity5277` 解析；两轴密度校正
   自然维任一 > 0（非空像——零固有维图 IDL 恒 0，布局钳 1×1 读几何反翻
   红）且 gBCR 非全零（≈ being rendered，s274 模型）才读 rect；
   content = border-box rect − 该轴 frame（gCS border-*-width 序列化已
   按 style none/hidden 归零 R3030 面；% padding 按 % 数值近似，
   FIXME: 随几何底座补 % 解析）；减后 ≤ 0 回落 naturalDim 近似。
6. **运行时表缺口定谳 s293**（非本片）：srcset 克隆 img 布局表无键 →
   布局无信号拉伸（replica3 srcset 臂 A/B 双侧同值 bare1/w601 bb 730、
   vb1 bb 高 10）+ IDL 几何臂关门 → naturalWidth srcset/1x+2x 12F 维
   持。修复方向 = load 期外 img 装载时同步 cached_* 三表（挂账 s293）。

## 施工（4 文件 + 1 新集成测试）

- `crates/render-foundation/src/image_cache.rs`：Gap-0b
  `strip_negative_svg_dims`（decode_svg_bytes 入口先剥）+ Gap-0
  `svg_intrinsic_kind` `vw > 0` 守卫 + 单测 ×4。
- `crates/layout-engine/src/tree.rs`：缺口1 约束方程 frame 减法
  （all-auto ratio 臂，margin_px helper：Auto=0/% 相对 CB/abs resolve）
  + 缺口2 no-ratio attr 臂守卫（width-only/height-only 对称，跳 R2172
  比注入 + 缺轴落 SVG_DEFAULT_W/H）+ 缺口3 L1（noratio img 固有维注入
  先取 no_ratio 真维）。
- `crates/webview/src/webview.rs`：host `__zwImgHasIntrinsicSignal` 回调
  注册（三表任一含键判据，口径同 `__zw_get_image_size`）。
- `crates/engine/src/js_dom_shim/part04.js`：img width/height IDL 渲染
  维臂（未渲染近似之后、naturalDim 回落之前；信号门 + gBCR − frame +
  非空像/非零盒判据 + 负值回落）。
- `crates/engine/tests/r5421_svg_ratio_frame_noratio_attr.rs`（新）：
  六面——ratio-only frame 减法（740CB+border/margin → bb 730×250）、
  无 frame 守卫（R2054 C2 原形逐字节不变）、no-ratio attr 臂缺轴
  default 轴 ×2（60×150 / 300×60）、ComputedIntrinsic 守卫不波及
  （60×20 比注入保留）、L1 no_ratio 真维胜 bogus pixmap（70×160）。

## 验证（昨夜 pre-rebase 轮 + 今日 rebased 复验轮）

- **单元/集成**：r5421 六面 6/6 Pass（今日 rebased 复跑）；
  render-foundation image_cache 122P/0F（含 s292 新钉 ×4）；
  concat node --check 18 片干净；cargo fmt 4 crate clean；
  `make guarded-clippy` workspace -D warnings 干净（30.79s）。
- **solo 三门**（今日 rebased B 侧）：naturalWidth **240P/18F（A）→
  246P/12F（B）**，B 复跑同 246P/12F——残 12F 恰 = srcset/1x+2x 变体
  （6 面 × 2 密度，s293 面边界精确）；density-vs-zoom **6P/3F**（残 3F
  = zoom 渲染流车道，intrinsic width only: baseline F→P）；
  relevant-mutations **126P/0F/0T** 哨兵原样（s291 基线）。
- **全树 A/B**（今日 rebased 对，双方 TIME_LIMIT=14400
  RAF_FRAME_DRIVEN=1，探针删除后语料）：**8419 键双侧同集（A-only/B-only
  全 0），翻转恰 7 枚全 F→P**（naturalWidth 6 + density-vs-zoom
  baseline 1），P2F=0/F2P=14 行级 7963P/402F/57T→**7970P/395F/57T**
  （ΔP=+7/ΔF=−7/ΔT=0）——**第 24 个连续完美 A/B**。
- **昨夜 pre-rebase 对**（A=s291 基线二进制，B=s291+WIP；8454 键含 35
  探针行）：翻转 14 枚全 F→P（7 靶面 + 7 探针面 gate/replica/replica2），
  行级 7973P/427F/57T→7987P/413F/57T 精确闭合。**昨夜 B vs 今日 B**
  join：35 行差恰 = zz-s292 探针删除行（非探针行 0），三 incoming 提交
  （editing-compat justify⑥ a04dd1522 / M3 dataset 682f0632d /
  rendering R5422 914d6e00b）**对 html/semantics 树零位移**。
- **make test**：昨夜 pre-rebase 78 套件 js_dom_bridge **3730P/6F**；
  今日 rebased 复跑 **79 套件 9127P/6F**（+1 套件/+18P = incoming M3
  dataset 等新增钉；js_dom_bridge 3739P/6F）——6F = 5 已知 CSSOM
  cssText 域红（p56/p99s/r2809/r2810/r3276，s288-s291 逐键同红）+
  **r3033 新红（跨流归因见下，非本片）**，本片零新增红。
- **local reftest**：`make reftest` **Layout 521/521 + Text 204/204 全
  100%** EXIT=0。
- **reftest-upstream**（17012 面）：no-WIP（今日 A）15955/1057、with-WIP
  （今日 B）15957/1055、with-WIP 复跑 15959/1053——**确定性修复 4
  面**：insert-inline-in-blocks-n-inlines-middle-001（8.42%→0.00%）、
  table-anonymous-objects-130（4.03%→0）、-149（1.89%→0）、
  intrinsic-percent-replaced-dynamic-008（6.25%→0）——no-WIP 红、
  with-WIP 三跑全绿零重叠；**intrinsic-ratio-replaced-box-sizing
  7.37%→4.01% 改善仍红**（>1% 阈值，渲染流车道）；**table-anonymous-
  objects-129/150 = dom-flag onload 变更时序 flake 族**——129 同 B 二
  进制跨跑 ✓(19:41 A)→✗(19:44 B)→✓(19:47 B 复跑) 自证 race，且昨夜
  with-WIP 双跑（18:32/18:35）双绿；150 同族（flags=dom onload style
  变更，catch 窗同构），昨夜双绿今日双红归同族时序面（非本片确定性位
  移——与本片无 img 消费的纯 table fixture 无机理通道）。
- **product-smoke**：DC-13 welcome PASS（struct-check 0 issues）；
  product-smoke-legacy struct-check failures 0（trend-only）。

## 跨流归因（rule 9/10 记档，本流不单方修）

- **r3033（test_htmlcollection_nodelist_item_nameditem_r3033）新红**：
  逐提交 bisect——s291 tip（86b3bbadb）绿 → **51d8a6949（editing-compat
  M3 documents 簇收口）起 FAIL**。断言面：HTMLCollection for...in 枚举
  keys 实测 `0,1,length,item,namedItem` vs 期望 `0,1`（length/item/
  namedItem 变 enumerable 泄漏；与 named-proxy `__zw` 印记门 /
  iframe body 合成两刀同期）。修复面属 editing-compat M3 documents 车
  道。**收口对账（本轮 push 后 pull 发现）**：对方流已自修——b13988f2a
  （test-only，期望对齐 M3 原型 enumerable 化契约）+ 5ad2d49cf 落账
  「挂账第 6 已知红关闭」——双方归因一致（同一引入提交 51d8a6949、
  同一断言面），r3033 在 main 已闭，无需移交通知。
- **replica3 探针 clientWidth 臂**：引擎 clientWidth 含 border（bb=310
  时 clientWidth 310，应 300）——probe 自设断言面非语料钉面，挂账渲染
  流车道（探针已删）。
- 并行负载注记：今日 A/B 窗与 ZeroWeb-3 clone 的 dom/html-syntax A/B
  腿并窗（test-guard 包裹互不侵扰），129/150 时序族判定已含负载窗归因
  （跨二进制/同二进制混合态自证）。

## 挂账更新

- **新挂账① s293 运行时表同步**：load 期外 img（srcset 换源/script 注
  入）装载时同步 cached_image_sizes/ratios/no_ratio 三表——解锁
  naturalWidth srcset/1x+2x 12F + srcset 克隆几何拉伸面（replica3
  srcset 臂勘根地面真值已留存本账本）。
- **新挂账② clientWidth 含 border 缺口**：移交渲染流（clientWidth
  语义 = content + padding，引擎现含 border；probe 噪声面非语料钉面）。
- s285 挂账（宿主 documentElement 物化缺口根修）维持；s221 picture 父
  IMG 臂 env 化维持独立评估。

## 下一步候选

1. **s293 运行时表同步**（本片直接后继——12F srcset 面 + 克隆几何面）。
2. picture 父 IMG 臂 env 化独立评估（s221 microtask 时序面）。
3. 宿主 documentElement 物化缺口根修（s285 挂账）。

## 证据

- 昨夜勘根/施工轮（pre-rebase，s291 基）：s292-replica*.log（探针四
  轮+l1fix 三轮+dump）、s292-nwh-solo{,-b,-c,-d}.log（solo 迭代四轮，
  终态 246P/12F）、s292-density-vs-zoom.log（6P/3F）、
  s292-imgfam-{a,b}.log（family 对 1066 键 14 F→P）、
  s292-fulltree-{a,b}.log（8454 键 14 F→P）、s292-make-test.log
  （3730P/6F）、s292-reftest.log（local 全绿）、
  s292-reftest-upstream{,-a}.log（with-WIP 双跑）。
- 今日 rebased 复验轮：s292-nwh-solo-postrebase.log（B 246P/12F）、
  s292-nwh-solo-a-side.log（A 240P/18F）、
  s292-density-vs-zoom-postrebase.log（6P/3F）、
  s292-relevant-mutations-postrebase.log（126P/0F）、
  s292-fulltree-{a,b}-postrebase.log（8419 键 7 F→P）、
  s292-reftest-upstream-{a,b}-side.log + -b-side2.log（三跑 1057/1055/
  1053）、s292-reftest-postrebase.log（local 521+204 全绿）、
  s292-product-smoke{,-legacy}.log、s292-make-test-postrebase.log
  （9127P/6F）。
- 日志 md5：nwh-postrebase 75d218ae、density-postrebase 83c69e6c、
  relmut-postrebase df4d4ce5、fulltree-a 6dfa3d15、fulltree-b e27589e7、
  upstream-a-side 190e3b72、upstream-b-side 28954b98、upstream-b-side2
  d1a1ab56、reftest bb22371b、smoke 4702fd71、smoke-legacy 57a8902b、
  make-test-postrebase 316e3bce、nwh-a-side ab24d351。
