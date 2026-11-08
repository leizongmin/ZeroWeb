# M2 切片 351：css-module relative-urls 双案——跨行 with 子句续行 + css 模块响应体 url() 响应锚定

**日期**: 2026-11-09
**目标对**: `css-module/relative-urls.html` + `css-module/redirect-relative-urls.html`（s350 基线 17F 中双 F）
**结果**: 双案 F→P + redirect 静态臂新增 P（A 面缺席非退面）；域 join det F→P=2 / 新增 P=1 / P→F=∅ / T=8 同集零位移；语料口径 **17F→15F**

## 上游语义（勘根）

spec [fetch a single module script](https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script)：css 模块（`with { type: "css" }`）响应体合成 CSSStyleSheet，**相对 url() 以样式表 base URL（= fetch response URL，重定向面 = 终跳目标）在解析时定锚**（[CSS Values §3.2](https://drafts.csswg.org/css-values-4/#urls)），computed 值 = 绝对 URL。`redirect-relative-urls.html` 静态 import 书写形 specifier 与 `with { type: "css" }` **跨行**（import-attributes 声明文法允许子句跨行，换行不终止声明）。redirect.py 语义：module map 按**请求 URL** 记账（redirect.html 判据「redirecting URL 与 target 是 distinct entries」），而样式表定锚用**响应 URL**——两 URL 分离是测试核心。

## 旧红与根因

| 面 | 旧红形态 | 根因 |
|---|---|---|
| relative-urls | `assert_not_equals: got disallowed value -1`（indexOf 无 `css-module/resources/image.png`） | 引擎 **url() 解析面整体缺失**：css parser 无 base URL 上下文，`BackgroundImageValue::Url(String)` 逐字存储 raw `./image.png`，computed 序列化 `url("./image.png")`（computed_style.rs `image_layer_list_to_css` 逐字回放） |
| redirect 静态臂 | **子测缺席**（静态 import 链断，test() 未注册；动态臂单独 Fail） | `split_statements` 顶层换行切句守卫只认 `from` 收尾（s116）：specifier 行尾切句 → `with { type: "css" };` 独立成句 → 提取丢属性面（`type_attr=None`，`__zw_mtype` 标记缺失 → .py 无 intent 臂接管即 Err）+ transform 发射孤儿 `with { … };` 残句（非合法 JS 语句形，首修切片曾使页面 tests:0 Timeout——transform 与提取必须同吃本扫描，s116 注记原话） |
| redirect 动态臂 | `expected "url(\\"https://wpt.test/…/resources/image.png\\")" but got "url(\\"./image.png\\")"` | 同 url() 解析面缺失（动态臂属性面完好，唯一缺口 = 定锚） |

## 施工（双面）

1. **跨行 with 子句续行**（script-sandbox `es_module.rs`，splitter 单面）：
   - `split_statements` 换行切句守卫扩 `statement_continues_with_attributes`：import/export 开头（整词）+ specifier 字面量收尾（`'`/`"`）+ 跨空白前瞻 `with` 整词 + 非空白 `{` → 不切句（attributes 子句是声明文法一部分）。`with (obj)` legacy 语句形不续行；后随普通语句（`foo()`）不吞句。
   - 提取/transform 同吃本扫描（s116 同层）：`suffix_after_specifier_literal` 取 from 子句首字面量后缀 → `parse_with_clause` 直接解析行内子句——提取器零改动。语料普查 `^\s*with {` 全树唯一站点 = redirect-relative-urls.html。
2. **css 模块响应体 url() 响应锚定**（runner 车道 `testharness.rs`，engine 零改动）：
   - `__zw_mtype=css` intent 臂交付文本经 `rewrite_relative_css_urls_s351(text, https://wpt.test/{file_rel})`：`url(<payload>)` 三引号形（单/双/裸）识别，payload 相对引用（无 scheme、非 `/` `#` 开头）→ `join_css_url_ref` 逐段归一 `.`/`..` 重写为 `url("绝对")`；绝对/片段/data: 逐字放行。锚 = 交付文件路径（重定向面 = 终跳 hop_clean——**响应 URL 通道**：fetcher 契约 `(page_url, src)` 无响应 URL 形参（s349 同形限制），重写即把响应 URL 应用到响应体的通道；module map 请求 URL 记账不触（redirect.html distinct-entries 判据保持）。
   - engine 全量 url() 解析面（parser base URL 参数 + cascade 穿线）不属本切片——挂账。
   - 单测 `css_module_relative_urls_resolved_against_response_url_s351`（静态/重定向双 fetch 面 + helper 裸形/`..`/绝对放行三面）+ `test_attributes_extractor_multiline_with_clause_s351`（红→绿：跨行形 (spec, css) 对；legacy `with (obj)` 不吞句）。

## Blast radius 普查

- splitter 守卫三重形状先验：单行 import 在 `;` 先切（守卫不触）；无分号 import 后随普通语句前瞻非 `with {` 不续行。语料 `^\s*with {` 唯一站点 = 目标页。
- 重写仅 mtype=css 臂：html/semantics 全语料含 `url(` 的 css = 目标双 fixture（css/CSS2、css/css-fonts 的 url( 走 `<link>`/`@import` 车道不触本臂）。
- style-element collateral A/B（stash 前后双跑 FILTER=style-element）：43 键逐位相同（1F+9T 既有长尾零位移）。

## 验证

- **solo 双文件 3 键全 P**（`FILTER=relative-urls`，exit 0；evidence/logs/s351-solo-pair.log）。红态证据：修前 solo relative-urls `assert_not_equals -1`、redirect 动态臂 raw url 断言失配 + 静态臂缺席（/tmp 会话日志）；splitter 首修暴露 transform 孤儿残句 → 页面 tests:0 Timeout → 迁移 splitter 层后 3 键全 P。
- **域 join**（LC_ALL=C + `(key,state)` sort -u + 双态键 0）：A=s350-b **17F/1104P/8T** vs B=s351-b **15F/1107P/8T**——det F→P=2 恰双目标逐键吻合；新增 P=1（redirect 静态臂，A 面缺席）；new-F=∅；removed-F=∅；T=8 同集双向 ∅。
- **collateral**：style-element 族 pre/post 43 键逐位相同（git stash A/B 双跑）。
- **单测**：script-sandbox `--lib` **420P/0F**（+1 新测）；runner `--bins` **303P/0F**（+1 新测）。
- **clippy** zero-script-sandbox/zero-wpt-runner `--all-targets -D warnings` 0 告警；`cargo fmt --all -- --check` 净。

## 挂账 / 下一步

- 引擎全量 CSS url() 解析面（css parser base URL 上下文 + computed 值绝对化；`<style>`/`<link>`/adopted combined-text 通道 per-sheet URL 盲区一并勘根）——本切片仅车道建模。
- fetcher 契约响应 URL 通道（真实 HTTP 车道重定向后样式表定锚；s349 Accept 头建模同族挂账）。
- 域残余 15F 逐案勘根（execution-timing 6 案长尾、import.defer 三案 JS 语言特性面独立切片、余散案）；T=8 长尾在册。
- testharness.rs 与并行 M3 流（SW fetcher）同文件不同区——pull--rebase 顺序合流。

## 日志

- evidence/logs/s351-solo-pair.log（solo 双文件 3 键）
- evidence/logs/s351-b-script-element.log（域跑 B 面）
- s350 基线 A 面：evidence/logs/s350-b-script-element.log（既有）
