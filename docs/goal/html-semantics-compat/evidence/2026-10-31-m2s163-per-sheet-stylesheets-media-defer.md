# M2-S163 per-sheet styleSheets 通道 + 外链 media 推迟取回 + CSSOM 尾随分号

日期：2026-10-31。基线：s161 终案对账（`/tmp/zw-s161-full4.log` 7090P/803F/91T 子测级）。切片号承并发注记顺延（原 WIP 标 s162 与已落账 s162 style-load 撞号，落账时按控制面指示重编号 s163，注释同步）。

## 切片目标

s161 候选①设计面：conditionally-block-rendering-on-link-media-attr 2F 收口——同步断言段（gCS purple + styleExists purple/brown）与 3000ms 注册段（styleSheets 双 sheet）四断言全绿；per-sheet 结构化通道替代 R5114 单条合并 blob。

## 施工面（6 文件：rust 3 + shim 3）

1. **css_wire.rs**：`css_declarations_to_string` 每条声明追加尾随 `;`、join 由 `"; "` 改 `" "`（CSSOM serialize a declaration block——real UA `rule.cssText` 实测 `h1 { color: purple; }` 形）。上游 support/utils.js `styleExists` 逐字节比对（`styleText == rule.cssText`）钉该形态，旧无分号形恒 false。下游 `_parseDeclarations` 按 `;` 切分尾空段无 `:` 跳过，读取面零波及（全量 join 仲裁 P2F=0）。
2. **runner 端点**（testharness.rs）：`interactions-of-styling-and-scripting/stylesheet.py` 参数化端点（上游 pin master 逐字核对）——`stylesNotMatchingEnvironment` → `h1 {color: brown;}`，否则 `h1 {color: purple;}`。抽出 `serve_interactions_stylesheet` 双站点共用：fetch 桥闭包（shim fetch 通道）+ `wpt_data_image_fetcher`（引擎 base 槽外链 CSS 通道——该通道不过 fetch 桥，文件映射把 .py 源码当 CSS 读，purple 表恒缺级联的黑屏根因）。**`delay` 参数不 sleep**——同步契约下 host sleep 冻结 JS 求值（同 execute 内结算，取回时点对页面不后移），延迟语义由 shim `_zwFetchSimDelay` 发题侧承载（net-api M4-S20 delay.py 同款先例）。
3. **part03 media 推迟取回**：`_ssVisit` 调用点包裹 defer——media 属性非空非 `all`（ASCII 归一）经宿主 `__zw_match_media`（1280×800，s160/s161 静态门同款 FIXME 承袭）求值不匹配 → `setTimeout(_ssDoVisit, 0)`（mixed 门同款异步形态）。spec：media 不匹配 stylesheet 照常加载注册但不阻塞其后 parsing script 与 load。同步契约下匹配 link 先于脚本体 fire（s152 交错触发语义保持），不匹配 link 推迟到后续 pump。
4. **part03 per-sheet 枚举口**：`_zwSsExtSheetEntries`——live 文档序 query_all('link') + `_zwLinkRelTokens` stylesheet 门 + href 非空门 + 同款身份键派生（`__zw_handle_for_selector` 命中 R100 身份表才 '@handle' 形；静态 parse 元素不在表 → plain sel，与静态扫描 `_elKey(sel,null)` 键同形无错位）。条目门：表有键且文本非空（'' = 取回 error = sheet 未注册）；取回原文经新宿主回调 `__zw_style_rules_text`（callbacks.rs，`style_rules_text` 直通）转 `\x1f`/`\x1e` wire（`_makeStyleSheet` 消费形；R5114 blob 通道在引擎侧已转换，per-sheet 入口缺等价桥）。R51c 查询视图对动态 link 不可见 → 表内残留键尾追（FIXME：插入序近似文档序）。
5. **part06 styleSheets getter**：枚举有条目 → 逐条 `_makeStyleSheet(null, text, href)` per-sheet 条目（cssom document-stylesheets 每 link 元素一个 sheet；media/disabled 不过滤注册，同 style 元素条目口径）；零条目 → R5114 blob fallback 原样（无 link 页零行为差）。**CSSImportRule 接口全局**：上游 styleExistsInSheet 对每条规则 `rule instanceof CSSImportRule`，全局缺位使首个规则比对即 ReferenceError（wire 通道 @-rule defer 无 import 规则产生，instanceof 恒 false 递归分支死路）；空壳构造器满足 interface reference。FIXME：CSSOM 规则接口族整体建模未做。
6. **part01 `_zwFetchSimDelay`**：识别本端点 `delay`（秒）参数 → 发题侧 setTimeout 延迟（30000ms cap，delay.py 同款）；`__zwScanCollapseTrickle` 扫描窗内折叠为 0（s152 trickle 同款——script-blocking stylesheet 的「load 先于后继脚本体」语义优先）。

