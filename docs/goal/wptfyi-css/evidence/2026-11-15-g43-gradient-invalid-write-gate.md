# g43 渐变 invalid 写面拒收刀

2026-11-15。driving: css-images gradient-interpolation-method-invalid 292F（0P，
g41 §5 A 刀挂账）。292 形全部为 `should not set` 断言——invalid 渐变声明被
shim CSSOM store 回显原文伪装成功（g30 同型写面通道），分两桶修法。

## 1. triage（292 形二桶分类，探针逐形定谳）

- **桶 A（282 形，无 `in` 子句）**：裸方法词在配置位——`lab lab`/`hsl shorter`
  （无 `in` 前缀）/`red, blue, lab`（空间词在色标段尾）/hue 法倒置
  `shorter hue hsl` 等。parse 必拒 → shim store 回显原文。
- **桶 B（10 形，含 `in` 子句）**：`in` 命中但方法畸形——极坐标缺 `hue` 后缀
  （`in hsl longer` ×4）/`in` 后倒置（`in to right`、`in 45deg`、conic
  `in from 90deg`）/未知空间，parse 宽松降级为无方法渐变**接受**（
  specified=None → 串化 passthrough → store 原文）；悬挂 `in` 3 形
  （`in,`/`45deg in`/`closest-side in`）走桶 A 同通道（parse 失败）。

## 2. 实现

- **parse 严格性**（css-parser `values/parse_transform.rs` 三处调用方各 +5）：
  linear/radial/conic 的 `strip_interpolation_hint` 后加
  `hs.present && hs.method.is_none() → return None`——`in` 子句一旦出现必须
  携带合法 `<color-interpolation-method>`（css-images-4 渐变文法），畸形整渐变
  拒收；此前 `in bogus` 契约（specified None passthrough）同步收紧为拒收
  （canon 钉 None 语义不变）。悬挂 `in` 形不触此门（present=false），由判定器
  覆盖。
- **写面拒收判定器**（css-parser `values/serialize_gradient.rs`）：
  `background_image_decl_invalid(value) -> bool`——括号感知 `split_top_layers`
  复用，任一层渐变前缀（六函数名单含 repeating 变体，大小写不敏感）但
  `parse_background_image` 失败 → 整声明 invalid；非渐变面（none/url/未识别
  函数）恒 false。判定完备性由 parse 保证：valid 页全形 parse 成功（g42 后
  0F 实证）→ 零误伤面；跌不破既有失败（canon None 面语义不变）。
- **回调哨兵**（engine `js_dom_bridge/callbacks.rs` `__zw_gradient_normalize_decl`）：
  判定器 true → 返 `"null"`（grid/anchor 同款哨兵约定，g41 注预留「A 刀另行
  收口」兑现）。
- **shim 臂**（engine `js_dom_shim/part06.js`）：`_grd === 'null'` →
  `return null`（赋值忽略保留原值）。回调未注册（兜底链）零漂移不变。

## 3. 单测与门禁

- css-parser **3,222P/0F**（+2 钉：`test_g43_malformed_in_clause_rejects_gradient`
  9 拒收形 + 3 合法零误伤形；`test_g43_background_image_decl_invalid_judge`
  6 invalid/5 放行/多层混合双向/`url+invalid → true`）。
- engine lib **3,991P/0F**；fmt `--all --check` clean；定向 clippy
  css-parser+engine `--all-targets -D warnings` 零告警。

## 4. 受控 A/B（B0 = results-g42-b1 g42 提交态；B1 = results-g43-b1）

| corpus | B0（g42-b1） | B1（g43-b1） | 净 |
|---|---|---|---|
| css-images | 2584P/631F | 2881P/334F | **+297P/−297F** |
| css-values | 4041P/1546F/6T | 4041P/1546F/6T | **±0 零回归** |

- 逐页（ab-g42.py 复用，案数恒同 3215/5593）：
  gradient-interpolation-method-invalid **F 292→0 全页清零**；
  gradient-position-invalid +4、conic-gradient-calc-angle-percentage-invalid +1
  （判定器溢出红利）；css-values 全量恒一。**REGRESSED=0，grand net +297。**
- 探针先行：invalid 页 solo 直跑 0 Fail 行（292→0）后才发射 corpus。
- g42 双 corpus（values 4041P ±0）作 B0 基线紧贴上刀提交态，单变量归因干净。

## 5. 残面挂账

- gradient-position-invalid 残 5F、conic-calc-angle-invalid 残 3F（位置/角度
  文法细形，另刀盘点）。
- color-stops-parsing 60F；image-set-parsing 114F + image-set-computed 36F
  （6b 指派 C+D 刀）。
- bridge replay 通道（style_from_mutations_lw fold 消费）invalid 拒收未接
  （runner 页面走 shim 通道无语料驱动，扩面 defer）。
