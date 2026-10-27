# HTML Semantics 兼容 — html/semantics 语义面（主攻大域）

**版本**: v2.0
**日期**: 2026-10-26（v1.0 立项 2026-10-08）
**状态**: Active——M2 切片 115 施工收口（script charset 响应解码轮：charset 族 **17F 翻绿**，scripting-1 **867P/138F/37T→884P/121F/37T**，全量套件 A/B 6545P/1220F/112T→**6565P/1202F/111T 零 P→非P**——script-charset-01 6P + module/charset-01 6P + charset-02 3P + charset-03 4P；根因：`serve-with-content-type.py` 动态端点不在语料 pull 面（classic 文件读落空透传外链宿主取回 Err；模块面 s107 仿真被 JSON 意图门收窄非 JSON ct → Err）——三刀（runner testharness.rs + Cargo.toml +encoding_rs 工作区既有）：①共享查询解析 helper ②fetcher scripty 意图分支（非 JSON 而 scripty → 交付 fn 文件**恒 UTF-8 lossy**——模块 spec 恒 UTF-8 charset 三忽略、BOM 不嗅探体内 U+FEFF 即语义）③inline classic 仿真分支（**非 module 型**——首版乌龙未辨 module 令 charset 感知内联反向翻红 module 面已修正为透传；解码序 BOM > ct charset 参数 > charset 属性 > UTF-8，encoding_rs）；挂账 script-charset-02（文档编码 windows-1250 基建）/03（运行期 charset 属性需 fetch 桥带提示）/css·text-module charset 5T（待决须用户拍板）；下一步 s116 候选：execution-timing 余 F 定刀 / 承袭挂账序；**用户决策 2026-10-23**：parser form element pointer 4F **缓行**——与 declarative shadow DOM 打捆 parser 批次、M2 收口后尾段单独立项（[决策记录](html-semantics-compat/evidence/2026-10-23-user-decision-parser-form-pointer-defer.md)），流按其余挂账序推进

