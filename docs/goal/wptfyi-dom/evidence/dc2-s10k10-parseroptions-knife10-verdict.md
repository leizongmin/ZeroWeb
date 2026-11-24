# DC-2 切片 10 刀⑩ 判读证据——ParserOptions 刀（default policy createParserOptions 链）

- 日期：2026-11-24
- 刀面：sanitizer-api ParserOptions 家族 Fail 140（sethtml-with-trustedtypes.tentative.html 129＋parsehtmlunsafe-with-trustedtypes.tentative.html 6＋sethtml-with-trustedtypes-immutable.tentative.html 5）；22 条 Pass 中「can remove a sanitizer」「can override runScripts」「cannot change runScripts from false to true」为假绿（sanitizer 默认移除 script／写链恒不执行恰好满足断言）。
- 根因：TT enforced 时每个 HTML 解析 sink（setHTMLUnsafe/innerHTML/outerHTML/IAH/ccf/parseHTMLUnsafe）解析前应经 default policy 的 createParserOptions 回调取得 ParserOptions（https://wicg.github.io/sanitizer-api/#integrate-with-trusted-types），整链缺位。刀⑨ text 修复把 sanitizer 默认移除 script 换成执行语义后，该家族假绿翻转暴露本刀面。

## §1 实施面（7 文件 15 处）

1. `part13-sanitizer.js`：
   - 新增 `_zwSanProcessParserOptions(options)`：enforced 门→default policy 在席门→TrustedParserOptions 印直通→caller options 深拷贝（sanitizer dict JSON 深拷贝／实例 get() clone）→`def.createParserOptions(_copy)`→返回值非 object（null/undefined/数字/串）抛 TypeError→返回 branded 产物。
   - 新增 `_zwSanSanitizeString(str, po, safe)`：materialize（fragmentAddedUnsafe）→filter→serialize 净串（串级净化原语，供 setter 面/ccf 消费）。
   - `_zwSanResolveOptions` 方法形归一（sanitizer 为 function→每调新 clone）。
   - `Document.parseHTMLUnsafe` 两参包装接 TT 链；`_zwSanParseHtmlImpl` 直通目标改包装前原始实现 `_zwSanLegacyUnsafe`（修净串回环递归——钉测 p8 首轮爆栈实证）。
2. `part04.js`：setHTMLUnsafe trap（sinkValue 后接链，options 转写）／innerHTML setter／outerHTML setter／IAH trap 四 sink 接入＋净化；outerHTML 三处 `_zwFragmentAdded` 改净化后 `_ohVal`（record/registry 与写串同源）。
3. `part03.js`：ShadowRoot setHTMLUnsafe／innerHTML 两臂 TT 双臂＋净化。
4. `part06.js`：ccf 二参形 TT 臂（sinkValue＋processParserOptions＋净化）；`runScripts === false` 产物 script 递归盖 `__zwFragNoExec` 章（walker 对 nodeType 11 容器根继续下走——首版根早退钉测 p3 实证）。
5. `part12-trusted-types.js`：policy createParserOptions 包装升级——输入归一（dict→Sanitizer 实例／实例→clone／"default" 预设串→new Sanitizer）、输出 branded 平铺＋`sanitizer()` 方法形（每调新 clone，克隆 testing 语料脸）；⑩b 补空 dict→`new Sanitizer('default')`（默认基产出可消毒实例）。
6. `part14-trusted-sinks.js`（⑩b）：sinkValue create 回调异常身份 rethrow（§4.2 根因 1）。
7. `js_dom_bridge_tests/part156_s10k10_parseroptions_pins.rs`（新）：十面常驻钉测。
8. `js_dom_bridge_tests.rs`：include 接入。

## §2 钉测（test-guard 包裹，debug profile）

- 初轮爆栈：`_zwSanParseHtmlImpl` 动态取公共 `parseHTMLUnsafe` 落盘通道被刀⑩ 包装占据→enforced＋inject 面无限递归（RangeError）。
- 二轮 5 脸红：p3（章 walker 根 nodeType 11 早退）／p6（钉测只覆盖 `globalThis.__zwTtEnforced` 而 sink 管道读闭包内 CSP 解析——createHTML 未跑、序断言反翻转）／p7（part12 归一层向回调交付 Sanitizer 实例，dict 形变异条件永不命中）／p10（outerHTML handle 臂 `_zwFragmentAdded` 用 raw value）。
- 终轮：钉测种子 HTML 带 CSP meta（require-trusted-types-for 'script'，与语料页同形）走真 enforcement＋ walker 修＋outerHTML 三处同源＋回调方法形变异→**1 passed**（GUARD_RC=0）。合并态（含 part12⑩b／part14⑩b／allowAttribute 修）复跑 **1 passed** 复确认。

