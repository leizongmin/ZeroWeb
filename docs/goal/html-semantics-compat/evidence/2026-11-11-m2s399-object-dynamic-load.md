# M2 切片 399 — object 动态加载链（插入钩子 + data setter + about:blank + win.Window；driving 8T→P）

日期：2026-11-11（承 s398 下一步：object contentDocument/object-events 中型切片三案联动）
commit：见 git log s399 提交

## 勘根

s398 收口后 the-object-element/ 目录遗留三个文件级 Timeout：`historical.html`（2 async_test，
静态 setAttribute+appendChild→onload→contentDocument 断言）、`object-events.html`（4 async_test，
error×2/load×2/about:blank instanceof）、`object-handler.html`（静态 object + 嵌套文档脚本回调，
parent callback 通道）。共同根因 = object 动态加载链缺失：

- 无插入触发：createElement('object')+setAttribute('data')+appendChild 后不发起取回、不派
  load/error 事件 → async_test 永挂 → 整文件 Timeout；
- 无 data IDL setter 触发（`obj.data = ...` 落 R5009 URL 反射写臂，纯属性写无加载）；
- historical 断言 contentDocument 非 null（s398 边界保持恒 null）+ contentDocument.location.origin；
- object-events 案 4 断言 `obj.contentWindow instanceof obj.contentWindow.Window`——嵌套 win 无
  Window 槽（_zwMakeIframeWin win 字面量无 name/Window，原型链 _zwChildWinProto37→NPO→ET.prototype）。

object-handler 需嵌套文档脚本执行通道（R206 object 移植面），不属本切片，散单池留存。

## 施工（三 shim 文件 + 两桥测文件）

