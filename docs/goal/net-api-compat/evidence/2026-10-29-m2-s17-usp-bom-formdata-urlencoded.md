# M2-S17 — USP BOM + formData urlencoded 字节面解析：28F 全清（2026-10-29）

**切片**: M1 归因档「独立小片」清单最后一项清偿（90 行：USP BOM（28）/
setters-stripping（54）/ integrity（16）——后两者已分别由 S4a / S7 收口，
本项经 M2-S16 记账 3 重新发现）。**+28 / 0 回归 / 0 MISSING**——
url/urlencoded-parser.any.js 77P/28F → **105P/0F 收口**。

## 红腿分型（fresh run 实测两形态）

- **BOM 剥除 ×4**：`URLSearchParams constructed with: ﻿test=﻿`（直写 +
  `%EF%BB%BF` 形态）与 request/response.formData 同名 2 腿——期望键值保留
  前导 U+FEFF，实测剥除。
- **URIError ×24**：全部 request/response.formData 腿——非法 % 序列
  （`%=a`/`%a=a`/`%a_=`）、非法 UTF-8（`%FE%FF`/`%FF%FE`）、半截多字节
  （`%C2`/`%C2x`）应 lossy 替换 U+FFFD **不抛**，实测 `URIError: URI
  malformed`（decodeURIComponent 形态）。

## 根因

- URLSearchParams 路径（`part02.js` `_zw_usp_decode`）：percent-decode 后
  用缺省 `new TextDecoder()` 解码——encoding spec 缺省即**剥**前导 BOM，
  而 url spec urlencoded parser 要求「UTF-8 decoding **without BOM**」
  （保留 U+FEFF）。
- formData 路径（`part01.js` `_zwParseFormUrlencoded`）：逐 pair 用
  `decodeURIComponent`——对非法序列抛 URIError（urlencoded parser 应字面
  保留非法 %、lossy UTF-8）；且体字节先经缺省 TextDecoder（剥 BOM）文本化。
  即 formData 未按 fetch spec「parse the urlencoded-byte sequence」在字节
  面语义解析。

## 修复（spec 形状对齐）

- `_zw_usp_decode`：解码改 `new TextDecoder('utf-8', { ignoreBOM: true })`
  （ignoreBOM=true = 不剥 BOM，shim host decoder 已支持）。
- `_zwParseFormUrlencoded` 改收**体字节**：整体经 `{ ignoreBOM: true }`
  lossy 文本化（保留 BOM），逐 pair `'+'→' '` + `_zwPercentDecodeBytes`
  （非法 % 字面保留）+ 同语义解码；`&`/`=` 分割与 '+' 替换在文本面安全
  （0x26/0x3D/0x2B 不入多字节序列）。U+FFFD 经「替换→再编码→再解码」往返
  1:1 保持（`%FE%FF` → 双 U+FFFD 腿验证）。旧 `decodeURIComponent` +
  缺省 TextDecoder 组合废弃；multipart 路径仍走剥 BOM 文本（不属本面）。
- Response.formData / Request.formData 两入口改传字节 `b`（原传文本），
  multipart 判定顺序不变。

## 验证

- 探针：urlencoded-parser **105/105**（+28 / 0F）。
- 全通道 A/B（`netapi-m2s17-r1.json` vs `netapi-m2s16-r1.json`，
  `abdiff-m2s17-r1.txt`）：**REGRESSIONS 0 / MISSING 0 / GAINS 28**（全部
  urlencoded-parser——formData 消费面 request-consume / response-consume /
  body/formdata 等 10 文件零扰动），pass **11011 → 11039**（分母 12077）。
- 范围引擎测试（test-guard 包裹 `cargo test -p zero-engine`）：**3593 过 /
  10 败**——预存 about:blank `Invalid URL` 9 败不变；第 10 败
  `test_details_name_exclusivity_template_deep_entry`（template 物化互斥
  断言）经 stash 干净树核对在无本切片改动的 cb78f5b53 上原样复现——由
  并行流 d6aadb07b（details/template 双面补树序互斥，DOM 域）引入的新
  测试红，归引擎 DOM 流（run-rules §10 归因纪律，共享面不越界不代修），
  本切片零关联。

## 记账

1. M1 归因档 90 行「独立小片」清单**全部清偿**（BOM 本档 / setters S4a /
   integrity S7）；后续小片清单不再有「有钉引擎缺口」存量（request-cache
   98F 待用户拍板、无钉两片随语料 pin 滚动重估）。
2. 老 parser 的 `body.trim()` 前后空白裁剪随之消失（urlencoded parser 无
   trim 步，spec 对齐）；A/B 零扰动实测无依赖面。
3. 并行流 d6aadb07b 新引入的 `test_details_name_exclusivity_template_deep_entry`
   红（见验证节）不在本 goal 域，未处理；如该流未捕获，控制面渲染域（或
   engine 域）账本为其归属记录的候选信号。
