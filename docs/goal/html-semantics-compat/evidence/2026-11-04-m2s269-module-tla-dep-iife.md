# M2 s269 — module TLA 依赖 IIFE async 包裹 + export 解构绑定接线（module-import-referrer Timeout→4P 收口）

日期：2026-11-04。切片号 s269（承 s268 下一步候选 #1；s116 挂账
"module-import-referrer 顶层 await face"收口）。1 文件 6 刀 4 钉
（script-sandbox 单车道，零 runner 改动）。

## 勘根

1. **入场承 s268 三候选核账**：候选③（script-element-css-src module
   MIME 余项）已被 s266 收口（3/3 F→P，s268 账本过时承接）——销
   候选③；候选②（image-loading-lazy-referrerpolicy-change，deferred
   img 面）留给后续轮次；选候选①（module-import-referrer 顶层
   await face，s116 挂账余项）。
2. **目标页结构**：inline module 含 2 静态 import（本目录 .js 与
   referrer-checker.py 端点）+ 1 个顶层 `await import(...)`；外链
   .js 为 TLA 依赖（`export ... from` re-export + 顶层
   `await import(...)`）。页断言 4 断言：inline static/dynamic
   referrer=文档 URL，external static/dynamic referrer=importer 模
   块 URL。
3. **Timeout 根因三层**：
   - ① `build_dep_iife` 恒同步 IIFE → TLA 依赖体内顶层 `await`
     落非 async 上下文 SyntaxError（s87 记录的「module TLA」面本
     体）；
   - ② `transform_export` 对解构声明（`export const { a } = ...`）
     `extract_binding_name` 返 "unknown" → 发射
     `_exports.unknown = unknown` ReferenceError；
   - ③ `module_provides_export` 同不识别解构导出名 →
     ensure_module_export 链接假报 "does not provide an export"。
4. **referrer 四断言链已通**：rewrite 穿 `_importMeta.url` +
   runner 端点臂 compute_module_referer（s116），本切零 runner 改
   动。
5. **TLA 探测器「宁欠勿过」**：naive `contains("await")` 会在
   promise_test(async (t) => {...}) 括号/花括号内 await 误报
   （blob-url-workers 等页会把同步依赖实例翻成 Promise 值致静默
   undefined）→ 改深度门+字符串/注释/正则上下文跳过+箭头守卫的字
   节面扫描器。
6. **裁定不实施「入口 transitive-TLA 升级」**（sync 入口 import
   TLA dep 应升 async）：top-level-await/ 语料资产为孤儿（无
   HTML 页引用），零钉面不扩面，FIXME 注记挂账。
7. **动态 import() TLA 模块面免费获益**：build_dep_iife async 包
   裹后 `__zw_load_module` eval 缓存 Promise，`Promise.resolve`
   天然解包，`await import(tla)` 语义正确，无需额外刀。

## 施工（1 文件 6 刀 4 钉，es_module.rs）

1. `enum DepInline { Fresh/Visited }` 均带 async 旗标
   （module_source_has_top_level_await 判定），`await_prefix`
   按依赖 async×enclosing_async 二维发射 `await ` 前缀。
2. `build_dep_iife`：TLA 依赖 `(async function(){...})()` 包裹，
   transform_import/transform_export 穿 enclosing_async。
3. `transform_import` 四臂（副作用/star-as/具名/默认）发射
   `var {exports_var} = {await_prefix}{memoized 或 exports_expr};`。
4. `transform_export` 三 dep 臂同 await 前缀；const/let/var 臂改
   用 `export_binding_wiring`（解构模式逐名接线
   `_exports.n = n;`，回落 extract_binding_name）。
5. `module_provides_export` 加解构名判定
   （extract_destructuring_export_names：对象模式顶层字符串感知
   条目，嵌套/数组/rest→None 保守回落）。
6. `compile_module_script` async_body 判定统一走 statement 扫描
   器（替代 naive `contains("await")`）。
   钉 ×4：tla 检测 9 断言矩阵 / tla 依赖 IIFE async+解构导出 /
   async 入口 await 消费 TLA 依赖 / provides_export 解构名。