> **▶ 当前状态（2026-10-28 M2 切片 115 施工收口，script charset 响应解码轮）**：
> **入场定谳**：charset 族五组失败三根因——①`serve-with-content-type.py` 动态
> 端点不在语料 pull 面：classic 面文件读落空→整段透传外链→宿主取回 Err 不执行
>（script-charset-01 全 6F）；模块面 s107 仿真被 JSON 意图门收窄，非 JSON ct →
> Err 网络错误（module/charset-01/02/03 全 11F）。②文档编码 windows-1250 未建模
>（script-charset-02 3F）。③运行期 charset 属性变脸无元素上下文（03 1F）。
> ②③挂账；css/text-module charset 5T 属待决（须用户拍板）。
> **spec 分流**：classic 解码序 BOM > ct charset 参数 > charset 属性 > 文档编码
>（runner 恒 UTF-8）——parser classic 走 inline_local_scripts（元素上下文在位）；
> module 恒 UTF-8（charset 参数/属性/BOM 三忽略——charset-01 判据即
> `assert_not_equals` mojibake；charset-02 UTF-8 BOM 体内 U+FEFF 为 JS 空白照常
> 执行、UTF-16 乱码语法错）——模块走 wpt_data_script_fetcher。
> **本轮三刀**：①共享查询解析 helper（fn/ct 百分号解码）②fetcher scripty 意图
> 分支（非 JSON 而 scripty → 交付 fn 文件恒 UTF-8 lossy，BOM 不嗅探；非 scripty
> 非 JSON → Err s107 口径）③inline classic 仿真（非 module 型 + 文件名匹配；
> decode_classic_script_bytes 四级解码序）。**首版乌龙**：inline 未辨 module——
> module 外链被 charset 感知内联（charset-01 余 4F 反向翻红、charset-02
> 1F→2F：BOM 嗅探令 UTF-16 模块照常执行）；修正 module 整段透传（ASCII 剥白 +
> ASCII-ci 本地判定，不扩 engine 公共面）。
> **验证**：runner bin **240/0**（+2 钉）。charset 族定向：四页 **17F 全翻绿**
>（script-01 6P / module-01 6P / module-02 3P / module-03 4P）；挂账三组形态
> 不动。**全量套件 A/B**（vs s114 基线）：6545P/1220F/112T→6565P/1202F/111T，
> 归一 diff = charset 17F→P + focus flake 复原（1T+77P→80P）+ modulepreload
> 假行消退（3F+1T→2F+1T 常态，与 s113 基线/定向重跑一致）——**零 P→非P 成立**。
> fmt + clippy（runner --all-targets -D warnings）过。渲染面零依赖——
> reftest/product-smoke 未跑。账本见
> [evidence/2026-10-28-m2s115-script-charset-decode.md](html-semantics-compat/evidence/2026-10-28-m2s115-script-charset-decode.md)。
> **范围**：runner testharness.rs + Cargo.toml（encoding_rs workspace 既有）；
> engine/webview/语料零变更；语料不可改维持。
> **挂账**：script-charset-02（文档编码 windows-1250——corpus loader 编码面
> 基建）；script-charset-03（运行期 charset 属性——shim fetch 桥带提示，跨界
> 契约）；css/text-module charset 5T（待决须用户拍板）；classic 运行时取回无
> charset 上下文恒 UTF-8 近似（语料无此形）。承袭：143 write 注入 `</script>`、
> defer 多写入完成序近似、063 Timeout、non-external-no-import、parse-error 族、
> http-accept-header、referrer-policies、dynamic import() with 透传、classic
> 外链 SRI、collect 账本打包重构、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt
> 插桩来源待查；**待决（须用户拍板）**：css/text 数据模块语料资源缺失。
> **下一步**：s116 入场定刀位（候选：execution-timing 余 F 定刀 /
> dynamic-import credentials 12F / referrer-policy 族 9F / 承袭挂账序）。
> **切片间隙**：本轮 runner 取回/内联面；下一候选同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 114 施工收口，write 面 defer 改道 + xlink:href 回退轮——其下一步 s115 charset 族已由 s115 收口，见上）**：
> **入场定谳**：三候选各 1F——096 write defer 外链走「取回即执行」阻塞链早执行
>（got `[#1, external#1, #2]`）；142 write 只读 `src`，SVG xlink:href 写入形外链
> 不执行（缺 `external script #1`）；143 write 注入多余 `</script>` 须关外层 SVG
> script 提前执行——write 片段解析与主解析器 open-element 栈耦合，挂账不动。
> write+defer 语料全树仅 096（102 命中 grep 但 defer 只在标题文字）；
> xlink:href 写入形仅 142，爆炸半径封闭。
> **本轮两刀**（shim part06.js `__zwDocWriteRunFace`）：①defer 改道臂——立即
> fetch（挂 s110 在飞计数挂 load 等待）、代码入 FIFO 队、排水投
> `__zwSchedRegister('defer',0,fn)`（s112 布防实时查 `__zwSchedHasDeferred()`，
> 运行期注册可见；排水同任务派 DCL = spec the-end 序；慢源迟到 `DeferDrained`
> 补执行；nomodule/s113 classify 门保持；throwaway 槽位仍留主链；非 runner
> 环境维持阻塞链）②src 读空回退 `xlink:href` + 副本补 `setAttribute('src')`
> ——既有 fetch 链零改动。③open() 三记账随 s97 同步重置。
> **验证**：定向 096/142 Pass。execution-timing 全子树 A/B（150 页）：唯一变化
> 096/142 F→P、行数恒等、零 P→非P。**全量套件 A/B**：6546P/1221F/111T→
> 6545P/1220F/112T 行数对账 7878→7877 无蒸发页；另两处异常定性负载 flake
> （focus 3 P→T 定向重跑 80P/0T 复原；modulepreload 基线即 2F+1T 满量漂 3F
> 重跑复原）——**全树零 P→非P 成立**。Rust/依赖零变更，无新增 fmt/clippy 面。
> 渲染面零依赖——reftest/product-smoke 未跑。账本见
> [evidence/2026-10-27-m2s114-write-defer-and-xlink-href.md](html-semantics-compat/evidence/2026-10-27-m2s114-write-defer-and-xlink-href.md)。
> **范围**：shim part06.js 单文件（+51 行）；engine Rust/webview/语料/依赖零变更；
> 语料不可改维持。
> **挂账**：143 深层面（write 注入 `</script>` 关外层 SVG script + SVG script
> 文本拼接求值）；defer 改道近似——多 defer 写入按取回完成序近似文档序、
> interactive 后注册无二次布防（语料均无此形）；part06.js 18436 行既有超限
>（拆分需独立切片）。承袭：063 Timeout、non-external-no-import（module async）、
> parse-error 族、http-accept-header、referrer-policies、dynamic import() with
> 透传、classic 外链 SRI、collect 账本打包重构、接口对象基建面、FIXME 组、
> /tmp/zw-hb*.txt 插桩来源待查；**待决（须用户拍板）**：css/text 数据模块语料
> 资源缺失。
> **下一步**：s115 入场定刀位（候选：143 深层面评估 / execution-timing 余 F
> 定刀 / 承袭挂账序）。
> **切片间隙**：本轮 engine shim 面；下一候选同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 113 施工收口，type/language essence 16 集合轮——其下一步 s114 write 面族已由 s114 收口，见上）**：
> **入场定谳**：s113 承 s112 解锁页 script-type-and-language-js（456 子测
> 229P/227F）。227 失败全为「shouldn't run」判据：type 臂 53 + language 臂 173 +
> 解析插入面 1（type="javascript" 被旧 `ends_with("javascript")` 误跑）。四动态
> 执行臂门况盘点：maybe-start 旧 `indexOf('javascript')` 子串 lax 门；R387 三臂/
> s95 children-changed/`_zwPrepareSrcScriptGate` **全无 type 门**。
> **spec 模型**（live spec 提取）：type string 推导——type 字面空串/无 type 且
> language 字面空串/双缺 → "text/javascript"；type 在场取原值；否则
> "text/"+language **原值不剥白**。分类：type string essence match（HTML
> §scripting languages 16 项 legacy 全表 ASCII-ci **等值、无参数语法**——
> with-params 语料 text/javascript;charset 不跑）→ classic；type 属性在场加
> **第二替代**：属性值剥边缘 ASCII ws 后 match（language 推导无此替代——
> " javascript" 不中，空白形不跑）；module；余 data block。
> **本轮两刀**：①extract.rs `script_type_is_javascript` 收窄 16 集合等值表
> （字面空串/剥后空串分轨；module 剥白口径不动，s93 钉保持）；②shim
> `_zwScriptTypeClassify`（推导+essence 双轨）+ blocked 门接入四臂（blocked 不执行
> 不置 run-once，摘属性重走门；module 沿用各臂既有让位）。
> **中间态一乌龙**：首版对 language 推导串也做边缘剥白——"text/javascript " 剥后
> 中集合误跑（family 余 60F 全空白形）；对照 spec 双轨文本修正后 family 463P/0F。
> essence 语义与 mimesniff MIME 解析不同：是字符串比对，参数串整体不中。
> **验证**：engine lib **3564/0**（含 s113 essence 钉）；webview **814/0**；runner
> bin **238/0**。**全量套件 A/B**（stash 前/后二进制同协议）：7900 子测行数恒等，
> 归一 per-page diff **唯一变化**即目标页 227F+229P→456P，**全树零 P→非P**
>（/tmp/s113-full-base.log、/tmp/s113-full-suite.log 存档）。fmt + clippy
>（engine/wpt-runner -D warnings）过。渲染面零依赖——reftest/product-smoke 未跑。
> 账本见
> [evidence/2026-10-27-m2s113-type-language-essence.md](html-semantics-compat/evidence/2026-10-27-m2s113-type-language-essence.md)。
> **范围**：engine extract.rs + shim part03/part04；webview/语料/依赖零变更；
> 语料不可改维持。
> **挂账**：解析面 language 属性未建模（无语料需求，遇语料再补）；shim module
> 判定剥白口径与 spec 精确等值差（无语料判据不动）；js-svg/-xhtml 变体不在
> runner 枚举面（SVG language 豁免未验证）；096/142/143（write 面族）、063
> Timeout、non-external-no-import（module async）；承袭：parse-error 族、
> http-accept-header、referrer-policies、dynamic import() with 透传、classic 外链
> SRI、collect 账本打包重构、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt 插桩来源
> 待查；**待决（须用户拍板）**：css/text 数据模块语料资源缺失。
> **下一步**：s114 入场定刀位（候选：142 write 臂 xlink:href 单点 / 143
> eventOrder 核验 / 096 write(defer)）。
> **切片间隙**：本轮 engine shim+提取面；下一候选同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 112 施工收口，解析期外链时序调度轮——其下一步 s113 type/language 缺口已由 s113 收口，见上）**：
> **入场定谳**：s112 承挂账「trickle 建模 ~11 行」——探针实证 trickle 族实为十页
> 而非六页，且分三根因：①调度缺失族八页（050/085/086/088/103/110/111/112）——
> runner `inline_local_scripts` 解析期外链一律原位内联直执，async/defer/trickle
> 时序语义塌缩为「立即同步」；②投递截断族——099（include-11.js srcdoc 串含字面
> `</script`，投递面提前封口 SyntaxError 零日志）+ script-type-and-language-js.js
> 43-45 行**注释内** `</script>` 同根因（`testParserInsertedDidNotRun` 定义于注释
> 后未注册，页面首内联 ReferenceError 短路，456 子测不可见）；③write 面族三页
> 不在本轮（096/142/143）。
> **本轮两刀**（单文件：tests/wpt-runner/src/testharness.rs）：①分类器
> `script_sched_unit`（预处理时刻冻结——spec prepare 定格，运行期 removeAttribute
> 不重分类；nomodule/nonce/module 维持原路径）+ 页面侧调度器
> `__zwSchedRegister`：async=trickle 后 setTimeout；defer 入 deferred 队列，
> `__zwSchedArmDrain` 于 interactive 后 DCL 前统一 drain（文档序）；block=trickle
> 后执行（解析阻塞近似，判据页只断言 log 序可观察面等价）。接线：drain 臂装在
> chain task1 **内部**（setTimeout FIFO 保 interactive 面先跑，103 断言依赖此序），
> DCL 派发幂等化改 drain 回调，load 门加 `__zwSchedPending()`；②投递体内
> `</script` → `<\/script` 转义（字符串/正则/注释 `\`+`/` 等值，
> any_js_window_wrapper 同口径）。
> **中间态回归（一乌龙）**：首轮终扫 load-error-events-1 四子测 Pass→Timeout——
> 调度单元 load/error 投递尾读 `document.currentScript`，相位执行时已清（getter
> 返 null，part06 R3258）事件永不派发。修正：注册语句同步原位执行时捕获
> `_zwEl` 传入闭包，投递尾 `script_event_dispatch_tail_el`。体内 currentScript
> 读取仍 null（挂账）。
> **验证**：runner bin **238/0**（236 基线 + s112 分类/转义两钉）。execution-timing
> 定向 22F→13F 九页转绿零 P→非P。全子树终扫 **638P/367F/37T+15N**（+240 行）：
> 十一页 Fail→Pass（上述九页 + script-defer——defer 不再抢跑 t.step 前断言 +
> delay-load-event-1——trickle(d3) 不再原位直执）；script-type-and-language-js
> 解锁 456 子测全注册（229P/227F，页保持 Fail——227 为先前不可见的真实
> type/language 分类缺口，候选独立切片）。归一 diff（/tmp/s112-final-scripting-1.log
> 存档）除上述外零变化。fmt + clippy（-p zero-wpt-runner --all-targets -D warnings）
> 过——as_deref 误加被 needless_option_as_deref 抓回（src 本就是 `Option<&str>`
> Copy）。渲染面零依赖——reftest/product-smoke 未跑。账本见
> [evidence/2026-10-27-m2s112-parser-sched-and-body-escape.md](html-semantics-compat/evidence/2026-10-27-m2s112-parser-sched-and-body-escape.md)。
> **范围**：仅 runner 投递/生命周期面；engine/webview/语料零变更；语料不可改维持。
> **挂账**：调度单元体内 currentScript 读取（需 shim idx 通道，遇实证再立项）；
> block 类不真暂停解析（050 等价仅判据面）；script-type-and-language-js 227 真实
> 缺口；096/142/143（write 面族原挂账）；063 Timeout；non-external-no-import
> （module async）；承袭：parse-error 族、http-accept-header、referrer-policies、
> dynamic import() with 透传、classic 外链 SRI、collect 账本打包重构、接口对象
> 基建面、FIXME 组、/tmp/zw-hb*.txt 插桩来源待查；**待决（须用户拍板）**：css/text
> 数据模块语料资源缺失。
> **下一步**：s113 入场定刀位（候选：script-type-and-language-js type/language
> 分类缺口 / 142 write 臂 xlink:href 单点 / 143 eventOrder 核验 / 096 write(defer)）。
> **切片间隙**：本轮 runner 投递面；下一候选同流，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 111 施工收口，原子插入语义轮——其下一步 s112 trickle 建模已由 s112 收口，见上）**：
> **入场定谳**：s111 承 147/148——fragment 内 3 工厂 inline script，#1 执行时
> `getElementById` 同批后继兄弟（147 移动/148 删除），修前报 `Cannot read
> properties of null (reading 'parentNode')`。四轮探针定谳双根因：①R387 直插臂无
> connected 门（s96 注释「连接态判定归调用方」，调用方从未补）——`frag.appendChild`
> 期 detached eval + 置 run-once，容器插入时被记账挡掉（npReg trace 实锤 gEBI 夹在
> fragment 内部追加与 s2 追加之间）；②s109 addedNodes 臂挂 notify 头部先于
> `_zwNodeParent` 链接记账——后继兄弟反链未指向真实插入目标，`_zwMutationInDoc`
> 上行失败 getElementById miss。spec 模型（spec-scripting.html 提取）：post-connection
> steps 只在插入元素仍连接时运行（HTML spec 自带示例与 148 同构）；「atomically
> inserting nodes」先完成全部 generic insert 再按文档序逐 script prepare；insert
> 遍历 inclusive descendants。
> **本轮三刀**：①part01 臂移位——addedNodes script 臂移至链接记账之后（批内前置
> 脚本执行时后继反链已登记）+ 非 SCRIPT 元素子树下降（文档序递归，129 形 div 内
> script）；②part03 `_zwScriptPrepConnected` 统一连接判定（proxy isConnected 布尔
> 直用；plain/盖章 plain 子——R368 只贴 `__zwHandle` 无 get trap——属性 undefined
> 回落身份反链 + parentNode 链爬升）；③part04 R387/R387b 三臂前置 connected 门，
> 未连接不置 run-once（重插入重走门）。
> **调试定谳（本轮两乌龙）**：①链式合并改分支语义——R387 原为「独立 if + if/else
> 对」三平铺语句，接成单链后 R377 回落臂命中吞掉外链 else 臂；平铺语句合并
> else-if 链前必须核对条件互斥性。②臂移位隐能化——旧位臂先于链接记账，maybe-start
> connected 门对刚插入节点恒 bail，外链 prepare 实际全靠 R387b；移位后 notify 臂
> 首次触达插入态外链，helper 只认 `__zwHostOwnsDynamicScripts` 不认 R387b 的
> `__zwHostDynamicScripts` → engine `r387b2` 钉抓到双执行（fetchCalls "2"≠"1"），
> 外链分支补同款让位（不置记账）后转绿。移位使原本「意外失活」的路径生效时，
> 排查该路径上所有让位契约。
> **验证**：engine lib **3548/3548**（r387b2 回归钉）；webview lib **814/814**；
> runner bin **236/0**（s109/s110 pin 保持）。execution-timing 定向扫页集合 vs
> s110 基线零 P→非P（117-119/129 首轮回归经子树下降+helper 修复回绿）。全子树
> 终扫 **398P/152F/37T**（+3 行/3 页），归一 diff（/tmp/s111-scripting-1b.log 存档）
> 恰为 049/147/148 三页 Fail→Pass。本轮零 .rs 变更（shim 全 JS + 文档），
> rustfmt/clippy 按实际影响跳过。渲染面零依赖——reftest/product-smoke 未跑。账本见
> [evidence/2026-10-27-m2s111-atomic-insert-and-connected-gate.md](html-semantics-compat/evidence/2026-10-27-m2s111-atomic-insert-and-connected-gate.md)。
> **范围**：engine shim part01/part03/part04；webview/语料/依赖零变更；语料不可改
> 维持（zz111/zz111b 探针页已删）。
> **挂账**：050/085/086/088/111/112（slow-loading 外链时序 pipe=trickle 建模
> ~11 行）；142 write 臂 xlink:href；143 eventOrder 完整核验；063 Timeout；
> non-external-no-import（module async）；**待决（须用户拍板）**：css/text 数据
> 模块语料资源缺失；承袭：parse-error 族、http-accept-header、referrer-policies、
> dynamic import() with 透传、classic 外链 SRI、collect 账本打包重构、接口对象
> 基建面、FIXME 组、/tmp/zw-hb*.txt 插桩来源待查。
> execution-timing 049/147/148 本轮销账（s110 销账面不变）。
> **下一步**：s112 入场定刀位（候选：050/085/086/088/111/112 trickle 建模 /
> 142 write 臂 xlink:href 单点 / 143 eventOrder 核验）。
> **切片间隙**：本轮 engine shim 面；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 108 施工收口，strict 首错聚合 + 短路收窄 + id 保真轮——其下一步 s109 入场探针定刀位已由 s109 收口，见上）**：
> **入场定谳**：s108 探针承 s107 三候选——候选 A（css/text 数据模块合成器）**证伪**：
> wpt-data（独立 repo，pinned tag）缺 css-module/resources/basic.css 等五个 CSS 资源
> 与 text-module/file.txt，fetch 恒 404，切片不可达；补语料属基准变更（guidelines
> #31 语料红线）记待决清单，s107「~30 行」估计据此修正。execution-timing 余 46F+1T
> 按根因分组定刀：①122-126 族 getElementById null（runner 装配丢 id）；④143 语料
> 故意 SyntaxError 被 runner 升级页级 Fail；②mutation 重触发机制面（②七行留 s109）、
> ③142 write 臂 xlink:href 不本轮。
> **本轮三刀**：①runner 投递 id 保真（`inline_local_scripts` 投递标签保留原标签 id，
> 与 nonce/nomodule/type/onload/onerror 同族「内联投递丢属性改语义」保真——122-126 族
> 以 getElementById("test") 取回 type=text/plain 外链单元做 mutation 重触发）；②runner
> 短路收窄（脚本抛错臂读 declared：==0 维持短路 Fail 防假绿、>0 放行轮询收集——WPT
> 语义 uncaught exception 报 window.onerror 不降级 test status；143 抛错时刻 pending=1
> 属正常中间态，故判据用 declared>0 非 pending==0）；③webview strict abort→首错聚合
> （两处 return Err 改 get_or_insert_with + continue——真浏览器语义 script 元素相互
> 独立；循环末聚合返 Err，Err 仅承载信息供 runner 防假绿判定）。**配套重构**：收尾段
> 单一出口——旧 `!pending` 分支在聚合判定前 `return Ok` 早退，无 DOM 变更的纯断言页
> （execution-timing 主体形态）首错被静默吞掉（新钉首轮红灯根因）；改 if/else 汇合后
> 统一 drain + 聚合 + 返回。
> **验证**：zero-webview lib **814/814**（+1 钉：三 script 页 is_err 且前后脚本照跑）；
> zero-wpt-runner bin **233/0**（+3 钉：投递 id 保留 / 抛错+declared>0 照收 Pass /
> 抛错+declared==0 维持短路——stateful MINI_HARNESS 变体带 tests 注册表与
> test_environment stub）；fmt clean + clippy webview/wpt-runner `-D warnings` clean。
> 定向 FILTER=execution-timing：非 P 页 **45 → 41**（**4 页销账**：067 语法错
> has-run——刀2+刀2b，语料故意埋错 + allow_uncaught_exception；104/114/115 defer
> mutation 族——刀1，三页均 getElementById("defer-script")）；122 null 崩溃消除进第二
> 层（页级 Fail→2 子测试断言级 Fail：async IDL 属性 + mutation 重触发，挂 s109）；
> 143 崩溃形→断言级 Fail（log 顺序断言真正执行，缺 inline script #3——write 嵌套
> 内联，挂 s109）。全子树终扫（FILTER=scripting-1 口径）**377P/173F/37T+15N**
> （s107：372P/175F/35T+15N，**+5P**），行级归一 diff（/tmp/s108-final-scripting-1.log
> 存档，基线 /tmp/s107-final-scan.log）**零 P→非P**（无 Pass 行消失）：4 Fail→Pass +
> 1 新增子测试 Pass（evaluation-order-1 module 抛错臂——刀2 直接战果）；Fail −2 净额
> 逐项核账；Timeout +2（evaluation-order-1/delay-load-event-1 页级落 deadline——均原
> Fail 页，declared>0 放行后未完成挂 deadline 的预测形）；microtasks/moving-between-
> documents 三处形态重排全为原 Fail 页短路→子测试级分解。渲染面零依赖——
> reftest/product-smoke 未跑。账本见
> [evidence/2026-10-27-m2s108-strict-continue-and-id-fidelity.md](html-semantics-compat/evidence/2026-10-27-m2s108-strict-continue-and-id-fidelity.md)。
> **范围**：webview webview.rs ×1（首错聚合 + 单一出口 + 1 钉）+ runner testharness
> ×1（id 保真 + declared 收窄 + 3 钉）；解析器/语料/依赖零变更；语料不可改维持。
> **挂账**：s109 主候选 mutation 重触发机制面七行（122 第二层 async IDL+时序、
> 123/124/125/126、144/145——嵌套执行 + 重扫 diff，共享主循环高风险面，独立切片细
> 设计）；142 write 臂 xlink:href；143 eventOrder 完整核验；063 Timeout（runner 脚本
> 记账）；slow-loading 外链时序 ~11 行（pipe=trickle 建模）；**待决（须用户拍板）**：
> css/text 数据模块语料资源缺失（五 CSS + file.txt 不在 pinned tag）——补语料属基准
> 变更；承袭：parse-error 族、http-accept-header、referrer-policies、dynamic import()
> with 透传、classic 外链 SRI、collect 账本打包重构、接口对象基建面、FIXME 组、
> /tmp/zw-hb*.txt 插桩来源待查。execution-timing 067/104/114/115 本轮销账。
> **下一步**：s109 入场定刀位（候选：mutation 重触发机制面七行体量评估——共享主循环
> 高风险面 / 142 write 臂 xlink:href 单点 / 143 eventOrder 核验搭车）。
> **切片间隙**：本轮 webview 执行面 + runner 收集/投递面 + 单测；下一挂账同流同面，
> 间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 107 施工收口，JSON module 执行面 + 顶层模块 SRI 轮——其下一步 s108 入场探针定刀位已由 s108 收口，见上）**：
> **入场定谳**：三候选——module 域 92 行最大、数据模块域合计 47 行（css 21 + json
> 17 + text 9）、execution-timing 余 46 行 scheduler 长尾异质。JSON module 定最小
> 可收口切片。MIME 合法集按**语料判据**定谳（不凭现行 spec 印象）：essence
> application/json | text/json | +json 后缀、`;参数` 剥离——valid-content-type 8 臂
> 全应成功（现行 spec 已收窄至 application/json + +json 后缀，text/json 是 corpus
> 时代合法集）；`serve-with-content-type.py` 端点本体不在 runner 语料——fetcher 按
> handler 语义模拟 `?fn=&ct=`，与 imported_script.py 模拟同模式。风险面定谳两处：
> ①integrity.html mismatch 行平凡 Pass（fetch 恒败→体不执行）——JSON 放行后必翻
> Fail，**须真实现 SRI** 才能保零 P→非P；②script-element-json-src 负例 src 是
> file.txt 非 .json，解除。
> **本轮四刀**：①fetcher JSON 意图分支（.json 静态文件默认 application/json +
> 端点 fn+ct 模拟，mime_is_json 门：合法交付原文/非法 Err 网络错误语义；ct 百分号
> 解码、裸 `+` 保留——`text/html+json` 裸 + 在 query 不表示空格）；②script-sandbox
> 新 `extract_static_module_imports_with_attributes`（复用 s105
> suffix_after_specifier_literal/parse_with_clause 提 pub(crate)，仅 import 语句，
> export-from/动态 import() 不提取）；③webview collect 链增 type_attr 参数（9 形参
> 函数级 allow + FIXME 挂账），type=json → serde_json 校验（非法→DepFetch/TopFetch
> 网络错误语义）+ 规范化内嵌合成 `export default <字面量>;`（default 导入经既有
> export transform 与导出扫描自动成立）；④顶层模块 SRI：`PageScript::ExternalModule
> { src, integrity }` 结构化（六消费点模式同步）+ webview
> integrity_metadata_matches（空格分隔多 hash 任一命中、algo 大小写不敏感、
> sha256/384/512，base64_encode 启用 + sha2 workspace dep）+ mismatch→元素 fire
> error + 体不执行。附刀：**外链模块成功 load 面**——模块成功后按 src 匹配 fire
> load（此前仅失败面有事件派发），**内联成功不派**（load-error-events-inline 四臂
> assert_unreached 判据；首版内联也派致 2 行 P→F，定向轮即定位改外链限定）；
> runner inline_local_scripts 对带 integrity 属性单元整段透传不内联（内联投递丢
> src/integrity 双锚——SRI 校验与事件 src 匹配都以 src 为锚）。
> **验证**：script-sandbox **373/373**（+带属性提取钉）；engine lib **3541/3541**
> （+integrity 提取钉）；webview lib **813/813**（+5 钉：JSON default 导入执行+
> 内联成功无 load / JSON 语法失败元素 error / SRI 命中执行+load / SRI 失配
> error+体不执行 / 承袭 s105）；wpt-runner fetcher 钉（.json 默认 MIME/端点
> fn+ct/解码/非 JSON 拒/sidecar 覆盖拒）；定向 **FILTER=json-module 26→30 行，
> 9F+2T→13P 零 P→非P**；load-error-events 滤页对照零回归（module/ 族行不变）；
> 全域交叉 integrity 四页（module 1P+2F→3P、json-module 2P、css/text mismatch 行
> 维持 Pass）；全子树终扫（FILTER=scripting-1 口径）**372P/175F/35T+15N**
> （s106：356P/185F/37T+15N，**+16P**），行级归一 diff（/tmp/s107-final-scan.log
> 存档）10 Fail + 2 Timeout → Pass 全正向、**零 P→非P**；fmt clean + clippy 六
> crate clean（renderer/browser/wpt-runner 模式改形波及）。渲染面零依赖——
> reftest/product-smoke 未跑。账本见
> [evidence/2026-10-27-m2s107-json-module-sri.md](html-semantics-compat/evidence/2026-10-27-m2s107-json-module-sri.md)。
> **范围**：script-sandbox es_module/import_attributes ×2 + engine extract ×1 +
> webview webview.rs/Cargo.toml ×2（sha2 dep）+ runner testharness ×1 + 六消费点
> 模式改形（browser/renderer/reftest 无行为变化）；解析器/语料零变更；语料不可改
> 维持。
> **挂账**：css/text 数据模块执行面（同一 collect 分派骨架已就位，type=css/text
> 合成器另切片，~30 行）；load-error-events.py 端点 module 路径 3 行；parse-error
> 族 Timeout（parse 失败元素事件模型与 s102 报告面差异）；http-accept-header
> （Accept 头建模 + 动态 import with 子句）；script-element-json-src（顶模块 MIME
> 拒绝→元素 error 事件模型与 s102 冲突待梳理）；referrer-policies；dynamic
> import() with 子句透传（现被丢弃）；export-from with 属性；classic 外链 SRI；
> collect_module_deps_recursive 账本打包重构（FIXME allow 承接）；hash 列表
> options；SVG script 族运行时面 4 页、execution-timing 余 ~42F、charset 族残留、
> svg-script-fragment-clone 3F、FIXME 组、接口对象基建面、/tmp/zw-hb*.txt 插桩
> 来源待查（承袭）。json-module 13 行 + module/integrity 2 行本轮销账。
> **下一步**：s108 入场探针定刀位（候选：css/text 数据模块合成器——本轮分派骨架
> 就位，css-module/text-module ~30 行 / execution-timing 余 ~42F scheduler 语义
> 定刀 / SVG script 运行时面 4 页体量评估）。
> **切片间隙**：本轮 script 管线面（fetch/编译/事件派发）+ 单测；下一挂账同流
> 同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 106 施工收口，SVG script 族静态面轮——其下一步 s107 入场探针定刀位已由 s107 收口，见上）**：
> **入场定谳**：三候选按 fail 形状重新分组——execution-timing 50F 中藏 **SVG script
> 族 8 页**（137/138/139/140/142/143/144/145），138/139/143 为 harness 级失败
> （`page script threw: SyntaxError: Unexpected token '.'`，143 的 `':'` 正是语料
> 故意埋的 `assert_unreached():`），共同根因优先于 scheduler 异质长尾，定刀。
> **根因（探针实证）**：①html5ever 对 foreign content 按 spec **markup 解析**——
> SVG script 内嵌套 `<script>` 为真实子元素（探针 dump 树形）；②提取器取源误用递归
> `text_content`（textContent 语义含后代元素文本），外层源吞入嵌套脚本文本。spec：
> 脚本源文本 = **child text content**（直接子文本节点拼接）；HTML script raw text
> 仅文本子节点，两取法恒等（其余 585 行零影响）。判据：138 期望 [ #1, #2, end#2]
> **end-tag 序**（子脚本先闭合先执行）；139 期望 [external, #1, end#1]；137 期望
> 空 href 不执行体。
> **本轮三刀**：①zero-dom 新 `child_text_content`（直接子 Text 拼接，与递归
> text_content 并列）；②extract 源文本改 child_text_content + 执行序按「pre-order
> 先行者是后行者祖先 → 后行者先行」重排（区间序；this_idx 保持全体 script
> pre-order 序与 shim getElementsByTagName 对齐，执行序与序号解耦）；③SVG 命名空间
> script href 空串跳过不落内联臂（HTML script src="" 维持旧回落不扩大）。族内余页
> 运行时面不触：142 write 注入外链 / 143 write 双 `</script>` 提前闭合外层（语料
> 故意双写）/ 144/145 mutation 改 textContent+type 重触发 prepare。
> **验证**：zero-dom lib **868/868**；engine lib **3540/3540**（+4 钉：嵌套源+序+
> 序号 / 外链嵌套序 / 空 href 双面 / 无嵌套序不变）；webview lib **809/809**；
> 定向 **FILTER=execution-timing 与 s105 基线 diff 恰 4 行（全 F→P）**；全子树终扫
> **356P/185F/37T + 15 NotRun**（s105：352P/189F/37T+15N），行级 diff **恰 4 行**、
> **零 P→非P**（comm 校验）、T/N 持平零旁损；fmt clean + clippy 三 crate clean。
> 诊断路径留档：fail 形状分组 → harness 级三页语料核对 → 探针 dump 双取法源文本
> 实证（探针已删）。渲染面零依赖——reftest/product-smoke 未跑。账本见
> [evidence/2026-10-27-m2s106-svg-script-family-static-face.md](html-semantics-compat/evidence/2026-10-27-m2s106-svg-script-family-static-face.md)。
> **范围**：document/mod.rs ×1（新访问器）+ extract.rs ×1（三刀 + 4 钉）；解析器/
> 语料/runner 零变更；语料不可改维持。
> **挂账**：SVG script 族运行时面 4 页（142/143/144/145，script 生命周期与 DOM
> 变更联动，另立项）；execution-timing 余 ~42F（scheduler 语义长尾 + 063 runner
> 脚本记账）；数据模块全量支持（fetcher MIME 门 + extractor 类型捕获 + registry
> 合成源 + rewriter）；.py 执行器前置；import attributes 余项（dep 级校验/
> export-from/动态 with）；charset 族（Timeout 形）；svg-script-fragment-clone 3F；
> FIXME 组；接口对象基建面（承袭）；基线扫描宜空载（承袭）；testharness.rs
> /tmp/zw-hb*.txt 插桩来源待查（承袭）。execution-timing 137/138/139/140 本轮销账。
> **下一步**：s107 入场探针定刀位（候选：execution-timing 余 ~42F scheduler 语义
> 定刀 / 数据模块全量支持或 .py 执行器前置体量评估 / SVG script 运行时面 4 页体量
> 评估）。
> **切片间隙**：本轮 zero-dom/engine 面 + 单测；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 105 施工收口，import attributes 静态校验轮——其下一步 s106 入场探针定刀位已由 s106 收口，见上）**：
> **入场定谳**：三候选评估——execution-timing 50F 异质（scheduler 排序长尾 + SVG
> integration-point 解析器面 + 138/139/143 疑似变换自伤）非单根因切片，否；063 =
> runner 脚本记账深坑，否；**import attributes 静态校验 3 Fail 页**最小高信定刀。
> 判据（语料原文核对）：invalid-type-attribute-error（4 外链模块 type 值
> notARealType/""/js/javascript → ev.error.constructor 全 TypeError，
> onerror=unreachable 即元素须 fire load 不 fire error）；unsupported-attribute
> （未知 key → SyntaxError）；invalid-import-errors-order（3 内联模块全
> SyntaxError——属性键错误先于同句/后句 specifier 解析错误与 type 值错误）。
> type 值合法性 = HTML module type map 不注册 "js"/"javascript"（JS 模块以无属性
> 为默认形态），语料四值判据一致；合法面 {"json","css","text"}。
> **本轮一刀**：新 script-sandbox `import_attributes` 模块
> `validate_static_import_attributes`——逐静态 import 语句（复用 es_module 5 扫描
> 件提 pub(crate)）定位 specifier 字面量后探测 `with {…}` 子句：任一属性键 ≠ type
> → SyntaxError（全量先于 type 值校验）；type 值 ∉ map → TypeError；块未闭合/
> 值非字符串字面量 → SyntaxError。webview is_module 臂图收集前接线：违例走 s102
> 报告面（error_ctor 分派 + 元素 fire load + continue 体不执行含 prelude，s103
> 同口径）——校验先于图收集保证属性键 SyntaxError 胜过裸 specifier 的
> DepResolution TypeError。合法属性零行为变化（仍走图收集，数据模块 MIME 面维持
> 现状）。放新文件不进 es_module.rs（后者已 3470 行超 2000 行准则）。
> **验证**：script-sandbox lib **372/372**（+7）；webview lib **809/809**（+2：非法
> type 值 TypeError 钉 + 未知 key 胜 bare specifier 序钉）；定向 **FILTER=
> import-attributes 4/4 全 Pass**（3 目标页 F→P + empty-attributes-clause 保持
> P）；全子树终扫 **352P/189F/37T + 15 NotRun**（s104：349P/192F/37T+15N），行级
> diff **恰 3 行**（3 目标页 F→P）、**零 P→非P**（comm 校验）、T/N 持平零旁损；
> fmt clean + clippy 双 crate clean。**扰动实录**：首版 map 漏 "text"（语料
> text-module 族 32 处 `type: "text"`）一轮全扫 text-module 5 页翻转（含
> integrity/load-error-events P→非P）——按语料属性值分布统计收录后
> FILTER=text-module 与基线零 diff 愈合。教训：module type map 合法面以全语料
> 属性值分布定界，不凭 spec 印象。**测试坑**：回归钉首版 `import BAD with {…}`
> （default binding 无 from 子句——非法 JS，import_clause_shape_ok 正确拒绝致无
> 报告），改裸字符串 `import "BAD" with {…}`（corpus 同形）后过。渲染面零依赖
> （JS 静态校验 + webview 编译臂）——reftest/product-smoke 未跑。账本见
> [evidence/2026-10-27-m2s105-import-attributes-static-validation.md](html-semantics-compat/evidence/2026-10-27-m2s105-import-attributes-static-validation.md)。
> **范围**：import_attributes.rs 新模块 + es_module.rs（5 扫描件 pub(crate)) +
> lib.rs（注册）+ webview.rs（接线 + 2 单测）；解析器/语料/runner 零变更；语料
> 不可改维持。
> **挂账**：import attributes 余项——数据模块全量支持（fetcher MIME 门
> testharness.rs:4904-4964 + extractor 类型捕获 + registry 合成源 + rewriter，
> 跨 crate 大切片）、`export … from` 属性子句与动态 import() with 选项、dep 级
> 属性校验；execution-timing 族 **50F**（scheduler 语义，另立项——063 为其中
> 一页）；.py 执行器 / 非 Error 抛出值 ev.error=null 身份 / 动态 import reject
> 形 / error-type-3 instantiation 优先级；charset 族（Timeout 形）；
> svg-script-fragment-clone 3F；FIXME 组（bufTop 漂移/setAttribute('src')/type
> 门/children-changed 其余入口）；接口对象基建面（承袭）；基线扫描宜空载（承袭）；
> testharness.rs /tmp/zw-hb*.txt 插桩两行来源待查（承袭）。import-attributes 3 页
> 本轮销账。
> **下一步**：s106 入场探针定刀位（候选：execution-timing scheduler 族定刀 /
> 063 Timeout 深探 / 数据模块全量支持或 .py 执行器前置体量评估）。
> **切片间隙**：本轮 script-sandbox/webview 面 + 单测；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 104 施工收口，frameset 文档 document.body 语义轮——其下一步 s105 入场探针定刀位已由 s105 收口，见上）**：
> **入场定谳**：承袭「116 frameset 文档语义缺口」（s97 记账）——execution-timing/116
> 1F：`assert_array_equals` 期望 `['document.body: <FRAMESET>']` 实得 `<BODY>`。判据
> 机制：head 内动态插入外链 find-body.js，文档解析完后记录 `document.body.localName`
> ——frameset 文档（head 后 `<frameset>` 无 body）spec 语义 body element = html 元素
> 第一个 body **或 frameset** 子元素。根因：解析树（html5ever spec 树构造）frameset
> 文档无 body（Rust DOM 正确），shim `document.body: _wrapSelector('body')`（part06
> :4900）静态 selector proxy 恒报幽灵 BODY。**候选对比**：063（cloneNode deep——
> clone 插入触发 body 内联脚本重复执行 + pending=1 挂起，runner 脚本记账深坑）与
> module 尾片（import attributes/.py 执行器基建体量）均大；116 单 getter 分派定刀。
> **本轮一刀**：part06 `document.body` 改 getter——`__zw_query_match('body')` 未命中
> 且 `'frameset'` 命中 → frameset proxy；否则原 body proxy（普通文档 identity/行为
> 零变化——proxy 按选择器缓存跨访问同对象；均无保持 body proxy 旧兜底，不新增
> null 分派）。webview +1 单测（frameset 文档 localName==='frameset' / 普通文档
> 'body' / identity true）。
> **验证**：webview lib **807/807**（+1）；engine lib **3533/3533**；定向
> **FILTER=116 1F→1P**；全子树终扫 **349P/192F/37T + 15 NotRun**（s103：348P/193F/
> 37T+15N），行级 diff **恰 1 行**（116 F→P）、**零 P→非P**（comm 校验）零旁损；
> fmt clean + clippy 双 crate clean。**探针取证**：①裸 execute_script 走轻量
> bootstrap（`__zw_query_match` 未定义）——shim 语义取证必须走页面脚本路径；②探针
> 脚本置 `</html>` 后 → 解析器建隐式 body，document.body 合法地是 body——判据文档
> 形状须与 116 同形（脚本在 head 内）。渲染面零依赖（reftest 自有 onload 提取
> tag 无关、layout 读 Rust DOM 不经 shim）——reftest/product-smoke 未跑。账本见
> [evidence/2026-10-27-m2s104-frameset-document-body.md](html-semantics-compat/evidence/2026-10-27-m2s104-frameset-document-body.md)。
> **范围**：part06.js ×1 + webview.rs ×1（1 单测）；解析器/语料/runner 零变更；语
> 料不可改维持。
> **挂账**：execution-timing 族 **50F**（scheduler 语义，另立项——063 为其中一页：
> cloneNode deep + runner 脚本记账重复执行）；module 尾片余项（import attributes /
> .py 执行器 / 非 Error 抛出值 ev.error=null 身份 / 动态 import reject 形 /
> error-type-3 instantiation 优先级）；charset 族（Timeout 形）；
> svg-script-fragment-clone 3F；FIXME 组（bufTop 漂移/setAttribute('src')/type 门/
> children-changed 其余入口）；接口对象基建面（承袭）；基线扫描宜空载（承袭）；
> testharness.rs /tmp/zw-hb*.txt 插桩两行来源待查（承袭）。116 本轮销账。
> **下一步**：s105 入场探针定刀位（候选：module 尾片体量评估（import attributes
> 或 .py 执行器前置）/ execution-timing scheduler 族定刀 / 063 Timeout 深探）。
> **切片间隙**：本轮 engine .js shim 面 + webview 单测；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 103 施工收口，module graph 取回/解析失败面轮——其下一步 s104 入场探针定刀位已由 s104 收口，见上）**：
> **入场定谳**：s102 残余①——dep 级取回失败空存根兜底：图内任一 dep fetch 失败
> 旧版 `registry.register(spec, "")` 后模块体**照常执行**，网络级失败面丢失。spec
> 语义（fetch-a-single-module-script / resolve-a-module-specifier）：图内取回失败
> → 元素 fire **error** + 体不执行 + 无 window 报告；图内裸 specifier 解析失败 →
> window 报 **TypeError** + 元素照常 fire **load** + 体不执行。判据页定谳：
> error-type-1 双模块（第一 syntaxerror.js parse→window SyntaxError+onload，s102
> 已达；第二 error-type-1.js 图内 404 → onerror、无 window 报告，log.length==3）、
> specifier-error（裸 specifier → ev.error=TypeError + log.length==1 + 元素不
> fire error）。关键事实：prelude 急切求值非入口注册模块（R5182）——体不执行必须
> 连 prelude 一并跳过，否则已取回依赖经 prelude 二次报错且误触 onload unreachable。
> **本轮三刀**：①engine `script_report_module_failure` 加 `error_ctor` 形参
> （编译/求值面 "SyntaxError"、解析面 "TypeError"）+ 新
> `script_dispatch_script_event_by_ordinal`（无 src 内联模块元素级派发）。②webview
> fetcher 分支：收集 `is_dep=true`（此处 spec 均为被运行模块的依赖——旧 false 使裸
> specifier 误作相对 URL 取回）、首 Err 捕获 `(failure, spec)` break，按类分派——
> DepResolution → TypeError 报告面（元素 load）；TopFetch|DepFetch → 元素 error
> （外链 R2944 `module_cache_key`=src 绝对化匹配；内联 ordinal）后 `continue`（编
> 译+prelude 一并跳过）；无 fetcher 路径与 `compile_module_contract` 动态 import 契
> 约路径不变。③+2 单测（dep 404 → onerror+体 undefined+无 window error+继续；裸
> specifier → TypeError+onload+体 undefined+继续）。
> **验证**：webview lib **806/806**（+2）；engine lib **3533/3533**；定向
> **error-type-1 1F→1P**（log=[SyntaxError,1,2] 链闭合）+ **specifier-error
> 1F→1P**、error-type-2 保 P；全子树终扫 **348P/193F/37T + 15 NotRun**（s102：
> 342P/180F/41T+27N）；**P→非P 5 页逐页归因非契约回归**（全为 s102 存根假阳诚实
> 化）：css/json/text-module load-error-events 各 1 子测试（inline,200 臂——
> import attributes/非 JS module 缺口；同页 `src,200` ×2 两版同 Fail、404 臂全保
> P）、nested-imports（delayed-modulescript.py 端点）、json-module/parse-error
>（parse-error.json 不在语料——s102 Pass 系「空存根→export 检查失败→碰巧
> SyntaxError 面」双重假阳）；连带 F→T 1 页（parse-error-location.tentative，体未
> 注册）。**增益**：fetch-error-2 F→P、import-subgraph-404 T→P（s102 F→T 族翻回
> 真 Pass）、load-error-events-inline N→P×2、integrity ×3 T→Fail+Pass（404 臂真
> P）、content-type-checking N→F+P2、http-accept T→F×2、module.html/valid-content-
> type N→F×7；fmt clean + clippy 双 crate clean；runner testharness.rs 零变更；
> reftest/product-smoke 未跑（无 render/layout 面）。账本见
> [evidence/2026-10-27-m2s103-module-graph-failure-face.md](html-semantics-compat/evidence/2026-10-27-m2s103-module-graph-failure-face.md)。
> **范围**：webview.rs ×1（fetcher 分支失败分派 + 2 单测）+ script_gen.rs ×1
>（error_ctor 形参 + ordinal 派发 helper）；shim/语料零变更；语料不可改维持。
> **挂账**：module 尾片（.py 执行器——nested-imports/integrity 族 load 期望臂、
> http-accept-header；import attributes + CSS/JSON/text module——load-error-events
> 200 臂、valid-content-type、content-type-checking Fail 面、parse-error 族；非
> Error 抛出值 ev.error=null 身份（error-type-3）；动态 import 已错模块 reject 形
>（instantiation-error-1）；error-type-3 instantiation 优先级）；116 frameset 文
> 档语义缺口；063 Timeout 承袭；charset 族（Timeout 形）；svg-script-fragment-clone
> 3F；FIXME 组（bufTop 漂移/setAttribute('src')/type 门/children-changed 其余入口）；
> 接口对象基建面（承袭）；基线扫描宜空载（承袭）；testharness.rs /tmp/zw-hb*.txt
> 插桩两行来源待查（承袭）。目标 2 页 + 连带增益本轮销账。
> **下一步**：s104 入场探针定刀位（候选：116 frameset 文档语义缺口 / 063
> Timeout / module 尾片余项——import attributes 或 .py 执行器前置体量评估）。
> **切片间隙**：本轮 webview/engine .rs 面；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 102 施工收口，module 失败报告面轮——其下一步 s103 入场探针定刀位已由 s103 收口，见上）**：
> **入场定谳**：s101 终扫 73 行 `page script threw`，35 页 declared=0——module
> 编译/求值失败在 strict 下整页 abort，掩盖页面其余用例。spec 语义（run a module
> script）：fetch **成功**后的编译/链接/求值失败 → 向 window 报异常（window error
> 事件 ev.error=抛出值，parse 失败为 SyntaxError）+ 元素照常 fire **load** + 页
> 面继续；仅**网络级取回失败** → 元素 fire **error**。判据页：module/error-type-1
> （parse→window SyntaxError+onload；network→onerror）、instantiation-error-3/4/5。
> **本轮四刀**：①engine `script_report_module_failure(msg, ordinal)`——
> `reportError(new SyntaxError(msg))`（shim reportError part01 既有面，ev.error 携
> Error 实例）+ `getElementsByTagName('script')[ordinal]` 元素 load 派发（文档序
> script 序号 = extract_script_csp_info 同口径，适配无 src 内联模块）。②webview 编
> 译失败臂：strict abort → 报告面 + continue（strict 亦不 abort——prepare 阶段失败
> 面非内联体抛错）。③webview 执行期失败臂：`strict && !is_module` 才 abort；module
> 执行期异常（求值抛错/变换后代码 parse 失败——compile_module_script 变换为词法面
> 不做全量 parse，error-type-1 syntaxerror.js `%` 体过变换 sandbox parse 期炸实
> 证）→ 同①报告面 + 继续。④外链 classic/module 取回失败两臂：补
> `script_dispatch_script_event(&abs, "error")`（R2944——旧注释声称 spec error 面
> 但实现仅 warn 静默跳过）。
> **验证**：webview lib **804/804**（+2：编译失败 window SyntaxError+onload+继续
> 钉、外链取回失败 onerror+继续钉）；engine lib **3533/3533**（含他流并入用例）；
> 定向：error-type-2 **1F→1P**、error-type-1/3 abort→子测试级 Fail（断言真跑）、
> instantiation-error **3/4/5 三页 1F→1P**；全子树终扫 **342P/180F/41T + 27
> NotRun**（s101：326P/220F/14T+18N），行级 diff **F→P 12 页 + 零 P→非P**
>（comm 校验）；**F→T 27 页归因**（非回归）：原 declared=0 abort 族跑完后测试体
> 因 feature 缺失（import attributes `with{type:"css"}`、.py CGI 端点、referrer
> 语义）未注册 → runner「completed without reporting (tests:0)」映射 Timeout——
> 状态更保真（window error 已报、页面继续），翻回需尾片；fmt clean + clippy 双
> crate `-D warnings` clean；runner testharness.rs 零变更；reftest/product-smoke
> 未跑（无 render/layout 面）。账本见
> [evidence/2026-10-27-m2s102-module-failure-report-face.md](html-semantics-compat/evidence/2026-10-27-m2s102-module-failure-report-face.md)。
> **范围**：webview.rs ×1（两失败臂 narrow + 取回失败投递 + 2 单测）+ script_gen.rs
> ×1（helper）；shim/语料零变更；语料不可改维持。
> **挂账**：module 尾片（.py 执行器前置 + import attributes；本面新暴露残余——dep
> 级取回失败仍空存根兜底（error-type-1 的 404.js 臂需「图中任一取回失败 → 元素
> error」语义，动 M2-S4E 存根契约须独立切片全扫审计）、非 Error 抛出值 ev.error=null
> （error-type-3 对象身份）、bare specifier 运行时 TypeError（specifier-error）、
> 动态 import 已错模块 reject 形（TypeError vs SyntaxError））；116 frameset 文档
> 语义缺口；063 Timeout 承袭；charset 族（现 Timeout 形）；svg-script-fragment-clone
> 3F；FIXME 组（bufTop 漂移/setAttribute('src')/type 门/children-changed 其余入口）；
> 接口对象基建面（承袭）；基线扫描宜空载（承袭）。F→P 12 页 + 祛 Mask 8 页本轮
> 销账。附带发现：testharness.rs 既有已提交 `/tmp/zw-hb*.txt` 调试插桩两行（非本
> 会话所加，来源待查）。
> **下一步**：s103 入场探针定刀位（候选：module 尾片 dep 级取回失败元素 error 面
> （存根契约变更，独立切片）/ 116 frameset 文档语义缺口 / 063 Timeout）。
> **切片间隙**：本轮 webview/engine .rs 面；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 101 施工收口，外链模块 import 收集基轮——其下一步 s102 入场探针定刀位已由 s102 收口，见上）**：
> **入场定谳（两层根因）**：nomodule-set-on-external-module-script 1F 整页 abort
> （`Module ./cocoa-module.js does not provide an export named default`，declared
> tests: 0）。①webview——ExternalModule 静态 import 收集基 = 页面 URL；spec 语义
> import 相对 **requesting module 自身 URL** 解析（HostResolveImportedModule
> referrer 为模块自身），模块体在 resources/ 子目录时其相对 import 解析到页目录 →
> 取回 404 → 空存根 → 假编译错。②runner——inline_local_scripts 连外链 module 也
> 内联（s94 保 type），engine 走 InlineModule 臂，①修复不触发、原始 src 解析基准
> 整体丢失。**方案取舍**：A) 跳过 module 内联——parser module 臂 load/error 投递
> 依赖 s100 投递尾，外置即丢（module/load-error-events 2P 回退）被否；B) data 属性
> 透传 origin——跨 crate 形状变更；取 **E1) 内联投递时重写模块体相对 spec**（投递
> 形态适配器职责，同 .sub.js 替换先例）。
> **本轮三刀**：①webview `module_dep_base`——ExternalModule 臂收集基 =
> `resolve_document_url(doc_url, src)`（InlineModule 维持页面 URL）。②runner
> `rewrite_inline_module_import_specs`——仅 `./`/`../` 相对 spec 以模块自身
> wpt_root 相对目录为基归一 root-absolute（lexer 感知提取定位 + 引号包裹整体替换；
> 裸 specifier 维持 s87 解析失败语义、root-absolute/scheme 不动、查询串保留；动态
> import() 走运行时链归 module 尾片）。③module 图**钻石依赖拼写别名**——collect
> 循环防护（按解析后 URL）命中时按**当前拼写补注册**同一源（`fetched` URL→源缓
> 存）：registry 按导入方体内原文拼写查表，旧版仅注册首拼写 → 第二拼写 lookup
> 落空 compile 失败（imports-self-inner 自导入实证；旧文档明记「已知限制」——本
> 轮连带解除的共享路径缺陷，②重写使拼写分歧显形）。
> **验证**：webview lib **802/802**（+2：收集基钉 dep 仅模块相对位置可达红→绿 +
> 拼写别名钉）；runner bin **229/229**（+1：helper 相对/上溯/root-absolute/裸/
> scheme 四态 + 端到端 module 重写而 classic 体不动）；定向 FILTER=
> nomodule-set-on-external-module **1F→1P**（两轮——仅①仍同错钉死根因②、
> ①+②+③ 全绿）、FILTER=module/imports **5 Pass 0 Fail**（中途态 5P→1F 假红
> 由③归位）；全子树终扫 **326P/220F/14T + 18 NotRun**，diff vs s100 基线**恰
> 1 页 1 行**（本页 Fail→Pass）、**零 P→非P**；fmt clean + clippy 双 crate
> `-D warnings` clean（ulimit 兜底沿 s91 口径）；reftest/product-smoke 未跑
> （无 render/layout 面）。账本见
> [evidence/2026-10-27-m2s101-external-module-import-base.md](html-semantics-compat/evidence/2026-10-27-m2s101-external-module-import-base.md)。
> **范围**：webview.rs ×1（收集基 + fetched 缓存补注册 + 2 单测）+ testharness.rs
> ×1（重写 helper + 调用点 + 1 单测）；shim/语料零变更；语料不可改维持。
> **挂账**：module 尾片（.py 执行器前置；text-module/css-module/json-module
> load-error 变体页同族汇入；动态 import() 相对模块解析基同列）；module 编译错误
> error 事件面（compile 失败须 error 事件而非 strict 整页 abort——本页「must run」
> 臂已销账、失败臂仍挂）；116 frameset 文档语义缺口；063 Timeout 承袭；charset
> 12F（.py）；svg-script-fragment-clone 3F；FIXME 组（bufTop 漂移/setAttribute
> ('src')/type 门/children-changed 其余入口）；接口对象基建面（承袭）；基线扫描宜
> 空载（承袭）。nomodule-external-module 1F 本轮销账。附带发现：testharness.rs
> 既有已提交 `/tmp/zw-hb*.txt` 调试插桩两行（非本会话所加，来源待查）。
> **切片间隙**：本轮 runner/webview .rs 面；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-26 M2 切片 100 施工收口，经典脚本 load/error 事件投递保真轮；施工跨 429 断点，断点后终扫复跑与断点前扫描 bitwise 一致——其下一步 s101 入场探针定刀位已由 s101 收口，见上）**：
> **入场核账**：s99 挂账 load-event 投递保真族——load-error-events-1 11 臂（parser
> defer/sync/async × 200/404 + dynamic ×4 + no-src 内联臂）+ nomodule-sync
> 「must run」。两面根因：①投递面——parser 外链被 inline_local_scripts 内联改写丢
> onload/onerror（s94 保真只留 nonce/nomodule/type），且内联后无取回环节，spec
> 「成功执行 fire load / 取回失败 fire error」随之丢失；②服务面——load-error-events.py
> 为 wptserve CGI 端点，语料 pull 面不含 .py，dynamic 200 臂取回落通用缺文件 404 →
> error 路径误触发。
> **本轮三刀**：①投递保真——script_event_handler_attr 提取回写 on* 内容属性（`&`/`"`
> 转义同 nonce 先例）+ 投递尾（200 体后 fire load、非 200 只 fire error 且脚本体
> 不执行；document.currentScript.dispatchEvent——R2934 编译链触发 on* 与
> addEventListener 两路，事件不冒泡不扰 window load）；src 查询串先剥离再归一。
> ②fetch 通道 .py fixture 静态等价（wpt_load_error_events_response，fetch/内联共
> 用）：test 参数 `^[a-zA-Z0-9_]+$` 门（缺席/非法 400）、`*_load`→200 +
> `<test>.executed=true`、其余 404 + assert_unreached 体（status 拒 eval）；路径精
> 确匹配 classic 端点，module 族变体不匹配归 module 尾片。③extract
> attr_from_open_tag 值面从**原文**切片——旧版从 ASCII 小写镜像切值，
> `onLoad(t1)`→`onload(t1)` 失效、base64 nonce 同面受损；lower 镜像等字节长偏移通用。
> **验证**：runner bin **228/228**（+2 单测）；engine lib **3530/3530**；
> FILTER=load-error-events le1 **11/11 全 Pass**（页完成无 Timeout）；
> FILTER=nomodule nomodule-sync **1F→1P**；全子树终扫 **325P/221F/14T + 18
> NotRun**，diff vs s99 恰 3 页 F/T→P（le1 2F+1T→11P 全清 / module
> load-error-events 2F→2P——parser module 臂 on* 保真连带 / nomodule-sync
> 1F→1P）、**零 P→非P**（/tmp/n99.txt vs /tmp/n100s.txt）；execution-timing/122
> 缺行系 teardown 粒度方差（s93 先例，单跑仍报既有 Fail）；fmt clean + clippy 双
> crate `-D warnings` clean（ulimit 兜底沿 s91 口径）；reftest/product-smoke 未跑
> （无 render/layout 面）。账本见
> [evidence/2026-10-26-m2s100-load-event-delivery-fidelity.md](html-semantics-compat/evidence/2026-10-26-m2s100-load-event-delivery-fidelity.md)。
> **范围**：testharness.rs ×1（fetch arm + fixture + inline 保真/投递尾 + 2 单测）+
> extract.rs ×1（attr 值切片）；shim/语料零变更；语料不可改维持。
> **挂账**：module 尾片（.py 执行器前置；text-module/css-module/json-module
> load-error 变体页同族汇入）；module 编译错误事件面（nomodule-set-on-external-
> module-script 1F）；116 frameset 文档语义缺口；063 Timeout 承袭；charset 12F
> （.py）；svg-script-fragment-clone 3F；FIXME 组（bufTop 漂移/setAttribute('src')/
> type 门/children-changed 其余入口）；接口对象基建面（承袭）；基线扫描宜空载（承
> 袭）。load-error-events classic 全臂 + nomodule-sync 本轮销账。
> **切片间隙**：本轮纯 runner/engine .rs 面；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-26 M2 切片 99 施工收口，script 布尔 IDL has 可见性轮——其下一步 s100 入场探针定刀位已由 s100 收口，见上）**：
> **入场核账**：s98 下一步候选入场探针——nomodule-set-on-async-classic-script
> 两行**同报** `assert_true: expected true got false`，两测试首断言同为
> `assert_true(supportsNoModule)`（`"noModule" in script` 探针）；若 load 门
> （`waitForLoadEvent = new Promise(r => window.onload = r)`）失灵应报 Timeout
> 而非断言失败——门正常，缺口在 `in` 可见性。**根因**：元素 proxy has trap
> （part05 生效版，R129 定谳）白名单无 _REFLECTED_BOOL 面——get trap（part04
> R3038/R3040）值面覆盖全表、`in` 探测恒 false（「'id' in el 仍 false」既有
> documented 限制同类面）。**本轮接线**：has 白名单新增 R5300 分支，
> `async`/`defer`/`noModule` 三名 **tag 门 SCRIPT**（':' 后缀容差同 R5266）——
> 全员表列名重蹈 R5249 menuitem 假阳性先例（接口成员归属按 tag 门控）；三名恰
> 为 HTMLScriptElement 接口成员（spec：reflected boolean，
> https://html.spec.whatwg.org/multipage/scripting.html#dom-script-nomodule）。
> **验证**：FILTER=nomodule 本两行 **2F→2P**——测试 1（noModule=false）四断言
> 全过（动态 appendChild src 执行 + onload 投递 + load 门 resolve）；测试 2
> （noModule=true）!executed/!loaded/!errored 全过（s96 prepare 门阻塞执行且不
> 投递 load/error——动态臂阻塞面实测）；全子树终扫 **314P/227F/15T + 18
> NotRun**，净 +2 Pass、行级 diff 仅本两行、**零 P→非P**（/tmp/s99-s1-rows.txt）；
> engine lib **3530/3530**（guard）；shim 拼接 node --check OK；纯 shim .js 变更
> fmt/clippy/reftest/product-smoke 不涉（沿 s91-s98 口径）。账本见
> [evidence/2026-10-26-m2s99-script-bool-idl-has-visibility.md](html-semantics-compat/evidence/2026-10-26-m2s99-script-bool-idl-has-visibility.md)。
> **范围**：part05.js ×1（has trap 一分支）；runner/语料零变更；语料不可改维持。
> **挂账**：load-event 投递保真族——parser 外链被 inline_local_scripts 内联投递
> 时 `onload`/`onerror` 内容属性被丢弃（s94 保真只留 nonce/nomodule/type）→
> nomodule-set-on-synchronously「must run」1F（脚本执行到位、`assert_true(loaded)`
> 假红）+ load-error-events 族 7F/3T 同族汇入；module 编译错误事件面
> （nomodule-set-on-external-module-script——缺 default export 的编译错误冒出为
> page script threw，spec 须落 error 事件）；116 frameset 文档语义缺口；063
> Timeout 承袭；module 尾片（.py 前置）；charset 12F（.py）；
> svg-script-fragment-clone 3F；FIXME 组（bufTop 漂移 / setAttribute('src') /
> type 门 / children-changed 其余入口）；接口对象基建面（承袭）；基线扫描宜空载
> （承袭）。nomodule-async 2F 本轮翻绿销账。
> **切片间隙**：本轮纯 engine shim 面（has trap）；下一挂账 load-event 保真族跨
> runner（inline_local_scripts）+ engine（执行后投递）两面。
>
> **▶ 上轮状态（2026-10-26 M2 切片 98 施工收口，write 注入 harness runner 面接线轮——其下一步 s99 入场探针定刀位已由 s99 收口，见上）**：
> **入场核账**：s97 挂账定谳——102 页脚本全执行但 write 注入 testharness.js 经
> fetch 通道取回 RAW 副本（`hook=undefined/state=null` 超时转储）。**完成机制核
> 实**（pinned testharness.js）：`notify_result → all_done()（要求
> test_environment.all_loaded）→ complete() → completion 回调 → 探针全局
> __zw_harness_complete`。**静态/write 两路不对称**：静态标签被 prepare_harness_html
> 内联替换（补丁 replacen + reporter 装配 + 生命周期脚本尾补调钩子），102 初始
> 文档只有 write 串里的 src——补丁与 reporter 都到不了注入副本，且 `document.open`
> 换文档后 load 不重发，`all_loaded` 恒 false，末测 done 后永不 complete。**本轮
> 接线**（testharness.rs fetch 静态回退特判）：`/resources/testharness.js` 服务
> 补丁源（`apply_harness_completion_patch` 与静态内联共用 replacen）+ eval 尾自调
> `__zw_mark_harness_loaded()`（注入副本求值完成即置 all_loaded；此刻
> tests.length=0，all_done 为 false 无误完成）；`/resources/testharnessreport.js`
> 服务 reporter 装配快照（上游 report 裸 loader，注册的 completion 回调到不了探针
> 全局）；reporter 提 `HARNESS_REPORTER_SOURCE` 常量、静态/fetch 共用装配面。
> **影响面核查**：write/dynamic 注入两路径的页面全语料仅 102 形（静态标签一律内
> 联替换走不到 fetch 通道），blast radius 有界。**验证**：execution-timing s97
> 97P/51F/2T → **98P/51F/1T**（逐行 diff 仅 102）；全子树终扫 **312P/229F/15T +
> 18 NotRun**，净 +1 Pass、行级 diff 仅 102 Timeout→Pass、**零 P→非P**
> （/tmp/s98-s1-rows.txt）；102 结果行 `Pass :: scheduler: defer script after
> initial onload event`（写入全家桶依序执行、eventOrder 断言通过后 t.done() 完
> 成——非空转判绿）；runner bin 单测 **226/226**（guard）；纯 runner .rs 变更，
> fmt/clippy/reftest 不涉（沿 s91-s97 口径）。账本见
> [evidence/2026-10-26-m2s98-write-harness-runner-face.md](html-semantics-compat/evidence/2026-10-26-m2s98-write-harness-runner-face.md)。
> **范围**：testharness.rs ×1（fetch 静态服务特判 + 装配常量/公共函数提升）；
> engine shim 零变更；语料不可改维持。
> **挂账**：116 frameset 文档语义缺口；063 Timeout（hook=function pending=1）承
> 袭；module 尾片（.py 前置）；charset 12F（.py）；svg-script-fragment-clone 3F；
> nomodule-async 2F（window.onload promise 门）；load-error-events 族（脱离树副本
> 元素事件保真）；FIXME：写流中页面脚本结构变更 bufTop 漂移、嵌套 write 内含外链
> 近似文档序、setAttribute('src') 通路、type 门、children-changed 其余变异入口
> （append()/textContent=/.text 已连接面）；接口对象基建面（承袭）；基线扫描宜空
> 载（承袭）。102 本轮翻绿销账。
> **切片间隙**：本轮纯 runner 面（.rs）；下一挂账 nomodule-async/load-error-events
> 回 engine shim 面，跨面间隙。
>
> **▶ 上轮状态（2026-10-26 M2 切片 97 施工收口，document.write 写入脚本执行面轮——其下一步 s98 入场探针定刀位已由 s98 收口，见上）**：
> **入场核账**：s96 HEAD 基线 **288P/251F/17T + 18 NotRun**（scripting-1 口径
> 574 行）。s97 聚焦 document.write 族：spec #document-write-steps 把写入标记交
> 解析器插入点（内联同步、外链按文档序阻塞写流内其后脚本、嵌套写入落在当前求
> 值窗），本引擎回放走 innerHTML/insertAdjacentHTML（inert）——写入脚本须显式
> 执行面。**本轮两刀**：①part06 `__zwDocWriteRunFace`（write 入口惰性安装，显式
> open/close 流与隐式流共用）：缓冲尾段子树收集（bufTop = 现长 − 基点，4096 节点
> 预算）——基点取**首轮 innerHTML 往返后**口径（dbg 探针 `kids=8 base=9` 实证序列
> 化/重解析对空白文本节点有损耗：9→7，写前快照使 bufTop 落负、收集恒空）；内联
> 槽位 `__zwDocWriteDoneSlots` 防重；求值窗 `__zwDocWriteEvalSlot`（part04 prepare
> then 内设置/清除）——窗内嵌套 write 的面走即时 pass（只执行新增内联、src 槽位
> 留续流链、不动记账，009「external before/dw/after」序）；外链 src 槽位阻塞链
> （水位线占位停本槽位 + `__zwDocWritePendingSlot` 挂链 + plain wrapper 重建
> handle 域副本**全属性透传**（012 include-N onload）+ `_zwPrepareSrcScript(copy,
> cont)` 续流清链/推进/续跑）；close 流接管 runner 无 settle 环境（基点 0，全家桶
> 依序）；②part04 insertBefore 统一出口挂 R387 同口径门（连接态→nomodule→
> for/event→run-once→内联 eval / `_zwPrepareSrcScript`）——018 testlib.addScript
> firstInParent 形态。**主发现（水位线占位）**：src 槽位 prepare 前不占位，续流
> 回调条件推进（`wm === slot`）永不成立 → 重入重复 prepare fresh 副本 → 无限
> fetch（102b/102c 90s 执行超时实证）；修后插桩定谳 close 流全序
> `close-takeover→face→slot0 内联→slot1 src→prepare→then→cont→face(wm=2)→slot2
> 内联→face-exit:wm=3`，102b/102c 残留 Fail 系探针缺 testlib.js（`log` 未定义）
> 作者缺陷，非引擎回归。**验证**：execution-timing corpus-only s96 83P/73F/3T →
> **97P/51F/2T**；全子树终扫 **311P/229F/16T + 18 NotRun**，净 +23 Pass、
> **零 P→非P**（comm 校验空；终扫 /tmp/s97-scripting1-final.txt 定为下轮基线）；
> 033/041/068/096 仅断言消息面变化（Fail→Fail 更深断言）；102 Fail→Timeout
> （页脚本全执行，write 注入 harness 未接完成探测——runner 面挂账）；116
> Timeout→Fail（frameset 文档 `document.body` 报 BODY——既有引擎缺口显形）；
> engine lib **3530/3530**（guard）；shim 13 part 真实序拼接 node --check OK；
> 纯 .js 变更 fmt/clippy 不涉；reftest 未跑（无 render/layout 面，沿 s91-s96
> 口径）。账本见 [evidence/2026-10-26-m2s97-document-write-run-face.md](html-semantics-compat/evidence/2026-10-26-m2s97-document-write-run-face.md)。
> **范围**：engine shim ×2（part06 执行面 + open/close/write 记账 / part04
> insertBefore 门 + prepare then 求值窗标记）；testharness.rs 上轮 writer 单元
> 截断修复随本切片验证（007-012 rawtext 语义）；无其他 .rs 触碰；语料不可改
> 维持（探针页 11 页已删）。
> **挂账**：102 runner 面完成探测接线（write 注入 harness `__zw_mark_harness_loaded`）；
> 116 frameset 文档语义缺口；063 Timeout（hook=function pending=1）承袭；module
> 尾片（.py 前置）；charset 12F（.py）；svg-script-fragment-clone 3F；
> nomodule-async 2F（window.onload promise 门）；load-error-events 族（脱离树副本
> 元素事件保真）；FIXME：写流中页面脚本结构变更 bufTop 漂移、嵌套 write 内含外链
> 近似文档序、setAttribute('src') 通路、type 门、children-changed 其余变异入口
> （append()/textContent=/.text 已连接面——insertBefore 本轮已挂）；接口对象
> 基建面（承袭）；基线扫描宜空载（承袭）。020/048 本轮翻绿销账。
> **切片间隙**：本轮纯 engine shim 面；下一挂账 102 接线在 runner 面（.rs），
> 跨面间隙。

