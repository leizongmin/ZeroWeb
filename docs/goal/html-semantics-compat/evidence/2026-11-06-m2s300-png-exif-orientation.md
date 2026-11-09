# M2 s300 — PNG eXIf 方向解码收口（reftest exif-png 三页 F→P；早块生效/晚块忽略 #url-metadata 语义；eXIf+ImageMagick Raw profile 双源——后者语料钉面；png 0.18 升级；s299 clippy 遗留修复）

日期：2026-11-06。切片号 s300（承 s299 下一步候选①，s296 JPEG EXIF 片的
PNG 侧姊妹刀）。勘根+施工+reftest A/B 由 429 中断轮完成，本 session 复验
收口（单测/clippy/solo 定谳/make test/落账）。3 文件 +4 fixture
（render-foundation Cargo.toml+image_cache.rs、wpt-runner reftest/resources.rs
+本 session clippy 遗留修复），新增依赖版本 png 0.17→0.18（workspace 根与
engine 已在 0.18.1，双版本共存态承袭，其余 0.17 钉面 crate 不动）。

## 勘根

**语料契约**（css/css-images/image-orientation/image-orientation-exif-png{,-2,-3}
.html 三页，reftest 通道——s296 JPEG 面为 testharness 通道，两通道分离）：
三页同契约钉 css-images-3 **#url-metadata** late-metadata 语义——early EXIF
（IDAT 前）生效、late EXIF（IDAT 后）忽略；exif-png/-2 用 `<img>`、-3 用
CSS background-image（解码层重排即 CSS from-image 默认值语义，与 s296
naturalWidth 双臂同型）。ref 页期望 early 图旋转 + late 图原样。

**资产三源实证**（语料 PNG 块结构逐枚 dump）：①F-exif-chunk-early.png
原生 eXIf 块在 IDAT 前；②F-exif-late.png 原生 eXIf 块在 IDAT 后；③
**F-exif-early.png 无原生 eXIf**——zTXt「Raw profile type exif」文本档案
（`\nexif\n    4160\n<hex>`，hex 解码 =「Exif\0\0」+TIFF 小端，JPEG APP1
同构；长度行带前导空格）承载方向——ImageMagick profile.c 约定格式，
Chromium/Firefox 实际读取面。双源缺一即 ③ 页臂红（语料直接钉死双源面，
非投机加固）。

**解码漏斗**：`decode_png_bytes`（render-foundation/image_cache.rs）产品
通道唯一入口（同 s296 单漏斗定性）；reftest harness `load_png_file`
（wpt-runner/reftest/resources.rs）为独立解码路径（R217 ZERO_PNG_EXPAND
逃生舱面，像素解码不委托）——两路径分别接线，方向探测共享
`png_orientation`/`apply_orientation`（同表同通道）。

**爆炸半径全盘点**（语料 1781 PNG 块级扫描 + 13 zTXt 档案 TIFF 逐枚解析）：
early eXIf 全语料恰 1 枚（target）、late eXIf 恰 1 枚（target）、
Raw-profile-exif zTXt 共 13 枚——12 枚 css-backgrounds/css-transforms 普通
支撑图 orientation 全部 =1 或缺失（零行为面，与 A/B 实证吻合）+1 枚
target（F-exif-early）。产品/测试 fixture（六目录树）PNG 零
eXIf/tEXt/zTXt（product-smoke 两门豁免实证）。

**png 0.17→0.18 动因**：exif_metadata（0.18 起自动填充 eXIf 块，
stream.rs parse_exif 无 opt-in 门）+ Info 生命周期 API 变更。0.18 破坏面
两处已适配：Decoder::new 要求 BufRead+Seek（字节切片经 Cursor）、
output_buffer_size 返回 Option<usize>。wpt-runner 本地 png 钉 0.17 不动
（Decoder::new(&bytes[..]) 0.17 Read+Seek 兼容），仅方向探测跨 crate 复用。

## 施工（2 crate 3 文件 + 4 fixture）

- render-foundation/image_cache.rs：`decode_png_bytes` 方向在 **read_info
  边界快照**（早块生效）——IDAT 后的 eXIf/文本块在 next_frame 期才入
  Info，快照点天然实现「晚块忽略」（#url-metadata 语义）；RGBA 转换后
  `apply_orientation` 重排换轴（与 JPEG 路径同表）。`png_orientation(bytes)`
  pub 探测（eXIf 优先→tEXt/zTXt「Raw profile type exif」次之；strip
  「Exif\0\0」前缀防御）、`png_orientation_from_info`（快照语义归调用方，
  不 pub）、`imagemagick_exif_raw_profile`（split_once 锚 + trim 长度行 +
  hex 过滤 take(2·len) + 长度吻合校验，截断档案按无 EXIF）、
  `apply_orientation` 改 pub（harness 复用）；decode_jpeg_bytes 的 PNG
  FIXME 销案。
- wpt-runner/reftest/resources.rs：load_png_file 改 bytes+Cursor 读入
  （0.17 Decoder 兼容），EXPAND/非 EXPAND 两臂各按探测结果重排换轴
  （ImageKey raw 键面零变化）。
