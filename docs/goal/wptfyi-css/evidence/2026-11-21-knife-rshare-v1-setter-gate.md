# 刀 RSHARE v1 判定收账——random() 写面 setter 双门 face（net +3，1 P2F 暂缓入库）

日期：2026-11-21 21:57–22:20 · lane rshare · clone 代发 ZeroWeb-3
域：css-values-5 §9 random() inline 写面（setter 双门假说刀）
预注册：`.acceptance/wptfyi-css/NOTES-knife-rshare-claim.md`（Z2/Z3 双侧同文）

## face v1（patch-only，md5 `e432330752beca545e0c8d508141acdd`，双文件）

1. `crates/engine/src/js_dom_bridge/callbacks.rs` `__zw_validate_decl_value` random 臂：
   值含 `random(` → `substitute_random_values(SENTINEL)` 替换文本裁决（js_dom_bridge
   supports_decl :6263 同口径）；文法 invalid / fail-closed None → 拒；fixed-echo
   保留形 → 放行；无真调用 → 普通裁决。
2. `crates/engine/src/js_dom_shim/part06.js` `_zwNormalizeDeclValue` 顶部 `random(`
   早退：原文放行，跳过颜色归一门（T47 DF-3 null 拒）。

FACE-MARKER 字面 `RANDOM-SHARE v1`（两文件注释，include_str 入 rodata）。

## 电池时间线（Z3 代发，log `knife-rshare-battery-20261121-215751.log`）

- 21:57:51 launcher 发射 `zw-knife-rshare-battery-v1`（Z2 首发两连败 rc=8＝他流
  interpolate-size 半成品 E0433，全员迁 Z3；Z2 21:56 preflight 树净）。
- 21:58–21:59 b0/b1 增量构建 62s/49s；RIDE 空（树净）；M2 双臂 md5 异
  （b0 `119dfb9fac2ed8830b60a7b65d46e51f` / b1 `455ab861bda25ab26cba87250c1a29d1`）；
  FACE-MARKER b0=0 / b1=1。
- 21:59–22:10 四 corpus 腿 ×双臂；22:10:45 GATE-DONE rc=0，trap 反剥
  TRAP-STRIP reversed，Z3 树回 faceless。
- bin 存档 `results-knife-rshare/bin/{b0,b1}-runner`（静态副本，后续补跑复用）。

## corpus 腿结果——三哨绿＋两处账务事故

| corpus | b0/b1 | 判定 |
|---|---|---|
| css-typed-om | 12337P/12337P | SET-IDENTICAL ✓（共享页 byte 同；ASYM 唯一 `zzom17probe.html` b0=39/b1=0＝他流 om17 探针残页混入 b0 页 census，gained/lost=0 免疫，登记不改账） |
| cssom | 888P/888P | SET-IDENTICAL ✓ |
| css-variables | 281P/281P | SET-IDENTICAL ✓ |
| values | **两臂全废** | 首跑 `Unknown command: testharness-values`（corpus 名笔误，正确名 `testharness-css-values`）；补跑后 670P/670P IDENTICAL——**36 页 954 案无一 random 页**（corpus 按需 fetch，random 页从未拉到 Z3 wpt-data） |

**教训**：电池腿判不了 corpus 未拉取的页面——面向子域的刀，发射前必须
`ls wpt-data/css/<corpus>/ | grep <靶页>` 核目标面在库，否则 corpus 腿是空转哨兵。

## 补跑一（corpus 名修正）：`zw-rshare-salvage-v1` 22:13–22:14

存档 bin × `testharness-css-values`（无 filter）→ 36 页 954 案 9s/臂，双臂
IDENTICAL（670P）——证明 corpus 面 face 零效果＝random 页不在库，非 face 无效。

## 补跑二（主靶页级）：`zw-rshare-salvage2-v1` 22:18–22:19

20 个 random 页自 Z2 wpt-data 拷入 Z3（官方上游页，无探针页；`random` filter
恰中 20 页，Z3 原有 62 页无 random 子串）→ `testharness-css-values random`
双臂 5s/臂，case 级 RECONCILE：

