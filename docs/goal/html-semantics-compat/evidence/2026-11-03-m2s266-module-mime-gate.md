# M2 s266 — module 外链 MIME 门（runner 预投递面按 spec 网络错误投递）

日期：2026-11-03。切片号 s266（承 s265 下一步候选 #1）。1 刀 + 1 钉
（runner 车道），script-element MIME 三页 3F→3P。

## 勘根

1. **规范面**：fetch a single module script——响应 MIME 非
   JavaScript 为**网络错误**：error 事件 + 体不执行（HTML
   webappapis）；classic 外链**无 MIME 门**（nosniff 之外）。
2. **死亡代码链定位**：webview 侧 `ScriptSourceFetcher`
   （webview.rs:146）与 runner 侧 `wpt_data_script_fetcher` 均有
   scripty 门（pipe→`.headers` sidecar→扩展名 .js/.mjs），但
   runner 模式下页面外链脚本全部被 `inline_local_scripts`
   （testharness.rs:10385）**预投递**——末臂读盘在册即
   `(combined, 200, body)` 200 投递 + load 尾，**无 MIME 检查**
   ：fetcher 的门对页面脚本单元是死代码。module 单元投递尾按
   status fire（200→load、非 200→error），体错误异步吞掉——
   即误投 200 时页面看到的是 load。
3. **探针实证**（gitignored 本地探针页，验证后已删）：A 臂
   module .js→load 正常；B 臂 module `.css`→**误 fire load**
   （spec 违例本体）；C 臂 classic `.css`→两事件皆无（SyntaxError
   体杀掉 classic 单元共元素投递尾——classic 无 MIME 门是 spec
   正确，仅投递形状注记）；D 臂 `script[src]` 计数 0——全标签
   被改写，确认面在预投递层。
4. **翻转面全树扫描**（python os.walk + 正则，DOTALL 跨行标签，
   2026-11-03）：module 外链非 js 扩展名 src 共 25 处——**3 处
   读盘在册**（css-module/script-element-css-src.html + basic.css
   、json-module/script-element-json-src.html + module.json、
   text-module/script-element-json-src.html + file.txt，三页判据
   全是 `assert_array_equals(log, ["error"])` + unreached load）
   ；22 处全为缺席 `.py` 端点单元，`.ok()` None 整段透传 fetcher
   端点臂（s250 语义不变）。**翻转面恰为 3 页，全部 F**——
   零回归面。

## 施工（1 文件 1 臂 + 1 钉）

1. **末臂 MIME 门**（testharness.rs inline_local_scripts 末臂）
   ：`is_module_type` 时派生 scripty——① `?pipe=header(
   Content-Type,X)` ② `<file>.headers` sidecar（Content-Type 行
   ）③ 扩展名 .js/.mjs 兜底；essence 去参小写后匹配
   text/javascript、application/javascript、text/ecmascript、
   application/ecmascript。非 scripty → `(combined, 404, "")`
   （error 投递尾 + 体不入页 = 网络错误语义）；classic 恒
   scripty（无 MIME 门）；读盘缺席整段透传不变。注释附规范
   链接与派生序同构说明。
2. **钉** `inline_local_scripts_module_non_scripty_src_error_
   delivery_s266`：module .css→error 尾 + 体不入页（精确投递
   形状断言）；module .js→load 尾 + 体入页；classic .css→体
   照旧入页（无 MIME 门）。

## 验证

- **红态实证**（HEAD + 钉，test-guard）：dump 显示 module .css
  单元走 `new Event('load')`——与探针 B 臂一致。
- **绿态**（test-guard）：inline_local_scripts 组 3/3（含新钉
  ，fmt 后复跑确认）；`cargo test -p zero-wpt-runner --bin
  zero-wpt-runner` 全量 **283 passed + 0 failed + 76 ignored**
  （默认 4G per-proc 帽被 runtime_path_tests 正当触发，按质量
  门禁升至 `--per-proc-mem 8 --total-mem 16`）。
- **fmt/clippy**：`cargo fmt --all --check` clean（仅本切文件
  入 diff）；clippy zero-wpt-runner `--all-targets` 零告警。
- **solo 复跑**（make testharness-html-semantics，test-guard
  包裹）：**3/3 F→P**——css-module/script-element-css-src、
  json-module/script-element-json-src、text-module/
  script-element-json-src（日志 evidence/logs/
  s266-script-element-mime-gate.log）。
- **全树 A/B 免跑判据**：DOTALL 重扫（首轮正则 `[^>]*` 不跨
  行可能漏多行标签，重扫补严）翻转面 25 处 = 3 在册 3F（本切
  收口）+ 22 缺席 .py 透传不变 → **零其他 face**，引 s258
  先例免跑（face 唯一 + solo 行为实证 + 套件绿态兜底）。

## 挂账更新

- module MIME 三页销案（本切收口）；classic 外链无 MIME 门为
  spec 正确（探针 C 的"两事件皆无"系 SyntaxError 体杀投递尾
  的投递形状注记，零 corpus face，不扩面）。
- 承前不动：多张 map merge + 动态插入（s262 架构级缓议挂起待
  拍板）；microtask checkpoint 时序族（s255）；credentials/
  shared-dependency/并发慢 import dedup（s256）；customizable
  -select 177F（GB-20261031 挂起）；renderer SetDomSnapshot
  entries 化（s174）；engine lib 5 红 R-t2qq（该流自查，本流
  不入侵）；s265 新记声明块直读路径未预处（零 corpus face）。
- css-module relative-urls / redirect-relative-urls 3F（CSS
  url() 基面，渲染流车道交接）；repeated-import ×3（
  serve-custom-response.py 端点缺口）；referrer-policies.sub
  （referrer-checker.py + `{{domains[www1]}}` 替换）；NotRun
  微面（json `;boundary` ×3、css `;params` ×2）。

## 下一步候选

1. repeated-import ×3 serve-custom-response.py 端点补齐
   （runner 车道，端点面先勘根缺什么响应形状）。
2. referrer-policies.sub 端点 + `{{domains[www1]}}` 替换面
   （runner 车道）。
3. css-module relative-urls CSS url() 基面→渲染流车道交接
   注记（本流不动渲染）。
