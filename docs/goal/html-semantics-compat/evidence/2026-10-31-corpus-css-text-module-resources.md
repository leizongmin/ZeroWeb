# 语料资产轮：css/text 数据模块资源补拉——待决①销账（用户授权 2026-10-31）

**性质**: 语料资产切片（非施工切片，零源码改动）——M1 语料预置脚本补拉 + A/B 定谳。
**授权**: 用户 2026-10-31 会话授权「待决① css/text 数据模块语料资源缺失，授权即解除」（补语料属基准变更，s108 记）。

## 目标

- s108（2026-10-27）定谳待决①：css-module/text-module 引用面资源不在本地语料。本轮执行销账。

## 勘根（缺口机制）

- M1 首拉（2026-10-15）时 fetch_dir_html 扩展白名单尚无 .css（2026-10-29 才加），
  fetch_dir_html 的 .txt 为 opt-in（GOAL_PULL_TXT，net-api 先例）。
- 此后 css-module/ 已含 .html → 幂等快路径整目录跳过，resources/ 列目录不再执行
  ——缺口固化（s158 补 link-rel-attribute.css 同坑）。
- text-module/file.txt 位于用例目录本体，非白名单扩展 → 从未拉取。

## 施工（tests/wpt-runner/scripts/goals/25-html-semantics-compat.sh ③b 段）

- 8 文件显式 fetch_raw（R4300 跨目录助资产同构）：
  - css-module/resources/：basic.css、basic-large.css（40KB，jsdelivr 403 走 raw
    回落成功）、bad-import.css、malformed.css、parse-error.css、
    load-relative-url.css、atImported.css（bad-import @import 传递引用，扫描新增）
  - text-module/：file.txt（10B）
- not_found.css / not_found.txt 为负向用例，保持缺席不拉。
- utf-8.css / utf-8.txt 仅经 serve-with-content-type.py?fn= 引用——runner 未实现
  该端点（FIXME 新记），端点落地时再补。

## 验证（A/B stash 法：同二进制 s170-era runner 09:35 构建，test-guard 包裹 +
ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1，stash 基线跑直接调 runner 绕过 make fetch 依赖回填）

- css-module：基线 29 行 vs 补后 29 行——28 稳定（16F/7P/5T/4NR 逐行同态）+
  唯一位移 **1 案 P→F**：script-element-css-src「script src=.css 应发 error」——
  基线缺资源 404 → error 事件凑巧符合预期（双向一致错误）；补后文件在，
  runner classic script 装载器对非 .js src 静默丢弃不发事件 → 真实缺口揭穿（记账）。
- text-module：基线 16 行 vs 补后 16 行——15 稳定 + 唯一位移 **1 案 P→F**：
  script-element-json-src（src="./file.txt"，同机制同族）。
- **预期面未翻绿（资产为前置非充分）**：integrity matches、content-type matches
  （text/css 行）、charset ×3——blocked 在更深的 runner 执行面：css/text module
  type 支持、serve-with-content-type.py 端点、redirect.py module map 语义——
  s107「css/text 数据模块执行面另切片」记账延续，本轮资产是该面切片的前置条件。
- A/B 日志：/tmp/zw-corpus-cssmodule-{0946,TRUEBASE}.log、
  /tmp/zw-corpus-textmodule-{WITH,BASE}.log。

## 范围

- tests/wpt-runner/scripts/goals/25-html-semantics-compat.sh（③b 段）
- wpt-data 本地语料 +8 文件（gitignore 不随提交）
- 本账本 + goal 文档状态块

## 记账

- **销账**：待决① css/text 数据模块语料资源缺失（用户授权 2026-10-31）。
  「待决三项」余 green-2x2.png（s126 已补齐，余分发面按 fetch 脚本幂等引导）、
  parser form pointer（2026-10-23 已决策缓行）——两项建议下轮控制面改记「已决策」。
- **FIXME 新记**：runner classic script 装载器对非 .js src 静默丢弃不发 error
  事件（2 案 A/B 定谳）；serve-with-content-type.py 端点未实现（css/text module
  content-type/charset/utf-8 面阻塞点）。
- **基准变更注记**：下轮全量 join 对 css-module/text-module/script src 非 JS 面
  的对应位移（含 2 案 P→F）按语料变更归因，非回归。
