# M3——net 层语义前置落定：RFC 9111 五缺口收口 + 全窗口零回归

**日期**: 2026-11-08（net 层语义前置完成；corpus 批次仍挂 46 号 py stash 端点前置）
**前置**: M2 证据档（wire 传播 @ f2f9ee1cb，族 102/102）。

## 一、交付内容（net 层五缺口，全部带 RFC 9111 / draft 锚定注释）

语料核对结论先行：现有 request-cache 族 102 腿全部跑在
`wpt_cache_py_wrapped_handler` 镜像路径上，**引擎 net 层缓存语义从未被窗口
直接检验**——status/no-vary-search/http-cache 三语料目录 20 文件未入窗。
本批把语料逐腿对应的 net 层语义缺口先在真实现里收口，等 46 号 py 端点
前置交付后 corpus 批次即可直接入窗验收。

### 1. §3 可存条件重写（`cache_policy.rs`）

旧 `is_cacheable_status`（2xx 硬门）删除，改 `storable_mode` 按 §3 判：

- 可存基础：final status + 无 no-store + **Expires 存在即算**（任意值，
  含不可解析——`compute_ttl_secs` 对非法 Expires 返回 `Some(0)` 立即过期
  而非不可存）+ max-age / public / private 任一；或状态 ∈ 启发式可存集。
- 启发式可存集 `HEURISTICALLY_CACHEABLE_STATUSES` = RFC 9110 §15.1
  {200, 203, 204, 404, 405, 410, 414, 501}，或带 `public` 标记；
  `private` 有意不入集（保守——规范文本未把 private 单列为启发式依据）。
- 显式过期信息存在压制启发式（含非法 Expires——存在即压制）；
  LM 启发式 = 10% × (Date|now − Last-Modified)，无 LM → 0（立即过期，
  带验证器则 RevalidateOnly）。
- 语料影响：status.any.js 13 状态 × stale/fresh 腿——fresh 4xx/5xx 带
  max-age 落盘（旧实现不落盘，Server-Request-Count 断言必炸）。

### 2. §4.2.2 启发式新鲜度（`cache_policy.rs`）

`heuristic_freshness_secs`：仅无显式过期信息时启用；参考时刻取可解析
`Date`，缺失回退接收时刻（now）；`(参考 − Last-Modified)` 饱和减法后
除 10——负差饱和为 0（时钟内倾）。无 LM 或不可解析 → 0。

### 3. No-Vary-Search 缓存键（新模块 `no_vary_search.rs` ~530 行）

draft-ietf-httpbis-no-vary-search（2026-05 版）解析 + 等价判定 + 键编码：

- `parse_no_vary_search`：RFC 9651 dictionary 解析（key-order 布尔 +
  params 通配/列表/except 三形态 + 组合裁决）；语料优先——`params` 裸键
  /`?1` 按通配处理、except-without-params 按非法回退精确匹配（May 2026
  草案文本相反，Chrome@WPT_REV 与语料 leg 1/8/13 按前者）；
  FIXME: 草案转 RFC 后重估。
- 等价判定：application/x-www-form-urlencoded 对解析 + no-vary/vary
  过滤 + 键序不敏感时按键稳定排序 + 确定性重编码（draft §6.1）。
- **自描述键设计**：`{queryless_base}\0nvs={enc}\0cq={canonical}{vary后缀}`
  ——config 编码进键，内存/磁盘/`resource_index` 注册零结构变更；
  `put_key`/`lookup`/`invalidate`/`ensure_disk_index` 四处经
  `split_nvs_key` 拆解，条目注册在 queryless 基座下；查询时在基座扫描
  等价类（`// OPTIMIZATION:` 注记——仅非默认 config 触发第二遍扫描）。
- `invalidate` 覆盖等价类目标（§4.4 语义——同一条目的任意等价 URI 均
  触发失效；Vary 后缀不参与失效匹配）。

### 4. §3.2 304 合并泛化 + §4.3.4（`http_cache.rs`）

- `merge_304_headers` 从固定四头泛化为逐头合并：§3.1 存储例外集
  （Connection 及其列举字段、逐跳头、Proxy-Auth*）+ Content-Length
  （MUST NOT）+ Content-Range/Trailer（MAY 略）跳过；Vary 合并一次；
  「Content-* 前缀不是信号」——除例外集外全部 replace-or-add。
