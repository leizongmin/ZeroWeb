# html-semantics-compat 历史轮次账归档（M1 至 M2 切片 413，2026-10-15 至 2026-11-12）

> 本文由 `docs/goal/html-semantics-compat.md` 的「轮次账」历史段按原样迁出；当前状态见 [../../html-semantics-compat.md](../../html-semantics-compat.md)。

- **2026-11-12 M2 切片 413**：声明式 CSS module parse 期 map 注册施工轮
  （tentative/ 11 文件全绿：2T→子测 P 键形迁移 + 6 NotRun→9P；webview
  map 快照 + prepare 播种 + collect 顶部命中臂四消费方穿参 + engine ⓪
  标记门 + shim `.sheet` undefined 臂）——kind 固定载体勘根：变异路径
  逐批 re-parse 快照（NodeId 旗标即死）→ html TEXT `data-zw-declmod`
  标记属性。map 键 page URL 基注册/import parent 基查找（语料全内联
  承载恒一致）；命中臂先于裸 specifier 检查（语料裸值形态）；CSP 阻止
  者不注册。陷阱：单测无 fetcher → 动态 import 机制注册面整体缺席
  （R3091 门，V8 默认 "Not supported"——fetcher 兼探针修正）；
  ACTIVE_ADOPTED_CSS 进程级槽并行测试互染（R-t2qq 既有语义）→ 双相
  合并单测 + 相间清槽。
  验证：solo ×2 37 子测全 Pass（0F/0T/0NotRun；目录余 style-error-01
  1T 非 import 簇在册）；engine 3909P/7F（+1P，
  在册 7F 零位移）+ webview 852P/2F（+1P，在册对零新增）；fmt/clippy
  clean（css-parser random.rs 跨流单列）；sweep ×2 双跑全等
  8275P/212F/17T + 5 NotRun，join-cross 零位移、join-b 对 s412 恰
  19 new/2 gone state-diff=0 纯 driving。
  **在册红 212F+19T→212F+17T**（−2T），NotRun 11→5。下一步：
  constructed-stylesheets 1T 同族勘根 / 挂账池承 s404/s405/s411/s412；
  select/combobox（簇 D）与 A2+D 簇立项征询仍待用户决策。
- **2026-11-12 M2 切片 412**（账补记 2026-11-12，s413 收口轮补齐——
  429 断点恢复会话轮次账条目缺漏）：声明式 CSS module 双臂 CSP +
  元素站 error 面施工轮（tentative/ CSP 簇 6 文件解挂，extract
  is_module + gate script 臂 + script_gen 元素站 ordinal 派发 + 常驻
  钉两件）——单臂只查 style-src 漏阻 + 被阻 module 无 error 派发双层
  红根。验证：solo 18 子测全 Pass；engine 3908P/7F（−3 favorable 并行
  流合入域）+ webview 851P/2F；sweep ×2 全等 8256P/212F/19T + 11
  NotRun，join-b 对 s411 恰 9 new/6 gone state-diff=0。**在册红
  212F+25T→212F+19T**（−6T）。详 evidence/2026-11-12-m2s412-style-
  module-csp.md。
- **2026-11-12 M2 切片 411**：maxlength 仅适用六文本态施工轮（driving
  1T→P，webview plan 门 + shim live 守卫 + 桥测钉 part110 + webview 钉）
  ——maxlength-number file-Timeout 勘根三层：number-invalid 对照锁定
  maxlength 单因子、相位探针定位 char '2' 起 MaxLengthReached noop、挂死
  链 = dispatch_action 不豁免 → send_keys 链 reject → 无 .catch 案面
  unhandled rejection 挂至预算。双写同口径门（16 非文本态黑名单 + shim
  normalize 六态白名单，missing/invalid default 落 Text 判适用）。
  验证：solo Pass；engine 3904P/10F（在册 9F 零位移 + 并行流 t49 单列，
  clean HEAD stash 复跑归因）+ webview 850P/2F 零位移；sweep ×2 双跑全等
  8249P/212F/25T join-cross 零位移、join-1 对 s410 恰 1 new/1 gone 纯
  driving 位移 state-diff=0；跨流三处门禁违例（clippy collapsible_if +
  两处 fmt）机械清偿 commit（rebase 判零 delta 跳过——并行流 g18/g19
  同批自偿）；focus-dynamic 争用 flake 单列不在册。
  **在册红 211F+24T→211F+23T**（−1T）。下一步：constructed-stylesheets
  9T 机制勘根 / 挂账池承 s404/s405（新增适用态打满 reject 面 /
  maxLength IDL 不适用态反射 / unhandled-rejection 换算）。
- **2026-11-12 M2 切片 410**：send_keys Enter formless 无隐式提交目标
  默认动作 no-op 施工轮（driving 1F→P，runner 单文件 +23 行）——
  the-input-element 散单双案勘根：date-readonly-keyboard Enter 落 Submit
  臂 → formless noop(NotApplicable) → dispatch_action 不豁免 → send_keys
  链 reject → unhandled rejection（真浏览器 implicit submission 以 form
  owner 为前提，formless 无动作）；施工 `enter_formless_noop` 臂三重例外
  （buttonish click 优先 / CE insertParagraph 不跳过 / form 内隐式提交族
  不动）。checkable 父 label 2F 勘根：桥测探针证 shim :active 路由健康、
  corpus 探针证父 label 零矩形 0,0,0,0（子/following 实矩形对照）→
  pointerdown 落 BODY，根因 layout-engine inline-container boxing 归渲染
  域（s405 先例移交记账）。验证：solo Pass；engine 3897P/9F（名册 diff 恰
  r307 F→P favorable 归并行流）+ webview 849P/2F 零位移；fmt/clippy
  clean；sweep ×2 join-1 恰 1 STATE-DIFF = driving F→Pass、new=0/gone=0
  键集 8484 全等（select-home-end T↔F 族漂移单列）；**在册红
  212F+24T→211F+24T**（−1F）。下一步：maxlength-number 1T 勘根 /
  constructed-stylesheets 9T 机制勘根 / 挂账池承 s404/s405。
- **2026-11-12 M2 切片 409**（账补记 2026-11-12，收口轮轮次账条目缺漏
  补录）：script-created link stylesheet 不阻塞 script 施工轮（driving
  2F→P，shim 两文件 + 桥测钉）——双层红根：s163 延迟仿真相对 URL miss
  （发题侧 base 归一）+ s349 阻塞计数缺 parser-inserted 限定；
  interactions-of-styling-and-scripting 目录清零。详见 Active 段留档与账本
  evidence/2026-11-12-m2s409-script-created-link-stylesheet.md；**在册红
  214F+24T→212F+24T**（−2F）。
- **2026-11-12 M2 切片 408**：detached 文档 style 元素 LinkStyle sheet +
  headEl 本地树三件套施工轮（driving 1F→P，shim 两文件+桥测钉）——
  the-style-element 10F 勘根定谳：唯一 corpus 驱动案 non-parser-created-doc
  （`createHTMLDocument().createElement('style')` + `doc.head.appendChild` 后
  `assert_true(!!style.sheet)`）四层红根：headEl 无本地树方法（R126 docEl 同
  族漏网——appendChild 落全局 Node.prototype，childNodes 不插、parentNode 不
  设，探针 P1/P7 实证）+ plain `_zwMEl` 产物无 sheet 路由（main getter 分支
  按 sel/handle 代理匹配不命中，探针 P6）+ R5271b 门对 plain owner 恒 false
  （`_zwIsConnected` 双 undefined → 尾部 rect probe false；spec LinkStyle：
  sheet 在任意文档树内即存在，detached 树含；未挂载 null 保持 LinkStyle.html
  not-in-a-Document 面）+ createElement 时直挂先例（R5256 dialog / s69
  img.decode）。施工三件：①part03 headEl 三件套（appendChild/removeChild/
  insertBefore 镜像 docEl trio R126 语义——先移旧父 + `_r223SetParent` +
  `_r130WireSiblings`；removeChild 带 NotFound 校验 + 迭代器/range 守卫；
  FIXME(headEl-adopt)：R330 adopt 传播与 CE 派发未接线 corpus 无钉；body
  字面未驱动不动）②part03 detached STYLE 产物直挂 `sheet` 访问器（type 门
  镜像 part04 R3036 s156/s160 精确匹配；体走 `_makeStyleSheet` 统一门单实现；
  FIXME(style-detached-rules)：textContent 内联规则未接 CSSOM cssRules 恒空）
  ③part06 `_makeStyleSheet` R5271b 门扩 plain owner 臂（parentNode 链 walk
  到 nodeType 9，64 步防环；sel/handle 形态不变；出树 gate false、回挂指纹
  命中同对象 R5271 缓存跨门一致）。常驻钉 part108 双测（driving 镜像+树接线
  +[SameObject] 身份+出树/回挂门矩阵+cssRules 恒空面 / detached type 门五案
  jibberish→null·TeXt/CsS→obj·空串→obj·带空白→null）。验证：双钉全绿
  （detached_ 28 passed）；单元 engine **3886P/10F**（+2 钉，失败清单逐名 =
  9 在册 + r307 单列零位移）+ webview 849P/2F；driving solo F→P；
  the-style-element 目录干净跑 **34P/0F/9T/6NotRun**（9T = tentative/ CSP +
  constructed-stylesheets 大机制独立候选）；全树 sweep ×2 join 恰 1
  STATE-DIFF 全 driving F→Pass 双跑一致真 new=0（run2 8484 keys
  8243P/215F/26T；run1 8491 keys 含遗留探针 7 键单列——sweep-1 起跑时探针
  未删；cross state-diff=1 = customizable-select watchdog 家族同二进制翻转
  单列）；过程陷阱：勘根探针目录 solo 污染（P6 旧架构断言恒红——修复走
  plain 访问器路线非 handle 形态，探针断言过时；且勘根前 `ls` 误报不存在是
  路径层级算错一级——probe 卫生纪律「勘根毕即删」重申）；钉首编 E0308
  （`Arc::new` 漏 `Mutex::new` 包裹，照抄锚点丢中间层）。fmt（本切片文件）
  干净（callbacks.rs HEAD 既有 fmt 脏承 s407 单列）/clippy clean/smoke 双
  PASS；**本流域在册红 215F+24T→214F+24T**（−1F）。下一步：散单续勘
  （links 族 4F / the-input-element 散单 / tentative/ constructed-stylesheets
  机制勘根后按簇决策）；A2+D 簇立项征询仍待用户决策。
- **2026-11-12 M2 切片 407**：静态 embed 链 + fallback 子内容激活施工轮（六
  driving 键 F→P 全解挂，shim 两文件+桥测+语料资产）——driving 六键
  （the-embed-element 5 + the-object-element twin 1，s406 收口在册 F）三层勘根：
  静态 embed 无人启航（s405 FIXME(embed-static-events) 收口位——embed-iframe
  子文档永不物化 `parent.childLoaded` 恒 false）+ s403 object 平扫误启 loaded
  祖先内 should-not-load 子 object（活测试 assert_unreached 文件红，spec
  fallback 内容不渲染步）+ s404 data: 不可渲染 essence 误映射 `image` 可渲染面
  （祖先门读不到渲染失败信号）。时序前提：objnav fetch host 桥同步 → 物化+子
  脚本一个 turn 完成 → 静态扫描置于 part06 load 分支监听器前即可，R5275 门零
  改动。施工四件：①part04 `__zwStartParserObjects` 重写——`_zwStaticAncGate407`
  祖先链静态门（`_realTag`+`__zw_parent` 上行 64 步防环：media 祖先 / object
  祖先 entry ∈ {done,image} → 跳过，键形态双查 QSA handle proxy
  `__zw_selector_for_handle` 反解 R4400 先例）+ object 循环同步直调（祖先门须读
  先行元素终态，defer 打散启动序；load 派发位无 mutation 风暴同步安全）+ 新
  EMBED 循环（src + being rendered 门复用 R3074 `_zwCheckVisibility`——
  display:none 祖先链 walk，cv 不在判面双 driving 案不受影响）；②part01 data:
  essence `/^(text\/|image\/)/i` → image 其余 → 新 `fallback` 态（text/plain 与
  SVG data: 口径保持）；③part01 `_zwFireObjectResultEvent` gate 加 fallback、
  映射 object?error:load（embed 恒 load s405 同款）；④语料资产
  embed-in-object-fallback-subdocument.html（75B pin 315976933870b34d…
  gh api 渠道，s227/s404 语料缺口同族——fetch_raw 常驻+update-wpt-data.sh
  挂账）。part107 桥测三钉：`embed_static_chain_faces_s407`（启航链 parent 旗标
  通道+media/loaded 门+静默面 fetch log 计数）+ `embed_object_fallback_activation_faces_s407`
  （激活矩阵五拓扑+不可渲染 essence 动态派 error+text/plain parity+data: 零网络
  取回）+ `embed_rendered_gate_media_object_gates_s407`（双红回归面）。验证：
  桥测三钉全绿 + 单元 engine **3884P/10F**（失败清单逐名 = s406 在册 9 + r307
  HEAD 既有红单列）+ webview 849P/2F（复跑 848P/3F 第 3 红 service_worker 单测
  隔离 Pass+全量复跑缺席 → 时序 flake 单列）+ driving solo 门修复后 embed 目录
  8 关键面全 Pass + object 目录 object-ignored 复绿 + constraint-validation 在册
  同红 + object-in-display-none-load-event Pass（佐证 object 不挂 rendered 门）
  + 全树 sweep ×2 **8484 keys 8242P/216F/26T 双跑全等 join-cross state-diff=0**
  （s292 协议）+ join 恰 6 STATE-DIFF 全 driving F→Pass 双跑一致真 new=0 +
  fmt（本切片文件）干净（callbacks.rs HEAD 既有 fmt 脏 0ff1bc92c 并行流负债
  单列）/clippy clean/smoke 双 PASS；**在册红 221F+24T→215F+24T**（−6F）。
  过程陷阱四件入账本：首轮终局 join 抓双红（represent-nothing-04 never-rendered
  须 rendered 门 / object-ignored 旧 defer 启航时子文档 step-after-done 被
  harness 吞掉假绿 vs 同步启航踩活测试——门语义缺口 A/B 实证非时序本质）+
  预修 sweep-1 select-event 82 键行静默丢失（同文件 T 塌缩家族新形态）+
  LC_ALL=C 管道只给首命令重踩（s287/s292 在案）+ clippy 须 `-- -D warnings`
  分离形；customizable-select 2 键 F→Timeout 文件零 embed/object 引用 + stash
  A/B HEAD 二进制 solo 双跑亦 T → watchdog 边界翻转族单列（s406 select-event
  /focus 塌缩族同款）。下一步：散单续勘（the-style-element 10F 未归因面 /
  links 族 4F / the-input-element 散单 / 挂账池承 s404/s405）；has_width/SVG
  rect 渲染面移交渲染域记账；A2+D 簇立项征询仍待用户决策。
- **2026-11-11 M2 切片 406**（账补记 2026-11-12，收口轮轮次账条目因会话 429
  打断漏记）：frames named access 扩 embed/object + 嵌套导航失败终态施工轮
  （embed-network-error 4 子测全解挂，commit 284a7bbb4，账本
  evidence/2026-11-11-m2s406-frames-named-embed-nav-fail.md）——network-error
  余 2F 双层勘根（R177 帧代理 named access iframe-only + href setter URL 解析
  静默中止）三件施工（R177 缓存序 named 扫描 + `_zwObjectNavFail` 失败三臂收拢
  + href setter 容差回落）；part106 桥测全绿 + engine 3880P/9F（execcommand
  2F HEAD 跨流红三重 A/B 实证单列）+ sweep ×2 join 恰 2 STATE-DIFF 跨跑
  state-diff=0；**在册红 223F+24T→221F+24T**（−2F）；sweep-1 test-guard SIGTERM
  runner 孤儿补全单列。详见承前块与账本。
- **2026-11-11 M2 切片 405**：embed 动态加载链施工轮（s399 镜像，the-embed-element 目录 5 文件级 Timeout 全解挂，shim 三文件+桥测）——driving `embed-javascript-url.html` 三子测（s404 object 孪生案同构）文件级 Timeout 勘根：embed 元素**零加载面**（无插入钩子/无加载核/无嵌套 navigable；named access 正则滤数字属性→`window[0]` undefined）。the-embed-element 目录勘根（s404 sweep 在册 5 文件级 T 全指向缺加载链）四文件差异面定界施工口径，语料钉 embed 失败语义定界：**失败（nonexistent host/X-Frame-Options/404）→ load 派、error 恒不派**（embed-network-error 四子测 onload=done/onerror=unreached——与 object error 事件相反；spec #the-embed-element-setup-steps display no plugin 语义）+ javascript: src 全静默（子测1「should do nothing」——非取回失败不走失败→load 语义）。施工四件：①part01 `_zwLoadObjectResource` 五参化（+isEmbed——entry 增 `embed` 印记、属性读 `isEmbed?'src':'data'`、javascript: 臂条件 `fireEvents && !entry.embed` 全静默）；②part01 `_zwFireObjectResultEvent` 事件映射 `state==='error' ? (entry.embed?'load':'error') : 'load'`（object allowed-schemas javascript: not-allowed error 面保持）；③part01 新 `_zwStartConnectedEmbed(el, connected, fireEvents)`（镜像 `_zwStartConnectedObject`：EMBED tag 门+共享加载核 isEmbed=true；共享 `_zwObjectNavCache`→s404 `window[N]`/s400 `window.length` 活跃集合自动计入；FIXME(embed-static-events) parser 静态扫描不接——语料驱动案全动态插入，静态 Pass 案 document-getters-return-null-for-cross-origin getSVGDocument null 面零触碰防回归）+ part04 两插入路同排（appendChild 融合+append 变参）；④part05 R5009 排除 `EMBED.src`+新 src IDL setter 重启臂（镜像 s399 OBJECT.data：attr 写+isConnected 门+旧 entry 删除+`_zwLoadObjectResource(key,sel,handle,true,true)`——change-src 重启兑现不依赖 /media/ 资产，失败面按语义映射 load 事件序断言与资源身份无关）。part105 桥测两钉：`embed_javascript_url_faces_s405`（javascript: 全静默零事件+`window[0]` 嵌套上下文+完成值重物化 load 1→2+`window[0].document.body.textContent` 读面（embed 无 contentDocument IDL）+URL 槽不动+javascript:1 静默中止+全程零 javascript: fetch）+`embed_failure_load_restart_faces_s405`（404→load 派 error 不派+error 态不计索引+data:text/html 物化 window[0]+src IDL 重启 fired 门复位第二 load+doc 换新）。验证：桥测 ×2 全绿 + 单元 engine **3881P/7F** 在册逐名同形（+2 新钉全绿，失败清单 diff 逐名相等）+ webview 849P/2F 同形 + driving solo（the-embed-element 全目录）**5 文件级 Timeout 全解挂**：embed-javascript-url 3P（driving 全中）+ embed-change-src 1P（重启臂兑现）+ embed-network-error 2P+2F（余 F=frames.named `existingEmbed1/2.location.href` 通道缺——R177 帧代理 iframe-only s405 不施工单列）+ embed-allowed-schemas 5F（load 派✓ 余 F=has_width `offsetWidth>0` UA 默认尺寸渲染面）+ embed-change-src-2 1F（load 派✓ 余 F=SVG replaced rect 尺寸渲染面）；目录余 13 键逐键同态零回归。全树 sweep ×2 join 对 s404 pairs 基线（启动遇并发负载突发 15min load 91，1min 回落 12 放行）：**13 NEW+5 GONE 恰 embed 族双跑逐键一致（确定性）**，selopt 家族 T↔F 形态翻/modulepreload F↔P 振荡/run2 select-event+fdtcb 整文件 T 塌缩（cross join 证 run1 基线同态=s292 时序 flake，s403 负载族）flake 单列；fmt/clippy clean。**本流域在册红 215F+29T→223F+24T**（−5T embed 解挂；+8F 子测起跑真面新显形——has_width 5F+frames.named 2F+rect 1F，全为渲染/帧代理域已定界未竟面）。过程陷阱：桥测 L1 断言自误（双 insert 后 length 应 1 非 0——emb4 data:text/html 同期已物化，error 态不计由 window[0] 身份面承载，修 `load/1`）。残差：frames.named embed/object 面余 2F+embed has_width/SVG rect 渲染面（移交渲染域记账）+parser 静态扫描不接（FIXME 记档）+embed 失败→load spec 精确文本未逐条核对（语料四子测即契约）。账本
  [evidence/2026-11-11-m2s405-embed-dynamic-load.md](html-semantics-compat/evidence/2026-11-11-m2s405-embed-dynamic-load.md)。
