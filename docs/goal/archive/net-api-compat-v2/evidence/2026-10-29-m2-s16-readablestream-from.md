# M2-S16 — ReadableStream.from 收敛：11F 全清，streams 域有钉最大收敛面收口（2026-10-29）

**切片**: M2-S15 入列的后续小片（streams 域有钉真引擎缺口）。**+11 / 0 回归 /
0 MISSING**——`part02.js` `ReadableStream.from` 按 streams spec §rs.from +
WebIDL §3.2.22 async sequence 原语重做，from.any.js 39P/11F → **50P/0F
收口**；全通道 A/B 增益全部集中该文件，其余 487 文件零扰动。

## 基线与探针

- 全通道基线（A 侧）：`netapi-m2s14-r1.json`——488 文件 / 12077 分母 /
  11000 pass（S15 零改动，直接承继）。
- 单文件探针（`make testharness-net-api FILTER=streams/readable-streams/from`）：
  改前 **39P/11F**，11 条红腿逐条定位：
  1. invalid iterables ×3：`'a string'` / `@@iterator` 返回非对象 /
     `@@asyncIterator` 返回非对象（期望 from() 同步 TypeError，旧实现收留
     字符串 / 错误延迟到 pull）。
  2. promises 迭代 ×2：'an array of promises' / 'a sync iterable of
     promises'（期望 value 展平，旧实现 enqueue Promise 本体）。
  3. next() 非对象 ×2：'returns a non-object' / 'fulfills with a
     non-object'（期望流 error(TypeError)，旧实现直接 enqueue 42）。
  4. cancel/return ×4：reason 未透传 / `return: 42` 未 TypeError 拒绝 /
     return() 同步抛被吞 / return() fulfill 非对象未校验。

## 语义标定（地面真值优先）

spec 文本处于 WebIDL「async sequence」原语迁移期（2026-10 HEAD 抽取），
其中「converting V to an IDL value of type T（T=any）」字面不含 value 展平，
node 22（旧 V8）实测 `from([Promise.resolve('a')])` 不展平——与语料期望冲突。
按「上游 corpus 为验收标尺」原则以 **Chrome 实测为地面真值**：

- **Chrome 154**（`google-chrome --headless=new` DOM 探针）：
  `ReadableStream.from('ab')` → **TypeError**（async-sequence 新模型 Object
  门实测成立，node 22 不抛为旧 GetIteratorFlattened 模型残留，弃用）；
  value 展平（array of promises → 'a'）、async generator yield promise 展平、
  next() 返回 42 / fulfill 42 → 流 TypeError、cancel 的 return 面四腿语义
  全部与语料一致。
- V8 探针矩阵 11 腿全标定（.acceptance 留档 `m2s16-engine-probe` 同口径）。

## 实现落点（`crates/engine/src/js_dom_shim/part02.js`，单块替换）

- **参数门**：非 Object（null/原始值/字符串）→ 同步 TypeError
  （WebIDL async_sequence 转换步 1）。
- **GetMethod 语义**：`@@asyncIterator` nullish → 回落 `@@iterator`；取到
  非 callable 且非 nullish → TypeError（保 S15 前已绿的非 callable 两腿）。
- **open 步**：迭代器方法 call 同步抛错重抛（re-throws 两腿）+ 返回非
  Object → 同步 TypeError（函数是 Object，放行——function-iterator 两腿）。
- **pull（get-the-next-value 步）**：`iterator.next()` 同步抛 → error；
  iterResult 非 Object → error(TypeError)；done → close；**value 经
  `Promise.resolve(value)` 展平**后 enqueue、value promise 拒绝转 error。
- **cancel（close 步）**：`iterator.return` GetV 抛错 → 拒绝；undefined/
  null → resolve（'return missing' 腿）；非 callable → TypeError 拒绝；
  call 同步抛 → 拒绝；**reason 透传 `return(reason)`**；fulfill 结果非
  Object → TypeError 拒绝；promise 拒绝原样传播。

## 验证

- 探针：from.any.js **50/50**（+11 / 0F）。
- 全通道 A/B（`netapi-m2s16-r1.json` vs `netapi-m2s14-r1.json`，
  `abdiff-m2s16-r1.txt`）：**REGRESSIONS 0 / MISSING 0 / GAINS 11**（全部
  from.any.js），pass **11000 → 11011**（分母 12077 不变）。
- 语料零改动（红腿认证为真缺口后引擎单侧收敛即可，无 S12/S14 式双侧翻面）。
- 跨文件影响面：`ReadableStream.from` 调用面仅 from.any.js（streams/piping、
  tee-detached-context 中的匹配为行文非调用）——A/B 零扰动实测印证。

## 记账

1. **spec 迁移期真值口径**：WebIDL async-sequence HEAD 文本的 value 转换
   步字面与 Chrome 实测（展平）存在落差，本切片按 Chrome 实测 + WPT 语料
   实现；「node 旧 V8 ≠ Chrome 新 V8」以地面真值探针标定，不留 spec 字面
   依赖。后续 streams/fetch 腿遇到规范迁移期（Iterator helpers、async
   sequence、explicit resource management）沿用此口径。
2. M2-S14 记账 2（引擎 js_dom_bridge 预存红）与本切片无关（V8 桥
   about:blank 基址面），本轮 test-guard 包裹 `cargo test -p zero-engine`
   核对 **3594 过 / 9 败**——9 败全部落在 S14 已记档的 about:blank
   `Invalid URL` 集合（S14 的 details_name_exclusivity 期间已由他流转绿，
   无本切片新败），未动（共享面不越界）。
3. M1 归因档 90 行「USP BOM（28）/ setters-stripping（54）/ integrity（16）
   独立小片」中，setters-stripping / integrity 已分别由 S4a / S7 清偿，
   **USP BOM 28F 至今未立项**（本轮 fresh run 实测仍在：urlencoded-parser
   105 内 28F，BOM 剥除 + 孤立代理 URIError 两种形态）——入列下一小片
   （M2-S17），控制面后续清单同步补记。
