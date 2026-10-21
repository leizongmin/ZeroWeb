# M2-S2 — traverse 事件序切片（2026-10-08）

**通道**: `make testharness-navigation`（test-guard 包裹）　**WPT pin**: `3159769338`（goals/lib.sh 同 pin）
**前序**: [2026-10-08-m2-s1-location-interface.md](2026-10-08-m2-s1-location-interface.md)

## 切片内容（M1 gap 归类 #3：traverse 事件序）

| 改动 | 文件 | spec 锚点 |
|---|---|---|
| fragment navigation（`location.hash = v`）派 popstate **同步**（setter 返回前）+ hashchange 仍异步（不同 task source） | `crates/engine/src/js_dom_shim/part02.js` `_setLocationHash` | HTML spec URL and history update steps；WPT event-order/before-load-hash、pushState-inside-popstate |
| back/forward/go 派发的 popstate/hashchange 标 `__zwTrusted`（isTrusted true，R312 内部口） | 同上 `_hist_dispatchPopState` | WPT popstate_event / hashchange_event `assert_true(e.isTrusted)` |
| hash setter / location.replace 触发的 hashchange 同标 `__zwTrusted` | 同上 `_pushHistNav` / `_replaceHistNav` | 同上 |
| PopStateEvent +`hasUAVisualTransition`（init 布尔，缺省 false） | `crates/engine/src/js_dom_shim/part05.js` | HTML spec PopStateEvent；WPT PopStateEvent |
| PopStateEvent 接口对象非 new 调用抛 TypeError（UIEventCtor109 wrapper 同款） | 同上 | WebIDL 接口构造器无 [[Call]]；WPT PopStateEvent 'called as normal function' |
| `history.scrollRestoration` 改 **per-entry** 属性（getter/setter 读写当前 entry；非法值静默忽略；pushState 克隆 mode） | `crates/engine/src/js_dom_shim/part02.js` | HTML spec session history entry scroll restoration mode；WPT scroll-restoration-basic / -navigation-samedoc |
| runner classic 脚本顶层 `let` 导出改 **accessor 转发**（旧值快照——闭包再赋值后跨脚本读过期） | `crates/engine/src/js_dom_bridge/script_gen.rs` | R201 var accessor 同款；触发案例 = before-load-hash 族 `let popstatesLeft` 闭包递减跨脚本不可见 |

## runner fidelity 缺口定位记录（before-load-hash 归因链）

`script_run_classic_page` 每脚本走间接 eval（R147/R198），顶层 `let` 以值快照导出
`globalThis.NAME=NAME`。before-load-hash 的 `let popstatesLeft = 1` +
`window.onpopstate = () => popstatesLeft--`：handler 闭包递减 **eval 局部绑定**，
后续脚本经 globalThis 属性读到的仍是快照 1 → 同步 popstate 断言恒假。
probe C 定界（handler 已跑、`window._handlerSaw=0`、绑定读仍 1）→ 导出机制而非
派发机制缺口。let 改 accessor 转发（lexical binding 恒不泄漏 global，无 var 的
accessor 自递归形态——R201 is_strict 注记）；const 保持值快照（不可重赋）。

## 数字

| corpus 域 | M1 基线 | 本轮 | Δ |
|---|---|---|---|
| traversal/history-traversal | 28/45 = 62.2% | 42/45 = **93.3%** | +31.1pp |
| history/the-location-interface | 31/36 = 86.1%（S1 后） | 31/36 = 86.1% | 0 |
| event-order 子目录 | 4/13 | **12/13** | +8 子测试 |

event-order 余 1 失败 = pagereveal/order-in-new-document-navigation（跨文档
pagereveal 事件面，M2 ③ 跨文档切片 / pagereveal 独立评估）。

history-traversal 余 2 失败 = pagereveal/order-in-bfcache-restore（
`/common/dispatcher/` 多 global 通信 helper——goal 域外 infra，M1 已记账）+
scroll-restoration-fragment-scrolling-samedoc（跨文档 manual 模式 fragment 滚动
抑制——runner 单文档近似下 hash 导航仍滚锚，跨文档面缺口记账）。

## 质量门禁

- `make test`：全绿——20,180 P / 0 F（workspace 腿 + quickjs clippy 腿 + renderer 两腿）
- `cargo clippy -p zero-wpt-runner -p zero-engine --all-targets -- -D warnings`：零 warning
- `cargo fmt --all -- --check`：零 diff
- 全量语料：[2026-10-08-m2-s2-full-corpus.txt](2026-10-08-m2-s2-full-corpus.txt)
  （332 案 428 子测试 125 Pass = 29.2%；零基线回归——M1 基线 81 Pass 案全保持 Pass；
  总量对比 S1 轮 111 P）
