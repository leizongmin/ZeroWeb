# HTML 文档面兼容 — 运行时控制面板（master.md）

**入口文档**: [../html-syntax-compat.md](../html-syntax-compat.md)
**创建日期**: 2026-09-12（goal 立项） | **最后更新**: 2026-10-03（M3 片 a 续二落地）

## 当前状态

**M1 + M2 四片已落地（2026-10-02）**：基线 71.6% → 首簇 charref 直通修齐 75.23%
（[m2-ncr-charref](evidence/2026-10-02-m2-ncr-charref.md)）→ 片 a foreign ns 保真
75.31%（[m2b-foreign-ns-fidelity](evidence/2026-10-02-m2b-foreign-ns-fidelity.md)）
→ 片 b CDATA + NUL 读回 75.33% → 片 c P5 document.characterSet 嗅探 75.34%
（quotes/meta 全绿；案预算工具 + make test 栈加固 32MiB——管线深嵌套测试栈边际
随 zero-dom 重编译抽签，渲染流域碰头项已记账飞书）→ **片 d 生命周期时序
46190/61304 = 75.35%**（the-end 4/4 + DCL-defer 1/1；runner 尾单 timer 任务严格
序 DCL→load→pageshow + shim `_zwTargetOverride` 四写点门控 + Event/子类
toStringTag）。片 b 根因
双层：html5ever 0.29.1 fragment 模式 CDATA 门不 consult context_elem（宿主预变换
+ 本地视图 ns 通道双层修齐，FIXME 记 0.39 升级可移除）+ `_zwMEl` 无 title 反射
（补 HTMLElement 全局 title/lang/dir 三元组；探针实证 html5ever 的 NUL charref
语义本就正确）。证据：
[evidence/2026-10-02-m2c-cdata-nul.md](evidence/2026-10-02-m2c-cdata-nul.md)、
[evidence/2026-10-02-m2d-p5-cherset.md](evidence/2026-10-02-m2d-p5-cherset.md)、
[evidence/2026-10-02-m2e-lifecycle.md](evidence/2026-10-02-m2e-lifecycle.md)。

**M2 已收口（2026-10-02 判定）**：DC-2 通道可执行面全绿 + DC-4 门禁实测齐
（make test 68 suites + clippy/fmt + reftest **700/700 零不一致**）。残差全分类：
P5 通道外 19 案（reftest/iframe/HTTP 头形态）+ html5lib 3 案（300s 复评 Timeout，
document.write 测试生成管线面）+ serializer 面（DC-3/M3）。判定文档：
[evidence/2026-10-02-m2-verdict.md](evidence/2026-10-02-m2-verdict.md)。挂账：
bench-gate 复评（三轮窗口被兄弟流 clone 活跃测试污染，等让随下轮静窗）；
html5ever 0.29→0.39 升级候选架构片。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | html/syntax + html/dom corpus 导入 + 基线 | ✅ M1（2026-10-02，113 案 71.6%） |
| P2 | 解析树一致性（innerHTML/outerHTML/DOMParser）逐簇修齐 | ✅ **M2 收口（2026-10-02）**：五片落地（charref/foreign ns/CDATA/characterSet/生命周期），通道可执行面全绿 75.35%，DC-4 门禁实测齐（reftest 700/700）；残差分类记账（P5 通道外 19 案 + html5lib 管线面 3 案） |
| P3 | 序列化边缘（XMLSerializer/HTML serializer） | ⏳ M3（escaping 0/9 + XML 面 0.9%） |
| P4 | html/dom 接口语义 + createContextualFragment 补面 | ⏳ M3（reflection 74.7% 边缘簇 + elements 12.1% + render-blocking IDL 面） |
| P5 | 文档级编码嗅探（encoding-compat M4 转入） | 🔄 testharness 通道可执行面首片 ✅（sniff→Document label→characterSet 管线，2026-10-02）；📊 通道外案面仍记账：charset/ 7 案（reftest 形态）+ xmldecl/ 3 案（iframe 形态）+ the-input-byte-stream 9 案（HTTP 头形态）= 19 案 testharness 通道外记账；encoding/ 域 bom-handling/eof-*/sniffing 案面（已在 wpt-data，encoding 通道按 document.characterSet 规则 skip）尚未基线，落地通道（reftest import / 通道扩展）随 M2+ 定 |
| — | render-blocking 机制面（62 案 19.3%） | 📊 M1 已基线；`blocking=render` 与渲染管线耦合，是否本 goal 修齐随 M3 碰头定 |

