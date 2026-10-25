# M2-s89 施工轮：execution-timing stylesheets-blocking 12F 簇（四根四刀）

日期：2026-10-26 ｜ 轮次：s89（施工，s86 切片序②）｜ 前序：[s88](2026-10-26-m2s88-classic-dynamic-import.md)

## 1. 入场红图（当拍，HEAD=4fdaf03e5 + FILTER=the-script-element/execution-timing/）

目录 **48P/94F/8T**。s86 记「12F 单簇」经当拍细分：目标簇仅 12 行——
106-\* ×10 + 107-\* ×2 的「scheduler: stylesheets blocking \*」行；其余
82F 为编号套件预存红（s88 基线同态，非 s88 回归）。四根：

1. **语料根**：`fetch_dir_html` 扩展名表缺 `*.css`——`css/background.css`
   （`#test {position:fixed}`）与 `css/import.css`（`@import url("background.css")`
   **无分号**）从未入语料（pin 31597693 上游原文）。
2. **装填根**：`set_active_external_css` 生产恒空（此前唯一调用点是单测）——
   getComputedStyle 桥（callbacks.rs `active_external_css()`）读空 → 外链规则
   对 `__zw_get_computed_style` 不可见 → 106-\* 脚本读 `position` 得 `static`。
3. **@import 根**：进程内 `resolve_external_css` 无递归展开；且无分号
   `@import` 语句文本拼进合并级联后，css-syntax 的 at-rule 消费遇裸 `{` 吞块
   ——单文件内无害（语句后本就无规则），拼接后把**下一张表的首条规则**吞进
   at-rule 块（106-import 面实证：双表 fetch 序正确、探针仍 `static`）。
4. **107 根**：隐式 `document.write` 走 `body.innerHTML = 基线` 同步重放
   （part06.js append 语义）——原 `#test` 注册元素被 innerHTML plain 产物
   替换（无 `__zwSelector`）→ getComputedStyle host miss → `position` 不在
   UA 默认表返 `''`（真实 UA 初始值即 `static`）。

## 2. 本轮一刀（四文件四点）

- **`crates/webview/src/webview.rs`**：①`load_html` 把管线拿到的外链样式表
  文本同步发布 `zero_engine::js_dom_bridge::set_active_external_css`
  （css-less 页传空串复位，旧页规则不跨文档泄漏）——getComputedStyle 级联
  与布局级联同源；②`resolve_external_css` 加 @import 递归队列（对齐
  async_load `poll_stylesheets` R2411：相对**导入表 URL** 解析、`css_seen`
  防环、FIFO 保文档序；CSP/Mixed 门对顶层 link 臂照走）。
- **`crates/webview/src/css_urls.rs`**：新增 `strip_import_statements`
  （Tokenizer span 切除顶层 `@import` 语句——prelude 消费到 `;` 或 `{}` 块
  为止，对齐 css-syntax consume-at-rule；块内/注释/字符串内不动）；接线
  `resolve_external_css` 与 `async_load.rs` `poll_stylesheets` 两处拼接点
  （语句文本剥离后再入合并级联，子表内容经队列单独抓取）。FIXME：@import
  臂不走 CSP/Mixed Content 门（async_load 既有同态；violation 元素站对导入
  规则无 link 序号可锚）；级联顺序近似——子表追加在导入表文本之后
  （async_load 既有同态，逐位拼接归基建轮）。
- **`crates/engine/src/js_dom_shim/part01.js`**：getComputedStyle `position`
  初始值回落（css-position-3
  <https://drafts.csswg.org/css-position/#position-property> initial value
  `static`）——host miss 的 plain 元素（innerHTML/document.write 重放产物
  无 `__zwSelector`）真值即初始值；`getPropertyValue` 与 camelCase 两路径，
  同 R5009 float 初始值回落形状。
- **`tests/wpt-runner/scripts/goals/lib.sh`**：`fetch_dir_html` 扩展名表补
  `*.css`（语料可复现——幂等快路径只查 `*.html`，存量目录须 FORCE=1 或按
  本轮命令补拉）。

## 3. 验证

- **单测**：webview `execution_timing_css` 探针三钉——外链直入级联 `fixed`
  / @import 链展开 `fixed` / write 后写入 link 不反塑当拍 `static`；
  css_urls 新增 4 单测（无分号纯 @import 表 / 有分号保余下规则 / 无分号
  吞块形态 / 嵌套·字符串·大小写不敏感）。zero-webview **799/799**、
  zero-engine **3523/3523**（4 跑中第 2 跑 webview 出现 1 失败未复现，
  3 路全绿——预存 flake 挂账待查）；fmt clean；clippy `-D warnings`
  webview/engine/wpt-runner clean；**reftest 716/716**（target-disk-guard
  清缓存后全量重编，非旧产物空网）。
- **语料数果**（make 目标 + FILTER）：execution-timing 子树
  **48P/94F/8T → 62P/80F/8T**；stash 基线逐行 diff 恰 **14 行 Fail→Pass**
  （106-\* ×10 + 107-\* ×2 目标 + **150-\* ×2 同根红利**「Stylesheet in
  BODY (with @import) blocking scripts」）、**Pass→Fail 0 行**、T 不变。
- **the-script-element 全子树**：**158P/295F/33T**（s88 账面
  145P/308F/33T 含 v8-code-cache +1 惯例）——非目标 **144 行 Pass 逐位
  不动**（零误伤）；F 计数差 1 为 TLA 页脚抛截断点漂移的报表行数方差。
- **dynamic-import 金丝雀**：33P/23F/10T 与 stash 基线 bitwise 同态——
  对 s87 账面 28P/27F/11T 的 +5P−1T 漂移是环境方差（超时边界，双扫同态
  证），归因不记本刀。
- **语料补拉**（wpt-data 为 gitignore 本地数据，命令留档）：
  `curl https://cdn.jsdelivr.net/gh/web-platform-tests/wpt@<WPT_REV>/html/semantics/scripting-1/the-script-element/execution-timing/css/{background,import}.css`
  （raw.githubusercontent.com 直连超时，jsdelivr 首选与 lib.sh 同款）。

## 4. 挂账残面（本轮不动）

- **编号套件 82F**（execution-timing 001–060 族）：`?pipe=trickle(d2)` 等
  wptserve 时序 pipe 未建模 + 多文档/iframe 时序——后续切片。
- **@import 臂 CSP/Mixed 门缺口**：见 §2 FIXME（async_load 既有同态）。
- **级联逐位拼接**：@import 子表插入导入表 @import 位置（css-cascade
  文档序全保真）——基建轮。
- **webview 799 内预存 flake**：第 2 跑单失败未复现，名字未捕获（日志
  仅留 grep 行）——待复现挂账。

## 5. 范围与记账

- 生产面四 crate 五文件（webview ×3、engine shim、runner lib.sh）+
  契约单测；script-sandbox/layout/render 零改动；语料不可改维持（fetch
  脚本即交付物）。
- 共享面：js_dom_shim 归本流（engine）；css-parser **未动**（吞规则根因
  在拼接点而非解析器，webview 侧剥离回避跨流改写）。
