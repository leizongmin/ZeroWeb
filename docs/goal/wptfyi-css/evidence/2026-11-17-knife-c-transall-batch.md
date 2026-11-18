# 刀 C + trans-all + WAAPI 校验批（2026-11-17）

rate-limit 中断轮孤儿 WIP 认领收口。批内三件套（引擎三文件，一提交一批账）：

1. **transition `all` 通配展开**（css-transitions §2，`FIXME(transition-all)` 逾期清偿）：
   `register_from_style_change` 按 `TRANSITIONABLE_PROPERTIES`（48 项严格可插值清单）
   通配展开，duration/delay/timing 取 `all` 项自身位；`register_property_if_changed`
   单属性门共用（duration≤0 / 端点空串同值 / 严格门不可插值对跳过，echo 池零扰动）。
   `query_core` eased==1.0 短路增 rule-\* 四属性例外——端点广播/有限 repeat 展开语义
   （to 单值对 from 多值位输出 `to, to`）回落插值臂，echo 原样形欠展开位。
2. **computed 值域钉（刀 C）**：`rule_width_list_to_gcs_computed` /
   `rule_color_list_to_gcs_computed` gCS 读面专用串化——静态 computed 值 repeat 结构
   保形（gap-decorations-{width,color}-computed 解析页钉；"Repeaters should expand"
   仅属过渡/动画插值输出臂），列表项与 row 单值 px 取整（列表项 `22.5px`→`22px`、
   row 单值 `58.5px`→`58px`），column 单值小数保形（css-multicol
   column-rule-width-interpolation 页 `8.1px` 铁锚）。指定值面（inline CSSOM 简写
   重组等）沿用原 `rule_width_list_to_gcs` 不动。
3. **WAAPI keyframes 值校验**（B3 −1 外证闭环认定的独立 hardening）：合成 keyframes
   文本通道经 `validate_decl_value` 门（与 CSSOM setProperty 同一校验器），非法值
   **丢该属性**（键塌缩 → 端点回落 underlying，Web Animations §
   processing-a-keyframes-argument，Chrome parity），合法值零变化；
   `ZW_ANIM_VALIDATION_DEBUG` 环境门 stderr 探针（callbacks.rs 同款先例）。
   惰性实证：b3d 链含终态校验五 corpus 页/子测集合全等零翻转（652a04f66）。

## 双基线 provenance 法证

| 链 | HEAD | 二进制 md5 | 内容 |
|---|---|---|---|
| b3c（B3 权威基线） | `71b07c7d8` | `58ec871f` | 纯净 twin B3 码态 |
| **head0（本批 B 侧）** | `46c3df57a` | `71851abb` | HEAD 纯净（含 editing ilb WSCANON 双修复 `4e091e6f7`/`7356bf2aa`；刀批 + es_module s419 WIP 全 stash 窗内构建，发射前回植） |
| c2（本批 A 侧） | `46c3df57a` | `97318db` | HEAD + 刀批 WIP（引擎三文件；es_module stash 不进二进制；style-system 刀 D 字段 03:10 晚于 v2 构建**不在**批内，已撤回存 patch 待整刀） |

二进制 markers 法证（strings 三点判据）：head0 无 `ZW_ANIM_VALIDATION_DEBUG`、无
es_module `__zwImpS$`；c2 含校验 marker（阳性对照）；head0/c2 同含 WSCANON
`_zwCeIsCollapsibleWsOnly` ×10（HEAD shim 代际一致）。

- 构建 peculiarity 备案：head0 构建仅重编 webview+wpt-runner（41s）——engine
  release 工件 fingerprint 命中 HEAD 态缓存（farm 构建 `cp -al` 共享 inode 代际
  产物），markers 三点判据实证二进制=HEAD 码态，非陈旧扫入。
- 首发事故备案：head0 链第一次发射（04:24:01 unit `zw-head0-chain-1501100`）
  systemd-run 无 WorkingDirectory → runner wpt-data repo 相对路径全空（gaps
  cases=788 pass=0 形、multicol cases=720 pass=0 形）——**空真绿坑**新形：
  corpus 链经 systemd-run 发射必须 `cd $ROOT` 包进内层 bash -c（run-rules
  systemd-run 重定向内层同源教训）。废账 results 目录已删，日志改存
  `/tmp/zw-transall-head0-chain-broken-cwd.log` 留证；重发射 04:25:04
  （unit `zw-head0-chain2-1545446`）五腿 rc=0。