> **▶ 上轮状态（2026-10-26 M2 切片 96 施工收口，script.src prepare 触发 + for/event 门轮——其下一步 s97 入场探针定刀位已由 s97 收口，见上）**：
> **入场核账**：s95 HEAD（8c997860f）基线 **281P/256F/19T + 18 NotRun**（scripting-1
> 口径 574 行）。s96 入场探针自 256F 选 runner 可达 classic 动态 src 两簇：
> execution-timing 022/024（append 空脚本后连接态设 src——src set 无 prepare 触发，
> 探针实证非延迟非通道慢）+ 149 三个 dom_inserted for/event 臂（R387 无门）。
> **本轮两刀**：①`_zwScriptForevBlocked` for/event 门（双属性在场且非
> 「for≈window 且 event≈onload|onload()（ASCII 剥白 CI）」即中止；R377 fallback/
> 内联臂/R387b 三路径统一接入）；②R387b 执行体提升 `_zwPrepareSrcScript` 单点
> + part05 set trap SCRIPT.src 专用分支（反射写属性 + connected → prepare；
> `_reflectedUrlAttr` 分支加 SCRIPT/src 排除）。**主发现（跨 part 作用域结）**：
> helper 定义于 part04 appendChild trap 闭包，part05 set trap 是另一函数作用域
> 不可见——探针 DIAG `hasFn=undefined` 定谳，`globalThis._zwPrepareSrcScript`
> 导出解结（_zwNodeContains part03 导出先例）。探针教训：首轮 2s 轮询误报
> Pass（teardown 竞态），改 onload+阶梯时点 + fetch 插桩定谳「从不 fetch」；
> 插桩复核 B 臂全链路：TRAP→conn=true→PREPARE-CALLED→fetch→执行于 load 前。
> **验证**：探针页 9/9 全绿；execution-timing 全目录 022 F→P、023 T→P、
> 024 F→P 目录内零回归；script-for-event 两文件 34/34；149 **6F→3F**（dom 臂
> 翻绿，残余 3F 全 parser 臂=document.write 挂账）；change-src-attr-prepare-
> a-script **T→P**（spec 属性变更步骤正典页）；全子树终扫 **288P/251F/17T
> + 18 NotRun**，diff vs 基线 **F/T→P 7 行 5 页、P→非P 0 页**（终扫
> /tmp/s96-final-scan.txt 定为下轮基线）；engine lib **3530/3530**（guard）；
> 纯 .js 变更 fmt/clippy 不涉；reftest 未跑（无 render/layout 面，沿 s91-s95
> 口径）。账本见 [evidence/2026-10-26-m2s96-script-src-prepare-and-forev-gate.md](html-semantics-compat/evidence/2026-10-26-m2s96-script-src-prepare-and-forev-gate.md)。
> **范围**：engine shim ×2（part04 helper×2 + R387 门/调用改造 / part05 URL
> 分支排除 + SCRIPT.src 专用分支）；无 .rs 触碰；语料不可改维持（探针页已删）。
> **挂账**：execution-timing document.write 族（149 parser 臂 + 005-012/018/
> 026/027）；020 data: URL（fetch 通道 data: 支持未证）；048 sel 形 script[0].src=
> 自身缺口；module 尾片（.py 前置）；charset 12F（.py）；svg-script-fragment-clone
> 3F；nomodule-async 2F（window.onload promise 门）；load-error-events 族（元素
> onload 投递保真）；FIXME：setAttribute('src') 通路、type 门、children-changed
> 其余变异入口（insertBefore/append()/textContent=/.text 已连接面）；接口对象
> 基建面（承袭）；基线扫描宜空载（承袭）。
> **切片间隙**：本轮纯 engine shim 面；下一挂账同流同面，间隙开放。

> **▶ 上轮状态（2026-10-26 M2 切片 95 施工收口，script.text IDL + children-changed 记账轮——其下一步 s96 入场探针定刀位已由 s96 收口，见上）**：
> **入场核账**：s94 HEAD（47bc3dffd）基线 **263P/270F/23T + 18 NotRun**（scripting-1
> 口径 574 行）。s95 入场探针自 270F 聚类选 runner 可达 classic 脚本文本语义两簇：
> script-text.html 6F（.text getter=child text content / setter=string replace all，
> 全 detached 纯 IDL 面）+ emptyish-script-elements 6F（解析期空源 bail-early 不置
> already-started → 补 Text 子须执行）。排除 runner 上限：script-charset-01（.py
> 执行器）、script-text-modifications（slow.py trickle 时序）。**本轮两刀**：
> ①script.text IDL（get trap 直接子 Text walk——comment/PI/元素子不计；set trap
> null→"null" 归一后委托 textContent——R3907 同口径）；②appendChild trap SCRIPT
> 父 children-changed→prepare（头部 pre 快照 + 尾部触发块：pre 空 + post 非空 →
> connected → already-started 门 → nomodule 门 → classic eval + 置位）；
> already-started 记账 = part06 __zw_set_current_script 投递时按 __zwSelector 记
> _zwScriptStarted（空源 bail/门压制不投递不记——与 spec prepare 置位时机对齐；
> handle 形沿 R387 _zwRanScripts）+ cloneNode 传播（R3901/R3902b 同型）。
> **回归与记账返工（本轮主发现）**：首轮纯 pre/post 近似出 execution-timing
> 059/060/062 三行 P→F——执行中脚本自摘子（removeChild/innerHTML=''）再补文本、
> started 脚本浅克隆后补文本，pre 快照捕获在变异后误判「未启动」；返工即投递
> 记账 + 克隆传播 + 执行后置位（pre/post 降第二道近似），三页回绿且连带解锁
> 克隆/补文本族：061/064/065 file Timeout→全绿解挂、036 Timeout→subtest Fail
> 显形、066 F→P、127/128 F→P。
> **验证**：定向 script-text 6P/0F、emptyish 7P/0F、059/060/062 回绿、120-126
> 与基线逐行一致；全子树终扫 **281P/256F/19T + 18 NotRun**，diff vs 基线
> **F/T→P 14 行 + 解挂 4 页 + 066 F→P、P→非P 0 行**（14 行 = script-text 6 +
> emptyish 6 + execution-timing 127/128 2），域账 **282P/256F/19T**（+1 惯例）。
> 单测腿：engine lib **3530/3530**（guard）；纯 .js 变更 fmt/clippy 不涉；
> reftest 未跑（无 render/layout 面，沿 s91-s94 口径）。账本见
> [evidence/2026-10-26-m2s95-script-text-and-children-changed.md](html-semantics-compat/evidence/2026-10-26-m2s95-script-text-and-children-changed.md)。
> **范围**：engine shim ×3（part03 get trap / part04 appendChild 钩子+克隆传播 /
> part06 投递记账）；无 .rs 触碰；语料不可改维持（探针页已删）。
> **挂账**：execution-timing 编号套件 78F（036 显形 deep clone 族 / 122-126
> type-async 面 / 005-012 document.write 族）；module 尾片（.py 前置）；charset
> 12F（.py）；svg-script-fragment-clone 3F；nomodule-async 2F（window.onload
> promise 门）；load-error-events 族（元素 onload 投递保真）；FIXME：children
> changed 其余变异入口（insertBefore/append()/textContent=/.text 已连接面）未挂
> 触发；接口对象基建面（承袭）；基线扫描宜空载（承袭；s95 终扫
> /tmp/s95-final-scan.txt 定为下轮基线）。
> **切片间隙**：本轮纯 engine shim 面；下一挂账同流同面，间隙开放。

> **▶ 上轮状态（2026-10-26 M2 切片 94 施工收口，prepare 双门 + 投递保真 + module 缓存键轮——其下一步 s95 入场探针定刀位已由 s95 收口，见上）**：
> **入场核账**：s93 HEAD（43f10a3e7）基线 **211P/282F/25T + 22 NotRun**
>（scripting-1 口径 540 行）。s94 入场探针自 282F 聚类选 runner 可达 classic
> 属性面：nomodule face 7F + script-for-event face 13F。**本轮三刀**：
> ①extract prepare 双门（`script_suppressed_by_nomodule`：nomodule 内容属性在
> 场压制 classic、module 不受影响——module/nomodule-attribute 以 this.js 顶层
> this 区分执行形态；`script_blocked_by_for_event`：双属性在场 → ASCII 剥白后
> for≈"window" 且 event≈"onload"|"onload()"（ASCII case-insens），任一属性缺
> 失照常执行——语料 Script 15/16 单属性臂定谳）；②shim R387 动态插入钩子
> nomodule 门（三执行分支条件前缀；不置 run-once——spec 中止在 already-started
> 置位前，摘属性可重来）；③runner 投递保真扩口（inline_local_scripts 保留
> nomodule——新 `open_tag_has_attr` bool 属性裸形在场判定，与 attr_from_open_tag
> 的 nonce 裸形视缺失分工；保留 type="module"——s92 挂账兑现）。
> **附带根因返工（本轮主发现）**：type 保真暴露 R5182 潜伏缺陷——`__moduleCache`
> 文档级记忆化键误用页面 URL，同文档第二枚 module script 被首枚记忆化命中短路、
> 模块体零次求值（currentScript-null file Timeout + probe-a~d 最小探针二分定谳
> 任意双 module 页挂起；潜伏态，语料此前无双内联 module 页）；修复 = 键参数分离
> `compile_module_script(source, url, cache_key, registry)`（内联 = 页面 URL 挂
> ordinal 片段、外链 = src 绝对化；三生产调用方语义不变）+ learning 沉淀
>（bugs/2026-10/2026-10-26-module-cache-key-page-url-collision.md）。
> **验证**：定向 nomodule FILTER 4F→P、script-for-event 18 行全绿（13F→P）、
> 探针 4 页全绿、currentScript-null 全绿（连带基线 Fail→Pass——module 求值
> currentScript=null 语义成立）；全子树终扫 574 行 **263P/270F/23T + 18 NotRun**，
> diff vs 基线 **F→P 23 行、P→非P 0 行**（for-event 13 + nomodule 4 +
> currentScript-null 1 + module-vs-script-2 1 + module/load-error-events 3——
> 页内 type=module 标签在，缓存键解锁，非 classic load-error 族）；module 族
> 多页首次以 subtest 粒度完成（type 4 例、imports 5 例、integrity inline 1、
> module script-for-event 18 例、empty-attributes-clause 1 例等翻绿）。
> 历史口径重述 **211P/282F/25T（540 行期）→ 263P/270F/23T（574 行期，行数增长
> = 粒度完成非口径变化）**，域账 **264P/270F/23T**（+1 惯例）。单测七腿：
> engine **3530**（+3）/runner **226**（+76i）/webview **800**/sandbox v8
> **365**（+1）/quickjs **260**（+1）；fmt/clippy 四 crate clean；reftest 未跑
>（无 render/layout 面，沿 s91-s93 口径）。账本见
> [evidence/2026-10-26-m2s94-prepare-gates-and-delivery-fidelity.md](html-semantics-compat/evidence/2026-10-26-m2s94-prepare-gates-and-delivery-fidelity.md)。
> **范围**：engine ×2（extract 双门 + shim 钩子）、script-sandbox（签名+单测）、
> webview（缓存键接线）、runner（投递保真）；css/layout/renderer 零改动；语料
> 不可改维持（probe 临时页已删）。
> **挂账**：async nomodule 2F（window.onload promise 门——window load 派发
> face）；sync 臂 1（元素 onload 属性投递丢失 / 元素 load 事件 face）；
> nomodule-set-on-external-module-script（外链 module 装载面）；module 族新显形
> Fail（charset-01/03、integrity 外链、single-evaluation-1/2、instantiation-6/7、
> specifier-error、evaluation-error-2、fetch-error-2、import-attributes 部分
> ——.py 执行器前置项不变，subtest 账已立）；execution-timing 80F / 顶层 65F /
> 完整接口对象基建面（承袭）；基线扫描宜空载（teardown 粒度负载敏感，承袭；
> s94 终扫 /tmp/s94-final-scan2.txt 定为下轮基线）。
> **切片间隙**：本轮 engine/sandbox/webview/runner 面；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-26 M2 切片 93 施工收口，classic 脚本属性面批刀轮——其下一步 s94 入场探针定刀位已由 s94 收口，见上）**：
> **入场核账**：s92 HEAD（2cf2a5c5c）基线 **201P/291F/25T**（the-script-element
> 子树 517 行）。s92「下一步 module 尾片盘点」入场探针后**转向**：module 尾片
> 43F 簇（css-module 20 + json-module 11 + text-module 8 + import-attributes 4）
> 被 wptserve .py 执行语义缺口封顶（serve-custom-response.py 等以静态文本投递，
> runner 无 .py 执行器），转 runner 可达 classic 属性面。**本轮两刀**：
> ①shim part03 R3019 占位区 `HTMLScriptElement` 接口对象 + 静态 `supports(type)`
> ——HTML #dom-script-supports 四值精确匹配（'classic'/'module'/'importmap'/
> 'speculationrules'→true，JavaScript MIME 类型/空白沾染/大小写变体全 false；
> spec 明示不做 ASCII case-insensitive、不剥空白；webReader 取回 spec 原文定谳，
> 语料为旧版无 importmap/speculationrules 断言，按 spec 现行实现两相兼容）；
> shimming 前 grep 定谳 shim/Rust 侧零 HTMLScriptElement 全局引用，`||` 守卫
> 无冲突面。②extract `trim_ascii_html_ws`（TAB/LF/FF/CR/SPACE）替换两函数的
> `str::trim`——type 剥离收窄到 ASCII whitespace，\x0B/\x85/\xA0/ /　
> 尾随不再误判 JS；调用方仅脚本提取循环两处，+单测 ASCII 照剥/非 ASCII 不提取
> 双形态。**验证**：定向 script-supports 5/5、script-type-whitespace 5/5 全绿；
> 全子树复扫（FILTER=scripting-1 加宽口径）540 行 211P/282F/25T+22 NotRun，
> diff vs 基线 9 行 F→P（两面全数）+23 行非 Diff 逐行归因：1 行 noscript Pass
> = 口径加宽（本来就绿）、22 行 = 超时页 teardown 粒度——**stash 复归位 s92
> 代码重建复跑 load-error 族 bitwise 同现（diff 空）+当前二进制全扫/单跑行集
> 一致**定谳非附带；基线缺行系 s92 扫描会话后台负载早停。历史口径重述
> **210P/282F/25T（517 行恒等，+9P −9F）**，域账 **211P/282F/25T**（+1 惯例）。
> 单测：zero-engine **3531/3531**（lib 3527=3526+1；engine 自 s92 零外来变更）；
> fmt/clippy clean（ulimit+timeout shell 兜底）；reftest 未跑（无 render/layout
> 面，沿 s91/s92 影响面口径）。账本见
> [evidence/2026-10-26-m2s93-classic-script-attribute-faces.md](html-semantics-compat/evidence/2026-10-26-m2s93-classic-script-attribute-faces.md)。
> **范围**：engine ×2（shim 一处 + extract 一函数族）；runner/webview/
> script-sandbox 零改动；css/layout/renderer 零改动；语料不可改维持。
> **挂账**：完整接口对象面（HTMLScriptElement 仅静态 supports，constructor/
> instanceof 同一性缺，`__zwHtmlTagIface` 全表无真实 class 同族挂账）；module
> 尾片 43F 簇（.py 执行器为前置，含 propagate-nonce-inline-module 1F 与
> module 路径 CSP 检查点缺位承袭）；execution-timing 80F / 顶层 65F（既有账）；
> runner 报告粒度受会话负载影响（超时页 teardown，基线扫描宜空载跑——s93 终扫
> /tmp/s93-final-scan.txt 定为下轮基线，FILTER=scripting-1 口径 540 行）。
> **下一步**：s94 入场探针定刀位（候选：execution-timing 80F scheduler 语义 /
> 顶层 65F / 接口对象基建面）。
> **切片间隙**：本轮 engine 面；下一挂账同流同面，间隙开放。