- **2026-11-11 M2 切片 404**：object `window[N]` 索引访问 + javascript: URL 导航面施工轮（driving 文件级 Timeout→三子测 Pass，shim 单文件+语料资产+桥测）——object-javascript-url.html 文件级 Timeout 双层勘根（s227 同族先验语料资产再验引擎）：①语料 `/resources/blank.html` 缺席（insertObjectNavigable 以 `data='/resources/blank.html'` 导航→404→error 代替 load→子测 2/3 首个 `await onload` 恒挂，state dump `{"loaded":true,"pending":2,"phase":4,"tests":3}` 证挂起非 TypeError；资产 gh api pin 31597693 渠道补齐 379B md5 04daab42…，goal 脚本 ③段挂 `fetch_raw "resources/blank.html"` 常驻重拉+update-wpt-data.sh 挂账注释「升级新 ref 后需重新补」）；②`window[N]` 索引通道缺（named access `_installNamedAccess` 正则 `/^[A-Za-z_$][A-Za-z0-9_$]*$/` 滤数字属性从不安装，资产补后 `window[0]` undefined→TypeError 快败）；③javascript: 导航臂缺（`_zwObjectNavigateEntry` 直落 fetch 通道）。施工三面 shim（part01）：`_zwWindowIndexedWin(idx)` + `defineProperty(globalThis,'0'..'15')` 索引层（own accessor enumerable:false 防 keys/for-in 漏出+configurable:true 保页面数值全局可遮蔽——spec own 属性优先 WindowProperties；活跃集合同 `_zwWindowLengthCount` 口径 iframe _iframeDocCache 未 destroy+object _zwObjectNavCache 已连元素；FIXME 物化序≈树序近似+16 上限截断+WindowProperties 异对象层不可整体 Proxy 化）+ `_zwLoadObjectResource` javascript: data 属性臂（静默 fallback state='error'，子测1「should do nothing」钉面；error 事件从同臂族 data: 解码失败/blob: 跨源；FIXME spec 网络错误语义精确面——完成值字符串虽建文档，语料仅钉无 load）+ `_zwObjectNavigateEntry` javascript: 臂（#javascript-protocol：href setter 先经 URL 解析 opaque path 百分号编码 `%22test%22`→**percent-decode**（spec 明确；decodeURIComponent 残缺 % 抛错回落原文）→R206 形参遮蔽+函数体内 `return eval(body)` 完成值透出（window/self/parent/top/document/location/navigator/history 遮蔽到嵌套上下文，其余全局回落页面 realm FIXME）→字符串完成值 `entry.fired=false` 复位+`_zwObjectMaterializeDoc`（URL 槽传 entry.url 不动）+`_zwFireObjectResultEvent`（navigate entry 无 fireEvents 参数，导航完成恒派 load spec 对齐）；非字符串/脚本抛错静默中止 doc 不动无事件，FIXME report the error 近似）。embed 孪生案（embed-javascript-url.html 同构）不同径不混本刀：embed 无插入钩子/加载链/嵌套 navigable——s405 候选。part104 桥测两钉：`window_indexed_access_faces_s404`（iframe+object 物化序索引+length=2+越界 undefined+descriptor get/enum:false/cfg:true/keys 不漏+移除动态重排 [1]→[0]→undefined+索引/length 读零 fetch）+`object_javascript_url_faces_s404`（data=javascript:"foo" error 无 load 零 fetch+window[0]=contentWindow+完成值字符串重物化 load 1→2+percent-decode 落 doc+URL 槽停 blank.html+javascript:1 静默中止无第三 load+obj2 初载恰一取回+javascript: URL 不落 host fetch）。验证：桥测 ×2 全绿（J4 断言过强返工一次——跨导航 WindowProxy identity 不保：`_zwMakeIframeWin` prevWin 通道不保 identity 重物化建新 win，s403 同型 named access 重 morph，corpus 断言面均为 fresh 读不触——改 live 解析等价 `window[0]===o2.contentWindow`+FIXME 记档）+ 单元 engine **3879P/7F** 在册逐名同形（失败清单 diff 逐名相等；+2 新钉全绿）+ webview 849P/2F 同形 + driving solo **三子测 Pass** + 全树 sweep ×2（idle，runner exit=1=带失败退出码口径 s403 同值非截断）join 对 s403 idle pairs（8478/8475，8226P/217-221F/31-32T raw）：**object-javascript-url 文件级 T 键双跑 GONE+`'javascript:"test"'`/`'javascript:1'` 两子测 NEW Pass 稳定**（子测1 do-nothing 基线已 Pass——文件挂起前已上报，本轮三子测齐报；raw 8228-8229P/217F/30-31T），selopt 家族形态翻（detailed 文件键 T↔F 跨跑+optional 文件键 run1 单现 T=s387 在册截断族）/checkable F→P（振荡族落有利侧，无因果路径）/dialog-overlay F→P（在册跨跑族）flake 单列，modulepreload-referrerpolicy P→F 同二进制跨跑翻转=s292 判据时序 flake 新观察挂池；object-allowed-schemas 六键全 P（javascript: not-allowed error 期望面经新 load 臂保持）+ object-handler 2P 保持（s403 修复在册复测确认）；fmt/clippy clean（`--compile-first` 仅支持 cargo test 重踩一次——cargo build 挂旗标被拒，s403 已入册本轮管道脚本固化去旗标）。**本流域在册红 215F+30T→215F+29T**（−1T object-javascript-url 文件级 Timeout 化三子测 Pass；F 口径零位移）。残差：`window[N]` 索引序物化序近似树序+16 上限截断+跨导航 WindowProxy identity 不保（重物化建新 win，corpus fresh 读不触 FIXME 记档）+ javascript: 求值 realm 回落页面 realm+脚本抛错静默无 console 上报面+data 属性 javascript: error 事件面 spec 精确语义语料仅钉无 load。账本
  [evidence/2026-11-11-m2s404-object-javascript-url.md](html-semantics-compat/evidence/2026-11-11-m2s404-object-javascript-url.md)。
- **2026-11-11 M2 切片 403**：object 嵌套文档脚本通道施工轮（R206 移植，driving 文件级 Timeout→双子测 Pass，纯 shim 三文件）——object-handler.html 勘根三面缺通道（parser 静态 object 无人启航——`_zwStartConnectedObject` 仅动态插入钩子接线/嵌套文档脚本不执行——`_zwObjectMaterializeDoc` 无 R206 通道/嵌套导航面缺——嵌套 location.href 写+named access morph+嵌套 history.replaceState 均无物）。施工：part04 `__zwStartParserObjects`（S6 同款扫描位，**静默** fireEvents=false——display:none 双案钉「不渲染不派 load」，FIXME(object-static-events) 记档 spec 偏差，恢复需 rendered 门）+ part06 window load 派发分支 `__zwStartParserIframes()` 后同位接线 + part01 `_zwStartConnectedObject` 三参化 + `_zwObjectMaterializeDoc` 重写（doc._zwURL 槽/R206 全参 win/win.location 挂 `_zwObjectLocation`/named access morph——parse 期 `_installNamedAccess` 装元素 face，spec navigable 值算法单 named 元素带嵌套上下文→WindowProxy，identity 守卫 element/prevWin/undefined/`_zwRunIframeScripts` 接通）+ 新 `_zwObjectLocation`（href get 读 `entry.doc._zwURL`；set `new URL` 解析同值 no-op、异值走导航；protocol/host/hostname/pathname/search/hash/origin 面，hash setter URL 槽近似 FIXME）+ `_zwObjectNavigateEntry`（fetch `objnav:<key>:nav`，2xx text/html 重物化+脚本重放，error/非 2xx→state='error'、doc/win=null fallback 语义，data 属性不动）。**回环修复**：materialize 首行 `state='done'` 先行——嵌套脚本执行中 contentWindow/contentDocument 消费读走 `_zwLoadObjectResource` 早返门（'loading' 期落穿=重取回+重物化自递归，桥测 A 链 170 连 fetch 实证、回调 href0:false 循环 ~145 轮；各臂尾部 state 赋值成幂等 no-op）。验证：part103 桥测 ×2 全绿（回调链五断言序 href0/na/rs/href1/data 全 true+静默事件空+终态 doc 停 test1+fetch 恰两取回+fallback 零取回；二次 sweep 零重取回+导航 404 fallback 语义+幂等）+ 单元 engine **3877P/7F** 在册逐名同形（+2 新钉全绿）+ driving solo **双子测 Pass**（余红 3 案 object-javascript-url T/constraint-validation F/in-object-fallback-2 F = s402 基线同态）+ 全树 sweep ×2（idle）join 对 s402 base 8475 pairs（8227P/218F/32T raw）：**object-handler ×2 NEW Pass 稳定 + link-rel Fail→Pass（s402 修复在册复测确认）**，ilf Pass→Fail 定谳**环境漂移单列**（stash no-WIP 二分 idle ×3 同红钉证非本切片引入，成因未究挂观察池——真回归定谳必须 no-WIP 对照跑单跑 with-WIP 不足），selopt Fail↔Timeout/dialog-overlay/checkable flake 振荡单列（基线已红/在册家族），GONE=object-handler 文件 T（胜）+selopt 文件键（形态翻）+zz-probe（s402 探针已删出账）+ fmt/clippy clean（`--compile-first` 仅支持 cargo test——clippy 挂旗标被拒去旗标）。**本流域在册红 215F+31T→215F+30T**（−1T object-handler 文件级 Timeout 化双 Pass；ilf 漂移 +1F 单列不入切片口径，下轮基线刷新定夺）。残差：静态 object 渲染终态事件不派（FIXME 单列）+ `_zwObjectLocation` hash setter URL 槽近似 + origin 无 about:blank 特例（语料无钉面）+ 嵌套 window.length/frames 面未接（s400 近似在册）+ 嵌套导航 objnav 通道不走 R56h inflight 闸门（text/html 无延迟 load 面）+ ilf 环境漂移成因未究。账本
  [evidence/2026-11-11-m2s403-object-nested-script-channel.md](html-semantics-compat/evidence/2026-11-11-m2s403-object-nested-script-channel.md)。
- **2026-11-11 M2 切片 402**：shadow 森林 handle 身份跨探测环刷新重建轮（driving 1F→P，engine 新模块+分支重构）——referent 核验定谳 s168b 修复的 link-rel-attribute shadow 案在 M3 Batch 2 声明式模板回环（38b972751）后回红（M3 序列化回环回归非 s168b 面失效）：shim `_zwSrQualifies180` list 臂 + serializer ShadowRoot 臂对命令式根恒发 `<template shadowrootmode>` → 探测环 dump 携标记 → refresh re-parse 物化 shadow 树——**树不丢，丢的是 handle 身份**（序列化不带 handle 标记，`__zw_get_computed_style_by_handle` 的 `nodes[handle]` 只来自重放窗口 `[applied..count)`，drain 尽后窗口空 → shadow 树内 handle 全落空 → gCS ''；探针 v4 A/B：t30/byHandle30 `""→rgb(0,128,0)`；刷新后新 attach 的 shadow 一个探测周期后同病实证）。施工：新模块 js_dom_bridge/shadow_forest.rs——`shadow_forest_mutations` 森林切片三遍 O(n)（①种子+最后归属表：handle 锚 placement 覆盖写 child→parent 最终归属口径——dump 已是终态真相，forest→light 挪回不重放防双份；②成员闭包记忆化下行+环防御；③顺序过滤发射；light 宿主命令式 shadow 整树 prune 迭代至不动点——宿主 create 不重放则 attach 发射必撞 unknown host handle Err 中止切片尾部，根出种子子树全弃重放零 Err 后继链不截断）+ `strip_declarative_shadow_templates` 深度配平剥声明式模板 span（普通 template 保留，无标记串零开销透传）9 单测 + computed_style_by_handle 增 `prefix_has_shadow && base_has_marker` 分支（剥模板基座 parse + 森林切片重放重注册 handle + 窗口 `[applied..count)` 照旧续放；s168 臂 else 分支语义不变，两臂分流互斥）。验证：单元 9 新钉全过 + engine lib 3874P/7F 在册逐名同形（img settle ×2/fetch/SW/response/websocket 时序族）+ webview 849P/2F 同形；driving solo **2P/1F→3P/0F**（solo+sweep1+sweep2 三跑全绿）；the-link-element 114P+探针（s168b 时代 1F stylesheet-media-change-no-fetch 区间内 intervening 修复现绿非本刀）；全树 sweep 双跑 join 对当轮 stash 基线（8475 pairs=8227P/218F/32T raw 含探针 1F）**sweep2 = stable 8474 + Fail→Pass ×1 恰 driving + new/gone ×0**（sweep1 差异键 = fdtcb 文件级 T 截断 4 子测在册 flake + home-end T→F 在册家族振荡；基线对 s401 cur 漂移 = 探针新增 1F 已删 + home-end 振荡——corpus 真红零位移；join 键字段反转重演 s401 在册坑——s401 cur2.pairs.u 是 state\x1fkey 形本轮 key\x1fstate，先对齐方向再 join）；fmt/clippy clean（matches! 闭包内 ref 绑定 E0027 去显式 ref；clippy doc_lazy_continuation 续行缩进）。**本流域在册红 216F+31T→215F+31T**（−1 driving；探针行勘根毕删下轮出账）。残差：light 宿主命令式 shadow 刷新后子树视图缺（lenient 不劣化，恢复需宿主 handle→dump 节点重锚 forward map 通道 FIXME 单列）+ sel 子 move-in 剥模板基座 miss（无 handle 面不受影响）+ 剥模板子串判病态串近似（与 shim `_zwStripEchoHtml180` FIXME 同族）。账本
  [evidence/2026-11-11-m2s402-shadow-forest-handle-rebuild.md](html-semantics-compat/evidence/2026-11-11-m2s402-shadow-forest-handle-rebuild.md)。
- **2026-11-11 M2 切片 401**：探测环 doctype 丢失 quirks 翻转修复轮（driving 1F→P，webview 单点+常驻钉）——散单 focus scroll-into-view 1F 八步探针链勘根定谳：直调 `ro.focus()` 同步读 scroll 完美（R5257 臂健康，`foc:seT=650,sy=650` 精确最近对齐）→ 键盘路径 `sy=650 seT=0` 发散 → final `cm=BackCompat seTag=BODY` → 早读 CSS1Compat → 相位标 t2 翻转 → 受控对照 `setAttribute+await` 即翻转（纯 setTimeout 不翻）→ `dt=html kids=2` shim 侧保声明而 DOM_SOURCE quirks——**真根因 = 宿主重建源丢文档声明**：runner 探测环（take_probe mutation_version 门控）dump `documentElement.outerHTML`（DOM §4.4 元素序列化不含 doctype）喂 `refresh_if_html_changed` 整页覆盖 cached_html → re-parse 无声明落 Quirks → compatMode=BackCompat + scrollingElement 分派 body → `scrollTop` 读 0；**波及面 = 所有 testharness 页首个 mutation 批后的探测环刷新**（quirks 敏感断言面才显形）。修复：webview.rs `refresh_if_html_changed` 入差分/替换前 `restore_live_doctype` 归一（活文档有 doctype 子节点 → `doc.outer_html(dt_child)` 前缀补回，同入 runner_probe_html 差分基线与 cached_html 比对形态稳定；真 quirks 页透传不凭空补；首个修复刷新后重建源恒带声明——自持续；共享路径单点修优于 runner 两 dump 点各补）+ webview Cargo.toml 加 zero-dom workspace 依赖（`zero_dom::NodeKind` inline 路径）+ basic.rs 常驻钉 `test_runner_refresh_preserves_doctype_s401` 两面（带声明页刷新恒 CSS1Compat + quirks 页透传仍 BackCompat，s371 强制分叉同款相位）。验证：探针复跑 marks 恒 CSS1Compat + `seTag=HTML seT=650`；driving solo 1F→P 三跑全绿（solo+sweep1+sweep2）；the-input-element 域 solo 1848 行零位移（fdtcb 1T=ZeroWeb-2 并发编译撞负载 `/proc` cwd 核归属，×2 复跑 80P 定谳；`zz-s401-focus-scroll-probe.html` 残留=前段 429 会话探针 0，清理后重跑干净+顺带销案 s400「radio 探针残留案」挂账）；全树 sweep 双跑 join 对 s400 基线（unique 8473，LC_ALL+C grep -a+awk hash-join）**Fail→Pass ×1 恰 driving、stable 双跑位同（除 2 键）、Pass→Fail ×0**（home-end 双键 T↔F 在册 flake 家族单列；dialog-overlay 渲染流 flake 键本轮双跑全绿归属渲染流复核；srcset `\r` 子测行=s400 截断族漏记 solo+双跑恒 P 补记；gone ×1=driving 文件 probe diagnostics 探针行出账非位移；raw cur unique 8474=8225P/216F/33T vs 基线 8222P/218F/33T）；单元 webview **849P/2F**（=s400 +1 新钉，2F 在册逐名同形）；smoke 双 PASS；fmt/clippy clean（过程记三点：①U+E015 PUA 字面不可见——探针 `keyDown("")` 实含 EE 80 95 字节，显示空串致 Edit/heredoc 找不到「原串」，`cat -A` 核字节为准；②join 键字段反转——keyed 文件 `key\x1fstate` 用 field 2 做 join 键 = state 分组叉积假挂起，join 键恒 field 1 或对 pairs 直接 awk hash-join；③s400 headline「217F」= raw 218F 剔探针行单列记法，口径连续性换算先对 raw 文件重算）。**本流域在册红 218F+31T→216F+31T**（−1 driving 修复 −1 radio 探针残留行出账销案 −1 dialog-overlay 按事实出账归属渲染流复核）。残差：脚本移除 doctype 节点后重建源再丢声明架构面（compatMode parse 派生非存储字段，FIXME 单列）。账本
  [evidence/2026-11-11-m2s401-refresh-doctype-quirks-flip.md](html-semantics-compat/evidence/2026-11-11-m2s401-refresh-doctype-quirks-flip.md)。
- **2026-11-11 M2 切片 400**：object scheme 取回面 + window.length 施工轮（driving 6F→P，shim 单文件+桥测）——s399 新显形 6F 勘根推翻「fallback 内容 DOM 化」设想（allowed-schemas 4F 断言消息 `Unhandled rejection: "error"`、cross-origin 2F `window.length` undefined，两案 fallback 子节点断言均已过→两族真根因 = scheme 解码缺位 + Window length getter 缺席）：part01 `_zwLoadObjectResource` 扩三臂——data: 换 `_zwDataURLProcessor` 解码（fetch() 通路同源；null→error、text/html essence→done 物化、其余→'image' success 无 navigable）+ blob: 走 `_zwBlobStore` 命中（origin 门正则同 fetch() scheme-fetch + `_zw_blobBytes` + TextDecoder；miss/跨源→error）+ bare-origin http(s) 合成空 text/html 嵌套文档（路径空或仅 /?/#；URL.canParse + _zwFetchBadPort 双门；FIXME 服务端目录语义）+ 抽 `_zwObjectMaterializeDoc` 五路共用物化核（about:blank/data/blob/http/bare-origin 收编 s398 内联段；entry 增 sel/handle 快照）+ host fetch 分诊**保持 s399 三臂**（其余类型→error——runner 对 bogus-scheme URL 宽松 2xx 信号不可信，object-events file:/http: error 案 load-bearing，首跑回归即证）+ part01 新增 `_zwWindowLengthCount()` + `Object.defineProperty(globalThis,'length',{get,enumerable:true,configurable:false})`（[LegacyUnforgeable] 形态，装 R177 frames 后；iframe 计 `_iframeDocCache` win 有且非 `_zwSwDestroyed`、object 计 `_zwObjectNavCache` win 有且 `_makeProxy(sel,handle).isConnected`——s96 先例；error 态无 win 自然不计）。part102 桥测两钉七相位（A-G data:/blob: 面：text/plain 替换内容 load+null 保持、text/html win+doc 物化+location.href 印记、无效 data: RFC 2397 failure error、blob 同源物化/text/plain null/跨源 error/revoked error、host fetch 零调用日志断言；L 相位 window.length 0→1→3→error 不计→移除递减→destroyed 递减 + descriptor enumerable/configurable:false + bare-origin 不走 host fetch）。验证：driving solo the-object-element 目录 40P（allowed-schemas **6P** 4F→P、cross-origin **2P** 2F→P、historical/object-events/object-attributes 4P 保持、object-javascript-url 1P+1T 半开、object-handler T 挂账）+ document-getters 3P 回归 exit 0；全树 sweep 双跑 join 对 s399 基线**迁移矩阵 Fail→Pass ×6 恰 driving 集、Pass→Pass ×8216 零转红零丢失、零 new/gone**（Fail→Timeout ×2 = select-home-end flake 家族真态 F 单列；dialog-overlay 1 键双跑不稳定=渲染流负载 flake 不入稳定集；stable 8222P/217F/33T unique 8473）；单元 engine 3955P/7F（= s399 3953P +2P 本切片新钉；7F 在册逐名同形）、webview 848P/2F 同形、script-sandbox 452P/0F；smoke 双 PASS；fmt/clippy clean（过程记四点：①bare-origin 正则过宽——`http://test:test` 无效 port 经 `[^\/]*` 剥离路径空误入合成臂派 load，须 parse 合法+bad port 双门；②host fetch 分诊翻 success 是反回归——s399「其余类型→error」臂对 runner 宽松 2xx load-bearing，file: 案首跑即翻 load，回退三臂、text/plain success 面由 data:/blob: 臂内语义承载；③grep 二进制日志静默零案——sweep 日志含 NUL（image body 落 Fail 行），无 `-a` 只出提示行 pairs 缩水 3734 行（s344 坑重踩）；④join 漏 `-t '\x1f'` 键含空格时首 token join → 85 万行叉积）。**本流域在册红 224F+31T→218F+31T**（−6 driving）。残差：bare-origin 目录语义 shim 近似 + host 非 text/html 非 image→error 保持（FIXME in part01）+ window.length frame/未消费 iframe 惰性物化不计 + object-handler 嵌套脚本通道（R206 移植面）+ object-javascript-url 剩余子测。账本
  [evidence/2026-11-11-m2s400-object-schemes-window-length.md](html-semantics-compat/evidence/2026-11-11-m2s400-object-schemes-window-length.md)。
