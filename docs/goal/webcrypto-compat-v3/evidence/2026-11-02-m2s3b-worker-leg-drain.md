# M2-3b worker 腿排干修复（2026-11-02）

## 范围与结论

销 [M2-3 挂账](2026-11-02-m2s3-worker-leg.md) hkdf/pbkdf2 worker 腿超时
（+12,533 子测试位预期），**归因修正并落地四件套修复**。修复全部落在
testharness runner 基建共享面（`tests/wpt-runner/src/testharness.rs` timer
stub / probe 环 + `crates/engine/src/js_dom_shim/part01.js` clearTimeout
probe-gated 接线），engine 生产路径零触碰。

- zzprobe13 双腿 3000/3000 全绿（默认 per-task 配置；修复前 worker 腿 90s
  超时仅 186/1503）。
- derive_bits 切片零 Timeout：pbkdf2 双腿 8632+8632 全 Pass、hkdf 7322 全
  Pass；56 非 Pass 全落 cfrg curve448/argon2（与 m2s3 锚既有 Fail 族同域，
  见全量对账）。
- `cargo test -p zero-wpt-runner` 276 passed / 0 failed。
- 全量对账 vs [m2s3 锚](2026-11-02-m2s3-full.json.gz)：见下文对账节。

## 归因修正（M2-3 挂账「投递空闲饥饿」假说被实证推翻）

M2-3 挂账归因「worker→页消息投递与宿主事件循环耦合退化（闲 worker
~0.7s/条）」。本轮 zzprobe13 仪器化探针（1500 digest promise_test，
`__zzX` live 发布 + runner 侧逐轮 dump）钉死真实机制：

1. **投递从未饥饿**：run9 实测 worker 级联 t≈2.6s 全部完成（dn=1502），
   全部 6011 条 relay postMessage 在 2.6s 内入队。上游协议每条 harness
   消息一条 relay 定时器（`postMessage` → wctx relay `setTimeout(fn,0)`，
   testharness.js WorkerTestEnvironment._dispatch → ports.postMessage），
   队列质量 ≈ 2× 子测试数（1500 子测试 → 6017 条）。
2. **页面侧 stub 队列逐轮二次失速**：runner timer stub
   `__zw_fire_due_timers` 每轮对整个队列 partition（due/rest）+ `due.sort`
   + `splice/concat` 重建，per-task 模式（M3-S1，默认 ON）每次只 fire 1 条
   → 每轮 O(N log N)、全程 O(N² log N)。run9 worker 腿 90s 预算只 fire
   **2070/6017**（`fire={n:2070, skip:2}`，1 fire/轮逐轮对账），队列头
   due −12s 仍在排队；同探针窗口腿（队列 3 条）5.7ms/轮 vs worker 腿
   43ms/轮——每轮成本随队列质量增长，二次复杂度实锤。
3. **「冻结」是 drain 速率崩塌的表象**：窗口腿同案 1503 子测试 r=1 即
   rn=1500（级联 22ms 完成、pending 卡 1 是探针自身 +20s diag 条目被 10s
   页面超时强制收案）→ 窗口腿 ~10s 正常终态。M2-3 zzprobe10 的
   「~0.7s/条」量化把「窗口即时 vs worker 排干慢」误读成投递耦合。
4. **修复后残余瓶颈 = sandbox 逐 execute 固定开销**：sorted-on-insert +
   clearTimeout 移除 + flood 门控 + probe delta 落地后 worker 腿仍 90s 仅
   854/1503（~105ms/轮）——A/B 定位：`ZW_TESTHARNESS_TIMER_PER_TASK=0`
   （batch，一次 execute 排干全部 due）**3000/3000 全绿**。数千条目堆上
   sandbox 逐脚本进出成本 ~105ms/轮 × ~6000 轮必超 90s 案预算，与页内
   每条处理无关（batch 一次进出全排干即证）。

## 修复内容（四件套）

| # | 修复 | 位置 | 效果 |
|---|------|------|------|
| 1 | 队列 sorted-on-insert（arm 期二分插入保 at 升序，fire 弹队首） | testharness.rs timer stub | 杀每轮全队列 partition+sort 二次失速；同 at 取二分上限位 = ES2019 稳定 sort 同 at push 序等价，派发序与旧实现逐条一致 |
| 2 | `__zw_clearTimeout` 真移除队列条目；part01 `clearTimeout`/`clearInterval` probe-gated 调用 | stub + part01.js | 消除已取消条目残留队列烧轮（此前 no-op 每条烧一轮 fire skip）；专用名模式与 `__zw_test_setTimeout` 同款（sandbox 每 execute 重置原生回调，页侧覆写不可靠） |
| 3 | flood 门控：fire 返回队列余量，>256 时跳过 modal/media/SW 逐轮轮询；flood cap：per-task 模式队列 >256 时单轮派发上限放宽到 256 条 due | take_probe + timer stub | 杀 flood 期每轮 4 次轮询 execute + 逐 timer 一轮的固定开销（~105ms/轮 × ~6000 轮）；轮数 ~6000 → ~24 |
| 4 | probe results delta 化（`slice(__ZW_PROBE_SEEN__)` 调用方内联，探测环累计追加） | take_probe + 探测环 | 杀 O(R²) 全量序列化（8632 子测试案每轮全量 stringify+parse 数百 KB） |

