# M2 s322 — embedded 两散红勘根：svg-as-image `<style>` :root 根背景提升（✗2.08%→✓0.00% 入账本）；srcdoc-relative-uri 定性 iframe 内容文档渲染缺位（结构性移交）

日期：2026-11-07。切片号 s322（承 s321 下一步候选①：svg-img-with-external-stylesheet /
image-srcdoc-relative-uri-print 两散红勘根，embedded 4 收尾）。

## Solo 基线

`make reftest-upstream FILTER=the-img-element` + FILTER=the-iframe-element 回看：

- the-img-element/svg-img-with-external-stylesheet ✗2.08%
- the-iframe-element/image-srcdoc-relative-uri-print ✗1.04%

## 案 1：svg-img-with-external-stylesheet（本流实修）

### 勘根

页面：`<img src="support/external-sheet.svg">`；external-sheet.svg 内联
`<style xmlns="http://www.w3.org/1999/xhtml">:root { background-color: green }</style>`
+ `<link rel="stylesheet" href="red-bg.css">`（red-bg.css 有意缺失——测试前提即
SVG-as-image 安全静态模式不加载外部样式表，期望绿色根背景）。

usvg 0.47 完全忽略 `<style>` 样式表（s322 前已科）→ 绿根背景丢失，img 内白底。
本流已有 R3996 promote_svg_root_background（render-foundation image_cache.rs ~:1069）
处理根 background-color 的 **style attr / presentation attr** 两来源（剥声明 +
Pixmap::fill viewport 级填充）；`<style>` 内 `:root` 规则是同一机制缺失的第三来源。

### 施工（crates/render-foundation/src/image_cache.rs 单文件）

1. 新增 `svg_stylesheet_root_background(text)`：扫描内联 `<style>` 元素体，
   逐规则取选择器段（上一 `}` 或体首至 `{`）含 `:root` 者，声明块解析复用
   svg_style_background_color（background-color / background 简写均识别）；
   多条规则文档序后者胜（同等特异性级联）。仅内联样式生效——SVG-as-image
   安全静态模式不加载外部样式表（预处理通道只见源字节，外部表天然不适用，
   与测试前提一致）。
2. promote_svg_root_background 值来源级联化：inline style attr > 样式表规则 >
   presentation attr（CSS cascade——presentation attr 为 presentational hint，
   author origin 内最低；css-cascade-5 #cascade-sort / SVG2 styling）。
3. 改写链语义：样式表来源时标签本无需改写（usvg 不读样式表），但既有 else
   链落入的 attr 删除臂恰好正确——落败的 presentation attr 须删（usvg 会把它
   画成 viewBox rect，级联上已落败 chromium 不绘制）。无 attr 时删除为 no-op，
   源字节保持原样。
4. 单测 `promote_svg_root_background_from_stylesheet` 5 组：语料形态（绿 +
   源字节不动 + letterbox 区像素填充）/ inline style 压样式表 / 样式表压
   presentation attr（落败 attr 删除）/ 同体多规则后者胜 / 非 `:root` 选择器
   不提升。

### 验证

- 单测 2/2（新旧两测，dev-guard 包裹）。
- solo：svg-img-with-external-stylesheet **✗2.08% → ✓0.00% 真通过-可信**
  （DC-14），`make import-wpt` 入 imported-tests.txt 账本与修复同提交
  （语料本就在册，导入=账本资产化钉形）。
- 全量 A/B（vs s321 基线 15992P/1028F）：17020 案 **15996P/1024F**
  （净 +4P/−4F）。键级 comm（LC_ALL=C）：
  - FIXED 5 = 驱动案 + s321 已 solo×3 定谳的 4 时序抖动键
    （insert-inline-in-blocks-n-inlines-middle-001 + table-anonymous-objects-
    130/148/153，本轮翻回=镜像抖动）。
  - REGRESSED 1 = table-anonymous-objects-151.xht——**solo×3 全 PASS**
    （s292 协议同二进制跨跑状态翻转=时序抖动；R5418 table-anon 抖动族在册）。
  - **零确定性回归。**
- make reftest 常驻内联集 725/725 全绿。

## 案 2：image-srcdoc-relative-uri-print（结构性定性，零本流施工）

### 勘根（探针证据链，页完测即删）

