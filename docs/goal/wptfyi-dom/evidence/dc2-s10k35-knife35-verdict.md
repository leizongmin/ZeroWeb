# DC-2 切片 10 刀㉟ verdict——disconnected-callbacks removal 面收口（六站 CE 断连派发补齐＝WC +25）

**2026-11-26 · 已落地 `fde571d6c`（推远端）；修改 3 文件（part02.js +24／part03.js +29／part05.js +17），clippy `zero-engine --all-targets` 0 警告（dev-guard scope，/tmp/s10k35/clippy-k35.log）**

## 结论

- **WC 语料净 +25**：`custom-elements/disconnected-callbacks.html` 全页 25F 清零，solo **40P/0F RC=0**；同页 connected-callbacks 刀㉞ 增益持有。
- **累计 84,207**（84,182＋25）＝仅 F→GONE 翻面计数（k30/k33/k34 先例口径）。

## 刀面（六站补 disconnectedCallback 派发，`_ceApplyConn` was 记账门防未连误派）

| 文件 | 站 |
|---|---|
| part02.js | `_zwXmlNode.removeChild` 新增（XHR responseXML XML 文档旧无此法＝整测 TypeError；splice 后 nodeType===1 派 dc） |
| part03.js | R194b 轻量 shadow removal／plain `_zwMEl` 树 `_zwMElRemoveChild` removal／detached doc docEl removal |
| part05.js | R194 工厂版 shadow removal／R249 工厂元素 removal |

## 判读证据（k35 电池，byte-bound diff md5 `dfc1ed7d`）

- 四腿串行电池 09:50:15–10:04:57（UTC，driver-k35.log），零 ABORT（rc=2 全腿＝语料存在 Fail 常态退出码）。
- **wc：F 945→921**（GONE 25 全页 disconnected-callbacks；raw comm 另见 NEW 1＝fragment-parsing 主文档键 P→F，**对 HEAD 基线 33809a5c6 复现＝刀㉞ 剥树潜 latent 面非本刀引入**，真对基线 GONE 25/NEW 0——订正详情见 [刀㊱ verdict 定性订正节](dc2-s10k36-knife36-verdict.md)）；T 20==20；P +42＝+25 翻面 −1 潜 latent ＋18 gethtml 截断带，逐页归因闭合。
- **san/dp：BYTE-IDENTICAL**（cmp 0 vs k34）。
- **dom：F/T 多重集 IDENTICAL**（F 364/T 12 恒）；P +98＝Range-mutations 双页已知抖动族。

## 残池移交

- disconnected-callbacks 池闭合（25F 全清）；fragment-parsing 潜 latent 面由刀㊱ 收口（[k36 verdict](dc2-s10k36-knife36-verdict.md)）。
- 刀㉚ 承继不变：Node replaceChild mutation events 1F。
