# M2-S341：document.currentScript 动态臂栈化 + iframe body onload 收口——簇 3 案 1F+2T→5P/0F（承 s340 勘向）

日期：2026-11-08。控制文档：`docs/goal/html-semantics-compat.md`（L5 链 s341 段）。
本切片收口 s340 挂账的簇聚面勘向：`moving-between-documents-during-evaluation.html`
（F）+ `moving-between-documents/ordering/delay-load-event-{1,2}.html`（文件级 T）。
spec 依据：
- currentScript：script element stack 栈顶（每 Document；动态 classic 脚本执行期
  push 自身，evaluation 完结 pop——源文档栈不因元素被移走而中断）
  https://html.spec.whatwg.org/multipage/webappapis.html#dom-document-currentscript
- body onload：load 事件路径 window → document → body，事件处理器内容属性按
  路径编译执行
  https://html.spec.whatwg.org/multipage/webappapis.html#event-handler-content-attributes

## 定谳（本切片新增事实）

1. **动态内联 classic 脚本执行臂无 currentScript 入栈**（during-evaluation 根因
   A）：四个同步 eval 臂（R387 appendChild 陷阱 / s97ib insertBefore 镜像 /
   s95Post parse 后 text 置值 / s109 `_zwScriptMaybeStartDynamic` 内联臂）执行期
   均未把执行元素入栈——嵌套动态案内 `document.currentScript` 经 idx 通道回落
   **外层 parser script**（实测 probe2：修复前 `cs=outerScript, stackLen=0`）。
   失败链：内层 `[1]` assert 抛 TypeError → 体中止 → `window.anotherDocument`
   恒 null → 外层读 `null.currentScript` 二次 TypeError。真实执行臂是 **s109
   （part03）**，非 R387（probe2 判别：`ranHdl=true` 且 R387 臂 wrapper 不落）。
2. **implementation 文档缺 currentScript 属性**（根因 B）：`_makeDetachedDocument`
   工厂文档无该 getter——spec 期望 `null`（栈空），实测 `undefined`（`[2]`
   destination-Document 断言面）。
3. **iframe 子文档 body onload 内容属性从未编译执行**（delay-load 根因）：shim
   load 派发（part01 finish 臂）只 `win.dispatchEvent`（win listener 通道），
   `<body onload="parent.onloadIframe()">` 的内容属性编译执行面缺失——t_new
   永不完成 → 文件级 Timeout。
4. **body 视图 getAttribute 在 load 派发时点不可靠**（probe4a 定谳 `dbg=e:a=null`
   ——finish 臂可达、视图查 null）：R168 属性印章打在惰性 `_tree` 根，
   `_zwRunIframeScripts`（raw markup 通道）前 `_tree` 未建——视图链查不到解析
   属性。**raw markup `<body …>` 标签直提**（簇 23 body id 同型先例）为可靠源，
   工厂槽 `_r159BodyAttrs` 与视图 getAttribute 依次回落（probe4c：
   `raw=…:csp=true:k ~ child=true` 全链通）。
5. **probe 文件选案规则定谳**：runner 案面排除按 .html 源内容判（含 `<iframe`
   字面 / `createElement('iframe')` 即排除，testharness_html_semantics.rs 内容
   规则）——probe3（主页面含 `<iframe` 字面）不可入案；helper.js 造 iframe
   （真实案形态）可入案。fetch_dir_html 幂等快路径与选案无关（无 manifest 缓存）。

## 施工

| 面 | 位置 | 内容 |
|---|---|---|
| currentScript getter 栈优先 | part06（getter） | 先读 `document.__zwScriptElStack` 栈顶，空则回落 idx 通道（parser 面） |
| detached 文档 getter | part03 `_makeDetachedDocument` | `get currentScript()` 返回自身栈顶或 `null`（spec 栈空语义） |
| s109/s111 内联臂入栈 | part03（eval 包裹） | push `el` → `(0,eval).call` → finally pop；异常经 `_zwReportListenerError`（原语义保持） |
| R387 appendChild 臂入栈 | part04（陷阱臂） | push `child` → eval → finally pop |
| s97ib insertBefore 臂入栈 | part04（镜像臂） | push `newNode` → eval → finally pop |
| s95Post text 置值臂入栈 | part04 | push `_s95self` → eval → finally pop |
| iframe finish body onload | part01（finish 臂） | raw markup → 槽 → 视图三级取 `onload`；CSP `__zwCspAttrAllowed` 门（R152 同款）+ `with(document)/with(this)` 基 + `parent` 形参绑 `entry.win.parent` |

