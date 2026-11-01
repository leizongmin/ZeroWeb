# M2-2 散新案归因收敛 evidence（2026-11-01）

> export-key-error-order + sharedarraybuffer（D1 拍板前无依赖面第二切片）。
> 两案归因 → 根因修复 → 全绿；对 M2-1 锚零回归。

## 归因与修复

### export-key-error-order（2P/4F → 6/6）

- **根因**：错误检查顺序。spec exportKey 步 6（算法未注册 export op →
  NotSupportedError）先于步 7（key 非 extractable → InvalidAccessError），
  wrapKey 步 11/12 同序。HKDF/PBKDF2 密钥 importKey 强制 extractable=false
  （派生输入密钥不可导出）——「未注册 export」与「不可导出」恒同时为真，
  顺序可观察。实现 extractable 检查在前 → InvalidAccessError 错答。
- **修复**：`part01b.js` 新增 `_zw_exportRegistered(name)`（三非对称族 +
  对称 AES-*/HMAC 注册，派生/摘要域未注册）；`part02.js` exportKey 对称
  分支与 wrapKey 各在 extractable 检查前插入注册检查 → NotSupportedError。
  AES-CBC 非可导出既有 2P（注册域走 InvalidAccessError）不变。

### sharedarraybuffer（0P/8F → 8/8）

- **根因**：BufferSource 接收面未拒 SharedArrayBuffer。spec 各 BufferSource
  参数无 `[AllowShared]`——SAB 本体或其视图须 TypeError；实现
  `_zw_bufToBytes` 对 SAB（无 `.buffer`/`.length`）走空字节分支 → digest/
  encrypt 静默按零字节成功、iv 长度校验错误抛 OperationError 错型。
- **修复**：`part01b.js` 新增 `_zw_isSharedBuffer`（`Object.prototype.toString`
  内部槽标签探测——SAB 经 `WebAssembly.Memory({shared:true}).buffer` 构造，
  构造器全局可能缺席，instanceof 不可依赖；标签跨 Realm 稳定），SAB/视图
  分别命中 `[object SharedArrayBuffer]`；`_zw_bufToBytes` 顶部 throw TypeError
  ——单点守卫覆盖全部 30 处 crypto BufferSource 调用面（digest/encrypt/
  decrypt/sign/verify/importKey/derive*/wrap/unwrap 及 AES-GCM iv/
  additionalData 等 param 成员），同步 throw 即 spec WebIDL 参数转换语义
  （Chrome 同为同步 TypeError）。detached-buffer 空字节既有语义不变。

## 结果（定向跑 + 全量跑，test-guard 包裹）

| 案 | M2-1 锚 | 切片后 |
|---|---|---|
| export-key-error-order.https.any.js | 2/6 | **6/6** |
| sharedarraybuffer.https.any.js | 0/8 | **8/8** |

- **零回归**（红线 P→非P）：对 M2-1 锚
  [2026-11-01-m2s1-full.json.gz](2026-11-01-m2s1-full.json.gz) 逐
  (case, subtest) 对照——181 案两侧全等、12 变化**全部 Fail→Pass** 且精确
  落两靶案（4+8）、**P→非P 0 例**；SAB 守卫 30 处调用面对其余全绿面零扰动。
  全量新分母：**40,925 = P 26,386 + F 14,536 + T 3**。两连跑
  （2026-11-01 22:57/23:02）状态逐项全等（0 变化）。
- **散新案账面**：M1 语料预置所记 5 个散新案至此**全绿**（3 案 M1 已绿 +
  本切片 2 案）；M2 无依赖面第一项（散新案归因）收口。
- 全量 JSON 锚：[2026-11-01-m2s2-full.json.gz](2026-11-01-m2s2-full.json.gz)
  （两连跑第一轮；后续切片对照基准，替代 m2s1 锚；M 末 DC-5② 终账）。

## M2 无依赖面余账

- worker 腿接线（触 runner/workers 通道基建，与 25 号流共享
  testharness.rs——接线切片落账前对账）；
- 1T 销案（algorithm-discards-context，DOM 边界）；
- 域门控红面（supports-modern 130/241、getPublicKey 42/80 等）全部待 D1
  拍板后按域切片。
