# M2 切片 400 — object scheme 取回面 + window.length（driving 6F→P）

日期：2026-11-11（承 s399 下一步：object fallback 内容渲染切片——勘根推翻后按真实根因改道）
commit：见 git log s400 提交

## 勘根

s399 全树 sweep 新显形 6F 逐案提取断言消息定谳——**两族非 fallback 渲染缺口**，原设想的
「spec fallback 步子节点 DOM 化」切片不成立：

- **object-allowed-schemas.sub.window.js 4F**（blob:/data:/https:/http: 期望 load 实际 error）：
  `Unhandled rejection with value: "error"`——s399 的 data: 显式门 error + blob:/裸 origin
  走 host fetch 失败。fallback 内容已渲染（area 断言已过），非 DOM 化问题。
- **object-fallback-failed-cross-origin-navigation.sub.html 2F**（跨源失败导航 fallback）：
  `window.length` 返 undefined 期望 0——fallback 子节点断言已过，缺的只是 Window length
  getter。

## 施工（一 shim 文件 + 一桥测文件）

- `part01.js` `_zwLoadObjectResource`：
  - 抽 helper `_zwObjectMaterializeDoc`（doc+win+URL 印记+win.name 快照物化核）——about:blank/
    data:text-html/blob:text-html/http-text-html/bare-origin 五路共用（s398 内联段收编）；
  - data: 门换 `_zwDataURLProcessor` 解码（RFC 2397；fetch() 通路同源）：null → error；
    text/html essence → done 物化；其余 → 'image' success（load 照派无 navigable）；
  - blob: 分支：origin 门（fetch() scheme-fetch 同语义正则）+ `_zwBlobStore` 命中 +
    `_zw_blobBytes` + TextDecoder；miss/跨源 → error；text/html → done，其余 → 'image'；
  - bare-origin http(s) 分支：解析后路径空或仅 /（?/# 开头同）→ shim 合成空 text/html
    嵌套文档（FIXME 服务端目录语义）；门 = URL.parse 合法（URL.canParse）+ 非 bad port
    （_zwFetchBadPort）；
  - entry 增 `sel`/`handle` 字段（创建时快照——window.length 连接态判定消费）；
  - host fetch 路径内容分诊**保持 s399 三臂**（text/html → done / image 非 svg → image /
    其余 → error——见陷阱②）。
  - 新增 `_zwWindowLengthCount()` + `Object.defineProperty(globalThis, 'length', { get,
    enumerable: true, configurable: false })`（[LegacyUnforgeable] 形态；装在 R177 frames
    Proxy 后）：iframe 计 `_iframeDocCache`（win 有且非 _zwSwDestroyed）+ object 计
    `_zwObjectNavCache`（win 有且元素 isConnected——`_makeProxy(sel, handle)`，script s96
    先例）；error 态无 win 自然不计（fallback 语义 length===0 断言面）。
- `part102_s400_object_schemes_window_length.rs`（新建，注册 50 号 include）：钉① data:/blob:
  七面（text/plain 替换内容 load + null 保持；text/html win+doc 物化 + location.href 印记；
  无效 data: error；blob 同源物化 / text/plain null / 跨源 error / revoked error；host fetch
  零调用日志断言）；钉② window.length 七相位（0→iframe 1→text/html object→bare-origin
  object 3→404 不计→移除 object 递减→移除 iframe destroyed 递减；descriptor
  enumerable/configurable:false；bare-origin 不走 host fetch）。

## 过程陷阱

- **bare-origin 正则过宽**（首跑 solo object-events 4P→2F 回归）：`http://test:test`（无效
  port）经 `^https?:\/\/[^\/]*` 剥离后路径空 → 误入合成臂派 load。修：URL.canParse +
  _zwFetchBadPort 双门（object-events「error event (using 'http:' protocol)」驱动案）。
- **host fetch 分诊翻 success 是反回归**（同首跑「error event (using 'file:' protocol)」翻
  load）：s399 的「其余类型 → error」臂对 runner fetch 对 bogus-scheme URL 的宽松 2xx 信号
  是 load-bearing 的——原方案「非 text/html 2xx → success」直接把 file: 案翻成 load。修：
  host 路径回退 s399 三臂；allowed-schemas 的 text/plain success 面全部由 data:/blob:
  臂内语义承载（驱动集不经 host fetch）。
- **grep 二进制日志静默零案**：sweep 日志含 NUL（image body 落 Fail 行），grep 无 `-a` 只出
  「binary file matches」提示行——pairs 文件静默缩水到 3734 行（s344 教训重踩一次，管道级
  grep -a 一刀切修）。
- **join 漏 `-t '\x1f'`**：默认空白分隔在「file :: subtest」键首 token 上 join → 85 万行
  叉积（-t 不可省；键含空格时尤其致命）。
