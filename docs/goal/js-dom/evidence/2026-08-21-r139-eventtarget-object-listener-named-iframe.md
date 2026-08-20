# R139 — EventTarget 对象 listener 全链 + named iframe window 注册 + Text 非 bubbling（events 28F→22F）

**日期**: 2026-08-21
**里程碑**: M4（WPT dom 上游基线建立与扩展）
**Driving 用例**: `dom/events/EventListener-handleEvent-cross-realm.html`（5F→0F）+ `dom/events/Event-dispatch-click.html`（bubbles 1F→0F）
**运行入口**: `make testharness-dom FILTER=handleEvent-cross-realm` 等

## 根因（四层，探针实证链）

1. **EventTarget.addEventListener 拒收对象 listener**——旧版
   `if (typeof cb !== 'function') return` 使 WebIDL EventListener callback 的
   对象形态根本不注册（cross-realm 全簇的前置根因；后续 dispatch 逻辑再对也
   无从生效）。
2. **dispatch 循环无 handleEvent 分派/上报**——`arr[i].call(target, event)`
   对对象 listener 抛 "not a function" 且 `catch (_) {}` 全吞（spec inner
   invoke 步骤 1-2 的 handleEvent Get + 非 callable TypeError report 缺失；
   revoked Proxy 的 Get 抛与 call 抛同样须 report）。
3. **named iframe 的 window named access 缺失**——`<iframe name=x>` 应使全局
   `x` 解析到其 contentWindow（HTML spec）；cross-realm 用例的
   `eventListenerGlobalObject.Object/TypeError/Proxy/addEventListener` 全部
   依赖它。且 lazy 注册（首读 contentWindow 时）对「load listener 内直接读
   全局名」来不及。
4. **Text.dispatchEvent 无条件转父**——`p.dispatchEvent(event)` 使父成为新
   target，pre-click activation 从父 INPUT 起找命中父自身 → 非 bubbling 的
   Text click 也翻转父 checked（spec：非冒泡 path = [target]）。

## 修复（四处）

1. **part05 EventTarget.prototype.addEventListener**：接受对象 listener
   （null/undefined 忽略），callable 判定移到派发点。
2. **part05 EventTarget.prototype.dispatchEvent 循环**：对象 listener 的
   handleEvent 分派（this=对象）+ Get 抛/非 callable → TypeError 经
   `_zwReportListenerError` 上报（part03 导出 globalThis）+ handleEvent
   call/函数 call 抛按 spec report the exception。
3. **part04 + part06**：iframe contentWindow 的 lazy 全局注册（两分支：src
   加载 win + no-src fallback win）+ window dispatchEvent 的 load 分支
   `__zwRegisterNamedIframes` 派发前一次性物化 + `_zwMakeIframeWin` 补
   Object/Function/Array/Error/TypeError/Proxy 与
   addEventListener/removeEventListener 转发。
4. **part03 Text.prototype.dispatchEvent**：仅 `event.bubbles` 时转父。

## A/B 验证

- **cross-realm**：5F→**0F（5P 双路径 100%）**；**dispatch-click**
  bubbles 用例 1F→0F。
- **events 全量**：28F→**22F 双路径一致**（polyfill 423P / native 424P
  附近，fail 集逐行 diff 空）。
- **涟漪收益**：traversal 1589P/15F→**1594P/10F（+5）**——EventTarget
  对象 listener 修复使 NodeIterator 用例同源受益。
- **nodes**：8464P/188F 双路径一致零回归；collections 49P 不变。
- **单测**：`test_eventtarget_object_listener_r139`（6 断言段：对象
  listener 注册+分派 this / handleEvent 缺失 TypeError 上报 / revoked
  callable Proxy call 抛上报 / Text 非 bubbling 不触发父 activation +
  bubbling 对照）。**首版教训**：用 getElementById（sel-based）input 时
  Text 无 R84 反链（R84 接链条件是 handle 父），改 createElement
  handle-based 复现 WPT 形态后通过。
- `make test` 66 套件全绿；fmt 无 diff；clippy 双矩阵零警告。

## 教训

1. **「注册即过滤」是对象 listener 类用例的第一道暗墙**——派发点逻辑再
   完整，注册点 `typeof !== 'function'` 早退就使一切后续修复失效；对象
   形态的接受/拒绝判定要放派发点（spec 的 callback 转换发生在 invoke）。
2. **catch-吞错与 spec「report the exception」语义相反**——listener 异常
   不传播但**必须上报**（error 事件 + onerror）；全吞等于对「跨 realm
   TypeError identity」类断言不可见。
3. **lazy 注册有初始化竞态**——按需注册（首读才发生）对「事件回调内直接
   读全局」形态永远迟到；named access 这类页面级不变量应在 load 派发前
   一次性物化。
4. **代理转发使 target 漂移**——`p.dispatchEvent(event)` 不是「冒泡」而是
   「重派发」（target=父）；真正的冒泡须保留原 target。Text 无自身派发
   面，转发的条件就是 `event.bubbles`。
5. **单测环境要贴用例形态**——sel-based 与 handle-based 元素对子节点
   反链（R84）行为不同；测试选形态与 WPT 用例不一致时会得到假失败。
