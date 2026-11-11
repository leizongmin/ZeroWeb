# M2 切片 406 — frames named access 扩 embed/object + 嵌套导航失败终态（embed 重派 load）

日期：2026-11-11（承 s405 下一步：frames.named 扩 embed/object 面）
commit：见 git log s406 提交

## referent 与勘根

driving：`the-embed-element/embed-network-error.sub.html` 子测 3/4（s405 后余红
2F）——「navigating an existing embed: nonexistent host / X-Frame-Options」：
初载 onload 内换 handler + `frames.existingEmbed1.location.href = <失败 URL>` →
期望第二 load（error unreached）。红态 = `Cannot read properties of undefined
(reading 'location')`——R177 帧代理 named access 只扫 `querySelectorAll('iframe')`
（s405 留档 FIXME 单列）。

双层勘根（桥测探针逐层定位）：

1. **named 通道缺**：R177 frames Proxy 只解析 iframe。embed 无 contentWindow IDL
   → named 值须经 `_zwObjectNavCache` entry.win（s403 物化核）承载。
2. **逗号查询无 pending 回落**（修复首版踩坑）：frames.named 扩展首版用
   `document.querySelectorAll('embed,object')` 树序扫描——桥测实证
   `__zw_query_all` 对**动态插入**元素返空（宿主视图 miss），单 tag 查询靠
   document QSA 的 `_zwPendingAdded` 合并回落存活（part06 tagM161 纯 tag 门），
   逗号形态无该回落 → 恒 0。runner 侧同构（iframe-network-error iframe twin
   全过恰因单 tag 走 pending 回落）。zero-dom 逗号路径本身健康（`split_top_
   level_selector_list` + `collect_matching_chains_opt` 探针 8 变体全对）——
   缺口在 document QSA pending 合并只覆盖纯 tag。
3. **URL 解析失败静默中止**（solo 二层）：`//{{hosts[][nonexistent]}}/` 字面
   （runner 替换表无 `{{hosts[][nonexistent]}}` token——testharness.rs:5634 表
   只有 alt 族）落 `_zwObjectLocation` href setter `new URL()` 抛 Invalid URL
   （host 含禁字 `[`/`]`）→ catch 静默 return → 零导航零事件 → harness 挂起
   （solo `pending:1, phase:4` 实证）。iframe twin 同输入过 = iframe href 写侧
   走 `_zwResolveIframeSrc` 容差（`/` 开头一律 wpt.test 前缀 → fetch 404 → 失败
   语义 → load）。node 双探针定谳（URL THROWS / RESOLVED 前缀）。

embed 失败语义（s405 语料钉面）延伸到导航面：失败 → load 派、error 恒不派
（embed-network-error 四子测同钉）。object 导航失败保持静默（s403 part103
once-terminal 钉面不破；FIXME spec 精确面维持）。

## 施工（part01.js 两面 + 桥测）

- part01.js R177 frames Proxy named access 扩 embed/object：iframe 扫描后追加
  `_zwObjectNavCache` 扫描——`entry.win` 在场 + `_makeProxy(sel,handle).isConnected`
  连接门（同 `_zwWindowLengthCount` 口径）+ live `__zw_get_attr(_handle, 'name')`
  匹配 → 返 `entry.win`（spec named access 值算法「唯一 named 元素有嵌套浏览
  上下文 → 返其 WindowProxy」；无 win 不解析）。扫描走缓存序（≈物化序近似树序，
  同 window[N] FIXME 口径——绕开逗号查询 pending 缺口）。
- part01.js `_zwObjectNavigateEntry` 失败三臂（fetch 空/error wire、非 2xx、
  非 text/html essence）收拢新 `_zwObjectNavFail`：`state='error'` + doc/win 槽
  清空 + embed entry `fired` 门复位重派 `_zwFireObjectResultEvent`（error 态 →
  embed 映射 load——s405 同一出口）；object 静默保持。
- part01.js `_zwObjectLocation` href setter：`new URL` 抛错回落
  `_zwResolveIframeSrc(String(v))` 容差（iframe 写侧同口径），解析仍空 → return。
- 桥测 part106（新文件 + js_dom_bridge_tests.rs 注册）：`frames_named_embed_
  nav_fail_faces_s406` 五断言面——①embed named access 解析（load1:true）+
  导航 404 → 第二 load（load2）+ error 恒不派；②object parity
  （frames.named === contentWindow）；③object 导航失败静默（part103 不破）；
  ④失败后 fallback 态（win 槽清空 → length 0 + frames.named 不再解析）；
  ⑤URL 解析失败容差（`//{{hosts[][nonexistent]}}/` 字面 → 落取回失败 → load）
  + fetch log 逐通道计数（blank×2/blank2×1/missing×2/字面 token×1）。

## 过程陷阱

