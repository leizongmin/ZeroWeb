# M2 s296 — EXIF 方向解码缺口根修（natural-size-orientation 1P/2F→3P/0F；解码层单漏斗 Orientation 物理重排换轴；全树 A/B 翻转恰 2 枚全 F→P 第 28 个连续完美）

日期：2026-11-06。切片号 s296（承 s295 下一步候选①：natural-size-orientation
2F 挂账面——zero-media 解码器特性片，勘根后定性实际落点为 render-foundation
图片解码单漏斗，非音视频 media crate）。1 文件 +145−1 行（含 4 组单测），
零 shim 零其他 crate。

## 勘根

**语料契约**（the-img-element/natural-size-orientation.html 三子测）：①无
EXIF PNG green-100x50 → 100/50（现绿）；②EXIF JPEG arrow-oriented-upright.jpg
→ 期望 144/240；③同 JPEG 挂 `image-orientation:none` 类 → **同期望 144/240**
——naturalWidth/Height 语义与 CSS none 渲染值解耦，纯解码层重排即双臂同修。
资产实证：APP1 EXIF IFD0 tag 0x0112 **Orientation=6**（SOF0 存储维
240w×144h，EXIF 2.32 §4.6.4「0th row = visual right side」→ 顺时针 90° →
显示 144×240，与断言精确吻合）。

**解码漏斗单点**：`decode_image_bytes`（crates/render-foundation/src/
image_cache.rs）是全部消费方唯一解码入口——webview/src/image_decoder.rs 为
纯委托、webview.rs 两直呼点同走该口；修复一次全覆盖。naturalWidth 流出点
webview.rs:1779 `image_natural_sizes.insert(key_hash, (img.size()…))` 直读
ImageData 尺寸——解码层换轴自动传播至 shim IDL 与布局/绘制全通道，零下游改
动。既往实现 jpeg-decoder 0.3 无 EXIF API、文件内零 orientation 处理。

**爆炸半径盘点**（两通道分离实证）：全语料树非恒等 EXIF JPEG 共 11 枚
（css/css-images/image-orientation/support/ 10 + images/arrow-oriented-*
upright.jpg 1）；引用 EXIF/oriented 资产的语料页共 102 页，**reftest-manifest
逐页核对全部不在清单**（reftest 通道零波及）；testharness 通道消费面 4 页
（natural-size-orientation + css/css-images 计算样式/插值 3 页——后三者断言
computed style 不触解码维）。预期全树恰 2 枚 F→P、P2F=0，实证由 A/B 判定。

**性能面**（guidelines #23）：render-foundation 无解码 bench 面（仅
render_bench）；EXIF 扫描为解码前 O(段数) 有界行走（SOI→SOS 头段，病态段
即退），恒等图（现实绝大多数）零像素工作；非恒等图一次 O(n) 重排。账面记
录，无 bench 门可测对象。

## 施工（1 文件：crates/render-foundation/src/image_cache.rs）

- `jpeg_exif_orientation(bytes)`：JPEG 头段遍历（TEM/SOI/RSTn 无长度段跳
  过；段长含自身 2 字节、恒大端；`bytes.len()-pos < seg_len+2` 病态即
  None——外部输入不 panic），APP1「Exif\0\0」命中后委托 TIFF 解析，SOS 停
  止。只扫 IFD0 首个命中项（Orientation 定义在 IFD0，Exif 子 IFD 镜像不在
  读取面）。
- `parse_tiff_orientation(tiff)`：双字节序（MM/II）；SHORT(3)/LONG(4) 值
  内联条目值域；其余类型按缺失；**值域 1..=8 之外返回 None**（「unknown」9
  按无方向处理——浏览器同型）。全 get/checked_add 界内读。
- `apply_orientation(data,w,h,o)`：8 方向映射与 image crate
  `apply_orientation`/libvips 同表——1 恒等 / 2 水平翻转 / 3 旋转 180°（双
  翻转叠加）/ 4 垂直翻转 / 5 主对角转置 / 6 顺时针 90° / 7 反对角转置 /
  8 逆时针 90°；5-8 宽高互换。转置族目标 (列 x, 行 y) 逐方向源索引公式
  （5: x·w+y / 6: (h−1−x)·w+y / 7: (h−1−x)·w+(w−1−y) / 8: x·w+(w−1−y)），
  usize 索引、split_at_mut 非重叠交换。越界值防御性恒等（解析侧已过滤）。
- `decode_jpeg_bytes` 接线：RGBA 转换后按 EXIF 重排换轴；`// FIXME:` PNG
  eXIf 块方向未处理（png crate exif_metadata 面；html/semantics 语料消费面
  仅 JPEG）；CSS from-image 默认值 + HTML naturalWidth 规范链接。

