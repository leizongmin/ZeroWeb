# 刀 KC-3 contain-intrinsic-size 声明写门电池判定：gained=40 / lost=0＝恰集全翻绿（2026-11-22）

电池＝unit `zw-knife-kc3-cis-decl-gate-r3`，14:49:49 发射 → RECONCILE/GATE-DONE/EXIT
rc=0 15:04:59（全程 ~15 分钟）。face＝`callbacks.rs`＋`part06.js` 双 hunk 纯插入
（patch md5 `8e73bd394928256a6d32b4f33b050ecd`，3 hunk +208/−0）：engine 侧
`cis_decl_normalize`＋`cis_axis_value_ok`＋`cis_split_top_level` 三私有函数＋
`__zw_cis_normalize_decl` 注册块（v8 match/非 v8 "null" 同构 sizing R5334）＋
shim `_zwNormalizeDeclValue` cis 路由臂（五属性 kebab 精确匹配＋!important 剥离）。
HEAD 基＝7f1a7dd77；bin b0 `f2c4e8c1` / b1 `012855c6`（M2 异同过＝face 已烘）。

## 电池协议

- 发射态＝face 在树（反向 apply-check 双驻留协议；r1 前向 check 撞驻留态
  rc=2 修正后 r3 过）。
- ride＝es_module.rs ×1（s1/s2/s3 md5 三核零扰动）。
- 腿序：css-sizing b0/b1 背靠背 → css-contain → css-values（CASE_WALL_S=60，
  test-guard 4G/8G/5400s 全包裹）。
- RECONCILE：键级 flip 全打印＋页级 ASYM 检测＋WATCH 四页逐案。

## RECONCILE（键级）

| 语料 | F2P | P2F | 判读 |
|---|---:|---:|---|
| css-sizing | 40 | **0** | invalid.html 恰集 40/40 全翻（shorthand 11＋长手 8×4 名单逐字命中） |
| css-contain | 0 | 0 | 哨兵零位移 |
| css-values | 0 | 0 | 哨兵零位移 |

TOTAL **gained=40 lost=0** ＝ 预注册红线（lost>0）未触发，恰集全翻绿。
valid.html 24P 恒持（reject-only 不动合法形）＋computed.html 8P 持位，全谱
css-sizing 3442P 基线零丢失。ASYM 零行＝无产物截断。

## 语义定谳（恰集反推文法全命中）

- per-axis `none | auto <length [0,∞]> | <length [0,∞)>`：简写 1–2 轴
  （`auto` 吸附后随长度 token——裸 `auto`、`auto legacy`、`auto legacy auto`、
  句尾裸 auto 全拒）；长手 `[auto ](none | len)`（字节级空白判界对齐
  `parse_cis_longhand`；`auto none` 按 apply 现行为放行）。
- 拒面四族全红：`legacy` 族（parse_length 落空）；百分比（token 尾 `%`
  预滤——`<length>` 不含 %，`10%`/`1px 1%`/`20%`）；裸负值（`-1px`）；
  多 token 残余（`1px 1px`/`1px auto`/`2em 3px 5px`——parse_length 末段
  num/unit 切分锚定整串，残余 token 落 f64 解析失败）。
- 借道拒：thin/medium/thick（parse_length 映 Px(1/3/5) 字符串级预滤）、
  min-content/max-content/fit-content/stretch keyword。
- var()/env()/anchor()/anchor-size()/calc-size()/random() 原文透传
  （sizing 臂同口径）；math 形整体 <length-percentage>＋常量非负
  （apply 臂 `contain_intrinsic_length_is_valid` Calc 分支逐字镜像）。

## 挂账（后续刀）

- valid.html 3F＝specified 级 canonical 折叠（`5px 5px`→`5px`、
  `none none`→`none`、`auto 1px auto 1px`→`auto 1px`）＋
  computed.html 13F＝gCS shorthand 聚合＋vw/vh 解析＋auto 旗标回显
  → **KC-4 另立**（跨 computed_style 字段域＋gCS 桥）。
- KC-1b（cis parse/gCS 53F）清单续挂。

电池产物 `results-knife-kc3-cis-decl-gate/`（b0/b1-json＋bin＋corpus.log＋
provenance）留证。门禁：rustfmt 净（本刀域）；clippy workspace 过（见
master.md 收账行）。
