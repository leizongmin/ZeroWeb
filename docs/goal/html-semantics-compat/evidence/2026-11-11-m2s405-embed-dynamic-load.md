# M2 切片 405 — embed 动态加载链（s399 镜像：插入钩子启航 + 失败映射 load + javascript: 全静默 + src 重启臂）

日期：2026-11-11（承 s404 下一步：s405 embed 动态加载链）
commit：见 git log s405 提交

## referent 与勘根

driving：`the-embed-element/embed-javascript-url.html` 三子测（s404 object 孪生案
同构）——①`javascript:"foo"` 于 src 属性 → 无事件；②已载 embed
`window[0].location.href='javascript:"test"'` → 第二 load；③`javascript:1` → 无
load。文件级 Timeout 根因 = embed 元素**零加载面**（无插入钩子/无加载核/无嵌套
navigable；named access 正则滤数字属性 → window[0] undefined）。

the-embed-element 目录勘根（s404 sweep 在册 5 文件级 T 全指向缺加载链）：
embed-javascript-url / embed-allowed-schemas（load+offsetWidth>0）/ embed-change-src
（src 更新重启）/ embed-change-src-2（data:svg + rect 尺寸）/ embed-network-error
（失败语义 + frames.named）。四文件差异面定界 s405 施工口径：语料钉 embed 失败
语义 = **load 派、error 恒不派**（embed-network-error 四子测 onload=done /
onerror=unreached）——与 object（error 事件）相反；javascript: src = 全静默
（子测1「should do nothing」）。/media/white.mp4/.webm（change-src 引用）资产缺席
不阻——失败面按语料语义映射 load，事件序断言（reload on src change）与资源身份
无关。

## 施工（三文件 shim + 桥测）

- `part01.js` `_zwLoadObjectResource` 五参化（+`isEmbed`）：entry 增 `embed` 印记；
  属性读 `data` → `isEmbed ? 'src' : 'data'`；javascript: 臂 `fireEvents &&
  !entry.embed`（embed 全静默——非取回失败不走「失败→load」语义）。
- `part01.js` `_zwFireObjectResultEvent` 事件映射：`state==='error'` →
  `entry.embed ? 'load' : 'error'`（embed display no plugin 语义，语料钉面）。
- `part01.js` 新 `_zwStartConnectedEmbed(el, connected, fireEvents)`（镜像
  `_zwStartConnectedObject`：EMBED tag 门 + 共享加载核 isEmbed=true）——共享
  `_zwObjectNavCache` → s404 `window[N]`/s400 `window.length` 活跃集合自动计入。
- `part04.js` 两插入路同排（appendChild 融合路径 + append 变参路径）
  `_zwStartConnectedEmbed(added[ci], cePconn)`。FIXME(embed-static-events)：parser
  静态扫描不接（语料驱动案全动态插入；静态 embed Pass 案
  document-getters-return-null-for-cross-origin 零触碰——零回归实测确认）。
- `part05.js` R5009 排除 `EMBED.src` + 新重启臂（镜像 s399 OBJECT.data：attr 写 +
  isConnected 门 + 旧 entry 删除重建 + `_zwLoadObjectResource(..., true, true)`）。

## 过程陷阱

- **桥测 L1 断言自误**：`load/0` 期望写成双 insert 后 length——emb4（data:text/html）
  同期已物化应计 1；error 态不计由 window[0] 身份面（bp2）承载。修 `load/1`。
- **负载污染窗口**：sweep 启动时 15min load 91（并发会话突发）——1min 回落 12 后
  放行，join 位移按双跑稳定集 + 在册 flake 家族归因（s403 协议）。

## 验证（/tmp/s405/ 全量日志）

- 桥测 ×2（part105）：`embed_javascript_url_faces_s405`（javascript: 全静默零事件
  + window[0] 嵌套上下文 + 完成值重物化 load 1→2 + window[0].document 读面（embed
  无 contentDocument IDL）+ URL 槽不动 + javascript:1 静默中止 + 零 fetch）+
  `embed_failure_load_restart_faces_s405`（404 → load 派 error 不派 + error 态不计
  索引 + data:text/html 物化 + src IDL 重启 fired 门复位第二 load + doc 换新）全绿。
- 单元：engine lib **3881P/7F**（7F = 在册逐名同形，失败清单 diff 逐名相等）+2 新
  钉全绿；webview lib 849P/2F 基线同形。