> **▶ 上轮状态（2026-10-26 M2 切片 92 施工收口，CSP nonce 传播轮——其下一步 s93 module 尾片盘点入场转向 classic 属性面，已由 s93 收口，见上）**：
> **入场核账**：s91 HEAD（abdd1b178）基线 **197P/295F/25T**。残面：
> string-compilation-nonce-{classic,module} must-not-inherit 4F（s88
> 挂账 CSP 面）——setAttribute onclick 两臂 spec 语义：事件处理器不
> 继承触发脚本 nonce → 体内 import() 的 module fetch 被 `script-src
> 'nonce-correct'` 拒 → TypeError reject；红因 = 引擎动态 import 宿主
> 解析路径无 CSP 检查、无 nonce 传播通道（import 成功 →
> promise_rejects_js 断言失败）。**既有基座**（security-hardening
> M2-s1~s7）：meta CSP 解析、`check_script(nonce,hash,url)`、violation
> 事件、eval/attr 门（`unsafe-eval` 放行页不激活）——缺口仅在
> import fetch 检查点。**本轮三刀**：①script-sandbox
> `DynamicImportState.nonce` 槽位 + resolver 契约升三参（空串↔None
> 双射）+ `set_dynamic_import_nonce`/`__zwSwapImportNonce` swapper
> （trait 默认 no-op，QuickJS 降级；空契约路径既有 type_error reject
> 零改动命中判据）；②webview 逐脚本 nonce 刷新（spec 动态 import 继承
> 触发脚本 nonce，hostimportmoduledynamically-retrieval）+ resolver
> 检查（violation 入 connect 共享队列 + 返空串）；③shim 求值窗——
> 处理器传 `''`（不继承）、timer 注册期捕获（字符串编译继承）、R387b
> 窗换 `''`（动态插入无 nonce）。**返工（投递保真）**：首轮全子树
> 2 行附带——propagate-nonce-external-{classic,module} 翻红，ZWDBG
> 双探针（resolver 收参 + 刷新序列，跑完即撤）定位：页面外链测试脚本
> 经 runner `inline_local_scripts` 内联投递丢原标签 nonce 属性 → 刷新
> 读 None → 槽位空。修复：投递保留 nonce（复用 extract
> `attr_from_open_tag` 升 pub，`&`/`"` 转义；上方 harness 注入块
> csp_nonce_attr 同款保真先例）。**验证**：nonce 两页 12/12（4 红臂
> 翻绿 + 8 inherit 臂在 CSP 检查开启下保持绿）；propagate-nonce 4 页
> 3 绿 1 红（inline-module 1F = s91 前既有 module 路径面）；终扫
> **201P/291F/25T**，diff vs 基线恰 4 行全落 nonce 族、族外零移动，
> **string-compilation 族 46P/0F/0T 完全清盘**（s88 入场 3P/3F/8T）。
> 单测六腿：runner **226**、engine **3526**、sandbox v8 **364**（+2
> 新测）、quickjs **259**、webview **800**；fmt/clippy 四 crate
> `-D warnings` clean（ulimit+timeout shell 兜底同 s91）；reftest 未
> 跑（无 render/layout 面，无政策快路径返 None，沿 s91 影响面口径）。
> 域账自此按 **202P/291F/25T** 记（raw 201P + 惯例 +1）。账本见
> [evidence/2026-10-26-m2s92-csp-nonce-string-compilation.md](html-semantics-compat/evidence/2026-10-26-m2s92-csp-nonce-string-compilation.md)。
> **范围**：script-sandbox ×2 / engine ×2（shim + extract 一词 pub）/
> webview / wpt-runner runner 侧五 crate；css/layout/renderer 零改动；
> 语料不可改维持。
> **挂账**：propagate-nonce-inline-module 1F + module 路径动态 import
> CSP 检查点缺位（es_module 重写走 `__zw_compile_module` shim 路径
> 绕开检查——与 classic 槽位路径正交）；`inline_local_scripts` 不保留
> `type="module"`（投递语义保真，本刀语料不受影响）；security-csp 域
> 旁证（他流域控制面，下次跑批建议重基线）；QuickJS 后端降级维持。
> **下一步**：s93 module 尾片盘点——the-script-element 余账
> （propagate-nonce-inline-module 等 module 路径面）按 s86 切片序④
> 入场先探针定刀位。
> **切片间隙**：本轮 script-sandbox/engine/webview/runner 面；下一
> 挂账同流同面，间隙开放。

> **▶ 上轮状态（2026-10-26 M2 切片 91 施工收口，动态 import() 重写器词法感知 + module 面收口轮——其下一步 s92 CSP nonce 面已由 s92 收口，见上）**：
> **入场核账**：s90 HEAD（8c4a575dd）the-script-element 全子树复扫
> **173P/294F/30T**（497 行）。残面：string-compilation-module
> eval 1F + 整页 1T、base-url-inline-module / nonce-module /
> base-url-external-module 整页 3T、base-url-external-classic 整页 1T
> （语料缺 dynamic-import/scripts/——fetch_dir_html depth-1 不下钻）。
> **根因**（STEPS 探针串实证）：`rewrite_dynamic_imports` 纯文本扫描
> 把**字符串字面量内** `import(` 也改写——字面量被 Function/eval/
> setTimeout 字符串体编译后在全局作用域执行，体内 `__zw_dynamic_import
> .call({ referrer: _importMeta.url }, ...)` 引用 IIFE 局部绑定
> `_importMeta` → ReferenceError，import 永不发生；WPT 子测试名（显式
> 字面量）在 s90 扫描被污染成 `__zw_dynamic_import.call(...)` 同源实证。
> **本轮三刀**：①es_module 重写器 `RewriteCtx` 栈词法感知——Literal
> （引号定界/转义对）/TemplateExpr（`${}` 内照常重写、花括号计深）/
> 行注释/块注释，栈空代码态照旧；5 拷贝点改 `char::len_utf8` 步进
> （首版按字节切遇 WPT 语料 `ś` panic、runner 批次 rc=101 全灭——返工
> ①）+多字节单测钉死；已知限制：含引号正则字面量误入 Literal 态、
> 漏改走 V8 原生宿主回调语义仍对；②语料 fetch_raw 补 scripts/ 5
> loader（gitignore 本地数据）+ part04 R387b 外链经典脚本求值窗
> `__zwSwapImportReferrer(自身绝对 URL)` finally 换回（concept-scripts
> -base-url）；③v8_runtime swapper 无参调用 = 纯 getter + part01
> 字符串 timer 注册期捕获基（fire 在求值窗外槽位已被改写）+ part03
> `_zwCallHandlerSwapBase(fn, thisArg, ev, base)` 拆通用形、
> `_zwCallHandlerDocBase` 变薄壳。**验证**：全子树基线/修后双扫 diff
> 归因配对核清零误伤：**+24P +1F −5T**——module 5（eval F→P+4 拆臂）、
> base-url-inline-module 5、base-url-external-classic 5、
> base-url-external-module 5、nonce-module 4P+2F（must-not-inherit
> 拆臂显形既有红面非回归）；族外 27 行派生名恢复 + import-meta 1 行
> 同名异写状态不变（名恢复即刀A 生效直接证据）。族终态 **42P/4F/0T**。
> 单测 engine **3526/3526**、webview **800/800**、sandbox v8 **362/362**
> （含新增 4 测）、quickjs 全绿；fmt clean、clippy 双 crate
> `-D warnings` clean（用户 systemd scope 本环境不可用，ulimit 24G +
> timeout 900 shell 兜底）。域账自此按 **198P/295F/25T** 记（raw
> 197P + 惯例 +1）。账本见
> [evidence/2026-10-26-m2s91-module-context-lexer-aware-rewrite.md](html-semantics-compat/evidence/2026-10-26-m2s91-module-context-lexer-aware-rewrite.md)。
> **范围**：script-sandbox ×2 / engine shim ×3 文件；css/layout/
> renderer 零改动；语料不可改维持（探针页跑完即删）。
> **挂账**：nonce-classic/-module must-not-inherit 4F（CSP nonce 语义
> 未建模，s88 已挂）；正则字面量限制（漏改走原生回调语义对、换基精度
> 降）；语料 fetch 深度工具局限在案；QuickJS 后端动态 import 不支持态
> 维持。
> **下一步**：s92 二选一定刀位——CSP nonce must-not-inherit 4F（引擎
> CSP 语义建模，s88 挂账合并）或 module 尾片盘点；开工先探针。
> **切片间隙**：本轮 script-sandbox/engine 面；下一挂账同流同面，
> 间隙开放。

> **▶ 上轮状态（2026-10-26 M2 切片 90 施工收口，string-compilation 动态 import() 字符串编译轮——其下一步 s91 module 上下文面已由 s91 收口，见上）**：
> **入场核账**：s89 HEAD（44e23748f）the-script-element 全子树复扫
> **158P/294F/33T**（485 行）。string-compilation 族基线 3P/3F/8T：
> classic 整页 T（字符串 setTimeout handler 被静默丢弃永不 fire，堵死
> harness 串行队列连累 4 臂 pending）、promise-result 2F（执行中换
> `<base>` 后 import 基走错）、base-url-inline-classic / nonce-classic
> 整页 T、module 系 4T + module eval 1F、base-url-external-classic 1T。
> **本轮三刀**：①part01.js `setTimeout/setInterval` 补字符串 handler
> `new Function` 注册期编译（HTML spec TimerInitializationSteps 的
> WebIDL callback 转换；语法错同步抛同 Chrome）——classic 页解堵拆臂；
> ②engine extract 新增 `script_effective_base_urls`（zero_dom 树序，
> `<base href>` 更新 current_base、script 记基，与 extract_page_scripts
> _indexed 同序号空间）接 webview `run_page_scripts_impl`——外链经典
> 脚本 referrer 用自身 src、内联用捕获表（concept-scripts-base-url：
> parse 时刻定格，动态 base 不回溯）；③script-sandbox 新增
> `register_dynamic_import_referrer_swapper`（`__zwSwapImportReferrer`
> 返旧换新，捕获 DynamicImportState Arc 不借 &mut self）+ shim
> `_zwCallHandlerDocBase`（inline/反射 handler 求值窗换当前文档基、
> finally 换回零外溢）接四处编译点——reflected/UA 臂 import() 用求值
> 时刻文档基（event-handler-processing-model）。**返工两笔记账**：
> 原文偏移扫 `<script` ≠ DOM 序号（harness 注入 + 字符串字面量过计数，
> DEBUG 探针实证后改 zero_dom 树序）；body-reflect `var fnB` 循环闭包
> 读末轮绑定（R2946 单测红，改 `_zwWrapHandlerDocBase` 参数绑定）。
> **验证**：全子树 stash 基线/修后双扫 diff 恰 31 行全落族内零误伤：
> **+20P −2F −3T +2F（nonce must-not-inherit 从整页 T 拆臂显形，非回归）**
> ——族 3P/3F/8T → **18P/2F/5T**（classic 5/5、promise-result 4/4、
> nonce 继承臂 4/4、base-url-inline-classic 5/5 全绿）。单测 engine
> **3526/3526**、sandbox v8 **358/358**、quickjs **255/255**、webview
> **800/800**（首跑 1F 系 target-disk-guard 清理 zero-image-decoder
> 独立进程二进制，补建全绿非代码回归）；fmt clean、clippy 三 crate
> clean（test-guard 包裹）；reftest **716/716** rc=0。域账自此按
> **174P/294F/30T** 记（raw 173P + v8-code-cache +1 惯例；F 双扫同
> 294，对 s89 账面 295F 为截断方差）。账本见
> [evidence/2026-10-26-m2s90-string-compilation-base-url.md](html-semantics-compat/evidence/2026-10-26-m2s90-string-compilation-base-url.md)。
> **范围**：engine shim ×2 / engine extract / script-sandbox ×2 /
> webview 六文件；css/layout/renderer 零改动；语料不可改维持（探针页
> gitignore 数据跑完即删）。
> **挂账**：module 上下文 5 文件 T（`<script type="module">` 页基座，
> module 面单独探）；base-url-external-classic 1T（dynamic
> createElement('script') 外链页面加载器）；nonce-classic must-not-
> inherit 2F（CSP nonce 语义未建模，s88 已挂）；QuickJS 后端
> classic 动态 import 不支持态维持（swapper trait 默认 no-op 降级）。
> **下一步**：s91 the-script-element 余账——module 页基座先行探针
> （区分页面加载失败 vs import 永不落定），再决 external-classic
> 动态加载器刀位。
> **切片间隙**：本轮 engine/script-sandbox/webview 面；下一挂账同流
> 同面，间隙开放。

> **▶ 上轮状态（2026-10-26 M2 切片 89 施工收口，execution-timing stylesheets-blocking 轮——其下一步 the-script-element 基面 classic 面已由 s90 收口，见上）**：
> **入场核账**：s88 基线（HEAD=4fdaf03e5）execution-timing 目录当拍
> **48P/94F/8T**——s86 记「12F 单簇」细分为 106-\* ×10 + 107-\* ×2 的
> 「scheduler: stylesheets blocking \*」目标行；其余 82F 编号套件预存红
> （s88 基线同态非回归）。四根：①语料根——`fetch_dir_html` 扩展名表缺
> `*.css`，css/background.css（`#test{position:fixed}`）与 css/import.css
> （`@import url("background.css")` 无分号）从未入语料；②装填根——
> `set_active_external_css` 生产恒空（唯一调用点是单测），getComputedStyle
> 桥读空级联；③@import 根——进程内无递归展开，且无分号语句文本拼进合并
> 级联后 css-syntax at-rule 消费遇裸 `{` 吞块，把**下一张表首条规则**吞掉
> （106-import 实证：双表 fetch 序正确探针仍 `static`）；④107 根——隐式
> write 走 `body.innerHTML=基线` 重放，原 `#test` 注册元素被 plain 产物
> 替换（无 `__zwSelector`）→ position 不在 UA 默认表返 `''`。
> **本轮一刀（四文件）**：webview `load_html` 同步发布外链样式表文本
> `set_active_external_css`（css-less 传空串复位防跨文档泄漏）+
> `resolve_external_css` @import 递归队列（对齐 R2411：相对导入表 URL、
> css_seen 防环、FIFO 文档序）；css_urls 新增 `strip_import_statements`
> （Tokenizer span 切除顶层 @import 语句，接线两处拼接点，css-parser
> 未动回避跨流改写）；engine shim getComputedStyle `position` 初始值
> `static` 回落（css-position-3，R5009 float 同形）；runner lib.sh 扩展
> 名表补 `*.css`。FIXME：@import 臂不走 CSP/Mixed 门（async_load 既有
> 同态）；级联顺序近似（子表追加在导入表文本后，逐位拼接归基建轮）。
> **验证**：execution-timing 子树 **48P/94F/8T → 62P/80F/8T**，stash 基线
> 逐行 diff 恰 **14 行 Fail→Pass**（106×10 + 107×2 + 150×2 同根红利）、
> Pass→Fail 0 行、T 不变；the-script-element 全子树 **158P/295F/33T**
> ——非目标 144 行 Pass 逐位不动（零误伤），F 差 1 为 TLA 截断点漂移
> 报表方差；dynamic-import 金丝雀与 stash 基线 bitwise 同态（对 s87
> 账面漂移为环境方差不记本刀）。单测 webview **799/799**（含探针三钉
> + css_urls 4 钉）、engine **3523/3523**；fmt + clippy 三 crate clean；
> reftest **716/716**（清缓存全量重编）。域账自此按 **159P/295F/33T** 记
> （raw 158P + v8-code-cache +1 惯例，F −13 与 P +14 之差同截断方差）。
> 账本见 [evidence/2026-10-26-m2s89-execution-timing-stylesheets.md](html-semantics-compat/evidence/2026-10-26-m2s89-execution-timing-stylesheets.md)。
> **范围**：生产面 webview ×3 / engine shim / runner lib.sh 五文件；
> script-sandbox/layout/css-parser 零改动；语料不可改维持（css 两文件
> 已按 pin 补拉本地，gitignore 数据命令留档）。挂账：编号套件 82F
> （trickle pipe/多文档时序）、webview 第 2 跑单失败未复现 flake。
> **卡点（非本流）**：`make test` 预存红维持 s86 记录——quickjs
> zero-renderer `mutation_arm_…reset_window`（归 perf/renderer 流未解）。
> **下一步**：s90 the-script-element 基面（s86 切片序③——string-compilation
> 余 4 臂挂 runner 计时器/Function Proxy/事件派发建模、nonce CSP、
> base URL、delay-load trickle、credentials M3；明细见 s88 账本 §4）。
> **切片间隙**：本轮 webview/engine/runner 面；下一挂账同流多面
> （shim/runner/CSP），间隙开放。

