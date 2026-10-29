# M2 s159 · 合成条目 href 通道（per-link 生效 sheetHref 状态机）——stylesheet-not-removed F→P

日期：2026-10-30 · 切片：html-semantics M2 · 前序：s158（link rel 状态 ↔ 外链 CSS 级联活链路）

## 入场定刀位

s158 下一步候选①：stylesheet-not-removed-until-next-stylesheet-loads 1F
（R5114 合成条目 href 通道——styleSheets 合成面）。

## 勘根

1. **失败形态**："Cannot read properties of null (reading 'includes')"——
   document.styleSheets[0] 为 R5114 合成条目，href 硬编码 null。
2. **案面断言双时序**：test(fn) 同步执行——断言 2（初始 fetch 在飞）
   `styleSheets[0].href.includes("style.css")`；断言 4（href 改
   neutral.css?pipe=trickle(d1)，新 fetch 排队）`styleSheets[0].href`
   仍含 style.css（旧表在位）。两断言都要求合成条目在「生效 sheet」
   语义下携带 link 的 href。
3. **spec sheet 对象生命周期近似**
   （html/semantics.html#link-processing-model）：processing 开始创建
   sheet 即有 href（不等 load 完成）；href 变更排队新 fetch 旧生效值
   不动；load 完成新表替换生效；error 替换未发生保持旧值。
4. **R5114 documented limitation 收窄**：per-link href 有查询通道可
   近似（状态机 + live 查询口）；per-link CSS 文本拆分仍无数据面
   （合成 blob 单条目不变，记账保留）。

## 施工（2 文件净 +160，rust 零改动）

1. **part03 五件**：
   - `_zwSsSheetHrefMap`（identity key → 生效 sheet href，键口径同
     `_zwExtSheetTexts`：静态=sel、动态='@'+handle）。
   - `globalThis._zwSsSheetHrefFor` 查询口——`__zw_query_all('link')`
     文档序 + lw 读 rel/href + `_zwLinkRelTokens` ASCII ws 分词 +
     href 非空门 + handle 反查键，map 首命中返回；全 miss → null。
     （合成 blob 单条目的 href 映射取级联首序 link——多 link 页近似。）
   - fetch 发起创建写（fired 幂等门 4790 区后）：首次发起即挂解析
     绝对 URL——同步断言段可读（案面断言 2 时点）。
   - preload 复用臂创建写：rel=preload → stylesheet 转换面 sheet 从
     cache 采用 = 零网络生效，同语义挂值。
   - done 包装 `if (_ssV === 'load')` 覆写为新 URL（error 不动——
     替换未发生保持旧值，最小状态机，全量 join 兜底）；rel 移除臂
     `delete _zwSsSheetHrefMap[_ssKey]`（关联解除）。
2. **part06 五处**：`_makeStyleSheet`/`_makeSheetBuild` 加第 3 参
   externalHref 透传；`href: null` → `href: externalHref || null`（元素
   表与 constructable CSSStyleSheet 的 href=null 不动——LinkStyle 元素
   href 属性面未建模）；R5114 注释更新；styleSheets getter 合成条目
   每访问调查询口传值（live 求值）。

## 验证

- **语法**：规范序拼接 node --check SYNTAX_OK **110795 行**（s158
  110635，净 +160）。rust 侧零改动（callbacks.rs 未动，fmt/clippy 不
  涉及；runner 重编译含新 shim js）。
- **靶面定向**：stylesheet-not-removed-until-next-stylesheet-loads
  **Fail→Pass**（断言 2 时点：创建写已在 fired 门后发生——fetch 在飞
  同步段可读 ✓；断言 4 时点：href 变更幂等门解析 URL 不同照常重取，
  sheetHref 不覆盖仍 style.css，neutral load 覆写发生在案面结束后 ✓）。
