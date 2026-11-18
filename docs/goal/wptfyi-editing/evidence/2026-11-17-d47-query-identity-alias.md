# d47：querySelector 身份别名——方言冻结消双派 + 宿主同节点判定回调

日期：2026-11-16/17。前置：master.md 遗留账 bi=2 双派族（d41 诊断 462F；
d42 归一刀 shim 侧跨方言键归一净 −27 持位；d46 探测循环再次实证
`initializeTest` 身份检查 `gEditor !== document.querySelector("div[contenteditable]")`
翻面 → 监听器重复注册 → `expected 1 but got 2`）。本刀换臂：**等价判定下放
Rust 宿主**（两个方言串各自在同一查询视图内解析、比 NodeId），shim 侧只做
别名冻结不猜等价——d42 教训（shim 侧归一改写破坏其他面）不再重复。

## 落码（4 文件，+95/−2）

1. **callbacks.rs +34**（唯一 Rust 变更）：宿主回调 `__zw_same_node_sel(a,b)`
   ——复用 `with_query_view_doc`/`uniq_sel_ctx`/`find_by_selector_cached`
   既有管线，同一查询视图内解析两 selector，NodeId 相等返回 `"1"`。零新状态。
2. **part05.js +41**：`_zwQueryAliasByRawSel` 别名表（raw selector → 首中选
   方言串）+ `_zwQueryIdentityAlias(rawSel, hitSel)`（宿主判同节点 → 返回
   既有代理保持身份；判不同节点 → 清陈旧表项防悬挂）+
   `_zwQueryIdentityRecord(rawSel, proxy)`（handle 代理跳过、512 项上限）。
3. **part04.js +12**：qSA R310 臂容器作用域别名门
   （`_qid47k = sel + ' ' + q`）。
4. **part06.js +10**：qS 返回点别名门（`!_s44stale` 分支）。

shim 全部消费点带 `typeof __zw_same_node_sel === 'function'` 降级守卫（宿主
未注册时行为同 d46）。

## A/B 轨迹（基线 = d46 终态二进制 1135P/2189F/4T，zz-probe 排除口径）

d47 WIP 二进制（/tmp/d46-tgt 重建，含 4 文件落码）：全语料
**1466P/1862F/3T，净 +331**；二次复跑逐键复现（/tmp/d47-ie2.json，1466P
恒等）。

账目闭合：共享键翻转净 +330（F2P=351 − P2F=21）+ 新键 +1P（`?Backspace,ul`
尾段补录 4 键中 1P/3F）− gone 0 = **+331**。

- **F2P=351**：deleting-in-list-items 4 变体 **+339**（d41 bi=2 双派族——
  d46 失败信息逐条实证 `One beforeinput event should be fired … expected 1
  but got 2`，别名冻结后单派翻绿）；non-collapsed 3 变体 +12。
- **P2F=21**：forwarddelete **9**（`abc[]<p>` div-join/跨容器 join 与 li-join
  族）+ non-collapsed **7**（`abc[<p>]` 部分选区跨元素族 6 + `abc{<br>}`
  innerHTML 1）+ backspace **1**（`abc<p>[]def` innerHTML）+ deleting-in-list
  **4**（`div{  <ol/ul><li><br></li>…  }` 空白装饰族，d45b/d46 空白门 P 案
  同形）。
- 超时 4→3：d46 `?Backspace,ul` 变体文件级 Timeout（超时中断、尾段子测键未
  注册）本刀跑完消失，尾段 4 子测键补录。
- 失败形态位移（计数不变）：部分旧 dil F 面从 gtr 断言红位移为 bi=0 红
  （别名冻结方言与桥 eval 路径解析方言不一致 → 派发键失配）。

## P2F 归因：双世界投影病（master ③）+ 收敛悖论实验（已回退）

P2F 机理（静态追踪 + 实验交叉证实）：别名把页面侧代理冻结在首查方言；桥
eval 路径（`__zw_ce_*` → impl `document.querySelector`）解析出另一方言的
代理 → `_dispatchWithBubble` 按派发代理冻结键收集监听器 → 与页面注册键失配
→ bi=0。d46 该路径靠方言振荡偶发对齐（bi=1 但 gtr 常红）；本刀冻结后对齐
反而失稳——21 面翻红即此。

**收敛实验**（part01.js CE impl 5 入口加 `_zwCeConvergeHostProxy(el)` 统一
el 身份 + part05.js 助手；实验二进制 = 本刀别名 + 收敛钩子，孤立轨
/tmp/d47d-*.json，与 WIP 终态共享键对拍）：

| 家族 | F2P | P2F | 净 | 判读 |
|---|---|---|---|---|
| forwarddelete | +9 | −3 | **+6** | div-join 面救回 |
| backspace | +5 | −3 | **+2** | 同上 |
| non-collapsed | +10 | −32 | **−22** | 救 10 面、毁 32 面 |
| deleting-in-list | +0 | −372 | **−372** | 别名 +339 胜利全灭 + 再毁 33 面 |
| 合计 | | | **−386** | 净负，回退 |

