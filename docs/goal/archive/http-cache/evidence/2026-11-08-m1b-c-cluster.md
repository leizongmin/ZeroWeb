# M1-b——runner cache.py fixture + 案级缓存包裹，C 簇 48 收敛

**日期**: 2026-11-08（M1 完成，git main @ 666d1f293；M1-a @ f1fb5956b）
**构成**: M1-a（net CacheMode 扩六态 + 矩阵单测）+ M1-b（本档：runner
cache.py 端点仿真 + 案级 HttpCache 包裹 + net 注入修正）。

## 一、交付内容（commit 666d1f293，3 文件 +447/-14）

### net 层（DC-2 前半语义修正）

1. **storable_mode 修正**（`crates/net/src/cache_policy.rs`）：生成即过期
   （ttl=0）但带验证器的响应改为可存 `RevalidateOnly` 条目（RFC 9111 §3
   锚定）——WPT stale 臂依赖「先存 max-age=0 响应、再发条件请求拿 304」，
   原实现返回 None 导致 stale 臂不可存。无验证器仍不存。
2. **注入映射修正**（`crates/net/src/resource_loader.rs`）：M1-a 提交的
   映射（NoCache→CC no-cache；Reload→CC max-age=0）**与语料和现行 spec
   相反**，本次按本地语料逐文件核实 + fetch spec §4.6 现行文本改正：
   - no-cache → `Cache-Control: max-age=0`（step 17；语料 no-cache 文件
     `expected_max_age_headers: [true]`）；
   - no-store / reload → `Pragma: no-cache` + `Cache-Control: no-cache`
     （step 18；语料 reload/no_store 文件 `expected_no_cache_headers: [true]`
     双头在场，且无任何 expected_max_age 断言）。
3. **default+显式条件头旁路**（step 16）：五头（If-Modified-Since/
   If-None-Match/If-Unmodified-Since/If-Match/If-Range）在场 → 置 NoStore
   旁路读写（提交网络时走 step 18 注入），用户条件头原样透传。
4. 新增单测 3 项（stale-at-birth 可存性/条件头判定/注入缺席不重复）；
   `cargo test -p zero-net --lib` 497P/0F。

### runner 层（`tests/wpt-runner/src/testharness.rs`）

1. **cache.py 端点 1:1 仿真**（`wpt_cache_py_fixture`）：上游
   `fetch/api/request/resources/cache.py`（2026-11-08 master 拉取）逐句移植
   ——token 观察记录（InM/IMS/Pragma/CC 仅在场者记、`ignore` 整条留空、
   记录先于响应分支）、`querystate` take 取走 JSON（缺记录回 `null`）、
   tag 引号化（ETag 头与 InM 比对共用）、ETag/Last-Modified 互斥（tag 优先）、
   Expires/Vary/CC 回显、ACO `*`、redirect→302 "Redirect" 先于条件判定、
   条件命中（字符串全等）→304 "Not Modified" 空体。
2. **案级缓存包裹**（`wpt_cache_py_wrapped_handler`）：DC-3 单实现——直接
   复用 zero-net `HttpCache`（与浏览器通道同一缓存语义层），default 路径
   镜像 `ResourceLoader::submit`：新鲜命中直回放（base 不达、stash 计数不增
   ——request-cache 断言核心）、可再验证合并条件头（用户已带不覆盖）→304
   走 `not_modified` 元数据合并回放缓存响应、非 304 走 `put_with_headers`
   （storable_mode 内部门控）、Miss 先 base 后存储。零回归门控：仅精确路径
   `fetch/api/request/resources/cache.py` 无体 GET 过缓存层，其余直通 base
   （XHR 通道 `access-control-basic-preflight-cache.py` 等按末段匹配会误捕，
   故不用 ends_with 门）。
3. **FIXME(http-cache M2)**：wire 第 11 参落地后按六模式态分派。

## 二、实测账（2026-11-08，`make testharness-net-api FILTER=request-cache`）

```
Pass 76 / Fail 30（v2 基线 102F → 30F，-72）
  request-cache-default             8P / 0F   ← C 簇收敛
  request-cache-default-conditional 40P / 0F  ← C 簇收敛（DC-2 前半达成）
  request-cache-force-cache         14P / 2F
  request-cache-no-cache             2P / 2F
  request-cache-only-if-cached       4P / 10F
  request-cache-reload               4P / 8F
  request-cache-no_store             0P / 8F
（filter 同窗 eventsource/request-cache-control.any.js 4P 未回归）
```

- **C 簇 48/48 绿**——freshness 命中/条件再验证/304 合并/Vary 语义经真实
  zero-net HttpCache 在 runner 通道显影（DC-2 前半）。
- **B 簇余 30F 全为 wire 传播面**（request.cache 六态 JS 校验后从未传出，
  runner 全按 default 分派——M2 主交付）。典型归因：reload 腿
  「state.length expected 2 but got 1」（第二请求被缓存直回放、未到
  fixture）、reload 存储腿 expected undefined ETag 但得缓存 ETag（同根）。
  语料自然显影的 24 腿（default 分派碰对的面）已提前绿。

## 三、零回归核查

- 同窗非本族对照：`eventsource/request-cache-control.any.js` 4P（基线即绿）。
- 独立族抽测：`make testharness-net-api FILTER=fetch/api/headers`
  **286P/0F**（大分母通用闭包路径直通）。
- 结构面：包裹器精确路径门 → 非 cache.py 请求与改前逐字节同路径；
  XHR 通道不经过 fetch 闭包，`*preflight-cache.py` 三件不受影响。

## 四、排障记录（可复用经验已沉淀）

整窗挂起一次：包裹器首版把 `case_cache.lock()....lookup()` 直接写进 match
判别式——**判别式临时 MutexGuard 存活到整个 match 结束**，臂内再锁自死锁
（构建/clippy 全绿无信号）。let 绑定后 guard 先行释放修复。已沉淀
`docs/learnings/bugs/2026-11/2026-11-08-match-scrutinee-mutex-guard-deadlock.md`。

## 五、验证链

- `cargo check -p zero-wpt-runner --all-targets` 干净；
  `cargo clippy -p zero-wpt-runner --all-targets -- -D warnings` 干净；
  `cargo clippy -p zero-net --all-targets -- -D warnings` 干净；
  `cargo test -p zero-net --lib`（test-guard 包裹）497P/0F/1I。
- 提交门禁 lei-pre-commit-guard PASS（3 文件 447 新增行 0 发现）。
- 提交 0e7949fbd → `git pull --rebase`（远端 6 新提交零冲突）→
  666d1f293 推送成功。
