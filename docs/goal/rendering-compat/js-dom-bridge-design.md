# JS-DOM-Bridge 设计（Reftest Harness JS → DOM 变更 → 重渲染）

**版本**：v1.0（RFC，read-only 设计产出）
**日期**：2026-06-23
**状态**：草稿（未落地代码）
**关联**：`docs/goal/rendering-compat/master.md`（R519/R520 plateau）；reftest harness `tests/wpt-runner/src/reftest.rs`；`crates/script-sandbox`

---

## 0. 执行摘要

- **一句话目标**：让 reftest harness 在截图前执行页面 JS 并把 DOM 变更（属性/文本/结构）应用到引擎的 `Document`，然后重新渲染，从而翻转 ~50+ 个「动态」reftest（css-grid img-src ~13 + css/CSS2/box-display delete/insert ~20 + CSS2 dynamic/opacity-transition 若干）。
- **当前阻塞**：`reftest.rs:675 execute_scripts` 对 HTML 串跑 JS、**无 DOM 绑定**、结果丢弃；`V8Sandbox` 仅有 `new/with_config/execute/execute_json`，**无 custom-binding API**。
- **推荐方案**：**string-callback + JS DOM-shim**——V8Sandbox 增一个极简 `register_callback(name, Fn(&[String])->String)`；harness 注入一段 JS shim 把标准 DOM API（`document.querySelector(id).src=v` 等）翻译成扁平化 Rust 回调（`__zw_set_attr / __zw_remove / __zw_append ...`），回调操作引擎 `Document`；JS 后 harness 重渲染。
- **为什么不是全 V8 对象代理**：标准 DOM 是 `obj.prop = v` 的对象/属性语义，rusty_v8 ObjectTemplate + property setter 工作量大且易错。**JS shim** 把对象语义降级为扁平字符串回调，V8Sandbox 侧零对象代理、改动最小，复杂度集中到 shim（JS，易迭代）。
- **首个落地步骤**（多会话）：Phase 0 = V8Sandbox `register_callback`（string-callback，零回归，单测）+ harness 注入空 shim 跑通「JS 执行 → 无 DOM 变更 → 重渲染」管线（0 flip，验证管线）。

---

## 1. 背景与目标

### 1.1 失败分布（JS 依赖案，self-source reftest）

reftest harness 的 `execute_scripts`（reftest.rs:675-701）提取 `<script>` 内容、丢进 `V8Sandbox::execute`，但：

- V8 沙箱**无 `document` / `window` / 任何 DOM 全局** → JS 抛 `ReferenceError: document is not defined`（实测见 grid/box-display 跑分日志）。
- 即便 JS 能跑，它操作的是**纯 V8 值**，与引擎 `zero_dom::Document` 无连接 → 渲染仍取**初始 HTML**。

因此所有「JS 改 DOM 后才达到期望状态」的 reftest 都渲染初始态 → FAIL。代表簇：

| 簇 | 数量 | 典型 JS |
|----|------|---------|
| css/css-grid 动态 | ~13 | `document.querySelector("img").src = "green.png"` / `getElementById('t').src='data:image/svg...'` |
| css/CSS2/box-display delete/insert | ~20 | `createElement`+`appendChild`/`insertBefore`/`removeChild` + `className` + `class="reftest-wait"` 同步 |
| CSS2 dynamic / opacity-transition | 若干 | `target.style.opacity=1`（须 DOM-JS bridge，reftest.rs:675 注释明言「不影响渲染输出」） |

### 1.2 业务/用户目标

- **业务**：reftest 真实通过率（self-source + chromium-Oracle）提升。
- **可验证成功标准**：① css-grid img-src ~13 案 self-source FAIL→PASS（最简 `img.src` 路径）；② box-display delete/insert ~20 案（DOM 结构变更 + reftest-wait）；③ 全量 reftest loose/strict 不退、`make test` green。

---

## 2. 现状分析

### 2.1 V8Sandbox（`crates/script-sandbox/src/v8_runtime.rs`）

```
V8Sandbox::execute(code):
  HandleScope(isolate) → ContextScope(context) → Script::compile → run → 返回 result_str
```

- 全局对象上**仅注册了 JSON**（line 270-273），无自定义函数。
- 无 `register_callback` / `set_global` / FunctionTemplate 暴露。
- `script-sandbox` crate **不依赖** `engine`/`dom`（低层沙箱），故 DOM 操作不能直接发生在沙箱内——必须由**宿主（reftest harness）**注入。

