# s10k54 刀㊺d：S4 双键收敛（getAttributeType-namespace ＋ DOMParser-parseFromString-regression）

日期：2026-11-27 · 流域：wptfyi-dom（DC-2 切片 13 TT 席）· 基：HEAD `6374a6ff8`（刀面树字节，终验臂时 HEAD 已进 css 流 docs-only c6e4793a0/7fb193098，零码刀漂移）

## 目标与根因链（两键）

1. **TrustedTypePolicyFactory-getAttributeType-namespace**（21 子测，页级 F 0 声明）：页对
   `DOMParser.parseFromString` 产物调 `doc.getElementsByTagNameNS(...)` → `_zwParsedDoc` 无此
   方法（`is not a function`）→ forEach 首轮即抛、0 测试声明。三 mime 文档（text/html /
   application/xhtml+xml / application/xml）各需 `getElementsByTagNameNS`＋`getAttributeNode`
   ＋元素 `namespaceURI`/`localName` 消费面。
2. **DOMParser-parseFromString-regression**（1 子测）：`data:text/html;${encodeURI(...)}` **无逗号**
   畸形 data: URL iframe → `contentDocument` 现行恒 null → 页同步读 `null.getElementById` 外抛
   TypeError（断言目标＝`assert_throws_js(TypeError, …)` 内的 null 赋值 TypeError）→ 页 F。
   真浏览器路径＝畸形 URL 导航失败 → **初始 about:blank 保留**（同源可访问）→ `getElementById`
   null → null 赋值 TypeError 恰落断言窗。

## 刀面（三文件，~110 行）

- **part02 `_zwParseEl.prototype.getAttributeNode`**：Attr 视图只读快照（name/nodeName/
  localName/prefix/namespaceURI/value/nodeValue/specified；语料读 attribute.name/namespaceURI）。
- **part02 `_zwParsedDoc.prototype.getElementsByTagNameNS`**：ns+local 双键匹配、`*` 通配；HTML
  mime（R81 `_htmlDoc` 同判）元素恒 XHTML ns＋local ASCII 小写；XML mime 源串
  `xmlns`/`xmlns:prefix` 平扫绑定表（FIXME：文档级平扫近似，嵌套作用域重绑定未建）；命中元素
  印章 `namespaceURI`/`localName`（R186 根印章同型，仅命中产物挂章）。
- **part01 刀㊸ consume 点印章**：`_zwDataHtml43 = true`（text/html 载荷已同步 finish 标记）。
- **part04 contentDocument getter 分流**：sandboxed 臂在前不变；data: URL 按「初始文档」语义
  两臂入——① 印章臂（`_zwDataHtml43`）：同步窗返缓存 initialDoc（`_zwMakeIframeDoc('html','')`
  ＋`_zwURL='about:blank'`，identity 稳定）；② **畸形臂（无逗号，`url.indexOf(',') === -1`，
  RFC 2397 data 段分隔符缺失 → fetch data URL processor 失败 → 导航不发生 → 初始 about:blank
  保留）**；其余合法 data:（image/svg+xml 等）＝真实导航恒 null（document-getters 三族不变）；
  getSVGDocument iframe 臂保持无条件 null。

## 判别式考古

`_zwTimerNesting` 方案证伪：`_defer`＝queueMicrotask（part01:4767）非定时器——window load
listener 与页同步测试体 nesting 同为 0，不可分窗；真浏览器「导航任务排队」语义在 shim 里由
**载荷 mime＋URL 有效性**表达（刀㊸只同步 finish text/html 有逗号载荷），印章＋逗号双判据闭合。

## 语料风险集闭合（全量 grep 三式）

`data:text/html` × contentDocument 共现 6 页：回归页（靶）、sandbox download ×2（sandbox 臂
在前不变）、declarative-shadow-dom（srcdoc/XHR 形）、iframe-allowfullscreen（postMessage 内读
＋same-origin 臂 src=blank.htm）、find-with-moving-iframe（srcdoc 形）＝**唯一消费者即靶页**。
无逗号 data:text/html iframe src 全库唯一＝回归页。三 getters 族（iframe/frame/embed
document-getters-return-null）读 image/svg+xml **有逗号**合法 URL → null 臂不变。

## 并行轮撞面与收编（双席同刀）