栈实现：`document.__zwScriptElStack` 数组（同 realm 单栈；parser 面走原 idx
通道不变，零回归）。R387/s97ib/s95Post/s109 四臂全部包栈——probe2 实证 R387
非实际执行臂，但三臂共享路径语义按 spec 同步补齐（后续形态切换执行臂时不缺）。

## 验证

- probe2（修复后重跑）：`cs=inner ~ stackLen=1 ~ evaluated=true`——栈通道生效，
  执行臂判别定谳（见定谳 1）。
- probe4a/probe4c（iframe body onload 链路）：a 臂 `dbg=e:a=null` 定谳视图链
  断点；c 臂 `raw=…:csp=true:k ~ child=true` 定谳全链通（探针验后删）。
- solo（FILTER=moving-between-documents，修复后 binary）：5 行全 P——
  during-evaluation 1P；delay-load-1/2 各 2P（t_original 原已 P，t_new 新 P）。
- 全域 A/B（key-state join，LC_ALL=C + sort -u 去重）：
  - the-script-element 子域：1070P/42F/11T/5NR → **1073P/41F/9T/5NR**
    （+3P/−1F/−2T，与簇目标逐案吻合）。
  - 全域（九子域）：8087P/328F/48T/11NR → **8090P/323F/47T/11NR**。
  - **唯一 both 侧状态翻转 = during-evaluation F→P（目标翻转），零 P→F 回归**。
  - 探针定谳的 flake：`customizable-select/select-home-end-pagedown-pageup-
    detailed.optional.html` 全域跑 4F→文件级 T（并行跑 page-timeout 边界）——
    同 binary solo 复现 4F 与基线一致（s341-solo-select-opt.log），非本片回归；
    全域 F −5 与 T −1 的差额即此 flake（−1F 簇 + −4F flake；−2T 簇 + +1T flake）。

## 挂账

1. **iframe 子文档脚本不设 currentScript**（预存缺口，本片未动）：`_zwRunIframeScripts`
   通道执行子文档脚本不走四同步臂栈——子文档内动态案 currentScript 语义未建
   （现无案面依赖，后续子文档脚本片勘）。
2. **select optional 并行跑 page-timeout 边界 flake**：全域 8 job 并行下偶发
   文件级 T（solo 稳定 4F）——记录在案，后续全域对比遇此键按 flake 归因。
3. **域长尾勘向（the-script-element 41F/9T）**：module/ 簇 ~20F（execorder/
   single-evaluation/evaluation-error/late-*-request 等）为主战场；9T 含 s340
   已挂 credentials setTimeout 变体、fetch-src/empty{,-with-base}、
   load-error-events-3、script-crossorigin-network.sub 等。
4. s340 既有挂账原样承继：主文档外链模块静态图 credentials（webview
   FIXME(s339)@5021）、s250 动态 src prepare credentials、window.onerror
   per-fire window.event。

## 证据日志

- `s341-probe-moving.log`（前 session）：during-evaluation 失败链探针——`[1]`
  assert 取值外层 script → TypeError 中止 → 外层 `null.currentScript` 定谳。
- `s341-probe2-path.log`：执行臂判别（`ranHdl=true`、R387 wrapper 不落 → s109
  臂）+ 栈修复实证（`cs=inner ~ stackLen=1`）。
- `s341-baseline.log`：修复前全域基线（8087P/328F/48T/11NR）。
- `s341-solo-mbd.log` / `s341-solo-mbd2.log` / `s341-solo-mbd3.log`：簇 solo
  三轮（body onload 视图链断点 → 槽源缺 → raw 源全通）。
- `s341-probe4-diag.log` / `s341-probe4b.log` / `s341-probe4c.log`：body onload
  链路探针（probe 验后删）。
- `s341-domain-after.log`：修复后全域（8090P/323F/47T/11NR）。
- `s341-solo-select-opt.log`：select optional flake 归因 solo（4F 同基线）。