## 验证

- **钉**：`cargo test -p zero-script-sandbox --lib`（test-guard）
  **400 passed / 0 failed**（377 基线 + 4 新钉 + 余量对齐）。
- **solo**：module-import-referrer.html **Timeout→4/4 子测 Pass**
  （日志 evidence/logs/s269-solo-final.log）。
- **族 A/B**（FILTER=the-script-element/module，stash 同二进制）：
  233P/41F 双侧同态——41F 全为既有红零位移；目标页 4 键唯一差
  异（日志 s269-family-{baseline-a,b}.log）。
- **全树 A/B**（stash 同二进制法，test-guard + RAF）：
  **7839P/433F/69T/11NR → 7845P/428F/68T/11NR**——(key,state)
  join 三处位移全部归因，**REGRESSED=∅**：
  - 目标闭合：module-import-referrer.html 文件级 Timeout→4 子测
    Pass（精确 4 键）；
  - checkable-active-onblur-with-click ×2 F→P：在册 `:active`
    漂移族（s238/s245/s248/s268 同键混态），B 日志内同文件
    `pressing the following label` ×2 Pass 与 `pressing the
    parent label` ×2 Fail 并存自证，销账；
  - select-home-end-pagedown-pageup-detailed.optional A:4 子测
    Fail（`:open` 选择器不支持）→ B:文件级 Fail（脚本挂死
    90s）：B 树 solo 同态复现 4F `:open`（日志
    s269-select-flaky-solo-b.log），挂死为负载时序 flaky，销账。
  （日志 s269-fulltree-{baseline-a,b}.log）
- **make test**（test-guard）：engine `cargo test --lib`
  **3722 passed + 5 failed**——5 红为 R-t2qq adopted-stylesheets
  系列既有（s265/s268 账本记档同集，尾分号 Blink 形过期期望），
  本流不入侵；script-sandbox lib **400 passed / 0 failed**。
- fmt/clippy：workspace guarded-clippy 零告警。

## 挂账更新

- module-import-referrer TLA face **销案**（s116 挂账该项收口；
  文件级 Timeout→4 子测全绿）。
- 新记录（零 corpus face，不扩面）：
  ① **sync 消费方 × TLA 依赖**：sync import 发射不改 await 前缀
  （await_prefix 恒空），消费方把缓存 Promise 当 namespace 直接
  属性访问得 undefined——spec 是整 graph async（sync 侧也
  await），语料无断言页，FIXME 注记；
  ② **入口 transitive-TLA 升级未建模**（top-level-await/ 孤儿资
  产，勘根⑥裁定）；
  ③ **TLA 探测器欠检测残面**：括号内 await（`(await p)` 形）、
  模板插值内、块语句 `{ await x; }`、深度 0 `=>` 后逗号续项——
  欠检测维持现状（该面今日即红），文档化不扩刀；
  ④ **dynamic-import/tla-deferred-evaluation.tentative.html +
  cyclic-tla-defer-eval.tentative.html**（import.defer 面未支
  持，族跑既有红在册）；
  ⑤ **modulepreload referrer 族**（echo-referrer.py 端点 +
  link modulepreload 面，独立候选）。
- 承前不动：image-loading-lazy-referrerpolicy-change 1F（s268
  候选② 承接）；R-t2qq 5 红（归该流自查）；css-module
  relative-urls / redirect-relative-urls 3F；module map 缺席
  （s256/s267 注记）。

## 下一步候选

1. image-loading-lazy-referrerpolicy-change 勘根（deferred img
   装载策略切换面，engine/webview 车道，s268 候选② 承接）。
2. modulepreload referrer 族（本切挂账⑤，runner 车道，s116
   referrer 端点仿真先例可复用）。
3. dynamic import() 元素级 referrerpolicy 穿参（s268 挂账，
   webview 三动态调用点现传 None）。
