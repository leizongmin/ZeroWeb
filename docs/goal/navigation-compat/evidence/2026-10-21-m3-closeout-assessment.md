# M3 收口评估（2026-10-21）

**依据**: S7 后全量语料 [2026-10-21-m3-s7-nested-iframe-contentdocument-full.txt](2026-10-21-m3-s7-nested-iframe-contentdocument-full.txt)
（584 案行 473P/68F/43T；对 S6 逐路径 diff 恰 1 变化、583 恒等）；门禁 make test
76 suites 22,327P/0F、clippy -D warnings 零 warning、fmt 干净连续全绿；
webmessaging 跨套件回归臂 135 行逐行恒等；reftest 收口复验 712/712 零不一致
（见 DC-4）。

## M3 完成面盘点（S1~S7 七切片）

| 切片 | 内容 | 域内累计 | 逐路径回归 |
|---|---|---|---|
| S1 | srcdoc 处理模型 + sandbox 指令（ASCII 解析 + allow-scripts 门）+ servedDoc 成熟度双读面 + frames named access + 子文档 location 写侧 + load 信任面 | 16P→25P | 零 |
| S2 | doc.location↔win.location 同一对象 + 全 URL 解析面 + 子窗 hash 写侧 + 合成树 innerText | 25P→27P | 零 |
| S3 | 联合 iframe session history（惰性播种 + 遍历路由 + 顶层账本不膨胀） | 27P→28P | 零 |
| S4 | replace-before-load 最小面（{{GET}} 模板 + 子脚本 history 形参 + frame 局部回退） | 全量 460P→464P | 簇外零变化 |
| S5 | parent→child postMessage + 子文档 onmessage 绑定面（with(window) 包合） | 28P→32P | webmessaging 臂恒等 |
| S6 | 子窗局部 history 遍历（`_zwDocBase` 文档代模型）+ onpopstate 面 + parser 期补启 + runInlineScripts allow-scripts 门 | 32P→36P | 576 恒等 |
| S7 | 子文档嵌套 iframe 自主载入（finish 递归 + 跨层 src 解析 + 深度界）+ lazy 仅 script-enabled 适用 + 子文档 contentDocument/contentWindow 面 | 36P→37P | 583 恒等 |

七切片累计：全量 437/476（M2 终态）→ **473P**（584 案行）；域内 the-iframe-element
16P→**37P**；每轮全量语料 per-path 精确 diff **零 Pass 回归**。

## DC-3 判定：✅ 满足（最小面口径）

门控批复口径 =「批准 M3 最小面」（2026-10-21 用户当面批复；未豁免 DC-3，
「DC-3 随 M3 实施结果判定」）。DC-3 判据原文：**嵌套 browsing context 最小面
（contentWindow/contentDocument/frames/parent/top）+ iframe 元素属性语义**。
逐项对照：

- **contentWindow**：R115 主文档面 + R139 named access + S1 子窗 stub + S5
  WindowProxy 投递面（block_modals 四案断言链）。
- **contentDocument**：R115 主文档面（跨源 null 门 + servedDoc 成熟度双读）+
  S7 子文档合成树面（`_zwMEl` 惰性 getter + 嵌套 entry 印）——lazy-in-script-
  disabled 三层断言链翻绿实证。
- **frames**：R177 窗口 frames proxy（numeric + named）+ R139 `__zwRegisterNamedIframes`。
- **parent/top**：S3 联合 session history 遍历路由 + S5 parent.postMessage 回程 +
  S6 子窗 history 越界 no-op（祖先遍历阻断自然涌现）。
- **iframe 元素属性语义**：src/srcdoc 处理模型（S1 + S7 递归）、sandbox 指令
  （ASCII-only 解析 + allow-scripts 双通道门——fetch 通道与 defer 通道，S6 连带
  收口 sandbox-ascii-case-insensitive）、loading=lazy（S6 script-enabled 链惰性 +
  S7 script-disabled 不豁免）、name（R139 named access）。

判据面全部落地且有 corpus 翻绿 + part63 集成测试背书（m3s2/m3s3/m3s5/m3s6/m3s7
案族）。**判定 = DC-3 按获批的最小面口径满足**——超出口径的深语义（见下残余
B/C 类）不属「最小面」判据范围，转后续记账。

## 残余定性（111 Fail/Timeout 全量盘点，584 案行）

**A. 域回流（不属本 goal envelope）**
- navigation-api scroll-behavior 4F + scroll-to-fragid 2F——滚动几何/渲染域
  （rendering-compat，既有回流账维持）。
- history-interface 1T（008 document.write 引擎全缺——parser 域，S10 定性维持）。
- pagereveal 1F + navigate-event 余 7F3T——Navigation API 深簇（task 排队模型
  经评估挂账、order-in-new-document 等），属 M2 P3b 已定性余项，非 M3 面。

**B. runner 形态 / M3 后续深语义（超「最小面」口径，记账池）**
- replace-before-load 余 35（18F+17T）：子窗 search setter、子 form 提交 frame
  定向、joint history.length、pageshow 簇——S4 最小面已立，深缺口按 FIXME 记账。
- iframe-loading-lazy 族 27（ predominantly T）：lazy↔eager 动态翻转 + nav 变体
  （location.assign/replace、link click、form submit）——需视口近似模型 +
  跨面导航触发，S7 已立 script-enabled 基础门。
- the-iframe-element 余 26（17F+9T）：meta-refresh 形态、src-encoding、
  javascript: URL 族、{{hosts}} 模板面、move_iframe_in_dom 重挂 T 族。
- 子发起嵌套导航控制余案 4T（ancestor/descendants/itself/other_frame_popup，
  runner skip 记账池）：sandbox 祖先导航守卫 + 父窗 popstate 监听面。
- popup 窗口管理（iframe_sandbox_popups_*）：依赖用户手势基建。

**C. 零散**：navigation-activation 2F（transient activation 时序面，user-activation
深域回流）。

## DC 全项复核

| DC | 判定 | 依据 |
|---|---|---|
| DC-1 WPT 导入与基线 | ✅ 满足（M4 评估维持） | 四 corpus 导入 + 基线 20.9% 落 evidence；wpt-suites.csv 逐切片回填至 473/584 |
| DC-2 会话历史与导航事件 | ✅ 满足（M2 收口维持） | location-interface 100% / traversal 93.3% / history-interface 98.2% / navigation-api 95.0%（S12 后全量 NotRun 清零）；M2 终态 437/476 |
| DC-3 iframe 浏览上下文（用户门控） | ✅ **满足（最小面口径）** | 门控获批 + S1~S7 完成面逐项对照判据（见上）；残余 B/C 类超口径记账 |
| DC-4 测试与质量不可退让 | ✅ 满足 | 每轮 make test 全绿（本轮 76 suites 22,327P/0F）+ clippy -D warnings + fmt 零 diff；每切片带集成测试（part63 m3s* 案族）+ per-path 零回归 diff；reftest 收口复验 **712/712 零不一致**（508 Layout + 204 Text——七切片零布局/绘制触达先例的全量兜底） |

## 结论

M1/M2/M3 判定完成，**DC-1/2/3/4 全部满足**（DC-3 按获批最小面口径）；bfcache
（挂账至 M3 后续立项——M3 最小面已成，frame tree 基础在位，Document 快照管线
未立项维持挂账）/ fission（charter 排除维持）定稿不变。残余 B/C 类（replace-
before-load 深缺口、lazy 动态翻转族、嵌套导航控制余案、popup 管理）不属本次
获批口径，转记账池由后续立项裁决。本 goal 具备 DONE 判定条件。