本席 chain9（build 4d761fe67ffd → solo9 探针 → tt8）跑出 Face A 三 mime 全绿＋Face B 探针
合法臂全绿，但**回归页本体仍 F**（无逗号畸形形未盖印章）→ 本席判读「畸形臂」处方后，兄弟
执行体（18:19 CC 实例波）先落同源字节（part04 ② 畸形段＋逗号子句并入 `_r115IsDataHtml`），
并按「后射者叶先 -9」协议杀 chain9 两臂（tt8/dom9 双 137）、克隆同构 chain10。处置＝**让位
核验方**（同源收敛解，收编不回退）；兄弟回执与本席追认均落
`.acceptance/dps/s10k54-knife45d-coord.md`。刀号 ㊺d（㊻ 已被 s10k46/47 占用，延续
㊺b/㊺c 后缀策略）。NOTICE 信道 /tmp/s10k54/NOTICE-s10k54-chain9-inflight.md 双向落盘。

## 二进制谱系与臂账（guard 包装）

| 臂 | run-bin md5 | 脸 | 说明 |
|---|---|---|---|
| tt7 基线（s10k53） | `e25ba2e88309` | 2256P/11F/19T | 刀㊺c 终版（含 6 早轮 zz 探针页） |
| solo9（4d761fe67ffd） | `4d761fe67ffd` | 探针＋两靶页 | Face A 21 子测 Pass；Face B 合法臂绿／畸形臂（未修）页 F＝畸形臂根因直证 |
| **tt10 终判（chain10）** | `452b659501c8` | **2278P/9F/19T** | F/T 集差恰两靶键离场；页集差恰两靶页；182 共享页 P 逐页恒等＝+22 精确归因零漂移 |
| **dom10（chain10）** | `452b659501c8` | **57,505P/361F/12T** | vs s10k53 锚（57,463P/361F/12T）：F/T 集逐键恒等（comm 双空）；P 差 +42 恰 Range-mutations 双页（+13/+29）＝已知截断抖动族，其余 346 页逐页恒等＝跨套件零漂移 |

- solo10（终版字节）：探针 fbA（合法逗号形）/fbB（无逗号形）双臂均 doc/`probe=null`/
  `url=about:blank`/`assign=TypeError`；getAttributeType 页 21 子测全 Pass；回归页 Pass。
- **tt9 静默死亡（一次性）**：chain10 tt9 臂 2m46s 静默 exit 1 零输出（stdout 块缓冲未刷）；
  dmesg 窗内无本进程 OOM/segv；同字节 tt10 复跑净全程 9min 满脸＝非确定性早退归档（缘由
  未定，同字节复现失败）。
- tt8/dom9（chain9 两臂）被兄弟叶杀 137（协议现象，无损证：Face A solo9 在案，终验由
  tt10 覆盖）。
- 判读依据：`/tmp/s10k54/{solo10,ver-tt10,ver-dom10}.log`（tt9 死亡 `ver-tt9.log`）、
  s10k53 锚 `/tmp/s10k53/{ver-tt7,ver-dom}.log`、逐页 join/comm 对账脚本在案。

## 残面挂账（不在本刀域）

- TT 真实池 5F→**3F**：block-text ×2（module R1 残，runner rAF-stub 排期）＋
  SVGScript-internal-slot（solo 悬挂族 T 耦合先中性化验证）；6 zz 早轮探针页仍在语料目录
  （battery face 常数项，终判可清）。
- getElementsByTagNameNS XML 绑定表作用域栈近似（FIXME 在码）；getAttributeNode 命名空间
  属性 prefix/localName 解析未建（语料 unprefixed 面）。
- TT 席对面域 s10k50b pinned 复跑终判挂账（非本席）。

## 累计头数 census 汇总（挂账清账）

- 口径＝DC-1 base 68,790（10 目录 11-22 对齐面）＋逐刀逐键已证 delta 累加；谱系考古
  68,790 → 83,117（k11 代）→ 84,291（k50）。fold 本轮三键 → **84,299**（＋MO +1〔s10k51
  dom 41→42P〕＋刀㊺b nav +5〔ver-tt5 五键〕＋刀㊺c +2〔tt7 S3 两键〕）。
- **＋本刀 +22 → 84,321**，距 ≥99,947＝**15,626**。
- dom 面级 −321F（dc1 683F→复验 362F→现 361F）**不入头数**：面级 F 差非加法（−188 已在
  tally 基内〔slice3 代〕；余 ~133＝他流引擎码刀侧效未逐键归因〔Range-mutations 双页 ±150
  截断抖动带／T 形 TIME_LIMIT 域差〕）——DC-3 全量对账以单一方法新鲜面重推头数时统一结算。