## 已完成切片

- **M3 片 a 续二（2026-10-03）**：detached 旗标通道贯通——child_nodes_json_full
  （arg[3]）+ `_zwParseEl._ensureMutTree` 桥 inert 印章 + `_zwMBuildNode` 递归旗标
  （插桩实证漏传点：body 顶层已传、元素递归未传）+ `_zwMEscapeText` 补 nbsp；
  escaping 1/9→3/9（div.innerHTML + DOMParser + created-innerHTML，转义分支含
  nbsp 全对）；rebase 撞兄弟流 slice19 merge（import 冲突取并集）后组合树复验
  3/3 绿。
- **M2 收口（2026-10-02）**：DC-2 判定成立（可执行面全绿 75.35% + 残差全分类）；
  DC-4 实测（reftest 700/700 零不一致补齐最后一块）；html5lib 300s 复评定性
  document.write 管线面；bench 复评挂账（兄弟流活跃污染）。
- **M2 片 d（2026-10-02）**：生命周期时序——runner 尾单 timer 任务严格序
  DCL→load→pageshow（DCL 异步入队 + bubbles；load/pageshow target=document 经
  `_zwTargetOverride` 四写点门控；pageshow whatwg#6794 语义 + PageTransitionEvent
  原型）+ Event/子类 toStringTag；the-end 4/4、DCL-defer 1/1，全通道 75.35%；
  bench 两窗污染挂账（兄弟流 clone 长测）。
- **M2 片 c（2026-10-02）**：P5 首片——`<meta charset>` 预扫描（spec 引号/失败/
  续扫语义 + 序列化快照实体解码，zero-dom 增 encoding_rs）→ Document label →
  原生 getter + `__zw_get_character_set` 回调 → shim 主文档面；quotes/meta 全绿，
  全通道 75.34%；ZW_CORPUS_CASE_TIMEOUT_SECS 工具 + make test 栈加固（碰头记账）。
- **M2 片 b（2026-10-02）**：foreign context CDATA 双层修齐（zero-dom
  `translate_cdata_sections` + `child_nodes_json_ctx` ns 通道）+ `_zwMEl`
  title/lang/dir 反射；CDATA 10/10、zero 14/14，全通道 75.33%。
- **M2 片 a（2026-10-02）**：sel 代理四身份 getter foreign ns 保真（part03 `_zwSelNs`
  host NS_MEMO + gen 印章 memo + part04 四 getter foreign 臂）——foreign gET 4 案
  全绿 + math-parse +4，全通道 75.31%；goals/40 补拉 parsing/resources。
- **M2 首簇（2026-10-02）**：character reference 直通双向修齐——engine 三处 setter
  快速路径删除 + shim 纯文本本地视图同语义；named-character-references 0/2231→全绿
  + zero.html +7，零回归；全通道 75.23% 回填 wpt-suites.csv。
- **M1（2026-10-02）**：goals/40 fetch 脚本（DIRS + depth-2 support 补拉）+ runner
  通道（testharness-html-syntax 子命令 + html_syntax_case_skipped）+ Makefile
  fetch-wpt-html-syntax / testharness-html-syntax（test-guard）+ 基线 113 案 71.6%
  + wpt-suites.csv 数据行回填。

## 下一步计划

1. **M3 片 a（进行中）**：noscript + scripting 旗标——zero-dom 核心已落
   （Document.scripting_enabled + html5ever TreeBuilderOpts + 序列化条件 raw 规则 +
   engine 调用点分流；单测 2 绿 + serializing 邻面零回归）；JS 序列化器
   `_zwMSerialize` noscript 条件 literal 已落 + detached 旗标通道贯通（续二）。
   **escaping 3/9，剩余 6 失败**：①template content 序列化反查 1 面（content 归
   template → disabled 旗标须进宿主序列化器——template_contents 反查或解析期标
   记）；②缺失 API 2 面（Range.createContextualFragment = DC-3 明确项；detached
   doc.write）；③主文档视图更新 2 面（insertAdjacentHTML afterbegin/
   document.write 后 firstChild null——handle 容器本地视图更新）；④XHR data:
   URL 1 面。
2. **挂账随行**：bench-gate 等让复评（静窗）；html5ever 0.39 升级评估；P5 通道外
   19 案落地通道（reftest import 优先——charset/ 形态契合）。

**待用户决策清单**：（空）