### 2.2 reftest harness（`tests/wpt-runner/src/reftest.rs`）

- `run_reftest_with_base`（line 269）渲染 test/ref PNG 对比。
- 渲染走 `render_html`-类入口（engine pipeline：parse → style → layout → paint），**一次性**。
- `execute_scripts`（line 675）在渲染流程外、对 HTML 串跑 JS、结果丢弃。

### 2.3 关键约束

- **零 count 回归**（项目硬标准）。
- **跨 crate 通信**：V8 回调（script-sandbox）须操作 `Document`（dom/engine），不能在 script-sandbox 内直接持有 Document（层级倒置）。→ 用**宿主注入回调**（dependency injection）：harness 注册回调，回调闭包捕获 `&mut Document`。
- **rusty_v8 对象代理成本高**：标准 DOM 是 `el.src = v`（对象+属性 setter），须 ObjectTemplate + accessor。→ 用 **JS shim** 把对象语义降级为扁平字符串回调，V8Sandbox 只需 `register_callback(name, Fn(&[String])->String)`。

---

## 3. 推荐方案：string-callback + JS DOM-shim

### 3.1 架构

```
reftest harness (tests/wpt-runner):
  1. parse HTML → Document D（既有）
  2. 计算 styles / 布局 / 渲染初始 PNG（既有，render_html）
  3. 【新】if 有 <script>:
       a. V8Sandbox::register_callback("__zw_set_attr", |args| { D.set_attr(args[0],args[1],args[2]); "ok" })
          register_callback("__zw_remove",   |args| { D.remove(args[0]); "ok" })
          register_callback("__zw_append",   |args| { D.append(args[0],args[1]); "ok" })
          ...（按 Phase 增补）
       b. V8Sandbox::execute(DOM_SHIM_JS)   // 注入标准 DOM 全局（见 3.2）
       c. V8Sandbox::execute(page_script)   // 页面 JS，经 shim 触发 __zw_* 回调改 D
       d. 【新】用变更后的 D 重新计算 styles/layout/paint → 覆盖初始 PNG
  4. 对比 test/ref PNG
```

### 3.2 JS DOM-shim（注入到 V8 全局，纯 JS）

把标准 DOM API 翻译成扁平回调。例（最小集，覆盖 img.src / className / textContent / remove）：

```js
globalThis.document = {
  querySelector: (sel) => __zw_wrap(sel),
  getElementById: (id) => __zw_wrap('#' + CSS.escape ? '#'+id : '#'+id),
};
globalThis.__zw_wrap = (sel) => ({
  set src(v)        { __zw_set_attr(sel, 'src', v); },
  set className(v)  { __zw_set_attr(sel, 'class', v); },
  set textContent(v){ __zw_set_text(sel, v); },
  remove()          { __zw_remove(sel); },
});
```

- shim 用 **CSS 选择器字符串** 作为元素句柄（WPT reftest JS 几乎都用 id，选择器稳定且 DOM 变更后仍可重选）。
- 复杂操作（`createElement`+`appendChild`，box-display 簇）需扩展 shim（`__zw_create`+`__zw_append`，Phase 2）。
- **已知限制**：选择器在 DOM 变更后可能失效（如 remove 后再选）；shim 对 box-display 增删案需「按 id 句柄」而非「按选择器重选」——Phase 2 用稳定 id 句柄（`__zw_get_by_id(id)` 返回句柄 token）。

### 3.3 V8Sandbox 改动（极简）

新增一个方法：

```rust
impl V8Sandbox {
    /// 注册扁平字符串回调（宿主注入 DOM 操作）。execute 前调用。
    pub fn register_callback(
        &mut self,
        name: &str,
        callback: Box<dyn Fn(&[String]) -> String + Send + Sync>,
    ) { /* 存入 self.callbacks；context 初始化时建 FunctionTemplate 挂到 global */ }
}
```

- rusty_v8：`FunctionTemplate::new` + `FunctionCallbackArguments` → 转 `Vec<String>` → 查表调 closure → `ReturnValue::set(String)`。
- context 已存在（`cached_context`），回调表在 context 创建时（`with_config`/首次 `execute`）一并挂载。
- **零行为变更**：无 `register_callback` 调用时，沙箱行为完全同今（既有 `execute` 路径不变）→ 零回归。

