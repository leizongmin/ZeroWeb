# DC-13 产品静态 smoke — morning.work 中文文章 fixture

> **R373 突破（2026-06-20）**：morning article `z_vs_chr`（800×600）= **28.72% → 16.41%（-12.3pp 真 chromium 改善）**。根因：morning 主残余是 3 个 `.item-tag` `<span>`（`display:inline` + background-color + padding 的标签徽章）被 ZeroWeb 映射为 Block 拉到满宽色条（~55k px）。修复 `shrink_inline_blocks_to_content` 扩 `display:inline && background_color != Transparent` 也按 intrinsic 内容宽收缩（R368 只覆盖 InlineBlock，未覆盖 inline；R372 复测时误判为 plateau，实为未试机制）。元素仍 block 堆叠（完整 inline-box 模型属 Phase A），但 width 收缩后蓝色面积大幅减少 → 真 chr 一致率提升 12.3pp。附带 reftest `clear-inline-001` FAIL→PASS（z_vs_chr 3.61→1.26%），self-source 442→443/490 零回归。article-zeroweb-cpu.png 已更新为 R373 后渲染。

> **R374 残余根因（2026-06-20，post-R373 16.41% 像素带分析）**：残余 diff 分散在多 y 带（y560-600 43.8% / y320-360 36.0% / y360-400 25.8%），非单点布局 bug。带色分析：① y560-600 chromium 有蓝色 `.item-tag` 徽章而 ZeroWeb 是背景 → **`.item-tag` 在 ZW 与 CHR 处不同 y**（上游文本高度差级联致位置偏移）；② y320-360 是代码块文本（`#3d4752`），ZW 渲染量少于 CHR（换行差异）。**根因 = 文本换行/行高级联**：fontdue glyph advance-width 与 Skia 不同 → 换行点不同 → 行数/高度不同 → 下游元素（.item-tag、代码块）y 位置偏移。属 **Phase A layout+paint shaping 统一**（R225/R320/R331 证 advance-width paint-only 改 net-negative，须 layout+paint 同源；estimate_char_width 用于 layout 断行，fontdue 用于 paint，二者与 chromium 实际度量均偏差）。line-height 计算本身正确（1.5×16=24px），cascade 来自 wrapping 非 line-height。**勿再以 morning 残余为单会话杠杆**——须 Phase A shaping 统一（多会话）。

> **R372 复测（2026-06-20，post-R355~R368，800×600 视口，R373 前）**：~~morning article `z_vs_chr` = 28.72%~~（已被 R373 的 16.41% 取代）。R372 时误判 plateau，因未识别 inline+bg 是未试机制。

**日期**: 2026-06-16
**源页面**: `https://morning.work/page/2026-02/fedora-macbook-three-finger-drag.html`（在 Fedora 上为 MacBook 实现 macOS 风格的三指拖拽）
**fixture**: `apps/browser/assets/morning-work/`（article.html + 4 个外链 CSS + 2 张图片）
**渲染模式**: ZeroWeb CPU 软件渲染（800×600，base_dir 加载外链 CSS/图片）vs headless Chromium（800×600，file://）

## 录制资源

| 资源 | 来源 | 状态 |
|------|------|------|
| article.html | 原页面，`<link>`/`<img>` 改相对路径 | ✅ |
| article.css | `/article.css` | ✅ |
| github.css | `/styles/github.css`（hljs 代码高亮基础样式） | ✅ |
| fira_code.css | `/FiraCode/fira_code.css`（@font-face，字体文件未下载，回退 monospace） | ⚠️ 字体回退 |
| JetBrainsMono.css | `/JetBrainsMono/JetBrainsMono.css`（同上） | ⚠️ 字体回退 |
| images/logo_lei.jpg | `/images/logo_lei.jpg` | ✅ |
| images/qrcode_*.jpg | `/images/qrcode_for_gh_*.jpg` | ✅ |
| cc.png（知识共享徽章） | 远程 `i.creativecommons.org` | ❌ curl 返回 162 字节（疑似重定向/拦截），已替换为占位，**记录为不可用资源** |

## 差距演化

| 阶段 | 像素 diff（vs Chromium） |
|------|--------------------------|
| 初始（R174 后） | **67.45%**（323,772 px）— 页面背景/代码块背景全白、布局塌 |
| R175 var() 继承修复后 | **28.72%**（137,874 px）— 页面背景 #f9f7f4 与代码块背景正确应用 |

## R175 根因：CSS 自定义属性未继承

morning.work 用 `:root { --color-bg / --color-code-bg / --color-primary ... }` 定义设计 token，元素通过 `var(--color-bg)` 引用。**ZeroWeb 自定义属性不继承**——`gather_custom_properties`（style-system/src/lib.rs）每元素只取自身级联的自定义属性，丢弃祖先（`:root`/`html`/`body`）定义的变量 → 后代 `var()` 解析失败 → 背景回退默认白/颜色丢失。

诊断探针（已删除，转 style-system 单测 `test_custom_property_inheritance`）证实：`--c` 定义在 `.a` 自身时 `var(--c)` 正确，但定义在 `:root`/`html`/`body` 祖先时解析失败（白）。

**修复**：`gather_custom_properties(cascaded, inherited)` 先继承父元素自定义属性，再用当前元素自身声明覆盖，再迭代解析值中 var()（可引用继承属性）；`compute_styles_recursive` 递归传递 `parent_custom`（自定义属性是继承属性）。

## 剩余 28.72% 差距

- **顶部蓝色全宽条（~55k px，#607cd2，y=169-241）**：经定位是 3 个 `.item-tag` `<span>`（Fedora/MacBook/Linux 标签徽章，直接规则非 @media）被**渲染为全宽堆叠块**而非行内小徽章。`ua_default_display("span")=None`（inline 正确），故问题在布局层——ZeroWeb 把 inline/inline-block 元素当作 block（属 R109 IFC/inline→block 架构范畴，非单会话修复）。**下一轮独立诊断目标**。
- fontdue vs Skia 字体度量噪声（CJK 文本行高/字宽差异）。
- 代码块无语法高亮（highlight.js 需 JS 运行时，sandbox 不完整）。
- @font-face web 字体未加载（fira_code/JetBrainsMono 回退 monospace）。

## 证据文件

- `article-chromium.png` — Chromium 参考截图
- `article-zeroweb-cpu.png` — ZeroWeb CPU 渲染（R175 后 28.72%）

## 方法

1. `node /tmp/capture-mw.mjs`（puppeteer，800×600）→ chromium shot。
2. `cargo test -p zero-wpt-runner dump_morning_work_png -- --ignored`（render_to_framebuffer_with_base + base_dir 加载外链 CSS/图片）→ ZeroWeb shot。
3. PIL 逐像素 diff + 区域/颜色分析。
