# M2-S4W — strict 外链 fetch 失败收窄 + navigate() 同 URL replace 面（2026-10-10）

**通道**: `make testharness-navigation`（test-guard 包裹；FILTER 单案最小复现迭代）
**全量运行日志**: [2026-10-10-m2-s4w-strict-narrow-family.txt](2026-10-10-m2-s4w-strict-narrow-family.txt)（终跑）；per-subtest 精确 diff 对 [2026-10-10-m2-s4v-location-reload-face.txt](2026-10-10-m2-s4v-location-reload-face.txt)（S4V 终态）
**前序**: [2026-10-10-m2-s4v-location-reload-face.md](2026-10-10-m2-s4v-location-reload-face.md)

## 切片内容（两独立面 + 记账）

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **strict 门面收窄**：`run_page_scripts_strict` 对外链脚本/模块 **fetch 失败不再 abort**（原 strict `return Err` 使整页 declared=0 Fail）——spec fetch-a-classic-script 失败派 script 元素 error 后**页面继续**；strict 收窄为「内联抛错」（其文档注释本意——WPT runner「空洞通过」闭合目标；非 strict 行为不变） | `webview.rs`（External/ExternalModule 两 fetch 分支） | [fetch a classic script](https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-classic-script)；WPT helpers.js pin/master **双 404** 家族（上游资产缺失非本仓可修） |
| **navigate() 同 URL replace**：history 'auto'（缺省）且目标 URL 等于当前 URL → replace（原恒 push）；显式 `history:'push'` 不改写 | `part02.js`（navigate 方法） | [navigate-to-a-url](https://html.spec.whatwg.org/multipage/document-lifecycle.html#navigate-to-a-url) historyHandling auto 步；WPT same-url-replace-same-document/-cross-document「navigate() to the current URL should replace」vs navigate-history-push-same-url「history:'push' 恒 push」（反向钉） |

**两根因关系**：外链 404 abort 掩盖四个用例的内联真断言（same-url-replace 双案被盖后恒 Fail、
push-same-url/form-submit-and-window-stop 真实即绿）；strict 收窄暴露 same-url-replace 双案
的第二根因（同 URL 恒 push），两 face 同轮收口。

## 数字

| corpus 域 | S4V 后 | 本轮 | Δ |
|---|---|---|---|
| 全量 | 382 P / 476 = 80.3% | **394 P / 476 = 82.8%** | +12（5 案翻绿） |
| navigation-api | 231 / 255 | **234 / 255**（91.8%） | +3 |
| the-history-interface | 48 / 49 | **55 / 56**（98.2%） | +7（007 全案） |

**12 Pass 新增 / 5 案 Fail→Pass / 零 Pass 回归**（per-subtest 精确 diff，Pass 集纯增）：
- navigate-history-push-same-url + form-submit-and-window-stop（strict 收窄直接收口——
  后者为 S4T 挂账「helpers.js 资产偏斜」族，撤销该挂账）；
- same-url-replace-same-document + -cross-document（同面暴露 → navigate() 同 URL replace
  面收口，同撤 S4T 同族挂账）；
- **007.html 全案 7 子测试**（S4T 前挂账「外部脚本基建」——实为外链脚本 404 abort 掩盖，
  strict 收窄后内联子测试真实全绿，撤销挂账；008 仍 Timeout 维持记账）。

## 记账与计数勘误

- **navigate-history-push-not-loaded 余 1F 定因**：runner 脚本阶段 `document.readyState`
  恒 "complete"（`__zwReadyState` 状态宿未注入——t8m 在 renderer/tab_scripts 路径已置
  loading→interactive→complete 三态过渡，testharness runner 路径缺同款）。**不并入本轮**：
  全语料 21 案断言 document.readyState（css/selectors、fetch、CSP、xhr 等跨 goal 域），
  readyState 过渡是共享 runner 语义变更，须独立轮次跨域测量后落地。
- **计数口径勘误（复核 S4V txt）**：S4V 标题「navigation-api 93.3%→93.7%」与其 txt 不符
  （txt 原始行 231/255 = 90.6%；疑为中途日志读数）——其 +1 案判定
  （navigate-destination-getState-reload 翻绿）确凿。本轮起百分数统一按 evidence txt 原始
  Pass 行计数，S4T/S4U 标题百分数同口径偏高（原始 230/255 = 90.2%），案数增量不受影响。

## 质量门禁

- `cargo clippy --workspace --all-targets -- -D warnings`：零 warning（webview.rs 变更轮）。
- `cargo fmt --all -- --check`：零 diff；`git diff --check`：零问题。
- `make test`：**20,363 P / 0 F**（首跑 17,699 P 后 renderer bounded_wait s41 一 F——兄弟流
  全量并发争用（本日第 2 例，先例 S4U r2946）；solo 复跑 1.27s 过后整轮重跑全绿）。
