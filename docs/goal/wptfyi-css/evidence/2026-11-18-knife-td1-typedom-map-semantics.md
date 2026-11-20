# 刀 TD1 · StylePropertyMap 语义补全（registered properties × Typed OM）（2026-11-18）

## 认领与背景

- 认领：[CLAIM-knife-td1-typedom-map-semantics.md](../../../.acceptance/wptfyi-css/CLAIM-knife-td1-typedom-map-semantics.md)；q2 lineage fork 21:2x 正式让位归本流收口（让位记录在 CLAIM 尾段），R2 扩面由其接任。
- 池：css-properties-values-api `typedom.html`（325 子测，RP1 gen3 b1 实测 159P/166F）为主靶；`at-property-typedom.html` 同域挂观察（0P/2F 预存，本刀不扩认领）。

## 侦查修正（探针定谳）

RP1 收口 census 初判「iteration 缺 registered 属性＝宿主枚举缺」被双代探针推翻
（zz-td1-probe.html，frozen b1-runner 首代 + face 二代）：

- 宿主 `el.style` length/item 枚举含自定义属性（ZZ1 `len=1 names=[--zzp1]`）。
- 真机制五族：①迭代值形错——`Symbol.iterator` 挂 values 且 entries 产单值非
  `[name, list]` 对（spec css-typed-om-1 §5.3 iterable<DOMString,
  FrozenArray<CSSStyleValue>>；46x `reading 'length' of undefined`＝
  `filter(e => e[0]==name)[0]` 空）；②computedStyleMap 枚举恒空（fake face
  `length: 0`，13x）；③set 分型 CSSStyleValue 不拒（`--` 臂盲放行，30x）；
  ④registered custom property append 不拒（62x）；⑤`--` 逗号串误拒（8x）+
  computed 值不按注册 syntax 分型（17x assert_false + 2x getAll 计数 + 1x）。

## 码面（3 文件）

1. `registered_properties.rs`——`TypeKind::name()` 逆映射 + `syntax_of()`
   （注册 syntax 描述符串化：Combined 分量 `|` 并置乘子直挂、ident 保大小写、
   Universal/Normal→`*`/`normal`）+ `names_json()`（字典序清单）+ 钉测
   `test_syntax_of_roundtrip`（乘子/ident 大小写/universal/未注册/清单五段）。
2. `callbacks.rs`——`__zw_registered_property_syntax`（未注册空串）+
   `__zw_registered_property_names` 两查询回调（thread_local 直读，无 doc 借用）。
3. `part10-typed-om.js`——
   - set：自定义属性仅受理 string/CSSUnparsedValue（分型值 TypeError——
     css-typed-om-1 §set 值解析步）；非 list-valued 逗号门对 `--` 豁免。
   - append：registered custom property 恒 TypeError（css-properties-values-api-1；
     未注册 `--` 保持既有并置面零扰动）。
   - 迭代：values/entries 产 CSSStyleValue 列表（单值属性亦单元素列表）、
     entries 对形 `[name, list]`、`Symbol.iterator` 改挂 entries、forEach 值列表形。
   - computed registered reify：syntax 乘子分段（`#` 顶层逗号/`+` 空白）→ 逐段
     分型 arm（unit 族 CSSUnitValue/`<custom-ident>` 与 ident 字面量 CSSKeywordValue
     ci 敏感/`<color>` `<url>` `<string>` `<transform-*>` opaque 基类/`<image>`
     CSSImageValue 相对 url 按 baseURI 绝对化/calc 族走既有 calc 树解析
     `<length-percentage>` 混合形 CSSMathSum/unitless zero 宽限）；`'*'` 与解析
     失败回落 unparsed；specified 面恒 unparsed（语料锚定）。
   - computedStyleMap 枚举：inline declared 名单 + 注册表名单（缓存键
     (el, inline 长度, 注册表 JSON)——首代逐名 gCS 预检 × item 每下标重算为
     二次方页超时主因，二代替换为缓存名单无预检）。

## 探针档案（probe-rp1/）

- 首代 face 探针（td1-face-runner，md5 `852b30ca…`）：typedom 321P/3F+1 hang
  ——两缺陷定谳：`<custom-ident>` reify 臂缺席（2F）+ 枚举二次方放大页超时
  （1 hang，页级 60s 超时耗尽）。
- 二代（td1-face-runner2）：**typedom.html 325P/0F 全绿**；at-property-typedom
  0P/2F 预存（@property 规则→运行时注册表接线缺口，b1 同分，池内挂账）。
- 九面诊断探针（ZZ1-ZZ9）双代存档 td1-probe.json/td1-probe2.json。

## A/B 定谳（四语料 × b0/b1，results-knife-td1/）

发射：systemd oneshot `knife-td1-battery-2156.service`（21:56–22:07:35 GATE-DONE，
rc=0 trap-clean）；日志 knife-td1-battery-2156.log。双建：b0 21:56:21→21:57:16、
b1 21:57:16→21:57:58（增量 release ~40-55s/侧）；bin md5 b0 `7daa3f68…`、
b1 `b3083474…`；fp-count（face 指纹串）b0=0 b1=1；S2 ride-along 窗内零扰动
（s419 es_module + s424d part115 + apply.rs 三文件双 md5 相同）。唯一变量＝TD1 face。
共存注记：他流 editing-run multitest.html hang 复现探针并行占 1 核（16 核余量），
本电池四语料皆断言型不受墙钟拉伸影响；二进制替换 rename-safe（其已 exec inode）。

| 语料 | b0 | b1 | ΔP | F2P | P2F | NEW/LOST |
|---|---|---|---|---|---|---|
| css-properties-values-api | 653P/718F | 819P/552F | +166 | 166 | 0 | 0/0 |
| css-typed-om | 11073P/1286F | 11073P/1286F | 0 | 0 | 0 | 0/0 |
| css-variables | 301P/271F | 301P/271F | 0 | 0 | 0 | 0/0 |
| cssom | 1968P/1511F | 1968P/1511F | 0 | 0 | 0 | 0/0 |

**总账：F2P=166 P2F=0 NEW=0 LOST=0，认领净账 +166，P2F 零容忍达标。**
pva 增量恰为主靶 typedom.html 池（探针 159P→325P 局部与全语料账一致）。
异常法证：8 腿全 rc=1（语料存失败＝常态）；.err 全 0 字节；timeout 计数
b0/b1 对称（css-variables 11/11、cssom 1/1，存量相互抵消，零新增）。

## 门禁

- fmt：cargo fmt -p zero-engine 后 FMT-CLEAN（仅触及本刀两函数）。
- clippy：`cargo clippy -p zero-engine --all-targets` 零新增——仅在案域外
  3 lint（animation.rs unnecessary_cast ×2、part115 ride-along doc_lazy_continuation ×1）。
- engine lib：**4191 passed / 0 failed**（18.26s）；钉测 test_syntax_of_roundtrip
  单独复核 1 passed / 4190 filtered（防 filter 空真绿）。

## 残面池（b1 census）

- css-properties-values-api 552F：含 at-property-typedom 0P/2F（@property 规则
  →运行时注册表接线缺口，已挂账）+ unit/keyword 边界族。
- css-typed-om 1286F、cssom 1511F、css-variables 271F：哨兵语料零位移，
  残面归后续刀候选（typed-om 头页归 layout 域池不变）。
