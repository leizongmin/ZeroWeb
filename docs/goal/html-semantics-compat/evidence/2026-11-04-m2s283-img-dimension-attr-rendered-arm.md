# M2 s283 — img dimension-attr rendered 臂双层修复（implicit-sizes-ignores-width 1F→P，全树 FIXED=1/REGRESSED=0）

日期：2026-11-04。切片号 s283（承 s282 下一步候选①——implicit width-attr
规范勘根 + 实施）。**两层修复**：shim `img.width/height` IDL getter
being-rendered 臂（part04.js）+ layout `apply_replaced_element_sizing`
img 免 attr 直写改固有注入（tree.rs 三臂）。

## 勘根

1. **spec letter-exact 双锚定**：①HTML §4.8.3 img IDL（embedded-content
   .html#dom-img-width，2026-09 快照原文 curl 取件）：「These attributes
   return the **rendered dimensions** of the image. When the image is not
   being rendered, they return the width and height content attributes'
   values, defaulting to naturalWidth and naturalHeight.」②rendering.html
   §15.4.3：「The width and height attributes on an img element's
   **dimension attribute source** map to the dimension properties…They
   similarly map to the aspect-ratio property」——hint 映射本身规范正确
   （img 无 picture 时 dimension attribute source = img 自身），**s282
   账本「style-system img 臂加 srcset/sizes 分流」假设否证销案**。
2. **corpus 面机制三层剥离**（探针页法，页自带 `img{width:auto}` + 断言
   `.width` IDL == innerWidth）：引擎给 400 的合成路径——①shim getter
   attr 可解析即直返属性值 400（渲染臂缺席）；②layout
   `apply_replaced_element_sizing` (Some(w),None) 臂 computed Auto 态
   无条件直写 definite attr 宽（**attr 二次入场**，压回作者覆盖值——
   inline `style="width:auto"` ((1,0,0) 最高 author 优先级) 探针仍 400
   定谳 H2=layout 绕过级联）；③auto 态既不写 attr 也不注入固有 → taffy
   容器宽 fill（body margin 8px 双侧 → 784 = 800−16 探针实证）。
3. **hint 通道独立撑面验证**：无 CSS 覆盖面探针（#plain attr img 无
   覆盖）gBCR/IDL 恒 400——style-system img 臂 hint（author origin
   (0,0,0) 最早位）承担全部无覆盖面的 attr sizing；`collect_
   presentational_hints` 标签臂仅 svg/img/table/tr/font/center/hr——
   **canvas/video/iframe 无 hint 通道**，layout attr 直写是其唯一通道
   → 修复必须门控 img 单标签。