- **2026-11-11 M2 切片 399**：object 动态加载链施工轮（driving 8 键 T→P，shim 三点+桥测扩面）——s398 惰性物化半补齐 spec #the-object-element 处理模型**触发面**：part01 `_zwLoadObjectNavigable` 泛化 `_zwLoadObjectResource`（fireEvents 终态事件面：2xx 含 image/* 替换内容→trusted load、fetch 失败/非 2xx/无法渲染→trusted error、`fired` 门终态一次 + _defer 微任务 + __zwMakeTrustedEvent 出口 bubbles/cancelable=false；about:blank 特例不 fetch 直建空 doc+win（iframe R130 语义）；data: URL 显式门不物化——FIXME data:text/html 留）+ `_zwStartConnectedObject` 插入步（与 _zwStartConnectedIframe 同形 _defer 异步，part04 appendChild/append 两循环同排调用，parser 扫描不做）+ part04 contentDocument 扩臂（同管线物化 fireEvents=false 读不触发，#dom-object-contentdocument）+ part05 data IDL setter 臂（R5009 URL 反射写臂加 `!(OBJECT && data)` 排除同款 + isConnected 门 + 旧 entry 删除重启语义，detached 只写属性）+ `_zwMakeIframeWin` 挂 `win.Window`（named FE prototype=_zwChildWinProto37，instanceof 沿 win 原型链命中，iframe/object 共用）。part100 钉①(f) 翻转（contentDocument 恒 null 边界→物化面）+ part101 桥测新建两钉六相位（A trusted load 七 flags+嵌套文档 origin；B 404 trusted error；C about:blank instanceof；D detached 只写属性；E image/* load+null 保持；F 重启序列 error,load）。driving solo historical 4P + object-events 4P + document-getters 3P 回归；全树 sweep 双跑 join 对 s398 基线位移精确归因（sweep2 stable 8217P/225F/31T unique 8473；gone 5T = 4 object 文件级 T + s398 探针 T；new 14 = 8P driving + 6F 新显形——allowed-schemas blob:/data:/http(s): 4F + cross-origin fallback 2F 为「管线推进后测试体首次可完成」的未竟面显形，8209 基线 P 零丢失零转红）；单元 engine 3953P/7F 在册同形（part100/101 四钉在列）、webview 2F 基线同形、script-sandbox 452P/0F；smoke 双 PASS；fmt/clippy clean（过程记三点：①R5009 URL 反射写臂抢占 OBJECT.data——`_REFLECTED_URL_TAGS` 表内成员 `obj.data=` 落 961 行纯属性写臂即返回，专用臂永不可达，按 IFRAME/IMG/SCRIPT src 先例加排除；②连接态判定 `_ceParentConnected` 对 plain handle 元素恒 false（`_ceConn` 只记 custom/plain 对象章），须 script s96 同款 `_makeProxy(sel,handle).isConnected===true`；③桥测闭包持 `&mut sandbox` 跨 execute 触发 E0499（s398 E0596 同族），改文件级自由函数传参）。**本流域在册红 217F+37T→224F+31T**（+6F 新显形承接 +1F select-home-end 非 detailed 键 flake 家族真态并账；driving 8 键 T→P）。残差：object-handler 嵌套脚本通道（R206 移植面）+ object-javascript-url T + 新显形 6F（fallback 内容 DOM 化/blob: 通道）散单池留存——fallback 渲染切片为下一候选（s400 勘根改道承接）。账本
  [evidence/2026-11-11-m2s399-object-dynamic-load.md](html-semantics-compat/evidence/2026-11-11-m2s399-object-dynamic-load.md)。
- **2026-11-11 M2 切片 398**：OBJECT contentWindow 嵌套浏览上下文面施工轮（2F→P，shim 双点+桥测）——spec #the-object-element 处理模型收口：part01 `_zwObjectNavCache`（独立缓存，不混 `_iframeDocCache`——object 嵌套 navigable 不走 iframe SW 观察/history 账本/servedDoc 成熟翻转链）+ `_zwLoadObjectNavigable`（首读惰性物化：无 data/fetch 缺席/非 2xx/Content-Type 非 text/html → null；text/html → `_zwMakeIframeDoc`+`_zwMakeIframeWin`+win.name 创建时点 name 内容属性快照——object-attributes「name 改后 contentWindow.name 仍 'o'」断言链）+ part04 R5274 臂拆臂（contentWindow 接管线、contentDocument 恒 null 边界保持、getSVGDocument/EMBED 不动）+ part100 桥测两钉九面（含 404/无 data/image/* 三 null 面+同 identity+R5274 恒 null 保持）。driving solo object-attributes 4F→4P 文件全绿；全树 sweep 双跑 join 对 s397 基线 gone/new 恰 driving 2 对零意外位移（sweep2 8209P/218F/37T unique 8464，全域 218F 含他流 radio 探针残留 1F 单列——共享 gitignored 语料上渲染流探针把上游断言体替换成恒红诊断，非本流面归渲染流处置，s343 先例 headline 排除）；单元 engine 3843P/7F 在册同形（part100 两钉在列）、webview 848P/2F 基线同形、script-sandbox 全绿；smoke 双 PASS（legacy 42/42 struct PASS）；fmt/clippy clean。过程记两点：①自建 iframe win.name 探针实测 win 字面量无 name 槽（browsers 域 browsing_context_name 族欠账不属本流域）且 `<iframe` 字面主源被 skip 谓词静默排除零案 exit 1（s341 坑复现）；②pairs join 连环伪影两轮——`tr ' '` 全转子测名内空格/单侧转换字节形态不一致均产 2.3MB 假位移，两侧同 `sed 's/ /\x1f/'` 首空格形态才闭合（s287/s292 教训重演）。**本流域在册红 219F+37T→217F+37T**。残差：object contentDocument 访问面（historical/object-handler）+ object-events 动态 data IDL setter 加载链散单留存。账本
  [evidence/2026-11-11-m2s398-object-content-window.md](html-semantics-compat/evidence/2026-11-11-m2s398-object-content-window.md)。
- **2026-11-11 M2 切片 397**：module parse 失败缓存语义施工轮（1F→P，webview tag-5 契约 + shim 态⑤）——s373 parse 预检把 parse 失败发 tag 3（逐次新 SyntaxError）违 module map 失败记账语义；新契约 tag 5 仅 webview compile_module_contract gate 臂发射（native/defer 面零暴露），shim `__zw_load_module` 态⑤首抛 SyntaxError 入 `__moduleFailures[resolved]` 复抛同对象（s87 四态扩五态），part99 桥测钉死 tag5 同对象 identity/tag3 逐次新对象对照。driving solo dynamic-imports-script-error 11P/0F；全树 sweep 双跑 join 对 s396 基线恰 driving 1 对零意外位移（sweep2 8207P/219F/37T unique 8463）；单元 script-sandbox 全绿、webview 848P/2F（2F 预存 env 红 stash 归因首记基线）、engine 3840P/7F 在册同形；smoke 双 PASS；fmt/clippy clean；**在册红 220F+37T→219F+37T**。同轮分诊定谳（s392 方法论）：current-pixel-density 10F 两根（basic 3F=sizing.rs R695 `.max(1.0)` 钳制根→渲染流；error 7F=broken img 尺寸供给根→渲染流+shim IDL 臂配套本流）、svg-img-density-vs-zoom 3F=CSS zoom 面→渲染流、dialog 居中簇 8F=R-s31 在册、tabular 4F=表布局亚像素归渲染流、forms 主池 146F 里 142F=customizable-select 特性族（簇 D 待用户决策）。账本
  [evidence/2026-11-11-m2s397-module-parse-failure-cache.md](html-semantics-compat/evidence/2026-11-11-m2s397-module-parse-failure-cache.md)。
- **2026-11-11 M2 切片 396**：object/embed width/height 内容属性 dimension hint 施工轮（4F→P，style-system 单臂零 shim 改动）——HTML rendering #attr-dimensions 同口径（rules for parsing dimension attribute values，html_length_attr 与 img/table 臂同型）；单测四面钉死（object 100/50、embed 100/100、50% 透传、无属性零 hint）；iframe/video 同属规范面但语料无 driving F 面不扩臂。driving solo 4F→4P（object-attributes width/height "auto"→100px/50px + embed-dimension exit 0 全绿；object.contentWindow 2F 独立面不动）；全树 sweep 双跑 join 对 s395 基线 gone/new 恰为 driving 4 对零意外位移（sweep2 稳定复跑 8209P/220F/37T unique 8463）；select-home-end-pagedown-pageup.optional T/F 时序 flake 定谳单列（同二进制 T/F 双跑翻转+零 object/embed 零 width 属性机制排除+solo 恒 Fail 真态，s393/s394/s395 三代 sweep 恒 T）；单元 style-system 2516P/0F、engine 3835P/8F（7F 在册同形+r5296 渲染流 R5484 已在册 env 竞态本轮命中）零新增红；smoke 双 PASS；**在册红 224F+37T→220F+37T**。承上一 session 429 残件（WIP+clippy 已就绪）接手完成全链验收。账本
  [evidence/2026-11-11-m2s396-object-embed-dimension-hints.md](html-semantics-compat/evidence/2026-11-11-m2s396-object-embed-dimension-hints.md)。
- **2026-11-11 M2 切片 395**：动态 append rect 供给链勘根+施工轮（2F→P）——五探针钉死：flush 后 host 全可见
  （P6 rect 8,58,70,20）、缺口=in-process rect 回调无 handle→selector 臂 + shim proxy 身份恒 handle-only；施工
  shim 单侧三 helper（_zwRectIdent/_zwLocalDefiniteRect/_zwLayoutRectTrap）+五 trap 消费面换接，`_layoutRect`
  裸语义不动（img IDL 臂 945 隔离 s284 attr 期望面雷区）；naturalWidth 金丝雀 join 匹配域 258/258 零真位移，
  全树 join 2F→P 精确归因零意外位移，在册红 226F+37T→**224F+37T**；单元 3834P/7F 在册同形、smoke 双 PASS。
  残差：无 id+无 definite 尺寸 flush 前仍 0、forward map 空返挂账低优先。账本
  [evidence/2026-11-11-m2s395-dynamic-append-rect-supply-chain.md](html-semantics-compat/evidence/2026-11-11-m2s395-dynamic-append-rect-supply-chain.md)。
- **2026-11-11 M2 切片 394**：双刀轮——textarea wrap=hard wrapping transformation 施工（2F→P）+ img unloaded
  attr sizing 归属定谳（2F 挂账，s394 内不施工）。刀①三处落点：part02 `_zwTaHardWrap` 共享 helper（`_zwFdEntry`
  邻位跨 part 作用域先例）+ FormData 构造器泛化臂 + part04 form 提交 TEXTAREA 臂（urlencoded/multipart/
  text-plain 三序列化单点覆盖）；刀②探针矩阵三轮（动态 attr/CSS/nosrc + 静态 img/div 对照 + sel 身份 2s）
  三面剥离——attr→style hints 健康、静态 geometry 健康、动态 append rect 供给链缺席定谳，修复须先勘 live doc
  发布与 mutation 同步链（s284 收窄面 + naturalWidth 30 行回归史为施工雷区）。join 2F→P 精确归因零意外位移，
  在册红 228F+37T→**226F+37T**；验收管线五阶段串行（driving solo→删探针→全树 sweep→单元 -j 2→smoke 双），
  在册 7F 单元红集同形零新增。账本
  [evidence/2026-11-11-m2s394-textarea-wrap-hard-dynamic-rect-attribution.md](html-semantics-compat/evidence/2026-11-11-m2s394-textarea-wrap-hard-dynamic-rect-attribution.md)。
- **2026-11-11 M2 切片 393**：双小刀施工轮——output value mode flag（1F→P）+ label 转发壳自递归 R315 翻转投毒
  勘根修复（2F→P）。刀①defaultValue setter 双态：override null → 文本写+视图重注册（R49/r2846 同链），非 null →
  仅覆写 override；刀②勘根三探针（栈捕获/身份/时间线）定谳 `_recordHandleChild` R315 翻转把盖章 plain stub 写入
  `_proxyCache` → `_makeProxy` 返回非 proxy → R4004 壳自 Feed 无限递归，修复=翻转 own-`addEventListener` 守卫
  （R315 页面持有 proxy 面零变化）。全树 join 位移全归因（3F→P+4 键 zw391_probe 语义性消失，s392 预测 8467 勘误
  →实 8463 唯一键），零意外位移；在册红 231F+37T→**228F+37T**。单元测试 -j 2 降并发过守卫（与并行流 build 内存
  争用两次被 guard 杀，阈值不动）；stale done 旗标 mtime 核新鲜度教训。账本
  [evidence/2026-11-11-m2s393-output-value-mode-flag-label-proxy-cache-poison.md](html-semantics-compat/evidence/2026-11-11-m2s393-output-value-mode-flag-label-proxy-cache-poison.md)。
- **2026-11-11 M2 切片 392**：forms 树散单余红分诊 + img 域勘根（承上一 session 429 残件）+ 自建探针口径卫生轮
  （零源码改动，17 键归属定谳，探针键 −37）——口径卫生删 25 个 gitignore 本地探针（select 27 行/forms 根 s327 10
  行/zzz-probe 等 0 行），下次全树预期 8199P/231F/37T=8467 键，select 域 corpus 口径回归 146F+2T（域 solo
  182P/146F/2T 精确验证）；**教训钉**：scripting-1/module probe8/9/10 是 s364 转正的 git 跟踪回归钉，误删即被
  status D 暴露并恢复——探针清理前必须 `git ls-files` 核对跟踪态。img 域 6 案 20 键勘根：basic 3F 亚像素盒
  取整 + error 6F broken-image alt-box 缺席 + svg-zoom 3F page zoom 面 + sizes-auto 4F+1T style 几何臂，四案
  归 rendering-compat 挂账；not-broken 2F（未加载 img 宽高属性→盒面）为流内 s394 候选。forms 散单 12 键：
  output 1F value mode flag 缺席 + label 2F img 深嵌点击无限递归（stack overflow）双入 s393 施工候选；
  textarea wrap=hard 2F 折行缺席 s394 候选；lineheight 1F 亚像素 gBCR rendering 挂账；button-click-submits
  1T solo 全绿=sweep 负载 flake；form-submission-target 6T 真实提交导航面归 navigation M3 门控挂账。
  账本
  [evidence/2026-11-11-m2s392-forms-scattered-triage-probe-hygiene.md](html-semantics-compat/evidence/2026-11-11-m2s392-forms-scattered-triage-probe-hygiene.md)。
- **2026-11-04 M2 切片 268**：css/json/text-module referrer-checker
  端点仿真 + 元素级 referrerpolicy 图穿参轮（s116 挂账
  "text/css/json-module referrer-policies（3T）"收口，engine +
  webview + runner 三车道 3 文件 3 刀 3 钉；三页 **3T→24P**）——
  勘根：上游 checker 体形逐字镜像（css `.referrer{content:"<R>"}`
  / json `{"referrer":"<R>"}` / text 裸体、no-referrer 臂**裸空
  体**）+ 有效策略三层（元素 referrerpolicy > pipe header > meta
  > SWCO 默认）；端点臂须前置 `__zw_mtype` intent 臂前（s267 同
  理由）；text 空体撞 fetcher 契约「空体=取回失败」防线→图收集
  放宽 `type=text` 空体合法；**方案伪影定谳**——runner https 文
  档锚 vs 语料 http token URL，跨源 default 策略臂误落
  `wpt_referrer_is_downgrade` 降级臂回空（首跑 solo 21P/3F），
  **曾试合成 host 豁免改 → 8 job 族 strict-policies strict-*
  P→F 回归**（HEAD stash solo 实证原本 Pass，s116 原口径不可侵）
  → revert，改走 s116 先例 `referrer_case_uses_http_page` 扩 3
  页（页期望锚 `(new URL(location.href)).origin` 的 http 形）。
  施工：engine `extract_script_referrer_policies`（镜像
  extract_script_csp_info 枚举、trim 空体→None、序数对位）；
  webview `collect_module_deps_recursive` +`referrer_policy` 形
  参递归穿参（静态图位 `script_referrer_policies.get(idx)`；
  动态 import 三调用点 None + FIXME；text 空体放宽；
  too_many_arguments 8 参形）；runner referrer-checker.py 三域
  端点臂（`__zw_lp` > pipe > doc policy > SWCO 默认，经既有
  `compute_module_referer` 回体，css/json/text 三体形）+ http
  页锚扩 3 页。验证：3 钉绿 + runner bin **285P/0F/76I**（284+1）
  + solo **24/24**（日志 evidence/logs/s268-solo-final.log）+
  豁免改回归证据链在档（s268-family-exemption-regression.log）+
  族跑终态 **80P/1F**（残红 image-loading-lazy-referrerpolicy-
  change 为 A/B 双红零位移在册独立面；s268-family-final.log）+
  全树 A/B **7814P/431F/72T→7836P/433F/69T** join 精确 24 翻绿
  （3 文件 Timeout→24 子测 Pass）、REGRESSED=∅（checkable ×2
  漂移族 B 日志同文件混态自证销账）+ make test **3722P+5F**
  （R-t2qq adopted-stylesheets 既有，s265 账本同集，本流不入侵）
  + fmt/clippy 干净。挂账：referrer-policies ×3 **销案**；新记
  dynamic import() 元素级 referrerpolicy 穿参（零 corpus face，
  随语料扩展补）；承前 image-loading-lazy-referrerpolicy-change
  1F 勘根入候选。账本
  [evidence/2026-11-04-m2s268-data-module-referrer-checker.md](html-semantics-compat/evidence/2026-11-04-m2s268-data-module-referrer-checker.md)。