### 3.4 harness 重渲染（reflow-after-JS）

- JS 执行后，Document D 已变更（属性/文本/结构）。
- harness 复用既有 engine pipeline 对 D 重算 styles/layout/paint → 覆盖初始 PNG。
- **reftest-wait 同步**：`<html class="reftest-wait">` + JS 末尾 `document.documentElement.className=""`——shim 把 `className=""` 映射为 `__zw_set_attr('html','class','')`，harness 在 JS 后检查 html class 是否仍含 `reftest-wait`：若是则**不等**（当前 harness 本就不等，无等待语义）→ 直接用 JS 后态。故 reftest-wait 对当前 harness 天然无影响（JS 后即截图）。

---

## 4. 分阶段实施计划

| Phase | 内容 | 验证（零回归） | 预期 flip |
|-------|------|---------------|-----------|
| **0** | V8Sandbox `register_callback`（string-callback，rusty_v8 FunctionTemplate）+ 单测；harness 注入**空 shim** 跑通「execute → 无 DOM 变更 → 重渲染」管线 | `make test` green；reftest 全量持平（空 shim 无变更，重渲染同初始） | 0（管线验证） |
| **1** | shim 最小集（`img.src` / `el.className` / `textContent` via `__zw_set_attr`/`__zw_set_text`）+ harness 重渲染 | css-grid img-src ~13 案 self-source FAIL→PASS；全量零回归 | ~13 |
| **2** | shim 增 `createElement`/`appendChild`/`insertBefore`/`removeChild`（按 **id 句柄 token**，非选择器重选）+ reftest-wait 语义 | css/CSS2/box-display delete/insert ~20 案 FAIL→PASS | ~20 |
| **3** | `el.style.prop = v`（CSSStyleDeclaration shim → `__zw_set_style(el,prop,val)`） | CSS2 dynamic / opacity-transition 簇 | 若干 |

每 Phase 独立可 land（零回归硬标准），失败可单 Phase 回退。

---

## 5. 风险与开放问题

1. **rusty_v8 FunctionTemplate 生命周期**：callback 表须在 isolate 生命周期内有效（`Send+Sync` 闭包 + `Persistent`）。Phase 0 单测验证。
2. **选择器 vs id 句柄**：Phase 1 用选择器（img-src 簇够用）；Phase 2 增删须用稳定 id 句柄（remove 后选择器失效）。
3. **base_dir / 图片加载**：`img.src = "green.png"` 后须触发图片抓取+解码（既有 `fetch_image_subresources`，R318）。Phase 1 验证 src 变更触发重抓取。
4. **跨 crate 边界**：harness（tests/wpt-runner）持有 `Document`，注册闭包捕获它——闭包 `Send+Sync` 要求 `Document` 同步可用（reftest 单线程，OK）。
5. **性能**：JS 后重渲染 = 2× 渲染开销，仅对有 `<script>` 的 case 触发（~500 案），可接受。
6. **scope**：仅 reftest harness（测试工具），**不**改浏览器主进程 JS 执行——浏览器侧 DOM-JS bridge 是更大里程碑，不在本 RFC scope。

---

## 6. 与既有工作的关系

- **不是**浏览器引擎的完整 DOM-JS binding（那是 milestone 规模，script-sandbox ↔ engine 全桥接）。本 RFC **仅** reftest harness 测试工具内的最小 bridge，用 string-callback + JS shim 规避 rusty_v8 对象代理成本。
- **不触碰** IFC Phase-A / multicol Phase-2 / baseline-export（独立结构性 lever）。
- Phase 0 的 `register_callback` 是**通用** V8Sandbox 能力，未来浏览器侧 DOM bridge 亦可复用。

---

## 7. 结论

JS-DOM-bridge 是 R513-R518 style-system vein 之后**最高杠杆**剩余 lever（~50 案），但非单会话 clean slice——须多会话分 Phase 落地。本 RFC 用 **string-callback + JS shim** 架构把 V8Sandbox 改动降到极小（一个 `register_callback` 方法 + 零回归），复杂度集中到 JS shim（易迭代）+ harness 重渲染。Phase 0（register_callback + 空 shim 管线）是 bounded、零回归的首步，验证后 Phase 1/2/3 逐步兑现 flip。