- `not_modified` 重构：304 头/验证器更新移出「304 自身可存」门
  （§4.3.4——更新 200 响应不需要 304 可存）；ttl/age/must_revalidate
  刷新仅对 `storable_mode` 有解的 304 执行。发现方式：红测
  `memory_304_refreshes_validators_for_the_next_request`——旧代码同款
  偏差被旧 `is_cacheable_status(304)=true` 掩盖，重写状态门后暴露。
- 语料影响：304-update 五腿（含 Connection 列举字段剥离、Vary 合并、
  Content-Length 不更新）。

### 5. §4.4 失效门 + POST/PATCH 存储路径（`resource_loader.rs`）

- GET 落盘形态门 `storeable_response_shape`：非 3xx 且非 206
  （3xx 需重定向链缓存语义 FIXME；206 需 Range 支持——§3.1 不支持
  Range 的缓存 MUST NOT 存部分内容）；§3 可存性本体仍由
  `storable_mode` 统一裁决。既有两处 GET 存储点切换到该门。
- 失效触发面从 2xx 扩到 **2xx|3xx（non-error）**；目标集 = 请求 URL +
  最终 URL + 同源 Location/Content-Location（新增
  `content_location_matches_target`——RFC 9110 §9.3.3）。
- POST/PATCH 响应带同址 `Content-Location` 时写缓存（语料 post-patch
  两腿——PATCH/POST + `Content-Location: ""` 空引用 join 到最终 URL）。

## 二、测试账（2026-11-08，test-guard 包裹 `cargo test -p zero-net --lib`）

**530P / 0F / 1I**（M2 基线 499P → +31：新模块矩阵 + 语料蒸馏腿 +
两个纯函数助手单测）。

- 新模块：解析矩阵（key-order 组合、通配/列表/except、非法回退）、
  语料 13 腿蒸馏等价测试、form-urlencoded 规范化（draft §6.1）、
  编解码往返、键拆解往返。
- http_cache：NVS 通配合并查询变体 / 键序等价 / 命名参数 / 非法精确
  回退 / 失效等价类 / 自描述键再验证命中 / §3.2 例外集合并。
- 两个既有测试改写（旧断言锁死旧状态门 bug——非 2xx 不落盘）：
  `test_cache_non_cacheable_status`（500+max-age=60 现可存）、
  `test_non_cacheable_status_codes`（无依据不入库 + 有 max-age 可存）。
- resource_loader：形态门（3xx/206 排除、4xx/5xx 放行）+
  Content-Location 门（空引用/相对路径/不一致/缺失/不可解析）。

## 三、全窗口零回归账（multiset JSON-vs-JSON，DC-4 门 #3）

终跑 `/tmp/zw-results-httpcache-m3/testharness-net-api.json`（守卫
`--per-proc-mem 4 --total-mem 8 --time-limit 1800`，全窗 ~7 分钟）：

- **NEW vs M2 终账：multiset 全等**——15,185 键（13,868P/1,308F/9T）
  逐键零变化：P→F 0、F→P 0、其他状态变化 0、键增删 0。M3 语义改动
  在既有窗口可观察面零扰动（request-cache 族 102 走 runner 镜像路径；
  net 层真缓存路径语义收口不影响既有语料的既有断言）。
- **NEW vs v3 终账（m4b，13,275P/1,412F/9T/14,696）**：真 P→F = **0**
  （DC-4 门 ✓）；F→P 104（M2 既账——族 102 + mode-same-origin 2）；
  键差 = M1-b/M2 窗口新增 489 净键，与 M2 终账一致。
- 对账口径：multiset（Counter）而非 dict——dict 折叠重复 (file,name)
  键会虚差 1（v3 终账 14,695 vs 14,696 即此伪影）；后续零回归账一律
  multiset。

## 四、划界与挂账

- **corpus 批次挂 46 号**：fetch/http-cache 20 文件（status/vary/
  no-vary-search/304-update/freshness/post-patch + partial 等）未入窗
  ——py stash 端点前置（`testharness_py_routes.rs` 碰撞避让维持）；
  入窗时 `testharness.rs` NET_API_CORPUS_SUBDIRS 加 fetch/http-cache。
- **范围外维持**：磁盘持久化、产品缓存策略、SW Cache API；
  partial.any.js 九腿需 Range 支持（§3.1 形态门已排除 206 落盘，
  FIXME 挂账）；重定向缓存语义（3xx 形态门 FIXME）。
- **NVS draft→RFC 重估**：通配/except 两处语料优先决策带 FIXME，
  草案转 RFC 后按 RFC 文本重估。
- 分区/凭据（credentials partition）、pragma no-cache html、split-cache、
  basic-auth 语料面未实现，挂 M3 后续批次。