**兼容性论证**：

- 派发序不变：at 升序 + 同 at FIFO ≡ 旧 partition + ES2019 稳定 sort；due
  集仍取 fire 调用时刻快照（fire 期间新到期条目等下一轮，与旧实现一致）。
- M3-S1 per-task 语义对小队列（≤256 活跃定时器）逐字保留——M3-S1 A/B 锚
  （dom 全量 54k 子测试）的 event-loop 时序断言面不触门（活跃定时器峰值
  数十条）；flood 形态（>256）= worker relay 聚合页（每 harness 消息一
  relay timer），handler 为 harness 记账、无 DOM/modal/媒体交互，checkpoint
  放宽不影响其断言语义。
- part01 生产路径零触碰：`__zw_clearTimeout` 仅 runner stub 挂载，probe
  typeof 判定早返。
- window 腿零回归由全量对账红线承接（见下）。

## 验证阶梯

1. `cargo build --release --bin zero-wpt-runner` 干净；`node --check`
   shim 全 16 分片按 bridge 拼接序整体校验通过（改动前后拼接体均 OK）。
2. `cargo test -p zero-wpt-runner`：276 passed / 0 failed（test-guard
   `--per-proc-mem 8 --total-mem 8`；4GB 限下单进程 4.3GB 被守卫杀——
   webview 重测试单进程累计分配，8GB 限复跑干净，阈值记账）。
3. zzprobe13（1500 digest promise_test 探针，gitignored 用后即删）：
   修复前 worker 腿 Timeout 186/1503 → 四件套后仍 Timeout 854/1503 →
   batch A/B 全绿（定位残余瓶颈）→ +flood cap 后**默认配置双腿
   3000/3000 全绿**。
4. derive_bits 切片：**零 Timeout**，pbkdf2 8632+8632 双腿全 Pass、hkdf
   7322 全 Pass；56 非 Pass 全落 cfrg curve448/argon2 tentative 域。
5. 全量对账 vs m2s3 锚：见下节。
6. 共享面回归门（dom/nodes 同机 A/B）：见下节。

## 全量对账（vs m2s3 锚）

全量 `testharness-webcrypto --json`（test-guard `--per-proc-mem 4
--total-mem 8 --time-limit 3600`，exit=0；诊断探针 zzprobe13 行剥除后落
[新锚 m2s3b](2026-11-02-m2s3b-full.json.gz)）：

- **红线 P→非P = 0**；状态迁移仅 2 例 = 锚中 hkdf/pbkdf2 worker 腿案级
  Timeout 键消失（翻绿）；**F 逐位全等**（29,066 → 29,066）。
- 分母 360 案 / **81,839 子测试 = P 52,768 + F 29,066 + T 5**（锚：
  68,645 = P 40,572 + F 29,066 + T 7）——**净增 +12,196 全 Pass**，T 7→5。
- 翻绿位归因（新增 (case,subtest) 键按案拆分）：pbkdf2 worker 腿 +8535、
  hkdf worker 腿 +3661（两案案级 Timeout 消失、子测试位补齐至与同源
  window 腿等量）；窗口腿 181 案与锚**零漂移**（无任何迁移键）。
- 余 T 5 与谱系一致：algorithm-discards-context（1T 谱系挂账，iframe 子
  realm 基建切片）+ Hybrid-KEM generateKey ×4（D1 域门控）。

## 共享面回归门（dom/nodes 同机 A/B）

timer stub / probe 环为全部 testharness 通道共享面，按 M3-S1 先例以 dom/nodes
（最大 timer-heavy 面）做同机 A/B：stash 工作区改动 → 独立 target 目录构建
HEAD 基线二进制（`__ZW_PROBE_SEEN__` 标记 0 验旧）→ 双臂各跑
`testharness-dom dom/nodes --json`（test-guard `--per-proc-mem 8`）：

- 基线臂 12,806 keys / 新臂 12,805 keys，diff 3 keys **全部落
  ParentNode-replaceChildren 单案相位漂移**：基线臂呈「2 子测试 Timeout +
  无案级键」相位，新臂呈「案级 Timeout + 该 2 子测试无行」相位。
- 同案双二进制各复跑 2 次定性：**旧二进制自身即非确定**（old-1 = 2 子测试
  Timeout，old-2 = 案级 Timeout），新二进制 2/2 稳定在旧臂主相位
  （old-1 同形，29 Pass + 2 既有 T）——M1 轮已记账的「T 案相位漂移」
  flake 判据，非本轮改动回归。
- 判定：**A/B 零回归成立**。clearTimeout 移除的等价性由构造保证（被取消
  条目旧路径 fire 时因 pending 已删 no-op skip，移除后仅少烧空轮，派发序
  不变），flood 门控/cap 仅对 >256 活跃定时器页面生效（dom/nodes 队列峰值
  数十条不触门，两臂全量 diff 佐证）。