- **2026-11-03 M2 切片 267**：serve-custom-response.py 端点仿真轮
  （s266 下一步候选 #1，runner 车道 2 文件；repeated-import 三页
  **3F→3P**）——勘根：上游端点语义（wptserve stash 按 `key` 记账
  ；stat 读数不计数 / reset-stat 清零 / set 更新配置；无 action 模
  块取回按 content-type 回体 + 计数，语料端点文件保持缺席 runner
  仿真）；投递拓扑 = 页面 fetch() action 面落 fetch handler、运
  行时元素装载 + 动态 import 落 script fetcher，两端共享案级状态
  ；**跨案竞态实证**（8 job 同进程线程 + 页 B/C fixed token 共用
  → 进程级全局表互踩：B error 步拿 C success 配置误 Ok，strict
  solo B 四步全过归因）；`__zw_mtype` intent 臂前置接管 css 端点
  取回 Err 不计数（页 A stat "0"）；`rewrite_dynamic_imports`
  `import(` 吃进 `removeScriptReimport(` 标识符 → ReferenceError
  ；`__zw_load_module` 先 host 后查缓存 → 复 import 重取（页 C
  stat "1"≠"0"）。施工：案级 stash（per-case Arc，fetch handler +
  script fetcher 两闭包共享，单测 None 回落进程级静态表、两既有
  wrapper 转 `#[cfg(test)]`）+ 端点模块取回臂前置 mtype 感知 MIME
  门（text 无校验 s250 口径 / css 按 module type map / 默认
  JS∪json∪css s107 同集）+ delivered_ok 交付缓存（首 Ok 记账回体
  、复取不计数、失败不记账）+ `is_identifier_continue_byte` 守卫
  两改写点 + `__zwModuleFetchKeys` 取回键缓存（键
  `spec␟parent␟typeHint`，host 调用前先查、成功记账失败不记）。验
  证：状态机钉 `serve_custom_response_endpoint_state_machine_s267`
  + 改写守卫钉 `test_rewrite_skips_import_suffix_of_identifier` 绿
  + 沙箱 **396P/0F**（395+1）+ solo 页 A 3 子测试全 P（error/404/
  MIME/成功/串并行 ×5 判据全过——s249 CSSStyleSheet 合成真实通路
  ）+ solo 页 B 5 步 P + **8 job 并行族 5/5**（日志
  evidence/logs/s267-repeated-import.log）+ 邻域 FILTER="module"
  330P/43F 对位 s261-ab 全域基线模块子集 37F **REGRESSED=∅
  FIXED=6** + runner bin **284P/0F/76I** + fmt/clippy 干净。挂账
  注记：delivered_ok 端点域仿真——webview 元素装载面真实 module
  map 仍缺席（s256 部分收窄保留）；fixed token 跨案共享为语料形
  态注记，后续端点类挂账沿用案级 stash。账本
  [evidence/2026-11-03-m2s267-serve-custom-response-endpoint.md](html-semantics-compat/evidence/2026-11-03-m2s267-serve-custom-response-endpoint.md)。
- **2026-11-03 M2 切片 266**：module 外链 MIME 门轮（s265 下一步候选
  #1，1 臂 + 1 钉；script-element MIME 三页 **3F→3P**）——勘根：spec
  fetch a single module script 非 JavaScript MIME = 网络错误（error
  事件 + 体不执行），classic 外链无 MIME 门（nosniff 之外）；runner
  fetcher scripty 门对页面脚本单元是死代码——`inline_local_scripts`
  （testharness.rs:10385）预投递末臂读盘在册即 200 投递 + load 尾无
  MIME 检查，module 单元投递尾按 status fire → 误 200 即误 load；
  探针四臂实证（module .js load / module .css 误 load=违例本体 /
  classic .css 双事件皆无=SyntaxError 杀尾注记 / `script[src]`=0 面
  在预投递层）；DOTALL 翻转面扫描 25 处 = 3 在册（css-module
  script-element-css-src、json-module 与 text-module
  script-element-json-src，判据全 `["error"]` + unreached load）+
  22 缺席 `.py` 端点透传不变。施工：末臂 `is_module_type` scripty
  派生（pipe header / `.headers` sidecar / .js .mjs 扩展名；非
  scripty→404 error 尾 + 体不入页）+ 钉
  `inline_local_scripts_module_non_scripty_src_error_delivery_s266`。
  验证：红态 dump 误 load + inline 组 3/3 + runner bin 全量
  **283P/0F/76I**（per-proc 帽 runtime_path_tests 正当触发升至
  `--per-proc-mem 8 --total-mem 16`）+ fmt/clippy 干净 + solo 三页
  **3F→3P**（日志 evidence/logs/s266-script-element-mime-gate.log）+
  DOTALL 重扫零其他 face → A/B 免跑（s258 先例）。挂账注记（零
  corpus face）：classic SyntaxError 体杀投递尾形状不扩面。账本
  [evidence/2026-11-03-m2s266-module-mime-gate.md](html-semantics-compat/evidence/2026-11-03-m2s266-module-mime-gate.md)。
- **2026-11-03 M2 切片 265**：shim CSSOM 规则文本输入预处理轮
  （s264 下一步候选 #1，1 刀 + 1 钉；charset 家族 5 页 **14/14**
  收口）——勘根：s264 残 2F 位对位归因（expected FFFD mojibake vs
  got NUL 交错逐位对应；0x00 合法 UTF-8 解码臂保留，FFFD 化为 CSS
  Syntax §3.3 解析入口预处理职责）；rust css-parser tokenizer 已有，
  缺口仅 JS shim `_ruleFromText`（part06.js:2041，insertRule /
  adopted / replaceSync 三调用点共享单点）；替换不增删 `{}` 与
  replaceSync 先切分顺序无涉。施工：`_ruleFromText` 入口
  `.replace(/\0/g, FFFD)` + 钉（replaceSync 臂 corpus 原形位对位 +
  insertRule 臂）。验证：红态 HEAD FAIL `false|false|1` + engine
  lib **3721P/5R**（R-t2qq 既有同集，stash 法归因无涉）+
  fmt/clippy 干净 + combined shim node --check OK + solo
  charset-bom **3/3**（日志 evidence/logs/
  s265-charset-bom-after-nul-fix.log）+ 全树 NUL 扫描 12 载体
  分类零其他 face → A/B 免跑（s258 先例）。挂账新记（零 corpus
  face）：声明块直读路径（cssText setter / style 属性 →
  `_parseDeclarations`）未预处，按语料扩展随补。账本
  [evidence/2026-11-03-m2s265-css-rule-text-nul-preprocessing.md](html-semantics-compat/evidence/2026-11-03-m2s265-css-rule-text-nul-preprocessing.md)。
- **2026-11-03 M2 切片 264**：css-module/text-module charset 语料资产
  补拉轮（零引擎刀；2 脚本挂账 + 7 资产本地补入）——勘根：5 页
  file-Timeout 全为资产缺席（③b 旧注记「端点落地时再补」失效，端点
  s115/s249/s250 已落地销案）；字节源甄别反转：上游 master 重编
  bom-utf-16be.css 与 corpus 判据不符，权威源 = lib.sh WPT_REV pin
  版（windows-1250.* pin 版本即含 U+FFFD 序列，corpus 判据按该形
  写定）；BOM 刀反转销案——charset-bom 判据（恒 UTF-8 解码 / charset
  参数忽略 / BOM 忽略）与既有 UTF-8 lossy 解码臂逐位吻合，「charset
  参数/BOM 嗅探未建模」挂账销案。施工：7 资产按 pin 版补入 + ③b
  白名单 7 条 fetch_raw + update-wpt-data.sh known-gaps 条目。
  验证：solo 双跑 **12 子测 Pass**（css-module/charset 5P + charset-2
  1P + charset-bom 1P/2F；text-module/charset 5P + charset-2 1P），
  残 2F = shim CSSOM `_ruleFromText` 缺 CSS Syntax §3.3 NUL→FFFD
  预处理（engine 车道 s265 候选，资产轮引擎刀不混切）。日志
  evidence/logs/s264-charset-assets-{css,text}.log。账本
  [evidence/2026-11-03-m2s264-charset-corpus-assets.md](html-semantics-compat/evidence/2026-11-03-m2s264-charset-corpus-assets.md)。
- **2026-11-03 M2 切片 263**：data-zw-harness 注入序勘定轮（零施工
  销案；保真族 s255 D 刀→s261 拆分→s263 量化收口）——runner 基建
  注入形 vs 上游双标签装载序三子面全树量化：2759 页双标签 0 页夹
  内容（合并注入不可观察）；宽松 contains 假阳性 20 页注释性引用
  不进 testharness 枚举 + 1 页空格形误命中正常起跑；属性丢弃仅 1 页
  type="text/javascript" 无害——零观察面销案，s255 D 刀余项清空。
  账本
  [evidence/2026-11-03-m2s263-harness-inject-order-closure.md](html-semantics-compat/evidence/2026-11-03-m2s263-harness-inject-order-closure.md)。
- **2026-11-03 M2 切片 262**：import map 注册序面轮（s260 挂账 #2 三面
  勘根分型，1 文件 2 刀 + 2 钉 + 1 知识面）——勘根：sort and normalize
  两算步收尾均为代码单元降序（规范注记 `"foo/bar/"` 先于 `"foo/"`），
  本实现升序迭代入 Vec 首中生效——嵌套前缀键最短遮蔽、嵌套 scope 最浅
  遮蔽方向恰反；多张 map（spec merge + resolved module set，现行首张
  合法生效）与动态插入（仅静态 HTML 抽取）两面勘根落账架构级缓议
  （corpus 零 face，挂起待拍板）。施工：code_unit_cmp 助手（UTF-16
  代码单元精确序）+ imports/scopes 注册收尾双降序 + 模块 doc 偏差注记
  改写 + s258 钉存储序断言随新序更新；知识 modules.md 边界条目重写
  （s260 三态化欠账一并事实化）。验证：红态两钉 HEAD FAIL 恰反序实证
  （short/shallow 遮蔽）+ script-sandbox 395 / webview 839 / renderer
  import 过滤 9 / browser blocked 过滤 4 全绿 + fmt/clippy 干净 +
  全树 A/B 免跑（s258 先例 corpus 零排序 face，消费面套件兜底）。
  账本
  [evidence/2026-11-03-m2s262-importmap-sort-order.md](html-semantics-compat/evidence/2026-11-03-m2s262-importmap-sort-order.md)。
- **2026-11-03 M2 切片 261**：data-zw-harness 投递标记保真轮（s260 下一步
  候选 #1 = s255 候选 #3 收口，3 面 4 文件 + 4 钉）——勘根：runner
  `inline_local_scripts` 页面内容单元与基建统一打 `data-zw-harness`，
  webview CSP script 检查点与 eval 门禁包装（s253）均以 `!zw_harness` 为
  豁免门——CSP 页所有外链页脚本借基建标记整体逃逸 `script-src`（s253
  「错进错出」面），真实浏览器对外链脚本做源匹配。施工：runner 页面单元
  改打 `data-zw-inlined`（`data-inline` 记录原始 src，基建保持 harness）+
  extract `ScriptElementCsp.inlined_src` + webview 检查点
  `external_src.or(inlined_src)` 按原始 src 走既有外链臂（nonce 照常参与，
  纯内联 hash 臂与基建豁免不动）。验证：红态实证三面（runner 投递钉 +
  webview 同源/异源双钉 HEAD FAIL；eval 伴随钉 HEAD 绿 = 护轨性质）+
  webview 839 / runner 282 全绿 + engine 3720+5 与 HEAD 同集零 delta +
  fmt/clippy 零告警 + solo security 24P/1T 基线逐位同 + 全树 A/B 行级
  7794P/439F/11N/77T 双侧逐数同、(key,state) 对 8318/8318 LOST 0/GAINED 0
  ——完美 A/B 零位移（改动臂只在 CSP 页生效，非 CSP 页双门不激活）。
  s253「CSP 页外链页脚本整体豁免」挂账销案。新记档：engine lib 5 红
  （R-t2qq adopted-stylesheets 系列既有，归因该流自查）。账本
  [evidence/2026-11-03-m2s261-delivery-marker-page-content.md](html-semantics-compat/evidence/2026-11-03-m2s261-delivery-marker-page-content.md)。
- **2026-11-03 M2 切片 260**：import map Blocked 终止跨面接线轮（s259 挂账
  收口，5 刀 + 5 钉）——勘根：resolve_mapped `Option<String>` 签名把
  Blocked（null 条目）塌缩为 None 与 Miss 不可区分，五面接线侧（webview
  shim `__zw_compile_module` / native resolver、renderer collect_module_deps
  与 `__zw_compile_module` handler、tab_js_worker 同形两处、script_prefetch
  静态依赖展开）一律按 Miss parent-join 继续抓取——页面显式阻断的 URL
  fetchable 即误装载（规范：null 条目解析失败，不得回退）。施工：resolve_mapped
  三态化（Resolved/Blocked/Miss 直报，base 解析失败守卫臂保持 Miss）；webview
  apply_import_map Option 化（映射命中按 base 绝对化 / 未命中原样 passthrough
  / Blocked None）+ 两调用点 let-else 空契约（shim 契约串与 native resolver
  返空串 → v8_runtime TypeError reject 面，与 CSP violation 拒绝同通道）；
  renderer `resolve_import_dep` pub(crate) 共享 helper + collect Err（null
  entry 错误）+ handler String::new()；tab 本地同形 helper（跨 crate 无共享
  面）同接线；prefetch Blocked 跳过（被阻断 URL 属噪声抓取）。钉面 ×5：
  红态钉 renderer/tab collect（HEAD 实证抓取被阻断 URL 且收集成功——bypass
  在账）+ 签名变更同伴 3 钉（resolve_mapped null→Blocked 精确/URL 形态两臂、
  renderer helper 三态、webview apply_import_map 三态——旧 Option 签名下
  无法编译，编译失败不冒充行为失败；webview passthrough 臂勘正为原样契约，
  fetch 侧 join）。验证：修复后 script-sandbox **393** / renderer **371** /
  webview **836** 全绿 + browser bin 740 绿（2 子进程并行负载 flaky 单跑绿
  ——compositor_frame/save_as 不触及 import map 面；5 个 s257 已知 HEAD 红
  不计）+ fmt/clippy 四 crate（browser 双形态）零告警；image_decoder proxy
  钉红 = target-disk-guard 删 `zero-image-decoder` 产物，重建后绿——环境缺
  产物非回归；消费面 grep 核对 resolve_mapped 生产调用仅三处全改造；零
  corpus 面（s258 grep 承用）+ 无 map 页面路径逐字节不变 → 无全树 A/B。
  挂账备注：map 只在顶层契约入口应用（传递依赖从不看 map）——既有形态，
  Blocked 在依赖链终止由 collect Err 传播承担，若 WPT 面要求传递依赖可见
  map 另立切片。账本
  [evidence/2026-11-03-m2s260-import-map-blocked-wiring-termination.md](html-semantics-compat/evidence/2026-11-03-m2s260-import-map-blocked-wiring-termination.md)。
- **2026-11-03 M2 切片 259**：import map Blocked 终止回退 + 回溯臂并刀轮
  （s258 挂账收口，两刀）——勘根：resolve_imports_match 缺规范步骤 6
  回溯判别（afterPrefix 产物越出映射目标前缀域按成功返回）；resolve_registered_specifier
  `String` 签名无错误通道——Blocked 按 Miss 降级（URL 形态说明符命中阻断
  后落回 asURL/相对解析继续内联，裸键落 "Module not found" 错误原因失真）。
  施工 **2 刀 + 2 钉**：回溯刀（前缀键臂补序列化前缀判别 → Blocked）+
  错误通道刀（resolve_registered_specifier String→Result，Blocked →
  RuntimeError 终止全部回退；transform/export 七臂 `?` 直传；定序链
  filter_map 跳过遵循 module_graph_postorder 尽力而为契约；module_provides_export
  星号/显式重导出臂 Blocked 按 Missing 继续不消解其他路径）。钉面 ×2：
  import_map 回溯钉 + compile_dependency_iife 端到端两臂钉（裸键/URL 形态
  键 null 条目均 null-entry 错误终止；resolve 级 Err 断言旧签名下无法编译，
  红态证据落公共 API 钉——编译失败不冒充行为失败）。验证：红态实证（HEAD
  越界 URL 按成功返回 / "Module not found: blocked" 降级）+ 修复后
  `zero-script-sandbox` **392 全绿**（390 基线 + 2 钉，fmt 后复跑同绿）
  + clippy 零告警 + zero-webview check 过；零 corpus 面（s258 grep 承用）
  + 公共 API 签名不变 → 无全树 A/B。不涉面核对：宿主 import.meta.resolve
  回调面本就规范正确（s257 空串 → prelude TypeError）；webview/宿主编译面
  resolve_mapped Option 签名仍降级——收敛为 s260 候选（三态化 + resolver
  错误通道 + 失败 tag 路由三面一次收口）。账本
  [evidence/2026-11-03-m2s259-import-map-blocked-fallback-termination.md](html-semantics-compat/evidence/2026-11-03-m2s259-import-map-blocked-fallback-termination.md)。
- **2026-11-03 M2 切片 258**：import map 映射值注册期归一化轮（s257 挂账
  收口，共享解析层小刀）——勘根：normalize_specifier_map 映射值原样存储，
  resolve_imports_match 对前缀键目标裸 `Url::parse` 无基址——
  path-absolute/relative 值误 Blocked，精确键相对值原样返回（resolve_mapped
  消费方拿裸相对串按导入方基址错误抓取）。规范核实勘定 s257 口径错误：
  sorting and normalizing 步骤 3-6 对值级失败是**置 null 条目并继续**
  （成员仍注册），整张拒注仅限 JSON/结构非法——s257「归一失败整张 map
  拒注册」勘误记档。施工 **1 刀 + 5 钉 + 1 钉数据校正**：normalize_specifier_map
  字符串值经 resolve_url_like 按基址解析存序列化 URL，失败或尾斜杠键地址
  不带尾斜杠 → null（阻断条目）；resolve_imports_match 目标解析失败臂注明
  对 parse 产物不可达（阻断面收敛），保留不变式守卫；钉面 ×5（前缀键
  path-absolute / 精确键相对值含存储断言 / 裸串置 null / 尾斜杠一致性 /
  scope 值）+ non_special 钉数据校正（旧 map 形态不规范，意图不变两侧行为
  等价）。验证：红态实证——5 钉在 HEAD 恰好 5 失败且模式逐一对准目标行为
  （误 Blocked / 原样相对串 / Resolved("bar") / 父级路径解析）；修复后
  `zero-script-sandbox` **390 全绿**（385 基线 + 5 钉，default v8，
  test-guard 守卫）；fmt 零改写 + clippy all-targets 零告警；无全树 A/B
  ——corpus 零 importmap 面（grep 证据）+ 消费面测试 map 全规范形态绝对
  值（三面零漂移）+ parse Err 面不变。新记档：resolve_imports_match 回溯
  臂缺口（规范步骤 8 afterPrefix 越出前缀域应 TypeError，现按成功返回）。
  账本
  [evidence/2026-11-03-m2s258-import-map-value-normalization.md](html-semantics-compat/evidence/2026-11-03-m2s258-import-map-value-normalization.md)。
- **2026-11-03 M2 切片 257**：renderer/tab 宿主 import.meta.resolve 接线轮
  （s254 挂账收口，多进程面对称小刀）——勘根承 s254：两宿主已 parse map
  （`parse_page_import_map` → `Arc<Mutex<Option<ImportMap>>>` 槽位，快照
  换代每代重设）但未注册 `__zw_importmap_resolve`——`import.meta.resolve`
  落 prelude 守卫臂抛 TypeError（回调缺席与 resolution failure 同落点，
  es_module.rs:422 typeof 守卫 + 空串守卫）。施工 **2 刀 + 2 钉**：
  js_worker / tab_js_worker 的 register_module_compile_callback 各补宿主
  回调（webview s254 五刀同形：`map.resolve` → `resolve_document_url`，
  Miss|Blocked → 空串；map 回调时刻锁定读取与 `__zw_compile_module` 同
  模式——锁毒化回落空 map 走 Miss 不放大；宿主 shim 通道 → s256 poll
  泵不涉）；钉面 `renderer_js_worker_importmap_resolve_wired_s257` /
  `tab_js_worker_importmap_resolve_wired_s257`（tab 侧沿用 r386 feature
  gate；worker 真装配路径五断言臂：精确键 / 前缀键组合 / asURL 回退 /
  裸说明符返空 / 换代失效）。验证：红态实证（neutralize 注册名首断言
  挂）；renderer **369 全绿**（368 基线 + 1 钉）；browser bin **741 过 +
  5 失败 = HEAD 既有**（stash 法同跑 HEAD 740 + 同 5 定谳非本轮回归）+
  lib 10 全绿 + cdp_file_scope_policy 2 失败 HEAD 既有；fmt/clippy 零
  告警（v8 + quickjs-only 双形态）；零 runner 路径改动 → 无全树 A/B
  （钉面即门禁）。新记档：**import map 映射值归一化缺口**
  （resolve_imports_match 映射目标裸 `Url::parse` 无基址——
  path-absolute/relative 值误 Blocked；规范要求 register 按基址归一化
  失败整张拒注册；webview 面同缺，钉面用绝对值绕开）。挂账承前不动：
  data-zw-harness 投递标记保真面（s255）、import maps 剩余语义面（多张
  map/动态插入/Blocked 降级）、credentials/shared-dependency/microtask
  时序族/并发慢 import dedup（s256）。账本
  [evidence/2026-11-03-m2s257-import-meta-resolve-hosts.md](html-semantics-compat/evidence/2026-11-03-m2s257-import-meta-resolve-hosts.md)。
