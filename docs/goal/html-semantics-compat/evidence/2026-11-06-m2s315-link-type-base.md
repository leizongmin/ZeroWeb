# M2 s315 — the-link-element 簇 3 红页收口：link `type` 候选门 + `<base>` 基准前移 + 语料双缺件（7/7 全绿）

日期：2026-11-06。切片号 s315（承 s314 下一步候选①小簇排队，勘根→施工型切片）。

## 勘根/边界确认

- **簇红页形态**：the-link-element 7 reftest 案 = 3 红 + 4 绿（s161 谱系
  stylesheet-change/empty-href/media/remove-href 全绿维持）。
- **三红页三根因**（互不相同，两语料一引擎两处引擎面）：
  1. link-rel-attribute-ascii-case-insensitive（mismatch ✗0.00%）——**纯语料缺件**：
     页引用 `stylesheet.css`（上游 36B 红底表）本地缺，link1/link3 双 404 → test 与
     notref 同白底恒同绘。引擎 rel token 匹配**既有** ASCII 不敏感
     （extract.rs `eq_ignore_ascii_case`——`StyLeShEeT` 折叠、`ſtyleſheet` U+017F 非
     ASCII 不折叠，语义正确），零引擎改动。
  2. link-type-attribute（match ✗2.08% 恰 div 面积）——**type 门缺口**：全 5 link 带
     非法/非 CSS type（application/javascript / ABCtext/css / text/cssDEF /
     text/invalid / invalid），正确语义全不加载 → 绿 div ≡ ref 绿 div；我们无视
     type 加载 data:text/css 红 !important 表 → 红 div ≠ ref。runner 侧 R5364 已有
     data:text/css 装载（资源半在，type 门半缺）。
  3. stylesheet-with-base（match ✗100.00% 全像素差）——**缺件 + base 基准双缺口**：
     `<base href="resources/">` + `href="stylesheet.css"` 应解析
     resources/stylesheet.css（上游 38B 绿底表，本地缺件）；三条装载路径（runner
     merge/webview 同步/async_load 渐进）均只按文档目录/URL join，不认 `<base>`。
- **回归面圈定**：语料 28 页带 `<base href>` 中仅 2 页有 stylesheet link
  （with-base 已修 / empty-href 空 href 不受影响）；15 页带 type 属性中 14 页全为
  合法 `text/css`（门后行为不变）。
- **上游资产核实**（GitHub API contents + raw 内容逐字节）：top-level
  stylesheet.css = `body{background-color:red}`、resources/stylesheet.css =
  `body{background-color:green}`，与两页设计语义吻合。

## 施工

1. **语料补件**（s304/s311 先例，上游 sparse/raw 原件）：
   - html/semantics/document-metadata/the-link-element/stylesheet.css（36B，红底）
   - html/semantics/document-metadata/the-link-element/resources/stylesheet.css
     （38B，绿底）
2. **engine extract.rs 共享谓词 + helper**（engine 单侧，三消费方共判）：
   - `link_type_allows_stylesheet(Option<&str>)`：type 缺失/空串 = 无提示候选；非空
     须规范化（trim → `;` 前段 → ASCII 小写）== text/css（HTML §the-link-element
     candidate stylesheet type 门；chromium 同）。
   - `extract_document_base_href(html)`：文档序首个带 href 的 base 元素 raw value
     （HTML §document-base-url；空 href 回落 None；既有
     `script_effective_base_urls` 同款语义的样式表面）。
   - type 门接入 `extract_stylesheet_links` / `extract_stylesheet_link_anchors`
     （hrefs 变体经前者复用）。
3. **三装载路径 `<base>` 前移**：
   - runner resources.rs：`load_linked_stylesheets` + `merge_page_css_document_order`
     （后者循环自有 rel/href 门补 type 谓词——触发门①走 extract 已过滤，混合
     data:/file: link 页循环臂同判防漏）基准 `Path::join` 前移。
   - webview.rs `resolve_external_css` + async_load.rs `begin_stylesheet_fetch`：
     `Url::join` 前移（产品路径与 harness 对齐——engine-direct reftest ≡ WebView
     像素契约）。
4. 单测 3 案：engine lib 2 案（type 门 11 形态矩阵；type 门页形提取（5 拒 2 纳）+
   base 提取 4 态）+ webview coverage 1 案（`<base href="/sub/">` + 相对 href 仅经
   base join 命中 `/sub/style.css`——产品 fetch 路径 base 前移钉住，语料
   stylesheet-with-base 走 runner 路径盖不到该面）。

## 语义边界（记录）

- `<base>` 前移仅样式表面：脚本（既有 `script_effective_base_urls`）、图片/字体
  资源提取面既有行为不动（语料无驱动页，FIXME 不设占位）。
