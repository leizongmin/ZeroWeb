# M2 s150 · 动态 link rel=stylesheet 事件面——插入/href 变更 fetch + load/error 派发

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s149（runner 终态判定收口——
harness 页级超时早收形）

## 入场定刀位

s149 下一步候选①：早收形 91 文件 partial rows 单根翻拣——首个靶 =
document-metadata/the-link-element/（link-multiple-load-events 4F 的
stylesheet.py 端点仿真面勘根，s138 echo 端点先例）。勘根时发现单靠
stylesheet.py 端点仿真补不齐：runner headless 路径**动态 link 完全无
fetch、无 load/error 事件记账**——升级为 shim 事件面主刀（Face A），
端点仿真为配套（s138 先例同形）。

## 勘根（探针 p1–p4 + 浏览器路径对照定谳）

1. **runner headless 路径零 link 事件记账**：`resolve_external_css` 只做
   静态 CSS 收集，无 load/error 事件派发；动态 `<link rel=stylesheet
   href=…>` 插入后**零 fetch、零事件**——探针 p1–p4（动态插入/setAttribute
   href/innerHTML 注入/完整 load+error 监听）1s 窗口均「neither event」。
2. **浏览器 AsyncPageLoad 路径完整**（对照面）：页面加载期静态 link 有
   fetch 与样式应用——事件面缺失是 headless shim 通道的结构洞，非
   runner 特有。
3. **幂等语义**（HTML spec link processing + CSSOM view）：href 解析后
   绝对 URL 不变不重取；rel 值含 stylesheet 空白分词命中即算；detached
   link 不触发（连接门）；https 文档 http 子资源 mixed-content 阻断 →
   error（webappsec mixed content should-block-fetch）。
4. **shim 身份双表示约束**：innerHTML flatten 产物 = plain handle
   （`'__n1'` 形），宿主 tag/attr 表 miss → `_realTag` 回落恒 DIV、
   `__zw_get_attr_handle` miss——s111 既有 LINK/SCRIPT 甄别同样 miss，
   LINK 臂根本不进。s111 modulepreload helper（s124）为结构模板。

## 施工（shim 4 文件 + runner 端点，净 +232）

1. **part03 `_zwStylesheetMaybeStart` helper**（`_zwModulePreloadMaybeStart`
   结构模板）：TAG 门（plain handle 形 `_realTag` 回落 DIV 时放行，误触发
   面由后继 rel/href 门兜底）→ 连接门（`_zwMutationInDoc` 父链爬升）→
   attr 读通道（sel/handle 形走宿主桥；plain handle 形 elObj 第 4 参
   getAttribute 回落）→ rel 空白分词含 stylesheet → href 非空 → 幂等门
   （prevHref 第 3 参解析后绝对 URL 比较，null = 插入触发总取）→ mixed
   content 门（http: 子资源 + https: 页 → setTimeout error）→ fetch →
   `ok ? load : error` / catch error。事件经 `_dispatchWithBubble` +
   `_makeEvent`（bubbles:false，元素站）。
2. **part01 s111 两臂**（顶层 `_an109n` + 下降 `_walk111`）：LINK 甄别
   修正（plain handle 形 `_realTag` 回落 DIV 时按节点 tagName 判——仅
   LINK，SCRIPT 甄别行为不动）+ 插入触发（prevHref=null）。
3. **part04 setAttribute 钩子**：`n === 'href'` 且 LINK 时写前捕获旧值
   （`_zwSsPrev`，lw 读），写入后触发 helper（prevHref=旧值）——写前
   捕获是幂等基准正确的关键（写后再读即新值，恒真 → 恒不触发）。
4. **part05 R3069 臂同形钩子**（`_zwSsPrevI`），iframe reload 钩子后触发。
5. **runner stylesheet.py 端点**（testharness.rs，beacon_stash 后）：
   per-case `Arc<Mutex<HashMap>>` 计数表；`?id=` token take 语义——
   `count` 参数读计数（take 后不回填，下次读归 0），否则 take+1 回填并
   返回 `text/css` `body {color: red;}`——上游 stylesheet.py take-and-clear
   计数同形。配套上游字节资源 good.css / style.css 落 wpt-data。

## 验证

- **link-multiple-load-events 案 1–4：4F→4P**——幂等门精确（`count=1`
  /`count=0` 断言全过：插入取 1 次、href/rel/type/media 触碰不重取）。
  案 5/6 仍 Fail（见挂账）。
- **link-load-event-alt-001：T→P**；**link-load-error-events 文件级
  Timeout 消**（10s 即收出具体行：Basic load、404、Import 链 success 面、
  https control 等 8 案 Pass）；**link-style-error 三文件事件面活化**
  （48P：nosniff 缺失面正确 error + wrong-Content-Type/broken/no 面按
  ok 判定；12F 为 MIME/nosniff 判定缺失边界——Face B）；**stylesheet-
  bad-mime-type 2P、stylesheet-non-OK-status 1P**（ok:false → error 路径
  实证：wpt_pipe_status 404 仿真 + MIME 面上游同形）。
