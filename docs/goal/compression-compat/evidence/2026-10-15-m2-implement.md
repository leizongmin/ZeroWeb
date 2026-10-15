# M2 实现 — 流式状态机 + 手工帧校验（2026-10-15）

## 运行方式

- 通道：`make testharness-compression`（同 M1；JSON 账经 guard 直采
  `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 --
  ./target/release/zero-wpt-runner testharness-compression --json`）。
- 原始账本：[2026-10-15-m2-implement.json](2026-10-15-m2-implement.json)。
- 被测实现：M2 重写面——`crates/engine/src/js_dom_bridge/compress.rs`（host 流式
  状态机）+ `crates/engine/src/js_dom_shim/part02.js`（shim 构造/transform/flush）
  + `crates/engine/src/js_dom_bridge/callbacks.rs`（六回调）。

## 总账

| 项 | M1 基线 | M2 |
|---|---|---|
| 执行用例 | 18 | 18 |
| 子测试 | 309 | 318（3 案 Timeout 解锁，+9 子测试进入执行） |
| Pass | 165 | **245** |
| Fail | 141 | 73（全部 brotli 挂账簇） |
| Timeout | 3（9 子测试） | **0** |
| 通过率 | 53.40% | **77.04%** |

**非 brotli 面 245/245 = 100%**（M1 非 brotli 面六修复簇全清）。分母 318 =
309 + 9（原 Timeout 案内子测试现在执行：6 个非 brotli 全过 + 3 个 brotli 计挂账）。

## 五项修复落地（对照控制面清单）

1. **构造器 TypeError**：WebIDL enum 转换（`String(format)` toString 异常传播 +
   大小写敏感精确匹配，失败 TypeError）——`*-constructor-error` 两案 6/6 绿
  （M1 4F 清零；brotli 构造同样 TypeError，corpus 判定面一致）。
2. **严格 BufferSource chunk**：ArrayBuffer / ArrayBufferView 放行，
   SharedArrayBuffer 背书（whatwg/html#5380）与数组/原始值 → transform 抛
   TypeError（双侧 error，write/read 双 reject）——`*-bad-chunks` 非 brotli
   面 42/42 绿（M1 48F 中非 brotli 部分清零，余 16F 为 brotli）。
3. **流式 update/finish 状态机**：host handle 表（容量 64 FIFO 逐出，逐出返错
   不透明重建）+ 三格式状态机（`Decompress::new(false)` 裸 inflate + 手工帧）；
   transform 逐 chunk 回传产出——write 后未 close 即可读出（M1 3 案 9 子测试
   Timeout 清零：correct-input / extra-input / uint8array-output）。
4. **解压错误 TypeError 化 + 手工帧校验**：zlib CMF/FLG（FCHECK %31、FDICT 拒绝）
   + adler32（内联）尾验；gzip 头（ID/CM/FLG 保留位/FEXTRA/FNAME/FCOMMENT/FHCRC
   CRC16）+ 成员尾 CRC32（`flate2::Crc`）+ ISIZE + 多成员串联（RFC 1952 concat，
   成员边界重建 inflate core）；截断/成员后 trailing junk 报错——
   `decompression-corrupt-input` 26P/3F（3F = brotli，非 brotli 面 26/26 全绿，
   含 DATA 字节翻转 success/error 分类、字典流拒绝、truncation/trailing junk）。
5. **空流 done 语义**：合法空流（gzip 空成员 / zlib 空 / raw stored 空）finish
   正常 done 零输出——`decompression-empty-input` 非 brotli 3/3 绿。

## 依赖复用确认（DC-3）

- **零新增依赖**：CRC32 用 `flate2::Crc`（flate2 既有直接依赖，WOFF zlib 先例）；
  adler32 内联 ~10 行；manifest 未动。
- 单元测试：compress.rs `tests` mod 10 项（分块往返 1B/7B/大块 × 3 格式、
  write-未-close 可读、空流、截断/trailing junk、CMF/FLG/FDICT/adler、gzip
  ID/CM/FLG/CRC/ISIZE、扩展头 FNAME+FHCRC、多成员串联+垃圾、handle 面逐出错误）；
  part11.rs 两端到端测试更新（read-until-done + TypeError 断言）。engine lib
  全量 3049 绿。

## 已知限制（挂账）

- brotli 73 子测试 mission-defer（需 brotli crate，不引重依赖；构造抛 TypeError
  语义正确，功能面缺）。
- CSV byte wire ~4× 开销（V8 字符串往返）不变；流表 64 条封顶（超期逐出 →
  TypeError，防静默截断流）。
