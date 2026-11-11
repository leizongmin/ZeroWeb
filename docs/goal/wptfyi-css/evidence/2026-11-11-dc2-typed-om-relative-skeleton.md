# DC-2 组 E：Typed-OM 相对色骨架保形 + sibling-index()/count() 求值

日期：2026-11-11　批次：DC-2 color 上下文组 E（v8 归类五组之 E）　基线：results-v10（relative-color 族 6 页逐页）+ probe-typedom 定向

## 1. 钉面与期望（v10 基线）

| 页面 | v10 | 失败形态 |
|---|---|---|
| relative-color-with-system-color.html | 1P/2F | test2 got 宿主 resolved 色（expected `hsl(from currentcolor h s calc(32 + l))`）；test3 setter 即拒（`assert_not_equals` got ""） |
| parsing/color-computed-relative-color.html | 1123P/46F | 46F 中 `LCH(from var(--accent) l c calc(h + 180 * sibling-index()))` supported 校验拒（expected true） |
| parsing/alpha-color-parsing-valid.html | 42P/0F | 全绿（`_zwCCDeclAlpha` 已建 sibling 裸函数 valid 锚——decl 面不缺） |
| relative-color-with-zoom.html | 0P/1F | calc 数字形，sibling 无关，零漂移面 |

spec 语义（css-color-5 §relative-colors）：Typed-OM computedStyleMap 返回 computed 值（骨架态）——origin 折叠到 currentcolor 时保形 `fn(from currentcolor <channels>)`（currentcolor 依赖 used value 上下文，computed 时间不解析）；origin 为绝对色（system color/具名色）时 computed 整体解析（test1 accentcolor 宗）。getComputedStyle 返回 resolved 色——两通道必须分开建模。

## 2. 死点链定谳

1. **test2**：computedStyleMap.get → `_zwOmComputedProp` 走宿主 gCS（resolved 色）——无骨架通道。
2. **test3**：setter 早死——`_zwCCDeclRelative` → `_zwCCParseColorValue` → `_zwCCParseCompRel('sibling-index()')`：非 ChanKw/非 calc 文法/非数字 → null → parse 失败 → decl null → setter 拒。
3. **computed 页 supported 宗**：`__zwCssColorValid` → `_zwCCResolveText` → comps3 calc 文本 parse 成功（CalcSpec 认）但 eval 期 `sibling-index()` 无解 → NaN → resolve null → invalid。
4. sibling-index 语义确证：lightseagreen（--accent）lch hue ≈190.11，期望 10.11 = 190.11 + 180×**1** → 1-based 父级子列表索引。

## 3. 修复内容（part10 一处 + part11 七处 + decl canon 一刀 + var 作者臂两刀）

- **part10 `_zwOmRelativeSkeletonText`**：inline 声明 `fn(from <origin> <channels>)` 形态检测（8 色函数白名单 + 括号配平 origin 切分）→ origin 经 part11 出口折叠为 currentcolor 时序列化骨架保形（channels 原文 + sibling-index() 替换，detached 保留原文不编造）→ `_zwOmComputedProp` 对颜色属性（color/-color/fill/stroke）前置骨架检测，绝对色态走宿主 resolved 通道不变（test1 零漂移）。
- **part11 `_zwCCOriginSkeleton` + `__zwCssRelativeOrigin` 出口**：currentcolor 字面 / light-dark 选支（复用 `_zwCCUsedSchemeDark`，runner 环境 light）递归折叠；绝对色/var() 支返 ''（FIXME：var 支未展开）。
- **part11 `_zwCCSubSibling(expr, el)`**：`sibling-index()`/`sibling-count()` 文本替换——el 缺席（supports/decl oracle 支持性通道）哨兵 1，el 存在按 1-based 索引/总数。刀位三处：`_zwCCResolveText` 入口（文本通道全谱）、`_zwCCEvalRel` 尾参 el（三调用点：alpha chanexpr/relative comps/relative alpha 槽）、toOut calc 分支。
- **part11 槽 parse**：`_zwCCParseCompRel`/`_zwCCAlphaSpec` 裸 sibling 函数存 `{chanexpr}`（原 ParseCompRel null 拒 / AlphaSpec chan 槽误判）；`_zwCCDeclRelSlotOk` sibling 函数形文法放行（decl 面只校验文法）。
- **part11 decl canon**：`_zwCCDeclColorCanonic` kind `lightdark` 保形 `{ raw: t }`（probe3 诊断页实证：原折叠为 rgb(0,0,0)——decl oracle resolve 选支丢 currentcolor 语义，骨架面依赖 origin 原文；alpha 先例同形；语料零钉面——color-valid 的 light-dark 锚走 fnHead 白名单外透传不触达 canon）。
- **var 作者臂（独立缺口，computed 页 var(--accent/--color/--mycolor/--mygray/--bg-color) 13 案同根）**：part01c 挂 `__zwPlainAuthorDecl` 出口（`_zwPlainAuthorDecl` 通用 prop 闭包私有化出口化）+ part11 `_zwCCVarLookup` inline 链查空后作者快照沿父链首声明（invalid 页 var 零钉面，零翻红）。

零回归面：sibling 刀全带 `sibling-` indexOf 早退——无 token 值二进制行为零变化；alpha 页 decl 面走 `_zwCCDeclAlpha` 独立校验器不触达本批刀位（现状绿保持）；var 作者臂只补「inline 查空」路径，invalid 页钉面无 var 案。

## 4. 测量（probe-typedom 定向）

probe3（七刀+骨架首测）：**零漂移**——诊断页（zz-diag，跑后即删）实证根因上移：setter decl 归一已把 light-dark origin 折叠为 rgb(0,0,0)（`_zwCCDeclColorCanonic` 通用 resolve 尾段），骨架检测正确判绝对色走宿主；sibling setter 链已通（test3 消息从 setter 拒变 computed 值差）→ 引出 decl canon 保形刀。

