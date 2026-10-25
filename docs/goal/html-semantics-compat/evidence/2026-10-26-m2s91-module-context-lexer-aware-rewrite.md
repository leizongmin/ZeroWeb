# M2-s91：动态 import() 重写器词法上下文感知 + module 上下文面收口（string-compilation 族清盘）

日期：2026-10-26。归属：html-semantics-compat / s86 切片序③ the-script-element 基面（承接 s90 string-compilation classic 三刀，HEAD 8c4a575dd）。

## 1. 面与基线

s90 HEAD 基线全子树复扫（`make testharness-html-semantics FILTER=scripting-1/the-script-element`，test-guard 包裹，/tmp/s90-fixed-scan.txt 497 行）：**173P/294F/30T**。本轮残面（module 上下文 + 外链加载器）：

- **string-compilation-module**：eval 臂 1F + 整页 1T（余 4 臂 pending 被 T 掩盖）。
- **string-compilation-base-url-inline-module / -nonce-module / -base-url-external-module**：整页 3T。
- **string-compilation-base-url-external-classic**：整页 1T——语料缺 `dynamic-import/scripts/` 目录（fetch_dir_html 只列顶层目录，depth-1 不下钻 scripts/，与 s89 的 *.css 缺口同类），页面 fetch 404。

## 2. 根因探针（STEPS 串）

单臂探针页（s91probe-*，跑完即删）+ `assert_true(false, 'STEPS='+window.__steps.join('|'))` 把状态串带出 Fail 消息。链条：

1. module 页 Promise 测试全 pending/Fail，Fail 消息 = `ReferenceError: _importMeta is not defined`。
2. `_importMeta` 是 es_module 运行时 IIFE 的局部绑定；字符串编译产物（Function/eval/setTimeout 字符串体/handler 属性）在全局作用域求值，体内被重写的 `__zw_dynamic_import.call({ referrer: _importMeta.url }, ...)` 引用不到 → ReferenceError → import 永不发生。
3. 为什么字符串编译产物里会有重写文本：`rewrite_dynamic_imports` 是**纯文本扫描器**，分不出代码与字面量——`var src = "import('./x.js')"` 这类字符串字面量内的 `import(` 也被改写。字面量随后被 Function/eval/setTimeout 编译执行，全局作用域炸 `_importMeta`。WPT 子测试名（显式字面量 `"import() must reject when there is a " + label`）在 s90 扫描里被污染成 `__zw_dynamic_import.call({ referrer: _importMeta.url }, ) must reject...` 即同源实证。

## 3. 三刀

### 刀A：重写器词法上下文感知（script-sandbox）

`crates/script-sandbox/src/es_module.rs` 重写 `rewrite_dynamic_imports`：`RewriteCtx` 栈替代逐字符盲扫——