> **▶ 上轮状态（2026-10-26 M2 切片 88 施工收口，classic-script dynamic import 轮——其下一步 stylesheets-blocking 12F 簇已由 s89 收口，见上）**：
> **入场核账**：s87 遗留 classic 根 `Error: Not supported` 面当拍
>（HEAD=a2b8e71e8 全子树基线复扫 138P/313F/34T，域账面 −1P 为
> v8-code-cache 未上报同缺）细分：propagate-nonce-{inline,external}-
> classic 2F、string-compilation-classic 1F+1T、inline-event-handler 1T、
> base-url/nonce-classic 3T、css-module/import-css-module-dynamic 1F。
> **s87 归因勘误**：credentials 12F/1T 整族走 iframe + HTTP cookie
> handler（fetch set-cookie.py → iframe appendChild →
> fetch_tests_from_window），**非** classic import 根（本轮修复对其零
> 移动反证）；delay-load-event 1F 须异步取回 + trickle 时序建模。
> **本轮一刀（共享路径）**：script-sandbox 注册 V8 原生
> `set_host_import_module_dynamically_callback`（isolate slot 持
> referrer+resolver），三态消费与 `__zw_load_module` 同款契约（返空→
> 逐次新 TypeError / 空码→TypeError 携已解析 URL / `resolved\x1fiife`→
> 按需最小安装 prelude helpers + TryCatch 编译求值）；未注册 resolver
> 保 V8 默认 `Error: Not supported` 零外溢；`Sandbox` trait 新增
> `set_dynamic_import_resolver/referrer` 默认 no-op（QuickJS 降级不变）。
> webview `__zw_compile_module` 闭包体提升 `compile_module_contract`
> 共享函数，原生 resolver 同契约；每 classic 脚本前刷 referrer=文档 URL。
> **验证**：the-script-element/ 全子树基线/修复后双扫 bitwise
> **138P/313F/34T → 144P/308F/33T**，diff 恰 6 行全红→绿零误伤
>（nonce-classic ×2 + nonce-external-module + string-compilation-classic
> :: eval 4×F→P；inline-event-handler T→P；css-module/import-css-module-
> dynamic F→P 域外红利——无 type CSS module 动态 import 现按契约 reject
> TypeError）；fetch-error 16P/script-error/credentials 逐位不动。单测
> v8 **357/357**（+5 classic 原生 import 钉：成功/eval 字符串/取回失败/
> 图断/无 resolver 保真）、quickjs 255/255、webview 792/792；fmt +
> clippy `-D warnings` 两 crate clean。域账自此按 **145P/308F/33T** 记。
> 账本见 [evidence/2026-10-26-m2s88-classic-dynamic-import.md](html-semantics-compat/evidence/2026-10-26-m2s88-classic-dynamic-import.md)。
> **范围**：生产面 script-sandbox/webview 两 crate；runner 零改动；
> apps/browser 与 renderer 宿主未动（未注册 resolver 行为不变）；语料
> 不可改维持。FIXME 挂账：classic 上下文重复失败 `__moduleFailures`
> 同对象记账未接；目标模块体 import.meta 无注入。**卡点（非本流）**：
> `make test` 预存红维持 s86 记录——quickjs zero-renderer
> `mutation_arm_…reset_window`（归 perf/renderer 流未解）。
> **下一步**：s89 execution-timing stylesheets-blocking-scripts 12F 簇
>（s86 切片序②）；string-compilation 余 4 臂/nonce CSP/base URL/
> delay-load/credentials 残面归基面③、CSP、runner 建模、M3 各账
>（明细见账本 §4）。
> **切片间隙**：本轮 script-sandbox/webview 面；下一挂账 runner 面
>（stylesheets-blocking-scripts 时序），间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 87 施工收口，dynamic-import must-reject 族轮——其下一步 classic 面已由 s88 收口，见上）**：
> **入场核账**：s86 切片序①当拍 dynamic-import 子树 **6P/49F/11T**
>（FILTER=the-script-element/module/dynamic-import）。must-reject 24F
>「单根」经当拍细分为三根：①错误类型根 18F——`__zw_load_module` 宿主返空
> 一律普通 `Error`，spec fetch-a-single-module-script 取回失败须
> TypeError；②对象身份根 4F——parse/求值失败不缓存，重复 import 每次新
> 对象（spec module map 失败记账：parse/求值/依赖解析失败复抛同对象，
> fetch 期失败逐次新对象）；③MIME 根 2F——wptserve
> `?pipe=header(Content-Type,…)` 响应头模板未建模，wrongMimetype 按扩展名
> 误判 scripty。**本轮一刀（三面）**：es_module prelude `__zw_load_module`
> 失败三态（宿主返空→逐次新 TypeError 不入缓存 / `resolved\x1f` 空码标记
> →TypeError 入 `__moduleFailures` 复抛同对象 / eval 抛错→原错误对象入
> 缓存复抛）；webview `collect_module_deps_recursive` 返
> `ModuleGraphFailure` 三分类（TopFetch/DepResolution/DepFetch——裸
> specifier 无 import map 即解析失败），`__zw_compile_module` 失败两态
> 回传（fetch 失败返空 / 解析失败返空码标记）——renderer/tab 旧宿主返空
> 契约降级不劣化；runner `wpt_data_script_fetcher` MIME 派生序新增最高
> 优先 `?pipe=header` 臂。**验证**：子树 **6P/49F/11T → 28P/27F/11T**；
> the-script-element 全子树 stash 基线复扫 bitwise——**116P/335F/34T →
> 138P/313F/34T，diff 恰 22 行全 Fail→Pass、全落 dynamic-import/ 目标
> 族零误伤**（TLA 3 案页脚抛 rc=1 截断两轮同态、`v8-code-cache.html`
> 未及上报，runner 行为先于本轮）；域账自此按 **139P/313F/34T** 记。
> 单测：es_module 新增失败身份语义四态钉（取回 TypeError×新对象/图断
> TypeError×同对象/parse SyntaxError×同对象/求值 Error×同对象+体恰跑
> 一次）+ v8 352/352 + quickjs 8/8 + webview 792/792；fmt clean + clippy
> 三 crate clean（video_registry map_entry 门禁已由渲染流 `9d50e4a91`
> 同刻自修解除，见卡点）。账本见
> [evidence/2026-10-25-m2s87-dynamic-import-must-reject.md](html-semantics-compat/evidence/2026-10-25-m2s87-dynamic-import-must-reject.md)。
> **范围**：生产面 script-sandbox/webview/runner 三文件 + 契约单测；
> apps/browser 与 renderer 宿主未动（返空契约降级路径行为不变）；语料
> 不可改维持。**卡点（非本流，碰头记录）**：`make test` 预存红维持 s86
> 记录——quickjs zero-renderer `mutation_arm_…reset_window` 本轮复跑仍红
> 0.63s（page_scripts.rs 无新提交，归 perf/renderer 流未解）。
> **碰头记录（已化解）**：video_registry map_entry 代修与渲染流 `9d50e4a91`
> 同刻撞车（同款 entry API 改写）——保留上游版，本流重复提交 rebase
> skip 移除，零净损；渲染流 guarded-clippy 已全绿佐证门禁解除。
> **下一步**：s88 classic-script dynamic import 重写——classic 脚本
> `import()` 未重写，裸 V8 无宿主回调 reject `Error: Not supported`
>（credentials 12F + delay-load-event/propagate-nonce-classic/
> string-compilation-classic 同根），配 `check-cookie.py`/`set-cookie.py`
> runner handler 仿真 + 跨源 cookie jar；instantiation 2F 归 module 基面
> 切片④。
> **切片间隙**：本轮 script-sandbox/webview/runner 面；下一挂账同面
>（script-sandbox 重写），间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 85 定证收口，s84 根因证伪轮——其下一步挂账经 s86 核账已被 s79/s80 消费，见上）**：
> s84「gBCR materialize 滞后」根因三路定证**证伪**（s53 先例零代码轮，
> 插桩临时已全部回退/删除，工作树回 s84 基线）：①take_probe 链直插
> （ZW_PROBE_TRACE）——corpus complex 逐轮 `flush 即 apply → version
> 同拍 bump → 门即开 → changed=true → refresh fired=true`，GATE MISS
> 0 次（fired=false 仅 html 未变轮，正确 no-op），s84 三处漏批候选
> （version 门时序 / runner_probe_html 基线 / timer 泵批次归属）无一
> 成立；②Rust 复现测试（复刻 probe loop 双结构逐拍采样）——round 0
> rect 即非零（normal dialogA@8,100,784,18 …、slots dialogA@8,100,0,36
> …），12 轮 + +100ms 稳定，s84「三拍恒零、+100ms 才有效」不存在；
> ③TD 点击坐标实测——normal 嵌套 20/20 Pass、clickOn 落点非零命中
> 正确目标。**红图修正**（make + FILTER 当拍实测）：closedby 家族
> **104P/19F**——closedby 27P/0F、stacked 4P/0F、start-open 2P/0F、
> **simple 6P/0F（s84 记 2F 过时复绿）**、corner-cases 29P/1F（s53
> keep）、bounds-clicking 12P/2F（R-s31 渲染流）、complex 24P/16F
> （16F 全 shadow-slots，normal 20/20 Pass）。**归账**：16F=declarative
> shadow 联合切片（s51 用户门禁维持）、2F=渲染流、1F=keep——本流
> 工作面（engine shim/webview/runner）无剩余可施工面。s84 伪值推断
> （存疑不下死论）与「红图当拍复核」教训见账本。账本见
> [evidence/2026-10-25-m2s85-closedby-rootcause-falsified.md](html-semantics-compat/evidence/2026-10-25-m2s85-closedby-rootcause-falsified.md)。
> **范围**：零代码轮（s53 先例），不改生产面；corpus 不可改维持
> （正常面下我方 rect 同步可见，无需「新鲜度」修复）。
> **下一步**：testdriver-Actions 支持面（s78 挂账：hash-name-dynamic
> 5F + 指针/键盘合成输入族——runner 语料 testdriver.js 缺席面，runner
> 面本流工作面）。
> **切片间隙**：本轮零代码；下一挂账 runner 面亦本流，间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 84 分诊收口，closedby 同红假设重估轮——s85 证伪其根因，见上）**：
> A 族 closedby×slot 16F 挂账依据（crbug 417571176 同红，2026-04）重估
> **过时**：wpt.fyi chrome-158.0.8122.0 summary 实证 closedby 家族全绿
> （complex 40/40、simple 6/6、closedby 27/27、bounds 14/14、corner
> 30/30、stacked 4/4、start-open 2/2）——closedby 已 ship，**23F 转可
> 施工**（complex 16 + simple 2 + bounds 2 + corner 1）。**根因隔离**
> （三步插桩探针，临时已删）：失败首腿全是 light-dismiss 误触发——
> 首测点击 popover 的 ld 落点 px=0/py=0（corpus clickOn 2-rAF 后采样
> rect 恒零）→ s30 backdrop retarget → popover pass 误隐；gBCR 时序
> 探针证 showModal/showPopover 后 sync/2raf/4raf 三拍 rect 恒零、
> +100ms 才有效，且序列化 htmlLen +23（closedby+open）与 rect 转有效
> 同拍——mutation 批 materialize（apply→render→rect 快照）在 rAF 驱动
> 节拍整链滞后 ~5+ 探测轮。**下一步**：Rust 侧定证 take_probe
> flush/version-gate/refresh 链哪环漏批（候选：mutation_version 门
> 时序、runner_probe_html 基线、timer 泵与 rAF 拍批次归属），修复 rect
> 新鲜度（closedby 23F 与 abspos-dialog-layout 5F 共桥——本轮顺带观测
> `:modal` UA position:fixed 未上、dialog 盒 in-flow）。账本见
> [evidence/2026-10-25-m2s84-closedby-parity-recheck.md](html-semantics-compat/evidence/2026-10-25-m2s84-closedby-parity-recheck.md)。
> **范围**：零代码轮（s53 先例），不改生产面；corpus 不可改（2-rAF
> 采样是上游语料行为，Chrome 同步 rect 语义使其天然正确——修复须落
> 我方 rect 新鲜度）。
> **下一步**：s85 修复片——Rust 契约测试复现 gBCR 滞后 + take_probe
> 链定证 + materialize 时机修复；验证门 closedby 家族 23F 红转绿 +
> 域/pin 逐位持平。
> **切片间隙**：本轮零代码；下一挂账 webview/runner 面（rect 快照链
> ——本流工作面），间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 83 收口，handle 形元素 matches complex 读面轮）**：
> s82 遗留 matches complex 面分诊**推翻非平凡判定**：probe 实证 gap 非
> 同 turn 窗口而是**永久**——post-drain `matches('#anc span')` 仍 false；
> qS 倒查命中并登记 R100 锚，但 t6 锚臂 host `__zw_matches` 读原始
> dom_html 快照（apply 后锚不可解析），'0' 无条件 return 短路。s82 已
> 落双形态父链解析 + compound 匹配器，JS 侧判定材料齐备（无需 Rust
> 回调）。**修复**：part05 `_zwHandleMatchesComplex` 三态契约（true/
> false/null——null=有不可判 complex，落回锚臂 host 全语义，pseudo 面
> 不劣化）+ part04 t6 臂重排（R134 → s83 JS 走 → 锚臂 → regex）。列表
> 逐 complex 判定；compound 带 pseudo/组合器非后代/子代保守不判（防
> over-match）；48 跳封顶；不越 shadow 边界。**关键序**：JS 走必须先
> 于锚臂——锚臂 '0' 短路后置补偿不可达（probe 定位）。契约
> dom_bridge_tests `handle_element_matches_complex_s83`（18 断言）。
> **验证**：probe 同 turn + post-drain 全 Pass（临时已删）+ engine lib
> **3523P/0F** + 域（FILTER=embedded-content）**976P/252F/30T 逐位
> 持平 REG 0** + interactive pin **439P/44F/0T 逐位持平** + fmt clean
> + clippy 仅 s78 既有警告。账本见
> [evidence/2026-10-25-m2s83-handle-matches-complex.md](html-semantics-compat/evidence/2026-10-25-m2s83-handle-matches-complex.md)。
> **范围**：兄弟组合器（`+`/`~`）保守 false（反链无前兄弟指针，语料
> 0 需求）；锚臂 host 快照底层未动（JS 走已覆盖可判面，host 修复仅剩
> pseudo complex 面，无已知消费方）。
> **下一步**：720-stretch 渲染展开 22F（需布局几何，归 layout 集成
> 面，启动前查 oracle master 碰撞记录）。
> **切片间隙**：本轮 js_dom_shim 两件 + 契约 dom_bridge_tests；下一
> 挂账 layout 集成面需跨流协调，间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 82 收口，handle 形元素 closest 读面轮）**：
> s81 范围注记的「同 turn insert 身份缺口」分诊升级：createElement 产物
> 是 handle 形代理（`__zwSelector` 恒 null），closest 的 `!sel` 门使其
> **永久盲**（非同 turn 窗口——已 append 排水后仍 null；身份永驻 handle
> 形）。分诊矩阵实证：matches 单 compound true（t6 臂）、complex false
> （stale 缺口）、closest ×3 全 null；语料需求 0（117F × createElement
> 39 处 × closest/matches 共现 = 0），立项按 spec 定义 + 真实站点
> slice（jQuery/Swiper 创建元素 `closest`）。**修复双臂**
> （part05 `_zwHandleClosest` + part04 closest 入口 handle 分支）：
> ①R100 锚回（曾 sel 向解析过）→ host `__zw_closest` 全语义；②JS 单
> compound 祖先走——父链双形态解析（sel→`__zw_parent` 视图文档；handle
> →`_zwNodeParent` 反链），48 跳封顶（R125 口径），nodeType≠1 停
> （不越 shadow 边界——spec closest 祖先轴不含 shadow host），complex/
> 伪类保守 null（`_matchCompoundOf` 不求值 pseudos，防 over-match）。
> 契约 dom_bridge_tests `handle_element_closest_s82`（15 断言）。
> **验证**：probe 4 面 Pass（临时已删）+ engine lib **3522P/0F** +
> 域（FILTER=embedded-content）**976P/252F/30T 逐位持平（REG 0——
> 语料 0 需求预判实证）** + interactive pin **439P/44F/0T 逐位持平** +
> fmt clean + clippy 仅 s78 既有 concat! 警告（不在本 diff）。账本见
> [evidence/2026-10-25-m2s82-handle-closest.md](html-semantics-compat/evidence/2026-10-25-m2s82-handle-closest.md)。
> **范围**：matches complex stale 面（handle 形锚回 host 快照 miss）
> 记遗留不建——需 Rust 视图文档 matches 回调或 query_all 回退，非平凡。
> **下一步**：720-stretch 渲染展开 22F（需布局几何，归 layout 集成面，
> 启动前查 oracle master 碰撞记录）或 matches complex stale 面分诊。
> **切片间隙**：本轮 js_dom_shim 两件 + 契约 dom_bridge_tests；下一
> 挂账同面或 layout 集成面，间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 81 收口，查询面 id 改名可见性轮）**：
> R125（s79）读侧只接 gEBI；查询面（querySelector/querySelectorAll/
> matches/closest）同 turn 改名 `#new` miss、`#old` stale hit。probe
> 分轮定位**根因补刀**：`el.id` IDL 反射读走纯快照（与 getAttribute
> 的 R2995 lw 链同 turn 分歧）——`_matchCompoundOf` ids 校验读
> `p.id` 故 token 形态拉真全灭。**修复两刀**：①part03 id 臂 lw 化
> （共享路径，DOM 反射 spec 同步语义；`_zwRawIds`/handle 形保持）；
> ②part05 s81 helpers + 六挂点双臂补偿——stale 剔除（lw 现值复验，
> 全形态）+ 单 compound 拉真（R125 表候选 + in-doc 门 + full
> compound 复验 + 子树门；qSA 文档序归并）。R3190 属性语义不动
> （补偿只动选择器 id 约束）。契约 dom_bridge_tests
> `query_id_rename_same_turn_visibility_s81`（17 断言）。
> **验证**：细粒度 probe 12 面 Pass（临时已删）+ engine lib
> **3521P/0F** + 域（FILTER=embedded-content）**976P/252F/30T 逐位
> 持平（REG 0 / GAIN 0——s80 预判无直接语料依赖，实证）** +
> interactive pin **439P/44F/0T 逐位持平** + fmt clean。
> 账本见
> [evidence/2026-10-25-m2s81-query-id-rename-visibility.md](html-semantics-compat/evidence/2026-10-25-m2s81-query-id-rename-visibility.md)。
> **范围**：complex 祖先段约束 pull-in 不接（single 门，stale 剔除
> 仍全形态生效）；同 turn insert 元素无 `__zwSelector`（批末 apply
> 才有身份）上行 closest/query 盲区——结构性身份缺口与改名无关
> （无 id 对照面同样 null 实证），后续独立切片。
> **下一步**：720-stretch 渲染展开 22F（需布局几何，归 layout 集
> 成面，启动前查 oracle master 碰撞记录）或 s81 遗留同 turn insert
> 身份缺口分诊。
> **切片间隙**：本轮 js_dom_shim 四件 + 契约 dom_bridge_tests；
> 下一挂账同面或 layout 集成面，间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 80 收口，跨批 selector 迁移 journal 轮）**：
> s79 遗留 runner 缺陷 C（hash-name-reference-dynamic 3P/5F）定位并修复。
> s79 假说「序列化/apply 只经客户端缓存属性」**证伪**：Rust 侧 attr-trace
> 实证 mutations 队列不 drain（1→9 单调）、同键 lw 读亦通——丢点在
> **apply 解析**：全以创建期 sel 入队（proxy 闭包 sel 冻结），批 2 改名
> 落管线 doc 后，批 3 起旧键属性写 `find_by_selector` 恒 miss 走
> lenient skip 静默丢写；序列化无 name / 视图查询 miss / gEBI 旧键落空
> 均为下游表现。**修复（s80 journal）**：`apply_dom_mutations_full` 新参
> `sel_renames`——SetAttr(id) 落地记（改名前 sel → `#新id`），失配臂
> `journal_find_sel_rename` 追链（4 跳封顶）重试；直查命中优先（新元素
> 抢占旧 id 不劫持）。持有方 `RenderPipeline::sel_renames`，导航边界清、
> 同页 `render_html` 重建不清（selector→selector 映射不受 slotmap 换代
> 约束——首版在重建点清空致语料无效，复跑定位后修正）。契约
> dom_bridge_tests `test_apply_sel_rename_journal_cross_batch_attr_s80`。
> **验证**：engine lib **3515P/0F** + 域（FILTER=embedded-content）
> **976P/252F/30T（GAIN 5 / REG 0，hash-name-dynamic 3P/5F→8P/0F）** +
> interactive pin **439P/44F/0T 逐位持平**。范围：journal 只挂
> SetAttr/RemoveAttr 失配臂（SetText/SetStyle 等无语料实证面暂不扩）。
> 账本见
> [evidence/2026-10-25-m2s80-cross-batch-sel-rename-journal.md](html-semantics-compat/evidence/2026-10-25-m2s80-cross-batch-sel-rename-journal.md)。
> **下一步**：`querySelector('[id=…]')` 同 turn 改名可见性（共享查询
> 视图面）或 720-stretch 渲染展开 22F（需布局几何，归 layout 集成面，
> 启动前查 oracle master 碰撞记录）。
> **切片间隙**：本轮 js_dom_bridge + pipeline + webview 导航清点 +
> 契约 dom_bridge_tests；下一挂账同面或 layout 集成面，间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 79 收口，IDL id 改名 R125 对称臂轮）**：
> s78 未解决项 hash-name-reference-dynamic 5F 分诊——**控制面原记
> 「testdriver-Actions 面」证伪**：runner 自带 test_driver.Actions stub，
> 8 行点击全链路可达（ZW_TD_DEBUG resolve 迹：首点击 `#area` 全通，首个
> DOM 变更后 map 解析失败）。5F 真实面 = shim 两缺陷（本轮修复）+ runner
> 属性传播缺陷（未解决，后续切片）。**修复（R5279 两刀）**：①part04
> `.id=` set trap 补 R125 sel-based id 覆盖表登记（与 setAttribute('id')
> 对称——缺登记同 execute getElementById miss 新 id；latest-wins 缺写使
> 「改名回原 id」被残留旧条目毒化）；②part06 getElementById 第二条
> pending-by-id 兜底环补 `_r125Overridden` 门（快照命中被覆盖表剔除后
> 不得经该环复活）。契约 part90 `idl_id_rename_getelementbyid_r5279`
> （8 断言，撤任一刀对应转红）。**验证**：engine lib **3512P/0F** + 域
> （FILTER=embedded-content）**971P/257F/30T 持平** + interactive pin
> **439P/44F/0T 逐位持平**（R5279 只增 id 覆盖登记，语料面零数字变化）。
> **新诊断（未解决）**：runner 命令 turn 的 resolve 时点**属性写丢失**
> （`setAttribute('name')` 不在 live 态亦不入序列化——id 写有客户端缓存
> 可见、name 写不可见，疑序列化/apply 管线只覆盖客户端缓存属性）+
> **覆盖表缓存代理空子树**（re-parse 代际后 gEBI 命中代理
> `querySelectorAll('area')` → 0）——hash-name-dynamic 5F 实际归属此组合
> 面；resolve 迹仪器化方法已验证（ZW_TD_DEBUG + 临时探针，跑完即删）。
> 账本见
> [evidence/2026-10-25-m2s79-idl-id-rename-r125.md](html-semantics-compat/evidence/2026-10-25-m2s79-idl-id-rename-r125.md)。
> **下一步**：runner 属性传播缺陷切片（refresh 管线属性 mutation 应用 +
> 代际代理失效，面在 tests/wpt-runner + shim 视图生命周期）或 720-stretch
> 渲染展开 22F（需布局几何，归 layout 集成面）。
> **切片间隙**：本轮 part04/06 shim + 契约 part90；下一挂账 runner 面
> 在 tests/wpt-runner，间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 78 收口，F4 image-map 命中轮）**：
> s67 八族地图 F4 收口（the-area-element 三文件 area-coords 32F +
> area-shape 17F + area-processing 14F）。失败面修正：控制面原记 84F 系
> 旧账——实测 68F 中 5F 是 hash-name-reference-dynamic 的 testdriver-Actions
> 面（runner 语料缺 testdriver.js，指针/键盘合成输入族，本切片外），
> image-map 实际可收面 63F。**根因**：runner headless 无渲染管线——host
> `__zw_elementFromPoint`（R2924 HitTestCache）与 `__zw_getBoundingClientRect`
> 均缺席，elementFromPoint 缺席臂此前只有 SVG 形状回落，usemap img 形状
> 命中面为空 → 63F 全 null。**修复（R5278 三刀）**：①解析核（part03）——
> float-list 逐字解析（分隔集 ASCII 空白+逗号+分号、`\v` 非空白、垃圾跳读、
> 浮点前缀截断含指数 `.4`/`1e1`/`2in`、error→0）+ shape 枚举（ASCII
> case-insensitive，missing/invalid → rect）+ 点入形状（rect min/max 归一
> 半开框、circle d²≤r²、poly on-edge 预检**边界含入** + even-odd 半开边
> 射线——star (101,101)/(199,101)/hitPoly (119,101) 三行 corpus 定谳、
> default 恒真）+ hash-name reference；②命中面（part06 `_zwImgMapHitSel`）
> ——elementFromPoint 缺席臂（SVG 后置臂，产品路径零波及），img 框 (0,0)+
> IDL 尺寸近似（FIXME 真实几何归 layout 集），无形状覆盖/usemap 不可解析 →
> img 自身；③契约 part90 两测试（65 断言）。**排障留档**：裸跑 runner 缺
> Makefile 目标的 `ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`，interactive pin 假红
> 7 行（滚动/点击时序面）；stash 验证基线同红排除代码回归后改走 make 目标
> ——runner 数果实测必须走 Makefile 目标。**验证**：域（make + FILTER）run3
> **907P/320F/31T → 971P/257F/30T**（GAIN 64 = area 63 + usemap-casing
> 加成 1 / 0 P→F；audio-lazy T→P 为已记抖动族顺风向；裸跑/make/全域截断跑
> 三方 embedded-content 数字逐位一致）+ interactive pin **439P/44F/0T 逐位
> 持平**（make 目标重跑后）+ engine lib **3499P/0F** + clippy/fmt clean。
> 账本见
> [evidence/2026-10-25-m2s78-image-map-hit.md](html-semantics-compat/evidence/2026-10-25-m2s78-image-map-hit.md)。
> **下一步**：720-stretch 渲染展开 22F（需布局几何，归 layout 集成面）或
> testdriver-Actions 支持面（hash-name-dynamic 5F + 指针/键盘合成输入族）。
> **切片间隙**：本轮 part03/06 shim + 契约 part90；下一挂账 720-stretch 需
> layout 面协同，间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-25 M2 切片 71 收口，F1b SVG 固有尺寸轮）**：
> s67 八族地图 F1b 收口（naturalWidth-naturalHeight-width-height 的 SVG
> data URL 族 77P/181F + svg-img-density-vs-zoom 1P/8F）。**根因**：runner
> 位图解码面不解 SVG——host `__zw_get_image_size` 返回 viewBox 原始宽高
> （缺省 100×100），非 CSS-images natural dimensions 组合表；resourceState
> 无固有轴元数据，密度校正无法逐轴表达语料 data-density 语义（attr 轴校正、
> 默认/AR 轴不校正）；display:none 未渲染 IDL 面（whatwg#11287）恒密度校正。
> **修复（R5277 五刀）**：①`__zwSvgPayload5277` data:SVG payload 提取
> （percent/base64/raw）；②`__zwSvgNaturalDims5277` 固有尺寸核（无引号 attr
> 按 HTML tokenizer 语义、length 非负 number|px、viewBox AR、w+h/w/h/none
> 组合表）；③sweep SVG 核前置 + 覆写面——携 axes（本核已写）幂等跳过、
> host 态（无 axes）原位覆写不重派 load、非 SVG 维持 host 幂等；④密度逐轴
> 折算（attr 轴按选中密度校正、默认/AR 轴恒原始；无 axes 旧态默认 both，
> raster 语义不变）；⑤未渲染 IDL 臂——display:none + loaded 回原始未校正
> intrinsic（内联样式直读近似，FIXME 样式表/继承面）+ avail 传播携 axes。
> **运行期 fast path 撤除留档**：fetch 臂 SVG 前置在 129 img 模板克隆风暴下
> 超 `_deferBudget=256` 静默丢回调 → inflight 泄漏 → load 链死锁（run5 全域
> 1001/1258 行截断实证），同步 dispatch 变体仍抖（77P/181F 不稳）——终版
> 撤除，运行期 SVG 语义归 host 解码器切片。契约 part89 两测试（27 断言，
> (h) 覆写面 load 不重派钉）。**验证**：域 **747P/479F/32T → 907P/320F/31T**
> （GAIN 160 = 160 F→P / 0 P→F；唯一 T 变化 video-loading-lazy-autoplay
> T→F 为 s70 已记环境抖动行翻回）+ naturalWidth 文件 77P/181F → **236P/22F**
> （3× 稳定）+ interactive **439P/44F/0T 逐位持平** + engine lib
> **3496P/0F** + clippy/fmt clean。账本见
> [evidence/2026-10-25-m2s71-svg-intrinsic-dims.md](html-semantics-compat/evidence/2026-10-25-m2s71-svg-intrinsic-dims.md)。
> **下一步**：F4 image-map 命中测试 84F（elementFromPoint 几何）优先，或
> 720-stretch 渲染展开 22F（需布局几何，归 layout 集成面）。
> **切片间隙**：本轮 part03/04 shim + 契约 part89；下一挂账 F4 亦 engine 面，
> 间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-25 M2 切片 70 收口，F2 source-set 族轮）**：
> s67 八族地图 F2 收口（parse-a-srcset 49 + select-an-image-source 13 +
> update-the-source-set 84 + relevant-mutations 46 + current-pixel-density
> 21 + 邻族 7）。**根因**：srcset/sizes/picture-source 选择面仅 R4162/R2419
> 「首 URL」近似——描述符语法/DPR 择面/media+type 门/relevant mutations
> 触发/密度校正全缺。**修复（R5277 九刀）**：①part03 共享选择核
> （parse-srcset/parse-sizes/create-source-set/update-source-set/
> select-image-source 五核，spec §4.8.4.3.7-12 逐字）；②候选 URL 就地绝对化
> （`_zwResolveFetchUrl` 文档基含 `<base href>`——available-images 键/请求
> URL/currentSrc 同源；raw/绝对键并存把同图拆两个 avail 表项，
> image-loading-lazy-available P→F 实证）；③全状态写入面切选中 URL（sweep
> srcset-only img / `.src` IDL 链 / revalidate）+ 密度入 resourceState +
> naturalWidth 密度折算；④relevant mutations 触发面（src/srcset/sizes 恒重
> 验、crossorigin/referrerpolicy content-state 枚举比对、SOURCE 变迁 → 其后
> img 兄弟、picture 树插入/移除）+ restart 收窄为选中 URL 变更（同 URL 命中
> 照常重派 load——spec maybe-omit-events 门）；⑤null selection（src/srcset
> 移除 → broken，error 仅在仍有关联时派）；⑥detached 更新延一步（重读
> loading——src-then-lazy-load 时序）+ lazy 视口门只拦 fetch 不拦 avail
> settle（avail 先于 lazy 决定）+ scrollIntoView 视口旗标近似（F5 几何收口
> 前债务标记）；⑦broken 无 width/height 属性 → IDL 0（s68 误正改 0——
> 300/150 是 UA icon 非 IDL 面）；⑧css-parser media_query.rs MQ4
> error-handling——非空 inner 无法解析 → 整条不匹配，不再退化空条件 all 恒
> 匹配（`all and (!)` 面 + matchMedia false 对齐，钉定测试撤换）；⑨extract.rs
> 候选收集接解析核。契约 part88 五测试。**验证**：域 **607P/678F/37T →
> 747P/479F/32T**（GAIN 133 = 133 F→P / 0 P→F；run3 捕获唯一 P→F 由②收口
> run4 复验翻回；run3→run4 差恰 2 行）+ interactive **439P/44F/0T 逐位持平**
> + engine lib **3484P/0F**（+5 契约）+ css-parser lib **3003P/0F** +
> clippy/fmt clean。账本见
> [evidence/2026-10-25-m2s70-source-set.md](html-semantics-compat/evidence/2026-10-25-m2s70-source-set.md)。
> **下一步**：F1b naturalWidth SVG 固有尺寸/密度校正尾片（210F 余量）或 F4
> image-map 命中测试 84F（elementFromPoint 几何）——按共享路径收益取一。
> **切片间隙**：本轮 part03/04/05 shim + extract.rs + css-parser media_query
> + 契约 part88（part85 键位随绝对化更新）；下一挂账 F1b/F4 亦 engine 面，
> 间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 67 收口，embedded-content 开片分诊轮）**：
> 两件事。**① 域分诊八族地图落档（819F 全归因）**：F1 img 静态加载管线
> ~260F（settle 只接 `.src` IDL setter 通道 R56h，parser 静态 img 无
> fetch/load 事件、naturalWidth 恒 0）+ F2 select-an-image-source/currentSrc
> /srcset/sizes ~300F（currentSrc 恒 ''，全族算法未实现，独立大切片）+
> F3 img.decode() 56F + F4 image-map 命中测试 84F（elementFromPoint 几何
> 底座→渲染流记账回流）+ F5 lazy-loading ~49F（视口几何缺失，F1 后重
> 分诊）+ F6 canvas 53F（getContext options 序 + Tab 序，逐案分诊）+
> F7 object/embed 窄面 25F + F8 杂项 2F。域刷新基线 **458P/819F/40T**
>（执行案 231 与 M1 一致；subtest 1060→1317 为 R5235 帧驱动后此前
> Timeout 悬挂案完成注册）。**② F7 null 反射收口（R5274 一刀）**：part04
> get-trap IFRAME 分支后补 OBJECT/EMBED 分支——contentDocument/
> contentWindow 恒 null（spec「does not represent a nested browsing
> context → null」尾步）+ getSVGDocument 返 null 函数（GetSVGDocument
> Support）；EMBED 无 contentDocument/contentWindow 成员保持 undefined。
> 契约 part84 两测试（钉①五臂 + 钉② IFRAME R130 面不回归）。**验证**：
> 域 **458P/819F/40T → 461P/816F/40T**（GAIN 3 / REG 0）+ interactive
> 439P/44F/0T 逐位持平 + lib **3469P/0F**（+2 契约）+ 翻转力实证（撤刀
> → 钉①红，恢复字节一致）。账本见
> [evidence/2026-10-24-m2s67-embedded-content-triage.md](html-semantics-compat/evidence/2026-10-24-m2s67-embedded-content-triage.md)。
> **下一步**：F1 img 静态加载管线开片（~260F 最大可收口单族；DCL 扫描
> settle + loading=lazy 跳过 + setAttribute('src') 通道，R5254 两段式
> 先例）。
> **切片间隙**：本轮 part04 get-trap + 契约 part84；下一挂账 F1 亦
> engine shim/host 桥面，间隙开放，开工前查 oracle master 碰撞记录
>（2026-10-24 已查：oracle master 记「engine 面占用标记已退场清除」，
> 无新占用）。
>
> **▶ 上轮状态（2026-10-24 M2 切片 66 收口，text-level-semantics 开片轮）**：
> 两件事。**① the-a-element 13F 两族全收（R5273 三刀）**：a.text
> getter/setter = textContent 精确委托（9F；option 的 strip-and-collapse
> 不适用）+ a/area toString 品牌检查（4F；WebIDL es-stringifier step 2，
> **探针实证 trap 臂才是 createElement 产物 toString 实际供给面**——原型
> defineProperty 不达，双侧同步防直装路径）。**② 共享路径根修（part05
> overlay 种子）**：a.text 委托暴露预存缺陷——textContent=（sel 形）写后
> 同 turn childNodes.length===0 而 firstChild 已见新值（探针钉定与 .text
> 无关）。根因 = 注册文本节点无 pending link，overlay added 并入跳过而
> removed 剔除生效，childNodes 与 firstChild 两视图分裂（R5269 probe62
> childLen=0 同根源）。修 = `_zwOverlayPendingChildNodes` 入口
> `_zwTextElsBySel` 种子（命中即以注册节点为基；appendChild 并入/
> removeChild 剔除/apply 后不双计均保持）。**验证**：域 **34P/0F**
>（exit 0 首次）+ links 119P/20F/4T（stash 二分与 HEAD 逐值一致，预存）
> + document-metadata 59P/59F/18T + tabular 130P/4F + interactive
> 439P/44F + engine lib **3463P/0F**。账本见
> [evidence/2026-10-24-m2s66-text-level-a-element.md](html-semantics-compat/evidence/2026-10-24-m2s66-text-level-a-element.md)。
> **下一步**：embedded-content / scripting-1 / microdata 按挂账序开片
>（分诊轮先行）。
> **切片间隙**：本轮 part03/part04/part05 shim + part05 overlay；下一挂
> 账 embedded-content 分诊未定，间隙开放，开工前查 oracle master 碰撞
> 记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 65 收口，http-equiv 分诊 + content-language pragma 轮）**：
> 两件事。**① http-equiv-and-name 2F 分诊（本域 shim 面最后未分诊项，
> 分诊轮全归档）**：-1 = CanvasText 系统色 × color-scheme applied-color
> 链 → 渲染流记账回流（与 s64 color-scheme 23F 同桶）；-2 = 本流修复。
> **② 修复面（R5272，dom crate 两刀）**：①`parse_selector_chain` 复合伪类
> 名字边界——名字取 `(`/`#`/`.`/`[`/`:` 最早命中、仅 `(` 按含参取 args，
> 旧 `r.find('(')` 扫过整段余串把 `:root:lang(de-DE)` 的 `root:lang` 当
> 名字 → SyntaxError；②`matches_lang` pragma-set default language 回落
>（spec「language of a node」Otherwise 分支）——祖先链全无 lang 时吃
> `<meta http-equiv=content-language>`（含逗号失效、首 ASCII 空白 token、
> 文档序最后胜出；空 lang = explicitly unknown 不回落；HTTP 头回落
> FIXME），R3281 共享权威面 style-system `:lang()` CSS 匹配同步吃到。
> 回归测试 zz_r5272×2。**验证**：域 **59P/59F/18T**（+1 NotRun 为 CSP
> tentative 行与本修复无关）+ interactive 439P/44F + style-system lib
> **2449P/0F**（共享权威零回归）+ zero-dom lib **868P/0F**（+2）。账本见
> [evidence/2026-10-24-m2s65-http-equiv-content-language.md](html-semantics-compat/evidence/2026-10-24-m2s65-http-equiv-content-language.md)。
> **下一步**：text-level-semantics 开片（M1 基线 21P/13F），分诊轮先行。
> 余账全归档：color-scheme 23F + http-equiv-and-name-1 1F + 管道/应用面
> ~27F 渲染流、CSP tentative + parse-world 底座。
> **切片间隙**：本轮 dom crate（query.rs/lang_dir.rs）；下一片
> text-level-semantics 分诊轮不改引擎生产面，间隙开放，开工前查 oracle
> master 碰撞记录（2026-10-24 已查：渲染流在 ::first-line/::selection/
> paint 面，无 dom/shim 占用）。
>
> **▶ 上轮状态（2026-10-24 M2 切片 64 收口，document-metadata 分诊 + link/style 窄面轮）**：
> 三件事。**① color-scheme 归属核对（定档渲染流）**：10F 死因 =
> corpus 打包缺口（缺 support/compute-root-color-scheme.js，上游有）
> ——按「打包缺口本地补」先例恢复（gitignored 本地数据，**步进
> 52P/54F/18T → 52P/66F/18T** +12F 归因入账）；断言面 23F 塌缩单根
>（inline CanvasText 系统色 × color-scheme 解析 + colorScheme IDL +
> root 应用色）= applied-color → 渲染流记账回流。**② link/style 事件
> 族分诊（35F+18T 四类）**：资源管道 ~13F + applied CSSOM ~13F + CSP
> tentative 底座 7F+5T + parse-world 底座 1F 挂账；shim 窄面 7F 本轮
> 全收。**③ 修复面（R5271）**：①link.fetchPriority 枚举反射（合法
> ascii 小写、missing/invalid → auto，2F）；②link.relList supported
> tokens（link types 表 20 关键字，supports() 1F）；③style.sheet
> [SameObject] WeakMap 缓存 + 直接子指纹失效（有效子变更换新对象 /
> 空 fragment 身份保持）+ connected 门（不在文档返 null）+
> StyleSheet 基接口 brand（instanceof 可达，LinkStyle 2F）。**验证**：
> 域 **58P/60F/18T** + tabular 130P/4F（终版二进制）+ interactive
> 439P/44F + lib **3458P/0F**（CSSOM 契约零回归）。账本见
> [evidence/2026-10-24-m2s64-metadata-triage-linkstyle.md](html-semantics-compat/evidence/2026-10-24-m2s64-metadata-triage-linkstyle.md)。
> **下一步**：http-equiv-and-name 2F 分诊（本域 shim 面最后未分诊项）；
> 域收口后 text-level-semantics 开片（M1 基线 21P/13F）。余账全归档：
> color-scheme 23F + 管道/应用面 ~27F 渲染流、CSP tentative + parse-world
> 底座。
> **切片间隙**：本轮 part04/part06 JS shim；下一挂账 http-equiv 分诊
> 未定，间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 63 收口，document-metadata base 族轮）**：
> document-metadata 第二实施切片：**43P/63F/18T → 52P/54F/18T**
>（GAIN 9 / REG 0），the-base-element 13/13 全绿。**修复面
>（R5270 三刀）**：①`_zwDocBaseURL` scheme 排除——spec 文档基 URL
> 算法末步 data/javascript scheme **立即返 fallback**（非继续找下一
> base，base-data 案 1 期望 document.URL 实证）。②pending 增补 base
> 先扫——R161 归并把 pending 追加快照尾树序不保（head.prepend 新
> base 先于快照 base 评估，案 3 实证）；corpus 未钉 pending valid
> base 树序案，先扫近似记已知限制。③BASE href IDL（R5008 块内分
> 支）——缺失/空串 → 文档地址；解析失败 → **原属性串**
>（base_href_invalid corpus 钉定 "//test:test"，首版「失败 → 文档
> 地址」翻红回退修正）；LINK 保持 R5008 口径（harness resolveUrl
> 消费面）。**验证**：the-base-element 13/13 + tabular 130P/4F +
> interactive 439P/44F + lib **3456P/0F**（_zwDocBaseURL 共享面
> a/form action、baseURI、img src 零回归；+2 为 oracle 流新增）。
> 账本见
> [evidence/2026-10-24-m2s63-base-url-family.md](html-semantics-compat/evidence/2026-10-24-m2s63-base-url-family.md)。
> **下一步**：meta color-scheme 10F 归属核对（疑 style-system/渲染
> 面）；link/style 事件 30F+18T 分诊（runner file 通道能力上限）；
> style type/media + CSP tentative 待分诊。
> **切片间隙**：本轮 part03；下一挂账分诊未定，间隙开放，开工前查
> oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 62 开片，document-metadata title.text 族轮）**：
> 两件事。**① tabular-data 余 4F 分诊定档（不修）**：processing-model-1
> 四案全是表格布局 span 几何（colspan/col span=1000 字面列数摊宽、
> rowspan=0 不参与行高分布；offsetWidth/clientHeight 读），控制面排除
> 清单明确归 rendering-compat——本流收口定格 **130P/4F**，余案归因
> 挂账（DOM 语义面 IDL clamp 15 断言已全绿）。**② document-metadata
> 开片（M1 基线 31P/75F/18T → 43P/63F/18T，GAIN 12 / REG 0）**：
> Mission 序 tabular-data 后下一域，域内从未切片。**修复面（R5269，
> title.text 族 12F 全收）**：①title.text getter（part03）——child text
> content 直接 Text 子原样拼接（comment/元素子不含、无空白归一；
> title.text-01 断言 .text="TEXT"≠textContent）；sel 形 SetText 后融合
> 视图不载新文本——walk 空且 lw 非空才直取 latest-wins（appendChild
> 变更链 walk 非空自然排除，lw 不可直接作信号——title.text-01 首版
> 回退实证）。②title.text setter（part04）——R3907 option.text 同口径
> 委托 textContent。③document.title setter（part06）——改经元素
> textContent setter 落地（自带宿主写 + 本地文本子注册；旧裸
> __zw_set_text 只写宿主，本地文本子 stale 实证）。**验证**：
> the-title-element 13/13 + tabular 130P/4F + interactive 439P/44F +
> lib 3454P/0F（R3035 共享面零回归）。账本见
> [evidence/2026-10-24-m2s62-title-text-family.md](html-semantics-compat/evidence/2026-10-24-m2s62-title-text-family.md)。
> **下一步**：base 族 9F（HTMLBaseElement.href IDL + base 感知 URL 解析）；
> meta color-scheme 10F 归属核对；link/style 事件 30F+18T 分诊
> （runner file 通道能力上限）；style type/media + CSP tentative 待分诊。
> **切片间隙**：本轮 part03/04/06；下一挂账 base 族亦 shim/net 面，
> 间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 61 收口，insertRow tbody 目标 + XML 导入链轮）**：
> tabular-data 域第四实施切片：**127P/7F → 130P/4F**（GAIN 3 / REG 0），
> interactive-elements 全域 **439P/44F/0T** 复跑逐值不变，zero-engine
> lib **3454P/0F**。**修复面（R5268）**：①table.insertRow append 位
> （part03）——删 `hasSection` 分支（旧：rows 空 + thead/tfoot 存在时新
> 行**直挂 table**），统一「`_zwTblChildIdxLast` 最后 tbody，无则新建
> tbody 挂 table」——spec append 位 = 最后 tbody 元素子，thead/tfoot
> 不是插入目标（table-insertRow thead/tfoot 案 2F 同修）；连带多 tbody
> append 位从「最后有行的 tbody」纠正为「最后 tbody 元素子」。②快照
> 元素级 getElementsByTagName（part02）——`_zwParseEl` 缺方法（仅 doc 级
> 有），委托 querySelectorAll 一行补齐（tBodies.html XML documentElement
> 查询面）。③importNode 快照域产物物化（part06）——`_zwParseEl`/
> `__zwIsMEl` 源 deep import 经 `_wcRebuildAsHandle` 物化为 handle 树
> （t8q adoptNode 同判据同重建器；raw _zwMEl 副本 append 进活树后
> `.tBodies` 读 undefined 实证）；浅克隆与真 wrapper 源不进分支零变化。
> **验证**：probe 6 案全绿（已删）+ table-insertRow 3/3 + tBodies 1/1 +
> 两域复跑零回归。账本见
> [evidence/2026-10-24-m2s61-insertrow-tbody-xml-import.md](html-semantics-compat/evidence/2026-10-24-m2s61-insertrow-tbody-xml-import.md)。
> **下一步**：tabular-data 余 4F 全在 processing-model-1——rowspan-0×2
> （clientHeight 渲染门）+ col-span-limits×2（span 处理模型），待分诊；
> cells/tBodies HTMLCollection 类型一致化（无断言，低优先）。
> **切片间隙**：本轮 part02/03/06；下一挂账分诊未定，间隙开放，开工前
> 查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 60 收口，table.rows HTMLCollection 轮）**：
> tabular-data 域第三实施切片：**122P/12F → 127P/7F**（GAIN 5 / REG 0），
> interactive-elements 全域 **439P/44F/0T** 复跑逐值不变，zero-engine
> lib **3454P/0F**（+5 为 oracle 流新增测试）。**修复面（R5267）**：
> ①`_zwMakeHTMLCollection` 加 **rebuild 钩子**——[SameObject] 集合的 live
> 读重建形态：每次属性读重算成员表（排序/过滤天然最新），不注册写侧
> live 维护（无 matches 不进 `_zwLiveCollections`；与 matches 形态互斥
> 守卫，既有调用方零变化）。②rows getter（part03）裸数组 → 集合形态：
> `_zwRowsCollCache` WeakMap 缓存（R3908 selectedOptions 同口径，键 =
> 元素 proxy；rebuild 闭包持 sel/handle 原始值防自持环）+ rebuild 每读
> 重算 `_zwTableRowsAll`/`_zwTblChildAll`（s59 地基复用）。table-rows
> 断言面全落既有完备工厂：instanceof（`_zwHCPrototype` 链）、named
> getter（id 全元素 + name 仅 HTML-ns）、ownKeys（[indices…, names…]
> 集合序）、gOPD、canonical 索引；live 断言（removeChild 后 namedItem
> →null）由重建天然过。**范围界定**：仅 rows 收进集合；cells/tBodies
> 保持数组（类型一致化留挂账）；`_rowList`/`_cellList` 消费面经 trap
> 语义兼容零改动。**验证**：table-rows 5/5 全绿 + 连带面逐文件复核零
> 回归（remove-row/insertCell/rowIndex/sectionRowIndex/cells/cellIndex/
> createTBody/table-insertRow 持平）。账本见
> [evidence/2026-10-24-m2s60-table-rows-htmlcollection.md](html-semantics-compat/evidence/2026-10-24-m2s60-table-rows-htmlcollection.md)。
> **下一步**：tabular-data 余 7F——table-insertRow 2F（无 tr 时插入位
> 语义：spec **末 tbody**，现实现首 tbody）+ tBodies 1F（DOMParser XML
> documentElement.getElementsByTagName 缺）可并片；处理模型 4F 待分诊。
> **切片间隙**：本轮 part03/05；下一挂账亦 shim 面——间隙开放，开工前
> 查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 59 收口，tr 族索引/集合读侧轮）**：
> tabular-data 域第二实施切片：**94P/40F → 122P/12F**（GAIN 28 / REG 0），
> interactive-elements 全域 **439P/44F/0T** 复跑逐值不变，zero-engine
> lib **3449P/0F**。**根因（R5266）**：tr 族六 getter（rowIndex/
> cellIndex/sectionRowIndex/rows/tBodies/cells）旧实现经 selector 查询
> 三缺陷——detached script-created 子树不可解析恒空 / 匹配全部后代非
> 直接子 / 无 namespace 过滤。**修复面**：①part03 新地基四件——
> `_zwIsHtmlNsEl`（**严格** xhtml 串判定，探针实证 createElementNS("",
> x) 产物 namespaceURI 读 **null**、解析树/createElement 读 xhtml 串；
> 区别于 `_zwTblChildIdx` 宽松口径不动）+ `_zwTblChildAll` 直接子收集 +
> `_zwTableRowsAll`（spec rows 排序：thead 行 → 直接 tr + tbody 行 →
> tfoot 行）+ `_zwIdxOfSelf` 身份计位（childNodes 缓存 proxy 恒等）。
> ②六 getter 重写：rowIndex 门槛 **parent**（table）或 section+**grandparent**
> table（非最近祖先，table>div>tr 案 -1）；sectionRowIndex 父 table/section；
> cellIndex 父 HTML tr 直接 td/th 子；handle-only created 全覆盖。③has
> trap 名册（part05，R5249 tag-gate 模式）——`"cellIndex" in th` 等恒
> false 补六名。**连带收口**：remove-row 3F + thead/tbody/tfoot rows 3F
> + table-insertRow 1F（`_rowList`/`_cellList` 转读修好的 rows/cells）。
> **lib 测试契约修补**（part15 R3243 三测试）：rows/cells 读侧走
> childNodes 融合视图后，测试换代点只 bump 宿主代际未调 shim 侧挂钩的
> 缺口显形（跨场景 pending 桶残留，探针实证 len=5 含建行）——按
> part12/38/41 既有契约补 `__zw_reset_pending_state` + 两处同场景续跑
> 步先重播种 pristine 快照。**验证**：五目标文件 45 断言全绿；域
> 122P/12F/0T；interactive-elements 439P/44F/0T（REG 0）；lib 3449P/0F；
> fmt/clippy 干净。账本见
> [evidence/2026-10-24-m2s59-tr-index-family.md](html-semantics-compat/evidence/2026-10-24-m2s59-tr-index-family.md)。
> **下一步**（s60 已收口）：table-rows 5F 已清；余挂账 table-insertRow
> 2F / tBodies 1F / 处理模型 4F / cells-tBodies 类型一致化。
>
> **▶ 上轮状态（2026-10-24 M2 切片 58 收口，tabular-data create/delete + setter 族轮）**：
> tabular-data 域首个实施切片：**67P/67F → 94P/40F**（GAIN 27 / REG 0），
> interactive-elements 全域 98 files **439P/44F/0T** 复跑逐值不变。
> **修复面三件（R5265）**：①table create/delete 派生族（part03）——
> createTBody 插**最后一个 HTML-ns tbody 之后**（createTBody.html 15 案
> 钉定，嵌套不作锚）、createTHead 有守卫+before tbody/tfoot、createTFoot
> 有守卫+append、createCaption 有守卫+首子、delete 三族删首匹配直接子；
> 门统一 `_realTag` 后缀取 `.split(':').pop()`（createElementNS 前缀案
> 放行）。②caption/tHead/tFoot getter 收紧（`_zwTblChildIdx` 精确索引 +
> 非 HTML ns 排除）。③IDL setter 族（part05 set trap 新分支 + part04
> set trap 补 receiver）——null 删首匹配、非 section/caption 形态
> TypeError、localName 不符/祖先环 HierarchyRequestError、caption 插
> **首子**（caption_001 firstChild 钉定）、thead/tfoot 有既有先删后原位
> 插（防 selector 重解析误删同形新子）。**本轮 bug 两个同 commit 修**：
> caption 插入段困嵌套 if/else 末臂永不可达（探针入口插桩实锤
> setter 路径从未调 insertBefore；修=卫语句+平铺共享段）；createTBody
> 位置误诊为包装身份断裂（同型元素 identity 断言失败优先怀疑顺序——
> 12 探针逐因子排除后插桩抓 `at:0` 实锤）。**验证**：目标案全绿
>（createTBody 15/15、caption_001 5/5、tHead 3/3、tFoot 2/2）；全域
> interactive-elements 439P/44F/0T（REG 0）；zero-engine lib 3448P/0F。
> 账本见
> [evidence/2026-10-24-m2s58-tabular-create-delete-family.md](html-semantics-compat/evidence/2026-10-24-m2s58-tabular-create-delete-family.md)。
> **下一步**（s59 已收口前两见）：tr 族 21F 已清；余挂账 table-rows
> 5F / remove-row 3F + section rows 3F / insertRow / tBodies 1F / 处理
> 模型 4F。
>
> **▶ 上轮状态（2026-10-24 M2 切片 57 收口，selectAll inert 作用域 + 钉案轮）**：
> 45F 全量复核（逐案读测试源钉断言机制）+ D 族唯一非几何门案收口
>（R5264），全域 438P/45F/0T → **439P/44F/0T**（GAIN 1 / REG 0）。**方案
> 取舍**：s46 五桶分组沿用并逐案核对——`inert-node-is-unselectable`
> 断言链 `document.execCommand('SelectAll')` →
> `getSelection().toString()`，此前 execCommand selectAll 未接（permissive
> no-op → toString 恒 ''），且模态 inert 遮罩是**纯 DOM 树行走**（区别于
> D 族其余案的 gBCR/elementFromPoint 几何门）。**修复**（part06）：
> execCommand 新增 selectall 分支——`_zwDialogModalTopSel()` 命中 → 栈顶
> 模态 dialog 子树，否则 body；复用 `_selection.selectAllChildren`（R2804
> 面）；不派 beforeinput/input（选区命令不经编辑宿主事件序，与 undo/redo
> 同口径）；`_zwQueryCommandState` 补真实反射。**钉案成果**（跨流协调
> 交付物）：余 44F = 渲染流门 ~21F（abspos 居中 6 / default-color 2 /
> overlay 2 / top-layer-position 1 / non-modal 1 / multiple-centered 1 /
> scroll-height 1 / visibility-hidden UA 1 / bounds-clicking 2 /
> modal-selection 拖选几何 1 / dialog-inert 1 / ancestor-is-inert 1 /
> inert-svg-hittest 1 / showModal top-layer 绘制序 1 / closed-details-
> layout-apis 1）+ shadow 底座 20F（closedby-complex 16 + focus-previous-
> outside 3 + form-submit-shadow 1，declarative shadow 待用户点名）+
> corner-cases ESC 1F（s53 定性 divergence 不弯就）。**验证**：目标
> 1F→1P；全域 439P/44F/0T；zero-engine lib 3448P/0F；对照面
> not-highlighted（reftest 视觉）/ with-modal-dialogs-and-iframes /
> 两 crash 测试（语料不载）零波及。账本见
> [evidence/2026-10-24-m2s57-selectall-inert-scope.md](html-semantics-compat/evidence/2026-10-24-m2s57-selectall-inert-scope.md)。
> **下一步**：本流 shim 面清空——跨流协调（渲染流按名取案，飞书已通知）
> + declarative shadow 联合切片待用户点名。**切片间隙**：本轮零 .rs 变更
>（仅 JS shim），下一挂账开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 56 收口，light dismiss 两腿轮）**：
> s49 定性「指针管线深水」的
> dialog-light-dismiss-pointer-capture.html（touch/mouse 双腿）**2F → 2P
> 全收口**（R5263），全域 436P/47F/0T → **438P/45F/0T**（GAIN 2 / REG 0
> / Timeout 0）。**方案取舍**：s49 三洞试错后回退定性指针管线深水；本轮
> 复核推翻前提——管线 capture 重定向面已全通（part04 setPointerCapture
> pending override + part06 `_zwProcessPendingCapture` up 派发前换防 +
> 尾簇 12 换防后取 capturedSel），真缺口唯一：light dismiss 决策仍是
> R4300 单腿（pointerdown 即关），capture 无从影响。spec 已演进两腿
>（down 记 nearest clicked dialog 祖先，up sameTarget 比对后关），实现
> 以**两态分类**等价替换身份模型：down `_zwDialogLdPick` 反文档序首个
> open+any 定类（'in' 链内 / 'out' 链外）存槽，up
> `__zw_dialog_light_dismiss_up` 重分类同态比对，相等且 'out' 才关
>（选择器保持 R5234 口径）；全腿组合等价（in/in、out/in、in/out 不关，
> out/out 关）——capture 面（down 'out' + capture 把 up 重定向 dialog 内
> 'in'）由此不关门。槽卫生：down 终态必写槽（早退也清）+ up 读取即清。
> popover pass 维持 down 腿单发（R5234 绿面零变化）。**验证**：目标
> 2F→2P（pointerId 全管线默认 1 与 `st.capture['1']` 对齐实证）；全域
> 438P/45F/0T；part60 R4 契约两腿化 + out→in capture 门腿（r4d）+ R6
> down+up 对（r4d 首轮 fallback `'button'` 命中 l1 首按钮误关，改
> `'#inner3'` 唯一 id——契约侧 bug）；zero-engine lib 3448P/0F；
> fmt/clippy 清。账本见
> [evidence/2026-10-24-m2s56-dialog-light-dismiss-two-leg.md](html-semantics-compat/evidence/2026-10-24-m2s56-dialog-light-dismiss-two-leg.md)。
> **下一步**：B 族清空，挂账序——C 族渲染流 20F 跨流协调（归渲染流）；
> declarative shadow 联合切片待用户点名。**切片间隙**：本切片为 engine
> shim 面（part01/part06），下一挂账开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 55 收口，xhr-response graft 轮）**：
> s54 分诊轮挂账的 xhr-response 1F **窄域收口**（R5262），F 族
> name-attribute.html 三案全清（**18P/0F**），全域 435P/48F/0T →
> **436P/47F/0T**（GAIN 1 / REG 0）。**方案取舍**：s54 定性 parse-world
> 基面三件套升级（波及 DOMParser/XHR/responseXML 全语料）vs 底座挂账；
> 本轮复核出第三条窄域路线——场景需求全落 `xhr.response.body` 单节点，
> 而 `_zwParseEl` 的 R3019 mutTree 桥（`_ensureMutTree`）本就把该节点惰性
> 物化为 wrapper-world `_zwMEl` 实体子树，仅差物化时机（惰性→主动）、
> 接口原型接线（裸 `_zwMEl` 不接线，R125 在 `_zwMBuildNode` 只覆盖解析
> 子）、`doc.body` 指向（仍是快照壳）。**修复**（part03 XHR HTML 文档
> 分支）：body `_ensureMutTree()` 主动物化 + R125 配方接线
>（`__zwHtmlTagIface.body` → `HTMLBodyElement.prototype`）+ `doc.body`
> 换实体节点。读语义零变化（旧 firstChild 等读面本就惰性经同一 mtree）；
> DOMParser 语料零波及（graft 仅 XHR `responseType='document'` HTML MIME
> 分支，`_zwParsedDoc` 工厂与 `_zwParseEl` 基面未动）；head/documentElement
> 保持快照面（场景不触及）。wrapper-world 实体面经 s54 探针全链验证
>（innerHTML 写解析接 R4100、qSA 返真实 parentNode 链完整节点、open IDL
> + toggle 队列全通）。**验证**：name-attribute 17P/1F → **18P/0F**；
> xhr 语料 stash 前后双侧同跑 net-api xhr 切片 **580P/14F/21T 逐位一致**
>（基线二进制 R5262 锚点 0 / graft 后 1 双验）；escaping.html
>（html-syntax，XHR document 读面消费方）4 子测试全 Pass；zero-engine
> lib 3448P/0F；探针页已删。账本见
> [evidence/2026-10-24-m2s55-xhr-body-graft.md](html-semantics-compat/evidence/2026-10-24-m2s55-xhr-body-graft.md)。
> **下一步**：F 族清空，续挂账序——B 族 pointer-capture 2F 分诊（s49
> 定性指针管线深水）或 C 族渲染流 20F 跨流协调；declarative shadow
> 联合切片待用户点名。**切片间隙**：本切片为 engine shim 面（part03），
> 下一挂账开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 54 收口，name-attribute 3F 分诊轮）**：
> F 族 name-attribute.html 三案分诊完成——**2F shim 收口 + 1F 底座缺口
> 挂账**，全域 433P/50F/0T → **435P/48F/0T**（GAIN 2 / REG 0）。
> **R5260**（mutation event and toggle event order → Pass）：R4009 toggle
> 事件任务队列旧 in-place 去重使 e0 解析期合并事件留队首；spec「queue a
> details toggle event task」合并语义为**移除已排 task + 重排队尾**（
> oldState 取首排程 pre-state）。修复换新 entry 对象 + old 承接
> prevEntry.old——旧 timer 经 fire 闭包身份守卫（`!== entry`）自动作废，
> 兼容 part81 `__zw_setTimeout` stub 环境；spec 推演 FIFO [e3,e2,e1,e0]
> 对齐测试期望。**R5261**（connected-in-template → Pass）：plain wrapper
> 模板 content 视图（part03 `_zwContentView`，模板 childNodes 内联存储
> 形态）缺 ParentNode 元素族 getter，`content.firstElementChild` 恒
> undefined、setup 断言即败；part04 `_tplContent`（sel/handle 形态，
> R151）有全套故 `template-in-disconnected` 本来就过。补
> firstChild/lastChild/children/firstElementChild/lastElementChild/
> childElementCount 六 getter（R151 同款形状）；探针全链：inner
> instanceof HTMLDivElement、innerHTML 解析互斥 [1,0]、qSA 全通。
> **xhr-response 1F 挂账**：`xhr.response.body` 是 `_zwParseEl` 快照（
> protoChain `_zwParseEl > Object > null` 无接口原型），场景链需三件套
> 探针逐步实证——①instanceof HTMLBodyElement（R130 setPrototypeOf 会剪
> 掉 `_zwParseEl.prototype` 全部方法面）；②innerHTML 写解析（ctor own
> data property 赋值仅覆写字符串，R3019 mutTree 桥只读 rewire 无
> setter）；③open IDL（qSA 经 `_zwWrapQ` 返回 `_zwParseEl`，`e.open`
> undefined）。定性 parse-world 写面切片，波及 DOMParser/XHR/responseXML
> 全语料，单独立项排期。**验证**：name-attribute 单案 15P/3F →
> **17P/1F**；zero-engine lib 3448P/0F（part60 R4100 契约 per-element
> 形态不断言跨元素序，重排队零回归）；探针页已删；fmt/clippy 不适用（
> diff 仅 JS shim）。账本见
> [evidence/2026-10-24-m2s54-details-name-3f-triage.md](html-semantics-compat/evidence/2026-10-24-m2s54-details-name-3f-triage.md)。
> **下一步**：parse-world 写面三件套切片（接口原型双链保留 / innerHTML
> setter 进解析 / open IDL）排期，或续挂账序 B 族 pointer-capture 2F、
> C 族渲染流 20F；declarative shadow 联合切片待用户点名。**切片间隙**：
> 本切片为 engine shim 面（part03），下一挂账开工前查 oracle master
> 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 53 收口，零代码轮）**：corner-cases
> 「focusin calls showModal, modeless」ESC 1F 复审定性——**spec 一致行为，
> 测试期望分歧**。四重证据同向（ESC 后 dialog 重开 open=true）：live
> spec close-the-dialog 恢复门（anchor 后代 OR wasModal → focusing steps
> → focusin → 处理器 showModal；showModal 前置守卫无一命中、无重入门）；
> Chromium main 源码（Close() 焦点恢复在 is_closing_ 释放后、showModal
> 无守卫）；直探 Chromium 151 与 **Chrome for Testing 158.0.8118.0
>（wpt.fyi 同 build）**真 ESC——皆重开；我方 runner 探针序列与直探逐步
> 同构。**反证与矛盾**：wpt.fyi 同 revision（c645564b0e，2026-10-23
> run）chrome-158 + firefox-161 summary 双双 30/30 绿（压缩格式已验证
> c=[passes,total]）——与同 build 直探矛盾只能落 wptrunner 环境层（页面
> 可见语义内构造不出 closed 读点世界）；s41 依据「Chrome-parity 同红」
>（crbug 417571176，2026-04）已被双引擎绿取代（该 crbug 的 CL 即 wpt
> #58337 断言形式修复）。**收口**：1F 保留标 spec-consistent known
> divergence，不弯就（s41 R5244 试错实测 REG 4 + 违背 spec/直探）；
> 全域 433P/50F/0T 零变化。账本见
> [evidence/2026-10-24-m2s53-corner-cases-esc-divergence.md](html-semantics-compat/evidence/2026-10-24-m2s53-corner-cases-esc-divergence.md)。
> **下一步**：F 族 name-attribute 3F 分诊（mutation events / XHR 解析 /
> template 附着底座缺失——评 shim 可收口面 vs 底座缺口）；B 族
> pointer-capture 2F（s49 定性指针管线深水）、C 族渲染流 20F 维持跨流
> 挂账；declarative shadow 联合切片待用户点名。**切片间隙**：本轮零代码
> 未触 engine 生产面，间隙维持开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 52 收口）**：R5257 focus 滚动进视口
> + R5258 moveBefore——**interactive-elements 文件级 Timeout 清零**（50F
> 全为明确 Fail）。**分诊**（本轮刀前）：shim 面红名单三案——
> focus-after-close 文件级 Timeout（6/7 Pass 悬挂末腿）、
> movebefore-setup 1F（`new_parent.moveBefore` TypeError）、showModal 1F
> （rendering on top——top-layer 绘制序渲染流面，转 C 族记账）。**R5257**
>（focus-after-close Timeout → 7 子测试全 Pass）：探针双根因——proxy
> focus() 无滚动步（spec focusing steps 尾 UA 滚动面）+ 视口 scroll 事件
> 只派 window（document 监听永收不到）。三修：①focus 尾 nearest 滚动
>（gBCR 出界才滚，R3047/R3051 面复用）；②几何 fallback——动态创建元素
> 无布局几何（runner gBCR 零盒实证），读 proxy inline style（handle-only
> 可达）position 非 static + top 可解析 → 文档 y 顶对齐（FIXME: 样式表
> 定位随渲染流底座）；③no-scroll 变体 `__zwFocusNoScroll` 旗——R4600
> dialog 内部 focusing steps（show/showModal/close 恢复/fixup）包旗不滚
>（Chromium close 恢复不滚，同测试 scrollTop==0 断言面）；④`_zwFireScroll`
> 补 document 级派发（非冒泡 bubble 腿门控，window 不双计）。**R5258**
>（movebefore-setup 1F → 1P）：Element.moveBefore = WebIDL 校验 + 委派
> insertBefore 机械——免 removal steps 语义（shim 移动面本无 dialog 删除
> 钩子，组栈/open 态天然原地保持；FIXME: 跨文档同文档门随 parser 批次）；
> ESC 断言链走既有 R5236/R4300 组栈序。**契约**（part83）：nearest 滚动
> 位 + 双通道恰一次/showModal-close 零滚动 + 焦点恢复/moveBefore open 态
> 保持 + ESC 关组栈顶/NotFoundError 零滚动。**验证**：focus-after-close
> **Timeout → 7 全 Pass**、movebefore-setup **1F → 1P**；全域 98 files
> 483 subtest **433P/50F/0T 对 s51 逐位 REG 0 / GAIN 3**；zero-engine lib
> 3448P/0F（+契约 part83）；fmt/clippy clean（test-guard 包裹）；release
> runner 含 R5257×4/R5258×1 锚点。账本见
> [evidence/2026-10-24-m2s52-focus-scroll-movebefore.md](html-semantics-compat/evidence/2026-10-24-m2s52-focus-scroll-movebefore.md)。
> **下一步**：declarative shadow 联合切片仍待用户点名（渲染流协调）；
> 挂账序续 C 族渲染流 20F（top-layer 绘制序 showModal 1F / modal-selection
> 1F / inert 族 4F——elementFromPoint/hit-test 缺口）与 B 族指针管线深水、
> F 族 name-attribute 3F 逐案分诊。**切片间隙**：本切片为 engine shim 面
>（part01/04 + 契约 part83），下一挂账 C 族为渲染流面需跨流协调，开工前
> 查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 51 收口）**：R5256 dialog 属性移除面
> + A/H 分诊改记。**分诊**（本轮刀前）：A 族 closedby×slot 16F（
> dialog-popover-closedby-complex 全部「shadow DOM slots」变体）与 H 前置
> 4F **同根**——探针实证 `<template shadowrootmode=open>` 在页面视图被
> 拆包平铺（part03:7639「parser 占位」互证）、shadowRoot null、shim
> attachShadow 虚拟树不下传 dom（attach_shadow 生产零接线）、shadow 内容
> rect=0/hit-test null（几何面未通）；clickOn 坐标驱动下 shim 虚拟单边
> 提升会 REG 现存 4 Pass（点击落 (0,0) 全员误关）。**改记**：A+H 统一
> 「declarative shadow parser 提升 + shadow 几何渲染流联合切片」，渲染流
> 半边维持用户决策门禁（web-components 归档流明示等点名协调
> rendering-compat；与 parser form pointer 缓行决策同域）。**R5256 两刀**
>（dialog shim 面）：①open 属性移除面关闭步——`_zwDialogAttrClosedR5256`
>（part01）清模态/组栈 + close/toggle 排队，无 cancel/无 beforetoggle/
> 焦点不恢复/returnValue 不变（tentative 断言契约；旧注记「不派 close」
> 已被 2025 attribute change steps 变更推翻）；removeAttribute 尾钩 +
> IDL falsy 臂双接通，**close watcher 门**（pre-snapshot 模态态/组栈双
> 信号——属性直开无 watcher 仅隐藏，首版 REG dialog-open 一案当场修正）；
> ②detached 文档 dialog 三方法——showModal 抛 InvalidStateError、show
> 静默返、close no-op（dialog-active-document 旧产物 TypeError 面）。
> **契约**（part82）：模态移除/IDL falsy/wasOpen 门/同任务 toggle 合并/
> detached 三方法/属性直开零 close 六腿。**验证**：close-via-attribute
> **2F → 2P**、active-document **1F → 1P**、dialog-open 维持 3P；全域
> 98 files 483 subtest **431P/51F/1T 对 s50 逐位 REG 0 / GAIN 3**；
> zero-engine lib 3447P/0F（+契约 part82）；fmt/clippy clean（test-guard
> 包裹）；release runner 含 R5256 锚点。账本见
> [evidence/2026-10-24-m2s51-dialog-attr-removal.md](html-semantics-compat/evidence/2026-10-24-m2s51-dialog-attr-removal.md)。
> **下一步**：declarative shadow 联合切片待用户点名（渲染流协调）；续
> dialog shim 面红名单（focus-after-close 1T / showModal 1F /
> dialog-inert 1F 等逐案分诊）。C 族渲染流 20F 与指针管线维持跨子系统
> 单独排期。**切片间隙**：本切片为 engine shim 面（part01/03/04），
> 下一挂账预期仍 shim 面或待决策项，间隙开放，开工前查 oracle master
> 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 50 收口）**：toggleEvent 1T 收口
> （R5254 parser 初始 open details toggle 补偿两段式）。**分诊**：
> toggleEvent.html 文件级 Timeout——11 async_test 中 9 Pass、pending: 2
> 永挂（t9 依赖 inline ontoggle 在 parser 声明 open 时 resolve promise；
> 末案 DOMParser 产物 ontoggle）。根因：R4009 toggle 队列只挂写路径尾钩
>（IDL setter / setAttribute / removeAttribute），parser 初始 open 由
> host 解析直建不经 JS 写路径；innerHTML / createElement-HTML / 包装
> 世界三解析面已接 R4100 扫描，**唯主文档初始 HTML 与 DOMParser 快照面
> 缺**。**两段式**（单段直扫两难均实证：注册期直扫 task 派发早于页面
> 脚本——runner 面 arm 到真线程 timer，t9 resolver 未就位永挂；DCL 直
> 扫分不清 parser open 与运行时 open——activation-behavior coalesced 案
> toggle 计数 2≠1 回归）：注册期 mark（`__zwDetailsInitOpenMark5254`，
> 页面脚本前文档纯 parser 态，open 者身份 key 入集）+ DCL fire
>（`__zwDetailsInitOpenFire5254`，capture listener，只对「初始 open 且
> 当前仍 open」走既有 `_zwToggleQueueNotify4009` 入队，标记集一次性消
> 费）；`__zw_reset_form_state` 尾清集重标记（跨导航）。**刀二
> DOMParser**：快照架构死穴——扫描快照 ≠ 用户 querySelector 手持快照，
> 实例属性 handler 不可达（首版直调计数 0 实证）；改 parseFromString
> 返回前扫 `details[open]`（打 open IDL getter + setTimeout task）+
> `_zwParseEl.prototype` ontoggle accessor 按身份键（`_zwEvKey`，同源
> 重解析稳定）登记 `_zwParsedToggleH` 全局表，task 经身份键寻址派发。
> **契约**（part81）：(a) 主文档初始 open 恰好一次 toggle（closed→open
> + target.open 真值）+ 同态写不重派（t9 断言面）；(b) 初始 closed 不
> 派；(c) DOMParser 产物收到 toggle + target.open 真值；(c2) 无 open
> details 零 task。时序模拟：记录式 `__zw_setTimeout` stub + 手动
> `__zw_fire` + 手动派 DCL。**验证**：toggleEvent.html **Timeout → 11
> 子测试全 Pass**；activation-behavior coalesced 案两段式回绿；全域
> 98 files **483 subtest 428P/54F/1T 对 s49 逐位 REG 0 / GAIN 2**
>（fail 集逐位一致，子测试总数 482→483 为 Timeout 不再吞 2 案）；
> zero-engine lib 3446P/0F（+契约 part81）；fmt/clippy clean
>（test-guard 包裹）；release runner 含 R5254 锚点（grep 实证）。
> 账本见
> [evidence/2026-10-24-m2s50-parser-open-toggle.md](html-semantics-compat/evidence/2026-10-24-m2s50-parser-open-toggle.md)。
> **下一步**：按挂账序候选 A 族 closedby×slot 16F（Chrome-parity 同红
> 钉住，slot 分配 shim 面）或 H 前置 4F（declarative shadow parser +
> 渲染流联合切片）；F 族 name-attribute 3F 与 DOMParser 快照架构
>（live 文档面）维持挂账。**切片间隙**：本切片为 engine shim 面
>（part01/02/03 + callbacks.rs 注册尾），下一挂账 A/H 均预期跨
> parser/shim/渲染流，间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 49 收口）**：B 族 light dismiss 5F 全挂账
> 零净变更 + F 族分诊 + R5253 收口 details auto-expand 1F。**B 族定性**：
> bounds-clicking 2F——探针实证 elementFromPoint 对 dialog 自身盒命中返
> null（modal+modeless 同断、后代正常），渲染流命中缺口并 C 族记账；
> pointer-capture 2F——R5252 两腿模型（down/up 双 target 判定 + capture
> retarget 救回）试错三洞（backdrop retarget 使 down 腿 contains=true 短
> 路漏记、capture 残留未隐式释放误救回、corner-cases modal 组入口永挂），
> git 回退定性**指针管线深水**，需先有指针序列契约面再动；corner-cases
> 1F 既有挂账（modeless ESC 竞态）。**F 族分诊**：closed-details-layout-
> apis 1F 渲染流面；name-attribute 3F（mutation events / XHR 解析 /
> template 附着底座未实现）；toggleEvent 1T 文件级 Timeout（9 子测试
> Pass 后 completion 未触发，未深查待分诊）。**R5253 两刀**（part02
> `_scrollToAnchorForHash`）：几何量测前插 spec 展开步（target 链逐个
> setAttribute('open','')，toggle/name 组链自动）+ 仅真展开时挂 rAF 补偿
> 臂（属性 mutation 批应用后重入：展开步 no-op + 既有滚动按新几何落位，
> 既有滚动面零变化）。**契约环境差异注记**：bridge 契约 sandbox 无帧驱
> 动/布局供给面，滚动落位不可建模（帧驱动 + 手动 tick 仍红）——契约钉
> (a) 展开/(b) 链外不误开/(c) no-crash，落位面由 WPT 单案承载。**验证**：
> auto-expand 单案 **Fail → Pass**；interactive-elements 全域 98 files
> 482 subtest **426P/54F/2T 对 s48 逐位 REG 0 / GAIN 1**；zero-engine
> lib 3445P/0F（+契约 part80）；fmt/clippy clean（test-guard 包裹）；
> release runner 含 R5253 JS 串锚点。账本见
> [evidence/2026-10-24-m2s49-fragment-details-expand.md](html-semantics-compat/evidence/2026-10-24-m2s49-fragment-details-expand.md)。
> **下一步**：按挂账序候选 A 族 closedby×slot 16F（Chrome-parity 同红
> 钉住）或 H 前置 4F（declarative shadow parser+渲染流联合切片）；
> toggleEvent 1T 分诊可作小切片插入。C 族渲染流 20F 与指针管线两腿模型
> 需跨子系统协调，单独排期。**切片间隙**：本切片为 engine shim 面
>（part02 fragment 导航），下一挂账 A/H 均预期跨 parser/shim/渲染流，
> 间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 48 收口）**：D 族 focus/inert 10F
> 逐案单跑取证后重分诊——可收口面仅 focusability 1F（R5251）。**重
> 分诊**：focus-previous-outside 3F + form-submit-dialog-shadow 1F →
> **挂账（H 族前置）**——首层根因是 `<template shadowrootmode=open>`
> 声明式 shadow root 从未被提升（`host.shadowRoot` 返 null，
> `querySelector` 即炸），shim 全 part 无 shadowrootmode 处理
>（attachShadow 命令式已有），4F 记入 H 族「parser+渲染流联合切片」
> 范围，R4600 焦点恢复 shadow/slot 边界是其后续面；dialog-inert 1F +
> modal-dialog-ancestor-is-inert 1F + inert-svg-hittest 1F +
> inert-node-is-unselectable 1F + modal-dialog-selection 1F → **挂账
>（渲染流面）**——elementFromPoint 对 open dialog 返 null / 点击穿透
> inert 祖先 / 选择窗口空串，同底座 = 命中测试几何链缺失，与 C 族
> top-layer/UA 布局 15F 合并记账。**根因**（focusability）：Tab 可聚焦
> 判定两处均无 editing host 分支——runner `dispatch_tab_focus` 页面候选
> 序选择器不含 contenteditable 宿主（本案观测路径，testdriver
> send_keys TAB）；host `FocusManager::scan_node` 同缺口（MoveFocus 效果
> 面）。spec：editing host 是 focusable area（HTML §focusable-area）。**落
> 地（R5251 两刀）**：runner dispatch_tab_focus 候选选择器尾补
> `[contenteditable]` + host 门（`isContentEditable`（R3187 枚举态）真
> 且父不可编辑 → 入零序；嵌套 editable 非 host / `contenteditable=false`
> 孤岛不入序；显式 tabindex 优先原臂，原生可聚焦元素落原 else 臂——首版
> 误让 button 落 CE 臂 continue 掉，回归当场修正）；dom FocusManager 补
> `contenteditable_state`/`is_editable`/`is_editing_host` 三 helper +
> scan_node 零序分支。**验证**：dialog-focusability 单案目标子测试
> **Fail → Pass**；interactive-elements 全域 98 files 482 subtest
> **425P/55F/2T 对 s47 逐位 REG 0 / GAIN 1**；zero-dom lib 866P/0F
>（+3 新单测，fail-first 先行）；zero-engine lib 3444P/0F（+契约
> part02 R5251）；fmt/clippy clean（test-guard 包裹）；release runner
> 含 R5251 JS 串锚点。账本见
> [evidence/2026-10-24-m2s48-editing-host-focusable.md](html-semantics-compat/evidence/2026-10-24-m2s48-editing-host-focusable.md)。
> **下一步**：按挂账序转 B 族 light dismiss 5F 分诊（invoker/backdrop
> 面）；H 前置 4F 与渲染流面 5F（并入 C 族记账）维持挂账；parser form
> element pointer 4F **已缓行**（2026-10-23 用户决策：与 declarative
> shadow DOM（H）打捆 parser 批次、M2 收口后尾段单独立项）。
> **切片间隙**：本切片为 runner 页面序脚本面（testharness.rs）+ dom
> FocusManager 面（crates/dom）——下一挂账 B 族 light dismiss 预期
> engine shim 面（part01 dialog 事件），间隙开放，开工前查 oracle
> master 碰撞记录。
>
> **▶ 上轮状态（2026-10-23 M2 切片 46 收口）**：余 60F 归因分诊落档（六族
> 地图——A closedby×slot 交叉 16F（crbug 417571176 Chrome-parity 同红）、
> B light dismiss 5F、C top-layer/UA 布局 15F（渲染流面）、D focus/inert
> 10F、E dialog API 语义 5F（可 shim 收口候选）、F details 零散 4F/1T +
> G 历史 menu 面 3F）+ 族 G 顺手收口（R5249）。**根因**（三半边「IDL 名
> 无 tag 门」同模式）：①part05 has trap track IDL 列 label/kind/srclang/
> default（media 切片 3 加）无条件命中——历史 menu/menuitem（menuitem 已
> 删为 HTMLUnknownElement）`'label' in el` 假阳性；②set trap R3069 臂
> FLAT 表 type 无 tag 排除——`menu.type=` 误写内容属性；③同臂 label 显式
> 列（R3069-fix）无 tag 门。**落地（R5249 双刀）**：has trap kind/srclang
> 收 TRACK 门、label/default 收 TRACK+OPTION 门（HTMLOptionElement 读侧
> R3069-fix 对称）；R3069 臂条件首项 MENU/MENUITEM 负门——赋值落 expando、
> 内容属性不变，name 臂自带门与 FLAT 其余名零变化。**验证**：
> contextmenu-historical 单案 **11/11 Pass**；interactive-elements 全域
> 98 files 482 subtest **423P/57F/2T 对 s45 逐位 REG 0 / GAIN 3**（GAIN
> 即三案）；lib **3442P/0F**（+2 契约 part78）；契约翻转力实证（tag 读恒
> TRACK → 钉 1(a) 红；负门短路 → 钉 1(c) 红）。**已知边界**：menu.type/
> label 读侧 FLAT 读仍返内容属性值（真浏览器 undefined，测试无此面）；
> track 接口其余成员 in 仍无条件（corpus 无 menu 面观测）。gates：clippy
> --workspace -D warnings（test-guard 包裹）/fmt clean、零新增依赖。账本
> 见 [evidence/2026-10-23-m2s46-historical-menu-idl-gate.md](html-semantics-compat/evidence/2026-10-23-m2s46-historical-menu-idl-gate.md)。
> **下一步**：按挂账序 E 族 dialog API 语义 5F（close-via-attribute 移除
> open 属性触发 closing/close 事件 2F 为首靶，active-document/showModal
> throw/movebefore 跟进）；parser form element pointer 4F **已缓行**
>（2026-10-23 用户决策：与 declarative shadow DOM（H）打捆 parser 批次、
> M2 收口后尾段单独立项）；declarative shadow DOM 同打捆待触发。**切片
> 间隙**：本切片为 shim 面，下一挂账 E 族亦 shim 面——间隙开放，开工前查
> oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-23 M2 切片 45 收口）**：s37 已知边界「点击
> summary-less details 内非激活后代不翻转」收口（2026-10-23 重新定性：
> spec 无 summary 翻转是 UA affordance 非 MUST，corpus 无观测面——性质
> 同 s42/s43：spec 正确性收口 + 契约钉住）。**根因**（通道分裂）：details
> 激活面两条通道各断半边——①`.click()` 合成通道 part04 click 默认动作段
> 只处理 receiver 自身三站（LABEL/SUMMARY/DETAILS），非激活后代无 nearest
> 上溯；②JS dispatch 通道 part03 R112 post act:details 门
> `_zwUaDispatch === true` 挡 `new MouseEvent`（无 UA 印记、无 synthetic
> 标）。**落地（R5248 双刀）**：part04 nearest 上溯分支——receiver 非
> 三站且 `_zwFindClickActivation` 命中 act:details 时
> `_zwDetailsActivateR4400` 翻转（spec the-details-element activation
> behaviour + legacy-pre-activation nearest ancestor (or self) with
> activation behavior；notPrevented 门内；receiver 三站让位防双翻；交互
> 后代 act:button/act:a/act:summary/checkbox 命中时 type 豁免自然不落）；
> part03 门放宽 `(_zwUaDispatch === true || _zwSyntheticClick !== true)`
> ——JS dispatch 进同款翻转，`.click()` 合成标保持不进（单翻互斥），
> preventDefault/connected 门由消费块入口守卫继承；hasSum 门槛让位不变
> （summary 站优先，R4009 面不回归）。**验证**：探针七案修后全翻且单翻
> 互斥成立；interactive-elements 全域 98 files 482 subtest **420P/60F/2T
> 对 s44 逐位 REG 0 / GAIN 0**（R4400 既有面无回归，corpus 无观测面无
> GAIN）；lib **3440P/0F**（+2 契约 part77）；契约翻转力实证（nearest
> 分支短路 → 钉 1(a) 红；门收紧 → 钉 1(c) 红）。**已知边界**：corpus 无
> 观测面（无 GAIN）；detached details 后代点击不翻转（R112 connected
> 门，spec 无条件但 corpus 无观测面）维持现状。gates：clippy --workspace
> -D warnings（test-guard 包裹）/fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s45-summaryless-details-descendant-activation.md](html-semantics-compat/evidence/2026-10-23-m2s45-summaryless-details-descendant-activation.md)。
> **下一步**：按挂账序其余余案归因；parser form element pointer 4F
> **已缓行**（2026-10-23 用户决策：与 declarative shadow DOM（H）打捆
> parser 批次、M2 收口后尾段单独立项）；declarative shadow DOM 同打捆
> 待触发。**切片间隙**：本切片为 shim 面，下一挂账开工前查 oracle
> master 碰撞记录（间隙开放）。
>
> **▶ 上轮状态（2026-10-23 M2 切片 44 收口）**：s39 剩余 1F「focus-after-close
> Complex dialog usage focus 恢复红（activeElement 得 div）」收口。**根因**
> （六层钻透）：Complex 案跨 testdriver 多命令 execute_script，宿主 apply
> 换代发生在 show 与 close 之间——①静态树节点 handle 形 proxy 无
> `_zwNodeParent` 反链记账（R91 只记 JS 动态操作）→ ②isConnected 反链
> miss + rect 探测兜底空 → 双假阴性 disconnected → ③R114 detached 臂误入
> → ④activeElement getter 的 R4600 focus-chain retarget 上行解析出
> **runner 投影容器形**——BND dump 实证 `_shadowHandles` 跨执行残留注册
> （`__n1/__n3/__n5/__n8`，meta hostSel:null、hostHandle 指向换代后复用的
> 活 handle）→ 误穿边界回吐投影 host DIV；⑤host tagName 防线被复用活
> handle 绕过、⑥`hostSel==null` 判据亦不成立（页面自建 shadow 的动态 host
> meta.hostSel 同为 null）——均不可判据。**落地（R5247，判据回归 spec）**：
> part04 getter + part01 `_zwIsConnected` 前置复核——`__zw_selector_for_handle`
> 宿主持久反查（不随换代清）+ querySelector 同一性核验（命中同 handle 且
> `__zw_contains` 在档才短路 true；`_zwRemovedHandles` 守门保持 remove 语义；
> shadow 树内锚宿主扁平 selector 命中别元素不短路——R91 反链 shadow 穿越
> 权威判定保持；part04 复核须前置——反链循环含中途 return 后置不可达）；
> part06 activeElement getter retarget 语义前提门——锚经同一性核验确认
> light 树直返锚（spec 焦点链 retargeting 仅当锚身处 shadow 树才穿边界；
> 判据与 stale 注册形态解耦），host 解析空壳兜底回落原锚。**验证**：
> focus-after-close 单案 **6/6 Pass**（Complex + shadow 双案同时翻绿）；
> interactive-elements 全域 98 files 482 subtest **420P/60F/2T 对 s43 逐位
> REG 0 / GAIN 1**（Fail 清单 diff 仅 Complex 案一行）；lib **3437P/0F**
>（+2 契约 part76）；契约翻转力实证（撤前置复核 → 钉 1(a) 红；light 门
> 无条件短路 → 钉 2(b) 红）。**已知边界**：focus-after-close 文件级
> Timeout（dont_scroll：focus 不滚 viewport → scroll 永不发 → promise
> 永挂）维持 s39 既有独立挂账；dialog-focus-previous-outside 3F 既有挂账。
> gates：clippy --workspace -D warnings（test-guard 包裹）/fmt clean、零
> 新增依赖。账本见
> [evidence/2026-10-23-m2s44-orphan-isconnected-retarget.md](html-semantics-compat/evidence/2026-10-23-m2s44-orphan-isconnected-retarget.md)。
> **下一步**：按挂账序 summary-less details 内非激活后代翻转（s37 邻近
> 缺口，真浏览器会翻转；2026-10-23 重新定性：spec 无 summary 翻转是 UA
> affordance 非 MUST，corpus 无观测面，优先级维持挂账序）；parser form
> element pointer 4F **已缓行**（2026-10-23 用户决策：与 declarative
> shadow DOM（H）打捆 parser 批次、M2 收口后尾段单独立项）；declarative
> shadow DOM 同打捆待触发。**切片间隙**：本切片为 shim 面，下一挂账
> summary-less 翻转亦 shim 面——间隙开放，开工前查 oracle master 碰撞
> 记录。
>
> **▶ 上轮状态（2026-10-23 M2 切片 43 收口）**：挂账「replaceWith
> self-removal 父桶记账」收口（s26 既有挂账，R371 模板残留）。**根因**：
> replaceWith sel 臂 remove 步只覆盖 apply 面（Remove/RemoveHandle
> mutation）与 sel/handle 字符串标记表（parentNode/isConnected 读），合并
> 视图子读（childNodes overlay / R322 归并查询）按**父桶 removed identity**
> 剔除——桶缺 removed 项 → 被替换者（快照 sel 节点）在合并视图残留至 apply
> 换代，视图 ≠ spec 终态。**落地（R5246 一刀）**：remove 步后经 canonical
> 记账入口 `_zwHCLiveInvalidate([], [被替换者 proxy], 父sel, null)` 直调
> （父桶 removed + 全局 pending-removed + live 集合失效一体；子树展平含
> template 内容子）；proxy identity 取基列表条目（`_r371Self`，与 overlay
> 剔除同一缓存对象）；记账先于 R371 rekey → dst 桶必存在，rekey 走合并分支
>（promote 分支保留护栏）。**验证**：契约
> test_dialog_replacewith_parent_bucket_accounting_r5246（part75 新文件，
> R1 corpus 同形全树视图退出 + R2 body 直子面）+ 既有
> r321_replacewith_fragment_flatten 断言更新（kids 前导宿主残留 DIV 消失，
> 视图即 spec 终态）；interactive-elements 全域 98 files 482 subtest
> **419P/61F/2T 对 s42 逐位 REG 0 / GAIN 0**（挂账本就「对扫描无害」，
> corpus 无观测面——切片性质同 s42：spec 正确性收口 + 契约钉住）；lib
> **3434P/0F**。**已知边界**：sel 臂 replaceWith 的 MO record 面（插入/
> 移除两段，R117 handle 臂已有）独立挂账不扩面。gates：clippy
> --workspace -D warnings（test-guard 包裹）/fmt clean、零新增依赖。账本
> 见 [evidence/2026-10-23-m2s43-replacewith-parent-bucket.md](html-semantics-compat/evidence/2026-10-23-m2s43-replacewith-parent-bucket.md)。
> **下一步**：按挂账序 summary-less details 内非激活后代翻转（s37 邻近
> 缺口，真浏览器会翻转）、focus-after-close Complex dialog usage focus
> 恢复深层（s39 剩余 1F）；parser form element pointer 4F **已缓行**
>（2026-10-23 用户决策：与 declarative shadow DOM（H）打捆 parser
> 批次、M2 收口后尾段单独立项）；declarative shadow DOM 同打捆待触发。
>
> **切片间隙宣告义务（2026-10-23 用户决策，配合
> [oracle 流解封协议](archive/incremental-rendering-oracle/master.md)）**：每个
> shim 面切片 push 后，若下一挂账不触 engine 生产面（余下挂账均为 shim
> 面），在「当前状态」宣告切片间隙开放；开工下一切片前先查 oracle
> master 碰撞记录有无占用标记，有则等待或改排其他 shim 挂账。
>
> **▶ 上轮状态（2026-10-23 M2 切片 41 收口）**：挂账「ESC 组栈边缘乱序」
> 定性收口（非代码轮）。**探针钉定**（zz-probe-s41，验证后已删）：
> corner-cases「focusin calls showModal, modeless」ESC 案事件序 =
> show() 聚焦空 modeless dialog（subject fallback）→ close-the-dialog
> 焦点恢复尾步 anchor 门命中（焦点锚=subject 后代）→ prevFocused
> focusing steps 触 focusin → 处理器 re-entrant showModal 重开。**spec
> 对账**：恢复门（锚 shadow-including 后代 OR wasModal）、show()/
> showModal() 双记录 prevFocused、focusing steps「not focusable →
> nothing」与现行文本逐字一致——R4600/R5236 无漂移。**试错回退**
> （R5244 subject 非 focusable 门，未合入）：目标案 GAIN 1 但 REG 4
> （dialog-focus-shadow「show:」三臂 + focus-previous-outside——Chrome
> 断言面「show() 无 focus delegate 聚焦 dialog 自身」，whatwg/html#8199
> 只移除 TAB 序可聚焦），两面矛盾 Chrome 选边聚焦面。**Chrome-parity
> 同红证据**：crbug 417571176（父 376516550 closedby 特性）2026-04
> 仍 "still failing according to wpt.fyi"，上游已改一次 assertion 仍未
> 过；本项目按 Chrome 拉齐，修复后随 corpus 刷新自然转绿。**收口**：
> 挂账关闭（语料钉住 + crbug 引用入账）；interactive-elements 全域
> 98 files **419P/61F/2T 对 s40 逐位 REG 0**、lib 3432P/0F、零代码净
> 变化。账本见
> [evidence/2026-10-23-m2s41-esc-stacked-parity.md](html-semantics-compat/evidence/2026-10-23-m2s41-esc-stacked-parity.md)。
> **下一步**：按挂账序非模态 close 不出组栈、R371 模板残留、
> summary-less details 内非激活后代翻转（s37）、focus-after-close
> Complex dialog usage focus 恢复深层（s39 剩余 1F）；parser form
> element pointer 4F **已缓行**（2026-10-23 用户决策：与 declarative
> shadow DOM（H）打捆 parser 批次、M2 收口后尾段单独立项）；
> declarative shadow DOM 同打捆待触发。
>
> **▶ 上轮状态（2026-10-23 M2 切片 40 收口）**：挂账 custom command
> （`--` 前缀）CommandEvent 派发收口（s24 defer）。**spec 钉定**：现行
> button activation behavior——validity Custom 态对任意元素 true →
> fire command 事件（command=值本身大小写保留、source=invoker、
> cancelable=true）→ Custom 态 return 无内建行为；派发形状
> bubbles=false/composed=true/isTrusted=true/无 relatedTarget（上游
> button-event-dispatch 语料钉死）。**落地（R5243 两刀）**：①part05
> `_defineEventSubclass('CommandEvent','Event',[command,source])`
> （ToggleEvent 同厂）；②part01 `_zwInvokerCommandActivate` custom
> 分支构造 CommandEvent + trusted 印记（R312 通道）经
> `_dispatchWithBubble` 在 target 派发后 return——分派在既有调用方
> 门内（disabled/preventDefault 零新写），`cmdCustom` 原值与 `cmd`
> 小写拆分、内建关键字面不变。**集群**：interactive-elements 全域
> 98 files 482 subtest **419P/61F/2T 与 s39 逐位打平**；dialog 集群
> 82 files 对 s35 账本 **REG 0 / GAIN 5**（全为 s39 既有收益）；
> forms/the-button-element 18 files 54 subtest 对 s24 逐位一致
> **REG 0**（corpus 零 custom command 消费）。**已知边界**：现行
> spec「invalid → Unknown 不派事件」未跟进（维持 R4400 auto 映射，
> corpus 钉住，跟进涉 dialog/details 内建关键字重构独立切片）；
> popover 关键字 + oncommand IDL + button command/commandFor IDL
> 反射维持挂账。gates：lib **3432P/0F**（+1 契约 part73
> test_invoker_custom_command_event_r5243）、clippy/fmt clean、零
> 新增依赖。账本见
> [evidence/2026-10-23-m2s40-custom-command-event.md](html-semantics-compat/evidence/2026-10-23-m2s40-custom-command-event.md)。
> **下一步**：按挂账序 ESC 组栈边缘乱序、非模态 close 不出组栈、
> R371 模板残留、summary-less details 内非激活后代翻转（s37）、
> focus-after-close Complex dialog usage focus 恢复深层（s39 剩余
> 1F）；parser form element pointer 4F **已缓行**（2026-10-23 用户
> 决策：与 declarative shadow DOM（H）打捆 parser 批次、M2 收口后
> 尾段单独立项）；declarative shadow DOM 同打捆待触发。
>
> **▶ 上轮状态（2026-10-23 M2 切片 39 收口）**：挂账 focus-in-shadow
> 跨流收口（focus-after-close「moved to shadow root」案 +
> dialog-focus-shadow-double-nested）。**探针钉定**（zz-probe 五迭代，
> 验证后已删）四层根因：①sel 模板 content 子列表 per-read 重建，跨读
> mutation 落空（spec content getter 恒返同一 fragment）；②content
> 条目 parentNode 指模板 wrapper，replaceWith 落错列表（探针 D
> `pidIsView=false`）；③`_zwMEl.replaceWith` 只有 remove 半边，替换者
> 落空；④focus delegate 非 dialog 分支缺 get-the-focusable-area 递归，
> 嵌套 shadow host 被跳过后焦点落树序后一 focusable（探针 E
> `osraeTag=BUTTON`≠autofocus 锚）。**落地（R5241 四刀 + R5242 一刀 +
> 收尾失效）**：①part04 content 子列表首读物化缓存（缓存数组即活
> 树）；②parentNode/parentElement getter 槽位消费补 parentHandle 分支
> （focus chain 上行达 shadow 边界）；③`_tcParent = _tplContent`（条
> 目父 = 视图本体）；④part03 replaceWith 补插入半边（plain 父门控，
> host proxy 面零变化）；⑤part01 focus delegate 递归（dialog 分支
> seq-focusable 优先序不动）；⑥innerHTML= setter sel 模板缓存失效
> （R5196 lib 红面收口，视图身份不动）。**集群**：
> double-nested **2F → 2P 全绿**；focus-after-close **2F+1T → 1F+1T**；
> interactive-elements 全域 98 files 419P/61F/2T 对 s38 账本
> **REG 0 / GAIN 3**；forms 域 401 files 3902P/707F/20T/1U
> **REG 0 / GAIN 3**（GAIN 为 label 文件尾 3 案 s38 跑时截断未注册，
> 共同 48 案逐位一致）。**已知边界**：focus-after-close 余 1F
> （Complex dialog usage focus 恢复，既有）；content 缓存不回灌非
> innerHTML= 的宿主侧子树变化。gates：lib **3431P/0F**、clippy/fmt
> clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s39-focus-in-shadow.md](html-semantics-compat/evidence/2026-10-23-m2s39-focus-in-shadow.md)。
> **下一步**：按挂账序 custom command `--` 前缀 CommandEvent defer、
> ESC 组栈边缘乱序、非模态 close 不出组栈、R371 模板残留、summary-less
> details 内非激活后代翻转（s37）、focus-after-close Complex dialog
> usage focus 恢复深层（s39 剩余 1F）；parser form element pointer 4F
> **已缓行**（2026-10-23 用户决策：与 declarative shadow DOM（H）打捆
> parser 批次、M2 收口后尾段单独立项，触发条件见
> [决策记录](html-semantics-compat/evidence/2026-10-23-user-decision-parser-form-pointer-defer.md)）。
>
> **▶ 上轮状态（2026-10-23 M2 切片 38 收口）**：挂账 appendChild move
> 后兄弟读 stale 收口。**探针钉定**（zz-probe 两迭代，验证后已删）：
> R334 appendChild/insertBefore wire 异步 drain，兄弟读无 pending 补偿
> （parentNode 有槽位补偿、兄弟没有）；两遍调用序列进一步钉出**共享视
> 图插位缺陷**——R182 重定位 pos 在旧位剔除前计算，剔除后未同步，
> cleanup `insertBefore(input, dialog)` 形态视图成 [dialog, input,
> script]（真树 [input, dialog, script]）；shadow 容器案露第三层——
> 上一子测试 stale parentSel 槽未覆写。**落地（R5240 三刀）**：①part04
> 兄弟 trap pending gate（sel 槽→融合视图定位 / 容器 ariaSync→
> registry 定位）；②part05 overlay 旧位剔除 pos 同步（**共享路径**，
> childNodes/children 全消费面受益）；③容器挂载路径 dual-form 槽覆写
> （R97 noWire 同款形状）。**集群**：focus-after-close **3F+1T →
> 2F+1T**（moved-in-between 案全绿）；interactive-elements 全域 98
> files 416P/64F/2T 对 s37 账本 **REG 0 / GAIN 1**；forms 域 4627
> subtest 逐位一致（REG 0）。**已知边界**：moved-to-shadow-root 案兄
> 弟断言过、红点推进到 `shadowRoot.activeElement`（focus-in-shadow 跨
> 流缺口，与 dialog-focus-previous-outside 同族，新挂账）。gates：lib
> **3430P/0F**（+1 为渲染流新增）、clippy/fmt clean、零新增依赖。账本
> 见 [evidence/2026-10-23-m2s38-pending-move-sibling.md](html-semantics-compat/evidence/2026-10-23-m2s38-pending-move-sibling.md)。
> **下一步**：按挂账序 parser form element pointer 4F（用户决策门禁，
> 单独立项征询）、custom command `--` 前缀 CommandEvent defer、ESC 组
> 栈边缘乱序、非模态 close 不出组栈、R371 模板残留、summary-less
> details 内非激活后代翻转（s37）、focus-in-shadow
> shadowRoot.activeElement + focusing steps 跨 shadow 恢复（s38 新，与
> dialog-focus 族跨流缺口同族）；declarative shadow DOM（H 定性后）待
> parser/渲染流联合切片。
>
> **▶ 上上轮状态（2026-10-23 M2 切片 37 收口）**：挂账 anchor-with-inline
> 导航缺口收口。**探针钉定**（单案隔离；多案探针 onhashchange 竞争污
> 染作废）：裸 a 后代 click 也不导航——R112 post 段 nearest activation
> 无 act:a 分支（落 input-change 兜底）；svg 案露出第二根因——summary
> 豁免祖先 walk 不认 SVG ns a[href]（`_zwIsInteractiveContentEl` ns
> 门），summary 误翻转。**落地（R5239）**：①R112 post 补 act:a/act:area
> 分支执行 `_zwAnchorActivate`（receiver 自身让位防双导航）；②summary
> 消费面祖先 walk 补 A/AREA href 判定（不扩共享交互函数）。**集群**：
> anchor-with-inline-element **Timeout 全红 → 5P/0F**；A/B 对照 summary
> 域 REG 0 / **GAIN 5**（窄面 20P/0F/0T 全绿）、details/label 逐位一
> 致；interactive-elements 全域 98 files 415P/65F/2T（Timeout 2 文件均
> 既有挂账）、dialog 子集对 s35 REG 0 / GAIN 1；forms 4627 subtest 计
> 数逐位一致。**已知边界**：点击 summary-less details 内非激活后代不
> 翻转（同根邻近缺口，真浏览器会翻转，记待办）。gates：lib **3429P/
> 0F**、clippy/fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s37-anchor-inline-activation.md](html-semantics-compat/evidence/2026-10-23-m2s37-anchor-inline-activation.md)。
> **下一步**：按挂账序 parser form element pointer 4F、custom command
> `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、非模态 close 不出组
> 栈、R371 模板残留、summary-less details 内非激活后代翻转（s37 新）；
> declarative shadow DOM（H 定性后）待 parser/渲染流联合切片。
>
> **▶ 上轮状态（2026-10-23 M2 切片 36 收口）**：挂账 impl-doc wrapper
> requestClose 收口。**探针钉定**：dialog-requestclose 面 5 files 78P/1F，
> 唯一红正是挂账项；合成 doc（`_zwMEl` 产物）dialog 四方法全缺，R125
> 原型链已接 HTMLDialogElement.prototype 但 prototype 空占位。**落地
>（R5238）**：`HTMLDialogElement.prototype.requestClose`（R4100
> details.name 同款 plain 世界供给惯例）——spec「request to close」
> 步骤照抄：open 缺 return、ownerDocument ≠ 主文档（非 fully active）
> return；trap 世界走 part04 R4200 版先于原型命中，双世界不重叠。
> **集群**：requestclose 面 **78P/1F → 79P/0F**（EXIT 1→0）、dialog
> 集群 82 files 对 s35 账本 **REG 0 / GAIN 1**（351P/59F/1T）、forms 域
> 4627 subtest 对 s34 基线计数逐位一致。**已知边界**：detached doc
> dialog 的 close/show/showModal 三方法仍缺（挂账范围外，探针 3F
> 实证）。R5237 撞渲染流，改 R5238。gates：lib **3429P/0F**、clippy/
> fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s36-wrapper-requestclose.md](html-semantics-compat/evidence/2026-10-23-m2s36-wrapper-requestclose.md)。
> **下一步**：按挂账序 anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边
> 缘乱序、非模态 close 不出组栈、R371 模板残留；detached doc dialog
> close/show/showModal 三方法（新）；declarative shadow DOM（H 定性
> 后）待 parser/渲染流联合切片。
>
> **▶ 上轮状态（2026-10-23 M2 切片 35 收口）**：挂账 E 余项 ESC 栈序
> 收口，**closedby 双语料面全清**（simple 6/6 + complex normal 面 s34
> 已绿）。**落地（R5236）**：`__zw_esc_dialog_cancel` 两段式——①
> `_zwTopLayer`（showModal+showPopover 共享插入序）反走，栈顶 popover
> 命中即 hidePopover、先碰 open dialog 切组栈；②R4300 组栈原逻辑不动
> （惰性补登记 + none 挡板 + 尾→头首个 open 组）。spec 钉定：close
> watcher 组栈，none dialog 组挡「再往下」不挡「更晚注册」——两段拼接
> 与 watcher 组栈语义在所有混合序一致。**集群**：82 files 411 subtest
> **350P/60F/1T**，同 key 对 s34 ON 基线 diff **REG 0 / GAIN 5**；回归
> 面 canceling 族 8P/0F、keyboard 77 键 A/B 逐位一致。设计迭代：首版
> 统一扫描破 part60 R2（lib test 抓住），两段式恢复。gates：zero-engine
> lib **3429P/0F**、clippy -D warnings、fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s35-esc-unified-toplayer.md](html-semantics-compat/evidence/2026-10-23-m2s35-esc-unified-toplayer.md)。
> **下一步**：按挂账序 impl-doc wrapper requestClose（s36 已收口）、
> anchor-with-inline 导航缺口、parser form element pointer 4F、custom
> command `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、非模态 close
> 不出组栈、R371 模板残留；declarative shadow DOM（H 定性后）待
> parser/渲染流联合切片。
>
> **▶ 上轮状态（2026-10-23 M2 切片 34 收口）**：挂账 F 帧驱动快照新鲜
> 化。**落地（R5235）**：Makefile `testharness-html-semantics` 入口默认
> `ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`（R2713a/R3254-KP4 既有开关，
> keyboard 套件同款入口模式；直接调 runner 维持 OFF 旧路径）。**对账三
> 面**：dialog filter 82 files 411 subtest **337P/73F/1T → 345P/65F/1T
> REG 0 / GAIN 8**（closedby-complex normal dialogB/popoverB——快照新
> 鲜后命中循环拿到真实盒）；testharness-html 合成面 7 files 14 subtest
> 逐字节一致；forms 域 401 files **4627 subtest 逐位一致**。**同日续勘
> 归因修正**：**挂账 G 移除**——spec 原文（#blocked-by-a-modal-dialog）
> 就是「栈顶模态 + flat tree 后代豁免」无祖先链豁免，gate 只判栈顶与
> spec 一致，三层栈探针是人造面（误关真因 = 快照滞后已修）；**挂账 H
> 重新定性**——最小探针实证 `<template shadowrootmode>` declarative
> shadow root 不支持（shadowRoot null、template 当活 DOM=R371 伪象），
> shadow slots 16F 归因 declarative shadow DOM 缺口（parser + shadow 树
> 渲染，后者归渲染流），修 light dismiss flat contains 是修在伪象上。
> **挂账 I（新）**：全量 testharness-html-semantics（2594 用例）>5400s
> 跑不完且 `--json` 缓冲到最后超时全丢——runner 增量输出或 make 分域入
> 口。gates：runner Makefile 一行无 Rust 变化、零新增依赖。账本见
> [evidence/2026-10-23-m2s34-raf-frame-driven-default.md](html-semantics-compat/evidence/2026-10-23-m2s34-raf-frame-driven-default.md)。
> **下一步**：E 余项 ESC 栈序（simple 5F——统一 top-layer 序覆盖
> popover 候选，涉 R4300 组栈改造）或按挂账序 impl-doc wrapper
> requestClose、anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘
> 乱序、非模态 close 不出组栈、R371 模板残留；declarative shadow DOM
> （H 定性后）待 parser/渲染流联合切片。
>
> **▶ 上轮状态（2026-10-23 M2 切片 33 收口）**：light dismiss 两遍模型
>（挂账 E 第一刀）。**落地（R5234）**：`__zw_dialog_light_dismiss_check`
> 两遍重写——①popover pass（`_zwTopLayer` 反走自顶向下，stale/摘属性/
> 断连跳过，contains 即级联停，否则 `_zwHidePopover`）；②dialog pass
>（反文档序首个 open+any 一锤定音：backdrop 命中自身 → requestClose；
> contains → break 内层挡外层；否则 requestClose——一次 pointerdown 至
> 多关一个 dialog）；③shadow 守卫（targetSel 解析不到 → 保守跳过）。
> **集群**：82 files 411 subtest **337P/73F/1T**，同 key 411 对 s32 逐位
> diff **REG 0 / GAIN 16**（closedby-complex 的 dialogA/outside-all/
> popoverA 面）。**探针钉定零盒根因 = 快照滞后**：`waitForRender`（双
> rAF）在 shim OFF 模式是同步 stub 不驱动 render，测试体内 gBCR 永远读
> 上一拍快照；`ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`（R2713a 既有开关）
> 实验实证 closedby 46 子测全出 **25P/21F** 对 OFF 同 key **回归 0
> 增益 +8**（normal dialogB/popoverB 全绿）——未默认启用（全集群对账
> 独立切片，挂账 F）。**新挂账**：F 帧驱动快照新鲜化（runner）、G
> runner modal retarget 多模态规则（下层 modal 盒内点击误 retarget
> 栈顶）、H shadow 影域 light dismiss（shadow slots 16F，target 解析
> 穿 shadow）、E 余项 ESC 栈序维持（simple 5F）。gates：zero-engine lib
> **3429P/0F**、clippy -D warnings、fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s33-light-dismiss-two-pass.md](html-semantics-compat/evidence/2026-10-23-m2s33-light-dismiss-two-pass.md)。
> **下一步**：挂账 F（帧驱动快照新鲜化 + 全集群对账，解锁 normal
> dialogB/popoverB 8 案与 simple Timeout 面已实证）或按挂账序 G/H、
> impl-doc wrapper requestClose、anchor-with-inline 导航缺口、parser
> form element pointer 4F、custom command `--` 前缀 CommandEvent defer、
> ESC 组栈边缘乱序、非模态 close 不出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-23 M2 切片 32 收口）**：popover 有效性语义。
> **红因链破案**：closedby-complex 40F 统一死于 `matches(':popover-open')`
> SyntaxError（host 引擎无该伪类 → promise_test 未处理拒绝全红）；
> closedby-simple 1F = `resetDialogOuterTest` 前置 `hidePopover()` 过度
> 抛出，文件级 Timeout = replica promise_test 不接体 rejection、其余 5
> 体同点 reject 挂死 completion callback（`pending:5 tests:6` 实测吻合）。
> **spec 钉定**：hide popover 算法仅在 validityResult 为 DOMException 时
> 抛——hidden/disconnected 态不匹配返 false 布尔 → hidePopover 静默
> no-op（popover-utils「should not throw」语料权威）。**落地三件**
>（R5233）：shim `_zwHidePopover` lenient 化（非 popover
> NotSupportedError / showing+断连 InvalidStateError / 其余静默）+
> matches 陷阱 `:popover-open` 裸伪类特判（:focus/:modal 同款 JS 侧特判）
> + part09 契约测试翻转与三态断言。**集群**：82 files 411 subtest
> **321P/89F/1T**，同 key 406 对 s31 逐位 diff **REG 0 / GAIN 0**；
> closedby-simple Timeout 解体 → 6 子测收敛 5F+1P；complex 40F 红因迁移
> 为统一行为缺口。**探针钉定两缺口**（一次 outside click → dA/dB 全关、
> pA/pB 不关）：**挂账 E（下一切片主目标）popover light dismiss 未建**
>（R3071 defer 至今）+ **dialog light dismiss 链停**（一次 click 应只关
> topmost clicked dialog 链）；simple 第二 ESC 断言面（ESC 栈序）同挂。
> focus-after-close.html Timeout 既有挂账维持。gates：zero-engine lib
> **3429P/0F**、clippy -D warnings、fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s32-popover-open-semantics.md](html-semantics-compat/evidence/2026-10-23-m2s32-popover-open-semantics.md)。
> **下一步**：挂账 E（popover light dismiss + 链停语义，engine click 路
> 径）或按挂账序挂账 A（强制同步布局冲刷）、impl-doc wrapper
> requestClose、anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘
> 乱序、非模态 close 不出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-23 M2 切片 31 收口）**：dialog showModal 布局
> 域。**谜题破案**：s30「dialog[open] computed block 但 gBCR 全零」是探
> 针伪象——gBCR 读 renderer 上一帧 rect 快照（rect_bridge），open 同 tick
> 读拿到旧布局；等帧后全部有盒。探测环刷新顺序（flush→timers→刷新）钉
> 死 stale 窗口：abspos-dialog-layout 9 案中 6 案红因 = showModal 同 turn
> 测量双侧 stale（expected 300/got 0），**挂账 A（本流）：强制同步布局
> 冲刷**（gBCR/offset* 读前冲刷 + 重排，CSSOM view 语义）。**`:modal`
> 通道落地**：模态性只在 shim JS 栈、DOM 序列化不携带 → 镜像 :focus 注
> 入通道（R4241）建全链——dom `Document.modal_dialogs`/`is_modal_dialog`
> + pipeline `inject_modal_state`（fresh-parse 后注入）+ webview 转发 +
> runner 探测环轮询 `_zwDialogModalAllSels()`（shim 全量导出，栈序、嵌
> 套模态皆匹配）+ matcher `:modal` 臂——作者 `dialog:modal` 规则自此真
> 实匹配。**UA 定位注入实验矩阵**（回归 0 纪律定性）：规范照抄（inset
> 0 + margin auto）= 9 回归（布局对 inset 指定 + size auto 走满视口拉
> 伸，light-dismiss 盒外点击全族失效）；inset auto 近似 = 3 回归（出流
> 重叠、selection/editing 点击几何漂移）；不注入 = **0 回归 0 增益**
>（320P/85F/2T 与 s30 逐位打平）→ 终态不注入。**挂账 B（渲染流）：
> abspos 收缩/fit-content 语义补齐**，UA 规则随该切片按规范回填（A 形
> 态实验已端到端验证轮询链）；s30「ancestor-is-inert 红因 = UA
> position:absolute 缺失」修正为「注入受布局收缩缺口阻断」。挂账 D：
> getComputedStyle 路径 `:modal` 注入（低优）。gates：zero-dom lib
> **863P/0F**、zero-engine lib **3428P/0F**（+1）、zero-style-system lib
> **2446P/0F**（+1）、clippy -D warnings、fmt clean、零新增依赖；渲染/
> UA 样式净变化为零免 product-smoke。账本见
> [evidence/2026-10-23-m2s31-dialog-modal-channel.md](html-semantics-compat/evidence/2026-10-23-m2s31-dialog-modal-channel.md)。
> **下一步**：挂账 A（强制同步布局冲刷，解锁 abspos/non-modal 布局语
> 料同 turn 断言面）或按挂账序 popover 有效性语义（`:popover-open` 选择
> 器 + hidePopover not-showing 抛 + popover-simple 余 hang）、impl-doc
> wrapper requestClose、anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘
> 乱序、非模态 close 不出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-23 M2 切片 30 收口）**：rendering inert 域第一
> 刀——hit-test 切面收口。宿主指针命中（runner `resolve_pointer_target`
> 候选循环）不排除模态 inert 子树，点击模态外元素照常命中。修复三件：
> ① 命中候选循环接 inert 排除（跳过 `!_zwDialogModalFocusGateOk(e)`——
> s29 门语义与 hit-test inert 排除同源直接复用，门补 globalThis 导出供
> 页面域调用，缺省回落原行为）；② backdrop 命中回落：候选全排除且栈顶
> 模态存在 → target=栈顶模态 dialog（新增 part01 `_zwDialogModalTopSel`；
> ::backdrop 无布局盒的近似）——真 Chrome 仲裁（ancestor 语料独立探针）
> 实测「点击 inert 区域 target=HTML」（backdrop 被 display:none 不渲染、
> 穿透到根），回落读 styleSheets 文本扫 `::backdrop{...display:none}`
> 命中则 target=documentElement；③ light dismiss backdrop 臂
>（`__zw_dialog_light_dismiss_check` 加 px/py）：backdrop 回落 target=
> dialog 曾压制 dismiss——target=栈顶模态 dialog 本身且落点在 gBCR 外 =
> 点击 backdrop → 照发 close request（真 Chrome md:dialog 且关闭）。
> 账本：dialog 集群 82 files **320P/85F/2T**（s29 同集 317P/87F/3T），
> 同 key 回归 **0**、增益 2（blocks-mouse-events F→P、light-dismiss-
> touch F→P 连带）+ backdrop-receives-element-events **Timeout→Pass**
>（3T 基线之一收口，backdrop 命中面直接语料）。gates：engine lib
> **3427P/0F**、clippy -D warnings、fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s30-modal-hit-test.md](html-semantics-compat/evidence/2026-10-23-m2s30-modal-hit-test.md)。
> **本轮钉定的相邻归因**：ancestor-is-inert 红因已换——dialog showModal
> 后 UA `position:absolute` 缺失（探针 pos=static、盒 (101,201,50x50)
> 落进父 div 内容区），归 abspos/layout 账（与「showModal 居中」同根），
> 本轮命中面修复已为该案铺平；inert-svg-hittest `:active` 断言引擎恒
> false（matcher 运行时伪类臂缺 active/hover），归交互态伪类账；
> selection 两案依赖指针拖选→Selection 链路，挂账；dialog-inert 依赖
> 居中布局，归 abspos 账。
> **下一步**：dialog UA showModal 布局（absolute + inset:0 + margin:auto
> 居中——abspos-dialog-layout ~10F 与 ancestor-is-inert/dialog-inert 同
> 根）或按挂账序 popover 有效性语义（`:popover-open` 选择器 +
> hidePopover not-showing 抛 + popover-simple 余 hang）、impl-doc
> wrapper requestClose、anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘
> 乱序、非模态 close 不出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-23 M2 切片 29 收口）**：模态焦点门接页面
> focus()——s28 门只覆盖宿主驱动路径（pre-focus 步、mousedown 默认动
> 作），页面直调 `el.focus()` 不在门内，inert-node-is-unfocusable 13F。
> 修复四件：① `part04 focus()` trap 接门（已聚焦 no-op 之后、失连臂之
> 前；失连元素门内跳过保 R114/T55 detached 契约）；② 门位在 R4003b
> label 前递**之后**——前递源（label 本就非 focusable area）不受模态
> inert 门，inert 语义作用在焦点落点，前递目标经递归 focus() 再入 trap
> 自行过门（inert-label-focus 断言面钉次序：门前置时该案 PASS→FAIL，
> 移位后归位）；③ showModal 栈推进移到 focusing steps 之前（嵌套模态
> 子树 focusing steps 过门）；④ R4600b inert 分支 focus fixup 落 body
> 走 `__zwModalGateBypass`（fixup 是 UA 级焦点回落非 focusing steps，
> Chromium 同款 activeElement===body）。门包含判定补 shadow-including：
> `proxy.contains` 只覆盖 light DOM，既有契约 R4600 R8（delegatesFocus
> 影子树内锚）暴露缺口——新增 part01 `_zwDialogShadowInclContains`
>（parentNode 上行遇 shadow root（`__zwHandle` 命中 `_shadowHandles`）
> 换 host 继续、frag 容器经 `_shadowHandleMeta` 跳 host，路数同 part06
> activeElement retarget，64 步 guard 防反链环）。契约
> `test_dialog_modal_focus_gate_s28` 增 G5/G6（页面 focus() 拒焦／失连
> 归解析节点所有权）。dialog 集群 82 files 407 subtest **317P/87F/3T**
>（s28 同集 303P/101F/3T），逐 subtest diff 同 key 回归 **0**、增益
> **14**（inert-node-is-unfocusable 13F 全清 15/15 +
> remove-dialog-should-unblock-document 连带 F→P）；gates：engine lib
> **3426P/0F**（含 G5/G6）。账本见
> [evidence/2026-10-23-m2s29-page-focus-gate.md](html-semantics-compat/evidence/2026-10-23-m2s29-page-focus-gate.md)。
>
> **▶ 上轮状态（2026-10-23 M2 切片 28 收口）**：corner-cases 余 2F 收口
> ——先建真 Chrome 基准仲裁（playwright-core + chromium-1243，CDP 真实
> 输入层；test_driver 桥接补 ButtonType/特殊键映射），语料基线 29/30
> 唯一失败＝esc-modeless。两案定性：esc-modeless 非 bug（真 Chrome 事件
> 流与引擎逐条一致，上游期望 closed 在 Chrome 自身即失败，记账 Chrome
> parity 非 gap）；click-modal 引擎分歧（真浏览器不可能聚焦模态外元素，
> runner pre-focus 步泄焦）→ 修复＝模态 inertness 焦点切面：`part01
> _zwDialogModalFocusGateOk`（栈顶 open+模态 dialog 子树外非 focusable
> area，stale 栈项跳过，stacked modal 仅栈顶生效）+ `part06
> _zwIsFocusableSel` 接入（宿主 pre-focus 步与 mousedown 默认动作共用
> 门）。契约 `test_dialog_modal_focus_gate_s28` G1–G4。集群 **303P/101F
> /3T**，回归 0 增益 1（corner-cases **29P/1F == 真 Chrome 29/30**）。
> 账本见
> [evidence/2026-10-23-m2s28-modal-focus-gate.md](html-semantics-compat/evidence/2026-10-23-m2s28-modal-focus-gate.md)。
>
> **▶ 上轮状态（2026-10-23 M2 切片 27 收口）**：corner-cases 毒化链根因
> 收口（R4620 批内 selector 时序）——根因在宿主共享 apply 路径而非调用方：
> JS 侧同 dispatch 的 op 引用批开始时的树态（spec insertBefore 的 ref 节点
> 对象不因前序 remove 失效），而 drain 顺序重放时前序 Remove 摘除 dialog →
> ref script 的 `:nth-child` 索引漂移（3→2）→ INS target 失配 warn+skip →
> dialog 永久 detached、同批 SetAttr 连锁失配（remove_child 已清 id_map）→
> 记录毁灭后 29 案 setup 全部 closedBy=none、ESC close request 找不到
> dialog（探针宿主 warn 实锤，R361 stash 走不到——target 先失败）。修复＝
> `apply_dom_mutations_full` 批前预解析 insert-adjacent 族四变体
> （Html/Text/Element/SelElement，同构失配语义）target 为 NodeId（不随树
> 态漂移）+ 四臂失配查表兜底（ref 被批内摘除时维持 skip，语义对齐 JS 抛错
> 面）。契约 `dom_bridge_tests::test_apply_ins_target_pre_remove_index_drift_r4620`
> （修复前红/修复后绿）。dialog 集群 82 files **302P/102F/3T**（s26 同集
> 272P/135F，+30P/−33F），逐 subtest 对 s26 账本 diff 同 key 回归 **0**、
> 同 key 增益 27（全落 corner-cases 毒化案）；corner-cases **1P/29F→28P/2F**，
> 3T 全为 s26 基线既有；focus-shadow 50P/0F、closedby 27/27、requestclose
> 75P 维持；九域全量留待下轮刷新（本轮证据＝集群逐 subtest diff）。gates：
> engine lib **3416P/0F**（含新契约）、clippy -D warnings、fmt clean、零新
> 增依赖。账本见
> [evidence/2026-10-23-m2s27-r4620.md](html-semantics-compat/evidence/2026-10-23-m2s27-r4620.md)。
> 余 2F＝`focusin calls showModal` 变体（focusin handler 内 re-entrant
> showModal 的 ESC/light-dismiss 判定边缘；clicking/modal 键 s25 同 key
> 既有）。
> **下一步**：corner-cases 余 2F（re-entrant showModal 边缘，同域小残差）
> 或按挂账序推进 inert 计算态 13F、abspos/layout ~10F（rendering 域）、
> popover 有效性语义（`:popover-open` 选择器 + hidePopover not-showing
> 抛 + popover-simple 余 hang）、impl-doc wrapper requestClose、
> anchor-with-inline 导航缺口、parser form element pointer 4F、custom
> command `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、非模态 close 不
> 出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-22/23 M2 切片 26 收口）**：dialog focusing steps 家族
> 收口（R4600 + rollover 挂死修复 R4600c/d）——spec show/showModal step 的
> run the dialog focusing steps、
> close-the-dialog 的焦点恢复落地：`_zwDialogFocusingStepsR4600`（autofocus
> 委派 → focusable area 递归判定（tag 表 + disabled/hidden/inert 子树传播/
> closed dialog 排除/delegatesFocus）→ focus 派 focusin）+
> `_zwDialogRestoreFocusR4600`（previously focused 捕获与恢复）+
> activeElement JS-observable（part06 focus chain retargeting：shadow 树内
> 沿 parentNode 上行解析焦点锚）。R4600b：focus-shadow 余 4F 根因＝R371
> replaceWith 重键的 `_r371Next` 后继快照取到**换行空白文本节点**（nodeType
> 3 无 selector），元素级 QSA R322 归并锚点只认 sel 字符串 → 恒尾插 → 焦点
> 扫描先命中后随兄弟；修复＝快照经 `_childNodeList(父sel)` 元素扫描推进到
> 最近后随**元素**兄弟。rollover 会话 R4600c/d 修 s26 树全量暴露的 3 文件
> 确定性整文件挂死（dialog-autofocus 族，load+focusin 双到齐缺 focusin
> ——runner 预注入 autofocus 先于页脚本，focusing steps 事件相位派发时
> 页监听器未注册且对已聚焦元素不重派）：part06 load 分支补 spec
> flushing-autofocus-candidates（processed flag + 首个合格候选 defer
> task，已聚焦态补发事件相位）+ part01 focusable area 补 pending-removed
> 门（元素级 QSA 无 overlay，同步摘除后仍命中）。testdriver click/down
> focus 步骤经 `_zwFocusSel` 门。
> part66 `test_dialog_focusing_steps_r4600` R1–R13 新契约。九域对账
> 5070→**5116P**（68.32%，Timeout 173 持平、subtests 7488 不变）：
> R4600c/d 隔离对照 **+3P/0 回归**（dialog-autofocus 族 3 挂死文件转
> Pass）；余 +43P 为 R4600 主体（focus-shadow 50F→0、closedby 27/27
> 维持）；同 key 回归 27 全落 corner-cases 毒化链（下述，s26 树已
> 同态）。corner-cases **1P/29F**（s25 28P/2F）：R4600 使 focusin 真实
> 触发音量 handler（remove+reinsert 共享 dialog），test#0 通过但毒化共享
> dialog → 后续 setup 全部 closedBy=none/conn=false——**新暴露**非新引入
> （s25 缺口休眠因 focusing steps 无操作），毒组合＝[reinsert#1+show+drain+
> ESC#1+focus+reinsert#2]，E1-E8 单因子全过；三层缝隙移交下轮：①R334 重
> 插 wire 完整性 ②ESC 派发锚 activeElement 反查 miss ③`_zw_esc_dialog_cancel`
> QSA-miss 顺栈 purge。另发现未修：R371 rekey 时 dst 桶不存在 → 被替换
> template 不入父桶 removed 合并视图残留（对扫描无害，spec 违例候选下轮）。
> **下一步**：corner-cases 毒化链三层缝隙（R334 重插 wire 完整性/ESC 派发
> 锚点/组栈 purge）；余挂账 inert 计算态 13F、abspos/layout ~10F（rendering
> 域）、popover 有效性语义（`:popover-open` 选择器 + hidePopover not-showing
> 抛 + popover-simple 余 hang）、impl-doc wrapper requestClose、
> anchor-with-inline 导航缺口、parser form element pointer 4F、custom
> command `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、非模态 close 不
> 出组栈、R371 模板残留。
>


