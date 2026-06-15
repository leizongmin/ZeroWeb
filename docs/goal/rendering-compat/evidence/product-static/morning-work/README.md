# DC-13 产品静态 smoke — morning.work 中文文章 fixture

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
