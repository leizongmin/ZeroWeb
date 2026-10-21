# M2-S4K — navigation.activation 暴露 + runner variant-meta 展开（2026-10-09）

**通道**: `make testharness-navigation`（test-guard 包裹，TIME_LIMIT=2700）
**全量运行日志**: [2026-10-09-m2-s4k-full-corpus.txt](2026-10-09-m2-s4k-full-corpus.txt)
**前序**: [2026-10-09-m2-s4j-form-submit.md](2026-10-09-m2-s4j-form-submit.md)

## 切片内容（C 类可切片首簇 + runner 基建缺口）

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **`navigation.activation` 暴露**：NavigationActivation 接口对象 + getter（entry = 激活时 entry record——**首次访问定格**，同文档 push/replace 不变；from = null；navigationType = 'push' 缺省）；replaceState 让 activation.entry 孤儿化（index -1 经 per-record index getter 自然成立） | `part02.js` | WPT navigation-activation history-pushState / -replaceState |
| **runner variant-meta 展开**：`case_variants`（uievents 尾簇 7 既有提取器复用）+ `run_any_js_corpus_subdirs_with_helpers_variants`（expand 门仅 navigation corpus 启用；每变体独立运行、case URL 追加 query、case 名带 query 与上游 dashboard 对齐）；其余 corpus 走原路径零变化 | `tests/wpt-runner/src/testharness.rs` | WPT state/* `?method=` 双态（此前 variant-meta 缺口 = S4 记账项） |

## 数字

| corpus 域 | S4J 后 | 本轮 | Δ |
|---|---|---|---|
| navigation-api 全域 | 175/230 = 76.1% | **190/255 = 74.5%** | +15 Pass（分母 +28：variant 展开 + ordering `?currententrychange` 变体面首次运行） |
| └ navigation-activation | 0/4 | **2/4**（history-pushState/-replaceState；bfcache ×2 = dispatcher infra 挂账） | — |
| └ state `?method=` 变体 | 2 Fail（无 variant 面） | **4 Pass** | — |
| └ ordering `?currententrychange` 变体 | 未运行 | 首次运行（数 Pass/数 Fail——新覆盖面） | — |
| 全量 | 314 P / 442 = 71.0% | **329 P / 476 = 69.1%** | +15 Pass（分母 +34 变体展开；占比回落系分母扩张非退步） |

**零回归**：全量 per-subtest 精确 diff——既有 case 状态零变化；新增条目 = variant 展开 + activation 两案。

## 挂账/后续

- ordering `?currententrychange` 变体 Fail 簇（~10 案）：CCE 事件序 Recorder 断言——C 类
  可切片余项（dispose/activation 深簇同域）。
- activation-after-bfcache ×2：cross-frame dispatcher infra（common/dispatcher 未拉取）——
  M3/runner 形态挂账。
- state same-document-away-and-back ×2：traverse navState 还原语义 + location-api 形态——
  C 类余项。

## 质量门禁

- `make test`：全绿（结果见 master 质量门禁行）。
- `cargo clippy --workspace --all-targets -- -D warnings` + fmt：零 warning/diff。