> **▶ 拆分动机（2026-10-08 预留编号立项，用户「候选 goal 也立项吧」）**：继
> net-api（79.7% 收口）之后的下一个"整域收割"级主攻——html/semantics 覆盖
> forms/embedded-content/tabular-data 等真实站点核心语义面；form-validation 已
> 打过样（monthly 高亮项），全语义面从未系统圈过。
>
> **▶ 基线事实（立项时点，M1 盘点复核）**：表单校验/控件状态面有存量
> （form-validation + html-syntax M 系遗产）；各子域（forms 约束校验/控件状态/
> 提交语义、tabular-data 表格模型、embedded-content 媒体与 iframe 元素属性等）
> 通过率未知——M1 分域基线定形。
>
> **▶ 域边界（先例：R4114-N 深结构归档条款）**：本 goal 只做 **DOM/语义/API 面**
> （元素属性/IDL 接口/约束/状态机）；布局渲染差异（table 布局、multicol、控件
> 视觉）归 rendering-compat。已归档 html-compat 的余账关系 M1 盘点核对防重复
> 计账。

## Mission

以 WPT html/semantics corpus 分域为验收标准，按子域切片推进（建议序：forms
约束与控件状态 → tabular-data → document-metadata/grouping/text-level →
embedded-content → links/microdata），每域独立收口出数字。

