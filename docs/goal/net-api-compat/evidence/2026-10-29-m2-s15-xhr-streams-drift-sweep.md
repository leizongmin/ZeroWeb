# M2-S15 — xhr/streams 语料漂移巡检：红腿全认证为真缺口（2026-10-29）

**切片**: M2-S13 巡检口径扩展（S13 记账 4 遗留面：xhr/streams 在通道分母内、
不在 goal 三族）。**巡检 0 对齐 / 0 引擎改动**——7 文件 pin→HEAD 漂移全部分类，
通道红腿认证为真引擎缺口，streams 新增一个**有钉**可收敛切片
（ReadableStream.from 11F）。

## 巡检方法与结果

M2-S13 同法（gh-proxy raw 并行 + `cmp` 字节级），扩为**双基准**：本地 vs
master HEAD（漂移面）+ 本地 vs `WPT_REV=3159769` pin（本地改动面，S14 修正
后模型）。

- 447 文件（xhr 352 + streams 95）：**对 pin 0 差异**——语料精确处于 pin，
  M2-S14 修正的 pin 模型实测成立。
- 对 HEAD 漂移 **7 文件**，逐一定性：

| 文件 | 差异方向 | 处置 |
|---|---|---|
| xhr/json.any.js | 上游新增 4 腿（JSON response charset 忽略，blob URL 载体） | 不追（additive，记账 1） |
| xhr/formdata/set-blob.any.js | 上游加严绿腿（File 身份三通道断言） | 不追（加严翻转风险，记账 1） |
| streams/piping/error-propagation-backward.any.js | 上游新增 1 腿（ws error 先于 enqueue 不写块） | 不追（additive） |
| streams/readable-streams/from.any.js | 上游加严一条绿腿（return() 改 unreached_func） | 不追（记账 1） |
| xhr/send-redirect-post-upload.htm | 仅 link 元数据引号转义（`"`→`'`），零语义 | 无需处置（文件不在通道分母） |
| xhr/progressevent-constructor.html | 上游已删除（HEAD 404），本地 10 腿全绿 | 留 pin（记账 2） |
| streams/writable-streams/crashtests/garbage-collection.any.js | 上游有改 | 无需处置（crashtests 不在通道分母） |

## 红腿归属认证（本巡检的核心产出）

通道 xhr/streams 红腿与漂移**无交集**——全部红腿跑在 pin 内容上：

- `streams/readable-streams/from.any.js` 39P/**11F**：11 条红腿
  （invalid iterables 拒绝 ×3、promises 迭代 ×2、next() 非对象错误传播 ×2、
  cancel/return 交互 ×3、array of promises）在 HEAD 内容中原样存在
  （HEAD 唯一改动是另一条绿腿的 return() 加严）→ **`ReadableStream.from`
  真引擎缺口，非漂移**。该族为 streams 域最大的有钉收敛面。
- 其余 xhr/streams 文件（含 mimesniff/fetch 三族）红腿继承既有归因
  （M1 归因档 + M2-S1 记账面）。

## 处置

- 零语料对齐、零引擎改动——所有漂移均非「上游删除红腿 / 期望翻转」型，
  无 M2-S13/S14 式可清偿面。
- 后续小片入列：**ReadableStream.from 收敛**（streams 域，语料有钉 11F，
  优先于无钉小片）；实现按 streams spec「The ReadableStream.from」と
  create-proxied-iterator 语义。

## 记账

1. **加严/additive 不追口径沿用 M2-S13 记账 2**：json/set-blob/from/
   error-propagation 四文件的加严腿与新增腿统一留给语料仓滚动 tag（或
   WPT_REV pin 滚动）时按分类表结算；单独追平会先吃翻转/新红腿。
2. **progressevent-constructor.html（10 绿腿）上游已删**：pin 下保留（删绿
   腿只降绝对通过数）；pin 滚动时该文件自然消失——届时分母 −10、pass −10，
   属预期结算非回归。
3. 巡检范围至此覆盖通道全部五域（fetch/url/mimesniff + xhr/streams）：
   S13 记账 4 的「xhr/streams 未巡检」挂账清偿，漂移巡检按 M2-S13 结论
   不常态化，收敛到 pin 滚动单点结算。