- **2026-11-03 M2 切片 256**：动态 import 延迟决议轮（s255 候选 #1，
  delay-load 根因切片 + trickle 慢模块时序族）——勘根含 **s253 判读
  勘误**：「trickle 冻结 JS 回合 2 秒」不成立——runner 脚本 fetcher
  忽略 trickle 管道即时返回（注册处注释自证），真因：native 通道
  `dynamic_import_callback`（s88 V8 原生回调——classic inline 面
  rewrite_dynamic_imports 不可达）脚本回合内**同步**取回+求值+resolve，
  断言微任务早于 window load 事件与 +1s step_timeout 排空（红态
  `got ["import start","slow"]`，spec 判据：import promise 取回完成前
  保持 pending、不阻塞 load 链；
  https://html.spec.whatwg.org/multipage/webappapis.html#hostimportmoduledynamically-retrieve-promise
  ）。施工 **8 刀**：script-sandbox `PendingDynamicImport` +
  `PENDING_DYNAMIC_IMPORTS` thread_local（`v8::Global` 非 Send 不能进
  `Arc<Mutex<DynamicImportState>>`；sandbox_key=`Arc::as_ptr::<Dynamic
  ImportState>` 身份过滤——同线程多 sandbox 存活期重叠只取本条目）+
  回调延迟包裹臂（宿主契约 `1\x1f<delay_ms>\x1f<inner>`，前缀判定
  无歧义同 s252 `3\x1f` 口径；CSP 阻止面同步不延迟、失败契约照慢
  语义）+ `settle_dynamic_import` 共享体（四态 + s124 module map +
  IIFE 求值 + 决议，回调臂与 poll 臂共用——v8 150.2 API 恒取
  `&PinScope`，TryCatch deref 单签名覆盖两调用点）+ `poll_pending_
  dynamic_imports`（本 sandbox 到期条目逐条 settle，IsolateEnterGuard
  栈纪律 + 逐条 WatchdogGuard Arm/Disarm + 每条后 microtask
  checkpoint（t-p103 D2-1 逐跳语义）；超时终止臂 cancel+reset、
  promise 保持 pending 不毒化；队空零成本早退）+ trait 默认 no-op
  （QuickJS 维持不支持态）+ Drop 按键清残条（PENDING_REJECTS 同款
  保险）；webview `WebViewConfig::dynamic_import_fetch_delay`（
  `Arc<dyn Fn(&str)->u64>` 输入解析 URL）+ resolver CSP 后包裹 +
  三探针点泵（execute_script / execute_script_with_dom 头部 +
  drain_pending_timers_until_idle 循环头）；runner 注册
  `trickle_delay_ms` provider（s112 解析器复用，`pipe=trickle((d)N)`
  秒×1000 30s 封顶——trickle 管道从此建模为真实网络秒级延迟）。钉面
  `test_dynamic_import_deferred_contract_pending_until_poll_s256`
  （persistent 镜像生产 sandbox；40ms 包裹三段断言 pending→未到期
  poll 不 settle→到期 settle 恰一次求值；红态实证——neutralize 包裹
  臂后首断言挂，旧版本因目标行为失败）。验证：script-sandbox **385
  全绿**（384 基线 + 1 钉）；webview **834 过 + 1 失败 = HEAD 既有**
  （image_decoder proxy_mode stash 法定谳非本轮回归）；runner 单测
  **281 全绿**（--test-threads=2；默认 8 线程 corpus 单测进程峰值
  4.2GB 撞 guard 4GB 上限被杀——既有负载形态非本切片引入）；fmt/
  clippy 三 crate 零告警（含 useless_format 钉面返修）；quickjs-only
  clippy 干净。solo 终态 delay-load-event **F→P**（s256-solo-
  delayload.log）；module 目录邻接面（s256-solo-module-dir.log 271
  行）fetch-error 8 子测试全 Pass、credentials/shared-dependency/
  TLA 家族维持基线态；全树 A/B（窗口独占）行级 7793P/440F/77T →
  **7794P/439F/77T**，键级 (key,state) **8307→8307 LOST 1/GAINED 1
  迁移恰 1 键 = 目标键全等零其他位移**。已知简化在账：同 spec 并发
  慢 import 挂起条目不 dedup（s124 缓存只同步臂命中面）；shim
  `__zw_compile_module` 通道不包裹（无语料目标）；非持久 context
  poll 落 resolve_context 所得 context。挂账更新：**并发慢 import
  dedup**（新记档）；承前不动：credentials 面（×12 + setTimeout 变
  体）、shared-dependency 取回计数、microtask checkpoint 排空时序
  族（s255）。账本
  [evidence/2026-11-03-m2s256-dynamic-import-deferred-poll.md](html-semantics-compat/evidence/2026-11-03-m2s256-dynamic-import-deferred-poll.md)。
- **2026-11-03 M2 切片 255**：星号导出歧义检测 + module 求值错原值
  报告 + 注册序确定化轮（s254 候选 #1）——勘根四根因：①
  `module_provides_export` 布尔归并无歧义语义（tc39 ResolveExport
  star 臂子解析歧义传播——Node 24 oracle 实证 7 形：7b→7c→{7d,7e}
  子链歧义 + 兄弟臂 7f 可消解**不豁免**仍 SyntaxError；星号歧义只
  影响导入方，7b 自身可链接照 evaluate）；② transform_export 三处
  Fresh star/re-export 内联裸 IIFE 无记忆化（R5182 只做 import 侧
  ——急切臂已缓存仍重跑，ierr-6 got 6c/6d 双份实证）；③ 求值错报
  告双折损：module 体抛错走 script_report_module_failure 重构
  SyntaxError 丢原值对象（error-type-3 `log[1].foo` 对
  `throw {foo:true}` 断言）+ shim reportError 非 Error 折 null（spec
  report the exception 任意 throwable 原值透传）；
  https://html.spec.whatwg.org/multipage/webappapis.html#dom-report-error
  ④ prelude 急切注册 `registry.specifiers()` 哈希序——求值序可观察
  （spec 依赖图 depth-first post-order，ierr-7 got [7e,7d,7c,7f,7b]
  实证）。施工 **A-E 刀**：es_module `ExportResolution` 三态
  （Provides/Ambiguous/Missing——星号臂 star_provides 计数 +
  star_ambiguous 传播旗，显式重导出臂仅 Provides 提前返回；
  Ambiguous→CompileError "conflicting star exports"，s102 通道映射
  SyntaxError）+ transform_export 三处 `memoized_dep_expr` +
  `module_graph_postorder` 注册序（静态依赖提取复用 transform 同款
  helper，动态 import() 非链接依赖不入；未注册/解析失败跳过；不可
  达余量字典序追加保全量覆盖）；script_gen
  `script_module_eval_error_report`（JS 侧取哨兵原值 reportError +
  元素照常 fire load）+ shim reportError error 原值透传（全仓两调
  用方：s102 臂恒 Error 实例不受扰）；webview try/catch 哨兵暂存
  **原值对象**（execute Rust Err 只剩文本）+ post-check + s107 门
  控防 load 双发；runner `data-zw-unit` 投递单元标记查回尾片（弃
  currentScript——module 体恒 null 死代码；弃序号口径——JS 字符串
  内 `<script` 文本计数膨胀）。验证：单测 script-sandbox **384 全
  绿**（+3 钉：`test_star_export_ambiguity_s255` 三形 6/7/对照 +
  `test_star_inline_dep_evaluated_once_s255` + `test_prelude_
  registration_postorder_s255`）；engine **3714 过 + 5 失败 = HEAD
  既有对端账**（js_dom_bridge CSSOM 5 案，82153965c 提交消息在账，
  stash 法定谳非本轮回归）；webview **835 全绿**；fmt/clippy 四
  crate 零告警；solo 终态 instantiation-error **7P+1T**（6/7 F→P，
  8 号 Timeout 既有）+ error-type **3P**（3 F→P）；中间红态两轮记
  档（ierr-7 仅 7d/7e 颠倒→暴露哈希序→E 刀；error-type-3 仍
  log[1] null→暴露 shim 折 null→C2 刀）；全树 A/B（窗口独占——
  s254 教训执行）行级 7793P/440F/77T、键级 **8303→8307（LOST 5 /
  GAINED 9）迁移恰 3 键与目标全等**，三处页级完成度位移全定性非回
  归：focus 跑满（Timeout 线消失 +1P，在册抖动族向完成摆动）/
  select 回惯常 4F（s251 起基线态）/ evaluation-order-1 截断掩盖解
  除新可见「Module script queueing a microtask」Fail（solo 复现同
  断言，与页内 classic 变体长期在账同机制——microtask checkpoint
  排空时序，B 刀去重复求值页面变快截断后移）。挂账更新：**microtask
  checkpoint 排空时序族**（classic/module 两变体同修面，新记档）；
  承前不动：renderer/tab import.meta.resolve 接线、import maps 剩
  余语义面、data-zw-harness 保真面（D 刀后收窄为注入序）、s250 动
  态内联 module 臂 eval 通道、instantiation-error-8 Timeout。账本
  [evidence/2026-11-03-m2s255-star-export-ambiguity.md](html-semantics-compat/evidence/2026-11-03-m2s255-star-export-ambiguity.md)。
- **2026-11-03 M2 切片 254**：import.meta.resolve + 进程内 import map
  接线轮（s253 候选 #1）——勘根：基建完整（`import_map.rs`
  parse/resolve/resolve_mapped 12 单测 + `resolve_registered_specifier`
  map 臂）但 webview 进程内路径零消费面（`extract_import_map_json`
  无调用方；renderer/tab 宿主已接 js_worker parse_page_import_map）；
  四键语义（bare 映射/URL-like 重写/ToString/链式 map 再应用）+ 语料
  import-meta/resources/ 整目录缺失（s227 教训施工前先验）。施工
  **9 刀**：es_module prelude `__zwImportMapResolve` 宿主桥（返空 =
  resolution failure → TypeError——spec 较动态 import join 降级更严；
  未接线宿主 typeof 守卫同落 TypeError 非 ReferenceError）+
  build_module_script/build_dep_iife 恒发射 `_importMeta.resolve`
  （String(spec) + 本模块 url 穿入，无 map 即纯 URL 解析零行为差）；
  webview 首张合法 map 解析 + `__zw_importmap_resolve` 回调（resolve
  全算法 + 映射值 base join）+ 动态 import 两 resolver 闭包
  `apply_import_map` 改写（resolve_mapped 仅实际命中——未命中原样，
  既有 parent-join 零漂移；CSP nonce/fetch 用映射后 abs）+ 静态面
  registry.set_import_map + helper fn（Blocked 按 Miss 降级 FIXME 同
  静态面注记）。验证：新钉 `test_import_meta_resolve_s254` 四断言
  （红态实证撤三处发射/桥行 FAILED）；script-sandbox 381 + webview
  835 全绿；solo FILTER=import-meta **4 Pass / 0 Fail**（守卫终态复
  跑）；全树 A/B 行级 7789P/439F/78T、键级 **8306→8303 闭合
  （-5 LOST +2 GAINED）迁移恰 5 键 = 目标 4 键 F→P + checkable-
  active-onblur F→P（在册 checkbox `:active` 漂移族回摆，s253 轮
  P→F 本轮归位基线态）**；两处键形态位移（focus-dynamic-type-change
  1 子测试→文件级 Timeout / select-home-end-pagedown-pageup 4 子测试
  Fail→文件级 Fail）solo 复跑证实 **A/B 窗口并发编译负载**（红态编
  译同机并行 CPU 争抢压时序重页：focus 80/80 复得、select 回惯常
  4F 同形）——教训记档：**全树 A/B 运行窗口禁并发编译负载**。挂账
  更新：renderer/tab 宿主 import.meta.resolve 接线（map 已在
  scope，补回调即同 webview 形）；import maps 剩余语义面（多张 map
  只取首张合法/动态插入 update/revoke 未实现/Blocked 降级）。
  账本
  [evidence/2026-11-03-m2s254-import-meta-resolve.md](html-semantics-compat/evidence/2026-11-03-m2s254-import-meta-resolve.md)。

- **2026-11-03 M2 切片 253**：CSP eval 门禁互斥模块编译通道轮
  （s252 候选 #1 收窄：nonce ×1 施工，delay-load ×1 勘根定谳改挂
  账）——三探针实证（包装安装点/eval-blocked 回调/v8 原生
  resolver，logs/s253-probe.log）**三通道分流**：内联 module 转换体
  动态 import 走 shim 通道（prelude `__zw_load_module` 编译 eval 在
  包装窗内被自家 CSP 误拦，EvalError）；外链 module 页 Pass 实为
  runner 投递统一打 `data-zw-harness` → 包装不装「错进错出」（新挂
  账：CSP 页外链页脚本整体豁免 eval 门禁的保真面）；classic 页走 v8
  原生 import() 通道（Rust 侧编译，包装不可见）。spec 口径：module
  evaluation 非 eval()，script-src 管模块 fetch（nonce 宿主 resolver
  已查）不管求值机制。施工 prelude 一行窄豁免——
  `(typeof __zwRealEval === 'function' ? __zwRealEval : eval)` 感知
  形（仅模块编译调用豁免；模块体内嵌套 eval/页面 eval 照拦；未装包
  装回落同形零变化；急切注册臂 IIFE 直嵌不经 eval 波及面封闭）。
  验证：新钉 `test_dynamic_import_eval_gate_exemption_s253` 三层断
  言（红态实证旧行 `rejected:EvalError` 同形签名）；script-sandbox
  380 + webview 835 全绿；solo nonce 四页全 Pass；全树 A/B 行级
  7782P/447F/77T 双侧同数、键级 **8306/8306 对迁移恰 2 键 = 目标键
  propagate-nonce-inline-module F→P + checkable-active-onblur 1 子
  测试 P→F（同跑同文件 3F+4P 混态自证，在册 checkbox `:active` 双向
  漂移族先例销账记基线态）**。挂账更新：delay-load 根因定谳（宿主
  resolver 链脚本执行窗内同步阻塞取回——trickle(d2) 冻结回合 2 秒，
  load 派发与 1s timer 排断言之后，修复需动态 import 异步通道：契约
  pending 态 + 完成泵，独立切片）；s250 动态内联 module 臂 eval 通道
  同暴露面记同类待勘。账本
  [evidence/2026-11-03-m2s253-csp-module-eval-exemption.md](html-semantics-compat/evidence/2026-11-03-m2s253-csp-module-eval-exemption.md)。

- **2026-11-03 M2 切片 252**：module 链接/instantiation 错误语义轮
  （s87 注记预账收口）——全景分型 dynamic-import 族 24+ F 键 7 簇，
  本切 instantiation/link 错误语义 4 键一个根。勘根探针（临时
  eprintln registry 键 dump）定谳**registry 双键现实**：runner
  pipeline 外链模块体相对 import 重写为路径绝对形、内层 import 保
  raw 形，direct 项失败消息 `/html/...` 形 vs nested 项 `./` 形 →
  instantiation-error-1/2 消息身份断言（log[0] 自引用）红；动态面
  `Err(CompileError) => String::new()` → TypeError 双红。施工双
  刀：**Fix A abs-only 注册**（注册键 = 解析后 URL，跨拼写 URL-join
  归一；s101 环臂补注册删 + `fetched` 参数删 + contract 入口键
  abs + **静态入口转换基改 module_dep_base**——s101 钉先红后绿
  实证基错 join 落页目录假 Module not found）；**Fix B 契约 tag 3**
  （`3\x1fresolved\x1fmessage` 链接失败态，prelude/v8 原生回调两
  侧逐次新 SyntaxError 不入缓存，s87 ③「暂并入」翻案独立成态
  ④；s250 动态 src 臂 tag 3 落 ③ 形 eval 语义注释注记）。验证：
  单测 s87 四态身份钉 + v8 native tag-3 新钉（script-sandbox 379 +
  webview 835 全绿）；solo 988P/75F/25T/5N（ΔP=+4 ΔF=−4）键级
  join 4 F→P 全目标零回归；全树 A/B **8317/8317 对迁移恰 4 键
  全 F→P，LOST=GAINED=0——第十一个连续完美 A/B**；abs 键下
  raw-hit 不再遮蔽 import map（全树零位移证存量无语料依赖旧序）。
  新挂账 instantiation-error-6/7 星号导出歧义检测（log 序列形状
  失败签名，真机制缺口）。账本
  [evidence/2026-11-03-m2s252-module-link-error-semantics.md](html-semantics-compat/evidence/2026-11-03-m2s252-module-link-error-semantics.md)。

- **2026-11-03 M2 切片 251**：动态内联 module 延迟求值轮（s250 挂账
  收口 + 归因修正）——勘根推翻前提：最小探针（前导脚严格 `var` +
  动态插入内联 module 体静态 import + `onExecute(pv)`）三测全
  Pass，定谳 strict 页面 var 经 `script_run_classic_page` 间接
  eval + `scan_top_level_decl_exports` **globalThis accessor 导
  出**对 shim eval 全可见，s250「replay 作用域不可见」8 探针结论
  作废；真根因 = 同脚本形态竞态（`var` 声明与 `appendChild` 同
  `<script>` 内，s109 内联臂同步 eval 时插入脚本 accessor 导出
  后缀未执行 → 体引用 ReferenceError → onExecute 不达 →
  pending；spec 动态插入模块 evaluate 排插入脚本之后）。施工
  shim 1 处（s109 内联 module 臂编译+四态状态机+求值整体
  `setTimeout(0)`，src 路径 rev3 同型；体文本读/空体门/run-once
  记账保持插入时刻同步——already-started 属 prepare 同步段；404
  臂延迟事件派发嵌套保持零时序漂移；classic 内联臂同步 eval 不
  动——124 形期望序依赖）。验证：solo 984P/79F/25T/5N（对 s250
  ΔP=+5 ΔN=−5）+ 全树 A/B 键级 **5 N→P 零回归，LOST=GAINED=5
  同键换形，F/T 零位移——第十个连续完美 A/B**（5 键 = 目标 3 臂
  text/json/css-module `inline, 200, not parser-inserted` +
  module/load-error-events-inline.html 2 动态内联臂名为 src 实
  内联同根同刀）；load-error-events 全家族 5 页 48 臂首次全绿；
  拼接 shim node --check 干净、Rust 零改动。挂账更新——s250
  「host 侧执行通道/沙盒重入 API」候选销案；prelude
  `__zw_load_module` 同型潜伏面假说弱化（动态 import 体经
  promise 异步求值，同型竞态不适用，归候选 #1 一并核）。账本
  [evidence/2026-11-03-m2s251-dynamic-inline-module-deferred-eval.md](html-semantics-compat/evidence/2026-11-03-m2s251-dynamic-inline-module-deferred-eval.md)。

- **2026-11-03 M2 切片 250**：text-module integrity matches 臂 +
  load-error-events 四页 src 臂族 + s100 拦截面 module 型豁免 +
  动态 src/内联 module prepare 轮——勘根四根：text 型 spec 无
  MIME essence 校验（fetch-a-single-module-script step 7.2 无条件
  创建 text module script；s249 fetcher 误校验，一行修正 + 无扩
  展名语料 `text-module/file` 10 字节补入）；load-error-events.py
  端点缺失（上游 wptserve CGI 不在语料 pull 面，runner fetcher
  四风味单臂建模：text/json/css 数据模块体镜像 + 素模块 `_load`
  回体/`_error` 直接 404，`test` 参数 `^[a-zA-Z0-9_]+$` 校验同上
  游）；s100 拦截面 module 型误伤（`inline_local_scripts` 对
  the-script-element/resources/load-error-events.py 无条件内联投
  递 → module 单元 currentScript 恒 null 投递尾静默空转 →
  module/ 变体 4 静态臂 pending 文件 Timeout；probe 三态实验定谳，
  `!is_module_type` 豁免修复零波及经典族）；动态 src module 执行
  通道三版收敛 rev1 裸 eval 丢 referrer/base 槽位（4 P→F）→
  rev2 同步时序翻 load-after-execution 交错（5 臂 F）→ rev3 队位
  调用时刻 + 延迟编译 + `_zwExecuteFetchedScript`/in-order 队列
  复用（基/队列/时序三面复位 + text-module src,200 动态臂随上下
  文翻绿）；动态内联 module prepare（`__zw_compile_inline_module`
  四态契约 + s109 内联分派臂）404 臂翻绿、200 臂挂账（严格页面
  var replay 作用域对 shim eval 不可见，host 侧执行通道待沙盒重
  入 API 设计）。施工 shim 3 处 + webview 1 回调 + runner 3 处 +
  单测 2 + scripts 注记。验证：全树 A/B 12 键翻绿（7 F→P + 5 N→P）
  P→F 0（8313 对，LOST/GAINED 16+13 全归因，第九个连续完美
  A/B）+ solo 979P/79F/25T/10N（对 s249 postpull ΔP=+16 ΔF=−7）
  + runner 281 绿 + webview 835 绿 + fmt/clippy 干净。账本
  [evidence/2026-11-03-m2s250-load-error-events-module-family.md](html-semantics-compat/evidence/2026-11-03-m2s250-load-error-events-module-family.md)。