- **逗号查询 pending 缺口**：首版树序 querySelectorAll 扫描在桥测红
  （dbg 探针 `0|1|1|0|0|0|0`——全部逗号变体 0、单 tag 1）→ 剥到
  `__zw_query_all('embed')` 亦空 → document QSA 的 pending 合并纯 tag 门实证。
  zero-dom 探针（临时 crate 内测试，勘根毕删）证逗号路径健康。改缓存序扫描。
- **桥测 fetch log 断言自误**：`contains("missing.html")` 漏配
  `missing2.html`（object 通道）——改 `/s406/missing` 前缀计数。
- 探针卫生：crate 内 zero-dom 临时探针 + 桥测 dbg 段勘根毕删。

## 验证（/tmp/s406/ 全量日志）

- 桥测 part106 全绿（exit 0）。
- 单元：engine lib **3880P/9F**（HEAD stash A/B 全量 3879P/9F——失败清单逐名
  相等，本切片 +1 新钉通过、零回归）；webview lib **849P/2F**（基线同形）。
- driving solo（the-embed-element 全目录，runner 重编两轮）：**embed-network-error
  4P/0F 全解挂**（4 子测含 nonexistent-host/XFO 两导航面）；目录余 F 逐键同态
  （allowed-schemas 5F has_width 渲染面 + change-src-2 1F SVG rect 渲染面 +
  document/in-object-fallback 族——s405 留档面零位移）。
- 全树 sweep ×2（runner 重编含 WIP）：run1 **8484 keys 8236P/224F/24T**
  （s405-run1 8234P/226F/24T——恰 −2F）、run2 8469 keys 8220P/224F/25T。
- join：run1 对 s405-run1 **恰 2 STATE-DIFF 全为 embed-network-error F→Pass，
  new=0 gone=0**；run2 对 s405-run2 同 2 F→P + select-event/focus 两文件级
  T 塌缩边界（flake 单列）；**跨跑 join-cross state-diff=0**（同二进制零状态
  翻转——迁移确定性成立，s292 协议）。
- fmt：`cargo fmt -p zero-engine -- --check` 干净；clippy `-p zero-engine
  --all-targets -D warnings` 干净。
- 过程注记：sweep-1 的 test-guard 在会话 rollover 时被外部 SIGTERM（exit 143）
  ——runner 孤儿续跑补完全树（8484 keys 与 run2 cross 验证），守卫中断单列
  非掩盖；sweep-2 exit=1 正常有红。

## 口径

- 本流域在册红：**223F+24T → 221F+24T**（−2F = embed-network-error
  navigating-existing 两子测，F→P 双跑一致）。
- engine lib 单测在册红 **9F**：7 既有（img/svg settle、execcommand matrix 外的
  fetch/response/websocket/iframe-worker 族）+ **execcommand_format_apply /
  toggle_state 2F = HEAD 既有跨流红**——stash A/B 三重实证（隔离、全量、单线程
  均 HEAD 2F 确定性），46b582b78 已归因 f415be99c（editing 流 SetInnerHtml 契约
  切换未同步 2 单测）；s405 当轮 7F 为该 commit 落树前时点记录，editing 流认账
  单列，非本切片面。
- sweep flake 单列（非在册红）：select-event.html 整文件 T 塌缩（跨跑 run2 命中，
  run1 全过——16 子测边界翻转）、focus-dynamic-type-change-on-blur 整文件 T
  塌缩（s405-run2 ↔ s406-run2）；均沿 s405 观察池口径续观。

## 残差与边界

- frames 索引面（`frames[i]`/`frames.length`）保持 iframe-only（spec 值应为全
  嵌套 navigable 枚举，window[N] 已含 object/embed——两通道不对称记档 FIXME；
  语料无驱动面不扩刀）。
- document QSA 逗号选择器 pending 合并缺口（动态插入面 `querySelectorAll
  ('a,b')` 恒 miss、宿主视图 miss 后无回落）——共享查询面缺口单列，未在本切片
  扩刀（修复波及面大，语料驱动面 = 本案已绕开）。
- embed 成功导航（location.href → 2xx text/html）不重派 load（s403 success 臂
  近似维持——语料无驱动面；javascript: 臂已有重派语义）。
- `{{hosts[][nonexistent]}}` 替换 token 缺席（runner 替换表只有 alt 族）——失败
  语义经容差前缀达意，精确 host 替换不施工。
- object named access 值 = entry.win（同 contentWindow identity，桥测钉）；
  object 无 navigable 时 frames.named 不解析（spec 值应为元素面——window NA
  系统承载，frames 别名面不扩）。

## 下一步

- 散单续勘（挂账池承 s404/s405：forward map 空返勘根/渲染流 5 面/
  form-submission-target 6T/sizes-auto 半边/ilf+modulepreload 观察池）；
  has_width/SVG rect 渲染面移交渲染域记账；A2+D 簇立项征询仍待用户决策。