**本轮迭代修正三件**（单测互证闭环）：①E0499 双重借用→split_at_mut；
②apply_orientation 首版 match `_` 臂把 5-8 拦成恒等（orientation 5 dims
断言抓到）——重构为越界前置门 + 1 显式 return + 5-8 落入转置循环；③测试
构造器 APP1 段长按 TIFF 字节序写（LE 组解析 None）——JPEG 段长恒大端，段
长改恒 to_be_bytes（字节序只作用于 TIFF 载荷）。

## 验证

- **单测**（guarded 定向：test-guard --compile-first，per-proc 4G/total
  8G/900s）：**126 passed / 0 failed**——新增 4 组：解析双字节序 × SHORT/
  LONG 全组合命中、无效面（无 EXIF 真实 fixture/unknown 9/字节序残损/段长
  截断全 None）、8 方向手算期望网格（源 A B C / D E F 行主序逐方向维度+
  像素序 + 越界 0/9 恒等）、e2e 真实 green_4x3.jpg 拼接 APP1（6 → 4×3→3×4
  像素总量守恒；1 → 维与像素逐字节同无 EXIF 解码）。
- **fmt/clippy**：`cargo fmt --all -- --check` clean；guarded-clippy
  zero-render-foundation --all-targets -D warnings 干净。
- **solo 目标面**：natural-size-orientation **3P/0F**（1P/2F → 3P，文件级
  红全消）。
- **目录级 A/B**（A = s295-fulltree-b.log the-img-element 行 1031 键；B =
  s296-imgelem-dir-b.log）：**shared 1031 键，翻转恰 2 枚全 F→P**（两 EXIF
  子测），**P2F=0，LOST=0，GAINED=0**——恰靶零涟漪。
- **全树 A/B**（A = s295-fulltree-b.log；B = s296-fulltree-b.log，双方
  TIME_LIMIT=14400 RAF_FRAME_DRIVEN）：**8419 键双侧同集，翻转恰 2 枚全
  F→P**（natural-size-orientation 两 EXIF 子测），**P2F=0**，LOST/GAINED
  双零，mixed-state 双零；行级 7990P/379F/53T → 7992P/377F/53T（ΔP=+2=
  翻转、ΔF=−2、ΔT=0；双侧各 3 键同态双行，统一解析器键级状态集对比）——
  **第 28 个连续完美**。css 计算样式 3 页零位移（解码维不触断言面，预期内）。
- **make test**：**81 线 9144P/5F** 对 s295（81 线 9137P/5F）——5F 与基线
  **精确同键**（p56/p99s/r2809/r2810/r3276 已知 CSSOM cssText 域红）零新增
  红；Δ+7P 测试名集合差分逐名归因：engine r5427 paint 溢出 ×5（
  ffaaf2e3a）+ declarative-shadow parser ×2（9fa8924f7）——均并行流前序提
  交非本片 diff（本片 Rust 改动全在 render-foundation，见下注记）。
  **结构注记**：workspace 腿 cargo test 顺序推进、engine lib 套件（已知
  5F）失败即中止该腿——render-foundation lib 套件位于腿中后位未执行，
  s294/s295/s296 三轮同形（81 线数稳定），跨轮对比有效；本片 Rust 证据 =
  上文定向守卫单测。render-foundation lib 套件结构性失明与 slice28 renderer
  lib 专项腿同族，挂账碰头账候选不在本片处理。
- **豁免面实证**（quality-gates 按影响选择）：reftest/product-smoke/
  product-smoke-legacy/bench-gate 全豁免——非语料 JPEG 全盘点（morning-work
  logo/qrcode fixture + testdata green_4x3）零 EXIF（解码输出对恒等图逐字
  节不变，单测 e2e 臂实证）；102 个 EXIF 引用语料页全部不在 reftest
  manifest；decode 无 bench 面。渲染/布局产品路径无改动（解码维仅对
  EXIF 图换轴，产品 fixture 无此形态）。

## 下一步候选

1. s295 候选②③④维持挂账：picture 父 IMG 臂 env 化独立评估（s221）/宿主
   documentElement 物化缺口根修（s285+）/plain-src 运行时 img 触发缺口根修
   （s293，需先立钉面）。
2. PNG eXIf 方向标签支持（png crate exif_metadata 面——无语料消费面，
  FIXME 已注记，低优先）。
3. css image-orientation 属性渲染面（rendering-compat 域移交候选——语料
   css/css-images/image-orientation/ 目录全部不在本通道 reftest manifest，
   无本域消费面）。

## 证据

- 本轮日志（evidence/logs/，gitignored 本地留存 + md5 指纹）：
  s296-natural-size-solo.log（3P/0F 终态）、s296-imgelem-dir-b.log（目录
  B 侧）、s296-fulltree-b.log（全树 B 侧）、s296-make-test.log（81 线
  9144P/5F）。
- 日志 md5：solo a0464a1a、dir 8b07361b、fulltree 672a8501、make-test
  e8cc9487。单测定向跑与 fmt/clippy 输出未落盘（126/126 摘要在本账）。
- 代码：crates/render-foundation/src/image_cache.rs +145−1（勘根基点
  decode_jpeg_bytes :1448 起）。
