# FileAPI M2-P1 — Blob/File 构造器 WebIDL 转换层（2026-10-11）

## 修复内容（js_dom_shim part02.js 共享路径 + part01.js 一处交界）

**311 → 435（+124P）= 87.2%**（35 案 499 子测试，0 Timeout）。JSON 全量：
[2026-10-11-m2-p1-constructor-webidl.json](2026-10-11-m2-p1-constructor-webidl.json)。
复现：`make testharness-fileapi`。

1. **sequence<BlobPart> 转换**：非对象/缺可调用 `@@iterator` → TypeError（原始值即使
   原型挂 iterator 也抛）；经真迭代器协议逐元素「迭代 + 转换」交错驱动——getter 记序
   断言（元素 0 toString 先于数组迭代器二次 length 复查）、迭代中 pop/unshift 反射、
   ToLength length getter 异常传播全走原生语义。
2. **元素联合（Blob or BufferSource or USVString）**：Blob→原样（File is-a Blob）、
   ArrayBuffer/TypedArray/DataView→原样、余者 ToString（null→'null'、Date/RegExp→抛、
   自定义 toString/valueOf 回退序）；另按 `_partSize/_partText` 既有鸭子形态兜底宿主
   Proxy 包装字节视图（`byteLength` 存在即保留——response.blob() 交界依赖）。
3. **USVString 归一**：unpaired surrogate → U+FFFD（unicode.any.js 大小/字节一致）。
4. **options 字典**：非对象 TypeError（null/undefined/正则/函数合法）；成员**字典序**
   读取（endings getter 先于 type——记序断言）；endings 枚举非法值 TypeError
   （undefined 缺省 transparent；null 也抛）；`endings:'native'` 行尾归一
   （\r\n|\r → 平台行尾，Win 探测）。
5. **type 判据**（https://w3c.github.io/FileAPI/#constructorBlob）：含 U+0020~U+007E
   之外字符 → 空串；否则 ASCII 小写化**原样保留**。取代 net-api M2 的 MIME
   parse-serialize（该语义属 mimesniff 头部解析面，`_zwParseMimeType` 仍供
   XHR/fetch Content-Type 使用）。slice contentType 改为仅 undefined 缺省继承
   （null → 'null' 串）。
6. **接口对象**：Blob.length=0 / File.length=2（WebIDL 必参个数）、无 new 调用
   TypeError、prototype 挂 `Symbol.toStringTag`（'[object Blob]'/'[object File]'——
   assert_class_string 面）。File 补 bits/name 必参 TypeError + name USVString 转换 +
   lastModified ToUnsignedLongLong 截断取模。
7. **Blob.arrayBuffer() → spec ArrayBuffer**（原 Uint8Array 折衷；FileReader readAs*
   消费端同步 Uint8Array 包裹）。
8. **part01 `_zwConsumeBodyBytes` 非流路径统一 Uint8Array**——`_zw_utf8_encode` 返普通
   Array，直进 `new Blob([bytes])` 会被 BlobPart union 转 USVString（'hi' → "104,105"，
   spec 行为浏览器一致）；body bytes 交界须为 BufferSource。

## 上游语料冲突仲裁（记档）

**mimesniff parsing.any.js 的 (Blob/File) 腿（865 子测试）与 FileAPI 语料对 Blob.type
断言互斥**：前者钉 MIME parse-serialize（`TEXT/HTML;CHARSET=GBK` →
`text/html;charset=GBK` 参数值保留大小写 + `gbk(` 加引号），后者钉范围判据+小写化。
chromium 实测（headless dump-dom，本机 chromium）：

| 输入 | chromium 实测 | FileAPI 算法 | mimesniff 腿 |
|---|---|---|---|
| `TEXT/HTML;CHARSET=GBK` | `text/html;charset=gbk` | ✓ | ✗（期望 GBK 保留） |
| ` image/gif ` | ` image/gif `（空格保留） | ✓ | — |
| `\timage/gif\t` | `''` | ✓ | — |
| `text/html;charset=gbk(` | `text/html;charset=gbk(`（无引号） | ✓ | ✗（期望加引号） |

**裁定**：chromium 与 FileAPI 语料一致 → 保留 FileAPI 语义。net-api 通道（已归档）
mimesniff 865 腿转为失败属上游语料间冲突，非回归；fetch+xhr 2060 ≈ 收口 2046（corpus
+16 漂移内持平），streams 905/1020、url 4429/6065、eventsource、xhr 全部与收口一致。

## 桥接测试翻新（钉旧行为 → spec 行为，同一提交）

r2789/r3011（arrayBuffer 断言经 Uint8Array 视图取索引）、r2791（readAsArrayBuffer
result=ArrayBuffer + byteLength）、r2792（File 补 name 必参 + 无 new 断 TypeError）、
r2978/r2982（Response/Request.blob().text() 经 `_zwConsumeBodyBytes` Uint8Array 修复
自愈）、r3021（断言经视图取字节）。

## 回归门禁

- zero-engine lib **2987P / 0F**（test-guard 包裹，RUST_MIN_STACK=32MiB——深嵌套管线
  测试栈前提，Makefile 同款）
- net-api 通道复跑：见上仲裁记档；其余 face 零回归
- make test / make reftest / fmt / clippy：见 master.md 当轮记录

## 残余失败（下轮切片）

- **P3 readAs* 读取状态机**（~31F）：result 在 loadstart/progress 阶段须为 null、
  multiple-reads 队列、abort 中断、Determining-Encoding charset 覆盖、
  readAsDataURL type 串
- **P4 零散**（~33F）：Blob.bytes() 缺失（5）、detached ArrayBuffer（4）、
  dom.window 平台对象 indexed-properties（3）、HTMLBodyElement String 化（2）、
  filelist 空表语义（4）、slice 多字节解码（6）、FrozenArray MessagePort（1）、
  url-with-fetch/xhr 边界（5）、url-format origin（1）、fileReader/progress 事件面（2）
