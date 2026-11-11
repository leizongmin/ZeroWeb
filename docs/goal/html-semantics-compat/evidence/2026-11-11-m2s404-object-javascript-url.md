# M2 切片 404 — object `window[N]` 索引访问 + javascript: URL 导航面（driving 文件级 Timeout → 三子测 P）

日期：2026-11-11（承 s403 下一步：object-javascript-url 勘根，solo 余红最近邻）
commit：见 git log s404 提交

## referent 与勘根

driving：`the-object-element/object-javascript-url.html` 三子测——

1. `javascript:"foo"` 于 data 属性 → 无 load（「should do nothing」）；
2. 已载 object `window[0].location.href='javascript:"test"'` → 完成值字符串
   建文档续导航 → **第二 load 事件**；
3. `window[0].location.href='javascript:1'` 非字符串完成值 → 无 load、文档不动。

文件级 Timeout 双层根因（s227 同族勘根先行——先验语料资产再验引擎）：

1. **语料资产缺席**：`insertObjectNavigable` 以 `data='/resources/blank.html'`
   导航，本域语料 resources/ 仅 testharness 基建 5 件——blank.html 缺 → 404 →
   error 代替 load → 子测 2/3 首个 `await onload` 恒挂（state dump
   `{"loaded":true,"pending":2,"phase":4,"tests":3}` 证挂起非 TypeError）。资产已
   补（gh api pin 31597693 渠道，379B md5 04daab423…，goal 脚本 ③段挂
   fetch_raw 常驻重拉 + update-wpt-data.sh 挂账注释）。
2. **`window[N]` 索引通道缺**：named access `_installNamedAccess` 正则过滤数字
   属性从不安装，`window[0]` undefined → TypeError 快败（资产补后显形面）。
3. **javascript: 导航臂缺**：`_zwObjectNavigateEntry` 直落 fetch 通道，
   javascript: URL 无求值面。

embed 孪生案（embed-javascript-url.html 同构 1P+文件 T）不同径：embed 无插入
钩子/加载链/嵌套 navigable，s405 候选不混本刀。

## 施工（三面 shim + 语料资产）

- `part01.js` `_zwWindowIndexedWin(idx)` + `defineProperty(globalThis,'0'..'15')`
  索引层（own accessor，enumerable:false 防 for-in/keys 漏出、configurable:true
  保页面数值全局可遮蔽）——活跃集合与 `_zwWindowLengthCount` 同口径（iframe
  _iframeDocCache 未 destroy + object _zwObjectNavCache 已连元素），物化序≈树序
  近似 FIXME（语料单嵌套无乱序钉面；WindowProperties 异对象层不可整体 Proxy 化）。
- `part01.js` `_zwLoadObjectResource` javascript: data 属性臂——静默 fallback
  （state='error'，error 事件从同臂族 data: 解码失败/blob: 跨源；子测1钉「无
  load」；FIXME：spec 网络错误语义精确面——完成值字符串虽建文档，语料仅钉无
  load，事件面无钉）。
- `part01.js` `_zwObjectNavigateEntry` javascript: 臂——`#javascript-protocol`
  求值通道：opaque path **percent-decode**（href setter 先经 URL 解析序列化
  `%22test%22`，spec 明确 percent-decode 后作脚本源；decodeURIComponent 残缺 %
  序列抛错回落原文）→ R206 形参遮蔽 + 函数体内 `return eval(body)` 完成值透出
  （window/self/parent/top/document/location/navigator/history 遮蔽到嵌套上下文，
  其余全局回落页面 realm FIXME 近似）→ 字符串完成值：`entry.fired=false` 复位 +
  `_zwObjectMaterializeDoc`（URL 槽不动）+ `_zwFireObjectResultEvent`（导航完成
  恒派 load，navigate entry 无 fireEvents 参数，无条件语义 spec 对齐）；非字符
  串/脚本抛错 → 静默中止（doc 不动、无事件；FIXME：spec report the error 近似）。
- 语料：`tests/wpt-runner/wpt-data/resources/blank.html`（gitignored 本地数据，
  上游 pin 315976933870b34d6ea30e3f6643403edae678ba）+ `25-html-semantics-compat.sh`
  ③段 `fetch_raw "resources/blank.html"` 常驻重拉条目 + `update-wpt-data.sh`
  挂账注释（升级新 ref 后需重新补）。

## 过程陷阱

- **`--compile-first` 仅支持 cargo test**（s403 已入册，本轮 solo 管道重踩一次
  ——cargo build 挂旗标被拒，去旗标重跑后入管道脚本固化）。
- **桥测 J4 断言过强**：跨 javascript: 导航钉 `window[0]` identity ——重物化建
  新 win（`_zwMakeIframeWin` prevWin 通道不保 WindowProxy identity，s403 同型，
  named access 重 morph），corpus 断言面均为 fresh 读不触该面。修：改 live 解析
  等价断言（`window[0] === o2.contentWindow` 双侧 fresh）+ FIXME 记档 shim 近似。
