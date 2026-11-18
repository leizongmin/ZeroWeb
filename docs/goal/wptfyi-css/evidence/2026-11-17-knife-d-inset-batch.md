# 刀 D rule-inset 全链批（2026-11-17）：css-parser + style-system + engine + inline 站

批内账（checkpoint A/B native 面 + v2 终态 A/B inline 站全量）：

## 交付面

1. **css-parser `parse_rule_inset`**（新文件 `crates/css-parser/src/values/parse_rule_inset.rs`，
   317 行）：css-gaps-1 §inset 槽语法单点 `parse_rule_inset_component`
   （`overlap-join | <length-percentage>`，负长/`%`/`vmin`/`calc` 透传）+
   四槽/轴对/全值简写顶层解析（`/` 双对形、单轴对形）。
2. **style-system 8 longhand / 15 shorthand**：registry 8 longhand 臂 +
   apply 15 简写展开臂（expand_rule_inset 投影落槽 canonical 文本，rule-color
   臂同模板）+ shorthand/mod.rs 15 名展开映射 + validate_decl_value 白名单
   15 简写 + 8 longhand（apply oracle 兜底裁决）。
3. **engine 读面**：gCS `rule-inset` 全值简写跨轴等值模型（列/行各四槽重组后
   双轴全等 → 该值，否则空串——bidirectional-shorthands 页 target1/target2
   oracle；修复 checkpoint 轮直通 column quad 的 REGRESSED=1）+ transition/
   WAAPI 通道（animation.rs `rule_inset_component_to_anim_css`）。
4. **inline 站（kd-inline-station T50/M/PINS）**：corpus 语料 `el.style[prop]=v`
   全落 shim 字符串面——checkpoint native 绑定展开代码 runner 从未执行（A 侧
   inset 页 0 PASS `expected "0px" but got "0"` raw echo 形定位根因）。三回调
   （`__zw_style_shorthand_expand`/`__zw_style_shorthand_reconstruct`）宿主
   注册，part03.js T50 setProp/readProp/removeProperty + M face writeDecl/
   readDeclValue 五处接线：写面展开落名单 longhand（简写键不占声明位）、读面
   raw 重组（缺任一 longhand → 空串 Chrome 口径）、remove 家族一并清、非家族
   名/展开失败回落通用路径零漂移。position-try 拆分（组 F）inline 站同型先例。

## 两轮 A/B 全账（ab-compare B0=results-transall-knife-d-base HEAD=c775846d4 纯净码态）

| 轮 | 二进制 | gained | lost | regressed | pages_gain | pages_loss |
|---|---|---|---|---|---|---|
| checkpoint（native 面） | 13c835e3 | 841 | 0 | 1 | 12 | 0 |
| v2 终态（inline 站 + 8 臂修复） | 17960cd4 | **1545** | **0** | **0** | **28** | **0** |

v2 终态五 corpus：gaps gained=1474（24 parsing 页 + 2 animation 页 2×240）、
multicol +14（column-rule-shorthand 0→12、column-rule-valid 3→5）、
typedom/shapes/masking SET-IDENTICAL。COMPARE-RC=0（PAGE-ONLY 零容忍满足）。
rule-inset-valid P 30→51（+21）。增益面 = parsing roundtrip 页 11 页 845 +
bidirectional 家族 6 页 231 + computed 页 4 页 157 + animation 插值 2×240 +
invalid 页 4 页 144 + multicol 14。

## v2 轮两 bug 定谳链（V8 探针 direct 回调三分法）

1. **8 longhand 读重组缺臂**：`gap_rule_reconstruct` match vals.len() 只有
   4/2 臂——`rule-inset` 名单 8 longhand 落 `_ => None` 恒空串，全值形 10 形
   REG（B0 未知属性 raw roundtrip 恒过 → B1 已知简写读空）。探针三分：expand
   8-pair JSON ✓ / reconstruct col `10px 20px / 10px 20px` ✓ / reconstruct
   8 → '' ✗ 定谳 shim 接线无虞、单点缺臂。修复 = 8 臂双轴四槽 quad 重组
   （gCS 跨轴等值模型同口径）。钉 part129_kd_rule_inset_inline_pins（9 pin：
   塌缩/全形/轴对/负长/overlap-join 逐槽拆分/跨轴失配/cap 对/家族 remove×2）。
