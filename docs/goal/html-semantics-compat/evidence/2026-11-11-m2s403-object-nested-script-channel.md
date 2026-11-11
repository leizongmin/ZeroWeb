# M2 切片 403 — object 嵌套文档脚本通道（R206 移植）+ parser 静态启航（driving 文件级 Timeout → 双子测 P）

日期：2026-11-11（承 s402 下一步：object-handler 嵌套文档脚本通道）
commit：见 git log s403 提交

## referent 与勘根

driving：`the-object-element/object-handler.html`——静态 `<object id=test name=obj
data=test0.html type=text/html>`，嵌套文档脚本 `parent.callback('test0')` +
`document.location.href='test1.html'` 自导航；页脚本断言 contentDocument.location、
`window['obj']`（named access navigable 值 = WindowProxy）、嵌套
`history.replaceState`（data 属性不动）。

语料缺三面通道（s398 在册钉）：

1. **parser 静态 object 无人启航**——`_zwStartConnectedObject` 此前仅动态插入钩子
   接线，解析期 `<object data>` 无启动路径 → 嵌套文档永不物化 → 回调链不执行 →
   文件级 Timeout。
2. **嵌套文档脚本不执行**——`_zwObjectMaterializeDoc` 只建 doc/win/URL 快照，无
   R206 通道。
3. **嵌套导航面缺**——嵌套 `document.location.href` 写、named access morph、嵌套
   history.replaceState 均无物。

静默面决策：sweep 用 fireEvents=false——display:none 双案
（object-in-display-none-load-event / object-image-display-none-loading）钉「不渲染
不派 load」，语料无静态终态事件驱动面 → 静默规避事件回归
（FIXME(object-static-events) 记档 spec 偏差，恢复需 rendered 门）。

## 施工（三文件 shim）

- `part04.js`：`__zwStartParserObjects`（S6 同款扫描位）——querySelectorAll('object')
  逐个 `_zwStartConnectedObject(obj, true, false)`（静默；无 data 跳过；try/catch
  包体）。
- `part06.js`：window load 派发分支 `__zwStartParserIframes()` 后同位接
  `__zwStartParserObjects()`。
- `part01.js`：
  - `_zwStartConnectedObject` 增第三参 `fireEvents`（透传
    `_zwLoadObjectResource`，缺省 true 保动态插入臂原语义）。
  - `_zwObjectMaterializeDoc` 重写：doc `_zwURL` 槽 + `_zwMakeIframeWin`（R206 全参
    window：location/history/navigator/parent=globalThis）+ `__r115SetWin` + win.name
    + **win.location 挂 `_zwObjectLocation`** + named access morph（parse 期
    `_installNamedAccess` 装的是元素 face——spec navigable 值须 WindowProxy，显式
    morph，identity 守卫 element/prevWin）+ **`_zwRunIframeScripts` 接通**。
  - 新 `_zwObjectLocation`（href get 读 `entry.doc._zwURL`；href set URL 解析同值
    no-op、异值走 `_zwObjectNavigateEntry`；protocol/host/pathname/search/hash 面；
    assign/replace/reload 委托）+ `_zwObjectNavigateEntry`（fetch `objnav:<key>:nav`，
    2xx text/html 重物化 + 脚本重放；error/非 2xx → state='error'、doc/win=null——
    fallback 语义；data 属性不动）。
  - **回环修复（过程实证）**：materialize 首行 `entry.state='done'` 先行——嵌套脚本
    执行中的 contentWindow/contentDocument 消费读走 `_zwLoadObjectResource` 早返门；
    'loading' 期落穿 = 重取回 + 重物化自递归（桥测 A 链 170 连 fetch 实证）。

## 过程陷阱

- **materialize 期 state 门落穿自递归**：初版脚本执行发生在 `state='loading'` 窗口
  内，嵌套脚本读 contentDocument → 消费臂落穿 → 全链重跑 → 回调链
  `href0:false,na:true,rs:true,href1:true,data:true` 循环 ~145 轮、fetch log ~170
  test0 + ~145 test1。修 = done 先行（各臂尾部 state 赋值成幂等 no-op）。
- **桥测读回 join 符**：`s403_read(sandbox, "")` 返数组逗号 join——断言串用 `|`
  连接须显式 `.join('|')`（读达式 `"__s403" + id` 拼接，id 传 `".join('|')"`）。