- **2026-11-03 M2 切片 249**：动态 import 属性三通道（json/css/text）轮——勘根
  定谳动态属性双通道丢弃单点（prelude `__zw_dynamic_import` 单参 + V8 原生回调
  import_attributes `#[allow(unused_variables)]` 弃用）+ fetcher 契约无类型形参
  （`__zw_mtype` 内部标记穿参，s124 同法）；施工 7 文件（es_module TC39 import
  calls 解析三 TypeError 臂 + v8_runtime FixedArray 提取/校验 + webview json/css/text
  三合成臂 + JSON parse 失败改判 SyntaxError + runner intent 臂 MIME 派生序/redirect
  跟随/type↔MIME 校验 + part06 CSSOM 颜色归一 148 命名色 + `_zwSerializeDecls` +
  redirect-target.css 语料补）+ 单测 ×4。验证：全树 A/B 13 F→P + 1 T→P + 2 T→F 预期
  换形 + P→F 0（8314/8314 全键，第八个连续完美 A/B）+ post-pull solo 家族切片
  1089 对零位移 + runner 279 绿 + fmt/clippy 干净。账本
  [evidence/2026-11-03-m2s249-dynamic-import-attributes-css-text-json.md](html-semantics-compat/evidence/2026-11-03-m2s249-dynamic-import-attributes-css-text-json.md)。

- **2026-11-03 M2 切片 248**：redirect.py 真 302 端点 + host wire
  data:/重定向臂 + finalUrl 污染通道轮——runner `common/redirect.py`
  200 伪重定向升级真 302（消费面封闭核：R360 消费方单测 mock 不经
  本臂）；webview `__zw_get_image_wire` data: 臂（decode_data_uri_bytes
  复用）+ 抓取 3xx 跟随 ≤4 跳 + `encode_image_wire` 共享；shim settle
  记 resp.url 入 `_zwAvailableImages[src].finalUrl` + 源判 finalUrl
  优先。security 全族 24 文件收口（11 `.redirect` + dataURI 翻绿）+
  toDataURL 往返 4 面 bonus 同根兑现。验证：solo security 24P/1T +
  邻接 solo 零位移 + 全树 A/B 16 全 F→P + 1 P→F 在册 flaky 销账
  LOST 0/GAINED 0（第七个连续完美 A/B）+ fmt/clippy 双 crate 干净。
  账本
  [evidence/2026-11-03-m2s248-redirect-endpoint-data-uri-wire.md](html-semantics-compat/evidence/2026-11-03-m2s248-redirect-endpoint-data-uri-wire.md)。
- **2026-11-03 M2 切片 247**：canvas origin-clean 面三根勘根轮——
  data: script 内联（runner `decode_data_url_script` RFC 2397 臂 +
  单测 ×2）+ no-cors img 取回通道（part01 `_zwImgNoCorsChannelDepth`
  s153 同型 + `_zwFetchApplyFilteredResponse` 第 7 参门控 +
  crossorigin 属性在场不豁免）+ 污染核心（part05
  `_zwCanvasSrcCrossOrigin` + ctx `_zwUnclean` 5 重置点归零 + setter
  set 时污染 + drawImage/createPattern/createImageBitmap 捕获 + 三读
  回 SecurityError；part04 DOM 代理同臂），security `.cross` 11 键
  F→P 翻绿。验证：solo 三轮（首轮签名换形暴露第二根 `failed to
  load`→二轮 `.cross` 全绿→crossorigin 修正后精确维持）+ 全树
  A/B 12 全 F→P 恰靶 LOST 0/GAINED 0（第六个连续完美 A/B）+
  node --check + fmt/clippy 干净 + r5388 单测 2 passed。账本
  [evidence/2026-11-03-m2s247-canvas-origin-clean.md](html-semantics-compat/evidence/2026-11-03-m2s247-canvas-origin-clean.md)。
- **2026-11-01 M2 切片 211**：button 激活门双缺口轮——command/
  commandfor 缺省 type + fieldset 传染提交门，button-click-submits
  两文件 4F 收口。勘根定谳缺省 type 缺口（invoker commands 集成后
  Submit Button 缺省要求两属性皆缺）+ 激活门 disabled 只查自身；勘
  根深挖实锤 lw 通道坑（`__zw_has_attr` 纯快照对 setAttribute 写恒
  miss，探针对比 lw='1'）；施工 part03 共享帮手 ×2（lw 融合）+
  part04 两门换帮手 + `_validityState` 走查共享化 + part09.rs R5378
  六腿常驻承载。验证：靶面 16/16 Pass + forms 全域 A/B 净 F→P=4 恰
  4 靶键 P→F=0 键漂移 0 + zero-engine 3671P+0F + fmt 零 diff +
  guarded-clippy 过。账本
  [evidence/2026-11-01-m2s211-button-effective-type-fieldset-submit-gate.md](html-semantics-compat/evidence/2026-11-01-m2s211-button-effective-type-fieldset-submit-gate.md)。
- **2026-11-01 M2 切片 210**：FORM 级约束校验双缺口轮——fieldset
  disabled 传染 + invalid 全遍历派发，form-validation-validate 3F 收
  口。勘根定谳双缺口（`_validityState` 无 fieldset 祖先走查——s193
  willValidate 臂读面基建已有同口径仅 validity 未接；FORM 臂短路
  return 吃掉后续控件 invalid 派发——times 实测 4→2 实锤）；施工
  part03 走查接入（首 legend 豁免 + R57 门不动）+ part04 全遍历旗标
  + part09.rs R5328 六腿常驻承载。验证：靶面 8/8 Pass + forms 全域
  A/B 净 F→P=3 恰 3 靶键 + onblur 族 s209 摆动闭环（2 回摆+1 回摆）
  键漂移 0 + zero-engine 3670P+0F + fmt 零 diff + guarded-clippy 过；
  misc 残面盘点新浮现 button-click-submits-with-commandfor 3F +
  button-click-submits 1F 次刀候选。账本
  [evidence/2026-11-01-m2s210-form-checkvalidity-fieldset-inherit-invalid-fanout.md](html-semantics-compat/evidence/2026-11-01-m2s210-form-checkvalidity-fieldset-inherit-invalid-fanout.md)。
- **2026-11-01 M2 切片 209**：number 编辑器双槽轮——键入 veto + caret
  臂 + badInput 接线，三语料 11 键收口。勘根定谳键入面无 veto 无
  caret（科学计数测 1-3 意外过经 parseFloat 前缀解析，Insert - at
  start 尾拼 1F；constraint/clobbering 键入中间态停原文无暴露换算）；
  施工 part01 `_numberEditor` 双槽（文法 veto + 暴露换算 + 删除编辑器
  空间化 + `__zw_text_caret_move` 新桥）+ part03 badInput 臂 + part04
  三 setter 非 skip 写清槽 + testharness.rs caret 消费门 + part09.rs
  R5327 八腿常驻承载。验证：三定向 11/11 Pass + forms 全域 A/B 净
  F→P=4 恰 4 靶键 + P→F=2 全落 onblur flake 族（stash 基线重跑同败
  已证）键漂移 0 + zero-engine 3669P+0F + fmt 零 diff + guarded-clippy
  过。账本
  [evidence/2026-11-01-m2s209-number-editor-buffer.md](html-semantics-compat/evidence/2026-11-01-m2s209-number-editor-buffer.md)。
- **2026-11-01 M2 切片 208**：:valid/:invalid 伪类候选门轮——
  output-validity 1F 收口。勘根定谳双通道形态（Rust 样式路径
  validation.rs 候选门已正确；失败走 shim matches 通道 `:valid` 臂无
  候选门直返 `_validityState().valid`，output 落通用汇总被 :valid 吸
  走）；施工 part04 matches 臂候选门（非 INPUT/SELECT/TEXTAREA/FORM 返
  false，FORM 聚合臂保留）+ part09.rs R5326 桥测试八断言常驻承载。
  验证：靶面 1/1 Pass + forms 全树 A/B 净 F→P=2 恰 1 靶键 + onblur 域
  1 键上摆（在册 flake）P→F=0 键漂移 0 + zero-engine 3667P+1F（在册
  键）+ fmt 零 diff + guarded-clippy 过；form owner table 勘定升格 4F
  解析器基建面退回候选池。账本
  [evidence/2026-11-01-m2s208-output-validity-pseudo-candidate-gate.md](html-semantics-compat/evidence/2026-11-01-m2s208-output-validity-pseudo-candidate-gate.md)。
- **2026-11-01 M2 切片 207**：form.action 文档 URL 反射特例轮——
  form-action-reflection-with-base-url 2F 收口。勘根定谳 R5009 片 d
  特例被 net-api M5-S7 解析优先分支吸走（空串对基 URL 解析=基 URL，
  无 `<base>` 掩面）；施工 part03 action getter 空值先判返文档 URL +
  part09.rs R5325 桥测试常驻承载。验证：靶面 3/3 Pass + forms 全树
  A/B 净 F→P=2 恰靶键 + onblur 域 ±2 双向（在册 flake）+ zero-engine
  3666P+1F + guarded-clippy 过；formAction `<base>` 同形缺口新记挂账。
  账本
  [evidence/2026-11-01-m2s207-form-action-document-url-reflection.md](html-semantics-compat/evidence/2026-11-01-m2s207-form-action-document-url-reflection.md)。
- **2026-11-01 M2 切片 206**：output value/defaultValue 活读模型轮——
  mutations.window 2F 收口（Descendant mutations + form.reset()）。
  勘根 spec §4.10.12 三缺定谳（getter 缓存盲 / value= 捕获 host 直读 /
  reset 未清 override）；施工 shim 2 文件（getter 活读臂 ×2、value setter
  捕获改融合视图、innerHTML/textContent= 删镜像、reset OUTPUT 单列臂）+
  part09.rs R5324 桥测试两腿常驻承载。验证：靶面 2/2 Pass + forms 全树
  A/B 净 F→P=2 恰靶键 P→F=0 键漂移 0 + zero-engine 3665P+1F（在册键）
  + guarded-clippy 过；output-validity.html 1F 定谳 pre-existing
  （:valid/:invalid 伪类面）挂账。账本
  [evidence/2026-11-01-m2s206-output-value-defaultvalue-live-read.md](html-semantics-compat/evidence/2026-11-01-m2s206-output-value-defaultvalue-live-read.md)。
- **2026-10-31 M2 切片 175**：detached 节点选择器身份盗用根修轮——枚举
  镜像钩首刀 P2F=16 勘根两刀（管线等值轻量臂实证换代理论证伪 → mutation
  日志暴露 showModal/close 打在镜像句柄上）定谳：detached 镜像
  `unique_selector_for_node` 盗用在树同 `tag.class` 真元素唯一选择器
  （uniqueness 门 doc 查询、真元素顶替）→ sel↔handle 表顶替 → normal 段
  全打镜像。根修 = 引擎可达门（parent 链不达 root 返 None）+ 钉测 +
  枚举钩撤下（hooks-on 实证注册恢复零 Pass 增益，s176 引擎侧 shadow
  寻址承接）。engine lib 3617P/10F（+1 钉测）/webview lib 830P/0F/
  interactive-elements 437P/26F P2F=0 与 s173 终态逐键同态/forms
  96P/12F 持平。账本
  [2026-10-31-m2s175-detached-selector-identity.md](html-semantics-compat/evidence/2026-10-31-m2s175-detached-selector-identity.md)。
- **2026-10-28 M2 切片 124**：module 网络面收口轮（link rel=modulepreload
  取回面）——勘根四项定谳：modulepreload referrerpolicy 族三面缺口
  （echo-referrer.py 端点 / shim 取回面 / 元素级 referrerpolicy 通道）+
  cross-origin 页实缺 script.py 端点（非 https 传输——www1:8444 别名已
  剥）；module-import-referrer TLA 根因精确化（依赖模块 TLA 不可用，
  build_dep_iife 同步 IIFE——深转换架构面保留）；dynamic import() 透传
  = css-module 复合保留；classic 外链 SRI 无语料需求。施工五件：shim
  `_zwModulePreloadMaybeStart`（同步内联取回——mp5 探针实证 setTimeout
  延后臂不轮转、import() 先行覆写 referrers）+ `_mo_notify` LINK 触发
  臂（顶层 + 下降）+ classic 动态 import() module map 记忆化（恰好一次
  求值 + 失败不入缓存）+ runner 端点仿真 ×2 + pin ×2。域家族 **6 行
  （4F+2T）→ 15/15 全 Pass**；全量 A/B +15P/−4F/−2T 全落家族真翻红 0
  （6653P/1145F/105T → 6668P/1141F/103T，行数 +9 = 挂起页未完成注册
  子测试现完成打印，口径已记账）；sandbox lib 377/0 + engine lib
  3570/0 + runner 256/0。账本
  [evidence/2026-10-28-m2s124-modulepreload-face.md](html-semantics-compat/evidence/2026-10-28-m2s124-modulepreload-face.md)。
- **2026-10-25 M2 切片 86**：scripting-1 开片分诊轮——s85 下一步
  「testdriver-Actions 支持面」过时（s79 已证伪 s78 注记、s80 journal
  已收口 hash-name-dynamic 8P/0F，FILTER 复跑 8/8 复证）；当拍红图
  **169P/335F/86T**（语料 +195 后），失败族四切片序：①dynamic-import
  must-reject → ②stylesheets-blocking-scripts 12F 簇 → ③the-script-
  element 基面 65F → ④module 尾片。一刀（R5286，runner）：moving-between-
  documents 53 案 helper 运行期 iframe 开第二文档——单文档 runner
  不可执行面归 navigation M3 门控，`html_semantics_case_skipped` 增
  helper 标记臂 + 契约单测。域 **169P/335F/86T → 117P/335F/34T**
  （−52P/−52T 全落 helper 案，F 逐位不变；墙钟 47→24min）。账本
  [evidence/2026-10-25-m2s86-scripting1-triage.md](html-semantics-compat/evidence/2026-10-25-m2s86-scripting1-triage.md)。
- **2026-10-24 M2 切片 69**：F3 img.decode() 收口（R5276 三刀）——根因
  decode IDL 无实现（get trap 无臂 + SVGImageElement 原型无方法）。
  ①共享解码核 `__zwImgDecodeCore`（IMG trap/detached 动态臂/SVG 原型三方
  复用，调用时捕获候选 + 恒异步 settle + 双点变异与活跃性复检 + 自驱
  fetch 链）；②part03 detached 产物动态臂 + SVGImageElement.decode；
  ③part04 trap 臂瘦身为闭包供给。契约 part86 两测试。域
  **549P/738F/35T → 607P/678F/37T**（GAIN 58；残留 2F=quick-attach 时序
  归 F2、2 案 F→T 预存环境抖动）；lib **3477P/0F**（+2 契约）。账本
  [evidence/2026-10-24-m2s69-img-decode.md](html-semantics-compat/evidence/2026-10-24-m2s69-img-decode.md)。
- **2026-10-24 M2 切片 68**：F1 img 静态加载管线收口（R5275 六刀）——
  根因 img settle 只接 `.src` IDL 通道（parser 静态 img 无 fetch/settle）
  + template 内容漏抓 + broken 终态缺失。part04 sweep/getter 面 + part05
  R56h restart 语义 + extract.rs template 递归 + testharness.rs 生命周期
  拆三任务链 + available-images `_defer` 化。域 **461P/816F/40T →
  549P/738F/35T**（GAIN 88）；naturalWidth 30P/228F→48P/210F + delay-load
  7 案全绿；interactive **439P/44F/0T 持平**；lib **3474P/0F**（+2 契约
  part85）。账本
  [evidence/2026-10-24-m2s68-img-static-loading.md](html-semantics-compat/evidence/2026-10-24-m2s68-img-static-loading.md)。
- **2026-10-24 M2 切片 67**：embedded-content 开片分诊轮——域刷新
  458P/819F/40T 八族地图（F1 img 静态加载管线 ~260F / F2
  select-an-image-source ~300F / F3 decode 56F / F4 image-map 命中 84F
  渲染流回流 / F5 lazy ~49F / F6 canvas 53F / F7 object|embed 窄面 25F /
  F8 杂项 2F）+ F7 null 反射收口（R5274 一刀，part04 get-trap
  OBJECT/EMBED 分支——contentDocument/contentWindow/getSVGDocument 恒
  null 面）+ 契约 part84 两钉 + 翻转力实证。域 **461P/816F/40T**（GAIN 3
  / REG 0）；interactive 439P/44F/0T 持平；lib **3469P/0F**（+2）。账本
  [evidence/2026-10-24-m2s67-embedded-content-triage.md](html-semantics-compat/evidence/2026-10-24-m2s67-embedded-content-triage.md)。
- **2026-10-23 M2 切片 46**：余 60F 归因分诊六族地图落档（A–G，含
  Chrome-parity 同红族 A/B 与可 shim 收口候选 E）+ 历史 menu 面 IDL 归属
  收口（R5249 双刀，shim part04/part05）——has trap track IDL 列
  kind/srclang/label/default tag 门（TRACK 与 TRACK+OPTION）+ R3069 写臂
  MENU/MENUITEM 负门（FLAT type 与显式 label 列无 tag 门致 menu.type=/
  menu.label= 误写内容属性）。契约 part78 两钉（menu 排除五臂 + track/
  option/FLAT 面四臂不回归）+ 翻转力实证。contextmenu-historical
  **11/11 Pass**；interactive-elements 全域 98 files **423P/57F/2T 对 s45
  逐位 REG 0 / GAIN 3**；lib **3442P/0F**（+2 契约）；零新增依赖。账本
  [evidence/2026-10-23-m2s46-historical-menu-idl-gate.md](html-semantics-compat/evidence/2026-10-23-m2s46-historical-menu-idl-gate.md)。
- **2026-10-23 M2 切片 45**：s37 已知边界「点击 summary-less details 内
  非激活后代不翻转」收口（R5248 双刀，shim part03/part04）——part04
  click 默认动作段 nearest activation 上溯分支（`.click()` 合成半边，
  receiver 三站让位 + 交互后代 type 豁免 + notPrevented 门继承）+
  part03 R112 post act:details 门放宽至 JS dispatch 通道
  （`_zwSyntheticClick` 标互斥防双翻，preventDefault/connected 门由入口
  守卫继承）。契约 part77 两钉（翻转/互斥六臂 + 让位/豁免三臂）+ 翻转力
  实证。interactive-elements 全域 98 files **420P/60F/2T 对 s44 逐位
  REG 0 / GAIN 0**（corpus 无观测面）；lib **3440P/0F**（+2 契约）；零
  新增依赖。账本
  [evidence/2026-10-23-m2s45-summaryless-details-descendant-activation.md](html-semantics-compat/evidence/2026-10-23-m2s45-summaryless-details-descendant-activation.md)。
- **2026-10-23 M2 切片 44**：s39 剩余 1F focus-after-close Complex 案收口
  （R5247，shim part01/part04/part06 三处）——跨代 handle proxy 孤儿
  isConnected 假阴性修正（宿主持久反查 + querySelector 同一性核验前置
  复核）+ activeElement 焦点链 retarget light 锚误穿收口（light 锚直返
  门，判据与 stale `_shadowHandles` 投影注册形态解耦）。契约 part76 两钉
  （孤儿 isConnected / light-shadow retarget 双向）+ 翻转力实证。
  focus-after-close 单案 **6/6 Pass**；interactive-elements 全域 98 files
  **420P/60F/2T 对 s43 逐位 REG 0 / GAIN 1**；lib **3437P/0F**（+2 契约）；
  零新增依赖。账本
  [evidence/2026-10-23-m2s44-orphan-isconnected-retarget.md](html-semantics-compat/evidence/2026-10-23-m2s44-orphan-isconnected-retarget.md)。
