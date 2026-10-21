# M2-S4U — window.close 卸载事件面 + javascript: 锚执行面（2026-10-10）

**通道**: `make testharness-navigation`（test-guard 包裹；FILTER 单案最小复现迭代）
**全量运行日志**: [2026-10-10-m2-s4u-window-close-jsurl.txt](2026-10-10-m2-s4u-window-close-jsurl.txt)（终跑）；per-subtest 精确 diff 对 [2026-10-10-m2-s4t-pushstate-face.txt](2026-10-10-m2-s4t-pushstate-face.txt)（S4T 终态）
**前序**: [2026-10-10-m2-s4t-pushstate-face.md](2026-10-10-m2-s4t-pushstate-face.md)

## 切片内容

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **window.close()**：此前全缺（TypeError 中断脚本）——headless in-memory 近似：派 `beforeunload`（cancelable）→ `unload` 两事件后不真关宿主（真关即毁 runner，documented 限制） | `part01b.js`（alert/confirm/prompt/open/print/stop 同区） | [dom-window-close](https://html.spec.whatwg.org/multipage/window-object.html#dom-window-close)（close 触发 unload document——beforeunload 先于 unload）；WPT prompt-and-unload-script-closeable「onload close() → onbeforeunload → onunload」 |
| **javascript: URL 锚激活**：`_navAnchorNavigate` chokepoint 前置 javascript: 判定——间接 eval 于**全局作用域**执行脚本体（不走 navigate 事件/host 交接）；**_defer 微任务执行** | `part02.js` | [the javascript: scheme](https://html.spec.whatwg.org/multipage/browsing-the-web.html#javascript-protocol);WPT javascript-url-global-scope「click javascript: 锚 → 当前全局作用域执行」；navigate-to-javascript「navigate event does not fire for javascript: URL」同源面 |

## 根因注记（javascript: 同步 eval 时序）

首版同步 eval 仍超时——真案的 `t.done()` 闭包变量 `t` 在 `async_test` 构造期（click 同步栈内）
尚未赋值，同步 eval 必 TypeError。真实浏览器于**导航 task** 中执行 javascript: 脚本（不在
click 栈内），`t` 已赋值。改 `_defer`（微任务，沙箱 task 近似）后收口。FILTER= 单案最小
复现 + 临时探针页（用后即删）定位。

## 数字

| corpus 域 | S4T 后 | 本轮 | Δ |
|---|---|---|---|
| 全量 | 379 P / 476 = 79.6% | **381 P / 476 = 80.0%** | +2（首次破 80%） |

**2 翻 Fail/Timeout → Pass，零回归**（per-subtest 精确 diff，F/T 集合恰删 2 行零新增）：
prompt-and-unload-script-closeable + javascript-url-global-scope。navigation-api 域计数
不变（两案均在 html/browsers 域）。

## 质量门禁

- `make test`：**20,362 P + 1 已知 load-flake**（renderer r2946 onload 时序 pin——姊妹
  crate、与 shim 无关；solo 复跑即绿，先例 S4L/S4M/S4Q）。
- clippy/fmt：本轮 diff 纯 JS（part01b.js/part02.js）+ 文档，无 `.rs` 变更（S4R 片
  clippy -D warnings 干净、fmt 零 diff 维持）。
- `git diff --check`：零问题。
