# M2-S7 — SRI integrity 校验门（fetch verify 步）（2026-10-29）

**切片**: goal v2.9 小片清单第 1 位——「SRI 切片（integrity 显影两腿同根因
同批收敛）」。清偿 M2-S3c 记账 1（「CORS invalid integrity」显影）与
M2-S6 记账 4（response-null-body SRI 腿显影）——两腿同根因（引擎无 SRI
实现，integrity 元数据被忽略）。

## 失败面（收敛前 6F）

| 腿 | 根因 |
|---|---|
| integrity.sub: Invalid integrity / Multiple invalid stronger than valid / both are invalid / CORS invalid integrity | mismatch 不拒绝（integrity 忽略 → fulfill） |
| integrity.sub: SHA-* integrity for opaque response | opaque + integrity 非空不拒绝 |
| response-null-body: Null body status with subresource integrity should abort | null body + integrity 非空不拒绝 |

## 实现面（`part01.js`，全 shim 侧单点）

**`_zwFetchVerifyIntegrity(response, integrity)`**（settleFetch 最终 settle
处的校验门；中间跳不校验，spec 校验在最终 response）：

1. **opaque filtered response + integrity 非空 → fail**（opaque 过滤已清空
   body，校验无输入，spec 恒不匹配；空 integrity 不入门照常 pass）。
2. **null body（204/205/304）+ integrity 非空 → fail**（fetch spec：response
   body null → network error，不做哈希比对）。
3. **metadata 解析**（W3C SRI grammar）：空白分隔 `alg-value`，alg ∈
   sha256/384/512 大小写不敏感；非法项跳过不拖垮整体；按算法强度取最强组，
   组内任一 value 匹配即通过（Multiple integrities 三腿语料面）；metadata
   非空但无合法 option → fail。
4. **base64 值宽容面**：base64url 字母表（-/_）与缺省 padding 合法
   （wrongpadding / base64url 两腿）；长度余 3 或字符非法跳过。
5. **摘要直调 host 回调** `__zw_crypto_subtle_digest`（RustCrypto sha1/sha2
   底座既有，与 part01b ECDSA 面同型）——不走 `crypto.subtle`：其按
   [SecureContext] 暴露，runner http 页锚（wpt.test:8000）下 undefined。
   digest 失败 fail-closed（校验失败不得放行响应）。
6. integrity 取值：init 优先、Request 载体回落（`_zwIntegrity` 既有存储，
   同 mode/method 取值型）；XHR/scheme dispatch（data:/blob:）路径不入门
   （语料无钉，最小实现）。

管道语义核对：runner top.txt 传输字节 `b'top'`，语料 sha256/384/512 三组
哈希逐一吻合（`sha256("top") = KHIDZcXn…`）——「SRI 摘要以传输字节计」。

## 验证

- probe（`testharness-net-api integrity`）：integrity.sub.any.js
  13P/5F → **18P/0F 收口**；response-null-body 9P/2F → **10P/1F**
  （SRI 腿翻绿，HEAD 腿既存 F 非本片）；abort/general 35P/26F 持平
  （settle 时序敏感族零扰动）。
- 全通道 A/B（`netapi-m2s7-r1.json` vs **直接前轮** `netapi-m2s6-r1.json`，
  逐 subtest）：**+6 增益 / 0 回归 / MISSING 0**（10887 → 10893 pass）；
  stderr **0 panic**，构建 0 warning（`build-m2s7-r1.log`）。
  - 本片 +6 = integrity.sub +5 + response-null-body SRI +1；其余全通道
    逐文件零变化。

## 记账（未清偿）

1. **Request 载体 integrity 传递面**：fetch(request) 继承 request.integrity
   已入门（回落取值），但 Request 构造器 integrity 校验（非 string 原样
   存储等 init-003 属性面）未动——语料当前无失败钉。
2. **scheme dispatch 路径（data:/blob:）SRI**：spec 上同样校验，语料无钉
   未入门；随完整性语料扩批再评估。