probe4（+decl canon 刀）：**relative-color-with-system-color 1P/2F → 3P/0F（+2 页清零）**，其余 5 页逐案零漂移；computed 页 sibling 案 supported 层转绿（消息从「supported 校验拒」变「computed 值差」）——残余为 var(--accent) 展开缺口（同型无 sibling 案 v10 同 F，独立面）→ 引出 var 作者臂刀。

probe5（+var 作者臂刀）：provenance 要求 build 日志 `Compiling zero-engine`（include_str! 陈旧二进制坑——touch lib.rs 强制复编，probe2 曾 0.21s 缓存假象零漂移暴露此坑）。**probe5 终态（2439P/69F）：var 作者臂交付 0 翻转**——computed 页 46F 计数与构成对 v10 不变；test 级 diff 全族仅 system-color 页 2 案翻正（骨架+canon 刀交付）。事后归因（probe5 JSON 逐案消息）：13 个 var 案的根因**不是 var 查找**——blue/orchid 系案 actual 已是语义正确色（rgba(0,0,255,0.8)、orchid 灰 152 表 c=0 折算正确），var 经既有通路已解析；真正缺位在 (a) calc 通道算术（`calc(h + 180)`/`calc(l / 2)`/rgb 亮度混合式/pow() → 解析失败黑底）与 (b) modern 形态序列化（lch/lab/color(srgb) 期望 vs legacy rgb 实际）。作者臂按守卫面保留（作者快照查空补查是 css-variables-4 既有语义缺口，e2final 全语料零回归），后续 calc 通道批若仍零钉面再议回收。

**共享树 stash 事故与恢复**（多实例协作记录）：fe rebase 10 提交前按共享树纪律把本页在飞 js 改动带标签 stash（"e2 typed-om js in-flight"，锚 602543d80），消息交接滞后导致本 session 一度判定改动丢失；恢复路径 `git stash apply`（stash 内=七刀+骨架）+ patch 重放（作者臂两刀落在 rebase 后树上幸存，备份 diff → checkout → apply → 重放），grep 全量核验 + node --check 过。教训：共享树 rebase 的 stash 必须**当场 pop 或在消息里显式交接 stash 名与锚哈希**——本次靠锚哈希快速定位，无 hash 时将靠上下文重放（成本高）。

## 5. 终验记录轮（results-e2final，rebase 4a0d1ba65 上）

对基线全量三语料（css-color/css-values/css-typed-om）：

- **css-color 10,808 → 10,842（净 +34）逐页零回归**，34 翻转按面归属拆分——本批骨架+canon+sibling 刀 **+2**（relative-color-with-system-color 页清零：`light-dark color`、`light-dark + sibling-index()`）；同树在库批兑现 +32（light-dark-image +29 / fill-stroke currentcolor +1 / system-color-compute fill+stroke +2，v10 基线早于该批落库的挂账回收）。
- **css-values 2,213P 对 v10 零漂移**（256 页同口径）。
- **css-typed-om 9,868P 对 v5b 9,872P = -4**——test 级 diff 全部为 grid 家族 'Can set grid-* to var() references' 4 案（grid-auto-flow/template-areas/template-columns/template-rows）：G1 门（part06 `_zwIsGridFamilyProp` 整值委托宿主 grid_cssom）不识别 var() 文法即拒，v5b 基线早于 G1 落库故未暴露。非本批面（part01c 出口只增、part11 sibling/var/canon 刀不触 grid 写面），随 grid var() 放行刀（grid_cssom 单实现双通道一处）修复，见下节 7。

## 6. 挂账

- relative-color-with-zoom 1F（calc 数字形 out-of-gamut 宗，sibling 无关）。
- relative-color-out-of-gamut 20F（色域映射建模，独立面）。
- color-computed-relative-color 残余 46F 三桶：hsl/hwb none 组序列化 ~20F（`50%` vs `50` 形态）、精度 round-trip ~11F（lch/lab/color-mix 嵌套 xyz）、var+calc 通道算术 ~13F（`calc(h+180)`/`calc(l/2)`/亮度混合/pow → 黑底；含 modern 形态序列化交叉）。
- var() origin 支未展开（`light-dark(var(--x), red)` 保守走宿主）。
- sibling-count() part10 骨架通道未建模（FIXME——channels 内 count 钉面暂无）。

## 7. grid var() 放行刀（g2varfix，随批修复）

canonicalize_grid_decl（grid_cssom.rs，native v8 写面 + shim `__zw_grid_normalize_decl` 单实现双通道）CSS-wide keyword 臂后新增：值含 `var(` 即 parse 期放行原文（css-variables-1 §3——var() 声明 computed-value 期才按属性文法重验，G1 门网格文法校验器不识别 var 文法曾误拒）。附带单测 `test_grid_decl_var_reference_accepted_at_parse_time`（4 钉属性 + CSS-wide keyword 不回退 + 非 var 非法仍拒）。

对基线全量两语料（results-g2varfix）：

- **css-typed-om 9,872P = v5b 基线复位，test 级 diff 零**——4 案 'Can set grid-* to var() references' 全部翻回。
- **css-grid 对 v13 共 635 页逐案零漂移**（0 翻转 0 回归）；本跑 case 列表 654 = 635 共页 + 19 数据目录漂移页（subgrid 族 17 + test-plan/index + zz-probe-grid 探针遗留，合计 37P，非本批面——数据目录为 gitignored fetch 面，v13 跑时该批文件未在列表）。探针遗留页 zz-probe-grid.html 已从数据目录清除（diag 证据不占语料列表面）。