- `Literal(u8)`（单/双引号/模板串定界字节）：不重写；处理转义对（UTF-8 按 char 步进）；`` ` `` 遇 `` ${ `` 压 `TemplateExpr(0)`。
- `TemplateExpr(u32)`（模板表达式态，值 = 未闭合 `{` 深度）：照常重写 `import(`，`{`/`}` 计深，0 深遇 `}` 回模板串。
- `LineComment` / `BlockComment`：不重写。
- 栈空 = 代码态：`import(` 首检重写，`"`/`'`/`` ` ``/`//`/`/*` 入栈。
- 5 处拷贝点全部按 `char::len_utf8` 步进（见返工①）。
- 已知限制：含引号的正则字面量（`/["']/`）会使扫描器误入 Literal 态——后续真实 import 漏改走 V8 原生宿主回调路径，语义仍对（回调同 resolver），仅丢失 referrer 换基精度；WPT 语料无此形态。

emit 串不变：`__zw_dynamic_import.call({ referrer: _importMeta.url }, `。新增 4 单测：双/单引号串内不重写、`${}` 表达式区照常重写（含嵌套花括号）、注释内不重写、多字节字符存活（`zażółć importé` 波兰文串）。

### 刀B：外链语料补齐 + R387b 求值窗换基（engine shim）

- 语料：`source tests/wpt-runner/scripts/goals/lib.sh && fetch_raw "html/semantics/scripting-1/the-script-element/module/dynamic-import/scripts/<name>.js"` 补 5 个 loader 脚本（setTimeout/eval/Function/reflected-inline-event-handlers/inline-event-handlers-UA-code；jsdelivr CDN 先行，WPT_REV 与语料同 pin）。gitignore 本地数据，不入提交。
- `crates/engine/src/js_dom_shim/part04.js` R387b（动态 `createElement('script')+src` 经典脚本）fetch.then 体：求值前按脚本 src 算绝对 URL，`__zwSwapImportReferrer(_zwAbs)` 换入、finally 换回。spec：外链经典脚本基 URL = 自身 URL（体内 eval/Function/import 同步读槽位）。
https://html.spec.whatwg.org/multipage/webappapis.html#concept-scripts-base-url

### 刀C：字符串 timer 注册期捕获基（engine shim + script-sandbox）

- `crates/script-sandbox/src/v8_runtime.rs` swapper 闭包加无参 getter 语义：`__zwSwapImportReferrer()` 无参 = 纯读（返回当前值不换）。
- `crates/engine/src/js_dom_shim/part01.js` setTimeout/setInterval 字符串分支：注册期 `__zwSwapImportReferrer()` 捕获脚本基，`new Function(fn)` 产物包装为 `_zwCallHandlerSwapBase(_zwCompiled, this, arg, _zwCapBase)`——fire 在求值窗外，届时槽位已被后续脚本改写，须用注册期捕获值。
- `crates/engine/src/js_dom_shim/part03.js` s90 的 `_zwCallHandlerDocBase` 拆出通用形 `_zwCallHandlerSwapBase(fn, thisArg, ev, base)`（base 由调用方给：inline 处理器传当前文档基，字符串 timer 传注册期捕获基）；`_zwCallHandlerDocBase` 变薄壳。part01 对 `_zwCallHandlerSwapBase` typeof 守卫（part03 后载）。

spec 分野：loader 脚本 `'../../imports-a.js'`（相对脚本自身解析——eval/Function/setTimeout 臂）vs `'./imports-a.js'`（相对文档基解析——反射/UA 臂，s90 刀③已覆盖）。

## 4. 中途返工

- **UTF-8 字符边界 panic（返工①，首轮全子树扫描空输出根因）**：拷贝点首版 `source[i..i+1]` 按字节切，WPT 语料多字节字符（`ś`）直接 panic（es_module.rs:395 `not a char boundary`），runner 批次 rc=101 全灭。改全 5 处 `char::len_utf8` 步进；新增多字节单测钉死。
- **E0004/E0308**：`match stack.last_mut()` 漏 None 臂（`.expect` 收敛）、`Some(...)` 残留 ×4（python 批量剥离）。
- **模板表达式漏改**：TemplateExpr 分支首版无 `import(` 检查，`${ import('./b.js') }` 漏改——补分支首检。
- **多字节单测断言**：JS 源保留 `\'` 转义，断言串两轮对不齐；改 raw string `r#"..."#` 免转义一次到位。

## 5. 验证

- **全子树 bitwise 复扫**（同 FILTER，基线/修复后双扫，修后 517 行 + DONE）：**173P/294F/30T → 197P/295F/25T**（+24P +1F −5T）。diff 归因（配对脚本 + 逐行核对）：
  - +24P 全落族内：module 5（eval F→P + 4 臂拆 T 显形）、base-url-inline-module 5、base-url-external-classic 5、base-url-external-module 5、nonce-module 4；
  - +1F：nonce-module must-not-inherit 2 臂显形（拆 T 露出的既有红面，s88 CSP 挂账，非回归）− module eval 1F 转 P；
  - −5T：module / base-url-inline-module / base-url-external-{classic,module} / nonce-module 整页 T 全消，无 pending 残留；
  - **族外零移动**：27 行派生名恢复（`__zw_dynamic_import.call(...)` 污染名 → WPT 原字面量名）+ import-meta-resolve-importmap 1 行同名异写，状态全数不变——名恢复本身即刀A 生效的直接证据。
  - 族终态：**42P/4F/0T**（s90 基线 classic 族 18P/2F/5T + module 面 0P/1F/5T）。
- **单测四腿**（本轮同代码态）：zero-engine **3526/3526**、zero-webview **800/800**、zero-script-sandbox v8 **362/362**（含新增 4 测）、quickjs 全绿。
- **fmt**：`cargo fmt -p zero-script-sandbox -p zero-engine` + `--check` clean（仅测试调用行重排）。
- **clippy**：`cargo clippy -p zero-script-sandbox -p zero-engine --all-targets -- -D warnings` clean。守卫说明：用户 systemd scope 本环境不可用（`Failed to connect to user scope bus`），退 `ulimit -v 24G + timeout 900` shell 兜底。
- 运行方式：make `testharness-html-semantics` + FILTER 透传（test-guard 包裹）；探针页跑完即删；语料为 gitignore 本地数据。

## 6. 挂账残面（本轮不动）

- **nonce-classic/-module must-not-inherit 4F**：CSP nonce 语义（字符串编译 handler 不继承 nonce 须拒）引擎未建模——s88 已挂账的 CSP 面。
- **正则字面量限制**：含引号正则使扫描器误入 Literal 态，后续 import 漏改走 V8 原生宿主回调（语义对、换基精度降）；WPT 语料无此形态，留待真实站点反馈。
- **语料 fetch 深度**：fetch_dir_html depth-1 不下钻子目录（本轮 scripts/ 缺口、s89 *.css 同类）——工具局限记录在案，按需 fetch_raw 手补。
- **QuickJS 后端**：维持不支持态，swapper 走 trait 默认 no-op，shim typeof 降级直执行（s90 契约单测钉死）。
