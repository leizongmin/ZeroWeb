# kc5r91 verdict——4.7 门注册 custom var()/env() 间接形预解析单刀（2026-11-25）

## 结论

- **刀面严格加法式**（style-system lib.rs，指纹 `ZW-KC5R91-VARIND-FACE`，b1 bin
  md5 345b13cd）：
  1. **站台门加宽**（~:3309）：原 `resolved_cascaded.values().any(|v|
     decl_value_has_random(v))` 会在元素自身 cascaded 无 random 词时整站跳过，
     var 间接形（raw 无 'random' 词根）构造性不可达——加 `|| iter.any(|(p,v)|
     p.starts_with("--") && decl_value_has_var_ref(v))`。
  2. **循环内加法臂**（4.7 `--*` 支路）：原裸 `if !decl_value_has_random(val)
     { continue; }` 改为先守卫 `!is_custom || !decl_value_has_var_ref(val)`
     继续跳过（标准属性 var 由 4.5 以同 customs 视图代入，维持 skip），注册
     custom 间接形 `resolve_env_and_var` 代入后重查，解析文本含 random 才进
     下方共用支路——缓存键文取消费属性（canonical_key_text），同 var 源同消费
     属性同缓存键同值、异消费属性异键异值（css-values-5 §9.4 auto＝
     element-scoped＋property-index-scoped 双锚）。
  3. **`decl_value_has_var_ref`**（windows(4) `var(`/`env(` ASCII 大小写不敏感
     粗扫，无分配）＋ `#[used]` 指纹 static。
- **零触论证兑现**：skip 分支构造性无 'random' 词（`decl_value_has_random`
  裸 windows(6) 词级前缀扫，`random-item(` 形恒命中上门进 K1 域）——加法臂
  触发域与 K1/K2 既有面互斥，G4 非 random 页 diff=0、哨兵全持为实证。
- **judge/verify 双算 PASS，NET +1**（P 6717→6718）：刀内 1 键 F→P（预注册
  精确 ±0）＋P2F=0 fail-stop＋GONE/NEW=0（9882/9882）＋非 random 页逐键恒等
  ＋独立恒等式 P1−P0=1 == F2P−P2F=1。

## F2P 账（1 刀内）

| 键 | 形 | 通路 |
|---|---|---|
| 'random() shared by custom property in var()' | 注册 custom `--x: random(...)`（var 源）经 `var(--x)` 消费于标准属性（equals＋not-equals 双断言） | 站台门加宽→4.7 加法臂代入→共用支路 |

## 探针门四形（probe2 双臂，b0 60a045c0 指纹=0 / b1 345b13cd 指纹=1）

| 形 | b0 | b1 | 判 |
|---|---|---|---|
| A v1/v2/v3（r90 残面① 逐字）| 3/42397/3 | 191000/191000/65666.8 | 全翻（v1==v2 ∧ v1≠v3）＝消费属性缓存键直证 |
| B math-depth wrapper 哨兵 | 红 | 红 | 恒形（apply 家族持 F，非本刀域）|
| C 混单位 `random(0, 100px)` | 3px | 3px | 规范正确：§9.1 consistent-type violation→invalid→IACVT→@property initial |
| C2 合法形同元素源 `random(0px, 100px)` | 3px | 71.27px | 翻＝gather_custom_properties（lib.rs:4347 步骤 2 自身级联＋步骤 3 initial 兜底）同元素 var 源可评 |
| D 直达 K2 | [] | [] | 恒形（K2 域 r88 已收）|
| vh raw 流通 | raw | raw | 恒形＝未注册 custom 不求值、消费点求值（§9.4）|

数值 run 间漂移＝doc_seed ASLR 已知形（同 run 内 v1==v2 判等不受影响）。

**勘误（收账后独立核）**：本表原转写首发值（b0 3/3/3／b1 67084.4/67084.4/93311.7）
不可复核——probe2 首发（00:38:29–00:38:34）输出被二发（00:39:16–00:39:20）同路径
覆写，上表数值绑定二发 in-situ 工件（probe2-b0/b1/css-values.json，mtime
00:39:18/20）；语义门（双臂 RC=0＋指纹 0/1＋A 形 v1==v2 ∧ v1≠v3 全翻）在在席
工件上逐项复核成立。首发值来源存疑不追认，覆写即收账窗兄弟重跑课再现
（pristine cp 未及落盘）。

## 定谳与勘误

- **残面① 定谳**：r90 §四 'random() shared by custom property in var()'
  held-F 根因＝双层门（站台门 :3309 对无 random 词 cascaded 整站 skip＋循环内
  `--*` 裸 continue）——本刀双层同修，1 键兑现。
- **混单位形结案**：C 形 b0/b1 恒 3px 非刀 miss＝invalid at computed-value-time
  规范正确；语料受害面 grep（(number, number-with-unit) 配对精化）＝零受害，
  r89 移交项 3 撤销。
- **wrapper 族计数勘误**：预注册「6 键」系 r90 §五 行计数转写误差，as-run
  'Shared by property' 前缀集＝**5 键**（color color-mix relative／math-depth
  add()／view-transition-name ident()／font-palette palette-mix calc／
  animation-timeline view()）；judge 程序化前缀集硬门，as-run 5 键全持 F。
  apply-support family 定谳维持（math-depth gCS 全空含 body 基线＝属性暴露层
  未支持；assert_false(equals) 形两元素相等或皆 initial 恒红）。
- **发射事故两连如实记**：bash -c 内重定向先于 mkdir（bash 打开重定向即败
  exit 1）＋脚本缺执行位；第三发 RC=0。课入 NOTES §五。

## 证据链（.acceptance/wptfyi-css/kc5r91/ 本地）

NOTES-kc5r91 §零-§七（布防＋as-run 全账）／probe2-r91.sh 双臂 RC=0（A 全翻/
C2 翻/B·D·vh 恒形）／battery-r91.log（FACE-FINGERPRINT B1=1/B0=0、双腿 RC=0、
DONE 00:46:20）／results-k91-{b0,b1}/css-values.json 满量 257 页/9882 case
（b0 Pass=6717 与 r90 基线逐位一致）／judge-r91.py VERDICT PASS／verify-r91.py
VERIFY-PASS／bins/r91-varind-runner（345b13cd，指纹=1）；代账 kc5r89/bins/
GENERATIONS.txt r91-varind 行。

门禁（收账轮独立复跑）：face 域 fmt 净——lib.rs 在席 1 处 rustfmt 差异
（~:3375 `rewrite_random_item_var_items` 实参折行）经 `git show 69bf56dc6` 零
删除行触该域＋pre-face（b23cae027）同缩进逐字核＝kc5r90 已提交态既有债非本刀
引入，为保 FACE-LIB md5 `c47d70d8` 判读绑定不置理（挂后续 fmt 清偿笔）；clippy
`-p zero-style-system --all-targets -- -D warnings`（systemd scope 24G 包裹）
RC=0 零告警。混单位语料 census 独立复核：number↔unit min/max 配对精化模式
全语料 **0 命中**（宽松扫 207 命中全为 keyword-key＋consistent-unit 合法形，
集中于 random-invalid.tentative.html key 位）＝证据「零受害」结案维持。

## 残面移交

1. 5 键 wrapper 族 apply-support 刀（逐属性另勘，math-depth 暴露层优先）——
   r91 首候选；
2. margin computed 通道 6 位截断精度形（r89 移交维持）；
3. DC-3 对账周账（12-01 周窗口）。
