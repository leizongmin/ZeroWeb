# M2 切片 412 — 声明式 CSS module 双臂 CSP + 元素站 error 面（tentative/ CSP 簇 6 文件解挂）

日期：2026-11-12（承 s411 下一步：tentative/ constructed-stylesheets 机制勘根后按簇决策；429 断点接手——上轮已留全套施工 WIP 与 /tmp/s412 前两轮 solo 工件）
commit：见 git log s412 提交

## referent 与勘根

driving 面：`document-metadata/the-style-element/tentative/` 11 文件（declarative CSS modules 提案，Edge explainer，tentative——未入 HTML spec 正文的 `<style type=module specifier=...>` 声明式样式模块）。s408 口径目录 34P/0F/9T/6NotRun（9T = tentative/ 8 文件级 Timeout + constructed-stylesheets 1T，6NotRun = 3 个 allowed 文件 6 子测单列）。

本切片按簇拆分勘根定谳（上轮 solo-1/solo-2 双跑 + 本轮终稿 solo 三角验证）：

- **簇 1（CSP 面，本切片）**：6 文件 `style-element-csp-{blocked,both-blocked,nonce-script-only,nonce-style-only,script-src-blocked,style-src-blocked-script-src-allowed}.html`——案面 = `<style type=module specifier=foo>`（onerror 或 addEventListener 注册 error 监听）+ 各 CSP 政策组合，断言 securitypolicyviolation 派发 + **元素站 error 事件**派发 + 内容清空。红根两层：
  1. **门面单臂**：`gate_inline_styles` 对所有 inline style 只查 style-src（`check_style_hashes`）——module 具 module script + style 双重身份，`script-src 'nonce-X'` 政策下无 nonce module 应被 script 臂阻止（spec：声明式 CSS module 检查点同时过 style-src 与 script-src；普通 style 只查 style-src）。script-src-blocked 文件整案挂此（violation 不派 → 断言等待挂死 → 文件 Timeout）。
  2. **error 面缺失**：被阻止 module 无元素站 error 派发——blocked/nonce-script-only/nonce-style-only/style-src-blocked-script-src-allowed 四文件的 error 子测 pending 挂死（solo-1 各文件「1-2 test(s) unreported」即 error 子测）；普通 HTML spec 里 style 元素无 error 面，error 是 module 提案语义（explainer：阻断派 error 事件承载 module 加载失败）。
- **簇 2（import 面，余红单列候选）**：`style-element-{basic-import,fixed}.html`（2 file-T，loaded:false——`<script type=module> import sheet from "foo" with { type: "css" }` + module map + `document.adoptedStyleSheets` 应用 + fixed-at-parse-time 语义）+ 3 个 allowed 文件 6 NotRun（t1/t2 `.done()` 在 module script 内、import 不解析则永不执行）。底座已存在（computed_style_cache.rs adoptedStyleSheets 汇总下发 / callbacks.rs Constructable stylesheets shim），缺 module map 注册与 `with {type:"css"}` 绑定——独立大机制候选，不在本切片。

spec 锚：declarative CSS modules 提案（MSEdgeExplainers/ShadowDOM explainer，tentative）——`<style type=module specifier>` 双重身份：CSP 检查点 style-src + script-src 双臂（内容 raw 文本 sha256 与 inline script 同口径，hash 源含缘空白）；任一臂阻止 → 内容清空 + securitypolicyviolation（每臂各一条）+ 元素站 error 事件（module 语义，普通 style 阻断无 error 面——`style_error` 仅 `is_module && 首条目` 置位防双臂双派）；allowed 面（`unsafe-inline`/nonce 满足任一臂组合）零 violation 零 error。

## 施工

