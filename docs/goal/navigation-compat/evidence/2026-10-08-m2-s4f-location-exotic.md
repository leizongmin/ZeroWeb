# M2-S4F — Location exotic 内部方法面（2026-10-08）

**通道**: `make testharness-navigation`（test-guard 包裹，TIME_LIMIT=2700）
**全量运行日志**: [2026-10-08-m2-s4f-full-corpus.txt](2026-10-08-m2-s4f-full-corpus.txt)
**前序**: [2026-10-08-m2-s4e-focus-reset.md](2026-10-08-m2-s4e-focus-reset.md)

## 切片内容（master 下一步计划 ①——⑤ exotic 内部方法面）

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **Location 对象 Proxy 包裹**（exotic 内部方法承载——plain object 无法自定义）：[[PreventExtensions]] 恒 false（Object.preventExtensions 抛 TypeError / Reflect 返 false）+ [[SetPrototypeOf]] 不可变（非原 prototype 返 false——Object.setPrototypeOf/__proto__ 抛 TypeError、Reflect 返 false；原 prototype no-op 返 true——`__proto__` set 经原型链 setter 以 proxy 为 receiver 回落本 trap）+ 新 own 属性 [[DefineOwnProperty]]/[[Set]] 恒 false（LegacyUnforgeable 面，已有属性 getter/setter 照常透传） | `crates/engine/src/js_dom_shim/part01.js`（`_makeLocation` 收口） | spec location-preventextensions / location-setprototypeof / location-defineownproperty；WPT location-prevent-extensions / location-prototype-setting-same-origin（[S4F 消掉 M2-S1 遗留 FIXME](https://html.spec.whatwg.org/multipage/nav-history-apis.html#location-preventextensions)） |
| **实例 prototype 链 Location.prototype**：接口对象装配提前（`instanceof Location` 面 + immutable-prototype「原 prototype」基准） | `part01.js` | 同上 |
| **helper 资产**：/common/test-setting-immutable-prototype.js（fetch 脚本 + runner NAVIGATION_ABSOLUTE_HELPERS 注册——原注释「plain object 未实现不拉」解除） | `tests/wpt-runner/scripts/goals/60-navigation-compat.sh` + `tests/wpt-runner/src/testharness.rs` | 同上 |

## 数字

| corpus 域 | S4E 后 | 本轮 | Δ |
|---|---|---|---|
| the-location-interface | 33/43 | **41/43 = 95.3%** | +8（prevent-extensions ×2 + prototype-setting ×6——原 1 Timeout 整案转可执行） |
| 全量 | 220 P / 435 = 50.6% | **230 P / 435 = 52.9%** | +10 |

**零回归**：全量 per-subtest 精确 diff——变更全部落在 location exotic 簇（Proxy 包裹对全语料
无副作用实证）。

## 余 2F 定性

- `location_port`：断言依赖页面 URL **带端口**（runner page URL `https://wpt.test/...` 无端口
  → 测试内 host 冒号解析退化）——M1 已记账的 runner 形态缺口，非语义缺口。
- `create-script-set-location`：load 期脚本追加 location.href 的 history entry 面——跨文档/
  load 序域，归 ③ 跨文档导航簇记账。

## 质量门禁

- `make test`：全绿 **20,209 P / 0 F**。
- `cargo clippy --workspace --all-targets -- -D warnings`：零 warning。
- `cargo fmt --all -- --check` + `git diff --check`：零 diff。