- **2026-10-23 M2 切片 43**：replaceWith self-removal 父桶记账收口（R5246
  一刀，s26 既有挂账「R371 rekey 时 dst 桶不存在，不记父桶 removed」）——
  sel 臂 remove 步补 `_zwHCLiveInvalidate` 直调（父桶 removed identity +
  全局 pending-removed + live 失效一体），合并视图残留终结、视图即 spec
  终态；记账先于 rekey，promote 分支留护栏。契约 part75
  test_dialog_replacewith_parent_bucket_accounting_r5246（R1/R2）+ part24
  r321 kids 断言更新（宿主残留 DIV 消失）。interactive-elements 全域
  98 files **419P/61F/2T 对 s42 逐位 REG 0 / GAIN 0**；lib **3434P/0F**
  （+1 契约）；零新增依赖。账本
  [evidence/2026-10-23-m2s43-replacewith-parent-bucket.md](html-semantics-compat/evidence/2026-10-23-m2s43-replacewith-parent-bucket.md)。
- **2026-10-23 M2 切片 42**：非模态 close 不出组栈收口（R5245 一刀，
  s23 已知保守项）——close-the-dialog 组栈出栈移出模态臂，modeless
  show/close 循环 stale 项无限增长终结；契约 part74
  test_dialog_group_stack_hygiene_r5245（R1–R4 行为不变量：循环/栈序/
  全关 no-op/模态臂保持）。interactive-elements 全域 98 files
  **419P/61F/2T 对 s41 逐位 REG 0 / GAIN 0**；lib **3433P/0F**（+1
  契约）；零新增依赖。账本
  [evidence/2026-10-23-m2s42-nonmodal-close-stack.md](html-semantics-compat/evidence/2026-10-23-m2s42-nonmodal-close-stack.md)。
- **2026-10-23 M2 切片 41**：挂账定性轮——「ESC 组栈边缘乱序」收口为
  Chrome-parity 同红（corner-cases「focusin calls showModal, modeless」
  ESC 案：crbug 417571176、父 376516550，2026-04 wpt.fyi 仍红）。探针
  钉定事件序 + spec 逐字对账（R4600 恢复门/R5236 组栈零漂移）；R5244
  假设门（subject 非 focusable no-op）GAIN 1 / REG 4（dialog-focus-shadow
  Chrome「show() 聚焦 dialog 自身」面）试错回退，零代码净变化。全域
  419P/61F/2T 对 s40 逐位 REG 0、lib 3432P/0F、零新增依赖。账本
  [evidence/2026-10-23-m2s41-esc-stacked-parity.md](html-semantics-compat/evidence/2026-10-23-m2s41-esc-stacked-parity.md)。
- **2026-10-23 M2 切片 40**：custom command（`--` 前缀）CommandEvent
  派发收口（R5243，s24 defer）——part05 CommandEvent 接口
  （`_defineEventSubclass`，command=''、source=null，无 relatedTarget）
  + part01 `_zwInvokerCommandActivate` custom 分支派发后 return
  （command=原值大小写保留、source=invoker、cancelable=true、
  bubbles=false/composed=true、R312 trusted 印记；分派在调用方
  disabled/preventDefault 门内零新写）。interactive-elements 全域
  98 files **419P/61F/2T 对 s39 打平**；dialog 集群对 s35 **REG 0 /
  GAIN 5**（s39 既有收益）；forms/the-button-element 对 s24 逐位一致。
  gates：engine lib **3432P/0F**（+1 契约 part73）、clippy/fmt 过、
  零新增依赖。账本
  [evidence/2026-10-23-m2s40-custom-command-event.md](html-semantics-compat/evidence/2026-10-23-m2s40-custom-command-event.md)。
  挂账：现行 spec invalid→Unknown 面（跟进涉内建关键字重构）、
  popover 关键字、oncommand IDL、command/commandFor IDL 反射、
  ESC 组栈边缘乱序、非模态 close 不出组栈、R371 模板残留、
  summary-less details 非激活后代翻转、focus-after-close Complex
  dialog usage 深层。
- **2026-10-23 M2 切片 27**：corner-cases 毒化链根因收口（R4620 批内
  selector 时序）——根因在宿主共享 apply 路径：drain 顺序重放时前序
  Remove 摘除移动语义节点 → 后续 INS ref 的 `:nth-child` 索引漂移 →
  target 失配 warn+skip → dialog 永久 detached + 同批 SetAttr 连锁失配
  （JS 侧 op 引用批开始时树态，spec insertBefore 的 ref 不因前序 remove
  失效）。修复＝`apply_dom_mutations_full` 批前预解析 insert-adjacent
  族四变体 target 为 NodeId + 四臂失配查表兜底；契约
  `test_apply_ins_target_pre_remove_index_drift_r4620`（前红后绿）。
  dialog 集群 82 files 272P→**302P**（−33F），同 key 回归 **0**、增益 27
  （corner-cases 1P/29F→**28P/2F**，余 2F＝re-entrant showModal 边缘）；
  3T 全为 s26 基线既有；九域全量留待下轮刷新。gates：engine lib
  3416P/0F、clippy/fmt 过、零新增依赖。账本
  [evidence/2026-10-23-m2s27-r4620.md](html-semantics-compat/evidence/2026-10-23-m2s27-r4620.md)。
  挂账：corner-cases 余 2F（re-entrant showModal）、inert 计算态 13F、
  abspos/layout ~10F（rendering 域）、popover 有效性语义、impl-doc
  wrapper requestClose、anchor-with-inline 导航缺口、parser form element
  pointer 4F、custom command `--` 前缀、ESC 组栈边缘乱序、非模态 close
  不出组栈、R371 模板残留。
- **2026-10-22/23 M2 切片 26**：dialog focusing steps 家族（R4600）+
  rollover 挂死修复（R4600c/d）——show/showModal 跑 spec dialog
  focusing steps（autofocus 委派 → focusable area 递归判定 → focus 派
  focusin）+ close-the-dialog 焦点恢复 + activeElement 焦点链
  retargeting + R4600b replaceWith 空白锚点（focus-shadow 50F→0）；
  rollover 会话修 3 文件确定性整文件挂死：part06 load 分支补 spec
  flushing-autofocus-candidates + part01 focusable area 补
  pending-removed 门。九域 5070→**5116P**（68.32%），Timeout 173 持平，
  R4600c/d 隔离对照 +3P/0 回归；同 key 回归 27 全落 corner-cases
  毒化链（R4600 激活 focusin → 毒化共享 dialog，新暴露非新引入）。
  make test 76 腿全绿、clippy/fmt 不涉（纯 .js）、零新增依赖。账本
  [evidence/2026-10-22-m2s26-r4600.md](html-semantics-compat/evidence/2026-10-22-m2s26-r4600.md)。
  挂账：corner-cases 毒化链三层缝隙（下一切片首选）、inert 计算态
  13F、abspos/layout ~10F（rendering 域）、popover 有效性语义、
  impl-doc wrapper requestClose、anchor-with-inline 导航缺口、parser
  form element pointer 4F、custom command `--` 前缀、ESC 组栈边缘
  乱序、非模态 close 不出组栈、R371 模板残留。
- **2026-10-22 M2 切片 25**：corner-cases hang 排查（R4500）——横切根因
  收口：shim `close()` void 违约（透传 wasOpen boolean）× 语料
  testharness cleanup abort 语义（返非 undefined / 抛错均
  `tests.abort()`）→ file Timeout 吞全文件；s23「detach 窗口身份链」
  归因作废。修复 part04 三处 close 包装 + part14 两处旧断言更正 +
  part60 void 族契约 `test_dialog_void_return_r4500`（R1-R3）。九域
  5016→5070P（同 key 回归 0、翻转 0），Timeout 180→173（closedby
  1P→27/27、corner-cases 1P→28P/2F 等 7 文件解除吞没），+55F 为新上报
  真断言非回归。余 popover-closedby-simple 1 hang（cleanup hidePopover
  抛 not-showing → abort 同族，popover 域挂账）。gates：engine lib
  3394P/0F、webview 785→786P/0F、page-runtime 136P/0F、integration lib
  787P/0F、clippy/fmt 过、零新增依赖；中途 stash→rebase→pop 并入远端
  共享面提交（floats + workers 呈现管线，同动 part04.js 无冲突）。证据
  见 [evidence/2026-10-22-m2s25-r4500.md](html-semantics-compat/evidence/2026-10-22-m2s25-r4500.md)。
- **2026-10-22 M2 切片 24**：ToggleEvent.source / commandfor 簇
  （R4400）——toggle-source 三文件 3/10→10/10，九域 5008→5016P（+8 回归
  0）。含横切修复：user_actions.rs plan 通道私有批回灌共享队列全史 +
  prevented request-close 丢弃 invoker source。证据见
  [evidence/2026-10-22-m2s24-r4400.md](html-semantics-compat/evidence/2026-10-22-m2s24-r4400.md)。
- **2026-10-22 M2 切片 23**：dialog closedby 簇（R4300，rollover 接续
  ——上会话 429 中断遗留实现+契约 WIP，本会话核验收口）——两段推进：
  ①runner 通道：inline_local_scripts 根路径按 wpt_root 内联（域外助资
  产 `/common/top-layer.js`、`/close-watcher/resources/helpers.js` 等
  可达，harness 本体先前已被 replace_script_source 消费无双注入）+
  send_keys uE00C Escape 臂（`__zw_dispatch_key_at_active` 焦点元素三
  形态派发：`__zwSelector`/`_proxyCache` 反查/wrapper 直派；未取消 →
  `__zw_esc_dialog_cancel` 默认动作）+ click 命令 light dismiss 前步 +
  goal 脚本 5 助资产 fetch；②产品语义：closedBy IDL（小写化 + auto 派
  生 + 反射）+ `_zwDialogClosedByState` + close watcher 组栈
  `_zwDialogModalStack`（show/showModal 压组、首个 open 组即停、组内
  none 挡下探、markup/IDL open 惰性补登记、stale 顺栈清出）+
  `__zw_esc_dialog_cancel` 重写（closedby 资格门 + 组栈序 +
  `_zwDialogRequestClose`）+ `__zw_dialog_light_dismiss_check`（任何
  open+any dialog 包含链外落点 → close request；`__zw_pointer_down_
  sequence` 头部钩与 runner click 共享）+ `__zw_reset_form_state` 清
  栈。part60 R1–R5 契约。九域 4980→5008P（68.02%），对 s22 逐 subtest
  diff 净 +28P、回归 1：interactive-elements 202→227P（canceling 族假
  Fail 转真断言：ESC 主断言/组栈 none 挡板 4P/light dismiss 9P/
  cancel-events 族新上报 4P/closedby 逐案 7P/non-modal-canceling 1P）
  + scripting-1 +3（SVG script scheduler 意外翻绿稳定，机制未定位观
  察账），余七域 0 变化；回归 1（bounds-clicking (1,1)：dialog UA
  abspos 布局/hit test 缺口，归 rendering 域）。zero-engine --lib
  3386P/0F、page-runtime 136P/0F、webview 785P/0F、integration
  html_compat 21P/0F、clippy（test-guard 20G 包裹）/fmt 过、零新增依
  赖。挂账：corner-cases ~28 案 file Timeout（focusin remove+reinsert
  → showModal → ESC close 不达，detach 窗口身份链）、ToggleEvent
  .source/commandfor 8F 真断言（下一切片）、ESC 组栈惰性补登记边缘乱
  序（保守）、非模态 close 不出栈（读侧 purge 兜）、bounds-clicking
  (1,1)（rendering 域）、余 s22 挂账维持。账本
  [evidence/2026-10-22-m2s23-r4300.md](html-semantics-compat/evidence/2026-10-22-m2s23-r4300.md)。
- **2026-10-22 M2 切片 22**：dialog 元素簇（R4200，跨午夜会话）——两段
  推进：①part01 dialog 语义主体重构：toggle 事件任务队列（R4009 同款
  per-element 去重合并、preOpen 调用方写前捕获）+ close 事件排队异步
  （spec queue an element task，trusted 通道，旧同步派发 onclose 早于后
  续脚本）+ show/showModal 请求态语义（modal 上 show 抛/非模态上
  showModal 抛，旧静默切态）+ beforetoggle 同步可取消 + 写后重查 +
  close-the-dialog 核心共用 + `_zwDialogRequestClose`（cancel 可取消 +
  重入 busy 守卫，part04 DIALOG tag 门分发）；②共享基建：part03
  Promise.withResolvers polyfill + `_realTag` sel 印章（detach 窗口）、
  part05 `_insertAdjacentVariadic` sel 项（prepend(parsedEl) 丢失）、
  part04 matches 裸属性 lw + addEventListener `{signal}` per-record +
  set trap open moAttr 尾钩、part01 show/showModal/close 调用点 MO
  record（aria helper 共用防双计）。part60 T1–T8 新契约 +
  part14 ④⑧/part59 W3 契约同步（close 排队断言）。九域 4887→4980P
  （67.54%），对 s21 逐 subtest diff 净 +93P、回归 0，全落
  interactive-elements（109→202P）：requestclose 0→75P、toggle-events
  0→12P、close/close-event-async/requestclose-2/-recurse/-closedby/
  submit-dialog-close-event 各 +1。zero-engine --lib 3378P/0F、
  page-runtime 136P/0F、webview 785P/0F、integration html_compat 21P/0F
  （包名 zero-integration-tests 口径补正）、clippy/fmt 过、零新增依赖。
  挂账：closedby 簇 ~85F runner testharness helper 缺口（下一切片）、
  inert 计算态 ~15F、focusing steps ~15F（wrong global realm 面）、
  abspos/layout ~10F（rendering 域）、impl-doc wrapper requestClose、
  MO 内部写仅覆盖 dialog open（保守面）、`_zwSelSeenTag` 幽灵 sel 误
  服务（保守已知）、其余 s21 挂账维持。
- **2026-10-21 M2 切片 21**：details name= 互斥簇（R4100）——spec
  exclusive accordions 双 prong（others 开写面关其他 open / given 插入与
  name 变更面自身让位）+ 组口径（同树根 + 同名非空 name + DETAILS）。
  核心结构发现：innerHTML-into-sel 解析产物 `_zwMEl` plain wrapper 与
  t8r 物化 identity 元素混合同树——统一 node-based 核心
  `__zwDetailsExclusivityNode4100`（childNodes DFS 组枚举、`_elKey`/对象
  身份双态）+ R4009 队列第 4 参 plainNode 通道 + 解析扫描顺序插入模拟
  （树序第一 open 胜出，CeAttach 整批一次扫描）+ 六挂接面 + 解析面三处。
  part01 `_REFLECTED_NAME_TAGS` details + part03 原型 name accessor。
  取舍：解析面排程 toggle（spec 字面）与 test 3 期望序互斥，取 spec 口
  径维持 test 3 F。part60 T1–T8 契约。九域 4873→4887P（66.29%），对 s20
  逐 subtest diff 净 +14P、回归 0，全落 interactive-elements（95→109P）：
  name-attribute 2P/14F+Timeout→15P/3F、fragment-insertion 1P/0F。
  zero-engine --lib 3374P/0F、page-runtime+webview 785P/0F、integration
  html_compat 21P/0F、clippy/fmt 过、零新增依赖。挂账：toggleEvent 尾部
  ontoggle（document-parse 面）、name-attribute test 3 期望序互斥观察、
  xhr/template setup error、dialog 元素簇（下一切片）。账本
  [evidence/2026-10-21-m2s21-r4100.md](html-semantics-compat/evidence/2026-10-21-m2s21-r4100.md)。
- **2026-10-21 M2 切片 20**：details/summary 深层面（R4009，rollover 接续
  ——上会话 429 中断遗留 WIP 本会话续推收口）——toggle 事件任务队列
  （part03：去重/合并/净零/同态不排程/detached；choke point part04 `open`
  IDL setter + attr 尾钩）+ summary 激活行为（part03 bubble 站钩 + part04
  post 分支，首个 summary 元素子 + 交互豁免含祖先面）+ ToggleEvent 接口
  （part05）+ 宿主空格路由（webview SUMMARY → plan_summary）。**宿主双
  toggle 缺陷重设计**：plan_summary followup toggle 与队列双派被集成测试
  证伪（`toggle:true,toggle:true`），移除 followup 发射权归队列，断言
  `click:false,toggle:true,click:true` 不变、事件形状转正。part60 T1–T9。
  九域 4855→4873P（66.09%），对 s19 逐 subtest diff 净 +18P、回归 0，
  全落 interactive-elements（77→95P）：toggleEvent 9P/0F、
  activation-behavior 9P/0F、anchor-without-link 2P/0F、
  details-keyboard-activation 1P/0F。zero-engine --lib 3358P/0F、
  page-runtime 136P/0F、webview 785P/0F、renderer activation 9P/0F、
  integration html_compat 21P/0F、clippy/fmt 过、零新增依赖。挂账：
  details name= 互斥簇 18F+Timeout（下一切片首选）、toggleEvent 尾部
  ontoggle 内容属性面、ToggleEvent.source/commandfor、anchor-with-inline
  synthetic click 导航缺口（导航域）、parser form pointer 4F、
  navigation-compat M3 项维持。
- **2026-10-21 M2 切片 19**：label 本体激活转发（R4008，rollover 双触发
  两会话同树协作合账，commit 88793f87d）——shim 交互豁免身份比较（bubble
  站挂钩 + R155 post 段，target 即 label 本体不豁免）+ 宿主面上移（renderer
  activate_label_at 既有 + webview user_actions plan 链递归 Activate，
  labelable 过滤防环/取消抑制/disabled noop）；shim 不承接宿主 generic
  Event（首发 shim finally 段转发被 renderer 双激活集成测试证伪后重设计，
  checkbox 双翻）。part60 W1–W6 重写 + integration 三面新测。九域
  4848→4855P（65.91%），对 s18 逐 subtest diff 净 +7P、回归 0：
  proxy-modifier tentative 4F 全翻绿（主面）、label-forwarded-click-pointer
  -properties 未上报→2P/0F（isTrusted trusted 面随宿主派发通道翻绿）、
  simulated-click-inert 翻绿。zero-engine --lib 3355P/0F、webview lib
  785P/0F、renderer activation OK、integration html_compat 21P/0F、
  clippy/fmt 过、零新增依赖。协作分工：对侧会话代码提交 + 全量 corpus，
  本会话 kill 冗余全量轮 + docs 合账 + 账本独立复算。挂账：parser form
  element pointer 表规则 4F 同根（深结构单独立项）、details 键盘激活 /
  summary toggle coalescing、navigation-compat M3 项维持。
- **2026-10-21 M2 切片 18**：label.form IDL（R4007，rollover 接续）——
  `.form` getter 分发区补 LABEL 分支，`_zwLabeledControl` 取控件后委托
  控件 `.form`（R4005 关联面全继承），label 自身 form 属性不参与。
  part60 契约 1 测（W1–W3）。九域 4838→4848P（65.82%，账本条目口径），
  对 s17 逐 subtest diff 净 +10P、回归 0：form.html label.form 8 案
  翻绿（7P/10F→15P/2F）、dialog autofocus 两案意外翻绿（焦点链联带）。
  zero-engine --lib 3325P/0F、make test 76 腿 22222P/0F 全绿、clippy/fmt
  过、零新增依赖。挂账：parser form element pointer 表规则 4F 同根
  （深结构单独立项）、proxy-modifier tentative 4F、details 键盘激活 /
  summary toggle coalescing、navigation-compat M3 项维持。
- **2026-10-21 M2 切片 17**：form owner 动态关联（R4005/R4006，rollover
  接续）——`.form` getter 重写（JS 视图 parentNode 三族统一上行 + form
  attr 控件所在树查找 + latest-wins 读 + OBJECT gate）、R52 消零 purge
  WeakRef 复活（identity 断修复、泄漏口径不变）、gEBI pending 窗口同 id
  树序取首 + `_r125Overridden` 改名门。part60 契约 1 测（W1–W4）。九域
  4774→4838P（65.90%），对 s16 逐 subtest diff 净 +64P、回归 0：
  form_attribute.html 43P/57F→100P/0F 满绿、association.window.js 翻绿、
  object.form/form-elements-filter +1+1、select-tab* 意外翻绿 2、
  load-error-events 上报展开 +2。zero-engine --lib 3310P/0F、make test
  全腿绿（GPU adapter 94P/2F 容差漂移结构不阻塞非本轮、归 rendering 流
  观察账）、clippy/fmt 过、零新增依赖。全量轮 5400s 限被兄弟流并行负载
  顶爆、10800s 放行记 evidence。挂账：label.form IDL（form.html 余 10F，
  下一切片）、form_owner_and_table parser 指针表规则 2F（深结构）、
  proxy-modifier 4F、details 键盘激活 / summary toggle coalescing、
  navigation-compat M3 项维持。