- **engine extract**（`pipeline/extract.rs`）：`StyleElementCsp` 增 `is_module` 判定——开标签 `type` 属性 trim 后 ASCII 大小写不敏感恰为 `module`（缺省/`text/css`/其他值 false；与 shim 侧 normalize 家族判字风格对齐）；CSP 检查点消费口径不变（全量 `<style>` 元素文档序）。
- **engine script_gen**（`js_dom_bridge/script_gen.rs`）：新增 `script_dispatch_style_event_by_ordinal`——按全量 `<style>` 元素文档序序号在元素站派发事件（镜像既有 `script_dispatch_script_event_by_ordinal` 形态）；run 尾统一派发点执行（listener 已注册）。
- **webview 门**（`webview.rs gate_inline_styles`）：module 增 script 臂（`check_script(nonce, Some(raw sha256), None)`）；双臂违规各入队一条 violation（同元素两条 = both-blocked 案「双 violation」断言面），`style_error: is_module && first_of_element` 只在首条目置位；block 判定改双臂任一即阻。
- **webview 派发**（`run_page_scripts` 尾 CSP 统一派发点）：`style_error && target_tag=="style"` 条目派元素站 error（ordinal 与 extract 同口径）；同元素只在首条目派一次。
- **常驻钉两件**：engine extract 单测 `extract_style_elements_csp_module_flag_s412`（module/text-css/缘空白大小写混合/缺省四案）+ webview `csp_style_gate_module_script_arm_blocks_and_fires_error_s412`（双腿：A `script-src nonce + style-src unsafe-inline`——script 臂阻止 + addEventListener error 面 + 恰一条 violation + 内容清空 + 普通 style 零位移；B `style-src 'none'` 无 script-src——style 臂阻止 + onerror attr 面 violation style-src-elem）。

## 过程陷阱

1. **429 断点接手（s411 陷阱 4 同款）**：上轮已留施工 WIP（4 文件 +186 行）与 /tmp/s412 tentative-solo-{1,2}.log 两轮证据，但 `style_error` 消费侧已写（run 尾派发臂在 webview.rs:5900 段）、证据未绑终稿——接手轮全部终稿重跑（电池/solo/sweep ×2 本轮均为终稿代码）。
2. **实现偏离自身文档意图（本轮修正）**：WIP `style_error: first_of_element` 对**普通** blocked style 也置位 error——extract.rs 注释自述「普通 style 无 error 面」；勘根确认 corpus 无任何「CSP-blocked 普通 style error」断言（style-error-01.html 是 @import fetch 面）后补 `is_module &&` 门，spec 保真且对 12 目标子测零影响（目标案 blocked 元素全是 type=module）。
3. **跨流 clippy 红（碰头信号单列不代修）**：`cargo clippy -D warnings` 死于 css-parser `values/random.rs:489 unused variable omitted`——渲染流 g20 批 8ed48256e 刚落文件，属渲染流域工作面（rule 9 不碰）。本切片以 `-A unused_variables` 定向复验本流两 crate 全绿（该 lint 是 strict 跑唯一 red），跨流红单列待渲染流自偿（g18/g19 同批自偿先例）。
4. **fmt 断点**：WIP 三处未格式化（extract struct 字段折行/import 排序/dispatch 调用形态），`cargo fmt --all` 机械清偿（仅涉本切片两文件，sweep 二进制已定型不受影响）。

## 验证（/tmp/s412/ 全量日志，终稿代码）

