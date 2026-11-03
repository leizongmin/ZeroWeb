# M2 s254 — import.meta.resolve + 进程内 import map 接线（4 键 F→P）

日期：2026-11-03。切片号 s254（承 s253 候选 #1）。目标页
import-meta-resolve-importmap.html 四键全 F：import map 在 webview
进程内路径（runner 通道）从未接线——基建（`import_map.rs` + 12 单
测 + `resolve_registered_specifier` map 臂）s92 起就在，缺的是消费面。

## 勘根

- **基建现状**：`ImportMap::parse/parse_from_page/resolve/
  resolve_mapped` 完整（精确键 + 尾斜杠前缀键 + scopes + Blocked，
  Vec 保插入序）；`extract_import_map_json`（extract.rs）此前**零调用
  方**——renderer/tab 多进程宿主已接（js_worker parse_page_import_map
  首张合法 map），webview 进程内路径从不解析。
- **四键语义**（WPT import maps spec）：①bare 键映射
  （`resolve("bare")` → `https://example.com/`）；②URL-like 键重写；
  ③ToString（`resolve()` → "undefined" 键、`resolve(1)` → "1"、
  `resolve({valueOf})` → "[object Object]"——String() 只咨询
  toString）；④链式 map 再应用（`import("./start")` → export-1，
  `import(import.meta.resolve("./start"))` → 映射结果再过 map →
  export-2）。
- **spec 口径**：import.meta.resolve 入参 ToString 后走 resolve a
  module specifier 全算法；map 命中 → 映射 URL 按 base 绝对化；
  URL-like 未命中 → asURL 回退；裸未映射/Blocked → resolution
  failure → TypeError（较动态 import 的 join 降级更严——spec 面新
  API 无向后兼容约束）。map 应用用 referring script base；map 键在
  parse 期按文档基址归一化。
  https://html.spec.whatwg.org/multipage/webappapis.html#dom-import-meta-resolve
- **语料**：import-meta/resources/ 整目录缺失（③d 补拉
  export-1.mjs/export-2.mjs，s227 教训——施工前先验资产存在性，
  免施工后假红误判）。semantics 树内仅此一页有 importmap（波及面单
  页）；importmap 脚本经 script_type_is_javascript 过滤不执行。

## 施工（9 处小刀）

es_module.rs 三处：
1. prelude `__zwImportMapResolve` 宿主桥：返空 → TypeError（resolution
   failure）；回调未注册宿主（renderer/tab 接线未达）typeof 守卫同落
   TypeError 而非 ReferenceError。
2. `build_module_script` 发射 `_importMeta.resolve`（String(spec) +
   本模块 `_importMeta.url` 穿入）——恒发射，无 map 时即纯 URL 解析
   语义，零行为差。
3. `build_dep_iife` 同型发射（依赖模块 import.meta.resolve 面）。

webview.rs 六处：
4. 页面 HTML `extract_import_map_json` → 首张合法 `ImportMap`
   （parse 失败 warn 整张跳过——spec register an import map 语义，
   与 js_worker 形态对齐）。
5. 无条件注册 `__zw_importmap_resolve` 回调（map.resolve 全算法 +
   映射值 base join 绝对化；Miss/Blocked 返空）。
6. `__zw_compile_module` 闭包：spec 经 `apply_import_map` 改写
   （resolve_mapped 只在实际命中时返回映射 URL——未命中原样，既有
   parent-join 路径零漂移）。
7. v8 原生 dynamic import resolver 闭包：同形 spec_eff（CSP nonce
   检查与 fetch 均用映射后 abs——fetch 的就是映射 URL）。
8. 静态面 registry `set_import_map`（resolve_registered_specifier
   map 臂激活）。
9. `apply_import_map` helper fn（Blocked 按 Miss 降级——FIXME 与静
   态面同注记：String 返回签名无错误通道）。

goal 脚本 ③d 段 fetch_raw ×2（export-1/2.mjs，幂等固化坑同 ③b）。

## 验证

- **红态实证**：新钉 `test_import_meta_resolve_s254`（发射断言 +
  宿主桥行为 + IIFE 局部基址穿入 + ToString no-arg TypeError 四断
  言）；撤三处发射/桥行 → FAILED，恢复 → 绿。守卫行为由 no-arg 臂
  兼覆盖（返空通道）。
- 单测：script-sandbox **381 全绿**（380 + 新钉）+ webview **835
  全绿**。fmt/clippy 干净（warning 全在 layout-engine 既有面）。
- solo（FILTER=import-meta，evidence/logs/s254-solo.log，守卫终态代
  码复跑）：**4 Pass / 0 Fail**——四键全翻绿，含链式第四键（新拉语
  料 + map 两级应用 + 映射后绝对 URL 模块缓存语义）。
- 全树 A/B（evidence/logs/s254-ab.log vs s253-ab.log）：行级
  7789P/439F/78T；键级 join **8306→8303 键闭合（-5 LOST +2 GAINED），
  迁移恰 5 键**——目标 4 键 **F→P** + checkable-active-onblur-with-
  click **F→P**（在册 checkbox `:active` 双向漂移族，s253 轮 P→F
  本轮回摆至基线态，通道无涉）。两处键形态位移（focus-dynamic-type-
  change-on-blur 1 子测试→文件级 Timeout；select-home-end-pagedown-
  pageup.optional 4 子测试 Fail→文件级 Fail）经 **solo 复跑证实为
  A/B 运行窗口并发编译负载所致**（红态编译与 A/B 同机并行，CPU 争
  抢压时序重页）：focus 页 solo 80/80 全 Pass（LOST 键复得）、select
  页 solo 回惯常 4 子测试 Fail（与 s251/s252/s253 同形）。
  **教训：全树 A/B 运行窗口禁并发编译负载。**
- A/B 构建为守卫前置码；守卫仅加未接线宿主回落臂（runner 进程内路
  径回调恒在场，行为恒等），守卫终态 solo 复跑 4/4 确认。

## 挂账更新

- **renderer/tab 多进程宿主 import.meta.resolve 接线（新）**：两宿
  主已 parse map（js_worker parse_page_import_map）但未注册
  `__zw_importmap_resolve` 回调——模块调 import.meta.resolve 落
  TypeError 守卫臂（s254 前为 "not a function" TypeError，均抛异常
  非回归）。补回调即同 webview 五刀形态（map 已在 scope）。
- **import maps 剩余语义面（新）**：多张 map 只取首张合法（后续
  spec 要求整页失败；现 warn 跳过）；`<script type=importmap>` 动态
  插入/update/revoke 未实现；Blocked 条目按 Miss 降级（FIXME 既有）。
- 承前不动：delay-load 异步通道、data-zw-harness 投递标记保真面、
  s250 动态内联 module 臂 eval 通道、instantiation-error 星号导出歧
  义等（见 s253 账本）。

## 下一步候选

1. instantiation-error-6/7 星号导出歧义检测（transform 管线内
   export * 链歧义归并 + blame 定位，中刀位）。
2. 动态 import 异步通道（delay-load 根因切片；顺带覆盖 trickle 类
   慢模块时序族）。
3. renderer/tab 宿主 import.meta.resolve 接线（小刀，补多进程面对
   称）。