2. **checkpoint 轮 regressed=1**：gCS `rule-inset` 臂直通 column quad →
   target2（col≠row）非空串 → 跨轴等值模型重写 + computed_style.rs 钉
   `rule_inset_full_shorthand_cross_axis_equality`（target1/target2/全 8 等值）。

## 质量门禁

- bench-gate：定向 `ZERO_WEB_BENCH_CRATES=zero-engine`（首跑漏 --setenv 跑成
  全 16 crate 杀掉重发 zw-kd-bench-inset2，phase 行只现 engine_bench 确证），
  microbench 1/1 PASS，perf-gate rc=0（硬件错配 WARN 在册 13500H vs 9700X）。
- engine lib `cargo test`：**4116 passed / 0F**（checkpoint 轮 4113 + 跨轴钉 +
  expand JSON 钉 + reconstruct 钉 + part129 9 pin 合并计）；rustfmt FMT-CLEAN；
  clippy zero-engine 零自产 warning（animation.rs duplicated `#[test]` 修复、
  gap_rule_* 三 fn 回落私有——StyleDecl 私有签名泄漏消解；script-sandbox
  es_module too_many_arguments = peer 在册连坐 exempt）。
- 二进制 provenance：B=46684957（strings rule-inset=0）；checkpoint
  A=13c835e3（rule-inset=15）；v2 A=17960cd4（rule-inset=14、
  `__zw_style_shorthand_expand`=9、`kd-inline-station`=2）。构建 = stash
  窗口（peer es_module 独立 stash、pop 逆序回植，script-sandbox 从 HEAD 编译）。

## 运行事故（在册）

1. **systemd unit cwd 毒化 fail-all 形**（chain2）：发射 shell 在 repo 根但
   unit 默认 cwd=$HOME → 五 corpus 全案秒败 pass=0 fail=788、五腿 41s、merged
   JSON 照常落盘——非 `[]` 空真绿而是 fail-all 满盘毒。同二进制单页探针
   repo cwd 34 Pass / $HOME cwd 页键变 `resources/testharness.js` = cwd 毒化
   指纹。毒 JSON rm 重跑；修复落 launch/probe 脚本内 `cd "${ROOT}"`
   （self-contained）；memory runner-page-path-wpt-data-relative 已补 chain 级
   fail-all 形与修法。首发 unit 路径转义笔误（`'\.`）秒死同轮杀掉重发。
2. **幻影 gained**：chain2 遗留毒 JSON 在 v2 腿未落终态时比对 → 假 gained；
   双侧 leg 终态后重比为准。

## census 探针（定向池）

`probe-knife-d-inset-census.sh`（CSS_CORPUS_ONLY_SUBSTR=rule-inset，22 页
= 20 parsing + 2 animation 插值）：**1377 pass / 882 fail / 0 timeout**
（21s，二进制 17960cd4）。fail 余量 = 化简 canonical 差异与 computed/序列化
长尾，为后续刀位（残面⑤ calc 代数插值 / WAAPI 余 71 / trans 余 42）输入。

## 总目标进度

本批 v2 终态 **净 +1545**（ab-compare B0=c775846d4 基线二进制五 corpus）。11-14
发布快照 62,896 / 197,163（31.90%）后各批（刀 A +148、trans 通道 +322、texp
+489、B3 +395、刀 C/trans-all +474、本批 +1545 等）批账累积，累计数在下次
DC-3 快照刷新时统一对账。刀 D 后队列：WAAPI 余 71 + trans 余 42；残面⑤
calc 代数插值；CSSA 声明式动画 gCS 咨询通道。