- driving solo（终稿 tentative-solo-final.log）：6 个 CSP 文件 **18 子测全 Pass**（含全部 error/violation 断言）；余红同 solo-2 形——basic-import/fixed 2 file-T（import 簇）+ 3 allowed 文件 6 NotRun（import 簇 `.done()` 不达）零位移。
- 单元：engine lib **3908P/7F**（+4P = 本切片 extract 钉 + 并行流合入 3 钉；失败清单 = s411 10F − 3：execcommand 2F 编辑流上游修复、t49 cssom 渲染流修复——favorable 位移归并行流合入域，零新增红）；webview lib **851P/2F**（+1P = 本切片双腿钉；image_decoder/csp_connect_gate 在册 2F 与 s411 同形零位移）。
- `cargo fmt --all -- --check` 净（WIP 三处机械清偿后）；`cargo clippy -p zero-engine -p zero-webview --all-targets -- -D warnings -A unused_variables` 净（css-parser 跨流红见陷阱 3）。
- 终局 runner 重编：freshness 门过（bin mtime > shim mtime，release 构建）；sweep 双跑（RAF_FRAME_DRIVEN，test-guard 包裹）**双跑全等 8487 keys 8256P/212F/19T + 11 NotRun**，join-cross **stable=8487 state-diff=0 new=0**（同二进制确定性对）。三 join（s292 协议，基线 /tmp/s411/pairs-b-*.txt = s411 rebase 后终稿批 212F+8247P+25T 双跑全等）：
  - join-b-1（对 s411 run1）：**恰 9 NEW = tentative/ CSP 6 文件 error/violation 子测 Pass + 恰 6 GONE = 同 6 文件 file 键 Timeout**（键形迁移 file-T→subtest-P），stable=8478 **state-diff=0**——纯 driving 位移，零其他变化。
  - join-b-2（对 s411 run2）：同 9/6 形，state-diff=0，GONE 侧零涉他键。
  - join-cross（run1×run2）：全等零位移。
  - （对 rebase 前第一批 /tmp/s411/pairs-{1,2}.txt 的 join 另存 join-{1,2}.txt：join-1 形同 + select-home-end Fail→Timeout watchdog flake 单列；join-2 另见 s411 run2 flake 形——select-event 3 password 子测/radio-focus file-T 基线批缺席、本轮齐——基线批内 flake 漂移单列不涉本切片。）
- `make test` 全量：**154 腿 153 绿 + 仅 engine lib 一腿 FAILED**（3908P/7F，失败清单与电池名册逐名相同 = 在册 7F；make 退出码非零由该在册红腿承接，同 s411 收口形态）。

## 口径

- 本流域在册红：**212F+25T → 212F+19T**（−6T = tentative/ CSP 簇 6 文件键形迁移 file-T→subtest-P，双跑全等；6NotRun 保持——3 个 allowed 文件属 import 簇未动，tentative/ 余 2T = basic-import/fixed 同属 import 簇单列候选）。
- engine lib 在册红承 s411 口径 10F → **7F**（−3 favorable 单列归并行流合入域：execcommand 2F 编辑流上游修复、t49 cssom 渲染流修复；零新增红）；webview lib 2F 零位移。

## 残差与边界

- **簇 2 import 面单列候选**：module map + `import ... with { type: "css" }` + `document.adoptedStyleSheets` 应用闭环 + fixed-at-parse-time 语义（basic-import/fixed 2 file-T + 3 allowed 文件 6 NotRun）。底座（adoptedStyleSheets 汇总下发、CSSStyleSheet shim）已存在，缺 module map 注册与绑定——勘根成本中等，s413 候选首位。
- **error 面仅 module**：普通 blocked style 派 violation 不派 error（提案语义）；若未来 HTML spec 正文化声明式 module，门面（`is_module` 判定）与派发面（`style_error`）两处同步。
- 双臂 sha256 口径：script 臂 hash 源为 raw 内容（含缘空白）——与 inline script 检查同口径；style 臂维持既有 trim 后 hash 面。
- css-parser random.rs unused-variable clippy 红承渲染流台账（本切片 strict 跑唯一 red，-A 单列复验）。

## 下一步

tentative/ import 簇（s413 候选）/ 挂账池承 s404/s405/s411（forward map 空返勘根/渲染流 5 面/form-submission-target 6T/sizes-auto 半边/ilf+modulepreload 观察池/适用态打满 reject 面/maxLength-minLength IDL 不适用态反射/unhandled-rejection 换算）；constructed-stylesheets 1T 与 import 簇同族可合并勘根；checkable 父 label 2F 渲染域移交承 s410；select/combobox（簇 D）与 A2+D 簇立项征询仍待用户决策。