**关键约束**：
1. **WPT 标尺先行**：M1 纯资产切片零源码改动
2. **分域收口制**：一个子域一个数字，不设总通过率门禁（corpus 太大）
3. **轻量修复优先**：约束校验/IDL 面走 shim；触 DOM parser 核心的深结构改动
   单独立项征询

覆盖范围（十个初始子域，fetch 脚本 DIRS 同清单）：forms（最大，含深层子目录
按 web-animations 先例追加）/ embedded-content / document-metadata /
grouping-content / text-level-semantics / tabular-data / interactive-elements /
scripting-1 / links / microdata。

### 排除（明确不在范围内）

- **table/multicol 等布局渲染差异** — rendering-compat 域
- **iframe 浏览上下文树** — navigation-compat M3 门控域
- **媒体元素播放语义** — media 系已归档 goal 域（embedded-content 中的媒体
  元素属性面归本 goal，播放管线不碰）
- **ARIA/无障碍映射** — 未立项域，挂账

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| engine shim | 元素 IDL 面/约束校验/状态机语义 | part 系 JS 语义 |
| dom/style-system | 属性解析与反射（不碰 parser 核心与布局） | 轻量优先 |
| WPT 资产 | html/semantics 十子域导入（分批） | fetch 脚本 + 账本回填 |