- fixture 四枚（crates/render-foundation/src/testdata/，109–185 字节构造
  图，红|蓝 2×1 非对称可观测）：exif_early（IHDR→eXIf→IDAT→IEND）/
  exif_late（IHDR→IDAT→eXIf→IEND）/exif_ztxt/exif_text（IHDR→zTXt|tEXt→
  IDAT→IEND）。
- **s299 clippy 遗留修复**（本 session，extract_base_href_prefix 两处
  question_mark lint——HEAD a4a86c463 stash 复验即报，本片门禁拦截；旧
  toolchain 未拦或未跑不追）：let-else→`?`、else-if 链→else 臂 `?`，
  语义恒等，30P harness 单测保绿。

## 验证

- **单测**（guarded 定向 test-guard --compile-first 4G/8G/900s）：
  render-foundation decode_tests **43/43**——新增 2 组：四 fixture 维度+
  像素断言（early/ztxt/text 1×2 上红下蓝、late 2×1 原样）+ png_orientation
  探测面（早块双源 Some(6)、晚块 None、普通 PNG None）；harness
  reftest::resources **30/30**（含 clippy 重写保绿）。
- **fmt/clippy**：`cargo fmt --all -- --check` clean；clippy
  render-foundation+wpt-runner --all-targets -D warnings clean（修复 s299
  遗留 2 lint 后；修复前 HEAD 同报 2 lint 实证非本片引入）。
- **solo 目标面**：image-orientation-exif-png 三页 **3P/0F 全 ✓0.00%**
  （A 侧三页红）。
- **全树 reftest A/B**（A=15971P/1041F，B=15975P/1037F，17012 键双侧同集）：
  翻转 6 F→P / 2 P→F——真面恰 **3 枚 target F→P**；其余 5 键
  （insert-block-in-blocks-n-inlines-middle-001、table-anonymous-objects
  -148/-149/-153、currentcolor-border-repaint-parent）**solo 三连（终态
  二进制 3×5 全 ✓0.00%）定谳时序 flake**（s292 同二进制状态翻转先例 +
  s299 tao 127/150 同族常设注记），净零。**真实 ΔP=+3，P2F=0**。
- **全树复跑 B2**（终态二进制含 clippy 修复，15975P/1037F 总量与 B 恒）：
  B→B2 差 4 键全落 flake 池——tao-153/currentcolor **回翻 F→P**（A→B 曾
  P→F，同池双向漂移铁证）、insert-block-002/border-collapse-dynamic
  -column-003 新入池（P→F）；B2 vs A = 3 target F→P + 5 flake 键，**两轮
  B 侧 target 三页全稳 P**。flake 池扩注：insert-block 兄弟对与
  border-collapse-dynamic-column-003 并入 s299 常设注记族。
- **make test**：**81 线 9155P/5F** 对 s296（81 线 9144P/5F，81 线结构
  s294-s296 三轮同形承继）——5F **精确同键**（p56/p99s/r2809/r2810/r3276
  已知 CSSOM cssText 域红）零新增红；ΔP=+11 为并行流累计非本片（本片
  Rust 面=render-foundation+wpt-runner，见上文定向守卫单测与 harness
  单测）。结构注记承 s296：workspace 腿 engine lib 套件已知 5F 失败即
  中止该腿，render-foundation lib 套件腿中后位未执行（s294-s296-s300
  四轮同形，跨轮对比有效）。
- **豁免面实证**：bench-gate——render-foundation 无解码 bench 面（仅
  render_bench），EXIF 探测为 read_info 元数据扫描 O(块数)、恒等图零像素
  工作；product-smoke(-legacy)——六 fixture 目录树 PNG 零元数据块（解码
  输出逐字节恒等）；oracle/其余门无解码维消费面。

## 下一步候选

1. css image-orientation 属性渲染面移交 rendering-compat 候选（s296/s299
   账承继——语料 image-orientation 目录 reftest 面已全绿收编后，剩余属性
   语义面归渲染域）。
2. s295 候选维持挂账：plain-src 运行时 img 触发缺口（s293① 已销案后无
   余钉形）；宿主 documentElement 物化残面两形（s298 常设注记，语料钉形
   再立候选）。

## 证据

- 日志（evidence/logs/，gitignored 本地留存 + md5）：s300-reftest-solo
  -exifpng.log（3P/0F）、s300-reftest-full-a.log（A 侧 15971P）、
  s300-reftest-full-b.log（B 侧 15975P）、s300-reftest-full-b2.log（终态
  二进制复跑 15975P）、s300-solo-adjud.log（5 键×3 全绿）、
  s300-make-test.log（81 线 9155P/5F）。md5：solo 66ad793c、full-a
  7385484d、full-b df36ff20、full-b2 75a2ea47、adjud 7ef4e4c3、make-test
  0d52c09f。单测/fmt/clippy 摘要在本账未落盘（s296 同例）。
- 代码锚：render-foundation/image_cache.rs decode_png_bytes :1427 起 +
  png_orientation/png_orientation_from_info/imagemagick_exif_raw_profile；
  wpt-runner/reftest/resources.rs load_png_file :1236 起 +
  extract_base_href_prefix clippy 修复 :1078。
- fixture 块结构/dump 与 13 zTXt 档案解析：本账勘根节（python 结构扫描
  未落盘，结论可由 png 快照工具复现）。
- s296（JPEG 姊妹片，apply_orientation 同表）/s299（clippy 遗留来源 +
  A/B 协议先例）账本承前。