## 权威账（c2 vs head0，五 corpus 逐子测对账）

| corpus | head0 | c2 | Δ |
|---|---|---|---:|
| css-gaps | 1067P/3081F | 1265P/2883F | **+198** |
| css-multicol | 631P/982F | 692P/921F | **+61** |
| css-typed-om | 9873P/2486F | 9873P/2486F | 0 |
| css-shapes | 4722P/2792F | 4830P/2684F | **+108** |
| css-masking | 1818P/4522F | 1925P/4415F | **+107** |
| **合计** | — | — | **净 +474** |

ab-compare 逐子测：**gained=474 / lost=0 / regressed=0，pages_gain=19 /
pages_loss=0**（页集合全等、declared 分母零差）。19 增益页逐页：gaps 7
（row-rule-color +38、row-rule-width +41、color-repeaters +37、
width-repeaters +43、width-multiple-values +16/+16、color-multiple-values +7）、
shapes 6（shape-outside 三页 +32/+21/+14、shape-margin +20、threshold +21）、
masking 6（clip-path-interpolation 两页 +30/+27、shape +24、
shape-control-points +16、xywh-rect +10）、multicol 2（column-rule-color
+30、column-rule-width +31）——增益面全部落 trans-all 可插值清单属性
（shape-*/clip-path）与 rule-\* 族，机理与两臂接线一一对应。

- 单页 census（rule-\* 插值页族 11 页＝9 增益+2 conversion 通道分解，b3c→c2）：
  **trans 234→42**
  （192 清偿＝trans-all 展开主增益）、waapi 77→71、cssa 231 持位（声明式
  CSS Animations gCS 咨询通道独立刀面未动，expected）、other 4 持位
  （conversion 页 harness 形）；页池总 546→348。inset 页池 102P/2157F
  （刀 D 域，本批未触）。
- head0 vs b3c：五 corpus 全 SET-IDENTICAL（gaps 1067/3081、multicol 631/982
  双腿逐数同）——editing ilb WSCANON 双修复对 CSS corpus 零位移实证，c2 对
  head0 差＝刀批净效应。

## 门禁

- engine lib **4,106 / 0F**（对 B3 基线 4,098 +8 新钉：all 通配中点/端点同值
  双断言、eased==1 rule-\* 广播、WAAPI 校验丢值/合法保形双帧、computed 保形/
  取整四钉；`test-guard --compile-first` + systemd 24G scope 包裹）。
- fmt：三改动文件 rustfmt --edition 2024 净（首轮 bare rustfmt 2021 edition
  误报 let-chains，带 --edition 复检定谳）。
- clippy：zero-engine / zero-style-system / zero-css-parser **零 warning**；
  唯一 warning = script-sandbox es_module peer WIP too-many-arguments（在册
  连坐，`71b07c7d8` 同款 exempt）。
- bench-gate 定向（`ZERO_WEB_BENCH_CRATES=zero-engine`，systemd 24G scope）：
  microbench **1/1 PASS**（engine_bench）+ perf-gate **rc=0**（绝对预算过门；
  基线硬件错配 WARN 在册，9700X vs 基线 13500H 既有形）。
- wpt-data 零写入；仓根零残留（git status 仅批内三文件 + peer es_module
  WIP + 本 evidence/master docs）。

## 残面队列（本批后）

- CSSA 声明式通道 ~231（增益页内）——声明式动画 gCS 咨询通道（镜像
  js_anim_registry 架构），B3 段已立独立刀面。
- WAAPI 余 ~71 + trans 余 ~42（增益页内，repeaters 页 trans 28+12 主体）。
- rule-shorthand.html 240F 池、from-longhands 4 对齐形（inline 站）持位。
- **刀 D：inset-cap/junction 几何族 22 页 102P/2157F**（下刀主目标；WIP 字段
  scaffold 已存 `.acceptance/wptfyi-css/wip-knife-d-inset-fields.patch`，完整刀
  需 css-parser parse_rule_inset + style-system registry/apply + engine gCS/paint
  五站全链）。
- 残面⑤：calc 代数插值全闭环（承重拒收边界重撤硬前置）。
