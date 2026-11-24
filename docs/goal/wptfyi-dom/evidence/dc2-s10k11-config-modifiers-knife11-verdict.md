# DC-2 切片 10 刀⑪ 判读证据——SanitizerConfig 校验矩阵＋modifiers 组合语义刀

- 日期：2026-11-25
- 刀面：sanitizer-api `sanitizer-config.html` Fail 17＋`sanitizer-modifiers.html` Fail 8（共 25）。
- 根因族：SanitizerConfig 校验矩阵（validate 步骤 subset/交集/重复归一/data-*）与 Sanitizer 修改方法（allowElement/removeAttribute/allowProcessingInstruction）组合语义整链缺位或方向错置（https://wicg.github.io/sanitizer-api/#valid-sanitizer-config ＋ #sanitizer-modify-methods）。

## §1 实施面（3 文件）

1. `part13-sanitizer.js`（8 处）：
   - canon 元素归一（`_zwSanCanonElAllow`/`_zwSanCanonElPlain`）：namespace member 缺席→XHTML、显式 `""`→null、null 保留（config 页 normalization 脸 `[{"name":"b","namespace":""}]` 期望 null ground truth；属性条目 canon 本已正确不动）。
   - validate 16.2.1.4：条目 removeAttributes ⊆ cfg.attributes 违反 TypeError。
   - validate 17.1.1.5：⊆ 误判改 ∩（条目 removeAttributes ∩ 全局 removeAttributes 非空炸——旧「must be within」是 modifiers 脸构造炸根因）。
   - validate 16.3 独立化：data-* 检查移出 elements 在席块（config 页该脸无 elements 构造）。
   - `allowElement` 重写：归一规则（rm −= cfg.removeAttributes→cfg.attributes 在席 rm ∩ 全局 attributes／缺席且 hadAttrs rm −= rawAttrs；attrs −= cfg.attributes −= cfg.removeAttributes −= rawRm）＋existing 命中替换语义（输入提供 member 归一替换、未提供删键、纯条目重置缺省形）＋removeElements 方向统一允许语义（带 attrs member 或非空 rm 拒、rm:[] 剔除、rWC 移动臂）。
   - `removeAttribute`：元素条目 rm 剥空保留 `[]`（`keepEmpty` 参数；allowAttribute 保持旧行为删键）。
   - `allowProcessingInstruction`：remove 方向重写剥除语义（在名单→splice 返 true、缺席→false）。
   - `_zwSanAttrListEqLoose` 新增（键缺席 ≡ 空数组）：重复调用三臂腿同输入第二遍——归一剥空结果与既有缺席态逐值相同，严格比较误计 changed（modifiers「with global elements and removeAttributes」脸 step3dup 期望 false 实测 true 的根因；纯条目重置分支保留严格版以区分「缺席 vs []」的置 [] 变化）。
   - dataAttributes 缺省后移：`raw.dataAttributes` 缺省 true 只在 validate 之后落（校验期 data-* 检查只认 raw 显式 true）——首射电池 P2F=2 定谳：sethtml-with-custom-elements 三脸 `attributes:["is","data-attr"]` 无 dataAttributes 构造被先缺省 true 的 cfg 误炸 TypeError；「the element is not」脸在 pre 即同因恒败（旧 16.3 在 elements 块内被 elements 在席触发），本刀顺爆转绿（+1 F2P 超出刀面）。
2. `js_dom_bridge_tests/part157_s10k11_config_modifiers_pins.rs`（新）：七组常驻钉测（canon 归一按 name 查找不依赖 get() 的 `_zwSanNsNameCmp` 排序位、校验矩阵 7 形 TypeError、替换语义全序列含三臂 dup false、removeElements 统一允许、removeAttribute 剥空保留、PI 双方向）。
3. `js_dom_bridge_tests.rs`：include 接入。

## §2 验证链

- node 全页复现（part13＋util.js assert_config 全模拟）：config 71/71＋modifiers 19/19 全过（NON_REPLACEABLE_ELEMENTS 2 处为 node eval const 作用域伪影，runner 页 realm 可见恒 Pass）。
- 钉测：`./target/test-guard --compile-first -- cargo test -p zero-engine --lib test_s10k11_config_modifiers_knife11_pins` **1 passed**；终版二进制复确认 1 passed。
- 本刀文件 `rustfmt --check` clean；clippy 仅余他流既有漂移（js_anim_registry.rs redundant_closure／cssa_registry.rs if_same_then_else，均 kc5r86/R2 已提交面，非本刀引入）。

## §3 电池（release 绑定，五腿 solo）

- 发射：`.acceptance/dps/s10k11-launch.sh`（md5 断言先行：v1 `ae392dafc8cae131360038be93f12be4`→dataAttributes 重排后 v2 `3b5da559f1931d0253dc02c9316d11a6`）；pre 锚＝k10 v2 电池（其后 ba010cd4a 仅 docs）。
- v1 电池（ae392daf）中途判读：SAN +23P/−23F 且 comm NEW=2（sethtml-with-custom-elements 两脸 P2F）→停批（kill -9 TT 腿）→根因 §1 dataAttributes 重排→重建 v2 重射全五腿。
- v2 终态（/tmp/s10k11-*-post-knife11-v2.log，17:26:32Z–17:57:22Z）：

| 腿 | pre (k10 v2) | post (k11 v2) | GONE | NEW | 结论 |
|----|----|----|----|----|----|
| WC | 18348P/1120F | 18267P/1120F | 0 | 0 | F 集逐键恒等；P−81＝截断落位（F 构成恒等已知噪声族） |
| DP | 1313P/222F | 1313P/222F | 0 | 0 | SET-IDENTICAL |
| SAN | 558P/198F | 584P/172F | **26** | **0** | F2P=26 精确（config 17＋modifiers 8＋custom-elements 1）/P2F=0/NET +26 |
| TT | 1544P/546F | 1544P/546F | 0 | 0 | SET-IDENTICAL |
| DOM | 57528P/496F | 57514P/496F | 0 | 0 | F 集逐键恒等；P−14＝截断落位 |

- SAN GONE 键清单：config 页 17（normalization 3＋dup 3＋intersect 3＋subset 1＋dataattr 1＋set/方法语义 6）＋modifiers 页 8（allowAttribute 1＋allowElement 4＋removeAttribute 3，两页 Fail 面全数收敛）＋sethtml-with-custom-elements「the element is not」1（顺爆）。

## §4 收账

- 刀⑪ 净增益 **+26**（SAN 558→584）；累计 83,117→**83,143**；距 ≥99,947 差 ≈16,804。
- 二进制字节绑定：终版 md5 `3b5da559f1931d0253dc02c9316d11a6`；判读全部绑定该 md5 断言发射器。
- 残面（下一步刀位候选）：sanitizer-basic-filtering.html 64F、sanitizer-parseHTMLunsafe-with-trustedtypes/javascript-url 22F 族、gethtml.html Timeout 页预算面。
