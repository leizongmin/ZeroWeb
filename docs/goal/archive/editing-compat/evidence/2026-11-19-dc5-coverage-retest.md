# DC-5 coverage 复测收口（2026-11-19）

DC-5 执行口径（收口与归档协议）全量兑现：editing/run + editing/other 两腿
逐案 `--json` 实跑进 gap 管线，editing topdir 覆盖率较 2026-11-05 快照
11.41% 抬升至 **92.5%**，editing/run 文件覆盖脱离 0/120 至 **119/127**，
未覆盖子测试三态清零路径成立（**unaccounted = 0**），
`docs/compat/trends/wpt-suites.csv` editing 收口数据行回填。

## 一、批链与判读绑定

- 批 `dc5-corpus.sh`（`.acceptance/dps/`）：06:29:28 → 10:57:23（4h28m），
  逐案 test-guard 包裹（per-proc 4G / total 8G / 墙钟按变体自适应）+
  `--json` 原子落盘 `dc5-results/`（289 产物）；镜像 corpus 脚本枚举与
  预算语义（run 面 `ZW_SCRIPT_TIMEOUT_MS=300000`）。
- writer 轮 429 亡故于在飞窗，批 setsid 存活自跑完，收养轮判读
  （e24 同款剧本）。
- 判读绑定：发射时 bin md5 断言 `40014d7d0bb16b2fa906bd74f45df478`
  （＝e24 六腿全绿验证二进制）；收养轮复核现树 md5 恒一；
  批 head 7db4d85fb → 判读树 9f47b0575 漂移全为 docs 提交，
  `git diff --stat 1cbcb7420..HEAD -- crates/ apps/ tests/wpt-runner/src Makefile`
  为空＝代码树与 e24 锚（1cbcb7420）逐字节同。
- **judge 9 锚全 JUDGE-PASS**（corpus 语义 vs e24 gate 语义零漂移）：
  ip 7185/10、ilb 1691/0、fb 5043/3、fs 2419/3、indent 1329/0、
  img 962/0、mt 10090/2、recursive-exec 1/0、never-throw 6/0。

## 二、coverage 行（coverage-2026-11-05 口径镜像）

分母＝wpt.fyi aligned stable 三家并集（chrome 158.0.8127.36 / firefox
160.0.1 / safari 27.2 @ 71b58bb191，`tests/wpt-runner/wptfyi-cache`），
ZW 实跑文件用 max(浏览器, ZW 实际) 修正；分子＝ZW Pass 子测试，
逐路径 keep-best 不双记。

| 目录 | 分母文件 | 覆盖文件 | 全过 | 分母子测试 | 通过 | 未覆盖子测试 | 通过率 |
|---|---:|---:|---:|---:|---:|---:|---:|
| editing | 888 | 581 | 373 | 117244 | 108448 | 8796 | **92.5%** |
| editing/run | 127 | 119 | 40 | 96809 | 95352 | 1457 | 98.49% |
| editing/other | 529 | 462 | 333 | 16823 | 13096 | 3727 | 77.85% |

- 快照对照：editing topdir **11.41%（13318/116715 @1eecaa0c30）→ 92.5%
  （+81.09pp）**；editing/run 文件覆盖 **0/120 → 119/127**；editing/other
  **2/510 → 462/529**。分母口径漂移（116715→117244）为 aligned revision
  滚动，与快照同约定（快照注亦声明与 run revision 存在漂移）。
- 未覆盖 8796 分解：具名排除/域外路径分母 5425（三态表）＋已执行路径
  真红 F 2878＋pin 漂移缺口 493（13 路径 browser 分母 > ZW 实跑 total，
  上游 pin 后新增子测试）。
- 未解析上报 180 条（171 run + 9 other）＝本地 zz-*/zw-probe-* 合成探针
  文件，非上游测试面，按口径不进分子分母（快照同款「浏览器清单无对应
  路径不记账」）；其中 multitest-ab 10092P 等探针面不入 coverage 账。

## 三、三态清零（判据 3）

| 态 | 文件数 | 子测试数 |
|---|---:|---:|
| 已执行 executed | 581 | P=108448 F=2878 |
| 具名排除 named-excluded | 47（https 4＋iframe 43） | 1275 |
| 上游不存在/域外划界 upstream-absent | 260（域外 232＋pin 缺 28） | 4150 |
| **未记账缺口 unaccounted** | **0** | **0** |

逐文件三态全账（含 47 具名排除清单）见
[2026-11-19-dc5-threestate-ledger.md](2026-11-19-dc5-threestate-ledger.md)。
exec1 474（exec-command-without-editable-element.tentative，iframe 排除
面）按 DC-4 既定 handoff 归 named-excluded，与 user 域决策门留置一致。

## 四、批面汇总与 T 归因

| 腿 | 案数 | Pass | Fail | Timeout |
|---|---:|---:|---:|---:|
| L1 editing/run | 183 | 137184 | 1454 | 31 |
| L2 editing/other | 95 | 13132 | 1728 | 10 |

- T=41 全部落 zz-fbq 探针文件（合成面不入账），真实语料文件 **T=0**。
- 11 案 INVALID-JSON 全为语料筛减面：runner 单文件 filter 枚举仍经
  `editing_case_skipped` 拒跑（undo-redo、typing-around-link-*、
  exec-command-without-editable-element 等 https/iframe 面），输出
  `[]`，三态归类 named-excluded，非执行缺陷。
- runner 变体语义：逐 query 变体名上报（`?N-M` 分片与 universe 分片
  路径精确对齐），逐路径记账无双记（multitest 10 分片、bold 4 分片等
  75 sliced 基全对齐）。

## 五、语料域残红面（记录，不阻塞 DC-5）

- 语料独立跑语义 F=2878（delete 231、inserttext 227、forwarddelete 218、
  removeformat 120、justify 四命令 209 等）＝[wptfyi-editing 流](../../../wptfyi-editing/master.md)
  在案域（d50-d56 轮系列逐键归因与刀序），非本 goal DC-4 三分表未归因键
  ——DC-4 验收账本口径为 gate 批腿面（multitest 窗），其终态
  「fb 3/fs 3/event 4/ip 10/mt 2 全 artifact 挂账＋exec1 handoff」
  于本批 corpus 语义下经 9 锚逐行复认恒一。
- e24 后主线零代码变化，enginelib 名册（4214/0）与 make test 红面归因
  复用 e24/R23 证据（质量门禁「已完成检查直接复用」条款）。

## 六、DC-5 判据对账

1. 两腿 `--json` 实跑进 gap 管线 ✓（本批 dc5-results 289 产物 →
   wptfyi_gap.py 消费）；
2. editing topdir 较 11.41% 抬升 ✓（**92.5%**）；editing/run 文件覆盖
   脱离 0/120 ✓（**119/127**）；
3. 未覆盖子测试三态清零路径成立 ✓（**unaccounted = 0**）；
4. `wpt-suites.csv` editing 收口数据行回填 ✓（2026-11-19 行，全量头条
   不回落——本行仅增 per-goal 数据，不动全量头条）。
