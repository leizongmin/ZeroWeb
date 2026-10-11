# FileAPI M2-P4 — 零散面（部分收口）（2026-10-11）

## 修复内容（js_dom_shim part02.js + part05.js 一处交界）

**467 → 488（+21P）= 97.8%**（35 案 499 子测试，0 Timeout）。JSON 全量：
[2026-10-11-m2-p4-scattered.json](2026-10-11-m2-p4-scattered.json)。
复现：`make testharness-fileapi`。

1. **Blob.bytes()**（+6P）：Promise\<Uint8Array\>（spec #dom-blob-bytes）——`_zw_blobBytes`
   每调用新建满足 [NewObject]（Blob-bytes 全簇 + Blob-newobject concurrent/[NewObject]）。
2. **Blob.slice [Clamp] long long**（+6P）：start/end 恰半取偶最近整数（0.5→0、1.5→2、
   2.5→2、3.5→4——非 truncate；`_zwClampLongLong`），Blob-slice double start/end 簇
   （(1.5)='cd' 非 'bcd' 等 6 案）。
3. **detached buffer 按 part 空**（+4P）：`_zw_partBytes` 对 detached ArrayBuffer/视图经
   `byteLength === 0` 短路（`buffer.slice`/`new Uint8Array(detached)` 会抛 TypeError）；
   `_partText` decode 捕获 detached 抛错回落 ''。**根因在 postMessage 序**：
   MessagePort.postMessage 旧序先 detach transfer 项再 structuredClone——克隆碰到
   detached buffer 即抛「Cannot perform Construct on a detached ArrayBuffer」
   （detachBuffer 首行即炸，4 案从未到过 Blob 构造）；改 spec serialize-with-transfer
   顺序（先序列化后 detach）。
4. **FileList 接口**（+4P）：'FileList' in window、FileList.length===0、
   fileList.item/length 经**原型链**（assert_inherits——R2830 裸对象 own 属性不满足）、
   item(0)→null。R2830 共享空单例经 setPrototypeOf 升级为 FileList 实例（own
   length/item/@@iterator 删除走原型；item/length getter 对无 `_zwFiles` 槽容错）。
5. **createObjectURL path 改 UUID v4**（+1P）：url-format「Path must end with a valid
   UUID」——crypto.randomUUID 优先，无则 RFC 4122 v4 手工拼装（随机字节缺省回落
   Math.random）。url-with-xhr「XHR with a fragment」随之翻绿（原 path 形态不符解析）。

## 探索后回退（记账）

平台对象 indexed-properties 的 BlobPart sequence 兜底（`length`+索引项结构判据）实测
**不可行**：上游 Blob-constructor「non-objects」簇钉 `{0:'FAIL', length:1}` 普通对象仍
TypeError——平台对象与普通对象结构同形，须品牌判定（元素代理面专轮）。已回退，288→488
维持 iterator-only 转换。

## 残余失败（11F，跨域挂账）

- **元素代理 String 化（5F）**：dom.window ×2 + File-constructor ×2（HTMLBodyElement
  in fileBits / object fileName——`String(el)` 经 R34xx `@@toPrimitive` 返 tag 名
  'body'，spec 应 '[object HTMLBodyElement]'）+ dom.window attributes（Attr 代理无
  toStringTag → '[object Object]' 15 字节 ≠ '[object Attr]' 13）。select（+1F in
  dom.window）另需 select.length/索引 options 面。**同根因集中元素代理面专轮**
  （@@toPrimitive/toString 移除或改造 + select/NamedNodeMap 索引面）——影响面
  js-dom 全通道须复跑验证。
- **url-with-fetch/xhr 4F**：Request 构造/revoke 生命周期（revoke after Request
  create/open 仍可 fetch——Request 对 blob URL 的强引用语义）+ fetch fragment。
- **FrozenArray\<MessagePort\> 1F**：postMessage transfer 后 message event 的
  `e.ports` 为空——ports 结构化转移面（webmessaging 域）。

## 回归门禁

- zero-engine lib **2987P / 0F**（test-guard，RUST_MIN_STACK=32MiB）；桥接测试零改动
- net-api 通道逐 face 恒等：url 4429/6065、fetch 1499、xhr 561、streams 905
- make test / fmt / clippy：见 master.md 当轮记录