- type 规范化为「`;` 前段 + ASCII 小写」最小形态：非 RFC MIME 全解析器（畸形参数
  段如 `text/css;=x` 不细究，语料无消费）；空串 type = 无提示候选（spec 同）。
- webview `resolve_external_css` 的 @import 臂不前移（导入表相对自身 URL 解析，
  spec 语义不同，R2411 既有口径维持）。
- perf-gate 豁免：extract 谓词为 link 元素逐元素分支（非样式/布局热路径）；base
  helper 仅在装载期一次遍历，无渲染热路径成本。承 s311-s314 先例。

## 验证

- 簇 solo reftest：**the-link-element 7/7 全绿**——ascii-case ✗0.00% → **✓99.98%**
  （test 白 vs notref 红，mismatch 可分）、link-type-attribute ✗2.08% → **✓0.00%**
  （type 门全拒，绿 div ≡ ref）、stylesheet-with-base ✗100.00% → **✓0.00%**
  （缺件+base 双修复，绿 body ≡ ref）；4 绿页零回归。日志
  [logs/s315-link-after.log](logs/s315-link-after.log)（修前基线
  logs/s315-link-initial.log：4P/3F）。
- 单测：engine lib s315 2 案 + webview coverage base 前移 1 案全绿（fetch_url
  家族 6 案同跑全绿）。
- clippy（engine+webview）：零告警。
- 产品 smoke（webview 装载路径变更面）：make product-smoke-legacy struct 0 +
  make product-smoke PASS（fallback 修正后复跑，[logs/s315-smoke2.log](logs/s315-smoke2.log)）。
- **make test 首跑抓到自伤回归（已修）**：webview_coverage
  `test_fetch_url_loads_external_stylesheet` FAILED（"no pure-red fill found
  (got 0 fills)"，日志 [logs/s315-make-test.log](logs/s315-make-test.log)）。
  归因探针：干净树 stash-probe **PASS**（1P/0F）→ 非跨流，是本切片 diff 自伤。
  根因：webview 两侧 base 前移用 `Option::zip`——页**无** `<base>` 时
  `zip(Some(url), None)` 返回 None 把文档基准整个清空 → 样式链接 join 基准丢失
  → 装载跳过。solo reftest 未抓到：reftest 走 runner resources.rs（该侧
  `.unwrap_or_else` 保底写法正确），webview 面只有 coverage 测试盖。修正为
  match 语义——页声明 base 且 join 成功才覆盖，否则维持文档 URL 基准（与
  runner 侧保底同构）；修正后 fetch_url 家族全绿。教训：改 Option 语义时
  zip 是「双双在场」不是「回退保持」，跨路径同语义改动要逐路径跑各自测试层
  （reftest 层绿 ≠ webview 层绿）。
- make test（guarded-test，后台全量）终态 **80 腿 ok + engine lib 3783P/8F**
  （8F=已知 7 键 + 跨流 fetch 键，与 s314 已知集完全一致——总 3781P→3783P
  全为本切片新案，零新失败；webview coverage 腿 fetch_url 家族 6 案全绿）。
  日志 [logs/s315-make-test2.log](logs/s315-make-test2.log)。

## 证据

日志（docs/goal/html-semantics-compat/evidence/logs/）：
- s315-link-initial.log（修前 4P/3F 基线）
- s315-link-after.log（7/7 全绿终态）
- s315-legacy-smoke.log / s315-smoke.log / s315-smoke2.log（回归门禁；smoke2 为
  fallback 修正后复跑）
- s315-make-test.log（首跑：71 legs ok + webview 1F 自伤回归证据）
- s315-make-test2.log（修正后全量终态）

语料锚：wpt-data/html/semantics/document-metadata/the-link-element/{stylesheet.css,
resources/stylesheet.css}（update-wpt-data.sh 挂账同提交）。
代码锚：crates/engine/src/pipeline/extract.rs（共享谓词 + base helper + type 门）；
tests/wpt-runner/src/reftest/resources.rs（双装载路径 base 前移 + type 门）；
crates/webview/src/{webview.rs,async_load.rs}（产品路径 base 前移）。

注记：本切片 diff 落 engine/webview/wpt-runner 5 文件 + 语料 2 补件；layout/
style-system/render-foundation 零触碰。

## 下一步候选

1. 小簇排队其余（embed 3/object 2/option 1/iframe 1/wbr 1）逐簇勘根；br 1 勘根前
   先 solo 复核 editing-compat br 刀（fe760dc57）外溢现势。
2. （挂账承前）range-intrinsic-size 跨流移交跟踪；shaped 值行 ellipsis 接线；
   生产动画重绘驱动等跨流协同项。
