---
date: 2026-11-16
modules: engine,style-system
---

# CSS.supports 与 setProperty 双通道文法分叉

## 问题描述

offset-path interpolation 六个全灭页（005/006/008/009/coord-box/composition，
合计 1,890F）曾归因于「parse 未放宽」。逐页抓取失败消息后定谳：全部死于
WPT harness 前置门 `assert_true('from'/'to' value should be supported)`
（interpolation-testcommon.js:396），即 `CSS.supports(prop, v)` 返回 false；
而同一批值在 `e.style.setProperty` 通道大多已接受——`offset-path-parsing-valid`
页 46/70 过，circle()/inset()/ellipse()/polygon()/xywh()/rect()/url() 标准形
全 P。同一值两条通道判定相反，整组 6 检查点被 supports 门连坐全灭。

## 根因分析

ZW 里「一个 CSS 值是否被接受」有两条互不知情的通道：

1. **setProperty 通道**：shim JS 验证器（part05/part06 侧）先判文法，宽收
   后存声明，只有布局/gCS 真正消费 parse 结果。它对基本形状标准形已放行。
2. **CSS.supports 通道**：shim part06 `_zwCssNamespace.supports` 两参形式
   委托宿主 `decl_supported`（engine `js_dom_bridge.rs`）→
   `known_properties()` 白名单 + `apply_property_value_with_quirks` 真跑
   Rust parse。Rust 侧 `parse_offset_path_fn` 当时只收 none/path()，
   ray()/基本形状全拒 → supports=false。

WPT interpolation 页每子测先过 supports 门，门不翻，后面的插值断言根本
不执行。因此「parse 已放宽」和「页面解锁」之间隔着一整条 supports 通道。
反之，只修 supports 而不修 parse 也会在 gCS/apply 站二次死。

## 解决方案

- 新属性刀的刀界探查必须**两通道分别探**：对同一批失败值同时跑
  `setProperty` 通路（parsing-valid 页形态）与 `CSS.supports` 通路
  （interpolation 页消息形态），先定位分叉点再动手，避免把 supports 门
  问题误诊成 parse 缺面（本轮 1,890F 的初始归因即为误诊）。
- supports 两参语义是「ZW 能 apply」（known-property gate + apply oracle），
  与 setProperty 的 shim 宽收天然可能分叉；对齐手段是放宽 Rust parse 臂
  （刀核心），而不是给 supports 加 JS 前置钩子（color/opacity 有过先例，
  属特例豁免，不宜扩权）。
- 探查产物直接定位了第三刀刀界：`parse_offset_path_fn` 全基本形状族放宽
  + gCS verbatim 臂即可翻门，WAAPI 离散翻转臂已通用无需新增；理论增益
  ~400-500P，精确账以刀后 A/B 为准。