4. **密度链闭合**：Rust layout 固有尺寸已密度折算（s219
   build_img_intrinsic_all → img_selected_pixel_density；srcset 100w
   无 sizes → 源尺寸回落 100vw=800 → 密度 0.125 → 固有 (800,800)）；
   shim `_` 臂 img_intrinsic_sizes 注入即无属性 img 的 used size 通道
   (#sizes 400 ✓)。attr-img 作者覆盖态应同通道。
5. **环境红灯归因（stash 法 HEAD 原态复跑）**：①`make browser-build`
   编译失败——rect_bridge.rs:280/291 引用 `js_dom_bridge::
   applied_mutations_cursor`/`MUT_DRAIN_GEN`（script-runtime feature
   门内 callbacks.rs）自身无门控，browser feature 组合态红；②make test
   5 失败全数 CSSOM cssText 序列化域（r2809/r2810/r3276/p99s/p56——
   尾分号/颜色序列化 red→rgb），HEAD 原态同红——**兄弟流在途工作所致，
   rule 9/10 记账不硬解**；两者均与本片改动域无交集（JS include_str!
   内容变更不影响 Rust 符号解析）。

## 施工（2 文件：shim 1 臂 + layout 3 臂门控）

- `crates/engine/src/js_dom_shim/part04.js` img width/height IDL
  getter：attr 可解析臂前置 **being-rendered 探针**——`_layoutRect`
  （与 gBCR 同源 layout-rect 快照）非空 → 返 rendered 维度（round）；
  无 rect（未布局/detached/display:none → host 无快照）→ 维持 attr 值
  = not-rendered 臂内容属性值（spec 半边）。IMG 单标签门（VIDEO/INPUT
  反射面不动）；FIXME 注记零盒（CSS width:0 真渲染零盒落 attr 臂）
  近似缺口。
- `crates/layout-engine/src/tree.rs` `apply_replaced_element_sizing`
  三臂 img 门：computed 侧 Auto（= hint 被作者覆盖态）免 attr 直写 →
  改注入 `img_intrinsic_sizes` 密度折算固有（与无属性 `_` 臂同通道；
  (Some,Some) 双 auto 注入双维 + stored_is_content_intrinsic 同步；
  无固有信号保持 auto）；**aspect-ratio 注入保留**（R2172 data-URI 比
  / attr 双值比 = attrs 只喂比例的规范语义半边）；canvas/embed/object
  等无 hint 通道维持 attr 直写零变化。规范锚 rendering.html
  #dimension-attributes。

## 验证

- **solo**：implicit-sizes-ignores-width F→P（s282 补资产后首报真面
  "expected 800 but got 400" → Pass）。
- **探针页三面对**（sizes/probe-s283-layout.html，验证后删除）：作者
  覆盖面 gBCR/clientWidth/IDL 三读 800 恰 innerWidth、sizes 面 400、
  无覆盖面 400（hint 通道零回归）。
- **family A/B**（FILTER=the-img-element，A = s282-imgfamily-b.log 同
  runner 同语料；日志 s283-imgfamily-b.log）：902P/46F/9T →
  904P/45F/9T，键级 **FIXED=1 恰靶 / REGRESSED=0 / T-changed=0**，
  GAINED=1 = 探针页键（已删，非 corpus）；zz-s272-probe2 双侧同 F
  （s272 留置探针，非回归面）。
- **clippy + fmt**：zero-layout-engine / zero-engine 双 crate
  --all-targets 零告警；fmt --check 干净。
- **make test**（test-guard 包裹全量）：**9085P/5F**——5F 全数 HEAD
  原态红（勘根⑤ stash 归因，CSSOM cssText 序列化域兄弟流在途），
  本片改动域（layout-engine/img 相关、js_dom_bridge img 面）全绿；
  QuickJS clippy 矩阵 + QUICKJS_TEST_CRATES（含 zero-wpt-runner）
  绿 = shim 编辑双运行时兼容。
- **全树 A/B**（A = s282-fulltree-b.log，同 runner；日志
  s283-fulltree-b.log）：7875P/408F/62T → 7876P/407F/62T（+1P/−1F
  恰好一对），键级 **FIXED=1 / REGRESSED=0 / T-changed=0 / LOST=0 /
  GAINED=0——第十七个连续完美**。

## 挂账更新

- **s282 真语义候选①销案**：style-system hint 分流假设规范否证；
  真根因双层（shim 渲染臂缺席 + layout attr 二次入场），本片收口。
- **新挂账：aspect-ratio from dimension attrs**（rendering.html
  §15.4.3「similarly map to the aspect-ratio property (using dimension
  rules)」）——style-system `aspect-ratio` 零命中（grep 实证），attr
  → CSS aspect-ratio 映射引擎缺席（tree.rs attr 比仅 taffy 侧局部，
  非 CSS 属性面）。
- **保持**：picture `source` child dimension attribute source（shim
  part03 `__zwUpdateSourceSet5277` FIXME 已注记，corpus 无观测面）；
  sizes=auto 机具切片（s282 候选② 三件套）；adoption/resize-observer
  （候选③）。
- **环境注记（兄弟流，不属本片）**：make browser-build 红灯
  （rect_bridge 未门控引用）；CSSOM cssText 5 红测（r2809/r2810/
  r3276/p99s/p56）——rule 9 碰头记账，待兄弟流收口。

## 下一步候选

1. **sizes=auto 机具**（~28 面 + 2 邻页，三件套：F5 简化挂起项解除
   + 渲染宽度源尺寸 + lazy script-created 通道）。
2. aspect-ratio dimension attrs 映射（新挂账，style-system +
   layout 跨切）。
3. adoption/resize-observer 真面勘根（机具面）。

## 证据

- `evidence/logs/s283-imgfamily-b.log`（family B；A =
  s282-imgfamily-b.log）
- `evidence/logs/s283-fulltree-b.log`（全树 B；A =
  s282-fulltree-b.log）
- `evidence/logs/s283-make-test.log`（全量 make test）
- 日志 md5：imgfamily df19362f…、fulltree a25db339…、make-test
  de3a1f3d…