- `part01.js`：`_zwLoadObjectNavigable` 泛化为 `_zwLoadObjectResource(objKey, sel, handle,
  fireEvents)`——终态事件面：2xx 资源（含 image/* 替换内容）→ trusted load；fetch 失败/非
  2xx/无法渲染类型 → trusted error；事件经 `_zwFireObjectResultEvent`（`fired` 门终态一次 +
  _defer 微任务派发 + __zwMakeTrustedEvent 出口，bubbles/cancelable=false iframe 同款）；
  `about:blank` 特例不 fetch 直建空 doc+win（iframe R130 初始导航语义）；`data:` URL 显式门
  （不物化嵌套上下文——FIXME data:text/html spec 可建，runner 无解码通道；document-getters
  SVG data: URL null 面口径）。新增 `_zwStartConnectedObject(el, connected)`（与
  _zwStartConnectedIframe 同形：_defer 异步启动 + fireEvents=true；消费读不触发事件）。
- `part04.js`：R5274 臂 contentDocument 扩臂（同 `_zwLoadObjectResource` 物化，fireEvents=
  false——读不触发 load；#dom-object-contentdocument 同源活文档/跨源 null）；插入钩子两处
  同排调用（appendChild ceAdded 循环 8466 + ParentNode.append added 循环 11187；parser 扫描
  1290 不做——静态 object 无事件 driving，消费面惰性物化已覆盖）。
- `part05.js`：set trap 新 OBJECT `data` 臂（R5009 URL 反射臂加 `!(OBJECT && data)` 排除
  同款——IFRAME/FRAME/IMG/SCRIPT src 先例；属性写 + isConnected 判定连接态 + 旧 entry 删除
  重启语义；detached 只写属性）；`_zwMakeIframeWin` 原型设置段挂 `win.Window`（named function
  expression，prototype=_zwChildWinProto37——instanceof 沿 win 原型链命中；iframe/object 共用）。
- `part100_s398_object_content_window.rs`：钉①(f) 翻转（contentDocument 恒 null 边界 → 同管线
  物化非 null）+ 头注更新（_zwLoadObjectNavigable 改名 s399 泛化）。
- `part101_s399_object_dynamic_load.rs`（新建，注册 49 号）：钉① 动态链四相位（A 插入钩子
  trusted load 七 flags + 嵌套文档 origin 同源；B 404 trusted error + load 不派；C about:blank
  直建 + win.Window instanceof；D detached data setter 只写属性 + URL 反射 round-trip）；钉②
  重启/边界（E image/* load 不 error + contentWindow/contentDocument 保 null；F data setter 重启
  序列 error,load——无 data 插入静默→404 error→换 text/html 旧 entry 作废再 load）。

## 过程陷阱

- **R5009 URL 反射写臂抢占**（钉② F 面首跑只出 `load`）：OBJECT.data 在 `_REFLECTED_URL_TAGS`
  表内，`obj.data = ...` 落 R5009 写臂（961 行）纯属性写后即返回，下方专用臂永不可达。按既有
  排除模式（IFRAME/FRAME/IMG/SCRIPT src）加 `!(OBJECT && p === 'data')` 排除后两钉全绿。
- **连接态判定误用 `_ceParentConnected`**：该函数对 plain handle 元素查 `_ceConn`（只记
  custom/plain 对象章）恒 false——createElement('object') 永判 detached。script s96 先例同款
  改 `_makeProxy(sel, handle).isConnected === true`。
- **桥测闭包借用**：`let read = |id| sandbox.execute(...)` 闭包持 `&mut sandbox` 跨后续
  execute 触发 E0499（s398 E0596 同族升级版）——改文件级自由函数 `fn s399_read(&mut V8Sandbox,
  id)` 传参调用。
- **join 字段序**：`key\x1fstate` 变换文件 join 须 `-1 1 -2 1`（误用 `-j2` 在 state 列 join
  → 4.1M 叉积垃圾；`\x1f` 不可见致「not sorted」报错文本看似键串粘连，cat -A / python 逐字节
  核验定位）。

## 验证（串行管线，/tmp/s399/ 全量日志）

- shim 拼装：25 文件真实 include 序（js_dom_bridge.rs generate_js_dom_shim 提取）`node --check`
  TRUE ORDER SYNTAX OK。
- driving solo（testharness-html-semantics FILTER=the-object-element）：historical **4P**
  （legacycaller/typemustmatch IDL/typemustmatch content/codebase）、object-events **4P**
  （error file:/error http:/load/load of about:blank，含 instanceof）；对 s398 目录基线位移 =
  2 文件 T→P + 2 文件 T→键级 F（见下）+ 其余案零位移。
- document-getters-return-null-for-cross-origin 回归 solo：exit 0 3P（Object.contentDocument/
  getSVGDocument/Embed.getSVGDocument 跨源 null 面——data: 门 + SVG data: URL 口径保持）。
- 全树 sweep 双跑 join（基线 = s398 cur2-s398.pairs unique 8464 = 8209P/218F/37T，key\x1fstate
  同形态 LC_ALL=C join -1 1 -2 1）：
  - cur1 = cur2 = unique **8473**（cur1 8217P/223F/33T，cur2 8217P/225F/31T）；双跑差异仅
    customizable-select select-home-end 双键 T/F 翻转（s396 已定谳负载时序 flake，真态 F）。
  - 基线位移精确归因：**gone 5T** = 4 个 object 文件级 Timeout 键（historical/object-events/
    allowed-schemas/cross-origin——文件可完成被子测键替代）+ zw-probe-iframe-win-name 1T（s398
    测量期探针残留）；**new 14 键** = 8P（historical 4 + object-events 4，driving 目标）+
    6F（allowed-schemas blob:/data:/http:/https: 4 + cross-origin fallback 2）。
  - **8209 基线 P 零转红零丢失**（P→P 全保持）；6F 为「管线推进后测试体首次可完成」的未竟面
    显形（blob: scheme 无取回通道、跨源 fallback 内容不渲染 DOM）——非既绿案回归，散单池
    留存。allowed-schemas about:/javascript: 2P 基线已有、P→P 保持。
- 单元：`cargo test -p zero-engine -j 2`（test-guard）**3953P/7F**——7F 与 s395/s396/s397/s398
  在册红集完全同形（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 /
  fetch_filtered_response_type_matrix / iframe_service_worker_messages_dispatch /
  response_body_readable_stream_r2967 / t8k_websocket ×2），part100 双钉 + part101 双钉 ok
  在列，零新增红；`-p zero-webview` **869P/2F** 在册同形（image-decoder 子进程 / CSP fetch
  mock）；`-p zero-script-sandbox` **452P/0F** 全绿。
- 产品 smoke：`make product-smoke` PASS；`make product-smoke-legacy` PASS。
- fmt/clippy：`cargo fmt --check -p zero-engine` exit 0；`cargo clippy -p zero-engine
  --all-targets`（test-guard，无 --compile-first——该旗标仅收 cargo test）Finished clean。

## 在册红口径

- 全域 sweep2：unique 8473 = 8217P/225F/31T（225F 含 select-home-end 双键 flake 真态 F 计入；
  31T 含他流渲染探针相关态）。
- **本流域口径 217F+37T → 224F+31T**：driving 8 键 T→P；新显形 6F（blob:/跨源 fallback 未竟面，
  管线成熟代价、散单池承接）；select-home-end 非 detailed 键真态 F 显形 +1F（flake 家族口径
  并账）；-5T 文件级键消失。他流 radio 探针 1F 单列口径不变（224F = 225 - 1）。

## 残差与边界（钉死入账）

- object-handler.html（嵌套文档脚本通道——R206 object 移植面）T 散单池留存。
- object-javascript-url.html T（javascript: URL object 加载语义）保持基线态。
- 新显形 6F 散单池：blob:/data:/http(s): schema object 取回通道（allowed-schemas 4F）+
  跨源失败导航 fallback 内容渲染（cross-origin 2F）——需 fallback 内容 DOM 化管线，中型切片。
- data:text/html data: URL 嵌套上下文物化（FIXME 留 part01）。
- iframe win.name 未实现（browsers 域欠账，s398 已记账不变）。

## 下一步

- 在册红 224F+31T（本流域口径）。下一候选：object fallback 内容渲染切片（新显形 6F 同根因
  承接：spec fallback 步——fetch 失败/error 态渲染子节点内容进 DOM，连带 cross-origin 2F 与
  allowed-schemas blob:/http: 面部分收敛）或本流域散单（focus scroll-into-view 1F / link-rel
  shadow DOM CSSOM）；簇 D（customizable select 特性族）仍待用户决策。
- 挂账池不变：`__zw_resolve_handle_selector` forward map 空返低优先勘根；渲染流 5 面→
  rendering-compat；form-submission-target 6T→navigation M3；sizes-auto lazy 半边待勘根；
  A2+D 簇立项征询待用户决策；radio 探针残留案归渲染流处置。