- driving solo（the-embed-element 全目录）：**5 文件级 Timeout 全解挂**——
  embed-javascript-url 3P（driving 全中）+ embed-change-src 1P（重启臂兑现，
  /media/ 资产不需要）+ embed-network-error 2P+2F（余 F = frames.named
  `existingEmbed1/2` 通道缺——R177 帧代理 iframe-only，s405 不施工单列）+
  embed-allowed-schemas 5F（load 派 ✓，余 F = has_width `offsetWidth>0` 渲染面——
  UA 默认尺寸/replaced 布局，渲染流域）+ embed-change-src-2 1F（load 派 ✓，余 F =
  SVG replaced content rect 尺寸渲染面）。目录余 13 键逐键同态零回归（document-
  getters 静态 embed getSVGDocument null 面保持 P）。
- 全树 sweep ×2：见下节落账。
- fmt：`cargo fmt -p zero-engine` 干净；clippy：`cargo clippy -p zero-engine
  --all-targets -- -D warnings` clean。
- 渲染面：本切片 shim 语义面（无布局/绘制通道变化），has_width/rect 新显形 F 归
  渲染域，产品 smoke 不适用（同 s398 口径）。

## 口径

- 全树 sweep（idle 域）：s404 idle 基线 8476/8476 pairs（8229P/217F/30T + 8228P/
  217F/31T raw）→ 本轮 ×2 8484 pairs（8234P/226F/24T）+ 8454（8202P/226F/26T raw
  ——run2 含 2 文件负载截断塌缩，见 flake 单列）。sweep 启动遇并发负载突发（15min
  load 91），1min 回落 12 放行。
- 稳定集位移（双跑逐键一致，13 NEW + 5 GONE 恰 embed 族）：**5 文件级 Timeout 键
  双跑 GONE**（javascript-url/allowed-schemas/change-src/change-src-2/network-error）
  + **13 子测 NEW**（javascript-url 2P + change-src 1P + network-error 2P+2F +
  allowed-schemas 5F + change-src-2 1F）——runner exit=1=带失败退出码口径非截断。
- flake 单列（在册家族/负载形态）：selopt 家族文件键 T↔F 形态翻（optional/detailed
  跨跑双态，s387 在册截断族）；modulepreload-referrerpolicy F↔P 振荡（s404 观察
  的时序 flake 本轮回摆，双态不入集）；run2 单现 textfieldselection/select-event +
  focus-dynamic-type-change-on-blur 整文件 T 塌缩（35+子测 GONE——cross join 证
  run1 基线同态，同二进制跨跑翻转=s292 时序 flake；负载突发窗口产物，s403 同族）。
- **本流域在册红 215F+29T → 223F+24T**（−5T：5 embed 文件级 Timeout 解挂；
  +8F：子测起跑后真面新显形——allowed-schemas has_width `offsetWidth>0` 5F +
  network-error frames.named 2F + change-src-2 SVG rect 1F。red 项计数随解挂净
  +3，但 embed 族加载链契约面全兑现，新 F 全部为渲染/帧代理域已定界未竟面）。
- 回归面：the-embed-element 目录余 13 键逐键同态零回归（document-getters 静态
  embed getSVGDocument null 面保持 P——parser 扫描不接的防回归设计兑现）。

## 残差与边界（钉死入账）

- frames.named 不含 embed/object（R177 帧代理 iframe-only）——network-error 子测
  3/4 余红（`frames.existingEmbed1.location.href` 导航面），FIXME 单列候选。
- embed allowed-schemas has_width（offsetWidth>0）渲染面 + change-src-2 SVG rect
  尺寸面——渲染流域候选（UA 默认尺寸 300×150 / replaced content 布局）。
- embed 静态 parser 扫描不接（FIXME(embed-static-events)，同 object 族记档）。
- embed 失败→load 语义按语料钉面落地（spec 精确文本未逐条核对——embed setup
  steps 的 display no plugin + load 派发面；语料四子测即契约）。
- 跨导航 WindowProxy identity 不保（s404 同款 shim 近似记档）。

## 下一步

- 在册红 223F+24T。下一候选：**frames.named 扩 embed/object 面**（R177 帧代理
  iframe-only——network-error 余 2F `frames.existingEmbed1/2.location.href` 导航
  面，本流域内小刀）；或散单续勘（挂账池承 s404：forward map 空返勘根/渲染流 5
  面/form-submission-target 6T/sizes-auto 半边/ilf+modulepreload 观察池）；
  has_width/SVG rect 渲染面移交渲染域记账；A2+D 簇立项征询仍待用户决策。
