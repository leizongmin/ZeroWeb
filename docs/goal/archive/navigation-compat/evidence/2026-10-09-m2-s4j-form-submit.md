# M2-S4J — form submit navigate 事件面（2026-10-09）

**通道**: `make testharness-navigation`（test-guard 包裹，TIME_LIMIT=2700）
**全量运行日志**: [2026-10-09-m2-s4j-full-corpus.txt](2026-10-09-m2-s4j-full-corpus.txt)
**前序**: [2026-10-09-m2-s4i-host-activation.md](2026-10-09-m2-s4i-host-activation.md)

## 切片内容（M4 评估 C 类首簇——form submit → navigate 事件，5T 收口）

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **`_zwFormSubmitNavigate` navigate 事件先行**：entry list/URL 计算后 fire navigate（sourceElement = **submitter \|\| form**、formData = POST 时 FormData(entry list)/GET null、navigationType = **瞬态激活 → push、同 URL（去 query）→ replace、异 URL → push**、canIntercept=true）；preventDefault → cancel（navigateerror 微任务派发）；intercept → 同文档提交链（entry push + CCE + handler 链） | `part04.js` + `part02.js`（formData 线程） | WPT navigate-form（POST formData/sourceElement=form/replace）/-get（GET formData null/url+'?'）/ -get-sourceElement（requestSubmit sourceElement=submitter）/-userInitiated（test_driver click → push+userInitiated+formData.get） |
| **POST 入面**：旧 `method=post 零投递` 早退拆除——navigate 事件照派；**未拦截 POST 仍零投递**（host 导航契约无 method/body 面——R-baidu5 排除契约维持，回归钉 test_form_submit_default_navigation_r_baidu5 复绿）；dialog 排除维持；POST 不做 query 变异 | `part04.js` | 同上 + js_dom_bridge_tests part30 ⑦ |
| **GET 空 entry list 追加 `?`**：query 突变后无 `?` → 追加（spec parse-driven query set 保留空 query 的 `?`） | `part04.js` | WPT navigate-form-get「destination.url = location.href + '?'」 |
| **navigateerror 微任务派发**：cancel 路径（`_navCancelNavigation`）与 form 默认（host 替换文档）路径的 navigateerror 从同步改 **queueMicrotask**——requestSubmit()/submit() 同步返回后测试才挂 onnavigateerror 监听（同步派发必漏）；入队先于 promise reject → ordering 序 [navigateerror, committed rejected, ...] 保持 | `part02.js` + `part04.js` | WPT navigate-form-requestSubmit（4 次提交各 await onnavigateerror）/ ordering navigate-canceled 序维持 |
| **runner click 补提交入口**：合成 click 无 JS 默认动作面——Activate 分发后按按钮判定（JS 侧 tagName/type + form 归属）补调 `__zwNavFormRequestSubmit(form, submitter)`（part04 新全局 → `_zwRunFormSubmit`） | `tests/wpt-runner/src/testharness.rs` + `part04.js` | WPT navigate-form-userInitiated |

## 定位过程记录（两处非显然根因）

1. **`__zw_request_navigate` 缺失早退**：`_zwFormSubmitNavigate` 首行 host 回调缺失即 return
   ——testharness 沙箱无该回调 → 提交静默零事件（probe 实证 submitted 无 fire）。guard 后移
   至默认分支 typeof 判定。
2. **同步 navigateerror 漏听**：requestSubmit 同步返回后测试才 `onnavigateerror = r`——同步
   派发时监听未挂（4 次 await 全悬挂）。改微任务派发（晚于同步段、早于 task，ordering 序不破）。

## 数字

| corpus 域 | S4I 后 | 本轮 | Δ |
|---|---|---|---|
| navigation-api 全域 | 169/230 = 73.5% | **175/230 = 76.1%** | +6 |
| └ navigate-form 族 | 5 Timeout | **6 Pass**（含 navigate-form 既有 Face 全绿） | — |
| 全量 | 308 P / 442 = 69.7% | **314 P / 442 = 71.0%** | +6 |

**零回归**：全量 per-subtest 精确 diff（S4I vs S4J）——Pass→Fail/Timeout 为 0。

## 质量门禁

- `make test`：全绿 **20,255 P / 0 F**（run 2 中 baidu5 POST 投递回归被门禁当场捕获 → 修复
  （未拦截 POST 零投递契约维持）后复跑全绿；run 3 中 renderer s41 确定性钉负载 flake
  solo 复跑即绿——兄弟流 crate，同 S4I 归因先例）。
- `cargo clippy --workspace --all-targets -- -D warnings`：零 warning。
- `cargo fmt --all -- --check` + `git diff --check`：零 diff。