- **全量 join**（对 s149 /tmp/zw-s149-full.log，(key,state) 对 sort -u）：
  s150 **7148P/847F/88T = 8080 对**（s149 7993 对）；lost 18 全审计
  （4F→P link-multiple 幂等案 + 6F→P urlencoded2 归档观察 + 8 文件级
  Timeout 键消）、**lost 中 P 键 = 0 = 零真 P→F**。
- **urlencoded2 6F→P 归档观察**（不冒认为本轮修复）：与 shim/runner
  改动零机制关联；三跑稳定复现（s150 全量 + 定向 ×2 均 62P/0F）；同族
  text-plain/multipart 12F 在 s149/s150 逐字节不变（encoding 面整体未动
  排除）。假说：s149 全量轮两次撞 test-guard 单进程 4GB 限重跑的内存
  压力窗口伪红（encoding 解码内存敏感路径降级）——挂账观察项，后续
  encoding 流对账时核销。
- **门禁：clippy workspace --all-targets -D warnings 零告警；runner bin
  265/0F；fmt 过**（本轮 .rs 干净；既有 `text_encoding.rs` fmt 漂移为
  encoding 流提交 a828dfac4 遗留，按工作面不重叠纪律不动，仅提及——
  `make test` fmt 前置门会先挡它）。

## s149 单测回归修复（本轮顺带发现，门禁基线闭合）

- **发现**：runner bin 263/1F——`waits_for_active_step_timeout_before_
  accepting_completion` 稳定失败（0.1s 确定性）；干净树（HEAD=s149）同
  红、父提交 6e9f0a5ac 绿 → **s149 引入**，其提交信息「runner bin
  264/0F」声明与事实不符（验证盲区）。
- **机理**：s149 把 probe JS `__zw_harness_complete` 检查前移 timer grace
  之前——completion 已发即立即终态；老 mock 在 async_test 工厂内同步发
  completion、4ms timer 后才发结果行（真实 pinned testharness 无此活
  路径：页级超时强制 COMPLETE 后 `done()`/`step_func_done` 全 no-op）→
  runner 立即收口丢 mock 结果行。
- **裁决**：生产语义正确（s149 主目标由 `harness_probe_is_terminal` 的
  phase==4 臂达成，不依赖前移；前移防的是恒近 pending timer 页面空转，
  s148 91 例实证）——修测试不回滚生产。mock 改为结果行后发 completion
  （真实顺序同形，原断言/契约保留），新增早收形守卫测试
  （`early_completion_with_no_results_yields_file_level_timeout_s149`：
  completion 先发无结果行 → 立即收口 + 文件级兜底 Fail 键，不空转）。
- **验证**：runner bin **265/0F**（264 旧 + 1 新增，零失败）。

## 挂账（本切片更新）

- **Face B：MIME/nosniff 判定矩阵**——link-style-error 系 12F（wrong
  Content-Type + nosniff 应 error、no Content-Type 应 error 的 sniff
  语义）+ data: URL @import 内层解析（Import of nonexistent/non-CSS/
  http 链 10F 主体）+ 跨源 ACAO 判定（shim fetch 通道无 CORS 判定，
  `Load of https:// :9443`）。css.py 端点仿真可配套。
- **Face C：script/render-blocking 时序面**——link-load-event（load 时机
  vs render blocking）、script-unblocked 两案、interactions 组 delay、
  conditionally-block、stylesheet-change/remove-href 系。
- **新记：stylesheet-media-change-no-fetch 1F**——media 属性变更不重取
  （应立即应用新 media 匹配）。本刀幂等门只挂 href；media 变更语义属
  Face B 邻接面。
- **link-multiple 案 5/6**：案 5 需 rel=preload fetch 面 + 缓存复用；
  案 6 页级 10s 时序必然（案 1–4 各吃 ~2.1s race 窗口计 8.4s → 案 5/6
  撞早收；上游 10s 内能过靠 preload 快速 settle）。
- **承袭不变**：document-without-browsing-context adopt/innerHTML 身份
  双表示面（R3031/t8r 代理原件 vs 物化 wrapper 不同源 + adopted 工厂
  对象 cleanup 疑破坏桥表——2F 持平）、meta long × corpus 通道放宽失效
  （s149 记）、details 族 lib flake、promise_test 串行启动面、待决三项
  （css/text 语料资源、green-2x2.png、parser form pointer 缓行）。
- **观察归档：urlencoded2 6F→P**（见验证节——s149 内存压力伪红假说，
  encoding 流对账时核销）。

## 下一步（s151 候选）

①Face B MIME/nosniff 判定矩阵（link-style-error 12F 真兼容面 + css.py
端点仿真，s138/s150 端点先例）；②Face C render-blocking 时序面勘根
（link-load-event / script-unblocked）；③adopt/innerHTML 身份双表示
深水面（document-without-browsing-context 2F）；④rel=preload fetch 面
（link-multiple 案 5 + 缓存复用）。
