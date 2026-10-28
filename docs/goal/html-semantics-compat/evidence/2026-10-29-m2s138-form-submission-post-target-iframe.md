# M2 s138 · form-submission-0 186F 构成面勘根 + 提交主干施工——urlencoded2 62F→26F（36 F→P）+ targetted-form 族文件级 Timeout 解锁（全量零坏位移）

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s137（FORM named properties）

## 入场定刀位

控制面 s138 候选②：form-submission-0 186F（urlencoded2 /
text-plain / multipart-formdata 三 .window.js 各 62F，s137 全量
基线最大单一失败池）构成面勘根。

## 构成面勘根定谳

**单一主干（186 案全部，失败形态一字不差 `assert_equals …
but got ""`）**：

1. **POST 零投递**：`_zwFormSubmitNavigate`（part04）POST 未拦截
   分支 `if (_isPost) return;`——host 导航契约无 method/body 面
   （M2-S4J documented 边界），提交后什么都不发生。
2. **form target 无解析**：提交通道只走顶层文档路径，`target`
   属性从未被读取——named iframe 目标不存在。
3. **iframe POST 装载通道缺**：`_zwLoadIframeEntry`（part01）对
   `__zw_fetch` 硬编码 `'GET'` + 空 body（位置 1/4）。
4. **echo 端点缺**：`/FileAPI/file/resources/echo-content-escaped.py`
   在 wpt-data 不存在（FileAPI 通道扫描按 `.py` 内容标记 skip，
   该目录从未拉取；runner fetch handler 无此路由）——enctypes
   helper 的 ACTION_URL 落空。

`but got ""` 的来源：helper 在 `form.submit()` 后挂
`formTargetFrame.onload`，iframe **初始 about:blank 装载**先触发
onload（提交本身零导航），`contentDocument.body.textContent`
即空串——assert 失败而非 Timeout 的原因。

**分层构成（主干之外的附加层）**：

- **enctype 编码器**：urlencoded 已有（GET query 复用同段）；
  text/plain 与 multipart 序列化缺。
- **File 条目层**：file input 在 entry list `continue; // FIXME`
  零贡献（normal form 通道 got ""）；FormData.append 把 File 值
  `String(v)` 成 `"[object File]"`（formdata 通道实测 got
  `a=%5Bobject+File%5D`）。
- **accept-charset 层**：windows-1252 编码 + `&#NNNN;` 实体回退
  缺（w1252 三案实测落 UTF-8）；lone surrogate 另需 USV 转换
  步（实测整个序列化抛错 → 提交中止 → got ""）。

## 施工（3 文件：runner 1 + shim 2）

- **runner 端点**（testharness.rs `wpt_data_fetch_handler` 新
  路由）：`FileAPI/file/resources/echo-content-escaped.py`——
  POST body 逐字节转义为 `\xNN` 文本（可打印 ASCII 原样；
  `\` 自身转义 `\x5C`），text/plain 交付，行为与上游 wptserve
  端点对 helper `unescape()` 可逆等价。附直测（round-trip +
  `\x00`/CRLF/backslash 形态 + 空体 GET）。
- **part01 `_zwLoadIframeEntry`**：flags.method / flags.body /
  flags.headersWire 透传 `__zw_fetch` 位置 1/3/4（fetch_bridge
  decode 同序）；缺省 GET 空 body 语义不变，既有 src 导航零变化。
- **part04 `_zwFormSubmitNavigate` ⑤b target 分支**（插在 GET
  query 突变后、顶层 navigate 事件前）：submitter formtarget >
  form target 属性；命中 named iframe（`querySelectorAll('iframe')`
  + name 内容属性匹配，R177 同款查询面）→ GET 带 query / POST
  带 body + enctype Content-Type（submitter formenctype > form
  enctype > 缺省，非法值回 urlencoded——headers wire `\x1e` 对，
  fetch_bridge encode_headers 同格式），经 `__zw_reload_iframe`
  复用 iframe 装载管线（load 事件管线微任务派发，R288），return
  不入顶层 navigate/历史；未命中名与 `_self/_parent/_top/_blank`
  落既有顶层行为。
- **`_encPair` 编码集修正**（rider，同函数）：原注释称
  encodeURIComponent 与 spec 编码集「除 `*` 集合差异一致」有误
  ——encodeURIComponent 的 unescaped 集还含 `! ' ( ) ~`，而
  spec application/x-www-form-urlencoded percent-encode set 仅豁免
  `*-.0-9A-Z_a-z`（https://url.spec.whatwg.org/#urlencoded-serializing）
  ——补编 `%21 %27 %28 %29 %7E`（`*` 两侧均字面量无需处理）。
  GET query 与 POST body 共享此函数，同向对齐。

## 验证

- 定向 urlencoded2：**36P/26F**（62F → 36 F→P；26 残余 = 22
  File + 4 windows-1252，全挂账层）。
- 定向 form-submission-0 全目录：键级 join 零坏位移（无任何
  Pass→Fail / Timeout→Fail）；urlencoded2 36 F→P 之外，
  form-double-submit 家族多文件出现**新见键**（文件级 Timeout
  解除后子测试首次出结果）。
