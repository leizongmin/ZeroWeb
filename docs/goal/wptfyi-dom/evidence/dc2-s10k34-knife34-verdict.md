# DC-2 切片 10 刀㉞ verdict——connected-callbacks shadow host 簇收口（WC +10＝2 页 10 键纯翻面）

**2026-11-26 · 记录腿 HEAD＝刀㉝ 码刀 `9413c0f16` 之上；修改 3 文件（shim js ×3，+71/−1），插桩全剥（剥后终树 diff md5 `0562bb75`），clippy `zero-engine --all-targets` 0 警告（dev-guard cgroup scope 包裹，/tmp/s10k34/clippy-k34.log）**

## 结论

- **WC 语料净 +10**（k34 电池 wc 腿：GONE 10/NEW 0、T 20==20 multiset 恒等）：`custom-elements/connected-callbacks.html` 8 键（document_types() 8 种文档中 ③ the document 型 1＋④ 七种跨文档型 7，`got []`＝shadow-including 连接面 CE 不派发同构簇）＋`custom-elements/disconnected-callbacks.html` 2 键（shadow host 断开对称面——修②下钻臂连接/断开双向自动生效的真增益）。
- **目标页 solo 剥后复验**（终树 fresh release rebuild）：connected-callbacks.html **40P/0F/0T RC=0**；disconnected-callbacks.html **15P/25F＝电池面逐键复现**（r3 13P/27F→15P/25F，+2 增益持有）。
- **零回归**：sanitizer-api / domparsing 双腿 **BYTE-IDENTICAL**（cmp 0）；dom 腿 F/T 364/12 恒等（GONE 0/NEW 0）。
- **累计 84,182**（84,172＋10，距 ≥99,947 差 15,765）＝仅 F→GONE 翻面计数、P 漂移带不计（k30/k33 先例口径）。

## 根因四站（码刀 3 文件 5 修）

| 站 | 根因 | 刀 |
|---|---|---|
| ① 真 shadow root 父连接判定 | `_ceParentConnected` 对 shadow root 父两路皆 false：sr 容器自身无 `_ceConn` 标记（walk 下钻只入 sr children、sr proxy 不入 walk 栈），且 sr proxy parentNode 上行恒 null（链断于 sr proxy）——已连 host 的 shadow 树直插 CE 恒不派 connectedCallback | part03 修①：`_shadowHandleMeta` 换位臂——hostSel→`__zw_contains('html',hostSel)` 判 host 连接态；hostHandle→`_ceConn` 标记换位查 |
| ② 轻量 shadow 树下钻 | plain host（无 handle 身份的跨文档工厂/detached 树 createElement 产物）连接/断开时其轻量 shadow 树内 CE 不进退——plain host 的 shadow 树不在 childNodes 视图（pkids 递归不可达）；spec shadow-including tree 双向对称 | part03 修②：`_zwLtShadowByHost` 页级注册表（attachShadow 轻量路径 R194b 内注册）＋`_ceApplyConn` walk 循环下钻臂（push `{n,d:_wcCurDoc,ao:_ceAO}` 与主 walk 同构） |
| ③ 工厂版 attachShadow 同注册 | iframe/detached 工厂宿主 own attachShadow（part05 R179）不走 part03 R194b → 下钻源缺表项（shape-4 iframe 变体 `calls got []` 实证——walk 进臂但注册表 length=0） | part05 修③：R179 工厂版同注册 `_zwLtShadowByHost`（`typeof` 守卫防页级变量未声明） |
| ④ XML appendChild CE 门放宽 | `_zwXmlNode` doc（XHR responseXML 等）的 createElement(NS) 产物是 plain 对象（无 handle）→ 旧门 `c.__zwHandle` miss 零派发（shadow host 携带场景 host 连接时其轻量 shadow 树内 CE 入队失败） | part02 修④：门放宽 handle→元素 `nodeType===1 && typeof _ceApplyConn==='function'`；解析构建面不受影响——`_zwXmlNode` 互挂（解析装配）直接 childNodes 接线不经本方法 |

spec 锚：https://dom.spec.whatwg.org/#concept-shadow-including-connected （host 连接态含 shadow 树，双向对称）。

## 判读证据（k34 电池，插桩树 diff md5 `111bd558`）

- 四腿串行电池 09:01:39–09:16:28 零 ABORT（driver-k34.log，RC=2 全腿＝语料存在 Fail 的常态退出码）。
- san/dp：`cmp -s` BYTE-IDENTICAL。
- wc：F 955→945（GONE 10/NEW 0）、T 20==20；P 17701→17652 **算术闭合＝+10 翻转（connected +8＋disconnected +2）−59（shadow-dom/declarative/gethtml.html 已知截断带）＝−49**，逐页 awk 对账（p3w/p34w）。
- dom：F/T 364/12 恒等；P 57463→57451＝+6（Range-mutations-replaceData）−18（Range-mutations-dataChange）＝已知截断抖动族（master.md 2026-11-23 碰撞记录在案形）。
- 剥后复验：connected 40P/0F/0T RC=0；disconnected 15P/25F＝电池面复现。判读件 /tmp/s10k34/（k34-*.log/lines、r3-*.lines 来自 /tmp/s10k33、strip-verify*.log、p3*/p34* 对账件）。

## 插桩定谳法（防重走）

- `globalThis.__zwK34A/B` 计数器（walk 进臂/命中注册）优于静态考古：探针 P7 `k34A=undefined`→walk 从未进下钻臂→part05 注册缺失（修③）；P8 `k34A=1`→host appendChild 门 miss→part02 门放宽（修④）。提交前两行插桩已剥（剥后零 `__zwK34` 残留）。
- 静态追 appendChild 覆盖链（R126/R207/B1 覆盖/_makeDetachedDocument）全程无效弯路；探针源码打印 `String(doc.documentElement.appendChild).slice(0,160)` 直接定位 B1 覆盖——插桩/源码打印优于静态猜测。
- adopt 双派预判排除：AO walk `_ceConnDoc` hintDoc=doc 记账→真连接时 old==new 不双派（plain host ownerDocument 链 hintDoc）；测试时序 `calls=[]` 在 sr.appendChild 之后天然清 adopt 面。

## 残池移交

- connected-callbacks **全清**；disconnected-callbacks 残 25F 机制未分型（同页 connected 同源面已收 2 键，残面是否另有机制待细判读——刀㉟ 候选 B）。
- 刀㉚ 承继不变：Node replaceChild mutation events 1F。
- 刀㉟ 二选一候选：A) TT residual 续簇（createHTMLDocument 5F/svg-script-set-href 7F/script-enforcement-008/009 importmap/module 残面）；B) disconnected-callbacks 残 25F 细判读。

## 过程事件（防重走）

- 前台 `until grep` 轮询超 600s 被移后台——长等待用 run_in_background＋timeout 显式上限（CC 后台默认 600s 杀全链教训在案）。
- dom 腿 log 1461B 静 5min＝runner 尾缓冲输出非挂死（跑完才打印，memory 在案），判活看 CPU。
- 电池批内零树改动（插桩树 md5 断言内嵌脚本）；剥桩在电池结束后执行＝无混合二进制。
