# M2-S1 — Location 接口语义切片（2026-10-08）

**通道**: `make testharness-navigation`（test-guard 包裹）　**WPT pin**: `3159769338`
**全量运行日志**: [2026-10-08-m2-s1-full-corpus.txt](2026-10-08-m2-s1-full-corpus.txt)（332 案 428 子测试 Pass/Fail/Timeout + 7 NotRun）

## 切片内容（M1 gap 归类 #2：Location 接口语义）

| 改动 | 文件 | spec 锚点 |
|---|---|---|
| Location 对象按 [LegacyUnforgeable] 面重建——接口成员落实例 own property（attributes → own accessor `{enumerable, configurable: false}`；operations → own data `{writable: false, enumerable: true, configurable: false}`） | `crates/engine/src/js_dom_shim/part01.js` | WebIDL [LegacyUnforgeable]；WPT location-non-configurable-toString-valueOf |
| `port` getter/setter（旧整体缺席） | 同上 | WPT location_port |
| stringifier `toString` → own data property（值 = getter 函数 + 内部 symbol brand check——this 非 Location 抛 TypeError） | 同上 | WebIDL §es-stringifier；WPT location-stringifier |
| `valueOf`（own data = Object.prototype.valueOf）/ `Symbol.toPrimitive`（own undefined）三旗全 false | 同上 | HTML spec location-defineownproperty |
| `window.Location` 接口对象（callable，调用 TypeError；prototype 无 own toString/valueOf） | 同上 | WPT location-prototype-no-toString-valueOf |
| href 写侧 / assign / replace 解析失败 → SYNTAX_ERR DOMException（`globalThis.DOMException` 优先，R384 wrong-global 先例） | `crates/engine/src/js_dom_shim/part02.js` | HTML spec location-href-setter/location-assign/location-replace |
| `new Document()` location 共享身份访问器（getter/setter dedup + setter no-op） | `crates/engine/src/js_dom_shim/part03.js` | WPT document_location getter/setter dedup |
| runner 绝对路径 helper 通道 + `/common/stringifiers.js` 内联 | `tests/wpt-runner/src/testharness.rs`、`scripts/goals/60-navigation-compat.sh` | timing `TIMING_ABSOLUTE_HELPERS` 同款语义 |

## 数字

| corpus 域 | M1 基线 | 本轮 | Δ |
|---|---|---|---|
| history/the-location-interface | 13/31 = 41.9% | 31/36 = **86.1%** | +44.2pp |
| history/the-history-interface | 40/49 = 81.6% | 40/49 = 81.6% | 0（零回归） |
| traversal/history-traversal | 28/45 = 62.2% | 28/45 = 62.2% | 0（零回归） |
| traversal/navigating-across-documents | 0/35 = 0% | 2/42 = 4.8% | 语料增长混杂（见下） |

同一 corpus（309 案口径）逐案对照 M1 JSON：无案由 Pass/Fail 翻 Timeout；location 域外唯一行为变化是 replace-before-load 20 案 Fail→Timeout（见「语料增长混杂」）。

## 余下 5 失败归类（the-location-interface）

1. **exotic 内部方法面 ×3**（location-prevent-extensions ×2 + location-prototype-setting-same-origin ×1）：spec location-setprototypeof / location-preventextensions——Location 为 exotic object，[[SetPrototypeOf]] 非原 prototype 恒返 false、[[PreventExtensions]] 恒返 false。plain object 无自定义内部方法（与 immutable-prototype 同族）；后者的 helper `/common/test-setting-immutable-prototype.js` 也未拉。已在 part01.js 标 FIXME 留 M2 后续切片（评估 Proxy 或引擎层 exotic 支持，影响所有 [LegacyUnforgeable] 接口建模）。
2. **runner 环境 ×1**（location_port）：runner 页面 URL 恒 `https://wpt.test/<case>`（无端口），而该用例从 href 手工抽取端口段并断言 `location.port` 相等——真 WPT 服务在 `:8000`，本用例在无端口 URL 下结构性不可过。location.port 语义本身正确（缺省返 `''`）。
3. **Navigation API 面 ×1**（create-script-set-location）：依赖 `navigation.addEventListener` + testdriver click——Navigation API 独立切片（M2 ④）。

## 语料增长混杂（本轮 fetch 幂等续拉落地，非本切片产物）

2026-10-07 fetch 恢复后入库：scroll-to-fragid（20 案执行，8 Pass）/ unloading-documents（1 案）/ the-iframe-element 130 .html（skip 规则漏网 2 案执行 0 Pass——启发式只查 `<iframe`/createElement 字面量，sandbox 导航两案经 helper 间接使用）/ replace-before-load 嵌套 helper 补齐（20 案由「helper 缺失快速 Fail」翻「testdriver 等真实用户交互 Timeout」——真缺口显形，单文档 runner 无交互注入，每案 60s 计入全量运行时长）。

## 质量门禁

- `make test`：全绿（含 S2 切片合并态，2026-10-08 跑）——20,180 P / 0 F（workspace 腿 +
  quickjs clippy 腿 + renderer 两腿，日志 `/tmp` 不持久，结论记录于此）
- `cargo clippy -p zero-wpt-runner -p zero-engine --all-targets -- -D warnings`：零 warning
- `cargo fmt --all -- --check`：零 diff
- 全量语料复跑（S2 合并态）：零基线回归（M1 基线 81 Pass 案全保持 Pass），
  见 [2026-10-08-m2-s2-full-corpus.txt](2026-10-08-m2-s2-full-corpus.txt)