## §3 电池（release 绑定）

- 二进制：post md5 `15b9dcc54aab45d17270a42e86350dbb`（特征串 `__zwSanProcessParserOptions` ×17 在席）；pre 快照 `/tmp/k10-pre-runner` md5 `459cfa8f8e499abbbd2781911745c758`（特征串 ×0——刀⑩ 前基线）；v1 快照 `/tmp/k10-v1-runner`（＝15b9dcc5，TT pre-wave 对照 A 侧）。
- 发射：`.acceptance/dps/s10k10-launch.sh`（md5 断言先行，五腿顺序 WC 1500s/DP 900s/SAN 900s/TT 2400s/DOM 2400s）。
- 锚面（k9 v2 日志 agg）：WC 461 keys P=18562/F=1120/T=20；DP 77 keys P=1312/F=223/T=1；SAN 42 keys P=415/F=341/T=1；DOM 422 keys P=57452/F=496/T=12；TT 无 k9 锚（pre 快照腿对照）。

## §4 判读（LC_ALL=C awk (status,page) agg＋comm 对账）

### §4.1 v1 电池（15b9dcc5；WC/DP/SAN 三腿完，TT 腿 OOM 死、DOM 腿未起——发射链连坐被杀）

- **WC**：键集 461/461 SET-IDENTICAL；F=1120/T=20 恒；gethtml.html P 4496→3880（−616，缺失全为 h6 矩阵行、post-exclusive=0）＝既知页预算截断落位（同源 k9 自对差 590，历史带 3622–6908）非回归。
- **DP**：键集 77→76；outerhtml-02「outerHTML and string conversion: undefined.」−1F/+1P＝**真修复**（刀⑩ outerHTML record/registry 三处 `_zwFragmentAdded` 改净化后 `_ohVal` 与写串同源——raw `undefined` 与 `String()` 化两通道分叉消除）；余键恒。
- **SAN**：P=415→542（+127）、F=341→214（−127）；家族 distinct 140F→14F：
  - sethtml-with-trustedtypes 129F→12F（+117P/−117F）
  - parsehtmlunsafe 6F→1F（+6P/−6F）
  - immutable 5F→1F（+4P/−4F）
  - 非家族页零漂移。
- **TT**：EXIT=137（SIGKILL）——与同 clone 兄弟核验轮 rustc debug 编译（RSS 2.6G）撞内存 OOM，日志截断作废。
- **DOM**：发射器连坐被杀未起跑。

### §4.2 残 14F 三根因（v1 判读定谳）

1. **12 案序断言**（sethtml：Element×8＋ShadowRoot×4「exception in createHTML prevents createParserOptions from running」）：sinkValue 把 createHTML 回调异常重包成样板 `new TypeError("Failed to execute...")`——重包件跨域 instanceof Error 失败（assert_throws_js(Error) 红脸）；对照脸「exception in createParserOptions is rethrown」12 案全绿（part12 wrapper `cbs.createParserOptions(_poNorm)` 在 try 外身份传播的 Error 过检）实证根因。修＝part14 回调异常身份 rethrow（spec process-value-with-default-policy rethrow 语义）。
2. **immutable「createParserOptions doesn't mutate original object」1F**：`assert_not_equals(d.querySelector("#allowed"), null)` 得 null——`allowAttribute("id")` 在默认基（canon `removeAttributes: []`＝属性全允许）被旧实现推入 removeAttributes＝方向反转，div#allowed 丢 id。修＝allowAttribute remove 方向改语料实形（在名单→剥除返 true；缺席→false 零变化——sanitizer-modifiers「allowAttribute() with global removeAttributes」ground truth 同向）。核验轮 part157 debug 探针复刻语料回调全绿预验（callerModified=false／forbidden=gone／allowed=present）。
3. **parsehtmlunsafe「createParserOptions doesn't mutate original object」1F**：同根因 2（同 allowAttribute 面）。

### §4.3 v2 电池（8806df198e89b914808130b9cab2eba3；合并态）

