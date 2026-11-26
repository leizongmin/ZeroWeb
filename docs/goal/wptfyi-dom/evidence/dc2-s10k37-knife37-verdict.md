# DC-2 切片 10 刀㊲ verdict——快照文档 createElement 实体化（TT 池 createHTMLDocument 5F 清零）

**2026-11-26 · 已落地 `ac6341283`（推远端）；修改 2 文件（part02.js +21 净／part03.js 工厂提取 367 行 churn），clippy 纯 js 改动复用 k35 同门（零 rust 面；k37 电池 san 腿 `Compiling zero-engine` ×4＝shim 重嵌编译门直证）**

## 结论

- **TT 语料净 +5**：`trusted-types/trusted-types-createHTMLDocument.html` 5F 全清，solo **13P/0F RC=0**（四文档形态 × blocked/instances/default 三 face 全绿）。
- **零回归**：TT 全量 A/B 对跑（同树对，patch md5 `e916e9cb` 恒绑定）base F 92 → post F 87＝恰 5 目标键翻面、**NEW Fail/Timeout 0**（Pass 2167→2172）。
- **累计 84,213**（84,208＋5）＝仅 F→翻面计数、P 漂移带不计。

## 根因

非主文档（trusted-types-createHTMLDocument 语料四形态：document／createHTMLDocument／DOMParser／XHR）的 TT sink 门与 DOM API 完备性分裂：

1. **DOMParser**：`parseFromString` 的 HTML 文档 `createElement` 产物是轻量裸对象（own property 直挂：`textContent`/`innerHTML` 是普通 expando——无 TT sink 路由、无 interface 原型、无可变 API）→ blocked face 不抛、innerHTML 赋值不解析（textContent 读回 `''`）。
2. **XHR**：`responseType='document'` 的 HTML MIME 响应走 `_zwParsedDoc` 快照工厂（part03 `_zwFillFromResponse`）——该工厂**无** `createElement`（parseFromString 的 own 版本盖不到这里）→ `doc.createElement is not a function`（blocked face 因 TypeError 误 Pass，instances/default face 真 Fail）。
3. **createHTMLDocument** 形态全绿＝参照面：其产物经 `_makeDetachedDocument` 工厂 createElement（`_zwMEl` 实体元素，TT sink 门与全可变 API 齐）。

探针定谳（zz-probe-k37，判后删）：P1 createElement 存在/形态 ✓；P2 快照 body append 后实体 div 自足解析 ✓（parentNode 断言为探针自设、真语料不断言）；P3 script textContent 无 sink ✗；P4 innerHTML 无 sink ✗；P5 XHR createElement undefined ✗；P6 参照面全绿 ✓。探针法教训：TT enforcement 页探针自体先吃 sink（plain 串传 parseFromString/innerHTML 全炸）——先建同名白名单 policy（CSP `trusted-types policy default` 只许 `policy`/`default` 名，自造名直接拒）。

## 刀面（3 站）

| 站 | 修 |
|---|---|
| ① part03 `_makeDetachedDocument` 工厂 createElement | 提取为 doc 参数化助手 `function _zwDocCreateElementFor(doc, t, options)`（闭包仅 `doc`，字节级原样搬移）＋ `globalThis.__zwDocCreateElementFor` 跨 part 导出；工厂方法改薄委托 |
| ② part02 `DOMParser.parseFromString` | own createElement 分流：`d._htmlDoc` → 实体产物（`__zwDocCreateElementFor(d, …)`）；XML 文档保持轻量产物（R123 大小写敏感／R5004 XML 序列化／PI-attributes 语料面零变化） |
| ③ part02 `_zwParsedDoc.prototype.createElement` | 新增同委托（XHR document 响应路径补缺位；XML parseFromString 的 own 版本盖原型零扰） |

## 判读证据（k37 电池，byte-bound diff md5 `e916e9cb`）

- 四腿串行电池 11:37:54–11:53:02（UTC，driver-k37.log），零 ABORT（rc=2 全腿＝语料存在 Fail 常态退出码）；san 腿 build 段 `Compiling zero-engine/page-runtime/webview/zero-wpt-runner` ×4＝电池二进制吃进刀 bytes。
- **san：输出段 BYTE-IDENTICAL**（cmp 0 vs k36，build 段差异剥后恒等）。
- **dp：F/T 多重集 IDENTICAL**（F 221/T 1 恒；**P 1314==1314 零漂移**）。
- **wc：F/T 多重集 IDENTICAL**（F 920/T 20 恒）；P 16496→16919＝**+423 全归 gethtml.html**（2235→2658，巨页截断带，F/T 零变化）。
- **dom：F/T 多重集 IDENTICAL**（F 364/T 12 恒）；P 57125→57431＝**+306 全归 Range-mutations 双页**（dataChange 704→866／replaceData 723→867，已知抖动族——k36 为 −217/−207 同带反弹）。
- **TT 全量 A/B**：base F 92／T 20 → post F 87／T 20；GONE 恰 5 createHTMLDocument 键、NEW 0。

## 残池移交

- TT 池余 **87 语料 F**（A/B 权威＝92−5；master.md 11-23 账 84 已过时， intervening 刀窗漂移未逐笔归因，以本 A/B 为准）＋Timeout 20 不变。Top：svg-script-set-href 7／script-enforcement-004 4／policy-without-return-value 4／script-enforcement-006/007/008/009 各 3／block-text-node-insertion-into-script-element 3／block-string-assignment-to-DOMParser-parseFromString 3／其余散布。
- 刀㉚ 承继不变：Node replaceChild mutation events 1F。
- sanitizer-api 语料（另一池，master 挂账 50F）本刀零波及（san 腿字节恒等）。