- 全量 join（对 s137 full log）：7914 → 7981 行，**配平完全
  闭合**——36 F→P 真翻红（P 6814+36+14=6864 ✓）+ 72 新见键
  （14P/58F；F 996−36+58=1018 ✓）+ 文件级 T 102→97（−5 ✓，
  5 文件 Timeout 行解除）+ 行数 +67（72 新 − 5 解除）✓。
  **零坏位移**（无 P→F / T→F / P→T）。新见键构成：
  targetted-form.js 族（`<form target="named-iframe">` 提交面，
  与主干同构）——form-double-submit-requestsubmit +36（1P/35F）、
  form-submit-action +20（8P/12F）、form-double-submit-default-action
  +5、preventdefault-click +3、其余四文件 +2 各——58 新见 F =
  主干解锁后首次可见的 double-submit 防护/提交语义深层面
  （非回归，先前藏于文件级 Timeout）。
- engine lib **3606/0**（并行流并入后新基线）；runner bin
  **256/1F**（76i，+1 = 新端点直测；唯一 F = 既有
  trend_csv_check encoding CSV 行错位，归口 encoding 流不变）；
  clippy（zero-engine + zero-wpt-runner --all-targets -D
  warnings）零告警；fmt runner 过——fmt engine 报
  `js_dom_bridge/url_idna.rs` 既有违例（并行流 14dbfb011 引入，
  非本轮改动面，不越权代写归口该流）；shim 拼接（13 part 按
  js_dom_bridge.rs 拼接序）node --check 过；语料零改动；无探针
  页（定向面即语料本身）。

## 挂账（本切片更新）

- **form-submission-0 残余构成**（s139 候选刀位，按量序）：
  ①text/plain + multipart 编码器（两文件 124 案中非 File 部分，
  multipart 需 boundary/binary 面）；②File 条目层（file input
  entry 贡献 + input.files setter/DataTransfer 面 + FormData
  append File 通道——urlencoded2 File 案期望仅 filename 序列化）；
  ③targetted-form 解锁的 58 新见 F 深层面勘根（double-submit
  防护族为主）；④accept-charset windows-1252 + `&#NNNN;` 实体
  回退 + lone surrogate USV 转换（urlencoded2 4 案）。
- **fmt：engine url_idna.rs 既有违例归口并行流**（14dbfb011，
  本轮不代写）。
- 承袭不变：「Named elements should override builtins」1F 挂
  待决（须用户拍板）；encoding-legacy-mb CSV 行错位归口
  encoding 流（runner bin 唯一 F）；RadioNodeList live 语义近似、
  past names 条目不显式清理、expando store/target 双轨、part03
  raw 链 type 读通道未勘 + reset 臂无还原臂、engine lib details
  flake（本轮 3606/0 未复现）、reset.html form element pointer
  面（待决缓行）、execution-timing 构成面、lazy-loading 26F、
  meta color-scheme 21F、写流盖章断链、part02 setAttribute 绕
  notify、041 nested write、deep clone 非 HTML ns、css-module
  corpus 门（待决）、www2 可达性、module-import-referrer TLA、
  dynamic import() with 透传、classic 外链 SRI、143 write、
  defer 多写完成序、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt、
  FileList 缺索引访问、live 树逗号列表 qSA 根缺口、渲染流面
  让位累积、s131 插入步缺口细化、radio 现象⑤/checked 属性增设/
  defaultChecked 无独立 corpus 面、s135 已知限制 3 条；
  **待决（须用户拍板，既有）**：css/text 数据模块语料资源缺失、
  green-2x2.png 语料资源分发、parser form element pointer 4F
  缓行（2026-10-23）。

## 过程教训

- **echo-content-escaped 的 backslash 转义须 `\x5C` 而非 `\\`**：
  helper `unescape()` 的 `\xNN` 还原先于 `\\`→`\`——若 backslash
  转义为 `\\`，body 字节 `\x41`（`\`+`x41`）会产出 `\\x41`，
  第一个 replace 把 `\x41` 吞成 `A`，还原结果错位。转义方案
  必须按消费方正则的**应用顺序**设计，不是对称可逆就行。
- **encodeURIComponent 与 spec urlencoded percent-encode set 的
  差异不止 `*`**：`! ' ( ) ~` 同样豁免于前者而 spec 要求编码。
  既有注释断言「一致」未被实测推翻过——语料跑到 `'` 案才暴露。
  共享编码器改动先对 spec 编码集清单逐字符核对。
- **文件级 Timeout 的解锁会让 F 计数上升**：主干打通后
  targetted-form 族首次真正跑子测试，58 个先前不可见的 F 浮出。
  F 上升 ≠ 回归——配平靠行数守恒 + 状态迁移分解（36 F→P +
  72 新见），只看 F 总数会把解锁误判为退化。
- **targeted 与 full 的 join 只做方向性证据**：文件级 Timeout
  的文件在定向跑中出全案子测试、在全量跑中只有单行 Timeout，
  两侧键集天然不对齐——权威对账必须全量对全量（本轮再次
  确认 run-rules 落盘纪律）。
