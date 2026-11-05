# M1 批次 2 接线 + 干跑基线（2026-11-05）

## 1. 接线范围（四件套模板 × 18 目录）

B1 值/属性解析序列化簇余 18 目录入常驻通道，四件套与批次 1 同款：

| 件 | 文件 | 内容 |
|---|---|---|
| 薄包装 | `tests/wpt-runner/src/testharness_css_corpus.rs` | 每目录共享底座 `run_css_corpus_dir` 薄包装 ×18 + 批次 2 注释块 + 中文 `///` 文档（含 M0 分母） |
| 分派 | `tests/wpt-runner/src/main.rs` | 18 match arm + 18 cmd fn（`///` 共享一份批次注释）+ help 18 行 |
| Makefile | `Makefile` | `.PHONY` 36 名 + 18 全量 target（TIME_LIMIT 缺省 3600）+ 18 `-one` 定向 target（缺省 300 + FILTER 透传） |
| 分片脚本 | `tests/wpt-runner/scripts/run-css-testharness-corpus.sh` | 显式映射臂 +2：`fill-stroke`、`compositing`（上游目录名无 `css-` 前缀，cssom/filter-effects 先例）；`css-*` 前缀 16 目录走通用规则零改动 |

语料无新增接线：批次目录已由 41 号 sync 对齐 pin（M0 终态 45 目录
missing=0）；`fetch-wpt-css-testharness` 依赖幂等。

验证：`cargo fmt --check` 干净；`cargo build --release -p zero-wpt-runner`
通过；18 个全量 target 经 Makefile + 分片脚本端到端实跑出汇总（见 §2）——
接线即被基线跑批本身验证。

## 2. 干跑基线（TIME_LIMIT=540，18 目录全部完整跑完，无截断）

| 目录 | cases | Pass | Fail | Timeout | 通过率 | M0 分母 | 跑到的子测试 |
|---|---:|---:|---:|---:|---:|---:|---:|
| css/css-text-decor | 48 | 101 | 334 | 0 | **23.2%** | 1,376 | 435 |
| css/css-box | 83 | 133 | 344 | 0 | **27.9%** | 1,063 | 477 |
| css/css-pseudo | 56 | 85 | 590 | 0 | **12.6%** | 865 | 675 |
| css/css-display | 33 | 80 | 299 | 0 | **21.1%** | 567 | 379 |
| css/css-mixins | 43 | 110 | 410 | 1 | **21.1%** | 618 | 521 |
| css/css-syntax | 43 | 123 | 309 | 0 | **28.5%** | 444 | 432 |
| css/fill-stroke | 4 | 0 | 9 | 0 | **0.0%** | 409 | 9 |
| css/css-shadow | 90 | 48 | 305 | 2 | **13.5%** | 377 | 355 |
| css/css-size-adjust | 5 | 6 | 14 | 0 | **30.0%** | 215 | 20 |
| css/css-content | 9 | 58 | 154 | 0 | **27.4%** | 253 | 212 |
| css/css-will-change | 5 | 20 | 153 | 0 | **11.6%** | 173 | 173 |
| css/css-color-adjust | 12 | 23 | 49 | 0 | **31.9%** | 171 | 72 |
| css/css-easing | 12 | 36 | 120 | 0 | **23.1%** | 167 | 156 |
| css/compositing | 18 | 66 | 63 | 0 | **51.2%** | 167 | 129 |
| css/css-counter-styles | 11 | 57 | 61 | 0 | **48.3%** | 158 | 118 |
| css/css-nesting | 22 | 14 | 103 | 0 | **12.0%** | 126 | 117 |
| css/css-color-hdr | 4 | 13 | 40 | 0 | **24.5%** | 116 | 53 |
| css/css-env | 10 | 24 | 18 | 0 | **57.1%** | 53 | 42 |
| **合计** | **508** | **997** | **3,375** | **3** | **22.8%** | 7,318 | 4,375 |

- transcripts：`.acceptance/css-v2-m1/batch2/`（诊断留档，不入库）。
- 分母口径与批次 1 相同：「跑到的子测试」为本通道实际产出（runner 只收
  `.html/.htm` 且含 `testharness` 引用的案），M0 分母为 wpt.fyi 面口径；
  fill-stroke / compositing / counter-styles 上游主体是 reftest 面，
  testharness 可执行面小属甄别预期（M0 §3 形态要点）。
- 通过率按本地可执行分母计；fill-stroke 0/9 为单目录低分母（9 子测试）
  现象，SVG fill/stroke 序列化面在引擎未实现——不改变 B1 簇整体判断。

## 3. DC-4 门假设校验（批次 2 后）

- 批次 2 整体 22.8%（最低非零分母目录 fill-stroke 0%），远高于 B1 3%
  复评触发线——**门假设成立，不触发留痕复评**。
- 门公式按 B1 全簇 7% 下缘计（B1 池 23,882 → 门增益 1,672）：批次 1+2
  干跑实测合计 4,608 passes（3,611 + 997），批次 2 单独贡献 997——B1 簇
  实际通过子测试已超整簇门增益份额，DC-4 ≥ 20,500 门不动。

## 4. M1 余量与下一步

- 批次 2 已入常驻通道；weekly 分片接线核对（DC-2 尾件）随 M2 收口统一做。
- B1 24 目录全部入账完毕。下一步：M2 = B2 规则/条件/级联 7 目录
  （css-conditional、mediaqueries、css-cascade、CSS2、css-forms、
  css-link-params、css-font-loading，分母 9,751〔CSS2 折 html 面 ~300〕）
  同款四件套接线 + 干跑基线。