页面：iframe `srcdoc` 内 `<img src="/images/green-100x50.png">`；ref = 主文档
img 同图 16px margin。✗1.04%。

- srcdoc 文本探针（zz-s322-srcdoc-text）"passed" 0.23% 但 layout-dump 证明
  iframe 盒空（文本从未渲染；0.23% ≈ 缺失文本行像素，次阈值带假绿，DC-14
  近似通过）。
- src= 对照探针（zz-s322-src-inline）0.24% 近似通过——与 srcdoc 同构。
- img 探针（srcdoc 内 img）1.04% 与基线一致；decoy 缓存探针证明图片键位
  错配非因（reftest 通道 build_image_cache 只扫主文档）。
- **定性：reftest 通道 iframe 内容文档渲染整体缺位**（src=/srcdoc= 均渲染为
  空 300×150 盒）。webview 通道已有 DOM 侧 iframe 异步物化
  （_zwFinishIframeEntry shim），像素合成缺位。
- 相对 URI 解析语义（/images/… → 语料根）在 testharness 通道已有等价实现
  （wpt_data_image_fetcher，testharness.rs ~:5444）；reftest 通道无对应——
  若后续立项 iframe 内容渲染，该 fetcher 语义可移植。

### 定性处置

结构性架构面（iframe 内容文档像素合成），非切片尺度施工——**勘根+挂账移交，
不施工**。移交面：渲染流（布局/绘制协作）+ 本流 webview 通道像素通道评估。
原案维持红（✗1.04%），与 embedded 4 其余 srcdoc/iframe 族同根（该族已在册）。

## 门禁

- cargo fmt --all -- --check 干净（首跑暴露测试内 src3 行超宽，rustfmt 修正）。
- cargo clippy --workspace --all-targets -D warnings（guarded-clippy）干净
  （首跑暴露 collapsible_if，if 合并）。
- make reftest 725/725 全绿。
- make test 全量：见控制文档状态行（本切片提交前完成）。

## 附：make test 首跑新红 wc_m1 勘根 + 静态哨兵改向（归因链实证）

首跑 make test 引擎腿 3786P/**10F**（已知 9F + 新红
wc_m1_factory_shadow_mutation_and_clone_template_content）。归因三步实证
（s292 协议 + stash 探针）：

1. solo ×3 全红（0.10s 断言失败）= 确定性红非时序抖动。
2. fe71fb01a 干净树（stash 本切片全部改动）复跑仍红 = 非本切片引入
   （本切片改动仅 render-foundation SVG 栅格化预处理，与 js_dom_bridge
   无因果链，实验证实）。
3. bisect：**6939ad5c8^ 绿 → 6939ad5c8 红**。新红由 6939ad5c8（formdata/
   submitter 簇 plain 世界 closest + 合成激活 click 坐标通道）引入——该提交
   落在 s321 make test 取证之后、fe71fb01a push 前 rebase 进本流基线
   （s321 账面「make test 9185P/9F 零新增红」对应 rebase 前树；归因纪律
   「main 是双流合成」的典型形态，本注记修正该时间差）。

断根：wc_m1 探针 `inShadow:` 用 `(n.s1.closest ? 'n/a' : 'ok')` 旧启发式
（「有 closest 方法=文档树节点」）；plain 世界 closest 落地后克隆/shadow
树 plain 元素亦有 closest → 恒 'n/a'。断言目标（slot 可达 + shadow 子树
挂接）仍有效，探针机制过期。

施工：part27.rs 探针改挂接结构断言 `n.s1.parentNode === n.host.shadowRoot`
（断言期望串不变 "s1:s1|inShadow:ok|shadowKids:1"；s294 静态哨兵期望改正
先例——语义演进后哨兵机制随迁，断言目标不动）。wc_m1 族 7/7 绿
（part27 全 wc_m1 前缀 7 测）。

同型启发式排查：part27.rs 其余测试无 closest 存在性探针，make test 无其他
新增红。

## 下一步候选

1. customizable-select ~30（forms tentative 高级面）评估立项价值。
2. （挂账承前）embed fallback-image（R57）、print pagination 7 键、
   range-intrinsic-size；counter scoping（reversed-1d/1e 次阈值绿根）待渲染流
   counter 面立项时移交。
3. iframe 内容文档渲染缺位（本切片案 2 定性）已移交渲染流/协作面，本流不再
   重复立项。
