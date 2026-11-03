# M2 s257 — renderer/tab 宿主 import.meta.resolve 接线（多进程面对称小刀；2 钉全绿）

日期：2026-11-03。切片号 s257（承 s254 挂账 #1——「renderer/tab 多进程宿主
import.meta.resolve 接线」，3 轮在账后收口）。纯接线小刀：宿主侧已有本代 map
槽位与解析基建，缺的只是回调注册一刀；零 runner 路径改动。

## 勘根（承 s254 挂账）

1. **挂账本体**（s254 evidence 原文）：两宿主已 parse map
   （js_worker `parse_page_import_map` → `Arc<Mutex<Option<ImportMap>>>`
   槽位，快照换代每代无条件重设），但未注册 `__zw_importmap_resolve`
   回调——模块调 `import.meta.resolve` 落 prelude `__zwImportMapResolve`
   （es_module.rs:422）守卫臂抛 TypeError（回调缺席与 resolution
   failure 同落点：typeof 守卫 + 空串返回守卫）。webview 进程内路径
   s254 五刀已接（webview.rs:4204 注册处 map.resolve +
   resolve_document_url，Miss|Blocked → 空串）；两宿主差同一刀。
2. **形态核对**：renderer（js_worker.rs）与 tab（tab_js_worker.rs）的
   register_module_compile_callback 均为 worker bootstrap 一次性装配，
   map 槽位读取模式与 `__zw_compile_module` 回调一致——回调时刻
   `lock().ok().and_then(clone)`（锁毒化回落空 map 走 Miss 面，不放大），
   SetDomSnapshot 每代重设后新 map 即生效。宿主为 shim 通道（无
   set_dynamic_import_resolver）→ s256 poll 泵不涉。
3. **新发现记档（非本切片修复面）**：`resolve_imports_match`
   （import_map.rs:199）映射目标值经裸 `Url::parse` 解析（无基址）——
   path-absolute/relative 映射值（如 `"utils/": "/assets/utils/"`）解析
   失败 → **Blocked**（规范要求 register an import map 阶段按基址
   归一化映射值，归一失败整张 map 拒注册）。webview s254 面同缺（非
   本轮接线引入）→ 新挂账「import map 映射值归一化缺口」。本切片钉面
   用绝对 URL 映射值绕开该缺口，只锁接线语义。
   https://html.spec.whatwg.org/multipage/webappapis.html#register-an-import-map

## 施工（2 刀 + 2 钉）

1. **renderer**（js_worker.rs）：register_module_compile_callback 内
   `__zw_compile_module` 注册前新增 `__zw_importmap_resolve` 宿主回调
   （webview s254 同形）：`map.resolve(spec, base)` → Resolved →
   `resolve_document_url(base, url)`（绝对值原样、相对值按页面基址
   绝对化）；Miss|Blocked → 空串（prelude 桥抛 TypeError）。
2. **tab**（tab_js_worker.rs）：同构一刀（注释按 tab 侧「快照换代」
   口径）。
3. **钉面 ×2**（镜像）：`renderer_js_worker_importmap_resolve_wired_s257`
   / `tab_js_worker_importmap_resolve_wired_s257`（tab 侧沿用 r386
   同款 feature gate——bare 无 v8/quickjs 不跑，生产 --features v8
   验证）。worker 真装配路径：spawn → set_dom_snapshot（importmap 页）
   → execute_script_direct 直调回调。五断言臂：精确键映射（绝对值
   原样）/ 尾斜杠前缀键子路径组合 / URL-like 未命中 asURL 回退（按
   base 绝对化）/ 裸说明符未映射返空（= TypeError 通道）/ 快照换代
   重设后旧键失效（跨代残留不可能存活）。

## 验证

- **红态实证**：两钉分别在 neutralize 注册名（改名即可达）后首断言挂
  （ReferenceError 通道——旧版本因目标行为失败），还原后全绿。
- **renderer**：`cargo test -p zero-renderer`（default v8，
  --test-threads=2）**369 全绿**（368 基线 + 1 钉）。
- **browser**：`cargo test -p zero-browser --features v8 --bin
  zero-browser`（--test-threads=2）**741 过 + 5 失败 = HEAD 既有**
  （compositor reaping / service-worker multiprocess restart /
  default_actions / cpu-gpu matrix / paste_and_go——stash 法同跑 HEAD
  740 过 + 同 5 失败定谳非本轮回归；740 基线 + 1 钉 = 741）；lib 10
  全绿；integration cdp_file_scope_policy 2 失败 = HEAD 既有（同
  stash 法）。
- **fmt/clippy**：两 crate all-targets 零告警（browser 侧含 --features
  v8 与 --no-default-features --features quickjs 双形态）。
- **无全树 A/B 说明**：本切片零 runner 路径 / 语料改动——宿主接线只
  影响 renderer/tab 两 app 的模块编译回调装配面（多进程生产路径），
  runner 走 webview 进程内路径不受影响；单测钉面即门禁。

## 挂账更新

- **import map 映射值归一化缺口**（新记档）：`resolve_imports_match`
  映射目标裸 `Url::parse` 无基址——path-absolute/relative 值 →
  Blocked；规范要求 register 阶段按基址归一化、失败整张拒注册。
  webview/宿主两面同缺（s254 起既存，非本轮引入）。
- 承前不动：import maps 剩余语义面（多张 map 整页失败语义 / 动态插入 /
  Blocked 降级——Blocked 降级与新缺口同域可并刀）；data-zw-harness
  投递标记保真面（s255 候选 #3）；credentials 面 / shared-dependency
  取回计数 / microtask checkpoint 时序族 / 并发慢 import dedup（s256）。

## 下一步候选

1. import map 映射值归一化（新缺口小刀——register 归一化 + 
   resolve_imports_match 阻断面收敛，可并 Blocked 降级语义）。
2. data-zw-harness 投递标记保真面（s255 候选 #3 承前）。
3. import maps 剩余语义面其余（多张 map / 动态插入）。
