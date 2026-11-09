# M2-S340：iframe 子文档 credentials 穿面收口（Gap-B）——credentials 簇 10P/2F→12P/0F（承 s339）

日期：2026-11-08。控制文档：`docs/goal/html-semantics-compat.md`（L5 链 s340 段）。
本切片收口 s338 定谳、s339 挂账的 Gap-B：iframe 子文档 script 元素 `crossorigin`
属性不入 `extract_script_credentials`（该遍历只走主文档 HTML；子文档 HTML 由
shim regex 路径解析）——2 个 use-credentials cross 子测（expected "found"）不可达。
spec：动态 import 的请求 credentials mode 取**调用点 script 元素**的 fetch
options（`crossorigin` 属性推导；invalid value default = anonymous → same-origin）。
https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script

## 定谳（本切片新增事实，承 s338 Gap-B / s339 桥位同型）

1. **native resolver 无 credentials 槽**（s338 FIXME 在案）：子文档 corpus 面的
   `import()` 全走 classic 上下文 native settle 臂（s339 定谳），resolver 契约
   `(spec, referrer, nonce, type)` 四参无 credentials 形参——即使子文档保留属性
   也无处穿。delayed promise_test 体打破「段序窗槽位刷新」直接穿槽不可行（s124
   `__zw_lp` 同形挂账）——但**注册窗捕获 + 体执行窗恢复**（s339 桥位包装同型）
   恰好闭合：注册发生在段序窗内（捕获=该段元素 mode），体由 harness 异步调度
   （执行窗恢复捕获值），import() 同步于体执行窗内 settle。
2. **credentials 语义窗 = 两级捕获**：段序置位行只保证「注册窗」槽值；promise
   体异步调度时槽已被后续段覆写——s339 注册包装（`_zwScopeBody`）扩展为同时
   捕获 credentials（注册窗）+ 恢复（体执行窗 try/finally），与桥位同一窗。
3. **跨文档零漂移**：合并 fn 返回后宿主全局槽残留最后段值——主文档后续 classic
   面 `import()` 会误继承子文档 credentials——fn.call 快照恢复项加第四槽
   （与 s339 三注册名恢复同窗）；主文档/worker 面槽恒 undefined → 空串 → 缺省
   same-origin 零漂移（sandbox 契约钉三态实证）。

## 施工

| 面 | 位置 | 内容 |
|---|---|---|
| resolver 签名 | `script-sandbox/v8_runtime.rs` | `DynamicImportResolver` 第 5 参 `&str` credentials（nonce/type_attr 同型演进通道）；`dynamic_import_callback` 调用点 lock 外读全局 `__zwActiveScriptCred`（undefined/null→""，to_string 取值）穿入；lib.rs trait 默认实现同型 |
| native 闭包 | `webview/webview.rs` | resolver 闭包接第 5 参 → `compile_module_contract` credentials 实参（`__zw_compile_module` 第 4 参同款判定：空串/same-origin 不带标记零 src 漂移，include 附 `__zw_cred` 查询标记）；FIXME(s339) 注释收口改写 |
| shim 提取 | `js_dom_shim/part01.js` | `_zwRunIframeScripts`：partCreds 槽（`_zwCredFromAttrs206`——crossorigin 带值捕获，use-credentials ASCII 大小写不敏感→'include'，缺属性/anonymous/非法→''；bare `crossorigin` 无值落 ''，extract.rs 同映射）+ partSrcs 同索引对齐（外链/内联双分支） |
| 段序置位 | `js_dom_shim/part01.js` | partsWrapped206 循环每段前置 `globalThis.__zwActiveScriptCred=<mode>;` 复位行（仅 harness 段在场拼——无钉面普通 fixture 零变化；JSON.stringify 生成字面量；try 外置位——mode 是元素静态属性语义） |
| 包装捕获 | `js_dom_shim/part01.js` | s339 注入段 `_zwScopeBody`：注册窗捕获 `__zwActiveScriptCred` 进闭包，体执行窗存旧→置捕获值→finally 恢复（与桥位同一 try/finally 窗） |
| fn.call 恢复 | `js_dom_shim/part01.js` | s339 快照恢复项加第四槽 `__zwActiveScriptCred`（跨文档残留收尾） |

不采用（备选否决）：delayed 体直读段序槽（harness 异步调度时槽已被后续段覆写
——腿 5/8 anonymous 段误继承段 9 的 include，真红面）；s250 动态 src prepare 面
同批穿 credentials（该面语料无钉，blast radius 扩大——独立挂账）；主文档外链
模块静态图 credentials（webview FIXME(s339) 5021 维持——collect_module_deps_
recursive None，语料无钉面，另轮）。

## 验证

- **契约**：sandbox `test_native_dynamic_import_credentials_slot_s340`（槽
  'include' 直传第 5 参 / 槽 ''→空串 / 槽缺席（undefined）→空串——三 spec 分钉
  躲 s124 module map 缓存）。sandbox lib **407P/0F**（406+1）、runner bins
  **291P/0F**、webview lib **844P/1F**（红 = `proxy_mode_decodes_png_correctly`
  环境红在案）、engine lib **3811P/6F**（已知六红精确）。
- **clippy/fmt**：`-D warnings`（script-sandbox+webview+engine）全绿；rustfmt
  全绿（resolver 闭包形参表 fmt 重排在案）。
- **solo 面**：credentials 主体 **10P/2F → 12P/0F**——use-credentials cross
  双腿（classic + module）cookie "not found"→"found" 翻绿；setTimeout 变体
  文件 Timeout 承基线（C3 族独立面）。
- **the-script-element 域 A/B**：`logs/s340-script-element-after.log`
  **1070P/42F/11T** vs `s339-script-element-after.log` 1068P/44F/11T——全行
  comm（state+case+subtest，LC_ALL=C sort -u）**1122 = 1122**，双向差恰为
  use-credentials cross 双腿同键 Fail→Pass（**零 P→F**，其余全域字节等价）。

## 挂账

- **主文档外链模块静态图 credentials**（webview FIXME(s339)@5021 维持）：
  `collect_module_deps_recursive` credentials 形参 None——外链模块元素自身取回
  及静态依赖图不带 `__zw_cred`（the-script-element 语料无钉面；内联入口动态
  import 面已由 s338 编译发射收口）。
- **s250 动态 src prepare credentials**（part03.js `__zw_compile_module` 3 参
  调用）：动态插入外链 module 元素的 crossorigin 未穿——语料无钉面，独立挂账。
- setTimeout 变体（`window.resolve` 字符串求值窗全局——C3 同族）+ `window.
  onerror` per-fire `window.event` 语义偏差（承 s336–s339）。

## 证据索引

- `logs/s340-credentials-solo.log`——credentials 簇 solo 终态（12P/0F +
  setTimeout 文件 Timeout）
- `logs/s340-script-element-after.log`——the-script-element 域全域终态（A/B
  vs s339）