- **pairs/join 工具瑕疵**：s403 join.sh A-ONLY 段 echo 未引号括号杀 GONE 枚举
  ——s404 join.sh 修正版（引号包字面量）。

## 验证（/tmp/s404/ 全量日志）

- 桥测 ×2（part104）：`window_indexed_access_faces_s404`（iframe+object 物化序
  索引 + length=2 + 越界 undefined + descriptor get/enum:false/cfg:true + keys
  不漏 + 移除动态重排 [1]→[0]→undefined + 索引/length 读零 fetch）+
  `object_javascript_url_faces_s404`（data=javascript:"foo" error 无 load 零
  fetch + window[0]=contentWindow + 完成值字符串重物化 load 1→2 + 内容
  percent-decode 落 doc + URL 槽停 blank.html + javascript:1 静默中止无第三
  load + obj2 初载恰一取回）全绿。
- 单元：engine lib **3879P/7F**（7F = s403 在册逐名同形：img settle ×2/fetch
  filtered/SW iframe/response body R2967/websocket ×2——失败清单 diff 逐名
  相等）+2 新钉全绿；webview lib 849P/2F（image_decoder proxy/csp_gate）基线同形。
- driving solo：object-javascript-url **文件级 Timeout → 三子测 Pass**（三断言
  全中 corpus 预期——do nothing / fire load / not fire load）。
- 全树 sweep ×2（idle，基线 = s403 idle pairs 8478/8475）：见下节落账。
- fmt：`cargo fmt -p zero-engine` 干净；clippy：`cargo clippy -p zero-engine
  --all-targets -- -D warnings` clean（test-guard 无 compile-first）。
- 渲染面：sweep 静默（无渲染通道变化），产品 smoke 不适用（同 s398 口径）。

## 口径

- 全树 sweep（idle）：s403 idle 基线 8478/8475 pairs（8226P/217-221F/31-32T raw）
  → 本轮 ×2 8476 pairs（8229P/217F/30T + 8228P/217F/31T raw）。
- 稳定集位移（双跑一致）：**object-javascript-url 文件级 T 键 GONE +
  `location.href = 'javascript:"test"' should fire a load event` / `javascript:1'
  should not fire a load event` 两子测 NEW Pass**（子测1 data 属性 do-nothing 基线
  已 Pass——文件挂起前已上报；本轮三子测齐报）。runner exit=1 为带失败退出码口径
  （s403 同值，非截断）。
- flake 单列（在册家族）：selopt 家族形态翻（detailed 文件键 T↔F 跨跑 + optional
  文件键 run1 单现文件级 T——s387 在册完成深度截断族）；checkable-active-onblur
  Fail→Pass（双跑 P——s403 在册振荡族落有利侧，本切片无因果路径）；dialog-overlay
  Fail→Pass（s403 在册 Pass↔Fail 跨跑族）。
- modulepreload-referrerpolicy Pass→Fail：同二进制跨跑翻转（run1 P/run2 F——s292
  判据=时序 flake），单列新观察挂池非本切片。
- **本流域在册红 215F+30T → 215F+29T**（−1T：object-javascript-url 文件级 Timeout
  化三子测 Pass；F 口径零位移）。embed 孪生文件 T 保持（s405 候选）。
- 回归面：object-handler 2P 保持（s403 修复在册复测确认）；object-allowed-schemas
  六键全 P（`javascript: scheme not allowed` error 期望面经新 load 臂保持）。

## 残差与边界（钉死入账）

- `window[N]` 索引序 = 物化序近似树序（FIXME）；16 上限截断（语料无深嵌套钉面）。
- 跨导航 WindowProxy identity 不保（重物化建新 win）——corpus fresh 读不触，
  FIXME 记档。
- javascript: 求值 realm 回落页面 realm（FIXME）；脚本抛错静默无 console 上报面。
- data 属性 javascript: 的 error 事件面：spec 精确语义（网络错误 vs 求值建文档）
  语料仅钉无 load——从同臂族派 error，FIXME。
- object-allowed-schemas `javascript:'x'` → error 期望面保持（新臂 error 事件
  同形，solo 口径见 sweep）。
- embed 孪生（embed-javascript-url.html）文件 T 不动——s405 候选（embed 动态
  加载链 s399 镜像 + javascript: nav + window[0]）。

## 下一步

- 在册红 215F+29T。下一候选：**s405 embed 动态加载链**（s399 镜像——动态 insert
  → 加载链 → javascript: nav 全链，embed twin 文件 T 唯一施工解）；或本流域散单
  续勘（挂账池承 s403：forward map 空返勘根/渲染流 5 面/form-submission-target
  6T/sizes-auto 半边/ilf 环境漂移观察池 + modulepreload 时序 flake 新观察）；A2+D
  簇立项征询仍待用户决策。
