# M2-S14 — request-disturbed init 给体不扰动源：spec 演进翻转（2026-10-29）

**切片**: [M2-S13](2026-10-29-m2-s13-corpus-drift-sweep.md) 记账 1（request-disturbed
期望翻转挂起）专项。**引擎语义翻转 + 语料同步对齐**——M2-S13 定界的互斥约束
（单独动任一边都翻红）本轮双侧同步解。全通道 A/B 对直接前轮基线
`netapi-m2s13-r1.json` 逐 subtest（结果见「验证」）。

## 语义与规范依据

上游于 2026-10-29 当天（M2-S13 巡检发现的 13h 窗内）改写
`fetch/api/request/request-disturbed.any.js` 期望：

- 旧（本地既有）：`new Request(req, {body: "init body"})` 后源 req
  `bodyUsed === true`（「became disturbed even if body is not used」）。
- 新（上游 HEAD）：`bodyUsed === false`（「with a new body is not
  disturbed」+「bodyUsed is false when init supplies a new body」）。

现行 fetch spec 构造器文本（fetch.spec.whatwg.org #dom-request）只在与
input 体交互的分支触碰源体：`Let finalBody be inputOrInitBody` 后，**仅当
`initBody is null and inputBody is non-null`** 才进入「unusable 检查 + 为
inputBody 建 proxy」分支；`initBody` 非空时 `finalBody` 直取 init 体，源体
全程不触碰——不扰动。旧引擎行为（init 给体也标源 used）对应早先的 spec
模型，上游语料随 spec 演进翻转。

## 实现面

- `crates/engine/src/js_dom_shim/part01.js` Request 构造器尾步扰动标记加
  `!bodyGiven` 作用域：`if (isRequestLike && !this._bodyNull && !bodyGiven)
  input._bodyUsed = true`——`bodyGiven = init.body != null` 既有校验器标志，
  标记仅落在「采纳 input 体」形态（initBody null + inputBody 非空）。注释
  同步改写并补 spec 链接；构造失败不标、「became disturbed」（无 init 体）
  旧腿语义不变。
- 前置 unusable 检查本就按 `!bodyGiven` 作用域（M2-S3 既有），与 spec 新
  分支结构一致，无需动。
- 语料 `tests/wpt-runner/wpt-data/fetch/api/request/request-disturbed.any.js`
  逐字对齐上游 HEAD（`cmp` 字节一致，M2-S13 巡检留存的 UP-master 副本）。
- `scripts/update-wpt-data.sh` 头部缺口清单 M2-S13 预警条目更新：翻转已
  双侧同步完成，新 ref 落地该腿为绿，无需人工干预。

## 验证

- 引擎单元测试（`cargo test -p zero-engine`，test-guard `--compile-first` 包裹）：
  3592 过 / 10 败——10 败为**长期预存**（stash 干净树与 M2-S11b 回溯均同一
  集合，与本切片无关，记账 2），本切片前后零新增失败。
- 单文件探针（`make testharness-net-api FILTER=request-disturbed`）：
  request-disturbed.any.js **5/5 Pass**——新腿「Input request used for
  creating new request with a new body is not disturbed」绿；无 init 体旧腿
  「became disturbed」保持绿；「creating a new request with a new body from
  a disturbed request」（unusable 拒绝）保持绿。
- 全通道 A/B（`netapi-m2s14-r1.json` vs 直接前轮 `netapi-m2s13-r1.json`，逐
  subtest，对账输出 `.acceptance/net-api-compat/abdiff-m2s14-r1.txt`）：
  **0 增益 / 0 回归 / 0 翻转**——唯一变化为改名腿对（removed 旧名
  「became disturbed even if body is not used」基线 Pass + added 新名
  「with a new body is not disturbed」本轮 Pass），pass 11000 → 11000，
  分母 12077 持平；其余全通道逐条零变化；stderr 0 panic（0 字节）。
  记账口径：本切片为 **spec 对齐零净账**——当日翻绿靠 M2-S13 语料巡检
  （+1 假红腿清偿），本切片保障的是语料滚动/引擎演进后该腿不翻红。

## 记账

1. **M2-S13「13h 窗漂移」叙事修正**：net-api 语料有正式 pin——
   `tests/wpt-runner/scripts/goals/lib.sh:14` `WPT_REV=315976933870b…`（上游
   2026-08-12 快照），拉取幂等（已存在即跳过），本地 mtime 只是首拉时刻
   （00:19–00:39）非内容时点。M2-S13 的 9 个 differ 实为 **pin（2026-08-12）
   → master HEAD（2026-10-29）约 2.5 个月漂移**，非 13 小时窗；巡检的分类
   与处置结论不变。S13 档记账 3 已加修正指针。语料对账基准自此明确为
   WPT_REV pin；滚动 pin/语料仓 tag 属既有待决项。
2. **预存引擎单元测试红（非本切片引入，不动）**：`cargo test -p zero-engine`
   存在 **10 个 js_dom_bridge 预存失败**（3592 过 / 10 败），失败模式
   `TypeError: Invalid URL: /api`——`V8Sandbox` 桥测试以 `about:blank` 为页基址
   构造 `new Request('/api')`，相对 URL 在 about:blank 基座下严格拒绝。归因
   证据：① stash 后干净树同 10 失败（本切片改动前置同一集合）；② 回溯
   `4a8a8e27c`（M2-S11b，先于两流近期活动）同样失败——长期预存；③ 本切片
   通道探针 5/5 绿。涉及面含 8 个 fetch/Request 域测试（fetch_sync_wire /
   cross_origin_origin / forbidden_headers / credentials / body_readers /
   headers_guard / signal_passthrough / response_request_constructors）+
   2 个 DOM 域（details_name_exclusivity / iframe_service_worker_messages）。
   V8 桥 about:blank 基址面归引擎域独立切片，未动（共享面不越界）。
3. S13 记账 1（request-disturbed 翻转挂起）**清偿**，见本档；S13 档已加指针。
   update-wpt-data.sh 缺口清单对应预警条目同步更新（新 ref 落地该腿为绿）。
