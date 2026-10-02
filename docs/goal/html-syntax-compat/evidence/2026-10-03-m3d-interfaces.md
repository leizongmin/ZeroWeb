# M3 片 d——per-interface IDL 清单（第一波，2026-10-03）

## 结果

**全通道 51391/61303 = 83.83%**（片 c 82.18%，+1011 passes），M1 基线逐案
**0 回归**。reflection 域续升：reflection-forms 7276/8271（88%）、
reflection-grouping 5042/5358（94%）、reflection-metadata 2637/3110（85%）、
reflection-text 9902/10202（97%）。DC-4 实测：make test 68 suites 全绿 +
fmt + reftest 704/704（clippy 无 Rust 变更面维持干净）。

## 修齐簇

| 簇 | 根因 | 修法 |
|---|------|------|
| UINT 反射宽松解析（colSpan/rowSpan/maxLength/cols/rows/start） | parseInt 吞 \v/BOM/nbsp 等非 spec 空白 | 新增 `_zwParseSpecInt/_zwParseSpecNonneg`（spec「rules for parsing (non-negative) integers」——空白仅 \t\n\f\r 空格）；UINT getter 改严格解析 + 范围 [0,2147483647]，越界/低于 min → default（ol.start min:1 补录） |
| limited-unsigned setter（colSpan/start 0 → 应抛） | setter 直写不抛 | UINT setter `min===1 && value===0` 抛 IndexSizeError；数值归一 `String(Number(v))`（"-0" → "0"） |
| input/select.size whitespace 簇 | 同上 parseInt 宽松 | 专用分支改 `_zwParseSpecNonneg` |
| hr.noShade boolean 缺表 | 表外 | `_REFLECTED_BOOL` + noShade |
| pre.width long 反射 | 无 width 数值面（hr.width 是 string 面） | PRE tag 门 long getter（spec parse + [minInt,maxInt] 越界 → 0） |
| base.href/link.href URL 反射缺失 | href URL 分支仅 A/AREA | 分支扩展 BASE/LINK——非空解析绝对 URL、空/缺失返 ''（harness resolveUrl 经 detached-a 组件读取回落原串——空串 parse 失败） |
| link.crossOrigin 枚举 | R3037 MAP 直读遮蔽 nullable 枚举（missing 期望 null 返 ''） | crossOrigin/as 枚举 getter 块移到 R3037 之前：missing → null，合法关键字 ascii 小写，invalid → "anonymous" |
| link.as 枚举 | FLAT 普通串面（junk 逐字返） | as 移出 FLAT，枚举 getter（16 关键字表 `_ZW_LINK_AS_KEYWORDS`，invalid/missing → ''）；setter 仍逐字（R3069 set 分支 + expando 豁免补 as/crossorigin） |
| link.nonce 误反射 | FLAT 命中写属性 | nonce 移出 FLAT——回到 expando 面（spec：nonce IDL set 不反射内容属性，harness getAttribute 期望 previousValue） |

## 残差（片 e 输入）

- forms 尾簇 1337F：form.enctype/formMethod 枚举归一的 IDL-set 面（198）、
  formAction/formAction URL 解析反射（32）、boolean undef 尾（57）。
- embedded 尾簇 2599F（img/iframe 维度与 URL 族）、tabular 1992F（table 章节
  属性族）、obsolete 797F、aria-enumerated 1201F（ARIA 枚举反射语义面）、
  aria-attribute-reflection.html 0/41（testdriver 面）。

## 门禁

- make test（TIME_LIMIT=1800）：68 suites 全绿；make reftest：704/704；
- fmt 干净；全通道 51391/61303 = 83.83%，M1 基线 0 回归。
