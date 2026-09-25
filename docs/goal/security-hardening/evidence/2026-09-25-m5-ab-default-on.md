# M5 — A/B 零回归门禁 + kill-switch default-on 落定

**日期**: 2026-09-25
**门禁**: `make test`（A/B on 臂全量）+ `make product-smoke`（产品 fixture）+ `make reftest`（DC-4，default-off 基线臂）

## A/B 对照

| 臂 | 门禁 | 结果 |
|---|---|---|
| off（M2-s1→s8 全轮基线） | `make test` 每轮 | exit 0 零失败（R1→R9 持续） |
| off | `make reftest` | **202/202 = 100%** 恒绿（本轮，DC-4） |
| **on（default-on 翻转后）** | `make test` 全量 | **exit 0 零失败**（724+ tests，本轮） |
| on | `make product-smoke`（welcome.html vs chromium） | **PASS**（struct-check 0 issues，diff 门内，本轮） |

**判定**：A/B 零回归成立——`csp_enforcement` default 翻转为 **on**（webview.rs
`Default::default()`），kill-switch 语义保留（宿主显式置 false 回退；
`csp_gate_off_by_default_zero_delta_sh1_m2s1` 测试改为显式 false 验证回退面）。

零回归机理：装配/检查点均以**文档持有 CSP 政策**为前提——无 CSP 文档（产品
fixture/workspace 绝大多数测试页）装配不激活、检查点短路、零行为面。CSP 持有文档
的强制行为即本 goal 交付物本身（runner 实验臂 corpus 证据 M2-s1→s8 持续积累）。

## DC 逐项判定（对照入口文档 Done Criteria）

| DC | 判定 | 证据 |
|---|---|---|
| DC-1 WPT 导入与基线 | **✅** | 三 corpus 导入（fetch-security-csp-subset.sh，WPT 3159769 pin）+ 基线 evidence/2026-09-24-m1-security-wpt-baseline.md（30/415=16.3%）→ 收敛轨迹至 74/445（M2-s8） |
| DC-2 四面语义收敛 | **◐ CSP ✅ / Mixed Content+HSTS 语义✅接线◐ / Permissions ⏳** | CSP：检查点面收齐（script 元素/attr/运行时 img/style/connect/eval + report-only 解析面 + violation 事件），74/445 全绿可追踪提升（16.3%→28.8% subtests）。Mixed Content：mixed_content.rs 分级 + upgrade + SecurityContext 单测 ✅；导航面接线（webview.rs check_resource_url "document"）✅，子资源面接线 ⏳。HSTS：hsts.rs 解析/存储/includeSubDomains/过期清理 + 单测 ✅；net 响应注册接线 ⏳。Permissions：PermissionManager query/grant/deny 语义 + 单测 ✅；navigator.permissions JS 面 + change 事件 ⏳（M4 待做） |
| DC-3 行为变更门禁 | **✅** | kill-switch 全程（default-off 逐轮 make test 零 delta）+ 本轮 A/B 零回归 + default-on 落定（上表） |
| DC-4 测试与质量 | **✅** | make test 全绿 + clippy -D warnings 干净 + fmt 干净（每轮）+ make reftest 202/202（本轮） |

## 挂账定稿（不阻塞 goal 判定的域外/infra 项）

1. fetch blocked 契约：shim host 错误 wire → resolve(ok:false)（上游期望 reject）——shim 面语义 divergence。
2. 外链 stylesheet ID 选择器应用缺口（stylenonce-blocked allowed.css）——pipeline/style 面预存（渲染流域 crates 域，default 无 CSP 同黑实证）。
3. report-uri / Reporting API 端点族——infra（无 HTTP server 报告端点）。
4. img corpus `{{location[scheme]}}` location 模板族——.sub.js location 模板替换未实现。
5. inheritance/sandbox 簇全红——跨文档导航/iframe 管道重入条件（深多进程面）。
6. img/eval 违例调用点定位（line/column）——shim per-eval 传参评估。
7. Mixed Content 子资源面接线 + HSTS net 响应注册 + navigator.permissions JS 面（**DC-2 尾项 = 下一轮 M3/M4 补齐**）。