## 验证

- `cargo check`（engine/wpt-runner）✓、`cargo fmt` ✓、`make guarded-clippy` workspace ✓、shim 14-part 拼接 `node --check` ✓。
- 归因轮三轮（各有独立证据）：
  1. 首轮定向 2F 未动（gCS black）→ 勘根揭双层根因：引擎 base 槽走 image_source_fetcher 文件映射不过 fetch 桥（.py 源码当 CSS）+ per-sheet 表存原文非 wire。
  2. 修复后定向 **2F → 2F 变形**（`CSSImportRule is not defined` + 两个 script-created 案 P→F）→ 前者 = CSSImportRule 全局缺位（首轮被 assert_equals 先行失败遮蔽）；后者 = host sleep 在同 execute 内结算使 delay 对页面不可见 + 尾追枚举暴露提前入表的 sheet（旧 blob 形被覆盖门 mask 的真位移）。
  3. 终修（端点去 sleep + `_zwFetchSimDelay` 承载 + CSSImportRule 全局）→ 定向 **8P/1F**：conditionally-block 双子测 F→P，style-element-media-match-block-script 保持 F（@import 通道，记账）。
- document-metadata 目录 join 对 s161 full4：**NEW-BAD 零**、净 −5（靶面 2 + 前段 s162 style-load 面 3）。
- 全量案级 join（full1 对 s161 full4，(key,state) 对 sort -u）：**P2F 0 / P2T 0 / T2P 0 / T2F 0**；实质 F2P = 3（conditionally-block ×2 本刀靶面 + style_load_event textContent ×1 前段已落账 s162 在本树复现）；其余 18 条「F2P」为文件级 Timeout 抖动 join 伪影（select-event/dialog 族 5 文件 8-job 并发截断，solo 复跑 270P+9F / 24P+16F / 27P / 120P / 120P 全部对齐 s161 基线）。终账 **7009P/783F/91T**（子测级，LOST 108 全归因并发截断 + s161 探针文件已删 + 靶面翻绿的文件级 Timeout 键消，GAINED 7 同源镜像）。
- css-module 面（cssText 尾随分号潜在波及）：位于 html/semantics/scripting-1 树内，全量 join P2F=0 仲裁零位移。

## 范围与门禁

rust 3 文件（css_wire/callbacks/testharness）+ shim 3 文件（part01/part03/part06）。reftest/product-smoke 未跑——本刀无渲染路径变更（cssText 为 readback-only、级联应用不经 wire；shim 时序变化限 testharness 面），沿 s161 shim/style 切片先例记账。

## 记账面（本切片不解决）

1. **style-element-media-match-block-script 1F**：style 元素 @import 通道——需 engine style @import 级联 + CSSOM CSSImportRule.styleSheet 子表 + style 元素扫描，下一候选勘根面。
2. **stylesheet-media-change-no-fetch 断言 2**：动态 link 对 query_all 不可见（R51c）——尾追枚举按表插入序近似，断言仍 F；结构化修法需 R51c 查询视图扩展。
3. **CSSOM 规则接口族**：CSSRule/CSSStyleRule/CSSMediaRule 等常量与 instanceof 面未建模（本轮只补 CSSImportRule 空壳）。
4. **link type essence 门**：枚举与 fetch 通道同缺（语料无形态）。
5. **media 求值视口 1280×800 FIXME** 承袭（s160/s161 同债）。