- **test-guard `--compile-first` 仅支持 `cargo test`**——`cargo clippy` 挂该旗标被
  拒（「仅支持 cargo test 命令」），去旗标重跑。
- **sweep 负载污染归因**：首轮 sweep ×2 与 ZeroSeed test-all、ZeroWeb-2 并发
  （load 227 vs 16 核）——selopt 文件键 Fail→Timeout、ilf Pass→Fail 疑似本切片
  回归。idle 复测 + **stash 二分（no-WIP 二进制 idle ×3）**定谳：selopt Timeout =
  负载假象（idle 复基线 Fail 形），ilf = **环境漂移**（no-WIP 同红 ×3，非本切片
  引入）；真回归检验必须 no-WIP 对照跑，单跑 with-WIP 不足定谳。

## 验证（/tmp/s403/ 全量日志）

- 桥测 ×2（part103）：`object_nested_script_channel_s403`（回调链五断言序
  href0/na/rs/href1/data 全 true + 静默事件空 + 终态 doc 停 test1 + fetch 恰两取回
  + fallback 零取回）+ `object_sweep_silent_idempotent_nav_error_s403`（二次 sweep
  零重取回 + 导航 404 fallback 语义 + 事件恒空）全绿。
- 单元：engine lib **3877P/7F**（7F = s402 在册逐名同形：img settle ×2/fetch
  filtered/SW iframe/response body R2967/websocket ×2）——+2 新钉全绿。
- driving solo：object-handler **文件级 Timeout → 双子测 Pass**；
  余红 3 案（object-javascript-url 文件级 T / object-constraint-validation F /
  object-in-object-fallback-2 F）s402 基线同态。
- 全树 sweep ×2（idle，基线 = s402 base 8475 pairs 8227P/218F/32T raw）：
  - 稳定集：**object-handler ×2 NEW Pass**（双跑一致）+ link-rel shadow 案
    Fail→Pass（s402 修复在册复测确认）。
  - ilf Pass→Fail：**环境漂移单列**（stash 二分 no-WIP ×3 同红钉证，非本切片；
    成因未究，挂观察）。
  - flake 振荡（基线已红/已属在册家族）：selopt Fail↔Timeout + 子测形态翻；
    dialog-overlay Pass↔Fail 跨 run 单现；checkable-active-onblur run1 单现。
  - GONE：object-handler 文件级 T（胜）+ selopt 文件键（形态翻）+
    zz-probe-shadow-gcs（s402 探针已删，单列出账）。
- fmt：`cargo fmt -p zero-engine` 干净；clippy：`cargo clippy -p zero-engine
  --all-targets -- -D warnings` clean。
- 渲染面：sweep 静默（无渲染通道变化），产品 smoke 不适用（同 s398 口径）。

## 口径

- 全树 sweep（idle）：base 8475 pairs（8227P/218F/32T）→ idle ×2
  8228P/217-221F/31-32T（selopt/ilf/checkable/dialog-overlay 形态振荡 raw 浮动）。
- **本流域在册红 215F+31T → 215F+30T**（−1T：object-handler 文件级 Timeout 化双
  Pass）。环境漂移 ilf +1F 单列观察（no-WIP 复现钉证，不入切片口径；下轮基线刷新
  决定是否吸收入册）。

## 残差与边界（钉死入账）

- 静态 object 渲染终态事件不派（FIXME(object-static-events)）——display:none 双案
  钉面，恢复需 rendered 门 + 事件面。
- `_zwObjectLocation` hash setter 为 URL 槽近似（无浏览器级 fragment 导航），FIXME
  单列；origin 按当前 URL 推导无 about:blank 特例（语料无钉面）。
- 嵌套导航 `objnav:` fetch 通道不走 R56h 图片 inflight 闸门（text/html 语义，无
  延迟 load 面）。
- 嵌套文档 window.length/frames 面未接（s400 已钉 window.length 0 近似在册）。
- ilf 环境漂移成因未究（no-WIP 同红）——挂观察池，下轮 sweep 前基线刷新复核。

## 下一步

- 在册红 215F+30T。下一候选：本流域散单续勘（挂账池：forward map 空返勘根/渲染流
  5 面/form-submission-target 6T/sizes-auto 半边/A2+D 簇立项征询——簇 D 仍待用户
  决策）；或 object-javascript-url 勘根（本轮 solo 余红最近邻）。
