# 刀 GRID-PLACESHORT 判定收账（2026-11-22 02:0x，watcher/四眼流）

**判定：PASS，net +38/−0，face 入库 0096ff43d**

## 电池链

- 认领笔：`.acceptance/wptfyi-css/NOTES-knife-grid-placeshort-claim.md`（01:35，
  主靶 44F/3 页＝grid-column-shortest-serialization 16＋grid-row-shortest-serialization
  15＋grid-area-computed 19；census 源 GRID-NAMEDLINES 流探针 01:27 复用零重跑）
- 发射：01:42 unit `zw-knife-grid-placeshort`（invocation 03fc06c9），
  rm -rf 并入 unit 命令行（r3 形，r6b 事故教训落实）；
  serialize gate SERIALIZE-WAIT 排队等 S2a r2b 排水（01:42→01:56）
- 脚本：`.acceptance/wptfyi-css/knife-grid-placeshort-battery.sh`；
  patch：`.acceptance/wptfyi-css/knife-grid-placeshort-face.patch`
  （md5 c85622043228032e71aff723abf19e3d）
- log：`.acceptance/wptfyi-css/knife-grid-placeshort-battery-0142.log`
- 产物：`.acceptance/wptfyi-css/results-knife-grid-placeshort/`
  （bin/b0-runner md5 5a0feeccbf7d986918009c89178bd2bc、bin/b1-runner
  f3fd82528405b3ff26926d0bc3cb172e、b0/b1 css-grid.json＋err、
  s1/s2/s3-md5.txt、ride-files.txt）

## 门序全绿

| 门 | 结果 |
|---|---|
| serialize gate | SERIALIZE-WAIT 01:42:03 起，01:56 排水后放行 |
| TREE-COMPILABLE | cargo check -p zero-wpt-runner 过（test-guard 包裹） |
| patch 纯度 | 1 文件（js_dom_bridge.rs）＋marker 在＋外域 grep 净 |
| RIDE census | 10 文件（S2b 五件＋es_module＋testharness 双件＋docs＋脚本）；S1/S2/S3 md5 零扰动 ✓ |
| M2 | b0 5a0feecc ≠ b1 f3fd8252 ✓ |
| FACE-MARKER | b0=0 b1=1（'GRID-PLACESHORT placement shorthand supports'）✓ |
| legs | b0 01:57:44→02:01:00 rc=1；b1 02:01:00→02:04:24 rc=1；JSON 双臂非空 |
| GATE-DONE / EXIT | 02:04:24 rc=0，TRAP-NOOP（face 未落树＝patch-only 保持） |

## RECONCILE（页级＋case 键级）

- grid-area-computed.html: **P 13→23 (+10)**
- grid-column-shortest-serialization.html: **P 0→14 (+14)**
- grid-row-shortest-serialization.html: **P 0→14 (+14)**
- **== TOTAL gained=38 lost=0**；无 ASYM（case 集恒等）；增益精确限定
  三主靶页，零外溢页。

## 红线（P2F fail-stop）——全保位

| 页 | b1 P/总 | 红线 | 判 |
|---|---|---|---|
| grid-column-invalid | 7/31 | ≥7 | OK |
| grid-row-invalid | 7/31 | ≥7 | OK |
| grid-area-invalid | 17/25 | ≥17 | OK |

写路径（e.style test_invalid_value）结构性不受 supports 臂影响，实证吻合。

## 残案逐案归因（12 案，域外不扩 face）

预注册「翻不全逐案挂因不扩 face」执行。b1 残 F 四机制：

- **A：grid-*-end Name 值写路径不落（~8 案）**——`first / first`→got
  `first / auto`（shortest 页 3 案）；grid-area 四值形尾 Name 落 auto
  （`auto / i / auto / i`→got `auto / i / auto / auto` 等 ~5 案）。apply
  存储端 Name-typed end 线回落 auto，写路径域，独立刀位。
- **B：ident escape 串化（2 案）**——`\31st` 串化缺空格（expected
  `\\31 st` got `\\31st`）＋`-zπ` in_array 形。共享串化器域。
- **C：span 1 i 整数 1 省略（1 案）**——expected `span i` got
  `span 1 i`，读侧 span+Name 坍缩缺口。
- **D：calc 域（3 案）**——`span calc(-1)`（calc 求值域）＋
  `calc(...) -a-` 长 hand supports（registry calc 支持面），b0 既红
  非本刀引入。

 census 漂移注记：grid-area-computed 本电池 35 案（b0 P13/F22）vs
 探针 census 19F——探针代际（v3 bin）与 HEAD+ride 代际页案集差异，
 双臂 case 集恒等（无 ASYM），A/B 判定不受影响。

## 预注册对账

- 主靶 F2P 31 期望 → 实得 28（shortest 两页 14+14）：差 3 案全落
  残案机制 A（`first / first`、`first`、`last / last`——supports 门已通，
  暴露更深写路径 Name-end 缺口，域外）。
- 卫星 grid-area-computed ≤19 → 实得 +10：差 9 案落残案机制 A/B/C/D。
- random-computed 掷币案：本刀三页无掷币形，未触发判例。
- 域边界：未触碰 S2a face 三件/GRID-NAMEDLINES/TEXT-INTERP/cssom-insets。

## 主题 3 余池普查情报（后续刀位）

probe census 复用普查：placement 三 invalid 页计数与认领笔红线吻合；
shorthand/template-invalid 家族（grid-shorthand-invalid 34F＋
grid-template-shorthand-invalid 66F 等）落 css-parser track-list 域＝
GRID-NAMEDLINES 对端认领域；template-columns/rows-interpolation
416+436F 归 S2 csinterp 插值域。残案机制 A（grid line Name 写路径）
为本流可认领候选刀位（与对端域不相交，待树窗）。