- **2026-10-20 M2 切片 16**：label 激活行为面（R4004）——交互内容判定
  存在性走 has 句柄（get_attr_handle 缺失返空串致恒真误判）、R368 盖章
  属性回填、label 激活转发三路归一（bubble 站挂钩新设 + post/trap 迁移
  `_zwLabeledControl` for 感知 + 交互 target 豁免补齐 + 双闸互斥收敛
  自转发死循环与 post+trap 双转）+ plainParent hop/R11 媒体补链/DFS 控件
  查找/盖章子事件方法转 live。part60 契约 1 测。九域 4710→4774P，对 s15
  逐 subtest diff 净 +64P、回归 0：clicking-noninteractive-* 57F 全数
  翻绿、意外翻绿 7 案（label-inside-anchor/proxy-click Timeout→Pass、
  dialog 系 5 案含 inert-label-focus 观察账消账）。zero-engine --lib
  3289P/0F、make test 主矩阵绿 + browser bin 腿既有失败 stash 对照定谳
  非本轮、clippy/fmt 过、零新增依赖。make test 与 clippy 并行触发 guard
  总内存超限（8GB 预算下编译峰值 ~8.8GB，16GB 放行）记 evidence。挂账：
  form_attribute 动态关联 ~50F（下一切片）、proxy-modifier tentative 4F、
  details 键盘激活 / summary toggle coalescing（M1 起既有）、
  navigation-compat M3 项维持。
- **2026-10-20 M2 切片 15**：labels IDL 簇 + label.focus() 前递
  （R4003/R4003b/R4003c，rollover 双触发两会话同树合记）——part03
  `_liveQueryCollection` 泛化（自定义快照 + NodeList 原型 target +
  `__zwNL` 印记透读）、part04 labeled control 算法 + labels 反查活集合
  （[SameObject] per-element-key 缓存居 part01 共享作用域）+ focus 前递
  + body/html offsetParent→null。part60 契约 3 测。九域 4683→4710P
  （64.00%），对 s14 逐 subtest diff 净 +27P、回归 0（labelable-elements
  +21、button/input-labels +1+1、forward-focus +4；inert-label-focus
  失败模式漂移 +0，run2 二进制未含 R4003c）。zero-engine --lib
  3273P/0F、make test 58 腿绿 + browser bin 腿 xvfb 复跑 665P/0F、
  clippy/fmt 过、零新增依赖。并发碰头信号（共享 target 指纹 E0425、
  run kill 史 ×4、06:43 一次死因未考证）记 evidence。挂账：
  clicking-noninteractive-* 57F（下一切片首选）、form_attribute 动态
  关联 ~50F、proxy-modifier 4F、inert-label-focus 负载超时观察账、
  navigation-compat M3 项维持。
- **2026-10-20 M2 切片 14**：meter 实际值算法 + double setter（R4002）——
  part04 `_zwMeterActualValues`（六 IDL getter 缺省/钳制 + 浮点整串解析，
  前导 + 跳过为语料钉死行为；R5009 片 e 旧前缀解析修正）+ 六 setter
  WebIDL double 转换（非有限 TypeError）+ best representation 写属性 +
  METER 移出 value 纯反射族（part03 host-read 委托支/part05 tag-gate
  不变）。part60 契约 1 测（W1–W5）。meter 簇 14P/48F→62P/0F，forms
  3688→3736，九域 4662→4683P（63.48%），对 s13 逐 subtest diff 净 +21P。
  **跨流窗口 −27P 对照实验归因、meter 排除**：B2 提交树 16 变更文件复跑
  与 B1 全量账本持平（账本绑定提交树）；无 meter 对照重建原样复现；
  scripting-1 −24 主假设归 script-sandbox 模块缓存三部曲（8458c9223/
  a85c7ade4——安静机恒 Timeout 而 s13 负载下 18P，方向与边界漂移相悖）、
  interactive-elements −3 签名匹配 s7 runner 通道 realm 家族（Fail
  "wrong global"）与窗口候选 3ccfdc479/b2fde4965/368ce9158 并存两说，
  commit 级 bisect 因旧 commit v8 源码构建指纹受阻留属主流。本切片由
  rollover 双触发并发会话同树推进，记账已合并（见切片账本说明）。make
  test 59 腿 58 绿 + 1 既有无 display 环境腿（xvfb 同二进制 665P/0F）、
  clippy/fmt 过、零新增依赖。挂账：labels IDL ~30F（下一切片）、
  form_attribute 动态关联 ~50F（中深）、clicking-noninteractive-* 57F、
  module/dialog 跨流碰头两项（属主流核账）、navigation-compat M3 项维持。
- **2026-10-19 M2 切片 13**：selected autocomplete value 算法 + HTMLCollection
  named access（R4001）——part04 category 全表 + token 算法（INPUT/SELECT/
  TEXTAREA getter，FORM/其余保持枚举反射；field token 小写为 corpus+Chrome
  67/67 钉死行为）、part05 setter 缓存旁路、part03 `_liveQueryCollection`
  named property 补齐（`document.forms.X` 可达）、part03 `.form` 表补
  FIELDSET（named access 修复暴露 vacuous-pass 真值化）。part60 契约
  W1–W5；part15 R3204 input 缺省断言 'on'→''（spec 修正）。九域
  4581→4662P（63.20%），对 s12 逐 subtest diff 净 +81P、回归 0（−11 条
  上报漂移全为 runner 超时边界，P 零损失）。zero-engine --lib 3245 全过、
  make test 全腿绿、clippy/fmt 过、零新增依赖。挂账：meter 净化 44F
  （下一切片）、labels IDL ~30F、form_attribute 动态关联 ~50F（中深）、
  encoding 三簇/double-submit 归 navigation-compat M3 维持、R3204 类
  既有契约钉旧行为风险记入（动枚举反射分发区先跑 part10/part15）。
- **2026-10-19 M2 切片 12**：populateForm 同 turn 二次插入 sibling 断链
  修复（R4000）——`_zwPendingParsedForSel` 候选收集 `candCount > 1`
  提前 return 缺陷删除（签名盖章截断致第二批 pending 节点 hits 恒 0，
  兄弟导航断链；歧义保守语义由 hits 唯一性判定承担不变）。part59 契约
  1 测（W11）。解锁切片 11 flag 契约：form-submission-algorithm t2/t3
  Fail→Pass，九域 4579→4581P（62.10%），对 s11 逐 subtest diff 净
  +2P、回归 0（账本绑定 rebase 后合并树，对 rebase 前复跑 0 变化）。
  zero-engine --lib 3224 全过、clippy/fmt 过、browser bin 腿 :102
  665P/0F、quickjs 矩阵腿全绿、零新增依赖。挂账：
  form-submission 余 F 归 iframe 导航基建簇（navigation-compat M3）、
  既有小挂账维持、二批插入后 body.childNodes 融合视图丢 base 尾节点
  （corpus 无消费面，观察项）。
- **2026-10-19 M2 切片 11**：firing submission events flag + dialog
  returnValue WebIDL 化 + 宿主 click 坐标注入（R3999）——每 form 粒度
  `__zwFiringSubmissionEvents`（spec 步 5.1/5.2/5.4/5.7，入口早退 +
  窗口置位 + finally 清位，仅激活/requestSubmit 通道）；returnValue
  setter DOMString 转换（null→"null"，undefined→"" 兼容例外）；泛型
  click offsetX/offsetY 派发期注入（与 image selected coordinates 同
  口径，事件类保持泛型不改 R108 面；并行 regression-guard 流协作归并，
  账本排查记录 3）。part59 契约 3 测 + part14 期望修正。九域
  4577→4579P（62.07%），对 s10 逐 subtest diff 净 +2P、回归 0：
  dialog-return-value 翻绿、dialog-form-submission 8/8。zero-engine
  --lib 3193 全过、make test 全腿绿（browser 腿 :102 复跑 664P/0F）、
  clippy/fmt 过、零新增依赖。挂账：populateForm 同 turn 二次插入 sibling 断链
  （R3254-K3 面，挡 form-submission-algorithm t2/t3，flag 契约已就位）、
  既有小挂账维持。
- **2026-10-18 M2 切片 10**：form 提交 construct-entry-list 对齐 + dialog
  提交分支（R3910）——constructing entry list flag 共享（submit 入口
  早退先于 submit 事件、构造窗口置位/finally 清位）、formdata 事件共享
  条目存储派发（handler 增删改写提交条目）、action 解析按 spec 顺序后移
  （旧位序丢 formdata/dialog 面）；method=dialog 分支（无祖先 return、
  image 坐标、optional value、_zwDialogClose）；close-the-dialog 步 1
  无 open 早退（四处共享路径）；SubmitEvent/FormDataEvent 构造器
  WebIDL 化。part59 契约 6 测。forms 3602→3605P、dialog-element
  38→47P、九域 4565→4577P（62.05%），对 s9 账本逐 subtest diff 净
  +12P、回归 0。zero-engine --lib 3169 全过、make test 全腿绿
  （browser 腿 :102 复跑）、clippy/fmt 过、零新增依赖。挂账：
  form-double-submit-requestsubmit 36F→Timeout（iframe 导航基建，归
  navigation-compat M3）、submit-dialog-close-event 挂起维持、
  firing submission events flag（t2-t4）、image 坐标 offsetX 面（t5）、
  dialog-return-value returnValue setter String 强转 1F。
- **2026-10-18 M2 切片 9**：selectedness/selectedIndex 语义（R3909）——
  selectedness setting（受影响者胜 affKey + size 1 非 disabled 回落）、
  ask-for-a-reset 汇流钩（`_mo_notify` childList 触发、批插依序、innerHTML
  容器合流、Blink 重回落）、selected 属性变更步两半、selectedIndex setter
  （可全不选）、select.value setter 双形态真态写（旧 guard 双重排除丢写面
  根因修复）、form.reset 复位算法、select.remove(n)/add 祖先检查 + 元素代理
  add、select value-missing（placeholder 四条件）、Option() 构造器
  （属性只随 defaultSelected + clean 层）；读侧 selectedIndex/value/视图
  全改 JS 真态，pristine-only size 1 跳 disabled 回落。part59 契约 12 测。
  select-value-selectedOption +8、selected-index 13/13、select-validity
  6/6 等**净 +34P**；forms 3569→3602P、九域 4531→4565P（61.59%），对
  s8/s6 账本 diff 均零回归。zero-engine --lib 3138 全过、make test 全腿绿、
  clippy/fmt 过、零新增依赖。挂账：option.form 1F、text-setter 1F、
  replaceChild sel 形落尾缺口、t8g 墙钟钉 clippy 并行负载边缘越限观察账。
- **2026-10-18 M2 切片 8**：HTMLOptionsCollection/option list 基建
  （R3908/R3908b）——`_zwListOfOptions` 统一构造（嵌套 optgroup 排除、
  select/hr/datalist 整枝）、HTMLOptionsCollection 代理（length/索引
  setter、replace 取先删后插序规避 overlay identity miss）、
  `_zwIsSelectTag` 前缀判定六处、selectedOptions 代理三 trap + 视图归一、
  option.selected dirty selectedness（不反射；宿主激活脚本
  script_gen.rs 分流属性写）。part58 契约（W1–W9）。common 三文件
  15P、selectedOptions 簇 18P、script-mutation 5P 等**净 +54P**；forms
  3515→3569P、九域 4477→4531P（61.13%），对 s7/s6 账本 diff 均零回归。
  zero-engine --lib 3111 全过、make test 全腿绿（browser 腿 :102 复跑，
  s6 同款环境归因）、clippy/fmt 过、零新增依赖。挂账：ask-for-reset 3F、
  selected-index 7F、add/remove 4F、constructor dirtiness 3F、validity 7F、
  通用 replaceChild sel 形 oldChild 落尾缺口（独立排查）。
- **2026-10-18 M2 切片 7**：option 元素文本/反射语义（R3907）——
  `.text` getter「collect text value + strip and collapse」（script
  命名空间排除模型、子列表递归、NBSP 保留）、`.label`/`.value` 无 ns
  存在即采纳 + collapse 回落（R122 实例层判定）、`.text` setter
  textContent 委托；part03 助手 4 个（R3028 latest-wins 先探、64 层
  防环）+ part04 setter 支，part57 `test_option_semantics_r3907` 契约
  （W1–W7；part16 近 2000 行上限新切片文件）。option 族 36→101P
  （+65P），forms 3450→3515P、九域 4409→4477P（60.40%），对 s6 账本
  逐 subtest diff 零回归。zero-engine --lib 3107 全过、clippy/fmt 过、
  零新增依赖。**stash 对照实验**排除 dialog +3P 因果（runner 案级
  realm 身份非确定性，归 wpt-runner 通道挂账；d11 detached-document
  showModal 不抛为 `_zwIsConnected` 预存缺口）。挂账：option 余簇 7F
  异根（selectedness dirtiness 5F、form owner 1F、isConnected 面 1F）、
  HTMLOptionsCollection/option list 基建簇 ~45F+。
- **2026-10-17 M2 切片 6**：文本控件选区语义（R3906）——selection
  setter「set the selection range」算法 + value setter 选区折叠 +
  异步 select 事件（part06 `_zwScheduleSelectEvent`）+ textarea 换行
  归一修正 + attr 回落 latest-wins + defaultValue= 真脏态守卫 +
  type 变态复位 direction 归一（初版裸 'none' 满账本暴露 80F 同名
  回归后修正，作废首轮取账重跑）+ runner variant meta 展开（R329
  机制复用）。part16 `test_textfieldselection_r3906` 契约（W1–W9）。
  textfieldselection 784P/0F、type-change-state 462P/0F；select-event
  0→270P、selection-not-application variant 化 183+79P；forms
  2848→3450P、九域 3807→4409P（4409/7412 = 59.48%，+602P，分母
  +499），对 s5 账本逐 subtest diff 零回归。zero-engine --lib 3100
  全过、clippy/fmt 过、零新增依赖；make test 三个 browser shell
  间歇失败归因为环境面（X :100 catlink 桥死亡 + load 抖动，隔离与
  main CI 双证）。挂账：editing 管线 `_textSelection` 裸 'none'
  观察账；066.html 预存 hang。
- **2026-10-17 M2 切片 5**：select.showPicker() spec 检查序（R3905）——
  part03 方法分发区新支（disabled→InvalidStateError 先于
  being-rendered→NotSupportedError（display:none + cv:hidden 祖先）先于
  NotAllowedError 门、`_zwTransientActive` 同步消费、multiple/size 不抛），
  part16 R3905 契约单测（9 场景 + 抛错面激活不消费钉针）。show-picker
  六文件 0→7P、customizable-select/nested-options 联带 +1P，forms
  2840→2848P、九域 3799→3807P（零回归；分母 6913 不变）。
  `make test` 全绿、clippy/fmt 过、零新增依赖。教训：corpus 全量轮
  禁与 make test/clippy 并行（首轮负载噪声作废）。
- **2026-10-17 M2 切片 4**：valueAsNumber/valueAsDate setter 抛错面
  （R3904）——part04 两 setter 按 spec setter steps 重排（valueAsNumber
  ±Infinity→TypeError 先于适用性、不适用→InvalidStateError、NaN→''；
  valueAsDate WebIDL `Date?` 转换先行（undefined→null）、不适用
  （含 datetime-local）→InvalidStateError、null/undefined/Invalid Date
  →清空），part16 R3904 契约单测（步序钉针 + 三清空面）。typeerror
  两文件 8F + datetime-local-valueasdate 1F 翻绿、同族 invalidstateerr
  两文件联带 +2P，九域 3788→3799P（零回归；分母 6913 不变）。
  `make test` 全绿、guarded-clippy/fmt 过、零新增依赖。
- **2026-10-16 M2 切片 3**：input.showPicker() spec 检查序 + 瞬态激活消费
  （R3903）——part03 方法分发区新支（不可变检查先于激活门/消费、readonly
  适用 12 类型表、NotAllowedError 门、`_zwTransientActive` 同步消费、无
  focus），latest-wins 属性读三级回落，part16 契约单测。show-picker 三文件
  0→129P（user-gesture 63 / disabled-readonly 44 / does-not-focus 22），
  forms 2697→2829P、九域 3626→3788P（零回归；分母 6656→6913 系 img
  naturalWidth 文件完整上报）。`make test` 全绿、guarded-clippy/fmt 过
  （fmt 既有违规系并行会话遗留，见切片账本）、零新增依赖。
- **2026-10-16 M2 切片 2c**：stepUp/stepDown 全七类型 spec 步进算法 +
  valueAsNumber/valueAsDate IDL 全类型面（R3902c）——part03 `_parseHtmlDateValue`
  date/week/datetime-local 回读校验 + `_formatHtmlDateValue` week/
  datetime-local 新支 + stepUp/stepDown 分支重写（`_reflectedTypeEnum` 类型门、
  `_suDefaultStep`/`_suScale` 缺省表、`_suParseBound` 类型域解析、`_suKInt`
  相对容差近整判定、步进基与 spec 步 3/4/5/7/8/9/10；sel 路径 min/step 走
  `__zw_get_attr_lw` latest-wins 读）+ valueAsNumber getter 全类型，part04
  valueAsNumber setter date 族/month 扩面，part16 R3317 契约修正（方法恒
  暴露 + InvalidStateError + 离格 snap→6）。stepping 簇 +68P、联带 IDL 面
  +64P，九域 3494→3626P（零回归）。`make test` 全绿、guarded-clippy/fmt 过、
  零新增依赖。
- **2026-10-16 M2 切片 2b**：dirty checkedness/value + files IDL +
  `:active`/`:hover` + compound pending 回落（R3902b）——query.rs
  Active/Hover 识别臂，part01 dirty/files 两 map，part03 files/indeterminate
  `_realTag` 门 + 非 file 恒 null + checked 脏态优先，part04 setAttribute/
  removeAttribute('value') 已脏不覆写 + clone 传播 dirty value/checkedness/
  indeterminate + files setter（FileList 存引用/null no-op/否则 TypeError），
  part05 `.checked=` 置脏 + DataTransfer.files FileList 视图，part06 compound
  pending 回落（`[` 预门）。cloning-steps/untrusted-key-event/files/
  checkable-active-space 四簇 +125P，九域 3369→3494P（零回归）。
  `make test` 全绿、guarded-clippy/fmt 过、零新增依赖。
- **2026-10-16 M2 切片 2a**：focus/focus-visible 面（R3902）——part06
  focus-visible 模态旗标，part04 焦点派发接线 + matches() `:focus`/
  `:focus-visible` 分支（isConnected 连接性判定），part01 rAF 预算
  64→512（promise_test 串行链 double-rAF 断链根因）。focus
  -dynamic-type-change 双文件 0→120P、forms +121P、九域 3248→3369P
  （零回归）。`make test` 全绿、clippy/fmt 过、零新增依赖。
- **2026-10-15 M2 切片 1**：input value IDL 面（R3901）——part01 四值模式/
  净化/类型迁移助手，part03 getter 模式分派，part04 setter 分派 + setAttribute/
  cloneNode 接线，part05 `input.type=` setter，用户编辑通道
  （`__zw_text_input`/`__zw_text_delete`/execCommand/script_gen 三脚本）净化
  绕行旗标。the-input-element 528→951P、forms 1897→2320P、九域全量
  2715→3248P（+533，零回归）。中途 13 回归（textarea 缓存写误删）当日定位
  恢复（经验已沉淀 docs/learnings/bugs/2026-10/）。`make test` 全绿、
  clippy/fmt 过；csv 行回填全量数字。零新增依赖。
- **2026-10-15 M1**：fetch 脚本九域+叶子助资产（jsdelivr CDN 回落先落入
  lib.sh 共享路径）；runner 通道 `testharness-html-semantics`（九域顶层递归
  扫描 + skip 规则双侧同源）；基线 42.63%（2715/6369，1151 案），分域账与
  失败簇归因 evidence 落档；csv 数据行回填。挂账：span-limits /
  downloading-resources 两处挂死排除（FIXME）。`make test` 全绿、
  clippy/fmt 过、零新增依赖。