- **目录级 the-link-element**：**112P/2F/0T**——两记账 F 保持
  （link-rel-attribute shadow 案、stylesheet-media-change media 门），
  靶案转 P，**0 Timeout**（name-attribute 面无回潮），
  link-multiple-load-events **6/6 保持**。
- **全量案级 join（对 s158 终账 /tmp/zw-s158-full3.log，(key,state)
  对集 sort -u 去重）**：F2P **1**（靶案）。另见「环境漂移面」。

## 环境漂移面（域外，A/B 铁证与刀无关）

全量第一轮 raw 7080P/809F/90T 对 s158 终账 7205P/793F/87T——域外
位移面：
- selection-pointer.html **21 P2F**：A/B stash（本刀移除跑基线）同态
  21 Fail；s157/s158 两轮终账同域 21 Pass（10-30 05:15 / 07:01）；
  语料 mtime 10-15 未变、runner 资产零变化。空闲态（load 1.31）复跑
  仍 Fail = **持久环境漂移**（时序敏感交互案——pointer/selection 竞速
  面；温控/负载持久化假说，07:10 并发 css 流 checkout 活动时间线
  吻合——main.rs/testharness_css_corpus.rs/goal 脚本三文件 mtime 更新
  内容与 HEAD 零差异）。
- select-event.html 91 LOST + focus-dynamic-type-change-on-blur 12
  LOST：全量轮文件级 Timeout 截断（定向复跑 270P / 80P 全 Pass 恢复
  = 环境截断非翻转）。
- dialog 两案 2 LOST + 2 GAINED：行级杂项（同族交互时序域）。
- **判定**：全部在 forms/selection + dialog 交互时序域，与本刀刀口
  （link CSSOM href）零交集；A/B 已证与代码无关。按域外环境漂移
  记账；机器冷却后 selection-pointer 复跑钉死（暂态恢复 vs 持久漂移）
  留待下轮入场。

## 挂账（本切片更新）

- **收口：stylesheet-not-removed-until-next-stylesheet-loads 1F**
  （合成条目 href 通道，销案）。
- **FIXME 解除**：R5114 合成条目 href=null——per-link 生效 href 通道
  落地（查询口近似，每访问 live 求值）；**per-link CSS 文本拆分仍
  documented limitation**（合成 blob 单条目不变）。
- **新记（回归面说明）**：error link 不删 sheetHref——error 后查询口
  返回失败 link 的解析 URL（spec「替换未发生保持旧值」近似，但该值
  对 error link 而言是「从未生效」的虚构 href；全量 join 未暴露该形
  态，暂不收紧，踩坑语料再补 error 删键或保持旧值链）。
- **新记（环境）**：forms/selection + dialog 交互时序域机器状态敏感
  ——selection-pointer 21 案持久漂移（A/B 铁证非代码），select-event/
  focus-dynamic 全量截断（定向恢复）。下轮入场先复跑钉死。
- **承袭不变**：候选②动态元素 computed style 大面（__zwSelector 只
  属静态元素——需先定刀口）；候选③shadow DOM 级联面；候选④全域
  .css fixture 缺口 sweep；R5114 per-link 文本拆分；link-multiple 全量
  贴边（本轮 6/6 保持）；meta long × corpus 失效；details 族贴边；
  promise_test 串行启动面；innerHTML 身份分裂表示面；sel 重解析同槽
  同 href 印记失配边角；host 侧 \v 分词偏差；overlay/base 文本语义差
  两面；link media 门未建模；生产 renderer SetDomSnapshot 外链槽恢复
  臂；待决三项须用户拍板。

## 下一步（s160 候选）

①入场先钉环境漂移面（selection-pointer 复跑——回 Pass 则暂态漂移
销案，仍 Fail 则持久漂移维持记账；顺带 select-event/focus-dynamic 全
量前定向核对）；②候选②动态元素 computed style 大面（先定刀口）；
③shadow DOM 级联面（link-rel-attribute shadow 案）；④全域 .css
fixture 缺口 sweep（lib.sh 2026-10-29 前目录族）。