### 依赖约束（run-rules §9 碰撞管理）

- **与 rendering-compat**：布局/绘制差异记账回流（table 布局是其活跃工作面），
  本 goal 不越界修渲染
- **与 navigation-compat M3**：embedded-content 的 iframe 面只做元素属性，
  frame 树语义挂其门控
- **与 css-parser 流（本会话 fuzz 修复侧）**：CSS 解析面零交集

---

## Done Criteria

- **DC-1** 十子域导入 + 分域基线 + csv planned 行回填（每域独立数据行）
- **DC-2** 逐域收口：forms / tabular-data / document-metadata / grouping /
  text-level / interactive-elements / scripting-1 / links / microdata /
  embedded-content 各自"分级通过率不再下行两个连续轮次 + 余案归因挂账"
- **DC-3** 回归测试账本纪律（imported-tests.txt）；make test 全绿、make reftest
  零回归、product-smoke 逐字节恒值

## 里程碑

M1 导入与分域基线（✅ 2026-10-15）→ M2 forms 域（最大杠杆，多切片）→
M3 其余九域依序推进 → M4 总收口归档。

## 轮次账

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

## 依赖约束备注

form-validation 遗产为 forms 域起点；html-syntax-compat（编号 40，已收口）的
parser 面资产为消费基础。
