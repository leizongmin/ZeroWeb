# DC-2 切片 10 刀㉒收账——TT secondary-document MEl plain 世界门+API（30F 清零＋createHTMLDocument 3F 附带清）

**日期** 2026-11-25　**判定** PASS（TT 206→173＝净 +33 face，GONE 33/NEW 0；WC/DP/SAN Fail/Timeout 集逐键恒等，DOM Fail 恒 364＋Timeout 13→12 截断抖动非刀效应）

## 提交与判读绑定

| 提交 | 内容 | 判读二进制 | 电池 |
|---|---|---|---|
| 刀㉒ `bf0f0f102`（谱系：初射 `8cdc3f8f2`〔电池 v6 first-shot 锚〕→ first-shot 判读暴露 default-policy 页 21 面 NEW 翻红→同刀内回退入参 raw 化＋raw 槽撤除→amend `bf0f0f102`〔推送终态；发射锚二进制 `332a810f` 以 `bf0f0f102` 构建，发射器 EXPECT_MD5+EXPECT_HEAD 双断言在锚上通过；first-shot 电池作废〕） | part03 十处（MEl plain 世界 setAttribute(NS) 门＋NS 印记、绑定族 setAttributeNode(NS)/setNamedItem(NS)、innerHTML 门、script textContent 直门＋AccBypass 让位、insertAdjacentHTML、detached doc execCommand 搬正）＋part14 两臂（iframe srcdoc / HTMLElement setHTMLUnsafe accessor）＋part169 钉测 | `332a810f5c6ae64becf2e2efc36a563b` | v6 |

发射器 `.acceptance/dps/s10k22-battery-v6.sh`（EXPECT_MD5+EXPECT_HEAD 双断言、flock 单例、五腿 test-guard 包裹、setsid detached）；墙钟 WC 7m25s/DP 13s/SAN 17s/TT 16m42s/DOM 10m40s。

## 根因链（归因→刀→first-shot 反证→回退）

1. **世界模型**：`document.implementation.createHTMLDocument` 产物是 plain `_zwMEl` 对象（R125 setPrototypeOf 按 tag 接口原型：script→HTMLScriptElement.prototype、iframe→HTMLIFrameElement.prototype、miss→HTMLElement.prototype），整层缺 TT 门/API＋stub 意外蒙混（缺方法 TypeError 使 throws 面「过」、缺门使 works 面「过」）。
2. **刀**：MEl 世界单咽喉复用——setAttribute(NS) 门＋NS 印记补章（entry.ns/prefix/local，qualified 拆局部名）；绑定族 setAttributeNode(NS)/setNamedItem(NS) 经 setAttributeNS 委托；innerHTML/iaH 门；execCommand 搬正落 detached doc 字面量（错位对象单页跑定谳）；textContent 直门经 `trustedTypes.getPropertyType`（`__zwTtGetPropertyType` 未导出，门查 globalThis 落空定谳）＋`__zwTtAccBypass` 让位防双门（part14 原型三名 setter 委托值已 String 化失品牌）。
3. **first-shot 反证（判读反捕获设计缺陷）**：初版把 `_r122Set`（Attr.prototype value/nodeValue/textContent 共享 setter——主文档世界全局共享）管道入参 raw 化（s→v），把 Trusted 品牌带过 IDL DOMString 边界——sinkValue 品牌命中透传臂跳过 default policy（`set-attributes-require-trusted-types-default-policy.html` 21 面翻红：Attr.value/Node.nodeValue/Node.textContent × 7 组合 seenSinkName 记账断言）。语料注释定谳真实浏览器语义：node 绑定族 IDL setter 入参一律 String 化失品牌（WebIDL converter 先行 toString），Trusted 输入经 String 化后仍须走 default policy。同刀内回退＋raw 槽撤除（`_r122Raw` 三消费点回落 `attr.value`），双单页复核归位（secondary-document 97P/0F 维持、default-policy 页 264P/0F 全绿）。
4. **钉测 V3 蒸馏**：detached attr `.value=` 写不查 enforcement（oe null 不调 set-an-attribute-value）＋绑定域复检字符串拒 Trusted（品牌已在 IDL 边界失传）；corpus `acceptTrustedTypeArgumentInIDL` 仅直设 setter（setAttribute/setAttributeNS）有＝node 绑定族无 works 面的真实语义根因。

## v6 判读（锚＝k21 v5 终态 TT F 206 / WC F 1116 / DP F 222 / SAN F 50 / DOM F 364）

| 腿 | Fail | GONE/NEW | Timeout 键集 |
|---|---|---|---|
| TT | **206→173（Pass 1876→1909，+33）** | **GONE 33＝secondary-document 30＋createHTMLDocument 3 / NEW 0** | 65=65 恒 |
| WC | 1116→1116 | 0/0 | 20=20 恒 |
| DP | 222→222 | 0/0 | 1=1 恒 |
| SAN | 50→50 | 0/0 | 1=1 恒 |
| DOM | 364→364 | 0/0 | 13→12＝Range-mutations-deleteData 截断抖动（v6 整页零行＝页预算更早截断，已知噪声页非刀效应） |

GONE 33 构成：secondary-document 30（attributeSetter 9 API throws 簇＋innerHTML/srcdoc/textContent throws＋execCommand/IAH/setHTMLUnsafe works＋setAttributeNS xlink works）＋createHTMLDocument 3（Default policy applies / Trusted Type assignment is blocked / Trusted Type instances created in the main doc can be used——detached doc 管道副产品意外收益）。

## 常驻钉测

part169 两测：V1 setAttribute 双向＋值断言／V2 setAttributeNS ns 印记（qualified/local 双读）／V3 node 绑定族 detached 写不查＋绑定域复检拒字符串／V4 innerHTML／V5 srcdoc 值断言／V6 script text 三名（textContent 直门＋text/innerText 原型门→AccBypass 委托让位防双门）／V7 setHTMLUnsafe 存在＋双向／V8 IAH 双向＋插入位置实证／V9 execCommand inserthtml 门内双向＋门外直返 false；N 负面页全 sink plain 直通零抛＋ns 印记与强制无关。门禁批内 2/2 绿；s10k 全家 25/25（`cargo test -p zero-engine --lib s10k`，test-guard --compile-first 包裹）。

## 质量门禁

fmt 零 diff（css-parser 双空行 fmt 连带两度回退保持本流域内）；clippy `-p zero-engine --lib --tests -D warnings` 0 warning；提交 lei-pre-commit-guard 两度 PASS（508 行首扫 0 发现；amend 重扫 44 行 0 发现）；release 重建 `cargo build --release -p zero-wpt-runner`（test-guard 包裹 48s）。

## 残面挂账（次刀输入）

TT 残 173F：from-literal 21（part12 fromLiteral API：三 Trusted 接口静态方法＋IsTemplateStringObject＋TrustedHTML 归一化；24 个 throw 假绿面防翻红）／block-string-assignment Document-write 10／default-policy-report-only 9／createHTMLDocument 残 5／script-enforcement-008/009 各 5 等；secondary-document 全清（97/0）、set-attributes-require-*default-policy 全清（264/0）。

## 累计

DC-2 累计 83,741→**83,774**（刀㉒净 +33，五腿电池口径）；距 ≥99,947 差 16,173。
