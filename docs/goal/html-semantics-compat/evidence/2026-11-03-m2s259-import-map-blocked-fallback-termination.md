# M2 s259 — import map Blocked 终止回退 + resolve_imports_match 回溯臂（同域并刀；2 钉全绿）

日期：2026-11-03。切片号 s259（承 s258 挂账——「Blocked 降级语义 +
回溯臂同域可并刀」，s258 下一步候选 #1）。两刀：共享解析层回溯判别一刀 +
es_module transform 面错误通道一刀；webview/宿主编译面残余降级挂账
（s260 候选）。

## 勘根（承 s258 挂账）

1. **回溯臂缺口**：`resolve_imports_match` 前缀键臂缺规范步骤 6 判别——
   afterPrefix 以映射目标为基址解析后，解析产物序列化不以目标序列化为代码
   单元前缀（回溯越出前缀域，如 `"lib/../c.js"` 相对 `"https://x.test/a/b/"`
   解析出 `https://x.test/a/c.js`）→ 规范要求 TypeError；此前按成功返回
   越界 URL。
   https://html.spec.whatwg.org/multipage/webappapis.html#resolve-an-imports-match
2. **Blocked 降级面**：`resolve_registered_specifier`（es_module.rs:1068）
   `String` 返回签名无错误通道——Blocked（null 条目）按 Miss 降级：URL 形态
   说明符命中阻断后落回 base.join 相对解析继续内联（该 URL 已被页面显式
   阻断，规范要求 TypeError 终止全部回退，不得 asURL/相对解析续路）；裸键
   命中阻断后落 "Module not found" 通道（错误原因失真）。消费面三段：
   transform/export 七臂（Result 函数可 `?` 直传）；`static_dependency_specifiers`
   → `postorder_visit` 定序链（`module_graph_postorder` 文档化尽力而为契约
   ——「未注册依赖与解析失败说明符跳过，链接/语法错误仍由 build 路径如实
   报告」，Blocked 遵循同一契约不入定序图）；`module_provides_export`
   提供性判定（星号子链 / 显式重导出链——Blocked 应按 Missing/未绑定继续，
   不消解其他提供路径，链路本身由 transform 期 `?` 终止）。
3. **不涉面核对**：webview `apply_import_map` / 宿主 `collect_module_deps`
   走 `resolve_mapped`（`Option` 签名）——Blocked 仍按 None 降级，本切片
   不动（`resolve_mapped` 契约「无命中路径逐字节一致」是在跑站点资产零漂移
   的关键，动它需错误通道契约改造，s260 候选）；宿主 `__zw_importmap_resolve`
   回调（import.meta.resolve 面）Blocked → 空串 → prelude TypeError
   **本就规范正确**（s257 已接）；corpus 零 importmap 面（s258 grep 证据
   承用）→ runner 通道零位移，无全树 A/B；公共 API（compile_dependency_iife
   等）签名不变 → 消费面零编译漂移（zero-webview check 过）。

## 施工（2 刀 + 2 钉）

1. **回溯刀**（import_map.rs `resolve_imports_match`）：afterPrefix 解析
   产物加 `url.as_str().starts_with(target.as_str())` 判别——不满足 →
   Blocked（与解析失败同臂；目标解析失败守卫臂保留，注明 s258 后对
   parse 产物不可达）。
2. **错误通道刀**（es_module.rs `resolve_registered_specifier`）：
   `String` → `Result<String, ScriptError>`；Blocked →
   `RuntimeError("Module specifier '…' is blocked by a null entry in the
   import map")`；Resolved/Miss 分支行为不变（尾部相对解析路径原样，
   `Ok` 包裹）。接线：transform/export 七臂 `?` 直传；定序链
   `filter_map(.ok())`（尽力而为契约注释）；`module_provides_export`
   星号臂 `let-else continue`、显式重导出臂 `match &from_specifier`
   分 `Some(Ok)/Some(Err)`（`None` = 无 from 子句本地导出语义保持，
   不得混入 Blocked——`.ok()` 拍平会把阻断误判为 Provides）。
3. **钉面 ×2**：
   `backtracking_above_prefix_target_blocked_s259`（import_map.rs——
   回溯越界 Blocked + 前缀域内 Resolved 判别臂）；
   `test_compile_dependency_iife_blocked_entry_terminates_fallbacks_s259`
   （es_module.rs——compile_dependency_iife 端到端两臂：裸键 null 条目 /
   URL 形态键 null 条目均以 null-entry 错误终止编译；选公共 API 级是
   因 resolve 级 `Err` 断言在旧 `String` 签名下无法编译——编译失败不冒充
   行为失败，红态证据落在本钉）。
4. **注释面**：import_map.rs 模块「已知偏差」#2 与 `resolve_mapped` 文档
   改为双面口径（es_module 面已终止 / webview·宿主面仍降级）；webview.rs
   `apply_import_map` FIXME 改挂余面。

## 验证

- **红态实证**：2 钉先于修复在 HEAD 跑——回溯钉 `Resolved("https://
  x.test/a/c.js")`（越界 URL 按成功返回）≠ Blocked；compile 钉臂 1
  `RuntimeError("Module not found: blocked")`（Miss 降级落相对解析通道）
  ≠ null-entry 终止。旧版因目标行为失败证据在账。
- **绿态**：修复后 `cargo test -p zero-script-sandbox`（default v8，
  test-guard 守卫）**392 全绿**（390 基线 + 2 钉；含 es_module 全部
  transform/export/compile 面与 import map P6/P6C 既有钉）。fmt 后复跑
  同 392 绿。
- **fmt/clippy**：`cargo fmt` 两 crate 过（rustfmt 少量归一化，复跑绿态
  确认）；clippy -p zero-script-sandbox --all-targets（test-guard 守卫）
  零告警；`cargo check -p zero-webview` 消费面过。
- **无全树 A/B**：corpus 零 importmap 面（s258 grep 证据承用）→
  runner 通道确定性零位移；公共 API 签名不变。

## 挂账更新

- **webview/宿主编译面 Blocked 降级**（承前，收敛为 s260 候选）：
  `apply_import_map`（webview.rs）/ `collect_module_deps`（js_worker /
  tab_js_worker）/ script_prefetch 走 `resolve_mapped`（`Option` 签名），
  Blocked 按 None 降级后按 parent-join 继续抓取——规范要求 TypeError
  终止。改造需 `resolve_mapped` 三态化或新增全量解析入口 + 动态 import
  resolver 错误通道（webview `set_dynamic_import_resolver` 契约）+ 宿主
  compile contract 失败 tag 路由，三面接线一次收口。
- 承前不动：import maps 剩余语义面（多张 map 整页失败语义 / 动态插入 /
  键排序序偏差）；data-zw-harness 投递标记保真面（s255 候选 #3）；
  credentials 面 / shared-dependency 取回计数 / microtask checkpoint
  时序族 / 并发慢 import dedup（s256）；customizable-select 177F（挂起
  待用户拍板 GB-20261031 征询在途）；renderer SetDomSnapshot entries 化
  （s174，低优先）。

## 下一步候选

1. webview/宿主编译面 Blocked 终止（resolve_mapped 三态化 + resolver
   错误通道 + compile contract 失败 tag 路由——三面一次收口）。
2. data-zw-harness 投递标记保真面（s255 候选 #3 承前）。
3. import maps 剩余语义面其余（多张 map / 动态插入 / 键排序序）。
