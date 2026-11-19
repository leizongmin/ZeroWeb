# nth-child rebind 探针撞头仲裁——本流独立实现撤改，采纳 site-opt PR #263（2026-10-31）

## 1. 背景与独立发现

簇 A 插入侧轮（R54xx 后续，S5 ≤ 300 线未达）做 rebind 环墙钟探测时独立发现：D1 sel-path
wire / R5331 刷新环的 plain 寻址选择器（`_zwPosSelOf` 位置串，`:nth-child` 链）在宽兄弟
列表上是 O(候选×兄弟) 全扫——`compute_element_position` 每候选两趟兄弟遍历。

探针页 `editing/run/zz-att1600.html`（语料 gitignore 域）：1600 span 宽宿主 + rebind
刷新环（1601 对 handle 重绑，每对 `querySelectorAll` + `querySelector` 形如
`html:nth-child(1) > body:nth-child(2) > div:nth-child(4) > span:nth-child(N)`）。
单案 filter 直跑：`make` 通道外以 `./target/test-guard --per-proc-mem 4 --total-mem 8
--time-limit 600 -- ./target/release/zero-wpt-runner testharness-editing-run zz-att1600`
驱动。

## 2. A/B（同日 21:25–21:45，基线 eb141f7e1 净树 vs 本流查询作用域索引实现）

| 树 | 单次查询 | refresh-probe（pairs=1601） | 案结局 |
|---|---|---|---|
| 基线（eb141f7e1） | qsa 16ms / qs 0-15ms（逐次） | render 14ms / **rebind 42,217ms** / snap 43ms / total 42,275ms | Pass 后 **Timeout**（completion callback 未达，42s rebind 撞墙钟） |
| 本流索引实现（撤改存档 wip/editing-nth-child-query-scope-index b88d132f0） | qsa 0ms / qs 0ms | render 17ms / **rebind 217ms（194×）** / snap 32ms / total 267ms | **Pass**（attached=1600 ms=292.7）无 Timeout |

中间插桩轮（/tmp/zz-rb3.txt）：1601 次 rebind-probe 逐条打点，qsa 恒 16ms、qs 0-15ms，
单点成本确认为 `:nth-child` 全扫非 DOM 构造（render 14ms、html_len 223,332 恒定）。

## 3. 撞头仲裁

- 对方：site-opt 流 PR #263（`site-opt/p125-query-nth-child-wedge`，OPEN，precheck
  pass）——github.com/trending 真站 renderer 楔死（SetDomSnapshot 单次 20.3s）同根因：
  thread_local 每查询位置索引（miss 回填 per-parent 全元素子 ElementPosition）+ 廉价成分
  先行 + shadow 变体覆盖 + 位置索引 merge-base 等值回归钉（修 shadow 直接子位置伪类翻转）
  + dom crate 回归测试与真站 fixture。
- 本流：Document 字段查询作用域索引（`query_position.rs`，RAII scope，最小字段索引），
  A/B 同判。**同根同判撞车，对方先落地（18:32 commit / 19:14 PR），且覆盖面更全（shadow
  + cheap-first + 测试钉）**。
- 仲裁：按本 goal「消费域收敛、不回退对方修复面」先例，本流同面机制撤改零残留——工作树
  已还原、实现全文存档本地分支 `wip/editing-nth-child-query-scope-index`（b88d132f0，不
  push）；本流 A/B 与插桩归因作为对方修复面在**编辑域 rebind 环**的独立佐证入库。

## 4. 编辑域影响评估（记性能余量，非语料阻塞项）

- 现行语料宿主树窄（几-几十子），`:nth-child` 单扫 µs 级，bold 墙钟 117s 账内 rebind 占
  比可忽略；42s 形态只在 1600 兄弟级压力页现形（合成探针）。
- 簇 A 验收线不动：S5 ≤ 300（现 346.6）的 villain 仍是 s111 臂 / CE loop / removeChild
  glue（removeChild 首刀已由并行流 R5340-glue 收口），与本撞头面无关。
- PR #263 合并后编辑域顺带受益（rebind 环、plain 寻址查询、真站宽页面嵌入场景）。

## 5. 销账条件（挂账）

1. PR #263 合入 main 后，重建探针页复验 rebind 环 ≤ 1s 同带（0F、无 Timeout）。
2. 探针页已失（本轮 21:47 清场或插桩二进制剥离后无法再生；`ZW_PROBE_REFRESH` 为已剥
   临时臂，净源码无此变量）。重建配方：1600 span 同父宿主 + 1601 对 handle 重绑环 +
   上节 `:nth-child` 链 selector 形态（/tmp/zz-rb3.txt 逐条实录，瞬时目录、以本文件
   §2 表为持久凭据）。
3. 若 #263 弃置/撤回，从 wip 分支复活本流实现收尾（补 shadow 覆盖 + 回归钉后另行走
   PR 流程）。