**收敛悖论定谳**：div-join 族需要 impl 拿页面族 el（直接命中页面监听键，
免世界同步）；dil/nc 表格与列表族需要 impl 拿桥方言 el（世界同步干净房
重建依赖它，收敛 el 断开重建链）。两族要求**相反的 el 身份**，入口级一刀
收敛不可能双赢——master ③（generation-aware 归一刀）才是正确解域。已
`git checkout` 回退 + Edit 移除助手，`git diff --stat` 复验恒等 95/2，
重建后全语料复跑 1466P 逐键复现。

## 哨兵（WIP 终态二进制）

- html-semantics（950 文件）：/tmp/d47-sem.json **P 恒等 5378**，共享键
  F2P=0/P2F=0；仅已知 select-home-end 振荡族 2 键本轮入册（F 425→427，
  s528/R57 系先例同族，非本刀域）✅
- selection（738 文件，probe 排除）：`/tmp/d47-sel.json`（00:08，终态二进制
  实测，jdwsp 定向探针逐键复现实证同二进制 601P/2855F）**43,386P/9,822F**
  vs d46 基线 `/tmp/d46-sel.json` 48,942P/4,312F——**净 −5,556，F2P=6/
  P2F=5,562** ❌（详判决节）

## 守卫与门禁

- 语料运行同 d46 守卫链：`./target/test-guard --per-proc-mem 4 --total-mem 8
  --time-limit {600,1500,2700} -- <runner> testharness-{input-events,
  html-semantics,selection} --json`。
- Rust 变更（callbacks.rs +34）：fmt 干净；clippy 仅既有警告
  （computed_style.rs:2750 unused `n`，非本轮写面未触碰）；engine lib 全量
  `cargo test -p zero-engine --lib`（dev-guard 24G scope + test-guard +
  独立 CARGO_TARGET_DIR=/tmp/d47-libtgt）**4076 passed / 0 failed**（较 d43
  记录 4059 增 17 = 并行流新落钉测，零失败）。
- 探针页 zz-probe-d47{a,b,c}.html 在 tests/wpt-runner/wpt-data/input-events/
  （gitignored，不入库）；运行产物均在 /tmp。

## 判决：不落（selection 哨兵否决，2026-11-17 补测收口）

上 session 在 selection 腿完成后 429 中断，哨兵未分析。本轮补测定谳：

- `selection` 全语料（738 案，probe 排除口径同 d46）：48,942P/4,312F →
  **43,386P/9,822F，净 −5,556（F2P=6/P2F=5,562）**。input-events +331 被
  selection 面全额吞没，合并净 −5,225，按 d42 前例（A/B 净负不落）判决
  **不落**，WIP 四文件 stash（`d47 query identity alias WIP (rejected: …)`
  保留配方），码态回 d46 终态。
- **P2F 族谱**（全在 join/innerHTML 断言面）：jdwsp
  join-different-white-space-style-paragraphs −2,855 + 左段右行 −1,299 +
  左行右段 −1,295（三族合计 −5,449）+ join-pre-and-other-block −64 净
  （P2F=65/F2P=1）+ dwuf −24 + plaintext-only insertLineBreak −24；
  F2P=6 = insertparagraph-in-non-splittable +5、join-pre-and-other-block +1。
- **机理判读**（与收敛悖论同解域）：别名门把首查方言冻结为身份；join 族每案
  `initializeTest` innerHTML 重写后同 raw sel 重查——旧方言仍在当前视图解析
  出同位置节点 → 同节点门放行 → 返回**旧查询视图**的代理，其 handle 指向
  重写前节点，innerHTML 读面读旧投影 → 断言大面积红。bi=2 dil 族要身份
  连续（跨换代保 listener 键），join 族要换代后读新投影——两族对「换代后
  旧代理该不该续用」要求相反，查询返回点一刀同样不可能双赢。**master ③
  generation-aware 归一刀再获负样本**：入口级（impl 入口 d42/收敛实验、
  查询返回点本刀）两处均判负，正确解域只剩按查询视图代际分桶。
- **二进制污染记档**：`/tmp/d46-tgt/release/zero-wpt-runner`（127,561,648
  字节，23:39:44 建）现为 **d47 WIP 二进制**，不再代表 d46 基线；后续 A/B
  基线二进制须重建，勿复用。d46-sel.json（21:21）仍是有效 d46 基线产物。
- d47-sel.json 与定向探针（本轮 00:35，601P/2855F 逐键一致）双跑复现，
  负载波动解释排除；`/tmp/d47-seljoin.json`（0 字节）为上 session 中断残留。

