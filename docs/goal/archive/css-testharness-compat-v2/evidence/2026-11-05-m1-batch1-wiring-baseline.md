# M1 批次 1 接线 + 干跑基线（2026-11-05）

## 1. 接线范围（四件套模板 × 6 目录）

B1 值/属性解析序列化簇首批 6 目录（cssom、css-images、filter-effects、
css-ui、css-borders、css-logical）入常驻通道，模板与一期 M3 扩面一致：

| 件 | 文件 | 内容 |
|---|---|---|
| 薄包装 | `tests/wpt-runner/src/testharness_css_corpus.rs` | 每目录 5 行 `pub fn run_<dir>_cases`（共享底座 `run_css_corpus_dir`），批次 1 注释块 + 中文 `///` 文档（含 M0 分母） |
| 分派 | `tests/wpt-runner/src/main.rs` | 6 match arm + 6 cmd fn（`///` 共享一份批次注释）+ help 6 行 |
| Makefile | `Makefile` | `.PHONY` 12 名 + 6 全量 target（TIME_LIMIT 缺省 3600）+ 6 `-one` 定向 target（缺省 300 + FILTER 透传） |
| 分片脚本 | `tests/wpt-runner/scripts/run-css-testharness-corpus.sh` | 显式映射臂 2 个：`cssom`→`testharness-cssom`、`filter-effects`→`testharness-filter-effects`（上游目录名无 `css-` 前缀，motion 先例；`css-*` 前缀 4 目录走通用规则零改动） |

语料无新增接线：批次目录已由 41 号 sync 脚本对齐 pin（M0 终态 45 目录
missing=0）；Makefile 的 `fetch-wpt-css-testharness` 依赖幂等，不重复拉本批
目录。

验证：`cargo fmt --check` 干净；`cargo check/build --release -p
zero-wpt-runner` 通过；六 target 实跑出汇总（见 §2）。

## 2. 干跑基线（TIME_LIMIT=540，六目录全部完整跑完）

| 目录 | cases | Pass | Fail | Timeout | 通过率 | M0 分母 | 跑到的子测试 |
|---|---:|---:|---:|---:|---:|---:|---:|
| css/cssom | 177 | 1,952 | 1,523 | 4 | **56.1%** | 4,456 | 3,479（78%） |
| css/css-images | 41 | 781 | 2,434 | 0 | **24.3%** | 3,595 | 3,215（89%） |
| css/filter-effects | 33 | 151 | 237 | 0 | **38.9%** | 3,007 | 388（13%） |
| css/css-ui | 119 | 221 | 690 | 5 | **24.1%** | 2,457 | 916（37%） |
| css/css-borders | 79 | 252 | 678 | 0 | **27.1%** | 1,601 | 930（58%） |
| css/css-logical | 86 | 154 | 1,024 | 0 | **13.1%** | 1,448 | 1,178（81%） |
| **合计** | **535** | **3,611** | **6,586** | **9** | **35.7%** | 16,563 | 10,106（61%） |

- transcripts：`.acceptance/css-v2-m1/batch1/`（诊断留档，不入库）。
- M0 分母来自甄别表（wpt.fyi 面口径）；「跑到的子测试」为本通道实际产出
  （runner 只收 `.html/.htm` 且含 `testharness.js` 的案，reference/manual/
  纯 reftest 页不计）——filter-effects 分母大头是 reftest/SVG 面，testharness
  可执行面本就只有 33 案。通过率按本地可执行分母计。

## 3. DC-4 门假设校验

门公式按 B1 全簇 7% 通过率计（一期同类簇取下缘）。批次 1 六目录实测
35.7%（最低 css-logical 13.1%）——**远高于 3% 复评触发线，门假设成立，
不触发留痕复评**。按实测率外推 B1 余 18 目录（分母 7,318）保持保守 7%
假设仍有充足余量（批次 1 已单独贡献约 3,611 passes）。

## 4. M1 余量与下一步

- 批次 1 已入常驻通道；weekly 分片接线核对（DC-2 尾件）随 M2 收口统一做。
- 下一步：B1 余 18 目录同款四件套接线 + 干跑基线（css-text-decor、css-box、
  css-pseudo、css-display、css-mixins、css-syntax、fill-stroke〔需补脚本映射
  臂〕、css-shadow、css-size-adjust、css-content、css-will-change、
  css-color-adjust、css-easing、compositing〔需补映射臂〕、css-counter-styles、
  css-nesting、css-color-hdr、css-env）。