- **桥测断言面误读**（B2/D2）：data:text/html 物化嵌套上下文后 contentWindow 也非 null
  （「true/false/」应为「false/false/」）——首次跑测前先按状态语义核对每字段预期。

## 验证（串行管线，/tmp/s400/ 全量日志）

- shim 拼装：25 文件序 `node --check` SYNTAX OK。
- 桥测：part102 两钉 + part100/101 s398/s399 四钉全绿（filter `object_ s400`）。
- driving solo（testharness-html-semantics FILTER=the-object-element，40P/4 非绿）：
  - object-allowed-schemas **6P**（4F→P：blob:/data:/https:/http:；about:/javascript: 2P 保持）；
  - object-fallback-failed-cross-origin-navigation **2P**（2F→P）；
  - historical **4P**、object-events **4P**、object-attributes **4P** 保持；
  - object-javascript-url 1P+1T、object-handler 1T（挂账口径不变）；
  - 其余目录案零位移。
- document-getters-return-null-for-cross-origin solo：exit 0 3P（SVG data: URL null 口径保持）。
- 全树 sweep 双跑 join（基线 = s399 cur2-s399.pairs unique 8473，keyed 去重 + LC_ALL=C +
  join -t '\x1f'）：cur1 = cur2 unique 8473，双跑仅 1 键翻转（dialog-overlay computed
  overlay P/F——渲染流域案、并发负载时序 flake，不入稳定集）。基线迁移矩阵：
  - **Fail→Pass ×6 = driving 集精确命中**（allowed-schemas 4 + cross-origin 2）；
  - Pass→Pass ×8216（基线 P 零转红零丢失）；
  - Fail→Timeout ×2 = select-home-end 双键（s396 已定谳负载时序 flake 家族、真态 F——
    本轮双跑均 T，口径单列）；
  - new/gone 键零（除 dialog flake 键在基线 P 侧、本轮不稳定侧）。
- 单元：`cargo test -p zero-engine -j 2`（test-guard）**3955P/7F**（= s399 3953P + 本切片
  2 新钉；7F 与 s395–s399 在册红集逐名相同——img_runtime_deferred_settle_r5275 /
  svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix /
  iframe_service_worker_messages_dispatch / response_body_readable_stream_r2967 /
  t8k_websocket ×2）；`-p zero-webview` 848P/2F 在册同形；`-p zero-script-sandbox`
  452P/0F 全绿。
- 产品 smoke：`make product-smoke` PASS；`make product-smoke-legacy` PASS。
- fmt/clippy：`cargo fmt --check -p zero-engine` exit 0；`cargo clippy -p zero-engine
  --all-targets`（test-guard，无 --compile-first）Finished clean。

## 在册红口径

- 全域 sweep2：unique 8473 = 8222P/217F/33T（33T 含 select-home-end 双键本轮 T 态 + 他流
  渲染探针相关态；dialog-overlay 1 键双跑不稳定不计入稳定集）。
- **本流域口径 224F+31T → 218F+31T**：driving 6F→P；select-home-end 双键真态 F（T 态单列
  不改判）；他流 radio 探针 1F 单列口径不变。

## 残差与边界（钉死入账）

- bare-origin 目录语义为 shim 合成近似（FIXME 留 part01——真实服务端 listing 不可达）。
- host fetch 路径非 text/html 非 image 类型 → error 保持（spec 可渲染判定 MIME sniff 全链
  未建，FIXME 留 part01）——text/plain over http 等面待 runner fetch 信号可信后重开。
- window.length 计数面：frame 元素/未消费 iframe 的惰性物化不计（_iframeDocCache entry 仅
  contentWindow 面创建——FIXME 留 part01）；嵌套 win 对象未挂 length（顶层 globalThis 面
  已覆盖 driving 集）。
- object-handler.html（嵌套文档脚本通道——R206 object 移植面）T 散单池留存。
- object-javascript-url.html 1P+1T（javascript: URL object 语义半开）——基线文件级 T 已
  松动为子测级，剩余子测挂账。

## 下一步

- 在册红 218F+31T（本流域口径）。下一候选：本流域散单（focus scroll-into-view 1F /
  link-rel shadow DOM CSSOM）或 object-handler 嵌套文档脚本通道（R206 object 移植）；
  簇 D（customizable select 特性族）仍待用户决策。
- 挂账池不变：`__zw_resolve_handle_selector` forward map 空返低优先勘根；渲染流 5 面→
  rendering-compat；form-submission-target 6T→navigation M3；sizes-auto lazy 半边待勘根；
  A2+D 簇立项征询待用户决策；radio 探针残留案归渲染流处置。