```
random-computed.tentative.html: b0 N=154 P=119 F=35 → b1 P=122 F=32  FLIP
  Pass -> Fail :: Property width value 'random(fixed random(1, 2), 10px, 100px)'
  Fail -> Pass :: Property translate value 'random(fixed random(-2, -1), 10%, 100%)'
  Fail -> Pass :: Property color value 'rgb(random(30, 10) random(60, 10) random(90, 10))'
  Fail -> Pass :: Property background-image value 'image-set(url("http://example.com/image.png") calc(random(fixed 0.3…'
  Fail -> Pass :: Property rotate value 'random(1, 0) random(1, 0) 1 90deg'
== TOTAL gained=3 lost=0（20 页 398 案）
```

其余 19 页全 0 位移；三绿哨页：random-invalid 39/39、random-item-valid 21/21、
random-item-invalid 14/14（v1 词门对 invalid 形零触碰 ✓ 预注册兑现）。

## 判定（对预注册）

- 预期 +6~+12 未兑现，实际净 +3（4 F2P − 1 P2F）。
- **假说部分证实**：color 空格形 rgb、rotate 确系 setter 双门（P17/P18 机制）F2P；
  translate（% fixed-echo 形放行后 gCS 可服务）、background-image image-set
  连带 F2P（未预注册，同门受益）。
- **假说部分证伪**：margin 简写 2 案（P20 文法门）、aspect-ratio、filter、scale
  全部留 F——setter 门非其唯一根因，v2+ 需逐案归因。
- **P2F 一案**：width 嵌套 `random(fixed random(1, 2), 10px, 100px)`——§9 禁止
  random() 嵌套；b0 靠 setter 拒收通过（拒＝规范正确面），face 的 SENTINEL 替换
  对嵌套形宽松求值成功（`substitute_random_values` 未实现嵌套拒绝）→ 放行后
  computed 值偏离期望集 → 反挂。
- **face v1 不入库**（P2F 未清偿不落码）；patch 与双臂 bin 存档 `.acceptance/`。

## v2 设计（定稿待窗）

`__zw_validate_decl_value` random 臂、替换前加嵌套检测：括号扫描维护
「是否处于某个 `random(` 括号域内」栈，已在 random 域内再遇 `random(` → 直接拒
（§9 语法禁嵌套；b0 拒收面即规范面）。FACE-MARKER 升 `RANDOM-SHARE v2`。
验证＝重建 b1' 重跑 5s 页级腿（期望 122+1=123P、0 P2F、net +4）＋random-invalid
哨页复绿。

**重建窗口约束（本轮未执行的原因）**：Z3 树驻 zwv3 流 part03/part10 face
（typed-om 刀 WIP）、Z2 树驻 g1a 流 interpolate-size 四站＋part03/part10 staged
WIP——任一树重建 b1' 都会把他流 face 扫进二进制（双向污染）。v2 重建待安静窗。

## 残余挂账（下一轮素材）

- random-computed 残 32 F：cluster A 支持性门 9 案、font-size % 基 ×3、
  --percent/--random-length 注册 cprops 替换、translate fixed-echo gCS 服务、
  margin 简写真因、:510 Maximum random 机制。
- P14 fixed 150000px WebKit 数值公式差（维持挂账）。
- random-in-animations 24F / random-item-computed 23F / random-in-custom-function
  8F 等页级族（v2+ 域）。

## 工件索引

- face patch：`.acceptance/wptfyi-css/knife-rshare-face-v1.patch`（Z2/Z3 双侧）
- 电池/补跑脚本：`knife-rshare-{battery,salvage,salvage2}.sh`（Z3 ACC）
- 日志：`knife-rshare-battery-20261121-215751.log`、
  `knife-rshare-salvage-221341.log`、`knife-rshare-salvage2-221844.log`（Z3 ACC）
- 结果：`results-knife-rshare/{b0,b1}/*.json`＋`bin/`＋transcript ×2（Z3 ACC）
- 飞书通知：22:1x watcher GATE-DONE 一发；收账一发