- 增量：part12⑩b（空 dict sanitizer→`new Sanitizer('default')` 归一）＋part14⑩b（身份 rethrow）＋part13 allowAttribute 修——均为 v1 判读三根因的对应修复。
- 绑定纪律：提交字节＝判读二进制（allowAttribute 修晚于 v1 构建，故五腿全量 v2 重跑；v1 三腿数据作对照存档）。
- TT 增量回归网：pre-wave 腿＝/tmp/k10-v1-runner（15b9dcc5，含刀⑩ 主链、不含三修复）solo 补跑；v2 TT 对 pre-wave TT 集差（预期 NEW=GONE=0——TT 主语料零 createParserOptions 引用、createXYZTests 直调不经 sinkValue、eval 门自有包装）。
- **判读结果（v2 对 k9 锚，全域 agg＝LC_ALL=C awk (status,page)＋comm/join 逐键）**：
  - **WC**：461/461 键 SET-IDENTICAL；F=1120/T=20 恒；唯一数值差 gethtml.html P 4496→4282（−214，历史带 3622–6908 内）＝页预算截断落位非回归。
  - **DP**：键 77→76（＝outerhtml-02 Fail 键耗尽）；P 1312→1313/F 223→222（+1P/−1F）＝「outerHTML and string conversion: undefined.」单脸真修复（与 v1 同脸）；余零漂移（comm 侧 8 个 xhtml 键为首轮提取伪影，全域重建后消失）。
  - **SAN**：P 415→558（+143）/F 341→198（−143）/T=1 恒；键 42→40＝−3 Fail 键＋1 新 Pass 键；家族三页**140F 全收敛**：sethtml-with-trustedtypes 129F→0F（22P→151P，v1 残 12 序断言脸由 part14⑩b 身份 rethrow 修复）、parsehtmlunsafe-with-trustedtypes 6F→0F（1P→7P）、immutable 5F→0F（0P→5P，allowAttribute 修）——与核验方 debug 探针预验（callerModified=false／forbidden=gone／allowed=present）逐项吻合；附加修复 sanitizer-config −1F、sanitizer-modifiers −2F（allowAttribute remove 方向 ground truth 脸）；非家族零回归键（join valdiff 全部为正向修复）。
  - **DOM**：422/422 键 SET-IDENTICAL（query 后缀页首轮提取伪影修正后）；F=496/T=12 恒；唯一差异 Range-mutations-dataChange +29P／replaceData +47P＝既知 ±45P 截断抖动族（F 构成恒等）；余零漂移。
  - **TT**：v2 P=1544/F=546/T=64（227 键）；日志零 createParserOptions 引用（TT 主语料不经 sinkValue 链）；对 pre-wave 腿对账＝名字级 GONE=0、common 1742 键全恒，唯一变化 default-policy.html 3 脸 Fail→Pass（div.innerHTML/script.src/script.text throw＝part14⑩b 身份 rethrow 增益，页总测数 19 恒）——**无 TT 回归，v3 回退预案不触发**（详见 §4.5）。

### §4.4 判读工具与锚面

- 判读：`LC_ALL=C awk`（status,page) agg＋`comm -3` 逐键对账（grep 行级有静默零输出失效前科）；测试名提取按状态行剥前缀取余（防 message 正文假命中）。
- 锚面（k9 v2 agg）：WC 461 keys P=18562/F=1120/T=20；DP 77 keys P=1312/F=223/T=1；SAN 42 keys P=415/F=341/T=1；DOM 422 keys P=57452/F=496/T=12。
- gethtml.html P 历史带（同源对）：k9 3906/4496、k8 3637/3622/4390、k7 6393、k6 6537、k5 6569、k4 6908——截断抖动带。

### §4.5 TT pre-wave 对照（/tmp/k10-v1-runner 15b9dcc5 vs v2 8806df19；名字级 (status,test name) 键）

- pre-wave P=1541/F=549/T=64（227 键）→ v2 P=1544/F=546/T=64：+3P/−3F。
- GONE=0；NEW=3＝default-policy.html 三脸翻正：`div.innerHTML throw`／`script.src throw`／`script.text throw` Fail→Pass（v2 设计预期 NEW=GONE=0 在「TT 主语料零 createParserOptions 引用」意义上成立——default-policy.html 的例外传播脸经 innerHTML/src/text sink 受 part14⑩b 身份 rethrow 影响为正向增益）。
- 页级：default-policy.html 两侧总状态行数 19 恒（F 6→3、P 12→15）；default-policy-report-only.html 恒。
- 结论：身份 rethrow 无 TT 域回归，v3 回退预案不触发。

## §5 挂账

- xhtml 残 1F（XML 域）；IAH handle-only 协议突变 getter 页 TO 族；XML 域 ~200F；registries 残面序；setHTMLUnsafe runScripts:true 执行面缺失（无语料脸，安全优先不新增）。
- sanitizer-modifiers.html 残 8F（v2 后）：`allowElement` remove-basis 推入反转＝part13 :599-602 同型 bug（本刀只修 allowAttribute）；「allowElement with global elements」等锚侧红同族。
- sanitizer-config.html 残 17F：config 语义细节残留（既有挂账延续，本刀 −1F）。
